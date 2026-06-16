#!/usr/bin/env python3
# [OPUS-4.8] Hermetic tests for the periodic drift scanner (bead sq-ncvq.11,
# epic sq-ncvq). Authored by Opus 4.8 (Fable unavailable; flag for re-review
# when Fable returns).
#
# Fully hermetic: imports scripts/drift-scan.py and runs each scanner against
# FIXTURE repo layouts built in a tmp dir — NO live `gh` / `git` / network. We
# exercise the scanners directly + main(--dry-run) (which makes no gh calls) +
# the dedup-key stability contract.
#
# Run:  python3 scripts/tests/test_drift_scan.py
# (stdlib only; no pytest required — mirrors test_flow_on.py.)

from __future__ import annotations

import importlib.util
import io
import json
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
DRIFT_SCAN = REPO_ROOT / "scripts" / "drift-scan.py"


def _load_module():
    spec = importlib.util.spec_from_file_location("drift_scan", DRIFT_SCAN)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["drift_scan"] = mod
    spec.loader.exec_module(mod)
    return mod


drift_scan = _load_module()


# --------------------------------------------------------------------------- #
# Fixture-repo builder
# --------------------------------------------------------------------------- #
def _write(root: Path, rel: str, content: str = "") -> None:
    p = root / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(content, encoding="utf-8")


def make_crate(root: Path, name: str, *, public: bool = True) -> None:
    body = f'[package]\nname = "{name}"\n'
    if not public:
        body += "publish = false\n"
    _write(root, f"crates/{name}/Cargo.toml", body)


def make_registry(root: Path, sources: list[str], ids: list[str] | None = None) -> None:
    """Write a minimal bench/benchmarks.toml with the given `source` lines (and
    optionally `id` lines for the dashboard scanner)."""
    ids = ids or []
    lines = []
    n = max(len(sources), len(ids))
    for i in range(n):
        lines.append("[[benchmark]]")
        if i < len(ids):
            lines.append(f'id          = "{ids[i]}"')
        if i < len(sources):
            lines.append(f'source      = "{sources[i]}"')
        lines.append("")
    _write(root, "bench/benchmarks.toml", "\n".join(lines))


def make_dashboard(root: Path, featured_tokens: list[str]) -> None:
    aliases = ", ".join(f"'{t}'" for t in featured_tokens)
    js = (
        "var FEATURED_SUITES = [\n"
        f"  {{ key: 'X', title: 'X', aliases: [{aliases}] }}\n"
        "];\n"
    )
    _write(root, "bench/dashboard/dashboard.js", js)


def make_skill(root: Path, surface: str, text: str) -> None:
    _write(root, f"skills/{surface}/SKILL.md", text)


# --------------------------------------------------------------------------- #
# bench-missing (§5.A)
# --------------------------------------------------------------------------- #
class BenchMissingTest(unittest.TestCase):
    def _root(self) -> Path:
        root = Path(self._tmp.name)
        make_crate(root, "sparq-nlq")
        make_crate(root, "sparq-engine")  # exempt: query stack
        make_crate(root, "sparq-bench")  # exempt: the harness itself
        # registry references engine (via cli harness) but NOT nlq.
        make_registry(root, sources=["crates/sparq-cli/src/main.rs"])
        return root

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)

    def test_flags_unbenched_crate(self):
        items = drift_scan.scan_bench_missing(self._root())
        keys = {it.dedup_key for it in items}
        self.assertIn("bench-missing:sparq-nlq", keys)

    def test_does_not_flag_exempt_query_stack(self):
        items = drift_scan.scan_bench_missing(self._root())
        keys = {it.dedup_key for it in items}
        self.assertNotIn("bench-missing:sparq-engine", keys)
        self.assertNotIn("bench-missing:sparq-bench", keys)

    def test_registered_crate_not_flagged(self):
        root = Path(self._tmp.name)
        make_crate(root, "sparq-geo")
        make_registry(root, sources=["bench/geo/run.sh; crates/sparq-geo/examples/bench_geo.rs"])
        items = drift_scan.scan_bench_missing(root)
        self.assertEqual([], [it for it in items if it.dedup_key == "bench-missing:sparq-geo"])


