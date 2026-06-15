#!/usr/bin/env python3
# [OPUS-4.8] Merge-triggered flow-on engine (bead sq-ncvq.2, epic sq-ncvq).
# Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns).
#
# Reads a MERGED PR's changed-file list + labels + title, evaluates the
# declarative rules in scripts/flow-on-rules.toml, and opens ONE GitHub issue
# (labelled `flow-on` + `auto`) per triggered follow-on template.
#
# WHY GITHUB ISSUES, NOT `bd create`:
# (research/maintenance-flow-on-automation-design.md §2.2) In CI there is NO `bd`
# dolt DB — it lives only in the orchestrator's main checkout and is gitignored,
# and hand-editing `.beads/` is forbidden. So this script must NOT touch `bd` or
# `.beads/`. Instead it emits each follow-on as a GitHub issue; the orchestrator
# reconciles those issues into beads (`bd create`) out-of-band.
#
# IDEMPOTENCY: each `create` template has a `dedup_key`. After expansion the key
# is embedded in the issue body as `<!-- flow-on-key: <key> -->`. Before creating,
# this script searches OPEN issues for that marker and skips if one exists. So
# re-running the workflow (or re-merging) never duplicates a follow-on.
#
# Usage:
#   flow-on.py --pr 123                      # read changed files/labels/title via gh, create issues
#   flow-on.py --pr 123 --dry-run            # print what WOULD be created; no gh writes
#   flow-on.py --pr 123 --changed-files f.txt --labels-file l.txt --title "..."   # hermetic inputs (tests)
#   flow-on.py --pr 123 --rules path.toml    # override rule file
#
# Hermetic mode: if --changed-files / --labels-file / --title are all provided,
# the script makes NO gh calls for inputs. With --dry-run it makes no gh calls at
# all, so it is fully testable without network access.

from __future__ import annotations

import argparse
import fnmatch
import json
import os
import re
import subprocess
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_RULES = REPO_ROOT / "scripts" / "flow-on-rules.toml"

# Labels every flow-on issue carries (in addition to a rule's own labels).
BASE_LABELS = ["flow-on", "auto"]

# Public-surface paths whose change should trigger the "changed-public-feature"
# docs rule ONLY when no SKILL.md was touched in the same PR. A change anywhere
# under skills/ counts as "the docs were synced", suppressing that rule.
SKILL_PATHS = ("skills/",)


@dataclass
class CreateTemplate:
    dedup_key: str
    title: str
    body: str
    labels: list[str] = field(default_factory=list)


@dataclass
class Rule:
    id: str
    description: str = ""
    when_paths: list[str] = field(default_factory=list)
    when_new_paths: list[str] = field(default_factory=list)
    when_label: str | None = None
    when_title: str | None = None
    creates: list[CreateTemplate] = field(default_factory=list)


@dataclass
class FollowOn:
    """A fully-expanded follow-on issue ready to be created (or printed)."""

    rule_id: str
    dedup_key: str
    title: str
    body: str
    labels: list[str]


# --------------------------------------------------------------------------- #
# Rule loading
# --------------------------------------------------------------------------- #
def load_rules(path: Path) -> list[Rule]:
    with path.open("rb") as fh:
        data = tomllib.load(fh)
    rules: list[Rule] = []
    for raw in data.get("rule", []):
        creates = [
            CreateTemplate(
                dedup_key=c["dedup_key"],
                title=c["title"],
                body=c["body"],
                labels=list(c.get("labels", [])),
            )
            for c in raw.get("create", [])
        ]
        rule = Rule(
            id=raw["id"],
            description=raw.get("description", ""),
            when_paths=list(raw.get("when_paths", [])),
            when_new_paths=list(raw.get("when_new_paths", [])),
            when_label=raw.get("when_label"),
            when_title=raw.get("when_title"),
            creates=creates,
        )
        if not (rule.when_paths or rule.when_new_paths or rule.when_label or rule.when_title):
            raise ValueError(f"rule {rule.id!r} has no trigger predicate")
        if not rule.creates:
            raise ValueError(f"rule {rule.id!r} has no create templates")
        rules.append(rule)
    return rules


# --------------------------------------------------------------------------- #
# Placeholder extraction
# --------------------------------------------------------------------------- #
def _first_segment_after(prefix: str, paths: list[str]) -> str | None:
    """Return the directory name immediately under `prefix` for the first path
    that starts with it (e.g. prefix='crates/', 'crates/sparq-foo/Cargo.toml'
    -> 'sparq-foo')."""
    for p in paths:
        if p.startswith(prefix):
            rest = p[len(prefix):]
            seg = rest.split("/", 1)[0]
            if seg:
                return seg
    return None


