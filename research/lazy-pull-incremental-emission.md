# Lazy-pull / incremental-emission execution for SELECT — design record

> 🤖 SPARQ agent — Fable-tier architect design deliverable for **sq-7d3dj.34.2**
> (D9c TTFB: first solution of a large SELECT). [FABLE-5]

**Status: design + decomposition (no production code in this record). Amended
2026-07-12 (decision `sq-7d3dj.34.2.7`, [FABLE]):** implementation of `.2.3` verified
that the buffered `bind_join` emits in `FxHashMap` group-iteration order (§2), so the
original "block-slice the seed" driver could NOT satisfy the original §8.1 byte-identity
for bind-join chains. Adjudicated resolution: the driver keeps block-slicing and the
emission-order contract for admitted join chains is **relaxed to deterministic seed-lex
order** (§5.1); byte-identity is retained only where structurally replicable. §5/§6/§8
and the §10 bead specs are amended in place; the superseded text is not preserved
inline (git history has it).

This is the
engine-side counterpart of `research/wave-d-pull-streaming-response-body.md` (Wave D):
Wave D designed the *transport* (emit sink → bounded channel → chunked HTTP body), and
PR #1745 implements its core; this record designs the *evaluate-phase* incremental
execution model that Wave D's tier "S3 — lazy general operators" explicitly deferred,
so that the first **solution** of a large SELECT is emitted long before the full result
is computed. It corrects two premises along the way (§3) and supersedes the deferred
bead `sq-7d3dj.29` with a concrete, non-columnar plan.

