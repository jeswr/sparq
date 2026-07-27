#!/usr/bin/env python3
# [OPUS-5] sq-toze.25 / GX-11 — the SLSA **Build L3** isolation contract for the release archives.
#
# WHY THIS TEST EXISTS. `release.yml` runs only on `v*` tags and `workflow_dispatch`, so NOTHING
# in this pipeline registers on a PR: the first time the L3 lane executes is the moment a release
# is being cut, when a mistake is expensive and half-irreversible. Worse, the failure mode here is
# SILENT — the compliance estate (compliance/slsa/controls.md SL-B3-b, both gap registers) will go
# on publishing an L3 claim for the archives long after someone "simplified" the trusted-builder
# call back into an in-band `actions/attest-build-provenance` step. A green release with a false
# level claim is the exact overclaim compliance/ exists to prevent, so the workflow SHAPE is
# pinned here, structurally, on every PR.
#
# WHAT L3 ACTUALLY REQUIRES (and therefore what is asserted): the provenance must be produced by a
# trusted control plane the user build steps cannot influence. Concretely that means all of —
#   * the signing runs in a SEPARATE job that is a bare `uses:` of the trusted reusable builder
#     (no `steps:`/`run:` of ours can execute inside it);
#   * that builder is referenced by an immutable SEMVER TAG, because the builder identity baked
#     into the provenance — the thing `slsa-verifier` matches a consumer policy against — is
#     derived from the `@ref`. A commit-SHA reference yields no verifiable builder identity, so
#     the repo-wide SHA-pin convention is deliberately NOT applied here;
#   * the ONLY thing crossing the boundary is `base64-subjects`, THREADED from the build job's
#     output — never recomputed inside the trusted builder from build artifacts;
#   * the trusted-builder job holds `id-token: write` (its own signing identity) but NOT
#     `contents: write` (it is not the uploader — the `release` job is);
#   * the digest-collecting job neither builds nor signs;
#   * and the whole thing is FAIL-CLOSED: `release` `needs:` the provenance job, and attaches the
#     signed bundle to the Release before SHA256SUMS is computed over it.
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

# The trusted reusable builder, referenced by an immutable semver tag. `@<40-hex>` is REJECTED —
# see the header: a SHA reference produces provenance with no resolvable builder identity.
TRUSTED_BUILDER = re.compile(
    r"^slsa-framework/slsa-github-generator/\.github/workflows/"
    r"generator_generic_slsa3\.yml@v\d+\.\d+\.\d+$"
)
SUBJECTS_EXPR = "${{ needs.package.outputs.hashes }}"
PROVENANCE_ARTIFACT_EXPR = "${{ needs.provenance.outputs.provenance-download-name }}"
HASHES_OUTPUT_VALUE = "${{ jobs.hashes.outputs.hashes }}"
SHA256SUMS_CMD = "sha256sum -- * > SHA256SUMS"

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


def index_of(body: list[str], needle: str) -> int:
    for i, line in enumerate(body):
        if needle in line:
            return i
    return -1


