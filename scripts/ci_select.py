#!/usr/bin/env python3
# [OPUS-4.8] Change-based CI test-selection: the affected-set selector.
# Bead sq-fmx4u.1 (epic sq-fmx4u). Design: research/change-based-test-selection.md
# §3 (affected-set algorithm), §4 (fail-safe rules), §7 (positions P1-P4).
# Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns).
#
# WHAT THIS IS (and is NOT):
#   Given the set of paths changed in a PR (a git diff), compute the set of
#   workspace crates whose tests/benchmarks MUST run = the reverse-dependency
#   closure (transitive dependents) of every changed crate, derived from
#   `cargo metadata`. Emits JSON: {"mode": "selected"|"full", "reason", "affected"}.
#
#   This is the SELECTOR LIBRARY (design §3). The CI WIRING — the `select`
#   pre-job (.github/workflows/ci-select.yml), the job-level `if:` guards, the
#   nextest filterset narrowing, ci-summary semantics — landed with bead
#   sq-fmx4u.3 [FABLE-5]; the rollout backstops (nightly full, ci-full label,
#   the enforcement flip) are bead sq-fmx4u.5. This file stays workflow-
#   agnostic: it reads a diff + cargo metadata and emits {mode, reason,
#   affected} plus the $GITHUB_OUTPUT lines (mode/affected/filterset) that the
#   wiring consumes.
#
# THE CORE INVARIANT (design §2, normative):
#   *A test is skipped => provably no change could affect it.* Skipping is an
#   optimization applied to a proof of non-interference; ABSENCE OF PROOF MEANS
#   RUN. Every branch below defaults to running MORE, never less:
#     - any §4.1 full-run trigger changed  => mode=full
#     - any changed path we cannot confidently attribute to a crate => mode=full
#     - ANY internal error (metadata parse, diff failure, unfetchable base,
#       malformed map) => mode=full, exit 0  (fail-closed, design §4.3)
#
# OWNERSHIP MAP (ci/path-ownership.toml, design §4.2):
#   The §4.1 full-run trigger table is baked in here as built-in DEFAULTS so the
#   selector fail-closes correctly on a clean clone even before the map file
#   exists. When `ci/path-ownership.toml` IS present it EXTENDS behaviour:
#   attributes non-crate paths to the crates whose tests read them, or marks a
#   path SAFE (proven inert for the Rust matrix). Bead sq-fmx4u.2 ships + audits
#   that map; this selector merely reads it. Absent map => built-in triggers +
#   crate-prefix ownership only (unmapped/unowned path => full).
#
# Usage:
#   ci_select.py                                   # real: git diff HEAD vs merge-base, cargo metadata
#   ci_select.py --base <sha> --head <sha>         # explicit revision pair
#   ci_select.py --event schedule                  # no diff => mode=full
#   ci_select.py --full                            # forced full (e.g. the `ci-full` label)
#   ci_select.py --shadow ...                      # report-only: compute + report the
#                                                  #   selection but emit mode=shadow so no
#                                                  #   downstream guard skips (design §6.4)
#   ci_select.py --changed-file paths.txt \        # hermetic inputs (tests): newline-delimited
#                --metadata-file meta.json          # cargo metadata JSON snapshot
#
# Stdlib only (design §7 P1): no third-party deps, no compile step. Runs under
# the CI setup-python 3.12; `tomllib` (stdlib >= 3.11) is imported lazily and
# only when a map file is actually present.
#
# ONE OPTIONAL PRECISION DEP — PyYAML ([OPUS-5] #5237). The content-based
# inert-edit carve-out (`yaml_edit_is_inert`) imports `yaml` LAZILY and nowhere
# else. It is optional in the strict sense: with PyYAML absent the selector runs
# unchanged on the stdlib alone and simply proves less (every `.github/**` YAML
# edit keeps forcing mode=full). P1 is therefore refined, not revoked — no
# third-party dep is ever REQUIRED for a correct selection.

from __future__ import annotations

import argparse
import fnmatch
import json
import os
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import PurePosixPath

# --- §4.1 unconditional full-run triggers (built-in defaults) ----------------
# Each entry is (pattern, reason). A pattern ending in "/" is a directory prefix
# (matches the dir itself or anything beneath it); otherwise it is an EXACT
# repo-relative path. Root "Cargo.toml" therefore matches ONLY the workspace
# root manifest — a crate's "crates/x/Cargo.toml" is attributed to crate x by
# ownership (and a version bump there rewrites Cargo.lock => full anyway, §3.3).
_FULL_TRIGGERS: list[tuple[str, str]] = [
    ("Cargo.lock", "Cargo.lock changed (resolved versions feed every build)"),
    ("Cargo.toml", "root Cargo.toml changed (workspace members / deps / lints / profiles / [patch])"),
    ("rust-toolchain", "toolchain pin changed (compiler version affects all codegen)"),
    ("rust-toolchain.toml", "toolchain pin changed (compiler version affects all codegen)"),
    (".cargo/", "cargo config changed (global rustflags / registries / build config)"),
    (".github/", "CI definition changed (including the selection wiring itself)"),
    ("scripts/", "shared CI/gate/coverage script changed (executed by CI)"),
    ("deny.toml", "cargo-deny config changed (redefines a supply-chain green)"),
    ("supply-chain/", "cargo-vet / supply-chain config changed"),
    ("ci/path-ownership.toml", "selection policy (ownership map) changed"),
]


# --- ORCHESTRATION-ONLY carve-out (change-class layer; sq path-aware CI) ------
# [OPUS-4.8] The broad `.github/` and `scripts/` full-run triggers above are
# CORRECT-BY-DEFAULT but OVER-BROAD: a PR that changes ONLY orchestration/workflow
# tooling (PR/issue/bead automation, routing, the merge-queue batchers, agent
# config) forces the FULL Rust matrix even though NOTHING it touches is read by any
# Rust build/test/clippy/coverage/bench/fuzz/CodeQL job. The maintainer's ask:
# stop running the engine CI on those PRs (they dominate the drain).
#
# SOUNDNESS (§2 — a skip must be a PROOF of non-interference, not a guess): this is
# a small, EXPLICIT, AUDITED ALLOWLIST of paths PROVEN inert for the Rust matrix,
# consulted BEFORE `_trigger_match` so it is the ONLY thing that can rescue a
# `.github/`/`scripts/` path from the full-run trigger. It is a WHITELIST, never a
# denylist: a `.github/`/`scripts/` path NOT matched here still hits the trigger and
# forces full (absence of proof ⇒ run). A matched path is treated exactly like an
# ownership-map `safe = true` verdict — it contributes NO crate, so if EVERY changed
# path is orchestration-safe the selection is mode=selected with an empty affected
# closure and every Rust lane (incl. the bench/fuzz/wasm SEED lanes) skips.
#
# THE INERTNESS OBLIGATION (enforced by scripts/tests/test_ci_select.py
# `OrchestrationSafeInertnessTests`): every entry here must be a path that NO
# `.github/workflows/*.yml` step feeding a Rust build/test/gate ever reads, and that
# no crate `include_str!`/`include_bytes!`/`../`-escapes to. The test greps the
# Rust-CI workflows for a reference to each allowlisted script/dir and FAILS if one
# is referenced (i.e. is actually Rust-CI-affecting) — so an entry can never silently
# become unsound when a script is later wired into a gate. A pattern ending in "/" is
# a directory prefix; otherwise an exact repo-relative path.
#
# NOT here (deliberately — these ARE read by the Rust matrix, so they must keep
# triggering full): every Rust-CI script (ci_select.py, ci_summary_gate.py,
# coverage*.py/sh, perf-gate.py, assemble-feature-matrix.py, feature-matrix-tiers.py,
# check-*.py gates, fetch-*.sh conformance fetchers, ci-bench.sh, ci-free-disk.sh,
# unsafe/mutants gates, sbom/vex tooling, docker-smoke.sh, wasm-deps-guard.sh, the
# fv/formal lane scripts, and scripts/tests/* that gate the engine); the Rust-CI
# workflow files themselves (ci.yml, feature-matrix.yml, codeql.yml, supply-chain.yml,
# bench.yml, fuzz.yml, miri.yml, asan.yml, kani.yml, metamorph.yml,
# vectorized-feature-off.yml, ci-select.yml, ci-summary.yml, conformance/coverage
# lanes); `.github/feature-matrix.d/**`; `.github/codeql/**`; `.github/actions/**`.
_ORCHESTRATION_SAFE: list[str] = [
    # Orchestration configuration + agent harness (never compiled/tested by cargo).
    "orchestration/",       # routing.toml + orchestration policy (the #3416 class)
    ".claude/",             # agent definitions / skills / workflows / settings
    ".beads/",              # the bead task DB (never read by any build/test)
    # Orchestration-only workflow files (PR/issue/bead/merge automation — none run
    # cargo build/test/clippy/coverage/bench/fuzz/CodeQL).
    ".github/workflows/triage-issue.yml",
    ".github/workflows/retriage.yml",
    ".github/workflows/pr-backlog.yml",
    ".github/workflows/pr-title.yml",
    ".github/workflows/batch-merge.yml",
    ".github/workflows/bead-autoclose.yml",
    ".github/workflows/promote-on-approval.yml",
    ".github/workflows/differential-update.yml",
    ".github/workflows/kb-dump.yml",
    ".github/workflows/pkg-ingest.yml",
    # NOTE deliberately NOT here: selection-alarm.yml / formal-alarm.yml — they are
    # monitors for the Rust/formal lanes (borderline), so they keep triggering full
    # (fail-closed; the value of skipping them is negligible and the audit is cleaner).
    # Orchestration-only scripts (PR/issue/bead/routing/dispatch automation). Each is
    # pinned inert by the OrchestrationSafeInertnessTests grep.
    "scripts/triage.py",
    "scripts/retriage.py",
    "scripts/routing-validate.py",
    "scripts/dispatch-plan.py",
    "scripts/bd-to-issues.py",
    "scripts/pr-backlog.py",
    "scripts/batch-merge.py",
    "scripts/save-agent-log.sh",
    "scripts/push-frontier.sh",
]


