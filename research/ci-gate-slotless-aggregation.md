# Getting the ci-summary gate off the build-runner slot — design record (sq-90cv4 follow-up)

> Status: **DESIGN + PHASE 1 LANDED.** The shipped sq-90cv4 increment is the *adaptive
> saturation budget* in `scripts/ci_summary_gate.py` (unit-tested; see the PR that added
> this record). This record documents the **deferred deeper fix** — removing the gate's
> slot occupancy entirely — so the follow-up bead starts from a vetted design instead
> of re-deriving one against the merge-critical required check. Authored by Claude
> Fable 5 `[FABLE-5]`.
>
> **Update (sq-6vshe.19, 2026-07-31) `[SONNET-4.6]`: Phase 1 — the transport seam — is
> implemented; Phase 2 — the transport switch — is NOT, and is still blocked on the
> maintainer. §6 below records exactly what landed, what the bead's hard constraint
> changed about §2, and what must be true before anything gates on it. The resident
> waiter is still the transport in `ci-summary.yml`; no runner slot has been freed
> yet.**

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

## 6. sq-6vshe.19 — Phase 1 (the transport seam) `[SONNET-4.6]`

### 6.1 The bead's hard constraint changes §2's shape

The bead states: **the required check stays exactly `gate`** (the branch-protection
ruleset names that context). §2 above proposed publishing a **commit status** and
re-pointing branch protection at it — that is a required-check *rename*, which the
constraint forbids. Two ways to satisfy both, neither costed here:

* **(i) Checks API, same name.** The evaluator `POST`s/`PATCH`es a check-run literally
  named `gate` (`checks: write`), leaving it `in_progress` while siblings are
  outstanding and concluding it when the verdict renders. Branch protection is
  untouched. This ALSO gives the state carrier for free (the check-run's `external_id`
  / `output` persists across invocations, which is the "check-run state" the bead
  names). Cost: the gate's self-exclusion is currently anchored on
  `details_url`'s `/runs/<SELF_RUN_ID>/` — an API-created check-run has no such URL, so
  `is_self` needs a second, name+`external_id`-based rule, and it must be at least as
  tight (a sibling literally named `gate` must not be able to self-exclude).
* **(ii) Commit status, same context.** GitHub matches a required context against both
  check-runs and commit statuses, so a status with context `gate` would satisfy
  `{context: "gate"}` without a ruleset edit. Rejected as the primary: during any
  overlap a check-run `gate` and a status `gate` both exist and which one branch
  protection honours is not something this repo should be discovering empirically on
  the check that authorises every merge.

**Recommendation: (i), staged behind §3.1's add-alongside-then-cut sequence.**

### 6.2 What Phase 1 actually landed

`scripts/ci_summary_gate.py` — no behaviour change, one structural change:

| before | after |
|---|---|
| `run_gate` = one ~340-line poll loop; all cross-poll state in locals | `evaluate_once(cfg, state, …) -> PollOutcome` (one evaluation), `GateState` (the former locals, JSON-serializable), `render_budget_exhausted(cfg, state, …)` (the end-of-budget render) |
| — | `run_gate` is a thin driver over those three |
| — | `run_gate_once` + CLI `--oneshot` + `GATE_STATE_PATH`: one evaluation, state persisted, exit `0`/`1`/`75` (`STILL_SETTLING_EXIT`) |

The INVARIANT the bead demands — *gate verdict semantics bit-identical for every
scenario in the existing unit tests* — is met **structurally**: both transports call
the same `evaluate_once`, so there is no second implementation to diverge. All 183
pre-existing tests pass unchanged. What genuinely needed proving is that the state
CARRIER is complete — that no decision input was left in a process-local — and
`TestSlotlessTransportSeam` proves it by driving `run_gate_once` through a JSON file
and asserting three-way parity with the resident loop (**same verdict, same number of
observations, same narration**) across the load-bearing scenarios. Verdict-only parity
was measurably too weak: a carrier that forgets `stable` still reaches the same exit
code via the #997 graceful-timeout path, having burned the whole budget. Each of the
eight decision-INPUT fields was mutation-checked individually — deleting any one from
`PERSISTED` REDs the suite.

