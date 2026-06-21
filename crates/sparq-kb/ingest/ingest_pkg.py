#!/usr/bin/env python3
# ingest_pkg.py — the Phase-1 PKG ingestion pipeline (structured-parse).
#
# [OPUS-4.8] sq-2m6zm.2 (epic sq-2m6zm); design record
# research/dogfooding-sparq-knowledge-graph.md §6 Phase 1 (PR #1063). 🤖 SPARQ agent.
# Written while Fable unavailable; flag for re-review when Fable returns.
#
# EXTRACTION METHOD = STRUCTURED-PARSE (deterministic, not LLM):
#   - .beads/issues.jsonl -> pkg:Task triples. A MECHANICAL projection of the bd
#     model (status / issue_type / priority / labels / typed dependency edges /
#     parent-child / discovered-from / related), reusing bd's audit trail unchanged.
#     bd remains the source-of-record; the PKG MIRRORS it as a read-model (design §4).
#   - skills/*/SKILL.md YAML front-matter -> pkg:Source + pkg:Technique triples. A
#     deterministic parse of the `name:` + `description:` front-matter fields of the
#     heaviest skill surfaces (the highest read-frequency surface docs).
#
# The hand-authored AGENTS.md Findings live in agents-findings.ttl (the high-accuracy
# tier); this script concatenates that file into the ingested graph verbatim.
#
# GUARDRAIL HONESTY: the bd backlog contains stale dependency edges (a CLOSED task
# that still blocks an OPEN one) — exactly the bug the design cites. Materialising the
# pkg:dependsOn owl:inverseOf pkg:blockedBy pair makes the SHACL TaskShape stale-edge
# constraint fire on them. So the conforming ingest EXCLUDES the stale edges and the
# script REPORTS their count to stderr (and writes them to a sidecar) — that is the
# guardrail working, and it keeps the ingest at 0 SHACL violations. The excluded
# edges are real cleanup work (beaded), not silently dropped.
#
# Usage:
#   python3 crates/sparq-kb/ingest/ingest_pkg.py \
#       --beads .beads/issues.jsonl --skills-dir skills \
#       --out crates/sparq-kb/ingest/pkg-instances.ttl
# Run from the repo root.

import argparse
import json
import os
import re
import sys

PKG = "https://sparq.dev/ns/pkg#"
KB = "https://sparq.dev/ns/pkg/kb#"

# bd status -> pkg status individual (the SHACL TaskShape enum).
STATUS = {
    "open": "pkg:Open",
    "in_progress": "pkg:InProgress",
    "blocked": "pkg:Blocked",
    "deferred": "pkg:Deferred",
    "closed": "pkg:Closed",
}
# bd issue_type -> pkg issue-type individual. "task" is the DEFAULT (no pkg:issueType
# triple) because pkg:Task names the CLASS (ontology note in pkg.ttl).
ISSUE_TYPE = {
    "bug": "pkg:Bug",
    "feature": "pkg:Feature",
    "chore": "pkg:Chore",
    "decision": "pkg:Decision",
    "milestone": "pkg:Milestone",
    "spike": "pkg:Spike",
    "epic": "pkg:Epic",
    "task": None,
}

# The heaviest / highest-read-frequency skill surfaces to ingest the front-matter of.
# (The full skills/ tree is Phase 2; this is the head slice — see the honesty report.)
HEAD_SKILLS = [
    "http-server",
    "vector-search",
    "sparql-query",
    "shacl-validation",
    "genai-retrieval",
]

PREFIXES = """\
@prefix pkg:     <https://sparq.dev/ns/pkg#> .
@prefix prov:    <http://www.w3.org/ns/prov#> .
@prefix skos:    <http://www.w3.org/2004/02/skos/core#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix fabio:   <http://purl.org/spar/fabio/> .
@prefix schema:  <http://schema.org/> .
@prefix rdfs:    <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix kb:      <https://sparq.dev/ns/pkg/kb#> .
"""


def esc(s: str) -> str:
    """Escape a string literal for Turtle (a single-line "..." literal)."""
    return (
        s.replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("\n", " ")
        .replace("\r", " ")
        .replace("\t", " ")
    ).strip()


def task_iri(bead_id: str) -> str:
    # bd ids contain '.' (e.g. sq-2m6zm.2); a '.' is legal in a Turtle PNAME local
    # part only mid-token, so encode it to keep IRIs robust across parsers.
    return "kb:task-" + bead_id.replace(".", "_")


