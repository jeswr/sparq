#!/usr/bin/env python3
# [OPUS-5] sparq-org/sparq#5371 — the STEP-level YAML seam. 🤖 SPARQ agent.
#
# WHY THIS EXISTS
# ===============
# The repo protects two levels well and the level between them not at all:
#
#   * JOB level — `ci-summary / gate` (scripts/ci_summary_gate.py) requires every
#     check-run on the head commit that is not DECLARED in
#     .github/advisory-registry.json, and scripts/check-advisory-registry.py keeps that
#     registry honest. Delete a job, or rename one out from under its declaration, and
#     something reds.
#   * INSIDE a python module — the scripts/ suites are mutation-checked. #4567 ran that
#     measurement over the module it touched (scripts/triage-area.py): 18/18 python
#     mutants dead.
#
#   * BETWEEN them — an individual workflow STEP. Nothing looked at it. #4567's reviewer
#     measured the complement of that 18/18: EVERY surviving mutant lived in a workflow
#     `if:`, a step, or a call site.
#
# The concrete shape, and the one this file was written against:
# `.github/workflows/triage-area.yml` runs `python3 scripts/triage-area.py --self-test`
# plus `python3 scripts/tests/test_triage_area.py` as the pre-apply guard before the
# half-hourly `--apply` cron mutates LIVE issue labels. DELETE that step — or add
# `continue-on-error: true` to it — and the job stays green, the gate stays green, every
# python test still passes, and the classifier applies unverified `area:` labels to live
# issues with nothing red anywhere. A wrong `area:` routes a worker at the wrong crate.
#
# WHAT IT CHECKS
# ==============
# Three rules over .github/workflows/*.yml, driven by .github/workflow-step-guards.json:
#
#   S1  PINNED SAFETY STEPS. Every entry in `pinned_steps` must still bind to a live
#       step: the workflow parses, the job id exists, the job carries no truthy
#       `continue-on-error`, EXACTLY ONE step in it has the declared `name:`, that step
#       still EXECUTES every string in `requires_run`, and — when `must_precede_step` is
#       declared — the step it protects exists and comes AFTER it. Each of those is one
#       of the mutations above, and none of them is observable anywhere else.
#
#       "Executes", not "contains": `requires_run` is matched against the reduction of
#       the `run:` body to the text that actually runs (see `executable_run_text`), so
#       commenting the invocation out, `echo`ing it, or moving it into a heredoc no
#       longer satisfies the pin. A substring test over the raw scalar accepted all
#       three, which made the pin assert the presence of the TEXT rather than of the
#       guard.
#
#       An `if:` is PINNED, not banned. A lane can legitimately be conditional (which
#       events verdict-bridge answers for, say), so an entry may declare
#       `expected_job_if` / `expected_step_if` and the checker asserts the live condition
#       matches it verbatim (whitespace-normalised). Declare nothing and the condition
#       must be absent. Either way `if: false`, an added disjunct, or a narrowed one is a
#       finding rather than a silent skip — which is the measured shape of every uncaught
#       workflow mutant in this repo.
#
#   S2  NO UNDECLARED STEP-LEVEL SWALLOW. A step-level `continue-on-error` that is not
#       literally `false` makes that step's failure invisible in the job's conclusion.
#       Inside a job that GATES it is therefore a silent gate removal. A job is exempt
#       only if its `name:` is DECLARED in .github/advisory-registry.json — the same
#       single source of truth ci_summary_gate.py uses to decide what does not gate —
#       because a declared advisory job already carries an owner_bead and a
#       promotion_criteria for exactly this. Anything else needs a `swallow_waivers`
#       entry here, with a reason. Note the deliberate strictness: a job in a
#       schedule-only workflow produces no check-run today, but it needs a waiver anyway,
#       because adding a `pull_request:` trigger later must not silently turn an
#       undocumented swallow into a live hole.
#
#   S3  THE REGISTRY CANNOT GO STALE. A registry is only as good as its coverage, and a
#       hand-maintained one drifts. So the checker DERIVES the shape S1 protects: a step
#       that INVOKES a self-test or a scripts/tests/ suite, followed in the same job by a
#       step that INVOKES a mutating command (`--apply`, `gh issue edit|close`,
#       `gh pr merge`, `gh label create|delete`), is a self-test-before-write LANE, and
#       every lane MUST have a `pinned_steps` entry naming that guard step and, as its
#       `must_precede_step`, the first write it stands in front of. A future workflow
#       with this shape is forced to declare its guard instead of quietly relying on
#       nobody deleting it.
#
#       Coverage is keyed on the LANE, not on the job. Keying it on `(workflow, job_id)`
#       — the first shape this file shipped — meant one registered lane vouched for the
#       whole job, so a SECOND guard/write sequence added beside it needed no pin of its
#       own and could then be deleted with nothing red: precisely the staleness S3 exists
#       to prevent, reintroduced inside the rule meant to prevent it.
#
# WHAT IT DOES NOT CHECK — read this before trusting it further than it goes
# =========================================================================
#   * S2 covers the declarative `continue-on-error:` key ONLY. A `run:` body that
#     swallows its own failure with `|| true` or `set +e` is NOT detected. That idiom has
#     legitimate uses throughout the tree, so banning it is a separate and much larger
#     piece of work, not a line in this script.
#   * `executable_run_text` is a CONSERVATIVE shell reduction, not a shell parser. It
#     drops comments, `echo`/`printf`/`:`/`true`/`false` arguments and heredoc bodies —
#     the ways a required command survives as text without running — but it cannot see
#     through indirection: a needle reached via `eval`, a variable, a function body, or a
#     sourced script still reads as executed, and one guarded by `if false; then` reads
#     as executed too. It closes the trivially-reachable bypasses; it does not decide
#     reachability.
#   * S3's markers are deliberately narrow (an invocation at the start of a line, not a
#     mention anywhere in the body) because a false positive here is an obligation
#     imposed on an unrelated author. A safety step that is neither a self-test nor
#     before a mutating CLI call will not be discovered; it has to be declared by hand.
#   * Like .github/advisory-registry.json, this registry is data in the same tree as the
#     code it guards: an author can delete an entry and the checker will pass. What it
#     buys is that the deletion is then an explicit, reviewable line in the diff instead
#     of an invisible property of a YAML file nobody diffed carefully. It is a regression
#     detector, not a defence against a deliberate committer.
#
# EXIT: 0 all clear, 1 findings, 2 usage/parse error.

