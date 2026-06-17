//! PageRank over a [`NodeGraph`].
//!
//! The classic random-surfer formulation: at each step the surfer follows a uniformly
//! random out-edge with probability `damping`, and teleports to a uniformly random node
//! with probability `1 - damping`. The returned vector is the stationary distribution —
//! it sums to 1.0 (modulo floating-point) and `rank[i]` is the long-run probability of
//! being at node `i`.
//!
//! **Dangling nodes** (no out-edges) would leak rank mass; the standard fix is applied —
//! their mass is redistributed uniformly across all nodes each iteration, so the vector
//! stays a probability distribution and the result is independent of dangling structure.

use crate::graph::NodeGraph;

/// Tuning for [`pagerank`].
#[derive(Clone, Copy, Debug)]
pub struct PageRankConfig {
    /// Damping factor `d` — the probability of following an out-edge rather than
    /// teleporting. The canonical value is `0.85`.
    pub damping: f64,
    /// Stop when the L1 change between iterations falls below this. `1e-9` is tight
    /// enough for ranking the top entities on graphs up to millions of nodes.
    pub tolerance: f64,
    /// Hard cap on iterations so a pathological graph still terminates. `100` is far
    /// more than the ~50–60 the power method needs at `d = 0.85`.
    pub max_iterations: usize,
}

impl Default for PageRankConfig {
    fn default() -> PageRankConfig {
        PageRankConfig {
            damping: 0.85,
            tolerance: 1e-9,
            max_iterations: 100,
        }
    }
}

/// Computes PageRank over `g`, returning `rank` indexed by node index (`rank[i]` for node
/// `i`). The vector sums to ~1.0. Returns an empty vector for an empty graph.
///
/// Complexity: `O((V + E) · iterations)` time, `O(V)` extra space. Deterministic — no RNG,
/// uniform initialisation, fixed iteration order.
pub fn pagerank(g: &NodeGraph, cfg: PageRankConfig) -> Vec<f64> {
    let n = g.len();
    if n == 0 {
        return Vec::new();
    }
    let nf = n as f64;
    let d = cfg.damping;
    let teleport = (1.0 - d) / nf;

    let mut rank = vec![1.0 / nf; n];
    let mut next = vec![0.0f64; n];

    for _ in 0..cfg.max_iterations {
        // Mass stuck on dangling (out-degree 0) nodes, redistributed uniformly.
        let mut dangling_sum = 0.0;
        for (i, &r) in rank.iter().enumerate() {
            if g.out_degree(i) == 0 {
                dangling_sum += r;
            }
        }
        let dangling_share = d * dangling_sum / nf;

        // Each node starts with the teleport + redistributed-dangling baseline, then
        // receives its share from each in-neighbour, split by that neighbour's
        // out-degree. Iterating by destination (in-edges) means each contribution is
        // read exactly once.
        for (i, slot) in next.iter_mut().enumerate() {
            let mut acc = 0.0;
            for &src in g.in_neighbors(i) {
                let src = src as usize;
                let outdeg = g.out_degree(src);
                // src has an edge into i, so its out-degree is >= 1.
                acc += rank[src] / outdeg as f64;
            }
            *slot = teleport + dangling_share + d * acc;
        }

        // L1 convergence check.
        let mut delta = 0.0;
        for (i, &r) in rank.iter().enumerate() {
            delta += (next[i] - r).abs();
        }
        std::mem::swap(&mut rank, &mut next);
        if delta < cfg.tolerance {
            break;
        }
    }
    rank
}
