<!-- [FABLE-5] Design record (design-for-review, epic sq-pntvh). Architect-tier synthesis:
     grounded in the prose digest of the merged Phases 1-3 and in the prior record
     research/vector-at-a-time-m4.md (whose file:line evidence this record inherits rather
     than re-verifying — the architect stage deliberately stays in prose). NO benchmark
     numbers appear (the work box is non-canonical; every runtime-perf payoff is
     EC2-measurement-gated). No ZK/MPC soundness claim is made beyond preserving the
     existing zk-decline invariant. DESIGN ONLY — no implementation lands with this doc. -->

# Vector-at-a-time (M4) completion: refined specs for the open phases

**Status:** design record, 2026-07-02. Epic **sq-pntvh**. Companion to
[`vector-at-a-time-m4.md`](vector-at-a-time-m4.md) (the operator-model record) — that record
chose *what* to build and in what order; this one pins the **byte-identity argument, the
eligibility gate, and the non-vacuous acceptance test** for each still-open phase, and takes
a position on the two genuinely subtle questions (the general-column FILTER, and evaluator
wiring). It is the build contract for the implementing fleet.

---

## 0. State + bead-id reconciliation

**Merged:** Phase 1 (differential byte-identity harness + native criterion bench), Phase 2
(`decode_numeric_column`: id column → contiguous `Vec<f64>`, NaN sentinel for non-numeric,
inline-integer gather-free fast path), Phase 3 (columnar residual-FILTER seam: transpose →
decode-once → branchless compare → order-preserving materialise; eligibility deliberately
narrowed to a single sargable numeric residual `?var OP constant` over an **all-inline-integer**
column; **fully disabled whenever the zk proof-trace is armed** so the scalar path records the
complete FILTER obligation set).

**Not yet true end-to-end:** the evaluator still never *constructs* a chunk on a real query —
the Phase-3 seam and the Phase-2 decode are building blocks the row evaluator does not call.
Wiring is the pivotal remaining step.

**Bead-id drift (important for dispatch).** The orchestration digest labels "wire the kernels
into the evaluator" as `sq-pntvh.7`; in the tracker that work is split across **sq-pntvh.5**
(the `columnar_eligible` dispatcher + morsel constant + columnar PROJECT) and **sq-pntvh.6**
(the morsel pull-pipeline), while tracked **sq-pntvh.7** is the *skip-aware merge-JOIN*. This
record specs the **wiring** under §4 (recommended landing: a re-scoped sq-pntvh.5; see the
re-scope note there) and gives the merge-JOIN a short scoping note under §5 — including an
output-order byte-identity trap that materially constrains it.

**Cross-record consistency.** Everything here preserves the prior record's settled decisions:
`Bindings` stays the operator-boundary type; columnar is a spliced accelerator between two
`Bindings` boundaries; row is the fallback for everything declined; `vectorized` stays
non-default (and never on for wasm) until an EC2-measured win exists.

---

## 1. Cross-cutting invariants (owned by ONE dispatcher, inherited by every seam)

All four phases share five invariants. To keep them from being re-implemented (and drifting)
per seam, they live in a single `columnar_eligible` dispatcher (§4) evaluated **once per
operator invocation**, and in one small stats hook. Every later phase plugs into these; no
phase re-decides them.

**I1 — Decline is total and silent-safe.** Any condition the columnar path cannot *prove*
byte-identical falls through to the unchanged scalar code for the whole operator invocation
(or, in the hybrid FILTER, for the specific delegated rows). Declining is always correct;
only entering columnar carries proof obligations.

**I2 — zk-decline is dispatcher-owned.** The check "proof-trace armed ⇒ ineligible" moves
into the dispatcher so *every* current and future seam (filter, aggregate, project, join)
inherits it by construction rather than by per-seam discipline. The Phase-3 rule is unchanged
in effect; this centralises it. Rationale: the scalar path is what records the complete
obligation set; no columnar seam may ever run while a trace is armed.

