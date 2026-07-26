#!/usr/bin/env python3
# [OPUS-5] Hermetic test suite for the cron-only-lane liveness alarm (issue #4328).
# 🤖 SPARQ agent.
#
# Covers, with one NAMED test per guard so a deleted or inverted guard reds here:
#   * cron parsing + PERIOD DERIVATION from the lane's own `schedule:` (daily /
#     weekly / sub-hourly / multi-entry / Vixie dom-or-dow), and the fail-safe
#     `None` for anything indeterminate;
#   * the two raise rules — N consecutive hard failures (the bench-ec2 shape) and
#     no-green-inside-the-derived-window (the miri cancelled-at-ceiling shape);
#   * every QUIET-DIRECTION fail-safe: never-ran, disabled, unreadable cron,
#     new-lane grace, deliberately-skipped;
#   * SCOPE DISCOVERY — schedule+dispatch only is in, any visible trigger is out,
#     no-`schedule:` is out (the bench-ec2 retirement), formal-alarm's manifest
#     lanes are out, and the live .github/workflows tree yields a non-empty,
#     genuinely cron-only set;
#   * IDEMPOTENCE — two ticks over the same dead lane produce exactly ONE issue
#     (the second edits it), and a recovered lane closes its issue;
#   * FAIL-LOUD — pagination that disagrees with `total_count`, an unparseable
#     workflow, an empty scan set, and a failed issue write all exit 2. The last
#     one is the exit-zero-swallowing guard: a finding that cannot be published
#     must never leave a green run behind;
#   * the YAML SEAM — cron-liveness.yml really triggers on `schedule`, its job
#     `if:` is exactly the fork guard (not something that can silently evaluate
#     false), the self-test step and the detector step exist with NO step-level
#     `if:`, `issues: write` is granted, actions are SHA-pinned, and
#     docs-quality.yml really invokes THIS file unconditionally. Substring and
#     count(...) assertions do not catch `if: false`; these do.
#
# Needs PyYAML (already a CI dependency); everything else is stdlib. Run:
#   python3 scripts/tests/test_cron_lane_liveness.py

from __future__ import annotations

import datetime as dt
import importlib.util
import json
import re
import sys
import tempfile
import unittest
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SCRIPT = REPO_ROOT / "scripts" / "cron_lane_liveness.py"
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "cron-liveness.yml"
DOCS_QUALITY = REPO_ROOT / ".github" / "workflows" / "docs-quality.yml"
WORKFLOWS_DIR = REPO_ROOT / ".github" / "workflows"
FV_MANIFEST = REPO_ROOT / "ci" / "formal-verification.toml"

_spec = importlib.util.spec_from_file_location("cron_lane_liveness", SCRIPT)
cll = importlib.util.module_from_spec(_spec)
assert _spec.loader is not None
_spec.loader.exec_module(cll)

NOW = cll._parse_iso("2026-07-26T12:00:00Z")
DAILY = {"workflow": "x.yml", "display_name": "X", "crons": ["17 5 * * *"]}


def runs(*pairs: tuple[str, str]) -> list[dict]:
    """(conclusion, iso) newest-first."""
    return [{"conclusion": c, "created_at": t} for c, t in pairs]


def ago(hours: float) -> str:
    return (NOW - dt.timedelta(hours=hours)).strftime("%Y-%m-%dT%H:%M:%SZ")


