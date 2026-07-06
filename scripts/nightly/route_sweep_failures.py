#!/usr/bin/env python3
# [OPUS-4.8] sq-ymr2e.11 — consolidated failure routing for the nightly full-sweep lane.
#
# WHY: a nightly all-surface sweep (a11y / visual / cross-browser / tauri) that opened one
# alert per failing TEST would drown the tracker and the nightly would stop being believed
# (research/web-gui-test-program.md §6.2: "alert fatigue is how nightlies die"). This router
# instead maintains EXACTLY ONE consolidated GitHub issue per failing CATEGORY: it finds the
# open issue for a category by its labels, REFRESHES it (a comment) while the category keeps
# failing, CREATES it on the first failure, and CLOSES it (with a recovery comment) once the
# category goes green again. Every artifact it files carries the 🤖 SPARQ-agent self-id.
#
# It is driven by the per-category job RESULTS of .github/workflows/nightly-full-sweep.yml
# (`needs.<job>.result` ∈ success | failure | skipped | cancelled). It NEVER gates CI — the
# nightly workflow runs only on schedule/workflow_dispatch, so it is never a sibling check-run
# of the `ci-summary / gate` aggregator (see that workflow's advisory/skip semantics).
#
# Local reproduction (no API writes):
#   python3 scripts/nightly/route_sweep_failures.py --dry-run --repo o/r \
#       --run-url https://example/run/1 \
#       --result a11y=failure --result visual=success --result cross-browser=skipped
#
# The category slugs are a CLOSED SET (CATEGORIES below); an unknown slug is rejected so a
# workflow-side typo can never silently mis-route or spawn a stray issue class.
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass

# The consolidated failure categories. Keep in lockstep with the category jobs in
# .github/workflows/nightly-full-sweep.yml — one entry per job that routes.
CATEGORIES: dict[str, dict[str, str]] = {
    "a11y": {
        "title": "accessibility full-sweep (axe / WCAG 2.1 AA)",
        "scope": "the full-surface axe + surface-render sweep (site/e2e, all route inventory)",
        "hint": "Inspect the a11y-sweep job log + the uploaded Playwright report; a serious/"
        "critical axe violation is a hard fail, a rise in moderate/minor breaks the ratchet "
        "(bench/a11y-baseline.json).",
    },
    "visual": {
        "title": "visual-regression full-surface sweep",
        "scope": "the container-pinned toHaveScreenshot sweep (site/e2e/visual)",
        "hint": "Download the visual-regression-diffs artifact; if the change is intentional, "
        "refresh baselines in a PR via `npm run vr:update`.",
    },
    "cross-browser": {
        "title": "cross-browser P1 journeys (firefox + webkit)",
        "scope": "the P1 functional journey suite on firefox + webkit (per-PR runs chromium only)",
        "hint": "Reproduce with SPARQ_NIGHTLY_BROWSERS=1 npx playwright test <spec> "
        "--project firefox --project webkit; an engine-specific behavioural gap is the "
        "usual cause.",
    },
    "tauri": {
        "title": "tauri-driver GUI smoke",
        "scope": "the real-IPC tauri-driver launch/round-trip smoke (Linux row is authoritative)",
        "hint": "Inspect the tauri-e2e-artifacts (driver log + screenshot). Environment-coupled "
        "flake is expected occasionally; a persistent red across several nights is a real "
        "regression.",
    },
}

# Statuses that MEAN a category failed. Everything else (success / skipped / cancelled) does
# not open an issue; success additionally CLOSES a stale open one (recovery).
FAILING = {"failure", "timed_out", "action_required"}

BASE_LABEL = "nightly-sweep"
SELF_ID = "> 🤖 **SPARQ agent** — automated nightly full-sweep report (bead sq-ymr2e.11)."


@dataclass
class Result:
    category: str
    status: str


def sh(args: list[str], *, check: bool = True, capture: bool = True) -> subprocess.CompletedProcess:
    """Run a subprocess (gh CLI). Returns the CompletedProcess; raises on non-zero if check."""
    return subprocess.run(
        args,
        check=check,
        text=True,
        capture_output=capture,
    )


def cat_label(category: str) -> str:
    return f"sweep:{category}"


def ensure_labels(category: str, dry_run: bool) -> None:
    """Create the routing labels if absent (idempotent). Never fails the run on label ops."""
    for name, color, desc in (
        (BASE_LABEL, "5319e7", "Nightly full-sweep consolidated failure tracker (sq-ymr2e.11)"),
        (cat_label(category), "b60205", f"Nightly sweep category: {category}"),
    ):
        if dry_run:
            print(f"[dry-run] ensure label {name!r} (color={color})")
            continue
        # --force updates an existing label rather than erroring; a perms/transient error here
        # must not sink the routing run, so we swallow it and let the create/comment step below
        # surface any genuine problem.
        sh(["gh", "label", "create", name, "--color", color, "--description", desc, "--force"], check=False)


