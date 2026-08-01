#!/usr/bin/env python3
# [OPUS-5] #5214 — dependency-closure overlap between ci.yml's rust-cache jobs.
#
# WHY THIS EXISTS
# ---------------
# `Swatinem/rust-cache` derives its cache key from the JOB NAME unless a
# `shared-key` is given. ci.yml used to leave SIXTEEN rust-cache steps unkeyed —
# the fourteen #5214 enumerates plus the `coverage-nightly` and
# `mutants-nightly-advisory` lanes — so each of them held its OWN dependency
# entry against the repository's shared 10 GB Actions cache budget, and budget
# pressure is the LRU-eviction mechanism sq-3sbrr (#1395) identified as the
# thing that makes a "warm" cache restore cold.
#
# sq-h2gz6 stopped the branch-scoped WRITES (`save-if: main`) but deliberately
# did NOT re-key those jobs, because collapsing them onto one entry is only
# sound if their dependency closures actually coincide. This script is that
# measurement, made repeatable and offline so the grouping below can be
# re-checked whenever the dependency graph moves.
#
# WHAT IT MEASURES
# ----------------
# For each job, the set of EXTERNAL (non-workspace) packages reachable from the
# workspace crates that job builds, and the pairwise Jaccard index of those
# sets. rust-cache never stores workspace-member artifacts — it deletes them
# from `target/` before saving — so the external closure is exactly the content
# a shared entry would have to serve.
#
# Roots are resolved from `crates/*/Cargo.toml` + the committed `Cargo.lock`;
# the script is stdlib-only, needs no network, and deliberately does not invoke
# cargo — a full `cargo metadata` resolve needs the registry index and the git
# dependencies, which a gating CI step should not have to fetch. The in-tree
# manifest walk was cross-checked once, during development, against a
# `cargo metadata --no-deps` reading of the same manifests: identical closure
# sizes for all twelve jobs measured below.
#
# WHAT IT DOES *NOT* MEASURE — read before trusting a number
# ----------------------------------------------------------
#  1. PACKAGE SETS, NOT BYTES. Two jobs with identical package sets can still
#     write different artifact volumes (different profiles, different feature
#     selections, different codegen flags). This says "the same crates", not
#     "the same cache size". The byte-level question needs `gh cache list`
#     against main, which is not available offline.
#  2. UNION-OF-FEATURES BEYOND THE WORKSPACE BOUNDARY. Kinds (normal/build/dev)
#     are honoured for workspace crates, where the manifests are in-tree. Once
#     the walk leaves the workspace it follows `Cargo.lock`, whose per-package
#     dependency list is the workspace-wide resolve — so a feature-gated
#     dependency of an external crate is counted even for a job that would not
#     compile it. That inflates BOTH sides of every comparison, which is why
#     the floor below is 0.90 and not 1.0.
#  3. THE RUST-CACHE KEY DIGEST. rust-cache mixes the rustc version, the
#     `CARGO*`/`RUST*`/`CC*`/`CXX*`/`CMAKE*` environment and the lockfile hashes
#     into the key. It does NOT appear to mix in a `--target` triple passed on
#     the cargo COMMAND LINE (as opposed to one set through the environment) —
#     that reading was NOT verified against the pinned action SHA here, because
#     it cannot be without network access. Nothing below depends on which way it
#     goes: the conservative reading says a wasm32 job sharing a key with a
#     native one would collide two artifact spaces on one entry, and the
#     permissive reading says it simply would not share the entry at all. Either
#     way the right move is to leave `wasm` out of the group, which is what
#     NON_CLOSURE_EXCLUSIONS does. Those jobs are deliberately NOT measured here
#     — the argument for keeping them apart is structural, not statistical.
#
# THE TRADE A SHARED KEY MAKES, STATED PLAINLY
# --------------------------------------------
# rust-cache skips saving when the exact key already exists, so on main exactly
# one member of a group wins the save race and its closure becomes the entry.
# Every other member restores the group's COMMON CORE and compiles its own
# extra packages cold. That is a smaller compile win than N warm caches would
# give, traded deliberately for the budget headroom — and it is still strictly
# better than the cold rebuild an evicted key forces. It is the same trade
# already documented on ci.yml's `coverage-measure` step.
#
# Run:  python3 scripts/ci-cache-closure-overlap.py           # print the table
#       python3 scripts/ci-cache-closure-overlap.py --check   # gate the grouping

from __future__ import annotations

import argparse
import sys
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# The `shared-key` the grouped jobs carry in ci.yml. Pinned on both sides by
# scripts/tests/test_cache_shared_key_grouping.py.
GROUP_SHARED_KEY = "conformance-suites"

# Minimum pairwise Jaccard for two jobs to be allowed to share one entry.
# Chosen with headroom over the measured minimum (printed by this script) so a
# routine dependency addition does not red CI, while a job that grows a genuinely
# separate closure — tens of unique packages — trips it and forces a re-key.
OVERLAP_FLOOR = 0.90

