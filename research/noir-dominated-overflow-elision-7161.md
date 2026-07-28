# Dominated overflow-check elision — noir design + measurement record (sq-jthy1)

**Bead:** `sq-jthy1` (epic `sq-uuvac`, `noir-optimization-program.md` §7 row 6, §10.3 *"elide overflow
checks dominated by a later range check (#7161); Brillig failure-point semantics in scope"*) ·
**Status:** **implemented, unit-tested and measured at ACIR-opcode level in a throwaway noir clone.
It works, and no regressions were observed in the suites and corpus that were run (§6, §8 lists what
was not run) — but it fires on 5 of 519 corpus packages for −0.029% ACIR opcodes, and
`bb` is absent so there is NO gate measurement. On that evidence NO upstream PR is proposed and the
bead stays OPEN.** The patch is **not carried in this repo** · **Author:** SPARQ agent 🤖 [OPUS-5] ·
**Date:** 2026-07-27

Analysed, built and measured against `noir-lang/noir` @ `e22cd89b` (workspace version
`1.0.0-beta.25`), in a throwaway clone outside this repo. Every line citation is at that commit.
Companion records: `noir-optimization-program.md` (§3.2 pass map, §3.3 ACIR-gen map, §4.2 landmine
list, §10 fleet spec), `noir-acir-expression-growth-4629.md`,
`noir-not-canonicalization-ifelse-merging.md` (whose §8 toolchain recipe this run reused).

## 0. What this record is, and what it is not

The bead is a **constraint-removal** bead: it deletes an enforced range check. A wrongly deleted
constraint *under-constrains* the circuit, which in a proving system is a correctness bug, not a
performance bug. So the record leads with the soundness argument and its side conditions, then
reports what was actually measured.

Environment, stated up front because it bounds every claim:

| capability | available | consequence |
|---|---|---|
| noir source at HEAD | yes (clone @ `e22cd89b`) | full source analysis |
| rustc `1.89.0` (the workspace pin) | yes — installs into a writable `RUSTUP_HOME` | `noirc_evaluator` builds; **`nargo` release-built from both the patched and unpatched trees** |
| upstream issue #7161 body | **yes** — fetched; 0 comments, still OPEN | §1 quotes it verbatim rather than paraphrasing §4.1 |
| corpus recompile + ACIR-opcode counts | **yes** — 519 packages, both compilers | §6.4 is measured |
| `bb` | **no** | **no gate counts** — nothing here is a gate measurement |
| AST fuzzer campaign | **not run** | §8 |
| noir git history | **no** — depth-1 clone | cannot check whether this was attempted upstream before |

So §2–§5 are source-derived facts plus a written argument; §6 is measurement from this box (and
therefore non-canonical); §7 is the reading of that measurement and §8 is what remains. The soundness
argument in §3 is an argument backed by guard tests and a differential corpus run — **not** a formal
proof and **not** externally audited.

## 1. The ask

Upstream issue [#7161](https://github.com/noir-lang/noir/issues/7161), TomAFrench, 2025-01-23, OPEN,
no labels, **zero comments** — the body in full:

> Consider the program
>
> ```text
> acir(inline) fn main f0 {
>         b0(v0: u32):
>           v2 = add v0, u32 1
>           v3 = add v1, u32 1
>           return v3
>         }
> ```
>
> I'm fairly certain (check this) that if we ran ACIRgen on this we'd end up with two separate range
> checks for `v2` and `v3`. However it's impossible for `v2` to fail its overflow check without `v3`
> also doing so. We can then skip the overflow check for `v2` and rely on `v3`'s.
>
> We could then add a pass which removes these unnecessary overflow checks to make more of the
> circuit's arithmetic unchecked.
>
> Note that this likely would change how certain tests fail so we may want to have a brillig overflow
> check inserted here. To be determined by whoever works on this.

Two notes on the body. `v3 = add v1, u32 1` is a **typo** for `add v2, u32 1` — `v1` is never
defined. And the issue is written as an *investigation* ("I'm fairly certain (check this)"), leaving
two open questions: does ACIRgen really emit both checks, and what to do about Brillig. §2 and §4
answer both.

## 2. What HEAD emits, and what an elision saves

**The overflow check is not an SSA instruction.** It is emitted at ACIR-gen time from the
`unchecked: false` flag on the SSA `Binary`: `acir/mod.rs:797-802` routes every unsigned add/sub/mul
into `check_unsigned_overflow`, which is exactly one call to `range_constrain_var(result, bit_size,
…)` (`acir/mod.rs:806-821`). An `unchecked` op returns early at `mod.rs:817` and emits **nothing**.
The issue's premise is confirmed.

What one deleted check costs today, from `acir_context/mod.rs:1126-1160`:

1. `mul_var(variable, predicate)` — under the default predicate `1` this folds away;
2. `get_or_create_witness_var(predicate_range)` — **forces the running expression into a fresh
   witness** (`acir_context/mod.rs:222-243`; `mark_variables_equivalent` makes the original var
   resolve to that witness from then on);
3. `range_constraint(witness, bit_size)` — the `BLACKBOX::RANGE` opcode, i.e. one `bit_size`-bit
   decomposition;
4. an assertion-payload attachment for the `"attempt to add with overflow"` message.

Item 3 is the saving the issue is after. Item 2 turns out to be a *second* saving rather than the
hidden cost §7 originally feared — measured in §6.2.

One further consequence of `range_constrain_var` shapes the design: it returns `predicate_range`,
i.e. `predicate · result`, **not** `result`. Under a variable predicate a checked add's SSA result
maps to `predicate · result` in ACIR, so dropping its check would change the *value* handed
downstream, not merely the constraint. §5 condition (c) exists for this.

## 3. The soundness argument, and the four ways it breaks

Write ⟦v⟧ for the field element ACIR assigns to SSA value `v`, and N for the unsigned type width.

**The core claim.** For a chain `x₁ = a₀ + b₁`, `x₂ = x₁ + b₂`, …, `xₖ = x_{k−1} + bₖ` of *unsigned
additions in an ACIR function*, where every `bᵢ` and `a₀` is known to lie in `[0, 2^N)` and the whole
chain stays below the modulus, ⟦x₁⟧ ≤ ⟦x₂⟧ ≤ … ≤ ⟦xₖ⟧ **as integers**. Hence ⟦xₖ⟧ < 2^N implies
⟦xᵢ⟧ < 2^N for every i, and the checks on x₁…x_{k−1} are implied by the check on xₖ.

This works because **ACIR add is non-reducing field arithmetic**: `acir/mod.rs:766` is a plain
`add_var` with the range check bolted on afterwards. It is not modular arithmetic at 2^N, so an
"overflowing" add does not wrap back into range — it produces the honest integer sum, which is then
rejected. The SSA interpreter states the same model explicitly (`ssa/interpreter/mod.rs:1336-1368`:
ACIR "extends in the field", Brillig "wraps to the bit size").

The claim breaks in exactly four ways, each mapping to a side condition in §5:

**(a) Non-monotone successors.** `xᵢ − c` is *decreasing* in `xᵢ`, and `xᵢ · c` is non-decreasing
only for `c ≥ 1` — for `c = 0` the product is 0 whatever `xᵢ` is. So a `sub` or `mul` successor does
**not** imply its predecessor's check. v1 restricts both ends of the chain to `add`. (`mul` by a
provably-nonzero operand is a sound extension; it is not implemented and not claimed here.)

**(b) Escape.** Once the check is dropped ⟦xᵢ⟧ may exceed 2^N, and every *other* consumer would then
see an out-of-range value: `lt` compares at the type width, `div`/`mod`/`truncate` euclidean-divide at
the type width, an array index is bounds-checked against the array length, and `return` hands it to
the ABI. So the chain link must be the value's **only** consumer.

Upstream has already codified this invariant, which is the strongest single piece of evidence that
the optimization fits HEAD's model rather than fighting it. `dfg/simplify/binary.rs:323-359`
(`can_simplify_arithmetic_identity`) documents: *"The only values that can hold a magnitude outside
their type's range are results of unchecked `add`/`sub`/`mul`. Every other value … is guaranteed to
fit its type"*, and refuses to rewrite `x + 0 → x` when `x` is an unchecked-arithmetic result. And
`dfg.rs:574-619` (`get_value_max_num_bits`) already returns, **for ACIR only**, the widened bound
`max(lhs, rhs) + 1` for an unchecked `add` and `lhs + rhs` for an unchecked `mul`, saturating at the
field width, with a comment spelling out that an unchecked result "may exceed the operands' type
width until a later range check or truncation brings it back". The elision does not invent an
invariant; it enlarges the population of values that already carry it.

**(c) Predicates.** After `flatten_cfg` an instruction's effective predicate is the most recent
`EnableSideEffectsIf`, and a checked unsigned add is predicate-consuming (`instruction.rs:931-960`,
`requires_acir_gen_predicate`). If the elided check ran under predicate 1 and the retained one under
a predicate that can be 0, the retained check is vacuous and the overflow escapes entirely — an
accepted execution that should have been rejected. Compounding it, §2's `predicate · result` return
value means the elision would also change the value. v1 therefore only fires while the predicate is
the literal `u1 1` (idiom copied from `make_constrain_not_equal.rs:83-92`).

**(d) Modular wraparound.** The integer/field correspondence, and with it monotonicity, holds only
while every intermediate stays below the modulus p. `FieldElement::max_num_bits()` is
`MODULUS_BIT_SIZE` = 254 for bn254 (`acir_field/src/field_element.rs:394`, smoke-tested at `:866`),
so p ≥ 2^253. v1 requires the chain's tracked bound to stay strictly below 254, i.e. ≤ 253, which
forces both operands to ≤ 252 bits and the sum below 2^253 ≤ p. This also disposes of underflowed
values for free: `get_value_max_num_bits` reports an unchecked `sub` as full field width, so any
chain fed by one immediately trips the bound and is refused.

## 4. The Brillig question the issue leaves open — answered: insert nothing, gate on the runtime

Brillig executes unsigned arithmetic in **fixed-width registers that wrap**
(`ssa/interpreter/mod.rs:1338-1352`: `wrapping_add`/`wrapping_sub`/`wrapping_mul` then truncate to
the bit size). So the elision is unsound in Brillig for a *different* reason than in ACIR: an
unchecked intermediate would not merely go unchecked, it would **wrap to a small value**, and the
retained later check would then see something perfectly in range. A program that must fail would
silently succeed with a wrong answer.

The fix is therefore **not** to insert a Brillig-side check — it is to not apply the transform in
Brillig at all, leaving Brillig's checks exactly where they are. Direct precedent one file over:
`check_u128_mul_overflow.rs:36-39` opens with `if !self.runtime().is_acir() { return; }` and its
module doc says *"In Brillig an overflow check is automatically performed on unsigned binary
operations so this SSA pass has no effect for Brillig functions."* The same sentence applies here.

**What that leaves observable** is the "changes how certain tests fail" the issue predicts: for an
input that overflows an intermediate, ACIR now fails at the *later* op while Brillig still fails at
the *earlier* one. Two facts bound how much that matters, and §6.3 measures it:

- The differential ACIR-vs-Brillig fuzzer **already tolerates this class of divergence**.
  `tooling/ast_fuzzer/src/compare/compiled.rs:205-235` treats two failures as equivalent when both
  messages merely *contain* `"overflow"`, with a dedicated arm for `"add with overflow"`, under the
  standing comment *"In Brillig we have constraints protecting overflows, while in ACIR we have
  checked multiplication unless we know its safe"*. The harness was built expecting ACIR and Brillig
  to disagree about *where* an overflow is caught.
- Because v1 keeps the chain monotone (§3a) and inside the field (§3d), the *outcome* — fails vs
  succeeds — is preserved on both runtimes. Only the failure site moves.

## 5. The transform

Implemented as a **second phase inside `checked_to_unchecked`**, not as a new pass in the pass list.
That is deliberate: §4.2 of the program record records PR #11580 being closed with *"doesn't have
enough optimizations to warrant the upkeep"*, so a new `SsaPass` entry is rent this change should not
have to pay. The phase runs at the existing `checked_to_unchecked` slot (`ssa/mod.rs:396`), after
flattening and after `remove_enable_side_effects` has minimized predicate coverage.

Phase 1 (analysis, `&self`): count every value's uses across instructions **and terminators**; then
walk each block in order, tracking whether the predicate is the literal `1` and maintaining the
checked adds seen so far in the block together with the width their result could reach if their check
were dropped. On reaching a checked unsigned `add`, an operand's defining instruction is marked
elidable iff:

| # | condition | closes §3 case |
|---|---|---|
| a | the function's runtime is ACIR | §4 |
| b | both the elided op and the consuming op are unsigned `add` with `unchecked: false` | (a) |
| c | the predicate is the literal `u1 1` at both | (c) |
| d | the operand's **total** use count across the function is exactly 1 | (b) |
| e | both are in the same block (state resets per block) | (b)/(c) |
| f | the consuming add's tracked bound `max(lhs_bits, rhs_bits) + 1` is `< FieldElement::max_num_bits()` | (d) |

Chains compose without an extra rule: an elided op is never the *last* op of a chain, because it is
only elided when a later checked add is looking at it. Every chain therefore terminates at a retained
check and §3's induction carries down it. Bit growth composes the same way — an elided operand
contributes its widened bound rather than the type width, which is what (f) tests.

Phase 2 (rewrite): one `simple_optimization` walk flipping the marked instructions to
`operator.into_unchecked()`.

Compile-time shape: one linear use-count scan plus one linear block walk. The one non-linear risk is
that `get_value_max_num_bits` recurses through cast/unchecked-arithmetic chains and is not memoized
upstream (HEAD carries a regression test, `dfg.rs:1213-1230`, only for *termination* on a 128-deep
chain, not for cost). The phase memoizes it locally. Measured effect in §6.4: none.

## 6. What was measured

Patch: `compiler/noirc_evaluator/src/ssa/opt/checked_to_unchecked.rs` (+~150 lines, +6 tests) plus one
updated snapshot in `ssa/opt/hint.rs`. **It is not carried in this repo.** All numbers below are from
this box and are therefore **non-canonical**; ACIR-opcode counts are the program's *iteration proxy*
(§5.1), never a gate claim.

**How they were generated**, so they can be reproduced or discarded: two release `nargo` binaries
built from the same clone at `e22cd89b`, one with the patch applied and one from a `git stash`ed
tree; ACIR opcode counts from `nargo info --json --force --program-dir <pkg>`, summing `opcodes`
over every entry point; failure differentials from `nargo execute --force [--force-brillig]
--program-dir <pkg>` with stdout+stderr compared byte-for-byte. **`--force` is load-bearing** — a
first pass of this comparison was invalid because `nargo` silently reuses a cached artifact from
`<pkg>/target/` and the second compiler never ran.

### 6.1 Unit tests, clippy, fmt

`cargo test -p noirc_evaluator` — **1890 passed, 0 failed, 15 ignored**. `cargo clippy -p
noirc_evaluator --all-targets` — clean. `cargo fmt -p noirc_evaluator -- --check` — clean.

Six tests added; the two positive ones assert `unchecked_add` where HEAD emits plain `add` (HEAD's
own committed `hint.rs` snapshot, below, is the direct evidence that it does), the four negative ones
are guards:

| test | pins |
|---|---|
| `elides_overflow_check_dominated_by_a_later_one` | the issue's own two-link chain (with `v1` fixed to the intended `v2`) |
| `elides_every_intermediate_check_of_an_addition_chain` | a 3-link chain: first two become `unchecked_add`, the last stays checked |
| `does_not_elide_overflow_check_when_the_intermediate_has_another_use` | §3(b) — an extra `lt` consumer blocks it |
| `does_not_elide_overflow_check_dominated_by_a_subtraction` | §3(a) — a `sub` successor is not monotone |
| `does_not_elide_overflow_check_under_a_variable_predicate` | §3(c) — `enable_side_effects v1` blocks it |
| `does_not_elide_overflow_check_in_brillig` | §4 — `brillig(inline)` is untouched |

Exactly one existing snapshot moved: `ssa::opt::hint::tests::test_black_box_hint`, which runs
`run_all_passes` over a four-add chain and now yields three `unchecked_add`s with the final `add`
retained. That is the intended behaviour.

### 6.2 What the ACIR actually becomes

For `fn main(x, y, z, w: u32) -> pub u32 { x + y + z + w }`, `--print-acir`, baseline vs patched:

```text
baseline (11 opcodes)                     patched (7 opcodes)
RANGE w0..w3, bits: 32                    RANGE w0..w3, bits: 32
ASSERT w5 = w0 + w1                       ASSERT w5 = w0 + w1 + w2 + w3
RANGE w5, bits: 32   // add with overflow  RANGE w5, bits: 32  // add with overflow
ASSERT w6 = w2 + w5                       ASSERT w4 = w5
RANGE w6, bits: 32   // add with overflow
ASSERT w7 = w3 + w6
RANGE w7, bits: 32   // add with overflow
ASSERT w4 = w7
```

Two `RANGE(32)` **and** two `AssertZero` opcodes disappear: §2 item 2 means each deleted check also
deletes a witness cut, so the intermediate assertions merge into one wide linear expression. On
synthetic unrolled accumulators (`sum += xs[i]`) the effect scales:

| fixture | baseline ACIR opcodes | patched | delta |
|---|---:|---:|---:|
| `x + y + z + w` (u32) | 11 | 7 | −36% |
| 8-term accumulator loop | 23 | 11 | −52% |
| 32-term accumulator loop | 95 | 35 | −63% |
| 128-term accumulator loop | 383 | 131 | −66% |

**This overstates the gate win, and the record must not be read as claiming otherwise.** The 32-term
case collapses to a single `AssertZero` carrying 32 linear terms; the backend still has to
arithmetise that. Relevant drift found while checking this: at HEAD the ACVM pipeline is
`GeneralOptimizer` → `RangeOptimizer` → `CommonSubexpressionOptimizer`
(`acvm-repo/acvm/src/compiler/mod.rs:1-12`) and the identifier `ExpressionWidth` **no longer exists
anywhere in the tree** — the CSAT expression-width transformer that `noir-optimization-program.md`
§3.3 lists as the fourth stage is gone. So nothing splits a wide expression before `bb` sees it, and
the ACIR-opcode delta and the gate delta are further apart here than usual. Captured as a follow-up
against §3.3.

### 6.3 Behavioural differential

- **`test_programs/execution_failure`, all 72 packages**, `nargo execute --force`, baseline vs
  patched: **byte-identical output** — same messages, same spans, same call stacks, same diagnostics.
  Repeated with `--force-brillig`: **also byte-identical**. (An earlier run of this comparison was
  invalid because `nargo` reuses a cached artifact unless `--force` is passed; the numbers here are
  from the corrected run.)
- On a purpose-built overflowing fixture (`x + y + z + w` with `x = 2³²−1, y = 1`), all four of
  {baseline, patched} × {ACIR, `--force-brillig`} fail with the identical message
  `Assertion failed: attempt to add with overflow`, the same line and the same call stack. The only
  difference is the highlighted **span**, which widens from `x + y` to `x + y + z + w`. That is the
  entire observable footprint of the failure-site change on this fixture.

### 6.4 Corpus

519 packages — noir `test_programs/benchmarks` (9) + `test_programs/execution_success` (509) + sparq
`zk/compose` (1) — compiled with both binaries, `nargo info --json --force`. **`zk/xpath` is not in
the corpus**: per this program record's own header note it was externalized to `sparq-org/noir_XPath`
and the in-tree directory now holds only `differential/`, `scripts/` and `tests/` with no
`Nargo.toml`, so §5.2's three `probe_xpath_*` figures have no in-repo source to reproduce from.

| | value |
|---|---:|
| packages compared | 519 (0 errors) |
| packages whose ACIR opcode count changed | **5** |
| **regressions** | **0** |
| total ACIR opcodes, baseline | 251 925 |
| total ACIR opcodes, patched | 251 852 |
| delta | **−73 (−0.029%)** |

| package | baseline | patched | delta |
|---|---:|---:|---:|
| `hint_black_box` | 27 | 8 | −19 |
| `compose` (sparq `zk/compose`) | 120 330 | 120 312 | −18 |
| `fold_complex_outputs` | 100 | 84 | −16 |
| `a_6_array` | 431 | 417 | −14 |
| `simple_bitwise` | 16 | 10 | −6 |

Compile time on `sha512_100_bytes` was also compared (3 runs each, release `nargo`, baseline vs
patched). The raw seconds are **deliberately not recorded here**: a wall-clock reading taken on this
box is non-canonical, and the two run sets overlap well inside their own spread, so the only claim
the measurement supports is a qualitative one — **no compile-time regression was observed**, i.e.
the #12927 O(n²) lesson is not being repeated. A quantitative compile-time claim would need a quiet box
and the canonical harness; neither was available, and nothing here should be read as one.

## 7. Reading the result honestly

The transform does what the issue asks, is correct as far as every check available here can tell, and
**no regressions were observed** in what was actually run — 519 corpus packages at ACIR-opcode level
and the 72-package `execution_failure` differential on both runtimes. That is finite evidence, not a
soundness proof: there is no formal proof and no external audit, the AST fuzzer was not run, and
`noir_test_failure` / the `compile_success_*` suites were not differenced (§8). On real code the win
is also small: five packages out of 519, −0.029% of corpus ACIR opcodes, and even that is measured on
the proxy metric rather than on gates.

Why so few? Condition (d) is the binding one: real Noir arithmetic rarely produces a chain whose
intermediates have exactly one consumer *and* are unsigned *and* are all additions. The synthetic
accumulator is the shape that hits, and it is not common in the corpus — the corpus's heavy hitters
are hashes and field arithmetic, which never enter this pass.

**Recommendation.** On this evidence the change should **not** go up as a standalone PR. §4.2's
maintenance-rent bar (PR #11580: *"doesn't have enough optimizations to warrant the upkeep"*) is a
real filter, and −0.029% of a proxy metric will not clear it. Two ways it could become worth
proposing, in order of expected value:

1. **Measure gates.** If the five hits — particularly `hint_black_box` at 27→8 and sparq's own
   `compose` — turn into a visible `bb gates` reduction, the picture changes, because the elision
   removes 32-bit range decompositions, which are among the more expensive things in an UltraHonk
   circuit. This is the single blocking measurement and it needs a box with `bb`.
2. **Fold it into the range-analysis work rather than shipping it alone.** The natural generalisations
   — a dominating *explicit* `RangeCheck` rather than only a later checked add, `mul` by a
   provably-nonzero operand, and cross-block dominance — are exactly `sq-rxir8` (#9463) and
   `sq-jj3ne` (#9429). Those beads already carry the dependency edges (§10.4), and a single pass that
   subsumes all three carries its upkeep far better than this one does alone.

The null-result discipline of `sq-seust` applies in spirit: an optimization that measures small is
information, and the honest move is to say so rather than to ship it because it was built.

## 8. What remains

Everything in `noir-optimization-program.md` §10.2 applies. Done in this session: unit tests, clippy,
fmt, ACIR-opcode corpus regression, `execution_failure` differential on both runtimes,
`sha512_100_bytes` compile time. Remaining:

1. **`bb gates -s ultra_honk` before/after** on the five changed packages and on the accumulator
   fixtures. Blocking — see §7.1.
2. **AST fuzzer** (`tooling/ast_fuzzer`) for a sustained run, since §4 leans on its equivalence rules.
3. `noir_test_failure` and the `compile_success_*` suites (only `execution_failure` was differenced).
4. A re-read of the #7161 thread for new comments, then — only if step 1 justifies it — a draft PR per
   §6 of the program record, @jeswr-gated, with the §7 recommendation stated in the body rather than
   hidden.

## 9. Adjacent findings

- **`required_bit_size` in `checked_to_unchecked.rs:126-171` diverges from
  `DataFlowGraph::get_value_max_num_bits`.** Its doc claims the two are "almost the same", but the
  local helper falls through to the static type width for an unchecked `add`/`sub`/`mul` while the
  dfg method returns the widened ACIR bound. For the `<`/`≤` tests in the Add/Mul arms the static
  width is the *conservative* side, so this is not a live bug there — but the `max_lhs_bits == 1 ||
  max_rhs_bits == 1` special case in the Mul arm reads the other way, and an `unchecked_add u1` in
  ACIR is the field element 2, which the dfg method reports as 2 bits and the local helper as 1.
  **Unverified**: no reproducer was constructed and no bug is claimed. It is regardless a hard
  ordering constraint for this bead — the elision must run *after* `checked_to_unchecked`, never
  before, or it would read the stale narrower bound for values it widened. Captured as follow-up.
- **`noir-optimization-program.md` §3.3 is stale** on the ACVM pipeline (§6.2). Captured as follow-up.
- The issue body's `v3 = add v1, u32 1` typo deserves a one-line correction when the thread is next
  touched — but §4.2 notes typo-scale contributions are closed on sight, so it rides along with a
  substantive comment or not at all.

## 10. Honest summary

The issue's premise checks out against HEAD, the Brillig question it leaves open has a clean answer
with in-tree precedent, and the transform is implemented with each of its four soundness side
conditions pinned by a guard test. The full `noirc_evaluator` suite is green, the 72-package failure
corpus is byte-identical on both runtimes, and 519 packages recompile with zero regressions.

What the measurement says is that the optimization is **real but small**: −0.029% ACIR opcodes across
the corpus, concentrated in five packages, on a proxy metric. Whether that is worth upstreaming is a
gate-count question this box cannot answer, and until it is answered the change stays here.
