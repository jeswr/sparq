#!/usr/bin/env python3
# [OPUS-4.8] sq-eifd: unit tests for the bench-adapter parsers. Authored by Opus
# 4.8 (Fable unavailable; flag for re-review when Fable returns).
#
# FIXTURE-based parser tests that DO NOT need a live engine:
#   - shacl_report_count : parse a SHACL ValidationReport graph -> (conforms, viol)
#   - http_sparql_adapter.parse_sparql_json : SELECT / ASK / COUNT(*) reductions
#   - vector_lib_adapter.recall_at_k + parse_neighbour_tsv : recall scoring + deficit
# shacl_report_count's test needs rdflib (the SHACL report parser's only dep); if
# rdflib is absent that one test SKIPS (the http + vector tests are stdlib-only and
# always run). Run with the venv that has rdflib for full coverage:
#   /tmp/sq-eifd-venv/bin/python scripts/bench-adapters/test_adapters.py
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
FIX = os.path.join(HERE, "fixtures")

import http_sparql_adapter as http  # noqa: E402
import vector_lib_adapter as vec  # noqa: E402

PASS = 0
FAIL = 0


def check(name, got, want):
    global PASS, FAIL
    if got == want:
        PASS += 1
        print("ok   %-46s = %r" % (name, got))
    else:
        FAIL += 1
        print("FAIL %-46s got %r want %r" % (name, got, want))


def approx(name, got, want, tol=1e-6):
    check(name, abs(got - want) < tol, True)


# --- http-sparql SPARQL-JSON parser -----------------------------------------
def test_http():
    with open(os.path.join(FIX, "sparql_select.json")) as f:
        sel = http.parse_sparql_json(f.read())
    check("http.select.count", sel["count"], 3)
    check("http.select.boolean", sel["boolean"], None)
    # 2-var rows are not a COUNT(*) wrap -> no scalar extracted
    check("http.select.count_value", sel["count_value"], None)

    with open(os.path.join(FIX, "sparql_count.json")) as f:
        cnt = http.parse_sparql_json(f.read())
    check("http.count.count", cnt["count"], 1)
    check("http.count.count_value", cnt["count_value"], "3")

    with open(os.path.join(FIX, "sparql_ask.json")) as f:
        ask = http.parse_sparql_json(f.read())
    check("http.ask.count", ask["count"], 1)
    check("http.ask.boolean", ask["boolean"], True)

    try:
        http.parse_sparql_json('{"head":{}}')
        check("http.bad-doc-raises", False, True)
    except ValueError:
        check("http.bad-doc-raises", True, True)

    # [OPUS-4.8] A non-boolean "boolean" member (e.g. the string "false") is an
    # invalid SPARQL-JSON ASK result and must be rejected, not coerced via bool().
    try:
        http.parse_sparql_json('{"head":{},"boolean":"false"}')
        check("http.non-bool-boolean-raises", False, True)
    except ValueError:
        check("http.non-bool-boolean-raises", True, True)


# --- vector recall scoring ---------------------------------------------------
def test_vector():
    with open(os.path.join(FIX, "knn_approx.tsv")) as f:
        approx_map = vec.parse_neighbour_tsv(f.read())
    with open(os.path.join(FIX, "knn_exact.tsv")) as f:
        exact_map = vec.parse_neighbour_tsv(f.read())
    check("vec.parse.nqueries", len(approx_map), 3)
    recall, n = vec.recall_at_k(approx_map, exact_map, 4)
    check("vec.recall.nqueries", n, 3)
    approx("vec.recall@4", recall, 11.0 / 12.0)
    check("vec.recall_deficit_milli", vec.recall_deficit_milli(recall), 83)
    # perfect recall -> deficit 0
    perfect, _ = vec.recall_at_k(exact_map, exact_map, 4)
    check("vec.recall@4.perfect", vec.recall_deficit_milli(perfect), 0)
    # disjoint query ids must raise (catch a mis-aligned fixture loudly)
    try:
        vec.recall_at_k({"zz": ["1"]}, {"yy": ["1"]}, 4)
        check("vec.disjoint-raises", False, True)
    except ValueError:
        check("vec.disjoint-raises", True, True)


# --- shacl report-count parser (needs rdflib) --------------------------------
def test_shacl():
    try:
        import rdflib  # noqa: F401
    except ImportError:
        print("skip shacl_report_count tests (rdflib not installed in this python)")
        return
    import shacl_report_count as src

    # A minimal non-conforming report: 2 results + sh:conforms false.
    report = """
    @prefix sh: <http://www.w3.org/ns/shacl#> .
    [] a sh:ValidationReport ;
       sh:conforms false ;
       sh:result [ a sh:ValidationResult ;
                   sh:sourceConstraintComponent sh:MinCountConstraintComponent ] ;
       sh:result [ a sh:ValidationResult ;
                   sh:sourceConstraintComponent sh:DatatypeConstraintComponent ] .
    """
    r = src.count_report(report, fmt="turtle")
    check("shacl.report.violations", r["violations"], 2)
    check("shacl.report.conforms", r["conforms"], False)

    conforming = """
    @prefix sh: <http://www.w3.org/ns/shacl#> .
    [] a sh:ValidationReport ; sh:conforms true .
    """
    rc = src.count_report(conforming, fmt="turtle")
    check("shacl.report.conforming.violations", rc["violations"], 0)
    check("shacl.report.conforming.conforms", rc["conforms"], True)


def main():
    test_http()
    test_vector()
    test_shacl()
    print("\n%d passed, %d failed" % (PASS, FAIL))
    return 1 if FAIL else 0


if __name__ == "__main__":
    sys.exit(main())
