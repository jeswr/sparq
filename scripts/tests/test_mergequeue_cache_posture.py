#!/usr/bin/env python3
# [SONNET-4.6] sq-6vshe.15 — INSPECTION test for the merge-queue cache/artifact
# posture (research/ci-mergequeue-speedup-2026-07.md §3.2, lever 2 of the CI
# structural-speedup program sq-6vshe; extends sq-6vshe.5, which owns key schema
# and backend sizing).
#
# WHY a test at all: both properties this bead establishes are single YAML lines
# that are invisible when absent. Nothing goes red if a future edit drops them —
# CI just silently gets slower and the shared cache budget silently churns again.
# So the two lines are PINNED here, structurally, over every workflow that can run
# on a ref other than `refs/heads/main`:
#
#   1. CACHE-SAVE DISCIPLINE. Every `Swatinem/rust-cache` step in a workflow
#      triggered by `merge_group` or `pull_request` must carry
#      `save-if: ${{ github.ref == 'refs/heads/main' }}`.
#      A save from a merge-queue entry is DEAD ON ARRIVAL: Actions cache scoping
#      makes an entry written from `refs/heads/gh-readonly-queue/<base>/pr-<N>-<sha>`
#      visible to that ref alone, and GitHub deletes that ref as soon as the entry
#      merges or is ejected — nothing can ever restore it, so the save is pure
#      post-step wall-clock on the queue's critical path. Branch-scoped saves also
#      churn the repo's shared 10 GB Actions-cache budget and LRU-evict the
#      main-scoped entries that every branch actually restores. This is sq-3sbrr's
#      doctrine, already live in feature-matrix.yml and the wasm lanes; sq-6vshe.15
#      closed the merge-queue half (ci.yml's 20 cache steps + vectorized-feature-off).
#      RESTORE is deliberately untouched — `save-if` gates saving only, and queue
#      refs branch off main and read main's entry, which is where the hit comes from.
#
#      #5166 widened the SCOPE from `merge_group` to `merge_group OR pull_request`.
#      The budget-churn half of the rationale never depended on the queue: an
#      ordinary PR head is just as branch-scoped as a `gh-readonly-queue/*` ref, and
#      the repo already applied the guard by hand on the gui/js/site-e2e/site-visual
#      PR lanes. `pull_request` is the precise discriminator for "this lane can run
#      on a ref that is not main" — every other trigger the repo's rust-cache lanes
#      use (`schedule`, `workflow_dispatch`, `workflow_run` filtered to main, `push`
#      with `branches: [main]`) already reports `github.ref == refs/heads/main`, so
#      the guard there is a no-op that only adds noise, and `release.yml` runs off
#      `refs/tags/v*`, where a main-only guard would stop it saving outright. Those
#      lanes are DELIBERATELY not asserted over — this suite pins the property where
#      it bites, not everywhere it could be typed.
#
#      One lane is EXEMPT with cause; see EXEMPT below. The exemption is itself
#      pinned, so it cannot outlive the conditions that justify it.
#
#   2. ARTIFACT DIET. The `nextest-archive` upload must set `compression-level: 0`.
#      `nextest.tar.zst` is already zstd-compressed; upload-artifact's default
#      DEFLATE-6 zip pass re-compresses incompressible bytes for a ~0 size delta,
#      burning CPU on the one job every test shard is blocked on. Level 0 stores the
#      file as-is. The test also pins the ARCHIVE CONTENT-PARITY contract the diet
#      must never break: the uploaded `path:` is exactly the file `cargo nextest
#      archive --archive-file` writes, and the artifact `name:` is the one the test
#      shards download — so the shards keep receiving the same byte-identical
#      archive, and the nextest test set is unchanged.
#
# Deliberately NOT asserted: sccache. Bead item 3 (an sccache/GHA-backend A/B on
# build-archive) is measure-first with a >=60 s median-win adoption bar, and no such
# measurement has been taken — so nothing about sccache is wired, claimed, or pinned
# here, and no verdict on it should be inferred from this file's silence.
#
# Hermetic: stdlib only (no PyYAML, no network, no gh) so it runs anywhere.
# Run:  python3 scripts/tests/test_mergequeue_cache_posture.py

