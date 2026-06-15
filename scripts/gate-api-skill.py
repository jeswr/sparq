#!/usr/bin/env python3
# [OPUS-4.8] Gate G2 — public-api→skill (bead sq-ncvq.5, epic sq-ncvq).
# Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns).
#
# PROACTIVE / merge-time half of the maintenance flow-on system
# (research/maintenance-flow-on-automation-design.md §2.1, gate G2). Complements
# the reactive engine (scripts/flow-on.py, PR #220): that engine mints a
# "sync SKILL.md" follow-on ISSUE after merge; THIS gate BLOCKS the PR before it
# merges so the docs never drift in the first place.
#
# RULE (G2): when a PR changes a PUBLIC SURFACE without touching any
# skills/**/SKILL.md in the same diff, FAIL and point at the AGENTS.md
# public-API → SKILL.md maintenance rule.
#
# A "public surface" is (matching AGENTS.md:44-51 and the reactive rule's
# `changed-public-feature-docs` paths in flow-on-rules.toml):
#   - any changed file under one of the binding/entry crates' `src/**`:
#       crates/sparq-cli/src/**, crates/sparq-server/src/**,
#       crates/sparq-py/src/**,  crates/sparq-wasm/src/**;
#   - a `pub` API change in a PUBLISHED Rust crate: a changed `crates/<x>/src/**`
#       file whose crate is NOT `publish = false` and whose diff adds/removes a
#       `pub` item (heuristic — see pub_api_changed()). The four binding crates
#       above are always in scope regardless of the `pub` heuristic.
#
# SUPPRESSION (mirrors flow-on.py's SKILL_PATHS exactly): the gate is satisfied
# the moment the SAME diff touches ANY skills/**/SKILL.md — that is the signal
# "the docs were synced in this change", identical to the reactive engine's
# suppression of its docs rule. This keeps the two halves consistent: a PR that
# the gate lets through is also a PR the reactive engine would NOT re-flag.
#
# ESCAPE HATCH (design §2.1): the per-PR label `skill-not-needed` (justified in
# the PR body) suppresses the gate. In CI the label list is passed via
# --labels-file; locally/in tests it is injected. (The reactive engine is
# orthogonal — it runs post-merge and is not a blocker.)
#
# DIFF SOURCE: git diff --name-only origin/<base>...HEAD (CI), or a fixture file
# (--changed-files, one path per line, optionally status-prefixed) for tests.
# To evaluate the `pub` heuristic for non-binding crates we read the unified
# diff (git diff origin/<base>...HEAD -- <file>) for added/removed `pub` lines;
# in hermetic mode the caller may inject this via pub_overrides.
#
# EXIT: 0 when no public surface changed, or a SKILL.md was touched, or the
# escape-hatch label is present; 1 (with the AGENTS.md pointer) otherwise.
#
# Usage:
#   gate-api-skill.py                        # CI: diff vs origin/<base>
#   gate-api-skill.py --base main
#   gate-api-skill.py --advisory             # warn-only soft-launch
#   gate-api-skill.py --dry-run --changed-files files.txt [--labels-file l.txt]
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

# Binding / entry-point crates: ANY src/** change here is a public-surface change
# (their entire src is the public contract — CLI flags, HTTP routes, Py/JS
# bindings). Mirrors flow-on-rules.toml `changed-public-feature-docs` when_paths.
BINDING_SURFACE_PREFIXES = (
    "crates/sparq-cli/src/",
    "crates/sparq-server/src/",
    "crates/sparq-py/src/",
    "crates/sparq-wasm/src/",
)

# Suppression signal — identical to flow-on.py's SKILL_PATHS: any SKILL.md under
# skills/ means the docs were synced in this diff.
SKILL_SUFFIX = "SKILL.md"
SKILL_PREFIX = "skills/"

# Escape-hatch label (design §2.1).
SKILL_NOT_NEEDED_LABEL = "skill-not-needed"

_STATUS_RE = re.compile(r"^([A-Z])\d*\t(.*)$")
_CRATE_SRC_RE = re.compile(r"^crates/([^/]+)/src/.+")


def parse_paths(lines: list[str]) -> list[str]:
    """Extract paths from a `git diff --name-only` / `--name-status` listing."""
    paths: list[str] = []
    for raw in lines:
        line = raw.rstrip("\n")
        if not line.strip():
            continue
        m = _STATUS_RE.match(line)
        if m:
            paths.append(m.group(2).split("\t")[-1])
        else:
            paths.append(line)
    return paths


def _normalize_base(base: str) -> str:
    """Accept a bare branch ('main') or a full ref ('refs/heads/main', as the
    merge_group event supplies in github.event.merge_group.base_ref) and return
    the remote-tracking ref to diff against ('origin/main')."""
    base = base.strip()
    if base.startswith("refs/heads/"):
        base = base[len("refs/heads/") :]
    if base.startswith("origin/") or base == "HEAD":
        return base
    return f"origin/{base}"


