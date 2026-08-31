#!/usr/bin/env python3
"""Reject any workflow step that writes an issue/PR label under a non-repo token.

# [OPUS-5] sparq#5133 (follow-up of sparq#4911) — the ONE half of #4911 that sparq owns.

WHAT #4911 IS. A census of live human-owned holds found that PR labels which the
autonomy loop treats as "a human decided this" were in fact written by machines —
including a population written under the maintainer's own account alias, which makes a
machine park indistinguishable from a human one at the label's name. #4911 proposed two
fixes and **neither is implementable here**: the alias writes originate in
`jeswr/agent-account-registry`, and the re-admission logic (`park_policy.py`,
`capacity_park_admission`) is registry code that does not exist in this checkout. #5133
exists so that is not mistaken for the root cause being fixed. It is not fixed.

WHAT THIS SCRIPT IS, THEN. #4911's first bullet rests on a sparq-side AUDIT: "every
sparq workflow that writes a PR label uses either the orchestrator GitHub App token or
the plain `github.token`; no maintainer-alias PAT appears in any label-writing step."
That audit was true when it was taken and is true now — this script reproduces it — but
it is a POINT-IN-TIME claim over 78 workflow files, and nothing stopped it regressing.
One future step of the shape

    - env:
        GH_TOKEN: ${{ secrets.SOME_MAINTAINER_PAT }}
      run: gh pr edit "$N" --add-label needs:user

would make sparq a SECOND producer of exactly the confusion #4911 measured, silently.
This gate makes that shape impossible to merge. It does NOT fix #4911 — sparq cannot —
it holds the one border sparq controls while the registry-side halves stay open.

THE INVARIANT. For every step that can write a label, every token-ish credential the
step carries must be provably one of:

  * `github.token` / `secrets.GITHUB_TOKEN` — the repo's own scoped token; and
  * `steps.<id>.outputs.token` where `<id>` is a step IN THE SAME JOB that uses
    `actions/create-github-app-token` — i.e. a real App token, identity `*[bot]`.

Anything else fails, including an unresolvable expression: a token that is PRESENT but
not PROVABLY approved is a violation, because the whole point is that the writer's
identity must be legible from the workflow. There is deliberately NO waiver comment —
a gate whose defeat is a one-line annotation is the footgun it is trying to prevent.
Extending the approved set must be a deliberate edit to this file, reviewed as such.

Steps carrying NO credential at all are skipped: an unauthenticated step cannot write a
label, so it is not a PAT risk (this is what keeps `--self-test` invocations of the
label-writing scripts from reding the gate).

WHAT COUNTS AS A LABEL WRITE. Two ways, so the check cannot be sidestepped by moving the
mutation one hop out of the YAML:

  1. INLINE — the step's `run:` body (or an `actions/github-script` `script:`) contains a
     label mutation (`--add-label`, `addLabelsToLabelable`, a mutating `gh api` call on a
     `/labels` path, …).
  2. TRANSITIVE (one hop) — the step invokes `scripts/<f>` where that file itself
     contains a label mutation. The label-writing script set is DERIVED by scanning the
     tree, not hard-coded, so a new label-writing script is covered the day it lands.

SCOPE LIMIT, stated because it is a real hole and not a claimed one: this checks the
mutation of an EXISTING issue's/PR's labels. Creating a NEW issue with `labels[]=` (what
`review_lane_alarm.py` does for its own alarm issue) is not matched — that can label only
the item it just created, so it cannot forge a hold on an existing PR, which is the
confusion #4911 is about.

Stdlib only — like its neighbours in this job it needs no setup step, so it runs
identically in a bare local checkout and in CI. The workflow parse is therefore a small
indentation-aware reader of the subset GitHub Actions actually uses.

    python3 scripts/check-label-write-tokens.py             # enforce over .github/workflows
    python3 scripts/check-label-write-tokens.py --self-test  # hermetic fixtures
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SELF = Path(__file__).resolve()

# --------------------------------------------------------------------------------------
# Label-mutation detection
# --------------------------------------------------------------------------------------

# Unambiguous WRITE spellings. Reads (`gh api repos/x/labels?per_page=100`, `gh pr view
# --json labels`) are deliberately absent: a read under a PAT is not what #4911 measured.
_LABEL_WRITE_PATTERNS = (
    r"--add-label\b",
    r"--remove-label\b",
    r"\baddLabelsToLabelable\b",
    r"\bremoveLabelsFromLabelable\b",
    r"\bclearLabelsFromLabelable\b",
    r"\.(?:addLabels|removeLabel|setLabels|removeAllLabels)\s*\(",
)
LABEL_WRITE_RE = re.compile("|".join(_LABEL_WRITE_PATTERNS))

# `gh api`-style REST mutation of a labels collection. The verb and the path can appear
# in either order on the logical line, so match them independently rather than in
# sequence. `--method GET .../labels` is a read and must NOT match.
_LABELS_PATH_RE = re.compile(r"/labels\b")
_MUTATING_VERB_RE = re.compile(
    r"(?:--method[=\s]+|-X[=\s]+)[\"']?(POST|PUT|PATCH|DELETE)\b", re.IGNORECASE
)


def _strip_shell_comment(line: str) -> str:
    """Drop a shell comment. A commented-out mutation is not an invocation.

    `check-advisory-registry.py` learned this the hard way: a whole-block scan also
    matched script paths named in COMMENTS and attributed a gate to the wrong job.
    """
    stripped = line.lstrip()
    if stripped.startswith("#"):
        return ""
    return re.split(r"\s#", line, maxsplit=1)[0].rstrip()


def _logical_lines(text: str) -> list[str]:
    """Executable lines, with shell backslash-continuations joined and comments dropped.

    A `gh api --method POST … /labels` call is routinely wrapped across lines; scanning
    raw lines would see the verb and the path separately and miss the mutation.
    """
    joined = re.sub(r"\\\n", " ", text)
    return [_strip_shell_comment(ln) for ln in joined.splitlines()]


def is_label_write(text: str) -> bool:
    """Does this shell / JS body mutate labels?"""
    for line in _logical_lines(text):
        if LABEL_WRITE_RE.search(line):
            return True
        if _LABELS_PATH_RE.search(line) and _MUTATING_VERB_RE.search(line):
            return True
    return False


def label_writing_scripts(root: Path) -> set[str]:
    """Repo-relative paths of tracked scripts that themselves mutate labels.

    Derived, never hard-coded — a hard-coded list silently stops covering the tree. This
    file is excluded: it quotes every marker it searches for.
    """
    found: set[str] = set()
    for path in sorted((root / "scripts").rglob("*")):
        if not path.is_file() or path.suffix not in {".py", ".sh", ".js", ".mjs", ".ts"}:
            continue
        if path.resolve() == SELF:
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        if is_label_write(text):
            found.add(path.relative_to(root).as_posix())
    return found


# --------------------------------------------------------------------------------------
# Token classification
# --------------------------------------------------------------------------------------

# Env names that carry a credential. A label write does not have to go through `gh`
# (a `curl -H "Authorization: bearer $X"` works too), so every credential-shaped env on
# a label-writing step is classified, not just GH_TOKEN.
TOKENISH_ENV_RE = re.compile(r"(TOKEN|^PAT$|_PAT$|^GH_PAT|CREDENTIAL|SECRET)", re.IGNORECASE)

# Credential-shaped names that are NOT a usable API credential on their own: the App
# private key + id are inputs to `create-github-app-token`, which is the approved path.
APP_INPUT_ENV = re.compile(r"(APP_ID|PRIVATE_KEY|APP_KEY|CLIENT_ID)$", re.IGNORECASE)

EXPR_RE = re.compile(r"\$\{\{(.+?)\}\}", re.DOTALL)
STEP_OUTPUT_TOKEN_RE = re.compile(r"^steps\.([A-Za-z0-9_-]+)\.outputs\.[A-Za-z0-9_-]+$")
APP_TOKEN_ACTION = "actions/create-github-app-token"


def _atoms(expression: str) -> list[str]:
    """Split a `${{ … }}` body into the operands a fallback chain is built from.

    `steps.app-token.outputs.token || github.token` -> both operands, each of which must
    independently be approved (either branch can be the one that actually runs).
    """
    parts = re.split(r"\|\||&&", expression)
    return [p.strip() for p in parts if p.strip()]


def classify_token(value: str, app_token_step_ids: set[str]) -> str | None:
    """Return None when `value` is provably an approved token, else why it is not."""
    value = value.strip()
    if not value:
        return None  # an empty/unset credential cannot write anything

    exprs = EXPR_RE.findall(value)
    if not exprs:
        # A literal. A hard-coded credential is never acceptable; anything else here is
        # not a credential expression at all (e.g. a `true`/`false` flag).
        if re.search(r"gh[pousr]_[A-Za-z0-9]{16,}|github_pat_", value):
            return "a hard-coded token literal"
        return None

    for expr in exprs:
        for atom in _atoms(expr):
            if atom in ("github.token", "secrets.GITHUB_TOKEN"):
                continue
            if atom.startswith("'") or atom.startswith('"') or atom in ("''", '""'):
                continue  # an empty-string fallback
            match = STEP_OUTPUT_TOKEN_RE.match(atom)
            if match:
                if match.group(1) in app_token_step_ids:
                    continue
                return (
                    f"`{atom}` — step id `{match.group(1)}` is not an "
                    f"`{APP_TOKEN_ACTION}` step in this job"
                )
            if atom.startswith("secrets."):
                return f"`{atom}` — a secret other than GITHUB_TOKEN (a PAT writes as its owner)"
            return f"`{atom}` — not provably `github.token` or an App token"
    return None


# --------------------------------------------------------------------------------------
# Minimal workflow parsing (stdlib only)
# --------------------------------------------------------------------------------------


def _indent(line: str) -> int:
    return len(line) - len(line.lstrip())


def _strip_comment(value: str) -> str:
    """Drop a trailing YAML comment from a scalar, respecting `${{ … }}` and quotes."""
    out, depth, quote = [], 0, ""
    i = 0
    while i < len(value):
        ch = value[i]
        if quote:
            if ch == quote:
                quote = ""
        elif ch in "'\"":
            quote = ch
        elif value.startswith("${{", i):
            depth += 1
        elif value.startswith("}}", i) and depth:
            depth -= 1
        elif ch == "#" and not depth and (not out or out[-1].isspace()):
            break
        out.append(ch)
        i += 1
    return "".join(out).strip()


def _scalar(raw: str) -> str:
    value = _strip_comment(raw)
    if len(value) >= 2 and value[0] == value[-1] and value[0] in "'\"":
        value = value[1:-1]
    return value


def _sub_block(lines: list[str], start: int, key_indent: int) -> tuple[list[str], int]:
    """Lines after `start` that are strictly more indented than `key_indent`."""
    body, i = [], start + 1
    while i < len(lines):
        line = lines[i]
        if line.strip() and _indent(line) <= key_indent:
            break
        body.append(line)
        i += 1
    return body, i


def _mapping(lines: list[str]) -> dict[str, str]:
    """Flat `KEY: value` pairs at the shallowest indent of `lines`.

    Block scalars (`KEY: |`) are folded to their body so a `run:` survives the same
    reader; nested mappings are not needed by this gate and are skipped.
    """
    result: dict[str, str] = {}
    candidates = [ln for ln in lines if ln.strip() and not ln.lstrip().startswith("#")]
    if not candidates:
        return result
    base = min(_indent(ln) for ln in candidates)
    i = 0
    key_re = re.compile(r"^([A-Za-z_][A-Za-z0-9_.-]*):\s*(.*)$")
    while i < len(lines):
        line = lines[i]
        if not line.strip() or _indent(line) != base:
            i += 1
            continue
        match = key_re.match(line.strip())
        if not match:
            i += 1
            continue
        key, inline = match.group(1), match.group(2)
        if inline.strip() in ("|", ">", "|-", ">-", "|+", ">+", ""):
            body, i = _sub_block(lines, i, base)
            if inline.strip() == "":
                # Nested mapping/sequence — only its raw text is ever needed.
                result[key] = "\n".join(body)
            else:
                result[key] = "\n".join(ln[base:] if len(ln) > base else ln for ln in body)
            continue
        result[key] = _scalar(inline)
        i += 1
    return result


def _split_steps(job_lines: list[str]) -> list[dict[str, str]]:
    """Every step of a job, as a flat key->value mapping."""
    start = None
    for i, line in enumerate(job_lines):
        if line.strip() in ("steps:",):
            start = i
            break
    if start is None:
        return []
    body, _ = _sub_block(job_lines, start, _indent(job_lines[start]))
    item_indent = next(
        (_indent(ln) for ln in body if ln.lstrip().startswith("- ")), None
    )
    if item_indent is None:
        return []

    starts = [
        i
        for i, ln in enumerate(body)
        if _indent(ln) == item_indent and ln.lstrip().startswith("- ")
    ]
    steps = []
    for n, s in enumerate(starts):
        end = starts[n + 1] if n + 1 < len(starts) else len(body)
        chunk = list(body[s:end])
        # Re-indent the leading `- key: v` to a plain `key: v` so `_mapping` sees a
        # uniform block.
        chunk[0] = " " * (item_indent + 2) + chunk[0].lstrip()[2:]
        steps.append(_mapping(chunk))
    return steps


def parse_workflow(text: str) -> tuple[dict[str, str], list[dict]]:
    """(workflow-level env, jobs). Each job: {id, env, steps}."""
    lines = text.splitlines()

    workflow_env: dict[str, str] = {}
    jobs_at = None
    for i, line in enumerate(lines):
        if _indent(line) != 0:
            continue
        if line.startswith("env:"):
            body, _ = _sub_block(lines, i, 0)
            workflow_env = _mapping(body)
        elif line.startswith("jobs:"):
            jobs_at = i

    jobs: list[dict] = []
    if jobs_at is None:
        return workflow_env, jobs

    body, _ = _sub_block(lines, jobs_at, 0)
    key_re = re.compile(r"^([A-Za-z_][A-Za-z0-9_-]*):\s*(?:#.*)?$")
    ids = [
        (i, key_re.match(ln.strip()).group(1))
        for i, ln in enumerate(body)
        if ln.strip() and _indent(ln) == min(
            (_indent(x) for x in body if x.strip()), default=0
        ) and key_re.match(ln.strip())
    ]
    for n, (i, job_id) in enumerate(ids):
        end = ids[n + 1][0] if n + 1 < len(ids) else len(body)
        job_lines = body[i + 1:end]
        job_env: dict[str, str] = {}
        for k, ln in enumerate(job_lines):
            if ln.strip() == "env:" and _indent(ln) == min(
                (_indent(x) for x in job_lines if x.strip()), default=0
            ):
                sub, _ = _sub_block(job_lines, k, _indent(ln))
                job_env = _mapping(sub)
                break
        jobs.append({"id": job_id, "env": job_env, "steps": _split_steps(job_lines)})
    return workflow_env, jobs


# --------------------------------------------------------------------------------------
# The check
# --------------------------------------------------------------------------------------


def _inline_body(step: dict[str, str]) -> str:
    """Text that can itself CONTAIN a label mutation — `run:` plus a `script:` input."""
    parts = [step.get("run", "")]
    with_map = _mapping(step["with"].splitlines()) if step.get("with") else {}
    if with_map.get("script"):
        parts.append(with_map["script"])
    return "\n".join(p for p in parts if p)


def _invokes_label_writer(step: dict[str, str], writers: set[str]) -> str | None:
    """The label-writing script this step EXECUTES, if any.

    Scoped to `run:` on purpose. A step's `with:` block is not executed — an
    `actions/checkout` with `sparse-checkout: scripts` whose comment happens to name
    `scripts/tests/test_arm_capability_wiring.py` fetches that file, it does not run it,
    and treating it as a label write would put a false gate on every checkout.
    """
    for line in _logical_lines(step.get("run", "")):
        for script in sorted(writers):
            if script in line:
                return script
    return None


def check_workflow(text: str, path: str, writers: set[str]) -> list[str]:
    workflow_env, jobs = parse_workflow(text)
    offences: list[str] = []

    for job in jobs:
        app_token_step_ids = {
            step["id"]
            for step in job["steps"]
            if step.get("id") and APP_TOKEN_ACTION in step.get("uses", "")
        }
        for step in job["steps"]:
            via = None
            if is_label_write(_inline_body(step)):
                via = "an inline label mutation"
            else:
                script = _invokes_label_writer(step, writers)
                if script:
                    via = f"`{script}`, which mutates labels"
            if via is None:
                continue

            step_env = _mapping(step["env"].splitlines()) if step.get("env") else {}
            effective = {**workflow_env, **job["env"], **step_env}

            # `actions/github-script` takes its credential as an input, not an env.
            with_map = _mapping(step["with"].splitlines()) if step.get("with") else {}
            if "github-token" in with_map:
                effective["github-token (with:)"] = with_map["github-token"]

            label = step.get("name") or step.get("uses") or "(unnamed step)"
            for name, value in sorted(effective.items()):
                if not TOKENISH_ENV_RE.search(name) or APP_INPUT_ENV.search(name):
                    continue
                reason = classify_token(value, app_token_step_ids)
                if reason:
                    offences.append(
                        f"{path}: job `{job['id']}` step \"{label}\" writes labels via "
                        f"{via}, but `{name}` is {reason}. A label written by a personal "
                        f"access token is indistinguishable from a human's own action "
                        f"(sparq#4911); use `github.token` or an "
                        f"`{APP_TOKEN_ACTION}` token."
                    )
    return offences


def check_tree(root: Path) -> list[str]:
    writers = label_writing_scripts(root)
    offences: list[str] = []
    for path in sorted((root / ".github" / "workflows").glob("*.yml")):
        text = path.read_text(encoding="utf-8", errors="replace")
        offences += check_workflow(text, path.relative_to(root).as_posix(), writers)
    return offences


# --------------------------------------------------------------------------------------
# Self-test
# --------------------------------------------------------------------------------------

_APPROVED_INLINE = """\
name: t
jobs:
  a:
    steps:
      - name: label it
        env:
          GH_TOKEN: ${{ github.token }}
        run: gh pr edit "$N" --add-label review:pass
