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


def R(name, status="completed", conclusion="success", url="", started="", rid=0):
    # started/rid feed the draft-tier superseded-run ordering (started_at, id);
    # pre-draft-tier tests omit them and keep their exact semantics.
    return {"name": name, "status": status, "conclusion": conclusion,
            "details_url": url, "html_url": "", "started_at": started, "id": rid}


GREEN = R("build + test")
GREEN2 = R("clippy")
PENDING = R("coverage", status="queued", conclusion=None)
IN_PROGRESS = R("coverage", status="in_progress", conclusion=None)
RED = R("clippy", conclusion="failure")


def tiny_cfg(**over):
    """Small budgets so the loop phases are reachable in a handful of polls:
    base budget = 4 polls, absolute cap = 8, settle = 2, floor = 2."""
    base = dict(self_run_id="999", interval=0, min_polls=2, settle_polls=2,
                base_polls=4, sat_interval=0, max_total_polls=8,
                sat_queue_min=5, progress_window=2, max_consec_fetch_failures=3,
                summary_path="")
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


def run(cfg, polls, depth=0, tier_ctx=None):
    """Drive run_gate with scripted polls + a constant queue depth (None = the
    depth API is unavailable). Returns (exit_code, captured_output). tier_ctx
    (default None) exercises the draft-tier integrity semantics."""
    out = io.StringIO()
    fetch = scripted(polls)

    def depth_fn():
        return depth

    with redirect_stdout(out):
        code = g.run_gate(cfg, fetch, depth_fn, sleep_fn=lambda s: None,
                          tier_ctx=tier_ctx)
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


class TestAdvisoryRule(unittest.TestCase):
    def test_word_boundary_rule(self):
        self.assertTrue(g.is_advisory("markdownlint-advisory (whole repo)"))
        self.assertTrue(g.is_advisory("external-links (lychee online, advisory)"))
        self.assertTrue(g.is_advisory("unsafe report (cargo-geiger, informational)"))
        self.assertFalse(g.is_advisory("cargo-deny (advisories, bans, licenses, sources)"))
        self.assertFalse(g.is_advisory("build + test"))
        self.assertFalse(g.is_advisory("gate"))


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


if __name__ == "__main__":
    unittest.main(verbosity=2)
