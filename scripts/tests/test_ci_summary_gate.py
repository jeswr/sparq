#!/usr/bin/env python3
# [FABLE-5] Hermetic unit tests for the ci-summary gate poll loop
# (scripts/ci_summary_gate.py — bead sq-90cv4). Authored by Claude Fable 5.
#
# The bead's mandate: prove the ADAPTIVE SATURATION BUDGET preserves the gate's
# real pass/fail semantics EXACTLY. The four load-bearing cases:
#   (a) all-green                      => SUCCESS   (test_all_green_success)
#   (b) real gating failure           => FAILURE   (test_real_failure_fails)
#   (c) saturation w/ queued siblings => still-settling, NOT a false RED, and the
#       eventual verdict is the REAL one (test_saturation_extension_then_green /
#       test_failure_during_extension_still_fails)
#   (d) genuine hang (no progress, idle queue) => FAILURE
#       (test_genuine_hang_fails), and saturation-forever is BOUNDED: the absolute
#       cap still REDs (test_saturation_forever_bounded_red) — the extension can
#       never convert a real failure into a pass NOR wait forever.
# Plus the pre-existing semantics that must not regress: the sq-ipkku/#997
# terminal-injection settle guard + graceful timeout, the sq-wjth advisory
# word-boundary exclusion, the anchored self-run exclusion, and the empty-set pass.
#
# Fully hermetic: fetchers are injected (no gh, no network, no sleep).
# Run:  python3 scripts/tests/test_ci_summary_gate.py   (stdlib only; no pytest)

from __future__ import annotations

import importlib.util
import io
import sys
import unittest
from contextlib import redirect_stdout
from pathlib import Path

sys.dont_write_bytecode = True  # keep repeated local runs hermetic (no stale .pyc)

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SCRIPT = REPO_ROOT / "scripts" / "ci_summary_gate.py"


def _load_module():
    spec = importlib.util.spec_from_file_location("ci_summary_gate", SCRIPT)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["ci_summary_gate"] = mod
    spec.loader.exec_module(mod)
    return mod


g = _load_module()


def R(name, status="completed", conclusion="success", url="", started="", rid=0,
      external_id=""):
    # started/rid feed the draft-tier superseded-run ordering (started_at, id);
    # pre-draft-tier tests omit them and keep their exact semantics.
    # external_id carries the finding-2 reporter correlation token (the triggering
    # feature-matrix run id); pre-finding-2 tests omit it (empty => no correlation).
    return {"name": name, "status": status, "conclusion": conclusion,
            "details_url": url, "html_url": "", "started_at": started, "id": rid,
            "external_id": external_id}


def W(run_id, workflow_id, *, name="CI", status="completed", conclusion="success",
      created="2026-07-21T14:00:00Z", attempt=1):
    """Actions workflow-run fixture for the #3505 authoritative resolver."""
    return {
        "id": run_id,
        "workflow_id": workflow_id,
        "name": name,
        "path": f".github/workflows/{name.lower().replace(' ', '-')}.yml",
        "head_sha": "deadbeef",
        "status": status,
        "conclusion": conclusion,
        "created_at": created,
        "run_started_at": created,
        "run_attempt": attempt,
        "html_url": f"https://github.test/o/r/actions/runs/{run_id}",
    }


GREEN = R("build + test")
GREEN2 = R("clippy")
PENDING = R("coverage", status="queued", conclusion=None)
IN_PROGRESS = R("coverage", status="in_progress", conclusion=None)
RED = R("clippy", conclusion="failure")


def tiny_cfg(**over):
    """Small budgets so the loop phases are reachable in a handful of polls:
    base budget = 4 polls, absolute cap = 8, settle = 2, floor = 2. The #3758
    unsatisfiable-hold grace is 2 confirming polls (prod: 9)."""
    base = dict(self_run_id="999", interval=0, min_polls=2, settle_polls=2,
                base_polls=4, sat_interval=0, max_total_polls=8,
                sat_queue_min=5, progress_window=2, max_consec_fetch_failures=3,
                summary_path="", unsat_grace_polls=2)
    base.update(over)
    return g.Config(**base)


def scripted(polls, repeat_last=True):
    """fetch_runs() stub: returns polls[i] on call i (an Exception instance is
    raised instead); repeats the final entry once exhausted."""
    state = {"i": 0, "calls": 0}

    def fetch():
        state["calls"] += 1
        i = min(state["i"], len(polls) - 1) if repeat_last else state["i"]
        state["i"] += 1
        entry = polls[i]
        if isinstance(entry, Exception):
            raise entry
        return list(entry)

    fetch.state = state
    return fetch


def run(cfg, polls, depth=0, tier_ctx=None, head_activity=None):
    """Drive run_gate with scripted polls + a constant queue depth (None = the
    depth API is unavailable). Returns (exit_code, captured_output). tier_ctx
    (default None) exercises the draft-tier integrity semantics. head_activity is
    the #3758 per-head unsatisfiability probe: a callable, or omitted (None) for
    "no probe wired" — which must leave every pre-#3758 path byte-identical."""
    out = io.StringIO()
    fetch = scripted(polls)

    def depth_fn():
        return depth

    with redirect_stdout(out):
        code = g.run_gate(cfg, fetch, depth_fn, sleep_fn=lambda s: None,
                          tier_ctx=tier_ctx, fetch_head_activity=head_activity)
    return code, out.getvalue()


class TestVerdictSemantics(unittest.TestCase):
    """The core pass/fail semantics — unchanged by the adaptive budget."""

    def test_all_green_success(self):
        code, out = run(tiny_cfg(), [[GREEN, GREEN2]])
        self.assertEqual(code, 0)
        self.assertIn("PASSED", out)

    def test_real_failure_fails(self):
        code, out = run(tiny_cfg(), [[GREEN, RED]])
        self.assertEqual(code, 1)
        self.assertIn("FAILED", out)
        self.assertIn("clippy", out)

    def test_skipped_and_neutral_pass(self):
        runs = [R("docs", conclusion="skipped"), R("wasm", conclusion="neutral"), GREEN]
        code, _ = run(tiny_cfg(), [runs])
        self.assertEqual(code, 0)

    def test_empty_set_passes(self):
        code, out = run(tiny_cfg(), [[]])
        self.assertEqual(code, 0)
        self.assertIn("stable empty set", out)

    def test_advisory_failure_excluded(self):
        runs = [R("vale (prose, advisory)", conclusion="failure"),
                R("unsafe report (cargo-geiger, informational)", conclusion="failure"),
                GREEN]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 0)
        self.assertIn("2 advisory check(s) excluded", out)

    def test_advisories_plural_still_gates(self):
        # sq-wjth word boundary: "advisories" must NOT match the advisory rule.
        runs = [R("cargo-deny (advisories, bans, licenses, sources)", conclusion="failure")]
        code, _ = run(tiny_cfg(), [runs])
        self.assertEqual(code, 1)

    def test_incomplete_conclusion_fails(self):
        # A terminal-status run with a null conclusion renders as "incomplete" and gates.
        runs = [R("weird", conclusion=None)]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 1)
        self.assertIn("incomplete", out)


class TestSettleAndGracefulTimeout(unittest.TestCase):
    """The sq-ipkku settle guard + the #997 graceful timeout must not regress."""

    def test_terminal_injection_does_not_starve_settle(self):
        polls = [
            [GREEN, PENDING],                    # pending holds the gate
            [GREEN, R("coverage")],              # all terminal, stable=1
            [GREEN, R("coverage"), R("late")],   # terminal INJECTION: stable=2 (no reset)
        ]
        code, out = run(tiny_cfg(), polls)
        self.assertEqual(code, 0)
        self.assertIn("PASSED", out)
        self.assertNotIn("::error::", out)

    def test_graceful_timeout_all_green_passes(self):
        # pending flip-flops so a full quiet settle never happens; depth is HIGH so
        # the pending polls past the base budget extend instead of hanging-RED; the
        # ABSOLUTE budget then renders the real (green) verdict — the #997 fix.
        flip = [[GREEN, PENDING], [GREEN, R("coverage")]] * 4
        code, out = run(tiny_cfg(), flip, depth=20)
        self.assertEqual(code, 0)
        self.assertIn("rendering the verdict on the final all-terminal set", out)

    def test_graceful_timeout_real_failure_still_fails(self):
        flip = [[GREEN, PENDING], [GREEN, R("coverage")]] * 3 \
            + [[GREEN, PENDING], [GREEN, R("coverage", conclusion="failure")]]
        code, out = run(tiny_cfg(), flip, depth=20)
        self.assertEqual(code, 1)
        self.assertIn("FAILED", out)


class TestFailFast(unittest.TestCase):
    """[FABLE-5] Fail-fast on a concluded gating failure (2026-07-17 maintainer
    directive): the gate REDs the moment a gating leg's failure is confirmed by
    the grace re-poll, instead of waiting out every other sibling — WITHOUT ever
    firing on a superseded/forgiven run, an in-flight rerun's predecessor, or an
    advisory leg. Each guard's mutation is caught: dropping fail-fast breaks the
    immediacy assertions; dropping a guard turns a must-pass scenario red."""

    def test_gating_failure_with_others_pending_reds_immediately(self):
        # (i) One concluded gating failure + siblings still pending => FAILURE at
        # the grace-confirm poll (attempt 2), not after the pending legs settle.
        # Mutation coverage: without fail-fast these polls run to the base budget
        # and red as a "genuine hang" — the fail-fast marker + the poll-count
        # assertions below then fail.
        code, out = run(tiny_cfg(), [[RED, PENDING]])
        self.assertEqual(code, 1)
        self.assertIn("FAILED (fail-fast)", out)
        self.assertIn("clippy", out)
        self.assertIn("attempt 2", out)
        self.assertNotIn("attempt 3", out, "the red must land at the grace-confirm poll")
        self.assertNotIn("genuine hang", out)

    def test_failed_leg_with_inflight_same_name_rerun_does_not_fail_fast(self):
        # (ii) A red leg whose same-name rerun is already IN PROGRESS must NOT
        # fail-fast — the gate waits on the rerun (which wins: the check-runs
        # listing returns the latest attempt, so the old failure drops out).
        # Mutation coverage: without the in-flight guard the identical failure
        # is observed on polls 1+2 and the gate REDs before the rerun lands.
        failed = R("clippy", conclusion="failure", started="2026-07-17T10:00:00Z", rid=1)
        rerun = R("clippy", status="in_progress", conclusion=None,
                  started="2026-07-17T10:05:00Z", rid=2)
        rerun_green = R("clippy", started="2026-07-17T10:05:00Z", rid=2)
        polls = [
            [failed, rerun, PENDING],
            [failed, rerun, PENDING],
            [rerun_green, R("coverage")],
        ]
        code, out = run(tiny_cfg(), polls)
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)
        self.assertNotIn("fail-fast", out)

    def test_cancelled_race_loser_select_mid_poll_does_not_fail_fast(self):
        # (iii) A concurrency-cancel race-loser select observed MID-POLL (siblings
        # still pending) must not fail-fast the gate: forgive_superseded drops it
        # upstream of the fail-fast scan (same-tier same-name success, the
        # sq-fmx4u.3 pure-select rule), and a cancelled conclusion is not
        # `failure` in any case. The eventual verdict is the real (green) one.
        sel_win = R(SELECT_NAME, conclusion="success",
                    started="2026-07-17T10:00:10Z", rid=2)
        sel_lost = R(SELECT_NAME, conclusion="cancelled",
                     started="2026-07-17T10:00:20Z", rid=3)
        polls = [
            [sel_win, sel_lost, PENDING],
            [sel_win, sel_lost, R("coverage")],
        ]
        code, out = run(tiny_cfg(), polls)
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)
        self.assertNotIn("fail-fast", out)

    def test_cancelled_leg_awaiting_rerun_does_not_fail_fast(self):
        # (iii-b) spec guard (a): a cancelled-then-rerun leg mid-poll — cancelled
        # is never a fail-fast trigger, the successor supersedes it, gate greens.
        old = R("build + test", conclusion="cancelled",
                started="2026-07-17T10:00:00Z", rid=1)
        new_running = R("build + test", status="in_progress", conclusion=None,
                        started="2026-07-17T10:05:00Z", rid=2)
        new_green = R("build + test", started="2026-07-17T10:05:00Z", rid=2)
        polls = [[old, PENDING], [old, new_running, PENDING], [old, new_green, R("coverage")]]
        code, out = run(tiny_cfg(), polls)
        self.assertEqual(code, 0, out)
        self.assertNotIn("fail-fast", out)

    def test_advisory_failure_mid_poll_does_not_fail_fast(self):
        # (iv) An advisory leg's failure while siblings are pending must neither
        # fail-fast nor gate at all (sq-wjth) — the verdict stays green.
        adv = R("vale (prose, advisory)", conclusion="failure")
        polls = [[adv, PENDING], [adv, R("coverage")]]
        code, out = run(tiny_cfg(), polls)
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)
        self.assertNotIn("fail-fast", out)

    def test_grace_repoll_dodges_a_transient_failure_read(self):
        # The grace re-poll: a failure observed ONCE that a fresh fetch does not
        # re-observe (an API read race) must not red the gate.
        polls = [
            [RED, PENDING],                 # racy read: clippy "failure"
            [GREEN2, PENDING],              # fresh fetch: clippy is green
            [GREEN2, R("coverage")],
        ]
        code, out = run(tiny_cfg(), polls)
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)
        self.assertIn("grace re-poll", out)
        self.assertNotIn("FAILED (fail-fast)", out)

    def test_all_terminal_failure_keeps_the_normal_render_path(self):
        # pending == 0 => fail-fast stands down and the classic settle + full
        # render_verdict path (with its richer message) is byte-identical.
        code, out = run(tiny_cfg(), [[GREEN, RED]])
        self.assertEqual(code, 1)
        self.assertIn("non-passing gating check(s)", out)
        self.assertNotIn("fail-fast", out)

    def test_fail_fast_never_turns_a_verdict_green(self):
        # Exit-0 invariant: fail-fast is an exit-1-only path — a green set with
        # pending work must still settle normally and pass.
        polls = [[GREEN, PENDING], [GREEN, R("coverage")]]
        code, out = run(tiny_cfg(), polls)
        self.assertEqual(code, 0)
        self.assertNotIn("fail-fast", out)


