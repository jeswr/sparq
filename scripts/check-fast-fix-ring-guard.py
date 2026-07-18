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
# STRATEGY — PIN THE WHOLE TOKEN-CONSUMING JOB + ASSERT EXACTLY ONE CONSUMER
# =============================================================================
# Rounds 1–4 of adversarial review proved that both a predicate-PARSING validator AND
# a fragment-EXTRACTING golden-match lose to a moving target:
#
#   * A parser cannot win: for every "reject a top-level ||" rule there is a
#     `(A && B && guard || true)` paren-wrap; for every "the guard regex is present"
#     rule there is a `!(guard)` negation; for every "both needles present" G3 rule
#     there is a conjunction→disjunction `repo OR SHA` swap (both needles survive).
#
#   * A FRAGMENT golden-match (rounds 1–3) still leaked, because it pinned only the
#     EXTRACTED pieces (the `if:`, the `select(...)`, the guard skeletons) and validated
#     only the FIRST token-consuming job. Codex round 4 found two residual holes:
#       (a) anything the extractor did NOT cover could still be weakened — an inline
#           `: # select(...)` demotion, an uncalled-function move of the guards, an
#           alt dispatch spelling `gh workflow run -R ... dispatch.yml` — because those
#           edits live OUTSIDE the extracted fragments, so the fragments still matched.
#       (b) a SECOND privileged job carrying the same golden `if:` but NO SHA/repo
#           defenses passed, because only the first token consumer was validated.
#
# Codex round 4 gave the terminal design in its own words: "Pin the COMPLETE
# token-consuming job, not extracted fragments" and "Require EXACTLY ONE privileged
# job/path or apply the complete canonical validation to every token consumer."
#
# So this checker does exactly that:
#
#   J1  PIN THE WHOLE JOB. Parse the workflow YAML, find every job whose steps
#       reference `secrets.REGISTRY_RING_TOKEN`, canonicalize the ONE token-consuming
#       job's ENTIRE definition (the whole job mapping: `if:`, `permissions:`, every
#       step's `env`, `name`, and the EXACT `run:` body — including the
#       `gh workflow run dispatch.yml` line, the fallback `select(...)`, and BOTH live
#       `!=` guard blocks) into a normalized, deterministic form (JSON, keys sorted)
#       and require BYTE-EQUALITY to the stored GOLDEN_JOB. Because the COMPLETE job is
#       pinned there is NOTHING left to weaken: no fragment to extract-and-dodge, no
#       predicate to demote to a comment (the comment is INSIDE the pinned run body, so
#       moving a predicate into it changes the run body), no guards to move into an
#       uncalled function (that changes the run body too), no alt dispatch spelling
#       (that changes the run body too), no operand swap, no negation, no OR-wrap. ANY
#       edit to that job — security-relevant or not — changes the normalized job and REDs.
#
#   J2  EXACTLY ONE TOKEN CONSUMER. Count the jobs that reference
#       `secrets.REGISTRY_RING_TOKEN` anywhere in their steps; require the count == 1
#       and that the single consumer is the golden job. A SECOND privileged job (codex
#       round-4 finding #b) makes the count 2 and REDs — there is no un-validated path
#       to the token.
#
# WHY THIS IS TERMINAL (provably immune to the fragment/demotion/duplication class):
#   - The whole-job pin means the checker's "attack surface" is the ENTIRE job text,
#     not a chosen subset. The round-1..4 bypasses ALL work by mutating job text that a
#     narrower rule did not look at; here there is no text the rule does not look at.
#   - Exactly-one-consumer means there is no second, un-pinned path to the token.
#   Together: the set of edits that (a) reach the token and (b) pass the checker is
#   EXACTLY {edits that leave the whole job byte-identical} = {no change to the job}.
#   A weakening is, by definition, a change to the job, so it cannot pass. The only way
#   to change the job AND pass is to update GOLDEN_JOB in the SAME PR — which is the
#   intended human review checkpoint (a reviewer sees the golden move and re-confirms
#   the guard is still sound), the CORRECT fail-safe direction.
#
# The GOLDEN_JOB below is the pretty-printed (indent=2, keys sorted) canonical JSON of
# the known-good `ring` job. It is human-readable ON PURPOSE — it IS the job content, so
# a reviewer can read exactly what is pinned. Regenerate after an intentional refactor:
#     python3 -c "import yaml,json; \
#       d=yaml.safe_load(open('.github/workflows/fast-fix-ring.yml')); \
#       print(json.dumps(d['jobs']['ring'],sort_keys=True,ensure_ascii=False,indent=2))"
# and paste the output between the GOLDEN_JOB delimiters, IN THE SAME PR as the workflow
# change, so the golden bump is the review checkpoint.
#
# NEGATIVE COVERAGE (--self-test, hermetic — no gh/git/network): the live workflow
# passes, and EACH round-1..4 bypass — including codex round-4's inline-comment
# demotion, uncalled-function move, alt dispatch spelling, AND a second privileged
# token-consuming job — is applied as a SURGICAL mutation to a PHYSICAL COPY of the real
# workflow and run through the LIVE `--root` path, and goes RED; the unmutated copy
# passes. See _self_test for the mutation→red table.
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
import difflib
import json
import sys
import tempfile
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent
WORKFLOW_REL = Path(".github") / "workflows" / "fast-fix-ring.yml"
WORKFLOW = REPO_ROOT / WORKFLOW_REL

