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
import json
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
FUZZ_YML = REPO_ROOT / ".github" / "workflows" / "fuzz.yml"  # [OPUS-4.8] sq-fmx4u.6
BENCH_YML = REPO_ROOT / ".github" / "workflows" / "bench.yml"  # [SONNET-4.6] sq-mel85
SELECT_YML = REPO_ROOT / ".github" / "workflows" / "ci-select.yml"
GATE_PY = REPO_ROOT / "scripts" / "ci_summary_gate.py"
CI_SELECT_PY = REPO_ROOT / "scripts" / "ci_select.py"  # [OPUS-4.8] sq-fmx4u.6
OWNERSHIP_TOML = REPO_ROOT / "ci" / "path-ownership.toml"  # [OPUS-4.8] sq-fmx4u.6
# [OPUS-4.8] path-aware CI audit: the merge_group changed-files gates.
CONTAINER_SCAN_YML = REPO_ROOT / ".github" / "workflows" / "container-scan.yml"
SUPPLY_CHAIN_YML = REPO_ROOT / ".github" / "workflows" / "supply-chain.yml"

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


def _ci_select_module():
    # [OPUS-4.8] sq-fmx4u.6: import the selector to read `_LANE_SEEDS` — the single
    # source of truth the fuzz.yml + ci.yml `wasm` guards must mirror.
    spec = importlib.util.spec_from_file_location("ci_select", CI_SELECT_PY)
    mod = importlib.util.module_from_spec(spec)
    sys.modules["ci_select"] = mod
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
        cls.fuzz = _load(FUZZ_YML)  # [OPUS-4.8] sq-fmx4u.6
        cls.bench = _load(BENCH_YML)  # [SONNET-4.6] sq-mel85
        cls.sel = _load(SELECT_YML)
        cls.members = _workspace_members()
        cls.gate = _gate_module()
        cls.lane_seeds = _ci_select_module()._LANE_SEEDS  # [OPUS-4.8] sq-fmx4u.6

    # ---- fail-closed guard shape -------------------------------------------------
    def test_every_selection_consuming_if_carries_the_fail_closed_disjunct(self):
        """Any job-level `if:` mentioning the selection outputs must include the
        `mode != 'selected'` disjunct, so empty/missing outputs mean RUN."""
        for wf_name, wf in (("ci.yml", self.ci), ("feature-matrix.yml", self.fm),
                            ("fuzz.yml", self.fuzz), ("bench.yml", self.bench)):
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
        for wf_name, wf in (("ci.yml", self.ci), ("feature-matrix.yml", self.fm),
                            ("fuzz.yml", self.fuzz), ("bench.yml", self.bench)):
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
        text = (CI_YML.read_text(encoding="utf-8") + FM_YML.read_text(encoding="utf-8")
                + FUZZ_YML.read_text(encoding="utf-8")  # [OPUS-4.8] sq-fmx4u.6
                + BENCH_YML.read_text(encoding="utf-8"))  # [SONNET-4.6] sq-mel85
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
        for wf_name, wf in (("ci.yml", self.ci), ("feature-matrix.yml", self.fm),
                            ("fuzz.yml", self.fuzz), ("bench.yml", self.bench)):
            for job_id, job in wf["jobs"].items():
                self.assertNotIn(
                    "matrix.", str(job.get("if", "")),
                    f"{wf_name}:{job_id}: job-level if cannot see the matrix context",
                )

    # ---- select job shape ----------------------------------------------------------
    def test_select_caller_jobs_are_unconditional_and_use_the_reusable_workflow(self):
        for wf_name, wf in (("ci.yml", self.ci), ("feature-matrix.yml", self.fm),
                            ("fuzz.yml", self.fuzz), ("bench.yml", self.bench)):
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
        for wf_name, wf in (("ci.yml", self.ci), ("feature-matrix.yml", self.fm),
                            ("fuzz.yml", self.fuzz), ("bench.yml", self.bench)):
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


