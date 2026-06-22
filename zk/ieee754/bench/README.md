# Benchmarks

Run amortised UltraHonk gate benchmarks with:

```sh
python3 scripts/benchmark_float_ops.py --ops add_f16 mul_f32 div_f64 --n-small 1 --n-big 8
```

Compare a candidate run against the committed baseline with:

```sh
python3 scripts/compare_float_benchmarks.py /tmp/candidate_float_ops.json --max-regression 1
```

The script builds temporary binary packages, runs `nargo compile`, then reads
the `circuit_size` reported by `bb gates -s ultra_honk`. It records a small-N
and big-N measurement and estimates per-call cost from the difference so the
fixed circuit padding/setup cost does not dominate the result. Each operation is
measured as repeated `acc = acc op b`, matching the reference library's
amortised benchmark harness.

Measured with `bb 5.0.0-nightly.20260324` and `nargo 1.0.0-beta.21`, the
native `FloatParts` / generated product-intermediate arithmetic path in `float_ops_latest.json` has
these per-call estimates with `--n-small 1 --n-big 8`:

| Size | Add | Sub | Mul | Div |
| --- | ---: | ---: | ---: | ---: |
| `f16` | `341.9` | `342.0` | `283.7` | `255.6` |
| `f32` | `446.0` | `446.0` | `355.7` | `367.9` |
| `f64` | `367.3` | `367.4` | `307.6` | `273.4` |
| `f128` | `630.1` | `630.1` | `543.9` | `524.6` |

The current f32/f64 add, mul, and div figures are below the May-2026 reference
`noir_IEEE754` amortised counts (`add`: `634.7`/`643.6`, `mul`:
`533.7`/`541.0`, `div`: `577.3`/`586.3`).

The SPARQL-needed kernels (comparison predicates, round-to-integral family,
`sqrt`, and the XPath float-to-integer casts) are measured by the same
`benchmark_float_ops.py` harness using a conversion-style pattern: a private
array of input witnesses keeps every call live, and the boolean/bit results
are XOR-folded. Per-call estimates for these therefore include one `new()`
decode per call, unlike the arithmetic rows above where the accumulator stays
decoded. Measured with the same toolchain and `--n-small 1 --n-big 8`,
`float_ops_baseline-kernels.json` has these per-call estimates:

| Op | `f16` | `f32` | `f64` | `f128` |
| --- | ---: | ---: | ---: | ---: |
| `eq` | `37.4` | `36.1` | `46.4` | `55.3` |
| `ne` | `37.4` | `36.1` | `46.4` | `55.3` |
| `lt` | `72.1` | `71.1` | `85.1` | `115.4` |
| `le` | `81.1` | `80.1` | `94.1` | `124.4` |
| `gt` | `82.1` | `81.1` | `95.1` | `125.4` |
| `ge` | `73.1` | `72.1` | `86.1` | `116.4` |
| `floor` | `283.9` | `276.0` | `304.6` | `383.1` |
| `ceil` | `284.9` | `277.0` | `305.6` | `384.1` |
| `trunc` | `268.3` | `260.4` | `289.0` | `361.1` |
| `round_ties_even` | `291.1` | `283.3` | `311.9` | `393.7` |
| `sqrt` | `587.4` | `308.4` | `587.4` | `498.3` |
| `to_u64` | `132.4` | `135.0` | `146.0` | `168.4` |
| `to_i64` | `154.1` | `156.7` | `167.7` | `165.7` |

`float_ops_latest.json` carries the union of the arithmetic and kernel rows so
`compare_float_benchmarks.py` guards all of them by default; the add/sub/mul/div
rows were re-measured alongside the new kernels with zero delta against
`float_ops_baseline-nargo-beta21.json`.

Integer conversion gates are measured by `scripts/benchmark_float_conversions.py`.
The harness feeds private arrays of integer witnesses through the generated
`From` impls and XORs the resulting `bits()` values so each conversion remains
live in the compiled circuit. Measured with the same toolchain and `--n-small 1
--n-big 8`, `float_conversions_latest.json` has these per-call estimates:

| Input | `f16` | `f32` | `f64` | `f128` |
| --- | ---: | ---: | ---: | ---: |
| `u8` | `170.0` | `171.0` | `171.9` | `171.3` |
| `u16` | `157.7` | `170.9` | `171.6` | `177.9` |
| `u32` | `162.6` | `156.9` | `170.0` | `176.3` |
| `u64` | `171.0` | `156.7` | `161.9` | `169.6` |
| `u128` | `184.7` | `178.0` | `175.6` | `207.1` |
| `i8` | `169.1` | `170.1` | `170.6` | `169.9` |
| `i16` | `164.9` | `169.9` | `170.4` | `176.4` |
| `i32` | `169.9` | `155.9` | `168.7` | `174.7` |
| `i64` | `179.9` | `165.9` | `172.4` | `167.9` |