# --------------------------------------------------------------------------- #
class TestCronFieldParsing(unittest.TestCase):
    def test_star_expands_to_full_range(self):
        self.assertEqual(cll.parse_cron_field("*", 0, 5), {0, 1, 2, 3, 4, 5})

    def test_step_over_star(self):
        self.assertEqual(cll.parse_cron_field("*/10", 0, 59),
                         {0, 10, 20, 30, 40, 50})

    def test_range_with_step(self):
        self.assertEqual(cll.parse_cron_field("1-10/3", 0, 59), {1, 4, 7, 10})

    def test_comma_list(self):
        self.assertEqual(cll.parse_cron_field("4,14,54", 0, 59), {4, 14, 54})

    def test_named_month_and_dow(self):
        self.assertEqual(cll.parse_cron_field("mar", 1, 12, cll._MONTHS), {3})
        self.assertEqual(cll.parse_cron_field("MON", 0, 7, cll._DOWS), {1})

    def test_out_of_range_is_cron_error(self):
        with self.assertRaises(cll.CronError):
            cll.parse_cron_field("99", 0, 59)

    def test_inverted_range_is_cron_error(self):
        with self.assertRaises(cll.CronError):
            cll.parse_cron_field("30-10", 0, 59)

    def test_wrong_field_count_is_cron_error(self):
        with self.assertRaises(cll.CronError):
            cll.parse_cron("0 6 * *")

    def test_sunday_seven_normalises_to_zero(self):
        self.assertEqual(cll.parse_cron("0 0 * * 7")["dow"], {0})


class TestPeriodDerivedFromSchedule(unittest.TestCase):
    """The expectation must come from the lane's OWN cron, never a table."""

    def test_daily_cron_yields_24h(self):
        self.assertAlmostEqual(cll.derive_period_hours(["41 5 * * *"], NOW), 24.0)

    def test_weekly_cron_yields_168h(self):
        self.assertAlmostEqual(cll.derive_period_hours(["0 6 * * 1"], NOW), 168.0)

    def test_ten_minute_cron_yields_a_sixth_of_an_hour(self):
        self.assertAlmostEqual(
            cll.derive_period_hours(["*/10 * * * *"], NOW), 1 / 6, places=6)

    def test_period_is_the_LARGEST_gap_not_the_smallest(self):
        # Two fires an hour apart then a 23h silence: the expectation is 23h.
        self.assertAlmostEqual(cll.derive_period_hours(["0 5,6 * * *"], NOW), 23.0)

    def test_multiple_schedule_entries_are_unioned(self):
        self.assertAlmostEqual(
            cll.derive_period_hours(["0 6 * * 1", "0 6 * * 4"], NOW), 96.0)

    def test_unparseable_cron_is_indeterminate(self):
        self.assertIsNone(cll.derive_period_hours(["not a cron"], NOW))

    def test_empty_schedule_is_indeterminate(self):
        self.assertIsNone(cll.derive_period_hours([], NOW))

    def test_cron_firing_at_most_once_in_the_window_is_indeterminate(self):
        # 29 Feb: at most one fire inside a 90-day window from July.
        self.assertIsNone(cll.derive_period_hours(["0 0 29 2 *"], NOW))

    def test_dom_and_dow_both_restricted_is_a_union_not_an_intersection(self):
        p = cll.parse_cron("0 0 1 * 1")
        self.assertTrue(cll._day_matches(p, dt.date(2026, 7, 6)))   # a Monday
        self.assertTrue(cll._day_matches(p, dt.date(2026, 7, 1)))   # the 1st
        self.assertFalse(cll._day_matches(p, dt.date(2026, 7, 8)))  # neither


