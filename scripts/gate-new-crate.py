#!/usr/bin/env python3
# [OPUS-4.8] Gate G1 — new-crate-completeness (bead sq-ncvq.4, epic sq-ncvq).
# Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns).
#
# This is the PROACTIVE / merge-time half of the maintenance flow-on system
# (research/maintenance-flow-on-automation-design.md §2.1, gate G1). It is the
# COMPLEMENT of the reactive engine in scripts/flow-on.py (PR #220): that engine
# mints follow-on ISSUES after a PR merges; THIS script BLOCKS the PR before it
# merges when a new crate would land without its required maintenance artifacts.
#
# [OPUS-4.8] sq-ncvq.10 doc-sync: this gate is the "Enforced by: **G1**" cell of
# the "new crate" row in the AGENTS.md "Post-batch re-evaluation checklist" table.
# That table row and this docstring are the two halves of the same rule — change
# one and update the other; the divergence is what sq-ncvq.10 exists to prevent.
#
# RULE (G1): when a PR ADDS a new `crates/<x>/Cargo.toml`, fail unless the SAME
# PR also provides, for that crate:
#   (a) a registered benchmark — a `[[benchmark]]` entry in bench/benchmarks.toml
#       whose `source` references `crates/<x>` (matching the reactive rule's
#       `source = crates/<x>` expectation in flow-on-rules.toml); OR the crate is
#       an intentional stub marked `publish = false` in its Cargo.toml (the
#       design's documented stub-exemption escape hatch);
#   (b) a README.md (crates/<x>/README.md) — the template content itself is
#       separately enforced by check-readme-template.py; G1 only requires its
#       PRESENCE so a new crate is never undocumented;
#   (c) a SKILL.md — ONLY if the crate is a PUBLIC surface. A crate is treated as
#       public iff its Cargo.toml does NOT carry `publish = false`. A public crate
#       must have at least one skills/<surface>/SKILL.md touched in the same PR
#       (the AGENTS.md public-API → SKILL.md rule applied to a brand-new surface).
#       A `publish = false` stub is internal-only and exempt from (c) — and, via
#       the same marker, exempt from (a) as well.
#
# ESCAPE HATCH (design §2.1): `publish = false` in the new crate's Cargo.toml
# marks it an intentional stub — exempt from the bench (a) and SKILL (c)
# requirements. The README (b) is still required (a stub still needs a one-line
# "what/why" README).
#
# [OPUS-5] RULE (G1-pkg-ci, #5843): when a PR ADDS a new `packages/<x>/package.json`,
# fail unless that package's tests are actually REACHABLE and actually RUN:
#   (d) `packages/<x>` resolves to an npm **workspace entry** — it matches one of
#       the `workspaces` globs in the repo-root package.json. A package outside the
#       workspace is not installed by the root `npm ci` every JS lane runs, so no
#       leg could invoke it even if one tried;
#   (e) some `.github/workflows/*.yml` leg INVOKES the package's `test` script —
#       a step whose effective working-directory is `packages/<x>` (its own
#       `working-directory:`, or the job's / workflow's `defaults.run` one) and
#       whose `run:` calls `npm test` / `npm run test`, or any step that calls
#       `npm test` with a workspace selector naming this package (`-w <name>`,
#       `--workspace=packages/<x>`, `--workspaces`). This fires when the package
#       DECLARES a `test` script; a package that ships test FILES but declares no
#       `test` script fails the same clause (there is nothing for a leg to call),
#       which is what stops the check being evaded by deleting the script.
# This is the follow-up #5843 named: G1 already asks a new package for its
# artifacts, but artifacts that no lane executes are indistinguishable from
# absent ones. The scan is a purpose-built structural walk of the workflow files
# (job blocks → step chunks → effective working-directory), NOT a YAML parse:
# this script is stdlib-only and the `flow-on-gates` lane that runs it installs
# no PyYAML. LIMITATION: it therefore assumes the repo's ordinary block-mapping
# workflow layout — a leg that invokes the tests through an indirection this walk
# cannot see (a shell script, a reusable workflow, a matrix-computed directory)
# reads as "no leg", so wire such a package with a directly visible step.
#
# DIFF SOURCE: the PR's changed-file list. In CI this is
#   git diff --name-only origin/<base>...HEAD          (changed files)
#   git diff --name-status origin/<base>...HEAD        (to find ADDED files)
# In tests / --dry-run it is read from a fixture file (one path per line, each
# optionally prefixed with a git status letter + tab, e.g. "A\tcrates/x/...").
#
# EXIT: 0 when every newly-added crate and npm package satisfies G1 (or there are
# none); 1 with a clear per-surface message naming exactly what is missing.
#
# Usage:
#   gate-new-crate.py                       # CI: derive diff from git vs origin/<base>
#   gate-new-crate.py --base main           # override base ref
#   gate-new-crate.py --advisory            # warn-only soft-launch (never exit 1)
#   gate-new-crate.py --dry-run --changed-files files.txt   # hermetic (tests)
#
# stdlib-only.

