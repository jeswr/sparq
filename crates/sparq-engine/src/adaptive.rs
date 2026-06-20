//! **Local adaptive divergence-triggered re-planning** (opt-in `adaptive-replan-local`).
//!
//! [OPUS-4.8] sq-p6p6. The port of `sparq-fedplan`'s `adaptive-replan` (`ReplanPolicy` +
//! `AdaptiveExecutor`) to the LOCAL evaluator's greedy BGP join loop in
//! [`crate::exec`]`::eval_bgp_binary`.
//!
//! ## Why the local path gets this for free
//!
//! The static greedy planner (GOO: `goo_seed` → `goo_pick`) scores each candidate join by
//! `cur_card · |P| · sel`, where the selectivity `sel = Π 1/max(ndv)` over the shared
//! variables uses **estimated** per-variable distinct-value counts (`var_ndv`, derived from
//! the per-predicate characteristic stats under an INDEPENDENCE assumption). Real data is
//! **skewed / correlated**, so that estimated ndv — and hence the chosen suffix order — can
//! be badly wrong.
//!
//! But the local evaluator **fully materialises** each intermediate join, so after a stage
//! completes the executor holds the **true** result and can read off, for *nothing extra*,
//! both the **true running cardinality** (`result.rows.len()`) and the **observed
//! distinct-value count of every bound variable** (the real `var_ndv`). The adaptive loop
//! uses the divergence between the *estimated* and the *true* running cardinality as the
//! **trigger**, and — when it fires — substitutes the observed ndvs into the selectivity and
//! re-derives the remaining picks (the **correction**), adopting the corrected pick only when
//! the shared hysteresis margin says it is a clear win.
//!
//! This is exactly the federated semantics (`corrected_selection` overrides the estimate with
//! the observation and re-plans the remaining patterns), specialised to the case where the
//! observation is free.
//!
//! ## Trigger + hysteresis (the SHARED policy)
//!
//! The divergence trigger and the adoption hysteresis are the **shared**
//! [`sparq_replan_policy`] crate — the very same `ReplanPolicy::{divergence_factor,
//! improvement_margin}` rules the federated planner uses, so the two re-planners cannot drift
//! apart on the decision logic:
//!
//! 1. **Trigger** — after a stage materialises, compare the planner's *estimated* running
//!    cardinality `new_card` against the *true* materialised `result.rows.len()`. If they
//!    diverge by more than `divergence_factor` (`o > k·e` or `e > k·o`), a re-plan of the
//!    remaining patterns is *considered* (only when ≥ 2 patterns remain — fewer is nothing to
//!    reorder, and bounded by `max_replans`).
//! 2. **Re-plan** — re-pick the next pattern with the **observed** per-variable ndvs (read
//!    off the materialised result) substituted into the selectivity. This reuses the EXISTING
//!    `goo_pick` cost model verbatim — no new cost model, no new operators.
//! 3. **Hysteresis** — adopt the corrected pick only when its estimated output beats the
//!    estimate-path pick by more than `improvement_margin` (default 10%). A tie or a
//!    sub-margin win keeps the estimate-path pick (anti-thrash).
//!
//! ## Soundness (load-bearing — see [`crate::exec`] `adaptive_oracle` tests)
//!
//! A BGP answer is the natural join of its per-pattern solution multisets, which is
//! **commutative and associative**: any join order over the same pattern set yields the
//! **same** result multiset. The adaptive path only ever changes the *order* in which the
//! remaining patterns are joined (and re-uses the same multiset-preserving merge / hash / bind
//! join operators the static loop uses) — it never adds, drops, or alters a pattern, and the
//! already-materialised prefix is carried across the switch unchanged. Hence the adaptive
//! result is **bit-for-bit the same multiset** as the static plan, irrespective of re-order.
//! That invariant is the gate, proved by the engine's `replan_result_equals_static` oracle.

