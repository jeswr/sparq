# `BoundedVec::push` capacity-assert slice of noir#5027 — status + pre-flip review (sq-b0vpc)

**Bead:** `sq-b0vpc` (epic `sq-uuvac`, `noir-optimization-program.md` §7 row 11, §10.3 *"`noir_stdlib/src/collections/bounded_vec.nr`; only the TomAFrench capacity-assert slice of #5027"*) ·
**Status:** **the implementation ALREADY EXISTS upstream as draft PR [noir#13314](https://github.com/noir-lang/noir/pull/13314)** (authored 2026-07-10, still `DRAFT`, unmerged). No new implementation was written for this bead, and none should be — the remaining work is the human author-review gate ([sparq#1840](https://github.com/sparq-org/sparq/issues/1840)), plus the two pre-flip findings in §3 ·
**Author:** SPARQ agent 🤖 [OPUS-5] · **Date:** 2026-07-29

## 0. What this record is, and what it is not

This bead was picked up cold by a fleet agent whose brief said "implement the
BoundedVec capacity-assert slice". It is already implemented. This record exists so
the next agent to read §10.3 does not re-derive a change that has been sitting
upstream since 2026-07-10.

Environment, stated up front because it bounds every claim below:

| capability | available | consequence |
|---|---|---|
| upstream PR/issue **web** pages (unauthenticated, read-only) | yes | PR state, diff and commit message are **verified**, quoted from `13314.diff` / `13314.patch` |
| upstream `master` sources (`raw.githubusercontent.com`) | yes | the line citations in §2–§3 are **verified at `master` on 2026-07-29** |
| noir checkout / `cargo` / `nargo` / `bb` | **no** | **none of the five `noir-optimization-program.md` §10.2 acceptance criteria were run in this session.** No number below is a measurement taken here |
| the issue #5027 discussion thread | **not read** — only the PR's own reference to it | TomAFrench's original framing is carried from §7 row 11 unverified |

Nothing was posted upstream from this bead.

## 1. Verified upstream state (2026-07-29)

| fact | value |
|---|---|
| PR | noir-lang/noir#13314, *"feat(stdlib): move BoundedVec push capacity check to unconstrained hint"* |
| author / state | @jeswr · `DRAFT`, unmerged (review state not checked) |
| commit | `efec45ef2e6a562b2c452bcfac84ff4fa9a9014c`, authored 2026-07-10 |
| files touched | `noir_stdlib/src/collections/bounded_vec.nr` only (+19 −1) — exactly the §10.3 target, no scope creep |
| `master` today | still carries `assert(self.len < MaxLen, "push out of bounds")` at `bounded_vec.nr:182` — the slice has **not** landed |

The change replaces the constrained assert in `push` with a call to a new
`unconstrained fn assert_push_in_bounds(len, max_len)` inside an `unsafe {}` block,
so the check still produces its error message during witness generation but emits no
ACIR constraint.

This matches the draft-first and one-optimization-per-PR halves of the §6 upstream
protocol. The §6 disclosure line is **not** verified here — the PR *body* was not
read, only the diff and the commit message. The block is on @jeswr, not on the fleet.

## 2. The safety argument, and where it actually lives

The commit message justifies the elision as:

> In constrained (ACIR) mode the ACVM already enforces array-write bounds: a write at
> `self.storage[self.len]` when `len >= MaxLen` raises
> `OpcodeResolutionError::IndexOutOfBounds`, so safety is fully preserved.

That names an **ACVM execution-time error**. An executor-side error is a property of
the honest prover's witness-generation run; on its own it does not establish that the
*constraint system* rejects an out-of-range write, which is the property that matters
once the assert is no longer constrained. §10.2 states the stake exactly: a wrongly
elided constraint under-constrains the circuit, and that is a soundness bug, not a
perf bug.

The stronger, in-tree argument the PR does not make is that upstream **already relies
on this same mechanism for every plain array index**. At `master`:

- `ssa_gen/context.rs:131-136` — `array_index_needs_explicit_oob_check(runtime, array_type)`
  returns `runtime.is_brillig() || array_type.element_size().0 != 1`.
- `ssa_gen/mod.rs:536-586` (read, and the shared `LValue::Index` path) and
  `context.rs:893-909` (write) call `codegen_access_check` **only when that
  predicate holds**.

So for ACIR with a simple element type (`element_size == 1`) the compiler emits *no*
explicit bounds constraint for `self.storage[self.len] = elem` and leans on ACIR's
built-in memory-op check — the in-code comment at `ssa_gen/mod.rs:591` says so
outright ("ACIR has built-in OOB checks"). For a composite element type the predicate
holds and `codegen_access_check` *is* emitted. Either way the array write carries a
dominating bound.

That reframes the residual risk honestly: if ACIR's built-in memory-op bound were not
binding in-proof, **every simple-element array index in noir would already be
under-constrained**, which is a far larger claim than this PR. The elision is
therefore *plausibly* dominated. It is **not certified sound here** — no acceptance
criterion was run in this session, and per `sq-qhy4` this repo does not label
unaudited ZK work sound.

## 3. Two findings a reviewer must clear before flipping the draft to ready

**3.1 The PR ships no test that goes red if the elision is wrong.**
`bounded_vec.nr:1353-1354` at `master` already has
`#[test(should_fail_with = "push out of bounds")] fn push_to_full_vector()`. With the
patch that test still **passes**, because the assert still fires during execution —
which is the whole point of routing it through the unconstrained hint. So the existing
test cannot distinguish "constrained check" from "unconstrained hint", and the PR adds
none that can.

Nor does §10.2's step-2 differential (`nargo execute --force` vs `--force-brillig`)
close that gap. Both sides are *execution* paths, and the new unconstrained assertion
makes **both** reject during honest witness generation whether or not any constraint
binds a malicious witness — so the differential stays green under exactly the
under-constrained implementation it would need to catch. Run it as what it is, an
execution-equivalence check, and not as the soundness regression guard.

A witness for the actual property has to bypass or mutate honest witness generation —
supply a witness whose out-of-range `self.storage[self.len]` write is *not* caught by
the prover-side hint — and show that the constraint system / verifier rejects it.
**No test available today establishes that**, neither in the PR nor at `master`, and
this record does not claim one exists; the flip stays blocked pending one (§4 item 2).

**3.2 The measured table is not reproducible in-repo.**
The commit message carries a before/after table (ACIR opcodes and UltraHonk gates on
the SIZE=500 reproducer) plus "0 delta" on the standard benchmark corpus and on the
sparq compose corpus. Two observations, neither of which establishes the numbers as
taken:

- *Consistent, but not evidence of provenance:* "sparq compose corpus (8 packages)"
  and the four named standard benchmarks match §5.2's baseline corpus exactly — 8
  compose packages and 4 noir benchmark packages. That is a consistency check on the
  **labels** only. Matching names and package counts do not establish that the
  commands were run, that the reported values came from those runs, or what
  toolchain/commit produced them; measurement execution and provenance remain
  **unverified**.
- *Not reproducible:* the gate row is the one figure later program records say the fleet
  environment could not produce — §10.8's capability table records no `bb`, and §10.10
  records "no `nargo`, no `bb`, no build". The harness bead `sq-i50o4` that would make
  any of this re-runnable has not landed. The row is therefore **unreproducible today**,
  which is not the same as wrong: the PR predates both records, and §5.2 says gate
  baselines are taken per-PR on ephemeral workspace scripts.

§5.1 makes the backend gate count the arbiter and §4.2's PR #10159 lesson is that
opcode counts alone mislead — so this row is load-bearing and it goes upstream under a
named human author. Confirming its provenance (or re-taking it) is a precondition for
the flip, not a nicety.

## 4. Recommendation

The bead's implementation work is **done and should not be repeated**. The open
actions, in order:

1. @jeswr confirms the §3.2 gate row's provenance or re-takes it (blocked on
   `sq-i50o4` for a reproducible version).
2. Add a test that is red when the capacity bound is not enforced at constraint level
   (§3.1) — one that bypasses or mutates honest witness generation and demonstrates
   verifier/constraint-system rejection of an out-of-range index, **not** an
   execution-path differential. Until such a test exists the PR carries no regression
   guard for the property the elision relies on, and stays blocked on it.
3. Upgrade the commit/PR safety justification from the ACVM execution-time error to
   the `array_index_needs_explicit_oob_check` contract (§2), which is the citable
   in-tree reason and is the argument a noir maintainer will want to see.
4. Then flip draft → ready per §6. No agent arms anything; upstream merge is the only "arm".

Companion records: `noir-optimization-program.md` (§7 row 11, §10.2 acceptance
protocol, §10.3 fleet spec, §10.11 status), `noir-optimization-new-opportunities.md`
(the `sq-i50o4` bottleneck).
