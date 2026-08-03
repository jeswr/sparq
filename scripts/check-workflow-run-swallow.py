#!/usr/bin/env python3
# [OPUS-5] sparq-org/sparq#5799 — the RUN-BODY half of the step-level swallow seam.
# 🤖 SPARQ agent.
#
# WHY THIS EXISTS
# ===============
# `ci-summary / gate` requires every check-run not DECLARED in
# .github/advisory-registry.json, and scripts/check-advisory-registry.py keeps that
# registry honest. Both operate on the JOB. A step that reports success because its
# `run:` body threw its own failure away is invisible to both: the step is green, the
# job is green, the gate is green, and whatever the command was supposed to verify was
# never verified.
#
# The declarative spelling of that (`continue-on-error: true`) is a separate concern.
# This file covers the shell spelling, which is the one with no YAML key to grep for:
#
#     python3 scripts/coverage-gate.py || true     # gate cannot fail
#     set +e                                       # ...nor can anything below here
#     cargo deny check advisories
#
# WHY IT IS NOT A BAN
# ===================
# #5799 asked for an inventory first, and the inventory is the whole reason this file
# is shaped the way it is. Over .github/workflows/*.yml there are ~36 `|| true` /
# `set +e` / `exit 0` sites inside jobs that GATE, and reading every one of them: they
# are `grep … || true` (grep exits 1 on no-match, which is not a failure), best-effort
# `git rebase --abort` / `pkill` cleanups, diagnostic `ls`/`tail`/`cat`, best-effort
# `gh issue edit` labelling, and documented early `exit 0`s in skip guards. Requiring a
# waiver for each would be ~36 entries of pure noise, and a checker that cries wolf 36
# times is a checker whose 37th finding gets waived without being read.
#
# So the rule keys on the COMMAND, not on the idiom: a swallow only matters when what
# is being swallowed is something that VERIFIES. "Verifies" is not re-invented here —
# it is scripts/check-advisory-registry.py's own gate classifier (a script filename
# carrying `gate`/`ratchet`/`tripwire`/`guard`, in any language, including through an
# `npm run` alias), loaded by path so the two cannot drift, WIDENED by the verification
# binaries that carry no such filename token and so would otherwise be invisible:
# `cargo test|nextest|clippy|deny|audit|miri|fmt|vet|semver-checks`, `pytest`,
# `actionlint`, `shellcheck`, a `scripts/tests/` suite, and any `--self-test`.
#
# WHAT COUNTS AS DISCARDING THE FAILURE
# =====================================
#   D1  `… || true` / `… || :` on the same logical line as the command. This is the
#       discard idiom proper. `… || rc=$?` is NOT this: it captures the status, and
#       ci.yml's nextest shard relies on exactly that to re-raise a genuine failure.
#   D2  The command sits in an errexit-OFF region (`set +e`, un-armed until the next
#       `set -e`) and the next effective line does not capture `$?`. Capturing is the
#       disciplined form — bench.yml wraps perf-gate.py in `set +e` precisely so it can
#       read rc and re-raise on anything but the "unchanged" code.
#   D3  The body's last top-level statement is a bare, unindented `exit 0`, which forces
#       the whole step green whatever ran above it. Indented `exit 0`s are early-exit
#       guards inside an `if`/`case` and are not this.
#
# RULES
# =====
#   R1  NO UNDECLARED SWALLOW. A gate-classified command discarded by D1/D2/D3 inside a
#       job that GATES needs an entry in .github/workflow-run-swallow-waivers.json. A
#       job is exempt only if its `name:` is DECLARED in .github/advisory-registry.json
#       — the same single source of truth ci_summary_gate.py uses — because a declared
#       advisory job already carries an owner_bead and promotion_criteria.
#   R2  NO STALE WAIVER. Every waiver must still bind to a live finding. A waiver that
#       matches nothing has stopped documenting anything and would silently pre-approve
#       the next swallow added to that step.
#
# WHAT IT DOES NOT CHECK — read this before trusting it further than it goes
# =========================================================================
#   * A captured status is assumed to be propagated. `cmd; rc=$?` exempts the command,
#     and whether the body then acts on `rc` is a dataflow question a line-oriented
#     checker cannot answer. D3 is what catches the specific case where it demonstrably
#     is not propagated *by this step* (dependency-monitoring.yml's audit, waived: a
#     later step in the same job re-raises it from `steps.deny.outputs.exit_code`).
#   * The gate-command set is a list, so a verification binary nobody has used here yet
#     is not classified. Widening it is a one-line edit; the classifier is deliberately
#     precise because a false positive is an obligation imposed on an unrelated author.
#   * Like every registry in this tree, the waiver file is data next to the code it
#     guards: an author can add an entry and this passes. What it buys is that the
#     removal of a gate becomes an explicit, reviewable line in the diff.
#
# Usage:
#   check-workflow-run-swallow.py              # check the live workflows (default)
#   check-workflow-run-swallow.py --root DIR   # use DIR as the repo root
#   check-workflow-run-swallow.py --inventory  # print every swallow site, waived or not
#   check-workflow-run-swallow.py --self-test  # hermetic negative fixtures
#
# EXIT: 0 all clear, 1 findings, 2 usage/parse error.

