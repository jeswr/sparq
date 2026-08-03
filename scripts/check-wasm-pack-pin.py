#!/usr/bin/env python3
# [OPUS-5] CI lint (#5305): every wasm-pack install site in .github/workflows must
# pin the ONE version declared in .github/wasm-pack-version.
#
# WHY THIS GATE EXISTS — the version skew it replaces:
# ci.yml's `wasm` job installed wasm-pack via `jetli/wasm-pack-action` pinned to
# `version: v0.13.1`, js.yml pinned `v0.15.0`, and sixteen other lanes ran
# `cargo install wasm-pack --locked`, which resolves to whatever the CURRENT release
# is (0.15.0 at the time of writing). Nothing goes red when those drift apart: the
# `wasm-pack test --node` headless suites just validate the #[wasm_bindgen] exports
# under an older wasm-pack — and so an older bundled wasm-bindgen CLI — than the one
# that builds the artifact publish.yml ships. The failure mode is a silent hole in
# what the wasm gate actually covers, which is exactly the shape that needs a lint
# rather than a comment.
#
# RULE: scan .github/workflows/*.yml (+ *.yaml). Let CANON be the version in
# .github/wasm-pack-version. FAIL if:
#   * a step `uses: jetli/wasm-pack-action@<ref>` whose `version:` input is not
#     exactly `v<CANON>` (a floating `latest` also fails — it re-adds an
#     unauthenticated api.github.com release lookup, cf. sq-khm3f), or
#   * a `cargo install wasm-pack ...` command does not carry `--version <CANON>`
#     (an unversioned `--locked` install floats to the current release), or
#   * there are NO install sites at all — a rename or a refactor that moved every
#     install out from under this lint would otherwise leave it passing vacuously.
#
# SCOPE: .github/workflows only. The `cargo install wasm-pack --locked` strings in
# js/guardrails/prepare-build.mjs and packages/eyereasoner-compat/guardrails/
# prepare-build.mjs are human-facing "install the toolchain" hints for a local
# checkout, not CI lanes, and are deliberately NOT asserted here — this file's
# silence about them is not a claim that they are pinned. This gate also says
# nothing about the install MECHANISM (prebuilt binary vs source compile); that
# is scripts/tests/test_js_wasm_pack_install.py's job, for the js lane only.
#
# The step splitter is single-sourced from scripts/check-install-action-tool.py
# rather than re-implemented. stdlib-only (no PyYAML): this runs in the
# docs-quality `ci-scripts` bucket, which installs no Python deps.
#
# EXIT: 0 when every site pins CANON; 1 with a per-offence message otherwise.
#
# Usage:
#   check-wasm-pack-pin.py                  # scan .github/workflows
#   check-wasm-pack-pin.py --root <dir>     # scan <dir>/.github/workflows
#   check-wasm-pack-pin.py --self-test      # hermetic logic self-test

from __future__ import annotations

import argparse
import importlib.util
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

ACTION = "jetli/wasm-pack-action"
VERSION_FILE = Path(".github") / "wasm-pack-version"
# A bare semver, e.g. `0.15.0` — the form cargo's `--version` takes.
_SEMVER_RE = re.compile(r"^\d+\.\d+\.\d+$")
# Shell command separators: a `run:` line may chain several commands, and only the
# `cargo install` ones are ours to read.
_SHELL_SPLIT_RE = re.compile(r"&&|\|\||[;|]")
# The options that pin the version cargo installs (`--vers` is the older spelling).
_CARGO_VERSION_OPTS = frozenset({"--version", "--vers"})
# Options whose value is the NEXT token when not written as `--opt=value`. Needed so
# that value is never mistaken for the crate argument; the list only has to cover
# what a `cargo install` line plausibly carries, and an unlisted option is simply
# treated as taking no value (which can only make us see MORE candidate crates).
_CARGO_VALUE_OPTS = frozenset(
    {
        "--git", "--branch", "--tag", "--rev", "--path", "--root", "--registry",
        "--index", "--profile", "--target", "--target-dir", "--features", "-F",
        "--bin", "--example", "--jobs", "-j", "--config", "-Z", "--color",
        "--message-format", "--artifact-dir", "--manifest-path",
    }
)


