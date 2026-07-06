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

```rust
// `compiled-rules` feature only (`sparq_reason::n3::compiled`, sq-zgbso.3): id-level
// COMPILED N3 evaluation for the access-control subset — no per-call text round-trip
// (the rule IR pre-interns its constants into the caller's Dict; the fixpoint joins run
// on the shared sparq-substrate kernels over [Id;3] facts).
pub fn compile(src: &str) -> Result<CompiledRuleSet, String>;   // parse + lower ONCE
pub fn eval(dict: &mut Dict, facts: &[[Id;3]], rules: &CompiledRuleSet) -> Vec<[Id;3]>;
pub fn intern_facts(dict: &mut Dict, src: &str) -> Result<Vec<[Id;3]>, String>; // test/harness fact loader
impl CompiledRuleSet { pub fn bind(&self, dict: &mut Dict) -> BoundRuleSet<'_>; // intern the symbol table
                       pub fn n_rules(&self) -> usize; pub fn n_facts(&self) -> usize; }
impl BoundRuleSet<'_> { pub fn eval(&self, dict: &mut Dict, facts: &[[Id;3]]) -> Vec<[Id;3]>; }
```

Compiled-rules scope is EXACTLY the WAC/ACP/ODRL-spike corpus subset (scoped `log:notIncludes` over stratum-complete predicates, `log:uri`, `log:(not)equalTo`, `string:` concatenation / encodeForUri / scrape / notGreaterThan); anything else is a loud `compile` error — full N3 stays with `reason_n3`. Closure set-equality vs `reason_n3` is pinned by `crates/sparq-reason/tests/compiled_equivalence.rs`.

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

