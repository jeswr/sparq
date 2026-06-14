# sparq-core

<p>
  <a href="https://crates.io/crates/sparq-core"><img src="https://img.shields.io/crates/v/sparq-core.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-core"><img src="https://docs.rs/sparq-core/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

The **dictionary-encoded RDF triplestore** at the heart of [sparq](../../README.md): a
`Graph` is a term dictionary plus six sorted permutation indexes.

It is the storage substrate every other sparq crate builds on. Terms are interned to
integer ids once; the six permutations (SPO/SOP/PSO/POS/OSP/OPS) make every triple-pattern
shape an index range scan. Loaders are parallel and streaming, read the RDF text formats from
a `&str` or any `Read` (wrap a `.gz` / `.bz2` / `.zst` decompressor to stream compressed input
straight into the parse), and an out-of-core, memory-mapped store with optional
block-compressed permutations queries datasets larger than RAM.

## 🚀 Quickstart

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use sparq_core::Graph;

let turtle = r#"<http://example.org/alice> a <http://schema.org/Person> ."#;
let g = Graph::load_str(turtle, "turtle")?;

// Range-scan a triple pattern over the permutation indexes.
let count = g.len();
assert_eq!(count, 1);
# Ok(()) }
```

## ✨ Features

- **Dictionary encoding** — every term interned to a `u32` id once; storage and joins
  operate on fixed-width integers, not strings.
- **Six permutation indexes** — every BGP triple pattern is an index range scan; no
  full-graph filtering.
- **Parallel + streaming loaders** — Turtle / N-Triples / N-Quads / TriG from a `&str` or any
  `Read`; a caller-supplied `.gz` / `.bz2` / `.zst` decompressor streams straight into the parse.
- **Incremental updates** — delta-overlay inserts/deletes with an optional write-ahead log.
- **Out-of-core store** — memory-mapped permutations (optional block compression) for
  datasets larger than RAM, with near-zero resident heap.
- **Named graphs & RDF 1.2** — full quad storage and structurally stored quoted triple terms.

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
