#!/usr/bin/env python3
# [OPUS-4.8] Hermetic unit/dry-run tests for the flow-on engine (bead sq-ncvq.2,
# epic sq-ncvq). Authored by Opus 4.8 (Fable unavailable; flag for re-review when
# Fable returns).
#
# Fully hermetic: imports scripts/flow-on.py and exercises rule evaluation +
# idempotency-key generation against fixture diffs. NO live `gh` calls — we only
# drive `evaluate()` and `main(--dry-run)`, neither of which touches the network.
#
# Run:  python3 scripts/tests/test_flow_on.py
# (stdlib only; no pytest required.)

from __future__ import annotations

import importlib.util
import io
import sys
import unittest
from contextlib import redirect_stdout
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
FLOW_ON = REPO_ROOT / "scripts" / "flow-on.py"
RULES = REPO_ROOT / "scripts" / "flow-on-rules.toml"


def _load_module():
    spec = importlib.util.spec_from_file_location("flow_on", FLOW_ON)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    # Register before exec so dataclasses can resolve forward annotations
    # (cls.__module__ lookup in _is_type) for the module-level @dataclass types.
    sys.modules["flow_on"] = mod
    spec.loader.exec_module(mod)
    return mod


flow_on = _load_module()


class RuleLoadingTest(unittest.TestCase):
    def test_rules_load_and_are_well_formed(self):
        rules = flow_on.load_rules(RULES)
        self.assertTrue(rules, "expected at least one rule")
        ids = {r.id for r in rules}
        # The taxonomy rules the design requires must all be present.
        for required in (
            "new-crate-bench-and-skill",
            "changed-public-feature-docs",
            "new-bench-dashboard-row",
            "competitor-feature-gather",
        ):
            self.assertIn(required, ids)
        # Every rule has a trigger + at least one create template.
        for r in rules:
            self.assertTrue(
                r.when_paths or r.when_new_paths or r.when_label or r.when_title,
                f"{r.id} has no trigger",
            )
            self.assertTrue(r.creates, f"{r.id} has no create templates")


class NewCrateTest(unittest.TestCase):
    """A PR that adds crates/sparq-foo/ must yield the bench + SKILL follow-ons."""

    def setUp(self):
        self.rules = flow_on.load_rules(RULES)

    def _eval(self, changed, added, labels=None, title="add sparq-foo crate"):
        return flow_on.evaluate(
            self.rules, 42, title, changed, added, labels or []
        )

    def test_new_crate_triggers_bench_and_skill(self):
        added = [
            "crates/sparq-foo/Cargo.toml",
            "crates/sparq-foo/src/lib.rs",
            "crates/sparq-foo/README.md",
        ]
        fos = self._eval(changed=added, added=added)
        rule_ids = {fo.rule_id for fo in fos}
        self.assertIn("new-crate-bench-and-skill", rule_ids)

        keys = {fo.dedup_key for fo in fos}
        self.assertIn("new-crate-bench-sparq-foo", keys)
        self.assertIn("new-crate-skill-sparq-foo", keys)

        # The {crate} placeholder is expanded in titles + bodies.
        bench_fo = next(fo for fo in fos if fo.dedup_key == "new-crate-bench-sparq-foo")
        self.assertIn("sparq-foo", bench_fo.title)
        self.assertIn("sparq-foo", bench_fo.body)
        self.assertNotIn("{crate}", bench_fo.title)
        self.assertNotIn("{crate}", bench_fo.body)

        # Base labels always present; the idempotency marker is embedded.
        self.assertIn("flow-on", bench_fo.labels)
        self.assertIn("auto", bench_fo.labels)
        self.assertIn(flow_on.key_marker("new-crate-bench-sparq-foo"), bench_fo.body)

    def test_modifying_existing_crate_file_does_not_fire_new_crate_rule(self):
        # Cargo.toml is CHANGED but not ADDED -> when_new_paths must not match.
        changed = ["crates/sparq-core/Cargo.toml", "crates/sparq-core/src/store/mod.rs"]
        added: list[str] = []
        fos = self._eval(changed=changed, added=added)
        self.assertNotIn("new-crate-bench-and-skill", {fo.rule_id for fo in fos})


