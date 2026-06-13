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
- [ ] bench/serve writer_spike — writer throughput @ batch 1/16/256; reader p50/p99 vs A1
- [ ] CHANGELOG entry
- [ ] Gate: cargo test --workspace --exclude sparq-py --release --no-fail-fast | grep -aE "^test result"
- [ ] wasm byte count unchanged @ 1,643,095 (sparq-serve must NOT enter wasm builds)
- [ ] Final commit SHA recorded here

## Existing test inventory (pre-resume, all CommitGranularity::Window)
- tests/ring.rs, tests/stress.rs, tests/time_travel.rs — ring/retention/epochs
- tests/writer.rs — group-commit, FIFO, failed-update isolation, readers_never_stall
- tests/writer_graph.rs — real GraphApplier batch semantics, compaction
- tests/real_store.rs — ring with real sparq_core::Graph

## Crash-resilience contract
Update this file + commit incrementally after each checklist item.
