# zk-core branch status

Successor agent resuming 2026-06-12. Predecessor left 3 commits, no STATUS.md.

## Scope audit (stage-1 core crate, plan research/zkp-query-proofs-plan.md v3)

| # | Item | Status |
|---|------|--------|
| 1 | sparq-zk crate + RDFC10 vs W3C rdf-canon suite | DONE (predecessor) — VERIFIED: suite test asserts 86-entry manifest (64 eval + 21 map + 1 negative), all pass (`w3c_rdf_canon_suite` ok) |
| 2 | Poseidon2-BN254 commitments + Noir cross-vectors | DONE (predecessor) — VERIFIED: `nargo_live_cross_check` ran live nargo 1.0.0-beta.21 (102s), bit-identical |
| 3 | <urn:sparq:zk> registry | DONE (predecessor) — 5 registry tests pass |
| 4 | zk-trace seam, non-default `zk` feature | DONE (predecessor) — `#[cfg(feature = "zk")]`-gated module, zero code when off; wasm gate measured separately (item 7) |
| 5 | bnode correlation guard | DONE (successor): verifier re-check hook in src/verify.rs (plan §2.4 layer 3 — independent spargebra re-parse, cross-graph join obligations, recheck()); prover guard now covered by tests/trace_guard.rs (7 integration tests incl. colliding-label rejection and the label-identified-union leak) — commit 7321168 |
| 6 | Criterion benches + baselines | DONE (successor): standalone bench/zk (bench/parse isolation pattern), baselines in bench/zk/README.md — commit 4941c40 |
| 7 | wasm size gate (feature off) | DONE (successor): baseline (4901be8, same path) 1,593,075 B == HEAD 1,593,075 B. NOT bit-identical: 197 bytes differ — all panic-location LINE NUMBERS in sparq-engine (cfg'd-out zk hook lines shift line numbers of following code); zero code/size change |
| 8 | workspace tests green | DONE: cargo test --workspace --exclude sparq-py --release --no-fail-fast → 712 passed, 0 failed (exit 0; a few ignored, all pre-existing) |

sparq-zk test totals now: 24 lib (incl. 6 verify) + 4 noir-cross + 2 rdf-canon + 7 trace_guard = 37, all pass (release).
Post-workspace-run fix ec767fb (verify.rs only: reject (NOT) EXISTS — fragment walk missed the GraphPattern inside FILTER expressions); sparq-zk re-run green 37/37, no other crate touched since the workspace run.

## Bench baselines (fanless M1, 2026-06-13; full table in bench/zk/README.md)
- canon: ~73k-153k triples/s (iri/bnode, 64-1024)
- commit end-to-end: ~4.1k-7.5k triples/s; leaves+fold: ~7k-9.7k triples/s
- poseidon2: permutation 92.5 us, hash40 583 us (correctness-first port; headroom documented)

## COMPLETE
All 8 scope items done. Branch zk-core ready for orchestrator merge gate (do not merge here).
Known stage-1 caveats, deliberate and documented in code:
- The union store identifies equal bnode LABELS across graphs (RDF merge forbids this);
  the Q6 guard rejects exactly the executed plans whose results could depend on it
  (tested: label_identified_union_leak_is_caught).
- Verifier re-check is coarser than the prover guard by design (cannot see private bnode values).
- Poseidon2 is a correctness-first port (~92 us/permutation); optimization headroom documented
  in bench/zk/README.md — do not optimize before a stage needs it.
