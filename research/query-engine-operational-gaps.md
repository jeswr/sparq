<!-- [OPUS-4.8] Design-for-review research record. SPARQ agent (researcher lens: query engine, optimization & performance). -->
# Query Engine — Operational & Optimization Gaps (lens: query processing)

> SPARQ agent 🤖 research record (design-for-review). **Read-heavy, no implementation.**
> This complements — and deliberately does **not** restate — the existing
> [`research/optimization-techniques.md`](./optimization-techniques.md), which already
> owns the *execution-kernel* sweep (vectorization, semi-join reducers, cardinality
> estimators, compression-exec, WCOJ). This doc covers the **operational /
> serving-grade** half of the lens that sweep does not: parameterized queries, plan
> caching, sub-plan (intermediate-result) caching, persistent statistics, spill-to-disk
> in the *local* evaluator, local adaptive re-optimization, incremental view
> maintenance, and per-query profiling/telemetry.

## 0. Honesty framing / evidence tags

`[V]` = verified in this repo by reading the code (file:line cited). `[M]` = measured
in a cited external paper/system. `[L]` = established in literature. `[S]` = my
inference / speculation for sparq. No work-box timings are presented as canonical.
Where a capability *partially* exists I say so and scope the proposal to the genuine
delta only.

---

## 1. Ground truth — what sparq's query engine ALREADY has (verified)

Read of `crates/sparq-engine/src/{exec,explain,cache,lib}.rs`, `crates/sparq-core/src/extsort.rs`,
`crates/sparq-fedplan/src/adaptive.rs`, `crates/sparq-server/src/metrics.rs`, plus
`research/optimization-techniques.md` and `research/concurrent-serving.md`:

| Capability | Status (verified) | Where |
|---|---|---|
| `EXPLAIN` / `EXPLAIN ANALYZE` (plan + per-operator actual rows + wall time) | **Implemented** `[V]` | `explain.rs:44,64`; trace at `exec.rs:304` |
| Join ordering: GOO above a budget, DPccp/DPhyp under it | **Implemented** `[V]` | `exec.rs:4068` (`goo_seed`/`goo_pick`); per `optimization-techniques.md` |
| Characteristic-set star-join estimates (opt-in `cs-planner`) | **Implemented, caller-built, thread-local per-exec** `[V]` | `cs.rs:1` (`with_cs_table`, "rebuild the table when the graph is rebuilt") |
| Base cardinality = index-range `partition_point` counts + `PredStat` marginals | **Implemented, computed per-query** `[V]` | `exec.rs:996,1605,4232` |
| Merge / hash / gallop / **LFTJ (WCOJ)** join dispatch via GYO | **Implemented** `[V]` | per `optimization-techniques.md` §1.1 |
| Whole-query **result cache** (opt-in `result-cache`), keyed `(algebra, version)`, LRU, determinism gate | **Implemented** `[V]` | `cache.rs:1` (`ResultCache`, `is_cacheable`) |
| `PreparedQuery` (parse-once / execute-many) | **Implemented** `[V]` | `lib.rs:460` |
| Intra-operator data-parallelism (rayon `par_iter` on scan-materialise, FILTER, CONSTRUCT) behind `parallel` feature | **Implemented** `[V]` (11 sites) | `exec.rs:847,1038,4792,4969,…` |
| External-memory sort/merge **for index BUILD** + dict-spill | **Implemented** `[V]` | `extsort.rs:1`; dict-spill commits |
| Federated streaming-join **spill** (ANAPSID-style) | **Implemented (fed only)** `[V]` | `sq-vf7q`; `sparq-fedplan` |
| Federated **adaptive / mid-execution re-plan** (divergence trigger, EWMA latency) | **Implemented (fed only, opt-in `adaptive-replan`)** `[V]` | `fedplan/src/adaptive.rs:1` |
| Server aggregate Prometheus `/metrics` (request counts, latency histogram) | **Implemented** `[V]` | `sparq-server/src/metrics.rs:1` |
| Single-flight in-flight request dedup (designed) | **Designed** `[V]` | `research/concurrent-serving.md` §5 verdict 1 |

