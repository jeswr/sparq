#!/usr/bin/env python3
# [OPUS-5] Hermetic tests for scripts/preflight.py — the diff-scoped pre-submit
# gate runner.
#
# EVERY test here is NAMED for the guard it pins and is written to go RED when
# that guard is deleted or inverted. The mutation log in the PR body records the
# measured kill for each. Follows scripts/tests/test_gates.py: stdlib only, no
# pytest required, no network; the two end-to-end cases build a throwaway git tree
# under tempfile rather than touching the real repo.
#
# Run:  python3 scripts/tests/test_preflight.py

from __future__ import annotations

import importlib.util
import subprocess
import sys
import tempfile
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


pf = _load("preflight", "preflight.py")

RUST = "crates/sparq-x/src/lib.rs"
PY = "scripts/thing.py"


class GuardShapeDetection(unittest.TestCase):
    """`added_symbols` must recognise a guard-shaped SURFACE and nothing else."""

    def test_pub_fn_with_guard_stem_is_detected(self) -> None:
        # Kills: deleting the `validate` stem from GUARD_STEMS.
        self.assertEqual(
            pf.added_symbols({RUST: ["pub fn validate_envelope(e: &E) -> bool {"]}),
            [(RUST, "validate_envelope")],
        )

    def test_private_rust_fn_is_not_a_surface(self) -> None:
        # Kills: dropping the `pub` requirement from RUST_GUARD_RE — the check
        # would then fire on every private helper and workers would disable it.
        self.assertEqual(pf.added_symbols({RUST: ["fn check_bounds(n: usize) {"]}), [])

    def test_pub_fn_without_a_guard_stem_is_not_flagged(self) -> None:
        # Kills: widening RUST_GUARD_RE to any `pub fn`.
        self.assertEqual(pf.added_symbols({RUST: ["pub fn render_row(r: &R) {"]}), [])

    def test_nested_python_def_is_not_a_surface(self) -> None:
        # Kills: relaxing PY_GUARD_RE's column-0 anchor.
        self.assertEqual(pf.added_symbols({PY: ["    def validate_lease(x):"]}), [])

    def test_module_level_python_def_is_a_surface(self) -> None:
        self.assertEqual(
            pf.added_symbols({PY: ["def validate_lease(x):"]}), [(PY, "validate_lease")]
        )

    def test_paths_outside_crate_src_and_scripts_are_out_of_scope(self) -> None:
        # Kills: dropping the RUST_SRC_RE / PY_SCRIPT_RE path filter.
        self.assertEqual(
            pf.added_symbols({"docs/x.rs": ["pub fn validate_envelope() {"]}), []
        )


