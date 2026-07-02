# sparq — Query-Engine Optimization Techniques: Discovery, Triage, Roadmap

A lead-architect triage of an open-ended discovery sweep of query-engine
optimization techniques (join/merge, execution architecture, cardinality/planning,
scan/compression execution, hardware/parallelism, RDF-specific + cross-pollination),
mapped onto sparq's **actual current design** and its **measured scaling gap**.

## Grounding: what sparq is *today*, and the prize

From `research/ARCHITECTURE.md` and `research/BENCHMARKS.md`:

- **Storage:** six sorted permutation indexes (PSO/POS/SPO/SOP/OPS/OSP), currently
  flat in-memory `Vec<[u32;3]>` (M1); column-major ZSTD blocks + skip metadata is
  the M3 *plan*, not yet built.
- **Dictionary:** dense u32 ids; small `xsd:integer`s inlined in the id
  (`INLINE_BASE + value`), value-sorted ⇒ numeric FILTER range-prunes via
  binary search. Tagged 64-bit ValueId is the M4 plan.
- **Joins:** merge join by default on co-sorted scans, gallop for skew, hash when
  not co-sorted, **Leapfrog Triejoin (LFTJ)** for cyclic/skewed BGPs. GYO
  acyclicity test routes cyclic→WCOJ, else binary.
- **Planner:** spargebra → physical IR → filter pushdown → cardinality-cost **greedy
  GOO** join ordering by DEFAULT. An OPT-IN `dp-planner` feature (bead `sq-iywur`) adds a
  connected-subgraph-complement-pair (**DPccp**) dynamic-programming enumerator that finds
  a `Cout`-optimal *bushy* tree, **falling back to GOO above a connected-subgraph budget**
  (and on disconnected BGPs). It is order-only (result-identical to GOO) and OFF by default.
  Hypergraph **DPhyp** / interesting-orders-in-the-DP-table are NOT implemented. Cardinality
  from block-metadata counts + per-relation multiplicities + **characteristic sets** for stars.
- **Execution:** a **materialising evaluator** — builds `Vec<SmallVec<[Id;4]>>`
  rows. **No SIMD kernels, no compilation, and the query evaluator is still
  row-at-a-time.** The first building block of the vectorized `DataChunk` M4 path
  has landed (bead `sq-hvfe`): `sparq-engine`'s opt-in `vectorized` feature ships a
  column-major `DataChunk` intermediate plus a numeric-FILTER comparison kernel
  (→ selection vector) and a selection/gather kernel — the vector-at-a-time analogue
  of the row evaluator's per-row FILTER, with a tested row↔column equivalence
  contract. It is **NOT yet wired into the evaluator** (`query`/`query_json` are
  unchanged); the remaining work is the morsel-driven pipeline + vectorized joins
  (roadmap T1.3). <!-- [OPUS-4.8] -->
- **Two targets:** native billion-scale (beat QLever) **and** a sub-~1.5 MB-gzip
  WASM browser build (Solid/RDFJS), where the only vector path is `+simd128`
  autovectorization and shipping a JIT is impractical.

**The measured prize (BENCHMARKS.md):** micro-optimisation got sparq to parity or
better than native QLever at 1.8M–10M *on the 5 compute queries it tuned*, but the
**honest finding is that the gap WIDENS with scale** — at 10M the broader compute
geomean was 0.27× (QLever ~3.7×) before the targeted fixes, and the residual
losses are exactly **per-row id→term materialisation + per-row dispatch overhead**
(q06 filter, q04 5M-row join, q10 OPTIONAL) and **flat uncompressed `Vec<[u32;3]>`
scans** that lose to QLever's compressed columnar block scans. **The scaling gap is
the prize.**

**Evidence-level tags** used below: `[M]` measured in cited paper/system;
`[L]` established in literature/theory; `[C]` author-claimed (paper's own claim,
not independently reproduced); `[S]` speculative / my inference for sparq; `[B]`
measured in sparq's own BENCHMARKS.md.

---

## 1. Techniques grouped by subsystem

Promise (P) is the scout's 1–5 score; *Win* = expected speedup with conditions;
*Cost* = implementation cost; *Ev* = evidence level for the headline number.

### 1.1 JOIN / merge algorithms

