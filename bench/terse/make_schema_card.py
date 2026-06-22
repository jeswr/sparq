#!/usr/bin/env python3
# [OPUS-4.8] sq-bzign (epic sq-2m6zm). 🤖 SPARQ agent — builds the arm-A SCHEMA CARD: the PKG
# prefixes + the class & property IRIs a plain-SPARQL agent must carry to author a grounded query
# (the apples-to-apples counterpart of the arm-B/C legend card). Generated from the LIVE PKG so it
# lists exactly the vocabulary in use — no more, no less. Stdlib + sparq-cli only; NON-CANONICAL.

from __future__ import annotations

import argparse
import re
import subprocess
import sys

PREFIXES = {
    "pkg": "https://sparq.dev/ns/pkg#",
    "kb": "https://sparq.dev/ns/pkg/kb#",
    "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
    "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
    "skos": "http://www.w3.org/2004/02/skos/core#",
    "dct": "http://purl.org/dc/terms/",
    "prov": "http://www.w3.org/ns/prov#",
}


def q(cli: str, data: str, sparql: str) -> list[str]:
    p = subprocess.run([cli, "query", data, "turtle", sparql, "--format", "tsv"],
                       capture_output=True, text=True)
    lines = p.stdout.splitlines()
    return [ln.strip().strip('"') for ln in lines[1:] if ln.strip()]


def shorten(iri: str) -> str:
    for pre, ns in PREFIXES.items():
        if iri.startswith(ns):
            return f"{pre}:{iri[len(ns):]}"
    return f"<{iri}>"


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--data", required=True)
    ap.add_argument("--cli", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args(argv[1:])

    classes = sorted(set(q(args.cli, args.data, "SELECT DISTINCT (STR(?c) AS ?s) WHERE { ?x a ?c }")))
    props = sorted(set(q(args.cli, args.data, "SELECT DISTINCT (STR(?p) AS ?s) WHERE { ?x ?p ?o }")))
    # the pkg: enum object values an agent pins (status/issueType/exploredStatus values)
    enums = sorted(set(q(args.cli, args.data,
        "PREFIX pkg: <https://sparq.dev/ns/pkg#> SELECT DISTINCT (STR(?o) AS ?s) WHERE "
        "{ ?x ?p ?o FILTER(isIRI(?o) && STRSTARTS(STR(?o), STR(pkg:))) }")))

    lines = ["# PKG schema card (plain-SPARQL, arm A) — prefixes + classes + properties + enum values",
             "# usage: write standard SPARQL with these PREFIX lines and IRIs."]
    lines.append("")
    for pre, ns in PREFIXES.items():
        lines.append(f"PREFIX {pre}: <{ns}>")
    lines.append("")
    lines.append("# classes:")
    for c in classes:
        lines.append(f"  {shorten(c)}")
    lines.append("# properties:")
    for p in props:
        lines.append(f"  {shorten(p)}")
    lines.append("# pkg: enum values (status / issueType / exploredStatus / assurance):")
    for e in enums:
        lines.append(f"  {shorten(e)}")
    card = "\n".join(lines) + "\n"
    with open(args.out, "w", encoding="utf-8") as fh:
        fh.write(card)
    print(f"[schema-card] wrote {args.out}: {len(classes)} classes, {len(props)} properties, "
          f"{len(enums)} enum values ({len(card)} chars)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
