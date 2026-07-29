#!/usr/bin/env python3
# [OPUS-4.8] Hermetic unit tests for the nightly change-based-selection bug ALARM
# (bead sq-va7at, epic sq-fmx4u). Design: research/change-based-test-selection.md
# §6.1. Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable
# returns).
#
# Covers the alarm's load-bearing behaviours:
#   * job_crate_tokens: delimited-token match (no substring collisions).
#   * correlate: precise per-crate finding, triage finding for narrowed shards,
#     the non-spam property (empty skip-set => no finding), and that a crate whose
#     tests actually RAN in the window is NOT flagged.
#   * TRIAGE_LANE_RE matches the REAL ci.yml shard job name.
#   * fail-loud: replay errors AND a total blocker both exit NON-ZERO.
#   * idempotency-key stability + the dry-run path (no gh calls).
#
# [OPUS-5] sparq-org/sparq#4965 — THE ADMISSION SEAM. The alarm's analysis was always
# correct and its workflow `if:` was where it died: the job admitted only
# `github.event.workflow_run.conclusion == 'failure'`, and fail-fast cancellation
# means a nightly with real failures concludes `cancelled`. 266 of 267 alarm runs were
# job-SKIPPED, and a skipped job is never red. Three new classes pin the fix, and each
# one can go RED:
#   * TestKnownPositiveRun30333511110 — the REAL job census of the real cancelled
#     nightly that carried four real failures. Asserts the alarm fires and names
#     EXACTLY sparq-mpc / sparq-fedplan / sparq-reason. The fixture is its own
#     negative control: sparq-core and sparq-geo sit in the SAME job family with
#     `cancelled` conclusions, so an admission that let cancelled jobs through would
#     name them too and this test would red.
#   * TestQuietOnACleanCancelledRun — the NOISE half. A cancelled run with nothing
#     genuinely failed must exit 0 SILENTLY (no finding, no red). Without this,
#     "admit everything" satisfies the positive test and buys a nightly alarm that
#     cries wolf 35 times a window — muted is inert by a slower route.
#   * TestWorkflowAdmissionSeam — evaluates the LIVE `if:` from
#     .github/workflows/selection-alarm.yml against real event payloads, with the
#     evaluator itself validated against a KNOWN ANSWER: the verbatim OLD expression
#     must reject the known positive and admit the 2026-07-19 `failure` run, exactly
#     as GitHub was observed to behave. An instrument that cannot reproduce the bug
#     cannot certify the fix.
#
# Fully hermetic: no gh, no git, no cargo, no network. Correlation is a pure
# function; the CLI is exercised via --failed-jobs-file / a stubbed `_run` +
# --metadata-file + --dry-run and a monkeypatched git-log/diff.
#
# Run:  python3 scripts/tests/test_ci_selection_alarm.py   (stdlib + PyYAML, which the
#       docs-quality quick-gates job installs once in its shared setup)

from __future__ import annotations

import importlib.util
import io
import json
import re
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
ALARM = REPO_ROOT / "scripts" / "ci_selection_alarm.py"
CI_YML = REPO_ROOT / ".github" / "workflows" / "ci.yml"
ALARM_YML = REPO_ROOT / ".github" / "workflows" / "selection-alarm.yml"


def _load_module():
    spec = importlib.util.spec_from_file_location("ci_selection_alarm", ALARM)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["ci_selection_alarm"] = mod  # resolve dataclass string annotations
    spec.loader.exec_module(mod)
    return mod


A = _load_module()

MEMBERS = {"sparq-core", "sparq-engine", "sparq-vectors", "sparq-geo", "sparq-rsp"}


def _wc(sha: str, pr: int | None, skipped: set[str]) -> "A.WindowCommit":
    return A.WindowCommit(sha=sha, pr=pr, skipped=frozenset(skipped))


class TestJobCrateTokens(unittest.TestCase):
    def test_feature_matrix_leg_names_its_crate(self):
        self.assertEqual(
            A.job_crate_tokens("opt-in sparq-engine (zk)", MEMBERS), {"sparq-engine"}
        )

    def test_no_substring_collision(self):
        # `sparq-core` must NOT match inside a longer hyphenated token.
        self.assertEqual(A.job_crate_tokens("build sparq-core-foo bundle", MEMBERS), set())
        # `sparq-vec` (not a member) present-as-prefix must not yield sparq-vectors,
        # and sparq-vectors delimited DOES match.
        self.assertEqual(
            A.job_crate_tokens("test sparq-vectors recall", MEMBERS), {"sparq-vectors"}
        )

    def test_generic_shard_names_no_crate(self):
        for name in ("test (load-aware shard bulk 1/3)",
                     "test (load-aware shard heavy-diskann)",
                     "coverage (nightly, full incl. heavy vectors)"):
            self.assertEqual(A.job_crate_tokens(name, MEMBERS), set(), name)

    def test_multiple_crates_all_implicated(self):
        self.assertEqual(
            A.job_crate_tokens("wasm (sparq-core, sparq-engine)", MEMBERS),
            {"sparq-core", "sparq-engine"},
        )