from __future__ import annotations

import re
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
CI_YML = WORKFLOWS / "ci.yml"

RUST_CACHE = "Swatinem/rust-cache@"
SAVE_IF_KEY = "save-if:"
# The canonical guard. Bare `false` would ALSO stop the queue-ref saves — and would
# stop main from ever seeding a cache, so every job would restore nothing forever.
# The value is pinned exactly for that reason.
SAVE_IF_VALUE = "${{ github.ref == 'refs/heads/main' }}"

# The ONE documented exemption from the save discipline (#5166), keyed by workflow
# filename. Value is the reason, quoted into the failure message of the test that
# re-validates the exemption's preconditions.
#
# mutants-diff.yml is `pull_request`-ONLY and its rust-cache step passes neither
# `key:` nor `shared-key:`, so rust-cache derives a default key unique to that
# workflow+job which NOTHING on main ever writes. A main-only guard would not
# redirect the save to main; it would delete it, leaving the lane permanently cold
# (a full workspace rebuild plus cargo-mutants every run) with no main-scoped entry
# to fall back on. The branch-scoped save it makes today is genuinely restorable —
# by the next push to the same PR head — which is the only reuse this lane can have.
EXEMPT = {
    "mutants-diff.yml": (
        "pull_request-only lane with a job-unique default cache key that no "
        "main-ref run ever seeds; the guard would delete the save, not move it"
    ),
}

NEXTEST_ARCHIVE_ARTIFACT = "nextest-archive"
UPLOAD_ARTIFACT = "actions/upload-artifact@"
DOWNLOAD_ARTIFACT = "actions/download-artifact@"

_NEW_LIST_ITEM = re.compile(r"^ {0,6}- ")
_ARCHIVE_FILE_ARG = re.compile(r"--archive-file\s+(\S+)")


# --------------------------------------------------------------------------- #
# Minimal structural helpers. PyYAML would flatten the comments these workflows
# carry and is not installed in every environment, so we walk lines instead.
# --------------------------------------------------------------------------- #
def _lines(path: Path) -> list[str]:
    return path.read_text().split("\n")


def _is_comment(line: str) -> bool:
    return line.lstrip().startswith("#")


def on_block(path: Path) -> list[str]:
    """The workflow's top-level `on:` block, comment lines stripped.

    Comments are dropped because several workflows carry a prose note explaining
    that `merge_group` was REMOVED (2026-07-18 maintainer directive), and counting
    those would be exactly the false positive that makes this suite assert the
    property over lanes that never see a queue ref.
    """
    lines = _lines(path)
    start = next(
        (i for i, l in enumerate(lines) if re.match(r'^(on|"on"):', l)),
        None,
    )
    if start is None:
        return []
    end = len(lines)
    for i in range(start + 1, len(lines)):
        line = lines[i]
        if line and not line[0].isspace() and not _is_comment(line):
            end = i
            break
    return [l for l in lines[start:end] if not _is_comment(l)]


def _declares_event(path: Path, *events: str) -> bool:
    pattern = re.compile(r"^\s+(" + "|".join(events) + r"):")
    return any(pattern.match(l) for l in on_block(path)[1:])


def triggers_on_merge_group(path: Path) -> bool:
    """True iff the workflow's top-level `on:` block declares `merge_group`."""
    return _declares_event(path, "merge_group")


def runs_on_non_main_refs(path: Path) -> bool:
    """True iff the workflow can run on a ref that is not `refs/heads/main`.

    `merge_group` runs on `refs/heads/gh-readonly-queue/*`; `pull_request` runs on
    the PR head ref. Every OTHER trigger the repo's rust-cache lanes use resolves to
    main (`schedule` and `workflow_run` run on the default branch, `push` is filtered
    to `branches: [main]`, `workflow_dispatch` is driven from main) — with the single
    exception of release.yml's `push: tags:`, which runs on `refs/tags/v*` and where
    a main-only guard would stop the save entirely rather than relocate it. So this
    predicate deliberately does NOT key off `push`.
    """
    return _declares_event(path, "merge_group", "pull_request", "pull_request_target")


