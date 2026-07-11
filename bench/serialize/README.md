<!-- [FABLE-5] sq-hmd7l.14 — serialization-throughput comparison panel. -->

# bench/serialize — serialization-throughput comparison panel

Compares sparq's RDF writer matrix (`sparq_engine::serialize::*`, driven through
the PUBLIC API via `crates/sparq-engine-serialize/examples/serialize_bench.rs`)
against **serd** (`serdi`), **Redland Raptor** (`rapper`), **Apache Jena**
(`riot`) and an **oxrdfio** scratch shim. Registered as `serialize-bench` in
[`bench/benchmarks.toml`](../benchmarks.toml); first-read analysis in
[`research/gap-serialize-2026-07.md`](../../research/gap-serialize-2026-07.md).

No standard serialization suite exists, so the honest form is corpus-based
throughput with a **round-trip gate**: every emitted document must re-parse to
exactly the store it came from (triple count + exact canonical N-Quads line
match) **before** its timing row is trusted. The generated corpus is
blank-node-free by construction so the gate is an exact set comparison, not
graph isomorphism (that axis is `canon-bench`'s business). The gate proves it
can fail on every run: a deliberately corrupted document must red it (PHASE B).

## Run

```sh
bench/serialize/run.sh --smoke   # sparq-only: nt+turtle(+stream) gates green +
                                 # the negative self-check; exit 0 = green
bench/serialize/run.sh           # full panel (all six sparq formats + peers)
```

Peers are gather-time only — never committed dependencies. `serdi` / `rapper` /
`riot` are picked up from `PATH` (or `SERIALIZE_SERDI` / `SERIALIZE_RAPPER` /
`SERIALIZE_RIOT`); an absent tool is an honest `absent` row, never a failure.
`SERIALIZE_OXRDFIO_BIN=auto` builds the oxrdfio shim via
[`scripts/bench-adapters/serialize_oxrdfio_adapter.sh`](../../scripts/bench-adapters/serialize_oxrdfio_adapter.sh)
(pinned to the same oxrdf/oxttl family the workspace uses).

## The two regimes (never cross-compare them)

- **In-process serialize-only** (PHASE A, sparq only): store loaded ONCE, each
  format timed min-of-`SERIALIZE_ITERS`; the buffered vs streaming vs pretty
  rows compare sparq's own writer regimes. MB/s here is over *output* bytes.
- **Pipeline** (PHASE C, cross-engine): read N-Triples → write
  `{turtle, ntriples}` as ONE process, min-of-`SERIALIZE_PIPE_ITERS` process
  walls, every engine measured under the same stopwatch (sparq runs its own
  `pipe` mode). The cross-comparable number is **`mbps_in`** (input corpus MB /
  wall); output byte counts differ per engine (prefix policies differ), so
  per-engine output MB/s is not cross-comparable and is not printed. A `riot`
  wall includes JVM startup — labelled in the envelope, not subtracted.

A competitor whose output fails the gate gets a recorded `red(rc=4)` outcome
and NO timing row (e.g. rapper's Turtle writer rewrites `xsd:double` literals
into bare decimal syntax, which re-parses as `xsd:decimal`). Only a *sparq*
gate failure reds the panel.

Scope: JSON-LD serialization is compared on the `jsonld-bench` axis
(sq-hmd7l.15), not here. RDF/XML has no sparq writer and no in-repo re-parse
oracle, so it is out of scope (recorded in the gap record). SPARQL
result-set serialization (JSON/XML/CSV) is a separate follow-up axis.

Set `SERIALIZE_JSON_OUT=bench/competitor-results/<name>.json` (git-ignored) to
capture the machine-readable envelope (host + engine versions + both regimes).
Work-box timings are NON-canonical — see the QUIET-BOX convention in
[`bench/CATALOG.md`](../CATALOG.md); numbers are never committed to markdown.
