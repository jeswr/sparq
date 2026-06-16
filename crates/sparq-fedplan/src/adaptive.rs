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
}

impl Default for ReplanPolicy {
    fn default() -> Self {
        ReplanPolicy {
            divergence_factor: 4.0,
            improvement_margin: 0.1,
            max_replans: 8,
        }
    }
}

/// Observed runtime statistics, accumulated as execution proceeds. Distinct from the
/// descriptor-derived *estimates*: every value here is a real count / measurement the
/// executor fed in at a stage boundary.
#[derive(Debug, Clone, Default)]
pub struct RuntimeStats {
    /// Observed leaf cardinality per pattern index (rows the pattern actually returned,
    /// union over its sources). Present only for patterns that have been (at least
    /// partially) evaluated.
    observed_leaf_card: HashMap<usize, f64>,
    /// Observed per-source latency, in abstract time units (e.g. ms). Used by the
    /// trigger to flag slow/unresponsive sources; the planner does not yet fold latency
    /// into the cost model (deferred), but it is captured so the trigger can act on it.
    observed_latency: HashMap<usize, f64>,
}

impl RuntimeStats {
    /// A fresh, empty statistics store.
    pub fn new() -> RuntimeStats {
        RuntimeStats::default()
    }

    /// Record the *observed* cardinality of a pattern's leaf (the real row count its
    /// source(s) returned). Overwrites any prior observation for that pattern.
    pub fn record_leaf_cardinality(&mut self, pattern: usize, observed: f64) {
        self.observed_leaf_card.insert(pattern, observed.max(0.0));
    }

    /// Record the *observed* latency of a source (abstract time units).
    pub fn record_source_latency(&mut self, source: usize, latency: f64) {
        self.observed_latency.insert(source, latency.max(0.0));
    }

    /// The observed leaf cardinality for `pattern`, if recorded.
    pub fn observed_leaf(&self, pattern: usize) -> Option<f64> {
        self.observed_leaf_card.get(&pattern).copied()
    }

    /// The observed latency for `source`, if recorded.
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
    /// diverges from its estimate past the policy factor; if so, re-plans the **suffix**
    /// with the corrected statistics and adopts the new order only when it wins by the
    /// hysteresis margin. Returns the [`ReplanOutcome`]. The prefix is never touched.
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
        let triggered =
            self.remaining
                .iter()
                .any(|&p| match (base_est.get(&p), stats.observed_leaf(p)) {
                    (Some(&e), Some(o)) => diverges(e, o, &self.policy),
                    _ => false,
                });
        if !triggered {
            return ReplanOutcome::NoDivergence;
        }

        // ---- Re-plan the SUFFIX with corrected statistics.
        // Build a sub-BGP of the remaining patterns; plan over it with observed cardinalities.
        let corrected = corrected_selection(self.base_selection, stats);
        let Some(new_suffix) = self.replan_suffix(&corrected) else {
            return ReplanOutcome::KeptWithinHysteresis;
        };

        // ---- Hysteresis: only adopt if the new suffix beats the current one by the margin.
        let cur_cost = self.suffix_cost(&self.remaining, &corrected);
        let new_cost = self.suffix_cost(&new_suffix, &corrected);
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
    fn replan_suffix(&self, corrected: &[PatternSources]) -> Option<Vec<usize>> {
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
        ))
    }

    /// Estimated remaining cost of executing `suffix` (in this order) on top of the current
    /// prefix, under the corrected statistics — the comparable used for hysteresis.
    fn suffix_cost(&self, suffix: &[usize], corrected: &[PatternSources]) -> f64 {
        suffix_cost_greedy(
            self.bgp,
            &self.executed,
            suffix,
            corrected,
            self.descriptors,
            &self.opts,
        )
    }
}

/// Re-derives a greedy left-deep order for `suffix`, anchored on the already-joined
/// `prefix`. Mirrors [`crate::plan_bgp`]'s selection rule (connected-first, smallest output
/// next, tie-break on index) but over a fixed prefix — so the only thing that can change is
/// the order of the not-yet-executed patterns. Pure, deterministic.
fn plan_suffix_greedy(
    bgp: &Bgp,
    prefix: &[usize],
    suffix: &[usize],
    selection: &[PatternSources],
    descriptors: &[crate::descriptor::SourceDescriptor],
    opts: &PlanOptions,
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
        let mut best: Option<(usize, f64, bool)> = None; // (pat, out_card, connected)
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
            let better = match &best {
                None => true,
                Some((bp, bcard, bconn)) => match (connected, *bconn) {
                    (true, false) => true,
                    (false, true) => false,
                    _ => out < *bcard || (out == *bcard && cand < *bp),
                },
            };
            if better {
                best = Some((cand, out, connected));
            }
        }
        let (pat, out, _) = best.expect("rem non-empty");
        order.push(pat);
        joined.push(pat);
        cur_card = out;
        rem.retain(|&p| p != pat);
    }
    order
}

/// Estimated total remaining cost of joining `suffix` (in the given order) onto `prefix`.
fn suffix_cost_greedy(
    bgp: &Bgp,
    prefix: &[usize],
    suffix: &[usize],
    selection: &[PatternSources],
    descriptors: &[crate::descriptor::SourceDescriptor],
    opts: &PlanOptions,
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
        total += cost;
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
        };
        let mut exec =
            AdaptiveExecutor::new(&bgp, &srcs, &sel, &plan, PlanOptions::default(), policy);
        let _ = exec.advance();
        let mut stats = RuntimeStats::new();
        stats.record_leaf_cardinality(0, 5.0);
        assert_eq!(exec.maybe_replan(&stats), ReplanOutcome::BudgetExhausted);
    }

    // ---- Latency capture is recorded and readable (drives source-switch trigger, deferred).
    #[test]
    fn latency_is_captured() {
        let mut stats = RuntimeStats::new();
        stats.record_source_latency(3, 250.0);
        assert_eq!(stats.observed_latency_of(3), Some(250.0));
        assert_eq!(stats.observed_latency_of(0), None);
    }
}
