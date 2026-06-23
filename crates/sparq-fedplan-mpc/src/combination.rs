//! **Phase 6** — FedUP-style result-aware source-**combination** pruning (design record
//! §3 / §8 Phase 6; bead sq-pwr.3, epics sq-pwr / sq-0jsc).
//!
//! Phase 2 ([`crate::selection`]) keeps each `(pattern, source)` candidate **in isolation**:
//! it answers "could this source contribute to *this pattern*?". A federated plan, however,
//! executes a **conjunction** of patterns, and the seam's secure-join cost grows with the
//! number of source-**combinations** (one source per pattern, the cross-product over the
//! per-pattern candidate sets) it must consider. FedUP (WWW'24) is the prior art for
//! *result-aware* plans: prune source-combinations that **provably contribute no answer**
//! before any execution. This pass is the seam's pre-MPC analogue — it refines the Phase-2
//! selection by surfacing which combinations are dead for a full-BGP answer, so the seam
//! feeds fewer combinations toward the (expensive) MPC path.
//!
//! # The one recall-safe, summary-expressible combination prune (and the honest non-rule)
//!
//! The load-bearing question is *what* a result-aware combination prune can prove from the
//! served descriptor **summaries** (no live data) that the per-pattern selector cannot already
//! prove. The honest answer is a single rule plus a deliberately-declined non-rule:
//!
//! * **Rule C1 — unsatisfiable-conjunct collapse.** A full-BGP answer is one variable
//!   assignment satisfying **every** conjunct simultaneously. If **any** pattern's Phase-2
//!   candidate list is empty, then — by the upstream HiBISCuS recall-safety invariant — *no*
//!   retained source can contribute any binding for that conjunct, so the conjunct matches the
//!   **empty relation**. An empty conjunct makes the whole conjunction empty (`∅ ⋈ R = ∅`),
//!   so **zero** full-BGP answers exist and **every** source-combination is dead. This is
//!   genuinely combination-level: per-pattern selection records the one pattern as empty but
//!   still hands the join planner the *other* patterns' full candidate lists, which would be
//!   multiplied into source-combinations the seam would route toward MPC. This pass is the
//!   first place that can prove those candidates are dead **for a full-BGP answer** and
//!   collapse the combination space to zero.
//!
//! * **The declined non-rule — value-overlap / bound-IRI propagation.** A tempting idea is to
//!   propagate a bound IRI constant across a shared join variable and prune a source whose
//!   value space cannot reach it. This is **not** recall-safely expressible from the public
//!   [`SourceDescriptor`] API, and declining it is a *correctness* decision, not an omission:
//!   (a) a constant in an object/subject *position* is already tested per-source by the
//!   upstream authority prune ([`SourceDescriptor::may_hold_authority`], `select_sources`
//!   Rule 2) — there is nothing left for a combination pass to add for a bound position; and
//!   (b) a join *variable* is bound to data-dependent resource values unknown at plan time, so
//!   treating an object-position constant as a binding on the join variable would be unsound,
//!   and intersecting two sources' authority *sets* would need a set enumerator the public API
//!   deliberately does not expose ([`SourceDescriptor::may_hold_authority`] is a single-IRI
//!   test). With only a single-IRI test and no value enumeration there is **no** recall-safe
//!   value-overlap prune at the combination level. Recorded here so a future contributor does
//!   not reach into the private authority set to build an unsound prune.
//!
//! # Recall-safety (the load-bearing property, mirroring the upstream invariant)
//!
//! > **A (pattern, source) candidate is marked combination-dead only when no full-BGP answer
//! > can use it; on any uncertainty it is kept.**
//!
//! The pass marks candidates dead **only** when some pattern is empty, i.e. when the
//! conjunction is *proved* unsatisfiable (an empty conjunct ⇒ no answers exist ⇒ marking every
//! candidate dead loses nothing). When every pattern is non-empty it drops **nothing** — there
//! is no further recall-safe combination prune available from the public summary. It reads no
//! descriptor capability of its own and adds no independent prune that could be unsound; it
//! merely **propagates** an already-recall-safe emptiness verdict across the conjunction.
//!
//! # Honesty / threat model (no claim beyond selection plumbing)
//!
//! This is **plumbing — result-aware source-combination routing, not a cryptographic
//! guarantee.** It performs **NO** MPC, runs **NO** secret-sharing, opens nothing, verifies
//! nothing, and reads only the (public) Phase-2 selection it is handed. It makes **NO**
//! soundness/privacy/security claim. The MPC estate (`sparq-mpc`) is research-grade,
//! **honest-majority semi-honest only**, and is **NOT** externally audited — the
//! accredited-cryptographer sign-off (sq-qhy4) and the coZK re-audit (sq-9hrn) are pending.
//! This pass does not change that posture by one inch. The only thing it "reveals" is which
//! combinations are infeasible, derived entirely from the caller's own descriptors. See the
//! crate `README.md` and `research/mpc-untrusted-planner-routing-design.md`.
//!
//! [OPUS-4.8] sq-pwr.3.