from __future__ import annotations

import argparse
import importlib.util
import json
import re
import sys
import tempfile
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent

WORKFLOW_DIR = Path(".github") / "workflows"
ADVISORY_REGISTRY = Path(".github") / "advisory-registry.json"
WAIVER_REGISTRY = Path(".github") / "workflow-run-swallow-waivers.json"

WAIVER_REQUIRED_FIELDS = ("workflow", "job_id", "step_name", "command", "why", "registered")


# The gate classifier lives in ONE place — scripts/check-advisory-registry.py, whose C3
# rule already answers "does this run-step invoke something that gates?" across
# python/shell/node and through npm aliases. Loaded by path (scripts/ is not a package)
# so widening that classifier widens this checker too, and neither can drift.
def _load_advisory_checker():
    spec = importlib.util.spec_from_file_location(
        "check_advisory_registry", REPO_ROOT / "scripts" / "check-advisory-registry.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


_adv = _load_advisory_checker()

# A command position in shell: start of line, or after a separator / substitution /
# keyword. Requiring one keeps `echo "cargo test failed" || true` from being read as an
# invocation of cargo test.
_CMD_POS = (
    r"(?:^|[;&|(){}`]|\$\(|(?<![\w-])(?:then|do|else|elif|time|env|sudo|xargs|exec|nohup)\s)\s*"
)

# Verification binaries that carry no gate-ish FILENAME token and so are invisible to
# check-advisory-registry.py's classifier, plus the two guard shapes #5371 identified
# (a scripts/tests/ suite and a `--self-test`). A swallowed one of these is the hole.
VERIFY_RES = (
    ("cargo verification subcommand", re.compile(
        _CMD_POS + r"(?P<cmd>cargo\s+(?:\+[\w.-]+\s+)*"
        r"(?:test|nextest|clippy|deny|audit|miri|fmt|vet|semver-checks|udeps))\b")),
    ("pytest", re.compile(_CMD_POS + r"(?P<cmd>pytest)\b")),
    ("actionlint", re.compile(_CMD_POS + r"(?P<cmd>actionlint)\b")),
    ("shellcheck", re.compile(_CMD_POS + r"(?P<cmd>shellcheck)\b")),
    ("npm test/audit", re.compile(_CMD_POS + r"(?P<cmd>npm\s+(?:\S+\s+)*?(?:test|audit))\b")),
    ("scripts/tests suite", re.compile(r"(?:^|[\s;&|(`])(?P<cmd>[\w./-]*scripts/tests/\S+)")),
    ("--self-test invocation", re.compile(r"(?<![\w-])(?P<cmd>--self-test)(?![\w-])")),
)

# D1 — the discard idiom. `|| true` / `|| :`, terminated so `|| trueish_fn` is not it.
DISCARD_RE = re.compile(r"\|\|\s*(?:true|:)\s*(?=$|[;&|)}])")
# `set` with flag words; the LAST occurrence of e/+e in the flags wins, as in bash.
SET_FLAGS_RE = re.compile(r"^set\s+((?:[-+][A-Za-z]+\s*)+)(?:$|[\w\s\"'-]*$)")
# `rc=$?` / `CODE=$?` / `export rc=$?` — the disciplined capture that exempts D2.
CAPTURE_RE = re.compile(r"^(?:export\s+|local\s+|declare\s+)?[A-Za-z_]\w*=\$\?\s*$")
# D3 — a bare, unindented, unconditional `exit 0`.
BARE_EXIT0_RE = re.compile(r"^exit\s+0\s*(?:;\s*)?$")


def strip_comment(line: str) -> str:
    """Drop a shell comment. Crude but adequate: classification only cares whether a
    real command mentions something that verifies, and a `#`-led line never does."""
    if line.lstrip().startswith("#"):
        return ""
    return re.split(r"(?<=\s)#", line, maxsplit=1)[0].rstrip()


def logical_lines(body: str) -> list[tuple[str, str]]:
    """Split a run body into (raw, stripped) logical lines, joining `\\` continuations.

    A continued command is ONE line to bash, so `cargo test \\\n  --all || true` must be
    seen as a single unit or the discard would be attributed to a line with no command
    on it. `raw` keeps the leading indentation D3 needs; `stripped` is comment-free.
    """
    out: list[tuple[str, str]] = []
    pending_raw: str | None = None
    pending: list[str] = []
    for line in body.splitlines():
        text = strip_comment(line)
        if pending_raw is None:
            pending_raw = line
        if text.endswith("\\"):
            pending.append(text[:-1].rstrip())
            continue
        pending.append(text.strip() if pending else text)
        joined = " ".join(part for part in pending if part).strip()
        out.append((pending_raw, joined))
        pending_raw, pending = None, []
    if pending_raw is not None:
        out.append((pending_raw, " ".join(p for p in pending if p).strip()))
    return out


def initial_errexit(step: dict) -> bool:
    """Is errexit armed when the body starts?

    GitHub's default step shell is `bash --noprofile --norc -eo pipefail {0}`, and
    `shell: sh` is `sh -e {0}` — both armed. A custom template (`shell: bash {0}`) is
    armed only if it spells `-e` itself, and that omission is a swallow vector in its
    own right, so it is read literally rather than assumed safe.
    """
    shell = step.get("shell")
    if shell is None:
        return True
    shell = str(shell).strip()
    if shell in ("bash", "sh", "pwsh", "powershell", "python"):
        return shell in ("bash", "sh")
    return bool(re.search(r"(?<![\w+])-[A-Za-z]*e[A-Za-z]*(?:\s|$)", shell))


def apply_set(line: str, errexit: bool) -> bool:
    """Fold a `set -e` / `set +e` / `set -euo pipefail` into the errexit state."""
    match = SET_FLAGS_RE.match(line)
    if not match:
        return errexit
    for token in match.group(1).split():
        if len(token) < 2 or token[0] not in "-+":
            continue
        if "e" in token[1:]:
            errexit = token[0] == "-"
    return errexit


def gate_commands(line: str, npm_scripts: dict[str, list[str]]) -> list[str]:
    """Every gate-classified invocation on ONE logical line (empty = none).

    The shared classifier first (script filenames + npm aliases), then the verification
    binaries it cannot see. Descriptors are de-duplicated in a stable order so offence
    text is diff-comparable.
    """
    hits = list(_adv.classify_command(line, npm_scripts))
    for label, pattern in VERIFY_RES:
        match = pattern.search(line)
        if match:
            hits.append(f"{label} (`{' '.join(match.group('cmd').split())}`)")
    seen: set[str] = set()
    return [h for h in hits if not (h in seen or seen.add(h))]


def analyse_run_body(body: str, step: dict,
                     npm_scripts: dict[str, list[str]]) -> list[tuple[str, str, str]]:
    """Find every discarded gate-classified command in one `run:` body.

    Returns (command_descriptor, rule, line_text) triples, where rule is D1/D2/D3.
    """
    lines = logical_lines(body)
    effective = [(raw, text) for raw, text in lines if text.strip()]
    errexit = initial_errexit(step)
    findings: list[tuple[str, str, str]] = []
    flagged: set[tuple[str, str]] = set()

    # D3 is a property of the WHOLE body: an unindented trailing `exit 0` forces the
    # step green regardless of what ran above it.
    force_green = bool(effective and BARE_EXIT0_RE.match(effective[-1][0].rstrip()))

    for index, (_raw, text) in enumerate(effective):
        errexit_here = errexit
        errexit = apply_set(text, errexit)
        hits = gate_commands(text, npm_scripts)
        if not hits:
            continue
        discarded = DISCARD_RE.search(text)
        captured = index + 1 < len(effective) and CAPTURE_RE.match(effective[index + 1][1])
        for hit in hits:
            if discarded:
                rule = "D1"
            elif not errexit_here and not captured:
                rule = "D2"
            elif force_green:
                rule = "D3"
            else:
                continue
            key = (hit, text)
            if key in flagged:
                continue
            flagged.add(key)
            findings.append((hit, rule, text))
    return findings


RULE_TEXT = {
    "D1": "its failure is discarded by `|| true`",
    "D2": "it runs with errexit disabled (`set +e`) and its status is never captured",
    "D3": "the body ends in an unindented `exit 0`, forcing the step green",
}


def load_yaml(path: Path) -> dict:
    doc = yaml.safe_load(path.read_text(encoding="utf-8"))
    return doc if isinstance(doc, dict) else {}


def declared_advisory_names(root: Path) -> set[str]:
    path = root / ADVISORY_REGISTRY
    if not path.exists():
        return set()
    registry = json.loads(path.read_text(encoding="utf-8"))
    jobs = registry.get("jobs") if isinstance(registry, dict) else None
    return set(jobs) if isinstance(jobs, dict) else set()


def load_waivers(root: Path) -> tuple[list[dict], list[str]]:
    """Parse + schema-check the waiver registry. Returns (waivers, offences)."""
    path = root / WAIVER_REGISTRY
    if not path.exists():
        return [], [f"{WAIVER_REGISTRY}: missing — the run-swallow waiver registry is required"]
    try:
        registry = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return [], [f"{WAIVER_REGISTRY}: invalid JSON — {exc}"]
    waivers = registry.get("run_swallow_waivers") if isinstance(registry, dict) else None
    if not isinstance(waivers, list):
        return [], [f"{WAIVER_REGISTRY}: `run_swallow_waivers` must be a list"]

    offences: list[str] = []
    clean: list[dict] = []
    for index, entry in enumerate(waivers):
        where = f"{WAIVER_REGISTRY} run_swallow_waivers[{index}]"
        if not isinstance(entry, dict):
            offences.append(f"{where}: not an object")
            continue
        missing = [f for f in WAIVER_REQUIRED_FIELDS if not entry.get(f)]
        if missing:
            offences.append(f"{where}: missing required field(s) {', '.join(missing)}")
            continue
        clean.append(entry)
    return clean, offences


def steps_of(job: dict) -> list[dict]:
    steps = job.get("steps")
    if not isinstance(steps, list):
        return []
    return [s for s in steps if isinstance(s, dict)]


def collect_findings(root: Path) -> list[dict]:
    """Every discarded gate-classified command in a job that GATES."""
    advisory = declared_advisory_names(root)
    npm_scripts = _adv.load_npm_scripts(root)
    found: list[dict] = []
    for path in sorted((root / WORKFLOW_DIR).glob("*.yml")):
        doc = load_yaml(path)
        jobs = doc.get("jobs")
        if not isinstance(jobs, dict):
            continue
        for job_id, job in jobs.items():
            if not isinstance(job, dict):
                continue
            job_name = job.get("name", job_id)
            if isinstance(job_name, str) and job_name in advisory:
                continue  # declared non-gating; the advisory registry owns it
            for index, step in enumerate(steps_of(job)):
                run = step.get("run")
                if not isinstance(run, str):
                    continue
                step_name = step.get("name") or f"<step #{index}>"
                for command, rule, line in analyse_run_body(run, step, npm_scripts):
                    found.append({
                        "workflow": path.name, "job_id": job_id, "step_name": step_name,
                        "command": command, "rule": rule, "line": line,
                    })
    return found


def check_tree(root: Path) -> list[str]:
    waivers, offences = load_waivers(root)
    findings = collect_findings(root)

    used: set[int] = set()
    for finding in findings:
        match = None
        for position, waiver in enumerate(waivers):
            if (waiver["workflow"] == finding["workflow"]
                    and waiver["job_id"] == finding["job_id"]
                    and waiver["step_name"] == finding["step_name"]
                    and str(waiver["command"]) in finding["line"]):
                match = position
                break
        if match is not None:
            used.add(match)
            continue
        offences.append(
            f"R1 {finding['workflow']} job `{finding['job_id']}` step "
            f"{finding['step_name']!r}: {finding['command']} runs inside a job that "
            f"GATES but {RULE_TEXT[finding['rule']]} ({finding['rule']}) — the step "
            f"reports success whatever it found.\n"
            f"        line: {finding['line'][:160]}\n"
            f"        Fix the shell, or declare it in {WAIVER_REGISTRY} with a reason."
        )

    for position, waiver in enumerate(waivers):
        if position in used:
            continue
        offences.append(
            f"R2 {WAIVER_REGISTRY} run_swallow_waivers[{position}]: stale — no live "
            f"swallow matches {waiver['workflow']} job `{waiver['job_id']}` step "
            f"{waiver['step_name']!r} command {waiver['command']!r}. Delete the entry; "
            "leaving it would silently pre-approve the next swallow added there."
        )
    return offences


# ---------------------------------------------------------------------------
# Self-test — hermetic negative fixtures
# ---------------------------------------------------------------------------

_ADVISORY_FIXTURE = json.dumps({"jobs": {"lint (advisory)": {
    "owner_bead": "sq-fixture", "promotion_criteria": "n/a", "registered": "2026-08-03",
    "workflow": "fixture.yml", "job_id": "advisory_job"}}})


def _workflow(steps: str, job_name: str = "gating job", job_id: str = "build") -> str:
    return f"jobs:\n  {job_id}:\n    name: {job_name}\n    runs-on: ubuntu-latest\n    steps:\n{steps}"


_STEP_CLEAN = """      - name: Gate
        run: |
          set -euo pipefail
          grep -q ok file || true
          python3 scripts/coverage-gate.py
"""
_STEP_D1 = """      - name: Gate
        run: |
          set -euo pipefail
          python3 scripts/coverage-gate.py || true
"""
_STEP_D1_CARGO = """      - name: Gate
        run: |
          set -euo pipefail
          cargo nextest run --workspace || true
"""
_STEP_D1_CONTINUATION = """      - name: Gate
        run: |
          set -euo pipefail
          cargo clippy --workspace \\
            --all-targets || true
"""
_STEP_D2 = """      - name: Gate
        run: |
          set -euo pipefail
          set +e
          python3 scripts/coverage-gate.py
          echo done
"""
_STEP_D2_CAPTURED = """      - name: Gate
        run: |
          set -euo pipefail
          set +e
          python3 scripts/coverage-gate.py
          rc=$?
          set -e
          exit "$rc"
"""
_STEP_D2_REARMED = """      - name: Gate
        run: |
          set +e
          echo warming up
          set -e
          python3 scripts/coverage-gate.py
"""
_STEP_D3 = """      - name: Gate
        run: |
          set -euo pipefail
          cargo deny check advisories
          exit 0
"""
_STEP_D3_INDENTED_IS_CLEAN = """      - name: Gate
        run: |
          set -euo pipefail
          if [ -z "$SHA" ]; then
            echo "nothing to compare"
            exit 0
          fi
          cargo deny check advisories
"""
_STEP_CAPTURE_NOT_DISCARD = """      - name: Gate
        run: |
          set -euo pipefail
          cargo nextest run --workspace || rc=$?
          exit "${rc:-0}"
"""
_STEP_ECHO_MENTION = """      - name: Gate
        run: |
          set -euo pipefail
          echo "cargo test failed" || true
"""
_STEP_SHELL_NO_ERREXIT = """      - name: Gate
        shell: bash {0}
        run: |
          python3 scripts/coverage-gate.py
"""
_STEP_NON_GATE_SWALLOW = """      - name: Housekeeping
        run: |
          set -euo pipefail
          git rebase --abort 2>/dev/null || true
          grep -E '^dev/bench' refs || true
          ls -l out/ || true
"""


def _fixture_root(tmp: Path, workflow_text: str, waivers: object) -> Path:
    (tmp / WORKFLOW_DIR).mkdir(parents=True, exist_ok=True)
    (tmp / WORKFLOW_DIR / "fixture.yml").write_text(workflow_text, encoding="utf-8")
    (tmp / ADVISORY_REGISTRY).write_text(_ADVISORY_FIXTURE, encoding="utf-8")
    (tmp / WAIVER_REGISTRY).write_text(
        json.dumps({"run_swallow_waivers": waivers}), encoding="utf-8")
    return tmp


def _run_fixture(workflow_text: str, waivers: object = ()) -> list[str]:
    with tempfile.TemporaryDirectory() as raw:
        return check_tree(_fixture_root(Path(raw), workflow_text, list(waivers)))


def self_test() -> int:
    """Negative fixtures: each MUST fire, each clean one MUST NOT."""
    failures = 0

    must_fire = [
        ("D1 script gate", _workflow(_STEP_D1), "R1"),
        ("D1 cargo nextest", _workflow(_STEP_D1_CARGO), "R1"),
        ("D1 across a line continuation", _workflow(_STEP_D1_CONTINUATION), "R1"),
        ("D2 errexit off, status dropped", _workflow(_STEP_D2), "R1"),
        ("D3 trailing bare exit 0", _workflow(_STEP_D3), "R1"),
        ("shell: bash {0} never arms errexit", _workflow(_STEP_SHELL_NO_ERREXIT), "R1"),
    ]
    for label, text, prefix in must_fire:
        offences = _run_fixture(text)
        if not any(o.startswith(prefix) for o in offences):
            print(f"SELF-TEST FAIL: {label} should have produced a {prefix} finding")
            failures += 1

    must_be_clean = [
        ("clean gate + benign grep swallow", _workflow(_STEP_CLEAN)),
        ("set +e with the status captured", _workflow(_STEP_D2_CAPTURED)),
        ("errexit re-armed before the gate", _workflow(_STEP_D2_REARMED)),
        ("indented exit 0 in an early-exit guard", _workflow(_STEP_D3_INDENTED_IS_CLEAN)),
        ("`|| rc=$?` captures rather than discards", _workflow(_STEP_CAPTURE_NOT_DISCARD)),
        ("a gate NAMED in an echo is not an invocation", _workflow(_STEP_ECHO_MENTION)),
        ("swallowed non-gate housekeeping", _workflow(_STEP_NON_GATE_SWALLOW)),
        ("a declared-advisory job is exempt",
         _workflow(_STEP_D1, job_name="lint (advisory)", job_id="advisory_job")),
    ]
    for label, text in must_be_clean:
        offences = _run_fixture(text)
        if offences:
            print(f"SELF-TEST FAIL: {label} should be clean, got: {offences}")
            failures += 1

    # A waiver silences exactly its own site, and only while that site is live.
    waiver = [{"workflow": "fixture.yml", "job_id": "build", "step_name": "Gate",
               "command": "scripts/coverage-gate.py", "why": "fixture",
               "registered": "2026-08-03"}]
    if _run_fixture(_workflow(_STEP_D1), waiver):
        print("SELF-TEST FAIL: a matching waiver should silence its finding")
        failures += 1
    stale = _run_fixture(_workflow(_STEP_CLEAN), waiver)
    if not any(o.startswith("R2") for o in stale):
        print("SELF-TEST FAIL: a waiver matching nothing should red as stale (R2)")
        failures += 1
    # The waiver is keyed on the COMMAND, so it must not cover a different swallow
    # that appears later in the same step.
    narrow = _run_fixture(_workflow(_STEP_D1 + """        # noqa
      - name: Gate
        run: cargo nextest run --workspace || true
"""), waiver)
    if not any(o.startswith("R1") and "cargo" in o for o in narrow):
        print("SELF-TEST FAIL: a waiver must not cover a different command in the step")
        failures += 1
    for bad, label in (
        ({"run_swallow_waivers": [{"workflow": "fixture.yml"}]}, "incomplete waiver"),
        ({"run_swallow_waivers": {}}, "non-list waiver table"),
    ):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            (root / WORKFLOW_DIR).mkdir(parents=True)
            (root / WORKFLOW_DIR / "fixture.yml").write_text(_workflow(_STEP_CLEAN))
            (root / ADVISORY_REGISTRY).write_text(_ADVISORY_FIXTURE)
            (root / WAIVER_REGISTRY).write_text(json.dumps(bad))
            if not check_tree(root):
                print(f"SELF-TEST FAIL: {label} should be rejected by the schema check")
                failures += 1

    if failures:
        print(f"\nself-test: {failures} failure(s)")
        return 1
    print("self-test: all fixtures behaved as declared")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=REPO_ROOT)
    parser.add_argument("--self-test", action="store_true",
                        help="run the hermetic negative fixtures and exit")
    parser.add_argument("--inventory", action="store_true",
                        help="print every swallow site found, waived or not")
    args = parser.parse_args(argv)

    if args.self_test:
        return self_test()

    root = args.root.resolve()
    if not (root / WORKFLOW_DIR).is_dir():
        print(f"error: {root / WORKFLOW_DIR} is not a directory", file=sys.stderr)
        return 2

    if args.inventory:
        for finding in collect_findings(root):
            print(f"{finding['rule']}  {finding['workflow']}::{finding['job_id']}::"
                  f"{finding['step_name']}  [{finding['command']}]")
        return 0

    offences = check_tree(root)
    if offences:
        print("Run-body failure-swallow check FAILED:\n")
        for offence in offences:
            print(f"  - {offence}")
        print(f"\n{len(offences)} offence(s). See the header of {Path(__file__).name}.")
        return 1
    print("Run-body failure-swallow check: clean.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
