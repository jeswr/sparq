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
import re
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


# [OPUS-4.8] Only PIPELINE-RELEVANT labels cross into GitHub. bd carries ~200 free-form tags
# (from:*, effort:*, tier:*, roadmap, one-off topic tags) that triage/readiness/dispatch NEVER read.
# Passing them through would (a) force creating ~200 junk repo labels and (b) risk a LABEL-LESS issue:
# `gh issue create` fails on the first unknown label and the fallback drops ALL labels, so triage
# cannot derive priority → the issue is stuck untriaged. Keep only what the pipeline consumes.
_KEEP_PREFIX = ("area:", "needs:", "trust:", "kind:")
_SEC_KEYWORDS = ("zk", "mpc", "reasoner", "crypto", "auth", "e2ee")


def _pipeline_relevant(lb):
    """A label triage/readiness/dispatch actually reads: an area:/needs:/trust:/kind: label, or one
    carrying a security keyword so triage's soundness routing survives even without an area:/kind:."""
    return lb.startswith(_KEEP_PREFIX) or any(k in lb for k in _SEC_KEYWORDS)


def issue_labels(bead):
    labels = [lb["name"] if isinstance(lb, dict) else lb for lb in bead.get("labels", [])]
    out = {lb for lb in labels if _pipeline_relevant(lb)}   # whitelist — drop free-form bd tags
    p = bead.get("priority")
    if isinstance(p, int) and 0 <= p <= 4:
        out.add(f"priority:P{p}")
    out.add(f"role:{role_for(bead)}")
    # [OPUS-4.8] an epic is a tracking/umbrella issue, never a dispatchable work item — mark it
    # `kind:epic` so the readiness engine excludes it (else a worker would try to "implement" the
    # epic itself, producing a garbage PR). 57 of the 893 open beads are epics, 41 at P1 — they would
    # otherwise be selected FIRST by priority. Enforcement is in ready-issues.py; this is the signal.
    if str(bead.get("issue_type", "")).lower() == "epic":
        out.add("kind:epic")
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


# --- idempotent two-pass --apply (review C2) -----------------------------------------------------
def _run(args, check=True):
    return subprocess.run(args, capture_output=True, text=True, check=check)


def fetch_bd_map(repo):
    """{bd-id: issue_number} from existing issues carrying a `<!-- bd-id:... -->` marker (resume)."""
    out = _run(["gh", "issue", "list", "-R", repo, "--state", "all", "--limit", "10000",
                "--json", "number,body"]).stdout
    m = {}
    for it in json.loads(out or "[]"):
        mm = re.search(r"<!--\s*bd-id:(\S+?)\s*-->", it.get("body") or "")
        if mm:
            m[mm.group(1)] = it["number"]
    return m


def _body(bead, bid):
    desc = (bead.get("description") or "").strip()
    return f"{desc}\n\n<!-- bd-id:{bid} -->\n"


def _create_issue(repo, bid, bead):
    base = ["gh", "issue", "create", "-R", repo, "--title", f"{bid}: {bead['title']}", "--body", _body(bead, bid)]
    args = list(base)
    for l in issue_labels(bead):
        args += ["--label", l]
    r = _run(args, check=False)
    if r.returncode != 0:            # a label may not exist in a scratch repo → create without labels
        r = _run(base)
    return int(re.search(r"/issues/(\d+)", r.stdout).group(1))


def _ensure_marker(repo, num, marker):
    """Append `marker` to the issue body iff absent (idempotent)."""
    body = json.loads(_run(["gh", "issue", "view", str(num), "-R", repo, "--json", "body"]).stdout)["body"] or ""
    if marker in body:
        return
    _run(["gh", "issue", "edit", str(num), "-R", repo, "--body", body.rstrip() + "\n" + marker + "\n"])


