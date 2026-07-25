#!/usr/bin/env python3
# [OPUS-4.8] PreToolUse arm guard (bead sq-u59rq): NEVER auto-merge a PR whose BASE
# is another PR's branch (a "stacked" PR). Authored by Opus 4.8 (Fable unavailable;
# flag for re-review when Fable returns).
#
# WHY THIS GUARD EXISTS — the stacked-PR auto-merge trap (observed 2026-06-21, #1023):
# When you chain PR B onto PR A's branch to avoid a re-conflict (B's base = A's head
# branch, not `main`) and then arm BOTH for auto-merge, GitHub squash-merges B into
# its STACKED BASE (A's branch), not `main`, the moment A merges first. GitHub marks
# B "merged" and auto-deletes B's head branch (unreopenable) — but B's content never
# reaches `main`. The fix that day was to re-land identical content as a fresh PR
# (#1028). The standing rule (AGENTS.md, *Arming model* / *Stacked PRs*): do NOT arm
# auto-merge on a PR whose base is not `main`. Stack strictly sequentially, or retarget
# the upper PR's base back to `main` (`gh pr edit <n> --base main`) once the lower PR
# lands, THEN arm it.
#
# WHAT THIS DOES: it is a deterministic, network-light **PreToolUse hook** on `Bash`
# (sibling to the sparq-perf-reviewer agent-hook in .claude/settings.json) that fires
# on the arming step. It:
#   1. reads the PreToolUse hook JSON from stdin (tool_input.command),
#   2. ALLOWs untouched anything that is NOT a `gh pr merge … --auto` arming command,
#   3. for an arming command, parses the PR number, asks `gh pr view <n> --json
#      baseRefName,headRefName,number`, and
#   4. DENIES the arm (permissionDecision: "deny") when baseRefName != the trunk
#      (default `main`) — i.e. the PR is stacked on another branch — with a 🤖 SPARQ
#      reason telling the operator to retarget the base to `main` (or land sequentially)
#      before arming; otherwise ALLOWs.
# It is cheap (no model spend) and complements the perf-reviewer: both run on the same
# matcher and BOTH must allow for the arm to proceed (any deny blocks the tool call).
#
# FAIL-OPEN on its OWN errors, FAIL-CLOSED on a confirmed stacked base. If `gh` is
# missing, errors, or the command can't be parsed, this guard ALLOWs (it must never
# wedge the merge train on its own malfunction) — the perf-reviewer + ci-summary +
# branch protection still gate the actual merge. It DENIES only when it positively
# confirms baseRefName is something other than the trunk.
#
# Usage:
#   check-pr-arm-base.py                 # read hook JSON on stdin, emit decision JSON
#   check-pr-arm-base.py --trunk main    # override the trunk branch name (default main)
#   check-pr-arm-base.py --self-test     # hermetic logic self-test (no network)
#
# stdlib-only.

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import subprocess
import sys
import tempfile
from pathlib import Path

DEFAULT_TRUNK = "main"

SELF_ID = (
    "🤖 SPARQ stacked-PR arm guard"
)

# `gh pr merge` with `--auto` (the arming step). We also accept the future `--merge-when`
# style only via `--auto`, which is the canonical arming flag the orchestrator uses.
_GH_PR_MERGE_RE = re.compile(r"\bgh\s+pr\s+merge\b")
_AUTO_RE = re.compile(r"(?<![\w-])--auto(?![\w-])")


def _allow(reason: str) -> dict:
    return {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "permissionDecisionReason": reason,
        }
    }


def _deny(reason: str) -> dict:
    return {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }


def is_arm_command(command: str) -> bool:
    """True iff `command` is a `gh pr merge … --auto` arming invocation."""
    if not command:
        return False
    return bool(_GH_PR_MERGE_RE.search(command) and _AUTO_RE.search(command))


