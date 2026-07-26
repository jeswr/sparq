#!/usr/bin/env python3
"""Hermetic tests for the VERDICT-comment -> review-label bridge.

Two halves, because every uncaught mutant this project has measured lived at the YAML
seam rather than in the Python:

* ``TestDecisionCore`` / ``TestVerdictParsing`` — the pure predicate. Every guard in
  ``decide()`` has a NAMED test here that reds when the guard is deleted or inverted.
* ``TestWorkflowWiring`` — STRUCTURAL PyYAML inspection of
  ``.github/workflows/verdict-bridge.yml``. Substring and ``count(...) == N`` assertions
  do not catch ``if: false`` or a deleted step, so the workflow document is parsed and
  the job/step graph asserted against directly.
* ``TestCrossPolicyConsistency`` — imports the LIVE ``auto-arm.py`` /
  ``rearm-sweeper.py`` and pins the two invariants that make the bridge safe by
  construction: it never attests a PR the arming policy would refuse, and the
  informational label it writes is not a hold anywhere.

Stdlib unittest + PyYAML. No network, no gh, no git.
"""

# [OPUS-5] sparq-org/sparq — green-but-unqueued PR investigation, 2026-07-26.

from __future__ import annotations

import importlib.util
import json
import sys
import unittest
from pathlib import Path
from types import ModuleType

import yaml

REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = REPO_ROOT / "scripts"
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "verdict-bridge.yml"
AUTO_ARM_WORKFLOW = REPO_ROOT / ".github" / "workflows" / "auto-arm.yml"


def load(path: Path, name: str) -> ModuleType:
    """Import a hyphenated script as a module."""
    sys.path.insert(0, str(SCRIPTS))
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader, path
    module = importlib.util.module_from_spec(spec)
    # Register BEFORE exec: dataclasses resolves annotations via sys.modules[__module__].
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


vb = load(SCRIPTS / "verdict-bridge.py", "verdict_bridge")
auto_arm = load(SCRIPTS / "auto-arm.py", "auto_arm_policy")
rearm = load(SCRIPTS / "rearm-sweeper.py", "rearm_sweeper_policy")

HEAD = "a" * 40
OTHER = "b" * 40

# The live sweep step's `run:` is a FOLDED (`>-`) scalar, so YAML joins its argv onto one
# line. Anchoring on `--repo` is what distinguishes it from the `--self-test` step.
LIVE_RUN_NEEDLE = "python3 scripts/verdict-bridge.py --repo"


def comment(body: str, **kw) -> dict:
    return vb.comment(body, **kw)


def pr(**kw):
    return vb.pr_fixture(**kw)


