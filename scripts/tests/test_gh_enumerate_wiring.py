#!/usr/bin/env python3
# [OPUS-5] sparq-org/sparq#4985 — `gh <list> --limit N` truncates SILENTLY. 🤖 SPARQ agent.
#
# THE DEFECT THIS PINS. `gh pr list` / `gh issue list --limit N` return the newest N rows,
# print no warning and exit 0. Nothing in the response distinguishes "the population is N"
# from "the population is larger and was cut to N", so every caller that ENUMERATES a
# population to make a decision degrades invisibly the moment that population crosses its
# cap. Two shapes, both live in this repo:
#
#   * a WORK-QUEUE sweep (batch-merge, pr-backlog, pr-area-labels, feature-OFF
#     autodeclare) stops seeing the tail of the queue. It does not act late — it never
#     acts on the truncated tail on any tick, and reports a clean run while doing it.
#   * a DEDUPE / marker search (the three alarms, pr-backlog's marker set) stops seeing
#     the marker it exists to find. A missed marker reads as "not filed yet", so the
#     caller mints a DUPLICATE issue on every run: the non-spam invariant fails OPEN.
#
# `scripts/pr-backlog.py::collect_existing_markers` was not latent. It enumerated the
# UNFILTERED open-issue set at `--limit 500`; per #4985's live count on 2026-09-01 the
# repo carried 1707 open issues, so ~1200 issues' markers were invisible and the groomer
# re-filed already-open issues. That count is POINT-IN-TIME: the defect is
# depth-conditional and bites only while the population exceeds the cap.
#
# HOW THIS SUITE IS BUILT TO FAIL.
#   * Every behaviour test drives the REAL shipped function (`open_issue_exists`,
#     `collect_existing_markers`, `fetch_open_prs`, ...) through its own runner seam. The
#     truncation rule is never re-implemented here, so this file cannot agree with a
#     broken call site.
#   * Each site is pinned THREE ways: the happy path still finds its marker/rows (so a
#     guard that raised unconditionally would fail); a ceiling-sized response RAISES (the
#     fail-open -> fail-closed flip); and the argv actually carries the ceiling (so a site
#     reverted to `--limit 100` reds even though the guard object is still imported).
#   * test_control_* is the ANTI-VACUITY control: it asserts the stub harness itself can
#     observe a NON-raising under-ceiling fetch. If the control ever goes green while the
#     boundary tests also pass trivially, this file is measuring nothing.

from __future__ import annotations

import importlib.util
import json
import re
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SCRIPTS = REPO_ROOT / "scripts"
sys.path.insert(0, str(SCRIPTS))

import gh_enumerate  # noqa: E402


def _load(name: str, filename: str):
    path = SCRIPTS / filename
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader, path
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


FORMAL = _load("formal_lane_alarm", "formal_lane_alarm.py")
HEAVY = _load("heavy_set_alarm", "heavy_set_alarm.py")
SELECTION = _load("ci_selection_alarm", "ci_selection_alarm.py")
BACKLOG = _load("pr_backlog", "pr-backlog.py")
AREA = _load("pr_area_labels_wiring", "pr-area-labels.py")

REPO = "sparq-org/sparq"


def _limit_of(argv) -> int | None:
    """The integer following `--limit` in a captured argv, or None."""
    argv = list(argv)
    if "--limit" not in argv:
        return None
    return int(argv[argv.index("--limit") + 1])


class TestCeilingIsSanelySized(unittest.TestCase):
    """The ceilings must clear the live populations, or the sweep trades a silent
    truncation for a standing red — a strictly worse outcome, since a lane that reds
    every run is a lane somebody mutes."""

    def test_pr_ceiling_far_exceeds_the_old_caps(self):
        # Old caps were 200 (batch-merge, pr-backlog, feature-OFF) and 300 (pr-area).
        self.assertGreater(gh_enumerate.PR_CEILING, 300)

    def test_issue_ceiling_exceeds_the_live_open_issue_population(self):
        # ~1707 open issues at the time of writing; ISSUE_CEILING is shared with
        # ready-issues.py / retriage.py / triage-area.py, which all use 10000.
        self.assertGreaterEqual(gh_enumerate.ISSUE_CEILING, 10000)


