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
- **RDF 1.2 triple terms** — match [triple terms](https://www.w3.org/TR/rdf12-concepts/),
  including variables inside them.
- **Query plan introspection** — `EXPLAIN` and `EXPLAIN ANALYZE`.
- **Custom functions** — register Rust closures under function IRIs (the
  [SPARQL extension mechanism](https://www.w3.org/TR/sparql11-query/#extensionFunctions));
  see [`docs/extension-functions.md`](../../docs/extension-functions.md).
- **Custom aggregates + window functions** *(opt-in `window-functions` feature, OFF by default)* —
  register a named user aggregate (`CustomAggregateRegistry`) callable from a real `GROUP BY`, plus a
  window surface (`ROW_NUMBER`/`RANK`/`DENSE_RANK`, `LAG`/`LEAD`/`NTILE`, windowed
  `COUNT`/`SUM`/`AVG`/`MIN`/`MAX`, with `PARTITION BY` + `ORDER BY` and an optional `ROWS`/`RANGE`
  frame) two ways: programmatic (`window::apply_window`) and inline `OVER(…)` syntax (`query_over`, +
  a reusable `WINDOW w AS (…)` clause). **NON-STANDARD extension** — SPARQL has no W3C-REC `OVER`. The
  inline form is a *source rewrite* recognised ONLY on `query_over` (the vendored parser is untouched),
  so the standard `query`/`ask`/… surface stays exactly SPARQL 1.1; `DISTINCT`/computed function args,
  numeric `RANGE` offsets, and computed `PARTITION BY` are inline-deferred (use the programmatic API).
  Off, zero code compiles, the default build is byte-identical (no new deps).
- **Parameterized prepared queries** *(opt-in `params` feature, OFF by default)* — the canonical
  mitigation for SPARQL injection (#901). `PreparedQuery::bind(name, oxrdf::Term)` and
  `PreparedUpdate::bind` substitute a typed value into a free placeholder variable via a pure
  **algebra rewrite** — *never* string concatenation — so a hostile bound IRI/literal (e.g. one
  containing `> } INSERT … {` or a `"` break-out) is carried as opaque DATA and cannot alter the
  query structure. Covers SELECT/ASK/CONSTRUCT/DESCRIBE + UPDATE; fail-closed (rejects an unknown
  placeholder, a `BIND`/aggregate/`VALUES` output, or a blank node in a predicate/graph slot). Off,
  zero code compiles, the default build is byte-identical, no new deps. See
  [`skills/sparql-query/SKILL.md`](../../skills/sparql-query/SKILL.md).
- **Materialised-view / query-result cache** *(opt-in `result-cache` feature, OFF by default)* —
  a bounded, version-aware LRU (`cache::ResultCache`) that stores a SELECT/ASK `QueryResult` keyed
  by `(parsed query algebra, caller graph-version)`, replaying it instead of re-executing the same
  read query against a slowly-changing graph. **Sound only under a contract**: the caller bumps a
  `u64` *version* on every mutation, and the cache **refuses non-deterministic queries**
  (`NOW`/`RAND`/`UUID`/`STRUUID`/`BNODE`, remote `SERVICE`, any custom fn/aggregate — via
  `is_cacheable`). When off, zero cache code compiles, the default build is byte-identical, no new
  deps (std `HashMap`/`Mutex`/`Arc`).
- **MVCC / ACID transaction isolation** *(opt-in `txn` feature, OFF by default)* —
  a `txn::TransactionManager` over one logical `Graph` giving **snapshot-isolation** reads
  (`begin_read` → a cheap point-in-time `GraphSnapshot`, immune to later commits) and serialized
  **write** transactions (`begin_write` → a private COW fork) with **first-committer-wins** write-write
  conflict detection (OCC). A `WriteTxn` has read-your-own-writes, applies `UPDATE`s to its fork, and
  on `commit` either publishes a new generation (advancing a `u64` version) or returns
  `CommitError::Conflict` on an overlapping concurrent write set (atomic rollback); a stale but
  *non*-conflicting writer has its delta replayed (no lost update). For a *single writer* SI is
  serializability (see `research/concurrent-serving-litreview-A-mvcc-benchmarks.md` §A.1); durability
  is inherited from a directory-backed `Graph`'s WAL. Built entirely on the existing COW delta-overlay
  substrate; off, zero code compiles, the default build is byte-identical, no new deps.
- **RDF writer matrix** *(`serialize-rdf` feature — OFF for a library embedder, but the `sparq-cli`/`sparq-server`
  BINARIES enable it via their default-on `jsonld` feature, [OPUS-4.8] sq-oy1f.4)* — write a `Graph` (or
  `&[oxrdf::Triple]`) back out as Turtle / TriG / N-Quads / JSON-LD 1.1
  (`serialize::{graph_to_turtle, graph_to_trig, graph_to_nquads, graph_to_jsonld, …}`; the `*_with`
  variants + `prefixes_from_pairs([(prefix, iri), …])` accept a caller's own prefix policy). Plus
  deterministic **pretty** (indented) variants for Turtle / TriG / JSON-LD (`graph_to_turtle_pretty`
  / `graph_to_trig_pretty` / `graph_to_jsonld_pretty`, or `write_turtle_pretty` with
  `PrettyOptions { indent, abbreviate }`): subject grouping, p-o/object lists, `a` for `rdf:type`,
  used-only `@prefix`, *emission-order-independent* (sorted) — round-trip-correct. For true **W3C
  JSON-LD 1.1 Compaction** against a caller `@context`, `graph_to_jsonld_compact(&g, &ctx)` /
  `write_jsonld_compact` (+ `_pretty` variants) apply the full algorithm (term definitions, `@vocab`,
  `@container` coercion, `@reverse`, value/node/IRI compaction), plus **JSON-LD 1.1 Framing**
  (`graph_to_jsonld_framed`) — hand-rolled, dependency-free, **pyld-faithful** (differentially
  verified; see the `serialize::compact` rustdoc). The N-Triples writer (`triples_to_ntriples`) is
  always on; off, zero serializer code compiles, **no new dependencies**. See
  [`skills/data-formats/SKILL.md`](../../skills/data-formats/SKILL.md) recipe 6.
- **Oxigraph-shaped per-solution accessor** *(opt-in `query-solution` feature, OFF by default)* —
  `QueryResult::solutions()` yields borrowed, zero-copy `QuerySolution` views (one per row) matching
  Oxigraph's `QuerySolution` API — `get` by name / `VariableRef` / position, `iter` over the bound
  `(Variable, Term)` pairs, panicking `Index` — over the engine's columnar `{vars, rows}` table with
  no second materialisation (eases Rust Oxigraph interop / migration). Off, zero code compiles, the
  default build is byte-identical, no new deps. See
  [`skills/sparql-query/SKILL.md`](../../skills/sparql-query/SKILL.md).
- **Structured EXPLAIN** *(opt-in `explain-json` feature, OFF by default)* — `explain_plan` /
  `explain_plan_analyze` → a typed `PlanNode` tree (BGP `estimated`, `actual`/`nanos`, per-operator **q-error**) + `to_json()` + a bounded `SlowQueryRing`; off, build byte-identical.
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
