#!/usr/bin/env python3
# [OPUS-5] The `__global__` STOCK alarm — sparq-org/sparq#5127, the third bullet of
# #4582 that the attribution fix deliberately left out (it is a MONITORING mechanism,
# not a deriver change). 🤖 SPARQ agent.
#
# WHAT THIS IS. The fleet scheduler partitions work by `area:<name>`: an in-flight PR
# RESERVES its areas and a ready issue defers while any of its areas is reserved. A PR
# whose whole unit of work declares NO area cannot be attributed to a partition, and the
# registry's CLAIM leg folds that case into the SERIALIZING `__global__` partition
# (`areas |= issue_areas or {GLOBAL_PACKAGE}` — the fail-closed step recorded, with its
# measured consequence, in `scripts/ready-issues.py::unit_reservations`). One such PR
# therefore defers EVERY ready issue whatever crate it names. That is the fleet-wide
# stall this alarm exists to notice.
#
# WHY THE DERIVER'S OWN RUN STATUS CANNOT EXPRESS IT (#4582 bullet 3, restated by #5127,
# which records that `pr-area-label` reported `success` on every run in which #4578 sat
# unattributed). Emitting NO label IS `scripts/pr-area-labels.py`'s fail-closed contract,
# so those runs were honest. The alarm-worthy condition is not "a run failed", it is "the STOCK
# of unattributable active PRs is non-empty and PERSISTENT", which is a property of the
# whole open-PR population across TIME. No per-run status can carry it, and nothing in
# `scripts/pr-area-labels.py` is the right place to compute it. Hence a periodic sweep.
#
# THE THREE THINGS #5127 ASKS FOR, AND WHERE EACH LIVES HERE:
#   1. A SWEEP over OPEN, non-draft PRs with no `area:*` label, CROSS-CHECKED against
#      the provenance-linked source issue's areas so an issue-rescued PR is not alarmed
#      -> `classify_pr` / `sweep`. The linkage is NOT re-implemented: `source_issue_links`
#      is imported from `scripts/ready-issues.py`, where its own docstring calls it the
#      single definition of PR->issue linkage in the readiness engine, so this detector
#      and that engine can never disagree about what "linked" means. A second copy of that
#      rule is precisely the two-legs-disagree drift `ready-issues.py` documents at length.
#   2. PERSISTENCE ACROSS TICKS, so a PR unattributed ONCE is not alarmed and one
#      unattributed twice running is -> `next_streaks` over a tiny JSON state file the
#      workflow carries between runs in an Actions cache. A PR that gets attributed (or
#      closes, or is parked) drops out of the map, so the streak is CONSECUTIVE ticks by
#      construction rather than a cumulative total.
#   3. AN EXISTING SURFACE TO FIRE ON -> `.github/workflows/global-stock-alarm.yml`,
#      built to the same shape as ci-latency-alarm.yml / formal-alarm.yml /
#      review-alarm.yml: scheduled on main, `issues: write` and nothing else, one deduped
#      issue keyed by a body marker, and a hermetic `--self-test` as the first step.
#
# WHAT IT COUNTS, STATED AS A FLOOR RATHER THAN AS THE STOCK. Two deliberate
# under-counts, both in the quiet direction, both stated so nobody reads the number as
# exact:
#   * DRAFTS ARE EXCLUDED (#5127 says "OPEN, non-draft"). `ready-issues.py` treats every
#     active open PR as in flight, drafts included, and only releases a draft's areas
#     against the registry's per-row inertness PROOF (`is_provably_inert`), which is
#     stamped by the snapshot producer and is not readable from a plain `gh api pulls`
#     listing. A non-inert unattributed draft is real stock this alarm does not count.
#   * A CLOSED linked source issue does not rescue a PR here, mirroring
#     `_own_reservation`, which returns the empty set for any row whose `state` is not
#     OPEN — but when a closed linked issue DOES declare areas that fact is carried into
#     the finding, because
#     "your source issue is closed and took its areas with it" is a different first move
#     from "nothing ever declared an area".
#
# TWO DESIGN INVARIANTS (mirrors ci_selection_alarm.py / formal_lane_alarm.py):
#   1. FAIL-LOUD. An alarm-INFRASTRUCTURE error — gh failure, unparseable response,
#      corrupt state, or a COLD state read on a lane that has already completed a run
#      (i.e. the persistence layer silently stopped working) — exits 2 with an
#      `::error::` annotation. A detector whose memory has been quietly disconnected
#      would report an empty stock forever, which is the one outcome it must never do.
#   2. NON-SPAMMY. ONE open issue, keyed by a stable
#      `<!-- global-stock-alarm-key: <repo> -->` body marker searched over open
#      `global-stock-alarm`-labelled issues; an existing one is PATCHed in place rather
#      than duplicated, because the alarming set changes tick to tick.
#
# NOT A GATE. This runs from a schedule on main and files issues; it blocks no merge.
# That is true because its job is DECLARED in `.github/advisory-registry.json`, which
# #3773 made the ONLY thing that can make a check-run non-gating — NOT because of where
# its check-run lands (ci-summary.yml also runs on `push: branches: [main]`, so a
# scheduled run on main lands its check-run on exactly the head SHA the push-triggered
# gate polls; see sq-huwr8 and scripts/tests/test_alarm_lanes_non_gating.py).
#
# Usage:
#   global_stock_alarm.py --state-file s.json            # real (gh; env GITHUB_REPOSITORY)
#   global_stock_alarm.py --state-file s.json --dry-run  # print findings; no gh writes
#   global_stock_alarm.py --snapshot-file snap.json \
#       --state-file s.json --repo o/r --dry-run         # hermetic (tests)
#   global_stock_alarm.py --self-test                    # hermetic fixtures
#
# Stdlib only + the `gh` CLI (present on every GitHub runner).

