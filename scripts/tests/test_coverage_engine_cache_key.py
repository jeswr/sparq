#!/usr/bin/env python3
# [OPUS-5] Hermetic pin for the coverage-engine dep-cache key (sparq-org/sparq#5231).
#
# WHAT WENT WRONG, and why a test exists at all. `coverage-engine-merge` recompiles the
# instrumented sparq-engine objects that the three `coverage-engine-run` partitions'
# downloaded .profraw are merged against, so it wants those partitions' warm dependency
# cache. It asked for it with
#
#     key: coverage-engine-run-1          # in job `coverage-engine-merge`
#
# against the partitions' `key: coverage-engine-run-${{ matrix.part }}`. That does not
# work. Swatinem/rust-cache documents — and its config.ts implements — `key` as an
# ADDITIONAL segment composed with the automatic job-id segment, whereas `shared-key`
# REPLACES that segment:
#
#     key = prefix-key ("v0-rust")
#     if shared-key:  key += "-" + shared-key
#     else:           key += "-" + key-input (if any) ; key += "-" + GITHUB_JOB
#                                                       (add-job-id-key defaults to true)
#
# So the merge job's prefix carried `-coverage-engine-merge` and the run job's carried
# `-coverage-engine-run`: different restore-key PREFIXES, hence no restore, hence a cold
# dep compile in the merge job — while the comment above the step claimed the opposite.
# Wall-clock only; the instrumented objects are rebuilt there either way.
#
# The fix is one word per step (`key:` -> `shared-key:`) and is INVISIBLE WHEN ABSENT:
# nothing goes red if a future edit reintroduces `key:`, CI just silently gets slower and
# a step comment silently starts lying again. Same reasoning as
# test_mergequeue_cache_posture.py, which pins the sibling `save-if` posture.
#
# Two layers:
#   1. PURE logic — a faithful model of rust-cache's key-prefix construction, asserting
#      BOTH directions: the old `key:` pairing genuinely does not collide, the new
#      `shared-key:` pairing genuinely does. This is the claim the YAML depends on.
#   2. LIVE invariant over the committed ci.yml — the two jobs' rust-cache steps carry the
#      SAME `shared-key` and NEITHER carries a `key:` (which would re-split the entry).
#      Plus a vacuous-pass guard: the walker must actually find both jobs and their steps.
#
# Hermetic: stdlib only (no PyYAML, no git, no network).
# Run:  python3 scripts/tests/test_coverage_engine_cache_key.py

from __future__ import annotations

import re
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
CI_YML = REPO_ROOT / ".github" / "workflows" / "ci.yml"

RUST_CACHE = "Swatinem/rust-cache@"
RUN_JOB = "coverage-engine-run"
MERGE_JOB = "coverage-engine-merge"

_JOB_HEADER = re.compile(r"^  ([A-Za-z0-9_-]+):\s*(#.*)?$")
_STEP_START = re.compile(r"^      - ")
_WITH_ENTRY = re.compile(r"^          ([A-Za-z0-9_-]+):\s*(.*?)\s*$")


# --------------------------------------------------------------------------- #
# Layer 1: rust-cache's key-prefix construction, modelled.
# --------------------------------------------------------------------------- #
def key_prefix(
    *,
    job: str,
    shared_key: str | None = None,
    key: str | None = None,
    prefix_key: str = "v0-rust",
    add_job_id_key: bool = True,
) -> str:
    """The `keyPrefix` half of Swatinem/rust-cache's config.ts.

    Everything appended after this point (runner OS/arch, the rust+env hash, the lockfile
    hash) is identical for two jobs that compile the same thing on the same runner image,
    so the prefix is exactly where a cross-job restore is won or lost: rust-cache restores
    by prefix-matching its restore key, and a differing prefix can never match.
    """
    out = prefix_key
    if shared_key:
        out += f"-{shared_key}"
    else:
        if key:
            out += f"-{key}"
        if job and add_job_id_key:
            out += f"-{job}"
    return out