class TestPhase2LaneScoping(unittest.TestCase):
    """[OPUS-4.8] sq-fmx4u.6 (design §5.2, phase 2): the fuzz + wasm singleton
    lanes are scoped by their crate closure, and the coverage ratchet skips only
    on an empty affected set. Pins the guards' SHAPE + the seed sets against the
    selector's `_LANE_SEEDS` single source of truth so YAML and code cannot drift.

    The seed set a `mode != 'selected' || contains(affected, '"X"') || ...` guard
    references is exactly the set of quoted needles in the job `if:` expression."""

    @classmethod
    def setUpClass(cls):
        cls.ci = _load(CI_YML)
        cls.fuzz = _load(FUZZ_YML)
        cls.bench = _load(BENCH_YML)  # [SONNET-4.6] sq-mel85
        cls.members = _workspace_members()
        cls.lane_seeds = _ci_select_module()._LANE_SEEDS

    @staticmethod
    def _guard_needles(cond: str) -> list[str]:
        """Ordered, de-duplicated crate needles in a job `if:` expression."""
        seen: list[str] = []
        for n in NEEDLE_RE.findall(str(cond)):
            if n not in seen:
                seen.append(n)
        return seen

    # ---- fuzz lane (fuzz.yml) ------------------------------------------------------
    def test_fuzz_job_guarded_by_its_seed_closure(self):
        job = self.fuzz["jobs"]["fuzz"]
        cond = str(job.get("if", ""))
        self.assertIn(FAIL_CLOSED_DISJUNCT, cond,
                      "fuzz guard must be fail-closed (empty output => RUN)")
        needs = job.get("needs", [])
        needs = [needs] if isinstance(needs, str) else needs
        self.assertIn("select", needs, "fuzz job must need the select pre-job")
        self.assertEqual(self._guard_needles(cond), self.lane_seeds["fuzz"],
                         "fuzz guard needles must equal ci_select.py _LANE_SEEDS['fuzz']")

    def test_fuzz_has_unconditional_select_caller(self):
        job = self.fuzz["jobs"].get("select")
        self.assertIsNotNone(job, "fuzz.yml must carry the select pre-job")
        self.assertEqual(job.get("uses"), "./.github/workflows/ci-select.yml")
        self.assertNotIn("if", job, "select must be unconditional (gate needs it green)")
        self.assertNotIn("needs", job)

    def test_fuzz_runs_on_schedule_backstop(self):
        # The nightly heavy fuzz soak is the full-matrix backstop: a schedule event
        # carries no PR diff => selector mode=full => the fail-closed disjunct RUNS.
        # [FABLE-5] (2026-07-18 maintainer directive, merge-queue subset): merge_group
        # is REMOVED — the deterministic replay already gated the PR head, push-to-main
        # re-replays post-merge, and the nightly soak is the backstop. The polling
        # ci-summary gate never waits on a check that was never scheduled.
        on = _on_block(self.fuzz)
        self.assertIn("schedule", on, "fuzz.yml must keep its nightly schedule backstop")
        self.assertIn("push", on, "fuzz.yml must keep its push-to-main post-merge replay")
        self.assertNotIn("merge_group", on,
                         "fuzz.yml must NOT run on merge_group (2026-07-18 merge-queue "
                         "subset directive: PR head + push-to-main + nightly cover it)")

    # ---- differential-smoke lane (fuzz.yml) — [FABLE-5] sq-0iqzw --------------------
    def test_differential_smoke_job_guarded_by_its_seed_closure(self):
        job = self.fuzz["jobs"]["differential-smoke"]
        cond = str(job.get("if", ""))
        self.assertIn(FAIL_CLOSED_DISJUNCT, cond,
                      "differential-smoke guard must be fail-closed (empty output => RUN)")
        needs = job.get("needs", [])
        needs = [needs] if isinstance(needs, str) else needs
        self.assertIn("select", needs,
                      "differential-smoke must need the select pre-job")
        self.assertEqual(
            self._guard_needles(cond), self.lane_seeds["differential-smoke"],
            "differential-smoke guard needles must equal "
            "ci_select.py _LANE_SEEDS['differential-smoke']")
        # The job is a BLOCKING gate leg: its name must never pick up the
        # advisory/informational exclusion words (that would silently un-gate it).
        self.assertNotRegex(
            str(job.get("name", "")), r"(?i)\b(advisory|informational)\b",
            "differential-smoke must GATE — no advisory/informational token")

    # ---- bench (perf-gate) lane (bench.yml) — [SONNET-4.6] sq-mel85 ----------------
    def test_bench_job_guarded_by_its_seed_closure(self):
        job = self.bench["jobs"]["bench"]
        cond = str(job.get("if", ""))
        self.assertIn(FAIL_CLOSED_DISJUNCT, cond,
                      "bench guard must be fail-closed (empty output => RUN)")
        needs = job.get("needs", [])
        needs = [needs] if isinstance(needs, str) else needs
        self.assertIn("select", needs, "bench job must need the select pre-job")
        self.assertEqual(self._guard_needles(cond), self.lane_seeds["bench"],
                         "bench guard needles must equal ci_select.py _LANE_SEEDS['bench']")

    def test_bench_has_unconditional_select_caller(self):
        job = self.bench["jobs"].get("select")
        self.assertIsNotNone(job, "bench.yml must carry the select pre-job")
        self.assertEqual(job.get("uses"), "./.github/workflows/ci-select.yml")
        self.assertNotIn("if", job, "select must be unconditional (gate needs it green)")
        self.assertNotIn("needs", job)

    def test_bench_runs_on_pr_and_schedule_but_not_merge_group(self):
        # sq-mel85 added a nightly schedule to bench.yml as the full-run backstop: a
        # schedule event carries no PR diff => selector mode=full => the fail-closed
        # disjunct RUNS the whole suite.
        # [FABLE-5] sq-6vshe.6 (MAINTAINER-DIRECTED): merge_group is REMOVED. On a PR the
        # bench job runs the FAST DETERMINISTIC byte-count ratchet only (--deterministic-only),
        # which still gates; the merged-tree deterministic ratchet is a pure function of the code
        # (already ran on the PR head + re-runs on push-to-main), and the merged-tree wasm-
        # feature-OFF invariant is independently guarded on merge_group by vectorized-feature-off.yml,
        # so re-running bench on the merge_group ref only dragged the queue. The full NOISY timing
        # suite moved to the nightly EC2 lane (bench-ec2.yml nightly-full-bench).
        on = _on_block(self.bench)
        self.assertIn("schedule", on, "bench.yml must have the nightly schedule backstop")
        self.assertIn("pull_request", on,
                      "bench.yml must run on pull_request (the deterministic byte-count ratchet still gates)")
        self.assertNotIn("merge_group", on,
                         "bench.yml must NOT run on merge_group (sq-6vshe.6: the deterministic ratchet "
                         "already gated the PR head + re-runs on push-to-main; the noisy timing suite "
                         "moved to the nightly EC2 lane — keeping it on merge_group only dragged the queue)")

    def test_bench_main_history_and_ratchet_exclude_schedule(self):
        # CRITICAL (design §6.1 continuity, criterion (d)): the auto-ratchet + history +
        # dashboard writes must stay on the push-to-main path and NOT fire on the new
        # nightly `schedule` backstop (a scheduled run shares the last main commit's SHA,
        # so writing history would append a duplicate point + ratchet off a non-landing
        # run). Every such step's guard must exclude schedule.
        steps = self.bench["jobs"]["bench"]["steps"]
        names = [
            "Auto-ratchet the perf floor (commit improvements back to main)",
            "Ensure benchmark-data history branch exists",
            "Seed Pages dashboard onto benchmark-data (if absent)",
            # [SONNET-4.6] sq-mel85 nit: the cleanup step (restores Cargo.lock churn before
            # the benchmark-data branch switch) must also be schedule-excluded — the nightly
            # backstop never switches branches, so it needs no cleanup, and running it on
            # schedule would be a no-op that can't break anything, but pinning the guard
            # prevents silent removal of the schedule-exclusion that would leave the step
            # active on a path where `git checkout -- .` could discard useful artefacts.
            "Clean bench-induced tracked churn before history switch (main only)",
        ]
        by_name = {s.get("name"): s for s in steps}
        for n in names:
            self.assertIn(n, by_name, f"bench.yml lost the '{n}' step")
            cond = str(by_name[n].get("if", ""))
            self.assertIn("github.event_name != 'schedule'", cond,
                          f"'{n}' must exclude the schedule backstop (main continuity)")
            self.assertIn("refs/heads/main", cond,
                          f"'{n}' must stay push-to-main scoped")
        # The github-action-benchmark auto-push must also be schedule-excluded.
        store = by_name.get("Store + compare against history")
        self.assertIsNotNone(store, "bench.yml lost the history Store step")
        self.assertIn("github.event_name != 'schedule'",
                      str(store.get("with", {}).get("auto-push", "")),
                      "history auto-push must not fire on the schedule backstop")

    # ---- wasm lane (ci.yml) --------------------------------------------------------
    def test_wasm_job_guarded_by_its_seed_closure(self):
        job = self.ci["jobs"]["wasm"]
        cond = str(job.get("if", ""))
        self.assertIn(FAIL_CLOSED_DISJUNCT, cond)
        self.assertIn("needs.changes.outputs.rust_changed == 'true'", cond,
                      "wasm must keep the rust_changed path-filter guard")
        needs = job.get("needs", [])
        needs = [needs] if isinstance(needs, str) else needs
        self.assertIn("select", needs)
        self.assertIn("changes", needs)
        self.assertEqual(self._guard_needles(cond), self.lane_seeds["wasm"],
                         "wasm guard needles must equal ci_select.py _LANE_SEEDS['wasm']")

    # ---- coverage ratchet empty-affected skip (ci.yml) ----------------------------
    def test_coverage_ratchet_skips_only_on_empty_affected(self):
        job = self.ci["jobs"]["coverage-measure"]
        cond = str(job.get("if", ""))
        self.assertIn(FAIL_CLOSED_DISJUNCT, cond)
        self.assertIn("needs.select.outputs.affected != '[]'", cond,
                      "coverage ratchet may skip ONLY when affected == [] (design §5.2)")
        # It stays off the nightly path and behind rust_changed, unchanged.
        self.assertIn("github.event_name != 'schedule'", cond)
        self.assertIn("needs.changes.outputs.rust_changed == 'true'", cond)
        needs = job.get("needs", [])
        self.assertIn("select", needs)
        # Must NOT reference matrix.* in the job-level if (contexts-availability trap).
        self.assertNotIn("matrix.", cond)

    # ---- seed sets are real + mirrored ---------------------------------------------
    def test_lane_seeds_are_real_workspace_members(self):
        for lane, seeds in self.lane_seeds.items():
            for seed in seeds:
                self.assertIn(seed, self.members,
                              f"{lane} seed {seed!r} is not a workspace member")

    def test_ownership_toml_lanes_mirror_matches_lane_seeds(self):
        # The informational [lanes] table in ci/path-ownership.toml must equal the
        # selector's _LANE_SEEDS (same drift-guard idiom as [triggers]).
        self.assertIsNotNone(tomllib, "Python >= 3.11 required (tomllib)")
        with open(OWNERSHIP_TOML, "rb") as fh:
            data = tomllib.load(fh)
        mirror = data.get("lanes", {})
        self.assertEqual(mirror, self.lane_seeds,
                         "ci/path-ownership.toml [lanes] mirror drifted from _LANE_SEEDS")


