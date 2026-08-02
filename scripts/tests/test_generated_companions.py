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
#       The mutation table drives every check through the `check_root` AGGREGATE, and
#       that aggregate skips check_c2 for any entry whose check_c1 already red — so a
#       table-only suite can leave an individual check unpinned (delete check_c2 and
#       the C2 rows still red via a neighbouring C1 offence). test_check_c1/c2/c3_*
#       therefore call each check FUNCTION directly: deleting or inverting exactly one
#       of the three reds its own test.
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

    # ---- one direct test per check function -------------------------------
    # Each drives the check ALONE, so it pins that check specifically rather than
    # relying on the aggregate (see the module header). Each asserts both halves:
    # green on the clean fixture, and the SPECIFIC offence code on each mutation the
    # check owns — so inverting the check (always-red) fails too, not just deleting it.

    COMPANION = "scripts/tests/demo.golden.txt"

    def entry(self) -> dict:
        return G.load_registry(self.repo.root)["companions"][self.COMPANION]

    @staticmethod
    def _codes(offences: list[str]) -> set[str]:
        return {o.split(":", 1)[0] for o in offences}

    def test_check_c1_reds_on_every_unresolvable_path(self):
        root, comp = self.repo.root, self.COMPANION
        clean = self.entry()
        self.assertEqual(G.check_c1(root, comp, clean), [])

        for label, entry, expect in (
            ("dead regen recipe", dict(clean, regen="python3 scripts/gone.py"), "C1 REGEN"),
            ("stale sources glob", dict(clean, sources=["gone/*.yml"]), "C1 SOURCES"),
            ("stale anchor glob", dict(clean, scope_anchors=["gone/*.yml"]), "C1 SCOPE_ANCHORS"),
            ("field dropped", {k: v for k, v in clean.items() if k != "trigger"}, "C1 FIELDS"),
        ):
            with self.subTest(label):
                self.assertIn(expect, self._codes(G.check_c1(root, comp, entry)))

        # The companion itself going missing — the entry is untouched, the file is not.
        (root / comp).unlink()
        self.assertIn("C1 COMPANION", self._codes(G.check_c1(root, comp, clean)))

    def test_check_c2_reds_on_the_specific_unannotated_anchor(self):
        root, comp = self.repo.root, self.COMPANION
        entry = self.entry()
        self.assertEqual(G.check_c2(root, comp, entry), [])

        self.repo.anchor("one.yml").write_text("# leg, no scope note\n", encoding="utf-8")
        offences = G.check_c2(root, comp, entry)
        self.assertIn("C2 ANCHOR", self._codes(offences))
        # Names the offending file — a decomposer has to know WHICH anchor to annotate.
        self.assertTrue(any("one.yml" in o for o in offences), offences)
        # ...and only that one: the annotated sibling must stay green.
        self.assertFalse(any("two.yml" in o for o in offences), offences)

    def test_check_c3_reds_only_on_an_undeclared_golden(self):
        root = self.repo.root
        registry = G.load_registry(root)
        self.assertEqual(G.check_c3(root, registry), [])

        # A non-golden file beside the goldens is not swept.
        (root / "scripts" / "tests" / "test_thing.py").write_text("x\n", encoding="utf-8")
        self.assertEqual(G.check_c3(root, registry), [])

        (root / "scripts" / "tests" / "other.golden.yml").write_text("x\n", encoding="utf-8")
        offences = G.check_c3(root, registry)
        self.assertIn("C3 UNREGISTERED", self._codes(offences))
        self.assertTrue(any("other.golden.yml" in o for o in offences), offences)

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
