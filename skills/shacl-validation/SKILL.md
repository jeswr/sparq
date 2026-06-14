---
name: shacl-validation
description: "Validate RDF data against SHACL shapes with the sparq engine: SHACL Core constraints (class, datatype, cardinality, ranges, paths, logical, node/property, qualified, closed, in/hasValue), SHACL-SPARQL sh:sparql constraints (§5.2), and custom SPARQL-based constraint components (sh:ConstraintComponent, §6) — then read the conformance/violations validation report (W3C report vocabulary as Turtle or human text). Use when an agent needs to check whether a sparq_core::Graph conforms to shapes, run shape validation, or produce a SHACL validation report in Rust."
---

# sparq-shacl-validation

Validate a data `Graph` against a shapes `Graph` and get back a `ValidationReport`
(conformance flag + per-violation results, renderable as W3C-vocabulary Turtle or
plain text). Covers the full SHACL Core component set, SHACL-SPARQL (`sh:sparql`,
§5.2), and custom SPARQL-based constraint components (`sh:ConstraintComponent`, §6).

`sparq-shacl` is an **opt-in, native-only** crate: depending on it is what turns on
SHACL. It is NOT a dependency of any other sparq crate, so the core engine and the
wasm bundle carry zero SHACL code/cost unless you pull it in.

## Quickstart

`Cargo.toml`:

```toml
[dependencies]
sparq-core  = { path = "../sparq-core" }   # or version = "0.1"
sparq-shacl = { path = "../sparq-shacl" }  # or version = "0.1"
```

```rust
use sparq_core::Graph;

let data = Graph::load_str(r#"
    @prefix ex: <http://example.org/> .
    ex:alice a ex:Person ; ex:age "thirty" .   // age is a string, not an integer
"#, "turtle").unwrap();

let shapes = Graph::load_str(r#"
    @prefix sh:  <http://www.w3.org/ns/shacl#> .
    @prefix ex:  <http://example.org/> .
    @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    ex:PersonShape a sh:NodeShape ;
      sh:targetClass ex:Person ;
      sh:property [ sh:path ex:age ; sh:datatype xsd:integer ; sh:minCount 1 ] .
"#, "turtle").unwrap();

let report = sparq_shacl::validate(&data, &shapes);
assert!(!report.conforms);                 // sh:conforms = false
assert_eq!(report.results.len(), 1);       // one DatatypeConstraintComponent violation
eprintln!("{}", report.to_text());         // human-readable
println!("{}", report.to_turtle());        // W3C sh:ValidationReport graph
```

CLI-style end-to-end run via the bundled example (exits 0 iff the data conforms):

```sh
cargo run -p sparq-shacl --example validate -- data.ttl shapes.ttl
cargo run -p sparq-shacl --example validate -- data.ttl shapes.ttl --turtle
```

## Key APIs

Top-level functions (`sparq_shacl::…`):

```rust
// Parse shapes + validate in one call.
pub fn validate(data: &Graph, shapes: &Graph) -> ValidationReport;

// Validate against an ALREADY-parsed shapes model (amortise parsing across many graphs).
pub fn validate_with_model(data: &Graph, model: &ShapesModel) -> ValidationReport;

// Load Turtle resolving relative IRIs against a base (Graph::load_str has no base param).
pub fn load_turtle_with_base(text: &str, base: &str) -> Result<Graph, String>;

// Build a Graph from already-parsed oxrdf::Triples.
pub fn graph_from_triples<I: IntoIterator<Item = oxrdf::Triple>>(triples: I) -> Graph;
```

`ShapesModel` (`sparq_shacl::ShapesModel`):

```rust
pub fn ShapesModel::parse(shapes_graph: &Graph) -> ShapesModel;   // parse once, reuse
```

`ValidationReport` (`sparq_shacl::ValidationReport`):

