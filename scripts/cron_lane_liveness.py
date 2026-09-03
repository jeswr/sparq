#!/usr/bin/env python3
# [OPUS-5] Cron-only lane LIVENESS alarm (issue #4328). 🤖 SPARQ agent.
#
# WHY THIS EXISTS
# ---------------
# `bench-ec2.yml` failed EVERY scheduled run for 12 consecutive days
# (2026-07-15 → 2026-07-25, 13 × "Credentials could not be loaded") and nobody
# noticed. The reason is structural, not incidental: that workflow was
# `schedule:` + `workflow_dispatch:` only, so a failing run produces **no PR, no
# review, no merge-queue entry, no check-run on anybody's head commit** — it is
# invisible to every signal the fleet actually watches. The same shape killed
# `miri.yml` for 22 nights (see scripts/formal_lane_alarm.py).
#
# This script is the GENERAL fix for that class: it enumerates every cron-only
# lane in `.github/workflows/`, derives each lane's liveness expectation FROM
# ITS OWN `schedule:` cron (no per-workflow threshold table — a hard-coded table
# goes stale exactly the way the stale `jeswr/sparq` doc did), and raises one
# deduplicated GitHub issue per dead lane.
#
# SCOPE (derived, never hard-coded)
# ---------------------------------
# A workflow is IN SCOPE iff:
#   * its `on:` block has a `schedule:` key, AND
#   * every other `on:` key is in {schedule, workflow_dispatch}. Any other
#     trigger (pull_request / push / merge_group / workflow_run / issues / …)
#     makes the lane visible through a normal fleet signal, so it is out of
#     scope; and
#   * it is not already watched by another alarm. The only such alarm today is
#     formal-alarm.yml, whose watched set is READ OUT OF ci/formal-verification.toml
#     `[[lane]]` — derived from that manifest, not copied into a list here, so
#     adding a lane there automatically removes it from this alarm's scope.
#
# THE EXPECTATION, DERIVED FROM `schedule:`
# -----------------------------------------
# `derive_period_hours()` expands the lane's own cron expressions over the
# SIM_WINDOW_DAYS window (400 days — long enough to see two fires of any cron
# GitHub accepts, including a quarterly one) and takes the LARGEST gap between
# consecutive fire times (the union of
# all `- cron:` entries). That is the worst-case interval at which the lane is
# expected to produce a run: 24h for `41 5 * * *`, 168h for `0 6 * * 1`, 10min
# for `*/10 * * * *`. The staleness threshold is then
#
#     threshold_hours = max(period_hours * GRACE_PERIODS, MIN_GRACE_HOURS)
#
# i.e. three missed ticks, with a 6-hour floor so ordinary GitHub scheduling
# jitter (scheduled runs are routinely delayed or dropped under load) on a
# sub-hourly lane can never raise.
#
# TWO INDEPENDENT RAISE RULES (either one is enough)
# --------------------------------------------------
#   A. CONSECUTIVE-FAILURE — the newest CONSEC_FAILURES completed scheduled runs
#      all concluded in {failure, timed_out, startup_failure}. This is the
#      bench-ec2 shape: loudly red, every tick, seen by nobody.
#   B. NO-GREEN-IN-WINDOW — no `success` conclusion newer than `threshold_hours`.
#      This catches the shape rule A cannot: `cancelled`-at-the-timeout-ceiling
#      (the miri shape) and a lane that simply stopped firing.
#
# DETECTOR LANES: WHY `failure` IS NOT ALWAYS AN ILLNESS SIGNAL
# -------------------------------------------------------------
# Rules A and B both read the RUN CONCLUSION as a proxy for lane health. For
# most lanes that is right. For a DETECTOR lane it is exactly wrong: a detector
# exits non-zero when it FINDS something, so red is its verdict, not its
# breakage. Two such lanes are in this scan set today —
# `scripts/review_lane_alarm.py:504` `return 1` immediately after printing
# `::error::N open PR(s) have no countable review verdict…`, and
# `scripts/formal_lane_alarm.py:283` `return 1`, commented "LOUD: the alarm run
# itself goes red while any lane is verdict-less." Judged by A/B, a working
# review-alarm reads as a dead lane for as long as the review blind spot it
# reports persists — a false positive against a lane that is doing its job.
# False alarms train people to ignore alarms, which is the failure this whole
# file exists to prevent, so this is not a threshold to widen.
#
#   V. A lane may DECLARE, in its OWN YAML, that its non-zero exit is a verdict:
#      a line `# cron-liveness: verdict-lane` at column 0 (anchored there so the
#      declaration cannot be smuggled in from inside an indented `run:` block).
#      The declaration lives with the lane, so it moves and dies with the lane —
#      there is still no central table to rot, which was the whole design.
#
#      A declared lane is NOT muted. Rules A and B are replaced by rule V:
#      raise unless some completed scheduled run inside `threshold_hours`
#      actually SPOKE — conclusion in {success, failure}, the detector's own two
#      exit states. `cancelled` / `timed_out` / `startup_failure` are NOT
#      verdicts (the detector never got to speak), so the miri
#      cancelled-at-ceiling shape and a lane that stops firing altogether still
#      raise on a declared lane exactly as they do on any other.
#
#      HONEST BLIND SPOT, stated rather than hidden: a detector whose own
#      infrastructure is broken also exits non-zero, and the runs API cannot
#      distinguish that from a finding. On a declared lane that failure mode is
#      invisible here. That is the cost of the declaration, and it is why the
#      declaration is per-lane and printed in the run summary
#      (status LIVE-VERDICT-LANE) rather than inferred — a reader can see which
#      lanes are on the reduced rule. Inferring it instead from
#      `.github/advisory-registry.json` was rejected: that registry answers
#      "may this check block a merge", which is a strictly wider set — `asan.yml`
#      is declared advisory there and is precisely a lane whose nightly red DOES
#      mean it is broken.
#
# FAIL-SAFE IN THE QUIET DIRECTION (mandated: false alarms train people to
# ignore alarms, which is how this class recurs). NONE of these raise:
#   * a lane with zero completed scheduled runs (never ran yet);
#   * a lane whose GitHub workflow `state` is not `active` (disabled manually or
#     for inactivity — a deliberate off switch);
#   * a lane whose cron cannot be parsed, or whose cron fires fewer than twice in
#     the SIM_WINDOW_DAYS window (period indeterminate ⇒ no expectation ⇒ no
#     alarm);
#   * a NEW lane with no success yet whose FIRST observed run is younger than the
#     threshold (grace period — otherwise every newly-added lane alarms on its
#     first red tick);
#   * a lane whose newest completed scheduled run concluded `skipped` — a run
#     whose jobs all skipped is a deliberate, visible guard firing (bench-ec2's
#     `vars.AWS_BENCH_ROLE_ARN != ''` role-present guard is exactly this), not a
#     dead lane. KNOWN LIMITATION, stated rather than hidden: a lane that begins
#     skipping WRONGLY goes quiet here. That is the deliberate trade — those runs
#     are reported in the run summary as DELIBERATELY-INERT so they are visible
#     to a reader even though they raise nothing.
#
# QUIET IS NOT RECOVERED
# ----------------------
# The five states above are QUIET; only one of them is EVIDENCE OF HEALTH. An
# open alarm is auto-closed IFF `rec["recovered"]` — a real `success` inside the
# window (or, on a declared verdict-lane, a real verdict inside the window) —
# and the closing comment quotes that evidence. A lane that goes disabled, loses
# its run history, or starts skipping keeps its alarm OPEN, and main() prints it
# as LEFT OPEN. Closing on "it stopped raising" would state something false and
# re-create, inside the fix, the invisibility this file exists to end.
#
# EXIT CODES — and why they differ from formal_lane_alarm.py
# ----------------------------------------------------------
#   0 — the scan completed. Includes "dead lanes found AND their issues were
#       successfully filed/updated". THE ISSUE IS THE SIGNAL, not a red run.
#   2 — alarm-INFRASTRUCTURE error (cannot read a workflow, the runs API
#       disagrees with its own total_count, `gh issue create` failed, …). A
#       broken detector must never look like a quiet green.
#
# This deliberately diverges from formal_lane_alarm.py, which exits 1 while a
# lane is dead. That pattern cannot be used here: THIS workflow is itself a
# cron-only lane and therefore inside its own scope, so redding its own run on
# every finding would make the detector permanently report ITSELF as dead. Note
# the exit-zero risk this creates is closed at the source: a finding that cannot
# be turned into an issue raises AlarmError ⇒ exit 2. A finding is never
# silently dropped.
#   HONEST RESIDUAL: nothing watches this watcher if it dies COMPLETELY (it is
#   in its own scan set, so it catches its own intermittent failure, but a lane
#   that never runs cannot report on itself). Closing that needs an off-repo
#   heartbeat, which is out of scope here.
#
# NOT A GATE: schedule-only, so it creates no check-run on any PR head or
# merge-group ref and can never enter the `ci-summary / gate` poll set.
#
# Usage:
#   cron_lane_liveness.py                                  # live (gh; $GITHUB_REPOSITORY)
#   cron_lane_liveness.py --dry-run                        # scan + print; no gh writes
#   cron_lane_liveness.py --state-file s.json --now ISO \
#                         --dry-run                        # hermetic (tests)
#
# Stdlib + PyYAML (already a CI dependency) + the `gh` CLI (on every runner).

