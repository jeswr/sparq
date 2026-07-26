#!/usr/bin/env python3
# [OPUS-5] Readiness VISIBILITY suite — the "can the dispatcher see this work?" contract.
#
# The registry's dispatch.yml clones this repo and runs THIS repo's scripts/ready-issues.py +
# scripts/dispatch-plan.py, so a defect here is a fleet-wide dispatch defect. The suite pins the
# three things that made ready work invisible, each of which was measured live on sparq-org/sparq:
#
#   1. OCCUPANCY ATTRIBUTION — `packages_of` maps "no area: label" to the GLOBAL partition.
#      Applied to a CANDIDATE that is correct fail-closed behaviour (cross-cutting work must
#      serialize). Applied to an OCCUPANT it inverts: "cannot attribute" became "seizes every
#      crate". Nothing in the pipeline puts `area:` labels on PRs — all 60 open sparq PRs had
#      none — so ONE unlabelled PR held __global__ and drove the local frontier to zero.
#   2. status:in-progress-review — absent from BUSY_STATUS and from the reserve branch, so such
#      an issue was neither excluded nor reserving: a double-dispatch on both halves.
#   3. LOCAL/ORCHESTRATOR PARITY — dispatch.yml builds its readiness input from ISSUES ONLY and
#      suppresses issues covered by a linked open PR EXCEPT the in-flight ones, which it keeps
#      so they still reserve their package. The local CLI must preview THAT frontier; when it
#      did not, the two disagreed 0-vs-6 on the same live snapshot. Review round 2 found the
#      first fix still wrong for the dominant live shape (it dropped EVERY linked row, freeing
#      the crate an in-review issue is actively occupying) and the parity claim guarded only by
#      a source-substring assertion, so the regression survived mutation. The parity tests below
#      therefore run the REAL main()/--diagnose, stubbing only the two network calls.
#
# Plus the YAML seam: routing-self-tests.yml listed scripts/ready-issues.py in its `paths:` filter
# but never INVOKED its --self-test, so all of that script's assertions were dead in this repo's
# CI and only ran later inside the registry's dispatch tick (where a failure breaks EVERY target).
from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import re
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SCRIPTS = REPO_ROOT / "scripts"
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "routing-self-tests.yml"


def _load(name: str, filename: str):
    path = SCRIPTS / filename
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None, f"cannot load {path}"
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


ready = _load("ready_issues_under_test", "ready-issues.py")
plan = _load("dispatch_plan_under_test", "dispatch-plan.py")

READY = ["status:ready", "role:impl"]


def iss(number, labels, blockers=0, state="OPEN"):
    return {"number": number, "state": state, "labels": list(labels),
            "open_blockers": blockers}


def pr(number, labels, draft=False):
    return {"number": number, "state": "OPEN", "labels": list(labels),
            "pull_request": {}, "draft": draft}


def numbers(rows):
    return [row["number"] for row in rows]


def quiet(_message):
    pass


