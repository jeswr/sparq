#!/usr/bin/env python3
# [OPUS-4.8] PR-backlog hygiene groomer (scheduled, NON-gating).
# Authored by Opus 4.8 (1M context) — Fable unavailable; flag for re-review when Fable returns.
#
# WHAT THIS IS
# ------------
# The deterministic policy core of .github/workflows/pr-backlog.yml — a DAILY cron groomer
# that keeps the open-PR backlog tidy. It does NOT gate any PR (it runs on a schedule, never
# on a PR's head commit) and it makes NO model calls: it is pure, testable policy over the
# GitHub PR/issue state.
#
# POLICY (maintainer-directed)
# ----------------------------
#  1. DEPENDABOT PRs (author login == app/dependabot):
#       - If the PR's authoritative required-checks aggregator (the `gate` / ci-summary
#         check) has CONCLUDED in FAILURE -> CLOSE the PR with a polite "> 🤖 SPARQ agent"
#         comment AND file a DEDUPED issue `deps: bump <pkgs> to <vers>` (dedupe marker
#         `<!-- dep-upgrade:<pkg>@<ver> -->`) with labels role:impl / priority:P2 /
#         area:deps / kind:deps. The issue body links the closed PR, quotes the failing
#         check name(s), and REMINDS the implementing agent that a new/bumped dep needs a
#         cargo-vet attestation (supply-chain/config.toml) or the gate fails.
#       - If the gate is still IN-PROGRESS / QUEUED (not concluded) -> SKIP this run (a
#         later run will catch it once the gate concludes). Never close on a non-terminal
#         gate.
#       - GREEN dependabot PRs -> untouched (report only).
#  2. STALE PRs (open, non-dependabot, no update in > STALE_DAYS days):
#       - Add label `backlog:stale` + file a DEDUPED per-PR issue
#         `pr-backlog: resolve stale PR #<n> — <title>` (marker `<!-- pr-backlog:<n> -->`)
#         with labels role:impl / priority:P3 and a best-effort `area:<crate>` derived from
#         the PR's changed files (fall back to no area label). The issue body tells the
#         resolving agent to rebase/finish/split or close-with-rationale, and to check
#         research/ + AGENTS.md for standing deprecations FIRST (several stale PRs are
#         zkSPARQL spec docs a 2026-07 decision deprecated — honor that, don't blindly
#         finish them).
#  3. EXCLUSIONS (report-only, never touched):
#       - the release-plz PR (author app/github-actions, title starts `chore: release`)
#       - any PR labeled needs:user or trust:*
#       - any DRAFT PR whose head branch matches ^sparq-agent/ (the in-flight review loop)
#       - PR #EXCLUDE_PR (in review right now — passed via --exclude-pr)
#
# DESIGN: the policy is a PURE FUNCTION plan(prs, existing_issue_markers, now) -> [Action].
# An Action is a small dataclass carrying the exact `gh` argv it maps to. `live` mode runs
# the argv through a real gh; `--self-test` runs plan() over fixtures and asserts the argv,
# with gh STUBBED so no live mutation can happen from a test. ZERO human @mentions anywhere.
#
# USAGE
#   scripts/pr-backlog.py --repo owner/repo --exclude-pr 2431      # live groom
#   scripts/pr-backlog.py --repo owner/repo --exclude-pr 2431 --dry-run  # print plan, no gh
#   scripts/pr-backlog.py --self-test                             # hermetic; gh stubbed
#
# Exit 0 on success; non-zero only on a real error or a failed self-test.
import argparse
from contextlib import redirect_stderr
import io
import json
import re
import subprocess
import sys
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone

import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
# [OPUS-5] #1135: the shared Release-PR predicate (branch/author/title, never a label).
# NOT fail-soft: the stub treats every PR as a release PR, so a missing module makes this
# script groom nothing rather than groom the Release PR.
try:
    import release_pr_guard
except ImportError:  # pragma: no cover - see scripts/tests/test_release_publish_guard.py

    class release_pr_guard:  # type: ignore[no-redef]
        @staticmethod
        def arm_block_reason(**_kwargs) -> str:
            return (
                "release-pr-guard: scripts/release_pr_guard.py is not importable — "
                "treating every PR as untouchable (fail-closed, #1135)"
            )


PROG = "pr-backlog"

# A PR with no update in more than this many days is "stale".
STALE_DAYS = 7

DEPENDABOT_LOGIN = "dependabot"          # gh reports app authors as login "app/dependabot"
GITHUB_ACTIONS_LOGIN = "github-actions"  # release-plz PR author
SPARQ_AGENT_BRANCH_RE = re.compile(r"^sparq-agent/")

SELF_ID = "> 🤖 SPARQ agent"

# [OPUS-5] sparq-org/sparq#4985. `gh pr list --limit N` truncates SILENTLY: no error, no
# warning, just a short list. gh exposes no "there was more" flag, so SATURATION is the
# only signal available — hence a cap set far above the live population plus the
# assertion below, which turns a silent truncation into a visible one.
OPEN_PR_PAGE_CAP = 1000


def log(msg: str) -> None:
    print(f"[{PROG}] {msg}", file=sys.stderr)


def assert_not_truncated(rows, cap, what):
    """Refuse to act on a `gh --limit` page that came back SATURATED.

    A saturated page is indistinguishable from a complete one — gh returns exactly `cap`
    rows either way — so the groomer would silently stop seeing the TAIL of the backlog
    and degrade invisibly. Raising instead makes the day that happens loud.

    Kept local rather than lifted into a shared sibling module: each of these scripts is
    a standalone entry point run by its own workflow, and a new cross-script import has to
    be kept in step with that workflow's checkout manifest (the #3776 failure mode) — a
    coupling not worth taking on for an eight-line assertion.
    """
    if len(rows) >= cap:
        raise SystemExit(
            f"[{PROG}] ERROR: the {what} fetch returned {len(rows)} rows at its "
            f"--limit {cap} cap. gh TRUNCATES SILENTLY, so this snapshot is probably "
            f"partial and acting on it would groom only part of the backlog with no "
            f"signal. Raise the cap deliberately (OPEN_PR_PAGE_CAP)."
        )
    return rows


