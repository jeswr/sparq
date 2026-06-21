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
- **Incremental change-stream / PITR** (same `backup` feature) — `backup_delta::export_delta`
  captures the quad-set change between two **same-lineage** generations as a self-describing
  delta artifact keyed off the generation/writer-seq range; `backup_delta::replay` applies an
  ordered chain forward onto a restored base to reach a chosen recovery point (fail-closed on a
  corrupt or discontinuous chain). `sparq-server` adds `/admin/backup/delta?from=N` +
  `--restore-delta` for point-in-time recovery.
- **Response-bytes result cache** *(opt-in: `--features result-cache`, OFF by
  default)* — see below.

## 🗃️ Result cache (opt-in, `result-cache` feature)

A serving-layer cache from a request *identity* to the complete pre-serialized
response body, so a repeated read returns bytes in tens of nanoseconds instead of
re-executing. **OFF by default**; the default build carries zero cache code and no
extra dependency.

- **Key = (canonical-query × visibility-scope × per-pod epoch-vector)** (design
  §6.3). The query is cheaply canonicalized (whitespace, and opt-in variable
  renaming). The **visibility scope** is the identity of the *accessible graph set*
  a request runs under — derive a `ScopeKey` from
  `sparq_solid::AuthIndex::accessible(session, mode)`, **never** from the WebID
  (the Hasura lesson). Many WebIDs that share one public-read scope collapse to one
  key.
- **Access-control isolation is correctness, not privacy.** Bytes cached for one
  scope can never be served to a different scope (a different scope MUST miss —
  tested). This enforces the access-control boundary the auth layer defines; it is
  **not itself a confidentiality/privacy guarantee** (no cryptographic claim; it
  trusts a faithfully-derived scope key).
- **Invalidation = per-pod (per-named-graph) epoch bumps.** Each entry records the
  epoch of the graphs its query touched; a write to any of them makes it stale.
  Queries with an unbounded read footprint pin the global generation (invalidated
  by any write).
- **Single-flight leases** collapse a stampede on a hot uncached key into one
  execution + N waiters. **Byte-budget LRU + admission**: oversize/streaming bodies
  are never cached.

The cache stores opaque `Arc<[u8]>` bodies and a caller-derived `ScopeKey`; it never
depends on `sparq-solid` and never parses a query. It is a **different layer** from
`sparq-engine`'s embedded `result-cache` (the in-engine algebra-keyed LRU). Perf
targets (design §6.3) require a canonical host and are validated there, not in-tree.

## 📚 Learn more

- **Design** — [`research/concurrent-serving.md`](../../research/concurrent-serving.md) §6.
- **API reference** — [docs.rs/sparq-serve](https://docs.rs/sparq-serve).
- **Consumer** — [`sparq-server`](../sparq-server/README.md) (the HTTP wrapper).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
