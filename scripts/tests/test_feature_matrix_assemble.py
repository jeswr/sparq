#!/usr/bin/env python3
# [OPUS-4.8] Hermetic tests for the dynamic feature-matrix assembler (bead sq-ibrze).
# Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns).
#
# GATE-CRITICAL invariant under test: the per-crate fragment files in
# `.github/feature-matrix.d/*.yml`, assembled by scripts/assemble-feature-matrix.py,
# emit the EXACT set of `opt-in <name>` check-run names that the single required
# `ci-summary / gate` aggregator + branch protection discover as REQUIRED. If the
# split into fragments ever renamed/dropped/added a gating leg, branch protection
# would stop recognising a required check (letting an unverified PR merge, or blocking
# the repo on an "expected but missing" check). This test locks that set against the
# byte-for-byte pre-split golden snapshot.
#
# Hermetic: imports the assembler module + reads the committed fragments + golden file
# on disk (committed files, not git/network state), so the run is deterministic. NO
# network; the only subprocess is a local `bash` run of the workflow's failure-
# attribution script (TestAssembleFailureAnnotation, sq-2wo5t) — still offline.
#
# Run:  python3 scripts/tests/test_feature_matrix_assemble.py
# (stdlib + the same PyYAML the assembler needs; no pytest required — also
#  discoverable by `pytest`.)

from __future__ import annotations

import importlib.util
import io
import json
import re
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
ASSEMBLER = REPO_ROOT / "scripts" / "assemble-feature-matrix.py"
GOLDEN = Path(__file__).resolve().parent / "feature-matrix-legnames.golden.txt"
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "feature-matrix.yml"
FRAGMENT_DIR = REPO_ROOT / ".github" / "feature-matrix.d"

# The exact exclusion rule ci-summary.yml applies (\b(advisory|informational)\b,
# case-insensitive) — a leg NAME matching this would silently STOP gating.
ADVISORY_RE = re.compile(r"\b(advisory|informational)\b", re.IGNORECASE)


def _load_assembler():
    spec = importlib.util.spec_from_file_location("assemble_feature_matrix", ASSEMBLER)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _golden_names():
    names = []
    for line in GOLDEN.read_text(encoding="utf-8").splitlines():
        s = line.strip()
        if not s or s.startswith("#"):
            continue
        names.append(s)
    return names


