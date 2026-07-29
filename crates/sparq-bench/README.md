<!-- [OPUS-4.8] sq-4kr5: internal-stub README for a publish=false crate. -->
# sparq-bench

The **benchmark + differential harness** for [sparq](../../README.md): it runs
the same dataset and queries through `sparq` and through
[Oxigraph](https://github.com/oxigraph/oxigraph) (a mature, independent Rust
SPARQL engine), cross-checks that both return the same number of solutions, and
reports load/query timings and peak memory.

Why it exists: it serves a correctness role as well as a speed one — the
cross-check against an independent implementation is a cheap, continuous
oracle that catches engine regressions, while the timings feed the benchmarks
dashboard. Two differential fuzzers live here: `fuzz` (queries; nightly
`differential.yml`) and `update-fuzz` (SPARQL UPDATE sequences — ground terms,
non-canonical numeric lexicals, blank nodes under RDFC-1.0 isomorphism, `LOAD`,
RDF-1.2 triple terms — both sparq update paths vs Oxigraph, canonical per-step compare;
nightly `differential-update.yml`). Oracles are pluggable (`src/oracle.rs`), with an
opt-in second, independent engine in [`oracles/`](./oracles/README.md); they and the
adjudicated divergence classes in `bench/differential-divergences.json` are documented in the module headers.

> **Internal tooling — not published** to crates.io (`publish = false`). Run it
> as a workspace binary; it is not a library. Measured numbers belong in the
> [benchmarks dashboard](https://sparq.jeswr.org/dev/bench), never baked
> into docs.

Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
