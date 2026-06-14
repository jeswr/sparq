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
- `pub struct QueryBudget { pub deadline: Option<Instant> /*native only*/, pub max_rows: Option<usize> }`
  — `QueryBudget::unlimited()` is the no-op default.

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

**Property paths** (all 8 operators: `/  | ^  *  +  ?` and `!(…)` negated sets) — write them inline:

```rust
sparq_engine::query(&g,
    "PREFIX ex: <http://ex/> SELECT ?x WHERE { ex:alice ex:knows+ ?x }").unwrap();   // transitive
```

**RDF-star / RDF 1.2 quoted triples** (`<< s p o >>`) — stored structurally; patterns with variables
inside the quoted triple match. Load with the `rdf-12` feature enabled on oxrdf/oxttl (default in
this workspace):

```rust
let g = Graph::load_str(
    r#"@prefix ex: <http://ex/> . << ex:alice ex:age 30 >> ex:certainty 0.9 ."#, "turtle").unwrap();
sparq_engine::query(&g,
    "PREFIX ex: <http://ex/> SELECT ?s ?o ?c WHERE { << ?s ex:age ?o >> ex:certainty ?c }").unwrap();
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

**Budgets / timeouts / ASK-style early exit** — a `QueryBudget` is checked cooperatively at coarse
sites; tripping it fails with `"query budget exceeded (timeout)"` / `"... (max-rows)"`:

```rust
use sparq_engine::QueryBudget;
let budget = QueryBudget {
    max_rows: Some(10_000),
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
