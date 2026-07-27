#!/usr/bin/env python3
# [OPUS-5] sq-toze.25 (archives) + #4570 (every other file-shaped artifact) / GX-11 —
# the SLSA **Build L3** isolation contract for sparq's released artifacts.
#
# WHY THIS TEST EXISTS. `release.yml` runs only on `v*` tags and `workflow_dispatch`, and
# `dist.yml` only on `workflow_dispatch`, so NOTHING in these pipelines registers on a PR: the
# first time an L3 lane executes is the moment a release is being cut, when a mistake is expensive
# and half-irreversible. Worse, the failure mode here is SILENT — the compliance estate
# (compliance/slsa/controls.md SL-B3-b, both gap registers) will go on publishing an L3 claim long
# after someone "simplified" a trusted-builder call back into an in-band
# `actions/attest-build-provenance` step. A green release with a false level claim is the exact
# overclaim compliance/ exists to prevent, so the workflow SHAPE is pinned here, structurally, on
# every PR.
#
# WHAT L3 ACTUALLY REQUIRES (and therefore what is asserted): the provenance must be produced by a
# trusted control plane the user build steps cannot influence. Concretely that means all of —
#   * the signing runs in a SEPARATE job that is a bare `uses:` of the trusted reusable builder
#     (no `steps:`/`run:` of ours can execute inside it);
#   * that builder is referenced by an immutable SEMVER TAG, because the builder identity baked
#     into the provenance — the thing `slsa-verifier` matches a consumer policy against — is
#     derived from the `@ref`. A commit-SHA reference yields no verifiable builder identity, so
#     the repo-wide SHA-pin convention is deliberately NOT applied here;
#   * the ONLY thing crossing the boundary is `base64-subjects`, THREADED from a digest-collecting
#     job's output — never recomputed inside the trusted builder from build artifacts;
#   * the trusted-builder job holds `id-token: write` (its own signing identity) but NOT
#     `contents: write` (it is not the uploader — the `release` job is);
#   * the digest-collecting job neither builds nor signs;
#   * and the whole thing is FAIL-CLOSED: `release` `needs:` both provenance jobs, and attaches
#     both signed bundles to the Release before SHA256SUMS is computed over them.
#
# THE THREE LANES pinned here, and what each covers:
#   1. `release.yml#provenance`            — the CLI/server archives   (subjects from `package`)
#   2. `release.yml#provenance-artifacts`  — GUI bundles, SBOM/VEX,
#                                            served-surface conformance report
#   3. `dist.yml#provenance`               — the tiered bare binaries  (subjects from `build`)
# The ghcr.io container image is deliberately NOT here: it needs the generator's separate
# `generator_container_slsa3.yml` (image-digest subjects), which is its own migration. GX-11
# stays open for it — do not add a container assertion until that lane actually exists.
#
# NON-VACUITY. Structural assertions rot into decoration the moment they stop being able to fail
# (this repo has shipped exactly that: see test_release_container_multiarch.py's header, and the
# sq-w1dxx note that release.yml's container contract was pinned by a test nothing ran). So every
# assertion above is driven by MUTATIONS in `MUTATIONS` below — each one edits the workflow text
# into a plausible regression and this suite fails if the checker still passes. The in-band
# regression (M1) and the SHA-pin regression (M2) are the two that matter most: both look like
# housekeeping in review and both silently drop the level back to L2.
#
# Pure stdlib on purpose (no PyYAML): this must be runnable in any checkout, including ones where
# the docs-quality shared setup has not installed it.
#
# Run: python3 scripts/tests/test_release_slsa_l3_provenance.py

from __future__ import annotations

import re
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
RELEASE = REPO_ROOT / ".github" / "workflows" / "release.yml"
BUILD_MATRIX = REPO_ROOT / ".github" / "workflows" / "build-matrix.yml"
DIST = REPO_ROOT / ".github" / "workflows" / "dist.yml"

