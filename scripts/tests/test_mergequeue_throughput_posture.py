#!/usr/bin/env python3
# [OPUS-5] issue #5162 (steer channel for sq-6vshe.16 item (a)) — COUPLING PIN for the
# deferred `merge_queue.max_entries_to_build` 3 -> 5 raise
# (research/ci-mergequeue-speedup-2026-07.md §3.3, recorded in
# docs/branch-protection.md §Merge-queue throughput settings).
#
# WHY a test at all. The raise is approved in principle but HARD-GATED on a repo change
# that has not happened: `sq-6vshe.14`, the `queue-validated` push-to-main skip (§3.1).
# Until that lands, the runner-pool headroom 5 concurrent entries needs is still being
# burned by redundant push waves, so raising the parallelism first makes contention
# WORSE. That leaves a two-sided hazard that nothing in the repo currently detects:
#
#   * FORGOTTEN — `sq-6vshe.14` lands and the doc keeps asserting the precondition is
#     unmet, so the now-unblocked ruleset edit is never requested and the drain capacity
#     is silently left on the table. The claim in the doc ("no such job exists anywhere
#     in `.github/workflows/`") is a checkable statement ABOUT THIS REPO, and today
#     nothing rechecks it.
#   * PREMATURE — the documented value is edited to the raised number while the
#     precondition is still unmet, so the doc describes a ruleset the maintainer was
#     never licensed to ask for, in the exact configuration measured to be harmful.
#
# Neither shows up as a failure anywhere. Both are pinned here by coupling a single
# machine-readable marker in the doc to the LIVE state of `.github/workflows/`.
#
# SCOPE — what this test does NOT do. It does not read the live ruleset (`17688455`)
# and makes no claim that the documented values match it; the doc's own provenance note
# already records that the three profile-sourced values are unverified against the API
# at this commit. This suite checks the doc against the REPO only. Agents cannot edit
# rulesets at all, which is why the raise is carried as a maintainer steer issue
# (#5162) rather than applied from a PR.
#
# Hermetic: stdlib only (no PyYAML, no network, no gh) so it runs anywhere.
# Run:  python3 scripts/tests/test_mergequeue_throughput_posture.py

from __future__ import annotations

import re
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
DOC = REPO_ROOT / "docs" / "branch-protection.md"

# The bead whose landing clears the precondition, and the job id it introduces.
PRECONDITION_BEAD = "sq-6vshe.14"
PUSH_SKIP_JOB_IDS = frozenset({"queue-validated"})
STEER_ISSUE = "5162"

# The pre-approval baseline and the approved target, per §3.3 / the steer issue.
BASELINE = 3
APPROVED_TARGET = 5

# The doc's machine-readable pin. `status` is the doc's own claim about where the
# deferred edit stands; the tests below cross-check it against the repo.
#   blocked   — precondition unmet; the edit is not requestable and the value stays 3
#   unblocked — precondition met; the edit is now requestable but not yet applied
#   applied   — the maintainer has applied the ruleset edit and the value is raised
_MARKER = re.compile(
    r"<!--\s*mergequeue-throughput-pin:\s*"
    r"max_entries_to_build=(?P<value>\d+)\s+"
    r"precondition=(?P<precondition>\S+)\s+"
    r"status=(?P<status>blocked|unblocked|applied)\s+"
    r"steer=#?(?P<steer>\d+)\s*-->"
)
VALID_STATUSES = ("blocked", "unblocked", "applied")

# The rendered table row the marker must agree with, e.g.
#   | `max_entries_to_build` | `3` | entries built speculatively ... |
_TABLE_ROW = re.compile(r"^\|\s*`max_entries_to_build`\s*\|\s*`(\d+)`\s*\|", re.M)

_JOB_ID = re.compile(r"^  (?P<id>[A-Za-z0-9_-]+):\s*$")

# GitHub Actions honours BOTH spellings. Scanning only one would let a push-skip job
# added in the other extension slip past the coupling check below, leaving the doc
# asserting a precondition that has in fact cleared.
WORKFLOW_GLOBS = ("*.yml", "*.yaml")