class TestAdaptiveSaturationBudget(unittest.TestCase):
    """sq-90cv4: saturation extends, hangs RED, real verdicts are untouched."""

    def test_saturation_extension_then_green(self):
        # Siblings stay QUEUED through the base budget while the repo queue is deep
        # (the congestion-collapse shape) — the gate must extend, then pass for real.
        polls = [[GREEN, GREEN2, PENDING]] * 5 + [[GREEN, GREEN2, R("coverage")]]
        code, out = run(tiny_cfg(), polls, depth=20)
        self.assertEqual(code, 0)
        self.assertIn("Extending the wait (adaptive budget", out)
        self.assertIn("PASSED", out)
        self.assertNotIn("::error::", out)

    def test_genuine_hang_fails(self):
        # Pending forever, idle queue, zero progress => RED at the base budget.
        polls = [[GREEN, IN_PROGRESS]]
        code, out = run(tiny_cfg(), polls, depth=0)
        self.assertEqual(code, 1)
        self.assertIn("genuine hang", out)

    def test_saturation_forever_bounded_red(self):
        # The extension is BOUNDED: perpetual saturation still REDs at the absolute
        # cap — never an infinite wait, never a synthesized pass.
        polls = [[GREEN, PENDING]]
        code, out = run(tiny_cfg(), polls, depth=20)
        self.assertEqual(code, 1)
        self.assertIn("ABSOLUTE budget", out)

    def test_failure_during_extension_still_fails(self):
        # The adaptive budget must NEVER convert a real failure into a pass.
        polls = [[GREEN, PENDING]] * 5 + [[GREEN, R("coverage", conclusion="failure")]]
        code, out = run(tiny_cfg(), polls, depth=20)
        self.assertEqual(code, 1)
        self.assertIn("FAILED", out)

    def test_progress_signal_extends_when_depth_unknown(self):
        # Queue-depth API unavailable (None) but completions keep landing => the
        # progress signal alone extends; the eventual verdict is real.
        polls = [
            [R("a"), R("b"), PENDING, R("d", status="queued", conclusion=None),
             R("e", status="queued", conclusion=None), R("f", status="queued", conclusion=None)],
            [R("a"), R("b"), R("coverage"), R("d", status="queued", conclusion=None),
             R("e", status="queued", conclusion=None), R("f", status="queued", conclusion=None)],
            [R("a"), R("b"), R("coverage"), R("d"),
             R("e", status="queued", conclusion=None), R("f", status="queued", conclusion=None)],
            [R("a"), R("b"), R("coverage"), R("d"), R("e"),
             R("f", status="queued", conclusion=None)],
            [R("a"), R("b"), R("coverage"), R("d"), R("e"), R("f")],
        ]
        code, out = run(tiny_cfg(), polls, depth=None)
        self.assertEqual(code, 0)
        self.assertIn("PASSED", out)

    def test_no_progress_and_depth_unknown_is_a_hang(self):
        # Unknown depth + flat completions must NOT extend forever: it REDs at the
        # base budget exactly like the old behaviour (fail-closed on no evidence).
        polls = [[GREEN, IN_PROGRESS]]
        code, out = run(tiny_cfg(), polls, depth=None)
        self.assertEqual(code, 1)
        self.assertIn("genuine hang", out)


class TestSelfExclusionAndFetch(unittest.TestCase):
    def test_self_run_excluded_by_anchored_id(self):
        me = R("gate", status="in_progress", conclusion=None,
               url="https://github.com/o/r/actions/runs/999/job/1")
        sibling = R("build + test", url="https://github.com/o/r/actions/runs/9991/job/2")
        # Without the exclusion the pending self would deadlock the gate; with it,
        # only the green sibling counts.
        code, _ = run(tiny_cfg(), [[me, sibling]])
        self.assertEqual(code, 0)

    def test_anchoring_does_not_match_prefix_ids(self):
        self.assertTrue(g.is_self({"details_url": "https://x/actions/runs/999/job/4"}, "999"))
        self.assertTrue(g.is_self({"details_url": "https://x/actions/runs/999"}, "999"))
        self.assertFalse(g.is_self({"details_url": "https://x/actions/runs/9991/job/4"}, "999"))
        self.assertFalse(g.is_self({"html_url": "https://x/actions/runs/1999/job/4"}, "999"))
        # html_url fallback when details_url is absent.
        self.assertTrue(g.is_self({"html_url": "https://x/actions/runs/999/job/4"}, "999"))

    def test_transient_fetch_failures_tolerated(self):
        polls = [g.FetchError("blip"), g.FetchError("blip"), [GREEN], [GREEN]]
        code, out = run(tiny_cfg(), polls)
        self.assertEqual(code, 0)
        self.assertIn("skipping this poll", out)

    def test_persistent_fetch_failures_fail_closed(self):
        polls = [g.FetchError("down")]
        code, out = run(tiny_cfg(), polls)
        self.assertEqual(code, 1)
        self.assertIn("consecutive check-run fetch failures", out)


class TestNewestWorkflowRunResolution(unittest.TestCase):
    """[GPT-5.6] #3505: workflow identity/attempt, not stale check presence, wins."""

    def test_newest_order_prefers_new_run_id_then_same_run_attempt(self):
        same_time = "2026-07-21T14:00:00Z"
        old_rerun = W(101, 7, created=same_time, attempt=2)
        new_run = W(102, 7, created=same_time, attempt=1)
        self.assertEqual(g.newest_workflow_runs([old_rerun, new_run])["id:7"]["id"], 102)
        attempt_one = W(102, 7, created=same_time, attempt=1)
        attempt_two = W(102, 7, created=same_time, attempt=2)
        self.assertEqual(
            g.newest_workflow_runs([attempt_one, attempt_two])["id:7"]["run_attempt"], 2
        )

    def test_superseded_cancelled_and_failure_are_non_events(self):
        old_cancel = W(101, 7, conclusion="cancelled", created="2026-07-21T13:00:00Z")
        old_failure = W(102, 7, conclusion="failure", created="2026-07-21T13:30:00Z")
        newest = W(103, 7, created="2026-07-21T14:00:00Z")
        checks = [
            R("test shard", conclusion="cancelled", rid=1001,
              url="https://github.test/o/r/actions/runs/101/job/1"),
            R("test shard", conclusion="failure", rid=1002,
              url="https://github.test/o/r/actions/runs/102/job/2"),
        ]
        resolved, dropped = g.resolve_newest_workflow_runs(
            checks, [old_cancel, old_failure, newest], "999"
        )
        self.assertEqual(dropped, 2)
        code, out = run(tiny_cfg(), [resolved])
        self.assertEqual(code, 0, out)
        self.assertNotIn("cancelled", [r.get("conclusion") for r in resolved])
        self.assertNotIn("failure", [r.get("conclusion") for r in resolved])

    def test_newest_failure_still_fails_when_job_check_evaporated(self):
        newest = W(103, 7, conclusion="failure")
        resolved, _ = g.resolve_newest_workflow_runs([], [newest], "999")
        self.assertEqual(len(resolved), 1, "run-level failure evidence must be synthesized")
        code, out = run(tiny_cfg(), [resolved])
        self.assertEqual(code, 1)
        self.assertIn("workflow-run verdict (id:7)", out)

    def test_newest_failure_cannot_be_advisory_excluded_by_workflow_name(self):
        newest = W(103, 7, name="all advisory", conclusion="failure")
        resolved, _ = g.resolve_newest_workflow_runs([], [newest], "999")
        code, out = run(tiny_cfg(), [resolved])
        self.assertEqual(code, 1, out)

    def test_authoritative_advisory_job_failure_remains_non_gating(self):
        newest = W(103, 7, conclusion="failure")
        advisory_job = {
            "id": 2001,
            "name": "vale (prose, advisory)",
            "status": "completed",
            "conclusion": "failure",
            "started_at": "2026-07-21T14:05:00Z",
            "html_url": "https://github.test/o/r/actions/runs/103/job/2",
        }
        resolved, _ = g.resolve_newest_workflow_runs(
            [], [newest], "999", attempt_jobs={103: [advisory_job]}
        )
        code, out = run(tiny_cfg(), [resolved])
        self.assertEqual(code, 0, out)

    def test_green_run_with_evaporated_check_resolves_without_hang(self):
        newest = W(103, 7)
        resolved, _ = g.resolve_newest_workflow_runs([], [newest], "999")
        code, out = run(tiny_cfg(), [resolved])
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)

    def test_completed_jobs_listing_recovers_evaporated_required_leg(self):
        newest = W(103, 7, conclusion="failure")
        failed_job = {
            "id": 2001,
            "name": "required test shard",
            "status": "completed",
            "conclusion": "failure",
            "started_at": "2026-07-21T14:05:00Z",
            "html_url": "https://github.test/o/r/actions/runs/103/job/2",
        }
        resolved, _ = g.resolve_newest_workflow_runs(
            [], [newest], "999", attempt_jobs={103: [failed_job]}
        )
        self.assertEqual([r["name"] for r in resolved], ["required test shard"])
        code, out = run(tiny_cfg(), [resolved])
        self.assertEqual(code, 1, out)

    def test_evaporated_feature_group_still_requires_reporter(self):
        newest = W(103, 7)
        group_job = {
            "id": 2001,
            "name": "opt-in group (0)",
            "status": "completed",
            "conclusion": "success",
            "started_at": "2026-07-21T14:05:00Z",
            "html_url": "https://github.test/o/r/actions/runs/103/job/2",
        }
        resolved, _ = g.resolve_newest_workflow_runs(
            [], [newest], "999", attempt_jobs={103: [group_job]}
        )
        code, out = run(tiny_cfg(), [resolved])
        self.assertEqual(code, 1)
        self.assertIn("reporter verdict never landed", out)

    def test_rerun_attempt_uses_attempt_jobs_not_old_same_run_id_checks(self):
        rerun = W(103, 7, attempt=2)
        old_failure = R(
            "test shard", conclusion="failure", rid=1001,
            url="https://github.test/o/r/actions/runs/103/job/1",
        )
        latest_job = {
            "id": 2001,
            "name": "test shard",
            "status": "completed",
            "conclusion": "success",
            "started_at": "2026-07-21T14:05:00Z",
            "html_url": "https://github.test/o/r/actions/runs/103/job/2",
        }
        resolved, dropped = g.resolve_newest_workflow_runs(
            [old_failure], [rerun], "999", attempt_jobs={103: [latest_job]}
        )
        self.assertEqual(dropped, 1)
        code, out = run(tiny_cfg(), [resolved])
        self.assertEqual(code, 0, out)

    def test_rerun_attempt_keeps_only_current_manually_posted_run_checks(self):
        rerun = W(103, 7, attempt=2)
        rerun["run_started_at"] = "2026-07-21T14:05:00Z"
        old_report = R(
            g.FM_REPORT_NAME, started="2026-07-21T14:00:00Z", rid=1001,
            url="https://github.test/o/r/actions/runs/103", external_id="103",
        )
        current_report = R(
            g.FM_REPORT_NAME, started="2026-07-21T14:06:00Z", rid=1002,
            url="https://github.test/o/r/actions/runs/103", external_id="103",
        )
        resolved, dropped = g.resolve_newest_workflow_runs(
            [old_report, current_report], [rerun], "999", attempt_jobs={103: []}
        )
        reports = [r for r in resolved if r.get("name") == g.FM_REPORT_NAME]
        self.assertEqual([r["id"] for r in reports], [1002])
        self.assertEqual(dropped, 1)

    def test_duplicate_job_names_do_not_cross_workflow_identity(self):
        a = W(101, 7, name="CI A")
        b = W(102, 8, name="CI B")
        a_failure = R(
            "shared job", conclusion="failure", rid=1001,
            url="https://github.test/o/r/actions/runs/101/job/1",
        )
        b_running = R(
            "shared job", status="in_progress", conclusion=None, rid=1002,
            url="https://github.test/o/r/actions/runs/102/job/2",
        )
        resolved, _ = g.resolve_newest_workflow_runs(
            [a_failure, b_running], [a, b], "999"
        )
        self.assertEqual(len(g.failfast_failures(resolved)), 1,
                         "another workflow's same-name in-flight job must not mask failure")

    def test_cancelled_auto_redispatch_is_bounded_once(self):
        cancelled = W(104, 7, conclusion="cancelled")
        posts = []
        resolver = g.WorkflowRunResolver(
            self_run_id="999",
            fetch_checks=lambda: [],
            fetch_workflows=lambda: [cancelled],
            fetch_attempt_jobs=lambda run_id, attempt: [],
            redispatch=lambda run_id: posts.append(run_id),
            redispatch_settle_polls=2,
        )
        first = resolver()
        self.assertTrue(any(r.get("status") != "completed" for r in first),
                        "a dispatched cancellation must become pending, not failure")
        resolver()
        with self.assertRaisesRegex(g.SupersededLegsError, "superseded-legs"):
            resolver()
        self.assertEqual(posts, [104], "API lag must never cause a second POST")

    def test_completed_run_jobs_are_authoritative_and_cached(self):
        complete = W(104, 7)
        calls = []
        job = {
            "id": 2001,
            "name": "required test shard",
            "status": "completed",
            "conclusion": "success",
            "started_at": "2026-07-21T14:05:00Z",
            "html_url": "https://github.test/o/r/actions/runs/104/job/2",
        }

        def fetch_jobs(run_id, attempt):
            calls.append((run_id, attempt))
            return [job]

        resolver = g.WorkflowRunResolver(
            self_run_id="999",
            fetch_checks=lambda: [],
            fetch_workflows=lambda: [complete],
            fetch_attempt_jobs=fetch_jobs,
            redispatch=lambda run_id: self.fail("green run must not redispatch"),
        )
        first = resolver()
        second = resolver()
        self.assertEqual([r["name"] for r in first], ["required test shard"])
        self.assertEqual([r["name"] for r in second], ["required test shard"])
        self.assertEqual(calls, [(104, 1)], "terminal job inventory should be cached")

    def test_cancelled_retry_attempt_fails_loud_without_third_attempt(self):
        cancelled_retry = W(104, 7, conclusion="cancelled", attempt=2)
        posts = []
        resolver = g.WorkflowRunResolver(
            self_run_id="999",
            fetch_checks=lambda: [],
            fetch_workflows=lambda: [cancelled_retry],
            fetch_attempt_jobs=lambda run_id, attempt: [],
            redispatch=lambda run_id: posts.append(run_id),
        )
        with self.assertRaisesRegex(g.SupersededLegsError, "attempt 2"):
            resolver()
        self.assertEqual(posts, [])

    def test_unrecoverable_cancellation_uses_distinct_loud_gate_message(self):
        code, out = run(
            tiny_cfg(),
            [g.SupersededLegsError("superseded-legs, re-run required (#3505): fixture")],
        )
        self.assertEqual(code, 1)
        self.assertIn("superseded-legs, re-run required", out)
        self.assertIn("did not dispatch it more than once", out)


class TestAdvisoryRule(unittest.TestCase):
    def test_word_boundary_rule(self):
        self.assertTrue(g.is_advisory("markdownlint-advisory (whole repo)"))
        self.assertTrue(g.is_advisory("external-links (lychee online, advisory)"))
        self.assertTrue(g.is_advisory("unsafe report (cargo-geiger, informational)"))
        self.assertFalse(g.is_advisory("cargo-deny (advisories, bans, licenses, sources)"))
        self.assertFalse(g.is_advisory("build + test"))
        self.assertFalse(g.is_advisory("gate"))


