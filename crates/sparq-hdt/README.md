# sparq-hdt

Opt-in [HDT](https://www.rdfhdt.org/) (Header Dictionary Triples) reader for the
sparq RDF engine: load `.hdt` archives straight into a `sparq_core::Graph`.

```rust
let graph = sparq_hdt::load("dataset.hdt")?;   // .hdt.gz sniffed + decompressed too
let meta  = sparq_hdt::header("dataset.hdt")?; // the HDT header (VoID stats, provenance) as a Graph
// query them like any other sparq graph
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
  in every entry point and decompressed on the fly (streaming, pure-Rust flate2
  backend).
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

## Not (yet) supported

See the open beads for this crate (`bd list -l area:sparq-hdt`): writing HDT archives (blocked upstream — the wrapped crate has no
in-memory builder API, re-verified against hdt 0.6) and a decode-only fast path
(upstream builds pattern-query indexes ingest never uses).
