#!/usr/bin/env python3
# [OPUS-5] Tests for the review-lane blind-spot alarm (scripts/review_lane_alarm.py +
# .github/workflows/review-alarm.yml). 🤖 SPARQ agent.
#
# TWO HALVES, and the second is the load-bearing one.
#
# 1. BEHAVIOUR — the four obligations a verdict detector must meet, each as a NAMED test
#    that goes red on deletion OR inversion of the guard it covers:
#      * a PR with no verdict never reads as reviewed (so it can never be armed),
#      * a verdict bound to a SUPERSEDED head does not count,
#      * an author cannot verdict their own PR,
#      * the review brief's own quoted instruction text does not register as a pass.
#
# 2. THE YAML SEAM — measured in this repo, every uncaught mutant of an 18-mutant run
#    lived in a workflow `if:` / step / call-site, not in the Python. YAML `if:`/`on:`/
#    `permissions:` cannot be unit-tested by execution, so this suite pins their SHAPE:
#      * the workflow EXISTS and is scheduled (delete the trigger => red),
#      * the self-test step runs BEFORE the live step (reorder => red),
#      * the live step actually invokes the script (break the call site => red),
#      * the job holds `issues: write` and NOT `pull-requests: write` / `contents: write`
#        (an arming-authority escalation => red),
#      * docs-quality.yml wires THIS test file (drop the call site => red).
#
# The last one is the anti-vacuity anchor: without it, this whole file could be deleted
# from CI's reachable set and nothing would notice.
#
# Stdlib + PyYAML (already a docs-quality dependency). Run:
#   python3 scripts/tests/test_review_lane_alarm.py

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SCRIPT = REPO_ROOT / "scripts" / "review_lane_alarm.py"
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "review-alarm.yml"
DOCS_QUALITY = REPO_ROOT / ".github" / "workflows" / "docs-quality.yml"

_spec = importlib.util.spec_from_file_location("review_lane_alarm", SCRIPT)
alarm = importlib.util.module_from_spec(_spec)
assert _spec.loader is not None
_spec.loader.exec_module(alarm)

SHA_A = "a" * 40
SHA_B = "b" * 40
REPO = "sparq-org/sparq"

# Verbatim from the standing review brief. Substring-grepping for `VERDICT: pass` has
# already scored this quotation as a real pass in this repository.
BRIEF_INSTRUCTION = (
    "Post ONE comment per PR, opening with the `> 🤖 SPARQ agent` blockquote, ending "
    "with a line-anchored final line, exactly `VERDICT: pass` or `VERDICT: fail` "
    "(nothing after it).\nState the head SHA you reviewed.\n"
)


def pr(number=1, *, ref="research/x", login="jeswr", sha=SHA_A, draft=False, labels=(),
       created="2026-07-01T00:00:00Z", repo=REPO):
    return {
        "number": number,
        "draft": draft,
        "created_at": created,
        "title": f"pr {number}",
        "user": {"login": login},
        "labels": [{"name": name} for name in labels],
        "head": {"ref": ref, "sha": sha, "repo": {"full_name": repo}},
    }


def comment(body, login="reviewer-bot", created="2026-07-02T00:00:00Z"):
    return {"body": body, "user": {"login": login}, "created_at": created}


class TestNoVerdictIsNotArmable(unittest.TestCase):
    """OBLIGATION 1: a PR with no verdict must never read as reviewed."""

    def test_no_comments_at_all_is_a_blind_spot(self):
        self.assertEqual(alarm.classify_pr(pr(), [], REPO), "blind-spot")

    def test_chatter_without_a_verdict_line_is_a_blind_spot(self):
        chatter = [comment(f"looks good to me, head is {SHA_A}"), comment("bumping")]
        self.assertEqual(alarm.classify_pr(pr(), chatter, REPO), "blind-spot")

    def test_no_verdict_never_yields_a_polarity(self):
        self.assertIsNone(alarm.countable_verdict(pr(), [])[0])

    def test_an_unreviewed_pr_past_the_threshold_alarms(self):
        now = alarm._parse_iso("2026-07-26T00:00:00Z")
        findings, _ = alarm.find_blind_spot(
            [pr(number=42, created="2026-07-01T00:00:00Z")], {}, REPO, now, 24)
        self.assertEqual([f["number"] for f in findings], [42])