from __future__ import annotations

import argparse
import datetime as dt
import functools
import json
import os
import re
import subprocess
import sys
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_WORKFLOWS_DIR = REPO_ROOT / ".github" / "workflows"
DEFAULT_FV_MANIFEST = REPO_ROOT / "ci" / "formal-verification.toml"

# Triggers that leave a lane invisible to the fleet's normal signals. A lane
# with ONLY these is in scope.
INVISIBLE_TRIGGERS = frozenset({"schedule", "workflow_dispatch"})

# Conclusions that count as a hard, loud failure for rule A.
HARD_FAILURES = frozenset({"failure", "timed_out", "startup_failure"})

# Conclusions that mean a DETECTOR lane's run actually SPOKE: `success` = it
# found nothing, `failure` = it found something and said so. Everything else
# (cancelled / timed_out / startup_failure / action_required / stale / neutral)
# means the run never produced a verdict, so it is not evidence of life even on
# a declared verdict-lane.
VERDICT_CONCLUSIONS = frozenset({"success", "failure"})

# A cron-only lane declares "my non-zero exit is my VERDICT, not my breakage"
# with this line in its own workflow YAML. Anchored at column 0 and matching the
# WHOLE line: an indented occurrence (inside a `run:` block, a heredoc, a
# fixture) does NOT declare, and neither does a prose mention of the token.
VERDICT_LANE_MARKER = re.compile(r"^#[ \t]*cron-liveness:[ \t]*verdict-lane[ \t]*$",
                                 re.MULTILINE)

