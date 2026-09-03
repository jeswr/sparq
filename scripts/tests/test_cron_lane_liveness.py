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
import shlex
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
        # 29 Feb from a 2026-07 start: the window reaches 2027-08, and 2027 is
        # not a leap year, so this cron fires ZERO times — no gap, no
        # expectation, and the lane goes quiet rather than alarming.
        self.assertIsNone(cll.derive_period_hours(["0 0 29 2 *"], NOW))

    def test_a_QUARTERLY_cron_is_derivable_not_indeterminate(self):
        """`slsa-builder-pin-review.yml` (`37 6 1 1,4,7,10 *`) landed on main and
        fires ONCE in 90 days. Under the old window its period was None, so the
        lane was in scope and permanently unwatchable — quiet, not covered."""
        period = cll.derive_period_hours(["37 6 1 1,4,7,10 *"], NOW)
        self.assertIsNotNone(period)
        self.assertGreater(period, 80 * 24)    # a quarter, not a day
        self.assertLess(period, 95 * 24)

    def test_the_window_is_long_enough_to_see_two_fires_of_a_quarterly_cron(self):
        """Pins the REASON for the window length, so shortening it reds here and
        not only in the live-scope test that happens to have such a lane."""
        fires = cll.cron_fire_times(["37 6 1 1,4,7,10 *"], NOW)
        self.assertGreaterEqual(len(fires), 2)

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

    def test_a_lane_that_STARTS_skipping_after_failing_goes_quiet(self):
        """The newest-run test must dominate: a guard turned on after a bad run
        is a deliberate off switch, so the older failures must not still raise.
        Distinguishes the newest-run check from the all-runs-skipped check."""
        rec = cll.classify_lane(
            DAILY, "active",
            runs(("skipped", ago(1)), ("failure", ago(25)), ("failure", ago(49)),
                 ("failure", ago(73))),
            NOW)
        self.assertFalse(rec["raise_alarm"])
        self.assertEqual(rec["status"], "DELIBERATELY-INERT")


# --------------------------------------------------------------------------- #
VERDICT_DAILY = dict(DAILY, verdict_lane=True)