- **Atoms.** `Frame { obj, pred, val }` (`o[p->v]` → triple), `Member { obj, class }` (`o # c` → `rdf:type`), `Subclass { sub, sup }` (`c1 ## c2` → `rdfs:subClassOf`), `Equal { left, right }` (body-only; see Equal-atom semantics below), and `Builtin { op, args }` (body-only).
- **Equal-atom semantics (sq-pbz04.5.4 + sq-26vwp — RIF-Core fidelity; body Equal resolved at compile time, NEVER as an `owl:sameAs` triple):**
  1. **Equal in a rule head is REJECTED** — `validate()` returns `RifError::EqualInConclusion`. This is the RIF-Core syntactic restriction (unlike RIF-BLD, Core does not permit equality in conclusions).
  2. **`t = t` (identical after substitution) is eliminated** — `Equal { left: Term::Iri("x"), right: Term::Iri("x") }` (or `?n=?n`) is trivially true and dropped. It contributes NO bindings for range-restriction — if a head variable is SOLELY bound by a `?x=?x` atom, `validate()` rejects the rule with `UnboundHeadVar` (unifying a variable with itself binds nothing).
  3. **`?x = t` (one side a variable) is SUBSTITUTED** — `t` replaces `?x` throughout head+body at validate/lower time, so `?x` becomes **bound-by-substitution** (`?x # C :- ?x = <a>` collapses to the fact `<a> # C`). **`?x = ?y` (two distinct variables) is UNIFIED** — one name is renamed to the other everywhere, so the join requires the SAME node: same-node reflexivity fires *without* any `owl:sameAs` assertion (fixes V2), and an asserted `owl:sameAs` between DISTINCT nodes never over-derives the equality (fixes V1). Substitution runs to a fixpoint, so chained `?x=?y, ?y=t` collapse both to `t`.
  4. **Distinct GROUND constants are REJECTED fail-closed** — `validate()` returns `RifError::DistinctGroundEqual { left, right }` (including a distinct ground *created* by substitution, e.g. `?x=<a>, ?y=<b>, ?x=?y`). Value-space equality (e.g. `"1"^^xsd:integer = "1.0"^^xsd:decimal`) depends on the sq-v5evr value-space comparator (issue #1646, not yet merged); the front-end refuses rather than answering incorrectly. No body `Equal` atom ever reaches N3 lowering — a stray one fails closed, never emits `owl:sameAs`.
- **Builtins (`rif::Builtin`).** Numeric predicates (`NumericEqual`/`LessThan`/`GreaterThan`/`NotLessThan`/`NotGreaterThan`/`NumericNotEqual`) + functions (`NumericAdd`/`Subtract`/`Multiply`/`Divide`), string predicates (`StringContains`/`StartsWith`/`EndsWith`) + functions (`StringConcat`/`StringLength`/`StringUpperCase`/`StringLowerCase`/`StringEncodeForUri`), and list `ListContains`/`ListLength`/`ListConcatenate` (variadic — `is_variadic()` returns `true`; `arity()` is the minimum). `is_filter()` distinguishes a predicate (all args inputs) from a function (last arg = computed output). Each lowers to the equivalent `math:`/`string:`/`list:` N3 builtin. A **deferral ledger** (`rif::UNIMPLEMENTED`) records builtins that are NOT mapped because no sound N3 target exists today (e.g. `func:numeric-integer-divide` truncation semantics, `pred:matches` XSD-regex vs Rust-regex dialect gap, date/time builtins lacking a temporal tower); those entries are tracked, never silently dropped.
- **Builtin SAFETY / range-restriction (enforced).** `Document::validate()` (also called by `to_n3_source`/`closure`) **rejects** an unsafe rule with a `RifError` rather than letting the chainer loop or over-derive: a head variable not bound by a positive body atom (`UnboundHeadVar`), a builtin *input* not range-restricted (`UnboundBuiltinInput`), a builtin in a head (`BuiltinInHead`), wrong arity (`BadBuiltinArity`), Equal in a head (`EqualInConclusion`), or distinct ground constants in a body Equal (`DistinctGroundEqual`).
- **MONOTONE — NAF is EXCLUDED by design.** RIF-Core is monotone Horn; negation-as-failure / RIF-PRD actions / aggregation are **not in the dialect** and are not representable in the `Atom` model. Adding facts only ever *adds* conclusions. Larger-RIF surface (RIF-BLD function symbols, the SPARQL-RIF Core Entailment Regime) is documented out-of-scope in `rif::UNIMPLEMENTED` — tracked, never faked. The expressivity ratchet is `sparq-conformance`'s `rif_core_suite` (opt-in `rif-core` feature; `RIF_CORE_FLOOR`, a sparq-EXTENSION row in the central scoreboard).
- **RIF/XML importer** (`rif-xml` feature, `rif_xml::import()`): parses the W3C RIF-Core XML
  presentation syntax into a `rif::Document`. Applies two sound desugarings at import: body
  `Or` → rule-splitting (Lloyd-Topor, one rule per disjunct); body `Exists` → existential
  vars become ordinary body vars (range-restriction validated by `Document::validate`). Fail-closed:
  `Import` directives, non-Core elements, unknown `External` IRIs, named-argument uniterms,
  and malformed XML each produce a named `ImportError` variant. Parsing only — no new inference
  beyond the existing `rif-core` forward chainer. Unblocks sq-pbz04.5.5 (W3C RIF WG test-suite arm).

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

- **Scope (default, Phase E1):** `EL+⊥` minus RBox — `rdfs:subClassOf`/`owl:equivalentClass`, `owl:intersectionOf`, `owl:someValuesFrom` restrictions, `owl:disjointWith`, `owl:Thing`/`owl:Nothing` — **plus safe nominals** (bead `sq-pbz04.2.1`): a singleton `owl:oneOf` (`{a}`) and an object-valued `owl:hasValue` (`∃r.{a}`) are basic concepts reasoned over by completion rule **CR6** (the reachability-guarded nominal merge, "Pushing the EL Envelope" IJCAI-05 — the guard is what keeps `C ⊑ {a}, D ⊑ {a} ⊬ C ⊑ D` when `D` may be empty; negative tests in `tests/nominals.rs` pin it). Every CR6 derivation is sound; completeness is claimed for the typical safe usage, NOT for every EL++ nominal interplay (unrestricted nominal interaction needs a stronger calculus — ELK line of work, KR 2012). ABox `rdf:type` assertions are NOT internalized as nominal axioms (TBox classification only). Class axioms outside the recognised fragment are **not applied** and counted in `Report::skipped_axioms` (honest, never silently misapplied). Single-threaded.
- **RBox role reasoning (opt-in `rbox` feature, Phase E2, bead `sq-xetf7`):** add `features = ["rbox"]`. Applies `rdfs:subPropertyOf` role inclusions (**CR10**) and `owl:propertyChainAxiom` + `owl:TransitiveProperty` compositions (**CR11**, incl. the SNOMED-critical right-identity `r ∘ s ⊑ s`) via a saturated role automaton, so links propagate up the role hierarchy and along chains before CR4/CR5 fire. **OFF by default** — zero role-automaton code in the default/wasm build; without it RBox axioms are left unapplied (roles compared for equality only). Same `Classifier`/`classify_graph` API; no signature change.
- **Transitive reduction → Hasse diagram (opt-in `hasse` feature, Phase E3, bead `sq-s2nob`):** add `features = ["hasse"]`. `DirectHierarchy::from_closure(&h)` reduces the *full* closure to **direct (immediate) subsumers** and collapses **equivalence cliques** (`direct_super_classes` / `representative` / `equivalent_classes`); `classify_hasse_graph(&mut dict, &mut triples)` materializes the COMPACT taxonomy — direct `rdfs:subClassOf` + `owl:equivalentClass` edges, **O(N)** on a deep chain instead of the O(N²) full closure `classify_graph` emits. The closure of the direct edges (chased through cliques) re-derives the complete relation, so it loses nothing. **OFF by default** — zero reduction code without it; the full-closure `Classifier`/`classify_graph` API is unchanged. Deterministic (rep = min dict id, sorted output) so the Hasse **edge count** is a hard assertion target; timings advisory.
- **Concrete domains — CR7–CR9 (opt-in `cdomain` feature, bead `sq-pbz04.2.2`):** add `features = ["cdomain"]`. Faceted datatype restrictions — `owl:onDatatype` + `owl:withRestrictions` with `xsd:min/maxInclusive`/`xsd:min/maxExclusive` facets — are decided EXACTLY on the shared `sparq_substrate::numeric` value tower (`Dec` i128 fixed-point, never lossy f64) for `xsd:decimal`, `xsd:integer` and the 12 derived integer types (implicit bounds folded in, so `xsd:byte` + `minInclusive 1000` is genuinely empty; exclusive integer bounds TIGHTEN, so integer `(5, 6)` is empty while decimal `(5, 6)` is not). An **EMPTY** range is `⊑ owl:Nothing` (the clash reaches classes with an `∃p.range` obligation via CR5 → `unsatisfiable_classes()`); a **proven value-space containment** (`[5,10] ⊆ [0,20]`, integer-inside-decimal, point-in-range) threads subsumptions through data-property existentials via the ordinary CR1/CR3/CR4. Exact-numeric `DataHasValue` (`owl:hasValue 5`) and singleton `DataOneOf` (`owl:oneOf (5)`) are point ranges (`{5}`, `{5.0}` and faceted `[5,5]` unify on ONE concept). **Deferred — no verdict is EVER guessed** (stays in `skipped_axioms`): pattern/length/digit facets (an unknown facet defers the WHOLE range — ignoring it could fabricate a containment), float/double or non-numeric bases and bound values, `owl:onDataRange` (cardinality vocabulary, outside EL), `owl:datatypeComplementOf`, ill-formed bounds (`"300"^^xsd:byte`), and mixed range/class-expression nodes. Known sound incompleteness: a decimal-sorted range is not derived ⊆ an integer-sorted one (non-point cases), and a plain facet-free datatype IRI filler keeps its opaque-class treatment. **OFF by default** — zero concrete-domain code and no `sparq-substrate` dep without it; every concrete-domain occurrence is then skipped as before. `tests/cdomain.rs` pins the sat/unsat/deferral matrix with exact-closure oracles.
- **Deferred EL fragment (honest incompleteness, surfaced — NOT silently wrong):** without `cdomain`, ALL concrete-domain shapes (`owl:onDataRange`/`owl:withRestrictions`/`owl:onDatatype`/`owl:datatypeComplementOf` + literal `hasValue`/`oneOf`) land in `Report::skipped_axioms`; with it, the unsupported remainder above still does. Distinct from constructs **outside EL entirely** (unionOf / complementOf / allValuesFrom / cardinality / hasSelf / a **multi-individual** `owl:oneOf` — the profile's `ObjectOneOf` admits exactly one individual, more is a disjunction — all skipped, but those need ALC / Horn-SHIQ, not a deferred EL slice) and from RBox (a *gated* capability via `rbox`, not permanently deferred). Concurrent lock-free saturation is E4. `classify_graph` (full closure) and `classify_hasse_graph` (reduced) are both available — pick by whether you want every derived subsumption or just the immediate-parent taxonomy.
- **End-to-end scaling check.** `cargo run -p sparq-reason-el --features rbox,hasse --example snomed_go_scale_bench --release [SCALE]` runs a SNOMED/GO-shaped slice (is-a forest + transitive part-of + SNOMED right-identity role chain + existential restrictions) at 1×/2× and asserts a **relative** (dimensionless) property: closed-form derived counts hold at both scales (conformance) AND the work proxy doubles at most ~2× — confirming normalise + RBox + Hasse compose with **no hidden quadratic**. No hard-coded ms (work-box timings are non-canonical); `tests/snomed_go_scale.rs` is the CI-gated counterpart (runs under the `rbox`/`hasse` legs).
- **W3C OWL 2 EL suite — a pinned EXTENSION ratchet (bead `sq-pbz04.2.4`/`sq-pbz04.2.9`, opt-in `sparq-conformance/el-suite`).** The W3C OWL WG export (`tests/w3c/owl2/all.rdf`), filtered to `test:EL` ∧ `test:RDF-BASED` (Approved, inline RDF/XML premise, no `owl:imports`), is run through the **REAL** classifier: each premise is classified with `classify_graph` (materializing the complete `rdfs:subClassOf` lattice IN PLACE, with **`rbox`** + **`cdomain`** also on — the CI lane exercises the full shipped feature set) and each declared check decided — **consistency** (no unsatisfiable named class), **inconsistency** (some unsatisfiable named class), **positive-entailment** (the lattice ENTAILS the conclusion via the bnode-homomorphism `entail::entails` after output-vocabulary completions: datatype axiomatic-set + mutual-subsumption → `owl:equivalentClass` augmentation — a semantic identity that graduated WebOnt-equivalentClass-003, sq-pbz04.2.9), **negative-entailment** (non-conclusion NOT entailed). `EL_SUITE_FLOOR` is the **MEASURED PASS count** — a **`sparq extension`** row in the central scoreboard (`scoreboard::SUITES`), tallied **separately** and **NOT** a full-OWL-2-EL-conformance claim: tests needing **ABox inconsistency** (individual assertions), or a conclusion in `owl:sameAs`/`rdfs:subPropertyOf`/`owl:equivalentProperty`/`owl:TransitiveProperty`/`owl:unionOf` axiom form (output-vocabulary gaps distinct from inference gaps) are **audited PERMANENT divergences** (reported separately, **never summed into the floor**). `EL_SUITE_FLOOR` is read textually by `tests/scoreboard_floors.rs`, so the mirrored scoreboard value cannot drift; `--nocapture` prints an `OWL 2 EL ratchet pass N of M (floor F)` line the CI job `inference-conformance` re-greps.
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

- **FAIL-CLOSED CQ-shape gate (the soundness keystone) with broadened sound fragment.** PerfectRef is sound + complete only for **conjunctive queries**. The gate is **fail-closed** and now accepts a BROADENED sound fragment: **(B1)** top-level UCQ (`UNION` of CQ branches, each rewriting independently; `as_ucq`); **(B2)** RDF literal constants in role-atom object position (rigid, never `_`-eligible — the applicability condition is unchanged); **(B3)** `FILTER` over **distinguished-only** variables (passed through, fail-closed for non-distinguished vars); **(B4)** constant-only `VALUES` over distinguished variables (re-applied as an inner join, fail-closed for UNDEF / non-distinguished); **(Bbnode, sq-pbz04.3.6)** body blank nodes in subject/object positions are lifted to fresh existential variables — each distinct blank-node LABEL in a CQ maps to one unique `Unbound` id (shared labels get the same id, so `is_bound_var` correctly treats the shared position as bound and blocks the existential applicability condition on it; distinct labels get distinct ids). The **emitter** completes the round-trip: a repeated `Unbound` id maps to ONE output variable, so a shared body blank node emits as a genuine **JOIN** (not a cartesian product over-approximation) — the load-bearing invariant that lets the shared-existential SELECT cases (`sparqldl-07`/`-08`) graduate rather than fall to `oracle-divergent`. **(B5, sq-pbz04.3.2)** non-recursive property paths — sequence `p1/p2` (fresh non-distinguished intermediate, shared → a JOIN), inverse `^p` (subject/object swap), alternation `p1|p2` (branch multiplication into the B1 UCQ machinery) — are desugared to an equivalent CQ/UCQ BEFORE the entailment rewrite, after which PerfectRef soundness applies (STEP-0-verified: the vendored spargebra parser already lowers a top-level sequence/inverse to a BGP, so only a surviving `Path` alternation is rewritten — no double-translation); `+`/`*`/`?` (recursion / zero-length) and negated property sets stay fail-closed. **(B6)** `rdfs:subClassOf`, `rdfs:subPropertyOf`, `rdfs:domain`, `rdfs:range`, and all `owl:` predicates used as **atom predicates** are **rejected** (intensional-atom guard — fail-closed); annotation predicates (`rdfs:label/comment/seeAlso/isDefinedBy`) are admitted. Everything outside this fragment — `OPTIONAL`/`MINUS`/**recursive-or-negated** property paths (`+`/`*`/`?`/`!p`)/aggregation/variable-predicate/non-distinguished FILTER or VALUES — is **rejected as `CqError::OutOfScope(reason)`**, never silently mis-answered. The applicability condition (an existential generator fires only on an UNBOUND, non-distinguished, non-shared variable) is enforced explicitly. The `reduce` MGU treats **distinguished (answer) variables as rigid** — it never identifies two answer columns.
- **Scope (`experimental`):** the **positive** DL-Lite_R inclusions — `rdfs:subClassOf`/`subPropertyOf`, `rdfs:domain`/`range` (`∃R ⊑ A`, `∃R⁻ ⊑ A`), `owl:inverseOf`, `owl:equivalentClass`/`equivalentProperty` (decomposed to inclusion pairs, named operands only), and **unqualified** `∃R` `owl:someValuesFrom owl:Thing` restrictions. Non-QL axioms are counted in `RewriteReport::skipped_axioms`, never applied.
- **TBox-capture accounting (`TBox` struct, `sq-pbz04.3.3`).** `TBox::extract(triples)` now tallies every `rdfs:`/`owl:` triple it sees: positive inclusions go to `concept_incl`/`exists_super`/`role_incl`; `skipped` counts non-QL constructs (unchanged); **`consistency_relevant`** counts QL-legal negative/disjointness axioms (`owl:disjointWith`/`propertyDisjointWith`/`complementOf`) — present but not applied for rewriting (this crate does not check consistency); **`unrecognised_schema`** counts OWL/RDFS constructs the extractor does not classify — either an `rdfs:`/`owl:`-predicate triple not handled above, or an `rdf:type` triple whose object is unmodelled schema vocabulary (e.g. `:p rdf:type owl:FunctionalProperty`); `fully_captured()` can therefore be `false` even when all *predicate* IRIs are outside the `rdfs:`/`owl:` namespace. **`TBox::fully_captured()`** returns `true` iff `skipped == 0 && unrecognised_schema == 0` — an accounting/honesty signal that no schema triple was silently dropped, not a DL-Lite_R completeness proof. A `consistency_relevant > 0` count does not block `fully_captured()`.
- **Production path (`rewrite_production`):** baseline PerfectRef **augmented** with bounded **tree-witness** folding (existential witnesses captured with no unbounded chase) then **UCQ-containment minimisation** (drop disjuncts contained in a retained one). Same certain answers as `rewrite`, in a smaller UCQ. **Minimisation is FAIL-CLOSED:** containment is NP-complete, the homomorphism search is bounded, and an **undecided-within-budget** check **KEEPS** the disjunct — minimisation only ever removes a disjunct **proven contained**, so it removes no answers.
- **Oracle-tested; the FORMAL DL-Lite_R suite GRADUATED to a pinned floor (sq-qo1a9).** Validated against a hand-checked DL-Lite_R oracle (`sparq-reason-ql/tests/oracle.rs`, incl. tree-witness + minimisation cases), because no Rust PerfectRef reference exists to diff against. There is **no consistency checking**. On the **formal DL-Lite_R suite** — the hand-derived certain-answer oracle from `sq-g19x0`, every case a conjunctive query within sound rewriting — the rewrite is **sound AND complete case by case**: `rewrite_production`'s UCQ, evaluated over the **unmodified ABox** through the real engine, returns **exactly** the hand-derived certain answers. That is now a **pinned floor** (`sparq-conformance`'s `tests/ql_dllite_suite.rs`, opt-in `ql-experimental`; `QL_DLLITE_FLOOR = 11` sound-and-complete cases), registered as a **`sparq extension`** row in the central scoreboard (`scoreboard::SUITES`) and tallied **separately** — **NOT folded into the standards-conformance total**, and **NOT a full-OWL-2-QL-conformance claim** (there is no runnable normative W3C QL certain-answer suite; the W3C QL material is structural). Like the RIF-Core / RSP / BM25 extension rows, it pins a faithful sparq-OWN oracle. `QL_DLLITE_FLOOR` is read textually by `tests/scoreboard_floors.rs`, so the mirrored scoreboard value cannot drift.
- **The `pr:QL` `sparql11/entailment` arm: the SOUND subset is GRADUATED to a pinned named-case floor (sq-pbz04.3.4); the rest stays held with an exhaustive reason taxonomy (sq-kuvu3; opt-in `sparq-conformance/ql-experimental`).** Every `sd:EntailmentProfile pr:QL` case runs through a **six-condition graduation predicate** (`inference::sparql_entail::run_ql_graduation`), each condition **checked in code, never assumed**: (1) the fail-closed CQ-shape gate accepts the query AND it carries no intensional schema-vocabulary atom (B6, sq-pbz04.3.1 — now built into the gate); (2) the TBox is **totally captured** (`fully_captured()`, sq-pbz04.3.3); (3) **zero consistency-relevant** (negative/disjointness) axioms — no consistency check exists, so their presence means possible under-approximation; (4) default-graph dataset only; (5) the **regime-coincidence guard** — the crate computes CERTAIN ANSWERS while W3C entailment-regime solution mappings bind every variable to an RDF term; the semantics provably coincide iff all body terms are distinguished (a body blank node counts as a non-distinguished variable) OR the TBox has no existential-generating inclusion (`exists_super` empty) — the fail-closed §4 default, deliberately not widened; (6) the rewritten UCQ evaluated over the **unmodified data** is **result-equivalent to the W3C oracle**. The graduated cases form the **pinned named-case floor** `QL_ENTAILMENT_FLOOR_CASES` in `sparq-conformance`'s `tests/ql_entailment_floor.rs` (exact set equality: a regressing pinned case AND an unpinned newly-eligible case both fail — additions need an evidence-carrying PR; enforced in the `inference-conformance` CI job), mirrored as a **`sparq extension`** scoreboard row (`QL_ENTAILMENT_FLOOR`, read textually by `tests/scoreboard_floors.rs`) — **NEVER summed into the standards-conformance total, NOT a full-regime/full-profile OWL 2 QL conformance claim**. Every non-graduated case carries a specific taxonomy hold: **permanently-outside** (BIND / variable predicates / intensional schema queries / OPTIONAL–MINUS shapes — no sound rewriting in this design), **pending-gate** (B1/B2/B3/B4/B6 landed under sq-pbz04.3.1; **(B5)** non-recursive property-path desugaring — sequence/inverse/alternation — now landed under sq-pbz04.3.2, so a path-shaped CQ is no longer held at the gate; recursive/zero-length/negated paths stay fail-closed; **(Bbnode)** body blank nodes lifted to fresh existential variables so the applicability condition applies correctly — sq-pbz04.3.6), **pending-capture**, **pending-consistency** (bucket size reported; a DL-Lite_R consistency-check bead is warranted iff non-empty — currently 0), **pending-coincidence**, **oracle-divergent**, or **inconclusive** — plus a loud **unclassified-abstain** bucket the floor test asserts EMPTY, so no new abstain class can hide in a catch-all. In the inference BINARY every QL row (graduated or held) remains OutOfScope — no QL row can inflate the binary's conformance ratchet (the D-entailment precedent); `tests/ql_experimental_arm.rs` asserts exactly that plus the taxonomy invariants.

## OWL 2 Direct Semantics (`sparq-reason-dl`, separate opt-in crate — all five layers built: L1 model, L2 profile checker, L3 ALCH tableau, L4 dispatch, L5 conformance arm)

The three profile reasoners above (RL / EL / QL) each cover a *tractable* OWL fragment; **OWL 2 Direct Semantics** (the model-theoretic DL semantics) covers the boolean heart of DL — arbitrary `⊔` / `¬` / `∀` — that none of them can reach. `sparq-reason-dl` is a **separate opt-in crate** building a **layered, fail-closed Direct-Semantics checker**; **all five layers are built: L1 (structural model + extractor), L2 (syntactic EL/QL/RL profile checker), L3 (ALCH tableau — the first layer that does semantic reasoning), L4 (the fragment-dispatch `DirectChecker` + entailment-by-refutation, behind the crate's opt-in `dispatch` feature, bead sq-pbz04.4.4), and L5 (the `sparq-conformance` DIRECT-arm behind that crate's opt-in `dl-direct` feature, bead sq-pbz04.4.5)**. HONEST SCOPE: this is **not** full OWL 2 DL (SROIQ(D) satisfiability is 2NEXPTIME-complete and deliberately out of scope) — it is a scoped **ALCH-fragment** effort, sound/complete only within the argued fragment. L1 delivers:

- **A structural OWL model** (`sparq_reason_dl::model`) — `Axiom` / `ClassExpression` / `ObjectPropertyExpression` typed enums for the ALCH fragment: named classes, `owl:Thing`/`owl:Nothing`, `owl:intersectionOf` (⊓), `owl:unionOf` (⊔), `owl:complementOf` (¬), `owl:someValuesFrom` (∃R.C) and `owl:allValuesFrom` (∀R.C) over **named object properties**; GCIs, `owl:equivalentClass`, `owl:disjointWith`, `rdfs:subPropertyOf`, `rdfs:domain`/`rdfs:range`, and a ground ABox. Purely structural — **no semantics attached at L1**.
- **A FAIL-CLOSED reverse RDF mapping** — `extract(&Dict, &[[Id; 3]]) -> Result<Ontology, ExtractError>` maps the `(Dict, triples)` substrate into the model per the W3C *Mapping to RDF Graphs* tables restricted to ALCH. **A single out-of-fragment or malformed triple aborts the WHOLE extraction** with a typed `ExtractError`, rather than being silently dropped: the (future) checker must never reason over a graph it only *partially* understood — a dropped axiom can flip a consistency verdict. Understood in full, or refused. The rejection taxonomy has five arms — `OutOfFragment` (cardinality / nominals / inverses / `owl:sameAs` / property characteristics / chains / keys), `DataConstruct` (datatypes / data properties — no concrete domain in L1), `MalformedList`, `MalformedClassExpression`, `Unclassifiable` (an undeclared predicate that cannot be mapped soundly) — while annotations, declarations, and ontology headers are recognised and ignored.

**L2 — syntactic EL/QL/RL profile-membership checker (`profile`, bead sq-pbz04.4.2):** NOW
BUILT. `profile::profiles(onto: &Ontology) -> ProfileSet` runs a purely syntactic grammar walk
(W3C OWL 2 Profiles §2/§3/§4) over the structural model and returns a `ProfileSet` with three
fields — `el`, `ql`, `rl` — each a `Membership` enum: `Membership::In` (all axioms pass),
`Membership::NotIn(reason)` (first violation, fail-fast), or `Membership::Unknown(err)`
(extraction failure, only from `profile::profiles_from_extraction(&Result<Ontology, ExtractError>)`).
Convenience methods: `Membership::is_in()` / `is_not_in()` / `is_unknown()`;
`ProfileSet::in_all()` / `in_any()`. An empty ontology is `In` all three profiles. Terminating
by construction — no semantic reasoning, just a grammar walk over the finite acyclic structural model.

**L3 — terminating ALCH tableau (`nnf` + `tableau`, bead sq-pbz04.4.3):** NOW BUILT — the
consistency / class-satisfiability core. `tableau::consistency(&Ontology, Budget)` and
`tableau::class_satisfiability(&ClassExpression, &Ontology, Budget)` return a tri-state
`Verdict` — `Satisfiable`, `Unsatisfiable`, or `Unknown(UnknownReason)` — and
`tableau::consistency_from_extraction(&Result<Ontology, ExtractError>, Budget)` is the
fail-closed RDF-level entry: ANY extraction failure yields `Unknown(OutOfFragment)` BEFORE the
tableau starts (the checker never reasons over a partially-understood graph). The engine is a
completion-forest tableau with GCI internalisation, `⊓`/`⊔`/`∃`/`∀`/GCI rules matched modulo
the `rdfs:subPropertyOf` closure, **ancestor subset blocking** (sufficient precisely because
ALCH has no inverse roles), and chronological backtracking over `⊔`-branches. The full
termination / soundness / completeness argument — citing Baader–Sattler 2001, including why
subset blocking would be insufficient with inverses — is reproduced in the `tableau` module
docs (§3–§5). Budgets are **deterministic counts only** (`Budget { max_nodes,
max_rule_applications }`; wall-clock budgets banned); exhaustion yields
`Unknown(ResourceBudget)`, **never a verdict**. `nnf` provides the negation-normal-form
rewrite (`nnf` / `nnf_complement` / `is_nnf`) and the finite `subexpression_closure` the
termination argument rests on. HONEST BOUNDARY: verdicts are sound/complete ONLY for the exact
ALCH fragment (named classes, ⊤/⊥, ⊓/⊔/¬, ∃/∀ over named properties, GCIs, `subPropertyOf`,
ground ABox) — never beyond it; the implementation is not claimed worst-case optimal (ALC+GCI
satisfiability is EXPTIME-complete).

**L4 — fragment-dispatch checker + entailment-by-refutation (`check`, opt-in `dispatch`
feature, bead sq-pbz04.4.4):** NOW BUILT. `check::DirectChecker` (constructed with `new()` or
`with_budget(Budget)`) dispatches an extracted ontology IN ORDER — RL (via `sparq-reason`
materialization + clash scan, Theorem-PR1-precondition-CHECKED, divergence-guarded), EL (via
`sparq-reason-el`, triple-guarded: skipped-axioms / unapplied-axiom-kinds / ⊤-guard), QL
(consistency wholly deferred to sq-pbz04.3.4 — always abstains), else the L3 ALCH tableau —
returning `ConsistencyOutcome` / `EntailmentOutcome`: a tri-state verdict PLUS the `Branch`
that produced it (traceability). `entailment()` checks `O ⊨ α` per conclusion axiom by an
argued refutation encoding onto the tableau (`SubClassOf`, `ClassAssertion`,
`ObjectPropertyAssertion` via a fresh-class encoding, `EquivalentClasses`, `DisjointClasses`,
domain/range; property-axiom conclusions abstain). Every guard fails CLOSED — uncertainty is a
typed `UnknownReason`, never a guessed verdict. **Conclusion anonymous individuals
(sq-pbz04.4.13):** a blank-node individual in the CONCLUSION is read EXISTENTIALLY (per the
official Direct-Semantics tests) — L1's skolem-constant reading is entailment-preserving on the
premise but would certify a WRONG `NotEntailed` on the conclusion, so before the refutation loop a
TREE-shaped anonymous assertion set (`a p _:x`, `_:x` typed / chaining to more blank nodes) **rolls
up** into an existential class assertion `a : ∃p.(⊓ types ⊓ ⊓ ∃q.⟨child⟩)` the tableau decides
SOUNDLY in both directions (so `somevaluesfrom2bnode` / `WebOnt-someValuesFrom-003` graduate to
genuine passes); any non-rollable shape — shared between two assertions, cyclic, a named/nominal
successor, or an unanchored free-existential root — abstains fail-closed
(`ConclusionAnonymousIndividual`), never a skolem `NotEntailed`.

**L5 — the conformance DIRECT-arm (`sparq-conformance`, opt-in `dl-direct` feature, bead
sq-pbz04.4.5):** NOW BUILT. `inference::dl_suite::run_direct_arm` runs the DIRECT-sanctioned
arm of the OWL WG export (`tests/w3c/owl2/all.rdf`) with **tri-state accounting
`{Pass, Fail, OutOfFragment(reason)}` — an abstention is NEVER a pass**: a
profile-identification lane (the L2 checker vs the export's POSITIVE `test:profile` tags
ONLY — the explicit-negative `owl:NegativePropertyAssertion` direction was MEASURED and not
adopted, because L2's `In` is fragment-grammar membership and cannot refute full-profile
membership (runner module docs); nothing inferred from a missing tag; the `test:species`
DL/FULL check stays deferred) and a Direct
consistency/inconsistency/entailment lane (the L4 `DirectChecker` under a PINNED
deterministic count budget). Floors are EXACT-pinned in `tests/dl_suite.rs`
(`DL_PROFILE_FLOOR` / `DL_DIRECT_FLOOR`, `==` not `>=`, so abstention-inflation and
regression both fail), mirrored as **`sparq extension`** scoreboard rows labelled **scoped
fragment — NOT full OWL 2 DL**, never folded into standards-conformance totals; the
dual-tagged tests' RDF-Based runs stay in the RL `owl_suite` / `el-suite` lanes (separate
semantics). Functional-syntax-only inputs (27 cases) and `owl:imports` are OutOfScope;
`test:status test:Rejected` is excluded.

Deferred constructs — inverse roles, cardinality/functionality, nominals, transitivity,
`sameAs`/`differentFrom`, datatypes, keys — are each **rejected, never mis-mapped**, with a
named reason and unlock path in the design record's deferral ledger. See
`research/owl2-direct-semantics-scoping.md`.

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
- **Features:** `parallel` (default, rayon-parallel fixpoint), `explain` (NON-default — enables `why()`/`why_with()` and the `explain` module; zero hot-path cost when off, and `why` methods don't exist without it), `d-entail` (NON-default — enables `Profile::D` + the `dtype` module; zero code when off — the lean default/wasm build is byte-identical, `sq-e5atd`), `rif-core` (NON-default — enables the `rif` module: the RIF-Core monotone-Horn rule front-end over the N3 chainer with range-restriction safety; zero code when off, no new `Profile` variant, `sq-rh4gu`), `substrate-join` (NON-default — the RDFS predicate join + the rdfs9 type join + the OWL-RL `Δ⋈full` delta adjacency (`sq-qonbz.2`) drive the SHARED `sparq-substrate::join` kernels; `sq-yk6or` + `sq-pbz04.1.1` + `sq-qonbz.2`, see next bullet), `substrate-compare` (NON-default — the `compare` module: the SHARED `sparq-substrate::compare` SPARQL term total order implemented for dictionary ids, so entailed-solution ordering is parity-identical to the engine's `ORDER BY`; zero code when off, `sq-pbz04.1.2`, see the bullet after next), `compiled-rules` (NON-default — the `n3::compiled` module: id-level COMPILED N3 evaluation for the access-control rule subset over the shared substrate join kernels; zero code when off, `sq-zgbso.3`, see the Key-APIs compiled block).
- **Shared join kernels (`substrate-join`, opt-in, `sq-yk6or`, epic `sq-pbz04`).** The RDFS single-pass predicate join — rdfs7 (subPropertyOf rewrite), rdfs2 (domain typing), rdfs3 (range typing), keyed on the asserted triple's predicate — and the rdfs9 subclass-typing join (`sq-pbz04.1.1`, keyed on the type-assertion's OBJECT column: the "orientation" is just a different `JoinKeys` probe-column index) drive the *same* `sparq_substrate::join::{build_table, probe_emit, hash_probe_serial}` hash-join body the SPARQL engine drives (epic `sq-qonbz` Phase 3, #1300). The reasoner supplies its OWN `JoinKeys` (predicate-keyed) + its OWN `Budget` (the unbounded `NoBudget`; materialisation runs to completion — a closure-level budget is a fixpoint concern, installed around the whole call, not per-join), monomorphically — no `Box<dyn>`/vtable on the probe loop. This is the end-to-end proof of "share join logic across the engine AND the reasoners" (`research/shared-eval-substrate.md` Phase 5). **Behaviour-neutral:** the materialised closure is byte-identical to the hand-rolled `FxHashMap` adjacency path (asserted per-branch by `rdfs::tests::substrate_join_emits_identical_plain_branch` / `substrate_join_emits_identical_type_branch`, and whole-closure by `closure_is_byte_identical_across_join_paths`, which runs in BOTH feature states); only the join machinery changes. **OFF by default** so the byte/bundle ratchets stay exactly the hand-rolled path; the only deps it pulls (`sparq-substrate` `rows`+`join`, `smallvec`) are already in the crate's tree. **Residual disposition (`sq-pbz04.1.1`):** the `PropExpand` inverseOf/Symmetric predicate-rewrite branch is RETAINED hand-rolled *permanently* — its per-match combine is data-dependent (the `swapped` flag picks the subject/object orientation per matched build row) and cascades into a second dom/rng join keyed on the DERIVED predicate, a variable-arity shape the kernel's one-fixed-row-per-match combine cannot express without rebuilding the rule structure around it (full rationale in `substrate_join.rs`; the oriented emission is pinned by `rdfs::tests::prop_expand_inverse_types_through_oriented_domain` so any future adoption attempt inherits a red/green harness). **OWL-RL delta adjacency (`sq-qonbz.2`, NOW SHIPPED under this same feature):** the semi-naive `Δ⋈full` adjacency for `prp-fp` (functional), `prp-ifp` (inverse-functional), and `prp-trp` (transitive) is also behind `substrate-join`. A persistent `DeltaAdj` struct (two `DeltaTable`s — forward `out_tbl` keyed on `[p,s]`, backward `inc_tbl` keyed on `[p,o]`) replaces the per-round nested `FxHashMap` probes; `extend_one` grows both tables incrementally as new delta triples commit, and `probe_out`/`probe_inc` emit results via a generic `FnMut(Id)` closure (monomorphised, no `Box<dyn>`, no vtable — `check-no-dyn-dispatch.py` is clean). **Behaviour-neutral:** the OWL-RL ratchet output is byte-identical in both feature states; the three probe paths (`prp-fp` forward, `prp-ifp` backward, `prp-trp` backward) are pinned by `tests/substrate_join_owl.rs` (8 required-feature tests: fp/ifp/trp alone and in combination, closure-length and no-chain guards). UnionFind (`sameAs` merge) is NOT touched by this change.
- **Shared term total order (`substrate-compare`, opt-in, `sq-pbz04.1.2`, substrate seam 3).** The `compare` module implements the substrate's `CompareTerm` trait for the reasoner's term representation — a dictionary `Id` resolved against its `Dict` (`compare::IdTerm`) — so `compare::compare_ids` / `compare::sort_ids` order ids under the *same* `sparq_substrate::compare::compare_terms` total order the SPARQL engine's `ORDER BY` drives: error/unbound < blank < IRI < literal < RDF 1.2 triple term; literals numeric-aware (with the `exact_cmp` f64-collapse recheck for distinct integers past 2^53), then strict typed/temporal (`xsd:dateTime`/`xsd:date` by TIMELINE via the shared `sparq_core::temporal::Timeline` — cross-timezone order, not lexical; booleans; same-tag language strings; same-other-XSD lexically), then lexical string fallback; triple terms component-wise through the dict's structural component ids. **Ordering parity is pinned byte-for-byte against a REAL engine `ORDER BY`** over the same materialised closure (`tests/compare_parity.rs`, a mixed IRI/bnode/literal/triple-term fixture whose entailed rows participate); the observation hooks reuse the shared machinery (`Timeline`, the substrate `Num`/`Dec` tower, `parse_xsd_f64`) rather than reimplementing it, and the small `Num::of_literal` borrowed-parts mirror is anti-drift-pinned by a unit test against the substrate itself. Adopted MONOMORPHICALLY — `IdTerm` is a generic `CompareTerm` impl, no `Box<dyn>`/`&dyn` between the sort loop and the comparator (`scripts/check-no-dyn-dispatch.py` lists the module). **Purely additive:** no materialiser calls it — which triples are entailed and their emission order are byte-identical in both feature states; undecidable pairs (e.g. `NaN`) collapse to `Equal` exactly as the engine's sort does, and equal-comparing DISTINCT terms (equal values across datatypes, equal instants across timezones) keep stable-sort input order on both sides — the engine's own tie semantics, not a divergence.
- **Two value levels.** RDFS/OWL APIs work on dictionary `Id`s (`materialize*`, `Materialized(Owl)Graph`); N3 batch APIs intern into a `Dict` (`reason_n3`), while term-level N3 (`reason_n3_terms`, `MaterializedN3Graph`) works on `n3::Term` and is **not interned** (formula `{ … }` terms have no dictionary id). Don't mix the two.
- **The materialize → from_parts seam.** `materialize` mutates `(Dict, Vec<[Id;3]>)` *before* indexes are built. Use `Graph::parse_to_triples` (not `Graph::load_str`) so reasoning runs between parse and index build; then `Graph::from_parts`. It interns any vocabulary terms it needs and is idempotent (a second call adds nothing).
- **RDFS scope is deliberate:** the non-explosive subset (rdfs2,3,5,7,9,11 — subClass/subProperty/domain/range). No axiomatic or reflexive `rdfs:subClassOf`/`type` triples (they add no useful inferences and explode the store).
- **D-entailment (`Profile::D`, opt-in `d-entail`) scope + caveats:** materializes the rdfD1 datatype-typing rule — a well-formed literal `"l"^^d` of a *recognized* datatype `d` (the `Recognized` map; `xsd:string`/`rdf:langString` always, `Recognized::standard()` adds the numeric/boolean/temporal core) entails `"l"^^d rdf:type d`. The emitted typing triples are **generalized** (literal in subject position) — feed the closure to a query only after dropping literal-subject rows (they can never be a SPARQL answer; this is also why the W3C `d-ent-01` test correctly returns NO rows). The load-bearing invariant is **value-space equality** via `d_value_eq`: `"1"^^xsd:integer` ≡ `"1.0"^^xsd:decimal` (the integer/decimal value spaces coincide), compared as a CANONICAL DECIMAL STRING — **never an f64 fast path** (f64 silently aliases integers past 2^53 and loses decimal precision). `float`/`double` are a DISJOINT IEEE-754 value space; `date` and `dateTime` are disjoint temporal families. NOTE: the SHARED SPARQL term total order now lives in `sparq-substrate::compare` (`compare_terms` over the generic `CompareTerm` trait — error/unbound < blank < IRI < literal < triple, numeric-aware + strict typed/temporal + string fallback; epic `sq-qonbz` Phase 4, `sq-vezew`, #1300-chain). A reasoner that orders entailed solutions (RIF `order`, an EL/QL `ORDER BY` over a materialised answer set) reuses it by implementing `CompareTerm` for its own term type — the same monomorphisation seam `substrate-join` uses for `JoinKeys`; sparq-reason now SHIPS that impl for dictionary ids behind the opt-in `substrate-compare` feature (`sq-pbz04.1.2`, see the shared-term-total-order bullet above). The trait carries an `exact_cmp` **f64-collapse recheck** hook (`sq-rikm7`): the numeric arm coerces to f64 for speed, and when two operands tie there `exact_cmp` recovers the exact order of distinct integers past 2^53 / high-precision decimals — so a reasoner `ORDER BY` / `MIN` / `MAX` agrees with the relational `=`/`<` rather than falling into the very f64-aliasing this caveat warns about (return `None` from it if your term type has no exact numeric tier). D's typed *value-space-equality* comparator (`d_value_eq`, used for entailment not ordering) stays reasoner-resident for now; D-inconsistency (ill-typed-literal / value-space clashes) and cross-type value-space *subset* reasoning are tracked-not-yet-shipped here (epic `sq-pbz04`).
- **OWL 2 RL is sound but INCOMPLETE for class classification.** Running `Profile::OwlRl` / `--reason owl` over an EL ontology returns a `rdfs:subClassOf` hierarchy that silently omits existential-reasoning subsumptions (the calculus has no rule reasoning through an `∃r` successor). For the **complete** class hierarchy use `sparq-reason-el` (above), not more RL rules.
- **The RL materializer is COMPLETE for the assertion-style RL/RDF rules — the W3C OWL-RL conformance row is at the RL ceiling (sq-350ms).** Every rule with a positive-assertion head in Profiles §4.3 Tables 5/6/9 is implemented (the `owl.rs` per-rule status table + `research/inference-completeness-audit.md` §2/§2b are the per-rule proof). The 13 documented OWL-RL conformance divergences are PROVABLY outside the RL profile, **not** missing rules: TBox-axiom conclusions, invented class expressions (`owl:complementOf`/`unionOf`), reified `owl:AllDifferent` structures, the `prp-pdw`/`prp-fp`/`prp-ifp` **contrapositives** (RL has NO rule producing `owl:differentFrom` between INDIVIDUALS — `dt-diff` emits it only between unequal-value literals, otherwise it appears only in clash bodies), `owl:ReflexiveObjectProperty` (EXCLUDED from the RL grammar — there is no `prp-rfx`), and datatype-range INTERSECTION. They stay documented divergences (closing them would be unsound or beyond-profile); the inference ratchet HOLDS — see `inference-conformance-report.md` and the central scoreboard (`scoreboard::SUITES`, CI job `inference-conformance`) for the current pinned count. Multi-round assertion-rule completeness and the prp-pdw/prp-fp soundness boundary are pinned by in-crate guards in `owl.rs::tests`; the per-divergence disposition pass (sq-pbz04.1.3) re-audited all 13 from the raw export premises/conclusions (verdict: 13/13 PERMANENT, zero in-profile fixes), tagged every report-facing rationale `PERMANENT — …` with its rule-level grounding, and pinned tag+grounding with an in-crate disposition test in `owl_suite.rs`.
- **OWL incremental fallback is silent.** `MaterializedOwlGraph` drops to `OwlMode::Fallback` (re-materializes via `materialize_owl_rl` every mutation, still correct) when the base uses `owl:sameAs`, Functional/InverseFunctional, property chains, restrictions, cardinality, hasKey, oneOf, intersection/union — and on any TBox mutation. Check `.mode()` / `.full_rebuilds()` if incremental cost matters. These usually live in a static TBox, so the mode is decided once at load.
- **N3 incremental qualification is narrow.** `MaterializedN3Graph` only runs `N3Mode::Counting` (truly incremental) for a monotone, input-stratified rule fragment: forward rules with ground-IRI predicates, no conclusion blank nodes, builtins limited to the parity whitelist (`log:uri`, `log:equalTo`/`notEqualTo`, `string:concatenation`/`scrape`/`encodeForUri`), and negation only via the store-scoped `?x log:notIncludes { … }` idiom over input-only predicates. Anything else → `N3Mode::Fallback`; always consult `.fallback_reason()` (`None` ⇔ counting active). The full *batch* N3 engine (`reason_n3`) supports the much larger `math:`/`string:`/`list:`/`time:`/`log:` builtin set and goal-directed `<=` rules.
- **`why()` is a witness, not a proof set.** It returns the first derivation in deterministic order, or `None` if the triple isn't in the closure or `ExplainOpts` caps (default depth 128, 65 536 nodes) are exceeded — not an enumeration of all derivations.
- **Deletion semantics:** `delete` removes *base* (asserted) triples; a deleted base triple still derivable from the remainder stays in the closure, and deleting a derived-only fact is a no-op (standard materialized-view semantics).

## Migrating from eye-js (`@sparq-org/eyereasoner-compat`)

The npm package **`@sparq-org/eyereasoner-compat`** (`packages/eyereasoner-compat/`) is a
drop-in for [eye-js](https://github.com/eyereasoner/eye-js)'s `n3reasoner(data, query?, options?)`,
backed by this crate's wasm bundle (`crates/sparq-reason-wasm`) — no SWI-Prolog, a lighter
browser payload. It maps the eye-js surface onto three wasm entry points and is HONEST about the
boundary:

- **Output modes.** `derivations` (default, EYE `--pass-only-new`) → `Reasoner.reasonN3New`;
  `deductive_closure` (`--pass`) → `Reasoner.reasonN3`; `none` → empty. The `…_plus_rules` modes
  (`--pass-all` / `--pass-all-ground`, which echo rules into the output) **throw** — sparq's
  chainer consumes rules and emits only ground triples (a deferred follow-up bead, not faked).
- **Query filter.** `n3reasoner(data, query)` maps the EYE `--query` rule to a SPARQL `CONSTRUCT`
  over the materialised closure (`Reasoner.reasonN3Query`). Query rules using **builtins /
  `{ … }` formulae / `( … )` lists fail closed** (a clear error, never a wrong answer).
- **Builtins.** Only this crate's `math:`/`string:`/`list:`/`time:`/`log:` subset is available
  (EYE's full library is larger); an unsupported builtin simply does not fire.
- **SWIPL surface.** `SwiplEye` / `loadEyeImage` / `runQuery` / `EYE_PVM` / `linguareasoner` etc.
  are re-exported as **throwing migration stubs**; `SWIPL` / `cb` options warn-and-ignore.

See `packages/eyereasoner-compat/README.md` for the CDN (esm.sh/jsdelivr/unpkg) usage and the
full output-mode + builtins-coverage tables.

## See also

- `noir-circuit-patterns`, `noir-optimisation`, `verifiable-credentials-zk`, `sparql-formal-semantics` — the single-prover ZK estate; the `explain` `ProofTree` is intentionally a flat, id-free, premises-before-conclusion DAG meant as a ZK-derivation witness.
- `mpc-protocols` — multi-party layer over (federated) SPARQL.
- `hdt-format`, `fused-decompress-parse`, `rust-parallel-parsing` — sibling ingest/storage skills for getting triples into the graph you then reason over.
- `research/owl2-el-ql-reasoning-spike.md` — the EL/QL feasibility spike: why EL first, the RL-incompleteness proof (the CR4 counterexample), and the phased plan (E1–E6) `sparq-reason-el` implements.
- `research/reasoner-suite-on-substrate.md` §2.5 — the QL track design: the PerfectRef applicability trap, the strict CQ-shape gate, and why the production path (tree-witness + UCQ-containment minimisation) is sequenced late by soundness risk (the phased plan `sparq-reason-ql` implements through phases Q1–Q3; only the conformance-floor graduation remains deferred).
