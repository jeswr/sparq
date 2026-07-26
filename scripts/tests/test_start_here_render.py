#!/usr/bin/env python3
# [OPUS-5] sparq-org/sparq#4145 — the maintainer-front-door contract (#1135).
#
# THE FAILURE THIS SUITE EXISTS TO PREVENT. #1135 is titled "auto-maintained" and never was. On
# 2026-07-26 it was 13 days stale and SEVEN of its entries were already resolved (#1848/#992/#1917
# closed, #1084/#1973 closed-unmerged, #1791/#1155 merged) while still being presented to the
# maintainer as work needing them. A front door that lists resolved work is worse than no front
# door, so every rule below is pinned by a test that goes RED when the rule is deleted:
#
#   drop-when-resolved      a closed issue / merged PR disappears .............. TestDropRules
#   FAIL CLOSED             an unreadable ref is KEPT + marked, never dropped .. TestFailClosed
#   live PR state           a CONFLICTING PR reads "needs rebase", not "arm it"  TestGatedPrTable
#   never invent an ask     every rendered ask traces to the TOML ............. TestNeverInvents
#   65 536-char limit       🟢 truncates, 🔴 never does ....................... TestTruncation
#   no no-op churn          identical body => no `gh issue edit` ............... TestPublishPath
#   sticky exit             a later success cannot clear a failure ............. TestStickyStatus
#   list API, not search    label counts come from the un-lagged API ........... TestLiveQueries
#   THE YAML SEAM           triggers/permissions/steps/inputs, parsed as YAML .. TestRefreshWorkflowWiring
#
# THE YAML SEAM IS DELIBERATELY STRUCTURAL. Measured in this repo: 18/18 Python mutants died while
# every single uncaught mutant lived in a workflow `if:`, a deleted step, or a wrong call-site
# input. A substring or `count(...) == N` assertion over workflow TEXT cannot see `if: false`, a
# removed step, or `--dry-run` quietly added to the live invocation, so this suite parses the
# workflow with PyYAML and asserts on the parsed job/step/permission/trigger objects.
#
# Run: python3 scripts/tests/test_start_here_render.py   (also run by routing-self-tests.yml)

from __future__ import annotations

import importlib.util
import io
import json
import re
import sys
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from types import ModuleType

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SCRIPT = REPO_ROOT / "scripts" / "render-start-here.py"
CONFIG = REPO_ROOT / "orchestration" / "start-here.toml"
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "refresh-start-here.yml"
SELF_TEST_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "routing-self-tests.yml"
# The live body of #1135 as it stood on 2026-07-26, the revision this TOML was seeded from. It is
# an INDEPENDENT artifact (fetched from the API, not generated here), which is what makes the
# seed-fidelity test below able to detect an ask lost in the migration.
FIXTURE = REPO_ROOT / "scripts" / "tests" / "fixtures" / "start-here-1135-2026-07-26.md"

SHA_PINNED = re.compile(r"^[^@]+@[0-9a-f]{40}$")


def _load(path: Path, name: str) -> ModuleType:
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None, path
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


rsh = _load(SCRIPT, "render_start_here_under_test")


def workflow(path: Path = WORKFLOW) -> dict:
    doc = yaml.safe_load(path.read_text(encoding="utf-8"))
    assert isinstance(doc, dict), f"{path} must parse as a mapping"
    return doc


def triggers(doc: dict) -> dict:
    # PyYAML resolves the bare key `on` to the boolean True (YAML 1.1). Read both spellings so a
    # quoted "on:" cannot make these assertions silently vacuous.
    got = doc.get("on", doc.get(True))
    assert isinstance(got, dict), "the workflow must declare a mapping of triggers"
    return got


def steps_of(doc: dict) -> list[dict]:
    out: list[dict] = []
    for job in (doc.get("jobs") or {}).values():
        out.extend(job.get("steps") or [])
    return out


# --------------------------------------------------------------------------------------------
# Fixtures
# --------------------------------------------------------------------------------------------

OPEN_ISSUE = "sparq-org/sparq#100"
CLOSED_ISSUE = "sparq-org/sparq#101"
MERGED_PR = "sparq-org/sparq#102"
CLOSED_PR = "sparq-org/sparq#103"
CLEAN_PR = "sparq-org/sparq#104"
CONFLICTING_PR = "sparq-org/sparq#105"
DRAFT_PR = "sparq-org/sparq#106"
BROKEN = "sparq-org/sparq#107"
UNKNOWN_MERGEABLE_PR = "sparq-org/sparq#108"

STATES = {
    OPEN_ISSUE: rsh.RefState(ref=OPEN_ISSUE, state="open"),
    CLOSED_ISSUE: rsh.RefState(ref=CLOSED_ISSUE, state="closed"),
    MERGED_PR: rsh.RefState(ref=MERGED_PR, is_pr=True, state="closed", merged=True),
    CLOSED_PR: rsh.RefState(ref=CLOSED_PR, is_pr=True, state="closed"),
    CLEAN_PR: rsh.RefState(
        ref=CLEAN_PR, is_pr=True, state="open", mergeable="MERGEABLE", merge_state_status="CLEAN"
    ),
    CONFLICTING_PR: rsh.RefState(
        ref=CONFLICTING_PR,
        is_pr=True,
        state="open",
        mergeable="CONFLICTING",
        merge_state_status="DIRTY",
    ),
    DRAFT_PR: rsh.RefState(
        ref=DRAFT_PR,
        is_pr=True,
        state="open",
        draft=True,
        mergeable="MERGEABLE",
        merge_state_status="BLOCKED",
    ),
    BROKEN: rsh.RefState(ref=BROKEN, error="HTTP 403: rate limited"),
    UNKNOWN_MERGEABLE_PR: rsh.RefState(
        ref=UNKNOWN_MERGEABLE_PR,
        is_pr=True,
        state="open",
        mergeable="UNKNOWN",
        merge_state_status="UNKNOWN",
    ),
}

TEXT = {
    "header": "🤖 header",
    "preamble": "verified {date}. {dropped_sentence}",
    "blocked_intro": "human-only",
    "gated_intro": "protected paths",
    "process_intro": "counts are live",
}