Measured numbers cited here come from the canonical D9/D10 HTTP panel
(`research/perf-dominance-gap-2026-07-http-addendum.md`, PR #1742, EC2 same-box). No
number in this record is a new claim; every target is a goal to be measured by the
canonical re-gather bead (§10), never asserted in advance.

---

## 1. The problem, after PR #1745

The canonical D9c measurement: on SP2Bench q04 (541 911 rows, SELECT DISTINCT over a
large join) sparq-server has the **fastest full request** of all five engines measured,
but its **first response byte arrives at ~87 % of total time** (372 ms of 428 ms) —
oxigraph's first byte arrives at 7.8 ms (48× earlier), qlever's at 89 ms. For streaming
and early-abort consumers, time-to-first-solution is a real BEHIND axis
(addendum §4, verdict row D9c).

PR #1745 (open at time of writing) fixes the **serialize half**: it refactors
`eval_select_json_chunks` into an emit core (`exec::eval_select_json_emit`, a
`FnMut(String) -> ControlFlow<()>` sink), adds `query_json_stream_with_budget` to the
engine API, and gives `sparq-server` a `stream_select_json` path (blocking worker →
bounded `mpsc(4)` → `Body::from_stream`, first-chunk-buffered so small results keep
`Content-Length` and pre-first-byte failures keep their correct status). Concatenation
is byte-identical to the buffered path.

**What #1745 does NOT fix — the subject of this record:** on the *general* path,
`eval_select_json_emit` still calls `eval_modified(...)`, which **fully materialises the
entire `Bindings` before the first emit**. Only the single-pattern-scan fast path emits
during evaluation. So for q04's shape (DISTINCT over a multi-pattern join), the first
solution still waits for the whole join + dedup to finish; #1745 moves TTFB from
"materialise + serialise" to "materialise", which is most of the 372 ms. The
evaluate-phase gap is an *architecture* property of the relation-at-a-time evaluator,
not a serialisation detail.

---

## 2. Verified estate (read, not assumed)

**The evaluator is relation-at-a-time.** `exec::eval_graph_pattern_inner` (the pattern
dispatcher) and `exec::eval_modified` (the solution-modifier wrapper) each return a
fully materialised `Bindings { vars: Vec<Variable>, rows: Vec<Row>, sorted_by:
Option<Variable> }`, `Row = SmallVec<[Id; 4]>`. Rayon parallelism enters per-operator
(term materialisation, BGP row building, FILTER/BIND/ORDER BY expression loops, the
radix-partitioned hash-join build), always with order-preserving collection — the
parallel output is byte-identical to the serial output.

**The BGP join is a greedy left-deep chain** (GOO, `exec.rs` ~4400–4520): a
materialised **seed scan** (`scan_to_bindings`), then per step one of
`bind_join` (index-nested-loop probe of the next pattern — driven row-by-row by the
running result), `merge_join` (both sides sorted), or `hash_join` (rhs scanned then
joined). This left-deep, seed-driven shape is the structural reason a streaming path
can *reuse* the existing probe machinery block-by-block instead of rewriting operators.

**Two structural blockers for byte-identity, verified:**

1. `hash_join` (exec.rs:5617) picks its build side by comparing **actual** row counts
   (`left.rows.len() <= right.rows.len()`). A streaming path cannot know the driving
   side's total count without materialising it, so the buffered path's emission order
   for hash-step plans is not replicable from a stream.
2. `bind_join` (exec.rs:8058–8100; found during `.2.3` implementation, adjudicated in
   `sq-7d3dj.34.2.7`) groups ALL result rows into an `FxHashMap<join_value,
   Vec<row_idx>>` and iterates `for (val, ris) in groups` — **hash-bucket order over
   the full key set**, not seed/insertion order — returning `Bindings::unsorted` with
   no post-sort (project/distinct/slice all preserve that order). Slicing the seed
   into blocks regroups over each block's key subset, so a join value whose seed rows
   span a block boundary has its output rows split across the stream — bytes differ
   from the buffered path, guaranteed on any large fan-out result (the q04 class).
   Counterexample: seed join values block₁ = [A, B], block₂ = [A, C]; buffered emits
   all A-rows contiguously, block-sliced emits A(b₁), B, A(b₂), C.

§5.1 turns both into an explicit order-contract fork. Note the buffered `bind_join`
order being given up is an *incidental* artifact of fxhash values and table growth —
deterministic per build but semantically meaningless, unstable under a `rustc-hash`
bump or any result-set change, and destructive of seed sortedness. No consumer can
meaningfully rely on it (§5.1).

**In-flight exec.rs chain (designed-against, per their PR diffs):**

- **#1741 (SIP correlated join + top-k ORDER BY)** — adds a `sip` module, a
  `try_sip_join` call in the Join arm, `order_bindings(.., row_budget)` with a
  quickselect top-k, `SortCell::Iri`. Still materialises every intermediate.
- **#1747 (DISTINCT-projection loose skip-scan)** — adds `try_distinct_pushdown` in the
  Distinct arm of `eval_modified`: `DISTINCT { PROJECT ?v { BGP/UNION } }` enumerates
  distinct ids straight off a permutation scan (`scan_perm`). Where it fires it
  *removes* the join entirely — complementary to this design (a shape it covers never
  reaches the streaming pipeline; a shape it rejects, e.g. multi-var DISTINCT like q04's
  two-variable projection, falls through to us).
- **#1718 (compiled expressions)** — `CompiledExpr` / `compile_expr` hoist
  variable→column resolution out of FILTER/BIND/ORDER BY row loops. The streaming
  path's per-block residual filters should consume `CompiledExpr` directly.

None of the three changes `eval_select_json_emit` or the JSON emit region; the only
textual-adjacency risk for this program is #1745 itself (which must land first — this
design consumes its sink).

**M4 / columnar status (the gate matters):** the Seam-A EC2 measurement
(`sq-pntvh.9`, closed) returned **NO-WIN**; per the roadmap's own gate,
`sq-pntvh.6` (columnar morsel pull-pipeline, `VecOp::next() -> Option<DataChunk>`) and
`sq-pntvh.7` **stay deferred**. Consequence: **this design must not ride the columnar
pipeline.** It is specified at row level over the existing kernels; if Phase 6 ever
clears its gate, a columnar producer can slot behind the same classifier (§5) without
changing the contract. (The M4 roadmap reserved the module name `src/pipeline.rs` for
Phase 6 — this program deliberately uses `src/stream_pipeline.rs` to avoid squatting.)

**Wave-D bead reconciliation** (so the backlog stays honest):

