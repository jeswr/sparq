# sparq-algos

**Graph analytics** for the sparq RDF engine — an **opt-in** crate (vendor-parity,
epic sq-3183) that runs classic graph algorithms directly over a `sparq_core::Graph`.
PageRank, degree/in/out centrality, weakly-connected-component and label-propagation
community detection — computed from sparq-core's permutation indexes with **no extra
state**, no model, and no network. Nothing in the workspace depends on it; the default
engine build does not even compile it.

## 🚀 Quickstart

```rust,ignore
use sparq_algos::{
    NodeGraph, NodeFilter,
    pagerank, PageRankConfig,
    degree_centrality, Direction, top_k,
    weakly_connected_components, label_propagation, LabelPropConfig, num_communities,
};

// Project the RDF graph onto a directed node graph (subjects + entity objects = nodes,
// each triple (s,p,o) = an edge s → o; predicates erased, parallel edges collapsed).
let g = NodeGraph::build(&graph);                 // default: entities only (no literals)
let g = NodeGraph::build_with(&graph, NodeFilter::All); // include literal objects too

// PageRank — stationary distribution, sums to ~1.0, indexed by node index.
let ranks = pagerank(&g, PageRankConfig::default());     // d = 0.85
let term  = g.term(&graph, best_index);                  // node index → oxrdf::Term

// Degree centrality (In / Out / Total) and the top-k entities.
let deg = degree_centrality(&g, Direction::In);          // Vec<usize>, per node
let top = top_k(&deg, 10);                               // Vec<(node_index, score)>

// Community detection.
let comp = weakly_connected_components(&g);              // exact, union-find
let comm = label_propagation(&g, LabelPropConfig::default()); // heuristic, deterministic
let k    = num_communities(&comm);
```

## ✨ Features

- **`NodeGraph`** — a directed, predicate-erased CSR view of the graph keyed by dense node
  indices, built in one pass over `Graph::iter_ids`; forward + reverse adjacency, parallel
  edges collapsed, self-loops kept. Maps each node back to its dictionary `Id` / `Term`.
- **PageRank** — the random-surfer power method with correct dangling-node mass
  redistribution; deterministic (no RNG), converges in L1 to a configurable tolerance.
- **Degree centrality** — In / Out / Total, raw counts or normalised to `[0, 1]`, plus a
  deterministic `top_k`.
- **Community detection** — exact weakly-connected components (near-linear union-find) and
  a deterministic label-propagation heuristic; dense, ascending-order community ids.
- **Opt-in & lean** — consumes only sparq-core's public read API; the only dependencies
  are `sparq-core`, `oxrdf`, and `rustc-hash`. No cargo features; no engine, no wasm, no
  network code enters the build.

## 📚 Learn more

- The capability skill: [`skills/graph-analytics/SKILL.md`](../../skills/graph-analytics/SKILL.md).
- Source: `src/graph.rs` (the view), `src/pagerank.rs`, `src/centrality.rs`,
  `src/community.rs`. Tests live in each module and in `tests/`.

These are **topology** algorithms: edges are unweighted and predicate-erased. To analyse a
sub-graph (e.g. only `foaf:knows` edges), filter the source graph first; predicate-weighted
and predicate-projected views are tracked as follow-up beads.

## License

Licensed under the MIT license, same as the rest of the sparq workspace.