class TestCorrelate(unittest.TestCase):
    def test_precise_hit(self):
        # sparq-geo failed a named leg AND was skipped by PR #10 => precise finding.
        window = [_wc("a" * 40, 10, {"sparq-geo"}), _wc("b" * 40, 11, set())]
        findings = A.correlate(["opt-in sparq-geo (geosparql)"], window, MEMBERS)
        self.assertEqual(len(findings), 1)
        f = findings[0]
        self.assertEqual((f.kind, f.crate, f.key), ("crate", "sparq-geo", "crate:sparq-geo"))
        self.assertEqual(f.failed_jobs, ["opt-in sparq-geo (geosparql)"])
        self.assertEqual([wc.pr for wc in f.suspect_prs], [10])

    def test_named_job_but_crate_not_skipped_is_ignored(self):
        # sparq-geo failed but was NOT skipped anywhere => its own tests ran per-PR,
        # so it is NOT a selection bug. No finding.
        window = [_wc("a" * 40, 10, {"sparq-rsp"})]
        findings = A.correlate(["opt-in sparq-geo (geosparql)"], window, MEMBERS)
        # sparq-rsp was skipped but no failed job names it => still no finding.
        self.assertEqual(findings, [])

    def test_triage_for_narrowed_shard(self):
        # A generic shard failed AND selection skipped something => fail-safe triage.
        window = [_wc("a" * 40, 20, {"sparq-vectors"}), _wc("b" * 40, 21, {"sparq-geo"})]
        findings = A.correlate(["test (load-aware shard bulk 2/3)"], window, MEMBERS)
        self.assertEqual(len(findings), 1)
        f = findings[0]
        self.assertEqual(f.kind, "triage")
        self.assertIsNone(f.crate)
        self.assertEqual(f.failed_jobs, ["test (load-aware shard bulk 2/3)"])
        # both skip-bearing PRs are suspects
        self.assertEqual(sorted(wc.pr for wc in f.suspect_prs), [20, 21])

    def test_non_spam_empty_skipset_no_finding(self):
        # Nightly failed but NOTHING was skipped in the window => cannot be a
        # selection bug => no alarm.
        window = [_wc("a" * 40, 30, set()), _wc("b" * 40, 31, set())]
        self.assertEqual(
            A.correlate(
                ["test (load-aware shard bulk 1/3)", "opt-in sparq-core (mmap)"],
                window, MEMBERS,
            ),
            [],
        )

    def test_non_test_infra_failure_not_triaged(self):
        # A non-shard, non-crate infra lane failing must NOT mint a triage alarm,
        # even with active skips.
        window = [_wc("a" * 40, 40, {"sparq-geo"})]
        findings = A.correlate(["coverage floors (presence + monotonicity)"], window, MEMBERS)
        self.assertEqual(findings, [])

    def test_precise_and_triage_together(self):
        window = [_wc("a" * 40, 50, {"sparq-geo", "sparq-vectors"})]
        findings = A.correlate(
            ["opt-in sparq-geo (geosparql)", "test (load-aware shard bulk 3/3)"],
            window, MEMBERS,
        )
        kinds = sorted(f.kind for f in findings)
        self.assertEqual(kinds, ["crate", "triage"])


class TestTriageLaneRegexMatchesRealCi(unittest.TestCase):
    def test_regex_matches_ci_shard_job_name(self):
        # Pin the triage regex to the REAL ci.yml shard job name so a rename of the
        # shard lane can't silently drop the fail-safe triage path.
        text = CI_YML.read_text()
        self.assertIn("test (load-aware shard ${{ matrix.name }})", text,
                      "ci.yml shard job name changed — update TRIAGE_LANE_RE + this pin")
        # Concrete expanded names must match.
        for expanded in ("test (load-aware shard bulk 1/3)",
                         "test (load-aware shard heavy-diskann)"):
            self.assertTrue(A.TRIAGE_LANE_RE.match(expanded), expanded)


class TestRenderIssue(unittest.TestCase):
    def test_crate_issue_has_key_marker_and_selfid(self):
        f = A.Finding(kind="crate", key="crate:sparq-geo", crate="sparq-geo",
                      failed_jobs=["opt-in sparq-geo (geosparql)"],
                      suspect_prs=[_wc("c" * 40, 77, {"sparq-geo"})])
        title, body = A.render_issue(f, "999", "d" * 40, "sparq-org/sparq")
        self.assertIn("sparq-geo", title)
        self.assertIn(A.key_marker("crate:sparq-geo"), body)
        self.assertIn("\U0001f916 SPARQ agent", body)
        self.assertIn("#77", body)
        self.assertIn("P1", body)

    def test_triage_issue_lists_skipped_crates(self):
        f = A.Finding(kind="triage", key="triage:test (load-aware shard bulk 1/3)",
                      crate=None, failed_jobs=["test (load-aware shard bulk 1/3)"],
                      suspect_prs=[_wc("e" * 40, 88, {"sparq-vectors"})])
        _title, body = A.render_issue(f, "1", "f" * 40, None)
        self.assertIn("sparq-vectors", body)  # skipped set shown for triage


# --------------------------------------------------------------------------- #
# #4984 — THE RENDERED SUSPECT LIST
# --------------------------------------------------------------------------- #

# MEASURED 2026-07-28 by driving real nightly run 30333511110 through the real
# script (`--dry-run`, real gh + git replay + cargo metadata): the `crate:` findings
# carried 644 / 579 / 554 suspect PRs in 18,064 / 16,329 / 15,755-char bodies. The
# suspect set tracks the AGE OF THE BACKSTOP (last green nightly 2026-07-01), not the
# size of the bug, and dedupe is per-key — so the FIRST render of a key is the ONLY
# render, and whatever body lands is what persists.
MEASURED_SUSPECT_COUNT = 644

# GitHub's hard issue-body limit. Past it `gh issue create` 422s.
GITHUB_ISSUE_BODY_LIMIT = 65_536

SUSPECT_SECTION_HEADER = "**Suspect landed PR(s)**"
SUSPECT_SECTION_END = "**Nightly run:**"

# The disclosed count, read back out of the rendered body.
_OMITTED_RE = re.compile(r"\*\*(\d+) more suspect PR\(s\) not listed\.\*\*")


def _suspect_rows(body: str) -> list[str]:
    """The list rows of the rendered suspect section.

    Sliced between two FULL, distinctive anchor strings rather than split on a bare
    token — a substring that also occurs inside another word would silently truncate
    what this examines, and a row-count instrument that under-counts fails toward
    'the cap is working'. Both anchors are looked up with `.index`, so a rename REDs
    here instead of returning a plausible empty list. Validated against a known
    answer in test_short_list_renders_in_full_with_no_omission_notice (3 suspects in
    => exactly 3 rows out)."""
    start = body.index(SUSPECT_SECTION_HEADER)
    end = body.index(SUSPECT_SECTION_END, start)
    return [ln for ln in body[start:end].splitlines() if ln.startswith("- ")]


