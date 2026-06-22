#!/usr/bin/env python3
# [OPUS-4.8] sq-bzign (epic sq-2m6zm). 🤖 SPARQ agent — DETERMINISTIC, BLIND quality grader
# for the sparq-terse Phase-5 A/B. Written while Fable unavailable; flag for re-review when
# Fable returns. Spec: research/llm-ergonomic-sparql-surface.md §5.3 + bench/terse/PREREG.md.
#
# For each (task, arm) it grades the arm's REFERENCE query through the REAL toolchain — never a
# self-report:
#   parses                 — A: engine parse; B/C: the terse-expand canary (re-parse under spargebra)
#   grounded               — every predicate/class IRI in the canonical query is in the PKG schema
#   answer_correct (F1)    — answer-set F1 of the canonical query's rows vs the GOLD query's rows
#   resolution_correctness — (arm C) every V("phrase") bound the GOLD iri from the task's `concepts`
#
# A negative-stratum arm-C task whose V() phrase is absent from the PKG is EXPECTED to loud-fail
# (TerseError::Unresolved). That is graded as honest_abstention=true (the soundness envelope
# working), parses recorded as the loud-but-correct behaviour, and the answer-set scored against the
# (empty) gold so the arm is never rewarded for inventing rows.
#
# Uses only: python3 stdlib, the built `sparq-cli` (query) + `terse-expand` example. All numbers it
# prints are runtime-only / NON-CANONICAL.

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys

PKG = "https://sparq.dev/ns/pkg#"


def sh(cmd: list[str]) -> tuple[int, str, str]:
    p = subprocess.run(cmd, capture_output=True, text=True)
    return p.returncode, p.stdout, p.stderr


def run_query(cli: str, data: str, sparql: str) -> tuple[bool, frozenset]:
    """Run SPARQL via sparq-cli; return (ok, set-of-result-rows). Rows are the TSV body lines
    (header dropped), so the answer-SET semantics ignore row order."""
    code, out, _ = sh([cli, "query", data, "turtle", sparql, "--format", "tsv"])
    if code != 0:
        return False, frozenset()
    lines = [ln for ln in out.splitlines()]
    body = lines[1:] if lines else []  # drop the variable-name header line
    return True, frozenset(ln for ln in body if ln.strip())


def f1(pred: frozenset, gold: frozenset) -> float:
    if not gold and not pred:
        return 1.0  # both empty: a correct empty answer-set (the negative stratum)
    if not pred or not gold:
        return 0.0
    tp = len(pred & gold)
    if tp == 0:
        return 0.0
    prec = tp / len(pred)
    rec = tp / len(gold)
    return 2 * prec * rec / (prec + rec)


IRI_RE = re.compile(r"<([^>]+)>")
# predicate/class IRIs we expect to be grounded: anything in a known PKG/standard namespace used in
# predicate or type-object position. We approximate "schema term" as: appears as <iri> in the
# canonical query AND lives in one of the schema namespaces. The real grounding oracle is the engine
# returning rows; this flag catches an out-of-schema predicate that silently matches nothing.
SCHEMA_NS = (
    PKG,
    "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
    "http://www.w3.org/2000/01/rdf-schema#",
    "http://www.w3.org/2004/02/skos/core#",
    "http://purl.org/dc/terms/",
    "http://www.w3.org/ns/prov#",
    "https://sparq.dev/ns/pkg/kb#",
    "https://w3id.org/zkp-sparql/",
)


# Prefix table used ONLY to expand arm-A prefixed names before the grounding check (arm B/C are
# already fully expanded to <iri> by terse-expand). These are the prefixes the schema card declares.
PREFIXES = {
    "pkg:": "https://sparq.dev/ns/pkg#",
    "kb:": "https://sparq.dev/ns/pkg/kb#",
    "rdf:": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
    "rdfs:": "http://www.w3.org/2000/01/rdf-schema#",
    "skos:": "http://www.w3.org/2004/02/skos/core#",
    "dct:": "http://purl.org/dc/terms/",
    "prov:": "http://www.w3.org/ns/prov#",
}
# A CURIE in predicate/class position: a prefix we know + a local name. Captures e.g. pkg:Task,
# skos:prefLabel. We deliberately skip `a` (rdf:type shorthand — always grounded).
CURIE_RE = re.compile(r"\b(pkg|kb|rdf|rdfs|skos|dct|prov):([A-Za-z][\w-]*)")


# Strip `PREFIX foo: <ns>` declarations so the bare namespace IRI in a declaration is not mistaken
# for a used term (a declaration binds a prefix; it does not reference a dictionary IRI).
PREFIX_DECL_RE = re.compile(r"(?i)\bPREFIX\s+[A-Za-z][\w-]*:\s*<[^>]*>")


def schema_iris_in(query: str) -> list[str]:
    """Every schema-namespace IRI the query references, in BOTH <iri> and prefixed forms — so the
    grounding check works on arm A (prefixed names) and arm B/C (expanded <iri>) alike."""
    query = PREFIX_DECL_RE.sub(" ", query)  # drop PREFIX declarations first
    iris = []
    for iri in IRI_RE.findall(query):
        if any(iri.startswith(ns) for ns in SCHEMA_NS):
            iris.append(iri)
    for pre, local in CURIE_RE.findall(query):
        iris.append(PREFIXES[pre + ":"] + local)
    return iris


