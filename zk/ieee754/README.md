# sparq_ieee754 — IEEE 754 binary floating point for Noir

> **Provenance.** Vendored from the private `jeswr/test-lib` working tree on
> 2026-06-12. At vendoring time that tree was 76 commits ahead of its remote
> (HEAD `8a6fe4b`), plus uncommitted `src/ops/kernels.nr` edits that contain
> the newest kernel work; the working tree was copied as-is. The package was
> renamed from `test_lib` to `sparq_ieee754`. This subfolder is a temporary
> home: the library is to be broken out into its own repository and upstreamed
> later.
>
> Verified on vendoring: `nargo 1.0.0-beta.21` + `bb 5.0.0-nightly.20260324`;
> all tests pass and a fresh gate benchmark
> (`bench/float_ops_baseline-nargo-beta21.json`) reproduces the historical
> baseline (`bench/float_ops_latest.json`) with zero delta.

A Noir library providing IEEE 754 `f16`, `f32`, `f64`, and `f128` types,
generated at compile time from a single width parameter, with
gate-count-optimised arithmetic kernels (hint-and-verify witnesses, bounded
pow2 shift proofs, normalized-only round-pack paths).

## Usage

Add a path dependency:

```toml
[dependencies]
sparq_ieee754 = { path = "../zk/ieee754" }
```

```noir
use sparq_ieee754::{f16, f32, f64, f128};

fn main(a_bits: u64, b_bits: u64) -> pub u64 {
    let a = f64::new(a_bits);     // construct from raw IEEE 754 bits
    let b = f64::new(b_bits);
    let c = (a + b) * a / b - b;  // Add/Sub/Mul/Div are implemented
    c.bits()                      // recover raw bits
}
```

### Public API

The public API is intentionally only the generated `f16`, `f32`, `f64`, and
`f128` structs:

- `new(bits)` — construct from raw bits (`u16`/`u32`/`u64`/`u128`);
  constrains `(sign, exponent, mantissa)` to canonical IEEE field widths so
  `bits()` is injective (soundness fix sq-3x7dl.1).
- `bits()` — recover raw bits.
- `std::ops::Add`, `Sub`, `Mul`, `Div` — round-to-nearest-even arithmetic with
  full subnormal, infinity, and NaN handling (NaNs are canonicalised).
- `std::convert::From<u8 | u16 | u32 | u64 | u128 | i8 | i16 | i32 | i64>` —
  integer-to-float conversion with IEEE round-to-nearest-even.

Everything else (`FloatParts`, the `ops` kernels, `codegen`, `sizing`,
generated struct fields, `to_parts`) is private by design and enforced by
`scripts/test_public_api.sh` and `scripts/lint_private_function_usage.py`.

## Layout

- `src/lib.nr` — crate root; root-level generated-type triggers and tests.
- `src/codegen.nr` — comptime generation of the public float structs and impls
  via `#[generate_float_type(N)]`.
- `src/parts.nr` — internal `FloatParts<E, M>` carrier.
- `src/sizing.nr` — comptime layout/type helpers.
- `src/ops/kernels.nr` — arithmetic kernels: u64 kernel for f16/f32/f64, wide
  `u128`/Field kernel for f128, conversion helpers, proof helpers.
- `tests/` — external-package test sources (public API surface, private-field
  and private-method rejection, generated arithmetic vectors). These are
  copied into temporary packages by the scripts below, not built in place.
- `scripts/` — test and benchmark harnesses (see `scripts/README.md`).
- `bench/` — committed gate baselines (see `bench/README.md`).
- `AGENTS.md` — agent handoff: invariants, optimisation rules, known-good and
  known-bad patterns.

## Validation

```sh
nargo test --silence-warnings
bash ./scripts/test_generated_vectors.sh
bash ./scripts/test_public_api.sh
python3 scripts/lint_private_function_usage.py
```

## Benchmarks

`scripts/benchmark_float_ops.py` is the canonical amortised UltraHonk gate
harness: it builds temporary binary packages, runs `nargo compile`, reads
`circuit_size` from `bb gates -s ultra_honk` at small-N and big-N, and
estimates per-call cost from the difference.

```sh
python3 scripts/benchmark_float_ops.py --output /tmp/candidate_float_ops.json
python3 scripts/compare_float_benchmarks.py /tmp/candidate_float_ops.json --max-regression 1
```

Baselines (per-call gate estimates, `--n-small 1 --n-big 8`):

- `bench/float_ops_latest.json` — historical baseline vendored from test-lib
  (measured 2026-05-21, `nargo 1.0.0-beta.21`, `bb 5.0.0-nightly.20260324`).
- `bench/float_ops_baseline-nargo-beta21.json` — fresh baseline recorded
  2026-06-12 on the same toolchain after vendoring; identical per-call counts.

| Size | Add | Sub | Mul | Div |
| --- | ---: | ---: | ---: | ---: |
| `f16` | `341.9` | `342.0` | `283.7` | `255.6` |
| `f32` | `446.0` | `446.0` | `355.7` | `367.9` |
| `f64` | `367.3` | `367.4` | `307.6` | `273.4` |
| `f128` | `630.1` | `630.1` | `543.9` | `524.6` |

## Known gaps (relevant to SPARQL/XPath operator support)

Not yet implemented; tracked for the sparq engine work:

- Comparison predicates (`eq`/`lt`/`le`/`gt`/`ge` with IEEE NaN semantics) —
  only internal significand-magnitude helpers exist.
- `sqrt`.
- Round-to-integral (`round`/`floor`/`ceil`/`trunc`).
- Float-to-integer casts (integer-to-float exists; the reverse does not).
- Rounding-mode-explicit entry points (round-to-nearest-even is hard-coded).
