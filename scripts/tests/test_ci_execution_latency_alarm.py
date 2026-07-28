#!/usr/bin/env python3
"""Suite for scripts/ci_execution_latency_alarm.py. 🤖 SPARQ agent. Bead sq-1lc4i.

Three halves, because three different things can go wrong:

  1. THE CLASSIFIERS — every guard has a named test that reds when the guard is deleted or
     inverted. Includes the two guards that are easiest to get wrong in the SILENT
     direction: the missing-baseline fallback (a lane whose runs never complete must still
     be watchable) and the empty-scan-set fail-loud (a detector watching nothing must not
     report health).

  2. THE #4368 DISJOINTNESS — M1's scope is the exact set complement of
     `cron_lane_liveness.py`'s. If either predicate drifts the two detectors could both
     raise on one lane (duplicate issues) or neither could (a hole). Both directions are
     pinned, over the REAL workflow tree rather than fixtures.

  3. THE YAML SEAM — measured across this repo, every mutant that survived a Python-only
     battery lived in the workflow. So the step, its `if:`, the call site, the ordering,
     the permissions and the action pins are all asserted by EXACT match, never substring:
     `--apply-DROPPED` and `--self-test-DISABLED` both survive a containment check.
"""

from __future__ import annotations

import datetime as dt
import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "ci_execution_latency_alarm.py"
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
ALARM_WF = WORKFLOWS / "ci-latency-alarm.yml"
DOCS_QUALITY = WORKFLOWS / "docs-quality.yml"
REGISTRY_PATH = REPO_ROOT / ".github" / "advisory-registry.json"

_spec = importlib.util.spec_from_file_location("ci_execution_latency_alarm", SCRIPT)
alarm = importlib.util.module_from_spec(_spec)
sys.modules["ci_execution_latency_alarm"] = alarm
_spec.loader.exec_module(alarm)

NOW = dt.datetime(2026, 7, 28, 12, 0, tzinfo=dt.timezone.utc)


def _lane(**kw):
    return alarm._lane(now=NOW, **kw)


def _run_obj(**kw):
    return alarm._run_obj(now=NOW, **kw)


def _job_name(workflow_file: Path) -> str:
    jobs = yaml.safe_load(workflow_file.read_text())["jobs"]
    assert len(jobs) == 1, f"{workflow_file}: expected exactly one job, got {list(jobs)}"
    return str(next(iter(jobs.values()))["name"])


# ---------------------------------------------------------------------------------
# 1. classifiers
# ---------------------------------------------------------------------------------
class TestCronExpansion(unittest.TestCase):
    """Canary-validated against hand-computed answers. An expander that silently returns
    0 would make every lane look like a 100% deficit; one that over-counts would make
    every lane look healthy. Both directions are pinned."""

    def test_known_firing_counts(self):
        cases = [
            ("*/10 * * * *", 6, 36),
            ("*/10 * * * *", 24, 144),
            ("17 3 * * *", 24, 1),
            ("4,14,24,34,44,54 * * * *", 6, 36),
            ("2,7,12,17,22,27,32,37,42,47,52,57 * * * *", 6, 72),
            ("*/30 * * * *", 24, 48),
            ("37 1,7,13,19 * * *", 24, 4),
            ("0 */4 * * *", 24, 6),
        ]
        for expr, hours, want in cases:
            with self.subTest(expr=expr):
                got = alarm.expected_firings(expr, NOW - dt.timedelta(hours=hours), NOW)
                self.assertEqual(got, want)

    def test_weekly_cron_is_absent_from_a_window_that_does_not_contain_it(self):
        # 2026-07-28 is a Tuesday; the Monday fire is >24h back but <48h back.
        self.assertEqual(
            alarm.expected_firings("0 6 * * 1", NOW - dt.timedelta(hours=24), NOW), 0)
        self.assertEqual(
            alarm.expected_firings("0 6 * * 1", NOW - dt.timedelta(hours=48), NOW), 1)

    def test_dom_and_dow_both_restricted_is_a_UNION_not_an_intersection(self):
        # POSIX: when both day fields are restricted the match is their OR. Reading it as
        # AND would UNDER-count expected firings and hide a real deficit.
        n = alarm.expected_firings(
            "0 0 1 * 3",
            dt.datetime(2026, 7, 1, tzinfo=dt.timezone.utc),
            dt.datetime(2026, 7, 31, tzinfo=dt.timezone.utc))
        self.assertGreater(n, 1, "dom+dow must be a union (all Wednesdays plus the 1st)")

    def test_malformed_cron_raises_rather_than_returning_zero(self):
        for bad in ("", "* * * *", "* * * * * *", "60 * * * *", "*/0 * * * *",
                    "5-1 * * * *", "x * * * *"):
            with self.subTest(bad=bad):
                with self.assertRaises(alarm.CronError):
                    alarm.expected_firings(bad, NOW - dt.timedelta(hours=6), NOW)


