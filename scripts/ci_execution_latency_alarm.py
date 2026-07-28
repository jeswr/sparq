#!/usr/bin/env python3
"""CI EXECUTION-LATENCY alarm — three distinct choke modes, one detector. 🤖 SPARQ agent.

Bead sq-1lc4i. Maintainer-requested (2026-07-28): "setting up alerting for when runners
are not picked up for crons or delayed for other dispatches on both repos. This is an
indicator that you are choking on runner availability and is something that we would need
to manage."

WHY THREE MODES. They have different signatures and only ONE of them is runner
availability proper, so a single detector keyed on any one of them is blind to the other
two:

  M1 CRON-FIRING DEFICIT   a scheduled run that never fires produces NO artifact at all.
                           There is no run to inspect, no conclusion, no check-run. The
                           only way to see it is to compute what the lane's own `cron:`
                           EXPECTED and compare against what arrived.
  M2 QUEUE WAIT            a run sitting in `queued` past a measured threshold. This is
                           runner availability proper — the maintainer's stated concern.
  M3 EXECUTION OVERRUN     a run that is `in_progress` far past its lane's own measured
                           duration. It consumes capacity while looking perfectly healthy:
                           no red, no alert, no artifact.

=============================================================================
THE MEASUREMENT TRAP THIS DETECTOR EXISTS BECAUSE OF — read before editing
=============================================================================
The obvious instrument for "are runners being picked up" is job-level
`started_at - created_at` aggregated over recent runs. It does not work, and it fails in
the SAFE-LOOKING direction: it reports health while a leak is live.

MEASURED 2026-07-28 on run 30333511110 (sparq-org/sparq, a `schedule` CI run on main that
had been `in_progress` for 6.7 hours with 8 live jobs), N=65 jobs:

    pickup lag (started_at - created_at):   min 0s   p50 3s   MAX 10s   >60s: 0 jobs
    job-creation lag (created_at - run.created_at):        MAX 383 min  (6.4 hours)

Every job was picked up within ten seconds of EXISTING. The wait was invisible because a
matrix leg throttled behind `max-parallel` (or behind an account concurrency ceiling) is
not created as a job object at all while it waits — so it contributes no queue depth and
no pickup lag. `status=queued` read ZERO for the whole 6.7 hours.

Two consequences, both load-bearing:

  (a) A job that never finishes never appears in a completed run. Any statistic computed
      over COMPLETED work is structurally incapable of detecting a hang or a leak — the
      failing population is precisely the population it excludes. This is survivorship
      bias, and it is why M3's DETECTION VARIABLE is the live age of a `status=in_progress`
      run and NOTHING derived from finished work.

  (b) Completed-run history is still the right population for the THRESHOLD — "how long
      does this lane normally take" is a question about runs that took a length of time,
      i.e. runs that ended. The distinction is: completed runs answer HOW LONG IS NORMAL;
      only live runs answer IS SOMETHING STUCK RIGHT NOW. Never let (b) drift into (a).

  (c) FAIL-OPEN HOLE, closed deliberately: if a lane's runs never complete, it has no
      completed history, hence no derived baseline. A detector that SKIPS a lane with no
      baseline would go silent exactly when a lane is 100% hung. So a missing baseline
      falls back to EXEC_FLOOR_SECONDS instead of skipping. See find_execution_overruns.

=============================================================================
COORDINATION WITH sparq-org/sparq#4368 (`scripts/cron_lane_liveness.py`)
=============================================================================
#4368 watches CRON-ONLY lanes — its scope predicate is: `on:` has `schedule`, AND every
other `on:` key is in {schedule, workflow_dispatch}. It asks "is this lane DEAD?" over a
window of max(3 x period, 6h), keyed on run CONCLUSIONS (consecutive failures / no green).

M1 here is deliberately the EXACT SET COMPLEMENT: a workflow is in M1 scope iff it has a
`schedule:` AND at least one trigger OUTSIDE {schedule, workflow_dispatch}. The predicate
is computed, not listed, so the two populations can never overlap and can never both raise
on one lane — and if #4368's scope is edited, this one moves with it.

That complement is not a leftover. MEASURED on sparq main d2b6034a8: of 31 workflows
carrying a `schedule:`, 18 are cron-only (#4368's) and 13 are NOT — and the 13 include
every throughput-critical orchestration lane in the repo: ci.yml, auto-arm.yml (10-min),
batch-merge.yml (15-min), verdict-bridge.yml (10-min), pr-backlog.yml, refresh-start-here,
plus bench/codeql/container-scan/fuzz/scorecard/xpath-differential/zk-toolchain. #4368
cannot see any of them, and `ci.yml` — the single largest capacity consumer in the repo —
is among them.

M2 and M3 are keyed on live run STATE, which #4368 does not look at in any scope.

=============================================================================
THRESHOLDS — every one derived from measurement, with the N stated
=============================================================================
See the constants below. Nothing here is chosen by taste; each carries the sample it came
from and the multiple it represents. Re-derive with scripts/tests/-adjacent tooling if the
repo's shape changes; a threshold whose provenance is not stated is not maintainable.

NOT A GATE. This lane REDs when it finds a choke, and REDs fail-loud when the detector
itself breaks; neither is a property of the commit under test. It is DECLARED in
`.github/advisory-registry.json`, which since #3773 is the ONLY thing that makes a
check-run non-gating — not where its check-run lands. A scheduled run on main puts its
check-run on the same head SHA the push-triggered `ci-summary` gate polls, so the
declaration is what keeps it safe to RED. Pinned by
scripts/tests/test_alarm_lanes_non_gating.py.

Usage:
  ci_execution_latency_alarm.py                       # real (gh; env GITHUB_REPOSITORY)
  ci_execution_latency_alarm.py --dry-run             # print findings; no gh writes
  ci_execution_latency_alarm.py --state-file s.json \
      --now 2026-07-28T12:00:00Z --dry-run            # hermetic (tests)
  ci_execution_latency_alarm.py --self-test           # hermetic fixtures

Exit codes: 0 = clean, 1 = a choke was found (RED by design), 2 = the detector itself is
broken (fail-loud). Collapsing any pair would make a broken detector indistinguishable
from a healthy repo.

Stdlib + PyYAML (preinstalled on GitHub-hosted runners) + the `gh` CLI.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import subprocess
import sys
from pathlib import Path

try:
    import yaml
except ImportError as _exc:  # pragma: no cover - fail loud, never silently skip M1
    yaml = None
    _YAML_IMPORT_ERROR = _exc


class AlarmError(RuntimeError):
    """The detector itself is broken. Always exit 2 — never mask a choke."""


class CronError(ValueError):
    """An unparseable cron expression. Fail-safe QUIET (the lane is skipped, and counted
    in the census as `cron-unparseable` so the skip is visible rather than silent)."""


KEY_PREFIX = "ci-execution-latency-alarm-key"
BASE_LABELS = ["ci-latency-alarm", "auto"]
# Separator for the flat "<workflow path>|<event>" baseline keys a hermetic --state-file
# carries (JSON object keys cannot be tuples). A pipe cannot occur in either half.
BASELINE_KEY_SEP = "|"

# --- M1: cron firing deficit -------------------------------------------------------
# WINDOW: 24h. Chosen from measurement, not taste. A 6h window was tried first and
# rejected: at 6h a DAILY lane has an expectation of 0 or 1 and a single jittered fire
# swings the ratio between 0.00 and 1.00, so the mode would either alarm constantly or be
# switched off. Over 24h every daily lane on sparq delivered EXACTLY its expectation.
CRON_WINDOW_HOURS = 24.0
# THE CAP, and why nominal cron rate is NOT a usable expectation.
# MEASURED on sparq-org/sparq over the 24h to 2026-07-28T13:15Z, per-lane, event-filtered:
#   DAILY lanes (asan, bench, ci, kani, miri, fuzz, metamorph, differential, ... 16 of
#   them): delivered ratio 1.00 — every single one.
#   SUB-HOURLY lanes: delivered 0.08-0.25 of nominal. auto-arm (`*/10`, nominal 144) got
#   12. promote-on-approval (`*/10`, 144) got 12. rearm-sweeper (6/h, 144) got 13.
#   verdict-bridge (6/h, 144) got 12. batch-merge (4/h, 96) got 12. retriage (`*/30`, 48)
#   got 12.
# GitHub's `schedule` trigger is best-effort and documented as droppable under load; on a
# repo this busy it converges to ~12 scheduled runs per lane per day REGARDLESS of the
# cron. Comparing against nominal would put six lanes permanently in breach — an alarm
# that is always red is muted, and a muted alarm is how a real outage gets ignored.
# So the expectation is CAPPED at what GitHub demonstrably delivers. 0.5/h = 12 per 24h;
# the highest any sparq lane actually achieved in 24h was 13.
CRON_MAX_CREDIBLE_FIRINGS_PER_HOUR = 0.5
# Ignore firings younger than this. Without it the window edge races the scheduler: at
# 03:17:30 a `17 3 * * *` lane has an expectation of 1 whose run may not be created yet,
# giving ratio 0.00 and a phantom deficit on every single tick that lands near the cron.
CRON_GRACE_MINUTES = 15
# A lane needs at least this much capped expectation for a ratio to mean anything.
CRON_MIN_EXPECTED = 1
# Raise below this delivery ratio. VALIDATED, not chosen: over the 24 covered sparq lanes
# the MINIMUM capped delivery ratio was 0.75 (refresh-start-here.yml, 3 of 4), so 0.60
# sits 0.15 BELOW the worst healthy lane and fires on ZERO of them. It still catches the
# mode that has no artifact — a lane that stops entirely goes to 0.00 — and catches a
# halving of a sub-hourly lane's achieved rate (12 -> 6 of a capped 12 = 0.50).
CRON_DELIVERY_FLOOR = 0.60

# --- M2: queue wait (runner availability proper) -----------------------------------
# MEASURED run-level queue wait (`run_started_at - created_at`) over COMPLETED runs:
#   sparq-org/sparq               N=13,802   p50 0s  p90 0s  p99 0s   max 94.2m
#   jeswr/agent-account-registry  N= 3,920   p50 0s  p90 0s  p99 0s   max 99.5m
# The metric is ZERO-INFLATED: 22 of 13,802 sparq runs (0.16%) and 8 of 3,920 registry
# runs (0.20%) ever recorded a non-zero wait. That is why this threshold is NOT stated as
# a multiple of the observed maximum, the way M3's is — multiplying a max of 94.2m gives a
# threshold nothing could ever reach, and multiplying a p99 of 0s gives 0. For a
# zero-inflated distribution the honest statement is the false-positive rate.
# 15 min: fires on 3 of 13,802 sparq completed runs (0.022%) and 6 of 3,920 registry runs
# (0.153%). The registry six are not false positives — they are a REAL contiguous queue
# event, seven `pr-gate.yml` runs held 12.8-99.5 min between 00:58Z and 02:24Z on
# 2026-07-28, which is precisely the condition this mode exists to report.
QUEUE_MAX_WAIT_SECONDS = 15 * 60

# --- M3: execution overrun ---------------------------------------------------------
# Threshold = max(EXEC_OVERRUN_MULTIPLE x p90(completed durations for this workflow+event),
#                 EXEC_FLOOR_SECONDS).
# VALIDATED by a leave-one-out sweep over the event-filtered completed-run corpus — each
# run scored against a p90 computed from the OTHER runs in its own (workflow, event) cell:
#   sparq-org/sparq                N=8,676 runs / 119 cells
#   jeswr/agent-account-registry   N=2,063 runs /  30 cells
# Result: at K = 1.25 .. 2.0 the ONLY sparq run that fires is the genuine 2026-07-27
# nightly ci.yml outlier (17.33h against an 8.35h p90 = 2.08x), and ZERO registry runs
# fire. At K = 2.5 and above that real event is MISSED. So 2.0 is the LARGEST multiple
# that still detects the known positive, and it costs nothing: every other one of the
# 10,739 sampled runs stays silent.
# Instrument validated against a known answer before the result was trusted: injecting one
# synthetic run at 2.5x a cell's p90 fires at K=2.0 on both repos, so a zero result is a
# real zero and not a broken sweep.
EXEC_OVERRUN_MULTIPLE = 2.0
# Floor. Applies when a lane has NO usable completed baseline — including the case that
# matters most: a lane whose runs never complete (see (c) in the header). 6h is GitHub's
# hosted-runner per-job ceiling, so a RUN alive past 6h has necessarily outlived any single
# job's maximum possible life and is either a long matrix or a leak; either way it is worth
# a line. Never `continue` on a missing baseline — that is the fail-open hole.
EXEC_FLOOR_SECONDS = 6 * 60 * 60
# Completed runs sampled per (workflow, event) to build the baseline.
BASELINE_SAMPLE = 100
# A baseline needs at least this many completed samples to be trusted; below it, fall back
# to the floor (never skip).
BASELINE_MIN_N = 5

# `actions/runs` hard-caps pagination at 1000 results regardless of `--paginate`
# (MEASURED: total_count=21672, --paginate returned exactly 1000). Live-state queries are
# filtered by `status=` and are far below that, but the cap is asserted rather than assumed.
RUNS_PAGE_CAP = 10

# The trigger set that makes a lane INVISIBLE to PR review / merge queues. Identical to
# `cron_lane_liveness.INVISIBLE_TRIGGERS` (#4368) on purpose — see the coordination note.
INVISIBLE_TRIGGERS = frozenset({"schedule", "workflow_dispatch"})


# ---------------------------------------------------------------------------------
# cron expansion
# ---------------------------------------------------------------------------------
def _expand_field(spec: str, lo: int, hi: int) -> set[int]:
    """Expand one cron field to the set of values it matches."""
    if not spec:
        raise CronError("empty field")
    out: set[int] = set()
    for part in spec.split(","):
        if not part:
            raise CronError(f"empty list element in {spec!r}")
        step = 1
        if "/" in part:
            part, raw = part.split("/", 1)
            if not raw.isdigit():
                raise CronError(f"bad step in {spec!r}")
            step = int(raw)
            if step <= 0:
                raise CronError(f"non-positive step in {spec!r}")
        if part in ("*", "?"):
            a, b = lo, hi
        elif "-" in part:
            lhs, rhs = part.split("-", 1)
            if not (lhs.isdigit() and rhs.isdigit()):
                raise CronError(f"bad range in {spec!r}")
            a, b = int(lhs), int(rhs)
            if a > b:
                raise CronError(f"inverted range in {spec!r}")
        else:
            if not part.isdigit():
                raise CronError(f"not a number: {part!r}")
            a = b = int(part)
        out |= set(range(a, b + 1, step))
    out = {v for v in out if lo <= v <= hi}
    if not out:
        raise CronError(f"field matches nothing: {spec!r}")
    return out


def expected_firings(expr: str, start: dt.datetime, end: dt.datetime) -> int:
    """How many times `expr` fires in (start, end]. Minute-resolution walk.

    POSIX day semantics: when BOTH day-of-month and day-of-week are restricted the match
    is their UNION, not their intersection.
    """
    if not isinstance(expr, str):
        raise CronError(f"not a string: {expr!r}")
    fields = expr.split()
    if len(fields) != 5:
        raise CronError(f"expected 5 fields, got {len(fields)}: {expr!r}")
    minute, hour, dom, month, dow = fields
    mins = _expand_field(minute, 0, 59)
    hours = _expand_field(hour, 0, 23)
    doms = _expand_field(dom, 1, 31)
    months = _expand_field(month, 1, 12)
    dows = {d % 7 for d in _expand_field(dow, 0, 7)}
    dom_wild = dom.strip() in ("*", "?")
    dow_wild = dow.strip() in ("*", "?")
    if (end - start) > dt.timedelta(days=400):
        raise CronError("window too wide to expand")

    t = start.replace(second=0, microsecond=0) + dt.timedelta(minutes=1)
    n = 0
    while t <= end:
        if t.minute in mins and t.hour in hours and t.month in months:
            weekday = (t.weekday() + 1) % 7  # cron: Sunday == 0
            if dom_wild and dow_wild:
                day_ok = True
            elif dom_wild:
                day_ok = weekday in dows
            elif dow_wild:
                day_ok = t.day in doms
            else:
                day_ok = (t.day in doms) or (weekday in dows)
            if day_ok:
                n += 1
        t += dt.timedelta(minutes=1)
    return n


# ---------------------------------------------------------------------------------
# workflow scope
# ---------------------------------------------------------------------------------
def workflow_triggers(text: str) -> dict:
    """-> the parsed `on:` mapping. `on` is YAML 1.1 `true`, hence the two-key lookup."""
    if yaml is None:  # pragma: no cover
        raise AlarmError(f"PyYAML unavailable: {_YAML_IMPORT_ERROR}")
    try:
        doc = yaml.safe_load(text)
    except yaml.YAMLError as exc:
        raise AlarmError(f"unparseable workflow YAML: {exc}") from exc
    if not isinstance(doc, dict):
        raise AlarmError("workflow YAML is not a mapping")
    on = doc.get(True, doc.get("on"))
    return on if isinstance(on, dict) else {}


def m1_scope(on: dict) -> tuple[bool, list[str], bool]:
    """-> (in_m1_scope, cron expressions, is_cron_only).

    IN SCOPE iff the workflow has a `schedule:` AND at least one trigger OUTSIDE
    {schedule, workflow_dispatch}. That is the exact set complement of #4368's
    `cron_lane_liveness.py` scope, so the two detectors partition the scheduled
    workflows and can never both raise on the same lane.
    """
    if "schedule" not in on:
        return False, [], False
    sched = on.get("schedule") or []
    crons = [s.get("cron") for s in sched
             if isinstance(s, dict) and isinstance(s.get("cron"), str)]
    cron_only = set(on) <= INVISIBLE_TRIGGERS
    return (bool(crons) and not cron_only), crons, cron_only


# ---------------------------------------------------------------------------------
# detectors — pure functions over already-fetched state
# ---------------------------------------------------------------------------------
def find_cron_deficits(lanes: list[dict], now: dt.datetime,
                       window_hours: float = CRON_WINDOW_HOURS) -> tuple[list[dict], dict]:
    """M1. `lanes` = [{workflow, crons, cron_only, in_scope, state, schedule_run_times}].

    The census counts EVERY state exit, not just the alarming one: a per-state population
    is the only shape in which a MISSING edge is visible at all.
    """
    start = now - dt.timedelta(hours=window_hours)
    # The window ENDS a grace period before `now`, so a firing whose run may not have been
    # created yet is not counted as missing. Without this the tick that lands on a lane's
    # own cron minute manufactures a deficit for that lane, every day.
    end = now - dt.timedelta(minutes=CRON_GRACE_MINUTES)
    # What GitHub will actually deliver in a window of this length. Nominal cron rate is
    # not an expectation for sub-hourly lanes — see the constant's note.
    ceiling = int(CRON_MAX_CREDIBLE_FIRINGS_PER_HOUR * window_hours)
    findings: list[dict] = []
    census: dict[str, int] = {}

    def bump(state: str) -> None:
        census[state] = census.get(state, 0) + 1

    for lane in lanes:
        if not lane.get("in_scope"):
            bump("cron-only (watched by #4368)" if lane.get("cron_only")
                 else "not-scheduled")
            continue
        if lane.get("state") != "active":
            bump("disabled")
            continue
        try:
            nominal = sum(expected_firings(c, start, end) for c in lane["crons"])
        except CronError:
            bump("cron-unparseable")
            continue
        expected = min(nominal, ceiling)
        if expected < CRON_MIN_EXPECTED:
            bump("expectation-below-floor")
            continue
        actual = sum(1 for t in lane["schedule_run_times"] if start < t <= end)
        # No clamp on `actual`: an over-delivering lane gives a ratio above 1.0, which is
        # >= the floor and therefore healthy either way. A `min(actual, expected)` here
        # was removed after its own mutant SURVIVED — it changed no observable output, so
        # it was dead code, and an untestable guard is worse than no guard.
        ratio = actual / expected
        if ratio < CRON_DELIVERY_FLOOR:
            bump("firing-deficit")
            findings.append({
                "mode": "M1-cron-firing-deficit",
                "workflow": lane["workflow"],
                "expected": expected,
                "nominal": nominal,
                "actual": actual,
                "ratio": round(ratio, 3),
                "window_hours": window_hours,
                "crons": lane["crons"],
            })
        else:
            bump("delivering")
    return findings, census


def find_queue_overruns(live_runs: list[dict], now: dt.datetime,
                        max_wait: float = QUEUE_MAX_WAIT_SECONDS
                        ) -> tuple[list[dict], dict]:
    """M2. Runner availability proper: a run still in `queued` past `max_wait`."""
    findings: list[dict] = []
    census: dict[str, int] = {}

    def bump(state: str) -> None:
        census[state] = census.get(state, 0) + 1

    for run in live_runs:
        if run.get("status") != "queued":
            continue
        created = run.get("created_at")
        if not created:
            bump("queued-no-timestamp")
            continue
        waited = (now - _ts(created)).total_seconds()
        if waited > max_wait:
            bump("queued-over-threshold")
            findings.append({
                "mode": "M2-queue-wait",
                "workflow": run.get("name") or run.get("path") or "?",
                "run_id": run.get("id"),
                "event": run.get("event"),
                "waited_seconds": int(waited),
                "threshold_seconds": int(max_wait),
                "head_branch": run.get("head_branch"),
            })
        else:
            bump("queued-within-threshold")
    return findings, census


def find_execution_overruns(live_runs: list[dict], baselines: dict, now: dt.datetime,
                            multiple: float = EXEC_OVERRUN_MULTIPLE,
                            floor: float = EXEC_FLOOR_SECONDS
                            ) -> tuple[list[dict], dict]:
    """M3. A run `in_progress` past its own lane's measured duration.

    DETECTION VARIABLE is the live age of an in-flight run — deliberately NOT any
    statistic over completed runs, which structurally excludes the hangs this exists to
    catch (header note (a)).

    `baselines` maps (workflow_key, event) -> {"p90": seconds, "n": int}. A missing or
    under-sampled baseline falls back to `floor` and is REPORTED as such; it must never
    cause a `continue`, or a lane whose runs never complete would be unwatchable by its
    own detector (header note (c)).
    """
    findings: list[dict] = []
    census: dict[str, int] = {}

    def bump(state: str) -> None:
        census[state] = census.get(state, 0) + 1

    for run in live_runs:
        if run.get("status") != "in_progress":
            continue
        started = run.get("run_started_at") or run.get("created_at")
        if not started:
            bump("in-progress-no-timestamp")
            continue
        age = (now - _ts(started)).total_seconds()
        key = (run.get("path") or run.get("name") or "?", run.get("event"))
        base = baselines.get(key) or {}
        n = int(base.get("n") or 0)
        p90 = base.get("p90")
        if p90 is None or n < BASELINE_MIN_N:
            threshold = floor
            basis = f"floor (no usable baseline; n={n})"
        else:
            threshold = max(multiple * float(p90), floor)
            basis = f"max({multiple}x p90={int(p90)}s over n={n}, floor)"
        if age > threshold:
            bump("execution-overrun")
            findings.append({
                "mode": "M3-execution-overrun",
                "workflow": run.get("name") or run.get("path") or "?",
                "run_id": run.get("id"),
                "event": run.get("event"),
                "age_seconds": int(age),
                "threshold_seconds": int(threshold),
                "basis": basis,
                "head_branch": run.get("head_branch"),
            })
        else:
            bump("in-progress-within-threshold")
    return findings, census


def _ts(value: str) -> dt.datetime:
    try:
        return dt.datetime.fromisoformat(str(value).replace("Z", "+00:00"))
    except ValueError as exc:
        raise AlarmError(f"unparseable timestamp {value!r}: {exc}") from exc


def percentile(values: list[float], q: float) -> float:
    ordered = sorted(values)
    idx = min(len(ordered) - 1, int(round(q * (len(ordered) - 1))))
    return ordered[idx]


# ---------------------------------------------------------------------------------
# gh I/O
# ---------------------------------------------------------------------------------
def _gh(args: list[str], *, parse: bool = True):
    try:
        out = subprocess.run(
            ["gh", *args], check=True, capture_output=True, text=True, timeout=180
        )
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError) as exc:
        detail = getattr(exc, "stderr", "") or str(exc)
        raise AlarmError(f"gh {' '.join(args[:2])} failed: {detail}") from exc
    if not parse:
        return out.stdout
    try:
        return json.loads(out.stdout or "null")
    except json.JSONDecodeError as exc:
        raise AlarmError(f"gh {' '.join(args[:2])}: unparseable JSON: {exc}") from exc


def _paged_runs(repo: str, query: str) -> list[dict]:
    """Walk `actions/runs` pages. Terminates on a SHORT page, never on
    `len(runs) >= total_count`: a total_count read before a new run started is an
    UNDERCOUNT and stopping on it would truncate exactly when the list is growing."""
    runs: list[dict] = []
    page = 1
    while page <= RUNS_PAGE_CAP:
        payload = _gh(["api", f"repos/{repo}/actions/runs?{query}&per_page=100&page={page}"])
        if not isinstance(payload, dict):
            raise AlarmError("workflow-run listing is not an object")
        batch = payload.get("workflow_runs")
        if batch is None:
            raise AlarmError("workflow-run listing carries no workflow_runs")
        runs.extend(batch)
        if len(batch) < 100:
            return runs
        page += 1
    raise AlarmError(f"workflow-run listing exceeded {RUNS_PAGE_CAP} pages for {query!r}")


def fetch_live_runs(repo: str) -> list[dict]:
    """Every run currently `queued` or `in_progress`. This is the live-state read that
    M2 and M3 key on; it is NOT derived from completed work (header note (a))."""
    live: list[dict] = []
    for status in ("queued", "in_progress"):
        live.extend(_paged_runs(repo, f"status={status}"))
    return live


def fetch_baseline(repo: str, workflow_path: str, event: str) -> dict:
    """p90 of completed-run DURATION for one (workflow, event).

    Completed runs are the correct population for "how long is normal" and the WRONG
    population for "is something stuck" — see header notes (a) and (b).
    """
    wf = workflow_path.split("/")[-1]
    payload = _gh(["api", f"repos/{repo}/actions/workflows/{wf}/runs"
                          f"?status=completed&event={event}&per_page={BASELINE_SAMPLE}"])
    if not isinstance(payload, dict) or payload.get("workflow_runs") is None:
        raise AlarmError(f"{wf}: baseline response carries no workflow_runs")
    durations = []
    for run in payload["workflow_runs"]:
        started, updated = run.get("run_started_at"), run.get("updated_at")
        if not (started and updated):
            continue
        delta = (_ts(updated) - _ts(started)).total_seconds()
        if delta >= 0:
            durations.append(delta)
    if not durations:
        return {"p90": None, "n": 0}
    return {"p90": percentile(durations, 0.90), "n": len(durations)}


def fetch_lanes(repo: str, workflows_dir: Path, window_hours: float,
                now: dt.datetime) -> list[dict]:
    """Build the M1 lane list from the workflow files plus each lane's schedule runs."""
    if not workflows_dir.is_dir():
        raise AlarmError(f"no workflows directory at {workflows_dir}")
    listing = _gh(["api", f"repos/{repo}/actions/workflows?per_page=100"])
    if not isinstance(listing, dict) or listing.get("workflows") is None:
        raise AlarmError("workflow listing carries no workflows")
    state_by_path = {w["path"]: w.get("state") for w in listing["workflows"]}

    lanes: list[dict] = []
    start = now - dt.timedelta(hours=window_hours)
    for path in sorted(workflows_dir.glob("*.yml")) + sorted(workflows_dir.glob("*.yaml")):
        rel = f".github/workflows/{path.name}"
        on = workflow_triggers(path.read_text(encoding="utf-8"))
        in_scope, crons, cron_only = m1_scope(on)
        lane = {"workflow": rel, "crons": crons, "cron_only": cron_only,
                "in_scope": in_scope, "state": state_by_path.get(rel, "active"),
                "schedule_run_times": []}
        if in_scope and lane["state"] == "active":
            payload = _gh(["api", f"repos/{repo}/actions/workflows/{path.name}/runs"
                                  f"?event=schedule&per_page=100"])
            if not isinstance(payload, dict) or payload.get("workflow_runs") is None:
                raise AlarmError(f"{path.name}: schedule-run response carries no workflow_runs")
            times = [_ts(r["created_at"]) for r in payload["workflow_runs"]
                     if r.get("created_at")]
            # COVERAGE GUARD: the sample is the newest 100 runs. If the OLDEST sampled run
            # is newer than the window start, the count is TRUNCATED and would manufacture
            # a phantom deficit. Treat as indeterminate (fail-safe quiet), never as 0.
            if times and min(times) > start and len(times) >= 100:
                lane["in_scope"] = False
                lane["truncated"] = True
            lane["schedule_run_times"] = times
        lanes.append(lane)
    return lanes


