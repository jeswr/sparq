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

- [ ] Commit kept delta (read_your_writes primitive + doc fixes)        — A3 substrate
- [ ] tests/commute.rs        — CommuteGroup correctness == serial (differential), barrier handling
- [ ] tests/tokens.rs         — RYW: submit() token satisfied on this ring; not-yet on a lagging ring
- [ ] tests/batch_atomicity.rs— 50k-triple bulk update: reader sees 0-or-all (mirrors readers_are_not_blocked)
- [ ] bench/serve update_writer_spike — writer throughput @ batch 1/16/256; reader p50/p99 vs A1
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
