#!/usr/bin/env python3
# [SONNET-4.6] Hermetic unit tests for the EVENT-DRIVEN ci-summary status evaluator
# (scripts/ci_summary_status.py — bead sq-lfmvd, design record
# research/ci-gate-slotless-aggregation.md). 🤖 SPARQ agent.
#
# WHAT MUST NOT BREAK. The evaluator publishes the `ci-summary-status` commit status
# that is INTENDED to become the single required branch-protection check. Its verdict
# comes from the resident gate's own brain (ci_summary_gate.render_verdict et al.,
# already pinned by test_ci_summary_gate.py), so this suite deliberately does NOT
# re-test the gating semantics. It tests the TRANSPORT — the four places where a
# single short-lived observation could conclude something the 155-poll resident loop
# never would:
#
#   1. REGISTRATION FLOOR (design §3.6). An all-terminal green set observed before the
#      floor must publish PENDING, not SUCCESS — a workflow that has not registered
#      yet would otherwise be silently absent from the set. A RED is never floored.
#   2. THE REUSED HOLDS. A draft-tier-assembled selection with no full-tier successor,
#      and a missing feature-matrix reporter verdict, must hold at PENDING. The
#      draft-tier belt is armed by the TRIGGERING run's event, not this run's (which
#      is always `workflow_run`) — get that wrong and a draft head's REDUCED leg set
#      reads green, which is the one thing docs/branch-protection.md §Draft-tier CI
#      forbids.
#   3. AGGREGATOR SCOPING. The resident `gate` job is pending for as long as it polls;
#      if it stayed in the sibling set every shadow verdict would be a strictly-later
#      echo of the thing it exists to be compared against. Exclusion is by workflow
#      FILE PATH, so a RENAMED gate job is still excluded — asserted by renaming it.
#   4. MONOTONICITY. Evaluations are concurrent and unordered, so a slow one can POST
#      after a fast later one. A terminal status must never be downgraded to `pending`.
#
# HOW THIS SUITE IS BUILT TO FAIL. Every behaviour test drives the REAL
# ci_summary_status functions and, for the scoping tests, the REAL
# ci_summary_gate.WorkflowRunResolver over raw API-shaped fixtures — no
# re-implementation of either the verdict or the resolution lives here, so the tests
# cannot agree with a broken evaluator. TestAntiVacuityControls is the control: a
# genuinely failing sibling MUST come back FAILURE and an under-floor green MUST come
# back PENDING through the same paths, so a stub that returned one constant state
# could not pass this file.
#
# Fully hermetic: every fetcher and the status poster are injected (no gh, no network,
# no clock). Needs PyYAML for the wiring class only (already a docs-quality
# dependency).
# Run:  python3 scripts/tests/test_ci_summary_status.py

from __future__ import annotations

import importlib.util
import io
import os
import sys
import unittest
from contextlib import redirect_stdout
from datetime import datetime, timedelta, timezone
from pathlib import Path

try:
    import yaml
except ModuleNotFoundError:  # pragma: no cover - CI installs PyYAML explicitly
    yaml = None

sys.dont_write_bytecode = True  # keep repeated local runs hermetic (no stale .pyc)

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SCRIPTS = REPO_ROOT / "scripts"
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
EVALUATOR_YML = WORKFLOWS / "ci-summary-status.yml"
RESIDENT_YML = WORKFLOWS / "ci-summary.yml"
DOCS_QUALITY_YML = WORKFLOWS / "docs-quality.yml"


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


# The brain is loaded FIRST under its import name so the evaluator's
# `import ci_summary_gate` resolves to this same module object.
gate = _load("ci_summary_gate", SCRIPTS / "ci_summary_gate.py")
ev = _load("ci_summary_status", SCRIPTS / "ci_summary_status.py")

# Advisory status is DECLARED, never inferred (#3773) — declare exactly the fixtures
# that are meant to be non-gating here, as a real registry entry would.
DECLARED_ADVISORY = (
    "vale (prose, advisory)",
    "ci-summary status evaluator (advisory)",
)
gate.set_declared_advisory(DECLARED_ADVISORY)

NOW = datetime(2026, 7, 30, 12, 0, 0, tzinfo=timezone.utc)
SELECT_NAME = "select / select (change-based test selection)"


def R(name, status="completed", conclusion="success", url="", started="", rid=0,
      external_id=""):
    """A check-run in the vocabulary render_verdict judges (same shape as
    test_ci_summary_gate.R, so fixtures are comparable across the two suites)."""
    return {"name": name, "status": status, "conclusion": conclusion,
            "details_url": url, "html_url": "", "started_at": started, "id": rid,
            "external_id": external_id}


