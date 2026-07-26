#!/usr/bin/env python3
"""Tests for the crates.io publishing protections (issue #1135).

[OPUS-5] 🤖 SPARQ agent.

WHY THIS FILE EXISTS AS A SEPARATE SUITE
========================================
Two of the three protections live at seams that no other gate covers:

1. **The YAML seam.** `.github/workflows/release-plz.yml` runs only on `push: main`, so
   it NEVER registers on a PR's `gate` aggregator — a defect in it surfaces for the first
   time at release time, when a crates.io publish cannot be undone. This suite therefore
   parses that workflow **structurally** (`yaml.safe_load`, then indexing into
   `jobs.<id>.steps[i]`) rather than by substring. Measured in this repo: 18/18 Python
   mutants died while every uncaught mutant lived in a workflow `if:` / step / call-site.
   A `"--enforce" in text` assertion does not catch `if: false`; `steps[i]["if"]` does.

2. **The arming seam.** `scripts/release_pr_guard.py` is consulted by five scripts. This
   suite asserts each one actually calls it — deleting the call from any of them reds a
   named test here, in addition to that script's own self-test.

Every assertion below is written to DISCRIMINATE: for each guard there is a paired case
that exercises the same code path and expects the OPPOSITE outcome, so a fixture cannot
satisfy both the right and the wrong reading.

Run: python3 scripts/tests/test_release_publish_guard.py
"""

from __future__ import annotations

import datetime as dt
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

# PyYAML is REQUIRED, never optional: skipping on ImportError would make this whole
# suite silently vacuous, which is the exact failure mode it exists to prevent.
import yaml

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = REPO_ROOT / "scripts"
RELEASE_PLZ_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "release-plz.yml"
GUARD_SCRIPT_REL = "scripts/release-interval-guard.py"
RELEASE_JOB_ID = "release-plz-release"


def _load(name: str, filename: str):
    """Import a scripts/*.py module by path (the directory is not a package)."""
    spec = importlib.util.spec_from_file_location(name, SCRIPTS / filename)
    assert spec and spec.loader, filename
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


sys.path.insert(0, str(SCRIPTS))
release_pr_guard = _load("release_pr_guard", "release_pr_guard.py")
interval_guard = _load("release_interval_guard", "release-interval-guard.py")


def _workflow() -> dict:
    return yaml.safe_load(RELEASE_PLZ_WORKFLOW.read_text(encoding="utf-8"))


def _release_job_steps() -> list[dict]:
    jobs = _workflow()["jobs"]
    assert RELEASE_JOB_ID in jobs, (
        f"{RELEASE_PLZ_WORKFLOW.name} has no `{RELEASE_JOB_ID}` job — the guard's host "
        "job was renamed or removed"
    )
    return jobs[RELEASE_JOB_ID]["steps"]


def _guard_step_index(steps: list[dict]) -> int:
    matches = [
        i
        for i, step in enumerate(steps)
        if GUARD_SCRIPT_REL in str(step.get("run") or "")
    ]
    assert matches, (
        f"no step in `{RELEASE_JOB_ID}` runs {GUARD_SCRIPT_REL} — the #1135 "
        "publish-cadence guard step was DELETED from the release path"
    )
    assert len(matches) == 1, f"expected exactly one guard step, found {matches}"
    return matches[0]


def _release_plz_step_index(steps: list[dict]) -> int:
    matches = [
        i
        for i, step in enumerate(steps)
        if str(step.get("uses") or "").startswith("release-plz/action@")
    ]
    assert matches, (
        f"no `release-plz/action@…` step in `{RELEASE_JOB_ID}` — the release path this "
        "guard protects has moved; re-point the test"
    )
    return min(matches)


