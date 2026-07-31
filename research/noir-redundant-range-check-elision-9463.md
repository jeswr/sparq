# Redundant range / `LessThan` constraint elision (noir#9463) — status + pre-flip review (sq-rxir8)

**Bead:** `sq-rxir8` (epic `sq-uuvac`, `noir-optimization-program.md` §7 row 4, §10.1 *"noir#13266 — dominating-bound range/`Lt` elision"*) ·
**Status:** **the implementation ALREADY EXISTS upstream as draft PR [noir#13266](https://github.com/noir-lang/noir/pull/13266)** (opened 2026-07-04, still `DRAFT`, unmerged, no human review). No new implementation was written for this bead, and none should be — the remaining work is the human author-review gate plus the five pre-flip findings in §4 ·
**Author:** SPARQ agent 🤖 [OPUS-5] · **Date:** 2026-07-31

## 0. What this record is, and what it is not

This bead was routed as an implementation task ("implement the redundant
range/`LessThan` elision pass"). It is already implemented — the same tracking
situation `sq-b0vpc` hit (§10.11, `noir-bounded-vec-capacity-assert-5027.md`), for
the same reason: §10.1's status table records the PR but the row reads as
"in-progress" rather than "implemented, awaiting author review". This record exists
so the next agent to read §7 row 4 does not re-derive a 513-line pass that has been
sitting upstream since 2026-07-04, and so the review work that *is* outstanding is
written down.

Environment, stated up front because it bounds every claim below:

| capability | available | consequence |
|---|---|---|
| upstream PR/issue **API + diff** (unauthenticated, read-only) | yes | PR state, body, commits and the full diff are **verified**, quoted from the `.diff` and the REST payloads |
| upstream `master` sources (`raw.githubusercontent.com`) | yes | every line citation in §3 is **verified at `master` on 2026-07-31** |
| noir checkout / `cargo` / `nargo` / `bb` | **no** | **none of the six `noir-optimization-program.md` §10.2 acceptance criteria were run in this session.** No number in this record is a measurement taken here |

Nothing was posted upstream from this bead.

## 1. Verified upstream state (2026-07-31)

| fact | value |
|---|---|
| PR | noir-lang/noir#13266, *"feat(ssa): remove range checks and unsigned lt comparisons implied by a dominating bound"* |
| author / state | @jeswr · `DRAFT`, open, unmerged; 3 commits (`86e34c24`…`cebd23f8`, 2026-07-04), last updated 2026-07-10 |
| size | 8 files, +561 / −0 — one new pass (`ssa/opt/remove_redundant_range_checks.rs`, 513 lines incl. 13 tests), 2 one-line registrations, one `execution_success` fixture + its 2 snapshots. No existing file's behaviour edited; no scope creep |
| review | **zero human review comments.** The only comment on the thread is the repo's automated do-not-force-push sticky note |
| base | `bd01f81e` (master @ 2026-07-04). GitHub reports `mergeable: null` / `mergeable_state: unknown` — i.e. **not recomputed at fetch time**, which is not evidence of a conflict either way |
| `master` today | carries **no** `remove_redundant_range_checks` (absent from `ssa/opt/mod.rs`) — the pass has **not** landed, and no equivalent has landed in its place |
| rebase surface | the insertion anchor is intact: `ssa/mod.rs:395-396` still reads `remove_truncate_after_range_check` then `checked_to_unchecked` (the PR's hunk context, shifted +2 lines), and the mirrored sibling pass `remove_truncate_after_range_check.rs` is unchanged at master. A textual rebase looks trivial; **the measurements are the part that goes stale**, not the diff |
| the issue | #9463 *"Remove redundant LessThan constrains"* — still `open`, untouched since 2025-11-11. #9429 (its cross-link, row 7 / `sq-jj3ne`) likewise |

## 2. What the change does (read from the diff, not run)

A new SSA pass, slotted immediately after `remove_truncate_after_range_check` in
`primary_passes`, keeping one map from value → smallest proven *exclusive* upper
bound, plus a second map of `range_check`-derived bit sizes. Three fact sources,
two consumers:

| | learned from | consumed by |
|---|---|---|
| `range_check v to N bits` | ⇒ `v < 2^N`; recorded unconditionally | elides a dominated `range_check` of the same value to ≥ N bits, **and** an implied `lt` |
| `constrain (lt v, c) == u1 1` | ⇒ `v < c`; recorded unconditionally | elides an implied `lt` only — **never** a `range_check` |
| `r = mod v, c` (unsigned, constant `c ≠ 0`) | ⇒ `r < c`; recorded **only** while the side-effects condition is the constant `true` | elides an implied `lt` only |

An elided `lt` is replaced by `u1 1`, after which its consuming `constrain`
simplifies away in the same traversal. The `range_check`-elides-only-`range_check`
restriction is deliberate and is the PR's own second commit: the narrowing-cast
validation rule justifies casts by the range checks that *remain* in the SSA, so a
removed check's justification has to stay visible as a `range_check`. Signed `lt` is
neither learned from nor elided. Thirteen SSA-level unit tests ship with the pass, **six
of them `assert_ssa_does_not_change` non-firing tests** (tighter bound, non-dominating
constraint, signed, `lt`-must-not-elide-`range_check`, tighter `range_check`,
`mod` under a non-constant predicate) — those six are the guards that go red in the
*over-firing* (under-constraining) direction, which is the direction that matters.

## 3. Where the soundness argument rests (each premise checked against `master`)

The PR's module doc makes four load-bearing claims. Each is checkable in-tree, and
each checked out at `master` on 2026-07-31:

1. **"`constrain` and `range_check` are enforced unconditionally; flattening bakes
   the predicate into their operands."** Confirmed: `flatten_cfg.rs:1422-1426`
   rewrites `Constrain(lhs, rhs)` to `Constrain(cond·lhs, cond·rhs)`, and
   `flatten_cfg.rs:1472-1478` rewrites `RangeCheck { value }` to
   `RangeCheck { value: cond·value }`. So a surviving instruction whose operands are
   the *raw* values is one flattening did not predicate, and a predicated one keys any
   learned fact on the multiplied value (for which the bound is trivially true when
   `cond = 0`). The pattern match therefore cannot pick up a predicated constraint as
   if it were unconditional: a predicated `constrain` no longer has a constant `1`
   operand for `lt_true_bound` to match.
2. **"`mod` is predicated at ACIR generation, so its bound needs the constant-true
   guard."** Consistent with `flatten_cfg`'s own instruction classification (`Binary`
   sits in the "arguments need not be nullified" list, so the SSA `mod` keeps raw
   operands and the predication happens later), and with §10.7's independent reading
   of `euclidean_division_var` (the remainder's `r < rhs` bound constraint), taken at
   a different pin. Not re-derived from ACIR-gen sources here.
3. **No circularity** — the §2 program constraint inherited from jeswr's own #12780
   ("never optimize based on a comparison result that the comparison itself
   enforces"). The fact is learned when the `Constrain` is visited, which in SSA is
   strictly after the `lt` it constrains, so the enforcing comparison can never be
   its own victim. The pass also derives no bound from a value's *type*, because a
   `range_check` can be the very instruction establishing that type invariant.
4. **Dominance.** The clearing discipline (clear both maps when the previously
   visited block does not dominate the current one) is copied verbatim from
   `remove_truncate_after_range_check.rs:36-48`, unchanged at master. It is
   conservative-but-sound by transitivity: facts survive only through a chain of
   consecutive dominating blocks, and dominance is transitive, so every retained fact
   dominates the instruction it elides.

Per `sq-qhy4` this record does **not** certify the pass sound. §10.2's stake is
unchanged: a wrongly elided constraint under-constrains the circuit, which is a
correctness/soundness bug rather than a perf bug. What §3 establishes is only that
the four premises the PR states are the premises the compiler actually provides —
the review a maintainer would otherwise have to do from scratch.

## 4. Five findings a reviewer must clear before flipping the draft to ready

**4.1 The `execution_success` fixture cannot go red for *any* over-firing of the pass.**
`Prover.toml` supplies `a = 9`, and the fixture asserts `a < 16`, `a < 17`, `a < 10`.
All three hold at `a = 9`, so removing any of them — including the reversed-direction
`a < 10` the PR body calls "a semantically-active reversed-direction control assert"
— leaves the honest execution and its `Circuit output: 18` snapshot identical. The
PR adds no ACIR/SSA snapshot for the fixture either, so nothing pins that `a < 10`
survives the pass. The control is present in the source, not load-bearing in the
test. **Fix is cheap and idiomatic:** add a companion
`test_programs/execution_failure/` fixture over the same body with an input that
violates only the kept check (e.g. `a = 12`: `12 < 16` ✓, `12 < 17` ✓, `12 < 10` ✗),
so a wrongly elided `a < 10` turns the expected failure into a pass and the test goes
red. (This guards execution-level rejection; it still does not exercise a malicious
witness against the constraint system, which is the residual limit §10.11 records for
`sq-b0vpc`.) The six `assert_ssa_does_not_change` unit tests remain the pass's real
over-firing guards, and they are shape-specific by construction.

**4.2 A competing sibling PR from the same author is live.**
[noir#12927](https://github.com/noir-lang/noir/pull/12927) — *"value range analysis
for tighter bit-widths and fewer range checks"*, **not** a draft (ready for review,
25 files, +2524/−115, last updated 2026-06-05) — adds
`ssa/ir/dfg/range_analysis.rs` (+1710) **and its own `ssa/opt/range_check_elision.rs`
(+49)**. Both PRs are open, both are @jeswr's, and both elide range checks. §8 step 3
and §10.3 already record "coordinate with #12927 to avoid competing PRs", but they
scope that pre-flight to the flagship bead `sq-jj3ne`; it bites at row 4 too, and it
bites now. A maintainer opening both will ask which supersedes which. That question
should be answered in the PR bodies *before* either is flipped, not in review.

**4.3 The pipeline invariant the `mod` guard depends on is not asserted, though
upstream has the idiom for it.**
The guard reads `context.enable_side_effects`, which `simple_optimization.rs:71`
resets to the constant `1` **at the start of every block** — it is not a cross-block
dataflow. That is sound exactly under upstream's own documented invariant, *"a
non-trivial side-effects predicate is confined to a single block"*
(`remove_enable_side_effects.rs` preconditions; `checks.rs::assert_not_enable_side_effects`).
Upstream guards that invariant where it is relied on: `array_get.rs:238-247`'s
`array_get_optimization_pre_check` asserts it under `#[cfg(debug_assertions)]`
precisely so "a future pipeline change that breaks the invariant" fails loudly
"rather than letting it silently emit an unsound fold". #13266 relies on the same
invariant and ships no such check — and the idiom **already existed at the PR's own
base commit** `bd01f81e`, so this is an omission at authoring time, not post-hoc
drift. This is not a demonstrated live bug: at the pass's position (after
`flatten_cfg` and after `remove_enable_side_effects`) ACIR functions are single-block,
and Brillig functions carry no `EnableSideEffectsIf` at all. It is a ~10-line
hardening that costs nothing and matches the file next door.

**4.4 The second motivating example in #9463 no longer reproduces at HEAD.**
The issue's `let t = [a, a, a]; t[a % 3]` shows an explicit `lt` + `constrain "Index
out of bounds"` in its 2025-11 SSA dump. At master today
`array_index_needs_explicit_oob_check` (`ssa_gen/context.rs:131-136`) returns
`runtime.is_brillig() || array_type.element_size().0 != 1`, so an ACIR index into a
simple-element array emits **no** explicit check — ACIR's built-in memory-op bound is
relied on instead (the same mechanism §10.11 discusses for `sq-b0vpc`). The PR's
fixture works around this correctly and says so in a comment ("a composite element
type is used so the explicit SSA-level check is emitted"), so the PR is not wrong —
but the consequence is worth one line upstream rather than left for a maintainer to
discover: the `mod`-derived fact reaches composite-element ACIR indexing and Brillig,
not the issue's literal example. That is consistent with, and probably explains, the
PR's own honest caveat that the external 39-program ZK corpus is unchanged.

**4.5 The gate row is not reproducible in the fleet environment today; the ACIR half is.**
The PR's table is stated with a named toolchain (`bb 5.0.0-nightly.20260522`,
`bb gates -s ultra_honk`; ACIR opcodes via `nargo info --force`) and a named corpus
(508 `execution_success` + 9 `benchmarks` + 31 external circuits + 8 kernels) —
materially better provenance than the `sq-b0vpc` table §10.11 flags. Two things
remain true anyway: (a) **nothing was re-measured in this session**; (b) the
UltraHonk `circuit_size` rows are exactly the figures §10.8 and §10.10 record the
fleet environment cannot produce — no `bb`, with the harness bead `sq-i50o4` not
landed. The useful distinction, which the sibling records do not draw: the **ACIR
opcode half is reproducible today** via §10.8's recipe (rustc `1.89.0` installs into
a writable `RUSTUP_HOME`; a release `nargo` builds; `nargo info` needs no backend),
so `vector_dynamic_index` 733 → 616 and the 555-program no-regression sweep can be
re-taken now. Only the two `circuit_size` rows wait on `sq-i50o4`. §5.1 makes gates
the arbiter and §4.2's #10159 lesson is that opcode counts alone mislead, so the
gate rows stay load-bearing — but the cheap half of the confirmation is not blocked.

One arithmetic reconciliation to do while re-taking it: the PR body's summary says
the sweep covered *"the full `execution_success` suite + benchmarks (555
programs)"*, while its own measurement note says *"all 508 `execution_success`
programs + the 9 `test_programs/benchmarks` programs"* — which is 517, plus a
separately-named external corpus the same paragraph calls 39 programs (31 circuit
binaries + 8 kernel probes). 517 + 38 = 555 and 517 + 39 = 556, and §10.8's own
corpus count is 38 sparq packages, so the headline appears to fold the external
corpus into the "suite + benchmarks" figure with an off-by-one somewhere. Neither
reading changes the *result* (no regressions), but `noir-optimization-paper.md`
§1.2 has already carried "across 555 programs" forward as a claim, so the two
statements should agree before the flip.

## 5. Recommendation

The bead's implementation work is **done and should not be repeated**. Open actions,
in order:

1. Resolve the #12927 overlap (§4.2) — a one-line "supersedes / is superseded by /
   independent because …" in each PR body. This gates the flip; the others do not.
2. Add the `execution_failure` companion fixture (§4.1) so the reversed-direction
   control is actually red-on-wrong-answer.
3. Add the `#[cfg(debug_assertions)]` pre-check mirroring
   `array_get_optimization_pre_check` (§4.3).
4. Re-take the ACIR-opcode half of the table at current master with §10.8's recipe
   (§4.5) — it has drifted ~4 weeks — and add the §4.4 one-liner on the issue's
   second example while editing the body.
5. Then @jeswr flips draft → ready per §6. No agent arms anything; upstream merge is
   the only "arm".

Steps 2–4 are small enough to ride on the same rebase. Nothing here requires
re-deriving the pass.

Companion records: `noir-optimization-program.md` (§7 row 4, §10.1 status, §10.2
acceptance protocol, §10.4 dependency edges, §10.12 status),
`noir-bounded-vec-capacity-assert-5027.md` (the sibling already-implemented bead and
the ACIR built-in-bounds-check discussion), `noir-optimization-paper.md` §1.2 (the
#13266 case-study row), `noir-optimization-new-opportunities.md` (the `sq-i50o4`
bottleneck).