class TestVerdictParsing(unittest.TestCase):
    """The line-anchored + head-bound + provenance predicate."""

    def test_trailing_verdict_reads_the_last_non_blank_line(self):
        self.assertEqual(vb.trailing_verdict("prose\n\nVERDICT: pass\n\n"), "pass")
        self.assertEqual(vb.trailing_verdict("**VERDICT: fail**"), "fail")

    def test_verdict_must_be_the_final_line(self):
        """A quoted INSTRUCTION mentioning the phrase is not a verdict."""
        self.assertIsNone(
            vb.trailing_verdict("end with `VERDICT: pass`\n\nstill reviewing")
        )
        self.assertIsNone(vb.trailing_verdict("VERDICT: pass\nbut actually no"))

    def test_verdict_line_must_not_carry_trailing_content(self):
        """It is never read as a PASS — it degrades to AMBIGUOUS, not to a verdict."""
        for line in ("VERDICT: pass (with caveats)", "VERDICT: pass, mostly"):
            with self.subTest(line=line):
                self.assertEqual(vb.trailing_verdict(line), vb.AMBIGUOUS)

    def test_a_verdict_shaped_line_that_does_not_parse_is_ambiguous_not_absent(self):
        """Returning None here is the composition fail-open: an older, well-formed pass
        would survive the retraction and stay ``found[-1]``."""
        for line in (
            "VERDICT: FAIL",
            "VERDICT: fail (retracting my pass)",
            "**VERDICT: Fail**",
            "VERDICT - fail",
            "verdict: unclear",
            "VERDICT:",
        ):
            with self.subTest(line=line):
                self.assertEqual(vb.trailing_verdict(line), vb.AMBIGUOUS)

    def test_a_mention_of_the_phrase_is_still_not_verdict_shaped(self):
        """Quoted / blockquoted / fenced / bulleted mentions stay 'no verdict' — the
        instruction that tells reviewers what to write must influence nothing."""
        for line in (
            "> VERDICT: pass",
            "- VERDICT: pass",
            "* VERDICT: fail",
            "`VERDICT: pass`",
            "the brief says to end with VERDICT: pass",
            "```",
        ):
            with self.subTest(line=line):
                self.assertIsNone(vb.trailing_verdict(line))

    def test_an_unparseable_retraction_defeats_an_earlier_pass_at_the_same_head(self):
        """THE composition hole. A reviewer's only retraction channel is a comment, so a
        slightly-misformatted retraction must never leave the superseded pass standing."""
        stale_pass = comment(
            f"{HEAD}\n\nVERDICT: pass", created_at="2026-01-01T00:00:00Z", cid=1
        )
        for line in ("VERDICT: FAIL", "VERDICT: fail (retracting my pass)"):
            with self.subTest(line=line):
                retraction = comment(
                    f"{HEAD}\n\n{line}", created_at="2026-02-01T00:00:00Z", cid=2
                )
                self.assertEqual(
                    vb.head_bound_verdict([stale_pass, retraction], HEAD).value,
                    vb.AMBIGUOUS,
                )
                self.assertNotEqual(decision(pr(), [stale_pass, retraction]), "promote")
                self.assertEqual(
                    decision(
                        pr(labels={vb.REVIEW_ATTESTATION}), [stale_pass, retraction]
                    ),
                    "retract",
                    "an unreadable retraction must not leave review:pass standing",
                )

    def test_an_untrusted_ambiguous_line_cannot_suppress_a_trusted_pass(self):
        """Ambiguity is fail-closed, but only for REVIEWERS — otherwise any drive-by
        commenter could deny arming to every PR in the repo."""
        trusted_pass = comment(
            f"{HEAD}\n\nVERDICT: pass", created_at="2026-01-01T00:00:00Z", cid=1
        )
        for association in ("NONE", "CONTRIBUTOR", "BOT", ""):
            with self.subTest(association=association):
                drive_by = comment(
                    f"{HEAD}\n\nVERDICT: FAIL",
                    association=association,
                    created_at="2026-02-01T00:00:00Z",
                    cid=2,
                )
                self.assertEqual(decision(pr(), [trusted_pass, drive_by]), "promote")

    def test_an_ambiguous_line_bound_to_another_head_is_ignored(self):
        """Head binding still gates ambiguity — a retraction of a SUPERSEDED head must
        not suppress a fresh pass."""
        fresh = comment(
            f"{HEAD}\n\nVERDICT: pass", created_at="2026-01-01T00:00:00Z", cid=1
        )
        old = comment(
            f"{OTHER}\n\nVERDICT: FAIL", created_at="2026-02-01T00:00:00Z", cid=2
        )
        self.assertEqual(decision(pr(), [fresh, old]), "promote")

    def test_a_review_body_can_WITHHOLD_a_pass(self):
        """The composition hole is channel-independent: a reviewer who posts a pass as a
        comment and then withdraws it in a PR REVIEW body must not leave it standing."""
        old_pass = comment(
            f"{HEAD}\n\nVERDICT: pass", created_at="2026-01-01T00:00:00Z", cid=1
        )
        for line in ("VERDICT: fail", "VERDICT: FAIL"):
            with self.subTest(line=line):
                withheld = dict(
                    comment(f"{HEAD}\n\n{line}", created_at="2026-02-01T00:00:00Z", cid=2),
                    _channel="review",
                )
                self.assertNotEqual(decision(pr(), [old_pass, withheld]), "promote")
                self.assertEqual(
                    decision(pr(labels={vb.REVIEW_ATTESTATION}), [old_pass, withheld]),
                    "retract",
                )

    def test_a_review_body_can_NEVER_grant_a_pass(self):
        """Reading a review body as a pass would extend arming authority into a channel
        the standing review brief does not mandate. The promote-set may only shrink."""
        review_pass = dict(
            comment(f"{HEAD}\n\nVERDICT: pass", created_at="2026-03-01T00:00:00Z", cid=3),
            _channel="review",
        )
        self.assertIsNone(vb.head_bound_verdict([review_pass], HEAD))
        self.assertEqual(decision(pr(), [review_pass]), "flag")
        # ... not even as the NEWEST evidence over an older review fail.
        review_fail = dict(
            comment(f"{HEAD}\n\nVERDICT: fail", created_at="2026-02-01T00:00:00Z", cid=2),
            _channel="review",
        )
        self.assertEqual(
            vb.head_bound_verdict([review_fail, review_pass], HEAD).value, "fail"
        )

    def test_a_review_withholding_still_obeys_head_binding_and_provenance(self):
        good_pass = comment(
            f"{HEAD}\n\nVERDICT: pass", created_at="2026-01-01T00:00:00Z", cid=1
        )
        for kwargs, why in (
            ({"association": "NONE"}, "untrusted author"),
            ({}, "bound to another head"),
        ):
            body = f"{HEAD}\n\nVERDICT: fail" if kwargs else f"{OTHER}\n\nVERDICT: fail"
            with self.subTest(why=why):
                withheld = dict(
                    comment(body, created_at="2026-02-01T00:00:00Z", cid=2, **kwargs),
                    _channel="review",
                )
                self.assertEqual(decision(pr(), [good_pass, withheld]), "promote")

    def test_binds_head_requires_the_full_forty_hex_sha(self):
        self.assertTrue(vb.binds_head(f"reviewed {HEAD}", HEAD))
        self.assertFalse(vb.binds_head(f"reviewed {HEAD[:12]}", HEAD))
        self.assertFalse(vb.binds_head(f"reviewed {OTHER}", HEAD))

    def test_binds_head_rejects_a_sha_glued_into_a_longer_hex_run(self):
        self.assertFalse(vb.binds_head(f"reviewed {HEAD}cafe", HEAD))
        self.assertFalse(vb.binds_head(f"reviewed cafe{HEAD}", HEAD))

    def test_untrusted_association_is_not_a_reviewer(self):
        for association in ("CONTRIBUTOR", "FIRST_TIME_CONTRIBUTOR", "NONE", "", "BOT"):
            with self.subTest(association=association):
                self.assertIsNone(
                    vb.head_bound_verdict(
                        [comment(f"{HEAD}\n\nVERDICT: pass", association=association)],
                        HEAD,
                    )
                )

    def test_malformed_comment_payloads_never_read_as_a_pass(self):
        self.assertIsNone(
            vb.head_bound_verdict([None, 7, {}, {"body": None}, {"body": ""}], HEAD)
        )

    def test_latest_by_created_at_wins_regardless_of_input_order(self):
        early = comment(f"{HEAD}\n\nVERDICT: pass", created_at="2026-01-01T00:00:00Z", cid=1)
        late = comment(f"{HEAD}\n\nVERDICT: fail", created_at="2026-01-02T00:00:00Z", cid=2)
        for ordering in ([early, late], [late, early]):
            with self.subTest(ordering=[c["id"] for c in ordering]):
                self.assertEqual(vb.head_bound_verdict(ordering, HEAD).value, "fail")

    def test_recency_is_created_at_not_comment_id(self):
        """Comment ids are not monotone across a PR's timeline (reviews, transfers,
        cross-posted bodies), so ordering by id can silently resurrect a stale pass.
        Here the RETRACTION carries the LOWER id but the LATER created_at."""
        stale_pass = comment(
            f"{HEAD}\n\nVERDICT: pass", created_at="2026-01-01T00:00:00Z", cid=900
        )
        retraction = comment(
            f"{HEAD}\n\nVERDICT: fail", created_at="2026-01-02T00:00:00Z", cid=100
        )
        self.assertEqual(
            vb.head_bound_verdict([stale_pass, retraction], HEAD).value, "fail"
        )
        self.assertEqual(
            decision(pr(labels={vb.REVIEW_ATTESTATION}), [stale_pass, retraction]),
            "retract",
        )

    def test_editing_an_older_pass_cannot_reorder_it_ahead_of_a_newer_fail(self):
        """Ordering uses immutable created_at — an edit must not defeat a retraction."""
        early = dict(
            comment(f"{HEAD}\n\nVERDICT: pass", created_at="2026-01-01T00:00:00Z", cid=1),
            updated_at="2099-01-01T00:00:00Z",
        )
        late = comment(f"{HEAD}\n\nVERDICT: fail", created_at="2026-01-02T00:00:00Z", cid=2)
        self.assertEqual(vb.head_bound_verdict([early, late], HEAD).value, "fail")


