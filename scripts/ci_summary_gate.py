#!/usr/bin/env python3
# ci-summary gate poll loop — the single REQUIRED branch-protection check's brain.
# [FABLE-5] Extracted from the inline bash in .github/workflows/ci-summary.yml so
# the loop is UNIT-TESTABLE (bead sq-90cv4 mandates regression tests over the gate's
# verdict semantics), and extended with the ADAPTIVE SATURATION BUDGET described
# below. The workflow header comment in ci-summary.yml remains the doctrine for WHY
# the gate exists and what it aggregates; this file is the doctrine for HOW the loop
# decides. Invoked by ci-summary.yml with env: REPO, SHA, SELF_RUN_ID (+ GH_TOKEN
# for `gh`).
#
# SEMANTICS (faithful port of the bash — sq-prg4 / sq-ipkku / sq-wjth):
#   * Discovers every check-run on the head commit EXCEPT this gate's own run
#     (matched by an ANCHORED "/runs/<SELF_RUN_ID>(/|$)" test on details_url —
#     strictly tighter than the old substring `contains`, which could in principle
#     match a longer run id sharing the prefix).
#   * pending = check-runs with status != "completed". The settle window is re-armed
#     ONLY by pending work (the sq-ipkku / #997 guard): an injection of
#     already-terminal check-runs can never starve convergence.
#   * A verdict renders only when EVERY discovered sibling is terminal, never before
#     the MIN_POLLS startup floor, and only after SETTLE_POLLS consecutive quiet
#     polls. Verdict: advisory/informational-named checks (whole-word, case-
#     insensitive) are EXCLUDED; a gating check passes iff its conclusion is
#     success/skipped/neutral; an empty stable set passes.
#   * Exhausting the loop budget with pending == 0 renders the REAL verdict on the
#     final all-terminal set (the #997 graceful timeout), never a blind RED.
#
# ADAPTIVE SATURATION BUDGET (sq-90cv4). The gate is a WAITER occupying a runner
# slot; under org-pool saturation many concurrent gates starve the very builds they
# wait on, the base budget expires with siblings still QUEUED, and the old
# unconditional `exit 1` emitted a FALSE RED on a PR whose real legs never got to
# run (the 2026-07-02 congestion collapse; memory: reference-ci-congestion-collapse).
# Runner saturation is a THROUGHPUT signal, not a hang. So: at base-budget
# exhaustion with pending work, the gate now distinguishes:
#   * STILL SETTLING — the repo's Actions queue is deep (queued workflow-run count
#     >= SAT_QUEUE_MIN) OR sibling completions are still landing (completed count
#     rose over the last PROGRESS_WINDOW polls). Keep polling, at the slower
#     SAT_INTERVAL to cut API pressure, re-checking the signal each poll, up to the
#     ABSOLUTE cap MAX_TOTAL_POLLS.
#   * GENUINE HANG — pending work with an idle queue AND no recent completions.
#     RED immediately (the old behaviour, now correctly scoped to real hangs).
# The extension NEVER changes what a verdict says: exit 0 still happens ONLY via
# render_verdict over an all-terminal set (or the stable-empty set), so a genuinely
# failing leg still fails and nothing green is synthesised. The absolute cap +
# the workflow-level timeout-minutes bound the wait — no infinite gate.
#
# FETCH-FAILURE TOLERANCE: the bash `set -e` turned ONE transient `gh api` blip
# into a gate RED. A failed poll is now skipped (state untouched) and only
# MAX_CONSEC_FETCH_FAILURES consecutive failures fail the gate. No data => no
# verdict, so this cannot create a false pass.
#
# DRAFT-TIER INTEGRITY (bead: draft-tier CI). [FABLE-5] Draft PR heads run a REDUCED
# leg set (coverage / bench / CodeQL / heavy shards / wasm-equality skipped — see
# docs/branch-protection.md §Draft-tier CI); the ci-select job NAME carries the
# ", draft-tier" marker on draft-assembled runs, and — the STRUCTURAL mechanism —
# the ci-summary gate job's OWN check-run name is tiered the same way: a draft
# payload emits `gate, draft-tier`, never the required `gate` context, so branch
# protection ({context: "gate"}) is unsatisfiable by any draft-tier run and there
# is no supersession window at the un-draft moment (the required context is simply
# ABSENT until the full-tier run concludes). THE INVARIANT: a draft-tier gate
# result must NEVER admit a PR to the merge queue. This script adds four belts on
# top of the structural name tiering:
#   * DRAFT-GATE-ARTIFACT EXCLUSION — a `gate, draft-tier` check-run left on the
#     SHA by an earlier draft-tier gate run is an aggregator verdict over a
#     superseded assembly, not a leg: it is excluded from every sibling set (its
#     FAILURE must not permanently RED the full-tier gate on the same SHA, and its
#     SUCCESS carries nothing the live evaluation does not re-derive).
#   * SUPERSEDED-RUN FORGIVENESS — un-drafting fires ready_for_review on the SAME
#     head SHA; per-PR concurrency then CANCELS the in-flight draft-tier runs,
#     leaving conclusion=cancelled check-runs on the SHA the fresh full-tier gate
#     polls. A cancelled/stale check-run is excused iff a LATER check-run with the
#     same tier-normalized name exists (any state but cancelled/stale — this gate's
#     own fresh `gate` run supersedes its cancelled predecessor). A cancelled run
#     with NO successor still fails the gate; failure/timed_out are NEVER forgiven.
#     This matches branch protection's own semantics (it reads the LATEST run of a
#     required check name).
#   * STALE DRAFT-TIER LEG SET — a FULL-tier pull_request gate run refuses to
#     conclude success while any draft-tier-marked select check-run INSTANCE lacks
#     its OWN, distinct, strictly-later full-tier (unmarked) same-normalized-name
#     successor. Per-INSTANCE matters: ci.yml, bench.yml, feature-matrix.yml and
#     fuzz.yml all expose the IDENTICAL select check-run name, so the first
#     workflow's full-tier select must not release the hold for the other three
#     (whose full-tier runs may not have registered any check-runs yet). The loop
#     treats an unmatched instance as STILL-SETTLING (the ready_for_review re-runs
#     are seconds away); budget exhaustion in that state is a RED ("stale
#     draft-tier run, full run pending"), never a pass over draft legs.
#   * CONCLUSION-TIME DRAFT RE-CHECK — a DRAFT-tier gate run re-reads the PR's
#     CURRENT draft state from the API immediately before emitting a SUCCESS
#     verdict; if the PR is no longer a draft the gate concludes FAILURE ("stale
#     draft-tier run, full run pending") so the queue can never latch onto a
#     draft-tier green. API failure after retries fail-closes to RED (a draft PR
#     cannot merge anyway, so a false RED here is cheap; a false PASS is the
#     invariant violation).
#
# Exit-0 paths, exhaustively: (1) render_verdict over a stable-empty set;
# (2) render_verdict over an all-terminal set with zero non-passing GATING checks
# AND every change-based-selection pre-job check green (sq-fmx4u.3: `skipped` is
# satisfied only under a successful selection; a present-but-not-success select
# REDs outright) AND the draft-tier integrity checks above pass. There is no other
# `return 0`.
#
# Hermetic tests: scripts/tests/test_ci_summary_gate.py (stdlib-only unittest; no
# network — fetchers are injected). Run: python3 scripts/tests/test_ci_summary_gate.py

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import time
from dataclasses import dataclass, field