def W(run_id, workflow_id, *, path, name="CI", status="completed",
      conclusion="success", created="2026-07-30T11:00:00Z", attempt=1):
    """An Actions workflow-run as the list API returns it. `path` is load-bearing:
    it is the identity the aggregator filter keys on."""
    return {
        "id": run_id,
        "workflow_id": workflow_id,
        "name": name,
        "path": path,
        "head_sha": "deadbeef",
        "status": status,
        "conclusion": conclusion,
        "created_at": created,
        "run_started_at": created,
        "run_attempt": attempt,
        "html_url": f"https://github.test/o/r/actions/runs/{run_id}",
    }


def job_url(run_id: int, job_id: int = 1) -> str:
    """The details_url shape the resolver parses a run id out of."""
    return f"https://github.test/o/r/actions/runs/{run_id}/job/{job_id}"


GREEN = R("build + test")
GREEN2 = R("clippy")
QUEUED = R("coverage", status="queued", conclusion=None)
RED = R("clippy", conclusion="failure")

PAST_FLOOR = 600.0     # comfortably past the 180s registration floor
UNDER_FLOOR = 30.0


def cfg(**over):
    base = dict(registration_floor_seconds=180, absolute_budget_seconds=4020,
                summary_path="")
    base.update(over)
    return ev.EvalConfig(**base)


def decide(runs, *, elapsed=PAST_FLOOR, config=None, tier_ctx=None):
    """evaluate() with stdout captured (render_verdict prints its render)."""
    with redirect_stdout(io.StringIO()):
        return ev.evaluate(runs, elapsed=elapsed, cfg=config or cfg(), tier_ctx=tier_ctx)


def pr_tier():
    """The tier context a pull_request-triggered sibling completion produces."""
    return ev.tier_context_for("pull_request")


# ---------------------------------------------------------------------------------
# 1. the registration floor — the MIN_POLLS equivalent for a single observation
# ---------------------------------------------------------------------------------
class TestRegistrationFloor(unittest.TestCase):
    def test_all_green_past_the_floor_succeeds(self):
        self.assertEqual(decide([GREEN, GREEN2]).state, ev.SUCCESS)

    def test_all_green_under_the_floor_is_pending_not_success(self):
        """The load-bearing one: without this an evaluation that fires while only the
        two fastest workflows have registered would publish SUCCESS over a set that is
        missing everything slower."""
        decision = decide([GREEN, GREEN2], elapsed=UNDER_FLOOR)
        self.assertEqual(decision.state, ev.PENDING)
        self.assertIn("registration floor", decision.description)

    def test_a_red_is_never_floored(self):
        """A concluded gating failure in the authoritative newest run is never
        forgiven, so flooring it would only delay the red."""
        self.assertEqual(decide([GREEN, RED], elapsed=0.0).state, ev.FAILURE)

    def test_the_floor_is_read_from_the_config_not_hard_coded(self):
        self.assertEqual(
            decide([GREEN], elapsed=10.0, config=cfg(registration_floor_seconds=5)).state,
            ev.SUCCESS,
        )

    def test_an_empty_set_past_the_floor_passes(self):
        """Parity with the resident gate's stable-empty-set pass (a head that
        triggered no sibling workflow at all)."""
        self.assertEqual(decide([]).state, ev.SUCCESS)

    def test_an_empty_set_under_the_floor_holds(self):
        self.assertEqual(decide([], elapsed=UNDER_FLOOR).state, ev.PENDING)


# ---------------------------------------------------------------------------------
# 2. outstanding sets: pending, fail-fast, the reused holds, the absolute budget
# ---------------------------------------------------------------------------------
class TestOutstandingSets(unittest.TestCase):
    def test_a_pending_sibling_holds(self):
        decision = decide([GREEN, QUEUED])
        self.assertEqual(decision.state, ev.PENDING)
        self.assertIn("running", decision.description)

    def test_fail_fast_reds_while_siblings_are_still_running(self):
        decision = decide([QUEUED, RED])
        self.assertEqual(decision.state, ev.FAILURE)
        self.assertTrue(decision.failfast)
        self.assertIn("clippy", decision.description)

    def test_a_declared_advisory_failure_never_fails_fast(self):
        decision = decide([QUEUED, R("vale (prose, advisory)", conclusion="failure")])
        self.assertEqual(decision.state, ev.PENDING)

    def test_budget_exhaustion_renders_and_fails_closed(self):
        decision = decide([GREEN, QUEUED], elapsed=5000.0)
        self.assertEqual(decision.state, ev.FAILURE)
        self.assertIn("budget exhausted", decision.description)

    def test_budget_exhaustion_with_only_advisory_work_left_still_passes(self):
        """Reachable and correct: render_verdict judges only GATING checks, so an
        outstanding ADVISORY leg at budget exhaustion is a pass, not a red."""
        pending_advisory = R("vale (prose, advisory)", status="in_progress",
                             conclusion=None)
        self.assertEqual(decide([GREEN, pending_advisory], elapsed=5000.0).state,
                         ev.SUCCESS)


