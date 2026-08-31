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

## 4. Finding B — context identity is plausible, but cutover needs live proof

The Checks REST API can create a check-run named `gate` on an arbitrary head SHA, but
write access is restricted to GitHub App installation credentials. `GITHUB_TOKEN` is the
repository-scoped installation token available to an Actions run, so the evaluator can
request `checks: write`; a user token or classic PAT is not an equivalent fallback. The
official API contract is the authority here:
<https://docs.github.com/en/rest/checks/runs>; GitHub documents `GITHUB_TOKEN`'s App
installation identity at
<https://docs.github.com/en/actions/concepts/security/github_token>.

Ruleset 17688455 pins both the context `gate` and GitHub Actions' integration id. The
design must therefore **not assume** that a run created through `GITHUB_TOKEN` has the
right required-check identity merely because its display name matches. Phase 4 must prove
on live heads that every `gate-shadow` run reports the expected GitHub Actions App identity
and that create/update work for same-repository, fork, and merge-group heads. Cutover is
forbidden if that identity differs or is absent.

Two more consequences must be designed for, not assumed away:

- **The current job must eventually be renamed.** A job still named `gate` would emit a
  second, competing check-run of the same name. `scripts/tests/test_ci_select_wiring.py`
  `::TestRequiredCheckAnchor::test_gate_job_name_is_exactly_gate` pins today's name; it
  must be **re-pointed** at the trusted API publisher, never weakened or deleted.
- **An ordinary cutover PR cannot bootstrap itself.** A PR that renames its own resident
  `gate` job loses the required check before its evaluator definition is on the default
  branch. Running both publishers under the same name is also ambiguous and is not an
  acceptable bridge. §9 therefore requires a drained queue and one explicitly authorised
  administrator bypass for the cutover commit; the ruleset context remains exactly
  `gate`. There is no claim of a zero-intervention atomic migration.
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

**Where:** the evaluator-owned check-run on the head SHA. Its `external_id` is a short,
versioned identity (`sparq-gate:v1:<sha>`); its `output.text` carries a versioned machine
marker containing the JSON state. The codec must reject a mismatched SHA/schema, preserve
the human verdict summary separately, and tolerate unknown fields for forward
compatibility.

**The race:** N siblings completing at once means N concurrent evaluations
read-modify-writing the same state. The original §3.5 advised `cancel-in-progress: true`
to debounce. That is wrong in a way that matters:

- Cancelling an in-flight evaluation mid-write can leave torn state.
- Worse, if the cancelled invocation is the one carrying the *final* sibling's completion,
  no further event ever arrives and the gate hangs forever.

**Correct:** every event and sweep dispatch enters the **same** per-head-SHA concurrency
group with `cancel-in-progress: false` and the default single-pending queue. A burst may
replace an older pending invocation, but it cannot cancel the running one. This
coalescing is harmless only because each invocation performs a **full re-read of the
world** (`fetch_runs()` re-fetches every check-run from scratch), never applies the event
as an incremental delta, and because the sweep guarantees a later evaluation. This
behaviour is documented by GitHub's concurrency contract:
<https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency>.

Creation must also be idempotent. Under the concurrency lock the evaluator lists
same-App, same-name check-runs for the SHA, creates one only when none exists, and fails
closed if more than one live candidate exists. It never silently picks a winner from
duplicate required contexts. A newly requested sibling generation or rerun first moves
the existing evaluator check back to `in_progress` and clears its prior terminal verdict
before evaluation; a stale green can never remain authoritative while replacement work is
pending. The Checks update contract permits the App-owned run's status to be updated, and
this reopen path needs its own adversarial test.

## 7. Recommended architecture

The design has three deliberately separate pieces. No workflow that executes a PR or
merge ref receives `checks: write` or dispatch privilege.

### 7.1 Non-verdict seed doorbells

A small PR doorbell runs on `pull_request_target`, so GitHub takes its definition from the
default branch. It never checks out a ref, restores a cache, downloads an artifact, or
executes a repository script. Its token has `actions: write` and no contents/checks/PR
write; its only operation dispatches `ci-gate-eval.yml` at the exact default-branch ref,
passing the PR head SHA as untrusted data through an environment variable. This is the
same trusted-code/no-checkout pattern already used by `merge-group-doorbell.yml`.