ADVISORY_RE = re.compile(r"\b(advisory|informational)\b")
_PASSING = ("success", "skipped", "neutral")
# [FABLE-5] draft-tier CI: the marker ci-select.yml appends to the select job name
# on a draft-assembled run ("select (change-based test selection, draft-tier)").
# The tier travels in the check-run NAME — the same name-as-contract mechanism the
# advisory rule uses — so the gate can partition draft-assembled from full-assembled
# selection check-runs on a head SHA without any extra API surface.
DRAFT_TIER_MARKER = ", draft-tier"
# [FABLE-5] Draft-tier CI: THIS aggregator's own job name is tiered the same way
# (ci-summary.yml `gate` job): a draft-payload run emits the check-run
# `gate, draft-tier` and NEVER the required `gate` context — the structural half
# of the integrity invariant (branch protection's required_status_checks entry
# {context: "gate"} cannot be satisfied by a draft-tier run at all). A
# `gate, draft-tier` check-run left on a SHA by an earlier draft-tier gate run is
# therefore a tier ARTIFACT, not a leg: is_draft_gate_artifact() excludes it from
# the sibling set (see run_gate).
GATE_CHECK_NAME = "gate"
DRAFT_TIER_GATE_NAME = GATE_CHECK_NAME + DRAFT_TIER_MARKER
# Conclusions a LATER same-normalized-name check-run may excuse (superseded-run
# forgiveness). Deliberately ONLY the supersession artifacts — a genuine failure /
# timed_out is never forgiven by a later attempt.
_SUPERSEDABLE = ("cancelled", "stale")
# [FABLE-5] sq-fmx4u.3: the change-based test-selection pre-job (the reusable
# .github/workflows/ci-select.yml job, called from ci.yml + feature-matrix.yml).
# Its check-run name embeds this phrase; scripts/tests/test_ci_select_wiring.py
# pins the workflow job name against this regex so the two cannot drift apart.
SELECT_RE = re.compile(r"change-based test selection")


@dataclass
class Config:
    """Loop tunables. Prod values mirror the previous inline bash (INTERVAL /
    MIN_POLLS / SETTLE_POLLS / BASE_POLLS == the old 110-attempt cap) plus the
    sq-90cv4 adaptive-extension knobs. Tests inject tiny values."""

    self_run_id: str = ""
    interval: int = 20          # seconds between polls (base phase)
    min_polls: int = 3          # startup-race floor: no verdict before 3 polls
    settle_polls: int = 2       # all-terminal must hold this many consecutive polls
    base_polls: int = 110       # base budget: 110 x 20s ~= 37 min (the old hard cap)
    sat_interval: int = 40      # slower poll cadence during the saturation extension
    max_total_polls: int = 155  # absolute cap: 45 extension polls x 40s = +30 min
    sat_queue_min: int = 5      # queued workflow-runs in the repo => saturation
    progress_window: int = 15   # polls over which a completed-count rise = progress
    max_consec_fetch_failures: int = 5
    summary_path: str = field(default_factory=lambda: os.environ.get("GITHUB_STEP_SUMMARY", ""))