from __future__ import annotations

import argparse
import contextlib
import datetime as dt
import importlib.util
import io
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

BASE_LABELS = ["global-stock-alarm", "auto"]
KEY_PREFIX = "global-stock-alarm-key"

# How many CONSECUTIVE sweeps a PR must be unattributed before it alarms. 2 is the
# floor #5127 states ("unattributed once is not alarmed, twice running is"); it is a
# parameter because the sweep cadence and the fleet's dispatch cadence are set
# independently and only the OPERATOR knows how long a transient is allowed to be.
DEFAULT_MIN_TICKS = 2

STATE_SCHEMA = 1

# The states a PR can exit the sweep in. Reported as a census on every run, alarming or
# not, so a run that finds nothing still says what it looked at — an empty finding list
# over an empty or mis-parsed population is the failure this makes visible.
CENSUS_STATES = ("draft", "parked", "attributed-by-pr", "rescued-by-issue", "unattributed")


class AlarmError(Exception):
    """Alarm-INFRASTRUCTURE failure — surfaced LOUD (non-zero exit)."""


# --------------------------------------------------------------------------- #
# The readiness engine's own rules, imported rather than restated
# --------------------------------------------------------------------------- #
def _load_ready_engine():
    """Load `scripts/ready-issues.py` (hyphenated, so not importable by name).

    Same loader shape as `scripts/dispatch-plan.py`. This module consumes FOUR pure
    predicates from it — `source_issue_links`, `declared_packages`, `labels_of`,
    `is_parked` — and re-derives none of them.
    """
    path = REPO_ROOT / "scripts" / "ready-issues.py"
    spec = importlib.util.spec_from_file_location("ready_issues", path)
    if spec is None or spec.loader is None:
        raise AlarmError(f"cannot load the readiness engine at {path}")
    module = importlib.util.module_from_spec(spec)
    try:
        spec.loader.exec_module(module)
    except OSError as exc:
        raise AlarmError(f"cannot load the readiness engine at {path}: {exc}") from exc
    return module


try:
    _ready = _load_ready_engine()
except AlarmError as exc:  # pragma: no cover - exercised only by a broken checkout
    print(f"::error title=global-stock alarm infrastructure error::{exc}")
    sys.exit(2)

GLOBAL = _ready.GLOBAL


# --------------------------------------------------------------------------- #
# The sweep (pure)
# --------------------------------------------------------------------------- #
def classify_pr(pr: dict, issues_by_number: dict, links: dict) -> dict:
    """Classify ONE open PR against the `__global__` stock question.

    Returns {state, areas, linked, closed_source_areas}. `state` is one of
    CENSUS_STATES; only `unattributed` is a finding.

    Order is load-bearing. A draft is out of scope per #5127; a human-parked PR does not
    occupy at all (`ready-issues.py::occupies_area` releases on PARKED_AREA_LABELS
    alone), so it cannot be holding `__global__` and alarming on it would be a false
    positive; then the PR's OWN areas; then — the cross-check #5127 asks for — the
    provenance-linked source issue's areas, which is the sparq-side reconstruction of the
    `busy_packages_of_pulls` union that rescues an issue-attributed PR.
    """
    labels = _ready.labels_of(pr)
    linked = sorted(links.get(pr.get("number")) or ())
    if pr.get("draft") is True:
        return {"state": "draft", "areas": set(), "linked": linked, "closed_source_areas": set()}
    if _ready.is_parked(labels):
        return {"state": "parked", "areas": set(), "linked": linked, "closed_source_areas": set()}
    own = _ready.declared_packages(labels)
    if own:
        return {"state": "attributed-by-pr", "areas": own, "linked": linked,
                "closed_source_areas": set()}
    rescued: set[str] = set()
    closed_areas: set[str] = set()
    for number in linked:
        source = issues_by_number.get(number)
        if not isinstance(source, dict) or "pull_request" in source:
            continue
        areas = _ready.declared_packages(_ready.labels_of(source))
        if str(source.get("state", "open")).upper() == "OPEN":
            rescued |= areas
        else:
            closed_areas |= areas
    if rescued:
        return {"state": "rescued-by-issue", "areas": rescued, "linked": linked,
                "closed_source_areas": closed_areas}
    return {"state": "unattributed", "areas": set(), "linked": linked,
            "closed_source_areas": closed_areas}