class TestReusedHolds(unittest.TestCase):
    """The two settle-window holds the resident gate applies, reused verbatim."""

    def test_a_draft_tier_selection_with_no_successor_holds_pending(self):
        runs = [R(SELECT_NAME + ", draft-tier", started="2026-07-30T11:00:00Z", rid=1),
                GREEN]
        decision = decide(runs, tier_ctx=pr_tier())
        self.assertEqual(decision.state, ev.PENDING)
        self.assertIn("full-tier", decision.description)

    def test_the_same_set_with_a_full_tier_successor_passes(self):
        runs = [R(SELECT_NAME + ", draft-tier", started="2026-07-30T11:00:00Z", rid=1),
                R(SELECT_NAME, started="2026-07-30T11:30:00Z", rid=2),
                GREEN]
        self.assertEqual(decide(runs, tier_ctx=pr_tier()).state, ev.SUCCESS)

    def test_the_draft_belt_is_armed_by_the_TRIGGERING_runs_event(self):
        """This evaluation's own trigger is always `workflow_run`; the belt keys on
        the event of the run whose completion fired it. Reading our own trigger
        instead would disarm the belt on every PR head."""
        self.assertEqual(ev.tier_context_for("pull_request").event_name, "pull_request")
        self.assertEqual(ev.tier_context_for("merge_group").event_name, "merge_group")
        self.assertEqual(ev.tier_context_for("push").event_name, "push")

    def test_the_evaluator_is_never_itself_a_draft_tier_verdict(self):
        """run_tier is fixed to full, so _draft_recheck (which would need a live PR
        read) can never fire — that is why no PR lookup is wired at all."""
        for event in ("pull_request", "push", "merge_group", ""):
            self.assertEqual(ev.tier_context_for(event).run_tier, "full")
            self.assertIsNone(ev.tier_context_for(event).fetch_pr_draft)

    def test_a_missing_feature_matrix_reporter_holds_pending(self):
        runs = [R("opt-in group (sparq-engine 1/2)", url=job_url(77)), GREEN]
        decision = decide(runs, tier_ctx=pr_tier())
        self.assertEqual(decision.state, ev.PENDING)
        self.assertIn("reporter", decision.description)

    def test_the_reporter_landing_green_releases_the_hold(self):
        runs = [R("opt-in group (sparq-engine 1/2)", url=job_url(77)),
                R(gate.FM_REPORT_NAME), GREEN]
        self.assertEqual(decide(runs, tier_ctx=pr_tier()).state, ev.SUCCESS)


# ---------------------------------------------------------------------------------
# 3. run_once: the grace re-fetch, the unobservable set, the re-dispatch refusal
# ---------------------------------------------------------------------------------
def scripted(*batches):
    """fetch_runs() stub: returns batches[i] on call i (an Exception is raised
    instead); repeats the last batch once exhausted."""
    state = {"i": 0, "calls": 0}

    def fetch():
        state["calls"] += 1
        entry = batches[min(state["i"], len(batches) - 1)]
        state["i"] += 1
        if isinstance(entry, Exception):
            raise entry
        return list(entry)

    fetch.state = state
    return fetch


def once(fetch, *, elapsed=PAST_FLOOR, config=None, tier_ctx=None):
    with redirect_stdout(io.StringIO()):
        return ev.run_once(fetch, cfg=config or cfg(), elapsed_of=lambda: elapsed,
                           tier_ctx=tier_ctx)


