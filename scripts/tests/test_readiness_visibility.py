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
import subprocess
import sys
import tempfile
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
        # [OPUS-5] sparq#4336: the keys were synthetic `area:crate-21/22/23`. Those share the head
        # segment `crate`, and containment-aware conflict() now (correctly) treats a shared head
        # as one partition, so the fixture no longer modelled "three UNRELATED crates" at all.
        # Switched to three real, disjoint workspace crates — the assertion is unchanged in force.
        board = [pr(70, [])] + [
            iss(n, READY + ["priority:P1", f"area:{a}"])
            for n, a in ((21, "sparq-core"), (22, "sparq-hdt"), (23, "sparq-geo"))]
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


class TestSubCrateContainment(unittest.TestCase):
    """sparq#4336 — the UNDER-serialisation defect: a region key never overlapped its crate.

    `conflict()` compared partition keys by exact-string set overlap, so `area:sparq-server-http`
    did not overlap `area:sparq-server`. Both entered the frontier in one tick, and a sub-crate
    issue entered despite an open PR holding the parent crate: two workers, one crate, no lock.
    57.1% of same-crate 24h PR pairs share a file (research/crate-region-parallelism.md §4), and a
    semantic collision (both compile, both pass, together broken) is invisible to git.

    Every test here goes END-TO-END through compute_ready() where it can, so deleting
    `keys_conflict`, flattening `partition_path` to the identity, or reverting `conflict()` to
    `pkgs & blockers.keys()` reds this class.
    """

    # The four live labels that are NOT workspace crates -> they are regions inside one.
    CONTAINED = {"sparq-server-http": "sparq-server",
                 "sparq-core-nt-dict": "sparq-core",
                 "sparq-core-store": "sparq-core",
                 "sparq-engine-exec": "sparq-engine"}
    # The fifth label from the issue's table. It IS a real workspace crate at origin/main, so the
    # table was already stale — see test_conformance_floors_is_a_real_crate_and_must_stay_split.
    NOT_CONTAINED = "sparq-conformance-floors"

    def test_every_live_sub_crate_label_conflicts_with_its_parent_crate(self):
        for child, parent in self.CONTAINED.items():
            with self.subTest(child=child):
                self.assertTrue(ready.keys_conflict(child, parent),
                                f"{child} names a region inside {parent}")
                self.assertTrue(ready.keys_conflict(parent, child), "conflict must be symmetric")
                self.assertEqual(ready.partition_path(child), (parent,))

    def test_parent_crate_issue_blocks_a_sub_crate_issue_in_the_same_tick(self):
        # Ordering A: the PARENT is selected first (lower number wins the priority tie).
        board = [iss(1, READY + ["priority:P1", "area:sparq-server"]),
                 iss(2, READY + ["priority:P1", "area:sparq-server-http"])]
        self.assertEqual(numbers(ready.compute_ready(board, conflict_log=quiet)), [1],
                         "area:sparq-server-http must not enter alongside area:sparq-server")

    def test_sub_crate_issue_blocks_a_parent_crate_issue_in_the_same_tick(self):
        # Ordering B: the CHILD is selected first. Both orderings, per the acceptance criteria.
        board = [iss(1, READY + ["priority:P1", "area:sparq-server-http"]),
                 iss(2, READY + ["priority:P1", "area:sparq-server"])]
        self.assertEqual(numbers(ready.compute_ready(board, conflict_log=quiet)), [1],
                         "area:sparq-server must not enter alongside area:sparq-server-http")

    def test_open_pr_holding_the_parent_crate_blocks_a_sub_crate_issue(self):
        # The OCCUPANCY half of the defect, demonstrated live in the issue.
        board = [pr(70, ["area:sparq-server"]),
                 iss(20, READY + ["priority:P1", "area:sparq-server-http"])]
        self.assertEqual(ready.compute_ready(board, conflict_log=quiet), [],
                         "an open PR on sparq-server must hold the whole crate")

    def test_open_pr_holding_a_sub_crate_blocks_a_parent_crate_issue(self):
        board = [pr(70, ["area:sparq-core-store"]),
                 iss(20, READY + ["priority:P1", "area:sparq-core"])]
        self.assertEqual(ready.compute_ready(board, conflict_log=quiet), [],
                         "an open PR on a region must hold its containing crate")

    def test_sibling_sub_crate_keys_of_one_parent_conflict_with_each_other(self):
        # A sibling hole is the SAME defect one level down: sparq-core-store and
        # sparq-core-nt-dict are both files in crates/sparq-core. Neither string is a prefix of
        # the other, so a naive prefix-on-the-raw-key rule would pass the parent tests and still
        # put two workers in sparq-core.
        self.assertTrue(ready.keys_conflict("sparq-core-store", "sparq-core-nt-dict"))
        board = [iss(1, READY + ["priority:P1", "area:sparq-core-store"]),
                 iss(2, READY + ["priority:P1", "area:sparq-core-nt-dict"])]
        self.assertEqual(numbers(ready.compute_ready(board, conflict_log=quiet)), [1])

    def test_unrelated_crates_still_do_not_conflict(self):
        # The fix must not collapse everything into one lock and destroy the frontier.
        for a, b in (("sparq-core", "sparq-engine"), ("sparq-server", "sparq-hdt"),
                     ("sparq-zk", "sparq-mpc"), ("site", "bench")):
            with self.subTest(pair=(a, b)):
                self.assertFalse(ready.keys_conflict(a, b))
        board = [iss(n, READY + ["priority:P1", f"area:{a}"]) for n, a in
                 ((1, "sparq-core"), (2, "sparq-engine"), (3, "sparq-hdt"), (4, "site"))]
        self.assertEqual(numbers(ready.compute_ready(board, conflict_log=quiet)), [1, 2, 3, 4])

    def test_real_sibling_crates_sharing_a_name_prefix_stay_independent(self):
        # `sparq-engine-serialize` is a REAL crate directory, so it is NOT a region inside
        # `sparq-engine` even though the strings nest. Only the workspace tree can tell these
        # apart from `sparq-engine-exec`; a hand-written containment table cannot.
        for crate in ("sparq-engine-serialize", "sparq-engine-service", "sparq-reason-dl",
                      "sparq-lws-core", "sparq-zk-compose"):
            with self.subTest(crate=crate):
                self.assertEqual(ready.partition_path(crate), (crate,))
        self.assertFalse(ready.keys_conflict("sparq-engine", "sparq-engine-serialize"))
        self.assertFalse(ready.keys_conflict("sparq-engine-exec", "sparq-engine-serialize"))
        self.assertTrue(ready.keys_conflict("sparq-engine", "sparq-engine-exec"),
                        "the non-crate sibling must still collapse into the crate")

    def test_conformance_floors_is_a_real_crate_and_must_stay_split(self):
        # Issue #4336's table lists area:sparq-conformance-floors as a region inside
        # crates/sparq-conformance. It is NOT: crates/sparq-conformance-floors is a workspace
        # member at origin/main. The table was stale before it was written down, which is the
        # whole argument for deriving containment from the tree instead of listing it.
        self.assertTrue((REPO_ROOT / "crates" / self.NOT_CONTAINED / "Cargo.toml").is_file())
        self.assertEqual(ready.partition_path(self.NOT_CONTAINED), (self.NOT_CONTAINED,))
        self.assertFalse(ready.keys_conflict("sparq-conformance", self.NOT_CONTAINED))


