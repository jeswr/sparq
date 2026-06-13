---
name: cli
description: Use when you need to drive the sparq RDF/SPARQL engine from the command line — load a Turtle/N-Triples/N-Quads/TriG (or HDT) file and run a SPARQL query, build/query memory-mapped on-disk indexes for datasets larger than RAM, materialize RDFS/OWL-RL/N3 reasoning closures, stream-ingest huge gzip/bzip2/zstd dumps, or benchmark query suites. Covers the actual `sparq-cli` subcommands, positional argument order, and cargo feature flags.
---

# sparq-cli

`sparq-cli` is the command-line front-end to the `sparq` RDF triplestore + SPARQL engine. It loads RDF files (with transparent gzip/bzip2/zstd decompression), runs SPARQL, builds and queries out-of-core memory-mapped indexes, and materializes reasoning closures (RDFS / OWL-RL / N3).

**Argument style (important):** the CLI uses a hand-rolled positional parser — there is *no* clap, no `--help`, and **no GNU-style flags except `--reason`/`--proof`**. The first token is the subcommand; the rest are positional and order matters. An unknown/missing subcommand prints a short usage block and exits with code 2.

## Quickstart

Run via cargo (the binary is `sparq-cli`; build with `--release` — debug builds are far slower):

```bash
# Load a Turtle file and run one query (prints the SOLUTION COUNT, not the bindings).
cargo run --release -p sparq-cli -- \
  query data.ttl turtle 'SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10'
# stderr: loaded N triples in 0.123s (...)
# stdout: 10 solutions in 0.456ms
```

`format` is one of `turtle | ntriples | nquads | trig` (aliases: `n-triples`, `n-quads`, `application/trig`). `nquads`/`trig` are loaded as a dataset so `GRAPH {}` works. Compressed inputs are auto-detected by extension (`.gz`, `.bz2`, `.zst`/`.zstd`) and streamed.

## Key APIs (subcommands)

All invoked as `sparq-cli <subcommand> <args...>`:

- `query <data-file> <format> <sparql> [--reason <rdfs|owl|n3>]` — load file, run one query, print `<n> solutions in <ms>ms` to stdout. **Only the count is printed**, not the result rows.
- `reason <data-file> <format> <rdfs|owl|n3> [out.nt]` — materialize the entailed closure; print closure triple count; with `out.nt`, write the full closure as N-Triples. Add `--proof` (N3 only) to print each derivation step.
- `build <file[.gz|.bz2|.zst]> <format> <dir> [chunk_millions=16]` — EXTERNAL-MEMORY build: stream the (compressed) document straight to on-disk memory-mapped indexes via disk-backed sort/merge. For datasets whose indexes exceed RAM. `chunk_millions` sets the in-memory run size.
- `save <data-file> <format> <dir> [compressed]` — load into RAM then persist the six permutation indexes to `<dir>`. Add the literal word `compressed` for block-compressed permutations.
- `query-mmap <dir> <sparql>` — open a saved/built dir with indexes MEMORY-MAPPED (out-of-core) and run a query (prints solution count). Permutations stay in the OS page cache, not the process heap.
- `recompress <src-dir> <dst-dir>` — re-persist a saved dir with block-compressed permutations without re-parsing (dirs must differ).
- `ingest <file[.gz|.bz2|.zst]> [parse|intern|full] [max_millions]` — streaming-throughput experiment over N-Triples: `parse` (decompress+parse+count), `intern` (+dictionary), `full` (+build indexes). Reports triples/s.
- `bench <data-file> <format> <queries-dir> [iters=5] [count|materialize|json]` — load once, run every `*.rq` in the dir (sorted) `iters` times, print TSV `<name>\t<rows>\t<min_micros>`. Mode default `materialize`.
- `bench-mmap <index-dir> <queries-dir> [iters=5] [count|materialize|json] [decompress]` — same as `bench` but opens the dataset out-of-core; trailing literal `decompress` decodes compressed perms to RAM first. Mode default `count`.
- `scaling <data-file> <format> <queries-dir> [threads=1,2,4,8,…] [iters=3]` — parallel-efficiency sweep across rayon pool sizes; TSV `subsystem\tthreads\tbest_ms\tspeedup\tefficiency`.
- `probe-compress <perm-file>` / `compare-compress <data-file> <format> [<sparql>]` / `bench-remap [n] [dict] [iters]` — measurement/instrumentation probes.

