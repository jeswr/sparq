---
name: noir-circuit-patterns
description: Patterns and gotchas for writing Noir circuits that implement SPARQL primitives over committed RDF graphs. Use when implementing BGP matchers, joins, filters, projections, or hashing-to-field strategies; when sizing constraint budgets; when laying out public inputs that bind to a commitment scheme; or when interfacing nargo / Barretenberg (bb) tooling. Always confirm syntax against the noir-lang context7 docs (/noir-lang/noir) before writing code.
---

# Noir circuit patterns for ZKP-SPARQL

Reusable patterns for the Noir circuits that prove correct SPARQL
evaluation over committed RDF graphs. These are starting points;
verify any specific syntax via the `/noir-lang/noir` context7 docs
before pasting into code.

For generic Noir guidance the upstream skills cover better:
`noir-developer` for circuit structure / stdlib / workspace setup,
`noir-idioms` for the idiomatic ACIR-vs-Brillig style, `noir-testing`
for the testing framework. This skill is intentionally narrow to:
SPARQL-domain patterns (BGP, joins, filters, triple commitments),
plus the Noir-language gotchas this codebase has hit specifically
(orphan rule, `pub(crate)` for verifier-hint soundness, trait
associated constants in const-generic positions, IEEE-754
`total_compare` vs `Ord`, witness-vs-comptime branches). For cost
analysis on a candidate gadget see the sister skill `noir-optimisation`.

## Always-do

- **Pin the toolchain.** `Nargo.toml` declares `compiler_version`.
  README of the circuit dir documents the matching `bb` version.
- **Query context7 first.** Before guessing a stdlib function name, a
  trait bound, or a generic syntax, run
  `mcp__context7__query-docs` against `/noir-lang/noir`. The Noir
  language has evolved fast; recall is unreliable.
- **Document `main`'s public inputs.** A commented-out line per
  public input naming what it commits to and why.
- **Test every primitive.** Noir supports `#[test]` in source — use
  it. Cover an accept case and at least one reject case.
- **Comment constraint cost.** When writing a non-trivial gadget,
  estimate where the dominant cost lies (hashing? bit-decompositions?
  range proofs?) and note it.

## Triple commitment patterns

The credential graph is committed before the circuit consumes it. The
two natural shapes:

- **Per-triple Pedersen / Poseidon hash, accumulated into a Merkle
  tree.** The prover supplies a triple + a Merkle path; the circuit
  verifies the path against a public root. Best when triples are
  accessed sparsely (e.g. BGP with few patterns).
- **Sorted sequence committed via a single sponge hash.** The prover
  supplies the full ordered triple list; the circuit re-hashes and
  asserts equality with a public commitment. Best when most of the
  graph is touched anyway, or when sortedness is needed downstream
  (e.g. for sort-merge joins).

Either way, **domain-separate** the hash inputs (e.g. tag a triple
hash with a `b"triple-v1"` prefix) and document the tag in a
single table shared with `vc-cryptography`.

## BGP matching

For a triple pattern `(s ?x p o ?y)` with two variables:

1. Extract the candidate triple from the committed graph (Merkle
   path or sponge index).
2. Constrain the constants (`s`, `p`) to equal the corresponding
   triple positions.
3. Bind the variables (`?x`, `?y`) to the remaining positions.
4. The output is a witness binding for downstream operators.

Multi-pattern BGPs are joins over the per-pattern bindings — see
joins.

## Joins

- **Sort-merge** when both sides are sorted on the join key (cheap
  per-row, but needs a sort proof or pre-sorted commitment).
- **Lookup-table / set-membership** when one side is small and can be
  pre-loaded as a polynomial commitment.
- **Hash join** is rarely the right choice in-circuit because
  building a hash table burns constraints linear in the build side.

For a paper at the BGP level, sort-merge against a graph already
committed as a sorted sponge is usually the right default.

## Filters

`FILTER (?x > 5)` and friends decompose into:

- a comparison gadget (use `std::cmp` where it exists; else a
  bit-decomposition).
- a conditional select that drops the row.

