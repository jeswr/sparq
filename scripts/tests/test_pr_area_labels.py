#!/usr/bin/env python3
# [OPUS-5] 🤖 SPARQ agent — the guard suite for PR `area:` attribution.
# Registry issue jeswr/agent-account-registry#677.
#
# Every guard in scripts/pr-area-labels.py + ci/area-labels.toml +
# .github/workflows/pr-area-label.yml has a NAMED test here that goes RED when the guard
# is deleted or inverted. The three families:
#
#   1. DERIVATION (pure) — a crates/x-only PR narrows to x; a workspace-spanning PR stays
#      `__global__`; an unattributable path stays `__global__`; an area with no live label
#      is never invented.
#   2. EFFECTS (fake `gh`) — the recorded call list IS the assertion: additive-only (a
#      human `area:` label survives, observed on the fake's simulated label state, not by
#      grepping the source), idempotent (a second pass writes nothing), dry-run writes
#      nothing, a fork PR and a no-pull-request ref make ZERO api calls.
#   3. THE YAML SEAM — parsed STRUCTURALLY, never by substring or `count(...) == N`, which
#      is what lets `if: false`, a deleted step or a swapped call-site argument survive.
#      Measured in this project: 18/18 Python mutants died while EVERY surviving mutant
#      lived in a workflow `if:` / step / call-site. So: the workflow must have NO `if:`
#      at any level (job or step), the labelling step must exist and pass exactly the
#      four documented arguments, checkout must be pinned to the default branch (never the
#      PR head — the job holds `pull-requests: write`), the permissions block must be
#      exactly the least-privilege pair, `pull_request` must carry NO `paths:` filter, and
#      routing-self-tests.yml must actually INVOKE this file (a test nobody runs is a
#      call-site mutant that survives everything else).
#
# Needs PyYAML (same dependency the other wiring suites use); everything else is stdlib.
# Run:  python3 scripts/tests/test_pr_area_labels.py

from __future__ import annotations

import importlib.util
import json
import re
import sys
import tomllib
import unittest
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
DERIVER = REPO_ROOT / "scripts" / "pr-area-labels.py"
POLICY = REPO_ROOT / "ci" / "area-labels.toml"
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "pr-area-label.yml"
ROUTING_WF = REPO_ROOT / ".github" / "workflows" / "routing-self-tests.yml"
CRATES = REPO_ROOT / "crates"
SKILLS = REPO_ROOT / "skills"
REPO = "sparq-org/sparq"


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader, path
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


M = _load("pr_area_labels", DERIVER)


def _yaml(path: Path) -> dict:
    return yaml.safe_load(path.read_text(encoding="utf-8"))


def _on_block(wf: dict) -> dict:
    """The `on:` mapping. Bare `on` is the YAML boolean True, so PyYAML keys it under
    `True` (same trick the other workflow-anchor suites use)."""
    return wf.get("on", wf.get(True, {})) or {}


class FakeGH:
    """A `gh` stand-in that RECORDS every invocation and SIMULATES label state.

    The simulated state is what makes the additive-only and idempotence assertions
    discriminating: they check the resulting label set, not the absence of a string in
    the source. A hypothetical "prune stale areas" feature would remove a label here and
    turn the additive-only test red.
    """

    def __init__(self, prs, labels):
        # prs: {number: {"labels": [..], "files": [..], "title": str, "isDraft": bool}}
        self.prs = {n: dict(p) for n, p in prs.items()}
        self.labels = list(labels)
        self.calls: list[list[str]] = []

    @property
    def writes(self):
        return [c for c in self.calls if c[:2] == ["pr", "edit"]]

    def __call__(self, args):
        self.calls.append(list(args))
        if args[0] == "api":
            return "".join(f"{name}\n" for name in self.labels)
        if args[:2] == ["pr", "view"]:
            n = int(args[2])
            pr = self.prs[n]
            return json.dumps({"number": n,
                               "labels": [{"name": lb} for lb in pr["labels"]],
                               "files": [{"path": p} for p in pr["files"]]})
        if args[:2] == ["pr", "list"]:
            return json.dumps([
                {"number": n, "labels": [{"name": lb} for lb in p["labels"]],
                 "files": [{"path": f} for f in p["files"]],
                 "title": p.get("title", ""), "isDraft": p.get("isDraft", False)}
                for n, p in sorted(self.prs.items())])
        if args[:2] == ["pr", "edit"]:
            n = int(args[2])
            it = iter(args[3:])
            for tok in it:
                if tok == "--add-label":
                    lb = next(it)
                    if lb not in self.prs[n]["labels"]:
                        self.prs[n]["labels"].append(lb)
                elif tok == "--remove-label":          # never exercised by this module
                    lb = next(it)
                    if lb in self.prs[n]["labels"]:
                        self.prs[n]["labels"].remove(lb)
            return ""
        raise AssertionError(f"unexpected gh call: {args}")


