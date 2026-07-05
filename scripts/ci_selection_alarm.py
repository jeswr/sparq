#!/usr/bin/env python3
# [OPUS-4.8] Nightly change-based-selection BUG ALARM (bead sq-va7at, epic
# sq-fmx4u). Design: research/change-based-test-selection.md §6.1.
# Authored by Opus 4.8 (running in place of the intended sonnet tier — Fable
# unavailable). Flag for re-review when Fable returns.
#
# WHAT THIS IS (and is NOT):
#   The correctness safeguard that closes the loop on change-based test selection
#   (scripts/ci_select.py). The nightly FULL-matrix backstop (ci.yml `schedule`
#   => mode=full) already RE-RUNS everything a PR may have selection-skipped; the
#   `selection-backstop` job asserts that full-run invariant. What was MISSING is
#   the correlation ALARM: when a scheduled nightly job FAILS and that same job's
#   crate was selection-SKIPPED in one or more PRs that landed since the last
#   GREEN nightly, that intersection is *prima-facie* evidence of a selection
#   soundness bug (the non-interference proof in §2 was wrong — a change that
#   "looked unaffected" actually broke the crate). This script computes that
#   intersection and files a self-identified SPARQ-agent GitHub issue naming the
#   offending job(s) + the suspect PRs.
#
#   It is a DETECTOR, never a gate: it runs AFTER a nightly completes (a
#   `workflow_run` trigger, see .github/workflows/selection-alarm.yml), files
#   issues, and never blocks a merge (design §6.1: "fail-open").
#
# TWO DESIGN INVARIANTS (both load-bearing):
#   1. FAIL-SAFE / FAIL-LOUD (opposite of ci_select.py's fail-CLOSED). An
#      alarm-INFRASTRUCTURE error (cannot list the failed jobs, cannot load cargo
#      metadata, cannot replay a landed commit's selection) must NEVER be swallowed
#      into a silent exit-0 — that would MASK a real selection bug. main() files
#      every finding it COULD compute and then exits NON-ZERO with a loud
#      `::error::` annotation, so the alarm workflow run goes RED and a human
#      notices the detector itself is broken. Redding this post-hoc workflow blocks
#      nothing (it is not a required check).
#   2. NON-SPAMMY (dedupe). One issue per distinct failure<->skip mapping, keyed by
#      a stable `<!-- selection-alarm-key: ... -->` marker; before filing we search
#      OPEN labelled issues for that key and skip if one already exists (the exact
#      flow-on idempotency mechanism). And the alarm fires ONLY when selection
#      actually skipped something relevant: an empty skip-set in the window => no
#      finding, because a nightly failure with nothing skipped cannot be a
#      selection bug.
#
# WHY GIT-REPLAY (not a stored artifact) for "record per-PR selection decisions"
# (design §6.1 item 1): scripts/ci_select.py is a PURE function of (diff, cargo
# metadata, ownership map). The per-PR selection decision is therefore
# RECONSTRUCTIBLE from git history — the most durable "record" there is (git
# history outlives any artifact-retention window). For each landed commit we
# replay `select(git-diff, metadata)` and take skipped = members - affected when
# mode=='selected'. Using the CURRENT (HEAD) metadata is conservative: any member
# added/removed in the window changed the root Cargo.toml, a §4.1 full-run trigger
# => that commit replays mode=full => skipped=∅ => it cannot be a suspect anyway.
#
# WHY GITHUB ISSUES, NOT `bd create` (mirrors flow-on.py): CI has NO `bd` dolt DB
# (it lives only in the orchestrator's gitignored checkout; hand-editing `.beads/`
# is forbidden). So this script emits GitHub issues; the orchestrator reconciles
# alarm issues into P1 beads out-of-band. The bead's "check bd before filing"
# dedupe is satisfied at that reconciliation layer; in CI, dedupe is by open issue.
#
# Usage:
#   ci_selection_alarm.py --run-id <id> --head-sha <sha>     # real (gh + git + cargo)
#   ci_selection_alarm.py --run-id <id> --head-sha <sha> --dry-run   # no gh writes
#   ci_selection_alarm.py --run-id <id> --head-sha <sha> \
#       --failed-jobs-file jobs.txt --metadata-file meta.json        # hermetic inputs (tests)
#
# Stdlib only (design §7 P1): argparse/json/os/re/subprocess/importlib + the
# shared scripts/ci_select.py (imported by path). Runs under the CI setup-python
# 3.12.

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# Labels every alarm issue carries (mirrors flow-on's `flow-on`+`auto`). The label
# names carry NO `advisory`/`informational` token — irrelevant here (issues are
# not check-runs) but kept clean for consistency.
BASE_LABELS = ["selection-alarm", "auto"]