use sparq_fedplan::{Bgp, SourceDescriptor, SourceId};

use crate::seam::{SeamError, SeamPhase};
use crate::selection::{PrivatePatternSources, SelectedPrivateSources};

/// Why a (pattern, source) candidate is **combination-dead** — it survived the Phase-2 prune
/// but cannot contribute to any full-BGP answer at the combination level. There is exactly one
/// reason at this tier, but the enum is `#[non_exhaustive]` so a later result-aware rule can add
/// one without a breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CombinationPruneReason {
    /// The full BGP is unsatisfiable because pattern `witness` (a conjunct) has **zero**
    /// surviving sources after Phase 2. An empty conjunct makes the whole conjunction empty, so
    /// this candidate cannot participate in any answer (Rule C1).
    UnsatisfiableBgp {
        /// The empty-pattern index that witnesses the unsatisfiability (the smallest such index,
        /// for determinism).
        witness: usize,
    },
}

impl CombinationPruneReason {
    /// A short human label for the reason (for diagnostics / the audit trail).
    pub fn label(self) -> String {
        match self {
            CombinationPruneReason::UnsatisfiableBgp { witness } => format!(
                "full BGP unsatisfiable: conjunct (pattern {}) has no contributing source",
                witness
            ),
        }
    }
}

/// One (pattern, source) candidate that survived Phase 2 but is dead for a full-BGP answer at
/// the combination level, with the [`CombinationPruneReason`]. Surfaced so the combination prune
/// is **auditable** — a relying party sees exactly which surviving candidates the result-aware
/// pass found infeasible and why, rather than a silent collapse (the "declared, not implicit"
/// discipline, applied to combination pruning).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrunedCombination {
    /// The pattern the dead candidate belongs to.
    pub pattern: usize,
    /// The source index that is dead for a full-BGP answer.
    pub source: usize,
    /// The resolved source id.
    pub source_id: SourceId,
    /// Why it is dead.
    pub reason: CombinationPruneReason,
}

/// One connected component of the BGP join graph (pattern indices that are transitively linked
/// by a shared variable), surfaced so an auditor can see the Cartesian structure the
/// satisfiability verdict was computed over. Two patterns are in the same component iff they are
/// connected by a chain of [`Bgp::shares_var`] edges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BgpComponent {
    /// The pattern indices in this component, ascending (deterministic).
    pub patterns: Vec<usize>,
    /// Whether every member pattern is non-empty after Phase 2. `false` iff some member pattern
    /// has no surviving source (this component's sub-answer is empty). NOTE: a single empty
    /// component still makes the **whole** BGP unsatisfiable — see [`PrunedCombinations`].
    pub satisfiable: bool,
}

/// The output of the **Phase-6 result-aware source-combination prune**: the Phase-2 selection
/// carried through, plus the satisfiability verdict, the empty-pattern witnesses, the BGP join
/// structure, and the combination-dead audit trail.
///
/// This carries **no** secret material and runs **no** privacy-bearing logic — it is the
/// (public) Phase-2 candidate set plus a result-aware feasibility annotation derived entirely
/// from the caller's own descriptors. The per-pattern lists are carried through **unchanged**
/// (never mutated): the verdict is **advisory and auditable**, so the caller (a future
/// join-planner feeder) chooses whether to act on `bgp_satisfiable` / `pruned` rather than
/// having candidates silently removed.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct PrunedCombinations {
    /// The Phase-2 per-pattern candidate sets, carried through **verbatim** (the prune is
    /// advisory — the dead candidates are enumerated in `pruned`, not removed here).
    pub per_pattern: Vec<PrivatePatternSources>,
    /// Whether **any** full-BGP source-combination can produce an answer. `false` iff some
    /// pattern is empty after Phase 2 (an empty conjunct ⇒ the whole conjunction is empty,
    /// regardless of the BGP's connectivity).
    pub bgp_satisfiable: bool,
    /// The empty-pattern witness indices (ascending) that prove unsatisfiability; empty when
    /// `bgp_satisfiable`.
    pub empty_patterns: Vec<usize>,
    /// The connected components of the BGP join graph (sorted by minimum pattern index;
    /// deterministic). Structural metadata for the auditor — the Rule-C1 verdict is the simple
    /// "any empty conjunct" theorem, not a per-component decision.
    pub components: Vec<BgpComponent>,
    /// The combination-dead candidates with reasons, ascending by `(pattern, source)`. Empty
    /// when `bgp_satisfiable` (nothing is dropped on the live path).
    pub pruned: Vec<PrunedCombination>,
}

impl PrunedCombinations {
    /// Whether the full BGP is dead (no source-combination can produce an answer).
    pub fn is_bgp_dead(&self) -> bool {
        !self.bgp_satisfiable
    }