class TestDetectorLanesAreNotDeadLanes(unittest.TestCase):
    """A DETECTOR exits non-zero when it FINDS something, so `failure` is its
    verdict, not its breakage. Rules A and B read conclusion as a health proxy
    and therefore classify a WORKING detector as dead — measured live against
    `review-alarm.yml` (4/4 `failure`, every one a correct finding).

    Rule V replaces them for a lane that DECLARES itself, and only for that lane.
    """

    # ---- the declaration is read from the lane's own YAML, exactly ---------- #
    def test_a_column_zero_declaration_line_declares(self):
        self.assertTrue(cll.workflow_declares_verdict_lane(
            "# review-alarm\n# cron-liveness: verdict-lane\nname: x\n"))

    def test_an_INDENTED_occurrence_does_not_declare(self):
        """It must not be smugglable from inside a `run:` block or a heredoc."""
        self.assertFalse(cll.workflow_declares_verdict_lane(
            "jobs:\n  a:\n    steps:\n      - run: |\n"
            "          # cron-liveness: verdict-lane\n"))

    def test_a_PROSE_MENTION_of_the_token_does_not_declare(self):
        """Containment would accept this; a whole-line match must not."""
        self.assertFalse(cll.workflow_declares_verdict_lane(
            "# this lane is not a `# cron-liveness: verdict-lane` and never was\n"))
        self.assertFalse(cll.workflow_declares_verdict_lane(
            "# cron-liveness: verdict-lane-NOT\n"))

    def test_an_undeclared_lane_is_not_a_verdict_lane(self):
        self.assertFalse(cll.workflow_declares_verdict_lane(
            "# cron-liveness\nname: x\non:\n  schedule: []\n"))

    def test_discovery_reads_the_declaration_off_the_file(self):
        tmp = Path(tempfile.mkdtemp())
        (tmp / "d.yml").write_text(
            "# cron-liveness: verdict-lane\nname: d\non:\n  schedule:\n"
            "    - cron: '0 5 * * *'\njobs: {}\n", encoding="utf-8")
        (tmp / "p.yml").write_text(
            "name: p\non:\n  schedule:\n    - cron: '0 5 * * *'\njobs: {}\n",
            encoding="utf-8")
        got = {x["workflow"]: x["verdict_lane"]
               for x in cll.discover_cron_only_lanes(tmp)}
        self.assertEqual(got, {"d.yml": True, "p.yml": False})

    # ---- what the declaration changes --------------------------------------- #
    def test_a_declared_lane_failing_every_run_is_LIVE_not_dead(self):
        """The live review-alarm.yml shape: 4/4 `failure`, all correct findings."""
        rec = cll.classify_lane(
            VERDICT_DAILY, "active",
            runs(("failure", ago(1)), ("failure", ago(6)), ("failure", ago(12)),
                 ("failure", ago(18))),
            NOW)
        self.assertFalse(rec["raise_alarm"])
        self.assertEqual(rec["status"], "LIVE-VERDICT-LANE")
        self.assertEqual(rec["last_verdict"], cll._parse_iso(ago(1)).isoformat())

    def test_the_SAME_history_on_an_UNDECLARED_lane_still_raises(self):
        """The declaration is the ONLY difference — otherwise this fixture is
        the bench-ec2 shape and rules A and B must be untouched by rule V."""
        rec = cll.classify_lane(
            DAILY, "active",
            runs(("failure", ago(1)), ("failure", ago(6)), ("failure", ago(12)),
                 ("failure", ago(18))),
            NOW)
        self.assertTrue(rec["raise_alarm"])
        self.assertEqual(rec["status"], "DEAD")

    def test_a_declared_lane_that_STOPPED_speaking_still_raises(self):
        """The declaration is not a mute: no verdict inside the window raises."""
        rec = cll.classify_lane(
            VERDICT_DAILY, "active",
            runs(("failure", ago(80)), ("success", ago(104))), NOW)
        self.assertTrue(rec["raise_alarm"])
        self.assertIn("V:", rec["reason"])

    def test_a_declared_lane_cancelled_at_its_ceiling_still_raises(self):
        """`cancelled` is NOT a verdict — the miri shape survives the change."""
        rec = cll.classify_lane(
            VERDICT_DAILY, "active",
            runs(("cancelled", ago(1)), ("cancelled", ago(25)),
                 ("cancelled", ago(49)), ("success", ago(100))),
            NOW)
        self.assertTrue(rec["raise_alarm"])

    def test_a_declared_lane_timing_out_every_run_still_raises(self):
        rec = cll.classify_lane(
            VERDICT_DAILY, "active",
            runs(("timed_out", ago(1)), ("timed_out", ago(25)),
                 ("timed_out", ago(49)), ("timed_out", ago(73))),
            NOW)
        self.assertTrue(rec["raise_alarm"])

    def test_a_declared_lane_never_having_spoken_is_inside_its_grace(self):
        rec = cll.classify_lane(
            VERDICT_DAILY, "active",
            runs(("cancelled", ago(1)), ("cancelled", ago(3))), NOW)
        self.assertFalse(rec["raise_alarm"])
        self.assertEqual(rec["status"], "LIVE-VERDICT-LANE")

    def test_a_declared_lane_succeeding_is_LIVE(self):
        rec = cll.classify_lane(
            VERDICT_DAILY, "active", runs(("success", ago(1))), NOW)
        self.assertFalse(rec["raise_alarm"])
        self.assertEqual(rec["last_verdict_conclusion"], "success")

    def test_the_raised_issue_for_a_declared_lane_says_why_it_raised_anyway(self):
        rec = cll.classify_lane(
            VERDICT_DAILY, "active",
            runs(("cancelled", ago(1)), ("cancelled", ago(25)),
                 ("cancelled", ago(49)), ("success", ago(100))),
            NOW)
        _, body = cll.render_issue(rec)
        self.assertIn("# cron-liveness: verdict-lane", body)
        self.assertIn("never got to speak", body)