class FetchError(RuntimeError):
    """A poll's API fetch failed (transient or otherwise)."""


@dataclass
class TierContext:
    """[FABLE-5] Draft-tier CI: which tier THIS gate run is evaluating, and how to
    re-read the PR's live draft state at conclusion time. run_tier is computed from
    the trigger payload (pull_request + draft == true => "draft"; every other
    event/state => "full"). fetch_pr_draft() -> bool (current draft state), raising
    FetchError on API failure; None when the run has no PR (push/merge_group)."""

    run_tier: str = "full"  # "draft" | "full"
    event_name: str = ""
    fetch_pr_draft: object = None
    draft_check_retries: int = 3


def normalized_name(name: str) -> str:
    """A check-run name with the draft-tier marker stripped — the identity under
    which a draft-assembled select and its full-tier successor are the SAME leg."""
    return name.replace(DRAFT_TIER_MARKER, "")


def is_draft_tier(name: str) -> bool:
    """Was this check-run produced by a draft-tier-assembled run (name marker)?"""
    return DRAFT_TIER_MARKER in name


def is_draft_gate_artifact(name: str) -> bool:
    """Is this check-run a draft-tier gate VERDICT (`gate, draft-tier`)? Such a
    run is an aggregator artifact of a superseded draft-tier evaluation, never a
    leg: its FAILURE must not permanently RED the full-tier gate on the same SHA
    (the live run re-derives the verdict over the real legs), and its SUCCESS
    carries no information the live evaluation does not recompute. The full-tier
    `gate` name is deliberately NOT excluded here — a future sibling job
    literally named `gate` in another workflow must keep gating (the run-id
    self-exclusion comment in ci-summary.yml), and cancelled `gate` predecessors
    are handled by forgive_superseded instead."""
    return name == DRAFT_TIER_GATE_NAME


def _order_key(run: dict) -> tuple:
    """Later-run ordering: started_at (ISO-8601 Zulu strings compare correctly as
    text) tie-broken by the check-run id (monotonically allocated)."""
    return (run.get("started_at") or "", run.get("id") or 0)


def forgive_superseded(runs: list[dict]) -> tuple[list[dict], list[dict]]:
    """[FABLE-5] Draft-tier CI: drop cancelled/stale check-runs that a LATER
    check-run with the same tier-normalized name supersedes (any status, any
    conclusion EXCEPT cancelled/stale — an in-flight successor counts: the gate
    then waits on it). Returns (kept, forgiven).

    WHY: un-drafting fires ready_for_review on the SAME head SHA; the per-PR
    concurrency groups cancel the in-flight draft-tier runs, leaving
    conclusion=cancelled check-runs on the SHA that would otherwise permanently
    RED the fresh full-tier gate. Branch protection itself honours only the
    LATEST run of a check name, so excusing a superseded cancellation aligns the
    gate with the enforcement layer. SAFETY: only cancelled/stale are ever
    forgiven (a genuine failure/timed_out always gates, no matter what runs
    later), and a cancellation with NO successor still fails the gate. Call this
    over the RAW list (self-run included) so THIS gate run's own fresh `gate`
    check-run can supersede its cancelled predecessor.

    [OPUS-4.8] sq-fmx4u.3 hardening (2026-07-17 fleet jam): for the change-based
    SELECTION pre-job (is_select) the successor need only be a same-normalized-name
    SUCCESS ANYWHERE on the SHA, not one that is STRICTLY LATER. Rationale: select
    is a deterministic pure function of the SHA's diff, so a single same-name
    success PROVES the selection was computed soundly — the per-instance timestamp
    ordering of its concurrency-cancel race-loser siblings is irrelevant. Under the
    draft-tier + review-pipeline label churn (#2537/#2546) a head SHA accretes many
    cancel rounds whose doomed select instances can out-timestamp the winning run's
    already-concluded select success; the strictly-later rule then left a residual
    cancelled select that RED the gate over a provably-sound selection, escalating
    ~20 review:pass PRs to needs:user and starving the merge train. SAFETY is
    unchanged: this widening is scoped to select ONLY (an idempotent pre-job, never
    a leg that produces test evidence), only cancelled/stale are ever forgiven, and
    a genuine `failure` select or a cancelled select with NO successful same-name
    sibling still fails the gate. Non-select legs keep the strictly-later rule (a
    real leg's earlier stale success must never forgive its later cancellation)."""
    kept: list[dict] = []
    forgiven: list[dict] = []
    for r in runs:
        if r.get("conclusion") in _SUPERSEDABLE:
            key = normalized_name(r.get("name", ""))
            mine = _order_key(r)
            select_leg = is_select(r.get("name", ""))
            if any(
                o is not r
                and normalized_name(o.get("name", "")) == key
                and o.get("conclusion") not in _SUPERSEDABLE
                and (
                    # select: ANY same-name success proves a sound selection;
                    # every other leg: only a STRICTLY-LATER re-run supersedes.
                    (select_leg and o.get("conclusion") == "success")
                    or _order_key(o) > mine
                )
                for o in runs
            ):
                forgiven.append(r)
                continue
        kept.append(r)
    return kept, forgiven