def build_context(
    pr: int,
    pr_title: str,
    changed: list[str],
    added: list[str],
) -> dict[str, str]:
    """Compute the placeholder dict for {pr}/{pr_title}/{crate}/{suite}/{surface}.

    Prefer ADDED paths for crate/suite (the rules that use them are new-* rules),
    falling back to all changed paths so changed-surface rules still resolve."""
    pool_for_dir = added + changed  # added first so "new crate" wins
    crate = _first_segment_after("crates/", pool_for_dir)
    suite = _first_segment_after("bench/", pool_for_dir) or _first_segment_after("zk/", pool_for_dir)
    surface = _first_segment_after("skills/", pool_for_dir)
    return {
        "pr": str(pr),
        "pr_title": pr_title,
        "crate": crate or "",
        "suite": suite or "",
        "surface": surface or "",
    }


def expand(text: str, ctx: dict[str, str]) -> str:
    # Only expand known placeholders; leave unknown braces untouched.
    def repl(m: re.Match[str]) -> str:
        key = m.group(1)
        return ctx.get(key, m.group(0))

    return re.sub(r"\{([a-z_]+)\}", repl, text)


# --------------------------------------------------------------------------- #
# Rule evaluation
# --------------------------------------------------------------------------- #
def _any_glob_match(globs: list[str], paths: list[str]) -> bool:
    for g in globs:
        for p in paths:
            if fnmatch.fnmatch(p, g):
                return True
    return False


def rule_matches(rule: Rule, changed: list[str], added: list[str], labels: list[str], title: str) -> bool:
    # ALL present predicates must pass; absent predicates are ignored.
    if rule.when_paths and not _any_glob_match(rule.when_paths, changed):
        return False
    if rule.when_new_paths and not _any_glob_match(rule.when_new_paths, added):
        return False
    if rule.when_label is not None and rule.when_label not in labels:
        return False
    if rule.when_title is not None and not re.search(rule.when_title, title, re.IGNORECASE):
        return False

    # Special case for the changed-public-feature docs rule: suppress it when the
    # PR already touched a SKILL.md (the docs were synced in the same diff).
    if rule.id == "changed-public-feature-docs":
        if any(p.startswith(SKILL_PATHS) for p in changed):
            return False
    return True


def key_marker(dedup_key: str) -> str:
    return f"<!-- flow-on-key: {dedup_key} -->"


def evaluate(
    rules: list[Rule],
    pr: int,
    pr_title: str,
    changed: list[str],
    added: list[str],
    labels: list[str],
) -> list[FollowOn]:
    ctx = build_context(pr, pr_title, changed, added)
    out: list[FollowOn] = []
    seen_keys: set[str] = set()
    for rule in rules:
        if not rule_matches(rule, changed, added, labels, pr_title):
            continue
        for tmpl in rule.creates:
            dedup_key = expand(tmpl.dedup_key, ctx)
            if dedup_key in seen_keys:
                continue  # de-dup within a single run
            seen_keys.add(dedup_key)
            body = expand(tmpl.body, ctx).rstrip() + "\n\n" + key_marker(dedup_key)
            body += (
                f"\n\n> 🤖 SPARQ agent — auto-generated by the flow-on engine "
                f"(rule `{rule.id}`) from merged PR #{pr}. Reconcile into a bead."
            )
            labels_full = list(dict.fromkeys(BASE_LABELS + tmpl.labels))
            out.append(
                FollowOn(
                    rule_id=rule.id,
                    dedup_key=dedup_key,
                    title=expand(tmpl.title, ctx),
                    body=body,
                    labels=labels_full,
                )
            )
    return out


# --------------------------------------------------------------------------- #
# gh I/O
# --------------------------------------------------------------------------- #
def _gh(args: list[str]) -> str:
    proc = subprocess.run(
        ["gh", *args],
        capture_output=True,
        text=True,
        check=True,
    )
    return proc.stdout


def fetch_pr_inputs(pr: int) -> tuple[str, list[str], list[str], list[str]]:
    """Return (title, changed_files, added_files, labels) for a merged PR via gh."""
    info = json.loads(_gh(["pr", "view", str(pr), "--json", "title,labels,files"]))
    title = info.get("title", "")
    labels = [lbl["name"] for lbl in info.get("labels", [])]
    files = info.get("files", [])
    changed = [f["path"] for f in files]
    # gh's files[].additions/deletions don't reveal status; use the diff to find
    # purely-added files (status "A").
    added = _added_files_via_diff(pr)
    return title, changed, added, labels


def _added_files_via_diff(pr: int) -> list[str]:
    # `gh pr diff --name-only` lists changed files but not status. Use the patch
    # and detect "new file mode" hunks for added-file paths.
    patch = _gh(["pr", "diff", str(pr)])
    added: list[str] = []
    cur_new = False
    for line in patch.splitlines():
        if line.startswith("diff --git "):
            cur_new = False
        elif line.startswith("new file mode"):
            cur_new = True
        elif line.startswith("+++ b/") and cur_new:
            added.append(line[len("+++ b/"):])
            cur_new = False
    return added


