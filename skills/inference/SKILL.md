---
name: inference
description: Use when you need RDFS, OWL 2 RL, or Notation3/EYE-rule entailment over a sparq RDF graph — materialize the deductive closure (forward-chaining), query the entailed triples, maintain the closure incrementally under inserts/deletes, get derivation proof-trees (why()), or check OWL inconsistency; backed by the sparq-reason crate. For the COMPLETE OWL 2 EL class-subsumption lattice (which RL is sound-but-incomplete for), use the separate opt-in sparq-reason-el classifier (CR1-CR5 saturation). For OWL 2 QL certain-answer query rewriting (PerfectRef over DL-Lite_R, EXPERIMENTAL, fail-closed CQ-shape gate), use the separate opt-in sparq-reason-ql crate.
---

# sparq-inference

`sparq-reason` is sparq's **opt-in** reasoning crate: it forward-chains the deductive closure (RDFS, OWL 2 RL, or user-supplied Notation3 rules) over dictionary-encoded triples and **materializes** the entailed facts so querying stays exactly as fast as before. The core engine carries zero reasoning code/cost unless you depend on this crate. Reasoning works at the `[Id;3]` (RDFS/OWL) or `n3::Term` (N3) level; you wire the result back into a `sparq_core::Graph` to query it.

## Quickstart

Add the dep (native targets only — see Gotchas):

```toml
[dependencies]
sparq-core   = "0.1"
sparq-reason = "0.1"   # default features include `parallel`
```

Parse → materialize the RDFS closure in place → build a queryable graph (the canonical seam, exactly what the CLI does):

```rust
use sparq_core::Graph;
use sparq_reason::{materialize, Profile};

// 1. Parse to (Dict, triples) WITHOUT building indexes yet.
let (mut dict, mut triples) = Graph::parse_to_triples(turtle_text, "turtle")?;
let base = triples.len();

// 2. Expand `triples` in place with every entailed triple. Returns NEW triple count.
let added = materialize(Profile::Rdfs, &mut dict, &mut triples);
eprintln!("RDFS: {base} -> {} triples (+{added} entailed)", triples.len());

// 3. Build the indexed graph from the materialized closure and query as usual.
let g = Graph::from_parts(dict, triples);
# Ok::<(), String>(())
```

CLI equivalent (materialize and optionally dump the closure as N-Triples):

```bash
cargo run --release -p sparq-cli -- reason ontology.ttl turtle rdfs            # rdfs | owl | n3
cargo run --release -p sparq-cli -- reason ontology.ttl turtle owl out.nt      # write full closure
cargo run --release -p sparq-cli -- query data.ttl 'SELECT ...' --reason rdfs  # reason then query
```

## Key APIs

Re-exported from the crate root (`sparq_reason::…`):