def _is_comment(line: str) -> bool:
    return line.lstrip().startswith("#")


def workflows_defining_push_skip_job(workflows: Path | None = None) -> list[str]:
    """Workflow filenames that define a job whose id is a push-skip job id.

    Scans every `.yml` and `.yaml` file in `workflows` (default:
    `.github/workflows/`). Only lines inside the top-level `jobs:` mapping are
    considered, and comment lines are skipped — several files already discuss
    `sq-6vshe.14` and the `queue-validated` job in PROSE (ci.yml's cache-coupling
    note is one), and counting those would report the precondition as met while no
    such job runs.
    """
    root = WORKFLOWS if workflows is None else workflows
    hits = []
    for path in sorted(p for glob in WORKFLOW_GLOBS for p in root.glob(glob)):
        lines = path.read_text(encoding="utf-8").split("\n")
        try:
            start = next(i for i, l in enumerate(lines) if re.match(r"^jobs:", l))
        except StopIteration:
            continue
        for line in lines[start + 1 :]:
            if _is_comment(line) or not line.strip():
                continue
            if line[0] != " " and not _is_comment(line):
                break  # left the jobs: mapping
            m = _JOB_ID.match(line)
            if m and m.group("id").replace("_", "-") in PUSH_SKIP_JOB_IDS:
                hits.append(path.name)
                break
    return hits


def marker() -> re.Match[str]:
    m = _MARKER.search(DOC.read_text(encoding="utf-8"))
    assert m is not None, "caller must check presence first"
    return m


def expected_documented_value(status: str) -> int:
    """The ONLY `max_entries_to_build` value the doc may carry for `status`.

    `applied` means the maintainer applied the edit issue #5162 authorises, and that
    issue authorises exactly one edit: the baseline raised to the approved target,
    nothing else. So `applied` pins the exact target rather than merely "above the
    baseline" — any other number documents a ruleset nobody licensed.
    """
    if status not in VALID_STATUSES:
        raise ValueError(f"unknown marker status {status!r}")
    return APPROVED_TARGET if status == "applied" else BASELINE


class TestParserNonVacuity(unittest.TestCase):
    """Every assertion below reads one of two parsers. If either silently returns
    nothing, the coupling tests pass while checking NOTHING. Pin both."""

    def test_the_doc_carries_the_machine_readable_pin(self) -> None:
        text = DOC.read_text(encoding="utf-8")
        self.assertRegex(
            text,
            _MARKER,
            f"{DOC.relative_to(REPO_ROOT)} must carry the `mergequeue-throughput-pin` "
            "HTML comment in §Merge-queue throughput settings. It is the only "
            f"machine-readable record of where the deferred `max_entries_to_build` "
            f"{BASELINE} -> {APPROVED_TARGET} raise stands (steer issue #{STEER_ISSUE}); "
            "without it the deferral is prose that nothing rechecks.",
        )

    def test_the_rendered_table_row_is_found(self) -> None:
        self.assertEqual(
            len(_TABLE_ROW.findall(DOC.read_text(encoding="utf-8"))),
            1,
            "expected exactly one rendered `max_entries_to_build` table row in "
            f"{DOC.relative_to(REPO_ROOT)}; the marker/table drift guard reads it.",
        )

    def test_the_job_scanner_ignores_prose_mentions(self) -> None:
        # ci.yml discusses sq-6vshe.14 and the queue-validated push skip in a COMMENT
        # (the cache-coupling note). If the scanner counted comments it would report the
        # precondition as met today, and the coupling test would demand a doc
        # reconciliation for a job that does not exist.
        ci = WORKFLOWS / "ci.yml"
        self.assertTrue(ci.exists(), "fixture-by-reference vanished; re-point this test")
        self.assertIn(
            PRECONDITION_BEAD,
            ci.read_text(encoding="utf-8"),
            f"ci.yml no longer mentions {PRECONDITION_BEAD} in prose; re-point this test "
            "at another file that does, or it stops proving comments are ignored.",
        )
        self.assertNotIn(
            "ci.yml",
            workflows_defining_push_skip_job(),
            "the job scanner is matching comment lines — ci.yml only MENTIONS the "
            "queue-validated push skip, it does not define that job.",
        )


