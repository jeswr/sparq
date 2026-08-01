#!/usr/bin/env python3
# [OPUS-5] Hermetic tests for scripts/coverage-presence.py (sparq-org/sparq#5140).
#
# The gate's own failure mode was COVERAGE, not counting: check() iterated the recorded
# floors only, so a crate added to crates/ after the last --seed was guarded by nothing
# and could lose every one of its tests while the gate still printed all-ok. 25 of 67
# crates (incl. sparq-lws-wasm) had drifted into exactly that state. Three layers:
#
#   1. evaluate() PURE fixtures — the four fail directions (below floor, lost tests/ dir,
#      crate disappeared, crate UNTRACKED) and the ok direction, on dicts, no filesystem.
#   2. A LIVE invariant on this repo: every directory under crates/ with a Cargo.toml is
#      recorded in bench/coverage-presence.json, and the committed floors currently pass.
#   3. VACUOUS-PASS guards: the live scan must actually find the crates (a detector that
#      silently found nothing would make layer 2 pass by scanning an empty tree — which is
#      the same shape of hole this gate exists to close), and the untracked rule must be
#      shown to fail on a tree the file does not fully cover.
#
# Run:  python3 scripts/tests/test_coverage_presence.py
# (stdlib only; no pytest required — also discoverable by `pytest`.)

from __future__ import annotations

import importlib.util
import json
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
PRESENCE_JSON = REPO_ROOT / "bench" / "coverage-presence.json"
CRATES_DIR = REPO_ROOT / "crates"


def _load(name: str, filename: str):
    spec = importlib.util.spec_from_file_location(name, REPO_ROOT / "scripts" / filename)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


gate = _load("coverage_presence", "coverage-presence.py")


def _tree(**crates):
    """{crate: (tests, integration_dir)} -> the shape scan() returns."""
    return {c: {"tests": t, "integration_dir": i} for c, (t, i) in crates.items()}


class PureLogic(unittest.TestCase):
    """evaluate() decision table on dicts — no filesystem, no compile."""

    FLOORS = {
        "sparq-a": {"min_tests": 10, "had_integration_dir": True},
        "sparq-b": {"min_tests": 5, "had_integration_dir": False},
    }

    def test_at_and_above_floor_is_ok(self):
        # Floors are >=, so adding tests is always fine.
        oks, fails = gate.evaluate(
            self.FLOORS, _tree(**{"sparq-a": (10, True), "sparq-b": (99, False)})
        )
        self.assertEqual(fails, [])
        self.assertEqual(len(oks), 2)

    def test_below_floor_fails(self):
        _oks, fails = gate.evaluate(
            self.FLOORS, _tree(**{"sparq-a": (9, True), "sparq-b": (5, False)})
        )
        self.assertEqual(len(fails), 1)
        self.assertIn("sparq-a", fails[0])
        self.assertIn("9 tests < floor 10", fails[0])

    def test_lost_integration_dir_fails_even_at_floor(self):
        # The count can hold while the whole tests/ dir is deleted.
        _oks, fails = gate.evaluate(
            self.FLOORS, _tree(**{"sparq-a": (10, False), "sparq-b": (5, False)})
        )
        self.assertEqual(len(fails), 1)
        self.assertIn("lost its tests/ integration dir", fails[0])

    def test_tracked_crate_disappearing_fails(self):
        _oks, fails = gate.evaluate(self.FLOORS, _tree(**{"sparq-a": (10, True)}))
        self.assertEqual(len(fails), 1)
        self.assertIn("sparq-b", fails[0])
        self.assertIn("DISAPPEARED", fails[0])

    # --- the #5140 guard -------------------------------------------------------------
    def test_untracked_crate_fails(self):
        """A crate in crates/ with NO entry in the floor file is a FAIL, not a silent ok.

        Before #5140 this loop did not exist: an unrecorded crate was invisible to the
        gate, so its tests were guarded by nothing at all."""
        _oks, fails = gate.evaluate(
            self.FLOORS,
            _tree(**{"sparq-a": (10, True), "sparq-b": (5, False), "sparq-lws-wasm": (26, True)}),
        )
        self.assertEqual(len(fails), 1)
        self.assertIn("sparq-lws-wasm", fails[0])
        self.assertIn("UNTRACKED", fails[0])

    def test_untracked_crate_with_zero_tests_still_fails(self):
        # A brand-new crate typically lands with few or no tests; it must still be
        # recorded, otherwise it is un-gated forever (the sparq-wrapper-* seam case).
        _oks, fails = gate.evaluate(
            self.FLOORS,
            _tree(**{"sparq-a": (10, True), "sparq-b": (5, False), "sparq-new": (0, False)}),
        )
        self.assertEqual([f.split(":")[0] for f in fails], ["sparq-new"])

    def test_untracked_does_not_mask_other_failures(self):
        # Both directions are reported in one run, so a re-seed does not have to be
        # iterated once per hidden regression.
        _oks, fails = gate.evaluate(
            self.FLOORS, _tree(**{"sparq-a": (1, True), "sparq-b": (5, False), "sparq-new": (3, True)})
        )
        self.assertEqual(len(fails), 2)


class LiveRepoInvariant(unittest.TestCase):
    """This repo's committed floors must currently pass, and cover the whole tree."""

    @classmethod
    def setUpClass(cls):
        cls.floors = json.loads(PRESENCE_JSON.read_text())["crates"]
        cls.tree = gate.scan()

    def test_committed_floors_pass(self):
        _oks, fails = gate.evaluate(self.floors, self.tree)
        self.assertEqual(fails, [], "bench/coverage-presence.json is stale: " + "; ".join(fails))

    def test_every_crate_on_disk_is_tracked(self):
        # The literal #5140 defect. Re-run `python3 scripts/coverage-presence.py --seed`.
        missing = sorted(set(self.tree) - set(self.floors))
        self.assertEqual(missing, [], f"crates missing a presence floor: {missing}")

    def test_no_floor_for_a_crate_that_no_longer_exists(self):
        stale = sorted(set(self.floors) - set(self.tree))
        self.assertEqual(stale, [], f"floors for removed crates: {stale}")

    # --- vacuity guards --------------------------------------------------------------
    def test_scan_actually_finds_the_crates(self):
        """If scan() ever stops recognising crates, the two invariants above pass by
        comparing two empty sets — the exact vacuous-pass shape this gate guards against."""
        on_disk = {
            d.name for d in CRATES_DIR.iterdir() if d.is_dir() and (d / "Cargo.toml").exists()
        }
        self.assertGreater(len(on_disk), 40, "crates/ walk found implausibly few crates")
        self.assertEqual(set(self.tree), on_disk)

    def test_counts_are_non_vacuous(self):
        # A broken #[test] detector would report 0 everywhere; that direction fails the
        # floors loudly, but assert it here so the cause is obvious rather than a wall of
        # "tests removed?" lines.
        counted = sum(1 for info in self.tree.values() if info["tests"] > 0)
        self.assertGreater(counted, 40, "#[test] detector matched almost nothing")

    def test_lws_wasm_is_gated(self):
        # Named in #5140 as the crate adjacent to #2741's scope; its floor is the
        # regression this issue was filed about.
        self.assertIn("sparq-lws-wasm", self.floors)
        self.assertGreater(self.floors["sparq-lws-wasm"]["min_tests"], 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
