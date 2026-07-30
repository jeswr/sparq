#!/usr/bin/env python3
# ci-summary STATUS evaluator — ONE short-lived evaluation, NO resident waiter
# (bead sq-lfmvd, follow-up to sq-90cv4). Design record:
# research/ci-gate-slotless-aggregation.md §2. 🤖 SPARQ agent. [SONNET-4.6]
#
# WHAT THIS IS. `ci-summary / gate` (the single REQUIRED branch-protection check) is
# a WAITER: it polls sibling check-runs for up to ~67 minutes and occupies a hosted
# runner slot for the whole wait. Under pool saturation the waiters starve the very
# builds they wait on. This module is the same VERDICT applied by a different
# TRANSPORT: a `workflow_run`-triggered evaluation that runs for SECONDS each time
# any sibling workflow finishes, and publishes the aggregate as a COMMIT STATUS
# (`ci-summary-status`) on the head SHA. Slot cost becomes O(seconds) per sibling
# completion instead of O(tens of minutes) per PR.
#
# THE VERDICT BRAIN IS NOT REIMPLEMENTED HERE. Every pass/fail decision is
# `ci_summary_gate.render_verdict()` / `failfast_failures()` /
# `draft_selects_unsuperseded()` / `fm_report_status()` / `is_advisory()` — the
# functions the resident gate itself runs, already unit-tested in
# scripts/tests/test_ci_summary_gate.py. What THIS file owns is only the transport:
# which siblings are in scope, when a single observation is allowed to conclude, and
# how the conclusion becomes a commit status. That split is the whole point of the
# sq-90cv4 extraction (design §4) — a second copy of the gating semantics would be a
# second thing to get wrong.
#
# STAGED, NOT SWAPPED (design §3.1 — the risk this file must not get wrong). The
# `ci-summary-status` context is NOT required by branch protection yet, and this
# change does NOT touch the ruleset. It runs ALONGSIDE the resident `gate` job so
# live parity can be measured on real PRs first. A missing "expected" required check
# blocks every merge, so the required-check edit is a separate, maintainer-signed
# step; docs/branch-protection.md §"Event-driven aggregation" carries the runbook and
# the exit criteria (including the two gaps below that must close BEFORE the resident
# job can be dropped).
#
# TRUST MODEL (design §3.2 — resolved, deliberately narrower than the resident gate).
# The evaluator is triggered ONLY by `workflow_run` (plus a maintainer
# `workflow_dispatch`), which ALWAYS executes the DEFAULT-BRANCH copy of both the
# workflow file and this script. It therefore has no `pull_request` trigger at all:
# a `pull_request`-triggered job takes its definition from the PR head, and on
# same-repo agent branches that head can edit the very code holding
# `statuses: write` — i.e. a PR could forge its own green status for a context that
# is destined to become the required check. That is the sparq-org/sparq#3474 threat
# model (see .github/workflows/fast-fix-ring.yml) and it is why the "seed evaluation
# on `pull_request`" sketched in design §3.3 is NOT implemented. The cost is stated
# honestly in the two gaps below rather than paid in trust.
#
# TWO KNOWN GAPS, both blocking the "drop the resident job" step (NOT the shadow
# stage), each tracked as a follow-up issue:
#   (G-quiet) A head that triggers NO sibling workflow at all triggers no evaluation
#     either, so no status is ever published. While the resident gate exists this is
#     unreachable — ci-summary itself runs on every `pull_request`/`merge_group`/push
#     and its completion is a guaranteed `workflow_run` event — but after the job is
#     dropped a genuinely quiet head would sit with the context ABSENT. Absent is the
#     FAIL-CLOSED direction (branch protection blocks), never a false green, so this
#     is a liveness gap, not a soundness one. It needs either a guaranteed
#     always-running sibling or a periodic sweep before the job is dropped.
#   (G-redispatch) The resident gate re-dispatches a newest CANCELLED workflow run
#     once (`actions: write`, #3505). This evaluator holds READ-only `actions` on
#     purpose, so it cannot. It reports that state as PENDING (the resident gate's
#     re-dispatch is what makes progress) until the absolute budget, and only then
#     REDs. Dropping the resident job requires moving that capability here.
#
# HOW ONE OBSERVATION IS ALLOWED TO CONCLUDE (design §3.6 — startup-race parity).
# The resident gate's MIN_POLLS floor and settle window exist so a partial, early
# sibling set cannot be verdicted. A single evaluation has no history, so the
# equivalents are:
#   * REGISTRATION FLOOR — an all-terminal set may not conclude SUCCESS until
#     `registration_floor_seconds` have elapsed since the EARLIEST workflow run on
#     the head SHA (the clock GitHub itself starts when CI begins on that head).
#     Under the floor the verdict is PENDING. A FAILURE is never floored: a
#     concluded gating failure in the authoritative newest run is never forgiven, so
#     delaying it would only delay the red.
#   * THE HOLDS ARE REUSED, NOT RE-DERIVED — a draft-tier-assembled selection with
#     no full-tier successor (`draft_selects_unsuperseded`) and a missing
#     feature-matrix reporter verdict (`fm_report_status == "pending"`) both hold the
#     status at PENDING exactly as they hold the resident gate's settle window open.
#     Each of those late arrivals is itself a workflow completion, so it fires a
#     fresh evaluation — the event stream replaces the poll loop.
#   * ABSOLUTE BUDGET — past `absolute_budget_seconds` (the resident gate's own
#     ~67-minute cap) an outstanding set stops being held and is rendered by
#     `render_verdict`, which fails closed on incomplete legs. The wait can no more
#     be infinite here than it can there.
#
# MONOTONICITY (the reordering guard). Evaluations are concurrent and unordered: a
# slower evaluation that observed an EARLIER state can POST after a faster later one.
# So this module NEVER downgrades an already-published terminal status (success /
# failure) back to `pending`. Any genuinely new pending work on the same SHA arrives
# as a new workflow run, whose completion fires its own evaluation and re-publishes.
# Without this rule the last writer could leave a green head reading `pending`
# forever.
#
# SCOPE OF THE SIBLING SET. Both aggregators are excluded from the set they judge:
# this workflow (so evaluations never wait on each other — `WorkflowRunResolver`
# already drops the whole self-workflow by run id) and `ci-summary.yml` (the
# resident `gate` job, which is pending for as long as it polls; including it would
# make every shadow verdict a strictly-later echo of the thing it is meant to be
# compared against). Exclusion is by workflow FILE PATH, not by job name, so a
# renamed gate job cannot silently re-enter the set.
#
# Hermetic tests: scripts/tests/test_ci_summary_status.py (stdlib-only unittest; no
# network — every fetcher and the status poster are injected). Run as a HARD step in
# docs-quality.yml's `ci-scripts` bucket, the same lane that gates the resident
# gate's own suite.

