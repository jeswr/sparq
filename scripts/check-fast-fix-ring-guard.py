#!/usr/bin/env python3
# [OPUS-4.8] sparq-org/sparq#3475 — fail-closed regression guard for the fast-fix
# ring's cross-repo trust boundary. 🤖 SPARQ agent. Authored by Opus 4.8 (Fable
# routed off security surfaces by design; flag for re-review when Fable returns).
#
# WHAT THIS PROVES (about .github/workflows/fast-fix-ring.yml):
#   The ring's ONLY privileged capability is the cross-repo REGISTRY_RING_TOKEN
#   doorbell. A `workflow_run` workflow is ALSO delivered for fork / cross-repo
#   triggering runs, and `workflow_run` grants secret access even when the
#   triggering run itself lacked secrets. So guarding only on
#   conclusion=='failure' && event=='pull_request' is INSUFFICIENT: the
#   head_sha->PR fallback could misattribute a fork run whose head commit collides
#   with an internal PR's SHA to that pipeline-owned internal PR and fire the
#   doorbell (#3475, BLOCKING).
#
#   The FAIL-CLOSED fix is a top-level job guard
#       github.event.workflow_run.head_repository.full_name == github.repository
#   so a fork / spoofed cross-repo run NEVER reaches the token step, plus defense
#   in depth in the head_sha->PR resolution (same-repo AND same-head-SHA) and a
#   re-check of the resolved PR's live head_repo/head_sha before dispatch.
#
# The three checks below are diff-visible: if the head_repository guard is removed
# from the job `if:`, or the token step stops being gated behind that job, or the
# SHA-pin defense-in-depth is dropped, this script RED-fails.
#
#   G1  the `ring` job `if:` contains the fail-closed
#       head_repository.full_name == github.repository condition, AND-only: the
#       expression STRUCTURE must have NO top-level `||` (a `|| true` or any
#       depth-0 OR branch would satisfy the condition without the guard). Parsed,
#       not substring-tested for `&&` — the real condition contains `&&`.
#   G2  the step that consumes REGISTRY_RING_TOKEN / dispatches the registry lives
#       INSIDE the guarded `ring` job (so the guard actually gates the token) and
#       there is no OTHER job that references REGISTRY_RING_TOKEN without the guard.
#   G3  the head_sha->PR resolution `select(...)` filters on BOTH the exact fork-repo
#       predicate (head.repo.full_name == ${REPO}) AND head.sha == ${HEAD_SHA} (the
#       failing run's own SHA, not a constant), and the live-state read re-checks
#       head_repo + head_sha with FAIL-CLOSED `!=` guards that `exit 0` on mismatch
#       BEFORE the dispatch. Operands are matched EXACTLY (semantic, not substring)
#       so dropping the repo predicate, swapping the SHA operand for a constant, or
#       inverting a guard from `!=` to `==` each RED-fails.
#
# NEGATIVE COVERAGE (--self-test, hermetic — no gh/git/network):
#   * fork run (head_repository guard removed)       => G1 (guard present).
#   * top-level `|| true` bypass appended to the if  => G1 (AND-only, no fail-open OR).
#   * same-SHA internal-vs-fork collision            => G3 (head.sha filter rejects).
#   * stale-head (PR head moved off the run's SHA)   => G3 (live head_sha re-check).
#   * fork-repo predicate removed from the fallback  => G3 (fork PR could resolve).
#   * head.sha compared to a CONSTANT not ${HEAD_SHA}=> G3 (SHA-bind bypass).
#   * live `!=` guards inverted to `==` (fail-open)  => G3 (fail-closed inversion).
#   Each fixture is the real workflow with a SURGICAL mutation applied; the check
#   must go RED on every one, proving it is not vacuous. The last four fixtures
#   target the round-2 gaps: G1 previously rejected an OR only when NO `&&` was
#   present (the real if has `&&`, so a `|| true` bypass read clean), and G3
#   substring-matched, so dropping the fork-repo predicate, comparing to a
#   constant, or inverting the live guards all passed.
#
# Usage:
#   check-fast-fix-ring-guard.py              # check the live workflow (default)
#   check-fast-fix-ring-guard.py --root DIR   # use DIR as repo root
#   check-fast-fix-ring-guard.py --self-test  # hermetic negative fixtures
#
# Exit 0 = all clean; exit 1 = one or more offences. Needs PyYAML (installed in the
# docs-quality shared setup, same as the other workflow-lint self-tests).

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "fast-fix-ring.yml"

