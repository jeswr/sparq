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
  every migrated issue     -> MIGRATION_LABEL (authenticates the `<!-- bd-id:… -->` body marker)
  dependency edges         -> `Blocked-by: #NN` body markers (resolved to native deps in --apply)
The bead id (`sq-…`) is preserved in the issue title so existing PR-title tokens still resolve.
"""
import argparse
import json
import os
import re
import subprocess
import sys
import time

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


def b2i_bd_labels(bead):
    """The bead's OWN labels (post blocked:*->needs:* mapping, pre-derivation) — used by the
    summary to report how many `area:` labels the migration DERIVED rather than inherited."""
    return {_map_label(lb["name"] if isinstance(lb, dict) else lb)
            for lb in bead.get("labels", [])} - {None}


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


# --- area (package) derivation ------------------------------------------------------------------
# Audit 2026-07-25 (sq-gj35r): 551 of the 895 open beads carry NO `area:` label, so they migrated
# into the readiness engine's serializing `__global__` partition. triage.py refuses to promote a
# no-area issue at all (it parks `needs:area`), and retriage.py only emits PROMOTING deltas — so
# such an issue is TERMINALLY `status:untriaged`: present in GitHub, invisible to the orchestrator.
# Migrating an issue the orchestrator cannot classify is not migration, it is litter. So derive the
# package from the bead's own text, and when nothing is derivable make the parked state EXPLICIT
# with `needs:area` (a gate ready-issues.py already respects + maintainer-visible) instead of
# silently reserving the global partition.
_SURFACE_AREAS = {"site": "site", "gui": "gui", "bench": "bench", "ci": "ci", "docs": "docs",
                  "js": "js", "wasm": "sparq-wasm", "workflows": "ci", "release": "release",
                  "orchestration": "orchestration", "deps": "deps", "workspace": "workspace"}
# `verb(scope):` / `scope:` conventional title prefixes, e.g. "bug(sparq-solid): …", "site: …".
_TITLE_SCOPE = re.compile(r"^\s*(?:[a-z]+\(([a-z0-9\-/]+)\)|([a-z0-9\-]+))\s*:", re.I)
_CRATES_CACHE = None


def crate_names(root=None):
    """The workspace crate names (`crates/`), cached. Empty when run outside a checkout — the
    derivation then falls back to the conventional-title-prefix + surface rules only."""
    global _CRATES_CACHE
    if root is None and _CRATES_CACHE is not None:
        return _CRATES_CACHE
    base = root or os.path.join(os.path.dirname(os.path.abspath(__file__)), os.pardir, "crates")
    try:
        names = tuple(sorted(d for d in os.listdir(base)
                             if os.path.isdir(os.path.join(base, d)) and not d.startswith(".")))
    except OSError:
        names = ()
    if root is None:
        _CRATES_CACHE = names
    return names


def _scope_to_area(scope, crates):
    scope = (scope or "").lower().split("/")[0]
    for cand in (scope, f"sparq-{scope}"):
        if cand in crates:
            return cand
    return _SURFACE_AREAS.get(scope)


def _token_hits(names, text):
    """`names` occurring in `text` as whole tokens, ordered by FIRST OCCURRENCE, longest name
    winning over any name it contains (`sparq-reason-el` over `sparq-reason`). First-occurrence
    order matters: an alphabetical sort followed by a `[:limit]` truncation silently drops the
    most-specific hit — "deploy(site+docs): …" derived `ci,docs` and discarded `site`."""
    found = []
    for n in sorted(names, key=len, reverse=True):
        m = re.search(r"(?<![a-z0-9\-])" + re.escape(n) + r"(?![a-z0-9\-])", text)
        if m and not any(n in f for _, f in found):
            found.append((m.start(), n))
    return [n for _, n in sorted(found)]


def derive_areas(bead, crates=None, limit=2):
    """Derive `area:` values from a bead's own text, most-specific evidence first:

      1. the conventional `verb(scope):` / `scope:` title prefix — an explicit author declaration;
      2. crate names appearing as whole tokens in the TITLE;
      3. a surface keyword (site/gui/bench/ci/…) in the title's SCOPE REGION only — i.e. before the
         first colon. A bare "site"/"ci"/"docs" anywhere in running prose is NOT evidence: matching
         the whole title mislabeled a zkSPARQL spec bead `area:site` off one incidental word;
      4. the description, and ONLY when it names exactly ONE crate. A description that name-drops
         several neighbouring crates is a cross-cutting task, not a package assignment — that rule
         is what stopped "formalize codex-EC2 worker tooling" deriving `sparq-core,sparq-jsonld`.

    Returns [] when nothing is derivable; the caller then parks the issue `needs:area` rather than
    guessing. A wrong partition is worse than an explicit park: the park is maintainer-visible and
    retriage re-promotes it the moment an area lands, whereas a wrong area silently routes the work."""
    crates = crate_names() if crates is None else crates
    title = bead.get("title") or ""
    low = title.lower()
    m = _TITLE_SCOPE.match(title)
    if m:
        a = _scope_to_area(m.group(1) or m.group(2), crates)
        if a:
            return [a]
    hits = _token_hits(crates, low)
    if hits:
        return hits[:limit]
    scope_region = low.split(":", 1)[0] if ":" in low[:60] else ""
    surf = [_SURFACE_AREAS[k] for k in _token_hits(_SURFACE_AREAS, scope_region)]
    if surf:
        return list(dict.fromkeys(surf))[:limit]
    desc_hits = _token_hits(crates, (bead.get("description") or "")[:400].lower())
    return desc_hits if len(desc_hits) == 1 else []


def issue_labels(bead, crates=None):
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
    is_epic = str(bead.get("issue_type", "")).lower() == "epic"
    if is_epic:
        out.add("kind:epic")
    # Package partition (audit 2026-07-25) — see the derive_areas note. An epic is never
    # dispatched, so it needs no partition and is never needs:area-parked.
    if not is_epic and not any(lb.startswith("area:") for lb in out):
        derived = derive_areas(bead, crates)
        if derived:
            out |= {f"area:{a}" for a in derived}
        elif "needs:user" not in out:
            out.add("needs:area")
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


# Shape-constrained (audit 2026-07-25, issue #3797): the old `(\S+?)` capture matched ANY
# non-space run, so PROSE documenting the marker with an elided id ("<!-- bd-id:… -->" inside
# issues #2534/#2535/#3797) was scanned as a bead named "…" and injected into the migration map.
# Keep in sync with scripts/ci-close-merged-beads.py `_MARKER_RE`.
MARKER_RE = re.compile(r"<!--\s*bd-id:(sq-[0-9a-z]+(?:\.\d+)*)\s*-->")


def fetch_issues(repo):
    """Every issue (any state) with the fields the resume + reconcile passes need. ONE LIST call —
    the search index lags badly on this repo, so current state must come from the list API."""
    out = _run(["gh", "issue", "list", "-R", repo, "--state", "all", "--limit", "10000",
                "--json", "number,id,body,labels,author,title"]).stdout
    return json.loads(out or "[]")


def _writer_logins(repo, logins, cache=None):
    """The subset of `logins` with write-or-better permission on `repo` — the SAME trust primitive
    triage-issue.yml uses. An unprivileged decoy author (the resolve_resume_map threat model)
    cannot be in this set, which is exactly what makes author provenance load-bearing."""
    cache = {} if cache is None else cache
    ok = set()
    for lg in sorted(set(logins)):
        if lg not in cache:
            if lg.endswith("[bot]"):
                # App bots are not collaborators; the orchestrator App is the migration's own
                # identity and no unprivileged user can author as a `[bot]` login.
                cache[lg] = lg in _MIGRATION_BOTS
            else:
                r = _run(["gh", "api", f"repos/{repo}/collaborators/{lg}/permission",
                          "--jq", ".permission"], check=False)
                cache[lg] = (r.stdout or "").strip() in ("admin", "maintain", "write")
        if cache[lg]:
            ok.add(lg)
    return ok


_MIGRATION_BOTS = {"sparq-orchestrator[bot]"}


def migration_owned(issues, writers):
    """bd-ids whose marker-only issue carries MIGRATION-OWNED provenance, i.e. all of:
      * the marker payload is a real bd-id SHAPE (MARKER_RE — kills the #3797 phantom);
      * the title is the migration's own `"<bd-id>: <bead title>"` format (a decoy that copies a
        marker to capture a bead id does not also reproduce the title contract);
      * the AUTHOR has write-or-better permission on the repo — the threat model's attacker is by
        definition unprivileged, so this closes the forgeable-body hole the label was added for;
      * exactly ONE issue in the repo carries that bd-id (ambiguity is never resolved by guessing).
    This recovers the ~750 issues of the 2026-07-17 bulk run that predate MIGRATION_LABEL: without
    it every subsequent `--apply` aborts on the fail-closed unverifiable check and the migration is
    permanently unresumable."""
    seen = {}
    for it in issues:
        mm = MARKER_RE.search(it.get("body") or "")
        if not mm:
            continue
        seen.setdefault(mm.group(1), []).append(it)
    owned = set()
    for bid, cands in seen.items():
        if len(cands) != 1:
            continue                                  # ambiguous — fail closed, never guess
        it = cands[0]
        login = (it.get("author") or {}).get("login") if isinstance(it.get("author"), dict) \
            else it.get("author")
        if login in writers and (it.get("title") or "").startswith(f"{bid}:"):
            owned.add(bid)
    return owned


def fetch_bd_map(repo, issues=None):
    """(trusted, unverified) {bd-id: issue_number} maps from existing issues carrying a
    `<!-- bd-id:... -->` marker (resume). trusted = the issue ALSO carries MIGRATION_LABEL
    (attaching one needs triage/write permission); unverified = marker-only — could be a
    pre-label legacy migration OR an attacker decoy, so never trusted on its own
    (see resolve_resume_map, PR #2528 review round 2)."""
    issues = fetch_issues(repo) if issues is None else issues
    trusted, unverified = {}, {}
    for it in issues:
        mm = MARKER_RE.search(it.get("body") or "")
        if not mm:
            continue
        labels = {lb["name"] if isinstance(lb, dict) else lb for lb in it.get("labels") or []}
        (trusted if MIGRATION_LABEL in labels else unverified)[mm.group(1)] = it["number"]
    return trusted, unverified


def resolve_resume_map(trusted, unverified, checkpoint_map, owned=frozenset()):
    """Pure resume resolution (PR #2528 review round 2). A body marker alone is FORGEABLE: an
    unprivileged user can pre-create a decoy issue carrying a target `<!-- bd-id:… -->` marker,
    and a resume path that accepted it would (a) never create the real issue and (b) stamp the
    authentication label onto attacker content — after which ci-close-merged-beads.py would
    trust and close the decoy. So an existing issue is accepted as already-migrated ONLY on
    migration-owned provenance: it already carries MIGRATION_LABEL, its exact
    (bd-id, issue_number) pair is recorded in the trusted on-disk checkpoint this migration
    wrote, or its bd-id is in `owned` (see migration_owned: unique marker + migration title
    format + a WRITE-permission author, which the unprivileged decoy author cannot satisfy).
    A marker-only issue with none of the three is returned as `unverifiable` and the caller FAILS
    CLOSED (no label, no mapping, no create) for operator review.

    Returns (id_map, backfill, unverifiable): id_map = accepted {bd-id: number}; backfill = the
    subset needing MIGRATION_LABEL stamped (checkpoint-verified but not yet labeled);
    unverifiable = marker-only {bd-id: number} matches with no provenance."""
    id_map = dict(trusted)
    backfill, unverifiable = {}, {}
    for bid, num in unverified.items():
        if bid in id_map:
            continue  # an authenticated issue exists; the marker-only copy is a decoy/dup — inert
        if checkpoint_map.get(bid) == num or bid in owned:
            id_map[bid] = num
            backfill[bid] = num
        else:
            unverifiable[bid] = num
    return id_map, backfill, unverifiable


# Agent self-identification (standing repo rule): every issue/comment an agent opens says so, so a
# maintainer scanning the tracker can tell machine-filed items from human ones at a glance.
SELF_ID = "> 🤖 SPARQ agent — migrated from the local `bd` tracker by `scripts/bd-to-issues.py`."


def _body(bead, bid):
    desc = (bead.get("description") or "").strip()
    return f"{SELF_ID}\n\n{desc}\n\n<!-- bd-id:{bid} -->\n"


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


def _create_issue(repo, bid, bead, crates=None):
    args = ["gh", "issue", "create", "-R", repo, "--title", f"{bid}: {bead['title']}", "--body", _body(bead, bid)]
    for l in issue_labels(bead, crates):
        args += ["--label", l]
    r = _run(args, check=False)
    if r.returncode != 0:
        # FAIL LOUDLY (audit-2026-07-17): the old fallback retried with NO labels at all, silently
        # producing untriaged, partition-less issues. Labels are pre-created by ensure_labels, so a
        # failure here is a real problem the operator must see. The run is crash-resumable.
        raise SystemExit(f"issue create failed for {bid} (labels are pre-created; refusing the "
                         f"silent label-drop fallback): {(r.stderr or '').strip()[:300]}")
    return int(re.search(r"/issues/(\d+)", r.stdout).group(1))


def _snapshot_body(issues, num):
    """The issue body already held in the run's single LIST snapshot, or None if unknown."""
    if num is None:
        return None
    for it in issues:
        if it["number"] == num:
            return it.get("body") or ""
    return None


def _ensure_markers(repo, num, markers, body=None):
    """Append every missing edge marker to the issue body in ONE edit (idempotent). Returns True
    if it wrote.

    Two reasons this is plural rather than a per-marker call: (a) an issue with both a Blocked-by
    and a Parent edge would otherwise be read-modify-written twice, and the second write built from
    a STALE body silently drops the first marker; (b) `body` short-circuits the per-issue read using
    the run's single LIST snapshot — pass 2 walks 558 edges, so a no-op re-run used to cost 558 API
    reads just to discover it had nothing to do. A no-op re-run must be cheap, or nobody runs it."""
    if not markers:
        return False
    if body is None:
        body = json.loads(_run(["gh", "issue", "view", str(num), "-R", repo,
                                "--json", "body"]).stdout)["body"] or ""
    missing = [m for m in markers if m not in body]
    if not missing:
        return False
    _run(["gh", "issue", "edit", str(num), "-R", repo,
          "--body", body.rstrip() + "\n" + "\n".join(missing) + "\n"])
    return True


# --- ADD-only label reconciliation, batched (audit 2026-07-25) -----------------------------------
def plan_reconcile(open_ids, id_map, have_labels, crates=None):
    """{issue_number: [labels to ADD]} for already-mapped OPEN beads whose issue is missing labels
    the plan requires. ADD-only by construction — never removes, so a human's curation survives.
    An issue that already carries ANY `area:` keeps it (a human may have refined the partition):
    the derived area AND the `needs:area` park are both suppressed in that case."""
    out = {}
    for bid, num in sorted(id_map.items()):
        bead = open_ids.get(bid)
        if bead is None:
            continue                                   # bead is closed/absent — not our business
        have = set(have_labels.get(num) or ())
        want = set(issue_labels(bead, crates))
        if any(lb.startswith("area:") for lb in have):
            want = {lb for lb in want if not lb.startswith("area:") and lb != "needs:area"}
        gap = want - have
        if gap:
            out[num] = sorted(gap)
    return out


def _chunks(seq, n):
    seq = list(seq)
    return [seq[i:i + n] for i in range(0, len(seq), n)]


def _label_node_ids(repo, names):
    """{label name: node id} via GraphQL (the REST/`gh label` surfaces do not expose node ids)."""
    owner, name = repo.split("/", 1)
    q = ('query($o:String!,$r:String!,$c:String){repository(owner:$o,name:$r){'
         'labels(first:100,after:$c){nodes{id name} pageInfo{hasNextPage endCursor}}}}')
    ids, cursor = {}, None
    while True:
        args = ["gh", "api", "graphql", "-f", f"query={q}", "-F", f"o={owner}", "-F", f"r={name}"]
        args += ["-F", f"c={cursor}"] if cursor else ["-F", "c="]
        page = json.loads(_run(args).stdout)["data"]["repository"]["labels"]
        ids.update({n["name"]: n["id"] for n in page["nodes"]})
        if not page["pageInfo"]["hasNextPage"]:
            break
        cursor = page["pageInfo"]["endCursor"]
    missing = set(names) - set(ids)
    if missing:
        raise SystemExit(f"label node ids missing (run ensure_labels first): {sorted(missing)[:10]}")
    return ids


def apply_reconcile(repo, plan, node_ids, batch=8, pause=2.0, log=print):
    """Apply `plan` ({issue_number: [labels]}) with ONE GraphQL request per `batch` issues.

    Batching is not cosmetic: ~800 single `gh issue edit` mutations trip GitHub's secondary
    rate limiter, and a 403 mid-run leaves the repo half-labeled. Aliased mutations keep the
    whole reconcile inside ~100 requests, paced under the documented GraphQL mutation budget
    (5 points/mutation, 2000 points/min).

    Two distinct server refusals, two distinct responses (both learned from a real run):
      * `Resource limits for this query exceeded` — the REQUEST is too big. A batch of 20 hit
        this on the very first chunk, and GitHub had already executed the first few aliases
        before refusing, so a naive retry of the same shape would loop forever while partially
        applying. HALVE the chunk and retry; ADD-only mutations make the replay harmless.
      * a secondary-rate-limit 403 — the request is fine but too frequent. Back off, don't split.
    """
    if not plan:
        return 0
    label_ids = _label_node_ids(repo, {l for v in plan.values() for l in v})
    queue, done, total = _chunks(sorted(plan), batch), 0, len(plan)
    while queue:
        chunk = queue.pop(0)
        muts = []
        for i, num in enumerate(chunk):
            lids = ",".join(f'"{label_ids[l]}"' for l in plan[num])
            muts.append(f'm{i}: addLabelsToLabelable(input: {{labelableId: "{node_ids[num]}", '
                        f'labelIds: [{lids}]}}) {{ __typename }}')
        query = "mutation {\n  " + "\n  ".join(muts) + "\n}"
        applied = False
        for attempt in range(5):
            r = _run(["gh", "api", "graphql", "-f", f"query={query}"], check=False)
            if r.returncode == 0 and '"errors"' not in (r.stdout or ""):
                applied = True
                break
            err = ((r.stderr or "") + (r.stdout or "")).lower()
            if "resource limit" in err or "query is too large" in err:
                if len(chunk) == 1:
                    raise SystemExit(f"label reconcile: issue #{chunk[0]} is irreducibly over the "
                                     f"GraphQL resource limit: {err[:300]}")
                half = max(1, len(chunk) // 2)
                log(f"  request over the GraphQL resource limit — splitting {len(chunk)} -> {half}")
                queue[:0] = [chunk[:half], chunk[half:]]
                break
            if "secondary rate" in err or "abuse" in err or "was submitted too quickly" in err:
                wait = 30 * (2 ** attempt)
                log(f"  secondary rate limit — backing off {wait}s")
                time.sleep(wait)
                continue
            raise SystemExit(f"label reconcile failed on issues {chunk}: "
                             f"{((r.stderr or '') + (r.stdout or '')).strip()[:400]}")
        else:
            raise SystemExit(f"label reconcile: still rate-limited after 5 backoffs at {chunk}")
        if not applied:
            continue                       # split happened; the halves are back on the queue
        done += len(chunk)
        log(f"  reconciled {done}/{total} issue(s)")
        time.sleep(pause)
    return done


def apply_migration(repo, open_ids, blockers, parents, limit=None, checkpoint="/tmp/bd-migration-map.json",
                    reconcile=True, crates=None):
    """Resumable: pass 1 upserts issues by bd-id marker (checkpointing the map); pass 2 idempotently
    adds Blocked-by:/Parent: markers; pass 3 ADD-only label reconcile over already-mapped issues.
    Re-running creates nothing new and reconciles nothing."""
    issues_snapshot = fetch_issues(repo)
    node_ids = {it["number"]: it["id"] for it in issues_snapshot}
    have_labels = {it["number"]: [lb["name"] if isinstance(lb, dict) else lb
                                  for lb in it.get("labels") or []] for it in issues_snapshot}
    trusted, unverified = fetch_bd_map(repo, issues_snapshot)
    try:
        ckpt = json.load(open(checkpoint, encoding="utf-8"))
    except (FileNotFoundError, json.JSONDecodeError):
        ckpt = {}
    authors = {((it.get("author") or {}).get("login") if isinstance(it.get("author"), dict)
                else it.get("author")) for it in issues_snapshot}
    owned = migration_owned(issues_snapshot, _writer_logins(repo, {a for a in authors if a}))
    id_map, backfill, unverifiable = resolve_resume_map(trusted, unverified, ckpt, owned)
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
    ensure_labels(repo, {l for bid in ids for l in issue_labels(open_ids[bid], crates)})
    created, fresh = 0, {}
    for bid in ids:                                  # pass 1
        if bid in id_map:
            continue      # already migrated; MIGRATION_LABEL backfill is pass 3's job (batched)
        id_map[bid] = _create_issue(repo, bid, open_ids[bid], crates)
        created += 1
        fresh[bid] = _body(open_ids[bid], bid)
        have_labels[id_map[bid]] = issue_labels(open_ids[bid], crates)
        json.dump(id_map, open(checkpoint, "w"))     # checkpoint immediately (crash-resumable)
    edged = 0
    for bid in ids:                                  # pass 2 (idempotent)
        num = id_map.get(bid)
        markers = ([f"Blocked-by: #{id_map[b]}" for b in blockers.get(bid, []) if b in id_map]
                   + [f"Parent: #{id_map[p]}" for p in parents.get(bid, []) if p in id_map])
        if not markers:
            continue
        body = fresh.get(bid, _snapshot_body(issues_snapshot, num))
        edged += bool(_ensure_markers(repo, num, markers, body))
    # pass 3 (idempotent, ADD-only, batched): reconcile labels over every mapped OPEN bead. This
    # is also where `backfill` lands — MIGRATION_LABEL is part of every plan's label set, so the
    # ~750 pre-label issues of the 2026-07-17 bulk run get authenticated here in ~40 requests
    # rather than ~750 single edits (which trip the secondary rate limiter).
    reconciled = 0
    if reconcile:
        rplan = plan_reconcile({b: open_ids[b] for b in ids}, id_map, have_labels, crates)
        if rplan:
            ensure_labels(repo, {l for v in rplan.values() for l in v})
            node_ids.update({n: node_ids[n] for n in rplan if n in node_ids})
            unknown = [n for n in rplan if n not in node_ids]
            if unknown:                     # freshly created this run — re-fetch their node ids
                for it in fetch_issues(repo):
                    node_ids[it["number"]] = it["id"]
            reconciled = apply_reconcile(repo, rplan, node_ids)
    if backfill:
        print(f"[apply] authenticated {len(backfill)} pre-label issue(s) via migration-owned "
              f"provenance (unique marker + migration title format + write-permission author)")
    return id_map, created, reconciled, edged


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

    # --- marker shape (audit 2026-07-25, #3797) --------------------------------------------------
    # Prose that DOCUMENTS the marker is not a bead mapping. The old `(\S+?)` capture scanned "…"
    # as a bead id on three real issues and injected that phantom into the migration map.
    chk("prose marker with an elided id does not scan",
        MARKER_RE.search("fetch_bd_map trusts `<!-- bd-id:… -->` markers"), None)
    chk("non-bead marker payload does not scan", MARKER_RE.search("<!-- bd-id:not-a-bead -->"), None)
    chk("dotted subtask ids still scan",
        MARKER_RE.search("<!-- bd-id:sq-7d3dj.32.2.3 -->").group(1), "sq-7d3dj.32.2.3")

    # --- migration-owned provenance (audit 2026-07-25) -------------------------------------------
    # The 2026-07-17 bulk run predates MIGRATION_LABEL: ~750 marker-only issues. Without a third
    # provenance source every later --apply aborts on the fail-closed check, i.e. the migration is
    # unresumable. Provenance = unique marker + migration title format + WRITE-permission author;
    # the decoy threat model's attacker is unprivileged, so the author check is what closes the hole.
    legit = {"number": 42, "title": "sq-legit: do the thing",
             "body": "x\n<!-- bd-id:sq-legit -->\n", "author": {"login": "maintainer"}}
    decoy = {"number": 43, "title": "please look at this",
             "body": "<!-- bd-id:sq-victim -->", "author": {"login": "randomuser"}}
    wrongtitle = {"number": 44, "title": "unrelated title",
                  "body": "<!-- bd-id:sq-titleless -->", "author": {"login": "maintainer"}}
    prose_iss = {"number": 45, "title": "bug: scanner matches prose",
                 "body": "matches `<!-- bd-id:… -->`", "author": {"login": "maintainer"}}
    owned = migration_owned([legit, decoy, wrongtitle, prose_iss], {"maintainer"})
    chk("write-author + title contract is migration-owned", owned, {"sq-legit"})
    chk("unprivileged decoy author is NOT owned", "sq-victim" in owned, False)
    chk("write author WITHOUT the title contract is NOT owned", "sq-titleless" in owned, False)
    dup = [dict(legit), dict(legit, number=46)]
    chk("two issues sharing a bd-id are never owned (fail closed)",
        migration_owned(dup, {"maintainer"}), set())
    idm, bf, unv = resolve_resume_map({}, {"sq-legit": 42}, {}, {"sq-legit"})
    chk("owned marker-only issue resumes and backfills the auth label",
        (idm, bf, unv), ({"sq-legit": 42}, {"sq-legit": 42}, {}))
    idm, bf, unv = resolve_resume_map({}, {"sq-victim": 43}, {}, {"sq-legit"})
    chk("un-owned marker-only issue still fails closed",
        (idm, bf, unv), ({}, {}, {"sq-victim": 43}))

    # --- area (package) derivation (audit 2026-07-25) --------------------------------------------
    # A no-area issue reserves the readiness engine's serializing __global__ partition and triage.py
    # refuses to promote it, so it is TERMINALLY status:untriaged — present in GitHub, invisible to
    # the orchestrator. 551 of the 895 open beads had no area label.
    cr = ("sparq-core", "sparq-jsonld", "sparq-reason", "sparq-reason-el", "sparq-solid",
          "sparq-zk")
    chk("conventional verb(scope): prefix wins",
        derive_areas({"title": "bug(sparq-solid): ODRL bridge drops a rule",
                      "description": "touches sparq-core too"}, cr), ["sparq-solid"])
    chk("bare scope: prefix maps a surface",
        derive_areas({"title": "site: honest competitive feature matrix"}, cr), ["site"])
    chk("title crate mention beats description mention",
        derive_areas({"title": "widen the sparq-zk verifier",
                      "description": "sparq-core and sparq-solid are involved"}, cr), ["sparq-zk"])
    chk("longest crate name wins over its own prefix",
        derive_areas({"title": "sparq-reason-el RBox regularity check"}, cr), ["sparq-reason-el"])
    chk("description is the last resort",
        derive_areas({"title": "no scope here at all",
                      "description": "the fix lands in sparq-core"}, cr), ["sparq-core"])
    chk("nothing derivable -> [] (never guess)",
        derive_areas({"title": "LinkedIn advert post", "description": "enumerate claims"}, cr), [])
    # precision regressions found by sampling the real backlog (audit 2026-07-25)
    chk("a multi-crate description name-drop is NOT a package assignment",
        derive_areas({"title": "formalize codex-EC2 worker tooling",
                      "description": "moves sparq-core and sparq-jsonld notes"}, cr), [])
    chk("a surface word in running prose is not evidence",
        derive_areas({"title": "zkSPARQL spec: transcribe normative cores into the draft site"},
                     cr), [])
    chk("surface hits keep first-occurrence order, not alphabetical",
        derive_areas({"title": "deploy(site+docs): a /deploy surface"}, cr), ["site", "docs"])
    chk("first-occurrence ordering, longest-name-wins",
        _token_hits(("sparq-core", "sparq-reason", "sparq-reason-el"),
                    "sparq-reason-el then sparq-core"), ["sparq-reason-el", "sparq-core"])
    got = set(issue_labels({"id": "sq-a", "priority": 2, "issue_type": "task",
                            "title": "bug(sparq-solid): x", "labels": []}, cr))
    chk("derived area lands on the issue", "area:sparq-solid" in got, True)
    chk("derived area suppresses the needs:area park", "needs:area" in got, False)
    got = set(issue_labels({"id": "sq-b", "priority": 2, "issue_type": "task",
                            "title": "undecidable title", "labels": []}, cr))
    chk("no derivable area -> explicit needs:area park (not silent __global__)",
        "needs:area" in got, True)
    chk("an epic is never needs:area-parked (never dispatched)",
        "needs:area" in issue_labels({"id": "sq-c", "priority": 1, "issue_type": "epic",
                                      "title": "EPIC: umbrella", "labels": []}, cr), False)
    chk("a bd area label is never overridden by derivation",
        [lb for lb in issue_labels({"id": "sq-d", "priority": 2, "issue_type": "task",
                                    "title": "bug(sparq-solid): x",
                                    "labels": ["area:sparq-core"]}, cr)
         if lb.startswith("area:")], ["area:sparq-core"])

    # --- ADD-only label reconcile (audit 2026-07-25) ---------------------------------------------
    beads = {"sq-a": {"id": "sq-a", "priority": 2, "issue_type": "task",
                      "title": "bug(sparq-solid): x", "labels": []}}
    want = set(issue_labels(beads["sq-a"], cr))
    chk("reconcile plans exactly the missing labels",
        set(plan_reconcile(beads, {"sq-a": 10}, {10: []}, cr)[10]), want)
    chk("reconcile is a no-op once the labels are present",
        plan_reconcile(beads, {"sq-a": 10}, {10: sorted(want)}, cr), {})
    chk("reconcile never removes a label the issue gained elsewhere",
        plan_reconcile(beads, {"sq-a": 10}, {10: sorted(want) + ["status:ready", "role:site"]},
                       cr), {})
    chk("a human-curated area is preserved (no second area, no needs:area)",
        plan_reconcile({"sq-b": {"id": "sq-b", "priority": 2, "issue_type": "task",
                                 "title": "undecidable", "labels": []}},
                       {"sq-b": 11}, {11: ["area:sparq-core", "priority:P2", "role:impl",
                                           MIGRATION_LABEL]}, cr), {})
    chk("an unmapped/closed bead is never reconciled",
        plan_reconcile(beads, {"sq-zzz": 12}, {12: []}, cr), {})
    chk("batching chunks evenly", [len(c) for c in _chunks(range(45), 20)], [20, 20, 5])
    chk("empty reconcile does zero API calls", apply_reconcile("o/r", {}, {}), 0)
    # Adaptive split on the server's `Resource limits for this query exceeded` refusal (a real
    # 20-mutation batch hit it on the first chunk). Every issue must still be applied EXACTLY
    # once overall, and a single irreducible issue must fail loud rather than loop.
    calls = []

    def fake_run(args, check=True):
        q = args[-1]
        n = q.count("addLabelsToLabelable")
        calls.append(n)
        big = n > 2
        return type("R", (), {"returncode": 1 if big else 0,
                              "stdout": "" if big else "{}",
                              "stderr": "Resource limits for this query exceeded" if big else ""})()

    real_run, real_ids = _run, _label_node_ids
    try:
        globals()["_run"] = fake_run
        globals()["_label_node_ids"] = lambda repo, names: {n: f"L_{n}" for n in names}
        plan5 = {n: ["role:impl"] for n in range(1, 6)}
        got = apply_reconcile("o/r", plan5, {n: f"I_{n}" for n in plan5}, batch=5, pause=0)
        chk("split-and-retry still applies every issue exactly once", got, 5)
        # 5 refused -> [2,3]; 2 applied; 3 refused -> [1,2]; 1 applied; 2 applied. A refused size
        # is NEVER re-issued at the same size (that would loop while partially applying).
        chk("oversize batch splits strictly downward, never retried at the same size",
            calls, [5, 2, 3, 1, 2])
        calls.clear()
        globals()["_run"] = lambda a, check=True: type("R", (), {
            "returncode": 1, "stdout": "", "stderr": "Resource limits for this query exceeded"})()
        try:
            apply_reconcile("o/r", {1: ["role:impl"]}, {1: "I_1"}, batch=1, pause=0)
            chk("irreducible oversize issue fails loud", "no-exit", "SystemExit")
        except SystemExit:
            chk("irreducible oversize issue fails loud", "SystemExit", "SystemExit")
    finally:
        globals()["_run"], globals()["_label_node_ids"] = real_run, real_ids

    print("bd-to-issues self-test", "PASSED" if ok else "FAILED")
    return 0 if ok else 1


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default="sparq-org/sparq")
    ap.add_argument("--apply", action="store_true", help="actually create issues (bulk!) — held for go-ahead")
    ap.add_argument("--limit", type=int, help="migrate only the first N open beads (for testing)")
    ap.add_argument("--only", help="comma-separated bd-ids to migrate (curated pilot subset; ignores --limit)")
    ap.add_argument("--export-file", help="read bd export from a file instead of running bd")
    ap.add_argument("--no-reconcile", action="store_true",
                    help="skip the ADD-only label reconcile over already-migrated issues")
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
    derived = sum(1 for b in open_ids.values()
                  if not any(lb.startswith("area:") for lb in b2i_bd_labels(b))
                  and any(lb.startswith("area:") for lb in issue_labels(b)))
    parked = sum(1 for b in open_ids.values() if "needs:area" in issue_labels(b))
    print(f"beads (all): {len(issues)}  |  to migrate (open/in_progress): {len(open_ids)}")
    print(f"blocker (blocks) edges among migrated: {sum(len(v) for v in blockers.values())}")
    print(f"parent-child links among migrated:    {sum(len(v) for v in parents.values())}")
    print(f"gated needs:user (external/audit/blocked:*): {gated}")
    print(f"area: derived from bead text (bd carried none): {derived}")
    print(f"no derivable area -> parked needs:area (maintainer-visible, never silent "
          f"__global__): {parked}")
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
    print(f"\n[apply] migrating to {args.repo}{tag} — idempotent three-pass...")
    id_map, created, reconciled, edged = apply_migration(
        args.repo, open_ids, blockers, parents, limit=args.limit,
        reconcile=not args.no_reconcile)
    print(f"[apply] created {created} new issue(s); {edged} issue(s) gained edge markers; "
          f"{reconciled} issue(s) label-reconciled; {len(id_map)} bd-ids mapped.")
    print("[apply] a second run with the same inputs must report 0/0/0 — that is the "
          "idempotence contract.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
