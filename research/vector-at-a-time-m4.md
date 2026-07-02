<!-- [OPUS-4.8] Design record (design-for-review, epic sq-pntvh). Authored under the
     empirical-honesty mandate: every load-bearing claim traces to the actual code
     (file:line) or a cited prior design record; implemented / designed-only / proposed /
     not-yet-a-win are kept DISTINCT; NO benchmark numbers appear (none have been run for
     this path and work-box timings are non-canonical); no ZK/MPC privacy or soundness
     claim is made. This is a plan for the maintainer to review, not a shipped feature. -->

# Vector-at-a-time (M4) execution: operator model, wiring, and coexistence

**Status:** design record, 2026-07-01. Epic **sq-pntvh**. DESIGN + DECOMPOSE only — no
implementation lands with this document. It turns the "morsel operator model + row/columnar
coexistence + bit-identical-when-OFF" epic into a reviewed plan plus a dependency-ordered set
of impl beads.

This record builds on the maintainer's own execution-architecture decision in
[`optimization-techniques.md`](optimization-techniques.md) §2′ (the **BARQ blueprint** —
columns of dictionary-encoded ids + selection vectors + skip-aware merge joins, *no JIT*),
its constraints (**autovectorization-only, small wasm bundle, 128-bit gather-free kernels**),
and the first columnar block that landed under **sq-hvfe**. It does **not** re-argue
"vectorize vs compile" — that is settled there; this record is the *how-to-wire-it* layer.

---

## 0. What actually exists today (verified against the code, not the brief)

The brief's premise — *"the columnar `chunk.rs` kernels are UNREFERENCED in `exec.rs`, so the
evaluator is still 100% row-materialising"* — is **correct**, and I confirmed it directly. Two
refinements the brief does not state are load-bearing for the design, so I record them here.

**Confirmed: the columnar block is inert.**

- [`crates/sparq-engine/src/chunk.rs`](../crates/sparq-engine/src/chunk.rs) defines
  `DataChunk` (column-major `Vec<Vec<Id>>` + `len`), `VecCmp` (numeric comparison mirroring
  the row `NumCmp`, NaN-rejecting), `SelVec = Vec<usize>`, the comparison kernel
  `select_numeric`, and the gather kernel `apply_selection`, plus the round-trip
  `from_rows`/`to_rows`.
- It is behind the **`vectorized`** cargo feature, **OFF by default**
  ([`Cargo.toml`](../crates/sparq-engine/Cargo.toml) `vectorized = []`; default =
  `["parallel", "regex", "digest"]`) and registered only under that gate
  ([`lib.rs`](../crates/sparq-engine/src/lib.rs) `#[cfg(feature = "vectorized")] pub mod
  chunk;`). Outside `chunk.rs`, the only references to `DataChunk` / `select_numeric` are the
  gated re-export ([`lib.rs`](../crates/sparq-engine/src/lib.rs) `#[cfg(feature =
  "vectorized")] pub use chunk::{DataChunk, SelVec, VecCmp};`) and an isolation test
  (`tests/vectorized_exec_differential.rs`) — **the evaluator (`exec.rs`) never constructs a
  chunk or calls the kernel.** The intermediate relation everywhere is
  `Bindings { vars: Vec<Variable>, rows: Vec<Row> }` with `Row = SmallVec<[Id; 4]>`
  (`exec.rs:788,937`). So: **row-materialising, confirmed.**

**Refinement 1 — the row evaluator is NOT naïvely un-optimised: it already does term-free,
id-level numeric filtering *inside the scan*.** `ScanCmp::test_id` (`exec.rs:3013`) evaluates a
pushed-down `?v OP constant` predicate through `graph.numeric_value(id)` — O(1), *no term
materialised* — and `split_sargable` (`exec.rs:3134`) pushes such filters into the scan before
rows are ever built. This matters because it **relocates the columnar win**: the pushed-down
numeric FILTER is *already* cheap and column-shaped. The genuinely row-bound losses are (a) the
**residual** filter over a *materialised join result* — `apply_filter` (`exec.rs:6501`) walks
`b.rows` one at a time dispatching the whole expression tree through `eval_expr` per row — and
(b) **aggregate reducers** — `group_aggregate` (`exec.rs:5862`) calls `eval_aggregate` per group
over member rows. **That is where a vector path pays, not in the already-pushed-down scan.**

