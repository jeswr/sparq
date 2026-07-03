#!/usr/bin/env python3
# [OPUS-4.8] Hermetic unit tests for the change-based CI test-selector
# (bead sq-fmx4u.1, epic sq-fmx4u). Authored by Opus 4.8 (Fable unavailable;
# flag for re-review when Fable returns).
#
# Covers the design §3/§4 golden cases + the bead acceptance criteria (a)-(i):
#   (a) leaf-crate change   => exactly that crate
#   (b) sparq-core change    => all workspace members
#   (c) dev-dep edge         propagates to the dependent's tests
#   (d) optional-dep edge    propagates
#   (e) file move            marks BOTH crates
#   (f) unowned/unmapped     => full  (fail-safe)
#   (g) every §4.1 trigger   => full  (fail-safe)
#   (h) forced internal error=> mode=full, exit 0  (fail-closed)
#   (i) REAL cargo-metadata  closure shape (core root-like, geo leaf-like)
#
# Almost fully hermetic: the synthetic-graph cases build `cargo metadata`-shaped
# dicts in-process (no workspace resolve). Case (i) shells out to
# `cargo metadata --no-deps` and self-SKIPS when cargo is unavailable, so the
# suite never REQUIRES a live cargo (task hermeticity rule).
#
# Run:  python3 scripts/tests/test_ci_select.py   (stdlib only; no pytest needed)

from __future__ import annotations

import importlib.util
import io
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
CI_SELECT = REPO_ROOT / "scripts" / "ci_select.py"


def _load_module():
    spec = importlib.util.spec_from_file_location("ci_select", CI_SELECT)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["ci_select"] = mod
    spec.loader.exec_module(mod)
    return mod


cs = _load_module()

ROOT = "/repo"


def _dep(name, kind=None, optional=False, target=None):
    return {"name": name, "kind": kind, "optional": optional, "target": target, "path": None}


def _pkg(name, reldir, deps=(), source=None):
    return {
        "id": f"path+file://{ROOT}/{reldir}#0.1.0",
        "name": name,
        "source": source,
        "manifest_path": f"{ROOT}/{reldir}/Cargo.toml",
        "dependencies": list(deps),
    }


def _synthetic_meta():
    """A small DAG rooted at `core` (everything depends on core), with `app` a
    reverse-graph leaf. Exercises normal, dev, build and optional edges.

        core  <- parse <- engine <- app
        core  <- devlib (dev-dep of engine)
        core  <- optlib (optional dep of engine)
        core  <- buildlib (build-dep of parse)
    """
    pkgs = [
        _pkg("core", "crates/core", []),
        _pkg("parse", "crates/parse", [_dep("core"), _dep("buildlib", kind="build")]),
        _pkg("engine", "crates/engine",
             [_dep("core"), _dep("parse"), _dep("devlib", kind="dev"),
              _dep("optlib", optional=True)]),
        _pkg("app", "crates/app", [_dep("core"), _dep("engine")]),
        _pkg("devlib", "crates/devlib", [_dep("core")]),
        _pkg("optlib", "crates/optlib", [_dep("core")]),
        _pkg("buildlib", "crates/buildlib", [_dep("core")]),
        # A registry dep sharing NOTHING in-repo: must be ignored as an owner/edge.
        {"id": "registry+x#1.0", "name": "serde", "source": "registry+https://x",
         "manifest_path": "/reg/serde/Cargo.toml", "dependencies": []},
    ]
    member_ids = [p["id"] for p in pkgs if p["source"] is None]
    return {"workspace_root": ROOT, "packages": pkgs, "workspace_members": member_ids}


ALL_MEMBERS = ["app", "buildlib", "core", "devlib", "engine", "optlib", "parse"]


