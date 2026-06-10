# sparq-server

A **W3C-conformant HTTP server** exposing the [sparq](../../README.md) query engine.
Implements the **read** side of two W3C specifications over an in-memory
[`sparq_core::Graph`]:

* **[SPARQL 1.1 Protocol](https://www.w3.org/TR/sparql11-protocol/)** — the `query`
  operation.
* **[SPARQL 1.1 Graph Store HTTP Protocol](https://www.w3.org/TR/sparql11-http-rdf-update/)**
  — `GET`/`HEAD` on a graph resource.

The **write** side (SPARQL Update, and Graph Store `PUT`/`POST`/`DELETE`) is the separate
**T11b** thread and is intentionally answered with `501 Not Implemented` here.

## Running

```sh
# build (native; the server stack is behind the default-on `server` feature)
cargo build -p sparq-server

# serve a Turtle file on the default address 127.0.0.1:3030
cargo run -p sparq-server -- --format turtle data.ttl

# custom address / format (turtle | ntriples | nquads | trig)
cargo run -p sparq-server -- --addr 0.0.0.0:8080 --format ntriples data.nt

# no data file => empty default graph (still answers queries)
cargo run -p sparq-server
```

## Hardening flags (T15)

The endpoint ships with guards that make it safe to expose publicly. Each flag overrides
the matching `SPARQ_*` environment variable; the environment overrides the default.

| Flag | Env var | Default | Guard |
| --- | --- | --- | --- |
| `--query-timeout SECS` | `SPARQ_QUERY_TIMEOUT` | `30` (`0` disables) | Per-request query timeout. The engine evaluates under a cooperative budget and aborts at its next coarse check; the response is `503` with a JSON body. A hard await-cap of *timeout + 2 s* guarantees the `503` even if the engine is inside an uninstrumented stretch (the detached worker still stops at its next budget check). |
| `--max-body-bytes N` | `SPARQ_MAX_BODY_BYTES` | `1048576` (1 MiB) | Maximum request body. Larger bodies are rejected with `413` before the handler reads them. |
| `--max-concurrent N` | `SPARQ_MAX_CONCURRENT` | `32` | Maximum in-flight requests (all routes). Excess requests are load-shed immediately with `429` instead of queueing unboundedly. |
| `--max-results N` | `SPARQ_MAX_RESULTS` | unlimited (`0` disables) | Maximum SELECT result rows, enforced inside the engine via the row budget. Exceeding it is an **honest `413` refusal** (with the limit named in the error) — never a silent truncation. It is a *working-set* bound: a query whose intermediate result exceeds the cap is refused even if a later operator would shrink it (add `LIMIT`/aggregation to stay under). ASK is existence-only and ignores it. |
| `--max-subscriptions N` | `SPARQ_MAX_SUBSCRIPTIONS` | `256` | Maximum active subscriptions server-wide (T23); further `subscribe` requests are refused with a protocol `error`. |
| `--max-subscriptions-per-conn N` | `SPARQ_MAX_SUBSCRIPTIONS_PER_CONN` | `16` | Maximum active subscriptions per WebSocket connection (T23). |
| `--verbose` | — | off | Per-request logging via `tower_http::trace::TraceLayer` (respects `RUST_LOG`). |

Robustness, always on:

* **Structured JSON error bodies** — every error response is `{"error": "..."}` with
  `Content-Type: application/json` (headers such as `Allow` on a `405` are preserved).
* **Panic recovery** — a panicking handler returns `500` (JSON body) instead of a dead
  connection (`CatchPanicLayer`); a panic on the blocking query pool is mapped to `500`
  through the join error.
* **Graceful shutdown** — `SIGINT`/`SIGTERM` stop accepting connections and drain
  in-flight requests before exit.
* **Off-loop execution** — query evaluation (CPU-bound) runs on the blocking pool, so
  slow queries never stall the async accept/IO workers.

Library users: build the same stack with `AppState::with_config(graph, ServerConfig { … })`
+ `router(state)`, or wrap any router in the middleware via `harden(router, &config)`.

Then, e.g.:

```sh
curl 'http://127.0.0.1:3030/sparql?query=SELECT%20*%20WHERE%20%7B%20%3Fs%20%3Fp%20%3Fo%20%7D%20LIMIT%205'
curl -H 'Accept: text/csv' --data-urlencode 'query=SELECT * WHERE { ?s ?p ?o }' \
     http://127.0.0.1:3030/sparql
curl -H 'Content-Type: application/sparql-query' \
     --data 'ASK { ?s ?p ?o }' http://127.0.0.1:3030/sparql
```

## Tests

```sh
cargo test -p sparq-server
```

Unit tests cover the serialisers, content negotiation, query classification and the
subscription diff/encoding primitives; the `tests/protocol.rs` integration suite spins the
**actual** axum server on an ephemeral port and drives it over real HTTP, asserting
request forms, exact result media types, ASK booleans and HTTP status semantics.
`tests/hardening.rs` exercises every T15 guard the same way: timeout on a deliberately
slow query, body-size `413`, concurrency shed `429`, panic→`500`, the `--max-results`
refusal and the structured JSON error bodies. `tests/subscriptions.rs` drives the T23
WebSocket protocol end to end with `tokio-tungstenite`: initial result, update→diff,
silence on non-matching updates, unsubscribe, both subscription limits, the oversized
refusal and slot cleanup after a dropped socket.

## Endpoints

### SPARQL 1.1 Protocol — `query`

`/sparql`

| Request form | How |
| --- | --- |
| `GET` | `?query=<encoded>` (+ optional `default-graph-uri` / `named-graph-uri`) |
| `POST` direct | `Content-Type: application/sparql-query`, body = the query |
| `POST` url-encoded | `Content-Type: application/x-www-form-urlencoded`, `query=…` in body |
| `HEAD` | same as `GET` but no body (Content-Type + Content-Length preserved) |

### Graph Store HTTP Protocol — read

| Resource | How |
| --- | --- |
| Indirect | `GET /sparql/graph?default` or `GET /sparql/graph?graph=<iri>` |
| Direct | `GET /graphs/<path>` |

`GET`/`HEAD` serialise the graph as **N-Triples** (also offered for an `Accept: text/turtle`
request, since N-Triples is a syntactic subset of Turtle). Write verbs → `501`.

### SPARQL subscriptions — `ws://…/subscriptions` (T23)

SEPA-style live queries over WebSocket: register a SELECT once, receive
**added/removed bindings diffs** (each a full SPARQL JSON results object) after every
committed update that changes the result. JSON protocol, limits, the
re-evaluate+diff/coalescing model and the SEPA lineage/divergences are documented in
[`SUBSCRIPTIONS.md`](SUBSCRIPTIONS.md).

```text
client:  {"subscribe": {"query": "SELECT ?s WHERE { ?s <http://ex/age> ?o }"}}
server:  {"subscribed": {"id": 1}}
server:  {"notification": {"id": 1, "sequence": 0, "addedResults": {…full result…}, "removedResults": {…empty…}}}
         …POST /sparql update commits…
server:  {"notification": {"id": 1, "sequence": 1, "addedResults": {…}, "removedResults": {…}}}
client:  {"unsubscribe": {"id": 1}}
server:  {"unsubscribed": {"id": 1}}
```

## Result formats + conformance status

| Form | `Accept` | Content-Type | Status |
| --- | --- | --- | --- |
| SELECT | `application/sparql-results+json` (default) | `application/sparql-results+json` | conformant (engine native fast path) |
| SELECT | `application/sparql-results+xml` | `application/sparql-results+xml` | conformant — [XML results](https://www.w3.org/TR/rdf-sparql-XMLres/) |
| SELECT | `text/csv` | `text/csv; charset=utf-8` | conformant — [CSV/TSV](https://www.w3.org/TR/sparql11-results-csv-tsv/) (RFC 4180 quoting, CRLF) |
| SELECT | `text/tab-separated-values` | `text/tab-separated-values; charset=utf-8` | conformant — TSV term syntax + escaping |
| ASK | json (default) / xml | `application/sparql-results+json` / `+xml` | conformant boolean |
| CONSTRUCT / DESCRIBE | `application/n-triples` (default) / `text/turtle` | `application/n-triples` / `text/turtle` `; charset=utf-8` | conformant graph result (T16); the body is N-Triples — a syntactic subset of Turtle — under either type |

Negotiation is q-value aware; unsupported / absent `Accept` defaults to JSON (SELECT/ASK)
or N-Triples (CONSTRUCT/DESCRIBE).

**Streamed SELECT bodies (T16).** JSON SELECT results are evaluated to an ordered chunk
sequence inside the engine and streamed to the socket chunk by chunk instead of being
concatenated into one giant `String` first. The bytes, `Content-Type` and
`Content-Length` are identical to the buffered form (the length is known before the
response starts); the win is peak memory — the server never holds a *second* whole-result
copy. Measured on a 1M-row `SELECT * WHERE { ?s ?p ?o }` (202 MB JSON body): peak RSS
~750 MB → ~405–570 MB.

**DESCRIBE semantics.** The engine returns the **concise bounded description** (CBD) of
each described resource: all triples with the resource as subject, recursing through
blank-node objects. The SPARQL spec leaves the DESCRIBE result form to the
implementation; CBD is the de-facto standard choice.

### HTTP status semantics

| Condition | Status |
| --- | --- |
| Success | `200` (correct `Content-Type`) |
| Malformed query / missing `query` param | `400` |
| Query execution error | `500` |
| Unsupported method on `/sparql` | `405` + `Allow: GET, HEAD, POST` |
| `POST` with an unsupported `Content-Type` | `415` |
| Request body over `--max-body-bytes` | `413` |
| SELECT result over `--max-results` | `413` (honest refusal, see hardening flags) |
| Concurrency limit reached | `429` |
| Query timed out (`--query-timeout`) | `503` |
| SPARQL Update / Graph Store write | `501` (thread T11b) |

All error bodies are structured JSON: `{"error": "..."}`.

## Limitations / follow-ups

* **Named graphs.** The engine has a single default graph and no named-graph store
  (`GRAPH` patterns error at execution; `FROM` / `FROM NAMED` are ignored). The protocol's
  `default-graph-uri` / `named-graph-uri` params and the Graph Store *named*-graph
  selectors are **accepted and threaded through** but, with one default graph, have no
  effect — every graph resource maps onto the default graph. This needs roadmap **T9**
  (named-graph storage) before it can be made fully conformant.
* **CONSTRUCT / DESCRIBE serialisations.** Implemented (T16) via the engine's RDF-graph
  result API (`sparq_engine::construct` / `describe`), negotiated between
  `application/n-triples` and `text/turtle` (the body is N-Triples either way — valid
  Turtle). `application/rdf+xml` and a prefix-compacting Turtle writer are follow-ups.
* **SPARQL Update + Graph Store write.** Out of scope here (thread **T11b**, gated on the
  T10 mutable-store work); `501` with a clear message.
* **ASK** is implemented by rewriting the `ASK` to a `SELECT *` over the same pattern and
  testing for any solution — exact, and reuses the engine's verified evaluation, but does
  not yet short-circuit on the first match (it uses the engine's `count`).

## Running the official W3C conformance suite

The endpoints are shaped so the
[W3C SPARQL 1.1 Protocol test suite](https://www.w3.org/2009/sparql/docs/tests/) can be
pointed at a running server:

1. Start the server against the suite's data:
   `cargo run -p sparq-server -- --format turtle <suite-data>.ttl`
2. Configure the harness's service endpoint to `http://127.0.0.1:3030/sparql`.
3. The **query** (including CONSTRUCT/DESCRIBE), **result-format** and **HTTP-semantics**
   sections are expected to pass; **named-graph dataset** tests are expected to report
   not-implemented per the limitations above.

The in-process `tests/protocol.rs` suite mirrors the same assertions and runs in CI via
`cargo test -p sparq-server`.
