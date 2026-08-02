#!/usr/bin/env python3
# [OPUS-5] #5214 — INSPECTION test for ci.yml's rust-cache `shared-key` grouping.
#
# WHY A TEST AT ALL: a `shared-key` is one YAML line that is INVISIBLE when
# absent. Delete it and nothing goes red — rust-cache silently falls back to a
# job-name-derived key, the job quietly takes its own slice of the repository's
# shared 10 GB Actions-cache budget again, and the LRU eviction that sq-3sbrr
# (#1395) traced back to budget pressure starts making warm caches restore cold.
# That is exactly the regression this bead fixed, so the grouping is pinned here.
#
# The test binds FOUR things together so none of them can drift alone:
#   1. ci.yml — every job in the declared group carries the shared key, and no
#      job outside it does.
#   2. scripts/ci-cache-closure-overlap.py — its GROUPED table names real ci.yml
#      jobs (a rename on either side reds here rather than silently orphaning a
#      cache entry).
#   3. the MEASUREMENT itself — the script's own `--check` runs here, so a
#      dependency change that pulls one member's closure away from the rest
#      reds instead of quietly costing that job a cold dependency build.
#   4. the CLOSURE MODEL that measurement rests on — a model that counted every
#      declared dependency regardless of `optional = true` would report a
#      phantom common core and wave any grouping through, so the feature
#      resolution is pinned directly, both on a synthetic manifest and
#      end-to-end on the real ones.
#
# Hermetic: stdlib only (no PyYAML, no network, no gh, no cargo).
# Run:  python3 scripts/tests/test_cache_shared_key_grouping.py

from __future__ import annotations

import importlib.util
import re
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
CI_YML = WORKFLOWS / "ci.yml"
OVERLAP_SCRIPT = REPO_ROOT / "scripts" / "ci-cache-closure-overlap.py"

RUST_CACHE = "Swatinem/rust-cache@"
_JOB_ID = re.compile(r"^  ([A-Za-z0-9_-]+):\s*$")
_NEW_LIST_ITEM = re.compile(r"^ {0,6}- ")


