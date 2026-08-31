#!/usr/bin/env python3
# [OPUS-5] sq-khm3f (#3236) + #5849 — ONE lane-parameterised INSPECTION test for how
# the CONVERTED lanes put wasm-pack on PATH. Supersedes the per-lane
# test_js_wasm_pack_install.py.
#
# WHY A TEST AT ALL: the property is a single `uses:`/`run:` YAML line, and its
# failure mode is INVISIBLE. Reverting to `cargo install wasm-pack --locked` keeps a
# lane green on a good day — it just recompiles wasm-pack and its whole
# chrono/wasm-bindgen source tree from crates.io on every run, so a transient
# registry/CDN blip reds a HARD gate on an unrelated PR (observed on PR #1131:
# "download of wa/sm/wasm-bindgen failed / curl failed: [16] Error in the HTTP2
# framing layer", cleared by a plain re-run). Nothing goes red when the hardening is
# dropped, so the posture is pinned here instead. Per REGISTERED lane:
#
#   1. NO SOURCE COMPILE. The lane must not `cargo install wasm-pack` anywhere.
#   2. PREBUILT BINARY, SHA-PINNED. Exactly one install step, using an installer this
#      file allows for that lane's runner-arch profile, pinned to a 40-hex commit SHA
#      per the repo's action-pin policy.
#   3. EXACT VERSION PIN. Never `latest` — `latest` re-adds an unauthenticated
#      api.github.com release lookup, i.e. a second rate-limit-flake source, which is
#      the thing this hardening removes.
#   4. ARCH PROFILE. An X64_ONLY lane may use either installer; a MULTI_ARCH lane may
#      use only an ARCH-AWARE one (see ARCH_AWARE_INSTALLERS below). An X64_ONLY
#      lane's workflow is additionally checked for arm64 runner labels, so ADDING an
#      arm64 row reds this test instead of silently downloading a wrong-arch binary.
#   5. ORDERING (opt-in per lane). js.yml installs before the root `npm ci`, because
#      the package's `prepare` lifecycle (sq-bkag git-pin build) runs on `npm ci` and
#      needs wasm-pack on PATH to compile the wasm engine.
#
# WHY THE ARCH PROFILE MATTERS (#5849): jetli/wasm-pack-action is x86_64-ONLY. Its
# dist/index.js switches on `process.platform` alone and maps win32/darwin/linux to
# `x86_64-pc-windows-msvc` / `x86_64-apple-darwin` / `x86_64-unknown-linux-musl`;
# `process.arch` is never consulted, so on an arm64 runner it downloads an x86_64
# binary and Linux arm64 has no Rosetta to fall back on. taiki-e/install-action
# resolves the host arch, and its wasm-pack manifest carries a 0.15.0 entry for
# x86_64+aarch64 linux-musl, x86_64+aarch64 macos and x86_64 windows — which is why
# release.yml's arm64-bearing `gui-bundle` matrix needs no `cargo install` fallback.
#
# SCOPE — REGISTERED LANES ONLY. gui.yml / pages.yml / publish.yml /
# nightly-full-sweep.yml / site-e2e-hero.yml / site-visual.yml still `cargo install`
# wasm-pack; converting them is separate work (#5304) and is NOT asserted here, so
# this file's silence about them is not a claim that they are hardened. Registering a
# converted lane is a one-entry edit to LANES below. This file also asserts the
# INSTALL MECHANISM only — it says nothing about ci.yml's own wasm-pack usage or about
# unifying versions across lanes.
#
# The step splitter is single-sourced from scripts/check-install-action-tool.py rather
# than re-implemented. Hermetic: stdlib only (no PyYAML, no network, no gh).
# Run:  python3 scripts/tests/test_wasm_pack_install.py

from __future__ import annotations

import importlib.util
import re
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
WORKFLOW_DIR = REPO_ROOT / ".github" / "workflows"

