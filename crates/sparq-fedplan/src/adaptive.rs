//! **Live adaptive re-planning** — mid-execution plan switching when the *observed*
//! runtime statistics diverge from the cost-model estimates the static plan was built on.
//!
//! The static [`crate::plan_bgp`] commits to one join order + per-join algorithm up front,
//! from descriptor-derived **estimates**. In a real federation those estimates are often
//! wrong: a source returns far more (or fewer) bindings than its VoID stats predicted, or
//! a source is slow / unavailable. ANAPSID's harder half is to *react* — re-plan the part
//! of the query that has **not yet executed** with the corrected statistics, and switch to
//! the better plan if it is enough of a win to be worth the switch.
//!
//! This module is the opt-in (`adaptive-replan` feature) realisation of that, scoped to a
//! **sound boundary** (see below).
//!
//! ## What it does
//!
//! 1. **Capture actual runtime statistics** ([`RuntimeStats`]): as the executor finishes
//!    each leaf evaluation / join stage, it records the *observed* cardinality of that
//!    operator and the *observed* per-source latency. These are real counts, not estimates.
//! 2. **Re-plan trigger** ([`ReplanPolicy`]): after each stage boundary, compare observed
//!    against estimated. If a not-yet-executed pattern's leaf cardinality was estimated at
//!    `e` but a *correlated* source has now revealed it is really `≈ o` with `o > k·e`
//!    (or `o < e/k`) — i.e. the estimate was off by more than the policy's
//!    [`ReplanPolicy::divergence_factor`] `k` — the planner is re-invoked on the
//!    **remaining** patterns with the corrected leaf cardinalities substituted in.
//! 3. **Hysteresis / anti-thrash** ([`ReplanPolicy::improvement_margin`]): the re-planned
//!    suffix is adopted **only** if its estimated remaining cost beats the current
//!    suffix's by more than a margin (default 10%). A re-plan that is merely a tie, or a
//!    tiny win, is rejected — so stable-but-noisy stats do not cause the plan to flap.
//!
//! ## Per-source latency weighting (sq-b51o) — a HEURISTIC, not optimal
//!
//! Cardinality is not the only thing the static estimates get wrong: a source can be
//! *slow* (contended, far, rate-limited) even when its row counts are exactly as predicted.
//! [`RuntimeStats`] captures the *observed* per-source latency; sq-b51o folds it into the
//! cost model so the re-planner prefers orderings that defer sub-queries against an
//! observably slow source.
//!
//! ## Latency EWMA smoothing (sq-b51o follow-up) — a HEURISTIC α, not optimal
//!
//! The cost model is fed not the **single last** latency observation but a per-source
//! **exponentially-weighted moving average** (EWMA). Each call to
//! [`RuntimeStats::record_source_latency`] folds the new observation into the running
//! average for that source:
//!
//! > `ewma_new = α · observed + (1 − α) · ewma_prev`  (first observation seeds `ewma = observed`).
//!
//! The smoothing factor α ∈ (0, 1] is [`RuntimeStats::latency_alpha`], default `0.3` — a
//! **hand-picked heuristic, not a derived optimum**: it weights the latest sample at 30% and
//! the accumulated history at 70%, so a *single* transient spike moves the average only a
//! fraction of the way and does **not**, on its own, clear the re-plan trigger — but a
//! **sustained** shift (several consecutive high samples) does, as the average converges
//! toward the new level. This is the cleaner anti-thrash discipline the follow-up replaces
//! the bare single-sample-plus-clamp with: the EWMA itself absorbs noise; the absolute
//! `latency_floor`/`latency_cap` clamp on the resulting cost factor is kept as a **final
//! guard** (an outlier that *does* survive the average still cannot dominate the order).
//! Higher α tracks faster but is twitchier; lower α is calmer but laggier — 0.3 is a middle
//! bias, picked by hand, not tuned against a workload.
//!
//! The EWMA (not the raw last value) feeds **both** the latency cost factor *and* the
//! re-plan trigger, so the whole latency path is smoothed consistently. A source with no
//! observation has no EWMA and contributes `factor = 1.0` (inert), exactly as before.
//!
//! The weighting is deliberately simple and **honest about being a heuristic** (see
//! [`ReplanPolicy::latency_weight`]): for a candidate pattern, take the **slowest** observed
//! latency over the sources retained for it (a union is bottlenecked by its slowest arm),
//! divide by a baseline latency ([`ReplanPolicy::latency_baseline`]) to get a *relative*
//! slowness `s ≥ 0`, and scale that join's cost by
//!
//! > `factor = clamp(1 + latency_weight · (s − 1), latency_floor, latency_cap)`.
//!
//! A source at the baseline (`s = 1`) — or one with **no** latency observation — gets
//! `factor = 1.0`, i.e. the latency model is *off by construction* until evidence arrives,
//! so behaviour with no measurements is byte-identical to the cardinality-only planner. A
//! source observed at `2·baseline` with the default `latency_weight = 0.5` costs `1.5×`; the
//! clamp ([`ReplanPolicy::latency_cap`], default `4.0`) stops one outlier measurement from
//! dominating the order. This is a *bias*, not a guarantee: latency is noisy and the
//! constants are tuned by hand, not derived — it nudges the re-plan toward faster sources,
//! it does **not** claim to find the latency-optimal plan.
//!
//! Latency feeds the trigger too: a not-yet-executed pattern whose slowest source is
//! observed to be more than `divergence_factor×` the baseline is enough to *consider* a
//! re-plan (the hysteresis margin still gates whether one is *adopted*), so a query that is
//! latency-bound — not cardinality-bound — can still react.
//!
//! **The latency weighting changes execution STRATEGY/ORDERING only — never results.** It
//! enters exclusively through the *cost* term (and the suffix selection score), never the
//! output *cardinality*, so the soundness boundary below is untouched: the re-plan is still
//! a pure reorder of the not-yet-executed suffix, and the result multiset is unchanged.
//!
//! ## Soundness boundary (load-bearing — read this)
//!
//! Re-planning happens **only at stage boundaries** — between two leaf joins of the
//! left-deep plan, i.e. once an operator has fully produced its output and before the next
//! join begins. It is **NOT** a mid-operator swap: a join already in flight is never torn
//! down and rebuilt. Concretely, the executor model here ([`AdaptiveExecutor`]) runs the
//! plan as a sequence of *stages* (one per pattern after the seed); a re-plan may reorder
//! the **suffix** of stages that have not started, but never the prefix already produced.
//!
//! Why this is sound for query RESULTS:
//!
//! > A BGP is a conjunction of triple patterns; its answer is the natural join of the
//! > per-pattern solution multisets, which is **commutative and associative**. Any join
//! > order over the same set of patterns produces the **same** result multiset. A re-plan
//! > only changes the *order* in which the remaining patterns are joined (and the
//! > per-join algorithm, which [`crate::stream`] already proves is multiset-preserving) —
//! > it never adds, drops, or alters a pattern. The already-produced prefix is the partial
//! > join of an *initial subset* of the patterns; joining the remaining patterns onto it
//! > in any order yields the full join. Hence the adaptive result is bit-for-bit the same
//! > multiset as the static plan.
//!
//! The prefix's bindings are **carried across the switch unchanged** — the re-plan starts
//! from the current intermediate result, not from scratch, so no work is re-done and no
//! binding is lost or duplicated. This is verified by [`tests::replan_result_equals_static`].
//!
//! ## Deferred (filed as beads, NOT built here)
//!
//! * **Mid-operator** adaptivity — tearing down a join *while it is producing* and
//!   resuming the half-built hash tables under a new algorithm — is out of scope; the
//!   boundary above is the sound, well-scoped version. (Roadmap bead, epic sq-3183.)
//! * **Source switching** (failover to a replica mid-stage when a source goes dark) reuses
//!   the same trigger machinery but needs a live multi-source execution layer this pure
//!   crate does not own; deferred.
//!
//! [OPUS-4.8] sq-7s4z (epic sq-3183) — flagged for Fable re-review.

use crate::pattern::Bgp;
use crate::plan::{JoinTree, PlanOptions};
use crate::selection::PatternSources;
use std::collections::HashMap;

