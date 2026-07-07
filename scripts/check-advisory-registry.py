#!/usr/bin/env python3
# [SONNET-4.6] sq-qcnn.32 — advisory-job registry + gate-intent placement check.
# 🤖 SPARQ agent.
#
# Checks TWO directions across .github/workflows/*.yml:
#
#  C2: every job whose NAME matches \b(advisory|informational)\b must have an entry
#      in .github/advisory-registry.json with {owner_bead, promotion_criteria,
#      registered}.
#
#  C3: no run-step that invokes a gate-classified script (scripts/*gate*.py) may live
#      inside an advisory-named job without an explicit `gate_script_waiver` entry in
#      the registry for that job.
#
# MOTIVATION: #1679 (gate-intent G3 self-test sat in an advisory job = provably
# non-gating until caught); #1656 (gui-mock-ipc gated without recorded probation
# evidence). C2/C3 become diff-visible instead of review-catch-dependent.
#
# ADVISORY-JOB DEFINITION: a job whose `name:` field (at 4-space indent) contains the
# whole word "advisory" or "informational" (case-insensitive, same regex the
# ci-summary aggregator uses: \b(advisory|informational)\b). Workflow-level `name:` at
# indent 0 is NOT checked — only job-level `name:` values.
#
# GATE-CLASSIFIED SCRIPTS: any path matching `scripts/[^ ]*gate[^ ]*\.py` on a run:
# step, e.g. mutants-gate.py, coverage-gate.py, perf-gate.py, unsafe-gate.py,
# ci_summary_gate.py, gate-new-crate.py, gate-api-skill.py. This is the glob
# equivalent of "scripts/*gate*.py". G-numbered gate scripts (gate-new-crate.py,
# gate-api-skill.py) match because their filename contains "gate".
#
# REGISTRY: .github/advisory-registry.json (seeded by sq-qcnn.32).
# Each key is the job's `name:` value as written in the workflow YAML (unexpanded,
# including any ${{ matrix.xxx }} expressions). Required fields per entry:
#   owner_bead         — the bead responsible for tracking this advisory job
#   promotion_criteria — what must be true before the advisory token is removed
#   registered         — ISO-8601 date the entry was first recorded
# Optional:
#   gate_script_waiver — present iff the job legitimately runs a gate-classified
#                        script; its value is a human-readable justification
#
# Usage:
#   check-advisory-registry.py              # check live workflows (default)
#   check-advisory-registry.py --root DIR   # use DIR as repo root
#   check-advisory-registry.py --self-test  # hermetic negative fixtures (both classes)
#
# Exit 0 = all clean; exit 1 = one or more offences.
# stdlib-only.

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# Same regex as ci_summary_gate.py ADVISORY_RE — must stay in sync.
ADVISORY_RE = re.compile(r"\b(advisory|informational)\b", re.IGNORECASE)

# Gate-classified script pattern: path containing "gate" in the filename.
# Matches perf-gate.py, coverage-gate.py, mutants-gate.py, unsafe-gate.py,
# ci_summary_gate.py, gate-new-crate.py, gate-api-skill.py, etc.
GATE_SCRIPT_RE = re.compile(r"scripts/[^\s]*gate[^\s]*\.py")

# Required fields in every registry entry.
REQUIRED_FIELDS = ("owner_bead", "promotion_criteria", "registered")


# ---------------------------------------------------------------------------
# Lightweight GitHub Actions workflow parser (stdlib-only)
# ---------------------------------------------------------------------------

def _job_name_from_block(block: str, fallback_id: str) -> str:
    """Extract the `name:` value from a job block (4-space indent).

    Returns the fallback job-ID if no `name:` is present.
    Handles both quoted and unquoted values, including ${{ matrix.xxx }} expressions.
    """
    m = re.search(r"^    name:\s+(.+?)(?:\s*#.*)?$", block, re.MULTILINE)
    if not m:
        return fallback_id
    val = m.group(1).rstrip()
    # Strip surrounding quote pair (double or single)
    if len(val) >= 2 and val[0] == val[-1] and val[0] in ('"', "'"):
        val = val[1:-1]
    return val


