#!/usr/bin/env python3
# [SONNET-4.6] (sq-5owmc) Resolve the PR number for a `merge_group` event by
# parsing the merge-queue head ref — the deterministic, network-free primary path
# for the flow-on-gates G2/G6 escape-hatch label lookup.
#
# THE BUG THIS FIXES (cost PR #1542 two silent merge-queue ejections + ~4h stall):
# in `merge_group` context the G2 "Read PR labels" step resolved the PR via
# `gh api repos/.../commits/<MERGE_SHA>/pulls`. That lookup is unreliable in the
# merge queue: the merge-group head commit is a SYNTHETIC merge commit that is not
# associated with the PR the way `commits/<sha>/pulls` expects, and in the observed
# run the wired `MERGE_SHA` env came back EMPTY as well — so `pr-labels.txt` was
# empty, the `skill-not-needed` label never suppressed, G2 failed the whole GROUP,
# and the queue ejected the PR with NO failed run visible on the PR itself.
#
# THE FIX: the merge-queue branch name deterministically encodes the PR number:
#     refs/heads/gh-readonly-queue/<base_branch>/pr-<N>-<sha>
# (`github.event.merge_group.head_ref`). Parse <N> from that ref directly — no
# network, no synthetic-commit association guesswork. The workflow keeps the
# `commits/<sha>/pulls` API only as a SECONDARY fallback, and warns LOUDLY (never
# silently) if BOTH fail (fail-closed: the gate then runs unsuppressed).
#
# Pure + hermetic: `parse_pr_number_from_ref` does no I/O so the gates self-test
# suite (scripts/tests/test_gates.py) unit-tests it against synthetic refs.
#
# CLI:  python3 scripts/resolve-merge-group-pr.py "<head_ref>"
#   prints the PR number and exits 0 on a successful parse; exits 1 (no output)
#   when the ref is empty or not a recognisable gh-readonly-queue ref.

from __future__ import annotations

import re
import sys

# Anchor on the deterministic `gh-readonly-queue/` marker so an unrelated ref (or a
# base branch that happens to contain `pr-<n>-`) cannot match. The base branch may
# itself contain slashes (e.g. `release/v1`), so `.*` greedily consumes up to the
# LAST `/pr-<digits>-<hex>` segment, which is the PR + queue-head-sha suffix.
# `head_ref` may or may not carry the leading `refs/heads/` — the search is prefix
# agnostic.
_QUEUE_PR_RE = re.compile(r"gh-readonly-queue/.*/pr-(\d+)-[0-9a-fA-F]+")


def parse_pr_number_from_ref(ref: str | None) -> int | None:
    """Parse the PR number from a `gh-readonly-queue` merge-group head ref.

    Returns the PR number (int) or None when `ref` is empty / not a recognisable
    merge-queue ref. No I/O — safe to unit-test.

    NOTE (single-PR scope): sparq's merge queue validates one PR per group, so the
    ref carries exactly one `pr-<N>-<sha>` suffix. `.*` anchors the match on the
    LAST such segment, which is the PR under test. If batched (multi-PR) merge
    groups are ever enabled, the per-PR label semantics must be revisited — a batch
    ref would carry several `pr-<N>-` segments and label suppression would need to
    consider each PR independently.
    """
    if not ref:
        return None
    m = _QUEUE_PR_RE.search(ref.strip())
    if not m:
        return None
    return int(m.group(1))


def main(argv: list[str]) -> int:
    ref = argv[1] if len(argv) > 1 else ""
    num = parse_pr_number_from_ref(ref)
    if num is None:
        return 1
    print(num)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