/// Policy controlling when a mid-execution re-plan fires and whether its result is adopted.
///
/// The two knobs together give the divergence-then-hysteresis behaviour: a re-plan is
/// *triggered* by a large estimate error ([`Self::divergence_factor`]) but only *adopted*
/// when the new suffix is a clear cost win ([`Self::improvement_margin`]) — so noisy-but-
/// stable statistics never cause the plan to thrash.
#[derive(Debug, Clone)]
pub struct ReplanPolicy {
    /// Divergence factor `k` (> 1). A re-plan is *considered* when an observed cardinality
    /// `o` for a not-yet-executed operator diverges from its estimate `e` by more than this
    /// factor in either direction: `o > k·e` **or** `e > k·o`. Default `4.0` (a 4× miss).
    pub divergence_factor: f64,
    /// Minimum relative cost improvement (in `[0, 1)`) the re-planned suffix must show over
    /// the current suffix's estimated remaining cost before it is adopted — the hysteresis
    /// band that prevents thrashing. `0.1` means "only switch if it is ≥ 10% cheaper".
    /// Default `0.1`: enough to absorb cost-model noise yet low enough to act on the modest
    /// (10–20%) gains a two-/three-arm suffix reorder typically yields on small
    /// intermediates — a higher band would let re-planning *fire* but never *switch*.
    pub improvement_margin: f64,
    /// Hard cap on how many re-plans a single execution may perform (belt-and-braces
    /// anti-thrash + bounded planning overhead). Default `8`.
    pub max_replans: usize,
    /// [OPUS-4.8] sq-b51o. Weight `w ≥ 0` of observed per-source latency in the cost model
    /// (a HEURISTIC constant, not a derived optimum). A candidate join whose slowest retained
    /// source is observed at relative slowness `s = latency / latency_baseline` has its cost
    /// scaled by `clamp(1 + w·(s − 1), latency_floor, latency_cap)`. `w = 0` disables latency
    /// weighting entirely (pure cardinality planning); the default `0.5` lets a 2×-slow source
    /// cost 1.5× — a deliberately *gentle* bias, because latency measurements are noisy.
    pub latency_weight: f64,
    /// [OPUS-4.8] sq-b51o. Baseline latency (same abstract units as
    /// [`RuntimeStats::record_source_latency`]) a source is judged *relative to*: a source
    /// observed at this latency is "typical" (relative slowness `s = 1` ⇒ cost factor `1.0`).
    /// A hand-set reference point, not measured. Default `100.0`.
    pub latency_baseline: f64,
    /// [OPUS-4.8] sq-b51o. Lower clamp on the latency cost factor — a source observed *faster*
    /// than baseline can make its joins cheaper, but never below this floor (a fast source is
    /// still a remote sub-query). Default `0.5`.
    pub latency_floor: f64,
    /// [OPUS-4.8] sq-b51o. Upper clamp on the latency cost factor — stops a single outlier
    /// latency spike from dominating the join order (anti-thrash for the latency term, mirroring
    /// the cardinality `divergence_factor` discipline). Default `4.0`.
    pub latency_cap: f64,
}

impl Default for ReplanPolicy {
    fn default() -> Self {
        ReplanPolicy {
            divergence_factor: 4.0,
            improvement_margin: 0.1,
            max_replans: 8,
            latency_weight: 0.5,
            latency_baseline: 100.0,
            latency_floor: 0.5,
            latency_cap: 4.0,
        }
    }
}

impl ReplanPolicy {
    /// [OPUS-4.8] sq-b51o. The latency cost-multiplier for a single observed source latency,
    /// or `None` when latency weighting is off (`latency_weight == 0`). A source at the
    /// baseline (or with no observation — handled by the caller passing `None`) yields `1.0`,
    /// so the model is inert until evidence diverges from baseline. HEURISTIC — see the
    /// module docs.
    fn latency_factor(&self, latency: f64) -> f64 {
        if self.latency_weight <= 0.0 {
            return 1.0;
        }
        let baseline = self.latency_baseline.max(f64::MIN_POSITIVE);
        let relative = latency.max(0.0) / baseline;
        let raw = 1.0 + self.latency_weight * (relative - 1.0);
        let lo = self.latency_floor.clamp(0.0, 1.0);
        let hi = self.latency_cap.max(1.0);
        raw.clamp(lo, hi)
    }
}

/// Observed runtime statistics, accumulated as execution proceeds. Distinct from the
/// descriptor-derived *estimates*: every value here is a real count / measurement the
/// executor fed in at a stage boundary.
#[derive(Debug, Clone)]
pub struct RuntimeStats {
    /// Observed leaf cardinality per pattern index (rows the pattern actually returned,
    /// union over its sources). Present only for patterns that have been (at least
    /// partially) evaluated.
    observed_leaf_card: HashMap<usize, f64>,
    /// Per-source **EWMA-smoothed** latency, in abstract time units (e.g. ms). [OPUS-4.8]
    /// sq-b51o (+ follow-up): each [`Self::record_source_latency`] folds the new observation
    /// into a per-source exponentially-weighted moving average rather than overwriting — this
    /// is the value fed into the cost model + the re-plan trigger, so a single transient spike
    /// is damped and only a *sustained* shift moves the planner. A slow source's joins cost
    /// more, biasing the re-plan toward faster sources / deferring the slow one (see
    /// [`ReplanPolicy::latency_weight`]). Affects cost/ordering ONLY, never the result multiset.
    observed_latency: HashMap<usize, f64>,
    /// [OPUS-4.8] sq-b51o follow-up. EWMA smoothing factor α ∈ (0, 1] applied per source in
    /// [`Self::record_source_latency`]: `ewma_new = α·observed + (1−α)·ewma_prev`. A
    /// **hand-picked HEURISTIC**, default [`Self::DEFAULT_LATENCY_ALPHA`] (`0.3`) — see the
    /// module docs. Lower = calmer/laggier, higher = twitchier/faster-tracking; not derived
    /// from a workload. Clamped to `(0, 1]` on construction.
    latency_alpha: f64,
}

impl Default for RuntimeStats {
    fn default() -> Self {
        RuntimeStats {
            observed_leaf_card: HashMap::new(),
            observed_latency: HashMap::new(),
            latency_alpha: RuntimeStats::DEFAULT_LATENCY_ALPHA,
        }
    }
}

impl RuntimeStats {
    /// [OPUS-4.8] sq-b51o follow-up. Default per-source latency EWMA smoothing factor α — a
    /// hand-picked heuristic (latest sample weighted 30%, history 70%): enough history to damp
    /// a single transient spike below the re-plan trigger, enough recency to converge on a
    /// sustained shift within a few samples. NOT a derived optimum.
    pub const DEFAULT_LATENCY_ALPHA: f64 = 0.3;

    /// A fresh, empty statistics store with the default latency EWMA α
    /// ([`Self::DEFAULT_LATENCY_ALPHA`]).
    pub fn new() -> RuntimeStats {
        RuntimeStats::default()
    }

    /// [OPUS-4.8] sq-b51o follow-up. A fresh, empty statistics store with an explicit latency
    /// EWMA smoothing factor `alpha` (clamped to `(0, 1]`). `alpha = 1.0` recovers the old
    /// "single last observation" behaviour (no smoothing); the default
    /// ([`Self::DEFAULT_LATENCY_ALPHA`]) is `0.3`.
    pub fn with_latency_alpha(alpha: f64) -> RuntimeStats {
        RuntimeStats {
            latency_alpha: Self::clamp_alpha(alpha),
            ..RuntimeStats::default()
        }
    }

    /// The configured latency EWMA smoothing factor α (always in `(0, 1]`).
    pub fn latency_alpha(&self) -> f64 {
        self.latency_alpha
    }

    /// Clamp a requested α into the valid `(0, 1]` range (a degenerate `α ≤ 0` would freeze
    /// the average at its seed; `α > 1` would over-shoot). Uses `f64::MIN_POSITIVE` as the
    /// strictly-positive floor.
    fn clamp_alpha(alpha: f64) -> f64 {
        if alpha.is_nan() {
            return Self::DEFAULT_LATENCY_ALPHA;
        }
        alpha.clamp(f64::MIN_POSITIVE, 1.0)
    }

    /// Record the *observed* cardinality of a pattern's leaf (the real row count its
    /// source(s) returned). Overwrites any prior observation for that pattern.
    pub fn record_leaf_cardinality(&mut self, pattern: usize, observed: f64) {
        self.observed_leaf_card.insert(pattern, observed.max(0.0));
    }

    /// Record an *observed* latency sample for `source` (abstract time units), folding it into
    /// that source's **EWMA**: `ewma_new = α·observed + (1−α)·ewma_prev`, where α is
    /// [`Self::latency_alpha`]. The **first** sample for a source seeds the average
    /// (`ewma = observed`); subsequent samples decay the history. The smoothed value — not the
    /// raw sample — is what [`Self::observed_latency_of`] returns and the cost model/trigger
    /// read, so a single transient spike is damped (anti-thrash) while a sustained shift
    /// converges. [OPUS-4.8] sq-b51o follow-up. Affects cost/ordering ONLY, never results.
    pub fn record_source_latency(&mut self, source: usize, latency: f64) {
        let observed = latency.max(0.0);
        let entry = self.observed_latency.entry(source);
        match entry {
            std::collections::hash_map::Entry::Occupied(mut o) => {
                let prev = *o.get();
                let next = self.latency_alpha * observed + (1.0 - self.latency_alpha) * prev;
                o.insert(next);
            }
            std::collections::hash_map::Entry::Vacant(v) => {
                v.insert(observed);
            }
        }
    }

    /// The observed leaf cardinality for `pattern`, if recorded.
    pub fn observed_leaf(&self, pattern: usize) -> Option<f64> {
        self.observed_leaf_card.get(&pattern).copied()
    }

