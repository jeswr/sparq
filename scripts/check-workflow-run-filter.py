#!/usr/bin/env python3
# [OPUS-5] jeswr/sparq#5654 — every `workflow_run:` trigger must declare a non-empty
# `workflows:` filter. 🤖 SPARQ agent.
#
# =============================================================================
# WHY — the HONEST UNCERTAINTY this guard closes
# =============================================================================
# The slotless `ci-summary` aggregator design (research/ci-gate-slotless-aggregation.md)
# proposes a `workflow_run`-triggered evaluator that publishes a commit status, and the
# first draft deliberately OMITTED the `workflows:` key so the evaluator would never
# hard-code a sibling-workflow list. That draft recorded the omission as an explicitly
# UNRESOLVED uncertainty. #5654 asked for it to be resolved before the resulting
# `ci-summary/status` context is added to the required branch-protection contexts.
#
# The evidence, gathered 2026-09-03, and what each source actually supports:
#
#   1. actionlint REJECTS the omission. Not a heuristic — it is hard-coded in
#      `rule_events.go`:
#          if hook == "workflow_run" {
#              if len(event.Workflows) == 0 {
#                  rule.Error(event.Pos, "no workflow is configured for \"workflow_run\" event")
#      so any repo that adopts actionlint reds on the construct immediately.
#
#   2. The SchemaStore `github-workflow.json` schema — what VS Code and most
#      YAML-schema linters validate against — does NOT require it. `workflow_run`
#      is `oneOf: [null, object]` with `workflows` an ordinary optional property
#      (`minItems: 1` only constrains it WHEN PRESENT). So the omission is
#      schema-VALID.
#
#   3. GitHub's own documentation never defines the omission's behaviour. The
#      workflow-syntax reference gives `on.workflow_run.<branches|branches-ignore>`
#      its own documented entry but gives `workflows` NO entry stating whether it is
#      required or what omitting it means, and EVERY example on both the syntax page
#      and the events-that-trigger-workflows page specifies `workflows:`.
#
# The resolution is NOT "actionlint is right and the schema is wrong". It is that
# sources 1 and 2 CONTRADICT each other and source 3 — the only one that decides what
# the runtime actually does — is SILENT. Omitting `workflows:` is therefore relying on
# UNSPECIFIED behaviour, and its failure mode is silent in the worst possible place: if
# the trigger matches nothing, the evaluator simply never fires, no status is ever
# published, and a required-but-never-published context blocks every merge as "expected
# but missing" with no failing run to point at. That is unacceptable for the repo's
# single required check, so the decision is: ALWAYS declare `workflows:` explicitly.
# Accepting the sibling-list maintenance burden is the cheap, specified side of the
# trade; the design record carries the rationale and the alternatives considered.
#
# This guard makes that decision mechanical rather than prose, because prose is exactly
# what the first draft overlooked. All six `workflow_run` triggers already in the repo
# comply, so it is green on main today and reds only a NEW omission.
#
# SCOPE / non-goals: this is a narrow, single-rule structural check, NOT a general
# workflow linter and NOT a substitute for adopting actionlint (which the repo does not
# run in CI). It checks exactly one thing, which keeps it pure-stdlib — matching the
# neighbouring `check-fast-fix-ring-guard.py` precedent and staying runnable in a bare
# checkout with no PyYAML installed. (`quick-gates` does install PyYAML earlier in the
# job, so this is a portability property, not a hard ordering constraint.) The line
# scanner is deliberately confined to the `on:` block, and its agreement with a real
# YAML parser was differentially verified over all 78 workflows in the repo.
#
# Usage:
#   check-workflow-run-filter.py             # check .github/workflows (default)
#   check-workflow-run-filter.py --root DIR  # use DIR as repo root
#   check-workflow-run-filter.py --self-test # hermetic positive/negative fixtures
#
# Exit 0 = clean; exit 1 = one or more offences.

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
WORKFLOW_DIR = Path(".github") / "workflows"