def cfg(entries, *, process=("{needs_user} on needs:user",), flight=("in flight",)) -> rsh.Config:
    return rsh.Config(
        repo="sparq-org/sparq",
        issue=1135,
        text=dict(TEXT),
        entries=list(entries),
        process=list(process),
        flight=list(flight),
    )


def entry(ref, bucket="blocked", ask="THE ASK", short="short", what="what", untracked=False):
    refs = [ref] if isinstance(ref, str) else list(ref)
    return rsh.Entry(
        refs=refs, bucket=bucket, ask=ask, short=short, what=what, untracked=untracked
    )


COUNTS = rsh.Counts(11, 22, 33, 44, 5)


def render(entries, **kw):
    return rsh.render(cfg(entries, **kw), STATES, COUNTS, date="2026-07-26")


# --------------------------------------------------------------------------------------------


class TestDropRules(unittest.TestCase):
    """The core bug: resolved work must LEAVE the front door by construction."""

    def test_open_entry_is_kept(self):
        # Discriminating baseline: without this, "everything is dropped" would pass every
        # drop test below.
        out = render([entry(OPEN_ISSUE, ask="STILL OPEN")])
        self.assertIn("STILL OPEN", out.body)
        self.assertEqual(out.dropped, [])
        self.assertNotIn("Removed as resolved", out.body)

    def test_closed_issue_is_dropped_and_listed_as_removed(self):
        out = render([entry(OPEN_ISSUE, ask="KEEP"), entry(CLOSED_ISSUE, ask="GONE", short="c")])
        self.assertNotIn("GONE", out.body)
        self.assertIn("KEEP", out.body)
        self.assertIn("Removed as resolved", out.body)
        self.assertIn("`#101` c (**closed**)", out.body)
        self.assertEqual([e.short for e, _ in out.dropped], ["c"])

    def test_merged_pr_is_dropped_and_reported_as_merged(self):
        out = render(
            [
                entry(OPEN_ISSUE, ask="KEEP"),
                entry(MERGED_PR, bucket="gated-pr", ask="GONE", short="m", what="w"),
            ]
        )
        self.assertNotIn("GONE", out.body)
        self.assertIn("`#102` m (**merged**)", out.body)

    def test_closed_but_unmerged_pr_is_dropped_as_closed(self):
        # #1084/#1973 were exactly this shape: PRs closed without merging. Reporting them as
        # "merged" would be a false claim about what happened to the work.
        out = render(
            [entry(OPEN_ISSUE), entry(CLOSED_PR, bucket="gated-pr", short="cp", what="w")]
        )
        self.assertIn("`#103` cp (**closed**)", out.body)
        self.assertNotIn("`#103` cp (**merged**)", out.body)

    def test_the_seven_stale_shapes_all_drop(self):
        # The literal 2026-07-26 stale set, by shape: 3 closed issues, 2 closed PRs, 2 merged PRs.
        entries = [
            entry(OPEN_ISSUE, ask="ONLY SURVIVOR"),
            entry(CLOSED_ISSUE, ask="A", short="a"),
            entry(CLOSED_PR, bucket="gated-pr", ask="B", short="b", what="w"),
            entry(MERGED_PR, bucket="gated-pr", ask="C", short="c", what="w"),
        ]
        out = render(entries)
        self.assertEqual(len(out.dropped), 3)
        for gone in ("A", "B", "C"):
            self.assertNotIn(f"| {gone} |", out.body)
        self.assertIn("ONLY SURVIVOR", out.body)
        self.assertIn("3 entries were already resolved", out.body)

    def test_untracked_entry_is_never_dropped(self):
        # sq-qhy4 (the external cryptographer audit) has no GitHub ref; it may only leave by hand.
        out = render([entry([], ask="EXTERNAL AUDIT", untracked=True)])
        self.assertIn("EXTERNAL AUDIT", out.body)
        self.assertEqual(out.dropped, [])

    def test_multi_ref_entry_drops_only_when_every_ref_is_resolved(self):
        both_open = render([entry([OPEN_ISSUE, CLEAN_PR], ask="THREE FILINGS")])
        self.assertIn("THREE FILINGS", both_open.body)
        all_done = render([entry(OPEN_ISSUE), entry([CLOSED_ISSUE, MERGED_PR], ask="ALL FILED")])
        self.assertNotIn("ALL FILED", all_done.body)

    def test_partly_resolved_multi_ref_entry_is_kept_and_annotated(self):
        out = render([entry([OPEN_ISSUE, CLOSED_ISSUE], ask="THREE FILINGS")])
        self.assertIn("THREE FILINGS", out.body)
        self.assertIn("partly resolved: #101 closed", out.body)


