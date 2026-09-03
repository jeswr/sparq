#!/usr/bin/env python3
# [OPUS-5] 🤖 SPARQ agent — sparq-org/sparq#6096. The step-level YAML seam.
#
# =============================================================================
# WHAT THIS EXISTS FOR
# =============================================================================
# Mutation coverage in this repo stops at the Python/shell boundary. In the
# measurement cited in #6096 (taken while implementing #4567) 18/18 Python
# mutants died, and EVERY surviving mutant lived in a workflow step, a workflow
# `if:` condition, or a call site. Three edits turn a red gate green with no test
# failing anywhere:
#
#   1. DELETE a gate step        — `python3 scripts/tests/test_triage_area.py`
#                                  disappears from the workflows that carry it;
#                                  the test file still exists, still passes when
#                                  a human runs it, and is simply never executed.
#   2. `continue-on-error: true` — the step runs, fails, and the job stays green.
#   3. NARROW a step-level `if:` — the step never runs. This one is the quietest:
#                                  `ci-summary` treats `skipped` as non-failing
#                                  BY DESIGN, so nothing anywhere goes red.
#
# The pre-existing defence is per-lane seam fixtures (e.g.
# scripts/tests/test_readiness_visibility.py::TestAreaClassifierCronSeam pins
# triage-area.yml step by step). Those work, but they only protect the lanes
# somebody thought to write a fixture for — the class is repo-wide and the
# fixtures are opt-in, so the DEFAULT for a new gate step is unprotected. #6096
# asks for the general guard instead of more per-lane fixtures. This is it.
#
# =============================================================================
# WHAT A "GATE STEP" IS
# =============================================================================
# A `run:` step in any workflow whose script INVOKES one of the repo's own
# checkers or self-tests:
#
#     scripts/check-*.py|sh            scripts/tests/test_*.py|sh
#     <any scripts/*.py|sh> --self-test
#
# That is the population whose whole purpose is to fail. It is deliberately
# derived from the live tree by pattern rather
# than enumerated in a registry: a gate step added tomorrow is protected the
# moment it is written, with nothing to register. Registration is required only
# to make an exception, which is the direction that belongs in a diff.
#
# =============================================================================
# THE RULES
# =============================================================================
#   S0  A gate step must carry a `name:`.  Without one it has no stable identity,
#       so it cannot be declared in the registry and cannot be spoken about in a
#       review comment. Every live gate step already satisfies this.
#
#   S1  A gate step must not swallow its failure — no `continue-on-error` on the
#       step and none on the job that holds it. There is NO waiver for this. The
#       supported way to make a check non-blocking is the reviewed one: put the
#       JOB in .github/advisory-registry.json, where it carries an owner_bead and
#       promotion_criteria and `ci-summary` excludes it by declaration.
#
#   S2  A gate step must be unconditional, unless .github/gate-step-registry.json
#       declares its `if:` expression BYTE-EXACT with a reason. Byte-exact is the
#       point: `if: X` narrowed to `if: X && false` is a different string, so the
#       narrowing mutant reds even though the step is still "declared".
#
#   S3  Every checker in the tree must be invoked by at least one gate step in a
#       NON-advisory job of a workflow that runs on `pull_request`/`merge_group`
#       — i.e. by a step that can actually red the required `ci-summary / gate`.
#       Deleting the last such invocation of a checker reds here. Exceptions
#       (release-time gates, deliberately advisory soft-launches, indirect
#       invocation from another script) are declared with a reason.
#
#   S4  The registry may not rot. A declaration whose step no longer exists, no
#       longer carries that `if:`, or whose checker is now properly wired, is a
#       hard failure. This is what stops the registry from silently becoming a
#       list of things that used to be true — and it is a second reason a DELETED
#       conditional gate step reds.
#
# =============================================================================
# WHAT THIS DOES **NOT** COVER — read before trusting it
# =============================================================================
#   * JOB-level `if:`.  Measured on the tree this landed against, 12 of the 34
#     jobs carrying gate steps have one, and every one is legitimate
#     trigger-shaping (`github.event_name != 'push'`,
#     path filters, `refs/heads/main` guards). Declaring all of them would make
#     the registry mostly noise and would fight the live path-filter design. The
#     job level is governed by a different, already-reviewed mechanism: the
#     `ci-summary` aggregator plus .github/advisory-registry.json (C2/C3/C4 in
#     scripts/check-advisory-registry.py). A job-level `if: false` is therefore
#     still an uncaught mutant HERE; it is not an uncaught mutant repo-wide only
#     for the lanes with their own seam fixtures.
#   * A checker invoked through a variable, a generated command, or a wrapper
#     script is invisible to the S3 scan and needs an `unwired` declaration
#     (scripts/check-wasm-features.py is exactly this case).
#   * This is a REGRESSION DETECTOR, not a security control. A committer who
#     wants a gate gone can delete the step AND its registry entry in one commit.
#     The value is that the second half is a diff-visible edit to a file whose
#     only purpose is to be read in review, instead of a silent one-line deletion.
#
# USAGE
#   scripts/check-workflow-gate-steps.py             # check the live tree
#   scripts/check-workflow-gate-steps.py --root DIR  # check DIR as repo root
#   scripts/check-workflow-gate-steps.py --self-test # hermetic negative fixtures