# [OPUS-5] sq-g25hr: DEPLOYMENT-MANIFEST surfaces — the k8s/helm/terraform/bicep/
# PaaS manifests under deploy/ plus the two lint workflows that are their ONLY CI
# consumers. Same allowlist shape and same inertness obligation as
# _ORCHESTRATION_SAFE above (a "/"-suffixed entry is a directory prefix, else an
# exact repo-relative path), kept as a SEPARATE list purely so the audit trail can
# say "deploy-only" rather than mislabelling terraform as orchestration.
#
# WHY IT IS INERT for the Rust matrix: no workspace crate reads deploy/ (no
# include_str!/include_bytes!/CARGO_MANIFEST_DIR-relative read — the
# `deploy/**  safe = true` entry in ci/path-ownership.toml makes that an ENFORCED
# claim: scripts/ci_audit_inputs.py FAILS if any crate ever starts reading a
# safe-listed path). deploy/ IS read by deploy-lint.yml / deploy-terraform-lint.yml
# / container-scan.yml / release.yml and by two pure-python checks that live in
# docs-quality.yml + supply-chain.yml (test_release_container_multiarch.py,
# test_dependabot_coverage.py) — all of which are SEPARATE workflows this selector
# does not gate, so they keep running on a deploy-only PR. This list therefore only
# ever removes deploy paths from the *Rust* full set.
#
# The two workflow files are on this list (and consulted BEFORE _trigger_match) for
# the same reason the orchestration workflow files are: they live under `.github/`,
# which is otherwise an unconditional full-run trigger, and neither runs
# cargo build/test/clippy/coverage/bench/fuzz/CodeQL. `DeployOnlyInertnessTests`
# in scripts/tests/test_ci_select.py pins both properties.
_DEPLOY_ONLY: list[str] = [
    "deploy/",
    ".github/workflows/deploy-lint.yml",
    ".github/workflows/deploy-terraform-lint.yml",
]


def _allowlist_match(path: str, allowlist: list[str]) -> bool:
    """Is `path` on `allowlist`? A "/"-suffixed entry is a directory prefix; else
    an exact repo-relative path."""
    for entry in allowlist:
        if entry.endswith("/"):
            if path == entry.rstrip("/") or path.startswith(entry):
                return True
        elif path == entry:
            return True
    return False


def _orchestration_safe_match(path: str) -> bool:
    """[OPUS-4.8] Is `path` on the audited orchestration-only inert allowlist?
    A "/"-suffixed entry is a directory prefix; else an exact path. Consulted
    BEFORE _trigger_match so it is the sole rescue from a .github/scripts trigger;
    it can only ever REMOVE a path from the full set for a proven-inert path (a
    non-matching path still hits the trigger => full)."""
    return _allowlist_match(path, _ORCHESTRATION_SAFE)


def _deploy_only_match(path: str) -> bool:
    """[OPUS-5] sq-g25hr: is `path` on the audited deployment-manifest allowlist?
    Same pre-trigger position and same fail-safe posture as
    _orchestration_safe_match — a non-matching path still hits the trigger => full."""
    return _allowlist_match(path, _DEPLOY_ONLY)


# --- INERT-EDIT carve-out (CONTENT-based; #5237, generalising #2536) ---------
# [OPUS-5] The two allowlists above are PATH-based proofs: "this file is never read
# by the Rust matrix". They cannot cover a `.github/**` YAML file that genuinely DOES
# gate PRs, so by path such a file must keep forcing full. But a share of the edits
# those files receive are pure COMMENT edits, which cannot change what CI does.
#
# THE PROOF (content-based, per-revision-pair): an ALLOWLISTED `.github/**` YAML
# file's edit is inert iff its PARSED YAML DOCUMENT is identical at the merge-base
# and at head. Comments OUTSIDE a block scalar vanish under parsing, so a
# pure-comment edit compares equal; a comment added INSIDE a `|` block scalar stays
# part of the scalar STRING and compares unequal, which fail-closes to the full
# matrix exactly as it must (that text is a value, and is typically executed).
#
# THE PREMISE THE PROOF NEEDS — and why it is an ALLOWLIST, not a denylist. Parse
# equality is inertness only if EVERY consumer of the file reads it as YAML. A
# consumer that reads the raw TEXT breaks it: a comment carrying a searched-for
# literal flips a `contains`/`count` assertion with the document unchanged, so the
# selector could skip the very check that observes the comment. Enumerating the
# raw-text readers (the first shape of this carve-out, which grepped `crates/**/*.rs`
# for `.github/**` YAML path literals) is NOT sound, and the repo disproves it three
# ways — none of them Rust, one of them undetectable by a full-path grep:
#   * scripts/miri_classify_verdict.py `read_text`s `.github/workflows/miri.yml` and
#     asserts substrings + an exact `count(...) == 4`;
#   * scripts/check-fast-fix-ring-guard.py byte-pins `.github/workflows/`
#     `fast-fix-ring.yml` against a golden with comments explicitly load-bearing —
#     and builds the path as `Path(".github") / "workflows" / "fast-fix-ring.yml"`,
#     so no full-path literal exists for a grep to find;
#   * scripts/tests/test_mergequeue_cache_posture.py substring-scans the raw text of
#     EVERY `.github/workflows/*.yml` (`RUST_CACHE in p.read_text()`), which no
#     per-file inventory can ever discharge — it is a whole-directory raw-text read.
# The third alone means NO `.github/workflows/**` file can be proven inert by parse
# equality. So the carve-out fires only on trees whose consumers have been AUDITED as
# parse-based, listed below; every other `.github/**` YAML keeps forcing full.
#
# WHY IT IS RESTRICTED TO `.github/**`: outside that tree a *.yml can be a test
# FIXTURE that a crate reads verbatim (byte-for-byte, comments included), where
# parse-equality would NOT be inertness. The restriction is part of the proof, not
# a convenience.
#
# THE PyYAML POSITION (design §7 P1, "stdlib only", refined — NOT revoked): the
# selector still RUNS on the stdlib alone. PyYAML is an OPTIONAL PRECISION
# dependency: it is imported lazily and ONLY here, and its absence makes this
# carve-out return False for every path, i.e. the pre-#5237 behaviour (the
# `.github/` trigger fires => mode=full). A hand-rolled comment stripper was the
# alternative and was rejected: hand-rolled YAML block-scalar tracking in a
# soundness-critical path is exactly the risk this proof is meant to remove.
# .github/workflows/ci-select.yml installs PyYAML best-effort (non-fatal), so a
# failed install costs precision, never correctness.
_YAML_INERT_ROOT = ".github/"
_YAML_SUFFIXES = (".yml", ".yaml")