For merge groups, extend that existing default-branch doorbell to dispatch the sweep on
enqueue/dequeue; normal `workflow_run: requested` events from the explicit merge-group
workflow inventory provide the direct fast path once the group ref exists. There is no
new `merge_group`-triggered writer or dispatcher whose definition could come from the
combined ref.

Neither doorbell can publish or update a verdict. If one is absent or fails, the required
context remains absent and the scheduled sweep below is the backstop.

### 7.2 Default-branch evaluator

`ci-gate-eval.yml` has **no** `pull_request` or `merge_group` trigger. Its privileged job
runs only on:

- `workflow_run` with `types: [requested, completed]` and an explicit `workflows:` list;
- `workflow_dispatch` with a target SHA, dispatched at the exact default-branch ref by
  the sweeper or a trusted reporter.

GitHub requires the `workflow_run.workflows` selector; there is no wildcard subscription.
The event also runs the evaluator definition from the default branch and can receive a
write token even when the upstream workflow was unprivileged. Those are documented
platform properties, not local assumptions:
<https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_run>.

The list is generated from a committed wake-up inventory and pinned in both directions by
a drift test:

1. every workflow with a `pull_request` or `merge_group` trigger is listed or carries an
   explicit, reviewed non-gating exemption;
2. every listed display name exists exactly once and appears byte-for-byte in
   `ci-gate-eval.yml`;
3. every default-branch reporter that creates check-runs on another head SHA explicitly
   dispatches the evaluator after its last write, or carries a reviewed sweep-only
   exemption;
4. `pr-area-label` remains the unfiltered, always-expected source workflow for both PR and
   merge-group targets, and the two default-branch doorbells' no-checkout, no-check-write,
   dispatch-only posture is pinned.

An unknown workflow therefore fails the routing self-test; it cannot silently disappear
from the wake-up set. The inventory controls **when to wake**, not what the verdict sees:
each invocation still re-reads all workflow runs and check-runs on the target SHA.

The evaluator checks out and executes only its triggering default-branch SHA, with
credentials persistence disabled. A dispatched run refuses to evaluate unless its
workflow ref is the default branch and its execution SHA belongs to that branch's reviewed
history. It never checks out the target SHA, restores a cache from it, downloads its
artifacts, or executes text from the event payload. The target SHA is data: validate its
syntax, repository, current PR/merge-group membership, and target kind before any write.
Event fields reach scripts through environment variables or JSON files, never shell
interpolation.

Permissions are the current read set plus `checks: write` for the evaluator-owned check
and `actions: write` for the existing bounded once-only re-dispatch. The job enters the
per-SHA concurrency group from §6, loads state, performs one `step()`, and updates either
`gate-shadow` or (only after cutover) `gate`.

### 7.3 Default-branch sweep

`ci-gate-sweep.yml` runs at GitHub's minimum supported schedule interval and on manual
dispatch. It holds read permissions plus only the permission needed to dispatch the
evaluator; it does **not** hold `checks: write`. It enumerates current non-draft PR heads,
active merge-group heads, and evaluator-owned non-terminal checks, then dispatches
`ci-gate-eval.yml` at the exact default-branch ref once per SHA. The dispatched runs join
the same concurrency groups as event wake-ups.

`pr-area-label` is the one unconditional expected source workflow per target. The
evaluator records the target discovery time and refuses a stable-empty success until that
run is observed successful and the existing startup/settle semantics are satisfied. If
it never registers or never becomes terminal by the corresponding wall-clock deadline,
the sweep drives `gate-shadow`/`gate` to a terminal failure. A total Actions outage may
delay that diagnostic, but the missing required context remains fail-closed throughout.

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

## 9. Phased plan (each implementation phase is a future bead)

1. **Phase 0 — measure, then go/no-go.** Preserve the live Actions API snapshot linked in
   this PR's review discussion as structured evidence, then add a repeatable collector for
   waiter occupancy and sibling progress. Confirm peak concurrency, rather than only total
   minutes, is binding. A negative result chooses option (c) and stops here.