def _suspects(count: int, skipped: set[str], newest_pr: int = 20_000):
    """`count` suspects in WINDOW order — newest-landed first, as `git log` and
    `correlate` produce them. PR numbers DESCEND from `newest_pr` so the test can
    tell which end of the list survived the cap."""
    return [
        _wc(sha=f"{i:040x}", pr=newest_pr - i, skipped=skipped) for i in range(count)
    ]


def _crate_finding(suspects) -> "A.Finding":
    return A.Finding(
        kind="crate", key="crate:sparq-fedplan", crate="sparq-fedplan",
        failed_jobs=["mutation ratchet (cargo-mutants, advisory) (sparq-fedplan)"],
        suspect_prs=suspects,
    )


class TestSuspectOrderIsLandingRecency(unittest.TestCase):
    """The render truncates from ONE end and tells the reader that end is 'newest
    landed'. `WindowCommit` carries no timestamp, so that claim is not re-derivable
    at the render — it rests on `correlate` PRESERVING the order `git log` produced.
    These pin that invariant, so the ordering the body advertises cannot quietly
    become an arbitrary slice of an unordered set."""

    # `git log` emits reverse-chronological; `window_commits` does not re-sort. So a
    # window is newest-landed FIRST, and here PR numbers descend to encode that.
    WINDOW_PRS = [1006, 1005, 1004, 1003, 1002, 1001]

    def _window(self, skipped: set[str]):
        return [_wc(f"{pr:040d}", pr, skipped) for pr in self.WINDOW_PRS]

    def test_crate_finding_preserves_window_order(self):
        window = self._window({"sparq-geo"})
        findings = A.correlate(["opt-in sparq-geo (geosparql)"], window, MEMBERS)
        self.assertEqual(
            [wc.pr for wc in findings[0].suspect_prs], self.WINDOW_PRS,
            "correlate reordered the suspects; the rendered list claims "
            "'newest-landed first' and would then be lying",
        )

    def test_triage_finding_preserves_window_order(self):
        window = self._window({"sparq-geo"})
        findings = A.correlate(["test (load-aware shard bulk 1/3)"], window, MEMBERS)
        self.assertEqual(
            [wc.pr for wc in findings[0].suspect_prs], self.WINDOW_PRS,
            "the triage suspect list must also stay in landing order",
        )

    def test_the_cap_keeps_the_NEWEST_suspects_not_an_arbitrary_slice(self):
        """CRITERION 3. Which END survives is the whole ordering claim. Keeping the
        oldest — or any middle slice — would still 'render within a bound' and still
        state an omitted count, and would still be wrong."""
        suspects = _suspects(MEASURED_SUSPECT_COUNT, {"sparq-fedplan"}, newest_pr=20_000)
        self.assertEqual(suspects[0].pr, 20_000, "fixture must be newest-first")
        _title, body = A.render_issue(_crate_finding(suspects), "1", "a" * 40, None)
        rows = _suspect_rows(body)
        # The 20 newest landed are PRs 20000, 19999, … 19981 — descending, in order.
        self.assertEqual(
            [int(re.match(r"- #(\d+) ", r).group(1)) for r in rows],
            list(range(20_000, 19_980, -1)),
            "the cap must keep the 20 most recently landed suspects, in that order",
        )


