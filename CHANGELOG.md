# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Severity-aware conformance + conformance memo (`sparq-shacl`)** —
  `ValidationReport::conforms_violations_only()` (CI-style gating: warnings and
  infos report but don't break conformance) and `results_with_severity(iri)`;
  the spec's `sh:conforms` is untouched and the API stays infallible. Inside
  the validator, `conforms()` now memoises `(focus, shape) → bool` with a
  cycle-soundness rule that leaves the recursion guard unchanged (a result is
  cached only when no guard re-entry escaped below its frame). W3C core suite
  unchanged at 98/98; pinned by new cyclic-shape and severity tests.

- **Neighbor-sparse fallback for `most_similar` (`sparq-sim`)** — entities whose
  signature elements all point at degree-1 neighbors (every event names exactly
  one sport) generated NO candidates in v1. `SimConfig::profile_fallback`
  (default on) fills starved slots with predicate-profile (role) matches, ranked
  below every exactly-scored result; predicate blocks are scanned
  most-selective-first with a `max(4k, 64)` stop. Re-measured on the real
  olympics eval: Sport precision@10 0.000 (0/0) → 1.000 (400/400), City 0.500
  (2/4) → 0.995 (398/400), full 2400/2400 candidate coverage, ~0.3 ms mean
  latency cost. The per-element IDF hub budget recorded in the TODO is
  measured-and-rejected (the eval is saturated); numbers in `sparq-sim/TODO.md`.

- **Streaming store builds, weighted RRF, bytes-backed open (`sparq-vectors`)**
  — `StreamingWriter` removes the build-phase RAM ceiling (vectors append
  straight to the `.spqv` data section, the id→slot index spills to a sidecar;
  output byte-identical to the in-RAM builder — no format change; duplicates
  reported at finalize), `fuse_rrf_weighted` (Elasticsearch-style per-list
  weights, unit weights ≡ `fuse_rrf`), and `VectorStore::open_from_bytes`
  (filesystem-less environments; validation identical to `open`, all read paths
  shared). No wasm feature wired (deliberate — the crate stays out of the wasm
  graph); DiskANN-style persistent ANN, quantization and big-endian remain
  deferred with rationale in `sparq-vectors/TODO.md`.

- **Overlay window evaluation, t0, CONSTRUCT/ASK (`sparq-rsp`)** — closed
  windows are no longer rebuilt from scratch: the new `EvalMode` selects the
  R2R materialisation strategy — `PersistentDict` (one dictionary per
  continuous query, terms interned once at push time, per-window graphs from
  already-interned ids; the new default — wins every benchmark scenario,
  1.2–5.3× over the v1 rebuild, biggest on sliding windows), `Delta` (one
  live graph + set-semantic `apply_delta` per slide with churn-driven
  compaction; measured slower everywhere, kept opt-in), and `Rebuild` (the v1
  baseline, still right for unbounded-vocabulary streams). Plus: RSP-QL
  window origin `t0` (`WindowSpec::with_t0`), and continuous CONSTRUCT
  (`ContinuousConstruct` — stream-to-stream transformation with exact
  set-diff ISTREAM/DSTREAM) and ASK (`ContinuousAsk`) query forms. All three
  modes are observationally identical (pinned by tests); README throughput
  table re-measured per mode. Zero wasm-bundle impact (opt-in crate).

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