class TestM1CronFiringDeficit(unittest.TestCase):
    CAP = alarm.capped_expectation()

    def test_a_delivering_lane_does_not_raise(self):
        f, c = alarm.find_cron_deficits([_lane(fires=self.CAP)], NOW)
        self.assertEqual(f, [])
        self.assertEqual(c.get("delivering"), 1)

    def test_a_lane_that_fired_far_below_expectation_raises(self):
        f, c = alarm.find_cron_deficits([_lane(fires=1)], NOW)
        self.assertEqual(len(f), 1)
        self.assertEqual((f[0]["expected"], f[0]["actual"]), (self.CAP, 1))
        self.assertEqual(c.get("firing-deficit"), 1)

    def test_a_cron_that_never_fired_at_all_raises(self):
        # THE MODE WITH NO ARTIFACT. There is no run to inspect, so this can only be
        # caught by comparing against a computed expectation.
        f, _ = alarm.find_cron_deficits([_lane(fires=0)], NOW)
        self.assertEqual(len(f), 1)
        self.assertEqual(f[0]["actual"], 0)

    def test_the_expectation_is_CAPPED_at_what_github_really_delivers(self):
        """The load-bearing anti-cry-wolf guard. MEASURED: sparq's `*/10` lanes have a
        nominal expectation of 144/day and GitHub delivers ~12. Comparing against nominal
        would put six lanes permanently in breach, and a permanently-red alarm is a muted
        alarm. Delete the cap and this test reds."""
        f, _ = alarm.find_cron_deficits(
            [_lane(crons=("*/10 * * * *",), fires=self.CAP)], NOW)
        self.assertEqual(f, [], "a sub-hourly lane at GitHub's real ceiling is healthy")
        nominal = alarm.expected_firings(
            "*/10 * * * *", NOW - dt.timedelta(hours=alarm.CRON_WINDOW_HOURS), NOW)
        self.assertGreater(nominal, self.CAP,
                           "fixture is vacuous unless nominal exceeds the cap")

    def test_a_firing_inside_the_grace_window_is_not_counted_as_missing(self):
        """Without the grace period the tick that lands on a lane's own cron minute sees
        an expectation whose run does not exist yet, and manufactures a deficit daily."""
        lane = _lane(fires=0)
        lane["schedule_run_times"] = [NOW - dt.timedelta(minutes=1)] * self.CAP
        f, _ = alarm.find_cron_deficits([lane], NOW)
        self.assertEqual(len(f), 1, "runs newer than the grace cut-off are not yet due")

    def test_the_floor_boundary_is_a_strict_less_than(self):
        import math
        at_floor = math.ceil(self.CAP * alarm.CRON_DELIVERY_FLOOR)
        self.assertEqual(alarm.find_cron_deficits([_lane(fires=at_floor)], NOW)[0], [])
        self.assertEqual(
            len(alarm.find_cron_deficits([_lane(fires=at_floor - 1)], NOW)[0]), 1)

    def test_an_over_delivering_lane_is_healthy(self):
        f, _ = alarm.find_cron_deficits([_lane(fires=self.CAP * 3)], NOW)
        self.assertEqual(f, [])

    def test_a_lane_below_the_expectation_floor_is_quiet_and_counted(self):
        # A weekly lane has a capped expectation of 0 inside a 24h window.
        f, c = alarm.find_cron_deficits([_lane(crons=("0 6 * * 1",), fires=0)], NOW)
        self.assertEqual(f, [])
        self.assertEqual(c.get("expectation-below-floor"), 1)

    def test_a_disabled_lane_is_quiet_and_counted(self):
        f, c = alarm.find_cron_deficits([_lane(state="disabled_manually", fires=0)], NOW)
        self.assertEqual(f, [])
        self.assertEqual(c.get("disabled"), 1)

    def test_an_unparseable_cron_is_quiet_and_VISIBLE_in_the_census(self):
        f, c = alarm.find_cron_deficits([_lane(crons=("nonsense",), fires=0)], NOW)
        self.assertEqual(f, [])
        self.assertEqual(c.get("cron-unparseable"), 1,
                         "a fail-safe skip must be counted, or it is a silent skip")