class TestLiveDetectorLanesDoNotFalseAlarm(unittest.TestCase):
    """Bound to the REAL workflow files: the two live detector lanes must carry
    the declaration, and this alarm must NOT (its finding exits 0, so a
    `failure` from it genuinely means the detector is broken)."""

    DETECTORS = ("review-alarm.yml", "formal-alarm.yml")

    def test_the_live_detector_lanes_declare_themselves(self):
        for name in self.DETECTORS:
            path = WORKFLOWS_DIR / name
            if not path.exists():
                self.skipTest(f"{name} removed")
            self.assertTrue(
                cll.workflow_declares_verdict_lane(path.read_text(encoding="utf-8")),
                f"{name} exits non-zero on a FINDING and must declare "
                "`# cron-liveness: verdict-lane`, or this alarm reports it dead")

    def test_this_alarm_does_NOT_declare_itself_a_verdict_lane(self):
        # Self-declaring would be the mute switch: a red cron-liveness run means
        # AlarmError (exit 2), which is real breakage.
        self.assertFalse(
            cll.workflow_declares_verdict_lane(WORKFLOW.read_text(encoding="utf-8")))

    def test_a_declared_detector_that_reds_on_every_run_reads_LIVE(self):
        """End-to-end over the real review-alarm.yml, with its real measured
        history (4/4 `failure`): at head 06fc5577 this raised a DEAD alarm."""
        path = WORKFLOWS_DIR / "review-alarm.yml"
        if not path.exists():
            self.skipTest("review-alarm.yml removed")
        lanes = [x for x in cll.discover_cron_only_lanes(WORKFLOWS_DIR, FV_MANIFEST)
                 if x["workflow"] == "review-alarm.yml"]
        self.assertEqual(len(lanes), 1, "review-alarm.yml left this alarm's scope")
        rec = cll.classify_lane(
            lanes[0], "active",
            runs(("failure", ago(1.8)), ("failure", ago(7.2)),
                 ("failure", ago(15.2)), ("failure", ago(20.0))),
            NOW)
        self.assertFalse(rec["raise_alarm"], rec["reason"])


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

    @classmethod
    def setUpClass(cls):
        # Parsing ~70 workflow files is the slow part; do it once for the class.
        cls.lanes = cll.discover_cron_only_lanes(WORKFLOWS_DIR, FV_MANIFEST)
        cls.names = {x["workflow"] for x in cls.lanes}

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
                 fail_on: str | None = None,
                 create_returns: str | None = None,
                 read_back_as: int | None = None):
        self.calls: list[list[str]] = []
        self.open_issues = list(open_issues or [])
        self.fail_on = fail_on
        # `gh issue create` exiting 0 with no/again-wrong output — the secondary
        # content-creation rate-limit shape.
        self.create_returns = create_returns
        self.read_back_as = read_back_as
        self._next_number = 900

    def __call__(self, args: list[str]) -> str:
        self.calls.append(args)
        joined = " ".join(args)
        if self.fail_on and self.fail_on in joined:
            raise cll.AlarmError(f"simulated gh failure: {self.fail_on}")
        if args[:2] == ["gh", "api"] and "/issues?" in args[2]:
            page = int(re.search(r"[?&]page=(\d+)", args[2]).group(1))
            return json.dumps(self.open_issues if page == 1 else [])
        # Read-back of a single created issue: `repos/<o>/<r>/issues/<n>`.
        m = args[:2] == ["gh", "api"] and re.fullmatch(
            r"repos/[^/]+/[^/]+/issues/(\d+)", args[2])
        if m:
            return json.dumps({"number": self.read_back_as
                               if self.read_back_as is not None else int(m.group(1))})
        if args[:3] == ["gh", "issue", "create"]:
            self._next_number += 1
            body = args[args.index("--body") + 1]
            self.open_issues.append({"number": self._next_number, "body": body})
            if self.create_returns is not None:
                return self.create_returns
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

    def test_a_labelled_PULL_REQUEST_is_never_mistaken_for_the_alarm_issue(self):
        """The REST issues endpoint also returns PRs — including the PR that
        introduced this alarm, whose body quotes the dedupe marker."""
        fake = FakeGh(open_issues=[
            {"number": 4368, "pull_request": {"url": "…"},
             "body": f"<!-- {cll.KEY_PREFIX}: z.yml -->"}])
        cll._gh = fake
        self.run_main(DEAD_STATE)
        self.assertEqual(fake.kinds().count("issue create"), 1)
        self.assertEqual(fake.kinds().count("issue edit"), 0)

    def test_a_recovered_lane_closes_its_open_alarm_issue(self):
        fake = FakeGh(open_issues=[
            {"number": 4242, "body": f"<!-- {cll.KEY_PREFIX}: z.yml -->"}])
        cll._gh = fake
        self.assertEqual(self.run_main(LIVE_STATE), 0)
        self.assertEqual(fake.kinds().count("issue close"), 1)

    def test_the_closing_comment_quotes_the_success_it_claims(self):
        fake = FakeGh(open_issues=[
            {"number": 4242, "body": f"<!-- {cll.KEY_PREFIX}: z.yml -->"}])
        cll._gh = fake
        self.run_main(LIVE_STATE)
        closes = [c for c in fake.calls if c[:3] == ["gh", "issue", "close"]]
        comment = closes[0][closes[0].index("--comment") + 1]
        self.assertIn("2026-07-26T05:00:00", comment)

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