from __future__ import annotations

import argparse
import json
import re
import sys
import tempfile
from pathlib import Path

import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent

WORKFLOW_DIR = Path(".github") / "workflows"
GUARD_REGISTRY = Path(".github") / "workflow-step-guards.json"
ADVISORY_REGISTRY = Path(".github") / "advisory-registry.json"

PINNED_REQUIRED_FIELDS = ("id", "workflow", "job_id", "step_name", "why", "registered")
WAIVER_REQUIRED_FIELDS = ("workflow", "job_id", "step_name", "reason", "registered")

# S3 markers. Anchored at the start of a (folded or literal) line so that a step which
# merely NAMES a path — `scripts/tests/foo.py` inside an issue body, say — is not read as
# invoking it. `run: >-` folds to a single line, which this still matches.
GUARD_INVOCATION_RE = re.compile(
    r"(?m)^[ \t]*(?:python3?|bash|sh)[ \t]+(?:\S*scripts/tests/\S+|\S+[ \t]+--self-test\b)"
)
WRITE_INVOCATION_RE = re.compile(
    r"(?m)^[ \t]*(?:"
    r"python3?[ \t]+\S+[^\n]*--apply\b"
    r"|gh[ \t]+issue[ \t]+(?:edit|close)\b"
    r"|gh[ \t]+pr[ \t]+merge\b"
    r"|gh[ \t]+label[ \t]+(?:create|delete)\b"
    r")"
)


# The `run:` reduction used by S1's `requires_run`. A command that survives only as
# COMMENT text, as an `echo` argument or inside a heredoc body does not run, so none of
# those may satisfy a pin.
HEREDOC_RE = re.compile(r"<<-?[ \t]*[\"']?([A-Za-z_][A-Za-z0-9_]*)[\"']?")
ASSIGNMENT_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
# Builtins that consume their arguments as DATA rather than executing them.
DATA_COMMANDS = frozenset({"echo", "printf", ":", "true", "false"})


def strip_comment(line: str) -> str:
    """Drop a `#` comment, honouring quotes so a `#` inside a string survives.

    A `#` only opens a comment at the start of a word, so `area:core#x` is left alone.
    """
    out: list[str] = []
    quote: str | None = None
    after_space = True
    index = 0
    while index < len(line):
        char = line[index]
        if char == "\\" and index + 1 < len(line) and quote != "'":
            out.append(char)
            out.append(line[index + 1])
            index += 2
            after_space = False
            continue
        if quote is not None:
            out.append(char)
            if char == quote:
                quote = None
            index += 1
            after_space = False
            continue
        if char in "'\"":
            quote = char
            out.append(char)
            index += 1
            after_space = False
            continue
        if char == "#" and after_space:
            break
        out.append(char)
        after_space = char.isspace()
        index += 1
    return "".join(out)


def split_commands(line: str) -> list[str]:
    """Split a line on unquoted `;`, `&&`, `||`, `|`, `&` into candidate commands."""
    commands: list[str] = []
    current: list[str] = []
    quote: str | None = None
    index = 0
    while index < len(line):
        char = line[index]
        if char == "\\" and index + 1 < len(line) and quote != "'":
            current.append(char)
            current.append(line[index + 1])
            index += 2
            continue
        if quote is not None:
            current.append(char)
            if char == quote:
                quote = None
            index += 1
            continue
        if char in "'\"":
            quote = char
            current.append(char)
            index += 1
            continue
        if char in ";&|":
            commands.append("".join(current))
            current = []
            index += 2 if line[index : index + 2] in ("&&", "||") else 1
            continue
        current.append(char)
        index += 1
    commands.append("".join(current))
    return commands