def _run(fake, argv, expect=0):
    import io
    buf = io.StringIO()
    rc = M.main(argv + ["--repo", REPO], runner=fake, out=buf)
    assert rc == expect, f"rc={rc} expected {expect}\n{buf.getvalue()}"
    return buf.getvalue()


# =========================================================================== policy


class TestPolicyTable(unittest.TestCase):
    """ci/area-labels.toml must be internally valid and TOTAL over the live tree — an
    unmapped path silently means `__global__`, i.e. the starvation reg#677 describes."""

    @classmethod
    def setUpClass(cls):
        cls.policy = M.load_policy()
        cls.members = cls.policy.members
        cls.raw = tomllib.loads(POLICY.read_text(encoding="utf-8"))

    def test_every_map_row_area_is_a_live_crate_or_declared_lane(self):
        """A typo'd `area` would be dropped at runtime and the PR would stay global —
        so it must fail HERE, hermetically, instead."""
        allowed = self.members | self.policy.non_crate
        bad = sorted({a for _, a in self.policy.rows if a not in allowed})
        self.assertEqual(bad, [], f"[[map]] areas that are neither a workspace member "
                                 f"nor a [lanes] non_crate name: {bad}")

    def test_every_map_row_pattern_root_exists_on_disk(self):
        """A dead row is a silent hole: it looks like coverage and matches nothing.

        `[policy] anticipated_roots` is the ONLY escape hatch (a fetched/generated tree,
        or a file an in-flight PR adds) and is itself checked below."""
        anticipated = set(self.raw["policy"].get("anticipated_roots") or [])
        missing = []
        for pattern, _ in self.policy.rows:
            if pattern in anticipated:
                continue
            root = pattern[:-3] if pattern.endswith("/**") else pattern
            if "*" in root or "?" in root:
                continue                      # a glob (e.g. `*.md`) has no fixed root
            if not (REPO_ROOT / root).exists():
                missing.append(pattern)
        self.assertEqual(missing, [], f"[[map]] patterns whose root does not exist: {missing}")

    def test_anticipated_roots_are_declared_rows_and_really_absent(self):
        """Keeps the escape hatch honest: an entry must correspond to a real row, and must
        be REMOVED once the path lands (otherwise it hides a future typo forever)."""
        anticipated = list(self.raw["policy"].get("anticipated_roots") or [])
        patterns = {p for p, _ in self.policy.rows}
        self.assertEqual([a for a in anticipated if a not in patterns], [],
                         "anticipated_roots entry with no matching [[map]] row")
        landed = [a for a in anticipated
                  if (REPO_ROOT / (a[:-3] if a.endswith("/**") else a)).exists()]
        self.assertEqual(landed, [], f"anticipated_roots entries that now EXIST — remove "
                                     f"them so the strict check covers them: {landed}")

    def test_every_crates_subdirectory_is_a_workspace_member(self):
        """The implicit `crates/<name>` rule is only TOTAL if every directory under
        crates/ is a member; a non-member directory resolves to nothing (fail closed)."""
        dirs = {p.name for p in CRATES.iterdir() if p.is_dir()}
        self.assertEqual(sorted(dirs - self.members), [],
                         "crates/ subdirectories that are not workspace members would "
                         "fall back to __global__")

    def test_every_skill_surface_has_a_map_row(self):
        """A new skill surface with no row silently sends its PRs to `__global__`."""
        patterns = {p for p, _ in self.policy.rows}
        missing = sorted(d.name for d in SKILLS.iterdir()
                         if d.is_dir() and f"skills/{d.name}/**" not in patterns)
        self.assertEqual(missing, [], f"skills/ surfaces with no [[map]] row: {missing}")

    def test_every_tracked_file_in_the_repo_resolves(self):
        """TOTALITY over the REAL tree — every git-tracked path must attribute to some
        area. This is the test that goes red when a new top-level directory or root file
        lands without a row, which is exactly the regression that re-opens reg#677's
        starvation for every PR that touches it. A synthetic `dir/probe.txt` probe would
        NOT discriminate (it can pass while the directory's real layout is unmapped, and
        fail on directories that hold only one known file), so this walks git's index."""
        import subprocess
        proc = subprocess.run(["git", "-C", str(REPO_ROOT), "ls-files", "-z"],
                              capture_output=True, text=True, check=False)
        if proc.returncode != 0:
            self.skipTest("git ls-files unavailable")
        tracked = [p for p in proc.stdout.split("\0") if p]
        self.assertGreater(len(tracked), 1000, "suspiciously small tracked-file set")
        known = self.members | self.policy.non_crate
        unresolved = sorted({p for p in tracked
                             if M.attribute(p, self.policy) not in known})
        self.assertEqual(unresolved, [], f"{len(unresolved)} tracked path(s) with no area "
                                         f"attribution (each sends its PR to __global__): "
                                         f"{unresolved[:20]}")

    def test_malformed_policy_raises_rather_than_labelling_nothing(self):
        import tempfile
        with tempfile.TemporaryDirectory() as td:
            bad = Path(td) / "area-labels.toml"
            bad.write_text('[policy]\nmax_areas = 0\n[[map]]\npattern = "x"\narea = "ci"\n')
            with self.assertRaises(M.PolicyError):
                M.load_policy(path=bad)
            bad.write_text('[policy]\nmax_areas = 4\n')
            with self.assertRaises(M.PolicyError):
                M.load_policy(path=bad)


