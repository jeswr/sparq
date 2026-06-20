//! # sparq-replan-policy — the shared adaptive-re-plan POLICY core
//!
//! [OPUS-4.8] sq-p6p6. The **divergence trigger** + **hysteresis adoption** rule that
//! both of sparq's adaptive re-planners use, factored into one pure, dependency-free
//! place so the two paths **cannot drift apart** on the load-bearing decision logic:
//!
//! * the **federated** planner — `sparq-fedplan`'s `adaptive-replan` feature
//!   (`AdaptiveExecutor`), which plans over already-fetched source DESCRIPTORS, and
//! * the **local** evaluator — `sparq-engine`'s `adaptive-replan-local` feature, which
//!   materialises every intermediate join so true per-stage cardinality is known for free.
//!
//! The two executors operate on completely different intermediate representations, so the
//! *executor* is not shared — only this tiny policy core is. Everything here is plain `f64`
//! arithmetic: no RDF terms, no graph, no I/O, no `unsafe`, no dependencies. The functions
//! are `#[inline]`, so a consumer pays nothing for the abstraction over an in-crate copy —
//! the only thing it buys is that the two re-planners cannot diverge on the rule.
//!
//! ## The two knobs (verbatim from the federated policy, sq-7s4z)
//!
//! * [`ReplanPolicy::divergence_factor`] `k` (> 1) — a re-plan is *considered* only when an
//!   observed cardinality `o` diverges from its estimate `e` by more than `k` in either
//!   direction (`o > k·e` **or** `e > k·o`). Default `4.0` (a 4× miss). See [`diverges`].
//! * [`ReplanPolicy::improvement_margin`] `m` (∈ `[0, 1)`) — the re-planned alternative is
//!   *adopted* only when its estimated cost beats the current plan's by more than `m`
//!   (`new < cur·(1 − m)`). Default `0.1` (a ≥ 10% win). The hysteresis band that stops
//!   noisy-but-stable statistics from thrashing the plan. See [`adopt`].
//! * [`ReplanPolicy::max_replans`] — hard cap on re-plans per execution (belt-and-braces
//!   anti-thrash + bounded planning overhead). Default `8`.
//!
//! Divergence *triggers*, the margin *gates adoption* — together they give the
//! divergence-then-hysteresis behaviour the soundness argument (in each consumer) rests on.

#![forbid(unsafe_code)]

/// Policy controlling when a mid-execution re-plan fires and whether its result is adopted.
///
/// Shared by both adaptive re-planners (federated + local) so the trigger + adoption rule
/// cannot diverge. The fields mirror the original federated `ReplanPolicy` (sq-7s4z); the
/// federated crate keeps its extra latency knobs on its own type and delegates only the
/// cardinality trigger + hysteresis here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReplanPolicy {
    /// Divergence factor `k` (> 1). A re-plan is *considered* when an observed cardinality
    /// `o` for a not-yet-executed operator diverges from its estimate `e` by more than this
    /// factor in either direction: `o > k·e` **or** `e > k·o`. Default `4.0` (a 4× miss).
    pub divergence_factor: f64,
    /// Minimum relative cost improvement (in `[0, 1)`) the re-planned alternative must show
    /// over the current plan's estimated remaining cost before it is adopted — the
    /// hysteresis band that prevents thrashing. `0.1` means "only switch if it is ≥ 10%
    /// cheaper". Default `0.1`.
    pub improvement_margin: f64,
    /// Hard cap on how many re-plans a single execution may perform (belt-and-braces
    /// anti-thrash + bounded planning overhead). Default `8`.
    pub max_replans: usize,
}

impl Default for ReplanPolicy {
    fn default() -> Self {
        ReplanPolicy { divergence_factor: 4.0, improvement_margin: 0.1, max_replans: 8 }
    }
}

impl ReplanPolicy {
    /// Whether observed cardinality `observed` diverges from estimate `estimate` by more than
    /// this policy's [`Self::divergence_factor`] (in either direction). Thin method form of
    /// the free function [`diverges`].
    #[inline]
    pub fn diverges(&self, estimate: f64, observed: f64) -> bool {
        diverges(estimate, observed, self.divergence_factor)
    }

