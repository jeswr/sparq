#!/usr/bin/env python3
# [OPUS-5] sq-0lk — INSPECTION test for the rustfmt GATE posture in ci.yml.
#
# WHY a test at all. `cargo fmt --all --check` ran for the whole life of this repo
# under `continue-on-error: true`, because the workspace predated `rustfmt.toml` and
# the check failed on files no PR had touched. sq-0lk ran the one-time
# `cargo fmt --all` reformat (773 files / 61 crates) and removed that line, so the
# check now hard-fails like clippy beside it.
#
# The thing that makes this worth pinning is that EVERY way of undoing it is silent:
#
#   * re-adding `continue-on-error: true` to the step (or the job) turns a red leg
#     green and NOTHING reports it — this is the exact exit-zero swallowing shape
#     scripts/tests/test_preflight.py already pins for docs-quality.yml;
#   * putting "informational" / "advisory" back into the job's display name is caught
#     by check-advisory-registry.py C2 — but "non-blocking", the token this job
#     actually carried, is NOT in C2's regex, so restoring the old name specifically
#     is caught by nobody. All three are rejected here, because every leg in this job
#     gates and any of the three is now simply false;
#   * declaring the job in .github/advisory-registry.json would drop the whole
#     lint job — clippy `-D warnings` INCLUDED — out of the ci-summary gate set;
#   * dropping `rustfmt` from rust-toolchain.toml's `components` would put the gate
#     back on whatever formatter the machine happens to carry, which is issue #2360
#     verbatim (a workspace-wide failure on unchanged files). The version pin is not
#     decoration here: it is the precondition that makes gating on format SAFE.
#
# None of those five reds anything on its own, so all five are pinned here.
#
# What this test deliberately does NOT assert: that the tree is currently
# rustfmt-clean. That is not a property of the YAML, it is the job's own output —
# `cargo fmt --all --check` in the `lint` job is what checks it, on every PR.
#
# Hermetic: stdlib + PyYAML (installed once in docs-quality.yml's shared setup).
# No subprocess, no network, no gh, no cargo.
# Run:  python3 scripts/tests/test_fmt_gate_posture.py

import json
import pathlib
import re
import tomllib  # Python 3.11+, which every lane that runs this file already uses
import unittest

import yaml

REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
CI_YML = REPO_ROOT / ".github" / "workflows" / "ci.yml"
REGISTRY = REPO_ROOT / ".github" / "advisory-registry.json"
TOOLCHAIN = REPO_ROOT / "rust-toolchain.toml"

# The command whose posture this file exists to protect.
FMT_CHECK = "cargo fmt --all --check"

# ci-summary's pre-#3773 name rule, plus the "non-blocking" spelling the job used to
# carry. check-advisory-registry.py C2 keys on the first two; the third is included
# because it is what this job was called and is equally misleading now.
ADVISORY_TOKEN_RE = re.compile(r"\b(advisory|informational|non-blocking)\b", re.IGNORECASE)


def _lint_job():
    doc = yaml.safe_load(CI_YML.read_text(encoding="utf-8"))
    return doc["jobs"]["lint"]


def _fmt_step():
    for step in _lint_job().get("steps", []):
        if FMT_CHECK in str(step.get("run", "")):
            return step
    return None


class TestRustfmtLegGates(unittest.TestCase):
    def test_the_fmt_check_is_actually_wired(self):
        # If this fails the rest of the file is vacuous, so it is asserted first.
        self.assertIsNotNone(
            _fmt_step(), f"no step in ci.yml's `lint` job runs `{FMT_CHECK}`"
        )

    def test_the_fmt_leg_cannot_swallow_its_own_failure(self):
        # continue-on-error at EITHER the job or the step level turns a red leg
        # green. The step-level one is what sq-0lk removed.
        job, step = _lint_job(), _fmt_step()
        self.assertNotEqual(
            job.get("continue-on-error"), True,
            "the `lint` job is continue-on-error — every leg in it, clippy included, "
            "is swallowed",
        )
        self.assertNotEqual(
            step.get("continue-on-error"), True,
            f"the step running `{FMT_CHECK}` is continue-on-error — sq-0lk removed "
            "exactly this line; re-adding it silently un-gates formatting",
        )

    def test_the_fmt_leg_is_not_conditionally_skipped(self):
        # An `if:` on the step is the cheapest way to make a gate vacuous.
        self.assertIsNone(
            _fmt_step().get("if"),
            f"the step running `{FMT_CHECK}` carries an `if:` — it can be skipped",
        )


class TestLintJobStaysInTheGateSet(unittest.TestCase):
    def test_the_job_name_carries_no_advisory_token(self):
        name = str(_lint_job().get("name") or "lint")
        self.assertIsNone(
            ADVISORY_TOKEN_RE.search(name),
            f"the `lint` job is named {name!r}, which carries an advisory/informational/"
            "non-blocking token. Every leg in this job GATES (clippy -D warnings, the "
            "two rustdoc passes, bench/dict, and — since sq-0lk — cargo fmt), so any of "
            "the three is false. `advisory`/`informational` would also trip "
            "check-advisory-registry.py C2; `non-blocking` is not in C2's regex, which "
            "is why it is rejected here.",
        )

    def test_the_job_is_not_declared_advisory(self):
        # A declaration here is the ONLY thing that can make a check-run non-gating
        # (sparq-org/sparq#3773), and declaring THIS job would take the clippy
        # `-D warnings` gate down with the fmt leg.
        name = str(_lint_job().get("name") or "lint")
        declared = json.loads(REGISTRY.read_text(encoding="utf-8"))["jobs"]
        lowered = {k.lower() for k in declared}
        self.assertNotIn(
            name.lower(), lowered,
            f"{name!r} is declared in .github/advisory-registry.json — that excludes "
            "the WHOLE lint job (clippy -D warnings included) from the ci-summary gate",
        )


class TestTheFormatterVersionStaysPinned(unittest.TestCase):
    def test_rustfmt_is_a_pinned_toolchain_component(self):
        # Issue #2360: with an ambient formatter the same `cargo fmt --all --check`
        # produced different diffs on different boxes. Gating on format is only safe
        # while rustup materialises rustfmt from the pinned channel for every cargo
        # invocation inside this checkout.
        doc = tomllib.loads(TOOLCHAIN.read_text(encoding="utf-8"))
        toolchain = doc["toolchain"]
        self.assertIn(
            "rustfmt", toolchain.get("components", []),
            "rust-toolchain.toml no longer ships the `rustfmt` component — the "
            "`cargo fmt --all --check` gate would fall back to whatever formatter the "
            "machine carries, which is issue #2360",
        )
        self.assertTrue(
            str(toolchain.get("channel", "")).strip(),
            "rust-toolchain.toml declares no channel — the rustfmt component alone "
            "does not pin a formatter VERSION",
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