class TestDecisionCore(unittest.TestCase):
    """One named test per guard in ``decide()``."""

    def setUp(self):
        self.passing = comment(f"reviewed {HEAD}\n\nVERDICT: pass")
        self.failing = comment(f"reviewed {HEAD}\n\nVERDICT: fail", cid=2)

    def test_head_bound_pass_on_a_clean_pr_promotes(self):
        self.assertEqual(decision(pr(), [self.passing]), "promote")

    def test_pass_bound_to_a_superseded_head_never_promotes(self):
        stale = comment(f"reviewed {OTHER}\n\nVERDICT: pass")
        self.assertNotEqual(decision(pr(), [stale]), "promote")

    def test_hold_labels_are_fail_closed(self):
        for hold in sorted(vb.HOLD_LABELS) + ["needs:design", "needs:whatever"]:
            with self.subTest(hold=hold):
                self.assertEqual(decision(pr(labels={hold}), [self.passing]), "none")

    def test_draft_is_never_attested(self):
        self.assertEqual(decision(pr(is_draft=True), [self.passing]), "none")

    def test_closed_pr_is_never_attested(self):
        for state in ("MERGED", "CLOSED", ""):
            with self.subTest(state=state):
                self.assertEqual(decision(pr(state=state), [self.passing]), "none")

    def test_promotion_is_idempotent(self):
        self.assertEqual(
            decision(pr(labels={vb.REVIEW_ATTESTATION}), [self.passing]), "none"
        )

    def test_head_bound_fail_retracts_an_existing_attestation(self):
        self.assertEqual(
            decision(pr(labels={vb.REVIEW_ATTESTATION}), [self.failing]), "retract"
        )

    def test_absence_of_evidence_never_retracts(self):
        """A hand-applied review:pass with no comment behind it must survive."""
        self.assertEqual(decision(pr(labels={vb.REVIEW_ATTESTATION}), []), "none")
        stale_fail = comment(f"reviewed {OTHER}\n\nVERDICT: fail")
        self.assertEqual(
            decision(pr(labels={vb.REVIEW_ATTESTATION}), [stale_fail]), "none"
        )

    def test_green_mergeable_unreviewed_pr_is_flagged_visible(self):
        self.assertEqual(decision(pr(), []), "flag")

    def test_flag_requires_green_and_mergeable_and_non_draft(self):
        for kwargs in (
            {"is_draft": True},
            {"mergeable": "CONFLICTING"},
            {"mergeable": "UNKNOWN"},
            {"gate_conclusion": "failure"},
            {"gate_conclusion": None},
        ):
            with self.subTest(**kwargs):
                self.assertEqual(decision(pr(**kwargs), []), "none")

    def test_flag_skips_a_pr_already_in_a_review_lane(self):
        for lane in ("review:needs", "review:changes", "review:needs-user", "review:pass"):
            with self.subTest(lane=lane):
                self.assertEqual(decision(pr(labels={lane}), []), "none")

    def test_flag_is_cleared_once_any_verdict_binds(self):
        flagged = pr(labels={vb.UNREVIEWED_LABEL})
        self.assertEqual(decision(flagged, [self.passing]), "unflag")
        self.assertEqual(decision(flagged, [self.failing]), "unflag")
        self.assertEqual(decision(flagged, []), "none")


def decision(pull, comments) -> str:
    return vb.decide(pull, comments).action


# ------------------------------------------------------------------- driver (write path)
#
# `decide()` only produces a WORD. The layer that turns that word into arming authority is
# `VerdictBridge` — the dispatch table, the dry-run short-circuit, the write cap, the
# base-branch filter and the paged reads. A guard with no red test on THIS layer is the
# expensive kind: mis-wiring `"flag"` to `review:pass` would attest every never-reviewed
# green PR in the repo, and every decide()-level and YAML-level test would still pass.

REPO = "sparq-org/sparq"


def node(number: int, **overrides) -> dict:
    """A GraphQL pullRequest node as `list_open` returns it."""
    labels = overrides.pop("labels", ())
    reviews = overrides.pop("reviews", ())
    base = {
        "reviews": {
            "nodes": [
                {
                    "databaseId": r.get("id", 1),
                    "body": r.get("body"),
                    "authorAssociation": r.get("association", "MEMBER"),
                    "submittedAt": r.get("submitted_at", "2026-02-01T00:00:00Z"),
                    "author": {"login": r.get("user", "reviewer")},
                }
                for r in reviews
            ]
        },
        "number": number,
        "state": "OPEN",
        "isDraft": False,
        "baseRefName": "main",
        "headRefOid": HEAD,
        "mergeable": "MERGEABLE",
        "labels": {
            "nodes": [{"name": name} for name in labels],
            "pageInfo": {"hasNextPage": overrides.pop("labels_overflow", False)},
        },
    }
    base.update(overrides)
    return base


