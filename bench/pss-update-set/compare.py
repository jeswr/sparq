#!/usr/bin/env python3
# [OPUS-4.8] sq-b4lo / gh-52 — written while Fable 5 unavailable; re-review when Fable returns.
"""sparq vs QLever — WRITE-PATH parity on the PSS (Solid pod-server) update set.

  compare.py <iters> [--sparq URL] [--qlever URL] [--qlever-token TOK] [--tolerance F]

gh-52 locked sparq's Phase-2 serving contract to "N readers against 1 sequenced
writer + external coordination" (see crates/sparq-server/README.md → "Concurrency
contract"). The binding acceptance criterion for that single writer is **parity-or-
better vs the QLever-over-HTTP write path on PSS's actual update set** — NOT bulk
ingest. This script is that gate: it fires the SAME interactive SPARQL UPDATEs (the
PSS LDP-CRUD shapes) at BOTH a running sparq-server and a running QLever endpoint
over HTTP on one box, measures per-update commit latency on each, and FAILS (exit 1)
if sparq's p99 regresses past QLever's by more than --tolerance (default 1.0 = exact
parity-or-better).

This intentionally mirrors `bench/qlever-olympics/compare.py` (the READ-path
comparison) for the WRITE path. Both engines must be running and writable:

  # sparq (this repo):
  cargo run -p sparq-server --release -- --addr 127.0.0.1:7020   # no auth = writable

  # QLever (see bench/qlever-olympics/README.md for the venv + qlever CLI):
  #   the Qleverfile must enable updates; ACCESS_TOKEN gates the update endpoint.
  ../../.qlever-venv/bin/qlever start    # serving on :7019, writable with the token

  # then, from this directory:
  python3 compare.py 5 --qlever-token <ACCESS_TOKEN>

Honesty: QLever's write path is an external Docker-on-host process; its numbers are
machine-specific and are NOT committed anywhere (repo hygiene forbids hard-coded
perf). This script measures both live, on the same box, in the same run — the only
fair way to assert a RELATIVE bar. For a sparq-only smoke (no QLever), use the
`pss_update_throughput` binary in bench/serve with `--qlever-baseline-ms <recorded>`.
"""
import sys
import time
import urllib.error
import urllib.parse
import urllib.request


def parse_args(argv):
    iters = int(argv[1]) if len(argv) > 1 and not argv[1].startswith("-") else 200
    opts = {
        "sparq": "http://127.0.0.1:7020/sparql",
        "qlever": "http://127.0.0.1:7019",
        "qlever_token": None,
        "tolerance": 1.0,
    }
    i = 2 if (len(argv) > 1 and not argv[1].startswith("-")) else 1
    while i < len(argv):
        a = argv[i]
        if a == "--sparq":
            opts["sparq"] = argv[i + 1]; i += 2
        elif a == "--qlever":
            opts["qlever"] = argv[i + 1]; i += 2
        elif a == "--qlever-token":
            opts["qlever_token"] = argv[i + 1]; i += 2
        elif a == "--tolerance":
            opts["tolerance"] = float(argv[i + 1]); i += 2
        else:
            sys.exit(f"unknown arg {a}")
    return iters, opts


# --- The PSS update set (mirrors crates/sparq-server/tests/named_graphs.rs and the
#     bench/serve pss_update_throughput shapes): small interactive LDP-CRUD updates. ---

LDP_CONTAINS = "http://www.w3.org/ns/ldp#contains"
ACL = "http://www.w3.org/ns/auth/acl#accessControl"


def put_document(res, version):
    r = f"http://pod.ex/p0/r{res}.ttl"
    c = "http://pod.ex/p0/"
    return (
        f"DELETE WHERE {{ GRAPH <{r}> {{ <{r}> ?p ?o }} }} ; "
        f"INSERT DATA {{ GRAPH <{r}> {{ "
        f"<{r}> <http://www.w3.org/ns/posix/stat#size> {version} . "
        f"<{r}> <https://pss.dev/ns#etag> \"e-{res}-{version}\" . }} "
        f"GRAPH <{c}> {{ <{c}> <{LDP_CONTAINS}> <{r}> . }} }}"
    )