    /// The **EWMA-smoothed** observed latency for `source`, if any sample has been recorded.
    /// This is the smoothed running average, not the last raw sample ([OPUS-4.8] sq-b51o
    /// follow-up) — it is what the latency cost factor and the re-plan trigger consume.
    pub fn observed_latency_of(&self, source: usize) -> Option<f64> {
        self.observed_latency.get(&source).copied()
    }
}

/// Builds a fresh per-pattern source selection whose leaf cardinality estimates are
/// **overridden by observed values** where the runtime stats have them. Patterns with no
/// observation keep their original (descriptor-derived) estimate.
///
/// The override is applied proportionally across a pattern's retained sources so that the
/// per-source candidate breakdown — which the star-cardinality estimator reads — stays
/// consistent with the new total. Source *membership* is never changed here (re-planning
/// reorders; it does not re-prune, which keeps recall-safety intact).
fn corrected_selection(base: &[PatternSources], stats: &RuntimeStats) -> Vec<PatternSources> {
    base.iter()
        .map(|ps| {
            let Some(obs) = stats.observed_leaf(ps.pattern) else {
                return ps.clone();
            };
            let mut ps = ps.clone();
            let old_total: f64 = ps.candidates.iter().map(|c| c.estimated_cardinality).sum();
            if ps.candidates.is_empty() {
                return ps;
            }
            if old_total > 0.0 {
                // Scale each source's share to hit the observed total, preserving skew.
                let scale = obs / old_total;
                for c in &mut ps.candidates {
                    c.estimated_cardinality *= scale;
                }
            } else {
                // No prior signal to distribute on — spread the observation evenly.
                let per = obs / ps.candidates.len() as f64;
                for c in &mut ps.candidates {
                    c.estimated_cardinality = per;
                }
            }
            ps
        })
        .collect()
}

/// [OPUS-4.8] sq-b51o. The **slowest** observed latency over the sources retained for
/// `pattern` (a union is bottlenecked by its slowest arm), or `None` when no source of the
/// pattern has a latency observation yet.
fn pattern_slowest_latency(
    pattern: usize,
    selection: &[PatternSources],
    stats: &RuntimeStats,
) -> Option<f64> {
    let ps = selection.iter().find(|ps| ps.pattern == pattern)?;
    ps.candidates
        .iter()
        .filter_map(|c| stats.observed_latency_of(c.source))
        .fold(None::<f64>, |acc, l| Some(acc.map_or(l, |a| a.max(l))))
}

/// [OPUS-4.8] sq-b51o. The latency cost-multiplier for joining `pattern`: its slowest source's
/// observed latency run through [`ReplanPolicy::latency_factor`]. Returns `1.0` (inert) when no
/// source of the pattern has a latency observation — the latency model contributes nothing
/// until evidence arrives, so a measurement-free re-plan is byte-identical to cardinality-only.
fn pattern_latency_factor(
    pattern: usize,
    selection: &[PatternSources],
    stats: &RuntimeStats,
    policy: &ReplanPolicy,
) -> f64 {
    match pattern_slowest_latency(pattern, selection, stats) {
        Some(l) => policy.latency_factor(l),
        None => 1.0,
    }
}

/// [OPUS-4.8] sq-b51o. The pattern's slowest source's *relative slowness* `s = latency /
/// baseline` (un-clamped, un-weighted) under `policy`'s [`ReplanPolicy::latency_baseline`] —
/// what the trigger compares to `divergence_factor`. `None` when the pattern has no latency
/// observation.
fn pattern_relative_slowness(
    pattern: usize,
    selection: &[PatternSources],
    stats: &RuntimeStats,
    policy: &ReplanPolicy,
) -> Option<f64> {
    let baseline = policy.latency_baseline.max(f64::MIN_POSITIVE);
    pattern_slowest_latency(pattern, selection, stats).map(|l| l / baseline)
}

/// Whether observed cardinality `observed` diverges from estimate `estimate` by more than
/// the policy's factor (in either direction).
fn diverges(estimate: f64, observed: f64, policy: &ReplanPolicy) -> bool {
    let k = policy.divergence_factor.max(1.0);
    let e = estimate.max(1.0);
    let o = observed.max(0.0);
    o > k * e || e > k * o.max(1.0)
}

/// The outcome of one re-plan attempt at a stage boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplanOutcome {
    /// No observed statistic for a not-yet-executed pattern diverged past the threshold;
    /// the trigger did not fire.
    NoDivergence,
    /// The trigger fired but the re-planned suffix did not beat the current one by the
    /// hysteresis margin — the existing plan is kept (anti-thrash).
    KeptWithinHysteresis,
    /// The re-plan fired and the new suffix won by more than the margin — the remaining
    /// join order was switched.
    Switched,
    /// The re-plan budget ([`ReplanPolicy::max_replans`]) was exhausted; no further
    /// re-planning is attempted this execution.
    BudgetExhausted,
}

/// An adaptive executor that runs a static [`JoinTree`] as a sequence of **stages** (the
/// left-deep join order), capturing runtime statistics and re-planning the not-yet-run
/// suffix at stage boundaries.
///
/// This is the orchestration model the soundness boundary is stated against: the executor
/// always holds (a) the patterns already joined — the **prefix** — and (b) the patterns
/// still to join — the **suffix**. A re-plan reorders only the suffix.
pub struct AdaptiveExecutor<'a> {
    bgp: &'a Bgp,
    descriptors: &'a [crate::descriptor::SourceDescriptor],
    base_selection: &'a [PatternSources],
    opts: PlanOptions,
    policy: ReplanPolicy,
    /// Patterns already joined, in execution order (the prefix). The first is the seed.
    executed: Vec<usize>,
    /// Patterns not yet joined, in their *current planned* order (the suffix).
    remaining: Vec<usize>,
    replans_done: usize,
}

impl<'a> AdaptiveExecutor<'a> {
    /// Starts an adaptive execution from a static `plan` (its join order seeds the prefix
    /// /suffix split: nothing executed yet, the whole order is the suffix).
    pub fn new(
        bgp: &'a Bgp,
        descriptors: &'a [crate::descriptor::SourceDescriptor],
        base_selection: &'a [PatternSources],
        plan: &JoinTree,
        opts: PlanOptions,
        policy: ReplanPolicy,
    ) -> AdaptiveExecutor<'a> {
        AdaptiveExecutor {
            bgp,
            descriptors,
            base_selection,
            opts,
            policy,
            executed: Vec::new(),
            remaining: plan.join_order(),
            replans_done: 0,
        }
    }

    /// The patterns already executed (the prefix), in execution order.
    pub fn executed_order(&self) -> &[usize] {
        &self.executed
    }

    /// The patterns not yet executed (the suffix), in current planned order.
    pub fn remaining_order(&self) -> &[usize] {
        &self.remaining
    }

    /// Marks the front pattern of the suffix as executed (an operator completed), moving it
    /// into the prefix. Returns the pattern index, or `None` if the suffix is empty.
    pub fn advance(&mut self) -> Option<usize> {
        if self.remaining.is_empty() {
            return None;
        }
        let p = self.remaining.remove(0);
        self.executed.push(p);
        Some(p)
    }

    /// Considers a re-plan at the current stage boundary given the latest `stats`.
    ///
    /// Fires the trigger when a not-yet-executed pattern's observed leaf cardinality
    /// diverges from its estimate past the policy factor, **or** ([OPUS-4.8] sq-b51o) a
    /// not-yet-executed pattern's slowest source is observed to be `divergence_factor×`
    /// slower than the latency baseline; if so, re-plans the **suffix** with the corrected
    /// statistics (cardinality *and* latency-weighted cost) and adopts the new order only
    /// when it wins by the hysteresis margin. Returns the [`ReplanOutcome`]. The prefix is
    /// never touched, and latency enters cost/ordering ONLY — never the result multiset.
    pub fn maybe_replan(&mut self, stats: &RuntimeStats) -> ReplanOutcome {
        if self.remaining.len() < 2 {
            // Fewer than two remaining patterns ⇒ nothing to reorder.
            return ReplanOutcome::NoDivergence;
        }
        if self.replans_done >= self.policy.max_replans {
            return ReplanOutcome::BudgetExhausted;
        }

        // ---- Trigger: did any NOT-YET-EXECUTED pattern's observation diverge?
        let base_est: HashMap<usize, f64> = self
            .base_selection
            .iter()
            .map(|ps| (ps.pattern, ps.total_cardinality().max(0.0)))
            .collect();
        let card_triggered =
            self.remaining
                .iter()
                .any(|&p| match (base_est.get(&p), stats.observed_leaf(p)) {
                    (Some(&e), Some(o)) => diverges(e, o, &self.policy),
                    _ => false,
                });
        // [OPUS-4.8] sq-b51o (+ EWMA follow-up): a not-yet-executed pattern bound to an
        // observably slow source — whose slowest source's EWMA-SMOOTHED latency is past
        // divergence_factor× baseline — is also enough to *consider* a re-plan: a latency-bound
        // query reacts even when cardinalities are spot-on. Reading the EWMA (not the raw last
        // sample) means one transient spike does not move the smoothed value over the threshold,
        // so the trigger is anti-thrashed at the source; only a sustained shift converges past
        // it. The hysteresis margin still gates whether a switch is actually *adopted*.
        let lat_triggered = self.policy.latency_weight > 0.0
            && self.remaining.iter().any(|&p| {
                pattern_relative_slowness(p, self.base_selection, stats, &self.policy)
                    .is_some_and(|s| s > self.policy.divergence_factor.max(1.0))
            });
        if !card_triggered && !lat_triggered {
            return ReplanOutcome::NoDivergence;
        }

        // ---- Re-plan the SUFFIX with corrected statistics.
        // Build a sub-BGP of the remaining patterns; plan over it with observed cardinalities
        // and latency-weighted join costs.
        let corrected = corrected_selection(self.base_selection, stats);
        let Some(new_suffix) = self.replan_suffix(&corrected, stats) else {
            return ReplanOutcome::KeptWithinHysteresis;
        };

        // ---- Hysteresis: only adopt if the new suffix beats the current one by the margin.
        let cur_cost = self.suffix_cost(&self.remaining, &corrected, stats);
        let new_cost = self.suffix_cost(&new_suffix, &corrected, stats);
        let margin = self.policy.improvement_margin.clamp(0.0, 0.999);
        // new must be cheaper than (1 - margin) * current to win.
        if new_cost < cur_cost * (1.0 - margin) && new_suffix != self.remaining {
            self.remaining = new_suffix;
            self.replans_done += 1;
            ReplanOutcome::Switched
        } else {
            ReplanOutcome::KeptWithinHysteresis
        }
    }

    /// Re-plans just the remaining patterns, returning the new suffix order. Anchored to the
    /// already-executed prefix: the re-planner sees the prefix as "already joined" so it
    /// continues to prefer patterns connected to the running result (no Cartesian arm gets
    /// promoted spuriously). Returns `None` if the sub-plan is degenerate.
    fn replan_suffix(
        &self,
        corrected: &[PatternSources],
        stats: &RuntimeStats,
    ) -> Option<Vec<usize>> {
        // Plan the whole BGP with corrected stats but *force the prefix to come first* by
        // seeding the greedy planner's "joined" set with the executed prefix. The public
        // `plan_bgp` always re-chooses the seed, so we re-derive the suffix order with a
        // prefix-anchored greedy pass that mirrors `plan_bgp`'s join-selection rule.
        Some(plan_suffix_greedy(
            self.bgp,
            &self.executed,
            &self.remaining,
            corrected,
            self.descriptors,
            &self.opts,
            stats,
            &self.policy,
        ))
    }

    /// Estimated remaining cost of executing `suffix` (in this order) on top of the current
    /// prefix, under the corrected statistics — the comparable used for hysteresis. Costs are
    /// **latency-weighted** ([OPUS-4.8] sq-b51o): a join against a slow source costs more, so a
    /// suffix that defers the slow source compares favourably. Output cardinality is unaffected.
    fn suffix_cost(
        &self,
        suffix: &[usize],
        corrected: &[PatternSources],
        stats: &RuntimeStats,
    ) -> f64 {
        suffix_cost_greedy(
            self.bgp,
            &self.executed,
            suffix,
            corrected,
            self.descriptors,
            &self.opts,
            stats,
            &self.policy,
        )
    }
}

