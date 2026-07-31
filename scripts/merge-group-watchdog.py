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
#   * the group head commit's `committer.date` — GitHub STAMPS the group commit rather
#     than dating it at build time. THE LOAD-BEARING EVIDENCE IS THE DISTRIBUTION, not
#     any single commit: over 40 live group heads, `branch_creation - committer.date`
#     spans 2 s to 3929 s with a median of 16 s, and only 1 of 40 falls within +/-2 s of
#     the real creation time. `1bfb0174`'s stamp sits 344 s BEFORE its ref existed.
#     (An earlier draft of this note argued from a parent-commit contradiction instead;
#     that argument is withdrawn, because the parent's own date is a stamp too and the
#     claim could not be independently re-derived. The distribution stands on its own.)
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
# `.github/workflows/merge-queue-feedback.yml` routed every non-merge dequeue to
# `review:changes` by design. For a zero-dispatch CI_TIMEOUT that is a
# MACHINE-MANUFACTURED re-review: #4534 lost `review:pass` for a platform event drop,
# with nothing whatsoever wrong in its diff. `classify-dequeue` splits the two cases by
# the SUITE COUNT (never by the timeout reason alone) and prints a `route=` output the
# workflow branches on. It also recognises the watchdog's OWN dequeue — our recovery is
# implemented as dequeue+enqueue, which itself fires `pull_request.dequeued`, and
# without this arm the watchdog would demote every PR it rescued.
#
# ── #4619: `review:changes` MUST NAME AN EXIT THE AUTHOR CAN SATISFY ──────────────
# The zero-dispatch split above is narrow: it only rescues a CI_TIMEOUT whose group ref
# had ZERO check-suites. #4331 is the other half — a CI_TIMEOUT after checks that really
# did run, with the head unmoved, `review:pass` still head-bound, and `gate`
# completed/success over 107 check-runs and zero non-passing. Routing THAT to
# `review:changes` tells an author to change something when there is nothing to change,
# and nothing re-queues it, so it is a hold with no machine exit.
#
# So the reason now decides OWNERSHIP first (`INFRA_DEQUEUE_REASONS`), and a
# queue/runner-owned dequeue gets a third route, `requeue`: keep the verdict, re-arm,
# comment — the same machine-owned shape as `preserve`, but BOUNDED by the queue-attempt
# count read off the PR's own `added_to_merge_queue` timeline events. When that budget is
# spent the route falls back to `demote`, so a PR that genuinely cannot clear the queue
# still ends up on a human's desk — it just is not sent there on attempt one for a fault
# it does not have. Code-flavoured reasons (CHECKS_FAILED, MERGE_CONFLICT, …) are decided
# BEFORE any evidence is read and are unchanged.
#
# ── A DEFENSIVE CONTROL MUST NOT AMPLIFY WHAT IT REJECTS ──────────────────────────
# Rejecting an untrusted marker emits ONE annotation per PR, never one per marker.
# Measured the hard way: a per-marker warning against a PR carrying 567 markers
# produced ~110 KB of log and 567 annotations — past GitHub's annotation cap, burying
# every other signal in the run. A filter that emits per-item output hands the attacker
# a log-flood denial of service against its own operator: the cheaper the input is to
# produce, the louder the defence gets. State the count and the distinct authors; never
# iterate the payload into the log. (110 KB -> 368 bytes on that same input.)
#
# ── the verdict-stripping defect this ALSO has to fix ─────────────────────────────
# MEASURED on #4709, 2026-07-28 — the dequeue itself takes the verdict down with it:
#     05:14:56  labeled   review:pass    by sparq-orchestrator[bot]
#     06:06:38  removed_from_merge_queue (reason MANUAL)
#     06:06:55  labeled   review:changes by github-actions[bot]
#     06:06:55  unlabeled review:pass    by github-actions[bot]
# Seventeen seconds. Because our recovery IS a dequeue, a watchdog without this fix
# would strip `review:pass`, apply `review:changes`, disable the arm and admit the PR to
# the FIX pipeline — on every PR it rescued. That is a net regression: it would
# manufacture a `review:changes` backlog out of a platform event that never fired.
#
# The existing routing is CORRECT for its original purpose — a reviewer withdrawing a PR
# *should* pull the verdict. The defect is that `MANUAL` conflates "a reviewer withdrew
# this" with "infrastructure moved it", and the handler keys off THAT a dequeue happened,
# never WHY. So we discriminate on EVIDENCE, never on the event name: a fresh
# watchdog marker for this exact group head is what makes a dequeue ours.
#
# NON-NEGOTIABLE PRECONDITION on every preserve route (`verdict_is_still_for_this_tree`):
# a verdict may only survive a dequeue if the head has NOT moved since it was granted.
# A verdict attests to a specific TREE, so if a commit, a force-push, or an
# `unlabeled review:pass` lands after the grant, the label must be allowed to fall off.
# Verified by hand on #4709 before restoring it: `review:pass` at 05:14:56Z, last commit
# 04:56:39Z, zero head-moving events between. Both the latest grant and the latest move
# are used, so the realistic push-then-re-approve sequence still preserves correctly.
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
# all; the budget is per PR per wall-clock window.
#
# EXHAUSTION BEHAVIOUR, stated as it actually is: the entry is handed back to the
# platform's own 60-minute CI_TIMEOUT, and that dequeue does NOT preserve the verdict.
# The reason is mechanical and worth knowing: recovery is marker -> dequeue -> enqueue,
# so the marker is always ~30 s OLDER than the `added_to_merge_queue` it produces; on
# the next attempt the newest trusted marker therefore predates that attempt and the
# queue-attempt binding refuses it.
#
# That is the CORRECT outcome, not merely a tolerated one. By exhaustion the PR has
# burned three consecutive zero-dispatch groups, the only trusted observation names a
# SUPERSEDED group head, and preserving a verdict on the strength of a stale head's
# suite count would be unsound. A human look is warranted at that point. What exhaustion
# does NOT do is stall: there is no `needs:user`, no permanent hold, and a CAP row is
# emitted every tick for as long as it holds.
#
# [OPUS-5] #4619 amends where that refused dequeue LANDS, and only that. It falls to the
# bounded re-queue lane, which is spent at the same moment by construction: each recovery
# mints one `added_to_merge_queue`, so 1 original arm + 2 recoveries = 3 attempts, which
# is exactly DEFAULT_MAX_QUEUE_ATTEMPTS. After that a CI_TIMEOUT demotes to
# `review:changes` as it always did — the human look is still the terminal state.
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

