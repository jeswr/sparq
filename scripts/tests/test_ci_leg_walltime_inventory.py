#!/usr/bin/env python3
# [OPUS-5] Hermetic tests for the per-leg CI wall-time inventory
# (scripts/ci_leg_walltime_inventory.py, bead sq-6vshe.7). Design:
# research/ci-structural-speedup.md §8 and research/ci-leg-walltime-inventory.md.
#
# The instrument carries its own `--self-test` (the classification known-answer table
# and the decomposition arithmetic). This suite pins the two seams that a self-contained
# self-test structurally CANNOT see, both of which fail SILENTLY if broken:
#
#   1. THE CLASSIFIER-VS-REALITY SEAM. The bucket split is inferred from step NAMES, so
#      it decays the moment CI grows a step shape no rule knows — and it decays QUIETLY,
#      because an unmatched step still lands somewhere (OTHER) and the table still
#      renders. TestClassifierCoversTheLiveWorkflowTree classifies every REAL step name
#      in the live .github/workflows/ tree and bounds the unmatched share. Its own
#      negative control is the non-vacuity assertion: an invented step name must STILL
#      come back unmatched, so the bound cannot be met by a rule that matches everything.
#
#   2. THE CALL-SITE SEAM. A self-test that is not invoked is inert. TestWiredIntoCI
#      asserts this suite's own invocation exists in docs-quality.yml, so the suite
#      cannot silently leave CI.
#
# Plus the guards the report's honesty rests on: a skipped leg must never be averaged in
# as a cheap sample, coverage jobs must stay excluded (sq-piapk's topology, non-overlap),
# and every exit-2 fail-loud path must actually exit 2 rather than print a table over an
# empty population.
#
# Fully hermetic: no gh, no network, no PyYAML (the workflow tree is read with a small
# indentation state machine, so this runs on a bare interpreter as well as in CI).
#
# Run:  python3 scripts/tests/test_ci_leg_walltime_inventory.py

from __future__ import annotations

import io
import json
import re
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / "scripts"))

import ci_leg_walltime_inventory as inv  # noqa: E402

WORKFLOWS = REPO / ".github" / "workflows"
DOCS_QUALITY = WORKFLOWS / "docs-quality.yml"

# The share of DISTINCT live step names no rule matches. Measured 0.5% (1 of 196) at the
# time of writing over ci.yml + feature-matrix.yml, and that one is a coverage-job step
# the aggregator excludes anyway. The bound is deliberately loose so ordinary step churn
# does not red the gate, and tight enough that a whole new step family arriving
# unclassified does.
MAX_UNMATCHED_PCT = 12.0

_STEP_NAME = re.compile(r"^(\s+)- name: (.+)$")
_KEY = re.compile(r"^\s+(uses|run):")


def live_step_names(path: Path) -> set[str]:
    """Step `name:` values from a workflow file.

    Deliberately NOT "every `- name:` line": `strategy.matrix.include` entries use the
    same spelling, and folding matrix leg names (`bulk 1/3`, `heavy-hnsw`) into the
    step population would inflate the unmatched share with things that are not steps.
    A step is a `- name:` whose block also carries a `uses:` or `run:` key.
    """
    names: set[str] = set()
    lines = path.read_text(encoding="utf-8").splitlines()
    for idx, line in enumerate(lines):
        match = _STEP_NAME.match(line)
        if not match:
            continue
        indent, name = match.group(1), match.group(2).strip().strip("\"'")
        for follower in lines[idx + 1:]:
            if follower.strip() and not follower.startswith(indent + " "):
                break  # left this block without seeing uses:/run:
            if _STEP_NAME.match(follower) and len(follower) - len(follower.lstrip()) <= len(indent):
                break
            if _KEY.match(follower):
                names.add(name)
                break
    return names


def _step(name: str, start: str, end: str) -> dict:
    return {"name": name, "started_at": start, "completed_at": end}