class TestTotalPartitionMapping(unittest.TestCase):
    """The durable half: an UNKNOWN key must fail SAFE with no code change.

    Under-serialisation corrupts; over-serialisation only delays. So the map is total and every
    unrecognised key resolves UPWARD to the coarsest thing that could contain it.
    """

    def test_newly_invented_sub_crate_key_resolves_to_its_crate(self):
        # Nobody registers this label anywhere. It must still be caught by the lock.
        for invented in ("sparq-server-zzz", "sparq-core-brand-new-region",
                         "sparq-engine-exec-v2", "sparq-hdt-writer"):
            with self.subTest(key=invented):
                parent = ready.partition_path(invented)[0]
                self.assertIn(parent, ready.workspace_roots())
                self.assertTrue(invented.startswith(parent + "-"))
        board = [pr(70, ["area:sparq-server"]),
                 iss(20, READY + ["priority:P1", "area:sparq-server-zzz"])]
        self.assertEqual(ready.compute_ready(board, conflict_log=quiet), [],
                         "an invented region key must fail SAFE into its crate's lock")

    def test_unknown_key_under_an_unknown_parent_still_resolves_upward(self):
        # `upstream` is not a directory, so the workspace cannot confirm it — the key's own head
        # segment is then the coarsest partition it names, and honouring it OVER-reserves.
        self.assertEqual(ready.partition_path("upstream-noir"), ("upstream",))
        self.assertTrue(ready.keys_conflict("upstream", "upstream-noir"))
        self.assertTrue(ready.keys_conflict("upstream-noir", "upstream-oxigraph"))

    def test_single_segment_unknown_key_keeps_its_own_partition(self):
        # It names nothing narrower than itself, so it cannot be under-serialising, and routing it
        # to __global__ would be a fleet stall, not a fix: MEASURED on the live 2026-07-26
        # snapshot, a __global__ terminal fallback took the frontier from 6 to 0 because open PR
        # #4238 carries `area:deps` and `deps` is not a directory.
        for key in ("upstream", "deps", "workspace", "release", "accuracy", "frobnicate"):
            with self.subTest(key=key):
                self.assertEqual(ready.partition_path(key), (key,))
        board = [iss(1, READY + ["priority:P1", "area:deps"]),
                 iss(2, READY + ["priority:P1", "area:sparq-core"])]
        self.assertEqual(numbers(ready.compute_ready(board, conflict_log=quiet)), [1, 2])

    def test_degenerate_key_fails_all_the_way_closed_to_global(self):
        # The terminal of the resolution chain. A key with no head segment to latch onto reserves
        # EVERYTHING rather than nothing.
        for degenerate in ("", "-", "---"):
            with self.subTest(key=degenerate):
                self.assertEqual(ready.partition_path(degenerate), ())
                self.assertTrue(ready.keys_conflict(degenerate, "sparq-core"))
                self.assertTrue(ready.keys_conflict(degenerate, ready.GLOBAL))

    def test_global_conflicts_with_every_key_and_the_mapping_is_total(self):
        self.assertEqual(ready.partition_path(ready.GLOBAL), ())
        for key in ("sparq-core", "sparq-server-http", "site-papers", "deps", "zz-top"):
            with self.subTest(key=key):
                self.assertTrue(ready.keys_conflict(ready.GLOBAL, key))
                self.assertTrue(ready.keys_conflict(key, ready.GLOBAL))
                self.assertIsInstance(ready.partition_path(key), tuple)

    def test_containment_is_reflexive_and_symmetric(self):
        for key in ("sparq-core", "sparq-core-store", ready.GLOBAL, "deps"):
            with self.subTest(key=key):
                self.assertTrue(ready.keys_conflict(key, key))
        self.assertEqual(ready.keys_conflict("sparq-core", "sparq-core-store"),
                         ready.keys_conflict("sparq-core-store", "sparq-core"))