The conversion path uses an unconstrained highest-bit/lower-part witness
verified by a bounded pow2 split, which removes typed shift/bit-and scans from
the constrained circuit. This originally regressed `u8` inputs, but after the
direct exact-pack path below it benchmarks lower there too. Inputs that fit
entirely in the destination significand take a direct exact-pack path after the
highest-bit proof, skipping normalized round-pack because no rounding is
possible. Rounded conversions skip a redundant highest-index range check after
the pow2 split has already bounded the supported power-of-two input widths.
Rounded conversions normalize with one witnessed shift-to-target divmod,
verified by a Field equation against the highest-bit pow2 and a scaled
remainder bound. This replaces separate left-normalization and jammed
right-shift verifiers for the rounded integer paths.
Rounded integer conversions use a narrower integer-specific packer
that keeps round-to-nearest-even and finite overflow but skips impossible
underflow and subnormal cases; it also omits finite-overflow checks for
destination/input pairs where the source exponent cannot reach the destination
maximum. The u64 conversion packer rounds directly to the stored mantissa,
which saves a gate on rounded f16 rows and `u128_to_f32`; its overflow case
uses an explicit branch to produce zero mantissa instead of subtracting an
overflow-scaled hidden bit. The f128 conversion
path removes impossible right-shift branches for inputs up to `u64` and uses an
exact shift proof after the highest-bit verifier has established the required
shift. Signed conversions
derive the sign from the top bit of the
unsigned representation before computing magnitude; this benchmarks lower than
signed `< 0` comparisons for the f128 and i8 conversion rows. Generated
`bits()` uses a boolean select for the sign mask, which is neutral for
arithmetic benchmarks and saves gates on negative f128 conversion rows. It
also packs the f128 exponent with a typed shift instead of multiplication by
the exponent scale; this is neutral for f128 arithmetic and lowers f128
conversion rows, while the smaller formats keep multiplication because typed
shifts regress their operation benchmarks.

The focused `mul_f128` estimate improved from `106003` gates/call in
`baseline_mul_f128.json` to `1012` gates/call in `post_limb_mul_f128.json`
using `--n-small 1 --n-big 2`.

When refactoring toward more generic internals, keep the `u64` and wide kernels
specialized unless a candidate benchmark proves the change is neutral. The
generic pieces currently accepted in the baseline are the conversion boundary
helpers and generated product-intermediate multiplication.