def parse_pr_number(command: str) -> str | None:
    """Extract the PR number/URL argument from a `gh pr merge` command.

    `gh pr merge` takes the PR as its first positional (`<number> | <url> | <branch>`).
    We tokenise with shlex and return the first token after `merge` that is NOT an
    option (doesn't start with `-`) and is NOT the value of a preceding value-taking
    flag. We accept a bare number, a PR URL (…/pull/<n>), or a branch name — `gh`
    resolves all three; we only need to hand the SAME token back to `gh pr view`.
    """
    try:
        toks = shlex.split(command)
    except ValueError:
        return None
    # Find the `merge` subcommand position (after `gh pr`).
    try:
        i = toks.index("merge")
    except ValueError:
        return None
    # Flags that consume the following token as their value (so that value is NOT the
    # positional PR arg). Conservative list covering gh pr merge's value-flags.
    value_flags = {
        "-b",
        "--body",
        "-F",
        "--body-file",
        "-s",
        "--subject",
        "-t",
        "--subject",
        "--match-head-commit",
        "--author-email",
        "-R",
        "--repo",
    }
    j = i + 1
    n = len(toks)
    while j < n:
        tok = toks[j]
        if tok.startswith("-"):
            # `--flag=value` consumes nothing extra; a bare value-flag eats the next.
            if "=" not in tok and tok in value_flags:
                j += 2
            else:
                j += 1
            continue
        return tok
    return None


def pr_base_ref(pr: str, trunk: str) -> tuple[str | None, str | None]:
    """Return (baseRefName, error). error is set (and base None) if gh can't answer —
    callers FAIL OPEN on error so the guard never wedges the train on its own fault."""
    try:
        proc = subprocess.run(
            ["gh", "pr", "view", pr, "--json", "baseRefName,headRefName,number"],
            capture_output=True,
            text=True,
            timeout=30,
        )
    except FileNotFoundError:
        return None, "gh CLI not found"
    except subprocess.TimeoutExpired:
        return None, "gh pr view timed out"
    except OSError as e:  # pragma: no cover - defensive
        return None, f"gh invocation failed: {e}"
    if proc.returncode != 0:
        return None, (proc.stderr or proc.stdout or "gh pr view failed").strip()
    try:
        data = json.loads(proc.stdout)
    except json.JSONDecodeError:
        return None, "gh pr view returned non-JSON"
    base = data.get("baseRefName")
    if not isinstance(base, str) or not base:
        return None, "gh pr view returned no baseRefName"
    return base, None


def decide(command: str, trunk: str, base_lookup) -> dict:
    """Pure decision logic. `base_lookup(pr) -> (base, error)` is injected so the
    self-test can exercise this without a network/gh dependency."""
    if not is_arm_command(command):
        return _allow("not a `gh pr merge … --auto` arming command; base guard N/A")
    pr = parse_pr_number(command)
    if pr is None:
        # Can't identify the PR — fail OPEN (the perf-reviewer + ci-summary still gate).
        return _allow(
            f"{SELF_ID}: could not parse a PR argument from the arm command; "
            "allowing (other gates still apply)"
        )
    base, error = base_lookup(pr)
    if error is not None:
        # Fail OPEN on our own lookup failure — never wedge the merge train.
        return _allow(
            f"{SELF_ID}: could not resolve PR #{pr} base ({error}); allowing "
            "(other gates still apply)"
        )
    if base == trunk:
        return _allow(
            f"{SELF_ID}: PR #{pr} base is `{trunk}` (not stacked) — arm OK"
        )
    return _deny(
        f"{SELF_ID}: PR #{pr} base is `{base}`, NOT `{trunk}` — this is a STACKED PR. "
        "Arming auto-merge on a PR stacked on another branch squash-merges it into that "
        "base (not `main`) the moment the lower PR lands, so its content never reaches "
        "`main` and its head branch auto-deletes (unreopenable; bead sq-u59rq, #1023). "
        f"Retarget the base to `{trunk}` first (`gh pr edit {pr} --base {trunk}`) once "
        "the lower PR has landed, THEN arm — or land the stack strictly sequentially."
    )


