#!/usr/bin/env python3
# [OPUS-5] Hermetic tests for scripts/check-package-test-execution.py — the guard
# that a `packages/*` npm package's `test` script is actually invoked by a CI leg
# (sparq-org/sparq#5843, follow-up to #5742's gate G1).
#
# Two halves, both of which must stay non-vacuous:
#   1. DETECTOR tests drive the pure functions against fixture workflow text and
#      fixture manifests — no filesystem, no git, no network, stdlib only. The
#      negative fixtures are the point: a package with a `test` script that no
#      step invokes MUST be reported, or the guard is decorative.
#   2. LIVE tests run the detector over the REAL packages/ + .github/workflows/
#      and assert not merely that the verdict is PASS but that each real package
#      was positively ATTRIBUTED to a leg. A parser regression that silently
#      stopped finding steps would still produce "no violations"; asserting the
#      attribution is what makes the pass meaningful.
#
# Run:  python3 scripts/tests/test_check_package_test_execution.py
# (stdlib only; no pytest required — also discoverable by `pytest`.)

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent


def _load(name: str, filename: str):
    spec = importlib.util.spec_from_file_location(name, REPO_ROOT / "scripts" / filename)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


chk = _load("check_package_test_execution", "check-package-test-execution.py")


WORKFLOW = """\
name: js
on:
  pull_request:
jobs:
  js:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: js
    steps:
      - uses: actions/checkout@abc # v7.0.0
      - name: Install
        run: npm ci
      - name: Test (node --test)
        run: npm test
      - name: Conformance harness
        working-directory: packages/rdfjs-conformance
        run: |
          npm run typecheck
          npm test
      - run: npm test
        working-directory: packages/solid-server
  other:
    runs-on: ubuntu-latest
    steps:
      - name: From the root
        run: npm test -w packages/eyereasoner-compat
"""


class TestReadWorkflowSteps(unittest.TestCase):
    def test_job_default_step_override_and_block_scalars(self):
        steps = chk.read_workflow_steps(WORKFLOW)
        self.assertEqual(
            [(wd, run.strip().splitlines()[0]) for wd, run in steps],
            [
                ("js", "npm ci"),
                ("js", "npm test"),
                ("packages/rdfjs-conformance", "npm run typecheck"),
                ("packages/solid-server", "npm test"),
                (".", "npm test -w packages/eyereasoner-compat"),
            ],
        )

    def test_working_directory_after_run_is_still_the_same_step(self):
        # `- run:` first, `working-directory:` second (the js.yml solid-server
        # shape inverted) — a forward scan must find it, and must NOT leak into
        # the next sequence item.
        steps = chk.read_workflow_steps(WORKFLOW)
        self.assertEqual(steps[3][0], "packages/solid-server")
        self.assertEqual(steps[4][0], ".")

    def test_defaults_run_mapping_is_not_read_as_a_command(self):
        # `defaults: {run: {working-directory: js}}` contains a `run:` key that is
        # a MAPPING. Reading it as a shell block would inject a bogus step.
        self.assertEqual(len(chk.read_workflow_steps(WORKFLOW)), 5)

    def test_workflow_level_defaults_apply_to_a_job_without_its_own(self):
        text = (
            "defaults:\n  run:\n    working-directory: packages/x\n"
            "jobs:\n  a:\n    steps:\n      - run: npm test\n"
        )
        self.assertEqual(chk.read_workflow_steps(text), [("packages/x", "npm test")])

    def test_no_jobs_key_yields_no_steps(self):
        self.assertEqual(chk.read_workflow_steps("name: x\non: push\n"), [])