class TestWorkspaceDerivedRoots(unittest.TestCase):
    """Roots are READ FROM THE TREE. A table would already be stale (see conformance-floors)."""

    def test_roots_come_from_the_real_repository_tree(self):
        roots = ready.workspace_roots()
        for crate in (p.name for p in (REPO_ROOT / "crates").iterdir() if p.is_dir()):
            self.assertIn(crate, roots, f"crate {crate} must be a recognised partition root")
        for top in ("site", "bench", "scripts", "research", "crates"):
            self.assertIn(top, roots)
        self.assertNotIn(".github", roots, "dot-directories are not area: key names")

    def test_a_crate_added_with_no_code_change_is_recognised_immediately(self):
        # The anti-staleness property, exercised against a synthetic tree rather than asserted.
        with tempfile.TemporaryDirectory() as tmp:
            base = Path(tmp)
            (base / "crates" / "sparq-brandnew").mkdir(parents=True)
            (base / "crates" / "sparq-brandnew-region").mkdir(parents=True)
            (base / "site").mkdir()
            roots = ready.workspace_roots(str(base))
            self.assertEqual(roots, {"crates", "site", "sparq-brandnew", "sparq-brandnew-region"})
            # present in the tree -> its own partition; absent -> collapses into its parent
            self.assertEqual(ready.partition_path("sparq-brandnew-region", roots),
                             ("sparq-brandnew-region",))
            self.assertEqual(ready.partition_path("sparq-brandnew-unlisted", roots),
                             ("sparq-brandnew",))
            self.assertFalse(ready.keys_conflict("sparq-brandnew", "sparq-brandnew-region", roots))
            self.assertTrue(ready.keys_conflict("sparq-brandnew", "sparq-brandnew-unlisted", roots))

    def test_an_unreadable_tree_over_reserves_rather_than_under_reserves(self):
        # If the scan finds nothing, every key falls back to its head segment. That collapses all
        # `sparq-*` keys onto `sparq` — a fleet-wide slowdown, never a double dispatch.
        roots = ready.workspace_roots("/nonexistent/sparq-checkout")
        self.assertEqual(roots, set())
        self.assertEqual(ready.partition_path("sparq-core", roots), ("sparq",))
        self.assertTrue(ready.keys_conflict("sparq-core", "sparq-engine", roots),
                        "with no tree to read, unrelated crates must OVER-reserve")

    def test_dump_partitions_exports_the_mapping_for_the_registry_parity_fixture(self):
        # The registry's dispatch.yml mirrors this key space in `busy_packages_of_pulls`; the two
        # must agree or the fleet double-dispatches. This is the machine-readable contract.
        out = subprocess.run(
            [sys.executable, str(SCRIPTS / "ready-issues.py"), "--dump-partitions",
             "sparq-server-http", "sparq-engine-serialize", "deps"],
            capture_output=True, text=True, check=True).stdout
        dumped = json.loads(out)
        self.assertIn("sparq-core", dumped["roots"])
        self.assertEqual(dumped["resolved"]["sparq-server-http"], ["sparq-server"])
        self.assertEqual(dumped["resolved"]["sparq-engine-serialize"], ["sparq-engine-serialize"])
        self.assertEqual(dumped["resolved"]["deps"], ["deps"])