# The privileged secret whose every consumer must be the one pinned golden job.
RING_TOKEN_SECRET = "secrets.REGISTRY_RING_TOKEN"


class Offence(Exception):
    pass


# =============================================================================
# GOLDEN_JOB — the complete, known-good `ring` job, canonical JSON (keys sorted,
# indent=2). This is the ENTIRE token-consuming job: `if:`, `permissions:`, and the
# one step's `env` + `name` + full `run:` body (the select(...), both live `!=`
# guards, and the `gh workflow run dispatch.yml` doorbell are all inside it). The
# checker re-derives this exact canonical form from the workflow and requires
# BYTE-EQUALITY. To refactor the guard legitimately, regenerate this block IN THE SAME
# PR (see the header one-liner) — that golden bump is the review checkpoint.
# =============================================================================
GOLDEN_JOB = r"""{
  "if": "github.event.workflow_run.conclusion == 'failure' && github.event.workflow_run.event == 'pull_request' && github.event.workflow_run.head_repository.full_name == github.repository",
  "name": "fast-fix ring (advisory)",
  "permissions": {
    "pull-requests": "read"
  },
  "runs-on": "ubuntu-latest",
  "steps": [
    {
      "env": {
        "GH_TOKEN": "${{ github.token }}",
        "HEAD_SHA": "${{ github.event.workflow_run.head_sha }}",
        "REGISTRY_RING_TOKEN": "${{ secrets.REGISTRY_RING_TOKEN }}",
        "REPO": "${{ github.repository }}",
        "RUN_PR_NUMBER": "${{ (github.event.workflow_run.pull_requests[0] != null) && github.event.workflow_run.pull_requests[0].number || '' }}"
      },
      "name": "Resolve the PR for the failed run and ring the registry dispatcher",
      "run": "set -euo pipefail\n\n# --- Derive the PR number, SAME-REPO only (spoof guard #2 / deputy #3) ---\n# The top-level job `if:` already fail-closed on\n# head_repository.full_name == github.repository, so a fork/cross-repo run\n# never reaches this step. The head_sha->PR fallback below adds a SECOND,\n# independent barrier (defense in depth against a same-SHA collision):\n# every resolved PR must be BOTH same-repo AND pinned to the failing run's\n# exact head SHA before any dispatch.\nPR_NUMBER=\"${RUN_PR_NUMBER}\"\nif [ -z \"${PR_NUMBER}\" ]; then\n  # pull_requests is empty for fork PRs and can be empty for same-repo PRs\n  # whose association GitHub hasn't attached to the run; look the PR up by\n  # the failing run's head SHA, but ONLY accept a pull whose head repo is\n  # THIS repo AND whose head SHA is EXACTLY this run's head SHA (a fork PR,\n  # or an internal PR that merely shares this commit but points its head at\n  # a different SHA, is filtered out and the job skips).\n  if ! pulls=\"$(gh api \"repos/${REPO}/commits/${HEAD_SHA}/pulls\" \\\n      --jq \"[.[] | select(.head.repo.full_name == \\\"${REPO}\\\" and .head.sha == \\\"${HEAD_SHA}\\\" and .state == \\\"open\\\")] | .[0].number // empty\")\"; then\n    echo \"::notice::could not look up a PR for the failed run's head SHA — skipping (the registry cron is the backstop).\"\n    exit 0\n  fi\n  PR_NUMBER=\"${pulls}\"\nfi\nif [ -z \"${PR_NUMBER}\" ]; then\n  echo \"no same-repo PR is associated with the failed run's head SHA (fork PR, or the run was not a same-repo PR) — nothing to ring; skipping.\"\n  exit 0\nfi\n\n# --- Live PR state — labels/arming churn between the trigger and here ---\n# (ported from the former ci-summary fix-ring job; head_repo/head_sha now\n# re-read for the #3475 defense-in-depth check below.)\nif ! state=\"$(gh api \"repos/${REPO}/pulls/${PR_NUMBER}\" \\\n    --jq '{draft: .draft, armed: (.auto_merge != null), head_repo: .head.repo.full_name, head_sha: .head.sha, labels: [.labels[].name]}')\"; then\n  echo \"::notice::could not read live PR state — skipping the ring (the registry cron is the backstop).\"\n  exit 0\nfi\n# Defense in depth (#3475): the resolved PR must belong to THIS repo AND\n# its head must still be the failing run's exact SHA. This closes any\n# residual same-SHA-collision window in either resolution path (the\n# workflow_run.pull_requests[0] association OR the pulls-at-SHA lookup)\n# before the privileged doorbell fires.\nif [ \"$(jq -r '.head_repo' <<<\"$state\")\" != \"${REPO}\" ]; then\n  echo \"resolved PR #${PR_NUMBER} head repo is not ${REPO} (fork / cross-repo collision) — skipping.\"\n  exit 0\nfi\nif [ \"$(jq -r '.head_sha' <<<\"$state\")\" != \"${HEAD_SHA}\" ]; then\n  echo \"resolved PR #${PR_NUMBER} head SHA no longer matches the failing run's head SHA (stale head / same-SHA collision) — skipping.\"\n  exit 0\nfi\nhas() { jq -e --arg l \"$1\" '.labels | index($l) != null' <<<\"$state\" >/dev/null; }\nif [ \"$(jq -r '.draft' <<<\"$state\")\" = \"true\" ]; then\n  echo \"PR re-drafted since the trigger — not pipeline-owned for fast-fix; skipping.\"\n  exit 0\nfi\nif has \"review:needs-user\" || has \"needs:user\"; then\n  echo \"human-owned hold (review:needs-user / needs:user) — the repair loop must not pick this PR up; skipping.\"\n  exit 0\nfi\nif [ \"$(jq -r '.armed' <<<\"$state\")\" != \"true\" ] && ! has \"review:pass\" && ! has \"review:changes\"; then\n  echo \"PR is neither armed nor in the review pipeline (review:pass / review:changes) — skipping.\"\n  exit 0\nfi\nif [ -z \"${REGISTRY_RING_TOKEN}\" ]; then\n  # NO needs:ci-fix label fallback, deliberately: the registry PLAN\n  # derives ci-fix admission from the PR's gate STATE and treats\n  # needs:* PR labels as human-owned holds that EXCLUDE the PR from\n  # the repair enumeration — a label here would suppress the fix.\n  echo \"::notice::REGISTRY_RING_TOKEN unavailable (secret unconfigured) — cannot ring; the registry cron (every ~10 min) is the backstop.\"\n  exit 0\nfi\nGH_TOKEN=\"${REGISTRY_RING_TOKEN}\" gh workflow run dispatch.yml -R jeswr/agent-account-registry\necho \"rang jeswr/agent-account-registry dispatch for PR #${PR_NUMBER} — the repair sweep runs now instead of on the next cron tick.\"\n"
    }
  ],
  "timeout-minutes": 5
}"""


