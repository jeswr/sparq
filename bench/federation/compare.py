#!/usr/bin/env python3
# [FABLE-5] sq-hmd7l.12 — first comparative FEDERATION harness (FedShop-shaped).
#
# Drives a same-box federation panel: a FedShop-shaped shop federation (vendor +
# ratingsite member datasets, generated in-repo, deterministic) is served by LOCAL
# sparq-server member endpoints; three federating engines execute the SAME
# FedShop-shaped federated queries over them:
#
#   sparq    — sparq-server built `--features service` (the engine's SPARQL 1.1
#              SERVICE eval + bound-join pushdown), queried over HTTP /sparql.
#   comunica — @comunica/query-sparql (the reference federated-SPARQL JS engine),
#              via comunica_runner.mjs (gather-time npm install, never committed).
#   jena     — Apache Jena `arq` executing the same explicit-SERVICE query over an
#              empty default graph (the naive SERVICE baseline). OPTIONAL column:
#              set FED_JENA_ARQ=/path/to/apache-jena/bin/arq; honest n/a when unset.
#
# INVARIANT (bead sq-hmd7l.12): per query, the engines' RESULT SETS (canonical
# binding multisets, not just counts) must AGREE before ANY timing is reported;
# per-member HTTP REQUEST COUNTS and SOURCE-SELECTION precision/recall are always
# reported alongside wall time — never wall time alone.
#
# Request counts come from an in-process counting reverse proxy in front of EACH
# member endpoint (every engine is pointed at the proxies, so the counts are
# uniform across engines). Source-selection ground truth is measured, not assumed:
# a member is RELEVANT to a query iff at least one of the query's member-block
# patterns yields >=1 row when executed directly against that member (probed on
# the member's REAL port, so ground-truth probes never pollute the proxy counts).
#
# Two federation regimes:
#   explicit — the federated query names each member in a SERVICE <proxy-url>
#              clause (all three engines). Source selection is trivially bounded
#              by the query text; the counts still expose join strategy (e.g.
#              bound-join batching vs per-binding requests).
#   virtual  — the SERVICE-free conjunction of the same blocks, executed by
#              Comunica over sources=[all members] (its native federated mode:
#              the engine does source selection). sparq has no server-exposed
#              virtual-federation endpoint today (sparq-fedclient is a library)
#              — that column is an honest n/a (follow-up bead).
#
# Timing regimes differ by engine NATURE and are recorded, not hidden:
#   sparq: HTTP round-trip to the federator (server regime, includes result
#          serialisation); comunica: engine-internal exec time reported by the
#          runner (library regime, process startup excluded); jena: arq process
#          wall time INCLUDING JVM startup (flagged jvm_included in the JSON).
#
# NON-goals (v1, honest): no dockerized upstream FedShop distribution (the corpus
# is FedShop-SHAPED, generated in-repo — upstream FedShop RUNS its members in
# docker; same-box local replicas are the variance-controlled equivalent this
# panel wants); LargeRDFBench's public endpoints are DEAD (local replicas only);
# SolidBench is a follow-up note in bench/federation/README.md.
#
# Exit contract: 0 = every attempted query's oracle agreed (engines that cannot
# execute a construct record an honest per-query "error" cell and are excluded
# from that query's oracle — but sparq-vs-comunica agreement on >=1 query is
# REQUIRED, and any executed-but-DISAGREEING pair is a hard failure).
#
# --self-test runs the hermetic unit layer (no HTTP, no node, no server binary).
from __future__ import annotations

import argparse
import http.client
import http.server
import json
import os
import random
import shutil
import socket
import subprocess
import sys
import threading
import time
import urllib.parse
import urllib.request

FS = "http://sparq.dev/fedshop#"
PREFIXES = (
    "PREFIX fs: <http://sparq.dev/fedshop#>\n"
    "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n"
    "PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>\n"
)
XSD_INTEGER = "http://www.w3.org/2001/XMLSchema#integer"
XSD_STRING = "http://www.w3.org/2001/XMLSchema#string"

MEMBERS = ["vendor", "ratingsite"]