# ---------------------------------------------------------------------------------
# reporting
# ---------------------------------------------------------------------------------
def render_issue_body(repo: str, findings: list[dict], census: dict[str, dict],
                      now: dt.datetime) -> str:
    lines = [
        "> 🤖 **SPARQ agent** — automated CI execution-latency alarm "
        "(`scripts/ci_execution_latency_alarm.py`, bead sq-1lc4i). @jeswr runs multiple "
        "agents on this account; this issue was filed by CI automation.",
        "",
        f"CI execution latency in `{repo}` breached a measured threshold at "
        f"`{now:%Y-%m-%dT%H:%M:%SZ}`.",
        "",
    ]
    by_mode: dict[str, list[dict]] = {}
    for f in findings:
        by_mode.setdefault(f["mode"], []).append(f)
    for mode in sorted(by_mode):
        lines += [f"### {mode}", ""]
        for f in by_mode[mode]:
            if mode.startswith("M1"):
                lines.append(
                    f"- `{f['workflow']}` fired **{f['actual']}** of an expected "
                    f"**{f['expected']}** in the last {f['window_hours']:g}h "
                    f"(ratio {f['ratio']}, floor {CRON_DELIVERY_FLOOR}); "
                    f"cron `{' | '.join(f['crons'])}`")
            elif mode.startswith("M2"):
                lines.append(
                    f"- `{f['workflow']}` run {f['run_id']} ({f['event']}) has been "
                    f"**queued {f['waited_seconds']//60} min**, threshold "
                    f"{f['threshold_seconds']//60} min")
            else:
                lines.append(
                    f"- `{f['workflow']}` run {f['run_id']} ({f['event']}) has been "
                    f"**in progress {f['age_seconds']//60} min**, threshold "
                    f"{f['threshold_seconds']//60} min — basis: {f['basis']}")
        lines.append("")
    lines += ["**Census of every state exit** (emitted on every run, including the "
              "all-clear — a silent alarm is indistinguishable from a healthy system):", ""]
    for mode in sorted(census):
        lines += [f"`{mode}`", "", "```"]
        lines += [f"{k}: {v}" for k, v in sorted(census[mode].items())]
        lines += ["```", ""]
    lines.append(f"<!-- {KEY_PREFIX}: {repo} -->")
    return "\n".join(lines)