class SuppressionRequiresAReason(unittest.TestCase):
    """The escape hatch's whole value is the recorded reason."""

    def test_marker_with_a_reason_suppresses(self) -> None:
        self.assertEqual(
            pf.added_symbols(
                {RUST: [
                    "// preflight-allow: guard-untested — proven by the kani harness",
                    "pub fn validate_envelope() {",
                ]}
            ),
            [],
        )

    def test_marker_without_a_reason_does_not_suppress(self) -> None:
        # Kills: dropping the `and m.group("why").strip()` conjunct in suppressed().
        # Without it a bare `preflight-allow: guard-untested —` silences the check,
        # which is exactly the fail-open the corpus is full of.
        self.assertEqual(
            pf.added_symbols(
                {RUST: ["// preflight-allow: guard-untested —", "pub fn validate_envelope() {"]}
            ),
            [(RUST, "validate_envelope")],
        )

    def test_marker_for_a_different_check_does_not_suppress(self) -> None:
        # Kills: ignoring the check name in SUPPRESS_RE.
        self.assertEqual(
            pf.added_symbols(
                {RUST: [
                    "// preflight-allow: no-perf-numbers — unrelated",
                    "pub fn validate_envelope() {",
                ]}
            ),
            [(RUST, "validate_envelope")],
        )

    def test_a_hyphenated_check_name_parses_whole(self) -> None:
        # REGRESSION (mutation M3). SUPPRESS_RE originally allowed the separator to
        # be a bare `-`, so `[a-z0-9-]+` backtracked and ate its own hyphen:
        # `preflight-allow: guard-untested —` parsed as check=`guard`,
        # why=`untested —`. A marker with NO reason therefore produced a
        # well-formed match for a check name nobody wrote. Requiring whitespace on
        # BOTH sides of the separator is what makes this parse correctly — invert
        # that and this test reds.
        m = pf.SUPPRESS_RE.search("// preflight-allow: guard-untested — real reason")
        self.assertIsNotNone(m)
        assert m is not None
        self.assertEqual(m.group("check"), "guard-untested")
        self.assertEqual(m.group("why"), "real reason")

    def test_a_reasonless_marker_does_not_match_at_all(self) -> None:
        # The reason is mandatory in the REGEX, not in a downstream conjunct. If
        # this ever matches, `suppressed()` has a fail-open escape hatch.
        self.assertIsNone(pf.SUPPRESS_RE.search("// preflight-allow: guard-untested —"))
        self.assertIsNone(pf.SUPPRESS_RE.search("// preflight-allow: guard-untested —   "))

    def test_marker_two_lines_above_does_not_suppress(self) -> None:
        # Kills: widening the suppression window past (idx, idx-1), which would let
        # an unrelated marker elsewhere in the hunk silence a guard.
        self.assertEqual(
            pf.added_symbols(
                {RUST: [
                    "// preflight-allow: guard-untested — stale marker",
                    "",
                    "pub fn validate_envelope() {",
                ]}
            ),
            [(RUST, "validate_envelope")],
        )


class AddedLineParsing(unittest.TestCase):
    def test_only_added_lines_are_read(self) -> None:
        # Kills: reading removed or context lines — a guard DELETED by the diff
        # would otherwise be reported as newly untested.
        diff = (
            "diff --git a/scripts/a.py b/scripts/a.py\n"
            "--- a/scripts/a.py\n"
            "+++ b/scripts/a.py\n"
            "@@ -1,0 +2 @@\n"
            "+def validate_x(v):\n"
            "-def validate_gone(v):\n"
            " def context_untouched(v):\n"
        )
        self.assertEqual(pf.parse_added(diff), {"scripts/a.py": ["def validate_x(v):"]})

    def test_the_plus_plus_plus_header_is_not_treated_as_content(self) -> None:
        diff = "--- a/scripts/a.py\n+++ b/scripts/a.py\n@@ -0,0 +1 @@\n+def validate_x(v):\n"
        self.assertNotIn("+ b/scripts/a.py", pf.parse_added(diff)["scripts/a.py"])


class _Tree:
    """A throwaway git tree so check_guard_untested's `git ls-files` has something."""

    def __init__(self, td: str) -> None:
        self.root = Path(td)
        (self.root / "crates/sparq-x/src").mkdir(parents=True)
        (self.root / "crates/sparq-x/src/lib.rs").write_text("pub fn validate_envelope() {}\n")
        subprocess.run(["git", "init", "-q"], cwd=self.root, check=True)
        self.stage()

    def stage(self) -> None:
        subprocess.run(["git", "add", "-A"], cwd=self.root, check=True)

    def write(self, rel: str, text: str) -> None:
        p = self.root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)
        self.stage()


ADDED = {"crates/sparq-x/src/lib.rs": ["pub fn validate_envelope() {}"]}