class TestRaiseRules(unittest.TestCase):
    def test_n_consecutive_scheduled_failures_raises(self):
        rec = cll.classify_lane(
            DAILY, "active",
            runs(("failure", ago(1)), ("failure", ago(25)), ("failure", ago(49)),
                 ("success", ago(60))),
            NOW)
        self.assertTrue(rec["raise_alarm"])
        self.assertTrue(any(r.startswith("A:") for r in rec["rules_fired"]))

    def test_two_failures_after_a_success_do_not_raise(self):
        rec = cll.classify_lane(
            DAILY, "active",
            runs(("failure", ago(1)), ("failure", ago(25)), ("success", ago(49))),
            NOW)
        self.assertFalse(rec["raise_alarm"])

    def test_a_lane_that_succeeded_recently_does_not_raise(self):
        rec = cll.classify_lane(DAILY, "active", runs(("success", ago(2))), NOW)
        self.assertFalse(rec["raise_alarm"])
        self.assertEqual(rec["status"], "LIVE")

    def test_cancelled_at_ceiling_raises_via_the_no_green_rule(self):
        """The miri shape: never `failure`, so rule A cannot see it."""
        rec = cll.classify_lane(
            DAILY, "active",
            runs(("cancelled", ago(1)), ("cancelled", ago(25)),
                 ("cancelled", ago(49)), ("success", ago(400))),
            NOW)
        self.assertTrue(rec["raise_alarm"])
        self.assertEqual([r[0] for r in rec["rules_fired"]], ["B"])

    def test_success_just_inside_the_derived_window_does_not_raise(self):
        rec = cll.classify_lane(DAILY, "active", runs(("success", ago(71))), NOW)
        self.assertFalse(rec["raise_alarm"])

    def test_success_just_outside_the_derived_window_raises(self):
        rec = cll.classify_lane(DAILY, "active", runs(("success", ago(73))), NOW)
        self.assertTrue(rec["raise_alarm"])

    def test_weekly_lane_uses_its_own_longer_window(self):
        weekly = {"workflow": "w.yml", "crons": ["0 6 * * 1"]}
        # 200h stale would red a daily lane; a weekly lane's window is 504h.
        self.assertFalse(
            cll.classify_lane(weekly, "active", runs(("success", ago(200))),
                              NOW)["raise_alarm"])
        self.assertTrue(
            cll.classify_lane(weekly, "active", runs(("success", ago(600))),
                              NOW)["raise_alarm"])

    def test_subhourly_lane_gets_the_six_hour_jitter_floor(self):
        fast = {"workflow": "f.yml", "crons": ["*/10 * * * *"]}
        rec = cll.classify_lane(fast, "active", runs(("success", ago(5))), NOW)
        self.assertFalse(rec["raise_alarm"])          # 5h < the 6h floor
        self.assertAlmostEqual(rec["threshold_hours"], cll.MIN_GRACE_HOURS)

    def test_timed_out_and_startup_failure_count_as_hard_failures(self):
        rec = cll.classify_lane(
            DAILY, "active",
            runs(("timed_out", ago(1)), ("startup_failure", ago(25)),
                 ("failure", ago(49)), ("success", ago(60))),
            NOW)
        self.assertTrue(any(r.startswith("A:") for r in rec["rules_fired"]))