**Refinement 2 — the landed `select_numeric` kernel is a correctness *scaffold*, not yet a SIMD
win, and the design must say so.** `select_numeric` loops the id column calling
`graph.numeric_value(id)` per element (`chunk.rs:186`). For the dense caches
(`NumData::Owned(Vec<f64>)` / `Mapped`, `sparq-core/src/lib.rs:162`) that is a **random-access
gather** `slice[id - 1]` — the *same* cache-miss access pattern as the row path, and **not
auto-vectorisable** (a gather plus a branch, not a straight-line load). It is functionally
equivalent to the existing `ScanCmp::test_id` loop; it does not, as written, unlock SIMD. The
real SIMD enabler is a **decode step** that gathers the id column into a *contiguous* `Vec<f64>`
value column **once**, after which a branchless compare over contiguous `f64` auto-vectorises
(NEON/AVX on native, `+simd128` on wasm). There is exactly one case that skips the gather
entirely: **inline-integer ids** carry their value in the id itself
(`numeric_value` returns `(id - INLINE_BASE) as f64`, `sparq-core/src/lib.rs:2414`), so a SIMD
compare *directly over the id column* is gather-free and portable — the natural first target.

**Net:** the brief's premise stands; the two refinements move the design's centre of gravity to
(i) **residual filter + reducer aggregate**, not scan, and (ii) building the **decode-to-
contiguous-column primitive** as the actual SIMD enabler rather than treating the landed
`select_numeric` as if it were already one.

---

## 1. The operator model — morsel + row/columnar coexistence (not a rewrite)

The design goal is a path that **opts into columnar where it wins and runs the exact existing
row code otherwise** — coexistence, not replacement. Two seams achieve this at very different
risk/reward, and the plan uses them in sequence.

### 1.1 The interchange invariant

`Bindings` stays the operator-boundary type. `DataChunk` is defined as its **column-major
transpose of a slice of rows** — `from_rows` / `to_rows` are already the identity round-trip
(`chunk.rs:122-150`, with a test). This is the coexistence contract in one sentence: **any
columnar sub-pipeline is spliced between two `Bindings` boundaries; its `to_rows()` output is
consumed by the unchanged row evaluator.** Nothing downstream needs to know a columnar kernel
ran.

### 1.2 Seam A — local columnar *refinement* of one operator (retrofit, low risk)

Inside a single heavy operator (`apply_filter`, `group_aggregate`), when an **eligibility
predicate** says columnar wins, transpose `b.rows → DataChunk` once, run the columnar kernel,
and produce the result (a `SelVec` to retain, or a reduced value) — otherwise run the existing
row loop verbatim. This is a *local* opt-in with **no evaluator restructuring**; it is exactly
what the `chunk.rs` equivalence contract was built to enable. Its weakness is that it pays the
transpose **per operator** (§4), so it is only ever entered for operators heavy enough to amortise
it.

### 1.3 Seam B — the morsel pull-pipeline (the real M4, higher reward, later)

A pull-based operator tree (`VecOp::next() -> Option<DataChunk>`) that keeps data **columnar
across** scan → filter → project → partial-aggregate, materialising to rows **only at the
pipeline boundary**. Fixed **morsel size** (DuckDB uses 2048; pick a power of two) keeps
intermediates and selection vectors cache-resident. This is where the transpose tax disappears
(one decode in, one materialise out, not per-operator) and where BARQ's measured ceiling lives —
but it is a genuine new operator layer, so it is phased *after* Seam A has proven the
equivalence harness and measured a real win.

### 1.4 The dispatcher (the coexistence gate)

A cheap, side-effect-free `columnar_eligible(op_shape, col_dtype, batch_len) -> bool` predicate
decides per operator instance. It defaults to **false** (row path). It returns true only when
all hold: the operator shape is one with a columnar kernel; the column is a decodable dtype
(numeric / temporal / inline-int); and `batch_len ≥ threshold` so the transpose/decode amortises.
**Row is always the fallback for everything the predicate declines** — property paths,
string/REGEX filters, heterogeneous columns, tiny results. This predicate *is* the
row/columnar boundary and is the single place the two models meet.

---

## 2. Which operators get columnar kernels first, and the win each unlocks

Ranked by (expected win × fit × inverse risk). For each, the honest "where it is neutral or
loses" is stated inline — the full-detail limitations are §4.

