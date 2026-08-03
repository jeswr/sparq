#!/usr/bin/env python3
# [OPUS-5] CI lint (issue #5574): a workflow `name:` / `run-name:` value written as an
# UNQUOTED plain scalar must not contain a ` #`, because YAML silently truncates it.
#
# WHY THIS GATE EXISTS — the drift it is here to stop:
# Our step names deliberately carry an issue reference so a CI log line can be traced
# back to the change that introduced it, e.g.
#     - name: Self-test the advisory-registry checker (C2/C3/C4 logic — sq-qcnn.32, #3773)
# In YAML a `#` preceded by whitespace opens a COMMENT, so the parsed step name is
#     Self-test the advisory-registry checker (C2/C3/C4 logic — sq-qcnn.32,
# — truncated mid-clause, with an unbalanced paren, and with the issue number (the
# entire reason the reference was written) gone from the rendered log. Execution is
# unaffected, which is exactly why it accumulated silently to ELEVEN occurrences
# before anyone scanned for it (#5574). The fix is always the same: quote the value.
#
# RULE: scan .github/workflows/*.yml (+ *.yaml). Flag every `name:` / `run-name:`
# mapping entry whose value is a plain (unquoted) scalar containing whitespace
# followed by `#`. NOT flagged, because none of these truncate:
#   - a quoted value — `name: "... #3773)"` / `name: '... #3773)'` (this is the fix);
#   - `#` with no preceding whitespace — `name: issue#1740` is one plain scalar;
#   - a whole-line comment — `# name: ...`;
#   - a block scalar header (`name: >` / `name: |`), where the value is on later lines.
# Scope is deliberately `name:`/`run-name:` only. The truncation rule is a property of
# every plain scalar, but those are the keys whose value is rendered back to a human
# in the Actions UI and in logs, so they are where a silent truncation actually costs
# something — and a narrow rule keeps the gate free of false positives.
#
# WHY A HAND-ROLLED LEXICAL SCANNER (not PyYAML): a YAML parser is USELESS for this
# defect. By the time it hands back the value the truncation has already happened —
# it returns `... sq-qcnn.32,` and there is nothing left to compare against. The
# information that a `#3773)` was ever written survives only in the RAW LINE, so
# detection has to be lexical. (Being stdlib-only is a side benefit, not the reason.)
#
# EXIT: 0 when no plain-scalar name is truncated; 1 with a per-offence message
# otherwise (and 1 if the live scan sees implausibly few name lines — a vacuity guard
# so a broken regex cannot pass by matching nothing).
#
# Usage:
#   check-workflow-step-names.py                  # scan .github/workflows
#   check-workflow-step-names.py --root <dir>     # scan <dir>/.github/workflows
#   check-workflow-step-names.py --self-test      # hermetic logic self-test
#
# stdlib-only.

from __future__ import annotations

import argparse
import re
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# `name:` / `run-name:`, optionally introduced by a `- ` list dash, followed by at
# least one space and a value. Group 1 is the key, group 2 the raw value text.
_NAME_RE = re.compile(r"^[ \t]*(?:-[ \t]+)?(name|run-name):[ \t]+(\S.*)$")

# The value opens with a quote (already safe) or a block-scalar / alias / anchor /
# tag indicator (no trailing plain scalar to truncate).
_NON_PLAIN_START = ('"', "'", "|", ">", "&", "*", "!")

# Whitespace immediately before a `#` — the YAML comment trigger inside a plain scalar.
_COMMENT_TRIGGER_RE = re.compile(r"[ \t]#")

# Vacuity floor for the LIVE scan: the workflow tree has well over a thousand
# name-carrying lines, so seeing fewer than this means the regex (or the file
# discovery) broke and the gate would be passing on nothing.
MIN_LIVE_NAME_LINES = 200


def truncation_offence(line: str) -> tuple[str, str] | None:
    """Return (full_intended_value, truncated_value) if `line` is a `name:`/`run-name:`
    entry that YAML will silently truncate at a comment, else None."""
    m = _NAME_RE.match(line.rstrip("\n"))
    if m is None:
        return None
    value = m.group(2)
    if value.startswith(_NON_PLAIN_START):
        return None
    hit = _COMMENT_TRIGGER_RE.search(value)
    if hit is None:
        return None
    return value.rstrip(), value[: hit.start()].rstrip()


