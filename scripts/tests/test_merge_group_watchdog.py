#!/usr/bin/env python3
# [OPUS-5] sparq-org/sparq#4652 — the zero-dispatch merge-group watchdog test suite.
#
# Three layers, because on THIS repo every uncaught mutant in a recent sweep lived at
# the YAML seam rather than in the Python:
#
#   1. POLICY (pure) — decide_entry / classify_dequeue_route decision tables. Every
#      arm, and for the two headline claims a paired CONTROL that must NOT fire.
#   2. BEHAVIOUR (fake gh) — drive the real Watchdog.sweep() against a scripted gh and
#      assert on the MUTATIONS ISSUED, not just the log text. A watchdog that logs
#      "RECOVER" while issuing no mutation, or that mutates on the control input, dies
#      here.
#   3. WIRING (YAML inspection) — the `if:` expressions, the step list, and the CALL
#      SITE argv of both workflows. YAML `if:` cannot be executed in a unit test, so
#      its SHAPE is pinned instead: delete the classify step, invert the fail-safe
#      `!= 'preserve'` to `== 'demote'`, drop `sweep` from the call site, or add a
#      pull_request trigger to the cron watchdog, and a test here goes red.
#
# Hermetic: no gh, no network, no git. Needs PyYAML (already a docs-quality dep).
# Run:  python3 scripts/tests/test_merge_group_watchdog.py

from __future__ import annotations

import importlib.util
import json
import re
import sys
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SCRIPTS = REPO_ROOT / "scripts"
FEEDBACK_YML = REPO_ROOT / ".github" / "workflows" / "merge-queue-feedback.yml"
WATCHDOG_YML = REPO_ROOT / ".github" / "workflows" / "merge-group-watchdog.yml"
DOCS_QUALITY_YML = REPO_ROOT / ".github" / "workflows" / "docs-quality.yml"


def _load(module_name: str, filename: str):
    sys.path.insert(0, str(SCRIPTS))
    spec = importlib.util.spec_from_file_location(module_name, SCRIPTS / filename)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    spec.loader.exec_module(module)
    return module


mgw = _load("merge_group_watchdog", "merge-group-watchdog.py")

# The real incident's identifiers (#4652), so the fixtures are not invented shapes.
BASE = "3cc1bf828c1335577069f5fa65d832c0ae1c8c38"
HEAD = "1bfb0174f5cc2da1ed9dfe7997b7ab089e7cab26"
REF = f"gh-readonly-queue/main/pr-4534-{BASE}"
BUILT = mgw.parse_iso("2026-07-27T23:04:37Z")  # measured branch_creation timestamp
NOW = mgw.parse_iso("2026-07-27T23:20:00Z")


def entry(**overrides) -> "mgw.QueueEntry":
    fields = dict(
        pr_number=4534,
        pr_id="PR_4534",
        entry_id="MQE_4534",
        position=1,
        state="AWAITING_CHECKS",
        enqueued_at=mgw.parse_iso("2026-07-27T22:58:53Z"),
        base_oid=BASE,
        head_oid=HEAD,
    )
    fields.update(overrides)
    return mgw.QueueEntry(**fields)


def decide(**overrides) -> "mgw.Decision":
    kwargs = dict(
        suites=0,
        created_at=BUILT,
        now=NOW,
        grace_seconds=mgw.DEFAULT_GRACE_SECONDS,
        markers=(),
        recoveries_in_window=0,
        max_recoveries_per_pr=mgw.DEFAULT_MAX_RECOVERIES_PER_PR,
        run_recoveries=0,
        max_recoveries_per_run=mgw.DEFAULT_MAX_RECOVERIES_PER_RUN,
    )
    subject = overrides.pop("entry", entry())
    kwargs.update(overrides)
    return mgw.decide_entry(subject, **kwargs)


def marker(**overrides) -> "mgw.Marker":
    fields = dict(
        pr=4534,
        head=HEAD,
        base=BASE,
        ref=REF,
        suites=0,
        observed=NOW - timedelta(minutes=5),
        action="re-enqueue",
    )
    fields.update(overrides)
    return mgw.Marker(**fields)


# ── 1. policy ────────────────────────────────────────────────────────────────────