class TestRunOnce(unittest.TestCase):
    def test_a_fail_fast_red_must_be_re_observed_before_it_is_published(self):
        """The resident gate's grace re-poll, ported: a red seen once is confirmed on
        a fresh fetch. Here the second observation shows the leg green (an
        API-read race), so NO red is published."""
        fetch = scripted([QUEUED, RED], [GREEN, GREEN2])
        decision = once(fetch)
        self.assertEqual(decision.state, ev.SUCCESS)
        self.assertEqual(fetch.state["calls"], 2, "the grace re-fetch did not happen")

    def test_a_re_observed_red_is_published(self):
        fetch = scripted([QUEUED, RED], [QUEUED, RED])
        self.assertEqual(once(fetch).state, ev.FAILURE)

    def test_a_settled_verdict_costs_exactly_one_fetch(self):
        """No grace re-fetch on a non-fail-fast decision — the whole point is that one
        evaluation is cheap."""
        fetch = scripted([GREEN, GREEN2])
        self.assertEqual(once(fetch).state, ev.SUCCESS)
        self.assertEqual(fetch.state["calls"], 1)

    def test_an_unobservable_set_publishes_nothing(self):
        """"I cannot see" is not a verdict: the previous status must stand."""
        self.assertIsNone(once(scripted(gate.FetchError("api down"))))

    def test_a_cancelled_newest_run_is_reported_pending_not_red(self):
        """(G-redispatch) The evaluator holds no `actions: write`; the resident gate
        owns the once-only re-run POST, so this state is a WAIT, not a hang."""
        decision = once(scripted(gate.SupersededLegsError("superseded-legs")))
        self.assertEqual(decision.state, ev.PENDING)
        self.assertIn("re-dispatch", decision.description)

    def test_a_cancelled_newest_run_past_the_budget_reds(self):
        decision = once(scripted(gate.SupersededLegsError("superseded-legs")),
                        elapsed=5000.0)
        self.assertEqual(decision.state, ev.FAILURE)

    def test_a_stale_draft_tier_gate_artifact_is_not_a_leg(self):
        """A completed `gate, draft-tier` FAILURE on the SHA is a superseded
        aggregator verdict; counting it would permanently RED the head."""
        artifact = R(gate.DRAFT_TIER_GATE_NAME, conclusion="failure")
        self.assertEqual(once(scripted([GREEN, artifact])).state, ev.SUCCESS)

    def test_a_sibling_job_literally_named_gate_still_GATES(self):
        """The full-tier `gate` name is deliberately NOT name-excluded (see
        is_draft_gate_artifact's docstring): a future sibling job called `gate` in
        another workflow must keep gating. Path-based exclusion is what removes the
        resident gate's own check-runs."""
        self.assertEqual(
            once(scripted([GREEN, R("gate", conclusion="failure")])).state, ev.FAILURE
        )


# ---------------------------------------------------------------------------------
# 4. aggregator scoping — driven through the REAL WorkflowRunResolver
# ---------------------------------------------------------------------------------
CI_PATH = ".github/workflows/ci.yml"


def J(job_id, name, *, status="completed", conclusion="success", run_id=11):
    """An Actions JOB as the attempt-scoped Jobs API returns it. A COMPLETED workflow
    run's legs come from that API, not from its commit check-runs (#3505), so a fixture
    for a finished workflow has to supply them or the resolver correctly treats every
    check as evaporated."""
    return {"id": job_id, "name": name, "status": status, "conclusion": conclusion,
            "started_at": "2026-07-30T11:05:00Z", "completed_at": "2026-07-30T11:20:00Z",
            "html_url": job_url(run_id, job_id)}


def resolver_for(checks, workflows, *, self_run_id="900", attempt_jobs=None):
    """Build the evaluator's fetch_runs over RAW API-shaped fixtures, using the real
    resolver. Returns (fetch_runs, origin_of)."""
    return ev.make_fetch_runs(
        "o/r",
        "deadbeef",
        self_run_id,
        raw_checks=lambda: [dict(c) for c in checks],
        raw_workflows=lambda: [dict(w) for w in workflows],
        fetch_attempt_jobs=lambda run_id, attempt: list((attempt_jobs or {}).get(run_id, [])),
    )