# [SONNET-4.6] sq-mel85: bench metric tier invariant.
#
# INVARIANT: every mode:auto metric in bench/perf-baseline.json is either
#   (a) produced by a main-tier-only ci-bench.sh hook (a GITHUB_REF guard skips it
#       whenever GITHUB_REF != refs/heads/main, so it only runs on push-to-main and
#       local dev runs where GITHUB_REF is unset; every such event is mode=full by
#       construction, so no bench seed is needed), OR
#   (b) measured on the PR/merge_group tier, in which case its source crate must reach
#       a bench seed so a PR touching it triggers the gate.
#
# RATIONALE: the danger is a future PR-tier PROMOTION of a currently-main-tier-only
# metric (removing its GITHUB_REF guard). Without this test, that change would:
#   1. Start measuring, say, fts_bytes_per_doc on PRs that skip the bench lane (because
#      sparq-text is not a bench seed), and
#   2. Have the perf gate silently pass because the bench lane was skipped.
# This would be exactly the unsound skip the design forbids (§2). With this test:
#   - Moving a metric from main-tier-only to PR-tier fails assertion (b) in
#     _EXPECTED_MAIN_TIER below, forcing a conscious update.
#   - The update requires either adding a GITHUB_REF guard (keeping it main-only) OR
#     adding the source crate to _LANE_SEEDS["bench"] (making it a bench seed).
#
# Guard detection uses ci-bench.sh's top-level `if [...] != "refs/heads/main"` blocks
# (identified by the guard pattern at column 0) to build ranges, then checks that each
# metric's `add` call falls within a range — no hardcoded line numbers.
class TestBenchMetricTierInvariant(unittest.TestCase):
    """[SONNET-4.6] sq-mel85: pin the invariant that every mode:auto metric in
    bench/perf-baseline.json is either main-tier-only (GITHUB_REF guard in ci-bench.sh)
    or PR-tier with a bench seed covering its source crate."""

    # Pattern that identifies a top-level GITHUB_REF main-tier guard start line
    _GUARD_START_RE = re.compile(r'^if\b.*!=\s*["\']refs/heads/main["\']')
    # Pattern for any top-level `fi` (column 0, no leading whitespace)
    _TOP_FI_RE = re.compile(r'^fi\b')

    @classmethod
    def setUpClass(cls):
        baseline_path = REPO_ROOT / "bench" / "perf-baseline.json"
        with open(baseline_path, encoding="utf-8") as f:
            baseline = json.load(f)
        cls.auto_metrics = frozenset(
            k for k, v in baseline["metrics"].items() if v.get("mode") == "auto"
        )
        cls.cibench_lines = (
            REPO_ROOT / "scripts" / "ci-bench.sh"
        ).read_text(encoding="utf-8").splitlines()
        cls.lane_seeds = _ci_select_module()._LANE_SEEDS

    def _guard_ranges(self) -> list[tuple[int, int]]:
        """Return (start, end) 0-indexed inclusive ranges for each main-tier guard block.
        A block spans from the guard `if` to the next top-level `fi` at column 0.
        This relies on the verified structure of ci-bench.sh: each GITHUB_REF guard block
        opens with a top-level `if [...] != 'refs/heads/main'` and closes with a top-level
        `fi` at column 0; no nested top-level `if`/`fi` appear inside these blocks."""
        lines = self.cibench_lines
        ranges: list[tuple[int, int]] = []
        i = 0
        while i < len(lines):
            if self._GUARD_START_RE.match(lines[i]):
                start = i
                j = i + 1
                while j < len(lines) and not self._TOP_FI_RE.match(lines[j]):
                    j += 1
                ranges.append((start, j))  # j is the `fi` line (0-indexed)
                i = j + 1
            else:
                i += 1
        return ranges

    def _add_line_indices(self, metric: str) -> list[int]:
        """Return 0-indexed line numbers of add-call(s) for this metric in ci-bench.sh.
        Handles both literal names and the dynamic `add "vectors_${name}"` pattern used
        for the vectors metrics."""
        if metric.startswith("vectors_"):
            # Emitted dynamically as `add "vectors_${name}"` — match the shell expansion
            pat = re.compile(r'\badd\s+["\']?vectors_\$\{')
        else:
            pat = re.compile(r'\badd\s+["\']?' + re.escape(metric) + r'["\']?\s')
        return [i for i, ln in enumerate(self.cibench_lines) if pat.search(ln)]

    def _is_main_tier_only(self, metric: str, ranges: list[tuple[int, int]]) -> bool:
        """Return True iff every add-call for this metric falls inside a main-tier guard
        block (start < line_index < end, exclusive of the guard `if` and `fi` lines)."""
        indices = self._add_line_indices(metric)
        if not indices:
            return False  # metric not found → conservatively PR-tier
        return all(
            any(start < idx < end for start, end in ranges)
            for idx in indices
        )

    def test_auto_metric_tier_invariant(self):
        """Pin the main-tier-only vs PR-tier classification for every mode:auto metric.

        EXPECTED_MAIN_TIER: metrics behind a GITHUB_REF guard in ci-bench.sh — measured
        only on push-to-main (and local dev runs). No bench seed required (mode=full on
        all main-tier events).

        EXPECTED_PR_TIER: metrics measured on every CI tier, incl. PRs. Their source crates
        must reach a bench seed — verified by the acceptance tests in test_ci_select.py
        (test_core_pr_runs_bench / test_wasm_only_pr_runs_bench / test_engine_pr_runs_bench).

        IF THIS TEST FAILS after a change to ci-bench.sh or bench/perf-baseline.json:
          - New metric added? Classify it: add a GITHUB_REF guard (→ EXPECTED_MAIN_TIER) OR
            add its source crate to _LANE_SEEDS["bench"] (→ EXPECTED_PR_TIER).
          - Metric promoted from main-tier to PR-tier? Add its source crate to
            _LANE_SEEDS["bench"] AND move it from EXPECTED_MAIN_TIER to EXPECTED_PR_TIER.
          - NEVER leave a PR-tier hard-gated metric without a matching bench seed.
        """
        _EXPECTED_MAIN_TIER = frozenset({
            # ci-bench.sh measures these ONLY on push-to-main (GITHUB_REF guard skips on PRs):
            "fts_bytes_per_doc",           # sparq-text example; FTS_REF guard
            "vectors_diskann_recall_at10",  # sparq-vectors example; VECTOR_REF guard
            "vectors_pq_recall_at10",       # sparq-vectors example; VECTOR_REF guard
            "geo_compliance_deficit",       # sparq-geo example; GEO_REF guard
        })
        _EXPECTED_PR_TIER = frozenset({
            # ci-bench.sh measures these on EVERY tier (no GITHUB_REF guard); bench seeds cover:
            "store_bytes_per_triple",       # sparq-core (via sparq-cli + sparq-bench seeds)
            "comp_store_bytes_per_triple",  # sparq-core compressed profile (via sparq-cli + sparq-bench seeds; sq-7d3dj.32.2.5)
            "store_bytes_per_triple_small", # sparq-core (via sparq-cli + sparq-bench seeds)
            "dict_bytes_per_term",          # sparq-core (via sparq-cli + sparq-bench seeds)
            "wasm_bundle_bytes",            # sparq-wasm (direct bench seed)
        })
        # 1. The union must be exactly the full auto-metric set from perf-baseline.json.
        self.assertEqual(
            self.auto_metrics,
            _EXPECTED_MAIN_TIER | _EXPECTED_PR_TIER,
            "bench/perf-baseline.json mode:auto metrics changed. Update the expected "
            "sets in this test. See the docstring for the required update procedure.",
        )
        # 2. Guard detection must agree with the expected classification for each metric.
        ranges = self._guard_ranges()
        self.assertTrue(ranges, "no GITHUB_REF guard blocks found in ci-bench.sh — "
                        "guard detection is broken (check _GUARD_START_RE)")
        for metric in sorted(_EXPECTED_MAIN_TIER):
            self.assertTrue(
                self._is_main_tier_only(metric, ranges),
                f"{metric!r} is expected to be main-tier-only, but ci-bench.sh has no "
                f"GITHUB_REF main-tier guard enclosing its `add` call. Either add the "
                f"guard or move it to _EXPECTED_PR_TIER and add a bench seed.",
            )
        for metric in sorted(_EXPECTED_PR_TIER):
            self.assertFalse(
                self._is_main_tier_only(metric, ranges),
                f"{metric!r} is expected to be PR-tier, but seems to be behind a "
                f"GITHUB_REF guard. Move it to _EXPECTED_MAIN_TIER.",
            )
        # 3. Bench seed set must be non-empty (sanity guard for the invariant).
        self.assertGreater(len(self.lane_seeds["bench"]), 0,
                           "_LANE_SEEDS['bench'] is empty — no PR-tier gate exists")


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

    # [FABLE-5] Draft-tier CI: the exact marker expression ci-select.yml also
    # uses — empty on every non-draft payload, ", draft-tier" on a draft one.
    GATE_MARKER_EXPR = "${{ github.event.pull_request.draft == true && ', draft-tier' || '' }}"

    def test_gate_job_name_is_exactly_gate(self):
        """The branch-protection ruleset (id 17688455) requires context='gate'.
        If this name drifts the required check silently stops matching and the
        gate weakens with no error anywhere — so pin it.

        [FABLE-5] Draft-tier CI (GATE INTEGRITY): the name is now TIERED — the
        job name is the literal 'gate' plus the draft-tier marker expression, so
        * every non-draft payload (non-draft PR, merge_group, push) still
          renders EXACTLY 'gate' (the required context, unchanged), and
        * a draft pull_request payload renders 'gate, draft-tier' — a
          deliberately NON-required context, so a draft-tier run can never
          satisfy branch protection in ANY window (event latency, a dropped
          ready_for_review event, an Actions outage: the required context is
          simply absent until the full-tier run concludes)."""
        job = self.cs["jobs"]["gate"]
        name = job["name"]
        self.assertEqual(
            name, "gate" + self.GATE_MARKER_EXPR,
            "ci-summary gate job name must be exactly 'gate' + the draft-tier "
            "marker expression — the ruleset anchors on context='gate' (id "
            "17688455) for every non-draft payload, and the marker is what "
            "keeps a draft-tier run from ever emitting that required context",
        )
        # The non-draft rendering is the required context, byte-exact.
        self.assertEqual(name.replace(self.GATE_MARKER_EXPR, ""), "gate")
        # The draft rendering is the gate module's artifact name — the script
        # excludes it from sibling sets and normalizes it back to 'gate'.
        draft_rendered = name.replace(self.GATE_MARKER_EXPR,
                                      self.gate.DRAFT_TIER_MARKER)
        self.assertEqual(draft_rendered, self.gate.DRAFT_TIER_GATE_NAME)
        self.assertTrue(self.gate.is_draft_gate_artifact(draft_rendered))
        self.assertFalse(self.gate.is_draft_gate_artifact("gate"))
        self.assertTrue(self.gate.is_draft_tier(draft_rendered))
        self.assertEqual(self.gate.normalized_name(draft_rendered), "gate")

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
        Both the bare job name and the `workflow / job` display form are checked,
        in both tier renderings."""
        for name in ("gate", "ci-summary / gate",
                     "gate, draft-tier", "ci-summary / gate, draft-tier"):
            self.assertFalse(
                self.gate.is_advisory(name),
                f"gate check name {name!r} must NOT match the advisory exclusion",
            )

    def test_gate_is_not_detected_as_a_select_job(self):
        """The gate name must not match SELECT_RE — that would make the gate
        self-detect as a selection pre-job and add a circular verdict dependency
        (skipped gating only valid if 'gate' itself concluded success)."""
        for name in ("gate", "ci-summary / gate",
                     "gate, draft-tier", "ci-summary / gate, draft-tier"):
            self.assertFalse(
                self.gate.is_select(name),
                f"gate check name {name!r} must NOT match SELECT_RE",
            )


class TestHeavyRecallMergeGroupDemotion(unittest.TestCase):
    """[OPUS-4.8] sq-6vshe.6 (research/ci-structural-speedup.md §7, round 2): the two
    heavy sparq-vectors recall shards (heavy-diskann/heavy-hnsw) are DEMOTED off the
    merge_group ref — deterministic single-crate accuracy gates with no cross-PR
    interaction — while the bulk workspace-unification shards (the real cross-PR value)
    stay. Pins the guard SHAPE + the demoted-lane safety-net job so the demotion cannot
    silently rot or over-reach.

    Key safety invariants pinned here:
      * the demotion is merge_group-ONLY (never PR, never push-to-main — the full form);
      * it is a STEP guard, not a job `if:` (matrix context is unavailable there);
      * the check-run NAMES are unchanged (guard reports success, not a missing leg);
      * a heavy-shard failure on push-to-main auto-files via the demoted-lane filer;
      * the filer's job-name detection regex matches the real heavy shard names."""

    # The regex the heavy-recall-demoted-filer uses (in ci.yml) to find a FAILED heavy
    # shard by job name. Kept in sync with the workflow's `--jq ... test("...")`.
    _HEAVY_JOB_RE = re.compile(r"test \(load-aware shard heavy-")

    @classmethod
    def setUpClass(cls):
        cls.ci = _load(CI_YML)

    def _test_run_step(self) -> str:
        """The `Test (nextest, shard …)` step's run body."""
        for step in self.ci["jobs"]["test"]["steps"]:
            if str(step.get("name", "")).startswith("Test (nextest, shard"):
                return str(step.get("run", ""))
        self.fail("could not find the `Test (nextest, shard …)` step in the test job")

    def test_heavy_shards_demoted_off_merge_group_only(self):
        run = self._test_run_step()
        # The guard: merge_group AND a non-empty SHARD_FILTER (heavy shards only) -> exit 0.
        self.assertIn('"${{ github.event_name }}" = "merge_group"', run,
                      "heavy-recall demotion guard must key on the merge_group event")
        self.assertIn('-n "$SHARD_FILTER"', run,
                      "demotion guard must fire only on heavy shards (non-empty filter)")
        # It must be an AND so it never fires on a bulk shard or a non-merge_group event.
        self.assertRegex(
            run,
            r'"\$\{\{ github\.event_name \}\}" = "merge_group" \] && \[ -n "\$SHARD_FILTER"',
            "demotion must require BOTH merge_group AND a heavy shard (AND, not OR) — "
            "otherwise it would skip bulk shards or run on PR/push",
        )
        # The guard must NOT reference push / pull_request — the full form runs there.
        # (Assert the guard line itself mentions merge_group exclusively.)
        guard_lines = [ln for ln in run.splitlines()
                       if "github.event_name" in ln and "SHARD_FILTER" in ln]
        self.assertTrue(guard_lines, "expected a single-line combined event+shard guard")
        for ln in guard_lines:
            self.assertNotIn("push", ln,
                             "the demotion guard must not fire on push-to-main (full form)")
            self.assertNotIn("pull_request", ln,
                             "the demotion guard must not fire on PRs (full form, blocking)")

    def test_shard_filter_env_maps_to_matrix_filter(self):
        """The guard reads $SHARD_FILTER; it must be populated from matrix.filter (the
        heavy-shard discriminator — non-empty only on the two heavy shards)."""
        for step in self.ci["jobs"]["test"]["steps"]:
            if str(step.get("name", "")).startswith("Test (nextest, shard"):
                env = step.get("env", {})
                self.assertIn("SHARD_FILTER", env,
                              "the Test step must pass SHARD_FILTER via env")
                self.assertEqual(env["SHARD_FILTER"], "${{ matrix.filter }}",
                                 "SHARD_FILTER must come from matrix.filter")
                return
        self.fail("Test step not found")

    def test_heavy_shards_still_run_at_pr_and_push(self):
        """The demotion must NOT remove the heavy shards from the matrix — they must
        still run (full form) at PR level and on push-to-main. Confirm the matrix
        entries are untouched and the job triggers on those events."""
        matrix = self.ci["jobs"]["test"]["strategy"]["matrix"]["include"]
        heavy = [e for e in matrix if str(e.get("name", "")).startswith("heavy-")]
        self.assertEqual(
            {e["name"] for e in heavy}, {"heavy-diskann", "heavy-hnsw"},
            "both heavy recall shards must remain in the test matrix (demoted at "
            "runtime on merge_group only, never removed)",
        )
        on = _on_block(self.ci)
        self.assertIn("pull_request", on, "CI must run on pull_request (heavy full form)")
        self.assertIn("push", on, "CI must run on push (heavy full form on main)")

    def test_demoted_lane_filer_job_wired(self):
        """The demotion protocol (§7) requires a full-form-failure auto-filer so the
        demoted lane cannot silently rot. Pin the safety-net job's existence, its
        push-to-main-only guard, its scoped write perms, and its use of the filer."""
        job = self.ci["jobs"].get("heavy-recall-demoted-filer")
        self.assertIsNotNone(job, "missing the heavy-recall demoted-lane safety-net job")
        cond = str(job.get("if", ""))
        self.assertIn("github.event_name == 'push'", cond,
                      "safety-net must run on push-to-main only")
        self.assertIn("refs/heads/main", cond, "safety-net must be main-only")
        self.assertIn("needs.test.result", cond,
                      "safety-net must gate on the test job's result")
        needs = job.get("needs", [])
        needs = [needs] if isinstance(needs, str) else needs
        self.assertIn("test", needs, "safety-net must need the test job")
        perms = job.get("permissions", {})
        self.assertEqual(perms.get("contents"), "write",
                         "safety-net needs contents:write to append+push the bead")
        self.assertEqual(perms.get("issues"), "write",
                         "safety-net needs issues:write to file the GitHub issue")
        self.assertEqual(perms.get("actions"), "read",
                         "safety-net needs actions:read to inspect this run's jobs")
        body = "\n".join(str(s.get("run", "")) for s in job.get("steps", []))
        self.assertIn("scripts/ci-file-demoted-lane-failure.py", body,
                      "safety-net must invoke the generic demoted-lane filer")
        self.assertIn("--self-test", body,
                      "safety-net must self-test the filer before using it")

    def test_filer_job_name_detection_matches_real_heavy_shard_names(self):
        """The safety-net detects a failed heavy shard by matching its JOB NAME against
        `test \\(load-aware shard heavy-`. If a heavy shard is renamed and this regex is
        not, the filer would silently never fire (silent rot). Pin the regex against the
        REAL job names GitHub produces (workflow-job name template + matrix.name)."""
        test_job_name_tmpl = self.ci["jobs"]["test"]["name"]  # "test (load-aware shard ${{ matrix.name }})"
        matrix = self.ci["jobs"]["test"]["strategy"]["matrix"]["include"]
        heavy_names = [e["name"] for e in matrix if str(e.get("name", "")).startswith("heavy-")]
        for mname in heavy_names:
            real = test_job_name_tmpl.replace("${{ matrix.name }}", mname)
            self.assertRegex(
                real, self._HEAVY_JOB_RE,
                f"the filer's heavy-shard detection regex must match job name {real!r}",
            )
        # And it must NOT match a bulk shard or the nightly-coverage 'heavy' substring
        # job (which carries "heavy" but is not a heavy recall shard).
        self.assertNotRegex(
            test_job_name_tmpl.replace("${{ matrix.name }}", "bulk 1/3"),
            self._HEAVY_JOB_RE,
            "the filer regex must not match bulk shards",
        )
        self.assertNotRegex(
            "coverage (nightly, full incl. heavy vectors)",
            self._HEAVY_JOB_RE,
            "the filer regex must not match the nightly-coverage 'heavy' substring job",
        )


