# sparq

A from-scratch **RDF triplestore and SPARQL engine in Rust** — dictionary-encoded, with six
sorted permutation indexes, parallel execution, RDFS/OWL-RL/N3 inference, an out-of-core
(memory-mapped) mode, a WebAssembly build, and a W3C-conformant HTTP server.

> **Status: experimental research engine.** The query/build/inference paths are tested and
> benchmarked (see below), but the API is unstable and several SPARQL features are still in
> progress (named graphs, property paths, SERVICE, UPDATE — see [`research/roadmap.md`](research/roadmap.md)).

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
- **SPARQL query** — SELECT/ASK, Basic Graph Patterns, FILTER, OPTIONAL, UNION, MINUS, VALUES,
  BIND, aggregation + GROUP BY, ORDER BY, DISTINCT, LIMIT/OFFSET; sort-merge + hash + worst-case-
  optimal joins; greedy cardinality-based planning. Parallel scan / filter / sort / aggregation /
  serialization.
- **Inference** (opt-in `sparq-reason`) — RDFS, OWL-RL, and a Notation3 subset, with
  saturate-then-sweep single-pass materialization.
- **Out-of-core** — build on-disk indexes and query them **memory-mapped**, so billion-triple
  datasets are queryable in a fraction of the RAM.
- **HTTP server** (`sparq-server`) — W3C SPARQL 1.1 Protocol query endpoint (GET/POST), Graph
  Store read, result formats JSON / XML / CSV / TSV; content negotiation.
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
