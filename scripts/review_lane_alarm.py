#!/usr/bin/env python3
# [OPUS-5] Review-lane BLIND-SPOT alarm. 🤖 SPARQ agent.
#
# WHAT THIS IS: the "silence must not look like green" safeguard for VERDICT PRODUCTION,
# the sibling of scripts/formal_lane_alarm.py (which watches verdict production for the
# formal lanes) and scripts/ci_selection_alarm.py.
#
# MEASURED CAUSE (2026-07-26, paginated over all 118 open PRs, total_count cross-checked):
# every review that sparq PRs receive is produced OUT OF REPO by the registry's
# review-fix.yml, scheduled by the registry's dispatch.yml. That scheduler admits a PR
# only through `enumerate_review_items` (registry scripts/dispatch-claim.py), whose first
# three gates are AUTHOR-SIDE and fail closed:
#
#     if not HEAD_REF_RE.match(ref):        continue   # ^sparq-agent/issue-([1-9][0-9]*)-
#     if head_repo != repo:                 continue
#     if not login.endswith("[bot]") ...:   continue   # must be the worker App bot
#
# followed by a mandatory REGISTRY provenance record (`provenance_admission_error`) that
# only a host-side worker run can write. A PR opened by anything other than the worker App
# — every PR an interactive agent session pushes — fails all three by construction and is
# therefore INVISIBLE to the only review producer that exists. The census that proves it:
#
#     89 open PRs by sparq-orchestrator[bot]: 100% carry a review:* loop label
#     29 open PRs by jeswr:                     0 carry review:pass/changes/needs
#
# Zero is not a coincidence, it is a predicate. Nothing in sparq ever noticed, because
# absence of a review produces no artifact — no red check, no queue entry, nothing. This
# script makes that absence LOUD.
#
# THE ALARM: enumerate the BLIND SPOT — open, non-draft, non-human-held PRs that the
# registry lane can never reach — and RED this run (plus file one deduped issue) when any
# of them has sat without a countable verdict for longer than the threshold.
#
# WHAT COUNTS AS A VERDICT (deliberately strict; a wrong pass is worse than no pass):
#   1. LINE-ANCHORED. `VERDICT: pass` / `VERDICT: fail` as the LAST non-blank line of the
#      comment, matched with ^...$ under re.MULTILINE-free full-line comparison. NEVER a
#      substring search: the standing review brief quotes the string `VERDICT: pass` in its
#      own instruction text, and a substring grep has already scored that quotation as a
#      real pass in this repo.
#   2. HEAD-BOUND. The comment body must contain the CURRENT head as a standalone 40-hex
#      SHA. A force-push advances the head, so an old comment stops counting the instant
#      the tree it reviewed is superseded.
#   3. AUTHOR XOR REVIEWER. A comment posted by the SAME GitHub account that opened the PR
#      is never a review of it. Self-approval has already happened in this repo; a detector
#      that credits it would certify the exact failure it exists to find.
# A comment failing (1) is not a verdict at all. A comment satisfying (1) but failing (2)
# or (3) is a DISCREDITED verdict — reported by name in the issue body, because "reviewed
# on a stale head" and "never reviewed" need different human responses.
#
# TWO DESIGN INVARIANTS (mirrors formal_lane_alarm.py / ci_selection_alarm.py):
#   1. FAIL-LOUD: an alarm-INFRASTRUCTURE error (gh api failure, unparseable response)
#      exits NON-ZERO with a `::error::` annotation. A broken detector must never look
#      like a quiet green.
#   2. NON-SPAMMY: ONE open issue, keyed by a stable `<!-- review-lane-alarm-key: … -->`
#      body marker searched over open `review-alarm`-labelled issues before filing.
#
# WHAT THIS IS NOT. It is NOT a gate: scheduled on main, creates no check-run on any PR
# head or merge-group ref, blocks no merge. It grants NO arming authority and writes NO PR
# labels — it cannot arm, promote, or unblock anything, by construction (the workflow holds
# `issues: write` and nothing else). It does not lower any bar: it never marks a PR
# reviewed, it only refuses to let "nobody reviewed this" stay silent.
#
# Usage:
#   review_lane_alarm.py                              # real (gh; env GITHUB_REPOSITORY)
#   review_lane_alarm.py --dry-run                    # print findings; no gh writes
#   review_lane_alarm.py --prs-file prs.json \
#       --now 2026-07-26T22:00:00Z --dry-run          # hermetic (tests)
#   review_lane_alarm.py --self-test                  # hermetic fixtures
#
# Stdlib only + the `gh` CLI (present on every GitHub runner).

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import subprocess
import sys

