#!/usr/bin/env python3
"""Suite for scripts/ci_merge_group_poles.py. 🤖 SPARQ agent. Issue #5250.

Hermetic: no gh, no network, no git, stdlib-only (no PyYAML). Every case injects run
payloads.

The defect this profiler exists to prevent is a SIZING mistake — "the long lane must
be the pole" — so the load-bearing cases are the ones where duration and pole-ness
DISAGREE:

  * a long lane that starts early and ends mid-pack is NOT the pole (test_closer_is_
    last_to_end_not_longest_duration). This is exactly the ~20-40min CodeQL story:
    an expensive-sounding lane that does not set the entry wall.
  * a closer whose runner-up ends a second behind owns ~no exclusive tail, so
    deleting it saves ~nothing (test_exclusive_tail_is_zero_for_a_co_pole). Ranking
    on close-count alone would have called that a win.

Plus the population rules — an entry is dropped when it is incomplete, unclean or
single-run, and every drop is counted — because a silently narrowed population is
how a measurement lies while looking healthy.
"""

from __future__ import annotations

import datetime as dt
import importlib.util
import re
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "ci_merge_group_poles.py"
DOCS_QUALITY = REPO_ROOT / ".github" / "workflows" / "docs-quality.yml"

_spec = importlib.util.spec_from_file_location("ci_merge_group_poles", SCRIPT)
poles_mod = importlib.util.module_from_spec(_spec)
sys.modules["ci_merge_group_poles"] = poles_mod
_spec.loader.exec_module(poles_mod)

T0 = dt.datetime(2026, 8, 1, 12, 0, tzinfo=dt.timezone.utc)


def _stamp(minutes: float) -> str:
    return (T0 + dt.timedelta(minutes=minutes)).isoformat().replace("+00:00", "Z")


def _run(workflow: str, *, sha: str = "aaa", created: float = 0.0, ended: float = 5.0,
         status: str = "completed", conclusion: str = "success", attempt: int = 1) -> dict:
    return {
        "path": f".github/workflows/{workflow}",
        "name": workflow.removesuffix(".yml"),
        "head_sha": sha,
        "status": status,
        "conclusion": conclusion,
        "run_attempt": attempt,
        "created_at": _stamp(created),
        "run_started_at": _stamp(created + 0.5),
        "updated_at": _stamp(ended),
    }


class TestEntryGrouping(unittest.TestCase):
    def test_closer_is_last_to_end_not_longest_duration(self):
        """The pole is whoever ENDS last. A lane that runs longer but starts earlier
        and finishes first is not on the critical path."""
        entries, _ = poles_mod.group_entries([
            _run("ci.yml", created=0, ended=14),       # 14 min long, ends FIRST
            _run("codeql.yml", created=12, ended=16),  # 4 min short, ends LAST
        ])
        self.assertEqual(len(entries), 1)
        self.assertEqual(entries[0]["closer"], "codeql.yml")
        # and the long lane is still reported with its honest duration
        durations = {r["workflow"]: r["duration_s"] for r in entries[0]["runs"]}
        self.assertEqual(durations["ci.yml"], 14 * 60)
        self.assertEqual(durations["codeql.yml"], 4 * 60)

    def test_entry_wall_counts_queue_time_from_created_at(self):
        """`created_at`, not `run_started_at` — a run that waited 3 minutes for a
        runner cost the entry those 3 minutes."""
        entries, _ = poles_mod.group_entries([
            _run("ci.yml", created=0, ended=10),
            _run("bench.yml", created=1, ended=8),
        ])
        self.assertEqual(entries[0]["wall_s"], 10 * 60)

    def test_exclusive_tail_is_zero_for_a_co_pole(self):
        entries, _ = poles_mod.group_entries([
            _run("ci.yml", created=0, ended=10),
            _run("bench.yml", created=0, ended=10),
        ])
        self.assertEqual(entries[0]["tail_s"], 0.0)

    def test_exclusive_tail_is_the_gap_to_the_runner_up(self):
        entries, _ = poles_mod.group_entries([
            _run("ci.yml", created=0, ended=15),
            _run("bench.yml", created=0, ended=4),
            _run("js.yml", created=0, ended=6),
        ])
        self.assertEqual(entries[0]["closer"], "ci.yml")
        self.assertEqual(entries[0]["tail_s"], 9 * 60)  # 15 - 6, not 15 - 4

    def test_reruns_collapse_to_the_latest_attempt(self):
        runs = [
            _run("ci.yml", created=0, ended=10, attempt=1),
            _run("ci.yml", created=20, ended=30, attempt=2),
            _run("js.yml", created=0, ended=6),
        ]
        entries, _ = poles_mod.group_entries(runs)
        names = [r["workflow"] for r in entries[0]["runs"]]
        self.assertEqual(sorted(names), ["ci.yml", "js.yml"])
        self.assertEqual(len(names), 2, "the re-run must not be counted as a second lane")
        self.assertEqual(entries[0]["closer"], "ci.yml")

    def test_workflow_identity_is_the_path_not_the_display_name(self):
        a = _run("ci.yml")
        a["name"] = "CI"
        b = _run("ci.yml")
        b["name"] = "CI (renamed)"
        b["head_sha"] = "bbb"
        self.assertEqual(poles_mod.workflow_name(a), poles_mod.workflow_name(b))


