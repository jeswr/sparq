# Does a `schedule`-triggered run's check-run belong in the push gate's sibling set? — decision record

> **Status: DECISION, DESIGN-ONLY — no gate code changed by this record.** [OPUS-5] 🤖 SPARQ agent.
> Answers the question issue **#5352** raised and that the fix for **#3786** deliberately
> left open. The implementation is a change to `scripts/ci_summary_gate.py`, i.e. to the
> single required branch-protection check, and is specified in §6 for its own reviewed PR.
> Related: #3773 (advisory must be DECLARED), #3783, #3785, sq-huwr8 / #4558.

## 1. The question

`ci-summary`'s `gate` job discovers its sibling set **by head SHA**: every check-run on the
commit under evaluation, whatever produced it. `kani.yml` is `schedule` + `workflow_dispatch`
only — it has no `push`/`pull_request` trigger — yet its check-runs land on `main`'s head SHA
whenever the cron fires, because a scheduled run always executes against the default branch's
current head. If a push-to-`main` gate is polling that same SHA, the cron's legs are injected
into its sibling set mid-poll, re-arming the settle window.

#3786 made that *particular* incident harmless from one end: the loop no longer **awaits** legs
whose conclusion `render_verdict` excludes by construction (the DECLARED-advisory set), and
`kani …` is declared. It explicitly did not answer the general question:

> should `ci-summary` discover check-runs produced by a workflow run whose triggering **event**
> is unrelated to the event this gate is evaluating?

## 2. What is true today (not a hypothesis)

The mechanism is **measured in-repo**, not inferred. sq-huwr8 / #4558 (authored by the
maintainer) ran `render_verdict()` over the real, paginated 477-check-run set for `main` head
`cb0c6739` and found the three alarm lanes — all `schedule`/`workflow_run`-only — inside the
**gating** set, REDding `main`'s push gate. Its commit message states the general fact plainly:

> `ci-summary.yml` ALSO runs on `push: branches: [main]` …, and a scheduled (or `workflow_run`)
> alarm run on main puts its check-run on exactly the head SHA that push-triggered gate is
> polling. The gate actively waits for the name set to stabilise, so a check-run appearing
> mid-poll is picked up, not missed.

Three consequences follow, and only the first is closed:

1. **Latency / settle re-arm.** A *pending* cron leg holds the settle window. Closed by #3786
   **only for DECLARED-advisory legs**; an undeclared cron leg still holds it.
2. **Attribution.** A cron leg that *fails* and is not declared advisory REDs the push gate for
   a commit whose content had nothing to do with the failure — and, since the fail-fast change,
   REDs it immediately. `AGENTS.md` treats a red `main` as a stop-the-line condition, so this is
   a manufactured outage.
3. **Supersession.** For a workflow with *both* `push: main` and `schedule` triggers (`ci.yml`,
   `bench.yml`, `fuzz.yml`, `codeql.yml`, `container-scan.yml`, `scorecard.yml`,
   `xpath-differential.yml`, `zk-toolchain.yml`), a cron run created *after* the push run on the
   same head SHA becomes the newest run for that `workflow_id`, so `resolve_newest_workflow_runs`
   treats the **push run's legs as superseded artifacts** and judges the cron run's legs instead.
   Benign when both run the same code on the same SHA, but it is not a property anyone designed.

### 2.1 Current exposure — schedule-triggered jobs that GATE `main`'s push gate

Every job below lives in a workflow with **no** `push`/`pull_request`/`merge_group` trigger and
has **no** entry in `.github/advisory-registry.json`, so by the #3773 rule it gates. Derived by
cross-reading `.github/workflows/*.yml` against the registry on this checkout (17 job
definitions; the check-run count is larger — `miri` alone is a 24-shard matrix):