BASE_LABELS = ["review-alarm", "auto"]
KEY_PREFIX = "review-lane-alarm-key"
ALARM_KEY = "blind-spot"

# The registry review lane's own admission regex, replicated VERBATIM from
# scripts/dispatch-claim.py (registry, master @ 2026-07-26). Kept byte-identical on
# purpose: this alarm's whole claim is "the registry lane cannot see these PRs", so the
# reachability test must be the registry's test and not a paraphrase of it.
REGISTRY_HEAD_REF_RE = re.compile(r"^sparq-agent/issue-([1-9][0-9]*)-")

# A verdict line, anchored to a WHOLE line. The pattern is applied to an individually
# stripped line — never searched across a body — so no amount of surrounding prose can
# make it match.
VERDICT_LINE_RE = re.compile(r"^VERDICT:[ ]+(pass|fail)$")
# A standalone 40-hex commit SHA (word-bounded so a 41-char token never matches).
SHA_RE = re.compile(r"(?<![0-9a-fA-F])([0-9a-f]{40})(?![0-9a-fA-F])")

# Human-owned holds. Terminal: autonomy stands down until a human clears them, so an
# unreviewed PR carrying one is NOT an alarm — it is a human's open decision.
HUMAN_HOLD_LABELS = frozenset({"needs:user", "review:needs-user"})
# The registry loop's own labels. A blind-spot PR cannot legitimately carry one (nothing
# in the blind spot is enumerable by the lane that writes them), but if a human or a
# migration puts one there we honour it rather than double-reporting.
LANE_LABELS = frozenset({"review:pass", "review:changes", "review:needs", "review:parked"})

DEFAULT_MAX_AGE_HOURS = 24


class AlarmError(Exception):
    """Alarm-INFRASTRUCTURE failure — surfaced LOUD (non-zero exit)."""


# --------------------------------------------------------------------------- #
# Reachability: can the registry review lane EVER enumerate this PR?
# --------------------------------------------------------------------------- #
def registry_lane_reachable(pr: dict, repo: str) -> bool:
    """True iff `pr` passes every AUTHOR-SIDE admission gate of the registry review
    lane's `enumerate_review_items`. False => no review producer in existence can ever
    see this PR, which is precisely the blind spot this alarm watches.

    Only the three structural gates are replicated. The registry additionally requires a
    provenance record, but that is a LIVE registry read this repo has no token for, and it
    can only ever narrow admission further — so treating a PR as reachable here is the
    CONSERVATIVE direction (it under-reports the blind spot, never over-reports it)."""
    if not isinstance(pr, dict):
        raise AlarmError("pull-request listing carried a non-object entry")
    head = pr.get("head") or {}
    ref = str(head.get("ref") or "")
    head_repo = (head.get("repo") or {}).get("full_name")
    login = str((pr.get("user") or {}).get("login") or "")
    if not REGISTRY_HEAD_REF_RE.match(ref):
        return False
    if head_repo != repo:
        return False
    if not login.endswith("[bot]"):
        return False
    return True


# --------------------------------------------------------------------------- #
# Verdict detection (pure)
# --------------------------------------------------------------------------- #
def verdict_polarity(body: str) -> str | None:
    """The polarity of a comment's verdict line, or None when the comment carries no
    verdict at all.

    STRICTLY the LAST non-blank line, compared whole. This is the anti-substring rule:
    the review brief's own instruction text contains the literal `VERDICT: pass`, and a
    body that merely quotes it — anywhere, including as its own line followed by more
    prose — is NOT a verdict. Only a comment that ENDS on the line is."""
    if not isinstance(body, str):
        return None
    lines = [line.rstrip() for line in body.replace("\r\n", "\n").split("\n")]
    for line in reversed(lines):
        if not line.strip():
            continue
        match = VERDICT_LINE_RE.match(line.strip())
        return match.group(1) if match else None
    return None