```rust
// Which regime to materialize. OwlRl includes RDFS. `D` is opt-in (`d-entail` feature).
pub enum Profile { Rdfs, OwlRl, /* #[cfg(d-entail)] */ D }
impl Profile { pub fn parse(s: &str) -> Option<Profile>; } // "rdfs" | "owl" | "owl-rl" | "d"

// Batch materialization (expand in place; returns NEW triples; idempotent).
pub fn materialize(profile: Profile, dict: &mut Dict, triples: &mut Vec<[Id;3]>) -> usize;
pub fn materialize_rdfs (dict: &mut Dict, triples: &mut Vec<[Id;3]>) -> usize;
pub fn materialize_owl_rl(dict: &mut Dict, triples: &mut Vec<[Id;3]>) -> usize;

// D-entailment (datatype / value-space) — opt-in `d-entail` feature, `sparq_reason::dtype`.
// rdfD1 datatype-typing under a recognized datatype map; CORRECT TYPED value equality
// ("1"^^xsd:integer ≡ "1.0"^^xsd:decimal — canonical decimal, NOT an f64 fast path).
// `materialize(Profile::D, …)` uses the STANDARD map; pass a custom map via:
#[cfg(feature = "d-entail")]
pub fn materialize_d(d: &Recognized, dict: &mut Dict, triples: &mut Vec<[Id;3]>) -> usize;
#[cfg(feature = "d-entail")]
pub fn d_value_eq(lex_a: &str, dt_a: &str, lex_b: &str, dt_b: &str) -> bool; // value-space eq
#[cfg(feature = "d-entail")]
pub struct Recognized; // Recognized::standard() / ::new(iris) / ::default() (string+langString)

// OWL inconsistency check (cax-dw / cls-com / eq-diff / cls-nothing clashes) — run AFTER OWL closure.
pub fn inconsistencies(dict: &Dict, triples: &[[Id;3]]) -> Vec<String>;

// Notation3 / EYE-style forward rules (`{ premise } => { conclusion }` + builtins).
pub fn reason_n3(dict: &mut Dict, src: &str) -> Result<Vec<[Id;3]>, String>;
pub fn reason_n3_proof(dict: &mut Dict, src: &str)
    -> Result<(Vec<[Id;3]>, Vec<ProofStep>), String>;          // EYE --proof analogue
pub fn reason_n3_terms(src: &str, base: Option<&str>) -> Result<N3Closure, String>; // term-level, no Dict
pub fn reason_n3_terms_with_resolver(src, base, resolver: Option<&Resolver>) -> Result<N3Closure, String>;

// Incremental closure maintenance (closure stays == from-scratch materialize on the current base).
pub struct MaterializedGraph;     // RDFS;     mutations are dictionary-free
impl MaterializedGraph {
    pub fn new(dict: &mut Dict, base: &[[Id;3]]) -> Self;
    pub fn insert(&mut self, triples: &[[Id;3]]) -> usize;     // returns # actually added
    pub fn delete(&mut self, triples: &[[Id;3]]) -> usize;     // exact-count retraction; as cheap as insert
    pub fn contains(&self, t: &[Id;3]) -> bool;
    pub fn closure(&self) -> Vec<[Id;3]>;                      // sorted, deduped
    pub fn iter(&self) -> impl Iterator<Item=[Id;3]> + '_;
    pub fn full_rebuilds(&self) -> usize;                      // TBox edits => full rematerialize
}
pub struct MaterializedOwlGraph;  // OWL 2 RL; insert/delete take &mut Dict (fallback path interns)
impl MaterializedOwlGraph { pub fn new(dict: &mut Dict, base: &[[Id;3]]) -> Self;
    pub fn insert(&mut self, dict: &mut Dict, triples: &[[Id;3]]) -> usize;
    pub fn delete(&mut self, dict: &mut Dict, triples: &[[Id;3]]) -> usize;
    pub fn mode(&self) -> OwlMode; /* + contains/closure/len/full_rebuilds */ }
pub enum OwlMode { CountingMono, CountingFixpoint, Fallback }

pub struct MaterializedN3Graph; // user N3 rules; operates on n3::Term facts
impl MaterializedN3Graph { pub fn new(rules_src: &str, base_facts: &[[Term;3]]) -> Result<Self, String>;
    pub fn insert(&mut self, facts: &[[Term;3]]) -> usize;     // ground facts only
    pub fn delete(&mut self, facts: &[[Term;3]]) -> usize;
    pub fn mode(&self) -> N3Mode;          // Counting (incremental) | Fallback (re-runs engine)
    pub fn fallback_reason(&self) -> Option<&str>;  // None  <=>  counting path active
    pub fn closure(&self) -> Vec<[Term;3]>; /* + contains/len/full_rebuilds */ }
pub enum N3Mode { Counting, Fallback }

// `explain` feature only (see Gotchas): one derivation witness as a flat DAG.
pub fn why(&self, dict: &Dict, t: [Id;3]) -> Option<ProofTree>;          // RDFS / OWL graphs
pub fn why(&self, fact: &[Term;3])        -> Option<ProofTree>;          // N3 graph
pub struct ProofTree;  // .nodes() -> &[ProofNode], .root(), .conclusion(), .to_json(), .to_text()
pub struct ProofNode { pub conclusion: [String;3], pub rule: String, pub premises: Vec<u32> }
pub struct ExplainOpts { pub max_depth: usize, pub max_nodes: usize } // why_with(.., opts)
```

`Id` / `Dict` come from `sparq_core::dict`; `Term`, `Rule`, `N3Closure`, `ProofStep`, `Resolver` from `sparq_reason::n3` (re-exported at the crate root).

## RIF-Core rule front-end (opt-in `rif-core` feature, `sparq_reason::rif`)

The W3C RIF **Core** dialect — the **monotone Horn-rule** common subset of RIF-BLD/PRD — as a `rif::Document` rule front-end *over the same N3 forward chainer* (a faithful in-engine model that `validate`s for safety then lowers to N3; the proven `reason_n3` fixpoint computes the closure). Build with `features = ["rif-core"]`; **OFF by default**, so the lean/wasm build links zero RIF code (no new `Profile` variant — no match-arm churn anywhere).

