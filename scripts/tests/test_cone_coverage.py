#!/usr/bin/env python3
# [SONNET-4.6] Hermetic unit tests for cone_coverage.py (bead sq-6vshe.8).
#
# Mirrors the test-file pattern in scripts/tests/test_ci_select.py:
#   - imports the module under test via importlib.util (no installed package)
#   - synthetic cargo-metadata dicts (no real workspace needed)
#   - stdlib only; no pytest required (but pytest works fine)
#
# Covers:
#   (a) test_changed_leaf_crate_gives_exact_cone
#         A change to a leaf crate → cone = {leaf} ∪ its transitive rev-deps.
#   (b) test_changed_workspace_file_gives_full_run
#         A change to Cargo.toml (§4.1 trigger) → mode=full.
#   (c) test_changed_dev_dep_gives_correct_closure
#         A dev-dep-only change propagates to the dependent crate's cone.
#   (d) test_shadow_mode_default
#         shadow=True in the emitted doc by default (enforce=False).
#   (e) test_enforce_mode
#         With enforce=True the doc records enforce=True / shadow=False.
#   (f) test_compute_cone_unowned_path_gives_full_run
#         An unowned, unmapped path → fail-safe mode=full.
#   (g) test_shadow_report_cone_vs_full
#         shadow_report labels in-cone crates "cone-measured" and outside-cone
#         crates "inherited (unchanged cone): PASS".
#   (h) test_shadow_report_divergence_detected
#         An outside-cone crate below its floor is flagged as a divergence.
#   (i) test_shadow_report_full_mode_labels_all_measured
#         When mode=full, every crate is "cone-measured".
#
# Run:  python3 scripts/tests/test_cone_coverage.py
# Run with pytest:  pytest scripts/tests/test_cone_coverage.py -v

from __future__ import annotations

import importlib.util
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
CONE_COVERAGE = REPO_ROOT / "scripts" / "cone_coverage.py"


def _load_module():
    spec = importlib.util.spec_from_file_location("cone_coverage", CONE_COVERAGE)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["cone_coverage"] = mod
    spec.loader.exec_module(mod)  # type: ignore[union-attr]
    return mod


cc = _load_module()

ROOT = "/repo"

# ---------- helpers matching test_ci_select.py style -------------------------

def _dep(name, kind=None, optional=False):
    return {"name": name, "kind": kind, "optional": optional, "target": None, "path": None}


def _pkg(name, reldir, deps=(), source=None):
    return {
        "id": f"path+file://{ROOT}/{reldir}#0.1.0",
        "name": name,
        "source": source,
        "manifest_path": f"{ROOT}/{reldir}/Cargo.toml",
        "dependencies": list(deps),
    }


def _synthetic_meta(pkgs):
    """Build a minimal cargo-metadata dict from a list of _pkg() entries."""
    member_ids = [p["id"] for p in pkgs]
    return {
        "workspace_root": ROOT,
        "workspace_members": member_ids,
        "packages": pkgs,
    }


class TestComputeConeLeafCrate(unittest.TestCase):
    """(a) Changed leaf crate → cone = {leaf} ∪ transitive dependents."""

    def setUp(self):
        # core ← parse ← engine ← app
        # parse ← buildlib (build dep)
        self.pkgs = [
            _pkg("core", "crates/core", []),
            _pkg("parse", "crates/parse", [_dep("core")]),
            _pkg("engine", "crates/engine", [_dep("core"), _dep("parse")]),
            _pkg("app", "crates/app", [_dep("engine")]),
        ]
        self.meta = _synthetic_meta(self.pkgs)

    def test_changed_leaf_crate_gives_exact_cone(self):
        # Change only the `app` leaf (nothing depends on it).
        changed = [f"crates/app/src/main.rs"]
        doc = cc.compute_cone(changed, self.meta)
        self.assertEqual(doc["mode"], "cone")
        self.assertEqual(doc["cone_crates"], ["app"])
        # app has no dependents so the cone is just {app}.

    def test_changed_middle_crate_includes_dependents(self):
        # Change `parse`; engine and app depend on it transitively.
        changed = ["crates/parse/src/lib.rs"]
        doc = cc.compute_cone(changed, self.meta)
        self.assertEqual(doc["mode"], "cone")
        self.assertIn("parse", doc["cone_crates"])
        self.assertIn("engine", doc["cone_crates"])
        self.assertIn("app", doc["cone_crates"])
        # core is NOT a dependent of parse — it is a dependency of parse.
        self.assertNotIn("core", doc["cone_crates"])

    def test_changed_root_crate_includes_all(self):
        # Change `core`; everything depends on core.
        changed = ["crates/core/src/lib.rs"]
        doc = cc.compute_cone(changed, self.meta)
        self.assertEqual(doc["mode"], "cone")
        self.assertIn("core", doc["cone_crates"])
        self.assertIn("parse", doc["cone_crates"])
        self.assertIn("engine", doc["cone_crates"])
        self.assertIn("app", doc["cone_crates"])