def step_body(lines: list[str], start: int) -> list[str]:
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


def steps_using(lines: list[str], marker: str) -> list[tuple[int, list[str]]]:
    """Every step whose `uses:` names `marker`, as (0-based line index, body)."""
    return [
        (i, step_body(lines, i))
        for i, l in enumerate(lines)
        if marker in l and "uses:" in l
    ]


def with_value(body: list[str], key: str) -> str | None:
    """The value of `key` inside the step's `with:` mapping, or None."""
    for line in body:
        stripped = line.strip()
        if _is_comment(line):
            continue
        if stripped.startswith(key):
            return stripped[len(key) :].strip()
    return None


def merge_group_workflows_with_rust_cache() -> list[Path]:
    return sorted(
        p
        for p in WORKFLOWS.glob("*.yml")
        if RUST_CACHE in p.read_text() and triggers_on_merge_group(p)
    )


def guarded_workflows_with_rust_cache() -> list[Path]:
    """Every rust-cache lane the save discipline applies to, minus the exemptions."""
    return sorted(
        p
        for p in WORKFLOWS.glob("*.yml")
        if RUST_CACHE in p.read_text()
        and runs_on_non_main_refs(p)
        and p.name not in EXEMPT
    )


class TestParserNonVacuity(unittest.TestCase):
    """The assertions below iterate over discovered sets. If discovery silently
    returns nothing, every other test in this file passes while checking NOTHING.
    Pin the discovery itself."""

    def test_ci_yml_is_discovered_as_a_merge_group_lane(self) -> None:
        names = [p.name for p in merge_group_workflows_with_rust_cache()]
        self.assertIn(
            "ci.yml",
            names,
            "ci.yml is THE merge-queue pole (research §2.1) and carries rust-cache "
            f"steps; the on:-block parser failed to see its merge_group trigger. Found: {names}",
        )

    def test_the_widened_pull_request_set_is_discovered(self) -> None:
        # #5166's whole delta is that these PR lanes are now in scope. If the widened
        # predicate regressed to the merge_group-only one, every assertion below would
        # still pass while checking only the three merge_group lanes.
        names = {p.name for p in guarded_workflows_with_rust_cache()}
        for wf in (
            "bench.yml",
            "formal-verification.yml",
            "fuzz.yml",
            "python.yml",
            "xpath-differential.yml",
            "zk-toolchain.yml",
        ):
            self.assertIn(
                wf,
                names,
                f"{wf} is a pull_request-triggered rust-cache lane brought into scope by "
                f"#5166 but the discovery predicate does not see it. Found: {sorted(names)}",
            )

    def test_merge_group_lanes_remain_a_subset_of_the_widened_set(self) -> None:
        # The widening must not have LOST the sq-6vshe.15 guarantee.
        widened = {p.name for p in guarded_workflows_with_rust_cache()}
        for p in merge_group_workflows_with_rust_cache():
            self.assertIn(
                p.name,
                widened,
                f"{p.name} triggers on merge_group but fell out of the widened set — "
                "#5166 must extend sq-6vshe.15's scope, never narrow it.",
            )

    def test_main_only_lanes_are_not_dragged_in(self) -> None:
        # The counterpart non-vacuity check: a predicate that returned True for
        # everything would "pass" the test above while demanding a no-op guard on the
        # nightly lanes and a save-killing guard on the tag-driven release lane.
        names = {p.name for p in guarded_workflows_with_rust_cache()}
        for wf, why in (
            ("release.yml", "runs off refs/tags/v*, where a main-only guard stops the save"),
            ("kani.yml", "schedule + workflow_dispatch only — github.ref is already main"),
            ("miri.yml", "schedule + workflow_dispatch only — github.ref is already main"),
            ("pkg-ingest.yml", "push is filtered to branches: [main]"),
            ("selection-alarm.yml", "workflow_run filtered to main"),
        ):
            self.assertNotIn(
                wf,
                names,
                f"{wf} was pulled into the save-discipline set, but it {why}. The "
                "predicate is over-broad (#5166 header).",
            )

    def test_removed_merge_group_comment_is_not_a_trigger(self) -> None:
        # zk-toolchain.yml documents in PROSE that merge_group was removed. If the
        # parser counted comments, it would be in the set and this suite would demand
        # save-if on a lane that never runs on a queue ref.
        zk = WORKFLOWS / "zk-toolchain.yml"
        self.assertTrue(zk.exists(), "fixture-by-reference vanished; re-point this test")
        self.assertIn("merge_group", zk.read_text(), "prose reference gone; re-point")
        self.assertFalse(
            triggers_on_merge_group(zk),
            "zk-toolchain.yml only MENTIONS merge_group in a comment — the on:-block "
            "parser is matching comments.",
        )

    def test_step_walker_finds_every_rust_cache_step(self) -> None:
        for wf in guarded_workflows_with_rust_cache():
            text = wf.read_text()
            found = steps_using(text.split("\n"), RUST_CACHE)
            self.assertEqual(
                len(found),
                text.count(RUST_CACHE),
                f"{wf.name}: the step walker saw {len(found)} rust-cache steps but the "
                f"file names the action {text.count(RUST_CACHE)} times — a step shape the "
                "walker does not understand would be silently unchecked.",
            )