class TestQuietDirectionFailSafes(unittest.TestCase):
    def test_a_lane_with_no_runs_does_not_raise(self):
        rec = cll.classify_lane(DAILY, "active", [], NOW)
        self.assertFalse(rec["raise_alarm"])
        self.assertEqual(rec["status"], "NEVER-RAN")

    def test_a_disabled_lane_does_not_raise(self):
        rec = cll.classify_lane(
            DAILY, "disabled_manually",
            runs(("failure", ago(1)), ("failure", ago(25)), ("failure", ago(49))),
            NOW)
        self.assertFalse(rec["raise_alarm"])
        self.assertEqual(rec["status"], "DISABLED")

    def test_a_lane_disabled_for_inactivity_does_not_raise(self):
        rec = cll.classify_lane(
            DAILY, "disabled_inactivity",
            runs(("failure", ago(1)), ("failure", ago(25)), ("failure", ago(49))),
            NOW)
        self.assertFalse(rec["raise_alarm"])

    def test_an_unreadable_cron_does_not_raise(self):
        broken = {"workflow": "b.yml", "crons": ["every tuesday-ish"]}
        rec = cll.classify_lane(
            broken, "active",
            runs(("failure", ago(1)), ("failure", ago(25)), ("failure", ago(49))),
            NOW)
        self.assertFalse(rec["raise_alarm"])
        self.assertEqual(rec["status"], "INDETERMINATE-CRON")

    def test_a_brand_new_never_green_lane_is_inside_its_grace_period(self):
        """verdict-bridge shape: first run an hour ago, newest one red."""
        fast = {"workflow": "n.yml", "crons": ["*/10 * * * *"]}
        rec = cll.classify_lane(
            fast, "active",
            runs(("failure", ago(0.2)), ("success", ago(1.2))), NOW)
        self.assertFalse(rec["raise_alarm"])

    def test_a_new_lane_that_never_succeeded_is_quiet_until_the_window_passes(self):
        """Rule B only: two runs, so rule A (which needs three) cannot fire."""
        rec = cll.classify_lane(
            DAILY, "active",
            runs(("failure", ago(1)), ("failure", ago(25))), NOW)
        self.assertFalse(rec["raise_alarm"])  # oldest run only 25h old < 72h
        rec = cll.classify_lane(
            DAILY, "active",
            runs(("failure", ago(1)), ("failure", ago(80))), NOW)
        self.assertTrue(rec["raise_alarm"])   # now it has had its chance
        self.assertEqual([r[0] for r in rec["rules_fired"]], ["B"])

    def test_rule_a_needs_the_failure_streak_to_span_the_jitter_floor(self):
        """Three failures of a 10-minute lane is 20 minutes — a transient."""
        fast = {"workflow": "f.yml", "crons": ["*/10 * * * *"]}
        rec = cll.classify_lane(
            fast, "active",
            runs(("failure", ago(0.1)), ("failure", ago(0.2)),
                 ("failure", ago(0.3)), ("success", ago(0.4))),
            NOW)
        self.assertFalse(rec["raise_alarm"])
        rec = cll.classify_lane(
            fast, "active",
            runs(("failure", ago(0.1)), ("failure", ago(3.0)),
                 ("failure", ago(7.0)), ("success", ago(7.5))),
            NOW)
        self.assertTrue(rec["raise_alarm"])   # 7h streak clears the 6h floor

    def test_a_skipped_newest_run_is_deliberately_inert(self):
        """bench-ec2's `vars.AWS_BENCH_ROLE_ARN != ''` role-present guard shape."""
        rec = cll.classify_lane(
            DAILY, "active",
            runs(("skipped", ago(1)), ("skipped", ago(25)), ("skipped", ago(49))),
            NOW)
        self.assertFalse(rec["raise_alarm"])
        self.assertEqual(rec["status"], "DELIBERATELY-INERT")