class TestQuietIsNotRecovered(_EndToEnd):
    """An open alarm is closed ONLY on positive evidence of a healthy run.

    Every state below is QUIET — none of them raises — and NONE of them is a
    recovery. Closing on "it stopped raising" would assert something false and
    re-create the invisibility the alarm exists to end.
    """

    OPEN = [{"number": 4242, "body": f"<!-- {cll.KEY_PREFIX}: z.yml -->"}]

    def _quiet(self, state: dict) -> FakeGh:
        fake = FakeGh(open_issues=[dict(x) for x in self.OPEN])
        cll._gh = fake
        self.assertEqual(self.run_main(state), 0)
        return fake

    def test_a_newly_SKIPPED_lane_does_not_close_its_alarm(self):
        """The live remediation shape: #3785 fixed bench-ec2 by adding a
        role-present job guard, which makes runs `skipped`."""
        fake = self._quiet({"z.yml": {"state": "active", "runs": [
            {"conclusion": "skipped", "created_at": "2026-07-26T05:00:00Z"},
            {"conclusion": "failure", "created_at": "2026-07-25T05:00:00Z"},
            {"conclusion": "failure", "created_at": "2026-07-24T05:00:00Z"}]}})
        self.assertEqual(fake.kinds().count("issue close"), 0)

    def test_a_DISABLED_lane_does_not_close_its_alarm(self):
        fake = self._quiet({"z.yml": {"state": "disabled_manually", "runs": [
            {"conclusion": "failure", "created_at": "2026-07-26T05:00:00Z"}]}})
        self.assertEqual(fake.kinds().count("issue close"), 0)

    def test_a_lane_with_NO_RUNS_does_not_close_its_alarm(self):
        fake = self._quiet({"z.yml": {"state": "active", "runs": []}})
        self.assertEqual(fake.kinds().count("issue close"), 0)

    def test_a_lane_inside_its_NEW_LANE_GRACE_does_not_close_its_alarm(self):
        fake = self._quiet({"z.yml": {"state": "active", "runs": [
            {"conclusion": "failure", "created_at": "2026-07-26T11:00:00Z"}]}})
        self.assertEqual(fake.kinds().count("issue close"), 0)

    def test_a_DECLARED_verdict_lane_that_has_never_SPOKEN_does_not_close(self):
        """A declared verdict-lane inside its new-lane grace has produced no
        verdict at all. It raises nothing — and it has recovered from nothing."""
        (self.wf / "z.yml").write_text(
            "# cron-liveness: verdict-lane\nname: z\non:\n  schedule:\n"
            "    - cron: '0 5 * * *'\n  workflow_dispatch:\njobs: {}\n",
            encoding="utf-8")
        fake = self._quiet({"z.yml": {"state": "active", "runs": [
            {"conclusion": "cancelled", "created_at": "2026-07-26T11:00:00Z"}]}})
        self.assertEqual(fake.kinds().count("issue close"), 0)

    def test_a_DECLARED_verdict_lane_that_SPOKE_does_close(self):
        """Control for the test above: `failure` IS a verdict on a declared
        lane, so it is a genuine recovery and must still close."""
        (self.wf / "z.yml").write_text(
            "# cron-liveness: verdict-lane\nname: z\non:\n  schedule:\n"
            "    - cron: '0 5 * * *'\n  workflow_dispatch:\njobs: {}\n",
            encoding="utf-8")
        fake = self._quiet({"z.yml": {"state": "active", "runs": [
            {"conclusion": "failure", "created_at": "2026-07-26T11:00:00Z"}]}})
        self.assertEqual(fake.kinds().count("issue close"), 1)
        closes = [c for c in fake.calls if c[:3] == ["gh", "issue", "close"]]
        self.assertIn("`failure`", closes[0][closes[0].index("--comment") + 1])

    def test_a_left_open_alarm_is_REPORTED_not_silently_kept(self):
        import contextlib
        import io
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            self._quiet({"z.yml": {"state": "disabled_manually", "runs": [
                {"conclusion": "failure", "created_at": "2026-07-26T05:00:00Z"}]}})
        self.assertIn("LEFT OPEN", buf.getvalue())

    def test_a_genuine_success_STILL_closes(self):
        """The guard above must not have vacated the close path entirely."""
        fake = self._quiet(LIVE_STATE)
        self.assertEqual(fake.kinds().count("issue close"), 1)