def check_run(name: str = "gate", *, conclusion="success", status="completed", rid=1, started="2026-01-01T00:00:00Z") -> dict:
    return {
        "name": name,
        "status": status,
        "conclusion": conclusion,
        "id": rid,
        "started_at": started,
    }


class FakeGitHub:
    """Hermetic stand-in for the `gh` CLI: serves the driver's three reads, RECORDS every
    write. No network, no subprocess."""

    def __init__(self, nodes, *, comments=None, runs=None, pr_page=50, run_page=100):
        self.nodes = list(nodes)
        self._comments = dict(comments or {})
        self._runs = dict(runs or {})
        self.pr_page = pr_page
        self.run_page = run_page
        self.writes: list[list[str]] = []
        self.reads: list[str] = []
        self.total_count_override: int | None = None

    # -- writes --
    def write(self, argv: list[str]) -> str:
        self.writes.append(list(argv))
        return ""

    # -- reads --
    def read(self, argv: list[str]) -> str:
        self.reads.append(" ".join(argv))
        if argv[:2] == ["api", "graphql"]:
            return json.dumps(self._pr_page(argv))
        path = argv[1]
        if "/check-runs" in path:
            return json.dumps(self._check_runs(path))
        if "/comments" in path:
            return json.dumps(self._comment_page(path))
        raise AssertionError(f"unexpected read: {argv}")

    def _pr_page(self, argv: list[str]) -> dict:
        cursor = next(
            (a.split("=", 1)[1] for a in argv if a.startswith("cursor=")), "0"
        )
        start = int(cursor)
        page = self.nodes[start : start + self.pr_page]
        end = start + len(page)
        total = (
            self.total_count_override
            if self.total_count_override is not None
            else len(self.nodes)
        )
        return {
            "data": {
                "repository": {
                    "pullRequests": {
                        "pageInfo": {
                            "hasNextPage": end < len(self.nodes),
                            "endCursor": str(end),
                        },
                        "totalCount": total,
                        "nodes": page,
                    }
                }
            }
        }

    def _check_runs(self, path: str) -> dict:
        sha = path.split("/commits/", 1)[1].split("/", 1)[0]
        page = int(path.rsplit("&page=", 1)[1])
        runs = self._runs.get(sha, [check_run()])
        start = (page - 1) * self.run_page
        return {
            "total_count": len(runs),
            "check_runs": runs[start : start + self.run_page],
        }

    def _comment_page(self, path: str) -> list:
        number = int(path.split("/issues/", 1)[1].split("/", 1)[0])
        page = int(path.rsplit("&page=", 1)[1])
        body = self._comments.get(number, [])
        return body[(page - 1) * 100 : page * 100]


def bridge(fake: FakeGitHub, **kw) -> "vb.VerdictBridge":
    logs: list[str] = []
    b = vb.VerdictBridge(
        REPO,
        kw.pop("default_branch", "main"),
        gh=fake.write,
        gh_read=fake.read,
        log=logs.append,
        **kw,
    )
    b.logs = logs  # type: ignore[attr-defined]
    return b


PASS_COMMENT = [vb.comment(f"reviewed {HEAD}\n\nVERDICT: pass")]
FAIL_COMMENT = [vb.comment(f"reviewed {HEAD}\n\nVERDICT: fail")]