class KeyConstruction(unittest.TestCase):
    def test_key_input_is_composed_with_the_job_id(self):
        self.assertEqual(
            key_prefix(job="coverage-engine-merge", key="coverage-engine-run-1"),
            "v0-rust-coverage-engine-run-1-coverage-engine-merge",
        )

    def test_shared_key_replaces_the_job_id(self):
        self.assertEqual(
            key_prefix(job="coverage-engine-merge", shared_key="coverage-engine"),
            "v0-rust-coverage-engine",
        )

    def test_the_bug_two_jobs_naming_one_key_do_not_collide(self):
        """#5231 itself: the shape ci.yml used to carry produced two distinct prefixes."""
        run = key_prefix(job=RUN_JOB, key="coverage-engine-run-1")
        merge = key_prefix(job=MERGE_JOB, key="coverage-engine-run-1")
        self.assertNotEqual(run, merge)
        # And not merely unequal — neither is a prefix of the other, so the restore-key
        # prefix match rust-cache falls back to cannot rescue it either.
        self.assertFalse(merge.startswith(run))
        self.assertFalse(run.startswith(merge))

    def test_the_fix_two_jobs_sharing_one_shared_key_do_collide(self):
        self.assertEqual(
            key_prefix(job=RUN_JOB, shared_key="coverage-engine"),
            key_prefix(job=MERGE_JOB, shared_key="coverage-engine"),
        )

    def test_shared_key_wins_when_both_are_given(self):
        # Belt and braces for the live assertion below: even a stray leftover `key:` next
        # to a `shared-key:` would not reintroduce the job segment. It is still banned
        # live, because it reads as if it did something.
        self.assertEqual(
            key_prefix(job=MERGE_JOB, shared_key="coverage-engine", key="stray"),
            "v0-rust-coverage-engine",
        )


# --------------------------------------------------------------------------- #
# Layer 2: the committed ci.yml. Line-walked — PyYAML would drop the comments these
# workflows carry, and stdlib-only keeps this runnable anywhere.
# --------------------------------------------------------------------------- #
def job_block(lines: list[str], job_id: str) -> list[str]:
    """The lines of one top-level job, header exclusive."""
    start = None
    for i, line in enumerate(lines):
        m = _JOB_HEADER.match(line)
        if m and m.group(1) == job_id:
            start = i + 1
            break
    if start is None:
        return []
    for j in range(start, len(lines)):
        if _JOB_HEADER.match(lines[j]):
            return lines[start:j]
    return lines[start:]


def rust_cache_withs(block: list[str]) -> list[dict[str, str]]:
    """The `with:` mapping of every Swatinem/rust-cache step in a job block."""
    out: list[dict[str, str]] = []
    i = 0
    while i < len(block):
        if not (_STEP_START.match(block[i]) and RUST_CACHE in block[i]):
            i += 1
            continue
        entries: dict[str, str] = {}
        j = i + 1
        while j < len(block) and not _STEP_START.match(block[j]):
            m = _WITH_ENTRY.match(block[j])
            if m:
                entries[m.group(1)] = m.group(2)
            j += 1
        out.append(entries)
        i = j
    return out


class LiveWorkflow(unittest.TestCase):
    def setUp(self):
        lines = CI_YML.read_text().split("\n")
        self.blocks = {j: job_block(lines, j) for j in (RUN_JOB, MERGE_JOB)}
        self.withs = {j: rust_cache_withs(b) for j, b in self.blocks.items()}

    def test_walker_is_not_vacuous(self):
        """If the walker stops finding the jobs or their cache steps, every assertion
        below would pass by finding nothing — the exact failure this file exists to
        prevent, so it is asserted against directly."""
        for job in (RUN_JOB, MERGE_JOB):
            self.assertTrue(self.blocks[job], f"ci.yml: job `{job}` not found")
            self.assertEqual(
                len(self.withs[job]),
                1,
                f"ci.yml: expected exactly one {RUST_CACHE} step in `{job}`, "
                f"found {len(self.withs[job])}",
            )

    def test_both_jobs_declare_the_same_shared_key(self):
        keys = {}
        for job in (RUN_JOB, MERGE_JOB):
            with_ = self.withs[job][0]
            self.assertIn(
                "shared-key",
                with_,
                f"ci.yml: `{job}`'s rust-cache step must set `shared-key` so the merge "
                f"job and the run partitions land on ONE cache entry (#5231)",
            )
            keys[job] = with_["shared-key"]
        self.assertEqual(
            keys[RUN_JOB],
            keys[MERGE_JOB],
            "ci.yml: coverage-engine-{run,merge} must share ONE dep cache; their "
            f"shared-keys diverged ({keys[RUN_JOB]!r} vs {keys[MERGE_JOB]!r})",
        )

    def test_neither_job_uses_the_key_input(self):
        """`key:` is what #5231 was: it composes with the job id, so it re-splits the
        entry these two jobs are meant to share while LOOKING like it names one."""
        for job in (RUN_JOB, MERGE_JOB):
            self.assertNotIn(
                "key",
                self.withs[job][0],
                f"ci.yml: `{job}`'s rust-cache step must not set `key:` — rust-cache "
                f"composes it with the job id, which is precisely the #5231 bug",
            )


if __name__ == "__main__":
    unittest.main(verbosity=2)