class TestUnlabelledOccupantAttribution(unittest.TestCase):
    """Class 1 — an occupant we cannot attribute must reserve NOTHING, not EVERYTHING."""

    def test_unlabelled_pr_does_not_seize_the_global_partition(self):
        # THE live defect: one area-less PR made the entire frontier empty.
        waiting = iss(20, READY + ["priority:P1", "area:sparq-core"])
        unlabelled = pr(70, [])
        self.assertEqual(
            numbers(ready.compute_ready([unlabelled, waiting], conflict_log=quiet)), [20],
            "an area-less PR must not hold __global__ and stall the whole fleet")

    def test_unlabelled_pr_leaves_every_unrelated_crate_dispatchable(self):
        # The blast radius, not just one row: with the bug, ALL of these vanished at once.
        board = [pr(70, [])] + [
            iss(n, READY + ["priority:P1", f"area:crate-{n}"]) for n in (21, 22, 23)]
        self.assertEqual(
            numbers(ready.compute_ready(board, conflict_log=quiet)), [21, 22, 23])

    def test_area_labelled_pr_still_reserves_exactly_its_crate(self):
        # The fix must not become "PRs never reserve": a DECLARED area is still occupancy.
        waiting = iss(20, READY + ["priority:P1", "area:sparq-core"])
        other = iss(21, READY + ["priority:P1", "area:sparq-hdt"])
        board = [pr(70, ["area:sparq-core"]), waiting, other]
        self.assertEqual(numbers(ready.compute_ready(board, conflict_log=quiet)), [21],
                         "a PR labelled area:sparq-core must still hold sparq-core (only)")

    def test_area_less_ISSUE_candidate_still_fails_closed_to_global(self):
        # The asymmetry is deliberate — the CANDIDATE-side global rule is untouched.
        self.assertEqual(ready.packages_of({"role:impl"}), {ready.GLOBAL})
        self.assertEqual(ready.declared_packages({"role:impl"}), set())
        board = [iss(30, READY + ["priority:P0"]),
                 iss(31, READY + ["priority:P1", "area:sparq-core"])]
        self.assertEqual(numbers(ready.compute_ready(board, conflict_log=quiet)), [30],
                         "an area-less READY ISSUE must still serialize against everything")

    def test_unlabelled_in_progress_issue_also_reserves_nothing(self):
        # Same attribution rule on the issue-occupancy path, not just the PR path.
        board = [iss(72, ["status:in-progress"]),
                 iss(20, READY + ["priority:P1", "area:sparq-core"])]
        self.assertEqual(numbers(ready.compute_ready(board, conflict_log=quiet)), [20])


class TestInProgressReviewStatus(unittest.TestCase):
    """Class 2 — status:in-progress-review failed OPEN on BOTH halves."""

    def test_in_progress_review_is_busy(self):
        self.assertIn("status:in-progress-review", ready.BUSY_STATUS)
        self.assertTrue(ready.is_busy({"status:in-progress-review"}))

    def test_in_progress_review_issue_is_never_selected(self):
        board = [iss(10, READY + ["priority:P0", "area:sparq-zk",
                                  "status:in-progress-review"])]
        self.assertEqual(ready.compute_ready(board, conflict_log=quiet), [],
                         "an issue already in review must not be dispatched again")

    def test_in_progress_review_issue_reserves_its_area(self):
        # The other half: dispatch.yml KEEPS these rows in its input precisely because it
        # believes they reserve. If they do not, a second worker takes the same crate.
        board = [iss(10, ["status:in-progress-review", "area:sparq-zk"]),
                 iss(20, READY + ["priority:P1", "area:sparq-zk"])]
        self.assertEqual(ready.compute_ready(board, conflict_log=quiet), [],
                         "an in-review issue must still hold its crate")

    def test_exclusion_reason_names_the_review_status(self):
        self.assertEqual(
            ready.exclusion_reason({"status:ready", "status:in-progress-review"}),
            "busy: status:in-progress-review")


class TestDrainableBacklogVsConcurrencyFrontier(unittest.TestCase):
    """The two must stay distinct — conflating them hides a healthy backlog."""

    BOARD = [
        iss(1, READY + ["priority:P1", "area:sparq-core"]),
        iss(2, READY + ["priority:P2", "area:sparq-core"]),
        iss(3, READY + ["priority:P2", "area:sparq-core"]),
        iss(4, READY + ["priority:P1", "area:sparq-hdt"]),
    ]

    def test_ready_candidates_counts_all_drainable_work(self):
        self.assertEqual(
            sorted(c[1] for c in ready.ready_candidates(self.BOARD)), [1, 2, 3, 4])

    def test_compute_ready_serialises_one_per_package(self):
        self.assertEqual(numbers(ready.compute_ready(self.BOARD, conflict_log=quiet)), [1, 4])

    def test_frontier_is_always_a_subset_of_the_drainable_backlog(self):
        frontier = numbers(ready.compute_ready(self.BOARD, conflict_log=quiet))
        drainable = {c[1] for c in ready.ready_candidates(self.BOARD)}
        self.assertTrue(set(frontier) <= drainable)
        self.assertLessEqual(len(frontier), len(drainable))

    def test_every_dropped_attested_candidate_is_attributable(self):
        # A silent `continue` is how a label-regressed issue leaves the frontier forever.
        board = [
            iss(40, READY + ["priority:P1", "needs:design", "area:a"]),
            iss(41, READY + ["priority:P1", "area:b"], blockers=2),
            iss(42, READY + ["area:c"]),
            iss(43, ["status:ready", "priority:P1", "area:d"]),
            iss(44, ["priority:P1", "role:impl", "area:e"]),  # NOT attested -> must stay quiet
        ]
        lines = []
        ready.ready_candidates(board, log=lines.append)
        got = {int(re.search(r"#(\d+)", line).group(1)): line for line in lines}
        self.assertEqual(sorted(got), [40, 41, 42, 43])
        self.assertIn("gated by needs:design", got[40])
        self.assertIn("2 open blocker(s)", got[41])
        self.assertIn("no single valid priority", got[42])
        self.assertIn("no role:* label", got[43])
        self.assertNotIn(44, got, "a non-attested issue is not a candidate and must not log")