class TestWritePathDispatch(unittest.TestCase):
    """Every decision maps to the RIGHT label edit — the mutation that matters most is
    silent: a dispatch-table entry pointing `flag` at the attestation label."""

    def edit(self, fake: FakeGitHub) -> list[str]:
        self.assertEqual(len(fake.writes), 1, f"expected exactly one write: {fake.writes}")
        return fake.writes[0]

    def test_promote_adds_the_attestation_label_and_removes_nothing_else(self):
        fake = FakeGitHub([node(4200)], comments={4200: PASS_COMMENT})
        self.assertEqual(bridge(fake).run(), 0)
        argv = self.edit(fake)
        self.assertEqual(
            argv, ["pr", "edit", "4200", "--repo", REPO, "--add-label", vb.REVIEW_ATTESTATION]
        )
        self.assertNotIn("--remove-label", argv)

    def test_a_flagged_pr_is_unflagged_first_then_promoted_on_the_next_sweep(self):
        """The informational label is cleared before the attestation is written, so the
        two labels are never both live. Cycle 1 unflags; cycle 2 (labels updated by the
        first write) promotes. Neither cycle may write the attestation AND leave the
        flag, which would leave the PR in two lanes at once."""
        first = FakeGitHub(
            [node(4200, labels=[vb.UNREVIEWED_LABEL])], comments={4200: PASS_COMMENT}
        )
        bridge(first).run()
        self.assertEqual(
            self.edit(first),
            ["pr", "edit", "4200", "--repo", REPO, "--remove-label", vb.UNREVIEWED_LABEL],
        )
        second = FakeGitHub([node(4200)], comments={4200: PASS_COMMENT})
        bridge(second).run()
        self.assertEqual(
            self.edit(second),
            ["pr", "edit", "4200", "--repo", REPO, "--add-label", vb.REVIEW_ATTESTATION],
        )

    def test_retract_removes_the_attestation_and_adds_nothing(self):
        fake = FakeGitHub(
            [node(4200, labels=[vb.REVIEW_ATTESTATION])], comments={4200: FAIL_COMMENT}
        )
        bridge(fake).run()
        argv = self.edit(fake)
        self.assertEqual(
            argv,
            ["pr", "edit", "4200", "--repo", REPO, "--remove-label", vb.REVIEW_ATTESTATION],
        )
        self.assertNotIn("--add-label", argv)

    def test_an_unreadable_retraction_also_removes_the_attestation(self):
        """End-to-end proof of the composition fix, through the real write path."""
        comments = [
            vb.comment(f"{HEAD}\n\nVERDICT: pass", created_at="2026-01-01T00:00:00Z", cid=1),
            vb.comment(
                f"{HEAD}\n\nVERDICT: fail (retracting my pass)",
                created_at="2026-02-01T00:00:00Z",
                cid=2,
            ),
        ]
        fake = FakeGitHub([node(4200, labels=[vb.REVIEW_ATTESTATION])], comments={4200: comments})
        bridge(fake).run()
        argv = self.edit(fake)
        self.assertEqual(argv[argv.index("--remove-label") + 1], vb.REVIEW_ATTESTATION)
        self.assertNotIn("--add-label", argv)

    def test_flag_writes_ONLY_the_informational_label(self):
        """If this ever wrote review:pass, every green never-reviewed PR in the repo
        would be attested and armed. The decide()-level suite cannot see that."""
        fake = FakeGitHub([node(4200)], comments={4200: []})
        bridge(fake).run()
        argv = self.edit(fake)
        self.assertEqual(
            argv, ["pr", "edit", "4200", "--repo", REPO, "--add-label", vb.UNREVIEWED_LABEL]
        )
        self.assertNotIn(vb.REVIEW_ATTESTATION, argv)

    def test_unflag_removes_only_the_informational_label(self):
        fake = FakeGitHub(
            [node(4200, labels=[vb.UNREVIEWED_LABEL])], comments={4200: FAIL_COMMENT}
        )
        bridge(fake).run()
        argv = self.edit(fake)
        self.assertEqual(
            argv,
            ["pr", "edit", "4200", "--repo", REPO, "--remove-label", vb.UNREVIEWED_LABEL],
        )
        self.assertNotIn(vb.REVIEW_ATTESTATION, argv)

    def test_a_review_body_retraction_reaches_the_write_path(self):
        """Reviews ride the SAME GraphQL page as the PR, so this costs no extra API call
        — and a withdrawal posted as a review must remove the attestation."""
        fake = FakeGitHub(
            [
                node(
                    4200,
                    labels=[vb.REVIEW_ATTESTATION],
                    reviews=[
                        {
                            "body": f"{HEAD}\n\nVERDICT: FAIL",
                            "id": 7,
                            "submitted_at": "2099-01-01T00:00:00Z",
                        }
                    ],
                )
            ],
            comments={4200: PASS_COMMENT},
        )
        bridge(fake).run()
        argv = self.edit(fake)
        self.assertEqual(argv[argv.index("--remove-label") + 1], vb.REVIEW_ATTESTATION)
        self.assertNotIn("--add-label", argv)

    def test_a_review_body_pass_never_reaches_the_write_path_as_an_attestation(self):
        fake = FakeGitHub(
            [node(4200, reviews=[{"body": f"{HEAD}\n\nVERDICT: pass", "id": 7}])],
            comments={4200: []},
        )
        bridge(fake).run()
        argv = self.edit(fake)
        self.assertNotIn(vb.REVIEW_ATTESTATION, argv)
        self.assertIn(vb.UNREVIEWED_LABEL, argv)

    def test_the_pr_list_query_actually_requests_the_review_bodies(self):
        """Reviews ride the PR-list query. If the connection (or any field the
        normaliser reads) is dropped, every review-body retraction silently vanishes and
        the fake in these tests would happily keep serving them."""
        query = vb.PR_LIST_QUERY
        self.assertIn("reviews(", query, "the reviews connection is not requested")
        for gql_field in ("databaseId", "body", "authorAssociation", "submittedAt", "author"):
            with self.subTest(field=gql_field):
                self.assertIn(gql_field, query.split("reviews(", 1)[1].split("}}", 1)[0])

    def test_a_PENDING_review_is_not_evidence(self):
        """An unsubmitted review body is a draft; it must not suppress anything."""
        pending = node(
            4200,
            reviews=[{"body": f"{HEAD}\n\nVERDICT: FAIL", "id": 7, "submitted_at": None}],
        )
        self.assertEqual(
            vb.VerdictBridge.review_withholdings(pending),
            [],
            "an unsubmitted review body must not be normalised into evidence at all",
        )
        fake = FakeGitHub(
            [
                node(
                    4200,
                    reviews=[
                        {"body": f"{HEAD}\n\nVERDICT: FAIL", "id": 7, "submitted_at": None}
                    ],
                )
            ],
            comments={4200: PASS_COMMENT},
        )
        bridge(fake).run()
        self.assertEqual(
            self.edit(fake),
            ["pr", "edit", "4200", "--repo", REPO, "--add-label", vb.REVIEW_ATTESTATION],
        )

    def test_a_none_decision_writes_nothing(self):
        fake = FakeGitHub(
            [node(4200, labels=["needs:user"])], comments={4200: PASS_COMMENT}
        )
        self.assertEqual(bridge(fake).run(), 0)
        self.assertEqual(fake.writes, [])