# --------------------------------------------------------------------------------------
# Author-login normalisation. gh returns bot authors either as {"is_bot": true,
# "login": "app/dependabot"} (search API) or plain "dependabot" — normalise to the bare
# handle so the policy is source-agnostic.
# --------------------------------------------------------------------------------------
def author_handle(login: str) -> str:
    return (login or "").split("/")[-1].lower()


def is_dependabot(pr: dict) -> bool:
    return author_handle(pr.get("author_login", "")) == DEPENDABOT_LOGIN


def is_release_plz(pr: dict) -> bool:
    """[OPUS-5] #1135: one shared predicate for every arming/merging/grooming path.

    WAS: `author == github-actions AND title startswith "chore: release"` — a conjunction
    that misses the Release PR whenever EITHER signal shifts. release_pr_guard keys on
    branch OR author OR title and fails CLOSED on an unknown head branch. Widening is the
    safe direction: this script's only response to a match is to leave the PR untouched.
    """
    return (
        release_pr_guard.arm_block_reason(
            head_ref=pr.get("head_ref"),
            author_login=pr.get("author_login"),
            title=pr.get("title"),
        )
        is not None
    )


def has_excluded_label(pr: dict) -> bool:
    for lbl in pr.get("labels", []):
        low = lbl.lower()
        if low == "needs:user" or low.startswith("trust:"):
            return True
    return False


def is_sparq_agent_draft(pr: dict) -> bool:
    return bool(pr.get("is_draft")) and bool(
        SPARQ_AGENT_BRANCH_RE.match(pr.get("head_ref", "") or "")
    )


# --------------------------------------------------------------------------------------
# Gate conclusion. The authoritative required check is `gate` (from ci-summary). We locate
# it among the PR's status-check rollup entries by name and read its terminal conclusion.
# A gate that is present but NOT terminal (queued/in_progress/pending/expected) => "pending"
# => the dependabot policy SKIPS this run. A missing gate is treated as pending too (never
# close on an unobserved gate).
# --------------------------------------------------------------------------------------
_TERMINAL_FAIL = {"failure", "cancelled", "timed_out", "action_required", "startup_failure", "stale"}
_TERMINAL_OK = {"success", "neutral", "skipped"}


def gate_status(pr: dict):
    """Return ('failure'|'success'|'pending', [failing_check_names]).

    Judges the ci-summary `gate` check as authoritative. If for some reason the gate is not
    present, we conservatively report 'pending' (never act on an unobserved required check).
    """
    checks = pr.get("checks", [])
    gate = None
    for c in checks:
        if (c.get("name") or "").strip().lower() == "gate":
            gate = c
            break
    if gate is None:
        return "pending", []
    concl = (gate.get("conclusion") or "").lower()
    status = (gate.get("status") or "").lower()
    if status and status not in ("completed",):
        # queued / in_progress / pending / waiting / expected -> not terminal
        return "pending", []
    if concl in _TERMINAL_FAIL:
        # Collect the names of the actually-failing sibling checks (for the issue body),
        # falling back to the gate itself if no per-leg detail is available.
        failing = [
            (c.get("name") or "").strip()
            for c in checks
            if (c.get("conclusion") or "").lower() in _TERMINAL_FAIL
        ]
        failing = [f for f in failing if f] or ["gate (ci-summary)"]
        return "failure", failing
    if concl in _TERMINAL_OK:
        return "success", []
    # Unknown/empty terminal conclusion -> be conservative, treat as pending.
    return "pending", []


# --------------------------------------------------------------------------------------
# Dependabot title parsing -> (pkgs, vers, issue_title, dedupe_marker).
# Dependabot titles look like:
#   "build(deps): bump serde from 1.0.203 to 1.0.204"
#   "chore(deps): bump the tokio group with 2 updates"
#   "Bump actions/checkout from 4 to 5"
# We extract a best-effort <pkg> and <ver> for the title + dedupe marker; if we cannot,
# we fall back to a stable slug of the whole title so dedupe still works.
# --------------------------------------------------------------------------------------
_BUMP_RE = re.compile(
    r"bump\s+(?P<pkg>[^\s]+(?:\s+group)?)\s+from\s+(?P<from>\S+)\s+to\s+(?P<to>\S+)",
    re.IGNORECASE,
)


def parse_dependabot(pr: dict):
    """Return (issue_title, marker_key). marker_key is stable across re-runs for dedupe."""
    title = pr.get("title", "") or ""
    m = _BUMP_RE.search(title)
    if m:
        pkg = m.group("pkg").strip()
        ver = m.group("to").strip()
        issue_title = f"deps: bump {pkg} to {ver}"
        marker_key = f"{pkg}@{ver}"
    else:
        # Group/multi-update or unusual title: keep the human-readable remainder + a stable slug.
        remainder = re.sub(r"^[a-z]+\([^)]*\):\s*", "", title, flags=re.IGNORECASE).strip()
        remainder = re.sub(r"(?i)^bump\s+", "", remainder).strip() or title.strip()
        issue_title = f"deps: bump {remainder}"
        slug = re.sub(r"[^a-z0-9]+", "-", remainder.lower()).strip("-") or "unknown"
        marker_key = f"{slug}@bulk"
    return issue_title, marker_key


