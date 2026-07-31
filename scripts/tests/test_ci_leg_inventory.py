#!/usr/bin/env python3
"""Suite for scripts/ci_leg_inventory.py. 🤖 SPARQ agent. Bead sq-6vshe.7.

The instrument's whole value is that its table can be TRUSTED as steering data, so the
suite is organised around the three ways a measurement tool lies:

  1. IT MIS-ATTRIBUTES.   Every bucket rule has a named case, and the two directions that
     matter most are pinned in BOTH directions: a step that must land in `constant` and a
     step that must NOT. Includes GitHub's implicit `Set up job` / `Post <x>` /
     `Complete job` steps, which no workflow author writes and which ARE the per-job tax
     sub-part (e) exists to measure.

  2. IT GOES QUIETLY STALE.   The classifier keys on step NAMES, which drift with any
     workflow edit. The fail-loud `--max-unclassified` guard is asserted on both sides —
     over the limit exits 2, under it exits 0 — so deleting or inverting the comparison
     reds. Backed by a LIVE corpus check over the real .github/workflows/ci.yml: every
     step name in the default target workflow must classify. The step EXTRACTOR that
     corpus check depends on is itself pinned (it must find a known step and must reject
     a `matrix:` include entry), so the corpus assertion cannot go vacuous by extracting
     nothing.

  3. IT REPORTS HEALTH OVER AN EMPTY POPULATION.   An empty sample is the instrument
     failing, not a repo with no CI cost — asserted to exit 2, never to print a clean
     empty table. Same failure class ci_execution_latency_alarm.py's empty-scan-set guard
     exists for.

  4. IT MEASURES FROM THE WRONG INSTANT.   `wait` is anchored at the run's `created_at`
     (queued), not `run_started_at` (began executing); the ordinary API payload always
     carries both, so anchoring at the latter would drop the workflow queue interval from
     every leg with nothing to make the miss visible. The anchor cases hold the two
     timestamps distinct — on the live corpus they are equal — so the wrong anchor reds,
     as does using a re-run's stale `created_at`.

Plus the finding functions, where the easy bug is a one-sided predicate: `fold_candidates`
requires BOTH short AND overhead-dominated, and `family_imbalance` must ignore a
single-leg family. Each has a negative case that reds if the conjunction is loosened.

Run:  python3 scripts/tests/test_ci_leg_inventory.py   (stdlib only; no pytest, no gh)
"""

from __future__ import annotations

import importlib.util
import io
import json
import re
import sys
import tempfile
import unittest
from contextlib import redirect_stdout, redirect_stderr
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "ci_leg_inventory.py"
CI_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "ci.yml"
DOCS_QUALITY = REPO_ROOT / ".github" / "workflows" / "docs-quality.yml"

_spec = importlib.util.spec_from_file_location("ci_leg_inventory", SCRIPT)
inv = importlib.util.module_from_spec(_spec)
sys.modules["ci_leg_inventory"] = inv
_spec.loader.exec_module(inv)


def step(name, start=None, end=None):
    return {"name": name, "started_at": start, "completed_at": end}


def stamp(seconds: int) -> str:
    """A UTC timestamp `seconds` after a fixed epoch, so every span in this suite is
    exact integer arithmetic rather than a wall-clock read."""
    minutes, secs = divmod(seconds, 60)
    hours, mins = divmod(minutes, 60)
    return f"2026-07-30T{hours:02d}:{mins:02d}:{secs:02d}Z"


def job(name, *, start=0, end=100, created=None, steps=(), conclusion="success"):
    return {
        "name": name,
        "conclusion": conclusion,
        "created_at": stamp(created if created is not None else start),
        "started_at": stamp(start),
        "completed_at": stamp(end),
        "steps": list(steps),
    }


def span_step(name, at, seconds):
    return step(name, stamp(at), stamp(at + seconds))