# [OPUS-5] PLATFORM-MANAGED advisory exclusion (the exact fail-closed allow-list).
# LIVE DEFECT: main gate run 30136978362 (2026-07-25T00:46Z) failed fast on
# "✗ Dependabot: failure" — the GitHub-managed Dependabot Updates job (run
# 30136987253) concluded `security_update_not_possible` for npm `brace-expansion`,
# an UPSTREAM condition with no in-repo remedy. Because the name is chosen by
# GitHub it cannot carry the "(advisory)" token, so the name-token rule could not
# reach it and it GATED. These tests pin all four halves of the fix:
#   (1) exactly "Dependabot" failing does NOT red the verdict;
#   (2) an unknown/new platform-ish name still REDs (fail-closed — no wildcards);
#   (3) the pre-existing advisory-name rule is untouched; and
#   (4) a REAL gating failure alongside a Dependabot failure still REDs.
class TestPlatformManagedAdvisoryRule(unittest.TestCase):
    DEPENDABOT = "Dependabot"

    def test_predicate_matches_only_the_exact_allow_listed_name(self):
        self.assertTrue(g.is_platform_managed_advisory(self.DEPENDABOT))
        # Case-insensitive + surrounding-whitespace tolerant whole-name match.
        self.assertTrue(g.is_platform_managed_advisory("  dependabot "))
        self.assertTrue(g.is_platform_managed_advisory("DEPENDABOT"))
        # (2) FAIL-CLOSED: no substring/prefix/suffix/wildcard reach. Every one of
        # these is an unknown name and must keep gating.
        for unknown in (
            "Dependabot Updates",
            "Dependabot alerts",
            "dependabot-security-check",
            "verify Dependabot lockfile",
            "Dependabot / npm_and_yarn",
            "supply-chain gates (deny + vet + SBOM + VEX + OpenSSF + js-sbom)",
            "build + test",
            "gate",
        ):
            self.assertFalse(g.is_platform_managed_advisory(unknown), unknown)
            self.assertFalse(g.is_advisory(unknown), unknown)
        # (3) The name-token rule is a SEPARATE concern and is not absorbed by the
        # allow-list — an advisory-named leg is not "platform managed".
        self.assertFalse(g.is_platform_managed_advisory("vale (prose, advisory)"))
        self.assertTrue(g.is_advisory("vale (prose, advisory)"))
        # ...and the union predicate every consumer reads sees both rules.
        self.assertTrue(g.is_advisory(self.DEPENDABOT))

    def test_dependabot_failure_does_not_red_the_verdict(self):
        # (1) The live defect, end to end through the real verdict path.
        runs = [R(self.DEPENDABOT, conclusion="failure"), GREEN, GREEN2]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)
        self.assertIn("1 advisory check(s) excluded", out)

    def test_unknown_platform_managed_name_still_reds(self):
        # (2) Fail-closed at the VERDICT level, not just the predicate: a renamed or
        # newly-added platform job is a gating leg until this allow-list is edited.
        for unknown in ("Dependabot Updates", "Dependabot alerts", "dependabot-security"):
            with self.subTest(name=unknown):
                code, out = run(tiny_cfg(), [[R(unknown, conclusion="failure"), GREEN]])
                self.assertEqual(code, 1, out)
                self.assertIn(unknown, out)

    def test_existing_advisory_name_rule_still_excludes(self):
        # (3) Regression guard for sq-wjth alongside the new rule, including the
        # plural "advisories" that must keep GATING.
        runs = [R("vale (prose, advisory)", conclusion="failure"),
                R("unsafe report (cargo-geiger, informational)", conclusion="failure"),
                R(self.DEPENDABOT, conclusion="failure"),
                GREEN]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 0, out)
        self.assertIn("3 advisory check(s) excluded", out)
        code, _ = run(tiny_cfg(), [[
            R("cargo-deny (advisories, bans, licenses, sources)", conclusion="failure"),
            R(self.DEPENDABOT, conclusion="failure"),
        ]])
        self.assertEqual(code, 1)

    def test_real_failure_alongside_dependabot_still_reds(self):
        # (4) The exclusion must not become a blanket amnesty: a genuine gating
        # failure sharing the sibling set still REDs, and the Dependabot leg is not
        # what the verdict blames.
        runs = [R(self.DEPENDABOT, conclusion="failure"), RED, GREEN]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 1, out)
        self.assertIn("clippy", out)
        failing_lines = [ln for ln in out.splitlines() if ln.startswith("- ✗ ")]
        self.assertTrue(failing_lines)
        self.assertNotIn(f"- ✗ {self.DEPENDABOT}: failure", failing_lines)

    def test_dependabot_failure_does_not_fail_fast(self):
        # Fail-fast reuses is_advisory, so the exclusion must hold there too: a
        # Dependabot red while siblings are still running must not short-circuit.
        dependabot = R(self.DEPENDABOT, conclusion="failure")
        self.assertEqual(g.failfast_failures([dependabot, PENDING]), [])
        code, out = run(tiny_cfg(), [[dependabot, PENDING], [dependabot, GREEN]])
        self.assertEqual(code, 0, out)
        self.assertNotIn("fail-fast", out)

    def test_dependabot_only_workflow_failure_gets_no_synthetic_gating_verdict(self):
        # The resolver's run-level evidence path also reads is_advisory: GitHub's
        # Dependabot workflow RUN concludes failure, so without the exclusion a
        # synthetic "workflow-run verdict (…)" would red the gate even though the
        # only failing job is the excluded one.
        newest = W(103, 7, name="Dependabot Updates", conclusion="failure")
        dependabot_job = {
            "id": 2001,
            "name": self.DEPENDABOT,
            "status": "completed",
            "conclusion": "failure",
            "started_at": "2026-07-21T14:05:00Z",
            "html_url": "https://github.test/o/r/actions/runs/103/job/2",
        }
        resolved, _ = g.resolve_newest_workflow_runs(
            [], [newest], "999", attempt_jobs={103: [dependabot_job]}
        )
        self.assertNotIn(
            "workflow-run verdict (id:7)", [r.get("name") for r in resolved]
        )
        code, out = run(tiny_cfg(), [resolved])
        self.assertEqual(code, 0, out)


# [FABLE-5] sq-fmx4u.3: change-based test-selection semantics (design §5.3).
# The three load-bearing safety invariants, exactly as the bead states them:
#   (1) skipped-not-affected + select SUCCESS      => gate SUCCESS (no hang, no RED)
#   (2) a job that FAILED                          => gate FAILURE (selection never
#       masks a real failure, with or without skips present)
#   (3) select present-but-not-success             => gate FAILURE even if every
#       other sibling is green/skipped (a skip is only trustworthy under a
#       successful selection — fail-closed, design §4.3)
# plus the transition guarantee: select ABSENT (a pre-selection sibling set)
# preserves the previous semantics byte-for-byte (skipped is non-failing).
SELECT_NAME = "select / select (change-based test selection)"
SEL_OK = R(SELECT_NAME)


class TestSelectionSemantics(unittest.TestCase):
    def test_skipped_not_affected_with_select_success_passes(self):
        # Invariant 1: an enforced-selection run — wide lanes skipped, select green.
        runs = [SEL_OK,
                R("W3C/OGC GeoSPARQL conformance (ratchet)", conclusion="skipped"),
                R("opt-in sparq-rsp (rsp)", conclusion="skipped"),
                GREEN]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 0)
        self.assertIn("PASSED", out)
        # The bead's transparency line: N of M ran, K skipped, select healthy.
        self.assertIn("2 skipped", out)
        self.assertIn("selection pre-job succeeded", out)

    def test_orchestration_only_pr_gate_passes_with_codeql_and_engine_skipped(self):
        # [OPUS-4.8] path-aware CI: the concrete orchestration-only PR (#3416 class —
        # routing.toml + triage.py) sibling set. CodeQL is `skipped` (its new
        # rust_changed guard), every engine lane + opt-in leg is `skipped` (empty
        # affected closure), the cheap gates run green, and `select` concludes
        # SUCCESS (mode=selected attributes the skips). The gate must render PASS —
        # a fast green for an orchestration PR with the whole Rust matrix skipped.
        runs = [
            SEL_OK,
            R("CodeQL analysis (rust)", conclusion="skipped"),
            R("test (load-aware shard bulk 1/3)", conclusion="skipped"),
            R("opt-in sparq-engine (paths)", conclusion="skipped"),
            R("bench (deterministic ratchet)", conclusion="skipped"),
            R("differential-smoke", conclusion="skipped"),
            R("docs-quality quick-gates", conclusion="success"),
        ]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 0)
        self.assertIn("PASSED", out)
        self.assertIn("selection pre-job succeeded", out)
        # CodeQL's skip is attributed (select green), never a false RED.
        self.assertNotIn("FAILED", out)

    def test_real_failure_still_fails_under_selection(self):
        # Invariant 2: selection must never mask a genuine failure.
        runs = [SEL_OK, RED, R("docs", conclusion="skipped")]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 1)
        self.assertIn("FAILED", out)
        self.assertIn("clippy", out)

    def test_select_failure_reds_even_when_all_siblings_green(self):
        # Invariant 3: an unobservable selection blocks the merge outright.
        runs = [R(SELECT_NAME, conclusion="failure"), GREEN, GREEN2]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 1)
        self.assertIn("selection pre-job", out)

    def test_select_skipped_or_neutral_is_not_success(self):
        # "did not succeed" is anything but success — a skipped/neutral select
        # would otherwise sneak through the generic skipped/neutral tolerance.
        for concl in ("skipped", "neutral", "cancelled", None):
            runs = [R(SELECT_NAME, conclusion=concl),
                    R("x", conclusion="skipped"), GREEN]
            code, out = run(tiny_cfg(), [runs])
            self.assertEqual(code, 1, f"select conclusion={concl!r} must RED the gate")
            self.assertIn("selection pre-job", out)

    def test_select_absent_preserves_legacy_skip_semantics(self):
        # Transition guarantee: no selection check-run on the commit (old sibling
        # sets / other repos' shapes) => skipped stays non-failing, as before.
        runs = [R("docs", conclusion="skipped"), GREEN]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 0)
        self.assertNotIn("selection pre-job", out)

    def test_multiple_select_runs_all_must_succeed(self):
        # ci.yml AND feature-matrix.yml each call the reusable select job — the
        # gate sees two same-named runs; ONE failing must RED the gate.
        runs = [SEL_OK, R(SELECT_NAME, conclusion="failure"), GREEN]
        code, _ = run(tiny_cfg(), [runs])
        self.assertEqual(code, 1)

    def test_cancelled_select_forgiven_by_same_name_success(self):
        # [OPUS-4.8] sq-fmx4u.3 hardening: the 2026-07-17 fleet jam. Under the
        # draft-tier + review-pipeline label churn a head SHA accretes many
        # concurrency-cancel rounds; a doomed select INSTANCE (cancelled) can
        # out-timestamp the winning run's already-concluded select SUCCESS, so
        # forgive_superseded's strictly-later rule leaves a residual cancelled
        # select. A same-normalized-name SUCCESS on the SHA proves the selection
        # was computed soundly (select is a deterministic pure pre-job over the
        # diff), so the gate must NOT RED over a provably-sound selection.
        runs = [R(SELECT_NAME, conclusion="success", started="2026-01-01T00:00:10Z", rid=2),
                R(SELECT_NAME, conclusion="cancelled", started="2026-01-01T00:00:20Z", rid=3),
                R("W3C conformance", conclusion="skipped"),
                GREEN]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 0, "a cancelled select superseded by a same-name success must not RED")
        self.assertIn("PASSED", out)

    def test_cancelled_select_with_no_success_still_reds(self):
        # The forgiveness is narrow: a cancelled select with NO successful
        # same-name sibling is still an unobservable selection and REDs.
        runs = [R(SELECT_NAME, conclusion="cancelled"),
                R("x", conclusion="skipped"), GREEN]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 1)
        self.assertIn("selection pre-job", out)

    def test_failure_select_reds_even_with_a_success_sibling(self):
        # Only cancelled/stale race losers are forgiven — a genuine `failure`
        # select still REDs regardless of any success sibling (a real selection
        # failure is never masked by a concurrency winner).
        runs = [R(SELECT_NAME, conclusion="success"),
                R(SELECT_NAME, conclusion="failure"),
                R("x", conclusion="skipped"), GREEN]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 1)
        self.assertIn("selection pre-job", out)

    def test_cancelled_draft_select_not_forgiven_by_full_success(self):
        # [OPUS-4.8] cross-provider review of PR #3417, finding 1: the any-success
        # rule is SAME-TIER only. A full-tier select success must NOT erase a
        # later cancelled DRAFT-tier instance — that instance must stay visible
        # (fail-closed: the draft_selects_unsuperseded hold accounts for draft
        # instances per-instance; erasing one by a cross-tier name collision
        # would under-count the hold). Strictly-later cross-tier supersession is
        # a different, still-supported path (the un-draft flow).
        runs = [R(SELECT_NAME, conclusion="success", started="2026-01-01T00:00:10Z", rid=2),
                R(SELECT_NAME + ", draft-tier", conclusion="cancelled",
                  started="2026-01-01T00:00:20Z", rid=3),
                R("x", conclusion="skipped"), GREEN]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 1, "a cross-tier success must not forgive a cancelled draft select")
        self.assertIn("selection pre-job", out)

    def test_cancelled_full_select_not_forgiven_by_draft_success(self):
        # [OPUS-4.8] finding 1, other direction: a DRAFT-tier select success must
        # never stand in for a cancelled FULL-tier selection (draft-assembled
        # evidence never satisfies a full-tier verdict, design #2537).
        runs = [R(SELECT_NAME + ", draft-tier", conclusion="success",
                  started="2026-01-01T00:00:10Z", rid=2),
                R(SELECT_NAME, conclusion="cancelled",
                  started="2026-01-01T00:00:20Z", rid=3),
                R("x", conclusion="skipped"), GREEN]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 1, "a draft-tier success must not forgive a cancelled full-tier select")
        self.assertIn("selection pre-job", out)

    def test_cancelled_compound_select_keeps_strictly_later_rule(self):
        # [OPUS-4.8] cross-provider review of PR #3417, finding 2: the compound
        # fv-select + fv-manifest job CONTAINS the selection phrase but carries
        # independent gating evidence (the proof-inventory manifest), so it is
        # NOT a pure select and must keep the strictly-later supersession rule:
        # an EARLIER success never forgives its later cancellation.
        compound = "fv-select (change-based test selection) + fv-manifest (proof inventory)"
        runs = [R(compound, conclusion="success", started="2026-01-01T00:00:10Z", rid=2),
                R(compound, conclusion="cancelled", started="2026-01-01T00:00:20Z", rid=3),
                R("x", conclusion="skipped"), GREEN]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 1, "an earlier success must not forgive a later cancelled compound select+evidence job")
        self.assertIn("selection pre-job", out)

    def test_pure_select_detection_contract(self):
        # is_pure_select gates ONLY the forgiveness widening: pure reusable
        # selector names (tier-marked or not) qualify; the compound
        # select+manifest job and non-select legs do not.
        self.assertTrue(g.is_pure_select(SELECT_NAME))
        self.assertTrue(g.is_pure_select(SELECT_NAME + ", draft-tier"))
        self.assertFalse(g.is_pure_select(
            "fv-select (change-based test selection) + fv-manifest (proof inventory)"))
        self.assertFalse(g.is_pure_select("build + test"))
        self.assertFalse(g.is_pure_select("gate"))

    def test_select_name_detection_contract(self):
        # The name-detection contract with .github/workflows/ci-select.yml (also
        # pinned cross-file by scripts/tests/test_ci_select_wiring.py).
        self.assertTrue(g.is_select(SELECT_NAME))
        self.assertFalse(g.is_select("build + test"))
        self.assertFalse(g.is_select("gate"))
        # And the select job must never be advisory-excluded.
        self.assertFalse(g.is_advisory(SELECT_NAME))