"""

_APPROVED_APP = """\
name: t
jobs:
  a:
    steps:
      - name: mint
        id: app-token
        uses: actions/create-github-app-token@bcd2ba4 # v3.2.0
      - name: label it
        env:
          GH_TOKEN: ${{ steps.app-token.outputs.token || github.token }}
        run: gh pr edit "$N" --add-label review:pass
"""

_PAT_INLINE = """\
name: t
jobs:
  a:
    steps:
      - name: label it
        env:
          GH_TOKEN: ${{ secrets.MAINTAINER_PAT }}
        run: gh pr edit "$N" --add-label needs:user
"""

_PAT_JOB_ENV = """\
name: t
jobs:
  a:
    env:
      GH_TOKEN: ${{ secrets.MAINTAINER_PAT }}
    steps:
      - name: label it
        run: |
          gh issue edit "$N" --remove-label status:untriaged
"""

_PAT_TRANSITIVE = """\
name: t
jobs:
  a:
    steps:
      - name: sweep
        env:
          GH_TOKEN: ${{ secrets.MAINTAINER_PAT }}
        run: python3 scripts/label-writer.py --pr "$N"
"""

_PAT_REST_API = """\
name: t
jobs:
  a:
    steps:
      - name: label it
        env:
          GH_TOKEN: ${{ secrets.MAINTAINER_PAT }}
        run: |
          gh api --method POST \\
            "repos/$REPO/issues/$N/labels" -f 'labels[]=needs:user'