```rust
# #[cfg(feature = "rif-core")] {
use sparq_reason::rif::{Atom, Builtin, Document, Rule, Term};
use sparq_core::dict::Dict;
// uncle(?n,?u) :- parent(?n,?p) , brother(?p,?u)   — the canonical RIF Core rule (W3C rif01).
let mut doc = Document::new();
doc.push(Rule::fact(Atom::Frame { obj: Term::Iri("ex:Emeka".into()),     pred: Term::Iri("ex:parent".into()),  val: Term::Iri("ex:Oke".into()) }));
doc.push(Rule::fact(Atom::Frame { obj: Term::Iri("ex:Oke".into()),       pred: Term::Iri("ex:brother".into()), val: Term::Iri("ex:Chi".into()) }));
doc.push(Rule::implies(
    vec![Atom::Frame { obj: Term::Var("n".into()), pred: Term::Iri("ex:uncle".into()), val: Term::Var("u".into()) }],
    vec![Atom::Frame { obj: Term::Var("n".into()), pred: Term::Iri("ex:parent".into()),  val: Term::Var("p".into()) },
         Atom::Frame { obj: Term::Var("p".into()), pred: Term::Iri("ex:brother".into()), val: Term::Var("u".into()) }],
));
let mut dict = Dict::new();
let _entailed: Vec<[sparq_core::dict::Id;3]> = doc.closure(&mut dict)?;  // monotone forward-chaining closure
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

- **Atoms.** `Frame { obj, pred, val }` (`o[p->v]` → triple), `Member { obj, class }` (`o # c` → `rdf:type`), `Subclass { sub, sup }` (`c1 ## c2` → `rdfs:subClassOf`), `Equal { left, right }` (`a = b` → `owl:sameAs`), and `Builtin { op, args }` (body-only).
- **Builtins (`rif::Builtin`).** Numeric predicates (`NumericEqual`/`LessThan`/`GreaterThan`/`NotLessThan`/`NotGreaterThan`) + functions (`NumericAdd`/`Subtract`/`Multiply`/`Divide`), string predicates (`StringContains`/`StartsWith`/`EndsWith`) + functions (`StringConcat`/`StringLength`), and list `ListContains`/`ListLength`. `is_filter()` distinguishes a predicate (all args inputs) from a function (last arg = computed output). Each lowers to the equivalent `math:`/`string:`/`list:` N3 builtin.
- **Builtin SAFETY / range-restriction (enforced).** `Document::validate()` (also called by `to_n3_source`/`closure`) **rejects** an unsafe rule with a `RifError` rather than letting the chainer loop or over-derive: a head variable not bound by a positive body atom (`UnboundHeadVar`), a builtin *input* not range-restricted (`UnboundBuiltinInput`), a builtin in a head (`BuiltinInHead`), or wrong arity (`BadBuiltinArity`).
- **MONOTONE — NAF is EXCLUDED by design.** RIF-Core is monotone Horn; negation-as-failure / RIF-PRD actions / disjunction / aggregation are **not in the dialect** and are not representable in the `Atom` model. Adding facts only ever *adds* conclusions. Larger-RIF surface (RIF-BLD function symbols, the SPARQL-RIF Core Entailment Regime, the RIF/XML importer) is documented out-of-scope in `rif::UNIMPLEMENTED` — tracked, never faked. The expressivity ratchet is `sparq-conformance`'s `rif_core_suite` (opt-in `rif-core` feature; `RIF_CORE_FLOOR`, a sparq-EXTENSION row in the central scoreboard).

## OWL 2 EL classification (`sparq-reason-el`, separate opt-in crate)

OWL 2 RL is **sound but silently incomplete for class classification**: it has no rule that reasons *through* an existential successor, so `--reason owl` over an EL ontology (GO/ChEBI/SNOMED-style) returns a `rdfs:subClassOf` hierarchy that **silently omits** subsumptions like `A ⊑ D` from `A ⊑ ∃r.B`, `B ⊑ C`, `∃r.C ⊑ D` (Krötzsch, ISWC 2012). **`sparq-reason-el`** closes that gap — a consequence-based classifier that normalizes the TBox (Baader–Brandt–Lutz forms) and saturates `S(C)`/`R(r)` under completion rules **CR1–CR5** to compute the **complete** subsumption lattice, then emits it into the **same** `(Dict, Vec<[Id;3]>)` seam as the RL `scm-*` rules (queryable by plain BGP eval).

```rust,ignore
// Cargo.toml:  sparq-reason-el = "0.1"     // a SEPARATE crate; depending on it is the opt-in
use sparq_core::Graph;
use sparq_reason_el::{classify_graph, Classifier};

// Materialize the complete subsumption lattice in place, then query as usual.
let (mut dict, mut triples) = Graph::parse_to_triples(ttl, "turtle")?;
let report = classify_graph(&mut dict, &mut triples);   // adds derived rdfs:subClassOf edges
let g = Graph::from_parts(dict, triples);

// Or a typed, non-mutating view (super-classes / subsumption test / unsatisfiable classes):
let (dict, triples) = Graph::parse_to_triples(ttl, "turtle")?;
let h = Classifier::classify(&dict, &triples);
let _ = h.super_classes(some_class_id);
let _ = h.unsatisfiable_classes();      // classes forced ⊑ owl:Nothing (e.g. via disjointWith)
```