class TestDraftTierWiring(unittest.TestCase):
    """[FABLE-5] Draft-tier CI (docs/branch-protection.md §Draft-tier CI): draft
    PR heads run a REDUCED matrix — coverage / bench / CodeQL / heavy shards
    skipped, the wasm-equality leg kept only when the sparq-wasm closure is
    affected — and the un-draft moment (ready_for_review) re-runs everything at
    FULL tier so a fresh gate supersedes the draft-tier results. Pins:
      * the ci-select job-name draft-tier MARKER (the gate's tier detection
        contract — scripts/ci_summary_gate.py DRAFT_TIER_MARKER);
      * every draft skip guard's fail-open shape (`github.event_name !=
        'pull_request' || ... draft != true`: any non-PR event RUNS — push /
        merge_group / schedule semantics are byte-identical);
      * ready_for_review in the pull_request types of every gate-feeding
        workflow (without it the un-draft moment runs NOTHING and the head keeps
        its draft-tier results — the exact stale-green the invariant forbids);
      * the gate job's tier env plumbing (EVENT_NAME / PR_DRAFT / PR_NUMBER +
        pull-requests: read)."""

    # The fail-open draft guard: any non-pull_request event satisfies the first
    # disjunct => RUN; only a genuinely-draft PR head can skip.
    DRAFT_GUARD = "github.event_name != 'pull_request' || github.event.pull_request.draft != true"
    MARKER_EXPR = "${{ github.event.pull_request.draft == true && ', draft-tier' || '' }}"

    # Every gate-feeding workflow with a pull_request trigger. Advisory-only
    # workflows (pr-title, deploy-lint, deploy-terraform-lint) are deliberately
    # absent — the gate excludes their checks by name, so they neither gate nor
    # feed it. bead-autoclose/flow-on react to `closed` only (not gate-feeding).
    READY_FOR_REVIEW_WFS = [
        "ci.yml", "feature-matrix.yml", "bench.yml", "fuzz.yml", "codeql.yml",
        "ci-summary.yml", "vectorized-feature-off.yml", "docs-quality.yml",
        "supply-chain.yml", "flow-on-gates.yml", "formal-verification.yml",
        "js.yml", "gui.yml", "python.yml", "site-e2e.yml",
        "site-e2e-foundation.yml", "site-e2e-hero.yml", "site-visual.yml",
        "container-scan.yml", "zk-toolchain.yml",
    ]

    @classmethod
    def setUpClass(cls):
        wfdir = REPO_ROOT / ".github" / "workflows"
        cls.ci = _load(CI_YML)
        cls.bench = _load(BENCH_YML)
        cls.sel = _load(SELECT_YML)
        cls.codeql = _load(wfdir / "codeql.yml")
        cls.vfo = _load(wfdir / "vectorized-feature-off.yml")
        cls.summary = _load(wfdir / "ci-summary.yml")
        cls.js = _load(wfdir / "js.yml")
        cls.wfdir = wfdir
        cls.gate = _gate_module()
        cls.members = _workspace_members()

    # ---- the tier marker (the gate's detection contract) ---------------------
    def test_select_job_name_carries_the_draft_tier_marker(self):
        name = self.sel["jobs"]["select"]["name"]
        self.assertIn(self.MARKER_EXPR, name,
                      "ci-select's job name must append the draft-tier marker on "
                      "draft PR payloads (the gate's tier detection contract)")
        marker = self.gate.DRAFT_TIER_MARKER
        self.assertIn(marker, self.MARKER_EXPR,
                      "the YAML marker literal must be the gate's DRAFT_TIER_MARKER")
        # Both RENDERED spellings must satisfy the SELECT_RE contract, carry no
        # advisory token, and normalize to the same identity.
        full = name.replace(self.MARKER_EXPR, "")
        draft = name.replace(self.MARKER_EXPR, marker)
        for rendered in (full, draft, f"select / {full}", f"select / {draft}"):
            self.assertTrue(self.gate.is_select(rendered), rendered)
            self.assertFalse(self.gate.is_advisory(rendered), rendered)
        self.assertTrue(self.gate.is_draft_tier(draft))
        self.assertFalse(self.gate.is_draft_tier(full))
        self.assertEqual(self.gate.normalized_name(draft), full)

    # ---- draft skip guards ---------------------------------------------------
    def test_coverage_jobs_skip_on_draft_heads(self):
        for job_id in ("coverage-measure", "coverage-engine-run", "coverage"):
            cond = str(self.ci["jobs"][job_id].get("if", ""))
            self.assertIn(self.DRAFT_GUARD, cond,
                          f"ci.yml:{job_id} must skip on draft PR heads with the "
                          f"fail-open guard (draft-tier CI)")
        # The final aggregate keeps always() (it must still conclude when a
        # needed job failed on non-draft runs).
        self.assertIn("always()", str(self.ci["jobs"]["coverage"].get("if", "")))

    def test_bench_job_skips_on_draft_heads(self):
        cond = str(self.bench["jobs"]["bench"].get("if", ""))
        self.assertIn(self.DRAFT_GUARD, cond,
                      "bench.yml:bench must skip entirely on draft PR heads")
        self.assertIn(FAIL_CLOSED_DISJUNCT, cond,
                      "the selection disjunction must survive the draft guard")

    def test_codeql_analyze_skips_on_draft_heads(self):
        cond = str(self.codeql["jobs"]["analyze"].get("if", ""))
        self.assertIn(self.DRAFT_GUARD, cond,
                      "codeql.yml:analyze must skip on draft PR heads")
        on = _on_block(self.codeql)
        # [FABLE-5] (2026-07-18 merge-queue subset): merge_group removed — PR head +
        # push-to-main + weekly schedule carry the analysis coverage.
        self.assertNotIn("merge_group", on,
                         "CodeQL must NOT run on merge_group (2026-07-18 directive)")
        self.assertIn("schedule", on, "CodeQL must keep its weekly schedule")

    def test_heavy_shards_also_demoted_on_draft_heads(self):
        for step in self.ci["jobs"]["test"]["steps"]:
            if str(step.get("name", "")).startswith("Test (nextest, shard"):
                env = step.get("env", {})
                self.assertIn("IS_DRAFT_PR", env,
                              "the Test step must receive IS_DRAFT_PR via env")
                self.assertIn("github.event.pull_request.draft == true",
                              str(env["IS_DRAFT_PR"]))
                run = str(step.get("run", ""))
                self.assertIn('[ "$IS_DRAFT_PR" = "true" ] && [ -n "$SHARD_FILTER" ]',
                              run,
                              "draft demotion must require BOTH a draft head AND a "
                              "heavy shard (bulk shards are never demoted)")
                # The draft guard must be a SEPARATE guard from the merge_group
                # one (that one is pinned merge_group-exclusive above).
                draft_lines = [ln for ln in run.splitlines() if "IS_DRAFT_PR" in ln]
                for ln in draft_lines:
                    self.assertNotIn("merge_group", ln)
                return
        self.fail("Test step not found")

    def test_vfo_draft_scope_keeps_leg_iff_wasm_closure_affected(self):
        """artifact-exact-equality on a DRAFT head consults the SAME selector
        (scripts/ci_select.py) in-step and skips only on a definite
        mode=selected + sparq-wasm-not-affected verdict — every other path
        (non-draft, ci-full label, full/shadow/error mode, selector failure)
        leaves the paths-filter verdict (RUN, fail-closed)."""
        self.assertIn("sparq-wasm", self.members)
        steps = None
        for job_id, job in self.vfo["jobs"].items():
            for s in job.get("steps", []):
                if s.get("id") == "changes" and "ci_select.py" in str(s.get("run", "")):
                    steps = s
        self.assertIsNotNone(steps, "vfo decide step must invoke scripts/ci_select.py")
        run = str(steps["run"])
        env = steps.get("env", {})
        for key in ("IS_DRAFT_PR", "CI_FULL_LABEL", "BASE_SHA", "HEAD_SHA"):
            self.assertIn(key, env, f"vfo decide step must receive {key}")
        self.assertIn('[ "$IS_DRAFT_PR" = "true" ]', run)
        self.assertIn('[ "$CI_FULL_LABEL" != "true" ]', run,
                      "the ci-full label must force the full leg on drafts too")
        self.assertIn('[ "$mode" = "selected" ]', run,
                      "only a definite mode=selected verdict may skip (fail-closed)")
        self.assertIn('"sparq-wasm"', run,
                      "the needle must be the quoted sparq-wasm member name")
        # The skip assignment must exist exactly once, inside the guarded branch.
        self.assertEqual(run.count('rust_changed="false"'), 1)

    # ---- ready_for_review coverage -------------------------------------------
    def test_ready_for_review_wired_on_every_gate_feeding_workflow(self):
        for wf_name in self.READY_FOR_REVIEW_WFS:
            wf = _load(self.wfdir / wf_name)
            pr = _on_block(wf).get("pull_request") or {}
            self.assertIsInstance(pr, dict,
                                  f"{wf_name}: pull_request must enumerate types "
                                  f"(bare form lacks ready_for_review)")
            types = pr.get("types", [])
            self.assertIn("ready_for_review", types,
                          f"{wf_name}: without ready_for_review the un-draft moment "
                          f"runs nothing and the head keeps draft-tier results")
            for keep in ("opened", "synchronize", "reopened"):
                self.assertIn(keep, types, f"{wf_name}: must keep the default {keep}")

    # ---- gate plumbing -------------------------------------------------------
    def test_gate_check_name_is_tiered_on_draft_payloads(self):
        """[GATE INTEGRITY] The STRUCTURAL half of the invariant: on a draft
        payload the ci-summary gate job's own check-run is named
        `gate, draft-tier`, so a draft-tier run never produces the required
        `gate` context and branch protection cannot be satisfied by it in any
        window (event latency / dropped ready_for_review / Actions outage —
        the required context is simply ABSENT until the full-tier run
        concludes). Full rendering pinned by TestRequiredCheckAnchor."""
        name = self.summary["jobs"]["gate"]["name"]
        self.assertEqual(name, "gate" + self.MARKER_EXPR)

    def test_code_scanning_backstop_fed_and_documented_as_non_load_bearing(self):
        """The live ruleset also carries a `code_scanning` rule (CodeQL). A
        draft-built head carries no PR CodeQL analysis (analyze skips on
        drafts), so that rule independently blocks such a head — but it is an
        out-of-repo, owner-mutable setting and evadable (a non-draft PR sharing
        the head SHA supplies an analysis; outage relaxation), so it must be
        recorded as defense-in-depth ONLY and the feeding triggers must not
        rot: push-to-main + weekly schedule + ready_for_review (merge_group was
        removed by the 2026-07-18 merge-queue-subset directive)."""
        on = _on_block(self.codeql)
        push = on.get("push") or {}
        self.assertEqual(push.get("branches"), ["main"],
                         "codeql.yml must keep its push-to-main analysis run")
        self.assertIn("schedule", on)
        types = (on.get("pull_request") or {}).get("types", [])
        self.assertIn("ready_for_review", types,
                      "the un-draft moment must produce a fresh CodeQL analysis")
        doc = (REPO_ROOT / "docs" / "branch-protection.md").read_text(encoding="utf-8")
        self.assertIn("code_scanning", doc,
                      "docs/branch-protection.md must record the code_scanning "
                      "rule's role in the draft-tier design")
        self.assertIn("defense-in-depth only", doc,
                      "the doc must record code_scanning as defense-in-depth, "
                      "never the load-bearing draft-tier mechanism")
        self.assertIn("owner-mutable", doc)

    def test_gate_receives_tier_env_and_pr_read(self):
        perms = self.summary.get("permissions", {})
        self.assertEqual(perms.get("pull-requests"), "read",
                         "the gate needs pull-requests:read for the conclusion-time "
                         "draft re-check")
        step = next(s for s in self.summary["jobs"]["gate"]["steps"]
                    if "ci_summary_gate.py" in str(s.get("run", "")))
        env = step.get("env", {})
        for key in ("EVENT_NAME", "PR_DRAFT", "PR_NUMBER"):
            self.assertIn(key, env, f"gate step must export {key}")
        self.assertIn("github.event.pull_request.draft", str(env["PR_DRAFT"]))

    def test_bench_concurrency_cancels_only_pull_request(self):
        conc = self.bench.get("concurrency", {})
        self.assertEqual(str(conc.get("cancel-in-progress")),
                         "${{ github.event_name == 'pull_request' }}",
                         "bench must cancel superseded PR runs but never a "
                         "push/schedule run (history integrity)")

    def test_js_has_per_pr_concurrency(self):
        conc = self.js.get("concurrency", {})
        self.assertIn("github.event.pull_request.number", str(conc.get("group", "")))
        self.assertTrue(conc.get("cancel-in-progress"),
                        "js.yml must cancel superseded PR runs")


