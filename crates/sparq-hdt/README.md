# sparq-hdt

Opt-in [HDT](https://www.rdfhdt.org/) (Header Dictionary Triples) reader (and,
behind the `write` feature, writer) for the sparq RDF engine: load `.hdt` archives
straight into a `sparq_core::Graph`, and save a `Graph` back out.

```rust
let graph = sparq_hdt::load("dataset.hdt")?;   // .hdt.gz sniffed + decompressed too
let meta  = sparq_hdt::header("dataset.hdt")?; // the HDT header (VoID stats, provenance) as a Graph
// query them like any other sparq graph

// writing (opt-in `write` feature): Graph -> .hdt (honours .gz/.zst/.bz2 by extension)
# #[cfg(feature = "write")]
sparq_hdt::save(&graph, "out.hdt")?;
```

In sparq-cli (behind the opt-in `hdt` cargo feature —
`cargo build -p sparq-cli --features hdt`), the `hdt` format argument or a
`.hdt`/`.hdt.gz` file extension routes loading through this crate.

This is a separate crate so the core engine — and in particular the wasm build —
carries zero HDT code or dependencies. Native-only by design.

## What it does

- Wraps the maintained [`hdt`](https://crates.io/crates/hdt) crate
  (KonradHoeffner/hdt, MIT) for the binary decode rather than reimplementing the
  format (Plain/Log64 sequences, Plain-Front-Coded dictionary sections,
  BitmapTriples).
- **Format coverage**: the standard HDT v1.0 layout produced by hdt-cpp / hdt-java
  / the `hdt` crate — `FourSectionDictionary` (PFC) + `BitmapTriples`, SPO order.
  Exotic layouts from the W3C member submission that no mainstream tool emits
  (triples lists, alternate dictionary implementations) are rejected with an error
  by the wrapped reader.
- **Translation is id-level and single-pass**: each distinct HDT dictionary id is
  decompressed once, interned into the sparq dictionary, and memoized in a flat
  per-section table (shared-section terms are translated once even when used as
  both subject and object) — the term set is never materialized twice.
- HDT term shapes covered: IRIs, blank nodes, plain / language-tagged / datatyped
  literals (lang tags normalized to lowercase, matching sparq's other loaders).
- **GZipped containers** (`.hdt.gz`): detected by magic bytes — not file names —
  in every entry point and decompressed on the fly (streaming flate2). The default
  flate2 backend is pure-Rust `miniz_oxide`; the opt-in, native-only `zlib-ng` cargo
  feature (`cargo build -p sparq-hdt --features zlib-ng`) swaps in the faster zlib-ng
  C backend for gzip inflate at zero code change. Off by default and never reaches the
  wasm build (this crate is native-only and not in the wasm graph).
- **Header access**: `header()` / `header_reader()` decode just the dataset
  metadata triples (the "H" in HDT) into a queryable `Graph` without touching
  the dictionary/triples sections.

## Validation

`tests/roundtrip.rs`:

- `snikmeta.hdt` — a real-world archive vendored from the `hdt` crate's test suite
  (i.e. not produced by this code path) — must load to exactly the same 328-triple
  set as its N-Triples rendering loaded through sparq's own parser.
- A term-zoo N-Triples document round-tripped through a generated HDT archive
  (unicode, lang tags, datatypes, blank nodes, shared subject/object terms,
  inline-integer literals) must match sparq's direct N-Triples load.

`tests/write_roundtrip.rs` (the `write` feature): a `Graph` saved with `save` and
reloaded with `load` must equal the original term set — over the term zoo, a
multi-block graph, the empty graph, all three compression containers, and the
upstream-oracle load path (so the bytes `save` writes are spec-conformant, not just
something our own decoder accepts).

## Load throughput

The `bench_load` example loads ~1M synthetic triples (100k subjects, 50 predicates,
mixed IRI/literal objects) from a `.hdt` archive vs the equivalent `.nt.gz`, reporting
size on disk, load time, and throughput for each. Run it for the numbers:

```sh
cargo run --release -p sparq-hdt --example bench_load
```

HDT loads faster than gunzip-and-parse. On synthetic data with mostly unique literal
objects the HDT file is about the size of the `.nt.gz`; on real corpora with heavier
term reuse HDT archives are typically several times smaller than gzipped N-Triples,
which is the format's main draw alongside no-text-parse loading.

## Writing (opt-in `write` feature)

`save(&graph, path)` serialises a `Graph` to a standard-layout `.hdt` (or
`.hdt.gz` / `.hdt.zst` / `.hdt.bz2`, chosen by the output extension). HDT carries a
single default graph, so named graphs are ignored.

**Cost.** The current path round-trips through a **temporary N-Triples file**: it
renders the graph to N-Triples text, hands it to the wrapped crate's builder
(`Hdt::read_nt`, which re-parses and re-interns it), then `Hdt::write`s the result.
That is correct and interoperable, but it re-serialises and re-parses the whole
graph — work sparq already did on ingest. A direct in-memory builder that skips the
text round-trip (the inverse of `src/decode.rs`) is the faster path; it is queued
as an upstream contribution — see [`UPSTREAM.md`](./UPSTREAM.md). Enable with
`--features write`.

## Not (yet) supported

See the open beads for this crate (`bd list -l area:sparq-hdt`): the faster
direct-builder write path (`save` works today via the temp-N-Triples round-trip
above; the in-memory builder is queued upstream, `UPSTREAM.md`) and a decode-only
ingest fast path (we already roll our own in `decode.rs`; upstream builds
pattern-query indexes ingest never uses — also queued in `UPSTREAM.md`).