class TestJobScannerAgainstFixtures(unittest.TestCase):
    """Against the repo the scanner is all-negative today (no push-skip job exists
    yet), so its POSITIVE path — the half that fires the FORGOTTEN alarm — is never
    exercised. Drive it against controlled fixtures instead, in BOTH extensions
    GitHub Actions accepts."""

    _DEFINES_JOB = (
        "name: fixture\n"
        "on: [push]\n"
        "jobs:\n"
        "  queue-validated:\n"
        "    runs-on: ubuntu-latest\n"
        "    steps:\n"
        "      - run: 'true'\n"
    )
    _MENTIONS_JOB_ONLY = (
        "name: fixture\n"
        "on: [push]\n"
        "# waits on the queue-validated push skip (sq-6vshe.14)\n"
        "jobs:\n"
        "  # queue-validated:\n"
        "  build:\n"
        "    runs-on: ubuntu-latest\n"
        "    steps:\n"
        "      - run: 'true'\n"
    )

    def _scan(self, name: str, body: str) -> list[str]:
        with tempfile.TemporaryDirectory() as td:
            (Path(td) / name).write_text(body, encoding="utf-8")
            return workflows_defining_push_skip_job(Path(td))

    def test_a_defining_workflow_is_detected_in_either_extension(self) -> None:
        for name in ("push-skip.yml", "push-skip.yaml"):
            with self.subTest(workflow=name):
                self.assertEqual(
                    self._scan(name, self._DEFINES_JOB),
                    [name],
                    f"a workflow defining {'/'.join(sorted(PUSH_SKIP_JOB_IDS))} was not "
                    f"detected in {name}. GitHub Actions runs both `.yml` and `.yaml`, "
                    "so scanning only one extension would let the precondition clear "
                    "while the coupling test below kept passing on a stale "
                    "`status=blocked`.",
                )

    def test_a_prose_only_mention_is_ignored_in_either_extension(self) -> None:
        for name in ("prose.yml", "prose.yaml"):
            with self.subTest(workflow=name):
                self.assertEqual(
                    self._scan(name, self._MENTIONS_JOB_ONLY),
                    [],
                    f"{name} only MENTIONS the push skip in comments; counting that "
                    "would demand a doc reconciliation for a job that does not run.",
                )


class TestStatusValueRuleIsExact(unittest.TestCase):
    """The status/value rule decides whether a documented ruleset is one the
    maintainer was licensed to ask for, so pin it directly — the repo-state test
    only ever exercises the branch matching today's marker."""

    def test_applied_requires_exactly_the_approved_target(self) -> None:
        self.assertEqual(expected_documented_value("applied"), APPROVED_TARGET)

    def test_applied_rejects_every_off_target_value(self) -> None:
        for value in (BASELINE, APPROVED_TARGET - 1, APPROVED_TARGET + 1, 999):
            with self.subTest(value=value):
                self.assertNotEqual(
                    value,
                    expected_documented_value("applied"),
                    f"status=applied must accept only {APPROVED_TARGET}; {value} is "
                    f"not the edit issue #{STEER_ISSUE} authorises.",
                )

    def test_unapplied_statuses_require_the_baseline(self) -> None:
        for status in ("blocked", "unblocked"):
            with self.subTest(status=status):
                self.assertEqual(expected_documented_value(status), BASELINE)

    def test_an_unknown_status_is_an_error_not_a_silent_pass(self) -> None:
        with self.assertRaises(ValueError):
            expected_documented_value("shipped")


