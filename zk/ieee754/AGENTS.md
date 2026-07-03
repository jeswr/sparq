# Agent Instructions

This repository is a Noir library for experimenting with compile-time code generation of public floating-point types. Treat this file as the handoff for coding agents working on the repo.

## Project Shape

- `src/lib.nr` is the crate root and contains the root-level generated-type triggers. Tests that reference generated `f16`/`f32` names must stay in the crate root because sibling test modules resolve before root attribute-generated names are available.
- `src/codegen.nr` generates public `f16`, `f32`, `f64`, and `f128` structs and their operator impls.
- `src/parts.nr` defines the internal `FloatParts<E, M>` carrier.
- `src/sizing.nr` contains comptime layout/type helpers.
- `src/ops/` contains the arithmetic kernels, conversion helpers, and proof helpers.
- `scripts/benchmark_float_ops.py` is the canonical UltraHonk gate benchmark harness.
- `scripts/compare_float_benchmarks.py` is the gate-regression guard.
- `bench/float_ops_latest.json` is the committed baseline.
- `bench/README.md` records benchmark context and accepted optimization patterns.

## Skills To Use

When the agent runtime supports skills, load the relevant skill before editing. If skills are unavailable, follow the summaries here.

- `noir-developer`: use for any Noir language, stdlib, generic, trait, module, workspace, or dependency question. Confirm current Noir syntax before writing unfamiliar patterns.
- `noir-idioms`: use when writing or reviewing Noir code. Prefer hint-and-verify when sound, keep constrained loops comptime-bounded, use clear boolean/conditional forms, and measure any clever comparison rewrite.
- `noir-testing`: use when adding or changing tests. Remember that Noir tests are constrained by default and that root-level tests are needed for generated global names in this crate.
- `noir-optimisation`: use for every gate-count, backend-cost, `bb gates`, `nargo info`, Field-vs-integer, or unconstrained-witness decision. Always verify optimization claims with the benchmark harness.
- `diagnose`: use for failing tests, incorrect vectors, benchmark regressions, or toolchain failures. Build a reproducible loop first, then minimize, hypothesize, instrument, fix, and regression-test.
- `dry-extract-on-second-use`: use when a change repeats logic that already exists across packages or repos. Do not prematurely abstract a first occurrence, and do not keep a second cross-package copy.
- `code-review-and-quality`: use before merging or handing off any nontrivial change, especially agent-written code. Review correctness, readability, architecture, security, performance, and verification evidence.

## Public API Invariants

- The public API is intentionally the generated `f16`, `f32`, `f64`, and `f128` types.
- Raw bits are constructed with `new(bits)` and recovered with `bits()`.
  `new()` enforces TWO in-circuit invariants: (1) `decoded.bits() == bits` and
  (2) `exponent < 2^EXP_SIZE` and `mantissa < 2^MANT_SIZE` for each width.
  Invariant (2) is the soundness fix from sq-3x7dl.1: without it the packing
  `bits() = sign*2^(W-1) + exponent*2^mant_size + mantissa` is non-injective
  and a malicious prover can substitute `{exp-1, mantissa+2^mant_size}` to
  prove false float statements.
- Generated fields, `to_parts`, and internal helpers must remain private.
- `scripts/test_public_api.sh` checks that external packages can import the generated types and cannot import helper internals.
- Generated public `type` aliases are not viable in current Noir; generated public structs are used instead.
- Noir `std::ops::Add`/`Sub`/`Mul`/`Div` impls use `fn add(self, other: Self) -> Self`-style methods with no Rust-style associated `Output` type.

## Optimization Rule

Use `bb gates -s ultra_honk` through the benchmark harness as the source of truth. Do not claim a gate saving from `nargo info` alone.

Work in small, reversible slices. For every candidate change:

1. Make one focused change.
2. Run correctness tests.
3. Run a candidate benchmark.
4. Compare against `bench/float_ops_latest.json`.
5. Commit only if the change is gate-neutral and meaningfully simpler, or strictly improves gates.
6. Revert candidates that regress gates or only add noise.

Do not push unless explicitly asked.

## Validation Commands

Run these before committing an accepted slice:

```sh
nargo test --silence-warnings
bash ./scripts/test_generated_vectors.sh
bash ./scripts/test_public_api.sh
python3 scripts/lint_private_function_usage.py
python3 scripts/benchmark_float_ops.py --output /tmp/candidate_float_ops.json
python3 scripts/compare_float_benchmarks.py /tmp/candidate_float_ops.json --max-regression 1
```

When promoting a new benchmark baseline:

```sh
cp /tmp/candidate_float_ops.json bench/float_ops_latest.json
```

Then update `bench/README.md` if the gate table or optimization notes changed.

Use unsigned commits if signing prompts block automation:

```sh
git commit --no-gpg-sign -m "<small descriptive message>"
```

