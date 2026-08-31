# Getting the ci-summary gate off the build-runner slot — design record

> Status: **DESIGN-ONLY. No code in this repo implements it yet.**
> Originally written as the sq-90cv4 follow-up (Claude Fable 5 `[FABLE-5]`); **revised
> 2026-08-31 under bead sq-6vshe.19** `[OPUS-5]`, which re-scoped the work with a hard
> constraint that invalidates the original recommendation. §2 records the corrections.
> The operative mitigation today remains the sq-90cv4 *adaptive saturation budget* in
> `scripts/ci_summary_gate.py`.

## 1. Problem

`gate` — the single REQUIRED context in branch-protection ruleset 17688455, produced by
the `gate` job of `.github/workflows/ci-summary.yml` — is a *waiter*: a GitHub-hosted job
that polls sibling check-runs until they are all terminal. While polling it occupies one
job slot in the account-wide hosted-runner concurrency pool.

Verified against the checkout (`scripts/ci_summary_gate.py:424-444`, `Config`):

| Knob | Value | Meaning |
| --- | --- | --- |
| `interval` / `sat_interval` | 20 s / 40 s | poll cadence, base / extension phase |
| `min_polls` | 3 | startup-race floor — no verdict before 3 polls |
| `settle_polls` | 2 | all-terminal must hold this many consecutive polls |
| `base_polls` | 110 | base budget (110 × 20 s ≈ 37 min) |
| `max_total_polls` | 155 | absolute cap (+45 × 40 s ≈ 30 min) |
| `progress_window` | 15 | polls over which a rising completed-count means progress |
| `unsat_confirm_polls` | 3 | polls the unsatisfiable-hold state must persist |
| `max_consec_fetch_failures` | 5 | consecutive fetch failures before RED |

with `timeout-minutes: 80` on the job (`.github/workflows/ci-summary.yml:272`).

Per `research/ci-mergequeue-speedup-2026-07.md` §5 the gate holds a slot ~15–23 min per
merge-group entry, across the concurrent queue entries plus every open PR's own gate plus
the `push: branches: [main]` run — several slots doing no compute during exactly the
bursts when the pool is tightest. (Removing the *push* waiter is sq-6vshe.14, a separate
bead; it is not addressed here.)

The sq-90cv4 adaptive budget fixed the *false verdict* under saturation. It does not fix
the *slot occupancy* — a saturated pool now holds gate slots longer, the correct trade for
verdict honesty, but it leaves throughput on the table.

## 2. What this revision changes

The 2026-07 version of this record recommended re-publishing the verdict as a **commit
status** and **migrating branch protection** to that context, listing the migration as
design risk #1. Three corrections, each verified against the checkout:

1. **The migration is forbidden — and unnecessary.** sq-6vshe.19 constrains the required
   check to stay exactly `gate`. That forecloses the commit-status plan, but nothing is
   lost: an API-created *check-run* named `gate` satisfies the existing ruleset entry with
   no ruleset edit at all (§4). The original risk #1 disappears rather than needing
   staging.
2. **Pure event-driven re-dispatch is unsound on its own.** The original §2 proposed
   `workflow_run`-triggered re-evaluation as a complete replacement. It cannot satisfy
   sq-6vshe.19's own invariant *"a genuine hang still REDs"*, because a hang produces no
   completion events to trigger on (§3). A timer lane is load-bearing, not optional.
3. **The original §3.5 debounce advice was backwards.** It suggested a per-head-SHA
   concurrency group with `cancel-in-progress: true`. Applied to an evaluator, that
   cancels evaluations — including, potentially, the one that would observe the final
   all-terminal set — and hangs the gate (§6).

The original §4 claim still holds and is the reason this is tractable at all: the verdict
brain (`render_verdict`, `forgive_superseded`, `failfast_failures`, `is_advisory`, …) is
already extracted and unit-tested, so only the *transport* changes.

## 3. Finding A — a hang emits no events (the load-bearing flaw)

