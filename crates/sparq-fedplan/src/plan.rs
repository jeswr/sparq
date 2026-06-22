//! **Join planning**: a greedy join-order heuristic over the selected sources, with a
//! per-join **bind-vs-hash** algorithm decision, producing a [`JoinTree`] of estimated
//! cardinalities and costs.
//!
//! ## Inputs
//!
//! The planner consumes the per-pattern source selection ([`crate::select_sources`]) —
//! each pattern's retained sources and their CostFed cardinality estimates — plus the
//! source descriptors (for characteristic-set star-join cardinality). It never touches
//! the network: every estimate comes from the descriptors.
//!
//! ## Join order (greedy, selectivity-first)
//!
//! Start from the lowest-cardinality pattern; repeatedly join the connected pattern
//! (sharing a variable with the result so far) whose join yields the smallest estimated
//! output. This is the standard greedy left-deep heuristic; it avoids Cartesian
//! products while any connected pattern remains, and falls back to the next smallest
//! disconnected pattern only when the result component is exhausted. Ties break on
//! pattern index for determinism.
//!
//! ## Intermediate-result cardinality
//!
//! When several joined patterns form a **star** (share a subject variable, with bound
//! predicates), the size of joining the next star arm is estimated from the
//! characteristic sets — Neumann & Moerkotte's `Σ_{C⊇Q} count(C)·Π avg_mult` — which
//! captures the predicate *correlation* a per-predicate-independence product loses
//! (CS estimates are taken on the source contributing the star; a multi-source star
//! sums per source). Non-star joins fall back to an independence estimate scaled by the
//! joined predicate's selectivity.
//!
//! ## Bind vs hash (the per-join algorithm decision)
//!
//! For a join of a left intermediate result (`L` rows) with a right pattern
//! (`R` rows total over its sources, fan-out `f` = expected matches per bound value):
//!
//! * **Bind join** — issue, per left row, a bound request to the right's sources
//!   (`?s p ?o` with `?s` bound). Cost ≈ `L · (req + f)`: one request per left row plus
//!   the matched rows. Wins when `L` is small and the right is selective per bound value
//!   (small `f`). This is the SERVICE-style nested-loop / VALUES-pushdown join.
//! * **Hash / symmetric join** — scan both sides once and hash-join locally. Cost ≈
//!   `R + L`: one full retrieval of each side. Wins when `L` is large (so per-row
//!   requests would dominate) or the right is unselective.
//!
//! The planner picks the cheaper of the two estimated costs; the per-request constant
//! `req` (the round-trip penalty that makes bind joins expensive at scale) is the knob
//! in [`PlanOptions::request_cost`]. The decision *flips* as `L` grows past the point
//! where `L·req` overtakes `R`.
//!
//! ## Streaming (sq-vf7q)
//!
//! A third [`JoinAlgo::Streaming`] choice marks a hash-class join for **non-blocking
//! streaming execution with operator spill** (the [`crate::StreamJoin`] operator). The
//! planner picks it over a plain [`JoinAlgo::Hash`] when both inputs are large enough
//! that materialising a side up front is the risk: the combined estimated input rows
//! exceed [`PlanOptions::stream_threshold`]. It is the same abstract cost as Hash (one
//! scan per side) — the difference is the execution discipline (emit-as-you-go, bounded
//! memory), not the cost. Setting the threshold to `f64::INFINITY` disables it.
//!
//! ## Adaptive re-planning (sq-7s4z)
//!
//! `plan_bgp` itself stays a **static** planner — it commits to one order from the
//! estimates it is given. Live **mid-execution re-planning** (re-invoking the planner on
//! the remaining patterns when observed cardinalities diverge) lives in [`crate::adaptive`]
//! behind the opt-in `adaptive-replan` feature, which re-uses this module's cost model via
//! the crate-internal `cost_join_pub`. Mid-*operator* swaps remain deferred.
//!
//! [OPUS-4.8] sq-a35t / sq-vf7q / sq-7s4z.

use crate::descriptor::SourceDescriptor;
use crate::pattern::{Bgp, Term, Var};
use crate::selection::PatternSources;