use oxrdf::Variable;
use rustc_hash::{FxHashMap, FxHashSet};
use sparq_core::dict::{Id, NO_ID};
use sparq_replan_policy::ReplanPolicy;

// A thread-local toggle (`ADAPTIVE_ENABLED`) lets the differential ORACLE and the indicative
// micro-BENCHMARK drive the STATIC plan and the ADAPTIVE plan from the *same* feature-on build —
// there is no other way to obtain a true static baseline once the adaptive code is compiled in.
// `SWITCHES` counts adopted re-orders (test-only) so the oracle can assert it is non-vacuous.
//
// The toggle defaults to ENABLED, so the production feature-on path is unchanged: one thread-local
// `bool` read per BGP stage boundary (a coarse site, never a per-row path) — the same cost the
// existing `zk` seam pays. It is exposed ONLY through the feature-gated `bench` shim
// ([`crate::adaptive_bench`]); the static-plan path is never reachable from the public query API.
thread_local! {
    static ADAPTIVE_ENABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    static SWITCHES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Whether the local adaptive re-planner is enabled on this thread (default `true`). The toggle
/// exists only for the oracle + bench baseline (see [`with_static_plan`]); the public query API
/// never flips it, so production always runs adaptive.
#[inline]
pub(crate) fn enabled() -> bool {
    ADAPTIVE_ENABLED.with(|c| c.get())
}

/// Run `f` with the adaptive re-planner forced OFF (the static baseline), then restore. Used by
/// the `replan_result_equals_static` oracle and the indicative micro-benchmark to obtain a true
/// static result from the same feature-on build.
pub(crate) fn with_static_plan<R>(f: impl FnOnce() -> R) -> R {
    ADAPTIVE_ENABLED.with(|c| {
        let prev = c.replace(false);
        let r = f();
        c.set(prev);
        r
    })
}

/// Record that an adaptive re-order was adopted (called from the join loop). Used by the oracle +
/// bench to confirm the fixture exercised a *real* switch (non-vacuous). One thread-local increment
/// only when a switch is actually adopted (a rare, coarse event), so production cost is nil.
#[inline]
pub(crate) fn note_switch() {
    SWITCHES.with(|c| c.set(c.get() + 1));
}

/// Run `f`, returning its result and how many re-plan switches it adopted, so the oracle can assert
/// the fixture exercised a *real* switch (non-vacuous) and the bench can report it.
pub(crate) fn count_switches<R>(f: impl FnOnce() -> R) -> (R, usize) {
    SWITCHES.with(|c| c.set(0));
    let r = f();
    (r, SWITCHES.with(|c| c.get()))
}

/// The mutable adaptive-re-plan state threaded through one BGP join loop.
///
/// Holds the shared [`ReplanPolicy`] and the remaining re-plan budget. Cheap to construct
/// (`Default::default()` + a counter); when the `adaptive-replan-local` feature is off this
/// type — and every call site in [`crate::exec`] — is `#[cfg]`-compiled out, so the static
/// loop is byte-identical to before.
pub(crate) struct LocalReplan {
    policy: ReplanPolicy,
    replans_left: usize,
}

impl LocalReplan {
    /// A fresh re-plan state with the default shared policy (`divergence_factor = 4`,
    /// `improvement_margin = 0.1`, `max_replans = 8`).
    pub(crate) fn new() -> LocalReplan {
        let policy = ReplanPolicy::default();
        LocalReplan { replans_left: policy.max_replans, policy }
    }

    /// Whether the planner's estimated running cardinality `estimate` diverges from the true
    /// materialised `observed` past the shared policy factor — the re-plan TRIGGER. (Thin
    /// wrapper over [`ReplanPolicy::diverges`].)
    #[inline]
    pub(crate) fn diverges(&self, estimate: f64, observed: f64) -> bool {
        self.policy.diverges(estimate, observed)
    }

    /// At a **stage boundary**, consume one unit of re-plan budget and report whether a re-plan
    /// is permitted: budget remains AND `remaining ≥ 2` patterns are still unjoined (fewer is
    /// nothing to reorder). The caller has already established that the running cardinality
    /// diverged (via [`Self::diverges`]); this only meters the budget. Returns `true` when the
    /// corrected re-pick may be *attempted* (the hysteresis gate [`Self::should_adopt`] still
    /// decides whether it is *taken*).
    pub(crate) fn try_consume_budget(&mut self, remaining: usize) -> bool {
        if remaining < 2 || self.replans_left == 0 {
            return false;
        }
        self.replans_left -= 1;
        true
    }

    /// The hysteresis adoption gate: with the budget already consumed by
    /// [`Self::try_consume_budget`], adopt the re-planned (corrected) pick — whose estimated
    /// output is `corrected_out` — over the estimate-path pick (`baseline_out`) **only** when
    /// it wins by more than the policy's `improvement_margin`. A tie or a sub-margin win keeps
    /// the estimate-path pick (anti-thrash). Mirrors the federated adoption rule exactly.
    #[inline]
    pub(crate) fn should_adopt(&self, baseline_out: f64, corrected_out: f64) -> bool {
        self.policy.adopt(baseline_out, corrected_out)
    }
}

/// The **true** number of distinct bound values of each variable in the materialised result,
/// read off for free from the columns of the just-completed join. `NO_ID` (unbound) cells are
/// excluded — a variable that is unbound in every row contributes no observation.
///
/// This is the observed-statistics correction the adaptive loop substitutes for the planner's
/// independence-model `var_ndv` estimate when the trigger fires: the column of a materialised
/// `Bindings` literally *is* the set of values that variable took, so its distinct count is
/// the real selectivity that subsequent joins against it will see — no estimate, no
/// independence assumption.
pub(crate) fn observed_var_ndv(vars: &[Variable], rows: &[impl AsRef<[Id]>]) -> FxHashMap<Variable, f64> {
    let mut out: FxHashMap<Variable, f64> = FxHashMap::default();
    if vars.is_empty() {
        return out;
    }
    let mut seen: Vec<FxHashSet<Id>> = vec![FxHashSet::default(); vars.len()];
    for row in rows {
        let cells = row.as_ref();
        for (col, set) in seen.iter_mut().enumerate() {
            if let Some(&id) = cells.get(col) {
                if id != NO_ID {
                    set.insert(id);
                }
            }
        }
    }
    for (v, set) in vars.iter().zip(seen.iter()) {
        // A bound variable with at least one observed value contributes its real distinct
        // count (floored at 1.0 — a single distinct value is still a valid join key).
        if !set.is_empty() {
            out.insert(v.clone(), (set.len() as f64).max(1.0));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_shared_policy_defaults() {
        let r = LocalReplan::new();
        assert_eq!(r.policy.divergence_factor, 4.0);
        assert_eq!(r.policy.improvement_margin, 0.1);
        assert_eq!(r.replans_left, 8);
    }

    #[test]
    fn budget_is_bounded() {
        let mut r = LocalReplan::new();
        r.replans_left = 1;
        // First eligible stage consumes the budget…
        assert!(r.try_consume_budget(3));
        // …the next finds the budget exhausted.
        assert!(!r.try_consume_budget(3));
    }

    #[test]
    fn no_replan_with_fewer_than_two_remaining() {
        let mut r = LocalReplan::new();
        assert!(!r.try_consume_budget(1));
        assert!(!r.try_consume_budget(0));
        // Budget untouched.
        assert_eq!(r.replans_left, 8);
    }

    #[test]
    fn divergence_uses_shared_factor() {
        let r = LocalReplan::new();
        assert!(r.diverges(10.0, 5000.0), "500x fires");
        assert!(r.diverges(5000.0, 10.0), "1/500x fires");
        assert!(!r.diverges(1000.0, 1500.0), "1.5x is below 4x");
        assert!(!r.diverges(1000.0, 1000.0), "exact does not fire");
    }

    #[test]
    fn adoption_uses_shared_margin() {
        let r = LocalReplan::new();
        assert!(r.should_adopt(100.0, 80.0), "20% cheaper adopted");
        assert!(!r.should_adopt(100.0, 95.0), "5% cheaper rejected (in band)");
        assert!(!r.should_adopt(100.0, 100.0), "tie rejected");
    }

    #[test]
    fn observed_ndv_counts_distinct_bound_values() {
        let a = Variable::new_unchecked("a");
        let b = Variable::new_unchecked("b");
        let vars = vec![a.clone(), b.clone()];
        // Column a: ids {1,1,2} → 2 distinct. Column b: {7, NO_ID, 7} → 1 distinct.
        let rows: Vec<Vec<Id>> = vec![vec![1, 7], vec![1, NO_ID], vec![2, 7]];
        let ndv = observed_var_ndv(&vars, &rows);
        assert_eq!(ndv.get(&a), Some(&2.0));
        assert_eq!(ndv.get(&b), Some(&1.0));
    }

    #[test]
    fn observed_ndv_skips_all_unbound_column() {
        let a = Variable::new_unchecked("a");
        let vars = vec![a.clone()];
        let rows: Vec<Vec<Id>> = vec![vec![NO_ID], vec![NO_ID]];
        let ndv = observed_var_ndv(&vars, &rows);
        // An all-unbound column yields no observation (no spurious selectivity correction).
        assert!(!ndv.contains_key(&a));
    }

    // ===== The differential ORACLE: adaptive result == static result (the GATE) =====

    use crate::exec::adaptive_oracle::{bgp_binary_with_replan, eval_bgp_static, multiset_rows_eq};
    use sparq_core::Graph;

    /// A graph + query that exercises the mis-estimated-multi-pattern case the local re-planner
    /// targets: a selective anchor binds `?x`, then a fan-out join EXPLODES the running cardinality
    /// far past the planner's independence-model estimate (the divergence trigger, `o >> e`), while
    /// TWO arms — one on `?x`, one on the exploded `?y` — still remain. The explosion is observed in
    /// the materialised result, so `?y`'s TRUE distinct-value count (large) replaces its
    /// tiny capped estimate; under that correction the cheaper remaining arm FLIPS from the `?x`-arm
    /// (estimate-path) to the `?y`-arm. A real suffix re-order — with the result multiset unchanged.
    ///
    /// Shape: `?w :anchor ?x . ?x :fan ?y . ?x :armA ?a . ?y :armB ?b`.
    fn divergent_graph() -> Graph {
        let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
        // anchor: a tiny selective seed binding ?x to the 3 HOT x. GOO seeds here (smallest est).
        for x in 0..3 {
            ttl.push_str(&format!("ex:w{} ex:anchor ex:hx{} .\n", x, x));
        }
        // fan: each hot x fans to 40 y (the explosion AFTER the anchor join -> divergence with both
        // arms still remaining). Many cold x with 1 y inflate :fan's est/ndv so the anchor<->fan
        // estimate caps ?y's running ndv far below its TRUE post-explosion value.
        for x in 0..3 {
            for y in 0..40 {
                ttl.push_str(&format!("ex:hx{} ex:fan ex:y{}_{} .\n", x, x, y));
            }
        }
        for x in 0..1000 {
            ttl.push_str(&format!("ex:cx{} ex:fan ex:cy{} .\n", x, x));
        }
        // armA on ?x: 3 hot x, 20 objects each -> est ~60, ndv_subj 3. Under the ESTIMATE it is the
        // cheaper remaining arm (so the STATIC plan joins it next).
        for x in 0..3 {
            for a in 0..20 {
                ttl.push_str(&format!("ex:hx{} ex:armA ex:a{}_{} .\n", x, x, a));
            }
        }
        // armB on ?y: FEW distinct y-subjects (10) each with MANY objects (60) -> large est (~600),
        // ndv_subj ~10. Under the estimate (?y's running ndv capped tiny) it looks EXPENSIVE; once
        // the explosion is observed (?y's true ndv ~120), it becomes the cheaper arm, so the
        // re-plan flips the suffix to join armB before armA.
        for y in 0..10 {
            for b in 0..60 {
                ttl.push_str(&format!("ex:y0_{} ex:armB ex:b{}_{} .\n", y, y, b));
            }
        }
        Graph::load_str(&ttl, "turtle").unwrap()
    }

    const DIVERGENT_Q: &str = "PREFIX ex: <http://ex/> SELECT * WHERE { \
        ?w ex:anchor ?x . ?x ex:fan ?y . ?x ex:armA ?a . ?y ex:armB ?b }";

    /// LOAD-BEARING ORACLE: the adaptive (re-planned) execution yields EXACTLY the static plan's
    /// result multiset, on a fixture that actually exercises a re-plan switch. This is the gate
    /// the bead requires.
    #[test]
    fn replan_result_equals_static() {
        let g = divergent_graph();
        let static_res = eval_bgp_static(&g, DIVERGENT_Q);
        let (adaptive_res, switched) = bgp_binary_with_replan(&g, DIVERGENT_Q);
        assert!(
            switched,
            "fixture must exercise a REAL re-plan switch (else the oracle is vacuous)"
        );
        assert!(!static_res.rows.is_empty(), "fixture must produce some rows");
        assert!(
            multiset_rows_eq(&static_res, &adaptive_res),
            "adaptive (re-planned) result MUST equal the static plan's result multiset"
        );
    }

    /// Result-equivalence when the estimate is RIGHT (no divergence): the adaptive path is a
    /// pure no-op — same rows, no spurious switch.
    #[test]
    fn no_divergence_is_identical_and_no_switch() {
        // A 1:1 chain whose estimates are accurate: ?s :a ?o . ?o :b ?z.
        let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
        for i in 0..20 {
            ttl.push_str(&format!("ex:s{} ex:a ex:o{} .\n", i, i));
            ttl.push_str(&format!("ex:o{} ex:b ex:z{} .\n", i, i));
        }
        let g = Graph::load_str(&ttl, "turtle").unwrap();
        let q = "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:a ?o . ?o ex:b ?z }";
        let static_res = eval_bgp_static(&g, q);
        let (adaptive_res, switched) = bgp_binary_with_replan(&g, q);
        assert!(!switched, "accurate estimates must not trigger a switch");
        assert!(
            multiset_rows_eq(&static_res, &adaptive_res),
            "no-divergence adaptive result equals static"
        );
    }

    /// Result-equivalence on a range of unrelated multi-pattern shapes — a broad fuzz-ish check
    /// that the adaptive path never alters the answer, switch or no switch.
    #[test]
    fn adaptive_equals_static_across_shapes() {
        let g = divergent_graph();
        let queries = [
            // the divergent 4-pattern shape itself (the switch case)
            "PREFIX ex: <http://ex/> SELECT * WHERE { \
                ?w ex:anchor ?x . ?x ex:fan ?y . ?x ex:armA ?a . ?y ex:armB ?b }",
            // projected, reversed source order (planner must still converge to the same multiset)
            "PREFIX ex: <http://ex/> SELECT ?w ?b WHERE { \
                ?y ex:armB ?b . ?x ex:armA ?a . ?x ex:fan ?y . ?w ex:anchor ?x }",
            // a sub-shape that does NOT explode (anchor + a single arm)
            "PREFIX ex: <http://ex/> SELECT * WHERE { ?w ex:anchor ?x . ?x ex:armA ?a }",
        ];
        for q in queries {
            let static_res = eval_bgp_static(&g, q);
            let (adaptive_res, _switched) = bgp_binary_with_replan(&g, q);
            assert!(
                multiset_rows_eq(&static_res, &adaptive_res),
                "adaptive result equals static for query: {}",
                q
            );
        }
    }
}
