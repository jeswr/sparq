# sparq-engine

<p>
  <a href="https://crates.io/crates/sparq-engine"><img src="https://img.shields.io/crates/v/sparq-engine.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-engine"><img src="https://docs.rs/sparq-engine/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

The **SPARQL 1.1/1.2 query engine** over [`sparq-core`](../sparq-core) `Graph`s.

It parses SPARQL to algebra (via `spargebra`), compiles the algebra to a physical plan with
cardinality-based join ordering, and executes it in parallel over the permutation indexes —
sort-merge, hash and worst-case-optimal joins. It runs SELECT / ASK / CONSTRUCT / DESCRIBE,
the full FILTER built-in set, OPTIONAL / UNION / MINUS / VALUES / BIND, aggregation, all
property-path operators, and named-graph dataset clauses, with EXPLAIN / EXPLAIN ANALYZE for
plan introspection.

## 🚀 Quickstart

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use sparq_core::Graph;

let g = Graph::load_str(
    r#"<http://example.org/alice> a <http://schema.org/Person> ."#, "turtle")?;

let rows = sparq_engine::query(&g, "SELECT ?s WHERE { ?s a <http://schema.org/Person> }")?;
let json = sparq_engine::query_json(&g, "SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }")?;
# let _ = (rows, json);
# Ok(()) }
```

## ✨ Features

- **SPARQL 1.1/1.2 query** — SELECT / ASK / CONSTRUCT / DESCRIBE, OPTIONAL / UNION / MINUS /
  VALUES / BIND, sub-SELECT, aggregation + GROUP BY / HAVING, ORDER BY, DISTINCT, LIMIT/OFFSET.
- **All 8 property-path operators** and full FILTER built-ins with XSD-aware comparisons.
- **Cardinality-based planning** — greedy join ordering with sort-merge / hash / bind /
  worst-case-optimal join selection; `EXPLAIN` and `EXPLAIN ANALYZE`.
- **Named graphs** — `GRAPH`, FROM / FROM NAMED active-dataset scoping; zero-copy dataset views.
- **RDF 1.2 / RDF-star** — quoted triple-term patterns (including variables inside them).
- **Custom functions** — register Rust closures under function IRIs (SPARQL 17.6 extension
  mechanism); see [`docs/extension-functions.md`](../../docs/extension-functions.md).
- **`forbid(unsafe_code)`** — the crate contains zero `unsafe`.

## 📚 Learn more

- **How-to** — [`skills/sparql-query/SKILL.md`](../../skills/sparql-query/SKILL.md).
- **API reference** — [docs.rs/sparq-engine](https://docs.rs/sparq-engine).
- **Design** — [`research/ARCHITECTURE.md`](../../research/ARCHITECTURE.md) and the planning /
  parallelism verdicts in [`research/`](../../research).
- **Performance** — numbers live on the
  [benchmarks dashboard](https://jeswr.github.io/sparq/dev/bench), not in docs.
- **Contribute** — [`AGENTS.md`](../../AGENTS.md) and [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## License

[MIT](../../LICENSE).