class TestZeroSuiteDetection(unittest.TestCase):
    """The headline claim, and the control that stops it firing on everything."""

    def test_zero_suites_past_grace_is_recovered(self):
        # THE RED TEST: the exact #4534 shape — group ref built, zero check-suites,
        # well past the grace period — must be detected and recovered.
        self.assertEqual(decide().verdict, mgw.RECOVER)

    def test_control_suites_present_runs_pending_is_never_touched(self):
        # THE CONTROL: a perfectly healthy group whose check-runs are simply still
        # pending has SUITES. Without this the detector could pass by firing on
        # everything. Age is irrelevant here — even an hour-old pending group holds.
        for suites in (1, 8, 40):
            for age in (0, mgw.DEFAULT_GRACE_SECONDS * 100):
                verdict = decide(suites=suites, now=BUILT + timedelta(seconds=age)).verdict
                self.assertEqual(verdict, mgw.HOLD, (suites, age))

    def test_zero_check_runs_is_not_the_predicate(self):
        # A paths-filtered workflow that matches nothing still creates a check-SUITE
        # and produces no run. The detector must key on the suite count only; a
        # suites=N/runs=0 group is a HOLD.
        self.assertEqual(decide(suites=3).verdict, mgw.HOLD)

    def test_grace_boundary(self):
        self.assertEqual(decide(now=BUILT + timedelta(seconds=0)).verdict, mgw.WAIT)
        self.assertEqual(
            decide(now=BUILT + timedelta(seconds=mgw.DEFAULT_GRACE_SECONDS - 1)).verdict,
            mgw.WAIT,
        )
        self.assertEqual(
            decide(now=BUILT + timedelta(seconds=mgw.DEFAULT_GRACE_SECONDS)).verdict,
            mgw.RECOVER,
        )

    def test_grace_exceeds_measured_dispatch_latency_by_two_orders(self):
        # Measured create->first-suite latency on this repo is single-digit SECONDS
        # (repository activity branch_creation -> earliest check-suite created_at).
        # Pin a floor so nobody "tunes" the grace down to the noise band.
        self.assertGreaterEqual(mgw.DEFAULT_GRACE_SECONDS, 300)

    def test_only_awaiting_checks_is_actionable(self):
        self.assertEqual(mgw.ACTIONABLE_STATES, frozenset({"AWAITING_CHECKS"}))
        for state in ("QUEUED", "MERGEABLE", "UNMERGEABLE", "LOCKED"):
            self.assertEqual(decide(entry=entry(state=state)).verdict, mgw.SKIP, state)

    def test_entry_without_a_built_group_is_skipped(self):
        self.assertEqual(decide(entry=entry(head_oid=None)).verdict, mgw.SKIP)
        self.assertEqual(decide(entry=entry(base_oid=None)).verdict, mgw.SKIP)


class TestFailSafe(unittest.TestCase):
    """Anything not POSITIVELY established must be a refusal, never a recovery."""

    def test_unknown_suite_count_refuses(self):
        self.assertEqual(decide(suites=None).verdict, mgw.REFUSE)

    def test_missing_branch_creation_anchor_refuses(self):
        self.assertEqual(decide(created_at=None).verdict, mgw.REFUSE)

    def test_future_creation_time_refuses(self):
        self.assertEqual(decide(now=BUILT - timedelta(minutes=1)).verdict, mgw.REFUSE)

    def test_check_suite_count_returns_none_on_every_unreadable_shape(self):
        watchdog = FakeWatchdog.build()
        for payload in (
            "not json",
            "[]",
            json.dumps({"check_suites": []}),
            json.dumps({"total_count": 0}),
            json.dumps({"total_count": "0", "check_suites": []}),
            # total_count says zero but the array is non-empty: a garbled response must
            # never be mistaken for a positively-observed zero.
            json.dumps({"total_count": 0, "check_suites": [{"id": 1}]}),
        ):
            watchdog.gh.suites_raw = payload
            self.assertIsNone(watchdog.watchdog.check_suite_count(HEAD), payload)

    def test_positive_zero_is_returned(self):
        watchdog = FakeWatchdog.build()
        watchdog.gh.suites_raw = json.dumps({"total_count": 0, "check_suites": []})
        self.assertEqual(watchdog.watchdog.check_suite_count(HEAD), 0)


