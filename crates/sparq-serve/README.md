<!-- [OPUS-4.8] sq-4kr5: internal-stub README for a publish=false crate. -->
# sparq-serve

The **concurrent-serving core** of [sparq](../../README.md): a lock-free
*generation ring* — an arc-swapped chain of immutable store snapshots with
bounded retention and per-pod epoch vectors — plus the single **sequenced
writer** with group-commit batching that publishes those snapshots.

Why it exists: readers load the current generation in tens of nanoseconds and
**never block the writer**, and the writer **never waits for readers** or
reclaims in place — old generations are freed by ordinary `Arc` drop. This
replaced the double-buffered snapshot scheme whose pinned-snapshot writer stalls
and reclaim-poll degradation motivated the redesign.

The crate is **sync, runtime-agnostic, and library-first**: it exposes no HTTP
or async-runtime types (consumers such as `sparq-server` wrap it), and it must
never enter `sparq-wasm`'s dependency graph.

> **Internal crate — not published** to crates.io (`publish = false`). It is
> wrapped by `sparq-server`; there is no standalone public API surface.

Design: [`research/concurrent-serving.md`](../../research/concurrent-serving.md) §6.
Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
