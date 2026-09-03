#!/usr/bin/env python3
# [OPUS-5] 🤖 SPARQ agent — sparq-org/sparq#6096. Suite for the step-level YAML
# seam guard (scripts/check-workflow-gate-steps.py).
#
# The guard's own `--self-test` runs the S0–S4 mutation table over synthetic
# workflows. This suite covers what a synthetic tree cannot:
#
#   * the LIVE invariant — the real .github/workflows tree is clean under all
#     five rules, so the gate is green on merge rather than red on arrival;
#   * VACUITY — the detector really does see the repo (a scan that matches
#     nothing would pass every rule trivially, which is the exact failure mode
#     the guard exists to prevent, so it must not be possible to pass by seeing
#     nothing);
#   * the DETECTOR's boundaries — a checker merely mentioned in a comment or an
#     echo is not "invoked", and each supported invocation spelling is;
#   * the guard is itself WIRED into a gating job. Without this, the guard is
#     the very thing it detects: a checker nothing runs.
#
# Run:  python3 scripts/tests/test_workflow_gate_steps.py
# (stdlib unittest + PyYAML; no gh, no git, no network.)

from __future__ import annotations

import importlib.util
import json
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]


def _load(module_name: str, filename: str):
    spec = importlib.util.spec_from_file_location(
        module_name, REPO_ROOT / "scripts" / filename
    )
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


GS = _load("check_workflow_gate_steps", "check-workflow-gate-steps.py")


class TestLiveTreeIsClean(unittest.TestCase):
    """The real workflow tree must satisfy every rule."""

    def test_the_live_tree_has_no_violations(self):
        failures = GS.check(REPO_ROOT)
        self.assertEqual(
            failures, [],
            "the live .github/workflows tree violates the gate-step seam rules:\n  "
            + "\n  ".join(failures),
        )


class TestNotVacuous(unittest.TestCase):
    """A guard that sees nothing passes everything. That must be impossible."""

    @classmethod
    def setUpClass(cls):
        cls.steps = GS.collect_gate_steps(REPO_ROOT)

    # Deliberately LITERAL, not `GS.MIN_GATE_STEPS`. Asserting against the
    # module constant would be self-referential: lowering the floor to 0 would
    # satisfy the assertion instead of failing it, so the guard's own anti-vacuity
    # defence would have no test. These are the floors as reviewed in #6096;
    # raising them is fine, lowering them must break this test.
    REVIEWED_STEP_FLOOR = 100
    REVIEWED_CHECKER_FLOOR = 60

    def test_the_scan_finds_the_repo(self):
        self.assertGreaterEqual(
            len(self.steps), self.REVIEWED_STEP_FLOOR,
            "the gate-step detector matched fewer steps than the reviewed floor — "
            "fix the scan, do not lower the floor",
        )
        self.assertGreaterEqual(
            len(GS._checker_paths(REPO_ROOT)), self.REVIEWED_CHECKER_FLOOR
        )

    def test_the_floors_have_not_been_lowered(self):
        # The floors are the only thing standing between "the detector broke" and
        # "everything passes". Weakening them is the cheapest way to green a
        # broken scan, so it must be a visible test failure, not a one-line edit.
        self.assertGreaterEqual(GS.MIN_GATE_STEPS, self.REVIEWED_STEP_FLOOR)
        self.assertGreaterEqual(GS.MIN_CHECKERS, self.REVIEWED_CHECKER_FLOOR)

    def test_a_scan_that_finds_nothing_fails(self):
        # Delete the floors' teeth and the guard would green a repo with no
        # workflows at all. Assert the floors actually fire.
        import tempfile

        with tempfile.TemporaryDirectory() as tmp:
            empty = Path(tmp) / "root"
            (empty / GS.WORKFLOW_DIR).mkdir(parents=True)
            (empty / "scripts").mkdir(parents=True)
            failures = GS.check(empty)
        self.assertTrue(
            any(f.startswith("VACUOUS") for f in failures),
            "an empty tree passed the guard — the vacuity floors are not enforced, "
            f"so a parser regression would silently green everything. Got: {failures}",
        )

    def test_a_real_gating_lane_is_in_the_population(self):
        # Specificity, not just a count: the lane #6096 names must be covered, or
        # the floors could be met entirely by steps that are not gates.
        keys = {step.key for step in self.steps if step.gating}
        self.assertTrue(
            any("docs-quality.yml" in key for key in keys),
            "no gating docs-quality step in the population — the detector is not "
            "seeing the repo's highest-frequency gate lane",
        )


class TestTheGuardIsItselfWired(unittest.TestCase):
    """The guard must be run by a gating job, or it is what it detects."""

    def test_the_guard_and_this_suite_run_in_a_gating_job(self):
        gating_scripts: set[str] = set()
        for step in GS.collect_gate_steps(REPO_ROOT):
            if step.gating:
                gating_scripts |= step.scripts
        for path in (
            "scripts/check-workflow-gate-steps.py",
            "scripts/tests/test_workflow_gate_steps.py",
        ):
            self.assertIn(
                path, gating_scripts,
                f"{path} is not invoked by any non-advisory job of a "
                "pull_request/merge_group workflow — its assertions are dead",
            )