# --------------------------------------------------------------------------- #
class TestScopeDiscovery(unittest.TestCase):
    """Scope is DERIVED from each workflow's own `on:` block."""

    @staticmethod
    def _dir(files: dict[str, str]) -> Path:
        tmp = Path(tempfile.mkdtemp())
        for name, text in files.items():
            (tmp / name).write_text(text, encoding="utf-8")
        return tmp

    def test_schedule_plus_dispatch_only_is_in_scope(self):
        d = self._dir({"a.yml": "name: a\non:\n  schedule:\n    - cron: '0 5 * * *'\n"
                                "  workflow_dispatch:\njobs: {}\n"})
        self.assertEqual([x["workflow"] for x in cll.discover_cron_only_lanes(d)],
                         ["a.yml"])

    def test_a_pull_request_trigger_takes_a_lane_out_of_scope(self):
        d = self._dir({"a.yml": "name: a\non:\n  schedule:\n    - cron: '0 5 * * *'\n"
                                "  pull_request:\njobs: {}\n",
                       "b.yml": "name: b\non:\n  schedule:\n    - cron: '0 5 * * *'\n"
                                "jobs: {}\n"})
        self.assertEqual([x["workflow"] for x in cll.discover_cron_only_lanes(d)],
                         ["b.yml"])

    def test_a_workflow_run_trigger_takes_a_lane_out_of_scope(self):
        d = self._dir({"a.yml": "name: a\non:\n  schedule:\n    - cron: '0 5 * * *'\n"
                                "  workflow_run:\n    workflows: [CI]\njobs: {}\n"})
        self.assertEqual(cll.discover_cron_only_lanes(d), [])

    def test_a_workflow_with_no_schedule_is_out_of_scope(self):
        """The bench-ec2 retirement (#3785): removing `schedule:` leaves scope."""
        d = self._dir({"bench-ec2.yml": "name: b\non:\n  workflow_dispatch:\njobs: {}\n"})
        self.assertEqual(cll.discover_cron_only_lanes(d), [])

    def test_a_bench_ec2_shaped_lane_WITH_a_schedule_is_in_scope_and_raises(self):
        """Proof the bench-ec2 class is covered, not special-cased away."""
        d = self._dir({"bench-ec2.yml":
                       "name: Benchmarks (EC2)\non:\n  schedule:\n"
                       "    - cron: '0 6 * * *'\n  workflow_dispatch:\njobs: {}\n"})
        lanes = cll.discover_cron_only_lanes(d)
        self.assertEqual([x["workflow"] for x in lanes], ["bench-ec2.yml"])
        rec = cll.classify_lane(
            lanes[0], "active",
            runs(("failure", ago(1)), ("failure", ago(25)), ("failure", ago(49)),
                 ("failure", ago(73))),
            NOW)
        self.assertTrue(rec["raise_alarm"])

    def test_formal_alarm_manifest_lanes_are_excluded_by_reading_the_manifest(self):
        tmp = Path(tempfile.mkdtemp())
        (tmp / "kani.yml").write_text(
            "name: k\non:\n  schedule:\n    - cron: '0 6 * * *'\njobs: {}\n")
        (tmp / "other.yml").write_text(
            "name: o\non:\n  schedule:\n    - cron: '0 6 * * *'\njobs: {}\n")
        man = tmp / "fv.toml"
        man.write_text('schema = 1\n[[lane]]\nworkflow = "kani.yml"\n'
                       'max_verdict_age_hours = 48\n')
        self.assertEqual(
            [x["workflow"] for x in cll.discover_cron_only_lanes(tmp, man)],
            ["other.yml"])

    def test_an_unparseable_workflow_is_fail_loud_not_a_silent_scope_shrink(self):
        d = self._dir({"broken.yml": "name: [unclosed\n"})
        with self.assertRaises(cll.AlarmError):
            cll.discover_cron_only_lanes(d)

    def test_bare_on_key_resolved_to_yaml_true_is_still_read(self):
        d = self._dir({"a.yml": "name: a\non:\n  schedule:\n    - cron: '0 5 * * *'\n"
                                "jobs: {}\n"})
        doc = yaml.safe_load((d / "a.yml").read_text())
        self.assertEqual(cll.workflow_triggers(doc), {"schedule"})


class TestLiveRepositoryScope(unittest.TestCase):
    """The scan applied to the REAL .github/workflows tree."""

    def setUp(self):
        self.lanes = cll.discover_cron_only_lanes(WORKFLOWS_DIR, FV_MANIFEST)
        self.names = {x["workflow"] for x in self.lanes}

    def test_the_live_scan_finds_at_least_one_lane(self):
        # A scan that finds nothing is a broken scan, not a clean repo.
        self.assertGreater(len(self.lanes), 0)

    def test_every_discovered_lane_really_is_cron_only(self):
        for lane in self.lanes:
            doc = yaml.safe_load(
                (WORKFLOWS_DIR / lane["workflow"]).read_text(encoding="utf-8"))
            trig = cll.workflow_triggers(doc)
            self.assertIn("schedule", trig, lane["workflow"])
            self.assertTrue(trig <= cll.INVISIBLE_TRIGGERS, lane["workflow"])

    def test_every_discovered_lane_has_a_derivable_period(self):
        for lane in self.lanes:
            self.assertIsNotNone(cll.derive_period_hours(lane["crons"], NOW),
                                 f"{lane['workflow']} cron is unreadable")

    def test_pr_visible_workflows_are_not_in_scope(self):
        for visible in ("ci.yml", "bench.yml", "fuzz.yml", "codeql.yml"):
            self.assertNotIn(visible, self.names)

    def test_formal_alarm_lanes_are_not_double_watched(self):
        for watched in cll.formal_alarm_watched(FV_MANIFEST):
            self.assertNotIn(watched, self.names)

    def test_this_alarm_watches_itself(self):
        self.assertIn("cron-liveness.yml", self.names)

    def test_bench_ec2_is_in_scope_exactly_when_it_has_a_schedule(self):
        path = WORKFLOWS_DIR / "bench-ec2.yml"
        if not path.exists():
            self.skipTest("bench-ec2.yml removed")
        doc = yaml.safe_load(path.read_text(encoding="utf-8"))
        scheduled = "schedule" in cll.workflow_triggers(doc)
        self.assertEqual(scheduled, "bench-ec2.yml" in self.names)


