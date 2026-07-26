#!/usr/bin/env python3
# [OPUS-5] Pre-submit runner — shift the mechanical merge-gates LEFT, into the
# worker's own worktree, BEFORE the PR exists (bead sq-presubmit, see the
# "first-pass review yield" analysis in the PR body).
#
# WHY THIS EXISTS — measured, not assumed.
# ----------------------------------------
# A census of the 831 review-verdict records on the registry `ledger` branch
# (orchestration/review-verdicts/<owner>--<repo>--pr<N>-round<K>.json) found 378
# round-1 reviews, of which 211 returned `request_changes`. Hand-classifying every
# one of the 317 blocking (major/blocker) findings those 211 reviews raised showed
# that roughly HALF are real defects a reviewer earned (algorithm/spec/crypto bugs,
# races, credential exposure) and roughly half are AUTHORING-TIME preventable:
#
#     CORRECTNESS       82 issues   (real bug — not preventable here)
#     CLAIM-vs-CODE     67 issues   (prose in the diff asserts what the diff's own
#                                    code does not do — partly preventable)
#     GUARD-NOT-PINNED  63 issues   (the guard/test the PR adds cannot detect the
#                                    case it exists for — preventable)
#     MALFORMED-INPUT   27 issues
#     REPO-CONTRACT     20 issues   (violates a STATED, ALREADY-SCRIPTED repo rule)
#     SECURITY          19 issues
#     ERROR-SWALLOW     11 issues
#     CI-NOT-EXERCISED  11 issues
#     SCOPE-VIOLATION    9 issues
#     RACE               8 issues
#
# The REPO-CONTRACT column is the scandal: every one of those 20 findings is a rule
# the repository ALREADY OWNS A SCRIPT FOR (G1 gate-new-crate.py, G2
# gate-api-skill.py, G6 check-config-documented.py, check-no-perf-numbers.py,
# check-readme-template.py, check-privacy-claims.sh). Two sampled examples — sparq
# #4262 ("G1 new-crate-completeness gate fails: all three new crates ship without a
# README.md") and #3822 ("Hard G2 public-api→skill gate fails") — had the
# `flow-on-gates quick-gates (G1 + G2 + G6)` check-run ALREADY RED at the exact head
# SHA the reviewer bound their verdict to. A whole review round was spent restating
# a machine result.
#
# The root cause is an information asymmetry, not a missing check: NONE of the
# worker briefs (.claude/agents/sparq-rust-impl.md, sparq-rust-feature.md,
# sparq-ci-infra.md, sparq-docs.md, sparq-site.md) name `gate-new-crate.py` or
# `gate-api-skill.py` at all. The author is graded on a checklist they were never
# handed. This script IS that checklist, executable.
#
# WHAT IT DOES AND DOES NOT CLAIM
# -------------------------------
# MECHANICAL checks (this script decides them; a failure exits non-zero):
#   G1/G2/G6, no-perf-numbers, readme-template, privacy-claims  — delegated to the
#   existing gate scripts, scoped to YOUR diff, so they fire in-worktree instead of
#   on CI (and instead of in a review round).
#   guard-untested — NEW here: a guard-shaped PUBLIC symbol added by the diff with
#   no test anywhere in the tree that so much as names it. This is the STATICALLY
#   decidable slice of GUARD-NOT-PINNED.
#
# NOT MECHANICAL, and this script does not pretend otherwise: the other slice of
# GUARD-NOT-PINNED (a test EXISTS but cannot discriminate — it asserts a bound, a
# type, or a marker string rather than the behaviour) is only decidable by MUTATING
# the guard and re-running. Likewise CLAIM-vs-CODE needs a human/model to read the
# prose against the code. For those two, this script prints the obligation and
# requires the author to have executed it; it cannot and must not "pass" them.
#
# This script LOWERS NO BAR. Every mechanical check it runs is a gate that already
# blocks the merge; running it earlier only changes WHERE the author learns.
#
# SUPPRESSION: an inline `preflight-allow: <check> — <why>` marker on (or directly
# above) the offending line, mirroring the repo's existing `privacy-claims-allow:`
# convention. A suppression without a `— why` clause does NOT suppress.
#
# DIFF SOURCE: `git diff --name-status origin/<base>...HEAD` plus `git diff -U0`
# for added lines; or a hermetic fixture (--changed-files / --added-lines) in tests.

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# ---------------------------------------------------------------------------
# guard-untested: the statically decidable slice of GUARD-NOT-PINNED.
# ---------------------------------------------------------------------------
# Narrow ON PURPOSE. Only a guard-shaped symbol on a real SURFACE is flagged:
#   Rust   — `pub fn` / `pub(crate) fn` in crates/*/src/**.rs
#   Python — a module-level `def` (column 0) in scripts/*.py
# A private Rust `fn` or a nested Python `def` is NOT flagged: those are reached
# through their caller and the coverage ratchet already speaks to them. The point
# of this check is the surface a reviewer will ask "what test reds if you delete
# this?" about.
GUARD_STEMS = (
    "validate",
    "verify",
    "check",
    "assert",
    "require",
    "reject",
    "ensure",
    "forbid",
    "refuse",
    "guard",
)
# `fn validate_x` / `fn check_x` / `fn x_guard` / `fn is_x_valid` / `fn x_is_allowed`
_STEM = "|".join(GUARD_STEMS)
RUST_GUARD_RE = re.compile(
    r"^\s*pub(?:\s*\([^)]*\))?\s+(?:async\s+)?(?:const\s+|unsafe\s+|extern\s+\"[^\"]*\"\s+)*"
    rf"fn\s+(?P<name>(?:{_STEM})\w*|\w*_(?:{_STEM})\w*|is_\w+_(?:valid|safe|allowed|trusted|pinned))\s*[(<]"
)
PY_GUARD_RE = re.compile(
    rf"^def\s+(?P<name>_?(?:{_STEM})\w*|_?\w*_(?:{_STEM})\w*|_?is_\w+_(?:valid|safe|allowed|trusted|pinned))\s*\("
)

