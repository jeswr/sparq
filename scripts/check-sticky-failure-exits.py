#!/usr/bin/env python3
"""Reject exception handlers that can erase a failure already collected by a function.

This is deliberately a cheap, conservative structural lint.  It only flags a function
when both halves of the dangerous shape are visible in its AST:

* the function records a failure via ``*.append/add(...)`` or an assignment to a name
  containing ``failure``/``failures``; and
* one of that function's exception handlers directly returns 0 or exits with status 0.

Nested functions are analysed independently.  A narrow false positive can be documented
with ``sticky-failure-allow: <reason>`` on the handler's ``except`` line.
"""

# [GPT-5] Issue #3770 — repo-wide backstop for exit-zero swallowing sticky failures.

from __future__ import annotations

import argparse
import ast
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ALLOW = "sticky-failure-allow:"
FAILURE_WORDS = ("failure", "failures", "failed")


def _name(node: ast.AST) -> str:
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Attribute):
        return f"{_name(node.value)}.{node.attr}"
    if isinstance(node, ast.Subscript):
        return _name(node.value)
    return ""


def _is_zero(node: ast.AST | None) -> bool:
    # ``False == 0`` in Python, but ``return False`` is a failure result, not success.
    return (
        isinstance(node, ast.Constant)
        and type(node.value) is int
        and node.value == 0
    )


class FunctionFacts(ast.NodeVisitor):
    def __init__(self) -> None:
        self.records_failure = False
        self.zero_handlers: list[ast.ExceptHandler] = []

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        # A nested function owns its own outcome; do not mix its facts into the parent.
        return

    visit_AsyncFunctionDef = visit_FunctionDef

    def visit_Assign(self, node: ast.Assign) -> None:
        if any(any(word in _name(target).lower() for word in FAILURE_WORDS)
               for target in node.targets):
            self.records_failure = True
        self.generic_visit(node)

    def visit_AnnAssign(self, node: ast.AnnAssign) -> None:
        if any(word in _name(node.target).lower() for word in FAILURE_WORDS):
            self.records_failure = True
        self.generic_visit(node)

    def visit_AugAssign(self, node: ast.AugAssign) -> None:
        if any(word in _name(node.target).lower() for word in FAILURE_WORDS):
            self.records_failure = True
        self.generic_visit(node)

    def visit_Call(self, node: ast.Call) -> None:
        called = _name(node.func).lower()
        receiver, _, method = called.rpartition(".")
        if method in {"append", "add"} and any(
            word in receiver for word in FAILURE_WORDS
        ):
            self.records_failure = True
        self.generic_visit(node)

    def visit_ExceptHandler(self, node: ast.ExceptHandler) -> None:
        for statement in node.body:
            if isinstance(statement, ast.Return) and _is_zero(statement.value):
                self.zero_handlers.append(node)
            if isinstance(statement, ast.Raise):
                call = statement.exc
                if (
                    isinstance(call, ast.Call)
                    and _name(call.func) in {"SystemExit", "sys.exit", "exit"}
                    and call.args
                    and _is_zero(call.args[0])
                ):
                    self.zero_handlers.append(node)
            if (
                isinstance(statement, ast.Expr)
                and isinstance(statement.value, ast.Call)
                and _name(statement.value.func) in {"sys.exit", "exit"}
                and statement.value.args
                and _is_zero(statement.value.args[0])
            ):
                self.zero_handlers.append(node)
        self.generic_visit(node)


def inspect(path: Path) -> list[str]:
    source = path.read_text(encoding="utf-8")
    lines = source.splitlines()
    tree = ast.parse(source, filename=str(path))
    findings: list[str] = []
    for function in (
        node for node in ast.walk(tree)
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
    ):
        facts = FunctionFacts()
        for statement in function.body:
            facts.visit(statement)
        if not facts.records_failure:
            continue
        for handler in facts.zero_handlers:
            line = lines[handler.lineno - 1]
            if ALLOW not in line:
                findings.append(
                    f"{path.relative_to(ROOT) if path.is_relative_to(ROOT) else path}:"
                    f"{handler.lineno}: {function.name} records failures but an exception "
                    "handler exits 0; preserve sticky precedence or add a reasoned "
                    f"`{ALLOW} ...` waiver"
                )
    return findings


def self_test() -> None:
    cases = {
        "interleaved_bad.py": (
            "def sweep(items):\n"
            "    failures = []\n"
            "    for item in items:\n"
            "        try:\n"
            "            run(item)\n"
            "        except HardError as error:\n"
            "            failures.append(error)\n"
            "        except TransientError:\n"
            "            return 0\n"
            "    return 1 if failures else 0\n",
            1,
        ),
        "sticky_good.py": (
            "def sweep(items):\n"
            "    failures = []\n"
            "    transient = False\n"
            "    for item in items:\n"
            "        try:\n"
            "            run(item)\n"
            "        except HardError as error:\n"
            "            failures.append(error)\n"
            "        except TransientError:\n"
            "            transient = True\n"
            "    # precedence: collected-failure > transient-exhaustion > clean\n"
            "    return 1 if failures else 0\n",
            0,
        ),
        "waived.py": (
            "def sweep():\n"
            "    failures = []\n"
            "    try:\n"
            "        run()\n"
            "    except Error:  # sticky-failure-allow: isolated probe has no prior item\n"
            "        return 0\n"
            "    failures.append('later')\n",
            0,
        ),
    }
    with tempfile.TemporaryDirectory() as directory:
        for name, (source, expected) in cases.items():
            path = Path(directory) / name
            path.write_text(source, encoding="utf-8")
            actual = len(inspect(path))
            assert actual == expected, f"{name}: expected {expected}, got {actual}"
    print("sticky-failure exit lint self-test: interleaved sequence + controls OK")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("paths", nargs="*", type=Path)
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    paths = args.paths or sorted((ROOT / "scripts").rglob("*.py"))
    findings = [finding for path in paths for finding in inspect(path)]
    if findings:
        print("\n".join(findings), file=sys.stderr)
        return 1
    print(f"sticky-failure exit lint: {len(paths)} Python scripts clean")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