from __future__ import annotations

import os
import subprocess
import sys
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path

_SCRIPTS_DIR = Path(__file__).resolve().parent
if str(_SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPTS_DIR))

import ci_summary_gate as gate  # noqa: E402  (path shim above must run first)

# The commit-status context this evaluator publishes. DELIBERATELY NOT `gate` and not
# `ci-summary / gate`: during the staged migration both must be selectable in the
# ruleset UI without a maintainer mistaking one for the other, and the name carries no
# transport detail ("event-driven", "shadow") that would be wrong once it is the
# required check.
STATUS_CONTEXT = "ci-summary-status"

# The two AGGREGATOR workflows, excluded from every sibling set (see the header).
RESIDENT_GATE_WORKFLOW_PATH = ".github/workflows/ci-summary.yml"
EVALUATOR_WORKFLOW_PATH = ".github/workflows/ci-summary-status.yml"
AGGREGATOR_WORKFLOW_PATHS = (RESIDENT_GATE_WORKFLOW_PATH, EVALUATOR_WORKFLOW_PATH)

PENDING = "pending"
SUCCESS = "success"
FAILURE = "failure"
TERMINAL_STATES = (SUCCESS, FAILURE)

# GitHub truncates a commit-status description at 140 characters.
DESCRIPTION_LIMIT = 140

