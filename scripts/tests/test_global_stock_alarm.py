#!/usr/bin/env python3
# [OPUS-5] Tests for the `__global__` stock alarm (scripts/global_stock_alarm.py +
# .github/workflows/global-stock-alarm.yml), sparq-org/sparq#5127. 🤖 SPARQ agent.
#
# TWO HALVES, and the second is the load-bearing one.
#
# 1. BEHAVIOUR — the obligations a STOCK detector must meet, each a NAMED test that goes
#    red on deletion OR inversion of the guard it covers:
#      * a PR with no attribution anywhere is the finding (the thing being detected),
#      * a PR rescued by its provenance-linked OPEN source issue is NOT alarmed — the
#        cross-check #5127 asks for, and the one whose absence would make this lane fire
#        on most of the fleet,
#      * a draft / a human-parked PR is out of scope, because neither holds the partition,
#      * ONE sighting never alarms and TWO consecutive sightings do — the persistence
#        property that is the whole reason this is not a per-run status check,
#      * a PR that leaves the stock LOSES its streak, so a flickering PR can never
#        accumulate its way to an alarm,
#      * the memory advances on a RED tick (state is written before the verdict), and a
#        memory that silently stopped persisting is fail-LOUD rather than a clean board,
#      * an UNUSABLE restore (corrupt, or written for another repository) is fail-loud
#        for ONE tick and no more: the save step re-caches whatever is left at the state
#        path, so a tick that exited leaving the poison there would make every later
#        sweep restore it and die identically — which is why that one is a TWO-tick test.
#
# 2. THE YAML SEAM — measured in this repo, every uncaught mutant of an 18-mutant run
#    lived in a workflow `if:` / step / call-site, not in the Python. This lane's memory
#    lives ENTIRELY in that seam: the restore and save steps must agree on one cache path
#    and one key prefix, and the save must run on `always()`. Each of those is a silent
#    regression — break any one and the detector still runs green forever while never
#    being able to alarm. So they are pinned here by SHAPE:
#      * the restore key prefix and the save key are the same rolling family,
#      * restore and save name the same path, and the script is handed a state file
#        underneath it,
#      * the save step is guarded by `always()` (a stock alarm reds when it fires, and a
#        success-only save would freeze the streaks exactly then),
#      * the self-test step runs BEFORE the live step,
#      * the sparse-checkout carries ready-issues.py (the detector imports it; omitting
#        it is an ENOENT that no reviewer sees),
#      * the job holds `issues: write` and NOT `pull-requests: write` / `contents: write`
#        (a detector that could label the PRs it reports on would be a second, unreviewed
#        deriver),
#      * every `uses:` is SHA-pinned,
#      * docs-quality.yml wires THIS test file (drop the call site => red).
#
# The last one is the anti-vacuity anchor: without it, this whole file could leave CI's
# reachable set and nothing would notice.
#
# Stdlib + PyYAML (already a docs-quality dependency). Run:
#   python3 scripts/tests/test_global_stock_alarm.py

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import re
import tempfile
import unittest
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SCRIPT = REPO_ROOT / "scripts" / "global_stock_alarm.py"
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "global-stock-alarm.yml"
DOCS_QUALITY = REPO_ROOT / ".github" / "workflows" / "docs-quality.yml"

_spec = importlib.util.spec_from_file_location("global_stock_alarm", SCRIPT)
alarm = importlib.util.module_from_spec(_spec)
assert _spec.loader is not None
_spec.loader.exec_module(alarm)

REPO = "sparq-org/sparq"


def _pr(number, *, labels=(), draft=False, ref=None, head_repo=REPO):
    return {
        "number": number,
        "title": f"pr {number}",
        "draft": draft,
        "labels": [{"name": name} for name in labels],
        "head": {"ref": ref or f"agent/{number}", "repo": {"full_name": head_repo}},
        "user": {"login": "someone"},
        "body": "",
        "author_association": "NONE",
        "html_url": f"https://github.com/{REPO}/pull/{number}",
        "updated_at": "2026-08-01T00:00:00Z",
    }