# ===================================================================== THE YAML SEAM
class TestReleaseWorkflowStructure(unittest.TestCase):
    """Structural (not textual) assertions over .github/workflows/release-plz.yml.

    Mutants each of these kills, verified by hand-editing the workflow and re-running:
      * delete the guard step               -> test_guard_step_exists
      * `if: false` on the guard step       -> test_guard_step_is_unconditional
      * `continue-on-error: true`           -> test_guard_step_is_not_continue_on_error
      * move it after release-plz/action    -> test_guard_runs_before_release_plz
      * `--dry-run` instead of `--enforce`  -> test_guard_step_enforces
    """

    def setUp(self) -> None:
        self.steps = _release_job_steps()

    def test_guard_step_exists(self) -> None:
        self.assertIsInstance(_guard_step_index(self.steps), int)

    def test_guard_step_is_unconditional(self) -> None:
        step = self.steps[_guard_step_index(self.steps)]
        # `if:` PARSED, not grepped: `if: false`, `if: ${{ false }}` and any other
        # condition would each silently skip the guard while leaving the step text intact.
        self.assertNotIn(
            "if",
            step,
            "the #1135 publish-cadence guard step carries an `if:` condition. It must be "
            "unconditional — a skipped guard is an absent guard, and the job it protects "
            "publishes irreversibly.",
        )

    def test_guard_step_is_not_continue_on_error(self) -> None:
        step = self.steps[_guard_step_index(self.steps)]
        self.assertNotEqual(
            step.get("continue-on-error"),
            True,
            "`continue-on-error: true` on the guard step turns its refusal into an "
            "advisory warning and the release proceeds anyway.",
        )

    def test_guard_runs_before_release_plz(self) -> None:
        guard = _guard_step_index(self.steps)
        release = _release_plz_step_index(self.steps)
        self.assertLess(
            guard,
            release,
            "the publish-cadence guard runs AFTER `release-plz/action` — by then the tag "
            "is cut and (with publish = true) the crates are already on crates.io.",
        )

    def test_guard_step_enforces(self) -> None:
        run = str(self.steps[_guard_step_index(self.steps)]["run"])
        self.assertIn(
            "--enforce",
            run,
            "the guard step must invoke --enforce; --dry-run always exits 0 and would "
            "make the step decorative.",
        )
        self.assertNotIn(
            "--dry-run",
            run,
            "the guard step invokes --dry-run, which never fails and therefore never "
            "blocks a release.",
        )

    def test_release_job_checks_out_full_history(self) -> None:
        # PRECONDITION for the guard: on a shallow checkout the tag list is not
        # authoritative, and the guard (correctly) refuses. Losing fetch-depth: 0 would
        # wedge every release rather than admit one — but it must still be pinned, or a
        # future 'fix' would be to relax the shallow check instead.
        checkout = next(
            step
            for step in self.steps
            if str(step.get("uses") or "").startswith("actions/checkout@")
        )
        self.assertEqual(
            checkout.get("with", {}).get("fetch-depth"),
            0,
            "the release job must check out with fetch-depth: 0 so the `v*` tag list is "
            "authoritative for the #1135 cadence guard.",
        )

    def test_release_plz_action_is_sha_pinned(self) -> None:
        uses = self.steps[_release_plz_step_index(self.steps)]["uses"]
        ref = uses.split("@", 1)[1].split(" ")[0]
        self.assertRegex(
            ref,
            r"^[0-9a-f]{40}$",
            f"release-plz/action must be SHA-pinned, got {ref!r}",
        )

    def test_workflow_never_runs_on_pull_request(self) -> None:
        # If this ever gains a pull_request trigger, a PR could reach the release path.
        triggers = _workflow()[True] if True in _workflow() else _workflow()["on"]
        self.assertNotIn("pull_request", triggers)
        self.assertNotIn("pull_request_target", triggers)