class TestSuspectListBound(unittest.TestCase):
    """CRITERION 1 + 2. A stale-backstop suspect list must render BOUNDED and NAME
    what it omitted; a short list must render IN FULL. The pair is load-bearing —
    criterion 1 alone is satisfiable by always truncating, which would make every
    complete list look incomplete."""

    def test_644_suspects_render_bounded_and_name_the_omitted_count(self):
        """CRITERION 1, at the measured size. Delete the cap and every assertion here
        REDs: 644 rows rendered, no disclosure regex match, body back over 18 KB."""
        suspects = _suspects(MEASURED_SUSPECT_COUNT, {"sparq-fedplan"})
        _title, body = A.render_issue(
            _crate_finding(suspects), "30333511110", "6" * 40, "sparq-org/sparq"
        )
        rows = _suspect_rows(body)
        # The stated bound: 20 rows. A literal, not a read-back of the constant —
        # changing MAX_RENDERED_SUSPECTS must force the bound to be re-stated here.
        self.assertEqual(len(rows), 20, f"suspect list not capped at 20 rows: {len(rows)}")

        m = _OMITTED_RE.search(body)
        self.assertIsNotNone(m, f"644 suspects rendered as 20 rows with NO omission "
                                f"notice — that is silent truncation:\n{body}")
        # 644 suspects - 20 rendered = 624 omitted. Arithmetic done here, not copied
        # from the implementation.
        self.assertEqual(int(m.group(1)), 624)
        # …and the disclosure must match what was ACTUALLY rendered, so a body cannot
        # state a bound it did not honour.
        self.assertEqual(int(m.group(1)), MEASURED_SUSPECT_COUNT - len(rows))
        # The total is stated too, so the reader never has to add the two numbers.
        self.assertIn("644 total", body)

    def test_body_size_no_longer_tracks_the_window(self):
        """The actual defect: body length was a function of BACKSTOP AGE. After the
        cap, growing the window 3.6x must not grow the body — only the digits of the
        printed total change. Measured pre-fix: 644 suspects => 18,064 chars."""
        small = A.render_issue(
            _crate_finding(_suspects(644, {"sparq-fedplan"})), "1", "a" * 40, None)[1]
        # 2,300 is where an UNCAPPED body crosses GitHub's 65,536-char limit.
        huge = A.render_issue(
            _crate_finding(_suspects(2_300, {"sparq-fedplan"})), "1", "a" * 40, None)[1]
        self.assertLess(
            abs(len(huge) - len(small)), 50,
            f"body still grows with the window: {len(small)} -> {len(huge)} chars",
        )
        self.assertLess(len(huge), 4_000, "crate-finding body exceeded its stated bound")
        self.assertLess(len(huge), GITHUB_ISSUE_BODY_LIMIT)

    def test_triage_body_is_bounded_too(self):
        """Triage rows also carry each suspect's whole skip-set, so they are the
        widest rows the alarm renders. That width is O(workspace), not O(window) —
        but the ROW COUNT is the window-dependent term and must be capped here too."""
        members = {f"sparq-{i:02d}" for i in range(37)}  # real workspace order of size
        suspects = _suspects(MEASURED_SUSPECT_COUNT, members)
        f = A.Finding(kind="triage", key="triage:test (load-aware shard bulk 1/3)",
                      crate=None, failed_jobs=["test (load-aware shard bulk 1/3)"],
                      suspect_prs=suspects)
        _title, body = A.render_issue(f, "1", "a" * 40, None)
        self.assertEqual(len(_suspect_rows(body)), 20)
        self.assertIn("624 " + A.SUSPECT_OMISSION_MARKER, body)
        self.assertLess(len(body), 20_000, "triage body exceeded its stated bound")
        # …and the per-row skip-set survives the cap — it is what triage narrows from.
        self.assertIn("— skipped: sparq-00", body)

    def test_short_list_renders_in_full_with_no_omission_notice(self):
        """CRITERION 2, the PAIRED property. Without it, `suspects[:0]` — truncate
        everything, always — passes criterion 1. Also the KNOWN-ANSWER validation of
        the `_suspect_rows` instrument: 3 suspects in, exactly 3 rows out."""
        suspects = _suspects(3, {"sparq-fedplan"}, newest_pr=100)
        _title, body = A.render_issue(_crate_finding(suspects), "1", "a" * 40, None)
        rows = _suspect_rows(body)
        self.assertEqual(len(rows), 3, f"a 3-suspect list must render in full:\n{body}")
        for pr in (100, 99, 98):
            self.assertIn(f"#{pr}", body, "every suspect of a short list must appear")
        self.assertNotIn(
            A.SUSPECT_OMISSION_MARKER, body,
            "a COMPLETE list must carry no omission notice — otherwise the notice "
            "means nothing and 'omitted' stops being readable as a signal",
        )
        self.assertIsNone(_OMITTED_RE.search(body))
        self.assertIn("3 total", body)

    def test_exactly_at_the_cap_renders_in_full(self):
        """OFF-BY-ONE, complete side. 20 suspects is not an omission; a `<` where
        `<=` belongs would claim '0 more not listed' on a complete list."""
        _title, body = A.render_issue(
            _crate_finding(_suspects(20, {"sparq-fedplan"})), "1", "a" * 40, None)
        self.assertEqual(len(_suspect_rows(body)), 20)
        self.assertNotIn(A.SUSPECT_OMISSION_MARKER, body)

    def test_one_over_the_cap_discloses_exactly_one_omission(self):
        """OFF-BY-ONE, truncated side. 21 suspects, 20 rendered => exactly 1 omitted."""
        _title, body = A.render_issue(
            _crate_finding(_suspects(21, {"sparq-fedplan"})), "1", "a" * 40, None)
        rows = _suspect_rows(body)
        self.assertEqual(len(rows), 20)
        m = _OMITTED_RE.search(body)
        self.assertIsNotNone(m, f"21 suspects capped at 20 must disclose:\n{body}")
        self.assertEqual(int(m.group(1)), 1)

    def test_the_notice_states_the_ordering_basis_and_the_real_cause(self):
        """CRITERION 3 + the anti-suppression rule. A bounded list that does not say
        HOW it was ordered is an arbitrary first-N wearing a judgement's clothes; and
        the body must point at the stale backstop rather than imply the alarm is
        noisy."""
        _title, body = A.render_issue(
            _crate_finding(_suspects(644, {"sparq-fedplan"})), "1", "a" * 40, None)
        self.assertIn("Ordering is landing recency, newest first", body)
        self.assertIn("last GREEN nightly is far behind HEAD", body)
        self.assertIn("--dry-run", body)  # the full set stays reproducible

    def test_render_suspects_never_drops_rows_on_a_nonsense_cap(self):
        """A negative cap must not silently become a from-the-end slice
        (`suspects[:-1]` quietly drops the OLDEST suspect and discloses nothing)."""
        out = A.render_suspects(_suspects(5, {"x"}), "crate", cap=-1)
        self.assertIn("5 " + A.SUSPECT_OMISSION_MARKER, out)


