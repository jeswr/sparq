#!/usr/bin/env python3
# [OPUS-5] sparq-org/sparq#4820 — the nightly mutation-lane report must actually be
# READABLE, and must be DECLARED non-gating rather than assumed to be. 🤖 SPARQ agent.
#
# THE DEFECT THIS PINS. ci.yml's `mutants-nightly-advisory` matrix is the repo's largest
# nightly CI consumer and its outcome was unreadable: advisory (so never red on a PR),
# 51 legs deep (so per-leg conclusions need the matrix expanded), and rolled up into a
# run-level conclusion that reads `cancelled` whenever ONE sibling job anywhere in ci.yml
# is cancelled — so a night that failed ten legs reported the same status as a night
# killed at startup. The `mutants-nightly-report` job exists to carry that state in its
# own named check-run instead. Two ways that fix can silently stop working, both pinned
# here:
#
#   1. THE JOIN GOES QUIET. The report finds its legs by NAME — a matrix leg is
#      "<job name> (<matrix values>)". Rename the matrix job and the prefix matches
#      nothing, the report finds zero legs, and a zero-leg report that returned 0 would
#      be indistinguishable from a perfect night. So DEFAULT_LEG_PREFIX is bound to the
#      live YAML here, AND build_report() is asserted to RED on an empty leg set.
#
#   2. THE REPORT GATES. Since #3773 the ONLY thing that makes a check-run non-gating is
#      a key in .github/advisory-registry.json. This job REDs by design while the lane is
#      broken, and it runs on schedule/workflow_dispatch — which does NOT make it safe:
#      ci-summary.yml also runs on push to main, so a scheduled run's check-run lands on
#      exactly the head SHA that gate polls (the sq-huwr8 refutation, measured on main
#      head cb0c6739c). Undeclared, this reporter would block every merge on a nightly
#      capacity condition, and a reporter that blocks merges is one that gets muted.
#
# HOW THIS SUITE IS BUILT TO FAIL.
#   * The non-gating half drives the REAL ci_summary_gate.render_verdict() — the exact
#     function the `gate` job runs — and asserts on its integer exit code. No
#     re-implementation of the gating rule lives here.
#   * test_control_* is the ANTI-VACUITY control in BOTH halves: an UNDECLARED lane
#     failing must still RED the gate, and a HEALTHY night must report exit 0. Without
#     them a render_verdict() that always returned 0, or a build_report() that always
#     returned 1, would satisfy every other assertion in this file.
#   * The behaviour half drives the REAL build_report() over fixtures shaped like the
#     nights in #4820 — a timed-out leg with no artifact, and a GREEN leg whose artifact
#     is TRUNCATED (the case no leg conclusion can express at all).
#   * The suite pins its OWN call site in docs-quality.yml, so it cannot silently leave
#     CI's reachable set.
#
# Stdlib + PyYAML (already a docs-quality dependency). Run:
#   python3 scripts/tests/test_mutants_nightly_report.py

from __future__ import annotations

import importlib.util
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import ci_summary_gate as gate  # noqa: E402

REGISTRY_PATH = REPO_ROOT / ".github" / "advisory-registry.json"
CI_YML = REPO_ROOT / ".github" / "workflows" / "ci.yml"
DOCS_QUALITY_YML = REPO_ROOT / ".github" / "workflows" / "docs-quality.yml"

MATRIX_JOB_ID = "mutants-nightly-advisory"
REPORT_JOB_ID = "mutants-nightly-report"
REPORT_JOB_NAME = "mutation ratchet report (cargo-mutants, advisory)"

# A name deliberately NOT in the registry, for the anti-vacuity control.
UNDECLARED_LANE = "some undeclared lane that must gate"