class TestArmWorkflowsSelfTestTheGuard(unittest.TestCase):
    """Both arm sweeps must run the guard's self-test before arming anything.

    Their scripts come from the DEFAULT branch at cron time, so a regression in the
    shared predicate would otherwise first be observed by arming the Release PR.
    """

    def _run_block(self, workflow_name: str, job_id: str) -> str:
        data = yaml.safe_load(
            (REPO_ROOT / ".github" / "workflows" / workflow_name).read_text("utf-8")
        )
        steps = data["jobs"][job_id]["steps"]
        return "\n".join(str(step.get("run") or "") for step in steps)

    def test_auto_arm_self_tests_the_release_guard(self) -> None:
        self.assertIn(
            "scripts/release_pr_guard.py --self-test",
            self._run_block("auto-arm.yml", "arm"),
        )

    def test_rearm_sweeper_self_tests_the_release_guard(self) -> None:
        self.assertIn(
            "scripts/release_pr_guard.py --self-test",
            self._run_block("rearm-sweeper.yml", "sweep"),
        )

    def test_docs_quality_gates_the_publishing_protections(self) -> None:
        # This suite must itself be invoked by a GATING job, or every assertion above is
        # dead code. docs-quality's job name contains neither "advisory" nor
        # "informational", so ci-summary discovers it as gating.
        data = yaml.safe_load(
            (REPO_ROOT / ".github" / "workflows" / "docs-quality.yml").read_text("utf-8")
        )
        runs = "\n".join(
            str(step.get("run") or "")
            for job in data["jobs"].values()
            for step in job.get("steps", [])
        )
        self.assertIn("scripts/tests/test_release_publish_guard.py", runs)
        self.assertIn("scripts/release-interval-guard.py --self-test", runs)


# ============================================================== THE RELEASE-PR EXCLUSION
class TestEveryArmingPathConsultsTheGuard(unittest.TestCase):
    """The enumerated arming paths each call release_pr_guard.

    Deleting the call from any one of them reds the named test here. The scripts' OWN
    self-tests also go red (proved by mutation in the PR body) — this is the second,
    cross-file net, so a path added later without the guard is visible.
    """

    PATHS = {
        "auto-arm.py": "arm_block_reason",
        "rearm-sweeper.py": "arm_block_reason",
        "check-pr-arm-base.py": "arm_block_reason",
        "batch-merge.py": "arm_block_reason",
        "pr-backlog.py": "arm_block_reason",
    }

    def test_each_arming_path_imports_and_calls_the_guard(self) -> None:
        # AST, not substring. MEASURED: the first draft of this test asserted
        # `"release_pr_guard.arm_block_reason" in text` and a mutant that reverted
        # scripts/batch-merge.py's predicate to the old author-AND-title conjunction
        # SURVIVED — the phrase still appeared, in the function's DOCSTRING. A prose
        # mention is not a call site.
        import ast

        for filename, symbol in sorted(self.PATHS.items()):
            with self.subTest(script=filename):
                tree = ast.parse((SCRIPTS / filename).read_text(encoding="utf-8"))
                calls = [
                    node
                    for node in ast.walk(tree)
                    if isinstance(node, ast.Call)
                    and isinstance(node.func, ast.Attribute)
                    and node.func.attr == symbol
                    and isinstance(node.func.value, ast.Name)
                    and node.func.value.id == "release_pr_guard"
                ]
                self.assertTrue(
                    calls,
                    f"scripts/{filename} contains no CALL to "
                    f"release_pr_guard.{symbol} — an arming/merging path lost its "
                    "#1135 Release-PR exclusion. (A comment, docstring or import "
                    "mentioning the guard does not exclude anything.)",
                )
    def test_batch_merge_and_pr_backlog_catch_a_branch_only_release_pr(self) -> None:
        """BEHAVIOURAL, not structural: the case the OLD conjunction missed.

        `author == github-actions AND title startswith "chore: release"` returns False for
        a Release PR whose title was edited or whose author identity changed, even though
        its head branch is unmistakable. Both scripts must now catch it on the branch
        alone. This is the assertion that killed the surviving mutant.
        """
        batch = _load("batch_merge_under_test", "batch-merge.py")
        backlog = _load("pr_backlog_under_test", "pr-backlog.py")
        branch_only = {
            "head_ref": "release-plz-main",
            "author_login": "app/some-other-bot",
            "title": "Bump workspace version to 0.2.0",
        }
        ordinary = {
            "head_ref": "sparq-agent/issue-3801-x",
            "author_login": "app/sparq-orchestrator",
            "title": "fix(engine): a change",
        }
        for name, module in (("batch-merge", batch), ("pr-backlog", backlog)):
            with self.subTest(script=name):
                self.assertTrue(
                    module.is_release_plz(branch_only),
                    f"{name}.is_release_plz missed a Release PR identifiable by its head "
                    "branch alone — the old author-AND-title conjunction is back.",
                )
                # The discriminating half: it must not swallow ordinary worker PRs.
                self.assertFalse(module.is_release_plz(ordinary), name)

    def test_the_predicate_cannot_be_influenced_by_labels(self) -> None:
        # The whole point of #1135's keying decision: a label can be added by anything
        # holding pull-requests: write, so a label-keyed exclusion is defeated by
        # relabelling. Structural, so it survives a rewrite of the body.
        import inspect

        params = inspect.signature(release_pr_guard.arm_block_reason).parameters
        self.assertNotIn("labels", params)
        self.assertEqual(
            sorted(params), ["author_login", "head_ref", "title"], sorted(params)
        )

    def test_release_pr_is_blocked_and_a_worker_pr_is_not(self) -> None:
        # The discriminating pair. If either half stops holding the guard is either
        # useless (blocks nothing) or a merge-train brick (blocks everything).
        self.assertIsNotNone(
            release_pr_guard.arm_block_reason(
                head_ref="release-plz-main",
                author_login="app/github-actions",
                title="chore: release v0.2.0",
            )
        )
        self.assertIsNone(
            release_pr_guard.arm_block_reason(
                head_ref="sparq-agent/issue-3801-x",
                author_login="app/sparq-orchestrator",
                title="fix(engine): a change",
            )
        )

    def test_indeterminate_head_branch_blocks(self) -> None:
        for head in (None, "", "   "):
            with self.subTest(head=head):
                self.assertIsNotNone(
                    release_pr_guard.arm_block_reason(
                        head_ref=head, author_login="jeswr", title="fix: x"
                    )
                )


