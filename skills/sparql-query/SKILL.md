---
name: sparql-query
description: Run SPARQL 1.1/1.2 queries (SELECT/ASK/CONSTRUCT/DESCRIBE) and UPDATE against the sparq RDF engine in Rust — load RDF into a sparq_core::Graph, then use sparq_engine::{query, ask, query_json, count, construct, describe, update}; covers property paths, RDF-star quoted triples, aggregates/subqueries, custom extension functions (query_with_functions / FunctionRegistry), prepared queries, query budgets/timeouts, named-graph dataset views, and EXPLAIN. Use when an agent or developer needs to embed/execute SPARQL over sparq.
---

# sparq SPARQL query surface

`sparq-engine` is the SPARQL query/update engine over `sparq-core::Graph` (a dictionary-encoded,
permutation-indexed in-memory RDF store). You load RDF into a `Graph`, then call free functions in
`sparq_engine` to run SELECT/ASK/CONSTRUCT/DESCRIBE and SPARQL Update. Results come back either as a
typed `QueryResult` (rows of `Option<oxrdf::Term>`) or directly as a SPARQL-1.1-JSON string.

## Quickstart

`Cargo.toml`:

```toml
[dependencies]
sparq-core = "0.1"
sparq-engine = "0.1"   # default features: parallel, regex, digest
oxrdf = { version = "0.3", features = ["rdf-12"] }   # only if you build Terms yourself
```

```rust
use sparq_core::Graph;

let ttl = r#"@prefix ex: <http://ex/> .
    ex:alice ex:knows ex:bob ; ex:age 30 ; ex:name "Alice" .
    ex:bob   ex:knows ex:carol ; ex:age 25 ; ex:name "Bob" ."#;

// Load RDF: "turtle" | "ntriples" | "nquads" | "trig".
let g = Graph::load_str(ttl, "turtle").unwrap();

// SELECT -> typed rows.
let r = sparq_engine::query(
    &g,
    "PREFIX ex: <http://ex/> SELECT ?s ?age WHERE { ?s ex:age ?age . FILTER(?age > 28) }",
).unwrap();
for row in &r.rows {
    // r.vars: Vec<Variable> in column order; each row cell is Option<oxrdf::Term> (None = unbound).
    println!("{:?}", row[0].as_ref().map(|t| t.to_string()));
}

// SELECT straight to a SPARQL 1.1 JSON results string (skips per-cell Term allocation).
let json = sparq_engine::query_json(&g, "SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }").unwrap();

// ASK -> bool (early-exits: pattern evaluated under an implicit LIMIT 1).
let yes = sparq_engine::ask(&g, "PREFIX ex: <http://ex/> ASK { ex:alice ex:knows ?x }").unwrap();
```

`Graph::load_str` ignores graph names in TriG/N-Quads (folds into the default graph). To keep named
graphs (so `GRAPH <g> {…}` / `GRAPH ?g {…}` work), use `Graph::load_dataset(text, "nquads"|"trig")`.

## Key APIs

All entry points take `&Graph` + `&str` and return `Result<_, String>` (parse + eval errors are
`String`). The result types:

- `pub struct QueryResult { pub vars: Vec<oxrdf::Variable>, pub rows: Vec<Vec<Option<oxrdf::Term>>> }`
  — `len()` / `is_empty()` count solution rows.
- `pub struct QueryBudget { pub deadline: Option<Instant> /*native only*/, pub max_rows: Option<usize>, pub max_bytes: Option<usize> }`
  — `QueryBudget::unlimited()` is the no-op default. `max_rows` caps the working-set ROW count;
  `max_bytes` (`sq-s5is`) is the byte-accounted companion — it prices row WIDTH
  (`rows × vars × size_of::<Id>()`) plus the bytes of query-computed (BIND/aggregate/CONSTRUCT)
  literals, so a few very wide rows or a huge computed literal is bounded where the row cap is
  blind. Both are coarse cooperative ceilings (checked at operator entry / per outer loop), a
  conservative LOWER bound on heap — not an exact RSS quota; whichever trips first aborts with
  `query budget exceeded (max-rows|max-bytes)`.