The `u64` work kernel uses a 6-bit pow2 verifier for dynamic shifts; using the
general 7-bit verifier there costs roughly 25-67 gates/call on f16/f32/f64 ops.
Finite-add exponent alignment additionally uses a bounded verifier for the
known live significand width, saving roughly 1-12 gates/call on add/sub.
Round-pack uses a fixed shift-by-one sticky helper for overflow normalization;
calling the dynamic shift verifier there costs roughly 10-65 gates/call.
The u64 round-pack denormal-underflow shift also uses the bounded live-width
verifier, which removes the old unbounded u64 shift wrapper entirely.
The f128 wide path uses the same bounded live-width idea at 116 bits, replacing
the former full-128-bit dynamic shift verifier for alignment and underflow.
Round extraction and guard-bit setup use small constant arithmetic (`/`, `%`,
`*`) instead of typed shifts and masks where that benchmarks lower.
u64 left-normalization pow2 checks use mantissa-specific bounds for hidden-bit
and round-threshold shifts instead of proving every shift as a 64-bit value.
The f128 left-normalization pow2 checks are similarly bounded to 113/116 bits
for hidden-bit and round-threshold shifts.
The u64 kernels carry offset exponents as `u16`; the f16/f32/f64 exponent
work range fits there and avoids proving wider arithmetic in add/mul/div.
The f128 wide kernel also keeps offset exponent arithmetic in `u16`, while
leaving significand arithmetic on the wide `u128`/Field path.
The generated f128 operations now call the wide kernel directly on `FloatParts<u16, u128>`,
avoiding a conversion wrapper around the exponent field. Their wide add/sub,
mul, div, classification, normalization, and packing helpers are specialized
to the binary128 constants, while shared typed constructors remain generic.
Generated multiplication no longer carries a product-intermediate generic:
f16/f32/f64 use the u64 normalized product witness, and f128 calls the
specialized wide product split directly.
Jammed right-shift verifiers are used only on positive shifts, so their
zero-shift proof branch is removed from the hot helper.
The shared jam-low-bit helper uses arithmetic parity (`% 2`) instead of typed
`u128 & 1`, saving f128 arithmetic gates and rounded conversion gates.
The u64 shift-by-one sticky helper similarly uses `/ 4` and `% 4` for every
small format; this beats typed `>> 1`/`& 1` for f16/f64 add/sub and is neutral
for f32.
For f32-sized dynamic u64 jammed shifts and division quotients, sticky low-bit
setting uses the arithmetic parity helper; the same rewrite regresses f16/f64,
so those paths keep typed `|`.
Left-normalization verifiers are likewise only called with nonzero
significands, so their zero-value proof branch is removed.
Round overflow normalization no longer checks whether the current exponent is
finite first; out-of-range exponents still pack to infinity. The f16/f32 and
f128 initial round-pack overflow threshold checks use an unconstrained side
hint plus a bounded low-part proof; f64 keeps the typed comparison because the
witnessed form was neutral-to-slightly worse there. The f32 and f128
left-normalization threshold checks use the same proof shape with one narrower
low-part bound; f16 keeps the typed comparison because the witnessed form
regressed badly there. The f32 and f128 finite-normalization hidden-bit checks
also use this shape before mul/div left-normalization; f16/f64 keep typed
comparisons because they regressed with the witnessed form. The f32 and f128
post-rounding hidden-bit checks use the same proof shape to decide whether to
pack a subnormal result.
Final round overflow checks compare to `2 * hidden` exactly; the earlier
normalization step prevents larger rounded significands.
Small-kernel round-nearest-even uses an explicit overflow branch to reset the
rounded significand to `hidden`; this benchmarks slightly lower than arithmetic
subtraction by `overflow * hidden` in f16/f32/f64 ops.
Opposite-sign finite add/sub selects zero with equality first, then uses one
greater-than comparison to choose the nonzero difference.
The u64 dynamic shift verifiers use 4-bit/5-bit/6-bit pow2 decompositions for
f16/f32/f64 live-width bounds while keeping pow2 witnesses in `Field`; casting
pow2 down to an integer benchmarks much more expensive than a Field range
assertion.
Pow2 bit recomposition, product-chain verification, and unconstrained hinting
share numeric-generic helpers so the verifier remains concise without changing
the generated gates.
Small-kernel mul/div exponent sums widen only to `u32` before casting back to
`u16`; widening to `u64` costs a few extra gates/call.
The u64 division quotient uses a u64-native Euclidean witness with a constant
pow2 shift, Field reconstruction, and a typed `remainder < denominator` check,
avoiding the generic u128 quotient verifier on f16/f32/f64.
The f128 division quotient follows the same witness-and-verify shape: the
binary long division runs only in Brillig, while the circuit verifies the
`numerator * 2^115 = quotient * denominator + remainder` split and the
strict remainder bound.
The u64 multiplication product path witnesses the product overflow bit, scales
by `8` or `4` accordingly, and verifies one Field Euclidean split by
`2 * hidden`, returning the exponent bump with the jammed significand. Because
that divisor is a power of two, the split proves the remainder with a
mantissa-specific bit-size bound instead of a general typed comparison.
u64 mul/div use a normalized-only round-pack path that skips impossible
overflow and left-normalization checks while preserving denormal underflow,
round-to-nearest-even, finite overflow, and packing behavior.
The f128 mul/div paths use the same normalized-only packer after their
operation-specific normalization: mul performs the one possible product
overflow shift before packing, and div normalizes the numerator before the
witnessed split. The f128 scaled-product split also uses a power-of-two
remainder bit-size proof rather than the generic complement proof, and its
helper is specialized to the wide f128 path.
The f128 div pre-quotient normalization preserves its branch shape but doubles
the numerator with `* 2` instead of a typed `<< 1`, saving gates without the
regression seen from the broader branchless rewrite. Its normalized-significand
ordering check uses a hinted boolean plus a bounded difference proof; the same
shape regressed the u64 division paths and is kept wide-only.
Finite add/sub lexicographically swaps operands by magnitude, aligns only the
smaller operand, and computes `large +/- small` with one arithmetic expression
before round-pack. This avoids the old double-alignment and post-alignment
branch chain, bringing f32/f64 add below the reference library's May-2026 gate
counts. Addition special-case handling stays minimal before finite add/sub,
cutting f128 add/sub to `630.1` gates/call. The f128 significand tie case uses a
hinted ordering plus bounded difference proof; the same shape regressed f32 and
stays wide-only.

Cast and primitive-operation probes are available for optimization spikes:

```sh
python3 scripts/benchmark_cast_costs.py --output /tmp/cast_costs.json
python3 scripts/benchmark_primitive_ops.py --output /tmp/primitive_ops.json
```