def executable_run_text(run: object) -> str:
    """Reduce a `run:` body to the text a shell would actually EXECUTE.

    Conservative on purpose — see the module header's limits. It removes the three ways
    a command survives in the body as inert text: a comment, an argument to a data
    builtin (`echo foo`), and a heredoc body.

    The one shape an author has to know about: the separators `;`, `&&`, `||`, `|` and
    `&` are consumed by the split, so a `requires_run` needle must not span one. That
    direction fails closed (an offence to fix, not a bypass), which is why the split is
    written this way rather than trying to preserve them.
    """
    kept: list[str] = []
    terminator: str | None = None
    for raw in str(run).splitlines():
        if terminator is not None:
            # Inside a heredoc: data, not commands. `<<-` allows a tabbed terminator.
            if raw.strip() == terminator:
                terminator = None
            continue
        line = strip_comment(raw)
        opened = HEREDOC_RE.search(line)
        if opened:
            terminator = opened.group(1)
        for command in split_commands(line):
            words = command.split()
            while words and ASSIGNMENT_RE.match(words[0]):
                words = words[1:]  # `FOO=1 echo bar` still echoes
            if not words:
                continue
            if words[0].rsplit("/", 1)[-1] in DATA_COMMANDS:
                continue
            kept.append(command)
    return "\n".join(kept)


def swallows(value: object) -> bool:
    """Is this `continue-on-error:` value capable of hiding a failure?

    Absent or a literal ``false`` is safe. Literal ``true`` swallows. A ``${{ … }}``
    expression is treated as swallowing: its runtime value is not decidable here, and
    the fail-closed direction is to demand a declaration.
    """
    if value is None or value is False:
        return False
    if value is True:
        return True
    return str(value).strip().lower() != "false"


def normalise_condition(value: object) -> str:
    """Collapse a YAML `if:` to a comparable single line.

    A condition is authored as `>-`, `|`, or inline, and re-wrapping it is a formatting
    change rather than a semantic one; normalising whitespace keeps the pin from firing
    on a reflow while still catching an added disjunct or an `if: false`.
    """
    return " ".join(str(value).split())


def check_condition(kind: str, entry_id: str, where: str, live: object, expected: object) -> list[str]:
    """Pin (not ban) an `if:`. Declared => must match verbatim; undeclared => must be absent."""
    if expected is None:
        if live is None:
            return []
        return [
            f"S1 {entry_id}: {where} gained an `if:` ({normalise_condition(live)!r}) — the "
            f"{kind} can now be skipped while every check stays green. If the condition is "
            f"intended, pin it as `expected_{kind}_if` in the same reviewed diff."
        ]
    if live is None:
        return [
            f"S1 {entry_id}: {where} lost its pinned `if:` — the declared condition "
            f"{normalise_condition(expected)!r} is gone"
        ]
    if normalise_condition(live) != normalise_condition(expected):
        return [
            f"S1 {entry_id}: {where} `if:` drifted from its pin\n"
            f"        pinned: {normalise_condition(expected)}\n"
            f"        live:   {normalise_condition(live)}"
        ]
    return []


def load_yaml(path: Path) -> dict:
    doc = yaml.safe_load(path.read_text(encoding="utf-8"))
    return doc if isinstance(doc, dict) else {}


def jobs_of(doc: dict) -> dict:
    jobs = doc.get("jobs")
    return jobs if isinstance(jobs, dict) else {}


def steps_of(job: dict) -> list[dict]:
    steps = job.get("steps")
    if not isinstance(steps, list):
        return []
    return [s for s in steps if isinstance(s, dict)]


def declared_advisory_names(root: Path) -> set[str]:
    path = root / ADVISORY_REGISTRY
    if not path.exists():
        return set()
    registry = json.loads(path.read_text(encoding="utf-8"))
    jobs = registry.get("jobs") if isinstance(registry, dict) else None
    return set(jobs) if isinstance(jobs, dict) else set()