# The trusted reusable builder, referenced by an immutable semver tag. `@<40-hex>` is REJECTED —
# see the header: a SHA reference produces provenance with no resolvable builder identity.
TRUSTED_BUILDER = re.compile(
    r"^slsa-framework/slsa-github-generator/\.github/workflows/"
    r"generator_generic_slsa3\.yml@v\d+\.\d+\.\d+$"
)
SUBJECTS_EXPR = "${{ needs.package.outputs.hashes }}"
ARTIFACT_SUBJECTS_EXPR = "${{ needs.artifact-subjects.outputs.hashes }}"
DIST_SUBJECTS_EXPR = "${{ needs.build.outputs.binary-hashes }}"
PROVENANCE_ARTIFACT_EXPR = "${{ needs.provenance.outputs.provenance-download-name }}"
ARTIFACTS_PROVENANCE_EXPR = (
    "${{ needs.provenance-artifacts.outputs.provenance-download-name }}"
)
HASHES_OUTPUT_VALUE = "${{ jobs.hashes.outputs.hashes }}"
BINARY_HASHES_OUTPUT_VALUE = "${{ jobs.binary-hashes.outputs.hashes }}"
SHA256SUMS_CMD = "sha256sum -- * > SHA256SUMS"
# The per-tier digest steps run on EVERY matrix row, and macOS has no GNU `sha256sum` — only
# BSD `shasum`. See the assertion in `check` for why a bare `sha256sum` there fails closed at
# release time rather than on any PR.
DIGEST_STEP = "Record archive digests (subjects for the isolated provenance builder)"
BINARY_DIGEST_STEP = "Record dist binary digests (subjects for the isolated provenance builder)"
GUI_DIGEST_STEP = "Record GUI bundle digests (subjects for the isolated provenance builder)"
SBOM_DIGEST_STEP = "Record SBOM/VEX digests (subjects for the isolated provenance builder)"
CONFORMANCE_DIGEST_STEP = (
    "Record conformance-report digests (subjects for the isolated provenance builder)"
)
PORTABLE_DIGEST = "shasum -a 256"
MAC_ROW = re.compile(r"os:\s*macos-")
# The GUI digest list must be written OUTSIDE `gui-bundles/`, which is uploaded wholesale as
# `pkg-gui-*` and lands verbatim in the published release assets.
GUI_SUBJECTS_DIR = "../gui-subjects/"
# The collector requires each singleton lane BY NAME. A subjects blob that silently lost a lane
# still signs cleanly over a smaller set while the release notes claim the lane is covered.
REQUIRED_LANES_LINE = (
    "for required in subject-hashes-sbom.txt subject-hashes-conformance.txt; do"
)

# A line that is a YAML key (optionally a sequence item) — the only shape we strip a trailing
# `# comment` from, so block-scalar prose (the release body) is never truncated at a `#`.
KEY_LINE = re.compile(r"^\s*(?:-\s+)?[A-Za-z_][\w.-]*:")


# --------------------------------------------------------------------------------------
# A deliberately tiny, indentation-based reader. Workflow jobs sit at exactly two spaces
# under `jobs:`, which is all the structure these assertions need — and being stdlib means
# the mutation table can run anywhere.
# --------------------------------------------------------------------------------------
def job_block(text: str, job: str) -> list[str] | None:
    """Return the (comment-stripped) body lines of top-level job `job`, or None if absent."""
    lines = text.splitlines()
    start = None
    for i, line in enumerate(lines):
        if re.match(rf"^  {re.escape(job)}:\s*$", line):
            start = i + 1
            break
    if start is None:
        return None
    body: list[str] = []
    for line in lines[start:]:
        # Any non-blank line indented two spaces or less starts the next job (or ends `jobs:`).
        if line.strip() and not line.startswith("   "):
            break
        body.append(line)
    return strip_comments(body)


def strip_comments(lines: list[str]) -> list[str]:
    out: list[str] = []
    for line in lines:
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        if KEY_LINE.match(line) and " #" in line:
            line = line.split(" #", 1)[0].rstrip()
        out.append(line)
    return out