"""

_PAT_NON_LABEL = """\
name: t
jobs:
  a:
    steps:
      - name: ring the registry
        env:
          GH_TOKEN: ${{ github.token }}
          REGISTRY_RING_TOKEN: ${{ secrets.REGISTRY_RING_TOKEN }}
        run: |
          gh api --method GET "repos/$REPO/issues/$N/labels"
          curl -H "Authorization: bearer $REGISTRY_RING_TOKEN" "$DISPATCH_URL"
"""

_FAKE_APP_STEP = """\
name: t
jobs:
  a:
    steps:
      - name: not an app token step
        id: creds
        uses: some/other-action@abc
      - name: label it
        env:
          GH_TOKEN: ${{ steps.creds.outputs.token }}
        run: gh pr edit "$N" --add-label review:pass
"""

_NO_TOKEN = """\
name: t
jobs:
  a:
    steps:
      - name: self-test the sweeper
        run: python3 scripts/label-writer.py --self-test
"""

_GITHUB_SCRIPT_PAT = """\
name: t
jobs:
  a:
    steps:
      - name: label it
        uses: actions/github-script@abc
        with:
          github-token: ${{ secrets.MAINTAINER_PAT }}
          script: |
            await github.rest.issues.addLabels({owner, repo, issue_number: 1, labels: ['x']})
