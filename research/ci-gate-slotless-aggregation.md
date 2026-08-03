# Getting the ci-summary gate off the build-runner slot — design record (sq-90cv4 follow-up)

> Status: **DESIGN-ONLY.** The shipped sq-90cv4 increment is the *adaptive saturation
> budget* in `scripts/ci_summary_gate.py` (unit-tested; see the PR that added this
> record). This record documents the **deferred deeper fix** — removing the gate's
> slot occupancy entirely — so the follow-up bead starts from a vetted design instead
> of re-deriving one against the merge-critical required check. Authored by Claude
> Fable 5 `[FABLE-5]`.

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
7. **Trigger subscription.** The evaluator wants to fire for *every* sibling workflow,
   which means a `workflow_run` trigger with no `workflows:` filter — a construct whose
   routing behaviour GitHub does not document. Resolved as far as static evidence
   allows in §6; the residue is a named stage-1 observation that stage 2 is blocked on.

## 4. Verdict-function reuse

The sq-90cv4 extraction means the deeper fix does **not** re-implement semantics:
`is_advisory` / `is_self` / `render_verdict` and their tests
(`scripts/tests/test_ci_summary_gate.py`) are the shared, already-gated brain. The
follow-up only replaces the *transport* (resident loop → event-driven invocations).

## 5. Status

Tracked as a follow-up bead (created with sq-90cv4's PR; blocked on maintainer
appetite for a required-check migration). Until then the adaptive budget is the
operative mitigation, and the merge-queue/org plans reduce exposure.

The stage-1 implementation (bead sq-lfmvd — `.github/workflows/ci-summary-status.yml`
plus the `--evaluate` mode of `scripts/ci_summary_gate.py`) is in flight and is **not
on `main`** as of §6 below.

## 6. Trigger subscription — `workflow_run` with no `workflows:` filter [OPUS-5]

The stage-1 implementation deliberately omits `workflows:` from its `workflow_run`
trigger, so the aggregator never hard-codes a sibling list, and records that omission
as HONEST UNCERTAINTY to be confirmed on the first PR after it merges. Issue #5654
contributed a new datum — `actionlint 1.7.7` rejects the construct outright
(`no workflow is configured for "workflow_run" event`) — and asked for the question to
be decided *before* `ci-summary/status` is added to the required contexts. This section
is that decision and the evidence behind it.

### What the static evidence settles, and what it does not

- **GitHub's own parser accepts it.** `actions/languageservices`
  (`workflow-parser/src/workflow-v1.0.json`, the schema behind GitHub's workflow
  editor validation) defines `workflow-run` as `one-of: [null, workflow-run-mapping]`,
  and `workflow-run-mapping` declares `types` / `workflows` / `branches` /
  `branches-ignore` with **no required property**. Omitting `workflows:` — indeed
  writing a bare `workflow_run:` with no mapping at all — is valid to the parser GitHub
  itself ships. SchemaStore's `github-workflow.json` agrees: `workflows` carries
  `minItems: 1` *if present* and appears in no `required` list.
- **actionlint is stricter than GitHub, not a witness against it.** The error is
  actionlint's own rule, not a schema violation: `rule_events.go` errors whenever
  `len(event.Workflows) == 0` for `workflow_run`, unconditionally. It is a lint-level
  opinion about a construct GitHub's parser permits.
- **The docs neither bless nor forbid the omission.** The syntax reference has an
  `on.workflow_run.<branches|branches-ignore>` entry but **no**
  `on.workflow_run.workflows` entry, and every documented example specifies
  `workflows:`. The docs state the default for `types` explicitly ("By default, all
  activity types trigger workflows that run on this event") and state no default for
  `workflows`. That asymmetry is suggestive, not decisive.
- **This repo has no precedent to cite.** All six workflows that trigger on
  `workflow_run` — `batch-merge`, `fast-fix-ring`, `feature-matrix-report`,
  `pr-backlog`, `selection-alarm`, `verdict-bridge` — name `workflows:`. Nothing here
  has ever exercised the unfiltered form.

So **validity is settled** (GitHub accepts the file; actionlint's finding warrants no
change to it, and no lane may be added that turns that opinion into a red without an
explicit waiver for this file), and **routing is not**: whether an absent `workflows:`
means *every* workflow or *no* workflow is not answerable from any artifact reachable
without running it, and the failure mode stays silent.

### The decision

1. Ship stage 1 with the unfiltered trigger. It is valid, and the cost of being wrong
   at stage 1 is bounded — `ci-summary / gate` is still the only required context, so a
   non-firing evaluator publishes nothing and blocks nothing.
2. The stage-1 observation gets a **named criterion** rather than "look at the Actions
   tab". PASS: on a PR that touches no `.github/` file, `ci-summary-status` runs appear
   on the head for events originating from **at least two distinct triggering
   workflows** (distinct `github.event.workflow_run.name`). FAIL: no `ci-summary-status`
   run appears at all on a head whose siblings demonstrably ran.
3. **Stage 2 — adding `ci-summary/status` to the ruleset's required contexts — is
   BLOCKED on a PASS.** A required context that is never reported blocks every merge,
   which is exactly the cost of getting this wrong, so the observation is a gate rather
   than a note.
4. **Pre-committed fallback on FAIL**, recorded now so it is not re-litigated under
   merge pressure: enumerate `workflows:`, but pair the list with a **mechanical
   completeness check** — a `scripts/` gate that parses `.github/workflows/*.yml` and
   reds when a workflow that can run on a PR head is missing from the list. The
   implementation's objection to enumeration is that it under-aggregates *silently*; a
   checked list removes the silence, which is the property that actually matters. Do
   not hand-maintain the list.
5. **Rejected fallbacks**, recorded so they are not proposed again: `check_run` and
   `check_suite` cannot carry this transport. The docs are explicit that the event
   "does not trigger workflows if the check run's check suite was created by GitHub
   Actions or if the check suite's head SHA is associated with GitHub Actions" — which
   excludes precisely the check-runs being aggregated — and their `GITHUB_SHA` is the
   default branch's head, not the head under evaluation.

Two further documented `workflow_run` constraints the implementation should be re-read
against, both of which the in-flight file satisfies as written: it only fires for a
workflow file that exists on the **default branch** (the same property the trust model
relies on), and `workflow_run` chains are capped at three levels (the evaluator's
recursion guard keeps its chain at depth one, well inside the cap).