The ninth persisted field, `terminal_exit`, is the one decision **output**, and it is
what makes the carrier a durable DECISION record rather than merely a progress record.
`run_gate_once` stamps every verdict it renders — clean convergence, fail-fast, the
consecutive-fetch-failure cap, a genuine hang, the unsatisfiable draft-tier hold, the
absolute-budget render — and re-returns a stamped verdict verbatim, without fetching,
re-rendering or advancing the budget. Without it only the *budget-exhaustion* verdict
was protected (by the `attempt >= max_total_polls` refusal), and every verdict reached
below the budget — which is most of them — persisted a record indistinguishable from a
still-settling one, so a duplicate delivery, a delayed event or a retry after an
ambiguous transport failure could re-evaluate over a changed sibling set and render a
DIFFERENT verdict, including green after an earlier red. The parity scenarios cannot
pin this (they stop at the first verdict); `test_early_red_cannot_be_reopened_green_by_a_duplicate_delivery`
does, with the symmetric early-green case and a per-path stamping test beside it. Two
honest bounds: the marker is only as durable as the carrier (a lost or whole-rejected
record restarts the window and re-observes from scratch — a re-run, not a re-decision),
and it serialises SEQUENTIAL duplicates only, not the concurrent read-modify-write of
§6.3(3).

The carrier is also validated as **one atomic, versioned record** — every persisted
field present, typed, in range, and consistent with the others (`stable`/`unsat_polls`
cannot exceed the observations `completed_hist` records; `attempt` cannot be smaller
than the observations it must account for) — or it is rejected WHOLE and the window
restarts. A field-by-field merge was unsound in the shortening direction: a partial
`{"attempt": max_total_polls - 1}` preserved an almost-exhausted budget while resetting
the coupled settle state, so a single terminal-green observation reached the budget with
the settle condition unmet and `render_budget_exhausted`'s pending-zero graceful-timeout
path rendered that one observation GREEN. Writes are temp-file-plus-`os.replace` so an
evicted runner leaves the previous record rather than a valid-JSON prefix. Both
properties are pinned red-on-wrong-answer by
`test_partial_state_cannot_manufacture_a_premature_pass` and
`test_save_state_file_is_atomic_and_leaves_no_debris`.

`--oneshot` is wired to **nothing**. `ci-summary.yml` still runs the resident loop, and
`test_resident_transport_is_still_the_one_wired_to_the_required_check` REDs if that
changes.

### 6.3 What Phase 2 must resolve before it can gate

Beyond §3's six risks (all still open), the seam surfaced three that are properties of
the *transport*, not of the verdict, and so could not be settled in the script:

1. **The budgets are poll counts, not wall-clock.** `min_polls` (3), `base_polls`
   (110) and `max_total_polls` (155) were calibrated against a fixed 20 s cadence to
   mean ~37 m / ~67 m. Under event-driven invocation the cadence is whatever the
   sibling-completion stream does, so those numbers stop meaning any duration at all —
   a quiet PR could exhaust `min_polls` in three seconds, and a busy one could see 155
   completions well inside the intended window. **They must become wall-clock deadlines
   carried in the state.** This is the single largest correctness item in Phase 2.
2. **The once-only re-dispatch guard is process-local.** `WorkflowRunResolver` keeps an
   in-process attempted-set as the API-lag belt on #3505's bounded re-run of a newest
   *cancelled* workflow. It does not survive a process exit; only the durable
   server-side `run_attempt` marker does. A non-resident transport can therefore
   re-`POST` during API lag. Either carry the attempted-set in `GateState` or make the
   `run_attempt` check strictly sufficient.
3. **Concurrent invocations.** Sibling completions arrive in bursts, so two evaluations
   can overlap and read-modify-write one state. Needs a per-head-SHA concurrency group
   (cheap, `cancel-in-progress: false`) or a compare-and-set carrier — and note this
   interacts with (1): serialising evaluations behind a concurrency group re-introduces
   queueing latency between them. The `terminal_exit` marker (§6.2) closes the
   *sequential* duplicate/retried delivery independently of this, but two evaluations
   that both load a pre-verdict record before either saves can still both evaluate.

Also unresolved from §3.3: a `pull_request` seed evaluation is still required for the
stable-empty-set pass (a docs-only PR whose sibling set is empty produces no completion
event to trigger on), and the fork path cannot write checks/statuses.

### 6.4 Honest accounting of the saving

**Phase 1 frees zero runner slots.** The estimate the bead carries (3–6 slots during a
drain) is entirely Phase 2's, and it is a projection from the 2026-07-10 profile
(`research/ci-mergequeue-speedup-2026-07.md` §2.1), not a measurement — and that
record's own §6 re-profile caveat has already fired once for the lane set. Phase 2
should re-measure the entry wall before claiming any number.