class TestMarkerAgreesWithTheRenderedDoc(unittest.TestCase):
    def test_marker_value_matches_the_table_row(self) -> None:
        row = _TABLE_ROW.search(DOC.read_text(encoding="utf-8"))
        assert row is not None  # pinned by TestParserNonVacuity
        self.assertEqual(
            marker().group("value"),
            row.group(1),
            "the `mergequeue-throughput-pin` marker and the rendered "
            "`max_entries_to_build` table row disagree. A reader sees the table; the "
            "coupling check below reads the marker — they must not drift.",
        )

    def test_marker_names_the_precondition_and_the_steer_issue(self) -> None:
        m = marker()
        self.assertEqual(
            m.group("precondition"),
            PRECONDITION_BEAD,
            "the marker must name the bead the raise is gated on, so a reader who hits "
            "this pin can find the blocking work.",
        )
        self.assertEqual(
            m.group("steer"),
            STEER_ISSUE,
            "the marker must name the maintainer steer issue — agents cannot edit "
            "rulesets, so that issue is the only channel the edit can be applied "
            "through.",
        )


class TestDeferredRaiseIsCoupledToThePrecondition(unittest.TestCase):
    """The whole point of the file: the doc's status claim must match the repo."""

    def test_status_tracks_whether_the_push_skip_job_exists(self) -> None:
        defining = workflows_defining_push_skip_job()
        status = marker().group("status")
        if defining:
            self.assertNotEqual(
                status,
                "blocked",
                f"{PRECONDITION_BEAD} HAS LANDED — {', '.join(defining)} now defines a "
                f"push-skip job ({'/'.join(sorted(PUSH_SKIP_JOB_IDS))}), so the "
                f"precondition on the `max_entries_to_build` {BASELINE} -> "
                f"{APPROVED_TARGET} raise is CLEARED, but "
                f"{DOC.relative_to(REPO_ROOT)} still records it as blocked. Reconcile "
                f"§Merge-queue throughput settings item (a): set the marker to "
                f"`status=unblocked`, drop the 'no such job exists' claim, and ping "
                f"issue #{STEER_ISSUE} so the maintainer can apply the ruleset edit "
                "(agents cannot). Re-dump the live ruleset before requesting it — the "
                f"documented value {BASELINE} is profile-sourced, not API-verified.",
            )
        else:
            self.assertEqual(
                status,
                "blocked",
                f"the marker claims status={status!r}, but no workflow in "
                f".github/workflows/ defines a push-skip job "
                f"({'/'.join(sorted(PUSH_SKIP_JOB_IDS))}), so {PRECONDITION_BEAD} has "
                "NOT landed and the raise is still gated. Raising queue parallelism "
                "before the redundant push waves are skipped worsens runner contention "
                "rather than relieving it (research/ci-mergequeue-speedup-2026-07.md "
                "§3.1, §3.3).",
            )

    def test_documented_value_matches_the_claimed_status(self) -> None:
        m = marker()
        status, value = m.group("status"), int(m.group("value"))
        self.assertIn(status, VALID_STATUSES)
        if status == "applied":
            self.assertEqual(
                value,
                expected_documented_value(status),
                f"status=applied claims the maintainer applied the ruleset edit issue "
                f"#{STEER_ISSUE} authorises — `max_entries_to_build` {BASELINE} -> "
                f"{APPROVED_TARGET}, nothing else touched — but the documented value is "
                f"{value}. Either the edit was not actually applied, or the doc records "
                f"a value that was never approved; {APPROVED_TARGET} is the only figure "
                "measured (research/ci-mergequeue-speedup-2026-07.md §3.3) and the only "
                "one the steer issue carries.",
            )
        else:
            self.assertEqual(
                value,
                expected_documented_value(status),
                f"status={status} means the ruleset edit has NOT been applied, so the "
                f"documented value must still be the live baseline {BASELINE}. Writing "
                "the target value into the doc before the maintainer applies it makes "
                "this document describe a ruleset that does not exist.",
            )


class TestSuiteIsWiredIntoCi(unittest.TestCase):
    """A structural test that never runs is a comment. Pin its own call site."""

    def test_docs_quality_invokes_this_suite(self) -> None:
        dq = (WORKFLOWS / "docs-quality.yml").read_text(encoding="utf-8")
        self.assertIn(
            "scripts/tests/test_mergequeue_throughput_posture.py",
            dq,
            "this suite must be invoked by docs-quality.yml or it silently stops gating.",
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
