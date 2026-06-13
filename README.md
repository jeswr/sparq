# sparq

A from-scratch **RDF triplestore and SPARQL engine in Rust** — dictionary-encoded, with six
sorted permutation indexes, parallel execution, RDFS/OWL-RL/N3 inference, an out-of-core
(memory-mapped) mode with a compressed on-disk format, a WebAssembly build, and a
W3C-conformant HTTP server. Faster than QLever on every compute benchmark run so far
(see [the comparison at the bottom](#benchmarks--sparq-vs-qlever)).

> **Status: experimental research engine.** Tested against the full W3C SPARQL suites —
> **1229 of 1229 tests accounted for: 1225 pass, zero fail, zero skips**, plus 4 documented
> divergences where the suite's expected files are provably wrong against the spec and
> their own approved siblings (rendered with full rationale in the conformance report;
> rdf-tests issue drafts in [`docs/upstream-proposals.md`](docs/upstream-proposals.md)).
> Six former upstream-parser failures are fixed in a vendored spargebra
> ([`vendor/spargebra/SPARQ-PATCHES.md`](vendor/spargebra/SPARQ-PATCHES.md)); upstream has
> since fixed all six on its main branch, and the vendor copy retires when that releases.
> The API is still unstable; SERVICE federation remains unimplemented
> (see [`research/roadmap.md`](research/roadmap.md), audit:
> [`research/roadmap-completion-audit.md`](research/roadmap-completion-audit.md)).

## Packages

The engine core (always built):

| crate | what |
|---|---|
| [`sparq-core`](crates/sparq-core) | dictionary, six permutation indexes, parallel + streaming loaders, delta-overlay incremental updates + WAL, on-disk/mmap store (raw + block-compressed formats) |
| [`sparq-engine`](crates/sparq-engine) | SPARQL algebra → physical plan → parallel execution; query/update/CONSTRUCT/EXPLAIN entry points, custom-function registry, zero-copy named-graph dataset views, query budgets |

Interfaces:

| crate / package | what |
|---|---|
| [`sparq-cli`](crates/sparq-cli) | command line: query, build/query-mmap (out-of-core), reason, bench, scaling sweeps, save/recompress |
| [`sparq-server`](crates/sparq-server) | W3C SPARQL 1.1 Protocol HTTP server: query + update, Graph Store read, streaming results, WebSocket subscriptions, EXPLAIN, Prometheus `/metrics`, opt-in time-travel queries (`?generation=N`) |
| [`sparq-wasm`](crates/sparq-wasm) | WebAssembly build of the core engine (browser; tracked minimal bundle) |
| [`js/`](js/) | `@jeswr/sparq` — RDF/JS-typed npm package over the wasm build (zero runtime deps) |
| [`sparq-py`](crates/sparq-py) | Python bindings (`sparq` package, pyo3/maturin) |

Opt-in capability crates (none is a dependency of the core; zero wasm impact, enforced in CI):

| crate | what |
|---|---|
| [`sparq-reason`](crates/sparq-reason) | RDFS / OWL-RL / Notation3 inference (EYE-validated builtins, goal-directed `<=`), incremental closure maintenance |
| [`sparq-shacl`](crates/sparq-shacl) | SHACL Core validation — 100% of the W3C core suite |
| [`sparq-solid`](crates/sparq-solid) | [Solid](https://solidproject.org/) pod access control: WAC/ACP as N3 rules over a triples-native auth view; per-(WebID, client) session-filtered SPARQL, fail-closed |
| [`sparq-geo`](crates/sparq-geo) | GeoSPARQL: WKT, `geof:` functions callable from SPARQL, DE-9IM relations, R-tree index |
| [`sparq-text`](crates/sparq-text) | full-text search: owned BM25 inverted index over string literals, `text:` magic predicates rewritten into plain SPARQL |
| [`sparq-rsp`](crates/sparq-rsp) | RDF stream processing: deterministic RSP-QL windows, R/I/DSTREAM diffs |
| [`sparq-hdt`](crates/sparq-hdt) | load HDT archives into a graph |
| [`sparq-introspect`](crates/sparq-introspect) | one-scan schema introspection: characteristic sets, class/predicate profiles, LLM-ready digests |
| [`sparq-nlq`](crates/sparq-nlq) | grounded natural-language → SPARQL (introspect-grounded prompts, validate/repair loop, record/replay testing) |
| [`sparq-vectors`](crates/sparq-vectors) | embeddings: mmap vector store, HNSW ANN, entity verbalization (labels + literals), hybrid rank fusion |
| [`sparq-sim`](crates/sparq-sim) | training-free structural similarity from the permutation indexes |
| [`sparq-conformance`](crates/sparq-conformance) | W3C test-suite runner + scoreboard (the CI ratchet) |
| [`sparq-bench`](crates/sparq-bench) | dataset generation, differential correctness vs Oxigraph, scaling harness |
| [`sparq-gpu`](crates/sparq-gpu) | wgpu compute prototype — measured and parked ([verdict](research/gpu-verdict.md)) |

## Usage

> **Note (crates.io installs):** builds installed from crates.io (e.g. `cargo install
> sparq-cli`) resolve the upstream `spargebra` 0.4.6 parser — the vendored SPARQL-parser
> conformance fixes ([`vendor/spargebra/SPARQ-PATCHES.md`](vendor/spargebra/SPARQ-PATCHES.md))
> apply only to git builds (building this repository directly) until the upstream PRs land.

```sh
# build
cargo build --release

# run a query over a file (Turtle / N-Triples / N-Quads / TriG, optionally .gz/.bz2/.zst)
cargo run --release -p sparq-cli -- query data.ttl turtle \
  'SELECT ?s ?o WHERE { ?s <http://schema.org/name> ?o } LIMIT 10'

# build on-disk indexes once, then query them memory-mapped (out-of-core, ~0 heap)
cargo run --release -p sparq-cli -- build data.nt ntriples ./idx
cargo run --release -p sparq-cli -- query-mmap ./idx 'SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }'

# RDFS / OWL-RL materialization
cargo run --release -p sparq-cli -- reason ontology.ttl turtle rdfs

# HTTP server (W3C SPARQL Protocol) on :3030
cargo run --release -p sparq-server -- --addr 127.0.0.1:3030 --format turtle data.ttl

# per-subsystem parallel scaling sweep
cargo run --release -p sparq-cli -- scaling data.nt ntriples queries/ 1,2,4,8
```

As a library:

```rust
use sparq_core::Graph;

let g = Graph::load_str(turtle_text, "turtle")?;
let rows = sparq_engine::query(&g, "SELECT ?s WHERE { ?s a <http://schema.org/Person> }")?;
let json = sparq_engine::query_json(&g, "SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }")?;
```

Custom functions, query budgets, and named-graph dataset views compose on the same entry
points (`query_with_functions`, `query_with_budget`, `query_view` /
[`docs/extension-functions.md`](docs/extension-functions.md)). Python and JavaScript mirror
the same surface — see [`crates/sparq-py`](crates/sparq-py) and [`js/`](js/).

## Why it exists

To find out how fast a clean, modern RDF engine can be. The design follows the
dictionary-encoded + permutation-index lineage (Hexastore / RDF-3X / QLever), but the store,
planner, and physical execution are written from scratch with an emphasis on parallelism and
a memory-bounded out-of-core path.

## Features

- **Parsing** — N-Triples, Turtle, N-Quads, TriG; parallel; streaming + transparently
  decompressing (`.gz` / `.bz2` / `.zst`) with no full-document copy in RAM; parallel
  (sharded) dictionary construction by default.
- **SPARQL query** — SELECT/ASK, Basic Graph Patterns, FILTER (full built-in function set incl.
  REGEX), OPTIONAL, UNION, MINUS, VALUES, BIND, aggregation + GROUP BY, ORDER BY, DISTINCT,
  LIMIT/OFFSET, **property paths** (all 8 operators), **named graphs** (`GRAPH`, FROM /
  FROM NAMED, N-Quads/TriG datasets); sort-merge + hash + worst-case-optimal joins; greedy
  cardinality-based planning; numeric and **temporal side-caches** (dict-free dateTime
  FILTER/ORDER BY). Parallel scan / filter / sort / join-build / aggregation / serialization.
  **Custom functions** (SPARQL 17.6 extensible value testing): register Rust closures under
  function IRIs ([`docs/extension-functions.md`](docs/extension-functions.md)). **Dataset
  views**: zero-copy restriction of a query to a set of named graphs (a non-visible graph is
  indistinguishable from an absent one).
- **SPARQL Update** — the complete operation set (data ops, `DELETE/INSERT … WHERE` with GRAPH
  templates, `USING`, `LOAD`, `CLEAR`/`DROP`/`CREATE`/`COPY`/`MOVE`/`ADD`) over default and named
  graphs — **100% of the W3C update suite**. Updates are **incremental** (delta overlay +
  append-only dictionary, ~10⁶× faster than rebuild) with an optional **write-ahead log** for
  durability, and run end-to-end over HTTP in ~300 µs.
- **CONSTRUCT / DESCRIBE + streaming** — graph results with content negotiation; SELECT
  results stream in chunks (hundreds of MB lower peak RSS on large answers).
- **Subscriptions** — SEPA-style WebSocket subscriptions: register a SELECT, receive
  added/removed binding diffs after every committed update
  ([`crates/sparq-server/SUBSCRIPTIONS.md`](crates/sparq-server/SUBSCRIPTIONS.md)).
- **RDF 1.2 / RDF-star** — triple terms (`<< s p o >>`) stored structurally in the dictionary;
  triple-term patterns (incl. variables inside quoted triples) match; SPARQL 1.2 builtins,
  directional language tags, JSON/XML result rendering.
- **Inference** (opt-in `sparq-reason`) — RDFS, OWL-RL, and a Notation3 engine (EYE-validated
  builtins, goal-directed `<=`), with single-pass materialization and **incremental closure
  maintenance** (counting-based; deltas ~10³–10⁴× faster than re-materialization). With the
  non-default `explain` feature, `why(triple)` returns a **proof tree** for any inferred
  triple (rule + premises, recursively to asserted facts; flat, ZK-witness-friendly shape;
  JSON + human-readable renderings).
- **Validation** (opt-in `sparq-shacl`) — SHACL Core at **100% of the W3C core suite**.
- **Solid access control** (opt-in `sparq-solid`) — pods as named-graph-per-document;
  WAC/ACP evaluated as N3 rules materializing a queryable, triples-native authorization
  view; SPARQL filtered per (WebID, client) session through the zero-copy dataset view
  (~1 ms flat session overhead), fail-closed
  ([`crates/sparq-solid/README.md`](crates/sparq-solid/README.md), design:
  [`research/solid-access-control-design.md`](research/solid-access-control-design.md)).
- **GenAI toolkit** (opt-in) — schema introspection digests (`sparq-introspect`), grounded
  NL→SPARQL with validate/repair (`sparq-nlq`), entity-text embeddings + ANN + hybrid fusion
  (`sparq-vectors`), structural similarity (`sparq-sim`).
- **Out-of-core** — build on-disk indexes and query them **memory-mapped**; optional
  **block-compressed permutations** (2.5–2.75×, lazy block serving) behind a format-version
  byte — old raw directories keep loading.
- **HTTP server** (`sparq-server`) — W3C SPARQL 1.1 Protocol query + update endpoints, Graph
  Store read, result formats JSON / XML / CSV / TSV; content negotiation; **EXPLAIN /
  EXPLAIN ANALYZE** plan introspection (`?explain=…`); a **Prometheus `/metrics`**
  endpoint; opt-in **time-travel queries** (`--features time-travel`: `?generation=N`
  pins a retained generation, `Sparq-Generation` response tokens, honest 410 when
  history ages out) (see [`crates/sparq-server/README.md`](crates/sparq-server/README.md)).
  Docker image via the release workflow (see [`docs/release.md`](docs/release.md)).
- **WebAssembly** — the core engine builds for the browser with a minimal, CI-tracked bundle
  (1,643,103 B raw cargo artifact; ~1.2 MB shipped after `wasm-opt`).

## Documentation

- [`research/ARCHITECTURE.md`](research/ARCHITECTURE.md) — design blueprint.
- [`research/roadmap.md`](research/roadmap.md) — the work threads, and
  [`research/roadmap-completion-audit.md`](research/roadmap-completion-audit.md) — per-thread
  completion evidence (merge commits, verdicts, measurements).
- [`docs/extension-functions.md`](docs/extension-functions.md) — registering custom SPARQL
  functions (and the GeoSPARQL `geof:` registry built on it).
- [`bench/qlever-baselines.md`](bench/qlever-baselines.md) — the QLever comparison
  methodology and full numbers; [`research/BENCHMARKS.md`](research/BENCHMARKS.md) — the
  continuous Oxigraph differential harness.
- `research/` — design notes and measured verdicts on indexing, compression, inference,
  parallelism, RDF 1.2, hardware tuning, and the rejected/parked experiments.

## Benchmarks — sparq vs QLever

Benchmarked against **QLever** (a state-of-the-art engine) running natively on the same
machine (2020 MacBook Air M1, 16 GB), compute-only, cold, identical COUNT-wrapped queries.
Methodology, caveats, and per-query tables:
[`bench/qlever-baselines.md`](bench/qlever-baselines.md).

| workload | scale | sparq vs QLever |
|---|---|---|
| synthetic joins/OPTIONAL | 10M | **2.2–17× faster** |
| synthetic joins/OPTIONAL | 100M (in-memory) | **2.3–20× faster** |
| synthetic, **out-of-core (mmap)** | 100M | **2.6–20× faster**, ~0 committed heap, 0.67 s open |
| **real data (DBpedia olympics)** | 1.8M (skewed) | **1.7–12× faster** |

Selected per-query numbers (min-of-N, compute-only; QLever 0.5.47 native, reproduced):

| query | 10M QLever | 10M sparq | 100M QLever | 100M sparq | sparq advantage |
|---|--:|--:|--:|--:|--|
| 3-pattern star join | 73 ms | 14.5 ms | 900 ms | 158 ms | 5.0× / 5.7× |
| 2-pattern join | 54 ms | 24.8 ms | 600 ms | 266 ms | 2.2× / 2.3× |
| OPTIONAL | 59 ms | 3.4 ms | 650 ms | 33 ms | 17× / 20× |
| 1-pattern COUNT | 4 ms | 0.002 ms | 35 ms | 0.004 ms | range-count short-circuit |
| FILTER range | 3 ms | 0.005 ms | 8 ms | 0.005 ms | range-prune short-circuit |

The advantage holds across scale, real/synthetic data, and the in-memory vs out-of-core
paths — and the out-of-core (mmap) path matches or beats the in-memory query times with
near-zero committed heap, so QLever's low-RAM serving model is covered too.

**Ingestion at scale (real Wikidata):** 1,000,000,000 truthy triples → queryable
six-permutation out-of-core store in **737.8 s (1.355 M triples/s)** on an 8-vCPU
`r7i.2xlarge` (51.5 GB peak RSS, ~90 B/triple on disk; `COUNT(*)` over the result in 27 ms)
— [`research/wikidata-ingestion-benchmark.md`](research/wikidata-ingestion-benchmark.md).
Load thread-scaling reaches 2.99× @ 8 threads after the parallel dictionary-consolidation
work ([`research/dict-consolidation-verdict.md`](research/dict-consolidation-verdict.md)).

A 16 GB machine can reach the billion-triple regime too, with the dictionary as the wall:
the same 1B build on a 2-vCPU/16 GB `r8g.large` (Graviton4) completes in 4,691 s
(0.213 M triples/s) — swap-bound, because the term dictionary stays RAM-resident and grows
past 16 GB on a billion distinct terms, so a feasibility result, not a throughput one
([`research/wikidata-lowresource-stage1.md`](research/wikidata-lowresource-stage1.md)).

<!-- [OPUS-4.8] section written/updated by Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns -->
## Parsing

N-Triples / N-Quads use a **custom byte-level parser** that interns directly into the
dictionary with parallel newline-split chunking and a sharded dict merge; Turtle / TriG go
through `oxttl` under a custom statement-terminator chunker (`turtle_chunks`) that parallelises
even in the presence of blank nodes. Measured on the development machine (M1 Air, 4P+4E, 16 GB)
over a real 1.5M-triple Wikidata slice (`bench/parse/` harness, median of 3;
[`research/custom-parsers-baseline.md`](research/custom-parsers-baseline.md)):

| format | task | 1 thread | 8 threads | parallel speedup |
|---|---|--:|--:|--:|
| N-Triples | parse + intern | 234 MB/s (2.02 M/s) | 896 MB/s (7.76 M/s) | **3.84×** |
| N-Triples | full ingest (`load_str`, +6 perms) | 155 MB/s (1.34 M/s) | 585 MB/s (5.06 M/s) | 3.78× |
| Turtle | parse + intern | 0.998 s | 0.479 s | **2.1×** |
| Turtle | full ingest (`load_str`) | 1.207 s | 0.613 s | 2.0× |

The custom serial N-Triples parser (234 MB/s) already parses *and* interns faster than
`oxttl` parses-and-discards (185 MB/s, 1.27×). Turtle parses ~2.5× slower per byte than
N-Triples but its files are ~3.2× smaller, so per-*triple* the two are comparable at one
thread; the gap is parallelism (NT 5.06 vs Turtle 1.24 M/s at 8T full ingest). The Turtle
chunk-parallel path is shown byte-identical to serial `oxttl` on the real slice (1,500,000
statements, 41 distinct blank-node labels). RDF 1.2 / RDF-star terms parse but have no
standalone throughput figure yet.

**Streaming & compressed ingest.** Decompression overlaps parsing on a producer thread, so a
`.gz` / `.zst` stream is parsed without ever materialising the decompressed copy in RAM —
gzip-streaming ingest of the slice is **0.661 s** (8.5× faster than the pre-fix per-`read()`
flush) and zstd-streaming **0.576 s** (6.9×), both matching or beating decompress-to-RAM-then-parse
([`research/custom-parsers-baseline.md`](research/custom-parsers-baseline.md)). For the
external (out-of-core) build, overlapping bzip2 decode with parse lifts a `.bz2` build from
~0.39 to ~0.96 M triples/s (~2.4×), and parallelising the sibling-permutation sorts cuts a
10M external build 6.8 → 4.4 s (−35%)
([`research/fast-ingestion.md`](research/fast-ingestion.md)).

<!-- [OPUS-4.8] section written/updated by Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns -->
## Serialisation

The SPARQL-results JSON writer is chunk-parallel and the chunked path avoids the giant
single-string concat — a 1M-row scan returns its first chunk in 79 ms vs 305 ms for the
monolithic `query_json` (3.8×, a **memory/peak-RSS win**, not incremental delivery yet)
([`research/concurrent-serving.md`](research/concurrent-serving.md)). On a 302 MB JSON result
the writer alone runs at ~3.1 GB/s (8T)
([`research/custom-parsers-D4-compressed-serialization.md`](research/custom-parsers-D4-compressed-serialization.md)).
The per-cell JSON escaper was rewritten to bulk-copy already-safe runs, cutting a 701 MB
(5M-row) result's serialisation 1276 → 1034 ms (−19%)
([`research/BENCHMARKS.md`](research/BENCHMARKS.md)).

A `CompressedSink` frames the chunked output as multi-member gzip / multi-frame zstd so each
chunk compresses in parallel as it is produced. Measured on the same 302 MB JSON result
([`research/custom-parsers-D4-compressed-serialization.md`](research/custom-parsers-D4-compressed-serialization.md)):

| codec | output | ratio | parallel 8T throughput |
|---|--:|--:|--:|
| zstd −3 | 16.3 MB | **18.5×** | 6.5 GB/s |
| gzip −6 | 17.4 MB | 17.4× | 0.75 GB/s |
| gzip −1 | 22.6 MB | 13.4× | 3.8 GB/s |

zstd −3 strictly dominates gzip −6 (better ratio, ~8.7× faster parallel) and compresses
2.1× faster than the serializer produces, so overlapping it adds only ~8 ms to a 98 ms
serialization. On genuinely small (≤1 KiB) point-query responses a trained zstd vocabulary
dictionary takes the ratio from 2.3× to 11.4× (278 → 56 B per response). **Honesty caveat:**
the streaming overlap is measured by *replaying* committed chunks, not yet driven by a live
streaming server; and browsers / Node silently truncate multi-member `Content-Encoding: gzip`
and multi-frame zstd to the first member, so the shipped sink uses a single-member-gzip
variant (identical ratio, decodes everywhere) — both documented in
[`research/custom-parsers-D4-compressed-serialization.md`](research/custom-parsers-D4-compressed-serialization.md)
and [`research/custom-parsers-ADDENDUM-zstd-js-clients.md`](research/custom-parsers-ADDENDUM-zstd-js-clients.md).

**Honesty notes.** The synthetic dataset (uniform) favours simple joins; the skewed
real-data (olympics) and W3C-conformance results are the counterweight, and standard suites
(WatDiv/WDBench) remain on the list. QLever still scales a single index to the multi-billion
range where sparq's full-scale validation is pending bigger hardware. Every benchmark in the
table is reproducible from the committed scripts and stored baselines; subsystem-level
results (inference, updates, subscriptions, vectors, RSP) live in their crates' READMEs and
`research/` verdict files.

## License

MIT.
