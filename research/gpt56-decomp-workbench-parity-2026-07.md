# GPT-5.6 decomposition: workbench-parity leading beads — 2026-07

> 🤖 **SPARQ agent** (Claude Fable 5, architect stage). Design decisions for
> the de-risked, single-crate decomposition of four sq-lsp7k leading beads
> (PATHS, SHACL guard/fact-domain, facet API, autocomplete). Companion:
> `gpt56-decomp-rules-substrate-2026-07.md`. Beads carry the per-child specs.

## 1. PATHS (sq-lsp7k.3) — dedicated entry point, not a grammar patch

**Decision:** ship the Stardog-style `PATHS` query form the same way
`window_syntax.rs` ships inline `OVER(...)`: an opt-in cargo feature (`paths`)
whose syntax is reachable ONLY via dedicated entry points
(`query_paths` / `explain_paths`), leaving the vendored `spargebra` PEG and
every standard entry point untouched. `PATHS …` therefore stays a parse error
on the standard surface → zero W3C-conformance exposure (the 1229 ratchet
never sees it), and the whole feature is `sparq-engine`-single-crate.

Evaluator: the existing `eval_path`/`path_pairs` machinery computes only
(start,end) pair relations, so path materialization is a NEW module
(`src/paths.rs`) over `sparq-core` `scan_perm` — BFS levels for `SHORTEST`,
depth-bounded enumeration for `ALL` (a missing `MAX LENGTH` under `ALL` is a
loud error: the termination invariant), `CYCLIC` as start==end closure.
Staging (each differentially tested against a naive DFS reference written
inside the test): ① enumeration core with `VIA <predicate>`; ② surface parser
+ entry points + typed `Paths` EXPLAIN rendering; ③ `VIA { pattern }` edges by
materializing the pattern's designated `?from`/`?to` columns through the
existing `eval_select`.

## 2. SHACL fact-domain (sq-lsp7k.2.1) — compose `expand`, don't re-plumb

`rules::expand(data, shapes) -> Graph` (feature `shacl-af`) already returns
`data ∪ inferred-closure`. The switch is a thin composition:
`FactDomain::{Asserted, AssertedPlusInferred}` + a
`validate_with_domain(data, shapes, domain)` entry point that calls
`validate(&expand(data, shapes), shapes)` in the inferred arm. Existing
`validate*` signatures unchanged; test file follows the intentionally
un-feature-gated `shacl_af_feature_state.rs` pattern so both feature states
compile and assert.

## 3. Server SHACL guard mode (sq-lsp7k.2.4) — two decisions taken

The writer's `fork → apply → seal` seam (`ApplyUpdates` impl `ServerApplier`)
already validates-before-publish for free: the forked working `Graph` IS the
candidate post-state, and an `Err` from the seam maps to
`WriteError::Rejected` with the store untouched. All GSP writes mint SPARQL
updates through this one path, so ONE hook covers UPDATE + GSP. Two gaps need
decisions (proceed-and-document):

1. **Standing shapes designation.** `/shacl/validate` takes shapes per-request;
   the write path has no request shapes. Decision: shapes live in a
   **designated named graph in the store**, configured by
   `--shacl-guard-shapes <IRI>` / `SPARQ_SHACL_GUARD_SHAPES` (guard flag
   `--shacl-guard` / `SPARQ_SHACL_GUARD`, default OFF, the exact
   `tpf`/`shacl` double-opt-in discipline). Shapes are authored/updated via
   normal GSP on that graph; an empty/absent shapes graph ⇒ guard passes
   trivially (documented). Post-state validation excludes the shapes graph
   itself from the data domain.
2. **Report-through-rejection channel.** `WriteError::Rejected(String)` is the
   only error channel and both ends live in `sparq-server` (`ServerApplier`
   produces, `update_rejection_response` consumes). Decision: sentinel-prefixed
   payload (`"shacl-guard-violation\n" + report.to_turtle()`); the HTTP mapper
   detects the sentinel and returns **422** with the W3C ValidationReport body
   (Turtle, or the existing `shacl_report_to_json` projection under content
   negotiation). No `sparq-serve` change; reversible if a structured error
   channel lands later.

v1 validates the full post-state (RDFox semantics); delta-scoped validation is
an explicit follow-up, not smuggled in.

## 4. Facet-count API (sq-lsp7k.5) — home is `sparq-introspect`

`sparq-introspect` is already the scan-only, wasm-safe statistics crate
(class instance counts, predicate stats, characteristic sets) built purely on
`sparq-core` sorted-permutation scans — exactly the facet shape. The genuinely
new statistic is **grouped value distributions for a candidate set**.
Decision: `FacetRequest { class: Option<IRI>, constraints: Vec<(p, o)>,
facet_predicates, top_k }` → `FacetResponse` with type/predicate/value
`Counted` distributions + elision counts (the crate's existing conventions),
computed from `scan_perm` range counts (generalizing the engine's COUNT(*)
pushdown idea). Correctness oracle: a dev-dependency on `sparq-engine`
(workspace-internal, no supply-chain surface) evaluating the equivalent
`GROUP BY` aggregates. The server endpoint is a separate `sparq-server` bead
(feature + runtime flag, `tpf` discipline). Any *measured-advantage* claim is
bench-artifact work and stays architect-tier (sq-hmd7l axis).

## 5. IRI/label autocomplete (sq-lsp7k.9) — sibling index in `sparq-text`

`TextIndex` deliberately skips IRIs and has no completion shape, but its
`BTreeMap` range-scan prefix discipline, `apply_delta`/`reconcile` incremental
seams, and differential-vs-rebuild test convention are exactly right.
Decision: a **sibling `CompletionIndex`** in `sparq-text` (`src/complete.rs`)
indexing (a) every dict IRI under full-IRI + local-name keys and (b)
`rdfs:label`/`skos:prefLabel` literals keyed to their *subject entity* `Id`;
`complete(prefix, k, scores: Option<&FxHashMap<Id, f64>>)` with deterministic
(score desc, key asc, id asc) ordering. Ranking scores are **injected**, not
computed — `sparq-text` gains no `sparq-algos` dependency; the consumer joins
PageRank output on the shared `dict::Id`. Fuzzy matching is explicitly out of
v1 (net-new algorithmic surface; own bead later). Server exposure mirrors the
facet endpoint (feature + flag + `AppState`-held index reconciled on
staleness); PageRank wiring + GUI consumption + the 100M-triple p99 bench stay
architect-tier (cross-crate + perf claims).

## 6. Same-crate sequencing

`sparq-server/src/http.rs` is one hot file: the guard-mode, facet-endpoint,
and complete-endpoint beads are dependency-chained (guard → facets →
complete) purely to serialize merges — not a semantic ordering. Likewise the
datalog beads chain inside `sparq-reason` (see the companion record §3).
