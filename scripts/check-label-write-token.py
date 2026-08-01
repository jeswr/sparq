#!/usr/bin/env python3
# [OPUS-5] 🤖 SPARQ agent — static guard: a label write must carry a BOT identity, never a
# human/alias PAT.
#
# WHY THIS EXISTS (sparq-org/sparq#5133, follow-up to #4911)
# ----------------------------------------------------------
# #4911's root cause is that machine orchestration wrote `needs:user`-class holds on PRs using
# the MAINTAINER'S ALIAS credential, so the resulting `labeled` timeline events name a human.
# Every downstream detector that resolves a hold's OWNER from that event then classifies a
# machine-written park as human-owned and takes its quietest exit — which is how a large parked
# population became unrecoverable without a human touching each PR.
#
# BOTH of #4911's proposed fixes live in `jeswr/agent-account-registry`, NOT here: the alias
# writes originate there, and `park_policy.py` / `capacity_park_admission` do not exist in this
# checkout. #5133 exists to record that, and to stop the sparq-side work being mistaken for a
# resolution. **THIS GUARD DOES NOT FIX #4911.** It re-admits no parked PR, changes no admission
# policy, and cannot reach the registry. #4911's stated verification obligation — a park written by
# the alias being CLASSIFIED as machine-owned, and the auto-readmit sweep then admitting a PR it
# currently refuses — is dischargeable only by a registry PR, and nothing here discharges it.
#
# What it DOES own is the one half that is sparq's. #5133's central sparq-side claim is an AUDIT:
# "every sparq workflow that writes a PR label uses either the orchestrator GitHub App token or the
# plain `github.token`, and no alias PAT appears in any label-writing step." That was true when a
# human read the workflows, and nothing whatsoever kept it true afterwards. sparq already holds a
# cross-repo credential aimed at exactly the registry in question (`secrets.REGISTRY_RING_TOKEN`,
# batch-merge.yml + fast-fix-ring.yml) plus `secrets.KB_DUMP_TOKEN`, documented in kb-dump.yml as
# "a GitHub PAT". Wiring either into a label-writing step is a two-line diff that no existing check
# would notice, and it would make sparq a SECOND producer of the exact defect #4911 is about. An
# audit nobody can re-run is a claim; this file makes it an invariant.
#
# THE RULES
#
# R1 — `label-write-foreign-credential`
#   A step that can write an issue/PR label may not have a non-bot `secrets.*` reference in
#   scope — its own body, plus the job-level and workflow-level `env:` it inherits. SCOPE, not
#   merely the token expression, is deliberate: `env: RING: ${{ secrets.X }}` followed by
#   `GH_TOKEN="$RING" gh pr edit --add-label` never names a token expression at all, and a rule
#   that only read `GH_TOKEN:` would wave it through. "Bot" is a DECLARED set
#   (BOT_IDENTITY_SECRETS) rather than a name heuristic, so widening the trusted-identity surface
#   is a reviewable diff and a newly-added secret is untrusted by default.
#
# R2 — `label-write-unminted-token`
#   A label-writing step whose token expression reads `steps.<id>.outputs.token` must have `<id>`
#   be an `actions/create-github-app-token` step IN THE SAME JOB. Without this, R1 is satisfiable
#   by laundering any credential through a step output and "App token" becomes an unchecked
#   assertion about a string.
#
# WHAT COUNTS AS A LABEL WRITE. Direct `gh pr/issue edit --add-label`, `gh label create/delete/…`,
# the github-script `addLabels`/`removeLabel`/`setLabels` calls, and a MUTATING `gh api` against a
# `/labels` path — plus ONE HOP of indirection, because the dominant shape in this repo is a step
# that shells out to a script (`pr-area-labels.py`, `rearm-sweeper.py`, `verdict-bridge.py`, …).
# The label-writing script set is COMPUTED from `scripts/` by the same patterns, not hard-coded, so
# a new writer script is covered the moment it is added. Over-inclusion is the safe direction: it
# only makes the guard stricter, and `--self-test` is what proves it is not merely permissive.
#
# Comment lines are stripped before ANY classification. This is load-bearing, not tidiness: five
# workflows here discuss `--add-label`/`--remove-label` in prose (merge-queue-feedback.yml,
# rearm-sweeper.yml, routing-self-tests.yml, triage-area.yml, triage-issue.yml), and a guard that
# read prose would attribute a label write to whichever job merely explains one.
#
# STDLIB ONLY — no PyYAML, matching scripts/check-advisory-registry.py. A workflow-lint guard that
# cannot run without a pip install is a guard nobody runs before pushing; the indentation of
# GitHub Actions YAML is fixed enough to parse structurally, and `--self-test` pins the parser.
#
# Run:  python3 scripts/check-label-write-token.py --self-test   # prove the detector detects
#       python3 scripts/check-label-write-token.py               # enforce over .github/workflows

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS_DIR = REPO_ROOT / ".github" / "workflows"
SCRIPTS_DIR = REPO_ROOT / "scripts"