# --------------------------------------------------------------------------------------
# THE CONTRACT. Returns a list of human-readable failures; empty list == contract holds.
# Written as a pure function of the two workflow texts so MUTATIONS can drive it.
# --------------------------------------------------------------------------------------
def check(release_text: str, matrix_text: str) -> list[str]:
    bad: list[str] = []

    prov = job_block(release_text, "provenance")
    rel = job_block(release_text, "release")
    pkg = job_block(release_text, "package")

    if prov is None:
        return ["release.yml has no `provenance` job — the L3 trusted-builder lane is gone"]
    if rel is None:
        return ["release.yml has no `release` job"]

    # --- isolation: a bare `uses:` of the trusted builder, tagged, with nothing of ours inside.
    used = scalar(prov, "uses")
    if used is None or not TRUSTED_BUILDER.match(used):
        bad.append(
            "provenance job must be a bare call to the slsa-github-generator generic builder "
            f"pinned to a semver TAG (not a SHA); found: {used!r}"
        )
    if any(re.match(r"^\s*steps:\s*$", line) for line in prov) or scalar_all(prov, "run"):
        bad.append(
            "provenance job declares steps/run — anything we execute inside the trusted "
            "builder call destroys the isolation the L3 claim rests on"
        )

    # --- the boundary carries digests only, threaded from the build job.
    subjects = scalar(prov, "base64-subjects")
    if subjects != SUBJECTS_EXPR:
        bad.append(
            f"provenance `base64-subjects` must be threaded from the build matrix "
            f"({SUBJECTS_EXPR}); found: {subjects!r}"
        )
    prov_needs = scalar(prov, "needs") or ""
    for dep in ("setup", "package"):
        if dep not in prov_needs:
            bad.append(f"provenance job must `needs: {dep}`; found: {prov_needs!r}")

    # --- least privilege: its OWN signing identity, but it is not the release uploader.
    perms = scalar_all(prov, "id-token") + scalar_all(prov, "actions")
    if "write" not in scalar_all(prov, "id-token"):
        bad.append("provenance job needs `id-token: write` to mint its own signing identity")
    if "read" not in scalar_all(prov, "actions"):
        bad.append("provenance job needs `actions: read` to record the workflow entry point")
    if "write" in scalar_all(prov, "contents"):
        bad.append(
            "provenance job must NOT hold `contents: write` — it is not the uploader "
            "(`upload-assets: false`; the `release` job attaches the bundle)"
        )
    if scalar(prov, "upload-assets") != "false":
        bad.append(
            "provenance job must set `upload-assets: false` — the generator's own uploader "
            "targets github.ref, which is a BRANCH on workflow_dispatch"
        )
    del perms

    # --- fail-closed: no Release without the isolated provenance, and it ships in SHA256SUMS.
    rel_needs = scalar(rel, "needs") or ""
    if "provenance" not in rel_needs:
        bad.append(
            "the `release` job must `needs: provenance` — otherwise a release can ship while "
            f"the docs claim L3; found: {rel_needs!r}"
        )
    dl = index_of(rel, PROVENANCE_ARTIFACT_EXPR)
    sums = index_of(rel, SHA256SUMS_CMD)
    if dl < 0:
        bad.append(
            "the `release` job must download the signed provenance artifact by the generator's "
            f"own output name ({PROVENANCE_ARTIFACT_EXPR})"
        )
    elif sums < 0 or dl > sums:
        bad.append("the provenance download must precede the SHA256SUMS step")

    # --- the build job is NOT the signer: it must still exist and must hand over digests only.
    if pkg is None:
        bad.append("release.yml has no `package` job")

    # --- build-matrix.yml: the digest hand-off.
    if HASHES_OUTPUT_VALUE not in matrix_text:
        bad.append(
            "build-matrix.yml must expose the combined subjects as a workflow_call output "
            f"({HASHES_OUTPUT_VALUE})"
        )
    hashes = job_block(matrix_text, "hashes")
    build = job_block(matrix_text, "build")
    if hashes is None:
        bad.append("build-matrix.yml has no `hashes` job to collect the per-tier digests")
    else:
        if scalar(hashes, "if") != "inputs.mode == 'archive'":
            bad.append("the `hashes` job must be archive-mode only")
        if "build" not in (scalar(hashes, "needs") or ""):
            bad.append("the `hashes` job must `needs: build`")
        if any("cargo " in line for line in hashes):
            bad.append(
                "the `hashes` job must not build anything — it only republishes digests"
            )
        if any("id-token" in line for line in hashes):
            bad.append(
                "the `hashes` job must not hold a signing identity — signing belongs to the "
                "isolated trusted builder alone"
            )
    if build is None:
        bad.append("build-matrix.yml has no `build` job")
    else:
        names = scalar_all(build, "name")
        digest_artifacts = [n for n in names if n.startswith("archive-hashes-")]
        if not digest_artifacts:
            bad.append(
                "the build matrix must upload a per-tier `archive-hashes-<tier>` artifact"
            )
        if any(n.startswith("pkg-") for n in digest_artifacts):
            bad.append(
                "the digest list must not be uploaded under a `pkg-*` name — the `release` job "
                "downloads `pattern: pkg-*` wholesale into the published release assets"
            )
    return bad


# --------------------------------------------------------------------------------------
# Mutations: each must make `check` fail. A mutation that no longer applies (the anchor text
# vanished) is itself a failure — it means the contract moved and this table stopped testing it.
# --------------------------------------------------------------------------------------
def _sub(file: str, old: str, new: str):
    def apply(release_text: str, matrix_text: str) -> tuple[str, str]:
        texts = {"release": release_text, "matrix": matrix_text}
        if old not in texts[file]:
            raise AssertionError(f"mutation anchor not found in {file}: {old!r}")
        texts[file] = texts[file].replace(old, new, 1)
        return texts["release"], texts["matrix"]

    return apply


MUTATIONS = {
    # M1 — THE regression this test is really for: quietly go back to in-band attestation.
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
        "needs: [setup, package, provenance, sbom, gui-bundle, served-conformance]",
        "needs: [setup, package, sbom, gui-bundle, served-conformance]",
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
    # M8 — the release stops attaching the signed bundle at all.
    "signed provenance never attached to the release": _sub(
        "release",
        "name: ${{ needs.provenance.outputs.provenance-download-name }}",
        "name: sbom-vex",
    ),
}


class TestSlsaL3IsolationContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.release = RELEASE.read_text(encoding="utf-8")
        cls.matrix = BUILD_MATRIX.read_text(encoding="utf-8")

    def test_contract_holds(self):
        failures = check(self.release, self.matrix)
        self.assertEqual(failures, [], "SLSA L3 isolation contract violated:\n  - " + "\n  - ".join(failures))

    def test_every_mutation_is_caught(self):
        """Non-vacuity: each plausible regression must red this gate."""
        for name, mutate in MUTATIONS.items():
            with self.subTest(mutation=name):
                mutated_release, mutated_matrix = mutate(self.release, self.matrix)
                self.assertNotEqual(
                    (mutated_release, mutated_matrix),
                    (self.release, self.matrix),
                    f"mutation {name!r} changed nothing",
                )
                self.assertNotEqual(
                    check(mutated_release, mutated_matrix),
                    [],
                    f"mutation {name!r} was NOT caught — the assertion covering it is vacuous",
                )


if __name__ == "__main__":
    sys.exit(0 if unittest.main(exit=False, verbosity=2).result.wasSuccessful() else 1)
