# Getting the ci-summary gate off the build-runner slot — design record (sq-90cv4 follow-up)

> Status: **PARTLY IMPLEMENTED (stage 1 of 3).** [OPUS-5] sq-lfmvd shipped the §2
> transport — `.github/workflows/ci-summary-status.yml` plus
> `scripts/ci_summary_gate.py --evaluate` — publishing the `ci-summary/status` commit
> status *alongside* the existing `ci-summary / gate` job. **The required-check set is
> unchanged**: stages 2 (add the status as required) and 3 (drop the job) are ruleset
> edits that need maintainer sign-off and are tracked in
> `docs/branch-protection.md` §Slotless status migration. See §3 below for how each
> design risk was resolved, and §6 for what the implementation deliberately did *not*
> resolve. The sq-90cv4 increment this record was written as a follow-up to — the
> *adaptive saturation budget* — remains in place and is unaffected. Originally
> authored by Claude Fable 5 `[FABLE-5]`.

## 1. Problem

`ci-summary / gate` (the single REQUIRED branch-protection check) is a *waiter*: a
GitHub-hosted job that polls sibling check-runs until they are all terminal. While
polling it occupies one job slot in the account-wide hosted-runner concurrency pool.
Under load (many open PRs, each with its own gate) the waiters collectively starve
the build jobs they are waiting on; observed 2026-07-02 as a congestion collapse with
false `gate=FAILURE` verdicts on PRs whose real legs were green or merely queued.

The adaptive budget fixes the *false verdict* (queued-under-saturation now extends
the wait instead of concluding FAILURE; a genuine hang still fails). It does **not**
fix the *slot occupancy* — a saturated pool now holds gate slots longer, which is the
correct trade for verdict honesty but leaves throughput on the table.

## 2. Recommended deeper fix: event-driven evaluation, no resident waiter

Replace the resident poll loop with **short-lived evaluations triggered by sibling
completion events**:

- A `workflow_run` (`types: [completed]`) triggered workflow fires each time any
  sibling workflow finishes. Each firing runs for seconds: fetch the head commit's
  check-runs once, apply the *same* verdict function (`render_verdict` in
  `scripts/ci_summary_gate.py` — already extracted and unit-tested), and publish the
  result as a **commit status** (`statuses: write`) on the head SHA.
- Branch protection then requires that commit-status context instead of the
  `ci-summary / gate` job. The status is "pending" until the evaluator sees an
  all-terminal set, then success/failure by the existing semantics.
- Slot cost: O(seconds) per sibling completion instead of O(minutes-to-an-hour) per
  PR. No waiter exists to starve the pool, so the saturation false-RED class
  disappears structurally rather than being compensated for.

Why this shape and not the alternatives:

- **Concurrency-exempt / self-hosted tiny runner for the gate** — works, but needs
  org/runner configuration outside the repo (runner registration, labels, upkeep)
  and adds an always-on machine to the trust surface. Needs the maintainer; not a
  repo-only change.
- **Merge queue + org move** (already planned) reduces how many PR heads run CI
  concurrently but does not remove the waiter; complementary, not a substitute.
- **Keeping the poll loop but shortening it** regresses the verdict semantics the
  current design guarantees (waits for late-registering workflows, settle window).

## 3. Design risks the implementation bead must resolve

1. **Required-check migration.** Swapping the required check from a job to a status
   context must be atomic-ish: add the status context as required *alongside* the
   job first, prove parity on live PRs, then drop the job. A missing "expected"
   required check blocks every merge — stage it.
2. **Trust model changes (for the better, but verify).** `workflow_run` executes the
   workflow definition from the **default branch**, not the PR — the evaluator logic
   stops being PR-attested. Confirm the token scoping (`statuses: write` on the
   evaluator; the PR cannot forge the status).
3. **Bootstrap + quiet PRs.** A docs-only PR that triggers few/no sibling workflows
   needs at least one evaluation to publish the status (the current gate's
   stable-empty-set pass). A `pull_request`-triggered "seed" evaluation (short-lived,
   also non-resident) covers it, but its token cannot write statuses from forks —
   check the fork path.
4. **`merge_group` compatibility.** The merge-queue ref needs the same status; verify
   `workflow_run` events fire for merge-group-triggered sibling runs and that the
   status lands on the merge-group head SHA the ruleset checks.
5. **Event storms.** One evaluation per sibling-workflow completion is dozens per
   push; each is cheap, but debounce (concurrency-group per head SHA,
   `cancel-in-progress`) to keep it tidy.
6. **Startup race parity.** The current MIN_POLLS floor guards against verdicting a
   partial early set. The evaluator equivalent: never publish a success while any
   *expected* workflow for the trigger set has not yet registered — reuse the
   path-filter outputs or require a minimum sibling count seen before a green status.

## 4. Verdict-function reuse

