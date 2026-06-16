#!/usr/bin/env python3
# [OPUS-4.8] Hermetic tests for the proactive merge-gate scripts G1/G2 (beads
# sq-ncvq.4 + sq-ncvq.5, epic sq-ncvq). Authored by Opus 4.8 (Fable unavailable;
# flag for re-review when Fable returns).
#
# Hermetic w.r.t. git/network: imports scripts/gate-new-crate.py +
# scripts/gate-api-skill.py and drives their pure evaluate()/main() entry points
# against FIXTURE diff listings. NO live git and NO subprocess — the
# evaluate()-level tests inject every git/subprocess fact (crate stub status,
# bench registration, `pub`-diff heuristic) via the *_overrides kwargs, and
# main() is driven with --changed-files fixtures + --dry-run so it never shells
# out. [OPUS-4.8] (Caveat: the main() smoke tests call main() WITHOUT overrides,
# so they may still consult the in-repo bench/benchmarks.toml on disk — that is a
# committed file, not git/network state, so the runs stay deterministic.)
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

    def test_copied_crate_counts_as_added(self):
        # [OPUS-4.8] `git diff -C` reports a newly-introduced path as a copy
        # (`C`/`C100`, "C<score>\t<old>\t<new>"); the destination must still be
        # treated as added so a copy can't evade new-crate detection.
        changed, added = g1.parse_status_lines(
            ["C100\tcrates/sparq-old/Cargo.toml\tcrates/sparq-new/Cargo.toml"]
        )
        self.assertIn("crates/sparq-new/Cargo.toml", added)
        self.assertEqual(g1.added_crates(added), ["sparq-new"])


# --------------------------------------------------------------------------- #
# G2 — public-api → skill
# --------------------------------------------------------------------------- #
class G2Test(unittest.TestCase):
    # [OPUS-4.8] G2 now fires ONLY on a NET `pub `-item signature change in a
    # published crate's src/** (no more blanket binding-crate trip). The
    # evaluate()-level tests inject the pub_api_changed verdict via pub_overrides
    # so they stay hermetic (no live git); the _scan_pub_diff tests exercise the
    # added/removed-multiset logic directly.

    def test_server_pub_change_without_skill_fails(self):
        # A real pub-item change in a binding crate's src, no SKILL → FAIL.
        path = "crates/sparq-server/src/routes.rs"
        ok, hits = g2.evaluate(
            [path], labels=[], base="main", pub_overrides={path: True}
        )
        self.assertFalse(ok)
        self.assertIn(path, hits)

    def test_server_pub_change_with_skill_passes(self):
        path = "crates/sparq-server/src/routes.rs"
        ok, _ = g2.evaluate(
            [path, "skills/http-server/SKILL.md"],
            labels=[],
            base="main",
            pub_overrides={path: True},
        )
        self.assertTrue(ok)

    def test_binding_crate_comment_only_change_passes(self):
        # [OPUS-4.8] REGRESSION (blocked #250): a comment-/attribute-only edit to a
        # BINDING crate's src is NOT a public-surface change — it must PASS even
        # without a SKILL.md. pub_overrides={...: False} models "no net pub change".
        path = "crates/sparq-cli/src/main.rs"
        ok, hits = g2.evaluate(
            [path], labels=[], base="main", pub_overrides={path: False}
        )
        self.assertTrue(ok)
        self.assertEqual(hits, [])

    def test_ci_only_change_passes(self):
        # [OPUS-4.8] REGRESSION (#244 framing): a PR that touches NO crates/*/src/**
        # — only CI workflows / tests / non-src files — can never reach the `pub`
        # check, so G2 always passes. No pub_overrides needed: these paths never hit
        # git because _CRATE_SRC_RE rejects them.
        changed = [
            ".github/workflows/feature-matrix.yml",
            "crates/sparq-reason/tests/explain_owl.rs",
            "compliance/asvs/gap-register.md",
        ]
        ok, hits = g2.evaluate(changed, labels=[], base="main")
        self.assertTrue(ok)
        self.assertEqual(hits, [])

    def test_pub_item_relocation_does_not_trip(self):
        # [OPUS-4.8] REGRESSION (mis-fired #244): a pure relocation of a pub item —
        # the identical signature added once and removed once within a file — is
        # net-zero and must NOT count as a public-API change. Drive the pure
        # _scan_pub_diff + pub_api_changed multiset logic directly.
        diff = [
            "--- a/crates/sparq-reason/src/explain.rs",
            "+++ b/crates/sparq-reason/src/explain.rs",
            "@@ -257,1 +257,1 @@",
            "+pub fn n3_proof_tree(",
            "@@ -327,1 +400,0 @@",
            "-pub fn n3_proof_tree(",
        ]
        added, removed = g2._scan_pub_diff(diff)
        self.assertEqual(sorted(added), sorted(removed))  # cancels out
        # And a genuinely NEW export does trip (added with no matching removal).
        diff2 = [
            "+++ b/crates/sparq-core/src/store.rs",
            "+pub fn brand_new_export() {}",
        ]
        added2, removed2 = g2._scan_pub_diff(diff2)
        self.assertNotEqual(sorted(added2), sorted(removed2))

    def test_pub_item_regex_matches_all_item_forms_excludes_restricted(self):
        # [OPUS-4.8] The pub-item pattern must match every exported FORM
        # (fn/struct/enum/trait/const/type/mod/use) but NOT restricted
        # visibilities (`pub(crate)`/`pub(super)`/`pub(in …)`) nor a struct
        # `pub` field. Guard against silent drift of the source pattern.
        for exported in (
            "+pub fn foo()",
            "-pub use crate::X;",
            "+    pub struct S",
            "+pub enum E {",
            "-pub trait T {",
            "+pub const C: u8 = 0;",
            "+pub type Alias = u8;",
            "+pub mod m;",
        ):
            self.assertTrue(g2._PUB_ITEM_RE.match(exported), exported)
        for not_an_export in (
            "+pub(crate) fn foo()",
            "-pub(super) struct S",
            "+    pub(in crate::a) const X: u8 = 0;",
            "+    pub name: String,",  # struct field, not an item
            "+publicize();",  # `pub` not a whole word
        ):
            self.assertFalse(g2._PUB_ITEM_RE.match(not_an_export), not_an_export)

    def test_cli_change_suppressed_by_skill_not_needed_label(self):
        # Even a real pub change is suppressed by the escape-hatch label.
        path = "crates/sparq-cli/src/main.rs"
        ok, _ = g2.evaluate(
            [path],
            labels=["skill-not-needed"],
            base="main",
            pub_overrides={path: True},
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