Underlying engine entry points the CLI calls (for reference): `sparq_engine::query(&Graph, &str) -> Result<QueryResult, String>`, `::count(...) -> Result<usize, String>`, `::query_json(...) -> Result<String, String>`. Loading goes through `sparq_core::Graph::{load_str, load_dataset, load_reader_parallel, build_external, open, save, save_compressed}`.

## Common recipes

**Build out-of-core indexes from a compressed dump, then query without loading into RAM:**
```bash
cargo run --release -p sparq-cli -- build dump.nt.zst ntriples ./idx
cargo run --release -p sparq-cli -- query-mmap ./idx \
  'SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }'
```

**Materialize an RDFS closure and write it out:**
```bash
cargo run --release -p sparq-cli -- reason ontology.ttl turtle rdfs closure.nt
```

**N3 forward-chaining with a proof trace:**
```bash
cargo run --release -p sparq-cli -- reason rules.n3 turtle n3 --proof
```
(For `n3` the `<format>` argument is ignored — the file is parsed as Notation3 facts+rules.)

**Query with reasoning applied first (OWL-RL); inconsistencies print to stderr):**
```bash
cargo run --release -p sparq-cli -- query data.ttl turtle \
  'SELECT ?s WHERE { ?s a ?c }' --reason owl
```

**Benchmark a query suite, JSON-serialization mode, 10 iterations:**
```bash
cargo run --release -p sparq-cli -- bench data.nt ntriples ./queries 10 json
```

**Load an HDT archive (requires the `hdt` feature):**
```bash
cargo run --release -p sparq-cli --features hdt -- \
  query graph.hdt hdt 'SELECT ?s ?p ?o WHERE { ?s ?p ?o }'
# .hdt / .hdt.gz extensions are auto-detected even if you pass a wrong format arg.
```

## Gotchas / feature flags / prerequisites

- **`query` prints a count, not rows.** stdout is `<n> solutions in <ms>ms`; load stats and reasoning/inconsistency reports go to stderr. To get serialized bindings use the `bench ... json` mode or call `sparq_engine::query_json` from the library.
- **No `--help`.** Run a subcommand with too few args to see its one-line usage; an unknown subcommand prints the top-level usage and exits 2. Query errors exit 1.
- **`--reason <profile>` is a flag on `query`** (scanned anywhere in argv); the standalone `reason` subcommand instead takes the profile as the 3rd positional. Profiles: `rdfs`, `owl`, `n3`.
- **Default cargo features:** `mmap` (out-of-core), `mimalloc` (global allocator — `--no-default-features --features mmap` falls back to the system allocator for A/B), `dict-spill`.
- **HDT is opt-in:** `--features hdt`. It is OFF by default partly because the wrapped `hdt` crate raises MSRV to **1.87** (workspace floor is 1.85). HDT is read-only ingestion.
- **External-build env vars (native, `dict-spill` feature):** `SPARQ_DICT_SPILL=1` spills the term dictionary during `build` (N-Triples only) to bound peak RSS; tune with `SPARQ_DICT_SPILL_BUDGET_MB` (default ¼ RAM) and `SPARQ_DICT_SPILL_DISK_FLOOR_MB` (default 1024, aborts before filling disk). Output is byte-identical. Also `SPARQ_NO_PREFETCH=1` for the `bench-remap` probe.
- **Format ↔ ingest path:** N-Triples streams block-by-block (parallel parse, no full decompressed copy in RAM); Turtle/N-Quads/TriG are buffered whole for the parallel statement-splitter. zstd decompresses ~12× faster than bzip2 — recompress `.bz2` sources once with `zstd -9 -T0` for big ingests.
- **Compressed-perm dirs** written by `save ... compressed` / `recompress` are auto-detected by `query-mmap`/`bench-mmap`; `bench-mmap ... decompress` decodes them to RAM first.

## See also

- `core` — the `sparq_core::Graph` library API (load/save/open, the store, the Dict) used under the hood.
- `engine` — `sparq_engine::{query, count, query_json}` and query budgets / prepared queries.
- `reason` — RDFS/OWL-RL/N3 materialization (`sparq_reason`) invoked by `--reason` and the `reason` subcommand.
- `hdt` — the opt-in HDT loader (`sparq_hdt::load`).
- `server` — the HTTP SPARQL endpoint (`sparq-server`) for serving instead of one-shot CLI queries.