class TestCapAndIdempotence(unittest.TestCase):
    def test_repeat_detection_on_the_same_ref_is_a_noop(self):
        self.assertEqual(decide(markers=(marker(),)).verdict, mgw.NOOP)

    def test_a_marker_for_a_different_head_does_not_suppress_detection(self):
        self.assertEqual(
            decide(markers=(marker(head="b" * 40),), recoveries_in_window=1).verdict,
            mgw.RECOVER,
        )

    def test_per_pr_cap(self):
        self.assertEqual(decide(recoveries_in_window=1).verdict, mgw.RECOVER)
        self.assertEqual(decide(recoveries_in_window=2).verdict, mgw.CAP)
        self.assertEqual(decide(recoveries_in_window=99).verdict, mgw.CAP)
        self.assertEqual(mgw.DEFAULT_MAX_RECOVERIES_PER_PR, 2)

    def test_per_run_cap(self):
        self.assertEqual(decide(run_recoveries=1).verdict, mgw.RECOVER)
        self.assertEqual(decide(run_recoveries=2).verdict, mgw.CAP)

    def test_cap_is_a_refusal_not_a_mutation(self):
        # Exhaustion hands back to the platform CI_TIMEOUT; it must not act.
        harness = FakeWatchdog.build(comments=[_marker_comment(head="c" * 40),
                                               _marker_comment(head="d" * 40)])
        harness.run()
        self.assertEqual(harness.gh.mutations, [])
        self.assertTrue(any("decision=CAP" in row for row in harness.rows), harness.rows)


class TestDequeueRouting(unittest.TestCase):
    """Both arms of the CI_TIMEOUT split, keyed on the suite count."""

    def route(self, **overrides) -> "mgw.Route":
        kwargs = dict(
            reason="CI_TIMEOUT",
            markers=(marker(),),
            last_enqueued_at=NOW - timedelta(hours=1),
            live_suites=lambda _sha: 0,
            now=NOW + timedelta(minutes=40),
        )
        kwargs.update(overrides)
        return mgw.classify_dequeue_route(**kwargs)

    def test_arm_a_ci_timeout_with_zero_suites_preserves_the_verdict(self):
        result = self.route()
        self.assertEqual(result.route, mgw.ROUTE_PRESERVE)
        self.assertTrue(result.reenqueue)

    def test_arm_b_ci_timeout_with_suites_present_still_demotes(self):
        # The legitimate signal: checks genuinely ran and ran long. Keep demoting.
        self.assertEqual(self.route(live_suites=lambda _s: 8).route, mgw.ROUTE_DEMOTE)
        self.assertEqual(self.route(live_suites=lambda _s: 1).route, mgw.ROUTE_DEMOTE)

    def test_the_split_is_the_suite_count_not_the_reason(self):
        # Same reason, opposite outcome, purely because the suite count differs.
        zero = self.route(reason="CI_TIMEOUT", live_suites=lambda _s: 0)
        many = self.route(reason="CI_TIMEOUT", live_suites=lambda _s: 8)
        self.assertEqual(zero.route, mgw.ROUTE_PRESERVE)
        self.assertEqual(many.route, mgw.ROUTE_DEMOTE)

    def test_unreadable_suite_count_demotes(self):
        self.assertEqual(self.route(live_suites=lambda _s: None).route, mgw.ROUTE_DEMOTE)

    def test_no_marker_demotes(self):
        self.assertEqual(self.route(markers=()).route, mgw.ROUTE_DEMOTE)

    def test_marker_predating_this_queue_attempt_demotes(self):
        self.assertEqual(
            self.route(last_enqueued_at=NOW + timedelta(minutes=1)).route,
            mgw.ROUTE_DEMOTE,
        )

    def test_missing_enqueue_event_demotes(self):
        self.assertEqual(self.route(last_enqueued_at=None).route, mgw.ROUTE_DEMOTE)

    def test_stale_marker_demotes(self):
        self.assertEqual(self.route(now=NOW + timedelta(hours=7)).route, mgw.ROUTE_DEMOTE)

    def test_other_reasons_keep_todays_behaviour(self):
        for reason in ("CI_FAILURE", "MERGE_CONFLICT", "QUEUE_CLEARED", "ROLL_BACK",
                       "BRANCH_PROTECTIONS", "ALREADY_MERGED"):
            self.assertEqual(self.route(reason=reason).route, mgw.ROUTE_DEMOTE, reason)

    def test_watchdogs_own_dequeue_preserves_without_a_second_reenqueue(self):
        result = self.route(reason="MANUAL", now=NOW + timedelta(seconds=30))
        self.assertEqual(result.route, mgw.ROUTE_PRESERVE)
        self.assertFalse(result.reenqueue)

    def test_watchdog_attribution_expires(self):
        self.assertEqual(
            self.route(reason="MANUAL", now=NOW + timedelta(hours=2)).route,
            mgw.ROUTE_DEMOTE,
        )

    def test_a_real_failure_right_after_a_recovery_is_not_attributed_to_us(self):
        for reason in ("MERGE_CONFLICT", "CI_FAILURE"):
            self.assertEqual(
                self.route(reason=reason, now=NOW + timedelta(seconds=30)).route,
                mgw.ROUTE_DEMOTE,
                reason,
            )

    def test_reason_is_normalised(self):
        self.assertEqual(self.route(reason=" ci_timeout ").route, mgw.ROUTE_PRESERVE)
        self.assertEqual(self.route(reason="").route, mgw.ROUTE_DEMOTE)
        self.assertEqual(self.route(reason=None).route, mgw.ROUTE_DEMOTE)


