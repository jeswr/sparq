#!/usr/bin/env python3
# [OPUS-5] sparq-org/sparq#5235 — the GENERALISED one-file-scope-trap gate.
#
# WHAT TRAP. A GENERATED COMPANION is a file COMMITTED to git but DERIVED from other
# committed sources and pinned by a byte-comparing / regenerate-and-diff gate. Issue
# #2384 hit the feature-matrix instance: a bead scoped to `.github/feature-matrix.d/
# <crate>.yml` ALONE forbids the `scripts/tests/feature-matrix-legnames.golden.txt`
# regeneration it simultaneously requires, so the worker must DECLINE — or land a red
# gate. #2384 fixed that ONE surface. This script generalises the fix: the trap exists
# wherever the pair exists, and the cure is always the same two things —
#   (a) the EXACT regeneration command is documented next to the source, and
#   (b) the TWO-FILE SCOPE RULE is stated in a file the task DECOMPOSER actually reads
#       when it scopes the source.
# Neither is checkable as prose, so `.github/generated-companions.json` declares the
# pairs and this script enforces them.
#
# THE CHECKS (all fail-closed; exit 1 on any offence).
#   C1 RESOLVES  — every registered path resolves: the companion file EXISTS (existence
#                  only — this checker shells out to nothing, so it does not assert the
#                  file is git-TRACKED), every `sources` glob matches >=1 file, every
#                  `scope_anchors` glob matches >=1 file, and every in-repo path named by
#                  the `regen` command exists (so a renamed generator REDs instead of
#                  leaving a dead recipe behind).
#   C2 ANCHORED  — EVERY file matched by `scope_anchors` states the rule: it must contain
#                  the marker token `TWO-FILE` (case-insensitive) AND the literal
#                  companion path. Both, because either alone is satisfiable by accident —
#                  naming the path without the rule is what `sparq-canon.yml` did before
#                  #5235 (it said "a new leg also needs..." and omitted rename/remove),
#                  and the phrase without the path does not tell a decomposer WHICH second
#                  file to scope. This is the check with real teeth: a NEW feature-matrix
#                  fragment added without the SCOPE comment REDs here.
#   C3 COMPLETE  — every `scripts/tests/*.golden.*` file is a registered companion. This
#                  is the ratchet that keeps #5235's sweep from decaying: a future golden
#                  cannot land undeclared, with no recorded regen command and no anchor.
#
# WHAT THIS IS NOT. It does NOT regenerate anything and does NOT verify a companion is
# up to date — each pair's own gate already does that (test_feature_matrix_assemble.py,
# check-fast-fix-ring-guard.py, `gen-metric-labels.py --check`). This script only
# enforces that the pair is DECLARED and DOCUMENTED where a decomposer will see it. It
# is therefore deterministic, in-repo, no network, no build — milliseconds.
#
# Usage:
#   check-generated-companions.py              # check the live repo (default)
#   check-generated-companions.py --root DIR   # use DIR as repo root
#   check-generated-companions.py --self-test  # hermetic mutation table (no network/git)
#
# Exit 0 = clean; exit 1 = one or more offences.

from __future__ import annotations

import argparse
import json
import shlex
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
REGISTRY_REL = Path(".github") / "generated-companions.json"

# The token every scope anchor must carry, alongside the companion path. Kept short and
# greppable on purpose: `git grep -il two-file` enumerates the anchors.
MARKER = "two-file"

# C3's sweep glob — the surface issue #2384/#5235 names explicitly.
SWEEP_GLOBS = ("scripts/tests/*.golden.*",)

REQUIRED_FIELDS = ("sources", "regen", "pinned_by", "gate", "trigger", "scope_anchors")

# Path prefixes inside the repo that a `regen` command may name; a token starting with
# one of these is asserted to exist.
IN_REPO_PREFIXES = ("scripts/", "bench/", ".github/", "crates/", "site/", "compliance/")


def _glob(root: Path, pattern: str) -> list[Path]:
    """Resolve a registry path-or-glob against `root`, sorted for stable diagnostics."""
    if any(ch in pattern for ch in "*?["):
        return sorted(root.glob(pattern))
    p = root / pattern
    return [p] if p.exists() else []


def load_registry(root: Path) -> dict:
    path = root / REGISTRY_REL
    if not path.exists():
        raise SystemExit(f"REGISTRY-MISSING: {REGISTRY_REL} not found under {root}")
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise SystemExit(f"REGISTRY-MALFORMED: {REGISTRY_REL}: {exc}") from exc
    if not isinstance(data.get("companions"), dict):
        raise SystemExit(
            f"REGISTRY-MALFORMED: {REGISTRY_REL} has no `companions` object."
        )
    return data