from __future__ import annotations

import argparse
import json
import re
import sys
import tempfile
from pathlib import Path
from typing import Any

import yaml

REPO_ROOT = Path(__file__).resolve().parents[1]
REGISTRY_RELPATH = Path(".github") / "gate-step-registry.json"
ADVISORY_RELPATH = Path(".github") / "advisory-registry.json"
WORKFLOW_DIR = Path(".github") / "workflows"

# The triggers that produce a check-run `ci-summary / gate` aggregates on a PR.
GATING_TRIGGERS = frozenset({"pull_request", "merge_group"})

# A checker invocation at the head of a command: optionally an interpreter, then
# the script path. Anchored on a command boundary (line start, `;`, `&&`, `||`,
# a pipe, `then`, `do`) so a path merely MENTIONED in a comment or an echo does
# not count as running it.
_INVOKE_RE = re.compile(
    r"(?:^|[;&|(]|\bthen\b|\bdo\b)[ \t]*"
    r"(?:(?:python3?|bash|sh|uv[ \t]+run)[ \t]+)?"
    r"(?:\./)?"  # `./scripts/x.py` — no space after the `./`, unlike an interpreter
    r"(scripts/(?:tests/test_[\w.-]+|check-[\w.-]+)\.(?:py|sh))\b",
    re.MULTILINE,
)
# `<script> [args] --self-test` — the other spelling of "run the fixtures".
_SELFTEST_RE = re.compile(r"(scripts/[\w./-]+\.(?:py|sh))\b[^\n|;&]*[ \t]--self-test\b")

# Vacuity floors. A regex or parser regression that finds NOTHING would make
# every rule above pass trivially, which is the exact failure mode this whole
# file exists to prevent. Set well under the live counts (a clean run PRINTS
# them, so they are never asserted in a comment that could drift) so ordinary
# churn never trips them. Lowering either is a test failure, not a one-line
# edit — scripts/tests/test_workflow_gate_steps.py pins them independently.
MIN_GATE_STEPS = 100
MIN_CHECKERS = 60


def _checker_paths(root: Path) -> list[str]:
    """Every checker/self-test file in the tree, as repo-relative posix paths."""
    found: set[str] = set()
    for pattern in ("check-*.py", "check-*.sh", "tests/test_*.py", "tests/test_*.sh"):
        for path in (root / "scripts").glob(pattern):
            if path.is_file():
                found.add(path.relative_to(root).as_posix())
    return sorted(found)


def _invoked_scripts(run_body: str) -> set[str]:
    return set(_INVOKE_RE.findall(run_body)) | set(_SELFTEST_RE.findall(run_body))


