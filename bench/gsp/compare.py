#!/usr/bin/env python3
# [FABLE-5] sq-hmd7l.21 — Graph Store Protocol same-box panel driver.
"""sparq-server vs Fuseki GSP / Oxigraph GSP (+ Community Solid Server, LOOSE) —
Graph Store Protocol PUT/GET/DELETE round-trips, same box, loopback only.

  compare.py [--sparq URL] [--fuseki URL] [--oxigraph URL] [--css URL]
             [--sizes CSV] [--iters N] [--crud-ops N] [--smoke]
             [--json-out FILE] [--self-test]

Two workloads, each behind a HARD correctness gate that runs BEFORE any timing
(the bead invariant: round-trip content agreement first, numbers second):

  size-sweep   Deterministic bnode-free N-Triples payloads at each --sizes byte
               target (default 1 KB -> 10 MB). Gate: PUT the graph, GET it back
               (application/n-triples), parse, and require the returned triple
               SET to equal the sent set exactly (ground triples, so set
               equality IS isomorphism) — then DELETE and require the graph to
               read as absent (404, or 200 with zero triples: engines
               legitimately differ on GSP absent-graph reads). Timing (only
               after a green gate): per-iteration PUT / GET / DELETE latency on
               a fresh graph name each iteration.

  crud-replay  The PSS LDP-CRUD stream (bench/pss-update-set) PROJECTED onto
               GSP verbs: put_document -> PUT <r> + POST containment into the
               container graph; set_acl -> PUT of a one-triple <r>.acl graph
               (GSP PUT replace = the idempotent pointer swap; the conditional
               DELETE/INSERT/WHERE shape is not GSP-expressible); delete_document
               -> DELETE <r> + DELETE <r>.acl. Gate: a client-side reference
               model replays the same stream and every resource's final state
               (present triples / absent) must match a GET against the engine.

Engine columns:
  --sparq    sparq-server base URL (default http://127.0.0.1:7033). GSP endpoint:
             <base>/sparql/graph?graph=<iri> (indirect identification).
  --fuseki   Fuseki dataset base URL (e.g. http://127.0.0.1:3030/ds). GSP
             endpoint: <base>/data?graph=<iri>. Omit to skip.
  --oxigraph Oxigraph server base URL (e.g. http://127.0.0.1:7878). GSP
             endpoint: <base>/store?graph=<iri>. Omit to skip.
  --css      Community Solid Server pod base URL. **LOOSE column**: CSS is a
             Solid/LDP stack (persistence, access control, negotiation), not a
             pure GSP store — ops address the resource URL directly (LDP), and
             every css row is labelled LOOSE and must NEVER be averaged with
             the like-for-like columns. Omit to skip.

LOOPBACK ONLY (fail-closed): every engine URL must resolve to a loopback host
(127.0.0.1 / localhost / ::1). Anything else is refused before any request —
this harness must not be pointed at a remote endpoint.

Honesty: numbers are machine-specific and NOT committed anywhere (repo hygiene
forbids hard-coded perf). Anything measured on a shared work box is
NON-canonical; see research/gap-gsp-2026-07.md.
"""

import argparse
import json
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

DEFAULT_SIZES = "1024,10240,102400,1048576,10485760"  # 1 KB .. 10 MB
SMOKE_SIZES = "1024,10240"
LOOPBACK_HOSTS = {"127.0.0.1", "localhost", "::1"}

LDP_CONTAINS = "http://www.w3.org/ns/ldp#contains"
ACL_PTR = "http://www.w3.org/ns/auth/acl#accessControl"


# --------------------------------------------------------------------------- #
# Loopback guard (bead invariant: loopback-only testing, no bypass)
# --------------------------------------------------------------------------- #
def assert_loopback(url, label):
    """Hard-refuses a non-loopback engine URL. Returns the URL unchanged."""
    host = urllib.parse.urlparse(url).hostname
    if host not in LOOPBACK_HOSTS:
        sys.exit(
            f"{label}: refusing non-loopback URL {url!r} (host {host!r}) — "
            "this harness is loopback-only by design"
        )
    return url