def dep_marker(marker_key: str) -> str:
    return f"<!-- dep-upgrade:{marker_key} -->"


def stale_marker(pr_number: int) -> str:
    return f"<!-- pr-backlog:{pr_number} -->"


# --------------------------------------------------------------------------------------
# Best-effort area:<crate> from a PR's changed files. A path crates/<name>/... maps to
# area:<name>; the most-frequently-touched crate wins. Returns None if no crate path.
# --------------------------------------------------------------------------------------
_CRATE_RE = re.compile(r"^crates/([A-Za-z0-9_-]+)/")


def area_label_for(files) -> str:
    counts = {}
    for f in files or []:
        m = _CRATE_RE.match(f or "")
        if m:
            counts[m.group(1)] = counts.get(m.group(1), 0) + 1
    if not counts:
        return None
    # Deterministic: highest count, then lexicographic for tie-break.
    best = sorted(counts.items(), key=lambda kv: (-kv[1], kv[0]))[0][0]
    return f"area:{best}"


# --------------------------------------------------------------------------------------
# The action model. Each Action carries a `kind` (for the step-summary table) + the exact
# gh argv it maps to. `run(gh)` invokes gh; in self-test gh is a recording stub.
# --------------------------------------------------------------------------------------
@dataclass
class Action:
    kind: str                       # close | comment | issue | label | skip | untouched | exclude
    pr: int                         # PR the action concerns (0 for pure issue creation)
    reason: str                     # short human reason for the summary table
    argv: list = field(default_factory=list)  # gh argv (empty for report-only rows)

    def run(self, gh) -> None:
        if self.argv:
            gh(self.argv)


def _label_action(repo: str, name: str, color: str, desc: str) -> list:
    # `gh label create --force` is idempotent (create-or-update). Keeps labels present.
    return ["label", "create", name, "--repo", repo, "--color", color,
            "--description", desc, "--force"]


def dep_issue_body(pr: dict, failing_checks) -> str:
    checks_md = "\n".join(f"  - `{c}`" for c in failing_checks) or "  - `gate (ci-summary)`"
    marker = dep_marker(parse_dependabot(pr)[1])
    return (
        f"{SELF_ID} — dependabot PR #{pr['number']} was auto-closed by the pr-backlog "
        f"groomer because its required checks concluded in failure; the upgrade is now "
        f"tracked here instead of a red, stuck PR.\n\n"
        f"**Original PR:** #{pr['number']} — {pr.get('title','')}\n\n"
        f"**Failing check(s):**\n{checks_md}\n\n"
        f"A resolving agent should apply the same dependency bump on a normal branch/PR. "
        f"**Reminder:** a NEW or BUMPED dependency requires a `cargo-vet` attestation in "
        f"`supply-chain/config.toml` (and, where relevant, a `deny.toml` review) — without "
        f"it the supply-chain leg of the `gate` fails, which is why the dependabot PR could "
        f"not merge on its own.\n\n"
        f"{marker}\n"
    )


def stale_issue_body(pr: dict, area) -> str:
    area_note = f" (area: `{area}`)" if area else ""
    return (
        f"{SELF_ID} — this PR has had no activity for more than {STALE_DAYS} days and was "
        f"flagged `backlog:stale` by the pr-backlog groomer{area_note}.\n\n"
        f"**Stale PR:** #{pr['number']} — {pr.get('title','')}\n\n"
        f"A resolving agent should: rebase onto `main` and finish it, split it into a "
        f"landable slice, or close it with a written rationale. **Before doing any of "
        f"that, check `research/` and `AGENTS.md` for a standing deprecation that "
        f"supersedes this PR** — several long-stale PRs are zkSPARQL spec docs that a "
        f"2026-07 decision deprecated; honor that decision (close-with-rationale) rather "
        f"than blindly finishing the PR.\n\n"
        f"{stale_marker(pr['number'])}\n"
    )


