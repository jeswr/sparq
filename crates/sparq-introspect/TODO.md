# sparq-introspect TODO

## Planner tie-in: characteristic sets → cardinality estimation — DONE (engine-seams wave)

Implemented exactly as recorded below, with the gate passed:

- **Engine seam**: sparq-engine grew the **`cs-planner` cargo feature**
  (NON-default; the wasm bundle and default native build carry zero CS code).
  `sparq_engine::cs::CsTable` holds the table in dict-id space
  (`star_subjects(Q) = Σ_{C⊇Q} count(C)`,
  `star_cardinality(Q) = Σ_{C⊇Q} count(C)·Π_{p∈Q} avg_mult(C,p)` — Neumann &
  Moerkotte) and `with_cs_table(&Arc<CsTable>, f)` installs it thread-scoped
  around execution, mirroring `QueryBudget`/`DatasetView` installation.
- **Hook points wired** (`exec::CsCtx`, a zero-sized no-op without the feature):
  `goo_pick` scores a star candidate by the conditional expansion
  `star(Q ∪ {p}) / star(Q)` instead of the independence product (selectivity over
  other shared variables keeps the marginal model), and `record_pattern_ndv` uses
  `Σ_{C⊇Q} count(C)` as the subject-variable ndv. The EXPLAIN replay shares the
  same context. Join ORDER only — results identical; full suite green with the
  feature on. Note on the third recorded hook (seed choice): the seed is the
  smallest SINGLE-pattern cardinality, which `store.estimate` answers exactly
  (index range length) — there is no independence product in it for CS to
  replace, so the seed logic is intentionally unchanged.
- **This crate's accessor**: `characteristic_set_ids(graph) -> Vec<CsIdSet>` —
  the EXACT pre-resolution table in dict-id space (no string parsing, no caps;
  one SPO scan), feedable straight into `cs::CsTable`.
- **GATE — PASSED** (`sparq-engine/src/cs_gate.rs`, both estimators measured
  through the REAL planner recurrence vs true cardinalities; q-error =
  max(est/true, true/est), floored at 1). WatDiv is not in the repo, so the
  workload is star queries over the local bench data:
  - olympics (1,781,625 triples, 19 star queries from top-CS prefixes |Q|∈2..4
    plus cross-CS pairs): PredStat median 1.00 / gmean 6.58 / max **180,604**
    vs CS median 1.00 / gmean **1.00** / max **1.0**. The blow-ups are
    predicate-correlation cases (cross-CS pairs that are empty on the data:
    independence estimates ~135k–181k rows, CS says 0). Re-run with:
    `cargo test -p sparq-engine --release --features cs-planner -- --ignored cs_gate_olympics --nocapture`
  - synthetic correlated shapes (always-on CI gate, 12 queries):
    PredStat median 1.83 / gmean 7.02 / max 1283 vs CS 1.00 / 1.00 / 1.0; the CS
    estimator is asserted EXACT on pure stars over the builder's own data.

The original analysis is kept below for the record.

## Planner tie-in: characteristic sets → cardinality estimation (the dual use)

The design doc (§5.4, introspection report §1.2) prescribes building the
characteristic-set table once and using it twice: schema summary (done here) and the
engine's star-join cardinality estimator. The precise hook, read from
`crates/sparq-engine/src/exec.rs` (read-only):

- **`eval_bgp_binary`** (exec.rs, "Binary-join BGP plan: greedy cardinality ordering")
  is the greedy (GOO) planner. It seeds with the smallest single-pattern cardinality
  (`prepared[i].est = graph.store.estimate(&id_pat)`) and then scores each candidate
  join as `out = cur_card * prepared[i].est * sel`, where `sel` divides by
  `rndv.max(pndv)` per shared variable — an **independence assumption** across the
  patterns' predicates.
- **`pattern_var_ndv`** (exec.rs, directly below) supplies the per-variable distinct
  counts from `sparq_core::store::PredStat { count, ndv_subj, ndv_obj }` — per-predicate
  marginals only; predicate *correlation* on a shared subject is exactly what it cannot
  see, and exactly what characteristic sets capture.
- **The hook**: when ≥2 patterns share a subject variable with bound predicates
  (a star with predicate set `Q`), the CS estimate
  `Σ_{C ⊇ Q} count(C) · Π_{p∈Q} avg_mult(C, p)` (Neumann & Moerkotte) replaces the
  independence product — in the seed choice, in the per-candidate `out` score, and as
  the subject-variable ndv (`Σ_{C ⊇ Q} count(C)`). All inputs are in
  `CharacteristicSet { predicates, subjects, predicate_triples }` (avg_mult =
  `predicate_triples[i] / subjects`), keyed by dict ids before string resolution.
- **Gap (public-API)**: `sparq-engine` exposes no seam to inject an external
  cardinality model — `eval_bgp_binary` and `pattern_var_ndv` are private and call
  `store.estimate`/`store.pred_stat` directly. Wiring CS in needs either a
  `cs-planner` cargo feature in sparq-engine holding an optional CS table (built by
  this crate or at index build), or a public estimator trait on the engine.
  Per the zero-impact invariant this crate does NOT modify the engine; recording the
  hook here instead.
- **Gate when wired** (design doc §4): CS-based estimates beat the `PredStat`
  estimator on q-error over a WatDiv star-query workload.

## Public-API gaps hit during implementation (sparq-core)

- ~~**No public dictionary iterator**~~ **DONE (engine-seams wave)**: `Dict::iter()`
  yields `(Id, TermParts)` over exactly `1..=len()` — the dense-id contract is now
  official API. This crate's hand-rolled `1..=dict.len()` loops still work and can
  be migrated at leisure.
- ~~**Internal CS ids**~~ **DONE (engine-seams wave)**: `characteristic_set_ids()`
  exposes the pre-resolution table in dict-id space (see the planner-tie-in status
  above); the LLM-facing build is unchanged.
- `store.pred_stat` already holds `count / ndv_subj / ndv_obj`; this crate recomputes
  them in its own scans (storage-mode-proof, and the same pass also needs domain
  histograms) — fine, but if a sidecar/persisted introspection lands, dedupe.

## Feature follow-ups (from research/genai-ontology-introspection.md, not in v1 scope)

- VoID + VoID-ext export (`to_void()`), SHACL shape export with support/confidence
  (a CS ≈ a node shape).
- Retrieval-mode summary `schema_summary_for(seeds, budget)` — only the schema around
  given entities (the 10k-property-KG path).
- Cross-class join hints `(C, p, D)` with counts (predicate co-occurrence is captured;
  the C—p→D edge table is not yet).
- ABSTAT-style pattern minimalization via the class hierarchy (`rdfs:subClassOf`),
  e.g. olympics' `dbo:SportsEvent rdfs:subClassOf dbo:Sport` chains.
- Persisted sidecar (`*.introspect`) for O(output) summaries without rescanning;
  per-class sample *labels* (current samples are global per predicate, which can look
  odd on minority classes — e.g. a Person label sample shown under dbo:SportsTeam's
  rdfs:label row).
- WASM smoke test + bundle-size measurement (all operations are scans, no syscalls —
  expected trivial, unverified).
