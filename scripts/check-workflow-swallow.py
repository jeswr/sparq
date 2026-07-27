#!/usr/bin/env python3
"""Corpus-wide guard: a GATING workflow step must not throw away an earned failure.

Why this exists
---------------
``scripts/check-workflow-seam.py`` (#4385) proved the machinery — it joins backslash
continuations into logical lines first, then rejects ``||`` on the guarded command,
``set +e``, a relaxed ``shell:``, step/job ``if:``, ``continue-on-error`` and a missing
call site.  But it is pointed at ONE gate's steps by ``--needle``, and
``check-sticky-failure-exits.py`` audits Python, not YAML.  Everything else in the
repository's ``run:`` blocks was unguarded.  Discarding an earned failure has caused
three separate incidents here in a single day, and in a measured 18-mutant run every
mutant that survived lived at the YAML seam rather than in the Python.

This script is the corpus driver.  It **imports** the seam checker's primitives rather
than reimplementing (or editing) them — ``logical_lines`` is the Y11 fix and must not be
forked — and adds the three things that only appear at scale:

1. **A structural gating/advisory split.**  Not a list of blessed step names.  A step is
   *non-gating* when the repository's own merge gate already ignores it:

   * its workflow has no ``pull_request`` / ``merge_group`` / ``pull_request_target``
     trigger, so it never produces a check-run on a PR head for ``ci_summary_gate.py``
     to aggregate; or
   * its job ``name:`` matches ``\\b(advisory|informational)\\b`` — literally the
     ``ADVISORY_RE`` predicate ``scripts/ci_summary_gate.py`` excludes by and
     ``scripts/check-advisory-registry.py`` (C2) already forces into
     ``.github/advisory-registry.json`` with an owner and promotion criteria.

   So "advisory" is read off the same structure that makes it advisory in reality.  A
   step in an advisory job is censused, never a finding.

2. **A ``||`` taxonomy, because most ``||`` is not swallowing.**  On the live corpus a
   naive "``||`` in a gating job" rule reports 92 occurrences, of which the large
   majority are boolean tests (``[ a ] || [ b ]``), amplifiers (``|| { echo …; exit 1; }``
   — the *opposite* of swallowing), exit-code captures that are read back
   (``|| rc=$?`` … ``if [ "$rc" -ne 0 ]``), or shell comments.  A guard that reds on
   those is unshippable and gets deleted, which is worse than no guard.  Only the
   constructs that convert a non-zero exit into a zero one *with nobody looking* are
   findings.

3. **A ratcheting allowlist.**  Entries are keyed by a content fingerprint over
   ``(workflow, job, kind, normalised text)``, following #4385's allowlist design: an
   exemption cannot silently cover a different construct later added to the same step,
   and a **stale entry is a hard failure**, so the exemption cannot outlive the thing it
   exempts.

Findings are ADVISORY on the first pass (``--advisory``); the allowlist hygiene and the
corpus floors are HARD from day one.  See ``--help`` and the module docstring in
``scripts/check-workflow-swallow.allowlist.json`` for the promotion criteria.

Anti-vacuity: ``--min-workflows`` / ``--min-run-steps`` are corpus-level analogues of the
seam checker's ``--min-invocations``.  A glob typo, a moved directory, or a parse
regression that silently scans *nothing* must red rather than print a clean bill of
health.  #4385's own first CI run shipped a "wired but mis-argued" step whose floor could
be satisfied by its own argument; the floors here are asserted from the YAML by
``scripts/tests/test_workflow_swallow.sh`` (W7) and printed by the step itself.

Stdlib + PyYAML (already required by the other workflow-parsing lints).
``--self-test`` runs hermetic both-direction teeth over synthetic fixtures;
``scripts/tests/test_workflow_swallow.sh`` mutates a COPY of a real workflow.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import re
import sys
import tempfile
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_ALLOWLIST = Path(__file__).with_name("check-workflow-swallow.allowlist.json")


def _load_seam_module():
    """Import ``check-workflow-seam.py`` (a dashed filename) as a module.

    Importing rather than copying is deliberate: ``logical_lines`` is the fix for the
    Y11 mutant (``|| true`` written across a backslash continuation).  A forked copy
    would drift, and the drift would be invisible — exactly the failure class this file
    exists to catch.
    """
    path = Path(__file__).with_name("check-workflow-seam.py")
    spec = importlib.util.spec_from_file_location("check_workflow_seam", path)
    if spec is None or spec.loader is None:  # pragma: no cover - packaging accident
        raise SystemExit(f"check-workflow-swallow: cannot import {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_SEAM = _load_seam_module()
logical_lines = _SEAM.logical_lines
SAFE_SHELLS = _SEAM.SAFE_SHELLS
RELAX_ERRMODE_RE = _SEAM.RELAX_ERRMODE_RE

# The SAME predicate scripts/ci_summary_gate.py excludes by and
# scripts/check-advisory-registry.py (C2) registers. Kept in sync by W9.
ADVISORY_RE = re.compile(r"\b(advisory|informational)\b", re.IGNORECASE)

# Events that put a check-run on a PR head, i.e. that the required `gate` aggregates.
GATING_EVENTS = frozenset({"pull_request", "pull_request_target", "merge_group"})

# A run body opens a relaxed-errmode window with `set +e` / `set +o pipefail` (detected
# by the seam checker's own RELAX_ERRMODE_RE) and closes it by restoring `set -e` /
# `set -o pipefail`.
SET_RESTORE_RE = re.compile(r"^\s*set\s+-(?!-)(\S*)")
SUBST_ASSIGN_RE = re.compile(r"([A-Za-z_][A-Za-z0-9_]*)=\"?\$\(")

ASSIGN_RE = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)=")
TEST_HEAD_RE = re.compile(r"^(\[\[?|test|!)\b|^\[\[?\s")
CONTROL_HEAD_RE = re.compile(r"^(if|elif|while|until|case)\b")
# RHS spellings that discard the status outright.
NOOP_RHS_RE = re.compile(r"^(true|:)\s*(\)|;|$|>|#)")
MESSAGE_RHS_RE = re.compile(r"^(echo|printf|cat|tail|head|>&2|:\s*>)\b")

# Kinds. Findings are the first group; the rest are censused as structurally sound.
FINDING_KINDS = (
    "SWALLOW",
    "CAPTURE-UNREAD",
    "SUBSHELL-SWALLOW-UNCHECKED",
    "UNSCOPED-SET+",
    "UNCAPTURED-SET+",
    "CONTINUE-ON-ERROR",
    "UNSAFE-SHELL",
)
BENIGN_KINDS = (
    "BOOLEAN-TEST",
    "AMPLIFIER",
    "CAPTURE-READ",
    "FALLBACK-COMMAND",
    "SUBSHELL-CHECKED",
    "SCOPED-SET+",
)
REQUIRED_ALLOWLIST_FIELDS = ("workflow", "job", "kind", "snippet", "reason", "promotion", "registered")


# ---------------------------------------------------------------------------
# Shell surface analysis
# ---------------------------------------------------------------------------

def strip_comment(line: str) -> str:
    """Drop a shell ``#`` comment, respecting quotes.

    Load-bearing: the corpus contains prose comments that *mention* ``|| rc=$?`` while
    explaining why it is safe.  Counting those as code inflates the population and
    trains reviewers to ignore the guard.
    """
    out = []
    quote = None
    prev = ""
    for i, ch in enumerate(line):
        if quote:
            out.append(ch)
            if ch == quote and prev != "\\":
                quote = None
        elif ch in "'\"":
            quote = ch
            out.append(ch)
        elif ch == "#" and (i == 0 or line[i - 1].isspace()):
            break
        else:
            out.append(ch)
        prev = ch
    return "".join(out).rstrip()


def scan_or(line: str) -> list[tuple[int, int]]:
    """``[(offset, substitution_depth)]`` for every ``||`` that is shell CODE.

    Three states have to be distinguished or the classification is wrong:

    * inside quotes — ``echo "a || b"`` is prose, not an operator;
    * depth 0 — the operator decides whether the STATEMENT fails;
    * depth > 0 — nested in ``$( … )``/backticks, so it decides what the substitution
      yields, not whether the outer statement fails. That is the SUBSHELL axis.

    ``"$( … "x" … )"`` re-opens quoting inside the substitution, which is why the quote
    state is stacked rather than toggled.
    """
    found: list[tuple[int, int]] = []
    quote: str | None = None
    stack: list[str | None] = []
    tick = False
    i = 0
    prev = ""
    while i < len(line):
        ch = line[i]
        if quote == "'":
            if ch == "'":
                quote = None
        elif ch == "$" and line[i : i + 2] == "$(":
            stack.append(quote)
            quote = None
            i += 2
            prev = "("
            continue
        elif ch == ")" and stack:
            quote = stack.pop()
            i += 1
            prev = ")"
            continue
        elif quote == '"':
            if ch == '"' and prev != "\\":
                quote = None
        elif ch in "'\"":
            quote = ch
        elif ch == "`":
            if tick:
                tick = False
                quote = stack.pop()
            else:
                tick = True
                stack.append(quote)
        elif ch == "|" and line[i : i + 2] == "||":
            found.append((i, len(stack)))
            i += 2
            prev = "|"
            continue
        prev = ch
        i += 1
    return found


def _rhs_kind(rhs: str) -> tuple[str, str]:
    """Classify the right-hand side of the LAST ``||`` in a chain.

    ``a || b || c`` exits with ``c``'s status, so the chain's tail decides whether a
    failure survives.
    """
    rhs = rhs.strip()
    if rhs.startswith("{"):
        inner = rhs[1:].rsplit("}", 1)[0]
        # `exit 0` inside the block is a swallow wearing an amplifier's clothes — the
        # live corpus has two of these (`|| { echo "::warning::…"; exit 0; }`).
        if re.search(r"\b(exit|return)\s+(?![\"']?0[\"']?\s*(;|\}|$))\S+", inner) or re.search(
            r"\b(exit|return)\s*(;|$)", inner
        ):
            return "AMPLIFIER", ""
        return "SWALLOW", "block exits 0 / never exits"
    m = re.match(r"^(exit|return)\b\s*(\S*)", rhs)
    if m:
        arg = m.group(2).strip("\"';")
        if arg != "0":
            return "AMPLIFIER", ""
        return "SWALLOW", "`exit 0` converts the failure into a success"
    if NOOP_RHS_RE.match(rhs) or rhs in ("true", ":"):
        return "SWALLOW", "`|| true` / `|| :` discards the exit status"
    if MESSAGE_RHS_RE.match(rhs):
        return "SWALLOW", "the fallback only prints; the failure is discarded"
    if TEST_HEAD_RE.match(rhs):
        return "BOOLEAN-TEST", ""
    m = ASSIGN_RE.match(rhs)
    if m:
        return "CAPTURE", m.group(1)
    return "FALLBACK-COMMAND", ""


def _var_is_read(body: str, var: str) -> bool:
    """Is ``$var`` read anywhere in the run body (beyond the assignment itself)?

    A capture nobody reads is a swallow with extra steps.
    """
    return bool(re.search(r"\$\{?" + re.escape(var) + r"\b", body))


def _var_is_emptiness_checked(body: str, var: str) -> bool:
    """Is ``$var``'s value guarded against the empty string the swallow produced?

    ``V="$(cmd || true)"`` always succeeds; the failure signal moves into "V is empty".
    That is handled iff something downstream looks at emptiness.
    """
    pats = (
        r"-[zn]\s+\"?\$\{?" + re.escape(var),
        r"\$\{" + re.escape(var) + r"(:-|:\+|:\?)",
        r"\"?\$\{?" + re.escape(var) + r"\}?\"?\s*(==?|!=)\s*[\"']{2}",
    )
    return any(re.search(p, body) for p in pats)


def _output_is_consumed(body: str, var: str, step_id: str | None, wf_text: str) -> bool:
    """Is ``$var`` published as a step OUTPUT that some expression downstream reads?

    The emptiness check does not have to live in the same ``run:`` body. The live
    corpus has ``BASE="$(git rev-parse HEAD^ … || echo '')"`` followed by
    ``echo "base_sha=$BASE" >> "$GITHUB_OUTPUT"``, with the next step guarded by
    ``if: … && steps.base.outputs.base_sha != ''``. That failure IS handled — one
    step later — and a guard that cannot see it would be reporting a non-defect.
    """
    if not step_id:
        return False
    for m in re.finditer(
        r"([A-Za-z0-9_-]+)=\$\{?" + re.escape(var) + r"\}?[\"']?\s*>>\s*[\"']?\$\{?GITHUB_OUTPUT",
        body,
    ):
        if f"steps.{step_id}.outputs.{m.group(1)}" in wf_text:
            return True
    return False


def classify_run_body(
    body: str, step_id: str | None = None, wf_text: str = ""
) -> list[tuple[str, str, str]]:
    """``[(kind, detail, snippet)]`` for one ``run:`` body.

    Every ``||`` / ``set +`` construct is classified, benign ones included, so
    ``--census`` can show the split that justifies the finding rules.
    """
    out: list[tuple[str, str, str]] = []
    for raw in logical_lines(body):
        line = strip_comment(raw).strip()
        if not line:
            continue
        ors = scan_or(line)
        tops = [pos for pos, depth in ors if depth == 0]
        nested = [pos for pos, depth in ors if depth > 0]
        if tops:
            if CONTROL_HEAD_RE.match(line):
                out.append(("BOOLEAN-TEST", "`||` inside a control-flow condition", line[:180]))
                continue
            kind, detail = _rhs_kind(line[tops[-1] + 2 :])
            if kind == "CAPTURE":
                var = detail
                # The assignment itself is `|| rc=$?`, which contains no `$rc`, so ANY
                # `$rc` in the body is a genuine read.
                if _var_is_read(body, var):
                    out.append(("CAPTURE-READ", f"captured into `{var}`, read downstream", line[:180]))
                else:
                    out.append(
                        (
                            "CAPTURE-UNREAD",
                            f"`|| {var}=…` captures the failure into `{var}`, which is never read",
                            line[:180],
                        )
                    )
            else:
                out.append((kind, detail, line[:180]))
            continue
        # No top-level `||`: is one hidden inside a command substitution?
        if nested:
            m = SUBST_ASSIGN_RE.search(line)
            inner_kind, inner_detail = _rhs_kind(line[nested[-1] + 2 :].rstrip(")\"' "))
            if inner_kind != "SWALLOW":
                out.append(("FALLBACK-COMMAND", "`||` inside a substitution, non-swallowing tail", line[:180]))
            elif m and _var_is_emptiness_checked(body, m.group(1)):
                out.append(("SUBSHELL-CHECKED", f"`{m.group(1)}` is emptiness-checked downstream", line[:180]))
            elif m and _output_is_consumed(body, m.group(1), step_id, wf_text):
                out.append(
                    (
                        "SUBSHELL-CHECKED",
                        f"`{m.group(1)}` is published as a step output that a later `if:` guards on",
                        line[:180],
                    )
                )
            else:
                out.append(
                    (
                        "SUBSHELL-SWALLOW-UNCHECKED",
                        f"`$( … {inner_detail} )` and the result is never checked for the empty value",
                        line[:180],
                    )
                )

    # `set +e` windows, in body order.
    open_at: str | None = None
    window: list[str] = []
    for raw in body.splitlines():
        line = strip_comment(raw)
        if not line.strip():
            continue
        if RELAX_ERRMODE_RE.match(line):
            open_at = line.strip()
            window = []
            continue
        if open_at is not None:
            if SET_RESTORE_RE.match(line):
                if any("=$?" in w or "$?" in w for w in window):
                    out.append(("SCOPED-SET+", "restored, and the window's status is captured", open_at))
                else:
                    out.append(
                        (
                            "UNCAPTURED-SET+",
                            "the relaxed window restores `set -e` but never captures `$?`, "
                            "so every failure inside it is lost",
                            open_at,
                        )
                    )
                open_at = None
                continue
            window.append(line.strip())
    if open_at is not None:
        out.append(
            (
                "UNSCOPED-SET+",
                "`set +…` is never restored, so every later command in this step is un-gated",
                open_at,
            )
        )
    return out


# ---------------------------------------------------------------------------
# Corpus walk
# ---------------------------------------------------------------------------

def _triggers(wf: dict) -> set[str]:
    on = wf.get("on", wf.get(True))
    if isinstance(on, str):
        return {on}
    if isinstance(on, list):
        return {str(k) for k in on}
    if isinstance(on, dict):
        return {str(k) for k in on}
    return set()


def fingerprint(workflow: str, job: str, kind: str, snippet: str) -> str:
    """Content fingerprint of one construct — the ratchet's key.

    Keyed on the normalised text, not on a path plus a line number, so an exemption
    cannot drift onto a DIFFERENT construct later added to the same step (#4385's
    allowlist design), and editing the exempted command invalidates the exemption.
    """
    norm = " ".join(snippet.split())
    payload = f"{workflow}\n{job}\n{kind}\n{norm}"
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()[:16]


def scan_corpus(workflows_dir: Path) -> tuple[list[dict], list[str], int, int]:
    """``(records, hard_errors, workflows_scanned, run_steps_scanned)``.

    An unreadable or malformed workflow is a HARD error, never a silent skip: "we could
    not look" must not render as "nothing found".
    """
    records: list[dict] = []
    errors: list[str] = []
    n_workflows = 0
    n_steps = 0
    for path in sorted(workflows_dir.glob("*.yml")) + sorted(workflows_dir.glob("*.yaml")):
        n_workflows += 1
        try:
            wf_text = path.read_text(encoding="utf-8")
            wf = yaml.safe_load(wf_text)
        except (OSError, yaml.YAMLError) as exc:
            errors.append(f"{path.name}: unreadable ({exc})")
            continue
        if not isinstance(wf, dict):
            errors.append(f"{path.name}: not a workflow mapping")
            continue
        pr_gating = bool(GATING_EVENTS & _triggers(wf))
        for job_id, job in (wf.get("jobs") or {}).items():
            if not isinstance(job, dict):
                continue
            job_name = str(job.get("name", job_id))
            job_advisory = bool(ADVISORY_RE.search(job_name))
            for idx, step in enumerate(job.get("steps") or []):
                if not isinstance(step, dict):
                    continue
                body = step.get("run")
                if not body:
                    continue
                n_steps += 1
                step_name = str(step.get("name", f"step[{idx}]"))
                gating = pr_gating and not job_advisory
                found = classify_run_body(body, step.get("id"), wf_text)
                if step.get("continue-on-error"):
                    found.append(
                        ("CONTINUE-ON-ERROR", "the step's failure never reaches the job", "continue-on-error: true")
                    )
                shell = step.get("shell")
                if shell is not None and shell not in SAFE_SHELLS:
                    found.append(
                        ("UNSAFE-SHELL", f"`shell: {shell}` is not the default bash -eo pipefail", f"shell: {shell}")
                    )
                for kind, detail, snippet in found:
                    records.append(
                        {
                            "workflow": path.name,
                            "job": job_id,
                            "job_name": job_name,
                            "step": step_name,
                            "kind": kind,
                            "detail": detail,
                            "snippet": snippet,
                            "gating": gating,
                            "pr_gating": pr_gating,
                            "job_advisory": job_advisory,
                            "fp": fingerprint(path.name, job_id, kind, snippet),
                        }
                    )
    return records, errors, n_workflows, n_steps


def load_allowlist(path: Path) -> tuple[dict, list[str]]:
    if not path.is_file():
        return {}, [f"{path.name}: allowlist file not found"]
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        return {}, [f"{path.name}: unreadable ({exc})"]
    entries = raw.get("entries")
    if not isinstance(entries, dict):
        return {}, [f"{path.name}: missing an `entries` object"]
    problems = []
    for fp, entry in entries.items():
        if not isinstance(entry, dict):
            problems.append(f"{path.name}: entry {fp} is not an object")
            continue
        missing = [f for f in REQUIRED_ALLOWLIST_FIELDS if not entry.get(f)]
        if missing:
            problems.append(f"{path.name}: entry {fp} is missing {', '.join(missing)}")
    return entries, problems


# ---------------------------------------------------------------------------
# Reporting
# ---------------------------------------------------------------------------

def render_census(records: list[dict]) -> str:
    lines = ["kind                        gating  advisory-job  non-PR-workflow  total"]
    kinds = sorted({r["kind"] for r in records})
    for kind in kinds:
        rows = [r for r in records if r["kind"] == kind]
        gate = sum(1 for r in rows if r["gating"])
        adv = sum(1 for r in rows if r["pr_gating"] and r["job_advisory"])
        nonpr = sum(1 for r in rows if not r["pr_gating"])
        mark = "*" if kind in FINDING_KINDS else " "
        lines.append(f"{mark}{kind:<27}{gate:>6}{adv:>14}{nonpr:>17}{len(rows):>7}")
    lines.append("")
    lines.append("(* = finding class in a gating step; others are structurally sound)")
    for r in sorted(records, key=lambda r: (r["workflow"], r["job"], r["kind"])):
        if r["kind"] in FINDING_KINDS and r["gating"]:
            lines.append(f"  {r['fp']}  {r['workflow']}:{r['job']}  {r['kind']}  {r['snippet'][:110]}")
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--self-test", action="store_true", help="hermetic both-direction teeth")
    ap.add_argument("--census", action="store_true", help="print the full classified population and exit 0")
    ap.add_argument(
        "--check-allowlist",
        action="store_true",
        help="HARD: allowlist schema + stale-entry ratchet + corpus floors (no finding verdict)",
    )
    ap.add_argument("--check-corpus", action="store_true", help="report un-exempted findings in gating steps")
    ap.add_argument("--advisory", action="store_true", help="report findings but exit 0 (promotion path, see --help)")
    ap.add_argument("--workflows-dir", type=Path, default=ROOT / ".github" / "workflows")
    ap.add_argument("--allowlist", type=Path, default=DEFAULT_ALLOWLIST)
    ap.add_argument(
        "--min-workflows",
        type=int,
        default=0,
        help="fail if fewer workflows were scanned — a glob typo must red, not pass clean",
    )
    ap.add_argument(
        "--min-run-steps",
        type=int,
        default=0,
        help="fail if fewer run: steps were scanned, for the same reason",
    )
    args = ap.parse_args()

    if args.self_test:
        return self_test()
    if not (args.census or args.check_allowlist or args.check_corpus):
        ap.print_help()
        return 2

    if not args.workflows_dir.is_dir():
        print(f"check-workflow-swallow: {args.workflows_dir} is not a directory")
        return 1

    records, errors, n_workflows, n_steps = scan_corpus(args.workflows_dir)
    entries, allow_problems = load_allowlist(args.allowlist)

    if args.census:
        print(f"check-workflow-swallow census: {n_workflows} workflow(s), {n_steps} run: step(s)")
        print(render_census(records))
        return 0

    hard: list[str] = list(errors)
    if n_workflows < args.min_workflows:
        hard.append(
            f"scanned only {n_workflows} workflow(s), floor is {args.min_workflows} — "
            f"the corpus glob is not reaching the workflows"
        )
    if n_steps < args.min_run_steps:
        hard.append(
            f"scanned only {n_steps} run: step(s), floor is {args.min_run_steps} — "
            f"the step walk is not reaching the run bodies"
        )

    live_fps = {r["fp"] for r in records if r["kind"] in FINDING_KINDS and r["gating"]}

    if args.check_allowlist:
        hard.extend(allow_problems)
        stale = sorted(set(entries) - live_fps)
        for fp in stale:
            entry = entries.get(fp) or {}
            hard.append(
                f"STALE EXEMPTION {fp} ({entry.get('workflow', '?')}:{entry.get('job', '?')} "
                f"{entry.get('kind', '?')}) no longer matches anything in the corpus — the construct it "
                f"exempts was changed or removed, so DELETE the entry. The allowlist ratchets: an "
                f"exemption must not outlive its defect."
            )
        if hard:
            print("check-workflow-swallow: allowlist hygiene FAILED:")
            for h in hard:
                print(f"  {h}")
            return 1
        print(
            f"check-workflow-swallow: allowlist clean — {len(entries)} exemption(s), all still live; "
            f"{n_workflows} workflow(s) / {n_steps} run: step(s) scanned (floors {args.min_workflows}/"
            f"{args.min_run_steps}) — OK"
        )
        return 0

    findings = [r for r in records if r["kind"] in FINDING_KINDS and r["gating"] and r["fp"] not in entries]
    exempt = sum(1 for r in records if r["kind"] in FINDING_KINDS and r["gating"] and r["fp"] in entries)

    if hard:
        print("check-workflow-swallow: the scan is INCOMPLETE, so it proves nothing:")
        for h in hard:
            print(f"  {h}")
        return 1

    if findings:
        label = "ADVISORY" if args.advisory else "FAILED"
        print(
            f"check-workflow-swallow: {label} — {len(findings)} gating step(s) discard an earned failure "
            f"({exempt} allowlisted, {n_workflows} workflow(s) / {n_steps} run: step(s) scanned):"
        )
        for r in findings:
            print(f"  {r['workflow']}:{r['job']} [{r['kind']}] {r['step']}")
            print(f"    {r['detail']}")
            print(f"    {r['snippet'][:150]}")
            print(f"    fingerprint {r['fp']} — allowlist it only with a reason and promotion criteria")
        return 0 if args.advisory else 1

    print(
        f"check-workflow-swallow: no gating step discards an earned failure "
        f"({exempt} allowlisted, {n_workflows} workflow(s) / {n_steps} run: step(s) scanned) — OK"
    )
    return 0


# ---------------------------------------------------------------------------
# Hermetic teeth
# ---------------------------------------------------------------------------

_CASES: list[tuple[str, str, str]] = [
    # (label, expected kind, run body)
    ("plain `|| true`", "SWALLOW", "python3 scripts/gate.py --check || true"),
    (
        "Y11 `|| true` across a backslash continuation",
        "SWALLOW",
        "python3 scripts/gate.py --check \\\n  || true\n",
    ),
    ("`|| :`", "SWALLOW", "python3 scripts/gate.py || :"),
    ("message-only fallback", "SWALLOW", 'python3 scripts/gate.py || echo "::warning::soft"'),
    (
        "block fallback that exits 0",
        "SWALLOW",
        'python3 scripts/gate.py || { echo "::warning::non-fatal"; exit 0; }',
    ),
    ("bare `|| exit 0`", "SWALLOW", "python3 scripts/gate.py || exit 0"),
    ("block fallback that exits 1", "AMPLIFIER", 'python3 scripts/gate.py || { echo "::error::x"; exit 1; }'),
    ("`|| exit \"$rc\"`", "AMPLIFIER", 'python3 scripts/gate.py || exit "$rc"'),
    ("bracket test or bracket test", "BOOLEAN-TEST", '[ -n "$A" ] || [ -n "$B" ]'),
    ("`if a || b; then`", "BOOLEAN-TEST", 'if [ ! -f x ] || ! cmp -s a b; then\n  echo differ\nfi'),
    ("capture that is read", "CAPTURE-READ", 'cargo test || rc=$?\nif [ "$rc" -ne 0 ]; then exit "$rc"; fi'),
    ("capture that is never read", "CAPTURE-UNREAD", "cargo test || rc=$?\necho done"),
    ("fallback command", "FALLBACK-COMMAND", "git fetch --depth=1 origin a || git fetch origin a"),
    (
        "substitution swallow, emptiness-checked",
        "SUBSHELL-CHECKED",
        'PREV="$(gh api foo --jq .x 2>/dev/null || echo "")"\nif [ -z "$PREV" ]; then exit 0; fi',
    ),
    (
        "substitution swallow, never checked",
        "SUBSHELL-SWALLOW-UNCHECKED",
        'SUBJECT="$(git log -1 --format=%s 2>/dev/null || true)"\necho "$SUBJECT"',
    ),
    ("scoped `set +e` with capture", "SCOPED-SET+", "set +e\ncargo test\nrc=$?\nset -e\nexit \"$rc\""),
    ("`set +e` never restored", "UNSCOPED-SET+", "set +e\ncargo test\necho done"),
    ("`set +e` restored but never captured", "UNCAPTURED-SET+", "set +e\ncargo test\nset -e\necho done"),
    # Comments must not be read as code — a prose comment explaining `|| rc=$?`
    # inflated the naive census by 5 occurrences on the live corpus.
    ("a comment mentioning `|| true`", None, "# the old code said `python3 gate.py || true`\npython3 gate.py"),
    ("quoted `||` inside a string", None, 'echo "use a || b in the docs"'),
]

_CLEAN_WORKFLOW = """
on: [pull_request]
jobs:
  gates:
    name: real gates
    steps:
      - name: gate
        run: python3 scripts/gate.py --check