def scalar(body: list[str], key: str) -> str | None:
    """Value of the first `key: value` line at any depth."""
    pat = re.compile(rf"^\s*(?:-\s+)?{re.escape(key)}:\s*(.+?)\s*$")
    for line in body:
        m = pat.match(line)
        if m:
            return m.group(1)
    return None


def has_line(body: list[str], key: str, value: str) -> bool:
    return scalar_all(body, key).count(value) > 0


def scalar_all(body: list[str], key: str) -> list[str]:
    pat = re.compile(rf"^\s*(?:-\s+)?{re.escape(key)}:\s*(.+?)\s*$")
    return [m.group(1) for m in (pat.match(line) for line in body) if m]


def needs_set(body: list[str]) -> set[str]:
    """The job ids in `needs:`, as an EXACT set.

    Substring matching is not good enough here: `provenance` is a prefix of
    `provenance-artifacts`, so a naive `"provenance" in needs` stays true after the archives
    lane has been dropped from the dependency list — the assertion would go vacuous exactly
    where it matters. Only the inline-flow form (`needs: a` / `needs: [a, b]`) is understood,
    which is the form both workflows use; a block sequence would yield an empty set and red
    loudly rather than silently pass.
    """
    raw = scalar(body, "needs")
    if raw is None:
        return set()
    raw = raw.strip()
    if raw.startswith("[") and raw.endswith("]"):
        raw = raw[1:-1]
    return {part.strip() for part in raw.split(",") if part.strip()}


def index_of(body: list[str], needle: str) -> int:
    for i, line in enumerate(body):
        if needle in line:
            return i
    return -1


def step_block(body: list[str], name: str) -> list[str] | None:
    """Lines of the first `- name: <name>` step, up to the next step at the same indent."""
    start = None
    indent = 0
    for i, line in enumerate(body):
        m = re.match(r"^(\s*)-\s+name:\s*(.+?)\s*$", line)
        if m and m.group(2) == name:
            start, indent = i + 1, len(m.group(1))
            break
    if start is None:
        return None
    out: list[str] = []
    for line in body[start:]:
        if re.match(rf"^\s{{0,{indent}}}-\s", line):
            break
        out.append(line)
    return out


# --------------------------------------------------------------------------------------
# The isolation properties every trusted-builder lane must satisfy, applied identically to
# all three. Written once so a new lane cannot be added with a weaker contract than the
# archives lane it copies.
# --------------------------------------------------------------------------------------
def check_trusted_builder(
    body: list[str] | None,
    where: str,
    subjects_expr: str,
    deps: tuple[str, ...],
) -> list[str]:
    bad: list[str] = []
    if body is None:
        return [f"{where} is missing — an isolated trusted-builder lane is gone"]

    used = scalar(body, "uses")
    if used is None or not TRUSTED_BUILDER.match(used):
        bad.append(
            f"{where} must be a bare call to the slsa-github-generator generic builder "
            f"pinned to a semver TAG (not a SHA); found: {used!r}"
        )
    if any(re.match(r"^\s*steps:\s*$", line) for line in body) or scalar_all(body, "run"):
        bad.append(
            f"{where} declares steps/run — anything we execute inside the trusted "
            "builder call destroys the isolation the L3 claim rests on"
        )

    subjects = scalar(body, "base64-subjects")
    if subjects != subjects_expr:
        bad.append(
            f"{where} `base64-subjects` must be threaded from the digest-collecting job "
            f"({subjects_expr}); found: {subjects!r}"
        )
    needs = needs_set(body)
    for dep in deps:
        if dep not in needs:
            bad.append(f"{where} must `needs: {dep}`; found: {sorted(needs)!r}")

    if "write" not in scalar_all(body, "id-token"):
        bad.append(f"{where} needs `id-token: write` to mint its own signing identity")
    if "read" not in scalar_all(body, "actions"):
        bad.append(f"{where} needs `actions: read` to record the workflow entry point")
    if "write" in scalar_all(body, "contents"):
        bad.append(
            f"{where} must NOT hold `contents: write` — it is not the uploader "
            "(`upload-assets: false`)"
        )
    if scalar(body, "upload-assets") != "false":
        bad.append(
            f"{where} must set `upload-assets: false` — the generator's own uploader "
            "targets github.ref, which is a BRANCH on workflow_dispatch"
        )
    return bad