def open_issue_exists(dedup_key: str) -> bool:
    marker = key_marker(dedup_key)
    # [OPUS-4.8] Search by the stable, punctuation-light substring
    # `flow-on-key: <key>` rather than the full HTML-comment marker — GitHub's
    # search tokeniser handles `<!--`/`-->` unreliably, which could miss an
    # existing issue and break idempotency (create a duplicate). The full marker
    # is still verified against each returned body below, so the dedup decision
    # stays exact; the looser query only widens the candidate set.
    search = f"flow-on-key: {dedup_key}"
    out = _gh(
        [
            "issue",
            "list",
            "--state",
            "open",
            "--label",
            "flow-on",
            "--search",
            search,
            "--json",
            "number,body",
            "--limit",
            "100",
        ]
    )
    for issue in json.loads(out):
        if marker in (issue.get("body") or ""):
            return True
    return False


# Labels we have already ensured exist this run (avoid redundant gh calls).
_ENSURED_LABELS: set[str] = set()


def ensure_label(label: str) -> None:
    """[OPUS-4.8] Idempotently ensure `label` exists before it is applied.

    `gh issue create --label X` ERRORS if label X does not yet exist, so on a
    fresh repo (where none of the per-rule labels — bench/docs/zk/… — nor
    `flow-on`/`auto` exist yet) no follow-on issue would ever be created. We
    create-if-missing with `gh label create --force` (upsert; needs only the
    `issues: write` token the workflow already grants). Failures are swallowed:
    if the label can't be created (e.g. it already exists from a concurrent run)
    the subsequent create still succeeds, and a genuinely un-creatable label
    surfaces as the create's own error rather than masking it here."""
    if label in _ENSURED_LABELS:
        return
    try:
        _gh(["label", "create", label, "--force"])
    except Exception:  # noqa: BLE001 - best-effort upsert; create_issue reports real failures
        pass
    _ENSURED_LABELS.add(label)


def create_issue(fo: FollowOn) -> str:
    args = ["issue", "create", "--title", fo.title, "--body", fo.body]
    for lbl in fo.labels:
        ensure_label(lbl)
        args += ["--label", lbl]
    return _gh(args).strip()


# --------------------------------------------------------------------------- #
# CLI
# --------------------------------------------------------------------------- #
def _read_lines(path: str) -> list[str]:
    return [ln.strip() for ln in Path(path).read_text().splitlines() if ln.strip()]


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="Merge-triggered flow-on follow-on issue emitter.")
    ap.add_argument("--pr", type=int, required=True, help="merged PR number")
    ap.add_argument("--rules", default=str(DEFAULT_RULES), help="rule TOML path")
    ap.add_argument("--dry-run", action="store_true", help="print what WOULD be created; no gh writes")
    # Hermetic inputs (for tests / local runs without gh).
    ap.add_argument("--changed-files", help="file with one changed path per line")
    ap.add_argument("--added-files", help="file with one ADDED path per line (subset of changed)")
    ap.add_argument("--labels-file", help="file with one PR label per line")
    ap.add_argument("--title", help="PR title")
    args = ap.parse_args(argv)

    rules = load_rules(Path(args.rules))

    hermetic = args.changed_files is not None and args.title is not None
    if hermetic:
        changed = _read_lines(args.changed_files)
        # [OPUS-4.8] Default `added` to EMPTY, not `list(changed)`: a changed
        # file is not provably newly-added (mirrors the real `gh` path, which
        # derives `added` from "new file mode" diff hunks). Defaulting to all
        # changed files made `when_new_paths` rules fire on every modified path,
        # producing false follow-ons in hermetic/CI dry-runs. Pass --added-files
        # to exercise new-path rules.
        added = _read_lines(args.added_files) if args.added_files else []
        labels = _read_lines(args.labels_file) if args.labels_file else []
        title = args.title
    else:
        title, changed, added, labels = fetch_pr_inputs(args.pr)

    follow_ons = evaluate(rules, args.pr, title, changed, added, labels)

    if not follow_ons:
        print(f"flow-on: no rules triggered for PR #{args.pr}.")
        return 0

    created = 0
    skipped = 0
    for fo in follow_ons:
        if args.dry_run:
            print(f"[dry-run] WOULD create issue (rule={fo.rule_id}, key={fo.dedup_key}):")
            print(f"          title : {fo.title}")
            print(f"          labels: {', '.join(fo.labels)}")
            continue
        if open_issue_exists(fo.dedup_key):
            print(f"flow-on: skip (open issue exists) key={fo.dedup_key} rule={fo.rule_id}")
            skipped += 1
            continue
        url = create_issue(fo)
        print(f"flow-on: created {url} (rule={fo.rule_id}, key={fo.dedup_key})")
        created += 1

    if not args.dry_run:
        print(f"flow-on: done — {created} created, {skipped} skipped (already open).")
        # Write a GitHub Actions step summary if available.
        summary = os.environ.get("GITHUB_STEP_SUMMARY")
        if summary:
            with open(summary, "a") as fh:
                fh.write(f"### flow-on (PR #{args.pr})\n\n")
                fh.write(f"- created: {created}\n- skipped (already open): {skipped}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