# ── DECLARED trusted identities ───────────────────────────────────────────────────────────────
# A secret named here resolves to a BOT actor, so a label it writes carries a machine owner that
# #4911's owner-resolution can classify correctly. Anything absent is treated as possibly-a-human
# PAT. Adding a name here is the reviewable act of asserting "this credential is not a person".
BOT_IDENTITY_SECRETS: dict[str, str] = {
    # The workflow token. Writes are attributed to github-actions[bot].
    "GITHUB_TOKEN": "github-actions[bot] (workflow token)",
    # App credentials consumed by actions/create-github-app-token. Writes are attributed to the
    # sparq-orchestrator App's bot user, never to the maintainer.
    "ORCHESTRATOR_APP_ID": "sparq-orchestrator[bot] (App id, minted via create-github-app-token)",
    "ORCHESTRATOR_APP_PRIVATE_KEY": "sparq-orchestrator[bot] (App key, via create-github-app-token)",
}

APP_TOKEN_ACTION = "actions/create-github-app-token"

# ── Waivers ───────────────────────────────────────────────────────────────────────────────────
# (workflow file name, job id, step name) -> reason. EMPTY ON PURPOSE: at the time of writing no
# label-writing step in this repository needs a non-bot credential, which is precisely what #5133's
# audit found. A future entry is a diff a reviewer has to justify.
WAIVERS: dict[tuple[str, str, str], str] = {}

# ── Label-write detection ─────────────────────────────────────────────────────────────────────
LABEL_WRITE_PATTERNS: tuple[re.Pattern[str], ...] = (
    re.compile(r"--add-label\b"),
    re.compile(r"--remove-label\b"),
    re.compile(r"\bgh\s+label\s+(?:create|delete|edit|clone)\b"),
    re.compile(r"\.(?:addLabels|removeLabel|setLabels|removeAllLabels)\s*\("),
    # A MUTATING gh api / REST call against a labels collection, in either flag order.
    re.compile(r"(?:-X|--method)\s+(?:POST|PUT|PATCH|DELETE)\b[^\n]*?/labels\b"),
    re.compile(r"/labels\b[^\n]*?(?:-X|--method)\s+(?:POST|PUT|PATCH|DELETE)\b"),
)

# An action whose name is a labeler applies labels itself.
LABELER_ACTION_RE = re.compile(r"\blabeler\b")

SECRET_REF_RE = re.compile(r"\bsecrets\.([A-Za-z_][A-Za-z0-9_]*)")
STEP_OUTPUT_TOKEN_RE = re.compile(r"\bsteps\.([A-Za-z0-9_-]+)\.outputs\.token\b")


def text_writes_labels(text: str) -> bool:
    return any(p.search(text) for p in LABEL_WRITE_PATTERNS)


def strip_comment_lines(text: str) -> str:
    """Drop whole-line comments (YAML and, inside a `run:` block, shell).

    Both languages use `#`, so one pass covers both. A real label write can never live on a
    `#`-leading line in either, so nothing detectable is lost.
    """
    return "\n".join(line for line in text.splitlines() if not line.lstrip().startswith("#"))


def discover_label_writing_scripts(scripts_dir: Path) -> set[str]:
    """Repo-relative paths of scripts whose source performs a label write.

    Two exclusions, both for the same reason — a FIXTURE that quotes a label-write string is not a
    label write. `scripts/tests/` is excluded, or every `run: python3 scripts/tests/…` step would
    look privileged; and this guard's own file is excluded, since its `--self-test` corpus is made
    entirely of such strings and it would otherwise classify itself. Pinned in `self_test`.
    """
    found: set[str] = set()
    if not scripts_dir.is_dir():
        return found
    self_path = Path(__file__).resolve()
    for path in sorted(scripts_dir.rglob("*")):
        if path.suffix not in (".py", ".sh", ".js", ".mjs"):
            continue
        if "tests" in path.relative_to(scripts_dir).parts:
            continue
        if path.resolve() == self_path:
            continue
        try:
            source = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        if text_writes_labels(strip_comment_lines(source)):
            found.add(path.relative_to(REPO_ROOT).as_posix())
    return found