GRACE_PERIODS = 3.0      # missed ticks tolerated before rule B fires
MIN_GRACE_HOURS = 6.0    # floor: GitHub cron jitter on sub-hourly lanes
CONSEC_FAILURES = 3      # rule A threshold
# Cron expansion window for period derivation. It must be long enough to see at
# least TWO fires of any cron GitHub accepts, because one fire yields no gap and
# therefore no expectation (the lane then goes quiet — fail-safe, but also blind).
# 90 days was too short: `slsa-builder-pin-review.yml` landed on main with
# `37 6 1 1,4,7,10 *` (quarterly), which fires ONCE in 90 days, so the lane would
# have been silently unwatchable. 400 days sees ≥4 fires of a quarterly cron and
# ≥1 of every month, while still leaving a genuinely underivable cron
# (`0 0 29 2 *` from a non-leap-adjacent start, an unparseable expression)
# indeterminate and quiet.
SIM_WINDOW_DAYS = 400
RUNS_CAP = 60            # newest completed scheduled runs to consider per lane

BASE_LABELS = ["cron-liveness", "auto"]
KEY_PREFIX = "cron-liveness-key"

_MONTHS = {m: i + 1 for i, m in enumerate(
    "jan feb mar apr may jun jul aug sep oct nov dec".split())}
_DOWS = {d: i for i, d in enumerate("sun mon tue wed thu fri sat".split())}


class AlarmError(Exception):
    """Alarm-INFRASTRUCTURE failure — surfaced LOUD (exit 2)."""


class CronError(Exception):
    """An unreadable cron expression — fail-SAFE (the lane is skipped, quietly)."""


# --------------------------------------------------------------------------- #
# Cron parsing / period derivation  (pure)
# --------------------------------------------------------------------------- #
def _num(tok: str, names: dict[str, int] | None) -> int:
    tok = tok.strip()
    if names is not None and tok.lower() in names:
        return names[tok.lower()]
    if not tok.isdigit():
        raise CronError(f"not a number: {tok!r}")
    return int(tok)


def parse_cron_field(spec: str, lo: int, hi: int,
                     names: dict[str, int] | None = None) -> set[int]:
    """Expand one cron field to the set of values it matches."""
    spec = spec.strip()
    if not spec:
        raise CronError("empty field")
    out: set[int] = set()
    for part in spec.split(","):
        part = part.strip()
        if not part:
            raise CronError(f"empty list element in {spec!r}")
        step = 1
        if "/" in part:
            part, step_s = part.rsplit("/", 1)
            if not step_s.strip().isdigit():
                raise CronError(f"bad step in {spec!r}")
            step = int(step_s)
            if step < 1:
                raise CronError(f"non-positive step in {spec!r}")
        if part == "*":
            start, end = lo, hi
        elif "-" in part[1:]:
            a_s, b_s = part.split("-", 1)
            start, end = _num(a_s, names), _num(b_s, names)
        else:
            start = end = _num(part, names)
        if start < lo or end > hi or start > end:
            raise CronError(f"out-of-range/inverted range in {spec!r}")
        out.update(range(start, end + 1, step))
    if not out:
        raise CronError(f"field matches nothing: {spec!r}")
    return out


def parse_cron(expr: str) -> dict:
    """Parse a 5-field POSIX cron expression (the form GitHub Actions accepts)."""
    if not isinstance(expr, str):
        raise CronError(f"not a string: {expr!r}")
    fields = expr.split()
    if len(fields) != 5:
        raise CronError(f"expected 5 fields, got {len(fields)}: {expr!r}")
    minute, hour, dom, month, dow = fields
    dow_set = parse_cron_field(dow, 0, 7, _DOWS)
    if 7 in dow_set:  # POSIX: 7 and 0 are both Sunday
        dow_set = (dow_set - {7}) | {0}
    return {
        "minute": parse_cron_field(minute, 0, 59),
        "hour": parse_cron_field(hour, 0, 23),
        "dom": parse_cron_field(dom, 1, 31),
        "month": parse_cron_field(month, 1, 12, _MONTHS),
        "dow": dow_set,
        "dom_star": dom.strip() == "*",
        "dow_star": dow.strip() == "*",
    }


def _day_matches(parsed: dict, day: dt.date) -> bool:
    if day.month not in parsed["month"]:
        return False
    # Vixie semantics: when BOTH day-of-month and day-of-week are restricted the
    # match is their UNION; when only one is restricted, only that one applies.
    dom_ok = day.day in parsed["dom"]
    dow_ok = ((day.weekday() + 1) % 7) in parsed["dow"]  # Mon=0 -> cron Sun=0
    if parsed["dom_star"] and parsed["dow_star"]:
        return True
    if parsed["dom_star"]:
        return dow_ok
    if parsed["dow_star"]:
        return dom_ok
    return dom_ok or dow_ok