def _issue(number, *, labels=(), state="open"):
    return {"number": number, "state": state, "labels": [{"name": n} for n in labels]}


def _classify(pr, issues=(), links=None):
    by_number = {i["number"]: i for i in issues}
    return alarm.classify_pr(pr, by_number, links or {})


# --------------------------------------------------------------------------- #
# 1. behaviour — the sweep
# --------------------------------------------------------------------------- #
class TestTheSweepFindsExactlyTheUnattributedUnits(unittest.TestCase):
    def test_a_pr_with_no_attribution_anywhere_is_the_finding(self):
        self.assertEqual(_classify(_pr(1))["state"], "unattributed")

    def test_its_own_area_label_attributes_it(self):
        self.assertEqual(
            _classify(_pr(1, labels=["area:sparq-core"]))["state"], "attributed-by-pr"
        )

    def test_a_non_area_label_attributes_nothing(self):
        """The predicate is `area:<crate>`, not "carries labels" — a PR covered in
        priority/status labels is still unattributed."""
        self.assertEqual(
            _classify(_pr(1, labels=["priority:P1", "role:impl"]))["state"], "unattributed"
        )

    def test_a_draft_is_out_of_scope(self):
        self.assertEqual(_classify(_pr(1, draft=True))["state"], "draft")

    def test_a_human_parked_pr_is_out_of_scope(self):
        """`ready-issues.py::occupies_area` releases a PR's areas on the human-hold label
        ALONE, so a parked PR is not holding the partition and alarming on it would be a
        false positive."""
        for held in sorted(alarm._ready.PARKED_AREA_LABELS):
            with self.subTest(label=held):
                self.assertEqual(_classify(_pr(1, labels=[held]))["state"], "parked")

    def test_the_census_accounts_for_every_open_pr(self):
        prs = [_pr(1), _pr(2, labels=["area:sparq-core"]), _pr(3, draft=True),
               _pr(4, labels=["needs:user"])]
        findings, census = alarm.sweep(prs, {}, {})
        self.assertEqual([f["number"] for f in findings], [1])
        self.assertEqual(sum(census.values()), len(prs))
        self.assertEqual(census["unattributed"], 1)


class TestTheSourceIssueCrossCheck(unittest.TestCase):
    """THE headline guard. Without it this lane fires on every worker PR whose crate is
    declared on the source issue rather than on the PR itself — a shape the readiness
    engine treats as attributed (`unit_reservations` reserves the UNION of the PR and the
    issues it closes). Delete the rescue branch and these go red."""

    LINKS = {6: {60}}
    PR = staticmethod(lambda: _pr(6, ref="sparq-agent/issue-60-x"))

    def test_an_open_linked_source_issue_with_an_area_rescues_the_pr(self):
        verdict = _classify(self.PR(), [_issue(60, labels=["area:sparq-zk"])], self.LINKS)
        self.assertEqual(verdict["state"], "rescued-by-issue")
        self.assertEqual(verdict["areas"], {"sparq-zk"})

    def test_a_rescued_pr_is_not_even_recorded_as_stock(self):
        findings, census = alarm.sweep(
            [self.PR()], {60: _issue(60, labels=["area:sparq-zk"])}, self.LINKS
        )
        self.assertEqual(findings, [])
        self.assertEqual(census["rescued-by-issue"], 1)

    def test_a_linked_source_issue_with_no_area_rescues_nothing(self):
        self.assertEqual(
            _classify(self.PR(), [_issue(60, labels=["priority:P1"])], self.LINKS)["state"],
            "unattributed",
        )

    def test_a_closed_source_issue_does_not_rescue(self):
        """Mirrors `_own_reservation`, which returns the empty set for any row whose
        `state` is not OPEN: a closed issue contributes nothing to its unit's areas."""
        self.assertEqual(
            _classify(self.PR(), [_issue(60, labels=["area:sparq-zk"], state="closed")],
                      self.LINKS)["state"],
            "unattributed",
        )

    def test_the_closed_source_issues_lost_areas_reach_the_finding(self):
        """…but they are REPORTED, because "the issue closed and took the attribution
        with it" is a different first move from "nothing ever declared an area"."""
        verdict = _classify(self.PR(), [_issue(60, labels=["area:sparq-zk"], state="closed")],
                            self.LINKS)
        self.assertEqual(verdict["closed_source_areas"], {"sparq-zk"})

    def test_linkage_is_the_engines_own_so_a_fork_head_links_nothing(self):
        """A fork's head-branch text is attacker-controlled. This is asserted against the
        REAL `source_issue_links`, so the detector cannot be handed a rescue by a PR that
        merely NAMES an issue in its branch."""
        fork = _pr(7, ref="sparq-agent/issue-70-x", head_repo="attacker/sparq")
        self.assertEqual(alarm._ready.source_issue_links([fork], REPO), {})
        self.assertEqual(
            alarm._ready.source_issue_links([_pr(7, ref="sparq-agent/issue-70-x")], REPO),
            {7: {70}},
        )