class TestConflictAttribution(unittest.TestCase):
    """A conflict line must name the RAW held label and the COARSEST holder, deterministically."""

    def test_conflict_line_names_the_raw_parent_label_not_the_resolved_path(self):
        lines = []
        board = [pr(70, ["area:sparq-server"]),
                 iss(20, READY + ["priority:P1", "area:sparq-server-http"])]
        ready.compute_ready(board, conflict_log=lines.append)
        self.assertEqual(lines, ["conflict #20: area sparq-server held by pr#70"])

    def test_conflict_line_names_the_raw_child_label_when_the_child_holds(self):
        lines = []
        board = [pr(70, ["area:sparq-core-store"]),
                 iss(20, READY + ["priority:P1", "area:sparq-core"])]
        ready.compute_ready(board, conflict_log=lines.append)
        self.assertEqual(lines, ["conflict #20: area sparq-core-store held by pr#70"])

    def test_the_coarsest_holder_is_reported_when_several_conflict(self):
        # __global__ (path length 0) outranks any named area; named areas tie-break alphabetically.
        lines = []
        board = [iss(70, ["status:in-progress"] + ["area:sparq-server"]),
                 iss(71, ["status:in-progress"] + ["area:sparq-core"]),
                 iss(30, READY + ["priority:P0"])]
        ready.compute_ready(board, conflict_log=lines.append)
        self.assertEqual(lines, ["conflict #30: area sparq-core held by issue#71"])


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
        # [OPUS-5] sparq#4336: real disjoint crates, for the reason recorded on
        # test_unlabelled_pr_leaves_every_unrelated_crate_dispatchable.
        board = [pr(3803, []), pr(3799, []), pr(3798, [])] + [
            iss(n, READY + ["priority:P3", f"area:{a}"])
            for n, a in ((3694, "sparq-core"), (3756, "sparq-hdt"), (3757, "sparq-geo"))]
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