# The AUDITED PARSE-ONLY trees: `.github/**` YAML path prefixes every one of whose
# consumers loads the file with a YAML parser, never as raw text. ONLY these are
# eligible for the carve-out. Adding an entry is a soundness decision and requires
# auditing that tree's consumers; `YamlParseOnlyAllowlistTests` then pins the audit
# mechanically — it fails when a new file names one of these paths (by basename,
# which catches the segment-built-path case above) and when a new consumer of the
# tree itself appears undeclared.
#   * `.github/feature-matrix.d/` — per-crate feature-matrix leg DESCRIPTORS. Four
#     consumers glob the directory and `yaml.safe_load` each fragment
#     (scripts/assemble-feature-matrix.py, feature-matrix-tiers.py,
#     check-feature-test-execution.py, scripts/tests/test_feature_matrix_assemble.py);
#     the only other reference to the tree is a `paths:` filter in
#     .github/workflows/feature-matrix.yml, which GitHub evaluates without reading a
#     fragment. Nothing reads a fragment's text. That audited set is pinned by
#     `YamlParseOnlyAllowlistTests._AUDITED_TREE_CONSUMERS`, so a new consumer reds.
#     Over the 1500 first-parent commits before this was written, 117 touch this tree
#     and 8 change only blank/`#`-comment diff lines — the value this buys.
_YAML_PARSE_ONLY_TREES: tuple[str, ...] = (
    ".github/feature-matrix.d/",
)


def _yaml_safe_loader():
    """The lazily-imported `yaml.safe_load`, or None when PyYAML is absent.

    Returning None is the fail-closed signal: the caller then proves nothing and
    the path falls through to its `.github/` full-run trigger."""
    try:
        import yaml  # optional precision dep; see the block comment above
    except Exception:  # ModuleNotFoundError, or a broken install
        return None
    return yaml.safe_load


def _canonical(node):
    """A TYPE-TAGGED, order-canonical form of a parsed YAML document, so that
    `_canonical(a) == _canonical(b)` is document identity rather than Python's
    looser `==`.

    Two refinements over a plain `a == b` on the loaded objects, both in the
    fail-closed direction (they can only make two documents compare UNEQUAL,
    never equal):
      * scalars carry their type name and `repr`, so YAML `true` never compares
        equal to `1` (Python's `True == 1`) and `1` never equals `"1"`;
      * mappings are compared as a key-sorted item set (key ORDER in a mapping is
        not meaningful to any consumer) while sequences keep their order (step
        order in a workflow IS meaningful).
    """
    if isinstance(node, dict):
        return ("map", tuple(sorted(
            ((_canonical(k), _canonical(v)) for k, v in node.items()), key=repr)))
    if isinstance(node, (list, tuple)):
        return ("seq", tuple(_canonical(v) for v in node))
    if isinstance(node, (set, frozenset)):  # YAML !!set
        return ("set", tuple(sorted((_canonical(v) for v in node), key=repr)))
    return ("scalar", type(node).__name__, repr(node))


def yaml_edit_is_inert(path: str, blob_reader, loader=None) -> bool:
    """[OPUS-5] #5237: is this `.github/**` YAML edit PROVABLY inert — same parsed
    document at base and head?

    `blob_reader(side, path) -> str | None` reads the file's text at side
    "base"/"head" (None = unreadable/absent). FAIL-CLOSED on every uncertainty —
    no reader, no PyYAML, a path outside an AUDITED PARSE-ONLY tree
    (`_YAML_PARSE_ONLY_TREES`), a non-YAML suffix, an added/deleted/unreadable blob,
    a parse error, or any unequal document all return False, which leaves the path
    on its normal full-run trigger.
    """
    if blob_reader is None:
        return False
    if not path.startswith(_YAML_INERT_ROOT) or not path.endswith(_YAML_SUFFIXES):
        return False
    if not any(path.startswith(tree) for tree in _YAML_PARSE_ONLY_TREES):
        # Not an audited parse-only tree: some consumer may read this file's raw
        # TEXT, and parse-equality would then prove nothing. Notably every
        # `.github/workflows/**` file is in this branch.
        return False
    if loader is None:
        loader = _yaml_safe_loader()
        if loader is None:
            return False  # no parser => no proof => run everything
    base_text = blob_reader("base", path)
    head_text = blob_reader("head", path)
    if base_text is None or head_text is None:
        return False  # added, deleted or unreadable on one side => no proof
    try:
        base_doc = loader(base_text)
        head_doc = loader(head_text)
    except Exception:  # any parse failure => no proof
        return False
    return _canonical(base_doc) == _canonical(head_doc)


def git_blob_reader(base: str | None, head: str | None, repo_root: str | None):
    """A `yaml_edit_is_inert` blob reader backed by `git show <rev>:<path>`.

    Returns None (no reader at all => the carve-out is inert) when the revision
    pair cannot be resolved. `base` is resolved to the MERGE-BASE so the two texts
    are the same pair the three-dot `git diff base...head` reported as changed;
    without that, an unrelated base-branch edit to the same file would compare
    unequal and pointlessly force full.
    """
    if not base or not head:
        return None
    try:
        merge_base = _run(["git", "merge-base", base, head], cwd=repo_root).strip()
    except SelectorError:
        return None
    if not merge_base:
        return None
    revs = {"base": merge_base, "head": head}

    def read(side: str, path: str) -> str | None:
        rev = revs.get(side)
        if rev is None:
            return None
        try:
            return _run(["git", "show", f"{rev}:{path}"], cwd=repo_root)
        except Exception:
            # Absent on that side (add/delete), binary, or undecodable: no proof.
            return None

    return read


# Change-class labels emitted for the audit trail (design: the gate renders an
# explicit "skipped-by-class" attribution). PURELY DESCRIPTIVE of the diff — the
# skip decision itself is still the sound mode/affected math below.
_CLASS_ENGINE = "engine"
_CLASS_ORCHESTRATION = "orchestration-only"
_CLASS_DOCS = "docs-only"
# [OPUS-5] sq-g25hr: deployment manifests only (deploy/** + the two deploy lint
# workflows) — see _DEPLOY_ONLY.
_CLASS_DEPLOY = "deploy-only"
# [OPUS-5] sq-g25hr: EVERY changed path is on a proven-inert surface, but they span
# MORE THAN ONE of {orchestration, docs, deploy}. This used to collapse into
# `mixed` — the same token an engine+docs diff produces — so the consumers' skip
# case-arm could not distinguish "provably nothing for the Rust matrix" from
# "some Rust changed too" and conservatively ran the full suite. That is exactly
# the observed cloud-deploy class (deploy/** + deploy/**/README.md +
# .github/workflows/deploy-*.yml spans deploy AND, once other prose is touched,
# docs). The union of inert surfaces is inert — each component is INDEPENDENTLY
# proven not to be read by the Rust matrix, and inertness composes — so this class
# is as skippable as its pure counterparts. `mixed` now means EXACTLY "engine +
# something inert", which is the only reading that must run the full suite.
_CLASS_INERT_MIXED = "inert-mixed"
_CLASS_MIXED = "mixed"

