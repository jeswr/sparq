#!/usr/bin/env python3
"""Require an explicit, non-empty ``workflows:`` filter on every ``workflow_run`` trigger.

# [OPUS-5] Issue #5654 — resolves the HONEST UNCERTAINTY recorded by PR #5520 /
# bead sq-lfmvd, which PROPOSED a `ci-summary-status.yml` aggregator whose
# `workflow_run` trigger deliberately OMITTED `workflows:` so it would never
# hard-code a sibling list, and asked for the omission to be confirmed before
# `ci-summary/status` joined the required contexts.

STATE AT THE TIME OF WRITING.  That aggregator never landed: there is no
`ci-summary-status.yml` on `main`, and all six live `workflow_run` triggers
(`batch-merge`, `fast-fix-ring`, `feature-matrix-report`, `pr-backlog`,
`selection-alarm`, `verdict-bridge`) already name their upstream workflow.
So this lint fixes nothing today — it is PREVENTIVE, and it closes the open
question by making the answer enforceable instead of a comment someone must recall.

THE DECISION.  Treat `workflows:` as MANDATORY on `workflow_run`.  Two reasons:

* Reported in #5654 (NOT re-run here — actionlint is not installed in this
  repository and is not part of CI): `actionlint` 1.7.7 rejects the construct
  outright with `no workflow is configured for "workflow_run" event [events]`.
  That is cited as evidence that a mainstream workflow linter reads the key as
  required.  This script does not wrap, vendor or re-implement actionlint.
* The failure mode is SILENT.  A `workflow_run` trigger that never fires produces
  no run, no check-run and no annotation — the lane simply does not exist.  Nothing
  in CI distinguishes "never fired" from "fired and passed", so the mistake cannot
  be caught downstream; it has to be caught at authoring time.

The second reason is the load-bearing one, and it holds regardless of how GitHub
actually treats the omission: a trigger whose behaviour is undocumented and whose
failure is invisible is not something to rely on for a required check.

SCOPE.  Deliberately narrow.  This checks ONE structural property of ONE trigger
key: that a `workflow_run:` under a workflow's top-level `on:` carries a
`workflows:` filter with at least one entry.  It is not an actionlint substitute
and makes no claim about the rest of the trigger block.

Both spellings of the trigger body are parsed structurally — the block mapping
(`workflows:` on its own line) and the flow mapping (`workflow_run: {workflows:
[CI]}`).  In each case the EXACT `workflows` key must be present and name at
least one entry; a neighbouring key, a quoted scalar or an empty list does not
satisfy it.  An inline value that is not a well-formed flow mapping is reported
rather than assumed to be fine.

Pure stdlib (no PyYAML), so the gate runs identically on a bare runner and in a
local worktree.  The parse is indentation-scoped to the top-level `on:` block, so
the string `workflow_run` appearing in a `run:` script or a comment is not a match.

Usage:
    python3 scripts/check-workflow-run-filters.py              # scan .github/workflows
    python3 scripts/check-workflow-run-filters.py --self-test  # hermetic fixtures
"""

from __future__ import annotations

import argparse
import contextlib
import io
import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW_DIR = ROOT / ".github" / "workflows"

# A block-mapping key line: indent, key name, and whatever follows the colon.
KEY_RE = re.compile(r"^(?P<indent>[ ]*)(?P<key>[A-Za-z_][A-Za-z0-9_\-]*|\"[^\"]+\"|'[^']+'):(?P<rest>.*)$")


def _unquote(key: str) -> str:
    if len(key) >= 2 and key[0] == key[-1] and key[0] in "\"'":
        return key[1:-1]
    return key


def _is_structural(line: str) -> bool:
    """True for a line that participates in the block structure (not blank/comment)."""
    stripped = line.strip()
    return bool(stripped) and not stripped.startswith("#")


