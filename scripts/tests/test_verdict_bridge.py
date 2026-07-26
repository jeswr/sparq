#!/usr/bin/env python3
"""Hermetic tests for the VERDICT-comment -> review-label bridge.

Two halves, because every uncaught mutant this project has measured lived at the YAML
seam rather than in the Python:

* ``TestDecisionCore`` / ``TestVerdictParsing`` — the pure predicate. Every guard in
  ``decide()`` has a NAMED test here that reds when the guard is deleted or inverted.
* ``TestWorkflowWiring`` — STRUCTURAL PyYAML inspection of
  ``.github/workflows/verdict-bridge.yml``. Substring and ``count(...) == N`` assertions
  do not catch ``if: false`` or a deleted step, so the workflow document is parsed and
  the job/step graph asserted against directly.
* ``TestCrossPolicyConsistency`` — imports the LIVE ``auto-arm.py`` /
  ``rearm-sweeper.py`` and pins the two invariants that make the bridge safe by
  construction: it never attests a PR the arming policy would refuse, and the
  informational label it writes is not a hold anywhere.

Stdlib unittest + PyYAML. No network, no gh, no git.
"""

# [OPUS-5] sparq-org/sparq — green-but-unqueued PR investigation, 2026-07-26.

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path
from types import ModuleType

import yaml

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = REPO_ROOT / "scripts"
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "verdict-bridge.yml"
AUTO_ARM_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "auto-arm.yml"


def load(path: Path, name: str) -> ModuleType:
    """Import a hyphenated script as a module."""
    sys.path.insert(0, str(SCRIPTS))
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader, path
    module = importlib.util.module_from_spec(spec)
    # Register BEFORE exec: dataclasses resolves annotations via sys.modules[__module__].
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


vb = load(SCRIPTS / "verdict-bridge.py", "verdict_bridge")
auto_arm = load(SCRIPTS / "auto-arm.py", "auto_arm_policy")
rearm = load(SCRIPTS / "rearm-sweeper.py", "rearm_sweeper_policy")

HEAD = "a" * 40
OTHER = "b" * 40

# The live sweep step's `run:` is a FOLDED (`>-`) scalar, so YAML joins its argv onto one
# line. Anchoring on `--repo` is what distinguishes it from the `--self-test` step.
LIVE_RUN_NEEDLE = "python3 scripts/verdict-bridge.py --repo"


def comment(body: str, **kw) -> dict:
    return vb.comment(body, **kw)


def pr(**kw):
    return vb.pr_fixture(**kw)