def file_issue(repo: str, body: str, count: int) -> None:
    """One open issue, keyed by the body marker. Exact-match the marker rather than
    using `--search`: a key containing punctuation tokenises unreliably, which could MISS
    an existing issue and mint a duplicate."""
    marker = f"<!-- {KEY_PREFIX}: {repo} -->"
    existing = _gh(["api", "--paginate", "--slurp",
                    f"repos/{repo}/issues?state=open&labels={BASE_LABELS[0]}&per_page=100"])
    if not isinstance(existing, list):
        raise AlarmError("issue listing is not a list")
    for page in existing:
        for issue in page if isinstance(page, list) else []:
            if isinstance(issue, dict) and marker in str(issue.get("body") or ""):
                _gh(["api", "-X", "PATCH", f"repos/{repo}/issues/{issue['number']}",
                     "-f", f"body={body}"])
                print(f"::notice::updated existing ci-latency-alarm issue "
                      f"#{issue['number']}")
                return
    title = f"ci-latency-alarm: {count} CI execution-latency breach(es) in {repo}"
    _gh(["api", "-X", "POST", f"repos/{repo}/issues",
         "-f", f"title={title}", "-f", f"body={body}",
         *[arg for label in BASE_LABELS for arg in ("-f", f"labels[]={label}")]])
    print("::notice::filed a new ci-latency-alarm issue")


