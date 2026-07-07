<!-- [FABLE-5] Design record (decomposition stage, epic sq-pntvh). Third record in the M4
     series, the "fresh design pass" that research/vector-at-a-time-m4-completion-design.md
     §4/§7 explicitly deferred the morsel pipeline and the merge-JOIN to. Grounded by
     re-reading the ACTUAL code (file:line evidence in §0/§1), not the epic's framing.
     NO benchmark numbers appear (the work box is non-canonical; every runtime-perf payoff
     is EC2-measurement-gated). The only ZK-adjacent statement is the existing zk-decline
     rule, which is an obligation-trace-completeness property, NOT a cryptographic
     soundness claim — the v1 ZK verifier's external accredited-cryptographer audit is
     still pending (sq-qhy4). DESIGN ONLY — no implementation lands with this doc. -->

# Vector-at-a-time (M4): wiring roadmap for the remaining phases

**Status:** design record, 2026-07-06. Epic **sq-pntvh**. Predecessors:
[`vector-at-a-time-m4.md`](vector-at-a-time-m4.md) (operator model — *what* to build),
[`vector-at-a-time-m4-completion-design.md`](vector-at-a-time-m4-completion-design.md)
(build contract for Phases 4/5/8 + the hybrid FILTER — *how* each seam proves
byte-identity). This record is the deferred third pass those records called for: it
corrects the epic's now-stale premise, decides the remaining adoption order, designs the
Phase-6 morsel pull-pipeline, pins the join emission contract that `sq-7d3dj.19` is
waiting on, and re-cuts the open beads into independently landable, file-disjoint steps.

---

## 0. Corrected premise: the kernels are NOT unreferenced any more

The epic description ("built chunk.rs kernels are unreferenced in exec.rs") was true on
2026-07-01 and is **stale now**. Verified against `origin/main` (commit `68afbb25`):

- `apply_filter` (`crates/sparq-engine/src/exec.rs:6896`) contains the live,
  `cfg(feature = "vectorized")`-gated **columnar residual-FILTER seam** — Phase 3
  (`columnar_filter`, exec.rs:6933) calls `DataChunk::from_rows` →
  `decode_numeric_column` → `select_decoded` → `apply_selection` (the Phase-2 kernels,
  `crates/sparq-engine/src/chunk.rs`).
- `group_aggregate` (exec.rs:6130) contains the live **columnar reducer seam** — Phase 4
  (`columnar_aggregate`, exec.rs:6991) with the exact-`i128` reducers in
  `crates/sparq-engine/src/reduce.rs` (plus Kani result-equivalence harnesses,
  sq-sqtk2.3).
- The Phase-1 differential byte-identity harness exists
  (`crates/sparq-engine/tests/differentials/vectorized_byte_identity.rs`, plus the older
  multiset harness `vectorized_exec_differential.rs`), and the bench catalog carries the
  kernel micro row `vectorized-eval-micro` (`bench/benchmarks.toml`, gated behind the
  `vectorized` required-feature example `bench_vectorized`).
- The Phase-8 feature-OFF gate is live CI:
  `.github/workflows/vectorized-feature-off.yml` (feature-resolution guard +
  deterministic-artifact zero-delta + cfg-audit, with tripwire self-tests).

**Still true and load-bearing:** the two seams are *retrofit* seams — each transposes an
already-materialised `Bindings` inside one operator. No chunk ever flows *between*
operators, no scan ever produces a chunk, and the shared `columnar_eligible` dispatcher,
the `VEC_MIN_BATCH`/`VEC_MORSEL` constants, the I5 probe counters, and the I3
morsel-boundary budget prefixing from the completion record's §1/§4 are **not in the
code** (verified: zero hits for `columnar_eligible`, `VEC_MIN_BATCH`, `chunks_built` in
`crates/sparq-engine/src/`). Phases 3/4 each landed their own inline eligibility checks
instead. That duplication is exactly the drift the completion record's I1–I5
centralisation was designed to prevent, and closing it is the first remaining step.

## 1. State table (implemented-and-verified vs designed-only vs proposed)

