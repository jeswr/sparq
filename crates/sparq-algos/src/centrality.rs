//! Degree centrality over a [`NodeGraph`].
//!
//! The simplest, most widely-used centrality: a node's importance is how many edges touch
//! it. Three variants are exposed because direction matters in RDF — in-degree (how many
//! distinct subjects point at an entity — "authority"/popularity), out-degree (how many
//! distinct objects an entity points at — "hub"/fan-out), and total degree (the sum).
//!
//! Degrees here count **distinct neighbours** (parallel edges collapsed by the
//! [`NodeGraph`] build), so a node asserted as the object of one subject via five
//! predicates contributes in-degree 1 from that subject, not 5.

use crate::graph::NodeGraph;

/// Which direction(s) a [`degree_centrality`] computation counts.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Direction {
    /// In-edges only: `score[i] = in_degree(i)`.
    In,
    /// Out-edges only: `score[i] = out_degree(i)`.
    Out,
    /// Both: `score[i] = in_degree(i) + out_degree(i)`. The default.
    #[default]
    Total,
}

/// Raw degree of every node, indexed by node index. `O(V)`.
pub fn degree_centrality(g: &NodeGraph, dir: Direction) -> Vec<usize> {
    (0..g.len())
        .map(|i| match dir {
            Direction::In => g.in_degree(i),
            Direction::Out => g.out_degree(i),
            Direction::Total => g.in_degree(i) + g.out_degree(i),
        })
        .collect()
}

/// Degree centrality **normalised** to `[0, 1]` by dividing by the maximum possible
/// degree `n - 1` (the standard normalisation, so scores are comparable across graphs of
/// different size). Returns all-zero for a graph with `< 2` nodes.
pub fn degree_centrality_normalized(g: &NodeGraph, dir: Direction) -> Vec<f64> {
    let n = g.len();
    if n < 2 {
        return vec![0.0; n];
    }
    let denom = (n - 1) as f64;
    degree_centrality(g, dir)
        .into_iter()
        .map(|d| d as f64 / denom)
        .collect()
}

/// The `k` highest-scoring nodes as `(node_index, score)`, best first. Ties break by
/// **ascending node index** for determinism. `O(V log V)`.
pub fn top_k(scores: &[usize], k: usize) -> Vec<(usize, usize)> {
    let mut ranked: Vec<(usize, usize)> = scores.iter().copied().enumerate().collect();
    // Sort by score descending, then index ascending.
    ranked.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    ranked.truncate(k);
    ranked
}
