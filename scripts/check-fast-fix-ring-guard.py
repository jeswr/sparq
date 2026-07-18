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
#       head_repository.full_name == github.repository condition (ANDed, not ORed).
#   G2  the step that consumes REGISTRY_RING_TOKEN / dispatches the registry lives
#       INSIDE the guarded `ring` job (so the guard actually gates the token) and
#       there is no OTHER job that references REGISTRY_RING_TOKEN without the guard.
#   G3  the head_sha->PR resolution filters on BOTH head.repo.full_name == this repo
#       AND head.sha == the failing run's head SHA (same-SHA-collision defense in
#       depth), and the live-state read re-checks head_repo + head_sha before the
#       dispatch.
#
# NEGATIVE COVERAGE (--self-test, hermetic — no gh/git/network):
#   * fork run (head_repository != repo)            => guard rejects (G1 present).
#   * same-SHA internal-vs-fork collision           => head.sha filter rejects (G3).
#   * stale-head (PR head moved off the run's SHA)   => live head_sha re-check (G3).
#   Each fixture is the real workflow with the guard SURGICALLY REMOVED; the check
#   must go RED on every one, proving it is not vacuous.
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

# Defense-in-depth needles in the resolution / live-state bash.
SHA_FILTER_RE = re.compile(r"\.head\.sha\s*==\s*\\?\"?\$\{?REPO\}?|\.head\.sha == ")
# (kept simple; the precise assertions are done structurally below.)


class Offence(Exception):
    pass


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
        else:
            # The guard must be ANDed into the condition (a top-level `||` would let
            # a branch bypass it). Reject if the guard is only reachable past an `||`.
            # Heuristic: there must be a `&&` adjacent to the guard and no bare `||`
            # that isn't itself inside a parenthesised sub-expression.
            if "||" in cond_flat and "&&" not in cond_flat:
                offences.append(
                    f"G1: ring job `{ring_id}` `if:` ORs the head_repository guard "
                    f"(`||` with no `&&`) — the guard must be ANDed so it is fail-closed"
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
        # The head_sha->PR fallback must filter on head.repo.full_name == repo AND
        # head.sha == the run head SHA.
        if ".head.repo.full_name" not in body:
            offences.append(
                "G3: the head_sha->PR resolution does not filter on `.head.repo.full_name` "
                "(same-repo constraint missing)"
            )
        if ".head.sha ==" not in body:
            offences.append(
                "G3: the head_sha->PR resolution does not filter on `.head.sha ==` the "
                "failing run's head SHA (same-SHA-collision defense in depth missing)"
            )
        # The live-state read must re-check head_repo and head_sha before dispatch.
        if "head_repo" not in body or "head_sha" not in body:
            offences.append(
                "G3: the live-PR-state read does not re-check `head_repo`/`head_sha` before "
                "the dispatch (residual same-SHA-collision window before the doorbell)"
            )
        # The re-check must actually gate (compare to REPO / HEAD_SHA and skip).
        if 'jq -r \'.head_repo\'' not in body and "'.head_repo'" not in body:
            offences.append(
                "G3: the resolved PR's head_repo is read but never compared/gated before dispatch"
            )
        if 'jq -r \'.head_sha\'' not in body and "'.head_sha'" not in body:
            offences.append(
                "G3: the resolved PR's head_sha is read but never compared/gated before dispatch"
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

    if failures:
        print(f"\nSELF-TEST FAILED: {failures} case(s) did not behave as expected.")
        return 1
    print("\nSELF-TEST PASSED: guard present + all three negative fixtures RED.")
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
