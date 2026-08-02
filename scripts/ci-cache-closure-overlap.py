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
# CARGO COMMANDS that job actually runs, and the pairwise Jaccard index of those
# sets. rust-cache never stores workspace-member artifacts — it deletes them
# from `target/` before saving — so the external closure is exactly the content
# a shared entry would have to serve.
#
# A job is modelled as a list of `Cmd`s, one per cargo invocation in ci.yml:
# the package, the `--features` selection, whether `--all-features` /
# `--no-default-features` is passed, and whether the command builds test targets
# (`cargo test` / `cargo clippy --all-targets` pull dev-dependencies; `cargo run`
# does not). FEATURE RESOLUTION IS REAL for workspace crates: `optional = true`
# dependencies are traversed only when something actually activates them, the
# `[features]` table is expanded transitively (`dep:x`, `x/y`, weak `x?/y`,
# feature-to-feature edges, implicit optional-dependency features), and both a
# dependency's own `features = [...]` / `default-features = false` and the
# `default` feature are honoured. Without that, a crate like `sparq-conformance`
# — which every one of these jobs touches, and which keeps its whole async
# server stack, its JSON-LD parser and its three opt-in reasoner crates behind
# OFF-by-default features — would be counted as compiling all of them in every job, and
# the resulting phantom common core would make unrelated closures look identical.
#
# Roots are resolved from `crates/*/Cargo.toml` + the committed `Cargo.lock`;
# the script is stdlib-only, needs no network, and deliberately does not invoke
# cargo — a full `cargo metadata` resolve needs the registry index and the git
# dependencies, which a gating CI step should not have to fetch.
#
# WHAT IT DOES *NOT* MEASURE — read before trusting a number
# ----------------------------------------------------------
#  1. PACKAGE SETS, NOT BYTES. Two jobs with identical package sets can still
#     write different artifact volumes (different profiles, different codegen
#     flags). This says "the same crates", not "the same cache size". The
#     byte-level question needs `gh cache list` against main, which is not
#     available offline.
#  2. FEATURE RESOLUTION STOPS AT THE WORKSPACE BOUNDARY. Feature selection is
#     modelled for workspace crates, where the manifests are in-tree. Once the
#     walk leaves the workspace it follows `Cargo.lock`, whose per-package
#     dependency list is the workspace-wide resolve — so a feature-gated
#     dependency OF AN EXTERNAL CRATE is counted even for a job that would not
#     compile it. That inflates BOTH sides of every comparison, which is one
#     reason the floor below is 0.90 and not 1.0. It cannot manufacture a shared
#     core out of nothing the way the workspace-side over-approximation could,
#     because an external subtree is only entered when the workspace-side walk
#     genuinely reaches it.
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
from typing import NamedTuple

REPO_ROOT = Path(__file__).resolve().parent.parent

# The `shared-key` the grouped jobs carry in ci.yml. Pinned on both sides by
# scripts/tests/test_cache_shared_key_grouping.py.
GROUP_SHARED_KEY = "conformance-suites"

# Minimum pairwise Jaccard for two jobs to be allowed to share one entry.
# Chosen with headroom over the measured minimum (printed by this script) so a
# routine dependency addition does not red CI, while a job that grows a genuinely
# separate closure — tens of unique packages — trips it and forces a re-key.
OVERLAP_FLOOR = 0.90


class Cmd(NamedTuple):
    """One cargo invocation, as ci.yml runs it.

    `dev` is True for commands that build test targets (`cargo test`,
    `cargo clippy --all-targets`) and therefore pull dev-dependencies; False for
    `cargo run`/`cargo build`, which do not.
    """

    package: str
    features: tuple[str, ...] = ()
    dev: bool = False
    all_features: bool = False
    default_features: bool = True