class TestCliFailLoud(unittest.TestCase):
    """The CLI must exit NON-ZERO on any infra gap (fail-loud), never mask."""

    def _meta(self) -> dict:
        # Minimal cargo-metadata-shaped dict (see ci_select.parse_workspace).
        root = "/repo"
        pkgs = [
            {"id": "core 0", "name": "sparq-core", "source": None,
             "manifest_path": f"{root}/crates/sparq-core/Cargo.toml", "dependencies": []},
            {"id": "geo 0", "name": "sparq-geo", "source": None,
             "manifest_path": f"{root}/crates/sparq-geo/Cargo.toml",
             "dependencies": [{"name": "sparq-core"}]},
        ]
        return {"workspace_root": root, "packages": pkgs,
                "workspace_members": ["core 0", "geo 0"]}

    def test_replay_error_exits_nonzero(self):
        mod = _load_module()
        with tempfile.TemporaryDirectory() as d:
            meta_f = Path(d) / "meta.json"
            meta_f.write_text(json.dumps(self._meta()))
            jobs_f = Path(d) / "jobs.txt"
            jobs_f.write_text("test (load-aware shard bulk 1/3)\n")

            # Force one landed commit, then make the per-commit diff replay FAIL.
            mod.window_commits = lambda *a, **k: [("f" * 40, 123)]

            def _boom(sha, repo_root):
                raise mod.AlarmError("simulated git diff failure")

            mod.commit_changed_paths = _boom
            rc = mod.main([
                "--run-id", "1", "--head-sha", "f" * 40, "--repo", "o/r",
                "--repo-root", d, "--failed-jobs-file", str(jobs_f),
                "--metadata-file", str(meta_f), "--dry-run",
            ])
            self.assertEqual(rc, 1, "a replay error must fail loud (non-zero)")

    def test_total_blocker_exits_nonzero(self):
        mod = _load_module()
        # No --failed-jobs-file and no --repo => gather_failed_jobs total blocker.
        with tempfile.TemporaryDirectory() as d:
            meta_f = Path(d) / "meta.json"
            meta_f.write_text(json.dumps(self._meta()))
            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = mod.main([
                    "--run-id", "1", "--head-sha", "f" * 40,
                    "--repo-root", d, "--metadata-file", str(meta_f),
                ])
            self.assertEqual(rc, 1)

    def test_empty_failed_jobs_file_exits_nonzero(self):
        # An empty --failed-jobs-file on a failure-concluded run is anomalous
        # (jobs-API filter=latest re-run race or shape surprise) — must exit
        # NON-ZERO, not silently "no alarm" exit 0.  Covers the fix at the
        # run_alarm call site (sq-va7at reviewer concern 1).
        mod = _load_module()
        with tempfile.TemporaryDirectory() as d:
            meta_f = Path(d) / "meta.json"
            meta_f.write_text(json.dumps(self._meta()))
            jobs_f = Path(d) / "jobs.txt"
            jobs_f.write_text("")  # empty — simulates API race / shape surprise
            rc = mod.main([
                "--run-id", "1", "--head-sha", "f" * 40, "--repo", "o/r",
                "--repo-root", d, "--failed-jobs-file", str(jobs_f),
                "--metadata-file", str(meta_f), "--dry-run",
            ])
            self.assertEqual(rc, 1,
                             "empty failed-jobs on failure-concluded run must exit non-zero")

    def test_dry_run_reports_finding_without_gh(self):
        # End-to-end via the CLI dry-run path (no gh writes), exercising the REAL
        # ci_select replay. The landed commit touches only crates/sparq-geo/src
        # => geo is affected (it has no dependents), so sparq-core is SKIPPED
        # (mode=selected). A failed nightly leg naming sparq-core then intersects
        # that skip => one precise finding, printed but not filed.
        mod = _load_module()
        with tempfile.TemporaryDirectory() as d:
            meta_f = Path(d) / "meta.json"
            meta_f.write_text(json.dumps(self._meta()))
            jobs_f = Path(d) / "jobs.txt"
            jobs_f.write_text("opt-in sparq-core (mmap)\n")

            mod.window_commits = lambda *a, **k: [("a" * 40, 55)]
            mod.commit_changed_paths = lambda sha, repo_root: ["crates/sparq-geo/src/x.rs"]

            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = mod.main([
                    "--run-id", "42", "--head-sha", "b" * 40, "--repo", "o/r",
                    "--repo-root", d, "--failed-jobs-file", str(jobs_f),
                    "--metadata-file", str(meta_f), "--dry-run",
                ])
            out = buf.getvalue()
            self.assertEqual(rc, 0)
            self.assertIn("[dry-run] WOULD file", out)
            self.assertIn("crate:sparq-core", out)
            self.assertIn("#55", out)


# --------------------------------------------------------------------------- #
# #4965 — THE ADMISSION SEAM
# --------------------------------------------------------------------------- #

# The REAL job census of nightly CI run 30333511110 (sparq-org/sparq, 2026-07-28,
# event=schedule, head_branch=main). Fetched from
# `repos/sparq-org/sparq/actions/runs/30333511110/jobs` with `gh api --paginate`
# (--limit would have truncated silently): 84 jobs — 43 success, 31 cancelled,
# 6 skipped, 4 FAILURE. GitHub concluded the RUN `cancelled`.
#
# The four failures are reproduced VERBATIM. The cancelled/success/skipped rows are a
# faithful sample of the real names, chosen to preserve the property that matters: the
# cancelled rows are SIBLINGS of the failed ones in the very same job family, naming
# crates that were ALSO selection-skipped. So "admit any non-success job" and
# "admit the run, then correlate every job" both name extra crates and red the test.
KNOWN_POSITIVE_RUN_ID = "30333511110"
KNOWN_POSITIVE_HEAD_SHA = "63dfb7a0b0a8c92c8b478929a09b650843ce0a87"
KNOWN_POSITIVE_RUN_CONCLUSION = "cancelled"
KNOWN_POSITIVE_JOBS: list[tuple[str, str]] = [
    # (job name, conclusion) — the four GENUINE failures:
    ("mutation ratchet (cargo-mutants, advisory) (sparq-mpc)", "failure"),
    ("mutation ratchet (cargo-mutants, advisory) (sparq-fedplan)", "failure"),
    ("mutation ratchet (cargo-mutants, advisory) (sparq-reason, 1/3)", "failure"),
    ("mutation ratchet (cargo-mutants, advisory) (sparq-reason, 2/3)", "failure"),
    # …fail-fast cancelled siblings, same family, crates also in the skipped set:
    ("mutation ratchet (cargo-mutants, advisory) (sparq-core)", "cancelled"),
    ("mutation ratchet (cargo-mutants, advisory) (sparq-geo)", "cancelled"),
    ("mutation ratchet (cargo-mutants, advisory) (sparq-shacl)", "cancelled"),
    ("mutation ratchet (cargo-mutants, advisory) (sparq-solid)", "cancelled"),
    ("mutation ratchet (cargo-mutants, advisory) (sparq-server)", "cancelled"),
    ("mutation ratchet (cargo-mutants, advisory) (sparq-vectors)", "cancelled"),
    ("mutation ratchet (cargo-mutants, advisory) (sparq-engine, 1/24, 360)", "cancelled"),
    # …and green/skipped rows.
    ("unsafe-register (count ratchet)", "success"),
    ("detect rust changes", "success"),
    ("nightly-gate", "success"),
    ("coverage (nightly, full incl. heavy vectors)", "skipped"),
]

# The crates the four REAL failures name. Anything else appearing is over-admission.
KNOWN_POSITIVE_CRATES = {"sparq-mpc", "sparq-fedplan", "sparq-reason"}


def _jobs_tsv(jobs: list[tuple[str, str]]) -> str:
    """The exact `gh api --jq '.jobs[] | [.name, (.conclusion // "")] | @tsv'` shape."""
    return "".join(f"{name}\t{conclusion}\n" for name, conclusion in jobs)


