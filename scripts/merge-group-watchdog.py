#!/usr/bin/env python3
"""Recover a merge-queue entry whose merge-group ref was never dispatched by Actions."""

# [OPUS-5] sparq-org/sparq#4652 — the zero-dispatch merge-group watchdog.
#
# THE OUTAGE THIS FIXES (measured 2026-07-27/28, #4652): PR #4534's merge-group ref
# `gh-readonly-queue/main/pr-4534-3cc1bf828c…` was created at 23:04:37Z with head
# `1bfb0174…` and deleted at 00:05:04Z — 60 min 27 s later — having produced
# **0 check-suites, 0 check-runs, 0 workflow runs**. Eight workflows carry a bare
# unfiltered `merge_group:` trigger and fired for every other group that day; for this
# one ref none of them was ever CONSIDERED. #4534 sat at position 1 in AWAITING_CHECKS;
# the queue merges strictly in order and each entry needs its OWN group's required
# check, so #4400/#4636/#4639 sat behind it with green groups that could not land.
# GitHub eventually dequeued #4534 with reason CI_TIMEOUT at 00:05:03Z (the ruleset's
# `check_response_timeout_minutes: 60`), and the chain rebuilt 3 s later, discarding
# three already-green group CI runs. Net: 73 min 41 s of zero merges (23:18:59Z →
# 00:32:40Z) and ~66 min of wasted group CI.
#
# WHY ZERO check-SUITES IS THE LOAD-BEARING SIGNAL — and why "zero check-runs" is NOT.
# A `paths`-filtered workflow that matches nothing STILL creates a check-suite (it just
# reports no runs). So a group with real workflows but nothing to do reads as
# suites>0 / runs==0. Zero *suites* means no workflow was ever considered for the ref,
# i.e. the `merge_group` event never reached the Actions dispatcher. Weakening the
# predicate to check-runs would fire on legitimately-pending and legitimately-empty
# groups. The predicate here is `total_count == 0` on
# `GET /repos/{repo}/commits/{group_head_sha}/check-suites`, and NOTHING else.
#
# ── the anchor problem, and the measurement that settles it ───────────────────────
# The grace gate needs T_create = when the merge-group ref was BUILT. Two obvious
# proxies are WRONG and were rejected by measurement:
#   * `entry.enqueuedAt` — an entry can sit outside the `max_entries_to_build` window
#     for many minutes before its group is built, so this anchor is far too early.
#   * the group head commit's `committer.date` — GitHub STAMPS the group commit with
#     the entry's `enqueuedAt`, not the build time. Measured: head `0e043a13`
#     (PR #4636's group) is stamped 23:01:57Z == #4636's enqueuedAt, but that group's
#     branch was created at 23:19-ish; head `1bfb0174` is stamped 22:58:53Z while its
#     own PARENT `3cc1bf828` (#4632) did not reach `main` until 23:18:59Z — a commit
#     cannot predate its parent, which proves the field is a stamp, not a creation
#     time. Any latency computed from it is meaningless.
# The CORRECT, exact anchor is the repository ACTIVITY API:
#     GET /repos/{repo}/activity?activity_type=branch_creation&ref=<full ref>
# whose rows carry an exact `timestamp` plus `after` = the created head SHA. Measured
# for the incident ref: `branch_creation` at 23:04:37Z, `after=1bfb0174…`, deleted
# 00:05:04Z — a 3627 s lifetime with zero suites. It is also the ONLY enumeration
# source that can see such a ref at all: it has no workflow runs and the branch is
# deleted, so the runs API cannot find it.
#
# We match on `after == head_oid`, never on the ref NAME. Measured over the full
# retained queue population (1281 branch_creation rows, 2026-07-02 -> 07-28): 15 ref
# NAMES were created 2+ times (max 4x) because the queue rebuilds a group on the same
# base, while head SHAs had ZERO collisions. Head SHA is the only sound key, and it is
# the same key the recovery idempotence uses.
#
# ── the decision table (fail-safe: only RECOVER is a mutation) ────────────────────
#   SKIP     entry state is not AWAITING_CHECKS, or no speculative group is built yet
#   HOLD     >=1 check-suite present — dispatch happened; NOT our case (the control)
#   WAIT     zero suites but still inside the grace period
#   REFUSE   the suite count or the creation time could NOT be positively established
#   NOOP     a recovery was already attempted for this exact group head (idempotence)
#   CAP      the per-PR or per-run recovery budget is exhausted
#   RECOVER  zero suites, past grace, positively established, within budget => act
# Anything that cannot be POSITIVELY established is a REFUSE, never a recovery: a
# watchdog that re-enqueues on ambiguity thrashes the queue and is worse than the bug.
#
# ── emission ──────────────────────────────────────────────────────────────────────
# EVERY entry emits exactly one row EVERY tick — naming the ref, the head, the suite
# count and the decision — plus a closing census row, plus an explicit row when the
# queue is empty. A held CAP or REFUSE re-emits every tick for as long as it holds;
# a silent watchdog would convert a visible 73-minute outage into an invisible one.
#
# ── the dequeue-routing split (`classify-dequeue`) ────────────────────────────────
# `.github/workflows/merge-queue-feedback.yml` routes every non-merge dequeue to
# `review:changes` by design. For a zero-dispatch CI_TIMEOUT that is a
# MACHINE-MANUFACTURED re-review: #4534 lost `review:pass` for a platform event drop,
# with nothing whatsoever wrong in its diff. `classify-dequeue` splits the two cases by
# the SUITE COUNT (never by the timeout reason alone) and prints a `route=` output the
# workflow branches on. It also recognises the watchdog's OWN dequeue — our recovery is
# implemented as dequeue+enqueue, which itself fires `pull_request.dequeued`, and
# without this arm the watchdog would demote every PR it rescued.
#
# The group head SHA cannot be recovered from the API after the ref is deleted (that is
# exactly the population with no workflow runs to read it off), so the watchdog records
# it in a machine-readable marker comment BEFORE it acts. The marker is a POINTER, not
# a cached verdict: `classify-dequeue` re-derives the live suite count for the recorded
# SHA at decision time (the commit stays readable after the branch is deleted — verified
# on `1bfb0174`, still `total_count: 0`). A missing/stale marker, an unreadable count,
# or a non-zero count all fall back to today's demote behaviour.
#
# CLI:
#   python3 scripts/merge-group-watchdog.py sweep --repo O/R [--branch main] [--dry-run]
#   python3 scripts/merge-group-watchdog.py classify-dequeue --repo O/R --pr N --reason R
#   python3 scripts/merge-group-watchdog.py --self-test

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from typing import Callable, Iterable