def scan_text(text: str) -> tuple[list[tuple[int, str, str]], int]:
    """Scan one workflow file's text.

    Returns (offences, name_line_count) where each offence is
    (1-based line number, full intended value, value as YAML will parse it)."""
    offences: list[tuple[int, str, str]] = []
    name_lines = 0
    for lineno, line in enumerate(text.splitlines(), start=1):
        if _NAME_RE.match(line):
            name_lines += 1
        found = truncation_offence(line)
        if found is not None:
            offences.append((lineno, found[0], found[1]))
    return offences, name_lines


def scan_workflows(root: Path) -> tuple[list[tuple[Path, int, str, str]], int]:
    """Scan every workflow file under `root`/.github/workflows.

    Returns (offences, total_name_lines); each offence is
    (file, line number, full intended value, truncated value)."""
    wf_dir = root / ".github" / "workflows"
    results: list[tuple[Path, int, str, str]] = []
    total = 0
    if not wf_dir.is_dir():
        return results, total
    for path in sorted(wf_dir.iterdir()):
        if path.suffix not in (".yml", ".yaml"):
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="ignore")
        except OSError:
            continue
        offences, name_lines = scan_text(text)
        total += name_lines
        for lineno, full, truncated in offences:
            results.append((path, lineno, full, truncated))
    return results, total


# --------------------------------------------------------------------------- tests
# The exact shape #5574 was raised about: an unquoted step name whose issue reference
# is preceded by a space, so YAML eats `#3773)` and leaves an unbalanced paren.
_BAD_STEP_NAME = """\
jobs:
  lint:
    steps:
      - name: Self-test the advisory-registry checker (C2/C3/C4 logic — sq-qcnn.32, #3773)
        run: python3 scripts/check-advisory-registry.py --self-test
"""

# The fix: quoting the same value.
_GOOD_DOUBLE_QUOTED = """\
jobs:
  lint:
    steps:
      - name: "Self-test the advisory-registry checker (C2/C3/C4 logic — sq-qcnn.32, #3773)"
        run: python3 scripts/check-advisory-registry.py --self-test
"""

_GOOD_SINGLE_QUOTED = """\
jobs:
  lint:
    steps:
      - name: 'Publish-cadence guard (minimum release interval, #2552)'
        run: true
"""

# `#` with no preceding whitespace is part of the plain scalar — nothing is truncated.
_GOOD_NO_SPACE_BEFORE_HASH = """\
jobs:
  lint:
    steps:
      - name: Drift tripwire for issue#1740
        run: true
"""

# A `name:` continuation line (not on the dash) is still a mapping entry and is still
# truncated — the dash is optional in the rule.
_BAD_NAME_NOT_ON_DASH = """\
jobs:
  lint:
    steps:
      - id: guard
        name: Publish-cadence guard (fail-closed, #2552)
        run: true
"""

# Top-level `name:` and `run-name:` are in scope too.
_BAD_RUN_NAME = """\
name: docs-quality
run-name: docs-quality for #1135
"""

# A whole-line YAML comment that happens to mention `name:` must not be flagged.
_GOOD_COMMENT_LINE = """\
jobs:
  lint:
    steps:
      # - name: disabled step (see #1234)
      - name: live step
        run: true
"""

# A block-scalar header has no trailing plain scalar to truncate.
_GOOD_BLOCK_SCALAR = """\
jobs:
  lint:
    steps:
      - name: >
          a folded name mentioning #1234
        run: true
"""

# `run:` / `uses:` values are out of scope: a trailing comment there is idiomatic and
# intentional, and the shell text is not rendered back as a label.
_GOOD_OTHER_KEYS_UNTOUCHED = """\
jobs:
  lint:
    steps:
      - uses: actions/checkout@deadbeefdeadbeefdeadbeefdeadbeefdeadbeef # v6
        run: echo hi # a trailing comment here is fine
"""

# Two offences in one file — the scanner must report both, not stop at the first.
# NB the `#` must follow WHITESPACE to truncate: `first (#1)` would be a comment-free
# plain scalar, which is why the fixture spells the reference as ` #1)`.
_BAD_TWO_IN_ONE_FILE = """\
jobs:
  lint:
    steps:
      - name: first step, #1)
        run: true
      - name: second step, #2)
        run: true
"""