def comment_binds_head(body: str, head_sha: str) -> bool:
    """True iff the comment names the CURRENT head as a standalone 40-hex SHA.

    An OCCURRENCE test, exactly as the arming bridge does it: what makes the comment a
    verdict is its trailing VERDICT line, and what binds it to a tree is naming that
    tree. A comment naming only the PREVIOUS head fails here the moment a force-push
    lands, which is the required force-push invalidation."""
    if not isinstance(body, str) or not isinstance(head_sha, str):
        return False
    if not SHA_RE.fullmatch(head_sha):
        return False
    return head_sha in SHA_RE.findall(body)


def countable_verdict(pr: dict, comments: list[dict]) -> tuple[str | None, list[str]]:
    """-> (polarity, discredited_reasons).

    `polarity` is 'pass'/'fail' for the LATEST comment that is a verdict AND binds the
    current head AND was not written by the PR's own author. `discredited_reasons`
    names every comment that carried a real verdict line but failed one of the other two
    tests — the difference between "stale review" and "no review" is the difference
    between two very different human responses, so it is never collapsed."""
    head_sha = str((pr.get("head") or {}).get("sha") or "")
    pr_author = str((pr.get("user") or {}).get("login") or "")
    if not comments:
        return None, []
    ordered = sorted(
        (c for c in comments if isinstance(c, dict)),
        key=lambda c: str(c.get("created_at") or ""),
    )
    polarity: str | None = None
    discredited: list[str] = []
    for comment in ordered:
        body = comment.get("body")
        found = verdict_polarity(body if isinstance(body, str) else "")
        if found is None:
            continue
        author = str((comment.get("user") or {}).get("login") or "")
        # AUTHOR XOR REVIEWER, structurally. Checked BEFORE the head test so a
        # self-review is never reported as merely "stale" — its problem is not its age.
        if author and pr_author and author == pr_author:
            discredited.append(f"self-review by the PR author @{author} (never countable)")
            continue
        if not comment_binds_head(body if isinstance(body, str) else "", head_sha):
            discredited.append(
                f"verdict by @{author or '?'} names no current head "
                f"{head_sha[:12] or '(unknown)'} — superseded tree"
            )
            continue
        # Latest head-bound, non-self verdict wins: a later retraction defeats an
        # earlier pass.
        polarity = found
    return polarity, discredited


# --------------------------------------------------------------------------- #
# Classification (pure)
# --------------------------------------------------------------------------- #
def _labels(pr: dict) -> set[str]:
    out: set[str] = set()
    for label in pr.get("labels") or []:
        name = label.get("name") if isinstance(label, dict) else label
        if isinstance(name, str) and name:
            out.add(name)
    return out


def classify_pr(pr: dict, comments: list[dict], repo: str) -> str:
    """One of: registry-lane | draft | human-held | lane-labelled | has-verdict |
    blind-spot. Only `blind-spot` is alarm-worthy."""
    if pr.get("draft") is True:
        return "draft"
    if registry_lane_reachable(pr, repo):
        return "registry-lane"
    if HUMAN_HOLD_LABELS & _labels(pr):
        return "human-held"
    if LANE_LABELS & _labels(pr):
        return "lane-labelled"
    polarity, _ = countable_verdict(pr, comments)
    if polarity is not None:
        return "has-verdict"
    return "blind-spot"


def _parse_iso(ts: str) -> dt.datetime:
    try:
        return dt.datetime.fromisoformat(str(ts).replace("Z", "+00:00"))
    except (TypeError, ValueError) as exc:
        raise AlarmError(f"unparseable timestamp {ts!r}") from exc


def find_blind_spot(
    prs: list[dict], comments_by_pr: dict, repo: str, now: dt.datetime, max_age_hours: float
) -> tuple[list[dict], dict]:
    """-> (alarming findings, census). The census counts EVERY state exit, not just the
    alarming one: a per-state population is the only shape in which a MISSING edge is
    visible at all (a success-rate cannot express "nothing ever entered this state")."""
    census: dict = {}
    findings: list[dict] = []
    for pr in prs:
        number = pr.get("number")
        if not isinstance(number, int) or isinstance(number, bool) or number <= 0:
            raise AlarmError("pull-request listing carried an invalid number")
        comments = comments_by_pr.get(number) or comments_by_pr.get(str(number)) or []
        state = classify_pr(pr, comments, repo)
        census[state] = census.get(state, 0) + 1
        if state != "blind-spot":
            continue
        age = (now - _parse_iso(pr.get("created_at"))).total_seconds() / 3600.0
        if age < max_age_hours:
            census["blind-spot-fresh"] = census.get("blind-spot-fresh", 0) + 1
            continue
        _, discredited = countable_verdict(pr, comments)
        findings.append(
            {
                "number": number,
                "title": str(pr.get("title") or "")[:120],
                "author": str((pr.get("user") or {}).get("login") or "?"),
                "head_ref": str((pr.get("head") or {}).get("ref") or "?"),
                "head_sha": str((pr.get("head") or {}).get("sha") or "?"),
                "age_hours": round(age, 1),
                "discredited": discredited,
            }
        )
    findings.sort(key=lambda f: -f["age_hours"])
    return findings, census