Comparisons on field elements need a bit-decomposition unless the
values are known small. Document the bit budget in a comment.

## Projection / DISTINCT

Projection is free; DISTINCT requires either:

- a sort + adjacent-deduplicate pass, or
- a multi-set hash (commit to the unordered solution multi-set and
  argue equality).

The multi-set hash route plays nicely with Barretenberg's native
field; prefer it unless the verifier explicitly wants the result
ordered.

## Public input layout — the contract

Every `main` exports a precise public-input layout:

```
fn main(
    // -- public --
    graph_commitment: Field,        // Pedersen / Poseidon root over committed triples
    query_commitment: Field,        // commitment to the SPARQL query the prover claims to evaluate
    result_commitment: Field,       // commitment to the disclosed result multi-set
    // -- private --
    graph_witness: ...,             // triples + Merkle paths
    binding_trace: ...,             // intermediate operator bindings
    // ...
) -> pub Field {
    // verify graph commitment, evaluate, re-commit result, equality-assert
}
```

The exact shape evolves; what's invariant is that the verifier sees
*only* the three commitments + the result.

## Tooling cheats

- `nargo check` — fast type-check.
- `nargo test` — runs `#[test]` functions.
- `nargo execute` — run a circuit with a `Prover.toml` and inspect
  intermediate witnesses; useful for debugging before proving.
- `bb prove` / `bb verify` — proving / verifying via Barretenberg.
- `nargo info` — constraint count and gate breakdown; the budget
  document.

## When to escalate

If a primitive doesn't fit the constraint budget, **escalate to
`sparql-semantics` and the main session before optimising blindly** —
the right move may be to refine the supported fragment, not to
golf the gadget.

## See also

- `noir-optimisation` — cost model + decision rules. Read before
  spiking any `unconstrained + verified` primitive: §2 has the
  profitability conditions, §3 the bit-decomposition vs dynamic-shift
  trade-off, §8 the pre-spike checklist.

## Trait associated constants

Noir traits can declare associated constants with `let CONST: u32;`
syntax. From the Noir docs [t1]:

```rust
trait MyTrait {
    type Foo;
    let Bar: u32;
}

impl MyTrait for Field {
    type Foo = i32;
    let Bar: u32 = 11;
}
```

The constant can then be used in const-generic positions, e.g.
`x.assert_max_bit_size::<{ Self::EXP_BITS }>()`. In beta.16/17 the
expression in the const-generic slot is restricted: **only `+`, `-`,
`*`, `/`, `%` are accepted in the initialiser** *(verify on your
exact Noir version)*. Bitwise operators and function calls there
are not.

The IEEE 754 trait at `ieee754/src/float.nr:28-46` uses this
pattern as the foundational design choice:

```rust
pub trait IEEEFloat {
    let EXP_BITS: u32;
    let MANT_BITS: u32;
    // ...
}

impl IEEEFloat for Float32 {
    let EXP_BITS: u32 = 8;
    let MANT_BITS: u32 = 23;
    // ...
}
```

The default method `compose` at line 133-138 uses these in
const-generic position:

```rust
exponent.assert_max_bit_size::<Self::EXP_BITS>();
mantissa.assert_max_bit_size::<Self::MANT_BITS>();
```

Critical design note from `ieee754/src/float.nr:39-43`: the trait
keeps `EXP_BITS` / `MANT_BITS` as **two independent constants** rather
than deriving them from a single `TOTAL_BITS`, because
`assert_max_bit_size::<N>` requires a const-generic expression in
`N`, and the beta-era restriction on which arithmetic is allowed
there made `TOTAL_BITS - EXP_BITS - 1` not always acceptable. Keep
this in mind when adding new layout constants — prefer declaring
them as separate associated constants over deriving them with
complex arithmetic in const-generic positions.

For cost consequences (folding, monomorphisation), see
`noir-optimisation` §10-§11.

## Const generics vs trait associated constants

Noir has two related mechanisms for compile-time numerics on
generic items:

