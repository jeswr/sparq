# Scripts

## Generated Arithmetic Vectors

Generate the external-consumer arithmetic fixture with:

```sh
python3 scripts/generate_float_vectors.py --random-per-op 8
```

Run it with:

```sh
bash scripts/test_generated_vectors.sh
```

The generator uses a self-contained exact rational IEEE reference in Python and
emits Noir tests under `tests/generated_arithmetic/src/lib.nr`. The fixture is
run from a temporary package so it only imports the public `f16`, `f32`, `f64`,
and `f128` types.

Optional public-suite ingestion is available for IBM FPgen `.fptest` files:

```sh
python3 scripts/generate_float_vectors.py \
  --include-fpgen \
  --fpgen-cache .ieee754_test_cache \
  --download-fpgen
```

That path is intentionally opt-in. FPgen has its own value grammar and
exception-flag conventions; the lightweight parser here only accepts simple
round-to-nearest-even binary operation rows. Treat failures from that mode as
hardening input rather than as the default committed fixture.

## Gate Benchmarks

Run amortised gate benchmarks with:

```sh
python3 scripts/benchmark_float_ops.py --ops add_f16 mul_f32 div_f64 --n-small 1 --n-big 8
```

Run generated integer-to-float conversion benchmarks with:

```sh
python3 scripts/benchmark_float_conversions.py --conversions u16_to_f16 i64_to_f64 --n-small 1 --n-big 8
```

Compare a candidate benchmark file against the committed baseline with:

```sh
python3 scripts/compare_float_benchmarks.py /tmp/candidate_float_ops.json --max-regression 1
```

For conversion benchmark candidates, pass the conversion baseline explicitly:

```sh
python3 scripts/compare_float_benchmarks.py \
  /tmp/candidate_float_conversions.json \
  --baseline bench/float_conversions_latest.json \
  --max-regression 1
```

The benchmark script creates temporary binary packages, runs `nargo compile`,
then measures `circuit_size` with `bb gates -s ultra_honk`. It records small-N
and big-N measurements so the fixed circuit setup/padding cost can be separated
from the per-call estimate.

## Lints

Check that private constrained Noir helper functions have at least two call
sites with:

```sh
python3 scripts/lint_private_function_usage.py
```

Single-use private constrained helpers should be inlined. The lint ignores
`unconstrained fn` helpers because those are used as off-circuit witnesses, and
inlining them into constrained callers would move hint computation into the
circuit.