/// Re-derives a greedy left-deep order for `suffix`, anchored on the already-joined
/// `prefix`. Mirrors [`crate::plan_bgp`]'s selection rule (connected-first, smallest output
/// next, tie-break on index) but over a fixed prefix — so the only thing that can change is
/// the order of the not-yet-executed patterns. Pure, deterministic.
///
/// [OPUS-4.8] sq-b51o: the per-candidate selection key is the **latency-weighted** output
/// score `out · latency_factor(cand)` rather than raw output cardinality — a candidate bound
/// to an observably slow source is deferred. The factor is `1.0` whenever the candidate has no
/// latency observation (or latency weighting is off), so a measurement-free re-plan is
/// byte-identical to the cardinality-only planner (the bias enters *on the same axis* as
/// cardinality; it never alters the output cardinality the next join's estimate uses).
#[allow(clippy::too_many_arguments)]
fn plan_suffix_greedy(
    bgp: &Bgp,
    prefix: &[usize],
    suffix: &[usize],
    selection: &[PatternSources],
    descriptors: &[crate::descriptor::SourceDescriptor],
    opts: &PlanOptions,
    stats: &RuntimeStats,
    policy: &ReplanPolicy,
) -> Vec<usize> {
    let leaf_card: Vec<f64> = selection
        .iter()
        .map(|s| s.total_cardinality().max(0.0))
        .collect();
    // The running result starts as the prefix; estimate its cardinality as the product-free
    // greedy estimate of the prefix in its fixed order (only relative ordering of the suffix
    // matters for the comparison, so an approximate prefix cardinality is fine and identical
    // across the candidate suffixes).
    let mut joined: Vec<usize> = prefix.to_vec();
    let mut cur_card = prefix_cardinality(bgp, prefix, &leaf_card, descriptors, selection, opts);
    if joined.is_empty() {
        // No prefix: seed with the smallest-cardinality remaining pattern (mirrors plan_bgp).
        if let Some(&seed) = suffix.iter().min_by(|&&a, &&b| {
            leaf_card[a]
                .partial_cmp(&leaf_card[b])
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.cmp(&b))
        }) {
            joined.push(seed);
            cur_card = leaf_card[seed];
        }
    }
    let mut rem: Vec<usize> = suffix
        .iter()
        .copied()
        .filter(|p| !joined.contains(p))
        .collect();
    let mut order: Vec<usize> = joined
        .iter()
        .copied()
        .filter(|p| suffix.contains(p))
        .collect();

    while !rem.is_empty() {
        // (pat, true_out_card, latency_weighted_score, connected)
        let mut best: Option<(usize, f64, f64, bool)> = None;
        for &cand in &rem {
            let connected = joined.iter().any(|&j| bgp.shares_var(j, cand));
            let (out, _cost, _algo) = crate::plan::cost_join_pub(
                bgp,
                cur_card,
                &joined,
                cand,
                &leaf_card,
                descriptors,
                selection,
                opts,
                connected,
            );
            // [OPUS-4.8] sq-b51o: rank on the latency-weighted score, but advance `cur_card`
            // with the TRUE `out` — latency never changes the cardinality the planner feeds
            // forward (results/soundness are untouched), only which arm we pick next.
            let score = out * pattern_latency_factor(cand, selection, stats, policy);
            let better = match &best {
                None => true,
                Some((bp, _bcard, bscore, bconn)) => match (connected, *bconn) {
                    (true, false) => true,
                    (false, true) => false,
                    _ => score < *bscore || (score == *bscore && cand < *bp),
                },
            };
            if better {
                best = Some((cand, out, score, connected));
            }
        }
        let (pat, out, _, _) = best.expect("rem non-empty");
        order.push(pat);
        joined.push(pat);
        cur_card = out;
        rem.retain(|&p| p != pat);
    }
    order
}

/// Estimated total remaining cost of joining `suffix` (in the given order) onto `prefix`.
/// [OPUS-4.8] sq-b51o: each per-join cost is scaled by the candidate's latency factor (a slow
/// source's joins cost more); the output cardinality fed forward is the TRUE estimate, so
/// latency biases the cost comparable only, never the result.
#[allow(clippy::too_many_arguments)]
fn suffix_cost_greedy(
    bgp: &Bgp,
    prefix: &[usize],
    suffix: &[usize],
    selection: &[PatternSources],
    descriptors: &[crate::descriptor::SourceDescriptor],
    opts: &PlanOptions,
    stats: &RuntimeStats,
    policy: &ReplanPolicy,
) -> f64 {
    let leaf_card: Vec<f64> = selection
        .iter()
        .map(|s| s.total_cardinality().max(0.0))
        .collect();
    let mut joined: Vec<usize> = prefix.to_vec();
    let mut cur_card = prefix_cardinality(bgp, prefix, &leaf_card, descriptors, selection, opts);
    if joined.is_empty() {
        if let Some(&first) = suffix.first() {
            joined.push(first);
            cur_card = leaf_card[first];
        }
    }
    let mut total = 0.0f64;
    for &cand in suffix {
        if joined.contains(&cand) {
            continue;
        }
        let connected = joined.iter().any(|&j| bgp.shares_var(j, cand));
        let (out, cost, _algo) = crate::plan::cost_join_pub(
            bgp,
            cur_card,
            &joined,
            cand,
            &leaf_card,
            descriptors,
            selection,
            opts,
            connected,
        );
        // [OPUS-4.8] sq-b51o: latency-weight the cost of this join; cardinality fed forward
        // (`out`) stays the true estimate, so the comparable is latency-aware but the plan
        // remains a pure reorder (result-equivalent).
        total += cost * pattern_latency_factor(cand, selection, stats, policy);
        cur_card = out;
        joined.push(cand);
    }
    total
}