import gh_retry

PROGRAM = "merge-group-watchdog"

# ── tunables ─────────────────────────────────────────────────────────────────────
# Grace before a zero-suite group is called dropped.
#
# MEASURED, N=209 merge-group refs over 2026-07-25T00:00Z -> 2026-07-28T01:30Z, using
# the activity `branch_creation` timestamp -> earliest check-suite `created_at`:
#     min 1 s | p50 2 s | p90 3 s | p99 4 s | MAX 4 s   (whole-second quantisation)
# There is NO tail between 4 s and 3600 s. The failure mode is CATEGORICAL, not slow:
# a group either gets its suites within 1-4 s or gets none at all, ever.
#
# THREE confirmed zero-dispatch refs, one per day, all reaped by the 60-minute timeout:
#     pr-4331-f6be7767…  2026-07-26  head f44083c7  lifetime 61m07s  suites 0
#     pr-4534-3cc1bf82…  2026-07-27  head 1bfb0174  lifetime 60m27s  suites 0
#     pr-4709-63dfb7a0…  2026-07-28  head 98204656  held pos 1 48m   suites 0
# #4709 was caught LIVE at 05:55Z by this script's own `sweep --dry-run` and cleared by
# hand. Its siblings in the same chain that day discriminate the signal perfectly:
#     pr-4709 -> 0 suites / 0 runs        <-- dead
#     pr-4714 -> 8 suites / 95 runs       (ref built 05:33:08Z, first suite 05:33:10Z)
#     pr-4724 -> 8 suites / 54 runs
#     pr-4729 -> 8 suites / 69 runs
#     pr-4731 -> 8 suites / 93 runs
# Zero against eight, every time. Cost of that one dead ref: #4714 and #4729 were both
# already MERGEABLE behind it and could not land, four PRs stacked, and sparq merged
# ONE PR in sixty minutes.
#
# 120 s is 30x the measured MAXIMUM and 60x the median, and sits strictly below the
# 300 s cron interval, so it never costs an extra tick: any ref that is going to get
# suites has them long before the first tick that could look at it. Detection is
# tick-quantised, so effective recovery is 2-10 min against the 60-minute timeout.
DEFAULT_GRACE_SECONDS = 120
# At most this many watchdog recoveries for one PR inside the rolling window below.
# A re-enqueue mints a NEW queue attempt, so an attempt-scoped cap would be no cap at
# all; the budget is per PR per wall-clock window. Exhaustion is not an escalation —
# the entry is handed back to the platform's own CI_TIMEOUT, whose routing now
# preserves the review verdict (see classify-dequeue), and a CAP row is emitted every
# tick for as long as it holds.
DEFAULT_MAX_RECOVERIES_PER_PR = 2
DEFAULT_RECOVERY_WINDOW_SECONDS = 6 * 3600
# Blast-radius bound: one tick may never churn the whole queue.
DEFAULT_MAX_RECOVERIES_PER_RUN = 2
# How long after a watchdog marker a MANUAL/UNKNOWN dequeue is attributed to the
# watchdog's own recovery rather than to a human or the platform.
DEFAULT_SELF_DEQUEUE_WINDOW_SECONDS = 900
# A marker older than this is never allowed to steer a routing decision.
DEFAULT_MARKER_STALENESS_SECONDS = 6 * 3600
# Read a bounded slice of the queue; sparq's queue is single-digit deep.
QUEUE_ENTRY_PAGE = 20

# Only an entry that HAS a built group and is waiting on it can be zero-dispatch.
# QUEUED/UNMERGEABLE have no group to be dropped; MERGEABLE already reported;
# LOCKED is the queue's own transient.
ACTIONABLE_STATES = frozenset({"AWAITING_CHECKS"})

# Dequeue reasons a `dequeuePullRequest` mutation (ours) can surface as. A
# MERGE_CONFLICT / CI_FAILURE dequeue is never attributed to the watchdog.
SELF_DEQUEUE_REASONS = frozenset({"MANUAL", "UNKNOWN"})
ZERO_DISPATCH_REASON = "CI_TIMEOUT"

# Verdicts. Named constants so a test cannot silently drift from the emitter.
SKIP = "SKIP"
HOLD = "HOLD"
WAIT = "WAIT"
REFUSE = "REFUSE"
NOOP = "NOOP"
CAP = "CAP"
RECOVER = "RECOVER"

ROUTE_PRESERVE = "preserve"
ROUTE_DEMOTE = "demote"