    /// The pattern indices that still have at least one *live* candidate for a full-BGP answer,
    /// ascending. When the BGP is dead this is empty (every combination collapsed); otherwise it
    /// is every non-empty pattern (nothing was pruned on the live path).
    pub fn surviving_combination_patterns(&self) -> Vec<usize> {
        if self.is_bgp_dead() {
            return Vec::new();
        }
        self.per_pattern
            .iter()
            .filter(|p| !p.is_empty())
            .map(|p| p.pattern)
            .collect()
    }
}

/// **Phase 6** — the result-aware source-combination prune over the Phase-2 selection (design
/// record §3 / §8 Phase 6).
///
/// Given the BGP, the source descriptors, and the Phase-2 [`SelectedPrivateSources`], compute
/// whether any full-BGP source-combination can produce an answer (Rule C1: an empty conjunct
/// makes the whole conjunction unsatisfiable), and surface the combination-dead candidates plus
/// the BGP join structure. The Phase-2 per-pattern lists are carried through **unchanged**; the
/// verdict is advisory and auditable.
///
/// `sources` is taken for API symmetry and to read the BGP's join structure, but Rule C1 does
/// **not** re-read descriptor capabilities — it operates over the already-recall-safe Phase-2
/// emptiness, so it never introduces an independent (possibly unsound) capability prune. The
/// declined value-overlap non-rule (see the module docs) is *why* no descriptor capability is
/// re-tested here.
///
/// **Recall-safe by construction:** candidates are marked dead **only** when the conjunction is
/// *proved* unsatisfiable (some pattern empty ⇒ no answers exist). When every pattern is
/// non-empty, **nothing** is pruned. Deterministic throughout (index-ascending iteration,
/// components sorted by minimum index, the audit trail explicitly sorted).
///
/// # Errors
///
/// Returns [`SeamError::DescriptorMismatch`] (phase [`SeamPhase::SourceCombination`]) if the
/// Phase-2 selection's pattern count does not match the BGP's — the pass refuses to guess the
/// alignment rather than silently mis-attribute a pattern (fail-closed, the same posture as
/// Phase 2). It never panics and performs **no** MPC.
///
/// This function makes **no** soundness/privacy claim — see the module docs and the crate
/// `README.md`. [OPUS-4.8] sq-pwr.3.
pub fn prune_source_combinations(
    bgp: &Bgp,
    sources: &[SourceDescriptor],
    selected: &SelectedPrivateSources,
) -> Result<PrunedCombinations, SeamError> {
    // `sources` is read only for the (future) symmetry the signature promises and to keep the
    // call shape identical to Phase 2/3; Rule C1 is driven by the Phase-2 output. Touch it so a
    // length mismatch with the selection is still a clean, fail-closed signal rather than a
    // silent assumption.
    let _ = sources;

    // Fail-closed on a selection/BGP shape mismatch — never guess which pattern is which.
    if selected.per_pattern.len() != bgp.patterns.len() {
        return Err(SeamError::DescriptorMismatch {
            phase: SeamPhase::SourceCombination,
            source_id: String::new(),
            detail:
                "Phase-2 selection pattern count does not match the BGP (cannot align combinations)",
        });
    }

    // 1. The empty conjuncts (ascending). An empty pattern is a PROVED-empty relation (upstream
    //    recall-safety + the recall-safe Phase-2 participation prune), so it is a sound witness.
    let empty_patterns: Vec<usize> = selected
        .per_pattern
        .iter()
        .filter(|p| p.is_empty())
        .map(|p| p.pattern)
        .collect();
    let bgp_satisfiable = empty_patterns.is_empty();

    // 2. The connected components of the join graph (structural metadata for the auditor).
    let components = connected_components(bgp, &empty_patterns);

    // 3. The combination-dead audit trail. On the live (satisfiable) path nothing is pruned —
    //    there is no further recall-safe combination prune from the public summary. On the dead
    //    path every surviving candidate is dead for a full-BGP answer; record it against the
    //    smallest empty-pattern witness (determinism).
    let mut pruned: Vec<PrunedCombination> = Vec::new();
    if !bgp_satisfiable {
        let witness = empty_patterns[0];
        for ps in &selected.per_pattern {
            for cand in &ps.candidates {
                pruned.push(PrunedCombination {
                    pattern: ps.pattern,
                    source: cand.source,
                    source_id: cand.source_id.clone(),
                    reason: CombinationPruneReason::UnsatisfiableBgp { witness },
                });
            }
        }
    }
    // Already in (pattern, source) order (the Phase-2 output is pattern-ordered, candidates are
    // source-index ascending) — make it an explicit guarantee, mirroring `selection.rs`.
    pruned.sort_by_key(|p| (p.pattern, p.source));

    Ok(PrunedCombinations {
        per_pattern: selected.per_pattern.clone(),
        bgp_satisfiable,
        empty_patterns,
        components,
        pruned,
    })
}

