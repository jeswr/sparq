# RFC: Splitting `sparq-engine` — Phase-0 decision package (sq-6vshe.3) [FABLE-5]

> **Status: ADJUDICATED (2026-07-05, sq-6vshe.4) — program closed at two seams.**
> The direction was maintainer-ratified via steer #1402; Phases A1
> (`sparq-engine-serialize`, PR #1542) and A2 (`sparq-engine-service`, PR #1563) are
> MERGED. Phase A3 is **WITHDRAWN** — its §4 premise turned out to be false against the
> code (see the CORRECTION block in §4). Option B is **NOT EXECUTED** — the measured
> §6-D2 critical-path verdict came in under the ratified threshold, tripping this RFC's
> own veto condition. Architect ruling + Option-B reopening conditions in §10.
>
> Bead: **sq-6vshe.3** · Program: **sq-6vshe** / #1396 · Perf review: #1397 ·
> Hard constraint: **#1303** (no-dyn perf-neutrality gate) · Deferred measurement:
> **sq-6vshe.12** (cargo-llvm-lines / `--timings` attribution).
>
> Author: Claude Fable 5 [FABLE-5], from a structural prose digest of
> `crates/sparq-engine` (module inventory, reference-count dependency graph, cfg-site
> census, generic-fn census). Raw timings were deliberately **not** collected here — see
> §2 for exactly what this RFC does and does not know.

---

## 1. Motivation: why consider splitting at all

`sparq-engine` is the largest crate in the workspace and sits mid-graph on every
critical path (`sparq-core` → `sparq-parse`/`sparq-substrate` → **engine** → server →
solid/…). That position makes one big crate the worst case for every CI mechanism the
sq-6vshe program relies on, simultaneously (#1396 §3.1):

- **Caching** — cache granularity is crate granularity; any engine change invalidates
  engine plus its entire downstream closure in every job.
- **Change-based selection (sq-fmx4u)** — the affected-set is computed per crate;
  engine's reverse closure is approximately the workspace, so the *modal* heavy PR (one
  that touches engine) gets almost no selection benefit. Sub-crates make the sub-crate
  the selection unit.
- **Compile pipelining** — cargo parallelizes across crates; within a crate the frontend
  is essentially serial. Engine is the long serial pole; sibling sub-crates turn the pole
  into a diamond (only if the seams are siblings, not a chain — see §4).
- **Coverage** — the ratchet is per-crate, so the engine's instrumented test binary is
  one indivisible shard, currently the longest pole in the coverage matrix. The
  benchmark/CI catalog (and the sq-piapk sharding work it motivated) is the source of
  truth for the actual shard wall-times; this RFC deliberately quotes none, per repo
  hygiene. Decomposition removes the bottleneck *structurally* where sq-piapk shards it
  tactically.
- **Runtime/perf work** — the engine performance review (#1397) identifies the same
  crate as the runtime hotspot and the monomorphization-diet target (sq-6vshe.12);
  a crate structure that isolates the hot core from the periphery narrows the surface
  that perf work and its neutrality gates must reason about.
- **Dev/agent iteration** — `exec.rs` alone is ~12k physical LOC (~8.3k non-test) in one
  file; any touch re-typechecks and partially re-codegens the whole unit. With an agent
  fleet compiling all day, this recurring blast radius plausibly rivals the CI bill.

The honest counter-arguments (#1396 §3.2) are equally real: monomorphization can gut the
win (if codegen already happens downstream at instantiation sites, a split moves less
than hoped), cross-crate inlining on hot paths is a proof obligation against #1303, and
3–5 new crates carry one-time bookkeeping costs (coverage floors, README gates, feature
forwarding). That is why this is a decision package and not a plan of record.

## 2. Evidence base — and its limits

**What was measured (structure only):** a full module inventory of
`crates/sparq-engine/src` (23 top-level modules, ~35k physical LOC, large in-file
`#[cfg(test)]` fractions noted per module); a `crate::`-reference-count dependency graph
between modules; the cfg-site census for feature-gated code woven into the core
(31 `zk` + 9 `service` sites inside `exec.rs`); and a generic-fn census (only two
generic fns in `exec.rs`, so intra-engine monomorphization fan-out is LOW — the heavy
generic weight already lives in the extracted `sparq-substrate` join kernels).

**What was NOT measured — deliberately, and deferred:**

| Missing evidence | Where it lands |
|---|---|
| `cargo build --timings` frontend-vs-codegen attribution + pipelining critical path | sq-6vshe.12 (+ a timings run on the canonical CI runner class) |
| `cargo llvm-lines` per-symbol codegen attribution | sq-6vshe.12 (explicitly feeds this RFC's §6-D2) |
| git-churn heat map (which modules PRs actually touch) | pre-GO obligation for Option B (§6-D2), cheap to gather |
| per-seam coverage-*time* attribution of the engine shard | pre-GO obligation, from the CI catalog once seams are named |

Consequence: the **structural** half of the §3.4 decision rule is answerable now; the
**timings** half is not. The recommendation in §9 is shaped accordingly — no number in
this document should be read as a measured build- or run-time claim.

## 3. The measured seam map

### 3.1 Module inventory (structural sizes as digested, 2026-07)

| Module | LOC (real / in-file test) | Feature | Role |
|---|---|---|---|
| `exec.rs` | ~8.3k / ~3.7k | always-on | **The hub.** BGP planning AND execution in one unit: budget/trace/functions/aggregates/multiplicity/spatial/view/service_transport thread-local mods; `Row`/`Key` hot-path types; `eval_select`/`eval_ask`/`count_select`/`eval_select_json`; the whole planner (`split_sargable`, `prepare_bgp`/`Prepared`, `bgp_estimate`, `goo_*`, `CsCtx`, `wcoj_global_order`) |
| `lib.rs` | ~0.9k / ~1.9k | always-on | Public facade: `QueryBudget`, `FunctionRegistry`, `DatasetView`, `PreparedQuery/Update`, `QueryResult`, free fns `query/ask/count/query_json` (+ `_with_budget/_prepared/_view`), re-exports |
| `serialize.rs` + `serialize/{compact,frame}.rs` | ~2.1k+2.6k / ~2.9k | `serialize-rdf`, `streaming-serialization` | RDF writer matrix (Turtle/TriG/N-Quads/JSON-LD, buffered + streaming) |
| `update.rs` | ~1.1k / — | always-on | SPARQL 1.1 Update executor |
| `service.rs` | ~1.25k / — | `service` (not wasm32) | Federation: blocking HTTP (ureq), results JSON/XML parse, bound-join batching, egress policy |
| `window_syntax.rs` + `window.rs` | ~2.9k total | `window-functions` | SQL-style window functions + OVER front end |
| `construct.rs` | ~0.5k | always-on | CONSTRUCT/DESCRIBE templates (needs only `eval_select` + budget + `QueryResult`) |
| `dp.rs` | ~0.7k | `dp-planner` | DPccp bushy-join enumerator |
| `explain.rs` / `explain_json.rs` | ~0.6k each | always-on / `explain-json` | EXPLAIN (ANALYZE) text + JSON plan tree, slow-query ring |
| `chunk.rs` | ~0.6k | `vectorized` | Columnar `DataChunk` + FILTER/gather kernels |
| `zk.rs` | ~0.6k | `zk` | zk-trace recorder seam (derived-credential prover) |
| `txn.rs` | ~0.5k | `txn` | MVCC snapshot-isolation transactions |
| `json.rs` | ~0.4k | always-on | SPARQL-Results JSON writer |
| `semijoin.rs` | ~0.4k | `semijoin-bitmap` (← `yannakakis`) | KeyFilter semi-join reducer |
| `cache.rs` | ~0.4k | `result-cache` | Version-aware LRU result cache |
| `aggregate.rs` | ~0.35k | always-on | Custom aggregate registry (`AggFn`) |
| `solution.rs` | ~0.35k | `query-solution` | Oxigraph-shaped `QuerySolution` view |
| `cs.rs` + `cs_gate.rs` | ~0.6k total | `cs-planner` | Characteristic-set star-join cardinality + q-error gate |
| `params.rs` | ~1.0k | `params` | Parameterized prepared-query binding (anti-injection rewrite) |
| `dataset.rs` | ~0.15k | always-on | Dataset/graph build + decode helpers |

Sibling deps already behind clean seams: `sparq-core` (Graph, dict/Id, `store::Pattern`,
temporal) and `sparq-substrate` (numeric tower, join kernels, `compare_terms`).

### 3.2 The shape: a bidirectional hub

The graph is hub-and-spoke centered on `exec.rs`, with `lib.rs` as a thin facade above
it. `exec` is both **the thing everyone calls** (`construct`, `update`, `explain*`,
`cs_gate`, `dp`, `cs`, `semijoin`, `chunk`, `dataset` all reference it) and **the thing
that calls the optional extensions** (it reaches into `zk` [45 refs / 31 cfg sites],
`service` [23 / 9], `dp` [9], `json` [8], `semijoin` [5], `cs` [4], `chunk` [1]). The
operational/serving modules (`txn`, `cache`, `params`, `window_syntax`, `solution`)
layer on the **facade** (`query*/ask*/count*/update*`), not on `exec` directly, and
`serialize` needs only `triples_to_ntriples` + `QueryResult`. `service` is nearly
standalone and already crosses via the one existing `Box<dyn Transport>` install point
(per SERVICE call — off the per-row path).

### 3.3 Three natural boundaries (cleanest first)

- **Seam 1 — peripheral I/O & serving shell.** Writers (`serialize*`, `json`,
  `construct`), federation (`service`), and the serving/operational wrappers (`cache`,
  `txn`, `params`, `solution`, `explain*`, `window*`). Maps almost 1:1 onto the existing
  14 opt-in feature flags — the Cargo feature list already encodes this cut.
- **Seam 2 — planner vs executor.** Planner: `split_sargable`, cyclicity test,
  `prepare_bgp`/`Prepared`, `bgp_estimate`, `goo_*`, `CsCtx`, `collect_vars`,
  `wcoj_global_order`, plus `dp.rs` and `cs.rs`. Executor: `eval_*`, scan→bindings, join
  drivers, property paths, FILTER/BIND/aggregate eval, `semijoin`, `chunk`. Handoff is a
  join-order plan over `Prepared` — conceptually clean, **structurally leaky today**
  (shared `Prepared`/`Row`/`CsCtx` + thread-local budget/trace state, all in one file).
- **Seam 3 — eval substrate: ALREADY DONE.** `sparq-substrate` (numeric tower, JoinKeys
  join-kernel family, `compare_terms`) is the existence proof that a hot-path-heavy seam
  can be split behind concrete generics with **zero dyn**, consumed unconditionally by
  `exec`. It is the template any Seam-2 cut must copy.

### 3.4 The five couplings that resist splitting

1. **Thread-local scoped state** (28 install/`with_` sites across budget, trace,
   functions, aggregates, multiplicity, spatial, view, service_transport), read on hot
   paths (per-scan budget checks, the zk bool). Works across crates, but entangles init
   order — and if "cleaned up" into an `&dyn Context` it violates #1303. Any split must
   mandate a **concrete shared `Ctx`** in a base crate, never a trait object.
2. **Shared hot-path row type** `Row = SmallVec<[Id;4]>` (and `Key = SmallVec<[Id;2]>`)
   threaded through scan→join→project. Must remain a single concrete type in a base
   crate; boxing rows or trait-object iterators is a #1303 violation. The safe pattern is
   the substrate one (generics over concrete key types, monomorphized).
3. **`exec`↔`dp`↔`cs` near-cycle.** `dp`/`cs` build on `exec`'s `Prepared`/`CsCtx`/
   `goo_*` while `exec` calls into them. A planner crate needs `Prepared`/`CsCtx` to move
   *with* the planner (or into a shared base) or the extraction creates a crate cycle.
   This is the main structural blocker for Seam 2.
4. **Feature gates woven into the hot code.** The 31 `zk` + 9 `service` cfg sites are
   interleaved into `eval_select` itself; zk's zero-cost-when-off contract (one
   thread-local bool per scan when on, byte-identical when off) depends on those hooks
   staying in the executor crate with a concrete (non-dyn) recorder. Chained features
   (`yannakakis`→`semijoin-bitmap`, `streaming-serialization`→`serialize-rdf`) must be
   re-expressed by the forwarding facade.
5. **Hidden statistics coupling.** The planner reads dictionary NDV and per-pattern scan
   cardinalities straight off `sparq-core`'s Graph; a standalone planner needs an
   explicit `Statistics` interface — implementable with concrete generics, no dyn.

Reassuringly, monomorphization fan-out *inside* the engine is LOW (two generic fns in
`exec.rs`, both feature-gated or setup-time), so a split does not obviously multiply
generic instantiation across a new boundary — but "obviously" is exactly what
sq-6vshe.12's llvm-lines audit must confirm before anyone quotes it as fact.

## 4. The options

A note on the obvious third axis: a **storage** seam is *not* on the menu — storage
already lives in `sparq-core` behind a clean, consumed-today boundary, and the
update/dataset glue (`update.rs`, `dataset.rs`) is small and always-on; moving it buys
nothing structural.

### Option A — Periphery peel (feature-aligned sub-crates; hot core untouched)

**What moves.** Three peels, one PR each, cleanest first:

| New crate | Modules | Features forwarded |
|---|---|---|
| `sparq-engine-serialize` | `serialize.rs`, `serialize/compact.rs`, `serialize/frame.rs` (optionally `json.rs`, `construct.rs`) | `serialize-rdf`, `streaming-serialization` |
| `sparq-engine-service` | `service.rs` | `service` (keeps `cfg(not(wasm32))`) |
| `sparq-engine-serve` **(WITHDRAWN — see the CORRECTION below)** | `cache.rs`, `txn.rs`, `params.rs`, `solution.rs`, `explain_json.rs`, `window.rs`, `window_syntax.rs` | `result-cache`, `txn`, `params`, `query-solution`, `explain-json`, `window-functions` |

Left behind: the core engine (`exec` + planner + `update` + `dataset` + `aggregate` +
`explain` text + `lib` facade) plus the woven-in `zk` hooks.

**API across the seam.** Exactly the existing public facade: `QueryResult`,
`query*/ask*/count*/update*`, `QueryBudget`, `PreparedQuery/Update`, `DatasetView`,
`FunctionRegistry`, `triples_to_ntriples` — plus **one real API change**: the
`pub(crate)` scoped-context helpers (`with_view`/`active_dataset`/`view_scope`/
`with_functions` install/snapshot guards) must be promoted to a public, still-concrete,
non-dyn scoped-context API so the peeled crates can set up execution. `service` keeps
its existing `Box<dyn Transport>` install point (already dyn, per-SERVICE-call,
off-row-path — allowlisted).

> **CORRECTION (2026-07-05, sq-6vshe.4 ruling) [FABLE-5] — the A3/serve claim above is
> FALSE against the code.** A1 and A2 moved genuine *leaves* — code the executor calls
> into (`serialize` crossed only via `triples_to_ntriples` + `QueryResult`; `service.rs`
> had zero `crate::` back-refs). The seven serve modules are the opposite: upward
> consumers of the core facade, and the scoped-context promotion alone cannot free them.
> Re-derived per-module on `origin/main` (post-A1/A2):
>
> - **`lib.rs` ⇄ `params.rs` is a bidirectional cycle**, not just an upward dep:
>   `PreparedQuery::bind` / `PreparedUpdate::bind` in `lib.rs` call `params::bind_query`
>   / `params::bind_update`, while `params.rs` calls `crate::ask`.
> - **The promotion targets are co-consumed by staying modules.** `view_scope` /
>   `active_dataset` are used by `construct.rs`, `explain.rs`, and `lib.rs`'s own query
>   fns as well as by the moving candidates (`cache.rs`, `explain_json.rs`); promoting
>   them `pub` does not let them move, so the consumers still depend on core.
> - **Every serve module needs at least one `lib.rs`-defined item** (`QueryResult`,
>   `QueryBudget`, or the facade fns `query`/`ask`/`count`), and the verbatim-facade
>   requirement forces `sparq-engine` to re-export the moved modules (`lib.rs`
>   `pub use`s `cache`/`solution`/`explain_json`/`window_syntax`). A serve crate that
>   depends on `sparq-engine` while `sparq-engine` re-exports it is a forbidden Cargo
>   crate cycle.
> - **No leaf subset exists among the seven.** `explain_json.rs` is among the *most*
>   core-coupled (`exec`, `explain`, `explain_analyze`, `view_scope`, `active_dataset`,
>   `QueryBudget`) — the opposite of leafy. The leafiest members are `window.rs` and
>   `solution.rs`, each needing exactly one core item (`QueryResult`, defined in
>   `lib.rs`) — but freeing even them requires hoisting a core public type into a new
>   lower crate (the Option-B base-hoist mechanism in miniature), and `window_syntax.rs`
>   (which needs the core `query`/`query_with_budget`) could not follow `window.rs`,
>   fracturing the `window-functions` feature across two crates.
>
> The only mechanism that breaks the cycle is the Option-B `sparq-engine-base` hoist —
> and the §6-D2 measurement subsequently scored that hoist's payoff under threshold.
> A3 is therefore **WITHDRAWN**, not re-scoped (ruling in §10).

**Trade-offs.** (+) Mostly mechanical: the cut coincides with the feature flags, so
"byte-identical when off" contracts carry over. (+) Coverage: `serialize`'s ~5k LOC
(real+tests) and the serving shell become separately shardable crates on the existing
matrix. (+) Selection/caching: serialization- or federation-only PRs stop invalidating
the engine. (+) Zero hot-path exposure — #1303 risk is confined to the scoped-context
promotion, which stays concrete. (−) Does **not** break `exec.rs`, so the largest
incremental-rebuild blast radius and the serial frontend pole remain. (−) The peeled
modules are feature-gated, so *default-build* compile cost barely moves; the win is in
feature-on builds (coverage, all-features legs, server) and in shard/selection
granularity. (−) Three new crates' worth of coverage floors, README-template gates, and
release-plz bookkeeping.

**Coupling risks to overcome:** #1 (scoped-state promotion — concrete API, init-order
documented), #4 (feature-implication graph re-expressed in the facade), and the
`construct`/`json` placement question (always-on modules; moving them makes the new
crate a *chain* link for the server rather than a sibling — default: leave them in core,
move only the gated writers).

### Option B — Planner | executor split over a shared base crate (the deep cut)

**What moves.**

| New crate | Contents |
|---|---|
| `sparq-engine-base` | `Row`/`Key`, `Prepared`, `CsCtx`, sargable descriptors (`ScanCmp`/`CmpOp`/`NumCmp`), the concrete scoped `Ctx` (budget/trace/functions/aggregates/multiplicity/spatial/view), a generic non-dyn `Statistics` trait |
| `sparq-engine-plan` | `split_sargable`, cyclicity test, `prepare_bgp`, `bgp_estimate`, `goo_seed/goo_seed_sort/goo_pick`, `record_pattern_ndv`, `collect_vars`, `wcoj_global_order`, `dp.rs`, `cs.rs` |
| `sparq-engine` (core) | `eval_*`, scan→bindings, join drivers, property paths, FILTER/BIND/aggregate eval, `semijoin`, `chunk`, `update`, `construct`, `json`, the zk/service hook sites, the facade |

**API across the seam.** Planner consumes `Statistics` (generic over `&Graph`,
monomorphized — no dyn) and produces a join-order plan (essentially an ordering over
`Prepared`) plus the f64 cardinality estimate; executor consumes `Prepared` + plan +
`Ctx` from base. All three share the concrete `Row`.

**Trade-offs.** (+) The only option that breaks the ~8.3k-real-LOC `exec.rs` unit, which
is where library-only compile cost is most concentrated — this is the dominant predicted
compile/iteration win, *and it is exactly the prediction that sq-6vshe.12 must confirm
or kill* (if codegen dominates frontend, or downstream instantiation dominates codegen,
the split under-delivers). (+) Gives perf work (#1397) a planner it can test/replace in
isolation; DPccp and CS become planner-crate citizens instead of hub satellites. (+)
Siblings, not a chain: plan and exec can compile in parallel above base. (−) Highest
refactor cost and churn against in-flight engine PRs; requires the base-crate hoist
first. (−) #1303 exposure is real: the seam crosses per-query (not per-row) boundaries,
but every shared type must stay concrete and hot calls monomorphized/inlined
(thin-LTO + the perf-neutrality gate are the proof obligation, per #1396 §3.2). (−) The
zk/service cfg-woven hooks stay in the executor; the recorder install must remain a
concrete thread-local so the off-build stays byte-identical.

**Coupling risks to overcome:** all five of §3.4 — #3 is solved by construction
(`Prepared`/`CsCtx` move to base), #1/#2 by the concrete-`Ctx`/`Row` mandate, #5 by the
`Statistics` trait, #4 by keeping hook sites executor-side.

### Option C — In-place modularization of `exec.rs` (no new crates)

Split the single file into intra-crate modules (`plan/`, `eval/`, `ctx/`) and hoist the
shared types into an internal `base` module — the null-split alternative.

**Trade-offs.** (+) Zero crate-count overhead, zero facade work, zero #1303 exposure,
large navigability/review win, and it *is* the mandatory first refactor step of Option B
(so it is never wasted). (−) rustc's compilation unit is the crate: coverage shards,
selection granularity, cache invalidation, and pipelining are all unchanged — Option C
alone delivers **none** of the CI-structural payoffs in §1. It is a de-risking precursor,
not a destination.

## 5. Facade + feature-forwarding plan (applies to A and B)

Per the bead and #1396 §3.2: `sparq-engine` **remains the published crate** as a
`pub use` facade. External users and the feature-matrix keys see an unchanged surface:

- Every existing feature name stays on `sparq-engine` and forwards
  (`serialize-rdf = ["dep:sparq-engine-serialize", "sparq-engine-serialize/serialize-rdf"]`
  shape), preserving the implication graph (`yannakakis`→`semijoin-bitmap`,
  `streaming-serialization`→`serialize-rdf`).
- Verified mechanically: `cargo public-api` diff = no breaking change; the existing
  feature-matrix CI legs keyed on `sparq-engine` features stay green unmodified; the
  feature-OFF byte-identical/perf-neutrality proof gates (#1303 family, cf. PR #1386's
  lane) run against the facade exactly as today.
- New sub-crates are **internal by default** (documented as unstable-internal in their
  READMEs) so the public commitment surface does not widen.

## 6. GO/NO-GO decision framework (testable)

Framed against the program's decision rule (#1396 §3.4 — cited as §3.5 by the beads).
Thresholds below are **proposed targets for ratification**, not measurements.

| # | Criterion | Test | GO condition |
|---|---|---|---|
| D1 | **Structural seam quality** (§3.4-i) | Interface-width audit of this RFC (§3.3/§4): count of cross-seam `pub` items per seam; no orphan-rule tangles | ≥2 seams with a narrow interface. **Assessed now: PASS on structure** — Seam 1 crosses only the existing facade (+1 promoted scoped-context API); Seam 2 crosses `Prepared`+plan+`Ctx`+`Statistics`, narrow *after* the base-crate hoist |
| D2 | **Critical-path cut** (§3.4-ii) | `cargo build --timings` model on the canonical CI runner class + `cargo llvm-lines` attribution (sq-6vshe.12) + git-churn map, before/after prediction for the modal engine-touching PR | Material predicted reduction (order 25%+, per §3.4-ii). **Monomorphization sub-axis: ASSESSED — does NOT veto B** (sq-6vshe.12, indicative workbox `cargo llvm-lines -p sparq-engine --release`; numbers in the bead per §8). The NO-GO trigger ("codegen dominated by downstream/exported-generic instantiation") is **not** met: `sparq-engine` exports ~zero generic surface (the `lib.rs` facade fns are concrete; `exec.rs` defines two generic fns) so downstream crates do not monomorphize engine code, and engine's own IR is dominated by large *non-generic* function bodies — engine-defined monomorphization is a low-single-digit % of engine IR, and the substrate `JoinKeys`/`compare_terms` weight is concrete/single-instantiation, not a multiplier. So a planner\|executor cut divides genuine non-generic frontend work rather than merely relocating instantiation. **Caveats:** the majority-of-IR library-generic instantiations (iterators/`Vec`/`SmallVec`/hashbrown/rayon) re-instantiate per sub-crate and dedup only under fat-LTO, so the codegen half is partial; Seam B is a chain (executor depends on the plan), buying little cross-crate parallelism. **Timings sub-axis: MEASURED — FAIL against the threshold** (sq-aqr2f, indicative work-box `cargo build --timings`; numbers stay in the bead per §8/repo hygiene). In today's cold-cache CI, engine's own compile is a small minority of the full-build wall and is *off* the dependency critical path (the GPU stack `naga`→`wgpu-hal`→`wgpu-core`→`sparq-gpu` is the cold pole), so any engine split buys ~nothing cold. In the warm/cached regime (the modal engine-touching rebuild) engine *is* the single largest pole, but (i) the dependent tail — nearly as large — is untouchable by any internal split, since downstream rebuilds via the facade on any engine-half change; (ii) Seam B is a chain, so a full-engine rebuild is not shortened, only a one-half-touched rebuild is; (iii) the modal engine PR touches `exec.rs` = the *larger* executor half, so the split typically skips only the smaller planner half (`dp`/`cs` are even feature-gated off default). The realizable modal-PR saving falls well under the order-25% GO condition. **D2 = NO-GO for Option B in the current regime**; reopening conditions in §10 |
| D3 | **Coverage-shard rebalance** | Coverage matrix wall-times from the CI catalog before/after (per sq-piapk's instrumentation); new crates onboarded to the per-crate ratchet | Longest engine-family shard drops materially below today's single-crate shard on the existing 4-way matrix; every new crate meets a floor set from its *measured* post-split coverage (no fabricated floors; one direct unit test per new/promoted public fn) |
| D4 | **#1303 no-dyn HARD gate** | (a) perf-neutrality/feature-OFF proof lanes stay green; (b) seam-API dyn audit (ast-grep for `dyn` in cross-crate signatures; allowlist = the pre-existing `Box<dyn Transport>` only); (c) EC2 runtime benches non-regressing per the benchmark catalog | Zero new `dyn` on any per-row/per-scan path; byte-identical feature-off contract preserved. **Any violation is an unconditional VETO regardless of build wins** |
| D5 | **API + semantics neutrality** | `cargo public-api` diff on `sparq-engine`; full conformance + differential suites (dp/semijoin/yannakakis/vectorized/mvcc/dict) unchanged; wasm32 build (service excluded) green | No breaking public-API change; zero test-behavior drift |

**Veto conditions** (any one stops the corresponding option): hot-path dyn required to
break the `exec`↔`dp`↔`cs` cycle; D2 below threshold in the measured model; coverage
floors only satisfiable by lowering them; wasm or MSRV regressions from crate
re-plumbing.

## 7. Migration sketch (phased, each phase independently revertible)

Every phase is one PR behind the unchanged facade, so **revert = re-inline**; no
downstream crate edits at any step. One seam per PR, least-entangled first, with short
coordinated freeze windows on `exec.rs`-touching work (#1396 §3.2 churn note).

- **Phase 0 (this RFC + data):** maintainer steer on §9; run sq-6vshe.12
  (llvm-lines + `--timings` attribution) and the cheap churn map; re-score D2.
  *Status: DONE — steer ratified (#1402); llvm-lines half (sq-6vshe.12: monomorphization
  does not veto B) and `--timings` half (sq-aqr2f: critical-path share measured) both
  landed; D2 re-scored in §6 as NO-GO. The churn map was never gathered — it survives as
  a reopening obligation (§10), not a debt of this closed program.*
- **Phase A1:** peel `sparq-engine-serialize` (cleanest: crosses via
  `triples_to_ntriples` + `QueryResult` only). Gate: D3/D4/D5 checks green.
  *Status: DONE — merged as PR #1542.*
- **Phase A2:** peel `sparq-engine-service` (nearly standalone; keeps the dyn-Transport
  allowlist entry). *Status: DONE — merged as PR #1563.*
- **Phase A3:** promote the scoped-context API (concrete, documented init-order), then
  peel `sparq-engine-serve`. This is the step with the real API decision in it.
  *Status: WITHDRAWN — the premise is false against the code (§4 CORRECTION): the serve
  modules are upward consumers with a `lib.rs`⇄`params` cycle, and no leaf subset
  clears the A1/A2 bar. Executing it would smuggle in the Option-B base hoist that D2
  scored under threshold.*
- **Phase B0 (in-crate, = Option C):** split `exec.rs` into `plan/`/`eval/`/`ctx/`
  modules; hoist `Row`/`Prepared`/`CsCtx`/`Ctx` into an internal base module. No crate
  boundary yet — pure de-risking, valuable regardless of the B verdict.
  *Status: NOT EXECUTED — with B1/B2 closed its de-risking rationale lapses; per §8 it
  is developer ergonomics only, so it is deliberately not beaded under this program
  (revisit on its own merits if ergonomics ever warrant it).*
- **Phase B1 (GO on D2 only):** extract `sparq-engine-base`, introduce the generic
  `Statistics` trait, verify zero perf drift. *Status: NOT EXECUTED — D2 measured under
  threshold (§6); reopening conditions in §10.*
- **Phase B2 (GO on D2 only):** extract `sparq-engine-plan` (with `dp`/`cs`), executor
  stays in core. Final D1–D5 re-check; retire sq-piapk machinery if D3 makes it moot
  (coordinate — piapk explicitly waits on this profile). *Status: NOT EXECUTED — same
  gate as B1. Consequence: the sq-piapk tactical coverage-sharding machinery stays
  necessary; the structural retirement this phase promised does not happen.*

**Risks & unknowns:** thread-local init order across crates (mitigation: single
`Ctx` install API owned by base, asserted in debug builds); the zk zero-cost-when-off
contract across the hoist (mitigation: byte-identical off-build check already in CI);
compile-cost attribution genuinely unknown until sq-6vshe.12 (this RFC's central
epistemic gap); new-crate gate onboarding (README-template hard gate, coverage floors,
the feature-gated intra-doc-link rustdoc trap — use code spans); extraction-PR conflicts
with in-flight engine work; release-plz/publish surface for internal crates.

## 8. Predicted effects (directional only — numbers land in sq-6vshe.12)

- **Option A:** coverage-shard granularity + selection/caching wins are *structural
  certainties* (crate-granularity mechanisms); compile-time wins mostly in feature-on
  builds. Default-build critical path ~unchanged.
- **Option B:** the only candidate for a material modal-engine-PR critical-path cut,
  because it is the only one that divides the `exec.rs` frontend pole — contingent on
  D2, which can honestly return NO-GO.
- **Option C:** developer-ergonomics only; no CI-structural effect.

## 9. Recommendation (for ratification, not a decision)

*(Historical — adjudicated. The recommendation below was followed: Option A was
ratified via steer #1402 and executed to two seams; Option B was held on data and is
now closed on the measured data. The binding outcome is §10.)*

**Recommend: ratify Option A now; hold Option B at "gather sq-6vshe.12 data first";
fold Option C in as Option B's Phase B0 regardless.**

Rationale: Option A's case rests entirely on structural facts this RFC *does* establish
(the cut coincides with the existing feature flags; the crossings are the existing
public facade; the mechanisms it improves — coverage sharding, selection, cache
granularity — operate at crate granularity by construction), it is reversible per-PR
behind the unchanged facade, and its #1303 exposure is confined to one concrete API
promotion. Option B is the bigger prize but its justification is precisely the
measurement this RFC does not have; committing to it now would violate the program's own
decision rule (§3.4-ii), and sq-6vshe.12 is already scoped to produce the missing
number. An honest NO-GO on B — with A landed and C folded in — is a perfectly good
outcome for the program.

Decision requested from the maintainer:
1. **Ratify / amend / reject Option A** (Phases A1–A3) and the scoped-context API
   promotion it entails.
2. **Confirm the D1–D5 framework** (esp. the D2 threshold and the D4 dyn-allowlist) as
   the binding go/no-go rule for Option B.
3. **Sequence sq-6vshe.12** ahead of any Option B work; sq-6vshe.4 remains gated on the
   resulting D2 verdict.

## 10. Adjudication (2026-07-05) — the sq-6vshe.4 architect ruling [FABLE-5]

Inputs: the A3 structural stop-condition finding (sq-6vshe.4 comments, re-derived
against the code for this ruling), the D2 measurements (sq-6vshe.12 llvm-lines;
sq-aqr2f `cargo build --timings` — numbers live in the beads per repo hygiene; both
indicative work-box, non-canonical), and the delivered A1/A2 state (PRs #1542, #1563).
Issued under the proceed-and-document rule; a post-hoc steer issue was opened against
the #1402 ratification, since this ruling withdraws A3 from the ratified plan.

**Ruling.**

1. **Option B (planner|executor over `sparq-engine-base`): DO NOT EXECUTE.** The
   monomorphization axis cleared (it does not veto), but the measured critical-path
   axis came in under the §6-D2 GO condition, and "D2 below threshold in the measured
   model" is one of this RFC's ratified veto conditions (§6). What survives of B's case
   is a modest incremental-granularity + change-based-selection lever whose payoff is
   contingent on infrastructure that does not exist yet (see reopening conditions) —
   not enough to pay B's refactor cost, churn, and #1303 proof obligations.
2. **Phase A3 (`sparq-engine-serve`): WITHDRAWN.** The §4 premise is false against the
   code (§4 CORRECTION). No subset of the seven serve modules clears the bar A1/A2
   cleared — a verbatim leaf move with no core-type hoist and real feature-on codegen
   removed. Even the leafiest candidates (`window.rs`, `solution.rs`) require hoisting
   `QueryResult` out of `lib.rs` — the Option-B base-hoist mechanism in miniature, for
   strictly less payoff than the B case D2 just scored under threshold; the serve
   modules also sit off the warm-regime executor pole, so peeling them cannot shrink
   the pole that dominates the modal engine rebuild.
3. **The peel phase closes at two seams as delivered value.** A1 + A2 achieved the
   program's periphery goals: serialization-only and federation-only changes no longer
   invalidate `sparq-engine` (per-crate cache granularity + ci-select skipping), and
   the two heaviest feature-gated codegen bodies left the feature-on engine build.
   Stopping here is the honest reading of the evidence, not a concession.
4. **Phase B0 / Option C: not program work.** Its stated value was de-risking B; with B
   closed it is developer ergonomics only (§8) and is deliberately NOT beaded — this
   ruling creates nothing speculative.
5. **sq-piapk consequence:** the engine coverage shard is not structurally removed; the
   tactical sharding machinery stays necessary.

**Reopening conditions for Option B.** A regime change re-opens the question —
re-measure, do not extrapolate from the closed verdict:

- **Per-job build caching lands in CI** (the necessary precondition): the warm regime —
  the only one where engine is the dominant pole — becomes the regime CI actually runs
  in. Today's cold-cache CI sees ~nothing from any engine split.
- **A churn map shows planner-heavy engine PRs** (the still-ungathered §2 obligation):
  B only skips the untouched half, and today's modal engine PR touches the executor —
  the larger half.
- **The cold critical path collapses** — e.g. the GPU stack (`naga`/`wgpu-*`) leaves
  the workspace or the default build graph, leaving engine as the cold pole after all.
- **The dependent tail shrinks materially** — e.g. downstream crates narrow their deps
  so engine-own-compile's share of the engine-touching rebuild grows.

Procedure to reopen: after the caching precondition holds, re-run the sq-aqr2f
measurement on the then-current tree plus the churn map; Option B re-opens iff the
predicted modal-engine-PR saving clears the §6-D2 order-25% bar. The same re-measurement
would also be the moment to revisit a `QueryResult`-hoist serve peel, which shares the
base-hoist mechanism and the same economics.
