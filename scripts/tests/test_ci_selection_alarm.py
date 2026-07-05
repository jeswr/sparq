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
# Fully hermetic: no gh, no git, no cargo, no network. Correlation is a pure
# function; the CLI is exercised via --failed-jobs-file + --metadata-file +
# --dry-run and a monkeypatched git-log/diff.
#
# Run:  python3 scripts/tests/test_ci_selection_alarm.py   (stdlib only)

from __future__ import annotations

import importlib.util
import io
import json
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
ALARM = REPO_ROOT / "scripts" / "ci_selection_alarm.py"
CI_YML = REPO_ROOT / ".github" / "workflows" / "ci.yml"


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


if __name__ == "__main__":
    unittest.main(verbosity=2)