class TestFailClosed(unittest.TestCase):
    """A transient API error must never look like "resolved". Keeping is the safe direction."""

    def test_unreadable_ref_is_kept_and_marked_state_unknown(self):
        out = render([entry(BROKEN, ask="A P0 NOBODY ELSE CAN DO")])
        self.assertIn("A P0 NOBODY ELSE CAN DO", out.body)
        self.assertIn("state unknown", out.body)
        self.assertEqual(out.dropped, [])
        self.assertEqual(len(out.unknown), 1)

    def test_missing_state_for_a_ref_is_unknown_not_open(self):
        # `states` simply not containing the ref (a fetch that never happened) must land in the
        # SAME fail-closed branch as an error, and be reported as degraded rather than silently OK.
        out = rsh.render(
            cfg([entry("sparq-org/sparq#999", ask="UNFETCHED")]),
            {},
            COUNTS,
            date="2026-07-26",
        )
        self.assertIn("UNFETCHED", out.body)
        self.assertEqual(out.dropped, [])
        self.assertEqual(len(out.unknown), 1)

    def test_refstate_is_resolved_requires_positive_evidence(self):
        self.assertFalse(rsh.RefState(ref="r", error="boom").is_resolved())
        self.assertFalse(rsh.RefState(ref="r", state="").is_resolved())
        self.assertFalse(rsh.RefState(ref="r", state="open").is_resolved())
        self.assertTrue(rsh.RefState(ref="r", state="closed").is_resolved())
        self.assertTrue(rsh.RefState(ref="r", state="open", merged=True).is_resolved())

    def test_malformed_payload_is_unknown_not_open(self):
        # A 200 with a payload that has no usable `state` (a proxy error page, a schema change)
        # must be UNKNOWN. Treating it as "not closed" would be luck, not a rule.
        original = rsh._gh
        try:
            rsh._gh = lambda args: json.dumps({"nonsense": True})
            state = rsh.live_ref_state("sparq-org/sparq#7")
        finally:
            rsh._gh = original
        self.assertTrue(state.unknown)
        self.assertFalse(state.is_resolved())

    def test_pr_detail_failure_leaves_the_entry_unknown_not_mergeable(self):
        original = rsh._gh

        def fake(args):
            if args[0] == "api":
                return json.dumps({"state": "open", "pull_request": {"merged_at": None}})
            raise RuntimeError("gh pr view exploded")

        try:
            rsh._gh = fake
            state = rsh.live_ref_state("sparq-org/sparq#8")
        finally:
            rsh._gh = original
        self.assertTrue(state.unknown)
        self.assertEqual(rsh._pr_state_cell(state), "⚠️ state unknown")

    def test_a_partially_read_pr_is_unknown_even_though_it_looks_mergeable(self):
        # DISCRIMINATING case for the `state.unknown` short-circuit in _pr_state_cell: a ref whose
        # error is set but whose mergeable field happens to be populated (a read that failed
        # half-way, a cached value). Falling through to the mergeable branch would print
        # "MERGEABLE" about a PR we did not actually manage to read.
        half_read = rsh.RefState(
            ref="sparq-org/sparq#9",
            is_pr=True,
            state="open",
            mergeable="MERGEABLE",
            merge_state_status="CLEAN",
            error="connection reset after the first read",
        )
        self.assertEqual(rsh._pr_state_cell(half_read), "⚠️ state unknown")
        self.assertFalse(half_read.is_resolved())

    def test_an_unexpected_state_string_is_unknown_not_trusted(self):
        # `{"state": "reopened"}` / a non-string state must land in the UNKNOWN branch. Without the
        # payload check the value is trusted verbatim, and `42.lower()` raises deep inside the
        # render instead of being reported as an unreadable ref.
        original = rsh._gh
        try:
            for payload in ({"state": "reopened"}, {"state": 42}, {"state": None}, {}):
                rsh._gh = lambda args, p=payload: json.dumps(p)
                state = rsh.live_ref_state("sparq-org/sparq#7")
                self.assertTrue(state.unknown, f"{payload} must be UNKNOWN")
                self.assertFalse(state.is_resolved())
                self.assertEqual(rsh._pr_state_cell(state), "⚠️ state unknown")
        finally:
            rsh._gh = original

    def test_degradation_is_recorded_for_every_unreadable_ref(self):
        status = rsh.Status()
        states = rsh.collect_states(
            cfg([entry(BROKEN), entry(OPEN_ISSUE)]),
            status,
            fetch=lambda ref: STATES[ref],
        )
        self.assertEqual(status.code, rsh.EXIT_DEGRADED)
        self.assertTrue(any(BROKEN in n for n in status.notes))
        self.assertEqual(len(states), 2)


class TestStickyStatus(unittest.TestCase):
    """A later transient must never discard an earlier earned failure (three prior outages)."""

    def test_status_escalates_and_never_downgrades(self):
        status = rsh.Status()
        self.assertEqual(status.code, rsh.EXIT_OK)
        status.degrade("first failure")
        self.assertEqual(status.code, rsh.EXIT_DEGRADED)

    def test_a_later_lower_severity_report_cannot_clear_an_earlier_failure(self):
        # The exact shape of the three prior outages: a LATER transient (reported at a lower
        # severity) overwriting an EARLIER earned hard failure. `degrade` must be a monotone
        # maximum, not an assignment.
        status = rsh.Status()
        status.degrade("a ref could not be read", code=rsh.EXIT_DEGRADED)
        status.degrade("something benign happened afterwards", code=rsh.EXIT_OK)
        self.assertEqual(status.code, rsh.EXIT_DEGRADED)
        self.assertTrue(status.degraded)

    def test_interleaved_sequence_keeps_the_failure(self):
        # failure -> successes -> failure -> successes must still exit non-zero.
        status = rsh.Status()
        status.degrade("ref A unreadable")
        for _ in range(5):
            rsh.collect_states(cfg([entry(OPEN_ISSUE)]), status, fetch=lambda ref: STATES[ref])
        self.assertEqual(status.code, rsh.EXIT_DEGRADED)
        self.assertTrue(status.degraded)


class TestGatedPrTable(unittest.TestCase):
    """The table must describe the PR that exists NOW, not the one we last looked at."""

    def _row(self, body: str, number: str) -> str:
        rows = [ln for ln in body.splitlines() if ln.startswith(f"| **[#{number}]")]
        self.assertEqual(len(rows), 1, f"expected exactly one row for #{number}")
        return rows[0]

    def test_clean_pr_keeps_the_curated_ask(self):
        body = render([entry(CLEAN_PR, bucket="gated-pr", ask='"arm it"', what="w")]).body
        row = self._row(body, "104")
        self.assertIn("MERGEABLE", row)
        self.assertIn('"arm it"', row)

    def test_conflicting_pr_is_annotated_and_never_says_arm_it(self):
        body = render([entry(CONFLICTING_PR, bucket="gated-pr", ask='"arm it"', what="w")]).body
        row = self._row(body, "105")
        self.assertIn("⚠️ CONFLICTING", row)
        self.assertIn(rsh.REBASE_ASK, row)
        self.assertNotIn('"arm it"', row)

    def test_draft_blocked_pr_reports_both_facts(self):
        body = render([entry(DRAFT_PR, bucket="gated-pr", ask="decide the scope", what="w")]).body
        self.assertIn("draft, BLOCKED", self._row(body, "106"))

    def test_unknown_mergeability_is_not_reported_as_mergeable(self):
        # GitHub returns mergeable=UNKNOWN while it recomputes. Printing "MERGEABLE" there invites
        # the maintainer to arm a PR that cannot merge.
        body = render(
            [entry(UNKNOWN_MERGEABLE_PR, bucket="gated-pr", ask='"arm it"', what="w")]
        ).body
        row = self._row(body, "108")
        self.assertIn("state unknown", row)
        self.assertNotIn("MERGEABLE", row)

    def test_pr_links_point_at_pull_not_issues(self):
        body = render([entry(CLEAN_PR, bucket="gated-pr", what="w")]).body
        self.assertIn("https://github.com/sparq-org/sparq/pull/104", body)