# --------------------------------------------------------------------------- #
# Deterministic payload generation + N-Triples canonicalisation
# --------------------------------------------------------------------------- #
def gen_ntriples(target_bytes, tag):
    """Deterministic, bnode-free, ASCII-only N-Triples of >= target_bytes.

    Plain-ASCII IRIs + simple string literals only, so every serialiser
    (sparq / Fuseki / Oxigraph) re-emits the identical canonical line and the
    round-trip gate can require exact SET equality (ground triples: set
    equality is isomorphism)."""
    lines = []
    size = 0
    i = 0
    while size < target_bytes:
        line = (
            f"<http://gsp.bench/{tag}/s{i}> <http://gsp.bench/p> "
            f'"v{i}-{tag}-payload-padding-0123456789abcdef" .\n'
        )
        lines.append(line)
        size += len(line)
        i += 1
    return "".join(lines)


def canon(nt_text):
    """Canonical triple set of an N-Triples document: stripped, non-empty,
    non-comment lines. Sufficient for this harness's ASCII/ground payloads."""
    out = set()
    for raw in nt_text.splitlines():
        line = raw.strip()
        if line and not line.startswith("#"):
            out.add(line)
    return frozenset(out)


# --------------------------------------------------------------------------- #
# Engine adapters: put / get / delete on a named graph
# --------------------------------------------------------------------------- #
class GspEngine:
    """A standard GSP endpoint (indirect graph identification, ?graph=<iri>)."""

    loose = False

    def __init__(self, name, gsp_url):
        self.name = name
        self.gsp_url = gsp_url.rstrip("?")

    def _url(self, graph_iri):
        return f"{self.gsp_url}?graph={urllib.parse.quote(graph_iri, safe='')}"

    def _request(self, method, graph_iri, body=None, headers=None):
        req = urllib.request.Request(
            self._url(graph_iri), data=body, headers=headers or {}, method=method
        )
        try:
            with urllib.request.urlopen(req, timeout=120) as r:
                return r.status, r.read()
        except urllib.error.HTTPError as e:
            return e.code, e.read()
        except urllib.error.URLError as e:
            # e.g. a broken pipe when the server refuses an over-cap body mid-
            # stream. Surface it as a synthetic 5xx so the correctness gate
            # reports an honest FAIL (with detail) instead of a stack trace.
            return 599, f"transport error: {e.reason}".encode()

    def put(self, graph_iri, nt_body):
        return self._request(
            "PUT",
            graph_iri,
            body=nt_body.encode(),
            headers={"content-type": "application/n-triples"},
        )

    def post(self, graph_iri, nt_body):
        return self._request(
            "POST",
            graph_iri,
            body=nt_body.encode(),
            headers={"content-type": "application/n-triples"},
        )

    def get(self, graph_iri):
        return self._request(
            "GET", graph_iri, headers={"accept": "application/n-triples"}
        )

    def delete(self, graph_iri):
        return self._request("DELETE", graph_iri)


class CssEngine(GspEngine):
    """Community Solid Server — LOOSE column. LDP resource addressing: the
    resource URL IS the graph (direct identification); the payload rides as
    text/turtle (N-Triples is valid Turtle) because Turtle is CSS's native
    representation. Solid/LDP overhead (persistence, authz, negotiation) is
    INCLUDED in every number — never average with the like-for-like columns."""

    loose = True

    def _url(self, graph_iri):
        # Map the abstract graph IRI onto a pod resource path (LDP addressing).
        slug = urllib.parse.quote(graph_iri, safe="")
        return f"{self.gsp_url.rstrip('/')}/gsp-bench/{slug}.ttl"

    def put(self, graph_iri, nt_body):
        return self._request(
            "PUT",
            graph_iri,
            body=nt_body.encode(),
            headers={"content-type": "text/turtle"},
        )

    def post(self, graph_iri, nt_body):
        # LDP has no graph-merge POST onto an existing RDF resource; the loose
        # projection is read-modify-PUT. Recorded as one "post" op (the overhead
        # of the extra GET is part of the honest LOOSE architecture difference).
        status, body = self.get(graph_iri)
        merged = (body.decode() if status == 200 else "") + nt_body
        return self.put(graph_iri, merged)


