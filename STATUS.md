# STATUS — zk-ieee754-kernels

## State: COMPLETE (2026-06-13)

All four SPARQL kernel features (comparisons, round-to-integral family,
float-to-int casts, sqrt) are implemented, oracle-tested, and gate-benchmarked.
Nothing in flight. Working tree clean.

## What was done in the resumed session
1. Dirty state resolved: predecessor's benchmark_float_ops.py extension
   (XOR-fold harness for predicates/unary/casts) KEPT; its untracked
   bench/float_ops_baseline-kernels.json VALIDATED by a full fresh re-run of
   all 52 rows — 0 diffs (deterministic) — then committed (e841e36).
2. Old-op regression: add/sub/mul/div x f16..f128 re-measured; max delta vs
   bench/float_ops_baseline-nargo-beta21.json = 0.0 gates/call (compare script
   PASS at --max-regression 1).
3. bench/float_ops_latest.json extended to all 68 rows; bench/README.md and
   AGENTS.md document the new-kernel table + methodology (fa89c23).
4. Tests: main package 23/23; generated oracle vectors 121/121; public API
   6/6; private-usage lint clean.
5. Oracle reproducibility: generate_float_vectors.py with defaults (seed 754)
   regenerates tests/generated_arithmetic/src/lib.nr byte-identically.
   Independent numpy cross-check (f16/f32/f64, 19,890 evaluations over
   cmp/round/sqrt/cast): 0 real mismatches; 100 NaN-payload-only differences,
   all explained by the library's canonical-NaN policy.

## Toolchain
nargo 1.0.0-beta.21, bb 5.0.0-nightly.20260324 (matches recorded baselines).

## Next steps for a successor (none required)
Possible follow-ups: f32 sqrt is anomalously cheap (308.4) vs f16/f64 (587.4)
— worth understanding before optimizing sqrt; consider rem/abs/neg kernels if
SPARQL needs them. Do NOT push this branch.