# ---------------------------------------------------------------------------------
# 1. Classification
# ---------------------------------------------------------------------------------
class TestClassify(unittest.TestCase):
    CASES = [
        # GitHub's implicit steps — the floor of the per-job constant tax.
        ("Set up job", "setup"),
        ("Complete job", "setup"),
        ("Post Run actions/checkout@v7", "setup"),
        ("Post Cache cargo + target", "setup"),
        # Explicit per-job tax, taken verbatim from .github/workflows/ci.yml.
        ("Free runner disk before build + archive", "disk"),
        ("Rust (preinstalled stable)", "toolchain"),
        ("Ensure llvm-tools-preview on the pinned toolchain", "toolchain"),
        ("Cache cargo + target", "cache"),
        ("Install cargo-nextest", "tool-install"),
        ("Upload nextest archive", "artifact"),
        ("Download nextest archive", "artifact"),
        ("Fetch W3C SPARQL test suites (pinned)", "fixture-fetch"),
        # Compile-bearing, no test execution.
        ("Build + archive test binaries (nextest archive)", "compile"),
        ("clippy (deny warnings)", "compile"),
        ("rustdoc (deny warnings, workspace — all features)", "compile"),
        ("cargo check on the declared MSRV", "compile"),
        ("Build sparq-wasm for the browser target", "compile"),
        # Test execution.
        ("Doctests (nextest does not run these; run once, off the warm build)", "test-run"),
        ("Test (nextest, shard bulk 1/3)", "test-run"),
        ("Run conformance suites", "test-run"),
        ("Run SHACL core + SHACL-SPARQL suites", "test-run"),
        ("Headless wasm tests (wasm-pack test --node)", "test-run"),
        ("Measure + enforce per-crate coverage (FULL tier, max-remeasure gate)", "test-run"),
        # Cheap bookkeeping.
        ("Enforce ratchet (pass count must not regress)", "script"),
        ("Test-presence gate (fast, no compile)", "script"),
        ("Summarize", "script"),
        # A rendered report names its subject first, so the `^report ` anchor misses it.
        ("Cone coverage report (sq-6vshe.8 / sq-3dr4t) [SONNET-4.6]", "script"),
        ("Report unsafe usage (sparq-core, where the unsafe lives)", "script"),
    ]

    def test_every_case_lands_in_its_bucket(self):
        for name, expected in self.CASES:
            with self.subTest(name=name):
                self.assertEqual(inv.classify_step(name), expected)

    def test_buckets_roll_up_to_a_known_class(self):
        for bucket in {b for _, b in self.CASES}:
            self.assertIn(inv.CLASS_OF_BUCKET[bucket], inv.CLASSES)

    def test_an_unknown_name_is_unclassified_not_guessed(self):
        # The load-bearing NEGATIVE: silently absorbing an unrecognised step into `script`
        # or `other` is exactly how the table would start lying (module header point 3).
        self.assertEqual(inv.classify_step("Frobnicate the widget"), "unclassified")
        self.assertEqual(inv.classify_step(""), "unclassified")
        self.assertEqual(inv.classify_step(None), "unclassified")

    def test_a_compile_step_is_not_counted_as_constant_overhead(self):
        # Direction that would silently inflate the (e) fold signal if the rules drifted.
        for name in ("clippy (deny warnings)", "Build + archive test binaries (nextest archive)"):
            self.assertEqual(inv.CLASS_OF_BUCKET[inv.classify_step(name)], "compile")

    def test_a_test_step_is_not_counted_as_compile(self):
        self.assertEqual(
            inv.CLASS_OF_BUCKET[inv.classify_step("Test (nextest, shard heavy-diskann)")],
            "test-run")

    def test_a_step_that_merely_mentions_a_report_keeps_its_real_bucket(self):
        # The other direction of the "rendered report is bookkeeping" rule. These are the
        # real ci.yml names carrying the word `report` while doing the expensive work;
        # first-match-wins must reach them before the trailing script rule, or minutes of
        # compile and test-run time would be laundered into `other`.
        for name, expected in (
            ("Build instrumented engine objects (for `report`, no test run)", "compile"),
            ("Clippy the served-conformance report emitter (both served-surface features ON)",
             "compile"),
            ("Compile + run partition 2/3 (--no-report, emit .profraw)", "compile"),
            ("Upload report", "artifact"),
        ):
            with self.subTest(name=name):
                self.assertEqual(inv.classify_step(name), expected)

    def test_overrides_win_over_every_rule(self):
        name = "clippy (deny warnings)"
        self.assertEqual(inv.classify_step(name), "compile")
        self.assertEqual(inv.classify_step(name, {name: "script"}), "script")

    def test_an_override_naming_an_unknown_bucket_fails_loud(self):
        with self.assertRaises(inv.InventoryError):
            inv.classify_step("x", {"x": "not-a-bucket"})

    def test_overrides_match_exactly_not_by_substring(self):
        self.assertEqual(inv.classify_step("clippy (deny warnings)", {"clippy": "script"}),
                         "compile")


