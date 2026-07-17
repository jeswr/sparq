#!/usr/bin/env python3
# [FABLE-5] Hermetic unit tests for the daily CodeQL alert sweep
# (scripts/codeql_alert_sweep.py — the retroactive-triage half of the 2026-07-17
# "CodeQL is non-blocking" decision). Stdlib-only, no network, no gh: the pure
# plan()/build_issue_body()/parse_tracked() logic is exercised over API-shaped
# fixtures. Run:  python3 scripts/tests/test_codeql_alert_sweep.py
#
# The load-bearing properties:
#   (a) create on first alerts, update ONLY on a changed set, close at zero,
#       no-op otherwise — the idempotent-upsert contract;
#   (b) the marker round-trips (build -> parse) so "new vs already-tracked" is
#       computed from the issue itself, and identification is marker-based;
#   (c) API-shaped alert objects (incl. field-absent degenerate ones) normalize
#       without raising;
#   (d) mutation checks: an unchanged set must NOT update; a changed set MUST.

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

sys.dont_write_bytecode = True

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SCRIPT = REPO_ROOT / "scripts" / "codeql_alert_sweep.py"


def _load_module():
    spec = importlib.util.spec_from_file_location("codeql_alert_sweep", SCRIPT)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["codeql_alert_sweep"] = mod
    spec.loader.exec_module(mod)
    return mod


s = _load_module()

REPO = "sparq-org/sparq"

# API-shaped fixture: the fields the real /code-scanning/alerts objects carry
# (subset — the normalizer must not depend on anything else).
RAW_ALERT_41 = {
    "number": 41,
    "html_url": f"https://github.com/{REPO}/security/code-scanning/41",
    "rule": {
        "id": "rust/cleartext-logging",
        "severity": "error",
        "security_severity_level": "high",
    },
    "most_recent_instance": {
        "location": {"path": "crates/sparq-server/src/http.rs"},
    },
}
RAW_ALERT_52 = {
    "number": 52,
    "html_url": f"https://github.com/{REPO}/security/code-scanning/52",
    "rule": {"id": "rust/uncontrolled-allocation-size", "severity": "warning"},
    "most_recent_instance": {"location": {"path": "crates/sparq-core/src/mmap.rs"}},
}
# Degenerate: the API omits location/rule details for some alert kinds.
RAW_ALERT_BARE = {"number": 7}


def alerts(*raw):
    return [s.normalize_alert(r) for r in raw]


class TestNormalize(unittest.TestCase):
    def test_full_alert(self):
        a = s.normalize_alert(RAW_ALERT_41)
        self.assertEqual(a["number"], 41)
        self.assertEqual(a["rule"], "rust/cleartext-logging")
        self.assertEqual(a["severity"], "high")  # security level preferred
        self.assertEqual(a["path"], "crates/sparq-server/src/http.rs")

    def test_severity_falls_back_to_rule_severity(self):
        self.assertEqual(s.normalize_alert(RAW_ALERT_52)["severity"], "warning")

    def test_bare_alert_degrades_without_raising(self):
        a = s.normalize_alert(RAW_ALERT_BARE)
        self.assertEqual(a["number"], 7)
        self.assertEqual(a["rule"], "(unknown rule)")
        self.assertEqual(a["path"], "(no path)")