def absent(status, body):
    """A GSP read of an absent graph: 404 (Fuseki/CSS style) or 200 with zero
    triples (sparq serialises the absent named graph as the empty graph)."""
    if status in (404, 410):
        return True
    return status == 200 and not canon(body.decode(errors="replace"))


# --------------------------------------------------------------------------- #
# Workload A — resource-size sweep
# --------------------------------------------------------------------------- #
def gate_roundtrip(engine, size):
    """The HARD pre-timing gate: PUT -> GET -> content set-equality -> DELETE
    -> absent. Returns (ok, detail)."""
    graph = f"http://gsp.bench/gate/{size}"
    payload = gen_ntriples(size, f"gate{size}")
    sent = canon(payload)

    status, _ = engine.put(graph, payload)
    if status not in (200, 201, 204, 205):
        return False, f"PUT HTTP {status}"
    status, body = engine.get(graph)
    if status != 200:
        return False, f"GET HTTP {status}"
    got = canon(body.decode(errors="replace"))
    if got != sent:
        return (
            False,
            f"round-trip mismatch: sent {len(sent)} triples, got {len(got)} "
            f"({len(sent - got)} missing, {len(got - sent)} unexpected)",
        )
    status, _ = engine.delete(graph)
    if status not in (200, 204, 205):
        return False, f"DELETE HTTP {status}"
    status, body = engine.get(graph)
    if not absent(status, body):
        return False, f"graph still readable after DELETE (HTTP {status})"
    return True, f"{len(sent)} triples round-tripped byte-identically"


def time_sweep(engine, size, iters):
    """PUT/GET/DELETE latency over `iters` fresh graphs at `size` bytes.
    Only called after gate_roundtrip() passed."""
    payload = gen_ntriples(size, f"t{size}")
    lat = {"put": [], "get": [], "delete": []}
    for i in range(iters):
        graph = f"http://gsp.bench/sweep/{size}/{i}"
        for op, fn in (
            ("put", lambda g=graph: engine.put(g, payload)),
            ("get", lambda g=graph: engine.get(g)),
            ("delete", lambda g=graph: engine.delete(g)),
        ):
            t = time.perf_counter()
            status = fn()[0]
            lat[op].append((time.perf_counter() - t) * 1000.0)
            if status >= 400:
                raise RuntimeError(f"{engine.name}: timed {op} failed HTTP {status}")
    for v in lat.values():
        v.sort()
    return lat


# --------------------------------------------------------------------------- #
# Workload B — the PSS LDP-CRUD stream projected onto GSP verbs
# --------------------------------------------------------------------------- #
def doc_graph(res):
    return f"http://pod.ex/p0/r{res}.ttl"


def acl_graph(res):
    return f"http://pod.ex/p0/r{res}.ttl.acl"


CONTAINER = "http://pod.ex/p0/"


def doc_triples(res, version):
    r = doc_graph(res)
    return (
        f"<{r}> <http://www.w3.org/ns/posix/stat#size> \"{version}\" .\n"
        f"<{r}> <https://pss.dev/ns#etag> \"e-{res}-{version}\" .\n"
    )


def acl_triples(res, version):
    return f"<{doc_graph(res)}> <{ACL_PTR}> <http://pod.ex/p0/v{version}.acl> .\n"


def containment_triple(res):
    return f"<{CONTAINER}> <{LDP_CONTAINS}> <{doc_graph(res)}> .\n"


def crud_stream(ops, resources=8):
    """The pss-update-set stream shape (put / acl / put / delete cycle),
    projected onto GSP verbs. Yields (op_kind, graph_iri, body_or_None)."""
    out = []
    for i in range(ops):
        res = i % resources
        version = i // resources + 1
        m = i % 4
        if m in (0, 2):
            out.append(("put", doc_graph(res), doc_triples(res, version)))
            out.append(("post", CONTAINER, containment_triple(res)))
        elif m == 1:
            out.append(("put", acl_graph(res), acl_triples(res, version)))
        else:
            out.append(("delete", doc_graph(res), None))
            out.append(("delete", acl_graph(res), None))
    return out


