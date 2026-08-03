#!/usr/bin/env python3
# [OPUS-5] #5772 — INSPECTION test for how the `gui-bundle` job in
# .github/workflows/release.yml puts wasm-pack on PATH.
#
# WHY A TEST AT ALL: the property is a couple of YAML lines whose failure mode is
# INVISIBLE on the machine that edits them. This is an x86_64-and-arm64 build matrix,
# so the two ways of getting it wrong both look fine locally:
#
#   1. SOURCE COMPILE. `cargo install wasm-pack --locked` rebuilds wasm-pack and its
#      whole chrono/wasm-bindgen dependency tree from crates.io on EVERY row of EVERY
#      release run. It is green on a good day; a transient registry/CDN blip reds it
#      (observed on PR #1131 in the `js` lane: "download of wa/sm/wasm-bindgen failed
#      / curl failed: [16] Error in the HTTP2 framing layer", cleared by a re-run).
#      Here the blast radius is worse than a re-runnable PR check: the `ubuntu-24.04-arm`
#      row is BLOCKING (`soft: false`), so it fails the `release` job through `needs`.
#   2. AN ARCH-BLIND PREBUILT INSTALLER. jetli/wasm-pack-action — which js.yml and
#      ci.yml legitimately use, because both run x86_64-only — ignores `process.arch`
#      entirely: its dist/index.js switches on `process.platform` alone and always
#      requests `x86_64-pc-windows-msvc` / `x86_64-apple-darwin` /
#      `x86_64-unknown-linux-musl`. On this job's arm64 rows that downloads an x86_64
#      binary, and Linux arm64 has no Rosetta to fall back on.
#
# So the posture is pinned here instead:
#
#   1. NO SOURCE COMPILE. The `gui-bundle` job must not `cargo install wasm-pack`.
#   2. ARCH-AWARE PREBUILT, SHA-PINNED. wasm-pack comes from taiki-e/install-action,
#      which resolves the host arch and has manifest entries for every row of this
#      matrix, pinned to a 40-hex commit SHA per the repo's action-pin policy.
#   3. EXPLICIT `tool:` SELECTOR, EXACTLY VERSIONED. Required by
#      scripts/check-install-action-tool.py (the SHA pin drops install-action's
#      `@<tool>` ref selector, so without `tool:` it installs NOTHING — bead sq-ur7o),
#      and the version is an exact `X.Y.Z`, never floating.
#   4. ORDERING. The install precedes `npm run build:wasm`, which invokes wasm-pack.
#
# SCOPE — deliberately release.yml's `gui-bundle` job ONLY. Other lanes have their own
# posture (js.yml is pinned by test_js_wasm_pack_install.py); this file's silence about
# them is not a claim that they are hardened.
#
# The step splitter is single-sourced from scripts/check-install-action-tool.py rather
# than re-implemented. Hermetic: stdlib only (no PyYAML, no network, no gh).
# Run:  python3 scripts/tests/test_release_wasm_pack_install.py

from __future__ import annotations

import importlib.util
import re
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
RELEASE_YML = REPO_ROOT / ".github" / "workflows" / "release.yml"

JOB_ID = "gui-bundle"
INSTALL_ACTION = "taiki-e/install-action"
# Arch-blind: switches on process.platform alone, always requests the x86_64 asset.
ARCH_BLIND_ACTION = "jetli/wasm-pack-action"
# The repo's action-pin policy: a 40-char lowercase hex commit SHA, never a tag.
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
# `tool: wasm-pack@0.15.0` — an exact release. A bare `wasm-pack` (floating latest) or
# a partial `wasm-pack@0.15` must fail.
TOOL_RE = re.compile(r"^wasm-pack@\d+\.\d+\.\d+$")
# The arm64 rows are the whole reason an arch-aware installer is required. If a row is
# ever removed this test should be revisited, not silently weakened.
ARM64_ROWS = ("ubuntu-24.04-arm", "macos-14")