class TestPopulationRules(unittest.TestCase):
    def test_incomplete_entry_is_dropped_and_counted(self):
        entries, census = poles_mod.group_entries([
            _run("ci.yml", ended=10),
            _run("bench.yml", status="in_progress", conclusion=None, ended=3),
        ])
        self.assertEqual(entries, [])
        self.assertEqual(census["dropped_incomplete"], 1)
        self.assertEqual(census["entries_seen"], 1)

    def test_unclean_entry_is_dropped_by_default_kept_on_demand(self):
        runs = [_run("ci.yml", ended=10),
                _run("bench.yml", conclusion="cancelled", ended=3)]
        entries, census = poles_mod.group_entries(runs)
        self.assertEqual(entries, [])
        self.assertEqual(census["dropped_unclean"], 1)
        kept, _ = poles_mod.group_entries(runs, include_unclean=True)
        self.assertEqual(len(kept), 1)

    def test_skipped_and_neutral_conclusions_are_clean(self):
        entries, _ = poles_mod.group_entries([
            _run("ci.yml", ended=10, conclusion="skipped"),
            _run("bench.yml", ended=3, conclusion="neutral"),
        ])
        self.assertEqual(len(entries), 1, "a selection-skipped lane is normal, not unclean")

    def test_single_run_entry_is_dropped(self):
        entries, census = poles_mod.group_entries([_run("ci.yml", ended=10)])
        self.assertEqual(entries, [])
        self.assertEqual(census["dropped_single_run"], 1)


class TestRanking(unittest.TestCase):
    def _population(self):
        runs = []
        for i, sha in enumerate(("s1", "s2", "s3")):
            runs += [_run("ci.yml", sha=sha, created=0, ended=15),
                     _run("codeql.yml", sha=sha, created=0, ended=4),
                     _run("bench.yml", sha=sha, created=0, ended=12)]
        # one entry where a rare lane closes with a huge tail
        runs += [_run("ci.yml", sha="s4", created=0, ended=15),
                 _run("codeql.yml", sha="s4", created=0, ended=4),
                 _run("fuzz.yml", sha="s4", created=0, ended=40)]
        return runs

    def test_poles_rank_by_close_count_then_tail(self):
        entries, _ = poles_mod.group_entries(self._population())
        ranked = poles_mod.rank_poles(entries)
        self.assertEqual([r["workflow"] for r in ranked[:2]], ["ci.yml", "fuzz.yml"])
        self.assertEqual(ranked[0]["closes"], 3)
        self.assertEqual(ranked[0]["close_rate"], 0.75)
        self.assertEqual(ranked[1]["closes"], 1)
        self.assertEqual(ranked[1]["tail_s"], 25 * 60)  # 40 - 15

    def test_a_lane_that_never_closes_carries_no_tail(self):
        entries, _ = poles_mod.group_entries(self._population())
        by_wf = {r["workflow"]: r for r in poles_mod.rank_poles(entries)}
        self.assertEqual(by_wf["codeql.yml"]["closes"], 0)
        self.assertEqual(by_wf["codeql.yml"]["tail_s"], 0.0)
        self.assertEqual(by_wf["codeql.yml"]["n"], 4, "it still ran on every entry")

    def test_report_names_the_pole_and_the_census(self):
        entries, census = poles_mod.group_entries(self._population())
        text = poles_mod.render(entries, poles_mod.rank_poles(entries), census, 3)
        self.assertIn("ci.yml", text)
        self.assertIn("entries profiled: 4", text)
        self.assertIn("dropped:", text)

    def test_pole_table_is_not_padded_with_lanes_that_never_close(self):
        """Only two lanes ever close in this population, so a `--top 3` report must
        show two rows — padding it with a never-closing lane would print exactly the
        expensive-lane-must-be-the-pole misreading this profiler exists to prevent."""
        entries, census = poles_mod.group_entries(self._population())
        pole_block = poles_mod.render(
            entries, poles_mod.rank_poles(entries), census, 3
        ).split("ALL LANES")[0]
        self.assertIn("TOP 2 POLES", pole_block)
        self.assertNotIn("codeql.yml", pole_block)
        # …and it is still listed in the duration table below, with its real numbers.
        full = poles_mod.render(entries, poles_mod.rank_poles(entries), census, 3)
        self.assertIn("codeql.yml", full.split("ALL LANES")[1])


class TestFailLoud(unittest.TestCase):
    def test_empty_window_exits_nonzero_rather_than_reporting_health(self):
        original = poles_mod.fetch_merge_group_runs
        poles_mod.fetch_merge_group_runs = lambda repo, since: []
        try:
            code = poles_mod.main(["--repo", "o/n", "--since", "2026-07-01"])
        finally:
            poles_mod.fetch_merge_group_runs = original
        self.assertEqual(code, 3)

    def test_infrastructure_failure_exits_two(self):
        original = poles_mod.fetch_merge_group_runs

        def boom(repo, since):
            raise poles_mod.PoleError("gh exploded")

        poles_mod.fetch_merge_group_runs = boom
        try:
            code = poles_mod.main(["--repo", "o/n"])
        finally:
            poles_mod.fetch_merge_group_runs = original
        self.assertEqual(code, 2)

    def test_unparseable_timestamp_raises_rather_than_defaulting(self):
        bad = _run("ci.yml")
        bad["updated_at"] = "not-a-timestamp"
        with self.assertRaises(poles_mod.PoleError):
            poles_mod.group_entries([bad, _run("js.yml")])


class TestWiring(unittest.TestCase):
    def test_suite_runs_in_docs_quality(self):
        """A test nothing invokes is not a gate. Matched as a `run:` step — a mention
        in a comment must not satisfy this, which a containment check would allow."""
        step = re.compile(
            r"^\s*run:\s*python3 scripts/tests/test_ci_merge_group_poles\.py\s*$",
            re.MULTILINE)
        self.assertRegex(DOCS_QUALITY.read_text(), step,
                         "docs-quality.yml must invoke this suite as a run: step")


if __name__ == "__main__":
    unittest.main(verbosity=2)