# Idempotency marker embedded in each issue body (flow-on pattern).
KEY_PREFIX = "selection-alarm-key"

# Job conclusions that count as a "failed" nightly job.
FAILED_CONCLUSIONS = {"failure", "timed_out"}

# --- triage lanes (design §5.2 + the erratum) --------------------------------
# The change-based selector skips two lane CLASSES per PR:
#   (a) crate-NAMED lanes — feature-matrix legs `opt-in <crate> (...)`, per-crate
#       wasm/other jobs — where the crate name appears in the JOB name, so a
#       failed job maps PRECISELY to its crate by a delimited-token match; and
#   (b) the cross-crate bulk/heavy nextest SHARDS `test (load-aware shard <name>)`,
#       which are NARROWED per-PR by the nextest `filterset` (bulk) or a
#       single-crate skip guard (heavy) — their JOB names ("bulk 1/3",
#       "heavy-diskann") do NOT encode the failing crate.
# A class-(b) shard failure is a REAL selection-skip signal we must not mask, but
# we cannot name the crate from the job name alone. So a failed shard with no
# extractable crate goes to the FAIL-SAFE TRIAGE path: one issue naming the
# shard(s) + the whole skipped set in the window, for a human to narrow from the
# shard log. Everything ELSE that is unmappable (coverage/select/backstop/docs/
# supply-chain jobs — not per-crate test lanes) is NOT triage-eligible, so an
# infra-lane failure never mints a spurious selection alarm.
# scripts/tests/test_ci_selection_alarm.py pins this regex against the real
# ci.yml shard job name (`test (load-aware shard ${{ matrix.name }})`).
TRIAGE_LANE_RE = re.compile(r"^test \(load-aware shard ")


class AlarmError(Exception):
    """An alarm-INFRASTRUCTURE failure. main() surfaces it LOUD (non-zero exit)."""


