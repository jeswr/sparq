<!-- [OPUS-4.8] sq-4kr5: README for the library-internal serving core. It is
     PUBLISHABLE (no `publish = false`) only because the published `sparq-server`
     depends on it — a crates.io crate cannot depend on a `publish = false` crate;
     it has no standalone public API surface of its own. Keep this in sync with
     crates/sparq-serve/Cargo.toml. -->
<!-- [OPUS-4.8] sq-inzv: README brought to template. -->
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

## 🚀 Quickstart

This crate has no standalone surface — use it through
[`sparq-server`](../sparq-server/README.md), which wraps the generation ring and
sequenced writer behind the HTTP endpoint:

```sh
cargo run -p sparq-server -- --format turtle data.ttl
```

## ✨ Features

- **Lock-free generation ring** — readers pin the current immutable snapshot in
  tens of nanoseconds and never block the writer; old generations are freed by
  ordinary `Arc` drop (no in-place reclaim, no reclaim-poll degradation).
- **Single sequenced writer** — group-commit batching publishes each batch as
  one new immutable generation; serialisability is by construction.
- **Sync, runtime-agnostic, library-first** — no HTTP or async-runtime types;
  consumers wrap it. It must never enter `sparq-wasm`'s dependency graph.
- **Online backup/restore** (opt-in `backup` feature, default OFF) — `backup::export`
  serialises an already-immutable pinned `Generation` (triples + per-pod epoch vectors +
  writer seq) to one self-describing artifact **while serving** (no stop-the-world);
  `backup::import` re-hydrates a `Graph` from one, **fail-closed** on a corrupt/mismatched
  artifact. Distinct from the offline `sparq-cli save` and the `--persist` WAL. At-rest
  encryption is out of scope. `sparq-server` mounts `/admin/backup` + `/admin/restore` on it.

## 📚 Learn more

- **Design** — [`research/concurrent-serving.md`](../../research/concurrent-serving.md) §6.
- **API reference** — [docs.rs/sparq-serve](https://docs.rs/sparq-serve).
- **Consumer** — [`sparq-server`](../sparq-server/README.md) (the HTTP wrapper).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
