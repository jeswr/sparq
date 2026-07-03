#!/usr/bin/env python3
# [FABLE-5] sq-fmx4u.3: cross-file INSPECTION tests for the change-based
# test-selection wiring — the acceptance criterion "all guards verified
# fail-closed by inspection test (empty outputs => run)".
# [SONNET-4.6] sq-fmx4u.4: TestRequiredCheckAnchor — pin the three structural
# properties that make plain-skip merge-queue-safe (design §5.3 graduation note).
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
#   * REQUIRED-CHECK ANCHOR (sq-fmx4u.4) — the `ci-summary / gate` job must
#     retain the name "gate" (branch-protection context anchor), have no job-
#     level `if:` guard (always runs → queue never times out waiting), and the
#     workflow must trigger on merge_group. All three make plain-skip safe.
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


def _on_block(wf: dict) -> dict:
    """The workflow `on:` mapping. Bare `on` is the YAML boolean True, so PyYAML
    keys it under `True`; fall back to that (same trick as the anchor tests)."""
    return wf.get("on", wf.get(True, {})) or {}


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

    def test_enforce_is_default_shadow_is_the_escape_hatch(self):
        """[FABLE-5] sq-fmx4u.5 ENFORCEMENT FLIP. Enforce is now the DEFAULT — the
        selector's mode=selected skips are honored unless CI_SELECT_MODE is set to
        the literal 'shadow' (the report-only rollback escape hatch). The pre-flip
        shadow-by-default condition (`!= "enforce"` => --shadow) must be GONE."""
        text = SELECT_YML.read_text(encoding="utf-8")
        self.assertIn("--shadow", text, "the shadow escape hatch must still exist")
        self.assertIn("CI_SELECT_MODE", text)
        self.assertIn('= "shadow"', text,
                      "shadow must now be OPT-IN (CI_SELECT_MODE == 'shadow'); "
                      "enforce is the default")
        self.assertNotIn('!= "enforce"', text,
                         "the pre-flip shadow-by-default condition must be removed "
                         "(sq-fmx4u.5 enforce flip)")
        # Structural: the select step wires the shadow branch off CI_SELECT_MODE.
        step = self._select_step()
        self.assertIn("CI_SELECT_MODE", step.get("env", {}))

    def _select_step(self) -> dict:
        steps = self.sel["jobs"]["select"]["steps"]
        return next(s for s in steps if s.get("id") == "sel")

    def test_ci_full_label_override_forces_full(self):
        """[FABLE-5] sq-fmx4u.5 (design §6.2). Applying the `ci-full` PR label maps
        to ci_select.py --full (mode=full, nothing skipped). The override reads the
        PR label set and, when present, adds --full."""
        text = SELECT_YML.read_text(encoding="utf-8")
        self.assertIn("ci-full", text, "the ci-full label override must be wired")
        step = self._select_step()
        env = step.get("env", {})
        self.assertIn("CI_FULL_LABEL", env, "the select step must compute CI_FULL_LABEL")
        self.assertIn("labels.*.name", str(env["CI_FULL_LABEL"]),
                      "CI_FULL_LABEL must read the PR label name set")
        self.assertIn("'ci-full'", str(env["CI_FULL_LABEL"]))
        run = str(step["run"])
        self.assertIn("--full", run, "the ci-full path must force ci_select.py --full")
        # The label check must gate the --full branch (fail-open toward RUNNING more).
        self.assertIn('[ "$CI_FULL_LABEL" = "true" ]', run)

    def test_label_toggle_reevaluates_selection(self):
        """[FABLE-5] sq-fmx4u.5. Toggling the ci-full label must re-run selection, so
        both selection-consuming workflows react to labeled/unlabeled — and must NOT
        drop the default `synchronize` (push-to-PR) trigger while doing so."""
        for wf_name, wf in (("ci.yml", self.ci), ("feature-matrix.yml", self.fm)):
            on = _on_block(wf)
            pr = on.get("pull_request") or {}
            self.assertIsInstance(
                pr, dict, f"{wf_name}: pull_request must carry a types list")
            types = pr.get("types", [])
            for needed in ("labeled", "unlabeled"):
                self.assertIn(needed, types,
                              f"{wf_name}: pull_request must react to {needed} so the "
                              f"ci-full label toggle re-evaluates selection")
            for keep in ("opened", "synchronize", "reopened"):
                self.assertIn(keep, types,
                              f"{wf_name}: must keep the default {keep} trigger")

    def test_nightly_full_matrix_backstop(self):
        """[FABLE-5] sq-fmx4u.5 (design §6.1). ci.yml runs on a cron schedule (the
        nightly full-matrix backstop) and carries the `selection-backstop` guard job
        that fail-loud asserts a scheduled/dispatch run resolves to mode=full."""
        on = _on_block(self.ci)
        self.assertIn("schedule", on, "ci.yml must run on a cron schedule (nightly)")
        job = self.ci["jobs"].get("selection-backstop")
        self.assertIsNotNone(job, "ci.yml must carry the nightly selection-backstop guard")
        # Runs ONLY on the backstop events (skipped => gate-satisfied elsewhere).
        cond = str(job.get("if", ""))
        self.assertIn("schedule", cond)
        self.assertIn("workflow_dispatch", cond)
        # It must depend on select and read its mode, failing if it is not 'full'.
        needs = job.get("needs", [])
        needs = [needs] if isinstance(needs, str) else needs
        self.assertIn("select", needs, "backstop must need the select job")
        run = " ".join(str(s.get("run", "")) for s in job.get("steps", []))
        self.assertIn('"$SELECT_MODE" != "full"', run,
                      "the backstop must fail-loud when the scheduled run is not full")
        self.assertIn("exit 1", run)

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


