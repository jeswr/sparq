#!/usr/bin/env python3
# [SONNET-4.6] Hermetic tests for the FO-KM task validator's PER-CATEGORY gold check
# (bench/fo-km/validate_tasks.py, epic sq-mztg8).
#
# WHY: cc03 (`{event, artifact}`) and cc05 (`{claim, document, method}`) carry a
# dict-of-ints `gold_keys` behind a SINGLE `select` query, so they reached neither the
# list-gold row check nor the multi-part `gold_count` check — ANY non-empty result passed,
# even with wrong per-category values, so a refreshed gold was never materially proven.
# These tests pin the value comparison in BOTH directions (a correct table passes, a
# mutated category value fails) and pin the DISPATCH, so no task shape in the shipped
# tasks.jsonl silently escapes a gold check again.
#
# Hermetic w.r.t. cargo/network: `run()` (which shells out to the pkg-query binary) is
# monkeypatched with a table generator that reproduces pkg_query.rs::print_table's layout
# (cells joined by "  |  ", header, dashed separator, rows). No subprocess, no toolchain.
#
# Run:  python3 scripts/tests/test_fo_km_gold_check.py
# (stdlib only; no pytest required — also discoverable by `pytest`.)

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import os
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SEP = "  |  "  # pkg_query.rs::print_table joins cells with exactly this