def _indent_of(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def _block(lines: list[str], start: int, outer_indent: int) -> list[tuple[int, str]]:
    """The (index, text) lines nested strictly deeper than ``outer_indent`` after ``start``."""
    out: list[tuple[int, str]] = []
    for i in range(start + 1, len(lines)):
        line = lines[i]
        if not _is_structural(line):
            continue
        if _indent_of(line) <= outer_indent:
            break
        out.append((i, line))
    return out


def _child_keys(block: list[tuple[int, str]]) -> list[tuple[int, str, str, int]]:
    """Immediate child keys of a block: (line index, key, rest-of-line, indent)."""
    if not block:
        return []
    child_indent = min(_indent_of(line) for _, line in block)
    out = []
    for i, line in block:
        if _indent_of(line) != child_indent:
            continue
        m = KEY_RE.match(line)
        if m:
            out.append((i, _unquote(m.group("key")), m.group("rest"), child_indent))
    return out


def _strip_comment(rest: str) -> str:
    """Drop a trailing `# ...` comment.  Quote-aware enough for trigger values."""
    quote = None
    for pos, ch in enumerate(rest):
        if quote:
            if ch == quote:
                quote = None
        elif ch in "\"'":
            quote = ch
        elif ch == "#":
            return rest[:pos]
    return rest


def _split_flow_items(body: str) -> list[str] | None:
    """The top-level comma-separated items of a flow collection's interior.

    Returns ``None`` when brackets or quotes are unbalanced, so an unparseable
    construct is never mistaken for a well-formed one.
    """
    items: list[str] = []
    depth = 0
    quote: str | None = None
    escaped = False
    start = 0
    for pos, ch in enumerate(body):
        if quote:
            if escaped:
                escaped = False
            elif quote == '"' and ch == "\\":
                escaped = True
            elif ch == quote:
                quote = None
            continue
        if ch in "\"'":
            quote = ch
        elif ch in "[{":
            depth += 1
        elif ch in "]}":
            depth -= 1
            if depth < 0:
                return None
        elif ch == "," and depth == 0:
            items.append(body[start:pos])
            start = pos + 1
    if depth or quote:
        return None
    items.append(body[start:])
    return [item for item in items if item.strip()]


def _flow_key_value(item: str) -> tuple[str, str] | None:
    """Split one flow-mapping entry on its top-level `:`; ``None`` if it has none."""
    depth = 0
    quote: str | None = None
    escaped = False
    for pos, ch in enumerate(item):
        if quote:
            if escaped:
                escaped = False
            elif quote == '"' and ch == "\\":
                escaped = True
            elif ch == quote:
                quote = None
            continue
        if ch in "\"'":
            quote = ch
        elif ch in "[{":
            depth += 1
        elif ch in "]}":
            depth -= 1
        elif ch == ":" and depth == 0:
            return _unquote(item[:pos].strip()), item[pos + 1 :].strip()
    return None


def _flow_mapping(inline: str) -> dict[str, str] | None:
    """Parse `{k: v, ...}` into its top-level entries; ``None`` if not a flow mapping.

    Structural, so only the EXACT `workflows` key counts — `not_workflows`, or the
    word appearing inside a quoted scalar, does not satisfy the filter.
    """
    if not (inline.startswith("{") and inline.endswith("}")):
        return None
    items = _split_flow_items(inline[1:-1])
    if items is None:
        return None
    entries: dict[str, str] = {}
    for item in items:
        pair = _flow_key_value(item)
        if pair is None:
            return None
        entries[pair[0]] = pair[1]
    return entries


def _flow_value_is_populated(value: str) -> bool:
    """Does a flow-style value name at least one entry?  `[]` and null do not."""
    value = value.strip()
    if value.startswith("[") and value.endswith("]"):
        return bool(_split_flow_items(value[1:-1]))
    return value not in ("", "~", "null", "Null", "NULL")


def _filter_is_populated(lines: list[str], idx: int, rest: str, indent: int) -> bool:
    """Does the `workflows:` key at ``idx`` name at least one workflow?"""
    inline = _strip_comment(rest).strip()
    if inline:
        # Flow sequence (`[a, b]`) or a bare scalar; `[]` is an explicit empty list.
        return _flow_value_is_populated(inline)
    # Block sequence on the following lines.
    return any(line.strip().startswith("-") for _, line in _block(lines, idx, indent))


def find_violations(text: str) -> list[str]:
    """Return one message per `workflow_run` trigger missing a populated `workflows:`."""
    lines = text.splitlines()
    problems: list[str] = []

    for idx, line in enumerate(lines):
        if not _is_structural(line) or _indent_of(line) != 0:
            continue
        m = KEY_RE.match(line)
        if not m or _unquote(m.group("key")) != "on":
            continue

        on_block = _block(lines, idx, 0)
        for wr_idx, key, wr_rest, wr_indent in _child_keys(on_block):
            if key != "workflow_run":
                continue
            inline = _strip_comment(wr_rest).strip()
            if inline:
                # Inline (flow) mapping such as `workflow_run: {workflows: [CI]}`.
                entries = _flow_mapping(inline)
                if entries is None:
                    problems.append(
                        f"line {wr_idx + 1}: `workflow_run` has an UNPARSEABLE inline value; "
                        "write it as a mapping carrying an explicit `workflows:` filter"
                    )
                elif "workflows" not in entries:
                    problems.append(
                        f"line {wr_idx + 1}: `workflow_run` has no `workflows:` filter"
                    )
                elif not _flow_value_is_populated(entries["workflows"]):
                    problems.append(
                        f"line {wr_idx + 1}: `workflow_run` has an EMPTY `workflows:` filter"
                    )
                continue
            filters = _child_keys(_block(lines, wr_idx, wr_indent))
            populated = [
                _filter_is_populated(lines, f_idx, f_rest, f_indent)
                for f_idx, f_key, f_rest, f_indent in filters
                if f_key == "workflows"
            ]
            if not populated:
                problems.append(
                    f"line {wr_idx + 1}: `workflow_run` has no `workflows:` filter"
                )
            elif not any(populated):
                problems.append(
                    f"line {wr_idx + 1}: `workflow_run` has an EMPTY `workflows:` filter"
                )
        # Only the first top-level `on:` mapping is the trigger block.
        break

    return problems


def scan(paths: list[Path]) -> int:
    failures = 0
    for path in sorted(paths):
        for problem in find_violations(path.read_text(encoding="utf-8")):
            failures += 1
            rel = path.relative_to(ROOT) if path.is_relative_to(ROOT) else path
            print(f"{rel}:{problem}", file=sys.stderr)
    if failures:
        print(
            f"\n{failures} `workflow_run` trigger(s) without a populated `workflows:` filter.\n"
            "GitHub gives no signal when such a trigger never fires — the lane is simply\n"
            "absent — so name the upstream workflow(s) explicitly (issue #5654).",
            file=sys.stderr,
        )
    return failures


# --------------------------------------------------------------------------- self-test

_CLEAN = """\
name: downstream
on:
  workflow_run:
    workflows: ["ci-summary"]
    types: [completed]
jobs:
  a:
    runs-on: ubuntu-latest
    steps:
      - run: echo "on: workflow_run: with no workflows: key in prose"
"""

_MISSING = """\
name: aggregator
on:
  workflow_run:
    types: [completed]
"""

_EMPTY = """\
name: aggregator
on:
  workflow_run:
    workflows: []
    types: [completed]
"""

_BLOCK_SEQUENCE = """\
name: downstream
on:
  workflow_run:
    workflows:
      - ci-summary
      - feature-matrix
    types: [completed]
"""

_COMMENTED_OUT = """\
name: aggregator
on:
  workflow_run:
    # workflows: ["ci-summary"]  # deliberately omitted
    types: [completed]
"""

_NO_WORKFLOW_RUN = """\
name: plain
on:
  pull_request:
  push:
    branches: [main]
"""

_QUOTED_ON = """\
name: aggregator
"on":
  workflow_run:
    types: [completed]
"""

# Trailing comments must not be read as values.  Without comment-stripping the comment
# on `workflow_run:` parses as an inline mapping and this CLEAN trigger reports a
# false positive, while `workflows: []  # ...` reads as populated and a real EMPTY
# filter escapes.  Both directions are covered below.
_TRAILING_COMMENT_CLEAN = """\
name: downstream
on:
  workflow_run:  # fires once the aggregator concludes
    workflows: ["ci-summary"]  # the single required check
    types: [completed]
"""

_TRAILING_COMMENT_EMPTY = """\
name: aggregator
on:
  workflow_run:
    workflows: []  # intentionally unfiltered
    types: [completed]
"""

# Flow-style trigger bodies get the SAME structural treatment as block mappings.  A
# substring test for "workflows" accepts every one of the four rejects below — an
# empty list, a neighbouring key that merely contains the word, the word quoted as a
# value, and a truncated mapping — so these are what make the flow branch non-vacuous.
_FLOW_CLEAN = """\
name: downstream
on:
  workflow_run: {workflows: [CI], types: [completed]}
"""

_FLOW_EMPTY = """\
name: aggregator
on:
  workflow_run: {workflows: [], types: [completed]}
"""

_FLOW_WRONG_KEY = """\
name: aggregator
on:
  workflow_run: {not_workflows: [CI], types: [completed]}
"""

_FLOW_QUOTED_SCALAR = """\
name: aggregator
on:
  workflow_run: {types: ["workflows"]}
"""

_FLOW_UNPARSEABLE = """\
name: aggregator
on:
  workflow_run: {workflows: [CI]
"""

# Each case asserts the exact DIAGNOSIS, not merely the finding count.  The two
# violation branches ("no filter" / "EMPTY filter") always report the SAME count for a
# given trigger, so a count-only assertion is vacuous: disabling the missing-key branch
# leaves the empty-list branch to fire in its place and the totals never move.  Matching
# on the message is what makes each branch independently non-vacuous.
_SELF_TEST_CASES = [
    ("clean inline list", _CLEAN, []),
    ("missing workflows key", _MISSING, ["has no `workflows:` filter"]),
    ("empty workflows list", _EMPTY, ["has an EMPTY `workflows:` filter"]),
    ("block sequence", _BLOCK_SEQUENCE, []),
    ("commented-out filter is still missing", _COMMENTED_OUT, ["has no `workflows:` filter"]),
    ("no workflow_run trigger", _NO_WORKFLOW_RUN, []),
    ("quoted on: key", _QUOTED_ON, ["has no `workflows:` filter"]),
    ("trailing comments on a clean trigger", _TRAILING_COMMENT_CLEAN, []),
    ("trailing comment after an empty list", _TRAILING_COMMENT_EMPTY, ["has an EMPTY `workflows:` filter"]),
    ("flow mapping with a populated filter", _FLOW_CLEAN, []),
    ("flow mapping with an empty list", _FLOW_EMPTY, ["has an EMPTY `workflows:` filter"]),
    ("flow mapping with a neighbouring key", _FLOW_WRONG_KEY, ["has no `workflows:` filter"]),
    ("flow mapping with the word only as a value", _FLOW_QUOTED_SCALAR, ["has no `workflows:` filter"]),
    ("unparseable inline value", _FLOW_UNPARSEABLE, ["has an UNPARSEABLE inline value"]),
]


def self_test() -> int:
    failures = 0
    for label, text, expected in _SELF_TEST_CASES:
        found = find_violations(text)
        ok = len(found) == len(expected) and all(
            fragment in message for fragment, message in zip(expected, found)
        )
        if not ok:
            failures += 1
            print(f"  [FAIL] {label}: expected {expected}, found {found}")
        else:
            print(f"  [ok] {label}: {len(found)} violation(s), diagnosis matched")

    # End-to-end through scan(), so the file walk and exit code are exercised too.
    # Fixture diagnostics go to stderr by design; swallow them so the self-test output
    # is not mistaken for a real finding.
    with tempfile.TemporaryDirectory() as tmp, io.StringIO() as noise:
        good = Path(tmp) / "good.yml"
        bad = Path(tmp) / "bad.yml"
        good.write_text(_CLEAN, encoding="utf-8")
        bad.write_text(_MISSING, encoding="utf-8")
        with contextlib.redirect_stderr(noise):
            clean_count, bad_count = scan([good]), scan([bad])
        if clean_count != 0:
            print("  [FAIL] scan() flagged a clean workflow")
            failures += 1
        if bad_count != 1:
            print("  [FAIL] scan() did not flag a workflow missing `workflows:`")
            failures += 1

    print("self-test: PASS" if not failures else f"self-test: {failures} FAILURE(S)")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true", help="run hermetic fixtures")
    parser.add_argument("paths", nargs="*", type=Path, help="workflow files (default: all)")
    args = parser.parse_args()

    if args.self_test:
        return 1 if self_test() else 0

    paths = args.paths or sorted(WORKFLOW_DIR.glob("*.yml")) + sorted(WORKFLOW_DIR.glob("*.yaml"))
    if not paths:
        print(f"no workflow files found under {WORKFLOW_DIR}", file=sys.stderr)
        return 1
    return 1 if scan(paths) else 0


if __name__ == "__main__":
    sys.exit(main())