class TestAggregatorScoping(unittest.TestCase):
    """The resident gate polls for tens of minutes; if its own in-flight check-run
    stayed in the set, every shadow verdict would just echo it later."""

    CI_RUN = W(11, 1, path=CI_PATH, name="CI")
    GATE_RUN = W(12, 2, path=ev.RESIDENT_GATE_WORKFLOW_PATH, name="ci-summary",
                 status="in_progress", conclusion=None)
    SELF_RUN = W(900, 3, path=ev.EVALUATOR_WORKFLOW_PATH, name="ci-summary-status",
                 status="in_progress", conclusion=None)

    # The CI run is COMPLETED, so its one leg is served by the Jobs API.
    CI_JOBS = {11: [J(1, "build + test")]}
    CI_JOBS_RED = {11: [J(1, "clippy", conclusion="failure")]}

    def _decide(self, checks, workflows, **kw):
        kw.setdefault("attempt_jobs", self.CI_JOBS)
        fetch, origin = resolver_for(checks, workflows, **kw)
        with redirect_stdout(io.StringIO()):
            runs = ev._scoped(fetch())
            decision = ev.evaluate(runs, elapsed=PAST_FLOOR, cfg=cfg())
        return decision, runs, origin()

    def test_the_resident_gates_in_flight_check_is_excluded(self):
        checks = [R("gate", status="in_progress", conclusion=None, url=job_url(12))]
        decision, runs, _ = self._decide(checks, [self.CI_RUN, self.GATE_RUN])
        names = {r["name"] for r in runs}
        self.assertNotIn("gate", names)
        self.assertEqual(names, {"build + test"})
        self.assertEqual(decision.state, ev.SUCCESS)

    def test_including_it_would_have_held_the_verdict(self):
        """Anti-vacuity for the test above: the SAME fixture with the gate's workflow
        path unrecognised holds at PENDING, so the exclusion is what produced the
        SUCCESS — not an accidentally-green fixture."""
        not_an_aggregator = dict(self.GATE_RUN, path=".github/workflows/other.yml")
        checks = [R("gate", status="in_progress", conclusion=None, url=job_url(12))]
        decision, runs, _ = self._decide(checks, [self.CI_RUN, not_an_aggregator])
        self.assertIn("gate", {r["name"] for r in runs})
        self.assertEqual(decision.state, ev.PENDING)

    def test_exclusion_survives_a_RENAMED_gate_job(self):
        """Keyed on the workflow FILE PATH, so renaming the job cannot re-admit it."""
        checks = [R("aggregate", status="in_progress", conclusion=None, url=job_url(12))]
        decision, runs, _ = self._decide(checks, [self.CI_RUN, self.GATE_RUN])
        self.assertEqual({r["name"] for r in runs}, {"build + test"})
        self.assertEqual(decision.state, ev.SUCCESS)

    def test_an_earlier_evaluation_of_the_same_head_is_excluded(self):
        """The resolver drops the whole self-WORKFLOW by run id, so a previous
        evaluator run cannot make this evaluation wait on itself."""
        earlier = W(899, 3, path=ev.EVALUATOR_WORKFLOW_PATH, name="ci-summary-status",
                    status="in_progress", conclusion=None)
        checks = [
            R("ci-summary status evaluator (advisory)", status="in_progress",
              conclusion=None, url=job_url(899)),
        ]
        decision, runs, _ = self._decide(checks, [self.CI_RUN, self.SELF_RUN, earlier])
        self.assertEqual({r["name"] for r in runs}, {"build + test"})
        self.assertEqual(decision.state, ev.SUCCESS)

    def test_a_real_sibling_failure_is_never_scoped_away(self):
        """The control: scoping removes aggregators ONLY."""
        red_ci = dict(self.CI_RUN, conclusion="failure")
        checks = [R("gate", status="in_progress", conclusion=None, url=job_url(12))]
        decision, runs, _ = self._decide(checks, [red_ci, self.GATE_RUN],
                                         attempt_jobs=self.CI_JOBS_RED)
        self.assertEqual({r["name"] for r in runs}, {"clippy"})
        self.assertEqual(decision.state, ev.FAILURE)

    def test_the_elapsed_origin_counts_the_aggregators_too(self):
        """CI began on this head when the FIRST run — resident gate included — was
        created, even though that run is not judged."""
        early_gate = dict(self.GATE_RUN, created_at="2026-07-30T10:00:00Z",
                          run_started_at="2026-07-30T10:00:00Z")
        _, _, origin = self._decide([], [self.CI_RUN, early_gate])
        self.assertEqual(origin, "2026-07-30T10:00:00Z")

    def test_pure_helpers(self):
        self.assertTrue(ev.is_aggregator_workflow(self.GATE_RUN))
        self.assertTrue(ev.is_aggregator_workflow(self.SELF_RUN))
        self.assertFalse(ev.is_aggregator_workflow(self.CI_RUN))
        self.assertEqual(
            ev.aggregator_run_ids([self.CI_RUN, self.GATE_RUN, self.SELF_RUN]),
            {12, 900},
        )
        self.assertEqual(ev.aggregator_run_ids([dict(self.GATE_RUN, id=0)]), set())