MARKER_KEY = "merge-group-watchdog"
_MARKER_RE = re.compile(
    r"<!--\s*" + MARKER_KEY + r"\s+(?P<kv>[^>]*?)\s*-->",
)
_MARKER_FIELD_RE = re.compile(r"(?P<key>[a-z_]+)=(?P<value>\S+)")

QUEUE_QUERY = """query($owner:String!,$name:String!,$branch:String!,$first:Int!){
  repository(owner:$owner,name:$name){
    mergeQueue(branch:$branch){
      entries(first:$first){
        nodes{
          id
          position
          state
          enqueuedAt
          baseCommit{oid}
          headCommit{oid}
          pullRequest{number id}
        }
      }
    }
  }
}"""

# THE REMEDIATION CALL, and the three ways to get it wrong (each cost real time):
#   * `gh pr merge <N> --disable-auto` does NOT dequeue an already-queued PR — it
#     answers "is already queued to merge" and leaves the entry in place.
#   * `DequeuePullRequestInput` names its field `id`, NOT `pullRequestId`, and that
#     field wants the PULL REQUEST node id (`PR_…`) — not the merge-queue ENTRY id
#     (`MQE_…`), which is the natural wrong answer given the field name. (Enqueue, in
#     the other direction, really is `pullRequestId`.)
#   * `MergeQueueEntry` has no `headOid` field; the group head is `headCommit{oid}`.
DEQUEUE_MUTATION = """mutation($id:ID!){
  dequeuePullRequest(input:{id:$id}){ mergeQueueEntry{ id } }
}"""

ENQUEUE_MUTATION = """mutation($id:ID!){
  enqueuePullRequest(input:{pullRequestId:$id}){ mergeQueueEntry{ id position } }
}"""


class GhError(RuntimeError):
    """A GitHub CLI command failed."""


# ── pure helpers (no I/O — the self-test drives these directly) ──────────────────


def parse_iso(value: str | None) -> datetime | None:
    """Parse a GitHub ISO-8601 UTC timestamp. Returns None on anything unparseable."""
    if not value or not isinstance(value, str):
        return None
    text = value.strip()
    if text.endswith("Z"):
        text = text[:-1] + "+00:00"
    try:
        parsed = datetime.fromisoformat(text)
    except ValueError:
        return None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def iso(moment: datetime) -> str:
    return moment.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def queue_ref(branch: str, pr_number: int, base_oid: str) -> str:
    """The merge-group ref GitHub builds for an entry.

    VERIFIED against live data: `gh-readonly-queue/<base_branch>/pr-<N>-<BASE oid>`,
    where <N> is THIS entry's PR and the trailing sha is the group's BASE (the previous
    entry's head), NOT its head. Misreading that suffix as the head hid the #4652 cause
    for an hour, so the head is always carried separately.
    """
    return f"gh-readonly-queue/{branch}/pr-{pr_number}-{base_oid}"


def render_marker(
    *, pr: int, head: str, base: str, ref: str, suites: int, observed: datetime
) -> str:
    """The machine-readable marker line embedded in the watchdog's PR comment."""
    return (
        f"<!-- {MARKER_KEY} pr={pr} head={head} base={base} ref={ref} "
        f"suites={suites} observed={iso(observed)} action=re-enqueue -->"
    )


@dataclass(frozen=True)
class Marker:
    pr: int
    head: str
    base: str
    ref: str
    suites: int
    observed: datetime
    action: str


def parse_marker(body: str | None) -> Marker | None:
    """Parse a watchdog marker out of a comment body; None when absent/malformed."""
    if not body:
        return None
    match = _MARKER_RE.search(body)
    if not match:
        return None
    fields = {
        m.group("key"): m.group("value")
        for m in _MARKER_FIELD_RE.finditer(match.group("kv"))
    }
    observed = parse_iso(fields.get("observed"))
    if observed is None:
        return None
    try:
        pr = int(fields["pr"])
        suites = int(fields["suites"])
    except (KeyError, ValueError):
        return None
    head = fields.get("head", "")
    if not re.fullmatch(r"[0-9a-f]{40}", head):
        return None
    return Marker(
        pr=pr,
        head=head,
        base=fields.get("base", ""),
        ref=fields.get("ref", ""),
        suites=suites,
        observed=observed,
        action=fields.get("action", ""),
    )


@dataclass(frozen=True)
class QueueEntry:
    pr_number: int
    pr_id: str
    entry_id: str
    position: int
    state: str
    enqueued_at: datetime | None
    base_oid: str | None
    head_oid: str | None


@dataclass(frozen=True)
class Decision:
    verdict: str
    detail: str


@dataclass(frozen=True)
class Route:
    route: str
    reenqueue: bool
    detail: str


