# Getting the ci-summary gate off the build-runner slot — design record (sq-90cv4 follow-up)

> Status: **§2 IMPLEMENTED IN SHADOW (bead sq-lfmvd), REQUIRED-CHECK MIGRATION NOT DONE.**
> The sq-90cv4 increment was the *adaptive saturation budget* in
> `scripts/ci_summary_gate.py`. The event-driven transport designed in §2 now exists —
> `.github/workflows/ci-summary-status.yml` + `scripts/ci_summary_status.py`, unit-tested
> in `scripts/tests/test_ci_summary_status.py` — publishing the `ci-summary-status`
> commit status ALONGSIDE the resident `gate` job. **The branch-protection ruleset is
> UNCHANGED: `gate` is still the single required check and `ci-summary-status` is
> required by nothing.** §3 below records how each design risk was resolved, including
> the two that are resolved only for the SHADOW stage. The runbook, parity exit criteria
> and maintainer sign-off step live in `docs/branch-protection.md` §*Event-driven
> aggregation (staged migration)*. Design authored by Claude Fable 5 `[FABLE-5]`;
> implementation + the §3 resolutions by Claude Sonnet 4.6 `[SONNET-4.6]`.

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

> `[SONNET-4.6]` Each risk below now carries a **RESOLVED** note recording what the
> implementation actually did. Two of them (3 and 4) are resolved only for the SHADOW
> stage and are the explicit blockers on dropping the resident job — they are restated as
> the "two liveness gaps" table in `docs/branch-protection.md`. Nothing here was resolved
> by weakening the verdict: the whole gating decision is still
> `ci_summary_gate.render_verdict` and friends, called from the new transport.

1. **Required-check migration.** Swapping the required check from a job to a status
   context must be atomic-ish: add the status context as required *alongside* the
   job first, prove parity on live PRs, then drop the job. A missing "expected"
   required check blocks every merge — stage it.

   **RESOLVED (staged, not swapped).** The implementation touches no ruleset at all.
   `ci-summary-status` is published as a shadow context while `gate` remains the sole
   required check; the ordered add-then-prove-then-drop runbook, the parity sample and
   the maintainer sign-off step are written down in `docs/branch-protection.md`. The
   resident job's continued existence is pinned by
   `test_ci_summary_status.py::TestWorkflowWiring::test_the_resident_gate_is_STAGED_ALONGSIDE_not_swapped`,
   so this record's promise cannot silently become false.
2. **Trust model changes (for the better, but verify).** `workflow_run` executes the
   workflow definition from the **default branch**, not the PR — the evaluator logic
   stops being PR-attested. Confirm the token scoping (`statuses: write` on the
   evaluator; the PR cannot forge the status).

   **RESOLVED, and the answer was NARROWER than this record assumed.** The design's own
   §3.3 seed contradicts §3.2: a `pull_request`-triggered job takes its definition from
   the PR HEAD, and on same-repo agent branches that head can edit the very code holding
   `statuses: write` — so a PR could forge a green status for the context destined to be
   THE required check (the #3474 threat model that moved the fast-fix ring into its own
   `workflow_run` workflow). The evaluator therefore has **no `pull_request` trigger**:
   only `workflow_run` + `workflow_dispatch`, both of which execute the default-branch
   copy. Token: `statuses: write` as the ONLY write capability; `checks`/`contents`/
   `actions` read-only. Pinned by `TestWorkflowWiring`.
3. **Bootstrap + quiet PRs.** A docs-only PR that triggers few/no sibling workflows
   needs at least one evaluation to publish the status (the current gate's
   stable-empty-set pass). A `pull_request`-triggered "seed" evaluation (short-lived,
   also non-resident) covers it, but its token cannot write statuses from forks —
   check the fork path.

   **RESOLVED FOR THE SHADOW STAGE ONLY — blocks dropping the resident job.** The seed
   is not implemented, for the trust reason in 2 (and the fork-token problem this record
   anticipated is a second, independent reason it would not have worked). While the
   resident gate exists there is no quiet head: `ci-summary` itself runs on every
   `pull_request`/`merge_group`/push, and its completion is a guaranteed `workflow_run`
   event, so every head gets at least one evaluation. After the job is dropped a
   genuinely quiet head would leave the context ABSENT — which branch protection treats
   as blocking, i.e. fail-closed, never a false green. Closing it needs a guaranteed
   always-running sibling or a periodic sweep. The fork path is covered on the
   `workflow_run` side: that run executes in the base repo with base-repo permissions,
   so a fork PR's own read-only token is irrelevant (to be CONFIRMED on live traffic —
   it is a parity exit criterion, not a claim this record can prove).
4. **`merge_group` compatibility.** The merge-queue ref needs the same status; verify
   `workflow_run` events fire for merge-group-triggered sibling runs and that the
   status lands on the merge-group head SHA the ruleset checks.

   **PARTIALLY RESOLVED — the remaining half is an OBSERVATION, not a code change.**
   The evaluator subscribes to every workflow completion with no `branches` filter and
   reads `github.event.workflow_run.head_sha`, which for a merge-group-triggered sibling
   is the merge-group ref's commit — the SHA the queue's required checks are evaluated
   against. That `workflow_run` fires for merge-group-triggered runs and that the status
   lands where the ruleset looks CANNOT be proven from inside this repo; both are listed
   as parity exit criteria in `docs/branch-protection.md` and must be observed on a real
   queued batch before step 1 of the migration.
