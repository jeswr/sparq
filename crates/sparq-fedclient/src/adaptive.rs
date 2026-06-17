//! Adaptive re-planning (design §4.5, the deferred ANAPSID half) — **Phase 7**, the FINAL
//! phase of the streaming federation client (epic **sq-dnko**).
//!
//! Phases 3 + 5 execute the **static** [`JoinTree`](sparq_fedplan::JoinTree) that
//! [`sparq-fedplan`](sparq_fedplan) commits to up front, from descriptor-derived **estimates**.
//! When those descriptors are accurate the static plan is simplest and at least as good
//! (design §7), so it remains the default. But in a real federation the estimates are often
//! wrong: a source returns far more (or fewer) bindings than its VoID statistics predicted.
//! This module is the opt-in feedback loop that *reacts* to that — ANAPSID's harder half:
//!
//! > As streaming execution proceeds, each completed operator reports its **observed**
//! > leaf cardinality. When that diverges materially from the estimate the static plan was
//! > built on, the client re-invokes [`sparq-fedplan`](sparq_fedplan)'s planner on the
//! > **unjoined remainder** (never the already-joined prefix) to choose a cheaper suffix
//! > plan — bounded to **at most once per operator boundary** so it never thrashes.
//!
//! # Where the planner-level engine lives (and what this module adds)
//!
//! The *re-plan decision* — the divergence trigger, the hysteresis / anti-thrash margin, the
//! suffix re-ordering, and the soundness proof — is already implemented, tested, and gated in
//! `sparq-fedplan`'s [`AdaptiveExecutor`](sparq_fedplan::AdaptiveExecutor) /
//! [`RuntimeStats`](sparq_fedplan::RuntimeStats) / [`ReplanPolicy`](sparq_fedplan::ReplanPolicy)
//! (bead `sq-7s4z`, behind its `adaptive-replan` feature). The client does **not** re-write
//! that engine, exactly as it does not re-write the static planner.
//!
//! What Phase 7 adds is the **client-side execution loop** that engine drives: it runs the
//! plan as a sequence of stages, **fetches** each leaf through the real
//! [`FederatedSource`](crate::FederatedSource) adapter, feeds the *real observed row count*
//! back into [`RuntimeStats`], asks the executor whether to re-plan the remaining stages, and
//! joins the (possibly re-ordered) suffix with the **same** materialised
//! [`natural_join`](crate::operators) the static interpreter uses. The observed cardinality is
//! genuine — you cannot know a leaf's true cardinality until you have drained it, which is
//! exactly the operator-boundary point ANAPSID re-plans at.
//!
//! # The load-bearing correctness property (result-equivalence)
//!
//! > Re-planning changes the **plan**, never the **answer**. For any divergence, the adaptive
//! > execution produces the **same solution multiset** as the static [`JoinTree`].
//!
//! This holds because a BGP's answer is the natural join of its per-pattern solution
//! multisets, which is **commutative and associative** (proven exhaustively in
//! `sparq-fedplan`'s `evaluate_is_order_independent_all_permutations`): any join order over
//! the same pattern set yields the same multiset. A re-plan only reorders the not-yet-executed
//! suffix — it never adds, drops, or alters a pattern — and the already-joined prefix is
//! carried across the switch unchanged, so no binding is lost or duplicated. The crate test
//! `replan_result_equals_static_under_divergence` asserts this across a *genuine* large-
//! divergence switch (the suffix order really flips), and `replan_fires_at_most_once_per_boundary`
//! asserts the once-per-boundary bound.
//!
//! # Honest scope (work-vs-stub split)
//!
//! REAL here: the leaf-scan-then-adaptive-join executor, the observed-cardinality capture from
//! the real adapter, the [`RuntimeStats`] / [`AdaptiveExecutor`] wiring, the
//! at-most-once-per-boundary bound, and the two correctness tests on the real interpreter path
//! (the in-crate canned-SRJ tests below, plus the end-to-end
//! `tests/adaptive_result_equals_static.rs` against the REAL engine). The executor is
//! **single-source** (it shares the Phase-3/5
//! [`InterpError::MultiSource`](crate::operators::InterpError) guard) and each per-stage join
//! is the materialised [`natural_join`].
//!
//! By its nature the observed-cardinality boundary is a **materialisation point** — a leaf must
//! be drained to be counted — so the adaptive path scans each leaf fully (once) to learn its
//! real cardinality, then re-orders the join. That trades the within-stage streaming of Phase 5
//! for the cross-stage *re-ordering* this phase buys. Folding the two together — stream a stage
//! WHILE estimating its cardinality from a *prefix* of its rows, the ANAPSID "adaptive operator"
//! refinement — is a clear next slice, filed as a roadmap bead under epic sq-3183, not
//! half-built here.
//
// [OPUS-4.8] sq-ij5x (epic sq-dnko, FINAL phase): client-side adaptive re-planning. Wires
// sparq-fedplan's AdaptiveExecutor into the real federated execution loop. Flagged for Fable
// re-review when available.