def decide_entry(
    entry: QueueEntry,
    *,
    suites: int | None,
    created_at: datetime | None,
    now: datetime,
    grace_seconds: int,
    markers: Iterable[Marker],
    recoveries_in_window: int,
    max_recoveries_per_pr: int,
    run_recoveries: int,
    max_recoveries_per_run: int,
) -> Decision:
    """The whole watchdog policy, as a pure function of observed state.

    `suites is None` and `created_at is None` both mean "could NOT be positively
    established" and must never reach a mutation — see the REFUSE arms below.
    """
    if entry.state not in ACTIONABLE_STATES:
        return Decision(SKIP, f"entry state {entry.state} is not AWAITING_CHECKS")
    if not entry.head_oid or not entry.base_oid:
        return Decision(SKIP, "no speculative group built for this entry yet")
    if suites is None:
        return Decision(
            REFUSE,
            "check-suite count could not be positively established — taking no action",
        )
    if suites > 0:
        return Decision(
            HOLD,
            f"{suites} check-suite(s) present — the merge_group event was dispatched",
        )
    if created_at is None:
        return Decision(
            REFUSE,
            "no branch_creation activity row matches this group head — "
            "the group's build time could not be established, taking no action",
        )
    age = (now - created_at).total_seconds()
    if age < 0:
        return Decision(
            REFUSE,
            f"group ref creation {iso(created_at)} is in the future — clock skew, "
            "taking no action",
        )
    if age < grace_seconds:
        return Decision(
            WAIT,
            f"zero check-suites {int(age)}s after the ref was built; "
            f"inside the {grace_seconds}s grace",
        )
    if any(marker.head == entry.head_oid for marker in markers):
        return Decision(
            NOOP,
            "a recovery was already attempted for this exact group head — "
            "repeat detection on the same ref is a no-op",
        )
    if recoveries_in_window >= max_recoveries_per_pr:
        return Decision(
            CAP,
            f"per-PR recovery budget exhausted ({recoveries_in_window}/"
            f"{max_recoveries_per_pr}) — handing back to the platform CI_TIMEOUT, "
            "whose routing preserves the review verdict",
        )
    if run_recoveries >= max_recoveries_per_run:
        return Decision(
            CAP,
            f"per-run recovery budget exhausted ({run_recoveries}/"
            f"{max_recoveries_per_run}) — the next tick re-evaluates",
        )
    return Decision(
        RECOVER,
        f"zero check-suites {int(age)}s after the ref was built "
        f"(grace {grace_seconds}s) — dequeue + re-enqueue",
    )


def classify_dequeue_route(
    *,
    reason: str,
    markers: Iterable[Marker],
    last_enqueued_at: datetime | None,
    live_suites: Callable[[str], int | None],
    now: datetime,
    self_dequeue_window_seconds: int = DEFAULT_SELF_DEQUEUE_WINDOW_SECONDS,
    marker_staleness_seconds: int = DEFAULT_MARKER_STALENESS_SECONDS,
) -> Route:
    """Split a dequeue into the preserve-the-verdict route and today's demote route.

    FAIL-SAFE: every arm that cannot POSITIVELY establish zero-dispatch returns
    `demote`, which is the pre-existing behaviour.
    """
    ordered = sorted(markers, key=lambda m: m.observed, reverse=True)
    newest = ordered[0] if ordered else None
    normalised = (reason or "UNKNOWN").strip().upper()

    if newest is None:
        return Route(
            ROUTE_DEMOTE,
            False,
            "no merge-group-watchdog observation recorded for this PR",
        )

    marker_age = (now - newest.observed).total_seconds()

    # (a) our OWN recovery. The watchdog's dequeue+enqueue fires pull_request.dequeued;
    # without this arm the watchdog would demote every PR it just rescued.
    if normalised in SELF_DEQUEUE_REASONS and 0 <= marker_age <= self_dequeue_window_seconds:
        return Route(
            ROUTE_PRESERVE,
            False,
            f"dequeue reason {normalised} within {self_dequeue_window_seconds}s of the "
            f"watchdog's own recovery of head {newest.head[:8]} "
            f"({int(marker_age)}s ago) — the watchdog re-enqueues; verdict preserved",
        )

    # (b) the platform timeout. Split by SUITE COUNT, never by the reason alone.
    if normalised != ZERO_DISPATCH_REASON:
        return Route(
            ROUTE_DEMOTE,
            False,
            f"dequeue reason {normalised} is not {ZERO_DISPATCH_REASON}",
        )
    if last_enqueued_at is None:
        return Route(
            ROUTE_DEMOTE,
            False,
            "no added_to_merge_queue timeline event found — the watchdog observation "
            "cannot be bound to this queue attempt",
        )
    if newest.observed < last_enqueued_at:
        return Route(
            ROUTE_DEMOTE,
            False,
            f"newest watchdog observation {iso(newest.observed)} predates this queue "
            f"attempt (enqueued {iso(last_enqueued_at)})",
        )
    if marker_age < 0 or marker_age > marker_staleness_seconds:
        return Route(
            ROUTE_DEMOTE,
            False,
            f"watchdog observation is {int(marker_age)}s old — outside the "
            f"{marker_staleness_seconds}s freshness bound",
        )
    count = live_suites(newest.head)
    if count is None:
        return Route(
            ROUTE_DEMOTE,
            False,
            f"could not re-derive the check-suite count for head {newest.head[:8]}",
        )
    if count != 0:
        return Route(
            ROUTE_DEMOTE,
            False,
            f"head {newest.head[:8]} has {count} check-suite(s) — this timeout "
            "followed checks that genuinely ran",
        )
    return Route(
        ROUTE_PRESERVE,
        True,
        f"{ZERO_DISPATCH_REASON} with 0 check-suites re-derived live on group head "
        f"{newest.head[:8]} — the merge_group event was never dispatched, so the diff "
        "is not at fault; verdict preserved and the PR re-armed",
    )


# ── I/O layer ────────────────────────────────────────────────────────────────────


def run_gh(argv: list[str]) -> str:
    result = subprocess.run(["gh", *argv], capture_output=True, text=True, check=False)
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "unknown gh failure"
        raise GhError(f"gh {' '.join(argv[:3])} failed: {detail}")
    return result.stdout


def run_gh_read(argv: list[str]) -> str:
    """Idempotent READ path: bounded transient-only retries (#3759 helper)."""
    try:
        return gh_retry.run_gh_read(argv)
    except gh_retry.GhFatalError as error:
        raise GhError(str(error)) from error