def draft_selects_unsuperseded(runs: list[dict]) -> list[str]:
    """[FABLE-5] Draft-tier CI: names of draft-tier-marked selection check-run
    INSTANCES that have no OWN, distinct, strictly-later full-tier (unmarked)
    successor of the same normalized name — one entry per unsuperseded instance
    (duplicates preserved so the caller can report counts). Non-empty on a
    full-tier pull_request gate run == the leg set on this SHA is (still, at
    least partly) draft-tier-assembled: the gate must wait for the
    ready_for_review full re-runs to register, and must never conclude success
    over it.

    PER-INSTANCE, not per-name: ci.yml, bench.yml, feature-matrix.yml and
    fuzz.yml all call the same reusable ci-select job, so a head SHA carries up
    to FOUR draft-marked selects under the IDENTICAL check-run name. Matching by
    name alone would let the FIRST workflow's full-tier select supersede ALL of
    them, releasing the hold while the other workflows' full-tier runs may not
    have registered any check-runs yet — their skipped/vacuous draft-tier legs
    would then satisfy a full-tier verdict (the exact admission the invariant
    forbids; each full-tier run that registers also registers its pending legs,
    so demanding one successor per instance holds the gate until every selecting
    workflow's re-run is visible). Within each normalized-name group the marked
    and unmarked selects are therefore matched greedily in start order (the
    earliest marked instance consumes the earliest unused strictly-later
    unmarked one — a maximum matching for this single-key order structure).

    Deliberately fail-closed: repeated draft-tier rounds on ONE SHA (e.g. a
    ci-full label toggle while the PR was a draft) accumulate marked instances
    that each demand a successor; a hold that cannot be satisfied REDs at budget
    exhaustion ("stale draft-tier run, full run pending") rather than ever
    passing over draft-assembled legs. Re-running the selecting workflows (or
    pushing a new head) clears it."""
    groups: dict[str, tuple[list[dict], list[dict]]] = {}
    for r in runs:
        name = r.get("name", "")
        if not is_select(name):
            continue
        marked, unmarked = groups.setdefault(normalized_name(name), ([], []))
        (marked if is_draft_tier(name) else unmarked).append(r)
    out: list[str] = []
    for marked, unmarked in groups.values():
        marked.sort(key=_order_key)
        unmarked.sort(key=_order_key)
        fi = 0
        for m in marked:
            while fi < len(unmarked) and _order_key(unmarked[fi]) <= _order_key(m):
                fi += 1
            if fi < len(unmarked):
                fi += 1  # this full-tier successor is consumed by instance m
            else:
                out.append(m.get("name", ""))
    return sorted(out)


def is_advisory(name: str) -> bool:
    """Whole-word advisory/informational match (sq-wjth): excludes the standalone
    words (hyphen-/paren-/comma-delimited too) but NOT substrings — notably
    "cargo-deny (advisories, ...)" is plural and GATES."""
    return bool(ADVISORY_RE.search(name.lower()))


def is_select(name: str) -> bool:
    """[FABLE-5] sq-fmx4u.3: is this check-run the change-based test-selection
    pre-job? Matched by the stable phrase in its job name (case-insensitive).
    If the name ever drifts, detection degrades to select_runs == [] — i.e. the
    PRE-selection semantics (skipped is unconditionally satisfied), never a
    false RED; the wiring inspection test pins the name so it does not drift."""
    return bool(SELECT_RE.search(name.lower()))


def is_self(run: dict, self_run_id: str) -> bool:
    """True iff this check-run belongs to THIS gate's workflow run. Anchored to
    /runs/<id>(/|$) so a longer sibling run id sharing the prefix can't match."""
    if not self_run_id:
        return False
    url = run.get("details_url") or run.get("html_url") or ""
    return bool(re.search(rf"/runs/{re.escape(self_run_id)}(/|$)", url))


def _emit(line: str, summary_path: str = "") -> None:
    print(line, flush=True)
    if summary_path:
        with open(summary_path, "a", encoding="utf-8") as fh:
            fh.write(line + "\n")