2. **Phase 1 — pure refactor, zero behaviour change.** Extract `step()`; `run_gate` becomes
   a loop over it (§5). All existing tests pass unchanged; add tests for `step()` purity.
   Independently mergeable, no risk to the required check.
3. **Phase 2 — wall-clock state.** Re-express poll counters as deadlines anchored to a
   persisted start timestamp, keeping the resident driver's fixed cadence equivalent;
   add the fail-closed state codec and duplicate-check detection (§6).
4. **Phase 3 — inventory and seed doorbells.** Add the generated wake-up inventory, its
   both-direction drift test, the default-branch PR doorbell, and the evaluator dispatch
   from the existing merge-group doorbell (§7.1–§7.2). The resident gate is still
   authoritative.
5. **Phase 4 — shadow transport.** Ship the evaluator and sweeper publishing only the
   non-required `gate-shadow` context. Exercise PR, fork, rerun, no-leg, cancellation,
   genuine-hang, late reporter, and merge-group cases.
6. **Phase 5 — evidence gate.** Compare every terminal shadow verdict with the resident
   verdict and measure wake/terminal latency. Verify the check-run's App identity against
   the live ruleset integration. Any unexplained mismatch, duplicate, missing target, or
   permission failure blocks cutover.
7. **Phase 6 — controlled cutover.** Drain and pause merge-queue admission. With explicit
   administrator authorisation, land the reviewed cutover commit via one bypass: rename
   the resident job, switch the already-shipped default-branch evaluator from
   `gate-shadow` to `gate`, and re-point `TestRequiredCheckAnchor`. Immediately evaluate a
   canary PR and merge-group head before reopening admission. The ruleset continues to
   require exactly `gate` throughout.
8. **Phase 7 — cleanup.** Remove the resident polling transport, keep the rollback commit
   prepared, and update `ci-summary.yml`'s doctrine header and
   `docs/branch-protection.md`.

**Rollback.** Before Phase 6, disabling the shadow has no merge effect. After Phase 6,
rollback is another drained-queue, administrator-controlled commit restoring the resident
job name before disabling API publication. It is intentionally not described as an
ordinary self-bootstrapping PR.

## 10. Decisions and live questions

Resolved by this revision:

- `workflow_run` uses a complete explicit workflow list; no wildcard is assumed.
- PR/merge-ref execution is unprivileged and separate from the default-branch evaluator.
- every wake-up is idempotent and keyed by repository + head SHA;
- shadow publication precedes any required-context change;
- initial registration and a missing seed have explicit fail-closed behaviour.

Phase 4 must answer with live evidence, not maintainer guesswork:

1. Does `workflow_run` deliver both requested/completed wake-ups for merge-group-triggered
   source workflows, with the merge-group head SHA?
2. Does an evaluator-created check carry the exact App identity required by ruleset
   17688455 for same-repository, fork, and merge-group heads?
3. Which default-branch reporters need an explicit evaluator doorbell because their own
   `workflow_run.head_sha` is the default-branch SHA rather than the head they annotate?
4. Is peak concurrency or total slot time the binding constraint after the label-router
   and merge batching changes land?
5. Does scheduled hang detection remain timely under the same saturation it is intended
   to diagnose?

## 11. Verification status

Verified by reading this checkout: the `Config` constants and `run_gate` control flow
(`scripts/ci_summary_gate.py`); the gate job name, triggers, permissions and timeout
(`.github/workflows/ci-summary.yml`); the test counts, driver helper and cadence-sensitive
assertions (`scripts/tests/test_ci_summary_gate.py`); the required-check anchor
(`scripts/tests/test_ci_select_wiring.py`); the required context and aggregated lane set
(`docs/branch-protection.md`); and the existing cron lanes' schedules.

The 2026-09-01 revision also checked the official GitHub Actions documentation for the
`workflow_run` selector/default-branch trust model and concurrency coalescing, and the
Checks REST documentation for App-only writes. A live Actions API snapshot linked in this
PR's discussion confirmed that resident `ci-summary` waiters and queued build work coexist
under saturation.

Still not verified: evaluator-created check identity against ruleset 17688455,
merge-group wake-up payloads, required-check registration timing, and shadow/live verdict
equivalence. Phases 4–5 exist specifically to turn those platform assumptions into
evidence before the required check changes.