class TestReleasePrGuardEndToEndThroughTheHook(unittest.TestCase):
    """Drive scripts/check-pr-arm-base.py as the harness does: hook JSON on stdin."""

    def _decide(self, command: str, gh_stdout: str | None) -> str | None:
        with tempfile.TemporaryDirectory(prefix="sparq-1135-hook-") as tmp:
            fake_bin = Path(tmp) / "bin"
            fake_bin.mkdir()
            gh = fake_bin / "gh"
            if gh_stdout is None:
                gh.write_text("#!/bin/sh\nexit 1\n", encoding="utf-8")
            else:
                gh.write_text(
                    "#!/bin/sh\nprintf '%s\\n' " + repr(gh_stdout).replace('"', '\\"'),
                    encoding="utf-8",
                )
                gh.write_text(
                    "#!/bin/sh\ncat <<'JSON'\n" + gh_stdout + "\nJSON\n",
                    encoding="utf-8",
                )
            gh.chmod(0o755)
            env = os.environ.copy()
            env["PATH"] = f"{fake_bin}{os.pathsep}{env.get('PATH', '')}"
            env["CLAUDE_PROJECT_DIR"] = str(REPO_ROOT)
            proc = subprocess.run(
                [sys.executable, str(SCRIPTS / "check-pr-arm-base.py")],
                input=json.dumps({"tool_input": {"command": command}}),
                capture_output=True,
                text=True,
                env=env,
                timeout=30,
            )
            self.assertEqual(proc.returncode, 0, proc.stderr)
            return json.loads(proc.stdout)["hookSpecificOutput"]["permissionDecision"]

    RELEASE_PR = json.dumps(
        {
            "baseRefName": "main",
            "headRefName": "release-plz-main",
            "number": 900,
            "author": {"login": "app/github-actions"},
            "title": "chore: release v0.2.0",
        }
    )
    WORKER_PR = json.dumps(
        {
            "baseRefName": "main",
            "headRefName": "sparq-agent/issue-3801-x",
            "number": 3801,
            "author": {"login": "app/sparq-orchestrator"},
            "title": "fix(engine): a change",
        }
    )

    def test_arming_the_release_pr_is_denied(self) -> None:
        self.assertEqual(
            self._decide("gh pr merge 900 --auto --squash", self.RELEASE_PR), "deny"
        )

    def test_arming_an_ordinary_worker_pr_is_allowed(self) -> None:
        # Same command shape, same base (`main`), same everything except the PR's
        # identity — so the deny above is attributable to the guard and nothing else.
        self.assertEqual(
            self._decide("gh pr merge 3801 --auto --squash", self.WORKER_PR), "allow"
        )

    def test_an_unresolvable_pr_is_denied_fail_closed(self) -> None:
        self.assertEqual(
            self._decide("gh pr merge 3801 --auto --squash", None), "deny"
        )