def _triggers(doc: dict[str, Any]) -> set[str]:
    # YAML 1.1 parses a bare `on:` key as the boolean True, which is why every
    # workflow reader in this repo has to check both spellings.
    on = doc.get("on")
    if on is None:
        on = doc.get(True)
    if on is None:
        return set()
    if isinstance(on, str):
        return {on}
    if isinstance(on, list):
        return {str(item) for item in on}
    if isinstance(on, dict):
        return {str(key) for key in on}
    return set()


def _advisory_jobs(root: Path) -> set[tuple[str, str]]:
    """(workflow file, job_id) pairs declared advisory in the advisory registry."""
    path = root / ADVISORY_RELPATH
    if not path.exists():
        return set()
    data = json.loads(path.read_text(encoding="utf-8"))
    pairs = set()
    for entry in (data.get("jobs") or {}).values():
        workflow, job_id = entry.get("workflow"), entry.get("job_id")
        if workflow and job_id:
            pairs.add((str(workflow), str(job_id)))
    return pairs


class GateStep:
    """One `run:` step that invokes a repo checker."""

    def __init__(self, workflow: str, job_id: str, job: dict, step: dict, gating: bool):
        self.workflow = workflow
        self.job_id = job_id
        self.name = step.get("name")
        self.condition = step.get("if")
        self.step_coe = step.get("continue-on-error")
        self.job_coe = job.get("continue-on-error")
        self.scripts = _invoked_scripts(str(step.get("run", "")))
        # `gating` = this step can red the required `ci-summary / gate`: a
        # PR/merge_group trigger AND a job the advisory registry does not exempt.
        self.gating = gating

    @property
    def key(self) -> str:
        return f"{self.workflow}::{self.job_id}::{self.name}"


def collect_gate_steps(root: Path) -> list[GateStep]:
    advisory = _advisory_jobs(root)
    steps: list[GateStep] = []
    workflow_dir = root / WORKFLOW_DIR
    files = sorted(list(workflow_dir.glob("*.yml")) + list(workflow_dir.glob("*.yaml")))
    for path in files:
        doc = yaml.safe_load(path.read_text(encoding="utf-8"))
        if not isinstance(doc, dict):
            continue
        pr_observed = bool(GATING_TRIGGERS & _triggers(doc))
        for job_id, job in (doc.get("jobs") or {}).items():
            # A `uses:` job (reusable workflow) has no steps of its own.
            if not isinstance(job, dict) or not isinstance(job.get("steps"), list):
                continue
            gating = pr_observed and (path.name, str(job_id)) not in advisory
            for step in job["steps"]:
                if not isinstance(step, dict) or "run" not in step:
                    continue
                if not _invoked_scripts(str(step["run"])):
                    continue
                steps.append(GateStep(path.name, str(job_id), job, step, gating))
    return steps


def _load_registry(root: Path) -> dict[str, Any]:
    path = root / REGISTRY_RELPATH
    if not path.exists():
        return {"conditions": {}, "unwired": {}}
    data = json.loads(path.read_text(encoding="utf-8"))
    return {
        "conditions": data.get("conditions") or {},
        "unwired": data.get("unwired") or {},
    }