class TestFullRunTriggers(unittest.TestCase):
    """(b) §4.1 full-run triggers → mode=full."""

    def setUp(self):
        self.pkgs = [_pkg("mylib", "crates/mylib", [])]
        self.meta = _synthetic_meta(self.pkgs)

    def test_changed_workspace_file_gives_full_run(self):
        changed = ["Cargo.toml"]
        doc = cc.compute_cone(changed, self.meta)
        self.assertEqual(doc["mode"], "full")
        self.assertIn("mylib", doc["cone_crates"])

    def test_cargo_lock_gives_full_run(self):
        doc = cc.compute_cone(["Cargo.lock"], self.meta)
        self.assertEqual(doc["mode"], "full")

    def test_github_workflows_gives_full_run(self):
        doc = cc.compute_cone([".github/workflows/ci.yml"], self.meta)
        self.assertEqual(doc["mode"], "full")

    def test_scripts_gives_full_run(self):
        doc = cc.compute_cone(["scripts/coverage.sh"], self.meta)
        self.assertEqual(doc["mode"], "full")

    def test_rust_toolchain_gives_full_run(self):
        doc = cc.compute_cone(["rust-toolchain.toml"], self.meta)
        self.assertEqual(doc["mode"], "full")


class TestDevDepClosure(unittest.TestCase):
    """(c) Dev-dep change propagates to the dependent's cone."""

    def setUp(self):
        # testlib is a dev-dep of engine. A change to testlib should include engine
        # (because engine's dev tests run against testlib) and engine's rev-deps.
        pkgs = [
            _pkg("testlib", "crates/testlib", []),
            _pkg("engine", "crates/engine", [_dep("testlib", kind="dev")]),
            _pkg("app", "crates/app", [_dep("engine")]),
        ]
        self.meta = _synthetic_meta(pkgs)

    def test_changed_dev_dep_gives_correct_closure(self):
        changed = ["crates/testlib/src/lib.rs"]
        doc = cc.compute_cone(changed, self.meta)
        self.assertEqual(doc["mode"], "cone")
        # testlib itself must be in the cone.
        self.assertIn("testlib", doc["cone_crates"])
        # engine dev-depends on testlib → engine is in the cone.
        self.assertIn("engine", doc["cone_crates"])
        # app depends on engine → app is also in the cone.
        self.assertIn("app", doc["cone_crates"])


class TestShadowMode(unittest.TestCase):
    """(d) shadow=True by default."""

    def setUp(self):
        self.pkgs = [_pkg("lib", "crates/lib", [])]
        self.meta = _synthetic_meta(self.pkgs)

    def test_shadow_mode_default(self):
        doc = cc.compute_cone(["crates/lib/src/lib.rs"], self.meta)
        # shadow key is always True from compute_cone() since this PR is shadow-only.
        self.assertTrue(doc["shadow"])

    def test_shadow_mode_in_cli_output(self):
        # Invoke main() in compute-cone mode without --enforce; check shadow=True.
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as changed_f:
            changed_f.write("crates/lib/src/lib.rs\n")
            changed_path = changed_f.name
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as meta_f:
            json.dump(self.meta, meta_f)
            meta_path = meta_f.name
        out_path = changed_path + ".cone.json"
        try:
            rc = cc.main([
                "--mode", "compute-cone",
                "--changed-file", changed_path,
                "--metadata-file", meta_path,
                "--output", out_path,
            ])
            self.assertEqual(rc, 0)
            with open(out_path) as fh:
                doc = json.load(fh)
            self.assertTrue(doc.get("shadow"))
            self.assertFalse(doc.get("enforce"))
        finally:
            os.unlink(changed_path)
            os.unlink(meta_path)
            if os.path.exists(out_path):
                os.unlink(out_path)


