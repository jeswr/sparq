# bench/zk-compose

Benchmarks for the ZK proof-composition layer (`crates/sparq-zk-compose` +
the `zk/compose/` Noir circuit family).

Model: Opus 4.8 (Fable 5 unavailable — flag for re-review/upgrade when Fable
returns). Numbers below were measured by Opus 4.8.

## What is measured

1. **Gate counts** (`bb gates -s ultra_honk`, ground truth per the
   noir-optimisation skill) for every compiled circuit-family member, in the
   `zk/ieee754` JSON convention (`circuit_size`).
2. **Wall-clock prove/verify** for the small credential-scale e2e cases.

## Files

| file                          | contents |
|-------------------------------|----------|
| `gate_counts_latest.json`     | per-member ultra_honk gate counts |
| `prove_verify_timing.json`    | bb prove/verify wall-clock + proof sizes |
| `scripts/gate_counts.sh`      | regenerate the gate-count JSON |
| `scripts/prove_verify.sh`     | time prove+verify for one member |

## Reproduce

```sh
# gate counts (compiles the workspace, emits JSON to stdout)
bench/zk-compose/scripts/gate_counts.sh > bench/zk-compose/gate_counts_latest.json

# prove/verify timing for one member (needs a Prover.toml; the crate e2e tests
# leave real ones behind, or write one by hand)
bench/zk-compose/scripts/prove_verify.sh filter_int_d1
```

Toolchain: nargo 1.0.0-beta.21, bb 5.0.0-nightly.20260324.

## Latest results (darwin arm64, bb num_threads 8)

### Gate counts (ultra_honk `circuit_size`)

| member            | k | n  | r | circuit_size |
|-------------------|---|----|---|--------------|
| scan_k1_n16_r4    | 1 | 16 | 4 | 5,958 |
| scan_k2_n16_r8    | 2 | 16 | 8 | 11,011 |
| scan_k2_n64_r8    | 2 | 64 | 8 | 34,379 |
| filter_int_d1     | — | —  | — | 17,416 |
| filter_int_d2     | — | —  | — | 17,416 |
| filter_int_d4     | — | —  | — | 17,416 |
| filter_f64        | — | —  | — | 3,113 |

Notes:
- The scan members scale roughly linearly in `k * n` (the commitment-recompute
  sweep + the completeness double-loop dominate); `r` adds the row-soundness
  pass.
- `filter_int_d{1,2,4}` are **identical in gate count** (17,416): the blake3
  blackbox over the canonical token is the cost driver, and the token fits one
  64-byte blake3 block for all `d <= 19`, so digit count does not move the
  circuit size. The `d` family parameter exists only because the blackbox
  needs a comptime byte length; it leaks `ceil(log10(value))`, not gates.
- `filter_f64` (3,113) is the cheapest member — a pure `sparq_ieee754`
  comparison, no string hashing. It is a tested building block, not yet
  manifest-composable (operand binding deferred; see the crate README).

### Prove / verify wall-clock (small e2e)

| member          | prove (s) | verify (s) | proof bytes |
|-----------------|-----------|------------|-------------|
| filter_int_d1   | 1.13      | 0.16       | 14,656 |
| scan_k1_n16_r4  | 1.62      | 0.95       | 14,656 |

`prove` includes `--write_vk`. Proof size is constant (ultra_honk) regardless
of circuit size; verify time grows with public-input count (scan carries the
commitment + rows vectors, hence the higher verify cost).