def _ndjson(text: str) -> list[dict]:
    out: list[dict] = []
    for line in text.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            parsed = json.loads(line)
        except json.JSONDecodeError as error:
            raise GhError(f"gh api returned a non-JSON page line: {error}") from error
        if isinstance(parsed, dict):
            out.append(parsed)
    return out


class Watchdog:
    def __init__(
        self,
        repo: str,
        *,
        branch: str = "main",
        grace_seconds: int = DEFAULT_GRACE_SECONDS,
        max_recoveries_per_pr: int = DEFAULT_MAX_RECOVERIES_PER_PR,
        recovery_window_seconds: int = DEFAULT_RECOVERY_WINDOW_SECONDS,
        max_recoveries_per_run: int = DEFAULT_MAX_RECOVERIES_PER_RUN,
        dry_run: bool = False,
        gh: Callable[[list[str]], str] = run_gh,
        gh_read: Callable[[list[str]], str] | None = None,
        log: Callable[[str], None] = print,
        now: Callable[[], datetime] | None = None,
    ) -> None:
        try:
            self.owner, self.name = repo.split("/", 1)
        except ValueError as error:
            raise ValueError("repo must be OWNER/REPOSITORY") from error
        if not self.owner or not self.name or "/" in self.name:
            raise ValueError("repo must be OWNER/REPOSITORY")
        if grace_seconds < 60:
            raise ValueError("grace_seconds must be at least 60")
        self.repo = repo
        self.branch = branch
        self.grace_seconds = grace_seconds
        self.max_recoveries_per_pr = max_recoveries_per_pr
        self.recovery_window_seconds = recovery_window_seconds
        self.max_recoveries_per_run = max_recoveries_per_run
        self.dry_run = dry_run
        self.gh = gh
        self.gh_read = gh_read if gh_read is not None else gh
        self.log = log
        self._now = now or (lambda: datetime.now(timezone.utc))

    # -- reads ---------------------------------------------------------------

    def _graphql(self, query: str, **variables: object) -> dict:
        argv = ["api", "graphql", "-f", f"query={query}"]
        for key, value in variables.items():
            flag = "-F" if isinstance(value, (int, bool)) else "-f"
            argv += [flag, f"{key}={value}"]
        response = json.loads(self.gh_read(argv))
        if response.get("errors"):
            raise GhError(f"GraphQL returned errors: {response['errors']}")
        return response.get("data") or {}

    def queue_entries(self) -> list[QueueEntry]:
        data = self._graphql(
            QUEUE_QUERY,
            owner=self.owner,
            name=self.name,
            branch=self.branch,
            first=QUEUE_ENTRY_PAGE,
        )
        queue = ((data.get("repository") or {}).get("mergeQueue")) or {}
        nodes = ((queue.get("entries") or {}).get("nodes")) or []
        entries: list[QueueEntry] = []
        for node in nodes:
            if not isinstance(node, dict):
                continue
            pull = node.get("pullRequest") or {}
            entries.append(
                QueueEntry(
                    pr_number=int(pull.get("number", 0)),
                    pr_id=str(pull.get("id", "")),
                    entry_id=str(node.get("id", "")),
                    position=int(node.get("position", 0)),
                    state=str(node.get("state", "")).upper(),
                    enqueued_at=parse_iso(node.get("enqueuedAt")),
                    base_oid=(node.get("baseCommit") or {}).get("oid"),
                    head_oid=(node.get("headCommit") or {}).get("oid"),
                )
            )
        return entries

    def check_suite_count(self, sha: str) -> int | None:
        """`total_count` of check-suites on a commit, or None if not establishable.

        The authoritative counter is `total_count`; page 1's array length is read as a
        cross-check so a truncated/garbled response can never be mistaken for a
        positively-observed zero.
        """
        try:
            raw = self.gh_read(
                [
                    "api",
                    "-X",
                    "GET",
                    f"repos/{self.repo}/commits/{sha}/check-suites?per_page=100",
                ]
            )
            payload = json.loads(raw)
        except (GhError, json.JSONDecodeError):
            return None
        if not isinstance(payload, dict):
            return None
        total = payload.get("total_count")
        suites = payload.get("check_suites")
        if not isinstance(total, int) or not isinstance(suites, list):
            return None
        if total == 0 and suites:
            return None
        return total

    def group_created_at(self, ref: str, head_oid: str) -> datetime | None:
        """Exact ref-build time from the repository activity API, matched on head SHA."""
        try:
            raw = self.gh_read(
                [
                    "api",
                    "-X",
                    "GET",
                    "--paginate",
                    "-q",
                    ".[]|@json",
                    f"repos/{self.repo}/activity"
                    f"?activity_type=branch_creation&per_page=100"
                    f"&ref=refs/heads/{ref}",
                ]
            )
            rows = _ndjson(raw)
        except GhError:
            return None
        stamps = [
            parse_iso(row.get("timestamp"))
            for row in rows
            if row.get("activity_type") == "branch_creation"
            and str(row.get("after", "")) == head_oid
        ]
        stamps = [stamp for stamp in stamps if stamp is not None]
        return max(stamps) if stamps else None

    def pr_markers(self, pr_number: int) -> list[Marker]:
        raw = self.gh_read(
            [
                "api",
                "-X",
                "GET",
                "--paginate",
                "-q",
                ".[]|@json",
                f"repos/{self.repo}/issues/{pr_number}/comments?per_page=100",
            ]
        )
        markers = [parse_marker(row.get("body")) for row in _ndjson(raw)]
        return [marker for marker in markers if marker is not None]

    def last_enqueued_at(self, pr_number: int) -> datetime | None:
        try:
            raw = self.gh_read(
                [
                    "api",
                    "-X",
                    "GET",
                    "--paginate",
                    "-q",
                    ".[]|@json",
                    f"repos/{self.repo}/issues/{pr_number}/timeline?per_page=100",
                ]
            )
            rows = _ndjson(raw)
        except GhError:
            return None
        stamps = [
            parse_iso(row.get("created_at"))
            for row in rows
            if row.get("event") == "added_to_merge_queue"
        ]
        stamps = [stamp for stamp in stamps if stamp is not None]
        return max(stamps) if stamps else None

    # -- mutations -----------------------------------------------------------

    def post_marker_comment(self, entry: QueueEntry, ref: str, observed: datetime) -> None:
        marker = render_marker(
            pr=entry.pr_number,
            head=entry.head_oid or "",
            base=entry.base_oid or "",
            ref=ref,
            suites=0,
            observed=observed,
        )
        body = "\n".join(
            [
                "> 🤖 **SPARQ agent** — merge-group watchdog.",
                "",
                f"This PR's merge-group ref `{ref}` (head `{(entry.head_oid or '')[:8]}`)"
                " has **zero check-suites** — GitHub Actions never dispatched the"
                " `merge_group` event for it, so no required check can ever report and"
                " the entry would hold the head of the queue until the ruleset's"
                " 60-minute `CI_TIMEOUT` fires (sparq-org/sparq#4652).",
                "",
                "Recovering by dequeue + re-enqueue so the group is rebuilt. **Nothing"
                " is wrong with this PR's diff**; its review verdict is preserved.",
                "",
                marker,
            ]
        )
        self.gh(["pr", "comment", str(entry.pr_number), "--repo", self.repo, "--body", body])

    def recover(self, entry: QueueEntry) -> None:
        self.gh(
            [
                "api",
                "graphql",
                "-f",
                f"query={DEQUEUE_MUTATION}",
                "-f",
                f"id={entry.pr_id}",
            ]
        )
        self.gh(
            [
                "api",
                "graphql",
                "-f",
                f"query={ENQUEUE_MUTATION}",
                "-f",
                f"id={entry.pr_id}",
            ]
        )

    # -- the sweep -----------------------------------------------------------

    def emit(self, row: str) -> None:
        self.log(f"[{PROGRAM}] {row}")
        summary = os.environ.get("GITHUB_STEP_SUMMARY")
        if summary:
            try:
                with open(summary, "a", encoding="utf-8") as handle:
                    handle.write(f"- `{row}`\n")
            except OSError:
                pass

    def sweep(self) -> int:
        now = self._now()
        entries = self.queue_entries()
        if not entries:
            self.emit(
                f"branch={self.branch} queue is EMPTY — 0 entries to watch "
                f"(grace={self.grace_seconds}s)"
            )
            return 0

        counts: dict[str, int] = {}
        errors = 0
        run_recoveries = 0
        for entry in sorted(entries, key=lambda e: e.position):
            ref = (
                queue_ref(self.branch, entry.pr_number, entry.base_oid)
                if entry.base_oid
                else "(no group)"
            )
            suites: int | None = None
            created_at: datetime | None = None
            markers: list[Marker] = []
            recoveries_in_window = 0

            if entry.state in ACTIONABLE_STATES and entry.head_oid and entry.base_oid:
                suites = self.check_suite_count(entry.head_oid)
                if suites == 0:
                    created_at = self.group_created_at(ref, entry.head_oid)
                    try:
                        markers = self.pr_markers(entry.pr_number)
                    except (GhError, json.JSONDecodeError) as error:
                        self.emit(
                            f"pos={entry.position} pr=#{entry.pr_number} ref={ref} "
                            f"head={(entry.head_oid or '')[:8]} suites=0 "
                            f"decision={REFUSE} — marker history unreadable ({error})"
                        )
                        counts[REFUSE] = counts.get(REFUSE, 0) + 1
                        errors += 1
                        continue
                    cutoff = now - timedelta(seconds=self.recovery_window_seconds)
                    recoveries_in_window = sum(
                        1 for marker in markers if marker.observed >= cutoff
                    )

            decision = decide_entry(
                entry,
                suites=suites,
                created_at=created_at,
                now=now,
                grace_seconds=self.grace_seconds,
                markers=markers,
                recoveries_in_window=recoveries_in_window,
                max_recoveries_per_pr=self.max_recoveries_per_pr,
                run_recoveries=run_recoveries,
                max_recoveries_per_run=self.max_recoveries_per_run,
            )

            suite_text = "unknown" if suites is None else str(suites)
            row = (
                f"pos={entry.position} pr=#{entry.pr_number} state={entry.state} "
                f"ref={ref} head={(entry.head_oid or '-')[:8]} suites={suite_text} "
                f"decision={decision.verdict} — {decision.detail}"
            )

            if decision.verdict == RECOVER and not self.dry_run:
                try:
                    self.post_marker_comment(entry, ref, now)
                    self.recover(entry)
                except GhError as error:
                    self.emit(f"{row} :: RECOVERY FAILED ({error})")
                    self.log(
                        f"::error title={PROGRAM} recovery failed::PR #{entry.pr_number} "
                        f"ref {ref}: {error}"
                    )
                    counts["RECOVER_FAILED"] = counts.get("RECOVER_FAILED", 0) + 1
                    errors += 1
                    continue
                run_recoveries += 1

            self.emit(row + (" [dry-run]" if self.dry_run and decision.verdict == RECOVER else ""))
            if decision.verdict in (CAP, REFUSE, NOOP):
                self.log(
                    f"::warning title={PROGRAM} {decision.verdict}::"
                    f"PR #{entry.pr_number} ref {ref}: {decision.detail}"
                )
            counts[decision.verdict] = counts.get(decision.verdict, 0) + 1

        census = " ".join(f"{key}={value}" for key, value in sorted(counts.items()))
        self.emit(f"sweep complete: entries={len(entries)} {census} errors={errors}")
        return errors

    # -- the dequeue routing split -------------------------------------------

    def classify_dequeue(self, pr_number: int, reason: str) -> Route:
        now = self._now()
        try:
            markers = self.pr_markers(pr_number)
        except (GhError, json.JSONDecodeError) as error:
            return Route(
                ROUTE_DEMOTE, False, f"marker history unreadable ({error})"
            )
        return classify_dequeue_route(
            reason=reason,
            markers=markers,
            last_enqueued_at=self.last_enqueued_at(pr_number),
            live_suites=self.check_suite_count,
            now=now,
        )


