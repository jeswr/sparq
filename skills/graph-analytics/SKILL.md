---
name: graph-analytics
description: "Graph analytics over a sparq RDF graph with the opt-in sparq-algos crate: project the graph onto a directed NodeGraph and run PageRank, degree/in/out centrality, feature-gated exact Brandes betweenness and harmonic closeness, weakly-connected-component and label-propagation community detection — all read directly from sparq-core's permutation indexes, deterministic, no model, no network. Use when ranking entities by importance/centrality, finding clusters/communities, or computing connected components over a sparq Graph; topology-only (edges are predicate-erased and unweighted — filter the source graph for a per-predicate sub-graph)."
license: MIT
metadata:
  version: "0.1.0"
  homepage: https://github.com/jeswr/sparq
---

# sparq-algos — graph analytics

`sparq-algos` is an **opt-in** crate (vendor-parity, epic sq-3183) that runs classic graph
algorithms over a `sparq_core::Graph`: **PageRank**, **degree centrality** (in / out /
total), feature-gated exact **Brandes betweenness** and **harmonic closeness**,
**weakly-connected components**, and a deterministic **label-propagation** community
heuristic. It consumes only sparq-core's public read API (the borrowing
triple-id iterator + dict lookups), holds no graph state of its own, and nothing in the
workspace depends on it — the default engine build does not even compile it.

These are **topology** algorithms: every triple `(s, p, o)` becomes one directed edge
`s → o`, the predicate is **erased**, parallel edges are collapsed, and edges are
unweighted. To analyse a single relation (e.g. only `foaf:knows`), filter the source graph
first. There is no SPARQL-level integration — call the Rust API directly.

## Quickstart

`crates/sparq-algos/Cargo.toml` (extended centrality is explicitly enabled here):

```toml
[dependencies]
sparq-core  = { path = "../sparq-core" }
sparq-algos = { path = "../sparq-algos", features = ["centrality-extended"] }
oxrdf = "*"   # for oxrdf::{NamedNode, Term} when resolving node indices back to terms
```

Build the view once, then run any algorithm over it:

```rust,ignore
use sparq_algos::{
    NodeGraph, NodeFilter,
    pagerank, PageRankConfig,
    degree_centrality, degree_centrality_normalized, Direction, top_k,
    betweenness_centrality, closeness_centrality,
    weakly_connected_components, label_propagation, LabelPropConfig, num_communities,
};

// Project the RDF graph onto a directed node graph.
let g = NodeGraph::build(&graph);                          // entities only (literals dropped)
let g = NodeGraph::build_with(&graph, NodeFilter::All);    // include literal objects as nodes

// --- PageRank: stationary distribution, sums to ~1.0, indexed by node index ---
let ranks = pagerank(&g, PageRankConfig::default());       // d = 0.85, tol 1e-9
// node index -> the original RDF term:
let top_node = (0..g.len()).max_by(|&a, &b| ranks[a].total_cmp(&ranks[b])).unwrap();
let term = g.term(&graph, top_node);                       // oxrdf::Term

// --- Degree centrality (raw counts or normalised), plus the top-k ---
let indeg = degree_centrality(&g, Direction::In);          // Vec<usize>, per node
let norm  = degree_centrality_normalized(&g, Direction::Total); // Vec<f64> in [0,1]
let top10 = top_k(&indeg, 10);                             // Vec<(node_index, score)>, best first

// --- Exact shortest-path centrality over the weak (undirected) topology ---
let between = betweenness_centrality(&g);                  // unnormalised; unordered pairs
let close   = closeness_centrality(&g);                    // normalised harmonic mean, [0, 1]

// --- Community detection ---
let comp = weakly_connected_components(&g);                // exact, union-find; Vec<usize> labels
let comm = label_propagation(&g, LabelPropConfig::default()); // deterministic heuristic
let k    = num_communities(&comm);                         // distinct community count
```

## API surface

| Item | What it gives you |
| --- | --- |
| `NodeGraph::build(&graph)` | the directed node view, entities only (literal objects dropped) |
| `NodeGraph::build_with(&graph, NodeFilter::All)` | view that also makes literal objects nodes |
| `NodeGraph::{len, edge_count, is_empty}` | node / edge counts |
| `NodeGraph::{out_neighbors, in_neighbors, out_degree, in_degree}(i)` | adjacency by node index |
| `NodeGraph::{dict_id, index_of, term}` | node index ↔ sparq-core dict `Id` ↔ `oxrdf::Term` |
| `pagerank(&g, PageRankConfig)` | `Vec<f64>` stationary distribution (sums to ~1.0) |
| `degree_centrality(&g, Direction)` | `Vec<usize>` raw degree (`In` / `Out` / `Total`) |
| `degree_centrality_normalized(&g, Direction)` | `Vec<f64>` in `[0,1]` (divided by `n-1`) |
| `top_k(&scores, k)` | top-`k` `(node_index, score)`, ties → ascending index |
| `betweenness_centrality(&g)` | exact unnormalised Brandes score over unordered endpoint pairs; requires `centrality-extended` |
| `closeness_centrality(&g)` | normalised harmonic mean inverse distance; requires `centrality-extended` |
| `weakly_connected_components(&g)` | exact component labels (`Vec<usize>`) |
| `label_propagation(&g, LabelPropConfig)` | heuristic community labels (`Vec<usize>`) |
| `num_communities(&labels)` | distinct community count |

## Honest scope / caveats

- **Topology only.** Predicates are erased and edges are unweighted; parallel edges
  collapse to one. For a per-predicate or weighted analysis, pre-filter the source graph.
- **Literals are not nodes by default** (`NodeFilter::EntitiesOnly`) — analytics run over
  the *entity* graph. Pass `NodeFilter::All` to include literal objects.
- **Deterministic.** PageRank uses uniform init + fixed iteration order (no RNG); WCC is
  exact; `top_k` and `label_propagation` use ascending-index/label tie-breaks, so repeated
  runs on the same graph give identical results. Label propagation is still a *heuristic*
  (it does not optimise modularity and a connected component may split into several LP
  communities — or, on dense graphs, collapse into one); WCC is the exact answer to
  "which entities are connected at all".
- **Extended centrality uses weak topology.** Betweenness and closeness traverse each
  directed edge in either direction, like WCC and label propagation. A reciprocal edge is
  still one connection, and a self-loop contributes no shortest path. Betweenness is
  unnormalised and counts unordered endpoint pairs; closeness is the harmonic mean
  `sum(1/d)/(V-1)`, with unreachable nodes contributing zero.
- **Extended centrality is exact and all-pairs.** It is not sampled or approximate. Both
  algorithms take `O(V * (V + E))` time, so callers should use them only where that cost is
  acceptable. Enable the default-OFF `centrality-extended` feature to compile them.
- **PageRank** handles dangling (out-degree-0) nodes by redistributing their mass
  uniformly each iteration, so the result is a proper probability distribution.
- **In-memory.** The `NodeGraph` is built from one pass over `Graph::iter_ids` and held in
  RAM (CSR forward + reverse adjacency keyed by dense `u32` node indices); it does not
  reference the source graph after building, except `term()` which needs the dict.
- Opt-in, workspace v0.1.0, `#![forbid(unsafe_code)]`. [OPUS-4.8] pending re-review.