def set_acl(res, version):
    r = f"http://pod.ex/p0/r{res}.ttl"
    c = "http://pod.ex/p0/"
    acl = f"http://pod.ex/p0/v{version}.acl"
    return (
        f"DELETE {{ GRAPH <{c}> {{ <{r}> <{ACL}> ?old }} }} "
        f"INSERT {{ GRAPH <{c}> {{ <{r}> <{ACL}> <{acl}> }} }} "
        f"WHERE  {{ OPTIONAL {{ GRAPH <{c}> {{ <{r}> <{ACL}> ?old }} }} }}"
    )


def delete_document(res):
    r = f"http://pod.ex/p0/r{res}.ttl"
    c = "http://pod.ex/p0/"
    return (
        f"DROP SILENT GRAPH <{r}> ; "
        f"DELETE WHERE {{ GRAPH <{c}> {{ <{c}> <{LDP_CONTAINS}> <{r}> }} }}"
    )


def update_set(iters, resources=200):
    """The interleaved CRUD stream: cycle put → set_acl → put → delete."""
    out = []
    for i in range(iters):
        res = i % resources
        version = i // resources + 1
        m = i % 4
        if m in (0, 2):
            out.append(put_document(res, version))
        elif m == 1:
            out.append(set_acl(res, version))
        else:
            out.append(delete_document(res))
    return out


# --- HTTP update drivers ----------------------------------------------------

def sparq_update(url, sparql):
    req = urllib.request.Request(
        url,
        data=sparql.encode(),
        headers={"content-type": "application/sparql-update"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=60) as r:
        r.read()
        if r.status not in (200, 204):
            raise RuntimeError(f"sparq update HTTP {r.status}")


def qlever_update(base, token, sparql):
    data = {"update": sparql}
    if token:
        data["access-token"] = token
    req = urllib.request.Request(base, data=urllib.parse.urlencode(data).encode(), method="POST")
    with urllib.request.urlopen(req, timeout=60) as r:
        r.read()
        if r.status not in (200, 204):
            raise RuntimeError(f"qlever update HTTP {r.status}")


def measure(label, fn, updates):
    """Returns sorted per-update commit latencies (ms). Fails hard if any update errors."""
    lat = []
    for u in updates:
        t = time.perf_counter()
        try:
            fn(u)
        except urllib.error.URLError as e:
            sys.exit(f"{label}: update failed ({e}); is the endpoint running and writable?")
        lat.append((time.perf_counter() - t) * 1000.0)
    lat.sort()
    return lat


def pct(sorted_ms, p):
    if not sorted_ms:
        return float("nan")
    idx = min(int(len(sorted_ms) * p), len(sorted_ms) - 1)
    return sorted_ms[idx]


def main():
    iters, opts = parse_args(sys.argv)
    updates = update_set(iters)
    print(f"# PSS write-path parity — {len(updates)} interactive LDP-CRUD updates, parity-or-better gate\n")

    sparq = measure("sparq", lambda u: sparq_update(opts["sparq"], u), updates)
    qlever = measure(
        "qlever", lambda u: qlever_update(opts["qlever"], opts["qlever_token"], u), updates
    )

    hdr = f"{'engine':<8} {'p50(ms)':>9} {'p99(ms)':>9} {'max(ms)':>9} {'thr(/s)':>9}"
    print(hdr)
    print("-" * len(hdr))
    for label, lat in (("sparq", sparq), ("qlever", qlever)):
        total = sum(lat) / 1000.0
        thr = len(lat) / total if total > 0 else float("nan")
        print(f"{label:<8} {pct(lat,0.5):>9.2f} {pct(lat,0.99):>9.2f} {lat[-1]:>9.2f} {thr:>9.0f}")

    s99, q99 = pct(sparq, 0.99), pct(qlever, 0.99)
    bar = q99 * opts["tolerance"]
    print(f"\n# parity gate: sparq p99 {s99:.2f}ms vs QLever p99 {q99:.2f}ms × tol {opts['tolerance']} = {bar:.2f}ms")
    if s99 > bar:
        sys.exit(f"PARITY FAIL: sparq single-writer p99 {s99:.2f}ms regressed past QLever-parity bar {bar:.2f}ms")
    print("PARITY OK: sparq single-writer is parity-or-better vs QLever on the PSS update set")


if __name__ == "__main__":
    main()