class TestWritePathSafetyRails(unittest.TestCase):
    """dry-run, the write cap, the stacked-PR filter and the fail-closed label read."""

    def test_dry_run_performs_no_write_at_all(self):
        """--dry-run is this PR's central evidence; if it wrote, the evidence would be
        a live mutation of the repo."""
        fake = FakeGitHub([node(4200)], comments={4200: PASS_COMMENT})
        b = bridge(fake, dry_run=True)
        self.assertEqual(b.run(), 0)
        self.assertEqual(fake.writes, [], "a dry run must not touch a single label")
        self.assertTrue(any("PROMOTE" in line for line in b.logs), b.logs)

    def test_the_per_run_write_cap_is_enforced(self):
        fake = FakeGitHub(
            [node(n) for n in (1, 2, 3)],
            comments={n: PASS_COMMENT for n in (1, 2, 3)},
        )
        b = bridge(fake, max_writes=2)
        self.assertEqual(b.run(), 0)
        self.assertEqual(len(fake.writes), 2, fake.writes)
        self.assertTrue(any("write cap" in line for line in b.logs), b.logs)

    def test_a_stacked_pr_is_never_touched(self):
        """base != the default branch: auto-arm refuses these outright (the stacked-PR
        auto-merge trap), so attesting one would paint a misleading label."""
        fake = FakeGitHub(
            [node(4200, baseRefName="feature/stack-base")], comments={4200: PASS_COMMENT}
        )
        self.assertEqual(bridge(fake).run(), 0)
        self.assertEqual(fake.writes, [])

    def test_a_label_set_that_exceeds_one_page_fails_CLOSED(self):
        """An unseen label could be a hold, so the PR must be SKIPPED, not promoted."""
        fake = FakeGitHub(
            [node(4200, labels_overflow=True)], comments={4200: PASS_COMMENT}
        )
        b = bridge(fake)
        self.assertEqual(b.run(), 1, "an unreadable label set must be reported as an error")
        self.assertEqual(fake.writes, [], "a partially-read label set must never promote")
        self.assertTrue(any("SKIP inspect-failed" in line for line in b.logs), b.logs)

    def test_a_failed_label_edit_is_counted_as_an_error(self):
        def explode(argv):
            raise vb.GhError("403 label edit denied")

        fake = FakeGitHub([node(4200)], comments={4200: PASS_COMMENT})
        b = vb.VerdictBridge(REPO, "main", gh=explode, gh_read=fake.read, log=lambda _: None)
        self.assertEqual(b.run(), 1)

    def test_run_bridge_reds_the_workflow_on_a_hard_error(self):
        """The exit code is the only signal the cron surfaces."""

        class Boom:
            def run(self):
                return 3

        self.assertEqual(vb.run_bridge(Boom(), log=lambda _: None), 1)

    def test_run_bridge_survives_exhausted_TRANSIENT_reads_with_a_warning(self):
        class Transient:
            def run(self):
                raise vb.gh_retry.GhTransientExhausted("502 x5")

        logs: list[str] = []
        self.assertEqual(vb.run_bridge(Transient(), log=logs.append), 0)
        self.assertTrue(any("::warning" in line for line in logs), logs)


class TestPagedReads(unittest.TestCase):
    """PRs here carry up to ~1181 check-runs; a page-1 read silently truncates."""

    def test_check_runs_are_paged_and_cross_checked_against_total_count(self):
        runs = [check_run(f"filler-{i}", rid=i) for i in range(150)]
        runs.append(check_run("gate", rid=999, started="2026-01-01T05:00:00Z"))
        fake = FakeGitHub([node(4200)], runs={HEAD: runs}, run_page=100)
        b = bridge(fake)
        self.assertEqual(len(b.check_runs(HEAD)), 151)
        self.assertEqual(
            b.gate_conclusion(HEAD), "success", "the gate run lives beyond page 1"
        )

    def test_a_TRUNCATED_check_run_read_raises(self):
        """Fewer runs than total_count means the `gate` may be in the part never read."""
        fake = FakeGitHub([node(4200)], runs={HEAD: [check_run()]})
        original = fake._check_runs
        fake._check_runs = lambda path: dict(original(path), total_count=99)  # type: ignore[assignment]
        with self.assertRaises(vb.GhError):
            bridge(fake).check_runs(HEAD)

    def test_check_runs_CREATED_mid_pagination_do_not_red_the_sweep(self):
        """MEASURED on sparq#4074: `paged 479 check-runs, total_count=473` — runs were
        being created between page reads. Reading MORE than total_count is benign; a
        two-sided equality check skipped the PR and red the workflow every cycle. Only
        TRUNCATION (fewer) is dangerous. Growth also repeats entries across page
        boundaries, so the read de-duplicates by run id."""
        runs = [check_run(f"job-{i}", rid=i) for i in range(150)]
        runs.append(check_run("gate", rid=999, started="2026-01-01T05:00:00Z"))
        fake = FakeGitHub([node(4200)], runs={HEAD: runs}, run_page=100)
        original = fake._check_runs
        # total_count as first read, BEFORE the last 51 runs existed.
        fake._check_runs = lambda path: dict(original(path), total_count=100)  # type: ignore[assignment]
        b = bridge(fake)
        got = b.check_runs(HEAD)
        self.assertEqual(len(got), 151, "growth must not truncate the read")
        self.assertEqual(
            len({r["id"] for r in got}), len(got), "pages that overlap must de-duplicate"
        )
        self.assertEqual(b.gate_conclusion(HEAD), "success")

    def test_a_duplicated_check_run_across_pages_is_counted_once(self):
        """A window that shifts under pagination repeats entries; the server counts them
        once, so double-counting them would mask a real truncation."""
        dupe = check_run("gate", rid=7)
        fake = FakeGitHub([node(4200)], runs={HEAD: [dupe, dict(dupe)]})
        original = fake._check_runs
        fake._check_runs = lambda path: dict(original(path), total_count=1)  # type: ignore[assignment]
        self.assertEqual(len(bridge(fake).check_runs(HEAD)), 1)

    def test_the_check_run_read_is_page_bounded(self):
        """A pathological or looping response must FAIL the read, never spin forever.

        The fixture serves MAX+5 full pages then stops, so the assertion reds cleanly
        when the bound is removed instead of hanging the suite — a test that hangs on a
        mutant is a test that times CI out rather than reporting a defect."""
        pages = vb.MAX_CHECK_RUN_PAGES + 5
        runs = [check_run(f"j-{i}", rid=i) for i in range(pages * 100)]
        fake = FakeGitHub([node(4200)], runs={HEAD: runs}, run_page=100)
        with self.assertRaises(vb.GhError):
            bridge(fake).check_runs(HEAD)

    def test_comments_are_paged_so_a_verdict_beyond_page_one_is_seen(self):
        chatter = [
            vb.comment(f"noise {i}", cid=i, created_at="2026-01-01T00:00:00Z")
            for i in range(100)
        ]
        verdict = vb.comment(
            f"reviewed {HEAD}\n\nVERDICT: pass", cid=5000, created_at="2026-02-01T00:00:00Z"
        )
        fake = FakeGitHub([node(4200)], comments={4200: chatter + [verdict]})
        b = bridge(fake)
        self.assertEqual(len(b.comments(4200)), 101)
        b.run()
        self.assertEqual(len(fake.writes), 1, "the verdict on page 2 must be honoured")
        self.assertIn(vb.REVIEW_ATTESTATION, fake.writes[0])

    def test_open_prs_are_paged_and_cross_checked_against_total_count(self):
        fake = FakeGitHub([node(n) for n in range(1, 121)], pr_page=50)
        self.assertEqual(len(bridge(fake).list_open()), 120)
        fake.total_count_override = 200
        with self.assertRaises(vb.GhError):
            bridge(fake).list_open()


