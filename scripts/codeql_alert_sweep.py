#!/usr/bin/env python3
# [FABLE-5] Daily CodeQL alert sweep — the RETROACTIVE-triage half of the
# 2026-07-17 "CodeQL is non-blocking" maintainer decision.
#
# CONTEXT. CodeQL no longer gates merges on any event (scripts/ci_summary_gate.py
# §CODEQL DEMOTION; docs/branch-protection.md §CodeQL is advisory): the merge
# queue neither waits on nor reds because of a CodeQL leg. The compensating
# control is THIS sweep: alerts must not rot unseen in the code-scanning
# dashboard, so a scheduled job (.github/workflows/codeql-alert-sweep.yml, daily)
# lists the repo's OPEN code-scanning alerts and maintains ONE rolling GitHub
# issue — "CodeQL alerts requiring retroactive triage" — as the standing triage
# work item. This is the same relocated-signal-must-file-an-issue posture as the
# demoted-lane filer (scripts/ci-file-demoted-lane-failure.py), specialised to
# code-scanning alerts.
#
# BEHAVIOUR (idempotent upsert):
#   * open alerts present, no rolling issue  -> CREATE it (labels: from:agent +
#     self-improvement), listing every open alert (number / rule / severity / path).
#   * open alerts present, issue exists      -> UPDATE the body iff the alert set
#     changed (a NEW untracked alert appeared, or an alert was resolved); an
#     unchanged set is a NO-OP — re-running the sweep never churns the issue.
#   * zero open alerts, issue open           -> COMMENT + CLOSE it.
#   * zero open alerts, no issue             -> NO-OP.
# The issue is identified by a machine-readable MARKER comment in its body (never
# by title alone), and the marker carries the tracked alert numbers so "new vs
# already-tracked" is computable without parsing the human-readable table.
#
# SAFETY / HONESTY:
#   * Read side needs only `security-events: read`; write side only `issues: write`
#     (least privilege — the workflow grants exactly those).
#   * Any fetch failure aborts loudly BEFORE any write (a partial alert list must
#     never close or shrink the rolling issue).
#   * All decision logic is PURE (plan() below) and hermetically unit-tested with
#     fixtures: scripts/tests/test_codeql_alert_sweep.py (stdlib-only; no network).
#
# USAGE (the workflow):  REPO=owner/name GH_TOKEN=... python3 scripts/codeql_alert_sweep.py
#        (add --dry-run to print the planned action without writing)

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys

ISSUE_TITLE = "CodeQL alerts requiring retroactive triage"
LABELS = ("from:agent", "self-improvement")
MARKER_PREFIX = "codeql-alert-sweep:v1"
MARKER_RE = re.compile(r"<!--\s*" + re.escape(MARKER_PREFIX) + r"\s+tracked=(\[[0-9,\s]*\])\s*-->")


class SweepError(RuntimeError):
    """A gh/API failure — abort the sweep before any write."""


# ----------------------------- pure logic (unit-tested) -----------------------------


def normalize_alert(raw: dict) -> dict:
    """Project a code-scanning-API alert object down to the fields the issue
    lists. Tolerant of absent fields (the API omits location for some alert
    kinds) — every field degrades to a placeholder, never a KeyError."""
    rule = raw.get("rule") or {}
    inst = raw.get("most_recent_instance") or {}
    loc = inst.get("location") or {}
    return {
        "number": int(raw.get("number", 0)),
        "rule": rule.get("id") or rule.get("name") or "(unknown rule)",
        "severity": rule.get("security_severity_level") or rule.get("severity") or "(unknown)",
        "path": loc.get("path") or "(no path)",
        "url": raw.get("html_url") or "",
    }


def marker_line(numbers: list[int]) -> str:
    return f"<!-- {MARKER_PREFIX} tracked={json.dumps(sorted(numbers))} -->"


def parse_tracked(body: str) -> set[int] | None:
    """The alert numbers a rolling-issue body already tracks, from its marker.
    None when the body carries no (parseable) marker — the caller treats that as
    'not the rolling issue' (identification is marker-based, never title-based)."""
    m = MARKER_RE.search(body or "")
    if not m:
        return None
    try:
        return {int(n) for n in json.loads(m.group(1))}
    except (ValueError, TypeError):
        return None


def build_issue_body(alerts: list[dict], repo: str) -> str:
    """Deterministic issue body (alerts sorted by number) so an unchanged alert
    set always renders byte-identical — the idempotence the upsert compares on."""
    alerts = sorted(alerts, key=lambda a: a["number"])
    lines = [
        marker_line([a["number"] for a in alerts]),
        "",
        "> 🤖 Rolling issue maintained automatically by the **daily CodeQL alert sweep**",
        "> (`.github/workflows/codeql-alert-sweep.yml` → `scripts/codeql_alert_sweep.py`).",
        "> CodeQL is **advisory at merge time** (2026-07-17 maintainer decision — merge-queue",
        "> throughput); open code-scanning alerts are triaged **retroactively** here instead.",
        "> This issue updates when the open-alert set changes and closes itself at zero.",
        "",
        f"## Open code-scanning alerts ({len(alerts)})",
        "",
        "| Alert | Rule | Severity | Path |",
        "|---|---|---|---|",
    ]
    for a in alerts:
        ref = f"[#{a['number']}]({a['url']})" if a["url"] else f"#{a['number']}"
        lines.append(f"| {ref} | `{a['rule']}` | {a['severity']} | `{a['path']}` |")
    lines += [
        "",
        f"Full list: https://github.com/{repo}/security/code-scanning",
        "",
        "Triage each alert: fix it, or dismiss it in the dashboard with a reason.",
        "The next sweep reflects either outcome automatically.",
    ]
    return "\n".join(lines)


