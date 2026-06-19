<!-- [OPUS-4.8] sq-inzv: README brought to template. -->
# sparq-shacl

<p>
  <a href="https://crates.io/crates/sparq-shacl"><img src="https://img.shields.io/crates/v/sparq-shacl.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-shacl"><img src="https://docs.rs/sparq-shacl/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

Opt-in **SHACL Core + SHACL-SPARQL (`sh:sparql`)** validation over
[`sparq_core::Graph`]s. Constraints are evaluated by direct, index-backed permutation
scans (no SPARQL round-trip) except `sh:sparql`, which routes its `sh:select` through
`sparq-engine`. Validation yields a `ValidationReport` — `conforms` + per-result detail,
`to_turtle()` (the W3C SHACL report vocabulary), and `to_text()`.

Like `sparq-reason`, this crate is **isolated**: no other sparq crate depends on it, so
the core engine and the default wasm bundle carry zero SHACL code. The browser/JS
consumer opts in through `sparq-wasm`'s non-default `shacl` feature
(`Store.validate(data, shapes, format)`, a drop-in for `rdf-validate-shacl`).

## 🚀 Quickstart

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    eprintln!("{}", report.to_text());   // or report.to_turtle() for the report graph
}
// `conforms` counts EVERY result; for "warnings don't fail the build":
if report.conforms_violations_only() { /* only sh:Warning / sh:Info results */ }
# assert!(!report.conforms);
# Ok(()) }
```

For many data graphs against one shapes graph, parse once with
`ShapesModel::parse(&shapes)` and call `validate_with_model`. A CLI run is
`cargo run -p sparq-shacl --example validate -- data.ttl shapes.ttl [--turtle]`.

## ✨ Features

- **W3C core conformance — 98 / 98 (100%)** of the `sht:Validate` entries in the core
  section of the official [w3c/data-shapes](https://github.com/w3c/data-shapes) suite
  (pinned commit `b6e73695`). Run it: `crates/sparq-shacl/fetch-shacl-tests.sh` then
  `cargo test -p sparq-shacl --test w3c_core`.
- **SHACL 1.2 core constraints** — `sh:memberShape`, `sh:uniqueMembers`,
  `sh:{min,max}ListLength`, `sh:uniqueValuesFor`, `sh:closed sh:ByTypes`, and the
  disjunctive-list forms of `sh:datatype`/`sh:nodeKind` (sq-vg3y). The 1.2 harness
  passes strictly **44/45** with `shacl-af` off — the one SKIP is `nodeByExpression-001`,
  a `shacl-af` constraint — and **45/45** with it on; it stays SKIP-tolerant only for a
  constraint predicate the build still lacks, never masking a regression.
- **SHACL-SPARQL + custom §6 components** — `sh:sparql` (§5.2) and SPARQL-based
  constraint *components* (custom `sh:ConstraintComponent` with `sh:parameter` /
  `sh:validator`, §6), pinned by the W3C `sparql/*` sub-suites.
- **SHACL Advanced Features (opt-in `shacl-af`)** — a rule **inference** step
  (`sh:rule` / `sh:values`, not part of `validate`): `sh:TripleRule`, `sh:SPARQLRule`,
  value rules, the SHACL 1.2 node-expression algebra + function registry, and the
  `sh:expression` / `sh:nodeByExpression` constraints. Off ⇒ none of it compiles.
- **Differential fuzzing** — a deterministic SplitMix64 fuzzer
  (`tests/diff_fuzz.rs`) cross-checks reports against pluggable reference engines
  (pySHACL, Apache Jena SHACL, and the Zazuko / `rdf-validate-shacl` Node engines); it
  is `#[ignore]`d and runs as a nightly CI lane, skipping cleanly when no reference
  resolves.
- **Full path + target support** — all SHACL path forms (sequence / alternative /
  inverse / `zeroOrMore`·`oneOrMore`·`zeroOrOne`), `sh:targetNode`/`Class`/
  `SubjectsOf`/`ObjectsOf` + implicit class targets, plus `sh:severity` / `sh:message` /
  `sh:deactivated`.
- **SHACL Compact Syntax parser (opt-in `scs`)** — `parse_scs(text, base)` /
  `parse_scs_to_graph(text, base)` turn a [W3C SHACL Compact Syntax](https://w3c.github.io/shacl/shacl-compact-syntax/)
  document into the same shapes triples `validate` consumes (the *parse* direction of the
  SCS surface; the *display* direction ships client-side in the site). It round-trips
  **32/32** of the vendored W3C `shacl12-cs` valid fixtures graph-isomorphically
  (`cargo test -p sparq-shacl --features scs --test scs_roundtrip`). A hand-rolled lexer +
  recursive-descent parser over the `SHACLC.g4` grammar — directives (`BASE`/`IMPORTS`/
  `PREFIX` with the four implicit prefixes), `shape`/`shapeClass`, path expressions
  (`^` / `/` / `|` / `?*+` / grouping), `[min..max]` counts, `nodeKind`, bare-IRI
  `sh:datatype`-vs-`sh:class`, `@`shape-refs, `param=value`, `!`negation (`sh:not`),
  `|` disjunction (`sh:or`), nested `{...}` shapes and `[ ... ]` arrays. Adds **zero new
  dependencies**; unsupported constructs return a typed `ScsError` (never a silent
  mis-parse). Off ⇒ none of it compiles. The wasm `Store` binding is deferred (sq-quly).

## 📚 Learn more

- **How-to** — [`skills/shacl-validation/SKILL.md`](../../skills/shacl-validation/SKILL.md)
  (the exhaustive supported-constraint list and the report shape).
- **API reference** — [docs.rs/sparq-shacl](https://docs.rs/sparq-shacl).
- **Spec** — W3C SHACL (Core, SPARQL, Advanced Features); the implemented surface is
  pinned by the suites in [`tests/`](tests/).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md).

## Scope and non-goals

The remaining out-of-scope item is the full `sparql/pre-binding` semantics (rejecting
variable re-binding, `$shapesGraph`) — see `bd list -l area:sparq-shacl`. Validation
results are **not deduplicated** across traversal routes (matching the test suite: a
nested shape reached through two parents reports twice), and re-entrant recursion on the
same focus/shape pair counts as conforming (SHACL leaves recursion undefined).

## License

[MIT](../../LICENSE).