# The classes that PROVE the Rust matrix has nothing to do. The workflow `changes`
# pre-jobs (ci.yml / feature-matrix.yml / codeql.yml) case on exactly these tokens
# and treat every other token — including an unrecognised one — as a full run.
# scripts/tests/test_ci_select_wiring.py pins the workflow case-arms against this
# tuple so the two cannot drift.
_INERT_CLASSES: tuple[str, ...] = (
    _CLASS_ORCHESTRATION,
    _CLASS_DOCS,
    _CLASS_DEPLOY,
    _CLASS_INERT_MIXED,
)

# Docs-only surfaces: pure prose/markdown that no Rust build/test reads. NOTE a
# crate-owned README.md/*.md is deliberately NOT docs-only — it can be pulled into a
# doctest via #[doc = include_str!(...)] (design §3.2), so it is crate-owned and
# classified engine. AGENTS.md / skills/** feed the sparq-kb PKG tests (ownership
# map), so they are NOT docs-only either. This set is the repo-level prose that the
# ownership map already proves SAFE (research/**) plus top-level docs/ markdown.
_DOCS_ONLY_PREFIXES: list[str] = [
    "docs/",
    "research/",
]


def classify_change(changed_paths: list[str]) -> str:
    """[OPUS-4.8] Pure change-class of a diff (WHAT surfaces changed), for the audit
    trail. Orthogonal to `mode` (HOW MUCH runs) — this only LABELS; the sound skip
    math is unchanged. Fail-closed: any path that is on NONE of the proven-inert
    allowlists (orchestration-safe / docs-only / deploy-only) makes the class
    `engine` (or `mixed` if the diff also has inert paths), so a class is never MORE
    permissive than the mode. Empty diff => engine (a non-PR/full event carries no
    diff and runs everything anyway).

    [OPUS-5] sq-g25hr: a diff confined to inert surfaces but SPANNING more than one
    of them is `inert-mixed`, not `mixed` — see _CLASS_INERT_MIXED. `mixed` now
    means exactly "at least one engine path plus something inert"."""
    seen_inert: set[str] = set()
    seen_other = False
    for path in changed_paths:
        path = path.strip()
        if not path:
            continue
        if _orchestration_safe_match(path):
            seen_inert.add(_CLASS_ORCHESTRATION)
        elif any(path == p.rstrip("/") or path.startswith(p) for p in _DOCS_ONLY_PREFIXES):
            seen_inert.add(_CLASS_DOCS)
        elif _deploy_only_match(path):
            seen_inert.add(_CLASS_DEPLOY)
        else:
            seen_other = True
    if seen_other:
        # Any engine/unclassified path present. Pure-engine vs mixed is informational.
        return _CLASS_MIXED if seen_inert else _CLASS_ENGINE
    if len(seen_inert) > 1:
        return _CLASS_INERT_MIXED
    if seen_inert:
        return next(iter(seen_inert))
    return _CLASS_ENGINE


# --- phase-2 singleton-lane -> seed crates (design §5.2; beads sq-fmx4u.6, sq-mel85)
# [OPUS-4.8] A "lane" is a SINGLETON CI job (not a per-crate matrix leg) that
# always exercises a FIXED set of crates: the fuzz smoke (fuzz.yml), the wasm
# bundle build (ci.yml `wasm`), and the perf-gate benchmark (bench.yml `bench` —
# added by sq-mel85 [SONNET-4.6]). Phase 1 left these always-run (design P7); phase 2
# maps each to the crate closure it exercises and skips it when that closure is
# provably unaffected. A lane is affected iff any SEED crate is in the affected
# closure — and because `affected` is the REVERSE-dependency closure of the diff,
# a seed is in it iff the seed itself OR anything it transitively depends on
# changed, which is exactly "could this lane's build/behaviour differ" (design
# §3.3). So the forward-dependency reasoning is captured by a membership test on
# the reverse closure — no separate forward walk is needed here.
#
# SEEDS (single source of truth; the fuzz.yml + ci.yml `wasm` + bench.yml `bench`
# job `if:` guards MUST reference exactly these crate names — pinned by
# scripts/tests/test_ci_select_wiring.py, mirrored informationally in
# ci/path-ownership.toml `[lanes]`):
#   * fuzz — the cargo-fuzz targets. The out-of-workspace fuzz crate
#     (fuzz/Cargo.toml) depends on exactly these three: sparq-core (RDF parsers +
#     mmap loader), sparq-engine (SPARQL parse), sparq-shacl (SHACL validation).
#   * wasm — the ci.yml `wasm` job builds every browser bundle it names; each seed
#     pulls its own engine/core wasm32 graph, so the seed set is the bundle crates.
#   * bench — [SONNET-4.6] sq-mel85: the perf-gate benchmark (bench.yml). Seeds are
#     the benchmarked-crate closure of the HARD-GATED (merge-blocking) metrics that
#     scripts/perf-gate.py enforces on PRs: the store/dict byte-layout + parse
#     metrics come from sparq-core (exercised via the release binaries the bench job
#     builds — sparq-cli + sparq-bench — which both depend on sparq-core, so a core
#     change lands in this reverse closure), and the wasm_bundle_bytes floor comes
#     from the sparq-wasm browser bundle ci-bench.sh builds + measures. sparq-wasm
#     is a DIRECT seed on purpose: that floor is enforced ONLY in bench.yml and a
#     wasm-only diff does NOT flow up into engine/cli/bench (they do not depend on
#     sparq-wasm), so omitting it would silently drop the bundle gate for a wasm-only
#     PR — exactly the unsound skip §2 forbids. sparq-engine is the headline
#     benchmarked crate (and is embedded in the wasm bundle).
#     TRUE INVARIANT FOR SEED SOUNDNESS: every hard-gated metric measured on the
#     PR/merge_group tier must have its source crate reach a bench seed (store/dict/parse
#     ← sparq-core via the release binaries; wasm_bundle ← sparq-wasm). The four
#     additional mode:auto ratchets in bench/perf-baseline.json — fts_bytes_per_doc
#     (sparq-text), vectors_diskann_recall_at10 / vectors_pq_recall_at10 (sparq-vectors),
#     geo_compliance_deficit (sparq-geo) — need NO bench seed because scripts/ci-bench.sh
#     measures them on the MAIN tier only (GITHUB_REF guards skip them on any non-main
#     ref); every main-tier event is mode=full by construction. vectors/geo additionally
#     reach the sparq-cli seed transitively, but the soundness rests on the main-tier-only
#     measurement, not on that incidental reachability. On push-to-main the selector
#     returns mode=full (push is not a pull_request/merge_group event), so the FULL suite
#     always runs on main and the auto-ratchet + benchmark-data history stay continuous
#     (design §6.1).
# FAIL-CLOSED (design §2/§4.3): `lane_runs` runs the lane whenever mode != selected
# (full/shadow/error) OR any seed is not a current workspace member (a typo'd or
# renamed seed can never silently skip a lane — it forces the lane to run). The
# YAML guards inherit the same posture: an empty/missing `mode`/`affected` output
# satisfies the `mode != 'selected'` disjunct => RUN.
_LANE_SEEDS: dict[str, list[str]] = {
    "fuzz": ["sparq-core", "sparq-engine", "sparq-shacl"],
    "wasm": [
        "sparq-wasm",
        "sparq-reason-wasm",
        "sparq-rsp-wasm",
        "sparq-text-wasm",
        "sparq-shacl-wasm",
        "sparq-introspect",
        "sparq-solid",
        # [FABLE-5] sq-98c: not a bundle crate — the wasm job build+clippy-gates that
        # sparq-vectors keeps compiling on wasm32 with memmap2 target-gated out.
        "sparq-vectors",
    ],
    # [SONNET-4.6] sq-mel85: perf-gate bench closure (see the `bench` bullet above).
    "bench": [
        "sparq-engine",
        "sparq-cli",
        "sparq-bench",
        "sparq-wasm",
    ],
    # [FABLE-5] sq-0iqzw: fuzz.yml `differential-smoke` — the PR-level BLOCKING
    # sparq-vs-Oxigraph differential regression windows (the sparq-bench in-process
    # oracle). Seeds: the engine under test (sparq-core/sparq-engine) plus the
    # harness/oracle crate itself, so a generator or comparator change re-runs its
    # own gate.
    "differential-smoke": ["sparq-core", "sparq-engine", "sparq-bench"],
}