def _load(name: str, filename: str):
    spec = importlib.util.spec_from_file_location(
        name, REPO_ROOT / "scripts" / filename
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


gate = _load("check_install_action_tool", "check-install-action-tool.py")

# A job key: exactly two spaces of indent, then an identifier and a colon. Comments at
# that indent (this workflow is heavily commented) start with `#` and never match.
_JOB_KEY_RE = re.compile(r"^ {2}([A-Za-z_][A-Za-z0-9_-]*):\s*$")


def job_text(text: str, job_id: str) -> str:
    """Return the source lines of one job block (`  <job_id>:` through the line before
    the next sibling job key)."""
    lines = text.splitlines()
    start = None
    for i, line in enumerate(lines):
        m = _JOB_KEY_RE.match(line)
        if m and m.group(1) == job_id:
            start = i
            break
    assert start is not None, f"{RELEASE_YML.name} has no `{job_id}:` job"
    end = len(lines)
    for j in range(start + 1, len(lines)):
        if _JOB_KEY_RE.match(lines[j]):
            end = j
            break
    return "\n".join(lines[start:end])


def _code_lines(text: str) -> list[str]:
    """Lines with comments and blanks dropped — so a `#`-commented mention of the old
    command (this workflow documents why it was dropped) is never mistaken for a live
    step."""
    return [
        raw
        for raw in text.splitlines()
        if raw.strip() and not raw.strip().startswith("#")
    ]


class GuiBundleWasmPackInstall(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.job = job_text(RELEASE_YML.read_text(encoding="utf-8"), JOB_ID)
        cls.code = _code_lines(cls.job)

    def _wasm_pack_step(self) -> list[str]:
        # Match on the step's parsed `uses:` action, NOT on a substring of the block:
        # the splitter keeps comment lines, so a prose mention of an action leaks into
        # the PRECEDING step's block.
        blocks = [
            b
            for b in gate.split_steps(self.job)
            if (gate._step_uses(b) or (None,))[0] == INSTALL_ACTION
            and any("wasm-pack" in ln for ln in b if not ln.strip().startswith("#"))
        ]
        self.assertEqual(
            len(blocks),
            1,
            f"the `{JOB_ID}` job must have exactly one {INSTALL_ACTION} step "
            f"installing wasm-pack, found {len(blocks)}",
        )
        return blocks[0]

    def test_matrix_still_has_arm64_rows(self):
        """The premise: this job builds on arm64, which is why (2) below matters."""
        # Against the comment-stripped lines: the rationale comment above the install
        # step names both rows, so matching the raw job text would keep this assertion
        # green even after the rows themselves were dropped.
        rows = "\n".join(self.code)
        for row in ARM64_ROWS:
            self.assertIn(
                row,
                rows,
                f"the `{JOB_ID}` matrix no longer has the {row!r} row — the arch-aware "
                "installer requirement this file pins was written for the arm64 rows "
                "(#5772). Revisit this test rather than deleting the assertion.",
            )

    def test_no_cargo_install_wasm_pack(self):
        """(1) The source compile — the actual flake surface — must be gone."""
        offenders = [ln for ln in self.code if "cargo install wasm-pack" in ln]
        self.assertEqual(
            offenders,
            [],
            f"the `{JOB_ID}` job must NOT `cargo install wasm-pack`: recompiling it "
            "(and the chrono/wasm-bindgen source tree) from crates.io on every row of "
            "every release is what lets a transient registry blip fail a BLOCKING "
            f"row and take the release with it (#5772). Install the prebuilt binary "
            f"via {INSTALL_ACTION} instead.",
        )

    def test_installer_is_arch_aware(self):
        """(2a) The arch-BLIND installer must not be used on this arm64 matrix."""
        offenders = [ln for ln in self.code if ARCH_BLIND_ACTION in ln]
        self.assertEqual(
            offenders,
            [],
            f"{ARCH_BLIND_ACTION} ignores `process.arch` — it switches on "
            "`process.platform` alone and always requests the x86_64 asset — so on "
            f"this job's arm64 rows ({', '.join(ARM64_ROWS)}) it installs a binary the "
            f"runner cannot execute (#5772). Use {INSTALL_ACTION}, which resolves the "
            "host arch.",
        )

    def test_installs_prebuilt_binary_sha_pinned(self):
        """(2b) The replacement is the SHA-pinned prebuilt-download action."""
        block = self._wasm_pack_step()
        action, ref = gate._step_uses(block)
        self.assertEqual(action, INSTALL_ACTION)
        self.assertRegex(
            ref,
            SHA_RE,
            f"{INSTALL_ACTION} must be pinned to a 40-hex commit SHA (repo action-pin "
            f"policy), got {ref!r}",
        )

    def test_tool_selector_present_and_version_pinned_exactly(self):
        """(3) `tool:` is load-bearing under a SHA pin, and pins an exact release."""
        block = self._wasm_pack_step()
        self.assertTrue(
            gate._has_with_tool(block),
            "the wasm-pack step needs an explicit `with: tool:` — the SHA pin drops "
            f"{INSTALL_ACTION}'s `@<tool>` ref selector, so without it the action "
            "installs NOTHING and the later `npm run build:wasm` ENOENTs (sq-ur7o).",
        )
        tools = [
            ln.strip().split(":", 1)[1].strip()
            for ln in block
            if ln.strip().startswith("tool:")
        ]
        self.assertEqual(
            len(tools), 1, f"expected exactly one `tool:` input, found {tools!r}"
        )
        self.assertRegex(
            tools[0],
            TOOL_RE,
            "wasm-pack must be pinned to an exact release (e.g. wasm-pack@0.15.0). A "
            "floating `wasm-pack` resolves whatever is newest at release time, which "
            "is how a toolchain bump lands in a release build unreviewed.",
        )

    def test_install_precedes_wasm_build(self):
        """(4) `npm run build:wasm` invokes wasm-pack, so it must already be on PATH."""
        install_at = [i for i, ln in enumerate(self.code) if INSTALL_ACTION in ln]
        build_at = [
            i
            for i, ln in enumerate(self.code)
            if re.match(r"^\s*run:\s*npm run build:wasm\s*$", ln)
        ]
        self.assertEqual(len(install_at), 1, "one wasm-pack install step expected")
        self.assertTrue(
            build_at, f"the `{JOB_ID}` job must still run `npm run build:wasm`"
        )
        self.assertLess(
            install_at[0],
            min(build_at),
            "wasm-pack must be installed BEFORE `npm run build:wasm`: that script "
            "shells out to wasm-pack to build the lean shacl,jsonld bundle gui/app's "
            "`prebuild` hook syncs into js/wasm/.",
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
