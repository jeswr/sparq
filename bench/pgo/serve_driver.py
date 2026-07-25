#!/usr/bin/env python3
# [FABLE-5] sq-98w7z.4 — loopback HTTP driver for the PGO A/B of the sparq-server BINARY.
#
# Sends the .rq queries in --queries-dir to http://127.0.0.1:<port>/sparql over ONE
# keep-alive connection (POST application/sparql-query), so client overhead is a small,
# constant term that cancels in the baseline-vs-PGO ratio.
#
#   train   : exercise every query --repeat times (coverage for -Cprofile-generate);
#             no stats, fails hard on any non-200.
#   measure : --batches batches of (queries x --repeat) sequential requests; per-request
#             wall times; reports the BEST batch (req/s, p50/p99 ms) + total response
#             bytes. Bytes must be identical across variants (report.py enforces this
#             differential — a PGO/BOLT build must never change results).
#
# Loopback-only by construction (host is hardcoded 127.0.0.1). Stdlib-only.
import argparse
import http.client
import json
import statistics
import sys
import time
from pathlib import Path


def load_queries(qdir: Path):
    files = sorted(qdir.glob("*.rq"))
    if not files:
        sys.exit(f"[serve_driver] FATAL: no .rq files under {qdir}")
    return [(f.stem, f.read_text()) for f in files]


def run_pass(conn, queries, times, want_bytes):
    total = 0
    for name, q in queries:
        body = q.encode()
        t0 = time.perf_counter_ns()
        conn.request(
            "POST",
            "/sparql",
            body=body,
            headers={
                "Content-Type": "application/sparql-query",
                # The server negotiates a sensible type per query form (JSON results for
                # SELECT/ASK, an RDF serialization for CONSTRUCT/DESCRIBE).
                "Accept": "*/*",
            },
        )
        resp = conn.getresponse()
        payload = resp.read()
        dt_ns = time.perf_counter_ns() - t0
        if resp.status != 200:
            sys.exit(
                f"[serve_driver] FATAL: {name} -> HTTP {resp.status}: {payload[:200]!r}"
            )
        if times is not None:
            times.append(dt_ns)
        if want_bytes:
            total += len(payload)
    return total


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--port", type=int, required=True)
    ap.add_argument("--queries-dir", type=Path, required=True)
    ap.add_argument("--mode", choices=["train", "measure"], required=True)
    ap.add_argument("--repeat", type=int, default=25, help="passes over the query mix per batch")
    ap.add_argument("--batches", type=int, default=3, help="measure mode: batches; best reported")
    ap.add_argument("--out", type=Path, help="measure mode: write the JSON result here")
    ap.add_argument("--timeout", type=float, default=120.0)
    args = ap.parse_args()

    queries = load_queries(args.queries_dir)
    conn = http.client.HTTPConnection("127.0.0.1", args.port, timeout=args.timeout)

    if args.mode == "train":
        for _ in range(args.repeat):
            run_pass(conn, queries, times=None, want_bytes=False)
        conn.close()
        print(f"[serve_driver] trained: {len(queries)} queries x {args.repeat} passes", file=sys.stderr)
        return

    best = None
    for b in range(args.batches):
        times = []
        total_bytes = 0
        t0 = time.perf_counter_ns()
        for _ in range(args.repeat):
            total_bytes += run_pass(conn, queries, times, True)
        wall_s = (time.perf_counter_ns() - t0) / 1e9
        ms = sorted(t / 1e6 for t in times)
        batch = {
            "queries": len(queries),
            "passes": args.repeat,
            "requests": len(times),
            "wall_s": round(wall_s, 4),
            "rps": round(len(times) / wall_s, 2),
            "p50_ms": round(statistics.median(ms), 4),
            "p99_ms": round(ms[min(len(ms) - 1, int(len(ms) * 0.99))], 4),
            "total_bytes": total_bytes,
        }
        print(f"[serve_driver] batch {b + 1}/{args.batches}: {batch['rps']} req/s p50={batch['p50_ms']}ms", file=sys.stderr)
        if best is None or batch["rps"] > best["rps"]:
            best = batch
    conn.close()

    out = json.dumps(best, indent=2)
    if args.out:
        args.out.write_text(out + "\n")
    print(out)


if __name__ == "__main__":
    main()
