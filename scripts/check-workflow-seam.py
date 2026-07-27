#!/usr/bin/env python3
"""Assert a gate is actually WIRED — the YAML seam is where vacuity lives.

Why this is a separate script
-----------------------------
A gate's Python can be perfectly correct while the gate checks nothing, because
the defect lives in the workflow that calls it: an ``if:`` that never matches,
``continue-on-error: true``, a deleted step, or a ``|| true`` that throws away
an already-earned failure. Discarding an earned failure has caused three
separate incidents in this repository in a single day, so the guard against it
has to be at least as careful as the thing it guards.

It was NOT. This file exists because the previous inline version of this check
(a heredoc inside ``scripts/tests/test_model_attribution.sh``) scanned the
``run:`` block **line by line** and asked "does this line mention the gate AND
contain ``|| true``?". Shell does not work that way. Written across a backslash
continuation —

.. code-block:: yaml

    run: |
      python3 scripts/check-model-attribution.py --check-briefs \\
        || true

— the gate's exit status is discarded, and no single line satisfies both halves
of the predicate. The mutant passed the whole seam test 20/20 while the check
printed *"gate steps are unconditional and do not swallow failure"*. Living in
a heredoc is also why it had no test of its own: you cannot point a mutant at a
string embedded in the test that would have to catch it. Both problems are
fixed by making it a script with an argument.

Two populations, two mechanisms — and why there is no exclusion list
-------------------------------------------------------------------
This check has now found the same defect class three times: *the guard is
unguarded*. Every instance had one root — a step's ROLE was decided by
substring-matching a FILENAME in its ``run:`` body:

1. the step that runs THIS checker names the gate in its own ``--needle``
   argument, so it was counted as a gate invocation and the
   ``--min-invocations`` floor stopped detecting a deleted call site;
2. a ``grep`` for the suite's filename was satisfied by the adjacent
   ``shellcheck <suite>`` line, so deleting the one line that RAN the suite
   was invisible;
3. the ``--exclude <filename>`` self-exemption introduced to patch (1) was
   applied before the neutering checks, so it also exempted that step from
   *all* of them — a single additive ``continue-on-error: true`` on the
   gating seam step disarmed the whole gate while it printed ``OK``.

(3) is a direct consequence of the patch for (1), which is the tell: a guard
that must exempt itself **by filename** is structurally prone to exempting
itself from everything. So the exclusion list is retired rather than reordered,
and the two questions it conflated are answered separately:

* **Is this step a gate INVOCATION?** — resolved at command position by
  :func:`executes_target`, the same mechanism that fixed (2). A step that
  merely NAMES the gate (``shellcheck <gate>``, ``cat <gate>``, or this
  checker's own ``--needle <gate>``) is not running it, so it is not counted.
  No exemption is needed to say so, and ``--exclude`` is gone from the CLI —
  passing it is now an error, not a silent no-op.
* **Must this step be un-neutered?** — asked of every step whose body MENTIONS
  a needle. That is a strict superset of the invocations and deliberately
  includes the meta-step that runs this checker, because neutering *that* step
  is how the guard gets disarmed.

A step cannot red its OWN neutering — ``continue-on-error`` discards precisely
the exit status that would report it. So the seam step is not self-sufficient
and never claims to be: it catches the suite being unwired (``--require-exec``),
and the suite catches the seam step being neutered (it runs this checker over
the real workflow from a different step). Both edges are real and each is
pinned by a named red test. The residual is disarming BOTH steps in one diff.

What it checks, for every ``run:`` step whose body mentions a ``--needle``
--------------------------------------------------------------------------
* the step is reached at all — no ``if:`` on the step and none on its job;
* ``continue-on-error`` is not set;
* the needle's **logical** line (backslash continuations joined first) contains
  no ``||`` fallback, which under ``set -e`` is the one construct that converts
  a failure into a success;
* the ``run:`` body does not relax the shell's error handling (``set +e``,
  ``set +o pipefail``) — GitHub runs bash with ``-eo pipefail``, and turning
  that off silently un-gates every command that is not last;
* the step does not select a non-bash ``shell:``, which would drop ``-e``
  entirely;
* at least ``--min-invocations`` matching steps exist, so DELETING a call site
  reds this check rather than going unnoticed.

Advisory-by-design invocations are expressed as an explicit mode of the gate
script itself (``--advisory``), never as ``|| true`` in YAML — that is the whole
point of the distinction, and this checker is what keeps it real.

Stdlib + PyYAML (already required by the other workflow-parsing lints).
``--self-test`` runs hermetic both-direction teeth, including the continuation
form above.
"""

