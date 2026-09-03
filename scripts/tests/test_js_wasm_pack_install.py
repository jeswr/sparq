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
# SCOPE — properties 1–4 are deliberately js.yml ONLY. gui.yml / site-e2e-hero.yml /
# site-visual.yml / nightly-full-sweep.yml / pages.yml / publish.yml still `cargo install`
# wasm-pack; converting them is separate work and is NOT asserted here, so this file's
# silence about them is not a claim that they are hardened.
#
# release.yml is the ONE other lane asserted, by ReleaseGuiBundleWasmPackInstall below:
#
#   6. THE RELEASE GUI BUNDLE USES AN ARCH-AWARE INSTALLER. #5772: release.yml's
#      `gui-bundle` matrix has arm64 rows (`ubuntu-24.04-arm`, BLOCKING; `macos-14`), and
#      jetli/wasm-pack-action cannot serve them — its dist/index.js switches on
#      `process.platform` alone and always requests the `x86_64-*` asset, so on arm64
#      Linux it installs a binary the runner cannot execute. That lane therefore installs
#      wasm-pack through taiki-e/install-action, whose wasm-pack manifest carries
#      `aarch64_linux_musl`/`aarch64_macos` entries. Reverting it to `cargo install
#      wasm-pack --locked` (its previous state) would work on every row and stay green on
#      a good day — it just puts the crates.io source-compile flake surface back on a
#      BLOCKING row of a release build — and swapping in jetli would break the arm64 rows
#      outright. Neither goes red on its own, hence this assertion.
#
# One CROSS-WORKFLOW property is asserted, by WasmPackVersionUnified below:
#
#   5. ONE VERSION REPO-WIDE. Every workflow that installs wasm-pack through a PINNED
#      installer — jetli/wasm-pack-action (`with: version: vX.Y.Z`) or
#      taiki-e/install-action (`with: tool: wasm-pack@X.Y.Z`) — must name exactly one
#      exact version (read indentation-aware, so a `version:`/`tool:` under a sibling
#      mapping does not count as a pin), and they must all request the SAME one (a step
#      with none takes the installer's default — the latest release — i.e. it is an
#      unpinned lane, which is the same regression in a different shape). Both shapes are
#      normalised to `vX.Y.Z` before comparison, so the two installers cannot skew past
#      each other. #5771: ci.yml's `wasm`
#      job — the lane that RUNS the headless `wasm-pack test --node` suites — pinned
#      v0.13.1 while js.yml — the lane that BUILDS the published artifact — pinned
#      v0.15.0, so a wasm-pack/wasm-bindgen behaviour change between those releases was
#      exercised in the build lane and never in the test lane. Nothing goes red when the
#      two drift apart again, hence this assertion. It says nothing about WHICH version
#      is right; bumping is fine, bumping one lane only is not.
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
RELEASE_YML = WORKFLOWS / "release.yml"

WASM_PACK_ACTION = "jetli/wasm-pack-action"
# The arch-aware alternative (#5772): selects the download target from arch AND
# platform, and names the tool via `with: tool: <name>[@<version>]`.
INSTALL_ACTION = "taiki-e/install-action"
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


def _with_values(block: list[str], want: str) -> list[str]:
    """Every value of a `<want>:` key that is a DIRECT child of the step's `with:`
    mapping, in order (so duplicates are visible to the caller).

    Scoping matters: only a direct child of `with:` reaches the action as an input. A
    bare "any stripped line starting `version:`" scan would also read a `version:`
    living under some sibling mapping (`env:`, a matrix entry, a nested input object),
    so a step that dropped its `with.version` — and therefore silently takes the
    action's default, the #5771 regression — could still look pinned. The
    indentation-aware walk mirrors `check-install-action-tool.py`'s `_has_with_tool`.
    """
    values: list[str] = []
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
        if key.startswith(f"{want}:"):
            values.append(key.split(":", 1)[1].strip())
    return values


def _with_versions(block: list[str]) -> list[str]:
    """`with: version:` — jetli/wasm-pack-action's version input."""
    return _with_values(block, "version")