class TestM2QueueWait(unittest.TestCase):
    def test_a_run_queued_past_the_threshold_raises(self):
        late = _run_obj(status="queued",
                        age_min=int(alarm.QUEUE_MAX_WAIT_SECONDS / 60) + 5)
        f, c = alarm.find_queue_overruns([late], NOW)
        self.assertEqual(len(f), 1)
        self.assertEqual(c.get("queued-over-threshold"), 1)

    def test_a_run_queued_within_the_threshold_does_not_raise(self):
        f, c = alarm.find_queue_overruns([_run_obj(status="queued", age_min=1)], NOW)
        self.assertEqual(f, [])
        self.assertEqual(c.get("queued-within-threshold"), 1)

    def test_m2_ignores_in_progress_runs(self):
        f, _ = alarm.find_queue_overruns(
            [_run_obj(status="in_progress", age_min=10_000)], NOW)
        self.assertEqual(f, [], "an in_progress run is M3's population, not M2's")

    def test_an_hour_long_queue_wait_raises_under_the_SHIPPED_threshold(self):
        """ABSOLUTE, not relative to the constant. The sibling tests above derive their
        fixture age FROM QUEUE_MAX_WAIT_SECONDS, so they scale with it and cannot fail
        when it changes — a mutant that multiplied the constant by 10**6 SURVIVED them.
        MEASURED known positive: jeswr/agent-account-registry held `pr-gate` runs in
        `queued` for 12.8-99.5 minutes between 00:58Z and 02:24Z on 2026-07-28."""
        f, _ = alarm.find_queue_overruns([_run_obj(status="queued", age_min=60)], NOW)
        self.assertEqual(len(f), 1, "an hour in `queued` must alarm")

    def test_the_queue_threshold_stays_inside_its_validated_band(self):
        """The band the false-positive measurement was made over. Outside it the stated
        0.022% / 0.153% figures no longer describe the shipped behaviour."""
        self.assertGreaterEqual(alarm.QUEUE_MAX_WAIT_SECONDS, 5 * 60)
        self.assertLessEqual(alarm.QUEUE_MAX_WAIT_SECONDS, 30 * 60)