class TestNpmTestTargets(unittest.TestCase):
    def _t(self, command: str, cwd: str = "."):
        return chk.npm_test_targets(chk._tokenise(command), cwd)

    def test_bare_and_run_forms_target_the_working_directory(self):
        self.assertEqual(self._t("npm test", "packages/x"), ["packages/x"])
        self.assertEqual(self._t("npm run test", "packages/x"), ["packages/x"])
        self.assertEqual(self._t("npm run --silent test", "packages/x"), ["packages/x"])

    def test_a_different_script_is_not_the_test_script(self):
        for command in (
            "npm run test:unit",
            "npm run testify",
            "npm run typecheck",
            "npm run build",
            "npm ci",
            "npm install",
        ):
            with self.subTest(command=command):
                self.assertEqual(self._t(command, "packages/x"), [])

    def test_workspace_selectors(self):
        self.assertEqual(self._t("npm test -w packages/x"), ["packages/x"])
        self.assertEqual(self._t("npm run test --workspace packages/x"), ["packages/x"])
        self.assertEqual(self._t("npm test --workspace=@scope/name"), ["@scope/name"])
        self.assertEqual(self._t("npm test --workspaces"), [chk.ALL_WORKSPACES])

    def test_workspace_value_is_not_mistaken_for_the_script_name(self):
        # `npm run -w foo test` — `foo` is the flag's VALUE, `test` the script.
        self.assertEqual(self._t("npm run -w packages/x test"), ["packages/x"])
        # ...and `npm run -w packages/x build` still runs `build`, not `test`.
        self.assertEqual(self._t("npm run -w packages/x build"), [])

    def test_a_non_npm_command_is_never_coverage(self):
        self.assertEqual(self._t("node --test", "packages/x"), [])
        self.assertEqual(self._t("yarn test", "packages/x"), [])


class TestRunBlockTargets(unittest.TestCase):
    def test_cd_moves_the_working_directory(self):
        self.assertEqual(
            chk.run_block_targets("cd packages/x && npm test", "."), {"packages/x"}
        )
        self.assertEqual(
            chk.run_block_targets("cd packages/x\nnpm test\n", "."), {"packages/x"}
        )

    def test_cd_elsewhere_does_not_credit_the_package(self):
        # `cd` resolves against the step's working directory, as a shell would,
        # so the credit goes to the subdirectory — never to packages/x itself.
        self.assertEqual(
            chk.run_block_targets("cd site\nnpm test\n", "packages/x"),
            {"packages/x/site"},
        )

    def test_untokenisable_command_yields_no_coverage(self):
        self.assertEqual(chk.run_block_targets('npm test "unclosed', "packages/x"), set())


class TestEvaluate(unittest.TestCase):
    GLOBS = ["packages/*", "js", "site"]
    PKG = {"name": "@scope/x", "scripts": {"test": "node --test"}}

    def test_uncovered_test_script_is_a_violation(self):
        violations = chk.evaluate({"packages/x": self.PKG}, self.GLOBS, set())
        self.assertEqual([p for p, _ in violations], ["packages/x"])
        self.assertIn("NO .github/workflows", violations[0][1][0])

    def test_covered_by_directory_or_by_package_name_passes(self):
        for covered in ({"packages/x"}, {"@scope/x"}, {chk.ALL_WORKSPACES}):
            with self.subTest(covered=covered):
                self.assertEqual(
                    chk.evaluate({"packages/x": self.PKG}, self.GLOBS, covered), []
                )

    def test_coverage_of_a_sibling_package_does_not_count(self):
        violations = chk.evaluate({"packages/x": self.PKG}, self.GLOBS, {"packages/y"})
        self.assertEqual([p for p, _ in violations], ["packages/x"])

    def test_package_without_a_test_script_needs_no_leg(self):
        self.assertEqual(
            chk.evaluate({"packages/x": {"name": "@scope/x"}}, self.GLOBS, set()), []
        )

    def test_non_workspace_member_is_a_violation(self):
        violations = chk.evaluate({"packages/x": self.PKG}, ["js"], {"packages/x"})
        self.assertEqual(len(violations), 1)
        self.assertIn("`workspaces`", violations[0][1][0])

    def test_allowlist_exempts_a_package(self):
        self.assertEqual(
            chk.evaluate(
                {"packages/x": self.PKG},
                self.GLOBS,
                set(),
                allowlist={"x": "deliberately not CI-tested because ..."},
            ),
            [],
        )

    def test_allowlist_entry_without_a_written_reason_exempts_nothing(self):
        # The reason IS the reviewable artifact. A vacuous value must not disable
        # the invariant for the package — otherwise the repo-wide gate is
        # defeated by an entry that says nothing.
        for reason in ("", "   ", "\n\t", None, False, True, 0, 1, [], {}, ["why"]):
            with self.subTest(reason=reason):
                violations = chk.evaluate(
                    {"packages/x": self.PKG}, self.GLOBS, set(), allowlist={"x": reason}
                )
                self.assertEqual([p for p, _ in violations], ["packages/x"])

    def test_workspaces_object_spelling_is_read(self):
        self.assertTrue(
            chk.is_workspace_member(
                "packages/x", chk.workspace_globs({"workspaces": {"packages": ["packages/*"]}})
            )
        )