def _draft_recheck(tier_ctx: TierContext | None, summary_path: str = "") -> int:
    """[FABLE-5] Draft-tier conclusion-time re-check, applied on EVERY would-be-
    SUCCESS path (including the stable-empty set): a DRAFT-tier run confirms the
    PR is STILL a draft immediately before emitting success. Un-drafted => the
    ready_for_review full-tier run supersedes this one, so conclude FAILURE
    ("stale draft-tier run, full run pending") — the merge queue must never latch
    a draft-tier green. An unreadable draft state (API failure after bounded
    retries, or no fetcher wired) fail-closes to FAILURE: a draft PR cannot merge
    regardless, so a false RED here is cheap and a false PASS is the invariant
    violation. Returns 0 (ok to pass) or 1 (fail). Full-tier runs: always 0."""
    if not tier_ctx or tier_ctx.run_tier != "draft":
        return 0
    still_draft = None
    last_err: Exception | None = None
    if tier_ctx.fetch_pr_draft is not None:
        for _ in range(max(1, tier_ctx.draft_check_retries)):
            try:
                still_draft = tier_ctx.fetch_pr_draft()
                break
            except FetchError as exc:  # transient API blip: bounded retry
                last_err = exc
    if still_draft is None:
        _emit(
            "### ci-summary: FAILED — draft-tier run could not confirm the PR's "
            f"current draft state at conclusion time (last error: {last_err}). "
            "Fail-closed: a draft-tier result must never admit a PR to the merge "
            "queue, so an unverifiable draft state REDs (re-run once the API "
            "recovers; a draft PR cannot merge regardless).",
            summary_path,
        )
        print("::error::ci-summary failed — draft state unverifiable on a draft-tier run.")
        return 1
    if still_draft is False:
        _emit(
            "### ci-summary: FAILED — stale draft-tier run, full run pending. This "
            "gate run evaluated the REDUCED draft-tier leg set, but the PR is no "
            "longer a draft: the ready_for_review full-tier run on this head SHA "
            "supersedes this check (docs/branch-protection.md §Draft-tier CI).",
            summary_path,
        )
        print("::error::ci-summary failed — stale draft-tier run on a now-ready PR.")
        return 1
    return 0


def render_verdict(runs: list[dict], summary_path: str = "", tier_ctx: TierContext | None = None) -> int:
    """Shared by the clean-converge, graceful-timeout, and post-extension paths, so
    every path applies IDENTICAL gating semantics. Returns the process exit code.

    DRAFT-TIER INTEGRITY ([FABLE-5], see the header): with a TierContext,
      * a FULL-tier pull_request verdict REDs while any draft-tier-marked select
        INSTANCE lacks its own later full-tier successor (stale draft-tier leg
        set — at least one selecting workflow's ready_for_review full run has
        not registered on this SHA);
      * a DRAFT-tier verdict that would otherwise be SUCCESS first re-reads the
        PR's CURRENT draft state from the API: no-longer-draft => FAILURE ("stale
        draft-tier run, full run pending"), and an unreadable state fail-closes
        to FAILURE after bounded retries. A draft-tier verdict that is already a
        FAILURE skips the re-check (a RED can never be latched by the queue).
    Without a TierContext (tests / push / merge_group) the semantics are exactly
    the pre-draft-tier ones.

    SELECTION SEMANTICS ([FABLE-5] sq-fmx4u.3, design §5.3): a `skipped`
    conclusion is satisfied ONLY when the change-based selection pre-job
    (is_select) succeeded — a skip is trustworthy iff the thing that decided to
    skip ran to a successful conclusion. Concretely:
      * every select check-run present must have conclusion == "success";
        anything else (failure, cancelled, skipped, neutral, stale) REDs the
        gate outright, even if every other sibling is green — an unobservable
        selection means the skips on this commit are unattributable (§4.3);
      * with select green (or absent — e.g. a pre-selection sibling set, where
        no skip was produced by selection), `skipped` stays non-failing exactly
        as before. Absent-select degradation is deliberately the PRE-sq-fmx4u.3
        behaviour, never a new failure mode.
    A job that FAILED still fails the gate regardless of selection — selection
    can only ever decide whether a SKIP is satisfied, never mask a failure."""
    # [FABLE-5] Draft-tier belt: a full-tier pull_request gate must never conclude
    # over a leg set whose selection was assembled draft-tier (checked FIRST — it
    # invalidates the whole set, including an otherwise-green one).
    if tier_ctx and tier_ctx.run_tier == "full" and tier_ctx.event_name == "pull_request":
        stale = draft_selects_unsuperseded(runs)
        if stale:
            counts: dict[str, int] = {}
            for n in stale:
                counts[n] = counts.get(n, 0) + 1
            detail = ", ".join(
                f"{n} ×{c}" if c > 1 else n for n, c in sorted(counts.items())
            )
            _emit(
                "### ci-summary: FAILED — stale draft-tier run, full run pending. The "
                "selection on this head SHA is (at least partly) draft-tier-assembled: "
                f"{len(stale)} draft-marked select instance(s) have no OWN later "
                f"full-tier successor ({detail}). Each selecting workflow's "
                "ready_for_review full-tier re-run must register its own successor "
                "(ci/bench/feature-matrix/fuzz share one select name — one full-tier "
                "select must never release the hold for the others). A draft-tier leg "
                "set must never admit a non-draft PR to the merge queue "
                "(docs/branch-protection.md §Draft-tier CI).",
                summary_path,
            )
            print("::error::ci-summary failed — stale draft-tier leg set on a non-draft head.")
            return 1
    total = len(runs)
    if total == 0:
        if _draft_recheck(tier_ctx, summary_path) != 0:
            return 1
        _emit("ci-summary: no sibling checks to aggregate (stable empty set) — passing.", summary_path)
        return 0
    gating = [r for r in runs if not is_advisory(r.get("name", ""))]
    excluded = total - len(gating)
    # Selection pre-job health — searched over ALL runs (not just gating) so a
    # hypothetical advisory-renamed select could still never green-light a skip.
    # NB superseded-cancelled select INSTANCES are already dropped upstream by
    # forgive_superseded (including the deterministic-select same-name-success
    # race-loser rule, sq-fmx4u.3 hardening), so any cancelled select that
    # SURVIVES to here has NO successful same-name sibling and rightly REDs.
    select_runs = [r for r in runs if is_select(r.get("name", ""))]
    select_ok = all(r.get("conclusion") == "success" for r in select_runs)
    skipped_ct = sum(1 for r in gating if r.get("conclusion") == "skipped")
    if not select_ok:
        _emit(
            f"### ci-summary: FAILED — the change-based test-selection pre-job did not "
            f"succeed, so the {skipped_ct} skipped gating check(s) on this commit cannot "
            f"be attributed to a sound selection (fail-closed, sq-fmx4u.3 / design §4.3).",
            summary_path,
        )
        for r in select_runs:
            _emit(f"- ✗ {r.get('name')}: {r.get('conclusion') or 'incomplete'}", summary_path)
        print("::error::ci-summary failed — the selection pre-job must conclude success.")
        return 1

    def _satisfied(r: dict) -> bool:
        c = r.get("conclusion")
        if c == "skipped":
            return select_ok  # always True past the gate above; kept explicit so a
            # future refactor that moves this check cannot silently trust a skip.
        return c in _PASSING

    failed = [r for r in gating if not _satisfied(r)]
    if failed:
        _emit(
            f"### ci-summary: FAILED — {len(failed)} non-passing gating check(s) of "
            f"{len(gating)} gating ({excluded} advisory check(s) excluded)",
            summary_path,
        )
        for r in failed:
            _emit(f"- ✗ {r.get('name')}: {r.get('conclusion') or 'incomplete'}", summary_path)
        print("::error::ci-summary failed — see the non-passing gating checks above.")
        return 1
    if _draft_recheck(tier_ctx, summary_path) != 0:
        return 1
    _emit(
        f"### ci-summary: PASSED — all {len(gating)} gating check(s) green (or skipped/neutral); "
        f"{excluded} advisory check(s) excluded; set stable."
        + (
            " DRAFT-TIER verdict (reduced leg set; PR draft state re-confirmed). This "
            f"check-run is `{DRAFT_TIER_GATE_NAME}`, never the required `{GATE_CHECK_NAME}` "
            "context — it cannot satisfy branch protection; the full matrix re-runs at "
            "ready_for_review and only its full-tier gate can."
            if tier_ctx and tier_ctx.run_tier == "draft"
            else ""
        ),
        summary_path,
    )
    if select_runs:
        _emit(
            f"selection: {len(gating) - skipped_ct} of {len(gating)} gating check(s) ran, "
            f"{skipped_ct} skipped (selection and/or path-filter; selection pre-job succeeded).",
            summary_path,
        )
    return 0