# [SONNET-4.6] Robustness-hardening tests (sq-90cv4 follow-up: Copilot gaps).
# Covers:
#   (a) _gh_json_lines converts subprocess raises (FileNotFoundError, TimeoutExpired)
#       → FetchError → routed into the existing bounded skip path, not a raw crash.
#   (b) fetch_queue_depth raising in run_gate → depth treated as None (unknown) →
#       NO saturation extension granted on depth alone (conservative branch).
# Mutation check: removing either try/except causes the test to go RED (the
# underlying exception propagates instead of being caught/re-raised as FetchError).
class TestSubprocessRobustness(unittest.TestCase):
    """[SONNET-4.6] subprocess raises in _gh_json_lines / fetch_queue_depth must be
    converted to graceful-degradation paths, never raw crashes."""

    def test_gh_json_lines_file_not_found_raises_fetch_error(self):
        """FileNotFoundError (gh not on PATH) must surface as FetchError so the
        caller's FetchError handler (bounded retry / skip-this-poll) applies."""
        from unittest.mock import patch
        with patch("subprocess.run", side_effect=FileNotFoundError("gh: not found")):
            with self.assertRaises(g.FetchError) as ctx:
                g._gh_json_lines(["repos/x/y"])
        self.assertIn("subprocess raised", str(ctx.exception))

    def test_gh_json_lines_timeout_raises_fetch_error(self):
        """TimeoutExpired must also surface as FetchError (same bounded-retry path)."""
        from unittest.mock import patch
        import subprocess as _sp
        with patch("subprocess.run", side_effect=_sp.TimeoutExpired("gh", 30)):
            with self.assertRaises(g.FetchError):
                g._gh_json_lines(["repos/x/y"])

    def test_subprocess_raise_routed_into_skip_tolerance(self):
        """A FileNotFoundError inside _gh_json_lines → FetchError → treated as a
        skipped poll (not a gate crash).  Two such errors then a green poll must pass
        (within the 3-failure tolerance of tiny_cfg)."""
        # Simulate _gh_json_lines raising FetchError (as it does after the fix for
        # FileNotFoundError) for the first two calls, then returning [GREEN].
        call_count = {"n": 0}

        def fake_fetch():
            n = call_count["n"]
            call_count["n"] += 1
            if n < 2:
                raise g.FetchError("subprocess raised: gh: not found")
            return [dict(GREEN)]

        out = io.StringIO()

        def depth_fn():
            return 0

        with redirect_stdout(out):
            code = g.run_gate(tiny_cfg(), fake_fetch, depth_fn, sleep_fn=lambda s: None)
        output = out.getvalue()
        self.assertEqual(code, 0)
        self.assertIn("skipping this poll", output)
        self.assertIn("PASSED", output)

    def test_fetch_queue_depth_raises_treated_as_unknown_no_extension(self):
        """When fetch_queue_depth() RAISES (subprocess spawn error inside the closure),
        run_gate must catch it, treat depth as None (unknown), and NOT grant a
        saturation extension — unknown depth with no progress is a genuine hang → RED.
        Mutation check: remove the try/except in run_gate → RuntimeError propagates
        instead of being caught → this test goes RED."""
        out = io.StringIO()
        fetch = scripted([[GREEN, IN_PROGRESS]])

        def raising_depth():
            raise RuntimeError("subprocess.run raised: gh not found")

        with redirect_stdout(out):
            code = g.run_gate(tiny_cfg(), fetch, raising_depth, sleep_fn=lambda s: None)
        output = out.getvalue()
        self.assertEqual(code, 1, "unknown-depth + no-progress must be a genuine hang RED")
        self.assertIn("genuine hang", output)
        self.assertNotIn("Extending the wait", output)

    def test_fetch_queue_depth_raises_progress_still_extends(self):
        """Unknown depth (depth_fn raises) does NOT block the progress-signal path:
        if completions are landing the gate still extends (progress alone suffices)."""
        # Six polls with decreasing pending; completions rising → progress=True.
        # depth_fn always raises (unknown). Verify extension activates then passes.
        polls = [
            [R("a"), R("b"), PENDING, R("d", status="queued", conclusion=None),
             R("e", status="queued", conclusion=None), R("f", status="queued", conclusion=None)],
            [R("a"), R("b"), R("coverage"), R("d", status="queued", conclusion=None),
             R("e", status="queued", conclusion=None), R("f", status="queued", conclusion=None)],
            [R("a"), R("b"), R("coverage"), R("d"),
             R("e", status="queued", conclusion=None), R("f", status="queued", conclusion=None)],
            [R("a"), R("b"), R("coverage"), R("d"), R("e"),
             R("f", status="queued", conclusion=None)],
            [R("a"), R("b"), R("coverage"), R("d"), R("e"), R("f")],
        ]
        out = io.StringIO()
        fetch = scripted(polls)

        def raising_depth():
            raise RuntimeError("depth API down")

        with redirect_stdout(out):
            code = g.run_gate(tiny_cfg(), fetch, raising_depth, sleep_fn=lambda s: None)
        self.assertEqual(code, 0)
        self.assertIn("PASSED", out.getvalue())


SELECT_FULL = "select / select (change-based test selection)"
SELECT_DRAFT = "select / select (change-based test selection, draft-tier)"


def draft_ctx(fetch_pr_draft, run_tier="draft", event="pull_request", retries=3):
    return g.TierContext(run_tier=run_tier, event_name=event,
                         fetch_pr_draft=fetch_pr_draft, draft_check_retries=retries)


def counting(value):
    """fetch_pr_draft stub returning `value` (an Exception instance raises)."""
    state = {"calls": 0}

    def fetch():
        state["calls"] += 1
        if isinstance(value, Exception):
            raise value
        return value

    fetch.state = state
    return fetch


