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
# network; the only subprocesses are local runs of the workflow's failure-attribution
# script (TestAssembleFailureAnnotation, sq-2wo5t) and of the group runner/reporter
# with stubbed cargo/gh binaries (TestGroupRunner/TestGroupReporter) — still offline.
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


class TestMergeGroupClassAccounting(unittest.TestCase):
    """[FABLE-5] merge-group change-class (extends #3420/#3421): the expected
    per-class opt-in leg set for a MERGE-GROUP event, via the same composed
    tier+selection path the workflow's assemble step runs. A docs-only/
    orchestration-only queued batch (select emits mode=selected + affected=[]
    over the batch diff) assembles ZERO legs — the `legs` output then skips the
    matrix job, an attributed skip, never a missing required check. An engine
    batch keeps its affected legs; an unresolvable batch (select fail-safe to
    mode=full) keeps the FULL merge-group leg set."""

    @classmethod
    def setUpClass(cls):
        cls.mod = _load_assembler()
        cls.legs = cls.mod.load_legs()

    def _assemble(self, select_mode, affected):
        tiered = self.mod.filter_legs_by_tier(self.legs, "merge_group", "test")
        return self.mod.filter_legs_by_selection(tiered, select_mode, affected)

    def test_docs_only_batch_assembles_zero_legs(self):
        # The #2533-shaped docs-only batch: classify_change => docs-only, the
        # select pre-job => selected + empty closure => zero opt-in legs on the
        # merge_group ref. (Identical outcome for orchestration-only.)
        self.assertEqual(self._assemble("selected", "[]"), [])

    def test_engine_batch_keeps_its_affected_legs(self):
        # An engine batch narrows to the affected closure exactly as on a PR —
        # class gating never removes a leg the selection would keep.
        crates = sorted({leg["crate"] for leg in self.legs})
        keep = crates[0]
        got = self._assemble("selected", json.dumps([keep]))
        self.assertTrue(got, "an engine batch must keep its affected legs")
        self.assertEqual({leg["crate"] for leg in got}, {keep})

    def test_unresolvable_batch_fails_safe_to_full_merge_group_set(self):
        # Any classify/select resolution error => mode=full => the FULL
        # merge-group leg set (fail-safe = cost, never soundness). A classifier
        # mutated back to always-full lands every batch here — visible as the
        # RED docs-only fixtures in test_ci_select.py, and as this full set
        # re-appearing on docs-only groups in the live audit trail.
        full = self._assemble("full", "[]")
        tiered = self.mod.filter_legs_by_tier(self.legs, "merge_group", "test")
        self.assertEqual(full, tiered)


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