# ci.yml jobs that share `GROUP_SHARED_KEY`, mapped to the workspace crates each
# one builds. The conformance/oracle jobs differ by which fixture suite they
# fetch and which crate's tests they run; seven of the ten additionally run the
# same `sparq-conformance-scoreboard` binary, and every one of them builds
# `sparq-conformance` itself, which is where the coincidence comes from.
GROUPED: dict[str, list[str]] = {
    "conformance": ["sparq-conformance"],
    "shacl-conformance": ["sparq-shacl", "sparq-conformance"],
    "geo-conformance": ["sparq-geo", "sparq-conformance"],
    "solid-conformance": ["sparq-solid", "sparq-conformance"],
    "odrl-conformance": ["sparq-policy", "sparq-conformance"],
    "text-oracle": ["sparq-text", "sparq-conformance"],
    "rsp-oracle": ["sparq-rsp", "sparq-conformance"],
    "jsonld-conformance": ["sparq-conformance"],
    "service-federation-conformance": ["sparq-conformance"],
    "inference-conformance": ["sparq-conformance"],
}

# Jobs kept OUT of the group on the strength of this measurement. `geiger`
# reports unsafe usage in sparq-core alone; `lint` clippies and rustdocs the
# whole workspace with --all-features and is a strict superset of everything.
# Both are asserted to sit BELOW the floor, so widening the group to either one
# without re-measuring goes red.
CLOSURE_EXCLUDED: dict[str, list[str]] = {
    "geiger": ["sparq-core"],
    "lint": [],  # empty == every workspace member
}

# Jobs kept out for reasons this script cannot measure. Recorded here so the
# next reader does not have to re-derive them, and so "why is that one still
# unkeyed?" has an answer in the tree.
NON_CLOSURE_EXCLUSIONS: dict[str, str] = {
    "wasm": "builds for wasm32-unknown-unknown, a different artifact space from "
    "every other job here; sharing one entry with a native job either collides "
    "the two or is a guaranteed miss, and neither is worth a budget slot (see "
    "limitation 3 above)",
    "msrv": "pins the MSRV toolchain and overrides CARGO_TARGET_*_RUSTFLAGS; "
    "rustc version and CARGO* env are both in the key digest, so it can never "
    "share an entry with a pinned-stable job anyway",
    "coverage-nightly": "sets RUSTFLAGS for lld and builds llvm-cov-instrumented; "
    "RUSTFLAGS is in the key digest",
    "mutants-nightly-advisory": "cargo-mutants rebuilds per mutant with its own "
    "flags; not a plain dependency closure",
    "build-archive": "already carries the `test-workspace` shared-key",
    "coverage-measure": "already carries the `coverage-measure` shared-key",
    "coverage-engine-run": "already carries a pinned per-shard `key`",
    "coverage-engine-merge": "already carries a pinned `key`",
}

_KIND_SECTION = {
    "normal": "dependencies",
    "build": "build-dependencies",
    "dev": "dev-dependencies",
}


def load_workspace() -> dict[str, dict]:
    """Every workspace member's parsed manifest, keyed by package name."""
    root = tomllib.loads((REPO_ROOT / "Cargo.toml").read_text())
    members = {}
    for rel in root["workspace"]["members"]:
        manifest = tomllib.loads((REPO_ROOT / rel / "Cargo.toml").read_text())
        members[manifest["package"]["name"]] = manifest
    return members


def load_lock() -> tuple[dict, dict]:
    """(name -> [locked packages], (name, version) -> locked package)."""
    lock = tomllib.loads((REPO_ROOT / "Cargo.lock").read_text())
    by_name: dict[str, list[dict]] = {}
    for pkg in lock["package"]:
        by_name.setdefault(pkg["name"], []).append(pkg)
    by_id = {(p["name"], p["version"]): p for p in lock["package"]}
    return by_name, by_id


def _declared_deps(manifest: dict, kinds: set[str]) -> set[str]:
    """Dependency package names declared by a workspace manifest.

    Renames are resolved through `package = "..."` so the name matches the lock,
    and `[target.'cfg(...)'.dependencies]` tables are folded in — a job that
    builds on Linux still pays for whatever the Linux cfg selects.
    """
    names: set[str] = set()
    tables = [manifest] + list((manifest.get("target") or {}).values())
    for kind in kinds:
        section = _KIND_SECTION[kind]
        for table in tables:
            for name, spec in (table.get(section) or {}).items():
                if isinstance(spec, dict) and "package" in spec:
                    name = spec["package"]
                names.add(name)
    return names