def crud_reference_state(stream):
    """Client-side reference model: the expected final canonical triple set per
    graph after replaying `stream` (None body on delete removes the graph)."""
    state = {}
    for op, graph, body in stream:
        if op == "put":
            state[graph] = canon(body)
        elif op == "post":
            state[graph] = state.get(graph, frozenset()) | canon(body)
        else:
            state.pop(graph, None)
    return state


def run_crud(engine, ops, resources):
    """Replays the stream (timed per op class), then verifies EVERY graph the
    reference model predicts — present graphs must match exactly, deleted
    graphs must read absent. Returns (ok, detail, lat)."""
    stream = crud_stream(ops, resources)
    lat = {"put": [], "post": [], "delete": []}
    for op, graph, body in stream:
        t = time.perf_counter()
        if op == "put":
            status, _ = engine.put(graph, body)
        elif op == "post":
            status, _ = engine.post(graph, body)
        else:
            status, _ = engine.delete(graph)
        lat[op].append((time.perf_counter() - t) * 1000.0)
        # DELETE of an already-absent graph is a legitimate 404 in the stream
        # (a delete op can precede any put for that resource's acl graph).
        if status >= 400 and not (op == "delete" and status == 404):
            return False, f"{op} <{graph}> failed HTTP {status}", lat

    expected = crud_reference_state(stream)
    touched = {g for _, g, _ in stream}
    for graph in sorted(touched):
        status, body = engine.get(graph)
        if graph in expected:
            if status != 200:
                return False, f"final GET <{graph}> HTTP {status}", lat
            got = canon(body.decode(errors="replace"))
            if got != expected[graph]:
                return (
                    False,
                    f"final state mismatch on <{graph}>: expected "
                    f"{len(expected[graph])} triples, got {len(got)}",
                    lat,
                )
        elif not absent(status, body):
            return False, f"deleted <{graph}> still readable (HTTP {status})", lat
    for v in lat.values():
        v.sort()
    return True, f"{len(stream)} GSP ops, final state matches reference model", lat


# --------------------------------------------------------------------------- #
# Reporting
# --------------------------------------------------------------------------- #
def pct(sorted_ms, p):
    if not sorted_ms:
        return float("nan")
    idx = min(int(len(sorted_ms) * p), len(sorted_ms) - 1)
    return sorted_ms[idx]


def label_of(engine):
    return f"{engine.name} (LOOSE: Solid/LDP stack)" if engine.loose else engine.name


def print_lat_rows(prefix, label, lat):
    for op, times in lat.items():
        if times:
            print(
                f"{prefix:<18} {label:<34} {op:<7} n={len(times):<5}"
                f" p50={pct(times, 0.5):>9.2f}ms p99={pct(times, 0.99):>9.2f}ms"
            )