| lane | job (check-run name) | `cron` |
| --- | --- | --- |
| `merge-group-watchdog.yml` | `recover zero-dispatch merge groups` | every 5 min (`timeout-minutes: 15`) |
| `promote-on-approval.yml` | `promote` | every 10 min |
| `rearm-sweeper.yml` | `restore dropped arms + sweep stuck arms` | every 10 min |
| `verdict-bridge.yml` | `bridge verdict comments to review labels` | every 10 min |
| `retriage.yml` | `retriage` | every 30 min |
| `triage-area.yml` | `clear the needs:area park` | twice hourly |
| `pr-backlog.yml` | `groom` | every 4 h |
| `miri.yml` | `miri (… shard N/24)`, `miri doctests (sparq-core)` | nightly |
| `differential.yml` | `nightly differential (sparq vs Oxigraph, …)` | nightly |
| `differential-update.yml` | `nightly UPDATE differential (…)` | nightly |
| `metamorph.yml` | `nightly metamorphic TLP/NoREC (…)` | nightly |
| `datalog-souffle.yml` | `datalog differential vs Soufflé` | nightly |
| `shacl-diff-fuzz.yml` | `SHACL differential fuzz (…)` | nightly |
| `dependency-monitoring.yml` | `cargo-deny advisories -> tracking issue` | nightly |
| `drift-scan.yml` | `drift-scan` | weekly |
| `slsa-builder-pin-review.yml` | `trusted-builder tag pin -> quarterly review issue` | quarterly |

The top of that table is the load-bearing part: the orchestration bots fire on a 5–30 minute
cadence and are exactly the jobs most prone to transient `gh`/API failure, so consequence (2)
is a routine exposure on `main`, not a nightly coincidence. `merge-group-watchdog`'s `watch`
job may run for minutes (its resweep loop sleeps between passes, ceiling 15 min) on a 5-minute
cadence, so consequence (1) is close to a standing condition for a `main` push gate. **This
duty cycle is predicted from the configuration, not measured** — measuring it is follow-up (F1).

Several of these lanes carry a header asserting the opposite, e.g. `differential.yml` and
`datalog-souffle.yml`: *"this workflow never produces a check-run on a PR head commit, so the
`ci-summary / gate` aggregator never sees it"*. The premise is true and the conclusion does not
follow — the identical error sq-huwr8 refuted for the alarm lanes. Correcting those headers is
follow-up (F3).

### 2.2 Which gate runs can be polluted

Only the **`push: branches: [main]`** run. A scheduled run's head SHA is the default branch's
head, which is never a PR head (a PR must be ahead of its base) and never a `merge_group` ref
(a fresh temporary commit). `workflow_dispatch` is different: it runs against a **chosen** ref,
so a dispatch on a PR branch *does* land in that PR's gate — deliberately, by someone naming
that ref.

## 3. Why the naive form of the fix is unsound

The obvious implementation — keep only siblings whose producing run's `event` equals the event
this gate is evaluating (`push`/`pull_request`/`merge_group`) — **must be rejected**. The gate
structurally *depends* on a check-run produced by a run with a different triggering event:

- `feature-matrix-report.yml` is `workflow_run`-triggered by design, and that placement is a
  security requirement (PR #3511 finding 1 — the privileged `checks: write` token must never be
  reachable from PR-head-controlled content). It posts the per-leg `opt-in <name>` check-runs
  **and** the `feature-matrix report` summary that `fm_report_status()` / `awaiting_report`
  block the settle window on.
- An event-equality filter would drop all of them: the gate would stop awaiting the reporter it
  is required to await, and the entire per-leg feature-matrix gating evidence would silently
  leave the gating set. That is a strictly worse hole than the one being closed.
- `fast-fix-ring.yml` and `selection-alarm.yml` are also `workflow_run`; `ci-select.yml` and
  `build-matrix.yml` are `workflow_call` (their jobs inherit the **caller's** event, so they are
  unaffected either way).

So the axis is **not** "same event". It is "was this run *about the ref under test*".

## 4. Options considered

**(A) Do nothing beyond #3786.** Leaves §2 consequences (1) and (2) fully open for the 17
undeclared jobs in §2.1. Rejected.

**(B) Declare every cron-only job advisory.** Reuses the #3773 registry, is per-job and
diff-visible, and needs no gate change. **Rejected**, for two reasons: (i) it is a semantic
mislabel — `miri`, `differential`, `metamorph` are hard lanes whose conclusions genuinely
matter; the registry key asserts *"this check's conclusion never gates"*, when the true claim is
*"this check never legitimately appears on a gated ref"*; (ii) it creates the mirror image of
the #3773 hole — the entry outlives the trigger configuration, so the day someone gives
`miri.yml` a `pull_request` trigger the lane is silently non-gating, with no diff on the gate
and no C4 complaint (C4 binds the key to `workflow` + `job_id`, not to the trigger set).