/// Approximate cardinality of the running result after the fixed `prefix` (used as the left
/// size when costing the suffix). Folds the prefix in its given order through the same
/// per-join estimate the planner uses. An empty prefix yields `1.0` (neutral).
fn prefix_cardinality(
    bgp: &Bgp,
    prefix: &[usize],
    leaf_card: &[f64],
    descriptors: &[crate::descriptor::SourceDescriptor],
    selection: &[PatternSources],
    opts: &PlanOptions,
) -> f64 {
    if prefix.is_empty() {
        return 1.0;
    }
    let mut joined: Vec<usize> = vec![prefix[0]];
    let mut cur = leaf_card[prefix[0]];
    for &cand in &prefix[1..] {
        let connected = joined.iter().any(|&j| bgp.shares_var(j, cand));
        let (out, _cost, _algo) = crate::plan::cost_join_pub(
            bgp,
            cur,
            &joined,
            cand,
            leaf_card,
            descriptors,
            selection,
            opts,
            connected,
        );
        cur = out;
        joined.push(cand);
    }
    cur
}

/// A minimal reference BGP executor used to PROVE result-equivalence under re-planning: it
/// joins the per-pattern solution multisets in any given order and returns the result.
///
/// This is intentionally *order-driven* — you hand it a join order and it folds the
/// patterns' solution sets together left-deep using [`crate::blocking_hash_join`] on the
/// shared variables. Because BGP join is commutative/associative, evaluating the same
/// patterns in two different orders must yield the same result multiset; the adaptive
/// tests drive this with a static order and a mid-execution re-planned order and assert
/// equality.
pub mod exec_oracle {
    use crate::pattern::{Bgp, Term, Var};
    use crate::stream::{blocking_hash_join, Tuple};

    /// Per-pattern solution multisets, indexed by pattern position in the BGP. Each is the
    /// (already-fetched) set of tuples a pattern's source(s) returned.
    pub type SolutionSets = Vec<Vec<Tuple>>;

    /// The shared (join) variables between the variables already produced by the running
    /// result and a candidate pattern.
    fn shared_vars(produced: &[Var], pattern: &crate::pattern::TriplePattern) -> Vec<Var> {
        pattern
            .vars()
            .into_iter()
            .filter(|v| produced.contains(v))
            .cloned()
            .collect()
    }

    /// The variables a pattern binds (its variable positions), as owned `Var`s.
    fn pattern_vars(pattern: &crate::pattern::TriplePattern) -> Vec<Var> {
        pattern.vars().into_iter().cloned().collect()
    }

    /// Evaluates the BGP by joining the per-pattern `solutions` in the given `order`,
    /// left-deep. Returns the result multiset. Patterns sharing no variable with the
    /// running result are Cartesian-joined (empty join key ⇒ full cross product), matching
    /// SPARQL BGP semantics. Deterministic given the inputs and order.
    pub fn evaluate(bgp: &Bgp, solutions: &SolutionSets, order: &[usize]) -> Vec<Tuple> {
        if order.is_empty() {
            return Vec::new();
        }
        let mut produced: Vec<Var> = pattern_vars(&bgp.patterns[order[0]]);
        let mut acc: Vec<Tuple> = solutions[order[0]].clone();
        for &p in &order[1..] {
            let pat = &bgp.patterns[p];
            let jv = shared_vars(&produced, pat);
            acc = if jv.is_empty() {
                cross_product(&acc, &solutions[p])
            } else {
                blocking_hash_join(&acc, &solutions[p], &jv)
            };
            for v in pattern_vars(pat) {
                if !produced.contains(&v) {
                    produced.push(v);
                }
            }
        }
        acc
    }

    /// Full cross product of two solution multisets (a disconnected/Cartesian join).
    fn cross_product(left: &[Tuple], right: &[Tuple]) -> Vec<Tuple> {
        let mut out = Vec::with_capacity(left.len() * right.len());
        for l in left {
            for r in right {
                out.push(merge(l, r));
            }
        }
        out
    }

    /// Merges two tuples (union of bindings; on a shared var the left value wins — equal by
    /// construction when reached via a real join, arbitrary-but-consistent for Cartesian).
    fn merge(l: &Tuple, r: &Tuple) -> Tuple {
        let mut pairs: Vec<(Var, String)> = l
            .bindings()
            .iter()
            .map(|(v, s)| (v.clone(), s.clone()))
            .collect();
        for (v, s) in r.bindings() {
            if !pairs.iter().any(|(pv, _)| pv == v) {
                pairs.push((v.clone(), s.clone()));
            }
        }
        Tuple::new(pairs)
    }