from __future__ import annotations

import argparse
import fnmatch
import json
import os
import re
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
BENCH_REGISTRY = REPO_ROOT / "bench" / "benchmarks.toml"
ROOT_PACKAGE_JSON = REPO_ROOT / "package.json"
WORKFLOWS_DIR = REPO_ROOT / ".github" / "workflows"

# A line in a status-prefixed diff: "A\tpath", "M\tpath", "R100\told\tnew", etc.
_STATUS_RE = re.compile(r"^([A-Z])\d*\t(.*)$")


def parse_status_lines(lines: list[str]) -> tuple[list[str], list[str]]:
    """Split a `git diff --name-status`-style list into (changed, added).

    Each line is either "<STATUS>\t<path>" (optionally "<STATUS>\t<old>\t<new>"
    for renames/copies) or a bare path (treated as a generic change, NOT added).
    Returns (all_changed_paths, added_paths)."""
    changed: list[str] = []
    added: list[str] = []
    for raw in lines:
        line = raw.rstrip("\n")
        if not line.strip():
            continue
        m = _STATUS_RE.match(line)
        if m:
            status, rest = m.group(1), m.group(2)
            # For renames/copies the destination path is the last tab field.
            path = rest.split("\t")[-1]
            changed.append(path)
            # [OPUS-4.8] `A` (added) and `C` (copied) both materialise a NEW
            # destination path. Git only emits `C` with copy-detection enabled
            # (-C/--find-copies), but if a crate directory is introduced via a
            # copy the gate must still treat it as added, or new-crate detection
            # could be evaded (accidentally or deliberately).
            if status in ("A", "C"):
                added.append(path)
        else:
            # Bare path (e.g. `git diff --name-only`): a change, not provably added.
            changed.append(line)
    return changed, added


def _normalize_base(base: str) -> str:
    """Accept a bare branch ('main') or a full ref ('refs/heads/main', as the
    merge_group event supplies in github.event.merge_group.base_ref) and return
    the remote-tracking ref to diff against ('origin/main')."""
    base = base.strip()
    if base.startswith("refs/heads/"):
        base = base[len("refs/heads/") :]
    # Already an origin/* ref — use as-is.
    if base.startswith("origin/") or base == "HEAD":
        return base
    return f"origin/{base}"


