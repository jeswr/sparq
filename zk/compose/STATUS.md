# sparq-zk-compose — STATUS

Model: Opus 4.8 (Fable 5 unavailable — flag for re-review/upgrade when Fable returns).

ZK stage 2: composition prover + verifier for sparq. Worktree `sparq-zkcompose`,
branch `zk-compose`.

## Milestone log

### M0 — inherited scaffold validation (DONE)

Fable-authored scaffold at `zk/compose/` reviewed critically and validated:

- `nargo check` clean (nargo 1.0.0-beta.21).
- `nargo test --package sparq_zk_compose_core`: **22/22 pass** (accept + adversarial).
- Bit-compatibility CONFIRMED: the cross-vectors in `compose_core/src/tests.nr`
  are byte-for-byte identical to `crates/sparq-zk/tests/poseidon2_noir_cross.rs`
  fixture outputs (same poseidon lib tag v0.3.0, same `Poseidon2::hash`,
  length-bearing IV). h2/h3/commit_fold reproduce `0x168758…`, `0x038682…`,
  `0x23864a…`, `0x130bf2…`, `0x0ceb49…`, `0x046964…`.
- `filter_int` operand binding CONFIRMED against `sparq-zk/src/encode.rs` +
  `field.rs`: `field_from_hash_bytes = bytes[1..]` (low 31 bytes BE) matches the
  in-circuit `for i in 1..32 { hs = hs*256 + digest[i] }`; literal token
  `"<digits>"^^<…#integer>` matches oxrdf `Literal::to_string()`.

Verdict: scaffold is GOOD; committed as baseline unchanged. Added `.gitignore`
(target/) and this STATUS.

### Circuit family inventory (compiles)

| member            | K | N  | R | relation        |
|-------------------|---|----|---|-----------------|
| scan_k1_n16_r4    | 1 | 16 | 4 | scan_check      |
| scan_k2_n16_r8    | 2 | 16 | 8 | scan_check      |
| scan_k2_n64_r8    | 2 | 64 | 8 | scan_check      |
| filter_int_d1     | — | —  | — | filter_int (D=1)|
| filter_int_d2     | — | —  | — | filter_int (D=2)|
| filter_int_d4     | — | —  | — | filter_int (D=4)|
| filter_f64        | — | —  | — | filter_f64 (building block, not manifest-composable v1) |

## In flight / next

- Rust crate `crates/sparq-zk-compose` (ProofManifest + prover + verifier).
- Gate-count benches under `bench/zk-compose/`.
- e2e prove→verify + tamper tests.

### Exact next command

    cd /Users/jesght/Documents/GitHub/rdfjs/sparq-zkcompose && nargo check --program-dir zk/compose
