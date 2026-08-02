#!/usr/bin/env python3
# [OPUS-5] Hermetic unit tests for scripts/check-generated-companions.py — the
# generalised one-file-scope-trap gate (sparq-org/sparq#5235, generalising #2384).
#
# Two obligations, and they are different:
#   (1) THE CHECKER HAS TEETH. Each of C1/C2/C3 must go RED on the mutation it exists
#       to catch, against a hermetic fixture repo. Delegated to the script's own
#       `--self-test` mutation table (test_script_self_test_passes) so there is ONE
#       mutation table, plus direct tests here for the boundary cases the table does
#       not carry (a companion the registry does not sweep, the marker being
#       case-insensitive, both-halves-required).
#   (2) THE LIVE REGISTRY IS TRUE. A gate that only ever runs against fixtures proves
#       nothing about this repo, so test_live_repo_is_clean runs the real check over
#       the real checkout, and test_live_registry_declares_every_known_golden pins the
#       #5235 sweep result — the two `scripts/tests/*.golden.*` files the issue names —
#       so silently dropping an entry (which would make the gate vacuously green)
#       fails HERE.
#
# Fully hermetic apart from reading this checkout. Stdlib only; no pytest.
# Run:  python3 scripts/tests/test_generated_companions.py

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SCRIPT = REPO_ROOT / "scripts" / "check-generated-companions.py"


def _load():
    spec = importlib.util.spec_from_file_location("check_generated_companions", SCRIPT)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


G = _load()


class FixtureRepo:
    """A miniature repo with one registered source->companion pair."""

    def __init__(self, tmp: Path):
        self.root = tmp
        G._build_fixture(self.root)

    def anchor(self, name: str = "two.yml") -> Path:
        return self.root / "src.d" / name

    def offences(self) -> list[str]:
        return G.check_root(self.root)

    def codes(self) -> set[str]:
        # Offence strings start with "<CHECK> <KIND>:" — key on that prefix.
        return {o.split(":", 1)[0] for o in self.offences()}


class TestCheckerHasTeeth(unittest.TestCase):
    def setUp(self):
        self._td = tempfile.TemporaryDirectory()
        self.addCleanup(self._td.cleanup)
        self.repo = FixtureRepo(Path(self._td.name))

    def test_clean_fixture_is_green(self):
        self.assertEqual(self.repo.offences(), [])

    def test_script_self_test_passes(self):
        # The script's own mutation table — the single source of truth for C1/C2/C3
        # teeth. Run as a subprocess so the CLI contract is covered too.
        proc = subprocess.run(
            [sys.executable, str(SCRIPT), "--self-test"],
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertIn("all mutations red", proc.stdout)

    def test_anchor_needs_the_marker_AND_the_path_not_either(self):
        # Both halves are load-bearing: the marker without the path does not tell a
        # decomposer WHICH second file to scope; the path without the marker is what
        # sparq-canon.yml did before #5235 (it named the golden but framed it as
        # "a new leg also needs...", silently omitting rename and remove).
        marker_only = "# this is a TWO-FILE change\n"
        path_only = "# see scripts/tests/demo.golden.txt\n"
        both = "# renaming is a TWO-FILE change: scripts/tests/demo.golden.txt\n"
        for text, expect_red in ((marker_only, True), (path_only, True), (both, False)):
            with self.subTest(text=text.strip()):
                self.repo.anchor().write_text(text, encoding="utf-8")
                self.assertEqual("C2 ANCHOR" in self.repo.codes(), expect_red)

    def test_marker_match_is_case_insensitive(self):
        self.repo.anchor().write_text(
            "# a two-file change: scripts/tests/demo.golden.txt\n", encoding="utf-8"
        )
        self.assertNotIn("C2 ANCHOR", self.repo.codes())

    def test_every_anchor_is_checked_not_just_the_first(self):
        # The trap #2384 describes is per-FILE: it bites whichever fragment the bead
        # happened to scope. So an unannotated file must red even when a SIBLING
        # anchor matched by the same glob is fully annotated.
        self.assertEqual(self.repo.offences(), [])
        self.repo.anchor("one.yml").write_text("# leg, no scope note\n", encoding="utf-8")
        offences = self.repo.offences()
        self.assertTrue(any(o.startswith("C2 ANCHOR") for o in offences), offences)
        self.assertTrue(any("one.yml" in o for o in offences), offences)

    def test_c3_sweep_ignores_non_golden_files(self):
        # The ratchet must not red on an ordinary test file living beside the goldens,
        # or it would be turned off rather than obeyed.
        (self.repo.root / "scripts" / "tests" / "test_thing.py").write_text(
            "x\n", encoding="utf-8"
        )
        self.assertNotIn("C3 UNREGISTERED", self.repo.codes())

    def test_malformed_registry_fails_closed(self):
        (self.repo.root / ".github" / "generated-companions.json").write_text(
            "{not json", encoding="utf-8"
        )
        with self.assertRaises(SystemExit):
            self.repo.offences()


class TestLiveRepo(unittest.TestCase):
    def test_live_repo_is_clean(self):
        offences = G.check_root(REPO_ROOT)
        self.assertEqual(offences, [], "\n\n".join(offences))

    def test_live_registry_declares_every_known_golden(self):
        # Pins the #5235 sweep so an entry cannot be dropped to make the gate green.
        registry = G.load_registry(REPO_ROOT)
        declared = set(registry["companions"])
        for known in (
            "scripts/tests/feature-matrix-legnames.golden.txt",
            "scripts/tests/fast-fix-ring.golden.yml",
            "bench/dashboard/metric-labels.json",
        ):
            self.assertIn(known, declared)

    def test_live_entries_record_a_runnable_regen_recipe(self):
        # (a) of the issue: "confirm the regeneration command is documented". An entry
        # whose recipe does not name an existing in-repo generator is a dead recipe.
        registry = G.load_registry(REPO_ROOT)
        for companion, entry in registry["companions"].items():
            with self.subTest(companion=companion):
                named = [
                    t
                    for t in entry["regen"].split()
                    if t.startswith(G.IN_REPO_PREFIXES)
                ]
                self.assertTrue(named, f"{companion}: regen names no in-repo script")
                for token in named:
                    self.assertTrue((REPO_ROOT / token).exists(), token)

    def test_registry_json_is_wellformed_and_newline_terminated(self):
        raw = (REPO_ROOT / ".github" / "generated-companions.json").read_text(
            encoding="utf-8"
        )
        self.assertEqual(raw, raw.rstrip() + "\n", "registry needs one trailing newline")
        json.loads(raw)


if __name__ == "__main__":
    unittest.main(verbosity=2)
