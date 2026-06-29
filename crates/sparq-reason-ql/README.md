# sparq-reason-ql

<p>
  <a href="https://crates.io/crates/sparq-reason-ql"><img src="https://img.shields.io/crates/v/sparq-reason-ql.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-reason-ql"><img src="https://docs.rs/sparq-reason-ql/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

**EXPERIMENTAL** opt-in **OWL 2 QL** (DL-Lite_R) query-rewriting reasoner for the
[sparq](../../README.md) RDF engine. It rewrites a conjunctive SPARQL query into a **union of
conjunctive queries** that, evaluated over the **unmodified data**, returns the **certain
answers** under the schema — no materialisation, no closure storage (Calvanese et al.,
*PerfectRef*, JAR 2007). It is a **query-rewriter**, not a materialiser: it reuses the engine's
query path (the spargebra rewrite seam), so the planner and executor are unaware of reasoning.

This is a **separate crate** and the rewriter is behind an **off-by-default `experimental`
feature** — the core engine and the wasm build carry zero QL code, deps, or cost by default.

> **Soundness boundary (read this).** PerfectRef is sound + complete only for **conjunctive
> queries**. Firing it on anything else silently mis-answers. So this crate is **FAIL-CLOSED**:
> a query with `OPTIONAL` / `FILTER` / `MINUS` / `UNION` / a property path / aggregation / a
> variable predicate is **rejected as out-of-scope**, never rewritten. The regime is
> EXPERIMENTAL: it is validated against a **hand-checked DL-Lite oracle**, not graduated to a
> conformance floor.

## 🚀 Quickstart

```rust
# #[cfg(feature = "experimental")] {
use oxrdf::Triple;
use spargebra::SparqlParser;
use sparq_reason_ql::rewrite;
use std::str::FromStr;

// TBox: Manager rdfs:subClassOf Employee.
let tbox = vec![Triple::from_str(
  "<http://ex/Manager> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://ex/Employee> ."
).unwrap()];
let q = SparqlParser::new().parse_query(
  "SELECT ?x WHERE { ?x a <http://ex/Employee> }"
).unwrap();
let r = rewrite(&q, &tbox).unwrap();   // UCQ now also matches Managers (certainly Employees)
assert_eq!(r.report.disjuncts, 2);
# }
```

The CQ-shape gate, `as_conjunctive_query`, is **always present** (no feature needed): it
classifies any query as a CQ or `CqError::OutOfScope(reason)` and is the soundness keystone.

## ✨ Features

- **`rewrite` — PerfectRef baseline** *(opt-in `experimental`)* — rewrite + reduce saturation to a
  fixpoint over the positive DL-Lite_R inclusions: `rdfs:subClassOf` / `subPropertyOf`,
  `rdfs:domain` / `range` (`∃R ⊑ A`, `∃R⁻ ⊑ A`), `owl:inverseOf`, and unqualified `∃R`
  restrictions, with the existential **applicability condition** enforced explicitly.
- **`rewrite_production` — production path** *(opt-in `experimental`)* — PerfectRef **augmented**
  with bounded **tree-witness** folding (existential witnesses captured with no unbounded chase),
  then **UCQ-containment minimisation** (redundant disjuncts dropped by the homomorphism
  containment test). Returns the **same certain answers** as `rewrite` in a **smaller UCQ**.
- **Fail-closed CQ-shape gate** *(always present)* — `OPTIONAL`/`FILTER`/`MINUS`/`UNION`/paths/
  aggregation/variable-predicate queries are **rejected** as `OutOfScope`, not mis-answered.
- **Query-rewriter seam** — emits a `Union`-folded UCQ as a `spargebra::Query`, run unchanged by
  the engine; no store or planner changes.
- **Honest fragment reporting** — TBox axioms outside DL-Lite_R are counted in
  `RewriteReport::skipped_axioms`, never silently applied.

> **Minimisation is fail-closed.** UCQ containment is **NP-complete**; the homomorphism search is
> **bounded**, and on an **undecided-within-budget** check the disjunct is **KEPT**, never dropped.
> Minimisation only ever removes a disjunct **proven contained** in a retained one — so it removes
> no answers. (Dropping a non-contained disjunct would be an unsoundness bug.)

**Scope (honest):** positive-inclusion rewriting + tree-witness folding + containment minimisation.
**No consistency checking**, no qualified existentials. The regime stays **EXPERIMENTAL**: it is
oracle-tested (`tests/oracle.rs`, incl. the tree-witness + minimisation cases). It is **wired into
the conformance entailment-regime suite as experimental / OutOfScope** (sq-kuvu3, opt-in
`sparq-conformance/ql-experimental`): the rewriter runs over the `sd:EntailmentProfile pr:QL`
`sparql11/entailment` cases and the harness reports — honestly — what it computes (fail-closed
ABSTAIN / computed-equivalent evidence / computed-DIVERGENT gap), **never a graduated conformance
pass**. It is **not** graduated to a conformance floor: there is no scoreboard ratchet, so QL cannot
silently claim conformance — that graduation is a separate, deferred bead (it must sequence through
the contended conformance scoreboard; epic `sq-pbz04`). Enable the crate with `sparq-reason-ql =
{ version = "0.1", features = ["experimental"] }`.

## 📚 Learn more

- **How-to** — [`skills/inference/SKILL.md`](../../skills/inference/SKILL.md) (QL section).
- **API reference** — [docs.rs/sparq-reason-ql](https://docs.rs/sparq-reason-ql).
- **Design** — [`research/owl2-el-ql-reasoning-spike.md`](../../research/owl2-el-ql-reasoning-spike.md)
  (the QL track) and [`research/reasoner-suite-on-substrate.md`](../../research/reasoner-suite-on-substrate.md)
  §2.5 (the PerfectRef trap + the phased plan; this crate now implements phases Q1–Q3, with the
  conformance-floor graduation deferred to a follow-up scoreboard bead).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md) and [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## License

[MIT](../../LICENSE).
