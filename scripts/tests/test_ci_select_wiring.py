#!/usr/bin/env python3
# [FABLE-5] sq-fmx4u.3: cross-file INSPECTION tests for the change-based
# test-selection wiring — the acceptance criterion "all guards verified
# fail-closed by inspection test (empty outputs => run)".
#
# YAML `if:` expressions cannot be unit-tested by execution, so this suite pins
# their SHAPE instead, plus every cross-file contract the wiring relies on:
#   * FAIL-CLOSED GUARDS — every job-level `if:` that consumes the selection
#     outputs must carry the leading disjunct `needs.select.outputs.mode !=
#     'selected'`: an EMPTY/missing `mode` output (select failed, output lost)
#     satisfies it => the job RUNS (design §4.3). Nothing may skip on any
#     condition reachable with empty outputs.
#   * REAL CRATE NEEDLES — every literal '"<crate>"' needle passed to
#     contains(needs.select.outputs.affected, ...) must be a CURRENT workspace
#     member: a typo'd/renamed needle would make its lane skip exactly when it
#     should run (the silent-unsound-skip class this whole design forbids).
#   * NAME CONTRACT — the ci-select.yml job name must match the gate's
#     SELECT_RE (scripts/ci_summary_gate.py) and must NOT match the advisory
#     exclusion; otherwise a select failure could stop gating skips.
#   * UNCONDITIONAL SELECT — the `select` caller jobs must have no `if`/`needs`
#     (the gate REDs when a selection check-run is missing-not-success, so the
#     job must exist and run on every trigger).
#   * matrix-context trap — the per-shard skip must NOT be a job-level `if:`
#     referencing `matrix.*` (the matrix context is unavailable there and
#     silently evaluates empty — under enforcement that would skip every leg).
#
# Needs PyYAML (same dependency the assembler already has); everything else is
# stdlib. Run:  python3 scripts/tests/test_ci_select_wiring.py

from __future__ import annotations

import importlib.util
import re
import sys
import unittest
from pathlib import Path

try:
    import tomllib  # stdlib >= 3.11
except ModuleNotFoundError:  # pragma: no cover
    tomllib = None

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
CI_YML = REPO_ROOT / ".github" / "workflows" / "ci.yml"
FM_YML = REPO_ROOT / ".github" / "workflows" / "feature-matrix.yml"
SELECT_YML = REPO_ROOT / ".github" / "workflows" / "ci-select.yml"
GATE_PY = REPO_ROOT / "scripts" / "ci_summary_gate.py"

FAIL_CLOSED_DISJUNCT = "needs.select.outputs.mode != 'selected'"
NEEDLE_RE = re.compile(
    r"contains\(needs\.select\.outputs\.affected,\s*'\"([A-Za-z0-9_-]+)\"'\)"
)


def _load(path: Path) -> dict:
    return yaml.safe_load(path.read_text(encoding="utf-8"))


def _gate_module():
    spec = importlib.util.spec_from_file_location("ci_summary_gate", GATE_PY)
    mod = importlib.util.module_from_spec(spec)
    sys.modules["ci_summary_gate"] = mod
    spec.loader.exec_module(mod)
    return mod


def _workspace_members() -> set[str]:
    """Crate names from crates/*/Cargo.toml — hermetic (no cargo invocation)."""
    assert tomllib is not None, "Python >= 3.11 required (tomllib)"
    members = set()
    for manifest in sorted((REPO_ROOT / "crates").glob("*/Cargo.toml")):
        with open(manifest, "rb") as fh:
            data = tomllib.load(fh)
        name = data.get("package", {}).get("name")
        if isinstance(name, str):
            members.add(name)
    return members