| # | Operator | Columnar kernel / win | Where it does NOT win |
|---|----------|----------------------|------------------------|
| 1 | **Decode primitive** (enabler, not an operator) | Gather the id column → contiguous `Vec<f64>` (NaN sentinel) **once**; inline-int columns skip the gather (value is in the id) → a branchless, auto-vectorising compare. This is the thing that makes every kernel below actually SIMD. | The gather pass itself is still random-access; it only pays when the decoded column is *reused* by ≥1 downstream compare/reduce, or the column is inline-int. |
| 2 | **Residual FILTER** (`apply_filter`, post-join) | `select_numeric` over the *decoded* column → ascending `SelVec` → `apply_selection`. Replaces the per-row `eval_expr` dispatch (`exec.rs:6530`) with one tight compare loop. Biggest clear win: the residual filter is the row-bound loss the pushed-down scan filter already avoids. | Only for **residual** numeric/temporal comparisons. String/REGEX/`STR()` filters and multi-branch boolean trees stay row. A tiny post-join result loses to the transpose. |
| 3 | **Reducer AGGREGATE** (`group_aggregate`) | SUM / COUNT / MIN / MAX / AVG over a contiguous value column auto-vectorise to a SIMD add/min/max reduction. Strong win for whole-dataset aggregates and **low-cardinality** GROUP BY (reduce dominates). | **High-cardinality** GROUP BY is hash-partition-bound, not reduce-bound — columnar helps the per-group reduce, not the grouping. GROUP_CONCAT / SAMPLE / custom aggregates stay row. |
| 4 | **PROJECT** (`project_bindings`) | Column select/reorder is an O(width) move of column `Vec`s + a threaded selection vector, vs the current O(rows × width) per-cell rebuild (`exec.rs:6332`). Nearly free and exact. | Projection that introduces `NO_ID` (unbound) columns needs sentinel handling; still cheap but not zero. |
| 5 | **Skip-aware merge-JOIN** (BARQ) | Probe/Build/Skip over sorted permutation slices via `partition_point` seeks, staying columnar. Largest ceiling per §2′. | Largest change; only after the pipeline (Seam B) exists. Cyclic BGPs keep WCOJ/LFTJ; property paths stay row. |