def cron_fire_times(exprs: list[str], start: dt.datetime,
                    days: int = SIM_WINDOW_DAYS) -> list[dt.datetime]:
    """Union of fire times of every expression over [start, start+days)."""
    parsed = [parse_cron(e) for e in exprs]
    fires: set[dt.datetime] = set()
    base = start.replace(hour=0, minute=0, second=0, microsecond=0)
    for offset in range(days):
        day = (base + dt.timedelta(days=offset)).date()
        for p in parsed:
            if not _day_matches(p, day):
                continue
            for h in sorted(p["hour"]):
                for m in sorted(p["minute"]):
                    fires.add(dt.datetime(day.year, day.month, day.day, h, m,
                                          tzinfo=dt.timezone.utc))
    return sorted(fires)


@functools.lru_cache(maxsize=256)
def _period_cached(exprs: tuple[str, ...], day: dt.datetime) -> float | None:
    try:
        fires = cron_fire_times(list(exprs), day)
    except CronError:
        return None
    if len(fires) < 2:
        return None
    gaps = [(b - a).total_seconds() / 3600.0 for a, b in zip(fires, fires[1:])]
    return max(gaps)


def derive_period_hours(exprs: list[str], now: dt.datetime) -> float | None:
    """Largest gap (hours) between consecutive fires; None when indeterminate.

    None is the FAIL-SAFE answer: an unparseable cron, an empty schedule, or a
    cron that fires at most once in the SIM_WINDOW_DAYS window yields no
    expectation, and
    a lane with no expectation never raises.
    """
    if not exprs:
        return None
    # The window is day-aligned inside cron_fire_times, so the cache key can be
    # too: every call on the same day with the same crons is the same answer.
    day = now.replace(hour=0, minute=0, second=0, microsecond=0)
    return _period_cached(tuple(exprs), day)


# --------------------------------------------------------------------------- #
# Scope discovery  (pure, over the workflow YAML on disk)
# --------------------------------------------------------------------------- #
def _on_block(doc: dict) -> dict | list | str | None:
    # PyYAML resolves a bare `on:` key to the boolean True (YAML 1.1).
    if "on" in doc:
        return doc["on"]
    return doc.get(True)


def workflow_triggers(doc: dict) -> set[str]:
    on = _on_block(doc)
    if isinstance(on, str):
        return {on}
    if isinstance(on, list):
        return {str(x) for x in on}
    if isinstance(on, dict):
        return {str(k) for k in on}
    return set()


def workflow_crons(doc: dict) -> list[str]:
    on = _on_block(doc)
    if not isinstance(on, dict):
        return []
    sched = on.get("schedule")
    if not isinstance(sched, list):
        return []
    return [e["cron"] for e in sched
            if isinstance(e, dict) and isinstance(e.get("cron"), str)]


def workflow_declares_verdict_lane(text: str) -> bool:
    """True iff the workflow's OWN text carries the column-0 declaration line.

    Deliberately a whole-line, column-0 match rather than containment: an
    indented copy inside a `run:` block cannot declare, and neither can a
    sentence that merely names the token.
    """
    return VERDICT_LANE_MARKER.search(text) is not None


def formal_alarm_watched(manifest: Path) -> set[str]:
    """Workflows already covered by formal-alarm.yml — read, never duplicated."""
    try:
        import tomllib
        with open(manifest, "rb") as fh:
            data = tomllib.load(fh)
    except Exception:  # noqa: BLE001 — a missing/broken manifest just widens scope
        return set()
    return {lane["workflow"] for lane in data.get("lane", [])
            if isinstance(lane, dict) and isinstance(lane.get("workflow"), str)}


def discover_cron_only_lanes(workflows_dir: Path,
                             fv_manifest: Path | None = None) -> list[dict]:
    """Every cron-only lane, newest-first-stable by filename."""
    if not workflows_dir.is_dir():
        raise AlarmError(f"no workflows directory at {workflows_dir}")
    already_watched = formal_alarm_watched(fv_manifest) if fv_manifest else set()
    lanes: list[dict] = []
    for path in sorted(list(workflows_dir.glob("*.yml"))
                       + list(workflows_dir.glob("*.yaml"))):
        try:
            text = path.read_text(encoding="utf-8")
            doc = yaml.safe_load(text)
        except (OSError, yaml.YAMLError) as exc:
            # An unreadable workflow is an INFRASTRUCTURE problem, not a quiet
            # skip: the scan would otherwise silently shrink its own scope.
            raise AlarmError(f"cannot parse {path.name}: {exc}") from exc
        if not isinstance(doc, dict):
            continue
        triggers = workflow_triggers(doc)
        if "schedule" not in triggers or not triggers <= INVISIBLE_TRIGGERS:
            continue
        if path.name in already_watched:
            continue
        lanes.append({
            "workflow": path.name,
            "display_name": doc.get("name") or path.stem,
            "crons": workflow_crons(doc),
            "verdict_lane": workflow_declares_verdict_lane(text),
        })
    return lanes


# --------------------------------------------------------------------------- #
# The check  (pure)
# --------------------------------------------------------------------------- #
def _parse_iso(ts: str) -> dt.datetime:
    return dt.datetime.fromisoformat(ts.replace("Z", "+00:00"))