class TestEnforceMode(unittest.TestCase):
    """(e) With --enforce, enforce=True / shadow=False."""

    def setUp(self):
        self.pkgs = [_pkg("lib", "crates/lib", [])]
        self.meta = _synthetic_meta(self.pkgs)

    def test_enforce_mode(self):
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as changed_f:
            changed_f.write("crates/lib/src/lib.rs\n")
            changed_path = changed_f.name
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as meta_f:
            json.dump(self.meta, meta_f)
            meta_path = meta_f.name
        out_path = changed_path + ".cone.json"
        try:
            rc = cc.main([
                "--mode", "compute-cone",
                "--changed-file", changed_path,
                "--metadata-file", meta_path,
                "--output", out_path,
                "--enforce",
            ])
            self.assertEqual(rc, 0)
            with open(out_path) as fh:
                doc = json.load(fh)
            self.assertTrue(doc.get("enforce"))
            self.assertFalse(doc.get("shadow"))
        finally:
            os.unlink(changed_path)
            os.unlink(meta_path)
            if os.path.exists(out_path):
                os.unlink(out_path)


class TestUnownedPathFullRun(unittest.TestCase):
    """(f) Unowned, unmapped path → fail-safe mode=full."""

    def setUp(self):
        self.pkgs = [_pkg("mylib", "crates/mylib", [])]
        self.meta = _synthetic_meta(self.pkgs)

    def test_compute_cone_unowned_path_gives_full_run(self):
        changed = ["some/unknown/path/file.txt"]
        doc = cc.compute_cone(changed, self.meta)
        self.assertEqual(doc["mode"], "full")
        self.assertIn("mylib", doc["cone_crates"])


class TestShadowReport(unittest.TestCase):
    """(g) shadow_report labels crates correctly."""

    def setUp(self):
        self.pkgs = [
            _pkg("lib-a", "crates/lib-a", []),
            _pkg("lib-b", "crates/lib-b", []),
        ]
        self.meta = _synthetic_meta(self.pkgs)
        self.cone_doc = {
            "mode": "cone",
            "cone_crates": ["lib-a"],
            "changed_crates": ["lib-a"],
            "all_members": ["lib-a", "lib-b"],
            "reason": "test cone",
            "shadow": True,
        }
        self.coverage_summary = {
            "tier": "per-commit",
            "crates": {
                "lib-a": {"measured": True, "lines_pct": 85.0, "lines_covered": 85, "lines_total": 100, "seconds": 1},
                "lib-b": {"measured": True, "lines_pct": 90.0, "lines_covered": 90, "lines_total": 100, "seconds": 1},
            },
        }
        self.floor_doc = {
            "crates": {
                "lib-a": {"floor": 80},
                "lib-b": {"floor": 88},
            }
        }

    def test_shadow_report_cone_vs_full(self):
        report = cc.shadow_report(self.cone_doc, self.coverage_summary, self.floor_doc)
        rows = {r["crate"]: r for r in report["rows"]}

        # lib-a is in the cone → cone-measured.
        self.assertEqual(rows["lib-a"]["label"], "cone-measured")
        self.assertEqual(rows["lib-a"]["status"], "measured")
        self.assertTrue(rows["lib-a"]["in_cone"])

        # lib-b is outside the cone → inherited.
        self.assertEqual(rows["lib-b"]["label"], "inherited (unchanged cone): PASS")
        self.assertEqual(rows["lib-b"]["status"], "inherited")
        self.assertFalse(rows["lib-b"]["in_cone"])

        # No divergences (lib-b is above its floor).
        self.assertEqual(report["divergences"], [])

    def test_shadow_report_inherited_count(self):
        report = cc.shadow_report(self.cone_doc, self.coverage_summary, self.floor_doc)
        self.assertEqual(report["inherited_count"], 1)
        self.assertEqual(report["cone_size"], 1)
        self.assertEqual(report["total_crates"], 2)