    /// Convenience: build a `Term::Var`.
    pub fn v(name: &str) -> Term {
        Term::Var(Var::new(name))
    }
    /// Convenience: build a `Term::Iri`.
    pub fn iri(s: &str) -> Term {
        Term::Iri(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::exec_oracle::{evaluate, iri, v, SolutionSets};
    use super::*;
    use crate::descriptor::{PredPartition, SourceDescriptor, SourceId};
    use crate::pattern::{TriplePattern, Var};
    use crate::plan::plan_bgp;
    use crate::selection::select_sources;
    use crate::stream::Tuple;

    fn pred(p: &str, triples: u64, subjects: u64, objects: u64) -> PredPartition {
        PredPartition {
            predicate: p.into(),
            triples,
            distinct_subjects: subjects,
            distinct_objects: objects,
        }
    }

    fn t(pairs: &[(&str, &str)]) -> Tuple {
        Tuple::new(
            pairs
                .iter()
                .map(|(name, val)| (Var::new(*name), val.to_string())),
        )
    }

    fn multiset_eq(a: &[Tuple], b: &[Tuple]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let mut a: Vec<String> = a.iter().map(canon).collect();
        let mut b: Vec<String> = b.iter().map(canon).collect();
        a.sort();
        b.sort();
        a == b
    }
    fn canon(t: &Tuple) -> String {
        let mut parts: Vec<String> = t
            .bindings()
            .iter()
            .map(|(v, s)| format!("{}={}", v.0, s))
            .collect();
        parts.sort();
        parts.join("|")
    }

    // A 3-pattern chain: ?s :p ?o . ?o :q ?z . ?z :r ?w
    fn chain_bgp() -> Bgp {
        Bgp::new(vec![
            TriplePattern::new(v("s"), iri("http://ex/p"), v("o")),
            TriplePattern::new(v("o"), iri("http://ex/q"), v("z")),
            TriplePattern::new(v("z"), iri("http://ex/r"), v("w")),
        ])
    }

    // A 3-arm star on ?s: ?s :a ?x . ?s :b ?y . ?s :c ?z — all join on ?s, so the suffix
    // order after the seed is driven purely by the arms' cardinalities. Used by the trigger
    // test because here a divergent observation genuinely *flips* the remaining order.
    fn star_bgp() -> Bgp {
        Bgp::new(vec![
            TriplePattern::new(v("s"), iri("http://ex/a"), v("x")),
            TriplePattern::new(v("s"), iri("http://ex/b"), v("y")),
            TriplePattern::new(v("s"), iri("http://ex/c"), v("z")),
        ])
    }

    // ---- TRIGGER fires on divergent stats AND flips the remaining order.
    #[test]
    fn replan_trigger_fires_on_divergence() {
        let bgp = star_bgp();
        // Estimates: :a=10 (seed), :b=20, :c=1000 ⇒ static suffix after :a is [b, c].
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(100_000)
            .predicate(pred("http://ex/a", 10, 10, 10))
            .predicate(pred("http://ex/b", 20, 20, 20))
            .predicate(pred("http://ex/c", 1000, 1000, 1000))
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let plan = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        let mut exec = AdaptiveExecutor::new(
            &bgp,
            &srcs,
            &sel,
            &plan,
            PlanOptions::default(),
            ReplanPolicy::default(),
        );
        // Execute the seed (most selective arm, :a).
        let seed = exec.advance().unwrap();
        assert_eq!(seed, 0, ":a is the seed");
        let suffix_before = exec.remaining_order().to_vec();
        assert_eq!(suffix_before, vec![1, 2], "static suffix is [:b, :c]");
        // Now reality diverges: :b is actually HUGE (5000, est 20 ⇒ >4x) and :c is actually
        // tiny (5, est 1000 ⇒ >4x). The cheap-now arm :c should be re-ordered ahead of :b.
        let mut stats = RuntimeStats::new();
        stats.record_leaf_cardinality(1, 5000.0); // :b blew up.
        stats.record_leaf_cardinality(2, 5.0); // :c collapsed.
        let outcome = exec.maybe_replan(&stats);
        assert_eq!(
            outcome,
            ReplanOutcome::Switched,
            "divergent stats that make a different order cheaper must trigger + switch"
        );
        assert_eq!(
            exec.remaining_order(),
            &[2, 1],
            "the now-cheap arm :c is re-ordered ahead of the now-expensive :b"
        );
    }

    // ---- NO thrash on stable stats.
    #[test]
    fn no_replan_when_stats_stable() {
        let bgp = chain_bgp();
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(100_000)
            .predicate(pred("http://ex/p", 1000, 1000, 1000))
            .predicate(pred("http://ex/q", 10, 10, 10))
            .predicate(pred("http://ex/r", 500, 500, 500))
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let plan = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        let mut exec = AdaptiveExecutor::new(
            &bgp,
            &srcs,
            &sel,
            &plan,
            PlanOptions::default(),
            ReplanPolicy::default(),
        );
        let _ = exec.advance();
        // Observe cardinalities essentially equal to the estimates ⇒ no divergence.
        let mut stats = RuntimeStats::new();
        stats.record_leaf_cardinality(0, 1000.0);
        stats.record_leaf_cardinality(2, 500.0);
        assert_eq!(exec.maybe_replan(&stats), ReplanOutcome::NoDivergence);
        // Even a moderate miss (2x, below the 4x factor) does not fire.
        let mut stats2 = RuntimeStats::new();
        stats2.record_leaf_cardinality(0, 2000.0);
        assert_eq!(exec.maybe_replan(&stats2), ReplanOutcome::NoDivergence);
    }

    // ---- HYSTERESIS: trigger fires AND a cheaper order exists, but the win is below the
    //      margin ⇒ keep the current plan (anti-thrash). Same divergent fixture as the
    //      trigger test (where the *default* margin would switch) — only the high margin
    //      differs, isolating the hysteresis behaviour.
    #[test]
    fn hysteresis_rejects_marginal_replan() {
        let bgp = star_bgp();
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(100_000)
            .predicate(pred("http://ex/a", 10, 10, 10))
            .predicate(pred("http://ex/b", 20, 20, 20))
            .predicate(pred("http://ex/c", 1000, 1000, 1000))
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let plan = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        // A near-1.0 improvement margin means almost no re-plan ever clears the bar, even a
        // genuinely cheaper one — the anti-thrash band.
        let policy = ReplanPolicy {
            divergence_factor: 4.0,
            improvement_margin: 0.999,
            max_replans: 8,
            ..ReplanPolicy::default()
        };
        let mut exec =
            AdaptiveExecutor::new(&bgp, &srcs, &sel, &plan, PlanOptions::default(), policy);
        let _ = exec.advance();
        // Exactly the divergence the trigger test switches on under the default margin.
        let mut stats = RuntimeStats::new();
        stats.record_leaf_cardinality(1, 5000.0); // diverges (triggers)…
        stats.record_leaf_cardinality(2, 5.0);
        // …but the margin is so high the switch is rejected (anti-thrash); order unchanged.
        assert_eq!(
            exec.maybe_replan(&stats),
            ReplanOutcome::KeptWithinHysteresis
        );
        assert_eq!(
            exec.remaining_order(),
            &[1, 2],
            "rejected re-plan leaves the suffix order untouched"
        );
    }

    // ---- RESULT-EQUIVALENCE: a re-planned (reordered) execution yields EXACTLY the static
    //      plan's result multiset. This is the load-bearing soundness test.
    #[test]
    fn replan_result_equals_static() {
        // Star on ?s: ?s :a ?x . ?s :b ?y . ?s :c ?z — a divergent observation FLIPS the
        // suffix order (verified non-vacuously below), so this tests result-equivalence
        // across a *genuine* mid-execution switch, not a no-op re-plan.
        let bgp = star_bgp();
        // Concrete solution multisets per pattern (already-fetched federated results). Chosen
        // so the join is non-trivial: some subjects drop (no match on an arm), some fan out.
        let solutions: SolutionSets = vec![
            // pattern 0: ?s :a ?x
            vec![
                t(&[("s", "s1"), ("x", "x1")]),
                t(&[("s", "s2"), ("x", "x2")]),
                t(&[("s", "s3"), ("x", "x3")]),
            ],
            // pattern 1: ?s :b ?y  (s3 absent ⇒ dropped; s1 fans out to two y's)
            vec![
                t(&[("s", "s1"), ("y", "y1")]),
                t(&[("s", "s1"), ("y", "y7")]),
                t(&[("s", "s2"), ("y", "y2")]),
            ],
            // pattern 2: ?s :c ?z  (s2 absent ⇒ dropped)
            vec![
                t(&[("s", "s1"), ("z", "z1")]),
                t(&[("s", "s3"), ("z", "z3")]),
            ],
        ];
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(100_000)
            .predicate(pred("http://ex/a", 10, 10, 10))
            .predicate(pred("http://ex/b", 20, 20, 20))
            .predicate(pred("http://ex/c", 1000, 1000, 1000))
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let plan = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        let static_order = plan.join_order();
        let static_result = evaluate(&bgp, &solutions, &static_order);

        // Drive an adaptive execution: run the seed, then force a re-plan with divergent
        // stats that reorders the suffix, then run to completion in the NEW order.
        let mut exec = AdaptiveExecutor::new(
            &bgp,
            &srcs,
            &sel,
            &plan,
            PlanOptions::default(),
            ReplanPolicy::default(),
        );
        let _seed = exec.advance().unwrap();
        let mut stats = RuntimeStats::new();
        stats.record_leaf_cardinality(1, 5000.0); // :b blew up (est 20).
        stats.record_leaf_cardinality(2, 5.0); // :c collapsed (est 1000).
        let outcome = exec.maybe_replan(&stats);
        assert_eq!(
            outcome,
            ReplanOutcome::Switched,
            "fixture is designed to switch — equivalence is tested across a real reorder"
        );
        // Build the full execution order = prefix ++ remaining (re-planned) suffix.
        let mut adaptive_order = exec.executed_order().to_vec();
        adaptive_order.extend_from_slice(exec.remaining_order());
        // The switch genuinely changed the order (non-vacuous).
        assert_ne!(
            adaptive_order, static_order,
            "the re-plan must actually change the order here"
        );
        // …yet it is a permutation of the same pattern set (no pattern added/dropped).
        let mut a_sorted = adaptive_order.clone();
        let mut s_sorted = static_order.clone();
        a_sorted.sort();
        s_sorted.sort();
        assert_eq!(a_sorted, s_sorted, "re-plan must preserve the pattern SET");
        // The load-bearing invariant: same result multiset despite the different order.
        let adaptive_result = evaluate(&bgp, &solutions, &adaptive_order);
        assert!(
            multiset_eq(&static_result, &adaptive_result),
            "adaptive (re-planned) result MUST equal the static plan's result multiset"
        );
        assert!(!static_result.is_empty(), "fixture must produce some rows");
    }

    // ---- Result-equivalence holds for EVERY permutation of a connected BGP (exhaustive),
    //      proving the commutativity the soundness argument rests on.
    #[test]
    fn evaluate_is_order_independent_all_permutations() {
        let bgp = chain_bgp();
        let solutions: SolutionSets = vec![
            vec![
                t(&[("s", "s1"), ("o", "o1")]),
                t(&[("s", "s2"), ("o", "o2")]),
                t(&[("s", "s3"), ("o", "o1")]),
            ],
            vec![
                t(&[("o", "o1"), ("z", "z1")]),
                t(&[("o", "o2"), ("z", "z2")]),
            ],
            vec![
                t(&[("z", "z1"), ("w", "w1")]),
                t(&[("z", "z2"), ("w", "w2")]),
                t(&[("z", "z1"), ("w", "w3")]),
            ],
        ];
        let perms = [
            vec![0, 1, 2],
            vec![0, 2, 1],
            vec![1, 0, 2],
            vec![1, 2, 0],
            vec![2, 0, 1],
            vec![2, 1, 0],
        ];
        let reference = evaluate(&bgp, &solutions, &perms[0]);
        for p in &perms[1..] {
            let got = evaluate(&bgp, &solutions, p);
            assert!(
                multiset_eq(&reference, &got),
                "BGP evaluation must be order-independent for permutation {:?}",
                p
            );
        }
    }

    // ---- Budget exhaustion stops further re-planning.
    #[test]
    fn budget_exhaustion_stops_replanning() {
        let bgp = chain_bgp();
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(100_000)
            .predicate(pred("http://ex/p", 1000, 1000, 1000))
            .predicate(pred("http://ex/q", 10, 10, 10))
            .predicate(pred("http://ex/r", 500, 500, 500))
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let plan = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        let policy = ReplanPolicy {
            divergence_factor: 4.0,
            improvement_margin: 0.2,
            max_replans: 0, // no re-plans allowed.
            ..ReplanPolicy::default()
        };
        let mut exec =
            AdaptiveExecutor::new(&bgp, &srcs, &sel, &plan, PlanOptions::default(), policy);
        let _ = exec.advance();
        let mut stats = RuntimeStats::new();
        stats.record_leaf_cardinality(0, 5.0);
        assert_eq!(exec.maybe_replan(&stats), ReplanOutcome::BudgetExhausted);
    }

    // ---- Latency capture is recorded and readable.
    #[test]
    fn latency_is_captured() {
        let mut stats = RuntimeStats::new();
        stats.record_source_latency(3, 250.0);
        assert_eq!(stats.observed_latency_of(3), Some(250.0));
        assert_eq!(stats.observed_latency_of(0), None);
    }

    // ============================================================================
    // [OPUS-4.8] sq-b51o — per-source latency folded into the adaptive cost model.
    // ============================================================================

    // A 3-arm star on ?s where each arm's predicate is served by a DIFFERENT source, so a
    // per-source latency observation maps cleanly onto a per-pattern (per-arm) cost weight.
    //   pattern 0: ?s :a ?x  (source 0)
    //   pattern 1: ?s :b ?y  (source 1)
    //   pattern 2: ?s :c ?z  (source 2)
    fn three_source_star() -> (Bgp, [SourceDescriptor; 3]) {
        let bgp = Bgp::new(vec![
            TriplePattern::new(v("s"), iri("http://ex/a"), v("x")),
            TriplePattern::new(v("s"), iri("http://ex/b"), v("y")),
            TriplePattern::new(v("s"), iri("http://ex/c"), v("z")),
        ]);
        // Cards: :a = 10 (seed), :b = 100, :c = 200. Static suffix after :a is [:b, :c]
        // (smaller card first). One source per predicate ⇒ source i serves pattern i.
        let sa = SourceDescriptor::builder(SourceId::new("A"))
            .total_triples(100_000)
            .predicate(pred("http://ex/a", 10, 10, 10))
            .build();
        let sb = SourceDescriptor::builder(SourceId::new("B"))
            .total_triples(100_000)
            .predicate(pred("http://ex/b", 100, 100, 100))
            .build();
        let sc = SourceDescriptor::builder(SourceId::new("C"))
            .total_triples(100_000)
            .predicate(pred("http://ex/c", 200, 200, 200))
            .build();
        (bgp, [sa, sb, sc])
    }

    // ---- LATENCY DECISION: with cardinalities held EXACTLY at their estimates (no
    //      cardinality divergence at all), a slow source on the cheaper-by-card arm makes
    //      the planner DEFER it — the chosen suffix order flips purely on latency.
    #[test]
    fn latency_defers_slow_source() {
        let (bgp, srcs) = three_source_star();
        let sel = select_sources(&bgp, &srcs);
        let plan = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();

        // Baseline (no latency observed): suffix after the :a seed is [:b, :c] by card.
        let mut exec = AdaptiveExecutor::new(
            &bgp,
            &srcs,
            &sel,
            &plan,
            PlanOptions::default(),
            ReplanPolicy::default(),
        );
        assert_eq!(exec.advance().unwrap(), 0, ":a is the seed");
        assert_eq!(
            exec.remaining_order(),
            &[1, 2],
            "static-by-cardinality suffix is [:b, :c]"
        );

        // Now observe source 1 (serving :b) as VERY slow; cardinalities are spot-on (we
        // record them equal to the estimates, so the CARDINALITY trigger does NOT fire — only
        // the latency trigger does).
        let mut stats = RuntimeStats::new();
        stats.record_leaf_cardinality(1, 100.0); // == estimate ⇒ no card divergence.
        stats.record_leaf_cardinality(2, 200.0); // == estimate ⇒ no card divergence.
        stats.record_source_latency(1, 1000.0); // 10× baseline ⇒ slow.
        let outcome = exec.maybe_replan(&stats);
        assert_eq!(
            outcome,
            ReplanOutcome::Switched,
            "a slow source must trigger + switch even with cardinalities exactly as estimated"
        );
        assert_eq!(
            exec.remaining_order(),
            &[2, 1],
            "the slow :b arm is deferred behind the faster :c arm"
        );
    }

    // ---- RESULT-EQUIVALENCE across a LATENCY-driven reorder: the latency switch changes the
    //      order but the result multiset is identical to the static plan's (the load-bearing
    //      soundness invariant, exercised on the latency path specifically — mirrors
    //      replan_result_equals_static but the reorder is driven by latency, not cardinality).
    #[test]
    fn latency_replan_result_equals_static() {
        let (bgp, srcs) = three_source_star();
        // Concrete per-pattern solution multisets (already-fetched), non-trivial join.
        let solutions: SolutionSets = vec![
            // pattern 0: ?s :a ?x
            vec![
                t(&[("s", "s1"), ("x", "x1")]),
                t(&[("s", "s2"), ("x", "x2")]),
                t(&[("s", "s3"), ("x", "x3")]),
            ],
            // pattern 1: ?s :b ?y  (s3 absent ⇒ dropped; s1 fans out)
            vec![
                t(&[("s", "s1"), ("y", "y1")]),
                t(&[("s", "s1"), ("y", "y7")]),
                t(&[("s", "s2"), ("y", "y2")]),
            ],
            // pattern 2: ?s :c ?z  (s2 absent ⇒ dropped)
            vec![
                t(&[("s", "s1"), ("z", "z1")]),
                t(&[("s", "s3"), ("z", "z3")]),
            ],
        ];
        let sel = select_sources(&bgp, &srcs);
        let plan = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        let static_order = plan.join_order();
        let static_result = evaluate(&bgp, &solutions, &static_order);

        let mut exec = AdaptiveExecutor::new(
            &bgp,
            &srcs,
            &sel,
            &plan,
            PlanOptions::default(),
            ReplanPolicy::default(),
        );
        let _seed = exec.advance().unwrap();
        let mut stats = RuntimeStats::new();
        stats.record_source_latency(1, 1000.0); // slow :b ⇒ deferred (latency-only reorder).
        assert_eq!(
            exec.maybe_replan(&stats),
            ReplanOutcome::Switched,
            "fixture is designed to switch on latency — equivalence tested across a real reorder"
        );
        let mut adaptive_order = exec.executed_order().to_vec();
        adaptive_order.extend_from_slice(exec.remaining_order());
        assert_ne!(
            adaptive_order, static_order,
            "the latency re-plan must actually change the order here (non-vacuous)"
        );
        // Same pattern SET (no pattern added/dropped) …
        let mut a_sorted = adaptive_order.clone();
        let mut s_sorted = static_order.clone();
        a_sorted.sort();
        s_sorted.sort();
        assert_eq!(
            a_sorted, s_sorted,
            "latency re-plan preserves the pattern SET"
        );
        // … and the SAME result multiset despite the latency-driven order change.
        let adaptive_result = evaluate(&bgp, &solutions, &adaptive_order);
        assert!(
            multiset_eq(&static_result, &adaptive_result),
            "latency-reordered result MUST equal the static plan's result multiset"
        );
        assert!(!static_result.is_empty(), "fixture must produce some rows");
    }

    // ---- NO-THRASH on noisy-but-stable latency: latencies that jitter AROUND the baseline
    //      (never far enough to clear divergence_factor× and whose cost effect is below the
    //      hysteresis margin) must NOT trigger a switch — the latency term is anti-thrashed by
    //      the same discipline as the cardinality term.
    #[test]
    fn no_thrash_on_noisy_stable_latency() {
        let (bgp, srcs) = three_source_star();
        let sel = select_sources(&bgp, &srcs);
        let plan = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        let mut exec = AdaptiveExecutor::new(
            &bgp,
            &srcs,
            &sel,
            &plan,
            PlanOptions::default(),
            ReplanPolicy::default(),
        );
        let _ = exec.advance(); // execute :a seed.
                                // Latencies wobble around the 100.0 baseline (90/130/110) — noise, not a slow source:
                                // none reaches divergence_factor× (4× = 400) so the latency trigger stays silent.
        let mut stats = RuntimeStats::new();
        stats.record_source_latency(0, 90.0);
        stats.record_source_latency(1, 130.0);
        stats.record_source_latency(2, 110.0);
        assert_eq!(
            exec.maybe_replan(&stats),
            ReplanOutcome::NoDivergence,
            "noisy-but-stable latency around baseline must not trigger a re-plan"
        );
        assert_eq!(
            exec.remaining_order(),
            &[1, 2],
            "suffix order untouched by latency jitter"
        );
    }

    // ---- latency_weight = 0 fully disables the latency term (pure cardinality planning):
    //      even a wildly slow source neither triggers nor reorders.
    #[test]
    fn latency_weight_zero_is_inert() {
        let (bgp, srcs) = three_source_star();
        let sel = select_sources(&bgp, &srcs);
        let plan = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        let policy = ReplanPolicy {
            latency_weight: 0.0,
            ..ReplanPolicy::default()
        };
        let mut exec =
            AdaptiveExecutor::new(&bgp, &srcs, &sel, &plan, PlanOptions::default(), policy);
        let _ = exec.advance();
        let mut stats = RuntimeStats::new();
        stats.record_source_latency(1, 100_000.0); // absurdly slow, but weighting is off.
        assert_eq!(
            exec.maybe_replan(&stats),
            ReplanOutcome::NoDivergence,
            "latency_weight = 0 ⇒ latency is ignored entirely"
        );
        assert_eq!(exec.remaining_order(), &[1, 2]);
    }

    // ---- The latency factor itself: baseline ⇒ 1.0, slow ⇒ > 1 (capped), fast ⇒ < 1
    //      (floored), and weight 0 ⇒ always 1.0. Pins the documented heuristic constants.
    #[test]
    fn latency_factor_constants() {
        let p = ReplanPolicy::default(); // weight 0.5, baseline 100, floor 0.5, cap 4.0.
        assert_eq!(p.latency_factor(100.0), 1.0, "at baseline ⇒ neutral");
        assert_eq!(
            p.latency_factor(200.0),
            1.5,
            "2× baseline ⇒ 1 + 0.5·(2-1) = 1.5"
        );
        assert_eq!(
            p.latency_factor(1000.0),
            4.0,
            "10× baseline ⇒ clamped to the cap 4.0"
        );
        assert_eq!(
            p.latency_factor(0.0),
            0.5,
            "instant ⇒ clamped to the floor 0.5"
        );
        let off = ReplanPolicy {
            latency_weight: 0.0,
            ..ReplanPolicy::default()
        };
        assert_eq!(off.latency_factor(9999.0), 1.0, "weight 0 ⇒ always neutral");
    }

    // ============================================================================
    // [OPUS-4.8] sq-b51o follow-up — per-source latency EWMA smoothing.
    // ============================================================================

    // ---- EWMA mechanics + the default α are pinned. First sample seeds the average; each
    //      subsequent sample folds in at α = 0.3 (latest 30%, history 70%).
    #[test]
    fn ewma_smoothing_constants() {
        assert_eq!(
            RuntimeStats::DEFAULT_LATENCY_ALPHA, 0.3,
            "default latency EWMA α is the documented 0.3 heuristic"
        );
        let mut stats = RuntimeStats::new();
        assert_eq!(stats.latency_alpha(), 0.3);
        // First sample SEEDS the average (no history to decay).
        stats.record_source_latency(0, 100.0);
        assert_eq!(stats.observed_latency_of(0), Some(100.0), "first sample seeds EWMA");
        // Second sample: 0.3·1000 + 0.7·100 = 300 + 70 = 370.
        stats.record_source_latency(0, 1000.0);
        assert_eq!(
            stats.observed_latency_of(0),
            Some(370.0),
            "EWMA = α·observed + (1-α)·prev = 0.3·1000 + 0.7·100"
        );
        // Third sample at the same high level: 0.3·1000 + 0.7·370 = 300 + 259 = 559.
        stats.record_source_latency(0, 1000.0);
        assert_eq!(
            stats.observed_latency_of(0),
            Some(559.0),
            "EWMA converges toward a sustained level across samples"
        );
        // α = 1.0 recovers the old "single last sample" behaviour (no smoothing).
        let mut sharp = RuntimeStats::with_latency_alpha(1.0);
        sharp.record_source_latency(0, 100.0);
        sharp.record_source_latency(0, 1000.0);
        assert_eq!(sharp.observed_latency_of(0), Some(1000.0), "α=1 ⇒ no smoothing");
        // α is clamped into (0, 1]: a degenerate 0.0 / negative request never freezes the seed.
        assert!(RuntimeStats::with_latency_alpha(0.0).latency_alpha() > 0.0);
        assert_eq!(RuntimeStats::with_latency_alpha(5.0).latency_alpha(), 1.0);
    }

    // ---- ANTI-THRASH: a single TRANSIENT latency spike, after the source has an established
    //      (baseline) EWMA history, does NOT move the smoothed latency past the trigger — so no
    //      re-plan fires. This is the headline EWMA property the follow-up buys over the bare
    //      single-sample-plus-clamp.
    #[test]
    fn transient_latency_spike_does_not_trigger_replan() {
        let (bgp, srcs) = three_source_star();
        let sel = select_sources(&bgp, &srcs);
        let plan = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        let mut exec = AdaptiveExecutor::new(
            &bgp,
            &srcs,
            &sel,
            &plan,
            PlanOptions::default(),
            ReplanPolicy::default(),
        );
        let _ = exec.advance(); // execute :a seed.

        // Source 1 (serving :b) has been running at the baseline for a while: seed + several
        // at-baseline samples ⇒ EWMA sits at 100.0.
        let mut stats = RuntimeStats::new();
        for _ in 0..5 {
            stats.record_source_latency(1, 100.0);
        }
        // One transient spike of 1000.0 (10× baseline — which, as a RAW sample, would clear the
        // 4× divergence trigger). Under EWMA it lifts the average only to 0.3·1000 + 0.7·100 =
        // 370 < 400 (= divergence_factor × baseline), so the trigger stays silent.
        stats.record_source_latency(1, 1000.0);
        assert!(
            stats.observed_latency_of(1).unwrap() < 400.0,
            "one spike must not push the EWMA past the 4× trigger band"
        );
        assert_eq!(
            exec.maybe_replan(&stats),
            ReplanOutcome::NoDivergence,
            "a single transient latency spike (damped by the EWMA) must NOT trigger a re-plan"
        );
        assert_eq!(
            exec.remaining_order(),
            &[1, 2],
            "suffix order untouched by a transient spike"
        );
    }

    // ---- SUSTAINED SHIFT: the SAME source held at the SAME high latency for several
    //      consecutive samples DOES converge the EWMA past the trigger and re-plans — the EWMA
    //      reacts to a real regime change, it just refuses to react to a one-off.
    #[test]
    fn sustained_latency_shift_does_trigger_replan() {
        let (bgp, srcs) = three_source_star();
        let sel = select_sources(&bgp, &srcs);
        let plan = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        let mut exec = AdaptiveExecutor::new(
            &bgp,
            &srcs,
            &sel,
            &plan,
            PlanOptions::default(),
            ReplanPolicy::default(),
        );
        let _ = exec.advance(); // execute :a seed.

        // Same starting history as the transient test: source 1 established at baseline …
        let mut stats = RuntimeStats::new();
        for _ in 0..5 {
            stats.record_source_latency(1, 100.0);
        }
        // … then a SUSTAINED jump to 1000.0 for several consecutive samples. The EWMA climbs
        // 370 → 559 → 691 → 784 … crossing 400 (= 4× baseline) by the 2nd sustained sample.
        stats.record_source_latency(1, 1000.0); // 370 — not yet over.
        stats.record_source_latency(1, 1000.0); // 559 — over the 400 trigger band.
        assert!(
            stats.observed_latency_of(1).unwrap() > 400.0,
            "a sustained shift must converge the EWMA past the trigger band"
        );
        let outcome = exec.maybe_replan(&stats);
        assert_eq!(
            outcome,
            ReplanOutcome::Switched,
            "a sustained latency shift (EWMA converged past the trigger) MUST re-plan + switch"
        );
        assert_eq!(
            exec.remaining_order(),
            &[2, 1],
            "the sustainedly-slow :b arm is deferred behind the faster :c arm"
        );
    }

    // ---- RESULT-EQUIVALENCE under an EWMA-driven (sustained-latency) reorder: the smoothed
    //      latency path is still a pure suffix permutation — same result multiset as static.
    //      Mirrors latency_replan_result_equals_static but drives the switch via the EWMA
    //      (multiple samples), not a single raw value.
    #[test]
    fn ewma_replan_result_equals_static() {
        let (bgp, srcs) = three_source_star();
        let solutions: SolutionSets = vec![
            vec![
                t(&[("s", "s1"), ("x", "x1")]),
                t(&[("s", "s2"), ("x", "x2")]),
                t(&[("s", "s3"), ("x", "x3")]),
            ],
            vec![
                t(&[("s", "s1"), ("y", "y1")]),
                t(&[("s", "s1"), ("y", "y7")]),
                t(&[("s", "s2"), ("y", "y2")]),
            ],
            vec![
                t(&[("s", "s1"), ("z", "z1")]),
                t(&[("s", "s3"), ("z", "z3")]),
            ],
        ];
        let sel = select_sources(&bgp, &srcs);
        let plan = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        let static_order = plan.join_order();
        let static_result = evaluate(&bgp, &solutions, &static_order);

        let mut exec = AdaptiveExecutor::new(
            &bgp,
            &srcs,
            &sel,
            &plan,
            PlanOptions::default(),
            ReplanPolicy::default(),
        );
        let _seed = exec.advance().unwrap();
        // Sustained slow :b, accumulated via the EWMA (several samples), converging past trigger.
        let mut stats = RuntimeStats::new();
        for _ in 0..6 {
            stats.record_source_latency(1, 1000.0);
        }
        assert_eq!(
            exec.maybe_replan(&stats),
            ReplanOutcome::Switched,
            "fixture is designed to switch on the EWMA-smoothed latency"
        );
        let mut adaptive_order = exec.executed_order().to_vec();
        adaptive_order.extend_from_slice(exec.remaining_order());
        assert_ne!(
            adaptive_order, static_order,
            "the EWMA-driven re-plan must actually change the order (non-vacuous)"
        );
        let mut a_sorted = adaptive_order.clone();
        let mut s_sorted = static_order.clone();
        a_sorted.sort();
        s_sorted.sort();
        assert_eq!(a_sorted, s_sorted, "EWMA re-plan preserves the pattern SET");
        let adaptive_result = evaluate(&bgp, &solutions, &adaptive_order);
        assert!(
            multiset_eq(&static_result, &adaptive_result),
            "EWMA-reordered result MUST equal the static plan's result multiset"
        );
        assert!(!static_result.is_empty(), "fixture must produce some rows");
    }
}