ON_KEY = re.compile(r"""^(?:on|"on"|'on'):(.*)$""")
WORKFLOW_RUN_KEY = re.compile(r"^(\s*)workflow_run:\s*(.*?)\s*$")
KEY = re.compile(r"^(\s*)([A-Za-z_][\w-]*|\"[^\"]+\"|'[^']+'):\s*(.*?)\s*$")


def _indent(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def _is_skippable(line: str) -> bool:
    """Blank lines and whole-line comments carry no structure."""
    s = line.strip()
    return not s or s.startswith("#")


def _strip_comment(value: str) -> str:
    """Drop a trailing `# ...` comment from a scalar value.

    Only used on the short right-hand side of a key inside the `on:` block, where a
    `#` inside a quoted string would be the only false positive; no such value exists
    in this repo and the guard fails SAFE (it would read the value as shorter, i.e.
    more likely to red, never less).
    """
    if "#" not in value:
        return value
    return value.split("#", 1)[0].strip()


def _on_block(lines: list[str]) -> tuple[int, int] | None:
    """Return [start, end) line indices of the body of the top-level `on:` mapping.

    Restricting every later scan to this region is what makes a stdlib line scanner
    sound here: the region cannot contain a `run:` script body, so an incidental
    `github.event.workflow_run` reference in a job's shell script or an `if:`
    expression can never be mistaken for a trigger declaration.
    """
    start = None
    for i, line in enumerate(lines):
        if _is_skippable(line):
            continue
        if ON_KEY.match(line):
            start = i + 1
            break
    if start is None:
        return None
    for j in range(start, len(lines)):
        if _is_skippable(lines[j]):
            continue
        if _indent(lines[j]) == 0:
            return (start, j)
    return (start, len(lines))


def _child_block(lines: list[str], start: int, end: int, parent_indent: int) -> list[int]:
    """Indices of the structural lines strictly nested under `parent_indent`."""
    out = []
    for i in range(start, end):
        if _is_skippable(lines[i]):
            continue
        if _indent(lines[i]) <= parent_indent:
            break
        out.append(i)
    return out


def _workflows_is_populated(lines: list[str], idx: int, end: int, value: str) -> bool:
    """Is the `workflows:` at `idx` a NON-EMPTY list?

    An empty `workflows: []` is as bad as omitting the key — it names no workflow, so
    the trigger still matches nothing.
    """
    value = _strip_comment(value)
    if value:
        # Flow sequence on the same line: `workflows: [a, b]`.
        inner = value.strip()
        if inner.startswith("[") and inner.endswith("]"):
            return bool(inner[1:-1].strip())
        # Any other non-empty scalar (e.g. `workflows: ci-summary`) still names one.
        return True
    # Block sequence on following lines: at least one `- item` nested deeper.
    key_indent = _indent(lines[idx])
    for i in _child_block(lines, idx + 1, end, key_indent):
        if lines[i].strip().startswith("-"):
            return True
    return False


def check_text(text: str, rel: str) -> list[str]:
    """Return offence strings for one workflow document; empty == clean."""
    lines = text.split("\n")
    region = _on_block(lines)
    if region is None:
        return []
    start, end = region
    offences = []

    for i in range(start, end):
        if _is_skippable(lines[i]):
            continue
        m = WORKFLOW_RUN_KEY.match(lines[i])
        if not m:
            continue
        indent, inline = m.group(1), _strip_comment(m.group(2))

        if inline:
            # Flow mapping (`workflow_run: {workflows: [x]}`) or a bare scalar. Treat
            # anything that does not visibly name a populated `workflows` as an offence
            # rather than trying to parse flow YAML by hand.
            body = inline.strip()
            ok = body.startswith("{") and re.search(
                r"workflows\s*:\s*\[[^\]]*[^\s\]][^\]]*\]|workflows\s*:\s*[^\s,}]", body
            )
            if not ok:
                offences.append(
                    f"{rel}:{i + 1}: `workflow_run:` declares no non-empty `workflows:` "
                    f"filter (inline form: {body or '<empty>'})"
                )
            continue

        # Block form: look for a `workflows:` key among the immediate children.
        children = _child_block(lines, i + 1, end, len(indent))
        if not children:
            offences.append(
                f"{rel}:{i + 1}: `workflow_run:` has no filters at all — it must declare "
                f"a non-empty `workflows:` list"
            )
            continue
        child_indent = _indent(lines[children[0]])
        found = None
        for c in children:
            if _indent(lines[c]) != child_indent:
                continue
            km = KEY.match(lines[c])
            if km and km.group(2).strip("\"'") == "workflows":
                found = c
                break
        if found is None:
            offences.append(
                f"{rel}:{i + 1}: `workflow_run:` declares no `workflows:` filter"
            )
        elif not _workflows_is_populated(lines, found, end, KEY.match(lines[found]).group(3)):
            offences.append(
                f"{rel}:{found + 1}: `workflows:` is empty — it names no workflow, so the "
                f"`workflow_run` trigger still matches nothing"
            )

    return offences


def check_root(root: Path) -> list[str]:
    wf_dir = root / WORKFLOW_DIR
    if not wf_dir.is_dir():
        return [f"{WORKFLOW_DIR} not found under {root}"]
    offences = []
    for path in sorted(wf_dir.glob("*.yml")) + sorted(wf_dir.glob("*.yaml")):
        rel = str(path.relative_to(root))
        offences.extend(check_text(path.read_text(encoding="utf-8"), rel))
    return offences


REMEDY = (
    "Declare the sibling workflow(s) explicitly, e.g.\n"
    "    on:\n"
    "      workflow_run:\n"
    "        workflows: [ci-summary]\n"
    "        types: [completed]\n"
    "Omitting `workflows:` relies on behaviour GitHub does not document (actionlint "
    "rejects it outright; the SchemaStore schema permits it) and fails SILENTLY — the "
    "trigger may match nothing, so the workflow never fires. For a required status "
    "context that presents as 'expected but missing' and blocks every merge with no "
    "failing run to point at. See scripts/check-workflow-run-filter.py's header and "
    "research/ci-gate-slotless-aggregation.md §3.7 (#5654)."
)


# --------------------------------------------------------------------------- #
#  --self-test: hermetic fixtures                                             #
# --------------------------------------------------------------------------- #

CLEAN_BLOCK = """name: demo
on:
  workflow_run:
    workflows: [ci-summary]
    types: [completed]

jobs:
  a:
    runs-on: ubuntu-latest
    steps:
      - run: echo "${{ github.event.workflow_run.conclusion }}"
"""


def _self_test() -> int:
    failures = 0

    def expect(label: str, text: str, want_red: bool, needle: str = "") -> None:
        nonlocal failures
        offs = check_text(text, "fixture.yml")
        if want_red and not any(needle in o for o in offs):
            failures += 1
            print(f"  [FAIL] {label}: expected RED containing {needle!r}, got {offs}")
        elif not want_red and offs:
            failures += 1
            print(f"  [FAIL] {label}: expected CLEAN, got {offs}")
        else:
            print(f"  [ok]   {label}: {'RED' if want_red else 'clean'} (as expected)")

    # --- positives: valid shapes must NOT red (guards against over-firing) ---
    expect("block seq flow-style `workflows: [ci-summary]`", CLEAN_BLOCK, False)
    expect(
        "block seq dash-style workflows",
        CLEAN_BLOCK.replace(
            "    workflows: [ci-summary]\n", "    workflows:\n      - ci-summary\n"
        ),
        False,
    )
    expect(
        "quoted workflow name + branches filter",
        CLEAN_BLOCK.replace(
            "    workflows: [ci-summary]\n",
            '    workflows: ["CI"]\n    branches: [main]\n',
        ),
        False,
    )
    expect(
        "`workflows:` declared AFTER `types:` (key order must not matter)",
        CLEAN_BLOCK.replace(
            "    workflows: [ci-summary]\n    types: [completed]\n",
            "    types: [completed]\n    workflows: [ci-summary]\n",
        ),
        False,
    )
    expect(
        "a job step mentioning github.event.workflow_run must not be read as a trigger",
        """name: demo
on:
  pull_request:

jobs:
  a:
    runs-on: ubuntu-latest
    steps:
      - run: |
          workflow_run:
          echo "${{ github.event.workflow_run.id }}"
""",
        False,
    )
    expect(
        "a commented-out workflow_run is not a declaration",
        """name: demo
on:
  pull_request:
  # workflow_run:
  #   types: [completed]

jobs:
  a:
    runs-on: ubuntu-latest
    steps:
      - run: echo hi
""",
        False,
    )

    # --- negatives: every way to end up with no named sibling must RED ---
    expect(
        "THE #5654 CONSTRUCT — types: only, no workflows:",
        CLEAN_BLOCK.replace("    workflows: [ci-summary]\n", ""),
        True,
        "declares no `workflows:` filter",
    )
    expect(
        "empty flow list `workflows: []`",
        CLEAN_BLOCK.replace("workflows: [ci-summary]", "workflows: []"),
        True,
        "is empty",
    )
    expect(
        "bare `workflow_run:` with no filters at all",
        CLEAN_BLOCK.replace(
            "  workflow_run:\n    workflows: [ci-summary]\n    types: [completed]\n",
            "  workflow_run:\n",
        ),
        True,
        "no filters at all",
    )
    expect(
        "`workflows:` demoted to a comment",
        CLEAN_BLOCK.replace(
            "    workflows: [ci-summary]", "    # workflows: [ci-summary]"
        ),
        True,
        "declares no `workflows:` filter",
    )
    expect(
        "`workflows:` mis-nested one level too deep (under types:)",
        CLEAN_BLOCK.replace(
            "    workflows: [ci-summary]\n    types: [completed]\n",
            "    types:\n      - completed\n      workflows: [ci-summary]\n",
        ),
        True,
        "declares no `workflows:` filter",
    )
    expect(
        "inline flow mapping with no workflows key",
        CLEAN_BLOCK.replace(
            "  workflow_run:\n    workflows: [ci-summary]\n    types: [completed]\n",
            "  workflow_run: {types: [completed]}\n",
        ),
        True,
        "inline form",
    )
    expect(
        "second trigger in the same file omits workflows:",
        """name: demo
on:
  workflow_run:
    workflows: [ci-summary]
    types: [completed]
  pull_request:

jobs:
  a:
    runs-on: ubuntu-latest
    steps:
      - run: echo hi
""".replace(
            "    workflows: [ci-summary]\n", ""
        ),
        True,
        "declares no `workflows:` filter",
    )

    if failures:
        print(f"\nSELF-TEST FAILED: {failures} case(s) did not behave as expected.")
        return 1
    print(
        "\nSELF-TEST PASSED: valid `workflows:` shapes stay clean (including a job step "
        "that merely mentions github.event.workflow_run) and every no-named-sibling "
        "construct — the #5654 omission, `[]`, a bare trigger, a commented-out or "
        "mis-nested key, and an inline flow mapping — reds."
    )
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(
        description="require a non-empty `workflows:` filter on every workflow_run trigger (#5654)"
    )
    ap.add_argument("--root", type=Path, default=None, help="repo root override")
    ap.add_argument("--self-test", action="store_true", help="run hermetic fixtures")
    args = ap.parse_args()

    if args.self_test:
        return _self_test()

    offences = check_root(args.root if args.root is not None else REPO_ROOT)
    if offences:
        print("workflow_run `workflows:` filter check FAILED (#5654):", file=sys.stderr)
        for o in offences:
            print(f"  - {o}", file=sys.stderr)
        print(f"\n{REMEDY}", file=sys.stderr)
        return 1
    print("workflow_run `workflows:` filter check: OK (#5654).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
