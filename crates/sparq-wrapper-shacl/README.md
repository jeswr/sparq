# sparq-wrapper-shacl

Generate Rust object models from SHACL shapes. Shapes are parsed by
`sparq-shacl` — the workspace's one SHACL parser — then lowered to a
deterministic IR and emitted as a standalone, dependency-free Rust module.

> Model: Fable 5 [FABLE-5] (sq-1rg2q.12).

## 🚀 Quickstart

```rust
use sparq_wrapper_shacl::{lower, emit};

let shapes = sparq_shacl::load_turtle_with_base(
    r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        @prefix ex: <http://example.org/> .
        ex:PersonShape a sh:NodeShape ;
            sh:targetClass ex:Person ;
            sh:closed true ;
            sh:property [ sh:path ex:name  ; sh:datatype xsd:string ;
                          sh:minCount 1 ; sh:maxCount 1 ] ;
            sh:property [ sh:path ex:age   ; sh:datatype xsd:long ; sh:maxCount 1 ] ;
            sh:property [ sh:path ex:knows ; sh:class ex:Person ] .
    "#,
    "http://example.org/",
)?;
let model = sparq_shacl::ShapesModel::parse(&shapes);

let ir = lower(&model)?;
assert_eq!(ir.types[0].name, "PersonShape");
assert_eq!(emit(&ir), emit(&lower(&model)?)); // deterministic

let source = emit(&ir);
assert!(source.contains("pub name: String"));          // minCount 1, maxCount 1
assert!(source.contains("pub age: Option<i64>"));      // maxCount 1
assert!(source.contains("pub knows: Vec<Ref<PersonClass>>"));
assert!(source.contains("ALLOWED_PREDICATES"));        // sh:closed whitelist
# Ok::<(), Box<dyn std::error::Error>>(())
```

Write the emitted string to `<name>.rs` and pull it in with `mod <name>;`.

## ✨ Features

- **Cardinality.** `sh:maxCount 1` → `Option<T>`; `sh:minCount >= 1` with
  `sh:maxCount 1` → a required `T`; every other cardinality → `Vec<T>` with the
  bounds retained and checked at load time.
- **Value types.** `sh:datatype` → a checked scalar — a Rust primitive only
  where the whole value space fits, so `xsd:long` is an `i64` while unbounded
  `xsd:integer` and `xsd:decimal` keep their lexical form — `sh:class` → a typed
  `Ref<M>` reference, `sh:node` → a nested generated type (boxed, so recursive
  shapes have a finite Rust type), `sh:closed true` → a predicate whitelist plus
  a loader that rejects anything outside it.
- **Checked, not just typed.** The Rust type is the representation; the loader
  checks the datatype IRI **and** the lexical form against the datatype's own
  value space for the XSD boolean, floating-point, decimal and integer families.
  So `"not-an-integer"^^xsd:integer` is rejected even though it keeps its lexical
  form, and `"128"^^xsd:byte` is rejected even though it parses as an `i64`.
  Datatypes with no mechanical lexical space — `xsd:string`, `xsd:anyURI`, the
  date/time family, `rdf:langString`, anything outside XSD — are taken as given.
- **Deterministic.** The IR is totally ordered by content, never by shapes-graph
  traversal order, so the emitted bytes are reproducible.
- **Typed failure.** Ill-formed or contradictory shapes — an `sh:minCount` above
  its `sh:maxCount`, `sh:datatype` beside `sh:class`, a non-predicate `sh:path`,
  a name clash — return a `LowerError` instead of silently dropping meaning.
- **No dependencies in the output.** The emitted module needs only `std`; feed it
  triples from `sparq-core`, `sparq-wrapper`, or any other source.

## ⚠️ Scope

The generated code is a typed **reader**, not a SHACL validator. Only the
structural components above are represented; `sh:pattern`, `sh:minInclusive`,
`sh:in`, `sh:or`, `sh:qualifiedValueShape`, `sh:sparql` and the rest are **not**
enforced by it — validate with `sparq_shacl::validate` before trusting a graph.
Two limits inside the generated loader are likewise deliberate: `sh:class` is
checked against **direct** `rdf:type` triples only (no `rdfs:subClassOf`
closure, so a value typed solely as a subclass is rejected here yet conforms
under SHACL), and nesting stops at `MAX_NESTING_DEPTH` so cyclic data errors
rather than recursing forever.

## 📚 Learn more

- [`skills/rdf-wrapper/SKILL.md`](../../skills/rdf-wrapper/SKILL.md) — the
  wrapper surface this code generation belongs to.
- [`skills/shacl-validation/SKILL.md`](../../skills/shacl-validation/SKILL.md) —
  validating a data graph against the same shapes.
- [`crates/sparq-shacl`](../sparq-shacl/README.md) — the shapes parser and
  validator this crate builds on.

## License

MIT.