# --------------------------------------------------------------------------------------
# THE POLICY (pure). Given the PR list, the set of dedupe markers already present in open
# issues, the repo, the excluded PR number, and `now`, produce the ordered Action list.
# --------------------------------------------------------------------------------------
def plan(prs, existing_markers, repo, exclude_pr, now):
    actions = []
    existing_markers = set(existing_markers)

    for pr in prs:
        num = pr["number"]

        # --- exclusions (report-only) ---
        if exclude_pr is not None and num == exclude_pr:
            actions.append(Action("exclude", num, "PR in review (excluded by flag)"))
            continue
        if is_release_plz(pr):
            actions.append(Action("exclude", num, "release-plz PR"))
            continue
        if has_excluded_label(pr):
            actions.append(Action("exclude", num, "needs:user / trust:* label"))
            continue
        if is_sparq_agent_draft(pr):
            actions.append(Action("exclude", num, "sparq-agent/ review-loop draft"))
            continue

        # --- dependabot policy ---
        if is_dependabot(pr):
            state, failing = gate_status(pr)
            if state == "failure":
                # (a) file the deduped issue (before closing, so the close comment can cite it).
                issue_title, marker_key = parse_dependabot(pr)
                marker = dep_marker(marker_key)
                if marker in existing_markers:
                    actions.append(Action("skip", num,
                                          f"dep issue already filed ({marker_key})"))
                else:
                    actions.append(Action(
                        "issue", num, f"file dep issue ({marker_key})",
                        ["issue", "create", "--repo", repo, "--title", issue_title,
                         "--body", dep_issue_body(pr, failing),
                         "--label", "role:impl", "--label", "priority:P2",
                         "--label", "area:deps", "--label", "kind:deps"]))
                    existing_markers.add(marker)  # so a 2nd same-dep PR in this run dedupes
                # (b) close the PR with a polite self-ID comment.
                close_comment = (
                    f"{SELF_ID} — closing this dependabot PR because its required checks "
                    f"concluded in failure ({', '.join(failing)}). The upgrade is now "
                    f"tracked as an issue titled `{issue_title}` so it can be applied on a "
                    f"normal branch with the required cargo-vet attestation. Thanks, "
                    f"dependabot — this is not a rejection of the bump, just a re-route."
                )
                actions.append(Action(
                    "close", num, f"close failing dependabot PR ({', '.join(failing)})",
                    ["pr", "close", str(num), "--repo", repo, "--comment", close_comment]))
            elif state == "success":
                actions.append(Action("untouched", num, "green dependabot PR (left as-is)"))
            else:  # pending
                actions.append(Action("skip", num, "dependabot gate not yet concluded"))
            continue

        # --- stale policy (non-dependabot open PRs) ---
        updated = pr.get("updated_at")
        if updated is None:
            actions.append(Action("skip", num, "no updated_at timestamp"))
            continue
        age = now - _parse_iso(updated)
        if age <= timedelta(days=STALE_DAYS):
            actions.append(Action("untouched", num, f"fresh (updated {age.days}d ago)"))
            continue

        # stale: label + deduped issue.
        marker = stale_marker(num)
        already_labeled = any(l.lower() == "backlog:stale" for l in pr.get("labels", []))
        if not already_labeled:
            actions.append(Action(
                "label", num, "mark backlog:stale",
                ["pr", "edit", str(num), "--repo", repo, "--add-label", "backlog:stale"]))
        if marker in existing_markers:
            actions.append(Action("skip", num, f"stale issue already filed (#{num})"))
            continue
        area = area_label_for(pr.get("files"))
        issue_title = f"pr-backlog: resolve stale PR #{num} — {pr.get('title','')}"
        argv = ["issue", "create", "--repo", repo, "--title", issue_title,
                "--body", stale_issue_body(pr, area),
                "--label", "role:impl", "--label", "priority:P3"]
        if area:
            argv += ["--label", area]
        actions.append(Action("issue", num, f"file stale issue (#{num})", argv))
        existing_markers.add(marker)

    return actions


def _parse_iso(s: str) -> datetime:
    # gh emits RFC3339 with a trailing Z. Normalise to an aware UTC datetime.
    s = s.strip()
    if s.endswith("Z"):
        s = s[:-1] + "+00:00"
    dt = datetime.fromisoformat(s)
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    return dt


# --------------------------------------------------------------------------------------
# LIVE data collection (real gh). The self-test exercises it only through a hermetic stub.
# --------------------------------------------------------------------------------------
def _gh_json(argv):
    out = subprocess.check_output(["gh"] + argv, text=True)
    return json.loads(out)


def _normalise_checks(status_check_rollup):
    checks = []
    for c in status_check_rollup or []:
        # CheckRun entries have name/status/conclusion; StatusContext entries have
        # context/state. Normalise both into {name,status,conclusion}.
        if "name" in c:
            checks.append({"name": c.get("name"),
                           "status": c.get("status"),
                           "conclusion": c.get("conclusion")})
        else:
            state = (c.get("state") or "").lower()
            concl = {"success": "success", "failure": "failure",
                     "error": "failure", "pending": ""}.get(state, state)
            status = "completed" if state in ("success", "failure", "error") else "pending"
            checks.append({"name": c.get("context"),
                           "status": status, "conclusion": concl})
    return checks


def collect_prs(repo, gh_json=_gh_json):
    """Fetch the light open-PR snapshot used to decide which details are needed."""
    prs = assert_not_truncated(gh_json([
        "pr", "list", "--repo", repo, "--state", "open",
        "--limit", str(OPEN_PR_PAGE_CAP),
        "--json",
        "number,title,author,isDraft,headRefName,labels,updatedAt",
    ]), OPEN_PR_PAGE_CAP, "open PRs")
    norm = []
    for p in prs:
        norm.append({
            "number": p["number"],
            "title": p.get("title", ""),
            "author_login": (p.get("author") or {}).get("login", ""),
            "is_draft": bool(p.get("isDraft")),
            "head_ref": p.get("headRefName", ""),
            "labels": [l.get("name", "") for l in (p.get("labels") or [])],
            "updated_at": p.get("updatedAt"),
            "files": [],
            "checks": [],
        })
    return norm


def enrich_prs(prs, existing_markers, repo, exclude_pr, now, gh_json=_gh_json):
    """Fetch heavy fields per PR, only when the policy will inspect them.

    A failed detail query leaves that field empty, records the PR-local error, and does not
    prevent later PRs from being evaluated. The caller returns the error count as its exit
    status after finishing the run.
    """
    errors = 0
    existing_markers = set(existing_markers)
    fetch_errors = (subprocess.CalledProcessError, json.JSONDecodeError, OSError)

    for pr in prs:
        num = pr["number"]

        # [GPT-5.6] Match plan()'s exclusion order before issuing any detail query.
        if exclude_pr is not None and num == exclude_pr:
            continue
        if is_release_plz(pr) or has_excluded_label(pr) or is_sparq_agent_draft(pr):
            continue

        if is_dependabot(pr):
            try:
                detail = gh_json([
                    "pr", "view", str(num), "--repo", repo,
                    "--json", "statusCheckRollup",
                ])
                pr["checks"] = _normalise_checks(detail.get("statusCheckRollup"))
            except fetch_errors as error:
                log(f"ERROR: status-check fetch failed for PR #{num}: {error} — continuing.")
                errors += 1
            continue

        updated = pr.get("updated_at")
        if updated is None or now - _parse_iso(updated) <= timedelta(days=STALE_DAYS):
            continue
        if stale_marker(num) in existing_markers:
            continue

        try:
            detail = gh_json([
                "pr", "view", str(num), "--repo", repo, "--json", "files",
            ])
            pr["files"] = [f.get("path", "") for f in (detail.get("files") or [])]
        except fetch_errors as error:
            log(f"ERROR: changed-files fetch failed for PR #{num}: {error} — continuing.")
            errors += 1

    return errors