# ── DEQUEUE REASONS CARRY OWNERSHIP (#4619) ──────────────────────────────────────
# A dequeue reason says WHO can act on it, and that is what decides the exit state.
#   * CODE-flavoured (CHECKS_FAILED / CI_FAILURE / MERGE_CONFLICT / GIT_TREE_INVALID /
#     INVALID_MERGE_COMMIT / BRANCH_PROTECTIONS …) — the DIFF must change. `review:changes`
#     is exactly right: the author has an exit condition they can satisfy.
#   * QUEUE/RUNNER-flavoured — nothing in the diff is at fault, so `review:changes` names
#     an exit condition that DOES NOT EXIST. The case #4619 measured is PR #4331: a
#     CI_TIMEOUT dequeue with the head unmoved, `review:pass` still head-bound, and
#     `gate` completed/success over 107 check-runs with zero non-passing — latent at
#     N=1 when it was found. The author could change nothing to clear it, and nothing
#     else re-queued it, so it sat indefinitely — a hold with no machine exit.
# These route to ROUTE_REQUEUE instead: the verdict stays, the arm is restored, and the
# queue attempt itself is the re-verification. BOUNDED by the queue-attempt count so a PR
# that genuinely cannot get through the queue lands on the author's desk after all.
#
# QUEUE_CLEARED is deliberately NOT here. Clearing the queue is a HUMAN act, and a bot
# that re-queues what an operator just cleared fights its own operator; it keeps today's
# demote. ROLL_BACK is here because it means a SIBLING entry failed — by construction not
# this PR's diff.
INFRA_DEQUEUE_REASONS = frozenset({"CI_TIMEOUT", "ROLL_BACK"})
# How many times ONE tree may enter the merge queue under a single `review:pass` grant
# before the machine stops re-queueing it. The current attempt is counted, so 3 permits at
# most TWO machine re-queues; the third infra dequeue demotes to `review:changes`.
# This is the bound that stops a genuinely-unmergeable PR looping: the attempt count is
# read from the PR's own `added_to_merge_queue` timeline events, so every re-queue —
# ours, the watchdog's, or a human's — consumes the same budget.
DEFAULT_MAX_QUEUE_ATTEMPTS = 3
# The only action a marker may claim. Checked on the self-dequeue arm so that no other
# marker shape can ever be mistaken for a recovery in flight.
MARKER_ACTION_REENQUEUE = "re-enqueue"
# The verdict label whose survival across an infrastructure dequeue is the whole point
# of the routing split.
REVIEW_PASS = "review:pass"

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
# The MACHINE-OWNED exit (#4619). Like `preserve` it leaves the labels alone and re-arms;
# unlike `preserve` it does not claim the merge_group event was never dispatched, and it
# is bounded by DEFAULT_MAX_QUEUE_ATTEMPTS.
ROUTE_REQUEUE = "requeue"

# ── the required gate, re-verified at the unchanged head (#4619) ─────────────────
# A machine re-queue is only defensible when there is POSITIVE evidence that nothing needs
# changing, and the required check is that evidence. The name is matched by EXACT EQUALITY
# against the ready-tier name: sparq's ci-summary.yml publishes `gate` on a ready payload
# and `gate, draft-tier` on a draft one, and `gate` is a strict PREFIX of the draft name,
# so a prefix test would resolve the wrong tier. A draft cannot be in the merge queue at
# all, so the ready-tier name is the only correct lookup here.
GATE_CHECK_NAME = "gate"
GATE_SUCCESS = "success"
GATE_FAILURE = "failure"
GATE_PENDING = "pending"
GATE_MISSING = "missing"
GATE_UNKNOWN = "unknown"
# Mirrors scripts/rearm-sweeper.py's vocabulary so one operator reading both logs sees the
# same words. Only GATE_SUCCESS permits a machine re-queue; every other value demotes, so
# the cancelled/stale distinction that sweeper needs for its re-trigger is folded in here.
_GATE_OK_CONCLUSIONS = frozenset({"success", "neutral", "skipped"})
_GATE_BAD_CONCLUSIONS = frozenset(
    {"failure", "timed_out", "action_required", "startup_failure", "cancelled", "stale"}
)

# ── THE MARKER IS EVIDENCE, SO ITS AUTHOR IS PART OF THE PREDICATE ───────────────
# sparq is a PUBLIC repo and anyone can comment on a pull request. A marker read from an
# arbitrary comment author is UNAUTHENTICATED AUTHOR-CONTROLLED INPUT, and the routing
# split ACTS on it: a forged marker makes `review:pass` survive a dequeue and skips the
# arm-disable, so auto-arm.py / rearm-sweeper.py re-arm the PR within ~10 minutes — and
# on the CI_TIMEOUT arm it reaches `gh pr merge --auto` directly. Naming any commit that
# happens to have zero check-suites suffices, and such a sha is trivially obtainable.
#
# Markers are therefore accepted ONLY from this allow-list. This is the load-bearing
# control of the routing split, not a hardening nicety: it is the third occurrence in
# this estate of "the PR's own NAMED control was a forgeable channel", after #681
# (review evidence read from `pull["body"]`) and #937 (declarations from body/title).
# The check lives at the READ, so nothing downstream can forget to apply it.
TRUSTED_MARKER_AUTHORS = frozenset(
    {
        "github-actions[bot]",          # the workflow token's identity
        "sparq-orchestrator[bot]",      # the App used when ORCHESTRATOR_APP_ID is set
    }
)