/// The join algorithm chosen for a binary join node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinAlgo {
    /// Nested-loop / bound-request (SERVICE / VALUES-pushdown) join — probe the right
    /// with the left's bindings. Cheap when the left is small + right selective.
    Bind,
    /// Hash / symmetric join — scan both sides once, join locally. Cheap when the left
    /// is large or the right unselective.
    Hash,
    /// ANAPSID-style **non-blocking streaming join with bounded operator spill** — the
    /// execution-side [`crate::StreamJoin`]. Chosen over a plain [`JoinAlgo::Hash`] when
    /// the combined inputs are large (past [`PlanOptions::stream_threshold`]): the
    /// streaming operator emits matches as federated sub-results arrive and spills
    /// over-budget partitions instead of OOMing, while producing the identical result
    /// multiset. Same abstract cost as Hash (one scan of each side); the distinction is
    /// the *execution discipline*, not the cost model. [OPUS-4.8] sq-vf7q.
    Streaming,
}

/// A node of the planned (left-deep) join tree.
#[derive(Debug, Clone)]
pub enum JoinNode {
    /// A leaf: one triple pattern, evaluated against its retained sources (a union).
    Leaf {
        /// Pattern index within the BGP.
        pattern: usize,
        /// Estimated rows (union over the pattern's retained sources).
        cardinality: f64,
    },
    /// A binary join of `left` with the leaf pattern `right`.
    Join {
        left: Box<JoinNode>,
        /// Pattern index of the right (leaf) operand.
        right: usize,
        /// Chosen join algorithm.
        algo: JoinAlgo,
        /// Estimated output rows of this join.
        cardinality: f64,
        /// Estimated cost of this join (in the planner's abstract cost units).
        cost: f64,
    },
}

impl JoinNode {
    /// The estimated output cardinality of this (sub)tree.
    pub fn cardinality(&self) -> f64 {
        match self {
            JoinNode::Leaf { cardinality, .. } => *cardinality,
            JoinNode::Join { cardinality, .. } => *cardinality,
        }
    }
}

/// The full planned join tree for a BGP.
#[derive(Debug, Clone)]
pub struct JoinTree {
    /// The root of the left-deep join tree.
    pub root: JoinNode,
    /// Total estimated cost (sum of every join node's cost).
    pub total_cost: f64,
}

impl JoinTree {
    /// The patterns in the order the planner joins them (left-deep traversal).
    pub fn join_order(&self) -> Vec<usize> {
        let mut order = Vec::new();
        collect_order(&self.root, &mut order);
        order
    }
}

fn collect_order(node: &JoinNode, order: &mut Vec<usize>) {
    match node {
        JoinNode::Leaf { pattern, .. } => order.push(*pattern),
        JoinNode::Join { left, right, .. } => {
            collect_order(left, order);
            order.push(*right);
        }
    }
}

/// Tuning knobs for [`plan_bgp`].
#[derive(Debug, Clone)]
pub struct PlanOptions {
    /// Abstract cost of one bound remote request in a bind join — the round-trip
    /// penalty that makes bind joins expensive at scale. Higher ⇒ the planner switches
    /// to hash joins at a smaller left cardinality. Default `1.0` (one request ≈ one
    /// retrieved row).
    pub request_cost: f64,
    /// [OPUS-4.8] sq-vf7q. Combined-input-rows threshold above which a hash-class join is
    /// marked [`JoinAlgo::Streaming`] (non-blocking + spill) instead of [`JoinAlgo::Hash`].
    /// When the estimated `L + R` of a hash join exceeds this, materialising a side up
    /// front is the latency/memory risk, so the streaming operator is preferred. Set to
    /// `f64::INFINITY` to always use plain hash. Default `100_000` rows.
    pub stream_threshold: f64,
}

impl Default for PlanOptions {
    fn default() -> Self {
        PlanOptions {
            request_cost: 1.0,
            stream_threshold: 100_000.0,
        }
    }
}