# --------------------------------------------------------------------------- #
# 2. behaviour — persistence across ticks
# --------------------------------------------------------------------------- #
class TestStreaksAreConsecutive(unittest.TestCase):
    def test_a_cold_tick_starts_every_streak_at_one(self):
        self.assertEqual(alarm.next_streaks({}, [1, 2]), {"1": 1, "2": 1})

    def test_a_pr_seen_again_increments(self):
        self.assertEqual(alarm.next_streaks({"1": 1}, [1]), {"1": 2})

    def test_a_pr_that_leaves_the_stock_loses_its_streak(self):
        """CONSECUTIVE, not cumulative. Retaining the old count would let a PR that is
        unattributed on alternate sweeps alarm eventually, which is not the condition."""
        self.assertEqual(alarm.next_streaks({"1": 9}, [2]), {"2": 1})

    def test_a_non_count_streak_in_state_is_fatal(self):
        with self.assertRaises(alarm.AlarmError):
            alarm.next_streaks({"1": "many"}, [1])


class TestColdMemoryIsLoud(unittest.TestCase):
    def test_the_lanes_first_ever_sweep_may_read_cold(self):
        self.assertIsNone(alarm.cold_tick_error(False, 0))

    def test_a_warm_read_is_never_an_error(self):
        self.assertIsNone(alarm.cold_tick_error(True, 12))

    def test_a_cold_read_after_a_completed_run_is_an_error(self):
        """The silent-rot case: if the cache stopped persisting, every sweep would start
        from zero, nothing could ever reach the threshold, and the stock would read as
        empty forever. Returning None here would make that indistinguishable from a
        healthy board."""
        self.assertIsNotNone(alarm.cold_tick_error(False, 1))