def load_guard_registry(root: Path) -> tuple[dict, list[str]]:
    """Parse + schema-check the guard registry. Returns (registry, offences)."""
    path = root / GUARD_REGISTRY
    if not path.exists():
        return {}, [f"{GUARD_REGISTRY}: missing — the step-guard registry is required"]
    registry = json.loads(path.read_text(encoding="utf-8"))
    offences: list[str] = []

    pinned = registry.get("pinned_steps")
    if not isinstance(pinned, list) or not pinned:
        offences.append(
            f"{GUARD_REGISTRY}: `pinned_steps` must be a non-empty list — an empty "
            "registry silently disables S1"
        )
        pinned = pinned if isinstance(pinned, list) else []

    seen_ids: set[str] = set()
    for index, entry in enumerate(pinned):
        where = f"{GUARD_REGISTRY} pinned_steps[{index}]"
        if not isinstance(entry, dict):
            offences.append(f"{where}: not an object")
            continue
        missing = [f for f in PINNED_REQUIRED_FIELDS if not entry.get(f)]
        if missing:
            offences.append(f"{where}: missing required field(s) {', '.join(missing)}")
        entry_id = entry.get("id")
        if entry_id in seen_ids:
            offences.append(f"{where}: duplicate id {entry_id!r}")
        elif isinstance(entry_id, str):
            seen_ids.add(entry_id)

    waivers = registry.get("swallow_waivers")
    if waivers is None:
        waivers = []
    if not isinstance(waivers, list):
        offences.append(f"{GUARD_REGISTRY}: `swallow_waivers` must be a list")
        waivers = []
    for index, entry in enumerate(waivers):
        where = f"{GUARD_REGISTRY} swallow_waivers[{index}]"
        if not isinstance(entry, dict):
            offences.append(f"{where}: not an object")
            continue
        missing = [f for f in WAIVER_REQUIRED_FIELDS if not entry.get(f)]
        if missing:
            offences.append(f"{where}: missing required field(s) {', '.join(missing)}")

    registry = dict(registry)
    registry["pinned_steps"] = [e for e in pinned if isinstance(e, dict)]
    registry["swallow_waivers"] = [e for e in waivers if isinstance(e, dict)]
    return registry, offences


def check_pinned_steps(root: Path, pinned: list[dict]) -> list[str]:
    """S1 — every declared safety step still binds to a live, unconditional step."""
    offences: list[str] = []
    for entry in pinned:
        entry_id = entry.get("id", "<no id>")
        workflow = entry.get("workflow")
        job_id = entry.get("job_id")
        step_name = entry.get("step_name")
        if not (workflow and job_id and step_name):
            continue  # already reported by the schema check

        path = root / WORKFLOW_DIR / workflow
        if not path.exists():
            offences.append(f"S1 {entry_id}: {WORKFLOW_DIR / workflow} does not exist")
            continue
        doc = load_yaml(path)
        jobs = jobs_of(doc)
        if job_id not in jobs:
            offences.append(f"S1 {entry_id}: {workflow} has no job `{job_id}`")
            continue
        job = jobs[job_id]
        if not isinstance(job, dict):
            offences.append(f"S1 {entry_id}: {workflow} job `{job_id}` is not a mapping")
            continue

        offences += check_condition(
            "job",
            entry_id,
            f"{workflow} job `{job_id}`",
            job.get("if"),
            entry.get("expected_job_if"),
        )
        if swallows(job.get("continue-on-error")):
            offences.append(
                f"S1 {entry_id}: {workflow} job `{job_id}` gained `continue-on-error` — "
                "a failing guard no longer fails the job"
            )

        steps = steps_of(job)
        matches = [i for i, s in enumerate(steps) if s.get("name") == step_name]
        if not matches:
            offences.append(
                f"S1 {entry_id}: {workflow} job `{job_id}` has no step named "
                f"{step_name!r} — the pinned guard was deleted or renamed"
            )
            continue
        if len(matches) > 1:
            offences.append(
                f"S1 {entry_id}: {workflow} job `{job_id}` has {len(matches)} steps "
                f"named {step_name!r}; the pin must identify exactly one"
            )
            continue

        index = matches[0]
        step = steps[index]
        offences += check_condition(
            "step",
            entry_id,
            f"{workflow} step {step_name!r}",
            step.get("if"),
            entry.get("expected_step_if"),
        )
        if swallows(step.get("continue-on-error")):
            offences.append(
                f"S1 {entry_id}: step {step_name!r} gained `continue-on-error` — its "
                "failure is now invisible in the job's conclusion"
            )

        run = executable_run_text(step.get("run") or "")
        for needle in entry.get("requires_run") or []:
            if needle not in run:
                offences.append(
                    f"S1 {entry_id}: step {step_name!r} no longer runs {needle!r} — the "
                    "step survives but has stopped being the guard it was pinned as "
                    "(commenting the invocation out, echoing it, or moving it into a "
                    "heredoc leaves the text but not the guard)"
                )

        protected = entry.get("must_precede_step")
        if protected:
            after = [i for i, s in enumerate(steps) if s.get("name") == protected]
            if not after:
                offences.append(
                    f"S1 {entry_id}: {workflow} job `{job_id}` has no step named "
                    f"{protected!r}, which the guard is declared to precede"
                )
            elif index > min(after):
                offences.append(
                    f"S1 {entry_id}: step {step_name!r} now runs AFTER {protected!r} — a "
                    "guard that runs after the write it guards has already lost"
                )
    return offences