# ---------------------------------------------------------------------------------
# 2. Decomposition
# ---------------------------------------------------------------------------------
class TestDecompose(unittest.TestCase):
    def test_step_time_lands_in_the_right_class(self):
        entry = inv.decompose_job(job(
            "build + archive",
            start=100, end=400,
            steps=[span_step("Set up job", 100, 10),
                   span_step("Cache cargo + target", 110, 20),
                   span_step("Build + archive test binaries (nextest archive)", 130, 200),
                   span_step("Doctests (nextest does not run these)", 330, 60)],
        ), wait_anchor=stamp(40))
        self.assertEqual(entry["classes"]["constant"], 30.0)
        self.assertEqual(entry["classes"]["compile"], 200.0)
        self.assertEqual(entry["classes"]["test-run"], 60.0)
        self.assertEqual(entry["classes"]["unclassified"], 0.0)
        self.assertEqual(entry["total_s"], 300.0)
        # 300s job, 290s of steps -> 10s of runner start-up/teardown outside any step.
        self.assertEqual(entry["ungrouped_s"], 10.0)

    def test_wait_is_measured_from_the_runs_anchor_so_needs_edges_are_visible(self):
        entry = inv.decompose_job(job("test shard", start=300, end=400),
                                  wait_anchor=stamp(0))
        self.assertEqual(entry["wait_s"], 300.0)

    def test_wait_falls_back_to_the_jobs_own_created_at(self):
        entry = inv.decompose_job(job("solo", start=100, end=200, created=70))
        self.assertEqual(entry["wait_s"], 30.0)

    def test_a_skipped_step_with_null_timestamps_contributes_zero(self):
        entry = inv.decompose_job(job(
            "leg", start=0, end=50,
            steps=[step("Install cargo-nextest", None, None),
                   span_step("clippy (deny warnings)", 0, 50)]))
        self.assertEqual(entry["classes"]["constant"], 0.0)
        self.assertEqual(entry["classes"]["compile"], 50.0)

    def test_a_negative_span_is_clamped_not_subtracted(self):
        # One clock skew must not silently shrink a column below the truth.
        entry = inv.decompose_job(job(
            "leg", start=0, end=50,
            steps=[step("clippy (deny warnings)", stamp(40), stamp(10))]))
        self.assertEqual(entry["classes"]["compile"], 0.0)

    def test_an_unparseable_timestamp_fails_loud(self):
        with self.assertRaises(inv.InventoryError):
            inv.decompose_job(job("leg", steps=[step("clippy", "not-a-time", stamp(1))]))

    def test_a_job_without_a_name_fails_loud(self):
        with self.assertRaises(inv.InventoryError):
            inv.decompose_job({"steps": []})

    def test_a_malformed_step_entry_fails_loud(self):
        with self.assertRaises(inv.InventoryError):
            inv.decompose_job(job("leg", steps=["clippy"]))

    def test_leg_family_groups_matrix_siblings_and_leaves_solo_jobs_alone(self):
        self.assertEqual(inv.leg_family("test (load-aware shard bulk 1/3)"), "test")
        self.assertEqual(inv.leg_family("test (load-aware shard heavy-diskann)"), "test")
        self.assertEqual(inv.leg_family("build + archive test binaries (+ doctests once)"),
                         "build + archive test binaries")
        self.assertEqual(inv.leg_family("clippy"), "clippy")


