---
name: noir-optimisation
description: >-
  Cost model and optimisation reference for Noir circuits on the
  UltraHonk / Aztec Barretenberg stack. Use when sizing constraint
  budgets, deciding whether to push work into an `unconstrained`
  Brillig hint with an in-circuit verifier, comparing Field-arith vs
  typed `u64` bit operations, reading `bb gates` / `nargo info` /
  `noir-profiler gates` output, or weighing ACIR-opcode against
  backend-gate regressions. Companion to `noir-circuit-patterns` —
  that skill covers SPARQL-primitive shapes; this one covers which
  shape wins after measurement. **Always run `bb gates` before
  claiming a saving.** `nargo info` is misleading on its own — see
  §1. Intuition has misfired on this codebase before (PR #37 / spike
  ledger `bench/SPIKES.md`).
---

# Noir optimisation — cost model + decision rules

**This is a low-level library; always optimise for minimal gate
count.** Every call site pays the constraint cost on every proof.
Per-site savings of 5-15 gates compound across thousands of call
sites in downstream callers' circuits. Readability is secondary to
constraint count: when the two trade off, take the gates and add a
comment. The only exception is when two forms compile to the same
constraints (the optimiser CSE's identical sub-expressions); in that
case pick whichever reads better.

A reference for someone already writing Noir circuits in this
workspace. Optimisation in Noir is empirical: measure with **`bb gates`**
*before* declaring a win. The 2026-05-03 round-1 of `unconstrained_ops`
for IEEE 754 had us predict an ACIR-opcode + Width win for
`count_leading_zeros_u64_verified`; PR #37 measured **both regressing**
in both isolated and composed regimes, and the follow-up
bit-decomposition spike (`c0f433d`, reverted in `63d0b95`) regressed
twice as hard again. The 2026-05-10 `shr_sticky_u64` Field-arith spike
*looked* like a 17% win on `bb gates` until we discovered the relation
was unsound — see §3.4. Don't trust the textbook shape; measure, and
measure the right number.

For *which* primitive to write — BGP matchers, joins, filters — see
`noir-circuit-patterns`. This skill picks up where that one stops:
given a candidate gadget, does the proposed implementation beat the
in-circuit-spec baseline on the metric that actually drives prover
wall-clock?

For generic Noir guidance (language syntax, the stdlib, project
setup, testing patterns, JS integration) consult the upstream skills:
`noir-developer` (circuit-dev, stdlib, workspace), `noir-idioms`
(hint-and-verify principle, ACIR-vs-Brillig basics, `bool` vs Field,
`if/else` selects, `assert_eq` style), `noir-testing` (test
attributes and organisation), `noir-js` (noir_js / bb.js
integration). This skill is intentionally narrow: the empirical,
measured findings on `bb gates` cost that the upstream skills don't
cover.

## 1. The metric that matters: `bb gates`, not `nargo info`

The single most common mistake on this codebase is reasoning about
proving cost from `nargo info` alone. **`nargo info` is not proving
cost.** The canonical proving-cost metric on the Noir + UltraHonk
stack is **the `circuit_size` field returned by `bb gates`** [1, 2].

### 1.1 Three numbers, three meanings

`nargo info` reports per-function:

- **ACIR opcodes** — count of `AssertZero`, `BlackBoxFuncCall`,
  `MemoryOp`, `MemoryInit`, `BrilligCall`, `Call` instructions in the
  constrained portion of the circuit [3]. Each opcode's cost in
  backend gates varies by orders of magnitude — `assert(x == y)` and
  `assert(x < 2^64)` are both one ACIR opcode but the second lowers to
  hundreds of UltraHonk gates after black-box expansion.
- **Expression Width** — sum of witness-wire counts across all
  `AssertZero` opcodes in that function [4]. It's the per-row degree
  bound on PLONKish gates, not a gate count. Optimising for it can
  still mislead — see PR #37 in §3.3 and the spike ledger entry for
  `clz_u64`.
- **Brillig opcodes** — unconstrained-VM instructions; **free at
  proof time** [1, 5]. Don't optimise for this column. If you halve
  Brillig opcodes but double ACIR, you've made the prover slower.

`bb gates` reports the **post-expansion UltraHonk gate count**
(`circuit_size`), which is what the prover actually pays for. Two
empirical examples from this repo (`bench/unconstrained_gate_counts.bb_addition.json`,
commit `768448f`):

| Function                        | ACIR opcodes | Expression Width | `bb gates` |
|---------------------------------|-------------:|-----------------:|-----------:|
| `add_float32`                   |           81 |              622 |       6506 |
| `mul_float32`                   |           81 |             1062 |       7348 |
| `div_float64`                   |           83 |             4594 |      20015 |

Three functions with **the same 81-83 ACIR opcode count** span a 3×
range in actual proving cost. ACIR opcode count is a 5-50× underestimate
because each `BlackBoxFuncCall::RANGE` (every `assert_max_bit_size`,
every `u32`-typed input, every cast) expands to hundreds of UltraHonk
gates under Plookup [2].

### 1.2 The opposite trap: typed `>>`/`&` hide cost inside black boxes

Going the other way is just as misleading. A typed `u64`
`>>`/`&` baseline for `shift_right_sticky_u64` measured 34 ACIR
opcodes; a Field-arithmetic variant of the same primitive measured
158 ACIR opcodes (4.6× more). On `bb gates`, the Field-arith variant
came out **lower** — because every typed-`u64` shift/AND lowers to
`remove_bit_shifts` decomposition + range checks, and those black-box
expansions dominate the backend cost. Numbers from
`bench/unconstrained_gate_counts.bb_addition.json`,
`shr_sticky_u64_amplified_*` block:

| Variant                                       | ACIR | `bb gates` |
|-----------------------------------------------|-----:|-----------:|
| `shr_sticky_u64_amplified_baseline` (typed)   |   34 |      16899 |
| `shr_sticky_u64_amplified_field_arith`        |  158 |      12308 |
| `shr_sticky_u64_amplified_verified`           |  137 |      13970 |

The typed baseline has the **fewest** ACIR opcodes and the **highest**
`bb gates`. Reading only `nargo info` you would pick the worst variant.

### 1.3 Rule

- **`bb gates --include_gates_per_opcode` is the ground truth.** Use
  it for every "does this optimisation work?" question.
- **`nargo info` is iteration sugar.** It's faster to run than `bb gates`,
  so during a tight edit loop watching the Expression Width column is
  fine — but never claim a saving without `bb gates` to back it up.
- If `bb gates` and `nargo info` disagree on which variant wins, trust
  `bb gates` — and update the spike ledger entry so the next agent
  doesn't repeat the analysis.

## 2. The `bb gates` workflow

### 2.1 Toolchain — `bbup`

`bb` and `nargo` are version-coupled. `bb 0.84.0` errors with
*"Length is too large"* against bytecode produced by
`nargo 1.0.0-beta.17`. Run `bbup` with no arguments — it queries the
installed `nargo`, resolves the compatible `bb`, and downloads it [6].
For Noir 1.0.0-beta.17 this resolves to bb 3.0.0-nightly.20251104
(check with `bb --version`).

```sh
bbup                       # auto-detect; pulls compatible bb
bb --version               # confirm
nargo --version            # confirm
```

If you see `Length is too large`, your `bb` is older than your `nargo`.
Run `bbup` again.

### 2.2 `bb gates` invocation

The canonical invocation, after `nargo compile`:

```sh
cd <package_root>          # e.g. ieee754/
nargo compile
bb gates \
    -s ultra_honk \
    -b target/<package_name>.json \
    --include_gates_per_opcode
```

Output is JSON. The headline number is `functions[0].circuit_size`
(the UltraHonk gate count). The breakdown `functions[0].gates_per_opcode`
is a `[u32; N]` array indexed in the same order as the ACIR opcodes
emitted by the compiler — for `add_float32` on commit `768448f` the
top three entries (3093 + 1371 + 697) consumed 79% of the function's
total gates, and they are the range-check black boxes from
`decompose_hint` + casts. **This per-opcode view is where you find the
hotspot.**

### 2.3 `noir-profiler gates` for source-attributed flamegraphs

`noir-profiler gates` wraps `bb gates` with source-location attribution
and emits an SVG flamegraph [2]:

```sh
noir-profiler gates \
    --artifact-path target/<package_name>.json \
    --backend-path bb \
    --output target/ \
    -- --include_gates_per_opcode
```

The flamegraph attributes each backend gate to the Noir source line it
originated from. This is the right tool when `bb gates` per-opcode says
"opcode #17 costs 3093 gates" and you need to know **which line of the
source generated opcode #17**. Use it before optimising a function
larger than a screenful.

### 2.4 The padding-floor warning — measure per-call cost by amplification

UltraHonk pads small circuits to a fixed gate-count floor. On the
current `bb` (3.0.0-nightly.20251104), an empty
`fn main(x: pub u64) -> pub u64 { x }` reports `circuit_size = 2842`,
and every isolated primitive below ~1200 gates of real work rounds up
to `circuit_size = 4120` (the next power-of-two plus glue rows). This
means **a microbenchmark of a single primitive call cannot measure
per-call cost** — every variant reports 4120 and the differences are
invisible.

Evidence from `bench/unconstrained_gate_counts.bb_addition.json`:

| Variant                                    | `bb gates` |
|--------------------------------------------|-----------:|
| `shr_sticky_u64_isolated_baseline`         |       4120 |
| `shr_sticky_u64_isolated_verified`         |       4120 |
| `shr_sticky_u64_isolated_field_arith`      |       4120 |
| `shr_sticky_u64_composed_baseline`         |       4120 |
| `shr_sticky_u64_const20_baseline`          |       4120 |
| `shr_sticky_u128_isolated_baseline`        |       4120 |

All six different primitives hit the same floor. The numbers are
**uninformative** for per-call cost comparison.

The fix: **amplify**. Instantiate the primitive N times inside `main`
with witness-dependent inputs that defeat constant folding, then
back out the per-call cost as `(bb_gates − empty_floor) / N`. The
in-repo harness (`scripts/benchmark_gates.py`, `*_amplified_*` block,
N = 64) implements this pattern [7]:

```rust
fn main(value: pub u64, seed: pub u64) -> pub u64 {
    let mut acc: u64 = 0;
    for i in 0..64 {
        let shift = (seed + i as u64) & 63;
        acc = acc ^ shift_right_sticky_u64_verified(value + i as u64, shift);
    }
    acc
}
```

Per-call cost for `shift_right_sticky_u64_verified`:
`(13970 − 2842) / 64 ≈ 174 gates/call`.

Two further pitfalls when amplifying:

- **The accumulator must be witness-dependent.** A `for` loop that
  XORs in a constant gets unrolled and constant-folded into a single
  constant; the prover never sees the primitive. The harness uses
  `value + i as u64` and `(seed + i) & 63` so each call has distinct,
  witness-dependent arguments.
- **N must be large enough to dominate the floor.** With N = 64 the
  floor is ~25% of the smallest amplified measurement (4120 / 16899
  for the typed baseline) — within tolerance. N < 16 leaves the floor
  in the same order as the signal and per-call comparison breaks down.

## 3. The `unconstrained + verified` pattern — when it pays

For the *principle* (hint a result, verify it cheaply), see the
`noir-idioms` skill, "Core Principle: Hint and Verify", and
`noir-developer/circuit-dev/unconstrained-functions.md` for the
syntax. This section covers what `noir-idioms` does NOT: the
**measured** conditions under which the pattern actually beats the
in-circuit spec on UltraHonk `bb gates`, the soundness pitfalls
encountered on this codebase, and a heuristic for when to spike a new
`_verified` variant vs skip the experiment.

### 3.1 The actual condition for a win

`bb_gates(verifier) < bb_gates(in-circuit-spec)`. The verifier must
avoid the cost sinks that defeat constant-folding:

- **(a) data-dependent shifts.** `value >> count` where `count` is a
  witness lowers via `remove_bit_shifts` [8] to a runtime decomposition
  that scales linearly with the maximum shift amount. A shift by a
  *constant* is folded; a shift by a witness is not.
- **(b) data-dependent indexing.** `arr[i]` where `i` is a witness
  lowers to `MemoryOp` plus a range check on `i`; a constant index is
  folded out entirely.
- **(c) non-foldable surrounding context.** A loop with a constant
  bound and no witness reads is unrolled by the `unrolling` SSA pass
  and then constant-folded; the same loop with a witness-dependent
  bound is *not* unrolled.

### 3.2 Patterns where the unconstrained pattern wins reliably

- **Integer division.** Witness `q, r`; verify `q * d + r == n`,
  `r < d`. Verifier is one `AssertZero` plus one range check.
- **Square root.** Witness `s`; verify `s * s <= n < (s + 1) * (s + 1)`.
  This is the form Noir docs give as the headline example [5].
- **Merkle inclusion.** Witness the path; recompute the root from the
  leaf + path with a constant-arity hash gadget, equality-assert
  against the public root.
- **Byte / bit decomposition with constant stride.** As in the Noir
  doc's `u64_to_u8` example: the recombination is
  `sum_i x_i * 2^(stride*i)` — every shift amount is a compile-time
  constant [5].

### 3.3 Patterns where it loses — the PR #37 case study

`count_leading_zeros_u64_verified` was the predicted win that reversed.
The verifier shape was: witness `count`; assert `count <= 64`; if
`count == 64` then `value == 0`; else assert bit `(63 - count)` of
`value` is set and `value >> (count + 1) == 0`. The **dynamic
right-shift `value >> (count + 1)`** is the cost driver — the shift
amount is a witness, so cost (a) above bites.

The bit-decomposition spike that followed (witness `bits[64]` + one-hot
`is_leading` + cumulative-prefix flag) regressed even harder — it has
no data-dependent shift, but it pays N booleanity asserts plus a wide
recombination per bit. See `bench/SPIKES.md` entry
*"2026-05-03 — `clz_u64` verifier via explicit bit decomposition"*
for the full numbers and the verdict.

The merged primitive (`count_leading_zeros_u23_verified`, kept; the
u64 variant was deleted in PR #40) lives in the public API for two
reasons: (i) Lampe extraction needs a stable target name; (ii) future
helpers may compose with it under constant inputs where the verifier
*is* foldable. **No call site has been swapped.**

### 3.4 The Field-arithmetic soundness pitfall

The 2026-05-10 `shr_sticky_u64` Field-arithmetic spike is the most
recent example of how easily this trade-off goes wrong on the
soundness axis. The original Field-arith probe — measured as
`shr_sticky_u64_amplified_field_arith` in
`bench/unconstrained_gate_counts.bb_addition.json` — looked **17%
cheaper** than the typed baseline (12308 vs 16899 `bb gates`) and
was tempting to ship. It was unsound.

The unsound relation:

```rust
// Witness pow, pow_complement, quotient, remainder from a Brillig hint.
// Range-check them so the field elements behave like u64s.
pow.assert_max_bit_size::<65>();
pow_complement.assert_max_bit_size::<65>();
assert(pow * pow_complement == 0x10000000000000000);  // 2^64
assert(quotient * pow + remainder == value);
(remainder * pow_complement).assert_max_bit_size::<64>();
quotient.assert_max_bit_size::<64>();
// ... sticky-direction clauses elided ...
```

What this proves: *some* power-of-two factor `pow` satisfies the
Euclidean-division relation against `value`, with `pow * pow_complement
== 2^64`. What this does **not** prove: that `pow == 2^shift` for the
*specific* `shift` the caller asked for. An adversarial prover faced
with `(value=0b1010, shift=3)` can supply `pow=4, pow_complement=2^62`
and the verifier accepts — the result corresponds to `value >> 2`, not
`value >> 3`. **Whenever you witness `pow2 = 2^k` via an `unconstrained`
hint and don't bind it back to `k`, you've broken soundness.**

The sound fix — implemented in the production
`shift_right_sticky_u64_verified` at `ieee754/src/unconstrained_ops.nr`
lines 397-613 — adds a bit-decomposition of `shift` plus a product
chain that recomposes `pow` from those bits:

```rust
// In the Brillig hint, additionally produce the six low bits of shift.
unconstrained fn shr_sticky_u64_field_hints(shift: u64)
    -> (Field, Field, [Field; 6])
{
    // ... pow2, pow2_complement computed as before ...
    let mut bits: [Field; 6] = [0; 6];
    let mut s = shift;
    for i in 0..6 {
        bits[i] = (s & 1) as Field;
        s = s >> 1;
    }
    (pow2 as Field, pow2_complement as Field, bits)
}

// In the verifier:
//
// (i) bits[i] are boolean and recompose to shift.
for i in 0..6 {
    assert(bits[i] * (1 - bits[i]) == 0);
}
let recomposed: Field =
      bits[0]
    + bits[1] * 2
    + bits[2] * 4
    + bits[3] * 8
    + bits[4] * 16
    + bits[5] * 32;
assert(recomposed == shift as Field);

// (ii) pow == prod_i (1 + bits[i] * (2^(2^i) - 1)).
// Each factor is 1 when bit i is 0, and 2^(2^i) when bit i is 1.
let expected_pow2: Field =
      (1 + bits[0] * 1)
    * (1 + bits[1] * 3)
    * (1 + bits[2] * 15)
    * (1 + bits[3] * 255)
    * (1 + bits[4] * 65535)
    * (1 + bits[5] * 0xFFFFFFFF);
assert(pow2 == expected_pow2);

// (iii) onwards, the Euclidean-division clauses as before.
```

Cost of the binding for a 6-bit shift: 6 boolean checks + 5
multiplications in the product chain + 1 linear recomposition check.
After binding, the same `shr_sticky_u64_amplified_field_arith_sound`
benchmark drops to ~5% cheaper than the typed baseline. A 5% per-call
win is still a win on a low-level library: migrate the call sites
and bank the gates. The point of this section is not that 5% is
small — it's that the *unsound* 17% number was the lie, and the
sound 5% is what's actually available.

Always check: when a hint produces a value `y` that the verifier
treats as `f(x)` for some witness `x`, can the prover swap to a
different `y' = f(x') ≠ f(x)` that still passes every clause? If yes,
the verifier is unsound. Bit-decomposition of the input plus a
recomposition assertion is the standard fix; pay the cost.

### 3.5 Heuristic — when to spike vs when to skip

Spike a `_verified` variant only when **all four** of these hold:

1. The in-circuit spec contains an obviously expensive sub-computation
   (a CLZ via 6 conditional shifts, a hash with witness-driven input
   length, a division, ...).
2. The proposed verifier relation can be expressed *without* a
   data-dependent shift, a data-dependent index, or a >100-element
   loop over witnesses.
3. **The verifier binds every witnessed value back to the public
   input** — no `pow2 = 2^shift` without a recomposition check on
   `shift`; no `quotient = value / d` without a Euclidean-division
   check; no witnessed bit-position without a uniqueness clause. The
   soundness check is non-negotiable; the cost it imposes is part of
   the candidate's cost.
4. There is (or will be) a Lean obligation in `proofs/<crate>/` that
   pins the relation's uniqueness — otherwise the unconstrained hint
   is unverified and roborev will find it (PR #36's `pub unconstrained`
   hint was crate-privatised in `c7ecb8a` for exactly this reason).

If any of these fails, write the in-circuit spec directly and don't
spend the round.

## 4. When Field-arith helps vs when typed `u64` is fine

Empirical decision rules from this codebase's measurements (the
`shr_sticky_*` series in `bench/unconstrained_gate_counts.bb_addition.json`
and the `clz_*` series in `bench/SPIKES.md`):

### 4.1 Typed `u64` is already cheap

- **Bounded comparisons** (`<=`, `>=`, `<`, `==`). Plookup absorbs the
  range checks. Don't migrate `assert(x <= 64)` to Field-arith --
  measured constraint-equivalent and the typed form has no gate
  penalty. (This is the rare two-forms-compile-to-the-same-thing
  case; pick either.)
- **Constant-shift bitops** (`x >> 3`, `x & 0xFF`, `x << 8` with the
  shift a literal). Constant-folded by the SSA passes; ends up as
  cheap recombination. The `shr_sticky_u64_const20_*` benchmarks
  confirm this — the typed baseline at `shift = 20` is 15 Width / 17
  ACIR, compared to 72 / 34 in the dynamic-shift case.
- **Small dynamic shifts** (`shift < ~16`). Per-shift `remove_bit_shifts`
  cost is O(M) for max-shift M; at M = 16 this is cheap enough that
  Field-arith doesn't recover the overhead. `count_leading_zeros_u23`
  is in this regime and kept the dynamic-shift verifier.

### 4.2 Field-arith helps

- **Witness-dependent shifts of wide registers** (`value >> shift`
  where `value: u64` and `shift: u64` is a witness). The
  `shr_sticky_u64` case: Field-arith via Euclidean-division saves
  ~5% bb gates *with* the soundness binding, ~17% without (but you
  must include the binding). For `shr_sticky_u128` the gap is
  larger (see the same bench entry).
- **Large multiplications used in a recomposition.** When the verifier
  shape is `value == sum_i coeff_i * x_i` with coefficients known at
  compile time, Field is the natural arena; the typed form would
  recompute each shift.
- **Probability witnesses, inverses, division.** Anything that uses
  `field_inv_or_zero(x)` as a witness can only be expressed in Field.
  Don't try to write `x * x_inv == indicator` over `u64`.

### 4.3 Field-arith does **not** help

- **Small comparisons.** A `Field` cast back to `u64` to range-check
  costs a `BlackBoxFunc::RANGE` you don't pay in the typed form.
- **Bit-extraction with constant stride.** `(value as Field).to_le_bits()`
  is cheaper than hand-rolling, but if the stride and the count are
  both constants the typed `>>` / `&` form constant-folds away.
- **Compositions where the surrounding code is typed.** Each `as Field`
  / `as u64` cast inserts a range check (§5). If 90% of the function
  is typed-`u64`, casting in and out of `Field` for one operation
  can cost more than the operation saved.

### 4.4 Profile before migrating

The end-to-end `bb gates` numbers for this codebase's primitives
(commit `768448f`, `bench/unconstrained_gate_counts.bb_addition.json`):

| Function       | `bb gates` |
|----------------|-----------:|
| `add_float32`  |       6506 |
| `sub_float32`  |       6507 |
| `mul_float32`  |       7348 |
| `div_float32`  |       6977 |
| `add_float64`  |       5602 |
| `sub_float64`  |       5603 |
| `mul_float64`  |       8943 |
| `div_float64`  |      20015 |

`div_float64` is 3× the next-heaviest op. Before reaching for
Field-arith *anywhere*, run `bb gates --include_gates_per_opcode` on
`div_float64` and find the hotspot — global metric improvements are
much more likely to come from one expensive opcode than from a sweep
across the rest of the file.

## 5. Black-box / Plookup primitives

`BlackBoxFuncCall` opcodes lower to backend gadgets and can be either
the cheapest or the most expensive opcode in a circuit. Available
black-boxes in `nargo 1.0.0-beta.17` (stable through
`1.0.0-beta.20`) [9]:

- Logical: `AND`, `XOR`, `RANGE` — invoked via `&`, `^`,
  `x.assert_max_bit_size::<N>()` or any typed-`uN` boundary check.
- Hashes: `keccakf1600`, `sha256` (compress), `blake2s`, `blake3`,
  `pedersen_hash`, `pedersen_commitment`, `poseidon2_permutation`.
- Crypto: `ecdsa_secp256k1`, `ecdsa_secp256r1`, `embedded_curve_add`,
  `multi_scalar_mul`, `aes128_encrypt`.
- Recursion: `recursive_aggregation`.

When to reach for a black box:

- **Hashes always go through black boxes** if the hash is one of the
  supported set. Implementing Poseidon in arithmetic constraints is a
  ~100× regression for nothing.
- **Range checks dominate Plookup setup cost for small circuits** [2].
  Don't over-assert — if a `u32` is already typed `u32`, it is
  range-checked once at the function boundary, not on every use. The
  `decompose_hint` range-check expansion is opcodes #1 / #2 / #3 in
  `add_float32` and accounts for 79% of its 6506 gates.
- **Lookups are not generally available** — at time of writing, Noir
  exposes Plookup-style lookup arguments only via the `RANGE`
  black-box for arithmetic and the keyed gadgets above. There is no
  general `lookup_table(...)` user-defined gadget. If you find
  yourself wanting one, document the gap and use a polynomial
  commitment over the table instead (see `noir-circuit-patterns`
  §"Joins" — *lookup-table / set-membership* pattern).

`nargo info` does **not** distinguish black-box-call ACIR opcodes
from `AssertZero` opcodes in its summary count; the breakdown is
visible only through `bb gates --include_gates_per_opcode` or
`noir-profiler gates --include_gates_per_opcode` [2].

### 5.1 `assert_max_bit_size` is the canonical range check

Per the Noir Field docs [10]:

```rust
let x: Field = some_witness;
x.assert_max_bit_size::<32>();         // constrain x < 2^32
```

The signature is `pub fn assert_max_bit_size<let BIT_SIZE: u32>(self)`
on `Field`. Use this — rather than `assert(x < 2^32)` over `Field` —
when you've taken a `Field`-typed witness from a Brillig hint and
need to bound its size. The form `(remainder * pow_complement).assert_max_bit_size::<64>()`
in `verify_shr_sticky_u64_relation` is the canonical bounded-product
range check.

The Noir explainer also notes that `assert_max_bit_size` can replace
an explicit range check when array out-of-bound checks have already
been performed [11]: *"Converting a `Field` to `u32` can be costly
due to modulo operations. This cost can be mitigated by using
`assert_max_bit_size::<32>()` and `as u32`, especially when array
out-of-bound checks are already performed, as these checks can make
the explicit range check redundant"*.

## 6. Constant-folding rules

What the SSA optimisation passes do, in roughly the order they
run [12] (`compiler/noirc_evaluator/src/ssa/opt/`):

- `inlining` — function calls inlined; enables downstream folding.
- `unrolling` — loops with compile-time-known bounds and no `break`
  on a witness condition are unrolled.
- `mem2reg` — promotes memory references to SSA values; enables
  more folding.
- `constant_folding/` — folds expressions whose inputs are all
  constants after SSA propagation.
- `simple_optimization` / `simplify_cfg` / `flatten_cfg` — the
  `flatten_cfg` pass is **non-optional**, because ACIR has no
  control-flow operators; it removes all branches, replacing them
  with conditional selections [12]. After flattening, both arms of
  an `if` contribute constraints.
- `remove_bit_shifts` — lowers `>>` and `<<` by a witness amount to
  a bit-decomposition loop [8].
- `remove_unused_instructions` / `die/` — dead-instruction
  elimination, scheduled after `flatten_cfg`.

Practical rules:

- **Static is foldable.** A constant in a shift amount, an array
  index, or a loop bound is folded away.
- **Witnesses are not foldable.** Any operation whose result depends
  on a `pub`/private input survives into ACIR.
- **Both branches cost.** `if` over a witness condition is flattened
  to conditional select (`x = c ? a : b`), so both `a` and `b` must
  be computed and constrained. Don't write
  `if cheap_check { expensive } else { 0 }` to "skip" a cost.
- **Loop-unrolling thresholds.** Loops with witness-dependent bounds
  are not unrolled; the body must not depend on the iteration index
  for the unroller to fold the iteration count out.
- **Microbenchmarks with literal-constant inputs lie.** A
  microbenchmark that passes a literal constant into a CLZ helper
  measures the constant-folded happy path; it does not measure what
  the prover pays at a real call site. **Use `pub` witnesses for the
  inputs of every benchmark.** The `PRIMITIVE_BENCHMARKS` block in
  `scripts/benchmark_gates.py` threads `pub u64` witnesses through
  to defeat folding [7].

## 7. Common pitfalls

### Mixed bit-widths

Noir is strict about integer type matching for bitwise ops:
`(value as u64) & (mask as u8)` is rejected. The fix is **explicit
`u<N>` annotations on every literal and intermediate**:

```rust
let masked: u32 = value & 0x007F_FFFF_u32;
let probe: u32  = (low23 >> top_bit_pos) & 1_u32;
```

The `unconstrained_ops.nr` module's preamble documents this
discipline as a crate-wide convention [13].

### Unconstrained outputs leaking

A `pub unconstrained fn` exposes the unverified hint through the
public API, letting downstream callers bypass the verifier.
Roborev's PR #36 review caught this exact bug
(`c7ecb8a` made `count_leading_zeros_u23_unconstrained`
crate-private). **Rule: every `unconstrained` hint is private to the
module that contains its verifier.** The only sanctioned public
symbol is the `_verified` wrapper.

### Range-check density

Every `assert_max_bit_size::<N>()` (and every cast through a typed
`uN` boundary) lowers to `BlackBoxFunc::RANGE` (§5). If the upstream
type is already `uN`, the value is already range-checked at the
function boundary; re-asserting is a free regression.

### Shared-test-helper anti-pattern

A verifier copy-pasted into adversarial tests drifts. The convention
in this workspace is that the production `_verified` function and the
adversarial tests both call the same crate-private
`verify_*_relation(...)` helper (see `verify_shr_sticky_u64_relation`
in `ieee754/src/unconstrained_ops.nr` [13]). A regression in the
production verifier is then also caught by the adversarial-tests
path; with a copy, it can hide for a release.

### Both arms of `if` cost

After `flatten_cfg`, both arms of an `if` contribute constraints
even when only one is "selected" by the witness. Don't write
`if x == 0 { skip_expensive } else { expensive }` expecting to save
gates on the zero path — write a single straight-line expression and
let constant-folding handle constant inputs.

### Public-input layout drift

`main`'s public-input layout is a contract; changing the order
silently breaks every existing prover. `noir-circuit-patterns`
§"Public input layout" is the source of truth.

## 8. Decision checklist for a new `_verified` primitive

Run this before spending a round on a new candidate:

1. **What's the in-circuit-spec baseline?** Write it. Compile and run
   `bb gates` against an amplified harness (N ≥ 32) with `pub`
   witnesses. Record the per-call cost.
2. **What relation am I proposing for the verifier?** Sketch it on
   paper. Does it have a data-dependent shift (cost (a))? A
   data-dependent index (cost (b))? A loop bounded by a witness
   (cost (c))? Each "yes" is a strike.
3. **Does every witnessed value bind back to a public input?** If
   the hint produces `y = f(x)` and the verifier never relates `y`
   to `x` other than via clauses also satisfied by `f(x') ≠ f(x)`,
   the verifier is unsound. Add the binding (§3.4) and count its
   cost against the candidate.
4. **Can I sketch the verifier's `bb gates` from the relation
   shape?** Range checks cost ~50-200 gates each under UltraHonk; a
   bounded Field multiplication is ~20-50 gates; a Field equality is
   small (~5-10 gates) but each `AssertZero` opcode adds glue.
   Predict; if measurement disagrees by >2×, the relation is not
   what you thought it was.
5. **Is there a black-box that does this directly?** §5. Don't
   reimplement Poseidon.
6. **Have I budgeted for both isolated and composed measurement?**
   Run an amplified isolated harness *and* a composed harness that
   mirrors a real call site. If the composed delta differs from the
   isolated delta by more than ~20%, suspect a constant-folding
   artefact.
7. **Does the relation match the Lean obligation in
   `proofs/<crate>/`?** The Lampe extraction needs the verifier's
   relation to match the `*_unique` and `*_correct` lemmas
   line-for-line.
8. **Has a previous spike already tried this?** Check
   `bench/SPIKES.md` and `bench/unconstrained_gate_counts*.json`.
   Don't re-run a negative spike.

## 9. References

Noir documentation (retrieved 2026-05-11; pinned to docs version
`v1.0.0-beta.20` unless noted):

1. Profiler — *"Balancing proving and execution optimisations"* —
   `https://noir-lang.org/docs/tooling/profiler`. Source of the
   "Brillig opcodes are not a proving cost" framing and the "ACIR
   opcodes are at best approximations of proving performance"
   caveat.
2. Profiler — *"Profiling proving backend gates / Flamegraphing"* —
   same URL. Source of the `noir-profiler gates --include_gates_per_opcode`
   invocation and the `blackbox::range` Plookup-setup-cost
   discussion. Verified 2026-05-11 via context7 (`/noir-lang/noir`).
3. ACIR opcodes enum — `acvm-repo/acir/src/circuit/opcodes.rs`,
   `noir-lang/noir`. Six variants: `AssertZero`,
   `BlackBoxFuncCall`, `MemoryOp`, `MemoryInit`, `BrilligCall`,
   `Call`.
4. Issue noir-lang/noir#7525 — *"bug: expression width not honored
   in certain gates"* — defines expression width as the maximum
   number of witness wires per `AssertZero` gate.
5. Unconstrained Functions concept doc —
   `https://noir-lang.org/docs/noir/concepts/unconstrained` —
   canonical `u64_to_u8` example, 65 → 33 ACIR opcodes.
6. `bbup` — Aztec Barretenberg version-pinning tool. Run with no
   arguments to detect the installed `nargo` and pull a compatible
   `bb`. For `nargo 1.0.0-beta.17` this resolves to
   `bb 3.0.0-nightly.20251104`; `bb 0.84.0` errors with
   *"Length is too large"* against beta.17 bytecode.
7. `scripts/benchmark_gates.py` — the in-repo amplification harness.
   The `PRIMITIVE_BENCHMARKS` `*_amplified_*` block instantiates
   each primitive N = 64 times with witness-dependent inputs and
   reports `bb gates` per amplified `main`. Output schema documented
   in `bench/unconstrained_gate_counts.bb_addition.json`.
8. SSA pass `remove_bit_shifts` —
   `compiler/noirc_evaluator/src/ssa/opt/remove_bit_shifts.rs`.
   Lowers runtime shifts to bit-decomposition.
9. Black-box functions reference —
   `https://noir-lang.org/docs/noir/standard_library/black_box_fns`
   (also at `docs/versioned_docs/version-v1.0.0-beta.20/noir/standard_library/black_box_fns.md`).
10. `Field::assert_max_bit_size` — Noir Field type docs,
    `docs/versioned_docs/version-v1.0.0-beta.20/noir/concepts/data_types/fields.md`.
    Signature `pub fn assert_max_bit_size<let BIT_SIZE: u32>(self)`.
    Verified 2026-05-11 via context7.
11. *"Writing Noir efficiently"* —
    `docs/versioned_docs/version-v1.0.0-beta.20/explainers/explainer-writing-noir.md`.
    Source of the `assert_max_bit_size` + array out-of-bound
    redundancy guidance.
12. Noir compiler architecture — `docs/compiler/architecture.md`.
    SSA pass list and ordering, including the non-optional
    `flatten-cfg` pass.
13. `ieee754/src/unconstrained_ops.nr` —
    - `shift_right_sticky_u64_verified` (lines 475-482): the
      pub wrapper that calls the hint and asserts the relation.
    - `shr_sticky_u64_field_hints` (lines 397-415): Brillig hint
      producing `pow2`, `pow2_complement`, and the bit-decomposition
      of `shift`.
    - `verify_shr_sticky_u64_relation` (lines 492-613): the sound
      verifier with the product-chain binding from `shift` to `pow2`.
    - Module preamble documents the explicit-`u<N>`-annotation
      discipline and the crate-private hint convention.

In-repo benchmark ledgers (read these before every optimisation
round):

- `bench/SPIKES.md` — spike-by-spike verdicts and reasoning.
  Entries to date: `clz_u64` bit-decomposition (NO-GO);
  `shr_sticky_u64` Euclidean-division verifier (MIXED);
  `shr_sticky_u64` constant-shift verifier (NO-GO).
- `bench/unconstrained_gate_counts.bb_addition.json` — every
  `*_amplified_*` and `*_isolated_*` benchmark's `bb gates` plus
  `nargo info` (`expression_width`, `acir_opcodes`). The most
  recent run is the source of truth for per-call cost comparisons
  in this skill.
- `bench/unconstrained_gate_counts.json` — earlier ledger, pre-`bb
  gates` instrumentation. Useful for historical context; do not
  use for new decisions.

## 10. `comptime` evaluation — folding loops, constants, and helpers

`comptime` marks code that runs at compile time. The Noir docs list
the applicable forms: `comptime fn`, `comptime global`,
`comptime struct`, `comptime type`, `comptime { ... }` blocks,
`comptime let`, and `comptime for` [14]. The cost relevance is
simple: anything `comptime` is **not** in ACIR, so it pays zero
gates at proving time.

The canonical pattern in this codebase is a `comptime fn` that
loops over a trait associated constant to build a power-of-two
field constant:

```rust
// ieee754/src/float.nr:93-99
comptime fn implicit_bit_field() -> Field {
    let mut r: Field = 1;
    for _ in 0..Self::MANT_BITS {
        r = r * 2;
    }
    r
}
```

`Self::MANT_BITS` is a `u32` trait associated constant — known at
compile time per-impl (`23` for `Float32`, `52` for `Float64`).
The `for` body executes at compile time and the call site sees a
single folded `Field` constant. The same shape repeats for
`sign_field`, `exp_max`, `exp_bias` at
`ieee754/src/float.nr:103-127`.

Verified empirically in this repo: the file-level comment at
`ieee754/src/float.nr:85-86` records that the probe
`A::exp_max() + B::exp_max() + x` compiles to **0 ACIR opcodes**
for the constant portion. The compile-time fold is real, not
hypothetical.

The contrast: `for _ in 0..shift { ... }` where `shift` is a
witness is **not** unrolled and **not** folded — see §6, "Loops
with witness-dependent bounds are not unrolled". The fact that the
loop body is identical at every iteration does not save you; the
backend pays for each constrained iteration.

Rule: when a power-of-two or other compile-time-derivable constant
appears in arithmetic, wrap its construction in `comptime fn` so
the optimiser proves to itself that the value is foldable, rather
than relying on the SSA passes to find the fold post-hoc.

## 11. Generics + monomorphisation

Noir supports const generics on free functions and trait methods
via `<let N: u32>` [15]:

```rust
struct BigInt<let N: u32> { limbs: [u32; N] }
impl<let N: u32> BigInt<N> { fn first(...) -> Self { ... } }
```

Cost-relevant facts:

- **Each distinct instantiation generates its own ACIR.** A
  function called with `N=23` and `N=52` is monomorphised twice;
  the constraints are independent. If you have a helper that
  works at multiple widths, you pay the constraint cost of
  whichever instantiations the call graph reaches.
- **Identical instantiations dedupe.** Two call sites both passing
  `N=23` share a single monomorphisation. There is no per-call-site
  duplication cost; the SSA `inlining` pass (see §6) then folds the
  monomorphised body into the caller.
- **The IEEE 754 `IEEEFloat` trait is *not* generic in this sense.**
  It uses trait associated constants (`let EXP_BITS: u32;`) instead
  of `<let EXP_BITS: u32>` parameters — see `noir-circuit-patterns`
  §"Trait associated constants" for the structural reason. The cost
  consequence: `Float32` and `Float64` each get their own
  monomorphisation of every default method, and a generic helper
  like `pub(crate) fn ieee_eq<T>(a: T, b: T) -> bool where T: IEEEFloat`
  at `ieee754/src/float.nr:688` is monomorphised once per `T`.

Practical implication for cost control: prefer const-generic
parameters over runtime-witness shapes where possible. A function
`fn shr<let N: u32>(x: u64) -> u64 { x >> N }` folds the shift away
because `N` is comptime; the same with `shift: u64` does not (§6
"data-dependent shifts").

## 12. Unconstrained dispatch cost — the per-`unsafe` overhead

`unconstrained fn` bodies run in Brillig and are free at proving
time (§1 "Brillig opcodes are not a proving cost"). The dispatch
itself, however, is **not** free. Each `unsafe { ... }` call site
emits an ACIR `BrilligCall` opcode. The per-call ACIR cost is
approximately **49 opcodes per `unsafe { ... }` call regardless of
the unconstrained body's size** *(verify with `bb gates` on your
exact Noir version — this is a rule of thumb from the codebase's
spike measurements, not a published figure)*. Calling 64 distinct
unconstrained helpers from one verifier costs ~64 × 49 ACIR
opcodes; calling one helper that batches 64 outputs into a single
return costs ~49.