# ---------------------------------------------------------------------------------
# orchestration
# ---------------------------------------------------------------------------------
def run(args: argparse.Namespace) -> int:
    now = _ts(args.now) if args.now else dt.datetime.now(dt.timezone.utc)

    if args.state_file:
        state = json.loads(Path(args.state_file).read_text(encoding="utf-8"))
        repo = state.get("repo") or args.repo or "hermetic/fixture"
        lanes = state.get("lanes", [])
        for lane in lanes:
            lane["schedule_run_times"] = [_ts(t) for t in lane.get("schedule_run_times", [])]
        live = state.get("live_runs", [])
        baselines = {tuple(k.split(BASELINE_KEY_SEP, 1)): v
                     for k, v in (state.get("baselines") or {}).items()}
    else:
        repo = args.repo or os.environ.get("GITHUB_REPOSITORY") or ""
        if "/" not in repo:
            raise AlarmError(f"no usable repo slug: {repo!r}")
        root = Path(__file__).resolve().parents[1]
        lanes = fetch_lanes(repo, root / ".github" / "workflows",
                            args.window_hours, now)
        live = fetch_live_runs(repo)
        baselines = {}
        for r in live:
            if r.get("status") != "in_progress":
                continue
            key = (r.get("path") or r.get("name") or "?", r.get("event"))
            if key not in baselines and r.get("path"):
                baselines[key] = fetch_baseline(repo, r["path"], r.get("event") or "push")

    # EMPTY SCAN SET IS FAIL-LOUD. A detector watching nothing is not a healthy repo; it
    # is a broken detector, and the two must never look alike. This is the 100% question
    # applied to M1: if every workflow were cron-only, M1's population would be empty and
    # a `return 0` here would report health while watching zero lanes.
    if not lanes:
        raise AlarmError("empty scan set: no workflows discovered")
    if not any(lane.get("in_scope") for lane in lanes):
        raise AlarmError(
            "empty M1 scan set: no workflow has a `schedule:` plus a trigger outside "
            f"{sorted(INVISIBLE_TRIGGERS)} — either the repo changed shape or the scope "
            "predicate broke. Refusing to report health over an empty population.")

    m1, c1 = find_cron_deficits(lanes, now, args.window_hours)
    m2, c2 = find_queue_overruns(live, now)
    m3, c3 = find_execution_overruns(live, baselines, now)
    findings = m1 + m2 + m3
    census = {"M1-cron-firing-deficit": c1, "M2-queue-wait": c2,
              "M3-execution-overrun": c3}

    print(f"ci-latency census for {repo} at {now:%Y-%m-%dT%H:%M:%SZ} "
          f"({len(lanes)} workflows, {len(live)} live runs):")
    for mode in sorted(census):
        print(f"  {mode}:")
        for state, count in sorted(census[mode].items()):
            print(f"    {state}: {count}")
    for f in findings:
        print(f"  BREACH {f['mode']} {f['workflow']}")

    if not findings:
        print("::notice::CI execution latency is within every measured threshold")
        return 0

    body = render_issue_body(repo, findings, census, now)
    if args.dry_run:
        print(body)
    else:
        file_issue(repo, body, len(findings))
    print(f"::error::{len(findings)} CI execution-latency breach(es) in {repo} — "
          "runner pickup, cron delivery, or execution time is outside its measured band")
    return 1