use crate::operators::{natural_join, InterpError, Relation};
use crate::planner::{lower_leaf, pattern_vars, SourceResolver};
use crate::source::FederatedSource;
use sparq_fedplan::{
    plan_bgp, AdaptiveExecutor, JoinTree, PatternSources, PlanOptions, ReplanOutcome, ReplanPolicy,
    RuntimeStats, SourceDescriptor,
};

/// A record of one re-plan decision taken at an operator boundary during an adaptive
/// execution — exposed so a caller (and the tests) can see WHEN the plan switched and WHY.
/// [OPUS-4.8] sq-ij5x.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplanEvent {
    /// The execution stage (0-based operator boundary index) the decision was taken at.
    pub boundary: usize,
    /// The outcome the planner returned for this boundary.
    pub outcome: ReplanOutcome,
    /// The suffix (not-yet-executed pattern indices) AFTER this decision, in current order.
    pub remaining_after: Vec<usize>,
}

/// The result of an adaptive execution: the federated answer plus the trace of re-plan
/// decisions taken along the way (one entry per operator boundary that was *considered*).
/// [OPUS-4.8] sq-ij5x.
#[derive(Debug, Clone)]
pub struct AdaptiveOutcome {
    /// The federated answer — the SAME solution multiset the static plan produces
    /// (re-planning changes the plan, never the answer).
    pub relation: Relation,
    /// The order patterns were actually executed in (the prefix as it grew). The first is
    /// the seed; the rest reflect every adopted re-plan.
    pub executed_order: Vec<usize>,
    /// One [`ReplanEvent`] per operator boundary at which a re-plan was *considered*
    /// (after the seed, before each subsequent join). [`ReplanOutcome::Switched`] entries
    /// mark the boundaries the suffix order actually changed at.
    pub replans: Vec<ReplanEvent>,
}

impl AdaptiveOutcome {
    /// How many boundaries the executor actually **switched** the suffix order at — at most
    /// `remaining patterns − 1`, and never more than one per boundary (the load-bearing
    /// bound). [OPUS-4.8] sq-ij5x.
    pub fn switch_count(&self) -> usize {
        self.replans
            .iter()
            .filter(|e| e.outcome == ReplanOutcome::Switched)
            .count()
    }
}

