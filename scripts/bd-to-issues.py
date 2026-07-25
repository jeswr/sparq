#!/usr/bin/env python3
# [OPUS-4.8] Issue-native orchestration: migrate bd beads -> GitHub issues (Phase 1).
#
# HISTORICAL TOOL — the migration RAN on 2026-07-17. Beads (`bd` / `.beads/`) has been
# retired and GitHub issues are now the sole tracker (see docs/bd-migration.md + the
# committed sq-id -> issue map in docs/bd-migration-map.json). This script is kept as the
# migration record + a re-runnable reference; it is NOT part of any live workflow, and there
# is no longer a `bd` DB to export from on this repo.
"""bd-to-issues.py — one-time, idempotent migration of open bd beads into GitHub issues.

DEFAULT is --dry-run: it parses `bd export`, computes the issue payloads + label mapping + the
dependency edges, and prints a summary WITHOUT creating anything. `--apply` does the real two-pass
(create issues, then link blocked-by dependencies) and writes the `sq-… ↔ #NN` map — held for the
maintainer's go-ahead because it bulk-creates hundreds of issues.

Label mapping (bd -> issue):
  priority 0..4            -> priority:P{n}
  existing labels          -> passed through verbatim (area:<crate>=package, needs:*, kind:*, from:*)
  issue_type / kind:       -> role:<r> (feature/bug->impl, docs->docs, spike->research, chore->ci, ...)
  every migrated issue     -> MIGRATION_LABEL (authenticates the `<!-- bd-id:… -->` body marker)
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
# `gh issue create` fails on the first unknown label → issues are only created against a PRE-CREATED
# full label set (ensure_labels), failing LOUDLY instead of silently dropping labels. Keep only what
# the pipeline consumes.
_KEEP_PREFIX = ("area:", "needs:", "trust:", "kind:")
_SEC_KEYWORDS = ("zk", "mpc", "reasoner", "crypto", "auth", "e2ee")

# Migration-owned authentication label (PR #2528 review): scripts/ci-close-merged-beads.py trusts
# a `<!-- bd-id:… -->` body marker ONLY on an issue carrying this label, because issue bodies are
# forgeable by any GitHub user while attaching a label needs triage/write permission. Stamped on
# EVERY migrated issue; a resumed --apply backfills it ONLY onto issues with migration-owned
# provenance (recorded in the trusted checkpoint — see resolve_resume_map), never onto arbitrary
# body-marker matches. Keep in sync with _MIGRATION_LABEL there, and never let an issue template
# auto-apply it.
MIGRATION_LABEL = "bd-migration"
_BLOCKED_ON = re.compile(r"^blocked[-:]on[-:](.+)$")
_BLOCKED = re.compile(r"^blocked:(.+)$")


def _map_label(lb):
    """Migration gating pass (audit-2026-07-17): bd `blocked:*` / `blocked-on-*` tags become
    `needs:*` gates — the readiness engine only gates on the needs:/trust: prefixes, so an unmapped
    `blocked:ec2` bead would migrate DISPATCHABLE and burn worker attempts on an un-runnable task.
    `blocked-by-epic:*` is DROPPED: parent-child edges already model epic linkage, and mapping it to
    needs:* would reintroduce the sub-epic-leaf-invisibility bug the migration deliberately fixes."""
    if lb.startswith("blocked-by-epic:"):
        return None
    m = _BLOCKED_ON.match(lb) or _BLOCKED.match(lb)
    return f"needs:{m.group(1)}" if m else lb


def _pipeline_relevant(lb):
    """A label triage/readiness/dispatch actually reads: an area:/needs:/trust:/kind: label, or one
    carrying a security keyword so triage's soundness routing survives even without an area:/kind:."""
    return lb.startswith(_KEEP_PREFIX) or any(k in lb for k in _SEC_KEYWORDS)


