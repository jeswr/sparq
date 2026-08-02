#!/usr/bin/env python3
"""Merge-queue WALL-CLOCK POLE profiler — which workflow actually ends a merge_group
entry last, measured on demand. 🤖 SPARQ agent. Issue #5250.

WHY THIS EXISTS (the failure mode it is the fix for)
====================================================
Issue #3005 scoped a docs/deploy fast lane around "CodeQL-rust costs a merge_group
~20-40 minutes". That number was true once, and then `codeql.yml` migrated to
`build-mode: none` buildless extraction and it stopped being true — but the FIGURE
survived the migration that falsified it, got quoted into a bead, and re-scoped an
epic. The skip #3005 shipped is still sound (a rust-inert batch has no Rust to
analyse); it is simply a modest saving rather than the headline win, and the epic's
"docs-only merge_group in under ten minutes" target had been derived from a premise
that no longer held. `codeql.yml`'s own header already carried the correction.

The durable fix is NOT another number pasted into markdown — that is the same defect
one iteration later. It is an instrument: ask GitHub which workflow ends the entry,
today, in one command, and let the answer be re-derived whenever the fast-lane work
is scoped. So this script prints no baked-in expectations and asserts no poles; it
reports what the API says, and the caller reads the top of the table.

WHAT IS MEASURED, PRECISELY
===========================
An ENTRY is one queued merge_group batch, keyed by the merge-queue `head_sha` that
every workflow triggered by that batch shares. For each entry:

  entry wall   = max(updated_at) - min(created_at) over the batch's runs. `created_at`
                 (not `run_started_at`) is the start, because the maintainer-visible
                 cost of an entry INCLUDES the time a run object sat before a runner
                 picked it up; measuring execution only would hide provisioning
                 contention, which the 2026-07 profile measured as real wall-clock.
  the CLOSER   = the run with the maximum `updated_at`. This, not "the workflow with
                 the largest median duration", is the pole: a slow workflow that
                 starts early and finishes mid-pack costs the queue nothing.
  exclusive
  TAIL         = max(updated_at) - second-largest(updated_at). The wall-clock that
                 deleting the closer would actually save, holding every other run
                 fixed. A closer with a near-zero tail is a co-pole, and removing it
                 buys ~nothing — the distinction that the ~20-40 min story got wrong.

Poles are ranked by how OFTEN a workflow closes its entry, then by total exclusive
tail owned. Both columns are printed, because they disagree in an informative way:
a workflow that closes rarely but owns a huge tail when it does is a variance
problem, not a throughput problem.

WHAT IS DELIBERATELY NOT MEASURED
=================================
  * JOB-level attribution inside the pole workflow. The pole workflow tells you which
    lane to open; `research/ci-mergequeue-speedup-2026-07.md` §2.2 shows the shape of
    the job-level decomposition once you know which workflow to decompose. Fetching
    jobs for every run of every entry is a much larger API walk and is not needed to
    answer "which leg dominates a merge_group".
  * CHANGE-CLASS of each batch. Deriving "is this a docs-only entry" needs the
    batch's base..head diff, which is not in the runs payload and is usually not in a
    shallow checkout. Use `--since` around a known docs/deploy drain instead.

Entries containing a run that did not conclude cleanly (`cancelled`, `failure`,
`timed_out`, `startup_failure`, `action_required`) are EXCLUDED by default and
counted in the census: a batch ejected from the queue stops early, so its wall
understates the cost of a batch that merges. `--include-unclean` relaxes this; the
census line always reports how many entries each rule dropped, so a filtered
population can never be mistaken for the whole one.

STATUS — what is verified and what is not
=========================================
The pure core (grouping, closer attribution, exclusive tail, the population rules,
the ranking, the fail-loud exits) is exercised by
`scripts/tests/test_ci_merge_group_poles.py` — hermetic, injected payloads, and
mutation-checked: inverting any of those guards reds the suite. The `gh api` walk in
`fetch_merge_group_runs` is DOCUMENTED-UNTESTED — it was written against the runs API
contract but has not been run against the live API from the authoring environment, so
validate the query (`event`, `status`, the `created=>=` range encoding) on first real
invocation. Nothing gates on this script; a wrong query surfaces as an obviously empty
or short census, not as a silent wrong answer in CI.

USAGE
=====
  scripts/ci_merge_group_poles.py --repo owner/name              # last 14 days
  scripts/ci_merge_group_poles.py --since 2026-07-25 --top 5
  scripts/ci_merge_group_poles.py --json                         # machine-readable

Exit codes: 0 report produced, 2 infrastructure failure (gh/API/parse), 3 the window
held no usable entry (fail-loud — an empty profile must never read as "no poles").
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import statistics
import subprocess
import sys
import urllib.parse

# A run that concluded in one of these did its full work; anything else means the
# batch was cut short and its wall understates a merging batch.
CLEAN_CONCLUSIONS = frozenset({"success", "skipped", "neutral"})

DEFAULT_WINDOW_DAYS = 14
RUNS_PAGE_CAP = 20  # 100/page => 2000 runs, ~an order of magnitude over a 14d window


class PoleError(RuntimeError):
    """Infrastructure failure: gh, the API shape, or a timestamp we cannot parse."""


# ---------------------------------------------------------------------------------
# pure core (hermetic — every function below takes data, never touches the network)
# ---------------------------------------------------------------------------------
def _ts(value: str) -> dt.datetime:
    try:
        return dt.datetime.fromisoformat(str(value).replace("Z", "+00:00"))
    except (ValueError, TypeError) as exc:
        raise PoleError(f"unparseable timestamp {value!r}: {exc}") from exc


def percentile(values: list[float], q: float) -> float:
    ordered = sorted(values)
    idx = min(len(ordered) - 1, int(round(q * (len(ordered) - 1))))
    return ordered[idx]


def workflow_name(run: dict) -> str:
    """Display key for a run. `path` is the stable identity (a workflow's `name:` can
    be edited without changing the lane); the name is only a fallback."""
    path = str(run.get("path") or "")
    if path:
        return path.split("/")[-1]
    return str(run.get("name") or "<unnamed>")


def latest_attempts(runs: list[dict]) -> list[dict]:
    """Collapse re-runs: one run per (head_sha, workflow), the highest `run_attempt`.

    Without this a re-run of a failed leg is counted as a second, later-ending run of
    the same lane on the same entry — which both double-counts the lane and hands it a
    spurious closer credit, since a re-run by construction ends after everything else.
    """
    best: dict[tuple[str, str], dict] = {}
    for run in runs:
        key = (str(run.get("head_sha") or ""), workflow_name(run))
        prior = best.get(key)
        if prior is None or int(run.get("run_attempt") or 1) >= int(prior.get("run_attempt") or 1):
            best[key] = run
    return list(best.values())


def group_entries(runs: list[dict], *, include_unclean: bool = False
                  ) -> tuple[list[dict], dict[str, int]]:
    """Group merge_group runs into per-entry profiles. Returns (entries, census).

    The census counts every DROPPED entry by reason so the report can state what the
    population excludes rather than silently narrowing it.
    """
    census = {"entries_seen": 0, "dropped_incomplete": 0, "dropped_unclean": 0,
              "dropped_single_run": 0}
    by_sha: dict[str, list[dict]] = {}
    for run in latest_attempts(runs):
        sha = str(run.get("head_sha") or "")
        if not sha:
            continue
        by_sha.setdefault(sha, []).append(run)

    entries: list[dict] = []
    for sha, batch in by_sha.items():
        census["entries_seen"] += 1
        if any(str(r.get("status")) != "completed" for r in batch):
            census["dropped_incomplete"] += 1
            continue
        if not include_unclean and any(
                str(r.get("conclusion")) not in CLEAN_CONCLUSIONS for r in batch):
            census["dropped_unclean"] += 1
            continue
        ends = sorted(((_ts(r["updated_at"]), workflow_name(r)) for r in batch),
                      key=lambda pair: pair[0])
        if len(ends) < 2:
            # One run cannot have a closer distinct from itself, and its "exclusive
            # tail" would be the whole entry — an artefact, not a measurement.
            census["dropped_single_run"] += 1
            continue
        start = min(_ts(r["created_at"]) for r in batch)
        end, closer = ends[-1]
        runner_up = ends[-2][0]
        entries.append({
            "head_sha": sha,
            "start": start.isoformat(),
            "end": end.isoformat(),
            "wall_s": (end - start).total_seconds(),
            "closer": closer,
            "tail_s": (end - runner_up).total_seconds(),
            "runs": [{"workflow": workflow_name(r),
                      "duration_s": (_ts(r["updated_at"]) - _ts(r["created_at"])).total_seconds()}
                     for r in batch],
        })
    entries.sort(key=lambda e: e["start"])
    return entries, census


def rank_poles(entries: list[dict]) -> list[dict]:
    """Per-workflow aggregate over the entry population, poles first.

    Ranked by closes, then by owned tail — see the header on why those two columns
    are reported side by side instead of collapsed into one score.
    """
    stats: dict[str, dict] = {}
    for entry in entries:
        for run in entry["runs"]:
            row = stats.setdefault(run["workflow"], {
                "workflow": run["workflow"], "n": 0, "durations": [],
                "closes": 0, "tail_s": 0.0})
            row["n"] += 1
            row["durations"].append(run["duration_s"])
        closer = stats.get(entry["closer"])
        if closer is not None:
            closer["closes"] += 1
            closer["tail_s"] += entry["tail_s"]

    poles = []
    for row in stats.values():
        durations = row.pop("durations")
        poles.append({**row,
                      "median_s": statistics.median(durations),
                      "p90_s": percentile(durations, 0.90),
                      "max_s": max(durations),
                      "close_rate": row["closes"] / len(entries) if entries else 0.0})
    poles.sort(key=lambda r: (r["closes"], r["tail_s"]), reverse=True)
    return poles


def _m(seconds: float) -> str:
    return f"{seconds / 60.0:.1f}m"


def render(entries: list[dict], poles: list[dict], census: dict[str, int],
           top: int) -> str:
    walls = [e["wall_s"] for e in entries]
    # A lane that never closed an entry is NOT a pole and must never be printed under a
    # "TOP POLES" heading to pad it to `top` — that padding would invite the exact
    # misreading (expensive-looking lane ⇒ pole) this profiler exists to prevent.
    closers = [row for row in poles if row["closes"] > 0]
    lines = [
        f"merge_group entries profiled: {len(entries)}"
        f"  (seen {census['entries_seen']};"
        f" dropped: incomplete {census['dropped_incomplete']},"
        f" unclean {census['dropped_unclean']},"
        f" single-run {census['dropped_single_run']})",
        f"entry wall: median {_m(percentile(walls, 0.5))}"
        f"  p90 {_m(percentile(walls, 0.90))}  max {_m(max(walls))}",
        "",
        f"TOP {min(top, len(closers))} POLES of {len(closers)} lane(s) that ever ended an "
        f"entry (ranked by how often the lane ENDS the entry):",
        f"  {'workflow':<34} {'closes':>7} {'rate':>6} {'owned tail':>11} {'median':>8} {'p90':>8}",
    ]
    for row in closers[:top]:
        lines.append(
            f"  {row['workflow']:<34} {row['closes']:>7} {row['close_rate']:>5.0%}"
            f" {_m(row['tail_s']):>11} {_m(row['median_s']):>8} {_m(row['p90_s']):>8}")
    lines += ["", "ALL LANES (duration only — a long lane that ends mid-pack is not a pole):",
              f"  {'workflow':<34} {'n':>4} {'median':>8} {'p90':>8} {'max':>8}"]
    for row in sorted(poles, key=lambda r: r["median_s"], reverse=True):
        lines.append(f"  {row['workflow']:<34} {row['n']:>4} {_m(row['median_s']):>8}"
                     f" {_m(row['p90_s']):>8} {_m(row['max_s']):>8}")
    return "\n".join(lines)


# ---------------------------------------------------------------------------------
# gh I/O
# ---------------------------------------------------------------------------------
def _gh(args: list[str]):
    try:
        out = subprocess.run(["gh", *args], check=True, capture_output=True,
                             text=True, timeout=180)
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError) as exc:
        detail = getattr(exc, "stderr", "") or str(exc)
        raise PoleError(f"gh {' '.join(args[:2])} failed: {detail}") from exc
    try:
        return json.loads(out.stdout or "null")
    except json.JSONDecodeError as exc:
        raise PoleError(f"gh {' '.join(args[:2])}: unparseable JSON: {exc}") from exc


def fetch_merge_group_runs(repo: str, since: str) -> list[dict]:
    """Every completed `merge_group` run created on/after `since` (YYYY-MM-DD).

    Pages until a SHORT page, never on `total_count`: that field is read before the
    walk finishes and an undercount would truncate the window silently.
    """
    created = urllib.parse.quote(f">={since}", safe="")
    runs: list[dict] = []
    page = 1
    while page <= RUNS_PAGE_CAP:
        payload = _gh(["api", f"repos/{repo}/actions/runs"
                              f"?event=merge_group&status=completed&created={created}"
                              f"&per_page=100&page={page}"])
        if not isinstance(payload, dict) or payload.get("workflow_runs") is None:
            raise PoleError("workflow-run listing carries no workflow_runs")
        batch = payload["workflow_runs"]
        runs.extend(batch)
        if len(batch) < 100:
            return runs
        page += 1
    raise PoleError(f"merge_group run listing exceeded {RUNS_PAGE_CAP} pages since {since}")


def run(args: argparse.Namespace) -> int:
    repo = args.repo or os.environ.get("GITHUB_REPOSITORY") or ""
    if not repo:
        raise PoleError("--repo (or $GITHUB_REPOSITORY) required, as owner/name")
    since = args.since or (
        dt.datetime.now(dt.timezone.utc) - dt.timedelta(days=args.days)
    ).date().isoformat()
    entries, census = group_entries(fetch_merge_group_runs(repo, since),
                                    include_unclean=args.include_unclean)
    if not entries:
        print(f"::error::no usable merge_group entry since {since} "
              f"(seen {census['entries_seen']}; see the census rules in the header) — "
              f"widen --since or pass --include-unclean rather than reading this as "
              f"'no poles'")
        return 3
    poles = rank_poles(entries)
    if args.json:
        print(json.dumps({"repo": repo, "since": since, "census": census,
                          "entries": len(entries), "poles": poles}, indent=2))
    else:
        print(f"repo {repo}  event merge_group  since {since}\n")
        print(render(entries, poles, census, args.top))
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Profile which workflow ends a merge_group entry last")
    parser.add_argument("--repo", default="", help="owner/name (default $GITHUB_REPOSITORY)")
    parser.add_argument("--since", default="", help="YYYY-MM-DD (default --days ago)")
    parser.add_argument("--days", type=int, default=DEFAULT_WINDOW_DAYS)
    parser.add_argument("--top", type=int, default=3, help="poles to headline (default 3)")
    parser.add_argument("--include-unclean", action="store_true",
                        help="also count entries whose runs failed/were cancelled")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    try:
        return run(args)
    except PoleError as exc:
        print(f"::error::merge_group pole profiler failed: {exc}")
        return 2


if __name__ == "__main__":
    sys.exit(main())