def collect_existing_markers(repo, gh_json=_gh_json):
    """Every dedupe marker present in the body of an OPEN issue.

    [OPUS-5] sparq-org/sparq#4985. This was `gh issue list --limit 500`, and unlike the
    call sites that issue tabulated this one was ALREADY past its cap: #4985 counted 1707
    open issues on the live repo, `--limit` truncates SILENTLY (newest-first, no error, no
    warning), so every marker on an older issue was invisible. A marker this function
    fails to see is a dedupe MISS, and a dedupe miss re-files an issue the groomer already
    filed — the non-spam invariant failing OPEN, quietly.

    The markers are spread over two different label sets (`area:deps`/`kind:deps` for
    dep-upgrade issues, `backlog:stale` for stale-PR issues), so there is no single label
    scope to narrow by: the fetch is the whole open set, exhausted via `gh api
    --paginate` (same pattern as scripts/ready-issues.py `_fetch`). PRs are dropped —
    the issues endpoint returns both, and `gh issue list` did not return PRs.
    """
    pages = gh_json([
        "api", "--paginate", "--slurp",
        f"repos/{repo}/issues?state=open&per_page=100",
    ])
    issues = [i for page in pages if isinstance(page, list)
              for i in page if isinstance(i, dict) and "pull_request" not in i]
    markers = set()
    marker_re = re.compile(r"<!-- (?:dep-upgrade|pr-backlog):[^>]+ -->")
    for it in issues:
        for m in marker_re.findall(it.get("body") or ""):
            markers.add(m.strip())
    return markers


def ensure_labels(gh, repo):
    """Idempotently create the labels the policy uses (create-or-update via --force)."""
    for argv in (
        _label_action(repo, "backlog:stale", "fbca04", "Open PR stale >7d (pr-backlog groomer)"),
        _label_action(repo, "area:deps", "0e8a16", "Dependency-management work"),
        _label_action(repo, "kind:deps", "0e8a16", "Dependency bump / upgrade"),
    ):
        gh(argv)


# --------------------------------------------------------------------------------------
# Step-summary table (GitHub Actions job summary, or stdout locally).
# --------------------------------------------------------------------------------------
def render_summary(actions) -> str:
    lines = [
        "## pr-backlog groomer",
        "",
        "| PR | action | reason |",
        "| --- | --- | --- |",
    ]
    for a in actions:
        pr = f"#{a.pr}" if a.pr else "—"
        lines.append(f"| {pr} | {a.kind} | {a.reason} |")
    lines.append("")
    return "\n".join(lines)


# --------------------------------------------------------------------------------------
# CLI
# --------------------------------------------------------------------------------------
def real_gh(argv) -> None:
    subprocess.run(["gh"] + argv, check=True)


def main() -> int:
    ap = argparse.ArgumentParser(description="PR-backlog hygiene groomer (pure policy).")
    ap.add_argument("--repo", help="owner/repo (required unless --self-test).")
    ap.add_argument("--exclude-pr", type=int, default=None,
                    help="A PR number to leave untouched (in review).")
    ap.add_argument("--dry-run", action="store_true",
                    help="Fetch state + print the plan/summary, run NO gh mutations.")
    ap.add_argument("--self-test", action="store_true",
                    help="Run the hermetic self-test (gh stubbed) and exit.")
    ap.add_argument("--summary-file", default=None,
                    help="Write the step-summary table here (e.g. $GITHUB_STEP_SUMMARY).")
    args = ap.parse_args()

    if args.self_test:
        return self_test()

    if not args.repo:
        raise SystemExit(f"[{PROG}] ERROR: --repo is required (unless --self-test).")

    prs = collect_prs(args.repo)
    markers = collect_existing_markers(args.repo)
    now = datetime.now(timezone.utc)
    errors = enrich_prs(prs, markers, args.repo, args.exclude_pr, now)
    actions = plan(prs, markers, args.repo, args.exclude_pr, now)

    summary = render_summary(actions)
    if args.summary_file:
        with open(args.summary_file, "a", encoding="utf-8") as f:
            f.write(summary + "\n")
    print(summary)

    if args.dry_run:
        log("--dry-run: no gh mutations performed.")
        return errors

    gh = real_gh
    # Ensure the labels exist ONLY if we are about to use them.
    if any(a.kind in ("issue", "label", "close") for a in actions):
        ensure_labels(gh, args.repo)
    for a in actions:
        try:
            a.run(gh)
        except subprocess.CalledProcessError as e:
            log(f"WARNING: gh action failed for PR #{a.pr} ({a.kind}): {e} — continuing.")
    log(f"applied {sum(1 for a in actions if a.argv)} mutation(s).")
    return errors


