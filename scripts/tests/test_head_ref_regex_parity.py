#!/usr/bin/env python3
# [OPUS-5] Cross-check pin for the registry's worker head-ref admission regex
# `^sparq-agent/issue-([1-9][0-9]*)-`. 🤖 SPARQ agent. sparq-org/sparq#5462.
#
# THE PROBLEM. That pattern is the registry's own `enumerate_review_items` admission
# test — "is this PR one of the fleet's worker PRs?" — and it is REPLICATED verbatim in
# several in-repo scripts. The replication is deliberate, not sloppy: the consumers do
# not all have each other on disk — review-alarm.yml sparse-checks-out ONLY
# `scripts/review_lane_alarm.py` (nothing else in scripts/ exists in that job), and
# ready-issues.py runs from the REGISTRY's dispatch clone — so there is no one module
# all of them could import. But replication with no binding check is exactly the defect
# class that produced #4677: two components disagreeing about which PRs are visible,
# one labelling a PR the other never enumerates.
#
# THE PIN, in two halves; the second is the load-bearing one.
#
#   1. PARITY — every DECLARED copy below compiles to a byte-identical pattern string.
#      Loosen one copy (say, to `([0-9]+)`, which admits `issue-0-` and `issue-007-`)
#      and this reds — both on the pattern string and on the ref matrix below it.
#   2. COMPLETENESS — an AST sweep of the production scripts finds every
#      `<NAME> = re.compile(<literal mentioning sparq-agent/issue>)` and asserts the
#      discovered set equals the declared set. Without this half the suite would pin
#      only the copies that existed the day it was written, and copy N+1 — the actual
#      failure mode #5462 reports — would land unbound and silent.
#
# WHAT #5462 REPORTS vs WHAT IS HERE. #5462 counts FOUR copies and names a fourth in
# scripts/verdict-bridge.py pinned by test_verdict_bridge.py. Neither is on main as of
# this commit — verdict-bridge.py compiles no head-ref regex at all (it binds a verdict
# to a 40-hex head SHA instead), so DECLARED_COPIES holds the three that actually exist.
# Half 2 is precisely what makes that count self-maintaining: if verdict-bridge.py ever
# does grow one, the sweep reds until it is declared here.
#
# HONEST LIMITS of half 2: it recognises a copy expressed as a module-level
# `re.compile(...)` of a STRING LITERAL that mentions `sparq-agent/issue`. A copy built
# by `str.startswith`, by f-string assembly, or by splitting the ref would not be
# discovered, and a broader-but-different predicate (`^sparq-agent/`, as used by
# batch-merge.py's WORKER_BRANCH_RE and pr-backlog.py's SPARQ_AGENT_BRANCH_RE) is
# deliberately NOT in scope — those answer "is this any agent branch?", not "which
# issue does this worker PR target?".
#
# Stdlib (ast, unittest) plus PyYAML for the call-site check — already a docs-quality
# dependency. No gh, no git, no network. Run:
#   python3 scripts/tests/test_head_ref_regex_parity.py

from __future__ import annotations

import ast
import importlib.util
import sys
import unittest
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
SCRIPTS = REPO_ROOT / "scripts"
DOCS_QUALITY = REPO_ROOT / ".github" / "workflows" / "docs-quality.yml"

# The registry's `enumerate_review_items` admission regex, verbatim. If the REGISTRY
# ever loosens its gate, every copy below plus this constant must move in the same
# breath — a partial update is the drift this suite exists to catch.
CANONICAL = r"^sparq-agent/issue-([1-9][0-9]*)-"

# Every in-repo copy, as (script filename, module-level symbol), with the consumer that
# makes the copy necessary rather than a shared import.
DECLARED_COPIES = {
    ("review_lane_alarm.py", "REGISTRY_HEAD_REF_RE"),  # review-alarm.yml
    ("ready-issues.py", "_LINK_HEAD"),  # the registry's dispatch tick
    ("batch-merge.py", "WORKER_ISSUE_RE"),  # batch-merge.yml
}

# Half 2 sweeps the production scripts. scripts/tests/ is excluded: a test that compiles
# its own throwaway copy is a fixture, not a live admission decision.
EXCLUDED_DIRS = {"tests", "__pycache__"}


def _load(filename: str):
    """Import a hyphen-named script by path, the way this repo's suites do."""
    path = SCRIPTS / filename
    spec = importlib.util.spec_from_file_location(f"parity_{path.stem}", path)
    assert spec is not None and spec.loader is not None, f"cannot load {path}"
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def _production_scripts() -> list[Path]:
    return sorted(
        p
        for p in SCRIPTS.rglob("*.py")
        if not EXCLUDED_DIRS & set(p.relative_to(SCRIPTS).parts[:-1])
    )