# --------------------------------------------------------------------------- #
# Reporting
# --------------------------------------------------------------------------- #
def render_issue_body(findings: list[dict], census: dict, repo: str, max_age_hours: float) -> str:
    lines = [
        "> 🤖 SPARQ agent",
        "",
        f"`review-alarm`: **{len(findings)} open PR(s)** have gone more than "
        f"{max_age_hours:g}h with **no countable review verdict**, and no review producer "
        "can ever reach them.",
        "",
        "These PRs fail the registry review lane's author-side admission gates "
        "(`enumerate_review_items` in the registry's `scripts/dispatch-claim.py`): the head "
        "ref does not match `^sparq-agent/issue-([1-9][0-9]*)-`, and/or the author is not the "
        "worker App bot. That lane is the only verdict producer sparq has, so nothing will "
        "ever dispatch a review for them without a human or an orchestrator doing it by hand.",
        "",
        "| PR | age (h) | author | head ref | note |",
        "| --- | --- | --- | --- | --- |",
    ]
    for finding in findings[:50]:
        note = "; ".join(finding["discredited"]) or "no verdict comment at all"
        lines.append(
            f"| #{finding['number']} | {finding['age_hours']} | {finding['author']} | "
            f"`{finding['head_ref']}` | {note} |"
        )
    if len(findings) > 50:
        lines.append(f"| … | | | | {len(findings) - 50} more |")
    lines += [
        "",
        "**Census of every state exit** (open PRs in `" + repo + "`):",
        "",
        "```",
        *[f"{state}: {count}" for state, count in sorted(census.items())],
        "```",
        "",
        "A verdict is countable only when it is line-anchored (`VERDICT: pass|fail` as the "
        "comment's last non-blank line), names the current head as a standalone 40-hex SHA, "
        "and was **not** posted by the PR's own author.",
        "",
        f"<!-- {KEY_PREFIX}: {ALARM_KEY} -->",
    ]
    return "\n".join(lines)


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


def fetch_open_prs(repo: str) -> list[dict]:
    """Every OPEN PR, PAGINATED. `gh api --paginate` walks the Link headers; a truncated
    read would silently shrink the blind spot, so an unexpected shape is fatal."""
    data = _gh(["api", "--paginate", "--slurp", f"repos/{repo}/pulls?state=open&per_page=100"])
    if not isinstance(data, list):
        raise AlarmError("pull-request listing is not a list")
    prs: list[dict] = []
    for page in data:
        if not isinstance(page, list):
            raise AlarmError("pull-request page is not a list")
        prs.extend(page)
    return prs


def fetch_comments(repo: str, number: int) -> list[dict]:
    data = _gh(["api", "--paginate", "--slurp", f"repos/{repo}/issues/{number}/comments"])
    if not isinstance(data, list):
        raise AlarmError(f"comment listing for #{number} is not a list")
    out: list[dict] = []
    for page in data:
        if not isinstance(page, list):
            raise AlarmError(f"comment page for #{number} is not a list")
        out.extend(page)
    return out