def write_outputs(route: Route, log: Callable[[str], None] = print) -> None:
    """Emit the routing verdict as a row AND as GitHub step outputs."""
    log(
        f"[{PROGRAM}] dequeue route={route.route} reenqueue="
        f"{'true' if route.reenqueue else 'false'} — {route.detail}"
    )
    path = os.environ.get("GITHUB_OUTPUT")
    if not path:
        return
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(f"route={route.route}\n")
        handle.write(f"reenqueue={'true' if route.reenqueue else 'false'}\n")
        handle.write(f"detail={route.detail}\n")


# ── self-test ────────────────────────────────────────────────────────────────────


def _entry(
    *,
    pr: int = 4534,
    position: int = 1,
    state: str = "AWAITING_CHECKS",
    base: str | None = "3cc1bf828c1335577069f5fa65d832c0ae1c8c38",
    head: str | None = "1bfb0174f5cc2da1ed9dfe7997b7ab089e7cab26",
) -> QueueEntry:
    return QueueEntry(
        pr_number=pr,
        pr_id=f"PR_{pr}",
        entry_id=f"MQE_{pr}",
        position=position,
        state=state,
        enqueued_at=parse_iso("2026-07-27T22:58:53Z"),
        base_oid=base,
        head_oid=head,
    )


def self_test() -> None:
    now = parse_iso("2026-07-27T23:20:00Z")
    assert now is not None
    built = parse_iso("2026-07-27T23:04:37Z")  # the real #4534 branch_creation time

    def call(**overrides):
        kwargs = dict(
            suites=0,
            created_at=built,
            now=now,
            grace_seconds=DEFAULT_GRACE_SECONDS,
            markers=(),
            recoveries_in_window=0,
            max_recoveries_per_pr=DEFAULT_MAX_RECOVERIES_PER_PR,
            run_recoveries=0,
            max_recoveries_per_run=DEFAULT_MAX_RECOVERIES_PER_RUN,
        )
        entry = overrides.pop("entry", _entry())
        kwargs.update(overrides)
        return decide_entry(entry, **kwargs)

    # The incident: zero suites, well past grace => RECOVER.
    assert call().verdict == RECOVER, call()
    # THE CONTROL: suites present but runs still pending => never touched.
    assert call(suites=8).verdict == HOLD, call(suites=8)
    # Fail safe: an unknown suite count is never a recovery.
    assert call(suites=None).verdict == REFUSE, call(suites=None)
    # Fail safe: no branch_creation row => no anchor => no action.
    assert call(created_at=None).verdict == REFUSE, call(created_at=None)
    # Grace gate, expressed against the constant so a retune cannot leave it stale.
    grace = DEFAULT_GRACE_SECONDS
    assert call(now=built + timedelta(seconds=grace - 1)).verdict == WAIT
    assert call(now=built + timedelta(seconds=grace)).verdict == RECOVER
    # ...and pinned against the MEASURED dispatch latency (N=209, max 4 s).
    assert grace >= 20 * 4 and grace <= 300, grace
    # Idempotence: a repeat detection on the SAME head is a no-op.
    same = Marker(
        pr=4534,
        head="1bfb0174f5cc2da1ed9dfe7997b7ab089e7cab26",
        base="",
        ref="",
        suites=0,
        observed=now - timedelta(minutes=5),
        action="re-enqueue",
    )
    assert call(markers=(same,)).verdict == NOOP, call(markers=(same,))
    # A marker for a DIFFERENT head does not suppress a fresh detection.
    other = Marker(**{**same.__dict__, "head": "a" * 40})
    assert call(markers=(other,), recoveries_in_window=1).verdict == RECOVER
    # Caps.
    assert call(recoveries_in_window=2).verdict == CAP
    assert call(run_recoveries=2).verdict == CAP
    # Non-actionable states.
    for state in ("QUEUED", "MERGEABLE", "UNMERGEABLE", "LOCKED"):
        assert call(entry=_entry(state=state)).verdict == SKIP, state
    assert call(entry=_entry(head=None)).verdict == SKIP

    # Ref reconstruction, pinned against live-verified data.
    assert (
        queue_ref("main", 4644, "c96f5abe3e0b7ba50c1381a1503ba961313b3da7")
        == "gh-readonly-queue/main/pr-4644-c96f5abe3e0b7ba50c1381a1503ba961313b3da7"
    )

    # Marker round-trip.
    rendered = render_marker(
        pr=4534,
        head="1bfb0174f5cc2da1ed9dfe7997b7ab089e7cab26",
        base="3cc1bf828c1335577069f5fa65d832c0ae1c8c38",
        ref="gh-readonly-queue/main/pr-4534-3cc1bf828c1335577069f5fa65d832c0ae1c8c38",
        suites=0,
        observed=now,
    )
    parsed = parse_marker("body text\n\n" + rendered)
    assert parsed is not None and parsed.head.startswith("1bfb0174"), rendered
    assert parsed.suites == 0 and parsed.action == "re-enqueue"
    assert parse_marker("no marker here") is None
    assert parse_marker(f"<!-- {MARKER_KEY} pr=1 head=nothex suites=0 observed=x -->") is None

    # Routing split — both arms.
    def route(**overrides):
        kwargs = dict(
            reason=ZERO_DISPATCH_REASON,
            markers=(parsed,),
            last_enqueued_at=now - timedelta(hours=1),
            live_suites=lambda _sha: 0,
            now=now + timedelta(minutes=40),
        )
        kwargs.update(overrides)
        return classify_dequeue_route(**kwargs)

    assert route().route == ROUTE_PRESERVE, route()
    assert route().reenqueue is True
    # The other arm: a CI_TIMEOUT that followed REAL checks still demotes.
    assert route(live_suites=lambda _s: 8).route == ROUTE_DEMOTE, route(live_suites=lambda _s: 8)
    # Unreadable count => demote (fail safe).
    assert route(live_suites=lambda _s: None).route == ROUTE_DEMOTE
    # No marker => demote.
    assert route(markers=()).route == ROUTE_DEMOTE
    # A marker predating the current attempt cannot steer the route.
    assert route(last_enqueued_at=now + timedelta(minutes=1)).route == ROUTE_DEMOTE
    # No added_to_merge_queue event => demote.
    assert route(last_enqueued_at=None).route == ROUTE_DEMOTE
    # A stale marker => demote.
    assert route(now=now + timedelta(hours=7)).route == ROUTE_DEMOTE
    # Reasons other than CI_TIMEOUT keep today's behaviour.
    for other_reason in ("CI_FAILURE", "MERGE_CONFLICT", "QUEUE_CLEARED", "ROLL_BACK"):
        assert route(reason=other_reason).route == ROUTE_DEMOTE, other_reason
    # The watchdog's OWN dequeue preserves without a second re-enqueue.
    own = route(reason="MANUAL", now=now + timedelta(seconds=30))
    assert own.route == ROUTE_PRESERVE and own.reenqueue is False, own
    # ...but only inside the self-dequeue window.
    assert route(reason="MANUAL", now=now + timedelta(hours=2)).route == ROUTE_DEMOTE
    # A conflict dequeue right after a recovery is NOT attributed to the watchdog.
    assert route(reason="MERGE_CONFLICT", now=now + timedelta(seconds=30)).route == ROUTE_DEMOTE

    print(f"{PROGRAM} self-test: PASS")