def classify_lane(lane: dict, state: str | None, runs: list[dict],
                  now: dt.datetime) -> dict:
    """Classify one lane. Always returns a record; `raise_alarm` is the verdict.

    `runs` = completed `event=schedule` runs, NEWEST FIRST, each
    {conclusion, created_at}.
    """
    verdict_lane = bool(lane.get("verdict_lane"))
    rec: dict = {
        "workflow": lane["workflow"],
        "display_name": lane.get("display_name", lane["workflow"]),
        "crons": lane.get("crons", []),
        "verdict_lane": verdict_lane,
        "raise_alarm": False,
        "status": "",
        "reason": "",
        "period_hours": None,
        "threshold_hours": None,
        "last_success": None,
        "last_verdict": None,
        "last_verdict_conclusion": None,
        # POSITIVE evidence of a healthy recent run. Only this closes an open
        # alarm — "quiet" is not "recovered" (a disabled / never-ran /
        # deliberately-inert lane is quiet and still produces no verdict).
        "recovered": False,
        "recovery_evidence": "",
        "recent": [str(r.get("conclusion")) for r in runs[:8]],
        "rules_fired": [],
    }

    if state is not None and state != "active":
        rec["status"] = "DISABLED"
        rec["reason"] = f"workflow state is {state!r} — a deliberate off switch"
        return rec

    period = derive_period_hours(lane.get("crons", []), now)
    rec["period_hours"] = period
    if period is None:
        rec["status"] = "INDETERMINATE-CRON"
        rec["reason"] = ("cron unreadable or fires <2× in the "
                         f"{SIM_WINDOW_DAYS}-day window — no expectation derivable")
        return rec

    threshold = max(period * GRACE_PERIODS, MIN_GRACE_HOURS)
    rec["threshold_hours"] = threshold

    if not runs:
        rec["status"] = "NEVER-RAN"
        rec["reason"] = "no completed scheduled run yet"
        return rec

    if str(runs[0].get("conclusion")) == "skipped":
        rec["status"] = "DELIBERATELY-INERT"
        rec["reason"] = ("newest scheduled run concluded `skipped` — a guard is "
                         "firing on purpose, not a dead lane")
        return rec

    considered = [r for r in runs if str(r.get("conclusion")) != "skipped"]
    if not considered:
        rec["status"] = "DELIBERATELY-INERT"
        rec["reason"] = "every recent scheduled run concluded `skipped`"
        return rec

    successes = [r for r in considered if str(r.get("conclusion")) == "success"]
    last_success = max((_parse_iso(r["created_at"]) for r in successes), default=None)
    rec["last_success"] = last_success.isoformat() if last_success else None
    oldest_seen = min(_parse_iso(r["created_at"]) for r in considered)

    if verdict_lane:
        # Rule V — this lane DECLARED that its non-zero exit is its verdict, so
        # `failure` is evidence of life, not of illness. What must still hold is
        # that the lane SPOKE recently: a run that was cancelled, timed out or
        # never started produced no verdict and counts for nothing here, so the
        # miri cancelled-at-ceiling shape and a lane that stopped firing raise
        # on a declared lane exactly as on any other.
        spoke = [r for r in considered
                 if str(r.get("conclusion")) in VERDICT_CONCLUSIONS]
        newest_run = max(spoke, key=lambda r: _parse_iso(r["created_at"]),
                         default=None)
        newest = _parse_iso(newest_run["created_at"]) if newest_run else None
        rec["last_verdict"] = newest.isoformat() if newest else None
        rec["last_verdict_conclusion"] = (
            str(newest_run.get("conclusion")) if newest_run else None)
        if newest is not None:
            if (now - newest).total_seconds() / 3600.0 > threshold:
                rec["rules_fired"].append(
                    f"V: declared verdict-lane, but no scheduled run has produced "
                    f"a verdict (`success` or `failure`) for >{threshold:.1f}h "
                    f"({GRACE_PERIODS:g}× the {period:.1f}h cron period)")
        elif (now - oldest_seen).total_seconds() / 3600.0 > threshold:
            rec["rules_fired"].append(
                f"V: declared verdict-lane that has never produced a verdict, "
                f"and the lane has been running for >{threshold:.1f}h")
    else:
        # Rule A — N consecutive hard failures, spanning at least the jitter
        # floor. The span condition is the quiet-direction fail-safe for FAST
        # lanes: three consecutive failures of a `*/10` lane is 20 minutes,
        # which is a transient, not a dead lane. On a daily lane the N failures
        # span ~48h, so the floor is never the binding constraint there
        # (bench-ec2 still raises on day three).
        if (len(considered) >= CONSEC_FAILURES
                and all(str(r.get("conclusion")) in HARD_FAILURES
                        for r in considered[:CONSEC_FAILURES])):
            oldest_of_streak = _parse_iso(considered[CONSEC_FAILURES - 1]["created_at"])
            streak_hours = (now - oldest_of_streak).total_seconds() / 3600.0
            if streak_hours >= MIN_GRACE_HOURS:
                rec["rules_fired"].append(
                    f"A: newest {CONSEC_FAILURES} scheduled runs all failed hard "
                    f"(spanning {streak_hours:.1f}h)")

        # Rule B — no green inside the derived window.
        if last_success is not None:
            if (now - last_success).total_seconds() / 3600.0 > threshold:
                rec["rules_fired"].append(
                    f"B: no successful scheduled run for >{threshold:.1f}h "
                    f"({GRACE_PERIODS:g}× the {period:.1f}h cron period)")
        elif (now - oldest_seen).total_seconds() / 3600.0 > threshold:
            # Never succeeded — only actionable once the lane has had longer than
            # the threshold to produce its first green (grace for a NEW lane).
            rec["rules_fired"].append(
                f"B: never succeeded, and the lane has been running for "
                f">{threshold:.1f}h")

    if rec["rules_fired"]:
        rec["raise_alarm"] = True
        rec["status"] = "DEAD"
        rec["reason"] = "; ".join(rec["rules_fired"])
    elif verdict_lane:
        rec["status"] = "LIVE-VERDICT-LANE"
        if rec["last_verdict"]:
            rec["recovered"] = True
            rec["recovery_evidence"] = (
                f"newest scheduled run {rec['last_verdict']} produced a verdict "
                f"(`{rec['last_verdict_conclusion']}`); this lane declares that a "
                f"red run is its FINDING, not its breakage")
            rec["reason"] = (f"newest verdict {rec['last_verdict']} "
                             f"(inside {threshold:.1f}h) — declared verdict-lane, "
                             f"so `failure` counts as a verdict")
        else:
            rec["reason"] = "declared verdict-lane, within the new-lane grace period"
    else:
        rec["status"] = "LIVE"
        if last_success is not None:
            rec["recovered"] = True
            rec["recovery_evidence"] = (
                f"newest successful scheduled run {rec['last_success']}")
            rec["reason"] = (f"last success {rec['last_success']} "
                             f"(inside {threshold:.1f}h)")
        else:
            rec["reason"] = "within the new-lane grace period"
    return rec


