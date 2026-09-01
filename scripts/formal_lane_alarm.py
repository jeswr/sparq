#!/usr/bin/env python3
# [FABLE-5] Formal-lane NO-VERDICT alarm (bead sq-63ups). 🤖 SPARQ agent.
#
# WHAT THIS IS: the "silence must not look like green" safeguard for the SCHEDULED
# formal-verification lanes (kani.yml, miri.yml — the [[lane]] entries in
# ci/formal-verification.toml). Those lanes are deliberately OFF the per-PR critical
# path (nightly cron, no PR check-run), which means their failure mode is INVISIBLE:
# miri.yml was CANCELLED at its 90-minute ceiling 22 consecutive nights (2026-06-16 →
# 2026-07-07) — conclusion `cancelled`, never `failure` — and no human or gate noticed,
# because nothing red ever appeared anywhere. This script makes that class of rot loud:
# a lane whose newest SUCCESSFUL completed run on main is older than the manifest's
# `max_verdict_age_hours` (48h = two missed nightly verdicts) REDs this checker's own
# scheduled run AND files ONE deduped, self-identified SPARQ-agent GitHub issue.
#
# VERDICT DEFINITION (honest, run-level): a lane "has a verdict" iff a workflow RUN on
# main completed with conclusion `success` inside the window. A completed FAILURE run is
# a LOUD verdict for a human (the lane is visibly red in the Actions tab) but it is NOT
# a successful verdict — per the mandate, >48h without green is alarm-worthy either way;
# the issue body distinguishes the two cases (SILENT: cancelled/no runs at all vs RED:
# failing loudly) so triage starts in the right place. Note the consequence, stated
# plainly: while a lane is red every night on a KNOWN budget problem (e.g. the two
# mmap-validator totality harnesses, sq-og8u8), this alarm keeps one open issue alive —
# that is deliberate standing pressure, not noise (the dedupe key caps it at one issue).
#
# TWO DESIGN INVARIANTS (mirrors scripts/ci_selection_alarm.py):
#   1. FAIL-LOUD: an alarm-INFRASTRUCTURE error (cannot read the manifest, cannot list
#      workflow runs) exits NON-ZERO with a `::error::` annotation — a broken detector
#      must never look like a quiet green.
#   2. NON-SPAMMY: one open issue per lane, keyed by a stable
#      `<!-- formal-alarm-key: <workflow> -->` body marker searched over open
#      `formal-alarm`-labelled issues before filing (the flow-on idempotency mechanism).
#
# NOT A GATE: this runs from a schedule on main (formal-alarm.yml); it creates no
# check-run on any PR head, so it can never interact with the `ci-summary / gate` poll
# set. Redding this run blocks nothing — it exists to be SEEN.
#
# Usage:
#   formal_lane_alarm.py                          # real (gh api; env GITHUB_REPOSITORY)
#   formal_lane_alarm.py --dry-run                # print findings; no gh writes
#   formal_lane_alarm.py --runs-file runs.json \
#       --now 2026-07-07T12:00:00Z --dry-run      # hermetic (tests)
#   formal_lane_alarm.py --self-test              # hermetic fixtures
#
# Stdlib only + the `gh` CLI (present on every GitHub runner).

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gh_enumerate  # noqa: E402 - needs the sys.path line above

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MANIFEST = REPO_ROOT / "ci" / "formal-verification.toml"

BASE_LABELS = ["formal-alarm", "auto"]
KEY_PREFIX = "formal-alarm-key"


class AlarmError(Exception):
    """Alarm-INFRASTRUCTURE failure — surfaced LOUD (non-zero exit)."""


# --------------------------------------------------------------------------- #
# Manifest
# --------------------------------------------------------------------------- #
def load_lanes(manifest: Path) -> list[dict]:
    import tomllib

    try:
        with open(manifest, "rb") as fh:
            data = tomllib.load(fh)
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise AlarmError(f"cannot read manifest {manifest}: {exc}") from exc
    lanes = data.get("lane")
    if not isinstance(lanes, list) or not lanes:
        raise AlarmError(f"{manifest}: [[lane]] array missing or empty")
    for i, lane in enumerate(lanes):
        if "workflow" not in lane or "max_verdict_age_hours" not in lane:
            raise AlarmError(f"{manifest}: lane #{i} needs workflow + max_verdict_age_hours")
    return lanes