# ---------------------------------------------------------------------------------
# hermetic self-test — runs as the FIRST step of every alarm run, so a broken detector
# reds before it is trusted to watch anything.
# ---------------------------------------------------------------------------------
def capped_expectation(window_hours: float = CRON_WINDOW_HOURS) -> int:
    """The most a lane can be expected to deliver in a window — exposed so tests assert
    against the DERIVED value rather than re-hard-coding it (a test that pins 12 would go
    green-but-wrong if the cap changed)."""
    return int(CRON_MAX_CREDIBLE_FIRINGS_PER_HOUR * window_hours)


def _lane(workflow="a.yml", crons=("*/10 * * * *",), cron_only=False,
          in_scope=True, state="active", fires=0, now=None, spacing_min=30):
    """`fires` runs, all placed strictly INSIDE the counted window (older than the grace
    cut-off, newer than the window start), so a fixture's count is exactly its `fires`."""
    now = now or dt.datetime(2026, 7, 28, 12, 0, tzinfo=dt.timezone.utc)
    first = now - dt.timedelta(minutes=CRON_GRACE_MINUTES + 1)
    times = [first - dt.timedelta(minutes=spacing_min * i) for i in range(fires)]
    return {"workflow": workflow, "crons": list(crons), "cron_only": cron_only,
            "in_scope": in_scope, "state": state, "schedule_run_times": times}