# [FABLE-5] CI-economy grouping (maintainer directive 2026-07-18): the per-leg matrix
# is bin-packed into grouped runner jobs (assembler --grouped) and the gate-critical
# `opt-in <name>` check-runs are emitted per leg from INSIDE the group job by
# scripts/run-feature-matrix-group.py. THE load-bearing invariant: grouping is a pure
# PARTITION of the (selection/tier-filtered) leg set — no leg dropped, none duplicated,
# no name changed — so the golden name contract is untouched.
class TestGrouping(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.mod = _load_assembler()
        cls.legs = cls.mod.load_legs()

    def test_grouping_partitions_the_full_leg_set(self):
        groups = self.mod.group_legs(self.legs)
        flat = [leg["name"] for g in groups for leg in g["legs"]]
        self.assertEqual(sorted(flat), sorted(leg["name"] for leg in self.legs),
                         "grouping must partition the leg set exactly (no drop/dup)")
        self.assertEqual(len(flat), len(set(flat)), "no leg may appear twice")

    def test_group_capacity_respected_for_multi_leg_groups(self):
        # A single leg heavier than the capacity may stand alone; any MULTI-leg
        # group must fit the capacity (the <5 min wall-time target).
        for g in self.mod.group_legs(self.legs):
            if g["count"] > 1:
                self.assertLessEqual(
                    g["weight"], self.mod.GROUP_CAPACITY,
                    f"group {g['group']} exceeds the capacity with multiple legs")

    def test_grouping_is_deterministic(self):
        a = self.mod.group_legs(self.legs)
        b = self.mod.group_legs(self.legs)
        self.assertEqual(a, b, "grouping must be deterministic run-to-run")

    def test_groups_are_meaningfully_fewer_than_legs(self):
        groups = self.mod.group_legs(self.legs)
        self.assertLess(len(groups), len(self.legs) / 2,
                        "grouping must collapse the runner count substantially — "
                        "one-leg-per-group would defeat the CI-economy directive")

    def test_grouped_main_output_shape(self):
        obj = _run_main_json(self.mod, ["--grouped", "--event", "pull_request"])
        self.assertIn("include", obj)
        total = 0
        for g in obj["include"]:
            self.assertEqual(set(g.keys()), {"group", "cache_crate", "count", "legs"})
            inner = json.loads(g["legs"])  # legs is a JSON-encoded STRING (matrix scalar)
            self.assertEqual(len(inner), g["count"])
            total += g["count"]
            for leg in inner:
                self.assertEqual(set(leg.keys()), {"name", "crate", "features", "test"})
        self.assertEqual(total, len(self.legs))

    def test_grouped_composes_with_selection(self):
        obj = _run_main_json(self.mod, [
            "--grouped", "--event", "pull_request",
            "--select-mode", "selected", "--affected", "[]"])
        self.assertEqual(obj["include"], [],
                         "an empty affected closure must assemble ZERO groups")
        crates = sorted({leg["crate"] for leg in self.legs})
        keep = crates[0]
        obj = _run_main_json(self.mod, [
            "--grouped", "--event", "pull_request",
            "--select-mode", "selected", "--affected", json.dumps([keep])])
        inner = [leg for g in obj["include"] for leg in json.loads(g["legs"])]
        self.assertTrue(inner)
        self.assertEqual({leg["crate"] for leg in inner}, {keep})

    def test_explicit_weight_overrides_heuristic(self):
        leg = dict(self.legs[0])
        leg["weight"] = 7.5
        self.assertEqual(self.mod.leg_weight(leg), 7.5)
        leg["weight"] = None
        self.assertGreater(self.mod.leg_weight(leg), 0)

    def test_single_overweight_leg_gets_its_own_group(self):
        legs = [
            {"name": "huge", "crate": "c1", "features": "f", "test": True,
             "weight": self.mod.GROUP_CAPACITY * 3},
            {"name": "tiny", "crate": "c2", "features": "f", "test": True,
             "weight": 0.1},
        ]
        groups = self.mod.group_legs(legs)
        huge = [g for g in groups if any(l["name"] == "huge" for l in g["legs"])]
        self.assertEqual(len(huge), 1)
        self.assertEqual(huge[0]["count"], 1,
                         "an over-capacity leg must stand alone, never merged")

    def test_same_crate_legs_cluster_before_packing(self):
        # Two crates, each with legs that fit one bin: no group may interleave
        # them while a same-crate leg still fits — target-dir reuse is the point.
        legs = []
        for crate in ("ca", "cb"):
            for i in range(3):
                legs.append({"name": f"{crate}-{i}", "crate": crate,
                             "features": "f", "test": True, "weight": 1.0})
        for g in self.mod.group_legs(legs):
            self.assertEqual(len({leg["crate"] for leg in g["legs"]}), 1,
                             "fitting same-crate chunks must not be split across "
                             "crates when each crate fits a bin alone")


class TestWeightFieldValidation(unittest.TestCase):
    """load_legs() accepts the optional `weight` key and hard-errors on malformed
    values (a bad weight must never silently skew the packing)."""

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

    def test_missing_weight_defaults_to_heuristic(self):
        self._write(self._BASE)
        legs = self.mod.load_legs()
        self.assertIsNone(legs[0]["weight"])
        self.assertGreater(self.mod.leg_weight(legs[0]), 0)

    def test_explicit_positive_weight_is_accepted(self):
        for value in ("2", "0.5", "10.25"):
            self._write(self._BASE + f"  weight: {value}\n")
            legs = self.mod.load_legs()
            self.assertEqual(legs[0]["weight"], float(value))

    def test_malformed_weight_errors(self):
        for bad in ("0", "-1", "abc", "true", ".nan", ".inf", "[1]"):
            self._write(self._BASE + f"  weight: {bad}\n")
            with self.assertRaises(SystemExit) as cm:
                self.mod.load_legs()
            self.assertNotEqual(cm.exception.code, 0, f"weight: {bad} must error")


class TestGroupedWorkflowWiring(unittest.TestCase):
    """Pins the grouped-runner wiring in feature-matrix.yml ACROSS THE TRUSTED
    BOUNDARY (PR #3511 review, critical finding): the matrix is assembled with
    --grouped; the UNPRIVILEGED group job (contents: read only, no persisted
    credentials, NO token in its environment — it executes PR-controlled cargo
    build scripts/proc macros) runs scripts/run-feature-matrix-group.py and
    uploads the per-leg results as an artifact; the separate TRUSTED
    `report-legs` job (checks: write, executes no PR build code) downloads the
    artifacts, regenerates the assembled leg set as ground truth, and posts the
    gate-critical per-leg check-runs via --report."""

    @classmethod
    def setUpClass(cls):
        import yaml  # noqa: PLC0415

        cls.text = WORKFLOW.read_text(encoding="utf-8")
        cls.wf = yaml.safe_load(cls.text)
        cls.job = cls.wf["jobs"]["opt-in-features"]
        cls.report = cls.wf["jobs"]["report-legs"]

    def test_assemble_step_uses_grouped_mode(self):
        self.assertIn("assemble-feature-matrix.py --grouped", self.text)

    def test_group_job_runs_the_group_runner_script(self):
        runs = [s.get("run", "") for s in self.job["steps"]]
        self.assertTrue(any("run-feature-matrix-group.py" in r for r in runs),
                        "the group job must run scripts/run-feature-matrix-group.py")

    # ---- the trusted boundary itself (do NOT weaken any of these) ----------------
    def test_group_job_is_unprivileged(self):
        self.assertEqual(
            self.job.get("permissions"), {"contents": "read"},
            "the group job executes PR-controlled build code and must hold ONLY "
            "contents: read — a checks: write token here lets malicious build "
            "scripts/proc macros forge arbitrary check runs (PR #3511 review)")

    def test_group_job_checkout_does_not_persist_credentials(self):
        checkouts = [s for s in self.job["steps"]
                     if "actions/checkout" in str(s.get("uses", ""))]
        self.assertTrue(checkouts, "the group job must check out the repo")
        for s in checkouts:
            self.assertIs(
                (s.get("with") or {}).get("persist-credentials"), False,
                "the group job's checkout must set persist-credentials: false — "
                "PR-controlled build code must never find a token in .git/config")

    def test_group_job_env_has_no_token(self):
        env = {}
        for s in self.job["steps"]:
            env.update(s.get("env", {}) or {})
        for key in ("GH_TOKEN", "GITHUB_TOKEN"):
            self.assertNotIn(key, env,
                             f"the group job must not export {key} while running "
                             "PR-controlled cargo code")
        for key in ("GROUP_LEGS", "GROUP_NAME", "FMG_RESULTS"):
            self.assertIn(key, env, f"group runner step must set {key}")

    def test_no_cargo_running_job_holds_checks_write(self):
        """Workflow-wide sweep: any job that executes cargo (i.e. PR-controlled
        build-time code) must never hold checks: write nor export a token."""
        for job_id, job in self.wf["jobs"].items():
            steps = job.get("steps") or []
            runs = " ".join(str(s.get("run", "") or "") for s in steps)
            if "cargo" not in runs:
                continue
            self.assertNotEqual(
                (job.get("permissions") or {}).get("checks"), "write",
                f"job {job_id!r} executes cargo and must not hold checks: write")
            env = {}
            for s in steps:
                env.update(s.get("env", {}) or {})
            self.assertNotIn("GH_TOKEN", env,
                             f"job {job_id!r} executes cargo and must not export GH_TOKEN")

    def test_group_job_uploads_results_artifact(self):
        uploads = [s for s in self.job["steps"]
                   if "actions/upload-artifact" in str(s.get("uses", ""))]
        self.assertEqual(len(uploads), 1,
                         "the group job must upload exactly one results artifact")
        step = uploads[0]
        self.assertIn("feature-matrix-results-", str(step["with"]["name"]))
        self.assertIn("!cancelled()", str(step.get("if", "")),
                      "a group with FAILED legs must still ship its results so the "
                      "reporter posts the red per-leg check-runs")

    # ---- the trusted reporter side -----------------------------------------------
    def test_report_job_holds_checks_write_and_runs_no_cargo(self):
        self.assertEqual((self.report.get("permissions") or {}).get("checks"),
                         "write", "the reporter is the ONE checks: write holder")
        runs = " ".join(str(s.get("run", "") or "") for s in self.report["steps"])
        self.assertNotIn("cargo", runs,
                         "the trusted reporter must execute no PR build code")
        self.assertIn("run-feature-matrix-group.py --report", runs)
        self.assertIn("assemble-feature-matrix.py", runs,
                      "the reporter must regenerate the assembled leg set as its "
                      "ground truth (leg names are never taken from the artifact "
                      "as free text)")

    def test_report_job_downloads_the_results_artifacts(self):
        downloads = [s for s in self.report["steps"]
                     if "actions/download-artifact" in str(s.get("uses", ""))]
        self.assertEqual(len(downloads), 1)
        self.assertEqual(downloads[0]["with"]["pattern"], "feature-matrix-results-*")

    def test_report_job_runs_even_when_groups_fail(self):
        cond = str(self.report.get("if", ""))
        self.assertIn("!cancelled()", cond,
                      "the reporter must run when a group job FAILED (red legs "
                      "still need their check-runs)")
        self.assertIn("needs.setup.outputs.legs != '0'", cond)
        self.assertIn("needs.setup.result == 'success'", cond)

    def test_report_job_env(self):
        env = {}
        for s in self.report["steps"]:
            env.update(s.get("env", {}) or {})
        for key in ("FMG_RESULTS_DIR", "FMG_VALID_LEGS_FILE", "REPO", "HEAD_SHA",
                    "GH_TOKEN", "GROUP_JOBS_RESULT"):
            self.assertIn(key, env, f"reporter step must set {key}")

    def test_job_names_are_not_advisory(self):
        for job in (self.job, self.report):
            self.assertNotRegex(str(job.get("name", "")), ADVISORY_RE,
                                "both sides of the split must GATE — a red "
                                "reporter is the fail-closed enforcement")

    # ---- PR #3511 review finding 2: the runner script triggers its own gate ------
    def test_rust_path_filter_covers_the_matrix_scripts(self):
        import yaml  # noqa: PLC0415

        steps = self.wf["jobs"]["changes"]["steps"]
        flt = [s for s in steps if "dorny/paths-filter" in str(s.get("uses", ""))]
        self.assertEqual(len(flt), 1)
        filters = yaml.safe_load(flt[0]["with"]["filters"])
        rust = filters["rust"]
        for path in (
            "scripts/assemble-feature-matrix.py",
            "scripts/run-feature-matrix-group.py",
            ".github/feature-matrix.d/**",
            ".github/workflows/feature-matrix.yml",
        ):
            self.assertIn(
                path, rust,
                f"the rust path filter must include {path!r} — a PR changing only "
                "it must still run setup (self-test), the grouped execution and "
                "the reporter (PR #3511 review finding 2)")


class TestGroupRunner(unittest.TestCase):
    """Executes scripts/run-feature-matrix-group.py (EXECUTION mode) with a stubbed
    cargo binary (FMG_CARGO): a failing leg must NOT stop the group, every leg's
    outcome must land in the results file for the trusted reporter, the group must
    exit non-zero naming the failed leg — and the mode must need NO GitHub token
    and make NO API call (the trusted-boundary contract, PR #3511 review)."""

    RUNNER = REPO_ROOT / "scripts" / "run-feature-matrix-group.py"
    SCHEMA = "feature-matrix-group-results/v1"

    def _run(self, legs, fail_on="", with_results_env=True):
        import subprocess  # noqa: PLC0415

        with tempfile.TemporaryDirectory() as tmp:
            tmpdir = Path(tmp)
            cargo_log = tmpdir / "cargo.log"
            summary = tmpdir / "summary.md"
            results = tmpdir / "fmg" / "results.json"
            cargo = tmpdir / "cargo-stub"
            cargo.write_text(
                "#!/bin/bash\n"
                f"echo \"$@\" >> {cargo_log}\n"
                "case \" $* \" in *\" ${FAIL_ON:-__none__} \"*) exit 1;; esac\n"
                "exit 0\n",
                encoding="utf-8",
            )
            cargo.chmod(0o755)
            summary.touch()
            # Deliberately NO GH_TOKEN / gh stub in the environment: execution
            # mode must never need or touch a token (trusted-boundary contract).
            env = {
                "PATH": "/usr/bin:/bin",
                "FMG_CARGO": str(cargo),
                "FAIL_ON": fail_on,
                "GROUP_LEGS": json.dumps(legs),
                "GROUP_NAME": "g01 test-group",
                "GITHUB_STEP_SUMMARY": str(summary),
            }
            if with_results_env:
                env["FMG_RESULTS"] = str(results)
            proc = subprocess.run(
                [sys.executable, str(self.RUNNER)],
                capture_output=True,
                text=True,
                timeout=60,
                env=env,
            )
            results_obj = (
                json.loads(results.read_text(encoding="utf-8"))
                if results.exists()
                else None
            )
            cargo_lines = (
                cargo_log.read_text(encoding="utf-8").splitlines()
                if cargo_log.exists()
                else []
            )
            return proc, results_obj, summary.read_text(encoding="utf-8"), cargo_lines

    LEGS = [
        {"name": "alpha (f1)", "crate": "alpha", "features": "f1", "test": True},
        {"name": "beta (f2)", "crate": "beta", "features": "f2", "test": False},
    ]

    def test_all_green_writes_a_success_result_per_leg(self):
        proc, results, _, _ = self._run(self.LEGS)
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertEqual(results["schema"], self.SCHEMA)
        self.assertEqual(results["group"], "g01 test-group")
        self.assertEqual(
            results["results"],
            [{"name": "alpha (f1)", "failed_step": None},
             {"name": "beta (f2)", "failed_step": None}],
            "every leg's outcome must land in the results file for the reporter")

    def test_failing_leg_continues_reports_and_reds_the_group(self):
        proc, results, summary, _ = self._run(self.LEGS, fail_on="f1")
        self.assertEqual(proc.returncode, 1, "a failed leg must RED the group")
        # The failure names the exact leg, loudly.
        self.assertIn("opt-in alpha (f1)", proc.stdout)
        self.assertIn("::error", proc.stdout)
        # The group kept going: BOTH legs land in the results (fail-fast: false).
        self.assertEqual(
            results["results"],
            [{"name": "alpha (f1)", "failed_step": "build"},
             {"name": "beta (f2)", "failed_step": None}])
        self.assertIn("alpha (f1)", summary)

    def test_untested_leg_skips_cargo_test(self):
        proc, results, _, cargo_lines = self._run([self.LEGS[1]])
        self.assertEqual(proc.returncode, 0)
        self.assertEqual(results["results"][0]["failed_step"], None)
        self.assertFalse(any(line.startswith("test ") for line in cargo_lines),
                         "an untested leg must not invoke `cargo test`")

    def test_missing_results_env_is_fatal(self):
        # Without FMG_RESULTS the reporter would have nothing to post from —
        # every leg name would silently vanish from the gate's discovery set.
        proc, _, _, cargo_lines = self._run(self.LEGS, with_results_env=False)
        self.assertEqual(proc.returncode, 2)
        self.assertEqual(cargo_lines, [], "must refuse to run any leg without it")

    def test_execution_mode_makes_no_github_api_call(self):
        # No gh binary exists on the stub PATH; a mode that shelled out to gh
        # would crash. Green run == no API call was attempted.
        proc, _, _, _ = self._run(self.LEGS)
        self.assertEqual(proc.returncode, 0)
        self.assertNotIn("check-run", proc.stdout,
                         "execution mode must not attempt check-run emission")


class TestGroupReporter(unittest.TestCase):
    """Executes scripts/run-feature-matrix-group.py --report (the TRUSTED side of
    the split) with a stubbed gh binary. The artifact files it consumes were
    written inside jobs that ran arbitrary PR-controlled build code, so the
    load-bearing properties are: HOSTILE-INPUT validation (leg names must resolve
    in the assembled leg set — never PR-supplied free text; strict schema;
    duplicate names refused), and FAIL-CLOSED posting (only the deterministic
    fork-PR read-only-token denial degrades to a warning; every other Checks API
    failure reds the reporter job — PR #3511 review finding 3)."""

    RUNNER = REPO_ROOT / "scripts" / "run-feature-matrix-group.py"
    SCHEMA = "feature-matrix-group-results/v1"
    FORK_DENIAL = "gh: Resource not accessible by integration (HTTP 403)"

    VALID = {"include": [
        {"name": "alpha (f1)", "crate": "alpha", "features": "f1", "test": True},
        {"name": "beta (f2)", "crate": "beta", "features": "f2", "test": False},
    ]}

    def _result_file(self, entries, group="g01 alpha", schema=None):
        return {"schema": schema or self.SCHEMA, "group": group, "results": entries}

    def _run_report(self, files, gh_rc="0", gh_stderr="", valid=None,
                    group_jobs_result="failure", raw_files=None, drop_env=()):
        import subprocess  # noqa: PLC0415

        with tempfile.TemporaryDirectory() as tmp:
            tmpdir = Path(tmp)
            results_dir = tmpdir / "fmg-results"
            for i, obj in enumerate(files):
                sub = results_dir / f"feature-matrix-results-{i}"
                sub.mkdir(parents=True)
                (sub / "results.json").write_text(
                    json.dumps(obj), encoding="utf-8")
            for i, raw in enumerate(raw_files or []):
                sub = results_dir / f"raw-{i}"
                sub.mkdir(parents=True)
                (sub / "results.json").write_text(raw, encoding="utf-8")
            results_dir.mkdir(parents=True, exist_ok=True)
            valid_file = tmpdir / "valid-legs.json"
            valid_file.write_text(json.dumps(valid or self.VALID), encoding="utf-8")
            gh_log = tmpdir / "gh.log"
            gh = tmpdir / "gh-stub"
            gh.write_text(
                "#!/bin/bash\n"
                f"cat >> {gh_log}\n"
                f"echo >> {gh_log}\n"
                "if [ -n \"${GH_STDERR:-}\" ]; then echo \"$GH_STDERR\" >&2; fi\n"
                "exit ${GH_RC:-0}\n",
                encoding="utf-8",
            )
            gh.chmod(0o755)
            env = {
                "PATH": "/usr/bin:/bin",
                "FMG_GH": str(gh),
                "GH_RC": gh_rc,
                "GH_STDERR": gh_stderr,
                "FMG_RESULTS_DIR": str(results_dir),
                "FMG_VALID_LEGS_FILE": str(valid_file),
                "REPO": "example/repo",
                "HEAD_SHA": "deadbeef",
                "DETAILS_URL": "https://example.invalid/run/1",
                "GROUP_JOBS_RESULT": group_jobs_result,
                "FMG_RETRY_SLEEP": "0",
            }
            for key in drop_env:
                env.pop(key, None)
            proc = subprocess.run(
                [sys.executable, str(self.RUNNER), "--report"],
                capture_output=True,
                text=True,
                timeout=60,
                env=env,
            )
            gh_calls = (
                [json.loads(l) for l in gh_log.read_text().splitlines() if l.strip()]
                if gh_log.exists()
                else []
            )
            return proc, gh_calls

    def test_valid_results_post_terminal_check_runs(self):
        proc, gh_calls = self._run_report([
            self._result_file([{"name": "alpha (f1)", "failed_step": "clippy"}]),
            self._result_file([{"name": "beta (f2)", "failed_step": None}],
                              group="g02 beta"),
        ], group_jobs_result="success")
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertEqual([c["name"] for c in gh_calls],
                         ["opt-in alpha (f1)", "opt-in beta (f2)"],
                         "every leg must get its byte-identical `opt-in <name>` run")
        self.assertEqual([c["conclusion"] for c in gh_calls],
                         ["failure", "success"])
        for c in gh_calls:
            self.assertEqual(c["status"], "completed",
                             "check-runs must be TERMINAL (a pending one could "
                             "hold the polling ci-summary gate)")
            self.assertEqual(c["head_sha"], "deadbeef")
            self.assertTrue(c["name"].startswith("opt-in "),
                            "the reporter constructs the opt-in prefix itself")
        # Summary metadata comes from the ASSEMBLED set, not the artifact.
        self.assertIn("-p alpha --features f1", gh_calls[0]["output"]["summary"])

    def test_unassembled_leg_name_is_refused_and_nothing_posts(self):
        # THE forgery test: an artifact naming a non-assembled leg — e.g. trying
        # to spoof the required gate context — must fail validation before any
        # POST happens (never PR-supplied free text).
        for forged in ("ci-summary / gate", "gate", "alpha (f1) ", "totally-new"):
            proc, gh_calls = self._run_report([
                self._result_file([{"name": forged, "failed_step": None}]),
            ])
            self.assertEqual(proc.returncode, 1, f"forged name {forged!r} must fail")
            self.assertEqual(gh_calls, [], "no POST may happen on a forged artifact")

    def test_malformed_artifacts_fail_closed_without_posting(self):
        cases = [
            # not JSON at all
            {"raw_files": ["not json {"]},
            # wrong schema tag
            {"files": [self._result_file(
                [{"name": "alpha (f1)", "failed_step": None}], schema="v0")]},
            # extra top-level key
            {"raw_files": [json.dumps({"schema": self.SCHEMA, "group": "g01 a",
                                       "results": [], "extra": 1})]},
            # entry with extra keys
            {"files": [self._result_file(
                [{"name": "alpha (f1)", "failed_step": None, "summary": "pwn"}])]},
            # failed_step outside the enum
            {"files": [self._result_file(
                [{"name": "alpha (f1)", "failed_step": "pwned"}])]},
            # hostile group id (annotation/markdown injection shape)
            {"files": [self._result_file(
                [{"name": "alpha (f1)", "failed_step": None}],
                group="x\n::error::pwn")]},
            # empty results list
            {"files": [self._result_file([])]},
        ]
        for case in cases:
            proc, gh_calls = self._run_report(case.get("files", []),
                                              raw_files=case.get("raw_files"))
            self.assertEqual(proc.returncode, 1, f"case {case!r} must fail closed")
            self.assertEqual(gh_calls, [], f"case {case!r} must not POST anything")

    def test_duplicate_leg_across_artifacts_is_refused(self):
        # Duplicate sibling check-runs are the latest-run-confusion attack the
        # split exists to prevent — refuse them from the honest side too.
        proc, gh_calls = self._run_report([
            self._result_file([{"name": "alpha (f1)", "failed_step": None}]),
            self._result_file([{"name": "alpha (f1)", "failed_step": "build"}],
                              group="g02 alpha"),
        ])
        self.assertEqual(proc.returncode, 1)
        self.assertEqual(gh_calls, [])

    def test_fork_token_denial_degrades_to_warning(self):
        # The ONE expected, tolerable API failure: a fork PR's read-only token.
        proc, gh_calls = self._run_report(
            [self._result_file([{"name": "alpha (f1)", "failed_step": None}])],
            gh_rc="1", gh_stderr=self.FORK_DENIAL, group_jobs_result="success")
        self.assertEqual(proc.returncode, 0,
                         "fork-token denial must not fail the reporter — the "
                         "group jobs' own conclusions remain the gating signal")
        self.assertIn("::warning", proc.stdout)
        self.assertEqual(len(gh_calls), 1, "deterministic denial is not retried")

    def test_unexpected_api_failure_fails_closed_after_retries(self):
        # Auth regression / outage / rate limit / malformed request => the
        # reporter job REDs (finding 3) — never a silent warning.
        proc, gh_calls = self._run_report(
            [self._result_file([{"name": "alpha (f1)", "failed_step": None}])],
            gh_rc="1", gh_stderr="HTTP 502 Bad Gateway", group_jobs_result="success")
        self.assertEqual(proc.returncode, 1,
                         "an unexpected Checks API failure must fail the reporter")
        self.assertEqual(len(gh_calls), 3, "bounded retry before failing closed")
        self.assertIn("::error", proc.stderr)

    def test_zero_artifacts_with_successful_groups_is_an_inconsistency(self):
        proc, _ = self._run_report([], group_jobs_result="success")
        self.assertEqual(proc.returncode, 1,
                         "green groups + zero artifacts would silently drop every "
                         "per-leg check-run — must fail")
        proc, _ = self._run_report([], group_jobs_result="failure")
        self.assertEqual(proc.returncode, 0,
                         "failed groups gate via ci-summary; nothing to report")
        self.assertIn("::warning", proc.stdout)

    def test_missing_reporter_env_is_fatal(self):
        for key in ("FMG_RESULTS_DIR", "FMG_VALID_LEGS_FILE", "REPO", "HEAD_SHA"):
            proc, gh_calls = self._run_report(
                [self._result_file([{"name": "alpha (f1)", "failed_step": None}])],
                drop_env=(key,))
            self.assertEqual(proc.returncode, 2, f"missing {key} must be fatal")
            self.assertEqual(gh_calls, [])

    def test_real_assembled_legs_pass_reporter_validation(self):
        """End-to-end coherence: EXECUTION-mode output for real assembled legs
        must validate on the REPORT side against the real assembler output —
        pins the schema the two halves share across the trust boundary."""
        mod = _load_assembler()
        legs = mod.load_legs()
        first = {"name": legs[0]["name"], "failed_step": None}
        valid = {"include": [
            {"name": leg["name"], "crate": leg["crate"],
             "features": leg["features"], "test": leg["test"]}
            for leg in legs
        ]}
        proc, gh_calls = self._run_report(
            [self._result_file([first], group="g01 real")],
            valid=valid, group_jobs_result="success")
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertEqual(gh_calls[0]["name"], f"opt-in {legs[0]['name']}")


if __name__ == "__main__":
    unittest.main(verbosity=2)