def file_issue(repo: str, body: str, count: int) -> None:
    """One open issue, keyed by the body marker (the flow-on idempotency mechanism)."""
    marker = f"<!-- {KEY_PREFIX}: {ALARM_KEY} -->"
    existing = _gh(
        [
            "api",
            "--paginate",
            "--slurp",
            f"repos/{repo}/issues?state=open&labels=review-alarm&per_page=100",
        ]
    )
    if not isinstance(existing, list):
        raise AlarmError("issue listing is not a list")
    for page in existing:
        for issue in page if isinstance(page, list) else []:
            if isinstance(issue, dict) and marker in str(issue.get("body") or ""):
                _gh(
                    [
                        "api",
                        "-X",
                        "PATCH",
                        f"repos/{repo}/issues/{issue['number']}",
                        "-f",
                        f"body={body}",
                    ]
                )
                print(f"::notice::updated existing review-alarm issue #{issue['number']}")
                return
    title = f"review-alarm: {count} PR(s) unreachable by every review producer"
    _gh(
        [
            "api",
            "-X",
            "POST",
            f"repos/{repo}/issues",
            "-f",
            f"title={title}",
            "-f",
            f"body={body}",
            *[arg for label in BASE_LABELS for arg in ("-f", f"labels[]={label}")],
        ]
    )
    print("::notice::filed a new review-alarm issue")


# --------------------------------------------------------------------------- #
# Entry point
# --------------------------------------------------------------------------- #
def run(args: argparse.Namespace) -> int:
    repo = args.repo or os.environ.get("GITHUB_REPOSITORY") or ""
    if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repo):
        raise AlarmError("repo must be OWNER/REPOSITORY (set --repo or GITHUB_REPOSITORY)")
    now = _parse_iso(args.now) if args.now else dt.datetime.now(dt.timezone.utc)

    if args.prs_file:
        with open(args.prs_file, encoding="utf-8") as fh:
            payload = json.load(fh)
        prs = payload.get("pulls") or []
        comments_by_pr = payload.get("comments") or {}
    else:
        prs = fetch_open_prs(repo)
        comments_by_pr = {}
        for pr in prs:
            number = pr.get("number")
            if not isinstance(number, int) or isinstance(number, bool):
                raise AlarmError("pull-request listing carried an invalid number")
            # Only the blind-spot candidates need their comments read; a reachable or
            # drafted PR is decided without them.
            if pr.get("draft") is not True and not registry_lane_reachable(pr, repo):
                comments_by_pr[number] = fetch_comments(repo, number)

    findings, census = find_blind_spot(prs, comments_by_pr, repo, now, args.max_age_hours)

    print(f"review-alarm census over {len(prs)} open PR(s) in {repo}:")
    for state, count in sorted(census.items()):
        print(f"  {state}: {count}")
    for finding in findings:
        note = "; ".join(finding["discredited"]) or "no verdict comment at all"
        print(f"  BLIND SPOT #{finding['number']} ({finding['age_hours']}h) {note}")

    if not findings:
        print("::notice::no PR is outside every review producer's reach")
        return 0

    body = render_issue_body(findings, census, repo, args.max_age_hours)
    if args.dry_run:
        print(body)
    else:
        file_issue(repo, body, len(findings))
    print(
        f"::error::{len(findings)} open PR(s) have no countable review verdict and are "
        "unreachable by every review producer — verdict PRODUCTION, not transport, is the "
        "stalled stage"
    )
    return 1


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", default="")
    parser.add_argument("--now", default="")
    parser.add_argument("--prs-file", default="")
    parser.add_argument("--max-age-hours", type=float, default=DEFAULT_MAX_AGE_HOURS)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)
    if args.self_test:
        return _self_test()
    try:
        return run(args)
    except AlarmError as exc:
        print(f"::error::review-lane alarm infrastructure failure: {exc}")
        return 2


# --------------------------------------------------------------------------- #
# Hermetic self-test (runs as the FIRST step of every scheduled run — a detector is
# never trusted to watch anything before it has proved itself on this tick's code).
# --------------------------------------------------------------------------- #
SHA_A = "a" * 40
SHA_B = "b" * 40
# The literal the standing review brief quotes at every reviewer. Substring-grepping for
# it has already scored a false pass in this repo; the fixture pins that it cannot again.
BRIEF_QUOTE = (
    "End with a line-anchored final line, exactly `VERDICT: pass` or `VERDICT: fail`.\n"
    "Please review the diff now.\n"
)


def _pr(number=1, *, ref="research/x", login="jeswr", sha=SHA_A, draft=False, labels=(),
        created="2026-07-01T00:00:00Z", repo="sparq-org/sparq"):
    return {
        "number": number,
        "draft": draft,
        "created_at": created,
        "title": f"pr {number}",
        "user": {"login": login},
        "labels": [{"name": name} for name in labels],
        "head": {"ref": ref, "sha": sha, "repo": {"full_name": repo}},
    }