def check_c1(root: Path, companion: str, entry: dict) -> list[str]:
    """Every registered path resolves."""
    out = []
    missing = [f for f in REQUIRED_FIELDS if f not in entry]
    if missing:
        out.append(
            f"C1 FIELDS: `{companion}` is missing required field(s) "
            f"{', '.join(missing)}. Every entry must record where it comes from "
            f"(`sources`), how to regenerate it (`regen`), what pins it (`pinned_by`), "
            f"which job REDs (`gate`), WHEN the two-file rule fires (`trigger`), and "
            f"where the rule is stated (`scope_anchors`)."
        )
        return out

    if not (root / companion).exists():
        out.append(
            f"C1 COMPANION: `{companion}` is registered in {REGISTRY_REL} but does not "
            f"exist. Delete the entry if the artifact was removed."
        )

    for key in ("sources", "scope_anchors"):
        for pattern in entry[key]:
            if not _glob(root, pattern):
                out.append(
                    f"C1 {key.upper()}: `{companion}` declares {key} pattern "
                    f"`{pattern}`, which matches no file. A stale pattern silently "
                    f"empties the C2 anchor check."
                )

    for token in shlex.split(entry["regen"], comments=False, posix=True):
        if token.startswith(IN_REPO_PREFIXES) and not (root / token).exists():
            out.append(
                f"C1 REGEN: `{companion}`'s regen command names `{token}`, which does "
                f"not exist. The recorded recipe is dead:\n    {entry['regen']}"
            )
    return out


def check_c2(root: Path, companion: str, entry: dict) -> list[str]:
    """Every scope-anchor file states the two-file rule AND names the companion."""
    out = []
    if "scope_anchors" not in entry:
        return out
    for pattern in entry["scope_anchors"]:
        for path in _glob(root, pattern):
            if not path.is_file():
                continue
            text = path.read_text(encoding="utf-8", errors="replace")
            rel = path.relative_to(root).as_posix()
            has_marker = MARKER in text.lower()
            has_path = companion in text
            if has_marker and has_path:
                continue
            lacks = []
            if not has_marker:
                lacks.append(f"the `{MARKER.upper()}` marker")
            if not has_path:
                lacks.append(f"the literal companion path `{companion}`")
            out.append(
                f"C2 ANCHOR: `{rel}` is a declared scope anchor for the generated "
                f"companion `{companion}` but lacks {' and '.join(lacks)}. A decomposer "
                f"reads this file when it scopes the source, so the two-file rule must "
                f"be stated HERE — otherwise a task gets scoped to the source alone and "
                f"forbids the regeneration it requires (issue #2384). State when it "
                f"fires and how to regenerate:\n    {entry.get('trigger', '')}\n"
                f"    {entry.get('regen', '')}"
            )
    return out


def check_c3(root: Path, registry: dict) -> list[str]:
    """Every swept generated artifact is registered."""
    out = []
    registered = set(registry["companions"])
    for pattern in SWEEP_GLOBS:
        for path in _glob(root, pattern):
            rel = path.relative_to(root).as_posix()
            if rel not in registered:
                out.append(
                    f"C3 UNREGISTERED: `{rel}` matches the swept generated-artifact "
                    f"pattern `{pattern}` but is not declared in {REGISTRY_REL}. Every "
                    f"checked-in generated companion needs an entry recording its "
                    f"sources, its exact regeneration command, the gate that pins it, "
                    f"and the files where its two-file scope rule is stated (#5235)."
                )
    return out


def check_root(root: Path) -> list[str]:
    registry = load_registry(root)
    offences: list[str] = []
    for companion, entry in sorted(registry["companions"].items()):
        c1 = check_c1(root, companion, entry)
        offences.extend(c1)
        if not c1:
            offences.extend(check_c2(root, companion, entry))
    offences.extend(check_c3(root, registry))
    return offences


# ---------------------------------------------------------------------------
# Hermetic self-test: build a miniature repo, then prove each check goes RED on
# the mutation it exists to catch. No network, no git, no real repo paths.
# ---------------------------------------------------------------------------

_ANCHOR_OK = (
    "# Fragment for a leg.\n"
    "# SCOPE: renaming a leg is a TWO-FILE change — this file AND\n"
    "# scripts/tests/demo.golden.txt. Regenerate, never hand-edit.\n"
)


