#!/usr/bin/env python3
# [SONNET-4.6] sq-6vshe.15 — INSPECTION test for the merge-queue cache/artifact
# posture (research/ci-mergequeue-speedup-2026-07.md §3.2, lever 2 of the CI
# structural-speedup program sq-6vshe; extends sq-6vshe.5, which owns key schema
# and backend sizing).
#
# WHY a test at all: both properties this bead establishes are single YAML lines
# that are invisible when absent. Nothing goes red if a future edit drops them —
# CI just silently gets slower and the shared cache budget silently churns again.
# So the two lines are PINNED here, structurally, over every workflow that still
# triggers on `merge_group`:
#
#   1. CACHE-SAVE DISCIPLINE. Every `Swatinem/rust-cache` step in a
#      merge_group-triggered workflow must carry
#      `save-if: ${{ github.ref == 'refs/heads/main' }}`.
#      A save from a merge-queue entry is DEAD ON ARRIVAL: Actions cache scoping
#      makes an entry written from `refs/heads/gh-readonly-queue/<base>/pr-<N>-<sha>`
#      visible to that ref alone, and GitHub deletes that ref as soon as the entry
#      merges or is ejected — nothing can ever restore it, so the save is pure
#      post-step wall-clock on the queue's critical path. Branch-scoped saves also
#      churn the repo's shared 10 GB Actions-cache budget and LRU-evict the
#      main-scoped entries that every branch actually restores. This is sq-3sbrr's
#      doctrine, already live in feature-matrix.yml and the wasm lanes; sq-6vshe.15
#      closed the remaining gap (ci.yml's 20 cache steps + vectorized-feature-off).
#      RESTORE is deliberately untouched — `save-if` gates saving only, and queue
#      refs branch off main and read main's entry, which is where the hit comes from.
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
#   3. SCCACHE STAYS OFF THE QUEUE PATH. Bead item 3 (an sccache/GHA-backend A/B on
#      build-archive, issue #5164) is measure-first with a >=60 s median-win adoption
#      bar, and THAT MEASUREMENT HAS STILL NOT BEEN TAKEN — there is no verdict here,
#      neither adoption nor rejection. What ci.yml now carries is only the HARNESS that
#      makes the measurement possible: sccache is installed and wired as `RUSTC_WRAPPER`
#      ONLY when a maintainer dispatches the workflow with `sccache_ab: on`. That `if:`
#      guard on all FOUR arm steps — install, credential export, wrapper enable, stats —
#      and the input's `off` default, are the entire safety property. Drop the guard and
#      an unmeasured, unadopted compiler wrapper goes live on the merge-queue critical
#      path (precisely what the adoption bar exists to prevent), and the credential-export
#      step additionally leaks `ACTIONS_RUNTIME_TOKEN` into `GITHUB_ENV` on automatic
#      runs. All of it is pinned by TestSccacheArmIsDispatchOnly below — including a
#      mutation test proving the credential-export guard specifically is covered.
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

NEXTEST_ARCHIVE_ARTIFACT = "nextest-archive"
UPLOAD_ARTIFACT = "actions/upload-artifact@"
DOWNLOAD_ARTIFACT = "actions/download-artifact@"

_NEW_LIST_ITEM = re.compile(r"^ {0,6}- ")
_ARCHIVE_FILE_ARG = re.compile(r"--archive-file\s+(\S+)")

# A `steps:` list item, at the 6-space indent every job in these workflows uses.
_STEP_START = re.compile(r"^ {6}- (name|uses|run|if|with|env):")
# What it means to ENABLE sccache, as opposed to merely NAMING it. The always-on
# profile step carries `SCCACHE_ARM: ${{ ... sccache_ab ... }}` purely to LABEL its
# sample, and must not be caught: it sets no wrapper and runs no sccache binary.
_SCCACHE_ENABLER = re.compile(
    r"RUSTC_WRAPPER|SCCACHE_GHA_ENABLED|tool:\s*sccache|\bsccache\s+--"
)
# The fourth arm step names NEITHER the wrapper nor the binary: it is an
# `actions/github-script` block that re-exports the Actions cache-service credentials
# into `GITHUB_ENV` so the `sccache` binary can reach the GHA backend at all. The
# enabler pattern above cannot see it, so it is matched on the credential names
# themselves. Its guard is if anything MORE load-bearing than the others': an
# unguarded export puts `ACTIONS_RUNTIME_TOKEN` into `GITHUB_ENV`, and therefore into
# the environment of every later step in the job, on automatic runs.
_SCCACHE_CREDENTIAL_EXPORT = re.compile(
    r"ACTIONS_RESULTS_URL|ACTIONS_CACHE_URL|ACTIONS_RUNTIME_TOKEN"
)
# The one condition under which an sccache enabler may run. `workflow_dispatch` is the
# only trigger that can carry an input, so this expression is null — and the step skips
# — on every push / pull_request / merge_group run.
SCCACHE_ARM_GUARD = "github.event.inputs.sccache_ab == 'on'"


