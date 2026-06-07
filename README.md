# sparq — a state-of-the-art RDF + SPARQL engine in Rust

A from-scratch, dictionary-encoded triplestore and SPARQL query engine, built to
match/beat QLever, RDFox, Virtuoso, GraphDB, Blazegraph and MillenniumDB on the
standard benchmarks (see `research/ARCHITECTURE.md` for the blueprint and the
target scorecard).

## Status
**M1 (foundation):** dictionary encoding (`u32` ids), six sorted permutation
indexes (Hexastore/RDF-3X/QLever base), range-scan by binary search, SPARQL
SELECT with Basic Graph Patterns via greedy-ordered hash joins, a FILTER subset,
and DISTINCT/LIMIT/OFFSET. SPARQL syntax → algebra via `spargebra`; the store,
planner and physical execution are our own.

## Roadmap (see research/ARCHITECTURE.md)
- M2: merge joins on sorted permutations + worst-case-optimal (Leapfrog Triejoin)
  joins; DP/characteristic-set planning; OPTIONAL/UNION/aggregation.
- M3: block compression of id lists, parallel bulk load, parallelism.
- M4: characteristic-set cardinality, property paths, vectorized execution,
  inline numeric value-ids.
- WASM: port the hot path to the browser with a minimal bundle.

## Layout
- `crates/sparq-core` — dictionary + permutation-index store + bulk loader.
- `crates/sparq-engine` — SPARQL algebra → physical plan → execution.
- `crates/sparq-cli` — command-line loader/query runner.
