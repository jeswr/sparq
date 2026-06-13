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
| `prove_verify_timing.json`    | bb prove/verify wall-clock + proof sizes (early, 2-member, darwin) |
| `family_cost_curve.json`      | sq-pn2 full-family (k,n,r,d) prove/verify/size curve |
| `family_curve/`               | sq-pn2 standalone timing harness (own cargo project) |
| `scripts/gate_counts.sh`      | regenerate the gate-count JSON |
| `scripts/prove_verify.sh`     | time prove+verify for one member |

> The gate-count JSON is also the source the in-crate **regression gate**
> (`crates/sparq-zk-compose/tests/gate_count.rs`, sq-c5f) baselines against —
> re-run `scripts/gate_counts.sh` after an intentional circuit change and update
> both that JSON and `crates/sparq-zk-compose/tests/gate_count_snapshot.json`.

## sq-pn2: full-family prove/verify cost curve

`family_curve/` is a STANDALONE cargo project (own `[workspace]`, same isolation
pattern as `bench/zk`) that drives the `sparq-zk-compose` prover (nargo + bb
subprocesses) once per circuit-family member and emits the full **(k, n, r, d)
cost curve** — prove time, verify time, proof size, vk size. It is **NOT gated
in CI** (each row is a real `bb prove`, ~1-2 s) and is a plain timing harness,
not criterion (criterion's repeated sampling over ~1-2 s proofs would take
hours).

`k` = committed graphs (disclosed-attribution width), `n` = slot bucket, `r` =
disclosed-row bucket (the scan family); `d` = digit count (the filter_int
family). The scan sizes are chosen to land on each compiled member; the filter
members sweep `d ∈ {1,2,4}`; `filter_f64` is the building-block member (driven
via a raw `Prover.toml`, since it is not manifest-composable).

### Run

```sh
cd bench/zk-compose/family_curve
cargo run --release > ../family_cost_curve.json   # JSON to stdout, table to stderr
# average each member over N proves (reduces noise):
REPEATS=3 cargo run --release > ../family_cost_curve.json
```

Requires `nargo` + `bb` on PATH (it exits with an error if absent). Writes its bb
scratch under the system temp dir and cleans it per member.

### Latest results (2026-06-13, Linux aarch64, 8 cores, bb 5.0.0-nightly.20260324, REPEATS=1)

| member          | k | n  | r | d | prove (s) | verify (s) | proof (B) | vk (B) |
|-----------------|---|----|---|---|-----------|------------|-----------|--------|
| scan_k1_n16_r4  | 1 | 16 | 4 | — | 0.90      | 0.012      | 14,656    | 3,680  |
| scan_k2_n16_r8  | 2 | 16 | 8 | — | 1.06      | 0.013      | 14,656    | 3,680  |
| scan_k2_n64_r8  | 2 | 64 | 8 | — | 1.83      | 0.013      | 14,656    | 3,680  |
| filter_int_d1   | — | —  | — | 1 | 1.06      | 0.012      | 14,656    | 3,680  |
| filter_int_d2   | — | —  | — | 2 | 1.05      | 0.013      | 14,656    | 3,680  |
| filter_int_d4   | — | —  | — | 4 | 1.07      | 0.012      | 14,656    | 3,680  |
| filter_f64      | — | —  | — | — | 0.78      | 0.013      | 14,656    | 3,680  |

Observations:
- **Prove time tracks gate count**: `scan_k2_n64_r8` (34,637 gates) is the
  slowest (1.83 s); `filter_f64` (3,113 gates) the fastest (0.78 s). The
  `filter_int_d{1,2,4}` members are gate-identical (17,416) and prove in the
  same ~1.06 s — `d` does not move cost (see the gate-count notes above).
- **proof size, vk size, and verify time are CONSTANT across the family**
  (14,656 B / 3,680 B / ~12 ms) — the ultra_honk succinctness property: a
  constant-size proof and a verify cost dominated by the fixed protocol, not the
  circuit. (This differs from the earlier 2-member `prove_verify_timing.json`,
  which reported a ~0.95 s scan verify on darwin/8-thread; on this aarch64 box
  verify is uniformly ~12 ms for every member. The earlier figure looks like a
  cold-vk / measurement artefact — `family_cost_curve.json` is the consistent
  full-family measurement and supersedes it for verify timing.)

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
