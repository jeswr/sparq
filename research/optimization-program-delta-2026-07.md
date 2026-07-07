# Optimization program — wave-3 delta audit and dispatch plan (epic sq-7d3dj)

> 🤖 SPARQ agent — Fable-tier decomposition record, 2026-07-06. [FABLE-5]

## 0. Premise correction — this epic is already decomposed; do not re-audit

The dispatch brief asked for "EXACTLY ONE design record: a prioritized audit of the
5 axes". That audit **exists** and stands:

- `research/optimization-audit-2026-07.md` (PR #1355) — the five-axis prioritised
  roadmap, 14 beads, the do-not-touch negative space, and the measurement
  discipline. Nothing there is stale; it is not restated here.
- `research/engine-performance-review.md` (PR #1397) — five further runtime beads
  (sq-7d3dj.17–.21) from the hot-path profiling digests.

Re-auditing the five axes from scratch would duplicate two live records
(non-sycophantic honesty rule: already-done work is not re-decomposed). What the
epic actually needs at 7/22-complete is a **delta pass**: verify the estate against
the plan, correct any disjointness errors, close instrument gaps the landed work
exposed, and compose the next parallel wave under the current measurement
constraints. That is this record's entire scope.

## 1. Estate delta since the prior records (verified against code, not taken on faith)

Landed from the roadmap: `.1` (release-wasm profile; `js/package.json` `build:wasm`
now builds `--profile release-wasm`), `.2` (pre-sized chunk dict/vec), `.3` (overlay
zero-copy fast path; part (b) split to `.16`), `.5` (loopback HTTP throughput
harness at `bench/serve-throughput/`), `.10` (bounded parallel SELECT-JSON), `.17`
(radix permutation sort; lazy-perm follow-up split to `sq-dzfzq`), `.18`
(prefix-memoized IRI validation).

Adjacent context that changes wave composition:

- The **engine facade split** (sq-6vshe.4) is landing seams around
  `crates/sparq-engine/src/exec.rs` (`sparq-engine-serialize` #1542,
  `sparq-engine-service` #1563). Engine-exec beads must go one-per-wave and
  rebase-check against the seam PRs.
- **PR #1600** (site lane) added `site/scripts/check-bundle.mjs` — a deterministic
  wasm-free first-load guard for `/` and `/capabilities`. The wasm-bundle axis now
  has two instruments: the raw `wasm_bundle_bytes` ratchet
  (`scripts/ci-bench.sh` → `scripts/perf-gate.py`) for the engine artifact, and the
  site bundle guard for route payloads. Neither covers the *sibling* bundles
  (`build:reason-wasm` / `build:rsp-wasm` / `build:text-wasm` still build plain
  `--release`) — that is exactly bead `.15`, unchanged.
- No open PR currently touches the hot-path files below (verified against the open
  PR list on 2026-07-06), so wave composition is limited by bead-vs-bead
  disjointness only.

## 2. Per-axis status (instrument · open coverage · genuinely-new gap)

| Axis | Canonical instrument today | Open beads covering it | Genuinely-new gap found |
|---|---|---|---|
| Query perf | deterministic ratchets are byte-only; wall-clock = `op_*`/SP2B trend series on the EC2 rig (`bench/benchmarks.toml`, `sparq-cli bench`) | .4, .7, .11, .19, .20 (+ M4 sq-pntvh, DPccp sq-iywur — owned elsewhere) | none — held on the EC2-quota constraint (§4) |
| Memory | `store_bytes_per_triple[_small]`, `dict_bytes_per_term` ratchets; `Graph::heap_bytes` accessor; **no allocation-count instrument anywhere** (verified: no counting `GlobalAlloc`/dhat/stats_alloc in `bench/` or any bench harness) | .8, .16 (+ .6 alloc-removal, .19/.7 alloc claims) | **YES — sq-7d3dj.22**: the plans of .6/.7/.16/.19 all cite "allocation counts" against an instrument that does not exist |
| HTTP throughput | `bench/serve-throughput/` harness (req/s, p50/p99, peak RSS) — landed by .5; canonical numbers EC2-only | .12, .13 (both unblocked by .5) | **YES — sq-7d3dj.23**: the harness has **no TTFB metric** (verified by inspection), yet TTFB is Wave D's declared win metric |
| Ingest time | `parse_ns_per_byte` (advisory, pinned corpus) + byte-identity differential suites | .6, .9, .21, sq-dzfzq | none |
| WASM bundle size | `wasm_bundle_bytes` deterministic ratchet (raw artifact) + site first-load guard (#1600) | .14 (shipped-size trend), .15 (sibling bundles) | none new (both known gaps already beaded) |

The two new beads are **instrument** beads, deliberately: the epic's own discipline
says deterministic counters in CI, wall-clock only on the EC2 rigs — and both gaps
block other beads' *measured-before-after* invariant rather than being missing
optimizations. No new optimization idea earned a bead in this pass; the roadmap's
negative space ("already-optimised — do NOT touch") was re-checked and stands.

## 3. Disjointness corrections (the load-bearing finding of this pass)

The prior records ordered beads by impact-per-risk but did not pin file areas.
Verified file map for every open child:

| File / surface | Beads touching it | Consequence |
|---|---|---|
| `crates/sparq-engine/src/exec.rs` | .4, .7, .11 | NON-parallel. Edges added: `.7 ← .4` (was already `.11 ← .4`) |
| `crates/sparq-substrate/src/join.rs` | .19, .20 (both `probe_emit` and `Leapfrog` live here) | NON-parallel. Edge added: `.19 ← .20` (.20 is the lower-risk, M4-independent one) |
| `crates/sparq-core/src/store.rs` (+ `dict.rs`, `shared.rs`) | .6, .8, .16, sq-dzfzq | one per wave |
| `crates/sparq-core/src/nt.rs` + `dict.rs` | .9, .21 | one per wave (and .9/.21 also collide with the row above via `dict.rs`) — treat sparq-core as ≤1 bead per wave, full stop |
| `crates/sparq-server/src/http.rs` | .12 | free |
| `research/` (new file only) | .13 | free |
| `scripts/ci-bench.sh` | .14 | free — and .22 is explicitly scoped NOT to touch this file |
| `js/package.json` | .15 | free |
| `bench/serve-throughput/src/main.rs` | .23 | free |
| `bench/alloc-track/**` (new) | .22 | free |

Measurement edges also added (§2 rationale): `.6 ← .22`, `.7 ← .22`, `.19 ← .22`.
All open children now carry `area:*` + `tier:*` labels so the disjointness and
model routing are machine-readable at dispatch time.

## 4. Measurement discipline under the current constraint (EC2 quota out)

Work-box numbers are non-canonical (standing rule), and the EC2 wall-clock rig is
unavailable until quota returns. Consequence for sequencing, applied to the wave:

- **Dispatch now** — beads whose acceptance is *deterministic* (bytes, byte-identity
  differentials, series plumbing, design prose): `.12`, `.13`, `.14`, `.15`, `.22`,
  `.23`. `.12`'s hard acceptance is the byte-identical-body differential plus a
  chunked-emission unit test; its peak-RSS *win claim* is recorded on the EC2 trend
  later, and the PR must make no numeric perf claim.
- **HOLD (ready-pending-EC2 or pending-.22)** — beads whose *measured-before-after*
  invariant needs wall-clock or allocation counts: `.4`, `.20` (wall-clock trend
  A/B), `.6`, `.7`, `.19` (unblock when `.22` lands), `.8`, `.9`, `sq-dzfzq`, `.21`
  (EC2 A/B by design), `.11` (blocked on `.4`), `.16` (measure-first). Holding is
  the honest reading of the epic's "implements + MEASURES": landing the code while
  deferring the measurement indefinitely would make the win claim unfalsifiable.
- When EC2 quota returns, the next wave is `.4` (engine-exec) + `.20`
  (substrate-join) + one sparq-core bead (suggest `.6`, whose alloc win will by then
  be provable deterministically via `.22`) — still one bead per file area.

## 5. Wave-3 dispatch set (file-disjoint; one impl PR per bead, opened by the fleet)

| Bead | Crate/surface | Tier | Invariant (all inherit: no quality/ratchet regression) | Acceptance |
|---|---|---|---|---|
| sq-7d3dj.12 | `sparq-server` (`src/http.rs`) | sonnet | byte-identical CSV/TSV bodies vs the buffered path; hardening middleware untouched | differential body test + chunked-emission unit test, `cargo test -p sparq-server` |
| sq-7d3dj.13 | `research/` only (design record) | fable (opus fallback) | design-only — no crate code; impl children must depend on .23 | record lands with decomposed impl beads + explicit Content-Length contract change |
| sq-7d3dj.14 | `scripts/ci-bench.sh` | haiku | `wasm_bundle_bytes` gate behaviour bit-identical; new series trend-only | shipped-size series emitted beside the ratchet; `perf-gate.py` untouched |
| sq-7d3dj.15 | `js/package.json` | haiku | build-config only; main bundle + ratchet untouched | sibling bundles build `--profile release-wasm`; before/after sizes in the PR body (reference, non-ratcheted) |
| sq-7d3dj.22 | `bench/alloc-track/**` (new) | sonnet | bench-only; reproducible counts (or documented variance band); no gate moved | two consecutive runs emit identical series; allocator unit test in the harness crate |
| sq-7d3dj.23 | `bench/serve-throughput` | haiku | bench-only; existing series additive-unchanged | harness emits the TTFB series for the large-SELECT scenario |

Everything else in the epic stays exactly as specified by the two prior records —
this record adds no optimization content, only the delta above.