RUST_SRC_RE = re.compile(r"^crates/[^/]+/src/.*\.rs$")
PY_SCRIPT_RE = re.compile(r"^scripts/[^/]*\.py$")

# Where a test that "names the guard" is allowed to live.
TEST_PATH_HINTS = (
    "/tests/",
    "scripts/tests/",
    "/benches/",
)
TEST_NAME_RE = re.compile(r"(^|/)(test_[^/]*\.py|[^/]*_test\.py|[^/]*\.rs)$")

# The separator MUST be surrounded by whitespace. Without that, `[a-z0-9-]+`
# backtracks and eats its own hyphen: `preflight-allow: guard-untested —` parsed as
# check=`guard`, why=`untested —`, i.e. a marker with NO reason silently produced a
# well-formed match for a check name nobody wrote. Found by mutation M3 (the
# `why.strip()` conjunct survived deletion because it was unreachable — the test
# that "proved" a missing reason does not suppress was actually passing on the
# accidental check-name mismatch). Requiring `\s+[-—]\s+` makes the reason
# genuinely mandatory and makes a hyphenated check name parse whole.
SUPPRESS_RE = re.compile(r"preflight-allow:\s*(?P<check>[a-z0-9][a-z0-9-]*)\s+[-—]\s+(?P<why>\S.*)")


def suppressed(lines: list[str], idx: int, check: str) -> bool:
    """A `preflight-allow: <check> — <why>` marker on the line or the one above it.

    A marker with no `— why` clause does NOT suppress: the whole value of the
    escape hatch is the recorded reason. That is enforced by SUPPRESS_RE itself
    (the `\\S.*` reason group is not optional), so there is deliberately NO
    second `why` test here — a redundant conjunct would be unreachable code
    masquerading as a guard.
    """
    for probe in (idx, idx - 1):
        if probe < 0 or probe >= len(lines):
            continue
        m = SUPPRESS_RE.search(lines[probe])
        if m and m.group("check") == check:
            return True
    return False


@dataclass
class Finding:
    check: str
    path: str
    detail: str
    fix: str