# [SONNET-4.6] sq-fmx4u.4: required-check anchor tests.
#
# VERIFIED SEMANTICS (2026-07-03, against ruleset id 17688455):
# `gh api repos/sparq-org/sparq/rulesets/17688455` shows required_status_checks
# contains EXACTLY ONE entry: {"context":"gate","integration_id":15368} — i.e.
# `ci-summary / gate` is the SOLE required check.  No individual per-crate matrix
# legs, no individual `opt-in <X>` legs are in the required list.  The merge queue
# (grouping_strategy=ALLGREEN, check_response_timeout_minutes=60) therefore blocks
# ONLY on the single "gate" check, not on absent or skipped siblings.
#
# Consequence: PLAIN-SKIP IS SAFE.  A selection-skipped job (conclusion=skipped
# from a job-level `if:` guard) reports a complete check-run; the gate already
# treats `skipped` as non-failing when the select pre-job succeeded.  A leg
# filtered at assembly time (unassembled → no check-run spawned) is also safe
# because no leg name is individually required.  Neither case hangs the queue.
#
# The shim (guard moved inside the job as a first step so the job always occupies
# a slot but exits early) is NOT needed.  It would only be needed if a per-crate
# leg name appeared in required_status_checks — it does not.
#
# Three structural properties must not drift for the above to remain true.  They
# are pinned here (hermetically, no network/API calls):
#   (1) gate job name == "gate"  (matches the ruleset's context:"gate")
#   (2) gate job has NO if: guard  (always runs → merge queue always gets a
#       response within the 60-minute timeout window)
#   (3) ci-summary.yml triggers on merge_group  (required for the gate to produce
#       a check-run on queue entries at all)
# Bead sq-fmx4u.5 can safely flip CI_SELECT_MODE to "enforce" once these hold.
CISUM_YML = REPO_ROOT / ".github" / "workflows" / "ci-summary.yml"


class TestRequiredCheckAnchor(unittest.TestCase):
    """[SONNET-4.6] sq-fmx4u.4: pin the three properties that make plain-skip
    merge-queue-safe in this repo (design §5.3 graduation note)."""

    @classmethod
    def setUpClass(cls):
        cls.cs = _load(CISUM_YML)
        cls.gate = _gate_module()

    def test_gate_job_name_is_exactly_gate(self):
        """The branch-protection ruleset (id 17688455) requires context='gate'.
        If this name drifts the required check silently stops matching and the
        gate weakens with no error anywhere — so pin it."""
        job = self.cs["jobs"]["gate"]
        self.assertEqual(
            job["name"], "gate",
            "ci-summary gate job name must stay exactly 'gate' — "
            "the branch-protection ruleset anchors on context='gate' (id 17688455); "
            "renaming breaks the required check without any CI warning",
        )

    def test_gate_job_has_no_job_level_conditional(self):
        """The gate must run unconditionally on every event (pull_request,
        merge_group, push).  A job-level `if:` could silently skip it on some
        triggers, leaving the merge queue waiting 60 minutes before timing out."""
        job = self.cs["jobs"]["gate"]
        self.assertNotIn(
            "if", job,
            "ci-summary gate job must have no `if:` guard — it must always run "
            "so the merge queue receives the required 'gate' check-run within the "
            "60-minute check_response_timeout window (ruleset 17688455)",
        )

    def test_ci_summary_triggers_on_merge_group(self):
        """Without merge_group trigger ci-summary never runs on queue entries and
        the merge queue hangs forever waiting for the required 'gate' check."""
        on = self.cs.get("on", self.cs.get(True, {}))
        self.assertIn(
            "merge_group", on,
            "ci-summary must trigger on merge_group — absent, the queue entry "
            "never receives the required 'gate' check-run and hangs until timeout",
        )

    def test_gate_is_not_advisory_or_informational(self):
        """The gate name must never accidentally match the advisory/informational
        exclusion — that would un-gate it (its failure would stop being required).
        Both the bare job name and the `workflow / job` display form are checked."""
        for name in ("gate", "ci-summary / gate"):
            self.assertFalse(
                self.gate.is_advisory(name),
                f"gate check name {name!r} must NOT match the advisory exclusion",
            )

    def test_gate_is_not_detected_as_a_select_job(self):
        """The gate name must not match SELECT_RE — that would make the gate
        self-detect as a selection pre-job and add a circular verdict dependency
        (skipped gating only valid if 'gate' itself concluded success)."""
        for name in ("gate", "ci-summary / gate"):
            self.assertFalse(
                self.gate.is_select(name),
                f"gate check name {name!r} must NOT match SELECT_RE",
            )


if __name__ == "__main__":
    unittest.main(verbosity=2)