class TestNeverInvents(unittest.TestCase):
    """The renderer may DROP, ANNOTATE and ORDER. It may not author an ask."""

    @staticmethod
    def _blocked_items(body: str) -> list[str]:
        """Reassemble each 🔴 item from its wrapped lines, whitespace-normalised.

        Deliberately NOT a first-line-prefix check: comparing only the first rendered line lets
        anything appended to the tail of an ask through, which is exactly the "invents an ask"
        failure this test is named after.
        """
        section = body.split("## 🔴 Only you can unblock", 1)[1].split("\n## ", 1)[0]
        items: list[str] = []
        for line in section.splitlines():
            if re.match(r"^\d+\. ", line):
                items.append(line.split(". ", 1)[1])
            elif items and line.startswith("   "):
                items[-1] += " " + line.strip()
        return [" ".join(i.split()) for i in items]

    def test_every_rendered_blocked_item_is_verbatim_from_the_toml(self):
        config = rsh.load_config(CONFIG)
        states = {
            ref: rsh.RefState(ref=ref, state="open", is_pr=True, mergeable="MERGEABLE",
                              merge_state_status="CLEAN")
            for e in config.entries
            for ref in e.refs
        }
        body = rsh.render(config, states, COUNTS, date="2026-07-26").body
        curated = {" ".join(e.ask.split()) for e in config.entries}
        items = self._blocked_items(body)
        self.assertEqual(len(items), len(config.bucket("blocked")))
        for item in items:
            # EXACT equality: with every ref open and mergeable, no annotation applies, so any
            # difference at all is the renderer authoring copy.
            self.assertIn(item, curated, f"rendered 🔴 item is not verbatim TOML: {item[:90]}")

    def test_the_only_additions_are_the_declared_annotations(self):
        # And when an annotation DOES apply, it must be exactly one of the two declared ones.
        config = rsh.load_config(CONFIG)
        states = {
            ref: rsh.RefState(ref=ref, error="HTTP 502")
            for e in config.entries
            for ref in e.refs
        }
        body = rsh.render(config, states, COUNTS, date="2026-07-26").body
        curated = {" ".join(e.ask.split()) for e in config.entries}
        for item in self._blocked_items(body):
            stripped = item.replace(f" {rsh.UNKNOWN_MARK}.", "")
            self.assertIn(stripped, curated, f"unexpected addition: {item[-90:]}")

    def test_every_table_ask_cell_is_a_curated_ask_or_the_rebase_annotation(self):
        config = rsh.load_config(CONFIG)
        states = {
            ref: rsh.RefState(ref=ref, is_pr=True, state="open", mergeable="CONFLICTING",
                              merge_state_status="DIRTY")
            for e in config.entries
            for ref in e.refs
        }
        body = rsh.render(config, states, COUNTS, date="2026-07-26").body
        curated = {" ".join(e.ask.split()) for e in config.entries} | {rsh.REBASE_ASK}
        for line in body.splitlines():
            if not line.startswith("| **[#"):
                continue
            cells = [c.strip() for c in line.strip("|").split("|")]
            self.assertIn(cells[-1], curated, f"invented ask cell: {cells[-1][:80]}")

    def test_a_ref_absent_from_the_toml_never_appears(self):
        body = render([entry(OPEN_ISSUE)]).body
        self.assertNotIn("#404", body)

    def test_unsubstituted_placeholder_is_refused(self):
        # The real defect: `{needs_ec2}` (a digit in the name) slipped past the substitution regex
        # and shipped literally to the maintainer. Nothing placeholder-shaped may reach the body.
        with self.assertRaises(rsh.RenderAborted):
            render([entry(OPEN_ISSUE)], process=("{needs_EC2} issues",))

    def test_unknown_placeholder_name_fails_validation(self):
        doc = self._minimal_doc()
        doc["process"] = [{"text": "{needs_ec3} issues"}]
        with self.assertRaises(rsh.RenderAborted):
            rsh.parse_config(doc)

    def test_known_placeholders_are_substituted(self):
        body = render([entry(OPEN_ISSUE)], process=("{needs_user}/{needs_area}/{needs_ec2}",)).body
        self.assertIn("11/22/33", body)

    @staticmethod
    def _minimal_doc() -> dict:
        return {
            "meta": {"repo": "sparq-org/sparq", "issue": 1135},
            "text": dict(TEXT),
            "entry": [
                {"ref": OPEN_ISSUE, "bucket": "blocked", "ask": "a", "short": "s"},
            ],
        }


class TestMarkdownIntegrity(unittest.TestCase):
    """A wrapped link is a dead link — and the link IS the ask."""

    def _live_body(self) -> str:
        config = rsh.load_config(CONFIG)
        states = {
            ref: rsh.RefState(ref=ref, is_pr=True, state="open", mergeable="MERGEABLE",
                              merge_state_status="CLEAN")
            for e in config.entries
            for ref in e.refs
        }
        return rsh.render(config, states, COUNTS, date="2026-07-26").body

    def test_no_link_destination_is_broken_across_a_line(self):
        # textwrap's defaults split `https://github.com/sparq-org/...` at the hyphen, and a newline
        # inside a markdown link destination silently un-links it — so the maintainer loses the link
        # to the very thing they are being asked about. Measured on the first render of this file.
        body = self._live_body()
        for target in re.findall(r"\]\(([^)]*)\)", body):
            self.assertNotIn("\n", target, f"wrapped link destination: {target[:60]}")
            self.assertNotRegex(target, r"\s", f"whitespace in link destination: {target[:60]}")

    def test_every_markdown_link_is_closed(self):
        body = self._live_body()
        self.assertEqual(body.count("]("), len(re.findall(r"\]\([^)\s]+\)", body)))

    def test_table_rows_have_the_declared_column_count(self):
        # An unescaped pipe in curated copy shifts every later cell, which is how a "State" column
        # ends up displaying half an ask. Checked per SECTION, since the two tables differ in width.
        body = self._live_body()
        for header, width in (
            ("## 🟡 Decisions", 2),
            ("## 🟡 Maintainer-gated PRs", 4),
        ):
            section = body.split(header, 1)[1].split("\n## ", 1)[0]
            rows = [ln for ln in section.splitlines() if ln.startswith("| **[#")]
            self.assertTrue(rows, f"no rows rendered under {header}")
            for row in rows:
                self.assertEqual(len(row.strip("|").split("|")), width, row[:80])


