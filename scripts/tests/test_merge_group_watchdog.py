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

    def test_grace_is_bounded_by_the_measured_dispatch_latency(self):
        # MEASURED N=209 (2026-07-25 -> 07-28): create->first-suite latency
        # min 1 s / p50 2 s / p99 4 s / MAX 4 s, with no tail between 4 s and 3600 s.
        # Floor: >=20x the measured maximum, so nobody tunes the grace into the noise
        # band. Ceiling: <= the 300 s cron interval, so the grace never costs an extra
        # tick (a longer grace would silently halve the recovery speed).
        measured_max_seconds = 4
        self.assertGreaterEqual(mgw.DEFAULT_GRACE_SECONDS, 20 * measured_max_seconds)
        self.assertLessEqual(mgw.DEFAULT_GRACE_SECONDS, 300)

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

    def test_the_authoritative_counter_is_total_count_not_the_page_length(self):
        # `check_suites` is capped at per_page; `total_count` is the counter GitHub
        # itself maintains. Reading the array length would silently under-report any
        # group with more than one page of suites.
        watchdog = FakeWatchdog.build()
        watchdog.gh.suites_raw = json.dumps(
            {"total_count": 137, "check_suites": [{"id": i} for i in range(100)]}
        )
        self.assertEqual(watchdog.watchdog.check_suite_count(HEAD), 137)


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
        result = self.route(live_suites=lambda _s: None)
        self.assertEqual(result.route, mgw.ROUTE_DEMOTE)
        # "could not read it" and "it was 8" are the same ROUTE but different
        # operational facts, and the emitted row is the only place an operator sees
        # which one happened. Deleting the `count is None` arm would fall through to
        # `count != 0` and still demote — behaviourally equivalent, diagnostically
        # blind — so the distinction is asserted explicitly rather than left to a
        # mutant that "survives" by being invisible.
        self.assertIn("could not re-derive", result.detail)
        self.assertNotIn("check-suite(s)", result.detail)

    def test_no_marker_demotes(self):
        self.assertEqual(self.route(markers=()).route, mgw.ROUTE_DEMOTE)

    def test_the_newest_marker_wins(self):
        # A PR can accumulate several observations across queue attempts. The route
        # must follow the most recent one; picking the oldest would judge this dequeue
        # against a group head from a previous attempt.
        stale = marker(head="e" * 40, observed=NOW - timedelta(hours=5))
        fresh = marker(head=HEAD, observed=NOW - timedelta(minutes=5))
        seen: list[str] = []

        def spy(sha: str) -> int:
            seen.append(sha)
            return 0

        for order in ((stale, fresh), (fresh, stale)):
            seen.clear()
            result = self.route(markers=order, live_suites=spy)
            self.assertEqual(result.route, mgw.ROUTE_PRESERVE)
            self.assertEqual(seen, [HEAD], order)

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
        # Substrings of the joined argv that should raise GhError instead of replying.
        self.fail_on: tuple[str, ...] = ()

    def __call__(self, argv: list[str]) -> str:
        self.calls.append(argv)
        joined = " ".join(argv)
        for needle in self.fail_on:
            if needle in joined:
                raise mgw.GhError(f"simulated failure for {needle}")
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
            # SHA-KEYED on purpose. A fake that answers every /check-suites URL with
            # the same payload cannot tell the group HEAD from its BASE (or from
            # `main`), and both of those wrong-sha mutants survived until this fake
            # started resolving the sha out of the path.
            path = next(a for a in argv if "/check-suites" in a)
            sha = path.split("/commits/", 1)[1].split("/", 1)[0]
            if sha != HEAD:
                raise mgw.GhError(f"gh api: HTTP 404 — no such commit {sha}")
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
        # The comment goes to THIS entry's PR.
        self.assertEqual(harness.gh.mutations[0][2], "4534")
        queries = [
            next(a for a in m if a.startswith("query="))
            for m in harness.gh.mutations[1:]
        ]
        self.assertIn("dequeuePullRequest", queries[0])
        self.assertIn("enqueuePullRequest", queries[1])
        # BOTH mutations take the PULL REQUEST node id. GitHub names the dequeue input
        # field `id` while documenting it as "the ID of the pull request", so the merge-
        # queue-entry id (MQE_…) is the natural wrong answer and would fail at runtime.
        for mutation in harness.gh.mutations[1:]:
            ident = next(a for a in mutation if a.startswith("id="))
            self.assertEqual(ident, "id=PR_4534", mutation)
            self.assertNotIn("MQE_", ident)

    def test_unreadable_marker_history_refuses_rather_than_assuming_none(self):
        # Treating an unreadable comment history as "no prior recovery" would let the
        # per-PR cap be bypassed on every API blip.
        harness = FakeWatchdog.build(suites=0)
        harness.gh.fail_on = ("/comments",)
        harness.run()
        self.assertEqual(harness.gh.mutations, [])
        self.assertTrue(any("decision=REFUSE" in row for row in harness.rows), harness.rows)

    def test_a_failed_recovery_is_a_red_run_not_a_silent_success(self):
        harness = FakeWatchdog.build(suites=0)
        harness.gh.fail_on = ("dequeuePullRequest",)
        self.assertGreater(harness.run(), 0)
        self.assertTrue(any("RECOVERY FAILED" in row for row in harness.rows), harness.rows)
        self.assertTrue(any("::error" in row for row in harness.rows), harness.rows)

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

    def test_a_non_creation_activity_row_is_not_our_anchor(self):
        # The URL already filters activity_type server-side; this pins the client-side
        # half so a force_push / branch_deletion row carrying the same `after` sha can
        # never be mistaken for the group's build time.
        for activity_type in ("force_push", "push", "branch_deletion", "merge_queue_merge"):
            harness = FakeWatchdog.build(
                suites=0,
                activity=[{"activity_type": activity_type, "after": HEAD,
                           "timestamp": "2026-07-27T23:04:37Z"}],
            )
            harness.run()
            self.assertEqual(harness.gh.mutations, [], activity_type)
            self.assertTrue(
                any("decision=REFUSE" in row for row in harness.rows), activity_type
            )

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

    @staticmethod
    def _three_entry_queue():
        return [
            {
                "id": f"MQE_{pr}",
                "position": index,
                "state": "AWAITING_CHECKS",
                "enqueuedAt": "2026-07-27T22:58:53Z",
                "baseCommit": {"oid": BASE},
                "headCommit": {"oid": HEAD},
                "pullRequest": {"number": pr, "id": f"PR_{pr}"},
            }
            for index, pr in enumerate((4534, 4535, 4536), start=1)
        ]

    def test_per_run_budget_bounds_one_tick(self):
        harness = FakeWatchdog.build(suites=0, entries=self._three_entry_queue())
        harness.run()
        dequeues = [
            m
            for m in harness.gh.mutations
            if any("dequeuePullRequest" in a for a in m)
        ]
        self.assertEqual(len(dequeues), mgw.DEFAULT_MAX_RECOVERIES_PER_RUN)
        self.assertTrue(any("decision=CAP" in row for row in harness.rows), harness.rows)

    def test_the_budget_is_spent_on_the_head_of_the_queue_first(self):
        # Only the head-of-line entry actually blocks merges, so a bounded budget must
        # be spent in queue order. Recovering positions 3 and 2 while position 1 stays
        # broken would burn the budget and leave the outage in place.
        harness = FakeWatchdog.build(suites=0, entries=self._three_entry_queue())
        harness.run()
        recovered = [
            m[2] for m in harness.gh.mutations if m[:2] == ["pr", "comment"]
        ]
        self.assertEqual(recovered, ["4534", "4535"], harness.rows)


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

    def test_rows_record_how_many_groups_are_stacked_above(self):
        # The watchdog's own effectiveness measure. A rebuild discards every group
        # chained on top of the dead one, so `stacked` IS the cost multiplier — and a
        # climbing `stacked` over time is the signal that the grace has drifted too
        # long. Measured 2026-07-28: a 48-minute intervention had stacked=4,
        # stacked_green=2, and all four were discarded.
        nodes = [
            {
                "id": "MQE_4709", "position": 1, "state": "AWAITING_CHECKS",
                "enqueuedAt": "2026-07-27T22:58:53Z",
                "baseCommit": {"oid": BASE}, "headCommit": {"oid": HEAD},
                "pullRequest": {"number": 4709, "id": "PR_4709"},
            },
            {
                "id": "MQE_4714", "position": 2, "state": "MERGEABLE",
                "enqueuedAt": "2026-07-27T22:59:00Z",
                "baseCommit": {"oid": HEAD}, "headCommit": {"oid": "a" * 40},
                "pullRequest": {"number": 4714, "id": "PR_4714"},
            },
            {
                "id": "MQE_4729", "position": 3, "state": "MERGEABLE",
                "enqueuedAt": "2026-07-27T22:59:10Z",
                "baseCommit": {"oid": "a" * 40}, "headCommit": {"oid": "b" * 40},
                "pullRequest": {"number": 4729, "id": "PR_4729"},
            },
            {
                # No group built yet — nothing to discard, so it must NOT be counted.
                "id": "MQE_4731", "position": 4, "state": "QUEUED",
                "enqueuedAt": "2026-07-27T22:59:20Z",
                "baseCommit": None, "headCommit": None,
                "pullRequest": {"number": 4731, "id": "PR_4731"},
            },
        ]
        harness = FakeWatchdog.build(suites=0, entries=nodes)
        harness.run()
        head_row = next(r for r in harness.rows if "pr=#4709" in r)
        self.assertIn("stacked=2", head_row)
        self.assertIn("stacked_green=2", head_row)
        # The last entry with a group has nothing above it.
        tail_row = next(r for r in harness.rows if "pr=#4729" in r)
        self.assertIn("stacked=0", tail_row)
        self.assertIn("stacked_green=0", tail_row)

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