class TestShadowReportDivergence(unittest.TestCase):
    """(h) An outside-cone crate below its floor is flagged as a divergence."""

    def test_shadow_report_divergence_detected(self):
        cone_doc = {
            "mode": "cone",
            "cone_crates": ["lib-a"],
            "changed_crates": ["lib-a"],
            "all_members": ["lib-a", "lib-b"],
            "reason": "test cone",
            "shadow": True,
        }
        # lib-b is OUTSIDE the cone but BELOW its floor.
        coverage_summary = {
            "tier": "per-commit",
            "crates": {
                "lib-a": {"measured": True, "lines_pct": 85.0, "lines_covered": 85, "lines_total": 100, "seconds": 1},
                "lib-b": {"measured": True, "lines_pct": 70.0, "lines_covered": 70, "lines_total": 100, "seconds": 1},
            },
        }
        floor_doc = {
            "crates": {
                "lib-a": {"floor": 80},
                "lib-b": {"floor": 88},   # lib-b floor=88 but measured=70.0 → below floor
            }
        }

        report = cc.shadow_report(cone_doc, coverage_summary, floor_doc)

        # lib-b should be flagged as a divergence.
        divs = report["divergences"]
        self.assertEqual(len(divs), 1)
        self.assertEqual(divs[0]["crate"], "lib-b")
        self.assertEqual(divs[0]["type"], "outside-cone-below-floor")

    def test_shadow_report_divergence_log_written(self):
        cone_doc = {
            "mode": "cone",
            "cone_crates": ["lib-a"],
            "changed_crates": ["lib-a"],
            "all_members": ["lib-a", "lib-b"],
            "reason": "test",
            "shadow": True,
        }
        coverage_summary = {
            "crates": {
                "lib-a": {"measured": True, "lines_pct": 85.0, "lines_covered": 85, "lines_total": 100, "seconds": 1},
                "lib-b": {"measured": True, "lines_pct": 70.0, "lines_covered": 70, "lines_total": 100, "seconds": 1},
            }
        }
        floor_doc = {"crates": {"lib-a": {"floor": 80}, "lib-b": {"floor": 88}}}

        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            log_path = f.name
        try:
            cc.shadow_report(cone_doc, coverage_summary, floor_doc, divergence_log_path=log_path)
            with open(log_path) as fh:
                logged = json.load(fh)
            self.assertEqual(len(logged["divergences"]), 1)
            self.assertEqual(logged["divergences"][0]["crate"], "lib-b")
        finally:
            os.unlink(log_path)


class TestShadowReportFullMode(unittest.TestCase):
    """(i) When mode=full, every crate is labeled 'cone-measured'."""

    def test_shadow_report_full_mode_labels_all_measured(self):
        cone_doc = {
            "mode": "full",
            "cone_crates": ["lib-a", "lib-b"],
            "changed_crates": [],
            "all_members": ["lib-a", "lib-b"],
            "reason": "Cargo.lock changed",
            "shadow": True,
        }
        coverage_summary = {
            "crates": {
                "lib-a": {"measured": True, "lines_pct": 85.0, "lines_covered": 85, "lines_total": 100, "seconds": 1},
                "lib-b": {"measured": True, "lines_pct": 90.0, "lines_covered": 90, "lines_total": 100, "seconds": 1},
            }
        }
        report = cc.shadow_report(cone_doc, coverage_summary, None)

        rows = {r["crate"]: r for r in report["rows"]}
        # Both should be "cone-measured" since mode=full.
        for crate in ("lib-a", "lib-b"):
            self.assertEqual(rows[crate]["label"], "cone-measured",
                             f"{crate} should be cone-measured in full mode")
            self.assertTrue(rows[crate]["in_cone"])
        self.assertEqual(report["inherited_count"], 0)