class SyntheticGraphTests(unittest.TestCase):
    def setUp(self):
        self.meta = _synthetic_meta()

    def _select(self, paths, map_entries=None):
        return cs.select(paths, self.meta, map_entries)

    def test_a_leaf_crate_change_is_exactly_that_crate(self):
        sel = self._select(["crates/app/src/lib.rs"])
        self.assertEqual(sel.mode, "selected")
        self.assertEqual(sel.affected, ["app"])

    def test_b_core_change_is_all_members(self):
        sel = self._select(["crates/core/src/dict.rs"])
        self.assertEqual(sel.mode, "selected")
        self.assertEqual(sel.affected, ALL_MEMBERS)

    def test_c_dev_dep_edge_propagates(self):
        # devlib is a DEV-dep of engine; a devlib change must pull engine (+ app).
        sel = self._select(["crates/devlib/src/lib.rs"])
        self.assertEqual(sel.mode, "selected")
        self.assertIn("engine", sel.affected)
        self.assertEqual(sel.affected, ["app", "devlib", "engine"])

    def test_d_optional_dep_edge_propagates(self):
        # optlib is an OPTIONAL dep of engine; included regardless of features.
        sel = self._select(["crates/optlib/src/lib.rs"])
        self.assertEqual(sel.mode, "selected")
        self.assertEqual(sel.affected, ["app", "engine", "optlib"])

    def test_build_dep_edge_propagates(self):
        # buildlib is a BUILD-dep of parse; build kind is included.
        sel = self._select(["crates/buildlib/build_input.rs"])
        self.assertEqual(sel.affected, ["app", "buildlib", "engine", "parse"])

    def test_e_file_move_marks_both_crates(self):
        # --no-renames reports a move as delete+add; BOTH crate dirs attributed.
        sel = self._select(["crates/devlib/moved.rs", "crates/buildlib/moved.rs"])
        self.assertEqual(sel.mode, "selected")
        self.assertEqual(sorted(sel.changed_crates), ["buildlib", "devlib"])
        # parse is reachable ONLY via buildlib, engine via both -> proves union.
        self.assertIn("parse", sel.affected)
        self.assertIn("devlib", sel.affected)

    def test_f_unowned_unmapped_path_is_full(self):
        sel = self._select(["docs/whatever.md"])
        self.assertEqual(sel.mode, "full")
        self.assertIn("unowned", sel.reason)
        self.assertEqual(sel.affected, ALL_MEMBERS)  # full => run ALL crates

    def test_g_every_full_run_trigger_is_full(self):
        triggers = [
            "Cargo.lock",
            "Cargo.toml",
            "rust-toolchain",
            "rust-toolchain.toml",
            ".cargo/config.toml",
            ".github/workflows/ci.yml",
            "scripts/ci_select.py",
            "deny.toml",
            "supply-chain/audits.toml",
            "ci/path-ownership.toml",
        ]
        for path in triggers:
            with self.subTest(path=path):
                sel = self._select([path, "crates/app/src/lib.rs"])
                self.assertEqual(sel.mode, "full", f"{path} should force full")
                self.assertEqual(sel.affected, ALL_MEMBERS)

    def test_crate_cargo_toml_is_owned_not_a_root_trigger(self):
        # A crate manifest is owned by that crate (NOT caught by root Cargo.toml).
        sel = self._select(["crates/app/Cargo.toml"])
        self.assertEqual(sel.mode, "selected")
        self.assertEqual(sel.affected, ["app"])

    def test_longest_prefix_ownership(self):
        # Nested-crate safety: prefix match must not leak across sibling dirs.
        sel = self._select(["crates/engine/benches/x.rs"])
        self.assertEqual(sel.changed_crates, ["engine"])

    def test_json_contract_keys(self):
        sel = self._select(["crates/app/src/lib.rs"])
        obj = sel.to_json_obj()
        self.assertEqual(set(obj), {"mode", "reason", "affected"})