"""

_MUTANT_WORKFLOWS: list[tuple[str, str, bool]] = [
    (
        "gating job, `|| true` on a continuation line",
        """
on: [pull_request]
jobs:
  gates:
    name: real gates
    steps:
      - name: gate
        run: |
          python3 scripts/gate.py --check \\
            || true
""",
        True,
    ),
    (
        "same construct in an ADVISORY-named job is not a finding",
        """
on: [pull_request]
jobs:
  gates:
    name: mutation ratchet (advisory)
    steps:
      - name: gate
        run: |
          python3 scripts/gate.py --check \\
            || true
""",
        False,
    ),
    (
        "same construct in a workflow with no PR trigger is not a finding",
        """
on:
  schedule:
    - cron: '0 3 * * *'
jobs:
  gates:
    name: nightly
    steps:
      - name: gate
        run: python3 scripts/gate.py --check || true
""",
        False,
    ),
    (
        "continue-on-error on a gating step",
        """
on: [pull_request]
jobs:
  gates:
    name: real gates
    steps:
      - name: gate
        continue-on-error: true
        run: python3 scripts/gate.py --check
""",
        True,
    ),
    (
        # Mention vs claim. A step whose NAME contains "advisory" but which swallows
        # nothing is not a defect. #4385's gate flagged its own vocabulary table for
        # exactly this reason; a name-based rule here would flag three real steps in
        # docs-quality.yml whose titles merely say "advisory-registry".
        "a step whose NAME mentions advisory but swallows nothing",
        """