class TestM3ExecutionOverrun(unittest.TestCase):
    BASE = {(".github/workflows/a.yml", "schedule"): {"p90": 3600.0, "n": 50}}

    def test_a_run_inside_its_band_does_not_raise(self):
        f, c = alarm.find_execution_overruns([_run_obj(age_min=30)], self.BASE, NOW)
        self.assertEqual(f, [])
        self.assertEqual(c.get("in-progress-within-threshold"), 1)

    def test_a_run_past_its_band_raises(self):
        f, c = alarm.find_execution_overruns([_run_obj(age_min=8 * 60)], self.BASE, NOW)
        self.assertEqual(len(f), 1)
        self.assertEqual(c.get("execution-overrun"), 1)

    def test_m3_ignores_queued_runs(self):
        f, _ = alarm.find_execution_overruns(
            [_run_obj(status="queued", age_min=10_000)], self.BASE, NOW)
        self.assertEqual(f, [], "a queued run is M2's population, not M3's")

    def test_a_lane_with_NO_baseline_is_still_watched(self):
        """THE FAIL-OPEN HOLE. A lane whose runs never complete has no completed history,
        so it has no derived baseline. If a missing baseline caused a `continue`, the
        detector would go silent exactly when a lane is 100% hung — the alarm would be
        blind to the worst case it exists for. Ask the 100% question: if every run of a
        lane hung forever, would this still fire? It must."""
        f, _ = alarm.find_execution_overruns([_run_obj(age_min=7 * 60)], {}, NOW)
        self.assertEqual(len(f), 1)
        self.assertIn("floor", f[0]["basis"])

    def test_an_undersampled_baseline_is_not_trusted_even_when_it_is_LARGE(self):
        """The under-sampling guard only BITES when the thin baseline would WIDEN the
        threshold past the floor. A fixture with a small p90 collapses to the floor
        whether or not the guard exists, and cannot detect its removal — the earlier
        version of this test used p90=60s and both BASELINE_MIN_N mutants SURVIVED it."""
        thin_but_huge = {(".github/workflows/a.yml", "schedule"):
                         {"p90": 20 * 3600.0, "n": 1}}
        f, _ = alarm.find_execution_overruns([_run_obj(age_min=7 * 60)], thin_but_huge,
                                             NOW)
        self.assertEqual(len(f), 1,
                         "a 1-sample 20h p90 must not widen the threshold to 40h")
        self.assertEqual(f[0]["threshold_seconds"], int(alarm.EXEC_FLOOR_SECONDS))
        # Control: the SAME p90 with a real sample size legitimately widens the band.
        thick = {(".github/workflows/a.yml", "schedule"):
                 {"p90": 20 * 3600.0, "n": alarm.BASELINE_MIN_N}}
        self.assertEqual(
            alarm.find_execution_overruns([_run_obj(age_min=7 * 60)], thick, NOW)[0], [])

    def test_a_baseline_WIDER_than_the_floor_raises_the_threshold(self):
        """Quantifier direction: the floor is a MINIMUM, not the answer. A legitimately
        long lane (nightly ci.yml runs ~8.3h) must not alarm merely for exceeding 6h."""
        big = {(".github/workflows/a.yml", "schedule"): {"p90": 8 * 3600.0, "n": 40}}
        f, _ = alarm.find_execution_overruns([_run_obj(age_min=7 * 60)], big, NOW)
        self.assertEqual(f, [], "an 8.3h-p90 lane at 7h is healthy — do not cry wolf")

    def test_the_measured_2026_07_27_outlier_shape_raises(self):
        """KNOWN POSITIVE, from real data: nightly ci.yml ran 17.33h on 2026-07-27 against
        an 8.35h p90 over the preceding 13 nights (2.08x)."""
        big = {(".github/workflows/a.yml", "schedule"): {"p90": 8.35 * 3600.0, "n": 13}}
        f, _ = alarm.find_execution_overruns(
            [_run_obj(age_min=int(17.33 * 60))], big, NOW)
        self.assertEqual(len(f), 1)

    def test_the_detection_variable_is_live_age_not_completed_history(self):
        """A hang is invisible to any statistic over COMPLETED runs — a job that never
        finishes never appears in one. This pins that M3 reads the LIVE run's age: a run
        with a huge live age raises even when the completed baseline is enormous, and the
        baseline alone can never produce a finding without a live run."""
        big = {(".github/workflows/a.yml", "schedule"): {"p90": 8 * 3600.0, "n": 40}}
        f, c = alarm.find_execution_overruns([], big, NOW)
        self.assertEqual((f, c), ([], {}), "no live run => nothing to say")
        f, _ = alarm.find_execution_overruns([_run_obj(age_min=40 * 60)], big, NOW)
        self.assertEqual(len(f), 1)


