#!/usr/bin/env python3
# [OPUS-4.8] Hermetic tests for the proactive merge-gate scripts G1/G2 (beads
# sq-ncvq.4 + sq-ncvq.5, epic sq-ncvq). Authored by Opus 4.8 (Fable unavailable;
# flag for re-review when Fable returns).
#
# Fully hermetic: imports scripts/gate-new-crate.py + scripts/gate-api-skill.py
# and drives their pure evaluate()/main() entry points against FIXTURE diff
# listings. NO live git — the fact lookups that would hit disk/git (crate stub
# status, bench registration, `pub`-diff heuristic) are injected via the
# *_overrides kwargs, and main() is driven with --changed-files fixtures written
# to a temp dir + --dry-run so it never shells out.
#
# Run:  python3 scripts/tests/test_gates.py
# (stdlib only; no pytest required — also discoverable by `pytest`.)

from __future__ import annotations

import importlib.util
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


g1 = _load("gate_new_crate", "gate-new-crate.py")
g2 = _load("gate_api_skill", "gate-api-skill.py")


def _statused(added: list[str], modified: list[str] | None = None) -> list[str]:
    """Build a `git diff --name-status`-style fixture listing."""
    lines = [f"A\t{p}" for p in added]
    lines += [f"M\t{p}" for p in (modified or [])]
    return lines


def _write(tmp: Path, name: str, lines: list[str]) -> str:
    p = tmp / name
    p.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return str(p)


# --------------------------------------------------------------------------- #
# G1 — new-crate-completeness
# --------------------------------------------------------------------------- #
class G1Test(unittest.TestCase):
    def test_new_crate_with_nothing_else_fails(self):
        changed, added = g1.parse_status_lines(
            _statused(["crates/sparq-foo/Cargo.toml"])
        )
        violations = g1.evaluate(
            changed,
            added,
            stub_overrides={"sparq-foo": False},
            bench_overrides={"sparq-foo": False},
        )
        self.assertEqual(len(violations), 1)
        crate, missing = violations[0]
        self.assertEqual(crate, "sparq-foo")
        # All three artifacts are missing: README, bench, SKILL.
        self.assertEqual(len(missing), 3)
        joined = " ".join(missing)
        self.assertIn("README", joined)
        self.assertIn("benchmark", joined)
        self.assertIn("SKILL.md", joined)

    def test_new_crate_with_bench_readme_skill_passes(self):
        added = ["crates/sparq-foo/Cargo.toml", "crates/sparq-foo/README.md"]
        modified = ["skills/sparq-foo/SKILL.md", "bench/benchmarks.toml"]
        changed, added_paths = g1.parse_status_lines(_statused(added, modified))
        violations = g1.evaluate(
            changed,
            added_paths,
            stub_overrides={"sparq-foo": False},
            bench_overrides={"sparq-foo": True},
        )
        self.assertEqual(violations, [])

    def test_stub_crate_needs_only_readme(self):
        # publish = false → bench + SKILL are waived; README still required.
        added = ["crates/sparq-foo/Cargo.toml", "crates/sparq-foo/README.md"]
        changed, added_paths = g1.parse_status_lines(_statused(added))
        violations = g1.evaluate(
            changed,
            added_paths,
            stub_overrides={"sparq-foo": True},
            bench_overrides={"sparq-foo": False},
        )
        self.assertEqual(violations, [])

    def test_stub_crate_missing_readme_still_fails(self):
        changed, added = g1.parse_status_lines(
            _statused(["crates/sparq-foo/Cargo.toml"])
        )
        violations = g1.evaluate(
            changed, added, stub_overrides={"sparq-foo": True}
        )
        self.assertEqual(len(violations), 1)
        _, missing = violations[0]
        self.assertEqual(len(missing), 1)
        self.assertIn("README", missing[0])

    def test_no_new_crate_passes(self):
        changed, added = g1.parse_status_lines(
            _statused([], ["crates/sparq-cli/src/main.rs"])
        )
        self.assertEqual(g1.evaluate(changed, added), [])

    def test_changed_not_added_cargo_does_not_trigger(self):
        # An edit to an EXISTING crate's Cargo.toml is not a "new crate".
        changed, added = g1.parse_status_lines(
            _statused([], ["crates/sparq-cli/Cargo.toml"])
        )
        self.assertEqual(g1.added_crates(added), [])
        self.assertEqual(g1.evaluate(changed, added), [])


# --------------------------------------------------------------------------- #
# G2 — public-api → skill
# --------------------------------------------------------------------------- #
class G2Test(unittest.TestCase):
    def test_server_src_change_without_skill_fails(self):
        changed = ["crates/sparq-server/src/routes.rs"]
        ok, hits = g2.evaluate(changed, labels=[], base="main")
        self.assertFalse(ok)
        self.assertIn("crates/sparq-server/src/routes.rs", hits)

    def test_server_src_change_with_skill_passes(self):
        changed = [
            "crates/sparq-server/src/routes.rs",
            "skills/http-server/SKILL.md",
        ]
        ok, _ = g2.evaluate(changed, labels=[], base="main")
        self.assertTrue(ok)

    def test_cli_change_suppressed_by_skill_not_needed_label(self):
        changed = ["crates/sparq-cli/src/main.rs"]
        ok, _ = g2.evaluate(
            changed, labels=["skill-not-needed"], base="main"
        )
        self.assertTrue(ok)

    def test_published_crate_pub_change_without_skill_fails(self):
        # A pub API change in a published (non-binding) crate is in scope.
        path = "crates/sparq-core/src/store.rs"
        ok, hits = g2.evaluate(
            [path], labels=[], base="main", pub_overrides={path: True}
        )
        # crate_is_published reads disk; sparq-core has no publish=false, so it's
        # published. The injected pub_override makes it a pub change.
        self.assertFalse(ok)
        self.assertIn(path, hits)

    def test_published_crate_nonpub_change_passes(self):
        # A non-pub internal change is NOT a public surface.
        path = "crates/sparq-core/src/internal.rs"
        ok, hits = g2.evaluate(
            [path], labels=[], base="main", pub_overrides={path: False}
        )
        self.assertTrue(ok)
        self.assertEqual(hits, [])

    def test_non_surface_change_passes(self):
        ok, hits = g2.evaluate(
            ["research/notes.md", "bench/watdiv/run.sh"], labels=[], base="main"
        )
        self.assertTrue(ok)
        self.assertEqual(hits, [])

    def test_py_binding_change_without_skill_fails(self):
        ok, hits = g2.evaluate(
            ["crates/sparq-py/src/lib.rs"], labels=[], base="main"
        )
        self.assertFalse(ok)
        self.assertIn("crates/sparq-py/src/lib.rs", hits)


# --------------------------------------------------------------------------- #
# main() smoke (hermetic, via --changed-files + --dry-run; no git/network)
# --------------------------------------------------------------------------- #
class MainSmokeTest(unittest.TestCase):
    def test_g1_main_dry_run_reports_violation_but_exits_zero(self):
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            f = _write(tmp, "diff.txt", _statused(["crates/sparq-zzznew/Cargo.toml"]))
            rc = g1.main(["--dry-run", "--changed-files", f])
            self.assertEqual(rc, 0)  # dry-run never fails

    def test_g2_main_dry_run_on_clean_diff_passes(self):
        with tempfile.TemporaryDirectory() as d:
            tmp = Path(d)
            f = _write(tmp, "diff.txt", ["research/foo.md"])
            rc = g2.main(["--dry-run", "--changed-files", f])
            self.assertEqual(rc, 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