class TestRolelessInvisibility(unittest.TestCase):
    """The class that appears in no plan and no diagnostic — it must be REPORTED."""

    def test_roleless_ready_reports_the_invisible_issues(self):
        board = [
            iss(50, ["status:ready", "priority:P1", "area:a"]),          # roleless
            iss(51, ["status:ready", "area:b"]),                          # roleless, no priority
            iss(52, READY + ["priority:P1", "area:c"]),                   # fine
            iss(53, ["status:ready", "needs:user", "area:d"]),            # gated, not this class
        ]
        self.assertEqual(ready.roleless_ready(board), [50, 51])

    def test_roleless_issues_are_genuinely_absent_from_the_frontier(self):
        board = [iss(50, ["status:ready", "priority:P1", "area:a"])]
        self.assertEqual(ready.compute_ready(board, conflict_log=quiet), [])
        self.assertEqual(ready.roleless_ready(board), [50],
                         "invisible to dispatch, but NOT invisible to the report")

    def test_target_planner_exposes_roleless_ready_to_dispatch_yml(self):
        # dispatch.yml does getattr(dispatch, "roleless_ready", None) and degrades to a
        # "planner has no roleless_ready()" warning when the target predates it.
        self.assertTrue(callable(getattr(plan, "roleless_ready", None)),
                        "dispatch-plan.py must export roleless_ready for the registry planner")
        self.assertEqual(plan.roleless_ready(
            [iss(50, ["status:ready", "priority:P1", "area:a"])]), [50])


def worker_pr(number, issue_number):
    """A pipeline-owned worker PR whose head branch links `issue_number` (dispatch.yml's rule)."""
    return {"number": number, "state": "OPEN", "labels": [], "pull_request": {}, "draft": False,
            "head": {"ref": f"sparq-agent/issue-{issue_number}-fix",
                     "repo": {"full_name": "sparq-org/sparq"}},
            "body": "", "author_association": "NONE"}