- **Scope (default, Phase E1):** `EL+⊥` minus RBox — `rdfs:subClassOf`/`owl:equivalentClass`, `owl:intersectionOf`, `owl:someValuesFrom` restrictions, `owl:disjointWith`, `owl:Thing`/`owl:Nothing`. Class axioms outside that fragment are **not applied** and counted in `Report::skipped_axioms` (honest, never silently misapplied). Single-threaded.
- **RBox role reasoning (opt-in `rbox` feature, Phase E2, bead `sq-xetf7`):** add `features = ["rbox"]`. Applies `rdfs:subPropertyOf` role inclusions (**CR10**) and `owl:propertyChainAxiom` + `owl:TransitiveProperty` compositions (**CR11**, incl. the SNOMED-critical right-identity `r ∘ s ⊑ s`) via a saturated role automaton, so links propagate up the role hierarchy and along chains before CR4/CR5 fire. **OFF by default** — zero role-automaton code in the default/wasm build; without it RBox axioms are left unapplied (roles compared for equality only). Same `Classifier`/`classify_graph` API; no signature change.
- **Transitive reduction → Hasse diagram (opt-in `hasse` feature, Phase E3, bead `sq-s2nob`):** add `features = ["hasse"]`. `DirectHierarchy::from_closure(&h)` reduces the *full* closure to **direct (immediate) subsumers** and collapses **equivalence cliques** (`direct_super_classes` / `representative` / `equivalent_classes`); `classify_hasse_graph(&mut dict, &mut triples)` materializes the COMPACT taxonomy — direct `rdfs:subClassOf` + `owl:equivalentClass` edges, **O(N)** on a deep chain instead of the O(N²) full closure `classify_graph` emits. The closure of the direct edges (chased through cliques) re-derives the complete relation, so it loses nothing. **OFF by default** — zero reduction code without it; the full-closure `Classifier`/`classify_graph` API is unchanged. Deterministic (rep = min dict id, sorted output) so the Hasse **edge count** is a hard assertion target; timings advisory.
- **Deferred EL fragment (honest incompleteness, surfaced — NOT silently wrong):** OWL 2 EL itself admits two more capability families this classifier does **not** yet apply, so every occurrence is counted in `Report::skipped_axioms` (never mistaken for an opaque class):
  - **Safe nominals — completion rule CR6:** `owl:oneOf` (`{a, b, …}`), `owl:hasValue` (`∃r.{a}`).
  - **Concrete domains — CR7–CR9:** datatype restrictions / faceted ranges — `owl:onDataRange`, `owl:withRestrictions`, `owl:onDatatype`, `owl:datatypeComplementOf`.

  These are the deliberately-deferred slice (spike §"Hard parts"; reasoner-suite design §2.2 recommends keeping them in `skipped_axioms` for now over adding CR6–CR9). Distinct from constructs **outside EL entirely** (unionOf / complementOf / allValuesFrom / cardinality / hasSelf — also skipped, but those need ALC / Horn-SHIQ, not a deferred EL slice) and from RBox (a *gated* capability via `rbox`, not permanently deferred). Concurrent lock-free saturation is E4. `classify_graph` (full closure) and `classify_hasse_graph` (reduced) are both available — pick by whether you want every derived subsumption or just the immediate-parent taxonomy.
- **End-to-end scaling check.** `cargo run -p sparq-reason-el --features rbox,hasse --example snomed_go_scale_bench --release [SCALE]` runs a SNOMED/GO-shaped slice (is-a forest + transitive part-of + SNOMED right-identity role chain + existential restrictions) at 1×/2× and asserts a **relative** (dimensionless) property: closed-form derived counts hold at both scales (conformance) AND the work proxy doubles at most ~2× — confirming normalise + RBox + Hasse compose with **no hidden quadratic**. No hard-coded ms (work-box timings are non-canonical); `tests/snomed_go_scale.rs` is the CI-gated counterpart (runs under the `rbox`/`hasse` legs).
- **Use EL, not `--reason owl`, when you need a complete class hierarchy over an EL ontology.** RL is not an approximation you can tune up with more rules — EL needs a different algorithm.

## OWL 2 QL query rewriting (`sparq-reason-ql`, EXPERIMENTAL, separate opt-in crate)

OWL 2 QL (DL-Lite_R) is **FO-rewritable**: instead of materializing a closure, you **rewrite the query** into a **union of conjunctive queries** (UCQ) that, evaluated over the **unmodified data**, returns the **certain answers** under the schema (Calvanese et al., *PerfectRef*, JAR 2007). **`sparq-reason-ql`** is a query-rewriter (not a materializer): it reuses the engine's query path — it emits a rewritten `spargebra::Query` (a `Union`-folded UCQ) that the planner/executor run unchanged.

```rust,ignore
// Cargo.toml:  sparq-reason-ql = { version = "0.1", features = ["experimental"] }
use sparq_reason_ql::{rewrite, rewrite_production, as_conjunctive_query, CqError};
use spargebra::SparqlParser;

let q = SparqlParser::new().parse_query("SELECT ?x WHERE { ?x a <http://ex/Employee> }")?;
// `tbox`: &[oxrdf::Triple] carrying rdfs:subClassOf/subPropertyOf/domain/range, owl:inverseOf …
let r = rewrite(&q, &tbox)?;            // baseline PerfectRef UCQ; r.report.disjuncts / .skipped_axioms
// Production path: PerfectRef + tree-witness folding + UCQ-containment MINIMISATION (smaller UCQ,
// SAME certain answers). r.report.disjuncts_before_minimisation - r.report.disjuncts = dropped.
let p = rewrite_production(&q, &tbox)?;
// The CQ-shape gate alone (no feature needed) classifies a query without rewriting:
match as_conjunctive_query(&q) { Ok(_cq) => {}, Err(CqError::OutOfScope(why)) => { let _ = why; } }
```