class TestFeatureMatrixAssemble(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = _load_assembler()
        cls.legs = cls.mod.load_legs()
        cls.names = sorted(f"opt-in {leg['name']}" for leg in cls.legs)
        cls.golden = _golden_names()

    # ---- THE gate-critical proof -------------------------------------------------
    def test_leg_names_match_golden_exactly(self):
        """Assembled `opt-in <name>` set == the pre-split golden set, byte-for-byte."""
        # `assertEqual` on sorted lists gives a precise added/removed diff on failure.
        self.assertEqual(
            self.names,
            sorted(self.golden),
            "feature-matrix leg-name set drifted from the gate-critical golden. If you "
            "added a NEW opt-in leg, update scripts/tests/feature-matrix-legnames.golden.txt "
            "DELIBERATELY (and keep the new name free of advisory/informational). If you "
            "did NOT intend to change the gating set, this is a real regression.",
        )

    def test_no_duplicate_leg_names(self):
        names = [leg["name"] for leg in self.legs]
        dupes = sorted({n for n in names if names.count(n) > 1})
        self.assertEqual(dupes, [], f"duplicate leg names collapse gating checks: {dupes}")

    def test_no_leg_name_is_advisory_or_informational(self):
        """Every leg must GATE — none may match ci-summary's advisory exclusion."""
        offenders = [n for n in self.names if ADVISORY_RE.search(n)]
        self.assertEqual(
            offenders,
            [],
            "these leg names contain the whole word advisory/informational and would "
            f"silently STOP gating in ci-summary: {offenders}",
        )

    # ---- assembler output contract ----------------------------------------------
    def test_assembled_json_is_fromjson_ready(self):
        """The default output is a single JSON object {"include": [legs]}."""
        import io
        from contextlib import redirect_stdout

        buf = io.StringIO()
        argv = sys.argv
        sys.argv = ["assemble-feature-matrix.py"]
        try:
            with redirect_stdout(buf):
                self.mod.main()
        finally:
            sys.argv = argv
        obj = json.loads(buf.getvalue())
        self.assertIn("include", obj)
        self.assertIsInstance(obj["include"], list)
        self.assertEqual(len(obj["include"]), len(self.legs))
        for leg in obj["include"]:
            self.assertEqual(set(leg.keys()), {"name", "crate", "features", "test"})
            self.assertTrue(leg["features"], "features must be non-empty (cargo rejects bare --features)")
            self.assertIsInstance(leg["test"], bool)

    def test_names_mode_matches_default_mode(self):
        import io
        from contextlib import redirect_stdout

        buf = io.StringIO()
        argv = sys.argv
        sys.argv = ["assemble-feature-matrix.py", "--names"]
        try:
            with redirect_stdout(buf):
                self.mod.main()
        finally:
            sys.argv = argv
        printed = [ln for ln in buf.getvalue().splitlines() if ln.strip()]
        self.assertEqual(printed, self.names)

    # ---- workflow wiring guard (prevents reintroducing the static list) ----------
    def test_workflow_uses_dynamic_fromjson_matrix(self):
        text = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn(
            "matrix: ${{ fromJSON(needs.setup.outputs.matrix) }}",
            text,
            "opt-in-features must consume the assembled matrix via fromJSON",
        )
        # The static `include:` list must NOT be reintroduced into the workflow.
        self.assertNotRegex(
            text,
            r"\n\s+include:\s*\n\s+- name:",
            "a static `include:` leg list reappeared in feature-matrix.yml — add legs to "
            "the per-crate fragments under .github/feature-matrix.d/ instead",
        )

    def test_every_fragment_is_a_clean_leg_list(self):
        import yaml  # noqa: PLC0415  (lazy: only when the test runs)

        self.assertTrue(FRAGMENT_DIR.is_dir(), f"missing {FRAGMENT_DIR}")
        frags = sorted(FRAGMENT_DIR.glob("*.yml"))
        self.assertTrue(frags, "no fragment files found")
        for frag in frags:
            data = yaml.safe_load(frag.read_text(encoding="utf-8"))
            self.assertIsInstance(
                data, list, f"{frag.name}: top level must be a YAML list of legs"
            )


# [FABLE-5] sq-fmx4u.3: change-based selection over the assembled leg list
# (filter_legs_by_selection). The load-bearing property is FAIL-CLOSED: the ONLY
# input shape that may drop a leg is the exact mode "selected" with a well-formed
# affected list; every other input (shadow, full, unset, malformed JSON, wrong
# type) yields the FULL leg set — running more is always sound (design §2/§4.3).
class TestSelectionFiltering(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = _load_assembler()
        cls.legs = cls.mod.load_legs()

    def _filter(self, mode, affected):
        return self.mod.filter_legs_by_selection(self.legs, mode, affected)

    def test_selected_keeps_only_affected_crates(self):
        crates = sorted({leg["crate"] for leg in self.legs})
        keep = crates[0]
        got = self._filter("selected", json.dumps([keep]))
        self.assertTrue(got, "expected at least one leg for a real crate")
        self.assertEqual({leg["crate"] for leg in got}, {keep})
        # And every one of that crate's legs survives — filtering is per-crate,
        # never per-leg-subset.
        self.assertEqual(len(got), sum(1 for leg in self.legs if leg["crate"] == keep))

    def test_selected_with_empty_affected_yields_zero_legs(self):
        # Legitimate: a provably-empty closure => no legs; the workflow's `legs`
        # count output then SKIPS the matrix job (never an empty-matrix error).
        # [OPUS-4.8] This IS the orchestration-only / docs-only change-class outcome:
        # ci_select.py returns mode=selected + affected=[] for a diff that touches no
        # Rust (e.g. routing.toml + triage.py), so the whole opt-in feature matrix is
        # correctly assembled to ZERO legs (skipped-by-class) without any missing
        # required check — the gate discovers the reduced set by polling.
        self.assertEqual(self._filter("selected", "[]"), [])

    def test_change_class_adds_no_legs_full_golden_set_preserved(self):
        # [OPUS-4.8] The change-class layer is a SELECTION refinement (ci_select.py),
        # not a fragment change: the assembler's FULL leg set (the gate-name golden
        # contract) must be byte-identical to the committed golden snapshot. If the
        # change-class work ever accidentally added/renamed a leg, this fails.
        import io
        from contextlib import redirect_stdout
        buf = io.StringIO()
        argv = sys.argv
        try:
            sys.argv = ["assemble", "--names"]
            with redirect_stdout(buf):
                self.mod.main()
        finally:
            sys.argv = argv
        emitted = [ln for ln in buf.getvalue().splitlines() if ln.strip()]
        golden = [ln for ln in GOLDEN.read_text(encoding="utf-8").splitlines() if ln.strip()]
        self.assertEqual(sorted(emitted), sorted(golden))

    def test_shadow_full_and_unset_modes_keep_all(self):
        for mode in ("shadow", "full", "", None, "SELECTED", "enforce"):
            self.assertEqual(len(self._filter(mode, "[]")), len(self.legs),
                             f"mode={mode!r} must fail-close to the FULL leg set")

    def test_malformed_affected_fails_closed_to_full(self):
        for bad in (None, "", "not json", '{"a": 1}', '"str"', '[1, 2]'):
            self.assertEqual(len(self._filter("selected", bad)), len(self.legs),
                             f"affected={bad!r} must fail-close to the FULL leg set")

    def test_names_mode_ignores_selection_flags(self):
        # The golden gate-name proof must always dump the FULL set.
        import io
        from contextlib import redirect_stdout

        buf = io.StringIO()
        argv = sys.argv
        sys.argv = ["assemble-feature-matrix.py", "--names",
                    "--select-mode", "selected", "--affected", "[]"]
        try:
            with redirect_stdout(buf):
                self.mod.main()
        finally:
            sys.argv = argv
        printed = [ln for ln in buf.getvalue().splitlines() if ln.strip()]
        self.assertEqual(printed, sorted(f"opt-in {leg['name']}" for leg in self.legs))

    def test_main_applies_selection_to_matrix_json(self):
        import io
        from contextlib import redirect_stdout

        crates = sorted({leg["crate"] for leg in self.legs})
        keep = crates[-1]
        buf = io.StringIO()
        argv = sys.argv
        sys.argv = ["assemble-feature-matrix.py",
                    "--select-mode", "selected", "--affected", json.dumps([keep])]
        try:
            with redirect_stdout(buf):
                self.mod.main()
        finally:
            sys.argv = argv
        obj = json.loads(buf.getvalue())
        self.assertEqual({leg["crate"] for leg in obj["include"]}, {keep})


# [SONNET-4.6] sq-ldg8c: feature-matrix TIER + EVENT awareness.
#
# The load-bearing property is BEHAVIOUR-PRESERVATION at ZERO annotations: with no
# `tier:` field on any fragment (the state at merge), every event emits exactly today's
# full leg set and the check tier is empty. The tier machinery only takes effect once a
# fragment opts in (the separate flip bead sq-s5dvo). Malformed/unknown tier => HARD ERROR
# (never a silent demotion). Design: research/feature-matrix-pyramid.md §3/§5.


def _run_main_json(mod, extra_argv):
    """Run the assembler's main() with argv and return the parsed JSON object."""
    buf = io.StringIO()
    saved = sys.argv
    sys.argv = ["assemble-feature-matrix.py"] + list(extra_argv)
    try:
        with redirect_stdout(buf):
            mod.main()
    finally:
        sys.argv = saved
    return json.loads(buf.getvalue())


def _synth_legs():
    """Synthetic legs exercising both tiers across the engine/rest shard split.
    (The real fragments carry NO annotations, so the demotion path can only be
    exercised with synthetic input — the behaviour-preservation tests below cover
    the real, zero-annotation fragment set.)"""
    return [
        {"name": "a", "crate": "sparq-core", "features": "f1", "test": True, "tier": "test"},
        {"name": "b", "crate": "sparq-engine", "features": "f2", "test": True, "tier": "check"},
        {"name": "c", "crate": "sparq-kb", "features": "f3", "test": False, "tier": "check"},
        {"name": "d", "crate": "sparq-engine", "features": "f4", "test": True, "tier": "test"},
    ]


class TestTierFiltering(unittest.TestCase):
    """filter_legs_by_tier / filter_legs_by_shard as pure functions."""

    @classmethod
    def setUpClass(cls):
        cls.mod = _load_assembler()

    def _names(self, legs):
        return sorted(leg["name"] for leg in legs)

    # ---- tiered events (pull_request / merge_group) partition by tier -------------
    def test_pull_request_test_tier_excludes_check_legs(self):
        for event in ("pull_request", "merge_group"):
            got = self.mod.filter_legs_by_tier(_synth_legs(), event, "test")
            self.assertEqual(self._names(got), ["a", "d"],
                             f"{event}: default/test tier must exclude tier:check legs")

    def test_pull_request_default_tier_is_test(self):
        # tier=None defaults to 'test' (the matrix output).
        got = self.mod.filter_legs_by_tier(_synth_legs(), "pull_request", None)
        self.assertEqual(self._names(got), ["a", "d"])

    def test_pull_request_check_tier_selects_only_check_legs(self):
        for event in ("pull_request", "merge_group"):
            got = self.mod.filter_legs_by_tier(_synth_legs(), event, "check")
            self.assertEqual(self._names(got), ["b", "c"],
                             f"{event}: --tier check must emit exactly the demoted legs")

    # ---- backstop events (push/schedule/...) emit ALL as full legs ----------------
    def test_backstop_test_tier_emits_all_legs_regardless_of_tier(self):
        for event in ("push", "schedule", "workflow_dispatch", "weird-unknown", None):
            got = self.mod.filter_legs_by_tier(_synth_legs(), event, "test")
            self.assertEqual(self._names(got), ["a", "b", "c", "d"],
                             f"event={event!r}: backstop must run EVERY leg as a full leg")

    def test_backstop_check_tier_is_empty(self):
        # On a full backstop run everything ran as a full leg => the check tier is empty.
        for event in ("push", "schedule", "workflow_dispatch", "weird-unknown", None):
            got = self.mod.filter_legs_by_tier(_synth_legs(), event, "check")
            self.assertEqual(got, [],
                             f"event={event!r}: the check tier must be empty on a full run")

    def test_malformed_tier_flag_is_a_hard_error(self):
        for bad in ("checkk", "TEST", "full", ""):
            with self.assertRaises(SystemExit) as cm:
                self.mod.filter_legs_by_tier(_synth_legs(), "pull_request", bad)
            self.assertNotEqual(cm.exception.code, 0)

    # ---- shard split --------------------------------------------------------------
    def test_shard_engine_vs_rest_partitions_the_check_legs(self):
        check = self.mod.filter_legs_by_tier(_synth_legs(), "pull_request", "check")
        engine = self.mod.filter_legs_by_shard(check, "engine")
        rest = self.mod.filter_legs_by_shard(check, "rest")
        self.assertEqual(self._names(engine), ["b"], "engine shard == sparq-engine legs")
        self.assertEqual(self._names(rest), ["c"], "rest shard == non-engine legs")
        # The two shards partition the check set exactly (no overlap, no drop).
        self.assertEqual(self._names(engine) + self._names(rest), self._names(check))

    def test_shard_none_is_a_noop(self):
        legs = _synth_legs()
        self.assertEqual(self.mod.filter_legs_by_shard(legs, None), legs)

    def test_malformed_shard_flag_is_a_hard_error(self):
        with self.assertRaises(SystemExit) as cm:
            self.mod.filter_legs_by_shard(_synth_legs(), "nope")
        self.assertNotEqual(cm.exception.code, 0)


class TestTierFieldValidation(unittest.TestCase):
    """load_legs() accepts the optional tier/tier-reason keys and validates the tier
    value, with a fresh temp fragment dir so the real fragments are untouched."""

    def setUp(self):
        self.mod = _load_assembler()
        self._tmp = tempfile.TemporaryDirectory()
        self._dir = Path(self._tmp.name)
        self._orig = self.mod.FRAGMENT_DIR
        self.mod.FRAGMENT_DIR = str(self._dir)

    def tearDown(self):
        self.mod.FRAGMENT_DIR = self._orig
        self._tmp.cleanup()

    def _write(self, body):
        (self._dir / "frag.yml").write_text(body, encoding="utf-8")

    _BASE = (
        "- name: demo\n"
        "  crate: sparq-core\n"
        "  features: jsonld\n"
        "  test: true\n"
    )

    def test_missing_tier_defaults_to_test(self):
        self._write(self._BASE)
        legs = self.mod.load_legs()
        self.assertEqual(legs[0]["tier"], "test")

    def test_explicit_test_and_check_are_accepted(self):
        for value in ("test", "check"):
            self._write(self._BASE + f"  tier: {value}\n")
            legs = self.mod.load_legs()
            self.assertEqual(legs[0]["tier"], value)

    def test_tier_reason_override_key_is_allowed(self):
        self._write(self._BASE + "  tier: check\n  tier-reason: reviewed keep\n")
        legs = self.mod.load_legs()
        self.assertEqual(legs[0]["tier"], "check")

    def test_malformed_tier_value_errors(self):
        for bad in ("bogus", "checkk", "", "3", "[a, b]"):
            self._write(self._BASE + f"  tier: {bad}\n")
            with self.assertRaises(SystemExit) as cm:
                self.mod.load_legs()
            self.assertNotEqual(cm.exception.code, 0, f"tier: {bad} must error")

    def test_null_tier_errors(self):
        # A present-but-null tier (`tier:` with no value) must NOT be silently treated
        # as check; it is malformed => error.
        self._write(self._BASE + "  tier:\n")
        with self.assertRaises(SystemExit):
            self.mod.load_legs()

    def test_unknown_extra_key_still_errors(self):
        self._write(self._BASE + "  bogus-key: 1\n")
        with self.assertRaises(SystemExit):
            self.mod.load_legs()


class TestTierBehaviorPreservation(unittest.TestCase):
    """THE load-bearing invariant on the REAL fragment set: zero annotations =>
    byte-identical full leg set on every event, and an empty check tier."""

    @classmethod
    def setUpClass(cls):
        cls.mod = _load_assembler()
        cls.full_names = sorted(leg["name"] for leg in cls.mod.load_legs())

    def _include_names(self, obj):
        return sorted(leg["name"] for leg in obj["include"])

    def test_no_real_fragment_is_annotated_yet(self):
        # Guards the premise of this whole bead: the demotion flip is a SEPARATE bead.
        legs = self.mod.load_legs()
        self.assertTrue(all(leg["tier"] == "test" for leg in legs),
                        "a fragment carries a tier annotation — that belongs to the "
                        "flip bead sq-s5dvo, not the behaviour-preserving wiring bead")

    def test_pull_request_and_merge_group_emit_the_full_set(self):
        for event in ("pull_request", "merge_group"):
            obj = _run_main_json(self.mod, ["--event", event])
            self.assertEqual(self._include_names(obj), self.full_names,
                             f"{event}: test-tier matrix must be the FULL set unchanged")

    def test_push_emits_the_full_set(self):
        obj = _run_main_json(self.mod, ["--event", "push"])
        self.assertEqual(self._include_names(obj), self.full_names)

    def test_check_tier_is_green_empty_on_every_event(self):
        for event in ("pull_request", "merge_group", "push", "schedule"):
            obj = _run_main_json(self.mod, ["--event", event, "--tier", "check"])
            self.assertEqual(obj["include"], [],
                             f"{event}: check tier must be empty at zero annotations")
            for shard in ("engine", "rest"):
                obj = _run_main_json(
                    self.mod, ["--event", event, "--tier", "check", "--shard", shard]
                )
                self.assertEqual(obj["include"], [],
                                 f"{event}/{shard}: check-tier shard must be empty too")

    def test_emitted_legs_have_exactly_the_four_workflow_keys(self):
        # The internal `tier` key must be stripped from the matrix JSON.
        obj = _run_main_json(self.mod, ["--event", "pull_request"])
        for leg in obj["include"]:
            self.assertEqual(set(leg.keys()), {"name", "crate", "features", "test"})


# [FABLE-5] sq-2wo5t: LOUD assemble-failure attribution.
#
# When the `setup` ("assemble feature matrix") job fails, the matrix output is never
# produced, `opt-in-features` spawns ZERO legs, and EVERY required `opt-in *` check on
# the PR goes expected-but-MISSING — which historically misread as "one specific leg
# failing across multiple unrelated PRs" (the 2026-07-11 false 'spqv-provenance'
# main-regression alarm). The workflow therefore carries a final `if: failure()` step
# that emits a ::error annotation + job-summary block attributing the missing legs to
# the assemble job itself. These tests pin that step's WIRING (present, failure-only,
# LAST — so every earlier guard's failure triggers it) and EXECUTE its script (bash is
# the only subprocess; still offline/deterministic) to prove the annotation actually
# fires with the load-bearing text, non-vacuously.
class TestAssembleFailureAnnotation(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        import yaml  # noqa: PLC0415  (lazy: only when the test runs)

        workflow = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))
        cls.setup_steps = workflow["jobs"]["setup"]["steps"]
        cls.failure_steps = [
            s for s in cls.setup_steps if str(s.get("if", "")).strip() == "failure()"
        ]

    def test_setup_job_has_exactly_one_failure_attribution_step(self):
        self.assertEqual(
            len(self.failure_steps),
            1,
            "the setup job must carry exactly ONE `if: failure()` loud-attribution "
            "step (sq-2wo5t) — an assemble failure otherwise reads as a phantom "
            "cross-PR `opt-in *` leg regression",
        )

    def test_failure_attribution_step_is_last(self):
        # LAST is load-bearing: `if: failure()` fires only on an EARLIER step's
        # failure, so any guard step added AFTER it would fail silently again.
        self.assertIs(
            self.setup_steps[-1],
            self.failure_steps[0],
            "the failure-attribution step must be the LAST step of the setup job so "
            "every failure-capable guard step precedes (and thus triggers) it",
        )

    def test_failure_attribution_step_has_no_id_or_outputs_dependency(self):
        # It must be self-contained: no `${{ steps.* }}` reads (an earlier failed
        # step's outputs are empty) and plain `run:` (no action to fetch).
        step = self.failure_steps[0]
        self.assertIn("run", step, "attribution must be a plain run: step")
        self.assertNotIn("uses", step)
        self.assertNotIn("${{ steps.", step["run"])

    def test_failure_annotation_names_the_real_situation(self):
        run = self.failure_steps[0]["run"]
        self.assertIn("::error", run, "must emit a GitHub Actions ::error annotation")
        for needle in (
            "MISSING, not failing",
            "opt-in",
            "feature-matrix-legnames.golden.txt",
            "check-feature-test-execution.py",
        ):
            self.assertIn(
                needle, run,
                f"the loud annotation must mention {needle!r} so triage starts at "
                "the assemble job, not a phantom per-leg regression",
            )

    def test_failure_annotation_script_fires(self):
        """NON-VACUOUS: execute the step's script and assert the annotation + the
        job-summary block are actually produced (catches an unbound var under
        `set -u`, a broken heredoc/quoting, or dropped load-bearing text)."""
        import subprocess  # noqa: PLC0415  (lazy: only this test shells out)

        run = self.failure_steps[0]["run"]
        with tempfile.TemporaryDirectory() as tmp:
            summary = Path(tmp) / "step_summary.md"
            summary.touch()
            proc = subprocess.run(
                ["bash", "-c", run],
                capture_output=True,
                text=True,
                env={"PATH": "/usr/bin:/bin", "GITHUB_STEP_SUMMARY": str(summary)},
                timeout=30,
            )
            self.assertEqual(
                proc.returncode, 0,
                f"the attribution script must not itself fail:\n{proc.stderr}",
            )
            self.assertIn("::error", proc.stdout)
            self.assertIn("MISSING, not failing", proc.stdout)
            summary_text = summary.read_text(encoding="utf-8")
            self.assertIn("opt-in feature matrix NOT generated", summary_text)
            self.assertIn("MISSING, not failing", summary_text)
            self.assertIn("feature-matrix-legnames.golden.txt", summary_text)


if __name__ == "__main__":
    unittest.main(verbosity=2)