class TestUnitReservation(unittest.TestCase):
    """A PR and the issues it closes are ONE unit: they reserve the UNION, exactly ONCE.

    [OPUS-5] MEASURED on the live sparq snapshot (2026-07-27, 1473 open issues / 123 open PRs):
    65 occupying PRs + 46 in-flight issues produced 158 reservations over 49 distinct partition
    keys — 20 of them a duplicate of a key the unit's other half already held.

    Two facts pin the shape of the rule, and both are measurements rather than opinions:

    * DEDUP IS NOT A FRONTIER LEVER. `conflict()` tests membership in the SET of held keys, so a
      second occupant on an already-held key changes nothing. 158 -> 138 reservations left the held
      set at 49 keys and the live frontier at 3 -> 3. Every test below therefore asserts on the
      RESERVATION STRUCTURE and on end-to-end dispatch decisions, not on a frontier count that the
      dedup cannot move.
    * DROPPING THE ISSUE HALF UNDER-SERIALISES. Over the 94 open PRs with an open linked source
      issue: 31 pairs PR ⊋ issue, 18 identical, 6 source-with-no-`area:`, but 13 PR ⊊ issue and 26
      INCOMPARABLE. So in 39/94 = 41% of pairs the PR's key set is NOT a superset and dropping the
      issue's reservation frees a key the unit really occupies — two workers in one crate, the
      corrupting direction. Hence union, never drop.
    """

    @staticmethod
    def keys(units):
        return set().union(set(), *[areas for areas, _artifact in units])

    @staticmethod
    def by_pr(units):
        return {artifact["number"]: sorted(areas) for areas, artifact in units}

    # -- the headline guard, asserted through compute_ready AND through the real CLI ---------
    def test_pr_and_source_issue_reserve_the_union_exactly_once(self):
        # THE titled guard. The pair declares {sparq-core} (PR) and {sparq-hdt} (issue): the unit
        # must hold BOTH, under ONE artifact, and neither key may be reserved twice.
        source = iss(100, ["status:in-progress-review", "area:sparq-hdt"])
        worker = pr(101, ["area:sparq-core"])
        links = {101: {100}}
        units = ready.unit_reservations([worker, source], links)
        self.assertEqual(self.by_pr(units), {101: ["sparq-core", "sparq-hdt"]},
                         "the pair must reserve the union under the PR, as ONE unit")
        self.assertEqual(len(units), 1, "the source issue must not reserve a second time")
        # ...and the union is actually enforced: BOTH crates are held against fresh candidates.
        board = [worker, source,
                 iss(200, READY + ["priority:P1", "area:sparq-core"]),
                 iss(201, READY + ["priority:P1", "area:sparq-hdt"]),
                 iss(202, READY + ["priority:P1", "area:sparq-geo"])]
        self.assertEqual(
            numbers(ready.compute_ready(board, conflict_log=quiet, source_links=links)), [202],
            "both halves of the unit's union must block, and unrelated crates must not")

    def test_the_pair_is_ONE_occupant_not_two(self):
        # The dedup itself, stated so that removing `consumed` (letting the source issue reserve
        # again on its own) reds even though the HELD KEY SET is unchanged by that mutation.
        source = iss(100, ["status:in-progress-review", "area:sparq-core"])
        units = ready.unit_reservations([pr(101, ["area:sparq-core"]), source], {101: {100}})
        self.assertEqual([artifact["number"] for _areas, artifact in units], [101])
        self.assertEqual(sum(len(areas) for areas, _ in units), 1,
                         "one unit of work must produce exactly one reservation of sparq-core")

    def test_conflict_attribution_names_the_PR_of_a_paired_unit(self):
        # Attribution is the one thing dedup DOES change, and it changes it for the better: the
        # PR is the advanceable artifact (it can merge or close), the source issue is not.
        board = [iss(100, ["status:in-progress-review", "area:sparq-core"]),
                 pr(101, ["area:sparq-core"]),
                 iss(200, READY + ["priority:P1", "area:sparq-core"])]
        logs = []
        ready.compute_ready(board, conflict_log=logs.append, source_links={101: {100}})
        self.assertEqual(logs, ["conflict #200: area sparq-core held by pr#101"])

    # -- the four obligations that make the rule impossible to weaken into a DROP ------------
    def test_source_issue_broader_than_its_pr_does_not_lose_the_extra_areas(self):
        # 39/94 live pairs are non-superset. Narrowing the unit to the PR's own set here would
        # free sparq-hdt and sparq-geo while a worker is mid-flight on them.
        source = iss(100, ["status:in-progress-review", "area:sparq-hdt", "area:sparq-geo"])
        links = {101: {100}}
        self.assertEqual(self.by_pr(ready.unit_reservations([pr(101, ["area:sparq-hdt"]), source],
                                                            links)),
                         {101: ["sparq-geo", "sparq-hdt"]})
        board = [pr(101, ["area:sparq-hdt"]), source,
                 iss(200, READY + ["priority:P1", "area:sparq-geo"])]
        self.assertEqual(numbers(ready.compute_ready(board, conflict_log=quiet,
                                                     source_links=links)), [],
                         "an area the ISSUE half alone declares must still be held by the unit")

    def test_source_issue_with_no_area_keeps_the_units_reservation_intact(self):
        # sparq#4336 / PR #4360: the source issue carried NO `area:` at all. Folding it into the
        # unit must not shrink what the unit holds — the PR's own key must survive untouched.
        source = iss(100, ["status:in-progress-review"])
        links = {101: {100}}
        self.assertEqual(self.by_pr(ready.unit_reservations([pr(101, ["area:sparq-core"]), source],
                                                            links)),
                         {101: ["sparq-core"]})
        board = [pr(101, ["area:sparq-core"]), source,
                 iss(200, READY + ["priority:P1", "area:sparq-core"]),
                 iss(201, READY + ["priority:P1", "area:sparq-hdt"])]
        self.assertEqual(numbers(ready.compute_ready(board, conflict_log=quiet,
                                                     source_links=links)), [201])

    def test_a_units_reservation_is_a_SUPERSET_of_every_members_own(self):
        # MONOTONICITY — the property that makes under-serialisation structurally impossible,
        # whatever the two halves declare. Exhaustive over every containment direction, plus the
        # `{GLOBAL}` member: a half whose own rule yields the serializing partition must keep it.
        cases = [
            (["area:sparq-core"], ["area:sparq-core"]),               # identical
            (["area:sparq-core", "area:sparq-hdt"], ["area:sparq-core"]),   # PR superset
            (["area:sparq-core"], ["area:sparq-core", "area:sparq-hdt"]),   # issue superset
            (["area:sparq-core"], ["area:sparq-hdt"]),                # incomparable
            ([], ["area:sparq-hdt"]),                                 # PR declares nothing
            (["area:sparq-core"], []),                                # issue declares nothing
            ([f"area:{ready.GLOBAL}"], ["area:sparq-core"]),          # a GLOBAL-holding half
            (["area:sparq-core"], [f"area:{ready.GLOBAL}"]),
        ]
        for pr_labels, issue_labels in cases:
            with self.subTest(pr=pr_labels, issue=issue_labels):
                worker = pr(101, pr_labels)
                source = iss(100, ["status:in-progress-review"] + issue_labels)
                units = ready.unit_reservations([worker, source], {101: {100}})
                held = self.keys(units)
                for member in (worker, source):
                    self.assertLessEqual(
                        ready._own_reservation(member), held,
                        f"unit dropped a key member #{member['number']} reserves on its own")

    def test_a_GLOBAL_holding_half_still_serializes_the_whole_board(self):
        # The fail-closed global must survive the fold END-TO-END, not merely in the key set.
        board = [pr(101, [f"area:{ready.GLOBAL}"]),
                 iss(100, ["status:in-progress-review", "area:sparq-core"]),
                 iss(200, READY + ["priority:P1", "area:sparq-geo"])]
        self.assertEqual(numbers(ready.compute_ready(board, conflict_log=quiet,
                                                     source_links={101: {100}})), [],
                         "a unit holding __global__ must still block every unrelated crate")

    # -- the units that must NOT be folded together -------------------------------------------
    def test_two_unrelated_units_still_reserve_separately(self):
        board = [pr(101, ["area:sparq-core"]), iss(100, ["status:in-progress-review",
                                                         "area:sparq-core"]),
                 pr(301, ["area:sparq-hdt"]), iss(300, ["status:in-progress-review",
                                                        "area:sparq-hdt"])]
        units = ready.unit_reservations(board, {101: {100}, 301: {300}})
        self.assertEqual(self.by_pr(units), {101: ["sparq-core"], 301: ["sparq-hdt"]},
                         "two genuinely distinct units must remain two occupants")

    def test_an_issue_with_no_linked_pr_is_unaffected(self):
        lone = iss(100, ["status:in-progress-review", "area:sparq-core"])
        other = pr(301, ["area:sparq-hdt"])
        units = ready.unit_reservations([lone, other], {301: set()})
        self.assertEqual(sorted(a["number"] for _k, a in units), [100, 301])
        self.assertEqual(self.keys(units), {"sparq-core", "sparq-hdt"})
        # and it still blocks its own crate on the frontier
        board = [lone, iss(200, READY + ["priority:P1", "area:sparq-core"])]
        self.assertEqual(numbers(ready.compute_ready(board, conflict_log=quiet,
                                                     source_links={301: set()})), [])

    def test_a_fork_PR_never_folds_an_issue_into_its_unit(self):
        # source_issue_links is the ONLY linkage rule; a fork head is attacker-controlled text.
        fork = {"number": 101, "state": "OPEN", "labels": [], "pull_request": {}, "draft": False,
                "head": {"ref": "sparq-agent/issue-100-fix", "repo": {"full_name": "attacker/x"}},
                "body": "Closes #100", "author_association": "NONE"}
        self.assertEqual(ready.source_issue_links([fork], "sparq-org/sparq"), {})

    # -- the no-op contract the REGISTRY depends on -------------------------------------------
    def test_unit_reservations_without_links_is_identical_to_the_legacy_loop(self):
        # dispatch.yml calls `compute_ready(ready_input)` with no source_links. If that call is not
        # byte-identical to the pre-refactor loop, the two repositories cannot merge in either
        # order. Reproduce the LEGACY loop here and compare pairs AND their order (attribution is
        # order-sensitive: `blockers[area][0]` names the first reserver).
        board = [pr(101, ["area:sparq-core"]),
                 iss(100, ["status:in-progress-review", "area:sparq-core"]),
                 iss(102, ["status:in-progress", "area:sparq-hdt"]),
                 pr(103, ["area:sparq-geo", "needs:user"]),      # parked -> reserves nothing
                 pr(104, []),                                    # unattributable -> nothing
                 iss(105, READY + ["priority:P1", "area:sparq-zk"]),
                 iss(106, ["status:in-progress-review", "review:needs-user", "area:sparq-mpc"])]
        legacy = []
        for row in board:
            if str(row.get("state", "OPEN")).upper() != "OPEN" or not ready.occupies_area(row):
                continue
            labels = ready.labels_of(row)
            if "pull_request" in row or labels & ready.IN_FLIGHT_STATUS:
                areas = ready._reserving_packages(labels)
                if areas:
                    legacy.append((areas, row["number"]))
        self.assertEqual([(areas, artifact["number"])
                          for areas, artifact in ready.unit_reservations(board)], legacy)
        self.assertEqual(legacy, [({"sparq-core"}, 101), ({"sparq-core"}, 100),
                                  ({"sparq-hdt"}, 102)],
                         "sanity: the legacy expectation itself must be the live rule, not a stub")

    def test_compute_ready_without_source_links_is_unchanged(self):
        board = [pr(101, ["area:sparq-core"]),
                 iss(100, ["status:in-progress-review", "area:sparq-core"]),
                 iss(200, READY + ["priority:P1", "area:sparq-core"]),
                 iss(201, READY + ["priority:P2", "area:sparq-hdt"])]
        logs = []
        self.assertEqual(numbers(ready.compute_ready(board, conflict_log=logs.append)), [201])
        self.assertEqual(logs, ["conflict #200: area sparq-core held by pr#101"],
                         "omitting source_links must leave attribution exactly as it was")

    # -- the CALL SITE, not just the helper ----------------------------------------------------
    def test_the_real_CLI_folds_the_pair_into_one_unit(self):
        # [OPUS-5] The titled leg runs in main(): `compute_ready(visible, source_links=...)`.
        # Dropping that keyword argument at the call site leaves every helper test above green
        # while the CLI reverts to two independent reservations, so assert on main()'s own output.
        board = [iss(100, ["status:in-progress-review", "area:sparq-hdt"]),
                 iss(200, READY + ["priority:P1", "area:sparq-core"])]
        pulls = [dict(worker_pr(101, 100), labels=[{"name": "area:sparq-core"}])]
        prs = [dict(p, labels=[lb["name"] for lb in p["labels"]]) for p in pulls]
        out = TestLocalOrchestratorParity._run_cli(board + prs, pulls, argv=("--diagnose",))
        self.assertIn("unit occupancy: 1 unit(s), 2 reservation(s) over 2 partition key(s)", out,
                      "main() must pass source_links so the PR+issue pair is ONE unit")
        self.assertIn("concurrency frontier (compute_ready): 0", out,
                      "and the union (sparq-core from the PR) must actually hold #200 back")