class TestDraftTierIntegrity(unittest.TestCase):
    """[FABLE-5] Draft-tier CI: the invariant that a draft-tier gate result can
    NEVER admit a PR to the merge queue — conclusion-time draft re-check,
    stale-draft-tier leg-set belt, and superseded-cancellation forgiveness."""

    # ---- conclusion-time draft re-check ------------------------------------
    def test_draft_tier_success_requires_still_draft(self):
        fetch = counting(True)
        code, out = run(tiny_cfg(), [[R(SELECT_DRAFT), GREEN]],
                        tier_ctx=draft_ctx(fetch))
        self.assertEqual(code, 0)
        self.assertIn("DRAFT-TIER verdict", out)
        self.assertEqual(fetch.state["calls"], 1)

    def test_draft_tier_stale_on_ready_pr_fails(self):
        """The core invariant: an all-green draft-tier run whose PR was un-drafted
        before conclusion must RED with the supersession message."""
        code, out = run(tiny_cfg(), [[R(SELECT_DRAFT), GREEN]],
                        tier_ctx=draft_ctx(counting(False)))
        self.assertEqual(code, 1)
        self.assertIn("stale draft-tier run, full run pending", out)

    def test_draft_tier_unverifiable_state_fails_closed_bounded(self):
        fetch = counting(g.FetchError("api down"))
        code, out = run(tiny_cfg(), [[GREEN]], tier_ctx=draft_ctx(fetch, retries=3))
        self.assertEqual(code, 1)
        self.assertIn("could not confirm", out)
        self.assertEqual(fetch.state["calls"], 3, "retries must be bounded")

    def test_draft_tier_no_fetcher_fails_closed(self):
        code, out = run(tiny_cfg(), [[GREEN]], tier_ctx=draft_ctx(None))
        self.assertEqual(code, 1)

    def test_draft_tier_red_legs_skip_the_api_check(self):
        """A failing verdict REDs regardless; the draft re-check (an API call)
        must not even run — a RED can never be latched by the queue."""
        fetch = counting(True)
        code, out = run(tiny_cfg(), [[R(SELECT_DRAFT), RED]],
                        tier_ctx=draft_ctx(fetch))
        self.assertEqual(code, 1)
        self.assertEqual(fetch.state["calls"], 0)

    def test_draft_tier_empty_set_still_rechecks(self):
        """The stable-empty pass path must apply the same conclusion-time
        re-check — no bypass via an empty sibling set."""
        code, out = run(tiny_cfg(), [[]], tier_ctx=draft_ctx(counting(False)))
        self.assertEqual(code, 1)
        self.assertIn("stale draft-tier run", out)

    def test_full_tier_run_never_calls_the_draft_api(self):
        fetch = counting(True)
        code, _ = run(tiny_cfg(), [[GREEN]],
                      tier_ctx=draft_ctx(fetch, run_tier="full"))
        self.assertEqual(code, 0)
        self.assertEqual(fetch.state["calls"], 0)

    # ---- stale draft-tier leg-set belt (full-tier pull_request runs) --------
    def test_full_tier_holds_then_reds_on_unsuperseded_draft_select(self):
        """A full-tier pull_request gate over a draft-tier-assembled leg set must
        WAIT (the ready_for_review re-run is expected), and at budget exhaustion
        must RED — never conclude success over draft-tier legs."""
        polls = [[R(SELECT_DRAFT, started="2026-07-17T10:00:00Z", rid=1), GREEN]]
        code, out = run(tiny_cfg(), polls,
                        tier_ctx=draft_ctx(counting(True), run_tier="full"))
        self.assertEqual(code, 1)
        self.assertIn("awaiting the full-tier re-run", out)
        self.assertIn("stale draft-tier run, full run pending", out)

    def test_full_tier_passes_once_full_select_supersedes(self):
        draft_sel = R(SELECT_DRAFT, started="2026-07-17T10:00:00Z", rid=1)
        full_sel = R(SELECT_FULL, started="2026-07-17T10:05:00Z", rid=2)
        polls = [
            [draft_sel, GREEN],                       # full re-run not registered yet
            [draft_sel, full_sel, GREEN],             # it lands; set goes stable
            [draft_sel, full_sel, GREEN],
            [draft_sel, full_sel, GREEN],
        ]
        code, out = run(tiny_cfg(), polls,
                        tier_ctx=draft_ctx(counting(True), run_tier="full"))
        self.assertEqual(code, 0)
        self.assertIn("PASSED", out)

    def test_full_tier_requires_a_successor_per_draft_select_instance(self):
        """Cross-workflow collision: ci.yml, bench.yml, feature-matrix.yml and
        fuzz.yml all expose the IDENTICAL select check-run name, so a draft head
        carries FOUR draft-marked select instances. The hold must release only
        when EVERY instance has its own later full-tier successor — the first
        workflow's full-tier select must never release the other three."""
        drafts = [R(SELECT_DRAFT, started=f"2026-07-17T10:00:0{i}Z", rid=i)
                  for i in range(1, 5)]
        fulls = [R(SELECT_FULL, started=f"2026-07-17T10:05:0{i}Z", rid=10 + i)
                 for i in range(1, 5)]
        polls = [
            drafts + [GREEN],              # no full re-run registered yet: hold
            drafts + fulls[:1] + [GREEN],  # 1 of 4 registered: STILL hold
            drafts + fulls[:3] + [GREEN],  # 3 of 4 registered: STILL hold
            drafts + fulls + [GREEN],      # all four registered: settle + pass
            drafts + fulls + [GREEN],
            drafts + fulls + [GREEN],
        ]
        code, out = run(tiny_cfg(), polls,
                        tier_ctx=draft_ctx(counting(True), run_tier="full"))
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)
        # The per-instance hold must have been observable while 1..3 full-tier
        # selects were registered (the collision would have released at 1).
        self.assertIn("awaiting the full-tier re-run", out)

    def test_first_full_select_must_not_release_all_draft_instances(self):
        """REGRESSION (critic finding 2): with four draft-marked selects and only
        ONE later full-tier select ever registering, the gate must hold and then
        RED at budget exhaustion — never conclude success over the three
        workflows whose full-tier runs never registered any check-runs."""
        drafts = [R(SELECT_DRAFT, started=f"2026-07-17T10:00:0{i}Z", rid=i)
                  for i in range(1, 5)]
        one_full = R(SELECT_FULL, started="2026-07-17T10:05:00Z", rid=99)
        polls = [drafts + [one_full, GREEN]]
        code, out = run(tiny_cfg(), polls,
                        tier_ctx=draft_ctx(counting(True), run_tier="full"))
        self.assertEqual(code, 1)
        self.assertIn("awaiting the full-tier re-run", out)
        self.assertIn("stale draft-tier run, full run pending", out)
        self.assertIn("3 draft-marked select instance(s)", out)

    def test_full_tier_non_pr_event_ignores_draft_selects(self):
        """merge_group/push gates never see the belt (fresh ref; no PR payload)."""
        ctx = g.TierContext(run_tier="full", event_name="merge_group")
        code, _ = run(tiny_cfg(), [[R(SELECT_DRAFT), GREEN]], tier_ctx=ctx)
        self.assertEqual(code, 0)

    # ---- superseded-cancellation forgiveness --------------------------------
    def test_superseded_cancelled_forgiven(self):
        """A cancelled leg with a LATER same-named non-cancelled run (the
        ready_for_review concurrency-cancel artifact) must not RED the gate —
        tier-independent (matches branch protection's latest-run semantics)."""
        old = R("build + test", conclusion="cancelled",
                started="2026-07-17T10:00:00Z", rid=1)
        new = R("build + test", started="2026-07-17T10:05:00Z", rid=2)
        code, out = run(tiny_cfg(), [[old, new, GREEN2]])
        self.assertEqual(code, 0)
        self.assertIn("superseded-cancelled forgiven", out)

    def test_cancelled_without_successor_still_fails(self):
        code, _ = run(tiny_cfg(), [[R("build + test", conclusion="cancelled",
                                      started="2026-07-17T10:00:00Z", rid=1)]])
        self.assertEqual(code, 1)

    def test_genuine_failure_never_forgiven(self):
        """Only cancelled/stale are supersedable — a real FAILURE gates even with
        a later same-named success (no retry-away of a red)."""
        old = R("build + test", conclusion="failure",
                started="2026-07-17T10:00:00Z", rid=1)
        new = R("build + test", started="2026-07-17T10:05:00Z", rid=2)
        code, _ = run(tiny_cfg(), [[old, new]])
        self.assertEqual(code, 1)

    def test_cancelled_draft_select_forgiven_by_full_successor(self):
        """The draft-marked select cancelled mid-flight at un-draft has no later
        run under its OWN name — the tier-NORMALIZED name lets the full-tier
        select supersede it, so the select-health rule doesn't false-RED."""
        old = R(SELECT_DRAFT, conclusion="cancelled",
                started="2026-07-17T10:00:00Z", rid=1)
        new = R(SELECT_FULL, started="2026-07-17T10:05:00Z", rid=2)
        code, out = run(tiny_cfg(), [[old, new, GREEN]],
                        tier_ctx=draft_ctx(counting(True), run_tier="full"))
        self.assertEqual(code, 0)

    def test_cancelled_gate_predecessor_superseded_by_self(self):
        """The old draft-tier gate run cancelled by the ready_for_review
        concurrency group leaves a cancelled `gate` check-run on the SHA; THIS
        run's own (self-excluded) `gate` check-run must count as its superseder."""
        old_gate = R("gate", conclusion="cancelled",
                     started="2026-07-17T10:00:00Z", rid=1,
                     url="https://github.com/o/r/actions/runs/111/job/5")
        self_gate = R("gate", status="in_progress", conclusion=None,
                      started="2026-07-17T10:05:00Z", rid=2,
                      url="https://github.com/o/r/actions/runs/999/job/9")
        code, out = run(tiny_cfg(), [[old_gate, self_gate, GREEN]])
        self.assertEqual(code, 0, out)

    # ---- pure helpers -------------------------------------------------------
    def test_marker_helpers(self):
        self.assertTrue(g.is_draft_tier(SELECT_DRAFT))
        self.assertFalse(g.is_draft_tier(SELECT_FULL))
        self.assertEqual(g.normalized_name(SELECT_DRAFT), SELECT_FULL)
        self.assertTrue(g.is_select(SELECT_DRAFT),
                        "the draft-marked select must still satisfy SELECT_RE")
        self.assertFalse(g.is_advisory(SELECT_DRAFT))

    def test_draft_selects_unsuperseded_requires_strictly_later_full(self):
        draft_sel = R(SELECT_DRAFT, started="2026-07-17T10:05:00Z", rid=2)
        earlier_full = R(SELECT_FULL, started="2026-07-17T10:00:00Z", rid=1)
        self.assertEqual(g.draft_selects_unsuperseded([draft_sel, earlier_full]),
                         [SELECT_DRAFT],
                         "an EARLIER full select cannot supersede a draft one")
        later_full = R(SELECT_FULL, started="2026-07-17T10:06:00Z", rid=3)
        self.assertEqual(
            g.draft_selects_unsuperseded([draft_sel, earlier_full, later_full]), [])

    def test_draft_selects_unsuperseded_is_per_instance(self):
        """One full-tier successor covers exactly ONE draft-marked instance —
        same-named instances (the four selecting workflows) each need their own,
        and duplicates are preserved so the caller can report counts."""
        d1 = R(SELECT_DRAFT, started="2026-07-17T10:00:00Z", rid=1)
        d2 = R(SELECT_DRAFT, started="2026-07-17T10:00:01Z", rid=2)
        d3 = R(SELECT_DRAFT, started="2026-07-17T10:00:02Z", rid=3)
        f1 = R(SELECT_FULL, started="2026-07-17T10:05:00Z", rid=11)
        f2 = R(SELECT_FULL, started="2026-07-17T10:05:01Z", rid=12)
        f3 = R(SELECT_FULL, started="2026-07-17T10:05:02Z", rid=13)
        # 3 marked, 1 full => 2 instances remain unsuperseded (duplicates kept).
        self.assertEqual(g.draft_selects_unsuperseded([d1, d2, d3, f1]),
                         [SELECT_DRAFT, SELECT_DRAFT])
        # 3 marked, 3 later fulls => all matched.
        self.assertEqual(g.draft_selects_unsuperseded([d1, d2, d3, f1, f2, f3]), [])
        # An EARLIER full can never be a successor, even when otherwise unused.
        f_early = R(SELECT_FULL, started="2026-07-17T09:59:00Z", rid=10)
        self.assertEqual(g.draft_selects_unsuperseded([d1, d2, f_early, f1]),
                         [SELECT_DRAFT])
        # Interleaved rounds pair in start order: a full between two marked
        # instances supersedes the earlier one only.
        d_late = R(SELECT_DRAFT, started="2026-07-17T10:06:00Z", rid=4)
        self.assertEqual(g.draft_selects_unsuperseded([d1, f1, d_late]),
                         [SELECT_DRAFT])
        # started_at ties break on the check-run id (strictly-later still holds).
        d_tie = R(SELECT_DRAFT, started="2026-07-17T10:05:00Z", rid=20)
        f_tie = R(SELECT_FULL, started="2026-07-17T10:05:00Z", rid=21)
        self.assertEqual(g.draft_selects_unsuperseded([d_tie, f_tie]), [])
        self.assertEqual(
            g.draft_selects_unsuperseded(
                [d_tie, R(SELECT_FULL, started="2026-07-17T10:05:00Z", rid=19)]),
            [SELECT_DRAFT])

    # ---- draft-tier gate artifacts (the structural name tiering) -------------
    def test_gate_name_constants(self):
        """The gate's own tiered check-run name (ci-summary.yml renders
        `gate, draft-tier` on draft payloads) — pinned against the helpers."""
        self.assertEqual(g.GATE_CHECK_NAME, "gate")
        self.assertEqual(g.DRAFT_TIER_GATE_NAME, "gate, draft-tier")
        self.assertTrue(g.is_draft_gate_artifact("gate, draft-tier"))
        self.assertFalse(g.is_draft_gate_artifact("gate"),
                         "the full-tier gate name is NOT an artifact (a future "
                         "sibling job literally named `gate` must keep gating)")
        self.assertFalse(g.is_draft_gate_artifact("some gate, draft-tier"))
        self.assertEqual(g.normalized_name(g.DRAFT_TIER_GATE_NAME), "gate")
        self.assertTrue(g.is_draft_tier(g.DRAFT_TIER_GATE_NAME))
        self.assertFalse(g.is_advisory(g.DRAFT_TIER_GATE_NAME))
        self.assertFalse(g.is_select(g.DRAFT_TIER_GATE_NAME))

    def test_stale_draft_gate_failure_is_excluded_not_a_leg(self):
        """A COMPLETED draft-tier gate verdict left on the SHA is a tier
        artifact: its FAILURE must not permanently RED the full-tier gate on
        the same head (the live run re-derives the verdict over the real
        legs). Non-vacuous: without the exclusion this red would gate."""
        art = R(g.DRAFT_TIER_GATE_NAME, conclusion="failure",
                started="2026-07-17T10:00:00Z", rid=1,
                url="https://github.com/o/r/actions/runs/111/job/5")
        code, out = run(tiny_cfg(), [[art, GREEN, GREEN2]])
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)

    def test_cancelled_draft_gate_artifact_needs_no_successor(self):
        """A cancelled `gate, draft-tier` (concurrency-cancel at un-draft) is
        excluded as an artifact even before any successor registers — it must
        not RED the fresh full-tier gate while the sibling set settles."""
        art = R(g.DRAFT_TIER_GATE_NAME, conclusion="cancelled",
                started="2026-07-17T10:00:00Z", rid=1,
                url="https://github.com/o/r/actions/runs/111/job/5")
        code, out = run(tiny_cfg(), [[art, GREEN]])
        self.assertEqual(code, 0, out)


def probe(*values):
    """#3758 head-activity probe stub: yields values[i] on call i, repeating the
    last one. A value is written in the CALLER's shorthand and translated to the
    real three-state contract (PR #3765 finding 1):
      * int          => HeadActivityReport.confirmed(n) — an ACTUAL observation of n
                        non-terminal workflow runs on the head SHA (0 => the
                        awaiting_full hold cannot be satisfied by anything running)
      * None         => HeadActivityReport.unknown(...)  — probe unavailable
      * Exception    => RAISED from the probe (also UNKNOWN, via run_gate's guard)
      * a HeadActivityReport / anything else => returned verbatim, so a test can
                        hand back a raw value the production contract forbids."""
    state = {"i": 0, "calls": 0}

    def fetch():
        state["calls"] += 1
        value = values[min(state["i"], len(values) - 1)]
        state["i"] += 1
        if isinstance(value, Exception):
            raise value
        if value is None:
            return g.HeadActivityReport.unknown("probe unavailable (API failure)")
        if isinstance(value, int) and not isinstance(value, bool):
            return g.HeadActivityReport.confirmed(value)
        return value

    fetch.state = state
    return fetch