class TestWiring(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ci = _load(CI_YML)
        cls.fm = _load(FM_YML)
        cls.sel = _load(SELECT_YML)
        cls.members = _workspace_members()
        cls.gate = _gate_module()

    # ---- fail-closed guard shape -------------------------------------------------
    def test_every_selection_consuming_if_carries_the_fail_closed_disjunct(self):
        """Any job-level `if:` mentioning the selection outputs must include the
        `mode != 'selected'` disjunct, so empty/missing outputs mean RUN."""
        for wf_name, wf in (("ci.yml", self.ci), ("feature-matrix.yml", self.fm)):
            for job_id, job in wf["jobs"].items():
                cond = job.get("if", "")
                if "needs.select.outputs" in str(cond):
                    self.assertIn(
                        FAIL_CLOSED_DISJUNCT, str(cond),
                        f"{wf_name}:{job_id}: selection-consuming `if:` lacks the "
                        f"fail-closed disjunct {FAIL_CLOSED_DISJUNCT!r}",
                    )

    def test_selection_guarded_jobs_need_select(self):
        """A guard reading needs.select.* only resolves if `select` is in needs."""
        for wf_name, wf in (("ci.yml", self.ci), ("feature-matrix.yml", self.fm)):
            for job_id, job in wf["jobs"].items():
                if "needs.select.outputs" in str(job.get("if", "")):
                    needs = job.get("needs", [])
                    needs = [needs] if isinstance(needs, str) else needs
                    self.assertIn("select", needs, f"{wf_name}:{job_id}")

    def test_build_archive_runs_on_any_nonempty_affected(self):
        cond = str(self.ci["jobs"]["build-archive"]["if"])
        self.assertIn(FAIL_CLOSED_DISJUNCT, cond)
        self.assertIn("needs.select.outputs.affected != '[]'", cond)

    # ---- crate needles are real --------------------------------------------------
    def test_every_affected_needle_is_a_workspace_member(self):
        text = CI_YML.read_text(encoding="utf-8") + FM_YML.read_text(encoding="utf-8")
        needles = NEEDLE_RE.findall(text)
        self.assertTrue(needles, "expected contains(affected, ...) guards to exist")
        unknown = sorted({n for n in needles if n not in self.members})
        self.assertEqual(
            unknown, [],
            f"guard needle(s) {unknown} are not workspace members — such a lane "
            f"would SKIP exactly when it should run (silent unsound skip)",
        )

    def test_shard_crate_keys_are_workspace_members(self):
        matrix = self.ci["jobs"]["test"]["strategy"]["matrix"]["include"]
        tagged = [e for e in matrix if e.get("crate")]
        self.assertTrue(tagged, "expected the heavy shards to carry a crate key")
        for entry in tagged:
            self.assertIn(entry["crate"], self.members,
                          f"shard {entry.get('name')}: crate {entry['crate']!r}")
        # Both known heavies are sparq-vectors tests; the bulk shards must stay
        # cross-crate (crate == "" -> narrowed, never skipped).
        for entry in matrix:
            if not entry.get("crate"):
                self.assertIn("partition", entry)

    # ---- matrix-context trap -----------------------------------------------------
    def test_no_job_level_if_references_matrix_context(self):
        """`matrix` is NOT available in jobs.<job_id>.if — a reference silently
        evaluates empty, which under enforcement would skip every leg. The
        per-shard guard must live in a STEP (env/run), never the job `if:`."""
        for wf_name, wf in (("ci.yml", self.ci), ("feature-matrix.yml", self.fm)):
            for job_id, job in wf["jobs"].items():
                self.assertNotIn(
                    "matrix.", str(job.get("if", "")),
                    f"{wf_name}:{job_id}: job-level if cannot see the matrix context",
                )

    # ---- select job shape ----------------------------------------------------------
    def test_select_caller_jobs_are_unconditional_and_use_the_reusable_workflow(self):
        for wf_name, wf in (("ci.yml", self.ci), ("feature-matrix.yml", self.fm)):
            job = wf["jobs"].get("select")
            self.assertIsNotNone(job, f"{wf_name}: missing the select job")
            self.assertEqual(job.get("uses"), "./.github/workflows/ci-select.yml", wf_name)
            self.assertNotIn("if", job, f"{wf_name}: select must be unconditional")
            self.assertNotIn("needs", job, f"{wf_name}: select must be unconditional")

    def test_select_job_name_matches_the_gate_detection_contract(self):
        name = self.sel["jobs"]["select"]["name"]
        self.assertTrue(self.gate.is_select(name),
                        f"gate SELECT_RE does not match the select job name {name!r}")
        self.assertTrue(self.gate.is_select(f"select / {name}"),
                        "the reusable-call display form must match too")
        self.assertFalse(self.gate.is_advisory(name),
                         "the select job name must GATE (no advisory/informational)")

    def test_shadow_default_on_until_enforce(self):
        text = SELECT_YML.read_text(encoding="utf-8")
        self.assertIn("--shadow", text)
        self.assertIn("CI_SELECT_MODE", text)
        self.assertIn('!= "enforce"', text,
                      "shadow must be the default for ANY value but the literal 'enforce'")

    # ---- narrowing + assembly wiring ----------------------------------------------
    def test_bulk_shards_narrow_with_no_tests_pass_only_under_selection(self):
        steps = self.ci["jobs"]["test"]["steps"]
        run = next(s for s in steps if "cargo nextest run" in str(s.get("run", "")))
        script = str(run["run"])
        self.assertIn('[ "$SELECT_MODE" = "selected" ]', script)
        self.assertIn("SELECT_FILTERSET", script)
        # --no-tests=pass only in the narrowed branch: a full/shadow run keeps
        # the strict no-tests-is-an-error behaviour. Count CODE lines only
        # (the flag is also, correctly, discussed in a comment).
        code_lines = [ln for ln in script.splitlines() if not ln.lstrip().startswith("#")]
        self.assertEqual("\n".join(code_lines).count("--no-tests=pass"), 1)
        env = run.get("env", {})
        for key in ("SELECT_MODE", "SELECT_AFFECTED", "SELECT_FILTERSET", "SHARD_CRATE"):
            self.assertIn(key, env, f"test run step must receive {key} via env")

    def test_feature_matrix_setup_passes_selection_into_the_assembler(self):
        steps = self.fm["jobs"]["setup"]["steps"]
        assemble = next(s for s in steps if s.get("id") == "assemble")
        env = assemble.get("env", {})
        self.assertIn("SELECT_MODE", env)
        self.assertIn("SELECT_AFFECTED", env)
        self.assertIn("--select-mode", str(assemble["run"]))
        self.assertIn("legs=", str(assemble["run"]))
        # opt-in-features must skip on a zero-leg assembly instead of exploding
        # on an empty matrix (and != '0' fail-closes: an unset output means run).
        self.assertIn("needs.setup.outputs.legs != '0'",
                      str(self.fm["jobs"]["opt-in-features"]["if"]))

    def test_members_parse_sanity(self):
        self.assertIn("sparq-core", self.members)
        self.assertGreater(len(self.members), 30)


if __name__ == "__main__":
    unittest.main(verbosity=2)