# The fail-closed head_repository guard, matched tolerant of whitespace but NOT of
# the operands: it must compare workflow_run.head_repository.full_name to
# github.repository. (== only; a `!=` or an `||` context is caught by G1's AND check.)
HEAD_REPO_GUARD_RE = re.compile(
    r"github\.event\.workflow_run\.head_repository\.full_name\s*==\s*github\.repository"
)

# The privileged token / dispatch markers.
RING_TOKEN = "REGISTRY_RING_TOKEN"
DISPATCH_MARKER = "agent-account-registry"
# The exact command that fires the cross-repo doorbell — G3 checks the guards
# come BEFORE this in the run body.
DISPATCH_CMD = "gh workflow run dispatch.yml"

# --- G3 EXACT predicates in the (YAML-parsed) run body -----------------------
# These are matched EXACTLY (operands included), not by loose substring, so that
# a mutation which (a) drops the fork-repo predicate, (b) compares .head.sha to a
# constant instead of ${HEAD_SHA}, or (c) inverts a live fail-closed guard from
# `!=` to `==`, each produces a RED. In the parsed run string the workflow's
# `\\"` source escapes render as a single backslash-quote (`\"`).
#
# 1) head_sha->PR fallback: the `select(...)` must filter on BOTH the fork-repo
#    predicate AND the exact-SHA predicate bound to ${HEAD_SHA}.
G3_FALLBACK_REPO_PREDICATE = r'.head.repo.full_name == \"${REPO}\"'
G3_FALLBACK_SHA_PREDICATE = r'.head.sha == \"${HEAD_SHA}\"'
# 2) live-state defense in depth: each guard must be a FAIL-CLOSED `!=` compare
#    (repo != this repo => skip; head_sha != run's SHA => skip). An inversion to
#    `==` (fire only when they DON'T match) is caught because the `!=` form is gone.
G3_LIVE_REPO_GUARD = '.head_repo\' <<<"$state")" != "${REPO}"'
G3_LIVE_SHA_GUARD = '.head_sha\' <<<"$state")" != "${HEAD_SHA}"'


class Offence(Exception):
    pass


def _has_top_level_or(expr: str) -> bool:
    """True iff `expr` contains a `||` at parenthesis-nesting depth 0.

    A top-level `||` makes the whole condition satisfiable by an alternative
    branch, which would neuter the AND-chained head_repository guard (e.g. a
    trailing `|| true`). We must parse the expression STRUCTURE, not substring-
    test for `&&`: the real condition already contains `&&`, so the old
    `"||" in cond and "&&" not in cond` heuristic reported a top-level bypass
    clean. `||` inside a parenthesised sub-expression is fine (it does not gate
    the whole condition); only a depth-0 `||` is fail-open.
    """
    depth = 0
    i = 0
    n = len(expr)
    quote = ""  # current string-literal delimiter, "" when not inside one
    while i < n:
        c = expr[i]
        if quote:
            # Inside a single- or double-quoted literal: only the matching quote
            # ends it (GitHub expression literals don't use backslash escapes;
            # a doubled quote is an escaped quote — skip it).
            if c == quote:
                if i + 1 < n and expr[i + 1] == quote:
                    i += 1  # escaped quote
                else:
                    quote = ""
        elif c in ("'", '"'):
            quote = c
        elif c == "(":
            depth += 1
        elif c == ")":
            depth = max(0, depth - 1)
        elif c == "|" and i + 1 < n and expr[i + 1] == "|":
            if depth == 0:
                return True
            i += 1  # consume the second '|'
        i += 1
    return False


def _load(text: str) -> dict:
    doc = yaml.safe_load(text)
    if not isinstance(doc, dict):
        raise Offence("workflow is not a YAML mapping")
    return doc


def _ring_job(doc: dict) -> tuple[str, dict]:
    jobs = doc.get("jobs")
    if not isinstance(jobs, dict):
        raise Offence("workflow has no `jobs:` mapping")
    # The ring job is the one that references the token / dispatch. Prefer the id
    # `ring`, but locate by content so a rename can't hide the token.
    ring = None
    for jid, job in jobs.items():
        blob = yaml.safe_dump(job)
        if RING_TOKEN in blob or DISPATCH_MARKER in blob:
            ring = (jid, job)
            break
    if ring is None:
        raise Offence(
            f"no job references {RING_TOKEN}/{DISPATCH_MARKER} — cannot locate the ring job"
        )
    return ring