def _comment(body, login="reviewer", created="2026-07-02T00:00:00Z"):
    return {"body": body, "user": {"login": login}, "created_at": created}


def _self_test() -> int:  # noqa: C901 - a flat table of named assertions reads best flat
    failures: list[str] = []

    def check(name: str, condition: bool) -> None:
        if not condition:
            failures.append(name)

    repo = "sparq-org/sparq"

    # --- reachability: the registry lane's author-side gates, pinned to REAL branch
    # names measured on 2026-07-26.
    check(
        "worker branch + bot author is reachable",
        registry_lane_reachable(
            _pr(ref="sparq-agent/issue-2908-30221671021-1", login="sparq-orchestrator[bot]"), repo
        ),
    )
    check(
        "local agent branch is NOT reachable",
        not registry_lane_reachable(
            _pr(ref="ci/pr-freshness-auto-update-branch", login="jeswr"), repo
        ),
    )
    check(
        "bot author on a non-worker branch is NOT reachable",
        not registry_lane_reachable(
            _pr(ref="research/x", login="sparq-orchestrator[bot]"), repo
        ),
    )
    check(
        "worker branch from a NON-bot author is NOT reachable",
        not registry_lane_reachable(
            _pr(ref="sparq-agent/issue-1-2-1", login="jeswr"), repo
        ),
    )
    check(
        "fork head is NOT reachable",
        not registry_lane_reachable(
            _pr(
                ref="sparq-agent/issue-1-2-1",
                login="sparq-orchestrator[bot]",
                repo="attacker/sparq",
            ),
            repo,
        ),
    )
    check("issue-0 is not a worker branch", not REGISTRY_HEAD_REF_RE.match("sparq-agent/issue-0-1-1"))

    # --- verdict line: anchored, never a substring.
    check("plain pass parses", verdict_polarity(f"{SHA_A}\n\nVERDICT: pass") == "pass")
    check("plain fail parses", verdict_polarity(f"{SHA_A}\n\nVERDICT: fail") == "fail")
    check("trailing blank lines tolerated", verdict_polarity("VERDICT: pass\n\n  \n") == "pass")
    check("INSTRUCTION TEXT is not a verdict", verdict_polarity(BRIEF_QUOTE) is None)
    check(
        "quoted verdict followed by prose is not a verdict",
        verdict_polarity("VERDICT: pass\nactually I am still thinking") is None,
    )
    check("backticked line is not a verdict", verdict_polarity("`VERDICT: pass`") is None)
    check("suffixed line is not a verdict", verdict_polarity("VERDICT: pass (with caveats)") is None)
    check("wrong case is not a verdict", verdict_polarity("VERDICT: PASS") is None)
    check("unknown polarity is not a verdict", verdict_polarity("VERDICT: maybe") is None)
    check("empty body is not a verdict", verdict_polarity("") is None)
    check("non-string body is not a verdict", verdict_polarity(None) is None)

    # --- head binding.
    check("names current head", comment_binds_head(f"reviewed {SHA_A}\nVERDICT: pass", SHA_A))
    check("names only the old head", not comment_binds_head(f"reviewed {SHA_B}", SHA_A))
    check("short sha does not bind", not comment_binds_head(SHA_A[:12], SHA_A))
    check("41-hex token does not bind", not comment_binds_head(SHA_A + "c", SHA_A))
    check("unknown head never binds", not comment_binds_head(SHA_A, ""))

    # --- countable_verdict: the three rules composed.
    pr = _pr(sha=SHA_A, login="jeswr")
    good = [_comment(f"reviewed {SHA_A}\n\nVERDICT: pass", login="reviewer-bot")]
    check("a clean cross-author head-bound pass counts", countable_verdict(pr, good)[0] == "pass")

    stale = [_comment(f"reviewed {SHA_B}\n\nVERDICT: pass", login="reviewer-bot")]
    polarity, why = countable_verdict(pr, stale)
    check("SUPERSEDED HEAD does not count", polarity is None)
    check("superseded head is reported, not dropped", any("superseded" in r for r in why))

    selfrev = [_comment(f"reviewed {SHA_A}\n\nVERDICT: pass", login="jeswr")]
    polarity, why = countable_verdict(pr, selfrev)
    check("AUTHOR CANNOT VERDICT THEIR OWN PR", polarity is None)
    check("self-review is reported by name", any("self-review" in r for r in why))

    quoted = [_comment(f"{SHA_A}\n{BRIEF_QUOTE}", login="reviewer-bot")]
    check("INSTRUCTION SUBSTRING IS NOT A PASS", countable_verdict(pr, quoted)[0] is None)

    retract = [
        _comment(f"{SHA_A}\n\nVERDICT: pass", login="r1", created="2026-07-02T00:00:00Z"),
        _comment(f"{SHA_A}\n\nVERDICT: fail", login="r2", created="2026-07-03T00:00:00Z"),
    ]
    check("a later fail supersedes an earlier pass", countable_verdict(pr, retract)[0] == "fail")

    # --- classification + the alarm decision.
    now = _parse_iso("2026-07-26T00:00:00Z")
    check(
        "an unreviewed local-branch PR is a BLIND SPOT",
        classify_pr(_pr(), [], repo) == "blind-spot",
    )
    check(
        "NO VERDICT IS NOT ARMABLE — it never reads as reviewed",
        classify_pr(_pr(), [], repo) != "has-verdict",
    )
    check("a worker PR is the registry lane's", classify_pr(_pr(
        ref="sparq-agent/issue-9-1-1", login="sparq-orchestrator[bot]"), [], repo) == "registry-lane")
    check("a draft is not alarmed", classify_pr(_pr(draft=True), [], repo) == "draft")
    check(
        "a human hold is terminal",
        classify_pr(_pr(labels=("needs:user",)), [], repo) == "human-held",
    )
    check(
        "a lane label is honoured",
        classify_pr(_pr(labels=("review:pass",)), [], repo) == "lane-labelled",
    )
    check("a real verdict clears the blind spot", classify_pr(_pr(), good, repo) == "has-verdict")
    check(
        "a SELF verdict does NOT clear the blind spot",
        classify_pr(_pr(), selfrev, repo) == "blind-spot",
    )

    old = _pr(number=3426, created="2026-07-18T00:00:00Z")
    fresh = _pr(number=4342, created="2026-07-25T23:00:00Z")
    findings, census = find_blind_spot([old, fresh], {}, repo, now, DEFAULT_MAX_AGE_HOURS)
    check("the aged blind-spot PR alarms", [f["number"] for f in findings] == [3426])
    check("a fresh blind-spot PR does not alarm yet", census.get("blind-spot-fresh") == 1)
    check("the census counts every state exit", census.get("blind-spot") == 2)

    findings2, _ = find_blind_spot(
        [_pr(number=7, created="2026-07-18T00:00:00Z")], {7: good}, repo, now,
        DEFAULT_MAX_AGE_HOURS)
    check("a reviewed PR raises no finding", findings2 == [])

    findings3, _ = find_blind_spot(
        [_pr(number=8, created="2026-07-18T00:00:00Z")], {8: selfrev}, repo, now,
        DEFAULT_MAX_AGE_HOURS)
    check("a SELF-reviewed PR still alarms", [f["number"] for f in findings3] == [8])
    check(
        "the self-review reason reaches the issue body",
        bool(findings3) and any("self-review" in r for r in findings3[0]["discredited"]),
    )

    body = render_issue_body(findings3, {"blind-spot": 1}, repo, DEFAULT_MAX_AGE_HOURS)
    check("the issue body self-identifies", body.startswith("> 🤖 SPARQ agent"))
    check("the issue body carries the dedupe key", f"<!-- {KEY_PREFIX}: {ALARM_KEY} -->" in body)

    # --- fail-loud on malformed input (never a quiet green).
    for name, thunk in (
        ("malformed PR entry raises", lambda: registry_lane_reachable("nope", repo)),
        (
            "invalid PR number raises",
            lambda: find_blind_spot([{"number": 0}], {}, repo, now, 1.0),
        ),
        (
            "unparseable timestamp raises",
            lambda: find_blind_spot(
                [_pr(created="not-a-date")], {}, repo, now, 1.0
            ),
        ),
    ):
        try:
            thunk()
        except AlarmError:
            pass
        else:
            failures.append(name)

    if failures:
        for name in failures:
            print(f"FAIL: {name}")
        print(f"::error::review_lane_alarm self-test: {len(failures)} failure(s)")
        return 1
    print("review_lane_alarm self-test: all checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