# ci.yml jobs that share `GROUP_SHARED_KEY`, mapped to the cargo commands each
# one runs. Where the coincidence comes from: these five do turn on opt-in
# features, but the ones they turn on forward almost entirely to WORKSPACE
# crates — the sparq-reason* reasoners behind the inference arms contribute no
# external packages at all, and `jsonld-suite` adds a handful — while the
# genuinely heavy opt-in, the tokio/axum loopback server behind
# `service-loopback`, is off in every command here.
GROUPED: dict[str, tuple[Cmd, ...]] = {
    # cargo run --bin sparq-conformance / --bin sparq-conformance-scoreboard,
    # plus cargo test --test rdf_line_syntax_ratchet --test parser_differential.
    "conformance": (
        Cmd("sparq-conformance"),
        Cmd("sparq-conformance", dev=True),
    ),
    # cargo test -p sparq-shacl, in both feature states. No sparq-conformance
    # command at all in this lane.
    "shacl-conformance": (
        Cmd("sparq-shacl", dev=True),
        Cmd("sparq-shacl", ("shacl-af",), dev=True),
    ),
    # cargo test -p sparq-policy --test odrl_test_suite + the scoreboard binary.
    "odrl-conformance": (
        Cmd("sparq-policy", dev=True),
        Cmd("sparq-conformance"),
    ),
    # cargo test -p sparq-conformance --features jsonld-suite + the scoreboard.
    "jsonld-conformance": (
        Cmd("sparq-conformance", ("jsonld-suite",), dev=True),
        Cmd("sparq-conformance"),
    ),
    # cargo run --bin sparq-inference-conformance + one test command per opt-in
    # reasoner arm. Every one of those arms is a workspace crate (sparq-reason*),
    # so they add reasoning code but almost no external packages.
    "inference-conformance": (
        Cmd("sparq-conformance"),
        Cmd("sparq-conformance", ("d-entail",), dev=True),
        Cmd("sparq-conformance", ("rif-wg-core",), dev=True),
        Cmd("sparq-conformance", ("rif-core",), dev=True),
        Cmd("sparq-conformance", ("el-suite", "el-suite-par"), dev=True),
        Cmd("sparq-conformance", ("ql-experimental",), dev=True),
        Cmd("sparq-conformance", ("dl-direct",), dev=True),
    ),
}

