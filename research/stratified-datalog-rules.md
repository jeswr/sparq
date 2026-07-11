# Stratified Datalog rules: NAF + aggregation with a checked stratification (sq-6tykl.3)

<!-- [FABLE-5] Design record for the RDFox-parity stratified-Datalog BIG ROCK (epic
sq-6tykl, cross-ref competitive-parity epic sq-lsp7k). Phase 1 ships WITH this record:
crates/sparq-reason/src/datalog/ behind the opt-in `datalog` feature. Later phases are
beaded from §6. -->

> 🤖 **SPARQ agent** — design-for-review, written under the standing
> proceed-and-document rule (the surface-syntax choice in §2 resolves the maintainer's
> open question 4 of `research/competitive-feature-analysis-2026-07.md`; steer post-hoc
> via the tracking issue).

## 1. Why

RDFox's rich Datalog dialect — `NOT`/`NOT EXISTS` and `AGGREGATE` atoms under
stratification, incrementally maintained — is the single feature enterprises buy RDFox
for beyond OWL: real business rules (thresholds, counts, absence checks) that OWL cannot
express. Stardog's equivalent (Stride) is beta; GraphDB's `.pie` has no aggregates.
sparq already has the substrate for it: forward chainers (RDFS/OWL-RL/N3), an id-level
compiled-rule pipeline on the shared `sparq-substrate::join` kernels, store-scoped NAF
(`log:notIncludes`) — but **no stratification checker** (the N3 compiled path documents
stratification as a *caller discipline*), **no aggregates in rule bodies**, and no
Datalog surface syntax.

## 2. Surface syntax: a small native dialect (open question 4)

Options considered:

| option | for | against |
|---|---|---|
| **N3 builtins** | parser + NAF (`log:notIncludes`) exist | no aggregate idiom short of cwm's list-valued `log:collectAllIn` (formula/list-valued facts are exactly what the compiled subset excludes); extending N3 semantics non-standardly taints a W3C-adjacent surface |
| **RIF extension** | `rif::Document` validator exists | RIF-Core is monotone *by definition* — NAF/aggregates would be a nonstandard fork of a standard; XML presentation syntax is hostile to hand-written business rules |
| **native dialect (CHOSEN)** | direct RDFox rule-migration parity (the business goal); stratification + aggregation are first-class in the syntax; N3/RIF keep their standard semantics untainted | a new (small) parser |

**Decision: a small native rule dialect modelled on RDFox's Datalog surface**, lowered
onto the SAME machinery everything else uses (substrate join kernels, the substrate
`numeric` tower, dictionary ids). The dialect is deliberately tiny; anything outside the
fragment is a loud parse error naming the construct.

```text
@prefix ex: <http://example.org/> .
[?x, ex:deg, ?c] :- AGGREGATE([?x, ex:edge, ?y] ON ?x BIND COUNT(?y) AS ?c) .
[?x, a, ex:Hub]  :- [?x, ex:deg, ?c], FILTER(?c >= 3) .
[?x, a, ex:Leaf] :- [?x, a, ex:Node], NOT [?x, a, ex:Hub] .
```

Grammar (Phase 1): `@prefix` declarations; rules `head :- body .` with comma-separated
triple-pattern atoms `[s, p, o]` (constant predicate; `a` = `rdf:type`; IRIs, prefixed
names, `?vars`, bare-integer/`"…"^^dt` literals); body elements `atom`, `NOT atom`,
`AGGREGATE(atoms [ON ?g…] BIND COUNT(?v) AS ?c)`, `FILTER(x op y)`; multi-atom heads.

## 3. Semantics

Textbook **perfect-model (stratified) semantics**: `stratify` builds the dependency
graph — positive edges for body atoms, **non-monotonic** edges for `NOT` atoms and every
predicate inside an `AGGREGATE` body (adding a fact can change a count) — and either
assigns strata (iterative stratum-raising fixpoint, Ullman §3.8) or rejects the program
naming a node on the negation/aggregation cycle. Evaluation runs strata in order, so
every negated/aggregated relation is complete before it is read.

Load-bearing details, each pinned by a test:

- **Class granularity.** Dependency nodes are predicates, EXCEPT `rdf:type` atoms with a
  constant class, which get a per-CLASS node — RDF encodes unary relations as classes,
  so `NOT [?x, a, ex:Hub]` must not collide with an unrelated `[?y, a, ex:Leaf]` head
  (predicate-granular checking rejected exactly that program in development). A
  variable-class `rdf:type` atom is conservative: reading one sits above every class;
  deriving one feeds every class reader.
- **Safety (range restriction).** Every head/`FILTER` variable must be bound by a
  positive atom or an `AGGREGATE` (`ON`/`AS`) — checked at parse. `NOT` variables not so
  bound are existential wildcards (a repeated wildcard inside one `NOT` atom must match
  equal terms — the `log:notIncludes` idiom, kept).