    /// Whether a re-planned alternative of estimated cost `new_cost` should be adopted over a
    /// current plan of estimated cost `cur_cost`, under this policy's
    /// [`Self::improvement_margin`]. Thin method form of the free function [`adopt`].
    #[inline]
    pub fn adopt(&self, cur_cost: f64, new_cost: f64) -> bool {
        adopt(cur_cost, new_cost, self.improvement_margin)
    }
}

/// Whether observed cardinality `observed` diverges from estimate `estimate` by more than
/// the factor `k` (in either direction): `o > k·e` **or** `e > k·o`.
///
/// Ported verbatim from the federated `adaptive::diverges` (sq-7s4z): `k` is floored at
/// `1.0`, the estimate at `1.0` and the observation at `0.0`, with the symmetric branch
/// flooring the observation at `1.0` so a genuinely-zero observation against a large estimate
/// still counts as divergent.
#[inline]
pub fn diverges(estimate: f64, observed: f64, divergence_factor: f64) -> bool {
    let k = divergence_factor.max(1.0);
    let e = estimate.max(1.0);
    let o = observed.max(0.0);
    o > k * e || e > k * o.max(1.0)
}

/// Whether a re-planned alternative of estimated cost `new_cost` should be ADOPTED over the
/// current plan of estimated cost `cur_cost`, given the hysteresis `improvement_margin`.
///
/// Ported verbatim from the federated adoption rule (sq-7s4z): the new plan wins only when it
/// is cheaper than `(1 − margin)·cur_cost`, with `margin` clamped to `[0, 0.999]`. A tie or a
/// sub-margin win is rejected (anti-thrash). The *caller* additionally requires the new plan
/// to differ from the current one — adopting an identical re-order is a no-op there.
#[inline]
pub fn adopt(cur_cost: f64, new_cost: f64, improvement_margin: f64) -> bool {
    let margin = improvement_margin.clamp(0.0, 0.999);
    new_cost < cur_cost * (1.0 - margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diverges_fires_past_factor_both_directions() {
        // 4x either way fires; below does not.
        assert!(diverges(20.0, 5000.0, 4.0), "o much-greater-than k*e fires");
        assert!(diverges(1000.0, 5.0, 4.0), "e much-greater-than k*o fires");
        assert!(!diverges(1000.0, 2000.0, 4.0), "2x under 4x does NOT fire");
        assert!(!diverges(1000.0, 1000.0, 4.0), "exact estimate does NOT fire");
    }

    #[test]
    fn diverges_floors_match_federated() {
        // Estimate floored to 1.0, observation floored to 0.0; the symmetric branch floors o
        // to 1.0 — so a zero observation against a large estimate is divergent.
        assert!(diverges(100.0, 0.0, 4.0), "zero observed vs large estimate diverges");
        // k floored at 1.0: a degenerate factor < 1 behaves as 1.0 (strict inequality => a
        // genuine 2x miss still diverges, an exact match does not).
        assert!(diverges(10.0, 20.0, 0.5), "k<1 floored to 1 => any strict miss diverges");
        assert!(!diverges(10.0, 10.0, 0.5), "exact match never diverges even with k<1");
    }

    #[test]
    fn adopt_respects_margin() {
        // Default 10% margin: need a >10% win.
        assert!(adopt(100.0, 80.0, 0.1), "20% cheaper is adopted");
        assert!(!adopt(100.0, 95.0, 0.1), "5% cheaper is rejected (within band)");
        assert!(!adopt(100.0, 100.0, 0.1), "a tie is rejected");
        // A near-1.0 margin rejects almost every win (the anti-thrash extreme): the bar is
        // new < 100*(1 - 0.999) = 0.0999, so even a 99% cheaper alternative is rejected, but a
        // sufficiently tiny cost clears it.
        assert!(!adopt(100.0, 1.0, 0.999), "1.0 vs the 0.0999 bar is rejected");
        assert!(adopt(100.0, 0.05, 0.999), "0.05 clears the 0.0999 bar");
    }

    #[test]
    fn policy_methods_match_free_functions() {
        let p = ReplanPolicy::default();
        assert_eq!(p.diverges(20.0, 5000.0), diverges(20.0, 5000.0, p.divergence_factor));
        assert_eq!(p.adopt(100.0, 80.0), adopt(100.0, 80.0, p.improvement_margin));
        assert_eq!(p.divergence_factor, 4.0);
        assert_eq!(p.improvement_margin, 0.1);
        assert_eq!(p.max_replans, 8);
    }
}