class TestVerdictParsing(unittest.TestCase):
    """The line-anchored + head-bound + provenance predicate."""

    def test_trailing_verdict_reads_the_last_non_blank_line(self):
        self.assertEqual(vb.trailing_verdict("prose\n\nVERDICT: pass\n\n"), "pass")
        self.assertEqual(vb.trailing_verdict("**VERDICT: fail**"), "fail")

    def test_verdict_must_be_the_final_line(self):
        """A quoted INSTRUCTION mentioning the phrase is not a verdict."""
        self.assertIsNone(
            vb.trailing_verdict("end with `VERDICT: pass`\n\nstill reviewing")
        )
        self.assertIsNone(vb.trailing_verdict("VERDICT: pass\nbut actually no"))

    def test_verdict_line_must_not_carry_trailing_content(self):
        self.assertIsNone(vb.trailing_verdict("VERDICT: pass (with caveats)"))

    def test_binds_head_requires_the_full_forty_hex_sha(self):
        self.assertTrue(vb.binds_head(f"reviewed {HEAD}", HEAD))
        self.assertFalse(vb.binds_head(f"reviewed {HEAD[:12]}", HEAD))
        self.assertFalse(vb.binds_head(f"reviewed {OTHER}", HEAD))

    def test_binds_head_rejects_a_sha_glued_into_a_longer_hex_run(self):
        self.assertFalse(vb.binds_head(f"reviewed {HEAD}cafe", HEAD))
        self.assertFalse(vb.binds_head(f"reviewed cafe{HEAD}", HEAD))

    def test_untrusted_association_is_not_a_reviewer(self):
        for association in ("CONTRIBUTOR", "FIRST_TIME_CONTRIBUTOR", "NONE", "", "BOT"):
            with self.subTest(association=association):
                self.assertIsNone(
                    vb.head_bound_verdict(
                        [comment(f"{HEAD}\n\nVERDICT: pass", association=association)],
                        HEAD,
                    )
                )

    def test_malformed_comment_payloads_never_read_as_a_pass(self):
        self.assertIsNone(
            vb.head_bound_verdict([None, 7, {}, {"body": None}, {"body": ""}], HEAD)
        )

    def test_latest_by_created_at_wins_regardless_of_input_order(self):
        early = comment(f"{HEAD}\n\nVERDICT: pass", created_at="2026-01-01T00:00:00Z", cid=1)
        late = comment(f"{HEAD}\n\nVERDICT: fail", created_at="2026-01-02T00:00:00Z", cid=2)
        for ordering in ([early, late], [late, early]):
            with self.subTest(ordering=[c["id"] for c in ordering]):
                self.assertEqual(vb.head_bound_verdict(ordering, HEAD).value, "fail")

    def test_recency_is_created_at_not_comment_id(self):
        """Comment ids are not monotone across a PR's timeline (reviews, transfers,
        cross-posted bodies), so ordering by id can silently resurrect a stale pass.
        Here the RETRACTION carries the LOWER id but the LATER created_at."""
        stale_pass = comment(
            f"{HEAD}\n\nVERDICT: pass", created_at="2026-01-01T00:00:00Z", cid=900
        )
        retraction = comment(
            f"{HEAD}\n\nVERDICT: fail", created_at="2026-01-02T00:00:00Z", cid=100
        )
        self.assertEqual(
            vb.head_bound_verdict([stale_pass, retraction], HEAD).value, "fail"
        )
        self.assertEqual(
            decision(pr(labels={vb.REVIEW_ATTESTATION}), [stale_pass, retraction]),
            "retract",
        )

    def test_editing_an_older_pass_cannot_reorder_it_ahead_of_a_newer_fail(self):
        """Ordering uses immutable created_at — an edit must not defeat a retraction."""
        early = dict(
            comment(f"{HEAD}\n\nVERDICT: pass", created_at="2026-01-01T00:00:00Z", cid=1),
            updated_at="2099-01-01T00:00:00Z",
        )
        late = comment(f"{HEAD}\n\nVERDICT: fail", created_at="2026-01-02T00:00:00Z", cid=2)
        self.assertEqual(vb.head_bound_verdict([early, late], HEAD).value, "fail")


class TestDecisionCore(unittest.TestCase):
    """One named test per guard in ``decide()``."""

    def setUp(self):
        self.passing = comment(f"reviewed {HEAD}\n\nVERDICT: pass")
        self.failing = comment(f"reviewed {HEAD}\n\nVERDICT: fail", cid=2)

    def test_head_bound_pass_on_a_clean_pr_promotes(self):
        self.assertEqual(decision(pr(), [self.passing]), "promote")

    def test_pass_bound_to_a_superseded_head_never_promotes(self):
        stale = comment(f"reviewed {OTHER}\n\nVERDICT: pass")
        self.assertNotEqual(decision(pr(), [stale]), "promote")

    def test_hold_labels_are_fail_closed(self):
        for hold in sorted(vb.HOLD_LABELS) + ["needs:design", "needs:whatever"]:
            with self.subTest(hold=hold):
                self.assertEqual(decision(pr(labels={hold}), [self.passing]), "none")

    def test_draft_is_never_attested(self):
        self.assertEqual(decision(pr(is_draft=True), [self.passing]), "none")

    def test_closed_pr_is_never_attested(self):
        for state in ("MERGED", "CLOSED", ""):
            with self.subTest(state=state):
                self.assertEqual(decision(pr(state=state), [self.passing]), "none")

    def test_promotion_is_idempotent(self):
        self.assertEqual(
            decision(pr(labels={vb.REVIEW_ATTESTATION}), [self.passing]), "none"
        )

    def test_head_bound_fail_retracts_an_existing_attestation(self):
        self.assertEqual(
            decision(pr(labels={vb.REVIEW_ATTESTATION}), [self.failing]), "retract"
        )

    def test_absence_of_evidence_never_retracts(self):
        """A hand-applied review:pass with no comment behind it must survive."""
        self.assertEqual(decision(pr(labels={vb.REVIEW_ATTESTATION}), []), "none")
        stale_fail = comment(f"reviewed {OTHER}\n\nVERDICT: fail")
        self.assertEqual(
            decision(pr(labels={vb.REVIEW_ATTESTATION}), [stale_fail]), "none"
        )

    def test_green_mergeable_unreviewed_pr_is_flagged_visible(self):
        self.assertEqual(decision(pr(), []), "flag")

    def test_flag_requires_green_and_mergeable_and_non_draft(self):
        for kwargs in (
            {"is_draft": True},
            {"mergeable": "CONFLICTING"},
            {"mergeable": "UNKNOWN"},
            {"gate_conclusion": "failure"},
            {"gate_conclusion": None},
        ):
            with self.subTest(**kwargs):
                self.assertEqual(decision(pr(**kwargs), []), "none")

    def test_flag_skips_a_pr_already_in_a_review_lane(self):
        for lane in ("review:needs", "review:changes", "review:needs-user", "review:pass"):
            with self.subTest(lane=lane):
                self.assertEqual(decision(pr(labels={lane}), []), "none")

    def test_flag_is_cleared_once_any_verdict_binds(self):
        flagged = pr(labels={vb.UNREVIEWED_LABEL})
        self.assertEqual(decision(flagged, [self.passing]), "unflag")
        self.assertEqual(decision(flagged, [self.failing]), "unflag")
        self.assertEqual(decision(flagged, []), "none")


