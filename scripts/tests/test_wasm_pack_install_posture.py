#!/usr/bin/env python3
# [OPUS-5] #5776 — INSPECTION test for how EVERY lane puts wasm-pack on PATH.
#
# WHY A TEST AT ALL. sq-khm3f/#3236 removed the `cargo install wasm-pack` source
# compile from the gating `js` lane after a transient crates.io blip red-gated PR #1131
# ("download of wa/sm/wasm-bindgen failed / curl failed: [16] Error in the HTTP2 framing
# layer", cleared by a plain re-run), and pinned that ONE lane's posture in
# scripts/tests/test_js_wasm_pack_install.py. #5776 converted the other sixteen call
# sites — gui.yml (5), nightly-full-sweep.yml (4), publish.yml (3), pages.yml,
# release.yml, site-e2e-hero.yml, site-visual.yml — onto the shared composite action
# .github/actions/install-wasm-pack.
#
# The failure mode is INVISIBLE: a lane reverted to `cargo install wasm-pack` keeps
# passing on a good day, it just recompiles wasm-pack and its whole chrono/wasm-bindgen
# source tree from crates.io on every run, so a registry/CDN blip reds an unrelated PR.
# Nothing goes red when the hardening is dropped — hence this file.
#
# THE PROPERTY THAT MATTERS MOST is not "use the action", it is the ARCH GATE inside it.
# jetli/wasm-pack-action v0.4.0 selects its download target from `process.platform`
# ALONE and ignores `process.arch` (dist/index.js: linux -> x86_64-unknown-linux-musl,
# unconditionally). wasm-pack does publish aarch64-unknown-linux-musl and
# aarch64-apple-darwin tarballs, but the action can never request them. Drop the
# `runner.arch == 'X64'` condition and release.yml's `gui-bundle` matrix hands its
# `ubuntu-24.04-arm` row — a row deliberately marked BLOCKING (soft:false) — a static
# x86_64 ELF that installs "successfully" and then dies with `Exec format error`. That
# regression is one deleted `if:` clause away and would only show up in a release run,
# so it is pinned here explicitly.
#
# The step splitter is single-sourced from scripts/check-install-action-tool.py rather
# than re-implemented. Hermetic: stdlib only (no PyYAML, no network, no gh).
# Run:  python3 scripts/tests/test_wasm_pack_install_posture.py

from __future__ import annotations

import importlib.util
import re
import sys
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
COMPOSITE_DIR = REPO_ROOT / ".github" / "actions" / "install-wasm-pack"
COMPOSITE_YML = COMPOSITE_DIR / "action.yml"

# How workflows reference the composite. A local action is resolved relative to the
# checked-out repo root, so the path must be exactly this and the directory must hold an
# `action.yml`.
COMPOSITE_USES = "./.github/actions/install-wasm-pack"
WASM_PACK_ACTION = "jetli/wasm-pack-action"

# The repo's action-pin policy: a 40-char lowercase hex commit SHA, never a tag.
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
# The composite's default version input: an exact release, no leading `v`, never
# `latest` (which re-adds an unauthenticated api.github.com lookup per run).
VERSION_RE = re.compile(r"^\d+\.\d+\.\d+$")

# Lanes #5776 converted, and how many install sites each had. `>=`, not `==`: adding a
# new lane must not red this test, but silently reverting one back to a source compile
# (or deleting the step) must.
CONVERTED: dict[str, int] = {
    "gui.yml": 5,
    "nightly-full-sweep.yml": 4,
    "publish.yml": 3,
    "pages.yml": 1,
    "release.yml": 1,
    "site-e2e-hero.yml": 1,
    "site-visual.yml": 1,
}


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
    """Lines with comments and blanks dropped — this repo comments its workflows
    heavily, so a `#`-commented mention of the old command (js.yml and docs-quality.yml
    both explain WHY it was removed) must never be mistaken for a live step."""
    return [
        raw
        for raw in text.splitlines()
        if raw.strip() and not raw.strip().startswith("#")
    ]


def _step_local_uses(block: list[str]) -> str | None:
    """Return the `uses:` value of a LOCAL action step (`./...`), else None.

    gate._step_uses only matches `owner/repo@ref`; a local action has no `@ref`, so it
    needs its own matcher.
    """
    for raw in block:
        s = raw.lstrip(" ")
        if s.startswith("- "):
            s = s[2:]
        m = re.match(r"^uses:\s*(\./\S+)", s)
        if m:
            return m.group(1)
    return None