Beta.17 lint requires a `// Safety: ...` comment above every
`unsafe` block explaining why calling the unconstrained function
is acceptable [16]. The Noir docs are explicit that the comment
may sit on the `unsafe` block itself or the statement it resides
in [16]. The convention in this codebase, exemplified at
`ieee754/src/unconstrained_ops.nr:123-126`:

```rust
// Safety: `verify_clz_u23_relation` enforces the relation that
// uniquely binds `count` to `value`. Any tampered count is
// rejected.
let count: u32 = unsafe { count_leading_zeros_u23_unconstrained(value) };
```

Practical optimisation: **batch hint outputs into one
`unconstrained fn` return** so the verifier pays one dispatch, not
N. `shr_sticky_u64_field_hints` at
`ieee754/src/unconstrained_ops.nr:397-415` returns a 3-tuple
(`pow2`, `pow2_complement`, `bits[6]`) from a single Brillig call;
splitting it into three calls would multiply dispatch overhead.

## 13. Witness-vs-comptime branches — both arms cost only for witnesses

Section §6 already covers "both arms of `if` cost" *after*
`flatten_cfg`. The cost-relevant nuance is that `flatten_cfg` only
flattens branches on witnesses. Branches on comptime-known
conditions are eliminated entirely:

```rust
// Folded at compile time -- only the chosen arm contributes ACIR.
if comptime_known { expensive_a } else { expensive_b }

// Flattened to a multiplexer -- both arms contribute ACIR.
if witness_value { expensive_a } else { expensive_b }
```