JETLI = "jetli/wasm-pack-action"
INSTALL_ACTION = "taiki-e/install-action"
# Installers that resolve the HOST architecture. jetli is deliberately absent.
ARCH_AWARE_INSTALLERS = frozenset({INSTALL_ACTION})
ALL_INSTALLERS = frozenset({JETLI, INSTALL_ACTION})

# The repo's action-pin policy: a 40-char lowercase hex commit SHA, never a tag.
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
# jetli's exact release pin, e.g. `v0.15.0`. `latest` (and any floating ref) fails.
JETLI_VERSION_RE = re.compile(r"^v\d+\.\d+\.\d+$")
# install-action's `tool:` input, e.g. `wasm-pack@0.15.0`. A bare `wasm-pack` (which
# resolves `latest` over the network) or a partial `wasm-pack@0.15` fails.
TOOL_RE = re.compile(r"^wasm-pack@\d+\.\d+\.\d+$")

# Runner labels that are (or may be) arm64. Used only to police an X64_ONLY lane's
# declaration: if one of these appears anywhere in that workflow, the lane's profile
# is stale and its x86_64-only installer may now be wrong.
ARM64_RUNNER_LABELS = (
    "ubuntu-24.04-arm",
    "ubuntu-22.04-arm",
    "ubuntu-latest-arm",
    "macos-14",
    "macos-15",
    "macos-latest",
)

X64_ONLY = "X64_ONLY"
MULTI_ARCH = "MULTI_ARCH"

