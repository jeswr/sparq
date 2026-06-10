# sparq-hdt — gaps / follow-ups

## CLI hook (one line, for a sparq-cli maintainer)

sparq-cli was deliberately not modified (T24a scope: new opt-in crate only). To
expose HDT loading, add `sparq-hdt = { path = "../sparq-hdt", version = "0.1.0" }`
to `crates/sparq-cli/Cargo.toml` and route the format in the loader dispatch:

```rust
"hdt" => sparq_hdt::load(path)?,   // alongside the existing "ntriples"/"turtle"/… arms
```

(keyed off `--format hdt` or a `.hdt` file extension).

## Write support

Not implemented. The wrapped `hdt` 0.4 crate can write an `Hdt` it already holds
(`Hdt::write`) and can build one from an N-Triples FILE (`Hdt::read_nt`, used by
our tests/bench to generate fixtures), but there is no in-memory
triples -> FourSectDict builder API, so `Graph -> .hdt` would mean serializing the
graph to a temp N-Triples file first — not "cheap". Revisit if upstream grows a
builder API (its `nt` module is evolving in 0.6+); reading is the community win.

## hdt crate version pin

Pinned to `hdt = 0.4` (not 0.6): hdt 0.6 depends on `qwt`, whose default
`prefetch` feature requires NIGHTLY rustc on aarch64 (`#![feature(stdarch_aarch64_prefetch)]`),
breaking stable Apple-Silicon builds. Bump when upstream drops/feature-gates that
(tracked upstream: qwt prefetch is a default feature hdt does not disable).
hdt 0.4's MSRV is 1.87, so this crate declares `rust-version = "1.87"` (workspace
floor is 1.85).

## Loader gaps (upstream API shape)

- `Hdt::read` eagerly builds its own object-position index (and in 0.6 a wavelet
  matrix) for pattern queries we never run — pure ingest could skip it. An
  upstream "decode only" entry point would cut HDT->Graph load time.
- GZipped HDT containers (`.hdt.gz` as emitted by some publishers) are not
  auto-detected; callers can wrap the reader in a `GzDecoder` themselves via
  `load_reader` only after decompressing to a buffered stream.
- The HDT header (dataset metadata triples) is decoded by the wrapped crate but
  not exposed through `sparq_hdt`; `graph_from_hdt` is the seam if a caller wants
  it today.

## sparq-core gaps encountered

None — `Dict::intern_iri/intern_lit/intern_blank` + `Graph::from_parts` were
sufficient; no existing crate needed modification.