class _StubRun:
    """Stands in for ci_selection_alarm._run so the REAL gh/git call sites execute.

    Dispatches on the actual argv the script builds, so a change to those call sites
    surfaces here as an unstubbed-command error rather than a silent pass."""

    def __init__(self, jobs: list[tuple[str, str]], base_sha: str | None,
                 commits: list[tuple[str, int | None]],
                 diffs: dict[str, list[str]], run_conclusion: str = ""):
        self.jobs, self.base_sha = jobs, base_sha
        self.commits, self.diffs = commits, diffs
        self.run_conclusion = run_conclusion
        self.calls: list[list[str]] = []

    def __call__(self, cmd, cwd=None, check=True):  # noqa: ANN001 — mirrors _run
        self.calls.append(list(cmd))
        joined = " ".join(cmd)
        if cmd[0] == "gh" and "/jobs" in joined:
            return _jobs_tsv(self.jobs)
        if cmd[0] == "gh" and any(re.search(r"actions/runs/\d+$", c) for c in cmd):
            return self.run_conclusion + "\n"
        if cmd[0] == "gh" and "status=success" in joined:
            return (self.base_sha or "") + "\n"
        if cmd[:2] == ["git", "log"]:
            return "".join(f"{sha}\x00subject (#{pr})\n" for sha, pr in self.commits)
        if cmd[:2] == ["git", "diff"]:
            return "".join(f"{p}\n" for p in self.diffs.get(cmd[-1], []))
        raise AssertionError(f"unstubbed command: {cmd}")


def _metadata_with(members: list[str], deps: dict[str, list[str]] | None = None) -> dict:
    """Minimal cargo-metadata-shaped dict (see ci_select.parse_workspace)."""
    deps = deps or {}
    root = "/repo"
    pkgs = [
        {
            "id": f"{m} 0", "name": m, "source": None,
            "manifest_path": f"{root}/crates/{m}/Cargo.toml",
            "dependencies": [{"name": d} for d in deps.get(m, [])],
        }
        for m in members
    ]
    return {"workspace_root": root, "packages": pkgs,
            "workspace_members": [f"{m} 0" for m in members]}


class TestKnownPositiveRun30333511110(unittest.TestCase):
    """CRITERION 1. The real cancelled nightly that carried four real failures must
    make the alarm ACT, and name exactly the crates those four failures name."""

    MEMBERS = ["sparq-core", "sparq-geo", "sparq-mpc", "sparq-fedplan", "sparq-reason",
               "sparq-shacl", "sparq-solid", "sparq-server", "sparq-vectors",
               "sparq-engine"]

    def test_gather_failed_jobs_admits_only_the_four_genuine_failures(self):
        """The analysis half, driven through the REAL gather_failed_jobs over the real
        job census — cancelled siblings must not enter the failed set."""
        mod = _load_module()
        mod._run = _StubRun(KNOWN_POSITIVE_JOBS, None, [], {})
        failed = mod.gather_failed_jobs(
            KNOWN_POSITIVE_RUN_ID, "sparq-org/sparq", KNOWN_POSITIVE_RUN_CONCLUSION
        )
        self.assertEqual(
            len(failed), 4,
            f"expected the 4 genuinely-failed jobs of run {KNOWN_POSITIVE_RUN_ID}, "
            f"got {failed}",
        )
        crates: set[str] = set()
        for job in failed:
            crates |= mod.job_crate_tokens(job, set(self.MEMBERS))
        self.assertEqual(
            crates, KNOWN_POSITIVE_CRATES,
            "the failed jobs must map onto exactly the crates the four real failures "
            "name; extra crates mean cancelled siblings leaked into the failed set",
        )

    def test_end_to_end_alarm_fires_and_names_the_crates(self):
        """CRITERION 1, end to end through main(): the real cancelled run, the real job
        census, a landed PR that selection-skipped those crates => findings filed.

        Before the #4965 fix this run never reached the script at all — the workflow
        `if:` skipped the job (266 of 267 runs). Pinning it at the script level is only
        half the guard; TestWorkflowAdmissionSeam pins the other half."""
        mod = _load_module()
        with tempfile.TemporaryDirectory() as d:
            meta_f = Path(d) / "meta.json"
            # sparq-geo is the only crate the landed commit touches and nothing depends
            # on it, so ci_select selects {sparq-geo} and SKIPS every other member —
            # including the four failures' crates AND their cancelled siblings.
            meta_f.write_text(json.dumps(_metadata_with(self.MEMBERS)))
            landed_sha = "a" * 40
            mod._run = _StubRun(
                jobs=KNOWN_POSITIVE_JOBS,
                base_sha="9" * 40,
                commits=[(landed_sha, 4321)],
                diffs={landed_sha: ["crates/sparq-geo/src/lib.rs"]},
            )
            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = mod.main([
                    "--run-id", KNOWN_POSITIVE_RUN_ID,
                    "--head-sha", KNOWN_POSITIVE_HEAD_SHA,
                    "--run-conclusion", KNOWN_POSITIVE_RUN_CONCLUSION,
                    "--repo", "sparq-org/sparq", "--repo-root", d,
                    "--metadata-file", str(meta_f), "--dry-run",
                ])
            out = buf.getvalue()
            self.assertEqual(rc, 0, f"alarm exited {rc} on the known positive:\n{out}")
            self.assertIn("[dry-run] WOULD file", out,
                          "the alarm did NOT act on the known positive:\n" + out)
            for crate in sorted(KNOWN_POSITIVE_CRATES):
                self.assertIn(f"crate:{crate}", out,
                              f"finding for {crate} missing:\n{out}")
            self.assertIn("#4321", out, "the suspect landed PR must be named")
            # Negative control INSIDE the positive: crates whose jobs were merely
            # CANCELLED were skipped by the same PR, so they are in the skip set and
            # would be named the moment admission stopped distinguishing them.
            for cancelled_crate in ("sparq-core", "sparq-shacl", "sparq-server"):
                self.assertNotIn(
                    f"crate:{cancelled_crate}", out,
                    f"{cancelled_crate} was only CANCELLED, never failed — naming it "
                    f"means the admission stopped being precise:\n{out}",
                )