class TestDetectorBoundaries(unittest.TestCase):
    """What counts as invoking a checker, and what does not."""

    def test_each_supported_spelling_counts_as_an_invocation(self):
        for body in (
            "python3 scripts/check-thing.py",
            "python scripts/check-thing.py --strict",
            "  scripts/check-thing.py\n",
            "set -euo pipefail\npython3 scripts/check-thing.py",
            "foo && python3 scripts/check-thing.py",
            "./scripts/check-thing.py",
        ):
            self.assertIn(
                "scripts/check-thing.py", GS._invoked_scripts(body),
                f"invocation spelling not detected: {body!r}",
            )

    def test_a_mere_mention_is_not_an_invocation(self):
        # Otherwise a step that only NAMES a checker in a comment or an error
        # message would satisfy S3, and deleting the real invocation would be
        # silent again — the precise hole #6096 reports.
        for body in (
            'echo "see scripts/check-thing.py for details"',
            "# scripts/check-thing.py used to run here",
        ):
            self.assertNotIn(
                "scripts/check-thing.py", GS._invoked_scripts(body),
                f"a mention was counted as an invocation: {body!r}",
            )

    def test_the_self_test_spelling_counts(self):
        self.assertIn(
            "scripts/triage-area.py",
            GS._invoked_scripts("python3 scripts/triage-area.py --self-test"),
        )

    def test_bare_on_key_is_parsed_despite_yaml_1_1(self):
        # `on:` parses as the boolean True in YAML 1.1. Reading only doc["on"]
        # would make every workflow look non-PR-triggered, S3 would fire on
        # ~100 checkers, and the natural "fix" would be to weaken S3.
        self.assertEqual(GS._triggers({True: {"pull_request": None}}), {"pull_request"})
        self.assertEqual(GS._triggers({"on": ["push", "pull_request"]}),
                         {"push", "pull_request"})
        self.assertEqual(GS._triggers({"on": "push"}), {"push"})


class TestMutationTable(unittest.TestCase):
    """Every mutant #6096 measured as surviving must be caught."""

    def test_every_declared_mutant_is_caught(self):
        for label, expected, workflow, registry, advisory in GS.SELF_TEST_CASES:
            with self.subTest(mutant=label):
                failures = GS.check_fixture(workflow, registry, advisory)
                if expected == "":
                    self.assertEqual(failures, [], f"{label}: expected a clean tree")
                else:
                    self.assertTrue(
                        any(f.startswith(expected) for f in failures),
                        f"{label}: the mutant SURVIVED — expected a {expected} "
                        f"failure, got {failures or 'nothing'}",
                    )

    def test_the_table_covers_all_three_mutants_the_issue_names(self):
        # A guard whose table quietly lost a case would still pass the loop above.
        labels = " ".join(case[0] for case in GS.SELF_TEST_CASES)
        for needle in ("deleted gate step", "continue-on-error", "if"):
            self.assertIn(needle, labels)
        rules = {case[1] for case in GS.SELF_TEST_CASES}
        self.assertEqual(rules, {"", "S0", "S1", "S2", "S3", "S4"},
                         "every rule must have at least one negative fixture")

    def test_an_explicit_continue_on_error_false_is_allowed(self):
        # The rule is about SWALLOWING a failure. Writing the default out
        # explicitly is not a mutant, and flagging it would push authors to
        # delete the reassuring line.
        workflow = GS._BASE_WORKFLOW.replace(
            "      - name: Enforce the thing\n",
            "      - name: Enforce the thing\n        continue-on-error: false\n",
        )
        self.assertEqual(GS.check_fixture(workflow), [])

    def test_a_continue_on_error_expression_is_not_trusted(self):
        # `${{ ... }}` cannot be statically proven false, so it must not pass.
        workflow = GS._BASE_WORKFLOW.replace(
            "      - name: Enforce the thing\n",
            "      - name: Enforce the thing\n"
            "        continue-on-error: ${{ github.event_name == 'push' }}\n",
        )
        self.assertTrue(any(f.startswith("S1") for f in GS.check_fixture(workflow)))


class TestRegistryHygiene(unittest.TestCase):
    """The registry is only defensible if every entry carries a real reason."""

    @classmethod
    def setUpClass(cls):
        cls.data = json.loads(
            (REPO_ROOT / GS.REGISTRY_RELPATH).read_text(encoding="utf-8")
        )

    def test_every_declaration_has_a_substantive_reason(self):
        for section in ("conditions", "unwired"):
            for key, entry in (self.data.get(section) or {}).items():
                reason = str(entry.get("reason") or "")
                self.assertGreater(
                    len(reason.strip()), 40,
                    f"{section}.{key} has no substantive reason — an exemption "
                    "without an argument is just a disabled rule",
                )

    def test_every_condition_declaration_pins_an_expression(self):
        for key, entry in (self.data.get("conditions") or {}).items():
            self.assertTrue(str(entry.get("if") or "").strip(),
                            f"conditions.{key} declares no `if:` to pin against")

    def test_the_registry_grants_no_continue_on_error_waiver(self):
        # S1 has no waiver on purpose: the reviewed way to make a check
        # non-blocking is .github/advisory-registry.json. If a waiver section
        # ever appears here, that decision was reversed without review.
        self.assertEqual(
            set(self.data) - {"_doc"}, {"conditions", "unwired"},
            "an unexpected section appeared in the gate-step registry",
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
