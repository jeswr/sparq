#!/usr/bin/env python3
# [OPUS-5] sq-khm3f (#1131) / #5771 / #5305 — INSPECTION test for HOW this repo puts
# wasm-pack on PATH, and for the fact that it does so in exactly one way.
#
# WHY A TEST AT ALL: every property below is a single YAML line, and every failure
# mode is INVISIBLE — drop any one of them and CI stays green, on a good day, while
# a real guarantee is gone. Nothing reds, so the posture is pinned here instead.
#
# The properties, and the incident each one is standing on:
#
#   1. ONE SOURCE OF TRUTH FOR THE VERSION (#5305).
#      wasm-pack is installed by 18 steps across 9 workflows. They used to disagree:
#      two named a version via jetli/wasm-pack-action (ci.yml said v0.13.1, js.yml
#      said v0.15.0 — the #5771 skew) and sixteen ran a bare `cargo install wasm-pack
#      --locked`, resolving to whatever crates.io's newest release was that day. So
#      the lane that RUNS the headless `wasm-pack test --node` suites and the lanes
#      that BUILD and PUBLISH the artifact users install could each use a different
#      wasm-pack, and therefore a different bundled wasm-bindgen: a behaviour change
#      between releases got exercised in one lane and never the other, with every
#      lane individually green. The version now lives in ONE file,
#      .github/actions/install-wasm-pack/action.yml, and every lane calls it.
#      Asserted by CompositeActionIsTheSingleSourceOfTruth +
#      NoWorkflowBypassesTheCompositeAction.
#
#   2. NO SOURCE COMPILE ON THE GATING LANES (sq-khm3f).
#      `cargo install wasm-pack` compiles wasm-pack and its whole chrono/wasm-bindgen
#      source tree from crates.io on every run, so a transient registry/CDN blip reds
#      a HARD gate on an unrelated PR — observed on PR #1131 ("download of
#      wa/sm/wasm-bindgen failed / curl failed: [16] Error in the HTTP2 framing
#      layer"), cleared by a plain re-run. The prebuilt path fetches one static musl
#      tarball through @actions/tool-cache, which retries a failed download.
#      Asserted by NoWorkflowBypassesTheCompositeAction (no workflow may `cargo
#      install wasm-pack`) — the composite's own non-x64-Linux fallback is the single
#      permitted source install, and it is version-pinned.
#
#   3. PREBUILT BINARY, SHA-PINNED. jetli/wasm-pack-action is pinned to a 40-hex
#      commit SHA, per the repo's action-pin policy.
#
#   4. EXACT VERSION, NEVER `latest`. `latest` re-adds an unauthenticated
#      api.github.com release lookup on every run — a second rate-limit-flake source,
#      i.e. the thing sq-khm3f removed.
#
#   5. ORDERING IN THE `js` LANE. The install still precedes the root `npm ci`,
#      because the package's `prepare` lifecycle (sq-bkag git-pin build) runs on
#      `npm ci` and needs wasm-pack already on PATH to compile the wasm engine.
#
# SCOPE HONESTY: this file says nothing about WHICH version is correct. Bumping is
# fine; bumping one lane only is what it makes impossible.
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
WORKFLOWS = REPO_ROOT / ".github" / "workflows"
JS_YML = WORKFLOWS / "js.yml"

# The ONE file the wasm-pack version is written in, and the `uses:` value every lane
# reaches it by. A local action is referenced by path, so it has no `@ref` — the
# checked-out tree IS the version, which is why no pin is needed on this line.
COMPOSITE_PATH = REPO_ROOT / ".github" / "actions" / "install-wasm-pack" / "action.yml"
COMPOSITE_USES = "./.github/actions/install-wasm-pack"

