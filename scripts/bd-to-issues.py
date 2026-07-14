#!/usr/bin/env python3
# [OPUS-4.8] Issue-native orchestration: migrate bd beads -> GitHub issues (Phase 1).
"""bd-to-issues.py — one-time, idempotent migration of open bd beads into GitHub issues.

DEFAULT is --dry-run: it parses `bd export`, computes the issue payloads + label mapping + the
dependency edges, and prints a summary WITHOUT creating anything. `--apply` does the real two-pass
(create issues, then link blocked-by dependencies) and writes the `sq-… ↔ #NN` map — held for the
maintainer's go-ahead because it bulk-creates hundreds of issues.

Label mapping (bd -> issue):
  priority 0..4            -> priority:P{n}
  existing labels          -> passed through verbatim (area:<crate>=package, needs:*, kind:*, from:*)
  issue_type / kind:       -> role:<r> (feature/bug->impl, docs->docs, spike->research, chore->ci, ...)
  dependency edges         -> `Blocked-by: #NN` body markers (resolved to native deps in --apply)
The bead id (`sq-…`) is preserved in the issue title so existing PR-title tokens still resolve.
"""
import argparse
import json
import subprocess
import sys

ROLE_BY_TYPE = {"feature": "impl", "bug": "impl", "task": "impl", "chore": "ci",
                "spike": "research", "epic": "impl"}
ROLE_BY_KIND = {"docs": "docs", "design": "research", "research": "research", "perf": "perf",
                "test": "impl", "ci": "ci", "site": "site"}


def parse_export(lines):
    """Return (issues_by_id, edges) from `bd export` JSONL. Robust to edge records / dependencies
    arrays being expressed either as separate `_type` lines or as a `dependencies` field."""
    issues, edges = {}, []
    for line in lines:
        line = line.strip()
        if not line:
            continue
        d = json.loads(line)
        t = d.get("_type", "issue")
        if t and t != "issue":
            # a SEPARATE dependency record: {issue_id, depends_on_id, type} (or from/to variants)
            frm = d.get("issue_id") or d.get("from") or d.get("dependent") or d.get("blocked")
            to = d.get("depends_on_id") or d.get("to") or d.get("depends_on") or d.get("blocker")
            if frm and to:
                edges.append((frm, to, d.get("type", "blocks")))
            continue
        if "id" not in d:
            continue
        issues[d["id"]] = d
        for dep in d.get("dependencies") or []:
            if not isinstance(dep, dict):
                continue
            to = dep.get("depends_on_id") or dep.get("depends_on") or dep.get("to")
            etype = dep.get("type", "blocks")
            if to:
                edges.append((dep.get("issue_id", d["id"]), to, etype))
    # dedupe (an edge can appear both embedded in an issue and as a separate record)
    edges = sorted({(a, b, c) for (a, b, c) in edges})
    return issues, edges


def role_for(issue):
    labels = [lb["name"] if isinstance(lb, dict) else lb for lb in issue.get("labels", [])]
    for lb in labels:
        if lb.startswith("kind:") and lb[5:] in ROLE_BY_KIND:
            return ROLE_BY_KIND[lb[5:]]
    return ROLE_BY_TYPE.get(issue.get("issue_type", "task"), "impl")


def issue_labels(bead):
    labels = [lb["name"] if isinstance(lb, dict) else lb for lb in bead.get("labels", [])]
    out = set(labels)
    p = bead.get("priority")
    if isinstance(p, int) and 0 <= p <= 4:
        out.add(f"priority:P{p}")
    out.add(f"role:{role_for(bead)}")
    return sorted(out)


def plan(issues, edges, include_closed=False):
    """The migration plan: which beads become issues, their labels, blocker edges, and parent links.
    `blocks` edges become readiness blockers; `parent-child` edges become parent (sub-issue) links —
    NOT readiness blockers (this fixes bd's sub-epic-leaf-invisibility, where a subtask showed as
    blocked merely because its epic was open)."""
    open_ids = {i: b for i, b in issues.items()
                if include_closed or str(b.get("status", "open")).lower() in ("open", "in_progress")}
    blockers, parents = {}, {}
    for edge in edges:
        frm, to = edge[0], edge[1]
        etype = edge[2] if len(edge) > 2 else "blocks"
        if frm not in open_ids or to not in open_ids:
            continue
        if etype == "parent-child":
            parents.setdefault(frm, []).append(to)
        elif etype == "blocks":
            blockers.setdefault(frm, []).append(to)
        # other edge types (related/discovered/duplicate/…) are NOT readiness blockers — skipped
    return open_ids, blockers, parents


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default="sparq-org/sparq")
    ap.add_argument("--apply", action="store_true", help="actually create issues (bulk!) — held for go-ahead")
    ap.add_argument("--export-file", help="read bd export from a file instead of running bd")
    args = ap.parse_args()

    if args.export_file:
        lines = open(args.export_file, encoding="utf-8").read().splitlines()
    else:
        lines = subprocess.run(["bd", "export"], capture_output=True, text=True, check=True).stdout.splitlines()
    issues, edges = parse_export(lines)
    open_ids, blockers, parents = plan(issues, edges)

    # --- summary (always) ---
    from collections import Counter
    prio = Counter(f"P{b.get('priority')}" for b in open_ids.values())
    roles = Counter(role_for(b) for b in open_ids.values())
    pkgs = Counter(lb[5:] for b in open_ids.values()
                   for lb in issue_labels(b) if lb.startswith("area:"))
    print(f"beads (all): {len(issues)}  |  to migrate (open/in_progress): {len(open_ids)}")
    print(f"blocker (blocks) edges among migrated: {sum(len(v) for v in blockers.values())}")
    print(f"parent-child links among migrated:    {sum(len(v) for v in parents.values())}")
    print(f"priority: {dict(sorted(prio.items()))}")
    print(f"roles:    {dict(roles.most_common())}")
    print(f"top packages: {dict(pkgs.most_common(8))}")
    if not open_ids:
        print("no open beads to migrate.")
        return 0
    sample = next(iter(open_ids.values()))
    print("sample issue payload:")
    print(f"  title : {sample['id']}: {sample['title'][:70]}")
    print(f"  labels: {issue_labels(sample)}")
    print(f"  blockers: {[f'{b}' for b in blockers.get(sample['id'], [])] or 'none'}")

    if not args.apply:
        print("\n[dry-run] nothing created. Re-run with --apply (after go-ahead) to bulk-create.")
        return 0

    print("\n[apply] would create issues + link blockers here (guarded — implement on go-ahead).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
