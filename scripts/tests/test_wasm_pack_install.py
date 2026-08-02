#!/usr/bin/env python3
# [OPUS-5] sq-khm3f (#3236) + #5304 — INSPECTION test for how EVERY lane under
# .github/workflows puts wasm-pack on PATH.
#
# WHY A TEST AT ALL: the property is a single `uses:`/`run:` YAML line, and its
# failure mode is INVISIBLE. Reverting to `cargo install wasm-pack --locked` keeps
# a lane green on a good day — it just recompiles wasm-pack and its whole
# chrono/wasm-bindgen source tree from crates.io on every run, so a transient
# registry/CDN blip reds a HARD gate on an unrelated PR (observed on PR #1131:
# "download of wa/sm/wasm-bindgen failed / curl failed: [16] Error in the HTTP2
# framing layer", cleared by a plain re-run). Nothing goes red when the hardening
# is dropped, so the posture is pinned here instead:
#
#   1. NO SOURCE COMPILE. No workflow may `cargo install wasm-pack` — except the
#      one arch-guarded fallback described below.
#   2. PREBUILT BINARY, SHA-PINNED. wasm-pack comes from jetli/wasm-pack-action
#      (one static musl/dylib/exe tarball, fetched through @actions/tool-cache,
#      which retries the download), pinned to a 40-hex commit SHA per the repo's
#      action-pin policy.
#   3. EXACT VERSION PIN. `version:` is an exact `vX.Y.Z`, never `latest` —
#      `latest` re-adds an unauthenticated api.github.com release lookup, i.e. a
#      second rate-limit-flake source, which is the thing this bead removes.
#   4. COVERAGE. Every lane that is known to need wasm-pack still installs it, so
#      a conversion cannot be silently deleted along with the build step it feeds.
#   5. ORDERING (js.yml). The install still precedes the root `npm ci`, because the
#      package's `prepare` lifecycle (sq-bkag git-pin build) runs on `npm ci` and
#      needs wasm-pack on PATH to compile the wasm engine.
#
# THE ONE DOCUMENTED EXCEPTION (#5304): jetli/wasm-pack-action is x86_64-ONLY. Its
# dist/index.js switches on `process.platform` alone and maps win32/darwin/linux to
# `x86_64-pc-windows-msvc` / `x86_64-apple-darwin` / `x86_64-unknown-linux-musl`;
# `process.arch` is never consulted, so on an arm64 runner it downloads an x86_64
# binary. release.yml's `gui-bundle` matrix includes arm64 rows (ubuntu-24.04-arm,
# macos-14), so those rows MUST keep the source compile. A `cargo install wasm-pack`
# is therefore tolerated only in a step explicitly guarded on `runner.arch`, and the
# number of such steps is pinned so the flake surface cannot quietly regrow.
#
# SCOPE — this file asserts the INSTALL MECHANISM only. It deliberately says nothing
# about which wasm-pack VERSION each lane pins (ci.yml's headless-wasm-test job and
# the build lanes are on different releases today), so its silence there is not a
# claim that the versions are unified.
#
# The step splitter is single-sourced from scripts/check-install-action-tool.py
# rather than re-implemented. Hermetic: stdlib only (no PyYAML, no network, no gh).
# Run:  python3 scripts/tests/test_wasm_pack_install.py

from __future__ import annotations

import importlib.util
import re
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
WORKFLOW_DIR = REPO_ROOT / ".github" / "workflows"
JS_YML = WORKFLOW_DIR / "js.yml"

WASM_PACK_ACTION = "jetli/wasm-pack-action"
# The repo's action-pin policy: a 40-char lowercase hex commit SHA, never a tag.
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
# An exact release pin, e.g. `v0.15.0`. `latest` (and any floating ref) must fail.
VERSION_RE = re.compile(r"^v\d+\.\d+\.\d+$")