class TestMarkerCodec(unittest.TestCase):
    def test_round_trip(self):
        rendered = mgw.render_marker(
            pr=4534, head=HEAD, base=BASE, ref=REF, suites=0, observed=NOW
        )
        parsed = mgw.parse_marker(f"prose\n\n{rendered}\n")
        self.assertIsNotNone(parsed)
        self.assertEqual(parsed.head, HEAD)
        self.assertEqual(parsed.base, BASE)
        self.assertEqual(parsed.ref, REF)
        self.assertEqual(parsed.suites, 0)
        self.assertEqual(parsed.observed, NOW)
        self.assertEqual(parsed.action, "re-enqueue")

    def test_rejects_malformed(self):
        for body in (
            "",
            None,
            "no marker",
            f"<!-- {mgw.MARKER_KEY} pr=1 head=nothex suites=0 observed=2026-01-01T00:00:00Z -->",
            f"<!-- {mgw.MARKER_KEY} pr=1 head={HEAD} suites=0 -->",
            f"<!-- {mgw.MARKER_KEY} pr=x head={HEAD} suites=0 observed=2026-01-01T00:00:00Z -->",
            f"<!-- other-tool head={HEAD} suites=0 observed=2026-01-01T00:00:00Z -->",
        ):
            self.assertIsNone(mgw.parse_marker(body), body)

    def test_queue_ref_is_base_keyed_and_matches_live_format(self):
        # Verified against live data 2026-07-28: entry pos1 pr=4644 base=c96f5abe…
        # produced ref gh-readonly-queue/main/pr-4644-c96f5abe… with head 1612bd6f…
        self.assertEqual(
            mgw.queue_ref("main", 4644, "c96f5abe3e0b7ba50c1381a1503ba961313b3da7"),
            "gh-readonly-queue/main/pr-4644-c96f5abe3e0b7ba50c1381a1503ba961313b3da7",
        )


# ── 2. behaviour (fake gh) ───────────────────────────────────────────────────────


def _marker_comment(*, head: str = HEAD, observed: datetime | None = None) -> dict:
    stamp = observed or (NOW - timedelta(minutes=10))
    return {
        "body": "text\n\n"
        + mgw.render_marker(pr=4534, head=head, base=BASE, ref=REF, suites=0, observed=stamp)
    }


class ScriptedGh:
    """A gh runner scripted for exactly the argv shapes the watchdog issues."""

    def __init__(self, *, entries, suites_raw, activity, comments, timeline):
        self.entries = entries
        self.suites_raw = suites_raw
        self.activity = activity
        self.comments = comments
        self.timeline = timeline
        self.calls: list[list[str]] = []
        self.mutations: list[list[str]] = []

    def __call__(self, argv: list[str]) -> str:
        self.calls.append(argv)
        joined = " ".join(argv)
        if argv[:2] == ["api", "graphql"]:
            query = next(a for a in argv if a.startswith("query="))
            # Discriminate on the OPERATION keyword, not on a substring of the
            # selection set: both mutations select `mergeQueueEntry`, so a
            # `"mergeQueue" in query` test silently swallowed them and made this fake
            # report zero mutations for a real recovery.
            if query.startswith("query=query"):
                return json.dumps(
                    {"data": {"repository": {"mergeQueue": {"entries": {"nodes": self.entries}}}}}
                )
            assert query.startswith("query=mutation"), query
            self.mutations.append(argv)
            return json.dumps({"data": {}})
        if argv[:2] == ["pr", "comment"]:
            self.mutations.append(argv)
            return ""
        if "/check-suites" in joined:
            return self.suites_raw
        if "/activity" in joined:
            return "\n".join(json.dumps(row) for row in self.activity)
        if "/comments" in joined:
            return "\n".join(json.dumps(row) for row in self.comments)
        if "/timeline" in joined:
            return "\n".join(json.dumps(row) for row in self.timeline)
        raise AssertionError(f"unexpected gh call: {argv}")


