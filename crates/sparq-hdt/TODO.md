# sparq-hdt — gaps / follow-ups

Status audit 2026-06-12: every item from the previous revision of this file is
either IMPLEMENTED (with tests) or explicitly DEFERRED below with its reason.

## CLI hook — DONE

sparq-cli wires this crate in behind an opt-in `hdt` cargo feature
(`cargo build -p sparq-cli --features hdt`): the loader dispatch routes the
`hdt` format argument and the `.hdt` / `.hdt.gz` file extensions through
`sparq_hdt::load`; without the feature the CLI exits with a rebuild hint.
Opt-in rather than default because sparq-hdt's MSRV is 1.87 (the wrapped
`hdt` crate) vs sparq-cli's workspace floor of 1.85, and the HDT decode stack
is dead weight on non-HDT paths — rationale documented in
`crates/sparq-cli/Cargo.toml`. End-to-end binary tests:
`cargo test -p sparq-cli --features hdt`.

## GZipped containers — DONE

`.hdt.gz` archives are detected by MAGIC BYTES (0x1f 0x8b — file names are
ignored) in every entry point and decompressed on the fly with a streaming
`MultiGzDecoder` (flate2, pure-Rust backend).

## HDT header — DONE

`header(path)` / `header_reader(reader)` expose the dataset metadata triples
(VoID statistics, format/provenance — the "H" in HDT) as a queryable sparq
`Graph`, decoding only the head of the stream. The wrapped crate parses the
header during `Hdt::read` but keeps the field private, so these re-read the
control info + header sections directly (a few KB); if upstream makes
`Hdt::header` public, `graph_from_hdt` callers could get it without the
re-read.

## Write support — DEFERRED (blocked-upstream; re-verified 2026-06-12)

Not implemented. Checked hdt 0.6.0 (latest on crates.io): there is STILL no
in-memory triples -> FourSectDict builder API — `Hdt::read_nt` takes a file
*path* (its `nt` module builds dictionaries from files via a lasso interner),
so `Graph -> .hdt` would mean serializing the graph to a temp N-Triples file
first — not "cheap". The wrapped crate can write an `Hdt` it already holds
(`Hdt::write`, used by our tests to generate fixtures). Revisit when upstream
grows an in-memory builder; reading is the community win.

## hdt crate version pin — KEPT (re-verified 2026-06-12)

Pinned to `hdt = 0.4` (not 0.6). Re-checked against the currently published
versions: hdt 0.6.0 depends on `qwt = 0.3.4` with DEFAULT features, and qwt
0.3.4 still has `default = ["prefetch"]` whose aarch64 path requires NIGHTLY
rustc (`#![feature(stdarch_aarch64_prefetch)]`) — stable Apple-Silicon builds
break. Bump when qwt drops/feature-gates that or hdt disables the feature.
hdt 0.4's MSRV is 1.87, so this crate declares `rust-version = "1.87"`
(workspace floor is 1.85).

## Loader gaps (upstream API shape) — DEFERRED (blocked-upstream)

- `Hdt::read` eagerly builds its own object-position index (and in 0.6 a wavelet
  matrix) for pattern queries we never run — pure ingest could skip it. An
  upstream "decode only" entry point would cut HDT->Graph load time.

## sparq-core gaps encountered

None — `Dict::intern_iri/intern_lit/intern_blank` + `Graph::from_parts` +
`Graph::load_str` were sufficient; no existing crate needed modification.