class TestSupersededHeadDoesNotCount(unittest.TestCase):
    """OBLIGATION 2: a force-push must invalidate a verdict."""

    def test_verdict_naming_the_previous_head_does_not_count(self):
        stale = [comment(f"reviewed {SHA_B}\n\nVERDICT: pass")]
        self.assertIsNone(alarm.countable_verdict(pr(sha=SHA_A), stale)[0])

    def test_superseded_verdict_still_leaves_the_pr_in_the_blind_spot(self):
        stale = [comment(f"reviewed {SHA_B}\n\nVERDICT: pass")]
        self.assertEqual(alarm.classify_pr(pr(sha=SHA_A), stale, REPO), "blind-spot")

    def test_the_superseded_reason_is_reported_not_dropped(self):
        stale = [comment(f"reviewed {SHA_B}\n\nVERDICT: pass")]
        _, why = alarm.countable_verdict(pr(sha=SHA_A), stale)
        self.assertTrue(any("superseded" in reason for reason in why), why)

    def test_a_truncated_sha_does_not_bind(self):
        self.assertFalse(alarm.comment_binds_head(f"reviewed {SHA_A[:12]}", SHA_A))

    def test_a_longer_hex_run_does_not_bind(self):
        # A 41-hex token must not satisfy a 40-hex binding by prefix.
        self.assertFalse(alarm.comment_binds_head(SHA_A + "c", SHA_A))

    def test_the_same_verdict_counts_once_the_comment_names_the_new_head(self):
        fresh = [comment(f"reviewed {SHA_A}\n\nVERDICT: pass")]
        self.assertEqual(alarm.countable_verdict(pr(sha=SHA_A), fresh)[0], "pass")


class TestAuthorCannotVerdictOwnPr(unittest.TestCase):
    """OBLIGATION 3: author XOR reviewer, structurally.

    This is not hypothetical here. On 2026-07-26 the live repository carried three open
    PRs (#3765, #3435, #2278) with a line-anchored, HEAD-BOUND `VERDICT:` comment posted
    by the PR's own author at author_association MEMBER — the exact shape the arming
    bridge admits. They were all `fail`, so nothing self-armed; a `pass` in the same shape
    would have."""

    def test_self_posted_pass_does_not_count(self):
        selfrev = [comment(f"reviewed {SHA_A}\n\nVERDICT: pass", login="jeswr")]
        self.assertIsNone(alarm.countable_verdict(pr(login="jeswr"), selfrev)[0])

    def test_self_posted_pass_leaves_the_pr_in_the_blind_spot(self):
        selfrev = [comment(f"reviewed {SHA_A}\n\nVERDICT: pass", login="jeswr")]
        self.assertEqual(alarm.classify_pr(pr(login="jeswr"), selfrev, REPO), "blind-spot")

    def test_the_self_review_is_named_in_the_report(self):
        selfrev = [comment(f"reviewed {SHA_A}\n\nVERDICT: pass", login="jeswr")]
        _, why = alarm.countable_verdict(pr(login="jeswr"), selfrev)
        self.assertTrue(any("self-review" in reason for reason in why), why)

    def test_self_review_is_reported_as_self_not_merely_stale(self):
        # Checked BEFORE the head test on purpose: a self-review's problem is not its age,
        # and a human told "stale" would re-review the head and re-arm the same hole.
        selfrev = [comment(f"reviewed {SHA_B}\n\nVERDICT: pass", login="jeswr")]
        _, why = alarm.countable_verdict(pr(login="jeswr", sha=SHA_A), selfrev)
        self.assertTrue(any("self-review" in reason for reason in why), why)

    def test_a_different_account_on_the_same_head_does_count(self):
        # The guard must not be so broad that no verdict can ever pass.
        other = [comment(f"reviewed {SHA_A}\n\nVERDICT: pass", login="someone-else")]
        self.assertEqual(alarm.countable_verdict(pr(login="jeswr"), other)[0], "pass")

    def test_a_self_pass_cannot_override_a_cross_author_fail(self):
        comments = [
            comment(f"{SHA_A}\n\nVERDICT: fail", login="r1", created="2026-07-02T00:00:00Z"),
            comment(f"{SHA_A}\n\nVERDICT: pass", login="jeswr", created="2026-07-03T00:00:00Z"),
        ]
        self.assertEqual(alarm.countable_verdict(pr(login="jeswr"), comments)[0], "fail")