class TestOrchestratorOccupancyGap(unittest.TestCase):
    """The measured consequence of dispatch.yml stripping PR rows from its readiness input.

    [OPUS-5] `dispatch.yml` builds `readiness_input` from
    `[issue for issue in snapshot("issues", index) if "pull_request" not in issue]`, so PLAN
    reserves the ISSUE half of every unit and never the PR half. MEASURED on the live snapshot
    (2026-07-27): the PR-aware view holds 49 partition keys and emits a frontier of 3; the
    issue-only view holds 37 and emits 9 — and 7 of those 9 rows land in a key an open PR already
    holds. Registry CLAIM re-derives busy areas from the pulls snapshot and drops them, so they are
    not double-dispatched; but they are dropped AFTER compute_ready committed the frontier, so each
    burned a partition with no backfill (the registry's own issue #113 shape).

    sparq cannot close that from here — PLAN's input is built inside dispatch.yml, which the
    registry owns. What sparq owns is the DEFINITION (`unit_reservations`) and the measurement, so
    the gap is loud on every local run instead of being re-derived by hand each time.
    """

    BOARD = (
        pr(101, ["area:sparq-core"]),                                   # PR half only
        iss(100, ["status:in-progress-review", "area:sparq-hdt"]),      # issue half only
    )

    def test_stripping_pr_rows_loses_the_pr_half_of_every_unit(self):
        pr_aware, issue_only, unheld = ready.occupancy_parity(list(self.BOARD), {101: {100}})
        self.assertEqual(pr_aware, {"sparq-core", "sparq-hdt"})
        self.assertEqual(issue_only, {"sparq-hdt"})
        self.assertEqual(unheld, {"sparq-core"})

    def test_the_gap_is_exactly_what_lets_a_second_worker_into_the_crate(self):
        # Behavioural, not a set difference: the issue-only view DISPATCHES onto the PR's crate.
        board = list(self.BOARD) + [iss(200, READY + ["priority:P1", "area:sparq-core"])]
        issue_only = [row for row in board if "pull_request" not in row]
        self.assertEqual(numbers(ready.compute_ready(issue_only, conflict_log=quiet)), [200],
                         "sanity: this is what the orchestrator does today")
        self.assertEqual(numbers(ready.compute_ready(board, conflict_log=quiet,
                                                     source_links={101: {100}})), [],
                         "the PR-aware unit view must hold #200 back")

    def test_parity_measurement_is_empty_when_nothing_is_stripped(self):
        # No false alarm: an issue-only snapshot with no PR-held keys reports no gap.
        board = [iss(100, ["status:in-progress-review", "area:sparq-hdt"])]
        _pr_aware, _issue_only, unheld = ready.occupancy_parity(board, {})
        self.assertEqual(unheld, set())

    def test_diagnose_reports_the_gap_loudly(self):
        board = [iss(100, ["status:in-progress-review", "area:sparq-hdt"]),
                 iss(200, READY + ["priority:P1", "area:sparq-geo"])]
        pulls = [dict(worker_pr(101, 100), labels=[{"name": "area:sparq-core"}])]
        prs = [dict(p, labels=[lb["name"] for lb in p["labels"]]) for p in pulls]
        out = TestLocalOrchestratorParity._run_cli(board + prs, pulls, argv=("--diagnose",))
        self.assertIn("ORCHESTRATOR OCCUPANCY GAP", out)
        self.assertIn("sparq-core", out.split("ORCHESTRATOR OCCUPANCY GAP")[1])


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