def _run_obj(status="in_progress", event="schedule", path=".github/workflows/a.yml",
             age_min=10, now=None, name="A"):
    now = now or dt.datetime(2026, 7, 28, 12, 0, tzinfo=dt.timezone.utc)
    stamp = (now - dt.timedelta(minutes=age_min)).strftime("%Y-%m-%dT%H:%M:%SZ")
    return {"id": 1, "name": name, "path": path, "event": event, "status": status,
            "created_at": stamp, "run_started_at": stamp, "head_branch": "main"}


def _self_test() -> int:  # noqa: C901 - a flat table of named assertions reads best flat
    failures: list[str] = []

    def check(name: str, condition: bool) -> None:
        if not condition:
            failures.append(name)

    NOW = dt.datetime(2026, 7, 28, 12, 0, tzinfo=dt.timezone.utc)
    H6 = NOW - dt.timedelta(hours=6)

    # --- cron expansion, canary-validated against hand-computed answers ---
    check("cron */10 over 6h == 36", expected_firings("*/10 * * * *", H6, NOW) == 36)
    check("cron */10 over 24h == 144",
          expected_firings("*/10 * * * *", NOW - dt.timedelta(hours=24), NOW) == 144)
    check("cron nightly over 24h == 1",
          expected_firings("17 3 * * *", NOW - dt.timedelta(hours=24), NOW) == 1)
    check("cron explicit-minute list over 6h == 36",
          expected_firings("4,14,24,34,44,54 * * * *", H6, NOW) == 36)
    check("cron 4-hourly over 24h == 6",
          expected_firings("0 */4 * * *", NOW - dt.timedelta(hours=24), NOW) == 6)
    check("cron weekly Monday absent from a Tuesday 24h window",
          expected_firings("0 6 * * 1", NOW - dt.timedelta(hours=24), NOW) == 0)
    check("cron weekly Monday present in a 48h window",
          expected_firings("0 6 * * 1", NOW - dt.timedelta(hours=48), NOW) == 1)
    # POSIX day-field UNION semantics (dom AND dow both restricted -> OR, not AND).
    check("cron dom+dow both restricted is a UNION",
          expected_firings("0 0 1 * 3", dt.datetime(2026, 7, 1, tzinfo=dt.timezone.utc),
                           dt.datetime(2026, 7, 31, tzinfo=dt.timezone.utc)) > 1)
    for bad in ("", "* * * *", "* * * * * *", "60 * * * *", "*/0 * * * *", "5-1 * * * *"):
        try:
            expected_firings(bad, H6, NOW)
            check(f"cron {bad!r} must raise CronError", False)
        except CronError:
            pass
        except Exception:
            check(f"cron {bad!r} raised the wrong error", False)

    # --- M1 scope is the exact complement of #4368 ---
    sched_only = {"schedule": [{"cron": "*/10 * * * *"}], "workflow_dispatch": None}
    mixed = dict(sched_only, pull_request=None)
    check("M1 excludes a cron-ONLY lane (#4368 owns it)", m1_scope(sched_only)[0] is False)
    check("M1 marks a cron-only lane as such", m1_scope(sched_only)[2] is True)
    check("M1 includes a schedule+other-trigger lane", m1_scope(mixed)[0] is True)
    check("M1 excludes an unscheduled workflow", m1_scope({"push": None})[0] is False)

    # --- M1 detection ---
    CAP = capped_expectation()
    f, c = find_cron_deficits([_lane(fires=CAP, now=NOW)], NOW)
    check("M1 quiet when the lane delivers its capped expectation",
          not f and c.get("delivering") == 1)
    f, c = find_cron_deficits([_lane(fires=1, now=NOW)], NOW)
    check("M1 raises on a firing deficit", len(f) == 1 and c.get("firing-deficit") == 1)
    check("M1 finding carries the CAPPED expectation and the nominal rate",
          f and f[0]["expected"] == CAP and f[0]["actual"] == 1
          and f[0]["nominal"] > CAP)
    f, _ = find_cron_deficits([_lane(fires=0, now=NOW)], NOW)
    check("M1 raises when a cron fired ZERO times (the no-artifact mode)", len(f) == 1)
    # The cap is what stops a `*/10` lane alarming forever: nominal 144/day, GitHub
    # delivers ~12, and 12 of a capped 12 is healthy.
    f, _ = find_cron_deficits([_lane(crons=("*/10 * * * *",), fires=CAP, now=NOW)], NOW)
    check("M1 does not alarm on a sub-hourly lane delivering GitHub's real ceiling", not f)
    # A firing NEWER than the grace cut-off must not be counted as missing.
    edge = _lane(fires=0, now=NOW)
    edge["schedule_run_times"] = [NOW - dt.timedelta(minutes=1)] * CAP
    f, _ = find_cron_deficits([edge], NOW)
    check("M1 ignores firings inside the grace window (they are not yet due)", len(f) == 1)
    f, c = find_cron_deficits([_lane(cron_only=True, in_scope=False, fires=0)], NOW)
    check("M1 never raises on a cron-only lane",
          not f and c.get("cron-only (watched by #4368)") == 1)
    f, c = find_cron_deficits([_lane(state="disabled_manually", fires=0, now=NOW)], NOW)
    check("M1 quiet on a disabled lane", not f and c.get("disabled") == 1)
    f, c = find_cron_deficits([_lane(crons=("not a cron",), fires=0, now=NOW)], NOW)
    check("M1 quiet + counted on an unparseable cron",
          not f and c.get("cron-unparseable") == 1)
    # A weekly lane has a capped expectation of 0 inside a 24h window -> below the floor.
    f, c = find_cron_deficits([_lane(crons=("0 6 * * 1",), fires=0, now=NOW)], NOW)
    check("M1 quiet when the expectation is below the floor",
          not f and c.get("expectation-below-floor") == 1)
    # Boundary direction: at/above the delivery floor must NOT raise; just below must.
    import math  # noqa: PLC0415
    at_floor = math.ceil(CAP * CRON_DELIVERY_FLOOR)
    f, _ = find_cron_deficits([_lane(fires=at_floor, now=NOW)], NOW)
    check("M1 does not raise AT the delivery floor", not f)
    f, _ = find_cron_deficits([_lane(fires=at_floor - 1, now=NOW)], NOW)
    check("M1 does raise just BELOW the delivery floor", len(f) == 1)
    # Over-delivery clamps to 1.00 rather than exceeding it.
    f, _ = find_cron_deficits([_lane(fires=CAP * 3, now=NOW)], NOW)
    check("M1 clamps an over-delivering lane to healthy", not f)

    # --- M2 ---
    late = _run_obj(status="queued", age_min=int(QUEUE_MAX_WAIT_SECONDS / 60) + 5, now=NOW)
    early = _run_obj(status="queued", age_min=1, now=NOW)
    f, c = find_queue_overruns([late], NOW)
    check("M2 raises past the queue threshold",
          len(f) == 1 and c.get("queued-over-threshold") == 1)
    f, c = find_queue_overruns([early], NOW)
    check("M2 quiet within the queue threshold",
          not f and c.get("queued-within-threshold") == 1)
    f, _ = find_queue_overruns([_run_obj(status="in_progress", age_min=999, now=NOW)], NOW)
    check("M2 ignores in_progress runs (that is M3's job)", not f)

    # --- M3: detection variable is LIVE in_progress age ---
    base = {(".github/workflows/a.yml", "schedule"): {"p90": 3600.0, "n": 50}}
    f, c = find_execution_overruns([_run_obj(age_min=30, now=NOW)], base, NOW)
    check("M3 quiet inside the band", not f and c.get("in-progress-within-threshold") == 1)
    f, c = find_execution_overruns([_run_obj(age_min=8 * 60, now=NOW)], base, NOW)
    check("M3 raises past the band", len(f) == 1 and c.get("execution-overrun") == 1)
    f, _ = find_execution_overruns([_run_obj(status="queued", age_min=999, now=NOW)],
                                   base, NOW)
    check("M3 ignores queued runs (that is M2's job)", not f)
    # THE FAIL-OPEN HOLE: a lane whose runs never complete has no baseline. It must still
    # be watchable, or the detector goes silent exactly when a lane is 100% hung.
    f, _ = find_execution_overruns([_run_obj(age_min=7 * 60, now=NOW)], {}, NOW)
    check("M3 still raises with NO baseline at all (fail-open hole closed)", len(f) == 1)
    check("M3 names the floor as the basis when there is no baseline",
          f and "floor" in f[0]["basis"])
    thin = {(".github/workflows/a.yml", "schedule"): {"p90": 60.0, "n": 1}}
    f, _ = find_execution_overruns([_run_obj(age_min=7 * 60, now=NOW)], thin, NOW)
    check("M3 falls back to the floor on an under-sampled baseline", len(f) == 1)
    check("M3 does NOT use a 1-sample p90 as the threshold",
          f and f[0]["threshold_seconds"] == int(EXEC_FLOOR_SECONDS))
    # A big baseline must RAISE the threshold above the floor, not be ignored.
    big = {(".github/workflows/a.yml", "schedule"): {"p90": 8 * 3600.0, "n": 40}}
    f, _ = find_execution_overruns([_run_obj(age_min=7 * 60, now=NOW)], big, NOW)
    check("M3 respects a baseline WIDER than the floor", not f)
    f, _ = find_execution_overruns([_run_obj(age_min=17 * 60, now=NOW)], big, NOW)
    check("M3 raises on the measured 2026-07-27 17.3h-vs-8.3h shape", len(f) == 1)

    # --- census is emitted even on the all-clear ---
    _, c1 = find_cron_deficits([_lane(fires=36, now=NOW)], NOW)
    _, c2 = find_queue_overruns([early], NOW)
    _, c3 = find_execution_overruns([_run_obj(age_min=1, now=NOW)], base, NOW)
    check("a clean run still emits a non-empty census for every mode",
          bool(c1) and bool(c2) and bool(c3))

    # --- issue body carries the dedupe marker ---
    body = render_issue_body("o/r", [{"mode": "M2-queue-wait", "workflow": "a.yml",
                                      "run_id": 1, "event": "push", "waited_seconds": 99,
                                      "threshold_seconds": 60, "head_branch": "main"}],
                             {"M2-queue-wait": {"queued-over-threshold": 1}}, NOW)
    check("issue body ends with the dedupe marker",
          body.rstrip().endswith(f"<!-- {KEY_PREFIX}: o/r -->"))
    check("issue body self-identifies as a SPARQ agent", body.startswith("> 🤖"))
    check("issue body prints the census", "Census of every state exit" in body)

    # --- exit codes are distinct ---
    check("a bad repo slug is fail-loud exit 2",
          main(["--repo", "not-a-slug", "--dry-run"]) == 2)

    if failures:
        for name in failures:
            print(f"FAIL: {name}")
        print(f"::error::ci_execution_latency_alarm self-test: {len(failures)} failure(s)")
        return 1
    print("ci_execution_latency_alarm self-test: all checks passed")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="CI execution-latency alarm")
    parser.add_argument("--repo", default="")
    parser.add_argument("--now", default="")
    parser.add_argument("--state-file", default="")
    parser.add_argument("--window-hours", type=float, default=CRON_WINDOW_HOURS)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)
    if args.self_test:
        return _self_test()
    try:
        return run(args)
    except AlarmError as exc:
        print(f"::error::ci execution-latency alarm infrastructure failure: {exc}")
        return 2


if __name__ == "__main__":
    sys.exit(main())