# ─── FedShop-shaped federated queries ────────────────────────────────────────────────
# Each query is a list of member BLOCKS; the harness renders the explicit-SERVICE
# form (SERVICE <member-proxy> { block } per block) and the virtual SERVICE-free
# form ({ block } conjunction) from the same manifest — one source of truth, so the
# two regimes are the same algebra modulo federation.
QUERIES = [
    {
        "id": "q01-offer-review-join",
        "desc": "cheap well-rated products: offers (vendor) join reviews (ratingsite) on ?product",
        "select": "?product ?price ?rating",
        "blocks": [
            {
                "member": "vendor",
                "pattern": "?offer fs:product ?product ; fs:price ?price . FILTER(?price <= 40)",
            },
            {
                "member": "ratingsite",
                "pattern": "?review fs:reviewFor ?product ; fs:rating ?rating . FILTER(?rating >= 8)",
            },
        ],
    },
    {
        "id": "q02-product-detail",
        "desc": "every offer + every review of one product (highly selective cross-member star)",
        "select": "?offer ?price ?review ?rating",
        "blocks": [
            {
                "member": "vendor",
                "pattern": "?offer fs:product fs:Product1 ; fs:price ?price .",
            },
            {
                "member": "ratingsite",
                "pattern": "?review fs:reviewFor fs:Product1 ; fs:rating ?rating .",
            },
        ],
    },
    {
        "id": "q03-unreviewed-offers",
        "desc": "cheap offered products with NO review (OPTIONAL SERVICE + !bound anti-join)",
        "select": "?product ?price",
        "blocks": [
            {
                "member": "vendor",
                "pattern": "?offer fs:product ?product ; fs:price ?price . FILTER(?price <= 30)",
            },
            {
                "member": "ratingsite",
                "pattern": "?review fs:reviewFor ?product .",
                "optional": True,
            },
        ],
        "outer_filter": "FILTER(!bound(?review))",
    },
    {
        "id": "q04-vendor-only",
        "desc": "single-member query (only the vendor member is relevant — source-selection probe)",
        "select": "?offer ?product ?price",
        "blocks": [
            {
                "member": "vendor",
                "pattern": "?offer fs:product ?product ; fs:price ?price ; fs:vendor fs:Vendor0 . FILTER(?price <= 20)",
            },
        ],
    },
    {
        "id": "q05-vendor0-reviews",
        "desc": "reviews of Vendor0's products (selective left side -> bound-join request-count probe)",
        "select": "?product ?review ?rating",
        "blocks": [
            {
                "member": "vendor",
                "pattern": "?offer fs:product ?product ; fs:vendor fs:Vendor0 .",
            },
            {
                "member": "ratingsite",
                "pattern": "?review fs:reviewFor ?product ; fs:rating ?rating . FILTER(?rating <= 3)",
            },
        ],
    },
]


# ─── Deterministic FedShop-shaped member data ───────────────────────────────────────
def gen_member_data(member: str, n_products: int, seed: int = 42) -> list[str]:
    """N-Triples lines for one member of the shop federation. Deterministic in
    (member, n_products, seed). Product IRIs are SHARED across members (the join
    keys); offer/review subjects are member-local. ~10% of products have no offer
    and ~1/7 have no review, so exclusive-to-one-member products exist (q03/q04
    are non-empty) alongside the joinable overlap (q01/q02/q05 are non-empty)."""
    # str seeding is hash-stable (seeded via sha512, PYTHONHASHSEED-independent).
    rng = random.Random(f"{seed}:{member}")
    out: list[str] = []

    def t(s: str, p: str, o: str) -> None:
        out.append(f"{s} {p} {o} .")

    def product(i: int) -> str:
        return f"<{FS}Product{i}>"

    if member == "vendor":
        n_offer = 0
        for i in range(n_products):
            if i % 10 == 9 and i != 1:  # ~10% offer-less (Product1 always offered: q02)
                continue
            for _ in range(1 + (rng.random() < 0.5)):  # 1-2 offers per offered product
                offer = f"<{FS}Offer{n_offer}>"
                n_offer += 1
                vendor = f"<{FS}Vendor{rng.randrange(2)}>"
                price = rng.randrange(5, 100)
                t(offer, f"<{FS}product>", product(i))
                t(offer, f"<{FS}price>", f'"{price}"^^<{XSD_INTEGER}>')
                t(offer, f"<{FS}vendor>", vendor)
            t(product(i), "<http://www.w3.org/2000/01/rdf-schema#label>", f'"product {i}"')
    elif member == "ratingsite":
        n_review = 0
        for i in range(n_products):
            if i % 7 == 6 and i != 1:  # ~1/7 review-less (Product1 always reviewed: q02)
                continue
            for _ in range(1 + rng.randrange(3)):  # 1-3 reviews per reviewed product
                review = f"<{FS}Review{n_review}>"
                n_review += 1
                t(review, f"<{FS}reviewFor>", product(i))
                t(review, f"<{FS}rating>", f'"{rng.randrange(1, 11)}"^^<{XSD_INTEGER}>')
                t(review, f"<{FS}reviewer>", f"<{FS}Reviewer{rng.randrange(20)}>")
    else:
        raise ValueError(f"unknown member: {member}")
    return out


