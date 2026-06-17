#!/usr/bin/env python3
# [OPUS-4.8] Gate G6 — new-config/flag→docs (bead sq-ncvq.9, epic sq-ncvq).
# Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns).
#
# PROACTIVE / merge-time half of the maintenance flow-on system
# (research/maintenance-flow-on-automation-design.md §2.1, gate G6). It is a SUBSET
# of G2 (scripts/gate-api-skill.py) but covers CONFIG rather than `pub` items: a new
# `sparq-cli` / `sparq-server` CLI flag or `SPARQ_*` env var must be DOCUMENTED in
# the same change. G2 only sees `pub`-item signatures, so a flag added inside a
# hand-rolled arg-parser match arm (a string literal, not a `pub` symbol) slips
# right past it — that is exactly the gap G6 closes.
#
# RULE (G6): when a PR ADDS a new public config knob — a CLI flag literal
# (`"--flag"`) or a `SPARQ_*` environment variable token — to a sparq-cli /
# sparq-server `src/**` file, FAIL unless that knob is DOCUMENTED:
#   * it appears as ADDED text in a docs surface in the SAME diff (the crate's
#     own README.md, the sibling binding crate's README.md, or ANY
#     skills/**/SKILL.md), OR
#   * it is ALREADY documented on disk in one of those surfaces (a knob renamed or
#     rewired in code whose name was previously written down stays covered), OR
#   * the PR carries the `config-internal` escape-hatch label (design §2.1).
#
# WHY "added token", not "any src change" — mirrors G2's net-diff discipline
# (gate-api-skill.py): we read the per-file unified diff and keep only knob tokens
# that are NET-added (added and not also removed). A pure relocation of an existing
# flag/env handler within a file (the same `"--flag"` line removed once and added
# once) cancels out and never trips the gate; a comment that merely MENTIONS an
# existing flag is inert because that flag is already documented. Only a genuinely
# NEW knob — one whose token is not already in the docs and is freshly added in code
# — can fail.
#
# WHAT COUNTS AS A KNOB TOKEN:
#   * a long CLI flag: a double-quoted `"--xxx"` literal (lower-kebab), matching the
#     hand-rolled arg matchers in crates/sparq-{cli,server}/src/main.rs. Short flags
#     (`-h`) and the universal `--help` are NOT user config and are ignored.
#   * an environment variable: a `SPARQ_[A-Z0-9_]+` token.
# Both forms are extracted identically from code-diff lines and from docs (so the
# "documented?" test is a pure token-membership check — no format coupling).
#
# SCOPE: only crates/sparq-cli/src/** and crates/sparq-server/src/** are config
# surfaces (per the design's "(sparq-cli, sparq-server)"). A new flag/env added in
# any OTHER crate, a test, or a non-src file never reaches the knob check, so such a
# PR always passes — a CI-only / library-internal PR can never trip G6.
#
# DOCS SURFACES (where a knob is considered "documented"):
#   * crates/sparq-cli/README.md, crates/sparq-server/README.md
#   * any skills/**/SKILL.md
# These are exactly the surfaces the design names ("the matching SKILL.md / crate
# README documents it").
#
# DIFF SOURCE: git diff --name-status origin/<base>...HEAD (CI) to find changed +
# added files, plus the per-file unified diff (git diff origin/<base>...HEAD -- f)
# for the net-added knob tokens; or a fixture (--changed-files, one status-prefixed
# path per line) + injected diffs (in hermetic tests) so no live git is needed.
#
# EXIT: 0 when no new knob is added, or every new knob is documented, or the
# `config-internal` label is present; 1 (with a per-knob message) otherwise.
#
# Usage:
#   check-config-documented.py                          # CI: diff vs origin/<base>
#   check-config-documented.py --base main
#   check-config-documented.py --advisory               # warn-only soft-launch
#   check-config-documented.py --dry-run --changed-files files.txt [--labels-file l]
#
# stdlib-only.

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# Config surfaces: only these crates' src/** define user-facing knobs (design §2.1).
CONFIG_SRC_RE = re.compile(r"^crates/(sparq-cli|sparq-server)/src/.+")

# Docs surfaces where a knob counts as "documented".
DOC_READMES = (
    "crates/sparq-cli/README.md",
    "crates/sparq-server/README.md",
)
SKILL_PREFIX = "skills/"
SKILL_SUFFIX = "SKILL.md"

# Escape-hatch label (design §2.1).
CONFIG_INTERNAL_LABEL = "config-internal"

