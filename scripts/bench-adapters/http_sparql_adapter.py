#!/usr/bin/env python3
# [OPUS-4.8] sq-eifd: HTTP-SPARQL adapter KIND. Authored by Opus 4.8 (Fable
# unavailable; flag for re-review when Fable returns).
#
# POST a SPARQL query to an HTTP endpoint -> parse the SPARQL 1.1 Query Results
# JSON -> count solutions / read the ASK boolean / extract a COUNT(*) value.
# Shared across Fuseki, Virtuoso (the core-SPARQL reference engines from
# research/BENCHMARKS.md) AND the existing QLever HTTP path (this is the reusable
# unit the per-dir bench/qlever-*/compare.py hand-rolls today; G3 in
# research/capability-benchmark-program.md §2).
#
# The PARSER (parse_sparql_json) is the value here and is unit-tested against a
# captured sample response WITHOUT a live server. The transport (do_query) is a
# thin urllib POST; standing up Fuseki/Virtuoso is gather-only on a Docker box.
#
# SPARQL-JSON shapes handled (per the W3C rec):
#   - SELECT : {"head":{"vars":[...]}, "results":{"bindings":[ {var:{...}}, ...]}}
#              -> solution count = len(bindings); a single-binding single-var row
#                 is treated as a COUNT(*) and its value extracted (the
#                 correctness cross-check QLever's compare.py does today).
#   - ASK    : {"head":{}, "boolean": true|false} -> count = 1 if true else 0,
#              boolean carried through.
#
# Output: `<engine>\t<count>\t<query_us>` TSV on stdout (the same 3-col contract
# the rest of the adapters + the ci-bench hook use). Non-zero exit only on a
# transport/parse ERROR.
import json
import sys
import time
import urllib.parse
import urllib.request


def parse_sparql_json(text):
    """Reduce a SPARQL-results-JSON document to a comparable count.

    Returns {"count": int, "boolean": bool|None, "count_value": str|None}:
      count       : len(bindings) for SELECT, or 1/0 for ASK true/false.
      boolean     : the ASK boolean (None for SELECT).
      count_value : the scalar of a single-row single-var SELECT (a COUNT(*) wrap),
                    else None — the value to cross-check against sparq's count.
    Raises ValueError on a document that is neither a SELECT nor an ASK result."""
    obj = text if isinstance(text, dict) else json.loads(text)
    if "boolean" in obj:
        b = bool(obj["boolean"])
        return {"count": 1 if b else 0, "boolean": b, "count_value": None}
    results = obj.get("results")
    if results is None or "bindings" not in results:
        raise ValueError("not a SPARQL SELECT/ASK results document")
    bindings = results["bindings"]
    count_value = None
    if len(bindings) == 1 and len(bindings[0]) == 1:
        only = next(iter(bindings[0].values()))
        count_value = only.get("value")
    return {"count": len(bindings), "boolean": None, "count_value": count_value}


def do_query(endpoint, query, accept="application/sparql-results+json", timeout=300):
    """POST a SPARQL query (application/x-www-form-urlencoded) -> response bytes."""
    data = urllib.parse.urlencode({"query": query}).encode()
    req = urllib.request.Request(
        endpoint, data=data, headers={"Accept": accept}
    )
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return r.read().decode("utf-8")


def run_http_sparql(endpoint, query, engine="engine", iters=1, timeout=300):
    """Run a query `iters` times (min wall-clock), parse the JSON result.

    Returns {"engine","count","boolean","count_value","query_us"}.
    Raises on transport/parse error."""
    best_us = None
    parsed = None
    for _ in range(max(1, iters)):
        t0 = time.perf_counter()
        body = do_query(endpoint, query, timeout=timeout)
        dt_us = (time.perf_counter() - t0) * 1e6
        parsed = parse_sparql_json(body)
        if best_us is None or dt_us < best_us:
            best_us = dt_us
    return {
        "engine": engine,
        "count": parsed["count"],
        "boolean": parsed["boolean"],
        "count_value": parsed["count_value"],
        "query_us": int(round(best_us)),
    }


def main(argv):
    """CLI: http_sparql_adapter.py --endpoint URL (--query Q | --query-file F)
            [--engine NAME] [--iters N] [--json]
       OR : --parse-only <json-file>   (offline: just run the parser, for tests)"""
    endpoint = query = None
    engine = "engine"
    iters = 1
    want_json = False
    parse_only = None
    i = 0
    while i < len(argv):
        a = argv[i]
        if a == "--endpoint":
            i += 1
            endpoint = argv[i]
        elif a == "--query":
            i += 1
            query = argv[i]
        elif a == "--query-file":
            i += 1
            with open(argv[i], "r", encoding="utf-8") as fh:
                query = fh.read()
        elif a == "--engine":
            i += 1
            engine = argv[i]
        elif a == "--iters":
            i += 1
            iters = int(argv[i])
        elif a == "--parse-only":
            i += 1
            parse_only = argv[i]
        elif a == "--json":
            want_json = True
        else:
            sys.stderr.write("http_sparql_adapter: unknown arg %s\n" % a)
            return 2
        i += 1

    if parse_only is not None:
        with open(parse_only, "r", encoding="utf-8") as fh:
            res = parse_sparql_json(fh.read())
        json.dump(res, sys.stdout)
        sys.stdout.write("\n")
        return 0

    if not (endpoint and query):
        sys.stderr.write(
            "usage: http_sparql_adapter.py --endpoint URL (--query Q|--query-file F) "
            "[--engine NAME] [--iters N] [--json]   |   --parse-only <json-file>\n"
        )
        return 2
    try:
        res = run_http_sparql(endpoint, query, engine=engine, iters=iters)
    except Exception as e:  # noqa: BLE001 — adapter boundary
        sys.stderr.write("http_sparql_adapter: %s\n" % e)
        return 1
    sys.stdout.write("%s\t%d\t%d\n" % (res["engine"], res["count"], res["query_us"]))
    if want_json:
        sys.stderr.write(json.dumps(res) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