- **FAIL-CLOSED CQ-shape gate (the soundness keystone).** PerfectRef is sound + complete only for **conjunctive queries**. A query with `OPTIONAL`/`FILTER`/`MINUS`/`UNION`/a property path/aggregation/a variable predicate is **rejected as `CqError::OutOfScope(reason)`**, never silently mis-answered. The applicability condition (an existential generator fires only on an UNBOUND, non-distinguished, non-shared variable) is enforced explicitly — firing it on a projected/shared variable would drop a join (unsound). The `reduce` MGU likewise treats **distinguished (answer) variables as rigid** — it never identifies two answer columns (that would answer a different, more-constrained query and drop answers).
- **Scope (`experimental`):** the **positive** DL-Lite_R inclusions — `rdfs:subClassOf`/`subPropertyOf`, `rdfs:domain`/`range` (`∃R ⊑ A`, `∃R⁻ ⊑ A`), `owl:inverseOf`, and **unqualified** `∃R` `owl:someValuesFrom owl:Thing` restrictions. Non-QL axioms are counted in `RewriteReport::skipped_axioms`, never applied.
- **Production path (`rewrite_production`):** baseline PerfectRef **augmented** with bounded **tree-witness** folding (existential witnesses captured with no unbounded chase) then **UCQ-containment minimisation** (drop disjuncts contained in a retained one). Same certain answers as `rewrite`, in a smaller UCQ. **Minimisation is FAIL-CLOSED:** containment is NP-complete, the homomorphism search is bounded, and an **undecided-within-budget** check **KEEPS** the disjunct — minimisation only ever removes a disjunct **proven contained**, so it removes no answers.
- **Oracle-tested; the FORMAL DL-Lite_R suite GRADUATED to a pinned floor (sq-qo1a9).** Validated against a hand-checked DL-Lite_R oracle (`sparq-reason-ql/tests/oracle.rs`, incl. tree-witness + minimisation cases), because no Rust PerfectRef reference exists to diff against. There is **no consistency checking**. On the **formal DL-Lite_R suite** — the hand-derived certain-answer oracle from `sq-g19x0`, every case a conjunctive query within sound rewriting — the rewrite is **sound AND complete case by case**: `rewrite_production`'s UCQ, evaluated over the **unmodified ABox** through the real engine, returns **exactly** the hand-derived certain answers. That is now a **pinned floor** (`sparq-conformance`'s `tests/ql_dllite_suite.rs`, opt-in `ql-experimental`; `QL_DLLITE_FLOOR = 11` sound-and-complete cases), registered as a **`sparq extension`** row in the central scoreboard (`scoreboard::SUITES`) and tallied **separately** — **NOT folded into the standards-conformance total**, and **NOT a full-OWL-2-QL-conformance claim** (there is no runnable normative W3C QL certain-answer suite; the W3C QL material is structural). Like the RIF-Core / RSP / BM25 extension rows, it pins a faithful sparq-OWN oracle. `QL_DLLITE_FLOOR` is read textually by `tests/scoreboard_floors.rs`, so the mirrored scoreboard value cannot drift.
- **The BROADER `pr:QL` `sparql11/entailment` arm stays EXPERIMENTAL / OutOfScope (sq-kuvu3, opt-in `sparq-conformance/ql-experimental`).** That set is **not** the formal DL-Lite_R suite: it mixes intensional / non-DL-Lite certain-answer cases the sound rewriting fragment cannot answer. The rewriter runs over every `sd:EntailmentProfile pr:QL` case — each conjunctive query rewritten by `rewrite_production` and evaluated over the **unmodified data**, then compared to the suite oracle — and the harness reports **HONESTLY as experimental / OutOfScope, NEVER a graduated conformance pass** (NO floor `const` for this arm, NO row summed into any total). Outcomes (all OutOfScope): an **ABSTAIN** when the fail-closed CQ-shape gate rejects a non-conjunctive / non-DL-Lite query (never a guess), a **computed result-equivalent** evidence row when the rewritten UCQ genuinely matches the oracle, a **computed-DIVERGENT** row when it does not (an honest gap — e.g. an intensional `?c rdfs:subClassOf …` TBox query a certain-ABox-answer rewriter cannot answer), or an **inconclusive** setup-failure row. The runner is `sparq-conformance`'s `tests/ql_experimental_arm.rs` (asserts the honesty invariants — every row OutOfScope, at least one fail-closed abstain — NOT a pass-count floor); the inference binary prints the experimental QL section only with the feature on.

## Common recipes

**1. OWL 2 RL closure + inconsistency report.** OWL includes RDFS; run the clash check on the materialized result.

```rust
use sparq_core::Graph;
use sparq_reason::{materialize, inconsistencies, Profile};

let (mut dict, mut triples) = Graph::parse_to_triples(owl_text, "turtle")?;
materialize(Profile::OwlRl, &mut dict, &mut triples);
let clashes = inconsistencies(&dict, &triples);   // e.g. disjointWith / sameAs↔differentFrom
if !clashes.is_empty() { eprintln!("INCONSISTENT: {clashes:?}"); }
let g = Graph::from_parts(dict, triples);
# Ok::<(), String>(())
```

**2. Notation3 rules + facts in one document → entailed ground triples.** The rules and data live in the same N3 source; only ground facts survive the closure.