class TestContainerScanMergeGroupGate(unittest.TestCase):
    """[OPUS-4.8] path-aware CI audit: container-scan.yml narrows its heavy trivy
    image build+scan to container-relevant PRs via native `on.pull_request.paths`,
    but GitHub IGNORES `paths` for the merge_group event, so a bare `merge_group:`
    trigger rebuilt the full fat-LTO image + ran trivy on EVERY enqueue. The `trivy`
    job now leads with a `detect container changes` step that diffs the queued batch
    and skips the build+scan on a container-inert merge group (mirrors zk-toolchain's
    proven detect pattern). Pins the SHAPE so the gate cannot silently rot or over-reach.
    [FABLE-5] (2026-07-18 merge-queue subset): the merge_group TRIGGER is now removed
    (first test below pins its absence); the detect machinery stays as the fail-safe
    short-circuit on every remaining event and as the cheap revert path.

    Invariants pinned:
      * the workflow does NOT trigger on merge_group (2026-07-18 directive);
      * the detect step is FAIL-SAFE (default container=true; only merge_group can flip false);
      * the heavy build/scan steps are STEP-gated on the detect output (not a job `if:`,
        so the check-run name is preserved and reports success on the skip path);
      * the merge_group path set MATCHES the pull_request paths filter (no drift)."""

    @classmethod
    def setUpClass(cls):
        cls.wf = _load(CONTAINER_SCAN_YML)
        cls.trivy = cls.wf["jobs"]["trivy"]
        cls.steps = cls.trivy["steps"]

    def _step(self, name_prefix: str) -> dict:
        for step in self.steps:
            if str(step.get("name", "")).startswith(name_prefix):
                return step
        self.fail(f"container-scan trivy job missing a step named {name_prefix!r}")

    def test_does_not_trigger_on_merge_group(self):
        # [FABLE-5] (2026-07-18 merge-queue subset): merge_group removed — coverage
        # lives at the paths-filtered PR head + push-to-main + the weekly re-scan.
        # The detect-step machinery below stays (short-circuits to a full scan on
        # every remaining event; cheap revert path).
        on = _on_block(self.wf)
        self.assertNotIn("merge_group", on,
                         "container-scan must NOT trigger on merge_group "
                         "(2026-07-18 merge-queue-subset directive)")
        self.assertIn("schedule", on, "the weekly re-scan backstop must stay")

    def test_detect_step_present_and_fail_safe(self):
        step = self._step("Detect container changes")
        run = str(step.get("run", ""))
        self.assertEqual(step.get("id"), "changes",
                         "detect step must expose id=changes for the downstream if:s")
        # Fail-safe default: container=true, and only a merge_group event can flip it false.
        self.assertRegex(run, r"container=true",
                         "detect step must default to the full scan (container=true)")
        self.assertIn('"${EVENT_NAME}" = "merge_group"', run,
                      "only the merge_group event may narrow the scan (fail-safe elsewhere)")
        # Any diff error must fall back to the full scan.
        self.assertIn("fail-safe", run.lower(),
                      "detect step must document its fail-safe fallback")

    def test_heavy_steps_gated_on_detect_output(self):
        # The build + both trivy scans + the SARIF upload must all be STEP-gated on the
        # detect output — never run on a container-inert merge group.
        for prefix in ("Build image", "Trivy image scan (fail",
                       "Trivy image scan (SARIF", "Upload Trivy SARIF"):
            step = self._step(prefix)
            cond = str(step.get("if", ""))
            self.assertIn("steps.changes.outputs.container == 'true'", cond,
                          f"the {prefix!r} step must be gated on the detect output")

    def test_gate_is_step_not_job_level(self):
        # A job-level `if:` would make the whole job skip and CHANGE the check-run
        # conclusion accounting on the merge_group ref; the gate must be at STEP level so
        # the `trivy (…)` check-run always appears and concludes SUCCESS on the skip path.
        self.assertNotIn("container", str(self.trivy.get("if", "")),
                         "the container gate must be a STEP guard, not a job-level if:")

    def test_merge_group_path_set_matches_pr_paths_filter(self):
        """The merge_group detect grep must cover the SAME path set as the
        pull_request `paths:` filter — otherwise the two tiers would disagree about
        what counts as container-relevant (drift = an unsound skip or a wasted run)."""
        on = _on_block(self.wf)
        pr_paths = set(on["pull_request"]["paths"])
        run = str(self._step("Detect container changes").get("run", ""))
        # Each PR path must be represented in the detect grep. Map the glob forms to the
        # anchored regex fragments the detect step uses.
        expected_fragments = {
            "Dockerfile": "Dockerfile$",
            ".dockerignore": r"\.dockerignore$",
            ".trivyignore": r"\.trivyignore$",
            ".hadolint.yaml": r"\.hadolint\.yaml$",
            ".github/workflows/container-scan.yml": r"container-scan\.yml$",
            "Cargo.toml": r"Cargo\.toml$",
            "Cargo.lock": r"Cargo\.lock$",
            "crates/**/Cargo.toml": r"crates/[^/]+/Cargo\.toml$",
        }
        for pr_path in pr_paths:
            self.assertIn(pr_path, expected_fragments,
                          f"PR path {pr_path!r} has no mapped detect-grep fragment — "
                          "update the merge_group detect grep to match (no drift)")
            self.assertIn(expected_fragments[pr_path], run,
                          f"the merge_group detect grep must cover PR path {pr_path!r}")


