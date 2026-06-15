#!/usr/bin/env python3
# [OPUS-4.8] sq-55c1: pySHACL reference-engine adapter for the SHACL differential
# fuzzer (crates/sparq-shacl/tests/diff_fuzz.rs).
#
# This is a "report-cli" style adapter in the sense of bead sq-eifd: file-in /
# normalised-report-out, so its output is directly comparable to sparq-shacl's.
# When sq-eifd lands the shared report-cli harness, this script's normalisation
# (sh:conforms + the {focusNode, sourceConstraintComponent, resultPath} violation
# set) is the contract the runner depends on — point sq-eifd's adapter at the same
# JSON shape and the Rust runner is unchanged.
#
# CONTRACT (stdin -> stdout):
#   stdin  : a JSON object {"data": "<turtle>", "shapes": "<turtle>"}
#   stdout : a JSON object
#              {"conforms": bool,
#               "violations": [{"focus": str, "component": str, "path": str|null}, ...]}
#            — `focus`/`component` are full IRIs (or "_:bnode" for blank nodes, which
#            have no cross-graph identity); `path` is the sh:resultPath rendered as a
#            best-effort string (predicate IRI for the common simple-path case, else a
#            stable label) or null when the result carries no path.
#   exit   : 0 on a produced report (even non-conforming); non-zero only on an
#            adapter/engine ERROR (so the runner can distinguish "engine says X" from
#            "engine could not run"), with a diagnostic on stderr.
#
# Reads ONE request and emits ONE report (the runner spawns the interpreter once
# per case for isolation; a future optimisation can switch to a persistent
# line-protocol server, tracked as a TODO bead, but per-case spawn keeps the
# adapter trivially correct).
import json
import sys

SH = "http://www.w3.org/ns/shacl#"


def _term_key(term):
    """A graph-independent key for a term: blank nodes collapse to '_:bnode'
    (no stable cross-graph identity — the same tolerance the W3C runner uses)."""
    import rdflib

    if isinstance(term, rdflib.BNode):
        return "_:bnode"
    if isinstance(term, rdflib.URIRef):
        return str(term)
    if isinstance(term, rdflib.Literal):
        # Literals only appear as sh:value, not in the comparison key, but render
        # them stably anyway for diagnostics.
        return term.n3()
    return str(term)


def _path_key(report_graph, path_term):
    """Render an sh:resultPath as a comparable string. Simple predicate paths
    (a URIRef) are the overwhelming majority and render to the bare IRI — the
    same string sparq emits for `Path::Predicate`. Complex paths (sequence /
    inverse / alternative / *) are blank-node-rooted; we render a structural
    label that is stable per-shape but we do NOT try to byte-match sparq's
    Turtle path serialisation (impl-specific), so the runner treats a present
    complex path as "has a path" and compares the (focus, component) pair."""
    import rdflib

    if path_term is None:
        return None
    if isinstance(path_term, rdflib.URIRef):
        return str(path_term)
    # Complex / blank-rooted path: a sentinel meaning "non-simple path present".
    return "_:path"


def run(data_ttl, shapes_ttl):
    import rdflib
    import pyshacl

    data_g = rdflib.Graph()
    data_g.parse(data=data_ttl, format="turtle")
    shapes_g = rdflib.Graph()
    shapes_g.parse(data=shapes_ttl, format="turtle")

    # advanced=True turns on SHACL-AF (sh:node, logical components beyond the
    # core minimum that pySHACL treats as "advanced"); inference is left OFF so
    # we compare PURE validation, not validation-after-RDFS (sparq-shacl does no
    # inference either). meta_shacl off — we trust our generated shapes graphs.
    conforms, report_graph, _text = pyshacl.validate(
        data_g,
        shacl_graph=shapes_g,
        advanced=True,
        inference="none",
        meta_shacl=False,
        debug=False,
    )

    violations = []
    result_t = rdflib.URIRef(SH + "ValidationResult")
    rdf_type = rdflib.RDF.type
    p_focus = rdflib.URIRef(SH + "focusNode")
    p_comp = rdflib.URIRef(SH + "sourceConstraintComponent")
    p_path = rdflib.URIRef(SH + "resultPath")
    for res in report_graph.subjects(rdf_type, result_t):
        focus = report_graph.value(res, p_focus)
        comp = report_graph.value(res, p_comp)
        path = report_graph.value(res, p_path)
        violations.append(
            {
                "focus": _term_key(focus) if focus is not None else None,
                "component": _term_key(comp) if comp is not None else None,
                "path": _path_key(report_graph, path),
            }
        )
    return {"conforms": bool(conforms), "violations": violations}


def main():
    try:
        req = json.load(sys.stdin)
    except Exception as e:  # noqa: BLE001 — adapter boundary, report and exit
        sys.stderr.write("pyshacl_adapter: bad request JSON: %s\n" % e)
        return 2
    try:
        out = run(req["data"], req["shapes"])
    except Exception as e:  # noqa: BLE001
        sys.stderr.write("pyshacl_adapter: engine error: %s\n" % e)
        return 1
    json.dump(out, sys.stdout)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