class EndToEndGuardUntested(unittest.TestCase):
    """The behaviour the check exists for, exercised through the real entry point."""

    def test_guard_with_no_test_anywhere_reds(self) -> None:
        # THE headline guard. Kills: making check_guard_untested return [] always.
        with tempfile.TemporaryDirectory() as td:
            t = _Tree(td)
            self.assertEqual(len(pf.check_guard_untested(ADDED, t.root)), 1)

    def test_guard_named_by_an_integration_test_passes(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            t = _Tree(td)
            t.write("crates/sparq-x/tests/t.rs",
                    "#[test]\nfn t() { sparq_x::validate_envelope(); }\n")
            self.assertEqual(pf.check_guard_untested(ADDED, t.root), [])

    def test_an_unrelated_test_does_not_satisfy_the_guard(self) -> None:
        # THE anti-vacuity case. If this passed, any test file anywhere in the repo
        # would satisfy every guard and the check would be decorative.
        # Kills: replacing the per-symbol word search with "is there any test file".
        with tempfile.TemporaryDirectory() as td:
            t = _Tree(td)
            t.write("crates/sparq-x/tests/t.rs",
                    "#[test]\nfn t() { sparq_x::something_else(); }\n")
            self.assertEqual(len(pf.check_guard_untested(ADDED, t.root)), 1)

    def test_a_substring_match_does_not_satisfy_the_guard(self) -> None:
        # Kills: dropping the \b word boundaries — `validate_envelope_legacy` in an
        # old test would otherwise silently satisfy a new `validate_envelope`.
        with tempfile.TemporaryDirectory() as td:
            t = _Tree(td)
            t.write("crates/sparq-x/tests/t.rs",
                    "#[test]\nfn t() { sparq_x::validate_envelope_legacy(); }\n")
            self.assertEqual(len(pf.check_guard_untested(ADDED, t.root)), 1)

    def test_an_in_file_cfg_test_module_satisfies_the_guard(self) -> None:
        # Kills: restricting the corpus to tests/ directories only — a Rust unit
        # test legitimately lives in the same file as the guard.
        with tempfile.TemporaryDirectory() as td:
            t = _Tree(td)
            t.write("crates/sparq-x/src/lib.rs",
                    "pub fn validate_envelope() {}\n"
                    "#[cfg(test)]\nmod tests {\n"
                    "  #[test] fn t() { super::validate_envelope(); }\n}\n")
            self.assertEqual(pf.check_guard_untested(ADDED, t.root), [])

    def test_a_python_in_file_self_test_block_satisfies_the_guard(self) -> None:
        # REGRESSION: measured false positive on the real tree. This repo's
        # dominant convention for scripts/*.py is an in-file `--self-test` block,
        # not a tests/ file. scripts/release-interval-guard.py::check_version_group
        # is exercised only there, and the first version of this check flagged it.
        # Delete the is_py_script self-test branch in test_corpus() and this reds.
        with tempfile.TemporaryDirectory() as td:
            t = _Tree(td)
            t.write("scripts/thing.py",
                    "def validate_lease(x):\n    return x\n"
                    "def self_test():\n    assert validate_lease(1) == 1\n")
            added = {"scripts/thing.py": ["def validate_lease(x):"]}
            self.assertEqual(pf.check_guard_untested(added, t.root), [])

    def test_a_python_script_with_no_self_test_does_not_satisfy_the_guard(self) -> None:
        # The other direction: a plain script that merely CALLS the guard is not a
        # test host. Without this, every call site would satisfy every guard.
        with tempfile.TemporaryDirectory() as td:
            t = _Tree(td)
            t.write("scripts/thing.py", "def validate_lease(x):\n    return x\n")
            t.write("scripts/caller.py", "import thing\nthing.validate_lease(1)\n")
            added = {"scripts/thing.py": ["def validate_lease(x):"]}
            self.assertEqual(len(pf.check_guard_untested(added, t.root)), 1)

    def test_a_non_test_rust_src_file_naming_the_guard_does_not_satisfy_it(self) -> None:
        # Kills: dropping the `#[cfg(test)]` requirement on in-src corpus entries —
        # an ordinary CALL SITE would then count as a test.
        with tempfile.TemporaryDirectory() as td:
            t = _Tree(td)
            t.write("crates/sparq-x/src/caller.rs",
                    "pub fn go() { crate::validate_envelope(); }\n")
            self.assertEqual(len(pf.check_guard_untested(ADDED, t.root)), 1)


class ExitCodeContract(unittest.TestCase):
    """A finding must make the process exit non-zero — no exit-zero swallowing."""

    def test_main_exits_nonzero_on_a_finding(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            t = _Tree(td)
            cf = Path(td) / "changed.txt"
            cf.write_text("crates/sparq-x/src/lib.rs\n")
            al = Path(td) / "added.diff"
            al.write_text("--- a/crates/sparq-x/src/lib.rs\n"
                          "+++ b/crates/sparq-x/src/lib.rs\n"
                          "@@ -0,0 +1 @@\n"
                          "+pub fn validate_envelope() {}\n")
            rc = pf.main(["--root", str(t.root), "--changed-files", str(cf),
                          "--added-lines", str(al), "--only", "guard-untested", "--quiet"])
            self.assertEqual(rc, 1)

    def test_main_exits_zero_when_clean(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            t = _Tree(td)
            t.write("crates/sparq-x/tests/t.rs",
                    "#[test]\nfn t() { sparq_x::validate_envelope(); }\n")
            cf = Path(td) / "changed.txt"
            cf.write_text("crates/sparq-x/src/lib.rs\n")
            al = Path(td) / "added.diff"
            al.write_text("--- a/crates/sparq-x/src/lib.rs\n"
                          "+++ b/crates/sparq-x/src/lib.rs\n"
                          "@@ -0,0 +1 @@\n"
                          "+pub fn validate_envelope() {}\n")
            rc = pf.main(["--root", str(t.root), "--changed-files", str(cf),
                          "--added-lines", str(al), "--only", "guard-untested", "--quiet"])
            self.assertEqual(rc, 0)


class NonMechanicalObligationsAreStated(unittest.TestCase):
    """The script must never imply it has checked what it cannot check."""

    def test_the_mutation_obligation_is_printed(self) -> None:
        # Kills: deleting the NON_MECHANICAL block, which would leave a PASS reading
        # as "this diff is reviewed" rather than "the mechanical gates are green".
        self.assertIn("MUTATE YOUR HEADLINE GUARD", pf.NON_MECHANICAL)

    def test_the_claim_vs_code_obligation_is_printed(self) -> None:
        self.assertIn("READ YOUR OWN PROSE AGAINST YOUR OWN DIFF", pf.NON_MECHANICAL)


WORKER_BRIEFS = (
    "sparq-rust-impl.md",
    "sparq-rust-feature.md",
    "sparq-ci-infra.md",
    "sparq-docs.md",
    "sparq-site.md",
    "sparq-perf-engineer.md",
)


class WorkerBriefsCarryThePresubmitBlock(unittest.TestCase):
    """One brief silently losing a clause is a known, repeated failure here.

    A worker reads only its OWN brief, so an obligation that lives in five of six
    is absent for the sixth. These tests red the moment a brief drops the block or
    the copies drift apart.
    """

    def _briefs(self) -> dict[str, str]:
        return {
            n: (REPO_ROOT / ".claude" / "agents" / n).read_text()
            for n in WORKER_BRIEFS
        }

    def test_every_worker_brief_names_the_preflight_runner(self) -> None:
        missing = [n for n, t in self._briefs().items() if "scripts/preflight.py" not in t]
        self.assertEqual(missing, [], f"worker briefs missing the preflight step: {missing}")

    def test_every_worker_brief_carries_the_mutation_obligation(self) -> None:
        missing = [n for n, t in self._briefs().items() if "MUTATE YOUR HEADLINE GUARD" not in t]
        self.assertEqual(missing, [], f"briefs missing the mutation obligation: {missing}")

    def test_every_worker_brief_carries_the_claim_vs_code_obligation(self) -> None:
        missing = [
            n for n, t in self._briefs().items()
            if "READ YOUR OWN PROSE AGAINST YOUR OWN DIFF" not in t
        ]
        self.assertEqual(missing, [], f"briefs missing the claim-vs-code obligation: {missing}")

    def test_the_block_is_byte_identical_across_briefs(self) -> None:
        # Divergent copies are how a "shared contract" quietly stops being shared.
        marker = "## Before you open the PR (HARD — identical in every worker brief)"
        blocks = {}
        for n, t in self._briefs().items():
            self.assertIn(marker, t, f"{n} lost the block header")
            body = t.split(marker, 1)[1]
            # the block ends at the next top-level heading, or EOF
            end = body.find("\n## ")
            blocks[n] = body[:end] if end != -1 else body.rstrip() + "\n"
        distinct = set(b.rstrip() for b in blocks.values())
        self.assertEqual(
            len(distinct), 1,
            "the pre-submit block has diverged across worker briefs:\n"
            + "\n".join(f"  {n}: {len(b)} chars" for n, b in blocks.items()),
        )


class TheYamlSeamIsGating(unittest.TestCase):
    """The wiring, not just the logic.

    Measured on this repo: in an 18-mutant run every UNCAUGHT mutant lived at the
    workflow `if:`/step/call-site, not in the Python. A checker wired into an
    advisory job, guarded by an `if:`, or carrying `continue-on-error` is a checker
    that does not check. These tests red on each of those.
    """

    WORKFLOW = ".github/workflows/docs-quality.yml"
    STEP_RUNS = (
        "python3 scripts/preflight.py --self-test",
        "python3 scripts/tests/test_preflight.py",
    )

    def _job_hosting(self, run_cmd: str):
        import yaml

        doc = yaml.safe_load((REPO_ROOT / self.WORKFLOW).read_text())
        for jid, job in doc["jobs"].items():
            for step in job.get("steps", []):
                if run_cmd in str(step.get("run", "")):
                    return jid, job, step
        return None, None, None

    def test_both_preflight_legs_are_wired_into_the_workflow(self) -> None:
        for cmd in self.STEP_RUNS:
            jid, _job, _step = self._job_hosting(cmd)
            self.assertIsNotNone(jid, f"no step in {self.WORKFLOW} runs: {cmd}")

    def test_the_hosting_job_name_carries_no_advisory_token(self) -> None:
        # ci-summary EXCLUDES any check-run whose name matches
        # \b(advisory|informational|non-blocking)\b. Renaming the job to include one
        # of those tokens silently un-gates every leg in it.
        import re as _re

        for cmd in self.STEP_RUNS:
            jid, job, _step = self._job_hosting(cmd)
            assert job is not None
            name = str(job.get("name") or jid)
            self.assertIsNone(
                _re.search(r"\b(advisory|informational|non-blocking)\b", name, _re.I),
                f"{cmd} is hosted by job {name!r}, which ci-summary would EXCLUDE",
            )

    def test_neither_leg_can_swallow_its_own_failure(self) -> None:
        # continue-on-error at EITHER the job or the step level turns a red leg
        # green. This is the exact shape of the exit-zero swallowing defect the
        # verdict corpus reports 11 times.
        for cmd in self.STEP_RUNS:
            jid, job, step = self._job_hosting(cmd)
            assert job is not None and step is not None
            self.assertNotEqual(job.get("continue-on-error"), True,
                                f"job {jid} hosting {cmd} is continue-on-error")
            self.assertNotEqual(step.get("continue-on-error"), True,
                                f"the step running {cmd} is continue-on-error")

    def test_neither_leg_is_conditionally_skipped(self) -> None:
        # An `if:` on the step is the cheapest way to make a gate vacuous.
        for cmd in self.STEP_RUNS:
            _jid, _job, step = self._job_hosting(cmd)
            assert step is not None
            self.assertIsNone(step.get("if"),
                              f"the step running {cmd} is guarded by an if:")


class SelfTestIsWired(unittest.TestCase):
    def test_self_test_passes(self) -> None:
        proc = subprocess.run(
            [sys.executable, str(REPO_ROOT / "scripts" / "preflight.py"), "--self-test"],
            capture_output=True, text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
