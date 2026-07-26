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
//! * **The declined non-rule — value-overlap / bound-IRI propagation *from the
//!   [`SourceDescriptor`] summary*.** A tempting idea is to propagate a bound IRI constant
//!   across a shared join variable and prune a source whose value space cannot reach it. This is
//!   **not** recall-safely expressible from the public [`SourceDescriptor`] API, and declining it
//!   is a *correctness* decision, not an omission:
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
//! * **Rule C2 — quotient-summary provenance (sq-xkrt).** The way *out* of that declined
//!   non-rule is not a cleverer reading of [`SourceDescriptor`] but FedUP's actual lever: a
//!   **different input**. When a source publishes a [`SourceQuotientSummary`] and declares it a
//!   complete over-approximation of its graph, the BGP can be *evaluated at plan time* over the
//!   quotient, per source-combination, and a combination whose quotient-level evaluation is
//!   empty **provably** yields no concrete answer (the soundness argument lives in
//!   [`crate::quotient`]). That is the genuine result-aware prune: it can kill a combination
//!   whose patterns are individually non-empty but *jointly* unsatisfiable — e.g. two sources
//!   that each hold the join predicate but over disjoint authorities. It is opt-in per source
//!   (no summary, or an incomplete one ⇒ that source constrains nothing and is never pruned),
//!   bounded by an explicit [`CombinationBudget`] (an over-budget enumeration **declines** and
//!   prunes nothing), and it is reached only through
//!   [`prune_source_combinations_with_summaries`] — plain [`prune_source_combinations`] is
//!   unchanged, Rule C1 only.
//!
//! # Recall-safety (the load-bearing property, mirroring the upstream invariant)
//!
//! > **A (pattern, source) candidate is marked combination-dead only when no full-BGP answer
//! > can use it; on any uncertainty it is kept.**
//!
//! Under Rule C1 the pass marks candidates dead **only** when some pattern is empty, i.e. when
//! the conjunction is *proved* unsatisfiable (an empty conjunct ⇒ no answers exist ⇒ marking
//! every candidate dead loses nothing). When every pattern is non-empty it drops **nothing** —
//! there is no further recall-safe combination prune available from the [`SourceDescriptor`]
//! summary alone. It reads no descriptor capability of its own and adds no independent prune
//! that could be unsound; it merely **propagates** an already-recall-safe emptiness verdict
//! across the conjunction.
//!
//! Rule C2 keeps the same invariant against a *stronger* input: a candidate is dead only when
//! **every** source-combination containing it evaluates to the empty relation over the quotient
//! summaries — and a source that published no complete summary is treated as matching anything,
//! so it is never the reason a combination dies. Every way the enumeration can fail to be
//! decisive (a predicate-variable domain clash, an over-budget combination space, an over-wide
//! intermediate join) **declines** and prunes nothing, so uncertainty always resolves to *keep*.
//!
//! # Honesty / threat model (no claim beyond selection plumbing)
//!
//! This is **plumbing — result-aware source-combination routing, not a cryptographic
//! guarantee.** It performs **NO** MPC, runs **NO** secret-sharing, opens nothing, verifies
//! nothing, and reads only the (public) Phase-2 selection and the (public, source-published)
//! quotient summaries it is handed. It makes **NO** soundness/privacy/security claim. The MPC
//! estate (`sparq-mpc`) is research-grade, **honest-majority semi-honest only**, and is **NOT**
//! externally audited — the accredited-cryptographer sign-off (sq-qhy4) and the coZK re-audit
//! (sq-9hrn) are pending. This pass does not change that posture by one inch. The only thing it
//! "reveals" is which combinations are infeasible, derived entirely from the caller's own
//! descriptors; the *publication* of a quotient summary is itself a disclosure the source opts
//! into, enumerated honestly in [`crate::quotient`]. See the crate `README.md` and
//! `research/mpc-untrusted-planner-routing-design.md`.
//!
//! [OPUS-4.8] sq-pwr.3 (Rule C1) / sq-xkrt (Rule C2).

use std::collections::{BTreeMap, BTreeSet};

use sparq_fedplan::{Bgp, SourceDescriptor, SourceId, Term, TriplePattern};

use crate::quotient::{quotient_iri, QuotientTerm, SourceQuotientSummary};
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
    /// **Rule C2** — every source-combination that would use this candidate evaluates to the
    /// empty relation over the participating sources' quotient summaries, so no full-BGP answer
    /// can route through it. Only produced by
    /// [`prune_source_combinations_with_summaries`]; see [`crate::quotient`] for why an empty
    /// quotient-level evaluation proves the absence of a concrete answer.
    EmptyProvenance,
}

impl CombinationPruneReason {
    /// A short human label for the reason (for diagnostics / the audit trail).
    pub fn label(self) -> String {
        match self {
            CombinationPruneReason::UnsatisfiableBgp { witness } => format!(
                "full BGP unsatisfiable: conjunct (pattern {}) has no contributing source",
                witness
            ),
            CombinationPruneReason::EmptyProvenance => {
                "empty quotient-summary provenance: no live source-combination uses this candidate"
                    .to_string()
            }
        }
    }
}

/// One **live source-combination** surviving the Rule-C2 provenance prune: an assignment of one
/// source to each BGP pattern whose quotient-level evaluation is non-empty.
///
/// This is the FedUP-shaped deliverable — the (much smaller) set of combinations the seam still
/// has to consider routing toward MPC. It is an over-approximation: a live combination is *not*
/// promised to produce an answer, only not provably barred from one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCombination {
    /// One **source index** (the same index [`crate::PrivateCandidate::source`] carries) per BGP
    /// pattern, in pattern order.
    pub assignment: Vec<usize>,
}

/// The bound on the Rule-C2 provenance enumeration. The combination space is the *product* of the
/// per-pattern candidate counts, so it is worst-case exponential in the BGP size; this budget
/// keeps plan time bounded, and exceeding it **declines** the prune (recall-safe: nothing is
/// dropped) rather than truncating the search (which would be recall-*unsafe*, because a
/// truncated search cannot prove a combination dead).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct CombinationBudget {
    /// Maximum number of source-combinations to evaluate. Exceeded ⇒
    /// [`SummaryDeclineReason::TooManyCombinations`].
    pub max_combinations: usize,
    /// Maximum number of intermediate quotient-level bindings held while joining ONE
    /// combination. Exceeded ⇒ [`SummaryDeclineReason::JoinTooWide`].
    pub max_intermediate_bindings: usize,
}

impl CombinationBudget {
    /// A budget with explicit limits. Both must be non-zero to be useful; zero simply declines
    /// everything (still recall-safe — it prunes nothing).
    pub fn new(max_combinations: usize, max_intermediate_bindings: usize) -> CombinationBudget {
        CombinationBudget {
            max_combinations,
            max_intermediate_bindings,
        }
    }
}

impl Default for CombinationBudget {
    /// A deliberately modest default: federated BGPs that reach the MPC seam are small, and the
    /// point of the pass is to *save* work, not to spend an unbounded amount proving a prune.
    fn default() -> CombinationBudget {
        CombinationBudget::new(4096, 4096)
    }
}

