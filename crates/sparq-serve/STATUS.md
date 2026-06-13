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

## In flight / next exact command

- [ ] Run scheduler tests:
      `cargo test -p sparq-serve --release --test scheduler --test scheduler_real_store`
- [ ] Full gate:
      `cargo test --workspace --exclude sparq-py --release --no-fail-fast 2>&1 | grep -aE "^test result"`
- [ ] wasm unchanged:
      `cargo build -p sparq-wasm --target wasm32-unknown-unknown --release` + stat byte count;
      `cargo tree -p sparq-wasm -e normal | grep -c serve` == 0.
- [ ] Final commit + record SHA here.

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