class FakeWatchdog:
    def __init__(self, gh: ScriptedGh, rows: list[str], watchdog):
        self.gh = gh
        self.rows = rows
        self.watchdog = watchdog

    @classmethod
    def build(cls, *, suites: int | None = 0, entries=None, comments=None,
              activity=None, now: datetime | None = None, dry_run: bool = False):
        node = {
            "id": "MQE_4534",
            "position": 1,
            "state": "AWAITING_CHECKS",
            "enqueuedAt": "2026-07-27T22:58:53Z",
            "baseCommit": {"oid": BASE},
            "headCommit": {"oid": HEAD},
            "pullRequest": {"number": 4534, "id": "PR_4534"},
        }
        if suites is None:
            suites_raw = "not json"
        else:
            suites_raw = json.dumps(
                {"total_count": suites, "check_suites": [{"id": i} for i in range(suites)]}
            )
        gh = ScriptedGh(
            entries=[node] if entries is None else entries,
            suites_raw=suites_raw,
            activity=[{"activity_type": "branch_creation", "after": HEAD,
                       "timestamp": "2026-07-27T23:04:37Z"}]
            if activity is None
            else activity,
            comments=comments or [],
            timeline=[{"event": "added_to_merge_queue", "created_at": "2026-07-27T22:58:53Z"}],
        )
        rows: list[str] = []
        watchdog = mgw.Watchdog(
            "sparq-org/sparq",
            dry_run=dry_run,
            gh=gh,
            log=rows.append,
            now=lambda: now or NOW,
        )
        return cls(gh, rows, watchdog)

    def run(self) -> int:
        return self.watchdog.sweep()


class TestSweepBehaviour(unittest.TestCase):
    def test_zero_dispatch_entry_is_marked_then_dequeued_then_reenqueued(self):
        harness = FakeWatchdog.build(suites=0)
        self.assertEqual(harness.run(), 0)
        kinds = [m[:2] for m in harness.gh.mutations]
        self.assertEqual(
            kinds,
            [["pr", "comment"], ["api", "graphql"], ["api", "graphql"]],
            harness.gh.mutations,
        )
        # The marker comment is posted BEFORE the dequeue, so the dequeue-triggered
        # feedback workflow can positively identify our own recovery.
        self.assertEqual(harness.gh.mutations[0][:2], ["pr", "comment"])
        body = harness.gh.mutations[0][harness.gh.mutations[0].index("--body") + 1]
        self.assertIn("🤖", body)
        self.assertIsNotNone(mgw.parse_marker(body))
        queries = [
            next(a for a in m if a.startswith("query="))
            for m in harness.gh.mutations[1:]
        ]
        self.assertIn("dequeuePullRequest", queries[0])
        self.assertIn("enqueuePullRequest", queries[1])

    def test_control_group_with_suites_issues_no_mutation_at_all(self):
        harness = FakeWatchdog.build(suites=8)
        self.assertEqual(harness.run(), 0)
        self.assertEqual(harness.gh.mutations, [])
        self.assertTrue(any("decision=HOLD" in row for row in harness.rows), harness.rows)
        self.assertTrue(any("suites=8" in row for row in harness.rows), harness.rows)

    def test_unreadable_suite_count_issues_no_mutation(self):
        harness = FakeWatchdog.build(suites=None)
        harness.run()
        self.assertEqual(harness.gh.mutations, [])
        self.assertTrue(any("decision=REFUSE" in row for row in harness.rows), harness.rows)

    def test_missing_activity_row_issues_no_mutation(self):
        harness = FakeWatchdog.build(suites=0, activity=[])
        harness.run()
        self.assertEqual(harness.gh.mutations, [])
        self.assertTrue(any("decision=REFUSE" in row for row in harness.rows), harness.rows)

    def test_activity_row_for_a_different_head_is_not_our_anchor(self):
        harness = FakeWatchdog.build(
            suites=0,
            activity=[{"activity_type": "branch_creation", "after": "f" * 40,
                       "timestamp": "2026-07-27T23:04:37Z"}],
        )
        harness.run()
        self.assertEqual(harness.gh.mutations, [])

    def test_second_detection_on_the_same_ref_issues_no_mutation(self):
        harness = FakeWatchdog.build(suites=0, comments=[_marker_comment()])
        harness.run()
        self.assertEqual(harness.gh.mutations, [])
        self.assertTrue(any("decision=NOOP" in row for row in harness.rows), harness.rows)

    def test_dry_run_decides_but_never_mutates(self):
        harness = FakeWatchdog.build(suites=0, dry_run=True)
        harness.run()
        self.assertEqual(harness.gh.mutations, [])
        self.assertTrue(any("decision=RECOVER" in row for row in harness.rows), harness.rows)

    def test_per_run_budget_bounds_one_tick(self):
        nodes = []
        for index, pr in enumerate((4534, 4535, 4536), start=1):
            nodes.append(
                {
                    "id": f"MQE_{pr}",
                    "position": index,
                    "state": "AWAITING_CHECKS",
                    "enqueuedAt": "2026-07-27T22:58:53Z",
                    "baseCommit": {"oid": BASE},
                    "headCommit": {"oid": HEAD},
                    "pullRequest": {"number": pr, "id": f"PR_{pr}"},
                }
            )
        harness = FakeWatchdog.build(suites=0, entries=nodes)
        harness.run()
        dequeues = [
            m
            for m in harness.gh.mutations
            if any("dequeuePullRequest" in a for a in m)
        ]
        self.assertEqual(len(dequeues), mgw.DEFAULT_MAX_RECOVERIES_PER_RUN)
        self.assertTrue(any("decision=CAP" in row for row in harness.rows), harness.rows)