WASM_PACK_ACTION = "jetli/wasm-pack-action"
# The repo's action-pin policy: a 40-char lowercase hex commit SHA, never a tag.
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
# An exact release pin, e.g. `v0.15.0`. `latest` (and any floating ref) must fail.
VERSION_RE = re.compile(r"^v\d+\.\d+\.\d+$")
# The composite's source-build fallback. `--version <X>` is mandatory: without it the
# fallback resolves to crates.io's newest release, which is the #5305 drift itself.
CARGO_INSTALL_RE = re.compile(
    r"cargo install wasm-pack\b(?P<args>[^\n]*)"
)
CARGO_VERSION_RE = re.compile(r"--version[= ]+(?P<v>\S+)")


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
    old command (this repo comments its workflows heavily, and several of those
    comments quote `cargo install wasm-pack` to explain why it is gone) is never
    mistaken for a live step."""
    out = []
    for raw in text.splitlines():
        stripped = raw.strip()
        if not stripped or stripped.startswith("#"):
            continue
        out.append(raw)
    return out


def _with_versions(block: list[str]) -> list[str]:
    """Every value of a `version:` key that is a DIRECT child of the step's `with:`
    mapping, in order (so duplicates are visible to the caller).

    Scoping matters: `with.version` is the ONLY key that reaches the action as an
    input. A bare "any stripped line starting `version:`" scan would also read a
    `version:` living under some sibling mapping (`env:`, a matrix entry, a nested
    input object), so a step that dropped its `with.version` — and therefore silently
    takes the action's default, the #5771 regression — could still look pinned. The
    indentation-aware walk mirrors `check-install-action-tool.py`'s `_has_with_tool`.
    """
    versions: list[str] = []
    with_indent: int | None = None
    child_indent: int | None = None
    for raw in block:
        if not raw.strip() or raw.lstrip(" ").startswith("#"):
            continue
        # A key introduced on the dash line itself logically begins after the "- ".
        ind = gate._indent(raw)
        key = raw.lstrip(" ")
        if key.startswith("- "):
            ind += 2
            key = key[2:]
        if with_indent is None:
            if key.startswith("with:"):
                with_indent = ind
                child_indent = None
            continue
        if ind <= with_indent:
            # Dedented out of the `with:` mapping (e.g. into a sibling `env:`).
            with_indent = ind if key.startswith("with:") else None
            child_indent = None
            continue
        if child_indent is None:
            child_indent = ind
        if ind != child_indent:
            # Nested deeper than the mapping's own keys — not a `with:` input.
            continue
        if key.startswith("version:"):
            versions.append(key.split(":", 1)[1].strip())
    return versions


def _step_version(block: list[str]) -> str:
    """The wasm-pack version one step requests, or a DESCRIPTIVE SENTINEL when the
    step does not carry exactly one non-empty `with: version:` input.

    Only a `version:` that is a direct child of the step's `with:` mapping counts
    (see `_with_versions`): that is the one the action actually receives. A duplicate
    `with.version` is a sentinel too — YAML last-key-wins makes the effective pin
    ambiguous to a reader, and no assertion should have to guess.

    Returning a sentinel rather than nothing is what keeps the assertions honest. A
    step that LOST its `version:` installs whatever the action defaults to — the
    unpinned-lane regression — but if such a step simply contributed no entry, a
    comparison over the remaining entries would still agree and pass. As a sentinel it
    matches no `vX.Y.Z`, so it reds instead.
    """
    versions = _with_versions(block)
    exact = [v for v in versions if v]
    if len(exact) != 1 or len(versions) != 1:
        return f"<not exactly one `with: version:` input: {versions!r}>"
    return exact[0]


def _wasm_pack_installs(text: str) -> list[tuple[str, list[str]]]:
    """Every step in `text` that puts wasm-pack on PATH, as (kind, block).

    kind is one of:
      "composite" — `uses: ./.github/actions/install-wasm-pack` (the sanctioned route)
      "jetli"     — a direct `uses: jetli/wasm-pack-action@...` (bypasses the pin)
      "cargo"     — a live `run:` that `cargo install`s it (bypasses the pin AND
                    re-adds the sq-khm3f source-compile flake surface)

    Detection is per-STEP rather than per-line because this repo comments its
    workflows heavily and the splitter retains comment lines, so a prose mention of
    an action leaks into the preceding step's block; `_code_lines` is applied to the
    block before the `run:` scan for exactly that reason.
    """
    found: list[tuple[str, list[str]]] = []
    for block in gate.split_steps(text):
        uses = gate._step_uses(block)
        if uses and uses[0] == WASM_PACK_ACTION:
            found.append(("jetli", block))
            continue
        # A local action is `uses: <path>` with no `@ref`, so `_step_uses` (which
        # requires an `@`) does not see it; match the path on the step's own `uses:`.
        if any(
            ln.lstrip(" ").lstrip("- ").startswith("uses:")
            and COMPOSITE_USES in ln
            for ln in _code_lines("\n".join(block))
        ):
            found.append(("composite", block))
            continue
        if any("cargo install wasm-pack" in ln for ln in _code_lines("\n".join(block))):
            found.append(("cargo", block))
    return found


def _workflow_sources() -> dict[str, str]:
    return {
        path.name: path.read_text(encoding="utf-8")
        for path in sorted(WORKFLOWS.glob("*.y*ml"))
    }


class CompositeActionIsTheSingleSourceOfTruth(unittest.TestCase):
    """(1,3,4) The one file the version lives in, and the shape of what it installs."""

    @classmethod
    def setUpClass(cls):
        cls.text = COMPOSITE_PATH.read_text(encoding="utf-8")
        cls.steps = gate.split_steps(cls.text)

    def test_composite_action_exists(self):
        self.assertTrue(
            COMPOSITE_PATH.is_file(),
            f"{COMPOSITE_PATH.relative_to(REPO_ROOT)} is the single place the repo's "
            "wasm-pack version is written (#5305). Every lane `uses:` it; without the "
            "file they all fail to resolve the action.",
        )

    def _prebuilt_step(self) -> list[str]:
        blocks = [
            b
            for b in self.steps
            if (gate._step_uses(b) or (None,))[0] == WASM_PACK_ACTION
        ]
        self.assertEqual(
            len(blocks),
            1,
            f"the composite action must have exactly one {WASM_PACK_ACTION} step, "
            f"found {len(blocks)}",
        )
        return blocks[0]

    def test_prebuilt_step_is_sha_pinned(self):
        """(3) The prebuilt-download action is pinned to a commit SHA, not a tag."""
        action, ref = gate._step_uses(self._prebuilt_step())
        self.assertEqual(action, WASM_PACK_ACTION)
        self.assertRegex(
            ref,
            SHA_RE,
            f"{WASM_PACK_ACTION} must be pinned to a 40-hex commit SHA (repo "
            f"action-pin policy), got {ref!r}",
        )

    def test_prebuilt_version_is_exact_not_latest(self):
        """(4) An exact vX.Y.Z — `latest` re-adds a network lookup to resolve it."""
        version = _step_version(self._prebuilt_step())
        self.assertRegex(
            version,
            VERSION_RE,
            "wasm-pack must be pinned to an exact release (e.g. v0.15.0). `latest` "
            "costs an unauthenticated api.github.com lookup on every run — a second "
            "rate-limit-flake source (sq-khm3f).",
        )

    def test_source_fallback_pins_the_same_version(self):
        """(1) The composite's two install paths must not drift from EACH OTHER.

        The fallback exists for runners with no proven prebuilt asset. An unpinned
        `cargo install wasm-pack` there would resolve to crates.io's newest release —
        reintroducing, inside the very file that is supposed to end it, the exact
        drift #5305 is about.
        """
        live = [
            ln
            for ln in _code_lines(self.text)
            if "cargo install wasm-pack" in ln
        ]
        self.assertEqual(
            len(live),
            1,
            f"expected exactly one source-build fallback command, found {live!r}",
        )
        args = CARGO_INSTALL_RE.search(live[0]).group("args")
        m = CARGO_VERSION_RE.search(args)
        self.assertIsNotNone(
            m,
            "the source-build fallback must pass `--version <X.Y.Z>`; without it it "
            f"resolves to whatever crates.io ships that day (#5305). Got: {live[0].strip()!r}",
        )
        prebuilt = _step_version(self._prebuilt_step())
        self.assertEqual(
            m.group("v"),
            prebuilt.lstrip("v"),
            "the composite's prebuilt pin and its source-build fallback must request "
            f"the SAME wasm-pack: prebuilt asks {prebuilt!r}, fallback asks "
            f"{m.group('v')!r}. Bump both literals together.",
        )


class NoWorkflowBypassesTheCompositeAction(unittest.TestCase):
    """(1,2) Every wasm-pack install in the repo goes through the one pinned route.

    A lane that installs wasm-pack any other way is unpinned by construction, and
    that is invisible: the lane is green, it just used a different wasm-pack than the
    lane that tested the bindings (#5771) or than the lane that published them.
    """

    def test_no_workflow_cargo_installs_wasm_pack(self):
        """(2) The source compile — the actual flake surface — is gone from every lane."""
        offenders = [
            f"{name}: {ln.strip()}"
            for name, text in sorted(_workflow_sources().items())
            for kind, block in _wasm_pack_installs(text)
            if kind == "cargo"
            for ln in _code_lines("\n".join(block))
            if "cargo install wasm-pack" in ln
        ]
        self.assertEqual(
            offenders,
            [],
            "no workflow may `cargo install wasm-pack`: it compiles wasm-pack (and "
            "its whole chrono/wasm-bindgen source tree) from crates.io on every run, "
            "which let a transient registry blip red a gating lane (sq-khm3f, PR "
            "#1131), and unpinned it resolves to a different release than the other "
            f"lanes (#5305). Use `uses: {COMPOSITE_USES}` instead. Offenders: "
            f"{offenders}",
        )

    def test_no_workflow_calls_the_action_directly(self):
        """(1) A direct jetli step carries its own `version:` — a second pin to drift."""
        offenders = [
            name
            for name, text in sorted(_workflow_sources().items())
            for kind, _ in _wasm_pack_installs(text)
            if kind == "jetli"
        ]
        self.assertEqual(
            offenders,
            [],
            f"no workflow may call {WASM_PACK_ACTION} directly: doing so re-adds a "
            "per-lane `with: version:`, which is the drift #5771/#5305 removed. Call "
            f"`uses: {COMPOSITE_USES}` — the one file the version lives in. "
            f"Offenders: {offenders}",
        )

    def test_key_lanes_still_install_wasm_pack(self):
        """Anti-vacuity. Both assertions above are satisfied by a repo that installs
        wasm-pack NOWHERE, so name the lanes whose skew motivated this: ci.yml RUNS
        the headless suites, js.yml BUILDS the artifact the `js` gate packs, and
        publish.yml builds the artifact actually published to npm. If any drops out
        of the sample, the guarantee has silently narrowed."""
        via_composite = {
            name
            for name, text in _workflow_sources().items()
            for kind, _ in _wasm_pack_installs(text)
            if kind == "composite"
        }
        for expected in ("ci.yml", "js.yml", "publish.yml"):
            self.assertIn(
                expected,
                via_composite,
                f"{expected} must install wasm-pack via `uses: {COMPOSITE_USES}` — "
                "without it the single-source assertions above have nothing to "
                "constrain and pass vacuously (#5771, #5305). Lanes currently on the "
                f"composite: {sorted(via_composite)}",
            )


class JsLaneInstallOrdering(unittest.TestCase):
    """(5) `prepare` runs on `npm ci` and needs wasm-pack already on PATH."""

    def test_install_precedes_npm_ci(self):
        code = _code_lines(JS_YML.read_text(encoding="utf-8"))
        install_at = [i for i, ln in enumerate(code) if COMPOSITE_USES in ln]
        npm_ci_at = [
            i for i, ln in enumerate(code) if re.match(r"^\s*run:\s*npm ci\s*$", ln)
        ]
        self.assertEqual(
            len(install_at), 1, "one wasm-pack install step expected in js.yml"
        )
        self.assertTrue(npm_ci_at, "js.yml must still run the root `npm ci`")
        self.assertLess(
            install_at[0],
            min(npm_ci_at),
            "wasm-pack must be installed BEFORE `npm ci`: the package's `prepare` "
            "lifecycle (sq-bkag) runs on `npm ci` and compiles the wasm engine with "
            "wasm-pack from PATH.",
        )


class DetectionGuard(unittest.TestCase):
    """The assertions above are only as good as `_wasm_pack_installs`. Feed it the
    exact regressions it exists to catch — as synthetic text, touching no file — and
    prove it classifies each one rather than passing it through as unremarkable."""

    def _kinds(self, text: str) -> list[str]:
        return [kind for kind, _ in _wasm_pack_installs(text)]

    def test_detects_a_reverted_cargo_install(self):
        self.assertEqual(
            self._kinds(
                "jobs:\n  x:\n    steps:\n"
                "      - name: Install wasm-pack\n"
                "        run: cargo install wasm-pack --locked\n"
            ),
            ["cargo"],
        )

    def test_detects_a_direct_action_call(self):
        self.assertEqual(
            self._kinds(
                "jobs:\n  x:\n    steps:\n"
                "      - name: Install wasm-pack\n"
                f"        uses: {WASM_PACK_ACTION}@" + "0" * 40 + "\n"
                "        with:\n          version: v0.13.1\n"
            ),
            ["jetli"],
        )

    def test_accepts_the_composite_route(self):
        self.assertEqual(
            self._kinds(
                "jobs:\n  x:\n    steps:\n"
                "      - name: Install wasm-pack\n"
                f"        uses: {COMPOSITE_USES}\n"
            ),
            ["composite"],
        )

    def test_a_commented_out_command_is_not_a_live_step(self):
        """The repo's workflow comments quote `cargo install wasm-pack` to explain
        why it is gone; reading those as offences would make the gate unfixable."""
        self.assertEqual(
            self._kinds(
                "jobs:\n  x:\n    steps:\n"
                "      # do NOT `cargo install wasm-pack --locked` here\n"
                "      - name: Install wasm-pack\n"
                f"        uses: {COMPOSITE_USES}\n"
            ),
            ["composite"],
        )

    def test_a_step_that_lost_its_version_reads_as_unpinned(self):
        """`_step_version`'s sentinel must not look like an exact release."""
        block = [
            "      - uses: " + WASM_PACK_ACTION + "@" + "0" * 40,
            "        with:",
            "          args: --no-opt",
        ]
        self.assertNotRegex(_step_version(block), VERSION_RE)


if __name__ == "__main__":
    unittest.main(verbosity=2)
