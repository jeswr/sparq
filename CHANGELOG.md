# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **GeoSPARQL function completion (`sparq-geo`)** — the `geof:` registry grows from
  12 to 35 functions: `geof:getSRID` (`xsd:anyURI`), the generic DE-9IM
  `geof:relate`, the full Egenhofer (`geof:eh*`, 8) and RCC8 (`geof:rcc8*`, 8)
  relation families (GeoSPARQL 1.0 Req 25/26 matrix patterns), the set operations
  `geof:intersection`/`union`/`difference`/`symDifference` (polygonal operands via
  geo's `BooleanOps`; other operand types are a clear per-row expression error),
  and `geof:buffer` (unlocked by the geo 0.30→0.33 bump; metric radii buffer in a
  local equirectangular metre frame). All exercised through real SPARQL.
- **GeoIndex: antimeridian, named graphs, incremental updates (`sparq-geo`)** —
  ball queries crossing ±180° split into two longitude windows (merged, deduped;
  brute-force-verified near the seam); `GeoIndex::build` scans every named graph
  and entries record their origin graph + `geo:asWKT` node; new
  `GeoIndex::apply_delta` mirrors a `Graph::apply_delta` batch into the R-tree
  incrementally (rstar insert/remove, O(batch·log n) — no rebuild), including
  `geo:hasGeometry` ownership re-keying.
- **CRS reprojection (`sparq-geo`, opt-in `reproject` feature)** — pure-Rust
  proj4 (`proj4rs`; no C dependency) behind a curated EPSG table (27700 — verified
  against the Ordnance Survey worked example to ~1e-6°, 3857, 2154, 25832/25833,
  UTM 326xx/327xx): `reproject::to_crs84` brings projected literals into the
  geographic machinery (metric distance, GeoIndex).
- **HDT quality-of-life (`sparq-hdt`)** — GZipped containers (`.hdt.gz`) are
  detected by magic bytes (not file names) and decompressed on the fly in every
  entry point; new `header()`/`header_reader()` expose the HDT header (dataset
  metadata triples — VoID statistics, provenance) as a queryable sparq `Graph`.
- **CLI HDT ingestion (`sparq-cli`, opt-in `hdt` feature)** — `--features hdt`
  wires sparq-hdt into the loader dispatch: format argument `hdt` or a
  `.hdt`/`.hdt.gz` extension loads through `sparq_hdt::load`; without the feature
  the CLI exits with a rebuild hint. Off by default (MSRV 1.87 vs the CLI's 1.85
  floor + decode-stack weight; rationale in `crates/sparq-cli/Cargo.toml`).

### Changed

- **`sparq-geo` depends on geo 0.33** (was 0.30; clean API compatibility): brings
  the `Buffer` trait that makes `geof:buffer` implementable.
- **`GeoIndex::entries()` returns an iterator** (was a slice): entry slots are
  tombstoned/reused by incremental deletes.

- **OWL 2 RL rule completion (`sparq-reason`)** — the remaining OWL 2 RL/RDF rules
  (Profiles §4.3): `cls-oo` (oneOf member typing), `cls-maxqc1/2`
  (qualified-cardinality-0 clashes in `inconsistencies()`), the schema-level
  restriction-subsumption family `scm-hv`/`scm-svf1`/`scm-svf2`/`scm-avf1`/`scm-avf2`
  (indexed/guarded joins grouped by onProperty resp. filler — never the naive
  quadratic restriction×restriction pass), and the premise-free
  `cls-thing`/`cls-nothing1` axioms (occurrence-guarded). The rule-coverage table in
  `research/inference-completeness-audit.md` is now fully implemented or explicitly
  by-design (prp-ap, eq-ref, dt-*, reflexive scm-* — each argued). Conformance scores
  and closure throughput unchanged; zero wasm-bundle impact.

- **EXPLAIN / EXPLAIN ANALYZE (`sparq-engine`, `sparq-server`, T22)** — query-plan
  introspection: engine `explain()` renders the algebra tree plus a planning-only dry
  run of the BGP planner (greedy GOO join order with cardinality estimates, per-step
  join strategy — merge/hash/bind/worst-case-optimal — and pushed-down filters) by
  replaying the executor's own planning helpers; `explain_analyze()` (SELECT/ASK)
  executes under a thread-local per-operator trace reporting output rows + wall time
  per operator (zero-cost when off: one flag read per operator entry). Served on
  `/sparql` via `?explain=` / `explain=analyze` or `Accept: text/x-sparq-explain`.
- **Prometheus metrics (`sparq-server`, T22)** — hand-rolled text exposition at
  `GET /metrics` (no new dependencies): request counts by endpoint/status, a `/sparql`
  latency histogram, active-subscription and graph-triple gauges, an update counter.
- **SPARQL CONSTRUCT / DESCRIBE (`sparq-engine`, `sparq-server`)** — RDF-graph query
  results: spec-conformant CONSTRUCT template instantiation (unbound/illegal slots
  skipped, fresh blank nodes per solution, set-deduplicated) and DESCRIBE as concise
  bounded description; engine API `construct` / `describe` / `construct_ntriples`
  (+ `_with_budget` variants); served over HTTP with `application/n-triples` /
  `text/turtle` content negotiation under the existing query budget. W3C construct
  evaluation suites pass 9/9 run (1 skip: `FROM`).
- **Streamed SELECT results (`sparq-server`)** — JSON SELECT bodies are streamed from
  an ordered chunk sequence (engine `query_json_chunks_with_budget`; concatenation
  byte-identical, same `Content-Length`) instead of one giant string: peak server RSS
  for a 1M-row SELECT (202 MB body) drops from ~750 MB to ~405–570 MB.

## [0.1.0] - 2026-06-10

First release: an experimental, from-scratch RDF triplestore and SPARQL engine in Rust
(dictionary-encoded, six sorted permutation indexes, parallel execution), published as the
`sparq-core` / `sparq-engine` / `sparq-reason` / `sparq-cli` / `sparq-server` crates.
The API is unstable and several SPARQL features are still in progress (named graphs,
property paths, SERVICE, UPDATE) — see `research/roadmap.md`.

### Added

- **Storage (`sparq-core`)** — dictionary-encoded triple store with six sorted permutation
  indexes (Hexastore/RDF-3X lineage); a 3-permutation compact-index mode (~half the index
  memory) for memory-constrained targets; parallel index construction.
- **Out-of-core mode** — build the indexes on disk and query them memory-mapped: 100M-triple
  external build in 74 s at 4.2 GB peak RAM on a 2020 M1 Air, then 0.67 s open with ~0
  committed heap (OS page cache only), at query speeds matching the in-memory engine.
- **Parsing** — N-Triples, Turtle, N-Quads, TriG (via `oxttl`); parallel and streaming, with
  transparent `.gz` / `.bz2` / `.zst` decompression. Real-Wikidata ingestion ~1.3 M triples/s
  building all six permutations out-of-core on a fanless laptop.
- **SPARQL query (`sparq-engine`)** — SELECT/ASK; Basic Graph Patterns, FILTER, OPTIONAL,
  UNION, MINUS, VALUES, BIND, aggregation + GROUP BY, ORDER BY, DISTINCT, LIMIT/OFFSET;
  sort-merge, hash, and worst-case-optimal joins with greedy cardinality-based planning;
  parallel scan / filter / sort / aggregation / serialization.
- **Inference (`sparq-reason`, opt-in)** — RDFS, OWL-RL, and a Notation3 subset with
  saturate-then-sweep single-pass materialization (~17× over a naive fixpoint for RDFS,
  ~30× OWL-RL fast path) and union-find `owl:sameAs`.
- **CLI (`sparq-cli`)** — load/query/bench subcommands, on-disk index build + `query-mmap`,
  `reason`, a per-subsystem parallel-scaling harness, and hardware-tiered release binaries
  (per-ISA prefetch tuning selected per silicon family).
- **HTTP server (`sparq-server`)** — W3C SPARQL 1.1 Protocol query endpoint (GET/POST) and
  Graph Store HTTP Protocol read side; JSON / XML / CSV / TSV results with content
  negotiation; Docker image (distroless, `ghcr.io/jeswr/sparq-server`).
- **WebAssembly (`sparq-wasm`, unpublished)** — the core engine compiled for the browser with
  a minimal bundle (no threads, compact index); ships via npm later, not crates.io.

### Performance

Measured against native QLever 0.5.47 on the same machine (2020 MacBook Air M1, 16 GB),
compute-only, cold, min-of-N — methodology and full tables in `bench/qlever-baselines.md`:

- Synthetic 10M and 100M join/OPTIONAL queries: **2.2–20× faster** than QLever, in-memory and
  out-of-core (mmap) alike; the advantage holds from 10M to 100M.
- Real skewed data (DBpedia/Wallscope olympics, 1.78M triples): faster on **every** query,
  **1.7–12×**.
- Honest caveats: QLever's on-disk index is *compressed* (smaller disk footprint at
  billion-scale); billion-scale real data (Wikidata/WDBench) is untested on this hardware;
  single-pattern/FILTER comparisons short-circuit via index range-counting and are excluded
  from the claims above.

[0.1.0]: https://github.com/jeswr/sparq/releases/tag/v0.1.0
