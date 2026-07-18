#!/usr/bin/env python3
# [OPUS-4.8] sparq-org/sparq#3475 — fail-closed REGRESSION guard for the fast-fix
# ring's cross-repo trust boundary. 🤖 SPARQ agent. Authored by Opus 4.8 (Fable
# routed off security surfaces by design; flag for re-review when Fable returns).
#
# =============================================================================
# THREAT MODEL — read this first; it fixes the bar this checker is judged against
# =============================================================================
# This script is a REGRESSION DETECTOR. Its job is to catch an ACCIDENTAL or an
# agent-introduced WEAKENING of the fast-fix ring's cross-repo trust guard when a
# change touches `.github/workflows/fast-fix-ring.yml` — a change that a HUMAN then
# ARMS (workflow edits are human-armed per repo policy). It is NOT, and cannot be, a
# defense against an actor who ALREADY HAS repo-write and DELIBERATELY edits the
# workflow to exfiltrate the secret: such an actor could weaken the guard AND update
# this checker's golden in the same commit, or exfiltrate REGISTRY_RING_TOKEN by many
# other means (a brand-new workflow, a different job, an exfiltrating step). Defending
# against a malicious committer is OUT OF SCOPE here and is handled elsewhere —
# branch protection, required human review of `.github/workflows/**` changes, and the
# LIVE security control itself, which is the workflow's own
#     github.event.workflow_run.head_repository.full_name == github.repository
# job guard plus the SHA/repo defense-in-depth. THIS script does not enforce security
# at runtime; the WORKFLOW does. This script only makes an accidental regression to
# that guard LOUD in review.
#
# =============================================================================
# STRATEGY — CANONICAL GOLDEN MATCH (not predicate parsing)
# =============================================================================
# Rounds 1–3 of adversarial review proved that a predicate-PARSING validator cannot
# win: for every "reject a top-level ||" rule there is a `(A && B && guard || true)`
# paren-wrap; for every "the guard regex is present" rule there is a `!(guard)`
# negation; for every "both needles are present" G3 substring rule there is a
# conjunction→disjunction `repo OR SHA` swap (both needles survive) or a
# predicate-moved-into-a-comment. Parsing a moving target is a losing regress.
#
# So this checker does NOT parse the security meaning of the condition. It pins the
# EXACT, KNOWN-GOOD CANONICAL TEXT of each security-load-bearing fragment and requires
# the workflow's ACTUAL (normalized) fragment to be BYTE-EQUAL to that golden:
#
#   G1  the ring job's full `if:` condition, whitespace-normalized, must EXACTLY
#       equal GOLDEN_IF. Any OR-wrap, negation, operand swap, reorder, or extra
#       clause changes the normalized text ⇒ mismatch ⇒ RED. You cannot smuggle a
#       `|| true`, a `!(...)`, or `(... || true)` past an exact-string comparison.
#   G2  the privileged token/dispatch step lives INSIDE that one guarded job, and no
#       OTHER job's normalized `if:` differs from GOLDEN_IF while touching the token.
#   G3  the head_sha->PR resolution `select(...)` jq filter, normalized, must EXACTLY
#       equal GOLDEN_SELECT (a CONJUNCTION of the fork-repo predicate AND the exact
#       ${HEAD_SHA} predicate AND state==open — a `repo OR SHA` swap changes the text
#       ⇒ RED); and the two live fail-closed guard blocks — each a
#       `if <!= test>; then … exit 0; fi` skeleton — normalized, must EXACTLY equal
#       their goldens (an inversion to `==` or a moved-to-comment predicate changes
#       the executable skeleton ⇒ RED), and each must appear BEFORE the dispatch.
#
# WHY EXACT-MATCH IS STRICTLY STRONGER than the predicate parser it replaces: the
# adversary's whole bypass class works by producing text that is SEMANTICALLY unsafe
# but PARSES to the same predicate-presence signal. Against a canonical golden there is
# no such gap: any text that differs from the known-good form — safe or unsafe — reds.
# The tradeoff is deliberate: a LEGITIMATE refactor of the guard (rename ${REPO},
# restructure the select, reword a skip message) will red too, and the fix is to
# UPDATE THE GOLDEN IN THE SAME PR. That is the CORRECT fail-safe direction — a human /
# reviewer sees the golden change and re-confirms the guard is still sound, rather than
# a weakening sliding through on a predicate technicality. (§ headers/echo-message
# wording that is NOT security-load-bearing is normalized away so pure prose edits do
# not force a golden bump; see _canon_guard_block.)
#
# NEGATIVE COVERAGE (--self-test, hermetic — no gh/git/network): the live workflow
# passes, and each of the round-1/2/3 bypasses, applied as a SURGICAL mutation to a
# copy of the real workflow, goes RED. See _self_test for the mutation→red table.
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