def _step_if(block: list[str]) -> str | None:
    """Return the step's `if:` expression, else None."""
    for raw in block:
        s = raw.lstrip(" ")
        if s.startswith("- "):
            s = s[2:]
        if s.startswith("if:"):
            return s[len("if:") :].strip()
    return None


def _workflow_files() -> list[Path]:
    return sorted(
        [p for p in WORKFLOWS.iterdir() if p.suffix in (".yml", ".yaml") and p.is_file()]
    )


class NoSourceCompileAnywhere(unittest.TestCase):
    """(1) The source compile — the actual flake surface — is gone from every lane."""

    def test_no_workflow_cargo_installs_wasm_pack(self):
        offenders: list[str] = []
        for p in _workflow_files():
            for ln in _code_lines(p.read_text(encoding="utf-8")):
                if "cargo install wasm-pack" in ln:
                    offenders.append(f"{p.name}: {ln.strip()}")
        self.assertEqual(
            offenders,
            [],
            "no workflow may `cargo install wasm-pack`: compiling it (and the "
            "chrono/wasm-bindgen source tree) from crates.io on every run is what let a "
            "transient registry blip red a gating lane (sq-khm3f, PR #1131). Use "
            f"`uses: {COMPOSITE_USES}` instead — it downloads the prebuilt binary where "
            "that is proven and keeps a source fallback only for targets the prebuilt "
            f"path cannot serve. Offenders:\n  " + "\n  ".join(offenders),
        )

    def test_every_wasm_pack_install_uses_an_approved_mechanism(self):
        """Whatever puts wasm-pack on PATH must be the composite or the SHA-pinned
        action — never an ad-hoc `curl | tar` or a package manager."""
        bad: list[str] = []
        for p in _workflow_files():
            text = p.read_text(encoding="utf-8")
            for block in gate.split_steps(text):
                code = _code_lines("\n".join(block))
                if not any("wasm-pack" in ln for ln in code):
                    continue
                # Only steps that INSTALL wasm-pack; `wasm-pack build/test` invocations
                # and `npm run build:wasm` are consumers, not installers.
                name = next(
                    (ln for ln in code if ln.lstrip(" ").startswith("- name:")), ""
                )
                if "Install wasm-pack" not in name:
                    continue
                local = _step_local_uses(block)
                remote = gate._step_uses(block)
                if local == COMPOSITE_USES:
                    continue
                if remote and remote[0] == WASM_PACK_ACTION:
                    continue
                bad.append(f"{p.name}: {name.strip()}")
        self.assertEqual(
            bad,
            [],
            "every wasm-pack install step must `uses:` either "
            f"`{COMPOSITE_USES}` or the SHA-pinned `{WASM_PACK_ACTION}`. Offenders:\n  "
            + "\n  ".join(bad),
        )

    def test_converted_lanes_still_wired(self):
        """Each lane #5776 converted still installs via the composite — a silently
        deleted or reverted step reds here."""
        for filename, expected in sorted(CONVERTED.items()):
            with self.subTest(workflow=filename):
                path = WORKFLOWS / filename
                self.assertTrue(path.exists(), f"{filename} is missing")
                found = sum(
                    1
                    for block in gate.split_steps(path.read_text(encoding="utf-8"))
                    if _step_local_uses(block) == COMPOSITE_USES
                )
                self.assertGreaterEqual(
                    found,
                    expected,
                    f"{filename} must install wasm-pack via `{COMPOSITE_USES}` at least "
                    f"{expected} time(s) (#5776 converted that many sites), found "
                    f"{found}.",
                )