def _install_action_wasm_pack_pin(block: list[str]) -> str | None:
    """The wasm-pack version a taiki-e/install-action step pins, normalised to the
    `vX.Y.Z` form jetli uses — or None when the step does not install wasm-pack at all,
    or a DESCRIPTIVE SENTINEL when it installs wasm-pack without naming one exact
    version.

    install-action takes a CSV `with: tool:` list whose entries are `<name>[@<version>]`
    (`tool: cargo-llvm-cov,nextest`, `tool: mdbook@0.4.40`). A bare `tool: wasm-pack`
    installs the LATEST release — the unpinned lane #5771 is about, in this installer's
    shape — so it returns a sentinel rather than None: None would drop the lane out of
    the repo-wide comparison entirely and let the remaining lanes agree vacuously.

    A step with no parseable `tool:` at all returns None: it is not identifiable as a
    wasm-pack installer here, and `scripts/check-install-action-tool.py` already reds a
    SHA-pinned install-action step that omits `tool:`.
    """
    values = _with_values(block, "tool")
    entries = [e.strip() for v in values for e in v.split(",") if e.strip()]
    matches = [e for e in entries if e.split("@", 1)[0] == "wasm-pack"]
    if not matches:
        return None
    if len(values) != 1 or len(matches) != 1:
        # Duplicate `with: tool:` keys (YAML last-key-wins) or wasm-pack named twice:
        # which release the lane installs is not what a reader sees.
        return f"<not exactly one wasm-pack `with: tool:` entry: {values!r}>"
    _, _, version = matches[0].partition("@")
    if not version:
        return "<`tool: wasm-pack` with no @version: installs the latest release>"
    return version if version.startswith("v") else f"v{version}"


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
        versions = _with_versions(block)
        self.assertEqual(
            len(versions),
            1,
            "the wasm-pack step must carry exactly one `with: version:` input, found "
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


class ReleaseGuiBundleWasmPackInstall(unittest.TestCase):
    """(6) release.yml's `gui-bundle` job installs wasm-pack with an ARCH-AWARE,
    prebuilt-binary installer — #5772.

    The job's matrix spans x86_64 and arm64 rows, and the arm64 Linux row is BLOCKING
    (`soft: false`). Two regressions are pinned here because neither reds on its own:

      * `cargo install wasm-pack --locked` (this lane's previous state) works on every
        row but recompiles wasm-pack's whole source tree from crates.io per row, so a
        transient registry blip fails a release build.
      * jetli/wasm-pack-action — correct for the x86_64-only `js`/`wasm` lanes — ignores
        `process.arch` and always fetches the `x86_64-*` asset, so on `ubuntu-24.04-arm`
        it installs an unrunnable binary with no Rosetta to fall back on.
    """

    @classmethod
    def setUpClass(cls):
        cls.text = RELEASE_YML.read_text(encoding="utf-8")
        cls.code = _code_lines(cls.text)

    def _wasm_pack_steps(self) -> list[list[str]]:
        return [
            b
            for b in gate.split_steps(self.text)
            if _install_action_wasm_pack_pin(b) is not None
            or (gate._step_uses(b) or (None,))[0] == WASM_PACK_ACTION
        ]

    def test_matrix_still_has_arm64_rows(self):
        """Anti-vacuity: the requirement only exists because the job builds on arm64.

        If `gui-bundle` ever drops both arm64 rows the assertions below stop describing
        anything real — they should be re-derived, not left passing by luck."""
        for row in ("ubuntu-24.04-arm", "macos-14"):
            self.assertTrue(
                any(row in ln for ln in self.code),
                f"release.yml's gui-bundle matrix must still carry the {row} row — the "
                "arch-aware-installer requirement below is about exactly these rows "
                "(#5772).",
            )

    def test_no_cargo_install_wasm_pack(self):
        """The crates.io source compile — the flake surface — must be gone."""
        offenders = [ln for ln in self.code if "cargo install wasm-pack" in ln]
        self.assertEqual(
            offenders,
            [],
            "release.yml must NOT `cargo install wasm-pack`: recompiling it (and its "
            "chrono/wasm-bindgen source tree) from crates.io on every row of the "
            "gui-bundle matrix puts a transient-registry-blip failure on a BLOCKING row "
            f"of a release build (#5772). Use {INSTALL_ACTION} `with: tool: "
            "wasm-pack@<version>` instead.",
        )

    def _the_step(self) -> list[str]:
        """The one wasm-pack install step, or a clean failure. Shared so that a
        regression which REMOVES the step (e.g. reverting to `cargo install`) reports
        that fact in every test instead of raising IndexError out of one of them."""
        steps = self._wasm_pack_steps()
        self.assertEqual(
            len(steps),
            1,
            "release.yml must have exactly one wasm-pack install step (a "
            f"{INSTALL_ACTION} step with `with: tool: wasm-pack@<version>`), found "
            f"{len(steps)}",
        )
        return steps[0]

    def test_installs_via_arch_aware_sha_pinned_action(self):
        """Exactly one wasm-pack install, through the SHA-pinned arch-aware action."""
        action, ref = gate._step_uses(self._the_step())
        self.assertEqual(
            action,
            INSTALL_ACTION,
            f"release.yml's gui-bundle must install wasm-pack via {INSTALL_ACTION}, "
            f"not {action}. {WASM_PACK_ACTION} switches on `process.platform` alone and "
            "always requests the x86_64 asset, so it hands the BLOCKING "
            "`ubuntu-24.04-arm` row a binary it cannot execute (#5772).",
        )
        self.assertRegex(
            ref,
            SHA_RE,
            f"{INSTALL_ACTION} must be pinned to a 40-hex commit SHA (repo action-pin "
            f"policy), got {ref!r}",
        )

    def test_version_is_pinned_exactly(self):
        """An exact `wasm-pack@X.Y.Z` — a bare `tool: wasm-pack` takes the latest."""
        pin = _install_action_wasm_pack_pin(self._the_step())
        self.assertIsNotNone(
            pin,
            "release.yml's wasm-pack step names no version this test can read — it is "
            f"not a {INSTALL_ACTION} step carrying `with: tool: wasm-pack@<version>` "
            "(see the arch-aware-installer assertion above, #5772).",
        )
        self.assertRegex(
            pin,
            VERSION_RE,
            "release.yml must pin wasm-pack to an exact release (`tool: "
            f"wasm-pack@0.15.0`), got {pin!r}. A bare `tool: wasm-pack` installs "
            "whatever is latest at run time, so the shipped desktop bundles are built "
            "with an unpinned toolchain (#5771/#5772).",
        )


def _step_version(block: list[str]) -> str:
    """The wasm-pack version one step requests, or a DESCRIPTIVE SENTINEL when the
    step does not carry exactly one non-empty `with: version:` input.

    Only a `version:` that is a direct child of the step's `with:` mapping counts
    (see `_with_versions`): that is the one the action actually receives. A duplicate
    `with.version` is a sentinel too — YAML last-key-wins makes the effective pin
    ambiguous to a reader, and the repo-wide comparison should not have to guess.

    Returning a sentinel rather than nothing is what keeps the cross-workflow
    comparison honest. A step that LOST its `version:` installs whatever the action
    defaults to — the same unpinned-lane regression #5771 is about — but if such a
    step simply contributed no entry, the remaining lanes would still agree and both
    assertions below would pass. As a sentinel it reads as a pin unlike any other, so
    it reds `test_single_version_across_workflows` (and, being no `vX.Y.Z`,
    `test_every_step_pins_an_exact_version` too).
    """
    versions = _with_versions(block)
    exact = [v for v in versions if v]
    if len(exact) != 1 or len(versions) != 1:
        return f"<not exactly one `with: version:` input: {versions!r}>"
    return exact[0]


class WasmPackVersionUnified(unittest.TestCase):
    """(5) Every pinned-installer wasm-pack step in the repo asks for the SAME version,
    and every such step actually asks for one.

    ci.yml's `wasm` job RUNS the headless suites; js.yml BUILDS + packs the npm
    package's wasm artifact; release.yml's `gui-bundle` builds the wasm bundle embedded
    in every shipped desktop installer. If they skew, a wasm-pack/wasm-bindgen behaviour
    change is only ever exercised in some of them (#5771). This pins the EQUALITY, not
    the value — bumping is fine, bumping one lane only is not. Dropping a lane's
    `version:`/`@version` altogether is the same regression wearing a different hat
    (that lane silently takes the installer's default), so it is caught too.

    BOTH installer shapes count, normalised to `vX.Y.Z`: jetli/wasm-pack-action
    (`with: version:`) and taiki-e/install-action (`with: tool: wasm-pack@…`). release.yml
    must use the latter because jetli cannot serve its arm64 rows (#5772) — but a lane
    that changes installer must not thereby leave this comparison.
    """

    @staticmethod
    def _pins_from(sources: dict[str, str]) -> dict[str, list[str]]:
        """{workflow filename: [one entry PER wasm-pack step]}.

        Per-step, not flattened-over-values: a step with no `version:` must still
        occupy a slot (as a `_step_version` sentinel) instead of disappearing.
        Split out from `_pins` so the mutation guard below can feed it synthetic
        workflows without touching the tree.
        """
        found: dict[str, list[str]] = {}
        for name, text in sorted(sources.items()):
            if "wasm-pack" not in text:
                continue
            for block in gate.split_steps(text):
                action = (gate._step_uses(block) or (None,))[0]
                if action == WASM_PACK_ACTION:
                    found.setdefault(name, []).append(_step_version(block))
                elif action == INSTALL_ACTION:
                    # Most install-action steps install some other tool; only the ones
                    # that name wasm-pack join the comparison (#5772).
                    pin = _install_action_wasm_pack_pin(block)
                    if pin is not None:
                        found.setdefault(name, []).append(pin)
        return found

    @classmethod
    def _pins(cls) -> dict[str, list[str]]:
        return cls._pins_from(
            {
                path.name: path.read_text(encoding="utf-8")
                for path in sorted(WORKFLOWS.glob("*.y*ml"))
            }
        )

    def test_both_wasm_lanes_use_the_action(self):
        """Anti-vacuity: the lanes this unification covers must still be in the sample.

        ci.yml + js.yml are the two #5771 unified; release.yml joined via the
        install-action shape (#5772) and must not fall out of the comparison just
        because it pins through a different installer."""
        pins = self._pins()
        for expected in ("ci.yml", "js.yml", "release.yml"):
            self.assertIn(
                expected,
                pins,
                f"{expected} must install wasm-pack via a PINNED installer "
                f"({WASM_PACK_ACTION} or {INSTALL_ACTION} `with: tool: wasm-pack@…`) — "
                "without it the single-version assertion below has one fewer lane to "
                "compare, and with none it passes vacuously (#5771/#5772).",
            )

    def test_every_step_pins_an_exact_version(self):
        """Every wasm-pack step names an exact release — an omitted `version:` takes
        the action's default, which is exactly the unpinned lane #5771 removed."""
        for name, versions in sorted(self._pins().items()):
            for i, version in enumerate(versions):
                self.assertRegex(
                    version,
                    VERSION_RE,
                    f"{name}: {WASM_PACK_ACTION} step #{i} must carry exactly one "
                    f"exact `version:` (e.g. v0.15.0), got {version!r}. A step "
                    "without one installs whatever the action defaults to, so the "
                    "lane is unpinned and the repo-wide version agreement below "
                    "means nothing for it (#5771).",
                )

    def test_single_version_across_workflows(self):
        pins = self._pins()
        distinct = {v for versions in pins.values() for v in versions}
        self.assertEqual(
            len(distinct),
            1,
            "all workflows must install the SAME wasm-pack version, found "
            f"{sorted(distinct)} across { {k: v for k, v in sorted(pins.items())} }. "
            "ci.yml's `wasm` job RUNS the headless suites and js.yml BUILDS the "
            "published artifact: if they skew, a wasm-pack/wasm-bindgen behaviour "
            "change is only ever exercised in the build lane (#5771). Bump every "
            "lane together.",
        )

    # --- mutation guard for the two assertions above -------------------------
    # Hermetic synthetic workflows (no tree mutation): `_MUT_BAD` is `_MUT_GOOD`
    # with the `version:` line deleted, i.e. the exact regression the reviewer
    # described.
    _MUT_GOOD = """\
jobs:
  wasm:
    steps:
      - name: Install wasm-pack
        uses: jetli/wasm-pack-action@0d096b08b4e5a7de8c28de67e11e945404e9eefa # v0.4.0
        with:
          version: v0.15.0
      - name: Test
        run: wasm-pack test --node
"""
    _MUT_BAD = """\
jobs:
  wasm:
    steps:
      - name: Install wasm-pack
        uses: jetli/wasm-pack-action@0d096b08b4e5a7de8c28de67e11e945404e9eefa # v0.4.0
      - name: Test
        run: wasm-pack test --node
"""

    # The step has NO `with: version:` (so wasm-pack-action takes its default) but does
    # carry an unrelated, correctly-named `version:` under a SIBLING mapping. A
    # block-wide "any line starting `version:`" scan reads v0.15.0 here and calls the
    # lane pinned; only the `with:`-scoped walk sees the omission.
    _MUT_UNRELATED_VERSION = """\
jobs:
  wasm:
    steps:
      - name: Install wasm-pack
        uses: jetli/wasm-pack-action@0d096b08b4e5a7de8c28de67e11e945404e9eefa # v0.4.0
        with:
          cache-key: wasm-pack
        env:
          version: v0.15.0
      - name: Test
        run: wasm-pack test --node
"""

    # Two direct `with: version:` keys: YAML last-key-wins, so which release the lane
    # actually installs is not what a reader (or the first match) sees.
    _MUT_DUPLICATE_VERSION = """\
jobs:
  wasm:
    steps:
      - name: Install wasm-pack
        uses: jetli/wasm-pack-action@0d096b08b4e5a7de8c28de67e11e945404e9eefa # v0.4.0
        with:
          version: v0.15.0
          version: v0.13.1
      - name: Test
        run: wasm-pack test --node
"""

    def test_mutation_dropping_a_lanes_version_is_caught(self):
        """Deleting one lane's `version:` must NOT leave both assertions green."""
        good = self._pins_from({"a.yml": self._MUT_GOOD, "b.yml": self._MUT_GOOD})
        self.assertEqual(good, {"a.yml": ["v0.15.0"], "b.yml": ["v0.15.0"]})
        self.assertEqual(len({v for vs in good.values() for v in vs}), 1)

        mutated = self._pins_from({"a.yml": self._MUT_GOOD, "b.yml": self._MUT_BAD})
        # The unpinned lane is still IN the sample (so the anti-vacuity check keeps
        # passing, as the reviewer noted) — it must therefore fail on its own terms.
        self.assertIn("b.yml", mutated)
        self.assertNotRegex(
            mutated["b.yml"][0],
            VERSION_RE,
            "a step with no `version:` must not read as an exact pin",
        )
        self.assertEqual(
            len({v for vs in mutated.values() for v in vs}),
            2,
            "a step with no `version:` must count as a DISTINCT pin, so the "
            "single-version assertion goes red instead of comparing the one "
            "surviving lane against itself",
        )

    def test_mutation_version_outside_with_is_not_a_pin(self):
        """A `version:` that the action never receives must not read as a pin."""
        for label, text in (
            ("unrelated nested `version:`", self._MUT_UNRELATED_VERSION),
            ("duplicate `with: version:`", self._MUT_DUPLICATE_VERSION),
        ):
            with self.subTest(mutation=label):
                mutated = self._pins_from(
                    {"a.yml": self._MUT_GOOD, "b.yml": text}
                )
                self.assertIn("b.yml", mutated)
                self.assertNotRegex(
                    mutated["b.yml"][0],
                    VERSION_RE,
                    f"{label}: the step does not unambiguously pass one `version:` "
                    "input to the action, so it must not read as an exact pin",
                )
                self.assertEqual(
                    len({v for vs in mutated.values() for v in vs}),
                    2,
                    f"{label}: must count as a DISTINCT pin so the single-version "
                    "assertion goes red",
                )

    # --- the install-action shape (#5772) ------------------------------------
    _MUT_IA_GOOD = """\
jobs:
  gui-bundle:
    steps:
      - name: Install wasm-pack
        uses: taiki-e/install-action@18b1216eba7f8039b0f8d131d5473787f0edce68 # v2.85.3
        with:
          tool: wasm-pack@0.15.0
"""
    # A bare `tool: wasm-pack` takes install-action's default: the latest release.
    _MUT_IA_UNPINNED = """\
jobs:
  gui-bundle:
    steps:
      - name: Install wasm-pack
        uses: taiki-e/install-action@18b1216eba7f8039b0f8d131d5473787f0edce68 # v2.85.3
        with:
          tool: wasm-pack
"""
    # A CSV `tool:` list — wasm-pack alongside another tool, at a SKEWED version.
    _MUT_IA_SKEWED_CSV = """\
jobs:
  gui-bundle:
    steps:
      - name: Install tools
        uses: taiki-e/install-action@18b1216eba7f8039b0f8d131d5473787f0edce68 # v2.85.3
        with:
          tool: cargo-llvm-cov,wasm-pack@0.13.1
"""
    # An install-action step that installs something else entirely: must not join the
    # wasm-pack comparison at all (dozens of these exist across the tree).
    _MUT_IA_OTHER_TOOL = """\
jobs:
  coverage:
    steps:
      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@18b1216eba7f8039b0f8d131d5473787f0edce68 # v2.85.3
        with:
          tool: cargo-llvm-cov
"""

    def test_install_action_pin_is_normalised_and_compared(self):
        """`tool: wasm-pack@0.15.0` must read as the SAME pin as `version: v0.15.0`.

        Without the normalisation the two installers would each be self-consistent and
        skew freely past each other — which is #5771 again, one installer swap later."""
        agreeing = self._pins_from(
            {"js.yml": self._MUT_GOOD, "release.yml": self._MUT_IA_GOOD}
        )
        self.assertEqual(
            agreeing, {"js.yml": ["v0.15.0"], "release.yml": ["v0.15.0"]}
        )
        self.assertEqual(len({v for vs in agreeing.values() for v in vs}), 1)

    def test_mutation_install_action_regressions_are_caught(self):
        """Each way the install-action lane can lose its exact pin must go red."""
        for label, text in (
            ("bare `tool: wasm-pack`", self._MUT_IA_UNPINNED),
            ("skewed version in a CSV `tool:` list", self._MUT_IA_SKEWED_CSV),
        ):
            with self.subTest(mutation=label):
                mutated = self._pins_from(
                    {"js.yml": self._MUT_GOOD, "release.yml": text}
                )
                self.assertIn(
                    "release.yml",
                    mutated,
                    f"{label}: the lane must stay IN the sample (dropping out would "
                    "let the surviving lane agree with itself vacuously)",
                )
                self.assertEqual(
                    len({v for vs in mutated.values() for v in vs}),
                    2,
                    f"{label}: must count as a DISTINCT pin so the single-version "
                    "assertion goes red",
                )
        # And the unpinned one is additionally not an exact version.
        unpinned = self._pins_from({"release.yml": self._MUT_IA_UNPINNED})
        self.assertNotRegex(unpinned["release.yml"][0], VERSION_RE)

    def test_install_action_steps_for_other_tools_are_ignored(self):
        """Anti-false-positive: the tree is full of install-action steps for other
        tools; pulling them into the wasm-pack comparison would red it immediately."""
        self.assertEqual(self._pins_from({"ci.yml": self._MUT_IA_OTHER_TOOL}), {})


if __name__ == "__main__":
    unittest.main(verbosity=2)