# ---------------------------------------------------------------------------------
# 5. the elapsed clock
# ---------------------------------------------------------------------------------
class TestElapsedClock(unittest.TestCase):
    def test_earliest_created_at_picks_the_minimum(self):
        runs = [W(2, 1, path=CI_PATH, created="2026-07-30T11:30:00Z"),
                W(1, 2, path=CI_PATH, created="2026-07-30T11:00:00Z")]
        self.assertEqual(ev.earliest_created_at(runs), "2026-07-30T11:00:00Z")

    def test_no_runs_means_no_origin(self):
        self.assertEqual(ev.earliest_created_at([]), "")

    def test_a_run_with_no_created_at_falls_back_to_run_started_at(self):
        run = dict(W(1, 1, path=CI_PATH), created_at="")
        self.assertEqual(ev.earliest_created_at([run]), run["run_started_at"])

    def test_elapsed_is_measured_from_the_origin(self):
        self.assertAlmostEqual(
            ev.elapsed_seconds("2026-07-30T11:00:00Z", NOW), 3600.0, places=3
        )

    def test_an_unknown_or_unparseable_origin_keeps_the_floor_unmet(self):
        for origin in ("", "not-a-timestamp"):
            with self.subTest(origin=origin):
                self.assertEqual(ev.elapsed_seconds(origin, NOW), 0.0)
                self.assertEqual(
                    decide([GREEN], elapsed=ev.elapsed_seconds(origin, NOW)).state,
                    ev.PENDING,
                )

    def test_a_future_origin_clamps_to_zero_rather_than_going_negative(self):
        future = (NOW + timedelta(minutes=5)).isoformat().replace("+00:00", "Z")
        self.assertEqual(ev.elapsed_seconds(future, NOW), 0.0)

    def test_a_naive_timestamp_is_read_as_utc(self):
        self.assertAlmostEqual(
            ev.elapsed_seconds("2026-07-30T11:00:00", NOW), 3600.0, places=3
        )


# ---------------------------------------------------------------------------------
# 6. publication: monotonicity + idempotence
# ---------------------------------------------------------------------------------
class TestPublish(unittest.TestCase):
    def setUp(self):
        self.posted: list[tuple[str, str]] = []

    def _publish(self, decision, current):
        return ev.publish(decision, current=current,
                          post_status=lambda s, d: self.posted.append((s, d)))

    def test_a_first_status_is_posted(self):
        self.assertEqual(
            self._publish(ev.Decision(ev.PENDING, "waiting"), None), "posted"
        )
        self.assertEqual(self.posted, [(ev.PENDING, "waiting")])

    def test_pending_never_downgrades_a_published_success(self):
        """The reordering guard: a slow evaluation that observed an EARLIER state must
        not leave a green head reading `pending` forever."""
        action = self._publish(ev.Decision(ev.PENDING, "waiting"),
                               (ev.SUCCESS, "PASS — 40 gating check(s) green"))
        self.assertEqual(action, "skipped-no-downgrade")
        self.assertEqual(self.posted, [])

    def test_pending_never_downgrades_a_published_failure(self):
        action = self._publish(ev.Decision(ev.PENDING, "waiting"),
                               (ev.FAILURE, "FAIL — 1 gating check(s) not passing"))
        self.assertEqual(action, "skipped-no-downgrade")
        self.assertEqual(self.posted, [])

    def test_a_failure_may_overwrite_a_success(self):
        action = self._publish(ev.Decision(ev.FAILURE, "FAIL — clippy"),
                               (ev.SUCCESS, "PASS"))
        self.assertEqual(action, "posted")

    def test_a_success_may_overwrite_a_failure(self):
        """Re-run recovery: the newest attempt is authoritative (#3505)."""
        action = self._publish(ev.Decision(ev.SUCCESS, "PASS"), (ev.FAILURE, "FAIL"))
        self.assertEqual(action, "posted")

    def test_an_identical_status_is_not_re_posted(self):
        action = self._publish(ev.Decision(ev.PENDING, "waiting — 3 of 40 running"),
                               (ev.PENDING, "waiting — 3 of 40 running"))
        self.assertEqual(action, "skipped-identical")
        self.assertEqual(self.posted, [])

    def test_a_changed_pending_description_IS_re_posted(self):
        action = self._publish(ev.Decision(ev.PENDING, "waiting — 1 of 40 running"),
                               (ev.PENDING, "waiting — 3 of 40 running"))
        self.assertEqual(action, "posted")