class SelectorError(Exception):
    """Any internal failure. main() traps this => mode=full, exit 0 (§4.3)."""


@dataclass
class Selection:
    """Rich selection result. main() serialises only {mode, reason, affected}."""

    mode: str  # "selected" | "full"
    reason: str
    affected: list[str]  # sorted workspace-member names
    # Transparency detail for the step-summary (design §5.1); not in the JSON contract.
    changed_crates: list[str] = field(default_factory=list)
    file_owners: list[tuple[str, str]] = field(default_factory=list)  # (path, owner-label)
    all_members: list[str] = field(default_factory=list)
    # [OPUS-4.8] change-class of the diff (see classify_change / _INERT_CLASSES):
    # the audit-trail label for WHY the Rust lanes were (or were not) skipped. Part of
    # the JSON contract so the gate + tooling can render "skipped-by-class: <class>".
    change_class: str = "engine"

    def to_json_obj(self) -> dict:
        return {
            "mode": self.mode,
            "reason": self.reason,
            "affected": self.affected,
            "change_class": self.change_class,
        }


# --- metadata model ----------------------------------------------------------
@dataclass
class Workspace:
    root: str
    members: set[str]  # workspace-member package names
    # in-repo path packages (members + non-member in-repo path deps): name -> rel dir
    owners: list[tuple[str, str]]  # (rel_dir, name), only non-empty dirs, sorted longest-first
    reverse_adj: dict[str, set[str]]  # dep-name -> {packages that depend on it}


def _rel_dir(manifest_path: str, root: str) -> str:
    """Repo-relative POSIX dir of a Cargo.toml, or '' if outside root / at root."""
    mp = PurePosixPath(manifest_path)
    rt = PurePosixPath(root)
    try:
        rel = mp.parent.relative_to(rt)
    except ValueError:
        return ""  # manifest lives outside the repo root
    s = rel.as_posix()
    return "" if s == "." else s


def parse_workspace(meta: dict) -> Workspace:
    """Build the in-repo dependency model from `cargo metadata` JSON.

    In-repo path packages = packages whose `source` is null (path/workspace, not
    a registry/git dep) AND whose manifest lives under `workspace_root`. This
    covers workspace members PLUS any non-member in-repo path dependency
    (design §3.2, P3). Edges are the package-level dependency lists — normal +
    dev + build kinds, optional regardless of feature, target-specific
    regardless of host (design §3.3, P2): a conservative superset of any
    feature-resolved graph.
    """
    try:
        root = meta["workspace_root"]
        packages = meta["packages"]
        member_ids = set(meta["workspace_members"])
    except (KeyError, TypeError) as exc:
        raise SelectorError(f"cargo metadata missing expected keys: {exc}") from exc

    id_to_name: dict[str, str] = {}
    inrepo_names: set[str] = set()
    inrepo_dirs: dict[str, str] = {}
    inrepo_deps: dict[str, list[dict]] = {}

    for pkg in packages:
        pid = pkg.get("id")
        name = pkg.get("name")
        if pid is not None and name is not None:
            id_to_name[pid] = name
        # source == null AND manifest under root => an in-repo path package.
        if pkg.get("source") is None and name is not None:
            rel = _rel_dir(pkg.get("manifest_path", ""), root)
            # A manifest outside the root yields '' here; skip those (external
            # path deps live outside the repo and cannot own a changed path).
            manifest_under_root = str(pkg.get("manifest_path", "")).startswith(str(root))
            if manifest_under_root:
                inrepo_names.add(name)
                inrepo_dirs[name] = rel
                inrepo_deps[name] = pkg.get("dependencies", [])

    members = {id_to_name[i] for i in member_ids if i in id_to_name}

    # Reverse edges among in-repo packages only. A dependency is matched to an
    # in-repo package by NAME (our crate names are distinctive `sparq-*`); this
    # is deliberately conservative — if in doubt, an edge is included (run more).
    reverse_adj: dict[str, set[str]] = {n: set() for n in inrepo_names}
    for name, deps in inrepo_deps.items():
        for dep in deps:
            dn = dep.get("name")
            if dn in inrepo_names and dn != name:
                reverse_adj[dn].add(name)

    owners = sorted(
        ((d, n) for n, d in inrepo_dirs.items() if d),  # drop empty-dir (root) owners: fail-safe
        key=lambda pair: len(pair[0]),
        reverse=True,
    )
    return Workspace(root=root, members=members, owners=owners, reverse_adj=reverse_adj)


def reverse_closure(crate: str, reverse_adj: dict[str, set[str]]) -> set[str]:
    """{crate} U all transitive dependents of crate."""
    seen = {crate}
    stack = [crate]
    while stack:
        cur = stack.pop()
        for dependent in reverse_adj.get(cur, ()):  # .get() tolerates a missing key
            if dependent not in seen:
                seen.add(dependent)
                stack.append(dependent)
    return seen


def lane_runs(sel: "Selection", seeds: list[str]) -> bool:
    """[OPUS-4.8] sq-fmx4u.6 / sq-mel85: does a singleton lane (fuzz / wasm /
    bench) need to run for this Selection? This is the EXECUTABLE SPEC of the
    fuzz.yml + ci.yml `wasm` + bench.yml `bench` job `if:` guards — the YAML
    expresses the identical rule inline
    (`mode != 'selected' || contains(affected, '"<seed>"') || ...`), and
    scripts/tests/test_ci_select_wiring.py pins the guards' seed set against
    `_LANE_SEEDS` so the two cannot drift.

    Runs (returns True) unless the lane is PROVABLY inert (design §2):
      * mode != "selected" (full / shadow / error path) => run everything;
      * any seed is not a current workspace member => run (fail-closed: a
        typo'd/renamed seed must never silently skip its lane);
      * otherwise run iff some seed is in the affected reverse-dep closure.
    """
    if sel.mode != "selected":
        return True
    members = set(sel.all_members)
    if any(s not in members for s in seeds):
        return True  # unknown seed: cannot prove inert => run (fail-closed)
    affected = set(sel.affected)
    return any(s in affected for s in seeds)


# --- ownership matching ------------------------------------------------------
def _trigger_match(path: str) -> str | None:
    for pattern, reason in _FULL_TRIGGERS:
        if pattern.endswith("/"):
            if path == pattern.rstrip("/") or path.startswith(pattern):
                return reason
        elif path == pattern:
            return reason
    return None


def owning_package(path: str, owners: list[tuple[str, str]]) -> str | None:
    """Longest-prefix crate that owns `path`, or None (design §3.2)."""
    for reldir, name in owners:  # owners is pre-sorted longest-first
        if path == reldir or path.startswith(reldir + "/"):
            return name
    return None