def run_gate(cfg: Config, fetch_runs, fetch_queue_depth, sleep_fn=time.sleep,
             tier_ctx: TierContext | None = None) -> int:
    """The poll loop. `fetch_runs()` -> list of {name,status,conclusion,details_url,
    started_at,id} dicts (raises FetchError on API failure); `fetch_queue_depth()`
    -> int queued workflow-run count for the repo, or None when unknown (API
    failure — the gate then falls back to the progress signal alone). tier_ctx
    carries the draft-tier integrity context (None => pre-draft-tier semantics).
    Returns the exit code."""
    prev_names: list[str] | None = None
    stable = 0
    runs: list[dict] = []
    pending = 0
    completed_hist: list[int] = []
    consec_fetch_failures = 0
    extension_started = False

    attempt = 0
    while attempt < cfg.max_total_polls:
        attempt += 1
        try:
            raw = fetch_runs()
        except FetchError as exc:
            consec_fetch_failures += 1
            if consec_fetch_failures >= cfg.max_consec_fetch_failures:
                print(
                    f"::error::ci-summary: {consec_fetch_failures} consecutive check-run "
                    f"fetch failures — cannot observe the sibling set. Last error: {exc}"
                )
                return 1
            print(
                f"attempt {attempt}: check-run fetch failed ({exc}) — skipping this poll "
                f"({consec_fetch_failures}/{cfg.max_consec_fetch_failures} consecutive)."
            )
            sleep_fn(cfg.sat_interval if extension_started else cfg.interval)
            continue
        consec_fetch_failures = 0

        # [FABLE-5] Draft-tier CI: forgive cancelled/stale check-runs a later
        # same-normalized-name run supersedes — over the RAW list (self included)
        # so this run's own fresh `gate` check-run supersedes a cancelled
        # predecessor left on the SHA by the ready_for_review concurrency cancel.
        # Then drop (a) this run's own check-run and (b) any `gate, draft-tier`
        # check-run — a draft-tier gate VERDICT is a tier artifact of a
        # superseded evaluation, never a leg (a completed draft-tier gate
        # FAILURE on the SHA must not permanently RED the full-tier gate, and
        # its SUCCESS never was the required context).
        kept, forgiven = forgive_superseded(raw)
        runs = [
            r for r in kept
            if not is_self(r, cfg.self_run_id)
            and not is_draft_gate_artifact(r.get("name", ""))
        ]
        total = len(runs)
        pending = sum(1 for r in runs if r.get("status") != "completed")
        # [FABLE-5] Draft-tier CI: on a FULL-tier pull_request run, a draft-tier-
        # assembled selection with no full-tier successor means the ready_for_review
        # re-run has not registered yet — treat the set as STILL-SETTLING (hold the
        # settle window open) instead of concluding over draft-tier legs. Budget
        # exhaustion in this state REDs via render_verdict's stale-draft-tier belt.
        awaiting_full = bool(
            tier_ctx
            and tier_ctx.run_tier == "full"
            and tier_ctx.event_name == "pull_request"
            and draft_selects_unsuperseded(runs)
        )
        completed_hist.append(total - pending)
        # Settle is a POST-TERMINAL window re-armed ONLY by pending work (sq-ipkku):
        # already-terminal injections must not starve convergence.
        stable = 0 if (pending or awaiting_full) else stable + 1
        names = sorted({r.get("name", "") for r in runs})
        changed = " (name set changed)" if prev_names is not None and names != prev_names else ""
        prev_names = names
        extra = f", {len(forgiven)} superseded-cancelled forgiven" if forgiven else ""
        extra += ", awaiting the full-tier re-run (draft-tier selection present)" if awaiting_full else ""
        print(
            f"attempt {attempt}: {total} check-run(s), {pending} running, "
            f"all-terminal stable for {stable}/{cfg.settle_polls} poll(s){changed}{extra}",
            flush=True,
        )

        # Clean convergence: everything terminal, held for the settle, past the floor.
        if attempt >= cfg.min_polls and pending == 0 and stable >= cfg.settle_polls:
            return render_verdict(runs, cfg.summary_path, tier_ctx)

        # ADAPTIVE SATURATION BUDGET (sq-90cv4): at/after the base budget with work
        # still pending, extend ONLY while the evidence says throughput-starvation
        # (deep queue) or live progress — otherwise it's a genuine hang: RED.
        if attempt >= cfg.base_polls and pending > 0:
            progressing = (
                len(completed_hist) > cfg.progress_window
                and completed_hist[-1] > completed_hist[-1 - cfg.progress_window]
            )
            # [SONNET-4.6] Guard: if fetch_queue_depth() raises (e.g. subprocess.run
            # raises inside the closure before returning None), treat depth as the
            # "unknown" sentinel — never crash the gate, and never grant a saturation
            # extension on depth alone (None → saturated = False, conservative branch).
            try:
                depth = fetch_queue_depth()
            except Exception as exc:
                print(f"  (queue-depth fetch raised {exc!r} — treating depth as unknown)")
                depth = None
            saturated = depth is not None and depth >= cfg.sat_queue_min
            if not (saturated or progressing):
                print(
                    f"::error::ci-summary timed out — {pending} sibling check-run(s) never "
                    f"finished within the base budget, the Actions queue is idle "
                    f"(depth={depth if depth is not None else 'unknown'} < {cfg.sat_queue_min}) "
                    f"and no completions landed in the last {cfg.progress_window} poll(s): "
                    f"genuine hang, not a still-settling set. See the per-poll log above."
                )
                return 1
            if not extension_started:
                extension_started = True
                print(
                    f"::notice::ci-summary base budget reached with {pending} sibling(s) still "
                    f"pending, but the runner pool shows saturation/progress "
                    f"(queued runs={depth if depth is not None else 'unknown'}, "
                    f"progressing={progressing}) — this is a throughput signal, not a hang. "
                    f"Extending the wait (adaptive budget, sq-90cv4) up to poll "
                    f"{cfg.max_total_polls}."
                )
            else:
                print(
                    f"  extension: queued runs={depth if depth is not None else 'unknown'}, "
                    f"progressing={progressing} — still settling."
                )
            if attempt < cfg.max_total_polls:
                sleep_fn(cfg.sat_interval)
            continue

        if attempt < cfg.max_total_polls:
            sleep_fn(cfg.interval)

    # Absolute budget exhausted.
    if pending == 0:
        # The #997 graceful timeout: everything IS terminal, we just never got a
        # full quiet settle — render the real verdict, never a blind RED.
        print(
            "::notice::ci-summary loop budget reached with every sibling check terminal "
            "(the set kept being injected into without a full quiet settle) — rendering "
            "the verdict on the final all-terminal set."
        )
        return render_verdict(runs, cfg.summary_path, tier_ctx)
    print(
        f"::error::ci-summary timed out — {pending} sibling check-run(s) never finished "
        f"within the ABSOLUTE budget (base + saturation extension, sq-90cv4). The runner "
        f"pool stayed saturated longer than the extension allows; re-run this gate once "
        f"the queue drains. See the per-poll log above."
    )
    return 1


