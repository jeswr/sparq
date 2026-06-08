#!/usr/bin/env python3
"""sparq vs QLever on the Olympics dataset (~1.78M triples).

  compare.py <iters> <pass>     pass = endtoend | compute

Both passes run min-of-<iters> COLD runs (QLever cache cleared before each run;
sparq has no query cache). QLever's time is its own `query-time-ms`; sparq's is
in-process.

  endtoend  fair end-to-end: both compute AND serialise the full result to SPARQL
            JSON. Uses queries/ (10 queries). The result SIZE of every query is a
            correctness cross-check against QLever.

  compute   fair engine-only: isolates join/scan compute from serialisation.
            sparq runs queries/ in `count` mode (solution count, no term
            materialisation); QLever runs the COUNT(*)-wrapped queries-count/
            (8 queries: the BGP/scan/filter/OPTIONAL ones, not the two queries
            that are themselves aggregates). The COUNT VALUE is compared to
            sparq's count — a real correctness check, not just "1 row == 1 row".

Prereqs: `qlever start` running on PORT; sparq-cli built --release.
"""
import json
import subprocess
import sys
import urllib.parse
import urllib.request
from pathlib import Path

HERE = Path(__file__).parent
PORT = 7030
ENDPOINT = f"http://localhost:{PORT}"
ACCESS_TOKEN = "synthetic100m_token"

ITERS = int(sys.argv[1]) if len(sys.argv) > 1 else 5
PASS = sys.argv[2] if len(sys.argv) > 2 else "endtoend"
DATA = HERE / "synthetic100m.nt"
SPARQ_CLI = HERE / "../../target/release/sparq-cli"


def http(data):
    req = urllib.request.Request(ENDPOINT, data=urllib.parse.urlencode(data).encode())
    with urllib.request.urlopen(req, timeout=300) as r:
        return r.read()


def clear_cache():
    try:
        http({"cmd": "clear-cache", "access-token": ACCESS_TOKEN})
    except Exception:
        pass


def qlever_run(sparql):
    """(result_size, count_value_or_None, min_ms) over ITERS cold runs."""
    best = float("inf")
    size = None
    count_val = None
    for _ in range(ITERS):
        clear_cache()
        obj = json.loads(http({"query": sparql, "action": "json_export"}))
        meta = obj.get("meta", {})
        size = meta.get("result-size-total")
        bindings = obj.get("results", {}).get("bindings", [])
        if size is None:
            size = len(bindings)
        # A COUNT(*) query has one binding `?c` whose value is the count.
        if len(bindings) == 1 and len(bindings[0]) == 1:
            count_val = next(iter(bindings[0].values())).get("value")
        best = min(best, float(meta.get("query-time-ms", 0)))
    return size, count_val, best


def sparq_bench(queries_dir, mode):
    """Runs sparq-cli bench; returns ({name:(rows,micros)}, load_line). Fails hard."""
    proc = subprocess.run(
        [str(SPARQ_CLI), "bench", str(DATA), "ntriples", str(queries_dir), str(ITERS), mode],
        capture_output=True, text=True,
    )
    if proc.returncode != 0:
        sys.exit(f"sparq-cli failed (exit {proc.returncode}):\n{proc.stderr}")
    load = next((l.strip() for l in proc.stderr.splitlines() if "loaded" in l), "")
    out = {}
    for line in proc.stdout.splitlines():
        parts = line.split("\t")
        if len(parts) == 3:
            out[parts[0]] = (parts[1], parts[2])
    if not out:
        sys.exit(f"sparq-cli produced no results:\n{proc.stdout}\n{proc.stderr}")
    return out, load


def geomean(xs):
    g = 1.0
    for x in xs:
        g *= x
    return g ** (1.0 / len(xs)) if xs else float("nan")


def report(rows):
    """rows: list of (name, sparq_n, qlever_n, ok, sparq_ms, qlever_ms)."""
    hdr = f"{'query':<26} {'sparq#':>9} {'qlever#':>9} {'match':>6} {'sparq':>10} {'qlever':>10} {'speedup':>8}"
    print(hdr)
    print("-" * len(hdr))
    speedups, mismatches = [], 0
    for name, sn, qn, ok, sms, qms in rows:
        mismatches += 0 if ok else 1
        spd = qms / sms if sms and sms > 0 else float("nan")
        if spd == spd:
            speedups.append(spd)
        print(f"{name:<26} {str(sn):>9} {str(qn):>9} {('ok' if ok else 'DIFF'):>6} "
              f"{sms:>9.2f}m {qms:>9.2f}m {spd:>7.2f}x")
    print()
    g = geomean(speedups)
    print(f"geomean (qlever_time / sparq_time): {g:.2f}x  "
          f"({'sparq faster' if g > 1 else 'QLever faster'})")
    if mismatches:
        sys.exit(f"WARNING: {mismatches} correctness mismatch(es) vs QLever")


def run_endtoend():
    print(f"# end-to-end (both serialise full SPARQL JSON) — min of {ITERS} cold runs\n")
    sparq, load = sparq_bench(HERE / "queries", "json")
    print(f"sparq:  {load}\nqlever: Docker server on :{PORT}\n")
    rows = []
    for path in sorted((HERE / "queries").glob("*.rq")):
        name = path.stem
        srows, smicros = sparq.get(name, ("MISSING", "nan"))
        qsize, _, qms = qlever_run(path.read_text())
        sms = float(smicros) / 1000.0 if smicros not in ("nan", "ERROR", "MISSING") else float("nan")
        ok = str(srows) == str(qsize)
        rows.append((name, srows, qsize, ok, sms, qms))
    report(rows)


def run_compute():
    print(f"# compute-only (sparq count vs QLever COUNT-wrapped) — min of {ITERS} cold runs\n")
    sparq, load = sparq_bench(HERE / "queries", "count")
    print(f"sparq:  {load}\nqlever: Docker server on :{PORT}\n")
    rows = []
    for path in sorted((HERE / "queries-count").glob("*.rq")):
        name = path.stem  # matches a query in queries/
        srows, smicros = sparq.get(name, ("MISSING", "nan"))
        _, qcount, qms = qlever_run(path.read_text())
        sms = float(smicros) / 1000.0 if smicros not in ("nan", "ERROR", "MISSING") else float("nan")
        # sparq's solution count of the original query == QLever's COUNT value.
        ok = qcount is not None and str(srows) == str(qcount)
        rows.append((name, srows, qcount, ok, sms, qms))
    report(rows)


if __name__ == "__main__":
    if PASS == "endtoend":
        run_endtoend()
    elif PASS == "compute":
        run_compute()
    else:
        sys.exit("usage: compare.py <iters> <endtoend|compute>")