# --------------------------------------------------------------------------- #
# Run history (gh api, or a hermetic --runs-file)
# --------------------------------------------------------------------------- #
def _gh_json(args: list[str]) -> dict:
    try:
        out = subprocess.run(
            ["gh", "api", *args], check=True, capture_output=True, text=True, timeout=120
        )
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError) as exc:
        detail = getattr(exc, "stderr", "") or str(exc)
        raise AlarmError(f"gh api {' '.join(args)} failed: {detail}") from exc
    try:
        return json.loads(out.stdout)
    except json.JSONDecodeError as exc:
        raise AlarmError(f"gh api {' '.join(args)}: unparseable JSON: {exc}") from exc


def fetch_lane_runs(repo: str, workflow: str, runs_file: str | None) -> list[dict]:
    """-> completed runs on main, newest first: [{conclusion, created_at}]."""
    if runs_file:
        with open(runs_file, encoding="utf-8") as fh:
            data = json.load(fh)
        return data.get(workflow, [])
    resp = _gh_json(
        [
            f"repos/{repo}/actions/workflows/{workflow}/runs"
            f"?branch=main&status=completed&per_page=20"
        ]
    )
    runs = resp.get("workflow_runs")
    if runs is None:
        raise AlarmError(f"{workflow}: response carries no workflow_runs")
    return [{"conclusion": r.get("conclusion"), "created_at": r.get("created_at")} for r in runs]


def _parse_iso(ts: str) -> dt.datetime:
    return dt.datetime.fromisoformat(ts.replace("Z", "+00:00"))


# --------------------------------------------------------------------------- #
# The check (pure)
# --------------------------------------------------------------------------- #
def check_lane(lane: dict, runs: list[dict], now: dt.datetime) -> dict | None:
    """-> a violation dict, or None when the lane has a fresh successful verdict."""
    workflow = lane["workflow"]
    max_age = float(lane["max_verdict_age_hours"])
    successes = [r for r in runs if r.get("conclusion") == "success"]
    newest_success = None
    if successes:
        newest_success = max(_parse_iso(r["created_at"]) for r in successes)
        age_h = (now - newest_success).total_seconds() / 3600.0
        if age_h <= max_age:
            return None
    recent = [str(r.get("conclusion")) for r in runs[:7]]
    # Classification for triage: a lane failing LOUDLY (completed failures) needs a
    # different first move than one dying SILENTLY (cancelled-at-ceiling / never runs).
    mode = "RED" if any(c == "failure" for c in recent) else "SILENT"
    return {
        "workflow": workflow,
        "mode": mode,
        "max_age_hours": max_age,
        "last_success": newest_success.isoformat() if newest_success else None,
        "recent_conclusions": recent,
        "owner_bead": lane.get("owner_bead", ""),
        "description": lane.get("description", ""),
    }


def render_issue(v: dict) -> tuple[str, str]:
    since = v["last_success"] or "NEVER (no successful run on main in the fetched window)"
    title = (
        f"formal-alarm: {v['workflow']} has produced no successful verdict "
        f"for >{int(v['max_age_hours'])}h"
    )
    body = "\n".join(
        [
            "> 🤖 **SPARQ agent** — automated formal-lane no-verdict alarm "
            "(scripts/formal_lane_alarm.py, bead sq-63ups). @jeswr runs multiple agents "
            "on this account; this issue was filed by CI automation.",
            "",
            f"<!-- {KEY_PREFIX}: {v['workflow']} -->",
            "",
            f"The scheduled formal lane `{v['workflow']}` ({v['description']}) has produced "
            f"**no successful verdict on `main` for more than {int(v['max_age_hours'])} hours**.",
            "",
            f"* Failure mode: **{v['mode']}** — "
            + (
                "the lane is completing with `failure` (loudly red in the Actions tab); "
                "read the newest run's step summary for the failing harnesses/tests."
                if v["mode"] == "RED"
                else "the lane is being cancelled/never completing (e.g. killed at its "
                "`timeout-minutes` ceiling) — the failure class that produces NO signal "
                "anywhere. Read the newest run's log tail to see where it stalled."
            ),
            f"* Newest successful run: `{since}`",
            f"* Recent completed-run conclusions (newest first): {v['recent_conclusions']}",
            f"* Owner bead: `{v['owner_bead']}`" if v["owner_bead"] else "* Owner bead: (none)",
            "",
            "A formal lane with no verdict is NOT green — it is unverified. Fix the lane "
            "(or the code it verifies); do not mute this alarm. Config: "
            "`ci/formal-verification.toml` `[[lane]]`.",
        ]
    )
    return title, body