The sq-90cv4 extraction means the deeper fix does **not** re-implement semantics:
`is_advisory` / `is_self` / `render_verdict` and their tests
(`scripts/tests/test_ci_summary_gate.py`) are the shared, already-gated brain. The
follow-up only replaces the *transport* (resident loop → event-driven invocations).

## 5. Status

Tracked as a follow-up bead (created with sq-90cv4's PR; blocked on maintainer
appetite for a required-check migration). Until then the adaptive budget is the
operative mitigation, and the merge-queue/org plans reduce exposure.

[OPUS-5] **sq-lfmvd implemented §2 as stage 1 of the §3.1 staged migration** — see the
Status note at the top. §6 below records how each §3 risk was actually resolved and
where the implementation differs from what this record assumed.

## 6. How §3 was resolved in practice (sq-lfmvd) — including two corrections

[OPUS-5] Each §3 risk, and the resolution as shipped:

1. **Required-check migration** — staged exactly as §3.1 prescribes. Nothing in the
   implementation touches the ruleset (a workflow cannot edit its own repository
   ruleset); the three stages are tabulated in `docs/branch-protection.md`.
2. **Trust model — this record's §3.2 UNDERSTATED the risk, and §3.3's suggested shape
   was unsafe.** §3.3 proposed a `pull_request`-triggered seed. That would have been a
   privilege-escalation hole of the exact class #3474 closed for `fast-fix-ring.yml`:
   for a `pull_request` event GitHub takes the workflow **definition from the PR head**,
   and same-repo agent branches receive repository credentials — so a PR could have
   edited the seed job to publish a forged `success` on its own head, against a context
   that stage 2 makes merge-authorising. The implementation therefore has **no
   `pull_request` trigger at all**: the seed is the `requested` half of
   `workflow_run: types: [requested, completed]`, which is equally default-branch-defined.
   That also disposes of §3.3's open "check the fork path" question by construction —
   the token belongs to the default-branch run, not to the fork's PR run.
3. **Bootstrap + quiet PRs** — covered by the `requested` transport plus the fact that
   `ci-summary` itself triggers on every pull_request, so even a docs-only head
   produces at least one requested + one completed event. The `requested` transport is
   restricted to publishing `pending` (a just-created run's check-runs have not
   registered), and publishes *nothing* over an all-terminal set so a late seed can
   never downgrade an already-published success.
4. **`merge_group`** — the evaluator keys entirely off `workflow_run.head_sha`, which
   for a merge_group-triggered sibling is the merge-group ref's commit. **Not yet
   observed live**; confirming it is part of stage 1's parity observation.
5. **Event storms** — debounced by a per-head-SHA concurrency group with
   `cancel-in-progress: **false**`, not `true` as §3.5 suggested. `true` is wrong twice
   over here: it can kill an evaluation mid-publish, and a cancelled run leaves a
   `cancelled` check-run on the head SHA — which, with no same-name successor,
   `forgive_superseded` cannot excuse and which would RED the legacy gate. With `false`,
   GitHub still collapses a storm (at most one PENDING run per group) while the final
   event's evaluation always executes.
6. **Startup-race parity** — three guards rather than §3.6's single one, each of which
   can only downgrade an answer toward `pending`: (a) a `completed` evaluation refuses
   to conclude until the triggering run is visible in the fetched snapshot; (b) a
   would-be terminal verdict is re-derived from a fresh fetch after a settle pause and
   must match; (c) the `requested` transport is pending-only. §3.6's suggested
   "minimum sibling count" was rejected — a magic number would be exactly the kind of
   name/count fragility #3773 removed from the advisory rule.

**A risk §3 did not list**, found during implementation: the evaluator's *own* check-run
lands on the aggregated commit (a `workflow_run` run inherits the triggering run's head
SHA — empirically recorded for `nightly selection-bug alarm` in the advisory registry).
Left alone it would make the legacy `gate` wait on, and potentially RED on, a job whose
only work is to observe that gate. Resolved by `is_status_evaluator_artifact()`, an
exact whole-name self-exclusion in the same class as `gate, draft-tier` — deliberately
*not* an advisory-registry declaration, since the registry declares jobs that report a
real property, and a declaration could not have fixed the pending-hold half (`pending`
is counted before the advisory filter).

**Unverified, and it fails silently:** the evaluator's `workflow_run` trigger carries
no `workflows:` filter, so that it subscribes to *all* workflows is the documented
reading rather than an observed fact in this repo. If it is wrong the evaluator never
fires and no status is ever published — harmless while `gate` is the required check,
but it is the first thing stage-1 observation must confirm, and the fix is not to
enumerate workflows (that reintroduces the under-aggregation this whole design avoids).

**Still open before stage 3** (both fail closed to `pending` today, both recorded in
`docs/branch-protection.md`): the evaluator holds a draft PR at `pending` rather than
reproducing the draft-tier evaluation, and it cannot perform the waiter's bounded
once-only re-dispatch of a cancelled newest run — dropping the `gate` job also drops
that retry, so it needs a home first.