def _glob_match(path: str, pattern: str) -> bool:
    """Match an ownership-map pattern. `dir/**` => any path under dir; else fnmatch."""
    if pattern.endswith("/**"):
        root = pattern[:-3]
        return path == root or path.startswith(root + "/")
    return fnmatch.fnmatch(path, pattern)


def apply_ownership_map(path: str, map_entries: list[dict]) -> tuple[str, list[str]] | None:
    """First matching OWNERSHIP-VERDICT entry wins.

    Returns ("safe", []) for a SAFE-listed path, ("crates", [names]) for an
    attributed path, or None if no verdict entry matches (=> caller treats as
    unowned). A `readers`-only entry (the additional-readers mechanism, sq-m4bxc)
    is NOT an ownership verdict — it is applied separately by `additional_readers`
    — so it is transparent here (skipped, never a verdict and never a raise).
    """
    for entry in map_entries:
        pattern = entry.get("pattern")
        if not isinstance(pattern, str) or not _glob_match(path, pattern):
            continue
        if entry.get("safe") is True:
            return ("safe", [])
        crates = entry.get("crates")
        if isinstance(crates, list) and all(isinstance(c, str) for c in crates):
            return ("crates", list(crates))
        readers = entry.get("readers")
        if isinstance(readers, list) and all(isinstance(c, str) for c in readers):
            # additional-readers entry: not an ownership verdict; keep looking so
            # a later `crates`/`safe` entry can still resolve the path.
            continue
        # Matched but neither safe, a crate list, nor readers => fail-safe.
        raise SelectorError(f"ownership-map entry for {pattern!r} is neither safe, a crate list, nor readers")
    return None


def additional_readers(path: str, map_entries: list[dict]) -> list[str]:
    """[FABLE-5] sq-m4bxc: extra reader crates declared for `path` by `[[map]]`
    entries carrying a `readers` list — the additional-readers mechanism.

    These crates are UNIONED into the changed-crate set IN ADDITION to `path`'s
    normal crate-prefix owner (or its `crates` attribution), so a crate that
    #[cfg(test)] reads a SIBLING crate's committed input — where a real cargo dep
    edge would be a forbidden cycle (design §4.2) — still has its tests run when
    that input changes. The union across ALL matching `readers` entries is
    returned (sorted, deduped). This can only ENLARGE the affected set: it never
    changes a path's ownership verdict, so the selection is a strict superset of
    the selection computed WITHOUT these entries (monotone / fail-safe, §2).
    """
    out: set[str] = set()
    for entry in map_entries:
        pattern = entry.get("pattern")
        if not isinstance(pattern, str) or not _glob_match(path, pattern):
            continue
        readers = entry.get("readers")
        if isinstance(readers, list) and all(isinstance(c, str) for c in readers):
            out.update(readers)
    return sorted(out)


# --- the selection ----------------------------------------------------------
def select(
    changed_paths: list[str],
    meta: dict,
    map_entries: list[dict] | None = None,
    blob_reader=None,
    yaml_loader=None,
) -> Selection:
    """Pure core: changed paths + cargo metadata + optional map -> Selection.

    `blob_reader`/`yaml_loader` are the optional inputs to the #5237 content-based
    inert-edit carve-out (see `yaml_edit_is_inert`). Both default to None, which
    makes that carve-out a no-op — so a caller with no revision pair (the hermetic
    --changed-file path) gets exactly the pre-#5237 selection.

    Raises SelectorError on any malformed input; main() traps that to mode=full.
    """
    map_entries = map_entries or []
    ws = parse_workspace(meta)
    all_members = sorted(ws.members)
    change_class = classify_change(changed_paths)

    def full(reason: str) -> Selection:
        return Selection(
            mode="full",
            reason=reason,
            affected=all_members,  # "return ALL crates" (run everything)
            all_members=all_members,
            change_class=change_class,
        )

    changed_crates: set[str] = set()
    file_owners: list[tuple[str, str]] = []

    for path in changed_paths:
        path = path.strip()
        if not path:
            continue
        # [OPUS-4.8] ORCHESTRATION-ONLY carve-out (BEFORE the trigger check — the sole
        # rescue from a .github/scripts full-run trigger, and only for a PROVEN-inert
        # path). Treated exactly like a SAFE-listed path: contributes no crate, so a
        # pure-orchestration diff selects an empty closure and every Rust lane skips.
        if _orchestration_safe_match(path):
            file_owners.append((path, "ORCH-SAFE"))
            continue
        # [OPUS-5] sq-g25hr: the DEPLOY-manifest carve-out, same position and same
        # rationale as the orchestration one above — it is the only rescue for the
        # two `.github/workflows/deploy-*.yml` files from the `.github/` full-run
        # trigger, and it contributes no crate, so a deploy-only diff selects an
        # empty closure. `deploy/**` alone would also be rescued by the
        # `safe = true` map entry below (like `.beads/**`, which is on both), but
        # listing it here keeps the selection and the CLASS on one path list.
        if _deploy_only_match(path):
            file_owners.append((path, "DEPLOY-SAFE"))
            continue
        # [OPUS-5] #5237: the CONTENT-based inert-edit carve-out, same pre-trigger
        # position as the two path-based ones above. It is the only rescue from the
        # `.github/` full-run trigger for a YAML file in an AUDITED PARSE-ONLY tree
        # (`_YAML_PARSE_ONLY_TREES`), and it fires ONLY on a proof that this revision
        # pair's PARSED documents are identical (a pure-comment edit). Every other
        # `.github/**` YAML — `.github/workflows/**` included — falls through.
        # Like the others it contributes no crate, so a diff whose only trigger is
        # such an edit selects on the remaining paths instead of forcing full.
        if yaml_edit_is_inert(path, blob_reader, yaml_loader):
            file_owners.append((path, "YAML-INERT"))
            continue
        trig = _trigger_match(path)
        if trig is not None:
            file_owners.append((path, "FULL-TRIGGER"))
            return full(trig)

        # [FABLE-5] sq-m4bxc: additional-readers (monotone union). Extra reader
        # crates declared for this path are added REGARDLESS of ownership — the
        # union can only ENLARGE the affected set (design §4.2, fail-safe §2). It
        # never rescues an unowned/unmapped path (that still forces full below),
        # so the result stays a strict superset of the no-readers selection.
        extra = additional_readers(path, map_entries)
        for c in extra:
            if c not in ws.members:
                return full(f"ownership map declares unknown additional-reader crate {c!r} for {path}")
        changed_crates.update(extra)
        reader_tag = ("+readers:" + "+".join(extra)) if extra else ""

        owner = owning_package(path, ws.owners)
        if owner is not None:
            changed_crates.add(owner)
            file_owners.append((path, owner + reader_tag))
            continue
        verdict = apply_ownership_map(path, map_entries)
        if verdict is None:
            file_owners.append((path, "UNOWNED" + reader_tag))
            return full(f"unowned, unmapped path forces full run: {path}")
        kind, crates = verdict
        if kind == "safe":
            file_owners.append((path, "SAFE" + reader_tag))
            continue
        # kind == "crates": every mapped crate must be a real workspace member,
        # else the map is stale/invalid => fail-safe (design §4.2 validity rule).
        for c in crates:
            if c not in ws.members:
                return full(f"ownership map attributes {path} to unknown crate {c!r}")
        changed_crates.update(crates)
        file_owners.append((path, "+".join(crates) + reader_tag))

    affected: set[str] = set()
    for c in changed_crates:
        affected |= reverse_closure(c, ws.reverse_adj)
    affected &= ws.members

    if changed_crates and not affected:
        # A non-member in-repo path dep changed but nothing member-facing depends
        # on it; still surface it as changed but with an empty member set.
        reason = "changed in-repo package(s) have no member dependents"
    elif not changed_crates:
        if change_class in _INERT_CLASSES:
            reason = f"skipped-by-class: {change_class} — no Rust matrix affected"
        else:
            reason = "all changed paths are SAFE-listed or non-crate; no crate affected"
    else:
        skipped = len(ws.members) - len(affected)
        reason = f"{len(affected)} of {len(ws.members)} members affected ({skipped} skippable)"

    return Selection(
        mode="selected",
        reason=reason,
        affected=sorted(affected),
        changed_crates=sorted(changed_crates),
        file_owners=file_owners,
        all_members=all_members,
        change_class=change_class,
    )


