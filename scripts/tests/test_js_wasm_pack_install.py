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
# SCOPE — properties 1–4 are deliberately js.yml ONLY. gui.yml / nightly-full-sweep.yml /
# pages.yml / publish.yml / site-e2e-hero.yml / site-visual.yml still `cargo install`
# wasm-pack; converting them is separate work and is NOT asserted here, so this file's
# silence about them is not a claim that they are hardened. (Property 6 below covers
# release.yml's `gui-bundle` only, for the arch-specific reason given there.)
#
# Two CROSS-WORKFLOW properties are asserted, by WasmPackVersionUnified and
# ReleaseGuiBundleArchAware below:
#
#   5. ONE VERSION REPO-WIDE. Every workflow that installs wasm-pack through
#      jetli/wasm-pack-action must pass the action exactly one `with: version:` input
#      (read indentation-aware, so a `version:` under a sibling mapping does not count
#      as a pin), and they must all request the SAME one (a step with none takes the
#      action's default, i.e. it is an unpinned lane, which is the same regression in
#      a different shape). #5771: ci.yml's `wasm`
#      job — the lane that RUNS the headless `wasm-pack test --node` suites — pinned
#      v0.13.1 while js.yml — the lane that BUILDS the published artifact — pinned
#      v0.15.0, so a wasm-pack/wasm-bindgen behaviour change between those releases was
#      exercised in the build lane and never in the test lane. Nothing goes red when the
#      two drift apart again, hence this assertion. It says nothing about WHICH version
#      is right; bumping is fine, bumping one lane only is not.
#
#   6. ARCH-AWARE INSTALL WHERE THE MATRIX IS NOT ALL-x86_64 (#5772).
#      jetli/wasm-pack-action is x86_64-ONLY: its dist/index.js switches on
#      `process.platform` alone and always requests x86_64-pc-windows-msvc /
#      x86_64-apple-darwin / x86_64-unknown-linux-musl — `process.arch` is never
#      consulted — even though upstream publishes aarch64 assets. release.yml's
#      `gui-bundle` is the one wasm-pack-installing job whose matrix is not all-x64
#      (`ubuntu-24.04-arm`, a BLOCKING row with no Rosetta, and the arm64 `macos-14`),
#      so it must install from an ARCH-AWARE source — taiki-e/install-action, which
#      resolves per (arch, platform) from its bundled manifest. That job therefore must
#      not `cargo install wasm-pack` (the crates.io source-compile flake surface
#      properties 1–2 exist to remove) and must not reach for the x86_64-only action
#      either. Both regressions are silent: a source compile is green on a good day, and
#      the x86_64-only action would only fail once a release tag actually runs the job.
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
# The arch-aware alternative: resolves the asset per (arch, platform) from a bundled,
# checksum-pinned manifest, so it is the only one usable on a non-x86_64 row (#5772).
INSTALL_ACTION = "taiki-e/install-action"
# The job whose matrix is not all-x86_64 — see property 6 in the header.
ARCH_MIXED_JOB = "gui-bundle"
# The repo's action-pin policy: a 40-char lowercase hex commit SHA, never a tag.
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
# An exact release pin, e.g. `v0.15.0`. `latest` (and any floating ref) must fail.
VERSION_RE = re.compile(r"^v\d+\.\d+\.\d+$")
# install-action names the tool it installs as `<tool>[@<version>]`; an exact
# `wasm-pack@x.y.z` is required for the same reason `latest` is banned above.
TOOL_RE = re.compile(r"^wasm-pack@(\d+\.\d+\.\d+)$")
# Runner labels this recognises as arm64: GitHub's `*-arm` Linux images say so in the
# label, and the plain macOS 14+ images are arm64 by DEFAULT (the x86_64 variant is
# spelled out — `macos-15-intel` — so the anchored alternation must not match it).
# Deliberately an ALLOWLIST, not a classifier: it does not know about the sized images
# (`macos-14-xlarge` is arm64, `macos-14-large` is not). Being incomplete only ever
# UNDER-counts arm rows, and the one assertion using it requires at least one — so an
# unrecognised label reds the anti-vacuity check rather than passing it wrongly.
ARM64_RUNNER_RE = re.compile(r"(?:-arm(?:64)?$)|(?:^macos-(?:1[4-9]|latest)$)")


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
    """Every value of the `want:` key that is a DIRECT child of the step's `with:`
    mapping, in order (so duplicates are visible to the caller).

    Scoping matters: only a direct child of `with:` reaches the action as an input. A
    bare "any stripped line starting `<key>:`" scan would also read one living under
    some sibling mapping (`env:`, a matrix entry, a nested input object), so a step
    that dropped its `with.version` — and therefore silently takes the action's
    default, the #5771 regression — could still look pinned. The indentation-aware
    walk mirrors `check-install-action-tool.py`'s `_has_with_tool`.
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
    """`with: version:` inputs — the pin jetli/wasm-pack-action reads."""
    return _with_values(block, "version")


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
    """(5) Every jetli/wasm-pack-action step in the repo asks for the SAME version,
    and every such step actually asks for one.

    ci.yml's `wasm` job RUNS the headless suites; js.yml BUILDS + packs the npm
    package's wasm artifact. If they skew, a wasm-pack/wasm-bindgen behaviour change is
    only ever exercised in the build lane (#5771). This pins the EQUALITY, not the
    value — bumping is fine, bumping one lane only is not. Dropping a lane's `version:`
    altogether is the same regression wearing a different hat (that lane silently takes
    the action's default), so it is caught too.
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
            if WASM_PACK_ACTION not in text:
                continue
            for block in gate.split_steps(text):
                if (gate._step_uses(block) or (None,))[0] != WASM_PACK_ACTION:
                    continue
                found.setdefault(name, []).append(_step_version(block))
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
        """Anti-vacuity: the two lanes #5771 unified must still be in the sample."""
        pins = self._pins()
        for expected in ("ci.yml", "js.yml"):
            self.assertIn(
                expected,
                pins,
                f"{expected} must install wasm-pack via {WASM_PACK_ACTION} — without "
                "it the single-version assertion below has nothing to compare and "
                "passes vacuously (#5771).",
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


def _job_lines(text: str, job: str) -> list[str]:
    """The lines of one `jobs:` entry — from its `  <job>:` key up to the next key at
    the SAME indent (the next job).

    Job-scoped rather than file-scoped because release.yml installs wasm-pack in
    exactly one job and property 6 is a statement about THAT job's runner matrix; a
    file-wide scan would wrongly constrain any future all-x64 job in the same file.

    A DEDENTED COMMENT ends the block too. This repo comments its workflows heavily
    and a job's own comments live inside its body (indented past the job key), so a
    comment back at the job-key column is the next job's banner — release.yml's is
    literally the line after `gui-bundle` ends. Should some future job break that
    convention the slice truncates early, which is fail-CLOSED: the assertions below
    require the matrix AND the install step to be found, so a short slice reds rather
    than passing on a job it never read.
    """
    out: list[str] = []
    key_indent: int | None = None
    for raw in text.splitlines():
        stripped = raw.strip()
        if key_indent is None:
            if stripped == f"{job}:":
                key_indent = gate._indent(raw)
                out.append(raw)
            continue
        if stripped and gate._indent(raw) <= key_indent:
            break  # dedented to a sibling job (or out of `jobs:` entirely)
        out.append(raw)
    return out


def _matrix_runners(job_text: str) -> list[str]:
    """Every `os:` value in the job's matrix rows, comments stripped."""
    return re.findall(
        r"\bos:\s*([A-Za-z0-9._-]+)", "\n".join(_code_lines(job_text))
    )


def _wasm_pack_posture(job_text: str) -> dict[str, list]:
    """How one job puts wasm-pack on PATH, as three separately-assertable facts.

    Split out from the assertions so the mutation guard can feed it a synthetic job
    and prove each regression is actually DETECTED rather than merely absent today.
    """
    source_compiles = [
        ln.strip()
        for ln in _code_lines(job_text)
        if "cargo install wasm-pack" in ln
    ]
    x86_only: list[str] = []
    arch_aware: list[tuple[str, str]] = []
    for block in gate.split_steps(job_text):
        uses = gate._step_uses(block)
        if uses is None:
            continue
        action, ref = uses
        if action == WASM_PACK_ACTION:
            x86_only.append(ref)
        elif action == INSTALL_ACTION:
            # install-action installs many tools; only the wasm-pack one is ours.
            arch_aware += [
                (ref, tool)
                for tool in _with_values(block, "tool")
                if tool.split("@", 1)[0] == "wasm-pack"
            ]
    return {
        "source_compiles": source_compiles,
        "x86_only": x86_only,
        "arch_aware": arch_aware,
    }


class ReleaseGuiBundleArchAware(unittest.TestCase):
    """(6) release.yml's `gui-bundle` — the one wasm-pack job with arm64 rows —
    installs wasm-pack from an arch-aware source (#5772).

    Before this, the job ran `cargo install wasm-pack --locked` on every row: the
    crates.io source compile properties 1–2 exist to remove, on a matrix whose Linux
    rows are BLOCKING. The obvious conversion — reuse jetli/wasm-pack-action like the
    js/ci lanes — is WRONG here: that action never reads `process.arch`, so on
    `ubuntu-24.04-arm` it would fetch an x86_64 binary that cannot execute (no
    Rosetta). Both mistakes are invisible on a PR — release.yml is tag/dispatch
    triggered — so the posture is pinned here instead.
    """

    @classmethod
    def setUpClass(cls):
        cls.job = "\n".join(
            _job_lines(RELEASE_YML.read_text(encoding="utf-8"), ARCH_MIXED_JOB)
        )
        cls.posture = _wasm_pack_posture(cls.job)

    def test_the_job_matrix_still_has_an_arm64_row(self):
        """Anti-vacuity: the arch-awareness requirement only exists BECAUSE this job
        has a non-x86_64 row. If the matrix ever loses them the rest of this class is
        arguing about nothing, and should be re-derived rather than left passing."""
        runners = _matrix_runners(self.job)
        self.assertTrue(
            runners,
            f"release.yml's `{ARCH_MIXED_JOB}` job must still have a runner matrix — "
            "found no `os:` rows, so this class no longer inspects what it claims to",
        )
        arm = [r for r in runners if ARM64_RUNNER_RE.search(r)]
        self.assertTrue(
            arm,
            f"expected at least one arm64 row in `{ARCH_MIXED_JOB}` (e.g. "
            f"ubuntu-24.04-arm, macos-14), got {runners}. #5772's whole premise is "
            "that this matrix is not all-x86_64.",
        )

    def test_no_source_compile(self):
        """(1) applied to this job: the crates.io flake surface must be gone."""
        self.assertEqual(
            self.posture["source_compiles"],
            [],
            f"release.yml's `{ARCH_MIXED_JOB}` must NOT `cargo install wasm-pack`: "
            "recompiling it and its chrono/wasm-bindgen source tree from crates.io on "
            "every row puts a BLOCKING release row one transient registry blip from "
            f"red (#5772). Install the prebuilt binary via {INSTALL_ACTION} instead.",
        )

    def test_does_not_use_the_x86_64_only_action(self):
        """The x86_64-only action is what makes this job different from js/ci."""
        self.assertEqual(
            self.posture["x86_only"],
            [],
            f"release.yml's `{ARCH_MIXED_JOB}` must NOT install wasm-pack via "
            f"{WASM_PACK_ACTION}: it switches on `process.platform` alone and always "
            "requests an x86_64 asset, so the arm64 rows would download a binary they "
            f"cannot run (#5772). Use {INSTALL_ACTION}, which resolves per (arch, "
            "platform).",
        )

    def test_installs_via_sha_pinned_install_action_at_an_exact_version(self):
        """(2)+(3) applied to this job, via the arch-aware installer."""
        arch_aware = self.posture["arch_aware"]
        self.assertEqual(
            len(arch_aware),
            1,
            f"release.yml's `{ARCH_MIXED_JOB}` must have exactly one "
            f"{INSTALL_ACTION} step installing wasm-pack, found {arch_aware!r}",
        )
        ref, tool = arch_aware[0]
        self.assertRegex(
            ref,
            SHA_RE,
            f"{INSTALL_ACTION} must be pinned to a 40-hex commit SHA (repo "
            f"action-pin policy), got {ref!r}",
        )
        self.assertRegex(
            tool,
            TOOL_RE,
            "the `with: tool:` input must name an exact release "
            "(`wasm-pack@x.y.z`), got {!r}. A bare `wasm-pack` takes whatever "
            "install-action's manifest calls latest, which is the same floating pin "
            "`latest` is banned for above.".format(tool),
        )

    def test_version_matches_the_other_wasm_pack_lanes(self):
        """Same equality property as (5), across the two installer kinds: this job
        BUILDS the wasm bundle the desktop GUI ships, so it must not skew from the
        lanes that build + test the published one."""
        arch_aware = self.posture["arch_aware"]
        self.assertEqual(
            len(arch_aware),
            1,
            "no single arch-aware wasm-pack install to compare — see "
            f"test_installs_via_sha_pinned_install_action_at_an_exact_version "
            f"(found {arch_aware!r})",
        )
        here = TOOL_RE.match(arch_aware[0][1])
        self.assertIsNotNone(here, f"unparseable tool pin {arch_aware[0][1]!r}")
        jetli = {
            v.lstrip("v")
            for versions in WasmPackVersionUnified._pins().values()
            for v in versions
        }
        self.assertEqual(
            jetli,
            {here.group(1)},
            f"release.yml's `{ARCH_MIXED_JOB}` pins wasm-pack {here.group(1)} while "
            f"the {WASM_PACK_ACTION} lanes pin {sorted(jetli)}. Bump every lane "
            "together — a wasm-pack/wasm-bindgen behaviour change exercised in one "
            "lane only is exactly what #5771 removed.",
        )

    # --- mutation guard ------------------------------------------------------
    # Synthetic jobs (no tree mutation). `_MUT_ARCH_AWARE` is the shape shipped;
    # the other two are the two regressions this class exists to catch, and each
    # must be DETECTED by `_wasm_pack_posture`, not merely absent from the tree.
    _MUT_ARCH_AWARE = """\
jobs:
  gui-bundle:
    strategy:
      matrix:
        include:
          - { os: ubuntu-latest, label: x64-linux, soft: false }
          - { os: ubuntu-24.04-arm, label: arm64-linux, soft: false }
    steps:
      # A prose mention of `cargo install wasm-pack --locked` and of
      # jetli/wasm-pack-action, as the real file carries — neither may be read as live.
      - name: Install wasm-pack (prebuilt, arch-aware)
        uses: taiki-e/install-action@18b1216eba7f8039b0f8d131d5473787f0edce68 # v2.85.3
        with:
          tool: wasm-pack@0.15.0
  other-job:
    steps:
      - run: cargo install wasm-pack --locked
"""
    _MUT_SOURCE_COMPILE = _MUT_ARCH_AWARE.replace(
        """      - name: Install wasm-pack (prebuilt, arch-aware)
        uses: taiki-e/install-action@18b1216eba7f8039b0f8d131d5473787f0edce68 # v2.85.3
        with:
          tool: wasm-pack@0.15.0
""",
        """      - name: Install wasm-pack
        run: cargo install wasm-pack --locked
""",
    )
    _MUT_X86_ONLY_ACTION = _MUT_ARCH_AWARE.replace(
        """        uses: taiki-e/install-action@18b1216eba7f8039b0f8d131d5473787f0edce68 # v2.85.3
        with:
          tool: wasm-pack@0.15.0
""",
        """        uses: jetli/wasm-pack-action@0d096b08b4e5a7de8c28de67e11e945404e9eefa # v0.4.0
        with:
          version: v0.15.0
""",
    )

    def _synthetic(self, text: str) -> dict[str, list]:
        return _wasm_pack_posture("\n".join(_job_lines(text, ARCH_MIXED_JOB)))

    def test_mutation_source_compile_is_detected(self):
        good = self._synthetic(self._MUT_ARCH_AWARE)
        self.assertEqual(good["source_compiles"], [])
        self.assertEqual(good["x86_only"], [])
        self.assertEqual(len(good["arch_aware"]), 1)

        bad = self._synthetic(self._MUT_SOURCE_COMPILE)
        self.assertTrue(
            bad["source_compiles"],
            "reverting the step to `cargo install wasm-pack --locked` must be "
            "detected — otherwise test_no_source_compile passes vacuously",
        )
        self.assertEqual(bad["arch_aware"], [])

    def test_mutation_x86_only_action_is_detected(self):
        bad = self._synthetic(self._MUT_X86_ONLY_ACTION)
        self.assertEqual(
            bad["x86_only"],
            ["0d096b08b4e5a7de8c28de67e11e945404e9eefa"],
            "swapping in the x86_64-only action must be detected — that is the "
            "regression #5772 is specifically about",
        )
        self.assertEqual(bad["arch_aware"], [])

    def test_mutation_job_scoping_and_comments(self):
        """Two ways this could pass/fail for the wrong reason: reading a SIBLING
        job's steps, and reading a `#`-commented mention as a live step. The real
        release.yml exercises the second directly — the converted step is preceded by
        a comment naming BOTH the old `cargo install wasm-pack --locked` command and
        the x86_64-only action, so a comment-blind scan would red on the shipped,
        correct file."""
        good = self._synthetic(self._MUT_ARCH_AWARE)
        self.assertEqual(
            good["source_compiles"],
            [],
            "the sibling `other-job`'s `cargo install wasm-pack`, and the commented "
            "mentions inside the job, must not be attributed to `gui-bundle`",
        )
        self.assertEqual(
            _matrix_runners("\n".join(_job_lines(self._MUT_ARCH_AWARE, "other-job"))),
            [],
            "job slicing must stop at the next sibling job key",
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
