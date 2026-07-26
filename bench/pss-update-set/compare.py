#!/usr/bin/env python3
# [OPUS-4.8] sq-b4lo / gh-52 — written while Fable 5 unavailable; re-review when Fable returns.
# [SONNET-4.6] sq-hmd7l.5 — extended to Fuseki + Oxigraph; post-workload store-state oracle added.
"""sparq vs QLever / Fuseki / Oxigraph — WRITE-PATH parity on the PSS (Solid pod-server) update set.

  compare.py <iters> [--sparq URL] [--qlever URL] [--qlever-token TOK]
             [--fuseki URL] [--oxigraph URL] [--tolerance F] [--json-out FILE]

gh-52 locked sparq's Phase-2 serving contract to "N readers against 1 sequenced
writer + external coordination" (see crates/sparq-server/README.md → "Concurrency
contract"). The binding acceptance criterion for that single writer is **parity-or-
better vs the QLever-over-HTTP write path on PSS's actual update set** — NOT bulk
ingest. This script is that gate: it fires the SAME interactive SPARQL UPDATEs (the
PSS LDP-CRUD shapes) at a running sparq-server AND any of QLever / Fuseki / Oxigraph
endpoints over HTTP on one box, measures per-update commit latency on each, and FAILS
(exit 1) if sparq's p99 regresses past any competitor's by more than --tolerance
(default 1.0 = exact parity-or-better).

Store-state oracle (sq-hmd7l.5 INVARIANT): after the workload a COUNT of named-graph
quads is run against each active engine's query endpoint.  Disagreement is recorded
honestly in the output — latency numbers are flagged but NOT suppressed (the gate is
honesty, not a blind pass).

Engine URLs:
  --sparq    Full SPARQL endpoint URL (query + update via POST; default
             http://127.0.0.1:7020/sparql). sparq-server accepts both
             application/sparql-query and application/sparql-update on this path.
  --qlever   QLever base URL (update via form POST; omit to skip QLever;
             default None — QLever is skipped unless this flag is provided,
             EXCEPT that --qlever-token alone implies http://127.0.0.1:7019
             for backward-compat with the registered pss-update-parity invoke)
  --qlever-token TOKEN  QLever access token for the update endpoint (optional)
  --fuseki   Fuseki dataset base URL (update: <url>/update, query: <url>/query;
             e.g. http://127.0.0.1:3030/ds; omit to skip Fuseki)
  --oxigraph Oxigraph base URL (update: <url>/update, query: <url>/query;
             e.g. http://127.0.0.1:7878; omit to skip Oxigraph)
  --tolerance FLOAT  p99 parity tolerance multiplier (default 1.0 = exact parity)
  --json-out FILE    write machine-readable results summary to FILE (JSON)

This intentionally mirrors `bench/qlever-olympics/compare.py` (the READ-path
comparison) for the WRITE path. The active engines must be running and writable;
see scripts/bench/update-same-box.sh for the durable start/stop orchestration.

Honesty: numbers are machine-specific and are NOT committed anywhere (repo hygiene
forbids hard-coded perf). This script measures engines live, on the same box, in the
same run — the only fair way to assert a RELATIVE bar.
"""
import json as _json
import sys
import time
import urllib.error
import urllib.parse
import urllib.request


def parse_args(argv):
    iters = int(argv[1]) if len(argv) > 1 and not argv[1].startswith("-") else 200
    opts = {
        "sparq": "http://127.0.0.1:7020/sparql",
        "qlever": None,
        "qlever_token": None,
        "fuseki": None,
        "oxigraph": None,
        "tolerance": 1.0,
        "json_out": None,
    }
    i = 2 if (len(argv) > 1 and not argv[1].startswith("-")) else 1
    while i < len(argv):
        a = argv[i]
        if a == "--sparq":
            opts["sparq"] = argv[i + 1]
            i += 2
        elif a == "--qlever":
            opts["qlever"] = argv[i + 1]
            i += 2
        elif a == "--qlever-token":
            opts["qlever_token"] = argv[i + 1]
            i += 2
        elif a == "--fuseki":
            opts["fuseki"] = argv[i + 1]
            i += 2
        elif a == "--oxigraph":
            opts["oxigraph"] = argv[i + 1]
            i += 2
        elif a == "--tolerance":
            opts["tolerance"] = float(argv[i + 1])
            i += 2
        elif a == "--json-out":
            opts["json_out"] = argv[i + 1]
            i += 2
        else:
            sys.exit(f"unknown arg {a}")
    # Backward-compat with the registered pss-update-parity invocation
    # (bench/benchmarks.toml passes only --qlever-token): a token implies the
    # historical local QLever endpoint unless --qlever overrides it.
    if opts["qlever"] is None and opts["qlever_token"] is not None:
        opts["qlever"] = "http://127.0.0.1:7019"
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
    req = urllib.request.Request(
        base,
        data=urllib.parse.urlencode(data).encode(),
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=60) as r:
        r.read()
        if r.status not in (200, 204):
            raise RuntimeError(f"qlever update HTTP {r.status}")


