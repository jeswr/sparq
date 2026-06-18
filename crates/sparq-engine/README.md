# sparq-engine

<p>
  <a href="https://crates.io/crates/sparq-engine"><img src="https://img.shields.io/crates/v/sparq-engine.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-engine"><img src="https://docs.rs/sparq-engine/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

The [SPARQL 1.1](https://www.w3.org/TR/sparql11-query/) / [1.2](https://www.w3.org/TR/sparql12-query/)
query engine over [`sparq-core`](../sparq-core) `Graph`s.

Run conformant SPARQL over an in-memory or out-of-core graph, with `EXPLAIN` / `EXPLAIN ANALYZE`
for plan introspection and a hook for registering your own functions. How it plans and executes
queries is described in the design docs linked below.

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

- **SPARQL query** — run [SPARQL 1.1](https://www.w3.org/TR/sparql11-query/) and
  [1.2](https://www.w3.org/TR/sparql12-query/) over your data; conformance is tracked by the
  CI ratchets (see the root [`README.md`](../../README.md)).
- **Named graphs** — query across an active dataset with `GRAPH` and `FROM` / `FROM NAMED`.
- **RDF 1.2 / RDF-star** — match [quoted triple terms](https://www.w3.org/TR/rdf12-concepts/),
  including variables inside them.
- **Query plan introspection** — `EXPLAIN` and `EXPLAIN ANALYZE`.
- **Custom functions** — register Rust closures under function IRIs (the
  [SPARQL extension mechanism](https://www.w3.org/TR/sparql11-query/#extensionFunctions));
  see [`docs/extension-functions.md`](../../docs/extension-functions.md).
- **Custom aggregates + window functions** *(opt-in `window-functions` feature, OFF by default)* —
  register a named user aggregate (`CustomAggregateRegistry`) callable from a real SPARQL `GROUP BY`,
  and a window-function surface (`ROW_NUMBER` / `RANK` / `DENSE_RANK`, the offset/positional
  `LAG`/`LEAD`/`NTILE` (sq-hqhc), + the windowed aggregates `COUNT`/`SUM`/`AVG`/`MIN`/`MAX`, with
  `PARTITION BY` + `ORDER BY` and an optional `ROWS`/`RANGE` frame)
  available two ways: a programmatic pass (`window::apply_window` over a `QueryResult`) and an inline
  `OVER(…)` query syntax (`query_over` — e.g. `(SUM(?x) OVER (ORDER BY ?y ROWS BETWEEN UNBOUNDED
  PRECEDING AND CURRENT ROW) AS ?run)` in the SELECT, and a reusable `WINDOW w AS (…)` clause with
  `OVER w`). **Window functions are a NON-STANDARD extension**
  — SPARQL has no W3C-REC `OVER` syntax. The inline form is a *source rewrite* in front of the engine
  recognised ONLY on `query_over` (it does NOT change the vendored parser), so the standard
  `query`/`ask`/… surface stays exactly SPARQL 1.1 and conformance is unaffected. Inline covers the three
  ranking functions, the five windowed aggregates, `ROWS`/`RANGE` frames (sq-imj8), the offset/positional
  `LAG`/`LEAD`/`NTILE` and a named `WINDOW` clause (sq-hqhc); `DISTINCT`/computed-expression function
  arguments, numeric `RANGE` offsets, and a computed-expression `PARTITION BY` are inline-deferred
  (use the programmatic API). When the feature is off, zero window/aggregate-registry code compiles and
  the default build is byte-identical (no new deps).
- **Materialised-view / query-result cache** *(opt-in `result-cache` feature, OFF by default)* —
  a bounded, version-aware LRU (`cache::ResultCache`) that stores a SELECT/ASK `QueryResult` keyed
  by `(parsed query algebra, caller graph-version)`, so an endpoint that re-serves the same read
  query against a slowly-changing graph can replay the materialised result instead of re-executing.
  **Caching is sound only under a contract**: the caller bumps a `u64` *version* on every mutation
  (the engine can't observe writes through a borrowed `&Graph`), and the cache **refuses to store
  non-deterministic queries** — `NOW`/`RAND`/`UUID`/`STRUUID`/`BNODE`, a remote `SERVICE`, or any
  custom function / aggregate (detected conservatively by `is_cacheable`); those always evaluate
  fresh. Keying on the parsed algebra makes the cache insensitive to whitespace / comments / prefix
  spelling. When the feature is off, zero cache code compiles, the default build is byte-identical,
  and no new dependencies are added (std `HashMap`/`Mutex`/`Arc` only).
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