# --------------------------------------------------------------------------- #
# Shared selector (imported by path — scripts/ is not an importable package)
# --------------------------------------------------------------------------- #
def _load_ci_select():
    spec = importlib.util.spec_from_file_location(
        "ci_select", REPO_ROOT / "scripts" / "ci_select.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    # Register before exec so ci_select's dataclass string annotations
    # (from __future__ import annotations) resolve via sys.modules.
    sys.modules[spec.name] = mod
    spec.loader.exec_module(mod)
    return mod


# --------------------------------------------------------------------------- #
# Data model
# --------------------------------------------------------------------------- #
@dataclass(frozen=True)
class WindowCommit:
    """A landed commit in the (last-green-nightly, HEAD] window and the crate set
    its PR selection-SKIPPED (empty when it ran mode=full)."""

    sha: str
    pr: int | None
    skipped: frozenset[str]


@dataclass
class Finding:
    """One distinct failure<->skip mapping to file as an issue."""

    kind: str  # "crate" (precise) | "triage" (unmappable shard)
    key: str  # stable dedup key (embedded in the body marker)
    crate: str | None  # precise: the offending crate; triage: None
    failed_jobs: list[str]  # failed job(s) implicating this mapping
    suspect_prs: list[WindowCommit]  # landed PRs whose skip-set implicates this


# --------------------------------------------------------------------------- #
# Pure correlation core (hermetic-testable — no gh / git / cargo)
# --------------------------------------------------------------------------- #
def job_crate_tokens(job_name: str, members: set[str]) -> set[str]:
    """Workspace-member crate names that appear as DELIMITED tokens in a job name.

    Delimited = bounded on each side by a non `[A-Za-z0-9_-]` char (or a string
    boundary), so `sparq-core` matches inside `opt-in sparq-core (mmap)` but NOT
    inside `sparq-core-foo`, and `sparq-vec` does not match inside
    `sparq-vectors`. Usually 0 or 1 member; a job that names several crates
    implicates all of them (conservative)."""
    found: set[str] = set()
    for m in members:
        if re.search(rf"(?<![A-Za-z0-9_-]){re.escape(m)}(?![A-Za-z0-9_-])", job_name):
            found.add(m)
    return found


def correlate(
    failed_jobs: list[str],
    window: list[WindowCommit],
    members: set[str],
) -> list[Finding]:
    """Intersect the failed nightly jobs with the per-PR selection skips.

    Precise: for a failed job that names crate C AND C was skipped by some landed
    PR => a `crate` finding for C (its suspect PRs = the PRs that skipped C).
    Triage: a failed TRIAGE-LANE shard with no extractable crate, when ANYTHING
    was skipped in the window => one `triage` finding naming those shards + every
    PR that skipped anything. Fires only where selection actually skipped
    something relevant (design non-spam property)."""
    # crate -> [commits that skipped it]
    skip_to_prs: dict[str, list[WindowCommit]] = {}
    for wc in window:
        for c in wc.skipped:
            skip_to_prs.setdefault(c, []).append(wc)
    skipped_union = set(skip_to_prs)

    findings: list[Finding] = []

    # --- precise per-crate findings ---
    crate_jobs: dict[str, list[str]] = {}
    unmappable_triage_jobs: list[str] = []
    for job in failed_jobs:
        tokens = job_crate_tokens(job, members)
        implicated = tokens & skipped_union
        if implicated:
            for c in implicated:
                crate_jobs.setdefault(c, []).append(job)
        elif not tokens and TRIAGE_LANE_RE.match(job):
            # A narrowed shard with no crate in its name AND not resolved to a
            # skipped crate => fail-safe triage candidate.
            unmappable_triage_jobs.append(job)
        # else: a crate-named job whose crate was NOT skipped (its own tests ran
        # per-PR — not a selection bug), or a non-test infra lane => ignored.

    for crate in sorted(crate_jobs):
        findings.append(
            Finding(
                kind="crate",
                key=f"crate:{crate}",
                crate=crate,
                failed_jobs=sorted(set(crate_jobs[crate])),
                suspect_prs=skip_to_prs[crate],
            )
        )

    # --- fail-safe triage finding ---
    if unmappable_triage_jobs and skipped_union:
        jobs_sorted = sorted(set(unmappable_triage_jobs))
        # Suspect PRs = every landed PR that skipped ANY crate (we cannot narrow
        # which crate the generic shard's failure came from).
        suspects = [wc for wc in window if wc.skipped]
        findings.append(
            Finding(
                kind="triage",
                key="triage:" + "|".join(jobs_sorted),
                crate=None,
                failed_jobs=jobs_sorted,
                suspect_prs=suspects,
            )
        )
    return findings


# --------------------------------------------------------------------------- #
# Issue rendering
# --------------------------------------------------------------------------- #
def _pr_ref(wc: WindowCommit) -> str:
    return f"#{wc.pr}" if wc.pr is not None else f"`{wc.sha[:12]}`"


def key_marker(key: str) -> str:
    return f"<!-- {KEY_PREFIX}: {key} -->"


def render_issue(f: Finding, run_id: str, head_sha: str, repo: str | None) -> tuple[str, str]:
    """Return (title, body) for a finding."""
    run_url = (
        f"https://github.com/{repo}/actions/runs/{run_id}" if repo else f"run {run_id}"
    )
    if f.kind == "crate":
        title = (
            f"[selection-alarm] crate `{f.crate}` failed the nightly full-matrix "
            f"but was selection-skipped in landed PR(s)"
        )
        lead = (
            f"The nightly FULL-matrix backstop failed a job for crate "
            f"**`{f.crate}`**, and that crate was **selection-skipped** in one or "
            f"more PRs that landed since the last green nightly. Per the skip "
            f"invariant (research/change-based-test-selection.md §2) a test is "
            f"skipped only if *provably* no change could affect it — so this "
            f"intersection is prima-facie evidence the non-interference proof was "
            f"wrong for `{f.crate}`."
        )
    else:
        title = (
            "[selection-alarm] nightly full-matrix shard failure needs crate-triage "
            "(change-based selection active in window)"
        )
        lead = (
            "The nightly FULL-matrix backstop failed a cross-crate/narrowed test "
            "shard whose job name does not encode the failing crate, and "
            "change-based selection NARROWED that shard (dropped some crates) in "
            "PRs that landed since the last green nightly. The failing crate cannot "
            "be named from the job alone — read the shard log below to identify it, "
            "then check whether it was among the skipped crates listed."
        )

    prs = "\n".join(
        f"- {_pr_ref(wc)} (`{wc.sha[:12]}`)"
        + (f" — skipped: {', '.join(sorted(wc.skipped))}" if f.kind == "triage" else "")
        for wc in f.suspect_prs
    )
    jobs = "\n".join(f"- `{j}`" for j in f.failed_jobs)
    body = (
        f"{lead}\n\n"
        f"**Failed nightly job(s):**\n{jobs}\n\n"
        f"**Suspect landed PR(s)** (selection-skipped since last green nightly):\n"
        f"{prs}\n\n"
        f"**Nightly run:** {run_url} (head `{head_sha[:12]}`)\n\n"
        f"This is a DETECTOR signal, not a proof: a nightly failure can also be a "
        f"flake or an unrelated regression. Verify against the run log before "
        f"concluding a selection bug — but if it IS one, the selector's ownership "
        f"map / reverse-closure (scripts/ci_select.py, ci/path-ownership.toml) let "
        f"`{f.crate or 'the crate'}` be skipped when it should not have been; fix "
        f"that and add a golden test.\n\n"
        f"{key_marker(f.key)}\n\n"
        f"> \U0001f916 SPARQ agent — auto-filed by the change-based-selection "
        f"nightly alarm (scripts/ci_selection_alarm.py, bead sq-va7at). Reconcile "
        f"into a **P1** bead. [OPUS-4.8]"
    )
    return title, body


# --------------------------------------------------------------------------- #
# gh / git / cargo I/O (real, non-hermetic)
# --------------------------------------------------------------------------- #
def _run(cmd: list[str], cwd: str | None = None, check: bool = True) -> str:
    try:
        proc = subprocess.run(
            cmd, cwd=cwd, capture_output=True, text=True, check=check, timeout=300
        )
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError) as exc:
        stderr = getattr(exc, "stderr", "") or ""
        raise AlarmError(f"command failed: {' '.join(cmd)}: {exc}\n{stderr}") from exc
    return proc.stdout