**The pattern in the gaps.** Every "operational SQL-engine-grade" capability that
exists either (a) lives **only in the federation path** (adaptive re-plan, streaming
spill) or (b) is **whole-query coarse-grained** (result cache invalidates on a global
version bump; `PreparedQuery` is parse-cache, not parameterized). The *local* in-memory
evaluator is a single-shot "plan once → fully materialise in RAM → return" machine. The
genuine gaps below are exactly the things a mature engine (DuckDB, QLever, PostgreSQL,
RDFox, Virtuoso) does in its local execution path that sparq's local path does not.

---

## 2. Candidate gaps (each: what / why / fit / novelty / effort / prior art / decision-ask)

The detailed candidate objects are in the structured return. This section is the
narrative record + the honest cross-checks.

### 2.1 Parameterized / bound prepared queries (placeholder substitution)
- **Gap (verified):** `PreparedQuery` (`lib.rs:460`) is *parse-once*, but there is **no
  parameter binding** — no `?$0` / positional placeholder, no `bind(name, term)` API,
  no escaped value injection. Callers building queries from untrusted input (the Solid
  server, the GUI REPL, NL→SPARQL output) must do string concatenation, which is the
  SPARQL-injection footgun. `[V]`
- **Why it matters:** Jena's `ParameterizedSparqlString` and RDF4J's `$var` HTTP
  binding exist precisely for this; it is the standard mitigation in the LTBQP-security
  literature (arXiv 2210.04631). It also makes a *plan cache* (2.2) trivially effective:
  the same parameterized plan is reused with different constants.
- **Fit:** strong. The GUI-as-embedded-app + Solid directions both build queries
  programmatically; a safe binding API is directly aligned. The engine already accepts
  pre-built `Query` algebra via `PreparedQuery::from(Query)` (`lib.rs:491`), so binding
  can be a pure algebra-rewrite (substitute a `Variable` → `NamedNode`/`Literal`) with
  zero new execution paths.
- **Novelty check:** **genuinely absent.** `VALUES`-injection works at the SPARQL level
  but doesn't help the host-language safety story, and `PreparedQuery` does not bind.
- **Opt-in:** can be core (tiny, no deps) **or** a thin `params` feature — recommend
  core (it is a safety primitive, not a heavy capability).

### 2.2 Plan cache (cache the *physical plan*, not just the result)
- **Gap (verified):** the result cache (`cache.rs`) stores the *answer*; on a cache miss
  the whole planning pipeline (GOO/DPccp + cardinality probing) re-runs every execution.
  `concurrent-serving.md` §5 measured that **planning cost ≥ point-query cost** (parse
  alone 2.24 µs vs 9.34 µs total on that box — non-canonical work-box figures, cited
  only as the *shape* of the cost). For a parameterized query served at high QPS with
  different constants, the result cache never hits but the *plan* is identical. `[V]`
- **Why it matters:** a per-`(query-shape)` plan cache amortises planning across all
  bindings of a parameterized query — the classic prepared-statement win. Distinct from
  the result cache (which keys on constants + version).
- **Fit:** strong; composes with 2.1. Key on the algebra with constants abstracted to
  placeholders. Pure win for the server + GUI persistent-workspace (same saved queries
  re-run).
- **Novelty check:** **absent for local.** QLever's planner is cache-aware but at the
  *sub-result* level (2.3), not a reusable physical-plan object; sparq has neither.
- **Opt-in:** opt-in `plan-cache` feature (bounded LRU, like `result-cache`).

### 2.3 Sub-plan / intermediate-result cache (the QLever model)
- **Gap (verified):** sparq caches only the **final** `QueryResult` (`cache.rs`).
  **QLever's LRU caches results of *intermediate operations* and the planner treats a
  cached subtree as cost-zero, with explicit pinning** `[M, QLever wiki/UI]`. sparq has
  no notion of a cached SCAN/SORT/JOIN subtree shared across queries. `[V]`
- **Why it matters:** dashboards / federation sub-queries / autocomplete share large
  common sub-expressions (a popular SORT, a heavy star). Caching the subtree turns the
  *second* overlapping query into a cheap top-up. This is the one piece of MQO that the
  `concurrent-serving.md` sweep did **not** reject — it rejected *online batch* MQO, not
  *cross-query sub-result reuse*.