def check(root: Path, *, min_steps: int = MIN_GATE_STEPS,
          min_checkers: int = MIN_CHECKERS) -> list[str]:
    """Return a list of failure messages; empty means the tree is clean.

    The floors are parameters only so the hermetic fixtures below (which have two
    steps, not a whole repo) can relax them. The live invocation always uses the
    module constants.
    """
    failures: list[str] = []
    registry = _load_registry(root)
    conditions = registry["conditions"]
    unwired = registry["unwired"]
    steps = collect_gate_steps(root)

    # --- vacuity: a scan that finds nothing must not pass ---------------------
    if len(steps) < min_steps:
        failures.append(
            f"VACUOUS: found only {len(steps)} gate steps (floor {min_steps}). "
            "The detector matched almost nothing, so every rule below passed "
            "trivially — fix the scan, do not lower the floor."
        )
    checkers = _checker_paths(root)
    if len(checkers) < min_checkers:
        failures.append(
            f"VACUOUS: found only {len(checkers)} checkers under scripts/ "
            f"(floor {min_checkers}). The S3 scan is not seeing the tree."
        )

    declared_conditions_seen: set[str] = set()
    for step in steps:
        # --- S0: identity ----------------------------------------------------
        if not step.name:
            failures.append(
                f"S0 {step.workflow}::{step.job_id}: a gate step running "
                f"{sorted(step.scripts)} has no `name:`. Name it — an unnamed step "
                "has no stable identity to declare or to cite in review."
            )
            continue

        # --- S1: swallowed failure -------------------------------------------
        # `continue-on-error: false` is explicit and fine. Anything else — `true`
        # or a `${{ }}` expression this cannot statically prove false — is not.
        if step.step_coe is not False and step.step_coe is not None:
            failures.append(
                f"S1 {step.key}: `continue-on-error: {step.step_coe}` on a gate step. "
                "The step will run, fail, and the job will stay green. To make a "
                "check non-blocking, declare its JOB in "
                f"{ADVISORY_RELPATH.as_posix()} instead — that carries an owner_bead "
                "and promotion_criteria and is what `ci-summary` honours."
            )
        if step.job_coe is not False and step.job_coe is not None:
            failures.append(
                f"S1 {step.key}: job `{step.job_id}` carries "
                f"`continue-on-error: {step.job_coe}`, so every gate step in it is "
                f"advisory in fact but not in {ADVISORY_RELPATH.as_posix()}."
            )

        # --- S2: step-level condition ----------------------------------------
        if step.condition is not None:
            declaration = conditions.get(step.key)
            if declaration is None:
                failures.append(
                    f"S2 {step.key}: undeclared step-level `if: {step.condition}`. "
                    "A narrowed condition makes the step `skipped`, and `ci-summary` "
                    "treats `skipped` as non-failing by design — so nothing goes red. "
                    f"Declare the exact expression in {REGISTRY_RELPATH.as_posix()} "
                    "with a reason, or delete the condition."
                )
            else:
                declared_conditions_seen.add(step.key)
                expected = declaration.get("if")
                if str(expected) != str(step.condition):
                    failures.append(
                        f"S2 {step.key}: the step-level `if:` drifted from its "
                        f"declaration.\n      declared: {expected}\n      actual:   "
                        f"{step.condition}\n      If the narrowing is intended, update "
                        f"{REGISTRY_RELPATH.as_posix()} in the same commit so the "
                        "change is reviewed."
                    )
                if not str(declaration.get("reason") or "").strip():
                    failures.append(
                        f"S2 {step.key}: its declaration has no `reason`. A condition "
                        "on a gate step needs one — it is why the gate is allowed to "
                        "not run."
                    )

    # --- S3: every checker is actually invoked by a gating step ---------------
    gating_scripts: set[str] = set()
    for step in steps:
        if step.gating:
            gating_scripts |= step.scripts
    for checker in checkers:
        if checker in gating_scripts:
            if checker in unwired:
                failures.append(
                    f"S4 {checker}: declared `unwired` in "
                    f"{REGISTRY_RELPATH.as_posix()}, but a gating step now runs it. "
                    "Delete the declaration — a stale exemption hides the next "
                    "regression."
                )
            continue
        declaration = unwired.get(checker)
        if declaration is None:
            failures.append(
                f"S3 {checker}: no step in any `pull_request`/`merge_group` workflow "
                "runs this, outside the advisory registry. Its assertions are dead — "
                "it passes when a human runs it and never runs otherwise. Wire it "
                f"into a gating job, or declare it under `unwired` in "
                f"{REGISTRY_RELPATH.as_posix()} with a reason."
            )
        elif not str(declaration.get("reason") or "").strip():
            failures.append(
                f"S3 {checker}: its `unwired` declaration has no `reason`."
            )

    # --- S4: no stale declarations -------------------------------------------
    for key in sorted(set(conditions) - declared_conditions_seen):
        failures.append(
            f"S4 {key}: declared with a step-level `if:` in "
            f"{REGISTRY_RELPATH.as_posix()}, but no gate step with that "
            "workflow/job/name carries one. The step was renamed, deleted, or made "
            "unconditional — delete the declaration."
        )
    for checker in sorted(unwired):
        if checker not in checkers:
            failures.append(
                f"S4 {checker}: declared `unwired` in "
                f"{REGISTRY_RELPATH.as_posix()}, but no such file exists."
            )

    return failures