/// Why the Rule-C2 provenance pass **declined** to prune. Every variant means "nothing was
/// pruned" — declining is the recall-safe outcome, never a silent partial result.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SummaryDeclineReason {
    /// A variable occurs in a **predicate** position in one pattern and in a subject/object
    /// position in another. Predicates are kept concrete in a summary while subjects/objects are
    /// quotiented, so such a variable has no well-defined quotient image and a join on it could
    /// compare two different domains — which would prune unsoundly. See [`crate::quotient`].
    PredicateVariableDomainClash {
        /// The offending variable's name (the lexicographically smallest, for determinism).
        var: String,
    },
    /// The product of the per-pattern candidate counts exceeds
    /// [`CombinationBudget::max_combinations`].
    TooManyCombinations {
        /// The combination count (saturating: `usize::MAX` means the product overflowed).
        combinations: usize,
        /// The budget that was exceeded.
        budget: usize,
    },
    /// Joining one combination's quotient relations exceeded
    /// [`CombinationBudget::max_intermediate_bindings`].
    JoinTooWide {
        /// The budget that was exceeded.
        budget: usize,
    },
}

impl SummaryDeclineReason {
    /// A short human label for the reason (for diagnostics / the audit trail).
    pub fn label(&self) -> String {
        match self {
            SummaryDeclineReason::PredicateVariableDomainClash { var } => format!(
                "declined: variable ?{} is used in both a predicate and a subject/object position",
                var
            ),
            SummaryDeclineReason::TooManyCombinations {
                combinations,
                budget,
            } => format!(
                "declined: {} source-combinations exceed the budget of {}",
                combinations, budget
            ),
            SummaryDeclineReason::JoinTooWide { budget } => format!(
                "declined: a combination's quotient join exceeded {} intermediate bindings",
                budget
            ),
        }
    }
}

/// What the Rule-C2 quotient-summary provenance pass did — surfaced so a relying party can tell
/// "nothing was prunable" from "the pass never ran", rather than reading an empty audit trail as
/// a verdict.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum SummaryPruneOutcome {
    /// The pass was not attempted: no **complete** quotient summary was supplied for any source.
    /// The default — and what plain [`prune_source_combinations`] always reports.
    #[default]
    NotAttempted,
    /// Rule C1 already proved the BGP unsatisfiable (some conjunct is empty), so every
    /// combination is dead and there is nothing left for the provenance pass to prune.
    BgpAlreadyDead,
    /// The pass ran but **declined** — nothing was pruned. See the reason.
    Declined(SummaryDeclineReason),
    /// The pass ran to completion.
    Applied {
        /// How many source-combinations were evaluated.
        combinations_considered: usize,
        /// How many of them survived (their quotient-level evaluation was non-empty). Zero means
        /// the whole BGP was proved answer-free by the summaries — `bgp_satisfiable` is then
        /// `false` even though no conjunct was empty.
        combinations_live: usize,
    },
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
    /// component still makes the **whole** BGP unsatisfiable — see [`PrunedCombinations`]. This
    /// is a **Rule-C1 (Phase-2 emptiness) flag only**: a component can read `satisfiable == true`
    /// while the Rule-C2 provenance pass has proved every source-combination dead, so read
    /// [`PrunedCombinations::bgp_satisfiable`] for the overall verdict.
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
    /// Whether **any** full-BGP source-combination can produce an answer. `false` when some
    /// pattern is empty after Phase 2 (Rule C1: an empty conjunct ⇒ the whole conjunction is
    /// empty, regardless of the BGP's connectivity) — or, under Rule C2, when every
    /// source-combination's quotient-level evaluation was empty (`summary_prune` is
    /// [`SummaryPruneOutcome::Applied`] with `combinations_live == 0`).
    pub bgp_satisfiable: bool,
    /// The empty-pattern witness indices (ascending) that prove unsatisfiability; empty when
    /// `bgp_satisfiable`.
    pub empty_patterns: Vec<usize>,
    /// The connected components of the BGP join graph (sorted by minimum pattern index;
    /// deterministic). Structural metadata for the auditor — the Rule-C1 verdict is the simple
    /// "any empty conjunct" theorem, not a per-component decision.
    pub components: Vec<BgpComponent>,
    /// The combination-dead candidates with reasons, ascending by `(pattern, source)`. Under
    /// Rule C1 alone this is empty whenever `bgp_satisfiable` (nothing is dropped on the live
    /// path); under Rule C2 it also carries the [`CombinationPruneReason::EmptyProvenance`]
    /// candidates, which CAN be non-empty on an otherwise-satisfiable BGP.
    pub pruned: Vec<PrunedCombination>,
    /// What the Rule-C2 quotient-summary provenance pass did.
    /// [`SummaryPruneOutcome::NotAttempted`] for plain [`prune_source_combinations`].
    pub summary_prune: SummaryPruneOutcome,
    /// The source-combinations that survived the Rule-C2 provenance prune — the reduced set the
    /// seam still has to consider routing toward MPC. Populated **only** when `summary_prune` is
    /// [`SummaryPruneOutcome::Applied`] (the combination space is exponential, so it is not
    /// enumerated on the paths that prove nothing about it).
    pub live_combinations: Vec<SourceCombination>,
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
/// `sources` is read only to *validate* the selection's shape (that each candidate's source index
/// addresses the slice and agrees with the id it carries); Rule C1 does **not** re-read descriptor
/// capabilities — it operates over the already-recall-safe Phase-2 emptiness, so it never
/// introduces an independent (possibly unsound) capability prune. The declined value-overlap
/// non-rule (see the module docs) is *why* no descriptor capability is re-tested here.
///
/// **Recall-safe by construction:** candidates are marked dead **only** when the conjunction is
/// *proved* unsatisfiable (some pattern empty ⇒ no answers exist). When every pattern is
/// non-empty, **nothing** is pruned. Deterministic throughout (index-ascending iteration,
/// components sorted by minimum index, the audit trail explicitly sorted).
///
/// # Errors
///
/// Returns [`SeamError::DescriptorMismatch`] (phase [`SeamPhase::SourceCombination`]) if the
/// Phase-2 selection is not aligned with the BGP — a pattern count that differs, an entry at
/// position `i` naming a pattern other than `i` (out of range, duplicated or reordered), or a
/// candidate whose source index is outside `sources` or disagrees with the id it carries. The pass
/// refuses to guess the alignment rather than silently mis-attribute a pattern (fail-closed, the
/// same posture as Phase 2). It never panics and performs **no** MPC.
///
/// This function makes **no** soundness/privacy claim — see the module docs and the crate
/// `README.md`. [OPUS-4.8] sq-pwr.3.
pub fn prune_source_combinations(
    bgp: &Bgp,
    sources: &[SourceDescriptor],
    selected: &SelectedPrivateSources,
) -> Result<PrunedCombinations, SeamError> {
    // Fail-closed on a selection/BGP shape mismatch — never guess which pattern is which.
    if selected.per_pattern.len() != bgp.patterns.len() {
        return Err(SeamError::DescriptorMismatch {
            phase: SeamPhase::SourceCombination,
            source_id: String::new(),
            detail:
                "Phase-2 selection pattern count does not match the BGP (cannot align combinations)",
        });
    }
    // A matching length alone does NOT pin the alignment, and both [`SelectedPrivateSources`] and
    // [`PrivatePatternSources`] are publicly constructible, so a mis-aligned selection is a
    // reachable state rather than an internal invariant: a duplicated index (`[0, 0]` over a
    // two-pattern BGP) would evaluate one conjunct twice and silently omit another, and a
    // reordered one would attribute [`SourceCombination::assignment`] entries — documented as
    // BGP-pattern order — to the wrong pattern. Require entry `i` to BE pattern `i`, which
    // subsumes the range check, and validate each candidate against `sources` (the slice
    // `PrivateCandidate::source` indexes and `SourceCombination::assignment` hands downstream,
    // and whose id Rule C2 keys its summary lookup on).
    for (i, ps) in selected.per_pattern.iter().enumerate() {
        if ps.pattern >= bgp.patterns.len() {
            return Err(SeamError::DescriptorMismatch {
                phase: SeamPhase::SourceCombination,
                source_id: String::new(),
                detail: "Phase-2 selection names a pattern index outside the BGP",
            });
        }
        if ps.pattern != i {
            return Err(SeamError::DescriptorMismatch {
                phase: SeamPhase::SourceCombination,
                source_id: String::new(),
                detail: "Phase-2 selection is not in BGP pattern order (cannot align combinations)",
            });
        }
        for cand in &ps.candidates {
            let Some(descriptor) = sources.get(cand.source) else {
                return Err(SeamError::DescriptorMismatch {
                    phase: SeamPhase::SourceCombination,
                    source_id: cand.source_id.0.clone(),
                    detail: "Phase-2 selection names a source index outside the descriptor slice",
                });
            };
            if descriptor.id() != &cand.source_id {
                return Err(SeamError::DescriptorMismatch {
                    phase: SeamPhase::SourceCombination,
                    source_id: cand.source_id.0.clone(),
                    detail:
                        "Phase-2 candidate's source id does not match the descriptor at its index",
                });
            }
        }
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
        // Rule C1 alone never runs the provenance pass.
        summary_prune: SummaryPruneOutcome::NotAttempted,
        live_combinations: Vec::new(),
    })
}