class TestGateResolution(unittest.TestCase):
    """`gate` is resolved NEWEST-run-per-name; a cancelled twin must not read as live."""

    def test_the_newest_gate_run_wins_over_an_older_success(self):
        old = check_run("gate", conclusion="success", rid=1, started="2026-01-01T00:00:00Z")
        new = check_run("gate", conclusion="failure", rid=2, started="2026-01-01T09:00:00Z")
        for ordering in ([old, new], [new, old]):
            with self.subTest(order=[r["id"] for r in ordering]):
                fake = FakeGitHub([node(4200)], runs={HEAD: ordering})
                self.assertEqual(bridge(fake).gate_conclusion(HEAD), "failure")

    def test_an_incomplete_newest_run_reads_as_NO_gate_not_as_the_older_result(self):
        """Only `status: completed` may be believed. A queued or in-progress re-run can
        still carry the PREVIOUS attempt's `conclusion` field; reading it would report a
        superseded green as the live result — the cancelled-twin failure this repo has
        already been bitten by (sparq #3677)."""
        old = check_run("gate", conclusion="success", rid=1, started="2026-01-01T00:00:00Z")
        for status, leftover in (
            ("in_progress", None),
            ("in_progress", "success"),
            ("queued", "success"),
            ("waiting", "failure"),
        ):
            with self.subTest(status=status, conclusion=leftover):
                rerun = check_run(
                    "gate",
                    conclusion=leftover,
                    status=status,
                    rid=2,
                    started="2026-01-01T09:00:00Z",
                )
                fake = FakeGitHub([node(4200)], runs={HEAD: [old, rerun]})
                self.assertIsNone(bridge(fake).gate_conclusion(HEAD))

    def test_a_red_gate_still_never_flags_but_a_verdict_is_still_honoured(self):
        red = [check_run("gate", conclusion="failure")]
        fake = FakeGitHub([node(4200)], runs={HEAD: red}, comments={4200: []})
        self.assertEqual(bridge(fake).run(), 0)
        self.assertEqual(fake.writes, [], "a red PR is not the invisible population")


class TestCrossPolicyConsistency(unittest.TestCase):
    """The bridge must be safe against the LIVE arming policies, not a copy of them."""

    def test_bridge_holds_cover_every_auto_arm_exclusion(self):
        """Never attest a PR auto-arm would refuse — the label would be a lie."""
        required = auto_arm.HUMAN_OR_TRUST_LABELS | auto_arm.REVIEW_CHANGES_LABELS
        missing = required - vb.HOLD_LABELS
        self.assertEqual(missing, set(), f"auto-arm excludes these, the bridge does not: {missing}")

    def test_bridge_holds_cover_every_rearm_sweeper_exclusion(self):
        missing = rearm.EXCLUDED_LABELS - vb.HOLD_LABELS
        self.assertEqual(missing, set(), f"rearm-sweeper excludes these: {missing}")

    def test_the_informational_label_is_not_a_hold_in_any_policy(self):
        """review:unreviewed must never block an arm, or flagging would deadlock it."""
        label = vb.UNREVIEWED_LABEL
        self.assertNotIn(label, auto_arm.HUMAN_OR_TRUST_LABELS)
        self.assertNotIn(label, auto_arm.REVIEW_CHANGES_LABELS)
        self.assertNotIn(label, rearm.EXCLUDED_LABELS)
        self.assertEqual(rearm.exclusion_labels(frozenset({label})), [])
        self.assertEqual(vb.hold_labels({label}), [])

    def test_the_attestation_label_matches_the_arming_predicate(self):
        self.assertEqual(vb.REVIEW_ATTESTATION, auto_arm.REVIEW_LABEL)
        self.assertEqual(vb.REVIEW_ATTESTATION, rearm.REVIEW_ATTESTATION)


# --------------------------------------------------------------------------- YAML seam


def load_workflow(path: Path) -> dict:
    doc = yaml.safe_load(path.read_text(encoding="utf-8"))
    assert isinstance(doc, dict), path
    return doc


def triggers(doc: dict) -> dict:
    # PyYAML resolves the bare key `on` to the boolean True (YAML 1.1).
    return doc.get("on", doc.get(True)) or {}


def cron_minutes(doc: dict) -> set[int]:
    out: set[int] = set()
    for entry in triggers(doc).get("schedule") or []:
        minute_field = str(entry["cron"]).split()[0]
        out.update(int(m) for m in minute_field.split(","))
    return out


