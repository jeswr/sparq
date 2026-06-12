#!/usr/bin/env python3
"""Fail when a private constrained Noir helper has fewer than two call sites."""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_ROOT = REPO_ROOT / "src"
MIN_PRIVATE_USES = 2

FUNCTION_PATTERN = re.compile(
    r"^\s*"
    r"(?:(pub(?:\([^)]*\))?)\s+)?"
    r"(?:(comptime)\s+)?"
    r"(?:(unconstrained)\s+)?"
    r"fn\s+([A-Za-z_][A-Za-z0-9_]*)\b"
)


@dataclass(frozen=True)
class FunctionDef:
    path: Path
    line: int
    name: str
    unconstrained: bool
    attributes: tuple[str, ...]


def strip_comments_and_strings(source: str) -> str:
    source = re.sub(r"//.*", "", source)
    return re.sub(r'"(?:\\.|[^"\\])*"', '""', source)


def source_files() -> list[Path]:
    return sorted(SRC_ROOT.rglob("*.nr"))


def find_private_functions(path: Path, source: str) -> list[FunctionDef]:
    functions: list[FunctionDef] = []
    brace_depth = 0
    pending_attributes: list[str] = []

    for line_number, line in enumerate(source.splitlines(), start=1):
        match = FUNCTION_PATTERN.match(line)
        if match and brace_depth == 0:
            visibility, _comptime, unconstrained, name = match.groups()
            if not visibility:
                functions.append(
                    FunctionDef(
                        path=path,
                        line=line_number,
                        name=name,
                        unconstrained=unconstrained is not None,
                        attributes=tuple(pending_attributes),
                    )
                )
            pending_attributes = []
        elif brace_depth == 0 and line.strip().startswith("#"):
            pending_attributes.append(line.strip())
        elif line.strip():
            pending_attributes = []

        brace_depth += line.count("{") - line.count("}")

    return functions


def call_count(name: str, searchable_source: str) -> int:
    matches = re.findall(rf"(?<![A-Za-z0-9_]){re.escape(name)}(?![A-Za-z0-9_])", searchable_source)
    return max(0, len(matches) - 1)


def lint() -> int:
    files = source_files()
    raw_sources = {path: path.read_text() for path in files}
    searchable_source = "\n".join(strip_comments_and_strings(source) for source in raw_sources.values())
    failures: list[tuple[FunctionDef, int]] = []

    for path, source in raw_sources.items():
        clean_source = strip_comments_and_strings(source)
        for function in find_private_functions(path, clean_source):
            if function.unconstrained:
                continue
            if any(attribute.startswith("#[test") for attribute in function.attributes):
                continue

            uses = call_count(function.name, searchable_source)
            if uses < MIN_PRIVATE_USES:
                failures.append((function, uses))

    if not failures:
        return 0

    print("Private constrained Noir helpers must have at least two call sites.", file=sys.stderr)
    print("Inline single-use helpers or make intentionally shared APIs public/pub(crate).", file=sys.stderr)
    print(file=sys.stderr)
    for function, uses in failures:
        relative_path = function.path.relative_to(REPO_ROOT)
        print(f"{relative_path}:{function.line}: {function.name} has {uses} call site(s)", file=sys.stderr)

    return 1


if __name__ == "__main__":
    raise SystemExit(lint())