# THE LANE TABLE. Each entry states a FACT about the lane (which runners it uses, and
# whether the install ordering is load-bearing); the allowed installers are DERIVED
# from `arch` rather than declared, so a wrong installer cannot be legitimised by
# editing the same line that describes the lane.
LANES = (
    {
        "workflow": "js.yml",
        "arch": X64_ONLY,  # single job, `runs-on: ubuntu-latest`
        "before_npm_ci": True,
        "why": (
            "GATING npm-package lane; its `prepare` lifecycle compiles the wasm "
            "engine during the root `npm ci`, so wasm-pack must already be on PATH"
        ),
    },
    {
        "workflow": "release.yml",
        # `gui-bundle` matrix: ubuntu-latest, ubuntu-24.04-arm (BLOCKING), macos-14,
        # macos-15-intel, windows-latest — two of them arm64.
        "arch": MULTI_ARCH,
        "before_npm_ci": False,
        "why": (
            "the `gui-bundle` job builds the wasm bundle on a 5-row matrix whose "
            "arm64-linux row is BLOCKING, and a failed row fails `release` via `needs`"
        ),
    },
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


gate = _load("check_install_action_tool", "check-install-action-tool.py")


def _code_lines(text: str) -> list[str]:
    """Lines with comments and blanks dropped — so a `#`-commented mention of the old
    command (this repo comments its workflows heavily) is never mistaken for a live
    step."""
    out = []
    for raw in text.splitlines():
        stripped = raw.strip()
        if not stripped or stripped.startswith("#"):
            continue
        out.append(raw)
    return out


def _install_steps(text: str) -> list[tuple[str, list[str]]]:
    """(installer, block) for every step whose parsed `uses:` is a wasm-pack installer.

    Match on the PARSED `uses:`, NOT on a substring of the block: these workflows are
    heavily commented and the splitter keeps comment lines, so a prose mention of an
    action leaks into the PRECEDING step's block.

    An install-action step is only counted when its `tool:` names wasm-pack — the same
    workflow installs other tools (cargo-llvm-cov, cargo-mutants, …) with that action.
    """
    out = []
    for block in gate.split_steps(text):
        uses = gate._step_uses(block)
        if uses is None or uses[0] not in ALL_INSTALLERS:
            continue
        if uses[0] == INSTALL_ACTION and "wasm-pack" not in (_tool(block) or ""):
            continue
        out.append((uses[0], block))
    return out


def _cargo_install_steps(text: str) -> list[list[str]]:
    """Every step block that still compiles wasm-pack from source. Comment lines are
    ignored so the WHY-comments above these steps do not self-trigger."""
    return [
        block
        for block in gate.split_steps(text)
        if any("cargo install wasm-pack" in ln for ln in _code_lines("\n".join(block)))
    ]


def _input(block: list[str], key: str) -> str | None:
    """The value of a `with:` input on this step (None when absent)."""
    values = [
        ln.strip().split(":", 1)[1].strip()
        for ln in _code_lines("\n".join(block))
        if ln.strip().startswith(f"{key}:")
    ]
    return values[0] if len(values) == 1 else (None if not values else "<duplicated>")


def _tool(block: list[str]) -> str | None:
    return _input(block, "tool")


class WasmPackInstallPosture(unittest.TestCase):
    """(1)-(4), asserted per REGISTERED lane."""

    @classmethod
    def setUpClass(cls):
        cls.texts = {}
        for lane in LANES:
            path = WORKFLOW_DIR / lane["workflow"]
            cls.texts[lane["workflow"]] = (
                path.read_text(encoding="utf-8") if path.exists() else None
            )

    def _text(self, lane) -> str:
        text = self.texts[lane["workflow"]]
        self.assertIsNotNone(
            text, f"{lane['workflow']} is registered in LANES but does not exist"
        )
        return text

    def test_no_cargo_install_wasm_pack(self):
        """(1) The source compile — the actual flake surface — must be gone.

        No arch-guarded fallback is tolerated: both allowed installers ship prebuilt
        binaries for every runner arch their lane uses (#5849).
        """
        for lane in LANES:
            with self.subTest(workflow=lane["workflow"]):
                offenders = [
                    next((ln.strip() for ln in b if ln.strip()), "<empty step>")
                    for b in _cargo_install_steps(self._text(lane))
                ]
                self.assertEqual(
                    offenders,
                    [],
                    f"{lane['workflow']} must NOT `cargo install wasm-pack`: "
                    "compiling it (and the chrono/wasm-bindgen source tree) from "
                    "crates.io on every run is what let a transient registry blip red "
                    f"a gating lane (sq-khm3f, PR #1131). Lane: {lane['why']}. "
                    f"Offenders: {offenders}",
                )

    def test_exactly_one_install_step_with_an_allowed_installer(self):
        """(2) One install step, and its installer suits the lane's arch profile."""
        for lane in LANES:
            with self.subTest(workflow=lane["workflow"]):
                steps = _install_steps(self._text(lane))
                self.assertEqual(
                    len(steps),
                    1,
                    f"{lane['workflow']} must have exactly one wasm-pack install "
                    f"step, found {len(steps)}. It needs one because {lane['why']}; "
                    "if the wasm build genuinely left this lane, drop the lane from "
                    "LANES in the same change.",
                )
                installer = steps[0][0]
                allowed = (
                    ARCH_AWARE_INSTALLERS if lane["arch"] == MULTI_ARCH else ALL_INSTALLERS
                )
                self.assertIn(
                    installer,
                    allowed,
                    f"{lane['workflow']} is {lane['arch']} ({lane['why']}), so it may "
                    f"install wasm-pack only via {sorted(allowed)}, not {installer!r}. "
                    f"{JETLI} never reads `process.arch` — it hardcodes x86_64 targets "
                    "per platform, so on an arm64 runner it downloads an x86_64 binary "
                    "(#5849).",
                )

    def test_installer_is_sha_pinned(self):
        """(2b) Repo action-pin policy: a 40-hex commit SHA, never a floating tag."""
        for lane in LANES:
            with self.subTest(workflow=lane["workflow"]):
                for installer, block in _install_steps(self._text(lane)):
                    _, ref = gate._step_uses(block)
                    self.assertRegex(
                        ref,
                        SHA_RE,
                        f"{lane['workflow']}: {installer} must be pinned to a 40-hex "
                        f"commit SHA (repo action-pin policy), got {ref!r}",
                    )

    def test_version_is_pinned_exactly_not_latest(self):
        """(3) An exact version — `latest` costs a network lookup to resolve it."""
        for lane in LANES:
            with self.subTest(workflow=lane["workflow"]):
                for installer, block in _install_steps(self._text(lane)):
                    if installer == JETLI:
                        got, pattern, key = (
                            _input(block, "version"),
                            JETLI_VERSION_RE,
                            "version",
                        )
                    else:
                        got, pattern, key = _tool(block), TOOL_RE, "tool"
                    self.assertIsNotNone(
                        got,
                        f"{lane['workflow']}: the {installer} step must carry exactly "
                        f"one `{key}:` input. Under a SHA pin, install-action's "
                        "`@<tool>` ref selector is gone, so without `tool:` it "
                        "installs NOTHING (sq-ur7o).",
                    )
                    self.assertRegex(
                        got,
                        pattern,
                        f"{lane['workflow']}: wasm-pack must be pinned to an exact "
                        f"release (e.g. `v0.15.0` for {JETLI}, `wasm-pack@0.15.0` for "
                        f"{INSTALL_ACTION}), got {got!r}. `latest` costs an "
                        "unauthenticated release lookup on every run — a second "
                        "rate-limit-flake source (sq-khm3f).",
                    )

    def test_x64_only_lanes_have_no_arm64_runner(self):
        """(4) An X64_ONLY declaration must stay true.

        Deliberately an OVER-approximation: it scans the whole workflow, not just the
        job holding the install step, because job-scoping needs a real YAML parse and
        this file is stdlib-only. If an arm64 row lands in a job that does NOT use
        wasm-pack, the honest fix is still to re-check the lane and move it to
        MULTI_ARCH (or split the workflow), not to loosen this list.
        """
        for lane in LANES:
            if lane["arch"] != X64_ONLY:
                continue
            with self.subTest(workflow=lane["workflow"]):
                text = self._text(lane)
                found = [
                    label
                    for label in ARM64_RUNNER_LABELS
                    if any(label in ln for ln in _code_lines(text))
                ]
                self.assertEqual(
                    found,
                    [],
                    f"{lane['workflow']} is declared {X64_ONLY} but references arm64-"
                    f"capable runner label(s) {found}. Its installer may be x86_64-only "
                    f"({JETLI} hardcodes x86_64 targets), which downloads a wrong-arch "
                    f"binary. Move the lane to {MULTI_ARCH} and convert the step to "
                    f"{INSTALL_ACTION}.",
                )


class LaneOrdering(unittest.TestCase):
    """(5) The opt-in ordering constraint, for the lanes that declare one."""

    def test_install_precedes_npm_ci(self):
        for lane in LANES:
            if not lane["before_npm_ci"]:
                continue
            with self.subTest(workflow=lane["workflow"]):
                code = _code_lines(
                    (WORKFLOW_DIR / lane["workflow"]).read_text(encoding="utf-8")
                )
                install_at = [
                    i
                    for i, ln in enumerate(code)
                    if any(installer in ln for installer in ALL_INSTALLERS)
                ]
                npm_ci_at = [
                    i
                    for i, ln in enumerate(code)
                    if re.match(r"^\s*run:\s*npm ci\s*$", ln)
                ]
                self.assertEqual(
                    len(install_at), 1, "one wasm-pack install step expected"
                )
                self.assertTrue(
                    npm_ci_at, f"{lane['workflow']} must still run the root `npm ci`"
                )
                self.assertLess(
                    install_at[0],
                    min(npm_ci_at),
                    f"{lane['workflow']}: wasm-pack must be installed BEFORE `npm ci` "
                    "— the package's `prepare` lifecycle (sq-bkag) runs on `npm ci` and "
                    "compiles the wasm engine with wasm-pack from PATH.",
                )


if __name__ == "__main__":
    unittest.main(verbosity=2)