@dataclass
class Result:
    findings: list[Finding] = field(default_factory=list)
    ran: list[str] = field(default_factory=list)
    skipped: list[str] = field(default_factory=list)

    @property
    def ok(self) -> bool:
        return not self.findings


def added_symbols(added_lines: dict[str, list[str]]) -> list[tuple[str, str]]:
    """(path, symbol) for every guard-shaped surface symbol ADDED by the diff."""
    out: list[tuple[str, str]] = []
    for path, lines in sorted(added_lines.items()):
        if RUST_SRC_RE.match(path):
            rx = RUST_GUARD_RE
        elif PY_SCRIPT_RE.match(path):
            rx = PY_GUARD_RE
        else:
            continue
        for i, line in enumerate(lines):
            m = rx.match(line)
            if not m:
                continue
            if suppressed(lines, i, "guard-untested"):
                continue
            out.append((path, m.group("name")))
    return out


SELFTEST_MARKER_RE = re.compile(r"--self-test|def\s+self_test\b")


def test_corpus(root: Path) -> list[tuple[str, str]]:
    """(relpath, text) for every file that can host a test naming a guard.

    Three kinds of host, matching how this repo actually writes tests:
      * anything under a tests/ (or benches/) directory;
      * Rust source carrying `#[cfg(test)]` — a unit test lives inside the very
        file that defines the guard;
      * a `scripts/*.py` carrying an in-file `--self-test` block — the dominant
        convention here (check-readme-template.py, check-terminology.py, …), and
        the one whose omission produced a MEASURED false positive on
        scripts/release-interval-guard.py::check_version_group, which is tested
        at its own self_test() and by nothing under tests/.

    An ordinary call site never counts: a src file with no test marker is not a
    host, so `foo()` being invoked somewhere does not satisfy the guard.
    """
    corpus: list[tuple[str, str]] = []
    try:
        tracked = subprocess.run(
            ["git", "ls-files"], cwd=root, capture_output=True, text=True, check=True
        ).stdout.splitlines()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return corpus
    for rel in tracked:
        is_test_dir = any(h in "/" + rel for h in TEST_PATH_HINTS)
        is_rust_src = RUST_SRC_RE.match(rel) is not None
        is_py_script = PY_SCRIPT_RE.match(rel) is not None
        if not (is_test_dir or is_rust_src or is_py_script):
            continue
        if not (is_py_script or TEST_NAME_RE.search(rel)):
            continue
        try:
            text = (root / rel).read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        if is_rust_src and not is_test_dir and "#[cfg(test)]" not in text:
            continue
        if is_py_script and not is_test_dir and not SELFTEST_MARKER_RE.search(text):
            continue
        corpus.append((rel, text))
    return corpus


def check_guard_untested(added_lines: dict[str, list[str]], root: Path) -> list[Finding]:
    symbols = added_symbols(added_lines)
    if not symbols:
        return []
    corpus = test_corpus(root)
    findings: list[Finding] = []
    for path, name in symbols:
        word = re.compile(r"\b" + re.escape(name) + r"\b")
        if any(word.search(text) for _rel, text in corpus):
            continue
        findings.append(
            Finding(
                check="guard-untested",
                path=path,
                detail=f"new guard-shaped surface `{name}` is named by no test in the tree",
                fix=(
                    f"add a test that FAILS when `{name}` is deleted or inverted "
                    f"(not one that merely calls it), or annotate the definition with "
                    f"`preflight-allow: guard-untested — <why>`"
                ),
            )
        )
    return findings


# ---------------------------------------------------------------------------
# Delegated gates: the scripts the repo already owns and the briefs never name.
# ---------------------------------------------------------------------------
@dataclass
class Delegate:
    name: str
    argv: list[str]
    #  None  -> always run; otherwise run only if some changed path matches.
    path_filter: re.Pattern[str] | None = None
    pass_changed_files: bool = False
    pass_changed_paths: bool = False


