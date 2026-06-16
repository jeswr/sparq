//! **Source selection**: for each triple pattern of a BGP, which sources can
//! contribute, and at what estimated cardinality.
//!
//! Two complementary techniques, both reading only the served [`SourceDescriptor`]s:
//!
//! ## HiBISCuS-style authority/prefix pruning
//!
//! A source is *pruned* for a pattern only when its capability set makes a contribution
//! **impossible**. Three prune rules, each firing only on positive evidence:
//!
//! 1. **Bound-predicate prune** — when the pattern's predicate is a bound IRI and the
//!    source's predicate capability set (its `void:propertyPartition`s, which *are* the
//!    complete set of predicates it holds) does not contain it. A source with no triple
//!    using predicate `p` cannot match `?s p ?o` — provably.
//! 2. **Authority prune** (subject / object positions) — when a position is a bound IRI
//!    whose authority is *not* in the source's authority capability set, AND that set is
//!    declared **complete** ([`SourceDescriptor::authorities_complete`]). A
//!    VoID-parsed descriptor has an *incomplete* authority set (it sees only
//!    predicate/class authorities, never instance authorities), so this rule is
//!    **disabled** for it — never pruning on partial knowledge.
//! 3. **Bound-class prune** — when the pattern is `?x rdf:type C` with `C` a bound IRI
//!    and the source declares class partitions but none for `C`. (Only fires when the
//!    source declares *some* class partition — an absent class section means "unknown",
//!    not "no instances".)
//!
//! ### Recall-safety invariant (the load-bearing property)
//!
//! > **If a source `S` could return any binding for a pattern `tp`, [`select_sources`]
//! > retains `S` for `tp`.**
//!
//! Equivalently: a source is dropped for a pattern only when the descriptor *proves* it
//! holds no matching triple. Every prune rule fires only on positive evidence of
//! non-contribution; on any uncertainty (unknown predicate set membership impossible —
//! the predicate set is complete; incomplete authority set; absent class section) the
//! source is **kept**. The cardinality estimate never causes a prune: a source with an
//! estimate of 0 is still retained (estimates are lossy; only capability evidence
//! prunes). This is exactly HiBISCuS's design goal — maximise pruning subject to never
//! losing a result. Proven by the `recall_safe_*` tests below.
//!
//! ## CostFed-style skew-aware cardinality
//!
//! For each retained (pattern, source) pair, the cardinality is estimated from the
//! served per-predicate statistics — `void:triples` and the per-subject average
//! multiplicity — **not** a uniform-distribution guess, so per-predicate skew is
//! preserved. Bound subject/object positions divide the predicate's triple count by its
//! distinct-subject / distinct-object count respectively (CostFed's selectivity buckets).
//!
//! [OPUS-4.8] sq-a35t.

use crate::descriptor::SourceDescriptor;
use crate::pattern::{Bgp, Term, TriplePattern};

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// One source retained for a pattern, with its estimated contributed cardinality.
#[derive(Debug, Clone)]
pub struct SourceCandidate {
    /// Index into the `sources` slice passed to [`select_sources`].
    pub source: usize,
    /// CostFed-style estimated triples this source contributes to the pattern.
    pub estimated_cardinality: f64,
}

/// The sources retained for one triple pattern (by [`select_sources`]).
#[derive(Debug, Clone)]
pub struct PatternSources {
    /// Index of the pattern within the BGP.
    pub pattern: usize,
    /// Retained sources, ascending by source index (deterministic).
    pub candidates: Vec<SourceCandidate>,
}

impl PatternSources {
    /// Total estimated cardinality of the pattern across all retained sources (the
    /// per-pattern input size the join planner starts from — a union over sources).
    pub fn total_cardinality(&self) -> f64 {
        self.candidates
            .iter()
            .map(|c| c.estimated_cardinality)
            .sum()
    }

    /// Whether any source was retained for this pattern.
    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }
}

