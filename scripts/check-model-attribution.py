#!/usr/bin/env python3
"""Model-attribution honesty gate — inline ``[MODEL]`` markers must not lie.

Background (issues #2504, #2531, maintainer report on PR #4361)
---------------------------------------------------------------
``AGENTS.md`` § *The sub-agent shared contract* item 5 requires every agent to
stamp **the model that ACTUALLY authored the change**: an inline ``[MODEL]``
marker on new code/notes **plus** a ``Co-Authored-By: Claude MODEL`` commit
trailer. The two are meant to agree. They drifted:

* ``.claude/agents/sparq-rust-impl.md`` hard-codes *"you are Sonnet … tag your
  notes/code with ``[SONNET-4.6]``"*. That brief is appended to the system
  prompt via ``--append-system-prompt-file`` **regardless of which model the
  harness actually routes to**, so an Opus 5 worker dutifully stamps
  ``[SONNET-4.6]`` on code Sonnet never wrote. PR #4361 (routed to
  ``sparq-rust-impl`` on ``opus5``) added 22 such markers.
* Because the marker is hand-typed prose while the trailer is machine-emitted,
  only the trailer is trustworthy — exactly the duplicated-source-of-truth
  failure ``check-no-perf-numbers.py`` exists to prevent.

The markers are not decoration: they are the re-review signal ("go re-review
everything the weak/deprecated model wrote"). A *false* marker therefore does
active harm — it both hides real weak-model work and slanders good work.

What this gate does
-------------------
Three independent modes, each usable on its own:

``--check-commits <range>``
    The **recurrence backstop**, and the one that needs no maintainer action.
    For every non-merge commit in ``<range>``, every model marker the commit
    *ADDS* must name the same model as that commit's own ``Co-Authored-By``
    trailer. A commit that adds a marker while declaring a different model —
    or while declaring no model at all — FAILS. This catches the symptom at
    the point of landing no matter which brief, harness, or model generation
    caused it, and it treats ``[OPUS-4.8]``/``[OPUS-5]`` exactly like
    ``[SONNET-4.6]`` so the next deprecation does not re-run this incident.

``--check-commits-pr``
    The **CI form of the above**, and the only one that is correct there. Under
    ``actions/checkout`` a rev range against ``HEAD`` scans a shallow-grafted
    merge ref whose diff is the entire repository — see the long comment above
    :func:`check_commits_pr`. This mode rebuilds the range from the
    ``github.event.pull_request`` payload instead. A commit it cannot diff is
    reported as INCOMPLETE, never as a pass.

``--check-briefs``
    The **root cause**. ``.claude/agents/*.md`` must not hard-code a concrete
    model attribution; AGENTS.md rule 11 already says so in prose, nothing
    enforced it. Two sub-rules, R1 and R2, documented at their implementations.

``--blame <path>...``
    The **replacement capability**. Reports each marker's git-derived truth
    (the introducing commit's trailer) next to what the marker claims, so
    "re-review everything Sonnet wrote" stays answerable from authoritative
    data rather than from the prose that was measured to be wrong. Supports
    ``--model`` to filter and ``--only-mismatched``.

Escape hatch: a line carrying ``model-attribution-allow: <why>`` is exempt,
matching the house convention (``privacy-claims-allow``, ``terminology-allow``,
``perf-neutrality-allow``). A reason is REQUIRED — a bare marker does not
exempt, so the hatch cannot be used as a silent mute.

This file necessarily contains literal marker strings (the vocabulary table, the
fixtures) while attributing nothing, so it carries the file-level directive:
model-attribution-exempt: defines and tests the marker vocabulary itself

Stdlib-only, no network. ``--self-test`` runs hermetic both-direction teeth.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

# --------------------------------------------------------------------------
# The marker vocabulary.
#
# Canonical marker literal -> the set of Co-Authored-By trailer spellings that
# legitimately correspond to it. Trailers are matched case-insensitively and
# after stripping a trailing "(1M context)" style qualifier, so
# "Claude Opus 4.8 (1M context)" and "Claude Opus 4.8" are one model.
#
# Keep this table in sync with AGENTS.md § contract item 5 and the canonical
# per-tier table in .claude/workflows/fable-architect-drain.js. Adding a new
# model tier means adding it HERE too, or its markers become unrecognised (and
# unrecognised markers are ignored, not failed — see MARKER_RE).
# --------------------------------------------------------------------------
MODEL_ALIASES: dict[str, tuple[str, ...]] = {
    "SONNET-4.6": ("sonnet 4.6", "sonnet-4.6", "sonnet4.6"),
    "SONNET-4.5": ("sonnet 4.5", "sonnet-4.5"),
    "OPUS-4.8": ("opus 4.8", "opus-4.8", "opus4.8"),
    "OPUS-5": ("opus 5", "opus-5", "opus5"),
    "FABLE-5": ("fable 5", "fable-5", "fable5", "fable"),
    "HAIKU-4.5": ("haiku 4.5", "haiku-4.5", "haiku4.5"),
    "SOL": ("sol",),
}

# Alternate spellings of a marker that mean the same model. Commit SUBJECTS use
# the lowercase short form ("feat: … [opus5]"); inline code comments use the
# bracketed uppercase form. Both are markers and both must be honest.
MARKER_SYNONYMS: dict[str, str] = {
    "SONNET-4.6": "SONNET-4.6",
    "SONNET-4.5": "SONNET-4.5",
    "SONNET": "SONNET-4.6",
    "SONNET4.6": "SONNET-4.6",
    "OPUS-4.8": "OPUS-4.8",
    "OPUS4.8": "OPUS-4.8",
    "OPUS-5": "OPUS-5",
    "OPUS5": "OPUS-5",
    "FABLE-5": "FABLE-5",
    "FABLE5": "FABLE-5",
    "FABLE": "FABLE-5",
    "HAIKU-4.5": "HAIKU-4.5",
    "HAIKU4.5": "HAIKU-4.5",
    "HAIKU": "HAIKU-4.5",
    "SOL": "SOL",
}

# A model marker is a bracketed token from the vocabulary above and NOTHING
# else. Deliberately strict: an unknown bracketed token (`[TODO]`, `[solid]`,
# `[1m]`) is not a model marker and is ignored rather than guessed at, so this
# gate never fails a PR over prose it does not understand.
MARKER_RE = re.compile(
    r"\[(" + "|".join(sorted((re.escape(k) for k in MARKER_SYNONYMS), key=len, reverse=True)) + r")\]",
    re.IGNORECASE,
)

TRAILER_RE = re.compile(r"^\s*co-authored-by:\s*claude\s+(.+?)\s*(?:<[^>]*>)?\s*$", re.IGNORECASE | re.MULTILINE)

# In a COMMIT MESSAGE the trailer is its own line, so TRAILER_RE anchors. In a
# BRIEF the same string is quoted mid-sentence inside markdown emphasis —
# "commit with **`Co-Authored-By: Claude Sonnet 4.6 <noreply@…>`**" — so R1
# needs an unanchored form that stops at the address, a backtick, or emphasis.
BRIEF_TRAILER_RE = re.compile(r"co-authored-by:\s*claude\s+([^<`*_\n]+)", re.IGNORECASE)

# The documented placeholder in AGENTS.md item 5: `Co-Authored-By: Claude MODEL
# <noreply@anthropic.com>`. A brief may use the placeholder; naming a CONCRETE
# model is the defect.
PLACEHOLDER_TRAILERS = {"model", "<model>", "{model}", "$model", "model-name"}

ALLOW_RE = re.compile(r"model-attribution-allow:\s*\S+")

# File-level directive: "this file DISCUSSES model markers rather than claiming
# authorship with them." Needed because the gate is otherwise unable to tell an
# attribution from a mention — this script's own vocabulary table, its test
# fixtures, and the AGENTS.md paragraph explaining the convention all contain
# literal `[SONNET-4.6]` strings while attributing nothing. Without this, the
# gate flags every file that documents it, including itself. A reason is
# required, and the directive is greppable so its use stays reviewable.
FILE_EXEMPT_RE = re.compile(r"model-attribution-exempt:\s*\S+")

ALLOWLIST_PATH = "scripts/check-model-attribution.allowlist.json"


def line_fingerprint(line: str) -> str:
    """sha256 of the line with whitespace collapsed.

    Content-keyed, not line-numbered: line numbers drift with any edit above
    them, and a path-only key would silently exempt a *different* violation
    later added to the same file. Touch the line at all and the exemption
    lapses, which is the fail-closed direction.
    """
    import hashlib

    return hashlib.sha256(" ".join(line.split()).encode("utf-8")).hexdigest()


def load_allowlist(root: Path) -> tuple[dict[str, dict], list[str]]:
    """Return ``(fingerprint -> entry, errors)``.

    A malformed or unreadable allowlist is an ERROR, never a silent empty
    allowlist and never a silent pass: a corrupt allowlist must not be able to
    either mute the gate or wedge it invisibly.
    """
    import json

    path = root / ALLOWLIST_PATH
    if not path.exists():
        return {}, []
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        return {}, [f"{ALLOWLIST_PATH}: unreadable/invalid JSON ({exc}) — refusing to run fail-open"]
    errors: list[str] = []
    out: dict[str, dict] = {}
    for i, entry in enumerate(data.get("entries", [])):
        where = f"{ALLOWLIST_PATH}[{i}]"
        fp = str(entry.get("fingerprint", "")).strip()
        if not entry.get("reason", "").strip():
            errors.append(f"{where}: entry needs a non-empty 'reason'")
        if not entry.get("tracking", "").strip():
            errors.append(f"{where}: entry needs a non-empty 'tracking' issue/bead")
        if not fp or fp == "PLACEHOLDER":
            errors.append(f"{where}: entry needs a real 'fingerprint' (run --print-fingerprints)")
            continue
        out[fp] = entry
    return out, errors

_QUALIFIER = r"\((?:1m\s+context|[^)]*context[^)]*)\)"


def normalise_marker(raw: str) -> str:
    """Map any marker spelling to its canonical form (`opus5` -> `OPUS-5`)."""
    return MARKER_SYNONYMS[raw.upper()]


def normalise_trailer_model(raw: str) -> str | None:
    """Map a ``Co-Authored-By: Claude <X>`` value to a canonical marker name.

    Returns ``None`` for a value that names no model we know (including the
    bare ``Co-authored-by: Claude`` seen once in history) so callers can treat
    "undeclared" distinctly from "declared something else".
    """
    value = re.sub(_QUALIFIER, "", raw, flags=re.IGNORECASE).strip().lower()
    if not value or value in PLACEHOLDER_TRAILERS:
        return None
    for canonical, aliases in MODEL_ALIASES.items():
        if value in aliases:
            return canonical
    # Tolerate a trailing version drift ("opus 5.1") by longest-prefix match, so
    # a point release does not read as an unknown model and silently fail open.
    for canonical, aliases in MODEL_ALIASES.items():
        for alias in aliases:
            if value.startswith(alias):
                return canonical
    return None


CODE_SPAN_RE = re.compile(r"`[^`\n]*`")


def markers_in(text: str) -> list[str]:
    return [normalise_marker(m.group(1)) for m in MARKER_RE.finditer(text)]


def attribution_markers_in(text: str) -> list[str]:
    """Markers that CLAIM authorship, excluding ones merely being discussed.

    A real attribution is written bare — ``// [SONNET-4.6] sq-vkq9u: …``. Prose
    that talks *about* the convention quotes it in an inline code span —
    ``the `[SONNET-4.6]` marker``. Stripping code spans lets documentation,
    changelogs and this gate's own tests discuss markers without being accused
    of mis-attributing anything.

    Deliberately NOT used by the brief lint (R2): a brief that says "tag your
    code with `[SONNET-4.6]`" puts the literal in backticks, and that is exactly
    the defect, so R2 must keep seeing inside code spans.

    Trade-off, stated plainly: an attribution someone writes as `` `[X]` ``
    inside backticks is missed here. That is the safe direction for an
    honesty gate — it under-claims rather than accusing correct work.
    """
    return markers_in(CODE_SPAN_RE.sub(" ", text))


# --------------------------------------------------------------------------
# Mode: --check-commits  (the recurrence backstop)
# --------------------------------------------------------------------------


def _git(args: list[str], cwd: Path | None = None) -> str:
    """Run git and decode leniently.

    ``errors="replace"`` is load-bearing, not defensive boilerplate: a commit
    that touches a binary or latin-1 blob makes ``git show`` emit bytes that are
    not valid UTF-8, and a strict decode raises UnicodeDecodeError mid-scan.
    That crashed this gate on its own first CI run. Markers are pure ASCII, so
    replacing undecodable bytes cannot cause a missed or spurious finding.
    """
    return subprocess.run(
        ["git", *args], cwd=cwd, check=True, capture_output=True,
        encoding="utf-8", errors="replace",
    ).stdout


def commit_declared_model(message: str) -> str | None:
    """The commit's OWN declaration of which model authored it.

    Priority: the ``Co-Authored-By`` trailer (machine-emitted by the harness,
    hence authoritative) and then a subject-line tag such as ``[opus5]``. The
    subject tag is a weaker signal — it is hand-typed like the inline markers —
    so it is only consulted when there is no trailer at all.
    """
    trailers = TRAILER_RE.findall(message)
    for value in trailers:
        model = normalise_trailer_model(value)
        if model:
            return model
    subject = message.splitlines()[0] if message.splitlines() else ""
    subject_markers = markers_in(subject)
    if subject_markers:
        return subject_markers[0]
    return None


def added_lines(repo: Path, sha: str) -> list[tuple[str, str]]:
    """``(path, line)`` for every line this commit ADDS. Merges yield nothing."""
    out = _git(
        ["show", "--format=", "--unified=0", "--no-color", "--no-renames", sha],
        cwd=repo,
    )
    result: list[tuple[str, str]] = []
    path = "?"
    for line in out.splitlines():
        if line.startswith("+++ b/"):
            path = line[6:]
        elif line.startswith("+") and not line.startswith("+++"):
            result.append((path, line[1:]))
    return result


def is_graft_boundary(repo: Path, sha: str) -> bool:
    """Is ``sha`` a SHALLOW-CLONE GRAFT BOUNDARY (parents omitted from the clone)?

    This is the single most important predicate in the commit gate, because
    ``git show`` on a graft boundary emits **the entire tree as additions** —
    the gate then attributes every pre-existing marker in the repository to one
    commit and drowns any real finding in baseline noise. Measured on this PR's
    own head (docs-quality run 89851383239): **15,123 findings**, all attributed
    to ``refs/pull/4385/merge``, spanning files the PR never touched.

    Grafting is applied at the revision-WALK layer, not the object layer, so the
    commit object still records its real parents while the walker reports none.
    That asymmetry is the exact discriminator, and it distinguishes a graft from
    a legitimate ROOT commit (where "the whole tree is an addition" is TRUE and
    must still be scanned):

    ============================  ==================  ==================
    commit                        ``log --format=%P``  ``cat-file`` parents
    ============================  ==================  ==================
    shallow graft boundary        (empty)             present   -> GRAFT
    true root commit              (empty)             (empty)   -> scan it
    ordinary commit               present             present   -> scan it
    ============================  ==================  ==================

    Reading ``.git/shallow`` would also work but is worktree- and layout-
    sensitive; this is pure plumbing and needs no path guessing.
    """
    walked = _git(["log", "-1", "--format=%P", sha], cwd=repo).strip()
    if walked:
        return False
    try:
        raw = _git(["cat-file", "commit", sha], cwd=repo)
    except subprocess.CalledProcessError:
        return False
    for line in raw.splitlines():
        if not line.strip():
            break  # end of the header block; the message starts here
        if line.startswith("parent "):
            return True
    return False


def scan_commits(repo: Path, shas: list[str], tip: str) -> tuple[list[str], list[str]]:
    """Core of the commit gate: ``(violations, unscannable)`` over ``shas``.

    ``unscannable`` is NOT cosmetic. A commit whose diff we cannot compute is a
    HOLE in the backstop, and a backstop that reports "OK" over a hole is worse
    than no backstop — that is the precise failure this function exists to make
    impossible to report as a pass. Callers must surface it.
    """
    violations: list[str] = []
    unscannable: list[str] = []
    exempt_cache: dict[tuple[str, str], bool] = {}

    # Exemption is read at the range TIP, not per-commit. "Does this file
    # discuss markers?" is a property of the file as it ends up, not of each
    # intermediate commit — so adding the directive in a later commit of the
    # same PR correctly covers the earlier ones, and a reviewer sees one
    # consistent answer per file instead of a per-commit patchwork.

    def file_is_exempt(_sha: str, path: str) -> bool:
        key = (tip, path)
        if key not in exempt_cache:
            try:
                body = _git(["show", f"{tip}:{path}"], cwd=repo)
            except subprocess.CalledProcessError:
                body = ""
            exempt_cache[key] = bool(FILE_EXEMPT_RE.search(body))
        return exempt_cache[key]

    for sha in shas:
        if is_graft_boundary(repo, sha):
            unscannable.append(
                f"{sha[:8]}: SHALLOW-GRAFT BOUNDARY — its parents are not in this "
                f"clone, so `git show` would report the ENTIRE TREE as additions. "
                f"NOT scanned (a whole-tree scan is noise, not a signal). Deepen "
                f"the fetch so this commit has a parent, or scan a range whose "
                f"commits are fully present."
            )
            continue
        message = _git(["show", "-s", "--format=%B", sha], cwd=repo)
        declared = commit_declared_model(message)
        for path, line in added_lines(repo, sha):
            if ALLOW_RE.search(line):
                continue
            if file_is_exempt(sha, path):
                continue
            for marker in set(attribution_markers_in(line)):
                if declared is None:
                    violations.append(
                        f"{sha[:8]} {path}: adds marker [{marker}] but the commit "
                        f"declares NO model (no Co-Authored-By: Claude <model> "
                        f"trailer). Add the trailer for the model that actually "
                        f"authored this, or drop the marker."
                    )
                elif marker != declared:
                    violations.append(
                        f"{sha[:8]} {path}: adds marker [{marker}] but the commit "
                        f"declares [{declared}]. Stamp the model that ACTUALLY "
                        f"authored the change (AGENTS.md contract item 5); a brief "
                        f"telling you otherwise is the bug (#2531)."
                    )
    return violations, unscannable


def check_commits(repo: Path, rev_range: str) -> tuple[list[str], list[str]]:
    """Every marker a commit ADDS must name that commit's own declared model.

    Fail-closed on an undeclared commit that stamps a marker: if you are
    stamping provenance you must also declare it, otherwise the stamp is
    unverifiable and this gate would fail open exactly where it matters.

    This is the LOCAL/manual entry point (a rev range you name yourself). CI
    uses :func:`check_commits_pr` instead — see the warning there about why a
    rev range against ``HEAD`` cannot work under ``actions/checkout``.
    """
    # Prefer the merge-base over the raw two-dot range. On a normal clone whose
    # base ref is not an ancestor of HEAD, `A..HEAD` silently expands to most of
    # history — 48 MB of `git show` output on the first run here. Narrowing to
    # the merge-base keeps the scan proportional to the branch.
    #
    # This narrowing is USELESS on a shallow merge-ref checkout: merge-base
    # cannot see past a graft (it exits 1 with no output), which is precisely
    # why the CI path no longer relies on it. The graft guard in `scan_commits`
    # is what makes that case safe.
    if ".." in rev_range and "..." not in rev_range:
        base, _, tip_rev = rev_range.partition("..")
        try:
            merge_base = _git(["merge-base", base, tip_rev or "HEAD"], cwd=repo).strip()
            if merge_base:
                rev_range = f"{merge_base}..{tip_rev or 'HEAD'}"
        except subprocess.CalledProcessError:
            pass
    shas = _git(["rev-list", "--no-merges", rev_range], cwd=repo).split()
    tip = rev_range.split("..")[-1] or "HEAD"
    return scan_commits(repo, shas, tip)


# --------------------------------------------------------------------------
# Mode: --check-commits-pr  (the CI path)
#
# WHY THIS EXISTS AND `--check-commits origin/main..HEAD` DOES NOT WORK IN CI
# --------------------------------------------------------------------------
# `actions/checkout` (default `fetch-depth: 1`) checks out `refs/pull/N/merge`
# and records it in `.git/shallow`. That commit therefore has NO parents as far
# as any revision walk is concerned. Three consequences, all measured on this
# PR's own head (docs-quality run 89851383239, and reproduced end-to-end by
# `test_model_attribution.sh` on a synthetic CI-shaped repo):
#
#   1. A later `git fetch --depth=200 origin main` does NOT deepen it. The merge
#      ref is unreachable from `main`, so nothing about the graft changes.
#   2. `git merge-base origin/main HEAD` exits 1 with no output — narrowing the
#      range via merge-base CANNOT see past a graft. (This is the fix this PR
#      originally claimed; it does not work, and the reviewer was right.)
#   3. `rev-list --no-merges origin/main..HEAD` therefore yields the merge
#      commit itself — a merge that does not look like one, because its parents
#      are missing — and `git show` renders the WHOLE TREE as additions:
#      15,123 findings, every one attributed to `refs/pull/4385/merge`.
#
# So the range must be rebuilt from the PR's OWN head ref, using the fields
# GitHub already puts in the event payload, and the CI step must fetch that ref
# deeply enough (`commits + 1`) that every scanned commit has a real parent.
# --------------------------------------------------------------------------


def github_event() -> dict:
    """The ``pull_request`` event payload, or ``{}`` off CI / on another event."""
    if os.environ.get("GITHUB_EVENT_NAME", "pull_request") != "pull_request":
        return {}
    path = os.environ.get("GITHUB_EVENT_PATH")
    if not path:
        return {}
    try:
        payload = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}
    return payload if isinstance(payload, dict) else {}


def _object_exists(repo: Path, rev: str) -> bool:
    return (
        subprocess.run(
            ["git", "cat-file", "-e", f"{rev}^{{commit}}"],
            cwd=repo,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        ).returncode
        == 0
    )


def pr_scan_set(repo: Path, payload: dict) -> tuple[list[str], str, list[str]]:
    """``(shas, tip, problems)`` — the PR's OWN commits, from the event payload.

    Two derivations, in order of exactness, so a fallback is never silent:

    * ``base.sha..head.sha`` when the base commit is present locally AND is an
      ancestor of the head. Exact.
    * ``rev-list --max-count=<commits> <head.sha>`` otherwise, using GitHub's
      own commit count. Right for the linear branch shape, which is what the
      fleet produces.

    A missing head object is reported as a PROBLEM, never as an empty clean
    scan — "we could not look" must not render as "we looked and it was fine".
    """
    pr = payload.get("pull_request") or {}
    head_sha = ((pr.get("head") or {}).get("sha") or "").strip()
    base_sha = ((pr.get("base") or {}).get("sha") or "").strip()
    n_commits = pr.get("commits")

    if not head_sha:
        return [], "HEAD", [
            "no `pull_request.head.sha` in the event payload — cannot identify "
            "this PR's commits."
        ]
    if not _object_exists(repo, head_sha):
        return [], head_sha, [
            f"{head_sha[:8]}: the PR head commit is not in this clone. The CI step "
            f"must fetch `refs/pull/<n>/head` (depth = commits + 1) before running "
            f"this check; `actions/checkout` only provides the grafted merge ref."
        ]

    if base_sha and _object_exists(repo, base_sha) and subprocess.run(
        ["git", "merge-base", "--is-ancestor", base_sha, head_sha],
        cwd=repo, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False,
    ).returncode == 0:
        shas = _git(
            ["rev-list", "--no-merges", f"{base_sha}..{head_sha}"], cwd=repo
        ).split()
        return shas, head_sha, []

    if isinstance(n_commits, int) and n_commits > 0:
        shas = _git(
            ["rev-list", "--no-merges", f"--max-count={n_commits}", head_sha], cwd=repo
        ).split()
        return shas, head_sha, []

    return [], head_sha, [
        "the event payload has neither a usable `base.sha` nor a `commits` count, "
        "so the PR's own commit range cannot be bounded."
    ]


def check_commits_pr(repo: Path) -> tuple[list[str], list[str], str]:
    """``(violations, problems, description)`` for the current PR event."""
    payload = github_event()
    if not payload.get("pull_request"):
        event = os.environ.get("GITHUB_EVENT_NAME", "(none)")
        return [], [], f"not a pull_request event ({event}) — nothing to check"
    shas, tip, problems = pr_scan_set(repo, payload)
    if problems:
        return [], problems, "this PR's commits"
    violations, unscannable = scan_commits(repo, shas, tip)
    return violations, unscannable, f"this PR's {len(shas)} commit(s)"


# --------------------------------------------------------------------------
# Mode: --check-briefs  (the root cause)
# --------------------------------------------------------------------------

# R2's trigger: a line that instructs the agent about ITS OWN attribution.
# Second person + attribution vocabulary. Kept narrow on purpose — this rule
# only speaks about sentences that tell the agent what to stamp.
SELF_DIRECTIVE_RE = re.compile(
    r"(you are\s+(?:sonnet|opus|fable|haiku|sol)\b"
    r"|tag your\b"
    r"|stamp your\b"
    r"|self-tag\b"
    r"|your (?:inline )?marker\s+is\b"
    r"|commit with\b)",
    re.IGNORECASE,
)

# R2's exculpation: the line is model-AWARE (it points at the running model or
# explicitly forbids literals) rather than hard-coding. These are the phrasings
# the already-fixed briefs use.
MODEL_AWARE_RE = re.compile(
    r"(running model|runtime model|--model\b|never hard-code|not hard-coded"
    r"|never hard-coded|accurate history|derive (?:your|the))",
    re.IGNORECASE,
)


def brief_violations(root: Path) -> list[tuple[str, str, str]]:
    """Raw ``(fingerprint, location, message)`` findings, before allowlisting."""
    findings: list[tuple[str, str, str]] = []
    briefs = sorted((root / ".claude" / "agents").glob("*.md"))
    if not briefs:
        findings.append(("", str(root / ".claude/agents"),
                         "no agent briefs found — gate would be vacuous"))
        return findings
    for brief in briefs:
        rel = brief.relative_to(root)
        for lineno, line in enumerate(brief.read_text(encoding="utf-8").splitlines(), 1):
            if ALLOW_RE.search(line):
                continue
            fp = line_fingerprint(line)
            for match in BRIEF_TRAILER_RE.finditer(line):
                value = re.sub(r"[`*_]", "", match.group(1)).strip().lower()
                if value not in PLACEHOLDER_TRAILERS and normalise_trailer_model(value):
                    findings.append((fp, f"{rel}:{lineno}",
                        f"R1 hard-coded Co-Authored-By trailer names a concrete model "
                        f"({match.group(1).strip()!r}). Briefs are appended to the system "
                        f"prompt regardless of routed model — use the 'Co-Authored-By: "
                        f"Claude MODEL' placeholder and instruct the agent to derive it "
                        f"from the harness's --model (AGENTS.md rule 11)."))
            if SELF_DIRECTIVE_RE.search(line) and not MODEL_AWARE_RE.search(line):
                for marker in sorted(set(markers_in(line))):
                    findings.append((fp, f"{rel}:{lineno}",
                        f"R2 self-attribution directive hard-codes [{marker}]. The agent "
                        f"must derive its marker from the harness's running model, not "
                        f"from this file (AGENTS.md rule 11 / item 5; issues #2504, #2531)."))
    return findings


def check_briefs(root: Path) -> list[str]:
    """Lint ``.claude/agents/*.md`` for hard-coded model attribution.

    R1 — a brief must never spell a CONCRETE model in a ``Co-Authored-By``
    trailer. AGENTS.md documents ``Co-Authored-By: Claude MODEL`` as the
    placeholder form; a concrete name means the brief is dictating attribution
    independent of the harness's routing. No allowlist: this rule has no
    legitimate exception, because the trailer is what reaches git history.

    R2 — a line that directs the agent about its own attribution must not name
    a concrete marker unless it is model-aware (points at the running model, or
    explicitly forbids literals). Authorship STAMPS are untouched: a stamp is
    not a directive, so ``<!-- [OPUS-4.8] … -->`` and ``## Heading [FABLE-5]``
    never trigger R2. That matters — AGENTS.md item 5 says existing stamps are
    accurate history and must never be rewritten.
    """
    allowed, violations = load_allowlist(root)
    if violations:
        return violations
    findings = brief_violations(root)
    seen: set[str] = set()
    for fp, where, msg in findings:
        if fp and fp in allowed:
            seen.add(fp)
            continue
        violations.append(f"{where}: {msg}")
    # RATCHET: an allowlist entry whose violation no longer exists must be
    # DELETED. Without this, a fixed defect leaves a live exemption behind that
    # would silently re-cover the same line if it ever regressed.
    for fp, entry in allowed.items():
        if fp not in seen:
            violations.append(
                f"{ALLOWLIST_PATH}: STALE entry for {entry.get('path', '?')} — the "
                f"violation it exempts is gone (tracking {entry.get('tracking', '?')}). "
                f"Delete the entry; the gate now passes without it."
            )
    return violations


# --------------------------------------------------------------------------
# Mode: --blame  (the git-derived replacement for grep-ability)
# --------------------------------------------------------------------------


def blame_markers(
    repo: Path, paths: list[str], want_model: str | None, only_mismatched: bool
) -> tuple[list[str], dict[str, int]]:
    """Report each marker's CLAIMED model beside its git-derived TRUE model.

    This is what makes "re-review everything <model> wrote" answerable without
    trusting the inline prose: the answer comes from each line's introducing
    commit, which carries the machine-emitted trailer.
    """
    rows: list[str] = []
    tally: dict[str, int] = {}
    cache: dict[str, str | None] = {}
    grep = subprocess.run(
        ["git", "grep", "-nI", "-E", MARKER_RE.pattern, "--", *paths] if paths
        else ["git", "grep", "-nI", "-E", MARKER_RE.pattern],
        cwd=repo,
        capture_output=True,
        text=True,
    )
    for hit in grep.stdout.splitlines():
        try:
            path, lineno, text = hit.split(":", 2)
        except ValueError:
            continue
        claimed = markers_in(text)
        if not claimed:
            continue
        claim = claimed[0]
        if want_model and claim != want_model:
            continue
        blame = subprocess.run(
            ["git", "blame", "-L", f"{lineno},{lineno}", "--porcelain", "--", path],
            cwd=repo,
            capture_output=True,
            text=True,
        )
        if blame.returncode != 0 or not blame.stdout:
            continue
        sha = blame.stdout.split()[0]
        if sha not in cache:
            msg = _git(["show", "-s", "--format=%B", sha], cwd=repo)
            cache[sha] = commit_declared_model(msg)
        truth = cache[sha]
        state = "MATCH" if truth == claim else ("UNKNOWN" if truth is None else "MISMATCH")
        tally[state] = tally.get(state, 0) + 1
        if only_mismatched and state == "MATCH":
            continue
        rows.append(f"{state:8} {path}:{lineno}  claims [{claim}]  git says [{truth or '?'}]  ({sha[:8]})")
    return rows, tally


# --------------------------------------------------------------------------
# Self-test — hermetic, both directions, no network.
# --------------------------------------------------------------------------


def _write(root: Path, rel: str, text: str) -> None:
    p = root / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8")


def self_test() -> int:
    import tempfile

    failures: list[str] = []

    def expect(cond: bool, what: str) -> None:
        if not cond:
            failures.append(what)

    # ---- normalisation -----------------------------------------------------
    expect(normalise_trailer_model("Opus 4.8 (1M context)") == "OPUS-4.8", "trailer qualifier not stripped")
    expect(normalise_trailer_model("Sonnet 4.6") == "SONNET-4.6", "sonnet trailer not mapped")
    expect(normalise_trailer_model("MODEL") is None, "placeholder trailer must not resolve to a model")
    expect(normalise_trailer_model("") is None, "empty trailer must not resolve")
    expect(normalise_marker("opus5") == "OPUS-5", "lowercase subject-tag synonym not mapped")
    expect(markers_in("x [solid] [1m] [TODO] y") == [], "unknown bracket token must not be read as a marker")
    expect(markers_in("// [SONNET-4.6] note") == ["SONNET-4.6"], "inline marker not detected")

    # ---- R1/R2 brief lint --------------------------------------------------
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        # The real defect, verbatim in shape (sparq-rust-impl.md:13).
        _write(root, ".claude/agents/bad.md",
               "- **Model provenance (you are Sonnet, not Opus).** Tag your notes/code with "
               "**`[SONNET-4.6]`** and commit with **`Co-Authored-By: Claude Sonnet 4.6 "
               "<noreply@anthropic.com>`**.\n")
        v = check_briefs(root)
        expect(any("R1" in x for x in v), "R1 missed a concrete hard-coded Co-Authored-By trailer")
        expect(any("R2" in x for x in v), "R2 missed a hard-coded self-attribution directive")

    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        # Every legitimate shape observed across the 20 real briefs must PASS,
        # or the gate would force-rewrite accurate history (AGENTS.md item 5).
        _write(root, ".claude/agents/good.md",
               "<!-- [OPUS-4.8] Single-source: AGENTS.md wins if this drifts. -->\n"
               "## Engine-semantics trap catalog [FABLE-5]\n"
               "[OPUS-4.8] Authored by Opus 4.8 (1M context); flag for re-review.\n"
               "- **Model-attribution markers follow the RUNNING model.** Derive your inline "
               "marker + `Co-Authored-By` trailer from the harness's actual runtime model.\n"
               "- stamp the RUNNING model — `[FABLE-5]`/`[OPUS-4.8]`/etc.; NOT hard-coded literals\n"
               "- Existing `[OPUS-4.8]` stamps in this file are accurate history — leave them.\n"
               "- Use `Co-Authored-By: Claude MODEL <noreply@anthropic.com>`.\n")
        v = check_briefs(root)
        expect(v == [], f"false positive on legitimate brief shapes: {v}")

    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        _write(root, ".claude/agents/allowed.md",
               "Tag your code with `[SONNET-4.6]`. model-attribution-allow: fixture\n")
        expect(check_briefs(root) == [], "documented allow-marker did not exempt the line")
        _write(root, ".claude/agents/allowed.md", "Tag your code with `[SONNET-4.6]`. model-attribution-allow:\n")
        expect(check_briefs(root) != [], "bare allow-marker with NO reason must not exempt")

    expect(check_briefs(Path(td)) != [], "empty agents dir must fail rather than pass vacuously")

    # A corrupt allowlist must fail CLOSED — pinned over a brief that is CLEAN,
    # so this can only be failing on the allowlist. The shell suite's equivalent
    # fixture used to carry a live R2 violation, which meant "the gate returned
    # non-empty" was satisfied by the brief and the fail-open mutant
    # (`return {}, []`) survived the whole suite AND this self-test.
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        _write(root, ".claude/agents/clean.md",
               "- Markers follow the RUNNING model; derive them from the harness.\n")
        expect(check_briefs(root) == [], "CONTROL: the clean brief must pass on its own")
        _write(root, str(ALLOWLIST_PATH), "not json {")
        v = check_briefs(root)
        expect(any("unreadable/invalid JSON" in x for x in v),
               f"a corrupt allowlist did not fail CLOSED over a clean brief: {v}")

    # ---- commit gate -------------------------------------------------------
    with tempfile.TemporaryDirectory() as td:
        repo = Path(td)
        _git(["init", "-q", "-b", "main", str(repo)])
        _git(["config", "user.email", "t@example.com"], cwd=repo)
        _git(["config", "user.name", "t"], cwd=repo)
        _write(repo, "seed.txt", "seed\n")
        _git(["add", "seed.txt"], cwd=repo)
        _git(["commit", "-qm", "seed"], cwd=repo)
        base = _git(["rev-parse", "HEAD"], cwd=repo).strip()

        def cc(rng: str) -> list[str]:
            """check_commits, asserting the range was FULLY scanned.

            Every fixture below is a normal (non-shallow) clone, so an
            unscannable commit here would mean the graft guard is misfiring and
            silently emptying the scan — which would make every expectation
            below pass for the wrong reason.
            """
            violations, unscannable = check_commits(repo, rng)
            expect(unscannable == [], f"normal clone reported unscannable commits: {unscannable}")
            return violations


        # HONEST: marker agrees with the trailer -> must pass.
        _write(repo, "a.rs", "// [OPUS-5] honest\n")
        _git(["add", "a.rs"], cwd=repo)
        _git(["commit", "-qm", "feat: a\n\nCo-Authored-By: Claude Opus 5 <n@a.com>"], cwd=repo)
        expect(cc(f"{base}..HEAD") == [], "honest commit was flagged")
        honest = _git(["rev-parse", "HEAD"], cwd=repo).strip()

        # LYING: the exact reported defect — Opus 5 commit stamping [SONNET-4.6].
        _write(repo, "b.rs", "// [SONNET-4.6] mis-attributed\n")
        _git(["add", "b.rs"], cwd=repo)
        _git(["commit", "-qm", "feat: b\n\nCo-Authored-By: Claude Opus 5 <n@a.com>"], cwd=repo)
        v = cc(f"{honest}..HEAD")
        expect(len(v) == 1 and "SONNET-4.6" in v[0], f"mis-attributed marker not caught: {v}")
        lying = _git(["rev-parse", "HEAD"], cwd=repo).strip()

        # UNDECLARED: stamps a marker with no trailer at all -> fail-closed.
        _write(repo, "c.rs", "// [OPUS-4.8] undeclared\n")
        _git(["add", "c.rs"], cwd=repo)
        _git(["commit", "-qm", "feat: c"], cwd=repo)
        v = cc(f"{lying}..HEAD")
        expect(len(v) == 1 and "declares NO model" in v[0], f"undeclared stamp not caught: {v}")
        undecl = _git(["rev-parse", "HEAD"], cwd=repo).strip()

        # Subject tag is accepted as a weaker declaration when no trailer exists.
        _write(repo, "d.rs", "// [OPUS-5] subject-tagged\n")
        _git(["add", "d.rs"], cwd=repo)
        _git(["commit", "-qm", "feat: d [opus5]"], cwd=repo)
        expect(cc(f"{undecl}..HEAD") == [], "subject-tag declaration not honoured")
        subj = _git(["rev-parse", "HEAD"], cwd=repo).strip()

        # REMOVING a stale marker must never fail (cleanup must stay possible).
        _write(repo, "b.rs", "// cleaned\n")
        _git(["add", "b.rs"], cwd=repo)
        _git(["commit", "-qm", "chore: drop stale marker\n\nCo-Authored-By: Claude Opus 5 <n@a.com>"], cwd=repo)
        expect(cc(f"{subj}..HEAD") == [], "removing a stale marker was flagged")
        cleaned = _git(["rev-parse", "HEAD"], cwd=repo).strip()

        # REGRESSION (caught by this gate's own first CI run): a commit that
        # touches a non-UTF-8 blob must not crash the scan with
        # UnicodeDecodeError. The marker in the same commit must still be found.
        # NOTE: no NUL byte. Git classifies a blob containing NUL as binary and
        # prints "Binary files differ" instead of its contents — which means a
        # NUL-bearing fixture never reaches the decoder and the mutant survives.
        # (Verified: with a NUL in the fixture, reverting errors="replace" left
        # the self-test GREEN.) Latin-1 text is the shape that actually bites.
        (repo / "latin1.txt").write_bytes(b"caf\xe9 \x91\x92\xfe na\xefve\n")
        _write(repo, "f.rs", "// [SONNET-4.6] beside a binary blob\n")
        _git(["add", "latin1.txt", "f.rs"], cwd=repo)
        _git(["commit", "-qm", "feat: f\n\nCo-Authored-By: Claude Opus 5 <n@a.com>"], cwd=repo)
        v = cc(f"{cleaned}..HEAD")
        expect(any("SONNET-4.6" in x for x in v),
               "non-UTF-8 blob broke the scan (or hid a real finding)")
        cleaned = _git(["rev-parse", "HEAD"], cwd=repo).strip()

        # A file that DISCUSSES markers (this script, its tests, the AGENTS.md
        # paragraph) attributes nothing and must not be flagged...
        _write(repo, "doc.md",
               "<!-- model-attribution-exempt: documents the marker convention -->\n"
               "Markers look like `[SONNET-4.6]` or `[OPUS-4.8]`.\n")
        _git(["add", "doc.md"], cwd=repo)
        _git(["commit", "-qm", "docs: explain markers\n\nCo-Authored-By: Claude Opus 5 <n@a.com>"], cwd=repo)
        expect(cc(f"{cleaned}..HEAD") == [],
               "file-level exempt directive did not suppress a pure mention")
        exempted = _git(["rev-parse", "HEAD"], cwd=repo).strip()

        # ...but the directive must be a DELIBERATE per-file act, not the
        # default. Without it, the same content is flagged again.
        # Bare, not backticked: this is a real attribution claim, and with no
        # file-level directive it must still be caught.
        _write(repo, "doc2.md", "[SONNET-4.6] sq-x: wrote this helper.\n")
        _git(["add", "doc2.md"], cwd=repo)
        _git(["commit", "-qm", "docs: no directive\n\nCo-Authored-By: Claude Opus 5 <n@a.com>"], cwd=repo)
        expect(cc(f"{exempted}..HEAD") != [],
               "a file WITHOUT the exempt directive must still be checked")
        # A marker merely DISCUSSED in an inline code span is not an
        # attribution; a BARE one in the same file still is.
        expect(attribution_markers_in("the `[SONNET-4.6]` marker means") == [],
               "backticked mention was read as an attribution")
        expect(attribution_markers_in("// [SONNET-4.6] real stamp") == ["SONNET-4.6"],
               "bare attribution was lost by code-span stripping")
        cleaned = _git(["rev-parse", "HEAD"], cwd=repo).strip()

        # The allow-marker escape hatch works on the commit path too.
        _write(repo, "e.rs", "// [SONNET-4.6] restored verbatim  model-attribution-allow: revert of #1\n")
        _git(["add", "e.rs"], cwd=repo)
        _git(["commit", "-qm", "revert: e\n\nCo-Authored-By: Claude Opus 5 <n@a.com>"], cwd=repo)
        expect(cc(f"{cleaned}..HEAD") == [], "allow-marker did not exempt a commit line")

        # ---- file_is_exempt's UNREADABLE branch must fail CLOSED ------------
        # The exempt directive is read at the range TIP. If a commit ADDS a
        # marker to a file that no longer exists at the tip, `git show tip:path`
        # fails — and treating that as "exempt" would let any PR hide a false
        # marker by deleting the file in a later commit. Pinned because the
        # `return True` mutant survived both the suite and this self-test.
        _write(repo, "gone.rs", "// [SONNET-4.6] added, then the file is deleted\n")
        _git(["add", "gone.rs"], cwd=repo)
        _git(["commit", "-qm", "feat: g\n\nCo-Authored-By: Claude Opus 5 <n@a.com>"], cwd=repo)
        before_delete = _git(["rev-parse", "HEAD~1"], cwd=repo).strip()
        (repo / "gone.rs").unlink()
        _git(["add", "-A", "gone.rs"], cwd=repo)
        _git(["commit", "-qm", "chore: delete it\n\nCo-Authored-By: Claude Opus 5 <n@a.com>"], cwd=repo)
        # `git cat-file -e <rev>:<path>` on the BLOB — not _object_exists, which
        # asks for a commit and would report "missing" even for a live file,
        # making this control pass vacuously.
        tip_has_gone = subprocess.run(
            ["git", "cat-file", "-e", "HEAD:gone.rs"], cwd=repo,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False,
        ).returncode == 0
        expect(not tip_has_gone,
               "CONTROL: gone.rs must be UNREADABLE at the tip or this pins nothing")
        expect(
            subprocess.run(
                ["git", "cat-file", "-e", f"{before_delete}:seed.txt"], cwd=repo,
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False,
            ).returncode == 0,
            "CONTROL: the blob probe must report PRESENT for a file that exists",
        )
        v = cc(f"{before_delete}..HEAD")
        expect(any("gone.rs" in x for x in v),
               "an unreadable-at-tip file was treated as EXEMPT — fail-open hole")
        cleaned = _git(["rev-parse", "HEAD"], cwd=repo).strip()

        # ---- the SHALLOW-GRAFT teeth (the reviewer's blocking finding) ------
        # A shallow clone of the SAME repo. Its boundary commit has parents in
        # the object header but none to the walker — the `actions/checkout`
        # shape in miniature. Without the guard, `git show` on that commit
        # reports the whole tree and the gate drowns in baseline noise.
        with tempfile.TemporaryDirectory() as sd:
            shallow = Path(sd) / "shallow"
            _git(["clone", "-q", "--depth=1", "--no-local",
                  repo.as_uri(), str(shallow)])
            boundary = _git(["rev-parse", "HEAD"], cwd=shallow).strip()
            expect(is_graft_boundary(shallow, boundary),
                   "graft boundary not recognised — the whole-tree scan would return")
            expect(not is_graft_boundary(repo, base),
                   "a true ROOT commit was misread as a graft (its tree IS all additions)")
            expect(not is_graft_boundary(repo, honest),
                   "an ordinary commit was misread as a graft")
            v, u = scan_commits(shallow, [boundary], boundary)
            expect(v == [], f"a grafted commit produced FINDINGS instead of a hole: {len(v)}")
            expect(len(u) == 1 and "SHALLOW-GRAFT" in u[0],
                   f"a grafted commit was not reported as unscannable: {u}")

    if failures:
        print("SELF-TEST FAILED:")
        for f in failures:
            print(f"  - {f}")
        return 1
    print("check-model-attribution.py self-test: OK")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--self-test", action="store_true", help="run hermetic both-direction teeth")
    ap.add_argument("--check-briefs", action="store_true", help="lint .claude/agents/*.md for hard-coded attribution")
    ap.add_argument("--print-fingerprints", action="store_true", help="print allowlist fingerprints for current brief violations")
    ap.add_argument("--check-commits", metavar="RANGE", help="added markers must match each commit's own trailer")
    ap.add_argument(
        "--check-commits-pr",
        action="store_true",
        help=(
            "the CI form of --check-commits: derive the commit set from the "
            "github.event.pull_request payload (head.sha / base.sha / commits) "
            "instead of a rev range against HEAD, which under actions/checkout "
            "is a shallow-grafted merge ref whose diff is the whole tree"
        ),
    )
    ap.add_argument("--blame", nargs="*", metavar="PATH", help="report claimed-vs-git-derived model per marker")
    ap.add_argument("--model", help="with --blame: only markers claiming this model (e.g. SONNET-4.6)")
    ap.add_argument("--only-mismatched", action="store_true", help="with --blame: hide MATCH rows")
    ap.add_argument("--repo", default=".", help="repository root (default: cwd)")
    ap.add_argument(
        "--advisory",
        action="store_true",
        help=(
            "report findings but exit 0. Used ONLY for --check-commits while the "
            "pre-existing marker stock is being cleaned: measured 2026-07-26, 37 of "
            "119 open PRs add a false [SONNET-4.6], so hard-gating today would wedge "
            "the merge train on work whose only fault is copying the surrounding "
            "file. This is an explicit, greppable mode rather than a `|| true` in "
            "the workflow. check-workflow-seam.py enforces that distinction — it "
            "rejects a `||` fallback on this gate's command including one written "
            "across a backslash continuation, the spelling that defeated the "
            "earlier line-by-line seam check."
        ),
    )
    args = ap.parse_args()

    if args.self_test:
        return self_test()

    root = Path(args.repo).resolve()
    rc = 0

    if args.print_fingerprints:
        for fp, where, msg in brief_violations(root):
            print(f"{fp}  {where}  {msg.split('.')[0]}")
        return 0

    if args.check_briefs:
        violations = check_briefs(root)
        if violations:
            print("Hard-coded model attribution in agent briefs (AGENTS.md rule 11):")
            for v in violations:
                print(f"  {v}")
            rc = 1
        else:
            print("check-model-attribution: agent briefs are model-aware — OK")

    if args.check_commits or args.check_commits_pr:
        where = args.check_commits or "this PR's commits"
        skipped = ""
        try:
            if args.check_commits_pr:
                violations, problems, where = check_commits_pr(root)
                if where.startswith("not a pull_request event"):
                    skipped = where
            else:
                violations, problems = check_commits(root, args.check_commits)
        except Exception as exc:  # noqa: BLE001 — see below
            # An ADVISORY check must never fail the build, including on its own
            # internal error: the whole point of the advisory phase is that it
            # reports without blocking. A GATING run still propagates, so this
            # is not a fail-open hole in the enforcing path.
            if args.advisory:
                print(f"ADVISORY: model-attribution scan could not complete ({exc!r}) — not blocking.")
                return rc
            raise
        if skipped:
            print(f"check-model-attribution: {skipped}.")
        elif violations or problems:
            if violations:
                print(f"Mis-attributed model markers added in {where}:")
                for v in violations:
                    print(f"  {v}")
                print(
                    "\nEach marker you ADD must name the model that actually authored the line —\n"
                    "the same model as the commit's Co-Authored-By trailer. If you are restoring\n"
                    "historical content verbatim, add 'model-attribution-allow: <why>' to the line."
                )
            if problems:
                # NOT a pass, and deliberately not spelled like one. A commit we
                # could not diff is a HOLE in the backstop; reporting "OK" over a
                # hole is the inert-guard failure this whole mode exists to avoid.
                print(
                    f"check-model-attribution: INCOMPLETE — {len(problems)} commit(s) in "
                    f"{where} could not be scanned, so this run PROVES NOTHING about them:"
                )
                for p in problems:
                    print(f"  {p}")
            if args.advisory:
                print(
                    "\nADVISORY (#2531): reported, not blocking. This becomes a HARD gate once "
                    "the pre-existing marker stock is cleaned and the sparq-rust-impl brief is "
                    "fixed — both tracked on #2531."
                )
            else:
                rc = 1
        else:
            print(f"check-model-attribution: markers added in {where} match their commits — OK")

    if args.blame is not None:
        rows, tally = blame_markers(root, args.blame, args.model, args.only_mismatched)
        for r in rows:
            print(r)
        total = sum(tally.values())
        print(f"\n{total} marker(s): " + ", ".join(f"{k}={v}" for k, v in sorted(tally.items())))

    if not (args.check_briefs or args.check_commits or args.check_commits_pr or args.blame is not None):
        ap.print_help()
        return 2
    return rc


if __name__ == "__main__":
    sys.exit(main())