# ================================================================ THE CADENCE GUARD
class TestMinimumIntervalIsEnforced(unittest.TestCase):
    NOW = dt.datetime(2026, 7, 26, 12, 0, tzinfo=dt.timezone.utc)

    def _decide(self, hours_ago: float, *, version: str = "0.2.0"):
        return interval_guard.decide(
            now=self.NOW,
            workspace_version=version,
            tags=[("v0.1.0", self.NOW - dt.timedelta(hours=hours_ago))],
            crates_io_at=None,
        )

    def test_named_constant_is_24h(self) -> None:
        self.assertEqual(
            interval_guard.MIN_RELEASE_INTERVAL, dt.timedelta(hours=24)
        )

    def test_publish_inside_the_interval_is_refused(self) -> None:
        for hours in (0.0, 1.0, 12.0, 23.9):
            with self.subTest(hours=hours):
                self.assertFalse(self._decide(hours).allowed)

    def test_publish_outside_the_interval_is_permitted(self) -> None:
        # The discriminating half: if this failed too, "refused" above would prove
        # nothing (the guard could simply refuse everything).
        for hours in (24.0, 25.0, 240.0):
            with self.subTest(hours=hours):
                self.assertTrue(self._decide(hours).allowed)

    def test_crates_io_can_only_TIGHTEN_the_verdict(self) -> None:
        # A stale tag plus a recent crates.io publish must refuse: the guard takes the
        # MAX of the two sources, so neither can be used to argue for a shorter wait.
        verdict = interval_guard.decide(
            now=self.NOW,
            workspace_version="0.2.0",
            tags=[("v0.1.0", self.NOW - dt.timedelta(days=30))],
            crates_io_at=self.NOW - dt.timedelta(hours=1),
        )
        self.assertFalse(verdict.allowed)
        self.assertEqual(verdict.source, "crates.io")


class TestUnknownsRefuseRatherThanPublish(unittest.TestCase):
    NOW = dt.datetime(2026, 7, 26, 12, 0, tzinfo=dt.timezone.utc)

    def test_shallow_checkout_refuses(self) -> None:
        def shallow(_root, args):
            return "true\n" if args[:2] == ["rev-parse", "--is-shallow-repository"] else ""

        with self.assertRaises(interval_guard.GuardRefusal) as ctx:
            interval_guard.git_release_tags(REPO_ROOT, run_git=shallow)
        self.assertIn("shallow", str(ctx.exception))

    def test_unreadable_tag_list_refuses(self) -> None:
        def broken(_root, _args):
            raise interval_guard.GuardRefusal("git for-each-ref failed: boom")

        with self.assertRaises(interval_guard.GuardRefusal):
            interval_guard.git_release_tags(REPO_ROOT, run_git=broken)

    def test_unparseable_tag_date_refuses(self) -> None:
        def bad_date(_root, args):
            if args[:2] == ["rev-parse", "--is-shallow-repository"]:
                return "false\n"
            return "v0.1.0\tnot-a-date\n"

        with self.assertRaises(interval_guard.GuardRefusal):
            interval_guard.git_release_tags(REPO_ROOT, run_git=bad_date)

    def test_unreachable_crates_io_refuses(self) -> None:
        with self.assertRaises(interval_guard.GuardRefusal):
            interval_guard.crates_io_last_publish(
                ["sparq-core"], fetch=lambda _u: (None, "HTTP 503")
            )

    def test_a_definitive_404_is_NOT_an_unknown(self) -> None:
        # The discriminating counterpart: a successful "this crate does not exist" must
        # NOT be conflated with "I could not ask", or the first release could never ship.
        self.assertIsNone(
            interval_guard.crates_io_last_publish(
                ["sparq-core"], fetch=lambda _u: (None, None)
            )
        )

    def test_future_dated_last_release_refuses(self) -> None:
        verdict = interval_guard.decide(
            now=self.NOW,
            workspace_version="0.2.0",
            tags=[("v0.1.0", self.NOW + dt.timedelta(hours=1))],
            crates_io_at=None,
        )
        self.assertFalse(verdict.allowed)
        self.assertIn("FUTURE", verdict.reason)

    def test_publish_flag_is_read_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory(prefix="sparq-1135-flag-") as tmp:
            root = Path(tmp)
            for body, expected in (
                ("[workspace]\npublish = false\n", False),
                ("[workspace]\npublish = true\n", True),
                ("[workspace]\n", True),  # missing key -> strict
                ("", True),  # missing table -> strict
                ('[workspace]\npublish = "maybe"\n', True),  # non-boolean -> strict
            ):
                with self.subTest(body=body):
                    (root / "release-plz.toml").write_text(body, encoding="utf-8")
                    self.assertIs(
                        interval_guard.crates_io_publish_enabled(root), expected
                    )


