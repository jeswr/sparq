# Noir compiler optimisation — gap-analysis / new-opportunities pass (`sq-mtolx`)

<!-- [OPUS-5] sq-mtolx: P3 of the paper program (research/noir-optimization-paper.md
     §5.3). Code-level examination of the SSA/ACIR/ACVM stages the sq-uuvac program
     had NOT yet examined at implementation level, read against noir-lang/noir HEAD
     e22cd89b. DESIGN/FINDINGS-ONLY — no compiler patch, no upstream PR, and (see §0)
     NO measurement was possible in this environment, so NO measured-PR child bead is
     spawned. §6 carries candidate SPECS, each gated on a named kill-test. -->

> 🤖 **SPARQ agent** — gap-analysis record for bead `sq-mtolx`, child of the paper
> bead `sq-1j5ow`, program epic `sq-uuvac`. Model: Opus 5. This is a
> **findings-only** record: it examines code, it does not change it, and it opens no
> upstream PR. Read `research/noir-optimization-paper.md` §2.2 **through §1 below** —
> that gap table has stale entries and this record corrects them.

## 0. What this bead was asked for, and what it could actually deliver (read first)

The bead asks for three things. It delivered one of them, and the honest accounting
matters more than the findings:

| asked | delivered |
|---|---|
| Examine, at code level, the passes not yet implementation-examined | **YES** — six stage groups against HEAD `e22cd89b`, §3, with `file:line` evidence throughout |
| Produce **measured** candidates (a reproduced `bb gates` win via the P1 harness) | **NO — impossible here.** No `bb` and no `nargo` on this box; and the P1 harness bead `sq-i50o4` **has not landed** (there is no committed noir-fixture harness in `scripts/`) |
| For each candidate reproducing a gate win, spawn a measured-PR child bead under `sq-uuvac` | **ZERO spawned**, correctly — the promotion gate was never met by any candidate |

**No number in this record is measured.** Nothing was compiled, executed, or
benchmarked. Every cost statement is a *structural* count of emitted SSA
instructions, euclidean divisions, or range-check widths read off the source — a
necessary but **not sufficient** condition for a backend-gate win. The program's own
rule 7 (`noir-optimization-program.md` §6) is "numbers only from actual runs; if a win
doesn't reproduce on `bb gates`, the PR doesn't go up". Spawning implementation beads
for unmeasured hypotheses would launder that rule, so §6 lists candidate **specs**
that stay un-beaded until their kill-test runs.

Two further limits, stated so no reader over-reads this record:

- **No upstream issue thread was read.** This run had no GitHub API access. Every
  framing of #10438/#10439/#6631/#6629/#5501/#10109 below is the *paraphrase carried
  in the prior program records*, not the live thread. Re-checking each thread is a
  pre-flight obligation on any bead spawned from §6 — the same obligation §10.5–10.7
  of the program record already carry.
- **The clone was shallow**, so `git log`/`git blame` were unavailable; dating and
  "did upstream already fix this" evidence comes from `CHANGELOG.md` and from the
  presence/absence of `regression_*` fixtures, which is weaker.

## 1. Corrections to the paper's §2.2 gap table (it is stale; do not cite it as-is)

The gap table in `noir-optimization-paper.md` §2.2 was written against an older HEAD.
Five of its rows are wrong at `e22cd89b`, and two of the wrongs would embarrass the
paper if printed:

| §2.2 claim | Status at HEAD `e22cd89b` |
|---|---|
| `ssa/opt/flatten_cfg/value_merger.rs` | **Path moved** → `ssa/ir/dfg/simplify/value_merger.rs`. Not cosmetic: the merger is no longer a flattening-private helper, it is a DFG-level utility consumed by `remove_if_else.rs:109` |
| `try_merge_only_changed_indices` is the un-examined site | **The function does not exist.** It was *deliberately deleted* by upstream PR #8142; `remove_if_else.rs:651-654` says so in-tree and points at issue **#8145** ("investigate if this can be brought back") — a different, later ticket than #5501 |
| "precedent PR #11512 merged" for merging only accessed indices | #11512 landed as `array_set_window_optimization.rs`, which **does something else**: it rewrites `array_set`→`make_array` so the merger can short-circuit one lane's multiply. `CHANGELOG.md:67` ("Only merge modified array indices") **over-claims relative to the code it shipped**. Cite the pass doc (`array_set_window_optimization.rs:14-17`), never the changelog line. **Not actionable as a change** — a released changelog entry is historical record upstream does not rewrite; the remedy is the citation rule stated here, discharged in this record, so nothing is tracked |
| ACIR-gen comparison lowering (`acir_context/mod.rs:1163-1228`) listed as un-examined | **Already examined** — `sq-jfkwk`, null result, `noir-comparison-lowering-10159.md` (program record §10.7) |
| Quadratic ACIR expression growth (#4629) listed as un-examined | **Already examined** — `sq-felqr` design note, `noir-acir-expression-growth-4629.md` (program record §10.6) |

Also worth correcting in the paper's framing: §2.2 describes `mem2reg` as a candidate
for "cross-block promotion after flattening". `mem2reg` is **already** a full
Cytron-style IDF construction and is cross-block by design (`mem2reg.rs:1-14`), and
after flattening every ACIR function is a *single block* (`flatten_cfg.rs:14-16`), so
the post-flatten run has no cross-block work left to do on the constrained path.

## 2. The cross-cutting result: most of the un-examined surface cannot pay in gates

This is the strongest finding of the pass, it is verifiable without a proving backend,
and it explains *why* the program's seven PRs all clustered in one neighbourhood: they
went where the gates are. Four load-bearing facts, each quoted from HEAD:

1. **A whole Brillig function is one ACIR opcode.** `Opcode::BrilligCall` carries only
   `{id, inputs, outputs, predicate}`; the bytecode is fetched by the executor
   (`acvm-repo/acir/src/circuit/opcodes.rs:138-152`). Brillig bytecode is provably
   outside the constraint system, so **any unconstrained-path optimisation is a
   witness-generation win, not a proving-cost win.**
2. **ACIR has no loops.** Every ACIR loop must be fully unrolled or compilation fails
   with `RuntimeError::UnknownLoopBound` (`unrolling.rs:30-31`, `:45-47`, `:190-213`),
   and flattening asserts it (`flatten_cfg.rs:222-229`). After the Unrolling pass there
   is no loop left to unswitch and no induction variable left to eliminate.
3. **On the constrained path, memory promotion is a legalization obligation, not an
   optimisation.** A surviving `Allocate` is `RuntimeError::UnknownReference` and a
   surviving `Load`/`Store` is an `unreachable!` (`acir/mod.rs:531-541`). A mem2reg
   miss produces a compile error or an ICE — **never a larger circuit.**
4. **The compiler has no ACIR cost model at all.** `ir/target_cost.rs:1-9`: *"Brillig
   target cost estimation … If ACIR cost estimation is needed in the future, it can be
   added here alongside the Brillig estimates."* Every profitability heuristic in the
   unrolling and basic-conditional passes is denominated in Brillig opcodes.

Consequence for the paper: the honest headline of the "new opportunities" section is
not a list of wins. It is that **the un-examined stages are largely un-examined
because they sit on the unconstrained path or on a legalization obligation**, and that
the constrained-path opportunity that survives is concentrated in exactly one place —
worst-case-always *expansion* of an IR op into constraints, where a provable value
bound would narrow the expansion. That is the same class as every prior measured win
in this program, which is corroboration, not coincidence.

## 3. Stage-by-stage findings

### 3.1 Loops — `loop_invariant.rs` (LICM, #10439/#10438), `unrolling.rs` (#6631, #6629)

**LICM.** The issues' premise is real: LICM declines many loop-invariant `ArrayGet`s.
An `ArrayGet` hoists only if (P1) the index is a numeric constant below the
semi-flattened length (`dfg.rs:839-849` `is_safe_index` requires
`get_numeric_constant(index).is_some()`, so *every* dynamic index fails), or (P2) the
index is **the exact `ValueId`** of an enclosing loop's induction variable with a
constant bound (`loop_invariant.rs:743-768`) — matching `arr[i]` but not `arr[i+1]`,
`arr[i*2]` or `arr[cast i]`; otherwise (P3) it needs
`!is_control_dependent && does_execute && !is_impure` (`:257-259`).

But every P3 blocker is **load-bearing**: `does_execute` and `is_control_dependent`
exist to stop LICM manufacturing an out-of-bounds failure in an iteration or branch
the source never executes — pinned by the pass's own test at `:4343-4375` — and
`is_impure` preserves failure ordering (upstream #8724). Widening `is_safe_index`
itself is worse than a motion: it is consumed by ACIR-gen (`acir/arrays.rs:497-511`)
and by OOB-check insertion (`die/array_oob_checks.rs:161-175`), so relaxing it
**removes emitted constraints** — an under-constraining risk, not a perf knob.

The one sound residual is narrow: widen the *in-bounds prover* in
`can_be_hoisted_from_loop_bounds` from exact-induction-variable identity to affine
derivations, comparing against the semi-flattened length. An in-bounds `ArrayGet` is
total, so it needs no execution or ordering side-condition. Two dampeners: array
indices are validated to be exactly `u32` (`validation/mod.rs:299-308`), which kills
the obvious bit-width transplant from the program's prior wins; and LICM runs
pre-unroll (`mod.rs:284` vs `:290`), so post-unroll CSE may already collect the ACIR
win. Upstream has not fixed this — but every post-issue LICM changelog entry
(#11771, #12301, #12496, #12665, #12797, #12981, #12969) tightens rather than loosens,
and there is no `regression_10438`/`10439` fixture.

**Unrolling.** Both issues are **subsumed or wrong-path** on the constrained side, by
fact 2 of §2. The unroller substitutes the induction variable with a numeric constant
per iteration and constant-folds on insertion (`unrolling.rs:1309-1311`, `:2351-2369`);
its own snapshots show `add v, i` already emitted as `add v4, u32 1`
(`:3440-3463`). The bound-derived checked→unchecked rewrite — the one classical
strength-reduction effect with a gate consequence — **already runs pre-unroll** in
`loop_invariant/simplify.rs:306-349`. Unswitching does not exist anywhere
(`grep -i unswitch` → 0 hits), but after full unrolling and flattening the ACIR
residual is not the branch bodies (both arms are always emitted) — it is only the
`(N−1)×k` redundant merges, i.e. a post-unroll diamond-merge peephole, a different
transformation from the one the issue titles name.

Genuine but non-gate findings in the cost model, all Brillig-only: `iterations` is
computed as `upper − lower` ignoring both the step and the `LoopBoundKind`
(`:1872-1879`); `callee_costs` is passed as an **empty map** by the main pipeline
(`:121-127`), making the callee-weight refinement dead outside `preprocess_fns.rs`;
the size-revert is whole-function granularity and compares raw `num_instructions()`
against a decision made on Brillig-weighted `cost()`. One ACIR-path robustness gap:
`RUNAWAY_UNROLL_LIMIT` applies **only** when termination is unproven (`:1238-1250`), so
a provably-terminating `for i in 0..4_000_000_000` in ACIR hangs with no diagnostic.
Two adjacent diagnostics gaps in the same file: the ACIR bail-outs at `:392` and
`:419` discard the source location. **Disposition:** these are upstream correctness /
diagnostics defects, not optimisation candidates — they move no gate count, so §6's
measurement gate does not apply and they are deliberately absent from §4 and §6. They
*are* actionable, and are captured as a `self-improvement` follow-up issue filed with
this PR for upstream reporting — titled *"Noir upstream: ACIR runaway-unroll hang + two
location-discarding bail-outs (unrolling.rs)"* and carrying the durable dedupe marker
**`noir-acir-unroll-2026-07-28`**, which is the handle to search the tracker for. They
get no `sq-` bead under `sq-uuvac`, whose remit is measured optimisation PRs. No further
work on them is pending in this record.

### 3.2 Array/memory — `value_merger.rs`, `array_set_window_optimization.rs`, `mutable_array_set.rs`

Beyond the §1 corrections: there is now **no cheap path** in the merger at all. The
only early-out is pointer identity (`value_merger.rs:99-101`); everything else merges
every flattened slot, ~7 SSA instructions per element (2 `ArrayGet` + `Cast`×2 +
`unchecked_mul`×2 + `unchecked_add`) plus a `MakeArray`, recursing into nested element
types (`:218`) with **no size cap** — an `[[u32;32];32]` merges 1024 lanes. Upstream
pins the un-optimised behaviour as *expected* in a snapshot test,
`remove_if_else.rs:617` `merges_all_indices_even_if_they_did_not_change`.

Two distinct rewrites hide behind "#5501", and conflating them is the hazard:

- **Rewrite A — partial merged array** (what #8142 deleted, what #8145 tracks).
  **Soundness-blocked.** It requires proving every un-merged index is never observed
  downstream. The IR cannot discharge that: one dynamic-index read observes all slots;
  values escape via `Call`/`Return`/`Store`; and `alias_analysis.rs` is
  flow-insensitive Steensgaard over *references*, the wrong granularity — it cannot say
  "slot 3 of `v` is never read".
- **Rewrite B — push `ArrayGet` through `IfElse`** (the literal #5501 title).
  **Sound by construction** — it rewrites a *read* into `c*a_k + !c*b_k`, which is
  exactly what `merge_numeric_values` computes for that lane, and never weakens a
  value, so it carries no alias obligation. **Verified absent at HEAD**:
  `try_optimize_array_get_from_previous_instructions` matches only `ArraySet` and
  `MakeArray`, with `_ => ()` at `array_get.rs:340`, and `Instruction::IfElse` appears
  nowhere in `opt/array_get.rs`. It needs a profitability guard (it is a pessimisation
  when the merged array is read at many indices) and, critically, it may already be
  achieved end-to-end by the existing fold-then-DIE tail (`mod.rs:349`, `:388`, `:390`)
  — which is the finding most likely to kill it, and the one that cannot be settled
  without running the pipeline.

`array_set_window_optimization.rs` carries a hard cap,
`MAX_ARRAY_SEMI_FLATTENED_LENGTH = 64` (`:71-72`), motivated as a compile-time
guard (`:48-50`). For an array-heavy RDF/XPath workload a 128-byte string buffer is
silently declined — pure parameter tuning, trivially sound, cheapest thing in this
record to test. `mutable_array_set.rs` is conservative-sound throughout (direct-only
nesting at `:185`); no opportunity found, and its two
`assert_pass_does_not_affect_execution` tests are a materially stronger check than the
snapshot-only style used elsewhere.

### 3.3 Memory / conditionals — `mem2reg.rs`, `basic_conditional.rs`

`basic_conditional.rs` is hard-gated Brillig-only (`:280-284`), costed in Brillig
opcodes by a module that states no ACIR cost model exists. Per §2 fact 1, **a win
there cannot move a gate count.** Its declined shapes are real (direct-edge
terminator args unsupported at `:149-153`; a profitability threshold that can never
fire when a single array-typed exit parameter prices the merge at 20) but all on the
wrong path.

`mem2reg.rs` is not the block-local pass the paper's §2.2 assumed (§1). Its real
limitation is that it has **no alias analysis at all** — eligibility is syntactic
escape analysis (`:470-543`) — while the repo's 2964-line Steensgaard analysis is
consumed only by `load_store_forwarding`, which is block-local by construction
(`:8-12`). The empty quadrant is therefore *cross-block × alias-aware*, and §2 fact 3
makes it **vacuous for ACIR**: post-flatten there is one block, and pre-flatten a miss
is a compile error rather than a bigger circuit. The one annotated un-taken case is
the "no store in the declaration block" rule (`:481-487`, a workaround for #11482),
which is broader than the orphan case its test covers — but fixing it means
introducing a real `undef` plus a reaching-definitions check, and per §2 fact 3 it is
not a gate story. Also noted: the ordering comment at `mod.rs:375-380` is **stale**
(it describes a predecessor-unification scheme that IDF placement replaced).
**Disposition:** documentation-only and upstream-side; upstream convention closes
typo-scale PRs, so it is deliberately **not tracked** — it should ride in the diff of
whichever substantive PR from this program next touches `mod.rs`, or not go up at all.

### 3.4 Signed / wide-integer lowering — the strongest surviving candidate class

This is the one stage where the constrained path is demonstrably leaving constraints
on the table, and the evidence is an *asymmetry inside the same directory*:

- `expand_signed_math.rs` and `expand_signed_checks.rs` contain **zero** value-bound
  queries. Exhaustive grep for `max_num_bits|get_numeric_constant|num_bits` over both
  files yields one hit, and it is the constant `FieldElement::max_num_bits()`
  (`expand_signed_checks.rs:400`). Everything is driven by the declared type width
  (`expand_signed_math.rs:96`, `:142`; `expand_signed_checks.rs:94`, `:111`, `:140`).
- The expansion pass **immediately before** `expand_signed_math` in the pipeline,
  `remove_bit_shifts` (`mod.rs:305` vs `:310`), **does** query
  `dfg.get_value_max_num_bits` (`:147`, `:156`, `:347`) and takes a truncation-free
  happy path when the bound fits (`:184-189`).
- And `checked_to_unchecked` — the pass that elides overflow checks from bounds — bails
  on non-unsigned explicitly (`:46-49`), because `expand_signed_checks` has already
  destroyed the `Add{unchecked:false}` shape at pipeline position 4. There is no
  `signed_checked_to_unchecked`.

So signed ops receive worst-case-always expansion and are then structurally excluded
from the only bound-based elision the compiler has. Structurally: a signed `Lt` at
width N emits **three** euclidean divisions (`expand_signed_math.rs:105`, `:107`,
`:114`); signed `Div`/`Mod` emits three plus three `eq_var`; a checked signed
`Add`/`Sub` emits **four** (`expand_signed_checks.rs:96`, `:187`, `:188`, `:201`) —
against a *single* range check for the unsigned equivalent, or zero when
`checked_to_unchecked` fires.

The proposed rewrite is small and lands in one file: two bound-aware folds in
`simplify/binary.rs` — `Div(x, C) → 0` and `Lt(x, C) → 1` when
`get_value_max_num_bits(x) < C.num_bits()`. Because inserted instructions route
through `simplify()` and every later `fold_constants` re-pushes existing instructions
through the same path, the fold fires both at expansion time and again after inlining
and unrolling, when bounds actually exist — **no pass needs to move.**

**But it is soundness-blocked, and the blocker is precise.** The fold is sound only
under: `get_value_max_num_bits(v) ≤ N-1` for `v : iN` implies `v ∈ [0, 2^(N-1))`, i.e.
a bound below the sign bit *proves non-negativity*. That rests on an invariant — no raw
`Cast(Signed{s} → Signed{t>s})` may exist — which the compiler maintains by
construction (`ssa_gen/context.rs:441-454` decomposes it into an explicit
`sign_extend`) but **asserts nowhere**. If such a cast ever appeared, a sign-extended
negative would be reported narrow and the fold would silently under-constrain the
circuit. Any implementation must first add that guard. A second dampener: the
realistic firing population may be dominated by compile-time constants, because the
`iK → iN` widening case reaches the bound query through sign-extension *arithmetic*
and will report the conservative full width.

One clean, **unconditionally sound** candidate falls out of the same stage:
`check_u128_mul_overflow`'s single-constant branches (`:126-134`) emit
`Div(x, 2^64)` + `Constrain(q·pred, 0)`, where `range_check(x·pred, 64)` is
algebraically identical and strictly cheaper — `x / 2^64 == 0 ⟺ x < 2^64` needs no
bound at all. It must range-check `x·predicate` itself, because the pass runs after
flattening (`mod.rs:374` vs `:315`) and ACIR-gen hard-codes `predicate = one` for
`RangeCheck` (`acir/mod.rs:548-556`). Two negatives from the same file, recorded so
they are not re-derived: the "small constant operand" guard is already at its
theoretical maximum (`MAX_NON_OVERFLOWING_CONST_ARG = ⌊p / u128::MAX⌋`,
`:68-79`), and the signed-mul `range_check`+`truncate` pair is already removed by
`remove_truncate_after_range_check`.

Finally, a **refuted hypothesis** worth printing: the paper's suspicion that these
passes run *after* the late-cleanup band and so never get cleaned up is **false** —
in `primary_passes` (`mod.rs:205`) they run at `:210`, `:310`, `:374`, all before the
band at `:395-396`. (`expand_signed_checks` also appears at `:437`, but that is
`minimal_passes`, a testing-only Brillig list — do not read it as a second
pipeline position.) The real
ordering finding is the opposite shape: they run early enough, but they lower into a
form (`Lt`/`Div`) that the late bound-based cleanup cannot see.

### 3.5 ACVM — `general.rs` (#10109), `common_subexpression/`, `MergeExpressionsOptimizer`

**#10109 is a compile-time optimisation, not a proving-cost one**, and the paper must
say so. The "on-the-fly merge" the TODO asks for **already exists** for the dominant
path: `Expression::add_mul` is a sorted two-way merge that combines duplicate linear
and mul terms as it builds (`expression/mod.rs:224-243`, `:266-291`). What remains are
raw push sites in `acir_context/mod.rs` (`:680`, `:689`, `:696-700`) that bypass it.
The single channel by which term-merging could change *size* — `width()` returning an
over-approximation without `simplify_linear_terms`, which drives CSAT's slicing
decision — is **already closed by pass ordering** (`GeneralOptimizer` runs before CSAT,
`optimizers/mod.rs:64` vs `:90-96`). The uncovered surface is Brillig predicates and
inputs (`optimizers/mod.rs:70-78`), which per §2 fact 1 costs no gates.

The CSE optimizer's **key normalisation is already complete over its key space** — the
paper should not claim a missed normalisation here. Commutativity is handled by
`sort()` canonicalising each mul pair to `w_l ≤ w_r` (`expression/mod.rs:144-148`);
scalar multiples and sign by `normalize`'s leading-coefficient division and the `k/l`
rescale on a hit (`csat.rs:267-287`, `:299-314`); and affine-offset equivalence is
*vacuous*, not missed, because every cached expression is constructed constant-free
(`csat.rs:192-196`, `:376-379`, `:412-420`) with the constant left on the outer opcode.

The genuine miss is in **candidate formation, not hashing**: slices are chosen greedily
by witness-index adjacency (`csat.rs:416-424`), so two opcodes sharing a term set that
is not contiguous in witness order get different keys and never CSE. Upstream flags
exactly this in-code (`csat.rs:240-242`, issue #10192 — note this is a *different*
issue from the #10109 the paper's gap table cites, and the more valuable one). This is
the only ACVM candidate whose mechanism plausibly survives backend decomposition,
because it removes recomputation of a shared value rather than repacking the same
arithmetic into fewer containers.

Two more results here. First, a **phase-ordering opportunity that is pure reordering**:
`optimize_internal` documents that it "Accepts an injected `acir_opcode_positions` to
allow optimizations to be applied in a loop" (`optimizers/mod.rs:46`) — and **nothing
loops it** (`:35` is the sole production call site; the only other is a unit test at
`:121`), while the inner transformer loop concedes
"some of them don't stabilize unless we also repeat the backend agnostic
optimizations" (`common_subexpression/mod.rs:123-124`). Second, a **soundness trap**
that is excellent paper material: the CSE cache is pass-global but *rebuilt* on each of
the ≤3 passes (`:181` sits inside `transform_internal_once`), and the obvious fix —
hoisting it to persist across passes — is **unsound**, because a cache hit emits no
defining opcode (`:185`, `:194` `skip(len)`) while `MergeExpressionsOptimizer` deletes
opcodes between passes (`merge_expressions.rs:140`). A pass-2 hit on an entry whose
defining opcode was deleted leaves the witness free — an under-constrained circuit that
no opcode-count or gate-count metric would reveal.

Correcting the prior claim carried in the program record §10.6: the
`MergeExpressionsOptimizer` two-use refund is real (`merge_expressions.rs:97-116`) but
(a) it is **not** restricted to CSAT intermediates — it fires on any non-IO witness —
and (b) the use-count spans **all** opcode kinds while the merge itself requires both
to be `AssertZero` (`:121-136`); that stricter counting is a deliberate fix for a real
soundness bug (#6527, regression test at `:397-415`). And the cache is circuit-global
**within** a pass, not across the compilation.

### 3.6 ACIR-gen — already examined, cross-referenced not re-derived

Comparison lowering (#10159) and quadratic ACIR expression growth (#4629) are the two
ACIR-gen rows in the paper's gap table. Both were examined by `sq-jfkwk` and `sq-felqr`
(program record §10.7, §10.6). This pass did **not** re-derive them; their records
stand, including the adjacent unimplemented candidate `sq-jfkwk` found
(`bound_constraint_with_offset`'s constant-`rhs` fast path failing at exactly `2^128`).

## 4. Surviving candidates, ranked

Ranked by (plausibility of a *constrained-path* win) × (smallness/soundness of the
diff). **None is measured.** "Prior" is my honest expectation, stated so it can be
falsified by the kill-test.

| # | Candidate | Site | Sound? | Prior |
|---|---|---|---|---|
| N1 | Bound-aware `Div→0` / `Lt→1` folds, narrowing signed expansion | `simplify/binary.rs` (+ a `Cast(Signed→wider Signed)` guard) | **Blocked** on an invariant asserted nowhere (§3.4) | Best class-match to prior wins; firing rate may be constant-dominated |
| N2 | `check_u128_mul_overflow` single-const branch → predicated `RangeCheck` | `check_u128_mul_overflow.rs:126-134`, `:156-167` | **Yes**, unconditionally | Cheapest, lowest-risk; small population |
| N3 | CSE candidate formation by subset matching | `csat.rs:416-424` (upstream TODO `:240-242`, #10192) | Structural; obligation is *solvability*, guarded by `CircuitSimulator::check_circuit` | Only ACVM mechanism that survives backend decomposition; novelty limited (upstream flagged) |
| N4 | Loop `optimize_internal` to a fixpoint | `optimizers/mod.rs:34-35` | Yes (reordering only); verify `acir_opcode_positions` composition | Cheapest test of all; **high risk of opcode-only win** (see §5) |
| N5 | Push `ArrayGet` through `IfElse` (literal #5501) | new arm in `array_get.rs:284-341` | **Yes**, by construction | Likely subsumed by fold-then-DIE; needs a profitability guard |
| N6 | Widen LICM's in-bounds prover to affine IV derivations | `loop_invariant.rs:743-768` | Yes (in-bounds ⇒ total) | Narrow; u32 index kills the general case; post-unroll CSE may already collect it |
| N7 | Raise/tune `MAX_ARRAY_SEMI_FLATTENED_LENGTH` | `array_set_window_optimization.rs:71-72` | Yes (parameter) | Pure compile-time-vs-gates trade; array-heavy corpora sit on the wrong side |
| N8 | Unify both sign-bit lowerings on `Lt` so they CSE | `expand_signed_math.rs:105/107/177/178` | Yes | Small; enables dedup (`Lt` is `Always`, `Div` only `UnderSamePredicate`) |

Deliberately **not** promoted: the `MergeExpressionsOptimizer` exact-use-recount
(moves from over- to under-approximation — the direction of the #6527 bug) and the CSE
cross-pass cache hoist (unsound, §3.5). Both belong in the paper as soundness
discussion, not as candidates.

## 5. Honest negative results (this is paper content, not leftovers)

The paper's §5 explicitly wants the candidates that did not reproduce. This pass
produces nine, each with a code citation rather than a failed measurement — which is
weaker evidence than a measured null, and must be labelled as such:

| Killed | Why |
|---|---|
| Loop unswitching (#6631), ACIR path | No ACIR loop survives unrolling (§2 fact 2); residual is a merge-count peephole, not unswitching |
| Induction-variable elimination (#6629), ACIR path | IV is substituted with a constant *inside* the unroller and folded on insertion; the bound-derived checked→unchecked half already runs pre-unroll |
| Both, Brillig path | Real optimisations, **zero proving-cost impact** (§2 fact 1) |
| `flatten_basic_conditionals` improvements | Brillig-gated at `basic_conditional.rs:280-284` |
| mem2reg cross-block × alias-aware quadrant | Vacuous for ACIR (§2 fact 3): single block post-flatten; a miss is a compile error, not gates |
| ACVM #10109 on-the-fly term merge | Already exists in `add_mul`; the size channel is closed by pass ordering — **compile-time only** |
| CSE key normalisation gaps (commutativity / scaling / affine) | Already complete over a constant-free key space |
| `check_u128_mul_overflow` small-constant narrowing | Guard already at the theoretical maximum `⌊p/u128::MAX⌋` |
| "Signed passes run after the late-cleanup band" | Refuted: they run at `mod.rs:210/310/374`, the band at `:395-396` |

And one methodological caveat the paper should carry, because it predicts which
survivors will die: `MergeExpressionsOptimizer` **deliberately trades opcodes for
width**, documenting a backend cost assumption in its own precondition
(`merge_expressions.rs:53-56`). ACIR opcode count and UltraHonk gate count are
therefore decoupled *by construction* at the pass that dominates the opcode metric —
so any candidate whose mechanism is "emit fewer opcodes" (N4 especially) is at high
risk of reproducing at opcode level and vanishing at gate level. That is the #10159
lesson recurring at a different layer.

## 6. Candidate bead specs — **NOT created** (each gated on a kill-test)

Per §0 no bead is spawned, because none cleared the "reproduced `bb gates` win" gate.
These are the specs a future run should create under `sq-uuvac` **after** the named
kill-test comes back non-zero. Ordered cheapest-first. Every one of them additionally
inherits the §10.2 acceptance protocol and the §6 upstream etiquette from
`noir-optimization-program.md`, and must re-check its upstream issue thread first.

1. **N2 → bead.** Tier: sonnet. Target `check_u128_mul_overflow.rs`. Kill-test: a
   counter on the single-constant branches (`:126`/`:132`) over the corpus; if the
   branch is never taken, drop it. No bound needed, so this is the only spec that could
   go straight to a measured PR.
2. **N7 → bead.** Tier: sonnet. Target `array_set_window_optimization.rs:71-72`.
   Kill-test: count candidates rejected *solely* by the 64-element cap. Report compile
   time *and* gates — it is explicitly a trade.
3. **N4 → bead.** Tier: sonnet, measure-first. Target `optimizers/mod.rs:34-35`.
   Kill-test: run it and diff **both** `acir_opcodes` and `circuit_size`; also check
   `execution_failure` assertion messages still resolve through the composed
   `AcirTransformationMap`. Fail-closed if only opcodes move.
4. **N1 → bead.** Tier: **opus** (constraint removal, under-constraining risk). Target
   `simplify/binary.rs` + a validation guard. Kill-test: would-fire counters at the
   seven sign-extraction sites, run **twice** — at expansion time and after the last
   `fold_constants` — because reporting only the former understates and only the latter
   overstates. Gate on the guard for `Cast(Signed→wider Signed)` existing first.
5. **N8 → bead.** Tier: sonnet. Can ride with N1. Note the tempting variant (exempting
   constant-divisor `Div` in `requires_acir_gen_predicate`) is **not** cleared: under a
   false predicate the quotient is unconstrained, so dedup across predicates needs its
   own argument.
6. **N5 → bead.** Tier: opus (merge logic). Target a new `IfElse` arm in
   `array_get.rs`. Kill-test: count merged lanes still present *after* DIE — if DIE
   already removes them, the win is compile time, not gates, and the bead should say so
   rather than ship.
7. **N6 → bead.** Tier: opus. Target `loop_invariant.rs:743-768`. Kill-test: the free
   one first — `--skip-ssa-pass "Loop Invariant Code Motion"` needs **no patch** and
   bounds the whole pass's value; then a per-site decline counter sub-bucketed by which
   conjunct failed.
8. **N3 → bead.** Tier: opus. Target `csat.rs`. Kill-test: a read-only analysis
   counting term-multisets of size `width-1` occurring in ≥3 opcodes but non-contiguous
   in witness order — that count is the candidate's upper bound.

## 7. What actually unblocks this — and what it means for P1 (`sq-i50o4`)

The blocker is unchanged from program-record §10.5–10.7 and is purely environmental:
**no `bb`** (so gate counts are unreachable) and no `nargo` here. Program record §10.8
establishes the recipe for the `nargo` half — rustc `1.89.0` (the workspace pin)
installs cleanly into a writable `RUSTUP_HOME` — but that run still had no `bb`, so
gate counts have never been reached by this program's own agents. Anything downstream
of "reproduce a `bb gates` win" is blocked on a box with `bb` installed plus enough
disk to build the compiler (a 1 GB tmpfs is not enough).

The more useful finding is for the P1 harness bead. **P1 should wrap upstream's own
instruments, not rebuild them** — this pass found four that the paper's §4.4 does not
mention:

- `test_programs/gates_report.sh` already runs `bb gates` per artifact and emits
  **both** `acir_opcodes` and `circuit_size` per program, with `reports.yml` wiring the
  `noir-gates-diff` arbiter. It is the metric-authority split of paper §4.1, already
  implemented. Note it **excludes** `databus*`, `fold_*`, `workspace*` and three
  `regression_*` packages, so any headline must be scoped to the reported subset.
- `gates_report_brillig.sh` / `gates_report_brillig_execution.sh` are separate
  channels. Given §2 fact 1, **every claim in the paper must state which channel
  moved**; only the first is proving cost.
- `--skip-ssa-pass <substring>` (`noirc_driver/src/lib.rs:124-126`) gives a
  **zero-patch A/B** for any named pass — the cheapest possible way to bound a pass's
  total value before designing an improvement to it.
- `tooling/nargo_cli/examples/ssa_pass_impact.rs` already walks the corpus in parallel
  with before/after SSA diffing, and `tooling/ssa_cli` replays a dumped `.ssa` through
  `primary_passes` without a full program.

Within sparq, `scripts/ci-bench.sh` (`zk-gate-counts`) is the existing precedent for a
`nargo`+`bb` bench that skips cleanly when the toolchain is absent — P1 should follow
that skip discipline rather than inventing a new one.

The repeated pattern across §4's kill-tests is worth stating once as a protocol,
because it is what made `sq-seust` cheap: **an env-gated per-site firing counter over
the ~556-package corpus, run before any implementation.** Several candidates here can
be killed for the price of one instrumented build and zero design risk.

## 8. Draft: the paper's "new opportunities" section (§5 of the outline)

Skeleton the paper can take directly, matching the estate's measured-and-honest style:

1. **Framing.** The seven PRs occupy one neighbourhood; a gap analysis over the rest of
   the pipeline is the "surface further optimisations while writing" mandate.
2. **The structural result (§2 above).** Four facts — Brillig is one opcode, ACIR has no
   loops, memory promotion is legalization, there is no ACIR cost model — that
   *predict* where constrained-path wins can exist. This reframes the gap table from a
   wishlist into a partition, and it is verifiable without a proving backend.
3. **The surviving class.** Worst-case-always expansion narrowed by a provable bound —
   with the signed-lowering asymmetry (§3.4) as the concrete instance and the
   `remove_bit_shifts`-vs-`expand_signed_math` contrast as the evidence.
4. **Honest negative results (§5 above).** Nine, each citing code. Labelled as
   *source-level* nulls, weaker than `sq-seust`'s measured 0-firings-in-556 null.
5. **Soundness discussion.** The two rejected-because-unsound candidates (partial array
   merge; CSE cross-pass cache) are better paper material than the survivors: both look
   like free wins and both under-constrain the circuit.
6. **Status.** Candidates are hypotheses with specified kill-tests, not results. The
   section must not present any as a win until §7's blocker is cleared.

## 9. Status & honesty

- **FINDINGS-ONLY.** No compiler source was modified; no upstream PR is proposed from
  this bead; no upstream comment was drafted or posted; no issue thread was read.
- **Nothing was measured.** No gate count, opcode count, firing count or timing in this
  record was executed. Every "cheaper" is structural. Source-level nulls are weaker
  evidence than measured nulls and are labelled as such throughout.
- **No bead was spawned**, because no candidate met the bead's own promotion gate. The
  bead `sq-mtolx` should therefore stay **OPEN**, blocked on the same empirical half as
  `sq-eesz3` / `sq-felqr` / `sq-jfkwk` — plus, for the promotion step specifically, on
  the P1 harness (`sq-i50o4`) landing.
- **The four non-optimisation findings are dispositioned in place, not left open.** The
  measurement gate above governs the §6 optimisation specs *only*; it is not a reason to
  leave unrelated work untracked. So, explicitly: the ACIR runaway-unroll hang and the two
  location-discarding ACIR bail-outs (§3.1) are captured as a `self-improvement` follow-up
  issue filed with this PR, findable by its dedupe marker `noir-acir-unroll-2026-07-28`
  (§3.1 gives the full title); the stale `mem2reg` ordering comment (§3.3) and the `CHANGELOG.md:67`
  over-claim (§1) are documentation-only and deliberately untracked, for the reasons given
  at each. No reader should expect a bead for any of the four.
- **HEAD is a moving target.** All citations are `e22cd89b` (the same commit
  §10.5–10.8 analysed). The shallow clone denied `git log`/`git blame`, so
  "did upstream already fix this" rests on `CHANGELOG.md` and fixture presence.
- **No ZK claim.** These are circuit-*size* (proving-cost) questions. Nothing here
  makes, needs, or supports any zero-knowledge privacy or soundness property; the
  soundness language above is about *circuit under-constraining*, which is a
  correctness property of a compiler transformation, not a ZK claim.