/// Plans the join over `bgp` given the per-pattern source `selection`
/// ([`crate::select_sources`]) and the source `descriptors`. Returns `None` when the
/// BGP is empty. Deterministic: greedy choices break ties on pattern index.
pub fn plan_bgp(
    bgp: &Bgp,
    selection: &[PatternSources],
    descriptors: &[SourceDescriptor],
    opts: &PlanOptions,
) -> Option<JoinTree> {
    if bgp.is_empty() || selection.is_empty() {
        return None;
    }
    // Per-pattern leaf cardinality (union over retained sources).
    let leaf_card: Vec<f64> = selection
        .iter()
        .map(|s| s.total_cardinality().max(0.0))
        .collect();

    // Greedy left-deep: seed with the smallest-cardinality pattern.
    let n = bgp.len();
    let mut remaining: Vec<usize> = (0..n).collect();
    let seed = *remaining
        .iter()
        .min_by(|&&a, &&b| {
            leaf_card[a]
                .partial_cmp(&leaf_card[b])
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.cmp(&b))
        })
        .expect("non-empty");
    remaining.retain(|&p| p != seed);

    let mut root = JoinNode::Leaf {
        pattern: seed,
        cardinality: leaf_card[seed],
    };
    let mut joined: Vec<usize> = vec![seed];

    while !remaining.is_empty() {
        // Prefer a CONNECTED pattern (shares a var with something already joined); only
        // if none is connected do we accept a Cartesian arm (smallest leaf).
        let cur_card = root.cardinality();
        let mut best: Option<(usize, f64, f64, JoinAlgo, bool)> = None; // (pat, card, cost, algo, connected)
        for &cand in &remaining {
            let connected = joined.iter().any(|&j| bgp.shares_var(j, cand));
            let (card, cost, algo) = cost_join(
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
                Some((_, _, _, _, best_conn)) => {
                    // Connected always beats disconnected; within the same class, lower
                    // output card wins; tie-break on pattern index for determinism.
                    match (connected, best_conn) {
                        (true, false) => true,
                        (false, true) => false,
                        _ => {
                            let (_, best_card, ..) = best.as_ref().unwrap();
                            card < *best_card
                                || (card == *best_card && cand < best.as_ref().unwrap().0)
                        }
                    }
                }
            };
            if better {
                best = Some((cand, card, cost, algo, connected));
            }
        }
        let (pat, card, cost, algo, _) = best.expect("remaining non-empty");
        root = JoinNode::Join {
            left: Box::new(root),
            right: pat,
            algo,
            cardinality: card,
            cost,
        };
        joined.push(pat);
        remaining.retain(|&p| p != pat);
    }

    let total_cost = sum_cost(&root);
    Some(JoinTree { root, total_cost })
}

/// [OPUS-4.8] sq-7s4z. Crate-internal re-export of [`cost_join`] so the adaptive
/// re-planner ([`crate::adaptive`]) costs a re-planned suffix with the **identical** cost
/// model the static planner uses — there is exactly one cost function, so a re-plan and the
/// original plan are always comparable on the same scale.
#[cfg(feature = "adaptive-replan")]
#[allow(clippy::too_many_arguments)]
pub(crate) fn cost_join_pub(
    bgp: &Bgp,
    cur_card: f64,
    joined: &[usize],
    cand: usize,
    leaf_card: &[f64],
    descriptors: &[SourceDescriptor],
    selection: &[PatternSources],
    opts: &PlanOptions,
    connected: bool,
) -> (f64, f64, JoinAlgo) {
    cost_join(
        bgp,
        cur_card,
        joined,
        cand,
        leaf_card,
        descriptors,
        selection,
        opts,
        connected,
    )
}

/// Estimates `(output_cardinality, cost, algo)` for joining leaf pattern `cand` onto a
/// left intermediate of `cur_card` rows.
#[allow(clippy::too_many_arguments)]
fn cost_join(
    bgp: &Bgp,
    cur_card: f64,
    joined: &[usize],
    cand: usize,
    leaf_card: &[f64],
    descriptors: &[SourceDescriptor],
    selection: &[PatternSources],
    opts: &PlanOptions,
    connected: bool,
) -> (f64, f64, JoinAlgo) {
    let r = leaf_card[cand].max(1.0); // right side total rows.
    let l = cur_card.max(1.0); // left intermediate rows.

    // ---- Output cardinality.
    let out = if connected {
        // Try a characteristic-set star estimate first (captures predicate correlation
        // when `cand` extends a star already joined on its subject variable).
        star_estimate(bgp, joined, cand, descriptors, selection)
            .unwrap_or_else(|| independence_estimate(bgp, cur_card, joined, cand, leaf_card))
    } else {
        // Disconnected (Cartesian): product. Only chosen when nothing connects.
        l * r
    };

    // ---- Bind vs hash.
    // Bind: one bound request per left row + the matched rows it returns.
    //   fan-out f = output / left  (avg matches per left row).
    let fan_out = (out / l).max(0.0);
    let bind_cost = l * (opts.request_cost + fan_out);
    // Hash/symmetric: one full retrieval of each side.
    let hash_cost = r + l;
    let (algo, cost) = if bind_cost <= hash_cost {
        (JoinAlgo::Bind, bind_cost)
    } else if l + r > opts.stream_threshold {
        // [OPUS-4.8] sq-vf7q: a large hash join runs as the non-blocking + spill operator.
        // Same abstract cost as plain hash; the choice is the execution discipline.
        (JoinAlgo::Streaming, hash_cost)
    } else {
        (JoinAlgo::Hash, hash_cost)
    };
    (out, cost, algo)
}