class TestConfigValidation(unittest.TestCase):
    """A malformed curated file must fail loudly rather than render half a door."""

    def doc(self, **entry_overrides) -> dict:
        base = {"ref": OPEN_ISSUE, "bucket": "blocked", "ask": "a", "short": "s"}
        base.update(entry_overrides)
        return {
            "meta": {"repo": "sparq-org/sparq", "issue": 1135},
            "text": dict(TEXT),
            "entry": [base],
        }

    def test_the_shipped_toml_validates(self):
        config = rsh.load_config(CONFIG)
        self.assertEqual(config.repo, "sparq-org/sparq")
        self.assertEqual(config.issue, 1135)
        self.assertTrue(config.bucket("blocked"))
        self.assertTrue(config.bucket("decision"))
        self.assertTrue(config.bucket("gated-pr"))

    def test_unknown_bucket_is_rejected(self):
        with self.assertRaises(rsh.RenderAborted):
            rsh.parse_config(self.doc(bucket="urgent"))

    def test_missing_short_is_rejected(self):
        # `short` is what the removed-as-resolved line prints; without it a dropped entry becomes
        # an unattributed number.
        with self.assertRaises(rsh.RenderAborted):
            rsh.parse_config(self.doc(short=""))

    def test_a_refless_entry_must_declare_untracked(self):
        with self.assertRaises(rsh.RenderAborted):
            rsh.parse_config(self.doc(ref=""))
        rsh.parse_config(self.doc(ref="", untracked=True))  # declared => fine

    def test_untracked_contradicting_a_ref_is_rejected(self):
        with self.assertRaises(rsh.RenderAborted):
            rsh.parse_config(self.doc(untracked=True))

    def test_malformed_ref_is_rejected(self):
        for bad in ("#123", "sparq-org/sparq", "sparq-org/sparq#0", "https://…/issues/1"):
            with self.assertRaises(rsh.RenderAborted):
                rsh.parse_config(self.doc(ref=bad))

    def test_cross_repo_ref_is_accepted_and_labelled_with_its_repo(self):
        config = rsh.parse_config(self.doc(ref="jeswr/agent-account-registry#278"))
        self.assertEqual(config.entries[0].refs, ["jeswr/agent-account-registry#278"])
        self.assertEqual(rsh._ref_label("jeswr/agent-account-registry#278"),
                         "jeswr/agent-account-registry#278")

    def test_gated_pr_entry_needs_what_and_exactly_one_ref(self):
        with self.assertRaises(rsh.RenderAborted):
            rsh.parse_config(self.doc(bucket="gated-pr", what=""))
        with self.assertRaises(rsh.RenderAborted):
            rsh.parse_config(self.doc(bucket="gated-pr", what="w", ref=[OPEN_ISSUE, CLEAN_PR]))

    def test_a_pipe_in_a_table_cell_is_rejected(self):
        with self.assertRaises(rsh.RenderAborted):
            rsh.parse_config(self.doc(bucket="decision", ask="A | B"))

    def test_an_empty_entry_list_is_rejected(self):
        doc = self.doc()
        doc["entry"] = []
        with self.assertRaises(rsh.RenderAborted):
            rsh.parse_config(doc)


class TestTruncation(unittest.TestCase):
    """65 536 chars is a hard GitHub limit. 🟢 gives way; 🔴 never does."""

    RED = "THE UNPUBLISHED NPM PACKAGE — ONLY THE MAINTAINER CAN DO THIS"

    def _oversized(self, limit=6000, flight_items=40):
        entries = [entry(OPEN_ISSUE, ask=self.RED), entry(CLEAN_PR, bucket="decision", ask="DECIDE")]
        config = cfg(entries, flight=[f"flight item {i} " + "x" * 400 for i in range(flight_items)])
        return rsh.render(config, STATES, COUNTS, date="2026-07-26", limit=limit)

    def test_oversized_body_truncates_green_and_keeps_red(self):
        out = self._oversized()
        self.assertLessEqual(len(out.body), 6000)
        self.assertIn(self.RED, out.body)
        self.assertIn("DECIDE", out.body)
        self.assertIn("truncated to fit", out.body)

    def test_untruncated_body_keeps_every_green_item(self):
        # Discrimination: the truncation path must be entered only when needed.
        out = self._oversized(limit=rsh.BODY_LIMIT)
        self.assertIn("flight item 39", out.body)
        self.assertNotIn("truncated to fit", out.body)

    def test_green_items_are_dropped_from_the_tail_first(self):
        out = self._oversized(limit=4000)
        self.assertIn("flight item 0", out.body)
        self.assertNotIn("flight item 39", out.body)

    def test_removed_list_gives_way_before_the_process_notes(self):
        entries = [entry(OPEN_ISSUE, ask=self.RED)] + [
            rsh.Entry(refs=[CLOSED_ISSUE], bucket="blocked", ask="gone", short=f"resolved-{i}")
            for i in range(200)
        ]
        config = cfg(entries, process=("PROCESS NOTE {needs_user}",), flight=("green",))
        out = rsh.render(config, STATES, COUNTS, date="2026-07-26", limit=2600)
        self.assertLessEqual(len(out.body), 2600)
        self.assertIn(self.RED, out.body)
        self.assertIn("resolved entries were dropped", out.body)
        self.assertNotIn("resolved-199", out.body)

    def test_a_red_section_that_cannot_fit_aborts_instead_of_being_cut(self):
        entries = [
            rsh.Entry(refs=[OPEN_ISSUE], bucket="blocked", ask="RED " + "y" * 900, short=f"r{i}")
            for i in range(20)
        ]
        with self.assertRaises(rsh.RenderAborted):
            rsh.render(cfg(entries), STATES, COUNTS, date="2026-07-26", limit=3000)

    def test_the_live_body_fits_with_room_to_spare(self):
        config = rsh.load_config(CONFIG)
        states = {
            ref: rsh.RefState(ref=ref, state="open", is_pr=True, mergeable="MERGEABLE",
                              merge_state_status="CLEAN")
            for e in config.entries
            for ref in e.refs
        }
        body = rsh.render(config, states, COUNTS, date="2026-07-26").body
        self.assertLess(len(body), rsh.BODY_LIMIT)
        self.assertNotIn("truncated to fit", body)