def check_swallows(root: Path, waivers: list[dict], advisory: set[str]) -> list[str]:
    """S2 — no step-level `continue-on-error` in a gating job without a waiver."""
    waived = {(w.get("workflow"), w.get("job_id"), w.get("step_name")) for w in waivers}
    offences: list[str] = []
    for path in sorted((root / WORKFLOW_DIR).glob("*.yml")):
        doc = load_yaml(path)
        for job_id, job in jobs_of(doc).items():
            if not isinstance(job, dict):
                continue
            job_name = job.get("name", job_id)
            if isinstance(job_name, str) and job_name in advisory:
                continue  # declared non-gating; the advisory registry owns it
            for step in steps_of(job):
                if not swallows(step.get("continue-on-error")):
                    continue
                step_name = step.get("name")
                if (path.name, job_id, step_name) in waived:
                    continue
                label = step_name if step_name else f"<unnamed `{step.get('uses') or 'run'}` step>"
                offences.append(
                    f"S2 {path.name} job `{job_id}` step {label!r}: step-level "
                    "`continue-on-error` inside a job that GATES — its failure is "
                    "invisible in the check-run"
                )
    return offences


def check_undeclared_guard_lanes(root: Path, pinned: list[dict]) -> list[str]:
    """S3 — every self-test-before-write LANE has a `pinned_steps` entry.

    A lane is a guard step plus the FIRST mutating step after it, and coverage is keyed
    on that pair. Keying on `(workflow, job_id)` would let one registered lane vouch for
    every other guard/write sequence in the same job, which is the staleness S3 exists to
    catch.
    """
    covered = {
        (e.get("workflow"), e.get("job_id"), e.get("step_name"), e.get("must_precede_step"))
        for e in pinned
    }
    pinned_guards = {(e.get("workflow"), e.get("job_id"), e.get("step_name")) for e in pinned}
    offences: list[str] = []
    for path in sorted((root / WORKFLOW_DIR).glob("*.yml")):
        doc = load_yaml(path)
        for job_id, job in jobs_of(doc).items():
            if not isinstance(job, dict):
                continue
            steps = steps_of(job)
            guards = [i for i, s in enumerate(steps) if GUARD_INVOCATION_RE.search(s.get("run") or "")]
            writes = [i for i, s in enumerate(steps) if WRITE_INVOCATION_RE.search(s.get("run") or "")]
            for guard in guards:
                after = [i for i in writes if i > guard]
                if not after:
                    continue
                write = min(after)
                guard_name = steps[guard].get("name")
                write_name = steps[write].get("name")
                if (path.name, job_id, guard_name, write_name) in covered:
                    continue
                lane = (
                    f"S3 {path.name} job `{job_id}`: step {guard_name!r} self-tests "
                    f"before step {write_name!r} mutates live state, but "
                )
                if guard_name is None:
                    offences.append(
                        lane + "the guard step has no `name:` — a pin binds by name, so "
                        "name it and register it in `pinned_steps`"
                    )
                elif (path.name, job_id, guard_name) in pinned_guards:
                    offences.append(
                        lane + "no `pinned_steps` entry pins it with "
                        f"`must_precede_step: {write_name!r}` — the guard is pinned in "
                        "front of a different step, so it could be reordered behind this "
                        "write with nothing red"
                    )
                else:
                    offences.append(
                        lane + "no `pinned_steps` entry pins it — deleting the self-test "
                        "would be silent"
                    )
    return offences


def check_tree(root: Path) -> list[str]:
    registry, offences = load_guard_registry(root)
    if not registry:
        return offences
    pinned = registry["pinned_steps"]
    offences += check_pinned_steps(root, pinned)
    offences += check_swallows(root, registry["swallow_waivers"], declared_advisory_names(root))
    offences += check_undeclared_guard_lanes(root, pinned)
    return offences


# ---------------------------------------------------------------------------
# Hermetic self-test. Every fixture below is one of the mutations the checker
# exists to catch, plus the controls that must stay green — a checker whose
# negative fixtures all pass is a checker that proves nothing.
# ---------------------------------------------------------------------------

CLEAN_WORKFLOW = """\
name: sweeper
on:
  schedule:
    - cron: '0 * * * *'
permissions:
  issues: write
jobs:
  classify:
    name: sweep the board
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@abc
      - name: Self-test before it writes
        run: |
          python3 scripts/sweeper.py --self-test
          python3 scripts/tests/test_sweeper.py
      - name: Apply the sweep
        run: |
          python3 scripts/sweeper.py --apply
"""

CLEAN_REGISTRY = {
    "pinned_steps": [
        {
            "id": "sweeper-selftest",
            "workflow": "sweeper.yml",
            "job_id": "classify",
            "step_name": "Self-test before it writes",
            "requires_run": ["sweeper.py --self-test", "scripts/tests/test_sweeper.py"],
            "must_precede_step": "Apply the sweep",
            "why": "fixture",
            "registered": "2026-08-02",
        }
    ],
    "swallow_waivers": [],
}