# Reused so "non-failing conclusion" has exactly one definition in the repo.
_PASSING = gate._PASSING
_as_int = gate._as_int


@dataclass
class EvalConfig:
    """Transport tunables. Prod values mirror the resident gate's own floors so the
    two transports conclude on the same evidence; tests inject tiny values."""

    # Startup-race floor: seconds after the earliest workflow run on the head SHA
    # before an all-terminal set may conclude SUCCESS (the MIN_POLLS equivalent).
    registration_floor_seconds: int = 180
    # The resident gate's absolute cap: 110 base polls x 20s + 45 extension polls x
    # 40s ~= 67 min. Past this an outstanding set is rendered (fails closed).
    absolute_budget_seconds: int = 4020
    summary_path: str = field(default_factory=lambda: os.environ.get("GITHUB_STEP_SUMMARY", ""))


@dataclass
class Decision:
    """What to publish. `failfast` marks the one state that must be re-observed on a
    fresh fetch before it is trusted (the resident gate's grace re-poll)."""

    state: str
    description: str
    failfast: bool = False


def truncate(description: str, limit: int = DESCRIPTION_LIMIT) -> str:
    """GitHub silently truncates an over-long status description; do it visibly."""
    if len(description) <= limit:
        return description
    return description[: limit - 1].rstrip() + "…"


# ------------------------------- sibling-set scoping -------------------------------


def is_aggregator_workflow(run: dict, paths=AGGREGATOR_WORKFLOW_PATHS) -> bool:
    """True for a workflow run belonging to an aggregator (this evaluator or the
    resident gate). Keyed on the workflow FILE PATH — the one identity a job rename
    cannot change."""
    return (run.get("path") or "") in paths


def aggregator_run_ids(workflow_runs: list[dict], paths=AGGREGATOR_WORKFLOW_PATHS) -> set[int]:
    """Run ids of every aggregator run on the SHA, so both their run-level summaries
    AND their job check-runs can be dropped before the verdict sees them."""
    ids = {
        _as_int(run.get("id"))
        for run in workflow_runs
        if is_aggregator_workflow(run, paths)
    }
    ids.discard(0)
    return ids


def earliest_created_at(workflow_runs: list[dict]) -> str:
    """The elapsed clock's zero: the earliest workflow-run creation stamp on the head
    SHA, aggregators INCLUDED (the resident gate starting IS "CI began on this head").
    Empty when nothing has registered — which keeps elapsed at 0 and so keeps the
    registration floor unsatisfied, the fail-closed direction."""
    stamps = sorted(
        stamp
        for stamp in (
            (run.get("created_at") or run.get("run_started_at") or "")
            for run in workflow_runs
        )
        if stamp
    )
    return stamps[0] if stamps else ""


def parse_timestamp(value: str) -> datetime | None:
    """ISO-8601 with GitHub's trailing `Z`, or None when unparseable."""
    if not value:
        return None
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None
    return parsed if parsed.tzinfo else parsed.replace(tzinfo=timezone.utc)


def elapsed_seconds(origin: str, now: datetime) -> float:
    """Seconds since `origin`. 0.0 when the origin is unknown, unparseable, or in the
    future — every degraded reading keeps the registration floor UNMET rather than
    granting an early green."""
    started = parse_timestamp(origin)
    if started is None:
        return 0.0
    return max(0.0, (now - started).total_seconds())


# ---------------------------------- the decision -----------------------------------


def _non_passing(runs: list[dict]) -> list[dict]:
    """Gating checks that are not satisfied — used ONLY to word a description.
    `render_verdict` remains the authority for the verdict itself; this helper can
    legitimately come back EMPTY on a red (an unsuccessful selection pre-job, a
    missing feature-matrix reporter, a stale draft-tier leg set), and the caller then
    falls back to a generic description plus the step summary render_verdict wrote."""
    return [
        run
        for run in runs
        if not gate.is_advisory(run.get("name", ""))
        and (run.get("status") != "completed" or run.get("conclusion") not in _PASSING)
    ]


