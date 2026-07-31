#!/usr/bin/env python3
# [OPUS-5] 🤖 SPARQ agent — the YAML seam of the stale-analysis thread sweep (issue #4542).
#
# scripts/stale-analysis-threads.py's own --self-test proves the POLICY: a thread is
# resolved only when the analysis that filed it is provably disabled. Every assertion in it
# is worthless if the workflow that runs it is wired wrong, and measured in this repository
# every mutant that survived a python-only pin lived at the workflow seam — a job-level
# `if: false`, a deleted step, a `--dry-run` that was never removed, a self-test nobody
# invokes (#5080, and rearm-sweeper's own dry-run pin).
#
# So this suite reads BOTH workflows STRUCTURALLY (PyYAML, never substrings):
#   * the sweep lane can never run on a PR head — it would otherwise register a check-run
#     and change what `ci-summary / gate` waits for;
#   * it holds exactly the three permissions its three operations need;
#   * the self-test runs BEFORE the first mutation, and neither the job nor any step is
#     conditionally skippable;
#   * the production sweep is NOT --dry-run (a lane permanently in dry-run reports
#     beautifully and repairs nothing) and IS bounded (--max-resolves);
#   * docs-quality.yml really invokes the policy self-test AND this suite, so neither can
#     go quietly dead.
#
# Run: python3 scripts/tests/test_stale_analysis_threads_wiring.py

from __future__ import annotations

import re
import unittest
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SWEEP_WF = REPO_ROOT / ".github" / "workflows" / "stale-analysis-threads.yml"
DOCS_WF = REPO_ROOT / ".github" / "workflows" / "docs-quality.yml"
POLICY = "scripts/stale-analysis-threads.py"
SUITE = "scripts/tests/test_stale_analysis_threads_wiring.py"
SHA_PIN = re.compile(r"^[^@]+@[0-9a-f]{40}$")


def load(path: Path) -> dict:
    return yaml.safe_load(path.read_text(encoding="utf-8"))


def on_block(workflow: dict) -> dict:
    """Bare `on` is the YAML boolean True, so PyYAML keys it under `True`."""
    return workflow.get("on", workflow.get(True, {})) or {}


def run_text(step: dict) -> str:
    return str(step.get("run") or "")


class TestSweepWorkflow(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.wf = load(SWEEP_WF)
        cls.job = cls.wf["jobs"]["sweep"]
        cls.steps = cls.job["steps"]

    def test_never_runs_on_a_pr_head(self) -> None:
        # A `pull_request`/`push`/`merge_group` trigger would put a check-run on a head the
        # required gate polls, and would run a MUTATING lane from a PR ref.
        self.assertEqual(set(on_block(self.wf)), {"schedule", "workflow_dispatch"})
        crons = [entry["cron"] for entry in on_block(self.wf)["schedule"]]
        self.assertTrue(crons and all(len(cron.split()) == 5 for cron in crons), crons)

    def test_permissions_are_exactly_the_three_operations(self) -> None:
        # actions:read is the sweep's whole authority (the workflow `state` read);
        # pull-requests:write is what resolveReviewThread + the receipt consume;
        # contents:read is the policy checkout. Anything more is unearned.
        self.assertEqual(
            self.wf["permissions"],
            {"contents": "read", "pull-requests": "write", "actions": "read"},
        )

    def test_job_and_steps_are_not_conditionally_skippable(self) -> None:
        self.assertNotIn("if", self.job)
        self.assertIn("timeout-minutes", self.job)
        for step in self.steps:
            self.assertNotIn("if", step, step.get("name"))
            self.assertNotIn("continue-on-error", step, step.get("name"))

    def test_checkout_is_sha_pinned_and_takes_the_whole_scripts_directory(self) -> None:
        checkout = self.steps[0]
        self.assertRegex(checkout["uses"], SHA_PIN)
        with_ = checkout["with"]
        self.assertEqual(with_["sparse-checkout"], "scripts")
        self.assertIs(with_["persist-credentials"], False)
        self.assertIn("default_branch", str(with_["ref"]))

    def test_self_test_runs_before_the_first_mutation(self) -> None:
        selftest = [i for i, s in enumerate(self.steps) if f"{POLICY} --self-test" in run_text(s)]
        sweep = [
            i for i, s in enumerate(self.steps)
            if POLICY in run_text(s) and "--self-test" not in run_text(s)
        ]
        self.assertEqual(len(selftest), 1, "exactly one policy self-test step")
        self.assertEqual(len(sweep), 1, "exactly one sweep step")
        self.assertLess(selftest[0], sweep[0])

    def test_the_production_sweep_is_live_and_bounded(self) -> None:
        step = next(
            s for s in self.steps if POLICY in run_text(s) and "--self-test" not in run_text(s)
        )
        run = run_text(step)
        # A permanently dry lane classifies beautifully and unblocks nothing.
        self.assertNotIn("--dry-run", run)
        self.assertIn("--max-resolves", run)
        self.assertIn('--repo "$REPO"', run)
        self.assertEqual(step["env"]["REPO"], "${{ github.repository }}")
        self.assertIn("github.token", str(step["env"]["GH_TOKEN"]))


class TestSelfTestIsActuallyInvoked(unittest.TestCase):
    """A self-test nobody runs cannot go red — pin both invocations at their seam."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.wf = load(DOCS_WF)
        cls.job = cls.wf["jobs"]["quick-gates"]

    def _step_running(self, needle: str) -> dict:
        matches = [s for s in self.job["steps"] if needle in run_text(s)]
        self.assertEqual(len(matches), 1, f"{needle} must be invoked exactly once")
        return matches[0]

    def test_quick_gates_is_not_skippable(self) -> None:
        self.assertNotIn("if", self.job)

    def test_docs_quality_runs_the_policy_self_test(self) -> None:
        step = self._step_running(f"{POLICY} --self-test")
        self.assertNotIn("if", step)
        self.assertNotIn("continue-on-error", step)

    def test_docs_quality_runs_this_suite(self) -> None:
        step = self._step_running(SUITE)
        self.assertNotIn("if", step)
        self.assertNotIn("continue-on-error", step)


if __name__ == "__main__":
    unittest.main(verbosity=2)