class TestEmission(unittest.TestCase):
    """A silent watchdog turns a visible outage into an invisible one."""

    def test_every_entry_emits_a_row_naming_ref_suites_and_decision(self):
        for suites in (0, 8, None):
            harness = FakeWatchdog.build(suites=suites)
            harness.run()
            rows = [r for r in harness.rows if "decision=" in r]
            self.assertEqual(len(rows), 1, (suites, harness.rows))
            row = rows[0]
            self.assertIn(f"ref={REF}", row)
            self.assertIn("suites=", row)
            self.assertIn(f"head={HEAD[:8]}", row)
            self.assertIn("pr=#4534", row)

    def test_holding_states_re_emit_every_tick(self):
        # Three consecutive ticks on an unchanged CAP/NOOP condition must each emit.
        for _ in range(3):
            harness = FakeWatchdog.build(suites=0, comments=[_marker_comment()])
            harness.run()
            self.assertTrue(any("decision=NOOP" in r for r in harness.rows))
            self.assertTrue(any("::warning" in r for r in harness.rows), harness.rows)

    def test_empty_queue_emits_an_explicit_row(self):
        harness = FakeWatchdog.build(entries=[])
        harness.run()
        self.assertTrue(any("EMPTY" in r for r in harness.rows), harness.rows)

    def test_census_row_closes_every_sweep(self):
        harness = FakeWatchdog.build(suites=8)
        harness.run()
        self.assertTrue(any("sweep complete" in r for r in harness.rows), harness.rows)

    def test_refusals_and_caps_are_warnings_not_silence(self):
        harness = FakeWatchdog.build(suites=None)
        harness.run()
        self.assertTrue(any("::warning" in r and "REFUSE" in r for r in harness.rows),
                        harness.rows)


class TestClassifyEndToEnd(unittest.TestCase):
    def test_live_suite_count_is_re_derived_not_read_off_the_marker(self):
        # The marker says suites=0, but the LIVE count now reads 8. The route must
        # follow the live re-derivation, not the recorded value.
        harness = FakeWatchdog.build(suites=8, comments=[_marker_comment()])
        route = harness.watchdog.classify_dequeue(4534, "CI_TIMEOUT")
        self.assertEqual(route.route, mgw.ROUTE_DEMOTE)

    def test_zero_live_count_preserves(self):
        harness = FakeWatchdog.build(suites=0, comments=[_marker_comment()])
        route = harness.watchdog.classify_dequeue(4534, "CI_TIMEOUT")
        self.assertEqual(route.route, mgw.ROUTE_PRESERVE)

    def test_classify_never_mutates(self):
        harness = FakeWatchdog.build(suites=0, comments=[_marker_comment()])
        harness.watchdog.classify_dequeue(4534, "CI_TIMEOUT")
        self.assertEqual(harness.gh.mutations, [])


# ── 3. wiring (the YAML seam) ────────────────────────────────────────────────────


def _yaml(path: Path) -> dict:
    return yaml.safe_load(path.read_text(encoding="utf-8"))


def _on_block(wf: dict) -> dict:
    """Bare `on:` is the YAML boolean True, so PyYAML keys it under True."""
    return wf.get("on", wf.get(True, {})) or {}


def _steps(wf: dict, job: str) -> list[dict]:
    return wf["jobs"][job]["steps"]


def _step(wf: dict, job: str, needle: str) -> dict:
    for step in _steps(wf, job):
        if needle in (step.get("name") or "") or needle == step.get("id"):
            return step
    raise AssertionError(f"no step matching {needle!r} in job {job}")


