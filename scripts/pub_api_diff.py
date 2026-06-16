#!/usr/bin/env python3
# [OPUS-4.8] Shared `pub `-item diff helper (bead sq-l0a0, branch
# feat-flow-on-refine). Authored by Opus 4.8 (Fable unavailable; flag for
# re-review when Fable returns).
#
# WHY THIS MODULE EXISTS
# ----------------------
# Both halves of the maintenance flow-on system need to answer the SAME question
# — "does this unified diff NET-change a crate's PUBLIC Rust surface?" —
# distinguishing a genuine new/removed `pub ` export from inert
# comment / lint-attribute / pure-relocation edits:
#
#   * the PROACTIVE merge gate G2 (scripts/gate-api-skill.py) BLOCKS a PR that
#     changes a public surface without syncing a SKILL.md;
#   * the REACTIVE engine (scripts/flow-on.py, rule `changed-public-feature-docs`)
#     mints a "sync SKILL.md" follow-on ISSUE after merge for the same case.
#
# G2 originally tripped on ANY src change to a binding crate; PR #258 replaced
# that with a `pub `-item MULTISET diff (added vs removed signatures) so only a
# NET public-API change counts. The reactive flow-on rule was NOT given the same
# treatment, so it kept firing on comment-only / lint-attr-only / relocation
# diffs (dogfooded: PR #250 minted a redundant skill-sync follow-on for a
# comment+lint-attr-only diff — bead sq-l0a0). This module factors the G2 logic
# into ONE place both scripts import, so the two halves can never drift again.
#
# stdlib-only; no I/O — callers feed it unified-diff lines they already have.

from __future__ import annotations

import re

# [OPUS-4.8] A line is a PUBLIC-ITEM signature iff, after the leading diff marker
# (+/-) and any indentation, it begins with `pub ` (whitespace-separated) followed
# by one of the exported item keywords. Requiring whitespace after `pub` excludes
# restricted visibilities (`pub(crate)`/`pub(super)`/`pub(in …)`, all written
# `pub(` with no space) — those are NOT the crate's public surface. The
# item-keyword anchor further excludes a `pub` struct *field* (`pub name: T`),
# whose addition/removal is an internal-detail edit, not a new exported path —
# only the eight item forms below count.
PUB_ITEM_RE = re.compile(r"^[+-]\s*pub\s+(?:fn|struct|enum|trait|const|type|mod|use)\b")


def scan_pub_diff(diff_lines: list[str]) -> tuple[list[str], list[str]]:
    """Split unified-diff lines into (added, removed) public-item signatures.

    Each returned string is the diff line with its leading +/- marker stripped and
    inner whitespace collapsed, so an identical signature that is both added and
    removed (a pure relocation / re-indentation) compares equal across the two
    lists. File headers (`+++ b/…`, `--- a/…`) are skipped — they are never item
    signatures."""
    added: list[str] = []
    removed: list[str] = []
    for line in diff_lines:
        if line.startswith("+++") or line.startswith("---"):
            continue
        if not PUB_ITEM_RE.match(line):
            continue
        norm = " ".join(line[1:].split())
        (added if line[0] == "+" else removed).append(norm)
    return added, removed


def diff_has_net_pub_change(diff_lines: list[str]) -> bool:
    """True iff `diff_lines` (a unified diff) NET-changes a `pub `-exported item.

    A pure RELOCATION — the same signature added once and removed once (net-zero)
    — is NOT a public-API change. We compare added vs removed signatures as
    MULTISETS and report a change only when they differ: a genuinely NEW or
    DELETED export, or a signature whose text actually changed. Comment-only /
    attribute-only / non-`pub` edits yield two empty lists → no change."""
    added, removed = scan_pub_diff(diff_lines)
    return sorted(added) != sorted(removed)