def sweep(prs: list[dict], issues_by_number: dict, links: dict) -> tuple[list[dict], dict]:
    """-> (findings, census). A finding is one PR holding the `__global__` partition."""
    census = {state: 0 for state in CENSUS_STATES}
    findings: list[dict] = []
    for pr in prs:
        if not isinstance(pr, dict):
            raise AlarmError("pull-request listing carried a non-object entry")
        number = pr.get("number")
        if not isinstance(number, int) or isinstance(number, bool):
            raise AlarmError("pull-request listing carried an invalid number")
        verdict = classify_pr(pr, issues_by_number, links)
        census[verdict["state"]] += 1
        if verdict["state"] != "unattributed":
            continue
        findings.append({
            "number": number,
            "title": str(pr.get("title") or ""),
            "url": str(pr.get("html_url") or ""),
            "head_ref": str(((pr.get("head") or {}).get("ref")) or ""),
            "author": str(((pr.get("user") or {}).get("login")) or ""),
            "updated_at": str(pr.get("updated_at") or ""),
            "linked": verdict["linked"],
            "closed_source_areas": sorted(verdict["closed_source_areas"]),
        })
    findings.sort(key=lambda f: f["number"])
    return findings, census


# --------------------------------------------------------------------------- #
# Persistence across ticks (pure core; the file/cache plumbing is below)
# --------------------------------------------------------------------------- #
def next_streaks(previous: dict, current_numbers) -> dict:
    """This tick's consecutive-unattributed streak per PR, keyed by PR number as a str.

    Built from THIS tick's population only: a PR that became attributed, closed, drafted
    or parked simply has no entry, so its streak is gone rather than frozen. That is what
    makes the count CONSECUTIVE ticks and not a lifetime total — a PR that flickers in
    and out of the stock never accumulates its way to an alarm.
    """
    out = {}
    for number in current_numbers:
        key = str(number)
        prior = previous.get(key, 0)
        if not isinstance(prior, int) or isinstance(prior, bool) or prior < 0:
            raise AlarmError(f"state carries a non-count streak for PR #{key}: {prior!r}")
        out[key] = prior + 1
    return out


def cold_tick_error(state_loaded: bool, prior_completed_runs: int) -> str | None:
    """The message for a COLD state read that should NOT have been cold, or None.

    A cold read on the FIRST run of the lane is expected and harmless: nothing can alarm
    this tick, and the next tick has a prior observation. A cold read once the lane has
    already COMPLETED a run means the state did not survive — the persistence layer is
    broken and this detector has silently lost the only thing that lets it distinguish a
    transient from a stall. It reports an empty alarming set either way, so the two cases
    are indistinguishable from the outside; that is exactly why this one is LOUD.
    """
    if state_loaded or prior_completed_runs <= 0:
        return None
    return (
        f"no prior tick state was restored, but this lane has {prior_completed_runs} "
        f"completed run(s) that should each have written one. The cross-tick memory is "
        f"broken, so nothing can ever reach the alarm threshold and the stock would read "
        f"as empty forever."
    )


def load_state(path: Path, repo: str) -> dict | None:
    """-> the previous tick's state, or None when there is none. Corrupt state is FATAL.

    A corrupt file is an infrastructure error rather than a silent reset: a reset lowers
    every streak to zero, which is indistinguishable from a healthy empty stock. The next
    tick writes a fresh file, so the loud failure self-heals in one tick.
    """
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8") or "null")
    except (OSError, json.JSONDecodeError) as exc:
        raise AlarmError(f"cannot read tick state {path}: {exc}") from exc
    if data is None:
        return None
    if not isinstance(data, dict) or data.get("schema") != STATE_SCHEMA:
        raise AlarmError(f"tick state {path}: not a schema-{STATE_SCHEMA} object")
    if not isinstance(data.get("streaks"), dict):
        raise AlarmError(f"tick state {path}: `streaks` is not an object")
    if data.get("repo") != repo:
        raise AlarmError(
            f"tick state {path} was written for {data.get('repo')!r}, not {repo!r} — "
            f"streaks from another repository would be meaningless here"
        )
    return data