class TestWaitAnchor(unittest.TestCase):
    """Which instant `wait` is measured FROM — the difference between a queue-time column
    and a dependency-delay column.

    A run's `created_at` is when it was QUEUED, `run_started_at` when it began EXECUTING.
    Anchoring at the latter drops the workflow queue interval from every leg while the
    column still claims to be queue time, and since the ordinary payload always carries
    `run_started_at` there is no live case in which the miss shows up. These cases are
    therefore DEFINITIONAL — on this corpus the two timestamps are measured to be equal
    (module header), so they are held distinct here precisely so the wrong anchor reds.
    The re-run exception is pinned the same way: `run_attempt > 1` leaves `created_at` on
    the ORIGINAL attempt, so using it would report idle-before-re-run as queue time.
    """

    def test_a_first_attempt_anchors_at_the_runs_created_at_not_its_start(self):
        run = {"created_at": stamp(0), "run_started_at": stamp(30), "run_attempt": 1}
        self.assertEqual(inv.run_wait_anchor(run), stamp(0))

    def test_a_run_with_no_attempt_field_is_treated_as_a_first_attempt(self):
        run = {"created_at": stamp(0), "run_started_at": stamp(30)}
        self.assertEqual(inv.run_wait_anchor(run), stamp(0))

    def test_a_rerun_refuses_its_stale_created_at(self):
        # `created_at` here is the FIRST attempt's; reporting it would turn every leg's
        # wait into the age of the original run.
        run = {"created_at": stamp(0), "run_started_at": stamp(30), "run_attempt": 4}
        self.assertEqual(inv.run_wait_anchor(run), stamp(30))

    def test_a_run_carrying_neither_timestamp_yields_no_anchor(self):
        # None, not a guess: decompose_job then falls back to the job's own created_at.
        self.assertIsNone(inv.run_wait_anchor({"id": 7}))

    def test_a_non_object_run_fails_loud(self):
        with self.assertRaises(inv.InventoryError):
            inv.run_wait_anchor(["not-a-run"])


class TestLiveCollection(unittest.TestCase):
    """The gh path end-to-end, because that is where the anchor choice actually bites:
    a live run always carries `run_started_at`, so an anchor bug never reaches the
    job-`created_at` fallback the fixture tests exercise."""

    def stub_gh(self, runs_payload, jobs_payload):
        def fake(args):
            return runs_payload if "/workflows/" in args[-1] else jobs_payload

        original = inv._gh
        inv._gh = fake
        self.addCleanup(lambda: setattr(inv, "_gh", original))

    RUN_QUEUED, RUN_STARTED, JOB_STARTED = 0, 30, 100

    def live(self, attempt):
        self.stub_gh(
            {"workflow_runs": [{"id": 7, "run_attempt": attempt,
                                "created_at": stamp(self.RUN_QUEUED),
                                "run_started_at": stamp(self.RUN_STARTED)}]},
            {"jobs": [job("test (load-aware shard bulk 1/3)",
                          start=self.JOB_STARTED, end=400, created=self.RUN_QUEUED)]})
        [entry] = inv.collect("o/r", "ci.yml", None, None, 1, None)
        return entry

    def test_wait_covers_the_run_queue_interval_and_the_needs_delay(self):
        # Queued at 0, executing at 30 (a 30s workflow queue), leg starts at 100 (a further
        # 70s blocked on `needs:`). `wait` is BOTH — 70.0 here would mean the queue
        # interval had been dropped.
        self.assertEqual(self.live(attempt=1)["wait_s"], 100.0)

    def test_a_rerun_reports_dependency_delay_only(self):
        self.assertEqual(self.live(attempt=2)["wait_s"], 70.0)


# ---------------------------------------------------------------------------------
# 3. Aggregation
# ---------------------------------------------------------------------------------
class TestAggregate(unittest.TestCase):
    def test_columns_are_medianed_not_meaned(self):
        # A single cold-cache outlier must not move a leg's steering number.
        entries = [inv.decompose_job(job("leg", start=0, end=e)) for e in (100, 110, 900)]
        row = inv.aggregate_legs(entries)[0]
        self.assertEqual(row["n"], 3)
        self.assertEqual(row["total_s"], 110.0)

    def test_rows_are_ordered_slowest_first(self):
        rows = inv.aggregate_legs([
            inv.decompose_job(job("fast", start=0, end=10)),
            inv.decompose_job(job("slow", start=0, end=500)),
        ])
        self.assertEqual([r["leg"] for r in rows], ["slow", "fast"])

    def test_overhead_share_is_constant_over_total(self):
        row = inv.aggregate_legs([inv.decompose_job(job(
            "leg", start=0, end=100,
            steps=[span_step("Set up job", 0, 60),
                   span_step("clippy (deny warnings)", 60, 40)]))])[0]
        self.assertAlmostEqual(row["overhead_share"], 0.6)

    def test_unclassified_share_is_a_percentage_of_step_time(self):
        entries = [inv.decompose_job(job(
            "leg", start=0, end=100,
            steps=[span_step("clippy (deny warnings)", 0, 75),
                   span_step("Frobnicate the widget", 75, 25)]))]
        self.assertAlmostEqual(inv.unclassified_share(entries), 25.0)

    def test_unclassified_share_of_an_empty_sample_is_zero_not_a_crash(self):
        self.assertEqual(inv.unclassified_share([]), 0.0)