def apply_migration(repo, open_ids, blockers, parents, limit=None, checkpoint="/tmp/bd-migration-map.json"):
    """Resumable: pass 1 upserts issues by bd-id marker (checkpointing the map); pass 2 idempotently
    adds Blocked-by:/Parent: markers. Re-running creates nothing new."""
    id_map = fetch_bd_map(repo)
    ids = list(open_ids)[: limit] if limit else list(open_ids)
    created = 0
    for bid in ids:                                  # pass 1
        if bid in id_map:
            continue
        id_map[bid] = _create_issue(repo, bid, open_ids[bid])
        created += 1
        json.dump(id_map, open(checkpoint, "w"))     # checkpoint immediately (crash-resumable)
    for bid in ids:                                  # pass 2 (idempotent)
        for b in blockers.get(bid, []):
            if b in id_map:
                _ensure_marker(repo, id_map[bid], f"Blocked-by: #{id_map[b]}")
        for p in parents.get(bid, []):
            if p in id_map:
                _ensure_marker(repo, id_map[bid], f"Parent: #{id_map[p]}")
    return id_map, created


def _self_test():
    ok = True

    def chk(n, got, want):
        nonlocal ok
        good = got == want
        ok = ok and good
        print(f"  {'ok  ' if good else 'FAIL'} {n}: {got} (want {want})")

    # whitelist: keep area:/kind:/needs:/trust: + security-keyword labels; DROP free-form bd tags
    bead = {"priority": 2, "issue_type": "feature", "labels": [
        "area:sparq-core", "kind:test", "needs:user", "from:agent", "effort:M",
        "tier:fable", "roadmap", "federation", "noir"]}
    got = set(issue_labels(bead))
    chk("whitelist keeps pipeline labels", {"area:sparq-core", "kind:test", "needs:user"} <= got, True)
    chk("whitelist drops free-form tags", got & {"from:agent", "effort:M", "tier:fable", "roadmap",
                                                 "federation", "noir"}, set())
    chk("adds priority+role", {"priority:P2", "role:impl"} <= got, True)
    # security keyword survives the whitelist (soundness routing depends on it downstream)
    chk("keeps security keyword", "area:sparq-zk" in issue_labels(
        {"priority": 1, "issue_type": "feature", "labels": ["area:sparq-zk", "from:x"]}), True)
    # epic gets kind:epic
    chk("epic tagged", "kind:epic" in issue_labels(
        {"priority": 1, "issue_type": "epic", "labels": ["area:x"]}), True)
    print("bd-to-issues self-test", "PASSED" if ok else "FAILED")
    return 0 if ok else 1


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default="sparq-org/sparq")
    ap.add_argument("--apply", action="store_true", help="actually create issues (bulk!) — held for go-ahead")
    ap.add_argument("--limit", type=int, help="migrate only the first N open beads (for testing)")
    ap.add_argument("--only", help="comma-separated bd-ids to migrate (curated pilot subset; ignores --limit)")
    ap.add_argument("--export-file", help="read bd export from a file instead of running bd")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()

    if args.self_test:
        return _self_test()

    if args.export_file:
        lines = open(args.export_file, encoding="utf-8").read().splitlines()
    else:
        lines = subprocess.run(["bd", "export"], capture_output=True, text=True, check=True).stdout.splitlines()
    issues, edges = parse_export(lines)
    open_ids, blockers, parents = plan(issues, edges)
    if args.only:  # curated subset — keep only the named bd-ids (blockers/parents stay scoped to them)
        keep = {x.strip() for x in args.only.split(",") if x.strip()}
        missing = keep - set(open_ids)
        if missing:
            print(f"warning: --only ids not in the open set (skipped): {sorted(missing)}")
        open_ids = {k: v for k, v in open_ids.items() if k in keep}

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

    tag = f" (limit {args.limit})" if args.limit else ""
    print(f"\n[apply] migrating to {args.repo}{tag} — idempotent two-pass...")
    id_map, created = apply_migration(args.repo, open_ids, blockers, parents, limit=args.limit)
    print(f"[apply] created {created} new issue(s); {len(id_map)} bd-ids mapped (re-run is a no-op).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
