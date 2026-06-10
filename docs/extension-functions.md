# Custom SPARQL functions (extension-function registry)

sparq-engine implements [SPARQL 1.1 §17.6 *Extensible Value Testing*](https://www.w3.org/TR/sparql11-query/#extensionFunctions):
a query may call any IRI as a function, and the embedding application decides
what those IRIs mean. You register plain Rust closures under function IRIs in
a `FunctionRegistry` and run queries through `query_with_functions`; the
engine dispatches unrecognised function IRIs (anything that is not a builtin
or an XSD constructor cast like `xsd:integer(?x)`) to your registry.

## Registering and calling a function

```rust
use oxrdf::{Literal, Term};
use sparq_core::Graph;
use sparq_engine::{query_with_functions, FunctionRegistry};

// ex:double(?n) — parse the argument as an integer, return its double.
let mut reg = FunctionRegistry::new();
reg.register("http://ex/fn#double", |args: &[Term]| {
    let [Term::Literal(l)] = args else {
        return Err(format!("double() expects 1 literal argument, got {}", args.len()));
    };
    let n: i64 = l.value().parse().map_err(|e| format!("double(): {e}"))?;
    Ok(Term::Literal(Literal::from(n * 2)))
});

let g = Graph::load_str("<http://ex/a> <http://ex/age> 30 .", "turtle")?;
let r = query_with_functions(&g,
    "PREFIX fn: <http://ex/fn#>
     SELECT ?d WHERE { ?s <http://ex/age> ?a . BIND(fn:double(?a) AS ?d) }",
    &reg)?;
assert_eq!(r.rows[0][0].as_ref().unwrap().to_string(),
           "\"60\"^^<http://www.w3.org/2001/XMLSchema#integer>");
```

## The API surface

| item | what |
| --- | --- |
| `ExtFn` | `Arc<dyn Fn(&[Term]) -> Result<Term, String> + Send + Sync>` — concrete RDF terms in, one concrete RDF term out |
| `FunctionRegistry` | `register(iri, f)` / `get(iri)`; cheaply cloneable (`Arc`-shared functions), `Send + Sync` — build once, reuse across queries and threads |
| `query_with_functions(graph, sparql, &reg)` | [`query`] with the registry installed |
| `query_with_functions_and_budget(graph, sparql, &reg, &budget)` | the same under a cooperative `QueryBudget` |
| `with_functions(&reg, closure)` | scopes the registry over **any** other entry point — `ask`, `construct`, `describe`, `update`, `query_json_chunks_with_budget`, `explain_analyze`, … |

```rust
use sparq_engine::with_functions;

let yes  = with_functions(&reg, || sparq_engine::ask(&g, ask_query))?;
let next = with_functions(&reg, || sparq_engine::update(&g, insert_where))?;
```

The registry is installed thread-locally for the duration of the call
(guard-scoped — it uninstalls on return *and* on error/unwind) and is
automatically propagated into the engine's rayon workers, so parallel FILTER /
BIND / aggregate evaluation sees it too.

## Semantics

- **Arguments** arrive fully materialised: computed numerics/booleans as their
  typed literals, graph terms as themselves. An *unbound* or errored argument
  is an expression error **before** your function is called.
- **Returning `Err`** (wrong arity, unparsable lexical, domain error, …) is a
  SPARQL **expression** error, never a hard query error — the row is dropped
  by a `FILTER`, the variable left unbound by a `BIND`, exactly like the
  builtin functions report bad arguments. The error message is discarded; it
  only needs to be useful to a human debugging the extension.
- **Unknown IRIs**: an IRI in neither the registry nor the builtins stays the
  hard `unsupported SPARQL function` query error — same as without a registry.
- **Precedence**: builtins and XSD constructor casts win; a registration under
  `http://www.w3.org/2001/XMLSchema#integer` cannot shadow `xsd:integer(?x)`.
- **No-registry cost**: the registry-free entry points (`query`, `ask`, …)
  never consult a registry — their behaviour and hot-path cost are unchanged.

## Ready-made registries

- **GeoSPARQL** — `sparq_geo::geof_registry()` exposes the `geof:` spatial
  functions (`geof:distance`, the `geof:sf*` simple-features relations,
  `geof:envelope`/`boundary`/`convexHull`); see
  [`crates/sparq-geo/README.md`](../crates/sparq-geo/README.md).
- **sparq-server** — the opt-in `geo` cargo feature installs that registry on
  the server's query, update and subscription endpoints
  (`cargo run -p sparq-server --features geo`); see
  [`crates/sparq-server/README.md`](../crates/sparq-server/README.md). With
  the feature off the server carries no geometry code at all.

The wasm bundle is unaffected by all of this: the registry plumbing adds no
dependencies, and `sparq-wasm` does not depend on `sparq-geo`.
