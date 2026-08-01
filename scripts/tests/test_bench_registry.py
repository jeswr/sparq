#!/usr/bin/env python3
# [OPUS-5] sq-jfrp0 (issue #2679) — hermetic tests for the bench-registry assembler.
#
# INVARIANT UNDER TEST: bench PRs must stop serial-conflicting. The mechanism is
# per-suite FRAGMENT files (bench/registry.d/*.toml, bench/competitors.d/*.json,
# bench/catalog.d/*.md) assembled at READ time onto the committed trunk files, so
# two PRs adding different suites never touch the same path. The properties that
# have to hold for that to be safe:
#
#   1. BEHAVIOUR PRESERVATION — with zero fragments the assembled text is
#      BYTE-IDENTICAL to the trunk, so every consuming gate keeps its exact
#      pre-split behaviour.
#   2. DETERMINISM — fragments are ordered by filename, never by directory order,
#      so the assembled view is stable run-to-run and diff-clean.
#   3. NO SILENT COLLISION — a duplicate benchmark/competitor id across trunk and
#      fragments is a HARD error (every gate resolves a suite by id; a duplicate
#      would silently shadow one entry).
#   4. THE REAL TREE ASSEMBLES — the committed registries validate, the assembled
#      registry is parseable TOML, and no benchmark id was lost or duplicated by
#      the migration of the first entry into a fragment.
#
# Hermetic: stdlib only, no network, no git, no subprocesses. Fixtures are built in
# a tempdir; the only on-disk reads are the committed registries themselves.
#
# Run:  python3 scripts/tests/test_bench_registry.py

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SCRIPTS = REPO_ROOT / "scripts"


def _load(name: str, filename: str):
    spec = importlib.util.spec_from_file_location(name, SCRIPTS / filename)
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


br = _load("bench_registry", "bench_registry.py")
gate = _load("check_new_bench_registered", "check-new-bench-registered.py")


ENTRY = """\
[[benchmark]]
id          = "{bid}"
name        = "{bid} bench"
category    = "query"
measures    = "things"
invoke      = "bash bench/{bid}/run.sh"
source      = "bench/{bid}/run.sh"
dataset     = "synthetic"
quiet_box_sensitive = false
pinning     = "seeded"
records_to  = "stdout"
status      = "ok"
"""


def _tree(registry_frags=None, competitor_frags=None, catalog_frags=None, trunk=None):
    """Build a throwaway repo-shaped tree and return its root Path."""
    root = Path(tempfile.mkdtemp())
    bench = root / "bench"
    bench.mkdir()
    (bench / "benchmarks.toml").write_text(
        trunk if trunk is not None else "# trunk\n\n" + ENTRY.format(bid="alpha"),
        encoding="utf-8",
    )
    (bench / "competitors.json").write_text(
        json.dumps({"schema_version": 1, "competitors": [{"id": "oxigraph"}]}),
        encoding="utf-8",
    )
    (bench / "CATALOG.md").write_text("# catalog\n", encoding="utf-8")
    for dirname, frags, suffix in (
        ("registry.d", registry_frags, ".toml"),
        ("competitors.d", competitor_frags, ".json"),
        ("catalog.d", catalog_frags, ".md"),
    ):
        if frags is None:
            continue
        d = bench / dirname
        d.mkdir()
        for name, text in frags.items():
            (d / name).write_text(text, encoding="utf-8")
        # Every fragment dir carries a README that must never be assembled in.
        (d / "README.md").write_text(f"# {dirname}\n\nconvention docs\n", encoding="utf-8")
        del suffix
    return root


class BehaviourPreservation(unittest.TestCase):
    def test_no_fragment_dir_is_byte_identical_to_trunk(self):
        root = _tree()
        trunk = (root / "bench" / "benchmarks.toml").read_text(encoding="utf-8")
        self.assertEqual(br.registry_text(root), trunk)

    def test_empty_fragment_dir_is_byte_identical_to_trunk(self):
        # A dir holding only its README (the state right after this bead lands in
        # bench/competitors.d and bench/catalog.d) must not perturb the output.
        root = _tree(registry_frags={})
        trunk = (root / "bench" / "benchmarks.toml").read_text(encoding="utf-8")
        self.assertEqual(br.registry_text(root), trunk)

    def test_readme_is_never_assembled_as_a_catalog_fragment(self):
        root = _tree(catalog_frags={})
        self.assertEqual(
            br.catalog_text(root), (root / "bench" / "CATALOG.md").read_text("utf-8")
        )

    def test_unreadable_trunk_yields_empty_string(self):
        root = Path(tempfile.mkdtemp())
        self.assertEqual(br.registry_text(root), "")