def evaluate(
    runs: list[dict],
    *,
    elapsed: float,
    cfg: EvalConfig,
    tier_ctx: gate.TierContext | None = None,
) -> Decision:
    """ONE observation -> one status decision. `runs` is the already-scoped sibling
    set (aggregators, self-workflow and draft-gate artifacts removed).

    The ordering is load-bearing:
      1. an OUTSTANDING set (pending legs, or either reused hold) can still fail fast
         on a concluded gating failure, then holds at PENDING until the absolute
         budget, then renders (fail-closed);
      2. an ALL-TERMINAL set is rendered by `render_verdict` FIRST, so a red is
         published immediately; only a would-be GREEN is subject to the registration
         floor."""
    pending = [run for run in runs if run.get("status") != "completed"]
    awaiting_full = bool(
        tier_ctx
        and tier_ctx.run_tier == "full"
        and tier_ctx.event_name == "pull_request"
        and gate.draft_selects_unsuperseded(runs)
    )
    awaiting_report = gate.fm_report_status(runs) == "pending"

    if pending or awaiting_full or awaiting_report:
        failing = gate.failfast_failures(runs)
        if failing:
            names = gate._name_list(failing, limit=3)
            return Decision(
                FAILURE,
                truncate(
                    f"FAIL (fail-fast) — {len(failing)} gating check(s) concluded "
                    f"failure: {names}"
                ),
                failfast=True,
            )
        if elapsed >= cfg.absolute_budget_seconds:
            gate._emit(
                f"ci-summary-status: absolute budget ({cfg.absolute_budget_seconds}s) "
                f"exhausted with {len(pending)} pending check(s)"
                + (", awaiting the full-tier re-run" if awaiting_full else "")
                + (", awaiting the feature-matrix reporter" if awaiting_report else "")
                + " — rendering the verdict on the observed set (fails closed on "
                "incomplete legs, exactly as the resident gate's timeout does).",
                cfg.summary_path,
            )
            code = gate.render_verdict(runs, cfg.summary_path, tier_ctx)
            state = FAILURE if code else SUCCESS
            return Decision(
                state,
                truncate(
                    f"{'FAIL' if code else 'PASS'} — budget exhausted after "
                    f"{int(elapsed)}s with {len(pending)} check(s) unfinished"
                ),
            )
        held = []
        if awaiting_full:
            held.append("awaiting the full-tier re-run (draft-tier selection)")
        if awaiting_report:
            held.append("awaiting the feature-matrix reporter verdict")
        detail = "; ".join(held) or f"{len(pending)} of {len(runs)} check(s) running"
        return Decision(PENDING, truncate(f"waiting — {detail}"))

    code = gate.render_verdict(runs, cfg.summary_path, tier_ctx)
    if code:
        failing = _non_passing(runs)
        description = (
            f"FAIL — {len(failing)} gating check(s) not passing: "
            f"{gate._name_list(failing, limit=3)}"
            if failing
            else "FAIL — see the evaluation summary (selection / reporter / tier belt)"
        )
        return Decision(FAILURE, truncate(description))

    if elapsed < cfg.registration_floor_seconds:
        gate._emit(
            f"ci-summary-status: {len(runs)} check(s) all terminal and green, but only "
            f"{int(elapsed)}s have elapsed since CI began on this head — under the "
            f"{cfg.registration_floor_seconds}s REGISTRATION FLOOR, a workflow that has "
            "not registered yet could still be missing from this set, so the status "
            "stays PENDING (the resident gate's MIN_POLLS floor, event-driven).",
            cfg.summary_path,
        )
        return Decision(
            PENDING,
            truncate(
                f"waiting — {len(runs)} check(s) green but under the "
                f"{cfg.registration_floor_seconds}s registration floor ({int(elapsed)}s)"
            ),
        )

    gating = [run for run in runs if not gate.is_advisory(run.get("name", ""))]
    return Decision(
        SUCCESS,
        truncate(
            f"PASS — {len(gating)} gating check(s) green, "
            f"{len(runs) - len(gating)} advisory excluded"
        ),
    )


