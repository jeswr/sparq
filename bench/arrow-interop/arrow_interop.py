#!/usr/bin/env python3
"""[GPT-5.6] sq-vwnzh: correctness-gated Arrow/pyoxigraph interop panel."""
import argparse
import collections
import json
import os
import platform
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
DATA = os.path.join(HERE, "fixture.ttl")
QUERIES = ("select-all.rq", "select-group.rq")
XSD_STRING = "http://www.w3.org/2001/XMLSchema#string"


def normalize_cell(cell):
    """Make RDF 1.1 simple literals and explicit xsd:string identical."""
    if cell.startswith('"') and '"@' not in cell and '"^^<' not in cell:
        return cell + "^^<" + XSD_STRING + ">"
    return cell


def multiset(lines):
    return collections.Counter(
        tuple(normalize_cell(cell) for cell in line.rstrip("\n").split("\t"))
        for line in lines if line.rstrip("\n")
    )


def sparq_rows(query_path, binary=None):
    command = ([binary] if binary else [
        "cargo", "run", "--quiet", "-p", "sparq-arrow", "--features", "arrow",
        "--example", "arrow_interop", "--",
    ]) + [DATA, query_path]
    result = subprocess.run(command, cwd=ROOT, check=True, capture_output=True, text=True)
    return result.stdout.splitlines()


def pyoxigraph_rows(query_path):
    import pyoxigraph

    store = pyoxigraph.Store()
    store.load(path=DATA, format=pyoxigraph.RdfFormat.TURTLE)
    with open(query_path, encoding="utf-8") as handle:
        solutions = store.query(handle.read())
    return ["\t".join("" if value is None else str(value) for value in row) for row in solutions]


def expected():
    result = {}
    with open(os.path.join(HERE, "expected.tsv"), encoding="utf-8") as handle:
        for line in handle:
            query, *cells = line.rstrip("\n").split("\t")
            result.setdefault(query, collections.Counter())[tuple(cells)] += 1
    return result


def compare(exports, oracle):
    for query_name, engines in exports.items():
        for engine, rows in engines.items():
            got = multiset(rows)
            if got != oracle[query_name]:
                raise ValueError(f"{query_name}: {engine} result multiset differs from oracle")
        if len(set(map(frozenset, (multiset(rows).items() for rows in engines.values())))) > 1:
            raise ValueError(f"{query_name}: engine result multisets differ")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--iterations", type=int, default=5)
    parser.add_argument("--sparq-bin", help="prebuilt arrow_interop example")
    parser.add_argument("--json-out")
    args = parser.parse_args()

    engines = {"sparq-arrow": lambda q: sparq_rows(q, args.sparq_bin)}
    try:
        import pyoxigraph
        engines["pyoxigraph"] = pyoxigraph_rows
        pyoxigraph_version = pyoxigraph.__version__
    except ImportError:
        pyoxigraph_version = None  # absent tool => absent column

    exports = {}
    for query_name in QUERIES:
        query_path = os.path.join(HERE, query_name)
        exports[query_name] = {name: run(query_path) for name, run in engines.items()}
    compare(exports, expected())  # INVARIANT: equality completes before any stopwatch.

    rows = []
    for query_name in QUERIES:
        query_path = os.path.join(HERE, query_name)
        for engine, run in engines.items():
            samples = []
            for _ in range(args.iterations):
                started = time.perf_counter_ns()
                run(query_path)
                samples.append(time.perf_counter_ns() - started)
            rows.append({"query": query_name, "engine": engine,
                         "min_ns": min(samples), "iterations": args.iterations})

    envelope = {
        "verdict": "RESULT-SET-EQUAL",
        "comparison": "loose/in-process",
        "timing_scope": {"sparq-arrow": "process-inclusive unless --sparq-bin is reused",
                         "pyoxigraph": "in-process"},
        "versions": {"python": platform.python_version(), "pyoxigraph": pyoxigraph_version},
        "rows": rows,
    }
    rendered = json.dumps(envelope, indent=2, sort_keys=True)
    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as handle:
            handle.write(rendered + "\n")
    print(rendered)


if __name__ == "__main__":
    sys.exit(main())