class RegistryAssembly(unittest.TestCase):
    def test_fragment_entries_join_the_registry(self):
        root = _tree(registry_frags={"zeta.toml": ENTRY.format(bid="zeta")})
        text = br.registry_text(root)
        self.assertEqual(br.registry_ids(text), ["alpha", "zeta"])

    def test_assembled_output_is_parseable_toml(self):
        root = _tree(
            registry_frags={
                "beta.toml": "# note\n" + ENTRY.format(bid="beta"),
                "gamma.toml": ENTRY.format(bid="gamma"),
            }
        )
        parsed = tomllib.loads(br.registry_text(root))
        self.assertEqual(
            [b["id"] for b in parsed["benchmark"]], ["alpha", "beta", "gamma"]
        )

    def test_order_is_by_filename_not_directory_order(self):
        # Created in reverse order on purpose; the assembler must sort.
        root = _tree(
            registry_frags={
                "zulu.toml": ENTRY.format(bid="zulu"),
                "alfa.toml": ENTRY.format(bid="bravo"),
            }
        )
        self.assertEqual(br.registry_ids(br.registry_text(root)), ["alpha", "bravo", "zulu"])

    def test_duplicate_id_against_trunk_is_a_hard_error(self):
        root = _tree(registry_frags={"dup.toml": ENTRY.format(bid="alpha")})
        with self.assertRaises(br.RegistryError) as cm:
            br.registry_text(root)
        self.assertIn("duplicate benchmark id 'alpha'", str(cm.exception))

    def test_duplicate_id_across_two_fragments_is_a_hard_error(self):
        root = _tree(
            registry_frags={
                "a.toml": ENTRY.format(bid="dup"),
                "b.toml": ENTRY.format(bid="dup"),
            }
        )
        with self.assertRaises(br.RegistryError):
            br.registry_text(root)

    def test_fragment_without_a_benchmark_entry_is_rejected(self):
        root = _tree(registry_frags={"empty.toml": "# just a comment\n"})
        with self.assertRaises(br.RegistryError) as cm:
            br.registry_text(root)
        self.assertIn("declares no [[benchmark]]", str(cm.exception))

    def test_fragment_with_a_stray_table_is_rejected(self):
        root = _tree(
            registry_frags={"stray.toml": ENTRY.format(bid="x") + "\n[extra]\nk = 1\n"}
        )
        with self.assertRaises(br.RegistryError) as cm:
            br.registry_text(root)
        self.assertIn("unexpected top-level table", str(cm.exception))

    def test_fragment_entry_without_an_id_is_rejected(self):
        root = _tree(registry_frags={"noid.toml": "[[benchmark]]\nname = \"x\"\n"})
        with self.assertRaises(br.RegistryError) as cm:
            br.registry_text(root)
        self.assertIn("no `id` field", str(cm.exception))

    def test_second_entry_without_an_id_is_rejected(self):
        # Validation must be PER ENTRY, not fragment-wide: one identified entry
        # must not vouch for an id-less sibling, or the assembled registry would
        # carry a benchmark no id-based gate can ever resolve.
        root = _tree(
            registry_frags={
                "half.toml": ENTRY.format(bid="ok") + '\n[[benchmark]]\nname = "missing"\n'
            }
        )
        with self.assertRaises(br.RegistryError) as cm:
            br.registry_text(root)
        self.assertIn("entry #2 has no `id` field", str(cm.exception))

    def test_entry_declaring_two_ids_is_rejected(self):
        root = _tree(
            registry_frags={"two.toml": ENTRY.format(bid="x") + 'id = "y"\n'}
        )
        with self.assertRaises(br.RegistryError) as cm:
            br.registry_text(root)
        self.assertIn("declares 2 `id` fields", str(cm.exception))

    def test_id_before_the_first_benchmark_header_is_rejected(self):
        # A top-level `id` belongs to no entry; accepting it would leave an id that
        # duplicate detection never sees.
        root = _tree(
            registry_frags={"stray.toml": 'id = "loose"\n' + ENTRY.format(bid="x")}
        )
        with self.assertRaises(br.RegistryError) as cm:
            br.registry_text(root)
        self.assertIn("declared before the first [[benchmark]] header", str(cm.exception))


class CompetitorAssembly(unittest.TestCase):
    def test_fragment_competitor_joins_the_array(self):
        root = _tree(
            competitor_frags={"qlever.json": json.dumps({"competitors": [{"id": "qlever"}]})}
        )
        obj = br.competitors_obj(root)
        self.assertEqual([c["id"] for c in obj["competitors"]], ["oxigraph", "qlever"])
        self.assertEqual(obj["schema_version"], 1, "trunk-only keys must survive")

    def test_duplicate_competitor_id_is_a_hard_error(self):
        root = _tree(
            competitor_frags={"ox.json": json.dumps({"competitors": [{"id": "oxigraph"}]})}
        )
        with self.assertRaises(br.RegistryError) as cm:
            br.competitors_obj(root)
        self.assertIn("duplicate competitor id 'oxigraph'", str(cm.exception))

    def test_fragment_carrying_a_non_competitors_key_is_rejected(self):
        root = _tree(
            competitor_frags={
                "bad.json": json.dumps({"competitors": [{"id": "x"}], "values": {}})
            }
        )
        with self.assertRaises(br.RegistryError) as cm:
            br.competitors_obj(root)
        self.assertIn("unexpected key(s) ['values']", str(cm.exception))

    def test_malformed_json_fragment_is_rejected(self):
        root = _tree(competitor_frags={"bad.json": "{not json"})
        with self.assertRaises(br.RegistryError) as cm:
            br.competitors_obj(root)
        self.assertIn("not valid JSON", str(cm.exception))

    def test_entry_without_a_string_id_is_rejected(self):
        root = _tree(competitor_frags={"bad.json": json.dumps({"competitors": [{}]})})
        with self.assertRaises(br.RegistryError):
            br.competitors_obj(root)