SELECT/ASK entry points (each has `_prepared`, `_with_budget`, and `_view` variants):

- `query(&Graph, &str) -> Result<QueryResult, String>` — SELECT (also accepts ASK, returning the
  unit-row encoding: 0 vars, one empty row iff satisfiable).
- `query_json(&Graph, &str) -> Result<String, String>` — SELECT/ASK as SPARQL-1.1 JSON.
- `query_json_chunks_with_budget(&Graph, &str, &QueryBudget) -> Result<Vec<String>, String>` —
  streamed JSON (concatenation is byte-identical to `query_json`; for HTTP bodies).
- `ask(&Graph, &str) -> Result<bool, String>` — requires an ASK query.
- `count(&Graph, &str) -> Result<usize, String>` — solution count without materialising terms.

Graph-valued forms (return `Vec<oxrdf::Triple>`; `*_ntriples` returns a serialised `String`):

- `construct` / `describe` / `construct_or_describe` (form-agnostic) / `construct_ntriples`
  / `triples_to_ntriples(&[Triple]) -> String`.

Update (data lives in the default + named graphs):

- `update(&Graph, &str) -> Result<Graph, String>` — returns a NEW graph (O(n) rebuild).
- `update_in_place(&mut Graph, &str) -> Result<(), String>` — incremental delta-overlay, O(batch);
  WAL-durable for directory-backed graphs.