# External/audit gating (audit-2026-07-17): human/credential/externally gated beads (the sq-qhy4
# external-cryptographer-audit class, token-placement, maintainer-decision beads) must migrate as
# needs:user — NOT as plain dispatchable issues. needs:user is the ONLY durable parking state: the
# dispatcher's deferred-retry re-fires any needs-free status:deferred issue, so without this gate
# sq-qhy4 (P0, no area → the __global__ partition) would be dispatched FIRST, fail, and loop —
# serializing the frontier behind an issue no worker can implement.
_EXTERNAL_IDS = {"sq-qhy4", "sq-v286.8"}
_EXTERNAL_TITLE = re.compile(
    r"\bexternal\b.*\baudit\b|accredited|cryptographer|\bneeds:user\b|"
    r"maintainer[- ](sign-?off|review|greenlit|gated)|code-signing|notariz|"
    r"token (placement|upload|provisioning)", re.I)
_EXTERNAL_DESC = re.compile(r"agent[- ]out[- ]of[- ]scope|out[- ]of[- ]scope by definition", re.I)


def externally_gated(bead):
    """True when a bead is gated on a human/credential/external dependency (curated id list +
    title patterns + an explicit description marker). Deliberately errs toward over-gating:
    needs:user is maintainer-visible and trivially reversible, while a mis-dispatched external
    gate burns worker attempts and can reserve the global partition."""
    if str(bead.get("id", "")) in _EXTERNAL_IDS:
        return True
    if _EXTERNAL_TITLE.search(bead.get("title") or ""):
        return True
    return bool(_EXTERNAL_DESC.search((bead.get("description") or "")[:400]))


def issue_labels(bead):
    labels = [lb["name"] if isinstance(lb, dict) else lb for lb in bead.get("labels", [])]
    mapped = (_map_label(lb) for lb in labels)              # blocked:* -> needs:* gating pass
    out = {lb for lb in mapped if lb and _pipeline_relevant(lb)}   # whitelist — drop free-form tags
    p = bead.get("priority")
    if isinstance(p, int) and 0 <= p <= 4:
        out.add(f"priority:P{p}")
    out.add(f"role:{role_for(bead)}")
    out.add(MIGRATION_LABEL)  # authenticates the bd-id body marker downstream — see MIGRATION_LABEL
    if externally_gated(bead):
        out.add("needs:user")
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
    """(trusted, unverified) {bd-id: issue_number} maps from existing issues carrying a
    `<!-- bd-id:... -->` marker (resume). trusted = the issue ALSO carries MIGRATION_LABEL
    (attaching one needs triage/write permission); unverified = marker-only — could be a
    pre-label legacy migration OR an attacker decoy, so never trusted on its own
    (see resolve_resume_map, PR #2528 review round 2)."""
    out = _run(["gh", "issue", "list", "-R", repo, "--state", "all", "--limit", "10000",
                "--json", "number,body,labels"]).stdout
    trusted, unverified = {}, {}
    for it in json.loads(out or "[]"):
        mm = re.search(r"<!--\s*bd-id:(\S+?)\s*-->", it.get("body") or "")
        if not mm:
            continue
        labels = {lb["name"] if isinstance(lb, dict) else lb for lb in it.get("labels") or []}
        (trusted if MIGRATION_LABEL in labels else unverified)[mm.group(1)] = it["number"]
    return trusted, unverified


def resolve_resume_map(trusted, unverified, checkpoint_map):
    """Pure resume resolution (PR #2528 review round 2). A body marker alone is FORGEABLE: an
    unprivileged user can pre-create a decoy issue carrying a target `<!-- bd-id:… -->` marker,
    and a resume path that accepted it would (a) never create the real issue and (b) stamp the
    authentication label onto attacker content — after which ci-close-merged-beads.py would
    trust and close the decoy. So an existing issue is accepted as already-migrated ONLY on
    migration-owned provenance: it already carries MIGRATION_LABEL, or its exact
    (bd-id, issue_number) pair is recorded in the trusted on-disk checkpoint this migration
    wrote. A marker-only issue with neither is returned as `unverifiable` and the caller FAILS
    CLOSED (no label, no mapping, no create) for operator review.

    Returns (id_map, backfill, unverifiable): id_map = accepted {bd-id: number}; backfill = the
    subset needing MIGRATION_LABEL stamped (checkpoint-verified but not yet labeled);
    unverifiable = marker-only {bd-id: number} matches with no provenance."""
    id_map = dict(trusted)
    backfill, unverifiable = {}, {}
    for bid, num in unverified.items():
        if bid in id_map:
            continue  # an authenticated issue exists; the marker-only copy is a decoy/dup — inert
        if checkpoint_map.get(bid) == num:
            id_map[bid] = num
            backfill[bid] = num
        else:
            unverifiable[bid] = num
    return id_map, backfill, unverifiable