def run_once(
    fetch_runs,
    *,
    cfg: EvalConfig,
    elapsed_of,
    tier_ctx: gate.TierContext | None = None,
) -> Decision | None:
    """Fetch, scope, decide. `elapsed_of()` is read AFTER the fetch (the fetch is what
    discovers the origin stamp). Returns None when the sibling set could not be
    OBSERVED at all — the caller must then publish NOTHING and leave the previous
    status standing, because "I cannot see" is not a verdict.

    A fail-fast FAILURE is re-observed on a fresh fetch before it is trusted, which is
    the resident gate's grace re-poll (header §FAIL-FAST there): it costs one API read
    and removes the mid-registration read race."""
    try:
        runs = _scoped(fetch_runs())
    except gate.SupersededLegsError as exc:
        # (G-redispatch) The evaluator holds READ-only `actions`, so it cannot perform
        # the once-only re-run POST the resident gate does. Report the state and let
        # the resident gate make progress; the retry's completion fires a fresh
        # evaluation. Past the budget this is no longer a wait but a hang.
        elapsed = elapsed_of()
        if elapsed >= cfg.absolute_budget_seconds:
            return Decision(FAILURE, truncate(f"FAIL — {exc}"))
        return Decision(
            PENDING,
            truncate(
                "waiting — newest run cancelled; the resident gate owns the once-only "
                "re-dispatch (actions: write)"
            ),
        )
    except gate.FetchError as exc:
        print(
            f"::error::ci-summary-status: could not observe the sibling set ({exc}) — "
            "publishing NOTHING so the previous status stands."
        )
        return None

    decision = evaluate(runs, elapsed=elapsed_of(), cfg=cfg, tier_ctx=tier_ctx)
    if not decision.failfast:
        return decision
    print(
        "  fail-fast: a concluded gating failure was observed — immediate grace "
        "re-fetch to confirm before publishing the red."
    )
    try:
        confirm = _scoped(fetch_runs())
    except (gate.FetchError, gate.SupersededLegsError) as exc:
        print(f"  grace re-fetch failed ({exc}) — publishing nothing this evaluation.")
        return None
    return evaluate(confirm, elapsed=elapsed_of(), cfg=cfg, tier_ctx=tier_ctx)


def _scoped(runs: list[dict]) -> list[dict]:
    """Drop stale `gate, draft-tier` artifacts, exactly as the resident gate drops them
    from its own sibling set (a superseded aggregator verdict is never a leg).

    The full-tier `gate` name is deliberately NOT matched here — see
    `ci_summary_gate.is_draft_gate_artifact`'s docstring and the run-id self-exclusion
    rationale in ci-summary.yml: a name match would also drop any FUTURE sibling job
    literally named `gate` in another workflow, silently un-gating it. The resident
    gate's real check-runs are removed by the exact, rename-proof WORKFLOW PATH filter
    in `make_fetch_runs` instead. If an aggregator run is missing from the workflow-run
    list index (API lag), its job check-run survives as an "external" check and simply
    holds this evaluation at PENDING — the fail-closed direction, corrected by the next
    event."""
    return [run for run in runs if not gate.is_draft_gate_artifact(run.get("name", ""))]


# --------------------------------- publication ------------------------------------


def publish(
    decision: Decision,
    *,
    current: tuple[str, str] | None,
    post_status,
) -> str:
    """Apply the MONOTONICITY rule and the idempotence rule, then post.

    Returns the action taken — "posted", "skipped-identical" or "skipped-no-downgrade"
    — so the caller can log it and the tests can assert on it without inspecting the
    poster's side effects."""
    current_state, current_description = current or ("", "")
    if decision.state == PENDING and current_state in TERMINAL_STATES:
        return "skipped-no-downgrade"
    if (decision.state, decision.description) == (current_state, current_description):
        return "skipped-identical"
    post_status(decision.state, decision.description)
    return "posted"


# ----------------------------- live (gh-backed) wiring -----------------------------


