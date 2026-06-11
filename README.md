# sparq

A from-scratch **RDF triplestore and SPARQL engine in Rust** — dictionary-encoded, with six
sorted permutation indexes, parallel execution, RDFS/OWL-RL/N3 inference, an out-of-core
(memory-mapped) mode, a WebAssembly build, and a W3C-conformant HTTP server.

> **Status: experimental research engine.** Tested against the full W3C SPARQL suites —
> **1229 of 1229 tests accounted for: 1225 pass, zero fail, zero skips**, plus 4 documented
> divergences where the suite's expected files are provably wrong against the spec and
> their own approved siblings (rendered with full rationale in the conformance report;
> rdf-tests issue drafts in [`docs/upstream-proposals.md`](docs/upstream-proposals.md)).
> Six former upstream-parser failures are fixed in a vendored spargebra
> ([`vendor/spargebra/SPARQ-PATCHES.md`](vendor/spargebra/SPARQ-PATCHES.md), upstream PRs
> drafted). Catalogue: [`crates/sparq-conformance/FINDINGS.md`](crates/sparq-conformance/FINDINGS.md).
> The API is still unstable; SERVICE federation remains unimplemented
> (see [`research/roadmap.md`](research/roadmap.md)).

## Why it exists

To find out how fast a clean, modern RDF engine can be. The design follows the dictionary-encoded
+ permutation-index lineage (Hexastore / RDF-3X / QLever), but the store, planner, and physical
execution are written from scratch with an emphasis on parallelism and a memory-bounded
out-of-core path.

## Measured results

Benchmarked against **QLever** (a state-of-the-art engine) running natively on the same machine,
compute-only (cold), on synthetic and real data — full numbers in
[`bench/qlever-baselines.md`](bench/qlever-baselines.md):

| workload | scale | sparq vs QLever |
|---|---|---|
| synthetic joins/OPTIONAL | 10M | **2.2–17× faster** |
| synthetic joins/OPTIONAL | 100M (in-memory) | **2.3–20× faster** |
| synthetic, **out-of-core (mmap)** | 100M | **2.6–20× faster**, ~0 committed heap, 0.67 s open |
| **real data (DBpedia olympics)** | 1.8M (skewed) | **1.7–12× faster** |

The advantage holds across scale, real/synthetic data, and the in-memory vs out-of-core paths.
**Ingestion** of real Wikidata is competitive per-core with QLever (~1.3 M triples/s building all
six permutations out-of-core on a fanless laptop). **Inference**: single-pass RDFS materialization
(~17× over a fixpoint), an OWL-RL fast path (~30×), and union-find `owl:sameAs`. Hardware-specific
tuning (a per-ISA software prefetch) is measured and selected per silicon family (Apple / Intel /
AMD / Graviton) — see [`research/hw-bench-results.md`](research/hw-bench-results.md).

## Features

- **Parsing** — N-Triples, Turtle, N-Quads, TriG; parallel; streaming + transparently
  decompressing (`.gz` / `.bz2` / `.zst`) with no full-document copy in RAM.
- **SPARQL query** — SELECT/ASK, Basic Graph Patterns, FILTER (full built-in function set incl.
  REGEX), OPTIONAL, UNION, MINUS, VALUES, BIND, aggregation + GROUP BY, ORDER BY, DISTINCT,
  LIMIT/OFFSET, **property paths** (all 8 operators), **named graphs** (`GRAPH`, N-Quads/TriG
  datasets); sort-merge + hash + worst-case-optimal joins; greedy cardinality-based planning.
  Parallel scan / filter / sort / join-build / aggregation / serialization.
- **SPARQL Update** — the complete operation set (data ops, `DELETE/INSERT … WHERE` with GRAPH
  templates, `USING`, `LOAD`, `CLEAR`/`DROP`/`CREATE`/`COPY`/`MOVE`/`ADD`) over default and named
  graphs — **100% of the W3C update suite**. Updates are **incremental** (delta overlay +
  append-only dictionary, ~10⁶× faster than rebuild) with an optional **write-ahead log** for
  durability, and run end-to-end over HTTP in ~300 µs.
- **Subscriptions** — SEPA-style WebSocket subscriptions: register a SELECT, receive
  added/removed binding diffs after every committed update
  ([`crates/sparq-server/SUBSCRIPTIONS.md`](crates/sparq-server/SUBSCRIPTIONS.md)).