**(C) Event-equality filter.** Unsound — §3. Rejected.

**(D) Exclude runs whose `event` is exactly `schedule`.** Narrow, per-**run** (not per-workflow),
and orthogonal to the registry. Accepted — see §5.

## 5. Decision

**Exclude a check-run from the gate's sibling set iff it is positively attributable to a
workflow run whose `event` is exactly `schedule`. Nothing else is excluded on event grounds —
in particular `workflow_dispatch`, `workflow_run`, `workflow_call` and `release` runs keep
gating exactly as today.**

The principle, stated so it can be checked: **a scheduled run did not choose its ref.** GitHub
selects the default branch's current head for it, so its presence on the commit under
evaluation is a timing coincidence and carries no information about the push/PR being gated.
Every other trigger names a ref — a push, a PR head, a queue ref, a dispatcher's `ref` input, a
`workflow_run`'s `head_sha` — and therefore *is* about that commit.

Why this is safe where option (C) is not, and where option (B) is not:

- **Per-run, not per-workflow.** A workflow with both `pull_request` and `schedule` triggers
  keeps gating on its PR run; only the cron run is dropped. There is no way to make a lane
  non-gating by editing its triggers: a schedule-only lane cannot produce a PR-head check-run
  in the first place, so nothing is lost that was ever gating a merge.
- **Nothing structurally awaited is dropped.** The reporter chain is `workflow_run`; the
  selection pre-job is `workflow_call`. Neither is `schedule`.
- **No promote-time hole.** Give `miri.yml` a `pull_request` trigger and it gates immediately,
  with no registry edit and no way to forget one — the opposite of option (B).
- **The registry is not made redundant, and the two mechanisms stay separable.** The advisory
  registry answers *"should the gate ever judge this check's conclusion?"* (semantic, per-job,
  declared, reviewable). Event scope answers *"was this check-run produced by a run about this
  ref?"* (provenance, per-run, structural). Conflating them is what forced option (B)'s
  mislabel. `selection-alarm` (`workflow_run`) still needs its declaration; the schedule-only
  alarms keep theirs as defence in depth.
- **It fixes §2 consequence (3) too**: with cron runs out of the set, a nightly `ci.yml` run can
  no longer supersede the push run's legs on the same head SHA.

**It is not a silent exclusion.** The #3773 lesson is that the one check authorising merges must
never drop a leg without a record. So the verdict and the per-poll log must **name every
check-run excluded as cron-injected, with its producing run id** (§6.4). Unlike the old name
rule, the discriminator here is a server-supplied run property, not an author-chosen display
string.

### 5.1 Accepted residual risk, stated honestly

- A `workflow_dispatch` of a heavy lane onto a PR branch, or onto `main`, still gates. That is
  deliberate: someone named that ref. If it becomes a nuisance for a specific lane, the answer
  is that lane's registry entry, not a wider event filter.
- A cron lane's genuine failure no longer shows up on `main`'s push gate. That signal must
  already reach a human by another route — the alarm lanes (`formal-alarm`, `ci-latency-alarm`),
  `nightly-full-sweep`'s `route-failures`, and each lane's own red run. Follow-up (F2) is to
  confirm that routing exists for each lane in §2.1 **before** this lands; a lane whose only
  escalation path was "it reds main by accident" has no escalation path.
- Excluding cron legs slightly **widens the empty-set pass** on the push path, because today
  those legs accidentally hold the set open while the push's own legs register. §6.2 specifies
  the belt that closes it.

### 5.2 The same class, deliberately not swept in

`schedule` is not the only event GitHub resolves against the default branch rather than against
a ref someone named. Repository-event workflows are in the same position, and none of their jobs
is declared today:

- `triage-issue.yml` (`issues`-only), `verdict-bridge.yml` (`issue_comment` /
  `pull_request_review` / `workflow_run` / `schedule`), `refresh-start-here.yml`
  (`schedule` + `issues` + `pull_request`) — a bot commenting on an unrelated issue can land a
  gating check-run on `main`'s head while a push gate is polling it.
- `publish.yml` and `release-verify.yml` (`release`).

