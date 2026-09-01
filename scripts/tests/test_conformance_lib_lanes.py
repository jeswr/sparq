#!/usr/bin/env python3
# [OPUS-5] sq-mwnko — structural pin: every FEATURE-GATED lib unit-test module in
# `sparq-conformance` must have a CI leg that actually builds and runs it.
#
# THE BUG THIS EXISTS FOR. `cargo test -p sparq-conformance --features F --test T`
# builds ONLY the integration-test binary `T`. The crate's own `#[cfg(test)]` modules
# inside `src/` live in the LIB test binary, which that command never builds — so a
# unit test behind an opt-in feature can be dead in CI while looking perfectly wired.
# Measured consequence (sq-p6yb7): `inference::sparql_entail::ql_tests::
# gate_rejections_classify_into_the_taxonomy` sat RED on origin/main against a stale
# `contains("B5")` expectation — sq-pbz04.3.2's gate-side alternation desugaring had
# changed the rejection reason to the UNION/B1 message — and nothing noticed, because
# ci.yml's QL lane ran three `--test` targets and no `--lib`.
#
# WHY THE EXISTING GUARD DOES NOT COVER IT. `scripts/check-feature-test-execution.py`
# is the structural guard for this class, but its own header records the hole:
# "Inline ``#[cfg(feature = "F")] #[test] fn...`` inside ``src/`` files are not
# detected". It scans `crates/*/tests/**` files and Cargo `required-features` targets
# against feature-matrix legs / coverage.sh; it does not read ci.yml at all, and the
# conformance lanes deliberately live in ci.yml (they need fetched W3C fixtures or the
# loopback server stack, so `.github/feature-matrix.d/sparq-conformance.yml` documents
# them as out of scope for a hermetic matrix leg). This test closes that gap for the
# one crate the bug was found in; the workspace-wide generalisation is follow-up work.
#
# WHAT IS ASSERTED
#   1. Every `src/` module that (a) contains a `#[cfg(test)]` / `#[cfg(all(test,
#      feature = ...))]` test module AND (b) only compiles with some cargo feature ON
#      is covered by at least one ci.yml `cargo test -p sparq-conformance ... --lib`
#      invocation whose ACTIVATED feature set (the transitive closure through the
#      crate's own `[features]` table) is a superset of what that module needs.
#   2. Non-vacuity: the detector is run against synthetic inputs in both directions —
#      a lane WITHOUT `--lib` must be reported as uncovered, and the same lane WITH
#      `--lib` must be reported as covered. A detector that always returns "covered"
#      (the silent-false-green failure mode this whole file guards) fails here.
#
# Hermetic: stdlib only (no PyYAML, no cargo, no network, no gh).
# Run:  python3 scripts/tests/test_conformance_lib_lanes.py

from __future__ import annotations

import re
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
CI_YML = REPO_ROOT / ".github" / "workflows" / "ci.yml"
CRATE_DIR = REPO_ROOT / "crates" / "sparq-conformance"
CRATE_SRC = CRATE_DIR / "src"
CRATE_TOML = CRATE_DIR / "Cargo.toml"

# `src/main.rs` and `src/bin/` are BINARY roots, not part of the lib target, so a
# `#[cfg(test)]` module there is not built by `--lib` and is out of this pin's scope.
_SKIP_REL = ("main.rs",)
_SKIP_DIRS = ("bin",)

# `#[cfg(all(test, feature = "x"))]` / `#[cfg(all(feature = "x", test))]` — a test
# module gated INLINE on a feature.
_INLINE_TEST_CFG = re.compile(r"#\[cfg\(all\((?=[^)]*\btest\b)([^)]*)\)\)\]")
# A plain `#[cfg(test)]` module: the gating (if any) comes from the module DECLARATION
# in the parent `lib.rs` / `mod.rs`.
_PLAIN_TEST_CFG = re.compile(r"#\[cfg\(test\)\]")
_FEATURE_IN_CFG = re.compile(r'feature\s*=\s*"([^"]+)"')