def _load(name: str, filename: str):
    spec = importlib.util.spec_from_file_location(
        name, REPO_ROOT / "scripts" / filename
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


_steps = _load("check_install_action_tool", "check-install-action-tool.py")


def read_canonical_version(root: Path) -> str:
    """The single declared wasm-pack version: the first non-comment, non-blank line
    of .github/wasm-pack-version."""
    path = root / VERSION_FILE
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if not _SEMVER_RE.match(line):
            raise ValueError(
                f"{VERSION_FILE}: expected a bare semver (e.g. 0.15.0), got {line!r}"
            )
        return line
    raise ValueError(f"{VERSION_FILE}: no version line found")


def _step_version_input(block: list[str]) -> str | None:
    """The step's `version:` input value, or None. Comment lines are skipped: these
    workflows are heavily commented and the splitter keeps comments, so a prose
    mention must never be read as an input."""
    for raw in block:
        stripped = raw.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if stripped.startswith("version:"):
            return stripped.split(":", 1)[1].strip()
    return None


def _code_lines(text: str) -> list[tuple[int, str]]:
    """(1-based line number, line) with `#` comments and blanks dropped, so a
    commented-out or merely described command is never mistaken for a live step."""
    return [
        (i, ln)
        for i, ln in enumerate(text.splitlines(), start=1)
        if ln.strip() and not ln.strip().startswith("#")
    ]


def _cargo_install_args(line: str) -> list[list[str]]:
    """The argument tokens of every `cargo install` invocation on `line`.

    The line is split on shell separators first so that a chained command
    (`cargo install a && cargo install b`) contributes its own argument list rather
    than one merged blob."""
    invocations: list[list[str]] = []
    for chunk in _SHELL_SPLIT_RE.split(line):
        toks = chunk.split()
        for i, tok in enumerate(toks):
            # `cargo`, `~/.cargo/bin/cargo`, ... optionally with a `+toolchain`.
            if tok.rsplit("/", 1)[-1] != "cargo":
                continue
            j = i + 1
            while j < len(toks) and toks[j].startswith("+"):
                j += 1
            if j < len(toks) and toks[j] == "install":
                invocations.append(toks[j + 1 :])
    return invocations


def _wasm_pack_pin(args: list[str]) -> tuple[bool, str | None]:
    """(is this a wasm-pack install, the version it pins) for one `cargo install`.

    Cargo takes its options in ANY order, so `cargo install --locked wasm-pack` and
    `cargo install wasm-pack --locked` are the same command — the crate argument is
    therefore found by walking positionals, not by matching the token right after
    `install`. Option VALUES are stepped over so `--version 0.15.0` never reads as a
    crate name."""
    crates: list[str] = []
    version: str | None = None
    i = 0
    while i < len(args):
        tok = args[i]
        if tok.startswith("-"):
            name, sep, inline = tok.partition("=")
            if name in _CARGO_VERSION_OPTS:
                if sep:
                    version = inline
                elif i + 1 < len(args):
                    version = args[i + 1]
                    i += 1
            elif not sep and name in _CARGO_VALUE_OPTS:
                i += 1
        else:
            crates.append(tok)
        i += 1
    return "wasm-pack" in crates, version


def scan_text(text: str, canon: str) -> tuple[list[str], int]:
    """Check one workflow file's text against `canon`.

    Returns (offences, site_count) — offences are human-readable strings, and
    site_count is how many wasm-pack install sites were seen (used by the caller to
    reject a vacuous pass)."""
    offences: list[str] = []
    sites = 0
    # Line numbers for the `uses:` steps: the splitter returns line TEXT, so locate
    # each action line in the file to make the offence report navigable (gui.yml has
    # five byte-identical install lines).
    action_lines = [
        i for i, ln in _code_lines(text) if f"uses: {ACTION}@" in ln
    ]

    for block in _steps.split_steps(text):
        uses = _steps._step_uses(block)
        if uses is None or uses[0] != ACTION:
            continue
        where = action_lines[sites] if sites < len(action_lines) else "?"
        sites += 1
        got = _step_version_input(block)
        want = f"v{canon}"
        if got != want:
            offences.append(
                f"{where}: {ACTION} pins `version: {got}`, want `{want}`"
            )

    for lineno, line in _code_lines(text):
        for args in _cargo_install_args(line):
            is_wasm_pack, version = _wasm_pack_pin(args)
            if not is_wasm_pack:
                continue
            sites += 1
            if version is None:
                offences.append(
                    f"{lineno}: `{line.strip()}` has no `--version` "
                    "(floats to the current release)"
                )
            elif version != canon:
                offences.append(
                    f"{lineno}: `{line.strip()}` pins --version {version}, "
                    f"want {canon}"
                )

    return offences, sites


def scan_workflows(root: Path, canon: str) -> tuple[list[tuple[Path, str]], int]:
    wf_dir = root / ".github" / "workflows"
    results: list[tuple[Path, str]] = []
    sites = 0
    if not wf_dir.is_dir():
        return results, sites
    for path in sorted(wf_dir.iterdir()):
        if path.suffix not in (".yml", ".yaml"):
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="ignore")
        except OSError:
            continue
        offences, n = scan_text(text, canon)
        sites += n
        for offence in offences:
            results.append((path, offence))
    return results, sites