- **Fit:** medium-strong, but **soundness-heavy**: needs (a) a canonical sub-plan key
  (algebra + active-dataset + the same version discipline `cache.rs` already uses), and
  (b) the planner to consult it (a new `est = 0` path in `goo_pick`). The materialising
  evaluator makes the *cached object* trivial (it already produces `Vec<Row>`); the work
  is keying + planner integration + eviction accounting.
- **Novelty check:** **absent.** Honest partial overlap: the result cache is the
  degenerate top-of-tree case of this.
- **Opt-in:** opt-in `subplan-cache` feature; **defer behind 2.2** (plan cache is the
  cheaper, lower-risk first step).

### 2.4 Persistent, incrementally-maintained statistics catalog
- **Gap (verified):** cardinality is **recomputed per query** from index ranges +
  `PredStat` marginals; the characteristic-set table is **caller-built and must be
  rebuilt when the graph is rebuilt** (`cs.rs:24`). There is **no persistent statistics
  catalog** — no stored per-predicate object histograms / quantiles, no precomputed
  correlated-predicate join counts, no auto-maintenance on update, no on-disk stats for
  the mmap mode. `[V]`
- **Why it matters:** index-range counts give exact *single-pattern* sizes but only an
  independence-assumption product for joins; the literature (Neumann CS, SumRDF, shape
  statistics EDBT'21, worst-case stats HAL-01524387) shows precomputed RDF synopses beat
  independence on correlated BGPs — the exact mis-order that triggers intermediate
  blow-up. A *persistent* catalog also lets EXPLAIN show estimates without touching data,
  and lets the planner cost without a probing pass.
- **Fit:** strong and architecturally aligned: sparq's six sorted permutations make
  histogram/quantile extraction a linear scan; dense u32 ids make per-id frequency a
  flat array. For the mmap out-of-core mode this is the *enabling* piece (you can't
  probe ranges as cheaply when paged from disk). **Scope honestly:** auto-maintenance
  under updates is the hard part — v1 should be a **build-time / explicit-`ANALYZE`**
  catalog, not incremental.
- **Novelty check:** characteristic sets exist (opt-in, caller-built, in-RAM) — this is
  the **persistence + auto-build + histogram breadth** delta, not CS itself.
- **Opt-in:** opt-in `stats-catalog` feature, serialisable alongside the mmap format.

### 2.5 Spill-to-disk for the *local* evaluator (sort / hash-join / group-by)
- **Gap (verified):** `extsort.rs` spills **only during index BUILD**; dict-spill is for
  ingest; fed has a streaming-join spill. The **local SELECT evaluator fully materialises
  intermediates as `Vec<SmallVec<[Id;4]>>` in RAM** and the `QueryBudget` (`exec.rs:112`)
  merely **aborts** when a row cap / deadline is hit — there is no out-of-core hash join,
  external ORDER BY, or spilling GROUP BY. A query whose intermediate exceeds RAM on the
  native billion-scale target either OOMs or is budget-killed. `[V]`
- **Why it matters:** **DuckDB** spills hash-join, sort, and hash-aggregate to a buffer
  pool transparently (DuckDB 0.9 out-of-core join; memory-management blog) `[M]`. For
  sparq's stated native-billion-scale "beat QLever" goal, a query with a large
  intermediate is a correctness-of-completion issue, not just a speed one. The semi-join
  reducer / Yannakakis work in `optimization-techniques.md` *shrinks* intermediates but
  cannot *bound* them on adversarial BGPs — spill is the safety net.
- **Fit:** medium; it is a real engineering lift (a spilling operator needs a partition
  or run-file abstraction). But sparq already has the **building blocks**: `extsort.rs`'s
  run-file spill/merge and the out-of-core mmap discipline. Native-only (WASM cannot
  spill to a POSIX FS; the browser stays in-RAM + budget-abort — that asymmetry is fine
  and matches the existing native-vs-wasm split in `optimization-techniques.md` §2).
- **Novelty check:** **absent in the local evaluator.** Honest partial overlap: ingest
  spill + fed-join spill exist and are reusable scaffolding.
- **Opt-in:** opt-in `spill` feature, native-only (`cfg(not(wasm))`).

### 2.6 Local adaptive / mid-query re-optimization
- **Gap (verified):** the divergence-triggered re-planner lives **only in
  `sparq-fedplan`** (`adaptive.rs:1`). The local evaluator commits to one GOO order and
  runs it to completion — even though, because it **fully materialises each intermediate,
  the TRUE cardinality of every completed stage is known for free.**
  `optimization-techniques.md` §2(a) item 4 *recommends* exactly this ("mid-query
  re-optimization checkpoint … near-zero-cost") but it is **not implemented** for the
  local path. `[V]`
- **Why it matters:** this is the cheapest robustness lever sparq has and it is sitting
  unused. After each materialised join, compare actual vs estimated; if a remaining-BGP
  estimate was off by `>k×`, re-order the *unexecuted* patterns (the fed re-planner's
  exact recipe, ported to local). The literature (ReOpt mid-query re-opt; the
  order-robustness line in arXiv 2502.15181) shows this collapses catastrophic plans.
- **Fit:** strong; it is **mostly a port** of the existing fed `ReplanPolicy` +
  hysteresis to consume the row counts the local evaluator already produces. No new
  cost-model, no new operators.
- **Novelty check:** **absent for local** (present for fed). This is the honest "we built
  it once on the wrong side of the boundary" case.
- **Opt-in:** opt-in `adaptive-replan-local` feature (mirror the fed gate); de-risk GOO.

### 2.7 Incremental view maintenance (IVM) for materialised SELECT/CONSTRUCT views
- **Gap (verified):** the `cache.rs` module's doc-header *calls itself* a "materialised
  view", but it is a **result cache invalidated by a whole-graph version bump** — on any
  mutation the cached entry is discarded and recomputed from scratch, not delta-updated.
  There is **no incremental maintenance**: no counting-algorithm delta propagation, no
  standing CONSTRUCT view kept consistent under `apply_delta`. `[V]`
- **Why it matters:** there is a **2024+ ACM paper adapting the counting algorithm to
  SPARQL IVM** (dl.acm.org/doi/10.1145/3796549), and an established RDF linkset-IVM line
  (Springer 2016). A true IVM view updates in O(delta), not O(view) — the difference
  between a dashboard / standing CONSTRUCT (e.g. a reasoner-style derived graph, a Solid
  inbox aggregation) that refreshes incrementally vs one that recomputes on every write.
  It dovetails with sparq's existing reasoner materialisation (which already has
  `incremental_explain`) and the DeltaTriples overlay direction.
- **Fit:** medium; **soundness-heavy and scope-sensitive.** IVM is fully incremental only
  for the monotone + counting-friendly fragment (BGP/JOIN/UNION/positive-FILTER; COUNT/
  SUM/AVG but **not** MIN/MAX/SAMPLE per the literature). v1 must scope to that fragment
  and fall back to recompute elsewhere — exactly what the counting-algorithm papers do.
- **Novelty check:** the *name* exists in `cache.rs` but the *mechanism* does not — this
  is the honest "advertised but not delivered" gap. Worth a doc note to **rename the
  cache** so it stops over-claiming "materialised view".
- **Opt-in:** opt-in `ivm` feature; couples to the reasoner's incremental machinery.

### 2.8 Per-query profiling / plan telemetry (beyond EXPLAIN ANALYZE + aggregate /metrics)
- **Gap (verified):** EXPLAIN ANALYZE gives a per-operator trace **for one query you ask
  about**; `/metrics` gives **aggregate** request counters + a latency histogram
  (`metrics.rs`). There is **no structured per-query record** capturing
  plan-vs-actual cardinality (q-error), join order chosen, spill events, cache
  hit/miss-by-subtree, slow-query log, or a machine-readable EXPLAIN (JSON). `[V]`
- **Why it matters:** plan-vs-actual telemetry is the feedback signal that *every other
  candidate here consumes* — it tells you when stats (2.4) are wrong, when re-opt (2.6)
  should fire, when the plan cache (2.2) is caching a bad plan, and it surfaces directly
  in the **GUI** as a query-profiler panel (a clear maintainer direction:
  GUI-embedded-app with persistent workspaces). It is also the cheapest of the lot.
- **Fit:** strong; the EXPLAIN ANALYZE trace infrastructure (`exec.rs:304`) already
  collects per-operator rows + time — this is mostly **expose it as structured JSON +
  attach estimate-vs-actual** and a bounded slow-query ring. WASM-friendly (row counts
  work; wall time is 0 on wasm32, already handled).
- **Novelty check:** EXPLAIN ANALYZE (text) + aggregate metrics exist; the **structured
  JSON plan + q-error + slow-query ring** is the delta.
- **Opt-in:** small; JSON EXPLAIN can be core, the slow-query ring an opt-in server knob.

---

## 3. Cross-cutting honesty notes

- **Do NOT re-propose** vectorized execution, semi-join reducers (exact-bitmap /
  Yannakakis), WCOJ, FastLanes columns, or the cardinality-*estimator* algorithms — all
  already triaged in `optimization-techniques.md`. The persistent-stats candidate (2.4)
  is the *storage/maintenance* of statistics, distinct from the *estimator choice* that
  doc already covers.
- **Do NOT re-propose** online batch MQO / shared-scan or whole-query result caching —
  `concurrent-serving.md` already rejected the former for the hot path and shipped the
  latter. The sub-plan cache (2.3) is the *one* reuse form that doc left open.
- **`parallel` partially exists** — sparq has rayon data-parallel scan/filter/construct,
  so "add parallel execution" is **not** a clean gap. Morsel-driven *pipeline*
  parallelism with work-stealing is a real further step but is **entangled with the
  vectorized-engine rewrite** in `optimization-techniques.md` §2′ and is better tracked
  there, not duplicated here.
- **Work-box numbers** (the µs planning costs from `concurrent-serving.md`) are cited
  only as cost *shape*, never as canonical results.

---

## 4. Recommendation

A **robustness-and-reuse track for the local engine**, ordered cheapest-first so each
phase de-risks the next, all opt-in to honour the lean-core rule:

1. **Parameterized prepared queries** (2.1) — safety primitive, tiny, core; unlocks 2.2.
2. **Per-query structured profiling / q-error telemetry** (2.8) — the feedback signal
   the rest consume; surfaces in the GUI; cheap.
3. **Local mid-query re-optimization checkpoint** (2.6) — a *port* of the existing fed
   re-planner over the row counts the local evaluator already materialises; highest
   robustness-per-line.
4. **Plan cache** (2.2) — amortise planning over parameter bindings; composes with 1.
5. **Persistent statistics catalog with histograms** (2.4) — build-time / explicit
   `ANALYZE`; the enabling piece for the mmap out-of-core mode.
6. **Spill-to-disk for the local evaluator** (2.5) — native-only; the completion safety
   net for the billion-scale target; reuses `extsort` scaffolding.
7. **Sub-plan / intermediate-result cache** (2.3) — the QLever reuse model; defer behind
   the plan cache (shares the keying machinery).
8. **Incremental view maintenance** (2.7) — the largest, most soundness-sensitive;
   couples to the reasoner's incremental path; v1 scoped to the counting-friendly
   fragment. **Also rename `cache.rs`'s self-described "materialised view"** so it stops
   over-claiming until real IVM lands.

**If only one ships:** (2.6) local mid-query re-optimization — it is nearly free
(consumes data the engine already produces), it is a port not a greenfield build, and it
directly attacks the `BENCHMARKS.md` "gap widens with scale" finding by making GOO's
mis-orders self-correcting.

---

## 5. Phased plan (each phase → a future bead for the orchestrator)

1. **`params`** — parameterized prepared queries: positional/named placeholder binding as
   an algebra rewrite over `PreparedQuery::from(Query)`; escaping/validation; injection
   round-trip tests. *(core or thin feature)*
2. **`query-profile`** — structured JSON EXPLAIN + per-query plan-vs-actual (q-error) +
   bounded slow-query ring; reuse the EXPLAIN ANALYZE trace; GUI-facing schema.
3. **`adaptive-replan-local`** — port the fed `ReplanPolicy` + hysteresis to the local
   evaluator's materialised-stage row counts; differential test (same results as
   non-adaptive); de-risk GOO. *(opt-in)*