class TestFormalLaneAlarmDedupe(unittest.TestCase):
    """formal_lane_alarm.open_issue_exists — the dedupe guard, was `--limit 100`."""

    def setUp(self):
        self.calls = []
        self._real = FORMAL._run_gh

    def tearDown(self):
        FORMAL._run_gh = self._real

    def _stub(self, rows):
        def run(argv):
            self.calls.append(argv)
            return json.dumps(rows)
        FORMAL._run_gh = run

    def test_control_under_ceiling_finds_an_existing_marker(self):
        marker = f"{FORMAL.KEY_PREFIX}: kani.yml"
        self._stub([{"number": 1, "body": f"blah\n{marker}\n"}])
        self.assertTrue(FORMAL.open_issue_exists(REPO, "kani.yml"))

    def test_control_under_ceiling_reports_absence(self):
        self._stub([{"number": 1, "body": "unrelated"}])
        self.assertFalse(FORMAL.open_issue_exists(REPO, "kani.yml"))

    def test_argv_carries_the_shared_issue_ceiling(self):
        self._stub([])
        FORMAL.open_issue_exists(REPO, "kani.yml")
        self.assertEqual(_limit_of(self.calls[0]), gh_enumerate.ISSUE_CEILING)

    def test_ceiling_sized_response_raises_instead_of_filing_a_duplicate(self):
        # The fail-OPEN case: the marker is past the cut, so the old code returned False
        # and file_issue() minted a duplicate. It must now refuse the run.
        self._stub([{"number": n, "body": "x"} for n in range(gh_enumerate.ISSUE_CEILING)])
        with self.assertRaises(FORMAL.AlarmError):
            FORMAL.open_issue_exists(REPO, "kani.yml")


class TestHeavySetAlarmDedupe(unittest.TestCase):
    """heavy_set_alarm.open_issue_exists — a verbatim copy of the formal-lane dedupe."""

    def setUp(self):
        self.calls = []
        self._real = HEAVY._run_gh

    def tearDown(self):
        HEAVY._run_gh = self._real

    def _stub(self, rows):
        def run(argv):
            self.calls.append(argv)
            return json.dumps(rows)
        HEAVY._run_gh = run

    def test_control_under_ceiling_finds_an_existing_marker(self):
        marker = f"{HEAVY.KEY_PREFIX}: {HEAVY.ISSUE_KEY}"
        self._stub([{"number": 1, "body": marker}])
        self.assertTrue(HEAVY.open_issue_exists(REPO))

    def test_argv_carries_the_shared_issue_ceiling(self):
        self._stub([])
        HEAVY.open_issue_exists(REPO)
        self.assertEqual(_limit_of(self.calls[0]), gh_enumerate.ISSUE_CEILING)

    def test_ceiling_sized_response_raises_instead_of_filing_a_duplicate(self):
        self._stub([{"number": n, "body": "x"} for n in range(gh_enumerate.ISSUE_CEILING)])
        with self.assertRaises(HEAVY.AlarmError):
            HEAVY.open_issue_exists(REPO)


class TestSelectionAlarmDedupe(unittest.TestCase):
    """ci_selection_alarm.open_issue_exists — was `--limit 100` AND fail-open on an
    unreadable response (`except json.JSONDecodeError: return False`)."""

    def setUp(self):
        self.calls = []
        self._real = SELECTION._run

    def tearDown(self):
        SELECTION._run = self._real

    def _stub(self, payload):
        def run(cmd, check=True):
            self.calls.append(cmd)
            return payload if isinstance(payload, str) else json.dumps(payload)
        SELECTION._run = run

    def test_control_under_ceiling_finds_an_existing_marker(self):
        key = "ci.yml/build (feature-off)"
        self._stub([{"number": 7, "body": SELECTION.key_marker(key)}])
        self.assertTrue(SELECTION.open_issue_exists(key, REPO))

    def test_control_under_ceiling_reports_absence(self):
        self._stub([{"number": 7, "body": "unrelated"}])
        self.assertFalse(SELECTION.open_issue_exists("some/key", REPO))

    def test_argv_carries_the_shared_issue_ceiling(self):
        self._stub([])
        SELECTION.open_issue_exists("some/key", REPO)
        self.assertEqual(_limit_of(self.calls[0]), gh_enumerate.ISSUE_CEILING)

    def test_ceiling_sized_response_raises_instead_of_filing_a_duplicate(self):
        self._stub([{"number": n, "body": "x"} for n in range(gh_enumerate.ISSUE_CEILING)])
        with self.assertRaises(SELECTION.AlarmError):
            SELECTION.open_issue_exists("some/key", REPO)

    def test_unreadable_response_refuses_rather_than_claiming_nothing_is_open(self):
        # Previously `return False`, which sent a broken dedupe straight to create_issue().
        self._stub("not json at all")
        with self.assertRaises(SELECTION.AlarmError):
            SELECTION.open_issue_exists("some/key", REPO)