# ---------------------------------------------------------------------------------
# 4. Findings
# ---------------------------------------------------------------------------------
class TestFindings(unittest.TestCase):
    @staticmethod
    def row(leg, total, constant, family=None):
        return {"leg": leg, "family": family or inv.leg_family(leg), "n": 1,
                "total_s": total, "constant": constant,
                "overhead_share": constant / total if total else 0.0}

    def test_a_short_overhead_dominated_leg_is_a_fold_candidate(self):
        found = inv.fold_candidates([self.row("tiny gate", 60.0, 50.0)])
        self.assertEqual([c["leg"] for c in found], ["tiny gate"])

    def test_a_long_overhead_heavy_leg_is_not_a_fold_candidate(self):
        # Reds if the AND is loosened to an OR: this leg wants its overhead fixed
        # (cache/toolchain), not to be merged into a neighbour.
        self.assertEqual(inv.fold_candidates([self.row("big slow leg", 1200.0, 1000.0)]), [])

    def test_a_short_but_cheap_overhead_leg_is_not_a_fold_candidate(self):
        # The other half of the conjunction: short is not sufficient.
        self.assertEqual(inv.fold_candidates([self.row("quick real work", 60.0, 5.0)]), [])

    def test_a_zero_duration_leg_is_never_a_fold_candidate(self):
        self.assertEqual(inv.fold_candidates([self.row("skipped", 0.0, 0.0)]), [])

    def test_fold_candidates_are_ordered_by_reclaimable_constant_time(self):
        found = inv.fold_candidates([self.row("a", 100.0, 60.0), self.row("b", 100.0, 90.0)])
        self.assertEqual([c["leg"] for c in found], ["b", "a"])

    def test_an_unbalanced_family_is_reported_with_its_idle_time(self):
        rows = [self.row("test (bulk 1/3)", 100.0, 10.0),
                self.row("test (bulk 2/3)", 100.0, 10.0),
                self.row("test (heavy-diskann)", 400.0, 10.0)]
        [gap] = inv.family_imbalance(rows)
        self.assertEqual(gap["family"], "test")
        self.assertEqual(gap["slowest_leg"], "test (heavy-diskann)")
        self.assertAlmostEqual(gap["spread"], 4.0)
        # Two legs idle 300s each waiting for the slow one; the slow one idles 0.
        self.assertAlmostEqual(gap["idle_s"], 600.0)

    def test_a_single_leg_family_is_never_imbalanced(self):
        # Asserted at ratio=1.0, where a family's own spread of exactly 1.0 WOULD clear
        # the threshold — so the member-count guard, not the ratio, is what this pins.
        # At the default ratio the assertion would pass with that guard deleted.
        self.assertEqual(inv.family_imbalance([self.row("solo", 400.0, 10.0)], ratio=1.0), [])

    def test_a_family_whose_only_timed_leg_is_alone_is_not_imbalanced(self):
        # Same guard, reached the other way: two members but only one with a duration.
        rows = [self.row("test (a)", 400.0, 10.0), self.row("test (b)", 0.0, 0.0)]
        self.assertEqual(inv.family_imbalance(rows, ratio=1.0), [])

    def test_a_family_inside_the_ratio_is_not_reported(self):
        rows = [self.row("test (a)", 100.0, 10.0), self.row("test (b)", 120.0, 10.0)]
        self.assertEqual(inv.family_imbalance(rows), [])

    def test_the_doctest_bill_finds_the_doctest_step_wherever_it_runs(self):
        entries = [inv.decompose_job(job(
            "build + archive", start=0, end=300,
            steps=[span_step("Build + archive test binaries (nextest archive)", 0, 200),
                   span_step("Doctests (nextest does not run these)", 200, 100)]))]
        bill = inv.doctest_bill(entries)
        self.assertTrue(bill["found"])
        self.assertEqual(bill["total_s"], 100.0)
        self.assertEqual(bill["legs"][0]["leg"], "build + archive")

    def test_the_doctest_bill_reports_not_found_rather_than_zero(self):
        # `found: False` says "no doctest step was observed"; a bare 0.0 would read as
        # "doctests are free", which is the opposite conclusion.
        entries = [inv.decompose_job(job("leg", start=0, end=10,
                                         steps=[span_step("clippy (deny warnings)", 0, 10)]))]
        self.assertFalse(inv.doctest_bill(entries)["found"])

    def test_slowest_steps_are_ordered_and_capped(self):
        entries = [inv.decompose_job(job(
            "leg", start=0, end=100,
            steps=[span_step("clippy (deny warnings)", 0, 10),
                   span_step("Run conformance suites", 10, 90)]))]
        rows = inv.slowest_steps(entries, limit=1)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["step"], "Run conformance suites")
        self.assertEqual(rows[0]["bucket"], "test-run")