class TestSupplyChainMergeGroupGate(unittest.TestCase):
    """[OPUS-4.8] path-aware CI audit: supply-chain.yml gates its cargo-deny/vet
    step-groups on rust_changed, which was FORCED true on merge_group (paths don't
    apply there) — so the full deny+vet ran on every enqueue. The `Decide rust_changed`
    step now diffs the queued batch's base_sha..head_sha against the SAME `rust` path
    set the pull_request dorny filter uses; a rust-inert merge group skips the heavy
    deny+vet steps (SBOM stays always-on by the sq-6vshe.20 design). Pins the SHAPE.
    [FABLE-5] (2026-07-18 merge-queue subset): the merge_group TRIGGER is now removed
    (first test below pins its absence); the Decide-step merge_group branch stays as
    dead-but-fail-safe code and the cheap revert path.

    Invariants pinned:
      * does NOT trigger on merge_group (2026-07-18 directive);
      * the merge_group branch is FAIL-SAFE (rust=true on any diff error);
      * every path in the pull_request dorny `rust` filter is covered by the
        merge_group detect grep (no drift between the two tiers);
      * the deny/vet gating steps remain rust_changed-gated (unchanged)."""

    @classmethod
    def setUpClass(cls):
        cls.wf = _load(SUPPLY_CHAIN_YML)
        cls.job = cls.wf["jobs"]["supply-chain-gates"]
        cls.steps = cls.job["steps"]

    def _decide_run(self) -> str:
        for step in self.steps:
            if str(step.get("name", "")).startswith("Decide rust_changed"):
                return str(step.get("run", ""))
        self.fail("supply-chain missing the `Decide rust_changed` step")

    def _dorny_rust_filter(self) -> list[str]:
        for step in self.steps:
            if str(step.get("uses", "")).startswith("dorny/paths-filter"):
                filters = yaml.safe_load(step["with"]["filters"])
                return list(filters["rust"])
        self.fail("supply-chain missing the dorny rust paths-filter")

    def test_does_not_trigger_on_merge_group(self):
        # [FABLE-5] (2026-07-18 merge-queue subset): merge_group removed — coverage
        # lives at the PR head + push-to-main + dependency-monitoring's weekly cron.
        on = _on_block(self.wf)
        self.assertNotIn("merge_group", on,
                         "supply-chain must NOT trigger on merge_group "
                         "(2026-07-18 merge-queue-subset directive)")
        self.assertEqual((on.get("push") or {}).get("branches"), ["main"],
                         "the push-to-main post-merge run must stay")

    def test_merge_group_branch_is_fail_safe(self):
        run = self._decide_run()
        self.assertIn('"${EVENT_NAME}" = "merge_group"', run,
                      "Decide step must special-case merge_group with a batch diff")
        # Default rust=true inside the merge_group branch (fail-safe on any diff error).
        self.assertIn("rust=true", run,
                      "merge_group branch must default rust=true (fail-safe)")
        self.assertIn("fail-safe", run.lower(),
                      "merge_group branch must document + take its fail-safe fallback")

    def test_merge_group_grep_covers_every_pr_rust_path(self):
        run = self._decide_run()
        # Map each dorny glob to the regex fragment the merge_group grep must contain.
        glob_to_fragment = {
            "**/*.rs": r"\.rs$",
            "**/Cargo.toml": r"Cargo\.toml$",
            "Cargo.lock": r"^Cargo\.lock$",
            "rust-toolchain*": r"rust-toolchain",
            "deny.toml": r"^deny\.toml$",
            "supply-chain/**": r"^supply-chain/",
            ".github/workflows/supply-chain.yml": r"supply-chain\.yml$",
        }
        for glob in self._dorny_rust_filter():
            self.assertIn(glob, glob_to_fragment,
                          f"dorny rust glob {glob!r} has no mapped merge_group grep fragment "
                          "— update the Decide-step grep to match (no tier drift)")
            self.assertIn(glob_to_fragment[glob], run,
                          f"merge_group grep must cover the dorny rust glob {glob!r}")

    def test_deny_and_vet_steps_remain_rust_gated(self):
        gated = [str(s.get("name", "")) for s in self.steps
                 if "rust_changed == 'true'" in str(s.get("if", ""))]
        self.assertTrue(any("cargo-deny check (bans" in n for n in gated),
                        "cargo-deny bans/sources/licenses must stay rust_changed-gated")
        self.assertTrue(any("cargo-vet check" in n for n in gated),
                        "cargo-vet must stay rust_changed-gated")