class TestUnsatisfiableDraftTierHold(unittest.TestCase):
    """[OPUS-5] #3758: an `awaiting_full` hold on a head whose Actions queue is
    IDLE and whose every sibling is terminal (pending == 0) is UNSATISFIABLE — no
    full-tier successor can ever register, so the hold pins `stable` at 0 forever
    and (pre-fix) only the ~68-minute absolute budget exited, while a GATE re-run
    re-derived the identical stall (PR #3470 head 552368bd, run 30002198053 x3).
    The exit is ADDITIVE: RED (never a silent pass), naming what actually clears
    the state; a SATISFIABLE hold still waits, and the pending>0 hang path and
    normal green sets are untouched."""

    # The live incident's shape: four draft-marked select instances (ci / bench /
    # feature-matrix / fuzz share one select name), zero full-tier selects, every
    # sibling terminal, PR still a draft.
    DRAFTS = [R(SELECT_DRAFT, started=f"2026-07-23T11:24:0{i}Z", rid=i)
              for i in range(1, 5)]

    def _hold_polls(self):
        return [list(self.DRAFTS) + [GREEN, GREEN2]]

    def test_unsatisfiable_hold_with_idle_head_reds_immediately(self):
        """Zero non-terminal workflow runs on the head for `unsat_grace_polls`
        consecutive polls => conclude at once, with the stale-draft-tier verdict
        and the "a gate re-run cannot clear this" remedy."""
        p = probe(0)
        cfg = tiny_cfg()
        code, out = run(cfg, self._hold_polls(),
                        tier_ctx=draft_ctx(counting(True), run_tier="full"),
                        head_activity=p)
        self.assertEqual(code, 1, out)
        self.assertIn("UNSATISFIABLE", out)
        self.assertIn("stale draft-tier run, full run pending", out)
        self.assertIn("4 draft-marked select instance(s)", out)
        # It must NOT have burned the budget: the grace is 2 confirming polls, so
        # the loop concludes well before the base budget (4) and absolute cap (8).
        self.assertLessEqual(p.state["calls"], cfg.unsat_grace_polls + 1)
        self.assertIn(f"concluding at poll {cfg.unsat_grace_polls}", out)
        self.assertNotIn("attempt 5:", out)

    def test_red_message_names_what_actually_clears_the_state(self):
        """LOAD-BEARING for the repair lanes (they re-ran the gate 3x): the RED
        must say a gate re-run cannot help, and name the three things that do."""
        code, out = run(tiny_cfg(), self._hold_polls(),
                        tier_ctx=draft_ctx(counting(True), run_tier="full"),
                        head_activity=probe(0))
        self.assertEqual(code, 1)
        self.assertIn("RE-RUNNING THIS `ci-summary` GATE CANNOT CLEAR THIS STATE", out)
        self.assertIn("re-dispatch the SELECTING workflows", out)
        self.assertIn("ready_for_review", out)
        self.assertIn("push a new head commit", out)

    def test_satisfiable_hold_still_waits_and_passes_when_the_full_select_lands(self):
        """A QUEUED full-tier run on the head (probe > 0) means the hold CAN be
        satisfied: the gate must keep waiting — and when the full-tier selects
        register it still PASSES. No early RED on a live head."""
        fulls = [R(SELECT_FULL, started=f"2026-07-23T11:40:0{i}Z", rid=10 + i)
                 for i in range(1, 5)]
        polls = [
            list(self.DRAFTS) + [GREEN, GREEN2],           # hold, head busy
            list(self.DRAFTS) + [GREEN, GREEN2],           # hold, head busy
            list(self.DRAFTS) + [GREEN, GREEN2],           # hold, head busy
            list(self.DRAFTS) + fulls + [GREEN, GREEN2],   # full-tier wave lands
            list(self.DRAFTS) + fulls + [GREEN, GREEN2],
            list(self.DRAFTS) + fulls + [GREEN, GREEN2],
        ]
        code, out = run(tiny_cfg(), polls,
                        tier_ctx=draft_ctx(counting(True), run_tier="full"),
                        head_activity=probe(2))
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)
        self.assertIn("could still register", out)
        self.assertNotIn("UNSATISFIABLE", out)

    def test_live_head_activity_vetoes_the_base_budget_shortcut(self):
        """A probe that keeps seeing a non-terminal run on this head contradicts
        unsatisfiability, so the base-budget shortcut must NOT fire either — the
        gate falls back to the pre-#3758 absolute-budget bound and REDs there."""
        code, out = run(tiny_cfg(), self._hold_polls(), depth=0,
                        tier_ctx=draft_ctx(counting(True), run_tier="full"),
                        head_activity=probe(1))
        self.assertEqual(code, 1)
        self.assertNotIn("UNSATISFIABLE", out)
        self.assertIn("attempt 8:", out)                 # rode the absolute cap
        self.assertIn("stale draft-tier run, full run pending", out)

    def test_a_queued_run_appearing_resets_the_confirming_streak(self):
        """The probe evidence must be CONSECUTIVE: idle, then a queued run, then
        idle again must never accumulate the 2-poll grace, so the FAST exit cannot
        fire. (The run still ends RED via the base-budget belt on the first idle
        poll at/after the base budget — never on the strength of non-consecutive
        idle observations.)"""
        code, out = run(tiny_cfg(), self._hold_polls(),
                        tier_ctx=draft_ctx(counting(True), run_tier="full"),
                        head_activity=probe(0, 3, 0, 3, 0, 3, 0, 3))
        self.assertEqual(code, 1)
        self.assertIn("1/2 confirming poll(s)", out)
        self.assertNotIn("2/2 confirming poll(s)", out)
        self.assertNotIn("concluding at poll 2", out)   # the grace never accrued
        self.assertIn("concluding at poll 5", out)      # base budget + the veto poll
        self.assertIn("base poll budget was reached", out)
        self.assertIn("stale draft-tier run, full run pending", out)

    def test_unknown_probe_never_concludes_unsatisfiability(self):
        """An unavailable probe (None) or one that RAISES is UNKNOWN — a
        NON-observation, which can never prove the hold unsatisfiable. [PR #3765
        finding 1] Neither exit may fire on it: the gate falls back to the
        pre-#3758 absolute-budget bound (poll 8) and REDs there via the
        stale-draft-tier belt, and it must never claim an idle queue it never saw."""
        for label, p in (("none", probe(None)),
                         ("raises", probe(RuntimeError("boom")))):
            with self.subTest(probe=label):
                code, out = run(tiny_cfg(), self._hold_polls(),
                                tier_ctx=draft_ctx(counting(True), run_tier="full"),
                                head_activity=p)
                self.assertEqual(code, 1, out)
                self.assertIn("probe unavailable" if label == "none"
                              else "probe raised", out)
                self.assertIn("probe is UNKNOWN", out)
                self.assertIn("stale draft-tier run, full run pending", out)
                # NOT concluded from a non-observation, and no idle-queue claim.
                self.assertNotIn("UNSATISFIABLE", out)
                self.assertNotIn("idle Actions queue", out)
                self.assertIn("attempt 8:", out)        # rode the absolute bound

    def test_no_probe_wired_never_takes_the_base_budget_shortcut(self):
        """[PR #3765 finding 1] With NO probe wired there is NO per-head evidence at
        all, so the widened base-budget branch must not fire either: the per-head
        probe is the authoritative satisfiability signal, and an unwired probe is
        UNKNOWN exactly like a failed one. The wait falls back to the pre-#3758
        absolute cap (poll 8) and still REDs there — bounded, just not "proven"."""
        code, out = run(tiny_cfg(), self._hold_polls(), depth=0,
                        tier_ctx=draft_ctx(counting(True), run_tier="full"))
        self.assertEqual(code, 1)
        self.assertNotIn("UNSATISFIABLE", out)
        self.assertIn("probe is UNKNOWN (no probe wired)", out)
        self.assertIn("attempt 8:", out)
        self.assertIn("stale draft-tier run, full run pending", out)

    def test_repo_saturation_does_not_extend_a_zero_pending_hold(self):
        """A DEEP repo-wide Actions queue is not evidence that THIS head's hold can
        be satisfied — nothing of ours is pending in that queue, and a real
        re-dispatch would show up as a queued run on the head. So a CONFIRMED-idle
        head still concludes at the base budget under saturation (contrast:
        test_saturation_extension_then_green, where pending > 0). The grace is
        lifted above the budget here so the fast exit cannot pre-empt this path."""
        code, out = run(tiny_cfg(unsat_grace_polls=99), self._hold_polls(), depth=50,
                        tier_ctx=draft_ctx(counting(True), run_tier="full"),
                        head_activity=probe(0))
        self.assertEqual(code, 1)
        self.assertIn("UNSATISFIABLE", out)
        self.assertIn("base poll budget was reached", out)
        self.assertIn("attempt 4:", out)
        self.assertNotIn("attempt 5:", out)
        self.assertNotIn("Extending the wait", out)
        # The evidence must not misreport a depth-50 queue as idle (PR #3765 f1).
        self.assertNotIn("idle Actions queue", out)
        self.assertIn("CONFIRMED ZERO queued or in-progress workflow runs", out)

    def test_live_progress_still_extends_a_zero_pending_hold(self):
        """Progress (the completed count still rising) DOES justify extending: the
        set is churning, so a full-tier successor may still be landing."""
        polls = [
            list(self.DRAFTS) + [GREEN],
            list(self.DRAFTS) + [GREEN, GREEN2],
            list(self.DRAFTS) + [GREEN, GREEN2, R("msrv")],
            list(self.DRAFTS) + [GREEN, GREEN2, R("msrv"), R("docs")],
            list(self.DRAFTS) + [GREEN, GREEN2, R("msrv"), R("docs"), R("wasm")],
            list(self.DRAFTS) + [GREEN, GREEN2, R("msrv"), R("docs"), R("wasm"),
                                 R("fuzz")],
        ]
        code, out = run(tiny_cfg(), polls, depth=0,
                        tier_ctx=draft_ctx(counting(True), run_tier="full"))
        self.assertEqual(code, 1)              # ends RED at the absolute cap belt
        self.assertIn("Extending the wait", out)
        self.assertIn("stale draft-tier run, full run pending", out)

    # ---- everything else must be untouched ---------------------------------
    def test_pending_hang_path_unchanged_by_the_probe(self):
        """(3) The pending>0 genuine-hang path keeps its own message and never
        routes through the unsatisfiable-hold exit, even with the probe wired and
        an awaiting_full hold up."""
        polls = [list(self.DRAFTS) + [GREEN, PENDING]]
        code, out = run(tiny_cfg(), polls, depth=0,
                        tier_ctx=draft_ctx(counting(True), run_tier="full"),
                        head_activity=probe(0))
        self.assertEqual(code, 1)
        self.assertIn("genuine hang, not a still-settling set", out)
        self.assertNotIn("UNSATISFIABLE", out)

    def test_probe_is_not_consulted_without_a_hold(self):
        """A normal green set must be unaffected: the probe is never called (no
        extra API traffic on the happy path) and the verdict is still PASS."""
        p = probe(0)
        code, out = run(tiny_cfg(), [[GREEN, GREEN2]],
                        tier_ctx=draft_ctx(counting(True), run_tier="full"),
                        head_activity=p)
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)
        self.assertEqual(p.state["calls"], 0)

    def test_probe_not_consulted_on_a_draft_tier_run(self):
        """awaiting_full is a FULL-tier pull_request concept; a draft-tier run must
        never consult the probe nor take the exit."""
        p = probe(0)
        code, out = run(tiny_cfg(), [[R(SELECT_DRAFT), GREEN]],
                        tier_ctx=draft_ctx(counting(True)), head_activity=p)
        self.assertEqual(code, 0, out)
        self.assertIn("DRAFT-TIER verdict", out)
        self.assertEqual(p.state["calls"], 0)

    def test_failing_leg_still_fails_fast_inside_an_unsatisfiable_hold(self):
        """The fail-fast belt keeps priority: a concluded gating FAILURE inside the
        hold must RED with the fail-fast message, not the unsatisfiability one."""
        polls = [list(self.DRAFTS) + [RED, GREEN]]
        code, out = run(tiny_cfg(), polls,
                        tier_ctx=draft_ctx(counting(True), run_tier="full"),
                        head_activity=probe(0))
        self.assertEqual(code, 1)
        self.assertIn("FAILED (fail-fast)", out)

    def test_exit_can_never_render_a_pass(self):
        """Belt: _conclude_unsatisfiable_hold delegates to render_verdict, and if a
        future refactor ever let that render a PASS over an unsatisfiable hold, the
        coercion must still RED."""
        cfg = tiny_cfg()
        out = io.StringIO()
        original = g.render_verdict
        try:
            g.render_verdict = lambda runs, path="", ctx=None: 0
            with redirect_stdout(out):
                code = g._conclude_unsatisfiable_hold(
                    list(self.DRAFTS) + [GREEN], cfg,
                    draft_ctx(counting(True), run_tier="full"),
                    attempt=3, evidence="a synthetic pass-rendering verdict")
        finally:
            g.render_verdict = original
        self.assertEqual(code, 1)
        self.assertIn("must never render as a pass", out.getvalue())

    def test_grace_default_is_a_bounded_minutes_scale_window(self):
        """Prod knob sanity: the grace must be several polls (registration window
        for the ready_for_review re-dispatch) but a small fraction of the absolute
        budget that used to be the only exit."""
        prod = g.Config()
        self.assertGreaterEqual(prod.unsat_grace_polls, 3)
        self.assertLess(prod.unsat_grace_polls * prod.interval, 10 * 60)
        self.assertLess(prod.unsat_grace_polls, prod.base_polls)


class TestHeadActivityProbeWiring(unittest.TestCase):
    """[OPUS-5] #3758: the live probe must count only NON-TERMINAL runs on the head
    and must exclude this gate's OWN workflow — two concurrent ci-summary runs on
    one head must not each count the other as a candidate successor (that mutual
    stall is the bug)."""

    def _probe(self, runs, self_run_id="999"):
        original = g.make_fetch_workflow_runs
        try:
            g.make_fetch_workflow_runs = lambda repo, sha, sid="": (lambda: list(runs))
            fetch = g.make_fetch_head_activity("o/r", "deadbeef", self_run_id)
        finally:
            g.make_fetch_workflow_runs = original
        with redirect_stdout(io.StringIO()):
            return fetch()

    def test_counts_only_non_terminal_foreign_runs(self):
        report = self._probe([
            W(999, 1, name="ci-summary", status="in_progress", conclusion=None),
            W(101, 2, name="CI"),                                     # completed
            W(102, 3, name="bench", status="queued", conclusion=None),
        ])
        self.assertEqual(report.state, g.HeadActivity.CONFIRMED_BUSY)
        self.assertEqual(report.count, 1)
        self.assertFalse(report.confirms_idle)

    def test_own_workflow_runs_never_count(self):
        """A SECOND ci-summary run on the same head (same workflow_id) is not a
        candidate successor — a gate run cannot register a full-tier select."""
        report = self._probe([
            W(999, 1, name="ci-summary", status="in_progress", conclusion=None),
            W(998, 1, name="ci-summary", status="in_progress", conclusion=None),
            W(101, 2, name="CI"),
        ])
        self.assertEqual(report.state, g.HeadActivity.CONFIRMED_IDLE)
        self.assertEqual(report.count, 0)
        self.assertTrue(report.confirms_idle)

    def test_all_terminal_head_is_zero(self):
        report = self._probe([W(999, 1, name="ci-summary", status="in_progress",
                                conclusion=None),
                              W(101, 2, name="CI"), W(102, 3, name="bench")])
        self.assertEqual(report.state, g.HeadActivity.CONFIRMED_IDLE)
        self.assertTrue(report.confirms_idle)

    def test_fetch_failure_is_unknown_not_zero(self):
        """A failed probe must be UNKNOWN — never a zero COUNT, which would assert
        an idle head and could fire the RED on no evidence (PR #3765 finding 1)."""
        original = g.make_fetch_workflow_runs
        try:
            def boom(repo, sha, sid=""):
                def fetch():
                    raise g.FetchError("api down")
                return fetch
            g.make_fetch_workflow_runs = boom
            fetch = g.make_fetch_head_activity("o/r", "deadbeef", "999")
        finally:
            g.make_fetch_workflow_runs = original
        with redirect_stdout(io.StringIO()):
            report = fetch()
        self.assertEqual(report.state, g.HeadActivity.UNKNOWN)
        self.assertIsNone(report.count)
        self.assertFalse(report.confirms_idle)
        self.assertIn("api down", report.detail)