# --------------------------------------------------------------------------- #
# Issue rendering
# --------------------------------------------------------------------------- #
def render_issue(rec: dict) -> tuple[str, str]:
    title = (f"cron-liveness: {rec['workflow']} is a dead cron-only lane "
             f"(no green for >{rec['threshold_hours']:.0f}h)")
    body = "\n".join([
        "> 🤖 **SPARQ agent** — automated cron-only-lane liveness alarm "
        "(`scripts/cron_lane_liveness.py`, issue #4328). @jeswr runs multiple "
        "agents on this account; this issue was filed by CI automation.",
        "",
        f"<!-- {KEY_PREFIX}: {rec['workflow']} -->",
        "",
        f"`{rec['workflow']}` (**{rec['display_name']}**) is triggered ONLY by "
        "`schedule:`/`workflow_dispatch:`. A failing run therefore produces **no "
        "PR, no review, no merge-queue entry and no check-run on anyone's head "
        "commit** — it is invisible to every signal the fleet watches. This lane "
        "is currently not producing verdicts.",
        "",
        *([
            "This lane carries the `# cron-liveness: verdict-lane` declaration, so a "
            "`failure` conclusion is treated as its FINDING and never raises here. It "
            "raised anyway: no scheduled run has concluded `success` **or** `failure` "
            "inside the window — the runs were cancelled, timed out, never started, or "
            "stopped happening, so the detector never got to speak.",
            "",
        ] if rec.get("verdict_lane") else []),
        "| | |",
        "|---|---|",
        f"| cron | `{'`, `'.join(rec['crons'])}` |",
        f"| derived period (largest gap between fires) | {rec['period_hours']:.1f}h |",
        f"| staleness threshold ({GRACE_PERIODS:g}× period, "
        f"{MIN_GRACE_HOURS:g}h floor) | {rec['threshold_hours']:.1f}h |",
        f"| newest successful scheduled run | "
        f"{rec['last_success'] or '**NEVER** (none in the fetched window)'} |",
        *([f"| newest run that produced a VERDICT (`success`/`failure`) | "
           f"{rec['last_verdict'] or '**NEVER** (none in the fetched window)'} |"]
          if rec.get("verdict_lane") else []),
        f"| recent conclusions (newest first) | `{rec['recent']}` |",
        "",
        "**Why this raised:**",
        *[f"* {r}" for r in rec["rules_fired"]],
        "",
        "The expectation above is derived from the workflow's OWN `schedule:` "
        "block — there is no per-workflow threshold table to keep in sync.",
        "",
        "Fix the lane, or retire it properly (remove the `schedule:` block, as "
        "`bench-ec2.yml` did in #3785) — a retired lane leaves this alarm's scope "
        "automatically. Do not mute the alarm.",
    ])
    return title, body


def render_summary(records: list[dict]) -> str:
    lines = ["## Cron-only lane liveness", "",
             "| lane | status | period | last success | detail |",
             "|---|---|---|---|---|"]
    for r in sorted(records, key=lambda x: (not x["raise_alarm"], x["workflow"])):
        period = f"{r['period_hours']:.1f}h" if r["period_hours"] else "—"
        lines.append(
            f"| `{r['workflow']}` | {'**DEAD**' if r['raise_alarm'] else r['status']} "
            f"| {period} | {r['last_success'] or '—'} | {r['reason']} |")
    return "\n".join(lines)