def write_state(path: Path, repo: str, streaks: dict, now: dt.datetime) -> None:
    """Persist THIS tick's observation. Written before any verdict is rendered, so a
    tick that goes on to red (or to fail filing its issue) still advances the memory."""
    payload = {
        "schema": STATE_SCHEMA,
        "repo": repo,
        "generated_at": now.replace(microsecond=0).isoformat().replace("+00:00", "Z"),
        "streaks": streaks,
    }
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(payload, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    except OSError as exc:
        raise AlarmError(f"cannot write tick state {path}: {exc}") from exc


# --------------------------------------------------------------------------- #
# Rendering
# --------------------------------------------------------------------------- #
def _finding_line(finding: dict, streak: int) -> str:
    linked = ", ".join(f"#{n}" for n in finding["linked"]) or "none"
    line = (
        f"* #{finding['number']} — {finding['title']!r} (`{finding['head_ref']}`, "
        f"@{finding['author']}); unattributed for **{streak} consecutive sweeps**; "
        f"linked source issue(s): {linked}"
    )
    if finding["closed_source_areas"]:
        line += (
            f"; NOTE its linked source issue is CLOSED and carried "
            f"`area:{'`, `area:'.join(finding['closed_source_areas'])}` — the unit lost "
            f"its attribution when the issue closed"
        )
    return line


def render_issue(repo: str, alarming: list[dict], watching: list[dict], census: dict,
                 streaks: dict, min_ticks: int) -> tuple[str, str]:
    title = (
        f"global-stock-alarm: {len(alarming)} active PR(s) unattributed across "
        f"{min_ticks}+ consecutive sweeps"
    )
    body = "\n".join(
        [
            "> 🤖 **SPARQ agent** — automated `__global__` stock alarm "
            "(scripts/global_stock_alarm.py, #5127). @jeswr runs multiple agents on this "
            "account; this issue was filed by CI automation.",
            "",
            f"<!-- {KEY_PREFIX}: {repo} -->",
            "",
            f"Each PR below is OPEN, not a draft, not parked, carries no `area:*` label, "
            f"and no OPEN provenance-linked source issue supplies one either — so its whole "
            f"unit of work is unattributable and the scheduler's claim leg folds it into the "
            f"serializing `{GLOBAL}` partition, where it defers EVERY ready issue whatever "
            f"crate that issue names.",
            "",
            f"It has been in that state for **at least {min_ticks} consecutive sweeps**, "
            f"which is the part a per-run status cannot express: `pr-area-label` reports "
            f"`success` on every one of those runs, because emitting no label IS its "
            f"fail-closed contract.",
            "",
            f"### Unattributed for {min_ticks}+ consecutive sweeps",
            "",
            *[_finding_line(f, streaks[str(f["number"])]) for f in alarming],
            "",
            "### Unattributed this sweep, still below the threshold",
            "",
            *([_finding_line(f, streaks[str(f["number"])]) for f in watching] or ["* (none)"]),
            "",
            "### Census of every open PR this sweep",
            "",
            *[f"* {state}: {census[state]}" for state in CENSUS_STATES],
            "",
            "### What to do",
            "",
            "1. If the PR belongs to a crate, put the `area:<crate>` label on it (it must "
            "already exist — `scripts/pr-area-labels.py` never mints one), or on its source "
            "issue, which rescues the whole unit.",
            "2. If the deriver COULD have attributed it from its changed paths, the policy "
            "table `ci/area-labels.toml` is the defect, not this PR.",
            "3. If the PR is genuinely cross-cutting, it genuinely serializes the fleet — "
            "land it or park it; the stock is real, not a false positive.",
            "",
            "This issue is rewritten in place on every sweep. Close it once the stock is "
            "clear; the next sweep re-files if it is not.",
        ]
    )
    return title, body


# --------------------------------------------------------------------------- #
# gh plumbing
# --------------------------------------------------------------------------- #
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


def _flatten_pages(data, what: str) -> list:
    if not isinstance(data, list):
        raise AlarmError(f"{what} listing is not a list")
    out: list = []
    for page in data:
        if not isinstance(page, list):
            raise AlarmError(f"{what} page is not a list")
        out.extend(page)
    return out


def fetch_open_prs(repo: str) -> list[dict]:
    """Every OPEN PR, PAGINATED. A truncated read would silently shrink the stock, so an
    unexpected shape is fatal rather than best-effort."""
    return _flatten_pages(
        _gh(["api", "--paginate", "--slurp", f"repos/{repo}/pulls?state=open&per_page=100"]),
        "pull-request",
    )


def fetch_issue(repo: str, number: int) -> dict:
    data = _gh(["api", f"repos/{repo}/issues/{number}"])
    if not isinstance(data, dict):
        raise AlarmError(f"issue #{number} response is not an object")
    return data


def count_completed_runs(repo: str, workflow: str) -> int:
    """How many runs of THIS lane have already completed — the only evidence available
    for `cold_tick_error`'s "should this read have been cold?" question."""
    data = _gh([
        "api",
        f"repos/{repo}/actions/workflows/{workflow}/runs?status=completed&per_page=1",
    ])
    if not isinstance(data, dict) or not isinstance(data.get("total_count"), int):
        raise AlarmError(f"run listing for {workflow} carries no total_count")
    return int(data["total_count"])


def ensure_labels(repo: str) -> None:
    for label in BASE_LABELS:
        _gh(["label", "create", label, "--repo", repo, "--force"], parse=False)


def file_issue(repo: str, title: str, body: str) -> None:
    """ONE open issue, keyed by the body marker (the flow-on idempotency mechanism).
    PATCHed in place when it exists, because the alarming set changes tick to tick and a
    second issue per change would be the spam this must never produce."""
    marker = f"<!-- {KEY_PREFIX}: {repo} -->"
    existing = _flatten_pages(
        _gh([
            "api", "--paginate", "--slurp",
            f"repos/{repo}/issues?state=open&labels={BASE_LABELS[0]}&per_page=100",
        ]),
        "issue",
    )
    for issue in existing:
        if isinstance(issue, dict) and marker in str(issue.get("body") or ""):
            _gh([
                "api", "-X", "PATCH", f"repos/{repo}/issues/{issue['number']}",
                "-f", f"title={title}", "-f", f"body={body}",
            ])
            print(f"::notice::updated existing global-stock-alarm issue #{issue['number']}")
            return
    _gh([
        "api", "-X", "POST", f"repos/{repo}/issues",
        "-f", f"title={title}", "-f", f"body={body}",
        *[arg for label in BASE_LABELS for arg in ("-f", f"labels[]={label}")],
    ])
    print("::notice::filed a new global-stock-alarm issue")


# --------------------------------------------------------------------------- #
# Entry point
# --------------------------------------------------------------------------- #
def _parse_iso(ts: str) -> dt.datetime:
    return dt.datetime.fromisoformat(ts.replace("Z", "+00:00"))


def _load_snapshot(path: str) -> tuple[list[dict], dict]:
    """Hermetic input: {"pulls": [...], "issues": {"<number>": {...}}}."""
    with open(path, encoding="utf-8") as fh:
        payload = json.load(fh)
    issues = {int(k): v for k, v in (payload.get("issues") or {}).items()}
    return payload.get("pulls") or [], issues


def run(args: argparse.Namespace) -> int:
    repo = args.repo or os.environ.get("GITHUB_REPOSITORY") or ""
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repo):
        raise AlarmError("repo must be OWNER/REPOSITORY (set --repo or GITHUB_REPOSITORY)")
    now = _parse_iso(args.now) if args.now else dt.datetime.now(dt.timezone.utc)
    state_path = Path(args.state_file)

    hermetic = bool(args.snapshot_file)
    if hermetic:
        prs, issues_by_number = _load_snapshot(args.snapshot_file)
        links = _ready.source_issue_links(prs, repo)
    else:
        prs = fetch_open_prs(repo)
        links = _ready.source_issue_links(prs, repo)
        issues_by_number = {}
        # Only a PR that could still be a finding needs its source issues read: a draft,
        # a parked PR, or one carrying its own `area:` label is decided without them.
        for pr in prs:
            if pr.get("draft") is True or _ready.is_parked(_ready.labels_of(pr)):
                continue
            if _ready.declared_packages(_ready.labels_of(pr)):
                continue
            for number in sorted(links.get(pr.get("number")) or ()):
                if number not in issues_by_number:
                    issues_by_number[number] = fetch_issue(repo, number)

    findings, census = sweep(prs, issues_by_number, links)

    previous = load_state(state_path, repo)
    if previous is None:
        # Asked ONLY on a cold read, and only against this lane's own run history: a warm
        # tick needs no evidence, and one API call per sweep that answers nothing is a
        # failure surface for free.
        prior_completed_runs = (
            int(args.prior_runs or 0) if hermetic
            else (count_completed_runs(repo, args.workflow) if args.workflow else 0)
        )
        cold = cold_tick_error(False, prior_completed_runs)
        if cold:
            raise AlarmError(cold)
    streaks = next_streaks((previous or {}).get("streaks", {}), [f["number"] for f in findings])
    # Written BEFORE the verdict so a red tick — or one that fails to file its issue —
    # still advances the memory. Nothing below may return without this having happened.
    write_state(state_path, repo, streaks, now)

    print(f"global-stock census over {len(prs)} open PR(s) in {repo}:")
    for state in CENSUS_STATES:
        print(f"  {state}: {census[state]}")
    if previous is None:
        print(
            "::warning title=global-stock alarm has no prior tick::this is the lane's "
            "first sweep, so no PR can have been unattributed across consecutive sweeps "
            "yet. The stock is recorded; the next sweep can alarm."
        )

    alarming = [f for f in findings if streaks[str(f["number"])] >= args.min_ticks]
    watching = [f for f in findings if streaks[str(f["number"])] < args.min_ticks]
    if not alarming:
        print(
            f"global-stock alarm: no PR has held {GLOBAL} for {args.min_ticks}+ "
            f"consecutive sweeps ({len(watching)} unattributed but still below the "
            f"threshold)."
        )
        return 0

    for finding in alarming:
        print(
            f"::error title=PR unattributed across consecutive sweeps::"
            f"#{finding['number']} has carried no area attribution for "
            f"{streaks[str(finding['number'])]} consecutive sweeps and holds the "
            f"serializing {GLOBAL} partition"
        )
    title, body = render_issue(repo, alarming, watching, census, streaks, args.min_ticks)
    if args.dry_run:
        print(f"--- DRY RUN issue ---\n{title}\n{body}\n")
    else:
        ensure_labels(repo)
        file_issue(repo, title, body)
    # LOUD: the sweep itself goes red while the stock is persistent.
    return 1


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description="The `__global__` unattributed-PR stock alarm.")
    p.add_argument("--repo", default=os.environ.get("GITHUB_REPOSITORY", ""))
    p.add_argument("--state-file", default=".global-stock-state/state.json",
                   help="cross-tick streak memory (carried between runs by the workflow)")
    p.add_argument("--min-ticks", type=int, default=DEFAULT_MIN_TICKS,
                   help="consecutive sweeps a PR must be unattributed before it alarms")
    p.add_argument("--workflow", default="",
                   help="this lane's workflow file; when set, a COLD state read on a lane "
                        "that has already completed a run is a fail-loud error")
    p.add_argument("--now", help="ISO timestamp override (tests)")
    p.add_argument("--snapshot-file", help='hermetic: {"pulls": [...], "issues": {...}}')
    p.add_argument("--prior-runs", type=int, default=0,
                   help="hermetic: completed-run count for the cold-state check")
    p.add_argument("--dry-run", action="store_true", help="print findings; no gh writes")
    p.add_argument("--self-test", action="store_true")
    args = p.parse_args(argv)

    if args.self_test:
        return self_test()
    if args.min_ticks < 1:
        print("::error title=global-stock alarm infrastructure error::--min-ticks must be >= 1")
        return 2
    try:
        return run(args)
    except AlarmError as exc:
        # FAIL-LOUD: infrastructure breakage must never look like a quiet green.
        print(f"::error title=global-stock alarm infrastructure error::{exc}")
        return 2


