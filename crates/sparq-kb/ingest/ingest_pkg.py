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

# The deterministic YAML-LD authoring compiler (FO-bridge Phase 4, sq-mztg8.2). The
# write-path Findings tier is now AUTHORED in agents-findings.yaml.ld and compiled here
# (replacing the raw-Turtle agents-findings.ttl tier) — see yamlld_compile.py.
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from yamlld_compile import compile_yaml_ld  # noqa: E402

PKG = "https://sparq.dev/ns/pkg#"
KB = "https://sparq.dev/ns/pkg/kb#"
# [SONNET-4.6] sq-tzars.7: the HAND-AUTHORED tier named-graph IRI (byte-pinned in Rust as
# crates/sparq-kb/src/vocab.rs::TIER_HAND_AUTHORED_GRAPH). This script's output IS the
# hand-authored tier, so it stamps that graph IRI so a tier-aware loader/exporter
# (sq-tzars.8) can keep the hand-authored tier queryable-apart from the machine tier.
TIER_HAND_AUTHORED_GRAPH = "https://sparq.dev/ns/pkg/graph#hand-authored"

# [OPUS-4.8] sq-2m6zm.5 (bd-bridge eval). A `research/<doc>.md` reference inside a
# bead's `design` / `description` is the bd `--spec-id` style knowledge link. Project
# it as `pkg:Task dcterms:relation kb:doc-<doc>` + mint the research doc as a
# pkg:Source. This materialises the design's KILLER cross-link — the knowledge<->task
# JOIN that bd CANNOT express (bd has no model of the research corpus). The match is
# deterministic (a regex over the structured fields), not LLM-extracted.
RESEARCH_DOC_RE = re.compile(r"research/([a-z0-9][a-z0-9-]*\.md)")