/// Characteristic-set star estimate for joining `cand` onto the star already joined on
/// `cand`'s subject variable. Returns `None` when `cand` is not a star arm (subject not
/// a variable, predicate not bound, or no covering characteristic set on any source).
fn star_estimate(
    bgp: &Bgp,
    joined: &[usize],
    cand: usize,
    descriptors: &[SourceDescriptor],
    selection: &[PatternSources],
) -> Option<f64> {
    let tp = &bgp.patterns[cand];
    let sv = star_subject_var(tp)?;
    let cand_pred = tp.predicate_iri()?.to_string();

    // The predicates already joined on this subject variable (the star Q).
    let mut q: Vec<String> = Vec::new();
    for &j in joined {
        let jtp = &bgp.patterns[j];
        if star_subject_var(jtp) == Some(sv) {
            if let Some(p) = jtp.predicate_iri() {
                if !q.contains(&p.to_string()) {
                    q.push(p.to_string());
                }
            }
        }
    }
    if q.is_empty() || q.contains(&cand_pred) {
        return None; // no prior star arm, or repeated predicate (CS model can't condition).
    }
    q.sort();
    let mut q_new = q.clone();
    let pos = q_new.binary_search(&cand_pred).unwrap_err();
    q_new.insert(pos, cand_pred);

    // Sum the CS estimate over the sources retained for `cand` (a multi-source star is a
    // union). card(Q∪{p}) is the joined output; we report it directly (it already
    // conditions on the whole star).
    let mut total = 0.0f64;
    let mut covered = false;
    for c in &selection[cand].candidates {
        let src = &descriptors[c.source];
        let card_q = src.star_cardinality(&q);
        if card_q <= 0.0 {
            continue;
        }
        covered = true;
        total += src.star_cardinality(&q_new);
    }
    covered.then_some(total)
}

/// The subject variable of a star arm `?s <p> ?o`/`?s <p> lit` — a variable subject with
/// a bound predicate. `None` otherwise.
fn star_subject_var(tp: &crate::pattern::TriplePattern) -> Option<&Var> {
    match (&tp.subject, &tp.predicate) {
        (Term::Var(v), Term::Iri(_)) => Some(v),
        _ => None,
    }
}

/// Independence-assumption output estimate (the non-star fallback): the left rows scaled
/// by the right pattern's per-join selectivity. We approximate the right's selectivity
/// as `min(1, distinct-binding fraction)` via its leaf cardinality and the joined
/// pattern's cardinality — a coarse but monotone estimate (smaller right ⇒ smaller out).
fn independence_estimate(
    bgp: &Bgp,
    cur_card: f64,
    joined: &[usize],
    cand: usize,
    leaf_card: &[f64],
) -> f64 {
    let l = cur_card.max(1.0);
    let r = leaf_card[cand].max(1.0);
    // If the right shares a variable, the expected matches per left row ≈ r / ndv, with
    // ndv approximated by the larger of the two joined leaf cardinalities (a standard
    // join-selectivity heuristic: out = l*r/max(|L|,|R|)). Bounded below by 1 result
    // when any match is possible, above by the Cartesian product.
    let any_left = joined.iter().map(|&j| leaf_card[j]).fold(1.0f64, f64::max);
    let ndv = any_left.max(r).max(1.0);
    let _ = bgp; // shape already validated by the caller's `connected` flag.
    (l * r / ndv).clamp(0.0, l * r)
}