def _job(name="test (load-aware shard bulk 1/3)", **over) -> dict:
    job = {
        "name": name, "workflow_name": "ci", "conclusion": "success", "run_id": 1,
        "created_at": "2026-07-01T00:00:00Z",
        "started_at": "2026-07-01T00:00:10Z",
        "completed_at": "2026-07-01T00:04:10Z",
        "steps": [
            _step("Set up job", "2026-07-01T00:00:10Z", "2026-07-01T00:00:40Z"),
            _step("Download nextest archive", "2026-07-01T00:00:40Z", "2026-07-01T00:01:10Z"),
            _step("Test (nextest, shard bulk 1/3)", "2026-07-01T00:01:10Z", "2026-07-01T00:04:10Z"),
        ],
    }
    job.update(over)
    return job


class TestClassifierCoversTheLiveWorkflowTree(unittest.TestCase):
    """Seam 1: the step-name classifier against the workflows as they actually are."""

    def setUp(self) -> None:
        self.names: set[str] = set()
        for workflow in ("ci.yml", "feature-matrix.yml"):
            self.names |= live_step_names(WORKFLOWS / workflow)

    def test_population_is_not_empty(self) -> None:
        # Guard against the extractor silently returning nothing, which would make
        # every assertion below vacuously true.
        self.assertGreater(len(self.names), 100, "step-name extractor found ~nothing")

    def test_unmatched_share_is_bounded(self) -> None:
        unmatched = sorted(n for n in self.names if inv.classify_step(n)[0] == "unmatched")
        pct = 100.0 * len(unmatched) / len(self.names)
        self.assertLessEqual(
            pct, MAX_UNMATCHED_PCT,
            f"{len(unmatched)}/{len(self.names)} live step names ({pct:.1f}%) match no rule "
            f"in ci_leg_walltime_inventory._RULES, so the wall-time split would book them "
            f"as OTHER. Extend _RULES (or raise MAX_UNMATCHED_PCT deliberately). "
            f"Unmatched: {unmatched[:20]}",
        )

    def test_the_bound_is_not_met_by_a_catch_all(self) -> None:
        # NON-VACUITY CONTROL. If someone "fixes" a red bound by adding a rule that
        # matches everything, this reds instead.
        rule, bucket = inv.classify_step("Zorblat the quux")
        self.assertEqual((rule, bucket), ("unmatched", inv.OTHER))

    def test_the_real_load_bearing_steps_land_in_the_right_bucket(self) -> None:
        # The steps the whole inventory is FOR: the once-paid archive build, the shard
        # run that reuses it, and the doctest lane sq-6vshe.7(d) audits.
        self.assertEqual(
            inv.classify_step("Build + archive test binaries (nextest archive)")[1],
            inv.COMPILE)
        self.assertEqual(
            inv.classify_step("Test (nextest, shard bulk 1/3)")[1], inv.TEST)
        self.assertEqual(
            inv.classify_step(
                "Doctests (nextest does not run these; run once, off the warm build)")[1],
            inv.DOCTEST)
        # …and the trap: installing the runner is setup, not test time.
        self.assertEqual(inv.classify_step("Install cargo-nextest")[1], inv.SETUP)

    def test_doctest_bucket_is_reachable_from_the_live_tree(self) -> None:
        # sq-6vshe.7(d) needs the doctest bill as a LINE ITEM. If ci.yml's doctest step
        # is renamed out of the classifier's reach the audit goes silently to zero.
        doctest_steps = [n for n in self.names if inv.classify_step(n)[1] == inv.DOCTEST]
        self.assertTrue(doctest_steps, "no live step classifies as DOCTEST")


