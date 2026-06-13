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

<!-- [OPUS-4.8] section written/updated by Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns -->
## Using sparq with AI agents

sparq ships its usage docs as [Agent Skills](https://agentskills.io) (the open `SKILL.md`
format) so a coding agent can discover how to *use* the engine across every surface. Start with
[`AGENTS.md`](AGENTS.md) (a README for agents), then [`skills/SKILL.md`](skills/SKILL.md) — the
router that points at the per-surface skill (Rust, CLI, HTTP, Python, JS/WASM, plus reasoning,
SHACL, full-text, vector, GeoSPARQL, RSP-QL, ZK query proofs). In Claude Code, install them as a
plugin: `/plugin marketplace add jeswr/sparq` then `/plugin install sparq@sparq-tools`.

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

## Performance

Per [AGENTS.md](AGENTS.md), benchmark figures are not baked into the docs — they
drift. The live numbers come from the generated data:

- **Per-commit metrics** (wasm bundle bytes, store B/triple, dict B/term) →
  the dashboard at <https://jeswr.github.io/sparq/dev/bench> (orphan
  `benchmark-data` branch, updated by CI).
- **The benchmark registry + exact invocations** → [`bench/benchmarks.toml`](bench/benchmarks.toml)
  and [`bench/CATALOG.md`](bench/CATALOG.md). Run the named harness to reproduce any figure.

### sparq vs QLever

Benchmarked against **QLever** (a state-of-the-art engine) running natively on the
same machine, compute-only, cold, identical COUNT-wrapped queries. sparq matches or
beats native QLever on compute across synthetic and real (DBpedia olympics) data, at
10M and 100M triples, in-memory and out-of-core — and the out-of-core (mmap) path
holds that with near-zero committed heap (QLever's low-RAM serving model covered too).
Methodology, caveats, the per-query baselines, and how to reproduce them are
single-sourced in [`bench/qlever-baselines.md`](bench/qlever-baselines.md); run
`bench/qlever-olympics/compare.py` / `bench/qlever-synthetic/` for live numbers.

### Ingestion at scale (real Wikidata)

sparq builds a queryable six-permutation out-of-core store from 1,000,000,000 truthy
Wikidata triples on a single 8-vCPU box — the run log, hardware, throughput, peak RSS,
and the low-resource (16 GB, swap-bound) feasibility variant are single-sourced in
[`research/wikidata-ingestion-benchmark.md`](research/wikidata-ingestion-benchmark.md)
and [`research/wikidata-lowresource-stage1.md`](research/wikidata-lowresource-stage1.md).
Load thread-scaling after the parallel dictionary-consolidation work is in
[`research/dict-consolidation-verdict.md`](research/dict-consolidation-verdict.md).

<!-- [OPUS-4.8] section written/updated by Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns -->
## Parsing

N-Triples / N-Quads use a **custom byte-level parser** that interns directly into the
dictionary with parallel newline-split chunking and a sharded dict merge; Turtle / TriG go
through `oxttl` under a custom statement-terminator chunker (`turtle_chunks`) that parallelises
even in the presence of blank nodes. The custom serial N-Triples parser parses *and* interns
faster than `oxttl` parses-and-discards; Turtle parses slower per byte but its files are
smaller, so per-*triple* the two are comparable at one thread and the gap is parallelism. The
Turtle chunk-parallel path is shown byte-identical to serial `oxttl` on the real slice
(1,500,000 statements, 41 distinct blank-node labels). RDF 1.2 / RDF-star terms parse but have
no standalone throughput figure yet.

The parse/ingest throughput figures (per-thread + parallel speedup, N-Triples and Turtle)
are measured by the `bench/parse/` harness and single-sourced in
[`research/custom-parsers-baseline.md`](research/custom-parsers-baseline.md); run that
harness for live numbers (see [`bench/CATALOG.md`](bench/CATALOG.md)).

**Streaming & compressed ingest.** Decompression overlaps parsing on a producer thread, so a
`.gz` / `.zst` stream is parsed without ever materialising the decompressed copy in RAM, both
matching or beating decompress-to-RAM-then-parse
([`research/custom-parsers-baseline.md`](research/custom-parsers-baseline.md)). For the
external (out-of-core) build, overlapping bzip2 decode with parse and parallelising the
sibling-permutation sorts are measured in
[`research/fast-ingestion.md`](research/fast-ingestion.md).

<!-- [OPUS-4.8] section written/updated by Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns -->
## Serialisation

The SPARQL-results JSON writer is chunk-parallel and the chunked path avoids the giant
single-string concat — returning its first chunk far sooner than the monolithic `query_json`
(a **memory/peak-RSS win**, not incremental delivery yet)
([`research/concurrent-serving.md`](research/concurrent-serving.md)). The per-cell JSON
escaper bulk-copies already-safe runs rather than emitting char-by-char
([`research/BENCHMARKS.md`](research/BENCHMARKS.md)). Writer throughput and escaper figures
are measured by the `bench/parse/` compress-bench harness, single-sourced in
[`research/custom-parsers-D4-compressed-serialization.md`](research/custom-parsers-D4-compressed-serialization.md).

A `CompressedSink` frames the chunked output as multi-member gzip / multi-frame zstd so each
chunk compresses in parallel as it is produced. The codec comparison (compression ratio +
parallel throughput for zstd / gzip, and the trained-dictionary win on small point-query
responses) is measured by the compress-bench harness and single-sourced in
[`research/custom-parsers-D4-compressed-serialization.md`](research/custom-parsers-D4-compressed-serialization.md);
zstd −3 strictly dominates gzip −6 (better ratio, faster parallel) and compresses faster than
the serializer produces, so overlapping it adds negligible latency. **Honesty caveat:**
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