/// Source selection for every pattern of `bgp` against `sources`.
///
/// Returns, per pattern, the retained sources and their CostFed cardinality estimates.
/// **Recall-safe** (see the module docs): a source is pruned for a pattern only when
/// the descriptor proves it cannot contribute. Deterministic: candidates are ordered by
/// source index.
pub fn select_sources(bgp: &Bgp, sources: &[SourceDescriptor]) -> Vec<PatternSources> {
    bgp.patterns
        .iter()
        .enumerate()
        .map(|(pi, tp)| {
            let mut candidates: Vec<SourceCandidate> = Vec::new();
            for (si, src) in sources.iter().enumerate() {
                if can_contribute(tp, src) {
                    candidates.push(SourceCandidate {
                        source: si,
                        estimated_cardinality: estimate_cardinality(tp, src),
                    });
                }
            }
            // Already in ascending source-index order (enumeration order); explicit for
            // determinism guarantees if the loop ever changes.
            candidates.sort_by_key(|c| c.source);
            PatternSources {
                pattern: pi,
                candidates,
            }
        })
        .collect()
}

/// The HiBISCuS prune decision for one (pattern, source): `true` iff the source might
/// contribute. **Recall-safe**: returns `true` on any uncertainty; returns `false` only
/// on positive evidence of non-contribution.
fn can_contribute(tp: &TriplePattern, src: &SourceDescriptor) -> bool {
    // Rule 1 — bound-predicate prune. The predicate partition set is the COMPLETE set
    // of predicates the source holds, so a bound predicate absent from it ⇒ no match.
    if let Some(p) = tp.predicate_iri() {
        if !src.has_predicate(p) {
            return false;
        }
    }

    // Rule 3 — bound-class prune for `?x rdf:type C`. Only when the source declares
    // SOME class partition (otherwise the class section is unknown, not empty).
    if matches!(&tp.predicate, Term::Iri(p) if p == RDF_TYPE) {
        if let Some(c) = tp.object.as_iri() {
            // `src.has_class` membership is meaningful only if the source declares any
            // class at all; a descriptor with no class section can't prune here.
            if src.declares_any_class() && !src.has_class(c) {
                return false;
            }
        }
    }

    // Rule 2 — authority prune (subject/object). Disabled unless the source's authority
    // set is COMPLETE; `may_hold_authority` returns true (keep) when incomplete.
    if let Some(s) = tp.subject.as_iri() {
        if !src.may_hold_authority(s) {
            return false;
        }
    }
    if let Some(o) = tp.object.as_iri() {
        if !src.may_hold_authority(o) {
            return false;
        }
    }

    true
}