class TestShadowReportShardFiltering(unittest.TestCase):
    """(j) [SONNET-4.6] Crates in all_members but absent from shard summary are omitted from rows.

    In the CI matrix each coverage-measure shard writes coverage-summary.json only
    for the crates it measured. shadow_report() must operate on the INTERSECTION of
    cone_doc["all_members"] and coverage_summary["crates"] so that:
      - unmeasured crates produce no row (avoiding misleading divergence output),
      - all derived counts (total_crates, cone_size) match the row set,
      - a compact not_measured_in_shard count is reported instead.
    """

    def _make_cone(self, all_members, cone_crates):
        return {
            "mode": "cone",
            "cone_crates": sorted(cone_crates),
            "changed_crates": sorted(cone_crates),
            "all_members": sorted(all_members),
            "reason": "test cone",
            "shadow": True,
        }

    def test_unmeasured_crates_omitted_from_rows(self):
        """lib-c is in all_members but NOT in the shard summary → no row emitted."""
        cone_doc = self._make_cone(
            all_members=["lib-a", "lib-b", "lib-c"],
            cone_crates=["lib-a"],
        )
        coverage_summary = {
            "crates": {
                "lib-a": {"measured": True, "lines_pct": 85.0, "lines_covered": 85, "lines_total": 100, "seconds": 1},
                "lib-b": {"measured": True, "lines_pct": 90.0, "lines_covered": 90, "lines_total": 100, "seconds": 1},
                # lib-c intentionally absent — not measured in this shard
            }
        }
        report = cc.shadow_report(cone_doc, coverage_summary, None)

        row_crates = {r["crate"] for r in report["rows"]}
        self.assertNotIn("lib-c", row_crates,
                         "unmeasured-in-shard crate must not appear as a row")

    def test_compact_count_reflects_omitted_crates(self):
        """not_measured_in_shard count equals the number of omitted all_members crates."""
        cone_doc = self._make_cone(
            all_members=["lib-a", "lib-b", "lib-c", "lib-d"],
            cone_crates=["lib-a"],
        )
        coverage_summary = {
            "crates": {
                "lib-a": {"measured": True, "lines_pct": 85.0, "lines_covered": 85, "lines_total": 100, "seconds": 1},
                "lib-b": {"measured": True, "lines_pct": 90.0, "lines_covered": 90, "lines_total": 100, "seconds": 1},
                # lib-c, lib-d absent
            }
        }
        report = cc.shadow_report(cone_doc, coverage_summary, None)

        self.assertEqual(report["not_measured_in_shard"], 2,
                         "compact count must equal number of all_members crates absent from shard summary")

    def test_counts_match_row_set(self):
        """total_crates and cone_size must match rows, not workspace totals."""
        cone_doc = self._make_cone(
            all_members=["lib-a", "lib-b", "lib-c"],  # 3 workspace crates
            cone_crates=["lib-a"],
        )
        coverage_summary = {
            "crates": {
                # Only 2 of 3 measured in this shard; lib-a is in cone
                "lib-a": {"measured": True, "lines_pct": 85.0, "lines_covered": 85, "lines_total": 100, "seconds": 1},
                "lib-b": {"measured": True, "lines_pct": 90.0, "lines_covered": 90, "lines_total": 100, "seconds": 1},
            }
        }
        report = cc.shadow_report(cone_doc, coverage_summary, None)

        # total_crates must be 2 (shard-measured), not 3 (workspace)
        self.assertEqual(report["total_crates"], 2,
                         "total_crates must reflect shard-measured count, not workspace total")
        # cone_size: lib-a is in cone AND in shard summary → 1
        self.assertEqual(report["cone_size"], 1,
                         "cone_size must be over the shard-measured set")
        # Rows must have exactly 2 entries
        self.assertEqual(len(report["rows"]), 2,
                         "row count must match total_crates")
        # not_measured_in_shard accounts for the gap
        self.assertEqual(report["not_measured_in_shard"], 1)

    def test_no_false_divergences_for_unmeasured_crates(self):
        """A crate absent from the shard summary must NOT produce a divergence entry,
        even if it would be below-floor (it was simply not measured in this shard)."""
        cone_doc = self._make_cone(
            all_members=["lib-a", "lib-b"],
            cone_crates=["lib-a"],
        )
        # lib-b is outside the cone but also NOT in this shard's summary
        coverage_summary = {
            "crates": {
                "lib-a": {"measured": True, "lines_pct": 85.0, "lines_covered": 85, "lines_total": 100, "seconds": 1},
                # lib-b absent
            }
        }
        floor_doc = {"crates": {"lib-b": {"floor": 99}}}  # impossibly high floor
        report = cc.shadow_report(cone_doc, coverage_summary, floor_doc)

        self.assertEqual(report["divergences"], [],
                         "an unmeasured-in-shard crate must not produce a false divergence")