class OwnershipMapTests(unittest.TestCase):
    def setUp(self):
        self.meta = _synthetic_meta()

    def test_safe_entry_is_ignored(self):
        # research/** SAFE + a real crate change => only the crate counts.
        m = [{"pattern": "research/**", "safe": True}]
        sel = cs.select(["research/foo.md", "crates/app/src/lib.rs"], self.meta, m)
        self.assertEqual(sel.mode, "selected")
        self.assertEqual(sel.affected, ["app"])

    def test_safe_only_change_selects_empty(self):
        m = [{"pattern": "research/**", "safe": True}]
        sel = cs.select(["research/foo.md"], self.meta, m)
        self.assertEqual(sel.mode, "selected")
        self.assertEqual(sel.affected, [])

    def test_map_attributes_out_of_crate_path(self):
        # The #1392 refinement: the REAL fetched W3C path is tests/w3c/** and it
        # is read by the conformance crate (here mapped to `engine`).
        m = [{"pattern": "tests/w3c/**", "crates": ["engine"]}]
        sel = cs.select(["tests/w3c/rdf-tests/data.ttl"], self.meta, m)
        self.assertEqual(sel.mode, "selected")
        self.assertEqual(sel.affected, ["app", "engine"])

    def test_map_unknown_crate_fails_safe(self):
        m = [{"pattern": "tests/w3c/**", "crates": ["ghost-crate"]}]
        sel = cs.select(["tests/w3c/x"], self.meta, m)
        self.assertEqual(sel.mode, "full")
        self.assertIn("unknown crate", sel.reason)

    def test_map_malformed_entry_raises(self):
        m = [{"pattern": "tests/w3c/**"}]  # neither safe nor crates
        with self.assertRaises(cs.SelectorError):
            cs.select(["tests/w3c/x"], self.meta, m)


class MapValidityTests(unittest.TestCase):
    # #1392 refinement: validity uses a FETCHED/GENERATED allowlist, not a
    # brittle on-disk check, so `tests/w3c/**` (fetched, gitignored) validates
    # clean on a clean clone.
    def test_fetched_root_validates_when_allowlisted(self):
        members = {"sparq-conformance"}
        entries = [{"pattern": "tests/w3c/**", "crates": ["sparq-conformance"]}]
        with tempfile.TemporaryDirectory() as tmp:  # tests/w3c absent on disk
            problems = cs.validate_map(
                entries, members,
                known_generated_roots={"tests/w3c"}, repo_root=tmp,
            )
        self.assertEqual(problems, [])

    def test_unknown_crate_flagged(self):
        problems = cs.validate_map(
            [{"pattern": "x/**", "crates": ["nope"]}],
            members={"sparq-core"}, known_generated_roots={"x"},
        )
        self.assertTrue(any("unknown crate" in p for p in problems))

    def test_missing_nonallowlisted_root_flagged(self):
        with tempfile.TemporaryDirectory() as tmp:
            problems = cs.validate_map(
                [{"pattern": "nonexistent/**", "safe": True}],
                members=set(), known_generated_roots=set(), repo_root=tmp,
            )
        self.assertTrue(any("does not exist" in p for p in problems))


