# sparq-reason-dl

<p>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

**Opt-in OWL 2 Direct-Semantics support** for the [sparq](../../README.md) RDF engine. This is
**layer L1** of a layered, fail-closed Direct-Semantics checker: a purely **structural OWL
model** for the **ALCH** fragment plus a **fail-closed reverse RDF mapping** (RDF graph → typed
model). It is a **separate crate** — `sparq-core`, `sparq-engine`, and the wasm build carry zero
Direct-Semantics code, deps, or cost by default; DL is engaged only by depending on this crate.

> **Honest scope.** This crate does **not** implement OWL 2 DL. L1 is a **scoped ALCH-fragment
> structural layer with no reasoning at all** — it turns an RDF graph into a model it understood
> *in full*, or refuses it. The profile checker, ALCH tableau, and dispatch checker land in later
> beads (sq-pbz04.4.2 / .3 / .4). See the design record for the full scope and deferral ledger.

## 🚀 Quickstart

```rust
use sparq_core::Graph;
use sparq_reason_dl::{extract, ExtractError};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
const TTL: &str = r#"
  @prefix : <http://ex/> . @prefix owl: <http://www.w3.org/2002/07/owl#> .
  @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
  :A rdfs:subClassOf :B .
"#;
let (dict, triples) = Graph::parse_to_triples(TTL, "turtle")?;

// Fail-closed: the whole graph maps into the ALCH structural model, or the FIRST
// out-of-fragment / malformed triple is returned as a typed error.
match extract(&dict, &triples) {
    Ok(onto) => assert_eq!(onto.len(), 1),          // one SubClassOf axiom
    Err(ExtractError::OutOfFragment(_)) => {}       // e.g. a cardinality / nominal / inverse
    Err(_) => {}
}
# Ok(()) }
```

## ✨ Features

- **Structural ALCH model** (`model`) — `Axiom` / `ClassExpression` / `ObjectPropertyExpression`
  enums for the fragment: named classes, `owl:Thing`/`owl:Nothing`, `owl:intersectionOf` (⊓),
  `owl:unionOf` (⊔), `owl:complementOf` (¬), `owl:someValuesFrom` (∃R.C), `owl:allValuesFrom`
  (∀R.C) over **named object properties**; GCIs, `owl:equivalentClass`, `owl:disjointWith`,
  `rdfs:subPropertyOf`, `rdfs:domain`/`rdfs:range`, and a **ground ABox**. Purely structural —
  no semantics are attached at L1.
- **Fail-closed reverse RDF mapping** (`extract`) — maps `(Dict, &[[Id; 3]])` into the model per
  the W3C *Mapping to RDF Graphs* tables restricted to ALCH. **A single triple outside the
  fragment aborts the whole extraction** with a typed `ExtractError`, rather than being silently
  dropped: the downstream checker must never reason over a graph it only *partially* understood
  (a dropped axiom can flip a consistency verdict). Understood in full, or refused.
- **Structured rejection taxonomy** — `ExtractError` has five arms: `OutOfFragment` (cardinality,
  nominals, inverses, `owl:sameAs`, property characteristics, chains, keys, …), `DataConstruct`
  (datatypes / data properties — no concrete domain in L1), `MalformedList` (ill-formed
  `owl:intersectionOf`/`owl:unionOf` lists), `MalformedClassExpression` (a restriction missing
  its property/filler, conflicting shapes, cyclic nesting), and `Unclassifiable` (a predicate
  that cannot be mapped soundly without a declaration). Every arm has a diagnostic and a unit
  test; annotations, declarations, and ontology headers are recognised and ignored (they carry
  no ALCH-logical import).

**Reserved (empty in L1):** the `profile` (L2), `nnf`/`tableau` (L3), and `check` (L4) modules
are pre-declared stubs with no logic — they are populated by the later beads without touching
`lib.rs`. **Deferred** (each rejected, never mis-mapped): inverse roles, cardinality/functionality,
nominals (`owl:oneOf`/`owl:hasValue`), transitivity, `sameAs`/`differentFrom`, datatypes, keys —
with a named reason and unlock path in the design record's deferral ledger.

## 📚 Learn more

- **How-to** — [`skills/inference/SKILL.md`](../../skills/inference/SKILL.md) (Direct-Semantics section).
- **Design** — [`research/owl2-direct-semantics-scoping.md`](../../research/owl2-direct-semantics-scoping.md)
  (the layered fail-closed scope, the ALCH fragment, and the deferral ledger).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md) and [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## License

[MIT](../../LICENSE).