def _load_reporter():
    """scripts/mutants-nightly-report.py — hyphenated, so not importable by name."""
    path = REPO_ROOT / "scripts" / "mutants-nightly-report.py"
    spec = importlib.util.spec_from_file_location("mutants_nightly_report", path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


report = _load_reporter()
CI_DOC = yaml.safe_load(CI_YML.read_text())
CI_JOBS = CI_DOC["jobs"]


def _run(name: str, conclusion: str) -> dict:
    """One terminal check-run, shaped as the checks API returns it."""
    return {"name": name, "status": "completed", "conclusion": conclusion,
            "id": abs(hash((name, conclusion))) % 10_000_000, "_workflow_id": 1}


def _healthy_siblings() -> list[dict]:
    """A green sibling set, including the selection pre-job the gate requires to be
    successful before it will trust any `skipped` conclusion."""
    return [_run("changed-file test selection", "success"),
            _run("build", "success"),
            _run("clippy", "success")]


class TestLegJoinIsBoundToTheLiveWorkflow(unittest.TestCase):
    """If the report cannot find its legs it reports nothing, and nothing looks green."""

    def test_leg_prefix_equals_the_matrix_jobs_live_name(self) -> None:
        """THE join. Rename the matrix job without updating DEFAULT_LEG_PREFIX and every
        future report is empty — this REDs on the rename instead."""
        self.assertEqual(
            report.DEFAULT_LEG_PREFIX,
            CI_JOBS[MATRIX_JOB_ID]["name"],
            "scripts/mutants-nightly-report.py DEFAULT_LEG_PREFIX has drifted from "
            f"ci.yml jobs.{MATRIX_JOB_ID}.name — the report would find zero legs",
        )

    def test_report_job_never_counts_itself_as_a_leg(self) -> None:
        """The report job's own check-run sits in the same jobs list it reads. Matching it
        as a leg would make the report's own conclusion an input to itself."""
        self.assertIsNone(
            report.leg_key(CI_JOBS[REPORT_JOB_ID]["name"], report.DEFAULT_LEG_PREFIX),
            "the report job's name matches its own leg prefix — it would self-count",
        )

    def test_matrix_leg_names_join_to_artifact_identity(self) -> None:
        """A leg name carries the crate and (when sub-sharded) the `k/n` slice; the
        artifact carries the same identity as `-shard-k-n`. Both halves of the join are
        asserted against the shapes ci.yml's matrix actually produces."""
        p = report.DEFAULT_LEG_PREFIX
        self.assertEqual(report.leg_key(f"{p} (sparq-canon)", p), ("sparq-canon", None))
        self.assertEqual(report.leg_key(f"{p} (sparq-engine, 360, 9/24)", p),
                         ("sparq-engine", "9/24"))
        # A comma-joined FEATURE list is one matrix value (values are joined with ", ").
        self.assertEqual(report.leg_key(f"{p} (sparq-core, mmap,dict-spill, 360)", p),
                         ("sparq-core", None))
        self.assertIsNone(report.leg_key("build + test (ubuntu)", p))

    def test_download_pattern_matches_the_upload_name(self) -> None:
        """The report reads the artifacts the matrix legs upload. If the two names drift,
        every leg reports `no artifact` and the report REDs for a bookkeeping reason."""
        upload = [s for s in CI_JOBS[MATRIX_JOB_ID]["steps"]
                  if str(s.get("uses", "")).startswith("actions/upload-artifact@")]
        self.assertEqual(len(upload), 1, "expected exactly one artifact upload in the leg")
        download = [s for s in CI_JOBS[REPORT_JOB_ID]["steps"]
                    if str(s.get("uses", "")).startswith("actions/download-artifact@")]
        self.assertEqual(len(download), 1, "expected exactly one download in the report")
        pattern = download[0]["with"]["pattern"]
        self.assertTrue(pattern.endswith("*"), pattern)
        self.assertTrue(
            upload[0]["with"]["name"].startswith(pattern[:-1]),
            f"upload name {upload[0]['with']['name']!r} does not match download pattern "
            f"{pattern!r} — the report would see no evidence for any leg",
        )


class TestReportJobWiring(unittest.TestCase):
    """The YAML seam. Each assertion pins a property whose loss is SILENT."""

    def setUp(self) -> None:
        self.job = CI_JOBS[REPORT_JOB_ID]

    def test_runs_even_when_the_legs_failed(self) -> None:
        """Reporting on failed/cancelled legs IS the job. Without always(), the implicit
        needs-success gate skips it on exactly the nights it is needed."""
        self.assertIn("always()", self.job["if"])
        self.assertIn("needs.nightly-gate.outputs.fresh == 'true'", self.job["if"],
                      "always() defeats the needs gate but NOT the freshness guard — it "
                      "must stay explicit or an unchanged-HEAD nightly still reports")
        self.assertEqual(sorted(self.job["needs"]),
                         sorted(["nightly-gate", MATRIX_JOB_ID]))

    def test_reporter_has_no_write_capability(self) -> None:
        """`actions: read` is the whole capability. A reporter that could write must never
        be able to acquire authority over the lanes it reports on."""
        self.assertEqual(self.job["permissions"], {"contents": "read", "actions": "read"})

    def test_actions_are_sha_pinned(self) -> None:
        for step in self.job["steps"]:
            uses = step.get("uses")
            if uses:
                with self.subTest(uses=uses):
                    ref = uses.split("@", 1)[1]
                    self.assertRegex(ref, r"^[0-9a-f]{40}$",
                                     f"{uses} is not pinned to a full commit SHA")

    def test_the_verdict_cannot_be_swallowed(self) -> None:
        """The whole point is a conclusion that means something. `continue-on-error` on
        this job or its steps would restore exactly the swallowing #4820 is about — the
        sibling matrix job carries it at BOTH levels, which is why a ratchet regression
        there can never turn a leg red."""
        self.assertNotIn("continue-on-error", self.job)
        for step in self.job["steps"]:
            with self.subTest(step=step.get("name") or step.get("uses")):
                self.assertNotIn("continue-on-error", step)
        # The premise of that comparison, asserted rather than remembered.
        self.assertTrue(CI_JOBS[MATRIX_JOB_ID].get("continue-on-error"))

    def test_self_test_runs_before_the_report(self) -> None:
        """Repo convention: a regression in the reporter's own logic must red HERE, not
        silently produce an empty green report."""
        runs = "\n".join(s["run"] for s in self.job["steps"] if "run" in s)
        self.assertIn("scripts/mutants-nightly-report.py --self-test", runs)
        self.assertLess(runs.index("--self-test"),
                        runs.index("--jobs jobs.ndjson"),
                        "the self-test must run BEFORE the real invocation")

    def test_both_halves_of_the_report_always_run(self) -> None:
        """The leg report and the whole-run ratchet verdict are independent signals. Under
        the default `bash -e`, a red leg report would abort the step and hide the ratchet
        verdict behind it — the same swallowing, one level up."""
        step = next(s for s in self.job["steps"]
                    if s.get("name", "").startswith("Report legs"))
        self.assertIn("set +e", step["run"])
        self.assertIn("scripts/mutants-gate.py --check", step["run"])

    def test_annotations_are_not_teed_into_the_summary(self) -> None:
        """GitHub honours `::error::` only from the step LOG. Piping the reporter through
        `tee -a $GITHUB_STEP_SUMMARY` would render the annotations as literal summary text
        and emit none — so the reporter is invoked with --summary instead."""
        step = next(s for s in self.job["steps"]
                    if s.get("name", "").startswith("Report legs"))
        reporter_call = step["run"].split("--jobs jobs.ndjson")[1].split("\n\n")[0]
        self.assertIn("--summary", reporter_call)
        self.assertNotIn("tee", reporter_call.split("mapfile")[0])

    def test_the_job_listing_is_paginated(self) -> None:
        """A 51-leg matrix plus every sibling in ci.yml is well past one API page. An
        unpaginated listing silently under-reports the failing legs — the precise failure
        mode this job exists to end."""
        step = next(s for s in self.job["steps"]
                    if s.get("name", "").startswith("List this run"))
        self.assertIn("--paginate", step["run"])


class TestReportBehaviour(unittest.TestCase):
    """Drive the REAL build_report() over the fixture shapes from #4820."""

    def setUp(self) -> None:
        self.P = report.DEFAULT_LEG_PREFIX
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def _artifact(self, name: str, pkg: str, planned: int, finished: int) -> None:
        d = Path(self.tmp.name) / f"mutants-outcomes-nightly-{name}"
        d.mkdir(parents=True, exist_ok=True)
        (d / "mutants.json").write_text(json.dumps([{"i": i} for i in range(planned)]))
        (d / "outcomes.json").write_text(json.dumps({"outcomes": [
            {"summary": "CaughtMutant", "scenario": {"Mutant": {"package": pkg}}}
        ] * finished}))

    def _legs(self, *specs) -> list:
        jobs = []
        for suffix, conclusion, mins in specs:
            jobs.append({"name": f"{self.P} ({suffix})", "conclusion": conclusion,
                         "started_at": "2026-09-03T00:00:00+00:00",
                         "completed_at": f"2026-09-03T{mins // 60:02d}:{mins % 60:02d}:00+00:00",
                         "html_url": "u"})
        return report.collect_legs(jobs, self.P)

    def test_control_a_healthy_night_is_green(self) -> None:
        """ANTI-VACUITY CONTROL. If this ever REDs, every assertion below is vacuous — a
        build_report() that returned 1 unconditionally would satisfy them all."""
        self._artifact("sparq-canon", "sparq-canon", planned=5, finished=5)
        ev = report.collect_evidence(self.tmp.name)
        _md, _ann, v = report.build_report(self._legs(("sparq-canon", "success", 45)),
                                          ev, self.P)
        self.assertEqual(v["exit_code"], 0, v)
        self.assertEqual((v["not_completed"], v["no_evidence"], v["bad_evidence"]),
                         (0, 0, 0), v)

    def test_a_timed_out_leg_is_counted_as_budget_not_as_a_mutation_finding(self) -> None:
        """The ten red legs of #4820. They must be counted, and counted as COMPLETION —
        a leg conclusion cannot carry a mutation finding (both the step and the job are
        continue-on-error), so calling these 'mutation failures' would be wrong."""
        legs = self._legs(("sparq-mpc, 360", "failure", 360))
        _md, ann, v = report.build_report(legs, {}, self.P)
        self.assertEqual(v["not_completed"], 1, v)
        self.assertEqual(v["no_evidence"], 1, v)
        self.assertEqual(v["exit_code"], 1, v)
        self.assertTrue(any("NOT mutation findings" in a for a in ann), ann)

    def test_a_green_leg_with_truncated_evidence_still_reds(self) -> None:
        """THE case the leg conclusions cannot express. cargo-mutants writes outcomes.json
        incrementally, so a leg cut short uploads a well-formed artifact and — because the
        cargo-mutants failure is swallowed by continue-on-error — can still conclude
        `success`. Reading only conclusions, that crate looks measured. It is not."""
        self._artifact("sparq-engine-shard-9-24", "sparq-engine", planned=400, finished=31)
        ev = report.collect_evidence(self.tmp.name)
        self.assertEqual(ev[("sparq-engine", "9/24")]["state"], "truncated", ev)
        legs = self._legs(("sparq-engine, 360, 9/24", "success", 359))
        _md, _ann, v = report.build_report(legs, ev, self.P)
        self.assertEqual(v["not_completed"], 0, "the leg concluded green — by design")
        self.assertEqual(v["bad_evidence"], 1, v)
        self.assertEqual(v["exit_code"], 1, v)

    def test_an_empty_leg_set_reds_rather_than_reporting_a_perfect_night(self) -> None:
        """The failure mode of a broken name join. Zero legs must never read as zero
        problems — that is how a dead lane stays invisible."""
        _md, ann, v = report.build_report([], {}, self.P)
        self.assertEqual(v["exit_code"], 1, v)
        self.assertTrue(any("found NO legs" in a for a in ann), ann)

    def test_billed_runner_hours_are_measured_from_the_legs(self) -> None:
        """The cost half. Every shard-count/timeout comment in the matrix says to re-tune
        from the run's own timings; this is the number that supplies them."""
        legs = self._legs(("sparq-canon", "success", 30),
                          ("sparq-mpc, 360", "failure", 360))
        _md, _ann, v = report.build_report(legs, {}, self.P)
        self.assertEqual(v["runner_hours"], 6.5, v)


class TestTheReportIsDeclaredNonGating(unittest.TestCase):
    """This reporter REDs by design. What makes that safe is a DECLARATION, never a
    `|| true` — the sq-huwr8 lesson, applied to a lane in ci.yml rather than an alarm."""

    def setUp(self) -> None:
        gate.load_advisory_registry(str(REGISTRY_PATH))

    def test_registry_key_matches_the_live_yaml_job_name(self) -> None:
        """C4's binding, asserted from this side too: a rename must not be able to
        silently re-arm the reporter as a merge blocker."""
        self.assertEqual(CI_JOBS[REPORT_JOB_ID]["name"], REPORT_JOB_NAME)
        entry = json.loads(REGISTRY_PATH.read_text())["jobs"][REPORT_JOB_NAME]
        self.assertEqual(entry["workflow"], "ci.yml")
        self.assertEqual(entry["job_id"], REPORT_JOB_ID)

    def test_report_failure_does_not_red_the_gate(self) -> None:
        """THE headline guard. Delete the registry entry and this REDs on
        render_verdict's exit code — a behaviour kill, not a crash kill."""
        rc = gate.render_verdict(_healthy_siblings() + [_run(REPORT_JOB_NAME, "failure")])
        self.assertEqual(
            rc, 0,
            f"a `failure` from {REPORT_JOB_NAME!r} red the gate (render_verdict returned "
            f"{rc}). It must be declared in .github/advisory-registry.json: it REDs "
            f"whenever a mutation leg does not complete, which is a nightly capacity "
            f"condition with nothing to do with the commit under test.",
        )

    def test_control_undeclared_lane_failure_does_red_the_gate(self) -> None:
        """ANTI-VACUITY CONTROL. Identical fixture shape, undeclared lane. If this ever
        goes green the assertion above is measuring nothing."""
        self.assertFalse(gate.is_advisory(UNDECLARED_LANE))
        rc = gate.render_verdict(_healthy_siblings() + [_run(UNDECLARED_LANE, "failure")])
        self.assertEqual(
            rc, 1,
            "CONTROL FAILED: an UNDECLARED lane concluding `failure` did not red the "
            "gate. The non-gating assertion above is therefore vacuous.",
        )

    def test_declaring_the_report_does_not_forgive_a_real_gating_failure(self) -> None:
        """QUANTIFIER DIRECTION. The declaration must make the REPORTER non-gating, not
        the commit's own legs — this is what proves it did not buy green by weakening the
        gate."""
        runs = (_healthy_siblings()
                + [_run(REPORT_JOB_NAME, "failure"), _run("build + test", "failure")])
        self.assertFalse(gate.is_advisory("build + test"))
        self.assertEqual(gate.render_verdict(runs), 1,
                         "a real gating leg's failure was forgiven — the gate was weakened")


class TestOwnCallSite(unittest.TestCase):
    """A suite that leaves CI's reachable set stops pinning anything."""

    def test_docs_quality_runs_this_suite_and_the_reporter_self_test(self) -> None:
        text = DOCS_QUALITY_YML.read_text()
        self.assertIn("scripts/tests/test_mutants_nightly_report.py", text,
                      "this suite is not invoked from docs-quality.yml")
        self.assertIn("scripts/mutants-nightly-report.py --self-test", text,
                      "the reporter's hermetic self-test is not invoked from "
                      "docs-quality.yml")


if __name__ == "__main__":
    unittest.main(verbosity=2)