RING_TOKEN = "REGISTRY_RING_TOKEN"
DISPATCH_MARKER = "agent-account-registry"
# The exact command that fires the cross-repo doorbell — G3 checks the guards
# come BEFORE this in the run body.
DISPATCH_CMD = "gh workflow run dispatch.yml"

# =============================================================================
# GOLDEN CANONICAL FORMS
# =============================================================================
# These are the known-good, security-load-bearing fragments of
# .github/workflows/fast-fix-ring.yml. The checker normalizes the workflow's actual
# fragment (collapse whitespace; for guard blocks, strip the non-load-bearing echo
# message) and requires EXACT EQUALITY. If the guard is legitimately refactored, bump
# the matching golden IN THE SAME PR — that is the intended review checkpoint.

# G1: the ring job's full `if:` condition, whitespace-collapsed. Three ANDed clauses;
# the third is the #3475 fail-closed head_repository guard. Exact-match, so an OR-wrap
# `(... || true)`, a `!(...)` negation, an operand swap, a reorder, or any extra
# clause all differ from this string and RED.
GOLDEN_IF = (
    "github.event.workflow_run.conclusion == 'failure' "
    "&& github.event.workflow_run.event == 'pull_request' "
    "&& github.event.workflow_run.head_repository.full_name == github.repository"
)

# G3(a): the head_sha->PR fallback jq `select(...)` filter, normalized (whitespace
# collapsed). A CONJUNCTION (`and`) of the fork-repo predicate, the exact-${HEAD_SHA}
# predicate, and state==open. A `repo OR SHA` disjunction, a dropped predicate, or a
# constant-swapped SHA all change this string and RED. (In the YAML-parsed run body
# the workflow's `\\"` source escapes render as a single backslash-quote `\"`.)
GOLDEN_SELECT = (
    'select(.head.repo.full_name == \\"${REPO}\\" '
    'and .head.sha == \\"${HEAD_SHA}\\" '
    'and .state == \\"open\\")'
)

# G3(b): the two live fail-closed re-check guard blocks, reduced to their EXECUTABLE
# skeleton (the `if [ <test> ]; then` line + the `exit 0` fail-closed body), with the
# non-load-bearing echo message removed. Each must be a `!=` (fail-closed: mismatch =>
# skip) compare and must `exit 0`. An inversion to `==`, a dropped predicate, or a
# predicate that survives ONLY in a comment changes the executable skeleton and REDs.
GOLDEN_LIVE_REPO_GUARD = (
    'if [ "$(jq -r \'.head_repo\' <<<"$state")" != "${REPO}" ]; then '
    "exit 0 fi"
)
GOLDEN_LIVE_SHA_GUARD = (
    'if [ "$(jq -r \'.head_sha\' <<<"$state")" != "${HEAD_SHA}" ]; then '
    "exit 0 fi"
)


class Offence(Exception):
    pass


def _canon(s: str) -> str:
    """Whitespace-collapse: any run of whitespace (incl. newlines) => one space,
    stripped. Used for the `if:` condition and the jq select filter, where only
    whitespace/line-fold differences are non-load-bearing."""
    return " ".join(s.split())