# `#[cfg(feature = "x")]` immediately above a `mod y;` declaration.
_MOD_CFG = re.compile(r'^\s*#\[cfg\(feature\s*=\s*"([^"]+)"\)\]\s*$')
_MOD_DECL = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;")
_ATTR_OR_COMMENT = re.compile(r"^\s*(?://|#!?\[|/\*|\*)")

_FEATURES_HEADER = re.compile(r"^\s*\[features\]\s*$")
_TOML_SECTION = re.compile(r"^\s*\[")
_FEATURE_ENTRY = re.compile(r'^\s*([A-Za-z0-9_-]+)\s*=\s*\[(.*)\]\s*$')


# --------------------------------------------------------------------------------
# Cargo feature activation
# --------------------------------------------------------------------------------
def parse_feature_table(cargo_toml_text: str) -> dict[str, list[str]]:
    """`[features]` as {feature: [implied bare feature names]}.

    `dep:foo` and `crate/feat` entries are dropped: they activate something in
    ANOTHER crate, which can never gate a module in THIS crate's `src/`.
    """
    table: dict[str, list[str]] = {}
    in_features = False
    for line in cargo_toml_text.splitlines():
        if _FEATURES_HEADER.match(line):
            in_features = True
            continue
        if in_features and _TOML_SECTION.match(line) and not _FEATURES_HEADER.match(line):
            in_features = False
            continue
        if not in_features:
            continue
        m = _FEATURE_ENTRY.match(line)
        if not m:
            continue
        implied = [
            v.strip().strip('"')
            for v in m.group(2).split(",")
            if v.strip()
        ]
        table[m.group(1)] = [
            v for v in implied if v and "/" not in v and not v.startswith("dep:")
        ]
    return table


def activate(features: set[str], table: dict[str, list[str]]) -> set[str]:
    """Transitive closure of `features` through the crate's own feature table."""
    seen: set[str] = set()
    stack = list(features)
    while stack:
        f = stack.pop()
        if f in seen:
            continue
        seen.add(f)
        stack.extend(table.get(f, ()))
    return seen


# --------------------------------------------------------------------------------
# Which src/ modules carry feature-gated lib tests
# --------------------------------------------------------------------------------
def module_gates(src_dir: Path) -> dict[str, str]:
    """{module name -> feature that gates its `mod` declaration}.

    Built from every module-root file (`lib.rs` and `*/mod.rs`). Module names are
    unique across this crate; `test_module_names_are_unique` pins that assumption.
    """
    gates: dict[str, str] = {}
    roots = [src_dir / "lib.rs"] + sorted(src_dir.glob("*/mod.rs"))
    for root in roots:
        if not root.is_file():
            continue
        pending: str | None = None
        for line in root.read_text(encoding="utf-8").splitlines():
            m = _MOD_CFG.match(line)
            if m:
                pending = m.group(1)
                continue
            decl = _MOD_DECL.match(line)
            if decl:
                if pending:
                    gates[decl.group(1)] = pending
                pending = None
                continue
            if line.strip() and not _ATTR_OR_COMMENT.match(line):
                pending = None
    return gates


def lib_test_files(src_dir: Path) -> list[Path]:
    out = []
    for path in sorted(src_dir.rglob("*.rs")):
        rel = path.relative_to(src_dir)
        if rel.as_posix() in _SKIP_REL or rel.parts[0] in _SKIP_DIRS:
            continue
        out.append(path)
    return out