class TestEntrypointFailSafe(unittest.TestCase):
    """The OUTERMOST guard: main() must never let a failure preserve a verdict.

    These drive `main()` end to end because the exception handler and the GITHUB_OUTPUT
    writer are the two pieces the YAML actually consumes, and nothing else exercises
    them — a mutant that flipped the crash route to `preserve`, or that stopped writing
    `route=` at all, survived every other test in this file.
    """

    def _run_main(self, argv, gh_read):
        import os
        import tempfile

        original = mgw.run_gh_read
        out = Path(tempfile.mkdtemp()) / "outputs.txt"
        out.write_text("", encoding="utf-8")
        os.environ["GITHUB_OUTPUT"] = str(out)
        try:
            mgw.run_gh_read = gh_read
            code = mgw.main(argv)
        finally:
            mgw.run_gh_read = original
            os.environ.pop("GITHUB_OUTPUT", None)
        return code, dict(
            line.split("=", 1) for line in out.read_text(encoding="utf-8").splitlines()
            if "=" in line
        )

    def test_classifier_crash_routes_to_demote(self):
        def boom(_argv):
            raise mgw.GhError("HTTP 500 while reading comments")

        code, outputs = self._run_main(
            ["classify-dequeue", "--repo", "sparq-org/sparq", "--pr", "4534",
             "--reason", "CI_TIMEOUT"],
            boom,
        )
        self.assertEqual(code, 0)  # the step must not red the feedback job
        self.assertEqual(outputs["route"], mgw.ROUTE_DEMOTE)
        self.assertEqual(outputs["reenqueue"], "false")

    def test_transient_exhaustion_routes_to_demote(self):
        def exhausted(_argv):
            raise mgw.gh_retry.GhTransientExhausted("HTTP 504 (3 attempts)")

        code, outputs = self._run_main(
            ["classify-dequeue", "--repo", "sparq-org/sparq", "--pr", "4534",
             "--reason", "CI_TIMEOUT"],
            exhausted,
        )
        self.assertEqual(code, 0)
        self.assertEqual(outputs["route"], mgw.ROUTE_DEMOTE)

    def test_outputs_carry_every_field_the_yaml_reads(self):
        rows: list[str] = []
        import os
        import tempfile

        out = Path(tempfile.mkdtemp()) / "outputs.txt"
        out.write_text("", encoding="utf-8")
        os.environ["GITHUB_OUTPUT"] = str(out)
        try:
            mgw.write_outputs(
                mgw.Route(mgw.ROUTE_PRESERVE, True, "zero suites"), log=rows.append
            )
        finally:
            os.environ.pop("GITHUB_OUTPUT", None)
        written = out.read_text(encoding="utf-8")
        # The `if:` expressions read `route`; the preserve step reads `reenqueue` and
        # `detail`. Dropping any of the three silently disables the feature.
        self.assertIn(f"route={mgw.ROUTE_PRESERVE}\n", written)
        self.assertIn("reenqueue=true\n", written)
        self.assertIn("detail=zero suites\n", written)
        self.assertTrue(any("route=preserve" in row for row in rows), rows)

    def test_demote_outputs_are_lowercase_false(self):
        import os
        import tempfile

        out = Path(tempfile.mkdtemp()) / "outputs.txt"
        out.write_text("", encoding="utf-8")
        os.environ["GITHUB_OUTPUT"] = str(out)
        try:
            mgw.write_outputs(mgw.Route(mgw.ROUTE_DEMOTE, False, "no marker"),
                              log=lambda _m: None)
        finally:
            os.environ.pop("GITHUB_OUTPUT", None)
        self.assertIn("reenqueue=false\n", out.read_text(encoding="utf-8"))


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

    # EXACT-MATCH, not substring. A substring assertion cannot see a guard that is
    # still present but CONDITIONALLY INERT: `… route != 'preserve' && false` contains
    # every token a substring test looks for while making the step dead, so the fix
    # lane would silently stop being fed. Both guards are therefore pinned whole; a
    # deliberate change must update this line.
    DEMOTE_GUARD = (
        "github.event.pull_request.merged != true "
        "&& steps.classify.outputs.route != 'preserve'"
    )
    PRESERVE_GUARD = (
        "github.event.pull_request.merged != true "
        "&& steps.classify.outputs.route == 'preserve'"
    )

    def test_the_demote_guard_is_fail_closed(self):
        # `!= 'preserve'` means an EMPTY/MISSING output still demotes. Inverting this
        # to `== 'demote'` would make a classifier failure silently preserve.
        guard = _step(self.wf, "feedback", "Route genuine dequeue")["if"]
        self.assertEqual(guard, self.DEMOTE_GUARD)

    def test_the_preserve_guard_is_fail_open_in_the_safe_direction(self):
        guard = _step(self.wf, "feedback", "Preserve the verdict")["if"]
        self.assertEqual(guard, self.PRESERVE_GUARD)

    def test_no_step_guard_carries_a_constant_disjunct(self):
        # The conditionally-inert class in general: a literal `true`/`false` operand
        # anywhere in a step condition makes that step unconditionally on or off while
        # still reading as a guard.
        for step in self.steps:
            guard = step.get("if")
            if not guard:
                continue
            # `merged != true` is a legitimate COMPARISON against a literal; strip
            # every such comparison first, then any surviving bare literal is a
            # constant operand of && / || — i.e. an always-on or always-off step.
            residue = re.sub(r"[=!]=\s*(?:true|false)\b", "", guard)
            self.assertIsNone(
                re.search(r"\b(?:true|false)\b", residue),
                (step.get("name"), guard),
            )

    def test_the_route_literal_is_the_same_string_on_both_sides(self):
        # CROSS-FILE CONTRACT. The Python constant and the YAML `if:` literal are two
        # halves of one comparison; renaming or case-drifting either side silently
        # makes the preserve route unreachable (and, worse, fails SILENTLY GREEN
        # because the fail-safe `!= 'preserve'` then always demotes).
        raw = FEEDBACK_YML.read_text(encoding="utf-8")
        self.assertIn(f"steps.classify.outputs.route == '{mgw.ROUTE_PRESERVE}'", raw)
        self.assertIn(f"steps.classify.outputs.route != '{mgw.ROUTE_PRESERVE}'", raw)

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

    def test_preserve_route_re_arms_only_when_the_classifier_says_so(self):
        # When the watchdog itself is mid-recovery (reenqueue=false) a second arm here
        # would race its enqueue; when the platform timed the entry out (reenqueue=true)
        # something must put it back. Both directions must be wired.
        run = _step(self.wf, "feedback", "Preserve the verdict")["run"]
        self.assertIn('"$CLASSIFY_REENQUEUE" = "true"', run)
        self.assertIn("gh pr merge", run)
        self.assertIn("--auto", run)
        env = _step(self.wf, "feedback", "Preserve the verdict")["env"]
        self.assertEqual(env["CLASSIFY_REENQUEUE"], "${{ steps.classify.outputs.reenqueue }}")

    def test_preserve_route_checks_the_pr_is_still_open(self):
        run = _step(self.wf, "feedback", "Preserve the verdict")["run"]
        self.assertIn("gh pr view", run)
        self.assertIn("--json state", run)
        self.assertIn('"$STATE" != "OPEN"', run)

    def test_every_post_skip_step_carries_the_merged_guard(self):
        # The first step short-circuits a successful-merge dequeue; every later step
        # must independently re-assert it, because a step `if:` is not inherited.
        for step in self.steps:
            name = step.get("name") or step.get("uses") or ""
            if "Skip successful-merge" in name:
                continue
            self.assertIn(
                "github.event.pull_request.merged != true", step.get("if", ""), name
            )

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