def decision(pull, comments) -> str:
    return vb.decide(pull, comments).action


class TestCrossPolicyConsistency(unittest.TestCase):
    """The bridge must be safe against the LIVE arming policies, not a copy of them."""

    def test_bridge_holds_cover_every_auto_arm_exclusion(self):
        """Never attest a PR auto-arm would refuse — the label would be a lie."""
        required = auto_arm.HUMAN_OR_TRUST_LABELS | auto_arm.REVIEW_CHANGES_LABELS
        missing = required - vb.HOLD_LABELS
        self.assertEqual(missing, set(), f"auto-arm excludes these, the bridge does not: {missing}")

    def test_bridge_holds_cover_every_rearm_sweeper_exclusion(self):
        missing = rearm.EXCLUDED_LABELS - vb.HOLD_LABELS
        self.assertEqual(missing, set(), f"rearm-sweeper excludes these: {missing}")

    def test_the_informational_label_is_not_a_hold_in_any_policy(self):
        """review:unreviewed must never block an arm, or flagging would deadlock it."""
        label = vb.UNREVIEWED_LABEL
        self.assertNotIn(label, auto_arm.HUMAN_OR_TRUST_LABELS)
        self.assertNotIn(label, auto_arm.REVIEW_CHANGES_LABELS)
        self.assertNotIn(label, rearm.EXCLUDED_LABELS)
        self.assertEqual(rearm.exclusion_labels(frozenset({label})), [])
        self.assertEqual(vb.hold_labels({label}), [])

    def test_the_attestation_label_matches_the_arming_predicate(self):
        self.assertEqual(vb.REVIEW_ATTESTATION, auto_arm.REVIEW_LABEL)
        self.assertEqual(vb.REVIEW_ATTESTATION, rearm.REVIEW_ATTESTATION)


# --------------------------------------------------------------------------- YAML seam


def load_workflow(path: Path) -> dict:
    doc = yaml.safe_load(path.read_text(encoding="utf-8"))
    assert isinstance(doc, dict), path
    return doc


def triggers(doc: dict) -> dict:
    # PyYAML resolves the bare key `on` to the boolean True (YAML 1.1).
    return doc.get("on", doc.get(True)) or {}


def cron_minutes(doc: dict) -> set[int]:
    out: set[int] = set()
    for entry in triggers(doc).get("schedule") or []:
        minute_field = str(entry["cron"]).split()[0]
        out.update(int(m) for m in minute_field.split(","))
    return out