class TestMarkerRoundTrip(unittest.TestCase):
    def test_build_then_parse(self):
        body = s.build_issue_body(alerts(RAW_ALERT_52, RAW_ALERT_41), REPO)
        self.assertEqual(s.parse_tracked(body), {41, 52})

    def test_body_is_deterministic_and_sorted(self):
        # Same set, either order => byte-identical body (the idempotence the
        # upsert compares on), listed sorted by alert number.
        b1 = s.build_issue_body(alerts(RAW_ALERT_41, RAW_ALERT_52), REPO)
        b2 = s.build_issue_body(alerts(RAW_ALERT_52, RAW_ALERT_41), REPO)
        self.assertEqual(b1, b2)
        self.assertLess(b1.index("#41"), b1.index("#52"))

    def test_body_lists_rule_path_and_dashboard_link(self):
        body = s.build_issue_body(alerts(RAW_ALERT_41), REPO)
        self.assertIn("rust/cleartext-logging", body)
        self.assertIn("crates/sparq-server/src/http.rs", body)
        self.assertIn(f"https://github.com/{REPO}/security/code-scanning", body)

    def test_parse_rejects_unmarked_or_garbled_bodies(self):
        # Marker-based identification: an arbitrary issue body must read as
        # "not the rolling issue" (None), never as an empty tracked set.
        self.assertIsNone(s.parse_tracked("just some issue"))
        self.assertIsNone(s.parse_tracked(""))
        self.assertIsNone(s.parse_tracked(None))
        self.assertIsNone(s.parse_tracked("<!-- codeql-alert-sweep:v1 tracked=[oops] -->"))

    def test_parse_accepts_empty_tracked_list(self):
        self.assertEqual(s.parse_tracked(s.marker_line([])), set())


class TestPlan(unittest.TestCase):
    """The idempotent-upsert contract, case by case."""

    def _issue_for(self, *raw):
        return {"number": 900, "body": s.build_issue_body(alerts(*raw), REPO)}

    def test_first_alerts_no_issue_creates(self):
        action = s.plan(alerts(RAW_ALERT_41), None, REPO)
        self.assertEqual(action["action"], "create")
        self.assertEqual(action["title"], s.ISSUE_TITLE)
        self.assertEqual(action["labels"], ["from:agent", "self-improvement"])
        self.assertEqual(s.parse_tracked(action["body"]), {41})

    def test_unchanged_set_is_a_noop(self):
        # Mutation check: re-running the sweep on the same alert set must NOT
        # touch the issue (breaking build_issue_body determinism or the marker
        # comparison turns this red).
        action = s.plan(alerts(RAW_ALERT_41, RAW_ALERT_52),
                        self._issue_for(RAW_ALERT_41, RAW_ALERT_52), REPO)
        self.assertEqual(action["action"], "none")

    def test_new_alert_updates_and_names_it(self):
        action = s.plan(alerts(RAW_ALERT_41, RAW_ALERT_52),
                        self._issue_for(RAW_ALERT_41), REPO)
        self.assertEqual(action["action"], "update")
        self.assertEqual(action["number"], 900)
        self.assertEqual(action["new"], [52])
        self.assertEqual(action["resolved"], [])
        self.assertEqual(s.parse_tracked(action["body"]), {41, 52})

    def test_resolved_alert_updates_and_names_it(self):
        action = s.plan(alerts(RAW_ALERT_41),
                        self._issue_for(RAW_ALERT_41, RAW_ALERT_52), REPO)
        self.assertEqual(action["action"], "update")
        self.assertEqual(action["new"], [])
        self.assertEqual(action["resolved"], [52])

    def test_zero_alerts_closes_open_issue(self):
        action = s.plan([], self._issue_for(RAW_ALERT_41), REPO)
        self.assertEqual(action["action"], "close")
        self.assertEqual(action["number"], 900)
        self.assertIn("closing", action["comment"].lower())

    def test_zero_alerts_no_issue_is_a_noop(self):
        self.assertEqual(s.plan([], None, REPO)["action"], "none")

    def test_markerless_issue_body_treated_as_untracked(self):
        # An issue whose marker was hand-deleted: tracked degrades to the empty
        # set, so every open alert counts as new and the body (incl. marker) is
        # restored by an update — self-healing, never a crash.
        action = s.plan(alerts(RAW_ALERT_41),
                        {"number": 900, "body": "marker gone"}, REPO)
        self.assertEqual(action["action"], "update")
        self.assertEqual(action["new"], [41])


if __name__ == "__main__":
    unittest.main(verbosity=2)