/// **Phase 6, Rule C2** — [`prune_source_combinations`] *plus* the FedUP-style **quotient-summary
/// provenance** prune (design record §3 / §8 Phase 6; bead sq-xkrt).
///
/// Runs Rule C1 first, then — when the BGP is still satisfiable and at least one source published
/// a **complete** [`SourceQuotientSummary`] — evaluates the BGP at the quotient level once per
/// source-combination. A combination whose evaluation is empty **provably** produces no concrete
/// answer (the over-approximation argument in [`crate::quotient`]), so it is dropped from
/// [`PrunedCombinations::live_combinations`]; a candidate that no live combination uses is
/// recorded [`CombinationPruneReason::EmptyProvenance`] in the audit trail. If **no** combination
/// survives, `bgp_satisfiable` becomes `false` — the seam can skip the MPC path entirely.
///
/// This is the prune the first Phase-6 slice could not express: it kills a combination whose
/// patterns are individually non-empty but *jointly* unsatisfiable (e.g. two sources that each
/// hold the join predicate, over disjoint authorities) — the source-combination blow-up the
/// design record flags as the highest-leverage pre-MPC cost win.
///
/// **Recall-safe by construction**, in three layers:
/// 1. a source that published **no** summary, or one it did not declare complete, is treated as
///    matching anything — it is never the reason a combination dies;
/// 2. a quotient summary over-approximates its source's graph, so an empty quotient-level
///    evaluation is a *proof* that no concrete answer exists for that combination; and
/// 3. every way the enumeration could be indecisive **declines** and prunes nothing — a
///    predicate/term variable-domain clash, an over-budget combination space, or an over-wide
///    intermediate join, each reported in [`PrunedCombinations::summary_prune`].
///
/// The per-pattern candidate lists are still carried through **unchanged**: the verdict stays
/// advisory and auditable. Deterministic throughout.
///
/// # Errors
///
/// [`SeamError::DescriptorMismatch`] (phase [`SeamPhase::SourceCombination`]) when the Phase-2
/// selection does not line up with the BGP — pattern count, pattern order/range, or a candidate's
/// source index/id, exactly as [`prune_source_combinations`] — or when two [`SourceQuotientSummary`]s
/// declare the same [`SourceId`] (an ambiguous summary — the pass refuses to guess which binds).
/// It never panics and performs **no** MPC.
///
/// This function makes **no** soundness/privacy claim — see the module docs and the crate
/// `README.md`. [OPUS-4.8] sq-xkrt.
pub fn prune_source_combinations_with_summaries(
    bgp: &Bgp,
    sources: &[SourceDescriptor],
    selected: &SelectedPrivateSources,
    summaries: &[SourceQuotientSummary],
    budget: CombinationBudget,
) -> Result<PrunedCombinations, SeamError> {
    let mut out = prune_source_combinations(bgp, sources, selected)?;

    // Index the summaries by source id. A duplicate id is an ambiguous declaration: refuse rather
    // than silently letting one win (the same fail-closed posture as the Phase-2 descriptor map).
    let mut by_id: BTreeMap<&SourceId, &SourceQuotientSummary> = BTreeMap::new();
    for s in summaries {
        if by_id.insert(s.id(), s).is_some() {
            return Err(SeamError::DescriptorMismatch {
                phase: SeamPhase::SourceCombination,
                source_id: s.id().0.clone(),
                detail: "duplicate SourceQuotientSummary for this source id (ambiguous summary)",
            });
        }
    }
    // ONLY a complete summary may constrain anything — an incomplete one is not an
    // over-approximation, so nothing can be proved from it (recall-safe).
    by_id.retain(|_, s| s.is_complete());
    if by_id.is_empty() {
        out.summary_prune = SummaryPruneOutcome::NotAttempted;
        return Ok(out);
    }
    // Rule C1 already collapsed the whole combination space; there is nothing left to prove.
    if !out.bgp_satisfiable {
        out.summary_prune = SummaryPruneOutcome::BgpAlreadyDead;
        return Ok(out);
    }
    // A variable spanning the predicate and the subject/object domains has no well-defined
    // quotient image — joining on it could prune unsoundly, so decline.
    if let Some(var) = predicate_variable_domain_clash(bgp) {
        out.summary_prune = SummaryPruneOutcome::Declined(
            SummaryDeclineReason::PredicateVariableDomainClash { var },
        );
        return Ok(out);
    }

    // 1. The per-(pattern, candidate) quotient relation: the bindings that candidate's summary
    //    admits for that pattern, or `Unconstrained` when the source published no complete one.
    let mut relations: Vec<Vec<Relation>> = Vec::with_capacity(selected.per_pattern.len());
    for (i, ps) in selected.per_pattern.iter().enumerate() {
        // Rule C1 validated the alignment above (entry `i` IS pattern `i`, in range), so this
        // index is sound and the relation vector is genuinely in BGP-pattern order.
        let pattern = &bgp.patterns[i];
        relations.push(
            ps.candidates
                .iter()
                .map(|cand| match by_id.get(&cand.source_id) {
                    Some(summary) => Relation::Rows(match_pattern(pattern, summary)),
                    None => Relation::Unconstrained,
                })
                .collect(),
        );
    }

    // 2. The combination space, bounded. `checked_mul` so an overflowing product DECLINES rather
    //    than wrapping to a small number (silently enumerating the wrong thing) or saturating to
    //    `usize::MAX` (which a `max_combinations == usize::MAX` budget would then wave through,
    //    letting the enumeration below run unbounded). An overflow is reported as `usize::MAX`.
    let checked = relations
        .iter()
        .try_fold(1usize, |acc, rel| acc.checked_mul(rel.len()));
    let Some(combinations) = checked.filter(|n| *n <= budget.max_combinations) else {
        out.summary_prune =
            SummaryPruneOutcome::Declined(SummaryDeclineReason::TooManyCombinations {
                combinations: checked.unwrap_or(usize::MAX),
                budget: budget.max_combinations,
            });
        return Ok(out);
    };

    // 3. Enumerate (deterministically, candidate-index ascending) and keep the live ones.
    let mut choices: Vec<Vec<usize>> = vec![Vec::new()];
    for rel in &relations {
        // No relation is empty on this path (an empty candidate list would have made Rule C1
        // declare the BGP dead above), so every prefix product is bounded by the checked
        // `combinations` total and this capacity cannot overflow; `saturating_mul` keeps that a
        // fact of the arithmetic rather than an assumption.
        let mut next = Vec::with_capacity(choices.len().saturating_mul(rel.len()));
        for prefix in &choices {
            for c in 0..rel.len() {
                let mut extended = prefix.clone();
                extended.push(c);
                next.push(extended);
            }
        }
        choices = next;
    }
    let mut live: Vec<Vec<usize>> = Vec::new();
    for choice in &choices {
        match combination_is_live(&relations, choice, budget.max_intermediate_bindings) {
            Ok(true) => live.push(choice.clone()),
            Ok(false) => {}
            Err(reason) => {
                out.summary_prune = SummaryPruneOutcome::Declined(reason);
                return Ok(out);
            }
        }
    }

    // 4. A candidate used by NO live combination is combination-dead. (Rule C1 pruned nothing on
    //    this path — the BGP was satisfiable — so this is the whole audit trail.)
    let used: BTreeSet<(usize, usize)> = live
        .iter()
        .flat_map(|choice| choice.iter().copied().enumerate())
        .collect();
    let mut pruned: Vec<PrunedCombination> = Vec::new();
    for (i, ps) in selected.per_pattern.iter().enumerate() {
        for (c, cand) in ps.candidates.iter().enumerate() {
            if !used.contains(&(i, c)) {
                pruned.push(PrunedCombination {
                    pattern: ps.pattern,
                    source: cand.source,
                    source_id: cand.source_id.clone(),
                    reason: CombinationPruneReason::EmptyProvenance,
                });
            }
        }
    }
    pruned.sort_by_key(|p| (p.pattern, p.source));
    out.pruned = pruned;

    // 5. No surviving combination ⇒ the summaries proved the BGP answer-free, even though every
    //    conjunct is individually non-empty (so `empty_patterns` stays empty — the witness here
    //    is the provenance audit trail, not a conjunct).
    out.bgp_satisfiable = !live.is_empty();
    out.summary_prune = SummaryPruneOutcome::Applied {
        combinations_considered: combinations,
        combinations_live: live.len(),
    };
    out.live_combinations = live
        .into_iter()
        .map(|choice| SourceCombination {
            assignment: choice
                .iter()
                .enumerate()
                .map(|(i, &c)| selected.per_pattern[i].candidates[c].source)
                .collect(),
        })
        .collect();

    Ok(out)
}