def _is_re_compile(node: ast.AST) -> bool:
    return (
        isinstance(node, ast.Call)
        and isinstance(node.func, ast.Attribute)
        and node.func.attr == "compile"
        and isinstance(node.func.value, ast.Name)
        and node.func.value.id == "re"
    )


def discover_copies() -> dict[tuple[str, str], str]:
    """{(filename, symbol): pattern} for every `X = re.compile("...sparq-agent/issue...")`."""
    found: dict[tuple[str, str], str] = {}
    for path in _production_scripts():
        try:
            tree = ast.parse(path.read_text(encoding="utf-8"))
        except SyntaxError:  # pragma: no cover - a broken script is another gate's job
            continue
        for node in ast.walk(tree):
            if not isinstance(node, (ast.Assign, ast.AnnAssign)):
                continue
            if node.value is None or not _is_re_compile(node.value):
                continue
            args = node.value.args
            if not args or not isinstance(args[0], ast.Constant):
                continue
            pattern = args[0].value
            if not isinstance(pattern, str) or "sparq-agent/issue" not in pattern:
                continue
            targets = node.targets if isinstance(node, ast.Assign) else [node.target]
            for target in targets:
                if isinstance(target, ast.Name):
                    found[(path.name, target.id)] = pattern
    return found


class TestDeclaredCopiesAgree(unittest.TestCase):
    """Half 1 — every declared copy is byte-identical to the registry's admission regex."""

    def test_each_copy_is_byte_identical(self):
        for filename, symbol in sorted(DECLARED_COPIES):
            with self.subTest(script=filename, symbol=symbol):
                module = _load(filename)
                self.assertTrue(
                    hasattr(module, symbol),
                    f"{filename} no longer defines {symbol} — update DECLARED_COPIES",
                )
                self.assertEqual(getattr(module, symbol).pattern, CANONICAL)

    def test_the_copies_agree_on_the_refs_that_decide_visibility(self):
        # Byte-identity already implies this, but naming the decisions makes a future
        # "equivalent" rewrite of one copy prove itself rather than assert itself.
        cases = [
            ("sparq-agent/issue-2908-30221671021-1", "2908"),  # real worker head
            ("sparq-agent/issue-1-2-1", "1"),
            ("sparq-agent/issue-0-1-1", None),  # issue 0 does not exist
            ("sparq-agent/issue-007-1-1", None),  # no leading zeros
            ("sparq-agent/issue-42", None),  # the trailing `-` is required
            ("sparq-agent/fix-issue-42-1", None),  # anchored at the start
            ("fix/readiness-visibility-opus5", None),  # a local agent branch
        ]
        compiled = {
            (f, s): getattr(_load(f), s) for f, s in sorted(DECLARED_COPIES)
        }
        for ref, expected in cases:
            for key, regex in compiled.items():
                with self.subTest(ref=ref, copy=key):
                    match = regex.match(ref)
                    self.assertEqual(match.group(1) if match else None, expected)


class TestNoUndeclaredCopies(unittest.TestCase):
    """Half 2 — copy N+1 cannot land unbound."""

    def test_the_discovered_set_is_exactly_the_declared_set(self):
        discovered = discover_copies()
        undeclared = sorted(set(discovered) - DECLARED_COPIES)
        self.assertFalse(
            undeclared,
            "new in-repo copy of the worker head-ref regex is not pinned: "
            f"{undeclared}. Add it to DECLARED_COPIES in {Path(__file__).name} so it is "
            "cross-checked against the registry's admission regex.",
        )
        stale = sorted(DECLARED_COPIES - set(discovered))
        self.assertFalse(
            stale, f"declared copies no longer present: {stale} — prune DECLARED_COPIES"
        )

    def test_the_sweep_actually_reads_the_scripts(self):
        # Anti-vacuity: an empty or mis-rooted sweep would make the check above pass
        # trivially. The sweep must see the whole scripts/ tree and find the copies.
        scanned = _production_scripts()
        self.assertGreater(len(scanned), 50, "the scripts/ sweep found almost nothing")
        self.assertEqual(set(discover_copies()), DECLARED_COPIES)


class TestSuiteIsWiredIntoCI(unittest.TestCase):
    """Without this, the whole file could leave CI's reachable set unnoticed."""

    def test_docs_quality_invokes_this_suite_unconditionally(self):
        wf = yaml.safe_load(DOCS_QUALITY.read_text(encoding="utf-8"))
        found = False
        for job in wf["jobs"].values():
            for step in job.get("steps") or []:
                if "test_head_ref_regex_parity.py" in str(step.get("run", "")):
                    found = True
                    self.assertNotIn("if", step, "the suite must not be conditionally skipped")
                    self.assertNotIn("continue-on-error", step)
        self.assertTrue(found, "docs-quality.yml must invoke test_head_ref_regex_parity.py")


if __name__ == "__main__":
    unittest.main(verbosity=2, argv=[sys.argv[0], "-v"])