from __future__ import annotations

import argparse
import re
import shlex
import sys
import tempfile
from pathlib import Path

import yaml

# GitHub runs `run:` steps with `bash --noprofile --norc -eo pipefail` unless a
# `shell:` overrides it. These keep that contract; anything else drops `-e`.
SAFE_SHELLS = {"bash", "sh", "bash -eo pipefail {0}"}

# `set +e` / `set +o pipefail` / `set +ex` — anything that RELAXES error mode.
RELAX_ERRMODE_RE = re.compile(r"(?m)^\s*set\s+\+")


def logical_lines(script: str) -> list[str]:
    """Join backslash continuations, so one shell command is one string.

    This is the entire fix for the Y11 mutant. A line-by-line scan sees
    ``python3 gate.py \\`` and ``  || true`` as unrelated; the shell sees one
    command whose failure is discarded.
    """
    out: list[str] = []
    buf = ""
    for raw in script.splitlines():
        stripped = raw.rstrip()
        if stripped.endswith("\\"):
            buf += stripped[:-1].rstrip() + " "
            continue
        out.append((buf + stripped).strip())
        buf = ""
    if buf:
        out.append(buf.strip())
    return out


def simple_commands(script: str) -> list[list[str]]:
    """Every simple command in a ``run:`` body, as token lists.

    Splitting on the shell's command separators is what turns "the file is
    MENTIONED somewhere in this step" into "the file is the thing being RUN".
    Those are very different claims and only the second one means the test
    executes — see :func:`executes_target`.
    """
    cmds: list[list[str]] = []
    for line in logical_lines(script):
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        for part in re.split(r"\|\||&&|[;|&]", stripped):
            part = part.strip()
            if not part:
                continue
            try:
                toks = shlex.split(part, comments=True)
            except ValueError:
                # Unbalanced quotes somewhere else in the workflow: degrade to a
                # whitespace split rather than skipping the command entirely,
                # because skipping would fail OPEN.
                toks = part.split()
            if toks:
                cmds.append(toks)
    return cmds


# Interpreters that RUN their first non-flag argument. `shellcheck` and friends
# take the same argument and do not run it — which is the whole distinction the
# previous filename grep could not make.
INTERPRETERS = {"bash", "sh", "zsh", "dash", "ksh", "python3", "python"}