def _build_fixture(root: Path) -> None:
    (root / ".github").mkdir(parents=True, exist_ok=True)
    (root / "scripts" / "tests").mkdir(parents=True, exist_ok=True)
    (root / "src.d").mkdir(parents=True, exist_ok=True)
    (root / "scripts" / "tests" / "demo.golden.txt").write_text("a\n", encoding="utf-8")
    (root / "scripts" / "gen-demo.py").write_text("# generator\n", encoding="utf-8")
    (root / "src.d" / "one.yml").write_text(_ANCHOR_OK, encoding="utf-8")
    (root / "src.d" / "two.yml").write_text(_ANCHOR_OK, encoding="utf-8")
    (root / ".github" / "generated-companions.json").write_text(
        json.dumps(
            {
                "companions": {
                    "scripts/tests/demo.golden.txt": {
                        "sources": ["src.d/*.yml"],
                        "regen": "python3 scripts/gen-demo.py --names",
                        "pinned_by": "t.py::test_golden",
                        "gate": "w.yml / job",
                        "trigger": "when the name set changes",
                        "scope_anchors": ["src.d/*.yml"],
                    }
                }
            }
        ),
        encoding="utf-8",
    )


def _self_test() -> int:
    # (label, mutate(root), expected offence prefix)
    table = [
        (
            "C1 companion deleted",
            lambda r: (r / "scripts" / "tests" / "demo.golden.txt").unlink(),
            "C1 COMPANION",
        ),
        (
            "C1 required field dropped",
            lambda r: _rewrite_entry(r, lambda e: e.pop("trigger")),
            "C1 FIELDS",
        ),
        (
            "C1 sources pattern matches nothing",
            lambda r: _rewrite_entry(r, lambda e: e.__setitem__("sources", ["gone/*.yml"])),
            "C1 SOURCES",
        ),
        (
            "C1 regen names a renamed generator",
            lambda r: _rewrite_entry(
                r, lambda e: e.__setitem__("regen", "python3 scripts/gen-moved.py")
            ),
            "C1 REGEN",
        ),
        (
            "C2 an anchor drops the TWO-FILE marker",
            lambda r: (r / "src.d" / "two.yml").write_text(
                "# see scripts/tests/demo.golden.txt\n", encoding="utf-8"
            ),
            "C2 ANCHOR",
        ),
        (
            "C2 an anchor drops the companion path",
            lambda r: (r / "src.d" / "two.yml").write_text(
                "# this is a TWO-FILE change\n", encoding="utf-8"
            ),
            "C2 ANCHOR",
        ),
        (
            "C2 a NEW unannotated source file appears",
            lambda r: (r / "src.d" / "three.yml").write_text("# leg\n", encoding="utf-8"),
            "C2 ANCHOR",
        ),
        (
            "C3 a new golden lands undeclared",
            lambda r: (r / "scripts" / "tests" / "other.golden.yml").write_text(
                "x\n", encoding="utf-8"
            ),
            "C3 UNREGISTERED",
        ),
    ]

    failures = 0
    with tempfile.TemporaryDirectory() as td:
        clean = Path(td) / "clean"
        clean.mkdir()
        _build_fixture(clean)
        base = check_root(clean)
        if base:
            print("SELF-TEST FAIL: the unmutated fixture is not clean:")
            for o in base:
                print("  " + o)
            failures += 1
        else:
            print("  ok   unmutated fixture is clean")

        for label, mutate, expect in table:
            case = Path(td) / ("case" + str(len(label)) + expect.replace(" ", ""))
            case.mkdir(parents=True, exist_ok=True)
            _build_fixture(case)
            mutate(case)
            got = check_root(case)
            if any(o.startswith(expect) for o in got):
                print(f"  ok   RED on: {label}  ->  {expect}")
            else:
                print(f"  FAIL no {expect} offence for: {label}; got {got}")
                failures += 1

    if failures:
        print(f"\nself-test: {failures} failure(s).")
        return 1
    print("\nself-test: all mutations red, clean fixture green.")
    return 0


def _rewrite_entry(root: Path, mutate) -> None:
    path = root / ".github" / "generated-companions.json"
    data = json.loads(path.read_text(encoding="utf-8"))
    mutate(data["companions"]["scripts/tests/demo.golden.txt"])
    path.write_text(json.dumps(data), encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", default=None, help="repo root (default: this checkout)")
    ap.add_argument("--self-test", action="store_true", help="hermetic mutation table")
    args = ap.parse_args()

    if args.self_test:
        return _self_test()

    root = Path(args.root).resolve() if args.root else REPO_ROOT
    offences = check_root(root)
    if offences:
        print(
            f"generated-companion registry check: {len(offences)} offence(s) "
            f"({REGISTRY_REL}).\n"
        )
        for o in offences:
            print("  " + o + "\n")
        return 1
    registry = load_registry(root)
    print(
        f"generated-companion registry check: OK "
        f"({len(registry['companions'])} companion(s) declared, anchored and swept)."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