def _make_test_repo(tmp: Path, *, tag_at: dt.datetime | None) -> Path:
    root = tmp / "repo"
    (root / "crates" / "a").mkdir(parents=True)
    (root / "crates" / "b").mkdir(parents=True)
    (root / "Cargo.toml").write_text(
        '[workspace]\nmembers = ["crates/a", "crates/b"]\n'
        '[workspace.package]\nversion = "0.2.0"\n',
        encoding="utf-8",
    )
    (root / "crates" / "a" / "Cargo.toml").write_text(
        '[package]\nname = "a"\nversion.workspace = true\n', encoding="utf-8"
    )
    (root / "crates" / "b" / "Cargo.toml").write_text(
        '[package]\nname = "b"\nversion.workspace = true\n'
        '[dependencies]\na = { path = "../a", version = "0.2.0" }\n',
        encoding="utf-8",
    )
    (root / "release-plz.toml").write_text(
        "[workspace]\npublish = true\n"
        '[[package]]\nname = "a"\nversion_group = "sparq"\n'
        '[[package]]\nname = "b"\nversion_group = "sparq"\n',
        encoding="utf-8",
    )
    env = {
        **os.environ,
        "GIT_AUTHOR_NAME": "t",
        "GIT_AUTHOR_EMAIL": "t@e",
        "GIT_COMMITTER_NAME": "t",
        "GIT_COMMITTER_EMAIL": "t@e",
    }
    git = lambda *a: subprocess.run(  # noqa: E731
        ["git", "-C", str(root), *a], check=True, capture_output=True, env=env
    )
    git("init", "-q", "-b", "main")
    git("add", "-A")
    git("commit", "-qm", "init")
    if tag_at is not None:
        env["GIT_COMMITTER_DATE"] = tag_at.isoformat()
        subprocess.run(
            ["git", "-C", str(root), "tag", "-a", "v0.1.0", "-m", "r"],
            check=True,
            capture_output=True,
            env=env,
        )
    return root



