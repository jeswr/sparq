#!/usr/bin/env python3
# test_classify_cache.py — [OPUS-4.8] unit tests for the Fable classification cache.
#
# Pure-stdlib unittest. Run with either:
#     python3 -m unittest scripts.fable.test_classify_cache
#     python3 scripts/fable/test_classify_cache.py
#
# Covers set/get round-trips and sha256 hash-invalidation (stale-check).

import importlib.util
import io
import os
import tempfile
import unittest
from contextlib import redirect_stdout

# Load classify-cache.py by path (the hyphen makes it non-importable by name).
_HERE = os.path.dirname(os.path.abspath(__file__))
_SPEC = importlib.util.spec_from_file_location(
    "classify_cache", os.path.join(_HERE, "classify-cache.py"))
cc = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(cc)


class CacheTestBase(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.cache = os.path.join(self.tmp.name, "cache.json")
        self.target = os.path.join(self.tmp.name, "target.txt")
        with open(self.target, "w", encoding="utf-8") as fh:
            fh.write("hello benign prose\n")

    def tearDown(self):
        self.tmp.cleanup()

    def run_cli(self, *argv):
        """Invoke main() with --cache-path pinned; return (rc, stdout)."""
        buf = io.StringIO()
        with redirect_stdout(buf):
            rc = cc.main(["--cache-path", self.cache, *argv])
        return rc, buf.getvalue()


class TestSetGet(CacheTestBase):
    def test_set_then_get_round_trip(self):
        rc, _ = self.run_cli("set", self.target,
                             "--observed-tier", "downgraded",
                             "--classification", "code")
        self.assertEqual(rc, 0)

        rc, out = self.run_cli("get", self.target)
        self.assertEqual(rc, 0)
        entry = __import__("json").loads(out)
        self.assertEqual(entry["observed_tier"], "downgraded")
        self.assertEqual(entry["classification"], "code")
        # Observed tier supplied without --source -> source derives to "observed".
        self.assertEqual(entry["source"], "observed")
        self.assertEqual(entry["content_sha256"], cc.sha256_of_file(self.target))
        self.assertIn("updated", entry)

    def test_get_missing_returns_null_and_rc1(self):
        rc, out = self.run_cli("get", self.target)
        self.assertEqual(rc, 1)
        self.assertEqual(out.strip(), "null")

    def test_predicted_source_default(self):
        # No observed tier => prior is PREDICTED.
        self.run_cli("set", self.target, "--classification", "benign_prose")
        _, out = self.run_cli("get", self.target)
        entry = __import__("json").loads(out)
        self.assertIsNone(entry["observed_tier"])
        self.assertEqual(entry["source"], "predicted")


class TestHashInvalidation(CacheTestBase):
    def test_stale_check_true_when_uncached(self):
        rc, out = self.run_cli("stale-check", self.target)
        self.assertEqual(rc, 0)
        self.assertEqual(out.strip(), "true")

    def test_stale_check_false_after_set(self):
        self.run_cli("set", self.target, "--classification", "benign_prose")
        _, out = self.run_cli("stale-check", self.target)
        self.assertEqual(out.strip(), "false")

    def test_stale_check_true_after_content_change(self):
        self.run_cli("set", self.target, "--classification", "benign_prose")
        # Mutate the file -> sha changes -> entry is stale.
        with open(self.target, "a", encoding="utf-8") as fh:
            fh.write("an edit that changes the hash\n")
        _, out = self.run_cli("stale-check", self.target)
        self.assertEqual(out.strip(), "true")

    def test_reset_after_reclassify(self):
        self.run_cli("set", self.target, "--classification", "benign_prose")
        with open(self.target, "a", encoding="utf-8") as fh:
            fh.write("changed\n")
        # Re-run set (reclassify) refreshes the sha -> fresh again.
        self.run_cli("set", self.target, "--classification", "benign_prose")
        _, out = self.run_cli("stale-check", self.target)
        self.assertEqual(out.strip(), "false")


class TestListAndValidation(CacheTestBase):
    def test_list_reflects_entries(self):
        self.run_cli("set", self.target, "--classification", "code")
        _, out = self.run_cli("list")
        cache = __import__("json").loads(out)
        self.assertIn(self.target, cache)

    def test_set_missing_file_errors(self):
        rc, _ = self.run_cli("set", os.path.join(self.tmp.name, "nope.txt"))
        self.assertEqual(rc, 2)


if __name__ == "__main__":
    unittest.main()