def plan(alerts: list[dict], issue: dict | None, repo: str) -> dict:
    """The whole decision, purely. `alerts` are normalized open alerts; `issue`
    is the open rolling issue ({number, body}) or None. Returns one of:
      {action: "none",   reason: str}
      {action: "create", title, body, labels}
      {action: "update", number, body, new: [ints], resolved: [ints]}
      {action: "close",  number, comment}
    """
    if not alerts:
        if issue is None:
            return {"action": "none", "reason": "no open alerts and no rolling issue"}
        return {
            "action": "close",
            "number": issue["number"],
            "comment": (
                "🤖 All code-scanning alerts are resolved or dismissed — closing this "
                "rolling triage issue. The daily sweep reopens a fresh one if new "
                "alerts appear."
            ),
        }
    body = build_issue_body(alerts, repo)
    current = {a["number"] for a in alerts}
    if issue is None:
        return {"action": "create", "title": ISSUE_TITLE, "body": body,
                "labels": list(LABELS)}
    tracked = parse_tracked(issue.get("body", "")) or set()
    if tracked == current and issue.get("body") == body:
        return {"action": "none",
                "reason": f"alert set unchanged ({len(current)} tracked)"}
    return {
        "action": "update",
        "number": issue["number"],
        "body": body,
        "new": sorted(current - tracked),
        "resolved": sorted(tracked - current),
    }


# ----------------------------- live (gh-backed) wiring -----------------------------


def _gh_api(args: list[str]) -> str:
    try:
        proc = subprocess.run(["gh", "api", *args], capture_output=True, text=True)
    except OSError as exc:
        raise SweepError(f"gh spawn failed: {exc}") from exc
    if proc.returncode != 0:
        raise SweepError(proc.stderr.strip()[:400] or f"gh api exited {proc.returncode}")
    return proc.stdout


def fetch_open_alerts(repo: str) -> list[dict]:
    # 404 with code scanning never enabled / no analyses yet: treat as an ERROR,
    # not zero alerts — "no analysis uploaded" must never close the rolling issue.
    out = _gh_api([
        f"repos/{repo}/code-scanning/alerts?state=open&per_page=100",
        "--paginate", "--jq", ".[]",
    ])
    return [normalize_alert(json.loads(line)) for line in out.splitlines() if line.strip()]


def find_rolling_issue(repo: str) -> dict | None:
    """The open rolling issue, identified by the body MARKER (title is display
    only). Searches open issues under the sweep's own label to bound the scan."""
    out = _gh_api([
        f"repos/{repo}/issues?state=open&labels=from:agent&per_page=100",
        "--paginate", "--jq", ".[] | {number, body}",
    ])
    for line in out.splitlines():
        if not line.strip():
            continue
        issue = json.loads(line)
        if parse_tracked(issue.get("body") or "") is not None:
            return {"number": issue["number"], "body": issue.get("body") or ""}
    return None


def execute(repo: str, action: dict) -> None:
    kind = action["action"]
    if kind == "none":
        print(f"sweep: no-op — {action['reason']}")
        return
    if kind == "create":
        args = [f"repos/{repo}/issues", "-f", f"title={action['title']}",
                "-f", f"body={action['body']}"]
        for lbl in action["labels"]:
            args += ["-f", f"labels[]={lbl}"]
        out = _gh_api(args)
        print(f"sweep: created rolling issue #{json.loads(out)['number']}")
        return
    if kind == "update":
        _gh_api([f"repos/{repo}/issues/{action['number']}", "-X", "PATCH",
                 "-f", f"body={action['body']}"])
        print(f"sweep: updated #{action['number']} "
              f"(new alerts: {action['new'] or 'none'}; resolved: {action['resolved'] or 'none'})")
        return
    if kind == "close":
        _gh_api([f"repos/{repo}/issues/{action['number']}/comments", "-f",
                 f"body={action['comment']}"])
        _gh_api([f"repos/{repo}/issues/{action['number']}", "-X", "PATCH",
                 "-f", "state=closed"])
        print(f"sweep: closed #{action['number']} — zero open alerts")
        return
    raise SweepError(f"unknown action {kind!r}")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--dry-run", action="store_true",
                    help="print the planned action; write nothing")
    ns = ap.parse_args()
    repo = os.environ.get("REPO", "")
    if not repo:
        print("::error::codeql-alert-sweep: REPO must be set (owner/name).")
        return 1
    try:
        alerts = fetch_open_alerts(repo)
        issue = find_rolling_issue(repo)
        action = plan(alerts, issue, repo)
        print(f"sweep: {len(alerts)} open alert(s); rolling issue: "
              f"{('#' + str(issue['number'])) if issue else 'none'}; "
              f"planned action: {action['action']}")
        if ns.dry_run:
            print(json.dumps(action, indent=2))
            return 0
        execute(repo, action)
    except SweepError as exc:
        print(f"::error::codeql-alert-sweep failed (nothing written on fetch "
              f"failure): {exc}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