class CatalogAssembly(unittest.TestCase):
    def test_fragments_append_in_filename_order(self):
        root = _tree(
            catalog_frags={"zulu.md": "## zulu\n\nz\n", "alfa.md": "## alfa\n\na\n"}
        )
        text = br.catalog_text(root)
        self.assertLess(text.index("## alfa"), text.index("## zulu"))
        self.assertTrue(text.startswith("# catalog"))
        self.assertNotIn("convention docs", text)

    def test_fragment_without_a_leading_heading_is_rejected(self):
        root = _tree(catalog_frags={"x.md": "just prose\n"})
        with self.assertRaises(br.RegistryError) as cm:
            br.catalog_text(root)
        self.assertIn("must start with a markdown heading", str(cm.exception))


class GateIntegration(unittest.TestCase):
    """The G3 merge gate must understand the fragment layout."""

    def test_fragment_dirs_are_not_mistaken_for_new_bench_suites(self):
        added = [
            "bench/registry.d/newsuite.toml",
            "bench/competitors.d/newengine.json",
            "bench/catalog.d/newsuite.md",
            "bench/realsuite/run.sh",
        ]
        self.assertEqual(gate.added_bench_suites(added, set()), ["realsuite"])

    def test_new_id_in_a_fragment_is_not_flagged_as_a_trunk_append(self):
        trunk = "# trunk\n" + ENTRY.format(bid="alpha")
        head = trunk + "\n" + ENTRY.format(bid="beta")
        violations = gate.evaluate(
            [],
            head_toml=head,
            base_toml=trunk,
            featured_text="",
            head_trunk_toml=trunk,  # `beta` lives in a fragment, not the trunk
        )
        self.assertEqual(
            [r for _label, reasons in violations for r in reasons if "fragment file" in r],
            [],
        )

    def test_new_id_appended_to_the_trunk_is_flagged(self):
        base = "# trunk\n" + ENTRY.format(bid="alpha")
        head = base + "\n" + ENTRY.format(bid="beta")
        violations = gate.evaluate(
            [],
            head_toml=head,
            base_toml=base,
            featured_text="beta",  # satisfy the dashboard requirement in isolation
            head_trunk_toml=head,  # `beta` was appended to the trunk itself
        )
        reasons = [r for _label, rs in violations for r in rs]
        self.assertTrue(
            any("bench/registry.d/<suite>.toml" in r for r in reasons),
            f"expected a fragment-placement finding, got {reasons}",
        )

    def test_trunk_check_is_opt_in(self):
        base = "# trunk\n" + ENTRY.format(bid="alpha")
        head = base + "\n" + ENTRY.format(bid="beta")
        violations = gate.evaluate(
            [], head_toml=head, base_toml=base, featured_text="beta"
        )
        self.assertEqual(violations, [], "omitting head_trunk_toml must skip the check")


class CommittedTree(unittest.TestCase):
    """The real repository must assemble cleanly — this is what CI enforces."""

    def test_check_passes_on_the_committed_registries(self):
        self.assertEqual(br._check(), 0)

    def test_assembled_registry_is_valid_toml_with_unique_ids(self):
        parsed = tomllib.loads(br.registry_text())
        ids = [b["id"] for b in parsed["benchmark"]]
        self.assertEqual(len(ids), len(set(ids)), "duplicate benchmark id in the tree")

    def test_the_migrated_entry_is_visible_only_via_assembly(self):
        # bench/registry.d/sparq-wrapper.toml is the first real fragment: it must be
        # absent from the trunk and present in the assembled view, which is exactly
        # the property every consuming gate depends on.
        trunk = br.REGISTRY_TRUNK.read_text(encoding="utf-8")
        self.assertNotIn("sparq-wrapper-smoke", br.registry_ids(trunk))
        self.assertIn("sparq-wrapper-smoke", br.registry_ids(br.registry_text()))

    def test_crate_lookup_still_resolves_through_the_fragment(self):
        # gate-new-crate.py / drift-scan.py ask "does this crate have a registered
        # bench?" by matching `source = crates/<crate>`. The migrated entry is the
        # only registration for sparq-wrapper, so this only passes via assembly.
        gnc = _load("gate_new_crate", "gate-new-crate.py")
        self.assertTrue(gnc.crate_has_registered_bench("sparq-wrapper"))

    def test_fragment_dir_names_are_covered_by_the_non_suite_set(self):
        self.assertEqual(
            br.NON_SUITE_BENCH_DIRS, {"registry.d", "competitors.d", "catalog.d"}
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
