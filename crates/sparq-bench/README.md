<!-- [OPUS-4.8] sq-4kr5: internal-stub README for a publish=false crate. -->
# sparq-bench

The **benchmark + differential harness** for [sparq](../../README.md): it runs
the same dataset and queries through `sparq` and through
[Oxigraph](https://github.com/oxigraph/oxigraph) (a mature, independent Rust
SPARQL engine), cross-checks that both return the same number of solutions, and
reports load/query timings and peak memory.

Why it exists: it serves a correctness role as well as a speed one — the
solution-count cross-check against an independent implementation is a cheap,
continuous oracle that catches engine regressions, while the timings feed the
benchmarks dashboard.

> **Internal tooling — not published** to crates.io (`publish = false`). Run it
> as a workspace binary; it is not a library. Measured numbers belong in the
> [benchmarks dashboard](https://sparq.jeswr.org/dev/bench), never baked
> into docs.

Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