# Jobs kept OUT of the group on the strength of this measurement. Each is
# asserted to sit BELOW the floor against the group union, so folding one in
# without re-measuring goes red.
#
# The five conformance/oracle lanes here look like group members and are not.
# What each one adds on top of the group's closure, per this measurement:
# `geo-conformance` the georust geometry stack (geo / geo-types / i_overlay /
# rstar / wkt); `text-oracle` and `rsp-oracle` a proptest property-testing tree
# (their crates' dev-dependencies); `solid-conformance` the arkworks ZK stack,
# reached via sparq-solid -> sparq-zk; and `service-federation-conformance` the
# whole tokio/axum async server stack, which `service-loopback` turns on in
# every one of its feature states and which roughly doubles its closure.
# `geiger` reports unsafe usage in sparq-core alone and is a strict SUBSET of
# the group; `lint` clippies and rustdocs the whole workspace with
# --all-features and is a strict superset of everything.
CLOSURE_EXCLUDED: dict[str, tuple[Cmd, ...]] = {
    "geo-conformance": (
        Cmd("sparq-geo", dev=True),
        Cmd("sparq-geo", ("geosparql_rewrite",), dev=True),
        Cmd("sparq-conformance"),
    ),
    "solid-conformance": (
        Cmd("sparq-solid", dev=True),
        Cmd("sparq-conformance"),
    ),
    "text-oracle": (
        Cmd("sparq-text", dev=True),
        Cmd("sparq-conformance"),
    ),
    "rsp-oracle": (
        Cmd("sparq-rsp", dev=True),
        Cmd("sparq-conformance"),
    ),
    "service-federation-conformance": (
        Cmd("sparq-conformance", ("service-loopback",), dev=True),
        Cmd("sparq-conformance", ("service",), dev=True),
        Cmd("sparq-conformance", ("http-protocol",), dev=True),
        Cmd("sparq-conformance", ("federation-descriptors",), dev=True),
        Cmd("sparq-conformance", ("http-protocol", "federation-descriptors"), dev=True),
    ),
    # cargo geiger --manifest-path crates/sparq-core/Cargo.toml — the shipped
    # graph of one crate, no dev-dependencies.
    "geiger": (Cmd("sparq-core"),),
    # cargo clippy --workspace --all-targets, then again with --all-features.
    # `()` means "every workspace member"; expanded in measure().
    "lint": (),
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


def _normalise(spec: object) -> dict:
    """A dependency declaration as a uniform dict (`dep = "1"` included)."""
    if not isinstance(spec, dict):
        return {"optional": False, "features": [], "default": True, "package": None}
    return {
        "optional": bool(spec.get("optional", False)),
        "features": list(spec.get("features") or []),
        # `default-features = false` is the only value that turns them off.
        "default": spec.get("default-features", True) is not False,
        "package": spec.get("package"),
    }


def declared_deps(manifest: dict, kinds: set[str]) -> dict[str, dict]:
    """Dependency declarations of a workspace manifest, keyed by DEPENDENCY KEY.

    The key — not the package name — is what `dep:x` / `x/y` in a `[features]`
    entry refers to, so renames are kept as `["name"]` alongside it.
    `[target.'cfg(...)'.dependencies]` tables are folded in: a job that builds on
    Linux still pays for whatever the Linux cfg selects. When the same key is
    declared in more than one table (typically a normal dep that is also a
    dev-dep) the declarations are merged the way cargo's union would resolve
    them: optional only if EVERY declaration is optional, features unioned,
    default features on if ANY declaration wants them.
    """
    out: dict[str, dict] = {}
    tables = [manifest] + list((manifest.get("target") or {}).values())
    for kind in kinds:
        section = _KIND_SECTION[kind]
        for table in tables:
            for key, spec in (table.get(section) or {}).items():
                dep = _normalise(spec)
                if key in out:
                    prev = out[key]
                    prev["optional"] = prev["optional"] and dep["optional"]
                    prev["features"] = sorted(set(prev["features"]) | set(dep["features"]))
                    prev["default"] = prev["default"] or dep["default"]
                    prev["package"] = prev["package"] or dep["package"]
                else:
                    out[key] = dep
    for key, dep in out.items():
        dep["name"] = dep["package"] or key
    return out


def enabled_deps(
    manifest: dict,
    requested: set[str],
    default_features: bool,
    kinds: set[str],
) -> dict[str, tuple[str, frozenset[str], bool]]:
    """Resolve a feature selection to the dependencies it actually enables.

    Returns `dep key -> (package name, features requested ON that dep, whether
    its default features are on)`. An `optional = true` dependency appears ONLY
    when the selection activates it — via `dep:x`, via an `x/y` edge, or via the
    implicit feature an optional dependency defines when no `dep:x` mentions it.
    """
    feats = manifest.get("features") or {}
    deps = declared_deps(manifest, kinds)

    # An optional dependency defines an implicit feature of the same name unless
    # some feature entry refers to it as `dep:x`.
    explicit = {
        entry[len("dep:"):].removesuffix("?")
        for entries in feats.values()
        for entry in entries
        if entry.startswith("dep:")
    }
    implicit = {k for k, d in deps.items() if d["optional"] and k not in explicit}

    enabled: set[str] = set()
    activated: set[str] = set()
    requests: dict[str, set[str]] = {}
    # `x?/y` only fires once something else has activated x, so it is retried to
    # a fixpoint rather than resolved in declaration order.
    deferred: list[tuple[str, str]] = []

    pending = list(requested)
    if default_features and "default" in feats:
        pending.append("default")

    progressed = True
    while pending or progressed:
        while pending:
            feature = pending.pop()
            if feature in enabled:
                continue
            enabled.add(feature)
            if feature not in feats:
                # Not a declared feature: either the implicit feature of an
                # optional dependency, or a name this model does not know.
                if feature in implicit:
                    activated.add(feature)
                continue
            for entry in feats[feature]:
                if entry.startswith("dep:"):
                    activated.add(entry[len("dep:"):])
                elif "/" in entry:
                    key, sub = entry.split("/", 1)
                    if key.endswith("?"):
                        deferred.append((key[:-1], sub))
                        continue
                    if key in deps and deps[key]["optional"]:
                        activated.add(key)
                    requests.setdefault(key, set()).add(sub)
                else:
                    pending.append(entry)
        progressed = False
        for pair in list(deferred):
            key, sub = pair
            if key in activated or (key in deps and not deps[key]["optional"]):
                deferred.remove(pair)
                requests.setdefault(key, set()).add(sub)
                progressed = True

    resolved: dict[str, tuple[str, frozenset[str], bool]] = {}
    for key, dep in deps.items():
        if dep["optional"] and key not in activated:
            continue
        wanted = frozenset(set(dep["features"]) | requests.get(key, set()))
        resolved[key] = (dep["name"], wanted, dep["default"])
    return resolved


def every_feature(manifest: dict) -> set[str]:
    """What `--all-features` selects: declared features + implicit optional ones."""
    declared = set((manifest.get("features") or {}).keys())
    optional = {
        k for k, d in declared_deps(manifest, {"normal", "build"}).items() if d["optional"]
    }
    return declared | optional


def external_closure(
    commands: tuple[Cmd, ...],
    workspace: dict[str, dict],
    by_name: dict[str, list[dict]],
    by_id: dict[tuple[str, str], dict],
) -> set[tuple[str, str]]:
    """External packages the given cargo commands compile, as (name, version).

    Workspace crates are traversed but never returned: rust-cache strips
    workspace-member artifacts from `target/` before saving, so they are not
    part of what an entry serves. A command's `dev` flag applies to ITS PACKAGE
    only — a transitive workspace dependency contributes its normal+build deps,
    not the dev-dependencies of its own test suite, which no command here builds.
    """

    def resolve(spec: str) -> list[tuple[str, str]]:
        parts = spec.split()
        name, version = parts[0], (parts[1] if len(parts) > 1 else None)
        found = by_name.get(name, [])
        if version is not None:
            found = [p for p in found if p["version"] == version]
        return [(p["name"], p["version"]) for p in found]

    transitive = frozenset({"normal", "build"})
    visited: set[tuple[str, frozenset[str], bool, frozenset[str]]] = set()
    entered: set[tuple[str, str]] = set()
    stack: list[tuple[str, frozenset[str], bool, frozenset[str]]] = []

    for cmd in commands:
        if cmd.package not in workspace:
            raise SystemExit(f"{cmd.package!r} is not a workspace member")
        selection = (
            every_feature(workspace[cmd.package]) if cmd.all_features else set(cmd.features)
        )
        kinds = frozenset({"normal", "build", "dev"} if cmd.dev else {"normal", "build"})
        stack.append((cmd.package, frozenset(selection), cmd.default_features, kinds))

    while stack:
        state = stack.pop()
        if state in visited:
            continue
        visited.add(state)
        package, selection, defaults, kinds = state
        for name, wanted, defaults_on in enabled_deps(
            workspace[package], set(selection), defaults, set(kinds)
        ).values():
            if name in workspace:
                stack.append((name, wanted, defaults_on, transitive))
            else:
                entered.update(resolve(name))

    external: set[tuple[str, str]] = set()
    seen = set(entered)
    pending = list(entered)
    while pending:
        node = pending.pop()
        external.add(node)
        for spec in (by_id.get(node) or {}).get("dependencies", []):
            for ident in resolve(spec):
                if ident[0] in workspace or ident in seen:
                    continue
                seen.add(ident)
                pending.append(ident)

    return external


def jaccard(a: set, b: set) -> float:
    union = a | b
    return 1.0 if not union else len(a & b) / len(union)


def measure() -> dict[str, set[tuple[str, str]]]:
    workspace = load_workspace()
    by_name, by_id = load_lock()
    closures: dict[str, set[tuple[str, str]]] = {}
    for job, commands in list(GROUPED.items()) + list(CLOSURE_EXCLUDED.items()):
        if not commands:
            # `lint`: clippy/rustdoc the whole workspace, once per feature state.
            commands = tuple(
                Cmd(member, dev=True, all_features=all_feats)
                for all_feats in (False, True)
                for member in sorted(workspace)
            )
        closures[job] = external_closure(commands, workspace, by_name, by_id)
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
        print(f"  {job:31s} : Jaccard {jaccard(closures[job], union):.3f} "
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