class TestDescriptionBudget(unittest.TestCase):
    """GitHub truncates a status description at 140 chars; do it visibly and never
    emit one that would be silently cut."""

    def test_truncate_marks_the_cut(self):
        out = ev.truncate("x" * 200)
        self.assertEqual(len(out), ev.DESCRIPTION_LIMIT)
        self.assertTrue(out.endswith("…"))

    def test_a_short_description_is_untouched(self):
        self.assertEqual(ev.truncate("PASS"), "PASS")

    def test_every_decision_path_stays_within_the_limit(self):
        many_reds = [R(f"very long gating leg name number {i}", conclusion="failure")
                     for i in range(40)]
        for label, decision in (
            ("all-terminal red", decide(many_reds)),
            ("fail-fast", decide(many_reds + [QUEUED])),
            ("pending", decide([QUEUED] * 40)),
            ("floor", decide([GREEN] * 40, elapsed=UNDER_FLOOR)),
            ("success", decide([GREEN] * 40)),
            ("budget", decide([QUEUED] * 40, elapsed=5000.0)),
        ):
            with self.subTest(path=label):
                self.assertLessEqual(len(decision.description), ev.DESCRIPTION_LIMIT)


# ---------------------------------------------------------------------------------
# 7. wiring: the trust model, the staging invariant, the reachability of this file
# ---------------------------------------------------------------------------------
@unittest.skipIf(yaml is None, "PyYAML unavailable (CI installs it; see __main__)")
class TestWorkflowWiring(unittest.TestCase):
    """YAML `if:`/trigger shapes cannot be unit-tested by execution, so pin their
    SHAPE — these are the properties the security and staging arguments rest on."""

    @classmethod
    def setUpClass(cls):
        cls.wf = yaml.safe_load(EVALUATOR_YML.read_text(encoding="utf-8"))
        # PyYAML parses the unquoted key `on:` as the boolean True.
        cls.on = cls.wf.get("on", cls.wf.get(True))
        cls.job = cls.wf["jobs"]["evaluate"]

    def test_there_is_NO_pull_request_trigger(self):
        """THE trust invariant (#3474 class). A pull_request-triggered job runs the PR
        HEAD's copy of this workflow and script while holding `statuses: write` — i.e.
        a PR could forge a green status for the context destined to be REQUIRED.
        Every trigger here must execute the DEFAULT-BRANCH copy."""
        self.assertNotIn("pull_request", self.on)
        self.assertNotIn("pull_request_target", self.on)
        self.assertEqual(set(self.on), {"workflow_run", "workflow_dispatch"})

    def test_it_subscribes_to_every_workflows_completion(self):
        """No `workflows:` name filter: enumerating names would rebuild exactly the
        brittleness the aggregator exists to remove."""
        self.assertEqual(self.on["workflow_run"]["types"], ["completed"])
        self.assertNotIn("workflows", self.on["workflow_run"])

    def test_it_stands_down_on_its_own_completion(self):
        """Subscribing to all workflows includes itself; an evaluation must not chain
        off its own completion."""
        guard = self.job["if"]
        self.assertIn("github.event.workflow_run.name != 'ci-summary-status'", guard)
        self.assertEqual(self.wf["name"], "ci-summary-status",
                         "the stand-down guard compares against this workflow's name")

    def test_the_token_holds_exactly_one_write_capability(self):
        self.assertEqual(
            self.wf["permissions"],
            {"checks": "read", "statuses": "write", "contents": "read",
             "actions": "read"},
        )

    def test_it_cannot_redispatch_anything(self):
        """(G-redispatch) is a DELIBERATE limitation, not an oversight: `actions` is
        read-only, and the script's refusal path is what surfaces it."""
        self.assertEqual(self.wf["permissions"]["actions"], "read")
        self.assertRaises(gate.FetchError, ev._refuse_redispatch, 123)

    def test_the_job_name_is_the_registry_key_verbatim(self):
        """C4 binds the advisory declaration to (workflow file, job id) and compares
        the key to this literal, so a rename REDs check-advisory-registry.py instead
        of silently re-arming a shadow lane as a merge blocker."""
        self.assertEqual(self.job["name"], "ci-summary status evaluator (advisory)")
        self.assertNotIn("${{", self.job["name"])

    def test_it_is_debounced_per_head_sha(self):
        group = self.wf["concurrency"]["group"]
        self.assertIn("github.event.workflow_run.head_sha", group)
        self.assertTrue(self.wf["concurrency"]["cancel-in-progress"])

    def test_it_is_structurally_not_a_waiter(self):
        """A minute-scale cap is the proof the slot occupancy is gone; the resident
        gate's own cap is 80 minutes."""
        self.assertLessEqual(self.job["timeout-minutes"], 10)

    def test_the_sparse_checkout_carries_the_whole_import_graph(self):
        """ci_summary_status.py imports ci_summary_gate, which reads the registry —
        the #3434 class of failure (a runner given a script it cannot import)."""
        listed = set()
        for step in self.job["steps"]:
            raw = (step.get("with") or {}).get("sparse-checkout")
            if raw:
                listed |= {line.strip() for line in str(raw).split() if line.strip()}
        self.assertEqual(
            listed,
            {"scripts/ci_summary_gate.py", "scripts/ci_summary_status.py",
             ".github/advisory-registry.json"},
        )

    def test_the_triggering_runs_event_is_what_reaches_the_script(self):
        """Not this run's event (always `workflow_run`) — see
        TestReusedHolds.test_the_draft_belt_is_armed_by_the_TRIGGERING_runs_event."""
        env = {}
        for step in self.job["steps"]:
            env.update(step.get("env") or {})
        self.assertIn("github.event.workflow_run.event", env["TRIGGER_EVENT"])
        self.assertIn("github.event.workflow_run.head_sha", env["SHA"])
        self.assertEqual(env["SELF_RUN_ID"], "${{ github.run_id }}")

    def test_the_resident_gate_is_STAGED_ALONGSIDE_not_swapped(self):
        """design §3.1: a missing "expected" required check blocks every merge, so the
        resident `gate` job must still exist and still render the required context.
        Dropping it is a separate, maintainer-signed step."""
        resident = yaml.safe_load(RESIDENT_YML.read_text(encoding="utf-8"))
        name = resident["jobs"]["gate"]["name"]
        self.assertTrue(name.startswith(gate.GATE_CHECK_NAME), name)

    def test_this_suite_is_reachable_from_CI(self):
        """A test file nobody runs is not a gate. Pin the call site, as the sibling
        suites do."""
        self.assertIn(
            "python3 scripts/tests/test_ci_summary_status.py",
            DOCS_QUALITY_YML.read_text(encoding="utf-8"),
        )


