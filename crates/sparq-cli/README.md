# sparq-cli

<p>
  <a href="https://crates.io/crates/sparq-cli"><img src="https://img.shields.io/crates/v/sparq-cli.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-cli"><img src="https://docs.rs/sparq-cli/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

The **command-line interface** to the [sparq](../../README.md) RDF triplestore.

Load and query RDF files, build on-disk indexes once and query them memory-mapped
(out-of-core, near-zero heap), benchmark queries, and run RDFS / OWL-RL / N3 reasoning —
all from one binary. Input is any RDF text format with transparent `.gz` / `.bz2` / `.zst`
decompression; SELECT/ASK results render as a table, TSV, CSV, XML or JSON.

## 🚀 Quickstart

```sh
# Query a file (Turtle / N-Triples / N-Quads / TriG, optionally .gz / .bz2 / .zst)
cargo run --release -p sparq-cli -- query data.ttl turtle \
  'SELECT ?s ?o WHERE { ?s <http://schema.org/name> ?o } LIMIT 10'

# Build on-disk indexes once, then query them memory-mapped (out-of-core, ~0 heap)
cargo run --release -p sparq-cli -- build data.nt ntriples ./idx
cargo run --release -p sparq-cli -- query-mmap ./idx 'SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }'

# Materialize RDFS/OWL-RL/N3 inference before querying
cargo run --release -p sparq-cli -- query data.ttl turtle 'SELECT * WHERE { ?s ?p ?o }' --reason rdfs
```

## ✨ Features

- **`query`** — load a file and run one query; SELECT/ASK output as table / tsv / csv / xml /
  json (`--format`), CONSTRUCT/DESCRIBE as N-Triples.
- **`build` / `query-mmap`** — persist the six permutation indexes to disk, then query them
  memory-mapped without loading the dataset into RAM.
- **`bench`** — run a directory of `*.rq` queries N times each, one TSV timing line per query.
- **`--reason <rdfs|owl-rl|n3>`** — opt-in forward-chaining materialization before query.
- **Transparent decompression** — `.gz` / `.bz2` / `.zst` inputs detected by content.
  The gzip path defaults to the pure-Rust `miniz_oxide` backend; the opt-in, native-only
  `zlib-ng` cargo feature (`cargo build -p sparq-cli --features zlib-ng`, or
  `--features hdt,zlib-ng` to extend it to `.hdt.gz`) swaps in the faster zlib-ng C
  backend for gzip inflate at zero code change. Off by default; native-only, so it never
  reaches the wasm build.

## 📚 Learn more

- **How-to** — [`skills/cli/SKILL.md`](../../skills/cli/SKILL.md) (full subcommand reference)
  and [`skills/inference/SKILL.md`](../../skills/inference/SKILL.md) (reasoning).
- **API reference** — run `cargo run -p sparq-cli -- --help`; rustdoc at
  [docs.rs/sparq-cli](https://docs.rs/sparq-cli).
- **Design** — [`research/ARCHITECTURE.md`](../../research/ARCHITECTURE.md).
- **Performance** — see the [benchmarks dashboard](https://jeswr.github.io/sparq/dev/bench);
  numbers are not baked into docs.
- **Contribute** — [`AGENTS.md`](../../AGENTS.md) and [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## License

[MIT](../../LICENSE).