class TestTheWriteIsVerifiedByReadingItBack(_EndToEnd):
    """`gh issue create` can exit 0 having created NOTHING under a secondary
    content-creation rate limit, which no `rate_limit` field reports. The
    finding would then be dropped behind a green run."""

    def test_a_create_that_returns_no_url_exits_2(self):
        cll._gh = FakeGh(create_returns="")
        self.assertEqual(self.run_main(DEAD_STATE), 2)

    def test_a_create_whose_issue_does_not_read_back_exits_2(self):
        cll._gh = FakeGh(read_back_as=-1)
        self.assertEqual(self.run_main(DEAD_STATE), 2)

    def test_a_create_that_reads_back_is_reported_as_filed(self):
        """The guard above must not have made every successful file fail."""
        fake = FakeGh()
        cll._gh = fake
        self.assertEqual(self.run_main(DEAD_STATE), 0)
        self.assertEqual(fake.kinds().count("issue create"), 1)
        self.assertTrue(any(c[:2] == ["gh", "api"]
                            and re.fullmatch(r"repos/o/r/issues/\d+", c[2])
                            for c in fake.calls),
                        "the created issue was never read back")


class TestGhErrorsBecomeAlarmErrors(unittest.TestCase):
    """The load-bearing link of the exit-0 design, exercised on the REAL `_gh`
    rather than on the fake that raises AlarmError itself."""

    def test_a_nonzero_exit_becomes_an_AlarmError(self):
        with self.assertRaises(cll.AlarmError):
            cll._gh([sys.executable, "-c",
                     "import sys; sys.stderr.write('boom\\n'); sys.exit(3)"])

    def test_the_stderr_of_the_failing_command_is_carried_into_the_error(self):
        with self.assertRaises(cll.AlarmError) as ctx:
            cll._gh([sys.executable, "-c",
                     "import sys; sys.stderr.write('secondary rate limit\\n');"
                     " sys.exit(1)"])
        self.assertIn("secondary rate limit", str(ctx.exception))

    def test_an_unlaunchable_binary_becomes_an_AlarmError(self):
        missing = Path(tempfile.mkdtemp()) / "no-such-binary-4328"
        with self.assertRaises(cll.AlarmError):
            cll._gh([str(missing)])

    def test_a_command_that_succeeds_returns_its_stdout(self):
        """The conversion must not have swallowed the success path."""
        self.assertEqual(
            cll._gh([sys.executable, "-c", "print('ok')"]).strip(), "ok")


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

    def test_pagination_returning_nothing_against_a_nonzero_total_is_fail_loud(self):
        real = cll._gh_json
        try:
            cll._gh_json = lambda path: {"total_count": 50, "workflow_runs": []}
            with self.assertRaises(cll.AlarmError):
                cll.fetch_scheduled_runs("o/r", "z.yml")
        finally:
            cll._gh_json = real

    def test_pagination_short_of_total_count_is_fail_loud(self):
        """A page cap silently truncating a listing has caused wrong conclusions
        here before: a SHORT result against a larger total_count must red, not
        quietly become the answer."""
        pages = {1: [{"conclusion": "success", "created_at": "2026-07-0"
                                                             f"{d}T00:00:00Z"}
                     for d in range(1, 10)]}

        real = cll._gh_json
        try:
            cll._gh_json = lambda path: {
                "total_count": 50,
                "workflow_runs": pages.get(
                    int(re.search(r"[?&]page=(\d+)", path).group(1)), [])}
            with self.assertRaises(cll.AlarmError) as ctx:
                cll.fetch_scheduled_runs("o/r", "z.yml")
        finally:
            cll._gh_json = real
        self.assertIn("total_count=50", str(ctx.exception))

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