def gather_failed_jobs(run_id: str, repo: str) -> list[str]:
    """Names of the nightly run's jobs whose conclusion is a failure (design: the
    failed-job set to correlate). Uses `gh api --paginate` over the run's jobs."""
    out = _run(
        [
            "gh", "api", "--paginate",
            f"repos/{repo}/actions/runs/{run_id}/jobs",
            "--jq", ".jobs[] | [.name, (.conclusion // \"\")] | @tsv",
        ]
    )
    failed: list[str] = []
    for line in out.splitlines():
        if not line.strip():
            continue
        name, _, conclusion = line.partition("\t")
        if conclusion.strip() in FAILED_CONCLUSIONS:
            failed.append(name)
    # The workflow only fires on a failure-concluded run, so an empty failed-job
    # set is always anomalous (jobs-API filter=latest re-run race, shape surprise,
    # etc.).  Distinguish this from "no correlation found" to prevent silent exit 0
    # masking a real failure.
    if not failed:
        raise AlarmError(
            "gather_failed_jobs: empty failed-job set on a failure-concluded run "
            f"(run_id={run_id}); jobs-API shape surprise or filter=latest re-run "
            "race — cannot correlate, refusing to exit 0"
        )
    return failed


def last_green_nightly_sha(repo: str, workflow_file: str, event: str) -> str | None:
    """head_sha of the most recent SUCCESSFUL run of `workflow_file` for `event`
    (the last green nightly). None if there is no prior green run."""
    out = _run(
        [
            "gh", "api",
            f"repos/{repo}/actions/workflows/{workflow_file}/runs"
            f"?event={event}&status=success&per_page=1",
            "--jq", ".workflow_runs[0].head_sha // \"\"",
        ],
        check=True,
    )
    sha = out.strip()
    return sha or None


