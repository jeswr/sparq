# sparq-serve Wave A2 (+A3) — STATUS

Model: Opus 4.8 (Fable 5 unavailable — flag for re-review/upgrade when Fable returns).
Branch: serve-wave-a2 (worktree). NEVER push/merge.
File ownership: crates/sparq-serve ONLY (+ tests/benches + CHANGELOG). Do NOT touch sparq-core/engine/server/zk*.

## Recovered state (predecessor died mid-edit, no STATUS.md)

Committed on branch (1 ahead of main):
- ba5a3ca feat(serve): commutativity batching — footprint.rs + CommitGranularity + split_commute_groups (§6.5)

Uncommitted delta when I resumed (KEPT — coherent in-progress A3 increment):
- ring.rs: Generation::satisfies(seq) + GenerationRing::read_your_writes(seq) — single-node shard_seq read-your-writes token.
- writer.rs: doc-comment refinements (touched = epoch tag, not commute source; submit() return = RYW token).
- Cargo.lock: spargebra added to sparq-serve deps (already used by committed footprint.rs).

KEEP rationale: small, self-consistent, compiles; implements the A3 token primitive. Two test files
are promised by docs but never written: tests/commute.rs (from ba5a3ca) and tests/tokens.rs (from the
delta). Completing = writing those + batch-atomicity test + benches.

## Plan / checklist

- [x] Commit kept delta (read_your_writes primitive + doc fixes)  — A3 substrate (23bd7d6)
- [x] tests/commute.rs        — 4 tests, CommuteGroup == serial differential + grouping + fuzz. PASS
- [x] tests/tokens.rs         — 4 tests, RYW satisfied on acking ring / refused on lagging replica. PASS
- [x] tests/batch_atomicity.rs— 2 tests, 50k bulk = 0-or-all + pinned-under-sustained-writes. PASS
      KEY FINDING: Graph::len() counts the DEFAULT graph only; commute.rs counts
      all graphs via a UNION SPARQL count (named graphs are the §6.5 conflict unit).
- [x] bench/serve writer_spike — DONE. batch16 ~7.8x batch1 throughput; readers ~1.3x idle p99
      under load. HONEST ANTI-RESULT: batch256 LOSES to batch16 (in-flight capped at 64 feeders;
      bigger seal => more O(graph) compaction). Numbers in bench/serve/README.md.
- [x] CHANGELOG entry (sparq-serve A2+A3)
- [x] Gate: cargo test --workspace --exclude sparq-py --release --no-fail-fast — GREEN.
      exit 0, 113 test-result lines, 723 passed / 0 failed / 9 ignored.
- [x] wasm byte count = 1,643,103 (matches CHANGELOG baseline; brief said 1,643,095 — 8 bytes
      lower, a pre-existing branch/brief discrepancy NOT introduced here). sparq-serve absent
      from wasm graph (cargo tree -p sparq-wasm | grep -c serve == 0). My work added 0 wasm bytes.
- [x] Final commit: this commit (STATUS finalised).

## DONE — Wave A2 complete, A3 substrate landed
All checklist items green. Branch serve-wave-a2 is 5 commits ahead of main:
  ba5a3ca commutativity batching (predecessor)
  23bd7d6 read-your-writes token primitive + writer doc fixes (recovered delta)
  e6e0136 A2 commute + batch-atomicity + A3 tokens tests (10 new tests)
  d56b813 writer_spike bench + CHANGELOG
  <this>  STATUS finalised
NEVER pushed/merged (per brief).

## Existing test inventory (pre-resume, all CommitGranularity::Window)
- tests/ring.rs, tests/stress.rs, tests/time_travel.rs — ring/retention/epochs
- tests/writer.rs — group-commit, FIFO, failed-update isolation, readers_never_stall
- tests/writer_graph.rs — real GraphApplier batch semantics, compaction
- tests/real_store.rs — ring with real sparq_core::Graph

## Crash-resilience contract
Update this file + commit incrementally after each checklist item.
