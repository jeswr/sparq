<!-- [OPUS-4.8] sq-4kr5: README for the library-internal serving core. It is
     PUBLISHABLE (no `publish = false`) only because the published `sparq-server`
     depends on it — a crates.io crate cannot depend on a `publish = false` crate;
     it has no standalone public API surface of its own. Keep this in sync with
     crates/sparq-serve/Cargo.toml. -->
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

> **Library-internal core, not a standalone surface.** It is wrapped by
> `sparq-server` and has no public API of its own — the [surface map in
> `AGENTS.md`](../../AGENTS.md) does not list it as a usage entry point. It is
> nonetheless a *publishable* crate (its `Cargo.toml` does **not** set
> `publish = false`), and it must stay that way: the published `sparq-server`
> depends on it, and a crates.io crate cannot depend on a `publish = false`
> crate. So it ships to crates.io as plumbing for `sparq-server`, not as an
> independently useful library.

Design: [`research/concurrent-serving.md`](../../research/concurrent-serving.md) §6.
Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