The boundary between the two: if **every** value used in the
condition can be traced back to literals, comptime globals, trait
associated constants, or `comptime fn` results, the SSA passes
fold the condition before `flatten_cfg` runs and only one arm
survives. As soon as a witness enters the condition, both arms
contribute constraints.

Rule for cost-sensitive code: hoist branches on
trait-associated-constant predicates (e.g. "if this is a 32-bit
float vs a 64-bit float") so they fold; never write
`if cheap_witness_check { skip_expensive } else { expensive }`
expecting to skip the expensive branch — that pattern adds the
check's cost on top of always-paying for the expensive arm.

## 14. Style rules from this codebase's measurements

The marketplace `noir-idioms` skill prefers `if/else { x } else { y }`
over hand-rolled indicator-muxes and recommends precomputing values
once instead of repeating expensive calls. Both are sensible
high-level defaults. For a low-level library that wants to bank
every gate, the calibration on this codebase's measurements is
that two of those defaults are wrong: the optimiser already CSE's
identical sub-expressions (so caching predicate calls is free
verbosity), and mutually-exclusive 3-way+ Field-typed branches
compile to fewer constraints as indicator-muxes than as
`if/else if/else` chains. Both differences trace to specific
commits.

### 14.1 The optimiser already deduplicates identical sub-expressions

Empirical finding from the 2026-05-11 IEEE 754 kernel refactor
(commit `142ca02`): replacing six cached predicate locals at the top
of each `*_with_rounding` wrapper

```rust
// Before: caches `a_is_nan`, `a_is_inf`, ... in locals.
let a_is_nan = self.is_nan();
let b_is_nan = other.is_nan();
let a_is_inf = self.is_infinity();
let b_is_inf = other.is_infinity();
let a_is_zero = self.is_zero();
let b_is_zero = other.is_zero();
// ... later ...
if a_is_zero & b_is_zero { ... }
if a_is_nan | b_is_nan { ... }
```

with the inline form

```rust
if self.is_zero() & other.is_zero() { ... }
if self.is_nan() | other.is_nan() { ... }
```

was **gate-count-identical** across all eight headline benchmarks
(`add`/`sub`/`mul`/`div`_float{32,64}`, ±0 gates). The constraint
optimiser CSE's repeated trait-method calls whose body is a pair of
Field comparisons. Don't cache predicates in locals just to save
"redundant" calls -- the constraints are already shared. Since the
caching is gate-cost-neutral, take whichever form reads better in
context; the inline form is usually it.

**Use the inline form when:** the call is a pure read of struct
fields (`a.is_zero()`, `a.exponent()`, `a.sign()`) or a `comptime
fn` returning a constant (`Self::implicit_bit_field()`,
`Self::exp_max()`).

**Cache in a local when:** the value comes from a verified
relation that costs gates each time it's witnessed
(`denormal_shift_verified::<T>(a.mantissa())` returns a `(Field, Field)`
pair through a Brillig hint + verifier — call it once and reuse the
binding), or a value derived from arithmetic over witness inputs
that doesn't trivially CSE.

### 14.2 `if/else` vs indicator-mux

The `noir-idioms` rule is right for binary 2-way selects over
runtime-witnessed conditions:

```rust
// Idiomatic for 2-way over a witness condition
let val = if cond { x } else { y };
```

The compiler lowers this to a conditional select that costs the same
as `cond_f * (x - y) + y` (one Field multiply + one add). Two
gate-equivalent forms; take the readable one.

**Where indicator-mux wins (measured −86 gates total across the 8
kernels, commit `6f3d204`):** mutually exclusive 3-way (or wider)
chains over **Field-typed values** with **boolean-Field
indicators**. Take the gates, write a one-line comment naming the
branches:

```rust
// Mutually-exclusive 3-way: zero / denormal / normal.
// Indicator-mux beats `if A { x } else if B { y } else { z }` here.
let zero_f: Field   = operand.is_zero() as Field;
let denorm_f: Field = operand.is_denormal() as Field;
let mant: Field = operand.mantissa();
let effective = (1 - zero_f)
    * (denorm_f * mant * pow2_shift
       + (1 - denorm_f) * (mant + implicit_bit));
```

The per-site win is ~5-15 gates; across a kernel's worth of mux
sites that's ~80-200 gates banked. The pattern lives in
`kernels::common::effective_mantissa` /
`effective_exponent_offset`.

**Polarity gotcha — sanity-test every mux.** This session caught two
inverted indicator-mux formulae (mul `bit_overflow` scale,
`a_is_zero` outer multiplier — see commit `b3131d4`). Both compiled
cleanly and passed `nargo check`; only unit tests caught the bug.
Before committing a mux:

1. Plug in `indicator = 0` and read off the result — is it the
   correct branch's value?
2. Plug in `indicator = 1` — same check, the other branch.
3. Run `nargo test` from `ieee754_unit_tests/`. Always. The IEEE 754
   suite has 147 cases that catch sign-bit and edge-case regressions
   in seconds.

Don't trust intuition on polarity. The kill-indicator pattern
`(1 - kill) * (...)` reads naturally (kill = 1 zeroes the result)
but it is easy to type `kill * (...)` instead. The kernel tests
exist for this reason — they caught both bugs.

### 14.3 Mutations (`let mut x = ...; if cond { x = ... }`)

ACIR has no mutation; constrained `let mut` becomes a chain of
conditional selects after SSA flattening, equivalent to the
expression form. The cost is the same as writing the equivalent
chained `if/else`.

**Mutation is fine for:** override-style fixups where the special
cases are mutually exclusive in *practice* but the kernel computes a
"normal" value first and lets the wrapper replace it. The four
`*_with_rounding` skeletons in `crate::float` are written this way:

```rust
let mut result = normal_result;
if result_is_zero { result = Self::signed_zero(result_sign); }
if overflows_to_inf { ...; result = ...; }
if self.is_nan() | other.is_nan() { result = Self::nan(); }
result
```

Each assignment is a conditional select; the chain compiles to one
nested mux. Same cost as writing the equivalent `if/else if/else`
expression, more readable when the special cases have different
conditions.

**Avoid mutation for:** values that the kernel computes
*incrementally* under a runtime-witnessed loop condition. ACIR
flattens this badly — every iteration of an unrolled loop pays the
full mutation chain. Prefer functional folds with a single
"output" Field per iteration.

**Never** use `let mut` to simulate stateful control flow that
constant-folding would otherwise apply to. If you find yourself
writing `let mut state = 0; for ... { if cond { state = state + 1 } }`
where the iteration bound is comptime, write the closed-form
expression instead; the compiler will not deduce it from the
mutation chain.

### 14.4 Summary

| Pattern | Use when | Avoid when |
| --- | --- | --- |
| Inline predicate calls | Reads of struct fields or comptime functions | A witnessed-and-verified pair returned from a Brillig hint |
| Cached local | A verified relation costs gates per witness | A comptime-folded constant or pure field-access read |
| Binary `if/else` expr | 2-way select over a runtime-witnessed `bool` | Mutually exclusive 3-way+ chain over Field values |
| Indicator-mux | Mutually exclusive Field-typed selects, 3-way or wider | Binary selects (cost-equal -- pick the readable one) |
| `let mut result; if X { result = ... }` | Override-style special-case fixups in a wrapper | Stateful accumulation under a comptime-foldable loop |

Always measure with `bb gates`. The per-site delta in this codebase
is ~5-15 gates -- which compounds across kernels and is precisely
the kind of saving a low-level library is in the business of
banking. Polarity errors are recoverable in seconds with `nargo
test`; gate regressions are not always so cheap to catch.

### 14.5 Readability rules where the gate cost is equal

These rules apply when both forms compile to the same constraint
count -- they cost nothing to follow and improve cross-reader speed.

* **`bool as Field`, never `if b { 1 } else { 0 }`.** A `bool` is
  already `{0, 1}` in the field; the cast is an identity and the
  `if/else` lowers to the same conditional select. Examples in
  `ieee754/src/float.nr`:

  ```rust
  // Prefer:
  Self::signed_infinity(sign as Field)

  // Avoid (gate-equivalent but more characters of noise):
  Self::signed_infinity(if sign { 1 } else { 0 })
  ```

* **`<` / `>` over `.lt()` / `.gt()` for typed integers.** `Field`
  has no `<` operator (no natural total order on a finite field),
  so `Field::lt` / `Field::gt` are the only option there -- use
  them. For typed values (`u8`, `u32`, `u64`, ...) prefer the
  built-in operator syntax: `x < y` reads better than `x.lt(y)`
  and compiles to the same plookup-backed range check. Don't
  cast a typed value to Field just to use `.lt()`; if the value is
  already typed, keep it typed and use `<`.

* **`pow_32` over hand-rolled exponentiation loops for runtime
  exponents.** Where the exponent is a witness or runtime-derived
  value, `2.pow_32(exp)` is shorter and equivalent (or cheaper)
  than walking a per-row power-of-two table verifier; see
  `kernels::common::denormal_shift_verified` and `kernels::*`
  call sites. **Exception:** comptime exponents in `comptime fn`
  trait defaults -- the for-loop body
  `let mut r = 1; for _ in 0..N { r = r * 2; }` folds to a Field
  literal per impl, while `2.pow_32(N)` runs at runtime per call
  site (measured -~700 gates per op when an `assert_lt(x,
  2.pow_32(...))` attempt was reverted in `b3131d4`). See §15.1.

## 15. Keep values as Field; avoid round-trips through typed integers

Empirical finding from the 2026-05-11 IEEE 754 kernel migration
(commits `f08ba09` through `15b464f`): every `Field -> u64` cast
emits a range check (a `assert_max_bit_size::<64>` worth of
constraints — a few plookup rows). Every `u64 -> Field` cast is
free (the typed value is already known small). A round-trip
`some_field as u64 as Field` therefore pays the range check for
nothing.

**The kernels' biggest single optimisation this session was simply
not converting.** Pre-rewrite `mul_float64` was 8943 `bb gates`
because the mantissa product went through `u128` (a struct of two
`u64`s) and the per-step intermediates were `u64`-typed. Post-rewrite
it is 6282 — the product is a single Field multiplication, and the
intermediates that used to be `u64` are `Field` throughout. f64
division dropped even more drastically (20015 -> 6360, −68%) because
the long-division loop's `u128 / u64` was replaced by a single
Field-arith Euclidean witness `dividend == q * divisor + r` with
`q.assert_max_bit_size::<Q_BITS>()` and `r.lt(divisor)`. Same
correctness, no `u128` plumbing.

### 15.1 Concrete rules

- **Default to `Field` for intermediate values.** Bit-widths,
  positions, shift amounts, mantissa-with-implicit-bit values, even
  exponents — all `Field`. Only cast at function boundaries where
  the callee insists on a typed argument.
- **Don't introduce a `u64` local just to call `&`/`>>`/`<<`.**
  These typed bit-ops compile through black-box decomposition (see
  §1's `bb_gates` example: 34 ACIR opcodes for a typed shift
  compiled to MORE gates than 158 ACIR opcodes of Field-arith
  Euclidean splits). For shift-by-power-of-two, write
  `value * 2^k` (left shift) or witness `(q, r)` with `value == q *
  2^k + r` (right shift); both are pure Field arithmetic.
- **Avoid `i64` entirely.** Noir's `i64` doesn't cast directly to
  `Field` (compile error: "Only unsigned integer types may be casted
  to Field"). Track signed quantities in **offset form** instead:
  pick an `offset` larger than the maximum negative magnitude and
  store `value + offset`. The mul/div/sqrt kernels do this with
  `offset = T::implicit_bit_field()` for unbiased exponents (see
  `kernels::common::effective_exponent_offset`). Comparisons use
  `Field::lt`; the offset cancels in subtractions.
- **Use comptime-folded constants, not `pow_32`.** `T::implicit_bit_field()`,
  `T::exp_max()`, `T::exp_bias()` are `comptime fn`s whose for-loop
  bodies fold to Field literals per impl (verified: the probe
  `A::exp_max() + B::exp_max() + x` compiles to 0 ACIR opcodes).
  Calling `2.pow_32(Self::MANT_BITS)` produces the same value at
  runtime, costing ~198 gates per call site (measured in §16's
  primitive table; pow_32 slope = 198/call). An attempt to switch
  `compose` to `assert_lt(mantissa, 2.pow_32(Self::MANT_BITS))`
  cost ~700 gates per op until reverted in `b3131d4` -- that
  number combined the `assert_lt` setup, the `pow_32` calls, and
  several other deltas, but the broader rule stands: runtime
  `pow_32` of a comptime input wastes the cost of a per-circuit
  pow_32 amortisation chain that the for-loop pattern folds away.

### 15.2 When a `u64` cast IS necessary

Some helpers in this workspace take `u64` arguments out of legacy
convenience (`should_round_up`, `count_leading_zeros_u64_verified`,
`shift_right_sticky_u64_verified`). Each call site pays:

* one `assert_max_bit_size::<N>` for `Field -> u64` (cheap with
  plookup, but not free);
* the cost of the typed helper's body;
* one `as Field` on the way back (free).

Empirical guidance from the migration:

* **CLZ in the underflow normaliser** (`kernels/add.nr` line ~280):
  the existing `count_leading_zeros_u64_verified` saves more gates
  than a hypothetical Field-typed CLZ would, so the `u64`
  round-trip is justified. Documented as a known typed island in
  the kernel header.
* **`should_round_up`** (called from each kernel's rounding stage):
  re-implementing the directed-rounding dispatch in Field arithmetic
  was measured **more expensive** than the round-trip in sqrt; the
  typed helper stays. Documented similarly.
* **`shift_right_sticky_u64_verified`** in `add.nr`'s exponent
  alignment: the `_field_verified` sibling exists and is preferred
  in div/mul/sqrt where the surrounding code is already Field-typed.
  In add the alignment shift lives inside a u64-typed mantissa
  context so the round-trip is cheaper than refactoring the
  surrounding context.

The rule: when you find a u64 cast in the constrained body, ask
whether the surrounding context is already Field-typed. If yes,
look for a `_field` sibling helper. If no, the round-trip may be
the right local optimum — leave it and document why.

### 15.3 Field as the working type even where u128 "should" be

f64 mantissa products are 53×53 → 106 bits, which "needs" `u128`.
But `Field` is BN254 (~2^254), so 106-bit products fit in `Field`
with 148 bits of headroom. The pre-rewrite f64 mul kernel had a
hand-rolled `mul_u64_to_u128` doing schoolbook 32-bit-limb
multiplication; the post-rewrite is

```rust
let product: Field = a_mant_full * b_mant_full;
```

Then a Brillig hint produces `(high: u64, low: u64)` and the
verifier asserts `(high * 2^64 + low) as Field == product` with
range checks on `high` and `low`. One Field multiply replaces a
30-line `mul_u64_to_u128`. Same correctness, fraction of the gates.

The same trick applies anywhere "I need more than 64 bits of
intermediate state": work in Field, range-check the split at the
boundary, and only reach for `u128` if you actually need typed
bit-ops on a 128-bit value that won't fit a Field-arith Euclidean
relation. (In practice, you almost never do.)

## 16. Primitive-operation cost table (for code generators)

Measured 2026-05-11 on `bb 3.0.0-nightly.20251104` (UltraHonk) against
Noir 1.0.0-beta.17.

**Methodology -- read carefully.** A naive "one probe, one operation"
benchmark is misleading on UltraHonk because **most primitives have a
large one-time plookup-table setup cost that dwarfs the per-call
cost**. A circuit calling `Field::lt` once measures ~2979 gates;
calling it 20 times measures 6444. The marginal cost is **183 gates
per call, not 2979**. The 2796-gate one-time overhead amortises across
all calls in the same circuit that share the same plookup table.

Numbers below are obtained by running each primitive N times in a
loop, plotting bb_gates against N, and reporting the **slope** (the
per-call marginal cost) and the **intercept** (the one-time setup).
For a code generator emitting many calls in one circuit, the slope
is the right per-op cost. For a code generator concerned about a
single isolated use, use intercept + slope.

| Operation                       | Per-call (slope) | One-time setup (intercept) |
| ------------------------------- | ---------------: | -------------------------: |
| `Field` `+`, `*`                |                0 |                          0 |
| `bool as Field`                 |                0 |                          0 |
| `if b { 1 } else { 0 }`         |                0 |                          0 |
| `Field` `==`, `!=`              |              1-2 |                          0 |
| `assert(x == y)` (Field)        |                1 |                          0 |
| `if cond { x } else { y }`      |                2 |                          0 |
| `cond_f*x + (1-cond_f)*y` (mux) |                2 |                          0 |
| `assert_max_bit_size::<8>`      |              ~2 |                        105 |
| `assert_max_bit_size::<16>`     |              ~2 |                       2799 |
| `assert_max_bit_size::<24>`     |              ~3 |                       2967 |
| `assert_max_bit_size::<32>`     |              ~3 |                       2800 |
| `assert_max_bit_size::<48>`     |              ~4 |                       2808 |
| `assert_max_bit_size::<64>`     |              ~4 |                       2840 |
| `u64 ==`                        |              ~11 |                       2842 |
| `u64 <`                         |              ~13 |                       2843 |
| `u64 >> const`                  |              ~13 |                       2849 |
| `u64 >> witness`                |              ~96 |                       2796 (then floor jump at N=2 to 4120) |
| `Field::lt(x, y)`               |              183 |                       2796 |
| `assert_lt(x, y)` (bn254)       |              183 |                       2799 |
| `pow_32(x)` (runtime x)         |              198 |                         57 |
| `u64 &` (bitwise AND)           |              ~85 |                       4035 (floor jump at N=2 to 8232) |

**Read this as:** the first call to `Field::lt` in a circuit
contributes ~2979 gates (setup + 1 call). The 2nd through Nth call
each contribute 183. So a circuit calling `Field::lt` 10 times costs
~2796 + 10*183 = ~4626 gates total -- not 10 * 2979 = 29,790.

Raw amortisation data (the slopes above are computed from these):

```
Field::lt           N=1: 2979,  N=2: 3162,  N=10: 4621,  N=20: 6444    -> 183/call
assert_max_bit_size::<8>   N=1:  107,  N=2:  110,  N=10:  128,  N=20:  150    -> 2.3/call, setup ~105
assert_max_bit_size::<16>  N=1: 2801,  N=2: 2804,  N=10: 2824,  N=20: 2848    -> 2.5/call, setup ~2799
assert_max_bit_size::<24>  N=1: 2970,  N=2: 2974,  N=10: 2994,  N=20: 3018    -> 2.5/call, setup ~2967
assert_max_bit_size::<32>  N=1: 2803,  N=2: 2806,  N=10: 2828,  N=20: 2855    -> 2.7/call, setup ~2800
assert_max_bit_size::<48>  N=1: 2812,  N=2: 2817,  N=10: 2849,  N=20: 2888    -> 4.0/call, setup ~2808
assert_max_bit_size::<64>  N=1: 2844,  N=2: 2849,  N=10: 2883,  N=20: 2925    -> 4.3/call, setup ~2840
u64 <               N=1: 2856,  N=2: 2866,  N=10: 2975,  N=20: 3112    -> 13/call
u64 ==              N=1: 2853,  N=2: 2861,  N=10: 2952,  N=20: 3067    -> 11/call
u64 &               N=1: 4120,  N=2: 8232,  N=100: 11346, N=200: 19871  -> ~85/call after the floor jumps
u64 >> witness      N=1: 2896,  N=2: 4120,  N=100: 12451, N=200: 22076  -> ~96/call after the floor jumps
pow_32              N=1: 255,   N=2: 453,   N=10: 2037,  N=20: 4017    -> 198/call
```

### 16.1 Code-generator decision rules from this table

These are the rules a generator can apply mechanically. Numbers cite
per-call costs (the slope), since a real circuit emitting these
primitives will pay the one-time setup once.

1. **Field `+`, `*`, `==`, `!=`, `assert(==)`, simple muxes are
   free or near-free** (0-2 gates). Generate as many as the
   algorithm needs. Don't hoist or share these.

2. **`bool as Field` and `if b { 1 } else { 0 }` are
   gate-equivalent.** Both compile to the same conditional select
   (0 gates over baseline). Emit `b as Field` for readability.

3. **`if/else` and indicator-mux are gate-equivalent on a single
   binary Field select** (both 2 gates). The 5-15 gate per-site
   saving §14.2 describes comes from chained / nested 3-way+
   mutually-exclusive branches, not from individual binary muxes.

4. **`assert_max_bit_size::<N>` per-call cost is ~2-4 gates
   regardless of N.** The cost discontinuity is in the *setup*, not
   the per-call:

   * **`<8>` setup = ~105 gates.** bb composes 8-bit range checks
     from byte-decomposition tables that are already in the
     circuit's standard machinery; no dedicated plookup table is
     instantiated.
   * **`<16>`, `<32>`, `<48>`, `<64>` setup = ~2800-2840 gates.**
     Each instantiates a dedicated plookup table for that bit-width.
   * **`<24>` setup = ~2967 gates.** Non-power-of-2 widths are
     composed from smaller power-of-2 tables (~24 ≈ 16 + 8 or 3×8),
     adding overhead vs `<32>`.

   Implications for a code generator:

   * **Don't skimp on range checks** for soundness -- the marginal
     cost is trivial (2-4 gates).
   * **Tightening N saves setup gates, not per-call gates.**
     Migrating from `<64>` to `<32>` on every call saves ~40 setup
     gates if the circuit already uses `<32>` somewhere; saves
     nothing if both N values are needed in different places (each
     bit-width table is set up independently).
   * **Prefer power-of-2 bit-widths.** A circuit using `<24>` pays
     ~167 more setup gates than the same circuit at `<32>`. A code
     generator emitting many bit-widths should round up to
     `{8, 16, 32, 64}`.
   * **`<8>` is functionally free** (105 setup + ~2/call). Use it
     for narrow values without hesitation.
   * **Don't fragment plookup tables.** A circuit using one each of
     `<14>`, `<16>`, `<18>`, `<20>` range checks pays four separate
     setup costs. Settle on `<16>` for all four; cost = one table.

5. **u64 comparisons (`==`, `<`) are ~11-13 gates per call** -- the
   cheapest comparison primitive measured. Generator should prefer
   them over `Field::lt` (~183/call) whenever the value fits a
   typed integer.

6. **`Field::lt` is ~183 gates per call** -- ~14x more expensive
   than `u64 <`. Use only when the value is genuinely a Field
   element (e.g. an offset-form exponent that ranges beyond u64,
   or a Euclidean-split remainder bounded by another Field
   divisor). For values that fit in `u64`, cast and use `<`.

7. **u64 bitwise ops are ~85 gates per call for `&`, ~96 for
   `>> witness`.** Moderately expensive but FAR cheaper than the
   isolated-probe numbers suggested. The original §16 (now
   superseded) quoted 4062 gates for `u64 &` -- that was setup +
   one call. Per-additional-call cost is ~85. The generator should
   still prefer Field-arith Euclidean splits where they're
   structurally cheaper, but a small handful of `u64 &` calls is
   not a disaster.

8. **`pow_32(x)` is ~198 gates per call** for runtime x. Use it
   instead of walking a per-row power-of-two table for runtime
   exponentiation. For comptime exponents the for-loop pattern
   folds to a literal (0 gates per call site).

9. **Constant-shift vs witness-shift on `u64`**: ~13 vs ~96 gates
   per call. Constant shifts are 7x cheaper -- meaningful for
   bit-extraction loops over 64 positions, but not catastrophic
   for a handful of calls.

### 16.2 What the "one-time setup" cost is actually doing

UltraHonk uses Plookup tables for range checks and certain typed
operations. The first call to e.g. `assert_max_bit_size::<64>` in a
circuit instantiates the 64-bit range-check table; subsequent calls
add only the table-lookup constraint per value. Different bit-widths
have different tables -- which is why `<32>` and `<64>` have
near-identical per-call cost (each instantiates its own table) but
neither subsumes the other, and a circuit using both pays both
setup costs.

Consequence: **a circuit's reported cost ≠ sum of its primitives'
isolated costs.** The IEEE 754 kernels measure ~5000-6500 gates
each, not the ~30,000 you'd estimate from naively multiplying
isolated per-call numbers. Always measure the actual generated
circuit. The plookup-table-fragmentation guidance is in §16.1 rule
4.

### 16.3 Reproduction recipe

To re-measure per-call costs (slope) on a different toolchain:

```bash
# Make N versions of the same probe with N=1, 2, 10, 20 calls.
for n in 1 2 10 20; do
  mkdir -p "p_op_${n}/src"
  cat > "p_op_${n}/Nargo.toml" <<TOML
[package]
name = "p_op_${n}"
type = "bin"
authors = [""]
[dependencies]
ieee754 = { path = "<path-to-ieee754>" }
TOML
  body=""
  for i in $(seq 1 $n); do
    body="${body}    r = r + x.lt(y + $i) as Field;\n"
  done
  printf 'fn main(x: pub Field, y: pub Field) -> pub Field {\n    let mut r: Field = 0;\n%b    r\n}\n' \
    "$body" > "p_op_${n}/src/main.nr"
done

# Measure each.
for n in 1 2 10 20; do
  (cd "p_op_${n}" && nargo compile 2>/dev/null && \
   bb gates -s ultra_honk -b "target/p_op_${n}.json" \
     | grep circuit_size | sed -E "s/.*: ([0-9]+).*/N=${n}  bb_gates=\\1/")
done
```

Slope = (bb_gates(N=20) - bb_gates(N=1)) / 19, in gates per call.
Intercept = bb_gates(N=1) - slope.

## References for §10-§13

14. Noir comptime concept doc —
    `docs/versioned_docs/version-v1.0.0-beta.20/noir/concepts/comptime.md`.
    Lists `comptime fn`, `comptime global`, `comptime struct`,
    `comptime type`, `comptime { ... }`, `comptime let`,
    `comptime for`. Verified 2026-05-11 via context7.
15. Noir generics concept doc —
    `docs/versioned_docs/version-v1.0.0-beta.20/noir/concepts/generics.md`.
    Source of the `<let N: u32>` const-generic syntax for structs,
    free functions, and `impl` blocks. Verified 2026-05-11 via
    context7.
16. Noir unconstrained concept doc —
    `docs/versioned_docs/version-v1.0.0-beta.20/noir/concepts/unconstrained.md`.
    *"A `// Safety: ...` comment is required to explain why
    calling the unconstrained function is acceptable, either on
    the `unsafe` block itself or the statement it resides in."*
    Verified 2026-05-11 via context7. The ~49-ACIR-opcode
    per-`unsafe`-call dispatch figure is empirical from this
    repo's measurements — verify on your exact Noir version with
    `bb gates`.