# --------------------------------------------------------------------------- tests
def self_test() -> int:
    trunk = "main"

    def base_main(_pr):
        return "main", None

    def base_stacked(_pr):
        return "site/sq-1022-explain", None

    def base_error(_pr):
        return None, "gh CLI not found"

    cases = [
        # (label, command, base_lookup, expected_decision)
        (
            "not an arm command (no --auto)",
            "gh pr merge 1044 --squash",
            base_main,
            "allow",
        ),
        (
            "not a gh pr merge at all",
            "gh pr create --fill",
            base_main,
            "allow",
        ),
        (
            "arm, base=main → allow",
            "gh pr merge 1044 --auto --squash",
            base_main,
            "allow",
        ),
        (
            "arm, base=stacked branch → DENY",
            "gh pr merge 1023 --auto --squash",
            base_stacked,
            "deny",
        ),
        (
            "arm by URL, base=stacked → DENY",
            "gh pr merge https://github.com/jeswr/sparq/pull/1023 --auto --squash",
            base_stacked,
            "deny",
        ),
        (
            "arm with --body value flag before PR num, base=stacked → DENY",
            'gh pr merge --body "ship it" 1023 --auto --squash',
            base_stacked,
            "deny",
        ),
        (
            "arm, gh lookup error → fail OPEN (allow)",
            "gh pr merge 1023 --auto --squash",
            base_error,
            "allow",
        ),
        (
            "arm, --auto as --auto=… style still detected (base=stacked) → DENY",
            "gh pr merge 1023 --squash --auto",
            base_stacked,
            "deny",
        ),
        (
            "substring 'autograph' must NOT count as --auto",
            "gh pr merge 1044 --autograph",
            base_stacked,
            "allow",
        ),
    ]
    failures = 0
    for label, cmd, lookup, expected in cases:
        out = decide(cmd, trunk, lookup)
        got = out["hookSpecificOutput"]["permissionDecision"]
        ok = got == expected
        print(f"  [{'PASS' if ok else 'FAIL'}] {label}: {got} (want {expected})")
        if not ok:
            failures += 1

    # parse_pr_number unit checks
    pn_cases = [
        ("gh pr merge 1023 --auto --squash", "1023"),
        ("gh pr merge --repo jeswr/sparq 1023 --auto", "1023"),
        ("gh pr merge https://github.com/jeswr/sparq/pull/9 --auto",
         "https://github.com/jeswr/sparq/pull/9"),
        ("gh pr merge --auto --squash", None),
    ]
    for cmd, want in pn_cases:
        got = parse_pr_number(cmd)
        ok = got == want
        print(f"  [{'PASS' if ok else 'FAIL'}] parse_pr_number({cmd!r}): "
              f"{got!r} (want {want!r})")
        if not ok:
            failures += 1

    # [GPT-5.6] #3605: exercise the real hook command with fixture input from both
    # the repository root and an unrelated cwd. Claude Code deliberately runs hooks
    # in the shell's current cwd, so the settings entry MUST anchor the guard through
    # CLAUDE_PROJECT_DIR. The launcher's own missing/unreadable/script-error path is
    # fail-open, matching this guard's existing failure disposition: only a positively
    # confirmed non-trunk base may deny an arm.
    repo_root = Path(__file__).resolve().parent.parent
    guard_script = Path(__file__).resolve()
    settings_path = repo_root / ".claude" / "settings.json"
    try:
        settings = json.loads(settings_path.read_text(encoding="utf-8"))
        hook_command = settings["hooks"]["PreToolUse"][0]["hooks"][0]["command"]
    except (OSError, KeyError, IndexError, TypeError, json.JSONDecodeError) as error:
        print(f"  [FAIL] load arm-guard hook command: {error}")
        failures += 1
        hook_command = ""

    with tempfile.TemporaryDirectory(prefix="sparq-arm-guard-test-") as temp_dir:
        fixture_root = Path(temp_dir)
        off_root_cwd = fixture_root / "off-root-cwd"
        fake_bin = fixture_root / "bin"
        off_root_cwd.mkdir()
        fake_bin.mkdir()
        fake_gh = fake_bin / "gh"
        fake_gh.write_text(
            "#!/bin/sh\n"
            "printf '%s\\n' "
            "'{\"baseRefName\":\"feature/stacked\","
            "\"headRefName\":\"fixture\",\"number\":1023}'\n",
            encoding="utf-8",
        )
        fake_gh.chmod(0o755)

        fixture_env = os.environ.copy()
        fixture_env["CLAUDE_PROJECT_DIR"] = str(repo_root)
        fixture_env["PATH"] = f"{fake_bin}{os.pathsep}{fixture_env.get('PATH', '')}"

        fixture_cases = [
            ("non-arm fixture", "printf harmless", "allow"),
            ("stacked arm fixture", "gh pr merge 1023 --auto", "deny"),
        ]
        invocations = [
            ("direct/root cwd", [sys.executable, str(guard_script)], repo_root),
            ("direct/off-root cwd", [sys.executable, str(guard_script)], off_root_cwd),
            ("settings/root cwd", ["bash", "-c", hook_command], repo_root),
            ("settings/off-root cwd", ["bash", "-c", hook_command], off_root_cwd),
        ]
        for case_label, fixture_command, expected in fixture_cases:
            payload = json.dumps({"tool_input": {"command": fixture_command}})
            reference = None
            for invocation_label, argv, cwd in invocations:
                proc = subprocess.run(
                    argv,
                    cwd=cwd,
                    env=fixture_env,
                    input=payload,
                    capture_output=True,
                    text=True,
                    timeout=10,
                )
                try:
                    output = json.loads(proc.stdout)
                    got = output["hookSpecificOutput"]["permissionDecision"]
                except (KeyError, TypeError, json.JSONDecodeError):
                    output = None
                    got = None
                if reference is None:
                    reference = output
                ok = proc.returncode == 0 and got == expected and output == reference
                print(
                    f"  [{'PASS' if ok else 'FAIL'}] {case_label}, "
                    f"{invocation_label}: {got} (want {expected}, root-identical)"
                )
                if not ok:
                    if proc.stderr:
                        print(f"    stderr: {proc.stderr.strip()}")
                    failures += 1

        # A missing project script must produce an explicit allow with exit zero. A
        # non-zero PreToolUse hook exit is fail-closed in Claude Code and caused the
        # live all-Bash deadlock this regression test protects against.
        unavailable_projects = [
            ("missing guard script", fixture_root / "missing-project"),
            ("unreadable guard script", fixture_root / "unreadable-project"),
        ]
        unavailable_projects[0][1].mkdir()
        unreadable_scripts = unavailable_projects[1][1] / "scripts"
        unreadable_scripts.mkdir(parents=True)
        unreadable_guard = unreadable_scripts / "check-pr-arm-base.py"
        unreadable_guard.write_text("raise SystemExit(1)\n", encoding="utf-8")
        unreadable_guard.chmod(0o000)
        unavailable_payload = json.dumps(
            {"tool_input": {"command": "gh pr merge 1023 --auto"}}
        )
        for unavailable_label, project_dir in unavailable_projects:
            unavailable_env = fixture_env.copy()
            unavailable_env["CLAUDE_PROJECT_DIR"] = str(project_dir)
            unavailable_proc = subprocess.run(
                ["bash", "-c", hook_command],
                cwd=off_root_cwd,
                env=unavailable_env,
                input=unavailable_payload,
                capture_output=True,
                text=True,
                timeout=10,
            )
            try:
                unavailable_output = json.loads(unavailable_proc.stdout)
                unavailable_decision = unavailable_output["hookSpecificOutput"][
                    "permissionDecision"
                ]
            except (KeyError, TypeError, json.JSONDecodeError):
                unavailable_decision = None
            unavailable_ok = (
                unavailable_proc.returncode == 0 and unavailable_decision == "allow"
            )
            print(
                f"  [{'PASS' if unavailable_ok else 'FAIL'}] {unavailable_label}, "
                f"off-root cwd: {unavailable_decision} (want allow + exit 0)"
            )
            if not unavailable_ok:
                if unavailable_proc.stderr:
                    print(f"    stderr: {unavailable_proc.stderr.strip()}")
                failures += 1

    if failures:
        print(f"\nself-test: {failures} case(s) FAILED")
        return 1
    print("\nself-test: all cases PASS")
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="PreToolUse guard: deny arming auto-merge on a STACKED PR "
        "(base != trunk). Bead sq-u59rq."
    )
    ap.add_argument(
        "--trunk",
        default=DEFAULT_TRUNK,
        help=f"trunk branch a PR must target to be armable (default: {DEFAULT_TRUNK}).",
    )
    ap.add_argument(
        "--self-test",
        action="store_true",
        help="run the hermetic logic self-test (no network) and exit.",
    )
    args = ap.parse_args(argv)

    if args.self_test:
        return self_test()

    raw = sys.stdin.read()
    command = ""
    if raw.strip():
        try:
            payload = json.loads(raw)
            command = (payload.get("tool_input") or {}).get("command", "") or ""
        except json.JSONDecodeError:
            command = ""

    out = decide(command, args.trunk, lambda pr: pr_base_ref(pr, args.trunk))
    print(json.dumps(out))
    # A PreToolUse command hook signals its decision via the JSON above (exit 0). We
    # always exit 0 and let `permissionDecision` carry allow/deny — exit-code semantics
    # would be ambiguous next to the structured decision.
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