# ---------------------------------------------------------------------------------
# 5. Fixture ingestion
# ---------------------------------------------------------------------------------
class TestFixture(unittest.TestCase):
    def test_the_runs_shape_is_accepted(self):
        payload = {"runs": [{"run_started_at": stamp(0),
                             "jobs": [job("leg", start=10, end=20)]}]}
        [entry] = inv.collect_fixture(payload, None)
        self.assertEqual(entry["wait_s"], 10.0)

    def test_a_fixture_run_anchors_on_created_at_exactly_as_the_live_path_does(self):
        # Same anchor rule on both paths, or a fixture replay of a live run would not
        # reproduce that run's table.
        payload = {"runs": [{"created_at": stamp(0), "run_started_at": stamp(30),
                             "jobs": [job("leg", start=100, end=200, created=0)]}]}
        [entry] = inv.collect_fixture(payload, None)
        self.assertEqual(entry["wait_s"], 100.0)

    def test_a_bare_jobs_dump_is_accepted(self):
        [entry] = inv.collect_fixture({"jobs": [job("leg", start=0, end=20)]}, None)
        self.assertEqual(entry["total_s"], 20.0)

    def test_incomplete_jobs_are_excluded_from_the_sample(self):
        # A cancelled/skipped/running leg has no wall-time and must NOT be medianed in as
        # a fast one — that would understate the very legs this table steers.
        payload = {"jobs": [job("done", start=0, end=20),
                            job("cut", start=0, end=1, conclusion="cancelled"),
                            job("skip", start=0, end=1, conclusion="skipped"),
                            job("live", start=0, end=1, conclusion=None)]}
        self.assertEqual([e["leg"] for e in inv.collect_fixture(payload, None)], ["done"])

    def test_a_fixture_with_neither_runs_nor_jobs_fails_loud(self):
        with self.assertRaises(inv.InventoryError):
            inv.collect_fixture({"legs": []}, None)

    def test_a_non_object_fixture_fails_loud(self):
        with self.assertRaises(inv.InventoryError):
            inv.collect_fixture([], None)


