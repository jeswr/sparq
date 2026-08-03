#!/usr/bin/env python3
# [OPUS-5] Structural guard — every npm package under `packages/*` whose tests can
# exist must have a CI leg that actually RUNS them (sparq-org/sparq#5843, follow-up
# to #5742's gate G1).
#
# WHY THIS EXISTS. Gate G1 (scripts/gate-new-crate.py) is the DIFF-scoped gate
# that a newly-added surface ships its maintenance artifacts in the same PR;
# sparq-org/sparq#5742 extends it to require, for a new `packages/<x>`, a README
# and — when the package is publishable — at least one test FILE. That is the
# most a diff-scoped gate can do: it can see that a test file landed, never
# whether anything executes it. A package whose `test` script no workflow invokes
# is a false green of exactly the shape scripts/check-feature-test-execution.py
# exists to prevent on the Rust side — the tests are committed, reviewed, and
# never run. This guard closes that remaining deficiency, and it is REPO-WIDE
# rather than diff-scoped, so it also catches an EXISTING package whose CI leg is
# deleted. It stands alone rather than extending G1 precisely because the
# question is repo-wide: G1 sees only the diff.
#
# INVARIANT (fail-closed). For every directory `packages/<x>` containing a
# `package.json`:
#   (1) it resolves to an npm WORKSPACE entry — some glob in the root
#       package.json `workspaces` array matches it. A non-member is not installed
#       by the repo-root `npm ci`, so its dependencies never resolve and its tests
#       cannot run even if a workflow tried; and
#   (2) if it declares a `test` script, at least one step in some
#       `.github/workflows/*.yml` invokes THAT package's test script.
# A package with no `test` script is exempt from (2) — there is no script to
# invoke, so there is nothing that can silently not run. (Requiring a publishable
# package to HAVE tests at all is G1's job, not this guard's.)
#
# HOW COVERAGE IS DETECTED. Coverage is attributed STRUCTURALLY, never by
# substring: a bare `npm test` says nothing about which package it belongs to
# until its working directory is resolved. For every step `run:` block the
# effective working directory comes from the job's `defaults.run.working-directory`
# (or the workflow-level one), the step's own `working-directory:`, and any
# `cd <dir>` earlier in the same block. A command counts when it is an `npm test` /
# `npm run test` invocation, and it is attributed to:
#   * the package named by `-w <sel>` / `--workspace <sel>` / `--workspace=<sel>`
#     (the selector may be a directory path OR a package name), or
#   * EVERY workspace member, for `--workspaces` / `-ws`, or
#   * otherwise the package that the effective working directory IS.
# `npm run test:foo` is a DIFFERENT script and deliberately does not count.
#
# WHY A HAND-ROLLED WORKFLOW READER, NOT PyYAML. The repo parses YAML with PyYAML
# elsewhere, but PyYAML is a hash-pinned pip install available only in the jobs
# that ask for it. Keeping this guard stdlib-only lets it run in any gating job
# and lets its whole suite run with no install step. `read_workflow_steps` below
# reads only the block-style subset GitHub Actions workflows are written in, and
# is fail-closed by construction: a construct it does not recognise yields NO
# coverage, so the failure mode is a visible red gate, never a silent green.
#
# ESCAPE HATCH. scripts/check-package-test-execution.allowlist.json (optional; an
# absent file means an empty allowlist) maps a package directory name to a written
# reason. Use it only for a package that genuinely must not be CI-tested — the
# reason is the reviewable artifact, so it is ENFORCED, not merely documented: a
# key must be a bare package directory name and a value must be a non-empty,
# non-whitespace string. `{"x": ""}` / `{"x": null}` / `{"x": false}` would
# otherwise disable the whole invariant for packages/x with nothing to review, so
# a malformed entry exits 2 rather than granting a vacuous exemption.
#
# LIMITATIONS (honest):
#   * A step's `if:` condition is NOT evaluated. A leg wired behind `if: false`
#     would be counted as coverage here. Conditions are expressions over run-time
#     context, so a static scan cannot settle them; the workflow-level `paths`
#     filters that scope js.yml/gui.yml are intended wiring, not an evasion.
#   * Whether the covering workflow is GATING (vs. declared advisory in
#     .github/advisory-registry.json) is not checked — this guard answers "does
#     anything run it", not "does it block".
#   * Only npm is recognised (the repo uses npm workspaces exclusively); a yarn or
#     pnpm invocation would not be seen as coverage.
#   * Flow-style YAML (`steps: [{run: ...}]`) is not read. No workflow here uses
#     it, and because unrecognised input yields no coverage, adopting it would red
#     this gate rather than blind it.
#
# USAGE
#   python3 scripts/check-package-test-execution.py
#       Scan the real packages/ + .github/workflows/; exit 0 when every package
#       satisfies the invariant, 1 with per-package findings otherwise.
#   python3 scripts/check-package-test-execution.py --advisory
#       Report findings but always exit 0.
#
# stdlib-only. Unit suite: scripts/tests/test_check_package_test_execution.py.

