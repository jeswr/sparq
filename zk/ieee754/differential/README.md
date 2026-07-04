<!-- [SONNET-4.6] sq-3x7dl.14.1: internal-stub README for a publish=false standalone harness. -->

# sparq-ieee754-differential

PROOF M1 (sq-3x7dl.14.1): IEEE 754 differential harness for `sparq_ieee754` Noir primitives.

**Trusted oracle:**
- f32/f64: native Rust hardware floats (IEEE 754-2008 round-to-nearest-even).
- f16: `half::f16` (f32-widening arithmetic -- correctly rounded for normal values).
- f128: DEFERRED -- no stable Rust f128 or verified soft-float binding (sq-3x7dl.14.2).

**Sample coverage:** 32 random pairs + corner cases per arithmetic op; 20 per comparison.
Coverage is SAMPLE-BASED, not exhaustive.  This is VERIFICATION, not proof.
The trusted chain tested here: Noir source -> ACIR -> witness-gen (`nargo test`).
The ACIR->Barretenberg->proof lowering is NOT covered by this harness.

Run via `scripts/run_differential_harness.sh` (requires `cargo` + `nargo` on PATH).

**License:** MIT
