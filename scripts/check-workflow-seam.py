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


def seam_findings(
    workflow: Path,
    needles: tuple[str, ...],
    minimum: int,
    excludes: tuple[str, ...] = (),
) -> tuple[list[str], int]:
    """``(findings, invocations_found)`` for one workflow file.

    ``excludes`` drops steps that merely NAME the gate without running it — in
    practice the step that runs this very checker, whose ``--needle`` arguments
    are the gate's own filename. Without it that step would be counted as a
    gate invocation and would mask the deletion of a real one. Kept as an
    explicit, greppable flag rather than an implicit self-exemption.
    """
    findings: list[str] = []
    try:
        wf = yaml.safe_load(workflow.read_text(encoding="utf-8"))
    except (OSError, yaml.YAMLError) as exc:
        return [f"{workflow}: unreadable ({exc})"], 0
    if not isinstance(wf, dict):
        return [f"{workflow}: not a workflow mapping"], 0

    found = 0
    for job_name, job in (wf.get("jobs") or {}).items():
        if not isinstance(job, dict):
            continue
        job_if = job.get("if")
        for step in job.get("steps") or []:
            if not isinstance(step, dict):
                continue
            run = step.get("run") or ""
            if not any(n in run for n in needles):
                continue
            if any(x in run for x in excludes):
                continue
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
    if found < minimum:
        findings.append(
            f"{workflow.name}: expected at least {minimum} invocation(s) of "
            f"{'/'.join(needles)}, found {found} — a call site was deleted"
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

        # --exclude must not become a way to hide a deleted call site: a
        # workflow whose ONLY mention of the gate is the excluded meta-step has
        # zero real invocations and must red.
        wf.write_text(
            """
jobs:
  quick-gates:
    steps:
      - name: seam
        run: python3 scripts/check-workflow-seam.py --needle scripts/gate.py
""",
            encoding="utf-8",
        )
        findings, found = seam_findings(wf, needles, 1, ("check-workflow-seam.py",))
        if found != 0 or not any("call site was deleted" in f for f in findings):
            failures.append(
                f"--exclude masked a deleted call site (found={found}, findings={findings})"
            )

    if failures:
        print("check-workflow-seam.py self-test FAILED:")
        for f in failures:
            print(f"  - {f}")
        return 1
    print(f"check-workflow-seam.py self-test: OK ({len(_MUTANTS)} mutants killed)")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--self-test", action="store_true", help="run hermetic both-direction teeth")
    ap.add_argument("--workflow", type=Path, help="workflow file to check")
    ap.add_argument("--needle", action="append", default=[], help="substring identifying the gate's invocations (repeatable)")
    ap.add_argument("--min-invocations", type=int, default=1, help="fail if fewer matching steps than this exist")
    ap.add_argument("--exclude", action="append", default=[], help="ignore steps whose run body contains this (e.g. the step running THIS checker)")
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
        args.workflow, tuple(args.needle), args.min_invocations, tuple(args.exclude)
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