_PR_RE = re.compile(r"\(#(\d+)\)\s*$")


def window_commits(
    base_sha: str | None, head_sha: str, lookback_hours: int, repo_root: str
) -> list[tuple[str, int | None]]:
    """(sha, pr_number) for each landed commit in the window, newest first.

    Window = (last-green-nightly, HEAD] when base_sha is known; otherwise a
    time-bounded fallback (`--since`) so a cold start (no prior green nightly) is
    handled without a spurious loud failure."""
    if base_sha:
        rng = [f"{base_sha}..{head_sha}"]
    else:
        rng = [head_sha, f"--since={lookback_hours} hours ago"]
    out = _run(
        ["git", "log", "--no-merges", "--format=%H%x00%s", *rng],
        cwd=repo_root,
    )
    commits: list[tuple[str, int | None]] = []
    for line in out.splitlines():
        if not line.strip():
            continue
        sha, _, subject = line.partition("\x00")
        m = _PR_RE.search(subject)
        commits.append((sha.strip(), int(m.group(1)) if m else None))
    return commits


def commit_changed_paths(sha: str, repo_root: str) -> list[str]:
    """Paths a landed commit changed = `git diff --no-renames <sha>^ <sha>` (the
    PR's net diff on a squash-merged linear main; matches ci_select's --no-renames
    conservatism)."""
    out = _run(
        ["git", "diff", "--name-only", "--no-renames", f"{sha}^", sha],
        cwd=repo_root,
    )
    return [ln for ln in out.splitlines() if ln.strip()]


def build_window(
    commits: list[tuple[str, int | None]], meta: dict, map_entries: list[dict],
    ci_select, repo_root: str,
) -> tuple[list[WindowCommit], set[str], list[str]]:
    """Replay the selection decision for each landed commit. Returns
    (window, members, replay_errors). A replay error is collected (not swallowed)
    so main() can fail LOUD while still filing the findings it could compute."""
    ws = ci_select.parse_workspace(meta)
    members = set(ws.members)
    window: list[WindowCommit] = []
    replay_errors: list[str] = []
    for sha, pr in commits:
        try:
            changed = commit_changed_paths(sha, repo_root)
            sel = ci_select.select(changed, meta, map_entries)
            skipped = (
                frozenset(members - set(sel.affected))
                if sel.mode == "selected"
                else frozenset()
            )
        except Exception as exc:  # noqa: BLE001 — collect, don't mask (fail-loud later)
            replay_errors.append(f"{sha[:12]}: {exc}")
            continue
        window.append(WindowCommit(sha=sha, pr=pr, skipped=skipped))
    return window, members, replay_errors


# --------------------------------------------------------------------------- #
# Issue minting (mirror flow-on's CI guard + idempotency + label upsert)
# --------------------------------------------------------------------------- #
CI_ENV_VARS = ("GITHUB_ACTIONS", "CI")
ALLOW_LOCAL_ENV_VAR = "SELECTION_ALARM_ALLOW_LOCAL"


def _is_truthy(val: str | None) -> bool:
    return val is not None and val.strip().lower() in {"1", "true", "yes", "on"}