**I3 — Budget parity.** The row loops debit a cooperative budget per row. A columnar seam
must truncate at the *same row index* a capped query would truncate at under the scalar path.
Rule: if the scalar debit schedule is uniform-per-row (implementer verifies), the seam
computes `k = min(batch_len, budget_remaining)` at each morsel boundary, processes exactly the
first `k` rows columnar, debits `k`, and signals exhaustion identically. If the debit schedule
turns out non-uniform, the fallback rule is the zk pattern: **budget armed ⇒ decline** —
zero-risk, revisit later. Either way a budget-capped differential test is mandatory (§4 T4).

**I4 — Decode parity.** Columnar decode must be the *same value mapping* the scalar
comparison uses (`graph.numeric_value`, per the prior record) — every byte-identity argument
below has "columnar and scalar see the same f64 for the same id" as its premise. Derived /
local-vocab ids and `NO_ID` (unbound) are *not resolvable* by that mapping and must decode to
the NaN sentinel (never a spurious value).

**I5 — Probe observability (non-vacuity infrastructure).** A `vectorized`-gated stats hook —
relaxed atomic counters `{chunks_built, rows_columnar, rows_delegated, declines_by_reason}` —
compiled out entirely when the feature is OFF, documented unstable/test-facing. Every
acceptance test below asserts on these counters; without them "byte-identical" tests are
vacuously green when the seam silently declines. This is the single most important piece of
test infrastructure in the completion plan.

---

## 2. sq-pntvh.4 — columnar reducer aggregates (SUM / COUNT / AVG / MIN / MAX)

### Recommended approach

Vectorise **only the per-group fold**, never the grouping. The existing `build_groups`
(first-seen group order) runs unchanged; for each group whose member-row count passes the
size threshold and whose aggregated column passes the gate, the fold over member rows is
replaced by a columnar reducer; all other groups (and all other aggregate kinds) take the
existing scalar fold. Group table, group order, and result assembly are untouched — so
byte-identity reduces to "per-group reduced *term* identical".

**The gate criterion is reassociation safety, and it dictates everything.** A SIMD reduction
reorders and re-parenthesises the fold. Floating-point addition is not associative, so an f64
SUM/AVG fold reordered by SIMD lanes produces different rounding than the scalar sequential
fold — a byte difference. Exact integer arithmetic IS associative and commutative, so
reordering is harmless. Hence the v1 gate:

- **Eligible:** non-DISTINCT `SUM / COUNT / AVG / MIN / MAX` over an **all-inline-integer**
  column (same gate Phase 3 proved out), whole-dataset or GROUP BY via the per-group rule
  above.
- **Accumulator:** exact `i128`. Overflow is *structurally impossible* — inline-integer
  magnitude is bounded by the id encoding (well under 2^63) and row count is bounded far
  below 2^63, so |sum| < 2^63 · 2^63 = 2^126 < i128::MAX. The implementer must state this
  bound as a code comment with the actual inline-range constant, and keep a
  `debug_assert!`-checked add. No runtime decline path for overflow is needed.
- **MIN/MAX are tie-free under this gate**: an inline-integer id encodes its value, so equal
  value ⇒ equal id ⇒ *the same term*. The "two distinct terms compare equal as f64 — which
  term does MIN return?" ambiguity (which would otherwise need the scalar's exact-recheck /
  first-seen tie behaviour) cannot arise. This — not decode difficulty — is why the gate
  stays all-inline for v1. Bonus: because `numeric_value(inline id) = (id − INLINE_BASE)` is
  monotone in the id, MIN/MAX may reduce **directly over the id column, gather-free** and
  return the term of the winning id.
- **Final term construction is delegated to scalar helpers.** The reducer produces the exact
  integer (and count); the *term* for SUM (datatype + lexical form, including any promotion
  rule) and the AVG division (SPARQL AVG is a decimal division with engine-specific
  serialization) are produced by calling the same helper functions the scalar aggregate path
  calls, once per group. The columnar code never formats a term.