# ======================================================================= derivation


class TestDerivation(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.policy = M.load_policy()
        cls.known = cls.policy.members | cls.policy.non_crate

    def d(self, paths, known=None):
        return M.derive_areas(paths, self.policy, self.known if known is None else known)

    # --- the headline guard: a single-crate PR must NOT be __global__ ---------------
    def test_single_crate_pr_narrows_to_that_crate_and_not_global(self):
        """reg#677's live case: draft PR #4143 touched only crates/sparq-reason/** plus
        skills/inference/SKILL.md, seized all 54 crates, and deferred #2688/#2732/#2786."""
        areas, reason = self.d(["crates/sparq-reason/src/lib.rs",
                                "crates/sparq-reason/tests/rules.rs",
                                "skills/inference/SKILL.md"])
        self.assertEqual((sorted(areas), reason), (["sparq-reason"], "resolved"))
        self.assertNotIn(M.GLOBAL, areas)
        self.assertTrue(areas, "an empty area set IS __global__ — the bug being fixed")

    def test_crate_prefix_wins_over_map_rows(self):
        # `crates/sparq-core/README.md` must be sparq-core, NOT the root-markdown `docs`
        # row: a wrong narrow label is worse than a stalled frontier.
        self.assertEqual(M.attribute("crates/sparq-core/README.md", self.policy), "sparq-core")

    def test_crate_prefix_wins_even_when_a_map_row_also_matches(self):
        """DISCRIMINATING form of the precedence guard.

        With the live table no `[[map]]` row can match a `crates/...` path, so simply
        re-ordering the two rules is an EQUIVALENT mutant against real data — it passes
        the test above for the wrong reason and only bites later, when someone adds a
        `crates/**`-shaped row. So construct a policy where both rules match and assert
        the crate still wins."""
        overlapping = M.Policy(max_areas=4, rows=[("crates/**", "workspace")],
                               non_crate=["workspace"], members=self.policy.members)
        self.assertEqual(M.attribute("crates/sparq-core/src/lib.rs", overlapping),
                         "sparq-core")
        # ... and a NON-member path under crates/ does not fall through to that row
        # either: `crates/` is the member rule's exclusive territory, fail closed.
        self.assertIsNone(M.attribute("crates/README.md", overlapping))

    def test_glob_star_does_not_cross_a_slash(self):
        self.assertEqual(M.attribute("README.md", self.policy), "docs")
        self.assertEqual(M.attribute("skills/inference/SKILL.md", self.policy), "sparq-reason")
        self.assertEqual(M.attribute("site/src/app/page.md", self.policy), "site")

    # --- the fail-closed guards ----------------------------------------------------
    def test_genuinely_cross_cutting_pr_stays_global(self):
        wide = [f"crates/{c}/src/lib.rs"
                for c in sorted(self.policy.members)[:self.policy.max_areas + 1]]
        areas, reason = self.d(wide)
        self.assertEqual((areas, reason), (frozenset(), "cross-cutting"))

    def test_declared_max_areas_is_the_reviewed_value(self):
        """The cap is a SCHEDULER-SAFETY knob, so it is pinned ABSOLUTELY here.

        Every other cross-cutting test derives its fixture from `policy.max_areas`, which
        means raising the declared value moves those fixtures with it and nothing goes
        red — the cap could be set to 60 (i.e. "nothing is ever cross-cutting") silently.
        Changing the policy is allowed; changing it without updating this line is not."""
        self.assertEqual(self.policy.max_areas, 4)

    def test_a_five_crate_pr_stays_global_at_the_declared_cap(self):
        """The absolute-fixture companion to the test above: five NAMED crates, no
        dependence on `policy.max_areas`."""
        five = ["crates/sparq-core/src/lib.rs", "crates/sparq-engine/src/lib.rs",
                "crates/sparq-cli/src/main.rs", "crates/sparq-server/src/main.rs",
                "crates/sparq-shacl/src/lib.rs"]
        self.assertEqual(self.d(five), (frozenset(), "cross-cutting"))

    def test_max_areas_boundary_is_still_narrow(self):
        """Discriminates the cap from an off-by-one: exactly max_areas must RESOLVE."""
        ok = [f"crates/{c}/src/lib.rs"
              for c in sorted(self.policy.members)[:self.policy.max_areas]]
        areas, reason = self.d(ok)
        self.assertEqual(reason, "resolved")
        self.assertEqual(len(areas), self.policy.max_areas)

    def test_one_unattributable_path_poisons_the_whole_pr(self):
        areas, reason = self.d(["crates/sparq-core/src/lib.rs", "no/such/top/level/x.txt"])
        self.assertEqual((areas, reason), (frozenset(), "unresolved"))

    def test_unknown_crate_directory_without_a_live_label_is_unresolved(self):
        self.assertEqual(self.d(["crates/not-a-crate/src/lib.rs"]),
                         (frozenset(), "unresolved"))

    def test_a_file_directly_under_crates_is_not_a_crate_area(self):
        """DISCRIMINATING: `crates/README.md` must not become `area:README.md`. The
        live-label check would also reject that name, so assert on `attribute` directly —
        otherwise this fixture passes with the segment guard deleted."""
        self.assertIsNone(M.attribute("crates/README.md", self.policy))
        self.assertIsNone(M.attribute("crates", self.policy))

    def test_a_pr_that_ADDS_a_crate_resolves_once_its_label_exists(self):
        """The workflow reads the DEFAULT BRANCH's manifest, so a crate a PR is adding is
        not yet a workspace member. Such a PR collides with nothing and must still get a
        narrow partition — gated only on a human having created the `area:` label. Two
        live examples in the backfill were exactly this shape (#3464, #3581)."""
        new = "sparq-brand-new"
        self.assertNotIn(new, self.policy.members)
        self.assertEqual(
            M.derive_areas([f"crates/{new}/src/lib.rs"], self.policy, self.known | {new}),
            (frozenset({new}), "resolved"))
        # ... and without the label it is still fail-closed.
        self.assertEqual(M.derive_areas([f"crates/{new}/src/lib.rs"], self.policy, self.known),
                         (frozenset(), "unresolved"))

    def test_empty_change_set_stays_global(self):
        self.assertEqual(self.d([]), (frozenset(), "no-paths"))

    def test_area_with_no_live_label_is_never_invented(self):
        """The `POST .../labels` call CREATES an unknown label, so an area outside the
        live label set must be treated as unresolved."""
        self.assertEqual(
            M.derive_areas(["crates/sparq-reason/src/lib.rs"], self.policy,
                           self.known - {"sparq-reason"}),
            (frozenset(), "unresolved"))

    # --- additive-only planning ----------------------------------------------------
    def test_plan_never_proposes_removing_a_human_area_label(self):
        plan = M.plan_for_pr({"number": 1,
                              "labels": ["area:sparq-core", "review:parked"],
                              "files": ["crates/sparq-engine/src/lib.rs"]},
                             self.policy, self.known)
        self.assertEqual(plan["add"], ["area:sparq-engine"])
        self.assertEqual(plan["partition"], ["sparq-core", "sparq-engine"])

    def test_fail_closed_pr_that_already_has_a_human_area_keeps_it(self):
        plan = M.plan_for_pr({"number": 2, "labels": ["area:sparq-zk"],
                              "files": ["no/such/top/level/x.txt"]},
                             self.policy, self.known)
        self.assertEqual(plan["add"], [])
        self.assertEqual(plan["partition"], ["sparq-zk"])
        self.assertNotEqual(plan["partition"], [M.GLOBAL])

    def test_already_labelled_pr_is_a_noop(self):
        plan = M.plan_for_pr({"number": 3, "labels": ["area:sparq-engine"],
                              "files": ["crates/sparq-engine/src/lib.rs"]},
                             self.policy, self.known)
        self.assertTrue(plan["noop"])
        self.assertEqual(plan["add"], [])


# ========================================================================== effects


class TestEffects(unittest.TestCase):
    """What actually reaches `gh`. Assertions are on the recorded call list and the
    fake's simulated label state, never on the source text."""

    def fake(self, files=("crates/sparq-reason/src/lib.rs",), labels=()):
        policy = M.load_policy()
        live = [f"area:{a}" for a in sorted(policy.members | policy.non_crate)]
        return FakeGH({7: {"labels": list(labels), "files": list(files)}}, live)

    def test_apply_adds_exactly_the_derived_label(self):
        f = self.fake()
        _run(f, ["--pr", "7", "--apply"])
        self.assertEqual(f.writes, [["pr", "edit", "7", "--repo", REPO,
                                     "--add-label", "area:sparq-reason"]])
        self.assertEqual(f.prs[7]["labels"], ["area:sparq-reason"])

    def test_dry_run_is_the_default_and_writes_nothing(self):
        f = self.fake()
        out = _run(f, ["--pr", "7"])
        self.assertEqual(f.writes, [])
        self.assertEqual(f.prs[7]["labels"], [])
        self.assertIn("would label", out)
        self.assertIn("DRY RUN", out)

    def test_human_applied_area_label_survives_an_apply(self):
        """Outcome assertion: the fake would DROP the label if a removal were ever
        issued, so this reds on any future prune-stale-areas behaviour."""
        f = self.fake(labels=["area:sparq-zk"])
        _run(f, ["--pr", "7", "--apply"])
        self.assertIn("area:sparq-zk", f.prs[7]["labels"])
        self.assertIn("area:sparq-reason", f.prs[7]["labels"])
        flat = [tok for call in f.calls for tok in call]
        self.assertNotIn("--remove-label", flat)
        self.assertNotIn("DELETE", flat)

    def test_second_pass_is_a_pure_noop(self):
        f = self.fake()
        _run(f, ["--pr", "7", "--apply"])
        before = list(f.prs[7]["labels"])
        f.calls.clear()
        out = _run(f, ["--pr", "7", "--apply"])
        self.assertEqual(f.writes, [], "idempotence broken: a second pass wrote")
        self.assertEqual(f.prs[7]["labels"], before)
        self.assertIn("no-change", out)

    def test_backfill_is_idempotent_across_the_whole_open_set(self):
        policy = M.load_policy()
        live = [f"area:{a}" for a in sorted(policy.members | policy.non_crate)]
        f = FakeGH({
            11: {"labels": [], "files": ["crates/sparq-core/src/lib.rs"]},
            12: {"labels": [], "files": ["site/src/app/page.tsx", ".github/workflows/ci.yml"]},
            13: {"labels": [], "files": ["no/such/top/level/x.txt"]},
            14: {"labels": ["area:sparq-hdt"], "files": ["crates/sparq-hdt/src/lib.rs"]},
        }, live)
        _run(f, ["--backfill", "--apply"])
        self.assertEqual(sorted(f.prs[11]["labels"]), ["area:sparq-core"])
        self.assertEqual(sorted(f.prs[12]["labels"]), ["area:ci", "area:site"])
        self.assertEqual(f.prs[13]["labels"], [], "unresolved PR must stay __global__")
        self.assertEqual(f.prs[14]["labels"], ["area:sparq-hdt"])
        snapshot = {n: sorted(p["labels"]) for n, p in f.prs.items()}
        f.calls.clear()
        out = _run(f, ["--backfill", "--apply"])
        self.assertEqual(f.writes, [], "second backfill pass wrote — not idempotent")
        self.assertEqual({n: sorted(p["labels"]) for n, p in f.prs.items()}, snapshot)
        self.assertIn("0 relabelled", out)

    def test_fork_pr_makes_zero_api_calls_and_exits_zero(self):
        """`pull_request` gives a fork a READ-ONLY token. The run must report and stop —
        not crash, and not mislabel."""
        f = self.fake()
        out = _run(f, ["--pr", "7", "--apply", "--head-repo", "someone-else/sparq"])
        self.assertEqual(f.calls, [], "a fork run must not touch the API at all")
        self.assertEqual(f.prs[7]["labels"], [])
        self.assertIn("READ-ONLY token", out)

    def test_same_repo_head_is_not_treated_as_a_fork(self):
        """Discriminates the fork guard from a blanket skip."""
        f = self.fake()
        _run(f, ["--pr", "7", "--apply", "--head-repo", REPO])
        self.assertEqual(f.prs[7]["labels"], ["area:sparq-reason"])

    def test_empty_head_repo_fails_closed(self):
        f = self.fake()
        _run(f, ["--pr", "7", "--apply", "--head-repo", ""])
        self.assertEqual(f.calls, [])

    def test_no_pull_request_context_makes_zero_api_calls(self):
        """The merge_group path: the workflow passes `--pr 0`."""
        f = self.fake()
        out = _run(f, ["--pr", "0", "--apply", "--head-repo", ""])
        self.assertEqual(f.calls, [])
        self.assertIn("no pull-request context", out)

    def test_unreadable_label_set_labels_nothing(self):
        """Fail-closed on the label read: no labels visible => never guess."""
        f = FakeGH({7: {"labels": [], "files": ["crates/sparq-reason/src/lib.rs"]}}, [])
        out = _run(f, ["--pr", "7", "--apply"])
        self.assertEqual(f.writes, [])
        self.assertIn("inert", out)

    def test_transient_gh_failure_retries_but_a_persistent_one_raises(self):
        """A retry must not become exit-zero swallowing of an earned hard failure."""
        calls = {"n": 0}

        def flaky(args, **kw):
            calls["n"] += 1
            return None
        import subprocess as sp

        class R:
            def __init__(self, rc):
                self.returncode, self.stdout, self.stderr = rc, "ok\n", "boom"
        seq = [R(1), R(0)]
        orig = sp.run
        try:
            sp.run = lambda *a, **k: seq.pop(0)
            self.assertEqual(M._gh(["api", "x"], sleep=lambda _s: None), "ok\n")
            seq2 = [R(1), R(1), R(1)]
            sp.run = lambda *a, **k: seq2.pop(0)
            with self.assertRaises(RuntimeError):
                M._gh(["api", "x"], sleep=lambda _s: None)
        finally:
            sp.run = orig


# ======================================================================== YAML seam


class TestWorkflowSeam(unittest.TestCase):
    """Structural assertions over .github/workflows/pr-area-label.yml.

    A substring or `count(...) == N` check over the workflow TEXT does not catch
    `if: false`, a deleted step, or a swapped call-site argument — so everything below
    walks the PARSED document."""

    @classmethod
    def setUpClass(cls):
        cls.wf = _yaml(WORKFLOW)
        cls.jobs = cls.wf["jobs"]
        cls.job = cls.jobs["label"]
        cls.steps = cls.job["steps"]

    # --- triggers ------------------------------------------------------------------
    def test_triggers_cover_every_event_that_changes_the_path_set(self):
        on = _on_block(self.wf)
        self.assertEqual(sorted(on["pull_request"]["types"]),
                         ["opened", "ready_for_review", "reopened", "synchronize"])
        self.assertIn("merge_group", on)

    def test_pull_request_trigger_has_no_paths_filter(self):
        """A `paths:` filter here re-opens the starvation hole for whatever it excludes."""
        on = _on_block(self.wf)
        pr = on["pull_request"]
        self.assertNotIn("paths", pr)
        self.assertNotIn("paths-ignore", pr)

    def test_uses_pull_request_not_pull_request_target(self):
        """`pull_request_target` would hand a FORK's PR a WRITE token — the escalation
        this design deliberately refuses."""
        self.assertNotIn("pull_request_target", _on_block(self.wf))

    # --- the `if:` seam ------------------------------------------------------------
    def test_no_job_level_if_anywhere(self):
        """Catches `if: false` and every other job-level guard mutation."""
        guarded = sorted(j for j, spec in self.jobs.items() if "if" in spec)
        self.assertEqual(guarded, [], f"job-level `if:` found on {guarded}; every skip "
                                      f"decision belongs in Python (see the workflow header)")

    def test_no_step_level_if_anywhere(self):
        """Catches `if: false` on the labelling step, the classic surviving mutant."""
        guarded = [s.get("name") or s.get("uses") for s in self.steps if "if" in s]
        self.assertEqual(guarded, [], f"step-level `if:` found on {guarded}")

    # --- the call site -------------------------------------------------------------
    def test_the_labelling_step_exists_and_passes_exactly_the_documented_arguments(self):
        """Catches a DELETED step and a WRONG INPUT. Each argument is asserted as a
        whole token pair, so `--pr ${{ github.event.number }}` (a different, sometimes
        absent field) reds, and so does dropping `--apply` (which would make the live
        workflow a silent no-op that still reports success)."""
        runs = [s["run"] for s in self.steps if "run" in s]
        self.assertEqual(len(runs), 1, "expected exactly one run step")
        run = " ".join(runs[0].split())          # normalise line continuations
        self.assertIn("python3 scripts/pr-area-labels.py", run)
        for arg in ('--repo "${{ github.repository }}"',
                    '--pr "${{ github.event.pull_request.number || 0 }}"',
                    '--head-repo "${{ github.event.pull_request.head.repo.full_name || \'\' }}"',
                    "--apply"):
            self.assertIn(arg, run, f"missing/altered call-site argument: {arg}")
        self.assertNotIn("--dry-run", run)
        self.assertNotIn("--backfill", run)

    def test_the_run_block_guards_the_bootstrap_case(self):
        """The checkout is the DEFAULT BRANCH, so the deriver is absent on the PR that
        introduces it (measured: this job failed exactly that way on #4170) and on any ref
        after a revert. The guard must exist, must exit 0, must be a ::warning (a genuine
        disappearance has to be visible, not look like a healthy no-op), and — the
        discriminating part — must guard the SAME path the step then executes."""
        run = next(s["run"] for s in self.steps if "run" in s)
        flat = " ".join(run.split())
        guards = re.findall(r"if \[ ! -f (\S+) \]; then", flat)
        self.assertEqual(len(guards), 1, "expected exactly one existence guard")
        invoked = re.search(r"python3 (\S+\.py)", flat)
        self.assertIsNotNone(invoked)
        self.assertEqual(guards[0], invoked.group(1),
                         "the guarded path and the invoked path must be the same file")
        self.assertIn("exit 0", flat)
        self.assertIn("::warning title=pr-area-label inert::", flat)
        self.assertNotIn("::notice title=pr-area-label inert::", flat)

    def test_deriver_and_policy_paths_referenced_by_the_workflow_exist(self):
        run = " ".join(" ".join(s["run"] for s in self.steps if "run" in s).split())
        script = next(tok for tok in run.split() if tok.endswith(".py"))
        self.assertTrue((REPO_ROOT / script).is_file(), f"{script} does not exist")

    # --- the privileged-token posture ---------------------------------------------
    def test_checkout_uses_default_branch_not_pr_head(self):
        """The job holds `pull-requests: write`; running PR-authored code under it is the
        GITHUB_ENV-class escalation. Checkout MUST be pinned to the default branch."""
        checkouts = [s for s in self.steps if "actions/checkout" in str(s.get("uses", ""))]
        self.assertEqual(len(checkouts), 1)
        with_ = checkouts[0].get("with") or {}
        self.assertEqual(with_.get("ref"), "${{ github.event.repository.default_branch }}")
        self.assertIs(with_.get("persist-credentials"), False)

    def test_permissions_are_exactly_the_least_privilege_pair(self):
        self.assertEqual(self.wf["permissions"],
                         {"contents": "read", "pull-requests": "write"})
        for job in self.jobs.values():
            self.assertNotIn("permissions", job, "a job-level permissions block would "
                                                 "override the least-privilege default")

    def test_every_action_is_sha_pinned(self):
        unpinned = []
        for step in self.steps:
            uses = step.get("uses")
            if not uses:
                continue
            ref = uses.rsplit("@", 1)[-1]
            if len(ref) != 40 or not all(c in "0123456789abcdef" for c in ref):
                unpinned.append(uses)
        self.assertEqual(unpinned, [], f"actions not SHA-pinned: {unpinned}")

    def test_no_secret_other_than_the_scoped_github_token(self):
        text = WORKFLOW.read_text(encoding="utf-8")
        bad = [ln.strip() for ln in text.splitlines()
               if "secrets." in ln and "secrets.GITHUB_TOKEN" not in ln]
        self.assertEqual(bad, [], f"unexpected secret reference: {bad}")

    def test_job_name_is_not_declared_advisory(self):
        """The gate excludes a check IFF it is in .github/advisory-registry.json; the name
        token alone is diagnostic. Pin the intended GATING posture from both sides."""
        self.assertNotIn("advisory", self.job["name"].lower())
        registry = json.loads((REPO_ROOT / ".github" / "advisory-registry.json")
                              .read_text(encoding="utf-8"))
        self.assertNotIn(self.job["name"], registry["jobs"])


class TestSelfTestIsActuallyInvoked(unittest.TestCase):
    """The call-site mutant class: a suite nobody runs cannot go red. routing-self-tests
    must INVOKE this file and the deriver's own `--self-test`, and must WATCH the paths
    that can break either."""

    @classmethod
    def setUpClass(cls):
        cls.wf = _yaml(ROUTING_WF)

    def _run_block(self):
        steps = self.wf["jobs"]["validate"]["steps"]
        return " ".join(" ".join(s["run"] for s in steps if "run" in s).split())

    def test_routing_self_tests_invokes_this_suite_and_the_deriver_self_test(self):
        run = self._run_block()
        self.assertIn("python3 scripts/tests/test_pr_area_labels.py", run)
        self.assertIn("python3 scripts/pr-area-labels.py --self-test", run)

    def test_routing_self_tests_watches_every_file_this_suite_pins(self):
        on = _on_block(self.wf)
        watched = set(on["pull_request"].get("paths") or []) & set(
            on["push"].get("paths") or [])
        for path in ("scripts/pr-area-labels.py", "ci/area-labels.toml",
                     "scripts/tests/test_pr_area_labels.py",
                     ".github/workflows/pr-area-label.yml"):
            self.assertIn(path, watched, f"{path} is not in BOTH paths filters — a change "
                                         f"to it would not run this suite")

    def test_the_validate_job_has_no_if_guard(self):
        self.assertNotIn("if", self.wf["jobs"]["validate"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