def gated_test_modules(src_dir: Path) -> dict[str, set[str]]:
    """{source path (repo-relative posix) -> features required to compile its tests}.

    Only files whose test modules are gated at all are returned; an ungated
    `#[cfg(test)]` module is built by the default `cargo test --workspace` shards and
    needs no dedicated executor.
    """
    gates = module_gates(src_dir)
    found: dict[str, set[str]] = {}
    for path in lib_test_files(src_dir):
        text = path.read_text(encoding="utf-8")
        required: set[str] = set()

        # (a) inline `#[cfg(all(test, feature = "..."))]`
        for m in _INLINE_TEST_CFG.finditer(text):
            required |= set(_FEATURE_IN_CFG.findall(m.group(1)))

        # (b) a plain `#[cfg(test)]` in a module whose DECLARATION is feature-gated
        if _PLAIN_TEST_CFG.search(text):
            rel = path.relative_to(src_dir)
            names = [p.removesuffix(".rs") for p in rel.parts]
            required |= {gates[n] for n in names if n in gates}

        if required:
            found[path.relative_to(REPO_ROOT).as_posix()] = required
    return found


# --------------------------------------------------------------------------------
# Which ci.yml invocations build the lib test binary
# --------------------------------------------------------------------------------
def lib_lane_feature_sets(ci_yml_text: str) -> list[set[str]]:
    """Every `cargo test -p sparq-conformance ... --lib` lane's declared features."""
    # Join shell line-continuations so a wrapped command reads as one line.
    joined = re.sub(r"\\\n\s*", " ", ci_yml_text)
    out: list[set[str]] = []
    for line in joined.splitlines():
        if "cargo test" not in line or "-p sparq-conformance" not in line:
            continue
        if not re.search(r"(?<!\S)--lib(?!\S)", line):
            continue
        feats: set[str] = set()
        for m in re.finditer(r"--features[= ]([^\s]+)", line):
            feats |= {f for f in m.group(1).split(",") if f}
        out.append(feats)
    return out


def uncovered(src_dir: Path, cargo_toml_text: str, ci_yml_text: str) -> dict[str, set[str]]:
    """Gated lib-test modules with no `--lib` executor. Empty dict == healthy."""
    table = parse_feature_table(cargo_toml_text)
    lanes = [activate(f, table) for f in lib_lane_feature_sets(ci_yml_text)]
    gaps: dict[str, set[str]] = {}
    for path, required in gated_test_modules(src_dir).items():
        if not any(required <= lane for lane in lanes):
            gaps[path] = required
    return gaps


# --------------------------------------------------------------------------------
class TestFeatureTable(unittest.TestCase):
    def test_forwarding_features_are_expanded(self) -> None:
        table = parse_feature_table(CRATE_TOML.read_text(encoding="utf-8"))
        # `federation-descriptors = ["service-loopback", "sparq-server/..."]` — the
        # in-crate implication is kept, the cross-crate one dropped.
        self.assertIn("service-loopback", table.get("federation-descriptors", []))
        self.assertNotIn(
            "sparq-server/federation-descriptors",
            table.get("federation-descriptors", []),
        )
        self.assertIn(
            "service-loopback",
            activate({"federation-descriptors"}, table),
            "federation-descriptors must activate service-loopback transitively",
        )


class TestDetection(unittest.TestCase):
    def test_module_names_are_unique(self) -> None:
        """`module_gates` keys by bare module name — pin that that is unambiguous."""
        names: dict[str, str] = {}
        for path in lib_test_files(CRATE_SRC):
            rel = path.relative_to(CRATE_SRC)
            if rel.name in ("lib.rs", "mod.rs"):
                continue
            name = rel.name.removesuffix(".rs")
            self.assertNotIn(
                name,
                names,
                f"duplicate module name {name!r} ({rel} vs {names.get(name)}) — "
                "module_gates() keys by bare name and would conflate them",
            )
            names[name] = rel.as_posix()

    def test_the_known_gated_modules_are_all_detected(self) -> None:
        """The five modules the sq-mwnko audit enumerated, by both detection paths."""
        found = gated_test_modules(CRATE_SRC)
        expected = {
            # inline `#[cfg(all(test, feature = "ql-experimental"))]`
            "crates/sparq-conformance/src/inference/sparql_entail.rs": {"ql-experimental"},
            # plain `#[cfg(test)]` inside a feature-gated `mod` declaration
            "crates/sparq-conformance/src/inference/dl_suite.rs": {"dl-direct"},
            "crates/sparq-conformance/src/jsonld_bench.rs": {"jsonld-suite"},
            "crates/sparq-conformance/src/http_protocol.rs": {"service-loopback"},
            "crates/sparq-conformance/src/sd_gsp.rs": {"federation-descriptors"},
        }
        for path, feats in expected.items():
            self.assertIn(path, found, f"{path} lost its feature-gated test module")
            self.assertEqual(found[path], feats, f"{path} gating features changed")

    def test_ungated_test_modules_are_not_reported(self) -> None:
        """`compare.rs` has a plain `#[cfg(test)]` and no gate — the workspace shards
        already run it, so demanding a dedicated `--lib` lane for it would be noise."""
        self.assertNotIn(
            "crates/sparq-conformance/src/compare.rs", gated_test_modules(CRATE_SRC)
        )