The decision above **does not** cover them, on purpose: `schedule` is the case #5352 asks about
and the one with a criterion that is certainly true (a cron never chooses its ref). For the
others the run's `head_sha` semantics differ per event and must be **verified against the API
before being assumed** — `verdict-bridge.yml`'s `schedule` runs are already covered, its
`issue_comment` runs are not. If the check confirms they behave the same way, widening is a
one-line addition to the excluded-event set in §6.2 plus the matching tests; it is not a
different design. Treat it as a follow-up to the implementation PR, with evidence, rather than
as speculation folded into this decision.

## 6. Implementation spec (for its own PR)

Sequencing note: #3786's branch (`d27a4b8c`, not on `main` at time of writing) rewrites the same
`run_gate` region. **Land #3786 first**, then build on the `gating_runs` split it introduces.

### 6.1 Plumb the trigger event

`make_fetch_workflow_runs()`'s `--jq` projection does not currently select `event`. Add it
(`{id, workflow_id, name, path, head_sha, status, conclusion, event, created_at, run_started_at,
run_attempt, html_url}`), and carry it onto each synthesised check in `_attempt_job_check()` and
`_workflow_summary_check()` as `_workflow_event`, beside the existing `_workflow_id` /
`_workflow_run_id` / `_workflow_run_attempt` keys.

### 6.2 Filter, fail-closed

In `resolve_newest_workflow_runs()`, where a check is already mapped to its producing run via
`_workflow_run_id_of_check()`, drop the check iff that run's `event == "schedule"`. **Only when
the producing run is positively identified**: the existing branch that keeps a check whose run
id resolves to nothing (external / manually-posted check) must keep doing so — unknown
provenance gates, exactly as it does today.

Return the dropped set alongside `superseded` so the caller can report it (§6.4).

**The empty-set belt.** Extend #3786's `empty_gating_hold` so it is armed by *any* discovered
sibling still pending — **including a cron-excluded one** — while no gating leg has registered.
Exclusion must never make the "stable empty set" pass reachable sooner than it is today.

### 6.3 Guard the gate's own event

`ci-summary` never runs on `schedule`. Assert it rather than assume it: if `EVENT_NAME` is ever
`schedule`, skip the filter entirely (a scheduled gate evaluating scheduled siblings is the one
case where they *are* the work under test).

### 6.4 Reporting

- Per poll: `N cron-injected check-run(s) excluded (schedule-triggered run <id>; not about this
  commit)`.
- In `render_verdict`'s summary line, alongside the existing advisory-exclusion count, listing
  the excluded names. The PASSED line already says "gating set stable" post-#3786; keep that
  wording — it remains accurate.

### 6.5 Tests (`scripts/tests/test_ci_summary_gate.py`)

Each must be **red before the change**:

1. A pending `schedule`-run leg on the head does not hold the settle window, and an
   all-terminal gating set converges.
2. A **failing** `schedule`-run leg does not RED the verdict, while a failing `push`-run leg on
   the same fixture still does. (Mutation check: invert the `event` comparison and this pair
   must go red.)
3. A `workflow_run`-produced `feature-matrix report` / `opt-in <name>` check-run is **kept** and
   still awaited — the §3 regression guard, and the most important test in the set.
4. A `workflow_dispatch`-produced leg is **kept** and still gates.
5. A check-run whose producing run cannot be resolved is **kept** (fail-closed).
6. Empty-gating belt: a set of *only* pending cron legs does not reach the empty-set pass.
7. Supersession: a cron run created after a push run of the same workflow no longer supersedes
   the push run's legs.

## 7. Follow-ups (not in this record's scope)

- **F1** — measure the actual duty cycle of pending cron check-runs on `main`'s head over a day,
  to size §2 consequence (1) with evidence rather than configuration arithmetic.
- **F2** — for each lane in §2.1, confirm a failure escalation path that does not depend on
  REDding `main`'s push gate; file a bead for any lane that has none. **Gates F-impl.**
- **F3** — correct the "so the `ci-summary / gate` aggregator never sees it" claim in the headers
  of the five lanes that carry it verbatim — `differential.yml`, `differential-update.yml`,
  `datalog-souffle.yml`, `shacl-diff-fuzz.yml`, `metamorph.yml` — which is true of the per-PR
  gate and false of the `main` push gate. (`pr-backlog.yml`, `bead-autoclose.yml` and
  `release-verify.yml` make a narrower, PR-scoped version of the claim that is true as written.)
- **F-impl** — implement §6.
