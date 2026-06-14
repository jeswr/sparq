<!-- [OPUS-4.8] sq-2te — upstream contributions queued against KonradHoeffner/hdt. -->
# Upstream contributions to `KonradHoeffner/hdt`

This crate WRAPS [`hdt`](https://github.com/KonradHoeffner/hdt) (MIT). Two API
gaps in the wrapped crate (as of `hdt` 0.4) make our `save` / `load` paths pay a
cost they should not have to. Both are tractable upstream additions; the text
below is ready to file. If accepted, we can drop the corresponding local
work-arounds (the temp-N-Triples round-trip in `write.rs`; the vendored decoder in
`decode.rs`).

---

## Issue / PR 1 — sq-2te: an in-memory / iterator builder (no N-Triples text round-trip)

**Title:** Expose an in-memory builder: `Hdt::from_triples(iter)` (and feature-gate
the builder off `sophia`)

**Body:**

Today the only way to BUILD an `Hdt` is `Hdt::read_nt(path: &Path)`, which:

1. requires the triples to already be **serialised as N-Triples text in a file**
   (the `read_nt(reader)` variant is commented out in `src/hdt.rs`), and
2. is gated behind the `sophia` feature, which pulls the whole sophia adapter
   dependency tree.

For a consumer that already holds triples in memory as `(subject, predicate,
object)` strings (or as integer ids into its own dictionary), this forces a full
serialise-to-text → re-parse → re-intern round trip through a temp file. In our
case (`sparq-hdt`, an RDF engine) we hold the graph in a compact id-based store;
saving it to `.hdt` means writing every term back out as N-Triples text to a temp
file and having `read_nt` parse and re-intern all of it — work we already did once
on ingest, plus a full-graph text materialisation on disk.

**Request.** A builder entry point that takes triples directly, e.g.

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
`TriplesBitmap::from_triples` builds the bitmaps — both already exist; the only
missing piece is an entry point that feeds them from an in-memory iterator instead
of an oxttl parse of a file. Such a builder also need not depend on `sophia`/oxttl
at all, so it could live behind a lighter `build` feature (or no feature),
decoupling "I want to WRITE HDT" from "I want the sophia term adapter".

We're happy to open the PR. Reference reverse-direction implementation
(PFC + BitmapTriples DECODE) for cross-checking the on-disk layout:
`sparq-hdt/src/decode.rs`.

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
