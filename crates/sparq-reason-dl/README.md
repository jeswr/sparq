# sparq-reason-dl

<p>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

**Opt-in OWL 2 Direct-Semantics support** for the [sparq](../../README.md) RDF engine — a layered,
fail-closed Direct-Semantics checker. **Built: L1** (structural **ALCH** model + fail-closed
reverse RDF mapping), **L2** (syntactic EL/QL/RL profile checker), **L3** (the terminating
**ALCH tableau**), and **L4** (fragment dispatch + entailment-by-refutation, behind the opt-in
`dispatch` feature). A **separate crate** — `sparq-core`, `sparq-engine` and the wasm build carry
zero Direct-Semantics code, deps or cost by default.

> **Honest scope.** This crate does **not** implement OWL 2 DL. The tableau is sound and complete
> **only for the exact ALCH fragment** (named classes, ⊤/⊥, ⊓/⊔/¬, ∃/∀ over named object
> properties, GCIs, `rdfs:subPropertyOf`, ground ABox) — extended by the opt-in `dl_transitive` /
> `dl_datatypes` features below. Anything else is refused fail-closed as `Unknown(OutOfFragment)`
> BEFORE reasoning; count-budget exhaustion is `Unknown(ResourceBudget)`, never a verdict. The L4
> dispatch narrows its RL/EL verdicts behind explicit completeness guards; the design record
> carries the deferral ledger.

## 🚀 Quickstart

```rust
use sparq_core::Graph;
use sparq_reason_dl::extract;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
const TTL: &str = r#"
  @prefix : <http://ex/> . @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
  :A rdfs:subClassOf :B .
"#;
let (dict, triples) = Graph::parse_to_triples(TTL, "turtle")?;

// Fail-closed: the whole graph maps into the ALCH structural model, or the FIRST
// out-of-fragment / malformed triple comes back as a typed `ExtractError`
// (`OutOfFragment` / `DataConstruct` / `MalformedList` / … — never a partial model).
let onto = extract(&dict, &triples)?;
assert_eq!(onto.len(), 1); // one SubClassOf axiom
# Ok(()) }
```

## ✨ Features

- **Structural ALCH model** (`model`) — `Axiom` / `ClassExpression` / `ObjectPropertyExpression`
  enums: named classes, `owl:Thing`/`owl:Nothing`, ⊓, ⊔, ¬, ∃R.C, ∀R.C over **named object
  properties**; GCIs, `owl:equivalentClass`, `owl:disjointWith`, `rdfs:subPropertyOf`,
  `rdfs:domain`/`rdfs:range`, and a **ground ABox**. Purely structural — no semantics at L1.
- **Fail-closed reverse RDF mapping** (`extract`) — maps `(Dict, &[[Id; 3]])` into the model per
  the W3C *Mapping to RDF Graphs* tables restricted to ALCH. **A single triple outside the
  fragment aborts the whole extraction** with a typed `ExtractError`: a silently dropped axiom
  can flip a consistency verdict. Understood in full, or refused.
- **Structured rejection taxonomy** — `ExtractError` distinguishes unsupported logical
  constructs, data constructs, malformed lists/expressions and unclassifiable predicates;
  annotations, declarations and ontology headers are ignored (no ALCH-logical import).
- **Forward RDF renderer** (`render`, sq-pbz04.4.7) — `render_to_triples` maps the model back to
  OWL RDF triples (the inverse of `extract`), enabling full-fragment round-trip testing
  (`RDF → extract → render → extract` ≡ same model) and `render_to_turtle` diagnostics.

**Profile checker (L2, bead sq-pbz04.4.2):** `profile::profiles(onto)` checks OWL 2 EL/QL/RL
membership via a purely syntactic grammar walk (W3C OWL 2 Profiles §2/§3/§4), returning a
`ProfileSet` of `Membership::In` / `NotIn(reason)` / `Unknown(reason)`. Terminating by
construction, no semantic reasoning.

