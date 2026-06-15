<!-- [OPUS-4.8] sq-2te — upstream contributions queued against KonradHoeffner/hdt. -->
# Upstream contributions to `KonradHoeffner/hdt`

This crate WRAPS [`hdt`](https://github.com/KonradHoeffner/hdt) (MIT). One API
gap in the wrapped crate (as of `hdt` 0.4) makes our `save` / `load` paths pull a
dependency they should not have to. It is a tractable upstream addition; the text
below is ready to file.

> **Update (sq-ashy):** the temp-N-Triples round-trip in `write.rs` is GONE. `save`
> now encodes the FourSectDict PFC + BitmapTriples sections DIRECTLY from sparq's
> in-memory dict + triples (`src/encode.rs`, the inverse of `decode.rs`) by feeding
> the wrapped crate's **public** section builders (`DictSectPFC::compress`,
> `TriplesBitmap::from_triples`) and replaying `Hdt::write`'s section order with the
> public section writers. So we no longer NEED an upstream in-memory builder; the
> only residual cost is that those builders are reachable solely via the `sophia`
> feature (see below).

---

## Issue / PR 1 — sq-ashy: feature-gate the section builders off `sophia`

**Title:** Make the in-memory section builders usable without the `sophia` feature

**Body:**

The pieces needed to BUILD an archive in memory — `DictSectPFC::compress`,
`FourSectDict { … }`, `TriplesBitmap::from_triples`, and the `*::write` methods —
are already `pub`. A consumer that holds triples in memory can build + write a
spec-conformant `.hdt` directly from them (we do exactly this in
`sparq-hdt/src/encode.rs`), with NO N-Triples text round-trip.

The remaining friction is the FEATURE GATE: the only documented build entry point,
`Hdt::read_nt`, lives behind `sophia`, and enabling it pulls the whole sophia
adapter dependency tree (oxttl + the sophia term model) even for a consumer that
never touches a sophia term — it only wants `compress` / `from_triples` / `write`.

**Request.** Expose an in-memory builder (and/or move the existing section builders)
behind a lighter, sophia-free feature, e.g.

```rust
impl Hdt {
    /// Build an archive from an iterator of (subject, predicate, object) term
    /// strings (IRIs bare, blank nodes `_:label`, literals in N-Triples lexical
    /// shape — i.e. the dictionary's own string encoding).
    pub fn from_triples<I, S>(triples: I) -> Result<Self>
    where
        I: IntoIterator<Item = (S, S, S)>,
        S: AsRef<str>;
}
```

This is essentially what `read_nt` does AFTER it has parsed the file:
`FourSectDict::read_nt` builds the dictionary and the encoded triples, then
`TriplesBitmap::from_triples` builds the bitmaps — both already exist. A builder /
feature that does not depend on `sophia`/oxttl would decouple "I want to WRITE HDT"
from "I want the sophia term adapter".

We're happy to open the PR. Reference implementations for cross-checking the
on-disk layout: ENCODE — `sparq-hdt/src/encode.rs`; DECODE — `sparq-hdt/src/decode.rs`.

---

## Issue / PR 2 — sq-fkj: a decode-only / streaming-triples entry point (skip query indexes on bulk ingest)

**Title:** Add a decode-only `Hdt::triples_streaming(reader)` that skips
`TriplesBitmap` query-index construction

**Body:**

`Hdt::read` eagerly calls `TriplesBitmap::new`, which builds — purely to serve
triple-pattern / object / predicate QUERIES — a `WaveletMatrix`, a per-object
`Vec<Vec<u32>>`, a `sort_by_cached_key`, and an OP-index (`CompactVector` +
`Rank9Sel` bitmap). A consumer doing a one-shot bulk load (read every triple once,
in SPO order, into its own store) never issues those queries, so all of that is
built and immediately dropped — a large, cache-hostile cost on ingest.

**Request.** A decode-only entry point that reads the dictionary + the `bitmap_y`,
`bitmap_z`, `sequence_y`, `sequence_z` sections and yields `(s, p, o)` ids (or term
strings) in SPO order **without** constructing the `TriplesBitmap` query
structures, e.g.

```rust
impl Hdt {
    /// Decode-only: stream every triple in SPO order without building the
    /// wavelet matrix / OP-index used for pattern queries.
    pub fn triples_streaming<R: BufRead>(reader: R)
        -> Result<impl Iterator<Item = Result<[usize; 3]>>>;
}
```

Reference implementation: `sparq-hdt/src/decode.rs` already does exactly this
(it reads the same on-disk bytes, walks the bitmaps with a plain bit read, and
skips the rank/select build) and is differentially tested against the full
`Hdt::read` path on real and generated archives. If upstream adopts a decode-only
path we can delete our vendored decoder and call it instead.