# ----------------------------- live (gh-backed) wiring -----------------------------


def _gh_json_lines(args: list[str]) -> list[dict]:
    # [SONNET-4.6] Wrap subprocess.run so FileNotFoundError / TimeoutExpired / OSError
    # (e.g. `gh` not on PATH) are converted into FetchError, routing them into the
    # existing bounded-retry / skip-this-poll tolerance in run_gate exactly as a
    # non-zero exit code does — no raw crash, no false pass.
    try:
        proc = subprocess.run(["gh", "api", *args], capture_output=True, text=True)
    except (FileNotFoundError, subprocess.TimeoutExpired, OSError) as exc:
        raise FetchError(f"subprocess raised: {exc}") from exc
    if proc.returncode != 0:
        raise FetchError(proc.stderr.strip()[:300] or f"gh api exited {proc.returncode}")
    out = []
    for line in proc.stdout.splitlines():
        line = line.strip()
        if line:
            out.append(json.loads(line))
    return out


def make_fetch_runs(repo: str, sha: str):
    def fetch() -> list[dict]:
        # started_at + id feed the superseded-run ordering (draft-tier CI): a
        # cancelled/stale check-run is forgiven only for a strictly LATER
        # same-normalized-name successor.
        return _gh_json_lines(
            [
                f"repos/{repo}/commits/{sha}/check-runs",
                "--paginate",
                "--jq",
                ".check_runs[] | {name, status, conclusion, details_url, html_url, started_at, id}",
            ]
        )

    return fetch


