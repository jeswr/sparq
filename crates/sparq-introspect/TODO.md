# sparq-introspect — outstanding work

Tracked in beads (not here). Run `bd ready -l area:sparq-introspect` or
`bd list -l area:sparq-introspect`. See AGENTS.md for the no-markdown-TODOs policy.

## Notes

Design rationale and DONE-status records retained from the previous TODO list
(not task tracking).

### Planner tie-in: characteristic sets → cardinality estimation — DONE (engine-seams wave)

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
  max(est/true, true/est), floored at 1). The CS estimator is asserted EXACT on
  pure stars over the builder's own data; the always-on synthetic-correlated CI
  gate pins it. Re-run instructions and the measured q-error tables live in the
  gate test itself (`cargo test -p sparq-engine --release --features cs-planner`).

The original analysis (`eval_bgp_binary` / `pattern_var_ndv` hook points,
the public-API estimator-seam gap) is preserved in the engine's `cs` module
docs; it described the seam that the `cs-planner` feature now implements.

### Public-API gaps hit during implementation (sparq-core) — DONE

- ~~**No public dictionary iterator**~~ **DONE (engine-seams wave)**: `Dict::iter()`
  yields `(Id, TermParts)` over exactly `1..=len()` — the dense-id contract is now
  official API. This crate's hand-rolled `1..=dict.len()` loops still work and can
  be migrated at leisure.
- ~~**Internal CS ids**~~ **DONE (engine-seams wave)**: `characteristic_set_ids()`
  exposes the pre-resolution table in dict-id space (see above); the LLM-facing
  build is unchanged.
- `store.pred_stat` already holds `count / ndv_subj / ndv_obj`; this crate recomputes
  them in its own scans (storage-mode-proof, and the same pass also needs domain
  histograms) — fine, but if a sidecar/persisted introspection lands, dedupe.

### Feature follow-ups (from research/genai-ontology-introspection.md) — DONE records

- **VoID export (`to_void(dataset_iri)`) — DONE [OPUS-4.8] (sq-cc7).** Emits a W3C
  VoID description as N-Triples (a subset of Turtle, so it parses as either; no
  serializer dependency — oxrdf renders every term RFC-correctly). Top-level
  `void:Dataset` with `void:triples` / `void:entities` / `void:distinctSubjects` /
  `void:classes` / `void:properties` (all EXACT), plus one `void:classPartition` per
  class (`void:class` + `void:entities`) and one `void:propertyPartition` per
  predicate (`void:property` + `void:triples` + `void:distinctSubjects`). DEFERRED:
  `void:distinctObjects` (the crate tracks distinct objects only per-predicate, never
  a global de-duplicated count — a faithful figure needs an extra union pass, and the
  per-predicate `distinct_objects` mixes IRIs+literals so it is left out rather than
  emitted misleadingly); VoID-ext, `void:vocabulary`/`uriSpace`, and linkset
  partitions.
- **Retrieval-mode summary `schema_summary_for(seeds, budget)` — DONE [OPUS-4.8]
  (sq-cc7).** Seed-scoped digest for the 10k-property-KG path: each seed IRI is
  matched against the mined schema (a class seed pulls its profile + the cross-class
  join edges it touches; a predicate seed pulls its global profile), only that slice
  is rendered under the char budget, unmatched seeds noted. LIMITATION (honest):
  struct-level scoping — it filters the already-mined class/predicate profiles by IRI
  and does NOT re-scan, so it cannot chase the *instances* of a seed entity (the crate
  retains class/predicate profiles, not per-subject adjacency). The general
  dataset-shape summary remains `to_text_summary(budget)`.
- **Cross-class join hints `(C, p, D)` with counts — DONE [OPUS-4.8] (sq-cc7).**
  `Introspection::join_hints: JoinHints` — for each triple whose subject is typed `C`
  and whose IRI object is typed `D`, the `(C, p, D)` cell is incremented, mined in the
  SAME SPO scan as the characteristic sets (one object-type lookup per triple; only
  typed-subject→typed-object triples reach the inner product). Top edges by triple
  count, capped at `BuildOptions::max_join_hints` with exact tail aggregates. Multi-
  typed subjects/objects count under each declared type (documented).