# --------------------------------------------------------------------------- #
# Issue minting (mirrors ci_selection_alarm.py: label upsert + open-issue dedupe)
# --------------------------------------------------------------------------- #
def _run_gh(args: list[str]) -> str:
    try:
        out = subprocess.run(args, check=True, capture_output=True, text=True, timeout=120)
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError) as exc:
        detail = getattr(exc, "stderr", "") or str(exc)
        raise AlarmError(f"{' '.join(args)} failed: {detail}") from exc
    return out.stdout


def ensure_labels(repo: str) -> None:
    for label in BASE_LABELS:
        _run_gh(["gh", "label", "create", label, "--repo", repo, "--force"])


def open_issue_exists(repo: str, workflow: str) -> bool:
    # [OPUS-5] #4985: a fail-closed CEILING, not the old `--limit 100` cap. This is the
    # dedupe guard, so truncation is the fail-OPEN direction — a marker past the cap reads
    # as "nothing open" and files a duplicate on every run. Raising AlarmError stops the
    # run (main() reports it as ::error::) rather than spamming the tracker.
    out = _run_gh(
        [
            "gh", "issue", "list", "--repo", repo, "--state", "open",
            "--label", BASE_LABELS[0], "--json", "number,body",
        ] + gh_enumerate.limit_args(gh_enumerate.ISSUE_CEILING)
    )
    rows = gh_enumerate.guard(json.loads(out or "[]"), f"open '{BASE_LABELS[0]}' issues",
                              ceiling=gh_enumerate.ISSUE_CEILING, exc=AlarmError)
    marker = f"{KEY_PREFIX}: {workflow}"
    return any(marker in (item.get("body") or "") for item in rows)


def file_issue(repo: str, v: dict) -> None:
    title, body = render_issue(v)
    args = ["gh", "issue", "create", "--repo", repo, "--title", title, "--body", body]
    for label in BASE_LABELS:
        args += ["--label", label]
    url = _run_gh(args).strip()
    print(f"filed: {url}")


# --------------------------------------------------------------------------- #
# main
# --------------------------------------------------------------------------- #
def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description="Formal-lane no-verdict alarm.")
    p.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    p.add_argument("--repo", default=os.environ.get("GITHUB_REPOSITORY", ""))
    p.add_argument("--now", help="ISO timestamp override (tests)")
    p.add_argument("--runs-file", help="hermetic: JSON {workflow: [{conclusion, created_at}]}")
    p.add_argument("--dry-run", action="store_true", help="print findings; no gh writes")
    p.add_argument("--self-test", action="store_true")
    args = p.parse_args(argv)

    if args.self_test:
        return self_test()

    try:
        lanes = load_lanes(args.manifest)
        if not args.repo and not args.runs_file:
            raise AlarmError("--repo (or GITHUB_REPOSITORY) is required for a live run")
        now = _parse_iso(args.now) if args.now else dt.datetime.now(dt.timezone.utc)

        violations: list[dict] = []
        for lane in lanes:
            runs = fetch_lane_runs(args.repo, lane["workflow"], args.runs_file)
            v = check_lane(lane, runs, now)
            if v is None:
                print(f"OK: {lane['workflow']} has a successful verdict inside "
                      f"{lane['max_verdict_age_hours']}h")
            else:
                violations.append(v)
                print(f"::error title=formal lane has no verdict::{v['workflow']}: no "
                      f"successful run on main for >{int(v['max_age_hours'])}h "
                      f"(mode={v['mode']}; last success={v['last_success']}; "
                      f"recent={v['recent_conclusions']})")

        if not violations:
            print("formal-lane alarm: every scheduled formal lane has a fresh verdict.")
            return 0

        if args.dry_run:
            for v in violations:
                title, body = render_issue(v)
                print(f"--- DRY RUN issue ---\n{title}\n{body}\n")
        else:
            ensure_labels(args.repo)
            for v in violations:
                if open_issue_exists(args.repo, v["workflow"]):
                    print(f"dedupe: open formal-alarm issue already tracks {v['workflow']}")
                else:
                    file_issue(args.repo, v)
        # LOUD: the alarm run itself goes red while any lane is verdict-less.
        return 1
    except AlarmError as exc:
        # FAIL-LOUD: infrastructure breakage must never look like a quiet green.
        print(f"::error title=formal-lane alarm infrastructure error::{exc}")
        return 2