# --------------------------------------------------------------------------- #
# Minimal structural helpers. PyYAML would flatten the comments these workflows
# carry and is not installed in every environment, so we walk lines instead.
# --------------------------------------------------------------------------- #
def _lines(path: Path) -> list[str]:
    return path.read_text().split("\n")


def _is_comment(line: str) -> bool:
    return line.lstrip().startswith("#")


def triggers_on_merge_group(path: Path) -> bool:
    """True iff the workflow's top-level `on:` block declares `merge_group`.

    Comment lines are ignored: several workflows carry a prose note explaining
    that `merge_group` was REMOVED (2026-07-18 maintainer directive), and
    counting those would be exactly the false positive that makes this suite
    assert the property over lanes that never see a queue ref.
    """
    lines = _lines(path)
    start = next(
        (i for i, l in enumerate(lines) if re.match(r'^(on|"on"):', l)),
        None,
    )
    if start is None:
        return False
    end = len(lines)
    for i in range(start + 1, len(lines)):
        line = lines[i]
        if line and not line[0].isspace() and not _is_comment(line):
            end = i
            break
    block = lines[start:end]
    return any(
        re.match(r"^\s+merge_group:", l) for l in block if not _is_comment(l)
    )


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


def all_steps(lines: list[str]) -> list[tuple[int, list[str]]]:
    """Every `steps:` list item, as (0-based line index, body)."""
    return [
        (i, step_body(lines, i)) for i, l in enumerate(lines) if _STEP_START.match(l)
    ]


def _matches(body: list[str], pattern: re.Pattern[str]) -> bool:
    return any(pattern.search(l) for l in body if not _is_comment(l))


def sccache_enabler_steps(path: Path) -> list[tuple[int, list[str]]]:
    """Steps that actually turn sccache on (install it, wrap rustc, or invoke it)."""
    return [
        (idx, body)
        for idx, body in all_steps(_lines(path))
        if _matches(body, _SCCACHE_ENABLER)
    ]


def sccache_credential_export_steps(path: Path) -> list[tuple[int, list[str]]]:
    """Steps that hand the Actions cache-service credentials to the job environment."""
    return [
        (idx, body)
        for idx, body in all_steps(_lines(path))
        if _matches(body, _SCCACHE_CREDENTIAL_EXPORT)
    ]


def unguarded_sccache_arm_steps(lines: list[str]) -> list[tuple[int, str | None]]:
    """Every sccache ARM step whose `if:` is not exactly the dispatch-input guard.

    An arm step is one that enables sccache OR exports the backend credentials it
    needs. Takes LINES rather than a path so the mutation test below can run it over
    a deliberately-broken copy of ci.yml without writing to disk.
    """
    unguarded = []
    for idx, body in all_steps(lines):
        if not (
            _matches(body, _SCCACHE_ENABLER)
            or _matches(body, _SCCACHE_CREDENTIAL_EXPORT)
        ):
            continue
        got = with_value(body, "if:")
        if got != SCCACHE_ARM_GUARD:
            unguarded.append((idx, got))
    return unguarded