MARKER_KEY = "merge-group-watchdog"
# BOUNDED ON PURPOSE. `[^>]*?` is quadratic under backtracking: a 65 536-char comment
# of repeated `<!-- merge-group-watchdog ` cost 21.4 s to parse, and four such comments
# took `classify-dequeue` to 63 s — ~19 would exceed merge-queue-feedback.yml's
# 5-minute job timeout. sparq is PUBLIC and that workflow runs on every dequeue, so
# that is an anonymous denial of service against merge-queue routing. A real marker's
# key/value run is 262 chars; 400 is comfortable headroom and makes the parse linear.
_MARKER_RE = re.compile(
    r"<!--\s*" + MARKER_KEY + r"\s+(?P<kv>[^>]{0,400}?)\s*-->",
)

# Contexts in which a marker is being QUOTED rather than asserted. A trusted bot that
# echoes PR content — and `sparq-orchestrator[bot]` really does, quoting file spans on
# worker PRs — would otherwise launder an attacker's marker through the author filter.
# Stripped before matching, so a quoted marker is never seen at all.
_FENCED_RE = re.compile(r"(?ms)^\s*(`{3,}|~{3,}).*?^\s*\1\s*$")
_INDENTED_CODE_RE = re.compile(r"(?m)^(?: {4,}|\t).*$")
_BLOCKQUOTE_RE = re.compile(r"(?m)^\s*>.*$")
_INLINE_CODE_RE = re.compile(r"`[^`\n]*`")


def strip_quoted_contexts(body: str) -> str:
    """Remove fenced/indented code, blockquotes and inline spans from a comment body.

    The watchdog's own marker sits on its own unquoted line, so this never removes a
    genuine marker; it removes every context in which one is being displayed rather
    than claimed.
    """
    for pattern in (_FENCED_RE, _INDENTED_CODE_RE, _BLOCKQUOTE_RE, _INLINE_CODE_RE):
        body = pattern.sub(" ", body)
    return body
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
    body = strip_quoted_contexts(body)
    match = _MARKER_RE.search(body)
    if not match:
        return None
    # NESTING DEPTH, not a substring test. `<!-- quoted: <!-- merge-group-watchdog … -->`
    # matches on the INNER opener, whose payload is perfectly clean — so inspecting the
    # match tells you nothing. What matters is whether an enclosing comment was still
    # open at the match position: more `<!--` than `-->` before it means this marker is
    # being displayed inside another comment, not asserted.
    prefix = body[: match.start()]
    if prefix.count("<!--") > prefix.count("-->"):
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
    # The watchdog only ever records a ZERO observation, so anything else was not
    # written by this tool. Validated so the field is load-bearing rather than dead
    # data — a mutant flipping it to 1 otherwise changed nothing observable.
    if suites != 0:
        return None
    # Self-consistency: `ref` must name the same PR as `pr`. A marker whose two
    # identity fields disagree was not written by this tool.
    ref = fields.get("ref", "")
    if f"/pr-{pr}-" not in ref:
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


@dataclass(frozen=True)
class VerdictState:
    """What the PR timeline says about the verdict and the head it was granted for."""

    review_pass_at: datetime | None
    head_moved_at: datetime | None
    last_enqueued_at: datetime | None
    readable: bool = True
    # Tracked SEPARATELY from head_moved_at purely so the emitted reason is accurate:
    # folding a label revocation into "the head moved" tells an operator a commit
    # landed when none did, and sends them looking for a push that does not exist.
    verdict_revoked_at: datetime | None = None
    # #4619's loop bound: how many `added_to_merge_queue` events this tree has accrued
    # since the CURRENT `review:pass` grant. `None` means "could not be established" —
    # an unbounded lane is not a lane, so that demotes like every other unprovable case.
    queue_attempts_since_verdict: int | None = None