# --------------------------------------------------------------------------- #
# Hermetic self-test (no HTTP, no server)
# --------------------------------------------------------------------------- #
def self_test():
    failures = []

    def check(name, cond):
        if not cond:
            failures.append(name)

    # gen_ntriples: deterministic, >= target, bnode-free, canon-stable
    a, b = gen_ntriples(1024, "x"), gen_ntriples(1024, "x")
    check("gen deterministic", a == b)
    check("gen size floor", len(a) >= 1024)
    check("gen tag-distinct", gen_ntriples(1024, "y") != a)
    check("gen bnode-free", "_:" not in a)
    check("canon set equality", canon(a) == canon(a + "\n# comment\n\n"))
    check("canon counts lines", len(canon(a)) == a.count("\n"))

    # loopback guard: accepts loopback hosts, refuses others
    for ok_url in (
        "http://127.0.0.1:7033",
        "http://localhost:3030/ds",
        "http://[::1]:7878",
    ):
        check(f"loopback accept {ok_url}", assert_loopback(ok_url, "t") == ok_url)
    for bad_url in ("http://10.0.0.5:3030", "http://example.org/ds"):
        try:
            assert_loopback(bad_url, "t")
            check(f"loopback reject {bad_url}", False)
        except SystemExit:
            pass

    # absent(): both legitimate absent-graph read shapes; a non-empty 200 is present
    check("absent 404", absent(404, b""))
    check("absent empty 200", absent(200, b"\n"))
    check("present 200", not absent(200, b"<http://a> <http://b> <http://c> .\n"))

    # crud reference model: cycle over 1 resource = put(v1), acl, put(v2), delete
    stream = crud_stream(4, resources=1)
    state = crud_reference_state(stream)
    check("crud doc deleted", doc_graph(0) not in state)
    check("crud acl deleted", acl_graph(0) not in state)
    check("crud container survives", state.get(CONTAINER) == canon(containment_triple(0)))
    # one op fewer: the delete never runs -> the doc graph holds the LAST put's
    # triples (i=2 with resources=1 carries version i//resources+1 = 3)
    state3 = crud_reference_state(crud_stream(3, resources=1))
    check("crud doc present", state3.get(doc_graph(0)) == canon(doc_triples(0, 3)))

    # NON-VACUOUSNESS of the round-trip gate: an in-memory fake store passes it,
    # and a tampering variant (drops one triple on read) must FAIL it — this
    # exercises the REAL gate_roundtrip()/run_crud() logic, no HTTP needed.
    class _FakeStore(GspEngine):
        def __init__(self):
            super().__init__("fake", "http://127.0.0.1:0/gsp")
            self.graphs = {}

        def put(self, graph_iri, nt_body):
            created = graph_iri not in self.graphs
            self.graphs[graph_iri] = canon(nt_body)
            return (201 if created else 204), b""

        def post(self, graph_iri, nt_body):
            self.graphs[graph_iri] = self.graphs.get(graph_iri, frozenset()) | canon(
                nt_body
            )
            return 204, b""

        def get(self, graph_iri):
            if graph_iri not in self.graphs:
                return 404, b""
            return 200, "\n".join(sorted(self.graphs[graph_iri])).encode()

        def delete(self, graph_iri):
            if graph_iri not in self.graphs:
                return 404, b""
            del self.graphs[graph_iri]
            return 204, b""

    class _TamperingStore(_FakeStore):
        def get(self, graph_iri):
            status, body = super().get(graph_iri)
            lines = body.decode().splitlines()
            return status, "\n".join(lines[1:]).encode()  # drop one triple

    ok, _ = gate_roundtrip(_FakeStore(), 512)
    check("gate passes honest store", ok)
    ok, detail = gate_roundtrip(_TamperingStore(), 512)
    check("gate catches tampered read", not ok and "mismatch" in detail)
    ok, detail, _ = run_crud(_FakeStore(), 8, 2)
    check("crud gate passes honest store", ok)
    ok, _, _ = run_crud(_TamperingStore(), 8, 2)
    check("crud gate catches tampered read", not ok)

    # URL building: graph IRI is percent-encoded into the query string
    eng = GspEngine("t", "http://127.0.0.1:1/sparql/graph")
    check(
        "gsp url encodes graph",
        eng._url("http://g/x?y=1")
        == "http://127.0.0.1:1/sparql/graph?graph=http%3A%2F%2Fg%2Fx%3Fy%3D1",
    )
    css = CssEngine("css", "http://127.0.0.1:1/pod")
    check("css url is a resource path", "/pod/gsp-bench/" in css._url("http://g/x"))
    check("css marked loose", css.loose and not eng.loose)

    if failures:
        print(f"self-test FAILED: {failures}", file=sys.stderr)
        return 1
    print("self-test OK (gen/canon/loopback/absent/crud-model/url checks)")
    return 0