class TestPrBacklogMarkerDedupe(unittest.TestCase):
    """pr-backlog.collect_existing_markers — the site that was ACTIVELY truncating:
    `--limit 500` over ~1707 unfiltered open issues."""

    def test_control_under_ceiling_collects_markers(self):
        rows = [{"body": "<!-- pr-backlog:stale-42 -->"}, {"body": "no marker"}]
        markers = BACKLOG.collect_existing_markers(REPO, gh_json=lambda argv: rows)
        self.assertIn("<!-- pr-backlog:stale-42 -->", markers)

    def test_argv_carries_the_shared_issue_ceiling(self):
        seen = []

        def gh_json(argv):
            seen.append(argv)
            return []

        BACKLOG.collect_existing_markers(REPO, gh_json=gh_json)
        self.assertEqual(_limit_of(seen[0]), gh_enumerate.ISSUE_CEILING)

    def test_ceiling_sized_response_refuses_rather_than_half_the_markers(self):
        rows = [{"body": "x"} for _ in range(gh_enumerate.ISSUE_CEILING)]
        with self.assertRaises(SystemExit):
            BACKLOG.collect_existing_markers(REPO, gh_json=lambda argv: rows)


class TestPrBacklogOpenPrs(unittest.TestCase):
    """pr-backlog.collect_prs — was `--limit 200` against 93 live open PRs."""

    def test_control_under_ceiling_returns_the_prs(self):
        rows = [{"number": 5, "title": "t", "author": {"login": "a"}, "isDraft": False,
                 "headRefName": "h", "labels": [], "updatedAt": "2026-01-01T00:00:00Z"}]
        got = BACKLOG.collect_prs(REPO, gh_json=lambda argv: rows)
        self.assertEqual([p["number"] for p in got], [5])

    def test_argv_carries_the_pr_ceiling(self):
        seen = []

        def gh_json(argv):
            seen.append(argv)
            return []

        BACKLOG.collect_prs(REPO, gh_json=gh_json)
        self.assertEqual(_limit_of(seen[0]), gh_enumerate.PR_CEILING)

    def test_ceiling_sized_response_refuses_rather_than_grooming_a_partial_queue(self):
        rows = [{"number": n, "title": "", "author": {}, "isDraft": False,
                 "headRefName": "", "labels": [], "updatedAt": None}
                for n in range(gh_enumerate.PR_CEILING)]
        with self.assertRaises(SystemExit):
            BACKLOG.collect_prs(REPO, gh_json=lambda argv: rows)


class TestPrAreaLabelsBackfill(unittest.TestCase):
    """pr-area-labels.fetch_open_prs — `limit` is now a fail-closed ceiling, and the CLI
    default moved off 300."""

    def test_fetch_default_limit_is_the_pr_ceiling(self):
        import inspect
        sig = inspect.signature(AREA.fetch_open_prs)
        self.assertEqual(sig.parameters["limit"].default, gh_enumerate.PR_CEILING)

    def test_cli_limit_default_is_the_pr_ceiling_not_300(self):
        # The `--limit` flag is built inline in main(), so pin the argparse default at the
        # source level: a revert to `default=300` must red here.
        src = (SCRIPTS / "pr-area-labels.py").read_text()
        decl = re.search(r'add_argument\(\s*"--limit".*?\)\n', src, re.S)
        self.assertIsNotNone(decl, "pr-area-labels.py no longer declares --limit")
        self.assertIn("default=gh_enumerate.PR_CEILING", decl.group(0))

    def test_control_under_ceiling_returns_the_prs(self):
        rows = [{"number": 5, "labels": [], "changedFiles": 0, "isDraft": False,
                 "title": "t"}]
        calls = []

        def runner(argv, **_kw):
            calls.append(argv)
            # fetch_open_prs enriches each PR with a REST file enumeration; that second
            # call must not be answered with the PR list.
            return "" if "api" in argv else json.dumps(rows)

        got = AREA.fetch_open_prs(REPO, limit=50, runner=runner)
        self.assertEqual([p["number"] for p in got], [5])
        self.assertEqual(_limit_of(calls[0]), 50)

    def test_ceiling_sized_response_refuses_rather_than_labelling_a_partial_set(self):
        rows = [{"number": n, "labels": [], "changedFiles": 0, "isDraft": False,
                 "title": ""} for n in range(50)]

        def runner(argv, **_kw):
            return json.dumps(rows)

        with self.assertRaises(SystemExit):
            AREA.fetch_open_prs(REPO, limit=50, runner=runner)