# Lanes known to need wasm-pack. A lane dropping off this list without its build
# step going with it is a silent loss of coverage, so membership is asserted.
EXPECTED_LANES = (
    "ci.yml",
    "gui.yml",
    "js.yml",
    "nightly-full-sweep.yml",
    "pages.yml",
    "publish.yml",
    "release.yml",
    "site-e2e-hero.yml",
    "site-visual.yml",
)

# The arch-guarded source-compile fallback: {workflow file: number of such steps}.
# Grow this ONLY for a genuinely non-x86_64 runner row, and only alongside an
# `if: runner.arch == 'X64'` prebuilt step in the same job.
EXPECTED_ARCH_FALLBACKS = {"release.yml": 1}


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
    """Lines with comments and blanks dropped — so a `#`-commented mention of the
    old command (this repo comments its workflows heavily) is never mistaken for a
    live step."""
    out = []
    for raw in text.splitlines():
        stripped = raw.strip()
        if not stripped or stripped.startswith("#"):
            continue
        out.append(raw)
    return out


def _workflows() -> list[Path]:
    return sorted(
        p for p in WORKFLOW_DIR.iterdir() if p.suffix in (".yml", ".yaml")
    )


def _wasm_pack_steps(text: str) -> list[list[str]]:
    """Every step block whose parsed `uses:` action is the prebuilt-install action.

    Match on the PARSED `uses:`, NOT on a substring of the block: these workflows are
    heavily commented and the splitter keeps comment lines, so a prose mention of the
    action leaks into the PRECEDING step's block."""
    return [
        b
        for b in gate.split_steps(text)
        if (gate._step_uses(b) or (None,))[0] == WASM_PACK_ACTION
    ]


def _cargo_install_steps(text: str) -> list[list[str]]:
    """Every step block that still compiles wasm-pack from source. Comment lines are
    ignored so the WHY-comments above these steps do not self-trigger."""
    out = []
    for block in gate.split_steps(text):
        if any("cargo install wasm-pack" in ln for ln in _code_lines("\n".join(block))):
            out.append(block)
    return out


def _step_if(block: list[str]) -> str:
    """The step's `if:` expression (empty when unguarded)."""
    for raw in _code_lines("\n".join(block)):
        s = raw.lstrip(" ")
        if s.startswith("- "):
            s = s[2:]
        if s.startswith("if:"):
            return s.split(":", 1)[1].strip()
    return ""