class CompositeActionPosture(unittest.TestCase):
    """(2) The one place the mechanism is decided is itself pinned."""

    @classmethod
    def setUpClass(cls):
        cls.text = COMPOSITE_YML.read_text(encoding="utf-8")
        cls.blocks = gate.split_steps(cls.text)

    def test_composite_action_file_exists(self):
        """A local `uses: ./dir` resolves to `dir/action.yml` in the checkout — the
        filename is load-bearing, `action.yaml` at another path would not resolve."""
        self.assertTrue(
            COMPOSITE_YML.is_file(),
            f"{COMPOSITE_YML.relative_to(REPO_ROOT)} must exist: every converted lane "
            f"references it as `uses: {COMPOSITE_USES}`.",
        )

    def _prebuilt_step(self) -> list[str]:
        blocks = [
            b
            for b in self.blocks
            if (gate._step_uses(b) or (None,))[0] == WASM_PACK_ACTION
        ]
        self.assertEqual(
            len(blocks),
            1,
            f"the composite must have exactly one {WASM_PACK_ACTION} step, found "
            f"{len(blocks)}",
        )
        return blocks[0]

    def _source_step(self) -> list[str]:
        blocks = [
            b
            for b in self.blocks
            if any("cargo install wasm-pack" in ln for ln in _code_lines("\n".join(b)))
        ]
        self.assertEqual(
            len(blocks),
            1,
            f"the composite must have exactly one source-compile fallback step, found "
            f"{len(blocks)}",
        )
        return blocks[0]

    def test_prebuilt_action_is_sha_pinned(self):
        action, ref = gate._step_uses(self._prebuilt_step())
        self.assertEqual(action, WASM_PACK_ACTION)
        self.assertRegex(
            ref,
            SHA_RE,
            f"{WASM_PACK_ACTION} must be pinned to a 40-hex commit SHA (repo action-pin "
            f"policy), got {ref!r}",
        )

    def test_default_version_is_pinned_exactly_not_latest(self):
        """`latest` costs an unauthenticated api.github.com lookup on every run — a
        second rate-limit-flake source, which is the thing this action removes."""
        defaults = [
            ln.strip().split(":", 1)[1].strip().strip("\"'")
            for ln in self.text.splitlines()
            if ln.strip().startswith("default:")
        ]
        self.assertEqual(
            len(defaults),
            1,
            f"the composite must declare exactly one input default, found {defaults!r}",
        )
        self.assertRegex(
            defaults[0],
            VERSION_RE,
            "wasm-pack must be pinned to an exact release with NO leading `v` (the "
            "prebuilt step adds the `v`, the source fallback passes it to "
            "`cargo install --version`, which rejects one). Got "
            f"{defaults[0]!r}.",
        )

    def test_prebuilt_path_is_gated_on_x64_linux(self):
        """THE correctness property. jetli/wasm-pack-action ignores `process.arch` and
        always downloads x86_64 — so on release.yml's BLOCKING `ubuntu-24.04-arm` row it
        would install a binary that cannot exec. Both halves of the condition are
        load-bearing; neither may be dropped."""
        cond = _step_if(self._prebuilt_step())
        self.assertIsNotNone(
            cond, "the prebuilt step must carry an `if:` restricting it to x64 Linux"
        )
        self.assertIn(
            "runner.os == 'Linux'",
            cond,
            f"prebuilt step `if:` must restrict to Linux, got {cond!r}",
        )
        self.assertIn(
            "runner.arch == 'X64'",
            cond,
            "prebuilt step `if:` must restrict to X64: jetli/wasm-pack-action always "
            "downloads the x86_64 tarball regardless of the runner's architecture, so "
            "without this clause release.yml's BLOCKING ubuntu-24.04-arm row gets an "
            f"unexecutable binary. Got {cond!r}",
        )

    def test_source_fallback_covers_exactly_the_complement(self):
        """No runner may fall between the two steps (wasm-pack missing → the build step
        ENOENTs) or match both (a source compile that overwrites the prebuilt binary,
        silently re-adding the flake surface the change removed)."""
        prebuilt = _step_if(self._prebuilt_step())
        fallback = _step_if(self._source_step())
        self.assertIsNotNone(fallback, "the source fallback step must carry an `if:`")

        def _expr(cond: str) -> str:
            m = re.fullmatch(r"\$\{\{(.*)\}\}", cond.strip(), flags=re.S)
            return (m.group(1) if m else cond).strip()

        self.assertEqual(
            _expr(fallback),
            f"!({_expr(prebuilt)})",
            "the fallback's `if:` must be the exact negation of the prebuilt step's, so "
            "every runner takes exactly one branch. Note the `${{ }}` wrapper is "
            "REQUIRED around a leading `!` — bare `!` is a YAML tag indicator, not a "
            "boolean negation.",
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