class TestMergeGroupChangeClassGate(unittest.TestCase):
    """[FABLE-5] merge-group change-class gate (extends #3420/#3421 to the
    rust_changed layer): the ci.yml + feature-matrix.yml `changes` decide steps
    classify the queued batch's diff via `scripts/ci_select.py --classify-only`
    instead of hard-forcing rust_changed=true on merge_group, so a docs-only/
    orchestration-only batch skips the rust_changed-only lanes (lint / msrv /
    geiger / docker-smoke / coverage-floors; feature-matrix setup / check-tier /
    fedclient-boundary) with ATTRIBUTED skips. Pins the SHAPE:
      * the merge_group branch exists and is FAIL-SAFE (defaults true, the #3421
        fetch guard, `|| cls=engine` on the classifier invocation);
      * classification is DELEGATED to scripts/ci_select.py (single source of
        truth) — no duplicated grep path list in the step;
      * the skip-class set is EXACTLY {docs-only, orchestration-only}, spelled
        with the classifier module's own tokens (engine/mixed/unknown => full);
      * ci.yml's docker_changed is class-gated the same way;
      * fuzz.yml needs no such layer (its heavy jobs are select-gated and
        ci-select passes the merge_group SHA pair) and bench.yml has no
        merge_group trigger at all (pinned elsewhere) — this layer must NOT
        creep into them as a redundant/conflicting second gate."""

    @classmethod
    def setUpClass(cls):
        cls.ci = _load(CI_YML)
        cls.fm = _load(FM_YML)
        cls.fuzz = _load(FUZZ_YML)
        cls.select_mod = _ci_select_module()

    def _decide(self, wf, wf_name):
        for step in wf["jobs"]["changes"]["steps"]:
            if str(step.get("name", "")).startswith("Decide rust_changed"):
                return step
        self.fail(f"{wf_name} missing the `Decide rust_changed` step")

    def _both(self):
        return (("ci.yml", self._decide(self.ci, "ci.yml")),
                ("feature-matrix.yml", self._decide(self.fm, "feature-matrix.yml")))

    def test_merge_group_branch_present_and_fail_safe(self):
        for wf_name, step in self._both():
            run = str(step.get("run", ""))
            self.assertIn('"${EVENT_NAME}" = "merge_group"', run,
                          f"{wf_name}: decide step must special-case merge_group")
            self.assertIn("rust=true", run,
                          f"{wf_name}: merge_group branch must default rust=true (fail-safe)")
            self.assertIn("|| cls=engine", run,
                          f"{wf_name}: a classifier invocation failure must fall back to "
                          "cls=engine (fail-safe => full run)")
            self.assertIn("git cat-file -e", run,
                          f"{wf_name}: the #3421 SHA-resolution guard must precede the diff")
            self.assertIn("fail-safe", run.lower(),
                          f"{wf_name}: the fail-safe fallback must be documented + taken")
            # The event payload SHA pair must feed the step (the authoritative diff).
            env = step.get("env", {}) or {}
            self.assertEqual(str(env.get("MG_BASE_SHA", "")),
                             "${{ github.event.merge_group.base_sha }}", wf_name)
            self.assertEqual(str(env.get("MG_HEAD_SHA", "")),
                             "${{ github.event.merge_group.head_sha }}", wf_name)

    def test_batch_diff_fetch_deepens_the_shallow_checkout(self):
        # The `changes` job checks out at the default depth 1. A plain SHA fetch
        # leaves base_sha/head_sha PRESENT but unconnected (both cat-file guards
        # pass), and the classifier's three-dot diff then dies with "no merge
        # base" — the fail-safe would force class=engine on EVERY batch,
        # silently reducing the whole gate to a no-op. Every fetch in the
        # merge_group branch must therefore deepen to full history
        # (--depth=2147483647 == --unshallow, but valid on complete repos too —
        # the same merge-base rationale as ci-select.yml's fetch-depth: 0).
        for wf_name, step in self._both():
            run = str(step.get("run", ""))
            fetches = [ln for ln in run.splitlines() if "git fetch" in ln]
            self.assertTrue(fetches, f"{wf_name}: merge_group branch must fetch the SHA pair")
            for ln in fetches:
                self.assertIn(
                    "--depth=2147483647", ln,
                    f"{wf_name}: every batch-diff fetch must deepen the shallow "
                    f"checkout or the three-dot diff has no merge base "
                    f"(permanent fail-safe => the gate never skips): {ln.strip()!r}")

    def test_classification_is_delegated_not_duplicated(self):
        for wf_name, step in self._both():
            run = str(step.get("run", ""))
            self.assertIn("scripts/ci_select.py --classify-only", run,
                          f"{wf_name}: the merge_group branch must invoke the classifier "
                          "(scripts/ci_select.py --classify-only), the single source of truth")
            self.assertNotIn("grep -Eq", run,
                             f"{wf_name}: the merge_group branch must NOT re-encode the "
                             "class path sets as a grep — no duplicated path lists")

    def test_skip_class_set_is_exactly_docs_and_orchestration(self):
        # The case-arm must skip on EXACTLY the two proven-inert classes, spelled
        # with the classifier module's own tokens; the wildcard arm must force the
        # full run (engine/mixed/any unknown token => rust=true).
        docs = self.select_mod._CLASS_DOCS
        orch = self.select_mod._CLASS_ORCHESTRATION
        self.assertEqual((docs, orch), ("docs-only", "orchestration-only"),
                         "classifier tokens drifted — update the workflow case-arms in "
                         "lock-step (they match on these literal strings)")
        for wf_name, step in self._both():
            run = str(step.get("run", ""))
            self.assertIn(f"{docs}|{orch}) rust=false", run,
                          f"{wf_name}: the skip case-arm must cover exactly {docs}|{orch}")
            self.assertIn("*) rust=true", run,
                          f"{wf_name}: the wildcard arm must force the full run")
            self.assertNotIn("mixed) rust=false", run, wf_name)
            self.assertNotIn("engine) rust=false", run, wf_name)

    def test_ci_docker_changed_is_class_gated_with_rust(self):
        run = str(self._decide(self.ci, "ci.yml").get("run", ""))
        self.assertIn("rust=false; docker=false", run,
                      "ci.yml: docker_changed must be class-gated alongside rust_changed "
                      "(every docker-filter path classifies engine, so the skip is sound)")

    def test_classify_only_flag_exists_and_fails_safe_in_the_selector(self):
        # The wired flag must exist in the selector CLI and carry the documented
        # fail-safe (this is the cross-file contract the case-arm relies on).
        text = CI_SELECT_PY.read_text(encoding="utf-8")
        self.assertIn("--classify-only", text)
        self.assertTrue(hasattr(self.select_mod, "_classify_only_main"),
                        "ci_select.py must expose the classify-only entry point")

    def test_fuzz_has_no_redundant_rust_changed_layer(self):
        # fuzz.yml's merge-group class gating IS the select pre-job (batch-diff
        # selection, seed-closure guards) — adding a second rust_changed layer
        # there would be redundant and could only disagree on transient errors.
        self.assertNotIn("changes", self.fuzz["jobs"],
                         "fuzz.yml grew a `changes` job — its merge-group gating is the "
                         "select pre-job; keep one gate, not two")


if __name__ == "__main__":
    unittest.main(verbosity=2)