class TestInstructionTextIsNotAPass(unittest.TestCase):
    """OBLIGATION 4: never substring-grep. The brief quotes the literal at every
    reviewer, and a substring match on that quotation has produced a false pass here."""

    def test_the_brief_instruction_is_not_a_verdict(self):
        self.assertIsNone(alarm.verdict_polarity(BRIEF_INSTRUCTION))

    def test_a_comment_quoting_the_brief_stays_in_the_blind_spot(self):
        quoted = [comment(f"{SHA_A}\n{BRIEF_INSTRUCTION}")]
        self.assertEqual(alarm.classify_pr(pr(), quoted, REPO), "blind-spot")

    def test_backticked_literal_is_not_a_verdict(self):
        self.assertIsNone(alarm.verdict_polarity("`VERDICT: pass`"))

    def test_verdict_line_followed_by_prose_is_not_a_verdict(self):
        self.assertIsNone(
            alarm.verdict_polarity("VERDICT: pass\n\nActually, one more thought:")
        )

    def test_suffixed_verdict_line_is_not_a_verdict(self):
        self.assertIsNone(alarm.verdict_polarity("VERDICT: pass (modulo the nit above)"))

    def test_wrong_case_is_not_a_verdict(self):
        self.assertIsNone(alarm.verdict_polarity("VERDICT: PASS"))

    def test_a_genuine_trailing_verdict_line_does_parse(self):
        self.assertEqual(alarm.verdict_polarity(f"{SHA_A}\n\nVERDICT: pass\n\n"), "pass")


class TestRegistryLaneReachability(unittest.TestCase):
    """The reachability predicate is the claim this whole alarm rests on: it must be the
    registry's own admission test, pinned to REAL branch names measured on 2026-07-26."""

    def test_real_worker_branch_is_reachable(self):
        self.assertTrue(alarm.registry_lane_reachable(
            pr(ref="sparq-agent/issue-2908-30221671021-1",
               login="sparq-orchestrator[bot]"), REPO))

    def test_real_local_agent_branches_are_unreachable(self):
        for ref in ("ci/pr-freshness-auto-update-branch", "research/agent-context-sharing",
                    "fix/readiness-visibility-opus5", "feat/start-here-renderer"):
            with self.subTest(ref=ref):
                self.assertFalse(alarm.registry_lane_reachable(pr(ref=ref, login="jeswr"), REPO))

    def test_non_bot_author_on_a_worker_branch_is_unreachable(self):
        self.assertFalse(
            alarm.registry_lane_reachable(pr(ref="sparq-agent/issue-1-2-1", login="jeswr"), REPO))

    def test_fork_head_is_unreachable(self):
        self.assertFalse(alarm.registry_lane_reachable(
            pr(ref="sparq-agent/issue-1-2-1", login="sparq-orchestrator[bot]",
               repo="attacker/sparq"), REPO))

    def test_the_regex_is_byte_identical_to_the_registry_gate(self):
        # If the registry ever loosens its head-ref gate, this pin must be updated in the
        # same breath — a paraphrased predicate would silently mis-scope the alarm.
        self.assertEqual(alarm.REGISTRY_HEAD_REF_RE.pattern,
                         r"^sparq-agent/issue-([1-9][0-9]*)-")


class TestNonAlarmingStates(unittest.TestCase):
    """The alarm must not be so eager that a human hold or a real review re-fires it."""

    def test_draft_is_not_alarmed(self):
        self.assertEqual(alarm.classify_pr(pr(draft=True), [], REPO), "draft")

    def test_human_holds_are_terminal(self):
        for label in ("needs:user", "review:needs-user"):
            with self.subTest(label=label):
                self.assertEqual(alarm.classify_pr(pr(labels=(label,)), [], REPO), "human-held")

    def test_lane_labelled_prs_are_not_double_reported(self):
        self.assertEqual(alarm.classify_pr(pr(labels=("review:pass",)), [], REPO),
                         "lane-labelled")

    def test_a_genuinely_reviewed_pr_raises_no_finding(self):
        now = alarm._parse_iso("2026-07-26T00:00:00Z")
        good = [comment(f"reviewed {SHA_A}\n\nVERDICT: pass")]
        findings, _ = alarm.find_blind_spot(
            [pr(number=7, created="2026-07-01T00:00:00Z")], {7: good}, REPO, now, 24)
        self.assertEqual(findings, [])

    def test_a_fresh_blind_spot_pr_is_counted_but_not_alarmed(self):
        now = alarm._parse_iso("2026-07-26T00:00:00Z")
        findings, census = alarm.find_blind_spot(
            [pr(number=9, created="2026-07-25T23:00:00Z")], {}, REPO, now, 24)
        self.assertEqual(findings, [])
        self.assertEqual(census.get("blind-spot-fresh"), 1)

    def test_the_census_counts_every_state_exit(self):
        now = alarm._parse_iso("2026-07-26T00:00:00Z")
        _, census = alarm.find_blind_spot(
            [
                pr(number=1, created="2026-07-01T00:00:00Z"),
                pr(number=2, draft=True),
                pr(number=3, labels=("needs:user",)),
                pr(number=4, ref="sparq-agent/issue-5-1-1", login="sparq-orchestrator[bot]"),
            ],
            {}, REPO, now, 24)
        self.assertEqual(
            census,
            {"blind-spot": 1, "draft": 1, "human-held": 1, "registry-lane": 1},
        )