```rust
pub conforms: bool;                         // true iff results is empty (spec sh:conforms — counts EVERY result)
pub results: Vec<ValidationResult>;
pub fn conforms_violations_only(&self) -> bool;                       // ignore sh:Warning / sh:Info
pub fn results_with_severity<'a>(&'a self, severity: &'a str)         // full IRI, e.g. ".../shacl#Warning"
    -> impl Iterator<Item = &'a ValidationResult>;
pub fn to_turtle(&self) -> String;          // W3C report vocabulary (valid, round-trippable Turtle)
pub fn to_text(&self) -> String;            // human-readable summary
```

`ValidationResult` (`sparq_shacl::ValidationResult`) — all fields public:

```rust
pub focus_node: oxrdf::Term;
pub path: Option<sparq_shacl::Path>;        // sh:resultPath (property shapes / sh:closed)
pub value: Option<oxrdf::Term>;             // offending value node
pub source_shape: oxrdf::Term;
pub source_component: String;               // constraint-component IRI, e.g. ".../MinCountConstraintComponent"
pub severity: String;                       // severity IRI (default ".../shacl#Violation")
pub messages: Vec<oxrdf::Term>;             // sh:message literals
pub default_message: String;
pub fn effective_messages(&self) -> Vec<oxrdf::Term>;   // messages, or a generated default
```

`Path` (`sparq_shacl::Path`) — `Predicate | Inverse | Sequence | Alternative |
ZeroOrMore | OneOrMore | ZeroOrOne`; `path.to_turtle()` gives the Turtle path
expression used in `sh:resultPath`.

## Common recipes

**CI gating — fail on violations, allow warnings.** `report.conforms` counts every
result; use the severity-aware toggle so `sh:Warning`/`sh:Info` don't fail the build:

```rust
let report = sparq_shacl::validate(&data, &shapes);
if !report.conforms_violations_only() {
    eprintln!("{}", report.to_text());
    std::process::exit(1);
}
```

**Validate many data graphs against one shapes graph** — parse the shapes once:

```rust
let model = sparq_shacl::ShapesModel::parse(&shapes);
for data in data_graphs {
    let report = sparq_shacl::validate_with_model(&data, &model);
    // ...
}
```

**Inspect failures programmatically** instead of rendering:

```rust
for r in &report.results {
    let comp = r.source_component.rsplit(['#', '/']).next().unwrap();   // "MinCountConstraintComponent"
    println!("focus={} comp={comp} value={:?}", r.focus_node, r.value);
}
```

**SHACL-SPARQL (`sh:sparql`, §5.2)** — a constraint node carries an `sh:select`; it
runs per focus node with `$this` pre-bound (and `$PATH` on property shapes), and EACH
returned solution is one violation. `?value`→`sh:value` (defaults to the focus node
when unprojected), `?path`→`sh:resultPath`, `?message`→`sh:resultMessage`; `{?var}` /
`{$var}` templating in `sh:message`:

```rust
let shapes = Graph::load_str(r#"
    @prefix sh:  <http://www.w3.org/ns/shacl#> .
    @prefix ex:  <http://example.org/> .
    @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    ex:PersonShape a sh:NodeShape ;
      sh:targetClass ex:Person ;
      sh:sparql [
        a sh:SPARQLConstraint ;
        sh:prefixes ex:p ;
        sh:message "Age must not be negative" ;
        sh:select """SELECT $this ?value WHERE {
            $this <http://example.org/age> ?value . FILTER (?value < 0) }""" ;
      ] .
    ex:p sh:declare [ sh:prefix "ex" ; sh:namespace "http://example.org/"^^xsd:anyURI ] .
"#, "turtle").unwrap();
let report = sparq_shacl::validate(&data, &shapes);   // source_component ends with "SPARQLConstraintComponent"
```

**Custom SPARQL-based constraint component (`sh:ConstraintComponent`, §6).** Declare
the component (parameters + an `sh:ask`/`sh:select` validator) IN THE SHAPES GRAPH; it
activates on any shape that uses all its mandatory parameter predicates. Each parameter
value is pre-bound as `$paramName` alongside `$this`/`$value`:

```rust
let shapes = Graph::load_str(r#"
    @prefix sh:   <http://www.w3.org/ns/shacl#> .
    @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
    @prefix ex:   <http://example.org/> .
    # Component typed via a subclass of sh:ConstraintComponent (discovery follows rdfs:subClassOf*).
    ex:MyCC rdfs:subClassOf sh:ConstraintComponent .
    ex:MaxLenComponent a ex:MyCC ;
      sh:parameter [ sh:path ex:maxLen ] ;
      sh:validator [ a sh:SPARQLAskValidator ;
        sh:message "Value is longer than {$maxLen} characters" ;
        sh:ask "ASK { FILTER (STRLEN(STR($value)) <= $maxLen) }" ] .
    ex:S a sh:NodeShape ;
      sh:targetNode "abcdef", "ab" ;
      ex:maxLen 3 .                # using ex:maxLen activates the component on ex:S
"#, "turtle").unwrap();
// source_component is the component IRI; ASK=false → violation. sh:nodeValidator /
// sh:propertyValidator are preferred over the generic sh:validator by shape kind (§6.2.2).
```

**Relative-IRI test files / a base IRI** — `Graph::load_str` exposes no base, so use:

```rust
let g = sparq_shacl::load_turtle_with_base(&text, &format!("file://{path}")).unwrap();
```

## Gotchas / feature flags / prerequisites

- **No feature flags on this crate.** SHACL is engaged purely by depending on
  `sparq-shacl`. It transitively pulls in `sparq-engine` (to run `sh:sparql`/§6
  queries); both are **native-only and never in the wasm dependency graph**, so the
  isolation guarantee holds.
- **`sh:conforms` counts EVERY result regardless of severity** (matches the W3C suite).
  For "warnings don't fail" gating use `conforms_violations_only()` /
  `results_with_severity(..)`, NOT `conforms`.
- **Ill-formed shapes are skipped, not errored.** A shape never declared, an
  unparsable path, or an `sh:select` that fails to parse (e.g. undeclared prefix)
  contributes no results; the rest of validation still runs. `validate` never returns
  a `Result`/panics on bad shapes — so a silently-empty report can mean "no targets"
  rather than "conforms".
- **Results are NOT deduplicated** across traversal routes / component occurrences — a
  nested shape reached via two parents reports twice (intentional, matches the suite).
- **Recursion is treated as conforming.** Re-entering the same (focus, shape) pair
  counts as conforming (SHACL leaves recursion undefined); cyclic `sh:node`/`sh:property`
  terminate without stack overflow.
- **`sh:sparql` pre-binding:** `$this` (and `$PATH` on property shapes) is injected via
  an algebra-level VALUES on the parsed query — it lands below solution modifiers, so
  `LIMIT`/`ORDER BY`/`DISTINCT`/`SELECT *` behave correctly. `sh:prefixes` chases
  `sh:declare`(`sh:prefix`/`sh:namespace`) transitively through `owl:imports`.
- **§6 limits:** the W3C `sparql/component/*` suite is NOT directly runnable offline (it
  `owl:imports` the external `http://datashapes.org/dash` vocabulary); declare custom
  components inline in your shapes graph. Full `sparql/pre-binding` semantics (rejecting
  variable re-binding, `$shapesGraph`) are out of scope — see the crate's open beads (`bd list -l area:sparq-shacl`).
- **W3C conformance:** 98/98 of the core `sht:Validate` suite passes. Reproduce with
  `crates/sparq-shacl/fetch-shacl-tests.sh` then
  `cargo test -p sparq-shacl --test w3c_core` (self-skips if the gitignored suite is absent).
- §6 SPARQL-based constraint *components* are implemented and tested
  (`tests/sparql_components.rs`); the crate README documents them under
  "Supported constraint components".

## See also

- `sparql-query` — running standalone SPARQL through `sparq-engine` (what `sh:sparql`
  routes through).
- `graph-loading` / `compressed-ingest` — building the `sparq_core::Graph` you validate.
- `fused-decompress-parse`, `hdt-format` — alternative ingest paths feeding a `Graph`.