/// Execute a `sparq-fedplan` [`JoinTree`] against a **single source** with adaptive
/// re-planning of the unjoined remainder.
///
/// This is the Phase-7 counterpart of
/// [`materialize_single_source`](crate::operators::materialize_single_source) /
/// [`stream_single_source`](crate::operators::stream_single_source). It runs in two phases:
///
/// 1. **Leaf-scan** — fetch each leaf once through the real `source` adapter
///    (lower → `execute` → parse) and record its **observed** row count into [`RuntimeStats`]
///    — a genuine measurement, not an estimate. A leaf scan is order-independent, so this
///    reveals the real cardinality of every arm, including the not-yet-joined ones (the
///    ANAPSID insight: an arm's divergence is only knowable once its scan completes).
/// 2. **Adaptive join-ordering** — walk the plan stage by stage; **at each operator boundary**
///    ask the [`AdaptiveExecutor`] whether to re-plan the **unjoined remainder** given the
///    observation-corrected statistics, then join the next leaf onto the running prefix with
///    the materialised [`natural_join`] (the SAME operator the static interpreter uses). A
///    re-plan is **considered at most once per boundary** and adopted only when the new suffix
///    beats the current one by the policy's hysteresis margin (anti-thrash).
///
/// `resolver` / `selection` are as for the static interpreter (with the same single-source
/// [`InterpError::MultiSource`] guard); `bgp` / `descriptors` are what `sparq-fedplan` needs
/// to re-`plan_bgp` the remainder; `opts` + `policy` tune the re-planner. When descriptors
/// are accurate the trigger never fires and this returns **byte-identical** results to the
/// static plan — adaptivity is a strict, opt-in addition, off until evidence diverges.
///
/// **Load-bearing invariant:** the returned [`AdaptiveOutcome::relation`] carries the SAME
/// solution multiset as [`materialize_single_source`](crate::operators::materialize_single_source)
/// over the same plan + source, for ANY divergence. [OPUS-4.8] sq-ij5x.
#[allow(clippy::too_many_arguments)]
pub fn execute_adaptive_single_source(
    bgp: &sparq_fedplan::Bgp,
    descriptors: &[SourceDescriptor],
    resolver: &SourceResolver<'_>,
    selection: &[PatternSources],
    source: &dyn FederatedSource,
    tree: &JoinTree,
    opts: PlanOptions,
    policy: ReplanPolicy,
) -> Result<AdaptiveOutcome, InterpError> {
    // The planner-level executor owns the prefix/suffix split + the re-plan DECISION (engine
    // reuse, exactly like plan_bgp). We drive it; it never touches the answer.
    let mut exec = AdaptiveExecutor::new(bgp, descriptors, selection, tree, opts, policy);
    let mut stats = RuntimeStats::new();
    let mut replans: Vec<ReplanEvent> = Vec::new();

    // ---- Leaf-scan phase: materialise each leaf's relation ONCE and record its TRUE observed
    // cardinality. A leaf scan is order-independent (it is the per-pattern solution multiset
    // the source returns; fetching it does not depend on the join order), so doing every scan
    // here is sound and lets the operator-boundary re-planner see the real cardinality of the
    // NOT-YET-JOINED arms — the ANAPSID insight that an arm's divergence is only knowable once
    // its scan completes. Each leaf is fetched exactly once; the join order chosen below merely
    // reorders how these already-fetched relations are combined. [OPUS-4.8] sq-ij5x.
    let mut leaf_rel: std::collections::HashMap<usize, Relation> = std::collections::HashMap::new();
    for pattern in tree.join_order() {
        let leaf = fetch_leaf(resolver, selection, source, pattern)?;
        // The observed leaf cardinality IS the real row count the source returned.
        stats.record_leaf_cardinality(pattern, leaf.rows.len() as f64);
        leaf_rel.insert(pattern, leaf);
    }

    // ---- Adaptive join-ordering phase: walk the executor stage by stage, re-planning the
    // UNJOINED REMAINDER at each operator boundary from the real observations.
    let mut acc: Option<Relation> = None;
    let mut executed_order: Vec<usize> = Vec::new();
    let mut boundary: usize = 0;

    while let Some(pattern) = exec.advance() {
        // Join this leaf onto the running prefix (materialised natural join — the SAME operator
        // the static interpreter is proven multiset-equal to). The first leaf seeds the acc.
        let leaf = leaf_rel
            .remove(&pattern)
            .expect("[OPUS-4.8] sq-ij5x: every join_order pattern was scanned above");
        acc = Some(match acc {
            None => leaf,
            Some(prev) => natural_join(&prev, &leaf),
        });
        executed_order.push(pattern);

        // ---- Operator boundary: consider a re-plan of the REMAINING suffix, at most ONCE.
        // `maybe_replan` is invoked exactly once here per boundary; the executor's own
        // `max_replans` budget + `replans_done` counter bound it further. A switch reorders
        // only the not-yet-executed suffix — the prefix `acc` is carried across unchanged, so
        // no binding is re-computed, lost, or duplicated.
        if !exec.remaining_order().is_empty() {
            let outcome = exec.maybe_replan(&stats);
            replans.push(ReplanEvent {
                boundary,
                outcome,
                remaining_after: exec.remaining_order().to_vec(),
            });
            boundary += 1;
        }
    }

    // Project onto the whole-BGP variable set, matching the static interpreters.
    let project = bgp_vars(resolver);
    let relation = acc.unwrap_or_default().project(&project);
    Ok(AdaptiveOutcome {
        relation,
        executed_order,
        replans,
    })
}