/// One quotient-level binding: variable name → the class it is bound to. `BTreeMap` so bindings
/// are ordered/deduplicable and the pass is deterministic.
type Binding = BTreeMap<String, QValue>;

/// What a variable can be bound to at the quotient level. Subjects/objects bind a
/// [`QuotientTerm`]; a variable in predicate position binds the CONCRETE predicate IRI (predicates
/// are not quotiented). The two domains never mix, because a variable that would span them makes
/// the pass decline — see [`SummaryDeclineReason::PredicateVariableDomainClash`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum QValue {
    Term(QuotientTerm),
    Predicate(String),
}

/// A candidate's quotient relation for one pattern.
#[derive(Debug, Clone)]
enum Relation {
    /// The source published no complete summary: it constrains nothing and matches anything.
    Unconstrained,
    /// The quotient bindings the source's summary admits for this pattern (possibly none, which
    /// kills every combination using it).
    Rows(Vec<Binding>),
}

/// The lexicographically smallest variable used in BOTH a predicate position and a
/// subject/object position anywhere in `bgp`, if any.
fn predicate_variable_domain_clash(bgp: &Bgp) -> Option<String> {
    let mut predicate_vars: BTreeSet<&str> = BTreeSet::new();
    let mut term_vars: BTreeSet<&str> = BTreeSet::new();
    for p in &bgp.patterns {
        if let Some(v) = p.predicate.as_var() {
            predicate_vars.insert(v.0.as_str());
        }
        for t in [&p.subject, &p.object] {
            if let Some(v) = t.as_var() {
                term_vars.insert(v.0.as_str());
            }
        }
    }
    predicate_vars
        .intersection(&term_vars)
        .next()
        .map(|v| (*v).to_string())
}

/// The quotient bindings `summary` admits for `pattern` — the quotient-level evaluation of one
/// triple pattern against one source. Deduplicated and ordered (deterministic).
fn match_pattern(pattern: &TriplePattern, summary: &SourceQuotientSummary) -> Vec<Binding> {
    let mut rows: BTreeSet<Binding> = BTreeSet::new();
    for triple in summary.triples() {
        let mut binding = Binding::new();
        let matched = bind_position(
            &pattern.subject,
            &QValue::Term(triple.subject.clone()),
            &mut binding,
        ) && bind_position(
            &pattern.predicate,
            &QValue::Predicate(triple.predicate.clone()),
            &mut binding,
        ) && bind_position(
            &pattern.object,
            &QValue::Term(triple.object.clone()),
            &mut binding,
        );
        if matched {
            rows.insert(binding);
        }
    }
    rows.into_iter().collect()
}

/// Match ONE pattern position against a quotient value, extending `binding`. A repeated variable
/// within the pattern (`?x p ?x`) must bind consistently, so an already-bound variable matches
/// only its existing value.
fn bind_position(position: &Term, value: &QValue, binding: &mut Binding) -> bool {
    match position {
        Term::Var(v) => match binding.get(&v.0) {
            Some(existing) => existing == value,
            None => {
                binding.insert(v.0.clone(), value.clone());
                true
            }
        },
        // A bound IRI matches its own quotient class in a subject/object position, and the
        // concrete IRI in a predicate position (predicates are never quotiented).
        Term::Iri(iri) => match value {
            QValue::Term(q) => *q == quotient_iri(iri),
            QValue::Predicate(p) => p == iri,
        },
        // Every literal collapses to one class, so a bound literal matches any summarised literal
        // (an over-approximation — recall-safe) and never a predicate.
        Term::Literal(_) => matches!(value, QValue::Term(QuotientTerm::Literal)),
    }
}

/// Whether ONE source-combination's quotient-level evaluation is non-empty: the conjunctive join
/// of the chosen candidates' relations, `Unconstrained` relations imposing nothing.
fn combination_is_live(
    relations: &[Vec<Relation>],
    choice: &[usize],
    max_intermediate: usize,
) -> Result<bool, SummaryDeclineReason> {
    let mut acc: Vec<Binding> = vec![Binding::new()];
    for (pattern, &candidate) in choice.iter().enumerate() {
        let rows = match &relations[pattern][candidate] {
            Relation::Unconstrained => continue,
            Relation::Rows(rows) => rows,
        };
        let mut next: Vec<Binding> = Vec::new();
        for left in &acc {
            for right in rows {
                if let Some(merged) = merge_bindings(left, right) {
                    if next.len() >= max_intermediate {
                        return Err(SummaryDeclineReason::JoinTooWide {
                            budget: max_intermediate,
                        });
                    }
                    next.push(merged);
                }
            }
        }
        if next.is_empty() {
            return Ok(false);
        }
        acc = next;
    }
    Ok(true)
}