class TestLoadAllowlist(unittest.TestCase):
    """The escape hatch is only an escape hatch if the written reason is real."""

    def _write(self, text: str) -> Path:
        import tempfile

        d = tempfile.mkdtemp()
        self.addCleanup(__import__("shutil").rmtree, d, True)
        p = Path(d) / "allowlist.json"
        p.write_text(text, encoding="utf-8")
        return p

    def _load(self, text: str) -> dict:
        return chk.load_allowlist(self._write(text))

    def test_absent_file_is_an_empty_allowlist(self):
        self.assertEqual(chk.load_allowlist(Path("/nonexistent/allowlist.json")), {})

    def test_a_non_empty_string_reason_is_kept_verbatim(self):
        self.assertEqual(
            self._load('{"solid-server": "run by the e2e suite, not by js.yml"}'),
            {"solid-server": "run by the e2e suite, not by js.yml"},
        )

    def test_a_reasonless_entry_exits_2_rather_than_exempting(self):
        # `str(v)` would have turned each of these into a truthy exemption.
        for body in ('{"x": ""}', '{"x": "   "}', '{"x": null}', '{"x": false}',
                     '{"x": true}', '{"x": 0}', '{"x": 1}', '{"x": []}', '{"x": {}}',
                     '{"x": ["why"]}'):
            with self.subTest(body=body):
                with self.assertRaises(SystemExit) as cm:
                    self._load(body)
                self.assertEqual(cm.exception.code, 2)

    def test_a_key_that_is_not_a_bare_package_name_exits_2(self):
        for body in ('{"packages/x": "why"}', '{"": "why"}', '{"  ": "why"}',
                     '{"../x": "why"}', '{"x/": "why"}'):
            with self.subTest(body=body):
                with self.assertRaises(SystemExit) as cm:
                    self._load(body)
                self.assertEqual(cm.exception.code, 2)

    def test_malformed_json_and_non_object_exit_2(self):
        for body in ("{", '["x"]', '"x"'):
            with self.subTest(body=body):
                with self.assertRaises(SystemExit) as cm:
                    self._load(body)
                self.assertEqual(cm.exception.code, 2)

    def test_errors_name_every_bad_entry(self):
        errors = chk.allowlist_entry_errors({"x": "", "packages/y": "why", "z": "ok"})
        self.assertEqual(len(errors), 2)
        self.assertTrue(any("'x'" in e and "written reason" in e for e in errors))
        self.assertTrue(any("packages/y" in e for e in errors))


class TestLiveRepository(unittest.TestCase):
    """The real packages/ + .github/workflows/ must satisfy the invariant — and
    must be seen to satisfy it by positive attribution, not by an empty scan."""

    def setUp(self):
        import json

        self.packages = chk.discover_packages(chk.PACKAGES_DIR)
        self.globs = chk.workspace_globs(
            json.loads((REPO_ROOT / "package.json").read_text(encoding="utf-8"))
        )
        self.covered = chk.load_workflow_targets(chk.WORKFLOWS_DIR)

    def test_every_real_package_with_a_test_script_is_attributed_to_a_leg(self):
        tested = {
            d: m for d, m in self.packages.items() if chk.package_has_test_script(m)
        }
        self.assertTrue(tested, "no packages/* declares a `test` script — fixture rot")
        for pkg_dir, manifest in tested.items():
            with self.subTest(package=pkg_dir):
                self.assertTrue(
                    pkg_dir in self.covered or manifest.get("name") in self.covered,
                    f"{pkg_dir} is not invoked by any workflow step",
                )

    def test_live_verdict_is_pass(self):
        self.assertEqual(
            chk.evaluate(
                self.packages,
                self.globs,
                self.covered,
                allowlist=chk.load_allowlist(chk.ALLOWLIST),
            ),
            [],
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