# --------------------------------------------------------------------------- tests
_GOOD = """\
jobs:
  wasm:
    steps:
      - uses: actions/checkout@deadbeefdeadbeefdeadbeefdeadbeefdeadbeef # v7.0.0
      - name: Install wasm-pack
        uses: jetli/wasm-pack-action@0d096b08b4e5a7de8c28de67e11e945404e9eefa # v0.4.0
        with:
          version: v0.15.0
      - name: Install wasm-pack
        run: cargo install wasm-pack --version 0.15.0 --locked
"""

# The skew #5305 reports: the action pinned two minor versions behind.
_BAD_ACTION_SKEW = """\
jobs:
  wasm:
    steps:
      - uses: jetli/wasm-pack-action@0d096b08b4e5a7de8c28de67e11e945404e9eefa # v0.4.0
        with:
          version: v0.13.1
"""

_BAD_ACTION_LATEST = """\
jobs:
  wasm:
    steps:
      - uses: jetli/wasm-pack-action@0d096b08b4e5a7de8c28de67e11e945404e9eefa # v0.4.0
        with:
          version: latest
"""

_BAD_CARGO_UNPINNED = """\
jobs:
  site:
    steps:
      - name: Install wasm-pack
        run: cargo install wasm-pack --locked
"""

_BAD_CARGO_WRONG = """\
jobs:
  site:
    steps:
      - run: cargo install wasm-pack --version 0.13.1 --locked
"""

# `--version=X` is the same pin, written the other way.
_GOOD_CARGO_EQUALS = """\
jobs:
  site:
    steps:
      - run: cargo install wasm-pack --version=0.15.0 --locked
"""

# Cargo accepts its options BEFORE the crate argument, so these are the same
# commands as the three above with the tokens reordered, and must be judged the same.
_BAD_CARGO_UNPINNED_FLAG_FIRST = """\
jobs:
  site:
    steps:
      - run: cargo install --locked wasm-pack
"""

_BAD_CARGO_WRONG_FLAG_FIRST = """\
jobs:
  site:
    steps:
      - run: cargo install --version 0.13.1 --locked wasm-pack
"""

_GOOD_CARGO_FLAG_FIRST = """\
jobs:
  site:
    steps:
      - run: cargo install --version 0.15.0 --locked wasm-pack
"""

# A COMMENTED mention of the old command (js.yml has one in its rationale block)
# must not be read as a live install site.
_GOOD_COMMENT_ONLY = """\
jobs:
  js:
    steps:
      # 0.15.0 is what `cargo install wasm-pack` resolved to here, so the pin is
      # version-neutral today.
      - uses: jetli/wasm-pack-action@0d096b08b4e5a7de8c28de67e11e945404e9eefa # v0.4.0
        with:
          version: v0.15.0
"""