class TestCoverageNonOverlap(unittest.TestCase):
    """sq-piapk owns the coverage-instrumented topology; this bead must not report it."""

    def test_coverage_jobs_excluded_by_default(self) -> None:
        stats = inv.aggregate([_job(), _job(name="coverage (measure shard 1/4)")])
        self.assertEqual(stats["leg_count"], 1)
        self.assertEqual(stats["coverage_jobs_excluded"], 1)

    def test_coverage_jobs_included_only_on_opt_in(self) -> None:
        stats = inv.aggregate([_job(), _job(name="coverage (measure shard 1/4)")],
                              include_coverage=True)
        self.assertEqual(stats["leg_count"], 2)

    def test_non_coverage_legs_are_not_swept_up(self) -> None:
        for name in ("test (load-aware shard bulk 1/3)", "shacl-conformance",
                     "wasm build (sparq-wasm)", "lint"):
            self.assertFalse(inv.is_coverage_job(name), name)


class TestSkippedLegsAreNotCheapSamples(unittest.TestCase):
    """A skipped leg costs nothing; averaging it in would understate every leg."""

    def test_skipped_job_is_dropped(self) -> None:
        self.assertIsNone(inv.decompose_job(_job(conclusion="skipped")))

    def test_never_started_job_is_dropped(self) -> None:
        self.assertIsNone(inv.decompose_job(_job(started_at=None)))

    def test_zero_wall_job_is_dropped(self) -> None:
        self.assertIsNone(inv.decompose_job(
            _job(completed_at="2026-07-01T00:00:10Z")))

    def test_a_real_job_survives(self) -> None:
        # Control: the drop rules must not eat everything.
        self.assertIsNotNone(inv.decompose_job(_job()))


class TestUnaccountedTimeIsReportedNotRedistributed(unittest.TestCase):
    """Job wall-clock minus its steps is runner overhead. It must stay visible."""

    def test_gap_lands_in_unaccounted(self) -> None:
        row = inv.decompose_job(_job(completed_at="2026-07-01T00:05:10Z"))  # +60s gap
        self.assertAlmostEqual(row["unaccounted"], 60.0, places=2)
        self.assertAlmostEqual(row["wall"], 300.0, places=2)

    def test_buckets_plus_unaccounted_equal_wall(self) -> None:
        row = inv.decompose_job(_job())
        total = sum(row[b] for b in inv.BUCKETS) + row["unaccounted"]
        self.assertAlmostEqual(total, row["wall"], places=2)

    def test_queue_is_separate_from_wall(self) -> None:
        # queue is created_at -> started_at and is NOT part of the job's wall-clock.
        row = inv.decompose_job(_job())
        self.assertAlmostEqual(row[inv.QUEUE], 10.0, places=2)


class TestFailLoud(unittest.TestCase):
    """Reporting a table over an empty population is the one forbidden outcome."""

    def _main(self, argv: list[str]) -> int:
        buf = io.StringIO()
        with redirect_stdout(buf):
            return inv.main(argv)

    def test_empty_dump_exits_2(self) -> None:
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as handle:
            handle.write("[]")
        self.assertEqual(self._main(["--from-json", handle.name]), 2)

    def test_missing_dump_exits_2(self) -> None:
        self.assertEqual(self._main(["--from-json", "/nonexistent/jobs.json"]), 2)

    def test_non_list_dump_exits_2(self) -> None:
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as handle:
            handle.write('{"jobs": []}')
        self.assertEqual(self._main(["--from-json", handle.name]), 2)

    def test_all_jobs_excluded_exits_2(self) -> None:
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as handle:
            json.dump([_job(name="coverage (measure shard 1/4)")], handle)
        self.assertEqual(self._main(["--from-json", handle.name]), 2)

    def test_a_real_dump_exits_0(self) -> None:
        # Control: exit 2 must mean something.
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as handle:
            json.dump([_job()], handle)
        self.assertEqual(self._main(["--from-json", handle.name]), 0)