5. **Event storms.** One evaluation per sibling-workflow completion is dozens per
   push; each is cheap, but debounce (concurrency-group per head SHA,
   `cancel-in-progress`) to keep it tidy.

   **RESOLVED, plus a hazard this record did not name.** `concurrency: group:
   ci-summary-status-<head sha>` with `cancel-in-progress: true`, as designed. The
   unnamed hazard: concurrent, unordered evaluations mean a SLOWER one that observed an
   earlier state can POST after a faster later one, leaving a green head reading
   `pending` with no further event coming. The evaluator therefore never downgrades an
   already-published terminal status back to `pending` (any genuinely new pending work
   arrives as a new run, whose completion fires its own evaluation). Pinned by
   `TestPublish`.
6. **Startup race parity.** The current MIN_POLLS floor guards against verdicting a
   partial early set. The evaluator equivalent: never publish a success while any
   *expected* workflow for the trigger set has not yet registered — reuse the
   path-filter outputs or require a minimum sibling count seen before a green status.

   **RESOLVED — a TIME floor, not a count floor.** A minimum sibling count would have to
   be maintained by hand for every trigger class, which is the brittleness ci-summary
   exists to avoid. Instead: an all-terminal set may not conclude SUCCESS until 180s
   after the EARLIEST workflow run on the head SHA (the clock GitHub starts when CI
   begins on that head; aggregator runs count toward the origin even though they are not
   judged). A FAILURE is never floored — a concluded gating failure in the authoritative
   newest run is never forgiven, so flooring it would only delay the red. The other two
   partial-set hazards are the resident gate's own holds, reused verbatim rather than
   re-derived: a draft-tier-assembled selection with no full-tier successor
   (`draft_selects_unsuperseded`) and a missing `feature-matrix report` verdict
   (`fm_report_status`) both hold the status at `pending`, and each of those late
   arrivals is itself a workflow completion, so the event stream replaces the poll loop.
   Past a 4020s absolute budget (the resident gate's own cap) an outstanding set is
   rendered by `render_verdict`, which fails closed on incomplete legs.

### 3.7 One risk this record missed: the aggregators must not judge each other

`[SONNET-4.6]` Not in the original list, and load-bearing. During staging the resident
`gate` job is *pending for as long as it polls*, and its check-run sits on the same head
SHA. If it stayed in the evaluated set, every shadow verdict would be a strictly-later
echo of the very thing it exists to be compared against — the parity evidence would be
worthless. Earlier evaluations of the same head are the same problem in miniature.
Resolution: both aggregators are excluded by workflow **FILE PATH** before the verdict
sees them (`WorkflowRunResolver` already drops the whole self-workflow by run id), so a
RENAMED gate job cannot re-enter the set. Deliberately NOT by check-run name: the
full-tier name `gate` is not name-excluded anywhere in this repo, because a future
sibling job literally called `gate` in another workflow must keep gating
(`ci_summary_gate.is_draft_gate_artifact`'s docstring records the same rule). Pinned by
`TestAggregatorScoping`, including an anti-vacuity twin showing the same fixture holds at
`pending` when the path is not recognised.

## 4. Verdict-function reuse

The sq-90cv4 extraction means the deeper fix does **not** re-implement semantics:
`is_advisory` / `is_self` / `render_verdict` and their tests
(`scripts/tests/test_ci_summary_gate.py`) are the shared, already-gated brain. The
follow-up only replaces the *transport* (resident loop → event-driven invocations).

`[SONNET-4.6]` **Held.** `scripts/ci_summary_status.py` contains no pass/fail rule of its
own: `render_verdict` renders every verdict, `failfast_failures` decides every early red,
`draft_selects_unsuperseded` / `fm_report_status` supply both holds, `is_advisory` decides
every exclusion, and `WorkflowRunResolver` does all newest-run resolution.
`scripts/tests/test_ci_summary_status.py` therefore tests only the transport and states
that scope explicitly, so the two suites do not drift into two half-copies of one rule.
The one deliberate NON-reuse is the cancelled-run re-dispatch: the resolver's re-dispatch
hook is wired to a function that refuses, because the evaluator holds read-only `actions`
(see the second liveness gap).

## 5. Status

`[SONNET-4.6]` **§2 shipped in shadow (bead sq-lfmvd); the required-check migration is
NOT done and needs maintainer sign-off on the ruleset edit.** The resident gate remains
the authority and the adaptive budget remains the operative mitigation for saturation.
Next actions, in order: run the parity sample in `docs/branch-protection.md`; add
`ci-summary-status` alongside `gate` in the ruleset; close the two liveness gaps
(quiet-head trigger, re-dispatch capability); then drop the resident job and its advisory
declaration in one change.