def _body(bead, bid):
    desc = (bead.get("description") or "").strip()
    return f"{desc}\n\n<!-- bd-id:{bid} -->\n"


def ensure_labels(repo, labels):
    """Idempotently create EVERY label the migration will use, BEFORE any issue create. Fails
    LOUDLY on any real failure: `gh issue create` errors on the first unknown label, and the old
    fallback dropped ALL labels — a silent label-less issue is permanently status:untriaged and
    loses its package partition. No --force: existing labels keep their curated colors."""
    failed = []
    for l in sorted(labels):
        r = _run(["gh", "label", "create", l, "-R", repo, "--color", "ededed",
                  "--description", "bd migration label"], check=False)
        if r.returncode != 0 and "already exists" not in (r.stderr or ""):
            failed.append(l)
    if failed:
        raise SystemExit(f"refusing --apply: {len(failed)} label(s) could not be created "
                         f"(fail-loud, no silent label drop): {failed[:10]}")


def _create_issue(repo, bid, bead):
    args = ["gh", "issue", "create", "-R", repo, "--title", f"{bid}: {bead['title']}", "--body", _body(bead, bid)]
    for l in issue_labels(bead):
        args += ["--label", l]
    r = _run(args, check=False)
    if r.returncode != 0:
        # FAIL LOUDLY (audit-2026-07-17): the old fallback retried with NO labels at all, silently
        # producing untriaged, partition-less issues. Labels are pre-created by ensure_labels, so a
        # failure here is a real problem the operator must see. The run is crash-resumable.
        raise SystemExit(f"issue create failed for {bid} (labels are pre-created; refusing the "
                         f"silent label-drop fallback): {(r.stderr or '').strip()[:300]}")
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
    trusted, unverified = fetch_bd_map(repo)
    try:
        ckpt = json.load(open(checkpoint, encoding="utf-8"))
    except (FileNotFoundError, json.JSONDecodeError):
        ckpt = {}
    id_map, backfill, unverifiable = resolve_resume_map(trusted, unverified, ckpt)
    ids = list(open_ids)[: limit] if limit else list(open_ids)
    # Fail closed (PR #2528 review round 2): a marker-only issue with NO migration provenance
    # (not labeled, not in the trusted checkpoint) may be an attacker decoy pre-created to
    # capture a bd-id. Refuse to label it, map to it, OR create a competing issue — stop for
    # operator review instead.
    bad = {b: n for b, n in unverifiable.items() if b in set(ids)}
    if bad:
        raise SystemExit(
            f"refusing --apply: {len(bad)} existing issue(s) carry a bd-id body marker but have "
            f"NO migration provenance (no '{MIGRATION_LABEL}' label, not in the checkpoint) — "
            f"possible decoys; review + label or close them manually: "
            f"{sorted((b, '#' + str(n)) for b, n in bad.items())[:10]}")
    # Pre-flight: the FULL label set exists before the first create (fail-loud, item 10).
    ensure_labels(repo, {l for bid in ids for l in issue_labels(open_ids[bid])})
    created = 0
    for bid in ids:                                  # pass 1
        if bid in id_map:
            if bid in backfill:
                # Resume: backfill the authentication label — ONLY on a checkpoint-verified
                # issue this migration created before MIGRATION_LABEL existed (idempotent).
                _run(["gh", "issue", "edit", str(id_map[bid]), "-R", repo,
                      "--add-label", MIGRATION_LABEL])
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
    # every migrated issue carries the authentication label the autoclose mapping requires
    chk("stamps the migration authentication label", MIGRATION_LABEL in got, True)
    # security keyword survives the whitelist (soundness routing depends on it downstream)
    chk("keeps security keyword", "area:sparq-zk" in issue_labels(
        {"priority": 1, "issue_type": "feature", "labels": ["area:sparq-zk", "from:x"]}), True)
    # epic gets kind:epic
    chk("epic tagged", "kind:epic" in issue_labels(
        {"priority": 1, "issue_type": "epic", "labels": ["area:x"]}), True)
    # blocked:* -> needs:* gating pass (audit-2026-07-17)
    chk("blocked:ec2 maps to needs:ec2", _map_label("blocked:ec2"), "needs:ec2")
    chk("blocked-on-zk maps to needs:zk", _map_label("blocked-on-zk"), "needs:zk")
    chk("blocked-by-epic is dropped", _map_label("blocked-by-epic:sq-3kd2g"), None)
    chk("plain label passes through", _map_label("area:sparq-core"), "area:sparq-core")
    got = set(issue_labels({"priority": 1, "issue_type": "task",
                            "labels": ["blocked:ec2", "blocked-by-epic:sq-x", "area:bench"]}))
    chk("mapped gate survives whitelist", ("needs:ec2" in got, "blocked:ec2" in got,
                                           any("epic" in lb for lb in got)), (True, False, False))
    # external/audit gating: the sq-qhy4 class must land needs:user (durably parked)
    qhy4 = {"id": "sq-qhy4", "priority": 0, "issue_type": "task",
            "title": "[cert][cryptoreview] External accredited-cryptographer audit of the ZK "
                     "verifier + Noir circuits", "labels": []}
    chk("sq-qhy4 class gated needs:user", "needs:user" in issue_labels(qhy4), True)
    chk("curated id gated", externally_gated({"id": "sq-v286.8", "title": "x"}), True)
    chk("needs:user title gated", externally_gated(
        {"id": "sq-z", "title": "needs:user — flip GitHub Pages source"}), True)
    chk("agent-out-of-scope desc gated", externally_gated(
        {"id": "sq-z", "title": "plain", "description": "Agent-out-of-scope by definition."}), True)
    chk("plain feature not gated", externally_gated(
        {"id": "sq-a", "title": "feat(core): add a parser fast-path",
         "description": "speed up ingest"}), False)
    # Resume provenance (PR #2528 review round 2): an attacker-authored body-only marker that
    # PRECEDES migration must be neither label-backfilled nor accepted as the canonical mapping —
    # it surfaces as unverifiable and apply_migration fails closed for operator review.
    idm, bf, unv = resolve_resume_map({}, {"sq-target": 7}, {})
    chk("pre-migration decoy is not accepted as the mapping", "sq-target" in idm, False)
    chk("pre-migration decoy is never label-backfilled", bf, {})
    chk("pre-migration decoy surfaces as unverifiable (fail closed)", unv, {"sq-target": 7})
    # A decoy alongside the real labeled issue is inert: the labeled issue wins, no backfill.
    idm, bf, unv = resolve_resume_map({"sq-real": 5}, {"sq-real": 6}, {})
    chk("labeled issue is canonical; shadowing decoy is inert",
        (idm, bf, unv), ({"sq-real": 5}, {}, {}))
    # Checkpoint provenance: a pre-label legacy issue THIS migration created resumes + backfills…
    idm, bf, unv = resolve_resume_map({}, {"sq-legacy": 9}, {"sq-legacy": 9})
    chk("checkpoint-verified legacy issue resumes and backfills",
        (idm, bf, unv), ({"sq-legacy": 9}, {"sq-legacy": 9}, {}))
    # …but only on an EXACT (bd-id, number) match — a different issue number is unverifiable.
    idm, bf, unv = resolve_resume_map({}, {"sq-legacy": 9}, {"sq-legacy": 8})
    chk("checkpoint number mismatch stays unverifiable", (idm, bf, unv),
        ({}, {}, {"sq-legacy": 9}))
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
    gated = sum(1 for b in open_ids.values() if "needs:user" in issue_labels(b))
    no_area = sum(1 for b in open_ids.values()
                  if str(b.get("issue_type", "")).lower() != "epic"
                  and not any(lb.startswith("area:") for lb in issue_labels(b)))
    print(f"beads (all): {len(issues)}  |  to migrate (open/in_progress): {len(open_ids)}")
    print(f"blocker (blocks) edges among migrated: {sum(len(v) for v in blockers.values())}")
    print(f"parent-child links among migrated:    {sum(len(v) for v in parents.values())}")
    print(f"gated needs:user (external/audit/blocked:*): {gated}")
    print(f"WARNING non-epic beads with NO area label (each reserves the serializing __global__ "
          f"partition): {no_area}")
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
