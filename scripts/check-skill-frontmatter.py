#!/usr/bin/env python3
# [OPUS-4.8] HARD gate: validate the YAML frontmatter of every Agent-Skills
# SKILL.md (bead sq-56im).
#
# WHY: GitHub renders these skills/<surface>/SKILL.md files and parses their
# YAML frontmatter. An UNQUOTED colon inside a `description:` value (e.g.
# "...the sparq-text crate: build..." or "text:matches") makes the YAML parser
# read the second colon as a nested mapping key, producing the render error
#   "mapping values are not allowed in this context".
# That error was reported on skills/full-text-search/SKILL.md (and was present
# on geosparql / python / vector-search). This check catches the class of bug
# before it ships: it actually `yaml.safe_load`s each frontmatter block.
#
# WHAT it asserts for every skills/**/SKILL.md:
#   1. The file begins with a `---` line and has a closing `---` (a frontmatter
#      block exists).
#   2. The block parses as YAML (yaml.safe_load) without error.
#   3. The parsed value is a mapping (dict).
#   4. It has non-empty STRING `name` and `description` keys (the two fields the
#      agentskills.io format requires).
#
# It also WARNS (never fails) when `name` does not match the lowercase,
# hyphen-separated Agent-Skills convention or does not equal the parent
# directory name — these are nice-to-haves, kept advisory so the gate only
# fails on the real render-breaking defect.
#
# Exit 0 when every file is valid; exit 1 listing each offending file and its
# precise parse/validation error otherwise.
#
# Usage:
#   python3 scripts/check-skill-frontmatter.py            # glob skills/**/SKILL.md
#   python3 scripts/check-skill-frontmatter.py path/to/SKILL.md ...

from __future__ import annotations

import glob
import os
import re
import sys

try:
    import yaml  # PyYAML
except ImportError:  # pragma: no cover - exercised only without PyYAML
    sys.stderr.write(
        "error: PyYAML is required (pip install pyyaml). "
        "The frontmatter must be parsed with a real YAML loader.\n"
    )
    sys.exit(2)

# Agent-Skills `name` convention: lowercase letters/digits, hyphen-separated.
NAME_RE = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")


def extract_frontmatter(text: str) -> tuple[str | None, str | None]:
    """Return (frontmatter_block, error). Exactly one is non-None.

    The frontmatter is the text between the first two lines that are exactly
    `---`. The opening `---` must be the very first line of the file.
    """
    lines = text.splitlines()
    if not lines or lines[0].strip() != "---":
        return None, "no YAML frontmatter: file does not start with a '---' line"
    for i in range(1, len(lines)):
        if lines[i].strip() == "---":
            return "\n".join(lines[1:i]), None
    return None, "unterminated frontmatter: no closing '---' line found"


def check_file(path: str) -> tuple[list[str], list[str]]:
    """Return (errors, warnings) for a single SKILL.md."""
    errors: list[str] = []
    warnings: list[str] = []
    try:
        text = open(path, encoding="utf-8").read()
    except OSError as e:
        return [f"could not read: {e}"], []

    block, fm_err = extract_frontmatter(text)
    if fm_err is not None:
        return [fm_err], []

    try:
        data = yaml.safe_load(block)
    except yaml.YAMLError as e:
        # Flatten the (often multi-line) YAML error to a single readable line.
        msg = " ".join(str(e).split())
        return [f"YAML parse error: {msg}"], []

    if not isinstance(data, dict):
        return [f"frontmatter is not a mapping (parsed as {type(data).__name__})"], []

    for field in ("name", "description"):
        val = data.get(field)
        if val is None:
            errors.append(f"missing required '{field}' field")
        elif not isinstance(val, str):
            errors.append(f"'{field}' is not a string (got {type(val).__name__})")
        elif not val.strip():
            errors.append(f"'{field}' is empty")

    # ---- advisory: name convention + directory match ----
    name = data.get("name")
    if isinstance(name, str) and name.strip():
        if not NAME_RE.match(name):
            warnings.append(
                f"name '{name}' is not lowercase-hyphen (Agent-Skills convention)"
            )
        # The repo root skills/SKILL.md is an index, not a surface skill, so its
        # directory is `skills` rather than the skill name — only check nested ones.
        parent = os.path.basename(os.path.dirname(os.path.abspath(path)))
        if parent != "skills" and name != parent:
            warnings.append(f"name '{name}' != directory '{parent}'")

    return errors, warnings


def main() -> int:
    paths = sys.argv[1:] or sorted(glob.glob("skills/**/SKILL.md", recursive=True))
    if not paths:
        print("check-skill-frontmatter: no SKILL.md files found under skills/.")
        return 0

    n_bad = 0
    n_warn = 0
    for path in paths:
        errors, warnings = check_file(path)
        for w in warnings:
            n_warn += 1
            print(f"WARN  {path}: {w}")
        if errors:
            n_bad += 1
            for e in errors:
                print(f"FAIL  {path}: {e}")

    n_ok = len(paths) - n_bad
    print(
        f"\ncheck-skill-frontmatter: {n_ok}/{len(paths)} SKILL.md valid"
        f" ({n_bad} broken, {n_warn} warning(s))."
    )
    if n_bad:
        print(
            "::error::SKILL.md frontmatter is invalid YAML — GitHub will fail to "
            "render it. Quote any 'description'/'name' value containing a colon."
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