- **Const generic parameters:** `<let N: u32>` on a free function,
  struct, or `impl` block [t2]. Example:
  `struct BigInt<let N: u32> { limbs: [u32; N] }`. The value is
  supplied at the call / instantiation site.
- **Trait associated constants:** `let CONST: u32;` declared in
  the trait and filled in by each `impl`. The value is fixed per
  implementation; callers select via the `Self` type.

Pick the one that matches the dependency: const generic parameter
when the value varies per call (different array widths in the same
code path); trait associated constant when the value is intrinsic
to a type (an IEEE 754 format's layout is intrinsic to that
format, not chosen per call site).

## Witness-vs-comptime branches

`if c { a } else { b }` has two compilation modes:

- If `c` is comptime-known (a literal, a `comptime global`, a
  trait associated constant, or the result of a `comptime fn`),
  the SSA passes fold the condition before `flatten_cfg` runs and
  only the chosen arm contributes constraints.
- If `c` involves any witness, the `flatten_cfg` SSA pass
  rewrites the `if` to a conditional select: both arms execute
  and their constraints both apply, with the result multiplexed
  on `c`.

Consequence for structural patterns:

- A `BGP` matcher that branches on a constant pattern position
  (`if subject_is_iri { ... }`) folds when `subject_is_iri`
  comes from the query commitment.
- A filter that branches on a witness value (`if x > 5 { keep }
  else { drop }`) pays for both arms even when only one is
  "selected"; structure the filter as a single straight-line
  expression with a conditional multiplier instead.

See `noir-optimisation` §6 and §13 for the SSA-pass detail.

## Adversarial-prover testing

For the basics of `#[test]` / `#[test(should_fail)]` /
`#[test(should_fail_with = "...")]` and general test organisation,
see the `noir-testing` skill (assertion patterns, test attributes,
test organisation).

Convention specific to this workspace: every `unconstrained + verified`
pattern needs an accept-path `#[test]`, plus at least one
`should_fail_with` per distinct failure mode the verifier's clauses
encode. Feed deliberately wrong witnesses through the production
`verify_*_relation` helper and assert each specific tampering is
caught by name. The shared verifier helper (see `noir-optimisation`
§7 "Shared-test-helper anti-pattern") means adversarial tests and
production calls share verifier code, so regressions surface in both.

Example from `ieee754/src/unconstrained_ops.nr`:

```rust
#[test(should_fail_with = "shift==0 quotient mismatch")]
fn rejects_shift0_quotient_mismatch() { ... }
```

The message-match is essential -- it pins WHICH clause caught the
lie, not just that some clause did. Drop a `should_fail_with` for
each clause in the verifier's safety proof.

## Orphan rule

Noir enforces the orphan rule for trait implementations: you
cannot write `impl Foreign for Foreign` — at least one of the
trait or the type must be defined in the current crate. This
blocks the natural-looking:

```rust
// Rejected: both `From` and `Field` are foreign.
impl From<Float32> for Field {
    fn from(f: Float32) -> Field { ... }
}
```

The workaround used throughout this codebase is to define an
**inherent method** instead of an impl of a foreign trait. From
`ieee754/src/float.nr:141-145`:

```rust
fn to_field(self) -> Field {
    self.sign() * Self::sign_field()
        + self.exponent() * Self::implicit_bit_field()
        + self.mantissa()
}
```

Call sites use `f.to_field()` rather than `Field::from(f)`. The
same convention applies in reverse — `From<Field>` for `Float32`
is fine (the impl is in the same crate as `Float32`), but bear in
mind whether the crate boundary supports the impl you want before
reaching for `From` / `Into`.

## Standard library traits in this stack

Noir's standard library exposes the conventional Rust-style
traits *(verify exact paths on your Noir version)*:

- `std::convert::From` / `Into`
- `std::ops::{Add, Sub, Mul, Div, Neg}` for operator overloading
- `std::cmp::{Eq, Ord}` for equality / ordering

**Watch out: not every operation a type "supports" has total
semantics.** IEEE 754 `<` is **not** total — comparing any value
to NaN returns `false`, so `a < b`, `a > b`, and `a == b` all
return `false` for NaN inputs simultaneously. This violates the
`Ord` contract (a total order). Implementing `Ord` for `Float32`
would either lie about the semantics or produce non-IEEE
results.