| bead | Wave-D scope | status after #1745 |
|---|---|---|
| sq-7d3dj.24 (emit seam) | `query_json_stream_with_budget` over a FnMut sink | implemented by #1745 — close when it merges |
| sq-7d3dj.25 (bounded-channel body) | spawn_blocking → mpsc → Body::from_stream | implemented by #1745 (simpler framing: single-chunk = buffered + Content-Length) — close when it merges |
| sq-7d3dj.26 (truncation trailer) | unterminated-body floor + `X-Sparq-Truncated` trailer | floor mechanism holds in #1745 (an aborted chunked stream never gets its terminating zero-length chunk); the *trailer* remains open |
| sq-7d3dj.27 (lazy single-pattern scan) | emit per 64 KiB during the scan | largely implemented by #1745 for the borrowed-`Cow` scan; re-verify then close |
| sq-7d3dj.28 (streaming admission / pin cap) | bound held generations under slow clients | still open, unchanged by this record |
| sq-7d3dj.29 (lazy general operators, deferred to sq-pntvh) | evaluate-phase TTFB for join shapes | **superseded by this record** — the sq-pntvh vehicle is gated NO-WIN; closed in favour of the child beads in §10 |

---

## 3. Premise corrections (honest re-framing)

1. **Hash-DISTINCT is NOT a pipeline breaker.** Both the epic framing ("queries whose
   plans require pipeline-breaking operators — DISTINCT over a large join — can't
   stream") and Wave D §3 ("hash-`DISTINCT` … fundamentally non-incremental") are wrong
   on this point: a first-seen hash `DISTINCT` emits every row the moment it is first
   seen — the first output row is correct immediately, and `SELECT DISTINCT` without
   `ORDER BY` guarantees no order. The genuinely blocking operators are full `ORDER BY`,
   blocking aggregation, and sort-based dedup. **q04's shape is streamable.** The
   actual blocker for q04 is the relation-at-a-time **join materialisation**, and that
   is fixable without touching operator semantics.
2. **The columnar morsel pipeline is not the vehicle.** Wave D deferred evaluate-phase
   streaming to the M4 pull pipeline; the Seam-A NO-WIN verdict honestly defers that
   pipeline. Row-level block streaming over the existing scan/probe kernels is
   independent of the columnar bet and available now.
3. **TTFB alone is gameable — the honest metric is time-to-first-solution.** A server
   can emit the status line + JSON `head` before evaluating anything. That would beat
   the D9c number without helping any real consumer. This program's metric is
   **TTFS** (request → first byte of the first `bindings` element), measured alongside
   TTFB; the design does emit the head early (it is statically known for the classified
   shapes), but no bead may claim the D9c win on head-emission alone.

---

## 4. Options considered

| option | verdict | why |
|---|---|---|
| A. Volcano-style row `Iterator` operators | **rejected** | already rejected by Wave D §8 for the seam and doubly so for the whole tree: the engine's thread-local budget/view guards and per-operator rayon parallelism do not survive re-entrant pull; a serial pull tree would also sacrifice the parallel totals that won D9a/D9e (the mandate forbids regressing totals to buy TTFB — that is oxigraph's trade, 48× earlier first byte but 42× slower total). |
| B. Columnar morsel pull pipeline (`sq-pntvh.6`) | **deferred, not available** | gated NO-WIN by the Seam-A measurement; riding it would couple a P1 measured gap to a deferred bet. |
| C. Full push-based rewrite of `eval_graph_pattern` | **rejected** | months of churn across 13 k lines under an active optimisation chain; the D9c gap does not need it — the top-of-plan shapes that matter are enumerable. |
| D. **Classified streaming fast path over the landed emit sink** | **chosen** | mirrors the proven `single_pattern_scan_json_emit` pattern: a classifier admits plan shapes it can stream *correctly*, everything else falls back to the buffered path unchanged (fail-closed). Bounded new code in one module; reuses the existing scan/probe/filter kernels block-by-block; composes with #1741/#1747/#1718 untouched. |

---

## 5. The execution model (option D, specified)

New module `crates/sparq-engine/src/stream_pipeline.rs`; one hook in
`eval_select_json_emit` after the single-pattern fast path:

```text
eval_select_json_emit(graph, pattern, flush, emit):
  1. single_pattern_scan_json_emit(...)        (landed, #1745)
  2. stream_pipeline::try_stream_json_emit(...)   ← THIS PROGRAM
  3. buffered general path (eval_modified → serialise chunks)  (unchanged fallback)
```

**Classifier.** `try_stream_json_emit` pattern-matches the *top* of the plan:

```text
Streamable ::= [Slice] [Distinct|Reduced] Project (BGP+filters | Union(Streamable-BGP…))
```