class TestFeedbackWiring(unittest.TestCase):
    """Mutate the `if:`, the step, or the call site and one of these goes red."""

    def setUp(self):
        self.wf = _yaml(FEEDBACK_YML)
        self.steps = _steps(self.wf, "feedback")
        self.names = [s.get("name") or s.get("uses") for s in self.steps]

    def test_classify_step_exists_with_the_id_the_guards_reference(self):
        ids = [s.get("id") for s in self.steps]
        self.assertIn("classify", ids, self.names)

    def test_classify_call_site_invokes_the_classifier_with_pr_and_reason(self):
        run = _step(self.wf, "feedback", "classify")["run"]
        self.assertIn("scripts/merge-group-watchdog.py", run)
        self.assertIn("classify-dequeue", run)
        self.assertIn("--pr", run)
        self.assertIn("--reason", run)
        self.assertIn("$PR_NUMBER", run)
        self.assertIn("$DEQUEUE_REASON", run)

    def test_the_demote_guard_is_fail_closed(self):
        # `!= 'preserve'` means an EMPTY/MISSING output still demotes. Inverting this
        # to `== 'demote'` would make a classifier failure silently preserve.
        guard = _step(self.wf, "feedback", "Route genuine dequeue")["if"]
        self.assertIn("steps.classify.outputs.route != 'preserve'", guard)
        self.assertNotIn("== 'demote'", guard)
        self.assertIn("github.event.pull_request.merged != true", guard)

    def test_the_preserve_guard_is_fail_open_in_the_safe_direction(self):
        guard = _step(self.wf, "feedback", "Preserve the verdict")["if"]
        self.assertIn("steps.classify.outputs.route == 'preserve'", guard)
        self.assertIn("github.event.pull_request.merged != true", guard)

    def test_the_two_routes_are_mutually_exclusive(self):
        demote = _step(self.wf, "feedback", "Route genuine dequeue")["if"]
        preserve = _step(self.wf, "feedback", "Preserve the verdict")["if"]
        self.assertNotEqual(demote, preserve)
        self.assertIn("route != 'preserve'", demote)
        self.assertIn("route == 'preserve'", preserve)

    def test_preserve_route_never_demotes_the_label(self):
        # Preserving the verdict IS leaving the labels alone: the route must issue no
        # label mutation and must not kill the arm. (`review:changes` appears in this
        # step's explanatory COMMENT prose, so match on the gh flags, not the token.)
        run = _step(self.wf, "feedback", "Preserve the verdict")["run"]
        self.assertNotIn("--add-label", run)
        self.assertNotIn("--remove-label", run)
        self.assertNotIn("--disable-auto", run)
        self.assertNotIn("gh pr edit", run)

    def test_demote_route_still_swaps_the_labels(self):
        # The pre-#4652 behaviour must survive intact on the other arm.
        run = _step(self.wf, "feedback", "Route genuine dequeue")["run"]
        self.assertIn('--add-label "review:changes"', run)
        self.assertIn('--remove-label "review:pass"', run)
        self.assertIn("--disable-auto", run)

    def test_checkout_is_pinned_to_the_default_branch_never_the_pr_head(self):
        step = _step(self.wf, "feedback", "Checkout the dequeue classifier")
        self.assertTrue(str(step["uses"]).startswith("actions/checkout@"))
        self.assertRegex(str(step["uses"]), r"@[0-9a-f]{40}$")
        self.assertEqual(step["with"]["ref"], "${{ github.event.repository.default_branch }}")
        self.assertIs(step["with"]["persist-credentials"], False)
        # The pull_request_target trap: never the PR head.
        rendered = yaml.safe_dump(step)
        for forbidden in ("pull_request.head", "github.head_ref", "merge_commit_sha"):
            self.assertNotIn(forbidden, rendered)

    def test_permissions_add_reads_only(self):
        perms = self.wf["permissions"]
        self.assertEqual(perms.get("checks"), "read")
        self.assertEqual(perms.get("contents"), "read")
        self.assertEqual(perms.get("pull-requests"), "write")

    def test_trigger_is_unchanged(self):
        on = _on_block(self.wf)
        self.assertEqual(list(on), ["pull_request_target"])
        self.assertEqual(on["pull_request_target"]["types"], ["dequeued"])

    def test_step_order_classify_before_both_routes(self):
        order = {name: i for i, name in enumerate(self.names)}
        classify = next(i for n, i in order.items() if n and "Classify the dequeue" in n)
        preserve = next(i for n, i in order.items() if n and "Preserve the verdict" in n)
        demote = next(i for n, i in order.items() if n and "Route genuine dequeue" in n)
        checkout = next(i for n, i in order.items() if n and "Checkout the dequeue" in n)
        self.assertLess(checkout, classify)
        self.assertLess(classify, preserve)
        self.assertLess(classify, demote)