class TestLocalOrchestratorParity(unittest.TestCase):
    """The divergence that mattered: the local CLI must preview the dispatched frontier.

    [OPUS-5] The parity claim used to rest on a SOURCE-SUBSTRING assertion plus two test-local
    reimplementations of both sides — so `compute_ready(visible)` -> `compute_ready(issues)` in
    the real main() restored the whole bug with the suite green. Every assertion below now runs
    the REAL main()/--diagnose over a stubbed snapshot, stubbing ONLY the two network calls, and
    compares its printed frontier against a mirror of dispatch.yml's comprehension.
    """

    @staticmethod
    def _orchestrator(issues_and_prs, pulls=()):
        """Mirror of dispatch.yml `ready_input`: ISSUES ONLY; a linked row survives iff in-flight.

        Copied shape (agent-account-registry .github/workflows/dispatch.yml):
            ready_input = [row for row in readiness_input
                           if "status:in-progress" in row["labels"]
                           or "status:in-progress-review" in row["labels"]
                           or (row["number"] not in linked and trusted(...))]
        The `trusted(...)` conjunct is registry-side (needs the issue author + the per-repo
        trusted-bot list) and is out of scope for the local preview; see dispatchable_view's
        docstring for that documented residual divergence.
        """
        linked = ready.linked_issue_numbers(list(pulls), "sparq-org/sparq")
        rows = [it for it in issues_and_prs if "pull_request" not in it]
        rows = [it for it in rows
                if it["number"] not in linked
                or {"status:in-progress", "status:in-progress-review"} & set(it["labels"])]
        return numbers(ready.compute_ready(rows, conflict_log=quiet))

    @staticmethod
    def _run_cli(issues_and_prs, pulls=(), argv=()):
        """Execute the REAL main() end-to-end; only `_fetch`/`_fetch_pulls` are stubbed."""
        real_fetch, real_pulls, real_argv = ready._fetch, ready._fetch_pulls, sys.argv
        ready._fetch = lambda repo, ceiling=10000: [dict(it) for it in issues_and_prs]
        ready._fetch_pulls = lambda repo: [dict(p) for p in pulls]
        sys.argv = ["ready-issues.py", *argv]
        out, err = io.StringIO(), io.StringIO()
        try:
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
                rc = ready.main()
        finally:
            ready._fetch, ready._fetch_pulls, sys.argv = real_fetch, real_pulls, real_argv
        assert rc == 0, f"main() exited {rc}"
        return out.getvalue()

    @classmethod
    def _local(cls, issues_and_prs, pulls=()):
        """The frontier the REAL CLI prints, parsed from its `P<n>  #<num>  [...]` rows."""
        return [int(n) for n in re.findall(
            r"^P\d+\s+#\s*(\d+)\s", cls._run_cli(issues_and_prs, pulls), re.M)]

    # -- the executed counterexample -------------------------------------------------------
    # in-review #100 on sparq-core, covered by its OWN worker PR, + attested #200 on sparq-core.
    COUNTEREXAMPLE_PULLS = (worker_pr(101, 100),)
    COUNTEREXAMPLE_BOARD = (
        iss(100, READY + ["priority:P1", "area:sparq-core", "status:in-progress-review"]),
        iss(200, READY + ["priority:P1", "area:sparq-core"]),
    )

    def test_in_flight_issue_covered_by_its_own_pr_still_reserves_its_crate(self):
        # THE regression. Dropping every linked row frees sparq-core while #100 is actively
        # being worked, and the next tick dispatches a SECOND worker onto the same crate.
        board, pulls = list(self.COUNTEREXAMPLE_BOARD), list(self.COUNTEREXAMPLE_PULLS)
        self.assertEqual(self._orchestrator(board, pulls), [],
                         "sanity: the orchestrator keeps #100 as an occupant, so #200 conflicts")
        self.assertEqual(self._local(board, pulls), [],
                         "local CLI dispatched #200 onto a crate the orchestrator sees as busy")

    def test_cli_frontier_equals_orchestrator_frontier_on_the_counterexample(self):
        board, pulls = list(self.COUNTEREXAMPLE_BOARD), list(self.COUNTEREXAMPLE_PULLS)
        self.assertEqual(self._local(board, pulls), self._orchestrator(board, pulls))

    def test_diagnose_frontier_equals_orchestrator_frontier_on_the_counterexample(self):
        # --diagnose reads its own `visible`; it regressed independently of the default path.
        out = self._run_cli(list(self.COUNTEREXAMPLE_BOARD), list(self.COUNTEREXAMPLE_PULLS),
                            argv=("--diagnose",))
        self.assertIn("concurrency frontier (compute_ready): 0", out)
        self.assertIn("drainable backlog (ready_candidates): 1", out)

    def test_diagnose_does_not_call_an_in_flight_row_merely_pr_covered(self):
        # The bucket must agree with the view: a row dispatchable_view KEEPS is reported as
        # busy, not as suppressed. Otherwise the taxonomy explains a drop that never happened.
        counts, _roleless, _cands, frontier = ready.diagnose(
            list(self.COUNTEREXAMPLE_BOARD), linked={100})
        self.assertEqual(counts.get("busy: status:in-progress-review"), 1)
        self.assertIsNone(counts.get("covered by an open linked PR"))
        self.assertEqual(numbers(frontier), [])

    # -- the previously-covered shapes, now through the real CLI ---------------------------
    def test_unlabelled_prs_do_not_make_the_two_views_disagree(self):
        # The exact live shape: many area-less open PRs + attested issues on distinct crates.
        board = [pr(3803, []), pr(3799, []), pr(3798, [])] + [
            iss(n, READY + ["priority:P3", f"area:crate-{n}"]) for n in (3694, 3756, 3757)]
        self.assertEqual(self._local(board), self._orchestrator(board))
        self.assertEqual(self._local(board), [3694, 3756, 3757])

    def test_linked_pr_suppression_matches_on_both_sides(self):
        # A linked row with NO in-flight status is still suppressed on both sides.
        board = [iss(60, READY + ["priority:P1", "area:sparq-core"]),
                 iss(61, READY + ["priority:P1", "area:sparq-hdt"])]
        pulls = [worker_pr(62, 60)]
        self.assertEqual(self._local(board, pulls), self._orchestrator(board, pulls))
        self.assertEqual(self._local(board, pulls), [61])

    def test_local_cli_applies_linked_pr_suppression_at_all(self):
        # Behavioural, not a substring: deleting the linked_issue_numbers call in main() must
        # let #60 back onto the frontier ahead of the crate it is already covered on.
        board = [iss(60, READY + ["priority:P0", "area:sparq-core"]),
                 iss(61, READY + ["priority:P1", "area:sparq-hdt"])]
        self.assertEqual(self._local(board, [worker_pr(62, 60)]), [61],
                         "main() must suppress issues covered by an open linked PR, as dispatch does")