def research_doc_iri(doc: str) -> str:
    # doc is e.g. "structure-aware-vectorisation.md"; strip the .md for the local part.
    stem = doc[:-3] if doc.endswith(".md") else doc
    return "kb:doc-research-" + stem.replace(".", "_")

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
    research_docs = {}  # doc -> count of beads linking it (drives the pkg:Source mint)
    spec_links = 0  # total pkg:Task -> research-doc edges emitted
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
        # The research-doc spec link (the knowledge<->task JOIN). A research/<doc>.md
        # named in the bead's structured design/description fields becomes a
        # dcterms:relation edge to the minted research-doc Source. Deterministic regex,
        # NOT LLM extraction.
        blob = (o.get("design") or "") + " " + (o.get("description") or "")
        docs = sorted(set(RESEARCH_DOC_RE.findall(blob)))
        for doc in docs:
            lines.append(f"  dcterms:relation {research_doc_iri(doc)} ;")
            research_docs[doc] = research_docs.get(doc, 0) + 1
            spec_links += 1
        # close the statement: replace the trailing ' ;' of the last line with ' .'
        lines[-1] = lines[-1].rstrip()[:-1] + "."

    # Mint each referenced research doc as a pkg:Source (so the cross-link query joins
    # tasks <-> a typed source node, not a bare IRI). SourceShape requires a title +
    # an explored-status, so emit both. confidence reflects that these are the project's
    # OWN design records (high reliability); explored-status defaults to Explored.
    if research_docs:
        lines.append("\n\n# === Research-doc Sources (bd spec-link targets; sq-2m6zm.5) ===\n")
    for doc in sorted(research_docs):
        diri = research_doc_iri(doc)
        title = f"research/{doc} — sparq design record"
        lines.append(f"\n{diri} a pkg:Source , pkg:Document , fabio:Expression ;")
        lines.append(f'  dcterms:title "{esc(title)}" ;')
        lines.append(f'  dcterms:identifier "research/{doc}" ;')
        lines.append(f'  dcterms:format "text/markdown" ;')
        lines.append(f"  pkg:exploredStatus pkg:Explored ;")
        lines.append(f"  pkg:followUpPriority 2 ;")
        lines.append(f"  pkg:confidence 0.9 .")

    stats = {
        "tasks": len(issues),
        "deps_emitted": emitted_deps,
        "stale_excluded": len(stale),
        "stale": stale,
        "research_docs": len(research_docs),
        "spec_links": spec_links,
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
                    help="LEGACY: a pre-rendered findings .ttl to append verbatim "
                         "(the raw-Turtle tier; superseded by --findings-yamlld)")
    ap.add_argument("--findings-yamlld", default=None,
                    help="the WRITE-PATH authoring source (a *.yaml.ld Findings doc, "
                         "FO-bridge Phase 4 sq-mztg8.2): deterministically COMPILED to "
                         "the schema.org-typed PKG Turtle and appended. An ambiguous "
                         "concept token fails the compile (reds the ingest).")
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    header = (
        "# pkg-instances.ttl — the Phase-1 ingested PKG graph (GENERATED by\n"
        "# crates/sparq-kb/ingest/ingest_pkg.py; do not hand-edit).\n"
        "# [OPUS-4.8] sq-2m6zm.2 (epic sq-2m6zm). 🤖 SPARQ agent — PKG ingestion PoC.\n"
        "# Structured-parse of .beads/issues.jsonl (-> pkg:Task) + heaviest skills'\n"
        "# front-matter (-> pkg:Source/pkg:Technique). The AGENTS.md Findings tier is\n"
        "# AUTHORED in ingest/agents-findings.yaml.ld and deterministically COMPILED by\n"
        "# yamlld_compile.py (FO-bridge Phase 4, sq-mztg8.2). Conforms to pkg.shapes.ttl\n"
        "# (0 violations); the bd backlog's stale closed->open edges are EXCLUDED (see\n"
        "# the script header + the SHACL honesty report).\n\n"
    )

    # [SONNET-4.6] sq-tzars.7: stamp this output as the HAND-AUTHORED tier. The stamp is a
    # single untyped node (no rdf:type), so it is not targeted by any SHACL shape and does
    # not affect the 0-violation conformance; it names the tier so downstream tooling can
    # keep the tiers queryable-apart (per-tier ARTIFACTS, not in-store named graphs).
    tier_stamp = (
        "\n# === Tier stamp (sq-tzars.7): this graph IS the hand-authored tier ===\n"
        f"<{TIER_HAND_AUTHORED_GRAPH}> rdfs:label "
        '"hand-authored tier -- the human-curated PKG (ingest_pkg.py projector output)"@en .\n'
    )
    parts = [header, PREFIXES, tier_stamp, "\n# === Tasks (mechanical bd projection) ===\n"]
    bead_lines, bstats = project_beads(args.beads)
    parts.extend(bead_lines)
    parts.append("\n\n# === Sources + Techniques (skills front-matter parse) ===\n")
    skill_lines, sstats = project_skills(args.skills_dir)
    parts.extend(skill_lines)

    out_text = "\n".join(parts) + "\n"
    findings_n = 0
    # WRITE-PATH (preferred): compile the YAML-LD authoring source deterministically.
    # An ambiguous/unresolvable concept token raises CompileError -> the ingest reds,
    # which is the fail-closed contract (never a silent wrong IRI). The compiled tier
    # carries its OWN @prefix header (it is a standalone Turtle document), and the PKG
    # loader parses documents independently, so prefix scope does not leak.
    if args.findings_yamlld:
        with open(args.findings_yamlld, encoding="utf-8") as f:
            compiled = compile_yaml_ld(f.read())
        out_text += (
            "\n# === Findings (COMPILED from agents-findings.yaml.ld; "
            "FO-bridge Phase 4) ===\n" + compiled
        )
        findings_n = compiled.count("a pkg:Finding")
    # LEGACY: a pre-rendered findings .ttl appended verbatim (the old raw-Turtle tier).
    elif args.findings:
        with open(args.findings, encoding="utf-8") as f:
            blob = f.read()
            out_text += (
                "\n# === Findings (pre-rendered .ttl; appended verbatim) ===\n" + blob
            )
            findings_n = blob.count("a pkg:Finding")

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
        f"stale_excluded={bstats['stale_excluded']} "
        f"research_docs={bstats['research_docs']} spec_links={bstats['spec_links']} "
        f"skills={sstats['skills']} findings={findings_n} -> {args.out}",
        file=sys.stderr,
    )
    print(f"[ingest] stale-edge sidecar -> {sidecar}", file=sys.stderr)


if __name__ == "__main__":
    main()
