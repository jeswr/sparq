# Guard mortality, and detecting it by diffing the mutation kill set (sq-1lc4i)

**Status:** design record — a test-quality failure class and the detection method for it.
Authored under the proceed-and-document rule.
**Author:** Claude Opus 5, 2026-07-28. [OPUS-4.8]
**Scope:** the Python orchestration/CI scripts and their YAML seams, where the evidence was
gathered. The mechanism is not Rust-specific, but every measurement below is from that
surface. Companion: `research/test-quality-program-plan.md` (the per-crate cargo-mutants
ratchet — a different surface and a different mechanism; this record does not restate it).

---

## 1. The finding

> **A guard placed inside a feature's region inherits that feature's mortality.**

A test or assertion that is physically nested inside the block of code, self-test section,
or module that it protects is deleted *along with* that block when the feature is removed,
refactored, or cut. The guard's death is invisible at review time because the deleted lines
**look like** the feature — a reviewer reading the diff sees a coherent removal, and the
assertion that would have objected is inside the removal.

The generalised form is broader than physical nesting. A guard is mortal whenever its
survival is *coupled* to the thing it guards:

| coupling | how the guard dies |
|---|---|
| **Textual nesting** — assertion lives inside the feature's region | removed with the region |
| **Call-site indirection** — every assertion invokes the helper *directly*, never through the production path | the one line wiring the helper into production is deletable, and nothing objects |
| **Wrong-module binding** — the guard reads a symbol from the module that *defines* it, not the module the consumer *loads* | the guard passes while the consumer sees nothing |
| **Fixture-injected contract** — the test fixture *adds* the contract to its own copy of the subject before asserting on it | the fixture satisfies a contract the real target does not |

All four share a signature: **the unit under test still passes, and the system is no longer
protected.** Line coverage cannot see any of them, because the guard genuinely executes.

The last two are the nastiest, because the guard is not merely absent — it is *actively
reporting success* about a contract nothing upstream honours.

## 2. The detection method

### 2.1 Diff the kill set across a deletion

Run the mutation battery **before** and **after** a removal, and compare per-mutant status.

> **A mutant that moves `KILLED` → `SURVIVED` when a feature is *removed* is the
> signature of guard mortality. Nothing else produces that transition.**

A mutant flipping to `SURVIVED` means the assertion that used to catch it is gone. If the
deletion was supposed to remove only the feature, then the deletion also took a guard with
it — one that was protecting something else, otherwise the mutant would have been removed
from the spec along with its target.

Distinguish the two legitimate outcomes so the signature stays sharp:

- mutant's **target** was deleted → the anchor no longer exists → the mutant leaves the
  spec entirely. This is expected and is *not* the signature.
- mutant's target **still exists** but the mutant now survives → **guard mortality.**

### 2.2 The corollary that makes it necessary

> **A green suite after a deletion proves nothing about what the deletion took with it.**

This is why review and CI both miss the class. The suite no longer *contains* the assertion
that would have complained, so its greenness is vacuous with respect to the removal. Asking
"did the tests pass after I deleted this?" is the wrong question; the right one is "which
assertions did I stop running?" — and only a kill-set diff answers it.

### 2.3 Method trap: the results artefact must be append-only

A results file named after the first run is silently **overwritten** by a re-run against the
same path, so the pre-fix state is destroyed exactly when it becomes the evidence. During
this work `results-reg-1031.json` was re-used for a post-fix run; the earlier `SURVIVED`
verdict survived only in the append-only per-run log, and the overwrite was provable only
from file mtimes. **Write each run to its own path, and cite the per-run log, not a mutable
results path.**

### 2.4 The symmetric rename, for contracts crossing a module or repo boundary

Kill-set diffing catches guards that *disappear*. It does **not** catch a guard that is
bound to the wrong side of a contract — for that the mutant must be a **symmetric rename**:

> **Rename the symbol in the production module *and* in its own tests, together. If nothing
> reds, no assertion is checking the name the CONSUMER resolves — only that the definition
> and the test agree with each other.**

A one-sided rename is useless here: it reds trivially because the test still names the old
symbol. The symmetric version is the one that exposes the coupling, and it is exactly the
mutant that survived in §3.3. Where the consumer is in another repository, or loads the
module dynamically, this is the only cheap way to test the contract at all.

The corresponding practice: **assert the symbol through the consumer's own resolution path**
(the same `getattr` / import the consumer performs), and never let a fixture supply the
contract it is testing for (§3.3).