on: [pull_request]
jobs:
  gates:
    name: real gates
    steps:
      - name: Enforce advisory-job registry (C2/C3) — GATING
        run: python3 scripts/check-advisory-registry.py
""",
        False,
    ),
    (
        # The emptiness check may live one step away, in a downstream `if:`.
        "substitution swallow published as an output a later `if:` guards on",
        """
on: [pull_request]
jobs:
  gates:
    name: real gates
    steps:
      - name: resolve base
        id: base
        run: |
          BASE="$(git rev-parse HEAD^ 2>/dev/null || echo '')"
          echo "base_sha=$BASE" >> "$GITHUB_OUTPUT"
      - name: use it
        if: steps.base.outputs.base_sha != ''
        run: echo use
""",
        False,
    ),
    (
        "…the same, with nothing downstream reading the output",
        """
on: [pull_request]
jobs:
  gates:
    name: real gates
    steps:
      - name: resolve base
        id: base
        run: |
          BASE="$(git rev-parse HEAD^ 2>/dev/null || echo '')"
          echo "base_sha=$BASE" >> "$GITHUB_OUTPUT"
      - name: use it
        run: echo use
""",
        True,
    ),
    (
        "merge_group-only workflow still gates",
        """
on: [merge_group]
jobs:
  gates:
    name: real gates
    steps:
      - name: gate
        run: python3 scripts/gate.py || true