| Piece | Status | Evidence |
|---|---|---|
| Phase 1 harness + criterion micro-bench | implemented + verified | `tests/differentials/vectorized_byte_identity.rs`; `examples/bench_vectorized.rs`; catalog row `vectorized-eval-micro` |
| Phase 2 `decode_numeric_column` + inline fast path | implemented + verified | `chunk.rs:212` |
| Phase 3 columnar FILTER seam (all-inline, zk-decline) | implemented + verified | `exec.rs:6896–6968` + `columnar_filter_seam` tests (exec.rs:12357) |
| Phase 4 columnar reducer aggregates (i128) | implemented + verified | `exec.rs:6157–6165`, `reduce.rs`, `columnar_aggregate_seam` tests (exec.rs:12909) |
| Phase 8 feature-OFF CI proof | implemented + verified | `.github/workflows/vectorized-feature-off.yml` + `scripts/check-vectorized-feature-off.py` |
| Phase 5 dispatcher + probe counters + morsel/budget wiring | **designed-only** (completion record §4) | no `columnar_eligible`/`VEC_MIN_BATCH`/`chunks_built` in tree |
| sq-y5ew5 hybrid tri-mask general-column FILTER | **designed-only** (completion record §3) | gate in `columnar_filter` is still all-inline-integer |
| EC2 Seam-A measurement (the Phase-6 gate) | **missing entirely** — no bead existed before this record | completion record §4 gates the pipeline on "an EC2-measured Seam-A win"; nothing scheduled it |
| Phase 6 morsel pull-pipeline | **proposed** — designed in §4 below | prior records deferred to "a fresh design pass" |
| Phase 7 skip-aware merge-JOIN | **proposed** — scoped in §5 below | blocked on Phase 6 + the emission contract |

## 2. Adoption order (the decomposition decision)

The brief's suggested order was "likely scan → filter → join". The landed reality chose
(correctly) **filter → aggregate first**, because retrofit seams inside single operators
need no streaming model and prove byte-identity locally. **Scan does not go columnar as
a standalone step at all**: a chunk-producing scan feeding today's row operators would be
transposed straight back to rows at the next operator — pure overhead, no coverage. Scan
adoption therefore arrives *with* the pipeline (as `scan-to-chunk` inside Phase 6), and
join comes last (it needs the pipeline plus an emission contract). The remaining order:

1. **Consolidate + observe** (`sq-pntvh.5` re-scoped): the shared `columnar_eligible`
   dispatcher owning I1–I5, the probe counters, the morsel constants, and
   morsel-by-morsel budget prefixing in `apply_filter`. Without the counters, every later
   "byte-identical" test is vacuously green when the seam silently declines — this is
   test infrastructure for everything after it, so it goes first.
2. **Broaden FILTER coverage** (`sq-y5ew5`): the hybrid tri-mask over general decoded
   columns, exactly per completion record §3 (do NOT replicate the lexical
   recheck/local-vocab — delegate tie/unknown lanes to the scalar predicate). Plugs into
   the dispatcher as one gate-line; widens what the measurement in step 3 can see.
3. **Measure on EC2** (new bead `sq-pntvh.9`): the completion record made the pipeline
   *conditional* on a measured Seam-A win but nothing scheduled that measurement. Run the
   catalog micro row + a new end-to-end eligible-shape A/B (feature ON vs OFF) on a
   throwaway EC2 bench instance; record the verdict as a bead comment on the epic
   (no numbers committed to docs — work-box and one-off EC2 numbers are non-canonical).
   The verdict gates step 4: **no pipeline build on an unmeasured hope**.
4. **Morsel pull-pipeline** (`sq-pntvh.6`): scan→filter→project columnar across
   operators for the narrow conjunctive shape, §4 below. This is where PROJECT and
   selection-vector threading live (moved out of .5 per the completion record's re-scope
   — standalone columnar PROJECT has no payoff without a pipeline to thread selections
   across).
5. **Skip-aware merge-JOIN** (`sq-pntvh.7`): inside the pipeline, order-identical
   eligibility only, inheriting the `sq-7d3dj.19` emission contract, §5 below.

## 3. The staged-adoption seam pattern (precedent: the reasoner's DeltaTable seam, #1643)

