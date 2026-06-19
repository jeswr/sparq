<!-- [OPUS-4.8] sq-2te / sq-v7be — upstream-contribution status for KonradHoeffner/hdt. -->
# Upstream contributions to `KonradHoeffner/hdt`

This crate WRAPS [`hdt`](https://github.com/KonradHoeffner/hdt) (MIT). This file
tracks the two upstream gaps the sparq-hdt work originally queued against it and
their CURRENT status against `hdt` master / 0.6.x.

> **Status (sq-v7be, verified 2026-06-19 against `KonradHoeffner/hdt` master):**
> - **Builder gap — OBVIATED (no upstream change needed from us).** See item 1.
> - **Decode-only entry point — OPEN UPSTREAM DRAFT PR
>   [`KonradHoeffner/hdt#124`](https://github.com/KonradHoeffner/hdt/pull/124).**
>   See item 2.

---

## Item 1 — in-memory section builders without `sophia` — OBVIATED

**Original ask (sq-ashy):** make the in-memory section builders usable without
pulling the `sophia` adapter dependency tree, so sparq could WRITE a `.hdt` from
its own in-memory dict + triples with no N-Triples text round-trip.

**Why it is obviated.** On current `hdt` master the section builders sparq needs
are already `pub` and **not** gated behind `sophia`:

- `DictSectPFC::compress(&BTreeSet<&str>, block_size) -> DictSectPFC` — gate-free.
- `TriplesBitmap::from_triples(&[TripleId]) -> TriplesBitmap` — gate-free.

Upstream also split the N-Triples ingest path (`lasso` / `oxttl`) out into its own
`nt` feature, so the `sophia` term adapter is no longer dragged in just to reach
the builders. (Note: there is **no** `Hdt::from_triples` constructor on the `Hdt`
struct itself — the in-memory build is done at the section level via the two
builders above, which is what sparq does.)

sparq already builds + writes a spec-conformant archive directly from these:
`sparq-hdt/src/encode.rs` calls `DictSectPFC::compress` per FourSectDict section and
`TriplesBitmap::from_triples` for the SPO bitmaps, with **no** N-Triples text
round-trip (the `save` path — `crates/sparq-hdt/src/write.rs` — and the
`encode.rs`/`decode.rs` round-trip oracle confirm this). So no upstream change is
required for the write path; this item is closed (tracked under landed bead
`sq-ashy`).

> **Caveat — sparq is still pinned to `hdt` 0.4.** On the wrapped crate's **0.4**
> line these section builders are reachable only via the experimental `sophia`
> feature, so sparq-hdt's `write` cargo-feature still turns it on
> (`Cargo.toml: write = ["hdt/sophia"]`); the `sophia`-free reachability described
> above is the situation on `hdt` **master / 0.6**. The 0.4 → 0.6 bump is itself
> blocked (the 0.6 path pulls `qwt`, whose default `prefetch` feature needs nightly
> on aarch64) — tracked by `sq-2l1` / `sq-th5i`. So the *upstream contribution* is
> obviated (we never need to file it), but sparq keeps paying the `sophia` gate for
> writes until that dependency bump lands.

---

## Item 2 — decode-only / streaming-triples entry point — OPEN UPSTREAM DRAFT PR

**Status:** open **draft** PR
[`KonradHoeffner/hdt#124`](https://github.com/KonradHoeffner/hdt/pull/124)
(`feat: decode-only Hdt::triples_streaming (skip query-index build on bulk reads)`),
authored by `@jeswr`. It is **jeswr-review-gated** — not yet marked ready for
maintainer review — so it is not merged. Tracking bead: `sq-fkj`.

**What it adds.** `Hdt::read` eagerly calls `TriplesBitmap::new`, which builds —
purely to serve triple-pattern / object / predicate QUERIES — a `WaveletMatrix`,
a per-object `Vec<Vec<u32>>`, a `sort_by_cached_key`, and an OP-index
(`CompactVector` + `Rank9Sel` bitmap). A consumer doing a one-shot bulk load (read
every triple once, in SPO order, into its own store) never issues those queries, so
all of that is built and immediately dropped — a large, cache-hostile cost on
ingest. The PR adds a decode-only entry point that reads the dictionary +
`bitmap_y` / `bitmap_z` / `sequence_y` / `sequence_z` and yields triples in SPO
order WITHOUT constructing the `TriplesBitmap` query structures, e.g.

```rust
impl Hdt {
    /// Decode-only: stream every triple in SPO order without building the
    /// wavelet matrix / OP-index used for pattern queries.
    pub fn triples_streaming<R: BufRead>(reader: R)
        -> Result<impl Iterator<Item = Result<[usize; 3]>>>;
}
```

**Reference implementation.** `sparq-hdt/src/decode.rs` already does exactly this —
it is the DEFAULT load path (`decode::graph_from_reader`, driven by the public
`load_reader` in `lib.rs`): it reads the same on-disk bytes, walks the bitmaps with
a plain bit read, and skips the rank/select build. It is differentially tested
against the full `Hdt::read` path (retained only as the oracle,
`load_reader_via_upstream` in `lib.rs`) on real and generated archives. If the
upstream PR lands we can delete our vendored decoder and call the upstream entry
point instead.