Scan is deliberately **not** first: the sargable numeric/temporal FILTER is *already* pushed
into the scan term-free (§0, refinement 1), so "columnar scan-filter" would mostly re-implement
an existing win. The scan's columnar contribution is instead **feeding the decode primitive
(#1)** a contiguous id column.

---

## 3. HARD constraint: bit-identical output + byte-identical bundle when the feature is OFF

This is a merge-blocking property, not an aspiration. Three mechanisms enforce it.

**3.1 Feature gate = the bundle proof.** Everything columnar stays behind `vectorized`
(default OFF), the pattern `chunk.rs` already follows. When OFF, **zero columnar code compiles**,
so the browser artifact is byte-for-byte unchanged and the deterministic
`wasm_bundle_bytes` metric that `scripts/perf-gate.py` hard-gates (2%, auto-ratchet;
`bench/perf-baseline.json`) **cannot move**. No new *default* dependency may be added (the
landed block already adds none — only `sparq_core::{dict::Id, Graph}`, existing deps). The wasm
build profile must keep `vectorized` OFF even if a future phase turns it on for native (wasm is
gather-poor and single-threaded; §4), so the default browser bundle is protected by
construction.

**3.2 Bit-identical output = a differential harness, landed FIRST (Phase 1).** Each columnar
operator is an **equivalence-checked accelerator**: for a query corpus, running with the feature
ON must yield **byte-identical SPARQL-JSON** to the feature-OFF row path. This is the analogue of
the existing `query_json_chunks_concat_is_byte_identical` test (`lib.rs` tests). The subtle
correctness surface the harness must pin:

- **NaN / type-error semantics** — `VecCmp` already mirrors the row operators' IEEE-754
  rejection of NaN (`chunk.rs:64`, tested), matching a FILTER type-error → row excluded.
- **Row order** — the row path preserves first-seen order (DISTINCT, group order via
  `build_groups`, ORDER BY stability). `SelVec` is ascending and `apply_selection` is
  order-preserving (`chunk.rs:199`), and transpose is order-preserving — so the invariant holds,
  but it is the easiest thing to break and the harness must assert it explicitly.
- **Unbound handling** — `NO_ID` columns from OPTIONAL/PROJECT must decode to "not numeric" (no
  pass), never to a spurious value.
- **Budget + cooperative cancellation** — the row loops call `budget::exhausted`; a columnar
  kernel over a large batch must check the budget at a batch/selection boundary so a
  capped/timed-out query truncates identically.
- **ZK-trace coupling (soundness-relevant, do not miss)** — `apply_filter` records a
  `record_filter` obligation per row under the `zk` feature (`exec.rs:6552`). A columnar filter
  path MUST still emit the identical obligation set, or the ZK witness is incomplete. If the
  `vectorized` and `zk` features are both on, the columnar filter must fall back to (or
  additionally run) the obligation-recording path. Simplest safe rule for Phase 3: **when `zk`
  tracing is armed, the columnar filter seam is disabled** and the row path runs — documented,
  not silent.

**3.3 Perf-neutral-when-OFF = a CI proof, not a promise (Phase 8).** A CI check builds the
default (feature-OFF) workspace and asserts the deterministic artifacts (`wasm_bundle_bytes` and
a native deterministic byte metric) are unchanged versus `origin/main`'s floor — i.e. the same
ratchet `perf-gate.py` already runs, plus an explicit "feature-off diff" so a reviewer sees the
zero-delta rather than inferring it.

---

## 4. Honest risks and limitations

**Transpose / boundary cost is real and per-operator in Seam A.** `from_rows` / `to_rows` are
full O(rows × width) copies (`chunk.rs:122,142`). For a small `Bindings`, or an operator the
kernel only marginally accelerates, the transpose **dominates and the columnar path is a net
loss** — hence the eligibility threshold, and hence Seam A is only worth it on heavy operators
while the pipeline (Seam B) is what actually removes the tax.

**The gather problem is the central technical risk.** As landed, `select_numeric` re-gathers
`numeric_value(id)` per element — same random access as the row path, no SIMD (§0, refinement 2).
Every claimed SIMD win in §2 is **contingent on the decode primitive (#1) landing first** and on
the decoded column being reused or the column being inline-int. A phase plan that wired filter/agg
seams *before* the decode primitive would ship a columnar path that is bit-identical but **not
faster** — the plan orders decode first precisely to avoid that.

**Memory overhead.** `DataChunk` is `Vec<Vec<Id>>` — one allocation per column plus the outer
`Vec`, plus a `SelVec` and a decoded `Vec<f64>` per batch. For **wide, short** intermediates this
is *more* allocation than the row `SmallVec<[Id; 4]>`, which inlines ≤4 columns with zero heap
traffic. Morsel batching (Seam B) caps peak footprint to one batch, but adds a per-column decode
buffer. Net: columnar trades per-row dispatch for per-column buffers — a win only when rows ≫
width and the batch is large.

**Where columnar loses outright (stays row, by design):**

- **Property paths** — recursive, row-shaped; `eval_path` is not a batch operator.
- **String / heterogeneous operators** — REGEX, `STR*`, `CONCAT`, lang/datatype tests do not
  vectorise into a numeric column.
- **Tiny / point queries** — the browser's `single_pattern_scan_json` fast path
  (`exec.rs:1056`) already bypasses `Bindings` entirely; the columnar path must **not** intercept
  or regress it.
- **High-cardinality GROUP BY** — hash-partition-bound; the reducer win is second-order.
- **OPTIONAL / MINUS null semantics** — `NO_ID` sentinels complicate column kernels; keep row
  until the pipeline handles nulls explicitly.

**wasm asymmetry.** No gather intrinsic and single-thread by default, so on `wasm+simd128` only
the **inline-int (gather-free)** compare and the **contiguous-decoded** compare/reduce actually
SIMD; AVX-512 gather kernels are rejected per §2′. The design must not assume a native gather win
transfers to the browser.

**Maintenance / two implementations.** Every columnar operator is a *second* implementation that
must track the row path's exact semantics (NaN, type errors, order, budget, and the ZK-trace
obligation coupling above). This is the standing cost of coexistence and the reason the
differential harness (Phase 1) is a hard prerequisite, not a follow-up.

---

## 5. Recommendation

Proceed **phased and retrofit-first**, gated on the equivalence harness as the acceptance gate:

1. Land the **differential byte-identity harness + a native criterion bench** *before* any
   operator seam — it is the safety net every later phase leans on, and the bench is how a real
   (non-work-box, non-canonical here) win is proven rather than asserted.
2. Build the **decode-to-contiguous-value-column primitive** (with the inline-int gather-free
   fast path) early — it is the actual SIMD enabler; the landed `select_numeric` is only a
   scaffold without it.
3. Retrofit **residual FILTER** and **reducer AGGREGATE** via Seam A behind the feature, each
   equivalence-tested and each preserving order / budget / ZK-trace coupling.
4. Only once Seam A has measured a real win: build the **morsel pull-pipeline** (Seam B),
   **project + selection-vector threading**, and finally the **skip-aware merge-join**.
5. Keep `vectorized` **non-default** (never default-on for wasm) until a native benchmark shows a
   real win; the bundle stays byte-identical the whole way.

Everything above is opt-in and reversible: if a phase does not measure a win, it stays behind the
feature and the default build is unchanged.

---

## 6. Phased plan (each phase is a future bead under sq-pntvh)

Dependency-ordered. Phase *n* depends on phase *n−1* unless noted; all depend on the epic.

1. **Phase 1 — equivalence + bench scaffold.** A `vectorized`-gated differential harness
   asserting byte-identical SPARQL-JSON (feature ON vs OFF) over a query corpus, plus a native
   criterion bench for residual-filter/aggregate over a synthetic column. The acceptance gate for
   every later phase. *(dep: epic)*
2. **Phase 2 — decode primitive.** `DataChunk::decode_numeric_column` (id column → contiguous
   `Vec<f64>` NaN-sentinel) + the inline-integer gather-free fast path; unit-tested like the
   existing `chunk.rs` kernels. The real SIMD enabler. *(dep: Phase 1)*
3. **Phase 3 — columnar residual-FILTER seam** in `apply_filter` behind the feature: eligible
   single sargable numeric/temporal residual → transpose → `select_numeric` over the decoded
   column → `apply_selection`, else row fallback. Preserves order, budget, and the ZK-trace
   obligation (disable the seam when `zk` tracing is armed). *(dep: Phase 2)*
4. **Phase 4 — columnar reducer aggregates** (SUM/COUNT/MIN/MAX/AVG) over the decoded column in
   `group_aggregate` for whole-dataset and low-cardinality GROUP BY, first-seen order preserved.
   *(dep: Phase 2; may run parallel to Phase 3)*
5. **Phase 5 — columnar PROJECT + selection-vector threading**: project as column moves; thread
   `SelVec` through filter → project without re-gather; land the `columnar_eligible` dispatcher +
   morsel batch-size constant. *(dep: Phase 3)*
6. **Phase 6 — morsel pull-pipeline** (`VecOp::next() -> Option<DataChunk>`) keeping scan →
   filter → project → partial-aggregate columnar across operators, materialising to rows only at
   the boundary; wired for conjunctive BGP+filter shapes behind the feature. *(dep: Phase 5)*
7. **Phase 7 — skip-aware vectorized merge-JOIN** (BARQ) over sorted permutation slices via
   `partition_point` seeks, inside the pipeline. Largest; may be split when scoped. *(dep: Phase
   6)*
8. **Phase 8 — perf-neutral-when-OFF CI proof**: a feature-OFF build + deterministic-artifact
   diff (wasm bundle + native byte metric) asserting zero delta vs the floor, making §3.3 a gate
   rather than a promise. *(dep: Phase 1; runs alongside the operator phases)*

---

## 7. Open questions for the maintainer

1. **Default-on threshold.** What measured native win (and on which benchmark queries) justifies
   promoting `vectorized` from opt-in toward default-on for *native*? (wasm stays OFF regardless.)
   The plan deliberately leaves this to a measured decision.
2. **ZK-trace × columnar filter.** Phase 3 proposes the simplest-safe rule — *disable the
   columnar filter seam whenever `zk` tracing is armed* — so the obligation set is always complete.
   Is that acceptable, or is a columnar path that *also* records obligations worth the extra
   complexity later?
3. **Morsel size.** Adopt a fixed 2048 (DuckDB) or make it a tuned constant per target
   (native vs wasm cache sizes)? Phase 5 needs the number pinned.
4. **Scope of Phase 7 (merge-join).** Land it as one bead or split Probe/Build/Skip + the WCOJ
   coexistence boundary into sub-beads when scoped? It is the one "large" item here.