4. **`plan-cache`** — bounded-LRU physical-plan cache keyed on placeholder-abstracted
   algebra + active dataset + version; composes with phase 1. *(opt-in)*
5. **`stats-catalog`** — persistent per-predicate object histograms/quantiles +
   correlated-predicate join counts; explicit `ANALYZE`; serialise alongside mmap;
   wire into `goo_pick` as a better seed. *(opt-in)*
6. **`spill`** — native-only out-of-core hash-join / external ORDER BY / spilling
   GROUP BY built on `extsort` run-files; budget escalates to spill before abort.
   *(opt-in, `cfg(not(wasm))`)*
7. **`subplan-cache`** — cache intermediate operator results with planner cost-zero
   awareness + pinning (QLever model); shares phase-4 keying. *(opt-in, after phase 4)*
8. **`ivm`** — counting-algorithm IVM for the monotone/COUNT-SUM-AVG fragment of standing
   SELECT/CONSTRUCT views; recompute-fallback elsewhere; couple to reasoner incremental;
   rename `cache.rs` to stop over-claiming "materialised view". *(opt-in)*

---

## 6. Open questions for the maintainer

1. **Lean-core boundary:** parameterized queries (2.1) and JSON EXPLAIN (2.8) are tiny
   and dependency-free — promote to **core**, or keep *everything* behind features for
   strict lean-core?