- `update_in_place_capturing(&mut Graph, &str, &QueryBudget) -> Result<Vec<UpdateEffect>, String>`
  + `apply_effects(&mut Graph, &[UpdateEffect])` — apply once, capturing the RESOLVED delta, then
  replay it onto a second (e.g. durable mirror) graph WITHOUT re-executing the text. Use this when
  mirroring a sequence of updates so non-deterministic functions (`NOW()`/`RAND()`/`UUID()`/`BNODE()`)
  and `LOAD <remote>` are not re-rolled on the second graph (the server's `--persist` path uses it).

Prepared (parse/plan once, execute many):

- `PreparedQuery::parse(&str) -> Result<PreparedQuery, String>` (also `FromStr`, and
  `From<spargebra::Query>` for programmatically built algebra); `query_prepared`, `ask_prepared`,
  `count_prepared`, `construct_prepared`, `describe_prepared` (+ `_with_budget`).

Extension functions (SPARQL 17.6) and dataset views:

- `FunctionRegistry::new()` + `.register(iri, |args: &[Term]| -> Result<Term, String>)`;
  `query_with_functions(&Graph, &str, &FunctionRegistry)` (+ `_and_budget`); `with_functions(&reg, ||
  …)` scopes the registry over ANY other entry point.
- `DatasetView { base: &Graph, named: Arc<FxHashSet<Term>>, default: DefaultGraphMode }`;
  `query_view` / `ask_view` / `count_view` / `query_json_view` (+ `_with_budget`), or
  `with_view(&v, || …)`.
- *(opt-in `window-functions`)* `CustomAggregateRegistry::new()` + `.register(iri, |members:
  &[Option<Term>]| -> Result<Option<Term>, String>)`; `query_with_aggregates(&Graph, &str, &reg)`
  (+ `_and_budget`) parses+installs, `with_aggregates(&reg, || …)` scopes it. Window functions:
  `window::{WindowSpec, WindowFunction, WindowAggregate, WindowFrame, FrameUnit, FrameBound, SortKey,
  apply_window}` (programmatic pass), or `query_over(&Graph, &str)` (+ `_with_budget`) for the inline
  `OVER(…)` query syntax (a source rewrite over the engine; covers `ROW_NUMBER`/`RANK`/`DENSE_RANK`,
  the windowed aggregates `COUNT`/`SUM`/`AVG`/`MIN`/`MAX` (sq-imj8), the offset/positional
  `LAG`/`LEAD(?x[, n[, default]])` and `NTILE(n)` (sq-hqhc), `PARTITION BY`/`ORDER BY`,
  `ROWS`/`RANGE` frames, and a reusable named `WINDOW w AS (…)` clause (sq-hqhc)).

Introspection: `explain(&Graph, &str)` (plan-only) and `explain_analyze(&Graph, &str)` (plan + per-
operator execution trace).

## Common recipes

**Aggregates, GROUP BY / HAVING, subqueries** — standard SPARQL 1.1; no special API:

```rust
sparq_engine::query(&g,
    "SELECT ?p (COUNT(*) AS ?c) WHERE { ?s ?p ?o } GROUP BY ?p HAVING (COUNT(*) >= 2)").unwrap();
// COUNT/SUM/AVG/MIN/MAX/GROUP_CONCAT/SAMPLE; SUM/AVG skip unbound members.
sparq_engine::query(&g,
    "SELECT ?s WHERE { { SELECT ?s WHERE { ?s <http://ex/age> ?a } ORDER BY DESC(?a) LIMIT 1 } }").unwrap();
```

*Aggregates over an EMPTY result* (SPARQL 1.1 §18.5.1, pinned by the engine's
`aggregate_empty_semantics` tests and the W3C `agg-empty-group-*` conformance cases):

- **No `GROUP BY`** ⇒ the whole result is ONE implicit group, so you get exactly **one** row
  even when nothing matched. In it: `COUNT(*)`/`COUNT(?x)`/`SUM(?x)`/`AVG(?x)` are
  **`"0"^^xsd:integer`** (a *bound* `0`), `GROUP_CONCAT(?x)` is the empty string `""`, and
  `MIN`/`MAX`/`SAMPLE` are **unbound** (an empty-set error surfaces as an unbound cell).
  Mind the split: `SUM`/`AVG` over empty are `0`, not unbound — so `COALESCE(SUM(?x), 0)`
  returns `0` because `SUM` already returned a bound `0`.
- **With `GROUP BY`** ⇒ an empty input has **zero groups**, so you get **zero** rows (the
  single-implicit-group rule applies only when no `GROUP BY` is written).

**Property paths** (all 8 operators: `/  | ^  *  +  ?` and `!(…)` negated sets) — write them inline:

```rust
sparq_engine::query(&g,
    "PREFIX ex: <http://ex/> SELECT ?x WHERE { ex:alice ex:knows+ ?x }").unwrap();   // transitive
```

**RDF-star / RDF 1.2 quoted triples** (`<< s p o >>`) — stored structurally; patterns with variables
inside the quoted triple match. Load with the `rdf-12` feature enabled on oxrdf/oxttl (default in
this workspace). In Turtle/TriG you can write the reifying triple `<< s p o >>` (subject or object
position, optionally `<< s p o ~ reifier >>`) or the annotation block `s p o {| … |}` (which also
**asserts** the base triple); both desugar to the standard `rdf:reifies <<( s p o )>>` form. The
underlying triple TERM `<<( s p o )>>` is **object-position only** (RDF 1.2). See the SPARQL-star
functions `TRIPLE`/`isTRIPLE`/`SUBJECT`/`PREDICATE`/`OBJECT` for constructing/decomposing them.

```rust
let g = Graph::load_str(
    r#"@prefix ex: <http://ex/> . << ex:alice ex:age 30 >> ex:certainty 0.9 ."#, "turtle").unwrap();
sparq_engine::query(&g,
    "PREFIX ex: <http://ex/> SELECT ?s ?o ?c WHERE { << ?s ex:age ?o >> ex:certainty ?c }").unwrap();
// Annotation-block form (asserts ex:alice ex:age 30 AND records the certainty reifier):
let g2 = Graph::load_str(
    r#"@prefix ex: <http://ex/> . ex:alice ex:age 30 {| ex:certainty 0.9 |} ."#, "turtle").unwrap();
```

**Custom extension functions** — register Rust closures under function IRIs, then `query_with_functions`:

```rust
use oxrdf::{Literal, Term};
use sparq_engine::{query_with_functions, FunctionRegistry};

let mut reg = FunctionRegistry::new();
reg.register("http://ex/fn#double", |args: &[Term]| {
    let [Term::Literal(l)] = args else { return Err("double() expects 1 literal".into()) };
    let n: i64 = l.value().parse().map_err(|e| format!("double(): {e}"))?;
    Ok(Term::Literal(Literal::from(n * 2)))
});
let r = query_with_functions(&g,
    "PREFIX fn: <http://ex/fn#> SELECT ?d WHERE { ?s <http://ex/age> ?a . BIND(fn:double(?a) AS ?d) }",
    &reg).unwrap();
// To use the registry with ANOTHER entry point: with_functions(&reg, || sparq_engine::ask(&g, q))
```

**Custom aggregate registry** (opt-in `window-functions` feature) — register a Rust closure as a
named aggregate IRI, then call it from a real SPARQL `GROUP BY`. Unlike a scalar extension function,
the closure FOLDS the group's per-member values (`None` ≡ unbound for that member) into one term:

```rust
// Cargo.toml: sparq-engine = { version = "0.1", features = ["window-functions"] }
use oxrdf::{Literal, Term};
use sparq_engine::{query_with_aggregates, CustomAggregateRegistry};

let mut reg = CustomAggregateRegistry::new();
reg.register("http://ex/agg#product", |members: &[Option<Term>]| {
    let mut acc: i64 = 1;
    for m in members { if let Some(Term::Literal(l)) = m { acc *= l.value().parse::<i64>().map_err(|e| e.to_string())?; } }
    Ok(Some(Term::Literal(Literal::from(acc))))
});
let r = query_with_aggregates(&g,
    "PREFIX ex: <http://ex/> PREFIX agg: <http://ex/agg#> \
     SELECT ?d (agg:product(?s) AS ?p) WHERE { ?x ex:dept ?d ; ex:sales ?s } GROUP BY ?d",
    &reg).unwrap();
// query_with_aggregates DECLARES every registry IRI to the parser (so `agg:product(?s)` parses as an
// aggregate, not a scalar call) AND installs the registry for evaluation. `with_aggregates(&reg, ||..)`
// is the scoped form (composes with with_functions / with_view).
// CAVEAT: a `DISTINCT` custom aggregate (`agg:product(DISTINCT ?s)`) does not currently PARSE in the
// vendored spargebra (a parser limitation, tracked as a follow-up bead); non-DISTINCT works.
```

**Window functions** (opt-in `window-functions` feature) — **NON-STANDARD extension**: SPARQL has no
W3C-REC `OVER (PARTITION BY … ORDER BY …)` syntax, so sparq exposes windowing two ways, both following
the SQL:2003 window model (the surface Stardog / AnzoGraph expose) and neither touching the engine's
W3C-conformance SPARQL surface.

*(a) Programmatic pass over a `QueryResult`* — the full surface. Run an ordinary SELECT, then apply a
`WindowSpec` (`ROW_NUMBER` / `RANK` / `DENSE_RANK`; the offset/positional `Lag` / `Lead` / `Ntile`
(sq-hqhc); or a windowed aggregate over the whole partition, or — sq-imj8 — over an explicit
`ROWS`/`RANGE` frame). One column is appended; row order is preserved:

```rust
use sparq_engine::window::{apply_window, SortKey, WindowFunction, WindowSpec};
use oxrdf::Variable;

let result = sparq_engine::query(&g, "SELECT ?emp ?dept ?sales WHERE { … }").unwrap();
let spec = WindowSpec {
    partition_by: vec![Variable::new("dept").unwrap()],
    order_by: vec![SortKey::desc(Variable::new("sales").unwrap())],
    function: WindowFunction::Rank,            // RANK() over (PARTITION BY ?dept ORDER BY ?sales DESC)
    frame: None,                               // a frame applies only to an Aggregate (None = whole partition)
    new_var: Variable::new("rank").unwrap(),
};
let ranked = apply_window(&result, &spec).unwrap(); // ?rank appended; RANK has gaps after ties
// Windowed aggregate over a FRAME (sq-imj8): a running total is
//   WindowFunction::Aggregate { agg: WindowAggregate::Sum, of }
//   + frame: Some(WindowFrame::rows_running())  // ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
// `WindowFrame { unit: FrameUnit::Rows|Range, start, end }` with `FrameBound::{UnboundedPreceding,
// Preceding(n), CurrentRow, Following(n), UnboundedFollowing}`; RANGE bounds are peer-group based
// (no numeric RANGE offset). `frame: None` keeps the whole-partition default.
```

*(b) Inline `OVER(…)` query syntax* (sq-h564) — write the window directly in the SELECT projection and
run it with `query_over` / `query_over_with_budget`. This is a **source rewrite in front of the engine**
(it does NOT change the vendored `spargebra` parser): `query_over` lifts each `(FN() OVER (…) AS ?out)`
item out of the projection, runs the window-stripped SELECT through the ordinary engine, applies the
programmatic pass above, then reprojects. **Covered subset:** the ranking functions `ROW_NUMBER()`,
`RANK()`, `DENSE_RANK()`, the windowed aggregates `COUNT(?x)` / `SUM(?x)` / `AVG(?x)` / `MIN(?x)` /
`MAX(?x)` (sq-imj8 — single `?var` argument; case-insensitive), and the offset/positional functions
`LAG(?x[, n[, default]])` / `LEAD(…)` / `NTILE(n)` (sq-hqhc). A spec can be reused via a named
`WINDOW w AS (…)` clause and `OVER w` (sq-hqhc). `PARTITION BY ?v …`, `ORDER BY` over
projected variables **or computed expressions** (sq-c1jv) — e.g. `ORDER BY (?a + ?b)`, `DESC(?sales + 0)`,
`STRLEN(?s)`; each expression is bound to a fresh helper var in the rewritten inner SELECT and dropped
from the output — in both `DESC(…)` and `… DESC` spellings, an explicit `ROWS`/`RANGE` frame on an
aggregate (sq-imj8 — `ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW`, `ROWS n PRECEDING`,
`RANGE BETWEEN … AND CURRENT ROW`, etc.), multiple window columns, and `SELECT *`. A query with no
`OVER` clause is run unchanged (so `query_over` is a strict superset of `query` for non-window queries):

```rust
// Cargo.toml: sparq-engine = { version = "0.1", features = ["window-functions"] }
let r = sparq_engine::query_over(&g,
    "PREFIX ex: <http://ex/> \
     SELECT ?emp (ROW_NUMBER() OVER (PARTITION BY ?dept ORDER BY DESC(?sales)) AS ?rn) \
     WHERE { ?emp ex:dept ?dept ; ex:sales ?sales }").unwrap(); // ?rn = per-?dept rank, descending by ?sales
```

**Inline-OVER caveats (HONEST):** the `OVER` surface (and the `WINDOW` clause) is a sparq extension, not
W3C SPARQL, recognised ONLY on the `query_over` entry point (a `(… OVER …)` clause / `WINDOW` clause is
still a parse error on `query`/the standard surface). A ranking function call must be empty
(`ROW_NUMBER()`); a windowed aggregate takes a single `?var` argument (`SUM(?x)`); `LAG`/`LEAD` take a
bare `?var` plus an optional integer offset and a constant default, `NTILE` a positive integer; the
`AS ?out` alias is required. The `OVER` operand is either an inline `(…)` spec or a name bound by a
trailing `WINDOW name AS (…)` clause. `ORDER BY` keys may be projected variables OR computed expressions
(sq-c1jv) but `PARTITION BY` keys must be projected variables. A `ROWS`/`RANGE` frame is valid only on an
aggregate (a frame on a ranking or offset function errors), and `RANGE` supports only the peer-group
bounds (`UNBOUNDED …` / `CURRENT ROW`), not a numeric `RANGE n PRECEDING` offset. **Deferred
(programmatic API only, beaded):** a `DISTINCT` windowed aggregate (`COUNT(DISTINCT ?x) OVER …`), an
aggregate / `LAG`/`LEAD` argument over a computed-expression, numeric `RANGE` offsets, and `PARTITION BY`
over a computed expression.

**Budgets / timeouts / ASK-style early exit** — a `QueryBudget` is checked cooperatively at coarse
sites; tripping it fails with `"query budget exceeded (timeout)"` / `"... (max-rows)"` /
`"... (max-bytes)"`:

```rust
use sparq_engine::QueryBudget;
let budget = QueryBudget {
    max_rows: Some(10_000),
    max_bytes: Some(64 << 20), // byte-accounted companion (sq-s5is); None = off
    #[cfg(not(target_arch = "wasm32"))]
    deadline: Some(std::time::Instant::now() + std::time::Duration::from_secs(2)),
};
let r = sparq_engine::query_with_budget(&g, "SELECT * WHERE { ?s ?p ?o }", &budget);
// For existence checks prefer ask()/ASK — it streams under an implicit LIMIT 1 (cheapest early exit).
```

**Named-graph dataset view** (zero-copy restriction; a non-visible graph is indistinguishable from
an absent one):

```rust
use std::sync::Arc;
use oxrdf::{NamedNode, Term};
use sparq_engine::{DatasetView, DefaultGraphMode, FxHashSet, query_view};

let store = Graph::load_dataset(nquads, "nquads").unwrap();
let visible: Arc<FxHashSet<Term>> =
    Arc::new([Term::NamedNode(NamedNode::new("http://ex/g1").unwrap())].into_iter().collect());
let v = DatasetView { base: &store, named: visible, default: DefaultGraphMode::StoreDefault };
let r = query_view(&v, "SELECT ?s WHERE { GRAPH ?g { ?s ?p ?o } }").unwrap(); // only g1 visible
```

## Gotchas / feature flags / prerequisites

- **Errors are `String`** — both SPARQL parse errors and evaluation/type errors. SPARQL is parsed by
  (a vendored) `spargebra` with `sparql-12` + `sep-0006`.
- **Only SELECT/ASK** go through `query`/`query_json`/`count`; CONSTRUCT/DESCRIBE have their own
  functions, and a form mismatch is a clean `Err` (e.g. `ask()` on a SELECT).
- **`SELECT *`** never exposes blank-node "variables" (`_:x` in a pattern is an existential var).
- **`update` vs `update_in_place`** — `update` returns a fresh `Graph` (input borrowed, untouched);
  `update_in_place(&mut g, …)` mutates via the delta overlay (call `Graph::compact` periodically).
  `LOAD` only resolves `file://`; set the base dir with `with_load_base(path, || update(...))`.
- **SPARQL `SERVICE` federation** is the non-default `service` cargo feature (pulls `ureq`; off on
  wasm). When enabled, outbound `SERVICE` fetches go through a **default-deny SSRF egress filter**:
  an endpoint that resolves to a loopback / RFC1918 / link-local (incl. the `169.254.169.254`
  cloud-metadata IP) / unique-local / unspecified address is refused (checked on the *resolved* IP,
  so DNS rebinding can't bypass it). To federate to a trusted internal endpoint, allowlist its host
  for the scope: `with_service_egress_allow([host.to_string()], || query(&g, q))`. Public endpoints
  need no opt-in.
- **`SERVICE` bind-join (`VALUES` pushdown)** — when a `SERVICE` sub-query is the right side of a
  join (or `OPTIONAL`) whose join variables are already bound by the left side, sparq pushes a
  *block* of those bindings into the remote query as a `VALUES` clause (the brTPF/FedX "bound join")
  instead of fetching the whole remote relation and joining locally. For a selective join this slashes
  the rows the endpoint returns and the data transferred. It is **on by default** — a
  correctness-preserving optimisation of the existing `SERVICE` path (the answer is identical to the
  unbound-then-local-join path; `SILENT`, `OPTIONAL`/left-join, multi-var and empty-binding edge cases
  are all preserved) — and **falls back to the verbatim forward** when it can't apply (variable
  endpoint, no bound join var, a join key bound to a blank node). The only tuning knob is the block
  size (distinct binding tuples per remote request): **opt-in** via
  `with_service_bound_join_block_size(n, || query(&g, q))` or the `SPARQ_SERVICE_BIND_BLOCK` env var
  (default ~50). The knob never changes results — only the remote-request count vs per-request size.
- **Window functions + custom aggregate registry** are the non-default `window-functions` cargo
  feature. **Window functions are a NON-STANDARD extension** — there is no W3C-REC SPARQL `OVER`
  syntax. sparq exposes them as a programmatic pass over a `QueryResult` (the SQL:2003 model
  Stardog/AnzoGraph expose), `ROW_NUMBER`/`RANK`/`DENSE_RANK`, the offset/positional
  `LAG`/`LEAD`/`NTILE` (sq-hqhc), + a windowed aggregate over the whole partition or an explicit
  `ROWS`/`RANGE` frame (sq-imj8),
  AND as an inline `OVER(…)` query syntax via the dedicated `query_over` entry point (a *source rewrite*
  in front of the engine — it does NOT change the vendored parser, so the standard `query`/`ask`/… surface
  stays exactly SPARQL 1.1 and conformance is unaffected; `OVER` is a parse error everywhere except
  `query_over`). The inline syntax covers the three ranking functions, the windowed aggregates
  `COUNT`/`SUM`/`AVG`/`MIN`/`MAX` with `PARTITION BY`/`ORDER BY` and `ROWS`/`RANGE` frames (sq-imj8), the
  offset/positional `LAG`/`LEAD`/`NTILE` and a reusable named `WINDOW w AS (…)` clause (sq-hqhc);
  `DISTINCT`/computed-expression function arguments, numeric `RANGE` offsets and a computed-expression
  `PARTITION BY` are inline-deferred (use the programmatic API). The custom-aggregate side DOES ride real SPARQL `GROUP BY` (a declared aggregate
  IRI is part of the SPARQL 1.1 extension grammar). When the feature is off, zero window/aggregate-registry
  code compiles and the default build is byte-identical (no new dependencies). See the recipes above.
- **Default cargo features** (`parallel`, `regex`, `digest`): `regex` powers REGEX/REPLACE; `digest`
  powers MD5/SHA*; `parallel` enables rayon scan/join/sort/aggregate. The **wasm** crate
  (`sparq-wasm`) disables defaults, so on `wasm32-unknown-unknown` REGEX/hash builtins and
  `QueryBudget::deadline` are absent (`max_rows` still works); UUID()/STRUUID() are native-only.
- **Composition**: `with_functions` and `with_view` install thread-locally (propagated into rayon
  workers), nest in either order, and uninstall on return/unwind. The registry is cheap to clone
  (`Arc`-shared) — build once, reuse across queries/threads.
- **Performance**: `query_json` avoids per-cell `Term` allocation; `count` avoids materialisation;
  prefer `PreparedQuery` when running one query against many graphs (e.g. continuous/RSP queries).
- **CLI/HTTP alternative**: `sparq-cli` and `sparq-server` (W3C SPARQL Protocol, `?explain=`,
  result formats JSON/XML/CSV/TSV, `/metrics`) wrap this same surface if you don't want to embed.

## See also

- `rust-parallel-parsing` / `fused-decompress-parse` — fast/compressed RDF ingest into a `Graph`.
- `hdt-format` — loading `.hdt` archives into a `Graph` (`sparq-hdt`).
- `sparql-formal-semantics` — the algebra/semantics reference for the SPARQL fragment.
- `noir-circuit-patterns` / `verifiable-credentials-zk` / `mpc-protocols` — the ZK/MPC estate built
  on the `zk` trace seam (non-default `zk` feature; consumed by `sparq-zk`).
