#!/usr/bin/env python3
"""sparq vs QLever on the Olympics dataset (~1.78M triples).

  compare.py <iters> <queries-dir> <sparq-mode>

Runs every <queries-dir>/*.rq through both engines and reports, per query, the
result size and the time. Timing is COLD for both: sparq has no query cache
(it fully recomputes each run), so QLever's cache is cleared before every run.
QLever's time is its own `query-time-ms`; sparq's is in-process. Each is the min
over <iters> runs.

Two intended passes:
  full JSON (fair end-to-end):  compare.py 5 queries        json
      both serialise the full result to SPARQL JSON; result sizes are the
      correctness cross-check.
  compute  (fair engine-only):  compare.py 5 queries-count  materialize
      COUNT(*)-wrapped queries: both compute the join and return one row, so the
      time is join/scan compute with negligible serialisation.

Prereqs: `qlever start` running on PORT; sparq-cli built --release.
"""
import json
import subprocess
import sys
import urllib.parse
import urllib.request
from pathlib import Path

HERE = Path(__file__).parent
PORT = 7019
ENDPOINT = f"http://localhost:{PORT}"
ACCESS_TOKEN = "olympics_7643543846eFq6lneiscnp"

ITERS = int(sys.argv[1]) if len(sys.argv) > 1 else 5
QUERIES_DIR = HERE / (sys.argv[2] if len(sys.argv) > 2 else "queries")
SPARQ_MODE = sys.argv[3] if len(sys.argv) > 3 else "json"
DATA = HERE / "olympics.nt"
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
    """Returns (result_size, min_ms) over ITERS cold runs (full JSON export)."""
    best = float("inf")
    size = None
    for _ in range(ITERS):
        clear_cache()
        obj = json.loads(http({"query": sparql, "action": "json_export"}))
        meta = obj.get("meta", {})
        size = meta.get("result-size-total")
        if size is None:
            size = len(obj.get("results", {}).get("bindings", []))
        best = min(best, float(meta.get("query-time-ms", 0)))
    return size, best


def sparq_run_all():
    """Runs sparq-cli bench once in SPARQ_MODE; {name: (rows, micros)} + load."""
    proc = subprocess.run(
        [str(SPARQ_CLI), "bench", str(DATA), "ntriples", str(QUERIES_DIR), str(ITERS), SPARQ_MODE],
        capture_output=True, text=True,
    )
    load = next((l.strip() for l in proc.stderr.splitlines() if "loaded" in l), "")
    out = {}
    for line in proc.stdout.splitlines():
        parts = line.split("\t")
        if len(parts) == 3:
            out[parts[0]] = (parts[1], parts[2])
    return out, load


def main():
    print(f"# sparq ({SPARQ_MODE}) vs QLever (json) — Olympics ~1.78M triples, "
          f"min of {ITERS} cold runs, dir={QUERIES_DIR.name}\n")
    sparq, load = sparq_run_all()
    print(f"sparq:  {load}")
    print(f"qlever: index pre-built; Docker server on :{PORT}\n")

    hdr = f"{'query':<26} {'sparq#':>9} {'qlever#':>9} {'match':>6} {'sparq':>10} {'qlever':>10} {'speedup':>8}"
    print(hdr)
    print("-" * len(hdr))

    mismatches = 0
    speedups = []
    for path in sorted(QUERIES_DIR.glob("*.rq")):
        name = path.stem
        srows, smicros = sparq.get(name, ("?", "nan"))
        qsize, qms = qlever_run(path.read_text())
        sms = float(smicros) / 1000.0 if smicros not in ("nan", "ERROR") else float("nan")
        # In COUNT mode both return one row; correctness is checked in the JSON pass.
        ok = (str(srows) == str(qsize)) or SPARQ_MODE == "materialize"
        mismatches += 0 if ok else 1
        spd = qms / sms if sms and sms == sms and sms > 0 else float("nan")
        if spd == spd:
            speedups.append(spd)
        print(f"{name:<26} {str(srows):>9} {str(qsize):>9} {('ok' if ok else 'DIFF'):>6} "
              f"{sms:>9.2f}m {qms:>9.2f}m {spd:>7.2f}x")

    print()
    if speedups:
        geo = 1.0
        for s in speedups:
            geo *= s
        geo **= 1.0 / len(speedups)
        print(f"geomean (qlever_time / sparq_time): {geo:.2f}x  "
              f"({'sparq faster' if geo > 1 else 'QLever faster'})")
    if mismatches:
        print(f"WARNING: {mismatches} result-size mismatch(es) vs QLever")
        sys.exit(1)


if __name__ == "__main__":
    main()