# ─── Query rendering (explicit-SERVICE vs virtual) ───────────────────────────────────
def render_query(query: dict, mode: str, member_urls: dict[str, str] | None = None) -> str:
    """Render one manifest query. mode='explicit' wraps each block in
    SERVICE <member_urls[member]> {...}; mode='virtual' emits the SERVICE-free
    group conjunction (each block keeps its own group braces, so FILTER scope is
    identical across the two renderings)."""
    parts: list[str] = []
    for block in query["blocks"]:
        if mode == "explicit":
            if member_urls is None or block["member"] not in member_urls:
                raise ValueError(f"no endpoint URL for member {block['member']!r}")
            group = f"SERVICE <{member_urls[block['member']]}> {{ {block['pattern']} }}"
        elif mode == "virtual":
            group = f"{{ {block['pattern']} }}"
        else:
            raise ValueError(f"unknown mode: {mode}")
        if block.get("optional"):
            group = f"OPTIONAL {{ {group} }}"
        parts.append(group)
    if query.get("outer_filter"):
        parts.append(query["outer_filter"])
    body = "\n  ".join(parts)
    return f"{PREFIXES}SELECT {query['select']} WHERE {{\n  {body}\n}}"


def block_probe_query(block: dict) -> str:
    """Standalone probe for source-selection ground truth: does this member hold
    ANY solution to this block's pattern?"""
    return f"{PREFIXES}SELECT * WHERE {{ {block['pattern']} }} LIMIT 1"


# ─── Canonical result-set agreement (the oracle) ─────────────────────────────────────
def canon_term(term: dict) -> tuple:
    """Canonical comparable form of one SPARQL-results-JSON term. RDF 1.1: a plain
    literal IS an xsd:string literal, so a missing datatype and an explicit
    xsd:string datatype canonicalise identically. bnode labels are NOT comparable
    across engines — the harness's queries never project bnodes; a projected bnode
    canonicalises to a fixed marker so agreement stays label-independent."""
    kind = term.get("type", "")
    if kind in ("uri", "iri"):
        return ("iri", term["value"])
    if kind in ("bnode", "blank"):
        return ("bnode", "_")
    if kind in ("literal", "typed-literal"):
        dt = term.get("datatype") or XSD_STRING
        lang = (term.get("xml:lang") or term.get("lang") or "").lower()
        if lang:
            return ("literal", term["value"], "", lang)
        return ("literal", term["value"], dt, "")
    raise ValueError(f"unknown term kind in results: {term!r}")


def canon_rows(bindings: list[dict]) -> list[tuple]:
    """Canonical MULTISET (sorted list) of solution rows; each row is the sorted
    tuple of (var, canonical-term). Unbound vars are simply absent, matching
    SPARQL-results-JSON semantics in every engine."""
    rows = []
    for b in bindings:
        rows.append(tuple(sorted((var, canon_term(term)) for var, term in b.items())))
    return sorted(rows)


def rows_agree(a: list[dict], b: list[dict]) -> bool:
    return canon_rows(a) == canon_rows(b)


def precision_recall(contacted: set[str], relevant: set[str]) -> tuple[float | None, float | None]:
    p = len(contacted & relevant) / len(contacted) if contacted else None
    r = len(contacted & relevant) / len(relevant) if relevant else None
    return p, r


# ─── Counting reverse proxy (uniform per-member request counts) ──────────────────────
class _CountingProxyHandler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    HOP_BY_HOP = {
        "connection", "keep-alive", "proxy-authenticate", "proxy-authorization",
        "te", "trailers", "transfer-encoding", "upgrade",
    }

    def _forward(self) -> None:
        srv = self.server  # CountingProxy
        with srv.lock:
            srv.count += 1
        body = b""
        length = self.headers.get("Content-Length")
        if length:
            body = self.rfile.read(int(length))
        conn = http.client.HTTPConnection("127.0.0.1", srv.target_port, timeout=60)
        try:
            headers = {}
            for k, v in self.headers.items():
                lk = k.lower()
                # Strip hop-by-hop headers; strip accept-encoding so the upstream
                # answers identity-encoded and the raw body forwards verbatim;
                # rewrite Host to the real member authority.
                if lk in self.HOP_BY_HOP or lk in ("accept-encoding", "host", "content-length"):
                    continue
                headers[k] = v
            headers["Host"] = f"127.0.0.1:{srv.target_port}"
            if body:
                headers["Content-Length"] = str(len(body))
            conn.request(self.command, self.path, body=body or None, headers=headers)
            resp = conn.getresponse()
            payload = resp.read()
            self.send_response(resp.status)
            for k, v in resp.getheaders():
                if k.lower() in self.HOP_BY_HOP or k.lower() == "content-length":
                    continue
                self.send_header(k, v)
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)
        except (OSError, http.client.HTTPException) as e:
            try:
                self.send_error(502, f"upstream member unreachable: {e}")
            except OSError:
                pass
        finally:
            conn.close()

    do_GET = do_POST = do_HEAD = do_OPTIONS = _forward

    def log_message(self, *args):  # quiet
        pass