class FailClosedMainTests(unittest.TestCase):
    """(h) ANY internal error => mode=full, exit 0 — the fail-closed boundary."""

    def _run_main(self, argv):
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = cs.main(argv)
        return code, json.loads(buf.getvalue())

    def _write(self, text, suffix=".json"):
        fd, path = tempfile.mkstemp(suffix=suffix)
        with os.fdopen(fd, "w") as fh:
            fh.write(text)
        self.addCleanup(lambda: os.path.exists(path) and os.remove(path))
        return path

    def test_malformed_metadata_json_fails_full(self):
        meta_path = self._write("{ this is not json ")
        changed = self._write("crates/app/src/lib.rs\n", suffix=".txt")
        code, obj = self._run_main(["--metadata-file", meta_path, "--changed-file", changed,
                                    "--repo-root", ROOT])
        self.assertEqual(code, 0)
        self.assertEqual(obj["mode"], "full")

    def test_missing_metadata_keys_fails_full(self):
        meta_path = self._write(json.dumps({"packages": []}))  # no workspace_root/members
        changed = self._write("crates/app/src/lib.rs\n", suffix=".txt")
        code, obj = self._run_main(["--metadata-file", meta_path, "--changed-file", changed,
                                    "--repo-root", ROOT])
        self.assertEqual(code, 0)
        self.assertEqual(obj["mode"], "full")

    def test_nonexistent_metadata_file_fails_full(self):
        changed = self._write("crates/app/src/lib.rs\n", suffix=".txt")
        code, obj = self._run_main(["--metadata-file", "/no/such/meta.json",
                                    "--changed-file", changed, "--repo-root", ROOT])
        self.assertEqual(code, 0)
        self.assertEqual(obj["mode"], "full")

    def test_schedule_event_is_full(self):
        meta_path = self._write(json.dumps(_synthetic_meta()))
        code, obj = self._run_main(["--event", "schedule", "--metadata-file", meta_path,
                                    "--repo-root", ROOT])
        self.assertEqual(code, 0)
        self.assertEqual(obj["mode"], "full")
        self.assertEqual(obj["affected"], ALL_MEMBERS)

    def test_ci_full_override_is_full(self):
        meta_path = self._write(json.dumps(_synthetic_meta()))
        code, obj = self._run_main(["--full", "--metadata-file", meta_path, "--repo-root", ROOT])
        self.assertEqual(code, 0)
        self.assertEqual(obj["mode"], "full")

    def test_hermetic_end_to_end_selected(self):
        meta_path = self._write(json.dumps(_synthetic_meta()))
        changed = self._write("crates/app/src/lib.rs\n", suffix=".txt")
        code, obj = self._run_main(["--metadata-file", meta_path, "--changed-file", changed,
                                    "--repo-root", ROOT])
        self.assertEqual(code, 0)
        self.assertEqual(obj["mode"], "selected")
        self.assertEqual(obj["affected"], ["app"])


class WiringHookTests(unittest.TestCase):
    """[FABLE-5] sq-fmx4u.3: the hooks the CI wiring consumes — the shadow rollout
    mode, the nextest filterset output, and clean full-mode on non-PR events."""

    def _run_main(self, argv):
        buf = io.StringIO()
        with redirect_stdout(buf):
            code = cs.main(argv)
        return code, json.loads(buf.getvalue())

    def _write(self, text, suffix=".json"):
        fd, path = tempfile.mkstemp(suffix=suffix)
        with os.fdopen(fd, "w") as fh:
            fh.write(text)
        self.addCleanup(lambda: os.path.exists(path) and os.remove(path))
        return path

    def test_push_event_is_clean_full(self):
        # Any event without a PR diff (push, or a future event name) => full by
        # construction, not via the error trap.
        meta_path = self._write(json.dumps(_synthetic_meta()))
        code, obj = self._run_main(["--event", "push", "--metadata-file", meta_path,
                                    "--repo-root", ROOT])
        self.assertEqual(code, 0)
        self.assertEqual(obj["mode"], "full")
        self.assertNotIn("selector error", obj["reason"])
        self.assertEqual(obj["affected"], ALL_MEMBERS)

    def test_shadow_wraps_selected(self):
        # --shadow: the selection is COMPUTED (affected preserved for the report)
        # but the emitted mode is 'shadow', so no guard's `mode == 'selected'`
        # branch can ever fire => nothing skips.
        meta_path = self._write(json.dumps(_synthetic_meta()))
        changed = self._write("crates/app/src/lib.rs\n", suffix=".txt")
        code, obj = self._run_main(["--shadow", "--metadata-file", meta_path,
                                    "--changed-file", changed, "--repo-root", ROOT])
        self.assertEqual(code, 0)
        self.assertEqual(obj["mode"], "shadow")
        self.assertIn("SHADOW (computed mode=selected", obj["reason"])
        self.assertEqual(obj["affected"], ["app"])

    def test_shadow_wraps_full_and_error_uniformly(self):
        # The wrap is uniform: even a computed full / a selector error emits
        # mode=shadow — one downstream rule (shadow is never 'selected').
        meta_path = self._write(json.dumps(_synthetic_meta()))
        code, obj = self._run_main(["--shadow", "--event", "schedule",
                                    "--metadata-file", meta_path, "--repo-root", ROOT])
        self.assertEqual(obj["mode"], "shadow")
        self.assertIn("computed mode=full", obj["reason"])
        code, obj = self._run_main(["--shadow", "--metadata-file", "/no/such/meta.json",
                                    "--changed-file", self._write("x\n", suffix=".txt"),
                                    "--repo-root", ROOT])
        self.assertEqual(code, 0)
        self.assertEqual(obj["mode"], "shadow")
        self.assertIn("selector error", obj["reason"])

    def test_output_file_carries_mode_affected_filterset(self):
        # The $GITHUB_OUTPUT contract the guards + bulk shards consume.
        meta_path = self._write(json.dumps(_synthetic_meta()))
        changed = self._write("crates/app/src/lib.rs\n", suffix=".txt")
        out_path = self._write("", suffix=".out")
        code, obj = self._run_main(["--metadata-file", meta_path, "--changed-file", changed,
                                    "--repo-root", ROOT, "--output-file", out_path])
        self.assertEqual(code, 0)
        with open(out_path, encoding="utf-8") as fh:
            lines = dict(ln.split("=", 1) for ln in fh.read().splitlines() if "=" in ln)
        self.assertEqual(lines["mode"], "selected")
        self.assertEqual(json.loads(lines["affected"]), ["app"])
        self.assertEqual(lines["filterset"], "package(app)")

    def test_filterset_joins_members_with_plus(self):
        self.assertEqual(
            cs.filterset(cs.Selection(mode="selected", reason="", affected=["a", "b"])),
            "package(a) + package(b)",
        )
        self.assertEqual(cs.filterset(cs.Selection(mode="full", reason="", affected=[])), "")