def check_collector(
    body: list[str] | None, where: str, deps: tuple[str, ...]
) -> list[str]:
    """A digest-collecting job must neither build nor hold a signing identity."""
    bad: list[str] = []
    if body is None:
        return [f"{where} is missing — the digest hand-off is gone"]
    needs = needs_set(body)
    for dep in deps:
        if dep not in needs:
            bad.append(f"{where} must `needs: {dep}`; found: {sorted(needs)!r}")
    if any("cargo " in line for line in body):
        bad.append(f"{where} must not build anything — it only republishes digests")
    if any("id-token" in line for line in body):
        bad.append(
            f"{where} must not hold a signing identity — signing belongs to the "
            "isolated trusted builder alone"
        )
    return bad


# --------------------------------------------------------------------------------------
# THE CONTRACT. Returns a list of human-readable failures; empty list == contract holds.
# Written as a pure function of the three workflow texts so MUTATIONS can drive it.
# --------------------------------------------------------------------------------------
def check(release_text: str, matrix_text: str, dist_text: str) -> list[str]:
    bad: list[str] = []

    prov = job_block(release_text, "provenance")
    prov_artifacts = job_block(release_text, "provenance-artifacts")
    rel = job_block(release_text, "release")
    pkg = job_block(release_text, "package")

    if prov is None:
        return ["release.yml has no `provenance` job — the L3 trusted-builder lane is gone"]
    if rel is None:
        return ["release.yml has no `release` job"]

    # --- lane 1: the release archives (sq-toze.25).
    bad += check_trusted_builder(
        prov, "release.yml#provenance", SUBJECTS_EXPR, ("setup", "package")
    )
    # --- lane 2: GUI bundles + SBOM/VEX + conformance report (#4570).
    bad += check_trusted_builder(
        prov_artifacts,
        "release.yml#provenance-artifacts",
        ARTIFACT_SUBJECTS_EXPR,
        ("setup", "artifact-subjects"),
    )
    bad += check_collector(
        job_block(release_text, "artifact-subjects"),
        "release.yml#artifact-subjects",
        ("sbom", "served-conformance", "gui-bundle"),
    )
    # --- lane 3: dist.yml's tiered bare binaries (#4570).
    bad += check_trusted_builder(
        job_block(dist_text, "provenance"), "dist.yml#provenance", DIST_SUBJECTS_EXPR, ("build",)
    )

    # --- fail-closed: no Release without BOTH isolated bundles, and both ship in SHA256SUMS.
    rel_needs = needs_set(rel)
    for dep in ("provenance", "provenance-artifacts"):
        if dep not in rel_needs:
            bad.append(
                f"the `release` job must `needs: {dep}` — otherwise a release can ship while "
                f"the docs claim L3 coverage; found: {sorted(rel_needs)!r}"
            )
    sums = index_of(rel, SHA256SUMS_CMD)
    for label, expr in (
        ("archives", PROVENANCE_ARTIFACT_EXPR),
        ("non-archive artifacts", ARTIFACTS_PROVENANCE_EXPR),
    ):
        dl = index_of(rel, expr)
        if dl < 0:
            bad.append(
                f"the `release` job must download the signed {label} provenance artifact by the "
                f"generator's own output name ({expr})"
            )
        elif sums < 0 or dl > sums:
            bad.append(f"the {label} provenance download must precede the SHA256SUMS step")

    # --- the build job is NOT the signer: it must still exist and must hand over digests only.
    if pkg is None:
        bad.append("release.yml has no `package` job")

    # --- the collector must require each singleton lane BY NAME. A blob that silently lost the
    # SBOM or the conformance report still signs cleanly over a smaller set, and the release
    # notes would go on claiming the lane is covered — the vacuous-artifact class.
    subjects_job = job_block(release_text, "artifact-subjects")
    if subjects_job is not None:
        combine = step_block(subjects_job, "Combine + base64-encode the subjects")
        if combine is None:
            bad.append("release.yml#artifact-subjects has no `Combine + base64-encode` step")
        else:
            if not any(REQUIRED_LANES_LINE in line for line in combine):
                bad.append(
                    "release.yml#artifact-subjects must require BOTH singleton lanes by name "
                    f"(`{REQUIRED_LANES_LINE}`) — a missing lane must fail closed, not shrink "
                    "the subjects set silently"
                )
            if not any("subject-hashes-gui-*.txt" in line for line in combine):
                bad.append(
                    "release.yml#artifact-subjects must require at least one "
                    "`subject-hashes-gui-*.txt` — the GUI bundles are release assets"
                )

    # --- release.yml: each non-archive lane publishes its digests as a `subject-hashes-*` artifact.
    for job, step_name, prefix in (
        ("sbom", SBOM_DIGEST_STEP, "subject-hashes-sbom"),
        ("served-conformance", CONFORMANCE_DIGEST_STEP, "subject-hashes-conformance"),
        ("gui-bundle", GUI_DIGEST_STEP, "subject-hashes-gui-"),
    ):
        body = job_block(release_text, job)
        if body is None:
            bad.append(f"release.yml has no `{job}` job")
            continue
        if step_block(body, step_name) is None:
            bad.append(
                f"release.yml#{job} has no `{step_name}` step — that lane's subjects hand-off "
                "is gone and its provenance silently drops back to in-band L2"
            )
        if not any(n.startswith(prefix) for n in scalar_all(body, "name")):
            bad.append(
                f"release.yml#{job} must upload its digest list as a `{prefix}*` artifact "
                "(the collector's download pattern)"
            )

    # --- the GUI digest list must NOT be written into `gui-bundles/`: that directory is uploaded
    # wholesale as `pkg-gui-*`, which the `release` job downloads with `pattern: pkg-*` straight
    # into the PUBLISHED assets — a stray digest list would ship as a release asset.
    gui = job_block(release_text, "gui-bundle")
    if gui is not None:
        gui_digest = step_block(gui, GUI_DIGEST_STEP)
        if gui_digest is not None:
            if not any(GUI_SUBJECTS_DIR in line for line in gui_digest):
                bad.append(
                    "the GUI digest list must be written outside `gui-bundles/` (i.e. into "
                    f"`{GUI_SUBJECTS_DIR}`) — that directory is published verbatim as release assets"
                )
            # macOS + Windows rows: no GNU coreutils on Darwin.
            if any(MAC_ROW.search(line) for line in gui) and not any(
                PORTABLE_DIGEST in line for line in gui_digest
            ):
                bad.append(
                    "the GUI bundle matrix includes macOS rows, so its digest step must use a "
                    f"portable digest (`{PORTABLE_DIGEST}`) — macOS has no GNU `sha256sum`"
                )

    # --- build-matrix.yml: the digest hand-off, once per mode.
    for value, mode in (
        (HASHES_OUTPUT_VALUE, "archive"),
        (BINARY_HASHES_OUTPUT_VALUE, "binary"),
    ):
        if value not in matrix_text:
            bad.append(
                f"build-matrix.yml must expose the {mode}-mode subjects as a workflow_call "
                f"output ({value})"
            )
    hashes = job_block(matrix_text, "hashes")
    binary_hashes = job_block(matrix_text, "binary-hashes")
    build = job_block(matrix_text, "build")
    if hashes is None:
        bad.append("build-matrix.yml has no `hashes` job to collect the per-tier digests")
    else:
        if scalar(hashes, "if") != "inputs.mode == 'archive'":
            bad.append("the `hashes` job must be archive-mode only")
        bad += check_collector(hashes, "build-matrix.yml#hashes", ("build",))
    if binary_hashes is None:
        bad.append(
            "build-matrix.yml has no `binary-hashes` job — dist.yml's tiered binaries lose "
            "their isolated-builder subjects and fall back to in-band L2"
        )
    else:
        if scalar(binary_hashes, "if") != "inputs.mode == 'binary'":
            bad.append("the `binary-hashes` job must be binary-mode only")
        bad += check_collector(binary_hashes, "build-matrix.yml#binary-hashes", ("build",))
    if build is None:
        bad.append("build-matrix.yml has no `build` job")
    else:
        names = scalar_all(build, "name")
        for prefix, published in (("archive-hashes-", "pkg-"), ("binary-hashes-", "sparq-cli-")):
            digest_artifacts = [n for n in names if n.startswith(prefix)]
            if not digest_artifacts:
                bad.append(
                    f"the build matrix must upload a per-tier `{prefix}<tier>` artifact"
                )
            if any(n.startswith(published) for n in digest_artifacts):
                bad.append(
                    f"the digest list must not be uploaded under a `{published}*` name — that "
                    "is the name of the SHIPPED payload artifact"
                )
        # --- the digest steps must be able to RUN on every row they are scheduled on. macOS
        # runners ship BSD `shasum` and no GNU coreutils, so a bare `sha256sum` reds both
        # Darwin tiers; they then upload no digest artifact, the collector job that
        # `needs: build` never runs, and the release + provenance jobs never run either. That
        # is a release-time-only failure on a lane no PR exercises — exactly what this suite
        # exists to catch — so require the portable branch for as long as a mac row exists.
        for step_name in (DIGEST_STEP, BINARY_DIGEST_STEP):
            digest = step_block(build, step_name)
            if digest is None:
                bad.append(
                    f"build-matrix.yml has no `{step_name}` step — that subjects hand-off is gone"
                )
            elif any(MAC_ROW.search(line) for line in build) and not any(
                PORTABLE_DIGEST in line for line in digest
            ):
                bad.append(
                    "the build matrix includes macOS rows, so "
                    f"`{step_name}` must use a portable digest (`{PORTABLE_DIGEST}`) — "
                    "macOS has no GNU `sha256sum`"
                )
    return bad