class TestEmptyAllMembersFallback(unittest.TestCase):
    """(k) [SONNET-4.6] When cone.json is produced via the error path, all_members
    may be empty. shadow_report() must fall back to the shard summary keys so the
    report still surfaces measured crates and any divergences.

    This guards against the silent-monitoring-blindness path: without the fallback,
    the intersection of {} and measured_in_shard is {} and no rows are emitted at
    all, hiding real below-floor crates from the shadow report.
    """

    def _make_error_cone(self):
        """Cone doc that mirrors compute_cone()'s error-path output (all_members=[])."""
        return {
            "mode": "full",
            "cone_crates": [],
            "changed_crates": [],
            "all_members": [],   # error path: empty
            "reason": "cone-coverage selector error, failing to full run: <exc>",
            "shadow": True,
        }

    def test_empty_all_members_falls_back_to_shard_summary(self):
        """Rows are produced for shard-measured crates even when all_members is empty."""
        cone_doc = self._make_error_cone()
        coverage_summary = {
            "crates": {
                "lib-a": {"measured": True, "lines_pct": 85.0, "lines_covered": 85, "lines_total": 100, "seconds": 1},
                "lib-b": {"measured": True, "lines_pct": 90.0, "lines_covered": 90, "lines_total": 100, "seconds": 1},
            }
        }
        report = cc.shadow_report(cone_doc, coverage_summary, None)

        row_crates = {r["crate"] for r in report["rows"]}
        self.assertIn("lib-a", row_crates,
                      "shard-measured crate must appear in rows when all_members is empty")
        self.assertIn("lib-b", row_crates,
                      "shard-measured crate must appear in rows when all_members is empty")
        self.assertEqual(len(report["rows"]), 2,
                         "all shard-measured crates must produce rows via the fallback")

    def test_empty_all_members_all_rows_cone_measured(self):
        """When all_members is empty and mode=full (the error path), all shard
        crates are treated as in-cone (cone-measured) — consistent with full-mode
        semantics — and no spurious divergences are produced."""
        cone_doc = self._make_error_cone()
        coverage_summary = {
            "crates": {
                "lib-a": {"measured": True, "lines_pct": 85.0, "lines_covered": 85, "lines_total": 100, "seconds": 1},
                "lib-b": {"measured": True, "lines_pct": 90.0, "lines_covered": 90, "lines_total": 100, "seconds": 1},
            }
        }
        floor_doc = {"crates": {"lib-a": {"floor": 80.0}, "lib-b": {"floor": 80.0}}}
        report = cc.shadow_report(cone_doc, coverage_summary, floor_doc)

        # In full mode every crate is in-cone; divergence detection is for outside-cone
        # crates only, so no divergences must fire.
        self.assertEqual(report["divergences"], [],
                         "full-mode error-path fallback must not produce spurious divergences")
        # But rows must be present — the monitoring is NOT blind.
        labels = {r["label"] for r in report["rows"]}
        self.assertEqual(labels, {"cone-measured"},
                         "all fallback rows must be labelled cone-measured in full mode")

    def test_empty_all_members_not_measured_count_is_zero(self):
        """When falling back to shard summary, not_measured_in_shard must be 0
        (effective_members == measured_in_shard, so their difference is empty)."""
        cone_doc = self._make_error_cone()
        coverage_summary = {
            "crates": {
                "lib-a": {"measured": True, "lines_pct": 85.0, "lines_covered": 85, "lines_total": 100, "seconds": 1},
            }
        }
        report = cc.shadow_report(cone_doc, coverage_summary, None)

        self.assertEqual(report["not_measured_in_shard"], 0,
                         "no crates are missing from the shard when falling back to its keys")


class TestGetChangedPathsStrip(unittest.TestCase):
    """(l) [SONNET-4.6] _get_changed_paths() strips leading/trailing whitespace
    from each line in a hermetic --changed-file so callers receive clean paths."""

    def test_leading_trailing_whitespace_stripped(self):
        """Lines with leading/trailing spaces or \\r are returned stripped."""
        import tempfile

        content = "  crates/lib-a/src/lib.rs  \n\r\ncrates/lib-b/src/lib.rs\r\n  \n"
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as fh:
            fh.write(content)
            tmp_path = fh.name

        try:
            import argparse
            args = argparse.Namespace(changed_file=tmp_path, base=None, head="HEAD")
            paths = cc._get_changed_paths(args, None)
        finally:
            import os
            os.unlink(tmp_path)

        self.assertEqual(paths, [
            "crates/lib-a/src/lib.rs",
            "crates/lib-b/src/lib.rs",
        ], "returned paths must have no leading/trailing whitespace")

    def test_blank_lines_excluded(self):
        """Blank lines (including whitespace-only) are excluded from output."""
        import tempfile

        content = "crates/lib-a/src/lib.rs\n   \n\ncrates/lib-b/src/lib.rs\n"
        with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as fh:
            fh.write(content)
            tmp_path = fh.name

        try:
            import argparse
            args = argparse.Namespace(changed_file=tmp_path, base=None, head="HEAD")
            paths = cc._get_changed_paths(args, None)
        finally:
            import os
            os.unlink(tmp_path)

        self.assertEqual(len(paths), 2, "whitespace-only lines must be excluded")


if __name__ == "__main__":
    unittest.main(verbosity=2)