class TestCensusAndExitCodes(unittest.TestCase):
    def test_a_clean_run_still_emits_a_census_for_every_mode(self):
        """A silent alarm is indistinguishable from a healthy system."""
        _, c1 = alarm.find_cron_deficits([_lane(fires=36)], NOW)
        _, c2 = alarm.find_queue_overruns([_run_obj(status="queued", age_min=1)], NOW)
        _, c3 = alarm.find_execution_overruns(
            [_run_obj(age_min=1)], TestM3ExecutionOverrun.BASE, NOW)
        for name, census in (("M1", c1), ("M2", c2), ("M3", c3)):
            with self.subTest(mode=name):
                self.assertTrue(census, f"{name} emitted no census on the all-clear")

    def test_the_three_exit_codes_are_distinct(self):
        # 0 = clean, 1 = a choke, 2 = the detector is broken. Collapsing any pair makes a
        # broken detector indistinguishable from a healthy repo.
        clean = self._run(lanes=[_lane(fires=36)], live=[])
        alarmed = self._run(lanes=[_lane(fires=0)], live=[])
        broken = alarm.main(["--repo", "not-a-slug", "--dry-run"])
        self.assertEqual((clean, alarmed, broken), (0, 1, 2))

    def test_an_empty_scan_set_is_fail_LOUD(self):
        """100% question, applied to M1: if every scheduled workflow were cron-only, M1's
        population would be empty. Reporting 'clean' over an empty population is how a
        detector reports health while watching nothing."""
        self.assertEqual(self._run(lanes=[], live=[]), 2)
        only_cron_only = [_lane(cron_only=True, in_scope=False)]
        self.assertEqual(self._run(lanes=only_cron_only, live=[]), 2)

    def test_the_two_empty_scan_set_guards_report_DISTINCT_causes(self):
        """They are not redundant: 'no workflow files at all' means the sparse-checkout
        dropped `.github/workflows` (a deploy defect), while 'nothing in scope' means the
        predicate or the repo changed shape. Collapsing them loses the more actionable
        diagnosis — and without this assertion, deleting the first guard SURVIVES, because
        the second one also exits 2 on an empty list."""
        import contextlib
        import io
        for lanes, expect in (([], "no workflows discovered"),
                              ([_lane(cron_only=True, in_scope=False)], "empty M1 scan set")):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                rc = self._run(lanes=lanes, live=[])
            self.assertEqual(rc, 2)
            self.assertIn(expect, buf.getvalue())

    @staticmethod
    def _run(lanes, live, baselines=None):
        state = {
            "repo": "o/r",
            "lanes": [dict(x, schedule_run_times=[t.strftime("%Y-%m-%dT%H:%M:%SZ")
                                                 for t in x["schedule_run_times"]])
                      for x in lanes],
            "live_runs": live,
            "baselines": baselines or {},
        }
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as fh:
            json.dump(state, fh)
            path = fh.name
        return alarm.main(["--state-file", path, "--now", "2026-07-28T12:00:00Z",
                           "--dry-run"])


class TestIssueBody(unittest.TestCase):
    def test_the_body_carries_the_dedupe_marker_and_self_identifies(self):
        body = alarm.render_issue_body(
            "o/r",
            [{"mode": "M2-queue-wait", "workflow": "a.yml", "run_id": 1, "event": "push",
              "waited_seconds": 99, "threshold_seconds": 60, "head_branch": "main"}],
            {"M2-queue-wait": {"queued-over-threshold": 1}}, NOW)
        self.assertTrue(body.rstrip().endswith(
            f"<!-- {alarm.KEY_PREFIX}: o/r -->"))
        self.assertTrue(body.startswith("> 🤖"))
        self.assertIn("Census of every state exit", body)