over a BGP whose GOO join chain it re-derives with the same planner calls
(`goo_seed`/`goo_pick`, shared with the T22 EXPLAIN dry-run). Admission is decided by
the **step kinds** in that chain (below). Anything unrecognised — BIND / OPTIONAL /
MINUS / property paths / GROUP BY / VALUES / subqueries / named-graph forms / full
ORDER BY (v1) — returns `None`: the buffered path runs exactly as today. Fallback is
also forced whenever a zk-trace recorder is armed (the recorder's scan-completeness
contract requires the recording Bindings path; no zero-knowledge property is claimed or
altered by this design — the v1 verifier's external cryptographer sign-off remains
pending, `sq-qhy4`) and whenever EXPLAIN/analyze instrumentation is active.

**Driver.** The seed scan is consumed in **blocks** with an adaptive ramp: first block
`STREAM_BLOCK_FIRST` (design parameter, ~4 096 rows) doubling to `STREAM_BLOCK_MAX`
(~65 536) — small first block ⇒ first solutions in microseconds-to-milliseconds of
engine work; large steady-state blocks ⇒ per-block overhead amortised to noise. For
each block, the chain steps are applied with the existing kernels:

- `bind_join` step → probe the block's rows through the pattern's index (unchanged
  match semantics), **emitting in seed-lex order** (§5.1): within a block the
  `FxHashMap` grouping is used ONLY as a scan cache (one index scan per distinct join
  value, matches cached), then the block's rows are walked **in order**, each row's
  matches appended contiguously in scan order — NOT the buffered kernel's
  hash-bucket-order group loop;