DELEGATES = [
    Delegate("G1 new-crate-completeness", ["python3", "scripts/gate-new-crate.py"],
             pass_changed_files=True),
    Delegate("G2 public-api-to-skill", ["python3", "scripts/gate-api-skill.py"],
             pass_changed_files=True),
    Delegate("G6 new-config-to-docs", ["python3", "scripts/check-config-documented.py"],
             pass_changed_files=True),
    Delegate("no-perf-numbers", ["python3", "scripts/check-no-perf-numbers.py", "--enforce"],
             path_filter=re.compile(r"\.(md|typ)$"), pass_changed_paths=True),
    Delegate("readme-template", ["python3", "scripts/check-readme-template.py", "--enforce"],
             path_filter=re.compile(r"^crates/[^/]+/README\.md$"), pass_changed_paths=True),
    Delegate("privacy-claims", ["bash", "scripts/check-privacy-claims.sh"]),
]


def run_delegates(changed: list[str], root: Path, changed_files_path: Path | None) -> Result:
    res = Result()
    for d in DELEGATES:
        matched = [p for p in changed if d.path_filter.search(p)] if d.path_filter else changed
        if d.path_filter and not matched:
            res.skipped.append(f"{d.name} (no matching path in the diff)")
            continue
        argv = list(d.argv)
        if d.pass_changed_files and changed_files_path is not None:
            argv += ["--changed-files", str(changed_files_path)]
        if d.pass_changed_paths:
            existing = [p for p in matched if (root / p).exists()]
            if not existing:
                res.skipped.append(f"{d.name} (all matching paths deleted)")
                continue
            argv += existing
        if not (root / argv[1]).exists():
            res.skipped.append(f"{d.name} (script absent at {argv[1]})")
            continue
        proc = subprocess.run(argv, cwd=root, capture_output=True, text=True)
        res.ran.append(d.name)
        if proc.returncode != 0:
            tail = (proc.stdout + proc.stderr).strip().splitlines()
            res.findings.append(
                Finding(
                    check=d.name,
                    path="(diff)",
                    detail="\n      ".join(tail[-12:]) or f"exit {proc.returncode}",
                    fix=f"reproduce with: {' '.join(argv)}",
                )
            )
    return res


# ---------------------------------------------------------------------------
# The obligations no script can decide. Printed, never auto-passed.
# ---------------------------------------------------------------------------
NON_MECHANICAL = """\
NOT CHECKED BY THIS SCRIPT — you must execute these yourself before opening the PR.
They are the two largest preventable review-failure classes in the verdict corpus
(GUARD-NOT-PINNED 63 findings, CLAIM-vs-CODE 67 findings) and neither is decidable
by static analysis:

  1. MUTATE YOUR HEADLINE GUARD. Take the feature named in your PR title. DELETE or
     INVERT it in the worktree and RUN the suite. If nothing goes red, your test is
     vacuous — that is a blocking defect, and it is the single most common one the
     reviewers find. Do not reason about it; execute it. Report which test died.
     (`guard-untested` above only catches a guard with NO test at all. A test that
     exists but asserts a bound, a type, or a marker string instead of the behaviour
     passes this script and fails review.)

  2. READ YOUR OWN PROSE AGAINST YOUR OWN DIFF. For every line of documentation,
     rustdoc, README, SKILL.md, comment, research record or PR-body claim you added:
     point at the code in THIS diff that makes it true. If you cannot, delete the
     sentence or fix the code. Overclaiming is blocking. The corpus is full of
     diffs whose docs describe a module, flag, constant or test file that the diff
     does not contain.
"""


def collect_diff(base: str, root: Path) -> tuple[list[str], dict[str, list[str]]]:
    merge_base = subprocess.run(
        ["git", "merge-base", base, "HEAD"], cwd=root, capture_output=True, text=True
    ).stdout.strip() or base
    names = subprocess.run(
        ["git", "diff", "--name-only", "--diff-filter=d", f"{merge_base}...HEAD"],
        cwd=root, capture_output=True, text=True, check=True,
    ).stdout.split()
    raw = subprocess.run(
        ["git", "diff", "-U0", f"{merge_base}...HEAD"],
        cwd=root, capture_output=True, text=True, check=True,
    ).stdout
    return names, parse_added(raw)


