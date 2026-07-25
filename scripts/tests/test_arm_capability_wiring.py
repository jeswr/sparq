#!/usr/bin/env python3
# [OPUS-5] sparq-org/sparq#3760 — cross-file INSPECTION pins for the arm path.
#
# THE OUTAGE. The re-arm sweeper could not enable auto-merge at all:
#   run 30033315483 (2026-07-23T18:21Z) failed arming PR #3454 (OPEN, non-draft, CLEAN,
#   review:pass) with "GraphQL: Resource not accessible by integration
#   (enablePullRequestAutoMerge)", and had been doing so every ten minutes.
# THE CAUSE. rearm-sweeper.yml declared `permissions: contents: read`. Enabling auto-merge
# is a repository WRITE operation. auto-arm.yml armed two PRs 90 minutes earlier
# (run 30027010495, 16:52Z) on the same repository with the same plain GITHUB_TOKEN — its
# only difference was `contents: write`. `allow_auto_merge` was already true and the `main`
# ruleset carried no integration restriction, so nothing maintainer-only was involved.
#
# A `permissions:` block cannot be unit-tested by execution, so this suite pins the SHAPE
# of everything the fix depends on:
#   * PERMISSION PIN — both arm workflows must declare `contents: write` AND
#     `pull-requests: write`. A silent downgrade to `read` reproduces #3760 exactly, and
#     it is invisible in a diff review of a python-only change.
#   * PROBE WIRED, AND FIRST — each workflow must run `--probe-arm-capability` as its own
#     step BEFORE the arming step, so a token that cannot arm reds the job once instead of
#     failing per-PR forever.
#   * SAME TOKEN — the probe step's GH_TOKEN expression must be byte-identical to the arm
#     step's. A probe that attests a different token than the sweep uses is worse than no
#     probe: it would report can-arm while the sweep is denied.
#   * REMEDIATION CONTENT — the single ::error must name every flippable thing
#     (`contents: write`, the repository "Allow auto-merge" setting, and the
#     ORCHESTRATOR_APP_ID / ORCHESTRATOR_APP_PRIVATE_KEY secrets). An ::error that says
#     only "permission denied" is what made this cost days.
#   * NO-DRIFT — the two scripts must stay self-contained (each workflow sparse-checks out
#     only its own file, so they CANNOT import a shared helper) while keeping the same
#     denial-marker set. This pins the duplication instead of letting it rot.
#   * EXIT SEMANTICS — neither script may lose its non-zero exit on an arm failure.
#   * STICKY PRECEDENCE (#3766) — collecting failures is only half the job. Both scripts
#     must compute the exit from the FINAL accumulated state with the precedence
#     `collected-failure > transient-exhaustion > clean`, so a LATER candidate's exhausted
#     transient can never downgrade an EARLIER candidate's real arm failure to the lenient
#     ::warning + exit 0. Pinned in both (deliberately duplicated) files.
#
# Needs PyYAML (already a docs-quality dependency); everything else is stdlib. Run:
#   python3 scripts/tests/test_arm_capability_wiring.py

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path
from types import ModuleType

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
REARM_YML = WORKFLOWS / "rearm-sweeper.yml"
AUTO_ARM_YML = WORKFLOWS / "auto-arm.yml"
REARM_PY = REPO_ROOT / "scripts" / "rearm-sweeper.py"
AUTO_ARM_PY = REPO_ROOT / "scripts" / "auto-arm.py"

PROBE_FLAG = "--probe-arm-capability"
# The exact string GitHub returned in run 30033315483 — the regression anchor.
LIVE_DENIAL = (
    "GraphQL: Resource not accessible by integration (enablePullRequestAutoMerge)"
)
# Every remediation lever a human can actually pull, by exact name.
REQUIRED_REMEDIATION_TOKENS = (
    "contents: write",
    "pull-requests: write",
    "Allow auto-merge",
    "autoMergeAllowed",
    "ORCHESTRATOR_APP_ID",
    "ORCHESTRATOR_APP_PRIVATE_KEY",
)

ARM_WORKFLOWS = {
    "rearm-sweeper.yml": (REARM_YML, "scripts/rearm-sweeper.py"),
    "auto-arm.yml": (AUTO_ARM_YML, "scripts/auto-arm.py"),
}


def load(path: Path) -> dict:
    return yaml.safe_load(path.read_text(encoding="utf-8"))