/// CostFed-style skew-aware cardinality estimate for a (pattern, source) the prune kept.
///
/// * Unbound predicate (`?s ?p ?o`): the source's total triples — the only sound bound
///   when the predicate is open.
/// * Bound predicate, both subject and object unbound: the predicate's `void:triples`.
/// * Bound predicate + bound subject: `triples / distinctSubjects` (avg multiplicity) —
///   the expected fan-out of one subject.
/// * Bound predicate + bound object: `triples / distinctObjects` when known, else the
///   conservative `triples` (recall-safe over-estimate — never prunes).
/// * Bound predicate + both bound: `triples / (distinctSubjects · distinctObjects)`,
///   floored at 0 (presence is uncertain; the join planner treats it as a probe).
fn estimate_cardinality(tp: &TriplePattern, src: &SourceDescriptor) -> f64 {
    let Some(p) = tp.predicate_iri() else {
        // Open predicate: bound only by the source's total triples.
        return src.total_triples.max(1) as f64;
    };
    let Some(part) = src.predicate(p) else {
        // Prune kept it but predicate absent — only reachable for non-IRI predicate;
        // be conservative.
        return src.total_triples.max(1) as f64;
    };
    let triples = part.triples.max(1) as f64;
    let s_bound = tp.subject.is_bound();
    let o_bound = tp.object.is_bound();
    match (s_bound, o_bound) {
        (false, false) => triples,
        (true, false) => {
            // Expected objects for one subject = avg multiplicity.
            triples / part.distinct_subjects.max(1) as f64
        }
        (false, true) => {
            if part.distinct_objects > 0 {
                triples / part.distinct_objects as f64
            } else {
                triples // unknown distinct-objects ⇒ conservative.
            }
        }
        (true, true) => {
            let denom = part.distinct_subjects.max(1) as f64 * part.distinct_objects.max(1) as f64;
            (triples / denom).max(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::{CharSet, ClassPartition, PredPartition, SourceId};
    use crate::pattern::Var;

    fn iri(s: &str) -> Term {
        Term::Iri(s.to_string())
    }
    fn var(s: &str) -> Term {
        Term::Var(Var::new(s))
    }
    fn pred(p: &str, triples: u64, subjects: u64, objects: u64) -> PredPartition {
        PredPartition {
            predicate: p.into(),
            triples,
            distinct_subjects: subjects,
            distinct_objects: objects,
        }
    }

    // Source A holds foaf:knows (dbpedia authority); Source B holds foaf:name only.
    fn source_a() -> SourceDescriptor {
        SourceDescriptor::builder(SourceId::new("A"))
            .total_triples(1000)
            .predicate(pred("http://xmlns.com/foaf/0.1/knows", 200, 100, 80))
            .build()
    }
    fn source_b() -> SourceDescriptor {
        SourceDescriptor::builder(SourceId::new("B"))
            .total_triples(500)
            .predicate(pred("http://xmlns.com/foaf/0.1/name", 300, 300, 290))
            .build()
    }

    #[test]
    fn bound_predicate_prunes_source_without_it() {
        // ?s foaf:knows ?o — only A holds foaf:knows; B is pruned.
        let bgp = Bgp::new(vec![TriplePattern::new(
            var("s"),
            iri("http://xmlns.com/foaf/0.1/knows"),
            var("o"),
        )]);
        let sel = select_sources(&bgp, &[source_a(), source_b()]);
        assert_eq!(sel[0].candidates.len(), 1);
        assert_eq!(sel[0].candidates[0].source, 0); // A
    }

    #[test]
    fn recall_safe_unbound_predicate_keeps_all_sources() {
        // ?s ?p ?o — predicate open: NO source may be pruned (any source could match).
        let bgp = Bgp::new(vec![TriplePattern::new(var("s"), var("p"), var("o"))]);
        let sel = select_sources(&bgp, &[source_a(), source_b()]);
        assert_eq!(
            sel[0].candidates.len(),
            2,
            "open predicate must keep every source"
        );
    }

    #[test]
    fn recall_safe_incomplete_authority_never_prunes_subject_object() {
        // A bound subject IRI on a foreign authority must NOT prune a source whose
        // authority set is incomplete (VoID-parsed) — only the predicate prunes.
        // Source A holds foaf:knows; subject is a wikidata IRI A never declared an
        // authority for. Because A's authority set is incomplete, A is KEPT.
        let bgp = Bgp::new(vec![TriplePattern::new(
            iri("http://www.wikidata.org/entity/Q42"),
            iri("http://xmlns.com/foaf/0.1/knows"),
            var("o"),
        )]);
        let sel = select_sources(&bgp, &[source_a()]);
        assert_eq!(
            sel[0].candidates.len(),
            1,
            "incomplete authority set must not prune"
        );
    }

    #[test]
    fn complete_authority_prunes_foreign_subject_but_keeps_local() {
        // A source with a COMPLETE authority set (HiBISCuS capability) DOES prune a
        // bound subject on a foreign authority — but keeps a local-authority subject.
        let src = SourceDescriptor::builder(SourceId::new("DBp"))
            .predicate(pred("http://xmlns.com/foaf/0.1/knows", 200, 100, 80))
            .authorities_complete() // declares: I only mint foaf authority terms.
            .build();
        let srcs = [src];
        // foreign-authority subject ⇒ pruned.
        let foreign = Bgp::new(vec![TriplePattern::new(
            iri("http://www.wikidata.org/entity/Q42"),
            iri("http://xmlns.com/foaf/0.1/knows"),
            var("o"),
        )]);
        assert!(select_sources(&foreign, &srcs)[0].is_empty());
        // local-authority subject ⇒ kept (foaf authority is in the capability set).
        let local = Bgp::new(vec![TriplePattern::new(
            iri("http://xmlns.com/foaf/0.1/agentX"),
            iri("http://xmlns.com/foaf/0.1/knows"),
            var("o"),
        )]);
        assert!(!select_sources(&local, &srcs)[0].is_empty());
    }

    #[test]
    fn bound_class_prunes_only_when_class_section_declared() {
        // ?x rdf:type :Person — a source that declares classes but NOT :Person prunes;
        // a source with NO class section is KEPT (recall-safe: absent ≠ empty).
        let typed = TriplePattern::new(var("x"), iri(RDF_TYPE), iri("http://ex/Person"));
        let bgp = Bgp::new(vec![typed]);
        // Source with class section but no :Person ⇒ pruned.
        let declares = SourceDescriptor::builder(SourceId::new("C"))
            .predicate(pred(RDF_TYPE, 10, 10, 5))
            .class(ClassPartition {
                class: "http://ex/Company".into(),
                entities: 5,
            })
            .build();
        assert!(select_sources(&bgp, &[declares])[0].is_empty());
        // Source with rdf:type predicate but NO class section ⇒ kept.
        let no_classes = SourceDescriptor::builder(SourceId::new("D"))
            .predicate(pred(RDF_TYPE, 10, 10, 5))
            .build();
        assert!(!select_sources(&bgp, &[no_classes])[0].is_empty());
    }

    #[test]
    fn costfed_cardinality_is_skew_aware() {
        // foaf:knows on A: 200 triples, 100 subjects, 80 objects.
        let a = source_a();
        // ?s knows ?o — full predicate triples.
        let open = TriplePattern::new(var("s"), iri("http://xmlns.com/foaf/0.1/knows"), var("o"));
        assert_eq!(estimate_cardinality(&open, &a), 200.0);
        // bound subject ⇒ avg multiplicity 200/100 = 2.
        let sb = TriplePattern::new(
            iri("http://ex/x"),
            iri("http://xmlns.com/foaf/0.1/knows"),
            var("o"),
        );
        assert_eq!(estimate_cardinality(&sb, &a), 2.0);
        // bound object ⇒ 200/80 = 2.5.
        let ob = TriplePattern::new(
            var("s"),
            iri("http://xmlns.com/foaf/0.1/knows"),
            iri("http://ex/y"),
        );
        assert_eq!(estimate_cardinality(&ob, &a), 2.5);
        // open predicate ⇒ total triples bound.
        let op = TriplePattern::new(var("s"), var("p"), var("o"));
        assert_eq!(estimate_cardinality(&op, &a), 1000.0);
    }

    #[test]
    fn estimate_never_prunes_zero_cardinality_source() {
        // A source whose CS-derived estimate would be tiny must still be RETAINED:
        // estimates are lossy, only capability evidence prunes (recall-safety).
        let a = SourceDescriptor::builder(SourceId::new("A"))
            .total_triples(1)
            .predicate(pred("http://ex/p", 1, 1, 1))
            .char_set(CharSet {
                predicates: vec!["http://ex/p".into()],
                subjects: 1,
                avg_multiplicity: vec![0.0], // pathological zero-multiplicity.
            })
            .build();
        let bgp = Bgp::new(vec![TriplePattern::new(
            var("s"),
            iri("http://ex/p"),
            var("o"),
        )]);
        let sel = select_sources(&bgp, &[a]);
        assert_eq!(
            sel[0].candidates.len(),
            1,
            "low/zero estimate must not prune the source"
        );
    }
}
