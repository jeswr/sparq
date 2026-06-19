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

The snippet below is **not hand-written prose** — it is the `quickstart` region of
[`crates/sparq-engine/examples/quickstart.rs`](https://github.com/jeswr/sparq/blob/main/crates/sparq-engine/examples/quickstart.rs),
embedded verbatim via mdBook's `{{#rustdoc_include}}`. That file is compiled and run by
`cargo test -p sparq-engine --examples`, so this example cannot silently drift from the
public API:

<!-- [OPUS-4.8] sq-384j — tested-example embedding. The fence is `rust,ignore` so
`mdbook test` does NOT recompile the snippet standalone: it references the workspace
crates `sparq-core`/`sparq-engine`, which a bare rustdoc invocation cannot resolve
(that is the "mdbook-keeper only if inline blocks need workspace deps" case from the
bead — avoided here by letting `cargo test` be the compile-and-run gate on the real
file). The anchor keeps the guide a fragment of a passing test. -->
```rust,ignore
{{#rustdoc_include ../../../crates/sparq-engine/examples/quickstart.rs:quickstart}}
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