/// Merge two compatible bindings (agreeing on every shared variable), or `None` if they conflict.
fn merge_bindings(left: &Binding, right: &Binding) -> Option<Binding> {
    let mut merged = left.clone();
    for (var, value) in right {
        match merged.get(var) {
            Some(existing) if existing != value => return None,
            Some(_) => {}
            None => {
                merged.insert(var.clone(), value.clone());
            }
        }
    }
    Some(merged)
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

    // ── Rule C2: quotient-summary provenance (sq-xkrt) ──────────────────────────────────

    use crate::quotient::SummaryTerm;

    const MEMBER_OF_FLAT: &str = "http://ex/memberOfFlat";
    const HAS_ADDRESS: &str = "http://ex/hasAddress";

    /// `?p memberOfFlat ?flat . ?flat hasAddress ?addr` — the join-on-`?flat` shape the
    /// disjoint-authority tests use.
    fn flat_join_bgp() -> Bgp {
        Bgp::new(vec![
            TriplePattern::new(var("p"), iri(MEMBER_OF_FLAT), var("flat")),
            TriplePattern::new(var("flat"), iri(HAS_ADDRESS), var("addr")),
        ])
    }

    /// A complete summary for a `memberOfFlat` holder whose flats live under `flat_authority`.
    fn membership_summary(id: &str, flat_authority: &str) -> SourceQuotientSummary {
        SourceQuotientSummary::builder(SourceId::new(id))
            .triple(
                SummaryTerm::iri("http://people.example/alice"),
                MEMBER_OF_FLAT,
                SummaryTerm::iri(format!("{}/1", flat_authority)),
            )
            .complete()
            .build()
    }

    /// A complete summary for a `hasAddress` holder keyed on flats under `flat_authority`.
    fn address_summary(id: &str, flat_authority: &str) -> SourceQuotientSummary {
        SourceQuotientSummary::builder(SourceId::new(id))
            .triple(
                SummaryTerm::iri(format!("{}/1", flat_authority)),
                HAS_ADDRESS,
                SummaryTerm::Literal,
            )
            .complete()
            .build()
    }

    /// **The load-bearing Rule-C2 win.** Two sources both hold the join predicate and both survive
    /// Phase 2, so Rule C1 alone prunes NOTHING — but their quotient summaries key the join
    /// variable on *disjoint authorities*, so one of the two source-combinations provably yields no
    /// answer. The provenance pass kills it; the same call WITHOUT summaries does not. That
    /// contrast is the whole point of the phase, so both halves are asserted here.
    #[test]
    fn disjoint_join_authorities_prune_a_combination_rule_c1_cannot_see() {
        let bgp = flat_join_bgp();
        let sources = vec![
            source("http://a/", &[MEMBER_OF_FLAT]), // 0 — flats under flats-a
            source("http://b/", &[HAS_ADDRESS]), // 1 — addresses for flats-b (disjoint)
            source("http://c/", &[HAS_ADDRESS]), // 2 — addresses for flats-a (joinable)
        ];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        // Precondition: pattern 1 really did keep BOTH address holders.
        assert_eq!(selected.per_pattern[1].candidates.len(), 2);

        // Baseline — Rule C1 alone: satisfiable, nothing pruned, provenance never attempted.
        let c1 = prune_source_combinations(&bgp, &sources, &selected).unwrap();
        assert!(c1.bgp_satisfiable);
        assert!(c1.pruned.is_empty());
        assert_eq!(c1.summary_prune, SummaryPruneOutcome::NotAttempted);
        assert!(c1.live_combinations.is_empty());

        // Rule C2 — with the summaries, the (A, B) combination is proved dead.
        let summaries = vec![
            membership_summary("http://a/", "http://flats-a.example"),
            address_summary("http://b/", "http://flats-b.example"),
            address_summary("http://c/", "http://flats-a.example"),
        ];
        let out = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap();

        assert_eq!(
            out.summary_prune,
            SummaryPruneOutcome::Applied {
                combinations_considered: 2,
                combinations_live: 1,
            }
        );
        // Exactly one live combination: pattern 0 → source 0 (A), pattern 1 → source 2 (C).
        assert_eq!(
            out.live_combinations,
            vec![SourceCombination {
                assignment: vec![0, 2]
            }]
        );
        // B is combination-dead for pattern 1, with the provenance reason.
        assert_eq!(out.pruned.len(), 1);
        assert_eq!(out.pruned[0].pattern, 1);
        assert_eq!(out.pruned[0].source, 1);
        assert_eq!(out.pruned[0].source_id, SourceId::new("http://b/"));
        assert_eq!(out.pruned[0].reason, CombinationPruneReason::EmptyProvenance);
        // An answer is still possible overall, and the selection is carried through unchanged.
        assert!(out.bgp_satisfiable);
        assert_eq!(out.per_pattern, selected.per_pattern);
    }

    /// When EVERY combination's quotient evaluation is empty, the BGP is proved answer-free even
    /// though no conjunct is empty — `bgp_satisfiable` flips to false with NO empty-pattern
    /// witness, and the seam can skip the MPC path entirely.
    #[test]
    fn every_combination_dead_makes_the_bgp_unsatisfiable_without_an_empty_conjunct() {
        let bgp = flat_join_bgp();
        let sources = vec![
            source("http://a/", &[MEMBER_OF_FLAT]),
            source("http://b/", &[HAS_ADDRESS]),
        ];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        let summaries = vec![
            membership_summary("http://a/", "http://flats-a.example"),
            address_summary("http://b/", "http://flats-b.example"),
        ];
        let out = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap();

        assert!(!out.bgp_satisfiable);
        assert!(out.is_bgp_dead());
        assert!(
            out.empty_patterns.is_empty(),
            "no conjunct is empty — the witness is the provenance, not a pattern"
        );
        assert!(out.live_combinations.is_empty());
        assert_eq!(
            out.summary_prune,
            SummaryPruneOutcome::Applied {
                combinations_considered: 1,
                combinations_live: 0,
            }
        );
        // Both surviving Phase-2 candidates are recorded dead, ascending by (pattern, source).
        assert_eq!(out.pruned.len(), 2);
        assert_eq!((out.pruned[0].pattern, out.pruned[0].source), (0, 0));
        assert_eq!((out.pruned[1].pattern, out.pruned[1].source), (1, 1));
        assert!(out.surviving_combination_patterns().is_empty());
    }

    /// Recall-safety layer 1: a source that published NO summary constrains nothing, so it is
    /// never the reason a combination dies. Same disjoint-authority setup as the headline test,
    /// but B publishes nothing — B must survive.
    #[test]
    fn a_source_without_a_summary_is_never_pruned() {
        let bgp = flat_join_bgp();
        let sources = vec![
            source("http://a/", &[MEMBER_OF_FLAT]),
            source("http://b/", &[HAS_ADDRESS]),
        ];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        // Only A publishes a summary; B is unconstrained.
        let summaries = vec![membership_summary("http://a/", "http://flats-a.example")];
        let out = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap();

        assert!(out.bgp_satisfiable);
        assert!(
            out.pruned.is_empty(),
            "a source with no summary must never be pruned"
        );
        assert_eq!(
            out.live_combinations,
            vec![SourceCombination {
                assignment: vec![0, 1]
            }]
        );
    }

    /// Recall-safety layer 1 (second half): a summary the source did NOT declare complete is not
    /// an over-approximation, so nothing can be proved from it — the pass reports `NotAttempted`
    /// and prunes nothing, even though the declared triples would otherwise kill the combination.
    #[test]
    fn an_incomplete_summary_proves_nothing_and_prunes_nothing() {
        let bgp = flat_join_bgp();
        let sources = vec![
            source("http://a/", &[MEMBER_OF_FLAT]),
            source("http://b/", &[HAS_ADDRESS]),
        ];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        // Byte-identical to the "all dead" fixture EXCEPT the missing `.complete()` calls.
        let summaries = vec![
            SourceQuotientSummary::builder(SourceId::new("http://a/"))
                .triple(
                    SummaryTerm::iri("http://people.example/alice"),
                    MEMBER_OF_FLAT,
                    SummaryTerm::iri("http://flats-a.example/1"),
                )
                .build(),
            SourceQuotientSummary::builder(SourceId::new("http://b/"))
                .triple(
                    SummaryTerm::iri("http://flats-b.example/1"),
                    HAS_ADDRESS,
                    SummaryTerm::Literal,
                )
                .build(),
        ];
        let out = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap();

        assert_eq!(out.summary_prune, SummaryPruneOutcome::NotAttempted);
        assert!(out.bgp_satisfiable);
        assert!(out.pruned.is_empty());
        assert!(out.live_combinations.is_empty());
    }

    /// Positive control (the four-flatmates shape, summarised): every source's summary agrees on
    /// the join authority, so every combination is live and NOTHING is pruned. A regression that
    /// over-pruned would fire here.
    #[test]
    fn four_flatmates_with_agreeing_summaries_prunes_nothing() {
        let bgp = flat_join_bgp();
        let sources = vec![
            source("http://a/", &[MEMBER_OF_FLAT]),
            source("http://a2/", &[MEMBER_OF_FLAT]),
            source("http://b/", &[HAS_ADDRESS]),
            source("http://c/", &[HAS_ADDRESS]),
        ];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        let summaries = vec![
            membership_summary("http://a/", "http://flats.example"),
            membership_summary("http://a2/", "http://flats.example"),
            address_summary("http://b/", "http://flats.example"),
            address_summary("http://c/", "http://flats.example"),
        ];
        let out = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap();

        assert!(out.bgp_satisfiable);
        assert!(out.pruned.is_empty());
        assert_eq!(
            out.summary_prune,
            SummaryPruneOutcome::Applied {
                combinations_considered: 4,
                combinations_live: 4,
            }
        );
        // All 2×2 combinations survive, enumerated deterministically.
        assert_eq!(
            out.live_combinations,
            vec![
                SourceCombination {
                    assignment: vec![0, 2]
                },
                SourceCombination {
                    assignment: vec![0, 3]
                },
                SourceCombination {
                    assignment: vec![1, 2]
                },
                SourceCombination {
                    assignment: vec![1, 3]
                },
            ]
        );
    }

    /// A variable repeated WITHIN one pattern (`?x knows ?x`) must bind consistently: a source
    /// whose summary only records cross-authority `knows` cannot match it, so the combination is
    /// dead. A regression that let the second occurrence overwrite the first would call it live.
    #[test]
    fn a_variable_repeated_in_one_pattern_must_bind_consistently() {
        const KNOWS: &str = "http://ex/knows";
        let bgp = Bgp::new(vec![TriplePattern::new(var("x"), iri(KNOWS), var("x"))]);
        let sources = vec![source("http://a/", &[KNOWS])];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();

        // Cross-authority `knows` only ⇒ `?x` cannot take one class in both positions ⇒ dead.
        let cross = vec![SourceQuotientSummary::builder(SourceId::new("http://a/"))
            .triple(
                SummaryTerm::iri("http://l.example/1"),
                KNOWS,
                SummaryTerm::iri("http://r.example/1"),
            )
            .complete()
            .build()];
        let dead = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &cross,
            CombinationBudget::default(),
        )
        .unwrap();
        assert!(!dead.bgp_satisfiable);
        assert_eq!(dead.pruned.len(), 1);

        // Add a same-authority `knows` and the combination becomes live again.
        let same = vec![SourceQuotientSummary::builder(SourceId::new("http://a/"))
            .triple(
                SummaryTerm::iri("http://l.example/1"),
                KNOWS,
                SummaryTerm::iri("http://r.example/1"),
            )
            .triple(
                SummaryTerm::iri("http://l.example/1"),
                KNOWS,
                SummaryTerm::iri("http://l.example/2"),
            )
            .complete()
            .build()];
        let live = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &same,
            CombinationBudget::default(),
        )
        .unwrap();
        assert!(live.bgp_satisfiable);
        assert!(live.pruned.is_empty());
    }

    /// A bound LITERAL in the pattern matches the single literal class (an over-approximation —
    /// summaries never record a literal's value), while a bound IRI in the same position does not.
    #[test]
    fn a_bound_literal_matches_the_literal_class_and_an_iri_does_not() {
        const NAME_P: &str = "http://ex/name";
        // INCOMPLETE authorities, so the upstream authority prune does not empty the bound-IRI
        // pattern before the provenance pass gets to it (we are testing OUR matcher, not that one).
        let sources = vec![source_incomplete("http://a/", &[NAME_P])];
        let summaries = vec![SourceQuotientSummary::builder(SourceId::new("http://a/"))
            .triple(
                SummaryTerm::iri("http://people.example/alice"),
                NAME_P,
                SummaryTerm::Literal,
            )
            .complete()
            .build()];

        // Object is a bound literal ⇒ matches the summarised literal ⇒ live.
        let lit_bgp = Bgp::new(vec![TriplePattern::new(
            var("s"),
            iri(NAME_P),
            Term::Literal("Alice".to_string()),
        )]);
        let lit_sel = select_private_sources(&lit_bgp, &sources, &[]).unwrap();
        let lit = prune_source_combinations_with_summaries(
            &lit_bgp,
            &sources,
            &lit_sel,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap();
        assert!(lit.bgp_satisfiable, "a bound literal matches the literal class");

        // Object is a bound IRI ⇒ the summary holds only a literal there ⇒ dead.
        let iri_bgp = Bgp::new(vec![TriplePattern::new(
            var("s"),
            iri(NAME_P),
            iri("http://people.example/alice"),
        )]);
        let iri_sel = select_private_sources(&iri_bgp, &sources, &[]).unwrap();
        assert!(!iri_sel.per_pattern[0].is_empty(), "Phase 2 keeps the source");
        let bound = prune_source_combinations_with_summaries(
            &iri_bgp,
            &sources,
            &iri_sel,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap();
        assert!(!bound.bgp_satisfiable, "an IRI cannot match the literal class");
    }

    /// Decline 1 — a variable used in BOTH a predicate position and a subject position has no
    /// well-defined quotient image (predicates stay concrete, terms are quotiented), so joining on
    /// it could prune unsoundly. The pass declines and prunes nothing.
    #[test]
    fn a_predicate_term_variable_clash_declines_and_prunes_nothing() {
        // `?s ?p ?o . ?p name ?n` — `?p` is a predicate in pattern 0 and a subject in pattern 1.
        const NAME_P: &str = "http://ex/name";
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), var("p"), var("o")),
            TriplePattern::new(var("p"), iri(NAME_P), var("n")),
        ]);
        let sources = vec![source("http://a/", &[NAME_P])];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        assert!(!selected.per_pattern[0].is_empty() && !selected.per_pattern[1].is_empty());

        let summaries = vec![SourceQuotientSummary::builder(SourceId::new("http://a/"))
            .triple(
                SummaryTerm::iri("http://people.example/alice"),
                NAME_P,
                SummaryTerm::Literal,
            )
            .complete()
            .build()];
        let out = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap();

        assert_eq!(
            out.summary_prune,
            SummaryPruneOutcome::Declined(SummaryDeclineReason::PredicateVariableDomainClash {
                var: "p".to_string()
            })
        );
        assert!(out.pruned.is_empty(), "declining must prune nothing");
        assert!(out.bgp_satisfiable);
        assert!(out.live_combinations.is_empty());
    }

    /// Decline 2 — an over-budget combination space declines rather than truncating the search
    /// (a truncated search cannot PROVE a combination dead, so truncating would be recall-unsafe).
    #[test]
    fn an_over_budget_combination_space_declines_and_prunes_nothing() {
        let bgp = flat_join_bgp();
        let sources = vec![
            source("http://a/", &[MEMBER_OF_FLAT]),
            source("http://b/", &[HAS_ADDRESS]),
            source("http://c/", &[HAS_ADDRESS]),
        ];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        let summaries = vec![
            membership_summary("http://a/", "http://flats-a.example"),
            address_summary("http://b/", "http://flats-b.example"),
            address_summary("http://c/", "http://flats-a.example"),
        ];
        // 1 × 2 = 2 combinations, budget 1.
        let out = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &summaries,
            CombinationBudget::new(1, 4096),
        )
        .unwrap();

        assert_eq!(
            out.summary_prune,
            SummaryPruneOutcome::Declined(SummaryDeclineReason::TooManyCombinations {
                combinations: 2,
                budget: 1,
            })
        );
        assert!(out.pruned.is_empty());
        assert!(out.live_combinations.is_empty());
    }

    /// Decline 3 — an over-wide intermediate join declines too. The same inputs under the default
    /// budget DO prune, so this pins the budget as the cause rather than the data.
    #[test]
    fn an_over_wide_join_declines_and_prunes_nothing() {
        let bgp = flat_join_bgp();
        let sources = vec![
            source("http://a/", &[MEMBER_OF_FLAT]),
            source("http://b/", &[HAS_ADDRESS]),
        ];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        // A's summary admits TWO distinct `?flat` classes; B's addresses reach neither.
        let summaries = vec![
            SourceQuotientSummary::builder(SourceId::new("http://a/"))
                .triple(
                    SummaryTerm::iri("http://people.example/alice"),
                    MEMBER_OF_FLAT,
                    SummaryTerm::iri("http://flats-a.example/1"),
                )
                .triple(
                    SummaryTerm::iri("http://people.example/alice"),
                    MEMBER_OF_FLAT,
                    SummaryTerm::iri("http://flats-x.example/1"),
                )
                .complete()
                .build(),
            address_summary("http://b/", "http://flats-b.example"),
        ];

        // Budget of 1 intermediate binding: pattern 0 alone yields 2 ⇒ decline.
        let declined = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &summaries,
            CombinationBudget::new(4096, 1),
        )
        .unwrap();
        assert_eq!(
            declined.summary_prune,
            SummaryPruneOutcome::Declined(SummaryDeclineReason::JoinTooWide { budget: 1 })
        );
        assert!(declined.pruned.is_empty());

        // Same data under the default budget: the combination really is dead.
        let applied = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap();
        assert!(!applied.bgp_satisfiable);
        assert_eq!(applied.pruned.len(), 2);
    }

    /// Rule C1 fires first: when a conjunct is already empty the whole combination space is
    /// collapsed, and the provenance pass reports that it had nothing left to do (rather than
    /// silently claiming an `Applied` verdict over a dead BGP).
    #[test]
    fn rule_c1_short_circuits_the_provenance_pass() {
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("p"), iri(MEMBER_OF_FLAT), var("flat")),
            TriplePattern::new(var("flat"), iri(ABSENT), var("z")),
        ]);
        let sources = vec![source("http://a/", &[MEMBER_OF_FLAT])];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        let summaries = vec![membership_summary("http://a/", "http://flats-a.example")];
        let out = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap();

        assert_eq!(out.summary_prune, SummaryPruneOutcome::BgpAlreadyDead);
        assert!(!out.bgp_satisfiable);
        assert_eq!(out.empty_patterns, vec![1]);
        // The Rule-C1 audit trail is intact (the unsatisfiable-conjunct reason, not provenance).
        assert_eq!(out.pruned.len(), 1);
        assert_eq!(
            out.pruned[0].reason,
            CombinationPruneReason::UnsatisfiableBgp { witness: 1 }
        );
    }

    /// Two summaries for one source id is an ambiguous declaration: fail-closed, never guess.
    #[test]
    fn duplicate_summary_for_one_source_is_a_descriptor_mismatch() {
        let bgp = flat_join_bgp();
        let sources = vec![
            source("http://a/", &[MEMBER_OF_FLAT]),
            source("http://b/", &[HAS_ADDRESS]),
        ];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        let summaries = vec![
            membership_summary("http://a/", "http://flats-a.example"),
            membership_summary("http://a/", "http://flats-b.example"),
        ];
        let err = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            SeamError::DescriptorMismatch {
                phase: SeamPhase::SourceCombination,
                ..
            }
        ));
        assert!(format!("{}", err).contains("http://a/"));
    }

    /// A selection naming a pattern index outside the BGP is fail-closed too — the provenance pass
    /// refuses to guess which pattern a candidate set belongs to.
    #[test]
    fn selection_pattern_index_outside_the_bgp_is_a_descriptor_mismatch() {
        let bgp = flat_join_bgp();
        let sources = vec![
            source("http://a/", &[MEMBER_OF_FLAT]),
            source("http://b/", &[HAS_ADDRESS]),
        ];
        // Same LENGTH as the BGP (so the Rule-C1 shape check passes) but pattern 7 does not exist.
        let mismatched = SelectedPrivateSources {
            per_pattern: vec![
                PrivatePatternSources {
                    pattern: 0,
                    candidates: vec![PrivateCandidate {
                        source: 0,
                        source_id: SourceId::new("http://a/"),
                        estimated_cardinality: 1.0,
                    }],
                },
                PrivatePatternSources {
                    pattern: 7,
                    candidates: vec![PrivateCandidate {
                        source: 1,
                        source_id: SourceId::new("http://b/"),
                        estimated_cardinality: 1.0,
                    }],
                },
            ],
            pruned: Vec::new(),
        };
        let summaries = vec![membership_summary("http://a/", "http://flats-a.example")];
        let err = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &mismatched,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            SeamError::DescriptorMismatch {
                phase: SeamPhase::SourceCombination,
                ..
            }
        ));
        assert!(format!("{}", err).contains("outside the BGP"));
    }

    // A hand-built per-pattern entry: `pattern`, with one candidate per `(source index, id)`.
    // Used by the alignment regressions, which need selections the Phase-2 pass would never
    // produce but a caller can build (both types are publicly constructible). [SONNET-4.6]
    fn pattern_sources(pattern: usize, candidates: &[(usize, &str)]) -> PrivatePatternSources {
        PrivatePatternSources {
            pattern,
            candidates: candidates
                .iter()
                .map(|(source, id)| PrivateCandidate {
                    source: *source,
                    source_id: SourceId::new(*id),
                    estimated_cardinality: 1.0,
                })
                .collect(),
        }
    }

    /// A selection that names the SAME pattern twice has the BGP's length, so the count check
    /// alone waves it through — and it would then evaluate pattern 0 twice and silently omit
    /// pattern 1. Fail-closed on BOTH entry points. [SONNET-4.6]
    #[test]
    fn duplicate_pattern_index_is_a_descriptor_mismatch() {
        let bgp = flat_join_bgp();
        let sources = vec![
            source("http://a/", &[MEMBER_OF_FLAT]),
            source("http://b/", &[HAS_ADDRESS]),
        ];
        let duplicated = SelectedPrivateSources {
            per_pattern: vec![
                pattern_sources(0, &[(0, "http://a/")]),
                pattern_sources(0, &[(0, "http://a/")]),
            ],
            pruned: Vec::new(),
        };

        let err = prune_source_combinations(&bgp, &sources, &duplicated).unwrap_err();
        assert!(matches!(
            err,
            SeamError::DescriptorMismatch {
                phase: SeamPhase::SourceCombination,
                ..
            }
        ));
        assert!(format!("{}", err).contains("BGP pattern order"));

        let summaries = vec![membership_summary("http://a/", "http://flats-a.example")];
        let err = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &duplicated,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            SeamError::DescriptorMismatch {
                phase: SeamPhase::SourceCombination,
                ..
            }
        ));
    }

    /// A REORDERED selection is refused rather than silently producing a
    /// `SourceCombination::assignment` in selection-vector order — the field is documented as BGP
    /// pattern order, and the in-order control below pins that it really is. [SONNET-4.6]
    #[test]
    fn reordered_pattern_indices_are_refused_and_assignments_stay_in_bgp_order() {
        let bgp = flat_join_bgp();
        let sources = vec![
            source("http://a/", &[MEMBER_OF_FLAT]), // 0 — pattern 0's holder
            source("http://b/", &[HAS_ADDRESS]),    // 1 — pattern 1's holder
        ];
        let summaries = vec![
            membership_summary("http://a/", "http://flats-a.example"),
            address_summary("http://b/", "http://flats-a.example"), // joinable
        ];

        // Control: the genuine (in-order) selection is accepted, and the one live combination
        // reads pattern 0 → source 0, pattern 1 → source 1 — BGP-pattern order.
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        assert_eq!(selected.per_pattern[0].pattern, 0);
        let out = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap();
        assert_eq!(
            out.live_combinations,
            vec![SourceCombination {
                assignment: vec![0, 1]
            }]
        );

        // The same two entries, swapped: refused, not re-attributed.
        let reordered = SelectedPrivateSources {
            per_pattern: vec![
                pattern_sources(1, &[(1, "http://b/")]),
                pattern_sources(0, &[(0, "http://a/")]),
            ],
            pruned: Vec::new(),
        };
        let err = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &reordered,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            SeamError::DescriptorMismatch {
                phase: SeamPhase::SourceCombination,
                ..
            }
        ));
        assert!(format!("{}", err).contains("BGP pattern order"));
    }

    /// A candidate whose source index does not address the descriptor slice — or whose carried id
    /// disagrees with the descriptor at that index — is fail-closed: `assignment` hands those
    /// indices downstream and Rule C2 keys its summary lookup on the id. [SONNET-4.6]
    #[test]
    fn candidate_source_index_or_id_disagreeing_with_the_descriptors_is_a_descriptor_mismatch() {
        let bgp = flat_join_bgp();
        let sources = vec![
            source("http://a/", &[MEMBER_OF_FLAT]),
            source("http://b/", &[HAS_ADDRESS]),
        ];

        // Source index 9 is outside the two-descriptor slice.
        let out_of_range = SelectedPrivateSources {
            per_pattern: vec![
                pattern_sources(0, &[(0, "http://a/")]),
                pattern_sources(1, &[(9, "http://b/")]),
            ],
            pruned: Vec::new(),
        };
        let err = prune_source_combinations(&bgp, &sources, &out_of_range).unwrap_err();
        assert!(format!("{}", err).contains("outside the descriptor slice"));

        // Index 1 exists, but it is source B — not the id the candidate claims.
        let wrong_id = SelectedPrivateSources {
            per_pattern: vec![
                pattern_sources(0, &[(0, "http://a/")]),
                pattern_sources(1, &[(1, "http://a/")]),
            ],
            pruned: Vec::new(),
        };
        let err = prune_source_combinations(&bgp, &sources, &wrong_id).unwrap_err();
        assert!(matches!(
            err,
            SeamError::DescriptorMismatch {
                phase: SeamPhase::SourceCombination,
                ..
            }
        ));
        assert!(format!("{}", err).contains("does not match the descriptor"));
    }

    /// The combination count is `checked_mul`ed, so a product that OVERFLOWS `usize` DECLINES even
    /// under a `usize::MAX` budget. A saturating count would compare `usize::MAX > usize::MAX ==
    /// false`, wave the overflow through, and let the enumeration allocate unbounded. Declining
    /// prunes nothing (recall-safe). [SONNET-4.6]
    #[test]
    fn overflowing_combination_count_declines_under_an_unbounded_budget() {
        // 64 patterns, each held by BOTH sources ⇒ 2^64 combinations, one past `usize::MAX`.
        let preds: Vec<String> = (0..64).map(|i| format!("http://ex/p{}", i)).collect();
        let pred_refs: Vec<&str> = preds.iter().map(String::as_str).collect();
        let bgp = Bgp::new(
            preds
                .iter()
                .map(|p| TriplePattern::new(var("s"), iri(p), var("o")))
                .collect(),
        );
        let sources = vec![
            source("http://a/", &pred_refs),
            source("http://b/", &pred_refs),
        ];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        assert!(
            selected.per_pattern.iter().all(|p| p.candidates.len() == 2),
            "precondition: every pattern keeps both holders"
        );

        let summaries = vec![membership_summary("http://a/", "http://flats-a.example")];
        let out = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &summaries,
            CombinationBudget::new(usize::MAX, usize::MAX),
        )
        .unwrap();

        assert_eq!(
            out.summary_prune,
            SummaryPruneOutcome::Declined(SummaryDeclineReason::TooManyCombinations {
                combinations: usize::MAX,
                budget: usize::MAX,
            })
        );
        assert!(out.pruned.is_empty(), "a decline prunes nothing");
        assert!(out.live_combinations.is_empty());
        assert!(out.bgp_satisfiable);
    }

    /// The provenance pass is deterministic: identical inputs ⇒ identical output, twice
    /// (including the live-combination enumeration order).
    #[test]
    fn provenance_prune_is_deterministic() {
        let bgp = flat_join_bgp();
        let sources = vec![
            source("http://a/", &[MEMBER_OF_FLAT]),
            source("http://b/", &[HAS_ADDRESS]),
            source("http://c/", &[HAS_ADDRESS]),
        ];
        let selected = select_private_sources(&bgp, &sources, &[]).unwrap();
        let summaries = vec![
            membership_summary("http://a/", "http://flats-a.example"),
            address_summary("http://b/", "http://flats-b.example"),
            address_summary("http://c/", "http://flats-a.example"),
        ];
        let a = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap();
        let b = prune_source_combinations_with_summaries(
            &bgp,
            &sources,
            &selected,
            &summaries,
            CombinationBudget::default(),
        )
        .unwrap();
        assert_eq!(a, b);
    }

    /// The audit-trail labels for the new reason and each decline reason name their cause (a
    /// regression that blanked one would lose the diagnostic a relying party reads).
    #[test]
    fn provenance_labels_name_their_cause() {
        let reason = CombinationPruneReason::EmptyProvenance.label();
        assert!(reason.contains("provenance"), "names the cause: {reason}");

        let clash = SummaryDeclineReason::PredicateVariableDomainClash {
            var: "p".to_string(),
        }
        .label();
        assert!(clash.contains("?p"), "names the variable: {clash}");
        let budget = SummaryDeclineReason::TooManyCombinations {
            combinations: 9,
            budget: 4,
        }
        .label();
        assert!(budget.contains('9') && budget.contains('4'), "{budget}");
        let wide = SummaryDeclineReason::JoinTooWide { budget: 3 }.label();
        assert!(wide.contains('3'), "{wide}");
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