class TestLinkedIssueDetection(unittest.TestCase):
    """Fork PRs must never suppress an issue (the head branch text is attacker-controlled)."""

    REPO = "sparq-org/sparq"

    def _pull(self, ref, body="", full_name="sparq-org/sparq", association="NONE"):
        return {"head": {"ref": ref, "repo": {"full_name": full_name}},
                "body": body, "author_association": association}

    def test_pipeline_owned_head_links_its_issue(self):
        self.assertEqual(
            ready.linked_issue_numbers([self._pull("sparq-agent/issue-42-fix")], self.REPO), {42})

    def test_fork_head_does_not_link(self):
        self.assertEqual(ready.linked_issue_numbers(
            [self._pull("sparq-agent/issue-42-fix", full_name="attacker/sparq")], self.REPO),
            set(), "a fork PR must never suppress an issue")

    def test_closing_keyword_needs_a_trusted_association(self):
        untrusted = self._pull("patch-1", body="Closes #42", association="NONE")
        trusted = self._pull("patch-1", body="Closes #42", association="MEMBER")
        self.assertEqual(ready.linked_issue_numbers([untrusted], self.REPO), set())
        self.assertEqual(ready.linked_issue_numbers([trusted], self.REPO), {42})


class TestDiagnoseTaxonomy(unittest.TestCase):
    """--diagnose must account for EVERY open issue, so no class can hide."""

    def test_buckets_partition_the_open_backlog(self):
        board = [
            iss(1, READY + ["priority:P1", "area:a"]),
            iss(2, ["status:untriaged"]),
            iss(3, []),
            iss(4, READY + ["priority:P1", "needs:ec2", "area:b"]),
            iss(5, READY + ["priority:P1", "area:c"]),
            pr(70, []),
            iss(6, READY + ["priority:P1", "area:d"], state="CLOSED"),
        ]
        counts, roleless, cands, frontier = ready.diagnose(board, linked={5})
        self.assertEqual(sum(counts.values()), 5, "one bucket per OPEN issue, PRs/closed excluded")
        self.assertEqual(counts["ENUMERABLE"], 1)
        self.assertEqual(counts["covered by an open linked PR"], 1)
        self.assertEqual(counts["gated by needs:ec2"], 1)
        self.assertEqual(counts["no status:ready attestation"], 2)
        self.assertEqual(roleless, [])
        self.assertEqual([c[1] for c in cands], [1])
        self.assertEqual(numbers(frontier), [1])