def decide_entry(
    entry: QueueEntry,
    *,
    suites: int | None,
    created_at: datetime | None,
    now: datetime,
    grace_seconds: int,
    markers: Iterable[Marker],
    markers_readable: bool = True,
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
    if not markers_readable:
        return Decision(
            REFUSE,
            "this PR's marker history could not be read — an unreadable history is "
            "NOT evidence of no prior recovery, so the caps cannot be evaluated",
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
            f"{max_recoveries_per_pr}) — handing back to the platform CI_TIMEOUT, which "
            "WILL demote to review:changes (the only trusted observation names a "
            "superseded group head, so the verdict cannot be preserved soundly)",
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


def verdict_is_still_for_this_tree(state: VerdictState) -> tuple[bool, str]:
    """May the review verdict survive this dequeue?

    NON-NEGOTIABLE PRECONDITION on every preserve route. A verdict attests to a
    specific TREE; if the head has moved since `review:pass` was granted, the label
    describes code nobody reviewed and it must be allowed to fall off. Measured on
    #4709: `review:pass` at 05:14:56Z for head `c84c0810d681`, last commit 04:56:39Z,
    zero head-moving events between — which is exactly the check that made restoring it
    legitimate. Anything unprovable is a refusal to preserve.
    """
    if not state.readable:
        return False, "the PR timeline could not be read"
    if state.review_pass_at is None:
        return False, "no review:pass grant found on the timeline — no verdict to keep"
    if (
        state.verdict_revoked_at is not None
        and state.verdict_revoked_at > state.review_pass_at
    ):
        return False, (
            f"review:pass was REVOKED at {iso(state.verdict_revoked_at)}, after the "
            f"grant at {iso(state.review_pass_at)} — there is no live verdict to keep"
        )
    if state.head_moved_at is not None and state.head_moved_at > state.review_pass_at:
        return False, (
            f"the head moved at {iso(state.head_moved_at)}, after review:pass was "
            f"granted at {iso(state.review_pass_at)} — the verdict is not for this tree"
        )
    return True, f"review:pass granted {iso(state.review_pass_at)} and the head has not moved since"


def _gate_run_order(run: dict) -> tuple:
    """Newest-run ordering key: ISO-8601 Z sorts lexicographically, id breaks ties."""
    started = run.get("started_at")
    ident = run.get("id")
    return (
        started if isinstance(started, str) else "",
        ident if isinstance(ident, int) else -1,
    )


def resolve_head_gate(payload: object) -> str:
    """The newest `gate` check-run on a head, as one of the GATE_* values.

    Reads a `commits/<sha>/check-runs?check_name=gate` response. The NAME FILTER IS
    SERVER-SIDE, and that is the whole reason this is a handful of lines rather than the
    paginated scan `scripts/rearm-sweeper.py:resolve_gate` needs: sparq heads carry
    hundreds of check-runs (155 / 680 / 800 on the three stuck heads that sibling
    records for 2026-07-27), so an unfiltered first page contains no `gate` row at all
    and "short read" is indistinguishable from "no gate" unless the two are separate
    values. Filtering by name server-side leaves single digits, and `total_count` is
    still cross-checked against the returned array so a truncated response reads
    UNKNOWN, never MISSING.

    NEWEST RUN WINS, and nothing here relies on the endpoint's `filter` default having
    de-duplicated for us: GitHub leaves the cancelled twin of a superseded run on the
    head beside the run that replaced it, so "any cancelled row" would mask a green
    newest.
    """
    if not isinstance(payload, dict):
        return GATE_UNKNOWN
    total = payload.get("total_count")
    runs = payload.get("check_runs")
    # `isinstance(False, int)` is True, so a bool must be rejected explicitly.
    if isinstance(total, bool) or not isinstance(total, int):
        return GATE_UNKNOWN
    if not isinstance(runs, list):
        return GATE_UNKNOWN
    if len(runs) < total:
        return GATE_UNKNOWN
    # Belt-and-braces behind the server-side filter: match the tier's name EXACTLY, so
    # `gate, draft-tier` can never stand in for `gate`.
    rows = [
        run
        for run in runs
        if isinstance(run, dict) and run.get("name") == GATE_CHECK_NAME
    ]
    if not rows:
        return GATE_MISSING
    newest = max(rows, key=_gate_run_order)
    if newest.get("status") != "completed":
        return GATE_PENDING
    conclusion = newest.get("conclusion")
    if conclusion in _GATE_OK_CONCLUSIONS:
        return GATE_SUCCESS
    if conclusion in _GATE_BAD_CONCLUSIONS:
        return GATE_FAILURE
    return GATE_UNKNOWN


def classify_dequeue_route(
    *,
    reason: str,
    pr_number: int,
    markers: Iterable[Marker],
    verdict: VerdictState,
    live_suites: Callable[[str], int | None],
    head_gate: Callable[[], str],
    now: datetime,
    self_dequeue_window_seconds: int = DEFAULT_SELF_DEQUEUE_WINDOW_SECONDS,
    marker_staleness_seconds: int = DEFAULT_MARKER_STALENESS_SECONDS,
    max_queue_attempts: int = DEFAULT_MAX_QUEUE_ATTEMPTS,
) -> Route:
    """Route a dequeue to the author (`demote`) or to one of the two machine lanes.

    Three outcomes, keyed on OWNERSHIP first and evidence second:
      * `demote`   — the author must change the diff (or the machine lanes declined).
      * `preserve` — the merge_group event was never dispatched (#4652); zero suites.
      * `requeue`  — queue/runner-owned, the head's `gate` is green, budget remains
                     (#4619). The machine exit an infra dequeue previously lacked.

    FAIL-SAFE throughout: every arm that cannot POSITIVELY establish its precondition
    returns `demote`, which is the pre-existing behaviour. `head_gate` is a THUNK so the
    extra API read is only paid on the arm that needs it.
    """
    # BIND TO THIS PR. A marker is evidence about ONE pull request; without this a
    # marker written for a different PR (or naming a foreign zero-suite head) carries a
    # verdict through this PR's dequeue. Both the `pr=` field and the `ref=` segment
    # must agree with the PR being routed, so a single mistyped field cannot pass.
    own = [
        marker
        for marker in markers
        if marker.pr == pr_number and f"/pr-{pr_number}-" in marker.ref
    ]
    ordered = sorted(own, key=lambda m: m.observed, reverse=True)
    newest = ordered[0] if ordered else None
    normalised = (reason or "UNKNOWN").strip().upper()
    last_enqueued_at = verdict.last_enqueued_at

    # (0) OWNERSHIP FIRST (#4619). A reason the author can act on is theirs, full stop —
    # decided before any evidence is read, so no marker, gate or budget can ever carry a
    # CHECKS_FAILED / MERGE_CONFLICT dequeue away from `review:changes`.
    if normalised not in INFRA_DEQUEUE_REASONS and normalised not in SELF_DEQUEUE_REASONS:
        return Route(
            ROUTE_DEMOTE,
            False,
            f"dequeue reason {normalised} is author-owned — the diff itself must change",
        )

    # Applies to EVERY machine lane, before any of them is considered.
    keepable, why = verdict_is_still_for_this_tree(verdict)
    if not keepable:
        return Route(ROUTE_DEMOTE, False, f"verdict may not survive this dequeue: {why}")

    marker_age = (now - newest.observed).total_seconds() if newest is not None else None

    # (a) our OWN recovery. The watchdog's dequeue+enqueue fires pull_request.dequeued;
    # without this arm the watchdog would demote every PR it just rescued. MANUAL/UNKNOWN
    # conflate "a reviewer withdrew this" with "infrastructure moved it", so this arm —
    # and ONLY this arm — may act on them, and only on marker evidence.
    if normalised in SELF_DEQUEUE_REASONS:
        if newest is None:
            return Route(
                ROUTE_DEMOTE,
                False,
                "no merge-group-watchdog observation recorded for this PR",
            )
        if (
            newest.action != MARKER_ACTION_REENQUEUE
            or not 0 <= marker_age <= self_dequeue_window_seconds
        ):
            return Route(
                ROUTE_DEMOTE,
                False,
                f"dequeue reason {normalised} is not attributable to a watchdog recovery "
                f"(marker action {newest.action!r}, observed {int(marker_age)}s ago, "
                f"window {self_dequeue_window_seconds}s) — treated as a human withdrawal",
            )
        # DEFENCE IN DEPTH behind the author allow-list. This arm previously preserved on
        # the marker alone and compared its head to NOTHING, so "names that exact group
        # head" was not what the code implemented. Re-derive the count for the head the
        # marker names: a head that does not exist reads None, and one that was really
        # dispatched reads non-zero. Neither may carry a verdict through a dequeue.
        own_suites = live_suites(newest.head)
        if own_suites is None:
            return Route(
                ROUTE_DEMOTE,
                False,
                f"marker names head {newest.head[:8]} whose check-suite count cannot be "
                "read — not provably the watchdog's own recovery",
            )
        if own_suites != 0:
            return Route(
                ROUTE_DEMOTE,
                False,
                f"marker names head {newest.head[:8]}, which has {own_suites} "
                "check-suite(s) — that group WAS dispatched, so this is not a "
                "zero-dispatch recovery",
            )
        return Route(
            ROUTE_PRESERVE,
            False,
            f"dequeue reason {normalised} within {self_dequeue_window_seconds}s of the "
            f"watchdog's own recovery of head {newest.head[:8]} ({int(marker_age)}s ago, "
            "0 check-suites re-derived live) — the watchdog re-enqueues; verdict preserved",
        )

    # (b) the ZERO-DISPATCH preserve (#4652). The strongest evidence available — the
    # merge_group event was never dispatched at all — so it is tried first. Split by SUITE
    # COUNT, never by the reason alone. Each failed precondition is RECORDED rather than
    # returned: it is not a reason to demote, only a reason this stronger claim cannot be
    # made, and the weaker (bounded) re-queue lane below may still apply.
    decline = ""
    if normalised == ZERO_DISPATCH_REASON:
        if newest is None:
            decline = "no merge-group-watchdog observation recorded for this PR"
        elif last_enqueued_at is None:
            decline = (
                "no added_to_merge_queue timeline event found — the watchdog observation "
                "cannot be bound to this queue attempt"
            )
        elif newest.observed < last_enqueued_at:
            decline = (
                f"newest watchdog observation {iso(newest.observed)} predates this queue "
                f"attempt (enqueued {iso(last_enqueued_at)})"
            )
        elif marker_age < 0 or marker_age > marker_staleness_seconds:
            decline = (
                f"watchdog observation is {int(marker_age)}s old — outside the "
                f"{marker_staleness_seconds}s freshness bound"
            )
        elif newest.action != MARKER_ACTION_REENQUEUE:
            decline = f"marker action {newest.action!r} is not {MARKER_ACTION_REENQUEUE!r}"
        else:
            count = live_suites(newest.head)
            if count is None:
                decline = (
                    "could not re-derive the check-suite count for head "
                    f"{newest.head[:8]}"
                )
            elif count != 0:
                decline = (
                    f"head {newest.head[:8]} has {count} check-suite(s) — this timeout "
                    "followed checks that genuinely ran"
                )
            else:
                return Route(
                    ROUTE_PRESERVE,
                    True,
                    f"{ZERO_DISPATCH_REASON} with 0 check-suites re-derived live on group "
                    f"head {newest.head[:8]} — the merge_group event was never dispatched, "
                    "so the diff is not at fault; verdict preserved and the PR re-armed",
                )

    # (c) the BOUNDED machine re-queue (#4619). The reason is queue/runner-owned, so
    # `review:changes` would name an exit condition the author cannot satisfy. Two
    # positive preconditions, both required:
    #   * a BOUND, so a PR that genuinely cannot clear the queue stops looping and is
    #     handed to a human after all;
    #   * live proof there is nothing to change — the required `gate` green on the head,
    #     which `verdict_is_still_for_this_tree` has already proven is the head the
    #     verdict was granted for.
    # NOTE, honestly: a green head gate proves the PR's OWN tree is sound. It does NOT
    # predict the merge-group run, which composes this PR with the entries ahead of it.
    # The attempt bound, not the gate, is what stops a repeatedly-failing group looping.
    attempts = verdict.queue_attempts_since_verdict
    suffix = f" (zero-dispatch preserve declined: {decline})" if decline else ""
    if attempts is None:
        return Route(
            ROUTE_DEMOTE,
            False,
            "the queue-attempt count for this verdict could not be established — an "
            f"unbounded re-queue lane is not a lane, so this demotes{suffix}",
        )
    if attempts >= max_queue_attempts:
        return Route(
            ROUTE_DEMOTE,
            False,
            f"this tree has entered the merge queue {attempts} time(s) since review:pass "
            f"was granted (bound {max_queue_attempts}) — the machine re-queue budget is "
            f"exhausted, so the PR is handed to the author{suffix}",
        )
    gate = head_gate()
    if gate != GATE_SUCCESS:
        return Route(
            ROUTE_DEMOTE,
            False,
            f"the PR head's required `{GATE_CHECK_NAME}` check reads {gate}, not "
            f"{GATE_SUCCESS} — no live proof that nothing needs changing{suffix}",
        )
    return Route(
        ROUTE_REQUEUE,
        True,
        f"dequeue reason {normalised} is queue/runner-owned and `{GATE_CHECK_NAME}` is "
        f"{GATE_SUCCESS} on the unchanged head (queue attempt {attempts}/"
        f"{max_queue_attempts}) — nothing for the author to change, so the verdict is "
        f"kept and the PR re-queued rather than demoted to review:changes{suffix}",
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
        # LATE-BOUND ON PURPOSE. `gh: Callable = run_gh` binds the module function at
        # DEFINITION time, so a test that patches `merge_group_watchdog.run_gh` does not
        # change it and the real runner is used anyway. That defect made this suite post
        # 567 real comments to production PR #4534 during a mutation sweep: main() passes
        # gh_read= explicitly (late-bound, patched) but let `gh` fall through to the
        # stale default, so every simulated sweep issued a REAL `gh pr comment`.
        # None means "resolve at call time", which is what a test patch can reach.
        gh: Callable[[list[str]], str] | None = None,
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
        self.gh = gh if gh is not None else run_gh
        self.gh_read = gh_read if gh_read is not None else self.gh
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
        # `isinstance(False, int)` is True, so a bool must be rejected EXPLICITLY:
        # `{"total_count": false}` otherwise reads as a positively-observed ZERO and
        # drives a recovery + marker write + preserve off a malformed response.
        if isinstance(total, bool) or not isinstance(total, int):
            return None
        if not isinstance(suites, list):
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
        """Trusted markers only — see TRUSTED_MARKER_AUTHORS.

        A well-formed marker from an untrusted author is DROPPED and COUNTED, so a
        forgery attempt is visible in the log rather than silently discarded.
        """
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
        trusted: list[Marker] = []
        rejected = 0
        rejected_authors: set[str] = set()
        for row in _ndjson(raw):
            # AUTHOR FIRST, ALWAYS. The trust check used to run AFTER parse_marker,
            # so every anonymous comment was parsed before being discarded — a filter
            # placed after the expensive work does not protect the expensive work.
            # Cheap string compare first: untrusted bodies are never parsed at all.
            author = str((row.get("user") or {}).get("login", ""))
            if author not in TRUSTED_MARKER_AUTHORS:
                if MARKER_KEY in str(row.get("body") or ""):
                    rejected += 1
                    rejected_authors.add(author)
                continue
            marker = parse_marker(row.get("body"))
            if marker is not None:
                trusted.append(marker)
            continue
        if rejected:
            # ONE annotation per PR, not one per marker. MEASURED on #4534, which
            # already carries 96 forged markers naming the real zero-suite head: a
            # per-marker warning emitted 96 annotations and ~110 KB of log, which
            # would blow GitHub's annotation cap and bury every other signal. The
            # count and the distinct authors are what an operator needs.
            self.log(
                f"::warning title={PROGRAM} untrusted markers ignored::PR #{pr_number}: "
                f"{rejected} watchdog marker(s) from untrusted author(s) "
                f"{sorted(rejected_authors)} IGNORED ({len(trusted)} trusted accepted) — "
                f"markers are honoured only from {sorted(TRUSTED_MARKER_AUTHORS)}"
            )
        return trusted

    def verdict_state(self, pr_number: int) -> VerdictState:
        """One timeline read for all three facts the routing decision needs.

        A `committed` timeline event carries no `created_at` — its time is the
        commit's own `committer.date`. `head_ref_force_pushed` is a normal event with
        `created_at`. Both move the head, so both count.
        """
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
            return VerdictState(None, None, None, readable=False)

        enqueued: list[datetime] = []
        granted: list[datetime] = []
        moved: list[datetime] = []
        revoked: list[datetime] = []
        for row in rows:
            event = row.get("event")
            stamp = parse_iso(row.get("created_at"))
            if event == "added_to_merge_queue" and stamp:
                enqueued.append(stamp)
            elif event == "labeled" and stamp:
                if str((row.get("label") or {}).get("name", "")).lower() == REVIEW_PASS:
                    granted.append(stamp)
            elif event == "unlabeled" and stamp:
                # A verdict that was REMOVED is not a verdict, and it is recorded as a
                # revocation rather than a head move so the reason stays truthful.
                if str((row.get("label") or {}).get("name", "")).lower() == REVIEW_PASS:
                    revoked.append(stamp)
            elif event == "head_ref_force_pushed" and stamp:
                moved.append(stamp)
            elif event == "committed":
                committed = parse_iso(
                    (row.get("committer") or {}).get("date")
                ) or parse_iso((row.get("author") or {}).get("date"))
                if committed:
                    moved.append(committed)
        review_pass_at = max(granted) if granted else None
        return VerdictState(
            review_pass_at=review_pass_at,
            head_moved_at=max(moved) if moved else None,
            last_enqueued_at=max(enqueued) if enqueued else None,
            verdict_revoked_at=max(revoked) if revoked else None,
            # #4619's bound. Counted from the CURRENT grant, so a re-review after a push
            # legitimately resets the budget — the tree being queued is a different one.
            # `None` when there is no grant to count from; that is not "zero attempts".
            queue_attempts_since_verdict=(
                sum(1 for stamp in enqueued if stamp >= review_pass_at)
                if review_pass_at is not None
                else None
            ),
        )

    def head_gate_state(self, pr_number: int) -> str:
        """The required `gate` check on the PR's CURRENT head, as a GATE_* value.

        Every failure is GATE_UNKNOWN, which denies the re-queue lane — this read may
        only ever license an action, never excuse one.
        """
        try:
            sha = self.gh_read(
                [
                    "pr",
                    "view",
                    str(pr_number),
                    "--repo",
                    self.repo,
                    "--json",
                    "headRefOid",
                    "-q",
                    ".headRefOid",
                ]
            ).strip()
        except GhError:
            return GATE_UNKNOWN
        if not re.fullmatch(r"[0-9a-f]{40}", sha):
            return GATE_UNKNOWN
        try:
            payload = json.loads(
                self.gh_read(
                    [
                        "api",
                        "-X",
                        "GET",
                        f"repos/{self.repo}/commits/{sha}/check-runs"
                        f"?check_name={GATE_CHECK_NAME}&per_page=100",
                    ]
                )
            )
        except (GhError, json.JSONDecodeError):
            return GATE_UNKNOWN
        return resolve_head_gate(payload)

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
        ordered = sorted(entries, key=lambda e: e.position)
        for entry in ordered:
            # THE EFFECTIVENESS TERM. Every entry behind this one is chained ON TOP of
            # its group head, so a rebuild discards all of their CI too. The real cost
            # of a dead ref is therefore not the ref — it is how much work stacks above
            # it before anyone notices:
            #     cost ~= (groups stacked above the dead ref at detection) x (their CI)
            # Measured 2026-07-28: intervening at 48 min meant that term was 4 (two of
            # them already MERGEABLE, i.e. fully green, and all four were discarded on
            # the rebuild). Firing inside the grace should make it 0. Recording it on
            # every row makes the watchdog self-measuring: if `stacked` starts climbing
            # the grace has drifted too long, and the rows say so without anyone having
            # to notice an outage by hand.
            stacked = [
                other
                for other in ordered
                if other.position > entry.position and other.head_oid
            ]
            stacked_green = [o for o in stacked if o.state == "MERGEABLE"]
            ref = (
                queue_ref(self.branch, entry.pr_number, entry.base_oid)
                if entry.base_oid
                else "(no group)"
            )
            suites: int | None = None
            created_at: datetime | None = None
            markers: list[Marker] = []
            markers_readable = True
            recoveries_in_window = 0

            if entry.state in ACTIONABLE_STATES and entry.head_oid and entry.base_oid:
                suites = self.check_suite_count(entry.head_oid)
                if suites == 0:
                    created_at = self.group_created_at(ref, entry.head_oid)
                    try:
                        markers = self.pr_markers(entry.pr_number)
                    except (GhError, json.JSONDecodeError) as error:
                        # An unreadable history is NOT "no prior recovery" — assuming
                        # that would bypass the cap on every API blip. It falls through
                        # to the ONE decide/emit path below as a REFUSE, so there is a
                        # single row shape for an operator to parse.
                        markers_readable = False
                        errors += 1
                        self.log(
                            f"::warning title={PROGRAM} marker history unreadable::"
                            f"PR #{entry.pr_number}: {error}"
                        )
                    else:
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
                markers_readable=markers_readable,
                recoveries_in_window=recoveries_in_window,
                max_recoveries_per_pr=self.max_recoveries_per_pr,
                run_recoveries=run_recoveries,
                max_recoveries_per_run=self.max_recoveries_per_run,
            )

            suite_text = "unknown" if suites is None else str(suites)
            row = (
                f"pos={entry.position} pr=#{entry.pr_number} state={entry.state} "
                f"ref={ref} head={(entry.head_oid or '-')[:8]} suites={suite_text} "
                f"stacked={len(stacked)} stacked_green={len(stacked_green)} "
                f"decision={decision.verdict} — {decision.detail}"
            )

            if decision.verdict == RECOVER and not self.dry_run:
                try:
                    self.post_marker_comment(entry, ref, now)
                    # PROVE THE EVIDENCE CHANNEL BEFORE RELYING ON IT. The caps and the
                    # idempotence key are all read back through TRUSTED_MARKER_AUTHORS.
                    # If this runner's identity is not on that list, every marker we
                    # write is invisible to us: the per-head NOOP never fires, the
                    # per-PR cap never binds, and the watchdog degrades into a
                    # dequeue/enqueue thrash loop bounded only by the per-run budget.
                    # So read the marker back and refuse to act if it is not there.
                    if not any(
                        marker.head == entry.head_oid
                        for marker in self.pr_markers(entry.pr_number)
                    ):
                        raise GhError(
                            "the marker just posted is not readable back as TRUSTED — "
                            "refusing to dequeue. TRUSTED_MARKER_AUTHORS is probably "
                            "missing this runner's identity, and acting without a "
                            "readable marker would uncap the recovery loop"
                        )
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
            pr_number=pr_number,
            markers=markers,
            verdict=self.verdict_state(pr_number),
            live_suites=self.check_suite_count,
            # A THUNK: the extra two reads are only paid on the arm that needs them.
            head_gate=lambda: self.head_gate_state(pr_number),
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
        # `detail` can embed gh stderr, which is multi-line. A bare `k=v` line would
        # spill the remainder into the outputs file as further keys, so use the
        # heredoc form with a delimiter that cannot survive in the payload.
        delimiter = "MGW_DETAIL_EOF"
        detail = route.detail.replace(delimiter, "MGW-DETAIL-EOF")
        handle.write(f"detail<<{delimiter}\n{detail}\n{delimiter}\n")


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

    # Routing split — all three arms.
    def verdict_state(**overrides):
        fields = dict(
            review_pass_at=now - timedelta(hours=2),
            head_moved_at=now - timedelta(hours=3),
            last_enqueued_at=now - timedelta(hours=1),
            queue_attempts_since_verdict=1,
        )
        fields.update(overrides)
        return VerdictState(**fields)

    def route(**overrides):
        kwargs = dict(
            reason=ZERO_DISPATCH_REASON,
            pr_number=4534,
            markers=(parsed,),
            verdict=verdict_state(),
            live_suites=lambda _sha: 0,
            head_gate=lambda: GATE_SUCCESS,
            now=now + timedelta(minutes=40),
        )
        kwargs.update(overrides)
        return classify_dequeue_route(**kwargs)

    assert route().route == ROUTE_PRESERVE, route()
    assert route().reenqueue is True
    # #4619 — a CI_TIMEOUT that followed REAL checks is still not the AUTHOR's to fix.
    # It leaves the zero-dispatch arm (that claim needs zero suites) and lands on the
    # BOUNDED re-queue lane instead of `review:changes`.
    ran = route(live_suites=lambda _s: 8)
    assert ran.route == ROUTE_REQUEUE and ran.reenqueue is True, ran
    assert "check-suite(s)" in ran.detail, ran
    # ...and that lane is bounded: on the last permitted attempt it hands over.
    assert (
        route(
            live_suites=lambda _s: 8,
            verdict=verdict_state(queue_attempts_since_verdict=DEFAULT_MAX_QUEUE_ATTEMPTS),
        ).route
        == ROUTE_DEMOTE
    )
    # ...and it needs live proof there is nothing to change.
    for gate in (GATE_FAILURE, GATE_PENDING, GATE_MISSING, GATE_UNKNOWN):
        assert route(live_suites=lambda _s: 8, head_gate=lambda g=gate: g).route == ROUTE_DEMOTE, gate
    # ...and an unestablishable attempt count is not "zero attempts".
    assert (
        route(
            live_suites=lambda _s: 8,
            verdict=verdict_state(queue_attempts_since_verdict=None),
        ).route
        == ROUTE_DEMOTE
    )
    # A head that moved AFTER the verdict was granted => the verdict may not survive,
    # on EVERY machine lane.
    assert route(verdict=verdict_state(head_moved_at=now - timedelta(minutes=1))).route == ROUTE_DEMOTE
    # No review:pass at all => nothing to preserve.
    assert route(verdict=verdict_state(review_pass_at=None)).route == ROUTE_DEMOTE
    # An unreadable timeline => demote.
    assert route(verdict=verdict_state(review_pass_at=None, head_moved_at=None,
                                       last_enqueued_at=None, readable=False)).route == ROUTE_DEMOTE
    # CODE-flavoured reasons are the author's, decided before any evidence is read.
    for other_reason in ("CI_FAILURE", "MERGE_CONFLICT", "CHECKS_FAILED", "QUEUE_CLEARED"):
        assert route(reason=other_reason).route == ROUTE_DEMOTE, other_reason
    # ROLL_BACK is a SIBLING entry's failure — machine-owned, so it re-queues.
    assert route(reason="ROLL_BACK").route == ROUTE_REQUEUE
    # The watchdog's OWN dequeue preserves without a second re-enqueue.
    own = route(reason="MANUAL", now=now + timedelta(seconds=30))
    assert own.route == ROUTE_PRESERVE and own.reenqueue is False, own
    # ...but only inside the self-dequeue window, and MANUAL never reaches the re-queue
    # lane: a human withdrawal is a human decision.
    assert route(reason="MANUAL", now=now + timedelta(hours=2)).route == ROUTE_DEMOTE
    assert route(reason="MANUAL", markers=()).route == ROUTE_DEMOTE
    # A conflict dequeue right after a recovery is NOT attributed to the watchdog.
    assert route(reason="MERGE_CONFLICT", now=now + timedelta(seconds=30)).route == ROUTE_DEMOTE

    # The head-gate resolver: newest run wins, short reads are UNKNOWN not MISSING.
    def _gate_payload(rows, total=None):
        return {"total_count": len(rows) if total is None else total, "check_runs": rows}

    assert resolve_head_gate(_gate_payload([
        {"name": "gate", "status": "completed", "conclusion": "cancelled",
         "started_at": "2026-07-27T22:00:00Z", "id": 1},
        {"name": "gate", "status": "completed", "conclusion": "success",
         "started_at": "2026-07-27T23:00:00Z", "id": 2},
    ])) == GATE_SUCCESS
    assert resolve_head_gate(_gate_payload([
        {"name": "gate", "status": "completed", "conclusion": "failure",
         "started_at": "2026-07-27T23:00:00Z", "id": 2},
    ])) == GATE_FAILURE
    assert resolve_head_gate(_gate_payload([{"name": "gate", "status": "in_progress"}])) == GATE_PENDING
    assert resolve_head_gate(_gate_payload([])) == GATE_MISSING
    assert resolve_head_gate(_gate_payload([{"name": "gate, draft-tier", "status": "completed",
                                             "conclusion": "success"}])) == GATE_MISSING
    assert resolve_head_gate(_gate_payload([], total=7)) == GATE_UNKNOWN
    assert resolve_head_gate("not-a-payload") == GATE_UNKNOWN
    assert resolve_head_gate({"total_count": True, "check_runs": []}) == GATE_UNKNOWN

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