def canonical_job(job: dict) -> str:
    """Deterministic, whole-job canonical form: JSON with keys sorted and indent=2.
    Pins EVERY field of the job (if/permissions/env/name and the exact run body).
    Nothing is extracted or dropped, so there is no fragment to weaken and no comment,
    uncalled function, alt spelling, or second predicate that lives outside the pinned
    text."""
    return json.dumps(job, sort_keys=True, ensure_ascii=False, indent=2)


def _token_consumers(jobs: dict) -> list[str]:
    """Every job id whose serialized definition references secrets.REGISTRY_RING_TOKEN.
    Located by CONTENT (not job name) so a rename cannot hide a token consumer, and via
    the parsed job (not a raw grep of the file) so a token reference demoted into a
    `#`-comment in a run body still counts — that comment is part of the parsed run
    string."""
    out = []
    for jid, job in jobs.items():
        blob = yaml.safe_dump(job, sort_keys=True, allow_unicode=True)
        if RING_TOKEN_SECRET in blob:
            out.append(jid)
    return out


def check_doc(text: str) -> list[str]:
    """Return a list of offence strings; empty == clean. Whole-job pin (J1) + exactly
    one token consumer (J2)."""
    offences: list[str] = []
    doc = yaml.safe_load(text)
    if not isinstance(doc, dict):
        raise Offence("workflow is not a YAML mapping")
    jobs = doc.get("jobs")
    if not isinstance(jobs, dict):
        raise Offence("workflow has no `jobs:` mapping")

    # ---- J2: EXACTLY ONE job may consume the ring token ----
    consumers = _token_consumers(jobs)
    if len(consumers) == 0:
        offences.append(
            f"J2: no job references {RING_TOKEN_SECRET} — the token-consuming job is "
            f"missing or the secret reference was renamed; cannot pin the guarded job"
        )
        # Nothing to pin; the missing consumer is the whole finding.
        return offences
    if len(consumers) > 1:
        offences.append(
            f"J2: {len(consumers)} jobs reference {RING_TOKEN_SECRET} "
            f"({', '.join(consumers)}) — there must be EXACTLY ONE token-consuming job "
            f"so the whole-job pin covers every path to the token. A second privileged "
            f"job is a second, un-pinned path to the secret and REDs here."
        )
        # Fall through to J1 so the reviewer also sees any whole-job drift; the count
        # offence alone already fails the check.

    # ---- J1: the ONE token-consuming job, whole, must equal GOLDEN_JOB byte-exact ----
    matched_golden = [jid for jid in consumers if canonical_job(jobs[jid]) == GOLDEN_JOB]
    if not matched_golden:
        # Diff against the first consumer (the most likely intended job) for review.
        target = consumers[0]
        actual = canonical_job(jobs[target])
        diff = "\n".join(
            difflib.unified_diff(
                GOLDEN_JOB.splitlines(),
                actual.splitlines(),
                fromfile="GOLDEN_JOB",
                tofile=f"jobs.{target} (actual)",
                lineterm="",
            )
        )
        offences.append(
            f"J1: the token-consuming job `{target}` does not match the canonical "
            f"GOLDEN_JOB byte-for-byte. The WHOLE job is pinned, so ANY edit — an "
            f"OR-wrap / negation / operand swap in `if:`, a demoted-to-comment "
            f"predicate, an uncalled-function move of the guards, an alt dispatch "
            f"spelling, a dropped `select(...)` conjunct, an inverted `!=` guard, a "
            f"changed permission — reds here. If this is a LEGITIMATE refactor, "
            f"regenerate GOLDEN_JOB in this PR (see the header one-liner); that golden "
            f"bump is the review checkpoint.\n"
            f"       --- unified diff (GOLDEN_JOB vs actual) ---\n{diff}"
        )

    return offences