class TestPaging(unittest.TestCase):
    """The jobs endpoint is OBJECT-valued, so it is paged by hand rather than with
    `gh api --paginate` (which would emit unparseable concatenated objects). Only a run
    WIDE enough to need a second page exercises this — i.e. exactly the wide-matrix runs
    this inventory exists for — so it is pinned here rather than left to a live run."""

    def stub_gh(self, pages):
        calls = []

        def fake(args):
            calls.append(args[-1])
            return pages[len(calls) - 1]

        self.calls = calls
        original = inv._gh
        inv._gh = fake
        self.addCleanup(lambda: setattr(inv, "_gh", original))

    def test_a_second_page_is_fetched_and_concatenated(self):
        first = {"jobs": [job(f"leg {i}") for i in range(100)]}
        second = {"jobs": [job("leg 100")]}
        self.stub_gh([first, second])
        jobs = inv.fetch_jobs("o/r", 7)
        self.assertEqual(len(jobs), 101)
        self.assertIn("page=1", self.calls[0])
        self.assertIn("page=2", self.calls[1])

    def test_a_short_first_page_stops_immediately(self):
        self.stub_gh([{"jobs": [job("only")]}])
        self.assertEqual(len(inv.fetch_jobs("o/r", 7)), 1)
        self.assertEqual(len(self.calls), 1)

    def test_a_response_without_jobs_fails_loud(self):
        self.stub_gh([{"total_count": 3}])
        with self.assertRaises(inv.InventoryError):
            inv.fetch_jobs("o/r", 7)

    def test_a_run_listing_without_workflow_runs_fails_loud(self):
        self.stub_gh([{"total_count": 3}])
        with self.assertRaises(inv.InventoryError):
            inv.fetch_runs("o/r", "ci.yml", None, None, 5)

    def test_the_run_listing_honours_event_branch_and_limit(self):
        self.stub_gh([{"workflow_runs": [{"id": n} for n in range(5)]}])
        runs = inv.fetch_runs("o/r", "ci.yml", "pull_request", "main", 3)
        self.assertEqual(len(runs), 3)
        self.assertIn("event=pull_request", self.calls[0])
        self.assertIn("branch=main", self.calls[0])


# ---------------------------------------------------------------------------------
# 6. CLI behaviour — the fail-loud contract
# ---------------------------------------------------------------------------------
def write_fixture(payload) -> str:
    handle = tempfile.NamedTemporaryFile("w", suffix=".json", delete=False)
    json.dump(payload, handle)
    handle.close()
    return handle.name


def invoke(argv):
    out, err = io.StringIO(), io.StringIO()
    with redirect_stdout(out), redirect_stderr(err):
        code = inv.main(argv)
    return code, out.getvalue(), err.getvalue()


class TestCli(unittest.TestCase):
    CLEAN = {"jobs": [job("leg", start=0, end=100,
                          steps=[span_step("Set up job", 0, 10),
                                 span_step("clippy (deny warnings)", 10, 90)])]}
    # 80% of step time is a step no rule recognises.
    DRIFTED = {"jobs": [job("leg", start=0, end=100,
                            steps=[span_step("clippy (deny warnings)", 0, 20),
                                   span_step("Frobnicate the widget", 20, 80)])]}

    def test_a_clean_sample_reports_and_exits_zero(self):
        code, out, _ = invoke(["--fixture", write_fixture(self.CLEAN), "--steps", "--families"])
        self.assertEqual(code, 0)
        self.assertIn("per-leg wall-time inventory", out)
        self.assertIn("`leg`", out)

    def test_a_drifted_classifier_exits_two_and_names_the_step(self):
        # THE headline guard: a name-based classifier that no longer recognises the corpus
        # would produce a plausible WRONG table. It must fail loudly instead.
        code, out, err = invoke(["--fixture", write_fixture(self.DRIFTED)])
        self.assertEqual(code, 2)
        self.assertEqual(out, "")
        self.assertIn("unclassified", err)
        self.assertIn("Frobnicate the widget", err)

    def test_the_same_drifted_sample_passes_when_the_limit_is_raised(self):
        # The other side of the comparison, so the guard cannot be satisfied by a test
        # that would also pass with the check hard-wired to always fail.
        code, _, _ = invoke(["--fixture", write_fixture(self.DRIFTED), "--max-unclassified", "99"])
        self.assertEqual(code, 0)

    def test_an_override_rescues_a_drifted_step_without_raising_the_limit(self):
        overrides = write_fixture({"Frobnicate the widget": "script"})
        code, _, _ = invoke(["--fixture", write_fixture(self.DRIFTED), "--overrides", overrides])
        self.assertEqual(code, 0)

    def test_an_empty_sample_exits_two_rather_than_printing_a_clean_table(self):
        code, out, err = invoke(["--fixture", write_fixture({"jobs": []})])
        self.assertEqual(code, 2)
        self.assertEqual(out, "")
        self.assertIn("no completed jobs", err)

    def test_json_output_is_parseable_and_carries_every_section(self):
        code, out, _ = invoke(["--fixture", write_fixture(self.CLEAN), "--json"])
        self.assertEqual(code, 0)
        payload = json.loads(out)
        for key in ("legs", "doctests", "fold_candidates", "family_imbalance",
                    "slowest_steps", "unclassified_pct", "observations"):
            self.assertIn(key, payload)

    def test_no_repo_and_no_fixture_exits_two_instead_of_calling_gh(self):
        code, _, err = invoke(["--repo", ""])
        self.assertEqual(code, 2)
        self.assertIn("--repo is required", err)

    def test_a_nonpositive_run_count_exits_two(self):
        code, _, err = invoke(["--repo", "o/r", "--runs", "0"])
        self.assertEqual(code, 2)
        self.assertIn("--runs", err)


