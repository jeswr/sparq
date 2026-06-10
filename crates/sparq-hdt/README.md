# sparq-hdt

Opt-in [HDT](https://www.rdfhdt.org/) (Header Dictionary Triples) reader for the
sparq RDF engine: load `.hdt` archives straight into a `sparq_core::Graph`.

```rust
let graph = sparq_hdt::load("dataset.hdt")?;
// query it like any other sparq graph
```

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

## Validation

`tests/roundtrip.rs`:

- `snikmeta.hdt` — a real-world archive vendored from the `hdt` crate's test suite
  (i.e. not produced by this code path) — must load to exactly the same 328-triple
  set as its N-Triples rendering loaded through sparq's own parser.
- A term-zoo N-Triples document round-tripped through a generated HDT archive
  (unicode, lang tags, datatypes, blank nodes, shared subject/object terms,
  inline-integer literals) must match sparq's direct N-Triples load.

## Load throughput (sketch)

~1M synthetic triples (100k subjects, 50 predicates, mixed IRI/literal objects),
Apple Silicon, best of 3, via `cargo run --release -p sparq-hdt --example bench_load`:

| input            | size on disk | load           | throughput      |
|------------------|-------------:|---------------:|----------------:|
| `bench.hdt`      |      14.8 MB |         2.40 s | 416 k triples/s |
| `bench.nt.gz`    |      11.2 MB |         3.47 s | 289 k triples/s |
| (`bench.nt` raw) |      98.6 MB |              — |               — |

≈1.4× faster than gunzip-and-parse. On this synthetic data (mostly unique literal
objects) the HDT file is slightly larger than the `.nt.gz`; on real corpora with
heavier term reuse HDT archives are typically several times smaller than gzipped
N-Triples, which is the format's main draw alongside no-text-parse loading.

## Not (yet) supported

See `TODO.md`: writing HDT archives, GZipped-HDT containers, and exposing the HDT
header metadata.