class FakeGh:
    """Intercepts every `gh` invocation the renderer makes, and records it."""

    def __init__(self, *, body="stale body", fail=(), counts=3):
        self.calls: list[list[str]] = []
        self.published: list[str] = []
        self.body = body
        self.fail = tuple(fail)
        self.counts = counts

    def __call__(self, args: list[str]) -> str:
        self.calls.append(list(args))
        joined = " ".join(args)
        for token in self.fail:
            if token in joined:
                raise RuntimeError(f"simulated failure for {token}")
        if args[:1] == ["api"]:
            return json.dumps({"state": "open"})
        if args[:2] == ["pr", "view"]:
            return json.dumps(
                {"isDraft": False, "mergeable": "MERGEABLE", "mergeStateStatus": "CLEAN"}
            )
        if args[:2] == ["issue", "list"]:
            return json.dumps([{"number": i} for i in range(self.counts)])
        if args[:2] == ["pr", "list"]:
            return json.dumps([{"number": 1, "mergeable": "CONFLICTING"}])
        if args[:2] == ["issue", "view"]:
            return json.dumps({"body": self.body})
        if args[:2] == ["issue", "edit"]:
            path = args[args.index("--body-file") + 1]
            self.published.append(Path(path).read_text(encoding="utf-8"))
            return ""
        raise AssertionError(f"unexpected gh call: {args}")

    def edits(self) -> list[list[str]]:
        return [c for c in self.calls if c[:2] == ["issue", "edit"]]


class MainHarness(unittest.TestCase):
    def run_main(self, argv, fake: FakeGh):
        original = rsh._gh
        rsh._gh = fake
        buf = io.StringIO()
        try:
            with redirect_stdout(buf):
                code = rsh.main(argv)
        finally:
            rsh._gh = original
        return code, buf.getvalue()


class TestPublishPath(MainHarness):
    """No no-op churn; a degraded run still publishes but still reds; an aggregate failure aborts."""

    ARGV = ["--config", str(CONFIG)]

    def test_edit_happens_when_the_body_differs(self):
        fake = FakeGh(body="a stale hand-written body")
        code, _ = self.run_main(self.ARGV, fake)
        self.assertEqual(code, rsh.EXIT_OK)
        self.assertEqual(len(fake.edits()), 1)
        self.assertIn("🔴 Only you can unblock", fake.published[0])

    def test_identical_body_is_not_re_published(self):
        # Notification spam is how a front door gets muted, so an unchanged render must not edit.
        fake = FakeGh(body="whatever")
        self.run_main(self.ARGV, fake)
        rendered = fake.published[0]
        again = FakeGh(body=rendered)
        code, _ = self.run_main(self.ARGV, again)
        self.assertEqual(code, rsh.EXIT_OK)
        self.assertEqual(again.edits(), [], "an identical body must not be re-published")

    def test_dry_run_touches_nothing_and_prints_the_body(self):
        fake = FakeGh(body="stale")
        code, printed = self.run_main([*self.ARGV, "--dry-run"], fake)
        self.assertEqual(code, rsh.EXIT_OK)
        self.assertEqual(fake.edits(), [])
        self.assertNotIn(["issue", "view"], [c[:2] for c in fake.calls])
        self.assertIn("🔴 Only you can unblock", printed)

    def test_a_label_count_failure_aborts_without_publishing(self):
        # Aggregate numbers have no per-item "unknown" marker to hang on, so a body built from a
        # failed count would be a confident lie. Abort, publish nothing.
        fake = FakeGh(body="stale", fail=("issue list",))
        code, _ = self.run_main(self.ARGV, fake)
        self.assertEqual(code, rsh.EXIT_ABORT)
        self.assertEqual(fake.edits(), [])

    def test_a_held_pr_list_failure_aborts_without_publishing(self):
        fake = FakeGh(body="stale", fail=("pr list",))
        code, _ = self.run_main(self.ARGV, fake)
        self.assertEqual(code, rsh.EXIT_ABORT)
        self.assertEqual(fake.edits(), [])

    def test_a_per_ref_failure_publishes_everything_but_still_exits_non_zero(self):
        # THE WORST FAILURE would be silently omitting a P0 because of a transient 403. So the
        # entry stays, marked, and the process exits non-zero so the job reds.
        fake = FakeGh(body="stale", fail=("repos/sparq-org/sparq/issues/53",))
        code, _ = self.run_main(self.ARGV, fake)
        self.assertEqual(code, rsh.EXIT_DEGRADED)
        self.assertEqual(len(fake.edits()), 1)
        published = fake.published[0]
        self.assertIn("npm bootstrap publish", published)
        self.assertIn("state unknown", published)
        self.assertIn("ran DEGRADED", published)

    def test_an_edit_failure_aborts_non_zero(self):
        fake = FakeGh(body="stale", fail=("issue edit",))
        code, _ = self.run_main(self.ARGV, fake)
        self.assertEqual(code, rsh.EXIT_ABORT)

    def test_if_ref_for_an_unlisted_ref_costs_nothing(self):
        # The cost guard: at 60 merges/hour a full render per close would exceed GITHUB_TOKEN's
        # 1000 requests/hour/repository budget.
        fake = FakeGh(body="stale")
        code, _ = self.run_main([*self.ARGV, "--if-ref", "sparq-org/sparq#999999"], fake)
        self.assertEqual(code, rsh.EXIT_OK)
        self.assertEqual(fake.calls, [], "an unlisted ref must not cost a single API call")

    def test_if_ref_for_a_listed_ref_renders(self):
        fake = FakeGh(body="stale")
        code, _ = self.run_main([*self.ARGV, "--if-ref", "sparq-org/sparq#53"], fake)
        self.assertEqual(code, rsh.EXIT_OK)
        self.assertEqual(len(fake.edits()), 1)

    def test_a_broken_toml_aborts_before_any_api_call(self):
        fake = FakeGh(body="stale")
        code, _ = self.run_main(["--config", str(FIXTURE)], fake)  # markdown, not TOML
        self.assertEqual(code, rsh.EXIT_ABORT)
        self.assertEqual(fake.calls, [])