def _fixture(tmp: Path, workflow: str, registry: dict, advisory: dict | None = None) -> Path:
    root = tmp
    (root / WORKFLOW_DIR).mkdir(parents=True, exist_ok=True)
    (root / WORKFLOW_DIR / "sweeper.yml").write_text(workflow, encoding="utf-8")
    (root / GUARD_REGISTRY).write_text(json.dumps(registry), encoding="utf-8")
    (root / ADVISORY_REGISTRY).write_text(
        json.dumps({"jobs": advisory or {}}), encoding="utf-8"
    )
    return root


def self_test() -> int:
    # (label, workflow mutation, registry mutation, expected offence count sign)
    cases: list[tuple[str, str, dict, bool]] = []

    def case(label: str, workflow: str, registry: dict, should_fail: bool) -> None:
        cases.append((label, workflow, registry, should_fail))

    import copy

    # --- controls: these MUST stay clean, or every negative below proves nothing ---
    case("control: clean tree", CLEAN_WORKFLOW, CLEAN_REGISTRY, False)
    case(
        "control: an explicit `continue-on-error: false` is not a swallow",
        CLEAN_WORKFLOW.replace(
            "      - name: Apply the sweep\n",
            "      - name: Apply the sweep\n        continue-on-error: false\n",
        ),
        CLEAN_REGISTRY,
        False,
    )
    waived = copy.deepcopy(CLEAN_REGISTRY)
    waived["swallow_waivers"] = [
        {
            "workflow": "sweeper.yml",
            "job_id": "classify",
            "step_name": "Apply the sweep",
            "reason": "fixture",
            "registered": "2026-08-02",
        }
    ]
    case(
        "control: a waived swallow passes S2",
        CLEAN_WORKFLOW.replace(
            "      - name: Apply the sweep\n",
            "      - name: Apply the sweep\n        continue-on-error: true\n",
        ),
        waived,
        False,
    )
    case(
        "control: a declared-advisory job may swallow at the step",
        CLEAN_WORKFLOW.replace(
            "      - name: Apply the sweep\n",
            "      - name: Apply the sweep\n        continue-on-error: true\n",
        ),
        CLEAN_REGISTRY,
        False,
    )

    # --- S1 mutations ---
    case(
        "S1: the pinned guard step is DELETED",
        CLEAN_WORKFLOW.replace(
            "      - name: Self-test before it writes\n"
            "        run: |\n"
            "          python3 scripts/sweeper.py --self-test\n"
            "          python3 scripts/tests/test_sweeper.py\n",
            "",
        ),
        CLEAN_REGISTRY,
        True,
    )
    case(
        "S1: the pinned guard step gains continue-on-error",
        CLEAN_WORKFLOW.replace(
            "      - name: Self-test before it writes\n",
            "      - name: Self-test before it writes\n        continue-on-error: true\n",
        ),
        CLEAN_REGISTRY,
        True,
    )
    case(
        "S1: the pinned guard step gains an `if:`",
        CLEAN_WORKFLOW.replace(
            "      - name: Self-test before it writes\n",
            "      - name: Self-test before it writes\n        if: false\n",
        ),
        CLEAN_REGISTRY,
        True,
    )
    case(
        "S1: the guarded JOB gains an `if:`",
        CLEAN_WORKFLOW.replace(
            "    runs-on: ubuntu-latest\n", "    runs-on: ubuntu-latest\n    if: false\n"
        ),
        CLEAN_REGISTRY,
        True,
    )
    # An `if:` is PINNED, not banned: the three cases below are the whole contract.
    conditional = CLEAN_WORKFLOW.replace(
        "    runs-on: ubuntu-latest\n",
        "    runs-on: ubuntu-latest\n    if: >-\n      github.event_name == 'schedule'\n",
    )
    pinned_if = copy.deepcopy(CLEAN_REGISTRY)
    pinned_if["pinned_steps"][0]["expected_job_if"] = "github.event_name == 'schedule'"
    case("control: a job `if:` that matches its pin, re-wrapped", conditional, pinned_if, False)
    case(
        "S1: a pinned job `if:` gains a disjunct",
        conditional.replace(
            "      github.event_name == 'schedule'\n",
            "      github.event_name == 'schedule' || false\n",
        ),
        pinned_if,
        True,
    )
    case("S1: a pinned job `if:` disappears", CLEAN_WORKFLOW, pinned_if, True)
    pinned_step_if = copy.deepcopy(CLEAN_REGISTRY)
    pinned_step_if["pinned_steps"][0]["expected_step_if"] = "always()"
    case(
        "control: a step `if:` that matches its pin",
        CLEAN_WORKFLOW.replace(
            "      - name: Self-test before it writes\n",
            "      - name: Self-test before it writes\n        if: always()\n",
        ),
        pinned_step_if,
        False,
    )
    case(
        "S1: the guarded JOB gains continue-on-error",
        CLEAN_WORKFLOW.replace(
            "    runs-on: ubuntu-latest\n",
            "    runs-on: ubuntu-latest\n    continue-on-error: true\n",
        ),
        CLEAN_REGISTRY,
        True,
    )
    case(
        "S1: the guard survives but stops invoking one of its suites",
        CLEAN_WORKFLOW.replace("          python3 scripts/tests/test_sweeper.py\n", ""),
        CLEAN_REGISTRY,
        True,
    )
    # Deleting the line is the EASY mutation. These three keep the pinned text in the
    # body while removing the execution — the shape a substring test over the raw `run:`
    # scalar accepted, which made the pin assert the text rather than the guard.
    case(
        "S1: the guard's invocation is COMMENTED OUT (text survives, execution does not)",
        CLEAN_WORKFLOW.replace(
            "          python3 scripts/sweeper.py --self-test\n",
            "          # python3 scripts/sweeper.py --self-test\n",
        ),
        CLEAN_REGISTRY,
        True,
    )
    case(
        "S1: the guard's invocation survives only as ECHOED text",
        CLEAN_WORKFLOW.replace(
            "          python3 scripts/tests/test_sweeper.py\n",
            "          echo python3 scripts/tests/test_sweeper.py\n",
        ),
        CLEAN_REGISTRY,
        True,
    )
    case(
        "S1: the guard's invocation survives only inside a HEREDOC body",
        CLEAN_WORKFLOW.replace(
            "          python3 scripts/tests/test_sweeper.py\n",
            "          cat <<'EOF' > /tmp/note\n"
            "          python3 scripts/tests/test_sweeper.py\n"
            "          EOF\n",
        ),
        CLEAN_REGISTRY,
        True,
    )
    case(
        "control: a trailing `#` comment beside a live invocation is still a live invocation",
        CLEAN_WORKFLOW.replace(
            "          python3 scripts/sweeper.py --self-test\n",
            "          python3 scripts/sweeper.py --self-test  # rule-table regressions\n",
        ),
        CLEAN_REGISTRY,
        False,
    )
    case(
        "control: an invocation piped into another command still runs",
        CLEAN_WORKFLOW.replace(
            "          python3 scripts/tests/test_sweeper.py\n",
            "          python3 scripts/tests/test_sweeper.py | tee /tmp/log\n",
        ),
        CLEAN_REGISTRY,
        False,
    )
    case(
        "S1: the guard is moved AFTER the write it guards",
        CLEAN_WORKFLOW.replace(
            "      - name: Self-test before it writes\n"
            "        run: |\n"
            "          python3 scripts/sweeper.py --self-test\n"
            "          python3 scripts/tests/test_sweeper.py\n"
            "      - name: Apply the sweep\n"
            "        run: |\n"
            "          python3 scripts/sweeper.py --apply\n",
            "      - name: Apply the sweep\n"
            "        run: |\n"
            "          python3 scripts/sweeper.py --apply\n"
            "      - name: Self-test before it writes\n"
            "        run: |\n"
            "          python3 scripts/sweeper.py --self-test\n"
            "          python3 scripts/tests/test_sweeper.py\n",
        ),
        CLEAN_REGISTRY,
        True,
    )
    renamed_job = copy.deepcopy(CLEAN_REGISTRY)
    renamed_job["pinned_steps"][0]["job_id"] = "gone"
    case("S1: the pin points at a job that no longer exists", CLEAN_WORKFLOW, renamed_job, True)
    missing_protected = copy.deepcopy(CLEAN_REGISTRY)
    missing_protected["pinned_steps"][0]["must_precede_step"] = "Nope"
    case("S1: must_precede_step names a step that is gone", CLEAN_WORKFLOW, missing_protected, True)

    # --- S2 mutations ---
    case(
        "S2: an undeclared step-level swallow in a gating job",
        CLEAN_WORKFLOW.replace(
            "      - name: Apply the sweep\n",
            "      - name: Apply the sweep\n        continue-on-error: true\n",
        ),
        CLEAN_REGISTRY,
        True,
    )
    case(
        "S2: an expression-valued continue-on-error is not decidable => must be declared",
        CLEAN_WORKFLOW.replace(
            "      - name: Apply the sweep\n",
            "      - name: Apply the sweep\n        continue-on-error: ${{ github.event_name == 'schedule' }}\n",
        ),
        CLEAN_REGISTRY,
        True,
    )

    # --- S3 mutations ---
    empty_pins = {"pinned_steps": [], "swallow_waivers": []}
    case("S3/schema: an empty pinned_steps list", CLEAN_WORKFLOW, empty_pins, True)
    case(
        "S3: a SECOND self-test-before-write lane, in a second job, that nothing pins",
        CLEAN_WORKFLOW
        + """\
  second:
    name: another sweep
    runs-on: ubuntu-latest
    steps:
      - name: Self-test
        run: |
          python3 scripts/other.py --self-test
      - name: Write labels
        run: |
          gh issue edit 1 --add-label area:core
""",
        CLEAN_REGISTRY,
        True,
    )
    # The same hole INSIDE an already-pinned job. Job-granular coverage let the
    # registered lane vouch for this one, so the second guard needed no pin of its own
    # and could then be deleted with nothing red. The first lane is untouched, so only
    # lane-granular coverage reds here.
    two_lanes = CLEAN_WORKFLOW + """\
      - name: Self-test the second sweep
        run: |
          python3 scripts/other.py --self-test
      - name: Write labels
        run: |
          gh issue edit 1 --add-label area:core
      - name: Post a summary
        run: |
          python3 scripts/other.py --summarise
"""
    case(
        "S3: a SECOND lane in the ALREADY-PINNED job, with only the first registered",
        two_lanes,
        CLEAN_REGISTRY,
        True,
    )
    second_pin = {
        "id": "sweeper-selftest-2",
        "workflow": "sweeper.yml",
        "job_id": "classify",
        "step_name": "Self-test the second sweep",
        "requires_run": ["other.py --self-test"],
        "must_precede_step": "Write labels",
        "why": "fixture",
        "registered": "2026-08-02",
    }
    both_lanes = copy.deepcopy(CLEAN_REGISTRY)
    both_lanes["pinned_steps"].append(copy.deepcopy(second_pin))
    case("control: both lanes in one job, each with its own pin", two_lanes, both_lanes, False)
    wrong_protected = copy.deepcopy(CLEAN_REGISTRY)
    # Pinned in front of a LATER step, so S1 is satisfied while the write this guard
    # actually stands in front of is unprotected against a reorder.
    wrong_protected["pinned_steps"].append(
        {**copy.deepcopy(second_pin), "must_precede_step": "Post a summary"}
    )
    case(
        "S3: a lane pinned in front of a step LATER than the write it precedes",
        two_lanes,
        wrong_protected,
        True,
    )

    # --- schema mutations ---
    dup = copy.deepcopy(CLEAN_REGISTRY)
    dup["pinned_steps"].append(copy.deepcopy(dup["pinned_steps"][0]))
    case("schema: duplicate pin ids", CLEAN_WORKFLOW, dup, True)
    no_why = copy.deepcopy(CLEAN_REGISTRY)
    del no_why["pinned_steps"][0]["why"]
    case("schema: a pin with no `why`", CLEAN_WORKFLOW, no_why, True)
    bad_waiver = copy.deepcopy(CLEAN_REGISTRY)
    bad_waiver["swallow_waivers"] = [{"workflow": "sweeper.yml", "job_id": "classify"}]
    case("schema: a waiver with no reason", CLEAN_WORKFLOW, bad_waiver, True)

    failures = 0
    with tempfile.TemporaryDirectory() as tmpdir:
        for index, (label, workflow, registry, should_fail) in enumerate(cases):
            root = Path(tmpdir) / f"case{index}"
            advisory = (
                {"sweep the board": {"owner_bead": "x"}}
                if label.startswith("control: a declared-advisory")
                else {}
            )
            _fixture(root, workflow, registry, advisory)
            offences = check_tree(root)
            if bool(offences) != should_fail:
                failures += 1
                verb = "expected offences, got none" if should_fail else "expected clean"
                print(f"  FAIL [{label}]: {verb}")
                for offence in offences:
                    print(f"        {offence}")

    if failures:
        print(f"check-workflow-step-guards self-test: {failures}/{len(cases)} case(s) FAILED")
        return 1
    print(
        f"check-workflow-step-guards self-test: {len(cases)} cases OK "
        "(S1 step/job deletion + conditional + swallow + reorder + de-guard "
        "incl. comment/echo/heredoc, S2 undeclared/expression swallow, "
        "S3 undeclared lane incl. a second lane in an already-pinned job, schema)"
    )
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="#5371: workflow STEP-level safety-step registry check (S1/S2/S3)"
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
        help="run the hermetic mutation fixtures and exit",
    )
    args = parser.parse_args(argv)

    if args.self_test:
        return self_test()

    offences = check_tree(args.root)
    if not offences:
        print("check-workflow-step-guards: all clear (S1 + S2 + S3)")
        return 0

    print(f"check-workflow-step-guards: {len(offences)} offence(s):\n")
    for offence in offences:
        print(f"  {offence}")
    print()
    print(
        "  S1: the step is PINNED in .github/workflow-step-guards.json because deleting "
        "it, skipping it or swallowing its failure is invisible everywhere else. Restore "
        "it — or, if the guard genuinely moved, update the entry in the same reviewed diff."
    )
    print(
        "  S2: a step-level `continue-on-error` in a job that gates hides that step's "
        "failure from the check-run. Remove it, or add a `swallow_waivers` entry with a "
        "reason (a job declared in .github/advisory-registry.json is already exempt)."
    )
    print(
        "  S3: this lane self-tests before it mutates live state. Add a `pinned_steps` "
        "entry naming the guard step, with the write it stands in front of as its "
        "`must_precede_step`, so a later deletion or reorder reds instead of passing "
        "silently. A pin covers the LANE it names, not the whole job."
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