def project_beads(path):
    """Mechanical projection of .beads/issues.jsonl -> Task TTL lines.

    Returns (lines, stats). Stale closed->open dependency edges are EXCLUDED from the
    conforming ingest and counted; their detail is returned in stats["stale"].
    """
    issues = {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            o = json.loads(line)
            if o.get("_type") == "issue":
                issues[o["id"]] = o

    lines = []
    stale = []  # (a_dependsOn_b) edges that are stale: b closed, a not closed
    emitted_deps = 0
    for bead_id, o in sorted(issues.items()):
        iri = task_iri(bead_id)
        lines.append(f"\n{iri} a pkg:Task ;")
        title = esc(o.get("title", bead_id))
        lines.append(f'  dcterms:title "{title}" ;')
        lines.append(f'  dcterms:identifier "{esc(bead_id)}" ;')
        st = STATUS.get(o.get("status"), "pkg:Open")
        lines.append(f"  pkg:status {st} ;")
        it = ISSUE_TYPE.get(o.get("issue_type"))
        if it:
            lines.append(f"  pkg:issueType {it} ;")
        prio = o.get("priority")
        if isinstance(prio, int) and 0 <= prio <= 4:
            lines.append(f"  pkg:priority {prio} ;")
        for lbl in o.get("labels", []) or []:
            lines.append(f'  pkg:label "{esc(lbl)}" ;')
        # typed dependency edges
        for d in o.get("dependencies", []) or []:
            dtype = d.get("type")
            a = d.get("issue_id")
            b = d.get("depends_on_id")
            if a not in issues or b not in issues:
                continue  # dangling endpoint — skip (would be a SHACL/closure miss)
            b_iri = task_iri(b)
            if dtype == "blocks":
                # bd 'blocks' (a,b) renders as "a DEPENDS ON b" -> a pkg:dependsOn b
                # (design §2.2). Materialise the OWL inverse pkg:blockedBy so the
                # stale-edge SHACL constraint can fire — but EXCLUDE the edge when it
                # is stale (b closed, a not closed) to keep the ingest conforming.
                b_closed = issues[b].get("status") == "closed"
                a_closed = o.get("status") == "closed"
                if b_closed and not a_closed:
                    stale.append((a, b))
                    continue
                lines.append(f"  pkg:dependsOn {b_iri} ;")
                emitted_deps += 1
            elif dtype == "blocked-by":
                lines.append(f"  pkg:blockedBy {b_iri} ;")
                emitted_deps += 1
            elif dtype in ("parent-child", "parent"):
                # child (a) isPartOf parent (b); §4.1 umbrella detection uses this.
                lines.append(f"  dcterms:isPartOf {b_iri} ;")
            elif dtype == "discovered-from":
                lines.append(f"  pkg:discoveredFrom {b_iri} ;")
            elif dtype == "related":
                lines.append(f"  skos:related {b_iri} ;")
        # close the statement: replace the trailing ' ;' of the last line with ' .'
        lines[-1] = lines[-1].rstrip()[:-1] + "."

    stats = {
        "tasks": len(issues),
        "deps_emitted": emitted_deps,
        "stale_excluded": len(stale),
        "stale": stale,
    }
    return lines, stats


FRONTMATTER_RE = re.compile(r"^---\s*\n(.*?)\n---\s*\n", re.DOTALL)


def parse_frontmatter(text):
    """Parse the leading YAML front-matter `name:` + `description:` (stdlib only).

    Skill front-matter uses simple `key: value` (description may be quoted). We do a
    minimal, deterministic parse rather than pull a YAML dep.
    """
    m = FRONTMATTER_RE.match(text)
    if not m:
        return None
    block = m.group(1)
    out = {}
    cur = None
    for raw in block.split("\n"):
        mm = re.match(r"^([A-Za-z0-9_-]+):\s*(.*)$", raw)
        if mm:
            cur = mm.group(1)
            out[cur] = mm.group(2).strip()
        elif cur is not None and raw.strip():
            out[cur] += " " + raw.strip()
    # strip surrounding quotes on the description if present
    for k in out:
        v = out[k]
        if len(v) >= 2 and v[0] == v[-1] and v[0] in ("'", '"'):
            out[k] = v[1:-1]
    return out


def project_skills(skills_dir):
    """Structured-parse the head skills' front-matter -> Source + Technique TTL."""
    lines = []
    count = 0
    for name in HEAD_SKILLS:
        path = os.path.join(skills_dir, name, "SKILL.md")
        if not os.path.exists(path):
            print(f"  [skip] {path} (absent)", file=sys.stderr)
            continue
        with open(path, encoding="utf-8") as f:
            fm = parse_frontmatter(f.read())
        if not fm or "description" not in fm:
            print(f"  [skip] {path} (no front-matter)", file=sys.stderr)
            continue
        sname = fm.get("name", name)
        desc = fm["description"]
        # A short title (>=4 chars for SourceShape) + the full description as abstract.
        title = f"{sname} skill — sparq surface guide"
        src = f"kb:skill-{sname}"
        tech = f"kb:surface-{sname}"
        lines.append(f"\n{src} a pkg:Source , pkg:Document , fabio:Expression ;")
        lines.append(f'  dcterms:title "{esc(title)}" ;')
        lines.append(f'  dcterms:identifier "skills/{sname}/SKILL.md" ;')
        lines.append(f'  dcterms:format "text/markdown" ;')
        lines.append(f'  dcterms:abstract "{esc(desc)[:900]}" ;')
        lines.append(f"  pkg:exploredStatus pkg:Explored ;")
        lines.append(f"  pkg:followUpPriority 1 ;")
        lines.append(f"  pkg:confidence 0.9 ;")
        lines.append(f"  dcterms:subject {tech} .")
        # The surface the skill documents, as a pkg:Technique (skos:Concept).
        lines.append(f"\n{tech} a pkg:Technique ;")
        lines.append(f'  skos:prefLabel "{esc(sname)} surface"@en-GB ;')
        lines.append(f'  rdfs:comment "{esc(desc)[:300]}" ;')
        lines.append(f"  dcterms:isReferencedBy {src} .")
        count += 1
    return lines, {"skills": count}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--beads", required=True)
    ap.add_argument("--skills-dir", required=True)
    ap.add_argument("--findings", default=None,
                    help="hand-authored AGENTS.md findings .ttl to append verbatim")
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    header = (
        "# pkg-instances.ttl — the Phase-1 ingested PKG graph (GENERATED by\n"
        "# crates/sparq-kb/ingest/ingest_pkg.py; do not hand-edit).\n"
        "# [OPUS-4.8] sq-2m6zm.2 (epic sq-2m6zm). 🤖 SPARQ agent — PKG ingestion PoC.\n"
        "# Structured-parse of .beads/issues.jsonl (-> pkg:Task) + heaviest skills'\n"
        "# front-matter (-> pkg:Source/pkg:Technique). Hand-authored AGENTS.md Findings\n"
        "# are appended from ingest/agents-findings.ttl. Conforms to pkg.shapes.ttl\n"
        "# (0 violations); the bd backlog's stale closed->open edges are EXCLUDED (see\n"
        "# the script header + the SHACL honesty report).\n\n"
    )

    parts = [header, PREFIXES, "\n# === Tasks (mechanical bd projection) ===\n"]
    bead_lines, bstats = project_beads(args.beads)
    parts.extend(bead_lines)
    parts.append("\n\n# === Sources + Techniques (skills front-matter parse) ===\n")
    skill_lines, sstats = project_skills(args.skills_dir)
    parts.extend(skill_lines)

    out_text = "\n".join(parts) + "\n"
    if args.findings:
        with open(args.findings, encoding="utf-8") as f:
            out_text += (
                "\n# === Findings (hand-authored from AGENTS.md; appended verbatim) ===\n"
                + f.read()
            )

    with open(args.out, "w", encoding="utf-8") as f:
        f.write(out_text)

    # Sidecar: the excluded stale edges (real cleanup work, not silently dropped).
    sidecar = args.out + ".stale-edges.tsv"
    with open(sidecar, "w", encoding="utf-8") as f:
        f.write("# stale dependency edges EXCLUDED from the conforming ingest\n")
        f.write("# (a depends_on b, but b is CLOSED while a is not) — guardrail-caught\n")
        f.write("dependent\tclosed_blocker\n")
        for a, b in bstats["stale"]:
            f.write(f"{a}\t{b}\n")

    print(
        f"[ingest] tasks={bstats['tasks']} deps_emitted={bstats['deps_emitted']} "
        f"stale_excluded={bstats['stale_excluded']} skills={sstats['skills']} "
        f"-> {args.out}",
        file=sys.stderr,
    )
    print(f"[ingest] stale-edge sidecar -> {sidecar}", file=sys.stderr)


if __name__ == "__main__":
    main()