class TestLiveQueries(MainHarness):
    """Which API the counts come from is a correctness question, not a style one."""

    def test_counts_use_the_list_api_not_the_lagging_search_index(self):
        # MEASURED 2026-07-26: search/issues reported 34 issues on needs:ec2 while the list API
        # reported 35. Rendering a stale number as "live" is the same class of dishonesty this
        # whole issue is about.
        fake = FakeGh(body="stale")
        self.run_main(["--config", str(CONFIG), "--dry-run"], fake)
        joined = [" ".join(c) for c in fake.calls]
        self.assertTrue(any(c.startswith("issue list") for c in joined))
        self.assertFalse(
            any("search/issues" in c or "--search" in c for c in joined),
            "label populations must not be read from the search index",
        )

    def test_every_needs_label_population_is_queried(self):
        fake = FakeGh(body="stale")
        self.run_main(["--config", str(CONFIG), "--dry-run"], fake)
        joined = " || ".join(" ".join(c) for c in fake.calls)
        for label in ("needs:user", "needs:area", "needs:ec2"):
            self.assertIn(f"--label {label}", joined)
        self.assertIn("--label review:needs-user", joined)

    def test_held_pr_conflict_count_is_derived_from_the_live_list(self):
        counts = rsh.live_counts.__wrapped__ if hasattr(rsh.live_counts, "__wrapped__") else None
        self.assertIsNone(counts)  # no caching decorator may hide a stale count
        fake = FakeGh(body="stale")
        original = rsh._gh
        rsh._gh = fake
        try:
            got = rsh.live_counts("sparq-org/sparq")
        finally:
            rsh._gh = original
        self.assertEqual(got.held_prs, 1)
        self.assertEqual(got.held_prs_conflicting, 1)


class TestSeedFidelity(unittest.TestCase):
    """The migration from the hand-written body must not have silently lost an ask."""

    def test_every_ref_in_the_2026_07_26_body_survives_the_migration(self):
        fixture = FIXTURE.read_text(encoding="utf-8")
        config = rsh.load_config(CONFIG)
        states = {
            ref: rsh.RefState(ref=ref, is_pr=True, state="open", mergeable="MERGEABLE",
                              merge_state_status="CLEAN")
            for e in config.entries
            for ref in e.refs
        }
        body = rsh.render(config, states, COUNTS, date="2026-07-26").body
        wanted = set(re.findall(r"#(\d{2,5})\b", fixture))
        got = set(re.findall(r"#(\d{2,5})\b", body))
        self.assertEqual(
            wanted - got,
            set(),
            "these refs were on the maintainer's front door on 2026-07-26 and are not in the "
            "rendered output — an ask was lost in the migration to the TOML",
        )

    def test_the_seven_stale_entries_are_seeded_so_the_first_render_drops_them(self):
        config = rsh.load_config(CONFIG)
        refs = {ref for e in config.entries for ref in e.refs}
        for number in (1848, 992, 1917, 1084, 1791, 1155, 1973):
            self.assertIn(f"sparq-org/sparq#{number}", refs)

    def test_the_curated_gated_pr_asks_never_hard_code_a_state(self):
        # The State column is LIVE. A hand-written "CONFLICTING"/"MERGEABLE" in the curated copy is
        # exactly the decay this issue is about.
        config = rsh.load_config(CONFIG)
        for e in config.bucket("gated-pr"):
            for token in ("CONFLICTING", "MERGEABLE", "BLOCKED"):
                self.assertNotIn(token, e.ask + e.what, f"{e.short}: state belongs to live data")