# ---------------------------------------------------------------------------------
# 2. disjointness with #4368
# ---------------------------------------------------------------------------------
class TestScopeIsTheComplementOfCronLaneLiveness(unittest.TestCase):
    """M1's scope must be the exact set complement of `cron_lane_liveness.py`'s, so the
    two detectors partition the scheduled workflows: never both on one lane (duplicate
    issues), never neither (a hole)."""

    @staticmethod
    def _scheduled_workflows():
        out = {}
        for path in sorted(WORKFLOWS.glob("*.yml")):
            on = alarm.workflow_triggers(path.read_text())
            if "schedule" in on:
                out[path.name] = on
        return out

    def test_no_workflow_is_in_both_scopes(self):
        for name, on in self._scheduled_workflows().items():
            in_scope, _, cron_only = alarm.m1_scope(on)
            with self.subTest(workflow=name):
                self.assertFalse(in_scope and cron_only,
                                 f"{name} would be claimed by BOTH detectors")

    def test_every_scheduled_workflow_is_in_exactly_one_scope(self):
        for name, on in self._scheduled_workflows().items():
            in_scope, crons, cron_only = alarm.m1_scope(on)
            if not crons:
                continue  # a `schedule:` with no cron expression is neither's problem
            with self.subTest(workflow=name):
                self.assertTrue(in_scope or cron_only,
                                f"{name} would be claimed by NEITHER detector")

    def test_the_m1_population_over_the_real_tree_is_not_empty(self):
        """Anti-vacuity: if the predicate broke such that nothing was ever in scope, every
        assertion above would still pass. The measured shape is 13 in-scope lanes."""
        in_scope = [n for n, on in self._scheduled_workflows().items()
                    if alarm.m1_scope(on)[0]]
        self.assertGreaterEqual(len(in_scope), 5,
                                f"M1 scope collapsed to {in_scope}")

    def test_ci_yml_is_in_m1_scope(self):
        """ci.yml is the repo's largest capacity consumer AND carries a `schedule:`, but
        it has push/pull_request/merge_group triggers so #4368 structurally cannot see it.
        If this ever stops being true, the coordination note in both headers is wrong."""
        on = alarm.workflow_triggers((WORKFLOWS / "ci.yml").read_text())
        self.assertTrue(alarm.m1_scope(on)[0])
        self.assertFalse(alarm.m1_scope(on)[2])

    def test_a_cron_only_lane_from_the_real_tree_is_excluded(self):
        on = alarm.workflow_triggers((WORKFLOWS / "formal-alarm.yml").read_text())
        self.assertFalse(alarm.m1_scope(on)[0])
        self.assertTrue(alarm.m1_scope(on)[2])