def _refuse_redispatch(run_id: int) -> None:
    """(G-redispatch) The evaluator holds READ-only `actions`. Raising FetchError here
    makes `WorkflowRunResolver` surface the cancelled-newest-run state as
    SupersededLegsError, which `run_once` reports as PENDING — never as a silent pass,
    and never as an unauthorised write attempt."""
    raise gate.FetchError(
        "the event-driven evaluator holds no `actions: write`, so it does not "
        "re-dispatch a cancelled run; the resident ci-summary gate owns that retry"
    )


def make_fetch_runs(
    repo: str,
    sha: str,
    self_run_id: str,
    *,
    raw_checks=None,
    raw_workflows=None,
    fetch_attempt_jobs=None,
    aggregator_paths=AGGREGATOR_WORKFLOW_PATHS,
):
    """Build (fetch_runs, origin_of) — the resident gate's `WorkflowRunResolver` with
    an AGGREGATOR PATH FILTER spliced into its two fetchers, and no re-dispatch.

    The filter needs the workflow-run list (which carries `path`) in order to drop the
    aggregators' check-runs (which carry only a run id). `WorkflowRunResolver.__call__`
    fetches checks BEFORE workflows, so the check fetcher refreshes the workflow
    snapshot itself and the workflow fetcher serves it; a caller that inverts the order
    still gets a correct — merely one-invocation-stale — filter rather than a crash.
    `origin_of()` reports the earliest run stamp from the LATEST snapshot, aggregators
    included."""
    raw_checks = raw_checks or gate.make_fetch_check_runs(repo, sha)
    raw_workflows = raw_workflows or gate.make_fetch_workflow_runs(repo, sha, self_run_id)
    fetch_attempt_jobs = fetch_attempt_jobs or gate.make_fetch_attempt_jobs(repo)
    snapshot: dict = {"kept": [], "dropped": set(), "origin": ""}

    def _refresh() -> None:
        runs = raw_workflows()
        dropped = aggregator_run_ids(runs, aggregator_paths)
        snapshot["dropped"] = dropped
        snapshot["origin"] = earliest_created_at(runs)
        snapshot["kept"] = [
            run for run in runs if _as_int(run.get("id")) not in dropped
        ]

    def fetch_checks() -> list[dict]:
        _refresh()
        dropped = snapshot["dropped"]
        return [
            check
            for check in raw_checks()
            if gate._workflow_run_id_of_check(check) not in dropped
        ]

    def fetch_workflows() -> list[dict]:
        return snapshot["kept"]

    resolver = gate.WorkflowRunResolver(
        self_run_id=self_run_id,
        fetch_checks=fetch_checks,
        fetch_workflows=fetch_workflows,
        fetch_attempt_jobs=fetch_attempt_jobs,
        redispatch=_refuse_redispatch,
    )
    return resolver, (lambda: snapshot["origin"])


def make_fetch_current_status(repo: str, sha: str, context: str = STATUS_CONTEXT):
    """Read the LATEST commit status for `context` (the combined-status endpoint
    already collapses a context's history to its newest entry). Returns None when the
    context has never been posted OR when the read fails — an unreadable current state
    must not block a fresh publication, and re-posting an identical status is
    harmless."""

    def fetch() -> tuple[str, str] | None:
        try:
            rows = gate._gh_json_lines(
                [
                    f"repos/{repo}/commits/{sha}/status",
                    "--jq",
                    ".statuses[] | {state, description, context}",
                ]
            )
        except gate.FetchError as exc:
            print(f"  (current-status read failed: {exc} — treating as unpublished)")
            return None
        # Selection happens HERE, not inside the jq program: `context` comes from the
        # environment, and interpolating it into a jq string literal would break on any
        # quote it contained.
        mine = [row for row in rows if row.get("context") == context]
        if not mine:
            return None
        return mine[0].get("state") or "", mine[0].get("description") or ""

    return fetch


