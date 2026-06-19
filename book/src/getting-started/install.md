<!-- [OPUS-4.8] sq-h0tr — scaffold page. Prose distilled from README.md
"Quickstart" + CONTRIBUTING.md "The gate" (no lorem, no hard-coded perf numbers).
The include-wiring bead (sq-im8u) may later single-source the command blocks from
the canonical files. -->

# Install & build from source

sparq is a Rust workspace. The minimum supported Rust version (MSRV) is tracked in CI; a recent
stable toolchain works.

## Build

```sh
cargo build --release
```

## Query a file

Turtle, N-Triples, N-Quads, or TriG — optionally `.gz` / `.bz2` / `.zst` compressed:

```sh
cargo run --release -p sparq-cli -- query data.ttl turtle \
  'SELECT ?s ?o WHERE { ?s <http://schema.org/name> ?o } LIMIT 10'
```

## Build a persistent on-disk store once, then query it without loading into RAM

```sh
cargo run --release -p sparq-cli -- build data.nt ntriples ./idx
cargo run --release -p sparq-cli -- query-mmap ./idx 'SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }'
```

## Run the HTTP server

A W3C SPARQL 1.1 Protocol server on `:3030`:

```sh
cargo run --release -p sparq-server -- --addr 127.0.0.1:3030 --format turtle data.ttl
```

## Use it as a library

```rust,ignore
use sparq_core::Graph;

let turtle = r#"<http://example.org/alice> a <http://schema.org/Person> ."#;
let g = Graph::load_str(turtle, "turtle")?;
let _rows = sparq_engine::query(&g, "SELECT ?s WHERE { ?s a <http://schema.org/Person> }")?;
```

## The contributor gate

If you are working on the engine itself, the gate for landing a change is a green full-workspace
build, test, lint, and the conformance / performance ratchets:

```sh
cargo build --workspace
cargo test  --workspace
cargo clippy --workspace --exclude sparq-py --all-targets -- -D warnings
cargo fmt --check
```

See [`CONTRIBUTING.md`](https://github.com/jeswr/sparq/blob/main/CONTRIBUTING.md) for the full
contributor workflow and the conformance ratchet details.