# --------------------------------------------------------------------------- #
# 3. behaviour — the exit code carries the verdict
# --------------------------------------------------------------------------- #
class TestExitCodeCarriesTheVerdict(unittest.TestCase):
    """Exit-zero-swallowing is the class this repository has been bitten by repeatedly:
    every assertion below is on the process exit code of a REAL main() run over hermetic
    input, not on an internal return value."""

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.snap = Path(self._tmp.name) / "snap.json"
        self.state = Path(self._tmp.name) / "state.json"

    def _snapshot(self, pulls, issues=()):
        self.snap.write_text(json.dumps({
            "pulls": pulls, "issues": {str(i["number"]): i for i in issues},
        }))

    def _invoke(self, *extra):
        with contextlib.redirect_stdout(io.StringIO()):
            return alarm.main([
                "--repo", REPO, "--snapshot-file", str(self.snap),
                "--state-file", str(self.state), "--now", "2026-08-01T12:00:00Z",
                "--dry-run", *extra,
            ])

    def _streaks(self):
        return json.loads(self.state.read_text())["streaks"]

    def test_one_sighting_does_not_alarm_and_two_running_does(self):
        """THE persistence property, end to end. #5127: "a PR that is unattributed once
        is not alarmed but one unattributed twice running is"."""
        self._snapshot([_pr(1)])
        self.assertEqual(self._invoke(), 0, "a first sighting must not alarm")
        self.assertEqual(self._streaks(), {"1": 1})
        self.assertEqual(self._invoke(), 1, "a second consecutive sighting must alarm")

    def test_the_memory_advances_on_the_RED_tick(self):
        """The state is written BEFORE the verdict is rendered. If it were written only
        on the success path, the streaks would freeze at exactly the moment they matter."""
        self._snapshot([_pr(1)])
        self._invoke()
        self.assertEqual(self._invoke(), 1)
        self.assertEqual(self._streaks(), {"1": 2}, "a red tick must still advance memory")

    def test_an_attributed_pr_clears_its_streak(self):
        self._snapshot([_pr(1)])
        self._invoke()
        self._snapshot([_pr(1, labels=["area:sparq-core"])])
        self.assertEqual(self._invoke(), 0)
        self.assertEqual(self._streaks(), {})

    def test_a_rescued_pr_never_reaches_the_threshold(self):
        self._snapshot([_pr(6, ref="sparq-agent/issue-60-x")],
                       [_issue(60, labels=["area:sparq-zk"])])
        self.assertEqual(self._invoke(), 0)
        self.assertEqual(self._invoke(), 0)
        self.assertEqual(self._streaks(), {})

    def test_min_ticks_is_honoured(self):
        self._snapshot([_pr(1)])
        self.assertEqual(self._invoke("--min-ticks", "3"), 0)
        self.assertEqual(self._invoke("--min-ticks", "3"), 0)
        self.assertEqual(self._invoke("--min-ticks", "3"), 1)

    def test_min_ticks_below_one_is_refused(self):
        """A threshold of 0 (or a negative) would alarm on the first sighting, which is
        the behaviour #5127 explicitly rules out."""
        self._snapshot([_pr(1)])
        self.assertEqual(self._invoke("--min-ticks", "0"), 2)

    def test_corrupt_state_is_fail_loud_not_a_silent_reset(self):
        """A reset lowers every streak to zero, which is indistinguishable from a healthy
        empty stock — the masking outcome. Exit 2 instead."""
        self._snapshot([_pr(1)])
        self.state.write_text("{not json")
        self.assertEqual(self._invoke(), 2)

    def test_state_from_another_repository_is_fail_loud(self):
        self._snapshot([_pr(1)])
        self.state.write_text(json.dumps({"schema": 1, "repo": "other/repo", "streaks": {}}))
        self.assertEqual(self._invoke(), 2)

    def test_an_unusable_restore_is_replaced_so_the_NEXT_tick_recovers(self):
        """TWO ticks, because one tick cannot show this. The workflow saves whatever sits
        at the state path under a newer cache key on `always()`, so a tick that exits 2
        leaving the unusable file in place would re-cache it, the next sweep would
        restore it and fail identically, and the detector would be dead permanently
        rather than for one tick. Tick 1 must go red AND leave validated state; tick 2
        must reach a verdict."""
        for poison in ("{not json",
                       json.dumps({"schema": 1, "repo": "other/repo", "streaks": {}})):
            with self.subTest(poison=poison[:20]):
                self._snapshot([_pr(1)])
                self.state.write_text(poison)
                self.assertEqual(self._invoke(), 2, "the unusable restore is still LOUD")
                self.assertEqual(self._streaks(), {"1": 1},
                                 "and the file left for the cache is this tick's own state")
                self.assertEqual(self._invoke(), 1,
                                 "so the following tick reaches a verdict, not exit 2")

    def test_a_cold_read_on_a_lane_with_completed_runs_is_fail_loud(self):
        self._snapshot([_pr(1)])
        self.assertEqual(self._invoke("--prior-runs", "4"), 2)

    def test_a_clean_population_exits_zero(self):
        self._snapshot([_pr(1, labels=["area:sparq-core"]), _pr(2, draft=True)])
        self.assertEqual(self._invoke(), 0)
        self.assertEqual(self._invoke(), 0)

    def test_the_hermetic_self_test_passes(self):
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(alarm.main(["--self-test"]), 0)