class TestUnknownProbeIsNotConfirmedZero(unittest.TestCase):
    """[OPUS-5] PR #3765 cross-provider review, findings 1 + 2 — one bug class: an
    UNKNOWN probe result (absent / API failure / exception / non-report) was treated
    as a CONFIRMED-ZERO observation of the head's Actions queue.

    * FINDING 1 — the base-budget exit's veto was `head_busy` (confirmed non-zero
      ONLY), so an UNKNOWN probe silently SATISFIED the exit condition: a transient
      probe failure could RED a hold that was in fact SATISFIABLE, and the evidence
      line reported "an idle Actions queue (depth=50 < 5)" for a queue of depth 50
      it had never observed as idle.
    * FINDING 2 — `unsat_confirms` was not reset on UNKNOWN, so [zero, unknown,
      zero] reached a two-poll grace while claiming "2 consecutive poll(s)".

    The fix makes the probe THREE-STATE (`HeadActivityReport`): only
    `confirms_idle` is evidence; every non-observation falls through to continued
    polling and RESETS the confirming streak."""

    DRAFTS = TestUnsatisfiableDraftTierHold.DRAFTS
    FULLS = [R(SELECT_FULL, started=f"2026-07-23T11:40:0{i}Z", rid=10 + i)
             for i in range(1, 5)]

    def _hold_polls(self):
        return [list(self.DRAFTS) + [GREEN, GREEN2]]

    # ---- finding 1 ---------------------------------------------------------
    def test_transient_probe_failure_at_the_base_budget_does_not_red(self):
        """THE finding-1 regression. The head really IS busy (a full-tier wave is
        queued, so the hold is SATISFIABLE) but the probe blips at exactly the base
        budget poll. Pre-fix the blip took the base-budget exit and RED a
        satisfiable hold; the gate must instead keep polling — and when the
        full-tier selects land it PASSES. depth=50 pins the misreport too: a deep
        repo queue must never be logged as idle."""
        polls = [
            list(self.DRAFTS) + [GREEN, GREEN2],                   # 1  hold
            list(self.DRAFTS) + [GREEN, GREEN2],                   # 2  hold
            list(self.DRAFTS) + [GREEN, GREEN2],                   # 3  hold
            list(self.DRAFTS) + [GREEN, GREEN2],                   # 4  base budget
            list(self.DRAFTS) + self.FULLS + [GREEN, GREEN2],      # 5  wave lands
            list(self.DRAFTS) + self.FULLS + [GREEN, GREEN2],      # 6  settles
            list(self.DRAFTS) + self.FULLS + [GREEN, GREEN2],
        ]
        code, out = run(tiny_cfg(), polls, depth=50,
                        tier_ctx=draft_ctx(counting(True), run_tier="full"),
                        # busy, busy, busy, TRANSIENT FAILURE at the base budget
                        head_activity=probe(1, 1, 1, None, 1))
        self.assertEqual(code, 0, out)                  # no false RED
        self.assertIn("PASSED", out)
        self.assertNotIn("UNSATISFIABLE", out)
        self.assertIn("probe is UNKNOWN", out)          # honest reason logged
        self.assertIn("attempt 5:", out)                # it kept polling
        # ...and it never claimed the depth-50 queue was idle.
        self.assertNotIn("idle Actions queue", out)
        self.assertNotIn("depth=50 < 5", out)

    def test_unknown_probe_at_the_base_budget_logs_probe_unknown_not_idle(self):
        """The log-honesty half of finding 1: when the gate keeps polling because
        the probe is UNKNOWN it must SAY so, and must not attribute the wait to a
        throughput/saturation signal (saturation is not an extension reason for a
        zero-pending hold at all)."""
        code, out = run(tiny_cfg(), self._hold_polls(), depth=50,
                        tier_ctx=draft_ctx(counting(True), run_tier="full"),
                        head_activity=probe(None))
        self.assertEqual(code, 1)                       # still bounded: RED at cap
        self.assertIn("probe is UNKNOWN", out)
        self.assertIn("NOT observed", out)
        self.assertIn("deliberately", out)              # saturation is not the reason
        self.assertNotIn("throughput signal", out)
        self.assertNotIn("UNSATISFIABLE", out)
        self.assertNotIn("idle Actions queue", out)

    def test_a_bare_int_return_is_unknown_not_a_confirmed_count(self):
        """The ROOT CAUSE of finding 1 was a bare falsy return. The contract is now
        a HeadActivityReport, and a bare `0` — the pre-fix "idle head" value — must
        degrade to UNKNOWN rather than fire the RED."""
        code, out = run(tiny_cfg(), self._hold_polls(),
                        tier_ctx=draft_ctx(counting(True), run_tier="full"),
                        head_activity=lambda: 0)   # the FORBIDDEN bare-count return
        self.assertEqual(code, 1)
        self.assertNotIn("UNSATISFIABLE", out)
        self.assertIn("not a HeadActivityReport", out)
        self.assertIn("probe is UNKNOWN", out)
        self.assertIn("attempt 8:", out)

    def test_unknown_report_count_is_none_and_never_confirms_idle(self):
        """Type-level pin: `confirms_idle` must be a STATE test, not a truthiness
        test on the count — an UNKNOWN report's count is None, which is falsy."""
        unknown = g.HeadActivityReport.unknown("api down")
        self.assertIs(unknown.state, g.HeadActivity.UNKNOWN)
        self.assertIsNone(unknown.count)
        self.assertFalse(unknown.confirms_idle)
        self.assertFalse(bool(unknown.count))           # the trap it replaces
        idle = g.HeadActivityReport.confirmed(0)
        self.assertIs(idle.state, g.HeadActivity.CONFIRMED_IDLE)
        self.assertTrue(idle.confirms_idle)
        busy = g.HeadActivityReport.confirmed(3)
        self.assertIs(busy.state, g.HeadActivity.CONFIRMED_BUSY)
        self.assertFalse(busy.confirms_idle)

    # ---- finding 2 ---------------------------------------------------------
    def test_interleaved_unknown_resets_the_confirming_streak(self):
        """THE finding-2 regression. [confirmed-zero, UNKNOWN, confirmed-zero] must
        never reach the two-poll grace: "N consecutive polls" is a claim about N
        consecutive CONFIRMED observations. A probe that RAISES is handled
        identically to one that returns unknown. The budget knobs isolate the
        counter (base_polls above the cap), so ONLY the streak can conclude."""
        cfg = tiny_cfg(base_polls=99, max_total_polls=3, min_polls=1)
        for label, middle in (("none", None), ("raises", RuntimeError("boom"))):
            with self.subTest(unknown=label):
                code, out = run(cfg, self._hold_polls(),
                                tier_ctx=draft_ctx(counting(True), run_tier="full"),
                                head_activity=probe(0, middle, 0))
                self.assertEqual(code, 1, out)          # RED, but at the cap
                self.assertIn("1/2 confirming poll(s)", out)
                self.assertNotIn("2/2 confirming poll(s)", out)
                self.assertNotIn("UNSATISFIABLE", out)
                self.assertIn("attempt 3:", out)
                self.assertIn("stale draft-tier run, full run pending", out)

    def test_two_consecutive_confirmations_do_satisfy_the_grace(self):
        """The paired positive — without it the test above would also pass on a
        gate that had simply stopped concluding. Same knobs, no interleaved
        UNKNOWN: [confirmed-zero, confirmed-zero] DOES reach the grace and REDs at
        poll 2 naming the hold."""
        cfg = tiny_cfg(base_polls=99, max_total_polls=3, min_polls=1)
        code, out = run(cfg, self._hold_polls(),
                        tier_ctx=draft_ctx(counting(True), run_tier="full"),
                        head_activity=probe(0, 0))
        self.assertEqual(code, 1)
        self.assertIn("2/2 confirming poll(s)", out)
        self.assertIn("UNSATISFIABLE", out)
        self.assertIn("concluding at poll 2", out)
        self.assertIn("2 consecutive poll(s) CONFIRMED ZERO", out)
        self.assertNotIn("attempt 3:", out)

    def test_confirmed_busy_still_resets_the_streak(self):
        """Unchanged by the fix (and the contrast that shows the reset is about
        NON-observation, not just about zero-vs-nonzero): a confirmed BUSY poll
        also breaks the streak."""
        cfg = tiny_cfg(base_polls=99, max_total_polls=3, min_polls=1)
        code, out = run(cfg, self._hold_polls(),
                        tier_ctx=draft_ctx(counting(True), run_tier="full"),
                        head_activity=probe(0, 2, 0))
        self.assertEqual(code, 1)
        self.assertNotIn("2/2 confirming poll(s)", out)
        self.assertNotIn("UNSATISFIABLE", out)

    def test_a_skipped_poll_does_not_bridge_two_confirmations(self):
        """Same invariant, one layer out: a poll whose check-run fetch FAILED
        observed nothing at all (not even the hold), so it cannot sit inside a run
        of consecutive confirmations either."""
        cfg = tiny_cfg(base_polls=99, max_total_polls=3, min_polls=1)
        polls = [
            list(self.DRAFTS) + [GREEN, GREEN2],
            g.FetchError("transient"),
            list(self.DRAFTS) + [GREEN, GREEN2],
        ]
        code, out = run(cfg, polls,
                        tier_ctx=draft_ctx(counting(True), run_tier="full"),
                        head_activity=probe(0))
        self.assertEqual(code, 1)
        self.assertIn("check-run fetch failed", out)
        self.assertNotIn("2/2 confirming poll(s)", out)


class TestMergeGroupChangeClassAccounting(unittest.TestCase):
    """[FABLE-5] merge-group change-class gate (extends #3420/#3421): the expected
    per-class leg accounting for a MERGE-GROUP sibling set. On a docs-only/
    orchestration-only queued batch the rust_changed layer (ci.yml lint/msrv/
    geiger/docker-smoke/coverage-floors + feature-matrix setup/check-tier/
    fedclient-boundary) now SKIPS alongside the select-gated engine legs; every
    such leg still registers a check-run with conclusion `skipped` — an
    ATTRIBUTED skip under the green same-diff `select` pre-job — and the gate
    renders PASS. The fail-closed half: those same skips with a non-success
    select are UNATTRIBUTABLE and must RED (an engine batch could only see its
    legs skipped through a select that did not soundly conclude — the classify
    step and the select job compute the same classify_change over the same
    base_sha...head_sha, so on any resolved diff they cannot disagree)."""

    # The #2533-shaped docs-only merge-group batch: what the queue's merge_group
    # ref shows after this change — every engine-lane check-run PRESENT (never
    # missing) with conclusion `skipped`, cheap prose gates green, select green.
    _DOCS_ONLY_GROUP_SKIPS = [
        # ci.yml select-conjunct legs (already skipped via the batch-diff selection):
        "test (load-aware shard bulk 1/3)",
        "build + archive test binaries (+ doctests once)",
        "W3C SPARQL 1.1 conformance (ratchet)",
        # ci.yml rust_changed-only legs (NEWLY class-gated on merge_group):
        "lint (fmt + clippy + doc)",
        "MSRV build (workspace)",
        "cargo-geiger unsafe audit",
        "docker image smoke (bind posture + /health)",
        # feature-matrix.yml rust_changed-only legs (NEWLY class-gated):
        "assemble feature matrix",
        "feature-matrix check-tier (engine)",
        "fedclient dependency-boundary guard",
        "opt-in sparq-engine (paths)",
        # fuzz.yml (already select-gated on the batch diff):
        "cargo-fuzz (parsers + mmap loader, bounded)",
        "differential smoke (sparq vs Oxigraph, fixed regression windows)",
    ]

    def _group_runs(self, select_run):
        runs = [select_run]
        runs += [R(n, conclusion="skipped") for n in self._DOCS_ONLY_GROUP_SKIPS]
        runs += [
            R("docs-quality quick-gates"),
            R("supply-chain gates (deny + vet + SBOM + VEX + OpenSSF + js-sbom)"),
            # Deliberately FULL on merge groups (the documented #3421 unsound-skip
            # exception): wasm feature-OFF byte-equality on the MERGED bundle.
            R("vectorized wasm feature-OFF equality"),
        ]
        return runs

    def test_docs_only_merge_group_passes_with_attributed_skips(self):
        # The headline fixture: class=docs-only batch => gate PASS, every engine
        # leg an attributed skip, nothing missing. (No tier_ctx: merge_group runs
        # use the pre-draft-tier semantics, as in production.)
        runs = self._group_runs(SEL_OK)
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)
        self.assertIn(f"{len(self._DOCS_ONLY_GROUP_SKIPS)} skipped", out)
        self.assertIn("selection pre-job succeeded", out)
        self.assertNotIn("FAILED", out)

    def test_same_skips_with_failed_select_red(self):
        # Fail-closed: the identical skip set under a FAILED select is
        # unattributable => RED. This is what stops an engine batch from merging
        # on skipped legs — its legs can only skip through the select/classify
        # pair, and a select that did not conclude success reds the whole set.
        runs = self._group_runs(R(SELECT_NAME, conclusion="failure"))
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 1, out)
        self.assertIn("cannot be attributed to a sound selection", out)

    def test_same_skips_with_missing_select_conclusion_red(self):
        # A cancelled (unsuperseded) select is equally unattributable => RED.
        runs = self._group_runs(R(SELECT_NAME, conclusion="cancelled"))
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 1, out)

    def test_engine_leg_failure_never_masked_by_class_skips(self):
        # A genuinely failing sibling on the merge_group ref (e.g. the always-on
        # supply-chain SBOM steps, or a lane the class kept running) still REDs
        # the gate regardless of how many class-attributed skips surround it.
        runs = self._group_runs(SEL_OK) + [R("vectorized wasm feature-OFF equality (run)",
                                             conclusion="failure")]
        code, out = run(tiny_cfg(), [runs])
        self.assertEqual(code, 1, out)
        self.assertIn("FAILED", out)


FM_GROUP = R("opt-in group (sparq-engine 1/2)")          # a group job's own conclusion
FM_GROUP2 = R("opt-in group (sparq-server 1/1)")
FM_REPORT_OK = R("feature-matrix report")                 # reporter posted green
FM_REPORT_FAIL = R("feature-matrix report", conclusion="failure")
FM_REPORT_PENDING = R("feature-matrix report", status="in_progress", conclusion=None)


class TestFeatureMatrixReporterAwait(unittest.TestCase):
    """[FABLE-5] PR #3511 finding 1 (HIGH): ci-summary must STRUCTURALLY await the
    trusted feature-matrix reporter. When `opt-in group (…)` legs ran for this head
    (the reporter-timing-independent proof that legs were selected), the
    `feature-matrix report` summary check-run MUST exist and be terminal-SUCCESS
    before the gate can conclude green; a delayed/crashed reporter must never race
    past the gate — absence keeps the gate polling and FAILS CLOSED at timeout."""

    # ---- pure predicates ----------------------------------------------------
    def test_zero_leg_skeleton_placeholder_is_not_group_presence(self):
        """The unexpanded `${{ matrix.group }}` skeleton (zero-leg run, skipped) must
        NOT trigger the reporter requirement — production incident PR #3524: every
        docs/config-only PR timed out RED awaiting a verdict the reporter correctly
        never posts on a zero-leg run."""
        runs = [
            {"name": "opt-in group (${{ matrix.group }})", "status": "completed",
             "conclusion": "skipped", "details_url": ""},
        ]
        self.assertFalse(g.is_real_fm_group(runs[0]))
        self.assertEqual(g.fm_report_status(runs), "n/a")

    def test_forged_placeholder_name_with_success_still_requires_reporter(self):
        """SECURITY (sol on #3525): a REAL successful group whose PR-controlled name
        embeds `${{` must NOT masquerade as the skeleton — the exclusion requires
        the server-set skipped conclusion too."""
        runs = [
            {"name": "opt-in group (g01 ${{ attacker)", "status": "completed",
             "conclusion": "success", "details_url": ""},
        ]
        self.assertTrue(g.is_real_fm_group(runs[0]))
        self.assertEqual(g.fm_report_status(runs), "pending")

    def test_real_group_name_still_counts(self):
        runs = [
            {"name": "opt-in group (g01 sparq-engine)", "status": "completed",
             "conclusion": "success", "details_url": "https://github.com/o/r/actions/runs/123/job/9"},
        ]
        self.assertTrue(g.is_fm_group(runs[0]["name"]))
        self.assertEqual(g.fm_report_status(runs), "pending")

    def test_predicate_contract(self):
        self.assertTrue(g.is_fm_group("opt-in group (sparq-engine 1/2)"))
        self.assertFalse(g.is_fm_group("opt-in sparq-engine (paths)"),
                         "a per-LEG `opt-in <name>` check is not a group job")
        self.assertFalse(g.is_fm_group("feature-matrix report"))
        self.assertTrue(g.is_fm_report("feature-matrix report"))
        self.assertFalse(g.is_fm_report("feature-matrix report (extra)"))
        # Neither is advisory-excluded (both gate/participate normally).
        self.assertFalse(g.is_advisory("opt-in group (sparq-engine 1/2)"))
        self.assertFalse(g.is_advisory("feature-matrix report"))

    def test_status_helper(self):
        self.assertEqual(g.fm_report_status([GREEN, GREEN2]), "n/a")
        self.assertEqual(g.fm_report_status([FM_GROUP, GREEN]), "pending")
        self.assertEqual(g.fm_report_status([FM_GROUP, FM_REPORT_PENDING]), "pending")
        self.assertEqual(g.fm_report_status([FM_GROUP, FM_REPORT_OK]), "ok")
        self.assertEqual(g.fm_report_status([FM_GROUP, FM_REPORT_FAIL]), "failed")

    # ---- present-success => green ------------------------------------------
    def test_reporter_present_success_passes(self):
        code, out = run(tiny_cfg(), [[FM_GROUP, FM_GROUP2, FM_REPORT_OK, GREEN]])
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)

    def test_no_group_jobs_needs_no_reporter(self):
        """A doc-only PR (or a fully change-selected-out matrix / a merge_group
        that skipped the lane) has NO `opt-in group (…)` check-run, so no reporter
        is expected — the gate must not invent a requirement and false-RED."""
        code, out = run(tiny_cfg(), [[GREEN, GREEN2]])
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)

    # ---- crashed reporter (failure conclusion) => gate fails ----------------
    def test_reporter_failure_reds(self):
        code, out = run(tiny_cfg(), [[FM_GROUP, FM_REPORT_FAIL, GREEN]])
        self.assertEqual(code, 1, out)
        self.assertIn("FAILED", out)
        # The dedicated reporter belt names the reporter (belt-and-braces on top of
        # the normal gating-set render, which would also RED the failed check).
        self.assertIn("feature-matrix report", out)

    # ---- absent reporter => not concludable; fail-closed at timeout ---------
    def test_absent_reporter_never_concludes_green_holds_then_reds(self):
        """The core finding: group legs green, but the reporter's check-run NEVER
        lands. The gate must NOT conclude green in the settle window (it holds,
        still-settling), and at budget exhaustion FAILS CLOSED — never a
        conclude-by-timing over the group jobs' bare successes."""
        # Every poll: group jobs terminal-green, but NO `feature-matrix report`.
        code, out = run(tiny_cfg(), [[FM_GROUP, GREEN]])
        self.assertEqual(code, 1, out)
        self.assertIn("awaiting the feature-matrix reporter verdict", out)
        self.assertIn("reporter verdict never landed", out)
        self.assertNotIn("PASSED", out)

    def test_absent_reporter_holds_settle_window_open(self):
        """Non-vacuity: the SAME green group set WITH the reporter present passes
        immediately, proving the RED above is caused by the missing reporter, not
        by an unrelated hang."""
        code, out = run(tiny_cfg(), [[FM_GROUP, FM_REPORT_OK, GREEN]])
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)

    def test_reporter_lands_late_then_passes(self):
        """The realistic race: the reporter (workflow_run) posts its verdict a few
        polls AFTER the group jobs finish. The gate holds until it lands, then the
        settle completes and it passes — the whole point of the structural await."""
        polls = [
            [FM_GROUP, GREEN],                    # groups done; reporter not in yet
            [FM_GROUP, GREEN],                    # still awaiting
            [FM_GROUP, FM_REPORT_OK, GREEN],      # reporter verdict lands
            [FM_GROUP, FM_REPORT_OK, GREEN],      # settle
            [FM_GROUP, FM_REPORT_OK, GREEN],
        ]
        code, out = run(tiny_cfg(), polls)
        self.assertEqual(code, 0, out)
        self.assertIn("awaiting the feature-matrix reporter verdict", out)
        self.assertIn("PASSED", out)

    def test_reporter_pending_nonterminal_holds(self):
        """A present-but-in-progress reporter check must also hold (not conclude)."""
        polls = [
            [FM_GROUP, FM_REPORT_PENDING, GREEN],           # reporter in flight
            [FM_GROUP, FM_REPORT_PENDING, GREEN],
            [FM_GROUP, FM_REPORT_OK, GREEN],                # it finishes green
            [FM_GROUP, FM_REPORT_OK, GREEN],
            [FM_GROUP, FM_REPORT_OK, GREEN],
        ]
        code, out = run(tiny_cfg(), polls)
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)

    def test_failing_leg_still_fails_fast_while_awaiting_reporter(self):
        """A genuine failing gating leg must still fail FAST even while the gate
        is (correctly) holding for a not-yet-landed reporter verdict — the
        reporter await never masks a real failure."""
        polls = [
            [FM_GROUP, RED],   # group present (await armed) + a red leg, no reporter
            [FM_GROUP, RED],   # grace re-poll re-observes the same failure
        ]
        code, out = run(tiny_cfg(), polls)
        self.assertEqual(code, 1, out)
        self.assertIn("fail-fast", out)