# --------------------------------------------------------------------------- #
# Hermetic self-test
# --------------------------------------------------------------------------- #
_FIXTURE_MANIFEST = """\
schema = 1
[[suite]]
id = "a"
name = "n"
crate = "c"
harnesses = ["h"]
proves = ["crates/c/src/x.rs"]
nightly = true
pr_gate = true
[[lane]]
workflow = "kani.yml"
owner_bead = "sq-hkud"
max_verdict_age_hours = 48
[[lane]]
workflow = "miri.yml"
max_verdict_age_hours = 48
"""


def self_test() -> int:
    failures: list[str] = []
    now = _parse_iso("2026-07-07T12:00:00Z")

    def check(cond: bool, label: str) -> None:
        if not cond:
            failures.append(label)

    lane = {"workflow": "miri.yml", "max_verdict_age_hours": 48, "owner_bead": "sq-fo28"}
    # 1. fresh success => no violation.
    check(
        check_lane(lane, [{"conclusion": "success", "created_at": "2026-07-07T05:20:00Z"}], now)
        is None,
        "fresh success passes",
    )
    # 2. stale success + cancelled tail => SILENT violation.
    v = check_lane(
        lane,
        [
            {"conclusion": "cancelled", "created_at": "2026-07-07T05:20:00Z"},
            {"conclusion": "cancelled", "created_at": "2026-07-06T05:20:00Z"},
            {"conclusion": "success", "created_at": "2026-07-01T05:20:00Z"},
        ],
        now,
    )
    check(v is not None and v["mode"] == "SILENT", "stale+cancelled is a SILENT violation")
    # 3. no runs at all => SILENT violation with last_success None.
    v = check_lane(lane, [], now)
    check(v is not None and v["last_success"] is None, "no-runs is a violation")
    # 4. completed failures => RED violation (loud but still verdict-less).
    v = check_lane(
        lane, [{"conclusion": "failure", "created_at": "2026-07-07T05:20:00Z"}], now
    )
    check(v is not None and v["mode"] == "RED", "failing lane is a RED violation")
    # 5. success just inside the window passes; just outside fails.
    check(
        check_lane(lane, [{"conclusion": "success", "created_at": "2026-07-05T13:00:00Z"}], now)
        is None,
        "47h-old success passes",
    )
    check(
        check_lane(lane, [{"conclusion": "success", "created_at": "2026-07-05T11:00:00Z"}], now)
        is not None,
        "49h-old success violates",
    )
    # 6. end-to-end via main(): hermetic runs-file + dry-run => exit 1 + rendered issue.
    with tempfile.TemporaryDirectory() as td:
        man = Path(td) / "fv.toml"
        man.write_text(_FIXTURE_MANIFEST)
        runs = Path(td) / "runs.json"
        runs.write_text(json.dumps({
            "kani.yml": [{"conclusion": "success", "created_at": "2026-07-07T07:00:00Z"}],
            "miri.yml": [{"conclusion": "cancelled", "created_at": "2026-07-07T05:20:00Z"}],
        }))
        rc = main([
            "--manifest", str(man), "--runs-file", str(runs),
            "--now", "2026-07-07T12:00:00Z", "--dry-run", "--repo", "x/y",
        ])
        check(rc == 1, "hermetic e2e: one stale lane => exit 1")
        # 7. all-fresh => exit 0.
        runs.write_text(json.dumps({
            "kani.yml": [{"conclusion": "success", "created_at": "2026-07-07T07:00:00Z"}],
            "miri.yml": [{"conclusion": "success", "created_at": "2026-07-07T06:00:00Z"}],
        }))
        rc = main([
            "--manifest", str(man), "--runs-file", str(runs),
            "--now", "2026-07-07T12:00:00Z", "--dry-run", "--repo", "x/y",
        ])
        check(rc == 0, "hermetic e2e: all fresh => exit 0")
        # 8. unreadable manifest => infrastructure error exit 2 (fail-loud).
        rc = main([
            "--manifest", str(Path(td) / "missing.toml"), "--runs-file", str(runs),
            "--now", "2026-07-07T12:00:00Z", "--dry-run", "--repo", "x/y",
        ])
        check(rc == 2, "missing manifest => fail-loud exit 2")

    if failures:
        for f in failures:
            print(f"::error::formal_lane_alarm self-test FAILED: {f}")
        return 1
    print("formal_lane_alarm self-test: freshness window, SILENT/RED classification, "
          "e2e exits all behaved.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