- **Declined (stay scalar):** any DISTINCT aggregate, GROUP_CONCAT/SAMPLE/custom, non-inline
  columns (doubles/decimals/derived ids — *deliberately*, per reassociation safety, even
  though Phase 2 can decode them), high-cardinality GROUP BY below the per-group threshold,
  zk-armed (I2), budget rule (I3).

One mandated cross-check before coding: the implementer must confirm what the scalar
aggregate's accumulator actually is on all-inline-integer input. If the scalar path itself
accumulates in f64 (possible), then byte-identity requires matching *its* result — and since
an exact i128 sum of integers within the f64-exact range equals a sequential f64 sum only
while partial sums stay ≤ 2^53, the gate must additionally bound `Σ|v|` ≤ 2^53 (cheap: bound
`max|v| · rows`), or the scalar fold must first be fixed to exact integer arithmetic (a
separate, scalar-side bead — do not smuggle it into this one). The differential test T1
below is designed to catch this either way.

### Byte-identity argument

Grouping, group order, and output assembly are the unchanged scalar code. Per group: exact
integer arithmetic is order-insensitive, so the SIMD fold equals the scalar fold's value;
MIN/MAX ties are impossible under the inline gate so the returned term is forced; the final
term bytes come from the identical scalar helper. Every input outside that proof is declined
to the scalar fold (I1).

### Acceptance tests (non-vacuous)

- **T1 (exactness, mutation-proof):** kernel-level reducer test over an adversarial integer
  set whose *sequential f64* fold differs from the exact fold (partial sums crossing 2^53 —
  synthesise at kernel level; the kernel takes a column, so this needs no giant dataset).
  Assert the exact result. This test goes red if anyone later swaps the accumulator to f64.
- **T2 (end-to-end, probe-armed):** differential-harness queries — whole-dataset
  SUM/COUNT/AVG/MIN/MAX and a low-cardinality GROUP BY over an all-inline column — assert
  byte-identical SPARQL-JSON **and** `chunks_built ≥ 1` (I5). Include a GROUP BY whose
  first-seen group order differs from sorted order, to pin order preservation.
- **T3 (decline completeness):** DISTINCT aggregate, GROUP_CONCAT, a non-inline (decimal)
  column, and a zk-armed run each assert byte-identity **and** `chunks_built == 0`.
- **T4 (tie-freeness witness):** MIN/MAX over a column with the same value occurring many
  times — byte-identical, and (documentation-level) the inline-id argument recorded next to
  the kernel.

**Files (disjointness):** new `chunk/reduce.rs` (or `reduce.rs` sibling) + the
`group_aggregate` call site + one new test file. Does not touch the FILTER kernel (§3 works
in parallel without conflict).

---

## 3. sq-y5ew5 — general-column FILTER: the hybrid tri-mask (POSITION)

### The question

Broadening beyond all-inline-integer means facing the two scalar subtleties: (a) the f64-tie
**exact-lexical recheck** (two values equal as f64 are re-compared by exact/lexical value),
and (b) **derived/computed ids** resolved via a local vocabulary, not the global dictionary.
The bead as filed says "replicate both inside the columnar path". **Recommendation: do not
replicate. Build the hybrid.** The replication approach is strictly dominated: it forks two
exact-correctness-critical code paths (lexical comparison, local-vocab resolution) into a
second implementation that must track the scalar one forever, for negligible extra SIMD
coverage — ties and derived ids are precisely the rare lanes in the workloads this feature
targets.

### The hybrid design (tri-mask + scalar delegation)

One columnar pass over the decoded column classifies every lane into exactly one of:

1. **Confident** — decode produced a finite f64 `v` and `v ≠ c` (the constant's f64):
   decided branchlessly by the f64 comparison, no recheck.
2. **Tie** — decode finite and `v == c` bit-comparison-equal (also catches +0.0/−0.0):
   **delegated**.
3. **Unknown** — decode returned the NaN sentinel (non-numeric term, derived/local-vocab id,
   `NO_ID` unbound, or a genuine NaN-valued double, all of which conflate safely here):
   **delegated**.

Delegated lanes are evaluated by calling **the existing scalar row predicate on the original
`Bindings` rows** (Seam A has them — no row reconstruction). The final selection vector is
the ascending merge of confident-passes and delegated-passes; one order-preserving
`apply_selection` materialises survivors. The eligibility gate keeps Phase 3's *shape*
(single sargable `?var OP constant` residual; constant must decode to a finite f64 — decline
a NaN constant) but drops the all-inline-**column** requirement; the inline gather-free
kernel remains the fast path when the column *is* all-inline.

### Why the tie∪unknown set is the COMPLETE disagreement set (the load-bearing lemma)

Rounding an exact numeric value to f64 (round-to-nearest — any monotone rounding) is
**monotone**: `x ≤ y ⇒ f64(x) ≤ f64(y)`. Contrapositives, for a lane value `x` and constant
`c`: `f64(x) < f64(c) ⇒ x < c`, and `f64(x) > f64(c) ⇒ x > c`; likewise `x = c ⇒ f64(x) =
f64(c)`, so `f64(x) ≠ f64(c) ⇒ x ≠ c`. Therefore for **all six operators** the f64 verdict on
a non-tie lane agrees with the exact verdict; the only lanes where f64 and exact comparison
can disagree are exactly the `f64(x) == f64(c)` lanes — which the tri-mask computes
branchlessly and delegates. Nothing correctness-relevant is ever decided in the columnar code
on a lane where the scalar path could have behaved differently.

### Byte-identity argument (by construction, not by trust)

Premise: decode parity (I4). Case analysis per lane: (confident) the scalar path's own
comparison on that row sees the same two f64 values and — being a non-tie — takes no recheck,
so its verdict is the same f64 comparison the kernel computed; (tie / unknown) the hybrid's
verdict *is* the scalar predicate's verdict, because it calls it. Order: both index lists are
ascending; their merge preserves original row order; `apply_selection` is order-preserving.
The columnar path contains **zero** new lexical-comparison or vocab-resolution logic, so
there is nothing new to get wrong. zk/budget: I2/I3 unchanged (delegation calls the scalar
*predicate*, which records no obligations when no trace is armed; when a trace is armed the
whole seam never runs).

One escape hatch, for honesty: if the scalar predicate turns out not to be callable per-row
in isolation (entangled with loop state), the fallback is lane-set decline — rows in the
delegated set are processed by the *existing row loop* over just those indices. Same
argument, marginally more plumbing.

### Adaptive guard (optional, v2)

A mostly-non-numeric column pays decode + near-total delegation — correct but a net loss. An
optional refinement: if the delegated fraction of the *first* morsel exceeds a constant
(e.g. one half), decline the remaining morsels of this operator invocation to the row path.
Correctness is unaffected either way (I1); ship v1 without it if time-boxed.

### Acceptance tests (non-vacuous)

- **T1 (mixed-column differential, both-populations probe):** a column mixing inline ints,
  large non-inline integers, decimals (e.g. `0.1`-like non-representables), strings, unbound
  (`NO_ID` via OPTIONAL), and a BIND-derived value; assert byte-identity **and** probe
  counters show `rows_columnar > 0` **and** `rows_delegated > 0` — proving both halves of the
  hybrid executed.
- **T2 (tie-exactness witness):** a value/constant pair distinct exactly but equal as f64
  (e.g. `v = 9007199254740992`, `c = 9007199254740993`, `FILTER(?x < c)`: exact says *keep*,
  a naive pure-f64 path says *drop*). Assert the row survives and bytes match scalar. This
  test goes red if anyone later removes the delegation.
- **T3 (operator sweep):** all six operators × boundary constants over the T1 column, run
  through the Phase-1 differential harness (extend its corpus).
- **T4 (invariants):** zk-armed ⇒ `chunks_built == 0` + identical bytes; NaN-constant and
  non-sargable shapes decline.

**Files:** the select kernel + decode tri-state in `chunk.rs` (or a sibling), one dispatcher
gate-line, own test file. Parallel-safe with §2 (different files; disjoint `exec.rs` regions).

---

## 4. Wiring the seams into the evaluator (digest "sq-pntvh.7"; land as re-scoped sq-pntvh.5)

### Recommended approach

**Seam-A retrofit wiring, dispatcher-first — not the morsel pipeline.** The evaluator is
fully row-materialising: `apply_filter` and `group_aggregate` each receive a complete
`Bindings`. So "constructing a chunk" is a transpose of an already-materialised buffer — no
streaming, no buffering, no interaction with LIMIT-style early termination (those concerns
belong to the Phase-6 pull-pipeline, deliberately out of scope here). The wiring bead
delivers:

1. **`columnar_eligible` dispatcher** (one module): the decline hierarchy, checked in order —
   `cfg(feature = "vectorized")` (OFF ⇒ the call sites compile away entirely) → zk-trace
   armed (I2) → operator shape has a kernel (Phase-3/§3 FILTER shape; §2 aggregate shape) →
   `rows.len() ≥ VEC_MIN_BATCH` → column-dtype precheck. Plus the two constants
   (`VEC_MIN_BATCH`, suggest 256; `VEC_MORSEL`, suggest 2048 per the prior record's DuckDB
   note) in one place — values are placeholders to be EC2-tuned, and no perf claim attaches
   to them.
2. **`apply_filter` call site:** on eligible, extract the *one* tested column (O(rows), not a
   full-width transpose), run the Phase-3 kernel morsel-by-morsel (`VEC_MORSEL` rows per
   decode buffer, reused), apply I3 budget prefixing at each morsel boundary, materialise
   survivors once, order-preserving. Ineligible ⇒ fall into the existing row loop verbatim.
3. **`group_aggregate` call site** (lands with §2, but the dispatcher hook is cut here).
4. **The I5 stats hook.**

**Which shapes actually benefit (the honest eligibility rationale):** large materialised
intermediates with a numeric residual FILTER or a reducer aggregate — i.e. post-join
`Bindings` with row count well above `VEC_MIN_BATCH` and a sargable residual that could NOT
be pushed into the scan. Everything already fast stays untouched by construction: pushed-down
sargable filters never reach `apply_filter` (the scan handles them term-free, per the prior
record); the browser's `single_pattern_scan_json` fast path bypasses `Bindings` entirely and
is upstream of both seams; tiny intermediates fail `VEC_MIN_BATCH`. Where the transpose
overhead would be a net loss — small batches, non-sargable shapes, string filters — the
dispatcher never engages.

**Why the wiring is perf-neutral for the common non-vectorizable query:** the eligibility
decision costs one expression-shape match + one integer compare, **once per operator
invocation** (never per row), only in feature-ON builds; the ineligible branch falls through
to the *same* row loop that exists today. Feature-OFF: the dispatcher and call sites are
`cfg`-gated out — the scalar code path is token-identical to today's, which is exactly what
the Phase-8 gate (§6) turns into a checked property. **Recommendation: decide eligibility at
operator entry (dynamic, per invocation) rather than at plan time** — the row count isn't
known at plan time, the check is O(1)-amortised, and it avoids threading plan-level state.

**Re-scope note for the tracker:** move "columnar PROJECT + selection-vector threading" OUT
of sq-pntvh.5 and into sq-pntvh.6 — without the pull-pipeline there is no filter→project seam
for a selection vector to thread across, so PROJECT-as-column-moves has no payoff standing
alone. sq-pntvh.5 then = dispatcher + constants + `apply_filter` wiring + stats hook, i.e.
exactly this section. sq-pntvh.6 (pipeline) stays a later, separate design pass gated on
Seam A having an EC2-measured win (prior record §5.4).

### Byte-identity argument

The seam is spliced between two `Bindings` boundaries inside one operator; the kernel it
calls is the already-proven Phase-3 (then §3) kernel; order is preserved end-to-end
(transpose and `apply_selection` are order-preserving; morsel boundaries partition the row
range in order); budget truncation matches by I3; zk never engages by I2; everything else
declines to the verbatim row loop (I1). The wiring itself adds no new value-level logic — its
entire proof obligation is "same rows in, same rows out, same order, same side effects".

### Acceptance tests (non-vacuous)

- **T1 (end-to-end, the epic's exit criterion):** a real query through the *public* query
  API, feature ON — join → residual numeric FILTER over an inline column, row count above
  threshold — byte-identical to feature-OFF **and** `chunks_built ≥ 1`. This is the first
  test in the epic where the evaluator itself constructs a chunk; it retires the
  "unreferenced building blocks" finding.
- **T2 (untouched-path probes):** a regex FILTER, a below-threshold result, and a
  pushed-down-sargable-only query each assert `chunks_built == 0` + byte-identity.
- **T3 (zk composition):** with `zk` + `vectorized` both compiled and a trace armed, assert
  `chunks_built == 0` **and** the recorded obligation set is byte-identical to the zk-only
  build's on the same query.
- **T4 (budget parity):** an artificially tiny budget over an eligible query — identical
  truncation/error behaviour, feature ON vs OFF (pins I3's `k = min(...)` prefix rule, or
  documents the decline-when-armed fallback).
- **T5 (both feature states):** the standard both-feature-states CI legs stay green,
  proving the call sites compile in both worlds.

---

## 5. Tracked sq-pntvh.7 (skip-aware merge-JOIN) — scoping note, not a spec

Blocked on the Phase-6 pipeline; do not dispatch yet. One trap to record NOW because it
shapes Phase 6/7 design: **join output order is part of the byte-identity contract.** The
harness asserts byte-identical SPARQL-JSON, which encodes row order; a merge-join naturally
emits sort order, whereas the current row join emits whatever order the existing algorithm
produces. Unless the merge-join provably reproduces the row path's output order (plausible
only where the row path already emits index-sorted order), the choice is: (a) restrict
merge-join eligibility to provably-order-identical cases, or (b) knowingly relax the M4
contract to multiset-equality + canonical-sort *for the join phase only* — a policy change
requiring a maintainer decision and a harness mode, not something an implementer may decide
silently. Recommendation: (a) for as long as it has non-trivial coverage; treat (b) as a
maintainer question raised when Phase 7 is scoped. Split Probe/Build/Skip into sub-beads at
that point (prior record §7.4).

---

## 6. sq-pntvh.8 — the feature-OFF CI proof

### Recommended approach

Three checked legs plus tripwires, added as a lane feeding the existing required-gate
aggregator (no new required context; the aggregator stays the single gate):

1. **Feature-resolution guard.** Assert `vectorized` is absent from the *resolved* feature
   set of the default workspace build (via `cargo metadata`/unit-graph, not source grep).
   This is the leg that catches the real-world failure mode: some crate or dev-dependency
   edge silently enabling the feature workspace-wide (the known feature-unification trap),
   after which "OFF" builds aren't OFF anywhere. Also assert the wasm build profile's feature
   list never contains `vectorized` (the prior record's §3.1 wasm-stays-OFF rule, made a
   check).
2. **Deterministic-artifact exact-equality.** Build feature-OFF; assert `wasm_bundle_bytes`
   and the native deterministic byte metric (bytes/triple) are **exactly equal** to the
   pinned `bench/perf-baseline.json` floors — a zero-delta assertion, stricter than the
   existing percentage ratchet, so *any* accidental default-build impact from M4 work trips
   it. A PR that legitimately moves these metrics must re-pin the baseline explicitly
   in-diff, which is precisely the reviewable event we want.
3. **cfg-audit.** A script asserting every `vectorized` module registration and evaluator
   call site sits under `#[cfg(feature = "vectorized")]` (registration-point audit, not a
   blanket grep of kernel internals). Cheap, and catches the "helper function leaked out of
   the gate" drift class. Mind the known feature-gated intra-doc-link trap while writing any
   gated docs.
4. **Tripwires (the gate's own non-vacuity):** unit-test leg 1 against a synthetic metadata
   fixture with `vectorized` enabled — the guard must exit non-zero; perturb a baseline byte
   in a fixture — leg 2's comparator must fail. A gate that cannot fail is not a gate.

**Honest scope statement (must appear in the lane's docs):** this gate proves *compile-time
absence* and *artifact byte-determinism* of the OFF build. It does **not** prove runtime
performance neutrality on any workload and asserts no perf number — runtime payoff and
neutrality measurement remain EC2-gated, separately. That is the strongest claim the
deterministic ratchet infrastructure can honestly support, and it is sufficient: when zero
vectorized code compiles, the OFF binary has nothing to be slower *with*.

### Byte-identity argument

Legs 1+3 establish that no `vectorized` code exists in the OFF compilation; a binary cannot
be behaviourally or byte affected by code that was never compiled into it; leg 2 checks the
conclusion empirically on the two deterministic artifacts every PR.

### Acceptance tests

The tripwires (leg 4) are the acceptance tests, plus one green run of the full lane on
current main. Independence: depends only on merged Phase 1; no ordering constraint against
§§2–4 — **land it first** so it guards the rest of the epic while the seams are being wired.

---

## 7. Dispatch plan (order, parallelism, tiers)

| Order | Work | Bead | Depends on | Files (disjointness) | Tier |
|---|---|---|---|---|---|
| 1 | Feature-OFF CI proof (§6) | sq-pntvh.8 | — (Phase 1 merged) | CI lane + scripts + fixtures | sonnet (sparq-ci-infra; touches the required-gate aggregator) |
| 2 | Dispatcher + `apply_filter` wiring + stats hook (§4) | sq-pntvh.5 re-scoped (digest "sq-pntvh.7") | — | dispatcher module + `apply_filter` site + harness | sonnet (escalated review — engine-correctness surface) |
| 3a | Columnar reducer aggregates (§2) | sq-pntvh.4 | 2 (dispatcher + hook) | new reduce module + `group_aggregate` site | sonnet |
| 3b | Hybrid general-column FILTER (§3) | sq-y5ew5 | 2 | select kernel + decode tri-state | sonnet (escalated review — the tie lemma must survive review) |
| later | Morsel pipeline; then merge-JOIN (§5) | sq-pntvh.6, sq-pntvh.7 | 3a/3b + EC2-measured Seam-A win | — | fresh design pass first |

3a ∥ 3b are file-disjoint by construction (§2/§3 "Files" notes). Every implementer brief
must carry: the relevant § of this record verbatim, the I1–I5 invariants, the
both-feature-states gate, one direct unit test per new public fn (coverage-ratchet floor),
and the no-perf-numbers rule.

## 8. Open questions for the maintainer (non-blocking; proceed-and-document applies)

1. **§2 cross-check outcome:** if the scalar aggregate fold turns out to be f64 on integer
   input, do we (a) bound the gate by Σ|v| ≤ 2^53, or (b) first fix the scalar fold to exact
   integer arithmetic as its own bead? Recommendation: (b) — it improves the engine
   independently — but (a) is the no-scalar-change fallback.
2. **§5 join-order contract:** confirm byte-identity (option a) remains the M4 contract for
   the join phase, or pre-authorise the multiset+canonical-sort harness mode (option b).
3. **Morsel constants:** `VEC_MIN_BATCH`/`VEC_MORSEL` placeholders (256/2048) to be EC2-tuned
   before any default-on discussion — flagging so nobody reads the constants as measured.