# --------------------------------------------------------------------------- #
# The SWALLOW class. `if: false` and a deleted step change what the log shows;
# these mutations keep the step, keep the log, and delete the enforcement:
#
#   continue-on-error: true   (job or step)   the step "passes" while failing
#   `… || true` / `; exit 0` / `set +e`       the shell discards the exit status
#   `--dry-run` on the detector               EVERY observable is identical —
#                                             the summary renders, the ::warning::
#                                             is emitted, exit is 0 — and no issue
#                                             is ever filed. On a cron-only lane
#                                             there is, by this alarm's own
#                                             premise, nobody to notice.
#
# Two INDEPENDENT channels, either sufficient, so no single assertion is the
# whole guard: (1) key ABSENCE — `continue-on-error: 'true'` and `${{ true }}`
# both load as STRINGS, so `!= True` would pass; (2) the detector's argv by
# EQUALITY after shlex, not containment, because `--apply-DROPPED`-shaped
# mutants survive containment checks. Asserted at BOTH seams: cron-liveness.yml
# and the docs-quality.yml call site, which is a GATING check.
SWALLOW_IN_RUN = re.compile(
    r"""\|\|\s*(true|:|exit\b)      # || true / || : / || exit 0
      | ;\s*exit\s+0                # ; exit 0
      | &&\s*true\b                 # && true
      | \bset\s+\+e                 # set +e  (and set +eo pipefail)
      | \bexit\s+0\s*$              # a trailing unconditional exit 0
    """, re.VERBOSE | re.MULTILINE)

DETECTOR_ARGV = ["python3", "scripts/cron_lane_liveness.py",
                 "--repo", "$GITHUB_REPOSITORY",
                 "--summary-file", "$GITHUB_STEP_SUMMARY"]
SELFTEST_ARGV = ["python3", "scripts/tests/test_cron_lane_liveness.py"]


def argv_of(run: str) -> list[str]:
    """shlex the `run:` block, dropping the whitespace tokens a `\\`-continued
    line leaves behind. Any inserted flag, emptied value, or appended `|| true`
    changes this list."""
    return [t for t in shlex.split(run) if t.strip()]