class CountingProxy(http.server.ThreadingHTTPServer):
    """Reverse proxy 127.0.0.1:port -> 127.0.0.1:target_port counting every request."""

    daemon_threads = True

    def __init__(self, port: int, target_port: int):
        self.target_port = target_port
        self.count = 0
        self.lock = threading.Lock()
        super().__init__(("127.0.0.1", port), _CountingProxyHandler)
        self._thread = threading.Thread(target=self.serve_forever, daemon=True)
        self._thread.start()

    def reset(self) -> None:
        with self.lock:
            self.count = 0

    def snapshot(self) -> int:
        with self.lock:
            return self.count


# ─── Engine runners ───────────────────────────────────────────────────────────────────
def sparql_json_request(endpoint: str, query: str, timeout: float = 120.0) -> dict:
    data = urllib.parse.urlencode({"query": query}).encode()
    req = urllib.request.Request(
        endpoint,
        data=data,
        headers={
            "Content-Type": "application/x-www-form-urlencoded",
            "Accept": "application/sparql-results+json",
        },
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8"))


def run_sparq(endpoint: str, query: str, timeout: float) -> tuple[list[dict], float]:
    """Execute on the sparq federator; returns (bindings, wall_us of the HTTP round trip)."""
    t0 = time.perf_counter()
    doc = sparql_json_request(endpoint, query, timeout)
    wall_us = (time.perf_counter() - t0) * 1e6
    return doc["results"]["bindings"], wall_us


def run_comunica(
    runner: str, query: str, sources: list[str], timeout: float
) -> tuple[list[dict], float, str]:
    """Execute via comunica_runner.mjs; returns (bindings, engine exec_us, version)."""
    proc = subprocess.run(
        ["node", runner, *(f"--source={s}" for s in sources)],
        input=query.encode(),
        capture_output=True,
        timeout=timeout,
        cwd=os.path.dirname(os.path.abspath(runner)),
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"comunica runner failed (exit {proc.returncode}): "
            f"{proc.stderr.decode(errors='replace')[-2000:]}"
        )
    doc = json.loads(proc.stdout.decode("utf-8"))
    return doc["bindings"], doc["exec_ms"] * 1000.0, doc.get("engine_version", "unknown")


def run_jena(arq: str, query: str, timeout: float, workdir: str) -> tuple[list[dict], float]:
    """Execute via Jena arq over an empty default graph (naive SERVICE baseline).
    Wall time INCLUDES JVM startup — flagged jvm_included in the results JSON."""
    qfile = os.path.join(workdir, "jena-query.rq")
    with open(qfile, "w") as f:
        f.write(query)
    t0 = time.perf_counter()
    proc = subprocess.run(
        [arq, "--query", qfile, "--results", "JSON"],
        capture_output=True,
        timeout=timeout,
    )
    wall_us = (time.perf_counter() - t0) * 1e6
    if proc.returncode != 0:
        raise RuntimeError(
            f"arq failed (exit {proc.returncode}): {proc.stderr.decode(errors='replace')[-2000:]}"
        )
    doc = json.loads(proc.stdout.decode("utf-8"))
    return doc["results"]["bindings"], wall_us


# ─── Server lifecycle ────────────────────────────────────────────────────────────────
def wait_ready(endpoint: str, proc: subprocess.Popen, timeout_s: float, what: str) -> None:
    deadline = time.monotonic() + timeout_s
    probe = f"{PREFIXES}SELECT * WHERE {{ ?s ?p ?o }} LIMIT 1"
    while time.monotonic() < deadline:
        if proc.poll() is not None:
            raise RuntimeError(f"{what} exited during startup (code {proc.returncode})")
        try:
            sparql_json_request(endpoint, probe, timeout=5)
            return
        except OSError:
            time.sleep(0.25)
    raise RuntimeError(f"{what} not ready within {timeout_s}s")


def free_port_check(port: int) -> None:
    with socket.socket() as s:
        # SO_REUSEADDR matches the servers' own bind semantics — a previous run's
        # TIME_WAIT connections must not fail the check (only a live LISTENer should).
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try:
            s.bind(("127.0.0.1", port))
        except OSError as e:
            raise RuntimeError(f"port {port} not free (set --port-base): {e}") from e


# ─── The panel ────────────────────────────────────────────────────────────────────────
def run_panel(args) -> int:
    queries = QUERIES[:1] if args.smoke else QUERIES
    n_products = args.scale
    iters = args.iters
    workdir = args.workdir
    os.makedirs(workdir, exist_ok=True)

    member_port = {m: args.port_base + i for i, m in enumerate(MEMBERS)}
    proxy_port = {m: args.port_base + 10 + i for i, m in enumerate(MEMBERS)}
    fed_port = args.port_base + 20
    for p in [*member_port.values(), *proxy_port.values(), fed_port]:
        free_port_check(p)

    corpus_sha = {}
    import hashlib

    for m in MEMBERS:
        path = os.path.join(workdir, f"{m}.nt")
        data = "\n".join(gen_member_data(m, n_products)) + "\n"
        with open(path, "w") as f:
            f.write(data)
        corpus_sha[m] = hashlib.sha256(data.encode()).hexdigest()
        log(f"member {m}: {data.count(chr(10))} triples -> {path} (sha256 {corpus_sha[m][:16]}…)")

    procs: list[subprocess.Popen] = []
    proxies: dict[str, CountingProxy] = {}
    results: dict = {
        "suite": "federation-fedshop",
        "mode": "smoke" if args.smoke else "full",
        "scale_products": n_products,
        "iters": iters,
        "corpus_sha256": corpus_sha,
        "members": {
            m: {"endpoint": f"http://127.0.0.1:{proxy_port[m]}/sparql", "backing": "sparq-server"}
            for m in MEMBERS
        },
        "engines": {},
        "queries": [],
        "notes": [
            "NON-canonical unless gathered on a quiet dedicated box (bench/CATALOG.md rules).",
            "timing regimes: sparq=HTTP round-trip to federator; comunica=engine-internal exec"
            " (process startup excluded); jena=arq process wall INCLUDING JVM startup.",
            "explicit mode: SERVICE clauses name the members — source selection is bounded by"
            " the query text; request counts still expose join strategy. virtual mode"
            " (comunica only): engine-side source selection over sources=[all members].",
        ],
    }
    exit_code = 0
    try:
        # 1. members (real sparq-server instances) + counting proxies in front.
        for m in MEMBERS:
            logf = open(os.path.join(workdir, f"{m}.server.log"), "wb")
            proc = subprocess.Popen(
                [
                    args.sparq_server_bin,
                    "--addr",
                    f"127.0.0.1:{member_port[m]}",
                    "--format",
                    "ntriples",
                    "--query-timeout",
                    str(int(args.query_timeout)),
                    os.path.join(workdir, f"{m}.nt"),
                ],
                stdout=logf,
                stderr=logf,
            )
            procs.append(proc)
        for m in MEMBERS:
            wait_ready(
                f"http://127.0.0.1:{member_port[m]}/sparql",
                procs[MEMBERS.index(m)],
                args.ready_timeout,
                f"member {m}",
            )
            proxies[m] = CountingProxy(proxy_port[m], member_port[m])
        log(f"members up: {', '.join(f'{m}:{member_port[m]} (proxy :{proxy_port[m]})' for m in MEMBERS)}")

        # 2. the sparq federator: EMPTY graph, service feature, members allowlisted.
        fed_logf = open(os.path.join(workdir, "federator.server.log"), "wb")
        fed_proc = subprocess.Popen(
            [
                args.sparq_server_bin,
                "--addr",
                f"127.0.0.1:{fed_port}",
                "--query-timeout",
                str(int(args.query_timeout)),
                "--service-allow",
                "127.0.0.1",
            ],
            stdout=fed_logf,
            stderr=fed_logf,
        )
        procs.append(fed_proc)
        fed_endpoint = f"http://127.0.0.1:{fed_port}/sparql"
        wait_ready(fed_endpoint, fed_proc, args.ready_timeout, "sparq federator")
        log(f"sparq federator up at {fed_endpoint} (SERVICE allow: 127.0.0.1)")

        member_urls = {m: f"http://127.0.0.1:{proxy_port[m]}/sparql" for m in MEMBERS}
        engines = ["sparq", "comunica"] + (["jena"] if args.jena_arq else [])
        results["engines"] = {
            "sparq": {"kind": "server", "column": "SERVICE eval + bound-join (sparq-engine/service)"},
            "comunica": {"kind": "js-lib", "column": "@comunica/query-sparql"},
            **(
                {"jena": {"kind": "jvm-cli", "column": "arq explicit SERVICE (naive baseline)", "jvm_included": True}}
                if args.jena_arq
                else {}
            ),
        }
        if not args.jena_arq:
            results["notes"].append("jena column n/a: FED_JENA_ARQ unset (optional naive baseline).")

        agreed_any = False
        for q in queries:
            qres: dict = {"id": q["id"], "desc": q["desc"], "explicit": {}, "virtual": {}}
            results["queries"].append(qres)
            explicit_q = render_query(q, "explicit", member_urls)
            virtual_q = render_query(q, "virtual")

            # Source-selection ground truth: probe each block against EVERY member
            # directly (real ports — the proxies never see the probes).
            relevant: set[str] = set()
            for m in MEMBERS:
                for block in q["blocks"]:
                    doc = sparql_json_request(
                        f"http://127.0.0.1:{member_port[m]}/sparql",
                        block_probe_query(block),
                        args.query_timeout,
                    )
                    if doc["results"]["bindings"]:
                        relevant.add(m)
                        break
            qres["relevant_members"] = sorted(relevant)

            # ORACLE pass (one counted execution per engine) — BEFORE any timing.
            oracle: dict[str, list[dict]] = {}
            for eng in engines:
                for proxy in proxies.values():
                    proxy.reset()
                cell: dict = {}
                qres["explicit"][eng] = cell
                try:
                    if eng == "sparq":
                        bindings, _ = run_sparq(fed_endpoint, explicit_q, args.query_timeout)
                    elif eng == "comunica":
                        bindings, _, ver = run_comunica(
                            args.comunica_runner, explicit_q, [], args.query_timeout
                        )
                        results["engines"]["comunica"]["version"] = ver
                    else:
                        bindings, _ = run_jena(args.jena_arq, explicit_q, args.query_timeout, workdir)
                except (RuntimeError, OSError, subprocess.TimeoutExpired, KeyError, ValueError) as e:
                    cell["status"] = "error"
                    cell["error"] = str(e)[-500:]
                    log(f"{q['id']} [{eng}] ERROR: {str(e)[-200:]}")
                    continue
                oracle[eng] = bindings
                cell["rows"] = len(bindings)
                cell["requests"] = {m: proxies[m].snapshot() for m in MEMBERS}
                contacted = {m for m, c in cell["requests"].items() if c > 0}
                p, r = precision_recall(contacted, relevant)
                cell["contacted_members"] = sorted(contacted)
                cell["source_selection"] = {"precision": p, "recall": r}

            # Agreement: every pair of engines that EXECUTED must agree.
            ok_engines = sorted(oracle)
            disagree = []
            for i, a in enumerate(ok_engines):
                for b in ok_engines[i + 1 :]:
                    if not rows_agree(oracle[a], oracle[b]):
                        disagree.append((a, b))
            qres["oracle"] = {
                "executed": ok_engines,
                "agreed": not disagree and len(ok_engines) >= 2,
                "disagreements": [f"{a} vs {b}" for a, b in disagree],
            }
            if disagree:
                exit_code = 1
                log(f"{q['id']}: ORACLE DISAGREEMENT: {qres['oracle']['disagreements']} — no timing reported")
                continue
            if "sparq" in oracle and "comunica" in oracle:
                agreed_any = True
            if len(ok_engines) < 2:
                log(f"{q['id']}: <2 engines executed — no cross-check possible, no timing reported")
                continue
            log(
                f"{q['id']}: oracle AGREED across {ok_engines} "
                f"({len(canon_rows(oracle[ok_engines[0]]))} rows); relevant={sorted(relevant)}"
            )

            # TIMED pass (only after agreement; counts already recorded above).
            if not args.smoke:
                for eng in ok_engines:
                    walls = []
                    for _ in range(iters):
                        try:
                            if eng == "sparq":
                                _, w = run_sparq(fed_endpoint, explicit_q, args.query_timeout)
                            elif eng == "comunica":
                                _, w, _ = run_comunica(
                                    args.comunica_runner, explicit_q, [], args.query_timeout
                                )
                            else:
                                _, w = run_jena(args.jena_arq, explicit_q, args.query_timeout, workdir)
                        except (RuntimeError, OSError, subprocess.TimeoutExpired) as e:
                            qres["explicit"][eng]["timing_error"] = str(e)[-300:]
                            break
                        walls.append(w)
                    if walls:
                        qres["explicit"][eng]["best_us"] = round(min(walls))
                        qres["explicit"][eng]["iters"] = len(walls)

                # VIRTUAL regime: comunica-native source selection; sparq honest n/a.
                for proxy in proxies.values():
                    proxy.reset()
                vcell: dict = {}
                qres["virtual"]["comunica"] = vcell
                qres["virtual"]["sparq"] = {
                    "status": "n/a",
                    "reason": "no server-exposed virtual-federation endpoint (sparq-fedclient is a library; follow-up bead)",
                }
                try:
                    vbindings, _, _ = run_comunica(
                        args.comunica_runner, virtual_q, list(member_urls.values()), args.query_timeout
                    )
                    vcell["rows"] = len(vbindings)
                    vcell["requests"] = {m: proxies[m].snapshot() for m in MEMBERS}
                    contacted = {m for m, c in vcell["requests"].items() if c > 0}
                    p, r = precision_recall(contacted, relevant)
                    vcell["contacted_members"] = sorted(contacted)
                    vcell["source_selection"] = {"precision": p, "recall": r}
                    vcell["agrees_with_explicit_oracle"] = rows_agree(
                        vbindings, oracle[ok_engines[0]]
                    )
                    if not vcell["agrees_with_explicit_oracle"]:
                        # Honest: virtual-mode divergence is REPORTED, not fatal —
                        # engine-side source selection over a SERVICE-free conjunction
                        # is a different query regime (bnode/dedup semantics can differ).
                        log(f"{q['id']} [comunica virtual]: result set differs from explicit oracle (reported)")
                    walls = []
                    for _ in range(iters):
                        _, w, _ = run_comunica(
                            args.comunica_runner, virtual_q, list(member_urls.values()), args.query_timeout
                        )
                        walls.append(w)
                    vcell["best_us"] = round(min(walls))
                except (RuntimeError, OSError, subprocess.TimeoutExpired) as e:
                    vcell["status"] = "error"
                    vcell["error"] = str(e)[-500:]

        if not agreed_any:
            log("FAIL: no query achieved sparq-vs-comunica result-set agreement")
            exit_code = 1
    finally:
        for proxy in proxies.values():
            proxy.shutdown()
        for proc in procs:
            proc.terminate()
        for proc in procs:
            try:
                proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                proc.kill()

    if args.json_out:
        os.makedirs(os.path.dirname(args.json_out) or ".", exist_ok=True)
        with open(args.json_out, "w") as f:
            json.dump(results, f, indent=2)
        log(f"results JSON -> {args.json_out}")
    print_table(results)
    return exit_code


def print_table(results: dict) -> None:
    print(f"\nfederation-fedshop panel ({results['mode']}, scale={results['scale_products']} products)")
    print(f"{'query':28} {'engine':10} {'rows':>6} {'best_us':>10} {'requests':20} {'src-sel P/R':12} oracle")
    for q in results["queries"]:
        agreed = q.get("oracle", {}).get("agreed")
        for eng, cell in q.get("explicit", {}).items():
            if cell.get("status") == "error":
                print(f"{q['id']:28} {eng:10} {'ERROR':>6}")
                continue
            reqs = ",".join(f"{m}:{c}" for m, c in cell.get("requests", {}).items())
            ss = cell.get("source_selection", {})
            fmt = lambda v: "-" if v is None else f"{v:.2f}"
            print(
                f"{q['id']:28} {eng:10} {cell.get('rows', '-'):>6} "
                f"{cell.get('best_us', '-'):>10} {reqs:20} "
                f"{fmt(ss.get('precision')):>5}/{fmt(ss.get('recall')):<6} "
                f"{'AGREED' if agreed else 'NO'}"
            )
        vc = q.get("virtual", {}).get("comunica")
        if vc and "rows" in vc:
            reqs = ",".join(f"{m}:{c}" for m, c in vc.get("requests", {}).items())
            ss = vc.get("source_selection", {})
            fmt = lambda v: "-" if v is None else f"{v:.2f}"
            print(
                f"{q['id']:28} {'comunica*':10} {vc.get('rows', '-'):>6} "
                f"{vc.get('best_us', '-'):>10} {reqs:20} "
                f"{fmt(ss.get('precision')):>5}/{fmt(ss.get('recall')):<6} "
                f"{'=explicit' if vc.get('agrees_with_explicit_oracle') else 'DIFFERS'}"
            )
    print("(* = virtual regime: engine-side source selection over sources=[all members])\n")


def log(msg: str) -> None:
    print(f"[federation] {msg}", file=sys.stderr)


# ─── Hermetic self-test (no HTTP, no node, no server binary) ─────────────────────────
def self_test() -> int:
    failures: list[str] = []

    def check(name: str, cond: bool) -> None:
        if not cond:
            failures.append(name)

    # Generator: deterministic + member-disjoint local subjects + shared products.
    v1, v2 = gen_member_data("vendor", 50), gen_member_data("vendor", 50)
    r1 = gen_member_data("ratingsite", 50)
    check("gen deterministic", v1 == v2)
    check("gen nonempty", len(v1) > 50 and len(r1) > 50)
    check("gen vendor has offers", any("Offer0>" in line for line in v1))
    check("gen ratingsite has reviews", any("Review0>" in line for line in r1))
    check("gen shares product iris", any("Product1>" in l for l in v1) and any("Product1>" in l for l in r1))
    check(
        "gen scale changes data",
        gen_member_data("vendor", 10) != gen_member_data("vendor", 20),
    )

    # Rendering: explicit names both proxies; virtual names none; FILTER scope kept.
    urls = {"vendor": "http://127.0.0.1:1/sparql", "ratingsite": "http://127.0.0.1:2/sparql"}
    q1 = QUERIES[0]
    exp = render_query(q1, "explicit", urls)
    vir = render_query(q1, "virtual")
    check("explicit has both services", exp.count("SERVICE <") == 2 and urls["vendor"] in exp)
    check("virtual is service-free", "SERVICE" not in vir)
    check("virtual keeps filter scope", vir.count("{") == vir.count("}") and "FILTER" in vir)
    q3 = next(q for q in QUERIES if q["id"].startswith("q03"))
    exp3 = render_query(q3, "explicit", urls)
    check("q03 optional wraps service", "OPTIONAL { SERVICE" in exp3)
    check("q03 outer filter rendered", "!bound(?review)" in exp3)

    # Canonicalisation: RDF 1.1 plain-vs-xsd:string; datatype significance; multiset.
    lit_plain = {"type": "literal", "value": "a"}
    lit_str = {"type": "literal", "value": "a", "datatype": XSD_STRING}
    lit_int = {"type": "literal", "value": "a", "datatype": XSD_INTEGER}
    check("plain == xsd:string", canon_term(lit_plain) == canon_term(lit_str))
    check("datatype significant", canon_term(lit_plain) != canon_term(lit_int))
    check(
        "lang tag case-insensitive",
        canon_term({"type": "literal", "value": "a", "xml:lang": "EN"})
        == canon_term({"type": "literal", "value": "a", "xml:lang": "en"}),
    )
    row_a = {"x": {"type": "uri", "value": "http://e/1"}}
    row_b = {"x": {"type": "uri", "value": "http://e/2"}}
    check("agreement order-insensitive", rows_agree([row_a, row_b], [row_b, row_a]))
    check("agreement is multiset", not rows_agree([row_a], [row_a, row_a]))
    check("agreement value-sensitive", not rows_agree([row_a], [row_b]))
    check(
        "bnode labels not compared",
        rows_agree(
            [{"x": {"type": "bnode", "value": "b0"}}],
            [{"x": {"type": "bnode", "value": "genid-77"}}],
        ),
    )

    # Source-selection math.
    check("precision/recall", precision_recall({"a", "b"}, {"a"}) == (0.5, 1.0))
    check("precision empty contacted", precision_recall(set(), {"a"}) == (None, 0.0))

    # Probe query shape.
    check("probe has limit 1", block_probe_query(q1["blocks"][0]).rstrip().endswith("LIMIT 1"))

    if failures:
        print(f"[federation] self-test FAILED: {failures}", file=sys.stderr)
        return 1
    print("[federation] self-test OK", file=sys.stderr)
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--self-test", action="store_true", help="hermetic unit layer; no HTTP/node/server")
    ap.add_argument("--smoke", action="store_true", help="2 members, one query, oracle only (acceptance)")
    ap.add_argument("--scale", type=int, default=None, help="products in the shop corpus")
    ap.add_argument("--iters", type=int, default=5, help="timed iterations per engine per query")
    ap.add_argument("--port-base", type=int, default=int(os.environ.get("FED_PORT_BASE", "7141")))
    ap.add_argument("--sparq-server-bin", default=os.environ.get("SPARQ_SERVER_BIN", "target/release/sparq-server"))
    ap.add_argument("--comunica-runner", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "comunica_runner.mjs"))
    ap.add_argument("--jena-arq", default=os.environ.get("FED_JENA_ARQ", ""), help="path to Jena arq (optional column)")
    ap.add_argument("--workdir", default=os.environ.get("FED_WORKDIR", "/tmp/sparq-federation-bench"))
    ap.add_argument("--json-out", default="", help="results JSON path (suggest bench/competitor-results/, git-ignored)")
    ap.add_argument("--ready-timeout", type=float, default=120.0)
    ap.add_argument("--query-timeout", type=float, default=120.0)
    args = ap.parse_args()
    if args.self_test:
        return self_test()
    if args.scale is None:
        args.scale = 40 if args.smoke else 500
    return run_panel(args)


if __name__ == "__main__":
    sys.exit(main())
