# sparq-serve Wave B (read-side scheduler) — STATUS

Model: Opus 4.8 (Fable 5 unavailable — flag for re-review/upgrade when Fable returns).
Branch: serve-wave-b (worktree). NEVER push/merge.
File ownership: crates/sparq-serve ONLY (+ tests/benches + CHANGELOG). Do NOT touch sparq-core/engine/server/zk*.

## Scope (this wave)

Research doc §8 labels the scheduler **Wave C**; the brief tags it serve-"Wave B" =
goal #4 requirements 3 + 4 (cheap-query prioritisation + no head-of-line blocking),
which is exactly §6.2's two-tier execution design. Followed the brief's scope; the
§6.2 design and the litreview-C Umbra mechanism are the references. (Result cache /
streaming / tier-0 inline-execution remain later waves.)

## Done

- [x] `src/scheduler.rs` — cost-aware bounded thread pool, two lanes (cheap/heavy),
      reserved cheap capacity (no-HoL), Umbra SRPT-approx + unbounded aging in the
      heavy lane, panic isolation, SEDA shed-on-shutdown. Sync, runtime-agnostic,
      `std`-only (no new crate deps; wasm-graph guarantee trivially preserved).
- [x] `src/lib.rs` — module wired, public API exported (`Scheduler`, `SchedulerConfig`,
      `Ticket`, `Lane`, `Cost`, `SchedError`, `P0`, `DEFAULT_HEAVY_THRESHOLD`), crate
      docs updated.
- [x] `tests/scheduler.rs` — empirical-honesty suite (correctness/differential, lane
      classification, bounded heavy concurrency, head-of-line p99 containment
      [open-loop, coordinated-omission-safe], starvation-freedom, SRPT+aging ordering
      [deterministic], no-regression all-cheap vs plain pool, panic isolation,
      shutdown shed).
- [x] `tests/scheduler_real_store.rs` — scheduler over the production substrate
      (ring + real `sparq_engine` + sequenced `Writer`): result-equality vs direct
      execution, snapshot consistency preserved under concurrent writes.
- [x] CHANGELOG entry (Wave B).

## Verification — DONE (all green)

- [x] Scheduler tests: `scheduler.rs` 11 passed, `scheduler_real_store.rs` 2 passed.
- [x] Full gate `cargo test --workspace --exclude sparq-py --release --no-fail-fast`:
      exit 0, 118 `test result` lines, **746 passed / 0 failed / 10 ignored**.
- [x] wasm byte count = **1,643,103** (== A2 baseline; brief's 1,643,095 is the
      pre-existing 8-byte branch/brief discrepancy A2 already recorded — Wave B
      added ZERO wasm bytes: std-only, no new deps, not in wasm graph).
- [x] wasm-graph guard: `cargo tree -p sparq-wasm -e normal | sed 's| (/[^)]*)||g'
      | grep -cE 'sparq-serve\b'` == **0**. (The naive `grep -c serve` reports a
      false positive because the worktree dir is `sparq-serveb` — strip paths first.)
- [x] Implementation commit: 5ec347b.

## Headline empirical results (MEASURED UNDER CONCURRENT LOAD — caveat applies)

- **HoL (the win):** lane-split cheap p99 ~0.5–5 ms under 8 concurrent 200 ms scans;
  FIFO single-lane cheap p99 ~**398 ms** under the same load → ~80–750× HoL
  containment. A 200 ms scan never leaks into cheap p99 with the scheduler.
- **Starvation:** the heavy job completes alongside 1000+ flooded cheap jobs
  (reserved heavy slot runs it in parallel; aging guarantees no indefinite postpone).
- **No-regression (HONEST, non-zero):** all-cheap micro-workload (N=20k, ~300 ns/job
  work): scheduler ~2.2× a plain pool = **~0.5 µs/job** added absolute overhead
  (per-job ticket alloc + mutex/condvar vs a plain channel). ~6% of a 9 µs point
  query, negligible against anything heavier — but NOT zero, reported as such. Gate
  is 3.5× (a tripwire for a real machinery blow-up); the absolute-ns datum the test
  prints is the honest measure.
- Numbers taken while other heavy agents shared the machine — ratios are the signal;
  absolute throughput would need a quiet re-run for final claims. Structural
  properties (ordering, no-starvation, result equality, bounded concurrency) are
  load-independent and asserted hard.

## Honesty notes

- Timing numbers in tests are MEASURED UNDER CONCURRENT LOAD (other heavy agents
  running) — gates are deliberately wide; the printed ratios are the real data and
  may need a quiet re-run for final claims. Structural properties (ordering,
  no-starvation, result equality, bounded concurrency) are load-independent and
  asserted hard.
- True preemption is OUT of scope (engine has no re-entrant suspend/resume, §3.2);
  the heavy-concurrency cap + reserved cheap capacity deliver the no-HoL property
  without it. This matches §5 verdict 4 (true preemption REJECTED).

## Crash-resilience contract

Update this file + commit incrementally after each milestone.