def check_doc(text: str) -> list[str]:
    """Return a list of offence strings; empty == clean."""
    offences: list[str] = []
    doc = _load(text)
    jobs = doc.get("jobs", {})
    ring_id, ring = _ring_job(doc)

    # ---- G1: the ring job `if:` carries the fail-closed head_repository guard ----
    cond = ring.get("if")
    if not isinstance(cond, str) or not cond.strip():
        offences.append(f"G1: ring job `{ring_id}` has no `if:` guard at all")
    else:
        cond_flat = " ".join(cond.split())
        if not HEAD_REPO_GUARD_RE.search(cond_flat):
            offences.append(
                f"G1: ring job `{ring_id}` `if:` is MISSING the fail-closed "
                f"`workflow_run.head_repository.full_name == github.repository` guard "
                f"(a fork / same-SHA cross-repo run could reach the token step) — got: {cond_flat!r}"
            )
        elif _has_top_level_or(cond_flat):
            # The guard must be ANDed into the condition. A top-level `||`
            # (e.g. a trailing `|| true`, or `... || github.actor == 'x'`) makes
            # the whole condition satisfiable by an alternative branch, which
            # neuters the head_repository guard — REJECT it regardless of whether
            # a `&&` is also present. (A `||` nested inside parentheses does not
            # gate the whole condition and is allowed.)
            offences.append(
                f"G1: ring job `{ring_id}` `if:` has a top-level `||` — the "
                f"head_repository guard must be ANDed (no fail-open OR branch) so "
                f"it is fail-closed; got: {cond_flat!r}"
            )

    # ---- G2: the token step lives INSIDE the guarded ring job; no OTHER job uses it ----
    ring_blob = yaml.safe_dump(ring)
    if RING_TOKEN not in ring_blob:
        offences.append(
            f"G2: the guarded ring job `{ring_id}` does not reference {RING_TOKEN} — "
            f"the token step is not gated behind the head_repository guard"
        )
    for jid, job in jobs.items():
        if jid == ring_id:
            continue
        blob = yaml.safe_dump(job)
        if RING_TOKEN in blob or DISPATCH_MARKER in blob:
            jcond = job.get("if")
            jflat = " ".join(jcond.split()) if isinstance(jcond, str) else ""
            if not HEAD_REPO_GUARD_RE.search(jflat):
                offences.append(
                    f"G2: job `{jid}` also references the ring token/dispatch but is "
                    f"NOT behind the head_repository guard — a second unguarded path to the token"
                )

    # ---- G3: SHA/repo defense in depth in the resolution + live-state bash ----
    # Gather the ring job's run-step bodies.
    run_bodies = []
    for step in ring.get("steps", []) or []:
        if isinstance(step, dict) and isinstance(step.get("run"), str):
            run_bodies.append(step["run"])
    body = "\n".join(run_bodies)
    if not body:
        offences.append(f"G3: ring job `{ring_id}` has no `run:` body to inspect")
    else:
        # Semantic (not substring) matching: each predicate is matched WITH its
        # operands, so a mutation that drops the fork-repo predicate, swaps the
        # SHA operand for a constant, or inverts a live guard from `!=` to `==`
        # each produces a RED.
        #
        # (a) head_sha->PR fallback: fork-repo predicate must be present.
        if G3_FALLBACK_REPO_PREDICATE not in body:
            offences.append(
                "G3: the head_sha->PR resolution `select(...)` is MISSING the fork-repo "
                f"predicate `{G3_FALLBACK_REPO_PREDICATE}` — a fork PR sharing the run's "
                "head SHA could resolve to the pipeline (same-repo constraint dropped)"
            )
        # (b) head_sha->PR fallback: SHA predicate must compare to ${HEAD_SHA},
        #     NOT a constant. Matching the exact operand catches a constant swap.
        if G3_FALLBACK_SHA_PREDICATE not in body:
            offences.append(
                "G3: the head_sha->PR resolution `select(...)` does not pin "
                f"`{G3_FALLBACK_SHA_PREDICATE}` — the head.sha filter must compare to the "
                "failing run's ${HEAD_SHA} (not a constant/other value), else the same-SHA-"
                "collision defense in depth is bypassed"
            )
        # (c) live-state defense in depth: BOTH guards must be the FAIL-CLOSED
        #     `!=` form (mismatch => skip). An inversion to `==` removes the `!=`
        #     spelling and is caught here; each guard must also precede dispatch.
        for label, needle in (
            ("head_repo", G3_LIVE_REPO_GUARD),
            ("head_sha", G3_LIVE_SHA_GUARD),
        ):
            pos = body.find(needle)
            if pos < 0:
                offences.append(
                    f"G3: the live-PR-state {label} re-check is MISSING its fail-closed "
                    f"`{needle}` guard — either the guard was dropped or inverted from `!=` "
                    "to `==` (which would fire ONLY on a mismatch — fail-OPEN)"
                )
                continue
            # The guard block must skip (exit 0) on mismatch, and run before the
            # privileged dispatch command.
            block_tail = body[pos : pos + 400]
            if "exit 0" not in block_tail:
                offences.append(
                    f"G3: the live-PR-state {label} `!=` guard does not `exit 0` on "
                    "mismatch — it must fail-closed (skip the dispatch), not fall through"
                )
            disp = body.find(DISPATCH_CMD)
            if disp >= 0 and pos > disp:
                offences.append(
                    f"G3: the live-PR-state {label} `!=` guard appears AFTER the "
                    f"`{DISPATCH_CMD}` doorbell — the guard must gate BEFORE the dispatch"
                )

    return offences


