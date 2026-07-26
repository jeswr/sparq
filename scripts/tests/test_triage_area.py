#!/usr/bin/env python3
# [OPUS-5] Hermetic tests for scripts/triage-area.py — the `needs:area` backlog
# classifier that clears the migration's 257-issue dispatch block (#1135).
#
# These are DELIBERATELY not a copy of the script's own `--self-test`. That one
# proves the rules do what the author meant; this one proves the two properties
# that make the tool SAFE to point at 257 live issues:
#
#   1. FAIL CLOSED — no evidence must yield NO label. A wrong `area:` routes a
#      worker at the wrong crate and can put two workers on one conflict
#      partition; the park is maintainer-visible and self-clearing.
#   2. NO DRIFT between the two writes — `needs:area` is removed ONLY together
#      with >=1 real `area:` label. Either half alone re-breaks dispatch (a
#      still-parked issue stays invisible; an unparked no-area issue silently
#      reserves the serializing `__global__` partition).
#
# and the invariant that stops the rule table rotting:
#
#   3. EVERY area the rule table can emit must be a label the repo ALREADY has.
#      Enumerated statically from RULES — no network.
#
# Run:  python3 scripts/tests/test_triage_area.py
# (stdlib only; no pytest required — also discoverable by `pytest`.)

from __future__ import annotations

import importlib.util
import sys
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


TA = _load("triage_area", "triage-area.py")
CRATES = TA.crate_names()


def areas(title: str, body: str = "") -> list:
    return TA.classify(title, body, CRATES)[0]


class TestFailClosed(unittest.TestCase):
    """No evidence => no label. This is the property the whole design rests on."""

    def test_no_evidence_stays_parked(self):
        for title in ("do the thing",
                      "make sure the pinned overview issue gets maintained",
                      "Recurring chore: worktree disk-hygiene sweep",
                      "LinkedIn advert post: enumerate ALL implemented specs"):
            self.assertEqual(areas(title), [], title)

    def test_declaration_naming_no_real_crate_stays_parked(self):
        # sq-lsp7k.12 declares "NEW opt-in crate" — a package that does not exist.
        # Inventing one here would be the exact misroute the park exists to avoid.
        self.assertEqual(
            areas("BI/SQL wire-protocol facade",
                  "crate_or_surface: NEW opt-in crate (pgwire-class) | effort:XL"), [])

    def test_a_body_name_drop_is_not_evidence(self):
        # A bead that merely cites a neighbouring crate in prose must NOT be
        # partitioned into it — the failure mode that produced these 257 parks.
        self.assertEqual(
            areas("formalize the codex-EC2 worker tooling",
                  "the harness has been used against sparq-core and sparq-jsonld"), [])


class TestNoDrift(unittest.TestCase):
    """`needs:area` and the `area:` labels are written in ONE call, or not at all."""

    def test_plan_emits_labels_only_for_classifiable_issues(self):
        rows = TA.plan([
            {"number": 1, "title": "DL L4: narrow the RL disjointWith guard",
             "body": "", "labels": [{"name": TA.PARK_LABEL}]},
            {"number": 2, "title": "do the thing", "body": "",
             "labels": [{"name": TA.PARK_LABEL}]},
        ], CRATES)
        self.assertEqual(rows[0][1], ["area:sparq-reason-dl"])
        self.assertEqual(rows[1][1], [])

    def test_unpark_is_never_emitted_without_an_area(self):
        # apply_row is the ONLY writer, and it always carries both halves. Assert
        # on the argv it would build rather than trusting the prose above it.
        seen = {}
        TA._gh = lambda a: seen.setdefault("argv", a) or ""
        TA.apply_row(7, ["area:sparq-core", "area:sparq-vectors"])
        argv = seen["argv"]
        self.assertIn("--remove-label", argv)
        self.assertEqual(argv[argv.index("--remove-label") + 1], TA.PARK_LABEL)
        self.assertEqual(argv.count("--add-label"), 2)

    def test_already_classified_issue_is_a_no_op(self):
        # Idempotence: a re-run must never re-decide settled work.
        rows = TA.plan([{"number": 3, "title": "DL L4: whatever", "body": "",
                         "labels": [{"name": "area:sparq-core"},
                                    {"name": TA.PARK_LABEL}]}], CRATES)
        self.assertEqual(rows[0][1], [])
        self.assertTrue(rows[0][2].startswith("SKIP"))


class TestRuleTableHygiene(unittest.TestCase):
    def test_every_emitted_area_is_a_real_crate_or_a_known_surface(self):
        """Never invent a label. An `area:` the repo does not have is invisible to
        ready-issues.py and push-frontier.sh, so the issue stays undispatchable
        while LOOKING triaged — strictly worse than the park."""
        # The non-crate surfaces the repo really carries (checked against
        # `gh label list` when this test was written, 2026-07-26).
        surfaces = {"site", "site-specs", "site-papers", "gui", "js", "bench", "ci",
                    "docs", "deps", "release", "workspace", "upstream", "e2ee",
                    "zk", "zk-xpath", "knowledge-graph", "deploy-demo"}
        for rid, _scope, _rx, emitted, _why in TA.RULES:
            for a in emitted:
                self.assertTrue(a in CRATES or a in surfaces,
                                f"rule {rid} emits unknown area {a!r}")

    def test_rule_order_survives_the_known_name_drop_collisions(self):
        """Three orderings were each MEASURED wrong on the live backlog. Pin them:
        reordering the table silently re-breaks these and nothing else would."""
        # A .typ spec that names zk/ieee754 + zk/xpath is SPEC work.
        self.assertEqual(
            areas("zksparql.typ 7.3 stale estate sentence: still names zk/ieee754 "
                  "+ zk/xpath as in-tree"), ["site-specs"])
        # An ieee754 bead whose body literally reads "FILE: zk/xpath NO -- zk/ieee754/..."
        self.assertEqual(
            areas("ieee754 OPT: gate-neutral kernels.nr cleanups",
                  "FILE: zk/xpath NO -- zk/ieee754/src/ops/kernels.nr"), ["zk"])
        # An ODRL bead whose paper recorded it is ODRL work, not paper work.
        self.assertEqual(
            areas("odrl-bridge: materialise rule-level provenance",
                  "recorded as limitation #5 in site/papers/odrl-policy-bridge.typ"),
            ["sparq-policy"])

    def test_cross_cutting_issues_keep_every_area(self):
        """The partitioner maps multi-area to __global__ deliberately. Collapsing a
        genuinely cross-crate issue to one crate to make it look dispatchable is a
        lie that lands two workers in one partition."""
        self.assertEqual(
            areas("MPC M4-v1: attestation GATE assembly",
                  "Crate: sparq-mpc (pipeline.rs/proof.rs) + sparq-zk-compose "
                  "(federated reconstruct_public_inputs reuse). The buildable M4 v1:"),
            ["sparq-mpc", "sparq-zk-compose"])
        self.assertEqual(
            areas("[epic] Proof-of-correctness program for sparq_ieee754 & noir_XPath"),
            ["zk", "zk-xpath"])

    def test_declaration_span_stops_at_the_sentence_break(self):
        """sq-p4zci's field is followed by prose containing 'docs/SKILL examples';
        running the span to end-of-line derived a spurious area:docs."""
        self.assertEqual(
            areas("Datalog: surface wiring",
                  "crates: sparq-reason + sparq-cli. CLI flag + a handle for datalog "
                  "programs; docs/SKILL examples beyond the API reference."),
            ["sparq-reason", "sparq-cli"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