class TestRoutingSelfTestWorkflowWiring(unittest.TestCase):
    """The YAML seam — a self-test that no workflow INVOKES is not a gate.

    routing-self-tests.yml listed scripts/ready-issues.py under `paths:` (so it looked covered)
    while its `run:` block never executed that script's --self-test. Deleting either the paths
    entry or the invocation must red THIS test.
    """

    SOURCE = WORKFLOW.read_text(encoding="utf-8")
    INVOKED = ("scripts/routing-validate.py", "scripts/route-resolve.py",
               "scripts/dispatch-plan.py", "scripts/ready-issues.py")

    def test_every_self_tested_script_is_actually_invoked(self):
        run_block = self.SOURCE[self.SOURCE.index("Validate routing schema"):]
        for script in self.INVOKED:
            self.assertIn(f"python3 {script} --self-test", run_block,
                          f"{script} --self-test is never RUN — its assertions are dead in CI")

    def test_ready_issues_self_test_is_invoked_not_merely_path_filtered(self):
        # The precise regression: present in `paths:`, absent from `run:`.
        run_block = self.SOURCE[self.SOURCE.index("Validate routing schema"):]
        self.assertIn("python3 scripts/ready-issues.py --self-test", run_block)

    def test_this_test_file_is_itself_a_path_trigger(self):
        # Scoped to the `paths:` section ON PURPOSE: a whole-file substring search passes for
        # the WRONG reason, because this filename also appears in the run: block, so deleting
        # both paths entries left the old assertion green. Both trigger blocks
        # (pull_request + push) must list it, hence the exact count of 2.
        self.assertEqual(
            self._paths_section().count('"scripts/tests/test_readiness_visibility.py"'), 2,
            "edits to this suite must re-run the gate that executes it, on PR *and* push")

    def test_this_suite_is_invoked_by_the_workflow(self):
        run_block = self.SOURCE[self.SOURCE.index("Validate routing schema"):]
        self.assertIn("python3 scripts/tests/test_readiness_visibility.py", run_block)

    def _paths_section(self):
        return self.SOURCE[:self.SOURCE.index("permissions:")]

    def test_every_invoked_script_is_a_path_trigger(self):
        paths_section = self._paths_section()
        for script in self.INVOKED:
            # Exactly 2: the pull_request filter AND the push filter. Dropping either one
            # lets the script change without re-running the gate on that trigger.
            self.assertEqual(paths_section.count(f'"{script}"'), 2,
                             f"{script} must be a path trigger on BOTH pull_request and push")

    def test_gate_is_not_declared_advisory(self):
        # [OPUS-5] Guards the rule that is LIVE. ci-summary's discovery changed on
        # 2026-07-25 (#3773): a check is non-gating iff it is EXPLICITLY DECLARED in
        # .github/advisory-registry.json, keyed on workflow file + job id. The old
        # `\b(advisory|informational)\b` NAME rule is gone — it had silently neutralised
        # four real gates, two of them documented in-repo as HARD — so a rename can no
        # longer demote anything (declarations bind to job identity and
        # check-advisory-registry.py C4 reds on a rename). Asserting on the job NAME would
        # therefore guard a rule that no longer exists; the real demotion path is a
        # registry entry, so that is what is asserted.
        registry = json.loads(
            (REPO_ROOT / ".github" / "advisory-registry.json").read_text(encoding="utf-8"))
        declared = {(e.get("workflow"), e.get("job_id"))
                    for e in registry.get("jobs", {}).values()}
        self.assertNotIn(("routing-self-tests.yml", "validate"), declared,
                         "declaring this job advisory stops ci-summary gating on it")

    def test_merge_group_trigger_present(self):
        # merge_group cannot use a paths filter; without the trigger the queue ref
        # never exposes this gating check.
        self.assertRegex(self.SOURCE, r"(?m)^  merge_group:")


if __name__ == "__main__":
    unittest.main(verbosity=2)