class TestQuietOnACleanCancelledRun(unittest.TestCase):
    """CRITERION 2 — the NOISE half. `cancelled` is the COMMON nightly conclusion (35
    of 45 all-time). A cancelled run with nothing genuinely failed must produce NO
    finding and NO red; otherwise the fix trades a silent alarm for a muted one."""

    def test_clean_cancelled_run_is_a_quiet_no_op(self):
        mod = _load_module()
        clean = [(name, "cancelled" if c == "failure" else c)
                 for name, c in KNOWN_POSITIVE_JOBS]
        self.assertNotIn("failure", {c for _, c in clean},
                         "fixture must contain NO genuine failure for this control")
        with tempfile.TemporaryDirectory() as d:
            meta_f = Path(d) / "meta.json"
            meta_f.write_text(json.dumps(_metadata_with(["sparq-core", "sparq-geo"])))
            mod._run = _StubRun(clean, base_sha="9" * 40, commits=[("a" * 40, 1)],
                                diffs={"a" * 40: ["crates/sparq-geo/src/lib.rs"]})
            buf = io.StringIO()
            with redirect_stdout(buf):
                rc = mod.main([
                    "--run-id", "1", "--head-sha", "b" * 40,
                    "--run-conclusion", "cancelled",
                    "--repo", "o/r", "--repo-root", d,
                    "--metadata-file", str(meta_f), "--dry-run",
                ])
            out = buf.getvalue()
            self.assertEqual(rc, 0, "a clean cancelled run must NOT red the alarm lane")
            self.assertNotIn("WOULD file", out, "a clean cancelled run must file nothing")
            self.assertIn("nothing genuinely failed", out,
                          "the quiet no-op must still say why it did nothing:\n" + out)

    def test_failure_concluded_run_with_no_failed_job_is_still_LOUD(self):
        """ANTI-REGRESSION on the fail-loud invariant. The quiet branch must be scoped
        to conclusions that can legitimately carry zero failures. A run GitHub concluded
        `failure` MUST contain a failed job, so an empty set there is an API-shape
        anomaly and must still exit NON-ZERO — otherwise #4965's fix would have bought
        its quiet by deleting the masking guard the alarm was built around."""
        mod = _load_module()
        green_only = [(n, "success") for n, _ in KNOWN_POSITIVE_JOBS]
        with tempfile.TemporaryDirectory() as d:
            meta_f = Path(d) / "meta.json"
            meta_f.write_text(json.dumps(_metadata_with(["sparq-core"])))
            for conclusion in ("failure", "timed_out", ""):
                with self.subTest(run_conclusion=conclusion or "<unknown>"):
                    mod._run = _StubRun(green_only, None, [], {},
                                        run_conclusion=conclusion)
                    buf = io.StringIO()
                    with redirect_stdout(buf):
                        rc = mod.main([
                            "--run-id", "1", "--head-sha", "b" * 40,
                            "--run-conclusion", conclusion,
                            "--repo", "o/r", "--repo-root", d,
                            "--metadata-file", str(meta_f), "--dry-run",
                        ])
                    self.assertEqual(
                        rc, 1,
                        f"a {conclusion or 'conclusion-unknown'}-concluded run with no "
                        f"failed job must fail LOUD, not exit 0",
                    )


# --- the workflow `if:` seam ------------------------------------------------- #

# The expression this job carried until #4965, verbatim from
# .github/workflows/selection-alarm.yml. It is the KNOWN ANSWER the evaluator below is
# validated against: it must REJECT the cancelled known positive (which is why the
# alarm was skipped 266 times) and ADMIT a failure-concluded nightly (which is why it
# acted once, on 2026-07-19). An evaluator that cannot reproduce the bug cannot
# certify the fix.
OLD_BROKEN_IF = (
    "github.event_name == 'workflow_dispatch' || "
    "(github.event.workflow_run.conclusion == 'failure' && "
    "(github.event.workflow_run.event == 'schedule' || "
    "github.event.workflow_run.event == 'workflow_dispatch') && "
    "github.event.workflow_run.head_branch == 'main')"
)

_TOKEN_RE = re.compile(r"\s*(\(|\)|\|\||&&|==|!=|'[^']*'|[A-Za-z_][A-Za-z0-9_.\-]*)")


def _tokenize(expr: str) -> list[str]:
    """Tokenize the SMALL GitHub-expression grammar these `if:`s use. Anything outside
    it RAISES — the evaluator is fail-closed, so an expression shape it does not
    understand REDs the suite instead of being silently judged admissible."""
    toks, pos = [], 0
    while pos < len(expr):
        m = _TOKEN_RE.match(expr, pos)
        if not m:
            if expr[pos].isspace():
                pos += 1
                continue
            raise ValueError(f"unsupported token at {pos}: {expr[pos:pos + 30]!r}")
        toks.append(m.group(1))
        pos = m.end()
    return toks


def _lookup(path: str, ctx: dict):
    """`github.event.workflow_run.event` -> ctx['github']['event']['workflow_run']
    ['event']; a missing leg is null, and null == 'literal' is False (GitHub semantics
    for an absent context property)."""
    cur = ctx
    for part in path.split("."):
        if not isinstance(cur, dict) or part not in cur:
            return None
        cur = cur[part]
    return cur


def eval_if(expr: str, ctx: dict) -> bool:
    """Evaluate a `||`/`&&`/`==`/`!=`/parenthesised GitHub `if:` against a context."""
    toks = _tokenize(expr)
    pos = 0

    def peek():
        return toks[pos] if pos < len(toks) else None

    def operand():
        nonlocal pos
        t = toks[pos]
        pos += 1
        if t.startswith("'"):
            return t[1:-1]
        if t in ("true", "false"):
            return t == "true"
        return _lookup(t, ctx)

    def primary():
        nonlocal pos
        if peek() == "(":
            pos += 1
            val = or_expr()
            if peek() != ")":
                raise ValueError("unbalanced parentheses")
            pos += 1
            return val
        left = operand()
        if peek() in ("==", "!="):
            op = toks[pos]
            pos += 1
            right = operand()
            return (left == right) if op == "==" else (left != right)
        return bool(left)

    def and_expr():
        nonlocal pos
        val = primary()
        while peek() == "&&":
            pos += 1
            val = primary() and val
        return val

    def or_expr():
        nonlocal pos
        val = and_expr()
        while peek() == "||":
            pos += 1
            val = and_expr() or val
        return val

    result = or_expr()
    if pos != len(toks):
        raise ValueError(f"trailing tokens: {toks[pos:]}")
    return bool(result)