This codebase's resolution at `ieee754/src/float.nr:508`:
implement a separate `total_compare` method that follows IEEE
754-2008's `totalOrder` predicate, returning `i8` with order
`-NaN < -Inf < ... < -0 < +0 < ... < +Inf < +NaN`. NaN payloads
break the tie within same-sign NaNs by mantissa. The
trait-method form sidesteps `Ord` entirely; callers reach for
`total_compare` when they need lawful sortability and for
`ieee_eq` (`ieee754/src/float.nr:688`) when they need IEEE
equality (NaN ≠ NaN).

Rule: before implementing a foreign comparison / arithmetic
trait for a domain type, check whether the trait's contract is
satisfied. If not, expose the operation as an inherent method
under a name that signals the semantics (`total_compare`,
`ieee_eq`, etc.) and document why the trait was avoided.

## `pub(crate)` for soundness

Noir's visibility modifiers include `pub`, `pub(crate)`, and the
crate-private default [t3]. The `pub(crate)` modifier is the
load-bearing tool for keeping `unconstrained` hints sound.

**Rule: every `unconstrained fn` hint is `pub(crate)` (or
private), never `pub`.** A `pub unconstrained fn` exposes the
unverified Brillig hint through the crate's public API, letting
downstream callers wire the hint directly into their circuits
without the in-crate verifier — silently breaking soundness for
anyone who imports the crate.

This is not theoretical: PR #36 in this repo shipped a
`pub unconstrained fn count_leading_zeros_u23_unconstrained`, and
roborev's review forced the visibility change in commit `c7ecb8a`.
See `noir-optimisation` §7 "Unconstrained outputs leaking" for
the cost-and-soundness context.

The corresponding `pub(crate)` pattern at
`ieee754/src/unconstrained_ops.nr`:

```rust
// pub(crate) -- internal hint; only the verifier may call it.
pub(crate) unconstrained fn count_leading_zeros_u23_unconstrained(
    value: u32,
) -> u32 { ... }

// pub -- the verified wrapper is the only sanctioned public entry.
pub fn count_leading_zeros_u23_verified(value: u32) -> u32 {
    // Safety: `verify_clz_u23_relation` enforces the relation ...
    let count: u32 = unsafe { count_leading_zeros_u23_unconstrained(value) };
    verify_clz_u23_relation(value, count);
    count
}
```

The `pub(crate)` keeps the hint reachable from `verify_*_relation`
and from adversarial `#[test]`s in the same crate, while
preventing external callers from bypassing the verifier.

The same applies to `verify_*_relation` helpers themselves: they
are `pub(crate)` so the production wrapper *and* the adversarial
tests can both call them, but external crates cannot wire raw
relations into circuits the in-repo Lean proofs do not cover.

## References for new sections

t1. Traits — Noir docs,
    `docs/versioned_docs/version-v1.0.0-beta.20/noir/concepts/traits.md`.
    Source of the `let Bar: u32;` associated-constant syntax.
    Verified 2026-05-11 via context7.
t2. Generics — Noir docs,
    `docs/versioned_docs/version-v1.0.0-beta.20/noir/concepts/generics.md`.
    Source of the `<let N: u32>` const-generic syntax. Verified
    2026-05-11 via context7. The beta.16/17-era restriction to
    `+ - * / %` in the initialiser of an `assert_max_bit_size::<{ ... }>`
    expression is from in-repo experience (see
    `ieee754/src/float.nr:39-43` comment); verify on your exact
    Noir version.
t3. Visibility modifiers — `pub(crate)` is exemplified throughout
    this codebase (e.g. `ieee754/src/types.nr:23-25`,
    `ieee754/src/float.nr:688`,
    `ieee754/src/float32/add.nr:19`); the canonical Noir docs
    entry for visibility could not be confirmed via context7 in
    this round — verify on your Noir version before claiming
    exact behaviour beyond "restricts to current crate".
