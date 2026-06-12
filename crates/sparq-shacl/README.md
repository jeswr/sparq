# sparq-shacl

Opt-in **SHACL Core** validation over [`sparq_core::Graph`]s.

Parses a shapes graph into a shapes model, evaluates every SHACL Core
constraint component against a data graph by direct, index-backed permutation
scans (no SPARQL round-trip), and produces a `ValidationReport` with:

- the parsed results (`conforms` + `Vec<ValidationResult>` carrying
  focus node / result path / value / source shape / source constraint
  component / severity / messages),
- `to_turtle()` — the report as an RDF graph in the W3C SHACL
  validation-report vocabulary (valid Turtle, round-trips through a parser),
- `to_text()` — a human-readable rendering.

Like `sparq-reason`, this crate is **isolated**: it is not a dependency of any
other sparq crate, so the core engine and the wasm bundle carry zero SHACL
code, dependencies or runtime cost unless a consumer opts in.

## Status: W3C SHACL core test suite

**98 / 98 (100%)** of the `sht:Validate` entries in the core section of the
official [w3c/data-shapes](https://github.com/w3c/data-shapes) test suite
(`data-shapes-test-suite/tests/core/...`, pinned commit `b6e73695`), covering
`node`, `property`, `path`, `targets`, `misc`, `complex` (including the
SHACL-validating-SHACL meta test) and `validation-reports`.

Run it yourself (the suite is fetched into a gitignored directory; the test
self-skips when it is absent):

```sh
crates/sparq-shacl/fetch-shacl-tests.sh
cargo test -p sparq-shacl --test w3c_core -- --nocapture
```

## Supported constraint components

`sh:class`, `sh:datatype` (with XSD lexical-space ill-formedness checks),
`sh:nodeKind`, `sh:minCount`/`sh:maxCount`,
`sh:minInclusive`/`sh:minExclusive`/`sh:maxInclusive`/`sh:maxExclusive`
(numeric, string, boolean and date/time orderings, including the
timezone-presence comparability rule), `sh:minLength`/`sh:maxLength`,
`sh:pattern` (+`sh:flags`), `sh:languageIn`, `sh:uniqueLang`, `sh:equals`,
`sh:disjoint`, `sh:lessThan`, `sh:lessThanOrEquals`, `sh:not`, `sh:and`,
`sh:or`, `sh:xone`, `sh:node`, `sh:property`, `sh:qualifiedValueShape`
(+`sh:qualifiedMinCount`/`sh:qualifiedMaxCount`/
`sh:qualifiedValueShapesDisjoint`), `sh:closed` (+`sh:ignoredProperties`),
`sh:hasValue`, `sh:in`.

Targets: `sh:targetNode`, `sh:targetClass` (with `rdfs:subClassOf*` closure),
implicit class targets (a shape that is itself an `rdfs:Class`),
`sh:targetSubjectsOf`, `sh:targetObjectsOf`.

Property paths: predicate paths plus all SHACL path forms — sequence,
alternative, inverse, `zeroOrMore`/`oneOrMore`/`zeroOrOne` — evaluated by
direct graph walks (BFS closure for the recursive forms).

Also handled: `sh:severity`, `sh:message` (copied to `sh:resultMessage`),
`sh:deactivated`, recursive shape references (re-entrant validation of the
same focus/shape pair counts as conforming — SHACL leaves recursion
undefined).

## Usage

```rust
use sparq_core::Graph;

let data = Graph::load_str(r#"
    @prefix ex: <http://example.org/> .
    ex:alice a ex:Person ; ex:age "thirty" .
"#, "turtle")?;

let shapes = Graph::load_str(r#"
    @prefix sh: <http://www.w3.org/ns/shacl#> .
    @prefix ex: <http://example.org/> .
    @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    ex:PersonShape a sh:NodeShape ;
      sh:targetClass ex:Person ;
      sh:property [ sh:path ex:age ; sh:datatype xsd:integer ; sh:minCount 1 ] .
"#, "turtle")?;

let report = sparq_shacl::validate(&data, &shapes);

if !report.conforms {
    // Human-readable summary:
    eprintln!("{}", report.to_text());
    // Or the standard report graph:
    println!("{}", report.to_turtle());
}

// Severity-aware gating: `conforms` counts EVERY result (the spec's
// sh:conforms); for CI-style "warnings don't fail the build" use:
if report.conforms_violations_only() { /* only sh:Warning / sh:Info results */ }
```

A CLI-style end-to-end run via the bundled example
([`examples/validate.rs`](examples/validate.rs)):

```sh
cargo run -p sparq-shacl --example validate -- data.ttl shapes.ttl
# or the report graph instead of the text rendering:
cargo run -p sparq-shacl --example validate -- data.ttl shapes.ttl --turtle
```

It prints the report and exits 0 iff the data conforms.

For repeated validation of many data graphs against one shapes graph, parse
the shapes once:

```rust
let model = sparq_shacl::ShapesModel::parse(&shapes);
for data in data_graphs {
    let report = sparq_shacl::validate_with_model(&data, &model);
}
```

## Scope and non-goals

- **SHACL Core only.** SHACL-SPARQL (`sh:sparql` constraints, SPARQL-based
  constraint components) is out of scope; see `TODO.md`.
- Validation results are **not deduplicated** across traversal routes /
  component occurrences — matching the test suite's expectations (a nested
  shape reached through two parents reports twice).
