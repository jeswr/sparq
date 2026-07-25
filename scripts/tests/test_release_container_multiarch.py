#!/usr/bin/env python3
# [GPT-5.6] sq-fvzi6: hermetic regression test for the sparq-server release image contract.
#
# The workflow itself is the executable publication surface, so this test parses its docker
# job and pins the load-bearing shape: QEMU before Buildx, a native load/smoke/Trivy gate,
# semver/minor/latest tags, and one amd64+arm64 push with SBOM + provenance. The mutation test
# proves the platform assertion is non-vacuous.
#
# Run: python3 scripts/tests/test_release_container_multiarch.py

from __future__ import annotations

import re
import unittest
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "release.yml"


def _step_named(steps: list[dict], name: str) -> tuple[int, dict]:
    """Return one named workflow step and fail if the contract is absent or ambiguous."""
    matches = [(index, step) for index, step in enumerate(steps) if step.get("name") == name]
    assert len(matches) == 1, f"expected exactly one {name!r} step, found {len(matches)}"
    return matches[0]


def assert_release_container_contract(workflow_text: str) -> None:
    """Assert the release job emits the documented multi-platform image without losing gates."""
    workflow = yaml.safe_load(workflow_text)
    steps = workflow["jobs"]["docker"]["steps"]

    qemu_index, qemu = _step_named(steps, "Set up QEMU for arm64")
    buildx_indices = [
        index
        for index, step in enumerate(steps)
        if str(step.get("uses", "")).startswith("docker/setup-buildx-action@")
    ]
    assert len(buildx_indices) == 1, "release docker job must set up Buildx exactly once"
    assert qemu_index < buildx_indices[0], "QEMU must be registered before Buildx"
    assert re.fullmatch(
        r"docker/setup-qemu-action@[0-9a-f]{40}", qemu.get("uses", "")
    ), "QEMU action must be pinned to a full commit SHA"
    assert qemu.get("with", {}).get("platforms") == "arm64"

    _, metadata = _step_named(steps, "Image metadata (tags + OCI labels from the git tag)")
    tags = set(metadata["with"]["tags"].splitlines())
    assert tags == {
        "type=semver,pattern={{version}}",
        "type=semver,pattern={{major}}.{{minor}}",
        "type=raw,value=latest",
    }, "release image tags must preserve full semver, major.minor, and latest"

    _, smoke = _step_named(steps, "Build image for smoke test (load locally)")
    smoke_with = smoke["with"]
    assert smoke_with.get("load") is True and smoke_with.get("push") is False
    assert smoke_with.get("platforms") == "linux/amd64"
    assert smoke_with.get("tags") == "sparq-server:smoke"

    _, smoke_run = _step_named(steps, "Smoke test the image (docker run + /health + ASK query)")
    assert smoke_run.get("run") == "scripts/docker-smoke.sh sparq-server:smoke 3030"

    _, trivy = _step_named(
        steps, "Trivy scan released image (fail on fixable HIGH/CRITICAL)"
    )
    assert trivy["with"].get("image-ref") == "sparq-server:smoke"
    assert trivy["with"].get("exit-code") == "1", "Trivy release scan must remain gating"

    _, push = _step_named(steps, "Build and push")
    push_with = push["with"]
    assert push_with.get("push") is True
    platforms = {value.strip() for value in push_with.get("platforms", "").split(",")}
    assert platforms == {"linux/amd64", "linux/arm64"}
    assert push_with.get("tags") == "${{ steps.meta.outputs.tags }}"
    assert push_with.get("labels") == "${{ steps.meta.outputs.labels }}"
    assert push_with.get("provenance") == "mode=max"
    assert push_with.get("sbom") is True


class ReleaseContainerMultiarch(unittest.TestCase):
    def test_live_release_workflow_has_multiarch_manifest_and_native_gates(self):
        assert_release_container_contract(WORKFLOW.read_text(encoding="utf-8"))

    def test_dropping_arm64_descriptor_fails_contract(self):
        original = WORKFLOW.read_text(encoding="utf-8")
        mutated = original.replace(
            "platforms: linux/amd64,linux/arm64", "platforms: linux/amd64", 1
        )
        self.assertNotEqual(mutated, original, "mutation fixture did not alter the workflow")
        with self.assertRaises(AssertionError):
            assert_release_container_contract(mutated)


if __name__ == "__main__":
    unittest.main(verbosity=2)