# --- PR #3511 finding 2 (HIGH): same-SHA stale-report correlation ---------------
# feature-matrix reruns on the SAME head SHA (ready_for_review / label events), so a
# STALE `feature-matrix report` (success) from an EARLIER run can sit on the commit
# while the CURRENT run's reporter is delayed/crashed. The fix binds each report to
# its triggering feature-matrix run id (external_id) and requires it to equal the
# CURRENT `opt-in group (…)` run id (max `/actions/runs/<id>` across the group checks).
def _grp(run_id, name="opt-in group (sparq-engine 1/2)"):
    return R(name, url=f"https://github.com/o/r/actions/runs/{run_id}/job/7")


def _rep(run_id, conclusion="success", status="completed"):
    return R("feature-matrix report", status=status, conclusion=conclusion,
             external_id=str(run_id))


class TestFeatureMatrixReporterCorrelation(unittest.TestCase):
    """[FABLE-5] PR #3511 finding 2 (HIGH): a `feature-matrix report` verdict must
    correlate to the CURRENT feature-matrix run — a stale same-SHA report from an
    earlier run can never satisfy the reporter-await for a fresh group run."""

    def test_group_run_id_extraction_takes_the_latest(self):
        # Two group runs on the same SHA (stale 111, fresh 222); the max wins.
        runs = [_grp("111"), _grp("222", "opt-in group (sparq-server 1/1)"), GREEN]
        self.assertEqual(g.fm_group_run_id(runs), "222")
        # No parseable url => "" (graceful degradation, never a crash).
        self.assertEqual(g.fm_group_run_id([FM_GROUP, GREEN]), "")
        # html_url fallback when details_url is absent.
        hurl = {"name": "opt-in group (x 1/1)", "status": "completed",
                "conclusion": "success", "details_url": "",
                "html_url": "https://github.com/o/r/actions/runs/333/job/1"}
        self.assertEqual(g.fm_group_run_id([hurl]), "333")

    def test_matching_run_success_is_ok(self):
        runs = [_grp("222"), _rep("222"), GREEN]
        self.assertEqual(g.fm_report_status(runs), "ok")

    def test_matching_run_failure_is_failed(self):
        runs = [_grp("222"), _rep("222", conclusion="failure"), GREEN]
        self.assertEqual(g.fm_report_status(runs), "failed")

    def test_duplicate_run_same_sha_stale_success_still_awaiting(self):
        """THE CORE RACE: the fresh group run is 222, but the only report on the SHA
        is the STALE run 111's green verdict. Its external_id (111) != current run id
        (222), so it is IGNORED — the gate is still awaiting the CURRENT run's
        reporter, NOT satisfied by the stale success."""
        runs = [_grp("222"), _rep("111"), GREEN]  # stale success masquerading
        self.assertEqual(g.fm_report_status(runs), "pending")

    def test_stale_plus_fresh_uses_fresh_verdict(self):
        # Stale 111 success sits alongside the fresh 222 report; the fresh one is
        # judged. Fresh green => ok; fresh red => failed (stale success cannot save).
        self.assertEqual(
            g.fm_report_status([_grp("222"), _rep("111"), _rep("222"), GREEN]), "ok")
        self.assertEqual(
            g.fm_report_status(
                [_grp("222"), _rep("111"), _rep("222", conclusion="failure"), GREEN]),
            "failed")

    def test_fresh_pending_alongside_stale_success_holds(self):
        # Stale 111 is green but the CURRENT 222 reporter is still in flight => hold.
        runs = [_grp("222"), _rep("111"),
                _rep("222", status="in_progress", conclusion=None), GREEN]
        self.assertEqual(g.fm_report_status(runs), "pending")

    def test_no_external_id_falls_back_to_any_report(self):
        # Legacy reporter (no external_id) + resolvable group run id: degrade to the
        # pre-finding-2 any-report match (never a false RED). FM_REPORT_OK has no
        # external_id; the group has a run id.
        runs = [_grp("222"), FM_REPORT_OK, GREEN]
        self.assertEqual(g.fm_report_status(runs), "ok")

    def test_unresolvable_group_run_id_falls_back_to_any_report(self):
        # Group check has no parseable run id (FM_GROUP has empty url) but the report
        # carries an external_id: with no current run id to bind to, degrade to
        # any-report matching rather than hang forever.
        runs = [FM_GROUP, _rep("999"), GREEN]
        self.assertEqual(g.fm_report_status(runs), "ok")

    def test_stale_success_gate_holds_then_reds_end_to_end(self):
        """End-to-end through the poll loop: every poll has the fresh group run (222)
        green + only the STALE run 111's green report. The gate must NOT conclude on
        the stale success — it holds, then FAILS CLOSED at budget exhaustion."""
        code, out = run(tiny_cfg(), [[_grp("222"), _rep("111"), GREEN]])
        self.assertEqual(code, 1, out)
        self.assertIn("awaiting the feature-matrix reporter verdict", out)
        self.assertIn("reporter verdict never landed", out)

    def test_fresh_report_lands_after_stale_then_passes_end_to_end(self):
        """The realistic recovery: a stale green report sits from run 111, then the
        CURRENT run 222's reporter posts its own green verdict; the gate correlates,
        settles on the fresh verdict, and passes."""
        polls = [
            [_grp("222"), _rep("111"), GREEN],                 # only stale => holding
            [_grp("222"), _rep("111"), GREEN],                 # still awaiting fresh
            [_grp("222"), _rep("111"), _rep("222"), GREEN],    # fresh verdict lands
            [_grp("222"), _rep("111"), _rep("222"), GREEN],    # settle
            [_grp("222"), _rep("111"), _rep("222"), GREEN],
        ]
        code, out = run(tiny_cfg(), polls)
        self.assertEqual(code, 0, out)
        self.assertIn("awaiting the feature-matrix reporter verdict", out)
        self.assertIn("PASSED", out)


# --- Fix round 3 (fable): latest-run-relative presence -------------------------
# Same-SHA zero-leg rerun (ready_for_review / label) leaves an OLDER real group run's
# check-runs on the commit alongside a NEWER zero-leg skeleton run. Keying presence
# off ANY real group (the old run's) while keying the run id off the newer skeleton
# deadlocked the gate: current_run_id = the skeleton's id, no report ever carries it
# (zero-leg posts none), matched=[] => "pending" => timeout RED. FIX: presence AND the
# reporter requirement are decided relative to the LATEST feature-matrix run.
def _skel(run_id="", conclusion="skipped"):
    """The zero-leg skeleton check-run (unexpanded placeholder + server-set skipped),
    optionally carrying its own run id url (it is a check-run of that run)."""
    url = f"https://github.com/o/r/actions/runs/{run_id}/job/1" if run_id else ""
    return R("opt-in group (${{ matrix.group }})", conclusion=conclusion, url=url)


class TestLatestRunRelativePresence(unittest.TestCase):
    """[FABLE-5] fix round 3: a same-SHA zero-leg rerun must CONCLUDE (n/a), not
    deadlock awaiting a reporter the zero-leg run correctly never posts."""

    def test_a_mixed_old_real_plus_new_skeleton_is_na(self):
        """(a) THE DEADLOCK: an OLD real group run (111) + its report(111) sit on the
        SHA next to a NEWER zero-leg skeleton run (222). The latest run (222) is
        zero-leg, so NO reporter is expected — the gate must conclude n/a, NOT await
        run 222's absent report and time out RED."""
        runs = [_grp("111"), _rep("111"), _skel("222")]
        # current run id is the newer skeleton's (222) — the very shape that deadlocked.
        self.assertEqual(g.fm_group_run_id(runs), "222")
        self.assertEqual(g.fm_report_status(runs), "n/a")

    def test_a_end_to_end_concludes_without_awaiting(self):
        """(a) end-to-end: the mixed old-real + new-skeleton head PASSES immediately
        (no reporter await), never RED-on-timeout."""
        code, out = run(tiny_cfg(), [[_grp("111"), _rep("111"), _skel("222"), GREEN]])
        self.assertEqual(code, 0, out)
        self.assertIn("PASSED", out)
        self.assertNotIn("reporter verdict never landed", out)

    def test_b_old_skeleton_plus_new_real_awaits_new_reporter(self):
        """(b) reversed order: an OLD zero-leg skeleton (111) sits next to the NEWER
        real run (222). The latest run has real legs => require ITS reporter. Absent
        => pending; present with external_id == 222 => ok; a stale-only 111 report
        (there is none here) could never satisfy it."""
        pending = [_skel("111"), _grp("222"), GREEN]
        self.assertEqual(g.fm_group_run_id(pending), "222")
        self.assertEqual(g.fm_report_status(pending), "pending")
        ok = [_skel("111"), _grp("222"), _rep("222"), GREEN]
        self.assertEqual(g.fm_report_status(ok), "ok")
        # A report only for the OLD run id does NOT satisfy the latest real run.
        stale_only = [_skel("111"), _grp("222"), _rep("111"), GREEN]
        self.assertEqual(g.fm_report_status(stale_only), "pending")

    def test_c_isolated_skeleton_is_na(self):
        """(c) existing invariant preserved: an isolated zero-leg skeleton (with OR
        without a locating url) requires no reporter."""
        self.assertEqual(g.fm_report_status([_skel()]), "n/a")
        self.assertEqual(g.fm_report_status([_skel("222")]), "n/a")

    def test_d_forged_placeholder_success_is_latest_still_requires_reporter(self):
        """(d) SECURITY (r2 carried forward): a REAL successful group whose PR-controlled
        name embeds the `${{` placeholder counts as REAL for the run it belongs to. When
        it is the LATEST run, the reporter is STILL required — it must not drop the
        requirement by masquerading as the zero-leg skeleton (which needs a server-set
        `skipped`, not `success`)."""
        forged = R("opt-in group (g01 ${{ attacker)", conclusion="success",
                   url="https://github.com/o/r/actions/runs/222/job/3")
        self.assertTrue(g.is_real_fm_group(forged))
        # Latest run 222 is this forged-but-real group => require reporter.
        self.assertEqual(g.fm_report_status([forged, GREEN]), "pending")
        # Its reporter, correlated to 222, satisfies it.
        self.assertEqual(g.fm_report_status([forged, _rep("222"), GREEN]), "ok")
        # A zero-leg skeleton on an OLDER run (111) does not drop the requirement.
        self.assertEqual(g.fm_report_status([_skel("111"), forged, GREEN]), "pending")

    def test_e_real_group_unparseable_url_fails_closed(self):
        """(e) fail-closed semantics: a REAL group check with NO parseable run id means
        real legs ran on a run we cannot identify — we must NOT declare the latest run
        zero-leg. The reporter requirement is kept (degrading to any-report correlation),
        never n/a. FM_GROUP has an empty url; alone it holds pending, and it must keep
        the requirement even when a NEWER skeleton carries a parseable id."""
        # Real group, no url => pending (require reporter), never n/a.
        self.assertEqual(g.fm_report_status([FM_GROUP, GREEN]), "pending")
        # A real-unparseable group is NOT masked away by a parseable skeleton: the
        # skeleton (222) would otherwise be "the latest run" and look zero-leg, but the
        # unparseable real leg forbids that n/a — fail closed to require the reporter.
        self.assertEqual(g.fm_report_status([FM_GROUP, _skel("222"), GREEN]), "pending")
        # With any legacy report present it degrades to any-report (never a false RED).
        self.assertEqual(g.fm_report_status([FM_GROUP, FM_REPORT_OK, GREEN]), "ok")


if __name__ == "__main__":
    unittest.main(verbosity=2)
