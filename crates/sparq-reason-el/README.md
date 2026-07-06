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
- **Safe nominals — CR6** — singleton `owl:oneOf` (`{a}`) and object-valued `owl:hasValue`
  (`∃r.{a}`) classify (the reachability-guarded merge rule; every derivation sound, with
  negative tests pinning the guard). Completeness is claimed for typical safe usage, NOT for
  every EL++ nominal interplay; ABox `rdf:type` assertions are not internalized (TBox only).
- **Self-restrictions — CR-Self** — `owl:hasSelf "true"^^xsd:boolean` + `owl:onProperty r` is the
  EL profile's `ObjectHasSelf` (`∃r.Self`, local reflexivity): `X ⊑ ∃r.Self ⇒ (X,X) ∈ R(r)` and
  `∃r.Self ⊑ D` threads via the self-concept atom + CR1. A general `(X,X)` link from `X ⊑ ∃r.X`
  never triggers it (the load-bearing side-condition). Any malformed shape (non-`true`/non-boolean
  object, missing `owl:onProperty`) stays a counted skip. Under `abox` the self-loop realises as
  the property assertion `a r a` (the WG `New-Feature-SelfRestriction-001` "Peter likes Peter").
- **RBox role automaton** *(opt-in `rbox` feature, Phase E2)* — `rdfs:subPropertyOf` role
  inclusions (**CR10**), `owl:propertyChainAxiom` + `owl:TransitiveProperty` compositions
  (**CR11**), incl. the SNOMED-critical right-identity `r ∘ s ⊑ s`. OFF by default: zero
  role-automaton code without it, and RBox axioms are then left unapplied (roles compared for
  equality only).
- **Transitive reduction → Hasse diagram** *(opt-in `hasse` feature, Phase E3)* — `DirectHierarchy`
  reduces the full closure to the **direct (immediate) subsumers**, collapses **equivalence
  cliques**, and `classify_hasse_graph` emits the COMPACT taxonomy (direct `rdfs:subClassOf` +
  `owl:equivalentClass`) — O(N) Hasse edges on a deep chain instead of the O(N²) full closure.
- **Concrete domains — CR7–CR9** *(opt-in `cdomain` feature)* — faceted datatype restrictions
  (`owl:onDatatype` + `owl:withRestrictions` with min/max{In,Ex}clusive) over `xsd:decimal` /
  `xsd:integer` and its derived types (implicit bounds included), plus exact-numeric
  `DataHasValue` / singleton `DataOneOf` points — decided EXACTLY on the shared
  `sparq_substrate::numeric` value tower (never lossy f64). An **empty** range makes the class
  unsatisfiable (clash via CR5); a **proven** value-space containment threads subsumptions
  through data-property existentials. Anything not exactly decidable (pattern/length/digit
  facets, float/double or non-numeric bases/values, `owl:onDataRange`, complement) is
  **deferred, never guessed** — a wrong sat/unsat verdict would be an unsound entailment.
- **ABox realisation & whole-ontology consistency — CR6 nominals** *(opt-in `abox` feature)* —
  internalizes `ClassAssertion` (`a rdf:type C` ⇒ `{a} ⊑ C`) and `ObjectPropertyAssertion`
  (`a p b` ⇒ `{a} ⊑ ∃p.{b}`) as SAFE-NOMINAL axioms over CR6, then reads off derived instance
  typings (`a rdf:type C`), individual equality (`a owl:sameAs b`) and a whole-ontology
  `inconsistent` verdict (`{a} ⊑ ⊥` or a global `⊤ ⊑ ⊥`) via the additive `realize` /
  `realize_graph` entry — every emitted fact holds in EVERY model. The TBox
  `Classifier::classify` / `classify_graph` stay **byte-identical** (they never internalize
  assertions). Data-property assertions and non-EL class expressions stay counted skips
  (`Report::skipped_assertions`, fail-closed — never a guessed typing).
- **Honest fragment reporting** — class axioms outside the active fragment are counted in
  `Report::skipped_axioms`, never silently misapplied. Without `cdomain` that includes ALL
  concrete-domain shapes; with it, only the unsupported remainder above.

**Scope:** EL+⊥ + safe nominals/CR6 + self-restrictions/CR-Self (E1, default), EL+ role reasoning (E2, `rbox`), transitive
reduction (E3, `hasse`), exact-numeric concrete domains (CR7–CR9, `cdomain`), ABox realisation +
whole-ontology consistency (`abox`); concurrency is **E4**. Constructs outside EL entirely (union /
complement / `allValuesFrom` / cardinality / multi-individual `oneOf`) are always skipped. The
classifier is **single-threaded**. Enable with
`sparq-reason-el = { version = "0.1", features = ["rbox", "hasse", "cdomain", "abox"] }`. The
`snomed_go_scale_bench` example (`--features rbox,hasse`) is a *relative* (dimensionless, no
hard-coded ms) end-to-end scaling check confirming normalise + RBox + Hasse compose with no
hidden quadratic.

## 📚 Learn more

- **How-to** — [`skills/inference/SKILL.md`](../../skills/inference/SKILL.md) (EL section).
- **API reference** — [docs.rs/sparq-reason-el](https://docs.rs/sparq-reason-el).
- **Design** — [`research/owl2-el-ql-reasoning-spike.md`](../../research/owl2-el-ql-reasoning-spike.md)
  (why EL, the RL-incompleteness proof, the phased plan).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md) and [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## License

[MIT](../../LICENSE).