/// The connected components of `bgp`'s join graph (two patterns linked iff they share a
/// variable, [`Bgp::shares_var`]), as a deterministic list. Each component's pattern list is
/// ascending; the component vector is sorted by minimum pattern index. A component is
/// `satisfiable` iff none of its patterns is in `empty_patterns`.
fn connected_components(bgp: &Bgp, empty_patterns: &[usize]) -> Vec<BgpComponent> {
    let n = bgp.patterns.len();
    // Union-find over pattern indices.
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]]; // path halving.
            x = parent[x];
        }
        x
    }
    for i in 0..n {
        for j in (i + 1)..n {
            if bgp.shares_var(i, j) {
                let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                if ri != rj {
                    // Union toward the smaller root for a deterministic representative.
                    if ri < rj {
                        parent[rj] = ri;
                    } else {
                        parent[ri] = rj;
                    }
                }
            }
        }
    }

    // Group indices by root, in ascending member order.
    use std::collections::BTreeMap;
    let mut by_root: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..n {
        let r = find(&mut parent, i);
        by_root.entry(r).or_default().push(i);
    }

    let mut components: Vec<BgpComponent> = by_root
        .into_values()
        .map(|patterns| {
            let satisfiable = patterns.iter().all(|p| !empty_patterns.contains(p));
            BgpComponent {
                patterns,
                satisfiable,
            }
        })
        .collect();
    // Sort by minimum pattern index (each `patterns` is already ascending, so [0] is the min).
    components.sort_by_key(|c| c.patterns.first().copied().unwrap_or(usize::MAX));
    components
}

#[cfg(test)]
mod tests {
    use super::*;
    use sparq_fedplan::{PredPartition, Term, TriplePattern, Var};

    use crate::selection::{select_private_sources, PrivateCandidate};
    use crate::SourcePrivacyDescriptor;

    // ── Fixtures ────────────────────────────────────────────────────────────────────────

    const SHARES_HOUSE: &str = "http://ex/sharesHouse";
    const NAME: &str = "http://ex/name";
    const OWES: &str = "http://ex/owes";
    const MEMBER_OF: &str = "http://ex/memberOf";
    const ABSENT: &str = "http://ex/predicateNoSourceHolds";

    fn iri(s: &str) -> Term {
        Term::Iri(s.to_string())
    }
    fn var(s: &str) -> Term {
        Term::Var(Var::new(s))
    }

    // A source holding a set of predicates, identified by `id`, with a COMPLETE authority set
    // (so the upstream authority prune is active — used where we want a foreign-authority bound
    // position to legitimately empty a pattern).
    fn source(id: &str, preds: &[&str]) -> SourceDescriptor {
        source_with_authorities(id, preds, true)
    }

    // A source whose authority set is INCOMPLETE (the realistic VoID-parsed posture) — a bound
    // foreign-authority position never prunes it (recall-safe).
    fn source_incomplete(id: &str, preds: &[&str]) -> SourceDescriptor {
        source_with_authorities(id, preds, false)
    }

    fn source_with_authorities(id: &str, preds: &[&str], complete: bool) -> SourceDescriptor {
        let mut b = SourceDescriptor::builder(SourceId::new(id)).total_triples(100);
        for p in preds {
            b = b.predicate(PredPartition {
                predicate: (*p).to_string(),
                triples: 100,
                distinct_subjects: 10,
                distinct_objects: 10,
            });
        }
        if complete {
            b = b.authorities_complete();
        }
        b.build()
    }

    // ── Tests ───────────────────────────────────────────────────────────────────────────