class ChangedSurfaceTest(unittest.TestCase):
    def setUp(self):
        self.rules = flow_on.load_rules(RULES)

    def test_cli_change_without_skill_fires_docs_rule(self):
        changed = ["crates/sparq-cli/src/main.rs"]
        fos = flow_on.evaluate(self.rules, 7, "add --explain flag", changed, [], [])
        self.assertIn("changed-public-feature-docs", {fo.rule_id for fo in fos})

    def test_cli_change_with_skill_update_suppresses_docs_rule(self):
        changed = ["crates/sparq-cli/src/main.rs", "skills/cli/SKILL.md"]
        fos = flow_on.evaluate(self.rules, 8, "add --explain flag", changed, [], [])
        self.assertNotIn("changed-public-feature-docs", {fo.rule_id for fo in fos})


class CompetitorLabelTest(unittest.TestCase):
    def setUp(self):
        self.rules = flow_on.load_rules(RULES)

    def test_engine_change_requires_label(self):
        changed = ["crates/sparq-engine/src/exec/join.rs"]
        # Without the label, the gather rule must NOT fire.
        fos = flow_on.evaluate(self.rules, 9, "faster joins", changed, [], [])
        self.assertNotIn("competitor-feature-gather", {fo.rule_id for fo in fos})
        # With the label, it fires.
        fos = flow_on.evaluate(
            self.rules, 9, "faster joins", changed, [], ["competitor-relevant"]
        )
        self.assertIn("competitor-feature-gather", {fo.rule_id for fo in fos})


class NewBenchTest(unittest.TestCase):
    def setUp(self):
        self.rules = flow_on.load_rules(RULES)

    def test_new_bench_suite_triggers_dashboard_row(self):
        added = ["bench/widgets/benchmarks.toml", "bench/widgets/run.sh"]
        fos = flow_on.evaluate(self.rules, 11, "add widgets bench", added, added, [])
        dash = [fo for fo in fos if fo.rule_id == "new-bench-dashboard-row"]
        self.assertTrue(dash)
        self.assertEqual(dash[0].dedup_key, "dashboard-row-widgets")
        self.assertIn("widgets", dash[0].title)


class IdempotencyTest(unittest.TestCase):
    """The dedup-key marker is what open_issue_exists() searches for; verify the
    key is stable + embedded so a re-run is a no-op once the issue is open."""

    def setUp(self):
        self.rules = flow_on.load_rules(RULES)

    def test_dedup_key_stable_across_runs(self):
        added = ["crates/sparq-foo/Cargo.toml"]
        a = flow_on.evaluate(self.rules, 42, "t", added, added, [])
        b = flow_on.evaluate(self.rules, 42, "t", added, added, [])
        self.assertEqual(
            sorted(fo.dedup_key for fo in a),
            sorted(fo.dedup_key for fo in b),
        )

    def test_marker_round_trips(self):
        key = "new-crate-bench-sparq-foo"
        body = "some body\n\n" + flow_on.key_marker(key)
        # open_issue_exists greps for exactly this marker substring.
        self.assertIn(flow_on.key_marker(key), body)


class DryRunTest(unittest.TestCase):
    """main(--dry-run) must print the would-create plan and make NO gh calls.

    We provide all inputs hermetically (changed-files/title) so fetch_pr_inputs
    is never reached, and --dry-run skips open_issue_exists/create_issue."""

    def test_dry_run_on_new_crate_fixture(self):
        import tempfile

        with tempfile.TemporaryDirectory() as td:
            changed = Path(td) / "changed.txt"
            changed.write_text("crates/sparq-foo/Cargo.toml\ncrates/sparq-foo/src/lib.rs\n")
            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = flow_on.main(
                    [
                        "--pr",
                        "42",
                        "--dry-run",
                        "--changed-files",
                        str(changed),
                        "--added-files",
                        str(changed),
                        "--title",
                        "add sparq-foo crate",
                    ]
                )
            out = buf.getvalue()
        self.assertEqual(rc, 0)
        self.assertIn("[dry-run] WOULD create", out)
        self.assertIn("new-crate-bench-sparq-foo", out)

    def test_dry_run_no_match_is_clean(self):
        import tempfile

        with tempfile.TemporaryDirectory() as td:
            changed = Path(td) / "changed.txt"
            changed.write_text("README.md\n")
            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = flow_on.main(
                    ["--pr", "1", "--dry-run", "--changed-files", str(changed), "--title", "docs"]
                )
            out = buf.getvalue()
        self.assertEqual(rc, 0)
        self.assertIn("no rules triggered", out)


if __name__ == "__main__":
    unittest.main(verbosity=2)