def require_ci(env: dict[str, str] | None = None) -> None:
    e = os.environ if env is None else env
    if any(_is_truthy(e.get(v)) for v in CI_ENV_VARS) or _is_truthy(e.get(ALLOW_LOCAL_ENV_VAR)):
        return
    raise AlarmError(
        "refusing to file GitHub issues outside CI (no GITHUB_ACTIONS / CI env "
        f"set). Use --dry-run to preview, or set {ALLOW_LOCAL_ENV_VAR}=1 to force."
    )


_ENSURED_LABELS: set[str] = set()


def ensure_label(label: str) -> None:
    if label in _ENSURED_LABELS:
        return
    try:
        _run(["gh", "label", "create", label, "--force"], check=True)
    except AlarmError:
        pass  # best-effort upsert; create_issue surfaces a genuine failure
    _ENSURED_LABELS.add(label)


def open_issue_exists(key: str, repo: str) -> bool:
    # Idempotency: list ALL open selection-alarm issues and EXACT-match the body
    # marker. We deliberately do NOT pass `--search` — a triage key contains
    # punctuation (parens/slashes) that gh's search tokeniser handles unreliably,
    # which could MISS an existing issue and break dedupe (mint a duplicate). The
    # deduped label keeps the open set tiny, so listing 100 is cheap.
    marker = key_marker(key)
    out = _run(
        [
            "gh", "issue", "list", "--repo", repo, "--state", "open",
            "--label", "selection-alarm", "--json", "number,body", "--limit", "100",
        ],
        check=False,
    )
    try:
        issues = json.loads(out or "[]")
    except json.JSONDecodeError:
        return False
    return any(marker in (i.get("body") or "") for i in issues)


def create_issue(title: str, body: str, labels: list[str], repo: str) -> str:
    args = ["gh", "issue", "create", "--repo", repo, "--title", title, "--body", body]
    for lbl in labels:
        ensure_label(lbl)
        args += ["--label", lbl]
    return _run(args).strip()


# --------------------------------------------------------------------------- #
# CLI
# --------------------------------------------------------------------------- #
def _parse_args(argv: list[str] | None) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description="Nightly change-based-selection bug alarm (design §6.1)."
    )
    p.add_argument("--run-id", required=True, help="the failed nightly CI run id")
    p.add_argument("--head-sha", required=True, help="the nightly run's head SHA")
    p.add_argument("--repo", default=os.environ.get("GITHUB_REPOSITORY"),
                   help="owner/name (default $GITHUB_REPOSITORY)")
    p.add_argument("--event", default="schedule",
                   help="the nightly trigger event to look up the last green run by")
    p.add_argument("--workflow-file", default="ci.yml",
                   help="workflow file whose last green run bounds the window")
    p.add_argument("--lookback-hours", type=int, default=25,
                   help="cold-start fallback window when no prior green nightly exists")
    p.add_argument("--repo-root", help="repo root (default: git toplevel)")
    p.add_argument("--dry-run", action="store_true", help="print findings; no gh writes")
    # Hermetic inputs (tests / offline):
    p.add_argument("--failed-jobs-file", help="hermetic: one failed job name per line")
    p.add_argument("--metadata-file", help="hermetic: cargo metadata JSON snapshot")
    return p.parse_args(argv)


def _resolve_repo_root(explicit: str | None) -> str:
    if explicit:
        return explicit
    try:
        return _run(["git", "rev-parse", "--show-toplevel"]).strip()
    except AlarmError:
        return str(REPO_ROOT)