def executes_target(script: str, target: str) -> bool:
    """Does this ``run:`` body EXECUTE ``target`` at command position?"""
    def same(arg: str) -> bool:
        arg = arg.lstrip("./")
        return arg == target or arg.endswith("/" + target) or target.endswith("/" + arg)

    for toks in simple_commands(script):
        i = 0
        # Skip `VAR=value` prefixes and `env`, which precede the real command.
        while i < len(toks) and (re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*=.*", toks[i]) or toks[i] == "env"):
            i += 1
        if i >= len(toks):
            continue
        head, rest = toks[i], toks[i + 1:]
        if Path(head).name in INTERPRETERS:
            for a in rest:
                if a.startswith("-"):
                    continue
                if same(a):
                    return True
                break  # only the FIRST non-flag argument is the script
        elif same(head):
            return True
    return False


def seam_findings(
    workflow: Path,
    needles: tuple[str, ...],
    minimum: int,
    require_exec: tuple[str, ...] = (),
) -> tuple[list[str], int]:
    """``(findings, invocations_found)`` for one workflow file.

    See the module docstring for why there is no ``excludes`` parameter. In
    short: an invocation is a step that EXECUTES a needle at command position,
    while the neutering checks apply to every step that MENTIONS one. Deciding
    both by the same filename substring is what made three separate versions of
    this guard exempt themselves from the thing they were guarding.
    """
    findings: list[str] = []
    try:
        wf = yaml.safe_load(workflow.read_text(encoding="utf-8"))
    except (OSError, yaml.YAMLError) as exc:
        return [f"{workflow}: unreadable ({exc})"], 0
    if not isinstance(wf, dict):
        return [f"{workflow}: not a workflow mapping"], 0

    found = 0
    mentions = 0
    executed: set[str] = set()
    for job_name, job in (wf.get("jobs") or {}).items():
        if not isinstance(job, dict):
            continue
        job_if = job.get("if")
        for step in job.get("steps") or []:
            if not isinstance(step, dict):
                continue
            run = step.get("run") or ""
            for target in require_exec:
                if executes_target(run, target):
                    executed.add(target)
            if not any(n in run for n in needles):
                continue
            # MENTIONS the gate: subject to every neutering check below,
            # including the step that runs this checker.
            mentions += 1
            # RUNS the gate: counted against --min-invocations. Resolved at
            # command position, never by substring — see the module docstring.
            if any(executes_target(run, n) for n in needles):
                found += 1
            where = f"{job_name}/{step.get('name', '?')}"
            if step.get("if") is not None:
                findings.append(f"{where}: step carries an `if:` ({step['if']!r}) so it can be skipped")
            if job_if is not None:
                findings.append(f"{where}: job carries an `if:` ({job_if!r}) so the whole gate can be skipped")
            if step.get("continue-on-error"):
                findings.append(f"{where}: continue-on-error swallows the failure")
            shell = step.get("shell")
            if shell is not None and shell not in SAFE_SHELLS:
                findings.append(
                    f"{where}: `shell: {shell}` is not the default bash -eo pipefail, "
                    f"so a failing command may not fail the step"
                )
            if RELAX_ERRMODE_RE.search(run):
                findings.append(
                    f"{where}: the run body relaxes the shell error mode (`set +…`), "
                    f"which un-gates every command that is not last"
                )
            for line in logical_lines(run):
                if not any(n in line for n in needles):
                    continue
                if "||" in line:
                    findings.append(
                        f"{where}: `||` fallback on the gate's command discards its "
                        f"exit status — `{line.strip()[:120]}`"
                    )
    for target in require_exec:
        if target not in executed:
            findings.append(
                f"{workflow.name}: {target} is REFERENCED but never EXECUTED at command "
                f"position. Being named by `shellcheck` (or any other tool that takes it "
                f"as an argument) is not the same as being RUN — deleting the one line "
                f"that runs it leaves every mention in place and every assertion inside "
                f"it silently uncovered."
            )
    if found < minimum:
        # Name the mention/execution gap explicitly. A needle that cannot
        # resolve at command position (e.g. one written with its arguments
        # attached) would otherwise fail here with no clue why — fail-closed
        # is right, fail-closed and undiagnosable is not.
        extra = ""
        if mentions > found:
            extra = (
                f"; {mentions - found} further step(s) NAME a needle without running it "
                f"at command position — being named by `shellcheck`, `cat`, or this "
                f"checker's own `--needle` argument is not being RUN"
            )
        findings.append(
            f"{workflow.name}: expected at least {minimum} invocation(s) of "
            f"{'/'.join(needles)}, found {found} — a call site was deleted{extra}"
        )
    return findings, found


# --------------------------------------------------------------------------
# Hermetic teeth. Each fixture is a MUTANT that must be caught; the clean one
# must pass. Without the clean case these would all pass a checker that simply
# always fails.
# --------------------------------------------------------------------------
_CLEAN = """
jobs:
  quick-gates:
    steps:
      - name: gate
        run: python3 scripts/gate.py --check
"""

_MUTANTS: dict[str, tuple[str, str]] = {
    "Y11 `|| true` on a backslash-continuation line": (
        "|| true",
        """
jobs:
  quick-gates:
    steps:
      - name: gate
        run: |
          python3 scripts/gate.py --check \\
            || true
""",
    ),
    "`|| true` on the same line": (
        "|| true",
        """
jobs:
  quick-gates:
    steps:
      - name: gate
        run: python3 scripts/gate.py --check || true
""",
    ),
    "`|| :` on a continuation line": (
        "||",
        """
jobs:
  quick-gates:
    steps:
      - name: gate
        run: |
          python3 scripts/gate.py \\
            --check \\
            || :
""",
    ),
    "step-level `if:`": (
        "step carries an `if:`",
        """
jobs:
  quick-gates:
    steps:
      - name: gate
        if: github.event_name == 'never'
        run: python3 scripts/gate.py --check
""",
    ),
    "job-level `if:`": (
        "job carries an `if:`",
        """
jobs:
  quick-gates:
    if: false
    steps:
      - name: gate
        run: python3 scripts/gate.py --check
""",
    ),
    "continue-on-error": (
        "continue-on-error",
        """
jobs:
  quick-gates:
    steps:
      - name: gate
        continue-on-error: true
        run: python3 scripts/gate.py --check
""",
    ),
    "`set +e` in the run body": (
        "relaxes the shell error mode",
        """
jobs:
  quick-gates:
    steps:
      - name: gate
        run: |
          set +e
          python3 scripts/gate.py --check
          echo done
""",
    ),
    "non-default shell drops -e": (
        "not the default bash",
        """
jobs:
  quick-gates:
    steps:
      - name: gate
        shell: bash --norc {0}
        run: python3 scripts/gate.py --check
""",
    ),
    "call site deleted": (
        "call site was deleted",
        """
jobs:
  quick-gates:
    steps:
      - name: something else
        run: echo hello
""",
    ),
}


def self_test() -> int:
    failures: list[str] = []
    # Counted, never hard-coded: the summary line used to say "9 mutants
    # killed" from `len(_MUTANTS)` alone, which would have kept saying 9 as
    # cases were added below it. A reassuring summary that does not track what
    # it summarises is the same defect class this script exists to catch.
    mutants = 0
    needles = ("scripts/gate.py",)
    with tempfile.TemporaryDirectory() as td:
        wf = Path(td) / "wf.yml"

        wf.write_text(_CLEAN, encoding="utf-8")
        findings, found = seam_findings(wf, needles, 1)
        if findings:
            failures.append(f"a correctly-wired gate was flagged: {findings}")
        if found != 1:
            failures.append(f"clean fixture: expected 1 invocation, found {found}")

        for name, (expect_substr, body) in _MUTANTS.items():
            mutants += 1
            wf.write_text(body, encoding="utf-8")
            findings, _ = seam_findings(wf, needles, 1)
            if not findings:
                failures.append(f"MUTANT SURVIVED: {name}")
            elif not any(expect_substr in f for f in findings):
                failures.append(f"{name}: caught, but not for the right reason: {findings}")

        # A `||` on a line that does NOT run the gate is legitimate (e.g. a
        # best-effort `git fetch … || true` preceding it) and must not be
        # flagged, or the checker becomes unusable and gets deleted.
        wf.write_text(
            """
jobs:
  quick-gates:
    steps:
      - name: gate
        run: |
          git fetch --quiet --depth=5 origin main || true
          python3 scripts/gate.py --check
""",
            encoding="utf-8",
        )
        findings, _ = seam_findings(wf, needles, 1)
        if findings:
            failures.append(f"a `|| true` on a NON-gate line was wrongly flagged: {findings}")

        # --- THE META-STEP: not counted, but NOT exempt -------------------
        # The step that runs this checker names the gate in its own `--needle`
        # argument. It must not be counted as a gate invocation (or deleting a
        # real call site still clears the floor), and it must STILL be subject
        # to every neutering check (or one additive `continue-on-error: true`
        # disarms the gate while the step prints "OK"). The retired
        # `--exclude <filename>` flag got the first half right and the second
        # half catastrophically wrong; command position gets both, with no
        # self-exemption to misapply.
        meta_step = "      - name: seam\n        run: python3 scripts/check-workflow-seam.py --needle scripts/gate.py\n"
        mutants += 1
        wf.write_text("jobs:\n  quick-gates:\n    steps:\n" + meta_step, encoding="utf-8")
        findings, found = seam_findings(wf, needles, 1)
        if found != 0:
            failures.append(
                f"the meta-step's own `--needle` argument was counted as an invocation (found={found})"
            )
        if not any("call site was deleted" in f for f in findings):
            failures.append(f"a workflow with zero real call sites was not flagged: {findings}")
        if not any("is not being RUN" in f for f in findings):
            failures.append(f"the mention-vs-execution gap was not diagnosed: {findings}")

        # The BLOCKING-3 pin, one case per neutering form. Each of these
        # SURVIVED on the real workflow under `--exclude`, with the seam step
        # reporting "4 invocation(s) … OK" every time.
        real_call = "      - name: gate\n        run: python3 scripts/gate.py --check\n"
        for why, neutered, expect_substr in (
            ("continue-on-error", "      - name: seam\n        continue-on-error: true\n"
             "        run: python3 scripts/check-workflow-seam.py --needle scripts/gate.py\n",
             "continue-on-error"),
            ("step `if:`", "      - name: seam\n        if: github.event_name == 'never'\n"
             "        run: python3 scripts/check-workflow-seam.py --needle scripts/gate.py\n",
             "step carries an `if:`"),
            ("non-default `shell:`", "      - name: seam\n        shell: bash -x {0}\n"
             "        run: python3 scripts/check-workflow-seam.py --needle scripts/gate.py\n",
             "not the default bash"),
            ("`set +e`", "      - name: seam\n        run: |\n          set +e\n"
             "          python3 scripts/check-workflow-seam.py --needle scripts/gate.py\n"
             "          echo done\n",
             "relaxes the shell error mode"),
            ("`|| true`", "      - name: seam\n        run: |\n"
             "          python3 scripts/check-workflow-seam.py --needle scripts/gate.py \\\n"
             "            || true\n",
             "discards its exit status"),
        ):
            mutants += 1
            wf.write_text(
                "jobs:\n  quick-gates:\n    steps:\n" + real_call + neutered, encoding="utf-8"
            )
            findings, found = seam_findings(wf, needles, 1)
            if found != 1:
                failures.append(f"meta-step neutered by {why}: expected 1 invocation, found {found}")
            if not any(expect_substr in f for f in findings):
                failures.append(
                    f"MUTANT SURVIVED: the meta-step neutered by {why} was not flagged: {findings}"
                )

        # The job-level `if:` form, which sits on the job rather than the step.
        mutants += 1
        wf.write_text(
            "jobs:\n  quick-gates:\n    if: false\n    steps:\n" + real_call + meta_step,
            encoding="utf-8",
        )
        findings, _ = seam_findings(wf, needles, 1)
        if not any("job carries an `if:`" in f for f in findings):
            failures.append(f"MUTANT SURVIVED: a job-level `if:` over the meta-step: {findings}")

        # ...and the control, so the five rows above red for their mutation and
        # not because a meta-step is flagged unconditionally.
        wf.write_text("jobs:\n  quick-gates:\n    steps:\n" + real_call + meta_step, encoding="utf-8")
        findings, found = seam_findings(wf, needles, 1)
        if findings or found != 1:
            failures.append(
                f"an UNNEUTERED meta-step alongside a real call site was flagged "
                f"(found={found}, findings={findings})"
            )

        # --- --require-exec: MENTIONED is not EXECUTED --------------------
        # The defect this exists for: deleting the single `bash <suite>` line
        # leaves `shellcheck <suite>` in place, so every filename-based check
        # still matches and every assertion inside the suite silently stops
        # running. Both directions, because a checker that always fails would
        # "pass" the mutant.
        target = "scripts/tests/suite.sh"
        wired = """
jobs:
  quick-gates:
    steps:
      - name: gate
        run: python3 scripts/gate.py --check
      - name: suite
        run: |
          shellcheck scripts/tests/suite.sh
          bash scripts/tests/suite.sh
"""
        wf.write_text(wired, encoding="utf-8")
        findings, _ = seam_findings(wf, needles, 1, (target,))
        if findings:
            failures.append(f"an EXECUTED suite was reported as an orphan: {findings}")

        mutants += 1
        wf.write_text(wired.replace("          bash scripts/tests/suite.sh\n", ""), encoding="utf-8")
        findings, _ = seam_findings(wf, needles, 1, (target,))
        if not any("never EXECUTED at command position" in f for f in findings):
            failures.append(
                "MUTANT SURVIVED: the one-line deletion of `bash <suite>` left "
                f"`shellcheck <suite>` behind and was not caught: {findings}"
            )

        # An interpreter that merely takes the path as a flag VALUE is not
        # running it, and a tool that takes it as an argument never was.
        for body, why in (
            ("        run: shellcheck scripts/tests/suite.sh\n", "shellcheck-only"),
            ("        run: echo scripts/tests/suite.sh\n", "echoed"),
            ("        run: cat scripts/tests/suite.sh > /dev/null\n", "cat-ed"),
        ):
            wf.write_text(
                "jobs:\n  quick-gates:\n    steps:\n      - name: gate\n"
                "        run: python3 scripts/gate.py --check\n      - name: x\n" + body,
                encoding="utf-8",
            )
            findings, _ = seam_findings(wf, needles, 1, (target,))
            if not any("never EXECUTED at command position" in f for f in findings):
                failures.append(f"a {why} reference was accepted as an execution")

        # ...and the forms that ARE an execution must be accepted, or the flag
        # is unusable and gets removed.
        for body, why in (
            ("        run: bash scripts/tests/suite.sh\n", "bash <path>"),
            ("        run: ./scripts/tests/suite.sh\n", "./<path>"),
            ("        run: bash -x scripts/tests/suite.sh\n", "bash -x <path>"),
            ("        run: FOO=1 bash scripts/tests/suite.sh\n", "env-prefixed"),
        ):
            wf.write_text(
                "jobs:\n  quick-gates:\n    steps:\n      - name: gate\n"
                "        run: python3 scripts/gate.py --check\n      - name: x\n" + body,
                encoding="utf-8",
            )
            findings, _ = seam_findings(wf, needles, 1, (target,))
            if findings:
                failures.append(f"a genuine execution ({why}) was reported as an orphan: {findings}")

    if failures:
        print("check-workflow-seam.py self-test FAILED:")
        for f in failures:
            print(f"  - {f}")
        return 1
    print(f"check-workflow-seam.py self-test: OK ({mutants} mutants killed)")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--self-test", action="store_true", help="run hermetic both-direction teeth")
    ap.add_argument("--workflow", type=Path, help="workflow file to check")
    ap.add_argument("--needle", action="append", default=[], help="path of the gate; steps EXECUTING it at command position are invocations, steps merely mentioning it are still checked for neutering (repeatable)")
    ap.add_argument("--min-invocations", type=int, default=1, help="fail if fewer EXECUTING steps than this exist")
    ap.add_argument("--require-exec", action="append", default=[], help="require this script to be EXECUTED at command position, not merely mentioned (repeatable)")
    # `--exclude <filename>` is RETIRED, not renamed. It exempted the step that
    # runs this checker from the invocation COUNT and — the defect — from every
    # neutering check as well, so one additive `continue-on-error: true` on the
    # gating seam step disarmed the gate while it printed "OK". Command-position
    # resolution makes the exemption unnecessary. Rejecting the flag loudly is
    # deliberate: an unrecognised argument is a hard argparse failure (rc=2), so
    # a stale call site reds instead of silently re-opening the hole.
    args = ap.parse_args()

    if args.self_test:
        return self_test()
    if not args.workflow or not args.needle:
        ap.print_help()
        return 2
    if not args.workflow.is_file():
        print(f"check-workflow-seam: {args.workflow} not found")
        return 1

    findings, found = seam_findings(
        args.workflow, tuple(args.needle), args.min_invocations,
        tuple(args.require_exec),
    )
    if findings:
        print(f"check-workflow-seam: {args.workflow} — the gate is not soundly wired:")
        for f in findings:
            print(f"  {f}")
        return 1
    print(
        f"check-workflow-seam: {args.workflow} — {found} invocation(s), each unconditional, "
        f"non-swallowed, and under the default error-exiting shell — OK"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
