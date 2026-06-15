# sparq-core

<p>
  <a href="https://crates.io/crates/sparq-core"><img src="https://img.shields.io/crates/v/sparq-core.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-core"><img src="https://docs.rs/sparq-core/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

The **[RDF](https://www.w3.org/TR/rdf12-concepts/) triplestore** at the heart of
[sparq](../../README.md) — the storage substrate every other sparq crate builds on.

Load [RDF 1.2](https://www.w3.org/TR/rdf12-concepts/) (named graphs and quoted triple terms
included) from the text formats, in memory or out-of-core for datasets larger than RAM, and
scan triple patterns. How the store is laid out and why is in the design docs linked below.

## 🚀 Quickstart

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use sparq_core::Graph;

let turtle = r#"<http://example.org/alice> a <http://schema.org/Person> ."#;
let g = Graph::load_str(turtle, "turtle")?;

let count = g.len();
assert_eq!(count, 1);
# Ok(()) }
```

## ✨ Features

- **RDF parsing & ingest** — load Turtle, N-Triples, N-Quads, and TriG from a `&str` or any
  `Read`, with transparent `.gz` / `.bz2` / `.zst` decompression
  ([guide](../../skills/data-formats/SKILL.md)).
- **Triple-pattern scans** — look up any triple pattern over the loaded graph.
- **Incremental updates** — insert and delete triples in place, with an optional
  write-ahead log.
- **Out-of-core store** — query datasets larger than RAM from a memory-mapped on-disk store,
  with optional block compression and near-zero resident heap.
- **Named graphs & RDF 1.2** — full quad storage and
  [quoted triple terms](https://www.w3.org/TR/rdf12-concepts/).

## 📚 Learn more

- **How-to** — [`skills/data-formats/SKILL.md`](../../skills/data-formats/SKILL.md) (ingest)
  and [`skills/sparql-query/SKILL.md`](../../skills/sparql-query/SKILL.md) (Rust API).
- **API reference** — [docs.rs/sparq-core](https://docs.rs/sparq-core).
- **Design** — [`research/ARCHITECTURE.md`](../../research/ARCHITECTURE.md); the indexing,
  compression and parsing verdicts live across the [`research/`](../../research) tree.
- **Performance** — numbers are not baked into docs; see the
  [benchmarks dashboard](https://jeswr.github.io/sparq/dev/bench).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md) and [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## License

[MIT](../../LICENSE).