class TestNoCappedEnumerationRemains(unittest.TestCase):
    """A site reverted to a hard-coded small `--limit` must RED even though the module is
    still imported for a sibling call. Pins the argv-building line itself."""

    SWEPT = {
        "batch-merge.py": ["pr\", \"list\""],
        "pr-backlog.py": ["pr\", \"list\"", "issue\", \"list\""],
        "pr-area-labels.py": ["pr\", \"list\""],
        "feature_off_autodeclare.py": ["pr list"],
        "ci_selection_alarm.py": ["issue\", \"list\""],
        "formal_lane_alarm.py": ["issue\", \"list\""],
        "heavy_set_alarm.py": ["issue\", \"list\""],
    }

    def test_every_swept_script_imports_the_shared_ceiling(self):
        for name in self.SWEPT:
            with self.subTest(script=name):
                src = (SCRIPTS / name).read_text()
                self.assertIn("import gh_enumerate", src)
                self.assertIn("gh_enumerate.limit_args(", src)

    def test_no_swept_script_hard_codes_the_old_enumeration_caps(self):
        # The old caps, as they appeared in argv: 100 / 200 / 300 / 500.
        old = ['"--limit", "100"', '"--limit", "200"',
               '"--limit", "300"', '"--limit", "500"']
        for name in self.SWEPT:
            src = (SCRIPTS / name).read_text()
            for literal in old:
                with self.subTest(script=name, literal=literal):
                    self.assertNotIn(literal, src)


class TestPromoteOnApprovalInlineSite(unittest.TestCase):
    """`.github/workflows/promote-on-approval.yml` enumerates open `trust:untrusted`
    issues from an INLINE heredoc, so it has no importable seam and no self-test of its
    own. Pin it structurally: the embedded script must parse, and must enumerate through
    the shared ceiling rather than a hard-coded cap. This poll is the only route out of
    quarantine, so a truncated fetch strands the tail permanently."""

    WORKFLOW = REPO_ROOT / ".github" / "workflows" / "promote-on-approval.yml"

    def _embedded(self) -> str:
        src = self.WORKFLOW.read_text()
        m = re.search(r"python3 - <<'PY'\n(.*?)\n\s*PY\n", src, re.S)
        self.assertIsNotNone(m, "the inline python heredoc is gone")
        return "\n".join(line[10:] if line.startswith(" " * 10) else line
                         for line in m.group(1).split("\n"))

    def test_embedded_script_is_valid_python(self):
        import ast
        ast.parse(self._embedded())

    def test_embedded_script_enumerates_through_the_shared_ceiling(self):
        body = self._embedded()
        self.assertIn("import gh_enumerate", body)
        self.assertIn("gh_enumerate.limit_args(gh_enumerate.ISSUE_CEILING)", body)
        self.assertIn("gh_enumerate.guard(", body)

    def test_embedded_script_does_not_hard_code_the_old_cap(self):
        self.assertNotIn('"--limit", "200"', self._embedded())

    def test_this_lane_runs_when_that_workflow_changes(self):
        # A structural pin cannot fire if the lane is not triggered by the file it pins.
        routing = (REPO_ROOT / ".github" / "workflows" / "routing-self-tests.yml").read_text()
        self.assertIn('".github/workflows/promote-on-approval.yml"', routing)


class TestThisSuiteIsActuallyInvoked(unittest.TestCase):
    """A suite nobody runs cannot go red. Pin this file's own call site, the same way
    test_pr_area_labels.py pins its own."""

    def test_routing_self_tests_invokes_this_suite_and_the_module_self_test(self):
        routing = (REPO_ROOT / ".github" / "workflows" / "routing-self-tests.yml").read_text()
        self.assertIn("python3 scripts/tests/test_gh_enumerate_wiring.py", routing)
        self.assertIn("python3 scripts/gh_enumerate.py --self-test", routing)


if __name__ == "__main__":
    unittest.main(verbosity=2)