```rust
use sparq_core::dict::Dict;
use sparq_reason::reason_n3;

let n3 = r#"
@prefix : <http://ex/> .
{ ?x a :Human } => { ?x a :Mortal } .
:socrates a :Human .
"#;
let mut dict = Dict::new();
let closure = reason_n3(&mut dict, n3)?;          // includes  :socrates a :Mortal
for t in &closure { println!("{} {} {} .", dict.term(t[0]), dict.term(t[1]), dict.term(t[2])); }
# Ok::<(), String>(())
```

**3. Incremental RDFS maintenance** (data updates cost time ∝ the change, not a full re-materialize):

```rust
use sparq_reason::MaterializedGraph;
let mut g = MaterializedGraph::new(&mut dict, &base_triples);
g.insert(&[[alice, ty, person]]);   // closure updated incrementally (one sweep over the delta)
g.delete(&[[alice, ty, person]]);   // exact-count retraction; a still-derivable fact survives
assert!(g.contains(&[alice, ty, agent]));   // entailed via subClassOf
let materialized: Vec<[u64; 3]> = g.closure();   // feed back into Graph::from_parts to query
```

> Note: an `insert`/`delete` touching a **TBox** triple (`subClassOf`/`subPropertyOf`/`domain`/`range`, and the OWL schema predicates) triggers a full rematerialize — watch `g.full_rebuilds()`. Keep ABox edits and schema edits separate when you care about incremental cost.

**4. Incremental N3 with a fallback check.** Term-level facts; verify you actually got the fast path.

```rust
use sparq_reason::{MaterializedN3Graph, n3::Term};
let rules = "{ ?x <http://ex/p> ?y } => { ?y <http://ex/q> ?x } .";
let mut g = MaterializedN3Graph::new(rules, &base_facts)?;   // base_facts: &[[Term;3]]
g.insert(&[[Term::Iri("http://ex/a".into()),
            Term::Iri("http://ex/p".into()),
            Term::Iri("http://ex/b".into())]]);
if let Some(why) = g.fallback_reason() { eprintln!("running batch fallback: {why}"); }
let closure: Vec<[Term;3]> = g.closure();
# Ok::<(), String>(())
```

**5. Derivation proof (`explain` feature).** Returns ONE witness derivation of a triple from the asserted base; render as text or JSON.

```rust
// Cargo.toml:  sparq-reason = { version = "0.1", features = ["explain"] }
use sparq_reason::MaterializedGraph;
let g = MaterializedGraph::new(&mut dict, &base);
if let Some(tree) = g.why(&dict, [alice, ty, agent]) {
    println!("{}", tree.to_text());   // indented, root first; rule ids like cax-sco / rdfs9 / prp-trp
    let json = tree.to_json();        // {"root":R,"nodes":[{"id":..,"conclusion":[s,p,o],"rule":..,"premises":[..]}]}
}
```