# ---------------------------------------------------------------------------------
# 7. Live corpus — the classifier must cover the workflow it defaults to
# ---------------------------------------------------------------------------------
_NAME_RE = re.compile(r"^(\s*)- name:\s*(.+?)\s*$")
_CMD_RE = re.compile(r"^\s*(run|uses):")


def workflow_step_names(path: Path) -> list[str]:
    """Every STEP name in a workflow, excluding `matrix:` include entries.

    A `strategy.matrix.include` entry is also written `- name: …`, but only a STEP carries
    a sibling `run:`/`uses:`. Selecting on that keeps `bulk 1/3` (a matrix VALUE, never
    returned by the jobs API as a step) out of the corpus. Hand-rolled rather than PyYAML
    so this suite stays stdlib-only and runs in any worktree.
    """
    lines = path.read_text(encoding="utf-8").splitlines()
    names: list[str] = []
    for index, line in enumerate(lines):
        match = _NAME_RE.match(line)
        if not match:
            continue
        indent, name = len(match.group(1)), match.group(2)
        for follower in lines[index + 1:]:
            if not follower.strip() or follower.lstrip().startswith("#"):
                continue
            stripped_indent = len(follower) - len(follower.lstrip())
            if stripped_indent <= indent:
                break
            if _CMD_RE.match(follower):
                names.append(name.strip().strip("\"'"))
                break
    return names


class TestLiveCorpus(unittest.TestCase):
    def test_the_step_extractor_itself_is_not_vacuous(self):
        names = workflow_step_names(CI_WORKFLOW)
        self.assertGreater(len(names), 50, "extractor found almost no steps — a corpus "
                                           "assertion over this would be vacuous")
        self.assertIn("Build + archive test binaries (nextest archive)", names)
        # The matrix-include trap: these are matrix VALUES, not steps.
        for matrix_value in ("bulk 1/3", "heavy-diskann"):
            self.assertNotIn(matrix_value, names)

    def test_every_step_name_in_ci_yml_classifies(self):
        # ci.yml is this tool's DEFAULT --workflow. If a step there is unrecognised, the
        # default invocation mis-attributes or trips its own fail-loud guard.
        unknown = sorted({n for n in workflow_step_names(CI_WORKFLOW)
                          if inv.classify_step(n) == "unclassified"})
        self.assertEqual(unknown, [], f"add a rule in ci_leg_inventory.py: {unknown}")


# ---------------------------------------------------------------------------------
# 8. The suite pins its own call site, so it cannot silently leave CI
# ---------------------------------------------------------------------------------
class TestWiring(unittest.TestCase):
    def test_this_suite_is_invoked_by_docs_quality(self):
        text = DOCS_QUALITY.read_text(encoding="utf-8")
        # assertTrue, not assertIn: a failed assertIn would dump the whole workflow.
        self.assertTrue("python3 scripts/tests/test_ci_leg_inventory.py" in text,
                        "docs-quality.yml no longer invokes this suite")

    def test_the_instrument_is_not_wired_as_a_gate(self):
        # It reports; it decides nothing about a commit. A workflow that RUNS it would
        # put its exit code on a check-run — deliberately not done (module header).
        for workflow in sorted((REPO_ROOT / ".github" / "workflows").glob("*.yml")):
            text = workflow.read_text(encoding="utf-8")
            self.assertNotIn("ci_leg_inventory.py --repo", text,
                             f"{workflow.name} invokes the inventory against the live API")


if __name__ == "__main__":
    unittest.main(verbosity=2)