# --- map validity (design §4.2; #1392 refinement) ---------------------------
def validate_map(
    map_entries: list[dict],
    members: set[str],
    known_generated_roots: set[str] | None = None,
    repo_root: str | None = None,
) -> list[str]:
    """Return a list of map-validity problems (empty == valid).

    Refinement from #1392: pattern-root existence is checked against a
    FETCHED/GENERATED allowlist, not a brittle on-disk check — otherwise a
    perfectly valid pattern like `tests/w3c/**` (the W3C suite is FETCHED by
    scripts/fetch-conformance.sh and gitignored) would falsely fail on a clean
    clone. The crate allowlist is likewise GENERATED from `cargo metadata`
    (`members`), so it never rots when crates move.
    """
    known_generated_roots = known_generated_roots or set()
    problems: list[str] = []
    for i, entry in enumerate(map_entries):
        pattern = entry.get("pattern")
        if not isinstance(pattern, str):
            problems.append(f"entry #{i}: missing/invalid `pattern`")
            continue
        is_safe = entry.get("safe") is True
        crates = entry.get("crates")
        readers = entry.get("readers")  # sq-m4bxc additional-readers (monotone union)
        if not is_safe and crates is None and readers is None:
            problems.append(f"{pattern!r}: none of `safe`, `crates`, `readers` set")
        for key, val, label in (("crates", crates, "crate"), ("readers", readers, "readers crate")):
            if val is None:
                continue
            if not isinstance(val, list):
                problems.append(f"{pattern!r}: `{key}` must be a list")
            else:
                for c in val:
                    if c not in members:
                        problems.append(f"{pattern!r}: unknown {label} {c!r} (not a workspace member)")
        # Pattern-root existence, tolerant of fetched/generated dirs.
        root = pattern[:-3] if pattern.endswith("/**") else pattern
        root = root.split("*", 1)[0].rstrip("/")
        if root and root not in known_generated_roots and repo_root is not None:
            on_disk = os.path.exists(os.path.join(repo_root, root))
            if not on_disk:
                problems.append(f"{pattern!r}: pattern root {root!r} does not exist and is not allowlisted")
    return problems


# --- input gathering (real, non-hermetic) -----------------------------------
def _run(cmd: list[str], cwd: str | None = None) -> str:
    try:
        out = subprocess.run(
            cmd, cwd=cwd, check=True, capture_output=True, text=True, timeout=180
        )
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError) as exc:
        raise SelectorError(f"command failed: {' '.join(cmd)}: {exc}") from exc
    return out.stdout


def git_changed_paths(base: str, head: str, repo_root: str | None) -> list[str]:
    # Three-dot = diff against the merge-base; --no-renames reports a move as
    # delete+add so BOTH crate dirs are attributed (design §3.1).
    out = _run(
        ["git", "diff", "--name-only", "--no-renames", f"{base}...{head}"],
        cwd=repo_root,
    )
    return [ln for ln in out.splitlines() if ln.strip()]


def load_metadata(metadata_file: str | None, repo_root: str | None) -> dict:
    if metadata_file:
        with open(metadata_file, encoding="utf-8") as fh:
            return json.load(fh)
    out = _run(["cargo", "metadata", "--format-version", "1", "--locked"], cwd=repo_root)
    return json.loads(out)


def load_ownership_map(map_file: str | None) -> list[dict]:
    """Read ci/path-ownership.toml if present. Absent => []. Malformed => error."""
    if not map_file or not os.path.exists(map_file):
        return []
    try:
        import tomllib  # stdlib >= 3.11
    except ModuleNotFoundError as exc:  # pragma: no cover - CI is 3.12
        raise SelectorError("tomllib unavailable (needs Python >= 3.11) but a map file exists") from exc
    try:
        with open(map_file, "rb") as fh:
            data = tomllib.load(fh)
    except (tomllib.TOMLDecodeError, OSError) as exc:
        raise SelectorError(f"malformed ownership map {map_file}: {exc}") from exc
    entries = data.get("map", [])
    if not isinstance(entries, list):
        raise SelectorError("ownership map: [[map]] must be an array of tables")
    return entries


# --- shadow mode (design §6.4; wired by sq-fmx4u.3, flipped by sq-fmx4u.5) ---
def shadow_wrap(sel: Selection) -> Selection:
    """[FABLE-5] Report-only rollout mode: keep the computed selection for the
    step summary but emit mode="shadow", which every downstream guard treats
    exactly like any non-"selected" mode — RUN EVERYTHING. The wrap is applied
    UNIFORMLY (even over a computed mode=full / the error path) so the emitted
    contract is one rule: --shadow => mode is never "selected" => nothing skips.
    """
    return Selection(
        mode="shadow",
        reason=f"SHADOW (computed mode={sel.mode}; report-only, nothing is skipped): {sel.reason}",
        affected=sel.affected,
        changed_crates=sel.changed_crates,
        file_owners=sel.file_owners,
        all_members=sel.all_members,
        change_class=sel.change_class,
    )


def filterset(sel: Selection) -> str:
    """[FABLE-5] The nextest filterset over the affected members, e.g.
    "package(a) + package(b)". Consumed by the cross-crate bulk test shards to
    NARROW (never skip) their partition when mode == "selected" (design §5.2).
    nextest fails loud on a package() naming no package in the archive, so a
    stale name here cannot silently select nothing."""
    return " + ".join(f"package({m})" for m in sel.affected)


# --- step summary + outputs --------------------------------------------------
def render_summary(sel: Selection) -> str:
    lines = ["### CI test-selection", "", f"**Mode:** `{sel.mode}` — {sel.reason}", ""]
    # [OPUS-4.8] Explicit change-class attribution line so the audit trail shows WHY
    # the Rust lanes were skipped (or not). No silent skips.
    lines.append(f"**Change-class:** `{sel.change_class}`")
    if sel.change_class in _INERT_CLASSES and sel.mode == "selected":
        lines.append(
            f"> skipped-by-class: `{sel.change_class}` — every changed path is proven "
            "inert for the Rust matrix, so the engine test/clippy/coverage/bench/fuzz/"
            "opt-in-feature lanes are skipped for this PR."
        )
    lines.append("")
    if sel.mode == "shadow":
        lines.append(
            "**SHADOW MODE** — selection is REPORT-ONLY on this run: every job still "
            "runs; the closure below is what enforcement WOULD run (flip: sq-fmx4u.5)."
        )
        lines.append("")
    if sel.file_owners:
        lines += ["| Changed file | Owning crate |", "|---|---|"]
        for path, owner in sel.file_owners:
            lines.append(f"| `{path}` | {owner} |")
        lines.append("")
    if sel.mode in ("selected", "shadow"):
        total = len(sel.all_members)
        run = len(sel.affected)
        lines.append(f"**Affected closure ({run} of {total}):** " + (", ".join(sel.affected) or "_none_"))
        lines.append("")
        lines.append(f"**Skipped by selection:** {total - run} of {total} members")
    else:
        lines.append("**Full matrix:** every crate runs.")
    return "\n".join(lines) + "\n"