class RealMetadataShapeTests(unittest.TestCase):
    """(i) Pinned against the REAL workspace metadata: core is root-like, geo is
    leaf-like, and closure(geo) is a subset of closure(core) (structural: geo
    depends on core, so anything depending on geo depends on core)."""

    @classmethod
    def setUpClass(cls):
        cls.meta = None
        if shutil.which("cargo") is None:
            return
        try:
            out = subprocess.run(
                ["cargo", "metadata", "--no-deps", "--format-version", "1"],
                cwd=str(REPO_ROOT), check=True, capture_output=True, text=True, timeout=180,
            )
            cls.meta = json.loads(out.stdout)
        except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError, ValueError):
            cls.meta = None

    def setUp(self):
        if self.meta is None:
            self.skipTest("cargo metadata unavailable (hermetic skip)")

    def test_core_root_like_geo_leaf_like(self):
        ws = cs.parse_workspace(self.meta)
        self.assertIn("sparq-core", ws.members)
        self.assertIn("sparq-geo", ws.members)
        core = cs.reverse_closure("sparq-core", ws.reverse_adj)
        geo = cs.reverse_closure("sparq-geo", ws.reverse_adj)
        n = len(ws.members)
        # core is root-like: a large majority of the workspace depends on it.
        self.assertGreaterEqual(len(core), n // 2)
        # geo is leaf-like: only a small handful depend on it.
        self.assertLess(len(geo), n // 4)
        # structural invariants that survive crate additions:
        self.assertIn("sparq-geo", core)               # geo depends on core
        self.assertNotIn("sparq-core", geo)            # core does not depend on geo
        self.assertTrue(geo.issubset(core))            # closure(geo) subset of closure(core)
        self.assertGreater(len(core), len(geo))        # core strictly more root-like

    def test_selected_mode_on_real_leaf_change(self):
        sel = cs.select(["crates/sparq-geo/src/lib.rs"], self.meta)
        self.assertEqual(sel.mode, "selected")
        self.assertIn("sparq-geo", sel.affected)
        self.assertNotIn("sparq-parse", sel.affected)  # parse does not depend on geo


if __name__ == "__main__":
    unittest.main(verbosity=2)