def make_fetch_pr_draft(repo: str, pr_number: str):
    """[FABLE-5] Draft-tier CI: the conclusion-time PR draft-state reader. Returns
    a () -> bool fetcher (True == still a draft) that raises FetchError on any
    API/parse failure — the caller (render_verdict via _draft_recheck) bounded-
    retries and fail-closes to RED, never to a pass."""

    def fetch() -> bool:
        try:
            proc = subprocess.run(
                ["gh", "api", f"repos/{repo}/pulls/{pr_number}", "--jq", ".draft"],
                capture_output=True,
                text=True,
            )
        except (FileNotFoundError, subprocess.TimeoutExpired, OSError) as exc:
            raise FetchError(f"subprocess raised: {exc}") from exc
        if proc.returncode != 0:
            raise FetchError(proc.stderr.strip()[:300] or f"gh api exited {proc.returncode}")
        val = proc.stdout.strip().lower()
        if val == "true":
            return True
        if val == "false":
            return False
        raise FetchError(f"unexpected .draft value {val!r}")

    return fetch


def make_fetch_queue_depth(repo: str):
    """Queued workflow-run count for the repo — the saturation signal. Returns None
    (unknown) on any failure so a permissions/API blip degrades to progress-only,
    never crashes the gate. Needs `actions: read` on the workflow token."""

    def fetch():
        proc = subprocess.run(
            [
                "gh",
                "api",
                f"repos/{repo}/actions/runs?status=queued&per_page=1",
                "--jq",
                ".total_count",
            ],
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            print(f"  (queue-depth fetch failed: {proc.stderr.strip()[:200]} — treating as unknown)")
            return None
        try:
            return int(proc.stdout.strip())
        except ValueError:
            return None

    return fetch


def main() -> int:
    repo = os.environ.get("REPO", "")
    sha = os.environ.get("SHA", "")
    self_run_id = os.environ.get("SELF_RUN_ID", "")
    if not repo or not sha or not self_run_id:
        print("::error::ci-summary: REPO, SHA and SELF_RUN_ID must all be set.")
        return 1
    # [FABLE-5] Draft-tier CI: the tier THIS run evaluates is decided by its own
    # trigger payload — a pull_request event with draft == true is a DRAFT-tier
    # gate (ci-summary.yml exports the payload's draft flag + PR number). Every
    # other event/state (push / merge_group / non-draft PR / missing payload) is
    # FULL tier, which keeps push/merge_group semantics byte-identical.
    event_name = os.environ.get("EVENT_NAME", "")
    pr_draft = os.environ.get("PR_DRAFT", "").strip().lower()
    pr_number = os.environ.get("PR_NUMBER", "").strip()
    run_tier = "draft" if (event_name == "pull_request" and pr_draft == "true") else "full"
    tier_ctx = TierContext(
        run_tier=run_tier,
        event_name=event_name,
        fetch_pr_draft=make_fetch_pr_draft(repo, pr_number) if pr_number else None,
    )
    print(f"ci-summary: evaluating tier={run_tier} (event={event_name or '<unset>'}).")
    cfg = Config(self_run_id=self_run_id)
    return run_gate(cfg, make_fetch_runs(repo, sha), make_fetch_queue_depth(repo),
                    tier_ctx=tier_ctx)


if __name__ == "__main__":
    sys.exit(main())