| Technique | Promise | Expected win (conditions) | Cost | Ev | Source |
|---|--:|---|---|---|---|
| **Exact bitmap semi-join filters on dense IDs** ("Not Yannakakis", CIDR'26) | 5 | Yannakakis-ideal reduction, ~0 filter overhead on dense-key joins; biggest on star/snowflake. Mem = domain/8 B per filter | **small** | C | cidrdb.org/cidr2026 p29-zhao |
| **Yannakakis / full semijoin reduction for acyclic BGPs (GYO)** | 5 | O(in+out); 2–2.4× avg, up to ~47,000× pathological; chain/snowflake with big intermediates | medium | L | arxiv 2504.03279 |
| **Predicate Transfer (Bloom semijoins over join graph)** | 5 | Large on selective acyclic BGPs w/ big intermediates; net-negative on tiny (must cost-gate) | medium | M | cidr2024 p22-yang |
| **Robust Predicate Transfer (RPT, LargestRoot)** | 5 | 1.46–1.56× avg vs DuckDB **+** collapses 30–371× join-order variance to ≤1.6× | medium | M | arxiv 2502.15181 |
| **Yannakakis+ (DAG of standard ops, plug-in)** | 4–5 | better on 160/162 queries, avg 2.41×, max ~47,059×; many-join low-selectivity | medium | M | qichen-wang yannakakis+ |
| **Shredded Yannakakis (Lookup/Expand, "without regret")** | 4 | improved 85.3% of 1,849 q, up to 62.5×, neutral by construction elsewhere | large | M | arxiv 2411.04042 |
| **TreeTracker Join (backjump+delete, no extra filter)** | 4 | ≤ binary-hash probe count on *any* plan; wins where dangling tuples inflate pipeline | medium | M | arxiv 2403.01631 |
| **Free Join (unify binary+WCOJ, COLT lazy trie)** | 4 | 2–10×+ on selective/skewed mixed-shape; competitive elsewhere | large | M | arxiv 2301.10841 |
| **Free Join extended to SORT-based WCOJ** | 4 | up to 3.1×/4.8× over Free Join; reuses sorted indexes (no hash trie) | large | M | arxiv 2505.19918 |
| **Factorised join execution (FDB products)** | 4 | orders of magnitude on many-to-many; none on 1:1/tiny | large | L | vldb vol5 p1232 |
| **Adaptive factorization via linear-chained hash (DuckDB ctx)** | 4 | ~17.6× targeted many-to-many/triangle; low downside (adaptive trigger) | medium | M | cidr2025 p21-gro |
| **Lookahead Information Passing (LIP), adaptive Bloom order** | 4 | robust near-optimal on star schema; trades DRAM probes for cache probes | small | M | vldb vol10 p889-zhu |
| **Fair/instance-optimal LFTJ seek** | 4 | instance-optimal seek; scales with term-frequency skew (RDF very skewed) | small | C/L | arxiv 2510.26016 |

### 1.2 EXECUTION ARCHITECTURE

| Technique | Promise | Expected win (conditions) | Cost | Ev | Source |
|---|--:|---|---|---|---|
| **Vector-at-a-time execution (columns-of-ValueIds + selection vectors) — BARQ blueprint for SPARQL** | 5 | **3.4× overall CPU-bound; 27–33× on heavy-intermediate (LSQB Q6/Q9); negligible IO regression. No JIT needed.** | large | M | arxiv 2504.04584 |
| **Adaptive batch sizing (next/skip/reset ratios)** | 4 | prevents the 33% regression on IO-bound/selective queries vectorization otherwise causes | small | M | arxiv 2504.04584 |
| **VOILA (one order-free op spec → vectorized OR compiled code)** | 4 | engineering win: one operator codebase, two targets; generated engines up to 35.5× vs known systems | large | C | t1mm3 vldb21 |
| **Copy-and-Patch / stencil compilation (no LLVM)** | 4 | compile ~100–1000× faster than LLVM; exec ~10× over interp, ~14% over LLVM -O0; native-mostly | large | M | sillycross copy-and-patch |
| **Relaxed Operator Fusion (ROF, staging + prefetch)** | 4 | up to 2.2× OLAP; **native-only** (WASM has no prefetch); helps cache-missing hash/seek joins | medium | M | cmu 2017 p1-menon |
| **Micro-adaptivity / bandit kernel selection (CAKE)** | 4 | up to 2× vs static heuristics; converges to best static (low risk); value grows with workload variety | medium | M | arxiv 2602.04181 |

### 1.3 CARDINALITY / PLANNING

| Technique | Promise | Expected win (conditions) | Cost | Ev | Source |
|---|--:|---|---|---|---|
| **Index-Based Join Sampling ("Cardinality Estimation Done Right")** | 5 | 100k lookups ⇒ only 17% off ≥2×, 3% badly off; reuses the six indexes | **small** | M | cidr2017 p9-leis |
| **Wander Join (random-walk join-size sampling)** | 5 | G-CARE: best-in-class graph-query estimation; statistics-free; doubles as COUNT approx | medium | M | cse.ust.hk sigmod16 |
| **LpBound (pessimistic lp-norm + LP bound)** | 5 | never-underestimate guarantee; tiny per-predicate stats (browser-friendly); prevents catastrophic plans | medium | C | arxiv 2502.05912 |
| **Mid-query re-optimization (materialization checkpoints)** | 4 | large where first estimate badly wrong, ~0 overhead otherwise; **free since sparq already materialises** | **small** | L | arxiv 1902.08291 |
| **SafeBound (compressed degree-seq bounds)** | 4 | up to 80% lower runtime vs PG, 500× faster planning, 6.8× less space; server-class | medium | M | arxiv 2211.09864 |
| **DPhyp (DP over query hypergraph)** | 4 | optimal bushy vs greedy; 6–15-pattern BGPs; negligible tiny | medium | L | DPccp shipped opt-in (`sq-iywur`); DPhyp still planned |
| **IKKBZ + LinDP (adaptive, poly-time large joins)** | 4 | better plans than GOO, scales to huge BGPs (path/reasoning expansion) | medium | L | db.in.tum.de hugejoins |
| **Zero-overhead bottom-up Yannakakis (semijoin in hash build)** | 4 | no-regret vs binary by construction; wins concentrated on pathological | large | C | vldb vol17 p3215-birler |

### 1.4 SCAN / FILTER / COMPRESSION-EXEC

| Technique | Promise | Expected win (conditions) | Cost | Ev | Source |
|---|--:|---|---|---|---|
| **FastLanes unified transposed layout + interleaved bit-packing (autovectorizing scalar decode)** | 5 | >40 vals/cycle, >100 B ints/s; 2–4× decode + size cut; **one scalar source → x86/ARM/wasm-simd128**. Coupled to a column-storage refactor | large | L | cwi 32992 |
| **BitWeaving/V & /H (word-parallel bit-sliced predicates, no SIMD)** | 4 | 2–6× realistic on selective numeric scans over narrow inline ints; pure integer ⇒ identical native+wasm | medium | L | jigneshpatel BitWeaving |
| **Column Sketches (1-byte order-preserving code map)** | 4 | ~2–5× selective filters over wide domains; robust (no selectivity cliff); Rust `ordbog` exists | medium | M | daslab columnsketches |
| **RLE-aware operators (compute once per run on sorted-prefix runs)** | 4 | 5–100× fewer comparisons on leading-column-bound patterns; needs a run/delimiter index | medium | L/S | uspto 9652501 |

### 1.5 HARDWARE / PARALLELISM

| Technique | Promise | Expected win (conditions) | Cost | Ev | Source |
|---|--:|---|---|---|---|
| **Branchless SIMD sorted-set intersection (Inoue / Vardanian)** | 5 | 1.6–5× kernel (dense/medium); ~1.5–3× on wasm-simd128 (128-bit, no gather); end-to-end ∝ time in merge loop | medium | M | ashvardanian; vldb 2735518 |
| **Vectorized batch model w/ selective load/store (MonetDB/X100)** | 5 | 10–50× tuple-at-a-time on scan/filter/agg; architectural rewrite; ceiling = bandwidth | large | L | vldb vol11 p2209-kersten |
| **Software-prefetch / coroutine latency-hiding (native)** | 4 | hides hash/index miss latency; **native-only** (no wasm prefetch) | medium | M | (ROF / hash-table studies) |
| **128-bit, gather-free, branch-free kernel discipline** | 4 | portability win: same kernels on wasm-simd128 + NEON; AVX-512 gather often net-negative | medium | M | vldb'23 vectorized-hashtables |

### 1.6 UPDATES

No technique in this sweep is update-specific; sparq is read-optimised and updates
are an M5 scope-gated tier (DeltaTriples overlay + RDFox-style mutable store).
**Relevance:** a semi-join reducer / predicate-transfer prepass operates over a
*read snapshot* and composes cleanly with a DeltaTriples overlay (run the reducer
over base+delta union). No conflict, no new requirement. *Out of scope for the
scaling prize.*

### 1.7 BROWSER (WASM) — cross-cutting

| Technique | Fit for WASM | Why |
|---|---|---|
| Vector-at-a-time (BARQ) | **Strong** | DuckDB-Wasm confirms vectorized engines AOT-compile to `wasm+simd128`, no runtime codegen `[M]` |
| FastLanes scalar decode | **Strong** | one scalar source autovectorizes to wasm-simd128; shrinks the fetched index blob `[L]` |
| BitWeaving (pure integer) | **Strong** | zero intrinsics, identical native/wasm `[L]` |
| Exact bitmap / Bloom semi-join over u32 ids | **Strong** | tiny, cache-friendly, trivially wasm-able `[S]` |
| LpBound stats | **Strong** | a few scalars per predicate fit the 2–4 GB budget `[C]` |
| Branchless SIMD intersection | **Medium** | only 128-bit, no gather ⇒ 1.5–3× not 5× `[M]` |
| ROF / software prefetch | **Reject for wasm** | WASM has no prefetch instruction `[M]` |
| Copy-and-Patch JIT | **Reject for wasm (near-term)** | in-wasm codegen is awkward; impractical in a small bundle `[S]` |
| Compilation (Cranelift/LLVM) | **Reject for wasm** | can't ship the toolchain in <1.5 MB `[S]` |

---

## 2. Prioritised roadmap

Composing with the **existing** sort-merge + LFTJ + GOO (opt-in DPccp) + inline-ValueId +
range-pruning design. The organising principle the sweep makes unavoidable: the
2018–2026 literature has **pivoted from better join *ordering* to join-order-*robust*
semi-join pre-reduction** (Yannakakis revival), and **the highest-leverage
architectural move is vector-at-a-time execution** — both of which sparq is missing
and both of which compose cleanly with what it has.

### (a) ADOPT NOW — high confidence, fits the design, small/medium cost

1. **Exact bitmap semi-join reducer on dense u32 ids** (CIDR'26). The single
   best *technique-architecture fit* in the whole sweep: sparq's dictionary
   *already* produces dense u32 ids and inlines small ints — exactly the regime
   where a flat bitmap is a perfect-hash, zero-false-positive, branch-free
   membership test, **strictly cheaper and more accurate than the Bloom filters**
   the rest of the literature settled for. Build over a join variable's reachable
   id range; fall back to Bloom only when the domain is sparse+huge. `small` cost,
   wasm-strong. `[C]`
2. **Yannakakis / full semi-join reduction over the acyclic BGP join tree**, with
   the **Yannakakis+ "DAG-of-standard-operators" integration recipe** so it reuses
   sparq's *existing* merge-join and index scans rather than a bespoke reducer
   engine. sparq's six sorted permutations make each semi-join a sorted-merge
   intersection it already does via `partition_point`. Route via the GYO test
   sparq already has: **acyclic → semi-join-reduce → join; cyclic → existing LFTJ.**
   This directly attacks the materialising-evaluator's #1 cost (large intermediates)
   and de-risks GOO. `medium`. `[L]/[M]`
3. **Index-Based Join Sampling** for cardinality (CIDR'17). `small` cost, reuses
   the six indexes, fixes the exact correlated-multi-join cases GOO/NDV/characteristic-
   sets miss. Strictly better seed for DPccp/GOO than independence-assumption stats.
   `[M]`
4. **Mid-query re-optimization checkpoint.** Because sparq *already materialises*
   each intermediate, the true size is known for free; feeding it back to re-order
   the remaining patterns is a near-zero-cost robustness layer. `small`. `[L]`

### (b) PROTOTYPE / SPIKE — high upside, must be measured before committing

5. **Vector-at-a-time execution (the BARQ blueprint).** This is the **M4 plan**
   already in the architecture, but the sweep upgrades it from "nice-to-have SIMD"
   to **the highest-leverage decision** (see §2 recommendation). Spike a *single*
   vectorized pipeline (scan → filter → merge-join → count) on columns of u32 +
   selection vector before committing the engine-wide rewrite. `large` overall;
   the spike is medium. `[M]`
6. **RPT / Predicate Transfer for the acyclic+skew case**, *if and only if* the
   exact-bitmap reducer (a1) proves insufficient on cyclic or sparse-domain BGPs.
   RPT's order-robustness (371×→1.6× variance) is the insurance policy for GOO.
   Spike on the worst-ordered WatDiv/WDBench BGPs. `medium`. `[M]`
7. **FastLanes-style transposed bit-packed columns** as the M3 column format
   (instead of plain ZSTD-3). It is uniquely aligned with sparq's two hard
   constraints (autovectorization-only + small wasm bundle) and directly attacks
   the scan-bandwidth loss. Coupled to the column-storage refactor, so spike the
   decode kernel standalone first and measure ints/s on native + wasm. `large`. `[L]`
8. **Wander Join** as a statistics-free cardinality estimator (G-CARE best-in-class
   on graphs). Spike its accuracy vs sparq's characteristic-sets on real
   correlated BGPs; if it wins it *replaces machinery with less machinery*. `medium`. `[M]`
9. **Adaptive factorization (linear-chained hash)** for the many-to-many RDF blow-up
   case — the retrofittable, low-downside route to factorised wins without an
   FDB-algebra rewrite. Spike on star-of-stars / popular-subject BGPs. `medium`. `[M]`

### (c) SHELF — promising but premature, or subsumed by (a)/(b)

- **Shredded Yannakakis** — its "without regret" property is attractive, but it
  *overlaps* with the exact-bitmap + Yannakakis+ adopt-now choices and is `large`.
  Pick one acyclic strategy first; revisit SYA only if (a2) underperforms inside a
  vectorized engine.
- **Free Join / SORT-based Free Join / TreeTracker** — unify binary+WCOJ elegantly,
  but sparq already routes the two via GYO and gets most of the value; revisit only
  if mixed acyclic-with-embedded-cycle BGPs show up as a measured loss.
- **VOILA, Copy-and-Patch, ROF, CAKE micro-adaptivity** — all `large`/`medium` and
  only pay off *after* the vectorized engine exists. VOILA in particular is best
  used now as a **design constraint** (write operators order-free) rather than a
  built layer.
- **LpBound / SafeBound** — adopt as a *safety net* (never-underestimate guard to
  prevent catastrophic plans), not a primary estimator; shelf until Index-Based
  Join Sampling (a3) is in and its failure modes are known.
- **DPhyp / IKKBZ+LinDP** — DPhyp is already the M2 plan; LinDP is a scale-up only
  needed if reasoning/path expansion generates very large BGPs.
- **BitWeaving, Column Sketches, RLE-aware operators, fair-LFTJ-seek, branchless
  SIMD intersection** — real but *second-order* (they speed compute that is already
  cheap relative to the bandwidth/materialisation ceiling). Harvest *after* the
  vectorized model exposes the hot loops; the fair-LFTJ-seek and SIMD-intersection
  are drop-in kernel swaps to do then.

### (d) REJECT (for this engine) — see §4 for the honest rationale

- **ROF software-prefetch and Copy-and-Patch JIT in the *browser*** — WASM has no
  prefetch and can't host a codegen toolchain in a small bundle.
- **Full LLVM/Cranelift data-centric compilation as the *primary* model** — the
  workload is memory-bandwidth-bound, where vectorization wins and compilation
  mainly helps cache-resident compute `[B from ARCHITECTURE decide-vec-vs-compile]`.
  Keep compilation as a *native-only, opt-in* fast path behind a VOILA-style spec,
  never the browser path.
- **AVX-512 gather-based kernels as a portability baseline** — frequently net-negative
  even on x86 and absent on wasm; build 128-bit gather-free kernels instead.
- **Full FDB factorised-algebra engine rewrite** — research-grade complexity;
  harvest factorisation in the specific forms of WCOJ tries + Yannakakis-Expand +
  adaptive-factorization instead.

### Native-billion-scale vs browser — explicit split

| Decision | Native (beat QLever, billions) | Browser (WASM, ≤ few-GB, ≤1.5 MB bundle) |
|---|---|---|
| Execution model | **Vectorized**, optionally + native-only compiled fast path | **Vectorized only** (AOT to wasm-simd128) |
| Semi-join reducer | Exact bitmap on dense ids; Bloom/RPT fallback for sparse/cyclic | Same — tiny u32 bitmaps/Blooms are ideal here |
| Compression-exec | FastLanes columns + ZSTD; SIMD-BP128 decode | FastLanes scalar decode (shrinks fetched blob) |
| Latency hiding | Software prefetch / ROF / coroutines (native intrinsics) | **None** — rely on cache-resident batches + selection vectors |
| Cardinality | Index-Based Join Sampling + characteristic sets + Wander Join; SafeBound guard | Index sampling + LpBound (tiny stats fit the budget) |
| Parallelism | Morsel work-stealing, pinned pool | Single-thread default; `wasm-bindgen-rayon` behind COOP/COEP |
| Kernels | 128-bit gather-free baseline + opt-in AVX-512/NEON | 128-bit gather-free only |

---

## 2′. The highest-leverage decision: EXECUTION ARCHITECTURE

**Stay materialising vs go vectorized vs compile — recommendation.**

The sweep makes this the single most consequential choice, and the evidence is
unusually direct because **BARQ (Stardog, 2025) is a vectorized SPARQL engine built
from exactly sparq's primitives** — columns of dictionary-encoded ids, selection
vectors, and *skip-aware merge joins over sorted permutation indexes* — and it
measured **3.4× overall CPU-bound throughput and 27–33× on heavy-intermediate
queries, with NO JIT/codegen** `[M, arxiv 2504.04584]`. sparq's measured residual
losses (per-row id→term, per-row dispatch, q04/q06/q10) are precisely what a
column-of-u32-batch + selection-vector layout removes. The skip-aware Probe/Build/Skip
merge join maps directly onto sparq's `partition_point` seeks into permutation
slices.

**Recommendation — NATIVE:** **Go vectorized** (the M4 plan, promoted to the
top-priority architectural investment). Keep the materialising path only as the
fallback for row-shaped operators (property paths). Reserve **data-centric
compilation as a native-only, opt-in fast path** for cache-resident compute-heavy
fragments (arithmetic FILTER/BIND trees, hot repeated queries), built behind a
**VOILA-style order-free operator spec** and a **Copy-and-Patch stencil backend**
(not LLVM) so compile latency never dominates the few-triples regime. Reasoning:
the workload is memory-bandwidth-bound — vectorization is empirically the paradigm
that hides cache-miss latency, while compilation's edge is on cache-resident
instruction-bound work (the minority here). Do *not* make compilation the primary
model.

**Recommendation — WASM:** **Vectorized, full stop. Never compile.** DuckDB-Wasm
confirms vectorized engines AOT-compile cleanly to `wasm+simd128` with no runtime
codegen, whereas a query-time JIT is impractical to ship in a <1.5 MB bundle and
awkward to host inside WASM. The asymmetry is the key strategic insight:
**vectorization is the JIT-free paradigm, and it is the *same* paradigm that ports
to the browser** — so a single vectorized operator codebase serves both targets,
with native-only compilation layered behind the same VOILA spec. This resolves the
apparent native-vs-wasm fork at zero codebase cost.

**Why not "stay materialising":** the BENCHMARKS.md trajectory shows
micro-optimisation of the materialising evaluator hit diminishing returns and the
gap *grows* with scale; the negative result on the flat `Vec<Id>` row buffer
confirms the row-storage layout is not the lever — **the execution *model* is.**
Staying materialising forecloses every SIMD/compression-exec/semi-join-reducer win
in this report, all of which assume column batches to fire.

---

## 3. Top 5 highest-leverage techniques to try next (with cheap-spike measurements)

Ranked by leverage = (expected win × fit × inverse cost), against the scaling prize.

### #1 — Vectorized scan→filter→merge-join→count pipeline (BARQ blueprint)
- **Why #1:** highest measured ceiling (3.4×–33×), unlocks every other technique
  here, serves *both* targets, and is the M4 plan already half-believed.
- **Cheap spike:** implement ONE vectorized pipeline over columns of u32 +
  selection vector for q04 (5M-row join) and q02 (full scan→count) — the two
  worst remaining 10M losses. **Measure:** compute time vs current materialising
  evaluator and vs native QLever at 10M; per-row CPU cost (cycles/tuple) before/after;
  peak RSS. **Success =** q04 0.29× → ≥0.7× and q02 0.23× → ≥0.7× of QLever, with
  the wasm build of the same pipeline autovectorizing under `+simd128`.

### #2 — Exact bitmap semi-join reducer on dense u32 ids
- **Why #2:** `small` cost, perfect fit for sparq's dense-id dictionary, makes the
  planner's mistakes not matter, composes with both join engines, wasm-strong.
- **Cheap spike:** for a 3–4 pattern selective chain/snowflake BGP, build a bitmap
  over the join variable's reachable id range from the most selective pattern and
  push it as a membership predicate into the other scans, *before* any join.
  **Measure:** surviving rows per pattern and total intermediate-binding count
  before vs after; end-to-end time; bitmap build cost (bytes + ms). **Success =**
  ≥5× intermediate-size reduction on a selective BGP with net end-to-end speedup,
  and graceful no-op on tiny BGPs.

### #3 — Yannakakis full semi-join reduction over the acyclic BGP join tree
- **Why #3:** O(in+out) guarantee on the *common* SPARQL shape (acyclic stars/chains/
  snowflakes), reuses existing sorted-index merge intersections, routes via the GYO
  test sparq already has. The exact, guarantee-carrying complement to #2.
- **Cheap spike:** on a chain BGP whose final result ≪ its largest binary
  intermediate, run a bottom-up + top-down semi-join sweep (merge-intersections of
  sorted id lists) before the joins. **Measure:** max intermediate size and total
  work with vs without the reducer; verify identical result counts (correctness
  gate vs QLever/Oxigraph). **Success =** intermediates bounded by output (no blow-up)
  with ≥2× end-to-end on the multi-pattern case, ~0 regression on single-pattern.

### #4 — Index-Based Join Sampling for cardinality
- **Why #4:** `small` cost, reuses the six indexes, fixes the correlated-multi-join
  mis-orders that NDV/characteristic-sets miss — the failure mode that triggers
  intermediate explosion in the first place.
- **Cheap spike:** spend a fixed ~10k–100k index-lookup budget probing real joins
  for a BGP where GOO currently mis-orders; estimate intermediate cardinalities from
  the survivors. **Measure:** q-error of sampled estimate vs true cardinality
  (recorded from a full run) across a handful of correlated BGPs; planning-time cost
  of the budget. **Success =** ≤2× q-error on ≥80% of BGPs at ≤a few ms planning
  overhead, and at least one plan flip that improves runtime.

### #5 — FastLanes transposed bit-packed column decode kernel
- **Why #5:** directly attacks the scan-bandwidth loss that makes the gap grow with
  scale, shrinks the resident/fetched index (native billions *and* wasm blob), and
  is the rare technique whose entire value prop is "fast via autovectorization on
  whatever target" — sparq's exact situation.
- **Cheap spike:** implement the scalar FastLanes decode for ONE delta-encoded
  permutation column (e.g. the object column of POS) and benchmark decode throughput
  standalone. **Measure:** integers/second decoded on native (x86 + Apple ARM) and
  on `wasm+simd128`; resident bytes vs current `Vec<[u32;3]>`; whether the compiler
  actually autovectorizes (inspect asm / wasm). **Success =** ≥2× decode throughput
  over the plain path and meaningful size reduction, with the *same scalar source*
  vectorizing on all three targets.

**If you can only spike ONE: spike #1 (the vectorized pipeline).** It is the gating
decision for the whole report — almost every other win assumes column batches —
and it is the only one that demonstrably closes the *scaling* gap (not just the
small-data parity sparq already has) while serving both native and browser from one
codebase.

---

## 4. Honest misfits — techniques that do NOT fit an in-memory, bandwidth-bound, correctness-first, WCOJ, browser-targeting engine

- **ROF software-prefetch / coroutine latency-hiding — does NOT fit the *browser*.**
  WASM has no prefetch instruction; this is a native-only win. Keep it native, never
  rely on it for the wasm target. `[M]`
- **Copy-and-Patch / stencil JIT and full Cranelift/LLVM compilation — do NOT fit the
  browser, and compilation does not fit as the *primary* model anywhere here.** The
  workload is bandwidth-bound (vectorization's home); compilation's edge is
  cache-resident compute. Shipping a codegen toolchain in <1.5 MB is impractical, and
  in-WASM self-modifying codegen is constrained. Native-only, opt-in, secondary. `[S]`
- **AVX-512 gather/scatter kernels as a baseline — do NOT fit the portability
  requirement.** The VLDB'23 vectorized-hash-table study + WASM-SIMD reality
  (fixed 128-bit, no gather, silent scalarization of many AVX-512 intrinsics) mean
  gather-based kernels are frequently net-negative *even on x86* and absent in the
  browser. Build 128-bit gather-free kernels; reserve AVX-512 as an opt-in native
  fast path. `[M]`
- **Full FDB factorised-algebra engine — does NOT fit a correctness-first,
  ship-it engine on cost grounds.** Research-grade complexity and a wholesale
  intermediate-representation rewrite. The factorisation *benefit* is real but is
  better harvested via WCOJ tries (already present) + Yannakakis-Expand +
  adaptive-factorization. `[L]`
- **Pure-WCOJ-everywhere — already correctly rejected by the architecture.** Pure
  WCOJ regresses on acyclic low-skew queries (Free Join / GraphflowDB result); the
  GYO-routed hybrid is right. Free Join's value is *unification convenience*, not a
  reason to abandon the hybrid. `[L]`
- **Bloom-based predicate transfer *where exact bitmaps apply* — a partial misfit.**
  The whole Bloom-filter PT/RPT/LIP line exists to avoid exact filters on *wide,
  sparse* SQL join keys. sparq's dense u32 ids invert that premise: prefer the
  **exact bitmap** reducer (#2) and use Bloom/RPT only as the sparse-domain / cyclic
  fallback. Adopting Bloom PT as the default would pay a false-positive-cleanup tax
  sparq doesn't need to pay. `[S]`
- **Heavy precomputed-summary estimators (SafeBound/SumRDF) as the *primary* path on
  the browser — misfit for the 2–4 GB budget.** Keep the *bounds* as a safety net;
  prefer index-sampling + tiny LpBound scalars for the browser; reserve compressed
  degree sequences for the server tier. `[S]`
- **Updates-tier techniques — out of scope for the scaling prize.** Read-optimised
  engine; updates are an M5 overlay and none of the sweep's techniques are
  update-shaped. Noted only for composition (semi-join reducer runs over base+delta). `[S]`

---

## 5. Meta-findings the sweep surfaced (most surprising / strategy-shaping)

1. **The field pivoted from better *cardinality estimation* to *order-robust execution*.**
   The strongest cross-cutting signal: with semi-join reduction (Yannakakis revival —
   Predicate Transfer, RPT, Yannakakis+, Shredded Yannakakis, TreeTracker) join order
   becomes *almost irrelevant* (371×→1.6× variance). For sparq this means the opt-in
   DPccp enumerator (`dp-planner`, OFF by default) + characteristic-set investment is
   **partly substitutable** by a semi-join-reduction layer that makes GOO's mistakes
   cheap. `[M]` <!-- [OPUS-4.8] sq-iywur: DPccp now shipped as an OPT-IN (off-by-default) planner path, not the default; corrected the earlier stale "shipped as default" note. -->
2. **sparq's dictionary encoding — chosen for compression — is a semi-join superpower.**
   The literature settled on *probabilistic Bloom* filters precisely because general
   SQL keys are wide/sparse. sparq's dense u32 ids make the **exact bitmap** variant
   strictly cheaper *and* more accurate. A body of work motivated by avoiding exact
   filters points sparq toward the exact-filter version. `[S]`
3. **Vectorization resolves the native-vs-wasm fork.** Vectorized is the JIT-free
   paradigm *and* the browser-portable one; BARQ proves it works with sparq's exact
   primitives and no codegen. This is asymmetric: **browser MUST be vectorized;
   native MAY additionally compile** — behind one VOILA-style spec, one codebase. `[M]`
4. **The two real levers are bandwidth, not raw SIMD.** Because the workload is
   bandwidth-bound, compression-into-cache (FastLanes decode in-register) and
   latency-hiding (native prefetch) dominate; SIMD mostly speeds compute that is
   already cheap. Prioritise the column model + decode over hand-SIMD kernels. `[M/L]`

---

### Source URLs (deduplicated, preserved)

- Predicate Transfer (CIDR'24): https://www.cidrdb.org/cidr2024/papers/p22-yang.pdf
- Robust Predicate Transfer (SIGMOD'25): https://arxiv.org/html/2502.15181v1 ; https://arxiv.org/pdf/2502.15181 ; https://arxiv.org/pdf/2307.15255
- Yannakakis pre-reducer / acyclic: https://arxiv.org/abs/2504.03279
- Exact bitmap semi-joins ("Not Yannakakis", CIDR'26): https://www.vldb.org/cidrdb/papers/2026/p29-zhao.pdf
- BARQ (vectorized SPARQL, Stardog): https://arxiv.org/html/2504.04584
- Yannakakis+: https://qichen-wang.github.io/files/yannakakis+.pdf ; https://dl.acm.org/doi/10.1145/3725423
- Shredded Yannakakis (SYA): https://arxiv.org/abs/2411.04042 ; https://arxiv.org/pdf/2411.04042
- TreeTracker Join: https://arxiv.org/pdf/2403.01631
- Free Join: https://arxiv.org/pdf/2301.10841 ; https://dl.acm.org/doi/10.1145/3589295
- Free Join → sort-based WCOJ: https://arxiv.org/html/2505.19918v1
- Factorised DB (FDB): http://vldb.org/pvldb/vol5/p1232_nurzhanbakibayev_vldb2012.pdf ; https://fdbresearch.github.io/principles.html
- Adaptive factorization (CIDR'25): https://vldb.org/cidrdb/papers/2025/p21-gro.pdf
- LIP (VLDB'17): https://www.vldb.org/pvldb/vol10/p889-zhu.pdf
- Fair/instance-optimal LFTJ seek: https://arxiv.org/pdf/2510.26016
- LpBound: https://arxiv.org/html/2502.05912 ; https://dl.acm.org/doi/10.1145/3725321
- Wander Join: https://www.cse.ust.hk/~yike/sigmod16.pdf ; https://dl.acm.org/doi/10.1145/3318464.3389702
- Index-Based Join Sampling (CIDR'17): https://www.cidrdb.org/cidr2017/papers/p9-leis-cidr17.pdf
- SafeBound: https://arxiv.org/abs/2211.09864 ; https://dl.acm.org/doi/abs/10.1145/3588907
- Zero-overhead Yannakakis / Diamond Hardened: https://arxiv.org/html/2601.00098 ; https://www.vldb.org/pvldb/vol17/p3215-birler.pdf
- DPhyp: https://resources.mpi-inf.mpg.de/departments/d5/teaching/ss09/queryoptimization/lecture8.pdf ; https://github.com/TantorLabs/pg_dphyp
- IKKBZ + LinDP: https://db.in.tum.de/~radke/papers/hugejoins.pdf
- Mid-query re-optimization: https://arxiv.org/pdf/1902.08291 ; https://arxiv.org/pdf/2202.12535
- FastLanes: https://ir.cwi.nl/pub/32992/32992.pdf
- BitWeaving: https://jigneshpatel.org/publ/BitWeaving.pdf
- Column Sketches: http://daslab.seas.harvard.edu/columnsketches/
- RLE-aware operators (patent): https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/9652501
- SIMD set intersection: https://ashvardanian.com/posts/simd-set-intersections-sve2-avx512/ ; https://dl.acm.org/doi/10.14778/2735508.2735518
- Vectorized vs compiled (Kersten): https://www.vldb.org/pvldb/vol11/p2209-kersten.pdf ; https://clickhouse.com/resources/engineering/vectorized-query-execution
- VOILA: https://t1mm3.github.io/assets/papers/vldb21.pdf
- Copy-and-Patch: https://sillycross.github.io/assets/copy-and-patch.pdf
- Relaxed Operator Fusion: https://db.cs.cmu.edu/papers/2017/p1-menon.pdf
- Micro-adaptivity / CAKE: https://arxiv.org/pdf/2602.04181