class TestCacheSaveDiscipline(unittest.TestCase):
    def test_every_non_main_rust_cache_step_saves_on_main_only(self) -> None:
        for wf in guarded_workflows_with_rust_cache():
            lines = _lines(wf)
            for idx, body in steps_using(lines, RUST_CACHE):
                where = f"{wf.name}:{idx + 1}"
                got = with_value(body, SAVE_IF_KEY)
                self.assertIsNotNone(
                    got,
                    f"{where}: rust-cache step in a merge_group/pull_request-triggered "
                    f"workflow has no `save-if`. Saves from a `gh-readonly-queue/*` ref are "
                    f"unrestorable (the ref is deleted at merge) and saves from a PR head "
                    f"are scoped to that branch alone, LRU-evicting the main entries every "
                    f"branch does restore — add "
                    f"`save-if: {SAVE_IF_VALUE}` (sq-6vshe.15 lever 2, widened by #5166).",
                )
                self.assertEqual(
                    got,
                    SAVE_IF_VALUE,
                    f"{where}: `save-if` must be exactly `{SAVE_IF_VALUE}`. A bare `false` "
                    "also stops main from ever SEEDING a cache; a wider condition lets the "
                    "dead queue-ref save back in.",
                )

    def test_restore_is_never_disabled(self) -> None:
        # `save-if` gates SAVING only — the whole design depends on queue refs and PR
        # heads still RESTORING main's entry. `lookup-only` would break that silently
        # (a cache "hit" that unpacks nothing).
        for wf in guarded_workflows_with_rust_cache():
            lines = _lines(wf)
            for idx, body in steps_using(lines, RUST_CACHE):
                self.assertIsNone(
                    with_value(body, "lookup-only:"),
                    f"{wf.name}:{idx + 1}: rust-cache `lookup-only` would disable RESTORE; "
                    "sq-6vshe.15 restricts SAVING only.",
                )