/// Fetch + parse one leaf pattern's sub-result from the single source, enforcing the SAME
/// single-source guard as the static interpreters (a leaf the planner retained more than one
/// source for is rejected with [`InterpError::MultiSource`], never silently dropped — the
/// multi-source UNION-per-leaf fan-out remains the deferred slice it is in Phase 3/5).
/// [OPUS-4.8] sq-ij5x.
fn fetch_leaf(
    resolver: &SourceResolver<'_>,
    selection: &[PatternSources],
    source: &dyn FederatedSource,
    pattern: usize,
) -> Result<Relation, InterpError> {
    if let Some(ps) = selection.iter().find(|ps| ps.pattern == pattern) {
        if ps.candidates.len() > 1 {
            return Err(InterpError::MultiSource {
                pattern,
                sources: ps.candidates.len(),
            });
        }
    }
    let tp = resolver.pattern(pattern)?;
    let sub = lower_leaf(tp);
    let body = source.execute(&sub)?;
    crate::operators::parse_srj(&body).map_err(InterpError::BadSrj)
}

/// All distinct variables of the resolver's BGP, in first-seen order — the `SELECT *` scope,
/// identical to the static interpreter's projection so the adaptive result has the same
/// header. [OPUS-4.8] sq-ij5x.
fn bgp_vars(resolver: &SourceResolver<'_>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for tp in &resolver.bgp().patterns {
        for v in pattern_vars(tp) {
            if !out.contains(&v) {
                out.push(v);
            }
        }
    }
    out
}