# --------------------------------------------------------------------------------------
# Hermetic self-test. gh is a recording stub — NO live mutation is possible. Asserts the
# emitted argv for each fixture PR + the re-run dedupe.
# --------------------------------------------------------------------------------------
def self_test() -> int:
    fails = 0

    def check(label, cond):
        nonlocal fails
        if cond:
            print(f"  ok   {label}")
        else:
            print(f"  FAIL {label}")
            fails += 1

    log("running --self-test (hermetic; gh STUBBED — no live mutation possible)")

    now = datetime(2026, 7, 17, 12, 0, 0, tzinfo=timezone.utc)
    repo = "sparq-org/sparq"

    def days_ago(n):
        return (now - timedelta(days=n)).strftime("%Y-%m-%dT%H:%M:%SZ")

    # [GPT-5.6] Collection regression fixture: the list stays light at backlog scale, heavy
    # fields are fetched only for actionable PRs, and one detail failure cannot stop the rest.
    large_raw = [
        {
            "number": 2000 + offset,
            "title": f"feat: fixture {offset}",
            "author": {"login": "someone"},
            "isDraft": False,
            "headRefName": f"fixture/{offset}",
            "labels": [],
            "updatedAt": days_ago(1),
        }
        for offset in range(76)
    ]
    large_raw[0].update({
        "title": "build(deps): bump serde from 1.0.203 to 1.0.204",
        "author": {"login": "app/dependabot"},
        "headRefName": "dependabot/cargo/serde-1.0.204",
    })
    large_raw[1]["updatedAt"] = days_ago(8)   # detail fetch fails
    large_raw[2]["updatedAt"] = days_ago(9)   # proves the run continues
    large_raw[3]["updatedAt"] = days_ago(10)  # deduped: no files needed

    collection_calls = []

    def collection_gh(argv):
        collection_calls.append(list(argv))
        if argv[:2] == ["pr", "list"]:
            return large_raw
        if argv[:3] == ["pr", "view", "2000"]:
            return {"statusCheckRollup": [
                {"name": "gate", "status": "completed", "conclusion": "success"},
            ]}
        if argv[:3] == ["pr", "view", "2001"]:
            raise subprocess.CalledProcessError(1, ["gh"] + argv)
        if argv[:3] == ["pr", "view", "2002"]:
            return {"files": [{"path": "crates/sparq-engine/src/lib.rs"}]}
        raise AssertionError(f"unexpected collection gh call: {argv}")

    collected = collect_prs(repo, gh_json=collection_gh)
    list_call = collection_calls[0]
    list_fields = set(list_call[list_call.index("--json") + 1].split(","))
    check("large-list fixture collects all 76 open PRs", len(collected) == 76)
    check("large-list fixture uses one light list query", len(collection_calls) == 1)
    check("mutation tripwire: list query excludes files/statusCheckRollup",
          list_fields.isdisjoint({"files", "statusCheckRollup"}))

    # --- #4985: a SATURATED `gh pr list` page must be REFUSED, not groomed ---------------
    # gh returns exactly `--limit` rows whether or not more existed, so a saturated page is
    # indistinguishable from a complete one. Without this the groomer silently stops seeing
    # the tail of the backlog. `assert_not_truncated` is exercised THROUGH collect_prs (not
    # called directly) so deleting the call site — not just the function — goes red.
    def saturated_gh(argv):
        return [dict(large_raw[0], number=3000 + i) for i in range(OPEN_PR_PAGE_CAP)]

    try:
        collect_prs(repo, gh_json=saturated_gh)
        check("a SATURATED open-PR page is refused (#4985)", False)
    except SystemExit as e:
        check("a SATURATED open-PR page is refused (#4985)", "TRUNCATES SILENTLY" in str(e))

    def one_short_gh(argv):
        return [dict(large_raw[0], number=3000 + i) for i in range(OPEN_PR_PAGE_CAP - 1)]

    check("a page one row UNDER the cap is accepted (the guard is not just 'always raise')",
          len(collect_prs(repo, gh_json=one_short_gh)) == OPEN_PR_PAGE_CAP - 1)

    # --- #4985: marker dedupe must READ EVERY open issue, not the newest page ------------
    # This call site was ALREADY truncating in production (`--limit 500` against a
    # >500-issue open backlog), so a marker on an older issue was invisible and the groomer
    # re-filed an issue it had already filed. The stub emulates BOTH gh shapes faithfully —
    # `gh api --paginate --slurp` returns a LIST OF PAGES, `gh issue list --limit N` returns
    # only the newest N — so reverting the fetch fails on the MISSED MARKER rather than on
    # an unrecognised command line.
    marker_corpus = [{"number": n, "body": "no marker here"} for n in range(1, 1201)]
    marker_corpus[4]["body"] = "prose\n<!-- pr-backlog:103 -->\ntail"     # issue #5: oldest end
    marker_corpus.append({"number": 9999, "pull_request": {"url": "…"},
                          "body": "<!-- dep-upgrade:serde@1.0.204 -->"})  # a PR, not an issue

    def marker_gh(argv):
        if argv[:2] == ["api", "--paginate"]:
            return [marker_corpus[i:i + 100] for i in range(0, len(marker_corpus), 100)]
        if argv[:2] == ["issue", "list"]:
            cap = int(argv[argv.index("--limit") + 1])
            return sorted(marker_corpus, key=lambda r: -r["number"])[:cap]
        raise AssertionError(f"unexpected marker gh call: {argv}")

    found_markers = collect_existing_markers(repo, gh_json=marker_gh)
    check("a marker on the 5th-oldest of 1200 open issues is still SEEN (#4985)",
          "<!-- pr-backlog:103 -->" in found_markers)
    check("a PULL REQUEST body carrying a marker is not counted as an existing issue",
          "<!-- dep-upgrade:serde@1.0.204 -->" not in found_markers)

    collection_stderr = io.StringIO()
    with redirect_stderr(collection_stderr):
        collection_errors = enrich_prs(
            collected,
            existing_markers={stale_marker(2003)},
            repo=repo,
            exclude_pr=None,
            now=now,
            gh_json=collection_gh,
        )
    detail_calls = collection_calls[1:]
    check("lazy detail fetches target only the three PRs that need heavy fields",
          [(call[2], call[-1]) for call in detail_calls] == [
              ("2000", "statusCheckRollup"), ("2001", "files"), ("2002", "files"),
          ])
    check("per-PR fetch failure contributes one exit error", collection_errors == 1)
    check("per-PR fetch failure is recorded loudly",
          "ERROR:" in collection_stderr.getvalue()
          and "PR #2001" in collection_stderr.getvalue()
          and "continuing" in collection_stderr.getvalue())
    check("per-PR fetch failure isolates and later enrichment continues",
          collected[2]["files"] == ["crates/sparq-engine/src/lib.rs"])
    check("dependabot status rollup is normalised by its lazy fetch",
          gate_status(collected[0])[0] == "success")

    fixtures = [
        # 1. failing dependabot PR -> close + issue
        {"number": 100, "title": "build(deps): bump serde from 1.0.203 to 1.0.204",
         "author_login": "app/dependabot", "is_draft": False, "head_ref": "dependabot/cargo/serde-1.0.204",
         "labels": [], "updated_at": days_ago(1), "files": [],
         "checks": [{"name": "gate", "status": "completed", "conclusion": "failure"},
                    {"name": "supply-chain / cargo-vet", "status": "completed", "conclusion": "failure"}]},
        # 2. green dependabot PR -> untouched
        {"number": 101, "title": "build(deps): bump tokio from 1.38.0 to 1.39.0",
         "author_login": "app/dependabot", "is_draft": False, "head_ref": "dependabot/cargo/tokio-1.39.0",
         "labels": [], "updated_at": days_ago(1), "files": [],
         "checks": [{"name": "gate", "status": "completed", "conclusion": "success"}]},
        # 3. in-progress dependabot gate -> skip (not concluded)
        {"number": 102, "title": "build(deps): bump anyhow from 1.0.80 to 1.0.86",
         "author_login": "app/dependabot", "is_draft": False, "head_ref": "dependabot/cargo/anyhow-1.0.86",
         "labels": [], "updated_at": days_ago(1), "files": [],
         "checks": [{"name": "gate", "status": "in_progress", "conclusion": None}]},
        # 4. 8-day-old non-dependabot PR -> stale label + issue, with area from files
        {"number": 103, "title": "feat(engine): experimental thing",
         "author_login": "someone", "is_draft": False, "head_ref": "feat/exp",
         "labels": [], "updated_at": days_ago(8),
         "files": ["crates/sparq-engine/src/lib.rs", "crates/sparq-engine/src/x.rs", "README.md"],
         "checks": []},
        # 5. release-plz PR -> excluded
        {"number": 104, "title": "chore: release v0.5.0",
         "author_login": "app/github-actions", "is_draft": False, "head_ref": "release-plz-2026",
         "labels": [], "updated_at": days_ago(10), "files": [], "checks": []},
        # 6. needs:user PR -> excluded
        {"number": 105, "title": "feat: gated on maintainer",
         "author_login": "someone", "is_draft": False, "head_ref": "feat/gated",
         "labels": ["needs:user"], "updated_at": days_ago(30), "files": [], "checks": []},
        # 7. sparq-agent/ DRAFT -> excluded
        {"number": 106, "title": "wip: review-loop draft",
         "author_login": "someone", "is_draft": True, "head_ref": "sparq-agent/review-106",
         "labels": [], "updated_at": days_ago(30), "files": [], "checks": []},
        # 8. explicitly excluded PR (--exclude-pr)
        {"number": 2431, "title": "chore(ci): the one in review",
         "author_login": "someone", "is_draft": False, "head_ref": "noise-reduction",
         "labels": [], "updated_at": days_ago(30), "files": [], "checks": []},
        # 9. fresh non-dependabot PR (3 days) -> untouched
        {"number": 107, "title": "feat: recent work",
         "author_login": "someone", "is_draft": False, "head_ref": "feat/recent",
         "labels": [], "updated_at": days_ago(3), "files": ["crates/sparq-core/src/a.rs"],
         "checks": []},
        # 10. trust:* labeled PR -> excluded
        {"number": 108, "title": "feat: sensitive",
         "author_login": "someone", "is_draft": False, "head_ref": "feat/trust",
         "labels": ["trust:untrusted"], "updated_at": days_ago(30), "files": [], "checks": []},
        # 11. stale PR with NO crate files -> stale issue with NO area label
        {"number": 109, "title": "docs: stale zkSPARQL spec doc",
         "author_login": "someone", "is_draft": False, "head_ref": "docs/zksparql",
         "labels": [], "updated_at": days_ago(20), "files": ["research/zksparql.md"], "checks": []},
    ]

    actions = plan(fixtures, existing_markers=set(), repo=repo, exclude_pr=2431, now=now)
    by_pr = {}
    for a in actions:
        by_pr.setdefault(a.pr, []).append(a)

    def argv_of(pr, kind):
        for a in by_pr.get(pr, []):
            if a.kind == kind:
                return a.argv
        return None

    # 1. failing dependabot -> issue argv + close argv
    dep_issue = argv_of(100, "issue")
    check("dep#100 emits an issue-create argv", dep_issue is not None and dep_issue[:2] == ["issue", "create"])
    check("dep#100 issue title parsed pkg@ver", "deps: bump serde to 1.0.204" in (dep_issue or []))
    check("dep#100 issue labeled role:impl/priority:P2/area:deps/kind:deps",
          all(l in (dep_issue or []) for l in ["role:impl", "priority:P2", "area:deps", "kind:deps"]))
    body100 = (dep_issue or [None] * 100)[dep_issue.index("--body") + 1] if dep_issue else ""
    check("dep#100 body reminds about cargo-vet/supply-chain",
          "cargo-vet" in body100 and "supply-chain/config.toml" in body100)
    check("dep#100 body carries dedupe marker",
          "<!-- dep-upgrade:serde@1.0.204 -->" in body100)
    dep_close = argv_of(100, "close")
    check("dep#100 emits a pr-close argv", dep_close is not None and dep_close[:2] == ["pr", "close"])
    check("dep#100 close comment quotes a failing check",
          any("supply-chain / cargo-vet" in x for x in (dep_close or [])))

    # 2. green dependabot -> untouched, NO mutating argv
    check("dep#101 untouched", any(a.kind == "untouched" for a in by_pr.get(101, [])))
    check("dep#101 emits NO mutating argv", all(not a.argv for a in by_pr.get(101, [])))

    # 3. in-progress dependabot -> skip, NO argv
    check("dep#102 skipped (gate not concluded)", any(a.kind == "skip" for a in by_pr.get(102, [])))
    check("dep#102 emits NO argv", all(not a.argv for a in by_pr.get(102, [])))

    # 4. stale PR -> label + issue with area:sparq-engine
    stale_label = argv_of(103, "label")
    check("stale#103 emits add-label backlog:stale",
          stale_label is not None and "backlog:stale" in stale_label and "--add-label" in stale_label)
    stale_issue = argv_of(103, "issue")
    check("stale#103 emits an issue-create argv", stale_issue is not None and stale_issue[:2] == ["issue", "create"])
    check("stale#103 issue titled per-PR", any("resolve stale PR #103" in x for x in (stale_issue or [])))
    check("stale#103 issue labeled role:impl/priority:P3", all(l in (stale_issue or []) for l in ["role:impl", "priority:P3"]))
    check("stale#103 area derived to area:sparq-engine", "area:sparq-engine" in (stale_issue or []))
    body103 = stale_issue[stale_issue.index("--body") + 1] if stale_issue else ""
    check("stale#103 body carries per-PR marker", "<!-- pr-backlog:103 -->" in body103)
    check("stale#103 body warns re research/AGENTS deprecations",
          "research/" in body103 and "AGENTS.md" in body103 and "deprecat" in body103.lower())

    # 5-8,10. exclusions -> exactly one 'exclude' action, no argv
    for pr, why in [(104, "release-plz"), (105, "needs:user"), (106, "sparq-agent draft"),
                    (2431, "--exclude-pr"), (108, "trust:*")]:
        acts = by_pr.get(pr, [])
        check(f"#{pr} excluded ({why})", len(acts) == 1 and acts[0].kind == "exclude" and not acts[0].argv)

    # 9. fresh PR -> untouched
    check("#107 fresh untouched", any(a.kind == "untouched" for a in by_pr.get(107, [])) and
          all(not a.argv for a in by_pr.get(107, [])))

    # 11. stale PR, no crate files -> issue with NO area label
    stale_issue_109 = argv_of(109, "issue")
    check("stale#109 emits an issue argv", stale_issue_109 is not None)
    check("stale#109 has NO area: label", not any(x.startswith("area:") for x in (stale_issue_109 or [])))

    # --- NON-VACUITY: at least one close, one dep-issue, one stale-issue, one label, one exclude ---
    kinds = [a.kind for a in actions]
    check("non-vacuous: >=1 close emitted", kinds.count("close") >= 1)
    check("non-vacuous: >=1 issue emitted", kinds.count("issue") >= 2)  # dep + stale
    check("non-vacuous: >=1 label emitted", kinds.count("label") >= 1)
    check("non-vacuous: >=1 exclude emitted", kinds.count("exclude") >= 1)

    # --- ZERO @mentions anywhere in any emitted argv ---
    mention_re = re.compile(r"(?<![\w`/])@[A-Za-z0-9](?:[A-Za-z0-9-]*[A-Za-z0-9])?")
    for a in actions:
        for tok in a.argv:
            hits = mention_re.findall(tok)
            if hits:
                check(f"NO @mention in #{a.pr} {a.kind} argv (found {hits})", False)
    check("no @mention scan completed", True)

    # --- DEDUPE: re-run with the markers now present emits NO duplicate issue argv ---
    existing = set()
    for a in actions:
        if a.kind == "issue":
            body = a.argv[a.argv.index("--body") + 1]
            m = re.search(r"<!-- (?:dep-upgrade|pr-backlog):[^>]+ -->", body)
            if m:
                existing.add(m.group(0))
    check("first run produced >=2 dedupe markers", len(existing) >= 2)
    rerun = plan(fixtures, existing_markers=existing, repo=repo, exclude_pr=2431, now=now)
    rerun_issue_kinds = [a for a in rerun if a.kind == "issue"]
    check("re-run emits ZERO duplicate issue argv", len(rerun_issue_kinds) == 0)
    # The stale LABEL is still emitted on re-run because the fixture PR isn't labeled yet
    # (idempotent gh --add-label); but the dep-issue re-run should now be a 'skip'.
    check("re-run turns dep#100 issue into a skip",
          any(a.kind == "skip" for a in rerun if a.pr == 100))
    check("re-run turns stale#103 issue into a skip",
          any(a.kind == "skip" for a in rerun if a.pr == 103))

    # --- gh STUB never touches the network: run() through a recording stub ---
    recorded = []
    def gh_stub(argv):
        recorded.append(argv)
    for a in actions:
        a.run(gh_stub)
    check("gh stub recorded exactly the mutating argv count",
          len(recorded) == sum(1 for a in actions if a.argv))
    check("gh stub recorded >0 argv (non-vacuous run)", len(recorded) > 0)

    # --- render_summary produces a table with a row per action ---
    summary = render_summary(actions)
    check("summary is a markdown table", summary.startswith("## pr-backlog") and "| PR | action | reason |" in summary)

    print()
    if fails == 0:
        log("self-test PASSED")
        return 0
    log(f"self-test FAILED ({fails} check(s))")
    return 1


if __name__ == "__main__":
    sys.exit(main())