class TestGuardOnARealGitRepository(unittest.TestCase):
    """End-to-end over a REAL git repo + real manifests, with only crates.io injected.

    The unit tests above stub `run_git`. This one does not: it proves the shallow check,
    the tag parse, and the version-group check work against actual git output — the
    integration seam a stubbed test cannot reach.
    """

    def test_recent_real_tag_refuses_and_an_old_one_permits(self) -> None:
        now = dt.datetime.now(dt.timezone.utc)
        never_published = lambda _url: (None, None)  # noqa: E731
        with tempfile.TemporaryDirectory(prefix="sparq-1135-git-") as tmp:
            recent = _make_test_repo(Path(tmp), tag_at=now - dt.timedelta(hours=2))
            log: list[str] = []
            code = interval_guard.run(
                recent, dry_run=False, now=now, fetch=never_published, log=log.append
            )
            self.assertEqual(code, 1, "\n".join(log))
            self.assertTrue(any("REFUSED" in line for line in log), log)

        with tempfile.TemporaryDirectory(prefix="sparq-1135-git-ok-") as tmp:
            old = _make_test_repo(Path(tmp), tag_at=now - dt.timedelta(days=5))
            log = []
            code = interval_guard.run(
                old, dry_run=False, now=now, fetch=never_published, log=log.append
            )
            self.assertEqual(code, 0, "\n".join(log))
            self.assertTrue(any("ALLOW" in line for line in log), log)

    def test_version_group_drift_refuses_when_publish_is_enabled(self) -> None:
        now = dt.datetime.now(dt.timezone.utc)
        with tempfile.TemporaryDirectory(prefix="sparq-1135-drift-") as tmp:
            root = _make_test_repo(Path(tmp), tag_at=now - dt.timedelta(days=5))
            # Drop crate `b` from the version_group: it stays cargo-publishable.
            (root / "release-plz.toml").write_text(
                "[workspace]\npublish = true\n"
                '[[package]]\nname = "a"\nversion_group = "sparq"\n',
                encoding="utf-8",
            )
            log: list[str] = []
            code = interval_guard.run(
                root, dry_run=False, now=now, fetch=lambda _u: (None, None), log=log.append
            )
            self.assertEqual(code, 1, "\n".join(log))
            self.assertTrue(any("version_group" in line for line in log), log)

    def test_version_group_drift_only_WARNS_while_publish_is_false(self) -> None:
        now = dt.datetime.now(dt.timezone.utc)
        with tempfile.TemporaryDirectory(prefix="sparq-1135-drift-off-") as tmp:
            root = _make_test_repo(Path(tmp), tag_at=now - dt.timedelta(days=5))
            (root / "release-plz.toml").write_text(
                "[workspace]\npublish = false\n"
                '[[package]]\nname = "a"\nversion_group = "sparq"\n',
                encoding="utf-8",
            )
            log: list[str] = []
            code = interval_guard.run(
                root, dry_run=False, now=now, fetch=lambda _u: (None, None), log=log.append
            )
            self.assertEqual(code, 0, "\n".join(log))
            self.assertTrue(any("::warning" in line for line in log), log)