def check_root(root: Path) -> list[str]:
    """Live path used by both the default run and every --self-test fixture: read the
    workflow from `root` and check it. The self-test mutates a PHYSICAL COPY of the
    real workflow on disk and calls THIS, so the fixtures exercise the exact code path
    that runs in CI."""
    wf = root / WORKFLOW_REL
    if not wf.exists():
        raise Offence(f"{wf} not found")
    return check_doc(wf.read_text(encoding="utf-8"))


# --------------------------------------------------------------------------- #
#  --self-test: hermetic negative fixtures                                    #
# --------------------------------------------------------------------------- #

def _self_test() -> int:
    text = WORKFLOW.read_text(encoding="utf-8")
    failures = 0

    def _run_on(mutated: str) -> list[str]:
        """Write the (mutated) workflow into a throwaway repo-root tree and run the
        LIVE --root path over it, so every fixture goes through check_root/check_doc
        exactly as CI does."""
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            wf = root / WORKFLOW_REL
            wf.parent.mkdir(parents=True, exist_ok=True)
            wf.write_text(mutated, encoding="utf-8")
            return check_root(root)

    def expect_clean(label: str, t: str) -> None:
        nonlocal failures
        offs = _run_on(t)
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
        offs = _run_on(t)
        if not any(needle in o for o in offs):
            failures += 1
            print(
                f"  [FAIL] {label}: expected an offence containing {needle!r} but got: {offs}"
            )
        else:
            print(f"  [ok]   {label}: correctly RED ({needle})")

    # 0) the real, live workflow (physical copy) must be clean.
    expect_clean("live workflow (physical copy via --root)", text)

    # --- round-1 fixtures: a guard is REMOVED (whole-job pin => J1) ---

    # 1) FORK RUN: remove the head_repository guard from the job `if:`.
    no_guard = text.replace(
        "\n      && github.event.workflow_run.head_repository.full_name == github.repository",
        "",
        1,
    )
    expect_red("R1 fork run (head_repository guard removed)", no_guard, "J1")

    # 2) SAME-SHA collision: drop the `.head.sha ==` filter from the fallback select.
    no_sha_filter = text.replace(
        'and .head.sha == \\"${HEAD_SHA}\\" and .state', "and .state", 1
    )
    expect_red("R1 same-SHA collision (head.sha filter removed)", no_sha_filter, "J1")

    # 3) STALE-HEAD: remove the live head_sha re-check block before dispatch.
    no_live_recheck = text.replace(
        '          if [ "$(jq -r \'.head_sha\' <<<"$state")" != "${HEAD_SHA}" ]; then\n'
        '            echo "resolved PR #${PR_NUMBER} head SHA no longer matches the failing run\'s head SHA (stale head / same-SHA collision) — skipping."\n'
        "            exit 0\n"
        "          fi\n",
        "",
        1,
    )
    expect_red("R1 stale-head (live head_sha re-check removed)", no_live_recheck, "J1")

    # --- round-2 fixtures: predicate-parser was too loose (whole-job pin => J1) ---

    # 4) TOP-LEVEL `|| true` appended to the if (bare, depth-0).
    or_bypass = text.replace(
        "&& github.event.workflow_run.head_repository.full_name == github.repository\n",
        "&& github.event.workflow_run.head_repository.full_name == github.repository\n"
        "      || true\n",
        1,
    )
    expect_red("R2 top-level `|| true` bypass appended", or_bypass, "J1")

    # 5) fork-repo predicate removed from the fallback `select(...)`.
    no_repo_predicate = text.replace(
        'select(.head.repo.full_name == \\"${REPO}\\" and .head.sha == \\"${HEAD_SHA}\\" '
        'and .state == \\"open\\")',
        'select(.head.sha == \\"${HEAD_SHA}\\" and .state == \\"open\\")',
        1,
    )
    expect_red("R2 fallback fork-repo predicate removed", no_repo_predicate, "J1")

    # 6) .head.sha compared to a CONSTANT instead of ${HEAD_SHA}.
    sha_constant = text.replace(
        '.head.sha == \\"${HEAD_SHA}\\" and .state',
        '.head.sha == \\"deadbeefdeadbeefdeadbeefdeadbeefdeadbeef\\" and .state',
        1,
    )
    expect_red("R2 fallback head.sha compared to a constant", sha_constant, "J1")

    # 7) live `!=` guards inverted to `==` (fail-open).
    inverted_guards = text.replace(
        'if [ "$(jq -r \'.head_repo\' <<<"$state")" != "${REPO}" ]; then',
        'if [ "$(jq -r \'.head_repo\' <<<"$state")" == "${REPO}" ]; then',
        1,
    ).replace(
        'if [ "$(jq -r \'.head_sha\' <<<"$state")" != "${HEAD_SHA}" ]; then',
        'if [ "$(jq -r \'.head_sha\' <<<"$state")" == "${HEAD_SHA}" ]; then',
        1,
    )
    expect_red("R2 live `!=` guards inverted to `==` (fail-open)", inverted_guards, "J1")

    # --- round-3 fixtures: bypasses codex demonstrated against the PARSER ---

    # 8) [R3] PARENTHESIZED OR-WRAP of the whole condition (actionlint-valid).
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
    expect_red("R3 parenthesized `(... || true)` OR-wrap of the whole if", paren_or_wrap, "J1")

    # 9) [R3] NEGATED equality `!(head_repo == repo)`.
    negated_guard = text.replace(
        "&& github.event.workflow_run.head_repository.full_name == github.repository",
        "&& !(github.event.workflow_run.head_repository.full_name == github.repository)",
        1,
    )
    expect_red("R3 negated `!(head_repo == repo)` guard", negated_guard, "J1")

    # 10) [R3] G3 CONJUNCTION->DISJUNCTION: `repo AND sha` -> `repo OR sha` in select.
    or_conjunction = text.replace(
        '.head.repo.full_name == \\"${REPO}\\" and .head.sha == \\"${HEAD_SHA}\\"',
        '.head.repo.full_name == \\"${REPO}\\" or .head.sha == \\"${HEAD_SHA}\\"',
        1,
    )
    expect_red("R3 fallback conjunction->disjunction (repo OR sha)", or_conjunction, "J1")

    # 11) [R3] PREDICATE MOVED TO A COMMENT: the executable select drops the fork-repo
    #     predicate but a `#`-comment retains the needle text. The whole-job pin
    #     includes the run body verbatim (comment + weakened select), so it REDs.
    predicate_in_comment = text.replace(
        '            if ! pulls="$(gh api "repos/${REPO}/commits/${HEAD_SHA}/pulls" \\\n'
        '                --jq "[.[] | select(.head.repo.full_name == \\"${REPO}\\" and .head.sha == \\"${HEAD_SHA}\\" and .state == \\"open\\")] | .[0].number // empty")"; then\n',
        '            # select(.head.repo.full_name == \\"${REPO}\\" and .head.sha == \\"${HEAD_SHA}\\" and .state == \\"open\\")\n'
        '            if ! pulls="$(gh api "repos/${REPO}/commits/${HEAD_SHA}/pulls" \\\n'
        '                --jq "[.[] | select(.head.sha == \\"${HEAD_SHA}\\" and .state == \\"open\\")] | .[0].number // empty")"; then\n',
        1,
    )
    expect_red("R3 fork-repo predicate demoted to a comment (select drops it)", predicate_in_comment, "J1")

    # 12) [R3] INVERTED LIVE GUARD accompanied by a comment-only expected needle.
    inverted_with_comment = text.replace(
        '          if [ "$(jq -r \'.head_repo\' <<<"$state")" != "${REPO}" ]; then\n',
        '          # if [ "$(jq -r \'.head_repo\' <<<"$state")" != "${REPO}" ]; then\n'
        '          if [ "$(jq -r \'.head_repo\' <<<"$state")" == "${REPO}" ]; then\n',
        1,
    )
    expect_red("R3 live guard inverted with comment-only `!=` needle", inverted_with_comment, "J1")

    # --- round-4 fixtures: codex's residual fragment/duplication holes ---

    # 13) [R4] INLINE-COMMENT DEMOTION of the select via a `: # select(...)` no-op:
    #     append `: # <golden select>` after weakening the live select. A fragment
    #     extractor that stripped only WHOLE-LINE comments (or matched the needle text
    #     anywhere) could be fooled; the whole-job pin includes the entire run body so
    #     any such edit REDs.
    inline_comment_demotion = text.replace(
        '                --jq "[.[] | select(.head.repo.full_name == \\"${REPO}\\" and .head.sha == \\"${HEAD_SHA}\\" and .state == \\"open\\")] | .[0].number // empty")"; then\n',
        '                --jq "[.[] | select(.head.sha == \\"${HEAD_SHA}\\" and .state == \\"open\\")] | .[0].number // empty")"; then : # select(.head.repo.full_name == \\"${REPO}\\" and .head.sha == \\"${HEAD_SHA}\\" and .state == \\"open\\")\n',
        1,
    )
    expect_red("R4 inline `: # select(...)` demotion (weakened live filter)", inline_comment_demotion, "J1")

    # 14) [R4] GUARDS MOVED TO AN UNCALLED FUNCTION: wrap both live `!=` guards in a
    #     `_dead_guards() { ... }` that is never called, so the needle text survives
    #     but no guard executes before dispatch. Whole-job pin REDs (run body changed).
    moved_to_uncalled_fn = text.replace(
        '          if [ "$(jq -r \'.head_repo\' <<<"$state")" != "${REPO}" ]; then\n'
        '            echo "resolved PR #${PR_NUMBER} head repo is not ${REPO} (fork / cross-repo collision) — skipping."\n'
        "            exit 0\n"
        "          fi\n"
        '          if [ "$(jq -r \'.head_sha\' <<<"$state")" != "${HEAD_SHA}" ]; then\n'
        '            echo "resolved PR #${PR_NUMBER} head SHA no longer matches the failing run\'s head SHA (stale head / same-SHA collision) — skipping."\n'
        "            exit 0\n"
        "          fi\n",
        "          _dead_guards() {\n"
        '          if [ "$(jq -r \'.head_repo\' <<<"$state")" != "${REPO}" ]; then\n'
        "            exit 0\n"
        "          fi\n"
        '          if [ "$(jq -r \'.head_sha\' <<<"$state")" != "${HEAD_SHA}" ]; then\n'
        "            exit 0\n"
        "          fi\n"
        "          }\n",
        1,
    )
    expect_red("R4 live guards moved into an uncalled function", moved_to_uncalled_fn, "J1")

    # 15) [R4] ALT DISPATCH SPELLING: `gh workflow run -R <repo> dispatch.yml` (flag
    #     before the workflow file) instead of the golden `gh workflow run dispatch.yml
    #     -R <repo>`. A checker that pinned only the exact golden dispatch string could
    #     miss this reordering; the whole-job pin REDs on any run-body change.
    alt_dispatch_spelling = text.replace(
        "GH_TOKEN=\"${REGISTRY_RING_TOKEN}\" gh workflow run dispatch.yml -R jeswr/agent-account-registry",
        "GH_TOKEN=\"${REGISTRY_RING_TOKEN}\" gh workflow run -R jeswr/agent-account-registry dispatch.yml",
        1,
    )
    expect_red("R4 alt dispatch spelling (gh workflow run -R ... dispatch.yml)", alt_dispatch_spelling, "J1")

    # 16) [R4] SECOND PRIVILEGED TOKEN-CONSUMING JOB: add a second job that consumes
    #     secrets.REGISTRY_RING_TOKEN with the SAME golden `if:` but NO SHA/repo
    #     defenses. The exactly-one-consumer count REDs (J2). We insert it right after
    #     the `jobs:` line so the parse yields two consumers.
    second_job = text.replace(
        "jobs:\n",
        "jobs:\n"
        "  ring_evil:\n"
        "    name: second ring (bypass)\n"
        "    if: >-\n"
        "      github.event.workflow_run.conclusion == 'failure'\n"
        "      && github.event.workflow_run.event == 'pull_request'\n"
        "      && github.event.workflow_run.head_repository.full_name == github.repository\n"
        "    runs-on: ubuntu-latest\n"
        "    steps:\n"
        "      - name: second unguarded ring\n"
        "        env:\n"
        "          REGISTRY_RING_TOKEN: ${{ secrets.REGISTRY_RING_TOKEN }}\n"
        "        run: |\n"
        "          GH_TOKEN=\"${REGISTRY_RING_TOKEN}\" gh workflow run dispatch.yml -R jeswr/agent-account-registry\n",
        1,
    )
    expect_red("R4 second privileged token-consuming job", second_job, "J2")

    if failures:
        print(f"\nSELF-TEST FAILED: {failures} case(s) did not behave as expected.")
        return 1
    print(
        "\nSELF-TEST PASSED: live workflow clean + all round-1..4 negative fixtures RED "
        "(whole-job pin J1 + exactly-one-consumer J2, each verified through the live "
        "--root path against a physically-mutated workflow copy)."
    )
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description="fast-fix-ring cross-repo guard checker")
    ap.add_argument("--root", type=Path, default=None, help="repo root override")
    ap.add_argument(
        "--self-test", action="store_true", help="run hermetic negative fixtures"
    )
    args = ap.parse_args()

    if args.self_test:
        return _self_test()

    root = args.root if args.root is not None else REPO_ROOT
    try:
        offences = check_root(root)
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