# --------------------------------------------------------------------------- #
class FakeGh:
    """Records every `gh` invocation the alarm makes; serves canned reads."""

    def __init__(self, open_issues: list[dict] | None = None,
                 fail_on: str | None = None):
        self.calls: list[list[str]] = []
        self.open_issues = list(open_issues or [])
        self.fail_on = fail_on
        self._next_number = 900

    def __call__(self, args: list[str]) -> str:
        self.calls.append(args)
        joined = " ".join(args)
        if self.fail_on and self.fail_on in joined:
            raise cll.AlarmError(f"simulated gh failure: {self.fail_on}")
        if args[:2] == ["gh", "api"] and "/issues?" in args[2]:
            page = int(re.search(r"[?&]page=(\d+)", args[2]).group(1))
            return json.dumps(self.open_issues if page == 1 else [])
        if args[:3] == ["gh", "issue", "create"]:
            self._next_number += 1
            body = args[args.index("--body") + 1]
            self.open_issues.append({"number": self._next_number, "body": body})
            return f"https://github.com/o/r/issues/{self._next_number}\n"
        return ""

    def kinds(self) -> list[str]:
        return [" ".join(c[1:3]) for c in self.calls]


DEAD_STATE = {"z.yml": {"state": "active", "runs": [
    {"conclusion": "failure", "created_at": "2026-07-26T05:00:00Z"},
    {"conclusion": "failure", "created_at": "2026-07-25T05:00:00Z"},
    {"conclusion": "failure", "created_at": "2026-07-24T05:00:00Z"},
    {"conclusion": "failure", "created_at": "2026-07-20T05:00:00Z"},
]}}
LIVE_STATE = {"z.yml": {"state": "active", "runs": [
    {"conclusion": "success", "created_at": "2026-07-26T05:00:00Z"},
]}}