# --------------------------------------------------------------------------- #
# Hermetic self-test
# --------------------------------------------------------------------------- #
REPO_FIXTURE = "sparq-org/sparq"


def _pr(number, *, labels=(), draft=False, ref=None, body="", assoc="NONE",
        head_repo=REPO_FIXTURE):
    return {
        "number": number,
        "title": f"pr {number}",
        "draft": draft,
        "labels": [{"name": name} for name in labels],
        "head": {"ref": ref or f"agent/{number}", "repo": {"full_name": head_repo}},
        "user": {"login": "someone"},
        "body": body,
        "author_association": assoc,
        "html_url": f"https://github.com/{REPO_FIXTURE}/pull/{number}",
        "updated_at": "2026-08-01T00:00:00Z",
    }


def _issue(number, *, labels=(), state="open"):
    return {"number": number, "state": state, "labels": [{"name": n} for n in labels]}


def _state_of(pr, issues=(), links=None):
    by_number = {i["number"]: i for i in issues}
    return classify_pr(pr, by_number, links or {})["state"]


def self_test() -> int:  # noqa: C901 - a flat table of named assertions reads best flat
    failures: list[str] = []

    def check(condition: bool, label: str) -> None:
        if not condition:
            failures.append(label)

    now = _parse_iso("2026-08-01T12:00:00Z")

    # --- the sweep -------------------------------------------------------- #
    check(_state_of(_pr(1)) == "unattributed", "a bare PR with no area is the finding")
    check(_state_of(_pr(2, labels=["area:sparq-core"])) == "attributed-by-pr",
          "its own area: label attributes a PR")
    check(_state_of(_pr(3, draft=True)) == "draft", "a draft is out of scope (#5127)")
    check(_state_of(_pr(4, labels=["needs:user"])) == "parked",
          "a human-parked PR does not occupy, so it cannot hold the global partition")
    check(_state_of(_pr(5, draft=True, labels=["needs:user"])) == "draft",
          "draft is decided before parked (order is fixed, not incidental)")

    # The cross-check #5127 asks for: an issue-rescued PR is NOT alarmed.
    linked = {6: {60}}
    check(_state_of(_pr(6, ref="sparq-agent/issue-60-x"), [_issue(60, labels=["area:sparq-zk"])],
                    linked) == "rescued-by-issue",
          "an OPEN linked source issue's area rescues the PR")
    check(_state_of(_pr(6, ref="sparq-agent/issue-60-x"), [_issue(60, labels=["P1"])],
                    linked) == "unattributed",
          "a linked source issue with no area rescues nothing")
    check(_state_of(_pr(6, ref="sparq-agent/issue-60-x"),
                    [_issue(60, labels=["area:sparq-zk"], state="closed")], linked)
          == "unattributed",
          "a CLOSED source issue does not rescue (mirrors unit_reservations' open snapshot)")
    verdict = classify_pr(_pr(6, ref="sparq-agent/issue-60-x"),
                          {60: _issue(60, labels=["area:sparq-zk"], state="closed")}, linked)
    check(verdict["closed_source_areas"] == {"sparq-zk"},
          "the closed source issue's lost areas are carried into the finding, not dropped")

    # Linkage is the engine's, so a fork head must never link (attacker-controlled text).
    fork = _pr(7, ref="sparq-agent/issue-70-x", head_repo="attacker/sparq")
    check(_ready.source_issue_links([fork], REPO_FIXTURE) == {},
          "a fork head does not link a source issue")

    prs = [_pr(1), _pr(2, labels=["area:sparq-core"]), _pr(3, draft=True),
           _pr(4, labels=["needs:user"]), _pr(6, ref="sparq-agent/issue-60-x")]
    links = _ready.source_issue_links(prs, REPO_FIXTURE)
    findings, census = sweep(prs, {60: _issue(60, labels=["area:sparq-zk"])}, links)
    check([f["number"] for f in findings] == [1], "the sweep finds exactly the bare PR")
    check(census == {"draft": 1, "parked": 1, "attributed-by-pr": 1,
                     "rescued-by-issue": 1, "unattributed": 1},
          f"the census accounts for every open PR (got {census})")

    # --- persistence across ticks ----------------------------------------- #
    check(next_streaks({}, [1, 2]) == {"1": 1, "2": 1}, "a cold tick starts every streak at 1")
    check(next_streaks({"1": 1}, [1, 2]) == {"1": 2, "2": 1},
          "a PR seen again increments; a new one starts at 1")
    check(next_streaks({"1": 5}, [2]) == {"2": 1},
          "a PR that leaves the stock loses its streak (CONSECUTIVE, not cumulative)")
    check("3" not in next_streaks({"3": 9}, []), "an empty stock retains nothing")

    # --- the cold-state fail-loud ----------------------------------------- #
    check(cold_tick_error(True, 5) is None, "a warm read is never an error")
    check(cold_tick_error(False, 0) is None, "the lane's first-ever sweep may read cold")
    check(cold_tick_error(False, 1) is not None,
          "a cold read after a completed run is a broken memory and must be LOUD")

    with tempfile.TemporaryDirectory() as td:
        snap = Path(td) / "snap.json"
        state = Path(td) / "state.json"

        def write_snapshot(pulls, issues=None):
            snap.write_text(json.dumps({
                "pulls": pulls,
                "issues": {str(i["number"]): i for i in (issues or [])},
            }))

        def invoke(extra=()):
            # Output is captured, not printed: the assertions below are on EXIT CODES,
            # and a dozen rendered dry-run bodies would bury the one line that matters
            # when this self-test fails as CI's first step.
            with contextlib.redirect_stdout(io.StringIO()):
                return main(["--repo", REPO_FIXTURE, "--snapshot-file", str(snap),
                             "--state-file", str(state), "--now", "2026-08-01T12:00:00Z",
                             "--dry-run", *extra])

        # Tick 1: one unattributed PR is RECORDED but must NOT alarm.
        write_snapshot([_pr(1), _pr(2, labels=["area:sparq-core"])])
        check(invoke() == 0, "tick 1: a first sighting exits 0")
        check(json.loads(state.read_text())["streaks"] == {"1": 1}, "tick 1 records the streak")
        # Tick 2: the same PR, still unattributed => ALARM.
        check(invoke() == 1, "tick 2: unattributed twice running exits 1")
        check(json.loads(state.read_text())["streaks"] == {"1": 2}, "tick 2 advances the streak")
        # Tick 3: it acquires an area => the stock empties and the streak is dropped.
        write_snapshot([_pr(1, labels=["area:sparq-core"])])
        check(invoke() == 0, "tick 3: an attributed PR exits 0")
        check(json.loads(state.read_text())["streaks"] == {}, "tick 3 clears the streak")
        # Tick 4: it regresses. Having alarmed before must not shortcut the threshold.
        write_snapshot([_pr(1)])
        check(invoke() == 0, "tick 4: a re-sighting after a clean tick starts over")

        # --min-ticks is honoured: 3 sweeps required means tick 2 is still quiet.
        state.unlink(missing_ok=True)
        write_snapshot([_pr(1)])
        check(invoke(["--min-ticks", "3"]) == 0, "min-ticks 3: sweep 1 quiet")
        check(invoke(["--min-ticks", "3"]) == 0, "min-ticks 3: sweep 2 still quiet")
        check(invoke(["--min-ticks", "3"]) == 1, "min-ticks 3: sweep 3 alarms")

        # An issue-rescued PR never reaches the threshold however long it sits.
        state.unlink(missing_ok=True)
        write_snapshot([_pr(6, ref="sparq-agent/issue-60-x")],
                       [_issue(60, labels=["area:sparq-zk"])])
        check(invoke() == 0 and invoke() == 0,
              "an issue-rescued PR never alarms, however many sweeps it sits for")
        check(json.loads(state.read_text())["streaks"] == {},
              "an issue-rescued PR is not even recorded as stock")

        # Corrupt / foreign state is FATAL, not a silent reset to zero.
        state.write_text("{not json")
        check(invoke() == 2, "corrupt state is a fail-loud exit 2")
        state.write_text(json.dumps({"schema": 1, "repo": "other/repo", "streaks": {}}))
        check(invoke() == 2, "state written for another repository is a fail-loud exit 2")
        state.unlink(missing_ok=True)
        check(invoke(["--prior-runs", "4"]) == 2,
              "a cold read on a lane with completed runs is a fail-loud exit 2")
        with contextlib.redirect_stdout(io.StringIO()):
            rc = main(["--repo", REPO_FIXTURE, "--snapshot-file", str(snap),
                       "--state-file", str(state), "--dry-run", "--min-ticks", "0"])
        check(rc == 2,
              "--min-ticks 0 is refused (it would alarm on first sighting)")

        # The rendered issue names the PR, its streak, and the dedupe marker.
        state.unlink(missing_ok=True)
        write_snapshot([_pr(1)])
        invoke()
        streaks = {"1": 2}
        findings, census = sweep([_pr(1)], {}, {})
        title, body = render_issue(REPO_FIXTURE, findings, [], census, streaks, 2)
        check(f"<!-- {KEY_PREFIX}: {REPO_FIXTURE} -->" in body, "the dedupe marker is in the body")
        check(body.startswith("> 🤖"), "the body self-identifies as a SPARQ agent")
        check("#1" in body and "2 consecutive sweeps" in body,
              "the body names the PR and its streak")
        check("1 active PR(s)" in title, "the title carries the alarming count")

        # write_state/load_state round-trip, including the fields the next tick reads.
        write_state(state, REPO_FIXTURE, {"9": 3}, now)
        restored = load_state(state, REPO_FIXTURE)
        check(restored is not None and restored["streaks"] == {"9": 3},
              "state round-trips through the cache file")

    if failures:
        for label in failures:
            print(f"::error::global_stock_alarm self-test FAILED: {label}")
        return 1
    print(
        "global_stock_alarm self-test: sweep classification (draft/parked/own-area/"
        "issue-rescued/unattributed), the issue cross-check, consecutive-tick streaks, "
        "the cold-state and corrupt-state fail-louds, and the end-to-end exit codes all "
        "behaved."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