# --------------------------------------------------------------------------------------
# Mutations: each must make `check` fail. A mutation that no longer applies (the anchor text
# vanished) is itself a failure — it means the contract moved and this table stopped testing it.
# --------------------------------------------------------------------------------------
def _sub(file: str, old: str, new: str):
    def apply(release_text: str, matrix_text: str, dist_text: str):
        texts = {"release": release_text, "matrix": matrix_text, "dist": dist_text}
        if old not in texts[file]:
            raise AssertionError(f"mutation anchor not found in {file}: {old!r}")
        texts[file] = texts[file].replace(old, new, 1)
        return texts["release"], texts["matrix"], texts["dist"]

    return apply


MUTATIONS = {
    # M1 — THE regression this test is really for: quietly go back to in-band attestation.
    # (Anchors the FIRST `uses:` in release.yml, i.e. the archives lane.)
    "in-band attest replaces the trusted builder": _sub(
        "release",
        "uses: slsa-framework/slsa-github-generator/.github/workflows/generator_generic_slsa3.yml@v2.1.0",
        "uses: actions/attest-build-provenance@0f67c3f4856b2e3261c31976d6725780e5e4c373",
    ),
    # M2 — "hardening" the generator to a SHA, which destroys the verifiable builder identity.
    "trusted builder pinned by SHA instead of tag": _sub(
        "release",
        "generator_generic_slsa3.yml@v2.1.0",
        "generator_generic_slsa3.yml@" + "0" * 40,
    ),
    # M3 — the release stops depending on provenance: releases ship, the L3 claim goes stale.
    "release no longer needs the provenance job": _sub(
        "release",
        "needs: [setup, package, provenance, provenance-artifacts, sbom, gui-bundle, served-conformance]",
        "needs: [setup, package, provenance-artifacts, sbom, gui-bundle, served-conformance]",
    ),
    # M4 — subjects recomputed inside the builder instead of threaded across the boundary.
    "subjects no longer threaded from the build matrix": _sub(
        "release",
        "base64-subjects: ${{ needs.package.outputs.hashes }}",
        "base64-subjects: ${{ steps.hash.outputs.hashes }}",
    ),
    # M5 — privilege creep: the trusted builder gains repo write.
    "trusted builder granted contents: write": _sub(
        "release",
        "contents: read # `upload-assets: false`",
        "contents: write # `upload-assets: false`",
    ),
    # M6 — the digest hand-off is severed; the builder would attest nothing.
    "build-matrix drops the hashes output": _sub(
        "matrix",
        "value: ${{ jobs.hashes.outputs.hashes }}",
        'value: ""',
    ),
    # M7 — a build step creeps into the digest-collecting job.
    "a cargo build creeps into the hashes job": _sub(
        "matrix",
        '          echo "subjects for the isolated provenance builder:"',
        '          cargo auditable build --release\n'
        '          echo "subjects for the isolated provenance builder:"',
    ),
    # M9 — "simplifying" the portable digest back to a bare GNU `sha256sum`, which is absent on
    # both Darwin rows. Reads as noise-removal in review; fails only when a release is cut.
    "archive digests use GNU-only sha256sum on the mac rows": _sub(
        "matrix",
        '          if command -v sha256sum >/dev/null 2>&1; then\n'
        '            sha256sum -- "${archives[@]}" > "archive-hashes-${{ matrix.tier }}.txt"\n'
        '          else\n'
        '            shasum -a 256 -- "${archives[@]}" > "archive-hashes-${{ matrix.tier }}.txt"\n'
        '          fi',
        '          sha256sum -- "${archives[@]}" > "archive-hashes-${{ matrix.tier }}.txt"',
    ),
    # M8 — the release stops attaching the signed bundle at all.
    "signed provenance never attached to the release": _sub(
        "release",
        "name: ${{ needs.provenance.outputs.provenance-download-name }}",
        "name: sbom-vex",
    ),
    # ---- #4570: the same regressions, one per newly-isolated lane. ----
    # MA1 — the non-archive bundle's subjects stop being threaded across the boundary.
    "artifact subjects no longer threaded from the collector": _sub(
        "release",
        "base64-subjects: ${{ needs.artifact-subjects.outputs.hashes }}",
        "base64-subjects: ${{ steps.hash.outputs.hashes }}",
    ),
    # MA2 — the release stops depending on the non-archive bundle: GUI/SBOM/conformance quietly
    # revert to in-band-only coverage while the release notes still advertise both bundles.
    "release no longer needs the provenance-artifacts job": _sub(
        "release",
        "needs: [setup, package, provenance, provenance-artifacts, sbom, gui-bundle, served-conformance]",
        "needs: [setup, package, provenance, sbom, gui-bundle, served-conformance]",
    ),
    # MA3 — the digest collector gains a signing identity, collapsing the isolation boundary
    # back into a job that handled build outputs.
    "artifact-subjects collector gains a signing identity": _sub(
        "release",
        "    permissions:\n      contents: read\n    outputs:\n      hashes: ${{ steps.combine.outputs.hashes }}",
        "    permissions:\n      contents: read\n      id-token: write\n    outputs:\n      hashes: ${{ steps.combine.outputs.hashes }}",
    ),
    # MA4 — the GUI digest step loses its portable branch: the macOS rows are exactly the `soft`
    # rows, so this drops their bundles from the subjects set without failing the release.
    "GUI digests use GNU-only sha256sum on the mac rows": _sub(
        "release",
        '          if command -v sha256sum >/dev/null 2>&1; then\n'
        '            sha256sum -- "${bundles[@]}" > "$OUT"\n'
        '          else\n'
        '            shasum -a 256 -- "${bundles[@]}" > "$OUT"\n'
        '          fi',
        '          sha256sum -- "${bundles[@]}" > "$OUT"',
    ),
    # MA5 — the GUI digest list is written next to the installers, so it ships as a release asset.
    "GUI digest list written into the published bundle dir": _sub(
        "release",
        'OUT="../gui-subjects/subject-hashes-gui-${{ matrix.label }}.txt"',
        'OUT="subject-hashes-gui-${{ matrix.label }}.txt"',
    ),
    # MA6 — a lane silently drops out of the subjects set: still a valid signature, over less.
    "collector stops requiring the conformance lane": _sub(
        "release",
        REQUIRED_LANES_LINE,
        "for required in subject-hashes-sbom.txt; do",
    ),
    # MA7 — the SBOM lane stops publishing digests at all.
    "sbom lane drops its subjects artifact": _sub(
        "release",
        "          name: subject-hashes-sbom\n",
        "          name: sbom-subjects\n",
    ),
    # MA8 — the second signed bundle never reaches the Release.
    "artifact provenance never attached to the release": _sub(
        "release",
        "name: ${{ needs.provenance-artifacts.outputs.provenance-download-name }}",
        "name: sbom-vex",
    ),
    # MA9 — dist.yml's binaries revert to in-band-only provenance.
    "dist subjects no longer threaded from the build matrix": _sub(
        "dist",
        "base64-subjects: ${{ needs.build.outputs.binary-hashes }}",
        "base64-subjects: ${{ steps.hash.outputs.hashes }}",
    ),
    # MA10 — dist's trusted builder pinned by SHA, destroying the builder identity.
    "dist trusted builder pinned by SHA instead of tag": _sub(
        "dist",
        "generator_generic_slsa3.yml@v2.1.0",
        "generator_generic_slsa3.yml@" + "0" * 40,
    ),
    # MA11 — the binary-mode digest hand-off is severed.
    "build-matrix drops the binary-hashes output": _sub(
        "matrix",
        "value: ${{ jobs.binary-hashes.outputs.hashes }}",
        'value: ""',
    ),
    # MA12 — a build step creeps into the binary digest collector.
    "a cargo build creeps into the binary-hashes job": _sub(
        "matrix",
        "          parts=( binary-hashes/*.txt )",
        "          cargo auditable build --release\n          parts=( binary-hashes/*.txt )",
    ),
    # MA13 — the binary digest step loses its portable branch (both Darwin tiers).
    "dist binary digests use GNU-only sha256sum on the mac rows": _sub(
        "matrix",
        '          if command -v sha256sum >/dev/null 2>&1; then\n'
        '            sha256sum -- "${binaries[@]}" > "binary-hashes-${{ matrix.tier }}.txt"\n'
        '          else\n'
        '            shasum -a 256 -- "${binaries[@]}" > "binary-hashes-${{ matrix.tier }}.txt"\n'
        '          fi',
        '          sha256sum -- "${binaries[@]}" > "binary-hashes-${{ matrix.tier }}.txt"',
    ),
}


class TestSlsaL3IsolationContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.release = RELEASE.read_text(encoding="utf-8")
        cls.matrix = BUILD_MATRIX.read_text(encoding="utf-8")
        cls.dist = DIST.read_text(encoding="utf-8")

    def test_contract_holds(self):
        failures = check(self.release, self.matrix, self.dist)
        self.assertEqual(failures, [], "SLSA L3 isolation contract violated:\n  - " + "\n  - ".join(failures))

    def test_every_mutation_is_caught(self):
        """Non-vacuity: each plausible regression must red this gate."""
        for name, mutate in MUTATIONS.items():
            with self.subTest(mutation=name):
                mutated = mutate(self.release, self.matrix, self.dist)
                self.assertNotEqual(
                    mutated,
                    (self.release, self.matrix, self.dist),
                    f"mutation {name!r} changed nothing",
                )
                self.assertNotEqual(
                    check(*mutated),
                    [],
                    f"mutation {name!r} was NOT caught — the assertion covering it is vacuous",
                )


if __name__ == "__main__":
    sys.exit(0 if unittest.main(exit=False, verbosity=2).result.wasSuccessful() else 1)