class TestExemptionsStayJustified(unittest.TestCase):
    """An exemption list is a hole. Pin the preconditions that make each entry sound,
    so the hole closes automatically the moment they stop holding."""

    def test_exempt_lanes_exist_and_still_use_rust_cache(self) -> None:
        for name in EXEMPT:
            path = WORKFLOWS / name
            self.assertTrue(
                path.exists(), f"{name} is exempted but no longer exists; drop the entry."
            )
            self.assertIn(
                RUST_CACHE,
                path.read_text(),
                f"{name} is exempted from a rust-cache rule but has no rust-cache step; "
                "drop the entry.",
            )

    def test_mutants_diff_still_has_no_main_ref_run_to_seed_its_key(self) -> None:
        # The exemption rests on TWO facts. If either changes, a main-scoped entry
        # becomes reachable, the guard stops being save-destroying, and the lane must
        # rejoin the discipline. Both are asserted so the exemption cannot go stale.
        path = WORKFLOWS / "mutants-diff.yml"
        events = [
            l.strip().split(":")[0]
            for l in on_block(path)[1:]
            if re.match(r"^  \S", l)
        ]
        self.assertEqual(
            events,
            ["pull_request"],
            f"mutants-diff.yml now triggers on {events}, not pull_request alone. If any "
            "of those runs on refs/heads/main it seeds the cache key, so "
            f"`save-if: {SAVE_IF_VALUE}` is now correct here — add it and remove the "
            "EXEMPT entry (#5166).",
        )
        body = steps_using(_lines(path), RUST_CACHE)[0][1]
        for key in ("key:", "shared-key:"):
            self.assertIsNone(
                with_value(body, key),
                f"mutants-diff.yml's rust-cache step now sets `{key}`. If that key is "
                "also written by a lane that runs on main, this lane can restore the "
                f"main-scoped entry and must carry `save-if: {SAVE_IF_VALUE}` like every "
                "other PR lane — re-evaluate the EXEMPT entry (#5166).",
            )


class TestNextestArchiveDiet(unittest.TestCase):
    def _upload_step(self) -> list[str]:
        lines = _lines(CI_YML)
        for idx, body in steps_using(lines, UPLOAD_ARTIFACT):
            if with_value(body, "name:") == NEXTEST_ARCHIVE_ARTIFACT:
                return body
        self.fail(
            f"ci.yml has no upload-artifact step named `{NEXTEST_ARCHIVE_ARTIFACT}` — "
            "the build-once contract (sq-vyxy) is gone or renamed; re-point this test."
        )

    def test_upload_does_not_recompress_the_zstd_archive(self) -> None:
        self.assertEqual(
            with_value(self._upload_step(), "compression-level:"),
            "0",
            "the nextest archive is already zstd-compressed, so upload-artifact's default "
            "DEFLATE-6 pass costs CPU on the merge-queue critical path for a ~0 size delta. "
            "Set `compression-level: 0` (sq-6vshe.15 lever 1).",
        )

    def test_uploaded_path_is_the_file_nextest_archive_writes(self) -> None:
        # Content parity, half 1: the diet must never change WHICH file ships.
        text = CI_YML.read_text()
        archive_files = set(_ARCHIVE_FILE_ARG.findall(text))
        self.assertTrue(
            archive_files,
            "ci.yml no longer passes `--archive-file` to cargo nextest; re-point this test.",
        )
        path = with_value(self._upload_step(), "path:")
        self.assertIn(
            path,
            archive_files,
            f"the uploaded path {path!r} is not one of the `--archive-file` targets "
            f"{sorted(archive_files)} — the shards would download something other than "
            "the archive that was just built.",
        )

    def test_the_test_shards_download_the_same_artifact_name(self) -> None:
        # Content parity, half 2: producer and consumers must agree on the name.
        lines = _lines(CI_YML)
        consumers = [
            idx
            for idx, body in steps_using(lines, DOWNLOAD_ARTIFACT)
            if with_value(body, "name:") == NEXTEST_ARCHIVE_ARTIFACT
        ]
        self.assertTrue(
            consumers,
            f"nothing in ci.yml downloads `{NEXTEST_ARCHIVE_ARTIFACT}` — the build-once "
            "archive would be uploaded and never consumed.",
        )


class TestSuiteIsWiredIntoCi(unittest.TestCase):
    """A structural test that never runs is a comment. Pin its own call site."""

    def test_docs_quality_invokes_this_suite(self) -> None:
        dq = (WORKFLOWS / "docs-quality.yml").read_text()
        self.assertIn(
            "scripts/tests/test_mergequeue_cache_posture.py",
            dq,
            "this suite must be invoked by docs-quality.yml or it silently stops gating.",
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