# --------------------------------------------------------------------------- #
def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--sparq", default="http://127.0.0.1:7033")
    ap.add_argument("--fuseki", default=None, help="Fuseki dataset base URL")
    ap.add_argument("--oxigraph", default=None, help="Oxigraph server base URL")
    ap.add_argument("--css", default=None, help="CSS pod base URL (LOOSE column)")
    ap.add_argument("--sizes", default=None, help=f"CSV bytes (default {DEFAULT_SIZES})")
    ap.add_argument("--iters", type=int, default=None, help="timed iterations/size")
    ap.add_argument("--crud-ops", type=int, default=None, help="CRUD stream length")
    ap.add_argument("--crud-resources", type=int, default=8)
    ap.add_argument("--smoke", action="store_true", help="small sizes + few iters")
    ap.add_argument("--json-out", default=None)
    ap.add_argument("--self-test", action="store_true", help="hermetic checks, no HTTP")
    args = ap.parse_args()

    if args.self_test:
        sys.exit(self_test())

    sizes_csv = args.sizes or (SMOKE_SIZES if args.smoke else DEFAULT_SIZES)
    sizes = [int(s) for s in sizes_csv.split(",") if s.strip()]
    iters = args.iters if args.iters is not None else (3 if args.smoke else 10)
    crud_ops = args.crud_ops if args.crud_ops is not None else (24 if args.smoke else 200)

    engines = []
    if args.sparq:
        base = assert_loopback(args.sparq, "sparq").rstrip("/")
        engines.append(GspEngine("sparq", f"{base}/sparql/graph"))
    if args.fuseki:
        base = assert_loopback(args.fuseki, "fuseki").rstrip("/")
        engines.append(GspEngine("fuseki", f"{base}/data"))
    if args.oxigraph:
        base = assert_loopback(args.oxigraph, "oxigraph").rstrip("/")
        engines.append(GspEngine("oxigraph", f"{base}/store"))
    if args.css:
        engines.append(CssEngine("css", assert_loopback(args.css, "css")))
    if not engines:
        sys.exit("no engine URLs given")

    print(
        f"# GSP same-box panel — sizes={sizes} iters={iters} crud_ops={crud_ops}"
        f" engines={[label_of(e) for e in engines]}"
    )
    results = {}
    any_gate_failed = False
    for engine in engines:
        label = label_of(engine)
        eres = {"loose": engine.loose, "sweep": {}, "crud": {}}
        for size in sizes:
            ok, detail = gate_roundtrip(engine, size)
            print(f"gate[{size}B]        {label:<34} {'OK' if ok else 'FAIL'}: {detail}")
            entry = {"gate": "ok" if ok else "fail", "gate_detail": detail}
            if ok:
                try:
                    lat = time_sweep(engine, size, iters)
                    print_lat_rows(f"sweep[{size}B]", label, lat)
                    entry["latency_ms"] = {
                        op: {"p50": round(pct(t, 0.5), 3), "p99": round(pct(t, 0.99), 3)}
                        for op, t in lat.items()
                    }
                except RuntimeError as e:
                    entry["gate"] = "fail"
                    entry["gate_detail"] = str(e)
                    print(f"sweep[{size}B]      {label:<34} FAIL: {e}")
            if entry["gate"] != "ok":
                any_gate_failed = True
            eres["sweep"][str(size)] = entry

        ok, detail, lat = run_crud(engine, crud_ops, args.crud_resources)
        print(f"gate[crud]         {label:<34} {'OK' if ok else 'FAIL'}: {detail}")
        eres["crud"] = {"gate": "ok" if ok else "fail", "gate_detail": detail}
        if ok:
            print_lat_rows("crud", label, lat)
            eres["crud"]["latency_ms"] = {
                op: {"p50": round(pct(t, 0.5), 3), "p99": round(pct(t, 0.99), 3)}
                for op, t in lat.items()
                if t
            }
        else:
            any_gate_failed = True
        results[engine.name] = eres

    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as fh:
            json.dump(
                {"sizes": sizes, "iters": iters, "crud_ops": crud_ops, "engines": results},
                fh,
                indent=2,
            )
            fh.write("\n")

    if any_gate_failed:
        sys.exit("round-trip correctness gate FAILED for at least one engine/workload")
    print("# all round-trip gates green")


if __name__ == "__main__":
    main()