def _load_overlap_module():
    spec = importlib.util.spec_from_file_location("ci_cache_closure_overlap", OVERLAP_SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


overlap = _load_overlap_module()


def _step_body(lines: list[str], start: int) -> list[str]:
    """The lines of the `steps:` list item whose first line is `lines[start]`."""
    i = start + 1
    while i < len(lines):
        line = lines[i]
        if not line.strip():
            i += 1
            continue
        indent = len(line) - len(line.lstrip())
        if _NEW_LIST_ITEM.match(line) or indent < 8:
            break
        i += 1
    return lines[start:i]


def rust_cache_steps_by_job() -> dict[str, list[list[str]]]:
    """job id -> the body of each `Swatinem/rust-cache` step inside that job."""
    lines = CI_YML.read_text().split("\n")
    steps: dict[str, list[list[str]]] = {}
    job = None
    for i, line in enumerate(lines):
        match = _JOB_ID.match(line)
        if match:
            job = match.group(1)
        if RUST_CACHE in line and "uses:" in line:
            assert job is not None, f"ci.yml:{i + 1}: rust-cache step before any job id"
            steps.setdefault(job, []).append(_step_body(lines, i))
    return steps


def with_value(body: list[str], key: str) -> str | None:
    for line in body:
        stripped = line.strip()
        if stripped.startswith("#"):
            continue
        if stripped.startswith(key):
            return stripped[len(key):].strip()
    return None


class TestParserNonVacuity(unittest.TestCase):
    """Every assertion below iterates a discovered set. If discovery returns
    nothing they all pass while checking nothing — so pin discovery itself."""

    def test_every_rust_cache_step_is_attributed_to_a_job(self) -> None:
        found = sum(len(v) for v in rust_cache_steps_by_job().values())
        named = CI_YML.read_text().count(RUST_CACHE)
        self.assertEqual(
            found,
            named,
            f"the step walker attributed {found} rust-cache steps to jobs but ci.yml "
            f"names the action {named} times — a step shape the walker does not "
            "understand would be silently unchecked.",
        )

    def test_the_group_is_not_empty(self) -> None:
        self.assertGreater(
            len(overlap.GROUPED),
            1,
            "a `shared-key` group of fewer than two jobs saves no cache budget; "
            "if the group collapsed to one, drop the key and this test with it.",
        )


class TestGroupedJobsCarryTheSharedKey(unittest.TestCase):
    def test_every_declared_group_member_is_a_real_ci_job(self) -> None:
        jobs = rust_cache_steps_by_job()
        for job in overlap.GROUPED:
            self.assertIn(
                job,
                jobs,
                f"ci-cache-closure-overlap.py groups `{job}`, but ci.yml has no job by "
                "that name with a rust-cache step. A rename on either side orphans the "
                "cache entry silently — fix both together.",
            )

    def test_every_group_member_carries_the_shared_key(self) -> None:
        jobs = rust_cache_steps_by_job()
        for job in overlap.GROUPED:
            # A missing job is diagnosed by the sibling test above; look it up
            # defensively so this one reports the KEY problem, not a KeyError.
            for body in jobs.get(job, []):
                self.assertEqual(
                    with_value(body, "shared-key:"),
                    overlap.GROUP_SHARED_KEY,
                    f"ci.yml job `{job}` must set "
                    f"`shared-key: {overlap.GROUP_SHARED_KEY}`. Without it rust-cache "
                    "derives the key from the job name and this job takes its own slice "
                    "of the shared 10 GB Actions-cache budget again (#5214).",
                )

    def test_no_job_outside_the_group_carries_the_shared_key(self) -> None:
        for job, bodies in rust_cache_steps_by_job().items():
            if job in overlap.GROUPED:
                continue
            for body in bodies:
                self.assertNotEqual(
                    with_value(body, "shared-key:"),
                    overlap.GROUP_SHARED_KEY,
                    f"ci.yml job `{job}` joined the `{overlap.GROUP_SHARED_KEY}` entry "
                    "without being declared in ci-cache-closure-overlap.py's GROUPED "
                    "table, so its dependency closure was never measured against the "
                    "group. Add it there (the script will check it) or give it its own "
                    "key.",
                )


class TestTheMeasurementStillHolds(unittest.TestCase):
    def test_declared_grouping_matches_the_measured_closures(self) -> None:
        self.assertEqual(
            overlap.check(overlap.measure()),
            0,
            "the declared shared-key grouping no longer matches the measured "
            "dependency closures — see the FAIL lines above. Run "
            "`python3 scripts/ci-cache-closure-overlap.py` for the full table.",
        )


class TestFeatureResolution(unittest.TestCase):
    """The closure model must not count a dependency the job never compiles.

    This is the assumption everything else rests on: if `optional = true` were
    ignored, every job touching `sparq-conformance` would be credited with its
    whole opt-in graph (the tokio/axum loopback server, the JSON-LD parser, four
    reasoners), the resulting phantom common core would push unrelated closures
    over the floor, and `--check` would wave any grouping through.
    """

    # A synthetic manifest, so this pins the RESOLVER rather than whatever the
    # real crates happen to declare today.
    FIXTURE = {
        "package": {"name": "fixture"},
        "features": {
            "default": ["plain"],
            "plain": [],
            "with-opt": ["dep:opt"],
            "with-weak": ["always?/extra"],
            "chain": ["with-opt"],
        },
        "dependencies": {
            "always": {"version": "1"},
            "opt": {"version": "1", "optional": True},
            # No `dep:implicit` anywhere, so it defines an implicit feature.
            "implicit": {"version": "1", "optional": True},
        },
        "dev-dependencies": {"only-dev": {"version": "1"}},
    }

    def enabled(self, features, default=True, kinds=frozenset({"normal", "build"})):
        return set(
            overlap.enabled_deps(self.FIXTURE, set(features), default, set(kinds))
        )

    def test_an_optional_dependency_is_absent_until_a_feature_activates_it(self) -> None:
        self.assertNotIn("opt", self.enabled([]))
        self.assertIn("opt", self.enabled(["with-opt"]))

    def test_a_feature_reached_transitively_activates_its_dependency(self) -> None:
        self.assertIn("opt", self.enabled(["chain"]))

    def test_an_optional_dependency_with_no_dep_reference_has_an_implicit_feature(self) -> None:
        self.assertNotIn("implicit", self.enabled([]))
        self.assertIn("implicit", self.enabled(["implicit"]))

    def test_a_non_optional_dependency_is_always_present(self) -> None:
        self.assertIn("always", self.enabled([], default=False))

    def test_default_features_are_only_applied_when_asked_for(self) -> None:
        # `default` -> `plain` activates no dependency, so observe it through the
        # feature edge instead: a weak `always?/extra` fires on a non-optional dep.
        self.assertEqual(
            overlap.enabled_deps(self.FIXTURE, {"with-weak"}, True, {"normal"})["always"][1],
            frozenset({"extra"}),
        )

    def test_dev_dependencies_are_only_pulled_by_test_building_commands(self) -> None:
        self.assertNotIn("only-dev", self.enabled([]))
        self.assertIn("only-dev", self.enabled([], kinds={"normal", "build", "dev"}))

    def test_the_real_closure_tracks_a_feature_gated_dependency(self) -> None:
        """End-to-end: the async runtime `service-loopback` pulls in is absent
        from the default build of the same crate."""
        workspace = overlap.load_workspace()
        by_name, by_id = overlap.load_lock()

        def closure(cmd):
            return overlap.external_closure((cmd,), workspace, by_name, by_id)

        off = closure(overlap.Cmd("sparq-conformance", dev=True))
        on = closure(overlap.Cmd("sparq-conformance", ("service-loopback",), dev=True))
        self.assertNotIn("tokio", {name for name, _ in off})
        self.assertIn("tokio", {name for name, _ in on})
        self.assertLess(
            off,
            on,
            "turning a feature ON must only ADD packages; if the default closure is "
            "not a subset of the feature-on one the resolver is losing edges.",
        )


class TestSuiteIsWiredIntoCi(unittest.TestCase):
    """A structural test that never runs is a comment. Pin its own call site."""

    def test_docs_quality_invokes_this_suite(self) -> None:
        docs_quality = (WORKFLOWS / "docs-quality.yml").read_text()
        self.assertIn(
            "scripts/tests/test_cache_shared_key_grouping.py",
            docs_quality,
            "this suite must be invoked by docs-quality.yml or it silently stops gating.",
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
