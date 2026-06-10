# sparq-introspect TODO

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

- **No public dictionary iterator**: vocabulary detection iterates ids `1..=dict.len()`
  and calls `term_parts(id)` — valid today (ids are dense, 1-based) but it leans on the
  id-assignment contract; a `Dict::iter()` (or `terms()`) would make it official.
- **Internal CS ids**: this crate resolves predicates/classes to IRI strings at build
  time (LLM-facing). For the planner tie-in the table should stay in dict-id space —
  keep the pre-resolution `FxHashMap<Box<[Id]>, CsAcc>` form behind a (future)
  `cs-planner`-facing accessor instead of re-parsing strings.
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