def _load():
    path = REPO_ROOT / "bench" / "fo-km" / "validate_tasks.py"
    spec = importlib.util.spec_from_file_location("fo_km_validate_tasks", path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["fo_km_validate_tasks"] = mod
    spec.loader.exec_module(mod)
    return mod


vt = _load()


def table(header_cells: list[str], rows: list[list[str]]) -> tuple[str, list[str]]:
    """Build a (header_line, data_rows) pair exactly as print_table would emit it."""
    return SEP.join(header_cells), [SEP.join(r) for r in rows]


def tasks() -> list[dict]:
    with open(REPO_ROOT / "bench" / "fo-km" / "tasks.jsonl") as fh:
        return [json.loads(line) for line in fh]


def task(tid: str) -> dict:
    return next(t for t in tasks() if t["id"] == tid)


# --- the two real result shapes, as pkg-query prints them --------------------------


def wide(gold: dict) -> tuple[str, list[str]]:
    """cc03: one row of aggregate columns, projection names PLURAL (`?events`)."""
    return table([f"{k}s" for k in gold], [[str(v) for v in gold.values()]])


def grouped(gold: dict) -> tuple[str, list[str]]:
    """cc05: `?kind (COUNT(DISTINCT ?x) AS ?n)` + GROUP BY — one row per category."""
    return table(["kind", "n"], [[k, str(v)] for k, v in sorted(gold.items())])


class TestCategoryGoldCc03(unittest.TestCase):
    """cc03 — the WIDE multi-column shape."""

    def setUp(self):
        self.gold = task("cc03")["gold_keys"]  # {"event": N, "artifact": N}

    def test_correct_result_passes(self):
        header, data = wide(self.gold)
        self.assertEqual(vt.check_category_gold("cc03", "gufo", header, data, self.gold), [])

    def test_mutated_event_count_fails(self):
        wrong = dict(self.gold, event=self.gold["event"] - 1)
        header, data = wide(wrong)
        failures = vt.check_category_gold("cc03", "gufo", header, data, self.gold)
        self.assertTrue(any("'event'" in f for f in failures), failures)
        self.assertTrue(any(str(self.gold["event"]) in f for f in failures), failures)

    def test_mutated_artifact_count_fails(self):
        wrong = dict(self.gold, artifact=self.gold["artifact"] + 7)
        header, data = wide(wrong)
        failures = vt.check_category_gold("cc03", "gufo", header, data, self.gold)
        self.assertTrue(any("'artifact'" in f for f in failures), failures)

    def test_swapped_columns_fail(self):
        """The value comparison is per-CATEGORY, not a bag of numbers."""
        header, _ = wide(self.gold)
        _, data = table([], [[str(self.gold["artifact"]), str(self.gold["event"])]])
        failures = vt.check_category_gold("cc03", "gufo", header, data, self.gold)
        self.assertEqual(len(failures), 2, failures)

    def test_non_empty_row_alone_does_not_pass(self):
        """The pre-fix hole: one non-empty row with arbitrary values used to pass."""
        header, data = table(["events", "artifacts"], [["1", "1"]])
        self.assertNotEqual(vt.check_category_gold("cc03", "gufo", header, data, self.gold), [])


class TestCategoryGoldCc05(unittest.TestCase):
    """cc05 — the GROUPED shape."""

    def setUp(self):
        self.gold = task("cc05")["gold_keys"]  # {"claim": N, "document": N, "method": N}

    def test_correct_result_passes(self):
        header, data = grouped(self.gold)
        self.assertEqual(vt.check_category_gold("cc05", "gufo", header, data, self.gold), [])

    def test_mutated_claim_count_fails(self):
        wrong = dict(self.gold, claim=self.gold["claim"] - 1)
        header, data = grouped(wrong)
        failures = vt.check_category_gold("cc05", "gufo", header, data, self.gold)
        self.assertTrue(any("'claim'" in f for f in failures), failures)

    def test_each_category_is_checked_independently(self):
        for cat in self.gold:
            wrong = dict(self.gold, **{cat: self.gold[cat] + 1})
            header, data = grouped(wrong)
            failures = vt.check_category_gold("cc05", "gufo", header, data, self.gold)
            self.assertTrue(any(f"'{cat}'" in f for f in failures), (cat, failures))

    def test_missing_group_fails(self):
        partial = {k: v for k, v in self.gold.items() if k != "method"}
        header, data = grouped(partial)
        failures = vt.check_category_gold("cc05", "gufo", header, data, self.gold)
        self.assertTrue(any("'method'" in f for f in failures), failures)

    def test_unexpected_group_fails(self):
        extra = dict(self.gold, gizmo=3)
        header, data = grouped(extra)
        failures = vt.check_category_gold("cc05", "gufo", header, data, self.gold)
        self.assertTrue(any("gizmo" in f for f in failures), failures)

    def test_right_row_count_wrong_values_fails(self):
        """gold_count for cc05 is the GROUP count (3) — matching it proves nothing."""
        header, data = grouped({k: 999 for k in self.gold})
        failures = vt.check_category_gold("cc05", "gufo", header, data, self.gold)
        self.assertEqual(len(data), task("cc05")["gold_count"])
        self.assertEqual(len(failures), len(self.gold), failures)


class TestUnparseableTableIsAFailure(unittest.TestCase):
    """An unreadable table must FAIL loudly, never pass as 'nothing to check'."""

    def test_entity_list_result_is_reported_unchecked(self):
        header, data = table(["x"], [["find-a"], ["find-b"]])
        failures = vt.check_category_gold("cc05", "gufo", header, data, {"claim": 2})
        self.assertTrue(any("UNCHECKED" in f for f in failures), failures)

    def test_empty_result_is_reported_unchecked(self):
        failures = vt.check_category_gold("cc03", "gufo", "events  |  artifacts", [], {"event": 1})
        self.assertTrue(any("UNCHECKED" in f for f in failures), failures)


class TestNormalisation(unittest.TestCase):
    def test_plural_projection_matches_singular_gold_key(self):
        self.assertEqual(vt.norm("events"), vt.norm("event"))
        self.assertEqual(vt.norm("artifacts"), vt.norm("artifact"))

    def test_distinct_categories_stay_distinct(self):
        self.assertNotEqual(vt.norm("claim"), vt.norm("document"))
        self.assertNotEqual(vt.norm("event"), vt.norm("artifact"))

    def test_colliding_gold_keys_are_reported(self):
        header, data = table(["kind", "n"], [["event", "1"]])
        failures = vt.check_category_gold("x", "gufo", header, data, {"event": 1, "events": 1})
        self.assertTrue(any("ambiguous" in f for f in failures), failures)


class TestShapePredicate(unittest.TestCase):
    def test_dict_of_ints_is_category_gold(self):
        self.assertTrue(vt.is_category_gold({"event": 1, "artifact": 2}))

    def test_dict_of_lists_is_not(self):
        """th03's gold_keys is a dict of ENTITY LISTS — the multi-part branch owns it."""
        self.assertFalse(vt.is_category_gold(task("th03")["gold_keys"]))

    def test_list_and_empty_are_not(self):
        self.assertFalse(vt.is_category_gold(["a", "b"]))
        self.assertFalse(vt.is_category_gold({}))

    def test_shipped_fixture_routes_cc03_and_cc05_here(self):
        """Pins the dispatch: these two are dict-of-ints gold behind a SINGLE query."""
        for tid in ("cc03", "cc05"):
            t = task(tid)
            self.assertTrue(vt.is_category_gold(t["gold_keys"]), tid)
            for arm, q in t["select"].items():
                if q is not None:
                    self.assertIsInstance(q, str, f"{tid}/{arm}")


# --- end-to-end: drive main() with a canned result table ---------------------------


def fake_run_factory(overrides: dict | None = None):
    """A `run()` stand-in: answers each query from tasks.jsonl's own gold.

    `overrides` maps a task id to a {category: wrong_value} patch, so a test can inject a
    deliberately wrong query result and assert the validator rejects it.
    """
    overrides = overrides or {}
    index = {}  # query string -> (task, part)
    for t in tasks():
        for q in t["select"].values():
            if q is None:
                continue
            for part, sub in (q.items() if isinstance(q, dict) else [("_", q)]):
                index[sub] = (t, part)

    def _run(overlay_path, sparql):
        if overlay_path == vt.OVERLAY["no-fo"]:
            return "x", []  # the no-FO arm cannot answer — that is the discrimination
        t, part = index[sparql]
        gold_keys, gold_count = t["gold_keys"], t["gold_count"]
        if vt.is_category_gold(gold_keys):
            gold = dict(gold_keys, **overrides.get(t["id"], {}))
            return grouped(gold) if "GROUP BY" in sparql else wide(gold)
        n = gold_count[part] if isinstance(gold_count, dict) else gold_count
        if "GROUP BY" in sparql:
            return table(["cat", "n"], [[f"c{i}", "1"] for i in range(n)])
        if "COUNT(" in sparql.upper():
            return table(["n"], [[str(n)]])
        return table(["x"], [[f"e{i}"] for i in range(n)])

    return _run


@contextlib.contextmanager
def driven(overrides=None):
    """Run main() against the fake table generator from the repo root; yield its stdout."""
    original_run, cwd, out = vt.run, os.getcwd(), io.StringIO()
    vt.run = fake_run_factory(overrides)
    os.chdir(REPO_ROOT)
    try:
        with contextlib.redirect_stdout(out):
            yield out
    finally:
        vt.run, _ = original_run, os.chdir(cwd)


class TestEndToEndDispatch(unittest.TestCase):
    def test_gold_matching_results_validate(self):
        """Every shipped task passes when each arm returns its own gold — so the new
        `else` (unrecognised shape) does NOT fire on any of the 16 tasks."""
        with driven() as out:
            vt.main()
        self.assertIn("TASKS DISCRIMINATE", out.getvalue())
        self.assertNotIn("unrecognised gold shape", out.getvalue())

    def test_wrong_cc03_event_count_fails_validation(self):
        gold = task("cc03")["gold_keys"]
        with driven({"cc03": {"event": gold["event"] - 1}}) as out:
            with self.assertRaises(SystemExit) as exit_ctx:
                vt.main()
        self.assertEqual(exit_ctx.exception.code, 1)
        self.assertIn("cc03", out.getvalue())
        self.assertIn("'event'", out.getvalue())

    def test_wrong_cc05_document_count_fails_validation(self):
        gold = task("cc05")["gold_keys"]
        with driven({"cc05": {"document": gold["document"] + 1}}) as out:
            with self.assertRaises(SystemExit) as exit_ctx:
                vt.main()
        self.assertEqual(exit_ctx.exception.code, 1)
        self.assertIn("cc05", out.getvalue())
        self.assertIn("'document'", out.getvalue())


if __name__ == "__main__":
    unittest.main(verbosity=2)