def git_diff(base: str) -> tuple[list[str], list[str]]:
    """Return (changed, added) from git vs the base ref using a 3-dot diff."""
    ref = _normalize_base(base)
    try:
        out = subprocess.run(
            ["git", "diff", "--name-status", f"{ref}...HEAD"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    except subprocess.CalledProcessError as e:  # pragma: no cover - CI-only path
        sys.stderr.write(f"error: git diff failed: {e.stderr}\n")
        sys.exit(2)
    return parse_status_lines(out.splitlines())


def added_crates(added: list[str]) -> list[str]:
    """Crate names whose Cargo.toml was ADDED (a brand-new crate)."""
    crates: list[str] = []
    for p in added:
        m = re.match(r"^crates/([^/]+)/Cargo\.toml$", p)
        if m:
            crates.append(m.group(1))
    # Stable, de-duplicated order.
    seen: set[str] = set()
    out: list[str] = []
    for c in crates:
        if c not in seen:
            seen.add(c)
            out.append(c)
    return out


def crate_is_stub(crate: str, changed: list[str]) -> bool:
    """True iff the new crate's Cargo.toml declares `publish = false` (an
    intentional stub, exempt from bench + SKILL requirements).

    Reads the on-disk Cargo.toml in the worktree (the PR's checkout already has
    the added file). In hermetic tests the file may not exist on disk; callers
    pass the stub status explicitly via the fixture, so a missing file is simply
    treated as NOT a stub (the strict default)."""
    cargo = REPO_ROOT / "crates" / crate / "Cargo.toml"
    try:
        text = cargo.read_text(encoding="utf-8")
    except OSError:
        return False
    return re.search(r"^\s*publish\s*=\s*false\b", text, re.MULTILINE) is not None


def crate_has_registered_bench(crate: str) -> bool:
    """True iff bench/benchmarks.toml has a `source` field referencing the crate.

    Mirrors the reactive rule's expectation (`source = crates/<x>`): we match any
    benchmark whose `source` line contains `crates/<crate>/` or `crates/<crate>"`
    (end of the path component), so a registered bench for the crate counts."""
    try:
        text = BENCH_REGISTRY.read_text(encoding="utf-8")
    except OSError:
        return False
    needle = re.compile(
        rf"^\s*source\s*=.*crates/{re.escape(crate)}(?:/|\b)", re.MULTILINE
    )
    return needle.search(text) is not None


# --------------------------------------------------------------------------- #
# [OPUS-5] (#5843) G1-pkg-ci — a new npm package's tests must actually RUN.
# --------------------------------------------------------------------------- #

# A test file inside a package: a path segment named test/tests/__tests__, or a
# `*.test.*` / `*.spec.*` basename (the two conventions in use under packages/).
_TEST_DIR_RE = re.compile(r"(?:^|/)(?:tests?|__tests__)/")
_TEST_FILE_RE = re.compile(r"(?:^|/)[^/]+\.(?:test|spec)\.[^/]+$")

# `npm test` / `npm run test`, with any flags in between (`npm -w x test`). The
# trailing lookahead keeps a SIBLING script whose name merely starts with "test"
# out (`npm run test:unit`, `npm run test-e2e` are NOT the `test` script).
_NPM_TEST_RE = re.compile(r"\bnpm\s+(?:\S+\s+)*?(?:run\s+)?test(?![-:\w])")

# `working-directory: <dir>` on its own line, anywhere in a step/job chunk.
_WD_RE = re.compile(
    r"^[ \t]*working-directory:\s*[\"']?([^\"'#\s]+)", re.MULTILINE
)
# A bare mapping key (`quick-gates:`) — used to find job keys and step bullets.
_BARE_KEY_RE = re.compile(r"^(\s+)[A-Za-z_][\w.-]*:\s*$")
_BULLET_RE = re.compile(r"^(\s*)-\s")


def added_packages(added: list[str]) -> list[str]:
    """Package dir names whose package.json was ADDED (a brand-new npm package).

    One level deep only (`packages/<x>/package.json`), so a vendored or nested
    manifest inside a package is not mistaken for a new package."""
    packages: list[str] = []
    for p in added:
        m = re.match(r"^packages/([^/]+)/package\.json$", p)
        if m:
            packages.append(m.group(1))
    seen: set[str] = set()
    out: list[str] = []
    for pkg in packages:
        if pkg not in seen:
            seen.add(pkg)
            out.append(pkg)
    return out


def read_package_manifest(package: str) -> dict:
    """Parse `packages/<package>/package.json`; {} if missing/unparseable.

    An unreadable manifest yields {} — which declares no `test` script and no
    name, so clause (e) reports the package as shipping nothing a leg could call
    rather than silently passing. A package cannot dodge the gate by being
    unreadable."""
    manifest = REPO_ROOT / "packages" / package / "package.json"
    try:
        data = json.loads(manifest.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}
    return data if isinstance(data, dict) else {}


def workspace_globs() -> list[str]:
    """The repo-root package.json `workspaces` globs (npm array form, or the
    `{"packages": [...]}` object form). [] when the root manifest is unreadable —
    fail-closed: nothing resolves, so clause (d) reports the package."""
    try:
        data = json.loads(ROOT_PACKAGE_JSON.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return []
    ws = data.get("workspaces") if isinstance(data, dict) else None
    if isinstance(ws, dict):
        ws = ws.get("packages")
    if not isinstance(ws, list):
        return []
    return [w for w in ws if isinstance(w, str)]


def package_in_workspaces(package: str, globs: list[str]) -> bool:
    """True iff `packages/<package>` matches one of the root workspace globs."""
    target = f"packages/{package}"
    return any(fnmatch.fnmatch(target, g.rstrip("/")) for g in globs)


def package_has_test_files(package: str, changed: list[str]) -> bool:
    """True iff the PR touches at least one test file inside the package."""
    prefix = f"packages/{package}/"
    for p in changed:
        if not p.startswith(prefix):
            continue
        rest = "/" + p[len(prefix) :]
        if _TEST_DIR_RE.search(rest) or _TEST_FILE_RE.search(rest):
            return True
    return False


def _strip_comment_lines(text: str) -> str:
    """Blank out whole-line comments, keeping line numbering intact.

    Essential: the workflow files DISCUSS commands in prose comments (js.yml's
    "The wasm build MUST precede `npm test`"), and a comment is not an executor."""
    return "\n".join(
        "" if ln.lstrip().startswith("#") else ln for ln in text.splitlines()
    )


def _sibling_blocks(lines: list[str], key_indents: list[tuple[int, int]]) -> list[str]:
    """Split `lines` at every entry of `key_indents` sitting at the SHALLOWEST
    indent, returning one joined block per sibling."""
    if not key_indents:
        return []
    top = min(indent for _, indent in key_indents)
    starts = [i for i, indent in key_indents if indent == top]
    blocks: list[str] = []
    for n, i in enumerate(starts):
        end = starts[n + 1] if n + 1 < len(starts) else len(lines)
        blocks.append("\n".join(lines[i:end]))
    return blocks


def _job_blocks(text: str) -> list[str]:
    """The text of each job under the workflow's top-level `jobs:` mapping."""
    lines = text.splitlines()
    start = next(
        (i for i, ln in enumerate(lines) if re.match(r"^jobs:\s*$", ln)), None
    )
    if start is None:
        return []
    end = len(lines)
    for i in range(start + 1, len(lines)):
        # A non-blank line at column 0 ends the `jobs:` mapping.
        if lines[i].strip() and not lines[i][:1].isspace():
            end = i
            break
    body = lines[start + 1 : end]
    keys = [
        (i, len(m.group(1)))
        for i, ln in enumerate(body)
        if (m := _BARE_KEY_RE.match(ln))
    ]
    return _sibling_blocks(body, keys)


def _default_working_dir(text: str) -> str | None:
    """The workflow-level `defaults.run.working-directory`, if any."""
    lines = text.splitlines()
    for i, ln in enumerate(lines):
        if re.match(r"^defaults:\s*$", ln):
            for nxt in lines[i + 1 :]:
                if nxt.strip() and not nxt[:1].isspace():
                    return None
                m = _WD_RE.match(nxt)
                if m:
                    return m.group(1)
            return None
    return None


def _job_steps(block: str, default_wd: str | None) -> list[tuple[str | None, str]]:
    """(effective working-directory, step text) for each step of one job.

    The effective directory is the step's own `working-directory:` if it has one,
    else the job's `defaults.run.working-directory` (everything before `steps:`),
    else the workflow-level default."""
    lines = block.splitlines()
    steps_at = next(
        (i for i, ln in enumerate(lines) if re.match(r"^\s*steps:\s*$", ln)), None
    )
    if steps_at is None:
        return []
    job_wd = default_wd
    for ln in lines[:steps_at]:
        m = _WD_RE.match(ln)
        if m:
            job_wd = m.group(1)
    body = lines[steps_at + 1 :]
    bullets = [
        (i, len(m.group(1)))
        for i, ln in enumerate(body)
        if (m := _BULLET_RE.match(ln))
    ]
    steps: list[tuple[str | None, str]] = []
    for chunk in _sibling_blocks(body, bullets):
        m = _WD_RE.search(chunk)
        steps.append((m.group(1) if m else job_wd, chunk))
    return steps


def _selects_workspace(command: str, package: str, package_name: str | None) -> bool:
    """True iff an npm invocation targets this package via a workspace selector."""
    if re.search(r"--workspaces\b", command):
        return True
    wanted = {f"packages/{package}"}
    if package_name:
        wanted.add(package_name)
    for m in re.finditer(r"(?:--workspace|-w)[=\s]+([^\s\"']+)", command):
        if m.group(1).rstrip("/") in wanted:
            return True
    return False


def workflow_running_package_tests(
    package: str,
    package_name: str | None,
    workflows_dir: Path | None = None,
) -> str | None:
    """Name of the first workflow file with a leg that invokes the package's
    `test` script, or None if no leg does."""
    wf_dir = WORKFLOWS_DIR if workflows_dir is None else workflows_dir
    try:
        paths = sorted(
            p for p in wf_dir.iterdir() if p.suffix in (".yml", ".yaml")
        )
    except OSError:
        return None
    target = f"packages/{package}"
    for path in paths:
        try:
            text = _strip_comment_lines(path.read_text(encoding="utf-8"))
        except OSError:
            continue
        default_wd = _default_working_dir(text)
        for block in _job_blocks(text):
            for wd, chunk in _job_steps(block, default_wd):
                if not _NPM_TEST_RE.search(chunk):
                    continue
                if (wd or "").rstrip("/") == target or _selects_workspace(
                    chunk, package, package_name
                ):
                    return path.name
    return None


def evaluate_package_ci(
    changed: list[str],
    added: list[str],
    *,
    manifest_overrides: dict[str, dict] | None = None,
    globs_override: list[str] | None = None,
    workflows_dir: Path | None = None,
) -> list[tuple[str, list[str]]]:
    """Return [(package, [missing-reasons])] for every newly-added npm package
    that violates G1-pkg-ci. An empty list means the gate PASSES.

    manifest_overrides / globs_override / workflows_dir let hermetic tests inject
    the manifest, the root workspace globs and the workflow corpus without
    touching the real tree."""
    manifest_overrides = manifest_overrides or {}
    globs = workspace_globs() if globs_override is None else globs_override
    violations: list[tuple[str, list[str]]] = []

    for pkg in added_packages(added):
        manifest = manifest_overrides.get(pkg)
        if manifest is None:
            manifest = read_package_manifest(pkg)
        scripts = manifest.get("scripts")
        test_script = scripts.get("test") if isinstance(scripts, dict) else None
        name = manifest.get("name") if isinstance(manifest.get("name"), str) else None

        missing: list[str] = []

        # (d) the package must be an npm workspace entry.
        if not package_in_workspaces(pkg, globs):
            missing.append(
                f"an npm workspace entry covering packages/{pkg} "
                "(a `workspaces` glob in the repo-root package.json) — without one "
                "the root `npm ci` never installs it, so no CI leg can run its tests"
            )

        # (e) some CI leg must invoke the package's `test` script.
        if test_script:
            hit = workflow_running_package_tests(pkg, name, workflows_dir)
            if hit is None:
                missing.append(
                    f"a CI leg that runs packages/{pkg}'s `test` script — add a step "
                    f"with `working-directory: packages/{pkg}` running `npm test` "
                    "(or an `npm test -w <name>` step) to a .github/workflows/ job, "
                    "so the tests are not shipped dead"
                )
        elif package_has_test_files(pkg, changed):
            missing.append(
                f"a `test` script in packages/{pkg}/package.json — the package "
                "ships test files but declares nothing for a CI leg to invoke"
            )

        if missing:
            violations.append((pkg, missing))

    return violations


def evaluate(
    changed: list[str],
    added: list[str],
    *,
    stub_overrides: dict[str, bool] | None = None,
    bench_overrides: dict[str, bool] | None = None,
) -> list[tuple[str, list[str]]]:
    """Return [(crate, [missing-reasons])] for every newly-added crate that
    violates G1. An empty list means the gate PASSES.

    stub_overrides / bench_overrides let hermetic tests inject the
    publish-status and bench-registration facts without touching disk."""
    stub_overrides = stub_overrides or {}
    bench_overrides = bench_overrides or {}
    violations: list[tuple[str, list[str]]] = []

    for crate in added_crates(added):
        is_stub = stub_overrides.get(crate)
        if is_stub is None:
            is_stub = crate_is_stub(crate, changed)

        missing: list[str] = []

        # (b) README.md — always required (even for stubs).
        readme = f"crates/{crate}/README.md"
        if readme not in changed:
            missing.append(
                f"a README at {readme} (added in the same PR)"
            )

        if not is_stub:
            # (a) registered benchmark.
            has_bench = bench_overrides.get(crate)
            if has_bench is None:
                has_bench = crate_has_registered_bench(crate)
            if not has_bench:
                missing.append(
                    "a registered benchmark in bench/benchmarks.toml "
                    f"(a [[benchmark]] whose `source` references crates/{crate}) "
                    "— or mark the crate `publish = false` if it is an intentional stub"
                )
            # (c) SKILL.md — required because the crate is a public surface.
            if not any(
                p.startswith("skills/") and p.endswith("SKILL.md") for p in changed
            ):
                missing.append(
                    "a skills/<surface>/SKILL.md for the new public surface "
                    "(AGENTS.md public-API → SKILL.md rule) — or mark the crate "
                    "`publish = false` if it is internal-only"
                )

        if missing:
            violations.append((crate, missing))

    return violations


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="G1 new-crate / new-npm-package completeness gate (sq-ncvq.4, #5843)."
    )
    ap.add_argument(
        "--base",
        default=os.environ.get("GATE_BASE_REF", "main"),
        help="base ref to diff against (origin/<base>); default 'main'.",
    )
    ap.add_argument(
        "--changed-files",
        help="hermetic input: a file of diff lines (status-prefixed or bare paths).",
    )
    ap.add_argument(
        "--dry-run",
        action="store_true",
        help="never exit non-zero; print the verdict only (for local/testing).",
    )
    ap.add_argument(
        "--advisory",
        action="store_true",
        help="soft-launch: report violations but always exit 0.",
    )
    args = ap.parse_args(argv)

    if args.changed_files:
        lines = Path(args.changed_files).read_text(encoding="utf-8").splitlines()
        changed, added = parse_status_lines(lines)
    else:
        changed, added = git_diff(args.base)

    violations = evaluate(changed, added)
    package_violations = evaluate_package_ci(changed, added)

    if not violations and not package_violations:
        print(
            "G1 new-crate-completeness: PASS — no new crate or npm package "
            "missing artifacts."
        )
        return 0

    print("G1 new-crate-completeness: FAIL")
    for crate, missing in violations:
        print(f"\n  crates/{crate} is a new crate but is missing:")
        for m in missing:
            print(f"    - {m}")
    for pkg, missing in package_violations:
        print(f"\n  packages/{pkg} is a new npm package but is missing:")
        for m in missing:
            print(f"    - {m}")
    print(
        "\nA new crate must ship its maintenance artifacts in the SAME PR "
        "(research/maintenance-flow-on-automation-design.md §2.1, gate G1). "
        "Stub crates may set `publish = false` in Cargo.toml to opt out of the "
        "bench + SKILL requirements (README still required). A new npm package "
        "must be a repo-root workspace entry AND have a CI leg that invokes its "
        "`test` script (G1-pkg-ci, #5843)."
    )

    if args.advisory or args.dry_run:
        print("\n(advisory/dry-run: not failing the build)")
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