Every false-RED protection in `run_gate` is a **counter over polls at a guaranteed
cadence**: `min_polls`, `settle_polls`, `progress_window`, `unsat_confirm_polls`,
`max_consec_fetch_failures`, `base_polls`, `max_total_polls`, plus the fail-fast grace
re-poll (an immediate re-fetch that must re-observe the identical failure set,
`scripts/ci_summary_gate.py:1879-1905`). These are meaningful only because the resident
driver guarantees one observation every 20 s.

An evaluator invoked on `workflow_run: completed` fires at the *sibling completion rate*,
which is neither periodic nor guaranteed:

- **Bursty.** A matrix shard set finishing together yields many invocations within
  seconds. `settle_polls = 2` would then be satisfied by two observations milliseconds
  apart — collapsing the post-terminal settle window that sq-ipkku exists to enforce.
- **Zero during a hang.** `progress_window = 15` polls (≈ 5 min of wall-clock) becomes 15
  *completions*, which by definition never arrive when work is stuck, so the saturation /
  hang discrimination never evaluates — and the `max_total_polls` RED never fires at all,
  because with no events there is no invocation to fire it.

Precision on severity: a `gate` check-run that never concludes still **blocks** the merge
(branch protection treats a pending required check as blocking), so this is a **liveness**
defect, not a safety hole. But the consequences are real and are exactly what sq-6vshe.19
names: the PR never learns it is red; `fast-fix-ring.yml` (triggered by `workflow_run` on
ci-summary's *completion*) never rings; the merge-queue entry sits until the queue's own
timeout.

**Therefore:** the counters must be re-expressed as **wall-clock deadlines** anchored to a
persisted start timestamp, *and* a mechanism must guarantee a minimum invocation rate.
Only a scheduled lane provides the latter. This repo already operates sub-hourly cron
sweepers, so the primitive is proven here rather than hypothetical — though note the
finest cadence in use is 5 minutes, on exactly one lane:

| Lane | `cron` | Cadence |
| --- | --- | --- |
| `merge-group-watchdog.yml` | `2,7,12,17,…,57 * * * *` | 5 min |
| `auto-arm.yml` | `4,14,24,34,44,54 * * * *` | 10 min |
| `promote-on-approval.yml` | `*/10 * * * *` | 10 min |
| `batch-merge.yml` | `7,22,37,52 * * * *` | 15 min |
| `ci-latency-alarm.yml` | `26 * * * *` | hourly |

## 4. Finding B — the required context needs no migration

Branch protection matches required checks by **context name**, and a check-run created
through the Checks API occupies the same name namespace as a job-produced one. An
evaluator that creates and updates a check-run named exactly `gate` on the head SHA
satisfies ruleset 17688455's existing `{context: "gate"}` entry with **no ruleset edit**.

This also dissolves the original §3.3 fork problem (a `statuses: write` token is not
available to fork PR heads): the evaluator runs from the **default branch** on
`workflow_run` / `schedule`, so it holds a full-permission token regardless of the PR's
origin — the same isolation property `fast-fix-ring.yml` already depends on
(sparq-org/sparq#3474).

Two consequences that must be designed for, not assumed away:

- **The current job must be renamed.** A job still named `gate` would emit a second,
  competing check-run of the same name. `scripts/tests/test_ci_select_wiring.py`
  `::TestRequiredCheckAnchor::test_gate_job_name_is_exactly_gate` pins today's name; it
  must be **re-pointed** at whatever now emits the `gate` check-run, never weakened or
  deleted.
- **Draft-tier integrity gets structurally weaker.** Today the invariant *"a draft-tier
  result can never satisfy branch protection"* is enforced by a YAML name expression
  (`gate${{ … ', draft-tier' … }}`, `ci-summary.yml:264`) — it holds even if the script is
  wrong. Under an API-created check-run the tiering moves into evaluator *code*, a weaker
  guarantee. This must be re-pinned by a unit test asserting the evaluator never names a
  check-run `gate` on a draft-tier evaluation.

## 5. Finding C — the tests pin the refactor to "extract, don't rewrite"

Counted against `scripts/tests/test_ci_summary_gate.py` (3038 lines): **186 tests**, of
which **115 call sites** drive the full loop through the `run(cfg, polls, …)` helper
(`scripts/tests/test_ci_summary_gate.py:133-146`), which calls `run_gate` with a scripted
list-per-poll `fetch_runs`, a constant `fetch_queue_depth`, and a no-op `sleep_fn`.
**11 assertions** are cadence-sensitive — they assert exact fetch/poll counts
(`fetch.state["calls"]` at lines 1791, 1806, 1819, 1833, 2395, 2401, 2486, 2505, 2516,
2529, 2551) — and one asserts on literal `"attempt 2"` / `"attempt 3"` log text.

sq-6vshe.19 requires those tests be **extended, not weakened**, and the verdict semantics
**bit-identical**. That rules out rewriting the loop. The only shape that satisfies it:

> Extract a pure `step(state, observation, cfg) -> (state, decision)` from the body of
> `run_gate`, and keep `run_gate` as a thin `while` loop over `step`. All 186 existing
> tests then pass **unchanged**, because the resident driver's behaviour is unchanged.
> The event-driven evaluator becomes a **second driver** over the same `step`.

This is a constraint derived from the invariant, not a stylistic preference.

## 6. Finding D — the state store and its write race

State that must survive between invocations is small (well under 1 KB of JSON): the settle
counter, the completed-count history, the fail-fast suspect set, the unsatisfiable-hold
counter, consecutive fetch failures, `extension_started`, and the new start timestamp.

**Where:** the `gate` check-run's own `external_id` / `output` on the head SHA. It is
per-SHA, already written on every invocation, and readable with `checks: read`.

**The race:** N siblings completing at once means N concurrent evaluations
read-modify-writing the same state. The original §3.5 advised `cancel-in-progress: true`
to debounce. That is wrong in a way that matters:

- Cancelling an in-flight evaluation mid-write can leave torn state.
- Worse, if the cancelled invocation is the one carrying the *final* sibling's completion,
  no further event ever arrives and the gate hangs forever.

**Correct:** `cancel-in-progress: false`, which serializes evaluations per head SHA. Note
the residual caveat honestly — GitHub retains only **one pending run** per concurrency
group, so a burst still coalesces and intermediate events are dropped. That is harmless
here *only because* each evaluation is a **full re-read of the world** (`fetch_runs()`
re-fetches every check-run from scratch) rather than incremental event processing, and
because the cron lane guarantees a final evaluation after any dropped burst. Idempotent
full re-evaluation plus a timer backstop is the load-bearing principle of this design.

## 7. Recommended architecture

A new default-branch workflow, `ci-gate-eval.yml`, with a job **not** named `gate`:

- **Triggers:** `workflow_run: types: [completed]` (fires for sibling workflows without
  needing a hard-coded name list); `pull_request` and `merge_group` as *seed* events so a
  `gate` check-run exists promptly; `schedule` at 5-minute granularity as the timer lane
  (§3); `workflow_dispatch` for manual validation.
- **Body:** resolve the target head SHA (`event.workflow_run.head_sha`; on the cron lane,
  enumerate open PR heads and merge-group refs carrying a non-terminal `gate` check-run),
  then run one `step()` — load state from the `gate` check-run, fetch, evaluate, write
  state back. On a terminal decision, conclude the `gate` check-run with the existing
  `render_verdict` summary.
- **Permissions:** today's reads plus **`checks: write`** (to create/update the `gate`
  check-run) and the existing `actions: write` (the #3505 bounded once-only re-dispatch).
  `checks: write` is a genuine privilege increase; it is acceptable only because this
  workflow runs from the default branch and never from PR-head content.
- **Concurrency:** group per head SHA, `cancel-in-progress: false` (§6).

## 8. Honest payoff analysis — peak concurrency, not billable minutes

This is where the bead's estimate needs qualifying.

- **Peak concurrent slot occupancy** drops from N resident waiters to a few ephemeral
  jobs. This is the metric that actually produced the 2026-07-02 congestion collapse, and
  the win here is real.
- **Total slot-seconds may barely improve, or may worsen.** Each evaluation pays fixed
  startup overhead — runner acquisition, the sparse `actions/checkout`, Python start —
  that the resident waiter pays exactly once. `docs/branch-protection.md` enumerates on
  the order of dozens of aggregated lanes per head, so the sum of those overheads is
  plausibly comparable to the resident wait it replaces.

So sq-6vshe.19's "frees 3–6 runner slots" is a **peak-concurrency** claim and must not be
restated as a billable-minutes saving. **This is also the decision point for the bead's
option (c) (reject with measurement):** if measurement shows the pool is bound by total
minutes rather than peak concurrency, this re-architecture is not worth its risk and the
adaptive saturation budget stays the operative mitigation.

## 9. Phased plan (each phase a future bead)

1. **Phase 0 — measure, then go/no-go.** Instrument the current gate's per-run slot-seconds
   and count sibling-completion events per head. Confirm peak concurrency (not total
   minutes) is the binding constraint. A negative result closes sq-6vshe.19 as option (c).
2. **Phase 1 — pure refactor, zero behaviour change.** Extract `step()`; `run_gate` becomes
   a loop over it (§5). All 186 existing tests pass unchanged; add tests for `step()`
   purity. Independently mergeable, no risk to the gate.
3. **Phase 2 — wall-clock re-expression.** Re-express the poll counters as deadlines
   anchored to a persisted start timestamp (§3), keeping the resident driver's fixed
   cadence so poll-count and wall-clock stay equivalent and every existing test still
   passes.
4. **Phase 3 — state codec.** Serialize/deserialize state via the `gate` check-run; unit-test
   round-trip plus tolerance of an unknown-field payload (forward compatibility).
5. **Phase 4 — SHADOW.** Ship `ci-gate-eval.yml` publishing a **non-required** check-run
   named `gate-shadow` alongside the live resident gate. Soak, and diff shadow vs live
   verdict per head. Zero merge risk. This is sq-6vshe.19's "shadow-run first", and it is
   the only way to settle the platform questions in §10.
6. **Phase 5 — cutover.** Only on a clean shadow diff: rename the resident job, flip the
   evaluator to publish `gate`, re-point `TestRequiredCheckAnchor` (§4).
7. **Phase 6 — remove the resident waiter**, and update `ci-summary.yml`'s doctrine header
   and `docs/branch-protection.md`.

**Rollback.** Because the ruleset is never edited (§4), reverting the evaluator's `gate`
publication and restoring the job name restores today's behaviour in a single commit, and
there is no window in which the required context is unsatisfiable. That property is the
main practical reason Finding B matters.

## 10. Open questions for the maintainer

1. **Does `workflow_run: completed` fire for sibling runs triggered by `merge_group`?**
   This is the single riskiest platform assumption in the design. It was **not** verified —
   see §11. Phase 4's shadow run is what proves or kills it.
2. **Does the merge queue tolerate a required check created after the merge-group ref is
   built?** The seed evaluation must register the `gate` check-run promptly, or the queue
   may treat the ref as carrying no required check at all.
3. **Appetite for `checks: write`** on a default-branch workflow (§7).
4. **Is peak concurrency or total minutes the binding constraint?** Phase 0 answers it;
   it determines go/no-go (§8).
5. **Hang-RED latency.** Five minutes is the finest cron granularity GitHub offers, and
   ticks are themselves delayed under load. Detection of a genuine hang moves from ~20 s
   to roughly 5–10 min. Acceptable?

## 11. Verification status

Verified by reading this checkout: the `Config` constants and `run_gate` control flow
(`scripts/ci_summary_gate.py`); the gate job name, triggers, permissions and timeout
(`.github/workflows/ci-summary.yml`); the test counts, driver helper and cadence-sensitive
assertions (`scripts/tests/test_ci_summary_gate.py`); the required-check anchor
(`scripts/tests/test_ci_select_wiring.py`); the required context and aggregated lane set
(`docs/branch-protection.md`); and the existing cron lanes' schedules.

**Not verified — no GitHub API call was made from this session, by orchestration policy.**
Every claim about GitHub *platform* behaviour — `workflow_run` firing for merge-group
siblings, check-run/branch-protection context matching, merge-queue check registration
timing, and concurrency-group pending-run coalescing — is drawn from platform
documentation and from how existing workflows in this repo already rely on it. None of it
is measured. Phases 0 and 4 exist precisely to convert these assumptions into evidence
before anything touches the required check.