def parse_added(unified: str) -> dict[str, list[str]]:
    """path -> added lines (no leading '+'). `-U0` so no context leaks in."""
    added: dict[str, list[str]] = {}
    path: str | None = None
    for line in unified.splitlines():
        if line.startswith("+++ b/"):
            path = line[6:]
            added.setdefault(path, [])
        elif line.startswith("+++ "):
            path = None
        elif path is not None and line.startswith("+") and not line.startswith("+++"):
            added[path].append(line[1:])
    return {k: v for k, v in added.items() if v}


def report(res: Result, quiet: bool) -> int:
    if res.ran and not quiet:
        print("preflight: ran  " + ", ".join(res.ran))
    if res.skipped and not quiet:
        print("preflight: skip " + "; ".join(res.skipped))
    if res.ok:
        print("\npreflight: PASS — every mechanical pre-submit gate is green on this diff.")
    else:
        print(f"\npreflight: FAIL — {len(res.findings)} mechanical finding(s):\n")
        for f in res.findings:
            print(f"  [{f.check}] {f.path}")
            print(f"      {f.detail}")
            print(f"      fix: {f.fix}\n")
    print()
    print(NON_MECHANICAL)
    return 0 if res.ok else 1


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="Pre-submit gate runner — run every mechanical merge-gate against "
                    "YOUR diff, in YOUR worktree, before the PR exists."
    )
    ap.add_argument("--base", default="origin/main", help="base ref (default origin/main)")
    ap.add_argument("--changed-files", help="hermetic input: file of changed paths")
    ap.add_argument("--added-lines", help="hermetic input: a unified diff to read added lines from")
    ap.add_argument("--root", default=str(REPO_ROOT), help="repo root (tests inject a fixture tree)")
    ap.add_argument("--only", choices=["guard-untested", "delegates"],
                    help="run a single check group (tests + fast local loops)")
    ap.add_argument("--quiet", action="store_true")
    ap.add_argument("--self-test", action="store_true", help="run the hermetic self-test and exit")
    args = ap.parse_args(argv)

    if args.self_test:
        return self_test()

    root = Path(args.root).resolve()
    changed_files_path: Path | None = None
    if args.changed_files:
        changed_files_path = Path(args.changed_files).resolve()
        changed = [
            ln.split("\t")[-1].strip()
            for ln in changed_files_path.read_text().splitlines()
            if ln.strip()
        ]
        added = parse_added(Path(args.added_lines).read_text()) if args.added_lines else {}
    else:
        changed, added = collect_diff(args.base, root)

    res = Result()
    if args.only != "delegates":
        res.findings += check_guard_untested(added, root)
    if args.only != "guard-untested":
        sub = run_delegates(changed, root, changed_files_path)
        res.findings += sub.findings
        res.ran += sub.ran
        res.skipped += sub.skipped
    if args.only != "delegates":
        res.ran.append("guard-untested")
    return report(res, args.quiet)