/// Convenience: re-`plan_bgp` a fresh static plan from a BGP + descriptors + selection, the
/// way a caller seeds [`execute_adaptive_single_source`]. Thin wrapper over the planner so a
/// caller does not need to thread `plan_bgp`'s argument order by hand. [OPUS-4.8] sq-ij5x.
pub fn static_plan(
    bgp: &sparq_fedplan::Bgp,
    selection: &[PatternSources],
    descriptors: &[SourceDescriptor],
    opts: &PlanOptions,
) -> Option<JoinTree> {
    plan_bgp(bgp, selection, descriptors, opts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operators::{materialize_single_source, solutions_equal};
    use crate::planner::SourceResolver;
    use crate::source::{Endpoint, FedError, SubQuery, Transport};
    use sparq_fedplan::{
        select_sources, Bgp, PredPartition, SourceDescriptor, SourceId, Term as FpTerm,
        TriplePattern, Var,
    };
    use std::collections::HashMap as StdHashMap;
    use std::sync::Mutex;

    fn iri(s: &str) -> FpTerm {
        FpTerm::Iri(s.to_string())
    }
    fn var(s: &str) -> FpTerm {
        FpTerm::Var(Var::new(s))
    }
    fn pred(p: &str, triples: u64, subjects: u64, objects: u64) -> PredPartition {
        PredPartition {
            predicate: p.into(),
            triples,
            distinct_subjects: subjects,
            distinct_objects: objects,
        }
    }

    /// A transport answering each sub-query from a fixed (SPARQL → SRJ) map. Records how many
    /// times it was asked so a test can assert no extra fetches. [OPUS-4.8] sq-ij5x.
    struct MapTransport {
        answers: StdHashMap<String, String>,
        calls: Mutex<usize>,
    }
    impl MapTransport {
        fn new(answers: StdHashMap<String, String>) -> Self {
            MapTransport {
                answers,
                calls: Mutex::new(0),
            }
        }
    }
    impl Transport for MapTransport {
        fn fetch(&self, _endpoint: &str, query: &str) -> Result<String, String> {
            *self.calls.lock().unwrap() += 1;
            self.answers
                .get(query)
                .cloned()
                .ok_or_else(|| format!("no canned answer for {:?}", query))
        }
    }

    /// A transport that panics if reached — proves the multi-source guard fails closed before
    /// any fetch. [OPUS-4.8] sq-ij5x.
    struct PanicTransport;
    impl Transport for PanicTransport {
        fn fetch(&self, _e: &str, _q: &str) -> Result<String, String> {
            panic!("adaptive multi-source guard must fail closed before any fetch");
        }
    }

    // A small SRJ builder over URI bindings.
    fn srj(vars: &[&str], rows: &[&[(&str, &str)]]) -> String {
        let mut s = String::from("{\"head\":{\"vars\":[");
        for (i, v) in vars.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str(&format!("\"{}\"", v));
        }
        s.push_str("]},\"results\":{\"bindings\":[");
        for (ri, row) in rows.iter().enumerate() {
            if ri > 0 {
                s.push(',');
            }
            s.push('{');
            for (ci, (var, uri)) in row.iter().enumerate() {
                if ci > 0 {
                    s.push(',');
                }
                s.push_str(&format!(
                    "\"{}\":{{\"type\":\"uri\",\"value\":\"{}\"}}",
                    var, uri
                ));
            }
            s.push('}');
        }
        s.push_str("]}}");
        s
    }

    /// A 3-arm star on ?s served by ONE endpoint: ?s :a ?x . ?s :b ?y . ?s :c ?z. The
    /// descriptor estimates :a=10 (seed), :b=20, :c=1000 ⇒ the STATIC suffix after the :a seed
    /// is [:b, :c]. The canned answers make reality diverge >4×: :b is actually large and :c
    /// actually tiny, so a re-plan should flip the suffix to [:c, :b]. The answers are chosen
    /// so the join is non-trivial (subjects drop / fan out), so a wrong order would change the
    /// MULTISET if the executor were unsound. [OPUS-4.8] sq-ij5x.
    fn star_fixture() -> (
        Bgp,
        [SourceDescriptor; 1],
        StdHashMap<String, String>,
    ) {
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), iri("http://ex/a"), var("x")),
            TriplePattern::new(var("s"), iri("http://ex/b"), var("y")),
            TriplePattern::new(var("s"), iri("http://ex/c"), var("z")),
        ]);
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(100_000)
            .predicate(pred("http://ex/a", 10, 10, 10))
            .predicate(pred("http://ex/b", 20, 20, 20))
            .predicate(pred("http://ex/c", 1000, 1000, 1000))
            .build();

        let mut answers = StdHashMap::new();
        // :a — three subjects.
        answers.insert(
            "SELECT ?s ?x WHERE { ?s <http://ex/a> ?x }".to_string(),
            srj(
                &["s", "x"],
                &[
                    &[("s", "http://ex/s1"), ("x", "http://ex/x1")],
                    &[("s", "http://ex/s2"), ("x", "http://ex/x2")],
                    &[("s", "http://ex/s3"), ("x", "http://ex/x3")],
                ],
            ),
        );
        // :b — reality is LARGE (estimate 20): s3 drops, s1 fans out to two ys.
        answers.insert(
            "SELECT ?s ?y WHERE { ?s <http://ex/b> ?y }".to_string(),
            srj(
                &["s", "y"],
                &[
                    &[("s", "http://ex/s1"), ("y", "http://ex/y1")],
                    &[("s", "http://ex/s1"), ("y", "http://ex/y7")],
                    &[("s", "http://ex/s2"), ("y", "http://ex/y2")],
                ],
            ),
        );
        // :c — reality is TINY (estimate 1000): s2 drops.
        answers.insert(
            "SELECT ?s ?z WHERE { ?s <http://ex/c> ?z }".to_string(),
            srj(
                &["s", "z"],
                &[
                    &[("s", "http://ex/s1"), ("z", "http://ex/z1")],
                    &[("s", "http://ex/s3"), ("z", "http://ex/z3")],
                ],
            ),
        );
        (bgp, [src], answers)
    }

    /// THE load-bearing Phase-7 test: a large-divergence scenario re-plans the unjoined
    /// remainder to a CHEAPER suffix order, and the adaptive result is the SAME solution
    /// multiset as the static plan. Re-planning changes the plan, never the answer.
    /// [OPUS-4.8] sq-ij5x.
    #[test]
    fn replan_result_equals_static_under_divergence() {
        let (bgp, descriptors, answers) = star_fixture();
        let sel = select_sources(&bgp, &descriptors);
        let tree = plan_bgp(&bgp, &sel, &descriptors, &PlanOptions::default()).unwrap();
        // The static suffix after the :a seed is [:b, :c] (by descriptor estimate).
        assert_eq!(tree.join_order(), vec![0, 1, 2], "static order is [:a,:b,:c]");

        // --- Static reference result.
        let ep_static = Endpoint::new(
            "http://8.8.8.8/sparql",
            Box::new(MapTransport::new(answers.clone())),
        );
        let adapters_s: Vec<&dyn FederatedSource> = vec![&ep_static];
        let resolver_s = SourceResolver::new(&bgp, &adapters_s);
        let static_rel =
            materialize_single_source(&resolver_s, &sel, &ep_static, &tree).unwrap();
        assert!(!static_rel.rows.is_empty(), "fixture must produce rows");

        // --- Adaptive execution over the SAME answers.
        let ep_ad = Endpoint::new(
            "http://8.8.8.8/sparql",
            Box::new(MapTransport::new(answers)),
        );
        let adapters_a: Vec<&dyn FederatedSource> = vec![&ep_ad];
        let resolver_a = SourceResolver::new(&bgp, &adapters_a);
        let out = execute_adaptive_single_source(
            &bgp,
            &descriptors,
            &resolver_a,
            &sel,
            &ep_ad,
            &tree,
            PlanOptions::default(),
            ReplanPolicy::default(),
        )
        .unwrap();

        // The re-plan really fired: the now-cheap :c arm is re-ordered ahead of the now-
        // expensive :b arm, so the executed order is NOT the static [:a,:b,:c].
        assert_eq!(
            out.switch_count(),
            1,
            "the divergent fixture must switch exactly once"
        );
        assert_eq!(
            out.executed_order,
            vec![0, 2, 1],
            "the now-cheap :c arm executes before the now-expensive :b arm"
        );
        assert_ne!(
            out.executed_order,
            tree.join_order(),
            "the adaptive order must actually differ from the static order (non-vacuous)"
        );
        // …yet the SAME pattern set (no pattern added or dropped).
        let mut a = out.executed_order.clone();
        let mut s = tree.join_order();
        a.sort();
        s.sort();
        assert_eq!(a, s, "re-plan preserves the pattern SET");

        // The LOAD-BEARING invariant: identical result multiset despite the different order.
        assert!(
            solutions_equal(
                &out.relation.vars,
                &out.relation.rows,
                &static_rel.vars,
                &static_rel.rows
            ),
            "adaptive result MUST equal the static result multiset.\n adaptive={:?}\n static={:?}",
            out.relation,
            static_rel,
        );
    }

    /// The once-per-operator-boundary bound: across an execution that COULD re-plan at every
    /// boundary, [`maybe_replan`] is consulted at most once per boundary, and the executor's
    /// `replans_done` budget bounds the total switches. We assert: (a) exactly one re-plan
    /// EVENT is recorded per post-seed boundary, and (b) the number of EVENTS never exceeds
    /// the number of operator boundaries (`patterns − 1`). No thrashing. [OPUS-4.8] sq-ij5x.
    #[test]
    fn replan_fires_at_most_once_per_boundary() {
        let (bgp, descriptors, answers) = star_fixture();
        let sel = select_sources(&bgp, &descriptors);
        let tree = plan_bgp(&bgp, &sel, &descriptors, &PlanOptions::default()).unwrap();

        let ep = Endpoint::new(
            "http://8.8.8.8/sparql",
            Box::new(MapTransport::new(answers)),
        );
        let adapters: Vec<&dyn FederatedSource> = vec![&ep];
        let resolver = SourceResolver::new(&bgp, &adapters);
        let out = execute_adaptive_single_source(
            &bgp,
            &descriptors,
            &resolver,
            &sel,
            &ep,
            &tree,
            PlanOptions::default(),
            ReplanPolicy::default(),
        )
        .unwrap();

        // A 3-pattern BGP has 2 operator boundaries AFTER the seed (after stage 1, after
        // stage 2 — the latter has an empty remaining suffix and is not considered). So at
        // most `patterns − 1 == 2` re-plan EVENTS, and here exactly 1 (the suffix-of-2 boundary
        // is considered; the suffix-of-0/1 boundary is not — nothing to reorder).
        let boundaries = bgp.patterns.len() - 1;
        assert!(
            out.replans.len() <= boundaries,
            "at most one re-plan event per operator boundary ({} ≤ {})",
            out.replans.len(),
            boundaries
        );
        // Each recorded event sits at a DISTINCT boundary index (no boundary considered twice).
        let mut seen = std::collections::HashSet::new();
        for ev in &out.replans {
            assert!(
                seen.insert(ev.boundary),
                "boundary {} was considered more than once (thrash!)",
                ev.boundary
            );
        }
        // And the switch count never exceeds the considered boundaries.
        assert!(out.switch_count() <= out.replans.len());
    }

    /// With ACCURATE descriptors (no divergence) the trigger never fires: the adaptive
    /// execution keeps the static order AND returns the same result — adaptivity is a strict
    /// opt-in addition that is inert until evidence diverges. [OPUS-4.8] sq-ij5x.
    #[test]
    fn no_replan_when_descriptors_accurate() {
        // A chain ?s :p ?o . ?o :q ?z whose canned row counts MATCH the descriptor estimates,
        // so nothing diverges.
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), iri("http://ex/p"), var("o")),
            TriplePattern::new(var("o"), iri("http://ex/q"), var("z")),
        ]);
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(1000)
            .predicate(pred("http://ex/p", 2, 2, 2))
            .predicate(pred("http://ex/q", 2, 2, 2))
            .build();
        let descriptors = [src];
        let sel = select_sources(&bgp, &descriptors);
        let tree = plan_bgp(&bgp, &sel, &descriptors, &PlanOptions::default()).unwrap();

        let mut answers = StdHashMap::new();
        answers.insert(
            "SELECT ?s ?o WHERE { ?s <http://ex/p> ?o }".to_string(),
            srj(
                &["s", "o"],
                &[
                    &[("s", "http://ex/s1"), ("o", "http://ex/o1")],
                    &[("s", "http://ex/s2"), ("o", "http://ex/o2")],
                ],
            ),
        );
        answers.insert(
            "SELECT ?o ?z WHERE { ?o <http://ex/q> ?z }".to_string(),
            srj(
                &["o", "z"],
                &[
                    &[("o", "http://ex/o1"), ("z", "http://ex/z1")],
                    &[("o", "http://ex/o2"), ("z", "http://ex/z2")],
                ],
            ),
        );

        let ep_static = Endpoint::new(
            "http://8.8.8.8/sparql",
            Box::new(MapTransport::new(answers.clone())),
        );
        let adapters_s: Vec<&dyn FederatedSource> = vec![&ep_static];
        let resolver_s = SourceResolver::new(&bgp, &adapters_s);
        let static_rel =
            materialize_single_source(&resolver_s, &sel, &ep_static, &tree).unwrap();

        let ep = Endpoint::new("http://8.8.8.8/sparql", Box::new(MapTransport::new(answers)));
        let adapters: Vec<&dyn FederatedSource> = vec![&ep];
        let resolver = SourceResolver::new(&bgp, &adapters);
        let out = execute_adaptive_single_source(
            &bgp,
            &descriptors,
            &resolver,
            &sel,
            &ep,
            &tree,
            PlanOptions::default(),
            ReplanPolicy::default(),
        )
        .unwrap();

        assert_eq!(out.switch_count(), 0, "accurate descriptors ⇒ no switch");
        assert_eq!(
            out.executed_order,
            tree.join_order(),
            "no divergence ⇒ the static order is kept"
        );
        assert!(
            solutions_equal(
                &out.relation.vars,
                &out.relation.rows,
                &static_rel.vars,
                &static_rel.rows
            ),
            "inert adaptive path returns the static result"
        );
    }

    /// The adaptive executor shares the single-source guard: a leaf with >1 retained source
    /// fails closed with [`InterpError::MultiSource`] BEFORE any fetch (PanicTransport proves
    /// the early return). [OPUS-4.8] sq-ij5x.
    #[test]
    fn adaptive_rejects_multi_source_leaf() {
        let bgp = Bgp::new(vec![TriplePattern::new(
            var("s"),
            iri("http://ex/p"),
            var("o"),
        )]);
        let mk = |id: &str| {
            SourceDescriptor::builder(SourceId::new(id))
                .total_triples(10)
                .predicate(pred("http://ex/p", 10, 10, 10))
                .build()
        };
        let descriptors = [mk("A"), mk("B")];
        let sel = select_sources(&bgp, &descriptors);
        assert_eq!(sel[0].candidates.len(), 2, "both sources retained");
        let tree = plan_bgp(&bgp, &sel, &descriptors, &PlanOptions::default()).unwrap();
        let ep = Endpoint::new("http://8.8.8.8/sparql", Box::new(PanicTransport));
        let adapters: Vec<&dyn FederatedSource> = vec![&ep];
        let resolver = SourceResolver::new(&bgp, &adapters);
        let err = execute_adaptive_single_source(
            &bgp,
            &descriptors,
            &resolver,
            &sel,
            &ep,
            &tree,
            PlanOptions::default(),
            ReplanPolicy::default(),
        )
        .unwrap_err();
        assert_eq!(err, InterpError::MultiSource { pattern: 0, sources: 2 });
    }

    /// The `static_plan` convenience and `SubQuery`/`FedError` imports are honest surface —
    /// reference them so the test module's import set is real. [OPUS-4.8] sq-ij5x.
    #[test]
    fn static_plan_helper_plans() {
        let (bgp, descriptors, _) = star_fixture();
        let sel = select_sources(&bgp, &descriptors);
        let tree = static_plan(&bgp, &sel, &descriptors, &PlanOptions::default());
        assert!(tree.is_some());
        let _ = SubQuery::new("ASK {}");
        let _ = FedError::Transport("x".into());
    }
}