# ── entrypoint ───────────────────────────────────────────────────────────────────


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    sub = parser.add_subparsers(dest="command")

    sweep_parser = sub.add_parser("sweep", help="scan the merge queue for zero-dispatch")
    sweep_parser.add_argument("--repo", required=True)
    sweep_parser.add_argument("--branch", default="main")
    sweep_parser.add_argument("--grace-seconds", type=int, default=DEFAULT_GRACE_SECONDS)
    sweep_parser.add_argument(
        "--max-recoveries-per-pr", type=int, default=DEFAULT_MAX_RECOVERIES_PER_PR
    )
    sweep_parser.add_argument(
        "--recovery-window-seconds", type=int, default=DEFAULT_RECOVERY_WINDOW_SECONDS
    )
    sweep_parser.add_argument(
        "--max-recoveries-per-run", type=int, default=DEFAULT_MAX_RECOVERIES_PER_RUN
    )
    sweep_parser.add_argument("--dry-run", action="store_true")

    classify_parser = sub.add_parser(
        "classify-dequeue", help="route a pull_request.dequeued event"
    )
    classify_parser.add_argument("--repo", required=True)
    classify_parser.add_argument("--pr", type=int, required=True)
    classify_parser.add_argument("--reason", default="UNKNOWN")

    args = parser.parse_args(argv)

    if args.self_test:
        self_test()
        return 0

    if args.command == "classify-dequeue":
        # FAIL SAFE: any failure at all routes to `demote`, which is the pre-existing
        # behaviour. A classifier crash must never silently preserve a verdict.
        try:
            watchdog = Watchdog(args.repo, gh_read=run_gh_read)
            route = watchdog.classify_dequeue(args.pr, args.reason)
        except (GhError, ValueError, json.JSONDecodeError, gh_retry.GhTransientExhausted) as error:
            route = Route(ROUTE_DEMOTE, False, f"classification failed ({error})")
        write_outputs(route)
        return 0

    if args.command == "sweep":
        try:
            watchdog = Watchdog(
                args.repo,
                branch=args.branch,
                grace_seconds=args.grace_seconds,
                max_recoveries_per_pr=args.max_recoveries_per_pr,
                recovery_window_seconds=args.recovery_window_seconds,
                max_recoveries_per_run=args.max_recoveries_per_run,
                dry_run=args.dry_run,
                gh_read=run_gh_read,
            )
            return 1 if watchdog.sweep() else 0
        except gh_retry.GhTransientExhausted as error:
            # Periodic + idempotent: a missed cycle is harmless, the next tick covers
            # it. Still emitted loudly so the miss is never invisible.
            print(
                f"::warning title={PROGRAM} skipped a cycle on transient GitHub API "
                f"failures::{error}"
            )
            return 0
        except (GhError, ValueError, json.JSONDecodeError) as error:
            print(f"[{PROGRAM}] fatal: {error}", file=sys.stderr)
            return 1

    parser.error("a subcommand is required unless --self-test is used")
    return 2


if __name__ == "__main__":
    sys.exit(main())