class TestThisLaneIsDeclaredNonGating(unittest.TestCase):
    """[OPUS-5] sq-huwr8 composition, found while re-checking this branch against
    the `main` it merges into.

    This workflow is `schedule:`-only, and a scheduled run runs ON MAIN — so its
    check-run lands on exactly the main head SHA that the PUSH-triggered
    `ci-summary` gate polls, and that gate waits for the check-run name set to
    stabilise, so a sibling appearing mid-poll is picked up rather than missed.
    Since #3773 the ONLY thing that makes a check-run non-gating is a
    declaration in `.github/advisory-registry.json` — the header claim "NOT A
    GATE: schedule-only" is TRUE as a premise and does NOT imply the conclusion.
    MEASURED on main head cb0c6739c: the identically-shaped
    `review-lane blind-spot alarm: failure` was listed as a GATING failure.

    Undeclared, the one red this lane can produce — exit 2, a broken detector —
    would block every merge in the repo on a condition unrelated to any commit.
    That is the pressure that gets alarms muted.
    """

    REGISTRY = REPO_ROOT / ".github" / "advisory-registry.json"

    @classmethod
    def setUpClass(cls):
        sys.path.insert(0, str(REPO_ROOT / "scripts"))
        import ci_summary_gate  # noqa: PLC0415 — imported for its REAL classifier
        cls.gate = ci_summary_gate
        cls.job = yaml.safe_load(
            WORKFLOW.read_text(encoding="utf-8"))["jobs"]["liveness"]
        cls.registry = json.loads(cls.REGISTRY.read_text(encoding="utf-8"))

    def setUp(self):
        # Install the LIVE registry into the gate module exactly as the gate's
        # own main() does. The module fails CLOSED (nothing advisory) until this
        # runs, so the assertions below are about the file, not the default.
        self.gate.load_advisory_registry(str(self.REGISTRY))

    def test_the_live_job_name_is_declared_non_gating(self):
        # Drives the gate's OWN is_advisory(), never a re-implementation of it,
        # so this cannot agree with a broken gate.
        self.assertTrue(
            self.gate.is_advisory(self.job["name"]),
            f"{self.job['name']!r} is NOT declared in "
            ".github/advisory-registry.json, so ci_summary_gate.py GATES on it "
            "(#3773) — an exit-2 run of this schedule-on-main lane would block "
            "every merge in the repo")

    def test_control_an_undeclared_name_is_NOT_non_gating(self):
        """ANTI-VACUITY: an is_advisory() that returned True unconditionally
        would pass the test above. It must not pass this one."""
        self.assertFalse(self.gate.is_advisory(
            self.job["name"] + " (this name is not declared anywhere)"))

    def test_the_declaration_binds_to_THIS_workflow_and_job_id(self):
        """C4's rename-invariance from this side: the declaration must name the
        stable identity, so renaming the job cannot silently re-arm the lane as
        a merge blocker (it fails closed — the renamed job GATES)."""
        entry = self.registry["jobs"][self.job["name"]]
        self.assertEqual((entry["workflow"], entry["job_id"]),
                         ("cron-liveness.yml", "liveness"))