Every remaining step lands the same way the reasoner adopted the substrate join kernels
(PR #1643, `sq-qonbz.2` — the house zero-overhead seam pattern):

- **Pure-addition, cfg-gated seam**: a targeted `#[cfg(feature = "vectorized")]` branch
  at an existing call site; the scalar path stays verbatim and is the total fallback
  (invariant I1). Feature OFF compiles to today's code, token-identical — and Phase 8's
  CI gate checks that conclusion on the deterministic artifacts every PR.
- **No dyn dispatch on per-row paths**: generic `impl FnMut` plumbing, like
  `DeltaTable::probe_emit` (`scripts/check-no-dyn-dispatch.py` stays clean).
- **Behaviour-neutral proof per seam**: byte-identical output in both feature states,
  asserted by required-feature differential tests PLUS the I5 probe counters proving the
  columnar path actually ran (non-vacuity) or provably declined.
- **Both-feature-state gates**: build/clippy/test/doc green with the feature ON and OFF;
  one direct unit test per new public fn (the coverage-ratchet floor); mind the
  feature-gated intra-doc-link trap (use code spans, not intra-doc links, for
  `vectorized`-gated items referenced from always-compiled docs).

## 4. Phase 6 design: the morsel pull-pipeline (`sq-pntvh.6`)

### Shape and seam

One new module (`crates/sparq-engine/src/pipeline.rs`), one seam call site. The pipeline
targets exactly the **conjunctive single-scan shape** for v1: a single triple-pattern
scan (or the first scan of a BGP before any join) + a dispatcher-eligible sargable
numeric residual FILTER + a projection, optionally terminating in a Phase-4-eligible
partial aggregate. Everything else — joins, OPTIONAL, UNION, property paths, ORDER BY,
DISTINCT — declines to the row evaluator for the whole query (I1). The seam is a single
`try_columnar_pipeline(graph, …) -> Option<Bindings>` attempt, dispatcher-gated, spliced
where the row evaluator currently sequences `scan_to_bindings` → `apply_filter` →
`project_bindings`; on `None` the row code runs verbatim.

### Operator model

- `trait VecOp { fn next(&mut self) -> Option<DataChunk>; }` — pull-based, one
  `VEC_MORSEL`-row chunk at a time, no `Box<dyn>` in the per-chunk loop (enum-dispatch or
  generics, matching the house no-dyn rule).
- **ScanOp** (`scan-to-chunk`): builds chunks *directly* from the store scan in
  permutation order — the first time a chunk is constructed without transposing an
  already-materialised `Bindings`. Column layout mirrors `scan_to_bindings`'s var
  positions; the `sorted_by` bookkeeping is computed identically (the truthful
  `actual_sort` rule at exec.rs:5394 carries over unchanged).
- **FilterOp**: the Phase-3 / y5ew5 kernel per chunk, emitting a **selection vector**
  alongside the chunk instead of materialising survivors.
- **ProjectOp**: column moves only — reorders/drops column vectors under the selection
  vector without re-gathering rows (this is the "columnar PROJECT + selection-vector
  threading" item, landing here per the completion-record re-scope).
- **Boundary**: `apply_selection` + `to_rows` materialises a `Bindings` exactly once at
  the pipeline exit (or feeds `columnar_aggregate` group folds directly when the query
  ends in an eligible aggregate).

### Byte-identity argument

Scan order: ScanOp enumerates the same store scan in the same permutation order as
`scan_to_bindings`, chunk boundaries partition that order contiguously, so concatenated
chunk rows = the row path's rows. Filter/project: the kernels are the already-proven
Phase-3/y5ew5 kernels and order-preserving column moves; a selection vector is ascending
per chunk, so survivor order is preserved within and across chunks. Budget: I3
morsel-boundary prefixing (`k = min(chunk_len, budget_remaining)`), identical truncation
row index — or decline-when-budget-armed if the scalar debit schedule proves
non-uniform, exactly the completion record's fallback rule. zk: I2 dispatcher-owned
decline — no pipeline while a proof trace is armed (obligation-trace completeness;
not a cryptographic claim). LIMIT/early-exit: v1 declines queries with LIMIT below the
materialisation boundary rather than reasoning about early termination (I1 — decline is
always correct).

### Acceptance (mechanical for the verifier)

- End-to-end differential: pipeline-eligible queries through the public query API,
  byte-identical SPARQL-JSON feature-ON vs feature-OFF, **and** probe counters
  (`chunks_built ≥ 2`, proving multi-morsel flow) — extending
  `tests/differentials/vectorized_byte_identity.rs`'s corpus.
- Decline probes: join-containing, LIMIT-carrying, ORDER-BY-carrying, and zk-armed
  queries assert `chunks_built == 0` + byte-identity.
- Budget parity test at an artificially tiny budget (both feature states).
- Both-feature-state CI legs + `vectorized-feature-off` gate green.

## 5. Phase 7 scoping + the emission contract pin (unblocks `sq-7d3dj.19`)

**Join order contract:** the completion record's §5 trap stands — SPARQL-JSON
byte-identity encodes row order, so the merge-JOIN's v1 eligibility is restricted to
**provably order-identical** cases (the prior record's option (a)); relaxing to
multiset+canonical-sort (option (b)) remains a maintainer policy decision, re-raised in
the proceed-and-document issue for this record, NOT something Phase 7's implementer may
decide. Cyclic BGPs stay on WCOJ/LFTJ (`sparq_substrate::join::{Trie, lftj_recurse}`)
untouched.

**The emission contract (pinned here so `sq-7d3dj.19` can proceed independently):** the
hash-join probe rework and the M4 vectorized join share one output discipline —
*emit `(build_idx, probe_row_range)` index-pair runs, materialise once per output
batch/chunk* (reserve exact match count; stitch rows into the reused output buffer at
the batch boundary; never clone a build `Row` per emitted match inside the probe loop).
`sq-7d3dj.19` implements this contract in the scalar `hash_join`/`probe_emit` path first
(feature-independent, byte-identical, its own bead in epic `sq-7d3dj`); Phase 7's morsel
join then *inherits* the emission code instead of duplicating it. Dependency edge added:
`sq-pntvh.7` depends on `sq-7d3dj.19` (both also touch
`crates/sparq-substrate/src/join.rs`, so the edge doubles as conflict sequencing).

## 6. Gates that make downstream verification mechanical

- **Correctness:** the byte-identity differential harness (per-phase corpus extension is
  part of each bead's acceptance test), the I5 probe counters (non-vacuity), the seam
  unit differentials in `exec.rs`, both-feature-state build/test legs.
- **Feature hygiene:** the `vectorized-feature-off` three-leg CI gate (already live)
  keeps OFF builds compile-time absent + artifact-byte-identical; any accidental
  default-build impact from wiring work trips the zero-delta leg.
- **Perf evidence:** catalog row `vectorized-eval-micro` (kernel micro) + the new
  end-to-end eligible-shape row added by `sq-pntvh.9`. Verdicts live in bead comments /
  EC2 console output; **no perf numbers are committed to docs** and no bead's acceptance
  test asserts a number — deterministic gates only (the perf-baseline zero-delta is a
  byte metric, not a timing).
- **Coverage ratchet:** every new public fn in the dispatcher/pipeline modules ships one
  direct unit test (the line-coverage floor bites thin facades reached only indirectly).

## 7. The re-cut bead plan (phased, file-disjoint)

| # | Bead | Crate/surface | Tier | Depends on | File-area (disjointness) |
|---|---|---|---|---|---|
| 1 | `sq-pntvh.5` — `columnar_eligible` dispatcher + I5 probe counters + `VEC_MIN_BATCH`/`VEC_MORSEL` + morsel/budget wiring in `apply_filter` | sparq-engine | sonnet | — (Phase 3/4 merged) | new `src/vec_dispatch.rs` + the two existing seam sites in `exec.rs` + harness extension |
| 2 | `sq-y5ew5` — hybrid tri-mask general-column FILTER (completion record §3 verbatim) | sparq-engine | sonnet | `sq-pntvh.5` | new `src/chunk_select.rs` + one dispatcher gate-line + own test file |
| 3 | `sq-pntvh.9` (new) — EC2 Seam-A measurement + end-to-end catalog row; verdict comment gates the pipeline | bench/ (catalog + adapter script; no engine code) | sonnet | `sq-pntvh.5` | `bench/benchmarks.toml` + `bench/scripts/` only |
| 4 | `sq-pntvh.6` — morsel pull-pipeline (§4): VecOp + scan-to-chunk + PROJECT/selection threading + boundary | sparq-engine | opus | `sq-pntvh.5`, `sq-y5ew5`, `sq-pntvh.9` verdict | new `src/pipeline.rs` + one seam call site in `exec.rs` + harness extension |
| 5 | `sq-pntvh.7` — skip-aware merge-JOIN in the pipeline, order-identical eligibility (§5) | sparq-engine | opus | `sq-pntvh.6`, `sq-7d3dj.19` | `src/pipeline.rs` join op + `sparq-substrate/src/join.rs` merge extension + own test file |

Beads 2 and 3 are file-disjoint and run in parallel (different surfaces: engine vs
bench). All sparq-engine beads are serialised by dependency edges — one engine bead in
flight at a time, zero merge-conflict risk by construction. Every bead's `-d` body
carries `crate`, `tier`, `INVARIANT` (result-set/byte equality vs the row path in both
feature states + no ratchet regression + feature-off-by-default), and a runnable
`ACCEPTANCE` (`cargo test -p sparq-engine --features vectorized …` / the gate lanes), so
verification is mechanical.

## 8. Judgment calls made under proceed-and-document

1. **Re-scoped the existing open beads in place** (`.5`/`.6`/`.7`/`y5ew5` get full
   specs via `bd update`) rather than creating duplicate children — duplicate beads are
   a live collision risk; only the genuinely missing fragment (`sq-pntvh.9`,
   measurement) is newly created.
2. **The Phase-6 EC2 gate is now a hard dependency edge**, not prose: the pipeline bead
   is blocked until the measurement bead's verdict lands. If the verdict is "no
   measurable Seam-A win", the honest outcome is that Phases 6/7 are **deferred**, the
   epic closes on consolidation + coverage + measurement, and the pipeline design above
   stays a designed-only record — building it anyway would be perf theatre.
3. **Join order contract stays option (a)** (order-identical eligibility) until the
   maintainer explicitly authorises the multiset+canonical-sort harness mode.
4. **The emission contract is pinned** (§5) so `sq-7d3dj.19` and Phase 7 cannot drift
   apart; `sq-7d3dj.19` lands first and Phase 7 inherits.