# ---------------------------------------------------------------------------
# Hermetic self-test (repo convention: a --self-test step wired HARD in
# docs-quality.yml, so the checker cannot silently rot).
# ---------------------------------------------------------------------------
def self_test() -> int:
    import tempfile

    failures: list[str] = []

    def chk(name: str, got: object, want: object) -> None:
        if got != want:
            failures.append(f"{name}: got {got!r}, want {want!r}")
        print(f"  {'ok  ' if got == want else 'FAIL'} {name}")

    # --- added_symbols: what is and is not a guard-shaped SURFACE ---
    rust = "crates/sparq-x/src/lib.rs"
    pyf = "scripts/thing.py"
    chk("rust pub fn validate_* is a guard",
        added_symbols({rust: ["pub fn validate_envelope(e: &E) -> bool {"]}),
        [(rust, "validate_envelope")])
    chk("rust pub fn *_guard is a guard",
        added_symbols({rust: ["pub fn admission_guard(x: u8) {"]}),
        [(rust, "admission_guard")])
    chk("rust pub fn is_*_valid is a guard",
        added_symbols({rust: ["pub fn is_lease_valid(l: &L) -> bool {"]}),
        [(rust, "is_lease_valid")])
    chk("rust pub(crate) fn check_* is a guard",
        added_symbols({rust: ["pub(crate) fn check_bounds(n: usize) {"]}),
        [(rust, "check_bounds")])
    chk("rust PRIVATE fn check_* is NOT a surface",
        added_symbols({rust: ["fn check_bounds(n: usize) {"]}), [])
    chk("rust pub fn with no guard stem is NOT a guard",
        added_symbols({rust: ["pub fn render_row(r: &R) {"]}), [])
    chk("python module-level def validate_* is a guard",
        added_symbols({pyf: ["def validate_lease(x):"]}), [(pyf, "validate_lease")])
    chk("python NESTED def is NOT a surface",
        added_symbols({pyf: ["    def validate_lease(x):"]}), [])
    chk("a non-src, non-scripts path is out of scope",
        added_symbols({"docs/x.rs": ["pub fn validate_envelope() {"]}), [])

    # --- suppression semantics: a reason is MANDATORY ---
    chk("suppression with a reason suppresses",
        added_symbols({rust: [
            "// preflight-allow: guard-untested — proven by the kani harness",
            "pub fn validate_envelope() {",
        ]}), [])
    chk("suppression WITHOUT a reason does NOT suppress",
        added_symbols({rust: [
            "// preflight-allow: guard-untested —",
            "pub fn validate_envelope() {",
        ]}), [(rust, "validate_envelope")])
    chk("suppression for a DIFFERENT check does not suppress",
        added_symbols({rust: [
            "// preflight-allow: no-perf-numbers — unrelated",
            "pub fn validate_envelope() {",
        ]}), [(rust, "validate_envelope")])

    # --- parse_added: -U0 unified diff, added lines only ---
    diff = (
        "diff --git a/scripts/a.py b/scripts/a.py\n"
        "--- a/scripts/a.py\n"
        "+++ b/scripts/a.py\n"
        "@@ -1,0 +2 @@\n"
        "+def validate_x(v):\n"
        "-def gone(v):\n"
    )
    chk("parse_added keeps only + lines", parse_added(diff), {"scripts/a.py": ["def validate_x(v):"]})
    chk("parse_added ignores the +++ header",
        "+++ b/scripts/a.py" in parse_added(diff)["scripts/a.py"], False)

    # --- END-TO-END on a real throwaway git tree: the check must RED on a guard
    #     with no test, and GREEN once a test names it. This is the behaviour the
    #     mutation test in scripts/tests/test_preflight.py deletes the detector on.
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        (root / "crates/sparq-x/src").mkdir(parents=True)
        (root / "crates/sparq-x/src/lib.rs").write_text("pub fn validate_envelope() {}\n")
        subprocess.run(["git", "init", "-q"], cwd=root, check=True)
        subprocess.run(["git", "add", "-A"], cwd=root, check=True)
        added = {"crates/sparq-x/src/lib.rs": ["pub fn validate_envelope() {}"]}
        chk("e2e: guard with NO test -> 1 finding",
            len(check_guard_untested(added, root)), 1)

        (root / "crates/sparq-x/tests").mkdir(parents=True)
        (root / "crates/sparq-x/tests/t.rs").write_text(
            "#[test]\nfn t() { sparq_x::validate_envelope(); }\n"
        )
        subprocess.run(["git", "add", "-A"], cwd=root, check=True)
        chk("e2e: guard named by a test -> 0 findings",
            len(check_guard_untested(added, root)), 0)

        # An unrelated test naming a DIFFERENT symbol must not satisfy the guard —
        # otherwise the check is vacuous (any test file in the repo would pass it).
        (root / "crates/sparq-x/tests/t.rs").write_text(
            "#[test]\nfn t() { sparq_x::something_else(); }\n"
        )
        subprocess.run(["git", "add", "-A"], cwd=root, check=True)
        chk("e2e: an unrelated test does NOT satisfy the guard",
            len(check_guard_untested(added, root)), 1)

    print()
    if failures:
        print(f"preflight --self-test: FAIL ({len(failures)})")
        for f in failures:
            print("  " + f)
        return 1
    print("preflight --self-test: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