class TestRefreshWorkflowWiring(unittest.TestCase):
    """THE YAML SEAM — parsed structurally, because text assertions cannot see `if: false`."""

    def setUp(self):
        self.doc = workflow()
        self.jobs = self.doc.get("jobs") or {}
        self.job = self.jobs.get("refresh") or {}
        self.steps = self.job.get("steps") or []

    # --- triggers -----------------------------------------------------------------------------
    def test_schedule_trigger_exists(self):
        crons = [c.get("cron") for c in triggers(self.doc).get("schedule") or []]
        self.assertTrue(crons, "without a cron the front door only refreshes on close events")
        for cron in crons:
            self.assertRegex(str(cron), r"^\S+( \S+){4}$")

    def test_schedule_runs_several_times_a_day(self):
        crons = [str(c["cron"]) for c in triggers(self.doc)["schedule"]]
        hours = set()
        for cron in crons:
            hour_field = cron.split()[1]
            hours.update(hour_field.split(",")) if hour_field != "*" else hours.add("*")
        self.assertGreaterEqual(len(hours), 2, "#4145 asks for a few refreshes a day")

    def test_close_events_are_wired_for_prompt_removal(self):
        got = triggers(self.doc)
        self.assertEqual((got.get("issues") or {}).get("types"), ["closed"])
        self.assertEqual((got.get("pull_request") or {}).get("types"), ["closed"])

    def test_workflow_dispatch_exists(self):
        self.assertIn("workflow_dispatch", triggers(self.doc))

    # --- permissions --------------------------------------------------------------------------
    def test_workflow_level_permissions_are_read_only(self):
        self.assertEqual(self.doc.get("permissions"), {"contents": "read"})

    def test_the_job_has_exactly_the_writes_it_needs(self):
        perms = self.job.get("permissions")
        self.assertIsInstance(perms, dict, "the job must declare explicit permissions")
        self.assertEqual(perms.get("issues"), "write", "it edits #1135")
        self.assertEqual(perms.get("pull-requests"), "read")
        self.assertEqual(perms.get("contents"), "read", "no contents:write — it pushes nothing")
        self.assertNotIn("actions", perms)

    # --- steps --------------------------------------------------------------------------------
    def test_every_action_is_sha_pinned(self):
        for step in self.steps:
            if "uses" in step:
                self.assertRegex(str(step["uses"]).split(" #")[0], SHA_PINNED.pattern)

    def test_checkout_does_not_persist_credentials(self):
        checkout = [s for s in self.steps if str(s.get("uses", "")).startswith("actions/checkout@")]
        self.assertEqual(len(checkout), 1)
        self.assertIs((checkout[0].get("with") or {}).get("persist-credentials"), False)

    def _render_steps(self) -> list[dict]:
        return [s for s in self.steps if "render-start-here.py" in str(s.get("run", ""))]

    def test_the_renderer_is_actually_invoked(self):
        self.assertTrue(self._render_steps(), "a workflow that never runs the renderer is theatre")

    def test_the_self_test_runs_before_the_live_render(self):
        runs = [str(s.get("run", "")) for s in self.steps]
        self_test = [i for i, r in enumerate(runs) if "--self-test" in r]
        live = [i for i, r in enumerate(runs) if self._is_live_invocation(r)]
        self.assertTrue(self_test, "the hermetic self-test step is missing")
        self.assertTrue(live, "the live render step is missing")
        self.assertLess(min(self_test), min(live), "validate the TOML before rewriting the issue")

    @staticmethod
    def _is_live_invocation(run: str) -> bool:
        for line in run.splitlines():
            line = line.strip()
            if not line.startswith("python3 scripts/render-start-here.py"):
                continue
            if "--self-test" in line:
                continue
            return True
        return False

    def test_the_live_step_is_not_secretly_a_dry_run(self):
        # The classic wrong-input mutant: `--dry-run` on the live step makes the workflow green
        # forever while the front door never changes.
        live = [s for s in self.steps if self._is_live_invocation(str(s.get("run", "")))]
        self.assertTrue(live)
        for step in live:
            self.assertNotIn("--dry-run", str(step["run"]))

    def test_the_live_step_has_a_token(self):
        for step in self.steps:
            if self._is_live_invocation(str(step.get("run", ""))):
                self.assertIn("GH_TOKEN", (step.get("env") or {}))

    def test_close_events_pass_the_if_ref_cost_guard(self):
        run = "\n".join(str(s.get("run", "")) for s in self.steps)
        self.assertIn("--if-ref", run, "an on-close run must not cost a full render")
        self.assertIn("github.event.issue.number", run)
        self.assertIn("github.event.pull_request.number", run)

    def test_the_scripts_the_workflow_names_exist(self):
        run = "\n".join(str(s.get("run", "")) for s in self.steps)
        for path in re.findall(r"python3 (scripts/[\w./-]+)", run):
            self.assertTrue((REPO_ROOT / path).is_file(), f"{path} does not exist")

    # --- disabling ----------------------------------------------------------------------------
    def test_no_job_or_step_is_disabled(self):
        # `if: false` is invisible to a substring assertion and turns the whole fix into a no-op.
        falsey = (False, "false", "${{ false }}", "0", None)
        for name, job in self.jobs.items():
            if "if" in job:
                self.assertNotIn(job["if"], falsey, f"job {name} is disabled")
            for i, step in enumerate(job.get("steps") or []):
                if "if" in step:
                    self.assertNotIn(step["if"], falsey, f"{name} step {i} is disabled")

    def test_fork_pull_request_events_are_skipped(self):
        # A fork PR close gets a read-only token, so the edit can only fail. Without this guard the
        # job reds on every external contributor's PR and the real failures get ignored.
        guard = " ".join(str(self.job.get("if", "")).split())
        self.assertIn("github.event.pull_request.head.repo.full_name == github.repository", guard)
        self.assertIn("github.event_name != 'pull_request'", guard)

    def test_concurrency_is_serialised_with_a_cron_backstop(self):
        concurrency = self.doc.get("concurrency")
        self.assertIsInstance(concurrency, dict)
        self.assertTrue(concurrency.get("group"))
        # cancel-in-progress is only safe BECAUSE a schedule re-renders unconditionally.
        if concurrency.get("cancel-in-progress"):
            self.assertTrue(triggers(self.doc).get("schedule"))


class TestSelfTestWorkflowWiring(unittest.TestCase):
    """A suite no workflow runs is not a gate (the ready-issues.py regression, #3423)."""

    def setUp(self):
        self.doc = workflow(SELF_TEST_WORKFLOW)

    def test_this_suite_and_the_renderer_self_test_are_invoked(self):
        runs = [
            str(step.get("run", ""))
            for job in (self.doc.get("jobs") or {}).values()
            for step in (job.get("steps") or [])
        ]
        joined = "\n".join(runs)
        self.assertIn("python3 scripts/render-start-here.py --self-test", joined)
        self.assertIn("python3 scripts/tests/test_start_here_render.py", joined)

    def test_every_front_door_file_is_a_path_trigger_on_both_events(self):
        got = triggers(self.doc)
        for event in ("pull_request", "push"):
            paths = (got.get(event) or {}).get("paths") or []
            for needed in (
                "orchestration/start-here.toml",
                "scripts/render-start-here.py",
                "scripts/tests/test_start_here_render.py",
                ".github/workflows/refresh-start-here.yml",
            ):
                self.assertIn(needed, paths, f"{needed} must re-run this gate on {event}")

    def test_the_gate_job_is_not_disabled_and_stays_gate_discoverable(self):
        for name, job in (self.doc.get("jobs") or {}).items():
            self.assertNotIn(job.get("if", "unset"), (False, "false"), f"job {name} disabled")
            job_name = str(job.get("name", "")).lower()
            # ci-summary treats a sibling as GATING iff its name says neither of these.
            self.assertNotIn("advisory", job_name)
            self.assertNotIn("informational", job_name)


if __name__ == "__main__":
    unittest.main(verbosity=2)
