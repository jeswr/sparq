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

Unit tests cover the serialisers, content negotiation and query classification; the
`tests/protocol.rs` integration suite spins the **actual** axum server on an ephemeral port
and drives it over real HTTP, asserting request forms, exact result media types, ASK
booleans and HTTP status semantics.

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

## Result formats + conformance status

| Form | `Accept` | Content-Type | Status |
| --- | --- | --- | --- |
| SELECT | `application/sparql-results+json` (default) | `application/sparql-results+json` | conformant (engine native fast path) |
| SELECT | `application/sparql-results+xml` | `application/sparql-results+xml` | conformant — [XML results](https://www.w3.org/TR/rdf-sparql-XMLres/) |
| SELECT | `text/csv` | `text/csv; charset=utf-8` | conformant — [CSV/TSV](https://www.w3.org/TR/sparql11-results-csv-tsv/) (RFC 4180 quoting, CRLF) |
| SELECT | `text/tab-separated-values` | `text/tab-separated-values; charset=utf-8` | conformant — TSV term syntax + escaping |
| ASK | json (default) / xml | `application/sparql-results+json` / `+xml` | conformant boolean |

Negotiation is q-value aware; unsupported / absent `Accept` defaults to JSON.

### HTTP status semantics

| Condition | Status |
| --- | --- |
| Success | `200` (correct `Content-Type`) |
| Malformed query / missing `query` param | `400` |
| Query execution error | `500` |
| Unsupported method on `/sparql` | `405` + `Allow: GET, HEAD, POST` |
| `POST` with an unsupported `Content-Type` | `415` |
| SPARQL Update / Graph Store write | `501` (thread T11b) |

## Limitations / follow-ups

* **Named graphs.** The engine has a single default graph and no named-graph store
  (`GRAPH` patterns error at execution; `FROM` / `FROM NAMED` are ignored). The protocol's
  `default-graph-uri` / `named-graph-uri` params and the Graph Store *named*-graph
  selectors are **accepted and threaded through** but, with one default graph, have no
  effect — every graph resource maps onto the default graph. This needs roadmap **T9**
  (named-graph storage) before it can be made fully conformant.
* **CONSTRUCT / DESCRIBE.** Classified correctly, but the engine exposes no RDF-graph
  result API, so they return `501` with an explanatory message. A follow-up once the engine
  gains a triple-result path (it would then negotiate `text/turtle` /
  `application/n-triples` / `application/rdf+xml`).
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
3. The **query**, **result-format** and **HTTP-semantics** sections are expected to pass;
   **update**, **CONSTRUCT/DESCRIBE** and **named-graph dataset** tests are expected to
   report not-implemented per the limitations above.

The in-process `tests/protocol.rs` suite mirrors the same assertions and runs in CI via
`cargo test -p sparq-server`.