class TestWatchdogWorkflowWiring(unittest.TestCase):
    def setUp(self):
        self.wf = _yaml(WATCHDOG_YML)

    def test_is_schedule_only_and_can_never_gate(self):
        on = _on_block(self.wf)
        self.assertIn("schedule", on)
        self.assertIn("workflow_dispatch", on)
        # A PR-head trigger would make this sweep a required check on every PR.
        for gating in ("pull_request", "pull_request_target", "merge_group", "push"):
            self.assertNotIn(gating, on, gating)

    def test_cron_is_five_minutely(self):
        minutes = self.wf["on" if "on" in self.wf else True]["schedule"][0]["cron"].split()[0]
        values = sorted(int(m) for m in minutes.split(","))
        self.assertEqual(len(values), 12, values)
        gaps = {b - a for a, b in zip(values, values[1:])}
        self.assertEqual(gaps, {5}, values)

    def test_sweep_call_site_invokes_the_sweep_subcommand(self):
        run = _step(self.wf, "watch", "Sweep the merge queue")["run"]
        self.assertIn("scripts/merge-group-watchdog.py", run)
        self.assertIn("sweep", run)
        self.assertIn("--repo", run)
        self.assertIn("--branch", run)

    def test_self_test_runs_before_the_live_sweep(self):
        names = [s.get("name") or s.get("uses") for s in _steps(self.wf, "watch")]
        self.assertLess(
            next(i for i, n in enumerate(names) if n and "Self-test" in n),
            next(i for i, n in enumerate(names) if n and "Sweep the merge queue" in n),
        )
        run = _step(self.wf, "watch", "Self-test policy")["run"]
        self.assertIn("merge-group-watchdog.py --self-test", run)
        self.assertIn("gh_retry.py --self-test", run)

    def test_checkout_is_default_branch_and_sparse_covers_both_scripts(self):
        step = _step(self.wf, "watch", "Checkout watchdog policy")
        self.assertEqual(step["with"]["ref"], "${{ github.event.repository.default_branch }}")
        self.assertIs(step["with"]["persist-credentials"], False)
        sparse = step["with"]["sparse-checkout"]
        self.assertIn("scripts/merge-group-watchdog.py", sparse)
        self.assertIn("scripts/gh_retry.py", sparse)

    def test_permissions_are_least_privilege(self):
        perms = self.wf["permissions"]
        self.assertEqual(perms.get("contents"), "read")
        self.assertEqual(perms.get("checks"), "read")
        self.assertEqual(perms.get("pull-requests"), "write")

    def test_actions_are_sha_pinned(self):
        # PyYAML drops `#` comments, so the 40-hex pin is asserted on the parsed value
        # and the readable `# vX.Y.Z` tag on the raw text.
        raw = WATCHDOG_YML.read_text(encoding="utf-8")
        seen = 0
        for step in _steps(self.wf, "watch"):
            uses = step.get("uses")
            if not uses:
                continue
            seen += 1
            self.assertRegex(uses, r"^[\w.-]+/[\w.-]+@[0-9a-f]{40}$", uses)
            self.assertRegex(raw, re.escape(f"uses: {uses} # v"), uses)
        self.assertGreater(seen, 0)

    def test_serialised_so_two_ticks_cannot_double_recover(self):
        concurrency = self.wf["concurrency"]
        self.assertEqual(concurrency["group"], "merge-group-watchdog")
        self.assertIs(concurrency["cancel-in-progress"], False)

    def test_job_name_is_not_advisory_classified(self):
        # `advisory`/`informational` are reserved tokens for the ci-summary aggregator
        # and require an advisory-registry entry; this job is neither.
        name = self.wf["jobs"]["watch"]["name"]
        self.assertIsNone(re.search(r"\b(advisory|informational)\b", name), name)


class TestSuiteIsWiredIntoCI(unittest.TestCase):
    """A test suite nobody runs is not a test suite (the effect-evidence rule)."""

    def test_docs_quality_runs_this_file_and_the_script_self_test(self):
        text = DOCS_QUALITY_YML.read_text(encoding="utf-8")
        self.assertIn("scripts/tests/test_merge_group_watchdog.py", text)
        self.assertIn("scripts/merge-group-watchdog.py --self-test", text)

    def test_the_running_job_is_not_advisory(self):
        wf = _yaml(DOCS_QUALITY_YML)
        for job_id, job in wf["jobs"].items():
            steps = job.get("steps") or []
            if any(
                "test_merge_group_watchdog.py" in str(step.get("run", ""))
                for step in steps
            ):
                self.assertIsNone(
                    re.search(r"\b(advisory|informational)\b", job.get("name", job_id)),
                    job.get("name", job_id),
                )
                return
        self.fail("no docs-quality job runs the watchdog suite")


if __name__ == "__main__":
    unittest.main(verbosity=2)