class _EndToEnd(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.wf = self.tmp / "workflows"
        self.wf.mkdir()
        (self.wf / "z.yml").write_text(
            "name: z\non:\n  schedule:\n    - cron: '0 5 * * *'\n"
            "  workflow_dispatch:\njobs: {}\n")
        self._real_gh = cll._gh

    def tearDown(self):
        cll._gh = self._real_gh

    def run_main(self, state: dict, extra: list[str] | None = None) -> int:
        sf = self.tmp / "state.json"
        sf.write_text(json.dumps(state))
        return cll.main(["--workflows-dir", str(self.wf),
                         "--fv-manifest", str(self.tmp / "nope.toml"),
                         "--state-file", str(sf), "--repo", "o/r",
                         "--now", "2026-07-26T12:00:00Z", *(extra or [])])


class TestIdempotence(_EndToEnd):
    def test_a_dead_lane_files_exactly_one_issue(self):
        fake = FakeGh()
        cll._gh = fake
        self.assertEqual(self.run_main(DEAD_STATE), 0)
        self.assertEqual(fake.kinds().count("issue create"), 1)

    def test_two_consecutive_ticks_do_not_open_a_second_issue(self):
        fake = FakeGh()
        cll._gh = fake
        self.run_main(DEAD_STATE)
        self.run_main(DEAD_STATE)
        self.assertEqual(fake.kinds().count("issue create"), 1)
        self.assertEqual(fake.kinds().count("issue edit"), 1)

    def test_the_second_tick_refreshes_the_existing_issue_in_place(self):
        fake = FakeGh(open_issues=[
            {"number": 4242, "body": f"<!-- {cll.KEY_PREFIX}: z.yml -->"}])
        cll._gh = fake
        self.run_main(DEAD_STATE)
        edits = [c for c in fake.calls if c[:3] == ["gh", "issue", "edit"]]
        self.assertEqual(len(edits), 1)
        self.assertEqual(edits[0][3], "4242")
        self.assertEqual(fake.kinds().count("issue create"), 0)

    def test_a_recovered_lane_closes_its_open_alarm_issue(self):
        fake = FakeGh(open_issues=[
            {"number": 4242, "body": f"<!-- {cll.KEY_PREFIX}: z.yml -->"}])
        cll._gh = fake
        self.assertEqual(self.run_main(LIVE_STATE), 0)
        self.assertEqual(fake.kinds().count("issue close"), 1)

    def test_a_healthy_lane_with_no_open_issue_writes_nothing(self):
        fake = FakeGh()
        cll._gh = fake
        self.assertEqual(self.run_main(LIVE_STATE), 0)
        self.assertNotIn("issue create", fake.kinds())
        self.assertNotIn("issue close", fake.kinds())

    def test_dry_run_performs_no_gh_writes_at_all(self):
        fake = FakeGh()
        cll._gh = fake
        self.assertEqual(self.run_main(DEAD_STATE, ["--dry-run"]), 0)
        self.assertEqual(fake.calls, [])


class TestFailLoud(_EndToEnd):
    def test_a_failed_issue_write_exits_2_and_never_swallows_the_finding(self):
        cll._gh = FakeGh(fail_on="issue create")
        self.assertEqual(self.run_main(DEAD_STATE), 2)

    def test_an_unparseable_workflow_exits_2(self):
        (self.wf / "bad.yml").write_text("name: [unclosed\n")
        cll._gh = FakeGh()
        self.assertEqual(self.run_main(DEAD_STATE), 2)

    def test_an_empty_scan_set_exits_2(self):
        (self.wf / "z.yml").unlink()
        cll._gh = FakeGh()
        self.assertEqual(self.run_main(DEAD_STATE), 2)

    def test_a_live_run_without_a_repo_exits_2(self):
        cll._gh = FakeGh()
        self.assertEqual(
            cll.main(["--workflows-dir", str(self.wf), "--repo", "",
                      "--now", "2026-07-26T12:00:00Z", "--dry-run"]), 2)

    def test_pagination_short_of_total_count_is_fail_loud(self):
        calls = {"n": 0}

        def fake_json(path: str):
            calls["n"] += 1
            return {"total_count": 50, "workflow_runs": []}

        real = cll._gh_json
        try:
            cll._gh_json = fake_json
            with self.assertRaises(cll.AlarmError):
                cll.fetch_scheduled_runs("o/r", "z.yml")
        finally:
            cll._gh_json = real

    def test_a_runs_response_without_total_count_is_fail_loud(self):
        real = cll._gh_json
        try:
            cll._gh_json = lambda path: {"workflow_runs": []}
            with self.assertRaises(cll.AlarmError):
                cll.fetch_scheduled_runs("o/r", "z.yml")
        finally:
            cll._gh_json = real

    def test_pagination_walks_every_page_until_total_count_is_met(self):
        pages: dict[int, list[dict]] = {
            1: [{"conclusion": "success", "created_at": f"2026-07-{d:02d}T00:00:00Z"}
                for d in range(1, 26)],
            2: [{"conclusion": "failure", "created_at": "2026-06-01T00:00:00Z"}],
        }
        seen: list[int] = []

        def fake_json(path: str):
            page = int(re.search(r"[?&]page=(\d+)", path).group(1))
            seen.append(page)
            return {"total_count": 26, "workflow_runs": pages.get(page, [])}

        real = cll._gh_json
        try:
            cll._gh_json = fake_json
            got = cll.fetch_scheduled_runs("o/r", "z.yml")
        finally:
            cll._gh_json = real
        self.assertEqual(seen, [1, 2])
        self.assertEqual(len(got), 26)


# --------------------------------------------------------------------------- #
class TestYamlSeam(unittest.TestCase):
    """`if: false`, a deleted step, or a dropped permission must red HERE.

    Substring / count(...) assertions do not catch a disarmed step, so each
    property is asserted on the PARSED structure at its exact location.
    """

    @classmethod
    def setUpClass(cls):
        cls.doc = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))
        cls.jobs = cls.doc["jobs"]
        cls.job = cls.jobs["liveness"]
        cls.steps = cls.job["steps"]

    def test_the_workflow_file_exists(self):
        self.assertTrue(WORKFLOW.is_file())

    def test_it_is_triggered_by_schedule(self):
        on = cll._on_block(self.doc)
        self.assertIn("schedule", on)
        self.assertTrue(cll.derive_period_hours(cll.workflow_crons(self.doc), NOW))

    def test_the_alarm_job_if_is_exactly_the_fork_guard(self):
        # Pinned literally: any other expression (including `false`) reds here.
        self.assertEqual(self.job.get("if"),
                         "github.repository == 'sparq-org/sparq'")

    def test_the_detector_step_exists_and_is_unconditional(self):
        hits = [s for s in self.steps
                if "scripts/cron_lane_liveness.py" in (s.get("run") or "")]
        self.assertEqual(len(hits), 1)
        self.assertNotIn("if", hits[0])

    def test_the_selftest_step_exists_and_is_unconditional(self):
        hits = [s for s in self.steps
                if "scripts/tests/test_cron_lane_liveness.py" in (s.get("run") or "")]
        self.assertEqual(len(hits), 1)
        self.assertNotIn("if", hits[0])

    def test_the_selftest_runs_before_the_detector(self):
        idx = {"self": None, "det": None}
        for i, s in enumerate(self.steps):
            run = s.get("run") or ""
            if "scripts/tests/test_cron_lane_liveness.py" in run:
                idx["self"] = i
            elif "scripts/cron_lane_liveness.py" in run:
                idx["det"] = i
        self.assertLess(idx["self"], idx["det"])

    def test_the_job_can_write_issues(self):
        self.assertEqual(self.job["permissions"].get("issues"), "write")
        self.assertEqual(self.job["permissions"].get("actions"), "read")

    def test_no_step_is_disarmed_with_a_constant_false(self):
        for s in self.steps:
            self.assertNotIn(str(s.get("if", "")).strip().lower(),
                             {"false", "${{ false }}"})

    def test_every_action_use_is_sha_pinned(self):
        for s in self.steps:
            if "uses" in s:
                self.assertRegex(s["uses"], r"@[0-9a-f]{40}$", s["uses"])

    def test_the_job_name_carries_no_advisory_token(self):
        # An advisory/informational token would demand an advisory-registry entry.
        self.assertNotRegex(self.job["name"], r"\b(advisory|informational)\b")

    def test_docs_quality_gates_this_test_file_unconditionally(self):
        dq = yaml.safe_load(DOCS_QUALITY.read_text(encoding="utf-8"))
        hits = [(jid, job, s) for jid, job in dq["jobs"].items()
                for s in (job.get("steps") or [])
                if "scripts/tests/test_cron_lane_liveness.py" in (s.get("run") or "")]
        self.assertEqual(len(hits), 1)
        jid, job, step = hits[0]
        self.assertNotIn("if", step)
        self.assertNotIn("if", job)


if __name__ == "__main__":
    unittest.main(verbosity=2)