def make_post_status(repo: str, sha: str, context: str = STATUS_CONTEXT, target_url: str = ""):
    """POST the commit status. Raises FetchError on failure so the caller decides
    whether an unpublishable verdict is worth reddening this (advisory) run."""

    def post(state: str, description: str) -> None:
        args = [
            "gh",
            "api",
            "--method",
            "POST",
            f"repos/{repo}/statuses/{sha}",
            "-f",
            f"state={state}",
            "-f",
            f"context={context}",
            "-f",
            f"description={description}",
        ]
        if target_url:
            args += ["-f", f"target_url={target_url}"]
        try:
            proc = subprocess.run(args, capture_output=True, text=True)
        except (FileNotFoundError, subprocess.TimeoutExpired, OSError) as exc:
            raise gate.FetchError(f"status POST subprocess raised: {exc}") from exc
        if proc.returncode != 0:
            raise gate.FetchError(
                proc.stderr.strip()[:300] or f"status POST gh api exited {proc.returncode}"
            )

    return post


def tier_context_for(trigger_event: str) -> gate.TierContext:
    """The evaluator is ALWAYS a full-tier evaluation (it is not a draft-tier run and
    never emits a draft-tier verdict), so `run_tier` is fixed and `_draft_recheck` is
    a no-op — no PR lookup is needed at all.

    `event_name` still has to be right: `render_verdict`'s draft-tier belt (a full-tier
    verdict must refuse a draft-tier-assembled leg set) is armed only for
    `event_name == "pull_request"`. The triggering RUN's originating event is what
    decides that, NOT this evaluation's own trigger (which is always `workflow_run`) —
    getting that wrong would disarm the belt on exactly the PR heads it exists for, and
    a draft head's reduced leg set could then read green."""
    return gate.TierContext(
        run_tier="full",
        event_name="pull_request" if trigger_event == "pull_request" else trigger_event,
        fetch_pr_draft=None,
    )


def main() -> int:
    repo = os.environ.get("REPO", "")
    sha = os.environ.get("SHA", "")
    self_run_id = os.environ.get("SELF_RUN_ID", "")
    if not repo or not sha or not self_run_id:
        print("::error::ci-summary-status: REPO, SHA and SELF_RUN_ID must all be set.")
        return 1
    context = os.environ.get("STATUS_CONTEXT", STATUS_CONTEXT)
    trigger_event = os.environ.get("TRIGGER_EVENT", "").strip()
    target_url = os.environ.get("TARGET_URL", "").strip()

    # Same fail-closed registry contract as the resident gate (#3773): without the
    # DECLARED-advisory set there is no way to know which checks are deliberately
    # non-gating, so refuse to evaluate rather than guess.
    registry_path = os.environ.get("ADVISORY_REGISTRY", gate.ADVISORY_REGISTRY_PATH)
    try:
        declared = gate.load_advisory_registry(registry_path)
    except gate.AdvisoryRegistryError as exc:
        print(
            f"::error::ci-summary-status: the advisory registry is unreadable ({exc}). "
            "The evaluator cannot decide which checks are DECLARED non-gating, so it "
            "publishes nothing and fails closed. Ensure ci-summary-status.yml's "
            f"sparse-checkout includes {gate.ADVISORY_REGISTRY_PATH}."
        )
        return 1

    cfg = EvalConfig()
    fetch_runs, origin_of = make_fetch_runs(repo, sha, self_run_id)
    now = datetime.now(timezone.utc)
    tier_ctx = tier_context_for(trigger_event)
    print(
        f"ci-summary-status: evaluating {sha[:12]} (triggering event="
        f"{trigger_event or '<unset>'}); {len(declared)} declared-advisory name(s); "
        f"context={context}."
    )

    decision = run_once(
        fetch_runs,
        cfg=cfg,
        elapsed_of=lambda: elapsed_seconds(origin_of(), now),
        tier_ctx=tier_ctx,
    )
    if decision is None:
        return 1

    action = publish(
        decision,
        current=make_fetch_current_status(repo, sha, context)(),
        post_status=make_post_status(repo, sha, context, target_url),
    )
    gate._emit(
        f"ci-summary-status: `{context}` = **{decision.state}** — "
        f"{decision.description} ({action}).",
        cfg.summary_path,
    )
    return 0


if __name__ == "__main__":  # pragma: no cover - exercised via the CI step
    try:
        sys.exit(main())
    except gate.FetchError as exc:  # a failed POST is the only escape left
        print(f"::error::ci-summary-status: could not publish the status ({exc}).")
        sys.exit(1)