fn sum_cost(node: &JoinNode) -> f64 {
    match node {
        JoinNode::Leaf { .. } => 0.0,
        JoinNode::Join { left, cost, .. } => cost + sum_cost(left),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::{CharSet, PredPartition, SourceId};
    use crate::pattern::{TriplePattern, Var};
    use crate::selection::select_sources;

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

    #[test]
    fn bind_vs_hash_flips_at_threshold() {
        // Single binary join; control L (left) and R (right) directly via leaf cards.
        // Bind cost = L*(req + out/L); Hash cost = R + L. With request_cost large, a big
        // L makes bind expensive ⇒ Hash; a small L makes bind cheap ⇒ Bind.
        // Build a 2-pattern star: ?s :p ?o (left), ?s :q ?z (right).
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), iri("http://ex/p"), var("o")),
            TriplePattern::new(var("s"), iri("http://ex/q"), var("z")),
        ]);
        // Source with both predicates; choose stats so the left is tiny or huge.
        let small_left = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(10_000)
            .predicate(pred("http://ex/p", 5, 5, 5)) // left leaf card = 5 (small)
            .predicate(pred("http://ex/q", 1000, 1000, 1000))
            .build();
        let srcs = [small_left];
        let sel = select_sources(&bgp, &srcs);
        let opts = PlanOptions {
            request_cost: 50.0,
            ..PlanOptions::default()
        };
        let tree = plan_bgp(&bgp, &sel, &srcs, &opts).unwrap();
        // The single join: left seed is the small pattern (:p, card 5). Bind should win.
        if let JoinNode::Join { algo, .. } = &tree.root {
            assert_eq!(*algo, JoinAlgo::Bind, "small left ⇒ bind join");
        } else {
            panic!("expected a join");
        }

        // Now make BOTH sides large so left intermediate is large ⇒ hash wins.
        let big = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(10_000)
            .predicate(pred("http://ex/p", 5000, 5000, 5000))
            .predicate(pred("http://ex/q", 5000, 5000, 5000))
            .build();
        let bigs = [big];
        let sel2 = select_sources(&bgp, &bigs);
        let tree2 = plan_bgp(&bgp, &sel2, &bigs, &opts).unwrap();
        if let JoinNode::Join { algo, .. } = &tree2.root {
            assert_eq!(*algo, JoinAlgo::Hash, "large left ⇒ hash join");
        } else {
            panic!("expected a join");
        }
    }

    #[test]
    fn large_hash_join_becomes_streaming() {
        // [OPUS-4.8] sq-vf7q. Both sides large + unselective ⇒ hash class; combined rows
        // past stream_threshold ⇒ the planner marks it Streaming (non-blocking + spill).
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), iri("http://ex/p"), var("o")),
            TriplePattern::new(var("s"), iri("http://ex/q"), var("z")),
        ]);
        let big = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(10_000_000)
            .predicate(pred("http://ex/p", 500_000, 500_000, 500_000))
            .predicate(pred("http://ex/q", 500_000, 500_000, 500_000))
            .build();
        let srcs = [big];
        let sel = select_sources(&bgp, &srcs);
        // Low stream_threshold so the large combined input trips streaming.
        let opts = PlanOptions {
            request_cost: 50.0,
            stream_threshold: 1000.0,
        };
        let tree = plan_bgp(&bgp, &sel, &srcs, &opts).unwrap();
        if let JoinNode::Join { algo, .. } = &tree.root {
            assert_eq!(*algo, JoinAlgo::Streaming, "large hash join ⇒ streaming");
        } else {
            panic!("expected a join");
        }
        // With the threshold raised above the inputs, the same join is plain Hash.
        let opts_hi = PlanOptions {
            request_cost: 50.0,
            stream_threshold: f64::INFINITY,
        };
        let tree_hi = plan_bgp(&bgp, &sel, &srcs, &opts_hi).unwrap();
        if let JoinNode::Join { algo, .. } = &tree_hi.root {
            assert_eq!(*algo, JoinAlgo::Hash, "threshold disabled ⇒ plain hash");
        } else {
            panic!("expected a join");
        }
    }

    #[test]
    fn join_order_starts_with_most_selective() {
        // Three patterns chained: ?s :p ?o . ?o :q ?z . ?z :r ?w .
        // Make :q the most selective leaf ⇒ it must be the seed (first in order).
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), iri("http://ex/p"), var("o")),
            TriplePattern::new(var("o"), iri("http://ex/q"), var("z")),
            TriplePattern::new(var("z"), iri("http://ex/r"), var("w")),
        ]);
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(100_000)
            .predicate(pred("http://ex/p", 1000, 1000, 1000))
            .predicate(pred("http://ex/q", 10, 10, 10)) // most selective.
            .predicate(pred("http://ex/r", 500, 500, 500))
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let tree = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        assert_eq!(
            tree.join_order()[0],
            1,
            "most-selective pattern :q seeds the plan"
        );
        // All three patterns appear exactly once.
        let mut order = tree.join_order();
        order.sort();
        assert_eq!(order, vec![0, 1, 2]);
    }

    #[test]
    fn star_join_uses_characteristic_sets() {
        // A 2-arm star ?s :a ?x . ?s :b ?y on a source whose CS says {a,b} co-occur on
        // 100 subjects (a 1x, b 2x) ⇒ joined card = 200, NOT the independence product.
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), iri("http://ex/a"), var("x")),
            TriplePattern::new(var("s"), iri("http://ex/b"), var("y")),
        ]);
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(10_000)
            .predicate(pred("http://ex/a", 100, 100, 100))
            .predicate(pred("http://ex/b", 200, 100, 100))
            .char_set(CharSet {
                predicates: vec!["http://ex/a".into(), "http://ex/b".into()],
                subjects: 100,
                avg_multiplicity: vec![1.0, 2.0],
            })
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let tree = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        // Root join output is the CS star cardinality 200 (100 subjects * 1 * 2).
        assert_eq!(
            tree.root.cardinality(),
            200.0,
            "star join uses CS correlation, not independence"
        );
    }

    #[test]
    fn empty_bgp_yields_no_plan() {
        let bgp = Bgp::new(vec![]);
        assert!(plan_bgp(&bgp, &[], &[], &PlanOptions::default()).is_none());
    }

    #[test]
    fn single_pattern_is_a_leaf() {
        let bgp = Bgp::new(vec![TriplePattern::new(
            var("s"),
            iri("http://ex/p"),
            var("o"),
        )]);
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .predicate(pred("http://ex/p", 42, 42, 42))
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let tree = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        assert!(matches!(tree.root, JoinNode::Leaf { pattern: 0, .. }));
        assert_eq!(tree.total_cost, 0.0, "a single leaf has no join cost");
        assert_eq!(tree.join_order(), vec![0]);
    }

    #[test]
    fn deterministic_plan() {
        // Same inputs ⇒ identical plan (join order + algos) across runs.
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), iri("http://ex/p"), var("o")),
            TriplePattern::new(var("o"), iri("http://ex/q"), var("z")),
        ]);
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .predicate(pred("http://ex/p", 100, 100, 100))
            .predicate(pred("http://ex/q", 50, 50, 50))
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let a = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        let b = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        assert_eq!(a.join_order(), b.join_order());
        assert_eq!(a.total_cost, b.total_cost);
    }

    // ============================================================================
    // [OPUS-4.8] sq-bif.3 — correctness suite: previously-uncovered plan branches
    // (the disconnected/Cartesian arm, the independence-estimate non-star fallback, every
    // `star_estimate` None path, multi-source star summation, JoinNode::cardinality on a
    // leaf, total_cost accumulation, and request_cost driving the bind/hash flip). Drives
    // the REAL `plan_bgp` / `cost_join` / `star_estimate` code.
    // ============================================================================

    // ---- A BGP whose patterns share NO variable is a Cartesian product: the planner has no
    //      connected arm to prefer, so it joins on `l * r` and never errors. The output
    //      cardinality of the (only) join is the product of the two leaf cardinalities.
    #[test]
    fn disconnected_bgp_is_cartesian_product() {
        // ?a :p ?b  and  ?c :q ?d — no shared variable.
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("a"), iri("http://ex/p"), var("b")),
            TriplePattern::new(var("c"), iri("http://ex/q"), var("d")),
        ]);
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(10_000)
            .predicate(pred("http://ex/p", 4, 4, 4)) // leaf card 4 (seed).
            .predicate(pred("http://ex/q", 5, 5, 5)) // leaf card 5.
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let tree = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        // The smaller (4) seeds; the join's output is the Cartesian product 4 * 5 = 20.
        assert_eq!(
            tree.root.cardinality(),
            20.0,
            "disconnected join output is the product of the two leaf cardinalities"
        );
        // Both patterns still appear exactly once.
        let mut order = tree.join_order();
        order.sort();
        assert_eq!(order, vec![0, 1]);
    }

    // ---- A connected NON-star join (a chain ?s :p ?o . ?o :q ?z — no shared SUBJECT
    //      variable, so the characteristic-set star estimate does not apply) uses the
    //      independence fallback: out = l*r/max(|L|,|R|), bounded by the Cartesian product.
    #[test]
    fn chain_join_uses_independence_estimate_not_cartesian() {
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), iri("http://ex/p"), var("o")),
            TriplePattern::new(var("o"), iri("http://ex/q"), var("z")), // joins on ?o
        ]);
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(10_000)
            .predicate(pred("http://ex/p", 100, 100, 100))
            .predicate(pred("http://ex/q", 40, 40, 40))
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let tree = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        // Seed is the smaller leaf (:q, 40). Independence: l=40, r=100,
        // ndv = max(40, 100) = 100 ⇒ out = 40*100/100 = 40. Strictly below the
        // Cartesian product 40*100 = 4000, proving the independence path (not Cartesian).
        let out = tree.root.cardinality();
        assert_eq!(out, 40.0, "chain join uses the independence estimate");
        assert!(out < 40.0 * 100.0, "independence estimate is below the Cartesian product");
    }

    // ---- `star_estimate` returns None (⇒ independence fallback) when the star arm REPEATS a
    //      predicate already in the star: the CS model cannot condition on the same predicate
    //      twice. A 2-arm star on the SAME predicate must therefore NOT pick up the CS
    //      cardinality of the existing covering set.
    #[test]
    fn repeated_predicate_star_arm_falls_back_to_independence() {
        // ?s :a ?x . ?s :a ?y — both arms use predicate :a (same star subject ?s).
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), iri("http://ex/a"), var("x")),
            TriplePattern::new(var("s"), iri("http://ex/a"), var("y")),
        ]);
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(10_000)
            .predicate(pred("http://ex/a", 100, 100, 100))
            .char_set(CharSet {
                predicates: vec!["http://ex/a".into()],
                subjects: 100,
                avg_multiplicity: vec![1.0],
            })
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let tree = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        // Independence: l=r=100, ndv=max(100,100)=100 ⇒ out = 100*100/100 = 100.
        assert_eq!(
            tree.root.cardinality(),
            100.0,
            "a repeated-predicate star arm falls back to independence (CS can't condition)"
        );
    }

    // ---- `star_estimate` returns None when NO served characteristic set covers the star's
    //      predicate set: a 2-arm star on a source with predicate stats but no covering CS
    //      falls back to independence rather than fabricating a CS cardinality.
    #[test]
    fn star_with_no_covering_charset_falls_back() {
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), iri("http://ex/a"), var("x")),
            TriplePattern::new(var("s"), iri("http://ex/b"), var("y")),
        ]);
        // CS covers only {a}, never {a,b} ⇒ no covering set for the joined star.
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(10_000)
            .predicate(pred("http://ex/a", 100, 100, 100))
            .predicate(pred("http://ex/b", 60, 60, 60))
            .char_set(CharSet {
                predicates: vec!["http://ex/a".into()],
                subjects: 100,
                avg_multiplicity: vec![1.0],
            })
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let tree = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        // Independence (seed :b, 60): l=60, r=100, ndv=max(60,100)=100 ⇒ 60*100/100 = 60.
        assert_eq!(
            tree.root.cardinality(),
            60.0,
            "no covering characteristic set ⇒ independence fallback, not a CS estimate"
        );
    }

    // ---- A multi-source star sums the characteristic-set estimate over the sources retained
    //      for the joining arm (a multi-source star is a UNION): two sources each with the
    //      same covering CS contribute additively.
    #[test]
    fn multi_source_star_sums_charset_estimates() {
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), iri("http://ex/a"), var("x")),
            TriplePattern::new(var("s"), iri("http://ex/b"), var("y")),
        ]);
        // Two sources BOTH holding {a,b} with the same CS (100 subjects, a 1×, b 2× ⇒ 200).
        let mk = |id: &str| {
            SourceDescriptor::builder(SourceId::new(id))
                .total_triples(10_000)
                .predicate(pred("http://ex/a", 100, 100, 100))
                .predicate(pred("http://ex/b", 200, 100, 100))
                .char_set(CharSet {
                    predicates: vec!["http://ex/a".into(), "http://ex/b".into()],
                    subjects: 100,
                    avg_multiplicity: vec![1.0, 2.0],
                })
                .build()
        };
        let srcs = [mk("S0"), mk("S1")];
        let sel = select_sources(&bgp, &srcs);
        let tree = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        // Each source's covering CS gives 200; the union over the two retained sources sums
        // to 400 (NOT a per-source 200 — that would silently drop a source's contribution).
        assert_eq!(
            tree.root.cardinality(),
            400.0,
            "multi-source star sums the CS estimate over retained sources (union)"
        );
    }

    // ---- JoinNode::cardinality reads the right variant for both a Leaf and a Join, and
    //      total_cost is the SUM of every join node's cost (a 3-pattern plan has 2 joins).
    #[test]
    fn total_cost_accumulates_over_join_nodes() {
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), iri("http://ex/p"), var("o")),
            TriplePattern::new(var("o"), iri("http://ex/q"), var("z")),
            TriplePattern::new(var("z"), iri("http://ex/r"), var("w")),
        ]);
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(100_000)
            .predicate(pred("http://ex/p", 1000, 1000, 1000))
            .predicate(pred("http://ex/q", 10, 10, 10))
            .predicate(pred("http://ex/r", 500, 500, 500))
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let tree = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default()).unwrap();
        // total_cost re-summed by walking the tree must equal the reported field.
        assert_eq!(tree.total_cost, sum_cost(&tree.root));
        // The root is a Join (3 patterns ⇒ 2 joins); its cardinality is the Join variant.
        assert!(matches!(tree.root, JoinNode::Join { .. }));
        // A 3-pattern join has a non-zero total cost (two real joins).
        assert!(tree.total_cost > 0.0);
    }

    // ---- request_cost is the knob that flips bind→hash: with a SMALL left, raising
    //      request_cost high enough makes each per-row bound request dominate a full scan, so
    //      the planner switches the same join from Bind to Hash purely on the knob.
    #[test]
    fn request_cost_knob_flips_bind_to_hash() {
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), iri("http://ex/p"), var("o")),
            TriplePattern::new(var("s"), iri("http://ex/q"), var("z")),
        ]);
        // Left (:p) small (10), right (:q) large (1000): low request_cost ⇒ Bind wins.
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(10_000)
            .predicate(pred("http://ex/p", 10, 10, 10))
            .predicate(pred("http://ex/q", 1000, 1000, 1000))
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let cheap = plan_bgp(
            &bgp,
            &sel,
            &srcs,
            &PlanOptions {
                request_cost: 1.0,
                ..PlanOptions::default()
            },
        )
        .unwrap();
        if let JoinNode::Join { algo, .. } = &cheap.root {
            assert_eq!(*algo, JoinAlgo::Bind, "cheap requests ⇒ bind join on a small left");
        } else {
            panic!("expected a join");
        }
        // Now make each request very expensive: L*(req+f) overtakes R+L ⇒ Hash.
        let pricey = plan_bgp(
            &bgp,
            &sel,
            &srcs,
            &PlanOptions {
                request_cost: 10_000.0,
                ..PlanOptions::default()
            },
        )
        .unwrap();
        if let JoinNode::Join { algo, .. } = &pricey.root {
            assert_eq!(*algo, JoinAlgo::Hash, "expensive requests flip the same join to hash");
        } else {
            panic!("expected a join");
        }
    }

    // ---- An empty `selection` slice (even with a non-empty BGP) yields no plan — the
    //      `selection.is_empty()` guard in `plan_bgp`.
    #[test]
    fn empty_selection_yields_no_plan() {
        let bgp = Bgp::new(vec![TriplePattern::new(
            var("s"),
            iri("http://ex/p"),
            var("o"),
        )]);
        assert!(plan_bgp(&bgp, &[], &[], &PlanOptions::default()).is_none());
    }

    // ---- A connected pattern is ALWAYS preferred over a disconnected (Cartesian) one as the
    //      NEXT arm, even when the disconnected leaf is smaller: the planner avoids a
    //      Cartesian product while any connected arm remains. (Seed selection is purely
    //      smallest-leaf; the connected-vs-disconnected preference is the *next-arm* rule, so
    //      the fixture makes the seed a connected pattern with the global-minimum leaf.)
    #[test]
    fn connected_arm_preferred_over_smaller_disconnected() {
        // ?s :p ?o . ?o :q ?z  (chain, connected) + ?a :mid ?b (disconnected).
        let bgp = Bgp::new(vec![
            TriplePattern::new(var("s"), iri("http://ex/p"), var("o")),
            TriplePattern::new(var("o"), iri("http://ex/q"), var("z")),
            TriplePattern::new(var("a"), iri("http://ex/mid"), var("b")),
        ]);
        let src = SourceDescriptor::builder(SourceId::new("S"))
            .total_triples(100_000)
            .predicate(pred("http://ex/p", 1, 1, 1)) // global-min leaf ⇒ connected SEED.
            .predicate(pred("http://ex/q", 100, 100, 100)) // connected, large.
            .predicate(pred("http://ex/mid", 5, 5, 5)) // disconnected, smaller than :q.
            .build();
        let srcs = [src];
        let sel = select_sources(&bgp, &srcs);
        let order = plan_bgp(&bgp, &sel, &srcs, &PlanOptions::default())
            .unwrap()
            .join_order();
        // Seed is :p (index 0, card 1). The next arm prefers the CONNECTED :q (index 1,
        // card 100) over the smaller DISCONNECTED :mid (index 2, card 5), so the Cartesian
        // arm is deferred to last.
        assert_eq!(order[0], 0, ":p (global-min, connected) seeds");
        assert_eq!(
            order[1], 1,
            "the connected :q is preferred as the next arm over the smaller disconnected :mid"
        );
        assert_eq!(
            *order.last().unwrap(),
            2,
            "the disconnected (Cartesian) arm is deferred to last"
        );
    }
}