def fuseki_update(base, sparql):
    """POST SPARQL UPDATE to a Fuseki dataset update endpoint (<base>/update)."""
    url = base.rstrip("/") + "/update"
    req = urllib.request.Request(
        url,
        data=sparql.encode(),
        headers={"content-type": "application/sparql-update"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=60) as r:
        r.read()
        if r.status not in (200, 204):
            raise RuntimeError(f"fuseki update HTTP {r.status}")


def oxigraph_update(base, sparql):
    """POST SPARQL UPDATE to an Oxigraph server update endpoint (<base>/update)."""
    url = base.rstrip("/") + "/update"
    req = urllib.request.Request(
        url,
        data=sparql.encode(),
        headers={"content-type": "application/sparql-update"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=60) as r:
        r.read()
        if r.status not in (200, 204):
            raise RuntimeError(f"oxigraph update HTTP {r.status}")


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


# --- Post-workload store-state oracle (sq-hmd7l.5 INVARIANT) ----------------

# Count named-graph quads as a deterministic store-state fingerprint after the workload.
_COUNT_Q = "SELECT (COUNT(*) AS ?n) WHERE { GRAPH ?g { ?s ?p ?o } }"


def _parse_sparql_count_json(data):
    """Extract first ?n binding value from SPARQL results JSON bytes."""
    obj = _json.loads(data)
    bindings = obj.get("results", {}).get("bindings", [])
    return bindings[0]["n"]["value"] if bindings else "0"


def state_count(query_url):
    """POST COUNT(*) over named-graph quads to a standard SPARQL query endpoint.

    Returns the count string or 'ERROR:<reason>'.
    """
    req = urllib.request.Request(
        query_url,
        data=_COUNT_Q.encode(),
        headers={
            "content-type": "application/sparql-query",
            "accept": "application/sparql-results+json",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            return _parse_sparql_count_json(r.read())
    except Exception as exc:  # noqa: BLE001
        return f"ERROR:{exc}"


def state_count_qlever(base, token=None):
    """COUNT named-graph quads via QLever's form-POST query endpoint."""
    data = {"query": _COUNT_Q}
    if token:
        data["access-token"] = token
    req = urllib.request.Request(
        base,
        data=urllib.parse.urlencode(data).encode(),
        headers={"accept": "application/sparql-results+json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            return _parse_sparql_count_json(r.read())
    except Exception as exc:  # noqa: BLE001
        return f"ERROR:{exc}"


def main():
    iters, opts = parse_args(sys.argv)
    updates = update_set(iters)

    # --- Build the active engine set -----------------------------------------
    # List of (label, update_driver) for engines that have a URL configured.
    active = []
    if opts["sparq"]:
        active.append(("sparq", lambda u: sparq_update(opts["sparq"], u)))
    if opts["qlever"]:
        tok = opts["qlever_token"]
        qlever_base = opts["qlever"]
        active.append(("qlever", lambda u: qlever_update(qlever_base, tok, u)))
    if opts["fuseki"]:
        fuseki_base = opts["fuseki"]
        active.append(("fuseki", lambda u: fuseki_update(fuseki_base, u)))
    if opts["oxigraph"]:
        oxi_base = opts["oxigraph"]
        active.append(("oxigraph", lambda u: oxigraph_update(oxi_base, u)))

    engine_labels = [n for n, _ in active]
    print(
        f"# PSS write-path parity — {len(updates)} interactive LDP-CRUD updates,"
        f" engines: {engine_labels}\n"
    )

    # --- Measure each engine sequentially (same box, fair relative comparison) --
    lat = {}
    for label, driver in active:
        lat[label] = measure(label, driver, updates)

    # --- Latency table --------------------------------------------------------
    hdr = f"{'engine':<10} {'p50(ms)':>9} {'p99(ms)':>9} {'max(ms)':>9} {'thr(/s)':>9}"
    print(hdr)
    print("-" * len(hdr))
    for label, times in lat.items():
        total_s = sum(times) / 1000.0
        thr = len(times) / total_s if total_s > 0 else float("nan")
        print(
            f"{label:<10} {pct(times, 0.5):>9.2f} {pct(times, 0.99):>9.2f}"
            f" {times[-1]:>9.2f} {thr:>9.0f}"
        )

    # --- Post-workload store-state oracle (sq-hmd7l.5 INVARIANT) ---------------
    # Count named-graph quads per engine; disagreement is recorded honestly —
    # latency numbers are NOT suppressed (the gate is honesty, not a blind pass).
    counts = {}
    if opts["sparq"]:
        counts["sparq"] = state_count(opts["sparq"])
    if opts["qlever"]:
        counts["qlever"] = state_count_qlever(opts["qlever"], opts["qlever_token"])
    if opts["fuseki"]:
        counts["fuseki"] = state_count(opts["fuseki"].rstrip("/") + "/query")
    if opts["oxigraph"]:
        counts["oxigraph"] = state_count(opts["oxigraph"].rstrip("/") + "/query")

    non_error_counts = {e: v for e, v in counts.items() if not v.startswith("ERROR")}
    state_agree = len(set(non_error_counts.values())) <= 1

    print("\n# post-workload named-graph quad COUNT (store-state oracle):")
    for e, v in counts.items():
        print(f"#   {e}: {v}")
    if non_error_counts:
        print(f"#   all_agree: {state_agree}")
    if not state_agree:
        print(
            "# WARNING: store-state MISMATCH across engines —"
            " latency rows are flagged but NOT suppressed"
        )

    # --- Relative parity gate -------------------------------------------------
    parity_gate = "single-engine-skip"
    parity_note = "single-engine run — relative parity gate skipped"

    if "sparq" in lat and len(lat) >= 2:
        s99 = pct(lat["sparq"], 0.99)
        fails = []
        for comp, times in lat.items():
            if comp == "sparq":
                continue
            c99 = pct(times, 0.99)
            bar = c99 * opts["tolerance"]
            print(
                f"\n# parity gate: sparq p99 {s99:.2f}ms vs {comp} p99 {c99:.2f}ms"
                f" × tol {opts['tolerance']} = {bar:.2f}ms"
            )
            if s99 > bar:
                fails.append(
                    f"PARITY FAIL: sparq single-writer p99 {s99:.2f}ms regressed past"
                    f" {comp}-parity bar {bar:.2f}ms"
                )
        if fails:
            parity_gate = "fail"
            parity_note = "; ".join(fails)
        else:
            parity_gate = "pass"
            parity_note = (
                "sparq single-writer is parity-or-better vs all active engines"
                " on the PSS update set"
            )

    print(f"\n# parity_gate: {parity_gate}")
    print(f"# {parity_note}")

    # --- Optional machine-readable JSON output --------------------------------
    if opts["json_out"]:
        latency_ms = {
            e: {
                "p50": round(pct(t, 0.5), 3),
                "p99": round(pct(t, 0.99), 3),
                "max": round(t[-1], 3),
            }
            for e, t in lat.items()
        }
        throughput_s = {}
        for e, t in lat.items():
            total_s = sum(t) / 1000.0
            throughput_s[e] = round(len(t) / total_s, 1) if total_s > 0 else 0.0

        summary = {
            "latency_ms": latency_ms,
            "throughput_s": throughput_s,
            "state_count": counts,
            "state_agree": state_agree,
            "parity_gate": parity_gate,
            "parity_note": parity_note,
        }
        with open(opts["json_out"], "w", encoding="utf-8") as fh:
            _json.dump(summary, fh, indent=2)
            fh.write("\n")

    # [SONNET-4.6] Keep this exit after --json-out is persisted: envelope callers
    # must be able to ingest honest measurements while still receiving a failed gate.
    if parity_gate == "fail":
        sys.exit(parity_note)


if __name__ == "__main__":
    main()