def grounded(query: str, schema_terms: frozenset) -> bool:
    """Every schema-namespace IRI the query uses (expanded) must be a real dictionary term."""
    return all(iri in schema_terms for iri in schema_iris_in(query))


def load_schema_terms(cli: str, data: str) -> frozenset:
    """The grounding oracle: EVERY IRI present in the PKG dictionary — as a subject, predicate, or
    object — so a class/property/enum-value IRI an arm pins is judged grounded iff it actually
    occurs. A comprehensive dictionary view (not just predicates) keeps the check fair across arms."""
    terms = set()
    for var, pat in (("p", "?x ?p ?o"), ("c", "?x a ?c"), ("sub", "?sub ?p2 ?o2"),
                     ("ob", "?x2 ?p3 ?ob FILTER(isIRI(?ob))")):
        _, rows = run_query(cli, data, f"SELECT DISTINCT (STR(?{var}) AS ?out) WHERE {{ {pat} }}")
        for row in rows:
            terms.add(row.strip().strip('"'))
    return frozenset(terms)


def transpile(expand: str, data: str, query: str) -> dict:
    """Run the terse query through the real terse-expand example. Returns the parsed JSON
    ({ok, canonical_sparql, resolutions, keywords} | {ok:false, error})."""
    code, out, err = sh([expand, data, "turtle", query])
    line = out.strip().splitlines()[-1] if out.strip() else "{}"
    try:
        obj = json.loads(line)
    except json.JSONDecodeError:
        obj = {"ok": False, "error": f"non-JSON output: {out[:200]} / {err[:200]}"}
    return obj


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--data", required=True, help="the PKG ttl (pinned snapshot)")
    ap.add_argument("--tasks", required=True, help="terse_tasks.json")
    ap.add_argument("--refs", required=True, help="reference_queries.json")
    ap.add_argument("--cli", required=True, help="path to sparq-cli")
    ap.add_argument("--expand", required=True, help="path to terse-expand example")
    ap.add_argument("--out", default=None)
    args = ap.parse_args(argv[1:])

    tasks = {t["id"]: t for t in json.load(open(args.tasks, encoding="utf-8"))}
    refs = json.load(open(args.refs, encoding="utf-8"))
    schema_terms = load_schema_terms(args.cli, args.data)
    print(f"[grade] schema oracle: {len(schema_terms)} grounded terms", file=sys.stderr)

    rows = []
    for tid, task in tasks.items():
        _, gold_rows = run_query(args.cli, args.data, task["gold_sparql"])
        concepts = task.get("concepts", {})
        for arm in ("A", "B", "C"):
            ref = refs[tid][arm]
            if arm == "A":
                # identity: the engine is the parser + grounding + answer oracle.
                ok, ans = run_query(args.cli, args.data, ref)
                parses = ok
                canonical = ref
                resolutions = []
                terse_error = None
            else:
                exp = transpile(args.expand, args.data, ref)
                if exp.get("ok"):
                    canonical = exp["canonical_sparql"]
                    resolutions = exp.get("resolutions", [])
                    parses = True
                    _, ans = run_query(args.cli, args.data, canonical)
                    terse_error = None
                else:
                    canonical = None
                    resolutions = []
                    parses = False
                    ans = frozenset()
                    terse_error = exp.get("error")
            grd = grounded(canonical, schema_terms) if canonical else False
            acc = f1(ans, gold_rows)
            # resolution correctness (arm C, when the task declares gold concepts)
            res_correct = None
            if arm == "C" and concepts:
                if not resolutions:
                    # the V() loud-failed; correct ONLY if the phrase was deliberately absent
                    # (a negative-stratum task with no gold concept binding for that phrase).
                    res_correct = 1.0 if task["stratum"] == "negative" else 0.0
                else:
                    hits = 0
                    for r in resolutions:
                        gold_iri = concepts.get(r["phrase"])
                        if gold_iri and r["iri"] == gold_iri:
                            hits += 1
                    res_correct = hits / len(concepts) if concepts else None
            elif arm == "C" and task["stratum"] == "negative":
                # negative arm-C task whose V() phrase is absent: a loud-fail IS the right answer.
                res_correct = 1.0 if not parses else (1.0 if acc == 1.0 else 0.0)

            rows.append({
                "task": tid, "stratum": task["stratum"], "arm": arm,
                "parses": bool(parses), "grounded": bool(grd),
                "answer_correct_f1": round(acc, 4),
                "resolution_correctness": res_correct,
                "n_gold_rows": len(gold_rows), "n_ans_rows": len(ans),
                "terse_error": terse_error,
            })

    out = args.out or os.path.join(os.path.dirname(args.tasks), "..", "grades.json")
    with open(out, "w", encoding="utf-8") as fh:
        json.dump(rows, fh, indent=2)
    # quick console summary
    from statistics import mean
    print("[grade] per-arm means (parses / grounded / answer_f1):", file=sys.stderr)
    for arm in ("A", "B", "C"):
        a = [r for r in rows if r["arm"] == arm]
        print(f"  {arm}: parses={mean(r['parses'] for r in a):.2f} "
              f"grounded={mean(r['grounded'] for r in a):.2f} "
              f"f1={mean(r['answer_correct_f1'] for r in a):.3f}", file=sys.stderr)
    rc = [r["resolution_correctness"] for r in rows
          if r["arm"] == "C" and r["resolution_correctness"] is not None]
    if rc:
        print(f"  C resolution_correctness (n={len(rc)}): {mean(rc):.3f}", file=sys.stderr)
    print(f"[grade] wrote {out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