# =============================================================================
# Self-test: hermetic negative fixtures. Each mutant below is one of the edits
# #6096 measured as surviving; every one must produce a failure here.
# =============================================================================

_BASE_WORKFLOW = """\
name: demo
on:
  pull_request:
  merge_group:
jobs:
  quick-gates:
    runs-on: ubuntu-latest
    steps:
      - name: Enforce the thing
        run: python3 scripts/check-thing.py
      - name: Self-test the thing
        run: |
          set -euo pipefail
          python3 scripts/tests/test_thing.py
"""


def build_fixture(tmp: Path, workflow: str, registry: dict | None = None,
                  advisory: dict | None = None) -> Path:
    """Write a minimal synthetic repo root: two checkers and one workflow."""
    root = tmp / "root"
    (root / WORKFLOW_DIR).mkdir(parents=True)
    (root / "scripts" / "tests").mkdir(parents=True)
    (root / "scripts" / "check-thing.py").write_text("# fixture\n", encoding="utf-8")
    (root / "scripts" / "tests" / "test_thing.py").write_text("# fixture\n", encoding="utf-8")
    (root / WORKFLOW_DIR / "demo.yml").write_text(workflow, encoding="utf-8")
    if registry is not None:
        (root / REGISTRY_RELPATH).write_text(json.dumps(registry), encoding="utf-8")
    if advisory is not None:
        (root / ADVISORY_RELPATH).write_text(json.dumps(advisory), encoding="utf-8")
    return root


def check_fixture(workflow: str, registry: dict | None = None,
                  advisory: dict | None = None) -> list[str]:
    """Run the rules over a synthetic tree, with the vacuity floors relaxed."""
    with tempfile.TemporaryDirectory() as tmp:
        root = build_fixture(Path(tmp), workflow, registry, advisory)
        return check(root, min_steps=0, min_checkers=0)


def _drop_selftest_step(workflow: str) -> str:
    return workflow.replace(
        "      - name: Self-test the thing\n"
        "        run: |\n"
        "          set -euo pipefail\n"
        "          python3 scripts/tests/test_thing.py\n",
        "",
    )


