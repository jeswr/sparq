#!/usr/bin/env python3
# [OPUS-4.8] sq-tmyw: hermetic tests for scripts/check-sbom-purl-canonical.py.
# Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns).
#
# Hermetic w.r.t. git/network: imports the check module and drives its pure evaluate()
# + the file parser against in-tmpdir fixtures. NO subprocess, NO live git. A final test
# generates the REAL workspace SBOM through scripts/sbom-normalize.jq (the exact
# publication transform) and asserts every purl is canonical — i.e. the live invariant
# the CI gate enforces. That last test is skipped (not failed) if cargo-cyclonedx / jq
# are unavailable, so the suite stays runnable in a bare environment while still proving
# the real invariant where the toolchain exists (as on the CI runner).
#
# Run:  python3 scripts/tests/test_sbom_purl_canonical.py
# (stdlib only; no pytest required — also discoverable by `pytest`.)

from __future__ import annotations

import importlib.util
import json
import shutil
import subprocess
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


chk = _load("check_sbom_purl_canonical", "check-sbom-purl-canonical.py")


def _sbom(purls: list[str], *, root_purl: str | None = None) -> dict:
    """Minimal CycloneDX-shaped doc carrying the given component purls (and an optional
    metadata.component purl, to exercise the metadata recursion)."""
    doc: dict = {
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "components": [{"type": "library", "purl": p} for p in purls],
    }
    if root_purl is not None:
        doc["metadata"] = {"component": {"type": "application", "purl": root_purl}}
    return doc


class TestEvaluate(unittest.TestCase):
    def test_all_canonical_passes(self):
        ok, lines = chk.evaluate(
            ["pkg:cargo/serde@1.0.0", "pkg:cargo/axum@0.8.4"]
        )
        self.assertTrue(ok, lines)
        self.assertIn("canonical", lines[0])

    def test_download_url_qualifier_fails(self):
        ok, lines = chk.evaluate(
            ["pkg:cargo/sparq-server@0.1.0?download_url=file://./crates/sparq-server"]
        )
        self.assertFalse(ok)
        self.assertTrue(any("NON-CANONICAL" in l for l in lines))

    def test_src_subpath_fragment_fails(self):
        # The exact GS-7 shape the normalizer had to be extended for.
        ok, lines = chk.evaluate(["pkg:cargo/sparq-cli@0.1.0#src/main.rs"])
        self.assertFalse(ok)
        self.assertTrue(any("NON-CANONICAL" in l for l in lines))

    def test_future_decoration_qualifier_fails(self):
        # A hypothetical NEW qualifier a future cargo-cyclonedx could add — the whole
        # point of the backstop: the normalizer pattern wouldn't match it, but this does.
        ok, lines = chk.evaluate(["pkg:cargo/serde@1.0.0?repository_url=https://x"])
        self.assertFalse(ok)

    def test_non_cargo_purl_is_unexpected(self):
        ok, lines = chk.evaluate(["pkg:npm/fzstd@0.1.1"])
        self.assertFalse(ok)
        self.assertTrue(any("non-cargo" in l for l in lines))

    def test_empty_fails(self):
        ok, lines = chk.evaluate([])
        self.assertFalse(ok)


class TestFileLevel(unittest.TestCase):
    def test_metadata_component_purl_is_checked(self):
        # A leak hiding ONLY on the root metadata.component must be caught.
        with tempfile.TemporaryDirectory() as d:
            f = Path(d) / "x.cdx.json"
            f.write_text(
                json.dumps(
                    _sbom(
                        ["pkg:cargo/serde@1.0.0"],
                        root_purl="pkg:cargo/sparq-server@0.1.0?download_url=file://.",
                    )
                )
            )
            purls = chk._walk_purls(json.loads(f.read_text()))
            ok, _ = chk.evaluate(purls)
            self.assertFalse(ok)

    def test_vex_like_doc_has_no_purls(self):
        # A pure-VEX doc (vulnerabilities, no components) carries no purls -> skipped by main.
        doc = {"bomFormat": "CycloneDX", "specVersion": "1.5",
               "vulnerabilities": [{"id": "RUSTSEC-2024-0436"}]}
        self.assertEqual(chk._walk_purls(doc), [])


class TestLiveWorkspaceSBOM(unittest.TestCase):
    """Generate the REAL normalized workspace SBOM and assert it passes — the exact
    invariant the CI gate checks. Skipped if the toolchain is absent."""

    def test_real_normalized_sbom_is_canonical(self):
        if not shutil.which("cargo-cyclonedx") or not shutil.which("jq"):
            self.skipTest("cargo-cyclonedx / jq not available")
        normalize = REPO_ROOT / "scripts" / "sbom-normalize.jq"
        with tempfile.TemporaryDirectory() as d:
            # Generate into the repo (cargo-cyclonedx writes next to each Cargo.toml),
            # normalize, copy out, then clean the generated crate SBOMs.
            subprocess.run(
                ["cargo", "cyclonedx", "--all", "--format", "json",
                 "--spec-version", "1.5"],
                cwd=REPO_ROOT, check=True, capture_output=True,
            )
            try:
                judged = 0
                for crate in ("sparq-server", "sparq-cli"):
                    src = REPO_ROOT / "crates" / crate / f"{crate}.cdx.json"
                    if not src.exists():
                        continue
                    dst = Path(d) / f"{crate}.cdx.json"
                    out = subprocess.run(
                        ["jq", "-f", str(normalize), str(src)],
                        check=True, capture_output=True, text=True,
                    ).stdout
                    dst.write_text(out)
                    purls = chk._walk_purls(json.loads(out))
                    ok, lines = chk.evaluate(purls)
                    self.assertTrue(ok, f"{crate}: {lines}")
                    judged += 1
                self.assertGreater(judged, 0, "no released-binary SBOM was generated")
            finally:
                for f in REPO_ROOT.glob("crates/**/*.cdx.json"):
                    f.unlink()


if __name__ == "__main__":
    unittest.main(verbosity=2)