def parse_jobs(text: str) -> list[dict]:
    """Parse a GitHub Actions workflow file into a list of job dicts.

    Each dict has:
      id         — the YAML key under `jobs:` (e.g. "markdownlint")
      name       — the job's `name:` value, or the id if absent
      block_text — the raw text of the job block (for gate-script scanning)

    Only the `jobs:` top-level section is parsed; other sections are ignored.
    Relies on the fixed 2-space indentation of job IDs in GitHub Actions YAML.
    """
    # Find the start of the jobs: section.
    m = re.search(r"^jobs:\s*$", text, re.MULTILINE)
    if not m:
        return []
    jobs_text = text[m.end():]

    # Find all 2-space-indented job keys (exactly 2 spaces, then a YAML key).
    # Pattern: start of line, exactly 2 spaces, identifier chars, colon,
    # optional whitespace/comment.
    job_key_re = re.compile(
        r"^  ([a-zA-Z][a-zA-Z0-9_-]*):\s*(?:#.*)?$", re.MULTILINE
    )

    matches = list(job_key_re.finditer(jobs_text))
    if not matches:
        return []

    results = []
    for i, match in enumerate(matches):
        job_id = match.group(1)
        start = match.start()
        end = matches[i + 1].start() if i + 1 < len(matches) else len(jobs_text)
        block_text = jobs_text[start:end]
        job_name = _job_name_from_block(block_text, job_id)
        results.append({"id": job_id, "name": job_name, "block_text": block_text})

    return results


def find_gate_scripts_in_block(block_text: str) -> list[str]:
    """Return all gate-classified script paths found in run: steps of a job block."""
    return GATE_SCRIPT_RE.findall(block_text)


# ---------------------------------------------------------------------------
# Registry loading
# ---------------------------------------------------------------------------

