#!/usr/bin/env python3
# [OPUS-5] sparq-org/sparq#4820 — tests for the mutation-ratchet leg report.
# 🤖 SPARQ agent.
#
# Two jobs, and the second is the one that can rot:
#
#   1. BEHAVIOUR. The report's whole reason to exist is that the run-level conclusion is
#      a LOSSY aggregate: `continue-on-error` legs that FAIL cannot fail the run, so the
#      badge is decided by the other jobs and says nothing about them. So the load-bearing
#      assertion is that a job list which aggregates to `cancelled` still reports its
#      failures distinctly — TestCancelledRunStillReportsFailures replays the SHAPE of
#      the 2026-07-27 census reported in #4820 (a run carrying both failed and cancelled
#      legs, which the run badge collapses to `cancelled`) for exactly that.
#      Its own negative control is TestGreenRunIsQuiet: a clean list must exit 0 and say
#      nothing alarming, so "call everything broken" cannot satisfy the positive test.
#
#   2. DRIFT PINS. The script matches legs by a NAME PREFIX and labels a leg "at cap"
#      from the caps ci.yml actually configures. Both are copies of workflow facts, and a
#      stale copy degrades silently to "no legs matched" / "nothing was capped" — a
#      report that reads as good news. TestPinnedToLiveWorkflow re-derives both from
#      .github/workflows/ci.yml and reds when they drift. The registry pin is here for
#      the same reason: an undeclared advisory job GATES (fail-closed since #3773), and
#      this one REDs on another lane's failures, so a lost declaration would block merges
#      on nightly mutation results.
#
# Hermetic: no gh, no network. Run: python3 scripts/tests/test_mutants_leg_report.py

from __future__ import annotations

import importlib.util
import json
import re
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "nightly" / "mutants_leg_report.py"
CI_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "ci.yml"
REGISTRY = REPO_ROOT / ".github" / "advisory-registry.json"
DOCS_QUALITY = REPO_ROOT / ".github" / "workflows" / "docs-quality.yml"

_spec = importlib.util.spec_from_file_location("mutants_leg_report", SCRIPT)
mlr = importlib.util.module_from_spec(_spec)
assert _spec.loader is not None
# Register before exec: the module's dataclasses carry `X | None` annotations, which
# dataclasses resolves through sys.modules on 3.11.
sys.modules["mutants_leg_report"] = mlr
_spec.loader.exec_module(mlr)


def leg(crate: str, conclusion: str, minutes: float, shard: str = "") -> dict:
    name = mlr.LANE_NAME + f" ({crate}" + (f", {shard}" if shard else "") + ")"
    hours, mins = divmod(int(minutes), 60)
    return {
        "name": name,
        "status": "completed",
        "conclusion": conclusion,
        "started_at": "2026-07-27T00:00:00Z",
        "completed_at": f"2026-07-27T{hours:02d}:{mins:02d}:00Z",
    }