from __future__ import annotations

import argparse
import fnmatch
import json
import posixpath
import re
import shlex
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
PACKAGES_DIR = REPO_ROOT / "packages"
WORKFLOWS_DIR = REPO_ROOT / ".github" / "workflows"
ALLOWLIST = REPO_ROOT / "scripts" / "check-package-test-execution.allowlist.json"

# Sentinel target meaning "npm ran the test script of every workspace member"
# (`npm test --workspaces`).
ALL_WORKSPACES = "*"

# Flags that take a SEPARATE value argument, so the value must not be mistaken for
# the script name when scanning `npm run <flags> test`.
_NPM_VALUE_FLAGS = {"-w", "--workspace", "--prefix", "-C"}


def _norm_dir(cwd: str, target: str) -> str:
    """Resolve `target` against `cwd`, both repo-relative POSIX paths."""
    joined = target if target.startswith("/") else posixpath.join(cwd, target)
    normed = posixpath.normpath(joined)
    return "." if normed in ("", ".", "/") else normed.lstrip("/")


# --------------------------------------------------------------------------- #
# Repository facts
# --------------------------------------------------------------------------- #


def workspace_globs(root_manifest: dict) -> list[str]:
    """The root package.json `workspaces` globs.

    npm also accepts the object form `{"packages": [...]}` (the yarn-compatible
    spelling), so both are read — a repo that switched spelling must not silently
    turn every package into a "not a workspace member" violation."""
    ws = root_manifest.get("workspaces")
    if isinstance(ws, dict):
        ws = ws.get("packages")
    if not isinstance(ws, list):
        return []
    return [g for g in ws if isinstance(g, str)]


def is_workspace_member(pkg_dir: str, globs: list[str]) -> bool:
    """True iff some root `workspaces` glob matches the package directory.

    fnmatch covers the `packages/*` form the repo uses; the exact-string compare
    covers the un-globbed entries (`js`, `site`, `gui/app`)."""
    pkg_dir = pkg_dir.rstrip("/")
    for g in globs:
        g = g.rstrip("/")
        if g == pkg_dir or fnmatch.fnmatch(pkg_dir, g):
            return True
    return False