def merge_group_workflows_with_rust_cache() -> list[Path]:
    return sorted(
        p
        for p in WORKFLOWS.glob("*.yml")
        if RUST_CACHE in p.read_text() and triggers_on_merge_group(p)
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
        for wf in merge_group_workflows_with_rust_cache():
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
    def test_every_merge_group_rust_cache_step_saves_on_main_only(self) -> None:
        for wf in merge_group_workflows_with_rust_cache():
            lines = _lines(wf)
            for idx, body in steps_using(lines, RUST_CACHE):
                where = f"{wf.name}:{idx + 1}"
                got = with_value(body, SAVE_IF_KEY)
                self.assertIsNotNone(
                    got,
                    f"{where}: rust-cache step in a merge_group-triggered workflow has no "
                    f"`save-if`. Saves from a `gh-readonly-queue/*` ref are unrestorable "
                    f"(the ref is deleted at merge) — add "
                    f"`save-if: {SAVE_IF_VALUE}` (sq-6vshe.15 lever 2).",
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
        for wf in merge_group_workflows_with_rust_cache():
            lines = _lines(wf)
            for idx, body in steps_using(lines, RUST_CACHE):
                self.assertIsNone(
                    with_value(body, "lookup-only:"),
                    f"{wf.name}:{idx + 1}: rust-cache `lookup-only` would disable RESTORE; "
                    "sq-6vshe.15 restricts SAVING only.",
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


class TestSccacheArmIsDispatchOnly(unittest.TestCase):
    """sq-6vshe.15 item 3 (#5164). The A/B harness may exist; the VERDICT does not.
    Until a >=60 s median win is measured, sccache must be unreachable from any
    automatic trigger — so every ARM step (the ones that enable sccache, AND the one
    that exports the Actions cache credentials it needs) carries the dispatch-input
    guard, and the input defaults to `off`."""

    def test_the_enabler_walker_is_not_vacuous(self) -> None:
        found = sccache_enabler_steps(CI_YML)
        self.assertTrue(
            found,
            "no sccache enabler step found in ci.yml. Either the A/B harness was removed "
            "(then delete this class and the item-3 note in the header) or the step walker "
            "no longer understands its shape — in which case the guard below is unchecked.",
        )

    def test_the_credential_export_walker_is_not_vacuous(self) -> None:
        # The enabler pattern does NOT match this step (it names neither the wrapper nor
        # the binary), so without its own matcher the guard below would silently skip the
        # one arm step that handles a secret. Pin that it is discovered.
        found = sccache_credential_export_steps(CI_YML)
        self.assertTrue(
            found,
            "no Actions-cache-credential export step found in ci.yml. Either the A/B "
            "harness was removed (then delete this class and the item-3 note in the "
            "header) or the step walker no longer understands its shape — in which case "
            "the ACTIONS_RUNTIME_TOKEN export below is unguarded and unchecked.",
        )

    def test_every_sccache_arm_step_is_guarded_by_the_dispatch_input(self) -> None:
        for wf in sorted(WORKFLOWS.glob("*.yml")):
            if not triggers_on_merge_group(wf):
                continue
            for idx, got in unguarded_sccache_arm_steps(_lines(wf)):
                self.fail(
                    f"{wf.name}:{idx + 1}: this step arms the sccache A/B (it enables "
                    f"sccache, or exports the Actions cache credentials sccache needs) "
                    f"but its `if:` is {got!r}, not `{SCCACHE_ARM_GUARD}`. The workflow "
                    "triggers on merge_group, so an unguarded arm step puts an UNMEASURED "
                    "compiler wrapper on the merge-queue critical path — and an unguarded "
                    "credential export additionally writes ACTIONS_RUNTIME_TOKEN into "
                    "GITHUB_ENV for every later step in the job. sq-6vshe.15 item 3 adopts "
                    "sccache only on a measured >=60 s median win, and no such measurement "
                    "exists (research/ci-mergequeue-speedup-2026-07.md §3.2).",
                )

    def test_dropping_the_credential_export_guard_is_caught(self) -> None:
        # MUTATION. The check above is only worth its comment if it goes red when the
        # guard is actually removed — and the credential-export step is the one that
        # slipped past the enabler pattern. Delete ITS `if:` line in memory and assert
        # the detector names that exact step.
        lines = _lines(CI_YML)
        exports = sccache_credential_export_steps(CI_YML)
        self.assertEqual(
            len(exports), 1, f"expected exactly one credential export step, got {exports}"
        )
        start, body = exports[0]
        guard_offsets = [
            i for i, l in enumerate(body) if l.strip() == f"if: {SCCACHE_ARM_GUARD}"
        ]
        self.assertEqual(
            len(guard_offsets),
            1,
            "the credential export step does not carry exactly one "
            f"`if: {SCCACHE_ARM_GUARD}` line — the mutation below would be a no-op.",
        )
        mutated = lines[: start + guard_offsets[0]] + lines[start + guard_offsets[0] + 1 :]
        self.assertIn(
            start,
            [idx for idx, _ in unguarded_sccache_arm_steps(mutated)],
            "removing the `if:` guard from the ACTIONS_* credential export step did NOT "
            "make the guard check red — the check does not cover that step, so the "
            "ACTIONS_RUNTIME_TOKEN export could be armed on automatic runs unnoticed.",
        )

    def test_the_ab_input_defaults_to_off(self) -> None:
        lines = _lines(CI_YML)
        start = next(
            (i for i, l in enumerate(lines) if re.match(r"^ {6}sccache_ab:", l)), None
        )
        self.assertIsNotNone(
            start,
            "ci.yml declares no `sccache_ab` workflow_dispatch input, but a step is "
            "guarded on it — that guard would then be permanently unsatisfiable, or "
            "(worse) the guard was dropped with the input.",
        )
        block = lines[start + 1 : start + 12]
        defaults = [
            l.split(":", 1)[1].strip()
            for l in block
            if l.strip().startswith("default:") and not _is_comment(l)
        ]
        self.assertEqual(
            defaults,
            ['"off"'],
            "the `sccache_ab` input must default to the QUOTED string \"off\". Unquoted "
            "`off` is YAML 1.1 boolean false and would render as `false` in the dispatch "
            "UI; any other default arms the un-adopted experiment by default.",
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
