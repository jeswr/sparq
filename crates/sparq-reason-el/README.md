# sparq-reason-el

<p>
  <a href="https://crates.io/crates/sparq-reason-el"><img src="https://img.shields.io/crates/v/sparq-reason-el.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-reason-el"><img src="https://docs.rs/sparq-reason-el/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

**Opt-in OWL 2 EL classifier** for the [sparq](../../README.md) RDF engine — a
consequence-based reasoner that computes the **complete `rdfs:subClassOf` subsumption
lattice** of an EL ontology, the one thing OWL 2 RL (`sparq-reason`) is **sound but silently
incomplete** for.

RL has no rule that reasons *through* an existential successor, so running `--reason owl` over
a biomedical EL ontology (GO, ChEBI, SNOMED-style) returns a hierarchy that silently omits
subsumptions like `A ⊑ D` from `A ⊑ ∃r.B`, `B ⊑ C`, `∃r.C ⊑ D` (Krötzsch, ISWC 2012). EL
closes that gap with a deterministic least-fixpoint saturation (PTIME). This is a **separate
crate** — the core engine and the wasm build carry zero EL code, deps, or cost by default.

## 🚀 Quickstart

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use sparq_core::Graph;
use sparq_reason_el::classify_graph;

const TTL: &str = "<http://ex/a> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://ex/b> .";

// Materialize the COMPLETE subsumption lattice as rdfs:subClassOf triples, in place.
let (mut dict, mut triples) = Graph::parse_to_triples(TTL, "turtle")?;
let report = classify_graph(&mut dict, &mut triples); // emits derived subsumptions
let g = Graph::from_parts(dict, triples);             // then query as usual
# let _ = (g, report);
# Ok(()) }
```

For a typed view (super-classes, subsumption test, unsatisfiable classes) use
`Classifier::classify`, which returns a `ClassHierarchy` without mutating the graph.

## ✨ Features

- **Complete EL+⊥ classification** — normalizes the TBox to the four Baader–Brandt–Lutz normal
  forms, then saturates `S(C)` / `R(r)` under completion rules **CR1–CR5** to a fixpoint.
- **CR4 existential traversal** — the load-bearing rule RL lacks; reasons through `∃r` links.
- **Same dict/Graph seam as RL** — emits the lattice as `rdfs:subClassOf` triples queryable by
  plain BGP eval; no store changes.
- **Unsatisfiable-class detection** — `owl:disjointWith` clashes surface `C ⊑ owl:Nothing`.
- **Honest fragment reporting** — axioms outside EL+⊥ (unionOf / cardinality / RBox / …) are
  counted in `Report::skipped_axioms`, never silently misapplied.

**Scope (MVP, Phase E1):** EL+⊥ minus RBox. RBox / property chains / transitive roles (CR10/
CR11) are **Phase E2**; transitive reduction + scale **E3**; concurrency **E4**; nominals +
concrete domains are deferred. The classifier is **single-threaded**.

## 📚 Learn more

- **How-to** — [`skills/inference/SKILL.md`](../../skills/inference/SKILL.md) (EL section).
- **API reference** — [docs.rs/sparq-reason-el](https://docs.rs/sparq-reason-el).
- **Design** — [`research/owl2-el-ql-reasoning-spike.md`](../../research/owl2-el-ql-reasoning-spike.md)
  (why EL, the RL-incompleteness proof, the phased plan).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md) and [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## License

[MIT](../../LICENSE).