def _strip_shell_comments(body: str) -> str:
    """Drop full-line shell comments so a `select(...)` (or any predicate) demoted to
    a `# ...` comment is NOT mistaken for the executable filter. Only whole-line
    comments are stripped (a leading `#` after optional whitespace); we do not attempt
    to strip trailing inline comments, which would require full shell tokenization and
    is unnecessary — the executable predicates in this workflow are never followed by
    an inline `#`."""
    return "\n".join(
        line for line in body.splitlines() if not line.lstrip().startswith("#")
    )


def _extract_select(body: str) -> str | None:
    """Return the whitespace-normalized `select( ... )` jq filter from the EXECUTABLE
    run body (shell comments stripped), or None if absent. Balanced-paren scan so
    nested `()` inside the filter are kept. Stripping comments first means a fork-repo
    predicate demoted into a `#`-comment is not read as the live filter."""
    body = _strip_shell_comments(body)
    idx = body.find("select(")
    if idx < 0:
        return None
    start = idx + len("select(")
    depth = 1
    i = start
    while i < len(body) and depth > 0:
        c = body[i]
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
        i += 1
    if depth != 0:
        return None
    return _canon("select(" + body[start : i - 1] + ")")


def _canon_guard_block(body: str, jq_field: str) -> str | None:
    """Extract the live re-check guard block keyed on `jq_field` (`.head_repo` /
    `.head_sha`) and reduce it to its EXECUTABLE, load-bearing skeleton:
    the `if [ … ]; then` test line + the fail-closed `exit 0`, joined by single
    spaces, with the (non-security) echo message line dropped.

    Returns None if no `if` line referencing that field is present. Because the
    skeleton keeps the `!=`/`==` operator and the compared operands but drops prose,
    an inversion, a dropped operand, or a predicate that survives only as a `#`
    comment all change (or empty) the skeleton and fail the golden match, while a
    reworded skip message does not force a golden bump."""
    lines = body.splitlines()
    start = None
    for i, line in enumerate(lines):
        st = line.strip()
        if st.startswith("if [") and jq_field in st and st.endswith("then"):
            start = i
            break
    if start is None:
        return None
    # Walk to the matching `fi`, keeping only the load-bearing statements.
    kept: list[str] = []
    for j in range(start, len(lines)):
        st = lines[j].strip()
        if not st or st.startswith("#"):
            continue
        if st.startswith("echo "):
            # non-load-bearing skip message
            continue
        kept.append(st)
        if st == "fi":
            break
    else:
        return None
    return _canon(" ".join(kept))


def _load(text: str) -> dict:
    doc = yaml.safe_load(text)
    if not isinstance(doc, dict):
        raise Offence("workflow is not a YAML mapping")
    return doc


def _ring_job(doc: dict) -> tuple[str, dict]:
    jobs = doc.get("jobs")
    if not isinstance(jobs, dict):
        raise Offence("workflow has no `jobs:` mapping")
    # The ring job is the one that references the token / dispatch. Locate by content
    # so a rename cannot hide the token.
    for jid, job in jobs.items():
        blob = yaml.safe_dump(job)
        if RING_TOKEN in blob or DISPATCH_MARKER in blob:
            return (jid, job)
    raise Offence(
        f"no job references {RING_TOKEN}/{DISPATCH_MARKER} — cannot locate the ring job"
    )


