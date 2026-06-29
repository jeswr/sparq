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
// Which regime to materialize. OwlRl includes RDFS.
pub enum Profile { Rdfs, OwlRl }
impl Profile { pub fn parse(s: &str) -> Option<Profile>; } // "rdfs" | "owl" | "owl-rl"

// Batch materialization (expand in place; returns NEW triples; idempotent).
pub fn materialize(profile: Profile, dict: &mut Dict, triples: &mut Vec<[Id;3]>) -> usize;
pub fn materialize_rdfs (dict: &mut Dict, triples: &mut Vec<[Id;3]>) -> usize;
pub fn materialize_owl_rl(dict: &mut Dict, triples: &mut Vec<[Id;3]>) -> usize;

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
- **EXPERIMENTAL — oracle-tested, NOT graduated to a conformance floor.** Validated against a hand-checked DL-Lite oracle (`tests/oracle.rs`, incl. tree-witness + minimisation cases), because no Rust PerfectRef reference exists to diff against. There is **no consistency checking**. Graduating the QL entailment-regime arm to a pinned conformance floor is a separate, deferred bead (it must sequence through the contended conformance scoreboard; epic `sq-pbz04`).

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
- **Features:** `parallel` (default, rayon-parallel fixpoint), `explain` (NON-default — enables `why()`/`why_with()` and the `explain` module; zero hot-path cost when off, and `why` methods don't exist without it).
- **Two value levels.** RDFS/OWL APIs work on dictionary `Id`s (`materialize*`, `Materialized(Owl)Graph`); N3 batch APIs intern into a `Dict` (`reason_n3`), while term-level N3 (`reason_n3_terms`, `MaterializedN3Graph`) works on `n3::Term` and is **not interned** (formula `{ … }` terms have no dictionary id). Don't mix the two.
- **The materialize → from_parts seam.** `materialize` mutates `(Dict, Vec<[Id;3]>)` *before* indexes are built. Use `Graph::parse_to_triples` (not `Graph::load_str`) so reasoning runs between parse and index build; then `Graph::from_parts`. It interns any vocabulary terms it needs and is idempotent (a second call adds nothing).
- **RDFS scope is deliberate:** the non-explosive subset (rdfs2,3,5,7,9,11 — subClass/subProperty/domain/range). No axiomatic or reflexive `rdfs:subClassOf`/`type` triples (they add no useful inferences and explode the store).
- **OWL 2 RL is sound but INCOMPLETE for class classification.** Running `Profile::OwlRl` / `--reason owl` over an EL ontology returns a `rdfs:subClassOf` hierarchy that silently omits existential-reasoning subsumptions (the calculus has no rule reasoning through an `∃r` successor). For the **complete** class hierarchy use `sparq-reason-el` (above), not more RL rules.
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