2. **Billion-scale completion policy:** for a local query whose intermediate exceeds RAM,
   is the intended contract **spill-to-disk** (2.5, slower-but-completes) or
   **budget-abort** (current)? This decides whether 2.5 is on the critical path for the
   "beat QLever" native target.
3. **IVM appetite:** is a true IVM layer (2.7) in scope, or should `cache.rs` simply be
   **renamed** to drop the "materialised view" claim and the version-invalidated result
   cache be considered sufficient for the foreseeable serving workloads?
4. **Stats maintenance model:** build-time / explicit-`ANALYZE` catalog (2.4) first, with
   incremental auto-maintenance deferred — acceptable? Or is auto-maintenance required
   for the update-heavy Solid workloads from day one?
5. **GUI profiler:** should the per-query profiling schema (2.8) be designed *now* against
   the GUI's query-profiler panel so the engine emits exactly what the GUI renders?

---

### Source URLs

- Jena ParameterizedSparqlString: https://jena.apache.org/documentation/query/parameterized-sparql-strings.html
- SPARQL query parameterization (w3c/sparql-dev #57): https://github.com/w3c/sparql-dev/issues/57
- LTBQP security (parameterization as mitigation): https://arxiv.org/pdf/2210.04631
- QLever intermediate-result cache + pinning: https://github.com/ad-freiburg/qlever/wiki/QLever-performance-evaluation-and-comparison-to-other-SPARQL-engines ; https://github.com/ad-freiburg/qlever
- DuckDB out-of-core hash join + memory management: https://duckdb.org/2023/09/26/announcing-duckdb-090 ; https://duckdb.org/2024/07/09/memory-management ; https://github.com/duckdb/duckdb/pull/4189
- SPARQL IVM (counting algorithm, 2024+): https://dl.acm.org/doi/pdf/10.1145/3796549
- Incremental maintenance of SPARQL linkset views (2016): https://link.springer.com/chapter/10.1007/978-3-319-44406-2_7
- Worst-case SPARQL cardinality from statistics: https://inria.hal.science/hal-01524387
- Shape statistics for SPARQL optimization (EDBT'21): https://openproceedings.org/2021/conf/edbt/p202.pdf
- ROSIE runtime optimization of SPARQL (incremental eval): https://arxiv.org/pdf/1605.06865
- Mid-query re-optimization / ReOpt; order-robustness: https://arxiv.org/pdf/2502.15181
- Adaptive query processing survey (eddies): https://www.cis.upenn.edu/~zives/research/aqp-survey.pdf