class TestDryRunIsInert(unittest.TestCase):
    """`--dry-run` reports and mutates NOTHING."""

    def _repo(self, tmp: Path) -> Path:
        return _make_test_repo(tmp, tag_at=dt.datetime.now(dt.timezone.utc))

    def test_dry_run_leaves_the_repository_byte_identical(self) -> None:
        with tempfile.TemporaryDirectory(prefix="sparq-1135-dry-") as tmp:
            root = self._repo(Path(tmp))

            def snapshot() -> tuple:
                show = lambda *a: subprocess.run(  # noqa: E731
                    ["git", "-C", str(root), *a], capture_output=True, text=True
                ).stdout
                files = sorted(
                    (p.relative_to(root).as_posix(), p.read_bytes())
                    for p in root.rglob("*")
                    if p.is_file() and ".git/" not in p.as_posix()
                )
                return (show("rev-parse", "HEAD"), show("tag", "-l"), show("status", "--porcelain"), files)

            before = snapshot()
            log: list[str] = []
            code = interval_guard.run(
                root,
                dry_run=True,
                now=dt.datetime.now(dt.timezone.utc),
                fetch=lambda _u: (None, None),
                log=log.append,
            )
            self.assertEqual(code, 0, "\n".join(log))
            self.assertEqual(before, snapshot(), "--dry-run mutated the repository")

    def test_dry_run_never_invokes_a_publishing_command(self) -> None:
        # Record every git invocation and assert none of them writes. `cargo publish` is
        # unreachable by construction (the module never spawns cargo), asserted below.
        seen: list[list[str]] = []

        def recording_git(root, args):
            seen.append(list(args))
            return interval_guard._run_git(root, args)

        with tempfile.TemporaryDirectory(prefix="sparq-1135-dry2-") as tmp:
            root = self._repo(Path(tmp))
            interval_guard.run(
                root,
                dry_run=True,
                now=dt.datetime.now(dt.timezone.utc),
                fetch=lambda _u: (None, None),
                git_runner=recording_git,
                log=lambda _m: None,
            )
        self.assertTrue(seen, "the dry run issued no git command at all")
        forbidden = {"tag", "push", "commit", "checkout", "reset", "clean", "update-ref"}
        for args in seen:
            self.assertNotIn(
                args[0], forbidden, f"--dry-run issued a mutating git command: {args}"
            )
    def test_the_guard_can_only_ever_spawn_git(self) -> None:
        # STRUCTURAL, not textual (the module's docstring legitimately says "cargo
        # publish"): every subprocess.run in the guard must have `git` as argv[0]. This is
        # what makes "--dry-run touches nothing" a property of the module rather than of
        # the one code path the test above happened to walk.
        import ast

        tree = ast.parse(
            (SCRIPTS / "release-interval-guard.py").read_text(encoding="utf-8")
        )
        spawns = [
            node
            for node in ast.walk(tree)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and node.func.attr in {"run", "Popen", "call", "check_output", "check_call"}
            and isinstance(node.func.value, ast.Name)
            and node.func.value.id == "subprocess"
        ]
        self.assertTrue(spawns, "no subprocess call found — re-point this test")
        for call in spawns:
            argv = call.args[0] if call.args else None
            self.assertIsInstance(
                argv, ast.List, f"line {call.lineno}: argv is not a literal list"
            )
            first = argv.elts[0]
            self.assertIsInstance(first, ast.Constant, f"line {call.lineno}")
            self.assertEqual(
                first.value,
                "git",
                f"line {call.lineno}: the cadence guard spawns {first.value!r}. It must "
                "only ever run `git` — spawning cargo (or anything else) would make it "
                "capable of the very publish it exists to prevent.",
            )

    def test_dry_run_reports_crates_versions_and_order(self) -> None:
        with tempfile.TemporaryDirectory(prefix="sparq-1135-dry3-") as tmp:
            root = self._repo(Path(tmp))
            log: list[str] = []
            interval_guard.run(
                root,
                dry_run=True,
                now=dt.datetime.now(dt.timezone.utc),
                fetch=lambda _u: (None, None),
                log=log.append,
            )
        text = "\n".join(log)
        self.assertIn("DRY-RUN", text)
        self.assertIn("publish order", text)
        # Dependency-first: `a` before `b`, because b depends on a.
        self.assertLess(text.index("1. a"), text.index("2. b"), text)
        self.assertIn("0.2.0", text)

    def test_dry_run_exits_zero_even_when_it_would_refuse(self) -> None:
        with tempfile.TemporaryDirectory(prefix="sparq-1135-dry4-") as tmp:
            root = self._repo(Path(tmp))  # tagged NOW -> inside the interval
            (root / "Cargo.toml").write_text(
                '[workspace]\nmembers = ["crates/a", "crates/b"]\n'
                '[workspace.package]\nversion = "0.3.0"\n',  # untagged -> would release
                encoding="utf-8",
            )
            log: list[str] = []
            code = interval_guard.run(
                root,
                dry_run=True,
                now=dt.datetime.now(dt.timezone.utc),
                fetch=lambda _u: (None, None),
                log=log.append,
            )
            self.assertEqual(code, 0)
            self.assertTrue(any("REFUSE" in line for line in log), log)


class TestLivePublishFlagIsStillFalse(unittest.TestCase):
    """A tripwire, not a policy: flipping `publish = true` is a MAINTAINER decision and
    must be a deliberate, reviewed diff — not something that rides in on an unrelated PR.
    When the maintainer flips it, this test is updated in the SAME commit."""

    def test_release_plz_toml_still_has_publish_false(self) -> None:
        self.assertFalse(
            interval_guard.crates_io_publish_enabled(REPO_ROOT),
            "release-plz.toml now enables crates.io publishing. If that is intended, the "
            "maintainer checklist in the #1135 PR body must be complete (version_group "
            "reconciled, Trusted Publishing configured) and this test updated in the "
            "same commit.",
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