def load_module(path: Path, name: str) -> ModuleType:
    """Import a hyphenated script by path (the scripts are not importable by name)."""
    # Both scripts import the sibling gh_retry helper (#3759), which is only importable
    # with scripts/ on the path — the workflows get that for free by running from there.
    scripts_dir = str(REPO_ROOT / "scripts")
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None, path
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def steps_of(document: dict) -> list[dict]:
    steps: list[dict] = []
    for job in (document.get("jobs") or {}).values():
        steps.extend(job.get("steps") or [])
    return steps


def run_of(step: dict) -> str:
    return str(step.get("run") or "")


class TestPermissionPin(unittest.TestCase):
    """#3760: `contents: read` on an arming job IS the bug. Pin write on both."""

    def test_both_arm_workflows_declare_contents_and_pull_requests_write(self) -> None:
        for name, (path, _script) in ARM_WORKFLOWS.items():
            document = load(path)
            permissions = document.get("permissions")
            self.assertIsInstance(
                permissions,
                dict,
                f"{name} must declare an explicit permissions block",
            )
            self.assertEqual(
                permissions.get("contents"),
                "write",
                f"{name}: enablePullRequestAutoMerge is a repository WRITE operation; "
                f"`contents: read` reproduces #3760 ({LIVE_DENIAL})",
            )
            self.assertEqual(
                permissions.get("pull-requests"),
                "write",
                f"{name} must keep `pull-requests: write`",
            )

    def test_arm_workflows_do_not_narrow_permissions_at_the_job_level(self) -> None:
        # A job-level `permissions:` REPLACES the workflow-level block, so a job-level
        # narrowing would silently reintroduce the outage.
        for name, (path, _script) in ARM_WORKFLOWS.items():
            for job_id, job in (load(path).get("jobs") or {}).items():
                if "permissions" not in job:
                    continue
                permissions = job["permissions"]
                self.assertIsInstance(permissions, dict, f"{name}:{job_id}")
                self.assertEqual(
                    permissions.get("contents"), "write", f"{name}:{job_id}"
                )
                self.assertEqual(
                    permissions.get("pull-requests"), "write", f"{name}:{job_id}"
                )


class TestProbeWiring(unittest.TestCase):
    """The probe must exist, run first, and attest the token the arm step uses."""

    def test_probe_step_exists_and_precedes_the_arm_step(self) -> None:
        for name, (path, script) in ARM_WORKFLOWS.items():
            steps = steps_of(load(path))
            probe_indexes = [
                index
                for index, step in enumerate(steps)
                if PROBE_FLAG in run_of(step) and script in run_of(step)
            ]
            arm_indexes = [
                index
                for index, step in enumerate(steps)
                if script in run_of(step)
                and PROBE_FLAG not in run_of(step)
                and "--self-test" not in run_of(step)
            ]
            self.assertEqual(
                len(probe_indexes), 1, f"{name} must run {PROBE_FLAG} exactly once"
            )
            self.assertTrue(arm_indexes, f"{name} must still have an arming step")
            self.assertLess(
                probe_indexes[0],
                min(arm_indexes),
                f"{name}: the capability probe must run BEFORE any arm",
            )

    def test_probe_and_arm_steps_use_the_identical_token_expression(self) -> None:
        for name, (path, script) in ARM_WORKFLOWS.items():
            steps = steps_of(load(path))
            probe = next(
                step
                for step in steps
                if PROBE_FLAG in run_of(step) and script in run_of(step)
            )
            arm = next(
                step
                for step in steps
                if script in run_of(step)
                and PROBE_FLAG not in run_of(step)
                and "--self-test" not in run_of(step)
            )
            probe_token = (probe.get("env") or {}).get("GH_TOKEN")
            arm_token = (arm.get("env") or {}).get("GH_TOKEN")
            self.assertTrue(probe_token, f"{name}: probe step needs GH_TOKEN")
            self.assertEqual(
                probe_token,
                arm_token,
                f"{name}: a probe on a DIFFERENT token than the arm step would attest "
                "capability the sweep does not have",
            )

    def test_self_test_step_survives(self) -> None:
        for name, (path, script) in ARM_WORKFLOWS.items():
            steps = steps_of(load(path))
            self.assertTrue(
                any(
                    "--self-test" in run_of(step) and script in run_of(step)
                    for step in steps
                ),
                f"{name} must keep running the policy self-test before arming",
            )