# ---------------------------------------------------------------------------------
# 8. anti-vacuity controls
# ---------------------------------------------------------------------------------
class TestAntiVacuityControls(unittest.TestCase):
    """If these ever go green in the wrong direction, this file is measuring
    nothing: the evaluator would be returning a constant."""

    def test_the_three_states_are_all_reachable_through_evaluate(self):
        self.assertEqual(
            [decide([GREEN, RED]).state,
             decide([GREEN, QUEUED]).state,
             decide([GREEN, GREEN2]).state],
            [ev.FAILURE, ev.PENDING, ev.SUCCESS],
        )

    def test_the_verdict_comes_from_the_real_brain(self):
        """An UNDECLARED advisory-token name still GATES (#3773) — proof that
        is_advisory, not a local name rule, is what decided the exclusion."""
        undeclared = R("site determinism grep-gate (advisory)", conclusion="failure")
        self.assertEqual(decide([GREEN, undeclared]).state, ev.FAILURE)
        declared = R("vale (prose, advisory)", conclusion="failure")
        self.assertEqual(decide([GREEN, declared]).state, ev.SUCCESS)

    def test_the_status_context_is_neither_of_the_check_run_names(self):
        """It must be unmistakable in the ruleset UI while both exist."""
        self.assertEqual(ev.STATUS_CONTEXT, "ci-summary-status")
        self.assertNotIn(ev.STATUS_CONTEXT,
                         {gate.GATE_CHECK_NAME, gate.DRAFT_TIER_GATE_NAME,
                          "ci-summary / gate"})


if __name__ == "__main__":
    # The wiring class SKIPS without PyYAML so the behaviour tests stay runnable on a
    # bare interpreter — but a skip inside CI would be a silent hole in the trust-model
    # pins (no pull_request trigger, one write capability, staged-not-swapped). CI
    # installs PyYAML explicitly (docs-quality.yml), so its absence THERE is a defect,
    # not a degradation: fail loudly instead of skipping.
    if yaml is None and os.environ.get("GITHUB_ACTIONS") == "true":
        print(
            "::error::test_ci_summary_status: PyYAML is missing, so the workflow-wiring "
            "pins (trust model + staged migration) did not run. Restore the "
            "docs-quality.yml PyYAML install step."
        )
        sys.exit(1)
    unittest.main(verbosity=2)