class TestReportStatesItsOwnUncertainty(unittest.TestCase):
    """A split whose classification has decayed must say so on the face of the table."""

    def test_unclassified_share_is_always_printed(self) -> None:
        md = inv.render_markdown(inv.aggregate([_job()]))
        self.assertIn("Unclassified share", md)

    def test_unmatched_step_names_are_named(self) -> None:
        job = _job()
        job["steps"].append(_step("Zorblat the quux",
                                  "2026-07-01T00:04:10Z", "2026-07-01T00:04:40Z"))
        job["completed_at"] = "2026-07-01T00:04:40Z"
        md = inv.render_markdown(inv.aggregate([job]))
        self.assertIn("Zorblat the quux", md)

    def test_recognised_gate_work_is_other_workload_not_classifier_decay(self) -> None:
        # OTHER is a WORKLOAD bucket: `script-gate` deliberately books recognised
        # guards/ratchets/reports there. A leg that spends real time in them is fully
        # classified, so the decay metric must stay at zero while OTHER grows. If
        # unclassified_pct is ever computed from the whole OTHER bucket again, this reds.
        job = _job()
        job["steps"].append(_step("Enforce ratchet (pass count must not regress)",
                                  "2026-07-01T00:04:10Z", "2026-07-01T00:09:10Z"))
        job["completed_at"] = "2026-07-01T00:09:10Z"
        stats = inv.aggregate([job])
        self.assertGreater(stats["other_pct"], 45.0)
        self.assertEqual(stats["unclassified_pct"], 0.0)
        self.assertEqual(stats["unmatched_steps"], [])

    def test_max_unclassified_pct_can_red_on_unmatched_steps(self) -> None:
        job = _job()
        job["steps"].append(_step("Zorblat the quux",
                                  "2026-07-01T00:04:10Z", "2026-07-01T00:09:10Z"))
        job["completed_at"] = "2026-07-01T00:09:10Z"
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as handle:
            json.dump([job], handle)
        buf = io.StringIO()
        with redirect_stdout(buf):
            self.assertEqual(inv.main(["--from-json", handle.name,
                                       "--max-unclassified-pct", "1"]), 1)
            # Control: a generous bound passes, so the exit code tracks the threshold.
            self.assertEqual(inv.main(["--from-json", handle.name,
                                       "--max-unclassified-pct", "99"]), 0)

    def test_the_threshold_ignores_recognised_gate_time(self) -> None:
        # The other half of the same seam, at the CLI: a leg dominated by a RECOGNISED
        # gate step must not trip the classifier-decay threshold at all.
        job = _job()
        job["steps"].append(_step("Enforce ratchet (pass count must not regress)",
                                  "2026-07-01T00:04:10Z", "2026-07-01T00:09:10Z"))
        job["completed_at"] = "2026-07-01T00:09:10Z"
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as handle:
            json.dump([job], handle)
        buf = io.StringIO()
        with redirect_stdout(buf):
            self.assertEqual(inv.main(["--from-json", handle.name,
                                       "--max-unclassified-pct", "0"]), 0)


class TestInScriptSelfTestPasses(unittest.TestCase):
    def test_self_test_exits_zero(self) -> None:
        result = subprocess.run(
            [sys.executable, str(REPO / "scripts" / "ci_leg_walltime_inventory.py"),
             "--self-test"],
            capture_output=True, text=True, timeout=120,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


class TestWiredIntoCI(unittest.TestCase):
    """Seam 2: a self-test nobody invokes is inert."""

    def test_this_suite_is_invoked_by_docs_quality(self) -> None:
        text = DOCS_QUALITY.read_text(encoding="utf-8")
        self.assertIn("python3 scripts/tests/test_ci_leg_walltime_inventory.py", text)

    def test_the_invocation_is_not_disarmed(self) -> None:
        # A step that carries `continue-on-error: true` is green whatever it finds.
        lines = DOCS_QUALITY.read_text(encoding="utf-8").splitlines()
        for idx, line in enumerate(lines):
            if "test_ci_leg_walltime_inventory.py" in line:
                window = "\n".join(lines[max(0, idx - 4):idx + 4])
                self.assertNotIn("continue-on-error", window)
                return
        self.fail("invocation not found")


if __name__ == "__main__":
    unittest.main(verbosity=2)