**ALCH tableau (L3, bead sq-pbz04.4.3):** `tableau::consistency` and
`tableau::class_satisfiability` decide consistency / class satisfiability via a
completion-forest tableau — GCI internalisation, rules matched modulo the `subPropertyOf`
closure, **ancestor subset blocking** (sufficient precisely because ALCH has no inverse roles),
backtracking over ⊔-branches; `tableau::consistency_from_extraction` is the fail-closed
RDF-level entry. Verdicts are tri-state; budgets are deterministic COUNTS (wall-clock banned).
The termination + soundness + completeness argument (Baader–Sattler 2001) is in the `tableau`
module docs; `nnf` supplies the NNF + finite subexpression closure it rests on.

**Fragment dispatch (L4, bead sq-pbz04.4.4, feature `dispatch`):**
`check::DirectChecker::consistency` dispatches an extracted ontology to the first matching
branch — **RL** (`sparq-reason` materialization + clash scan; `Inconsistent` sound via
*checked* Theorem PR1 preconditions, `Consistent` only past a divergence guard over the
constructs implicated in the documented RL rule-set divergences), **EL** (`sparq-reason-el`
classification; verdicts only for a pure ⊤-free EL+⊥ TBox with zero skipped axioms), **QL**
(opt-in `dispatch_ql`: `sparq-reason-ql`'s DL-Lite_R violation-query checker, whose OWN capture
accounting owns the verdict; without it, always `Unknown(QlConsistencyPending)`), or the
**ALCH tableau** (complete for the fragment). `check::DirectChecker::entailment` decides
premise ⊨ conclusion per conclusion-axiom by sound refutation encodings on the tableau (GCI /
class-assertion / the fresh-class trick + its role-subsumption and transitivity lifts); an
unencoded kind abstains. A **conclusion blank-node individual is read EXISTENTIALLY**
(sq-pbz04.4.13): a tree-shaped anonymous assertion set rolls up into an `∃`-class assertion
decided soundly; a non-rollable shape abstains `ConclusionAnonymousIndividual`, never a
skolem-constant `NotEntailed`. Every verdict carries its `Branch`; every guard fails closed.
`dispatch` pulls `sparq-reason` + `sparq-reason-el` — **off by default**.

**Transitive roles (opt-in `dl_transitive`, OFF by default, sq-zfwzq):** `owl:TransitiveProperty`
via the tableau's ∀₊-rule, argued in the `tableau` docs §5a. A transitivity-bearing premise may
supply the established role kind for a declaration-free conclusion assertion; an unknown
conclusion predicate still fails closed.

**Concrete domain (opt-in `dl_datatypes`, OFF by default, sq-pbz04.4.19):** ALCH(**D**) over an
*admitted sub-lattice* of the OWL 2 datatype map (`cdomain::Datatype`). L1 recognises data
properties and datatype IRIs in data-range positions; the tableau grows **concrete nodes** with
`∃_D`/`∀_D` rules and a clash decided by the EXACT `cdomain::satisfiable` oracle, so value-space
disjointness is MODELLED — two disjoint datatype ranges make a property's extension provably
empty (the `WebOnt-I5.3-015` mechanism). Argued in the `tableau` docs §5b (the unary,
feature-path-free `ALC(D)` restriction) + design record §5c, the lattice pinned by a
differential-parity lane against the repo's D-entailment value seam. Facets,
`owl:datatypeComplementOf`, enumerated data ranges, data-property ASSERTIONS, unadmitted
datatypes and conclusion-side data-property entailment stay refused in BOTH feature states.

**Deferred** (each rejected, never mis-mapped): inverses, cardinality, nominals, `sameAs`, keys,
and the datatype constructs above — see the design record.

## 📚 Learn more

- **How-to** — [`skills/inference/SKILL.md`](../../skills/inference/SKILL.md) (Direct-Semantics section).
- **Design** — [`research/owl2-direct-semantics-scoping.md`](../../research/owl2-direct-semantics-scoping.md)
  (the layered fail-closed scope, the ALCH fragment, and the deferral ledger).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md) and [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## License

[MIT](../../LICENSE).