def external_closure(
    roots: list[str],
    kinds: set[str],
    workspace: dict[str, dict],
    by_name: dict[str, list[dict]],
    by_id: dict[tuple[str, str], dict],
) -> set[tuple[str, str]]:
    """External packages reachable from `roots`, as (name, version) pairs.

    Workspace crates are traversed but never returned: rust-cache strips
    workspace-member artifacts from `target/` before saving, so they are not
    part of what an entry serves. `kinds` applies to the ROOTS only — a
    transitive workspace dependency contributes its normal+build deps, not the
    dev-dependencies of its own test suite, which no root here compiles.
    """

    def resolve(spec: str) -> list[tuple[str, str]]:
        parts = spec.split()
        name, version = parts[0], (parts[1] if len(parts) > 1 else None)
        found = by_name.get(name, [])
        if version is not None:
            found = [p for p in found if p["version"] == version]
        return [(p["name"], p["version"]) for p in found]

    seen: set = set()
    external: set[tuple[str, str]] = set()
    stack: list = []

    def push_deps(manifest: dict, kinds: set[str]) -> None:
        for dep in _declared_deps(manifest, kinds):
            if dep in workspace:
                if ("workspace", dep) not in seen:
                    seen.add(("workspace", dep))
                    stack.append(("workspace", dep))
                continue
            for ident in resolve(dep):
                if ident not in seen:
                    seen.add(ident)
                    stack.append(ident)

    for root in roots:
        if root not in workspace:
            raise SystemExit(f"{root!r} is not a workspace member")
        push_deps(workspace[root], kinds)

    while stack:
        node = stack.pop()
        if node[0] == "workspace":
            push_deps(workspace[node[1]], {"normal", "build"})
            continue
        external.add(node)
        for spec in (by_id.get(node) or {}).get("dependencies", []):
            for ident in resolve(spec):
                if ident[0] in workspace or ident in seen:
                    continue
                seen.add(ident)
                stack.append(ident)

    return external


def jaccard(a: set, b: set) -> float:
    union = a | b
    return 1.0 if not union else len(a & b) / len(union)


def measure() -> dict[str, set[tuple[str, str]]]:
    workspace = load_workspace()
    by_name, by_id = load_lock()
    kinds = {"normal", "build", "dev"}
    closures: dict[str, set[tuple[str, str]]] = {}
    for job, roots in GROUPED.items():
        closures[job] = external_closure(roots, kinds, workspace, by_name, by_id)
    for job, roots in CLOSURE_EXCLUDED.items():
        # `geiger` only reports on sparq-core's shipped graph, so no dev-deps.
        job_kinds = {"normal", "build"} if job == "geiger" else kinds
        closures[job] = external_closure(
            roots or sorted(workspace), job_kinds, workspace, by_name, by_id
        )
    return closures


def report(closures: dict[str, set]) -> None:
    print(f"external dependency closure per ci.yml rust-cache job "
          f"(shared-key: {GROUP_SHARED_KEY})\n")
    for job in GROUPED:
        print(f"  [grouped] {job:34s} {len(closures[job]):5d} external packages")
    for job in CLOSURE_EXCLUDED:
        print(f"  [singleton] {job:32s} {len(closures[job]):5d} external packages")
    print()

    grouped = [closures[j] for j in GROUPED]
    core, union = set.intersection(*grouped), set.union(*grouped)
    worst = min(
        (jaccard(closures[a], closures[b]), a, b)
        for a in GROUPED
        for b in GROUPED
        if a < b
    )
    print(f"  group common core : {len(core)} packages "
          f"({len(core) / len(union):.1%} of the {len(union)}-package union)")
    print(f"  weakest pair      : {worst[1]} vs {worst[2]} — Jaccard {worst[0]:.3f} "
          f"(floor {OVERLAP_FLOOR})")
    for job in CLOSURE_EXCLUDED:
        print(f"  {job:17s} : Jaccard {jaccard(closures[job], union):.3f} "
              f"vs the group union — below the floor, kept separate")
    print()
    for job, why in NON_CLOSURE_EXCLUSIONS.items():
        print(f"  not measured: {job} — {why}")


def check(closures: dict[str, set]) -> int:
    failures: list[str] = []
    for a in GROUPED:
        for b in GROUPED:
            if a >= b:
                continue
            overlap = jaccard(closures[a], closures[b])
            if overlap < OVERLAP_FLOOR:
                failures.append(
                    f"{a} and {b} share `shared-key: {GROUP_SHARED_KEY}` but their "
                    f"external dependency closures have diverged (Jaccard "
                    f"{overlap:.3f} < {OVERLAP_FLOOR}). One of them now pays a mostly "
                    f"cold dependency build on every run. Split it out of GROUPED "
                    f"and give it its own shared-key in .github/workflows/ci.yml."
                )
    union = set.union(*[closures[j] for j in GROUPED])
    for job in CLOSURE_EXCLUDED:
        overlap = jaccard(closures[job], union)
        if overlap >= OVERLAP_FLOOR:
            failures.append(
                f"{job} is declared a singleton on the strength of a LOW closure "
                f"overlap, but now measures Jaccard {overlap:.3f} >= {OVERLAP_FLOOR} "
                f"against the group union. Either fold it into GROUPED (and give it "
                f"`shared-key: {GROUP_SHARED_KEY}` in ci.yml) or record the real "
                f"reason it stays apart in NON_CLOSURE_EXCLUSIONS."
            )
    for line in failures:
        print(f"FAIL: {line}", file=sys.stderr)
    return 1 if failures else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="exit non-zero if the declared grouping no longer matches the measurement",
    )
    args = parser.parse_args()
    closures = measure()
    if args.check:
        rc = check(closures)
        if rc == 0:
            print("ci-cache-closure-overlap: grouping matches the measurement")
        return rc
    report(closures)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
