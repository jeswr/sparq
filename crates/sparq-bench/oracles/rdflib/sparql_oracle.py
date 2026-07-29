#!/usr/bin/env python3
# [SONNET-4.6] sq-qcnn.8 — the OPTIONAL rdflib subprocess oracle for the sparq differential fuzzer.
#
# Node F of research/differential-testing-value-level.md. rdflib is trivially provisioned (no JVM,
# no jar) but its SPARQL implementation is LESS CONFORMANT than Jena's — known aggregate and
# function gaps — so it produces more spurious divergences and therefore more triage. It is the
# optional third oracle, not the primary second one; prefer ../jena.
#
# Wire protocol (see crates/sparq-bench/src/oracle.rs, `SubprocessOracle`):
#
#   argv[1] = path to a Turtle data file
#   argv[2] = path to a SPARQL query file
#
#   exit 0 — stdout is a SPARQL-Results-JSON document (SELECT bindings or an ASK boolean)
#   exit 3 — "I cannot evaluate this query" (parse error, unimplemented feature, or a
#            CONSTRUCT/DESCRIBE, which SPARQL Results JSON cannot carry). A SKIP, not a divergence.
#   exit 2 — bad invocation. Any other non-zero exit is a backend failure.
#
# Only the JSON document goes to stdout; diagnostics go to stderr. The adapter treats unparseable
# stdout from a zero exit as a BROKEN oracle, so a stray print here would be reported as a fault
# rather than silently ignored.
#
# Requires: pip install rdflib

import sys

UNSUPPORTED = 3


def main(argv):
    if len(argv) != 3:
        sys.stderr.write("usage: sparql_oracle.py <data.ttl> <query.rq>\n")
        return 2

    try:
        from rdflib import Graph
    except ImportError as e:  # not provisioned — a backend fault, not a decline
        sys.stderr.write("backend: rdflib is not installed ({})\n".format(e))
        return 1

    data_path, query_path = argv[1], argv[2]

    # Parsing the data is a BACKEND concern: all engines are handed identical bytes, so one of
    # them rejecting them is a finding, not a reason to quietly skip the case.
    try:
        graph = Graph()
        graph.parse(data_path, format="turtle")
        with open(query_path, encoding="utf-8") as fh:
            query_text = fh.read()
    except Exception as e:
        sys.stderr.write("data: {}\n".format(e))
        return 1

    try:
        result = graph.query(query_text)
        if result.type not in ("SELECT", "ASK"):
            # CONSTRUCT / DESCRIBE have no SPARQL-Results-JSON form.
            sys.stderr.write("unsupported result form: {}\n".format(result.type))
            return UNSUPPORTED
        payload = result.serialize(format="json")
    except Exception as e:
        # rdflib raises a single family for "won't parse" and "can't evaluate"; both are the
        # decline bucket, which is why the harness must never attribute a skip to sparq.
        sys.stderr.write("unsupported: {}\n".format(e))
        return UNSUPPORTED

    if isinstance(payload, bytes):
        sys.stdout.buffer.write(payload)
    else:
        sys.stdout.write(payload)
    sys.stdout.flush()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