# --------------------------------------------------------------------------- #
# GitHub I/O  (paginated, total_count-cross-checked)
# --------------------------------------------------------------------------- #
def _gh(args: list[str]) -> str:
    try:
        out = subprocess.run(args, check=True, capture_output=True, text=True,
                             timeout=180)
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError) as exc:
        detail = (getattr(exc, "stderr", "") or "").strip() or str(exc)
        raise AlarmError(f"{' '.join(args[:4])}…: {detail}") from exc
    return out.stdout


def _gh_json(path: str) -> dict | list:
    raw = _gh(["gh", "api", path])
    try:
        return json.loads(raw)
    except json.JSONDecodeError as exc:
        raise AlarmError(f"gh api {path}: unparseable JSON: {exc}") from exc


def fetch_workflow_state(repo: str, workflow: str) -> str | None:
    """`active` / `disabled_*`; None when GitHub does not know the workflow yet."""
    try:
        meta = _gh_json(f"repos/{repo}/actions/workflows/{workflow}")
    except AlarmError as exc:
        if "404" in str(exc) or "Not Found" in str(exc):
            return "unregistered"  # never fired ⇒ classify_lane goes quiet
        raise
    return str(meta.get("state") or "active")


def fetch_scheduled_runs(repo: str, workflow: str) -> list[dict]:
    """Newest-first completed `event=schedule` runs, PAGINATED and cross-checked
    against the API's own `total_count`."""
    runs: list[dict] = []
    total: int | None = None
    page = 1
    while True:
        resp = _gh_json(
            f"repos/{repo}/actions/workflows/{workflow}/runs"
            f"?event=schedule&status=completed&exclude_pull_requests=true"
            f"&per_page=100&page={page}")
        if not isinstance(resp, dict):
            raise AlarmError(f"{workflow}: runs response is not an object")
        if total is None:
            total = resp.get("total_count")
            if not isinstance(total, int):
                raise AlarmError(f"{workflow}: runs response carries no total_count")
        got = resp.get("workflow_runs")
        if got is None:
            raise AlarmError(f"{workflow}: runs response carries no workflow_runs")
        runs.extend(got)
        want = min(total, RUNS_CAP)
        if len(runs) >= want or not got:
            break
        page += 1
    if total and not runs:
        raise AlarmError(
            f"{workflow}: total_count={total} but pagination returned 0 runs")
    if len(runs) < min(total or 0, RUNS_CAP):
        raise AlarmError(
            f"{workflow}: pagination returned {len(runs)} of an expected "
            f"{min(total or 0, RUNS_CAP)} runs (total_count={total})")
    runs.sort(key=lambda r: str(r.get("created_at")), reverse=True)
    return [{"conclusion": r.get("conclusion"), "created_at": r.get("created_at")}
            for r in runs[:RUNS_CAP]]


def ensure_labels(repo: str) -> None:
    for label in BASE_LABELS:
        _gh(["gh", "label", "create", label, "--repo", repo, "--force"])


def list_open_alarm_issues(repo: str) -> dict[str, int]:
    """{workflow -> issue number} from OPEN cron-liveness issues, fully paginated."""
    found: dict[str, int] = {}
    page = 1
    while True:
        raw = _gh(["gh", "api",
                   f"repos/{repo}/issues?state=open&labels={BASE_LABELS[0]}"
                   f"&per_page=100&page={page}"])
        try:
            batch = json.loads(raw)
        except json.JSONDecodeError as exc:
            raise AlarmError(f"issue list: unparseable JSON: {exc}") from exc
        if not isinstance(batch, list):
            raise AlarmError("issue list: response is not an array")
        for item in batch:
            # The REST issues endpoint also returns PULL REQUESTS. A PR carrying
            # the label (e.g. the one that introduced this alarm, quoting the
            # marker) must never be mistaken for the open alarm issue — editing
            # or closing it would be badly wrong.
            if "pull_request" in item:
                continue
            body = item.get("body") or ""
            for line in body.splitlines():
                if KEY_PREFIX in line:
                    key = line.split(f"{KEY_PREFIX}:", 1)[-1].replace("-->", "").strip()
                    if key:
                        found.setdefault(key, item.get("number"))
        if len(batch) < 100:
            break
        page += 1
        if page > 20:
            raise AlarmError("issue list: >2000 open cron-liveness issues — "
                             "the dedupe key is broken")
    return found


def upsert_issue(repo: str, rec: dict, existing: dict[str, int]) -> str:
    """File a new alarm issue, or refresh the open one IN PLACE (idempotent)."""
    title, body = render_issue(rec)
    number = existing.get(rec["workflow"])
    if number:
        _gh(["gh", "issue", "edit", str(number), "--repo", repo,
             "--title", title, "--body", body])
        return f"updated #{number}"
    url = _gh(["gh", "issue", "create", "--repo", repo, "--title", title,
               "--body", body, *sum((["--label", x] for x in BASE_LABELS), [])]).strip()
    # VERIFY THE WRITE BY READING IT BACK. `gh issue create` can exit 0 having
    # created NOTHING under a secondary content-creation rate limit, which no
    # `rate_limit` field reports. Without the read-back this function would
    # print "filed" and the finding would be silently dropped until the next
    # tick — an exit-zero swallow of exactly the kind this alarm exists to end.
    m = re.search(r"/issues/(\d+)$", url.splitlines()[-1].strip() if url else "")
    if not m:
        raise AlarmError(
            f"{rec['workflow']}: `gh issue create` exited 0 but returned no issue "
            f"URL ({url!r}) — the finding was NOT published")
    created = int(m.group(1))
    back = _gh_json(f"repos/{repo}/issues/{created}")
    if not isinstance(back, dict) or back.get("number") != created:
        raise AlarmError(
            f"{rec['workflow']}: issue #{created} did not read back after "
            f"creation — the finding was NOT published")
    return f"filed {url}"