class TestFailLoud(unittest.TestCase):
    """A broken detector must never look like a quiet green."""

    def test_malformed_pull_entry_raises(self):
        with self.assertRaises(alarm.AlarmError):
            alarm.registry_lane_reachable("not-a-dict", REPO)

    def test_invalid_pr_number_raises(self):
        now = alarm._parse_iso("2026-07-26T00:00:00Z")
        with self.assertRaises(alarm.AlarmError):
            alarm.find_blind_spot([{"number": 0}], {}, REPO, now, 24)

    def test_unparseable_timestamp_raises(self):
        now = alarm._parse_iso("2026-07-26T00:00:00Z")
        with self.assertRaises(alarm.AlarmError):
            alarm.find_blind_spot([pr(created="whenever")], {}, REPO, now, 24)

    def test_bad_repo_slug_raises(self):
        import argparse
        args = argparse.Namespace(repo="not-a-slug", now="", prs_file="", max_age_hours=24,
                                  dry_run=True, self_test=False)
        with self.assertRaises(alarm.AlarmError):
            alarm.run(args)

    def test_main_maps_infrastructure_failure_to_exit_two(self):
        self.assertEqual(alarm.main(["--repo", "not-a-slug", "--dry-run"]), 2)

    def test_the_embedded_self_test_passes(self):
        self.assertEqual(alarm.main(["--self-test"]), 0)


# --------------------------------------------------------------------------- #
# THE YAML SEAM
# --------------------------------------------------------------------------- #
def _load(path: Path) -> dict:
    return yaml.safe_load(path.read_text(encoding="utf-8"))


def _on_block(wf: dict) -> dict:
    """Bare `on` is the YAML boolean True, so PyYAML keys it under `True`."""
    return wf.get("on", wf.get(True, {})) or {}


