#!/usr/bin/env python3
# [GPT-5.6] #6349 — pin the hosted-runner admission budget for the two scheduled
# formal-safety matrices whose delayed executions can overlap the PR critical path.
"""Hermetic structural guard for scheduled Miri/Kani runner admission.

The test intentionally controls admission, not coverage: ``max-parallel`` delays
matrix entries but does not remove them.  Existing Miri and formal-verification
inventory tests remain authoritative for shard/suite completeness.
"""

from __future__ import annotations

import unittest
from pathlib import Path

import yaml


REPO_ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
DOCS_QUALITY = WORKFLOWS / "docs-quality.yml"

MIRI_MATRIX_CAP = 4
KANI_MATRIX_CAP = 2
# Miri also has one non-matrix doctest job.  Keep the combined scheduled formal
# workload below this ceiling so these informational lanes cannot fill the pool.
COMBINED_FORMAL_CAP = 8
# sparq-org's GitHub Free standard-runner allowance is the conservative policy envelope.
# More than half remains reserved for PR and merge-group checks even at the formal-lane cap.
HOSTED_RUNNER_ALLOWANCE = 20
MIN_PR_AND_MERGE_RESERVE = 12


def _load(name: str) -> dict:
    document = yaml.safe_load((WORKFLOWS / name).read_text(encoding="utf-8"))
    if not isinstance(document, dict) or not isinstance(document.get("jobs"), dict):
        raise AssertionError(f"{name}: workflow must contain a jobs mapping")
    return document


def _job_peak(job_id: str, job: dict) -> int:
    """Return the maximum simultaneously admitted jobs for one workflow job."""

    strategy = job.get("strategy")
    if not isinstance(strategy, dict) or "matrix" not in strategy:
        return 1
    cap = strategy.get("max-parallel")
    if not isinstance(cap, int) or isinstance(cap, bool) or cap < 1:
        raise AssertionError(
            f"{job_id}: every scheduled matrix must declare a positive integer "
            "strategy.max-parallel"
        )
    return cap


class TestNightlyRunnerBudget(unittest.TestCase):
    def test_individual_matrix_caps_are_pinned(self) -> None:
        miri = _load("miri.yml")["jobs"]["miri"]["strategy"]
        kani = _load("kani.yml")["jobs"]["kani"]["strategy"]
        self.assertEqual(miri.get("max-parallel"), MIRI_MATRIX_CAP)
        self.assertEqual(kani.get("max-parallel"), KANI_MATRIX_CAP)
        self.assertIs(miri.get("fail-fast"), False)
        self.assertIs(kani.get("fail-fast"), False)

    def test_combined_peak_counts_matrix_and_fixed_jobs(self) -> None:
        peak = 0
        for workflow_name in ("miri.yml", "kani.yml"):
            for job_id, job in _load(workflow_name)["jobs"].items():
                self.assertIsInstance(job, dict, f"{workflow_name}:{job_id}")
                peak += _job_peak(f"{workflow_name}:{job_id}", job)
        self.assertLessEqual(
            peak,
            COMBINED_FORMAL_CAP,
            "scheduled Miri/Kani admission no longer reserves runner capacity for PRs",
        )
        reserve = HOSTED_RUNNER_ALLOWANCE - peak
        self.assertGreater(
            MIN_PR_AND_MERGE_RESERVE,
            HOSTED_RUNNER_ALLOWANCE / 2,
            "the declared PR/merge reserve itself must remain a majority",
        )
        self.assertGreaterEqual(
            reserve,
            MIN_PR_AND_MERGE_RESERVE,
            "scheduled formal lanes leave less than the pinned PR/merge runner reserve",
        )

    def test_guard_is_wired_into_required_docs_quality(self) -> None:
        command = "python3 scripts/tests/test_nightly_runner_budget.py"
        quick_gates = _load(DOCS_QUALITY.name)["jobs"].get("quick-gates")
        self.assertIsInstance(quick_gates, dict, "docs-quality quick-gates job is missing")
        matches = [
            step
            for step in quick_gates.get("steps", [])
            if isinstance(step, dict) and step.get("run") == command
        ]
        self.assertEqual(
            len(matches),
            1,
            "required docs-quality quick-gates must invoke the runner-budget guard once",
        )
        self.assertNotIn("if", matches[0], "the runner-budget guard must be unconditional")
        self.assertNotIn(
            "continue-on-error", matches[0], "the runner-budget guard must remain blocking"
        )
        self.assertNotIn("if", quick_gates, "required quick-gates must be unconditional")
        self.assertNotIn(
            "continue-on-error", quick_gates, "required quick-gates must remain blocking"
        )


if __name__ == "__main__":
    unittest.main()