def _write_outputs(sel: Selection, output_file: str | None, summary_file: str | None) -> None:
    if output_file:
        with open(output_file, "a", encoding="utf-8") as fh:
            fh.write(f"mode={sel.mode}\n")
            fh.write("affected=" + json.dumps(sel.affected) + "\n")
            fh.write("filterset=" + filterset(sel) + "\n")
            # [OPUS-4.8] change-class output for the audit trail (consumers may
            # surface "skipped-by-class: <class>"); never a gating input.
            fh.write(f"change_class={sel.change_class}\n")
    if summary_file:
        with open(summary_file, "a", encoding="utf-8") as fh:
            fh.write(render_summary(sel))


# --- classify-only mode (merge-group change-class gating; #3420/#3421 follow-up)
def _classify_only_main(args: argparse.Namespace, output_file: str | None,
                        summary_file: str | None) -> int:
    """[FABLE-5] merge-group change-class: compute ONLY `classify_change` over the
    diff — no cargo metadata, no toolchain, no ownership map — so the cheap
    workflow `changes` pre-jobs (ci.yml / feature-matrix.yml / codeql.yml) can
    class-gate their `rust_changed` output on the MERGE-GROUP batch diff without
    duplicating the orchestration-safe / docs-only / deploy-only path lists (this
    file stays the single source of truth). Contract with the workflow step:

      * stdout is EXACTLY ONE line: the class token
        (engine|orchestration-only|docs-only|deploy-only|inert-mixed|mixed) — the
        shell consumer `case`s on the _INERT_CLASSES tokens and treats ANY other
        value, including an unrecognised one, as engine (run everything);
      * `change_class=<class>` is appended to --output-file/$GITHUB_OUTPUT and a
        one-line attribution to the step summary (audit trail, never gating);
      * FAIL-SAFE (design §4.3, the #3421 posture): --full, any event other than
        pull_request/merge_group, a missing base, an unresolvable diff, or ANY
        internal error => class `engine`, exit 0 — the consumer then runs the
        full matrix (cost, never soundness).
    """
    change_class = _CLASS_ENGINE
    reason = ""
    try:
        if args.full:
            reason = "forced full run (ci-full override) => class engine"
        elif args.event not in ("pull_request", "merge_group"):
            reason = f"{args.event} event: no PR/batch diff => class engine"
        else:
            if args.changed_file:
                with open(args.changed_file, encoding="utf-8") as fh:
                    changed = [ln for ln in fh.read().splitlines() if ln.strip()]
            else:
                if not args.base:
                    raise SelectorError("--base is required for a diff-based classify")
                repo_root = _resolve_repo_root(args.repo_root)
                changed = git_changed_paths(args.base, args.head, repo_root)
            change_class = classify_change(changed)
            reason = f"classified {len(changed)} changed path(s)"
    except Exception as exc:  # fail-safe boundary: ANY error => engine (full run)
        change_class = _CLASS_ENGINE
        reason = f"classifier error, failing safe to class engine (full run): {exc}"

    print(change_class)
    try:
        if output_file:
            with open(output_file, "a", encoding="utf-8") as fh:
                fh.write(f"change_class={change_class}\n")
        if summary_file:
            with open(summary_file, "a", encoding="utf-8") as fh:
                fh.write(f"**Change-class (classify-only):** `{change_class}` — {reason}\n")
    except OSError:
        pass  # never let an output/summary write break the fail-safe contract
    return 0


# --- main --------------------------------------------------------------------
def _parse_args(argv: list[str] | None) -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Change-based CI test-selection (design §3).")
    p.add_argument("--event", default="pull_request",
                   help="GitHub event; anything but pull_request/merge_group => full (no PR diff)")
    p.add_argument("--full", action="store_true", help="force mode=full (e.g. the ci-full label)")
    p.add_argument("--shadow", action="store_true",
                   help="report-only rollout mode (design §6.4): compute + report the "
                        "selection but emit mode=shadow so no downstream guard skips")
    p.add_argument("--classify-only", action="store_true",
                   help="emit ONLY the change-class token (no cargo metadata / selection): "
                        "the cheap merge-group class gate for the workflow `changes` "
                        "pre-jobs; fail-safe to `engine` on any error")
    p.add_argument("--base", help="base SHA/ref for the three-dot diff")
    p.add_argument("--head", default="HEAD", help="head SHA/ref (default HEAD)")
    p.add_argument("--changed-file", help="hermetic: newline-delimited changed paths")
    p.add_argument("--metadata-file", help="hermetic: cargo metadata JSON snapshot")
    p.add_argument("--map", dest="map_file", help="ownership map toml (default ci/path-ownership.toml)")
    p.add_argument("--repo-root", help="repo root (default: git toplevel / workspace_root)")
    p.add_argument("--summary-file", help="write markdown step-summary (default $GITHUB_STEP_SUMMARY)")
    p.add_argument("--output-file", help="write mode/affected outputs (default $GITHUB_OUTPUT)")
    return p.parse_args(argv)


def _resolve_repo_root(explicit: str | None) -> str | None:
    if explicit:
        return explicit
    try:
        return _run(["git", "rev-parse", "--show-toplevel"]).strip()
    except SelectorError:
        return None


def main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv)
    summary_file = args.summary_file or os.environ.get("GITHUB_STEP_SUMMARY")
    output_file = args.output_file or os.environ.get("GITHUB_OUTPUT")

    # [FABLE-5] merge-group change-class gating: the cheap classify-only entry
    # point (no cargo metadata). Everything below is the full selector.
    if args.classify_only:
        return _classify_only_main(args, output_file, summary_file)

    try:
        repo_root = _resolve_repo_root(args.repo_root)

        # Events with no PR diff, or the explicit override => full (design §3.1, §6).
        # [FABLE-5] sq-fmx4u.3: only pull_request and merge_group carry a sound
        # (base, head) revision pair; push/schedule/workflow_dispatch (and any
        # future event) get the full matrix by construction, not by error-trap.
        if args.full or args.event not in ("pull_request", "merge_group"):
            reason = "forced full run (ci-full override)" if args.full else f"{args.event} event: no PR diff"
            meta = load_metadata(args.metadata_file, repo_root)
            ws = parse_workspace(meta)
            sel = Selection(mode="full", reason=reason, affected=sorted(ws.members),
                            all_members=sorted(ws.members))
        else:
            if args.changed_file:
                with open(args.changed_file, encoding="utf-8") as fh:
                    changed = [ln for ln in fh.read().splitlines() if ln.strip()]
            else:
                if not args.base:
                    raise SelectorError("--base is required for a diff-based run")
                changed = git_changed_paths(args.base, args.head, repo_root)

            # [OPUS-5] #5237: the content-based inert-edit carve-out needs BOTH blob
            # versions of a changed `.github/**` YAML file. Absent a resolvable
            # revision pair this is None and the carve-out is a no-op (=> full on
            # any such path, the pre-#5237 behaviour).
            blob_reader = git_blob_reader(args.base, args.head, repo_root)

            map_file = args.map_file
            if map_file is None and repo_root is not None:
                candidate = os.path.join(repo_root, "ci", "path-ownership.toml")
                map_file = candidate if os.path.exists(candidate) else None
            map_entries = load_ownership_map(map_file)

            meta = load_metadata(args.metadata_file, repo_root)
            sel = select(changed, meta, map_entries, blob_reader=blob_reader)

    except Exception as exc:  # fail-closed boundary: ANY error => full run (design §4.3)
        # We could not even build the member list; emit full with an empty
        # affected is still "run everything" downstream (mode != 'selected').
        sel = Selection(mode="full", reason=f"selector error, failing to full run: {exc}", affected=[])

    if args.shadow:  # [FABLE-5] report-only rollout (design §6.4; flip: sq-fmx4u.5)
        sel = shadow_wrap(sel)

    print(json.dumps(sel.to_json_obj(), indent=2))
    try:
        _write_outputs(sel, output_file, summary_file)
    except OSError:
        pass  # never let a summary/output write turn a full run into a failure
    return 0


if __name__ == "__main__":
    sys.exit(main())