class TestNoSwallowAtEitherSeam(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.doc = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))
        cls.job = cls.doc["jobs"]["liveness"]
        cls.steps = cls.job["steps"]
        cls.dq = yaml.safe_load(DOCS_QUALITY.read_text(encoding="utf-8"))
        hits = [(jid, job, s) for jid, job in cls.dq["jobs"].items()
                for s in (job.get("steps") or [])
                if "scripts/tests/test_cron_lane_liveness.py" in (s.get("run") or "")]
        assert len(hits) == 1, f"expected exactly one call site, got {len(hits)}"
        cls.call_job_id, cls.call_job, cls.call_step = hits[0]

    def _step(self, needle: str) -> dict:
        hits = [s for s in self.steps if needle in (s.get("run") or "")]
        self.assertEqual(len(hits), 1, needle)
        return hits[0]

    # ---- seam 1: cron-liveness.yml ----------------------------------------- #
    def test_the_liveness_job_carries_no_continue_on_error_key(self):
        self.assertNotIn("continue-on-error", self.job)

    def test_no_step_of_the_liveness_job_carries_continue_on_error(self):
        for s in self.steps:
            self.assertNotIn("continue-on-error", s, s.get("name") or s.get("uses"))

    def test_the_detector_invocation_is_EXACTLY_the_live_form(self):
        """Kills `--dry-run` (deletes the feature, changes nothing observable),
        `--repo ""`, and any appended swallow — by EQUALITY, not containment."""
        self.assertEqual(argv_of(self._step("scripts/cron_lane_liveness.py")["run"]),
                         DETECTOR_ARGV)

    def test_the_selftest_invocation_is_EXACTLY_the_live_form(self):
        self.assertEqual(
            argv_of(self._step("scripts/tests/test_cron_lane_liveness.py")["run"]),
            SELFTEST_ARGV)

    def test_neither_run_block_can_discard_a_nonzero_exit(self):
        for needle in ("scripts/cron_lane_liveness.py",
                       "scripts/tests/test_cron_lane_liveness.py"):
            run = self._step(needle)["run"]
            self.assertIsNone(SWALLOW_IN_RUN.search(run), f"{needle}: {run!r}")

    def test_the_detector_step_gets_the_token_and_the_repo(self):
        env = self._step("scripts/cron_lane_liveness.py").get("env") or {}
        self.assertEqual(env.get("GH_TOKEN"), "${{ github.token }}")
        self.assertEqual(env.get("GITHUB_REPOSITORY"), "${{ github.repository }}")

    # ---- seam 2: the docs-quality.yml call site (a GATING check) ------------ #
    def test_the_call_site_job_carries_no_continue_on_error_key(self):
        self.assertNotIn("continue-on-error", self.call_job)

    def test_the_call_site_step_carries_no_continue_on_error_key(self):
        self.assertNotIn("continue-on-error", self.call_step)

    def test_the_call_site_run_cannot_discard_a_nonzero_exit(self):
        self.assertIsNone(SWALLOW_IN_RUN.search(self.call_step["run"]),
                          repr(self.call_step["run"]))

    def test_the_call_site_invocation_is_EXACTLY_the_live_form(self):
        self.assertEqual(argv_of(self.call_step["run"]), SELFTEST_ARGV)

    def test_the_call_site_job_is_not_advisory_registered(self):
        """`docs-quality / quick-gates` must remain GATING: an entry in the
        advisory registry is the only thing that makes a check non-gating, and
        this suite is the only per-PR enforcement the alarm has."""
        registry = json.loads(
            (REPO_ROOT / ".github" / "advisory-registry.json").read_text(
                encoding="utf-8"))
        declared = {(v.get("workflow"), v.get("job_id"))
                    for v in registry.get("jobs", {}).values()}
        self.assertNotIn(("docs-quality.yml", self.call_job_id), declared)

    # ---- the detectors of the above, on a KNOWN POSITIVE -------------------- #
    def test_the_swallow_regex_matches_every_form_it_claims_to(self):
        """A zero-result regex is a clean run too. Validate the instrument."""
        for bad in ("python3 x.py || true", "python3 x.py||true",
                    "python3 x.py || :", "python3 x.py; exit 0",
                    "python3 x.py\nexit 0", "set +e\npython3 x.py",
                    "set +eo pipefail\npython3 x.py", "python3 x.py && true",
                    "python3 x.py || exit 0"):
            self.assertIsNotNone(SWALLOW_IN_RUN.search(bad), bad)
        for ok in ("python3 x.py", "set -euo pipefail\npython3 x.py",
                   "python3 x.py --repo \"$R\"\npython3 y.py"):
            self.assertIsNone(SWALLOW_IN_RUN.search(ok), ok)

    def test_argv_equality_rejects_an_appended_flag(self):
        """Validate the instrument against a known positive: the exact mutant."""
        live = self._step("scripts/cron_lane_liveness.py")["run"]
        self.assertEqual(argv_of(live), DETECTOR_ARGV)
        self.assertNotEqual(argv_of(live + " --dry-run"), DETECTOR_ARGV)
        self.assertNotEqual(argv_of(live.replace('"$GITHUB_REPOSITORY"', '""')),
                            DETECTOR_ARGV)


if __name__ == "__main__":
    unittest.main(verbosity=2)