def close_recovered(repo: str, rec: dict, existing: dict[str, int]) -> str | None:
    """Close an open alarm ONLY on positive evidence of a healthy recent run.

    "Quiet" is not "recovered". A lane that is disabled, has no runs at all, is
    deliberately inert (newest run `skipped`), or has an indeterminate cron
    raises nothing — and produces no verdict either. Closing its alarm with the
    words "is producing successful scheduled runs again" would state something
    false and re-create, inside the fix, the invisibility this alarm exists to
    end. Those alarms stay OPEN and are reported by main() as left open.
    """
    number = existing.get(rec["workflow"])
    if not number or not rec.get("recovered"):
        return None
    _gh(["gh", "issue", "close", str(number), "--repo", repo, "--comment",
         "> 🤖 **SPARQ agent** — auto-closed by `scripts/cron_lane_liveness.py`: "
         f"`{rec['workflow']}` is producing scheduled-run verdicts again "
         f"({rec['recovery_evidence']})."])
    return f"closed #{number}"


# --------------------------------------------------------------------------- #
# main
# --------------------------------------------------------------------------- #
def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description="Cron-only lane liveness alarm.")
    p.add_argument("--workflows-dir", type=Path, default=DEFAULT_WORKFLOWS_DIR)
    p.add_argument("--fv-manifest", type=Path, default=DEFAULT_FV_MANIFEST)
    p.add_argument("--repo", default=os.environ.get("GITHUB_REPOSITORY", ""))
    p.add_argument("--now", help="ISO timestamp override (tests)")
    p.add_argument("--state-file",
                   help="hermetic: JSON {workflow: {state, runs:[{conclusion,created_at}]}}")
    p.add_argument("--dry-run", action="store_true",
                   help="scan + print findings; perform no gh writes")
    p.add_argument("--summary-file", type=Path,
                   help="append the markdown summary here ($GITHUB_STEP_SUMMARY)")
    args = p.parse_args(argv)

    try:
        now = _parse_iso(args.now) if args.now else dt.datetime.now(dt.timezone.utc)
        lanes = discover_cron_only_lanes(args.workflows_dir, args.fv_manifest)
        if not lanes:
            raise AlarmError(
                f"no cron-only lanes discovered under {args.workflows_dir} — the "
                "scanner found nothing to watch, which means the scan is broken, "
                "not that the repo has no cron lanes")

        hermetic = None
        if args.state_file:
            with open(args.state_file, encoding="utf-8") as fh:
                hermetic = json.load(fh)
        elif not args.repo:
            raise AlarmError("--repo (or GITHUB_REPOSITORY) is required for a live run")

        records: list[dict] = []
        for lane in lanes:
            wf = lane["workflow"]
            if hermetic is not None:
                entry = hermetic.get(wf, {})
                state, runs = entry.get("state", "active"), entry.get("runs", [])
            else:
                state = fetch_workflow_state(args.repo, wf)
                runs = [] if state == "unregistered" else \
                    fetch_scheduled_runs(args.repo, wf)
                if state == "unregistered":
                    state = "active"  # no runs ⇒ NEVER-RAN ⇒ quiet
            records.append(classify_lane(lane, state, runs, now))

        summary = render_summary(records)
        print(summary)
        if args.summary_file:
            with open(args.summary_file, "a", encoding="utf-8") as fh:
                fh.write(summary + "\n")

        dead = [r for r in records if r["raise_alarm"]]
        for r in dead:
            print(f"::warning title=dead cron-only lane::{r['workflow']}: {r['reason']}")

        if args.dry_run:
            for r in dead:
                title, body = render_issue(r)
                print(f"--- DRY RUN issue ---\n{title}\n{body}\n")
            return 0

        existing = list_open_alarm_issues(args.repo)
        if dead:
            ensure_labels(args.repo)
        for r in dead:
            print(f"{r['workflow']}: {upsert_issue(args.repo, r, existing)}")
        for r in records:
            if r["raise_alarm"]:
                continue
            done = close_recovered(args.repo, r, existing)
            if done:
                print(f"{r['workflow']}: {done}")
            elif existing.get(r["workflow"]):
                # Quiet but NOT recovered — say so rather than closing on a
                # claim the evidence does not support.
                print(f"{r['workflow']}: open alarm #{existing[r['workflow']]} "
                      f"LEFT OPEN — quiet but not recovered "
                      f"({r['status']}: {r['reason']})")
        # Exit 0: the ISSUE is the signal. A finding that could not be turned
        # into an issue has already raised AlarmError ⇒ exit 2 below.
        return 0
    except AlarmError as exc:
        print(f"::error title=cron-liveness alarm infrastructure error::{exc}")
        return 2


if __name__ == "__main__":
    sys.exit(main())