class TestWorkflowWiring(unittest.TestCase):
    """STRUCTURAL inspection: `if: false`, a deleted step, or a wrong input must red."""

    @classmethod
    def setUpClass(cls):
        cls.doc = load_workflow(WORKFLOW)
        cls.job = cls.doc["jobs"]["bridge"]
        cls.steps = cls.job["steps"]

    def step_by_run_needle(self, needle: str) -> dict:
        matches = [
            s for s in self.steps if needle in str(s.get("run") or "")
        ]
        self.assertEqual(
            len(matches), 1, f"expected exactly one step whose run contains {needle!r}"
        )
        return matches[0]

    def test_the_job_is_not_disabled_by_a_job_level_condition(self):
        """An `if: false` on the job silently turns the whole bridge off."""
        self.assertNotIn("if", self.job, "the bridge job must be unconditional")
        self.assertNotIn("continue-on-error", self.job)

    def test_no_step_is_disabled_or_swallowed(self):
        """Every step except the fail-soft App-token mint runs unconditionally."""
        for step in self.steps:
            name = step.get("name") or step.get("uses") or "<step>"
            with self.subTest(step=name):
                self.assertNotEqual(
                    str(step.get("if", "")).strip().lower(),
                    "false",
                    "a literal `if: false` disables this step",
                )
                self.assertNotIn("continue-on-error", step, name)
        conditional = [s for s in self.steps if "if" in s]
        self.assertEqual(
            [s.get("id") for s in conditional],
            ["app-token"],
            "only the App-token mint may be conditional",
        )

    def test_the_self_test_step_exists_and_runs_the_policy_self_test(self):
        step = self.step_by_run_needle("scripts/verdict-bridge.py --self-test")
        self.assertNotIn("if", step)
        self.assertIn("scripts/gh_retry.py --self-test", step["run"])

    def test_the_live_run_step_invokes_the_bridge_with_a_repo(self):
        step = self.step_by_run_needle(LIVE_RUN_NEEDLE)
        self.assertIn("--repo", step["run"])
        self.assertIn("--default-branch", step["run"])
        self.assertEqual(step["env"]["REPO"], "${{ github.repository }}")

    def test_the_informational_label_is_reified_before_use(self):
        """Without `gh label create`, every flag decision fails its label edit."""
        step = self.step_by_run_needle("gh label create")
        self.assertIn(vb.UNREVIEWED_LABEL, step["run"])
        self.assertIn("--force", step["run"], "creation must be idempotent")
        order = [self.steps.index(step), self.steps.index(
            self.step_by_run_needle(LIVE_RUN_NEEDLE)
        )]
        self.assertLess(order[0], order[1], "reify the label before the sweep uses it")

    def test_the_checkout_materializes_every_script_the_run_imports(self):
        checkouts = [s for s in self.steps if "actions/checkout" in str(s.get("uses"))]
        self.assertEqual(len(checkouts), 1)
        sparse = checkouts[0]["with"]["sparse-checkout"]
        for needed in ("scripts/verdict-bridge.py", "scripts/gh_retry.py"):
            self.assertIn(needed, sparse, f"{needed} would be missing at runtime")
        self.assertEqual(
            checkouts[0]["with"]["ref"],
            "${{ github.event.repository.default_branch }}",
            "the policy must come from the trusted default branch, never a PR head",
        )

    def test_it_never_triggers_on_a_pull_request_head(self):
        on = triggers(self.doc)
        for forbidden in ("pull_request", "pull_request_target", "push", "merge_group", "workflow_run"):
            self.assertNotIn(forbidden, on, f"{forbidden} would run this on candidate code")
        self.assertIn("schedule", on)
        self.assertIn("workflow_dispatch", on)

    def test_it_has_no_contents_write_authority(self):
        perms = self.doc["permissions"]
        self.assertEqual(perms.get("contents"), "read")
        self.assertEqual(perms.get("pull-requests"), "write")
        self.assertEqual(perms.get("issues"), "write")

    def test_the_cron_fires_before_each_auto_arm_sweep(self):
        """A wrong cron input makes every promotion wait a full extra arming cycle."""
        bridge_minutes = sorted(cron_minutes(self.doc))
        arm_minutes = sorted(cron_minutes(load_workflow(AUTO_ARM_WORKFLOW)))
        self.assertTrue(bridge_minutes, "the bridge must be scheduled at all")
        self.assertTrue(arm_minutes, "auto-arm's cron is the backstop this pairs with")
        self.assertEqual(
            set(bridge_minutes) & set(arm_minutes), set(), "must not collide with auto-arm"
        )
        for minute in bridge_minutes:
            following = [a for a in arm_minutes if a > minute]
            self.assertTrue(following, f"no auto-arm sweep follows minute {minute}")
            self.assertLessEqual(
                following[0] - minute, 10, f"minute {minute} waits too long to be armed"
            )

    def test_every_action_is_sha_pinned(self):
        for step in self.steps:
            uses = step.get("uses")
            if not uses:
                continue
            with self.subTest(uses=uses):
                ref = uses.split("@", 1)[1].split(" ")[0]
                self.assertEqual(len(ref), 40, f"{uses} is not SHA-pinned")
                int(ref, 16)

    def test_concurrency_serialises_the_sweep(self):
        concurrency = self.doc["concurrency"]
        self.assertEqual(concurrency["group"], "verdict-bridge")
        self.assertFalse(concurrency["cancel-in-progress"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