# ---------------------------------------------------------------------------------
# 3. the YAML seam
# ---------------------------------------------------------------------------------
class TestWorkflowSeam(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.wf = yaml.safe_load(ALARM_WF.read_text())
        cls.text = ALARM_WF.read_text()
        cls.job = next(iter(cls.wf["jobs"].values()))
        cls.steps = cls.job["steps"]

    def test_it_is_triggered_by_schedule(self):
        on = self.wf.get(True, self.wf.get("on"))
        self.assertIn("schedule", on)
        self.assertTrue([s["cron"] for s in on["schedule"]])

    def test_the_selftest_step_runs_BEFORE_the_detector(self):
        """A detector is never trusted to watch anything before it has proved itself on
        this tick's code. Ordering, not mere presence."""
        idx = {}
        for i, step in enumerate(self.steps):
            run = str(step.get("run", ""))
            if "--self-test" in run:
                idx["selftest"] = i
            elif "ci_execution_latency_alarm.py" in run:
                idx.setdefault("detector", i)
        self.assertIn("selftest", idx)
        self.assertIn("detector", idx)
        self.assertLess(idx["selftest"], idx["detector"])

    def test_the_selftest_invocation_is_EXACTLY_the_expected_command(self):
        """EXACT match, not substring: `--self-test-DISABLED` contains `--self-test` and
        would survive a containment check while doing nothing."""
        runs = [str(s.get("run", "")).strip() for s in self.steps]
        self.assertIn("python3 scripts/ci_execution_latency_alarm.py --self-test", runs)

    def test_the_detector_invocation_is_EXACTLY_the_expected_command(self):
        runs = [str(s.get("run", "")).strip() for s in self.steps]
        self.assertIn("python3 scripts/ci_execution_latency_alarm.py", runs)

    def test_no_step_is_conditional_or_continue_on_error(self):
        """A step `if:` with no status function defaults to `success()`, so a failed
        earlier step silently skips it — and `continue-on-error` turns a fail-loud
        detector fail-open."""
        for step in self.steps:
            with self.subTest(step=step.get("name")):
                self.assertNotIn("if", step)
                self.assertNotIn("continue-on-error", step)
        self.assertNotIn("continue-on-error", self.job)

    def test_the_job_holds_no_arming_authority(self):
        """Structural, not procedural: the detector must be unable to label, promote or
        push even if its logic were subverted."""
        perms = self.job["permissions"]
        self.assertEqual(perms.get("contents"), "read")
        self.assertEqual(perms.get("issues"), "write")
        self.assertEqual(perms.get("actions"), "read")
        for forbidden in ("pull-requests", "checks", "id-token", "packages"):
            self.assertNotIn(forbidden, perms)
        self.assertEqual(self.wf.get("permissions"), {"contents": "read"})

    def test_every_action_use_is_sha_pinned(self):
        import re
        for step in self.steps:
            uses = step.get("uses")
            if uses:
                with self.subTest(uses=uses):
                    self.assertRegex(uses, r"@[0-9a-f]{40}$")

    def test_the_checkout_does_not_persist_credentials(self):
        for step in self.steps:
            if str(step.get("uses", "")).startswith("actions/checkout@"):
                self.assertIs(step["with"]["persist-credentials"], False)

    def test_the_checkout_fetches_the_workflows_dir_M1_reads(self):
        """M1 derives its expectation from the workflow files themselves. A sparse
        checkout that omits them would make every lane read as `not-scheduled` — a silent
        total blind spot rather than an error."""
        for step in self.steps:
            if str(step.get("uses", "")).startswith("actions/checkout@"):
                sparse = str(step["with"].get("sparse-checkout", ""))
                self.assertIn(".github/workflows", sparse)
                self.assertIn("scripts/ci_execution_latency_alarm.py", sparse)


class TestSuiteIsWiredIntoCI(unittest.TestCase):
    """Anti-vacuity anchor: without this the whole file could leave CI unnoticed."""

    def test_docs_quality_runs_this_suite(self):
        runs = []
        for job in yaml.safe_load(DOCS_QUALITY.read_text())["jobs"].values():
            for step in job.get("steps") or []:
                runs.append(str(step.get("run", "")).strip())
        self.assertIn(
            "python3 scripts/tests/test_ci_execution_latency_alarm.py", runs,
            "docs-quality.yml no longer invokes this suite — it would stop running")

    def test_that_call_site_is_unconditional(self):
        for job in yaml.safe_load(DOCS_QUALITY.read_text())["jobs"].values():
            for step in job.get("steps") or []:
                if "test_ci_execution_latency_alarm.py" in str(step.get("run", "")):
                    self.assertNotIn("if", step)
                    self.assertNotIn("continue-on-error", step)


class TestDeclaredNonGating(unittest.TestCase):
    """This lane REDs on a finding. That is only safe because it is DECLARED advisory."""

    def test_the_lane_is_declared_in_the_advisory_registry(self):
        registry = json.loads(REGISTRY_PATH.read_text())["jobs"]
        name = _job_name(ALARM_WF)
        self.assertIn(name, registry,
                      "an undeclared alarm lane GATES every merge on repo-wide CI health")
        entry = registry[name]
        for field in ("owner_bead", "promotion_criteria", "registered", "workflow",
                      "job_id"):
            self.assertIn(field, entry)
        self.assertEqual(entry["workflow"], ALARM_WF.name)

    def test_the_real_gate_treats_this_lane_as_advisory(self):
        sys.path.insert(0, str(REPO_ROOT / "scripts"))
        import ci_summary_gate as gate  # noqa: PLC0415
        gate.load_advisory_registry(str(REGISTRY_PATH))
        self.assertTrue(gate.is_advisory(_job_name(ALARM_WF)))
        # Anti-vacuity control: a name that is NOT declared must gate.
        self.assertFalse(gate.is_advisory("some undeclared lane that must gate"))


class TestSelfTestIsRunnable(unittest.TestCase):
    def test_the_embedded_self_test_passes(self):
        proc = subprocess.run(
            [sys.executable, "-B", str(SCRIPT), "--self-test"],
            capture_output=True, text=True, timeout=120,
            env={"PATH": "/usr/bin:/bin", "PYTHONDONTWRITEBYTECODE": "1"})
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