# --------------------------------------------------------------------------- #
# skill-missing (§5.B)
# --------------------------------------------------------------------------- #
class SkillMissingTest(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.root = Path(self._tmp.name)

    def test_flags_public_crate_in_no_skill(self):
        make_crate(self.root, "sparq-orphan", public=True)
        make_crate(self.root, "sparq-known", public=True)
        make_skill(self.root, "known", "covers sparq-known nicely")
        items = drift_scan.scan_skill_missing(self.root)
        keys = {it.dedup_key for it in items}
        self.assertIn("skill-missing:sparq-orphan", keys)
        self.assertNotIn("skill-missing:sparq-known", keys)

    def test_private_crate_exempt(self):
        make_crate(self.root, "sparq-internal", public=False)
        # no skill names it, but it's private → not a public surface.
        items = drift_scan.scan_skill_missing(self.root)
        keys = {it.dedup_key for it in items}
        self.assertNotIn("skill-missing:sparq-internal", keys)

    def test_mention_anywhere_counts(self):
        make_crate(self.root, "sparq-sim", public=True)
        # named only in the top-level index skill — still counts as covered.
        make_skill(self.root, "", "index mentions sparq-sim")  # skills/SKILL.md
        items = drift_scan.scan_skill_missing(self.root)
        self.assertNotIn("skill-missing:sparq-sim", {it.dedup_key for it in items})


# --------------------------------------------------------------------------- #
# explain-asymmetry (§5.C)
# --------------------------------------------------------------------------- #
class ExplainAsymmetryTest(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.root = Path(self._tmp.name)

    def test_flags_when_rust_has_explain_but_wasm_does_not(self):
        _write(self.root, "crates/sparq-engine/src/explain.rs", "pub fn explain() {}")
        _write(self.root, "crates/sparq-server/src/http.rs", "// /explain route")
        _write(self.root, "crates/sparq-wasm/src/lib.rs", "pub fn query() {}")  # no explain
        items = drift_scan.scan_explain_asymmetry(self.root)
        self.assertEqual(["explain-asymmetry:wasm"], [it.dedup_key for it in items])

    def test_no_flag_when_wasm_exports_explain(self):
        _write(self.root, "crates/sparq-engine/src/explain.rs", "pub fn explain() {}")
        _write(self.root, "crates/sparq-wasm/src/lib.rs", "pub fn explain() {}")
        items = drift_scan.scan_explain_asymmetry(self.root)
        self.assertEqual([], items)

    def test_no_flag_when_rust_lacks_explain(self):
        _write(self.root, "crates/sparq-wasm/src/lib.rs", "pub fn query() {}")
        items = drift_scan.scan_explain_asymmetry(self.root)
        self.assertEqual([], items)


# --------------------------------------------------------------------------- #
# dashboard-row (§5.D)
# --------------------------------------------------------------------------- #
class DashboardRowTest(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.root = Path(self._tmp.name)

    def test_flags_unfeatured_family(self):
        make_registry(
            self.root,
            sources=["s", "s"],
            ids=["zk-commit-throughput", "lubm"],
        )
        make_dashboard(self.root, featured_tokens=["lubm"])  # zk NOT featured
        items = drift_scan.scan_dashboard_row(self.root)
        keys = {it.dedup_key for it in items}
        self.assertIn("dashboard-row:zk", keys)
        self.assertNotIn("dashboard-row:lubm", keys)

    def test_family_dedup(self):
        # two zk benches → ONE dashboard-row:zk item.
        make_registry(
            self.root,
            sources=["s", "s"],
            ids=["zk-commit-throughput", "zk-compose-gates"],
        )
        make_dashboard(self.root, featured_tokens=["lubm"])
        items = drift_scan.scan_dashboard_row(self.root)
        zk = [it for it in items if it.dedup_key == "dashboard-row:zk"]
        self.assertEqual(1, len(zk))

    def test_exempt_families_skipped(self):
        make_registry(
            self.root,
            sources=["s", "s"],
            ids=["serve-spikes", "cli-bench-suite"],
        )
        make_dashboard(self.root, featured_tokens=["lubm"])
        items = drift_scan.scan_dashboard_row(self.root)
        self.assertEqual([], items)


# --------------------------------------------------------------------------- #
# conformance-split (§5.E)
# --------------------------------------------------------------------------- #
class ConformanceSplitTest(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.root = Path(self._tmp.name)

    def test_flags_crate_local_ratchet(self):
        make_crate(self.root, "sparq-shacl")
        _write(self.root, "crates/sparq-shacl/tests/w3c_core.rs", "// ratchet")
        make_crate(self.root, "sparq-geo")
        _write(self.root, "crates/sparq-geo/tests/ogc_compliance_ratchet.rs", "// ratchet")
        items = drift_scan.scan_conformance_split(self.root)
        keys = {it.dedup_key for it in items}
        self.assertIn("conformance-split:sparq-shacl", keys)
        self.assertIn("conformance-split:sparq-geo", keys)

    def test_central_scoreboard_exempt(self):
        make_crate(self.root, "sparq-conformance")
        _write(self.root, "crates/sparq-conformance/tests/conformance_smoke.rs", "// ok")
        items = drift_scan.scan_conformance_split(self.root)
        self.assertEqual([], items)

    def test_non_conformance_tests_ignored(self):
        make_crate(self.root, "sparq-shacl")
        _write(self.root, "crates/sparq-shacl/tests/constraints.rs", "// unit test")
        items = drift_scan.scan_conformance_split(self.root)
        self.assertEqual([], items)


# --------------------------------------------------------------------------- #
# dedup-key stability + issue body / JSON contract
# --------------------------------------------------------------------------- #
class DedupKeyStabilityTest(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.root = Path(self._tmp.name)
        make_crate(self.root, "sparq-nlq")
        make_registry(self.root, sources=["crates/sparq-cli/src/main.rs"])

    def test_dedup_key_is_stable_across_runs(self):
        keys1 = {it.dedup_key for it in drift_scan.scan_all(self.root)}
        keys2 = {it.dedup_key for it in drift_scan.scan_all(self.root)}
        self.assertEqual(keys1, keys2)
        self.assertIn("bench-missing:sparq-nlq", keys1)

    def test_dedup_key_is_content_free(self):
        # No PR number / date / volatile token in the key — purely class:subject.
        for it in drift_scan.scan_all(self.root):
            self.assertRegex(it.dedup_key, r"^[a-z-]+:[\w.-]+$")

    def test_issue_body_embeds_marker(self):
        items = drift_scan.scan_all(self.root)
        it = items[0]
        body = drift_scan.build_issue_body(it)
        self.assertIn(drift_scan.key_marker(it.dedup_key), body)
        self.assertIn("🤖 SPARQ agent", body)

    def test_key_marker_matches_open_issue_search_substring(self):
        # The search substring (used for idempotency) must be a substring of the
        # full HTML-comment marker, so a found issue's body always matches.
        key = "bench-missing:sparq-nlq"
        self.assertIn(f"drift-key: {key}", drift_scan.key_marker(key))


# --------------------------------------------------------------------------- #
# main(--dry-run) end-to-end (no gh)
# --------------------------------------------------------------------------- #
class MainDryRunTest(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.root = Path(self._tmp.name)
        make_crate(self.root, "sparq-nlq")
        make_crate(self.root, "sparq-shacl")
        _write(self.root, "crates/sparq-shacl/tests/w3c_core.rs", "// ratchet")
        make_registry(self.root, sources=["crates/sparq-cli/src/main.rs"])

    def test_dry_run_prints_and_writes_json(self):
        json_path = Path(self._tmp.name) / "out.json"
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = drift_scan.main(
                ["--root", str(self.root), "--dry-run", "--json", str(json_path)]
            )
        self.assertEqual(0, rc)
        out = buf.getvalue()
        self.assertIn("bench-missing", out)
        self.assertIn("conformance-split", out)
        payload = json.loads(json_path.read_text())
        self.assertGreaterEqual(payload["drift_count"], 2)
        classes = {it["class"] for it in payload["items"]}
        self.assertIn("bench-missing", classes)
        self.assertIn("conformance-split", classes)

    def test_no_drift_clean_repo(self):
        clean = tempfile.TemporaryDirectory()
        self.addCleanup(clean.cleanup)
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = drift_scan.main(["--root", clean.name, "--dry-run"])
        self.assertEqual(0, rc)
        self.assertIn("no drift detected", buf.getvalue())


if __name__ == "__main__":
    unittest.main()