# A `version:` input belonging to a DIFFERENT action must not be attributed to the
# wasm-pack step (the splitter's sibling-leak case).
_GOOD_NO_LEAK = """\
jobs:
  js:
    steps:
      - uses: jetli/wasm-pack-action@0d096b08b4e5a7de8c28de67e11e945404e9eefa # v0.4.0
        with:
          version: v0.15.0
      - uses: some/other-action@deadbeefdeadbeefdeadbeefdeadbeefdeadbeef # v1
        with:
          version: v9.9.9
"""


def self_test() -> int:
    canon = "0.15.0"
    cases = [
        ("good (action + cargo both at canon)", _GOOD, 0, 2),
        ("bad (action two minors behind — the #5305 skew)", _BAD_ACTION_SKEW, 1, 1),
        ("bad (action floating `latest`)", _BAD_ACTION_LATEST, 1, 1),
        ("bad (cargo install with no --version)", _BAD_CARGO_UNPINNED, 1, 1),
        ("bad (cargo install pinning another version)", _BAD_CARGO_WRONG, 1, 1),
        ("good (--version=X spelling)", _GOOD_CARGO_EQUALS, 0, 1),
        (
            "bad (options before the crate, no --version)",
            _BAD_CARGO_UNPINNED_FLAG_FIRST,
            1,
            1,
        ),
        (
            "bad (options before the crate, wrong --version)",
            _BAD_CARGO_WRONG_FLAG_FIRST,
            1,
            1,
        ),
        ("good (options before the crate, at canon)", _GOOD_CARGO_FLAG_FIRST, 0, 1),
        ("good (commented mention is not a site)", _GOOD_COMMENT_ONLY, 0, 1),
        ("good (sibling action's version: does not leak)", _GOOD_NO_LEAK, 0, 1),
    ]
    failures = 0
    for label, text, want_offences, want_sites in cases:
        offences, sites = scan_text(text, canon)
        ok = len(offences) == want_offences and sites == want_sites
        print(
            f"  [{'PASS' if ok else 'FAIL'}] {label}: {len(offences)} offence(s) "
            f"(want {want_offences}), {sites} site(s) (want {want_sites})"
        )
        if not ok:
            failures += 1

    # The version-file reader: a bare semver wins, comments are skipped, junk fails.
    got = read_canonical_version(REPO_ROOT)
    ok = _SEMVER_RE.match(got) is not None
    print(f"  [{'PASS' if ok else 'FAIL'}] {VERSION_FILE} parses to {got!r}")
    if not ok:
        failures += 1

    if failures:
        print(f"\nself-test: {failures} case(s) FAILED")
        return 1
    print("\nself-test: all cases PASS")
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="Fail if any wasm-pack install site in .github/workflows "
        "disagrees with .github/wasm-pack-version (#5305)."
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

    root = Path(args.root)
    canon = read_canonical_version(root)
    offences, sites = scan_workflows(root, canon)

    if sites == 0:
        print("wasm-pack pin gate: FAIL\n")
        print(
            "No wasm-pack install site was found in .github/workflows. Either the wasm\n"
            "lanes were removed, or an install was rewritten into a shape this lint no\n"
            "longer recognises — in which case the lint is now VACUOUS and would pass\n"
            "over any future skew. Fix the lint (or delete it with the lanes)."
        )
        return 1

    if not offences:
        print(
            f"wasm-pack pin gate: PASS — all {sites} install site(s) in "
            f".github/workflows pin wasm-pack {canon} ({VERSION_FILE})."
        )
        return 0

    print("wasm-pack pin gate: FAIL\n")
    print(
        f"These wasm-pack install sites disagree with {VERSION_FILE} ({canon}).\n"
        "A lane that installs a different wasm-pack validates the #[wasm_bindgen]\n"
        "exports under a different wasm-bindgen CLI than the one that builds the\n"
        "artifact we publish — a silent hole in the wasm gate (#5305). Bring each to\n"
        f"{canon}, or bump {VERSION_FILE} and every lane together:\n"
    )
    for path, offence in offences:
        rel = path.relative_to(root) if path.is_relative_to(root) else path
        print(f"    - {rel}:{offence}")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