### 2.5 Before tuning a threshold, ask whether the field can express the quantity

A guard can also be vacuous because the *variable* is incapable of carrying the signal, not
because the assertion is weak. This one is worth a named check because the failure looks
exactly like "we have not seen a positive yet", which invites an indefinite search for a
better threshold instead of a look at the data source.

The tell is a distribution that is **too clean to be a measurement**:

- an interval that is **exactly** zero across a very large N — not near-zero, identically
  zero — is what two fields set *together* look like, not two events being timed;
- an interval that is **negative**, at all, for a quantity that cannot be negative.

MEASURED while cutting the queue-wait detector (§3.1's PR): `run_started_at − created_at`
was **exactly 0.0 on all 44,123** attempt-1 runs across two repositories, and
`run_started_at − attempt.created_at` was **negative on all 67** re-run attempts
(`{−1: 48, −2: 17, −4: 1, −5: 1}`). A queue wait cannot be negative, so those fields are
stamped together at start rather than spanning enqueue-to-start. **No threshold on them
could ever have worked**, and the all-zero corpus was never evidence that no queueing
happened — only that the fields do not report queueing.

The practical consequence is for the *exit condition* you write down. "Ship it when we
observe a positive" is wrong guidance if the field cannot produce one; it sends the next
reader to re-derive a detector that cannot exist. Say instead what *different data source*
would be required. Getting this backwards is itself a guard that reports success about
something it cannot see.

### 2.6 Cheap approximations when a full battery is too slow

- After any deletion, ask of each removed assertion: *what was this protecting, and does
  that thing still exist?*
- Grep the removed diff hunk for assertion keywords (`assert`, `chk(`, `check(`, `expect`)
  before committing a removal.
- Structurally, prefer that a guard live **outside** the region it protects (§4).
- Eyeball the distribution before tuning a threshold on it (§2.5).

## 3. Instances

Three distinct instances, same class, one working session. All are Python orchestration
scripts with hermetic self-tests plus a separate unittest suite.

### 3.1 Textual nesting — the sharpest case

**Where:** `scripts/ci-latency-alert.py` (jeswr/agent-account-registry#1031).

A `fetch_lanes` assertion — that each lane carries the `created_at` the new-lane guard
reads — had been appended *inside* the queue-wait mode's self-test region. When that whole
mode was cut, the assertion went with it.

| battery run | tree state | `fetch_lanes drops created_at` |
|---|---|---|
| `reg-v3.log` | after the mode was cut | **SURVIVED** |
| `reg-v4.log` | after the guard was restored standalone | **KILLED** |

Consequence had it shipped: the new-lane guard would have been **permanently unreachable in
production** — the field it reads would never be populated — while the self-test stayed
green, because the self-test exercised the guard against a hand-built fixture rather than
against the fetch path.

**Fix:** restore the assertion outside any mode's region, so no single feature removal can
take it.

### 3.2 Call-site indirection — inside the fix for the same class

**Where:** `scripts/ci-latency-alert.py` / `scripts/ci_execution_latency_alarm.py`.

An enrichment pass was added to correct a detector that read a stale timestamp. Every
assertion called the enrichment helper **directly**. The single line wiring it into the
production fetch path was therefore deletable with the suite green.

| battery run | tree state | `M2-CALLSITE enrichment pass deleted` |
|---|---|---|
| task log `b5h0sfjb8.output` | before a call-site test existed | **SURVIVED** |
| `reg-run2.log` | after adding a test driving the real fetch path | **KILLED** |

Notable because this occurred **inside the code written to fix call-site vacuity**. Knowing
about the class is not protection against it; only the kill-set diff caught it.

**Fix:** a test that drives the production entry point against a stubbed transport and
asserts both that the enrichment happened *and* that the downstream detector changed
behaviour because of it.

### 3.3 Wrong-module binding, plus a fixture-injected contract

**Where:** sparq-org/sparq#4823 with jeswr/agent-account-registry#1032 — the dispatch
inertness-proof pair. Both still open and uncorrected at the time of writing.

The consumer is the registry's `dispatch.yml`, which loads **sparq's**
`scripts/dispatch-plan.py` as the target planner and probes it with `getattr` for an
inertness contract, falling back to *"planner declares no inertness contract"*.

`INERT_FIELD` and `MACHINE_PARK_PR_LABEL` were both added to sparq's
`scripts/ready-issues.py`, and **neither appears in `scripts/dispatch-plan.py`** — that
file's re-export block forwards eight other names and #4823 does not touch it at all. The
probe's guard requires both, so it fails on the first `getattr` and the pair is inert as
shipped. Note the filename is ambiguous across the two repos: the registry has its own
`scripts/dispatch-plan.py`, which is only its self-test harness — the module that matters
is sparq's.

Two independent green signals failed to notice, one per repo, by different mechanisms:

- **sparq's vacuity battery reported all of its mutants dead**, but a *symmetric rename*
  (§2.4) of `INERT_FIELD` in production and self-test together **survives** it — every
  assertion references the symbol, so none can observe whether the consumer resolves it.
- **the registry's battery was green because its fixture planners inject the contract into
  the fixture's own source** before asserting on it. The fixture therefore satisfies a
  contract the real target does not — the fixture-injected row in §1.

**Fix shape:** re-export the names from the module the consumer loads, and assert them
through the consumer's own resolution path rather than the definer's.

**On the two frontier numbers**, which look contradictory and are not: the authors' `1 → 9`
measures the *lever* — the effect if the attestation is carried — while an independent
reviewer's replay of the merged state measured no movement, and movement only after adding
the missing re-export. Both are real; they quantify different things. The defect is that
the PR title asserts the lever figure as the delivered effect. *(The numeric replays are
the reviewer's, not this record's — see §6.)*

## 4. Practice

1. **Place a guard outside the region it protects.** If an assertion protects behaviour X,
   it must not live inside the block that implements X. A separate, plainly-named test
   class or self-test section survives a feature removal; a nested assertion does not.
2. **Diff the kill set across every deletion** that removes a region containing assertions.
   Report the transitions, not just the final totals — `20/20 killed` on the post-deletion
   tree says nothing about what left the spec.
3. **Use a symmetric rename for any contract crossing a module or repo boundary** (§2.4),
   and never let a fixture inject the contract it is asserting on.
4. **A mutant must target the production entry point**, not only the helper. If every
   assertion calls the helper directly, add one that goes through the real call site.
5. **Bind guards to the consumer's view.** Assert a symbol through the module that loads it.
6. **Write each battery run to its own path** and cite per-run logs (§2.3).
7. **Check the field can express the quantity before tuning a threshold on it** (§2.5), and
   write the exit condition in terms of the data source, not a future observation.
8. **A crash is not a kill.** A mutant counts as killed only on a *named* assertion failure;
   a traceback certifies nothing, because it reads the same whether the guarantee broke or a
   test stub broke. Anything else should be reported separately as crash-only.

## 5. Limits — what this record does not claim

- **Not a claim about frequency.** Three instances in one session on one surface is enough
  to name the class, not to estimate a rate. No sampling was done.
- **Not Rust-validated.** Every measurement is from Python orchestration scripts with
  hand-written mutation batteries. Whether the per-crate cargo-mutants ratchet exhibits the
  same class is untested; the ratchet reports a surviving *count*, which by construction
  cannot show a per-mutant transition, so detecting this class there would need per-mutant
  status diffing that the current gate does not emit.
- **Not a proposal to gate on it.** No CI enforcement is proposed here. Kill-set diffing is
  a manual practice for deletions; automating it needs a stable per-mutant identity across
  runs, which the current tooling does not provide.
- **The detection method has a blind spot**: it only fires for guards whose target survives
  the deletion. A guard protecting something that is *also* being removed is correctly
  silent — and a guard removed while its target is quietly kept elsewhere would need the
  target's own mutant to still be in the spec to show up.

## 6. Verification status

Evidence grades differ per instance and are stated rather than blurred.

- **§3.1 and §3.2 — measured here.** Both `KILLED` → `SURVIVED` → `KILLED` transitions are
  reproducible from the named per-run logs.
- **§3.3 — structurally verified here; numerically not.** It reached this record as a relay,
  and the relay's wording was wrong in two particulars, both corrected above: it named the
  ambiguous `dispatch-plan.py` without the repo, and it counted one missing constant where
  there are two. The *structural* claim — that neither symbol appears in the module the
  consumer loads, and that the consumer probes for both — was checked directly against file
  contents at the two PR head SHAs before being recorded. The *frontier numbers* are an
  independent reviewer's replay, reproduced here as attribution, not as this record's own
  measurement; anyone depending on them should re-derive them.

That split is the point. The repeated failure this record exists to prevent is a claim that
outlived its evidence — and a relayed claim is exactly where that starts. Checking §3.3
before recording it is what turned up the two errors in its wording; had it been written up
as received, this record would itself have become an instance of the problem it describes.
