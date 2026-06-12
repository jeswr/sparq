# zk-core branch status

Successor agent resuming 2026-06-12. Predecessor left 3 commits, no STATUS.md.

## Scope audit (stage-1 core crate, plan research/zkp-query-proofs-plan.md v3)

| # | Item | Status |
|---|------|--------|
| 1 | sparq-zk crate + RDFC10 vs W3C rdf-canon suite | DONE (predecessor) — VERIFIED: suite test asserts 86-entry manifest (64 eval + 21 map + 1 negative), all pass (`w3c_rdf_canon_suite` ok) |
| 2 | Poseidon2-BN254 commitments + Noir cross-vectors | DONE (predecessor) — VERIFIED: `nargo_live_cross_check` ran live nargo 1.0.0-beta.21 (102s), bit-identical |
| 3 | <urn:sparq:zk> registry | DONE (predecessor) — 5 registry tests pass |
| 4 | zk-trace seam, non-default `zk` feature | DONE (predecessor) — `#[cfg(feature = "zk")]`-gated module, zero code when off; wasm gate measured separately (item 7) |
| 5 | bnode correlation guard | PARTIAL: prover-side `bnode_guard` in trace.rs exists but UNTESTED (trace.rs has 0 tests); verifier re-check hook (plan §2.4 layer 3) NOT IMPLEMENTED (spargebra dep staged but unused) — MINE |
| 6 | Criterion benches + baselines | NOT DONE (placeholder src/bin/bench.rs) — MINE |
| 7 | wasm size gate (feature off) | NOT MEASURED — MINE (compare HEAD vs branch-point 4901be8) |
| 8 | workspace tests green | NOT RUN — MINE |

sparq-zk test totals as of audit: 18 lib + 4 noir-cross + 2 rdf-canon = 24, all pass (release).

## In flight
- Writing crates/sparq-zk/src/verify.rs (verifier-side static re-check, plan §2.4 layer 3)
  + crates/sparq-zk/tests/trace_guard.rs (prover guard + traced-execution integration tests).

## Next steps if killed
1. Finish verify.rs + trace_guard.rs, `cargo test -p sparq-zk --release`.
2. Criterion bench crates/sparq-zk/benches/, baselines into bench/zk/README.md, drop placeholder bin.
3. wasm gate: build sparq-wasm wasm32 release at HEAD and at 4901be8 (temp worktree), stat -f%z both.
4. cargo test --workspace --exclude sparq-py --release --no-fail-fast | grep -aE "^test result"
5. Update STATUS.md + final report.