**6. `log:semantics` / `log:content` document access.** The engine does NO I/O of its own; supply a `Resolver` closure to decide what an IRI may dereference to (otherwise those builtins simply don't fire):

```rust
use sparq_reason::reason_n3_terms_with_resolver;
let resolver = |iri: &str| std::fs::read_to_string(iri.trim_start_matches("file://")).ok();
let closure = reason_n3_terms_with_resolver(src, Some("http://ex/"), Some(&resolver))?;
# Ok::<(), String>(())
```

**Import cycles always terminate (with a LIVE resolver).** N3 is Turing-complete, so a `log:semantics` document whose closure re-imports a document active up the resolution stack — directly (`A→A`), indirectly (`A→B→A`), or via a re-used node in a diamond — would otherwise spin forever once a real (filesystem/network) resolver is wired (the offline conformance harness never hit it). The engine tracks the formulae whose closure is in progress; re-entering one already in progress returns it **unclosed** (cwm's "a document already being loaded is not re-loaded") instead of recursing, so reasoning terminates. A diamond that re-uses a shared document across **sibling** branches is not a cycle and still resolves on every branch — only the pathological cyclic case changes; valid acyclic imports are byte-identical to before.

## Gotchas / feature flags / prerequisites

- **Not in the *lean* wasm bundle, but wasm-portable.** `sparq-reason` pulls `regex` and (by default) `rayon`; it is never in the **lean** `sparq-wasm` triplestore bundle. For wasm or single-threaded builds use `default-features = false` (disables the `parallel`/rayon feature). The crate itself compiles to `wasm32-unknown-unknown` — `regex` (the N3 `string:matches` builtin) is pure-Rust and wasm-portable — and ships as the **tier-b `sparq-reason-wasm` ("W-reason") bundle** ([OPUS-4.8] sq-6qw3): a `Reasoner` exposing `materialize` / `entailed` / `materializeStats` / `reasonN3` (and, behind the bundle's opt-in `explain` feature, `why()` proof trees) for in-tab live inference, lazy-loaded on the showcase site's `/surface/inference` page. There is no Noir/ZK toolchain requirement here — proofs are plain Rust structs.
- **Features:** `parallel` (default, rayon-parallel fixpoint), `explain` (NON-default — enables `why()`/`why_with()` and the `explain` module; zero hot-path cost when off, and `why` methods don't exist without it), `d-entail` (NON-default — enables `Profile::D` + the `dtype` module; zero code when off — the lean default/wasm build is byte-identical, `sq-e5atd`), `rif-core` (NON-default — enables the `rif` module: the RIF-Core monotone-Horn rule front-end over the N3 chainer with range-restriction safety; zero code when off, no new `Profile` variant, `sq-rh4gu`), `substrate-join` (NON-default — the RDFS predicate join drives the SHARED `sparq-substrate::join` kernels; `sq-yk6or`, see next bullet).
- **Shared join kernels (`substrate-join`, opt-in, `sq-yk6or`, epic `sq-pbz04`).** The RDFS single-pass predicate join — rdfs7 (subPropertyOf rewrite), rdfs2 (domain typing), rdfs3 (range typing) — keyed on the asserted triple's predicate, drives the *same* `sparq_substrate::join::{build_table, probe_emit, hash_probe_serial}` hash-join body the SPARQL engine drives (epic `sq-qonbz` Phase 3, #1300). The reasoner supplies its OWN `JoinKeys` (predicate-keyed) + its OWN `Budget` (the unbounded `NoBudget`; materialisation runs to completion — a closure-level budget is a fixpoint concern, installed around the whole call, not per-join), monomorphically — no `Box<dyn>`/vtable on the probe loop. This is the end-to-end proof of "share join logic across the engine AND the reasoners" (`research/shared-eval-substrate.md` Phase 5). **Behaviour-neutral:** the materialised closure is byte-identical to the hand-rolled `FxHashMap` adjacency path (asserted by `rdfs::tests::substrate_join_emits_identical_plain_branch`); only the join machinery changes. **OFF by default** so the byte/bundle ratchets stay exactly the hand-rolled path; the only deps it pulls (`sparq-substrate` `rows`+`join`, `smallvec`) are already in the crate's tree. **Out of scope (kept on the hand-rolled path, documented in `substrate_join.rs`):** the `rdf:type`/rdfs9 subclass branch + the `PropExpand` inverseOf/Symmetric predicate-rewrite branch (non-uniform combine — orientation swap / type-object key), and the OWL-RL semi-naive `Δ⋈full` fixpoint with union-find `sameAs` (a different, incremental/mutating join shape than the substrate's static `&[Row]` kernel). Folding those in would not change *what* is emitted, only add reshape cost — tracked as a follow-up, not shipped here.
- **Two value levels.** RDFS/OWL APIs work on dictionary `Id`s (`materialize*`, `Materialized(Owl)Graph`); N3 batch APIs intern into a `Dict` (`reason_n3`), while term-level N3 (`reason_n3_terms`, `MaterializedN3Graph`) works on `n3::Term` and is **not interned** (formula `{ … }` terms have no dictionary id). Don't mix the two.
- **The materialize → from_parts seam.** `materialize` mutates `(Dict, Vec<[Id;3]>)` *before* indexes are built. Use `Graph::parse_to_triples` (not `Graph::load_str`) so reasoning runs between parse and index build; then `Graph::from_parts`. It interns any vocabulary terms it needs and is idempotent (a second call adds nothing).
- **RDFS scope is deliberate:** the non-explosive subset (rdfs2,3,5,7,9,11 — subClass/subProperty/domain/range). No axiomatic or reflexive `rdfs:subClassOf`/`type` triples (they add no useful inferences and explode the store).
- **D-entailment (`Profile::D`, opt-in `d-entail`) scope + caveats:** materializes the rdfD1 datatype-typing rule — a well-formed literal `"l"^^d` of a *recognized* datatype `d` (the `Recognized` map; `xsd:string`/`rdf:langString` always, `Recognized::standard()` adds the numeric/boolean/temporal core) entails `"l"^^d rdf:type d`. The emitted typing triples are **generalized** (literal in subject position) — feed the closure to a query only after dropping literal-subject rows (they can never be a SPARQL answer; this is also why the W3C `d-ent-01` test correctly returns NO rows). The load-bearing invariant is **value-space equality** via `d_value_eq`: `"1"^^xsd:integer` ≡ `"1.0"^^xsd:decimal` (the integer/decimal value spaces coincide), compared as a CANONICAL DECIMAL STRING — **never an f64 fast path** (f64 silently aliases integers past 2^53 and loses decimal precision). `float`/`double` are a DISJOINT IEEE-754 value space; `date` and `dateTime` are disjoint temporal families. NOTE: the SHARED SPARQL term total order now lives in `sparq-substrate::compare` (`compare_terms` over the generic `CompareTerm` trait — error/unbound < blank < IRI < literal < triple, numeric-aware + strict typed/temporal + string fallback; epic `sq-qonbz` Phase 4, `sq-vezew`, #1300-chain). A reasoner that orders entailed solutions (RIF `order`, an EL/QL `ORDER BY` over a materialised answer set) can reuse it by implementing `CompareTerm` for its own term type — the same monomorphisation seam `substrate-join` uses for `JoinKeys`. The trait carries an `exact_cmp` **f64-collapse recheck** hook (`sq-rikm7`): the numeric arm coerces to f64 for speed, and when two operands tie there `exact_cmp` recovers the exact order of distinct integers past 2^53 / high-precision decimals — so a reasoner `ORDER BY` / `MIN` / `MAX` agrees with the relational `=`/`<` rather than falling into the very f64-aliasing this caveat warns about (return `None` from it if your term type has no exact numeric tier). D's typed *value-space-equality* comparator (`d_value_eq`, used for entailment not ordering) stays reasoner-resident for now; D-inconsistency (ill-typed-literal / value-space clashes) and cross-type value-space *subset* reasoning are tracked-not-yet-shipped here (epic `sq-pbz04`).
- **OWL 2 RL is sound but INCOMPLETE for class classification.** Running `Profile::OwlRl` / `--reason owl` over an EL ontology returns a `rdfs:subClassOf` hierarchy that silently omits existential-reasoning subsumptions (the calculus has no rule reasoning through an `∃r` successor). For the **complete** class hierarchy use `sparq-reason-el` (above), not more RL rules.
- **The RL materializer is COMPLETE for the assertion-style RL/RDF rules — the W3C OWL-RL conformance row is at the RL ceiling (sq-350ms).** Every rule with a positive-assertion head in Profiles §4.3 Tables 5/6/9 is implemented (the `owl.rs` per-rule status table + `research/inference-completeness-audit.md` §2/§2b are the per-rule proof). The 13 documented OWL-RL conformance divergences are PROVABLY outside the RL profile, **not** missing rules: TBox-axiom conclusions, invented class expressions (`owl:complementOf`/`unionOf`), reified `owl:AllDifferent` structures, the `prp-pdw`/`prp-fp`/`prp-ifp` **contrapositives** (RL has NO rule producing `owl:differentFrom` — it appears only in clash bodies), `owl:ReflexiveObjectProperty` (EXCLUDED from the RL grammar — there is no `prp-rfx`), and datatype-range INTERSECTION. They stay documented divergences (closing them would be unsound or beyond-profile); the inference ratchet HOLDS at 1967. Multi-round assertion-rule completeness and the prp-pdw/prp-fp soundness boundary are pinned by in-crate guards in `owl.rs::tests`.
- **OWL incremental fallback is silent.** `MaterializedOwlGraph` drops to `OwlMode::Fallback` (re-materializes via `materialize_owl_rl` every mutation, still correct) when the base uses `owl:sameAs`, Functional/InverseFunctional, property chains, restrictions, cardinality, hasKey, oneOf, intersection/union — and on any TBox mutation. Check `.mode()` / `.full_rebuilds()` if incremental cost matters. These usually live in a static TBox, so the mode is decided once at load.
- **N3 incremental qualification is narrow.** `MaterializedN3Graph` only runs `N3Mode::Counting` (truly incremental) for a monotone, input-stratified rule fragment: forward rules with ground-IRI predicates, no conclusion blank nodes, builtins limited to the parity whitelist (`log:uri`, `log:equalTo`/`notEqualTo`, `string:concatenation`/`scrape`/`encodeForUri`), and negation only via the store-scoped `?x log:notIncludes { … }` idiom over input-only predicates. Anything else → `N3Mode::Fallback`; always consult `.fallback_reason()` (`None` ⇔ counting active). The full *batch* N3 engine (`reason_n3`) supports the much larger `math:`/`string:`/`list:`/`time:`/`log:` builtin set and goal-directed `<=` rules.
- **`why()` is a witness, not a proof set.** It returns the first derivation in deterministic order, or `None` if the triple isn't in the closure or `ExplainOpts` caps (default depth 128, 65 536 nodes) are exceeded — not an enumeration of all derivations.
- **Deletion semantics:** `delete` removes *base* (asserted) triples; a deleted base triple still derivable from the remainder stays in the closure, and deleting a derived-only fact is a no-op (standard materialized-view semantics).

## See also

- `noir-circuit-patterns`, `noir-optimisation`, `verifiable-credentials-zk`, `sparql-formal-semantics` — the single-prover ZK estate; the `explain` `ProofTree` is intentionally a flat, id-free, premises-before-conclusion DAG meant as a ZK-derivation witness.
- `mpc-protocols` — multi-party layer over (federated) SPARQL.
- `hdt-format`, `fused-decompress-parse`, `rust-parallel-parsing` — sibling ingest/storage skills for getting triples into the graph you then reason over.
- `research/owl2-el-ql-reasoning-spike.md` — the EL/QL feasibility spike: why EL first, the RL-incompleteness proof (the CR4 counterexample), and the phased plan (E1–E6) `sparq-reason-el` implements.
- `research/reasoner-suite-on-substrate.md` §2.5 — the QL track design: the PerfectRef applicability trap, the strict CQ-shape gate, and why the production path (tree-witness + UCQ-containment minimisation) is sequenced late by soundness risk (the phased plan `sparq-reason-ql` implements through phases Q1–Q3; only the conformance-floor graduation remains deferred).