def load_registry(root: Path) -> dict:
    """Load and return the advisory-registry.json `jobs` mapping.

    Returns an empty dict if the file does not exist (so the check
    fails with C2 offences rather than crashing).
    """
    registry_path = root / ".github" / "advisory-registry.json"
    if not registry_path.exists():
        return {}
    try:
        data = json.loads(registry_path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return {}
    return data.get("jobs", {})


# ---------------------------------------------------------------------------
# Main check
# ---------------------------------------------------------------------------

def check_workflows(root: Path) -> list[str]:
    """Scan all workflow files under root/.github/workflows.

    Returns a list of human-readable offence descriptions (empty = clean).
    Each entry is prefixed with C2 or C3 to identify the violated direction.
    """
    registry = load_registry(root)
    wf_dir = root / ".github" / "workflows"
    if not wf_dir.is_dir():
        return [f"C2: .github/workflows/ not found under {root}"]

    offences: list[str] = []

    for path in sorted(wf_dir.iterdir()):
        if path.suffix not in (".yml", ".yaml"):
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue

        for job in parse_jobs(text):
            job_name = job["name"]
            if not ADVISORY_RE.search(job_name):
                continue  # Not an advisory/informational job.

            # C2: must have a registry entry with all required fields.
            entry = registry.get(job_name)
            if entry is None:
                offences.append(
                    f"C2: {path.name}: job {job_name!r} is advisory/informational "
                    f"but has no entry in .github/advisory-registry.json"
                )
            else:
                missing = [f for f in REQUIRED_FIELDS if not entry.get(f)]
                if missing:
                    offences.append(
                        f"C2: {path.name}: job {job_name!r} registry entry is missing "
                        f"required fields: {missing}"
                    )

            # C3: no gate-classified script without a waiver.
            gate_scripts = find_gate_scripts_in_block(job["block_text"])
            if gate_scripts:
                has_waiver = bool(
                    entry.get("gate_script_waiver") if entry is not None else False
                )
                if not has_waiver:
                    offences.append(
                        f"C3: {path.name}: advisory job {job_name!r} invokes "
                        f"gate-classified script(s) {gate_scripts} without a "
                        f"`gate_script_waiver` in the registry"
                    )

    return offences


# ---------------------------------------------------------------------------
# Self-test — hermetic negative fixtures
# ---------------------------------------------------------------------------

# C2 negative fixture: an advisory job NOT present in a minimal registry.
_FIXTURE_C2_WORKFLOW = """\
name: test-workflow
on: [pull_request]
jobs:
  my-check:
    name: some check (advisory)
    runs-on: ubuntu-latest
    steps:
      - run: echo hello
"""

# C2 clean fixture: the same job IS in the registry.
_FIXTURE_C2_REGISTRY_CLEAN = {
    "some check (advisory)": {
        "workflow": "test.yml",
        "owner_bead": "sq-test",
        "promotion_criteria": "once clean",
        "registered": "2026-01-01",
    }
}

# C3 negative fixture: a gate-classified script inside an advisory job with
# NO waiver in the registry.
_FIXTURE_C3_WORKFLOW = """\
name: test-workflow
on: [pull_request]
jobs:
  ratchet:
    name: mutation ratchet (cargo-mutants, advisory)
    runs-on: ubuntu-latest
    steps:
      - run: |
          python3 scripts/mutants-gate.py --check outcomes.json
"""

# C3 registry entry WITHOUT a gate_script_waiver → C3 violation.
_FIXTURE_C3_REGISTRY_NO_WAIVER = {
    "mutation ratchet (cargo-mutants, advisory)": {
        "workflow": "ci.yml",
        "owner_bead": "sq-qcnn.11",
        "promotion_criteria": "sq-qcnn.27",
        "registered": "2026-01-01",
        # No gate_script_waiver
    }
}

# C3 registry entry WITH a gate_script_waiver → clean.
_FIXTURE_C3_REGISTRY_WITH_WAIVER = {
    "mutation ratchet (cargo-mutants, advisory)": {
        "workflow": "ci.yml",
        "owner_bead": "sq-qcnn.11",
        "promotion_criteria": "sq-qcnn.27",
        "registered": "2026-01-01",
        "gate_script_waiver": "intentional; the ratchet runs advisory while seeding",
    }
}

# A non-advisory job running a gate script must NOT be flagged.
_FIXTURE_NON_ADVISORY = """\
name: test-workflow
on: [pull_request]
jobs:
  hard-gate:
    name: coverage ratchet (HARD)
    runs-on: ubuntu-latest
    steps:
      - run: python3 scripts/coverage-gate.py --check
"""

# A job with `informational` in the name must also be caught.
_FIXTURE_INFORMATIONAL_WORKFLOW = """\
name: test-workflow
on: [pull_request]
jobs:
  proofs:
    name: kani proofs (bounded, informational)
    runs-on: ubuntu-latest
    steps:
      - run: cargo kani
"""

# An advisory job that runs a non-gate script must NOT trigger C3.
_FIXTURE_C3_CLEAN_NO_GATE = """\
name: test-workflow
on: [pull_request]
jobs:
  lint:
    name: markdownlint (advisory)
    runs-on: ubuntu-latest
    steps:
      - run: npx markdownlint-cli2
"""

# C2 — missing required field in registry entry.
_FIXTURE_C2_MISSING_FIELD_REGISTRY = {
    "some check (advisory)": {
        "workflow": "test.yml",
        # owner_bead missing
        "promotion_criteria": "once clean",
        "registered": "2026-01-01",
    }
}


def _run_check(workflow_text: str, registry: dict) -> list[str]:
    """Run the check on synthetic workflow text + a registry dict.

    Used only in self-test. Returns offences list.
    """
    offences: list[str] = []
    for job in parse_jobs(workflow_text):
        job_name = job["name"]
        if not ADVISORY_RE.search(job_name):
            continue
        entry = registry.get(job_name)
        if entry is None:
            offences.append(f"C2: job {job_name!r} unregistered")
        else:
            missing = [f for f in REQUIRED_FIELDS if not entry.get(f)]
            if missing:
                offences.append(
                    f"C2: job {job_name!r} missing fields: {missing}"
                )
        gate_scripts = find_gate_scripts_in_block(job["block_text"])
        if gate_scripts:
            has_waiver = bool(
                entry.get("gate_script_waiver") if entry is not None else False
            )
            if not has_waiver:
                offences.append(
                    f"C3: advisory job {job_name!r} runs gate scripts "
                    f"{gate_scripts} without waiver"
                )
    return offences


def self_test() -> int:
    """Hermetic self-test exercising both violation classes and their negatives.

    Returns 0 if all cases pass, 1 otherwise.
    """
    cases: list[tuple[str, str, dict, int, str]] = [
        # (label, workflow_text, registry, expected_offence_count, description)
        (
            "C2-negative: unregistered advisory job",
            _FIXTURE_C2_WORKFLOW,
            {},
            1,
            "an advisory job with no registry entry must produce a C2 offence",
        ),
        (
            "C2-clean: registered advisory job",
            _FIXTURE_C2_WORKFLOW,
            _FIXTURE_C2_REGISTRY_CLEAN,
            0,
            "a properly registered advisory job must produce no offence",
        ),
        (
            "C2-negative: registry entry missing required field",
            _FIXTURE_C2_WORKFLOW,
            _FIXTURE_C2_MISSING_FIELD_REGISTRY,
            1,
            "a registry entry missing owner_bead must produce a C2 offence",
        ),
        (
            "C3-negative: gate script in advisory job, no waiver",
            _FIXTURE_C3_WORKFLOW,
            _FIXTURE_C3_REGISTRY_NO_WAIVER,
            1,
            "a gate script inside an advisory job without waiver must produce a C3 offence",
        ),
        (
            "C3-clean: gate script in advisory job, with waiver",
            _FIXTURE_C3_WORKFLOW,
            _FIXTURE_C3_REGISTRY_WITH_WAIVER,
            0,
            "a gate script inside an advisory job WITH waiver must produce no offence",
        ),
        (
            "non-advisory HARD job with gate script: no offence",
            _FIXTURE_NON_ADVISORY,
            {},
            0,
            "a HARD (non-advisory) job running a gate script must never be flagged",
        ),
        (
            "informational job: also caught by C2",
            _FIXTURE_INFORMATIONAL_WORKFLOW,
            {},
            1,
            "an unregistered job with 'informational' in the name must produce a C2 offence",
        ),
        (
            "C3-clean: advisory job runs non-gate script",
            _FIXTURE_C3_CLEAN_NO_GATE,
            {
                "markdownlint (advisory)": {
                    "workflow": "test.yml",
                    "owner_bead": "sq-test",
                    "promotion_criteria": "once clean",
                    "registered": "2026-01-01",
                }
            },
            0,
            "an advisory job running a non-gate script must not trigger C3",
        ),
    ]

    failures = 0
    for label, wf_text, registry, expected_count, description in cases:
        offences = _run_check(wf_text, registry)
        got = len(offences)
        ok = got == expected_count
        status = "PASS" if ok else "FAIL"
        print(f"  [{status}] {label}: {got} offence(s) (want {expected_count})")
        if not ok:
            print(f"         [{description}]")
            for o in offences:
                print(f"         offence: {o}")
            failures += 1

    if failures:
        print(f"\nself-test: {failures} case(s) FAILED")
        return 1
    print("\nself-test: all cases PASS")
    return 0


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=(
            "sq-qcnn.32: advisory-job registry + gate-intent placement check (C2/C3)"
        )
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=REPO_ROOT,
        help="repo root (default: inferred from script location)",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="run hermetic negative fixtures and exit",
    )
    args = parser.parse_args(argv)

    if args.self_test:
        return self_test()

    offences = check_workflows(args.root)
    if not offences:
        print("check-advisory-registry: all clear (C2 + C3)")
        return 0

    print(f"check-advisory-registry: {len(offences)} offence(s):\n")
    for o in offences:
        print(f"  {o}")
    print()
    print(
        "  C2: add the job to .github/advisory-registry.json with "
        "{owner_bead, promotion_criteria, registered}."
    )
    print(
        "  C3: add a gate_script_waiver to its registry entry, or move the "
        "gate-classified script to a HARD (non-advisory) job."
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