def discover_packages(packages_dir: Path) -> dict[str, dict]:
    """Map `packages/<x>` -> its parsed package.json, one level deep.

    One level deep deliberately: a vendored manifest under
    `packages/<x>/node_modules/...` is not a package of this repo. An unparseable
    manifest becomes an empty dict — it then declares no `test` script, and clause
    (1) still applies, so it cannot dodge the guard by being unreadable."""
    out: dict[str, dict] = {}
    if not packages_dir.is_dir():
        return out
    for child in sorted(packages_dir.iterdir()):
        manifest = child / "package.json"
        if not manifest.is_file():
            continue
        try:
            data = json.loads(manifest.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            data = {}
        out[f"packages/{child.name}"] = data if isinstance(data, dict) else {}
    return out


def package_has_test_script(manifest: dict) -> bool:
    scripts = manifest.get("scripts")
    return isinstance(scripts, dict) and bool(scripts.get("test"))


# --------------------------------------------------------------------------- #
# npm command reading
# --------------------------------------------------------------------------- #


def _tokenise(command: str) -> list[str]:
    """Best-effort shell tokenisation. Run blocks contain `${{ }}` expressions and
    quoting that shlex can find unbalanced; a command we cannot tokenise yields no
    coverage (fail-closed)."""
    try:
        return shlex.split(command)
    except ValueError:
        return []


def npm_test_targets(argv: list[str], cwd: str) -> list[str]:
    """Given a tokenised `npm ...` command, return the targets whose `test` script
    it runs — resolved directory paths, package names, or the ALL_WORKSPACES
    sentinel. Empty when the command does not run the `test` script."""
    if not argv or argv[0] != "npm":
        return []
    rest = argv[1:]

    if rest[:1] == ["run"]:
        # `npm run [flags] <script>` — the first non-flag token is the script.
        rest = rest[1:]
        script = None
        i = 0
        while i < len(rest):
            tok = rest[i]
            if tok.startswith("-"):
                i += 2 if tok in _NPM_VALUE_FLAGS else 1
                continue
            script = tok
            break
        # `test:unit` / `testify` are DIFFERENT scripts; only `test` counts.
        if script != "test":
            return []
    elif rest[:1] != ["test"]:
        return []

    selectors: list[str] = []
    all_workspaces = False
    i = 0
    while i < len(rest):
        tok = rest[i]
        if tok in ("--workspaces", "-ws"):
            all_workspaces = True
        elif tok in ("-w", "--workspace") and i + 1 < len(rest):
            selectors.append(rest[i + 1])
            i += 1
        elif tok.startswith("--workspace="):
            selectors.append(tok.split("=", 1)[1])
        i += 1

    if all_workspaces:
        return [ALL_WORKSPACES]
    if selectors:
        # A selector is either a package NAME (kept verbatim, matched against
        # package.json `name`) or a directory path (resolved against cwd).
        return [s if s.startswith("@") else _norm_dir(cwd, s) for s in selectors]
    return [cwd]


def run_block_targets(run: str, cwd: str) -> set[str]:
    """Scan one `run:` block, tracking `cd` as it goes, and return every target
    whose npm `test` script the block invokes."""
    targets: set[str] = set()
    for line in run.splitlines():
        # Split on shell separators, left to right, so a `cd` takes effect for the
        # commands that follow it on the same line (`cd packages/x && npm test`).
        for command in re.split(r"&&|\|\||;|\|", line):
            argv = _tokenise(command.strip())
            if not argv:
                continue
            if argv[0] == "cd" and len(argv) > 1:
                cwd = _norm_dir(cwd, argv[1])
            else:
                targets.update(npm_test_targets(argv, cwd))
    return targets


# --------------------------------------------------------------------------- #
# Workflow reading (block-style subset, stdlib-only)
# --------------------------------------------------------------------------- #

# A mapping key line, after the optional leading `- ` of a sequence item.
_KEY_RE = re.compile(r"^(?P<key>[A-Za-z_][A-Za-z0-9_.-]*)\s*:(?P<rest>\s.*|)$")
# A block-scalar header: `|`, `>`, with optional chomping/indent indicators.
_BLOCK_SCALAR_RE = re.compile(r"^[|>][+-]?\d*$")


class _Line:
    """One significant workflow line, normalised so a sequence item's first key
    sits at the same indent as its sibling keys."""

    __slots__ = ("indent", "key", "rest", "is_item", "raw_indent")

    def __init__(self, raw: str) -> None:
        stripped = raw.lstrip(" ")
        self.raw_indent = len(raw) - len(stripped)
        self.is_item = bool(re.match(r"^-(\s|$)", stripped))
        if self.is_item:
            body = re.sub(r"^-\s*", "", stripped)
            # The dash and its following space(s) are part of the item's indent.
            self.indent = self.raw_indent + (len(stripped) - len(body))
        else:
            body = stripped
            self.indent = self.raw_indent
        m = _KEY_RE.match(body)
        self.key = m.group("key") if m else None
        self.rest = m.group("rest").strip() if m else ""


def _scalar(text: str) -> str:
    """Unwrap a plain/quoted YAML scalar, dropping a trailing ` # comment`."""
    text = text.strip()
    if text[:1] in ("'", '"') and text[-1:] == text[:1] and len(text) >= 2:
        return text[1:-1]
    return re.split(r"\s+#", text, maxsplit=1)[0].strip()


def _block_end(lines: list[_Line | None], start: int, indent: int) -> int:
    """Index one past the last line belonging to the block opened at `start`."""
    i = start + 1
    while i < len(lines):
        ln = lines[i]
        if ln is not None and ln.raw_indent <= indent:
            break
        i += 1
    return i


def _defaults_working_directory(lines: list[_Line | None], lo: int, hi: int) -> str | None:
    """The `defaults: { run: { working-directory: X } }` of the region [lo, hi)."""
    for i in range(lo, hi):
        ln = lines[i]
        if ln is None or ln.key != "defaults" or ln.rest:
            continue
        d_end = _block_end(lines, i, ln.indent)
        for j in range(i + 1, d_end):
            rn = lines[j]
            if rn is None or rn.key != "run" or rn.rest:
                continue
            r_end = _block_end(lines, j, rn.indent)
            for k in range(j + 1, r_end):
                wn = lines[k]
                if wn is not None and wn.key == "working-directory" and wn.rest:
                    return _scalar(wn.rest)
    return None


def read_workflow_steps(text: str) -> list[tuple[str, str]]:
    """Return [(effective working directory, run-block text)] for every step of a
    block-style workflow document.

    Fail-closed: a `run:` whose value cannot be read yields nothing, and a step
    whose `working-directory:` cannot be read falls back to the job default, so an
    unreadable construct under-reports coverage (a red gate) rather than
    over-reporting it (a silent green)."""
    raw_lines = text.splitlines()
    # `None` marks a blank or comment-only line: it has no indent of its own and
    # must never close a block.
    lines: list[_Line | None] = []
    for raw in raw_lines:
        stripped = raw.strip()
        lines.append(None if not stripped or stripped.startswith("#") else _Line(raw))

    # Workflow-level defaults, then per-job regions under the top-level `jobs:`.
    jobs_idx = next(
        (
            i
            for i, ln in enumerate(lines)
            if ln is not None and ln.indent == 0 and ln.key == "jobs" and not ln.rest
        ),
        None,
    )
    if jobs_idx is None:
        return []
    jobs_end = _block_end(lines, jobs_idx, 0)
    workflow_default = _defaults_working_directory(lines, 0, jobs_idx)

    job_indent = next(
        (
            lines[i].indent  # type: ignore[union-attr]
            for i in range(jobs_idx + 1, jobs_end)
            if lines[i] is not None
        ),
        None,
    )
    if job_indent is None:
        return []

    job_starts = [
        i
        for i in range(jobs_idx + 1, jobs_end)
        if lines[i] is not None and lines[i].indent == job_indent  # type: ignore[union-attr]
    ]

    out: list[tuple[str, str]] = []
    for n, start in enumerate(job_starts):
        end = job_starts[n + 1] if n + 1 < len(job_starts) else jobs_end
        job_default = _defaults_working_directory(lines, start, end) or workflow_default

        for i in range(start, end):
            ln = lines[i]
            if ln is None or ln.key != "run":
                continue
            # A `defaults:`-nested `run:` is a MAPPING, not a shell block; it has
            # no scalar value, and a mapping child is not a command.
            if ln.rest and _BLOCK_SCALAR_RE.match(ln.rest):
                body_end = _block_end(lines, i, ln.indent)
                run_text = "\n".join(
                    raw_lines[j] for j in range(i + 1, body_end) if lines[j] is not None
                )
            elif ln.rest:
                run_text = _scalar(ln.rest)
            else:
                continue

            out.append((_step_working_directory(lines, i, ln.indent) or job_default or ".", run_text))
    return out


def _step_working_directory(
    lines: list[_Line | None], run_idx: int, indent: int
) -> str | None:
    """The `working-directory:` sibling of the `run:` at `run_idx`.

    Siblings share the run key's indent and lie between this step's sequence-item
    line and the next one, so the search stops at an item boundary or a dedent and
    can never borrow another step's working directory."""

    def sibling(j: int) -> str | None:
        ln = lines[j]
        if ln is None or ln.raw_indent > indent:
            return None  # nested content (including a block scalar's body)
        if ln.indent < indent:
            raise StopIteration
        if ln.key == "working-directory" and ln.rest:
            return _scalar(ln.rest)
        return None

    found: str | None = None
    # Backwards to this step's `- ` line (inclusive), then forwards to the next.
    # When `run:` IS the `- ` line the step starts here, so there is nothing
    # behind it — scanning back would reach the PREVIOUS step's keys and credit
    # its working directory to this one.
    run_line = lines[run_idx]
    if run_line is None or not run_line.is_item:
        for j in range(run_idx - 1, -1, -1):
            try:
                hit = sibling(j)
            except StopIteration:
                break
            if hit is not None:
                found = hit
            ln = lines[j]
            if ln is not None and ln.raw_indent <= indent and ln.is_item:
                break
    if found is not None:
        return found
    for j in range(run_idx + 1, len(lines)):
        ln = lines[j]
        if ln is not None and ln.raw_indent <= indent and ln.is_item:
            break
        try:
            hit = sibling(j)
        except StopIteration:
            break
        if hit is not None:
            return hit
    return None


def workflow_test_targets(text: str) -> set[str]:
    """Every npm-`test` target invoked by any step of one workflow document."""
    targets: set[str] = set()
    for cwd, run in read_workflow_steps(text):
        targets.update(run_block_targets(run, _norm_dir(".", cwd)))
    return targets


def load_workflow_targets(workflows_dir: Path) -> set[str]:
    targets: set[str] = set()
    paths = sorted(workflows_dir.glob("*.yml")) + sorted(workflows_dir.glob("*.yaml"))
    for path in paths:
        targets.update(workflow_test_targets(path.read_text(encoding="utf-8")))
    return targets


# --------------------------------------------------------------------------- #
# Verdict
# --------------------------------------------------------------------------- #


def evaluate(
    packages: dict[str, dict],
    globs: list[str],
    covered: set[str],
    *,
    allowlist: dict[str, str] | None = None,
) -> list[tuple[str, list[str]]]:
    """Return [(package_dir, [reasons])] for every package violating the
    invariant. An empty list means the guard PASSES.

    An allowlist entry exempts a package only when it carries a written reason —
    a non-empty, non-whitespace string. load_allowlist rejects anything else
    outright; honouring the reason here too means no caller can grant an
    exemption by key membership alone."""
    allowlist = allowlist or {}
    violations: list[tuple[str, list[str]]] = []

    for pkg_dir, manifest in sorted(packages.items()):
        exemption = allowlist.get(posixpath.basename(pkg_dir))
        if isinstance(exemption, str) and exemption.strip():
            continue
        reasons: list[str] = []

        if not is_workspace_member(pkg_dir, globs):
            reasons.append(
                f"{pkg_dir} is not matched by any root package.json `workspaces` "
                "glob, so the repo-root `npm ci` never installs it and its tests "
                "cannot resolve their dependencies"
            )

        if package_has_test_script(manifest):
            pkg_name = manifest.get("name")
            covered_here = (
                ALL_WORKSPACES in covered
                or pkg_dir in covered
                or (isinstance(pkg_name, str) and pkg_name in covered)
            )
            if not covered_here:
                reasons.append(
                    f"{pkg_dir} declares a `test` script that NO .github/workflows "
                    "step invokes — its tests would never execute. Add a step with "
                    f"`working-directory: {pkg_dir}` running `npm test` (as js.yml "
                    "does for the other packages), or run it from the repo root "
                    f"with `npm test -w {pkg_dir}`"
                )

        if reasons:
            violations.append((pkg_dir, reasons))

    return violations


def allowlist_entry_errors(data: dict) -> list[str]:
    """Every reason an allowlist mapping is malformed, or [] when it is well formed.

    An exemption is only reviewable if it carries a written reason, so the reason
    is validated rather than coerced: `str(None)`/`str(False)` would turn a
    vacuous entry into the string "None"/"False" and silently disable both
    invariant clauses for that package."""
    errors: list[str] = []
    for key, value in data.items():
        if (
            not isinstance(key, str)
            or not key
            or key != key.strip()
            or key in (".", "..")
            or posixpath.basename(key) != key
        ):
            errors.append(
                f"key {key!r} is not a bare package directory name (e.g. \"solid-server\")"
            )
        if not isinstance(value, str) or not value.strip():
            errors.append(
                f"entry {key!r} has no written reason ({value!r}); an exemption must "
                "record WHY the package must not be CI-tested"
            )
    return errors


def load_allowlist(path: Path) -> dict[str, str]:
    """Read the optional allowlist. An absent file means an empty allowlist."""
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except OSError:
        return {}
    except ValueError as e:
        sys.stderr.write(f"error: {path} is not valid JSON: {e}\n")
        sys.exit(2)
    if not isinstance(data, dict):
        sys.stderr.write(f"error: {path} must be an object of package -> reason\n")
        sys.exit(2)
    errors = allowlist_entry_errors(data)
    if errors:
        for e in errors:
            sys.stderr.write(f"error: {path}: {e}\n")
        sys.exit(2)
    return dict(data)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="Every packages/* npm package's tests must be run by a CI leg (#5843)."
    )
    ap.add_argument(
        "--advisory",
        action="store_true",
        help="report violations but always exit 0.",
    )
    args = ap.parse_args(argv)

    try:
        root_manifest = json.loads(
            (REPO_ROOT / "package.json").read_text(encoding="utf-8")
        )
    except (OSError, ValueError) as e:
        sys.stderr.write(f"error: could not read the root package.json: {e}\n")
        return 2

    packages = discover_packages(PACKAGES_DIR)
    globs = workspace_globs(root_manifest)
    covered = load_workflow_targets(WORKFLOWS_DIR)
    violations = evaluate(packages, globs, covered, allowlist=load_allowlist(ALLOWLIST))

    if not violations:
        print(
            f"package-test-execution: PASS — {len(packages)} package(s) under "
            "packages/ are workspace members and every `test` script has a CI leg."
        )
        return 0

    print("package-test-execution: FAIL")
    for pkg_dir, reasons in violations:
        print(f"\n  {pkg_dir}:")
        for r in reasons:
            print(f"    - {r}")
    print(
        "\nA package whose tests no workflow runs is a false green: the tests are "
        "committed and reviewed but never execute. Wire a CI leg, or record a "
        f"written reason in {ALLOWLIST.relative_to(REPO_ROOT)}."
    )

    if args.advisory:
        print("\n(advisory: not failing the build)")
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