    /// A satisfiable connected 2-pattern BGP is a pure pass-through: nothing pruned, the
    /// selection carried through verbatim, the BGP one live component.
    #[test]
    fn satisfiable_bgp_carries_through_unchanged() {
        // ?p sharesHouse ?h . ?p name ?n   — joins on ?p.
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("p"), iri(SHARES_HOUSE), var("h")),
            TriplePattern::new(var("p"), iri(NAME), var("n")),
        ]);
        let sources = vec![source("http://a/", &[SHARES_HOUSE, NAME])];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();

        let out = prune_source_combinations(&bgp, &sources, &selected).unwrap();
        assert!(out.bgp_satisfiable);
        assert!(out.empty_patterns.is_empty());
        assert!(out.pruned.is_empty(), "nothing pruned on the live path");
        // The Phase-2 selection is carried through verbatim.
        assert_eq!(out.per_pattern, selected.per_pattern);
        // One connected component covering both patterns, satisfiable.
        assert_eq!(out.components.len(), 1);
        assert_eq!(out.components[0].patterns, vec![0, 1]);
        assert!(out.components[0].satisfiable);
        assert_eq!(out.surviving_combination_patterns(), vec![0, 1]);
        assert!(!out.is_bgp_dead());
    }

    /// Positive control (the required four-flatmates shape): a four-pattern star/chain where
    /// every pattern is held by some source must NOT fire the prune — a healthy federated query
    /// keeps every combination.
    #[test]
    fn four_flatmates_positive_control() {
        // ?p sharesHouse :H . ?p name ?n . ?p owes ?amt . ?p memberOf :Flat — a star on ?p.
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("p"), iri(SHARES_HOUSE), iri("http://ex/H")),
            TriplePattern::new(var("p"), iri(NAME), var("n")),
            TriplePattern::new(var("p"), iri(OWES), var("amt")),
            TriplePattern::new(var("p"), iri(MEMBER_OF), iri("http://ex/Flat")),
        ]);
        // Four flatmate sources, each holding all four predicates (so every pattern is non-empty).
        let sources = vec![
            source("http://a/", &[SHARES_HOUSE, NAME, OWES, MEMBER_OF]),
            source("http://b/", &[SHARES_HOUSE, NAME, OWES, MEMBER_OF]),
            source("http://c/", &[SHARES_HOUSE, NAME, OWES, MEMBER_OF]),
            source("http://d/", &[SHARES_HOUSE, NAME, OWES, MEMBER_OF]),
        ];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        let out = prune_source_combinations(&bgp, &sources, &selected).unwrap();

        assert!(
            out.bgp_satisfiable,
            "a healthy four-flatmates query is satisfiable"
        );
        assert!(out.pruned.is_empty(), "no combination pruned");
        // One connected star component over all four patterns.
        assert_eq!(out.components.len(), 1);
        assert_eq!(out.components[0].patterns, vec![0, 1, 2, 3]);
        assert!(out.components[0].satisfiable);
    }

    /// Rule C1: an empty conjunct in a CONNECTED BGP makes the whole BGP dead — every surviving
    /// candidate of the non-empty patterns is recorded combination-dead with the empty pattern
    /// as the witness.
    #[test]
    fn empty_pattern_makes_whole_connected_bgp_dead() {
        // Pattern 1's predicate is held by no source ⇒ Phase 2 empties it.
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("p"), iri(SHARES_HOUSE), var("h")),
            TriplePattern::new(var("p"), iri(ABSENT), var("z")), // no source holds ABSENT.
        ]);
        let sources = vec![source("http://a/", &[SHARES_HOUSE])];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        // Confirm the upstream really emptied pattern 1.
        assert!(selected.per_pattern[1].is_empty());

        let out = prune_source_combinations(&bgp, &sources, &selected).unwrap();
        assert!(!out.bgp_satisfiable);
        assert!(out.is_bgp_dead());
        assert_eq!(out.empty_patterns, vec![1]);
        // The one surviving candidate (pattern 0, source 0) is recorded dead with witness 1.
        assert_eq!(out.pruned.len(), 1);
        assert_eq!(out.pruned[0].pattern, 0);
        assert_eq!(out.pruned[0].source, 0);
        assert_eq!(
            out.pruned[0].reason,
            CombinationPruneReason::UnsatisfiableBgp { witness: 1 }
        );
        assert!(out.pruned[0].reason.label().contains("pattern 1"));
        // No pattern is live for a full answer.
        assert!(out.surviving_combination_patterns().is_empty());
    }

    /// The whole-BGP verdict holds regardless of connectivity: an empty pattern in one
    /// DISCONNECTED component still makes the full BGP unsatisfiable, while the components are
    /// reported with their individual satisfiability.
    #[test]
    fn empty_pattern_in_disconnected_bgp_still_kills_full_answer() {
        // Component A: ?p sharesHouse ?h (non-empty). Component B: ?x owes ?y joined with
        // ?x absent ?z — B's second pattern is empty.
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("p"), iri(SHARES_HOUSE), var("h")), // comp A (alone)
            TriplePattern::new(var("x"), iri(OWES), var("y")),         // comp B
            TriplePattern::new(var("x"), iri(ABSENT), var("z")),       // comp B (empty)
        ]);
        let sources = vec![source("http://a/", &[SHARES_HOUSE, OWES])];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();

        let out = prune_source_combinations(&bgp, &sources, &selected).unwrap();
        assert!(
            !out.bgp_satisfiable,
            "any empty conjunct kills the whole BGP"
        );
        assert_eq!(out.empty_patterns, vec![2]);
        // Two components: A = {0}, B = {1, 2}. A satisfiable, B not.
        assert_eq!(out.components.len(), 2);
        assert_eq!(out.components[0].patterns, vec![0]);
        assert!(out.components[0].satisfiable);
        assert_eq!(out.components[1].patterns, vec![1, 2]);
        assert!(!out.components[1].satisfiable);
    }

    /// Multiple empty patterns: the witness is the SMALLEST empty index (deterministic), and the
    /// audit trail is sorted by (pattern, source).
    #[test]
    fn multiple_empty_patterns_pick_smallest_witness_deterministically() {
        // Patterns 1 and 3 are empty (ABSENT); 0 and 2 are non-empty.
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("p"), iri(SHARES_HOUSE), var("h")), // 0 non-empty
            TriplePattern::new(var("p"), iri(ABSENT), var("a")),       // 1 empty
            TriplePattern::new(var("p"), iri(NAME), var("n")),         // 2 non-empty
            TriplePattern::new(var("p"), iri(ABSENT), var("b")),       // 3 empty
        ]);
        let sources = vec![source("http://a/", &[SHARES_HOUSE, NAME])];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();

        let out = prune_source_combinations(&bgp, &sources, &selected).unwrap();
        assert!(!out.bgp_satisfiable);
        assert_eq!(out.empty_patterns, vec![1, 3]);
        // Both surviving candidates (patterns 0 and 2) name witness 1.
        assert_eq!(out.pruned.len(), 2);
        for pc in &out.pruned {
            assert_eq!(
                pc.reason,
                CombinationPruneReason::UnsatisfiableBgp { witness: 1 }
            );
        }
        // Sorted by (pattern, source).
        assert_eq!(out.pruned[0].pattern, 0);
        assert_eq!(out.pruned[1].pattern, 2);
    }

    /// Recall-safety negative control for the DECLINED value-overlap non-rule: a join `?v p c`
    /// (object bound IRI) ⋈ `?v q ?z` where the source holding `q` has an authority set disjoint
    /// from `c` must NOT prune the source — the module does not invent an unsound bound-IRI
    /// propagation prune across a join variable.
    #[test]
    fn recall_safe_no_value_overlap_prune() {
        // Pattern 0: ?v sharesHouse <http://h-authority.example/H> (object on a foreign
        // authority). Pattern 1: ?v name ?n. They join on ?v.
        let bgp = Bgp::new(vec![
            TriplePattern::new(
                var("v"),
                iri(SHARES_HOUSE),
                iri("http://h-authority.example/H"),
            ),
            TriplePattern::new(var("v"), iri(NAME), var("n")),
        ]);
        // Source A holds sharesHouse (its authority is http://ex, NOT h-authority.example) and
        // name, with an INCOMPLETE authority set (the realistic VoID-parsed posture), so the
        // foreign-authority bound OBJECT does NOT prune A per-pattern — and crucially the join
        // variable ?v is NOT forced to the constant. Both patterns therefore retain A: the
        // scenario the declined value-overlap non-rule would (wrongly) try to prune.
        let sources = vec![source_incomplete("http://ex/a", &[SHARES_HOUSE, NAME])];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        // Positive precondition: BOTH patterns kept A (so we are genuinely on the live path).
        assert!(
            !selected.per_pattern[0].is_empty(),
            "pattern 0 keeps the source"
        );
        assert!(
            !selected.per_pattern[1].is_empty(),
            "pattern 1 keeps the source"
        );

        let out = prune_source_combinations(&bgp, &sources, &selected).unwrap();
        // The BGP is satisfiable and NOTHING is pruned — no unsound value-overlap inference
        // across the join variable, no bound-IRI propagation.
        assert!(out.bgp_satisfiable);
        assert!(
            out.pruned.is_empty(),
            "no candidate is pruned while the BGP is satisfiable (value-overlap prune declined)"
        );
        assert_eq!(out.surviving_combination_patterns(), vec![0, 1]);
    }

    /// The pass trusts the upstream recall-safe emptiness: when `select_sources`' complete-
    /// authority prune legitimately empties a pattern, that pattern is the witness — the
    /// combination pass does not re-derive or widen the capability prune.
    #[test]
    fn complete_authority_emptiness_is_respected_not_reproduced() {
        // A complete-authority source minting only `http://ex` terms; a pattern with a foreign-
        // authority bound subject is emptied upstream.
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("p"), iri(SHARES_HOUSE), var("h")), // 0 non-empty
            TriplePattern::new(
                iri("http://foreign.example/Subj"), // foreign-authority bound subject
                iri(NAME),
                var("n"),
            ), // 1 emptied by the complete-authority prune
        ]);
        let sources = vec![source("http://ex/a", &[SHARES_HOUSE, NAME])]; // authorities_complete()
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        assert!(
            selected.per_pattern[1].is_empty(),
            "upstream complete-authority prune empties pattern 1"
        );

        let out = prune_source_combinations(&bgp, &sources, &selected).unwrap();
        assert!(!out.bgp_satisfiable);
        assert_eq!(out.empty_patterns, vec![1]);
        // The witness is the upstream-emptied pattern; the live candidate (pattern 0) is dead.
        assert_eq!(out.pruned.len(), 1);
        assert_eq!(
            out.pruned[0].reason,
            CombinationPruneReason::UnsatisfiableBgp { witness: 1 }
        );
    }

    /// A pattern emptied by the Phase-2 PARTICIPATION prune (not by `select_sources`) is an
    /// equally valid witness — the rule keys on Phase-2 emptiness whatever its cause, and the
    /// participation prune is itself recall-safe.
    #[test]
    fn participation_emptied_pattern_triggers_collapse() {
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("p"), iri(SHARES_HOUSE), var("h")), // 0
            TriplePattern::new(var("p"), iri(NAME), var("n")),         // 1
        ]);
        // The only source holding `name` declares it will not participate ⇒ Phase 2 empties
        // pattern 1 (the source holds sharesHouse too, so pattern 0 survives only if another
        // source holds it; give pattern 0 its own participating source).
        let sources = vec![
            source("http://keep/", &[SHARES_HOUSE]),
            source("http://refuse/", &[NAME]),
        ];
        let privacy = vec![
            SourcePrivacyDescriptor::builder(SourceId::new("http://refuse/"))
                .participates(false)
                .build(),
        ];
        let selected = select_private_sources(&bgp, &sources, &privacy).unwrap();
        assert!(
            selected.per_pattern[1].is_empty(),
            "the participation prune empties pattern 1"
        );

        let out = prune_source_combinations(&bgp, &sources, &selected).unwrap();
        assert!(!out.bgp_satisfiable);
        assert_eq!(out.empty_patterns, vec![1]);
        // Pattern 0's surviving candidate is recorded dead.
        assert_eq!(out.pruned.len(), 1);
        assert_eq!(out.pruned[0].pattern, 0);
    }

    /// A single-pattern BGP: satisfiable with a source (no prune), unsatisfiable without one
    /// (empty witness, nothing to mark, but the BGP is dead).
    #[test]
    fn single_pattern_bgp_satisfiable_and_empty_cases() {
        let bgp = Bgp::new(vec![TriplePattern::new(
            var("p"),
            iri(SHARES_HOUSE),
            var("h"),
        )]);
        // Satisfiable.
        let sources = vec![source("http://a/", &[SHARES_HOUSE])];
        let sel = select_private_sources(&bgp, &sources, &[]).unwrap();
        let out = prune_source_combinations(&bgp, &sources, &sel).unwrap();
        assert!(out.bgp_satisfiable);
        assert!(out.pruned.is_empty());
        assert_eq!(out.components.len(), 1);

        // Unsatisfiable: no source holds the predicate.
        let none = vec![source("http://a/", &[NAME])]; // holds name, not sharesHouse.
        let sel2 = select_private_sources(&bgp, &none, &[]).unwrap();
        let out2 = prune_source_combinations(&bgp, &none, &sel2).unwrap();
        assert!(!out2.bgp_satisfiable);
        assert_eq!(out2.empty_patterns, vec![0]);
        assert!(
            out2.pruned.is_empty(),
            "the empty pattern has no candidate to mark"
        );
        assert!(out2.is_bgp_dead());
    }

    /// Components are deterministic and sorted: a BGP with two components and a shared-variable
    /// chain reports them sorted by minimum index, each with ascending members.
    #[test]
    fn components_are_deterministic_and_sorted() {
        // Comp X: 0-1-2 chained on ?p then ?q. Comp Y: 3 alone.
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("p"), iri(SHARES_HOUSE), var("q")), // 0 (p,q)
            TriplePattern::new(var("q"), iri(NAME), var("r")),         // 1 joins 0 on q
            TriplePattern::new(var("r"), iri(OWES), var("s")),         // 2 joins 1 on r
            TriplePattern::new(var("z"), iri(MEMBER_OF), var("w")),    // 3 disjoint
        ]);
        let sources = vec![source("http://a/", &[SHARES_HOUSE, NAME, OWES, MEMBER_OF])];
        let sel = select_private_sources(&bgp, &sources, &[]).unwrap();
        let out = prune_source_combinations(&bgp, &sources, &sel).unwrap();
        assert_eq!(out.components.len(), 2);
        assert_eq!(out.components[0].patterns, vec![0, 1, 2]);
        assert_eq!(out.components[1].patterns, vec![3]);
        // Sorted by min index: component {0,1,2} first.
        assert!(out.components[0].patterns[0] < out.components[1].patterns[0]);
    }

    /// Determinism: identical inputs ⇒ identical output, twice (including the unsatisfiable
    /// path).
    #[test]
    fn determinism_same_inputs_same_output() {
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("p"), iri(SHARES_HOUSE), var("h")),
            TriplePattern::new(var("p"), iri(ABSENT), var("z")),
        ]);
        let sources = vec![source("http://a/", &[SHARES_HOUSE])];
        let sel = select_private_sources(&bgp, &sources, &[]).unwrap();
        let a = prune_source_combinations(&bgp, &sources, &sel).unwrap();
        let b = prune_source_combinations(&bgp, &sources, &sel).unwrap();
        assert_eq!(a, b);
    }

    /// A longer shared-variable chain (0-1-2-3-4) must still collapse to ONE component with all
    /// five patterns — the union-find path-compression over a transitive chain produces a single
    /// connected component, exercising the multi-level `find` walk. A regression in the component
    /// merge (e.g. losing transitivity) would split the chain. [OPUS-4.8] sq-bif.
    #[test]
    fn long_shared_variable_chain_is_one_component() {
        // ?a p ?b . ?b p ?c . ?c p ?d . ?d p ?e . ?e p ?f  — a five-pattern chain, each joining
        // the next on the shared variable.
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("a"), iri(SHARES_HOUSE), var("b")),
            TriplePattern::new(var("b"), iri(NAME), var("c")),
            TriplePattern::new(var("c"), iri(OWES), var("d")),
            TriplePattern::new(var("d"), iri(MEMBER_OF), var("e")),
            TriplePattern::new(var("e"), iri(SHARES_HOUSE), var("f")),
        ]);
        let sources = vec![source("http://a/", &[SHARES_HOUSE, NAME, OWES, MEMBER_OF])];
        let sel = select_private_sources(&bgp, &sources, &[]).unwrap();
        let out = prune_source_combinations(&bgp, &sources, &sel).unwrap();
        assert!(out.bgp_satisfiable);
        // Exactly one component covering the whole transitive chain, ascending.
        assert_eq!(out.components.len(), 1);
        assert_eq!(out.components[0].patterns, vec![0, 1, 2, 3, 4]);
        assert!(out.components[0].satisfiable);
    }

    /// `BgpComponent::satisfiable` is per-component: in a disconnected BGP with one empty and one
    /// non-empty component, the non-empty component reports `satisfiable == true` while the empty
    /// one reports `false` — even though the WHOLE-BGP verdict is unsatisfiable. This pins the
    /// distinction between the per-component flag and the global `bgp_satisfiable`. [OPUS-4.8]
    /// sq-bif.
    #[test]
    fn per_component_satisfiability_is_independent_of_the_global_verdict() {
        // Comp A: 0-1 chained on ?p, both non-empty (satisfiable). Comp B: 2 alone, empty.
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("p"), iri(SHARES_HOUSE), var("h")), // 0 comp A
            TriplePattern::new(var("p"), iri(NAME), var("n")),         // 1 comp A
            TriplePattern::new(var("z"), iri(ABSENT), var("w")),       // 2 comp B (empty)
        ]);
        let sources = vec![source("http://a/", &[SHARES_HOUSE, NAME])];
        let sel = select_private_sources(&bgp, &sources, &[]).unwrap();
        let out = prune_source_combinations(&bgp, &sources, &sel).unwrap();

        // Global verdict: unsatisfiable (an empty conjunct kills the whole BGP).
        assert!(!out.bgp_satisfiable);
        assert_eq!(out.empty_patterns, vec![2]);

        // But the per-component flag distinguishes the live component from the dead one.
        assert_eq!(out.components.len(), 2);
        let comp_a = out
            .components
            .iter()
            .find(|c| c.patterns == vec![0, 1])
            .expect("the {0,1} component exists");
        assert!(
            comp_a.satisfiable,
            "the non-empty component is itself satisfiable"
        );
        let comp_b = out
            .components
            .iter()
            .find(|c| c.patterns == vec![2])
            .expect("the {2} component exists");
        assert!(!comp_b.satisfiable, "the empty component is unsatisfiable");
    }

    /// `CombinationPruneReason::label` names the witnessing pattern explicitly (a regression that
    /// blanked it or dropped the witness index would lose the audit-trail diagnostic). [OPUS-4.8]
    /// sq-bif.
    #[test]
    fn combination_prune_reason_label_names_the_witness_pattern() {
        let label = CombinationPruneReason::UnsatisfiableBgp { witness: 7 }.label();
        assert!(
            label.contains("unsatisfiable"),
            "label names the cause: {label}"
        );
        assert!(
            label.contains("pattern 7"),
            "label names the witness pattern: {label}"
        );
    }

    /// A selection whose pattern count does not match the BGP is a fail-closed
    /// DescriptorMismatch (phase SourceCombination) — never guess the alignment.
    #[test]
    fn selection_bgp_length_mismatch_is_descriptor_mismatch() {
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("p"), iri(SHARES_HOUSE), var("h")),
            TriplePattern::new(var("p"), iri(NAME), var("n")),
        ]);
        let sources = vec![source("http://a/", &[SHARES_HOUSE, NAME])];
        // Hand-craft a one-pattern selection against a two-pattern BGP.
        let mismatched = SelectedPrivateSources {
            per_pattern: vec![PrivatePatternSources {
                pattern: 0,
                candidates: vec![PrivateCandidate {
                    source: 0,
                    source_id: SourceId::new("http://a/"),
                    estimated_cardinality: 1.0,
                }],
            }],
            pruned: Vec::new(),
        };
        let err = prune_source_combinations(&bgp, &sources, &mismatched).unwrap_err();
        assert!(matches!(
            err,
            SeamError::DescriptorMismatch {
                phase: SeamPhase::SourceCombination,
                ..
            }
        ));
        assert!(format!("{}", err).contains("cannot align combinations"));
    }
}