class TestWorkflowWiring(unittest.TestCase):
    """STRUCTURAL inspection: `if: false`, a deleted step, or a wrong input must red."""

    @classmethod
    def setUpClass(cls):
        cls.doc = load_workflow(WORKFLOW)
        cls.job = cls.doc["jobs"]["bridge"]
        cls.steps = cls.job["steps"]

    def step_by_run_needle(self, needle: str) -> dict:
        matches = [
            s for s in self.steps if needle in str(s.get("run") or "")
        ]
        self.assertEqual(
            len(matches), 1, f"expected exactly one step whose run contains {needle!r}"
        )
        return matches[0]

    def test_the_job_is_not_disabled_by_a_job_level_condition(self):
        """An `if: false` on the job silently turns the whole bridge off."""
        self.assertNotIn("if", self.job, "the bridge job must be unconditional")
        self.assertNotIn("continue-on-error", self.job)

    def test_no_step_is_disabled_or_swallowed(self):
        """Every step except the fail-soft App-token mint runs unconditionally."""
        for step in self.steps:
            name = step.get("name") or step.get("uses") or "<step>"
            with self.subTest(step=name):
                self.assertNotEqual(
                    str(step.get("if", "")).strip().lower(),
                    "false",
                    "a literal `if: false` disables this step",
                )
                self.assertNotIn("continue-on-error", step, name)
        conditional = [s for s in self.steps if "if" in s]
        self.assertEqual(
            [s.get("id") for s in conditional],
            ["app-token"],
            "only the App-token mint may be conditional",
        )

    def test_the_self_test_step_exists_and_runs_the_policy_self_test(self):
        step = self.step_by_run_needle("scripts/verdict-bridge.py --self-test")
        self.assertNotIn("if", step)
        self.assertIn("scripts/gh_retry.py --self-test", step["run"])

    def test_the_live_run_step_invokes_the_bridge_with_a_repo(self):
        step = self.step_by_run_needle(LIVE_RUN_NEEDLE)
        self.assertIn("--repo", step["run"])
        self.assertIn("--default-branch", step["run"])
        self.assertEqual(step["env"]["REPO"], "${{ github.repository }}")

    def test_the_scheduled_sweep_is_never_a_hard_coded_dry_run(self):
        """`DRY_RUN: --dry-run` (or a literal `--dry-run` in the `run:`) turns the whole
        workflow into a permanent no-op while EVERY other structural test still passes.
        The flag may only come from the workflow_dispatch input."""
        step = self.step_by_run_needle(LIVE_RUN_NEEDLE)
        self.assertNotIn(
            "--dry-run", step["run"], "the sweep's argv must not hard-code --dry-run"
        )
        self.assertIn("$DRY_RUN", step["run"], "the flag must come from the env")
        dry_run = str(step["env"]["DRY_RUN"])
        self.assertIn(
            "github.event.inputs.dry_run",
            dry_run,
            "DRY_RUN must be gated on the manual dispatch input, nothing else",
        )
        self.assertTrue(
            dry_run.startswith("${{") and dry_run.endswith("}}"),
            f"DRY_RUN must be an expression, not the literal {dry_run!r}",
        )
        inputs = (triggers(self.doc).get("workflow_dispatch") or {}).get("inputs") or {}
        self.assertIn("dry_run", inputs, "the expression references an undeclared input")
        self.assertFalse(
            inputs["dry_run"].get("default"), "a scheduled sweep must default to writing"
        )

    def test_the_informational_label_is_reified_before_use(self):
        """Without `gh label create`, every flag decision fails its label edit."""
        step = self.step_by_run_needle("gh label create")
        self.assertIn(vb.UNREVIEWED_LABEL, step["run"])
        self.assertIn("--force", step["run"], "creation must be idempotent")
        order = [self.steps.index(step), self.steps.index(
            self.step_by_run_needle(LIVE_RUN_NEEDLE)
        )]
        self.assertLess(order[0], order[1], "reify the label before the sweep uses it")

    def test_the_checkout_materializes_every_script_the_run_imports(self):
        checkouts = [s for s in self.steps if "actions/checkout" in str(s.get("uses"))]
        self.assertEqual(len(checkouts), 1)
        sparse = checkouts[0]["with"]["sparse-checkout"]
        for needed in ("scripts/verdict-bridge.py", "scripts/gh_retry.py"):
            self.assertIn(needed, sparse, f"{needed} would be missing at runtime")
        self.assertEqual(
            checkouts[0]["with"]["ref"],
            "${{ github.event.repository.default_branch }}",
            "the policy must come from the trusted default branch, never a PR head",
        )

    def test_it_never_triggers_on_a_pull_request_head(self):
        on = triggers(self.doc)
        for forbidden in ("pull_request", "pull_request_target", "push", "merge_group", "workflow_run"):
            self.assertNotIn(forbidden, on, f"{forbidden} would run this on candidate code")
        self.assertIn("schedule", on)
        self.assertIn("workflow_dispatch", on)

    def test_it_has_no_contents_write_authority(self):
        perms = self.doc["permissions"]
        self.assertEqual(perms.get("contents"), "read")
        self.assertEqual(perms.get("pull-requests"), "write")
        self.assertEqual(perms.get("issues"), "write")

    def test_the_cron_fires_before_each_auto_arm_sweep(self):
        """A wrong cron input makes every promotion wait a full extra arming cycle."""
        bridge_minutes = sorted(cron_minutes(self.doc))
        arm_minutes = sorted(cron_minutes(load_workflow(AUTO_ARM_WORKFLOW)))
        self.assertTrue(bridge_minutes, "the bridge must be scheduled at all")
        self.assertTrue(arm_minutes, "auto-arm's cron is the backstop this pairs with")
        self.assertEqual(
            set(bridge_minutes) & set(arm_minutes), set(), "must not collide with auto-arm"
        )
        for minute in bridge_minutes:
            following = [a for a in arm_minutes if a > minute]
            self.assertTrue(following, f"no auto-arm sweep follows minute {minute}")
            self.assertLessEqual(
                following[0] - minute, 10, f"minute {minute} waits too long to be armed"
            )

    def test_every_action_is_sha_pinned(self):
        for step in self.steps:
            uses = step.get("uses")
            if not uses:
                continue
            with self.subTest(uses=uses):
                ref = uses.split("@", 1)[1].split(" ")[0]
                self.assertEqual(len(ref), 40, f"{uses} is not SHA-pinned")
                int(ref, 16)

    def test_concurrency_serialises_the_sweep(self):
        concurrency = self.doc["concurrency"]
        self.assertEqual(concurrency["group"], "verdict-bridge")
        self.assertFalse(concurrency["cancel-in-progress"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