- **RDF 1.2 / RDF-star** — triple terms (`<< s p o >>`) stored structurally in the dictionary;
  concrete triple-term patterns match; SPARQL 1.2 JSON/XML result rendering.
- **Inference** (opt-in `sparq-reason`) — RDFS, OWL-RL, and a Notation3 engine (EYE-validated
  builtins, goal-directed `<=`), with single-pass materialization and **incremental closure
  maintenance** (counting-based; deltas ~10³–10⁴× faster than re-materialization).
- **Validation** (opt-in `sparq-shacl`) — SHACL Core at **100% of the W3C core suite**.
- **Solid access control** (opt-in `sparq-solid`) — [Solid](https://solidproject.org/) Pods
  stored as named-graph-per-document; [WAC](https://solidproject.org/TR/wac)/[ACP](https://solidproject.org/TR/acp)
  semantics run as N3 rules materializing a queryable, triples-native authorization view;
  SPARQL filtered per (WebID, client) session, fail-closed
  ([`crates/sparq-solid/README.md`](crates/sparq-solid/README.md), design:
  [`research/solid-access-control-design.md`](research/solid-access-control-design.md)).
- **More opt-in crates** — `sparq-geo` (GeoSPARQL relations + R-tree index), `sparq-hdt` (HDT
  archives), `sparq-sim` (training-free structural similarity), `sparq-introspect`
  (characteristic sets + LLM-ready schema digests), `sparq-py` (Python bindings), and an
  RDF/JS-typed npm package under [`js/`](js/).
- **Out-of-core** — build on-disk indexes and query them **memory-mapped**, so billion-triple
  datasets are queryable in a fraction of the RAM.
- **HTTP server** (`sparq-server`) — W3C SPARQL 1.1 Protocol query + update endpoints, Graph
  Store read, result formats JSON / XML / CSV / TSV; content negotiation; **EXPLAIN /
  EXPLAIN ANALYZE** plan introspection (`?explain=…`) and a **Prometheus `/metrics`**
  endpoint (see [`crates/sparq-server/README.md`](crates/sparq-server/README.md)). Docker
  image via the release workflow (see [`docs/release.md`](docs/release.md)).
- **WebAssembly** — the core engine builds for the browser with a minimal bundle.

## Workspace

| crate | what |
|---|---|
| `sparq-core` | dictionary, six permutation indexes, parallel + streaming loaders, on-disk/mmap store |
| `sparq-engine` | SPARQL algebra → physical plan → parallel execution + result serialization |
| `sparq-reason` | RDFS / OWL-RL / N3 inference (opt-in) |
| `sparq-cli` | command-line loader / query / benchmark runner |
| `sparq-server` | W3C SPARQL HTTP server |
| `sparq-wasm` | WebAssembly bindings |
| `sparq-bench` | dataset generation, fuzzing, scaling harness |

## Quick start

```sh
# build
cargo build --release

# run a query over a file (Turtle / N-Triples / N-Quads / TriG, optionally .gz/.bz2/.zst)
cargo run --release -p sparq-cli -- query data.ttl turtle \
  'SELECT ?s ?o WHERE { ?s <http://schema.org/name> ?o } LIMIT 10'

# build on-disk indexes once, then query them memory-mapped (out-of-core)
cargo run --release -p sparq-cli -- build data.nt ntriples ./idx
cargo run --release -p sparq-cli -- query-mmap ./idx 'SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }'

# RDFS / OWL-RL materialization
cargo run --release -p sparq-cli -- reason ontology.ttl turtle rdfs

# HTTP server (W3C SPARQL Protocol) on :3030
cargo run --release -p sparq-server -- --addr 127.0.0.1:3030 --format turtle data.ttl

# per-subsystem parallel scaling sweep
cargo run --release -p sparq-cli -- scaling data.nt ntriples queries/ 1,2,4,8
```

## Documentation

- [`research/ARCHITECTURE.md`](research/ARCHITECTURE.md) — design blueprint.
- [`research/roadmap.md`](research/roadmap.md) — what's done and what's planned (with a
  dependency graph of the work threads).
- [`bench/qlever-baselines.md`](bench/qlever-baselines.md) — the QLever comparison methodology
  and numbers.
- `research/` — design notes on indexing, compression, inference, parallelism, RDF 1.2, and more.

## License

MIT.
