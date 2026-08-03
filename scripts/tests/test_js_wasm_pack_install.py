#!/usr/bin/env python3
# [OPUS-5] sq-khm3f — INSPECTION test for how the GATING `js` lane
# (.github/workflows/js.yml) puts wasm-pack on PATH.
#
# WHY a test at all: the property is a single `uses:`/`run:` YAML line, and its
# failure mode is INVISIBLE. Reverting to `cargo install wasm-pack --locked` keeps
# the lane green on a good day — it just recompiles wasm-pack and its whole
# chrono/wasm-bindgen source tree from crates.io on every run, so a transient
# registry/CDN blip reds this HARD gate on an unrelated PR (observed on PR #1131:
# "download of wa/sm/wasm-bindgen failed / curl failed: [16] Error in the HTTP2
# framing layer", cleared by a plain re-run). Nothing goes red when the hardening
# is dropped, so the posture is pinned here instead:
#
#   1. NO SOURCE COMPILE. js.yml must not `cargo install wasm-pack` anywhere.
#   2. PREBUILT BINARY, SHA-PINNED. wasm-pack comes from jetli/wasm-pack-action
#      (one static musl tarball, fetched through @actions/tool-cache, which retries
#      the download), pinned to a 40-hex commit SHA per the repo's action-pin policy.
#   3. EXACT VERSION PIN. `version:` is an exact `vX.Y.Z`, never `latest` —
#      `latest` re-adds an unauthenticated api.github.com release lookup, i.e. a
#      second rate-limit-flake source, which is the thing this bead removes.
#   4. ORDERING. The install still precedes the root `npm ci`, because the package's
#      `prepare` lifecycle (sq-bkag git-pin build) runs on `npm ci` and needs
#      wasm-pack on PATH to compile the wasm engine.
#
# SCOPE — deliberately js.yml ONLY, and this file keeps asserting the DIRECT `uses:`
# because the gating `js` lane installs wasm-pack ahead of `npm ci` on its own, without
# the shared composite. The other seven workflows (gui.yml / pages.yml / publish.yml /
# release.yml / nightly-full-sweep.yml / site-e2e-hero.yml / site-visual.yml) no longer
# `cargo install` wasm-pack either — #5776 moved all sixteen of their call sites onto
# .github/actions/install-wasm-pack — but that is asserted by its own posture test,
# scripts/tests/test_wasm_pack_install_posture.py, not here.
#
# The step splitter is single-sourced from scripts/check-install-action-tool.py
# rather than re-implemented. Hermetic: stdlib only (no PyYAML, no network, no gh).
# Run:  python3 scripts/tests/test_js_wasm_pack_install.py

from __future__ import annotations

import importlib.util
import re
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
JS_YML = REPO_ROOT / ".github" / "workflows" / "js.yml"

WASM_PACK_ACTION = "jetli/wasm-pack-action"
# The repo's action-pin policy: a 40-char lowercase hex commit SHA, never a tag.
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
# An exact release pin, e.g. `v0.15.0`. `latest` (and any floating ref) must fail.
VERSION_RE = re.compile(r"^v\d+\.\d+\.\d+$")


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


class JsLaneWasmPackInstall(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = JS_YML.read_text(encoding="utf-8")
        cls.code = _code_lines(cls.text)

    def _wasm_pack_step(self) -> list[str]:
        # Match on the step's parsed `uses:` action, NOT on a substring of the block:
        # this workflow is heavily commented, and the splitter keeps comment lines, so
        # a prose mention of the action leaks into the PRECEDING step's block.
        blocks = [
            b
            for b in gate.split_steps(self.text)
            if (gate._step_uses(b) or (None,))[0] == WASM_PACK_ACTION
        ]
        self.assertEqual(
            len(blocks),
            1,
            f"js.yml must have exactly one {WASM_PACK_ACTION} step, found "
            f"{len(blocks)}",
        )
        return blocks[0]

    def test_no_cargo_install_wasm_pack(self):
        """(1) The source compile — the actual flake surface — must be gone."""
        offenders = [ln for ln in self.code if "cargo install wasm-pack" in ln]
        self.assertEqual(
            offenders,
            [],
            "js.yml must NOT `cargo install wasm-pack`: compiling it (and the "
            "chrono/wasm-bindgen source tree) from crates.io on every run is what "
            "let a transient registry blip red this gating lane (sq-khm3f). Install "
            f"the prebuilt binary via {WASM_PACK_ACTION} instead.",
        )

    def test_installs_prebuilt_binary_sha_pinned(self):
        """(2) The replacement is the SHA-pinned prebuilt-download action."""
        block = self._wasm_pack_step()
        uses = gate._step_uses(block)
        self.assertIsNotNone(uses, "the wasm-pack step must have a `uses:` line")
        action, ref = uses
        self.assertEqual(action, WASM_PACK_ACTION)
        self.assertRegex(
            ref,
            SHA_RE,
            f"{WASM_PACK_ACTION} must be pinned to a 40-hex commit SHA (repo "
            f"action-pin policy), got {ref!r}",
        )

    def test_version_is_pinned_exactly_not_latest(self):
        """(3) An exact vX.Y.Z — `latest` re-adds a network lookup to resolve it."""
        block = self._wasm_pack_step()
        versions = [
            ln.strip().split(":", 1)[1].strip()
            for ln in block
            if ln.strip().startswith("version:")
        ]
        self.assertEqual(
            len(versions),
            1,
            "the wasm-pack step must carry exactly one `version:` input, found "
            f"{versions!r}",
        )
        self.assertRegex(
            versions[0],
            VERSION_RE,
            "wasm-pack must be pinned to an exact release (e.g. v0.15.0). "
            "`latest` costs an unauthenticated api.github.com lookup on every run "
            "— a second rate-limit-flake source (sq-khm3f).",
        )

    def test_install_precedes_npm_ci(self):
        """(4) `prepare` runs on `npm ci` and needs wasm-pack already on PATH."""
        install_at = [
            i for i, ln in enumerate(self.code) if WASM_PACK_ACTION in ln
        ]
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