- **Aggregate scoping.** Non-`ON` aggregate-body variables are aggregate-local; a name
  collision with the outer rule is a loud error (silent capture would be ambiguous).
  The `AS` output must be fresh. `COUNT(?v)` counts DISTINCT body matches per group
  (set semantics — Datalog relations are sets); empty groups produce no row, and counts
  mint `xsd:integer` literals into the caller's `Dict`.
- **FILTER** compares EXACT numerics (`xsd:integer`/`decimal` + derived integer types)
  via the shared substrate `Dec` tower; non-numeric/float operands fail the row
  (fail-closed — see §5 for the float decision).
- **Co-head predicates** of one rule share a stratum (mutual positive edges), so a
  rule's stratum is well-defined.

## 4. Architecture (Phase 1, shipped with this record)

`crates/sparq-reason/src/datalog/` behind the **opt-in `datalog` feature** (lean-core
posture: default build carries zero of it; pulls only the substrate `rows`+`join`
slices, no new transitive crate):

- `parser.rs` — recursive-descent, interns constants into the caller's `Dict` at parse.
- `stratify.rs` — the checker; public `stratify(&Dict, &Program) -> Result<Stratification>`
  so tooling can validate without evaluating.
- `eval.rs` — non-incremental per-stratum fixpoint. Positive-atom and aggregate-table
  joins drive the SHARED substrate kernels (`build_table` + `hash_probe_serial`) via the
  same thin layout-adapter shape as `crate::substrate_join` / `n3::compiled` — no new
  join implementation. Aggregate tables are computed once per stratum entry (their
  bodies read strictly-lower strata); `NOT` is a per-row absence check against the
  stratum-complete predicate index.
- `oracle.rs` (test-only) — the **differential reference**: an independent naive
  substitution evaluator (no kernels, no indexes, `i128` integer compare) that must
  agree with the engine on every fixture and on seed-randomised graphs (deterministic
  LCG). Rationale: an in-tree independent-implementation oracle is the house pattern
  (`compiled_equivalence`, the Rust-vs-N3 ODRL differential); an EXTERNAL-engine
  (Soufflé) differential arm is beaded (§6) rather than adding a CI dependency here.

Public API (5 items, each with a doctest + direct unit test): `parse_program`,
`Program::n_rules`, `stratify`, `Stratification::n_strata`, `eval`.

## 5. Honest scope: what Phase 1 does NOT do

- **Naive rounds within a stratum** (dedup at insertion) — correct, fixture-scale;
  semi-naive delta restriction is the next perf phase. No performance claims are made.
- **COUNT only**; `SUM`/`MIN`/`MAX`/`AVG` parse to a loud error (they need the value
  tower on aggregate inputs, not just outputs).
- **No `COUNT(DISTINCT ?v)`** distinct-of-one-variable form (COUNT = distinct matches).
- **No float/double FILTER operands** — exact `Dec` only, fail-closed rows. Extending
  to the engine's full relational comparison (`Num::cmp_relational`) is beaded.
- **Constant predicates everywhere**; variable predicates rejected loudly.
- **No incremental maintenance** — inserts/deletes re-evaluate; DRed/FBF-grade
  maintenance across strata is the BIG follow-up phase and must be SEQUENCED with the
  deletion-maintenance bead sq-6tykl.4 (same-crate collision, per the epic note).
- **No CLI/`MaterializedGraph` wiring** — library API only.
- The `datalog` feature is not yet in `scripts/coverage.sh`'s per-crate measurement
  (sparq-reason is measured default-features, so the module doesn't regress the floor;
  wiring it like `sparq-reason-el`'s `rbox,hasse` case is beaded).

## 6. Phased decomposition (beaded)

1. **Semi-naive per-stratum evaluation + scale bench** — delta-restricted positive
   joins (the `n3::compiled` `join_steps` discipline); measure before claiming.
2. **SUM/MIN/MAX/AVG aggregate functions** — value slot on `AggAtom`, substrate `Num`
   tower for input values, overflow semantics decided against SPARQL's.
3. **Incremental maintenance under insert/delete across strata** — counting/DRed for
   positive strata, rederivation at stratum boundaries; SEQUENCE with sq-6tykl.4.
4. **External-engine differential arm** — the same fixtures run through Soufflé (or
   crepe) in an optional CI lane; requires a cargo-vet/tooling decision.
5. **Fragment extensions** — `NOT` over conjunctions / `NOT EXISTS ?v IN`, FILTER
   expressions beyond binary numeric comparison, variable predicates (conservative ⊤
   dependency node), `COUNT(DISTINCT ?v)`.
6. **Surface wiring** — CLI `--reason datalog:<rules.dlog>`, `MaterializedGraph`-style
   handle, SKILL/docs examples beyond the API reference.
7. **N3-compiled adoption of the checker** — replace the documented caller-discipline
   stratification of `n3::compiled` `log:notIncludes` with this checked stratification.
8. **Coverage measure-case wiring** — add `--features datalog` to the sparq-reason
   measurement in `scripts/coverage.sh` (the `sparq-reason-el` pattern).