# A long CLI flag literal: a double-quoted "--lower-kebab" token. The hand-rolled
# matchers in main.rs compare argv against exactly these literals.
_FLAG_RE = re.compile(r'"(--[a-z][a-z0-9-]*)"')
# An environment variable: SPARQ_ + uppercase/underscore/digits.
_ENV_RE = re.compile(r"\bSPARQ_[A-Z0-9_]+\b")

# Flags that are NOT user config knobs (universal help; short -h handled elsewhere).
_FLAG_IGNORE = {"--help"}

_STATUS_RE = re.compile(r"^([A-Z])\d*\t(.*)$")


def parse_status_lines(lines: list[str]) -> tuple[list[str], list[str]]:
    """Split a `git diff --name-status`-style list into (all_changed, added).

    Each line is "<STATUS>\t<path>" (or "<STATUS>\t<old>\t<new>" for renames/
    copies) or a bare path (a generic change, not provably added). Returns
    (changed, added) where `added` holds A/C destination paths."""
    changed: list[str] = []
    added: list[str] = []
    for raw in lines:
        line = raw.rstrip("\n")
        if not line.strip():
            continue
        m = _STATUS_RE.match(line)
        if m:
            status, rest = m.group(1), m.group(2)
            path = rest.split("\t")[-1]
            changed.append(path)
            if status in ("A", "C"):
                added.append(path)
        else:
            changed.append(line)
    return changed, added


def _normalize_base(base: str) -> str:
    """Accept a bare branch ('main') or a full ref ('refs/heads/main', as the
    merge_group event supplies) and return the remote-tracking ref ('origin/main')."""
    base = base.strip()
    if base.startswith("refs/heads/"):
        base = base[len("refs/heads/") :]
    if base.startswith("origin/") or base == "HEAD":
        return base
    return f"origin/{base}"