class TestCancelledRunStillReportsFailures(unittest.TestCase):
    """The headline behaviour: a run that aggregates to `cancelled` names its failures.

    Shaped after the 2026-07-27 nightly reported in #4820 — 10 failing legs
    (sparq-mpc, sparq-server, sparq-fedplan, sparq-reason shard 2/3, and six
    sparq-engine shards at the 360-minute cap) alongside cancelled and successful ones.
    Leg count and durations are illustrative, not a transcript of that run.
    """

    def setUp(self) -> None:
        jobs = [
            # Non-lane jobs of the same run must not be counted at all.
            {"name": "build + test (stable)", "status": "completed", "conclusion": "success"},
            {"name": "coverage (nightly, full incl. heavy vectors)",
             "status": "completed", "conclusion": "cancelled"},
            leg("sparq-canon", "success", 42),
            leg("sparq-parse", "success", 51),
            leg("sparq-mpc", "failure", 214),
            leg("sparq-server", "failure", 301),
            leg("sparq-fedplan", "failure", 96),
            leg("sparq-reason", "success", 119, "1/3"),
            leg("sparq-reason", "failure", 118, "2/3"),
            leg("sparq-reason", "success", 44, "3/3"),
            leg("sparq-engine", "success", 88, "1/24"),
            leg("sparq-engine", "failure", 360, "9/24"),
            leg("sparq-engine", "failure", 360, "10/24"),
            leg("sparq-engine", "failure", 360, "11/24"),
            leg("sparq-engine", "failure", 360, "12/24"),
            leg("sparq-engine", "failure", 359, "19/24"),
            leg("sparq-engine", "failure", 360, "20/24"),
            leg("sparq-geo", "cancelled", 120),
        ]
        self.report = mlr.build_report(jobs)
        self.text = mlr.render(self.report)

    def test_counts_failures_distinctly_from_cancellation(self) -> None:
        self.assertEqual(len(self.report.of("failed")), 10)
        self.assertEqual(len(self.report.of("cancelled")), 1)
        self.assertEqual(len(self.report.of("ok")), 5)
        self.assertIn("10 of 16 legs FAILED", self.text)
        self.assertIn("1 was cancelled", self.text)

    def test_non_lane_jobs_are_excluded(self) -> None:
        # The two non-lane jobs above would inflate every count if the prefix filter
        # were dropped — the `cancelled` coverage job is the trap this guards.
        self.assertEqual(len(self.report.legs), 16)
        self.assertNotIn("coverage (nightly", self.text)

    def test_verdict_is_red(self) -> None:
        self.assertEqual(mlr.verdict(self.report), 1)

    def test_names_every_failing_leg(self) -> None:
        for label in ("sparq-mpc", "sparq-server", "sparq-fedplan",
                      "sparq-reason 2/3", "sparq-engine 9/24", "sparq-engine 20/24"):
            self.assertIn(label, self.text)
        annotations = "\n".join(mlr.annotations(self.report))
        self.assertEqual(annotations.count("::error"), 10)
        self.assertEqual(annotations.count("::warning"), 1)

    def test_capped_legs_are_distinguished_from_ordinary_failures(self) -> None:
        by_label = {x.label: x for x in self.report.legs}
        self.assertTrue(by_label["sparq-engine 9/24"].capped)
        self.assertTrue(by_label["sparq-engine 19/24"].capped)  # 359 min, within tolerance
        self.assertFalse(by_label["sparq-fedplan"].capped)      # a real failure, not a cap
        # 214 min is far past the 120-minute cap and far short of the 360-minute one
        # this leg actually ran under. Flagging it capped would report a genuine test
        # failure as a sizing problem — found by reproducing the lane end-to-end.
        self.assertFalse(by_label["sparq-mpc"].capped)
        self.assertIn("ran to a `timeout-minutes` cap", self.text)

    def test_partly_good_crate_is_still_unseedable(self) -> None:
        # sparq-engine shard 1/24 SUCCEEDED; the crate is still unseedable because
        # --seed refuses an incomplete shard set. sparq-canon/parse stay seedable.
        self.assertIn("sparq-engine", self.report.unusable_crates)
        self.assertIn("sparq-reason", self.report.unusable_crates)
        self.assertNotIn("sparq-canon", self.report.unusable_crates)
        self.assertIn("Crates this run cannot seed", self.text)

    def test_reports_the_cost(self) -> None:
        self.assertAlmostEqual(self.report.runner_hours, 3352 / 60, places=2)
        self.assertIn("runner-hours across 16 legs", self.text)


class TestGreenRunIsQuiet(unittest.TestCase):
    """The negative control — "call everything broken" must not pass the suite."""

    def setUp(self) -> None:
        self.report = mlr.build_report(
            [leg("sparq-canon", "success", 42), leg("sparq-parse", "skipped", 1)]
        )

    def test_exits_zero(self) -> None:
        self.assertEqual(mlr.verdict(self.report), 0)

    def test_says_nothing_alarming(self) -> None:
        text = mlr.render(self.report)
        self.assertIn("All 2 legs produced a verdict.", text)
        self.assertNotIn("FAILED", text)
        self.assertNotIn("Crates this run cannot seed", text)
        self.assertEqual(mlr.annotations(self.report), [])