class WasmPackInstallPosture(unittest.TestCase):
    """(1)-(4): the mechanism, across every lane."""

    @classmethod
    def setUpClass(cls):
        cls.texts = {p.name: p.read_text(encoding="utf-8") for p in _workflows()}

    def test_no_unguarded_cargo_install_wasm_pack(self):
        """(1) The source compile — the actual flake surface — must be gone.

        Only an explicitly arch-guarded fallback survives, because the prebuilt
        action ships x86_64 binaries only.
        """
        offenders = []
        for name, text in self.texts.items():
            for block in _cargo_install_steps(text):
                if "runner.arch" not in _step_if(block):
                    head = next(
                        (ln.strip() for ln in block if ln.strip()), "<empty step>"
                    )
                    offenders.append(f"{name}: {head}")
        self.assertEqual(
            offenders,
            [],
            "no workflow may `cargo install wasm-pack`: compiling it (and the "
            "chrono/wasm-bindgen source tree) from crates.io on every run is what "
            "let a transient registry blip red a gating lane (sq-khm3f, PR #1131). "
            f"Install the prebuilt binary via {WASM_PACK_ACTION} instead. On a "
            "non-x86_64 runner, guard the fallback with `if: runner.arch != 'X64'` "
            "and register it in EXPECTED_ARCH_FALLBACKS. Offenders: " + str(offenders),
        )

    def test_arch_fallbacks_are_exactly_the_registered_ones(self):
        """(1b) The tolerated source compiles cannot quietly regrow."""
        found = {}
        for name, text in self.texts.items():
            n = len(_cargo_install_steps(text))
            if n:
                found[name] = n
        self.assertEqual(
            found,
            EXPECTED_ARCH_FALLBACKS,
            "the set of workflows still compiling wasm-pack from source drifted "
            "from the registered arch-fallback allowlist. Every entry must be an "
            "arm64 matrix row that jetli/wasm-pack-action (x86_64-only) cannot "
            "serve; anything else re-adds the crates.io flake surface #5304 removed.",
        )

    def test_every_prebuilt_step_is_sha_pinned(self):
        """(2) The replacement is the SHA-pinned prebuilt-download action."""
        for name, text in self.texts.items():
            for block in _wasm_pack_steps(text):
                action, ref = gate._step_uses(block)
                self.assertEqual(action, WASM_PACK_ACTION)
                self.assertRegex(
                    ref,
                    SHA_RE,
                    f"{name}: {WASM_PACK_ACTION} must be pinned to a 40-hex commit "
                    f"SHA (repo action-pin policy), got {ref!r}",
                )

    def test_every_prebuilt_step_pins_an_exact_version(self):
        """(3) An exact vX.Y.Z — `latest` re-adds a network lookup to resolve it."""
        for name, text in self.texts.items():
            for block in _wasm_pack_steps(text):
                versions = [
                    ln.strip().split(":", 1)[1].strip()
                    for ln in _code_lines("\n".join(block))
                    if ln.strip().startswith("version:")
                ]
                self.assertEqual(
                    len(versions),
                    1,
                    f"{name}: each wasm-pack step must carry exactly one `version:` "
                    f"input, found {versions!r}",
                )
                self.assertRegex(
                    versions[0],
                    VERSION_RE,
                    f"{name}: wasm-pack must be pinned to an exact release (e.g. "
                    "v0.15.0). `latest` costs an unauthenticated api.github.com "
                    "lookup on every run — a second rate-limit-flake source "
                    "(sq-khm3f).",
                )

    def test_every_expected_lane_still_installs_wasm_pack(self):
        """(4) A converted lane cannot silently lose its install step."""
        for name in EXPECTED_LANES:
            with self.subTest(workflow=name):
                text = self.texts.get(name)
                self.assertIsNotNone(text, f"{name} is missing from .github/workflows")
                installs = len(_wasm_pack_steps(text)) + len(_cargo_install_steps(text))
                self.assertGreater(
                    installs,
                    0,
                    f"{name} builds with wasm-pack but no longer installs it. If the "
                    "wasm build genuinely moved out of this lane, drop it from "
                    "EXPECTED_LANES in the same change.",
                )


class JsLaneOrdering(unittest.TestCase):
    """(5) js.yml-specific: `prepare` runs on `npm ci` and needs wasm-pack on PATH.

    Scoped to js.yml on purpose — it is the one lane whose install ordering is load-
    bearing for an npm lifecycle hook rather than for an explicit later build step.
    """

    @classmethod
    def setUpClass(cls):
        cls.code = _code_lines(JS_YML.read_text(encoding="utf-8"))

    def test_install_precedes_npm_ci(self):
        install_at = [i for i, ln in enumerate(self.code) if WASM_PACK_ACTION in ln]
        npm_ci_at = [
            i
            for i, ln in enumerate(self.code)
            if re.match(r"^\s*run:\s*npm ci\s*$", ln)
        ]
        self.assertEqual(len(install_at), 1, "one wasm-pack install step expected")
        self.assertTrue(npm_ci_at, "js.yml must still run the root `npm ci`")
        self.assertLess(
            install_at[0],
            min(npm_ci_at),
            "wasm-pack must be installed BEFORE `npm ci`: the package's `prepare` "
            "lifecycle (sq-bkag) runs on `npm ci` and compiles the wasm engine with "
            "wasm-pack from PATH.",
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