def git_diff(base: str) -> tuple[list[str], list[str]]:
    """Return (changed, added) from git vs the base ref using a 3-dot diff."""
    ref = _normalize_base(base)
    try:
        out = subprocess.run(
            ["git", "diff", "--name-status", f"{ref}...HEAD"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    except subprocess.CalledProcessError as e:  # pragma: no cover - CI-only
        sys.stderr.write(f"error: git diff failed: {e.stderr}\n")
        sys.exit(2)
    return parse_status_lines(out.splitlines())


def knob_tokens(text: str) -> set[str]:
    """Extract every config-knob token (CLI flags + SPARQ_* env vars) from `text`.

    Used both on code-diff lines (to find what a PR introduces) and on docs (to
    test whether a knob is documented) — one extractor, so the membership test is
    format-agnostic."""
    flags = {f for f in _FLAG_RE.findall(text) if f not in _FLAG_IGNORE}
    envs = set(_ENV_RE.findall(text))
    return flags | envs


def scan_added_knobs(diff_lines: list[str]) -> set[str]:
    """Return knob tokens NET-added by a unified diff: present on an added (`+`)
    line and NOT on any removed (`-`) line. A pure relocation (same token removed
    once, added once) cancels; a comment that only mentions an unchanged flag is
    inert (no +/- line). File headers ('+++ '/'--- ') are skipped so a path is
    never mistaken for content."""
    added: set[str] = set()
    removed: set[str] = set()
    for line in diff_lines:
        if line.startswith("+++") or line.startswith("---"):
            continue
        if line.startswith("+"):
            added |= knob_tokens(line[1:])
        elif line.startswith("-"):
            removed |= knob_tokens(line[1:])
    return added - removed


def git_added_knobs(path: str, base: str) -> set[str]:
    """Net-added knob tokens for `path` read from git. Conservative on error
    (returns empty set — never a CI-only crash)."""
    ref = _normalize_base(base)
    try:
        out = subprocess.run(
            ["git", "diff", f"{ref}...HEAD", "--", path],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    except subprocess.CalledProcessError:  # pragma: no cover - CI-only
        return set()
    return scan_added_knobs(out.splitlines())


def documented_on_disk() -> set[str]:
    """Every knob token already written down in a docs surface in the worktree
    (the crate READMEs + all SKILL.md). A knob whose name is already here stays
    covered even when the PR only rewires it in code."""
    tokens: set[str] = set()
    for rel in DOC_READMES:
        p = REPO_ROOT / rel
        try:
            tokens |= knob_tokens(p.read_text(encoding="utf-8", errors="ignore"))
        except OSError:
            continue
    for sk in REPO_ROOT.glob("skills/**/SKILL.md"):
        try:
            tokens |= knob_tokens(sk.read_text(encoding="utf-8", errors="ignore"))
        except OSError:
            continue
    return tokens


def git_doc_added_knobs(changed: list[str], base: str) -> set[str]:
    """Knob tokens ADDED to any docs surface in this diff (so a knob documented in
    the SAME PR counts even if the README/SKILL did not exist before)."""
    docs = [
        p
        for p in changed
        if p in DOC_READMES
        or (p.startswith(SKILL_PREFIX) and p.endswith(SKILL_SUFFIX))
    ]
    tokens: set[str] = set()
    for p in docs:
        tokens |= git_added_knobs(p, base)
    return tokens


def config_src_changes(changed: list[str]) -> list[str]:
    """Changed paths under a config surface (sparq-cli/sparq-server src/**)."""
    return [p for p in changed if CONFIG_SRC_RE.match(p)]


def evaluate(
    changed: list[str],
    labels: list[str],
    base: str,
    *,
    code_knobs: dict[str, set[str]] | None = None,
    doc_added: set[str] | None = None,
    doc_disk: set[str] | None = None,
) -> tuple[bool, list[tuple[str, str]]]:
    """Return (ok, undocumented) where `undocumented` is [(knob, src_path)].

    ok=True (PASS) when: no new knob was added; OR every new knob is documented
    (added to docs in this diff OR already documented on disk); OR the
    `config-internal` label is present.

    The *_overrides let hermetic tests inject every git/disk fact:
      code_knobs : {src_path -> set(net-added knob tokens)}  (else read from git)
      doc_added  : knob tokens added to docs in this diff      (else read from git)
      doc_disk   : knob tokens already documented on disk       (else read disk)
    """
    src_paths = config_src_changes(changed)

    # Map each net-added knob to a source file that introduced it (for messaging).
    new_knobs: dict[str, str] = {}
    for p in src_paths:
        toks = (
            code_knobs[p]
            if code_knobs is not None and p in code_knobs
            else git_added_knobs(p, base)
        )
        for t in toks:
            new_knobs.setdefault(t, p)

    if not new_knobs:
        return True, []

    if CONFIG_INTERNAL_LABEL in labels:
        return True, []

    documented: set[str] = set()
    documented |= (
        doc_added if doc_added is not None else git_doc_added_knobs(changed, base)
    )
    documented |= doc_disk if doc_disk is not None else documented_on_disk()

    undocumented = sorted(
        (knob, src) for knob, src in new_knobs.items() if knob not in documented
    )
    return (not undocumented), undocumented


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="G6 new-config/flag→docs gate (sq-ncvq.9)."
    )
    ap.add_argument(
        "--base",
        default=os.environ.get("GATE_BASE_REF", "main"),
        help="base ref to diff against (origin/<base>); default 'main'.",
    )
    ap.add_argument(
        "--changed-files",
        help="hermetic input: a file of diff lines (status-prefixed or bare paths).",
    )
    ap.add_argument(
        "--labels-file",
        help="hermetic input: file of PR labels, one per line (for the escape hatch).",
    )
    ap.add_argument("--dry-run", action="store_true", help="never exit non-zero.")
    ap.add_argument(
        "--advisory", action="store_true", help="soft-launch: report but exit 0."
    )
    args = ap.parse_args(argv)

    if args.changed_files:
        lines = Path(args.changed_files).read_text(encoding="utf-8").splitlines()
        changed, _added = parse_status_lines(lines)
    else:
        changed, _added = git_diff(args.base)

    labels: list[str] = []
    if args.labels_file:
        labels = [
            ln.strip()
            for ln in Path(args.labels_file).read_text(encoding="utf-8").splitlines()
            if ln.strip()
        ]

    ok, undocumented = evaluate(changed, labels, args.base)

    if ok:
        print("G6 new-config/flag→docs: PASS — no undocumented new config knob.")
        return 0

    print("G6 new-config/flag→docs: FAIL")
    print("\n  These new config knobs were added in code but are documented in")
    print("  no crate README and no skills/**/SKILL.md:")
    for knob, src in undocumented:
        print(f"    - {knob}   (added in {src})")
    print(
        "\nA new public config key / CLI flag / SPARQ_* env var must be documented "
        "in the SAME change (research/maintenance-flow-on-automation-design.md "
        "§2.1, gate G6). Document each knob in the relevant crate README "
        "(crates/sparq-cli/README.md or crates/sparq-server/README.md) or the "
        "matching skills/<surface>/SKILL.md — or, if it is genuinely internal and "
        f"not user-facing, add the `{CONFIG_INTERNAL_LABEL}` label with a "
        "justification in the PR body."
    )

    if args.advisory or args.dry_run:
        print("\n(advisory/dry-run: not failing the build)")
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