class TestFailsLoud(unittest.TestCase):
    """Detector-infrastructure failure must exit 2, never a silent 0."""

    def test_renamed_lane_is_not_silent_success(self) -> None:
        with self.assertRaises(mlr.ReportError):
            mlr.build_report([{"name": "some other job", "status": "completed",
                               "conclusion": "success"}])

    def test_cli_exits_two_on_a_renamed_lane(self, ) -> None:
        import tempfile
        with tempfile.NamedTemporaryFile("w", suffix=".jsonl", delete=False) as handle:
            handle.write(json.dumps({"name": "other", "status": "completed",
                                     "conclusion": "success"}))
            path = handle.name
        self.assertEqual(mlr.main(["--jobs-file", path, "--summary-file", ""]), 2)

    def test_unknown_conclusion_counts_as_failed(self) -> None:
        report = mlr.build_report([leg("sparq-canon", "some_new_github_conclusion", 5)])
        self.assertEqual(len(report.of("failed")), 1)

    def test_self_test_passes(self) -> None:
        self.assertEqual(mlr.main(["--self-test"]), 0)


class TestPinnedToLiveWorkflow(unittest.TestCase):
    """The copies of workflow facts this script carries must not go stale."""

    @staticmethod
    def _lane_block() -> str:
        text = CI_WORKFLOW.read_text(encoding="utf-8")
        start = text.index("\n  mutants-nightly-advisory:")
        end = text.index("\n  mutants-nightly-report:", start)
        return text[start:end]

    def test_lane_name_matches_the_live_job_name(self) -> None:
        block = self._lane_block()
        names = re.findall(r"^    name: (.+)$", block, re.MULTILINE)
        self.assertIn(mlr.LANE_NAME, names,
                      "LANE_NAME no longer matches the ci.yml job name — the report would "
                      "match zero legs and read as 'nothing to see here'")

    def test_cap_minutes_match_the_live_matrix(self) -> None:
        block = self._lane_block()
        default = re.search(r"fromJSON\(matrix\.timeout_minutes \|\| '(\d+)'\)", block)
        self.assertIsNotNone(default, "the lane's timeout-minutes default expression moved")
        caps = {int(default.group(1))}
        caps.update(int(v) for v in re.findall(r"^\s+timeout_minutes: (\d+)$",
                                               block, re.MULTILINE))
        self.assertEqual(set(mlr.CAP_MINUTES), caps,
                         "CAP_MINUTES drifted from ci.yml's matrix — capped legs would be "
                         "reported as ordinary failures, hiding a sizing problem")

    def test_report_job_is_declared_advisory(self) -> None:
        entry = json.loads(REGISTRY.read_text(encoding="utf-8"))["jobs"].get(
            "mutation ratchet leg report (advisory)")
        self.assertIsNotNone(entry, "the report job must stay declared: it REDs on another "
                                    "lane's failures, and an undeclared advisory job GATES")
        self.assertEqual(entry["workflow"], "ci.yml")
        self.assertEqual(entry["job_id"], "mutants-nightly-report")

    def test_this_suite_still_runs_in_ci(self) -> None:
        # Same idiom as test_review_lane_alarm.py: the suite pins its own call site, so
        # it cannot silently leave CI. Nothing auto-discovers scripts/tests — every one
        # is wired by hand in docs-quality.yml — and an unwired drift pin is no pin.
        self.assertIn("python3 scripts/tests/test_mutants_leg_report.py",
                      DOCS_QUALITY.read_text(encoding="utf-8"))

    def test_the_report_script_is_invoked_by_the_workflow(self) -> None:
        text = CI_WORKFLOW.read_text(encoding="utf-8")
        start = text.index("\n  mutants-nightly-report:")
        block = text[start:text.index("\n  unsafe-register:", start)]
        # The self-test must run BEFORE the live report (formal-alarm ordering: a broken
        # detector reds before it is trusted to watch anything).
        self_test = block.index("mutants_leg_report.py --self-test")
        live = block.index("mutants_leg_report.py --jobs-file")
        self.assertLess(self_test, live)
        # Sparse checkout must actually fetch the script it then runs.
        self.assertIn("sparse-checkout: scripts/nightly/mutants_leg_report.py", block)
        self.assertIn("actions: read", block)

    def test_report_job_is_not_continue_on_error(self) -> None:
        text = CI_WORKFLOW.read_text(encoding="utf-8")
        start = text.index("\n  mutants-nightly-report:")
        block = text[start:text.index("\n  unsafe-register:", start)]
        directives = [line for line in block.splitlines()
                      if not line.lstrip().startswith("#")]
        self.assertNotIn("continue-on-error", "\n".join(directives),
                         "the report must be allowed to red — that is the whole point")
        self.assertIn("if: always() && needs.nightly-gate.outputs.fresh == 'true'", block)


if __name__ == "__main__":
    unittest.main(verbosity=2)