# --------------------------------------------------------------------------- #
#  --self-test: hermetic negative fixtures                                    #
# --------------------------------------------------------------------------- #

def _self_test() -> int:
    text = WORKFLOW.read_text(encoding="utf-8")

    failures = 0

    def expect_clean(label: str, t: str) -> None:
        nonlocal failures
        offs = check_doc(t)
        if offs:
            failures += 1
            print(f"  [FAIL] {label}: expected CLEAN but got offences:")
            for o in offs:
                print(f"           - {o}")
        else:
            print(f"  [ok]   {label}: clean (as expected)")

    def expect_red(label: str, t: str, needle: str) -> None:
        nonlocal failures
        offs = check_doc(t)
        if not any(needle in o for o in offs):
            failures += 1
            print(
                f"  [FAIL] {label}: expected an offence containing {needle!r} but got: {offs}"
            )
        else:
            print(f"  [ok]   {label}: correctly RED ({needle})")

    # 0) the real, live workflow must be clean.
    expect_clean("live workflow", text)

    # 1) FORK RUN: remove the head_repository guard from the job `if:`. A fork /
    #    cross-repo run whose head SHA collides with an internal PR would no longer
    #    be rejected at the job level. => G1 must RED.
    no_guard = re.sub(
        r"\n\s*&&\s*github\.event\.workflow_run\.head_repository\.full_name\s*==\s*github\.repository",
        "",
        text,
    )
    assert no_guard != text, "fixture-1 did not remove the guard — check the regex"
    expect_red("fork run (head_repository guard removed)", no_guard, "G1")

    # 2) SAME-SHA internal-vs-fork collision: drop the `.head.sha ==` filter from the
    #    head_sha->PR resolution. A fork commit that shares an internal PR's SHA could
    #    resolve to the internal PR via the pulls-at-SHA lookup. => G3 must RED.
    no_sha_filter = text.replace(
        'and .head.sha == \\"${HEAD_SHA}\\" and .state', "and .state"
    )
    assert no_sha_filter != text, "fixture-2 did not remove the head.sha filter"
    expect_red(
        "same-SHA collision (head.sha filter removed)", no_sha_filter, "G3"
    )

    # 3) STALE-HEAD: remove the live-state head_sha re-check before dispatch. A PR
    #    whose head moved off the run's SHA between the trigger and here would still
    #    ring. => G3 must RED (the live re-check needle is gone).
    #    Remove BOTH the head_sha field from the jq projection AND the guard block.
    no_live_recheck = re.sub(
        r'head_sha: \.head\.sha, ', "", text
    )
    no_live_recheck = re.sub(
        r"          if \[ \"\$\(jq -r '\.head_sha' <<<\"\$state\"\)\" != \"\$\{HEAD_SHA\}\" \]; then\n"
        r".*?\n            exit 0\n          fi\n",
        "",
        no_live_recheck,
        flags=re.DOTALL,
    )
    assert no_live_recheck != text, "fixture-3 did not remove the live head_sha re-check"
    expect_red("stale-head (live head_sha re-check removed)", no_live_recheck, "G3")

    # --- round-2 findings: the checker was too loose; these four fixtures pin it ---

    # 4) [G1 finding] TOP-LEVEL `|| true` BYPASS. The real if already contains `&&`,
    #    so the old `"||" in cond and "&&" not in cond` heuristic read a trailing
    #    `|| true` clean — which makes the whole condition true regardless of the
    #    head_repository guard. Append a depth-0 `|| true`. => G1 must RED.
    or_bypass = text.replace(
        "&& github.event.workflow_run.head_repository.full_name == github.repository\n",
        "&& github.event.workflow_run.head_repository.full_name == github.repository\n"
        "      || true\n",
        1,
    )
    assert or_bypass != text, "fixture-4 did not append the `|| true` bypass"
    expect_red("top-level `|| true` bypass appended", or_bypass, "G1")

    # 5) [G3 finding] REMOVE THE FORK-REPO PREDICATE from the head_sha->PR fallback
    #    `select(...)`. Without `head.repo.full_name == ${REPO}` a fork PR sharing the
    #    run's head SHA can resolve to the pipeline. The old `.head.repo.full_name`
    #    substring test also matched the live-state jq projection, so it passed. => G3.
    no_repo_predicate = text.replace(
        'select(.head.repo.full_name == \\"${REPO}\\" and .head.sha == \\"${HEAD_SHA}\\" '
        'and .state == \\"open\\")',
        'select(.head.sha == \\"${HEAD_SHA}\\" and .state == \\"open\\")',
        1,
    )
    assert no_repo_predicate != text, "fixture-5 did not remove the fork-repo predicate"
    expect_red("fallback fork-repo predicate removed", no_repo_predicate, "G3")

    # 6) [G3 finding] COMPARE .head.sha TO A CONSTANT instead of ${HEAD_SHA}. The old
    #    `.head.sha ==` substring test ignored the operand, so binding the filter to a
    #    fixed value (which never matches the real run SHA / breaks the SHA binding)
    #    passed. => G3 must RED.
    sha_constant = text.replace(
        '.head.sha == \\"${HEAD_SHA}\\" and .state',
        '.head.sha == \\"deadbeefdeadbeefdeadbeefdeadbeefdeadbeef\\" and .state',
        1,
    )
    assert sha_constant != text, "fixture-6 did not swap the head.sha operand"
    expect_red("fallback head.sha compared to a constant", sha_constant, "G3")

    # 7) [G3 finding] INVERT THE LIVE `!=` GUARDS TO `==` (fail-OPEN: the guard would
    #    now `exit 0` only when the repo/SHA MATCH, ringing on every mismatch/collision
    #    instead). The old `head_repo`/`head_sha` and `jq -r '.head_repo'` substring
    #    tests were still present after the flip, so it passed. => G3 must RED.
    inverted_guards = text.replace(
        'if [ "$(jq -r \'.head_repo\' <<<"$state")" != "${REPO}" ]; then',
        'if [ "$(jq -r \'.head_repo\' <<<"$state")" == "${REPO}" ]; then',
        1,
    ).replace(
        'if [ "$(jq -r \'.head_sha\' <<<"$state")" != "${HEAD_SHA}" ]; then',
        'if [ "$(jq -r \'.head_sha\' <<<"$state")" == "${HEAD_SHA}" ]; then',
        1,
    )
    assert inverted_guards != text, "fixture-7 did not invert the live guards"
    expect_red("live `!=` guards inverted to `==` (fail-open)", inverted_guards, "G3")

    if failures:
        print(f"\nSELF-TEST FAILED: {failures} case(s) did not behave as expected.")
        return 1
    print("\nSELF-TEST PASSED: guard present + all seven negative fixtures RED.")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", type=Path, default=None, help="repo root override")
    ap.add_argument(
        "--self-test", action="store_true", help="run hermetic negative fixtures"
    )
    args = ap.parse_args()

    if args.self_test:
        return _self_test()

    wf = WORKFLOW
    if args.root is not None:
        wf = args.root / ".github" / "workflows" / "fast-fix-ring.yml"
    if not wf.exists():
        print(f"error: {wf} not found", file=sys.stderr)
        return 1

    try:
        offences = check_doc(wf.read_text(encoding="utf-8"))
    except Offence as e:
        print(f"error: {e}", file=sys.stderr)
        return 1

    if offences:
        print("fast-fix-ring cross-repo guard check FAILED (#3475):", file=sys.stderr)
        for o in offences:
            print(f"  - {o}", file=sys.stderr)
        return 1
    print("fast-fix-ring cross-repo guard check: OK (#3475 fail-closed guard present).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