class TestWorkflowWiring(unittest.TestCase):
    """Mutate the workflow and one of these must go red."""

    @classmethod
    def setUpClass(cls):
        cls.wf = _load(WORKFLOW)
        cls.job = cls.wf["jobs"]["alarm"]
        cls.steps = cls.job["steps"]

    def test_the_workflow_exists(self):
        self.assertTrue(WORKFLOW.is_file(), WORKFLOW)

    def test_it_is_scheduled(self):
        # Delete the trigger => the detector never runs => red here.
        schedule = _on_block(self.wf).get("schedule")
        self.assertTrue(schedule, "review-alarm must keep a schedule trigger")
        self.assertTrue(all("cron" in entry for entry in schedule))

    def test_the_cron_is_explicit_not_shorthand(self):
        for entry in _on_block(self.wf)["schedule"]:
            self.assertNotIn("*/", entry["cron"], "explicit minutes keep the cadence auditable")

    def test_it_is_manually_dispatchable(self):
        self.assertIn("workflow_dispatch", _on_block(self.wf))

    def test_it_never_triggers_on_a_pr_or_merge_group(self):
        # A check-run on a PR head would drag this non-gate into the ci-summary poll set.
        for trigger in ("pull_request", "pull_request_target", "merge_group", "push"):
            self.assertNotIn(trigger, _on_block(self.wf), trigger)

    def test_the_job_has_no_if_guard(self):
        # Invert-the-`if:` is the classic vacuity mutant; the job simply has none to invert.
        self.assertNotIn("if", self.job)

    def test_the_script_is_actually_invoked(self):
        # Break the call site (rename/typo the script path) => red here.
        runs = " ".join(str(step.get("run", "")) for step in self.steps)
        self.assertIn("scripts/review_lane_alarm.py", runs)

    def test_the_self_test_runs_before_the_live_step(self):
        commands = [str(step.get("run", "")) for step in self.steps]
        selftest = next(i for i, c in enumerate(commands) if "--self-test" in c)
        live = next(
            i for i, c in enumerate(commands)
            if "review_lane_alarm.py" in c and "--self-test" not in c
        )
        self.assertLess(selftest, live, "a detector proves itself before it is trusted")

    def test_the_self_test_step_is_unconditional(self):
        for step in self.steps:
            if "--self-test" in str(step.get("run", "")):
                self.assertNotIn("if", step, "the self-test must never be skippable")

    def test_the_live_step_is_unconditional(self):
        for step in self.steps:
            run = str(step.get("run", ""))
            if "review_lane_alarm.py" in run and "--self-test" not in run:
                self.assertNotIn("if", step)

    def test_no_continue_on_error_anywhere(self):
        # Exit-zero swallowing: a later transient must never discard the earned red.
        self.assertNotIn("continue-on-error", self.job)
        for step in self.steps:
            self.assertNotIn("continue-on-error", step)

    def test_the_job_holds_issues_write_only(self):
        perms = self.job["permissions"]
        self.assertEqual(perms.get("issues"), "write")
        self.assertEqual(perms.get("contents"), "read")

    def test_the_job_has_NO_arming_authority(self):
        # THE escalation guard. Granting pull-requests:write would let this detector
        # write review:pass and therefore arm — the one thing it must never be able to do.
        perms = self.job["permissions"]
        self.assertNotIn("pull-requests", perms)
        self.assertNotEqual(perms.get("contents"), "write")
        for forbidden in ("actions", "checks", "id-token"):
            self.assertNotIn(forbidden, perms, forbidden)

    def test_the_workflow_level_permissions_are_read_only(self):
        self.assertEqual(self.wf["permissions"], {"contents": "read"})

    def test_it_does_not_act_on_the_transport_workflow(self):
        # verdict-bridge.yml owns the comment -> label TRANSPORT hop; this workflow owns
        # noticing that no verdict was ever PRODUCED. They must not both write the same
        # label or drive each other. Prose may name it (the header does); no executable
        # step may touch it, and no step may write a PR label at all.
        for step in self.steps:
            run = str(step.get("run", ""))
            self.assertNotIn("verdict-bridge", run)
            self.assertNotIn("workflow run", run)
            for label_write in ("gh pr edit", "gh issue edit", "--add-label", "/labels"):
                self.assertNotIn(label_write, run, label_write)

    def test_actions_are_sha_pinned(self):
        import re
        for step in self.steps:
            uses = step.get("uses")
            if uses:
                self.assertRegex(uses, r"@[0-9a-f]{40}$", uses)

    def test_the_job_name_carries_no_advisory_token(self):
        import re
        self.assertIsNone(re.search(r"\b(advisory|informational)\b", self.job["name"]))

    def test_the_checkout_does_not_persist_credentials(self):
        for step in self.steps:
            if str(step.get("uses", "")).startswith("actions/checkout@"):
                self.assertIs(step["with"]["persist-credentials"], False)

    def test_the_sparse_checkout_includes_the_script(self):
        # A sparse-checkout that misses the script would fail the run at HEAD, but silently
        # drift if the script is ever renamed; pin the pairing.
        for step in self.steps:
            if str(step.get("uses", "")).startswith("actions/checkout@"):
                self.assertIn("scripts/review_lane_alarm.py", step["with"]["sparse-checkout"])


class TestGateCallSite(unittest.TestCase):
    """The anti-vacuity anchor: without this, the whole suite could leave CI unnoticed."""

    def test_docs_quality_runs_this_suite(self):
        text = DOCS_QUALITY.read_text(encoding="utf-8")
        self.assertIn("scripts/tests/test_review_lane_alarm.py", text)

    def test_the_call_site_step_is_unconditional(self):
        wf = _load(DOCS_QUALITY)
        found = False
        for job in wf["jobs"].values():
            for step in job.get("steps") or []:
                if "test_review_lane_alarm.py" in str(step.get("run", "")):
                    found = True
                    self.assertNotIn("if", step, "the suite must not be conditionally skipped")
                    self.assertNotIn("continue-on-error", step)
        self.assertTrue(found, "docs-quality.yml must invoke test_review_lane_alarm.py")


if __name__ == "__main__":
    unittest.main(verbosity=2, argv=[sys.argv[0], "-v"])