# ── Structural workflow parsing (stdlib) ──────────────────────────────────────────────────────
def _indent_of(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def block_at(text: str, key: str, indent: int) -> str:
    """Return the body of mapping `key:` declared at exactly `indent` spaces.

    The body is every following line indented deeper than `indent` (blank lines included), which
    is how a nested mapping/sequence terminates in YAML.
    """
    lines = text.splitlines()
    header = re.compile(rf"^ {{{indent}}}{re.escape(key)}:\s*$")
    for i, line in enumerate(lines):
        if not header.match(line):
            continue
        body: list[str] = []
        for nxt in lines[i + 1:]:
            if not nxt.strip():
                body.append(nxt)
                continue
            if _indent_of(nxt) <= indent:
                break
            body.append(nxt)
        return "\n".join(body)
    return ""


@dataclass
class Step:
    name: str
    id: str
    uses: str
    text: str


@dataclass
class Job:
    id: str
    env_text: str
    steps: list[Step]


def _scalar(block: str, key: str) -> str:
    # The list marker is part of the first key's line (`- name: Park the PR`), so normalise it
    # away before matching — otherwise every step reads as `<unnamed>` and a waiver could never
    # be keyed to one.
    block = re.sub(r"^(\s*)-\s", r"\1  ", block, count=1)
    m = re.search(rf"^\s*{re.escape(key)}:\s*(.+?)\s*$", block, re.MULTILINE)
    if not m:
        return ""
    value = m.group(1)
    if len(value) >= 2 and value[0] == value[-1] and value[0] in "\"'":
        return value[1:-1]
    # Unquoted: ` #` opens a YAML comment, so GitHub shows the TRUNCATED name in the run log
    # (several step names here end in a bare `#1234)` and are already truncated that way). Mirror
    # that, or a WAIVERS key written from the displayed name would never match.
    return value.split(" #", 1)[0].rstrip()


def parse_steps(steps_block: str) -> list[Step]:
    lines = steps_block.splitlines()
    item_re = re.compile(r"^(\s*)-\s")
    starts = [i for i, line in enumerate(lines) if item_re.match(line)]
    if not starts:
        return []
    base = min(_indent_of(lines[i]) for i in starts)
    starts = [i for i in starts if _indent_of(lines[i]) == base]
    steps: list[Step] = []
    for n, start in enumerate(starts):
        end = starts[n + 1] if n + 1 < len(starts) else len(lines)
        text = "\n".join(lines[start:end])
        steps.append(
            Step(
                name=_scalar(text, "name") or _scalar(text, "uses") or _scalar(text, "id") or "<unnamed>",
                id=_scalar(text, "id"),
                uses=_scalar(text, "uses"),
                text=text,
            )
        )
    return steps


def parse_workflow(text: str) -> tuple[str, list[Job]]:
    """Return (workflow-level env text, jobs). Comments are already stripped by the caller."""
    wf_env = block_at(text, "env", 0)
    m = re.search(r"^jobs:\s*$", text, re.MULTILINE)
    if not m:
        return wf_env, []
    jobs_text = text[m.end():]
    key_re = re.compile(r"^  ([A-Za-z_][A-Za-z0-9_-]*):\s*$", re.MULTILINE)
    matches = list(key_re.finditer(jobs_text))
    jobs: list[Job] = []
    for i, match in enumerate(matches):
        start = match.start()
        end = matches[i + 1].start() if i + 1 < len(matches) else len(jobs_text)
        block = jobs_text[start:end]
        jobs.append(
            Job(
                id=match.group(1),
                env_text=block_at(block, "env", 4),
                steps=parse_steps(block_at(block, "steps", 4)),
            )
        )
    return wf_env, jobs


# ── Findings ──────────────────────────────────────────────────────────────────────────────────
@dataclass
class Finding:
    rule: str
    workflow: str
    job: str
    step: str
    detail: str

    def render(self) -> str:
        return (
            f"{self.workflow} :: job `{self.job}` :: step `{self.step}`\n"
            f"    [{self.rule}] {self.detail}"
        )


@dataclass
class Scanner:
    label_writing_scripts: set[str] = field(default_factory=set)

    def step_writes_labels(self, step: Step) -> bool:
        if LABELER_ACTION_RE.search(step.uses):
            return True
        if text_writes_labels(step.text):
            return True
        # Script indirection applies to a `run:` step only. A `uses:` step executes an action, not
        # a repo script — and several checkout steps name writer scripts in `sparse-checkout:`,
        # which is a checkout manifest, not a call.
        if step.uses:
            return False
        return any(script in step.text for script in self.label_writing_scripts)

    def scan_workflow(self, path: Path) -> list[Finding]:
        text = strip_comment_lines(path.read_text(encoding="utf-8", errors="replace"))
        wf_env, jobs = parse_workflow(text)
        findings: list[Finding] = []
        for job in jobs:
            minted = {s.id for s in job.steps if s.id and APP_TOKEN_ACTION in s.uses}
            for step in job.steps:
                if not self.step_writes_labels(step):
                    continue
                if (path.name, job.id, step.name) in WAIVERS:
                    continue
                scope = "\n".join([wf_env, job.env_text, step.text])
                for secret in sorted(set(SECRET_REF_RE.findall(scope))):
                    if secret in BOT_IDENTITY_SECRETS:
                        continue
                    findings.append(
                        Finding(
                            "label-write-foreign-credential",
                            path.name,
                            job.id,
                            step.name,
                            f"`secrets.{secret}` is in scope of a label-writing step and is not a "
                            f"declared bot identity. A label written with a human/alias credential "
                            f"names a PERSON in the `labeled` timeline event, and owner-resolution "
                            f"downstream then reads a machine-written hold as human-owned — the "
                            f"sparq-org/sparq#4911 root cause. Use `github.token` or an "
                            f"`{APP_TOKEN_ACTION}` token; if this credential really is a bot, "
                            f"declare it in BOT_IDENTITY_SECRETS with the actor it resolves to.",
                        )
                    )
                for step_id in sorted(set(STEP_OUTPUT_TOKEN_RE.findall(scope))):
                    if step_id in minted:
                        continue
                    findings.append(
                        Finding(
                            "label-write-unminted-token",
                            path.name,
                            job.id,
                            step.name,
                            f"the token expression reads `steps.{step_id}.outputs.token`, but no "
                            f"step with id `{step_id}` in this job uses `{APP_TOKEN_ACTION}`. The "
                            f"identity behind that token is unverified, so R1 would be satisfied "
                            f"by laundering any credential through a step output.",
                        )
                    )
        return findings


def enforce(workflows_dir: Path, scripts_dir: Path) -> list[Finding]:
    scanner = Scanner(discover_label_writing_scripts(scripts_dir))
    findings: list[Finding] = []
    for path in sorted(workflows_dir.glob("*.yml")) + sorted(workflows_dir.glob("*.yaml")):
        findings.extend(scanner.scan_workflow(path))
    return findings


# ── Self-test ─────────────────────────────────────────────────────────────────────────────────
# A detector that has only ever been run over a clean tree has not been shown to detect anything.
# Each fixture is written in the shape the corresponding regression would actually take.
_FIXTURE_ALIAS_PAT = """
name: f
on: [push]
jobs:
  park:
    runs-on: ubuntu-latest
    steps:
      - name: Park the PR
        env:
          GH_TOKEN: ${{ secrets.REGISTRY_RING_TOKEN }}
        run: gh pr edit "$N" --add-label needs:user
"""

_FIXTURE_LAUNDERED_VIA_ENV = """
name: f
on: [push]
jobs:
  park:
    runs-on: ubuntu-latest
    env:
      RING: ${{ secrets.REGISTRY_RING_TOKEN }}
    steps:
      - name: Park the PR
        run: GH_TOKEN="$RING" gh pr edit "$N" --add-label needs:user
"""

_FIXTURE_INDIRECT_SCRIPT = """
name: f
on: [push]
jobs:
  park:
    runs-on: ubuntu-latest
    steps:
      - name: Apply area labels
        env:
          GH_TOKEN: ${{ secrets.KB_DUMP_TOKEN }}
        run: python3 scripts/pr-area-labels.py --repo "$R" --pr "$N"
"""

_FIXTURE_UNMINTED_TOKEN = """
name: f
on: [push]
jobs:
  park:
    runs-on: ubuntu-latest
    steps:
      - name: Fabricate a token
        id: app-token
        run: echo "token=whatever" >> "$GITHUB_OUTPUT"
      - name: Park the PR
        env:
          GH_TOKEN: ${{ steps.app-token.outputs.token }}
        run: gh pr edit "$N" --add-label needs:user
"""

_FIXTURE_WORKFLOW_TOKEN_OK = """
name: f
on: [push]
jobs:
  park:
    runs-on: ubuntu-latest
    steps:
      - name: Park the PR
        env:
          GH_TOKEN: ${{ github.token }}
        run: gh pr edit "$N" --add-label needs:user
"""

_FIXTURE_APP_TOKEN_OK = """
name: f
on: [push]
jobs:
  park:
    runs-on: ubuntu-latest
    env:
      ORCHESTRATOR_APP_ID: ${{ secrets.ORCHESTRATOR_APP_ID }}
    steps:
      - name: Mint
        id: app-token
        uses: actions/create-github-app-token@bcd2ba49218906704ab6c1aa796996da409d3eb1 # v3.2.0
        with:
          app-id: ${{ secrets.ORCHESTRATOR_APP_ID }}
          private-key: ${{ secrets.ORCHESTRATOR_APP_PRIVATE_KEY }}
      - name: Park the PR
        env:
          GH_TOKEN: ${{ steps.app-token.outputs.token || github.token }}
        run: gh pr edit "$N" --add-label needs:user
"""

_FIXTURE_NON_LABEL_STEP_OK = """
name: f
on: [push]
jobs:
  ring:
    runs-on: ubuntu-latest
    steps:
      - name: Ring the registry dispatcher
        env:
          REGISTRY_RING_TOKEN: ${{ secrets.REGISTRY_RING_TOKEN }}
        run: GH_TOKEN="$REGISTRY_RING_TOKEN" gh workflow run dispatch.yml -R jeswr/agent-account-registry
"""

# The prose case, which is why strip_comment_lines() exists: a job that only DISCUSSES a label
# write, while holding a PAT for an unrelated cross-repo call, must not be flagged.
_FIXTURE_PROSE_ONLY_OK = """
name: f
on: [push]
jobs:
  ring:
    runs-on: ubuntu-latest
    steps:
      # A pull request consumes `gh pr edit --add-label` against the ISSUES labels endpoint.
      - name: Ring the registry dispatcher
        env:
          REGISTRY_RING_TOKEN: ${{ secrets.REGISTRY_RING_TOKEN }}
        run: |
          # gh pr edit --add-label would go here if this job labelled anything
          GH_TOKEN="$REGISTRY_RING_TOKEN" gh workflow run dispatch.yml -R jeswr/agent-account-registry
"""

# Only the labelling step is at fault: the sibling step's PAT is its own business.
_FIXTURE_SIBLING_STEP_SCOPING = """
name: f
on: [push]
jobs:
  mixed:
    runs-on: ubuntu-latest
    steps:
      - name: Ring
        env:
          RING: ${{ secrets.REGISTRY_RING_TOKEN }}
        run: GH_TOKEN="$RING" gh workflow run dispatch.yml -R jeswr/agent-account-registry
      - name: Park
        env:
          GH_TOKEN: ${{ github.token }}
        run: gh pr edit "$N" --add-label needs:user
"""

# A checkout manifest NAMES writer scripts without calling any of them; classifying it as a label
# write would flag the checkout's own `token:` and make the guard un-adoptable.
_FIXTURE_CHECKOUT_MANIFEST_OK = """
name: f
on: [push]
jobs:
  sweep:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0 # v7.0.0
        with:
          token: ${{ secrets.KB_DUMP_TOKEN }}
          sparse-checkout: |
            scripts/pr-area-labels.py
            scripts/rearm-sweeper.py
"""

_POSITIVES: tuple[tuple[str, str, str], ...] = (
    ("alias PAT on a direct label write", _FIXTURE_ALIAS_PAT, "label-write-foreign-credential"),
    ("alias PAT laundered through job env", _FIXTURE_LAUNDERED_VIA_ENV, "label-write-foreign-credential"),
    ("PAT on a step shelling out to a writer script", _FIXTURE_INDIRECT_SCRIPT, "label-write-foreign-credential"),
    ("token output from a non-App step", _FIXTURE_UNMINTED_TOKEN, "label-write-unminted-token"),
)

_NEGATIVES: tuple[tuple[str, str], ...] = (
    ("github.token label write", _FIXTURE_WORKFLOW_TOKEN_OK),
    ("minted App token label write", _FIXTURE_APP_TOKEN_OK),
    ("PAT on a step that writes no label", _FIXTURE_NON_LABEL_STEP_OK),
    ("PAT on a step that only DISCUSSES a label write", _FIXTURE_PROSE_ONLY_OK),
    ("PAT confined to a sibling step", _FIXTURE_SIBLING_STEP_SCOPING),
    ("writer scripts named in a checkout manifest", _FIXTURE_CHECKOUT_MANIFEST_OK),
)


def self_test(tmp_root: Path) -> int:
    scanner = Scanner(discover_label_writing_scripts(SCRIPTS_DIR))
    # The indirection fixture names a REAL writer script, so discovery is exercised live rather
    # than stubbed. If discovery ever stops finding it, that fixture would pass VACUOUSLY.
    if "scripts/pr-area-labels.py" not in scanner.label_writing_scripts:
        print(
            "SELF-TEST FAIL: scripts/pr-area-labels.py was not discovered as a label writer, so "
            "the indirection fixture would pass vacuously.",
            file=sys.stderr,
        )
        return 1
    # The fixture exclusions are load-bearing, not cosmetic: without them this guard classifies
    # ITSELF (its fixtures quote `--add-label`), and then every workflow step that merely RUNS the
    # guard counts as a label write.
    own = Path(__file__).resolve().relative_to(REPO_ROOT).as_posix()
    if own in scanner.label_writing_scripts:
        print(
            f"SELF-TEST FAIL: {own} classified ITSELF as a label writer — its fixture strings are "
            "not label writes, and self-classification makes every step that runs it privileged.",
            file=sys.stderr,
        )
        return 1

    failures = 0
    fixture = tmp_root / "fixture.yml"
    for label, source, rule in _POSITIVES:
        fixture.write_text(source, encoding="utf-8")
        rules = {f.rule for f in scanner.scan_workflow(fixture)}
        if rule not in rules:
            print(
                f"SELF-TEST FAIL: positive `{label}` was NOT caught (expected {rule}, got "
                f"{sorted(rules) or 'nothing'}).",
                file=sys.stderr,
            )
            failures += 1
    for label, source in _NEGATIVES:
        fixture.write_text(source, encoding="utf-8")
        found = scanner.scan_workflow(fixture)
        if found:
            print(
                f"SELF-TEST FAIL: negative `{label}` was flagged — {[f.rule for f in found]}.",
                file=sys.stderr,
            )
            failures += 1

    if failures:
        return 1
    print(
        f"check-label-write-token self-test PASS "
        f"({len(_POSITIVES)} positives caught, {len(_NEGATIVES)} negatives clean, "
        f"{len(scanner.label_writing_scripts)} label-writing scripts discovered)."
    )
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="Enforce that every label-writing workflow step carries a bot identity."
    )
    ap.add_argument("--self-test", action="store_true", help="run the detector against fixtures")
    ap.add_argument("--list", action="store_true", help="list the label-writing steps found")
    args = ap.parse_args(argv)

    if args.self_test:
        import tempfile

        with tempfile.TemporaryDirectory() as td:
            return self_test(Path(td))

    if args.list:
        scanner = Scanner(discover_label_writing_scripts(SCRIPTS_DIR))
        for path in sorted(WORKFLOWS_DIR.glob("*.yml")):
            text = strip_comment_lines(path.read_text(encoding="utf-8", errors="replace"))
            _, jobs = parse_workflow(text)
            for job in jobs:
                for step in job.steps:
                    if scanner.step_writes_labels(step):
                        print(f"{path.name} :: {job.id} :: {step.name}")
        return 0

    findings = enforce(WORKFLOWS_DIR, SCRIPTS_DIR)
    if findings:
        print(
            "Label writes must carry a BOT identity (sparq-org/sparq#5133, #4911):\n",
            file=sys.stderr,
        )
        for finding in findings:
            print(finding.render(), file=sys.stderr)
            print(file=sys.stderr)
        print(f"{len(findings)} violation(s).", file=sys.stderr)
        return 1
    print("check-label-write-token: OK — every label-writing step resolves to a bot identity.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