class TestTheRenderedIssue(unittest.TestCase):
    def _body(self, streaks=None):
        findings, census = alarm.sweep([_pr(1)], {}, {})
        _title, body = alarm.render_issue(
            REPO, findings, [], census, streaks or {"1": 2}, 2
        )
        return body

    def test_it_carries_the_dedupe_marker(self):
        """One open issue, keyed by this marker. Losing it turns the alarm into a
        per-sweep issue mill."""
        self.assertIn(f"<!-- {alarm.KEY_PREFIX}: {REPO} -->", self._body())

    def test_it_self_identifies_as_a_sparq_agent(self):
        self.assertTrue(self._body().startswith("> 🤖"))

    def test_it_names_the_pr_and_its_streak(self):
        body = self._body()
        self.assertIn("#1", body)
        self.assertIn("2 consecutive sweeps", body)


# --------------------------------------------------------------------------- #
# 4. the YAML seam — where this lane's memory actually lives
# --------------------------------------------------------------------------- #
class TestWorkflowSeam(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = WORKFLOW.read_text()
        cls.wf = yaml.safe_load(cls.text)
        cls.job = next(iter(cls.wf["jobs"].values()))
        cls.steps = cls.job["steps"]

    def _step(self, predicate):
        matches = [s for s in self.steps if predicate(s)]
        self.assertEqual(len(matches), 1, f"expected exactly one matching step, got {matches}")
        return matches[0]

    @property
    def restore(self):
        return self._step(lambda s: "cache/restore@" in str(s.get("uses", "")))

    @property
    def save(self):
        return self._step(lambda s: "cache/save@" in str(s.get("uses", "")))

    @property
    def sweep_step(self):
        return self._step(
            lambda s: "global_stock_alarm.py" in str(s.get("run", ""))
            and "--self-test" not in str(s.get("run", ""))
        )

    def test_it_is_scheduled(self):
        on = self.wf.get(True, self.wf.get("on"))
        self.assertIn("schedule", on)
        self.assertTrue([s["cron"] for s in on["schedule"]])

    def test_the_self_test_runs_before_the_live_sweep(self):
        order = [i for i, s in enumerate(self.steps) if "global_stock_alarm.py" in str(s.get("run", ""))]
        self_test = [i for i, s in enumerate(self.steps) if "--self-test" in str(s.get("run", ""))]
        self.assertTrue(self_test, "the hermetic self-test step is gone")
        self.assertLess(self_test[0], max(order),
                        "the detector must prove itself before it is trusted to watch")

    def test_the_live_step_hands_the_script_a_state_file_and_this_workflow(self):
        run = self.sweep_step["run"]
        self.assertIn("--state-file", run)
        self.assertIn("--workflow global-stock-alarm.yml", run,
                      "the cold-state fail-loud counts THIS lane's runs; a stale name 404s")

    def test_restore_and_save_agree_on_one_cache_path(self):
        """The silent regression this lane is most exposed to: a save that writes a path
        the restore does not read leaves the detector permanently amnesiac while every
        run stays green."""
        self.assertEqual(self.restore["with"]["path"], self.save["with"]["path"])
        self.assertIn(self.restore["with"]["path"].strip(), self.sweep_step["run"])

    def test_the_save_key_is_inside_the_restore_key_family(self):
        """`restore-keys` is a PREFIX match. If the save key stops carrying that prefix,
        nothing is ever restored — again, silently."""
        prefix = self.restore["with"]["restore-keys"].strip()
        self.assertTrue(prefix)
        self.assertTrue(
            self.save["with"]["key"].startswith(prefix),
            f"save key {self.save['with']['key']!r} is outside the restore family {prefix!r}",
        )
        self.assertTrue(self.restore["with"]["key"].startswith(prefix))

    def test_the_save_step_runs_on_always(self):
        """A persistent stock makes the sweep exit 1 and a broken detector makes it exit
        2. A save that only runs on success would drop the memory exactly on the ticks
        that matter."""
        self.assertIn("always()", str(self.save.get("if", "")))

    def test_the_save_step_is_main_scoped(self):
        """Repo cache-save doctrine (sq-3sbrr): an entry written from another ref is
        scoped to that ref and is unreachable churn."""
        self.assertIn("refs/heads/main", str(self.save.get("if", "")))

    def test_the_sparse_checkout_carries_the_readiness_engine(self):
        """The detector imports `source_issue_links` / `declared_packages` / `is_parked`
        from scripts/ready-issues.py rather than re-implementing them. Dropping it from
        the sparse-checkout is an ENOENT no reviewer sees in the diff."""
        checkout = self._step(lambda s: "actions/checkout@" in str(s.get("uses", "")))
        sparse = checkout["with"]["sparse-checkout"]
        self.assertIn("scripts/ready-issues.py", sparse)
        self.assertIn("scripts/global_stock_alarm.py", sparse)

    def test_the_detector_has_no_arming_authority(self):
        """It reports that PRs are unattributed; it must never be able to label one. That
        would be a second, unreviewed deriver competing with pr-area-label.yml."""
        perms = self.job["permissions"]
        self.assertEqual(perms.get("issues"), "write")
        self.assertNotIn("pull-requests", perms)
        self.assertNotEqual(perms.get("contents"), "write")

    def test_every_action_is_sha_pinned(self):
        for step in self.steps:
            uses = step.get("uses")
            if not uses:
                continue
            with self.subTest(uses=uses):
                self.assertRegex(uses, r"@[0-9a-f]{40}(\s|$)",
                                 "actions must be pinned to a full commit SHA")

    def test_no_step_swallows_a_failure(self):
        for step in self.steps:
            with self.subTest(step=step.get("name") or step.get("uses")):
                self.assertNotIn("continue-on-error", step)


class TestSuiteIsWiredIntoCI(unittest.TestCase):
    """Anti-vacuity anchor: without this, the whole file could leave CI's reachable set."""

    def test_docs_quality_runs_this_suite(self):
        self.assertIn(
            "scripts/tests/test_global_stock_alarm.py",
            DOCS_QUALITY.read_text(),
            "docs-quality.yml no longer invokes this suite — it would stop running",
        )

    def test_the_lane_is_declared_non_gating(self):
        """Cross-checked here as well as in test_alarm_lanes_non_gating.py, because a
        detector that REDs by design must never be able to block a merge — and the PRs it
        names are themselves open PRs."""
        registry = json.loads(
            (REPO_ROOT / ".github" / "advisory-registry.json").read_text()
        )
        job_name = next(iter(yaml.safe_load(WORKFLOW.read_text())["jobs"].values()))["name"]
        self.assertIn(job_name, registry["jobs"])
        self.assertEqual(registry["jobs"][job_name]["workflow"], WORKFLOW.name)

    def test_the_header_does_not_repeat_the_refuted_premise(self):
        """sq-huwr8: "it creates no check-run on a PR head, so it can never enter the
        gate poll set" is a TRUE premise with a FALSE conclusion. What makes this lane
        safe to RED is its registry declaration."""
        flat = re.sub(r"\n#\s*", " ", WORKFLOW.read_text())
        self.assertIsNone(
            re.search(r"never\s+enter\s+the\s+`?ci-summary\s*/\s*gate`?\s+poll\s+set",
                      flat, re.IGNORECASE)
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