def find_open_issue(repo: str, category: str, dry_run: bool) -> int | None:
    """Return the number of the single open consolidated issue for this category, or None."""
    if dry_run:
        # Test-only hook: SWEEP_DRYRUN_EXISTING=<cat>:<num>[,<cat>:<num>...] lets a local dry-run
        # exercise the refresh / recovery-close branches without any GitHub API access.
        import os

        for spec in os.environ.get("SWEEP_DRYRUN_EXISTING", "").split(","):
            if ":" in spec:
                c, n = spec.split(":", 1)
                if c.strip() == category and n.strip().isdigit():
                    print(f"[dry-run] search open issue: label {BASE_LABEL} + {cat_label(category)} → #{n.strip()} (simulated)")
                    return int(n.strip())
        print(f"[dry-run] search open issue: label {BASE_LABEL} + {cat_label(category)} → none")
        return None
    cp = sh(
        [
            "gh", "issue", "list",
            "--repo", repo,
            "--state", "open",
            "--label", BASE_LABEL,
            "--label", cat_label(category),
            "--json", "number,title",
            "--limit", "10",
        ],
        check=False,
    )
    if cp.returncode != 0:
        # A missing label makes `gh issue list --label` error; treat as "no open issue".
        return None
    try:
        rows = json.loads(cp.stdout or "[]")
    except json.JSONDecodeError:
        return None
    return rows[0]["number"] if rows else None


def issue_title(category: str) -> str:
    meta = CATEGORIES[category]
    return f"🤖 nightly-sweep: {meta['title']} is failing"


def issue_body(category: str, run_url: str) -> str:
    meta = CATEGORIES[category]
    return "\n".join(
        [
            SELF_ID,
            "",
            f"The nightly full-sweep lane's **{category}** category is failing.",
            "",
            f"- **Covers:** {meta['scope']}",
            f"- **First-seen run:** {run_url}",
            "",
            "### What to check",
            meta["hint"],
            "",
            "---",
            "This is the single consolidated tracker for this category (research/"
            "web-gui-test-program.md §6.2). While the category keeps failing, the nightly lane "
            "**refreshes this issue with a comment** instead of opening new ones; it will "
            "**close automatically** once the category goes green again. Do not open siblings.",
        ]
    )


def refresh_comment(category: str, run_url: str) -> str:
    return "\n".join(
        [
            SELF_ID,
            "",
            f"Still failing on the latest nightly run: {run_url}",
        ]
    )


def recovery_comment(category: str, run_url: str) -> str:
    return "\n".join(
        [
            SELF_ID,
            "",
            f"✅ Recovered — the **{category}** category passed on {run_url}. Closing this "
            "consolidated tracker automatically. It will reopen (a fresh issue) if the category "
            "regresses again.",
        ]
    )


def create_issue(repo: str, category: str, run_url: str, dry_run: bool) -> None:
    title = issue_title(category)
    body = issue_body(category, run_url)
    if dry_run:
        print(f"[dry-run] CREATE issue: {title!r}")
        print(f"          labels: {BASE_LABEL}, {cat_label(category)}")
        print("          body:\n" + "\n".join("            " + ln for ln in body.splitlines()))
        return
    sh(
        [
            "gh", "issue", "create",
            "--repo", repo,
            "--title", title,
            "--body", body,
            "--label", BASE_LABEL,
            "--label", cat_label(category),
        ]
    )
    print(f"created consolidated issue for category {category!r}")


def comment_issue(repo: str, number: int, body: str, dry_run: bool) -> None:
    if dry_run:
        print(f"[dry-run] COMMENT on #{number}:\n" + "\n".join("            " + ln for ln in body.splitlines()))
        return
    sh(["gh", "issue", "comment", str(number), "--repo", repo, "--body", body])


def close_issue(repo: str, number: int, dry_run: bool) -> None:
    if dry_run:
        print(f"[dry-run] CLOSE #{number}")
        return
    sh(["gh", "issue", "close", str(number), "--repo", repo])


def route_one(repo: str, result: Result, run_url: str, dry_run: bool) -> str:
    category = result.category
    ensure_labels(category, dry_run)
    existing = find_open_issue(repo, category, dry_run)

    if result.status in FAILING:
        if existing is not None:
            comment_issue(repo, existing, refresh_comment(category, run_url), dry_run)
            return f"{category}: refreshed #{existing}"
        create_issue(repo, category, run_url, dry_run)
        return f"{category}: created new consolidated issue"

    if result.status == "success":
        if existing is not None:
            comment_issue(repo, existing, recovery_comment(category, run_url), dry_run)
            close_issue(repo, existing, dry_run)
            return f"{category}: recovered — closed #{existing}"
        return f"{category}: green (no open issue)"

    # skipped / cancelled / unknown terminal — do not file, do not close (ambiguous signal).
    return f"{category}: {result.status} — no action"


def parse_results(pairs: list[str]) -> list[Result]:
    out: list[Result] = []
    for p in pairs:
        if "=" not in p:
            sys.exit(f"error: --result expects category=status, got {p!r}")
        cat, status = p.split("=", 1)
        cat, status = cat.strip(), status.strip().lower()
        if cat not in CATEGORIES:
            sys.exit(f"error: unknown category {cat!r} (known: {', '.join(CATEGORIES)})")
        out.append(Result(cat, status))
    return out


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="Route nightly full-sweep failures to one consolidated issue per category.")
    ap.add_argument("--repo", required=True, help="owner/name")
    ap.add_argument("--run-url", required=True, help="URL of the triggering workflow run")
    ap.add_argument(
        "--result",
        action="append",
        default=[],
        metavar="CATEGORY=STATUS",
        help="repeatable; STATUS is the job result (success|failure|skipped|cancelled|...)",
    )
    ap.add_argument("--dry-run", action="store_true", help="print planned actions; make no API calls")
    args = ap.parse_args(argv)

    results = parse_results(args.result)
    if not results:
        print("no --result pairs supplied; nothing to route")
        return 0

    print(f"routing {len(results)} category result(s) for {args.repo} (dry_run={args.dry_run})")
    for r in results:
        outcome = route_one(args.repo, r, args.run_url, args.dry_run)
        print(f"  - {outcome}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