def self_test() -> int:
    cases = [
        ("bad: unquoted step name with ` #3773)`", _BAD_STEP_NAME, 1),
        ("good: double-quoted (the fix)", _GOOD_DOUBLE_QUOTED, 0),
        ("good: single-quoted", _GOOD_SINGLE_QUOTED, 0),
        ("good: `#` not preceded by whitespace", _GOOD_NO_SPACE_BEFORE_HASH, 0),
        ("bad: `name:` off the dash line", _BAD_NAME_NOT_ON_DASH, 1),
        ("bad: top-level `run-name:`", _BAD_RUN_NAME, 1),
        ("good: whole-line comment mentioning name:", _GOOD_COMMENT_LINE, 0),
        ("good: block-scalar name header", _GOOD_BLOCK_SCALAR, 0),
        ("good: run:/uses: trailing comments untouched", _GOOD_OTHER_KEYS_UNTOUCHED, 0),
        ("bad: two offences in one file", _BAD_TWO_IN_ONE_FILE, 2),
    ]
    failures = 0
    for label, text, expected in cases:
        got = len(scan_text(text)[0])
        ok = got == expected
        print(
            f"  [{'PASS' if ok else 'FAIL'}] {label}: "
            f"{got} offence(s) (want {expected})"
        )
        if not ok:
            failures += 1

    # The reported truncation must be the value YAML ACTUALLY yields, not a guess:
    # assert on the exact pair for the canonical #5574 case.
    offence = truncation_offence(
        "      - name: Zero-dispatch watchdog suite (policy + behaviour + YAML seam — #4652)"
    )
    want = (
        "Zero-dispatch watchdog suite (policy + behaviour + YAML seam — #4652)",
        "Zero-dispatch watchdog suite (policy + behaviour + YAML seam —",
    )
    ok = offence == want
    print(f"  [{'PASS' if ok else 'FAIL'}] reported truncation matches YAML semantics")
    if not ok:
        failures += 1
        print(f"        got {offence!r}\n        want {want!r}")

    if failures:
        print(f"\nself-test: {failures} case(s) FAILED")
        return 1
    print("\nself-test: all cases PASS")
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="Fail if a workflow `name:`/`run-name:` plain scalar is silently "
        "truncated by an unquoted trailing ` #` comment (issue #5574)."
    )
    ap.add_argument(
        "--root",
        default=str(REPO_ROOT),
        help="repo root whose .github/workflows is scanned (default: this repo).",
    )
    ap.add_argument(
        "--self-test",
        action="store_true",
        help="run the hermetic logic self-test and exit.",
    )
    args = ap.parse_args(argv)

    if args.self_test:
        return self_test()

    root_path = Path(args.root)
    offences, total_names = scan_workflows(root_path)

    if total_names < MIN_LIVE_NAME_LINES:
        print(
            "workflow step-name truncation gate: FAIL (VACUOUS)\n\n"
            f"Only {total_names} `name:`/`run-name:` line(s) were seen under "
            f"{root_path}/.github/workflows,\nbelow the {MIN_LIVE_NAME_LINES} floor. "
            "The scan matched (almost) nothing, so a PASS\nwould be meaningless — "
            "fix the scanner or the floor rather than trusting this run."
        )
        return 1

    if not offences:
        print(
            "workflow step-name truncation gate: PASS — no `name:`/`run-name:` plain "
            f"scalar is truncated by a trailing comment ({total_names} name line(s) scanned)."
        )
        return 0

    print("workflow step-name truncation gate: FAIL\n")
    print(
        "In YAML a `#` preceded by whitespace opens a COMMENT, so these unquoted\n"
        "`name:` values are silently truncated — the issue reference they carry (the\n"
        "reason they carry it: log greppability and blame attribution) never reaches\n"
        "the Actions UI. Wrap each value in double quotes (issue #5574):\n"
    )
    for path, lineno, full, truncated in offences:
        rel = path.relative_to(root_path) if path.is_relative_to(root_path) else path
        print(f"    - {rel}:{lineno}")
        print(f"        written: {full}")
        print(f"        parsed : {truncated}")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