# (label, expected rule prefix or "" for clean, workflow, registry, advisory).
# Every non-empty row is one of the mutants #6096 measured as SURVIVING today.
SELF_TEST_CASES: list[tuple[str, str, str, dict | None, dict | None]] = [
    ("baseline is clean", "", _BASE_WORKFLOW, None, None),
    # M1 — the deleted step. The test file still exists and still passes when a
    # human runs it; nothing executes it. This is #6096's first bullet.
    ("M1 deleted gate step", "S3", _drop_selftest_step(_BASE_WORKFLOW), None, None),
    ("M2 continue-on-error on the step", "S1",
     _BASE_WORKFLOW.replace(
         "      - name: Enforce the thing\n",
         "      - name: Enforce the thing\n        continue-on-error: true\n"),
     None, None),
    ("M3 continue-on-error on the job", "S1",
     _BASE_WORKFLOW.replace(
         "    runs-on: ubuntu-latest\n",
         "    runs-on: ubuntu-latest\n    continue-on-error: true\n"),
     None, None),
    # M4 — #6096's quietest case: the step reports `skipped`, and `ci-summary`
    # treats `skipped` as non-failing by design.
    ("M4 undeclared step-level if", "S2",
     _BASE_WORKFLOW.replace(
         "      - name: Enforce the thing\n",
         "      - name: Enforce the thing\n        if: false\n"),
     None, None),
    # M5 — the reason S2 is byte-exact rather than presence-only: a DECLARED
    # condition narrowed with `&& false` is still declared.
    ("M5 narrowed declared condition", "S2",
     _BASE_WORKFLOW.replace(
         "      - name: Enforce the thing\n",
         "      - name: Enforce the thing\n        if: github.ref == 'x' && false\n"),
     {"conditions": {"demo.yml::quick-gates::Enforce the thing":
                     {"if": "github.ref == 'x'", "reason": "fixture"}}}, None),
    ("M6 unnamed gate step", "S0",
     _BASE_WORKFLOW.replace("      - name: Enforce the thing\n", "      - "), None, None),
    ("M7 stale condition declaration", "S4", _BASE_WORKFLOW,
     {"conditions": {"demo.yml::quick-gates::Enforce the thing":
                     {"if": "github.ref == 'x'", "reason": "fixture"}}}, None),
    ("M8 stale unwired declaration", "S4", _BASE_WORKFLOW,
     {"unwired": {"scripts/check-thing.py": {"reason": "fixture"}}}, None),
    # M9 — a declaration with no reason is not a declaration.
    ("M9 reasonless condition declaration", "S2",
     _BASE_WORKFLOW.replace(
         "      - name: Enforce the thing\n",
         "      - name: Enforce the thing\n        if: github.ref == 'x'\n"),
     {"conditions": {"demo.yml::quick-gates::Enforce the thing":
                     {"if": "github.ref == 'x'"}}}, None),
    # M10 — the triage-area shape: moving a checker to a cron-only lane leaves it
    # unable to red any PR, so S3 must not accept a non-PR workflow.
    ("M10 cron-only lane does not satisfy S3", "S3",
     _BASE_WORKFLOW.replace("on:\n  pull_request:\n  merge_group:\n",
                            "on:\n  schedule:\n    - cron: '0 * * * *'\n"),
     None, None),
    # M11 — since #3773 the advisory REGISTRY, not the job name, decides gating.
    # A checker that only runs in a declared-advisory job is provably non-gating.
    ("M11 advisory job does not satisfy S3", "S3", _BASE_WORKFLOW, None,
     {"jobs": {"demo quick-gates (advisory)":
               {"workflow": "demo.yml", "job_id": "quick-gates"}}}),
]


def self_test() -> int:
    ok = True
    for label, expected, workflow, registry, advisory in SELF_TEST_CASES:
        failures = check_fixture(workflow, registry, advisory)
        if expected == "":
            if failures:
                ok = False
                print(f"self-test FAILED [{label}]: expected clean, got:\n  "
                      + "\n  ".join(failures))
            else:
                print(f"  ok  {label}")
        elif any(f.startswith(expected) for f in failures):
            print(f"  ok  {label} -> {expected}")
        else:
            ok = False
            print(f"self-test FAILED [{label}]: expected a {expected} failure, got: "
                  f"{failures or 'nothing (the mutant survived)'}")
    if ok:
        print(f"self-test PASSED: {len(SELF_TEST_CASES)} fixtures, every mutant caught.")
        return 0
    return 1


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=REPO_ROOT,
                        help="repo root to check (default: this checkout)")
    parser.add_argument("--self-test", action="store_true",
                        help="run the hermetic negative fixtures and exit")
    args = parser.parse_args(argv)

    if args.self_test:
        return self_test()

    failures = check(args.root)
    if failures:
        print("Workflow gate-step seam violations "
              "(scripts/check-workflow-gate-steps.py, #6096):\n")
        for failure in failures:
            print(f"  - {failure}")
        print(f"\n{len(failures)} violation(s). Each one is an edit that turns a red "
              "gate green with no test failing anywhere.")
        return 1
    steps = collect_gate_steps(args.root)
    gating = sum(1 for step in steps if step.gating)
    print(f"gate-step seam OK: {len(steps)} gate steps ({gating} on the "
          f"`ci-summary / gate` path), {len(_checker_paths(args.root))} checkers, "
          "no swallowed failures, no undeclared conditions, no orphaned checkers.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