## Current Gate Baseline

Measured with `bb 5.0.0-nightly.20260324` and `nargo 1.0.0-beta.21`, using `--n-small 1 --n-big 8`:

| Size | Add | Sub | Mul | Div |
| --- | ---: | ---: | ---: | ---: |
| `f16` | `341.9` | `342.0` | `283.7` | `255.6` |
| `f32` | `446.0` | `446.0` | `355.7` | `367.9` |
| `f64` | `367.3` | `367.4` | `307.6` | `273.4` |
| `f128` | `630.1` | `630.1` | `543.9` | `524.6` |

The SPARQL kernels (comparisons `eq`/`ne`/`lt`/`le`/`gt`/`ge`, the
round-to-integral family, `sqrt`, and `to_u64`/`to_i64`) are measured with the
XOR-fold harness pattern, so their per-call estimates include one `new()`
decode per call; see the table in `bench/README.md` and
`bench/float_ops_baseline-kernels.json`. `bench/float_ops_latest.json` carries
all 68 rows, so the default `compare_float_benchmarks.py` run guards both the
arithmetic and the kernel operations.

The current f32/f64 add, mul, and div counts are below the May-2026 `noir_IEEE754` reference amortized counts:

| Size | Add/Sub | Mul | Div |
| --- | ---: | ---: | ---: |
| `f32` reference | `634.7` | `533.7` | `577.3` |
| `f64` reference | `643.6` | `541.0` | `586.3` |

## Reference Implementation

Optimization ideas can be compared against the remote reference repository `jeswr/noir_IEEE754`, branch `copilot/ct-kernel-f16-storage`. Inspect it from GitHub or clone it into a temporary directory rather than assuming a local checkout exists.

Useful reference areas to inspect:

- `ieee754/src/float.nr`
- `ieee754/src/kernels/add.nr`
- `ieee754/src/kernels/mul.nr`
- `ieee754/src/kernels/div.nr`
- `ieee754/src/kernels/common.nr`
- `scripts/bench_amortised.py`
- `bench/optimisation_queue.md`

Treat the reference repository as read-only context. Do not edit it while working on this repo. If benchmarking against it, document any deliberate methodology differences in `bench/README.md`.

## Known-Good Patterns

- Keep the u64 kernel for `f16`/`f32`/`f64` and the wide `u128`/Field kernel for `f128` unless a candidate benchmark proves a generic alternative is neutral or better.
- Keep pow2 witnesses in `Field` where possible and verify with `assert_max_bit_size`/Field equations instead of casting down to integer types.
- Prefer power-of-two remainder proofs via mantissa-specific bit-size bounds when the divisor is a known power of two.
- Use unconstrained witnesses plus cheap in-circuit verification for quotient/product splits when the verifier fully constrains the result.
- Use normalized-only round-pack paths for mul/div after operation-specific normalization has already established the preconditions.
- Keep dynamic shift verifiers bounded to the live width for each path.
- Keep offset exponent work in `u16` where the range fits.
- Generic helpers are welcome at conversion boundaries and for proof recomposition when benchmarks stay neutral.
- Downcasts are often expensive; upcast to `Field` and stay there when possible.

## Known-Bad Or Risky Patterns

- A broad generic collapse of the hot inner kernels compiled and passed tests, but regressed gates badly. Do not repeat it unless measuring a much smaller candidate slice.
- Avoid Field/integer round trips in hot paths. Prior probes measured `Field -> u64 -> Field` around 28.5 gates/iteration and `Field -> u128 -> Field` around 34.7 gates/iteration, while `Field.assert_max_bit_size::<64>` and `<128>` were much cheaper.
- Avoid replacing bounded pow2 verifiers with wider generic ones; previous changes showed meaningful regressions.
- Avoid adding gate-neutral abstraction that makes the Noir harder to read. Gate-neutral cleanup is only worth keeping when it removes real duplication or dead parameters.
- Do not add public helper exports unless the public API test is intentionally being changed.

## Useful Spikes

Cast and primitive operation probes are available when deciding which shape to test next:

```sh
python3 scripts/benchmark_cast_costs.py --output /tmp/cast_costs.json
python3 scripts/benchmark_primitive_ops.py --output /tmp/primitive_ops.json
```

Treat tiny zero-cost primitive rows cautiously unless amplified; circuit padding can hide very small effects.

## Benchmark Methodology

`scripts/benchmark_float_ops.py` builds temporary binary packages that depend on this library, runs `nargo compile`, then reads `functions[0].circuit_size` from `bb gates -s ultra_honk`. It measures a small-N and big-N circuit and estimates the per-call cost from the difference so fixed setup/padding does not dominate.

Each operation is measured as repeated `acc = acc op b`, including division. Keep this aligned with the reference methodology unless there is a deliberate benchmark-method change and it is documented in `bench/README.md`.