def _workflow_run_ctx(conclusion: str, event: str = "schedule",
                      head_branch: str = "main") -> dict:
    return {"github": {"event_name": "workflow_run",
                       "event": {"workflow_run": {"conclusion": conclusion,
                                                  "event": event,
                                                  "head_branch": head_branch}}}}


class TestWorkflowAdmissionSeam(unittest.TestCase):
    """CRITERION 1+2 at the seam that actually killed the alarm. Every assertion drives
    the LIVE `if:` string parsed structurally out of the workflow, so a comment
    mentioning `conclusion` cannot satisfy or break it."""

    @staticmethod
    def _live_if() -> str:
        doc = yaml.safe_load(ALARM_YML.read_text())
        return " ".join(doc["jobs"]["alarm"]["if"].split())

    # -- instrument validation (run FIRST: a broken evaluator invalidates everything) --
    def test_evaluator_reproduces_the_old_bug(self):
        """KNOWN ANSWER. The pre-#4965 expression must SKIP the cancelled known positive
        and RUN on a failure-concluded nightly — exactly the live behaviour measured
        over 267 runs (266 skipped; the one action on the 2026-07-19 `failure`). If this
        ever passes trivially, every other assertion in this class is vacuous."""
        self.assertFalse(
            eval_if(OLD_BROKEN_IF, _workflow_run_ctx("cancelled")),
            "CONTROL FAILED: the evaluator thinks the OLD expression admitted a "
            "cancelled run. It did not — that is the entire defect of #4965, and an "
            "instrument that cannot see it cannot certify the fix.",
        )
        self.assertTrue(
            eval_if(OLD_BROKEN_IF, _workflow_run_ctx("failure")),
            "CONTROL FAILED: the evaluator thinks the OLD expression rejected a "
            "failure-concluded nightly. It admitted them — that is how the alarm acted "
            "on 2026-07-19.",
        )

    def test_evaluator_is_fail_closed_on_an_unknown_shape(self):
        """An expression using syntax outside the supported grammar must RAISE, never
        be silently judged admissible."""
        with self.assertRaises(ValueError):
            eval_if("contains(github.ref, 'main') ? 1 : 0", _workflow_run_ctx("failure"))

    # -- the fix ---------------------------------------------------------------- #
    def test_live_if_admits_the_known_positive_cancelled_nightly(self):
        """CRITERION 1, seam half. Reinstate `conclusion == 'failure'` in the live
        workflow and this REDs by name."""
        self.assertTrue(
            eval_if(self._live_if(), _workflow_run_ctx(KNOWN_POSITIVE_RUN_CONCLUSION)),
            f"the selection-alarm job `if:` does NOT admit nightly run "
            f"{KNOWN_POSITIVE_RUN_ID} (conclusion=cancelled, event=schedule, "
            f"branch=main), which carried FOUR genuinely-failed jobs. A run-level "
            f"conclusion cannot express 'some job failed' — do not filter on it here; "
            f"scripts/ci_selection_alarm.py makes that decision over the real job list.",
        )

    def test_live_if_admits_every_terminal_run_conclusion(self):
        """The general property behind the known positive: NO run conclusion may be
        excluded here, because the aggregate is lossy for every one of them."""
        for conclusion in ("failure", "cancelled", "timed_out", "success",
                           "startup_failure", "neutral", "action_required", "stale"):
            with self.subTest(conclusion=conclusion):
                self.assertTrue(
                    eval_if(self._live_if(), _workflow_run_ctx(conclusion)),
                    f"conclusion={conclusion!r} is excluded by the job `if:`",
                )

    def test_live_if_still_rejects_the_runs_that_are_not_the_oracle(self):
        """QUANTIFIER DIRECTION. Widening on CONCLUSION must not widen on EVENT or
        BRANCH: only the full-matrix nightly on main is the selection oracle. A PR's CI
        completion carries a diff, so selection narrowing there is legitimate and
        correlating it would manufacture findings. Without this, 'admit everything'
        passes the two tests above."""
        for label, ctx in (
            ("pull_request run on main",
             _workflow_run_ctx("failure", event="pull_request")),
            ("push run on main", _workflow_run_ctx("failure", event="push")),
            ("merge_group run", _workflow_run_ctx("failure", event="merge_group")),
            ("scheduled run off main",
             _workflow_run_ctx("failure", head_branch="sparq-agent/x")),
        ):
            with self.subTest(case=label):
                self.assertFalse(
                    eval_if(self._live_if(), ctx),
                    f"the alarm must not fire for a {label}",
                )

    def test_operator_dispatch_always_runs(self):
        """The manual re-correlation path is unconditional (operator asked)."""
        self.assertTrue(
            eval_if(self._live_if(), {"github": {"event_name": "workflow_dispatch"}})
        )

    def test_the_workflow_passes_the_run_conclusion_to_the_script(self):
        """The conclusion is still NEEDED — for the fail-loud/quiet split — so it must
        reach the script. Structural: read the step's env + the run command."""
        doc = yaml.safe_load(ALARM_YML.read_text())
        steps = doc["jobs"]["alarm"]["steps"]
        correlate = [s for s in steps if "ci_selection_alarm.py" in s.get("run", "")]
        self.assertEqual(len(correlate), 1, "expected exactly one correlate step")
        step = correlate[0]
        self.assertIn(
            "workflow_run.conclusion", str(step.get("env", {})),
            "the step must receive the run conclusion — without it the script cannot "
            "tell an anomalous empty failed-job set from an ordinary cancelled run",
        )
        self.assertIn("--run-conclusion", step["run"])

    def test_suite_is_wired_into_ci(self):
        """Anti-vacuity anchor: this file could otherwise leave CI's reachable set."""
        text = (REPO_ROOT / ".github" / "workflows" / "docs-quality.yml").read_text()
        self.assertIn("scripts/tests/test_ci_selection_alarm.py", text)


if __name__ == "__main__":
    unittest.main(verbosity=2)