def run_alarm(args: argparse.Namespace) -> tuple[list[Finding], list[str], str]:
    """Gather inputs, replay the window, correlate. Returns
    (findings, replay_errors, repo). Raises AlarmError on a TOTAL blocker."""
    ci_select = _load_ci_select()
    repo = args.repo or ""
    repo_root = _resolve_repo_root(args.repo_root)

    # Failed jobs — hermetic file or gh.
    if args.failed_jobs_file:
        failed_jobs = [
            ln.strip() for ln in Path(args.failed_jobs_file).read_text().splitlines()
            if ln.strip()
        ]
    else:
        if not repo:
            raise AlarmError("--repo (or $GITHUB_REPOSITORY) required for the gh path")
        failed_jobs = gather_failed_jobs(args.run_id, repo)
    # The workflow only fires on a failure-concluded run, so an empty failed-job
    # set (from either the file path or the gh path) is always anomalous — an
    # empty correlation must be distinguishable from anomalous-empty to prevent a
    # silent exit 0 masking a real failure.
    if not failed_jobs:
        raise AlarmError(
            "run_alarm: empty failed-jobs set; workflow fires on failure-concluded "
            "runs only, so this is anomalous (jobs-API shape surprise, re-run race, "
            "or empty --failed-jobs-file). Refusing to exit 0."
        )

    # cargo metadata (member set + reverse closure for the replay).
    meta = ci_select.load_metadata(args.metadata_file, repo_root)
    map_file = os.path.join(repo_root, "ci", "path-ownership.toml")
    map_entries = ci_select.load_ownership_map(
        map_file if os.path.exists(map_file) else None
    )

    # Window of landed commits since the last green nightly (or time fallback).
    base = None if args.failed_jobs_file else last_green_nightly_sha(
        repo, args.workflow_file, args.event
    )
    if base is None and not args.failed_jobs_file:
        print("::notice::selection-alarm: no prior green nightly found; using "
              f"{args.lookback_hours}h cold-start fallback window")
    commits = window_commits(base, args.head_sha, args.lookback_hours, repo_root)
    window, members, replay_errors = build_window(
        commits, meta, map_entries, ci_select, repo_root
    )

    findings = correlate(failed_jobs, window, members)
    return findings, replay_errors, repo


def main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv)
    try:
        findings, replay_errors, repo = run_alarm(args)
    except AlarmError as exc:
        # FAIL-LOUD (invariant 1): a total blocker must red the run, never exit 0.
        print(f"::error::selection-alarm infrastructure error: {exc}", file=sys.stderr)
        return 1

    if not findings:
        print("selection-alarm: no nightly failure correlates with a selection "
              "skip in the window — no alarm.")
    else:
        if not args.dry_run:
            require_ci_error = None
            try:
                require_ci()
            except AlarmError as exc:
                require_ci_error = exc
            if require_ci_error is not None:
                print(f"::error::{require_ci_error}", file=sys.stderr)
                return 1
        filed = skipped = 0
        for f in findings:
            title, body = render_issue(f, args.run_id, args.head_sha, repo or None)
            if args.dry_run:
                print(f"[dry-run] WOULD file (kind={f.kind}, key={f.key}):")
                print(f"          title: {title}")
                print(f"          jobs : {', '.join(f.failed_jobs)}")
                print(f"          PRs  : {', '.join(_pr_ref(wc) for wc in f.suspect_prs)}")
                continue
            if open_issue_exists(f.key, repo):
                print(f"selection-alarm: skip (open issue exists) key={f.key}")
                skipped += 1
                continue
            url = create_issue(title, body, list(dict.fromkeys(BASE_LABELS)), repo)
            print(f"selection-alarm: filed {url} (kind={f.kind}, key={f.key})")
            filed += 1
        if not args.dry_run:
            print(f"selection-alarm: done — {filed} filed, {skipped} already open.")
            summary = os.environ.get("GITHUB_STEP_SUMMARY")
            if summary:
                try:
                    with open(summary, "a") as fh:
                        fh.write(f"### selection-alarm\n\n- filed: {filed}\n"
                                 f"- skipped (already open): {skipped}\n")
                except OSError:
                    pass

    # FAIL-LOUD (invariant 1): findings we COULD compute are filed above; if any
    # landed commit could not be replayed, red the run so the gap is investigated.
    if replay_errors:
        print("::error::selection-alarm: could not replay selection for "
              f"{len(replay_errors)} landed commit(s) — the correlation window is "
              "INCOMPLETE and a real selection bug may be unreported:",
              file=sys.stderr)
        for e in replay_errors:
            print(f"::error::  {e}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