def git_changed(base: str) -> list[str]:
    ref = _normalize_base(base)
    try:
        out = subprocess.run(
            ["git", "diff", "--name-only", f"{ref}...HEAD"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    except subprocess.CalledProcessError as e:  # pragma: no cover - CI-only
        sys.stderr.write(f"error: git diff failed: {e.stderr}\n")
        sys.exit(2)
    return parse_paths(out.splitlines())


def crate_is_published(crate: str) -> bool:
    """True iff crates/<crate>/Cargo.toml does NOT declare `publish = false`."""
    cargo = REPO_ROOT / "crates" / crate / "Cargo.toml"
    try:
        text = cargo.read_text(encoding="utf-8")
    except OSError:
        # Unknown crate manifest — treat as published (strict default).
        return True
    return re.search(r"^\s*publish\s*=\s*false\b", text, re.MULTILINE) is None


def pub_api_changed(path: str, base: str) -> bool:
    """Heuristic: does the unified diff for `path` add or remove a `pub` item?

    A `pub fn/struct/enum/trait/const/...` (or `pub use`) line being added or
    removed signals a public-API surface change. Reads the per-file unified diff
    from git; on any error returns False (conservative — never a CI-only crash).

    [OPUS-4.8] RESTRICTED visibilities — `pub(crate)`, `pub(super)`,
    `pub(in …)` — are NOT part of the crate's public surface (AGENTS.md: public
    API == exported items), so they must NOT trip the gate. They are always
    written `pub(` with no space, so requiring whitespace after `pub`
    (`pub\\s`) matches exactly the exported forms (`pub fn`, `pub use`,
    `pub struct`, …) while excluding the restricted ones."""
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
        return False
    pub_re = re.compile(r"^[+-]\s*pub\s")
    for line in out.splitlines():
        if line.startswith("+++") or line.startswith("---"):
            continue
        if (line.startswith("+") or line.startswith("-")) and pub_re.match(line):
            return True
    return False


def public_surface_changes(
    changed: list[str],
    base: str,
    *,
    pub_overrides: dict[str, bool] | None = None,
) -> list[str]:
    """Return the changed paths that constitute a public-surface change."""
    pub_overrides = pub_overrides or {}
    hits: list[str] = []
    for p in changed:
        if p.startswith(BINDING_SURFACE_PREFIXES):
            hits.append(p)
            continue
        m = _CRATE_SRC_RE.match(p)
        if m:
            crate = m.group(1)
            if not crate_is_published(crate):
                continue
            is_pub = pub_overrides.get(p)
            if is_pub is None:
                is_pub = pub_api_changed(p, base)
            if is_pub:
                hits.append(p)
    return hits


def skill_touched(changed: list[str]) -> bool:
    return any(p.startswith(SKILL_PREFIX) and p.endswith(SKILL_SUFFIX) for p in changed)


def evaluate(
    changed: list[str],
    labels: list[str],
    base: str,
    *,
    pub_overrides: dict[str, bool] | None = None,
) -> tuple[bool, list[str]]:
    """Return (ok, surface_hits). ok=True means the gate PASSES.

    PASS when: no public surface changed; OR a SKILL.md was touched; OR the
    `skill-not-needed` escape-hatch label is present."""
    hits = public_surface_changes(changed, base, pub_overrides=pub_overrides)
    if not hits:
        return True, []
    if skill_touched(changed):
        return True, hits
    if SKILL_NOT_NEEDED_LABEL in labels:
        return True, hits
    return False, hits


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="G2 public-api→skill gate (sq-ncvq.5)."
    )
    ap.add_argument(
        "--base",
        default=os.environ.get("GATE_BASE_REF", "main"),
        help="base ref to diff against (origin/<base>); default 'main'.",
    )
    ap.add_argument("--changed-files", help="hermetic input: file of diff lines.")
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
        changed = parse_paths(lines)
    else:
        changed = git_changed(args.base)

    labels: list[str] = []
    if args.labels_file:
        labels = [
            ln.strip()
            for ln in Path(args.labels_file).read_text(encoding="utf-8").splitlines()
            if ln.strip()
        ]

    ok, hits = evaluate(changed, labels, args.base)

    if ok:
        if hits:
            print(
                "G2 public-api→skill: PASS — public surface changed and a "
                "SKILL.md (or the skill-not-needed label) is present."
            )
        else:
            print("G2 public-api→skill: PASS — no public surface changed.")
        return 0

    print("G2 public-api→skill: FAIL")
    print("\n  These changed files are a public surface, but NO skills/**/SKILL.md")
    print("  was edited in the same PR:")
    for p in hits:
        print(f"    - {p}")
    print(
        "\nThe AGENTS.md public-API → SKILL.md maintenance rule (AGENTS.md "
        '"MAINTENANCE RULE (REQUIRED)" / "Public-API → SKILL.md maintenance rule") '
        "requires updating the matching skills/<surface>/SKILL.md in the SAME "
        "change. Edit the relevant SKILL.md (the surface map is in "
        "skills/SKILL.md), or — if no doc change is warranted — add the "
        f"`{SKILL_NOT_NEEDED_LABEL}` label with a justification in the PR body."
    )

    if args.advisory or args.dry_run:
        print("\n(advisory/dry-run: not failing the build)")
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