class TestNonVacuity(unittest.TestCase):
    """The detector must swing BOTH ways on synthetic input.

    This is the mutation the file exists for: a `uncovered()` that returned `{}`
    unconditionally would pass `test_every_gated_lib_test_has_an_executor` below
    while guarding nothing.
    """

    TOML = '[features]\nfoo = ["bar"]\nbar = ["dep:x"]\n'

    def _ci(self, lib: bool) -> str:
        flag = " --lib" if lib else ""
        return (
            "      - name: lane\n"
            "        run: |\n"
            f"          cargo test -p sparq-conformance --features foo{flag} \\\n"
            "            --test thing -- --nocapture\n"
        )

    def test_lane_without_lib_is_reported(self) -> None:
        lanes = lib_lane_feature_sets(self._ci(lib=False))
        self.assertEqual(lanes, [], "a lane with no --lib must not count as an executor")

    def test_lane_with_lib_is_recognised_across_a_continuation(self) -> None:
        lanes = lib_lane_feature_sets(self._ci(lib=True))
        self.assertEqual(lanes, [{"foo"}])

    def test_activation_covers_an_implied_feature(self) -> None:
        table = parse_feature_table(self.TOML)
        self.assertEqual(activate({"foo"}, table), {"foo", "bar"})

    def test_a_narrower_lane_does_not_cover_a_wider_requirement(self) -> None:
        table = parse_feature_table(self.TOML)
        self.assertFalse({"foo"} <= activate({"bar"}, table))


class TestWiring(unittest.TestCase):
    def test_every_gated_lib_test_has_an_executor(self) -> None:
        gaps = uncovered(
            CRATE_SRC,
            CRATE_TOML.read_text(encoding="utf-8"),
            CI_YML.read_text(encoding="utf-8"),
        )
        self.assertEqual(
            gaps,
            {},
            "sq-mwnko: these sparq-conformance lib test modules are feature-gated and "
            "run in NO ci.yml leg — they compile only with the listed feature(s) ON, "
            "and no `cargo test -p sparq-conformance --features ... --lib` invocation "
            "activates them, so a stale assertion in them can sit RED on main "
            f"unnoticed: {gaps}. Fix: add `--lib` to the ci.yml lane that already runs "
            "that feature (it costs one extra test-binary link), not by deleting the "
            "tests.",
        )

    def test_the_ql_lane_specifically_runs_lib(self) -> None:
        """The named instance from the issue, pinned so a revert is loud."""
        text = re.sub(r"\\\n\s*", " ", CI_YML.read_text(encoding="utf-8"))
        ql = [
            line
            for line in text.splitlines()
            if "cargo test" in line
            and "-p sparq-conformance" in line
            and "ql-experimental" in line
        ]
        self.assertTrue(ql, "the ql-experimental lane disappeared from ci.yml")
        for line in ql:
            self.assertRegex(
                line,
                r"(?<!\S)--lib(?!\S)",
                "the ql-experimental lane must build the lib test binary "
                "(inference::sparql_entail::ql_tests runs nowhere else)",
            )


if __name__ == "__main__":
    unittest.main(verbosity=2)