def check_doc(text: str) -> list[str]:
    """Return a list of offence strings; empty == clean. Canonical golden match."""
    offences: list[str] = []
    doc = _load(text)
    jobs = doc.get("jobs", {})
    ring_id, ring = _ring_job(doc)

    # ---- G1: the ring job `if:` must EXACTLY equal the golden condition ----
    cond = ring.get("if")
    if not isinstance(cond, str) or not cond.strip():
        offences.append(f"G1: ring job `{ring_id}` has no `if:` guard at all")
    else:
        cond_flat = _canon(cond)
        if cond_flat != GOLDEN_IF:
            offences.append(
                f"G1: ring job `{ring_id}` `if:` does not match the canonical golden "
                f"guard. Any change — an OR-wrap `(... || true)`, a `!(...)` negation, "
                f"an operand swap, a reorder, or an added clause — reds here because the "
                f"normalized text differs from the known-good condition. If this is a "
                f"legitimate refactor, update GOLDEN_IF in this PR.\n"
                f"       expected: {GOLDEN_IF!r}\n"
                f"       actual:   {cond_flat!r}"
            )

    # ---- G2: the token step is INSIDE the guarded ring job; no OTHER job uses it ----
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
            jflat = _canon(jcond) if isinstance(jcond, str) else ""
            if jflat != GOLDEN_IF:
                offences.append(
                    f"G2: job `{jid}` also references the ring token/dispatch but its "
                    f"`if:` is not the canonical golden guard — a second path to the "
                    f"token that is not fail-closed on head_repository"
                )

    # ---- G3: the resolution + live-state guards must match their goldens ----
    run_bodies = []
    for step in ring.get("steps", []) or []:
        if isinstance(step, dict) and isinstance(step.get("run"), str):
            run_bodies.append(step["run"])
    body = "\n".join(run_bodies)
    if not body:
        offences.append(f"G3: ring job `{ring_id}` has no `run:` body to inspect")
    else:
        # (a) head_sha->PR fallback: the executable `select(...)` filter must EXACTLY
        #     equal the golden conjunction. A `repo OR SHA` swap, a dropped predicate,
        #     or a constant-swapped SHA all change this string and RED. (A predicate
        #     retained only in a comment does not appear in the parsed jq filter, so
        #     the select still differs from the golden.)
        sel = _extract_select(body)
        if sel is None:
            offences.append(
                "G3: the head_sha->PR resolution `select(...)` jq filter is MISSING — "
                "the fork-repo AND exact-${HEAD_SHA} conjunction cannot be verified"
            )
        elif sel != GOLDEN_SELECT:
            offences.append(
                "G3: the head_sha->PR resolution `select(...)` does not match the "
                "canonical golden filter. It must be the CONJUNCTION "
                "`head.repo.full_name == ${REPO} AND head.sha == ${HEAD_SHA} AND "
                "state == open` — a `repo OR SHA` disjunction, a dropped predicate, or "
                "a constant-swapped SHA all differ from the golden and red. If this is "
                "a legitimate refactor, update GOLDEN_SELECT in this PR.\n"
                f"       expected: {GOLDEN_SELECT!r}\n"
                f"       actual:   {sel!r}"
            )

        # (b) live-state defense in depth: each fail-closed `!=` guard block must
        #     match its golden EXECUTABLE skeleton, and precede the dispatch. An
        #     inversion to `==`, a dropped operand, or a predicate demoted to a
        #     comment changes (or empties) the skeleton and reds.
        for label, jq_field, golden in (
            ("head_repo", ".head_repo", GOLDEN_LIVE_REPO_GUARD),
            ("head_sha", ".head_sha", GOLDEN_LIVE_SHA_GUARD),
        ):
            block = _canon_guard_block(body, jq_field)
            if block is None:
                offences.append(
                    f"G3: the live-PR-state {label} re-check `if` block is MISSING — "
                    f"the fail-closed `!=` guard was dropped (or its predicate now "
                    f"survives only in a comment, which does not execute)"
                )
                continue
            if block != golden:
                offences.append(
                    f"G3: the live-PR-state {label} re-check does not match the "
                    f"canonical golden guard. It must be the FAIL-CLOSED `!=` compare "
                    f"that `exit 0`s on mismatch — an inversion to `==`, a dropped "
                    f"operand, or a comment-only predicate all differ from the golden "
                    f"and red. If this is a legitimate refactor, update the golden in "
                    f"this PR.\n"
                    f"       expected: {golden!r}\n"
                    f"       actual:   {block!r}"
                )
            # Ordering: the guard block must appear BEFORE the privileged dispatch.
            pos = body.find(f"if [ \"$(jq -r '{jq_field}'")
            disp = body.find(DISPATCH_CMD)
            if pos >= 0 and disp >= 0 and pos > disp:
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
        if t == text:
            failures += 1
            print(f"  [FAIL] {label}: MUTATION DID NOT APPLY (fixture == live workflow)")
            return
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

    # --- round-1 fixtures: a guard is REMOVED ---

    # 1) FORK RUN: remove the head_repository guard from the job `if:`. => G1.
    no_guard = re.sub(
        r"\n\s*&&\s*github\.event\.workflow_run\.head_repository\.full_name\s*==\s*github\.repository",
        "",
        text,
    )
    expect_red("R1 fork run (head_repository guard removed)", no_guard, "G1")

    # 2) SAME-SHA collision: drop the `.head.sha ==` filter from the fallback. => G3.
    no_sha_filter = text.replace(
        'and .head.sha == \\"${HEAD_SHA}\\" and .state', "and .state"
    )
    expect_red("R1 same-SHA collision (head.sha filter removed)", no_sha_filter, "G3")

    # 3) STALE-HEAD: remove the live head_sha re-check block before dispatch. => G3.
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
    expect_red("R1 stale-head (live head_sha re-check removed)", no_live_recheck, "G3")

    # --- round-2 fixtures: predicate-parser was too loose ---

    # 4) TOP-LEVEL `|| true` appended to the if (bare, depth-0). => G1.
    or_bypass = text.replace(
        "&& github.event.workflow_run.head_repository.full_name == github.repository\n",
        "&& github.event.workflow_run.head_repository.full_name == github.repository\n"
        "      || true\n",
        1,
    )
    expect_red("R2 top-level `|| true` bypass appended", or_bypass, "G1")

    # 5) fork-repo predicate removed from the fallback `select(...)`. => G3.
    no_repo_predicate = text.replace(
        'select(.head.repo.full_name == \\"${REPO}\\" and .head.sha == \\"${HEAD_SHA}\\" '
        'and .state == \\"open\\")',
        'select(.head.sha == \\"${HEAD_SHA}\\" and .state == \\"open\\")',
        1,
    )
    expect_red("R2 fallback fork-repo predicate removed", no_repo_predicate, "G3")

    # 6) .head.sha compared to a CONSTANT instead of ${HEAD_SHA}. => G3.
    sha_constant = text.replace(
        '.head.sha == \\"${HEAD_SHA}\\" and .state',
        '.head.sha == \\"deadbeefdeadbeefdeadbeefdeadbeefdeadbeef\\" and .state',
        1,
    )
    expect_red("R2 fallback head.sha compared to a constant", sha_constant, "G3")

    # 7) live `!=` guards inverted to `==` (fail-open). => G3.
    inverted_guards = text.replace(
        'if [ "$(jq -r \'.head_repo\' <<<"$state")" != "${REPO}" ]; then',
        'if [ "$(jq -r \'.head_repo\' <<<"$state")" == "${REPO}" ]; then',
        1,
    ).replace(
        'if [ "$(jq -r \'.head_sha\' <<<"$state")" != "${HEAD_SHA}" ]; then',
        'if [ "$(jq -r \'.head_sha\' <<<"$state")" == "${HEAD_SHA}" ]; then',
        1,
    )
    expect_red("R2 live `!=` guards inverted to `==` (fail-open)", inverted_guards, "G3")

    # --- round-3 fixtures: the exact bypasses codex demonstrated against the parser ---

    # 8) [R3] PARENTHESIZED OR-WRAP of the whole condition: actionlint-valid,
    #    (failure && event && head_repo == repo || true). The old paren-depth
    #    scanner treated a `||` inside parens as safe. Golden exact-match reds.
    paren_or_wrap = text.replace(
        "    if: >-\n"
        "      github.event.workflow_run.conclusion == 'failure'\n"
        "      && github.event.workflow_run.event == 'pull_request'\n"
        "      && github.event.workflow_run.head_repository.full_name == github.repository\n",
        "    if: >-\n"
        "      (github.event.workflow_run.conclusion == 'failure'\n"
        "      && github.event.workflow_run.event == 'pull_request'\n"
        "      && github.event.workflow_run.head_repository.full_name == github.repository\n"
        "      || true)\n",
        1,
    )
    expect_red("R3 parenthesized `(... || true)` OR-wrap of the whole if", paren_or_wrap, "G1")

    # 9) [R3] NEGATED equality `!(head_repo == repo)`: passed the old presence regex.
    #    Golden exact-match reds (text differs).
    negated_guard = text.replace(
        "&& github.event.workflow_run.head_repository.full_name == github.repository",
        "&& !(github.event.workflow_run.head_repository.full_name == github.repository)",
        1,
    )
    expect_red("R3 negated `!(head_repo == repo)` guard", negated_guard, "G1")

    # 10) [R3] G3 CONJUNCTION->DISJUNCTION: `repo AND sha` -> `repo OR sha` in the
    #     fallback select. Both exact needles survived the old substring test.
    #     Golden exact-match reds (`and`->`or` changes the normalized filter).
    or_conjunction = text.replace(
        '.head.repo.full_name == \\"${REPO}\\" and .head.sha == \\"${HEAD_SHA}\\"',
        '.head.repo.full_name == \\"${REPO}\\" or .head.sha == \\"${HEAD_SHA}\\"',
        1,
    )
    expect_red("R3 fallback conjunction->disjunction (repo OR sha)", or_conjunction, "G3")

    # 11) [R3] PREDICATE MOVED TO A COMMENT: the executable select drops the fork-repo
    #     predicate but a `#`-comment line retains the needle text. The old substring
    #     test matched the comment. The golden matches the PARSED filter only, so the
    #     executable select differs and reds.
    predicate_in_comment = text.replace(
        '            if ! pulls="$(gh api "repos/${REPO}/commits/${HEAD_SHA}/pulls" \\\n'
        '                --jq "[.[] | select(.head.repo.full_name == \\"${REPO}\\" and .head.sha == \\"${HEAD_SHA}\\" and .state == \\"open\\")] | .[0].number // empty")"; then\n',
        '            # select(.head.repo.full_name == \\"${REPO}\\" and .head.sha == \\"${HEAD_SHA}\\" and .state == \\"open\\")\n'
        '            if ! pulls="$(gh api "repos/${REPO}/commits/${HEAD_SHA}/pulls" \\\n'
        '                --jq "[.[] | select(.head.sha == \\"${HEAD_SHA}\\" and .state == \\"open\\")] | .[0].number // empty")"; then\n',
        1,
    )
    expect_red("R3 fork-repo predicate demoted to a comment (select drops it)", predicate_in_comment, "G3")

    # 12) [R3] INVERTED LIVE GUARD accompanied by a comment-only expected needle: the
    #     executable `!=` is flipped to `==` while a `#`-comment carries the `!=` text.
    #     The old substring test found the `!=` in the comment. The golden matches the
    #     executable skeleton (comments stripped), so the inversion reds.
    inverted_with_comment = text.replace(
        '          if [ "$(jq -r \'.head_repo\' <<<"$state")" != "${REPO}" ]; then\n',
        '          # if [ "$(jq -r \'.head_repo\' <<<"$state")" != "${REPO}" ]; then\n'
        '          if [ "$(jq -r \'.head_repo\' <<<"$state")" == "${REPO}" ]; then\n',
        1,
    )
    expect_red("R3 live guard inverted with comment-only `!=` needle", inverted_with_comment, "G3")

    if failures:
        print(f"\nSELF-TEST FAILED: {failures} case(s) did not behave as expected.")
        return 1
    print(
        "\nSELF-TEST PASSED: live workflow clean + all round-1/2/3 negative fixtures RED "
        "(canonical golden match)."
    )
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