- `hash_join` step → the rhs scan is materialised ONCE up front as the build side (rhs
  is an independent single-pattern scan — cheap and size-known), the block probes it
  (the `sjoin` substrate's `probe_emit`, posting lists in build-row order);
- `merge_join` step (v1): **not admitted** (the running side's sortedness interacts
  with blocking; revisit after v1 evidence);
- residual FILTERs → per-block over `CompiledExpr` (#1718's hoist, compiled once);
- `Project` → column selection (statically known head ⇒ the JSON `head` is emitted
  before the seed scan starts);
- `Distinct`/`Reduced` → a persistent first-seen hash set across blocks, emitting
  survivors immediately;
- `Slice` → early-exit: stop the driver the moment LIMIT is satisfied (a free
  early-abort win the buffered path cannot have).

Serialisation appends each surviving row to the pending chunk and hands it to `emit`
at every `flush` boundary — identical byte layout to the buffered serialiser.
`budget::check` runs per block (the same cooperative pattern #1745 uses per 1024
scanned rows); `ControlFlow::Break` from the sink aborts the driver (client gone).

**Parallelism without losing order.** Within a block, row building/probing/filtering
uses the existing order-preserving `par_iter().filter_map(..).collect()` pattern
(worker budget snapshots re-installed exactly as `hash_join`'s parallel probe does
today); blocks are emitted strictly in seed-scan order and rows within a block in
seed-lex order (§5.1) — the parallel collect must preserve that order, exactly as the
buffered kernels' parallel paths already do. Steady-state throughput ≈ the
buffered path (same kernels, same parallelism, one extra sequence point per block);
the differential and bench beads (§10) verify "≈" instead of asserting it.

### 5.1 The emission-order contract (decision `sq-7d3dj.34.2.7`, amended)

The record originally forked the contract by *step kind*: bind-join chains
byte-identical (old M1), hash-step plans order-relaxed (old M2). §2's blocker 2 proved
the bind-chain half unimplementable under block-slicing, leaving two candidate
resolutions, adjudicated as follows.

**Rejected — final-join group-loop replay (byte-identity preserved):** materialise the
chain prefix through the same kernels, rebuild the identical `FxHashMap` groups, and
interleave serialisation into the group loop. Byte-identical by construction, but:
(i) the whole prefix is materialised, so "first solution before the join completes"
holds only for the FINAL step — the TTFS win shrinks exactly on the long-chain /
large-fan-out class the epic targets; (ii) it invalidates the block-ramp premise (and
bead `.2.4`'s in-block parallel probe) for bind chains; (iii) it welds the streaming
path to the buffered kernel's *incidental* hash-bucket order — any future improvement
to `bind_join`'s iteration order would break the streamed contract. That trades the
mandate's target metric for an order no spec, no consumer, and no design intent asked
for.

**Adopted — deterministic seed-lex order for ALL admitted join chains (bind and,
later, hash):** the streamed emission order is the **seed-lex order** — blocks in
seed-scan order; within a block, seed rows in order; per row, matches contiguous in
match-scan order; composing through the chain as the lexicographic
(seed row, match₁, match₂, …) order. Properties that make this *stronger* than the
raw "deterministic but may differ" phrasing: it is stable (independent of hash
internals), documentable, cheap (the per-block grouping stays as a scan cache), and it
**preserves seed sortedness** — which makes the §5-ladder step 1 ORDER BY elision
(bead `.2.5`) correct by construction, something the buffered hash-bucket order
destroys. Legal because admitted shapes never include ORDER BY: a SPARQL SELECT
without ORDER BY is a solution **multiset** with no order guarantee, and the emit
sink's only production consumer is `stream_select_json` → HTTP clients bound by that
spec. The lib.rs byte-identity rustdoc was true only while the general path still
evaluated buffered; `.2.3` amends the `query_json_stream_with_budget` /
`…_prepared_…` docs to the two-mode contract below.

**The resulting contract (supersedes the old M1/M2 fork; M1/M2 remain as
*milestones* — M1 = bind chains, M2 = hash-step extension, still gated on B1
evidence):**

- **Byte-identical (structural):** shapes whose emission the stream replicates
  exactly — the single-pattern scan fast path (landed, #1745) and any future
  order-preserving step kind (e.g. merge-join chains). Verification: byte-diff.
- **Relaxed (admitted join chains, bind or hash):** deterministic seed-lex order
  (hash steps: build fixed = rhs, posting-list order), which may differ from the
  buffered path. Verification is per-shape solution equivalence, NOT byte-diff:
  - DISTINCT/REDUCED shapes → solution-**set** equality vs buffered + head equality +
    valid JSON (dedup makes order the only difference);
  - non-slice, non-distinct → solution-**multiset** equality + head + valid JSON;
  - Slice (LIMIT/OFFSET) → the streamed solutions are a **sub-multiset of the
    UN-sliced buffered result with the buffered sliced count** — under a different
    legal order, LIMIT legitimately selects different rows, so bag-equality against
    the sliced buffered output would be a false invariant;
  - determinism itself → the streamed path run twice is byte-identical to itself.
- **Fallback (everything else):** the buffered path unchanged — trivially
  byte-identical; the harness asserts the path taken via testing stats rather than
  assuming it.

Budget-trip note: a row/byte/deadline trip mid-stream truncates a *different prefix*
on the two paths (seed-lex vs hash order). Both surface as a truncated chunked body /
`Err`, never a clean short 200 (§8.4) — a truncated response is an error surface, not
an answer, so prefix divergence there is contract-consistent.

**ORDER BY degradation ladder (v1.5, §10 B5):**

1. `ORDER BY` key already satisfied by the seed-scan order (`sorted_by` propagates
   through the chain — which §5.1's seed-lex emission preserves) → sort elided, plan
   admitted, streams fully; contract = ORDER-key-correct output (§8.2), not
   byte-identity to the buffered sort.
2. `ORDER BY` + small `LIMIT` → #1741's top-k: still blocking, but the buffer is k
   rows and the output tiny; no streaming needed (TTFS ≈ total, both small).
3. Full `ORDER BY` (no index order, no top-k) → **honest floor**: sort is a true
   pipeline breaker; the plan takes the buffered path, whose serialise phase already
   streams (#1745). TTFS ≈ evaluation time, stated as such — no design can beat it
   without changing the query's semantics.

GROUP BY/aggregates: outputs are typically small (bounded by group count); they keep
the buffered path and are out of scope.

---

## 6. Operator classification (the reference table)

| operator | class | streaming treatment |
|---|---|---|
| BGP triple scan (seed) | streaming | block driver, adaptive ramp |
| bind_join (INL probe) | streaming | per-block probe, seed-lex order (§5.1; M1) |
| hash_join | streaming-after-build | rhs built once, block probes, seed-lex × posting order (M2 only) |
| merge_join | not admitted v1 | fallback (buffered) |
| FILTER (scalar, compiled) | streaming | per-block, `CompiledExpr` |
| Project | streaming | static head, early head emission |
| DISTINCT / REDUCED (hash) | streaming **with state** | persistent first-seen set, immediate emission — *corrected class, §3.1* |
| LIMIT / OFFSET | streaming | early-exit driver stop |
| UNION (of streamable branches) | streaming | branch-sequential drivers |
| ORDER BY (index-order match) | streaming (elided) | §5 ladder step 1 |
| ORDER BY (top-k) | blocking, bounded | #1741, small output |
| ORDER BY (full) | **pipeline breaker** | buffered + streamed serialise (floor) |
| GROUP BY / aggregates | pipeline breaker | buffered (small outputs) |
| BIND / OPTIONAL / MINUS / paths / VALUES / subqueries | not admitted v1 | fallback; candidates for later extension |

---

## 7. Latency target and measurement discipline

- **Metric:** TTFS (request → first `bindings` byte) alongside the existing TTFB, via
  `scripts/bench-adapters/http_sparql_adapter.py` on the canonical same-box panel.
- **Goal (to be measured, not claimed):** on q04, first solution after roughly
  build-side + first-block work — the design intent is to land in qlever's
  first-byte class (tens of ms) and materially below sparq's current 372 ms, while
  **totals stay within noise** of the current fastest-of-field 428 ms. Whether the
  oxigraph class (single-digit ms) is reachable depends on q04's measured build-side
  cost — B1 answers that before any implementation bead runs.
- **Hard constraint (the mandate):** the canonical re-gather must show no regression
  on full-request latency anywhere in the panel. A TTFS win bought with a totals loss
  is a fail, per the performance-dominance mandate's honest-comparison rule.

---

## 8. Invariants (what every child bead preserves)

1. **Exact-or-relaxed-or-fallback (§5.1, amended by `sq-7d3dj.34.2.7`):** shapes
   whose emission is structurally order-preserving (single-pattern fast path; future
   merge-join chains) stay byte-identical to the buffered path; any shape not
   provably meeting its contract is *not admitted* rather than emitted differently.
2. **Deterministic seed-lex order + per-shape solution equivalence (admitted join
   chains, bind AND hash):** emission is the documented seed-lex order — deterministic
   (double-run byte-identical) but permitted to differ from the buffered path; the
   solutions satisfy the §5.1 per-shape equivalence vs buffered (set equality under
   DISTINCT/REDUCED, multiset equality otherwise, sub-multiset + count under Slice),
   with head equality and valid JSON. Never admitted for ORDER BY plans (elided-sort
   admission in `.2.5` must be **ORDER-key-correct**, which seed-lex order provides
   when the seed order satisfies the key).
3. **Fail-closed classification:** unknown/instrumented/recorder-armed shapes take the
   buffered path unchanged. The streaming path can only *narrow* — never alter — the
   semantics of what it admits.
4. **Budget & truncation semantics inherited unchanged:** per-block `budget::check`;
   post-first-byte trips surface as a truncated chunked body exactly as #1745
   documents (never a clean short 200 — Wave D §6's answer-safety invariant).
5. **Feature discipline:** always-compiled fast path plus a `stream_pipeline_testing`
   runtime toggle (the `sip_testing` / `distinct_pushdown_testing` precedent) so the
   differential harness can force both paths; no new cargo feature, no new
   dependency, sparq-core untouched.

---

## 9. Non-goals

CSV/TSV streaming (the T16 chunk seam is separate and already landed); columnar
execution (gated in `sq-pntvh`); HTTP/2/3; SaGe-style pagination/continuations;
OPTIONAL/MINUS/paths streaming (future extension once v1 evidence exists); planner
changes to *prefer* streamable plans (measure first); the server transport itself
(#1745 + open beads sq-7d3dj.26/.28 own it).

---

## 10. Decomposition (child beads)

Engine beads are dependency-serialised (same crate — the conflict-partition rule);
only B1 ∥ B4 and B5 ∥ B6 run in parallel, and no two parallel beads share a file.

| id | bead | crate / files | tier | invariant | acceptance test |
|---|---|---|---|---|---|
| sq-7d3dj.34.2.1 | q04 plan-shape + phase profile post-#1745: EXPLAIN the canonical q04 plan (bind_join vs hash_join steps), time eval vs serialise, pin M1-vs-M2 scope | bench (analysis only; findings as bead comments) | sonnet | honest scope call — M2 ships only on evidence | bead comment with plan shape + phase timings + explicit M1/M2 verdict |
| sq-7d3dj.34.2.2 | differential equivalence harness: byte-diff `eval_select_json_emit` concat vs `eval_select_json` over a corpus incl. multi-var DISTINCT-join, UNION, LIMIT, filters; non-vacuity mutation check. **Delivered — but its 4 000-row corpus fits one default block, so it is vacuous at the block boundary; the multi-block + §5.1-contract extension moved into `.2.3`'s acceptance.** | `crates/sparq-engine/tests/stream_pipeline_equivalence.rs` (new) | sonnet | harness is non-vacuous (a perturbed byte goes red) | `cargo test -p sparq-engine --test stream_pipeline_equivalence` |
| sq-7d3dj.34.2.3 | streaming pipeline v1 (M1, **seed-lex order contract §5.1**): classifier + block driver (per-block scan-cache probe, seed-order emission) + filter/project/first-seen-DISTINCT/slice + early head + budget/recorder fallbacks + `stream_pipeline_testing` toggle **with ramp-constant override** + rustdoc two-mode contract on the stream APIs + the §5.1 harness suites | `crates/sparq-engine/src/stream_pipeline.rs` (new) + one hook in `exec.rs` + `mod`/rustdoc in `lib.rs` + harness extension | opus | §8.1/§8.2 as amended (seed-lex determinism + per-shape solution equivalence); §8.3 fail-closed | harness green **including**: a testing-ramp (first = 8) fixture where a join value's seed rows straddle a block boundary (stats assert ≥ 2 blocks + streamed path taken), one default-ramp fixture with seed > 4 096 rows, a streamed double-run byte-diff (determinism), per-shape set/multiset/sub-multiset suites, a perturbed-**solution** mutation test going red, and a first-emit-before-eval-complete probe (emit observed while processed seed rows < total) |
| sq-7d3dj.34.2.4 | in-block parallel probe + adaptive ramp: order-preserving `par_iter` within blocks, worker budget snapshots, ramp constants. **Premise unchanged by `sq-7d3dj.34.2.7`** — the block ramp survives; the order preserved is §5.1 seed-lex | `crates/sparq-engine/src/stream_pipeline.rs` (serialised after .3) | sonnet | §8.2 seed-lex order retained; steady-state totals ≈ buffered (bench-verified, no number hard-coded) | harness green (incl. the multi-block suites) + criterion bench added and running (`cargo bench -p sparq-engine --bench stream_pipeline` compiles/runs) |
| sq-7d3dj.34.2.5 | classifier extensions: ORDER BY index-order elision (§5 ladder 1, **ORDER-key-correct** — byte-identity to the buffered sort is NOT the contract) + M2 hash-step admission IF the .1 verdict requires it (else explicitly skipped), under the same §5.1 relaxed contract | `crates/sparq-engine/src/stream_pipeline.rs` + harness (serialised after .4) | opus | §8.2 (uniform for bind + hash); elided sorts satisfy the ORDER BY key | harness ORDER BY (key-correctness) + per-shape equivalence suites green |
| sq-7d3dj.34.2.6 | canonical D9c re-gather: EC2 same-box panel with TTFS instrumentation added to the adapter; dashboard ingest; honest verdict row | `scripts/bench-adapters/http_sparql_adapter.py`, `bench/canonical-competitor-results/`, `bench/dashboard/` | sonnet | honest verdict — TTFS AND totals reported, regression = fail | new canonical envelope committed + `scripts/bench/ingest-canonical-competitors.mjs` clean |

Ordering edges (wired in bd): .1 → .3, .2 → .3, .3 → .4, .4 → .5, .4 → .6. Estate
reconciliation: sq-7d3dj.29 closed as superseded by this record (done at decomposition
time); on merge of #1745 close sq-7d3dj.24/.25 and re-verify-then-close .27.

Decision bead `sq-7d3dj.34.2.7` (raised from `.2.3` implementation, blocking):
adjudicated 2026-07-12 per §5.1 — driver keeps block-slicing, bind chains move to the
deterministic seed-lex contract, byte-identity retained only where structural;
`.2.3`/`.2.4`/`.2.5` re-specced as above; closed with this amendment.