"""

_CASES = (
    ("approved: inline github.token", _APPROVED_INLINE, 0),
    ("approved: App token with github.token fallback", _APPROVED_APP, 0),
    ("PAT on an inline --add-label", _PAT_INLINE, 1),
    ("PAT inherited from job env", _PAT_JOB_ENV, 1),
    ("PAT on a step invoking a label-writing script", _PAT_TRANSITIVE, 1),
    ("PAT on a wrapped `gh api --method POST …/labels`", _PAT_REST_API, 1),
    ("PAT on a step that does NOT write labels", _PAT_NON_LABEL, 0),
    ("`steps.*.outputs.token` from a non-App step", _FAKE_APP_STEP, 1),
    ("no credential at all on a label-writing script", _NO_TOKEN, 0),
    ("PAT as an actions/github-script github-token input", _GITHUB_SCRIPT_PAT, 1),
)


def self_test() -> int:
    writers = {"scripts/label-writer.py"}
    failures = 0
    for name, text, expected in _CASES:
        got = check_workflow(text, "fixture.yml", writers)
        ok = len(got) == expected
        print(f"  {'PASS' if ok else 'FAIL'}  {name} (expected {expected}, got {len(got)})")
        if not ok:
            failures += 1
            for line in got:
                print(f"        {line}")

    # Detector unit checks — these pin the two halves the fixtures exercise jointly.
    units = (
        ("read of a labels path is not a write", not is_label_write(
            'gh api --method GET "repos/o/r/issues/1/labels"')),
        ("`--paginate` labels read is not a write", not is_label_write(
            'gh api --paginate "repos/o/r/labels?per_page=100"')),
        ("wrapped POST to a labels path is a write", is_label_write(
            'gh api --method POST \\\n  "repos/o/r/issues/1/labels"')),
        ("github.token is approved", classify_token("${{ github.token }}", set()) is None),
        ("a non-GITHUB_TOKEN secret is rejected",
         classify_token("${{ secrets.X }}", set()) is not None),
        ("an App-token step output is approved",
         classify_token("${{ steps.app.outputs.token }}", {"app"}) is None),
    )
    for name, ok in units:
        print(f"  {'PASS' if ok else 'FAIL'}  {name}")
        if not ok:
            failures += 1

    if failures:
        print(f"\nself-test: {failures} case(s) FAILED")
        return 1
    print("\nself-test: all cases PASS")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--self-test", action="store_true", help="run hermetic fixtures")
    parser.add_argument(
        "--list-writers",
        action="store_true",
        help="print the derived set of label-writing scripts and exit",
    )
    args = parser.parse_args(argv)

    if args.self_test:
        return self_test()
    if args.list_writers:
        for script in sorted(label_writing_scripts(ROOT)):
            print(script)
        return 0

    offences = check_tree(ROOT)
    if offences:
        print("Label writes must be attributable to the repo token or the App:\n")
        for offence in offences:
            print(f"  ✗ {offence}")
        print(
            "\nsparq#4911: a machine-written hold under a human's PAT is "
            "indistinguishable from a human hold at the label's name."
        )
        return 1
    print("label-write token attribution: OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