class TestScriptContract(unittest.TestCase):
    """Pin the behaviour the workflows depend on, in both (deliberately duplicated) files."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.rearm = load_module(REARM_PY, "rearm_sweeper_under_test")
        cls.auto = load_module(AUTO_ARM_PY, "auto_arm_under_test")

    def test_both_expose_the_probe_and_denial_classifier(self) -> None:
        for module in (self.rearm, self.auto):
            self.assertTrue(callable(module.is_arm_denial))
            self.assertTrue(callable(module.probe_arm_capability_exit))
            self.assertEqual(module.CAN_ARM, "can-arm")
            self.assertEqual(module.CANNOT_ARM, "cannot-arm")
            self.assertEqual(module.INCONCLUSIVE, "inconclusive")

    def test_the_live_denial_text_classifies_as_a_capability_denial(self) -> None:
        for module in (self.rearm, self.auto):
            self.assertTrue(
                module.is_arm_denial(f"gh pr merge 3454 failed: {LIVE_DENIAL}"),
                "the exact run-30033315483 error must be recognised",
            )

    def test_per_pr_conditions_are_not_misclassified_as_capability_denials(self) -> None:
        # A race/CAS/transient must stay per-PR, or one flaky PR would stop the sweep.
        for module in (self.rearm, self.auto):
            for benign in (
                "Head branch was modified. Review and try again",
                "HTTP 502: 502 Bad Gateway",
                "Pull request is in unstable status",
                "simulated expectedHeadOid CAS rejection",
            ):
                self.assertFalse(module.is_arm_denial(benign), (module.PROGRAM, benign))

    def test_denial_marker_sets_do_not_drift_between_the_two_scripts(self) -> None:
        # The workflows sparse-check out only their own script, so a shared import is
        # impossible; pin the duplication instead of letting the copies diverge.
        self.assertEqual(
            tuple(self.rearm.ARM_DENIAL_MARKERS),
            tuple(self.auto.ARM_DENIAL_MARKERS),
            "both arm scripts must classify the same denial set",
        )

    def test_remediation_names_every_flippable_lever(self) -> None:
        for module in (self.rearm, self.auto):
            for token in REQUIRED_REMEDIATION_TOKENS:
                self.assertIn(
                    token,
                    module.ARM_REMEDIATION,
                    f"{module.PROGRAM}: the single ::error must name {token!r}",
                )

    def test_exit_semantics_never_return_zero_on_an_arm_failure(self) -> None:
        self.assertEqual(self.rearm.SweepOutcome().exit_code, 0)
        self.assertEqual(
            self.rearm.SweepOutcome(arm_failures=[(1, "denied")]).exit_code, 1
        )
        self.assertEqual(
            self.rearm.SweepOutcome(capability=self.rearm.CANNOT_ARM).exit_code, 1
        )
        self.assertEqual(self.auto.ArmOutcome().exit_code, 0)
        self.assertEqual(self.auto.ArmOutcome(arm_failures=[(1, "denied")]).exit_code, 1)
        self.assertEqual(
            self.auto.ArmOutcome(capability=self.auto.CANNOT_ARM).exit_code, 1
        )

    @staticmethod
    def _outcome_class(module: ModuleType):
        return getattr(module, "SweepOutcome", None) or module.ArmOutcome

    def test_both_outcomes_carry_sticky_transient_state(self) -> None:
        # #3766: the per-candidate transient must be RECORDED state, not an exception that
        # unwinds the run — an exception is what discarded the earlier collected failure.
        for module in (self.rearm, self.auto):
            outcome = self._outcome_class(module)()
            for attribute in (
                "transient_exhaustions",
                "sweep_transient",
                "hard_failed",
                "transient_detail",
            ):
                self.assertTrue(
                    hasattr(outcome, attribute),
                    f"{module.PROGRAM}: {attribute} is load-bearing for #3766",
                )

    def test_a_collected_failure_outranks_a_later_transient_exhaustion(self) -> None:
        for module in (self.rearm, self.auto):
            outcome_class = self._outcome_class(module)
            dominated = outcome_class(
                arm_failures=[(451, "unstable")],
                transient_exhaustions=[(452, "HTTP 504")],
            )
            self.assertTrue(dominated.hard_failed, module.PROGRAM)
            self.assertEqual(
                dominated.exit_code,
                1,
                f"{module.PROGRAM}: a LATER candidate's exhausted transient must never "
                "downgrade a COLLECTED arm failure to success (#3766)",
            )
            # ... while the lenient policy survives where it IS correct.
            transient_only = outcome_class(transient_exhaustions=[(452, "HTTP 504")])
            self.assertFalse(transient_only.hard_failed, module.PROGRAM)
            self.assertEqual(transient_only.exit_code, 0, module.PROGRAM)
            self.assertIsNotNone(transient_only.transient_detail, module.PROGRAM)

    def test_both_publish_the_outcome_on_the_runner(self) -> None:
        # The exit is computed at the END from this state, so it must be reachable after an
        # exception escapes run() — a local variable is exactly what #3766 lost.
        rearm_runner = self.rearm.RearmSweeper(
            "o/r", "main", gh=lambda _argv: "", log=lambda _line: None
        )
        auto_runner = self.auto.AutoArmer("o/r", "main", lambda _argv: "", lambda _l: None)
        for runner, module in ((rearm_runner, self.rearm), (auto_runner, self.auto)):
            self.assertIsInstance(
                runner.outcome, self._outcome_class(module), module.PROGRAM
            )

    def test_both_expose_an_end_of_run_precedence_function(self) -> None:
        self.assertTrue(callable(self.rearm.sweep_exit))
        self.assertTrue(callable(self.auto.arm_exit_code))
        for module in (self.rearm, self.auto):
            source = (REARM_PY if module is self.rearm else AUTO_ARM_PY).read_text(
                encoding="utf-8"
            )
            self.assertIn(
                "collected-failure > transient-exhaustion > clean",
                source,
                f"{module.PROGRAM}: the precedence rule must be documented in-source",
            )

    def test_capability_probe_query_reads_the_repository_setting(self) -> None:
        for module in (self.rearm, self.auto):
            self.assertIn("autoMergeAllowed", module.CAPABILITY_QUERY)

    def test_a_null_viewer_permission_never_blocks(self) -> None:
        # MEASURED: Repository.viewerPermission is null when authenticated as a GitHub App,
        # and the Actions GITHUB_TOKEN *is* an App installation token — so it can only ever
        # be diagnostics. PullRequest.viewerCanEnableAutoMerge is likewise unusable: it is
        # false for already-armed (#2521), queued (#3764), draft and merged PRs even under
        # an ADMIN user token. Gating on either would refuse legitimate arms. This pins the
        # only thing that matters behaviourally: a null viewerPermission with the setting
        # ON must still read as can-arm.
        for module in (self.rearm, self.auto):
            response = {
                "data": {
                    "repository": {
                        "autoMergeAllowed": True,
                        "viewerPermission": None,
                    },
                    "viewer": {"login": "github-actions[bot]"},
                }
            }
            verdict = self._probe_with(module, response)
            self.assertEqual(verdict.status, module.CAN_ARM, verdict)
            # ... and the setting being OFF must be decisive regardless.
            response["data"]["repository"]["autoMergeAllowed"] = False
            verdict = self._probe_with(module, response)
            self.assertEqual(verdict.status, module.CANNOT_ARM, verdict)
            self.assertIn("Allow auto-merge", verdict.detail)

    @staticmethod
    def _probe_with(module: ModuleType, response: dict):
        import json

        def fake_gh(_argv: list[str]) -> str:
            return json.dumps(response)

        runner = (
            module.RearmSweeper("o/r", "main", gh=fake_gh, log=lambda _line: None)
            if hasattr(module, "RearmSweeper")
            else module.AutoArmer("o/r", "main", fake_gh, lambda _line: None)
        )
        return runner.probe_arm_capability()

    def test_both_scripts_stay_import_free_of_each_other(self) -> None:
        for path in (REARM_PY, AUTO_ARM_PY):
            source = path.read_text(encoding="utf-8")
            self.assertNotIn("import rearm_sweeper", source, path)
            self.assertNotIn("import auto_arm", source, path)


class TestSelfTestsRunInCi(unittest.TestCase):
    """The self-tests are only worth writing if something runs them on every PR."""

    def test_docs_quality_runs_both_self_tests_and_this_wiring_suite(self) -> None:
        document = load(WORKFLOWS / "docs-quality.yml")
        gating = [
            run_of(step)
            for job_id, job in (document.get("jobs") or {}).items()
            for step in (job.get("steps") or [])
            if "advisory" not in str(job.get("name", job_id)).lower()
        ]
        blob = "\n".join(gating)
        for needle in (
            "scripts/rearm-sweeper.py --self-test",
            "scripts/auto-arm.py --self-test",
            "scripts/tests/test_arm_capability_wiring.py",
        ):
            self.assertIn(
                needle,
                blob,
                f"docs-quality.yml must run `{needle}` in a GATING job",
            )


if __name__ == "__main__":
    unittest.main(verbosity=2)