""",
        True,
    ),
]


def self_test() -> int:
    failures: list[str] = []

    for label, expected, body in _CASES:
        kinds = [k for k, _, _ in classify_run_body(body)]
        if expected is None:
            if kinds:
                failures.append(f"{label}: expected NO construct, got {kinds}")
        elif expected not in kinds:
            failures.append(f"{label}: expected {expected}, got {kinds or ['nothing']}")

    with tempfile.TemporaryDirectory() as td:
        wfdir = Path(td) / "workflows"
        wfdir.mkdir()

        # Clean control: without it every assertion below would pass a checker that
        # simply always reports a finding.
        (wfdir / "clean.yml").write_text(_CLEAN_WORKFLOW, encoding="utf-8")
        records, errors, n_wf, n_steps = scan_corpus(wfdir)
        if errors:
            failures.append(f"clean control produced errors: {errors}")
        if any(r["kind"] in FINDING_KINDS and r["gating"] for r in records):
            failures.append(f"clean control was flagged: {records}")
        if (n_wf, n_steps) != (1, 1):
            failures.append(f"clean control: expected 1 workflow / 1 step, got {n_wf}/{n_steps}")

        for label, body, should_flag in _MUTANT_WORKFLOWS:
            (wfdir / "clean.yml").write_text(body, encoding="utf-8")
            records, _, _, _ = scan_corpus(wfdir)
            flagged = [r for r in records if r["kind"] in FINDING_KINDS and r["gating"]]
            if should_flag and not flagged:
                failures.append(f"MUTANT SURVIVED: {label}")
            if not should_flag and flagged:
                failures.append(f"FALSE POSITIVE: {label} -> {[r['kind'] for r in flagged]}")

        # A malformed workflow must be a HARD error, never a silent skip.
        (wfdir / "clean.yml").write_text(_CLEAN_WORKFLOW, encoding="utf-8")
        (wfdir / "broken.yml").write_text("jobs:\n  a:\n   - [\n", encoding="utf-8")
        _, errors, _, _ = scan_corpus(wfdir)
        if not errors:
            failures.append("an unparseable workflow was silently skipped instead of erroring")
        (wfdir / "broken.yml").unlink()

        # The fingerprint must move when the exempted TEXT changes, and must not
        # collide across jobs — otherwise an exemption drifts onto a new defect.
        a = fingerprint("w.yml", "j", "SWALLOW", "gate.py || true")
        b = fingerprint("w.yml", "j", "SWALLOW", "gate.py  ||  true")
        c = fingerprint("w.yml", "j", "SWALLOW", "other.py || true")
        d = fingerprint("w.yml", "k", "SWALLOW", "gate.py || true")
        if a != b:
            failures.append("fingerprint is whitespace-sensitive; reformatting would break exemptions")
        if a == c or a == d:
            failures.append("fingerprint does not separate different constructs/jobs")

    if failures:
        print("check-workflow-swallow.py self-test FAILED:")
        for f in failures:
            print(f"  - {f}")
        return 1
    print(
        f"check-workflow-swallow.py self-test: OK "
        f"({len(_CASES)} classification cases, {len(_MUTANT_WORKFLOWS)} corpus mutants + clean control)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
