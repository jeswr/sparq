---
name: http-server
description: Run or point an agent at a sparq SPARQL 1.1 Protocol HTTP endpoint (sparq-server) — /sparql query+update over GET/POST, content negotiation (SELECT/ASK JSON/XML/CSV/TSV; CONSTRUCT/DESCRIBE + Graph Store N-Triples/prefix-Turtle/RDF-XML), Graph Store read AND write (PUT/POST/DELETE on graph resources, RDF/XML bodies accepted), EXPLAIN, Prometheus /metrics, WebSocket + SSE subscriptions, and opt-in time-travel ?generation pinning. Use when starting the server, querying/updating a running endpoint, choosing Accept/Content-Type, or embedding the axum router.
---

# sparq-http-server

`sparq-server` is a W3C-conformant HTTP server (axum/tokio) that exposes the sparq
query engine over a `sparq_core::Graph` — in-memory by default, or **durable on disk**
with `--persist DIR` (updates WAL-fsync'd, survive a restart with no rebuild; see the
"Durability" gotcha). It implements the **SPARQL 1.1 Protocol** (`query` + `update` at
`/sparql`) and the **Graph Store HTTP Protocol** (read + write), with `Accept`-driven
content negotiation, hardening guards, Prometheus `/metrics`, WebSocket + SSE subscriptions,
and opt-in time-travel + GeoSPARQL.

## Quickstart

Run the binary (server stack is the default-on `server` feature):

```sh
# serve a Turtle file on the default address 127.0.0.1:3030 (loopback — safe default)
cargo run -p sparq-server -- --format turtle data.ttl
# no data file => empty default graph (still answers queries, just no rows)
cargo run -p sparq-server
# custom bind addr / format (turtle | ntriples | nquads | trig). A NON-loopback bind
# (e.g. 0.0.0.0) is REFUSED unless --allow-remote / SPARQ_ALLOW_REMOTE=1 (no auth — see below).
cargo run -p sparq-server -- --addr 0.0.0.0:8080 --allow-remote --format ntriples data.nt
```

> **Security: optional Bearer-token write gate; loopback by default.** With no token
> configured, every endpoint is unauthenticated (the back-compat default). Set
> `--auth-token <TOKEN>` (env `SPARQ_AUTH_TOKEN`) to require `Authorization: Bearer <TOKEN>`
> on every **write** (a SPARQL Update on `/sparql` — `application/sparql-update`, or a
> `query`/`update` body that parses as an update — and the GSP `PUT`/`POST`/`DELETE`
> methods); otherwise `401` with `WWW-Authenticate: Bearer` (constant-time compared; mirrors
> QLever's `-a <token>`). Add `--auth-token-read` (env `SPARQ_AUTH_TOKEN_READ=1`) to ALSO
> gate reads. The subscription transports — the `/subscriptions` WebSocket and the
> `/subscriptions/sse` SSE stream (both a *read* surface) — are gated by `--auth-token-read`
> too (bead `sq-cxk5`, closing the prior read-auth bypass): the SSE GET takes the
> `Authorization: Bearer` header; the WS upgrade accepts that header OR (for browsers, which
> cannot set headers on a WS handshake) a `Sec-WebSocket-Protocol: bearer.<token>` subprotocol.
> With no read gate, both are open (back-compatible). The server binds **loopback by default** and **refuses a non-loopback
> bind** (e.g. `0.0.0.0`) unless you set `--allow-remote` (env `SPARQ_ALLOW_REMOTE=1`) OR the
> whole surface is authenticated (`--auth-token` AND `--auth-token-read`) — a write-token
> alone still leaves reads open. Deliver the token over TLS (terminate it at a proxy). For
> real per-user authz, front it with a reverse proxy / API gateway (or `sparq-solid`). SPARQL
> `SERVICE` federation is OFF
> in the default build (the `service` cargo feature is off); a `SERVICE` clause then
> errors at execution. Build with `--features service` to enable it — and even then the
> server is **default-DENY-all SERVICE**: a `SERVICE <iri>` reaches **nothing** unless its
> host is on the egress allowlist (`--service-allow` / `--service-allow-file` /
> `SPARQ_SERVICE_ALLOW`; bead `sq-4w18`). This is an SSRF guard: a `SERVICE` clause turns
> attacker-controlled query text into an outbound request from the server host (worst case
> the `169.254.169.254` cloud-metadata IP). The allowlist is enforced before any socket is
> opened, on the *resolved* IP (DNS-rebinding-safe). See "SERVICE federation (egress
> allowlist)" below.
>
> **Security response headers (always on, ASVS V14.4 / ASVS-G1; beads `sq-cmvh`, `sq-2bhm`).**
> Every response — success, streamed, error and auth-gated (`401`) alike — carries a hardening
> header set, stamped by a `map_response` layer in `harden()`:
> `X-Content-Type-Options: nosniff`,
> `Content-Security-Policy: default-src 'none'; frame-ancestors 'none'`,
> `X-Frame-Options: DENY`, and `Referrer-Policy: no-referrer`. These suit a SPARQL *data* API
> (no HTML is rendered): the tightest CSP says the body loads/runs nothing, and `frame-ancestors`
> + `X-Frame-Options: DENY` say it is never meant to be framed. Each header is only added when
> absent, so a custom handler can override one. **Deliberately omitted:** `Strict-Transport-
> Security` (the origin serves plain HTTP — HSTS belongs on the fronting TLS proxy);
> `X-XSS-Protection` (deprecated, superseded by CSP); CORS / `Cross-Origin-*` / `Permissions-
> Policy` (browser-app document policies, meaningless for a no-CORS data API — adding CORS would
> *widen* the surface). No *blanket* `Cache-Control: no-store` is forced: results are uncached by
> default (no `ETag`/`Cache-Control: public` is ever set), so there is nothing to tighten, and a
> blanket value would wrongly override `/health` / `/metrics` — but the sensitive auth-refusal
> (`401` from `unauthorized()`) **does** carry `Cache-Control: no-store` so a shared cache never
> retains it (`sq-2bhm`).
>
> **Error bodies carry a generic class, never internals (ASVS V7 / ASVS-G3; beads `sq-cz89`,
> `sq-j9zs`, [OPUS-4.8] `sq-kfel`).** Every error is the structured `{"error":"<msg>"}` envelope
> where `<msg>` is a STABLE generic category (malformed-query / auth / not-found / server-error) —
> never the caller's input, a loaded-RDF fragment, a server filesystem path, a secret, or a
> `Debug` of an internal type. The full detail goes to the server log under
> `target: "sparq_server"` (gated behind `--verbose` / `RUST_LOG`), not the response body. All
> sensitive error paths funnel through one `sanitized_error` helper; regression-guarded by
> `tests/hardening.rs` (`no_echo_*` + `FORBIDDEN_INTERNALS`) and `tests/tpf.rs`.

Point a client at it (the endpoint is `/sparql`):

```sh
# GET, query as URL param (URL-encoded); default result media is SPARQL-JSON
curl -G http://127.0.0.1:3030/sparql --data-urlencode 'query=SELECT * WHERE { ?s ?p ?o } LIMIT 5'

# POST direct: body IS the query
curl http://127.0.0.1:3030/sparql -H 'Content-Type: application/sparql-query' \
     --data 'ASK { ?s ?p ?o }'

# POST url-encoded form, negotiate CSV
curl http://127.0.0.1:3030/sparql -H 'Accept: text/csv' \
     --data-urlencode 'query=SELECT * WHERE { ?s ?p ?o }'

# SPARQL Update -> 204 No Content (atomic; failure -> 400, no partial effect)
curl -i http://127.0.0.1:3030/sparql -H 'Content-Type: application/sparql-update' \
     --data 'INSERT DATA { <http://ex/a> <http://ex/p> <http://ex/b> }'
```

Embed the router in your own tokio app (library use):

```rust
use sparq_core::Graph;
use sparq_server::{router, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let graph = Graph::load_str(include_str!("data.ttl"), "turtle")?;
    let app = router(AppState::new(graph));          // axum::Router
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3030").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

## Key APIs

Library surface re-exported from `sparq_server` (behind the default `server` feature):

- `fn router(state: AppState) -> axum::Router` — builds the hardened endpoint router
  (`/sparql`, `/sparql/graph`, `/graphs/*path`, `/subscriptions`, `/subscriptions/sse`,
  `/health`, `/metrics`). [OPUS-4.8] sq-bxog: `/subscriptions/sse` is the SSE transport.
- `AppState::new(graph: Graph) -> AppState` — default `ServerConfig`.
- `AppState::with_config(graph: Graph, config: ServerConfig) -> AppState`.
- `AppState::current(&self) -> PinnedGen` — lock-free pin of the current immutable
  generation snapshot (`PinnedGen = Arc<sparq_serve::Generation<Graph>>`; call
  `.snapshot() -> &Graph`, `.number() -> u64`, `.published_at()`, `.epochs()`).
- `AppState::apply_update(&self, sparql: &str) -> Result<u64, String>` — submit a SPARQL
  Update through the sequenced group-commit writer; **blocks** until the containing
  generation is published; returns that generation number (read-your-writes token). Call
  off the async workers (`spawn_blocking`).
- `AppState::at(&self, number: u64) -> Option<PinnedGen>` — pin a retained generation
  (**`time-travel` feature only**).
- `struct ServerConfig { query_timeout: Option<Duration>, update_where_timeout: Option<Duration>,
  max_body_bytes: usize,
  max_concurrent: usize, header_read_timeout: Option<Duration>, max_results: Option<usize>, max_query_rows: Option<usize>,
  max_query_bytes: Option<usize>,
  max_decompress_ratio: usize, max_subscriptions: usize, max_subscriptions_per_conn: usize,
  verbose: bool, redact_logs: bool, allow_remote: bool, auth_token: Option<String>, auth_token_read: bool,
  service_allow: ServiceAllowlist, /* + time_travel_* under feature, + audit_log under audit-log feature */ }` with
  `ServerConfig::default()` and `ServerConfig::from_env()`.
  (`update_where_timeout` = separate, typically-SHORTER writer-side WHERE deadline for SPARQL
  UPDATE that bounds writer-queue **head-of-line blocking** from a slow update — `None` =
  use `query_timeout`, `sq-nulp`; `max_query_rows` = coarse memory cap; `max_query_bytes` =
  byte-accounted memory cap that also prices row WIDTH + computed-literal size — `sq-s5is`;
  `max_decompress_ratio` = zip-bomb guard — `sq-ebii`;
  `header_read_timeout` = **slow-loris guard**: max time a connection may take to send its
  complete request-header block, enforced at hyper's HTTP/1 connection layer by
  `sparq_server::serve` — `None` disables; default 15s — `sq-2gqr`.)
  `auth_token` (set: gates the write surface with a Bearer token, constant-time compared;
  `None`: no write auth) and `auth_token_read` (gate reads too) are honoured by the library
  `router` itself — embedders get the gate for free (`sq-zcby`).
- `fn bind_posture(addr: &SocketAddr, allow_remote: bool, auth: AuthPosture) -> BindPosture`
  — the bind gate the binary applies: `Loopback` (proceed), `RemoteAllowed { warning }`
  (proceed + log), or `RemoteRefused { message }` (refuse). `AuthPosture::{None, WriteOnly,
  ReadAndWrite}` (via `AuthPosture::from_config(&config)`) folds the token + read-gate into
  the decision: a non-loopback bind is allowed when `--allow-remote` is set OR the surface is
  fully authenticated (`ReadAndWrite`); a write-token alone (`WriteOnly`) still requires
  `--allow-remote` because reads stay open. This is a *bind-time* posture gate; per-request
  auth is the `auth_token` fields above (enforced by `router`/`harden`-wrapped handlers).
- `fn harden(routes: axum::Router, config: &ServerConfig) -> axum::Router` — wrap any
  router in the production middleware (panic→500, concurrency-limit→429, body-limit→413,
  JSON error bodies, optional trace).
- `async fn serve(listener: tokio::net::TcpListener, app: axum::Router, header_read_timeout:
  Option<Duration>, shutdown: impl Future<Output=()>) -> std::io::Result<()>` — the accept +
  graceful-drain loop the binary runs **instead of `axum::serve`** (`sq-2gqr`). It is a faithful
  port of axum's own loop (per-connection task, watch-channel drain, `with_upgrades()` so the
  `/subscriptions` WebSocket still works) with one addition: it installs a `hyper_util` TokioTimer
  and hyper's HTTP/1 `header_read_timeout`. **Why:** `axum::serve` never installs a timer, so
  hyper's header-read deadline is inert there and a slow-loris client holds a connection (and a
  `concurrency_limit` slot) open indefinitely; this loop closes that hole. Pass `None` to opt back
  out to the unbounded behaviour.
- Re-exports for cache layers/tests: `PinnedGen`, `GLOBAL_POD: &str`
  (`"urn:sparq:pod:global"`), and `sparq_serve::{Epoch, PodEpochs, PodId}`.
- Serializer/negotiation helpers (always compiled, no `server` feature): module
  `sparq_server::negotiate` — `fn negotiate(accept: Option<&str>) -> Format`,
  `fn negotiate_graph(accept: Option<&str>) -> GraphFormat`; module
  `sparq_server::exec` — `fn prepare(&str) -> Result<Prepared, PrepareError>` and
  `fn prepare_with_dataset(&str, &DatasetOverride) -> Result<Prepared, PrepareError>` (applies the
  SPARQL-Protocol `default-graph-uri`/`named-graph-uri` override, sq-z33x),
  `fn apply_update_dataset(&str, &UsingOverride) -> Result<String, UpdateDatasetError>` (the
  update-side `using-*` override), `enum QueryForm { Select, Ask, Construct, Describe }`; module
  `sparq_server::results`.

## Common recipes

**1. Query forms and result negotiation.** Default result media is SPARQL-JSON. Set
`Accept` to choose (q-value aware, defaults to JSON for SELECT/ASK, N-Triples for
CONSTRUCT/DESCRIBE):

| Query form | Accept | Content-Type returned |
| --- | --- | --- |
| SELECT | `application/sparql-results+json` (default) / `+xml` / `text/csv` / `text/tab-separated-values` | matching results media |
| ASK | json (default) / xml | `application/sparql-results+json` / `+xml` |
| CONSTRUCT / DESCRIBE | `application/n-triples` (default) / `text/turtle` / `application/rdf+xml` | matching RDF media; N-Triples, prefix-compacting Turtle, or RDF/XML <!-- [OPUS-4.8] sq-rt6v --> |

```sh
curl -G http://127.0.0.1:3030/sparql -H 'Accept: application/sparql-results+xml' \
     --data-urlencode 'query=SELECT ?s WHERE { ?s ?p ?o }'
# prefix-compacting Turtle:
curl -G http://127.0.0.1:3030/sparql -H 'Accept: text/turtle' \
     --data-urlencode 'query=CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }'
# RDF/XML:
curl -G http://127.0.0.1:3030/sparql -H 'Accept: application/rdf+xml' \
     --data-urlencode 'query=CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }'
```

**2. EXPLAIN a query plan (no execution) or analyze (execute + per-operator trace).**
`text/plain` response. Use `explain` / `explain=plan` (or `Accept: text/x-sparq-explain`)
for the dry run, `explain=analyze` to run + trace (SELECT/ASK only):

```sh
curl -G 'http://127.0.0.1:3030/sparql?explain=true' \
     --data-urlencode 'query=SELECT * WHERE { ?a <http://ex/knows> ?b . ?b <http://ex/age> ?age }'
```

**3. Time-travel reads (opt-in feature).** Build with `--features time-travel`. Every
`/sparql` response carries the `Sparq-Generation` header; capture it and pin a later read
with `?generation=N` (URL param or url-encoded body — body wins). Updates' 204 carries the
generation containing the write:

```sh
cargo run -p sparq-server --features time-travel -- data.ttl --time-travel-generations 32
# read-your-writes: capture the generation an update lands in
G=$(curl -si http://127.0.0.1:3030/sparql -H 'content-type: application/sparql-update' \
      --data 'INSERT DATA { <http://ex/a> <http://ex/p> <http://ex/b> }' \
      | grep -i sparq-generation | tr -d '\r' | awk '{print $2}')
# later, read the store AS OF that generation (old data even after more updates)
curl -G http://127.0.0.1:3030/sparql --data-urlencode 'query=SELECT * WHERE { ?s ?p ?o }' \
     --data-urlencode "generation=$G"
```
Status: aged-out generation → `410 Gone`; never-published / unparsable / pinning an
*update* → `400`.

**4. WebSocket subscriptions (SEPA-style live SELECT).** Connect to
`ws://127.0.0.1:3030/subscriptions`, send `subscribe`, get `subscribed` + an initial
`notification` (sequence 0 = full result as `addedResults`), then added/removed bindings
diffs after each committed update that changes the result:

```text
client:  {"subscribe": {"query": "SELECT ?s WHERE { ?s <http://ex/age> ?o }", "alias": "ages"}}
server:  {"subscribed": {"id": 1, "alias": "ages"}}
server:  {"notification": {"id": 1, "sequence": 0, "addedResults": {…full result…},
                           "removedResults": {"head":{"vars":["s"]},"results":{"bindings":[]}}}}
         …POST /sparql update commits…
server:  {"notification": {"id": 1, "sequence": 1, "addedResults": {…}, "removedResults": {…}}}
client:  {"unsubscribe": {"id": 1}}
server:  {"unsubscribed": {"id": 1}}
```
`addedResults`/`removedResults` are each full SPARQL-JSON results objects. Refusals and
failed re-evaluations come back as `{"error": {"message": …, "id"?: n}}`.

[OPUS-4.8] sq-cxk5: when `--auth-token-read` is set, the upgrade is gated behind the read
token (`401` before upgrade otherwise). A non-browser client sends `Authorization: Bearer
<TOKEN>` on the handshake; a **browser** (which cannot set WS handshake headers) passes it as
a subprotocol: `new WebSocket("ws://host/subscriptions", ["bearer." + token])`. The server
takes the substring after the `bearer.` prefix as the token, validates it (constant-time),
and echoes the subprotocol back per RFC 6455. With no read token configured, the upgrade is
open (back-compatible).

**4b. SSE subscriptions (`text/event-stream`).** [OPUS-4.8] sq-bxog: the SAME subscription
engine over Server-Sent Events, for clients that prefer a plain HTTP GET stream to a
WebSocket. `GET /subscriptions/sse?query=<SELECT>[&alias=<x>]` opens one subscription per
stream (the query is in the query string — SSE is one-way, so there is no `subscribe`/
`unsubscribe` frame; close the stream to unsubscribe). The events carry the SAME JSON as
the WS path — only the framing differs (`event:` / `data:` / `id:` lines, blank-line
terminated, `: ping` keep-alive comments hold idle connections open). The SSE `id:` mirrors
the per-subscription `sequence`.

```sh
curl -N 'http://127.0.0.1:3030/subscriptions/sse?query=SELECT%20?s%20WHERE%20{%20?s%20%3Chttp://ex/age%3E%20?o%20}&alias=ages'
```
```text
event: subscribed
data: {"subscribed":{"id":1,"alias":"ages"}}

event: notification
id: 0
data: {"notification":{"id":1,"sequence":0,"alias":"ages","addedResults":{…full result…},"removedResults":{…empty…}}}

  …POST /sparql update commits…
event: notification
id: 1
data: {"notification":{"id":1,"sequence":1,"alias":"ages","addedResults":{…},"removedResults":{…}}}
```
[OPUS-4.8] sq-cxk5: like the WS path, when `--auth-token-read` is set this GET is gated
behind the read token via the `Authorization: Bearer <TOKEN>` header (it is a plain GET, so
the header is the only auth channel — no WS subprotocol) — `401` before the stream opens
otherwise. A registration refusal (missing/non-SELECT/malformed `query` → `400`; capacity/budget →
`503`) is returned as a normal `{"error":"…"}` JSON HTTP response BEFORE the stream opens —
SSE cannot set a status once the stream is flowing. A later re-evaluation failure ends the
stream with a final `event: error` frame. Both transports share one registry + change
source, so the per-conn / server-wide subscription caps and the `sparq_active_subscriptions`
gauge count SSE streams and WS subscriptions together.

**5. Graph Store read + write + operational endpoints.**

```sh
# READ (GET/HEAD): serialises the addressed graph in the Accept-negotiated RDF syntax
# (default N-Triples; also text/turtle = prefix-compacting Turtle, application/rdf+xml = RDF/XML)
curl http://127.0.0.1:3030/sparql/graph?default                 # GSP indirect (default graph)
curl 'http://127.0.0.1:3030/sparql/graph?graph=http://ex/g'     # GSP indirect (named graph)
curl http://127.0.0.1:3030/graphs/whatever                      # GSP direct (request URI is the graph IRI)
curl -H 'Accept: application/rdf+xml' http://127.0.0.1:3030/sparql/graph?default   # RDF/XML read [OPUS-4.8] sq-rt6v

# WRITE (sq-gxsj): body is RDF, format by Content-Type
#   (turtle | n-triples | n-quads | trig | application/rdf+xml [OPUS-4.8] sq-rt6v)
# PUT = REPLACE graph contents (201 if created, 204 if replaced):
curl -X PUT 'http://127.0.0.1:3030/sparql/graph?graph=http://ex/g' \
     -H 'content-type: text/turtle' --data '<http://ex/s> <http://ex/p> <http://ex/o> .'
# POST = MERGE (additive); selector-less POST to /sparql/graph creates a fresh server-named graph:
curl -X POST 'http://127.0.0.1:3030/sparql/graph?graph=http://ex/g' \
     -H 'content-type: application/n-triples' --data '<http://ex/s2> <http://ex/p> <http://ex/o2> .'
# DELETE = DROP the graph (204; 404 if a named graph is absent; ?default → CLEAR DEFAULT, always 204):
curl -X DELETE 'http://127.0.0.1:3030/sparql/graph?graph=http://ex/g'

curl http://127.0.0.1:3030/health                               # -> "ok"
curl http://127.0.0.1:3030/metrics                              # Prometheus text exposition

# Admin WAL compaction / vacuum for ERASURE-COMPLETENESS (sq-x32t). POST-only; gated by the
# WRITE token. Physically purges data removed by DELETE / DROP (incl. orphaned literal VALUES) from the
# on-disk store so a logical erasure is followed by real erasure. 200 ok; 409 if in-memory
# (no --persist, nothing to purge); 503 on a transient durable-write error (retryable):
curl -X POST -H 'Authorization: Bearer <TOKEN>' http://127.0.0.1:3030/admin/compact
```

`/metrics` is hand-rolled Prometheus text exposition (no metrics dependency); the
middleware wraps the whole hardening stack, so shed (`429`), body-limit (`413`) and panic
(`500`) responses are counted with the status the client saw:

| Metric | Type | What |
| --- | --- | --- |
| `sparq_http_requests_total{endpoint,status}` | counter | requests by endpoint + response status |
| `sparq_query_duration_seconds` | histogram | wall time of `/sparql` (query + update); buckets 1 ms … 10 s |
| `sparq_active_subscriptions` | gauge | active WebSocket subscriptions (scrape time) |
| `sparq_graph_triples` | gauge | triples in the published graph (scrape time) |
| `sparq_updates_total` | counter | successfully applied SPARQL updates |

**`POST /admin/compact` — WAL compaction / vacuum (erasure-completeness, `sq-x32t`).** A logical
`DELETE` / `DROP GRAPH` retracts data from the live view but leaves the superseded bytes in earlier
`--persist` WAL segments (and the dictionary) until a compaction folds the live state into a fresh
base. This admin op does that on demand: it **physically rewrites** the on-disk store to only the
current live triples, with a **re-interned (purged) dictionary**, then **atomically swaps** the
directory (rollback-safe two-rename + WAL truncate; an interrupted swap is healed on the next
open). So a deleted triple's value — including an orphaned **literal value** (e.g. personal data) —
is gone from disk, not just hidden. It runs on the single writer thread strictly **between
batches** (no race with a concurrent write), preserves the live triple set **exactly** (no
generation published; reads keep flowing throughout). **POST-only**, gated by the **write** token
(`--auth-token`), like an UPDATE. Responses: `200` ok; `409` if the server is in-memory (no
`--persist` — there is no on-disk history to purge, so a no-op success would mislead); `503` on a
transient durable-write error (retryable, writer stays alive). **Offline equivalent:** stop the
server and run `sparq-cli compact <persist-dir>` (see the `cli` skill). **Honest scope:** this
scrubs the engine's own on-disk segments + dictionary; it cannot reach bytes already copied off-box
(filesystem snapshots, COW history, external backups) — those are the operator's responsibility
(see `compliance/privacy/retention-erasure-runbook.md` §7a/§7b).

GSP **writes** translate into a server-minted SPARQL Update (`DROP`/`CLEAR` + `INSERT
DATA`) and submit through the SAME sequenced group-commit writer the
`application/sparql-update` operation uses — so they share its atomicity, snapshot
consistency, blocking-on-commit semantics, the `Sparq-Generation` header (time-travel
feature), AND its **auth gate** (a GSP write is as powerful as an UPDATE, so `PUT`/`POST`/
`DELETE` are gated by `--auth-token` exactly like an UPDATE; `GET`/`HEAD` are reads). A
malformed body → `400`; an unsupported body Content-Type → `415`.

**5b. Container (ghcr.io).** Published on every release tag (`ghcr.io/jeswr/sparq-server`,
distroless). The image sets `SPARQ_ALLOW_REMOTE=1` so the `0.0.0.0` bind it needs (loopback
is unreachable through Docker's port map) boots out of the box — running the container is the
operator's explicit choice to publish a surface. **This default has NO auth.** Because every
`SPARQ_*` var is read from the environment, secure it with `-e` (no flag wiring):

```sh
docker run --rm -p 3030:3030 ghcr.io/jeswr/sparq-server                       # empty graph, no auth
docker run --rm -p 3030:3030 -v "$PWD/data:/data:ro" \
  ghcr.io/jeswr/sparq-server --format turtle /data/dataset.ttl                # serve a dataset
docker run --rm -p 3030:3030 \
  -e SPARQ_AUTH_TOKEN="$TOK" -e SPARQ_AUTH_TOKEN_READ=1 \
  ghcr.io/jeswr/sparq-server                                                  # fully Bearer-gated
```

Deliver the token over TLS (terminate at a proxy). See `crates/sparq-server/README.md` →
"Running the container image".

**5c. Federation discovery — VoID + Service Description (OPT-IN, `sq-d3d8`).** A server can
advertise itself as a discoverable federation node by serving two read-only RDF descriptors:

- `GET /.well-known/void` — a [W3C VoID](https://www.w3.org/TR/void/) dataset description
  (`void:triples` / `void:entities` / `void:classes` / `void:properties` + one
  `void:classPartition` per class and `void:propertyPartition` per predicate), **plus the
  characteristic-set source statistics** (sq-mr32, federation A3/Z2): sparq already mines
  per-entity-type predicate co-occurrence + multiplicity (Neumann & Moerkotte characteristic
  sets), so the served VoID also emits them under a documented sparq extension vocab
  `scs:` (`<http://sparq.dev/ns/cs#>`) — `scs:characteristicSet` linking the dataset to one
  `scs:CharacteristicSet` node per retained set (`scs:subjects` = `count(C)`, plus one
  per-predicate `scs:predicateStat` reusing `void:property`/`void:triples` and adding
  `scs:avgMultiplicity`), with the EXACT distinct-set count on `scs:distinctCharacteristicSets`.
  The CS stats are a strict superset of the standard VoID (a VoID-only client ignores the
  `scs:` triples; a CostFed/Odyssey-class source-selector gets sharp star/multi-join
  cardinalities). Generated by `sparq-introspect`'s `Introspection::to_void_with_cs`
  (`Introspection::to_void` remains the CS-free base).
- `GET /sparql` **with no `query` parameter** — a [SPARQL 1.1 Service
  Description](https://www.w3.org/TR/sparql11-service-description/) generated from the server's
  **actual** capabilities (sq-qfcb), never a hard-coded fiction:
    - `sd:Service` + `sd:endpoint` (the request `Host`'s `/sparql`);
    - `sd:supportedLanguage` — `SPARQL11Query` always; `SPARQL11Update` **only when an anonymous
      client can run one** (it is suppressed when a `--auth-token` write gate is configured,
      because then an unauthenticated SD reader cannot use Update);
    - `sd:resultFormat` — the four SPARQL-results serialisations (JSON/XML/CSV/TSV) plus the RDF
      graph serialisations the CONSTRUCT/DESCRIBE/GSP-read path emits (Turtle/N-Triples/RDF-XML),
      and `sd:inputFormat` — the RDF serialisations the GSP write path parses (Turtle/N-Triples/
      RDF-XML). These mirror exactly what `crate::negotiate` produces/accepts; an integration test
      drives a real SELECT per advertised result format and asserts the returned `Content-Type`
      matches, so the advertisement cannot over-promise;
    - `sd:feature sd:BasicFederatedQuery` — **only** when the `service` cargo feature is compiled
      in (the `SERVICE` clause is then evaluable); omitted otherwise;
    - `sd:extensionFunction` — one per function the engine has **actually registered**: the
      `geof:` GeoSPARQL set with the `geo` feature (read back through
      `FunctionRegistry::iris()`, so it can never drift from what runs), none without it;
    - the default dataset — its default graph linked to the VoID document via `dcterms:source`,
      **plus** an `sd:namedGraph` enumeration of every IRI-named graph in the served dataset
      (sq-optl): each is an `sd:NamedGraph` with its `sd:name` (the `FROM NAMED`-referenceable IRI)
      and an `sd:graph` `sd:Graph` carrying that graph's `void:triples` count. The names come off
      the same pinned snapshot the VoID descriptor reads, sorted for determinism, and **IRI-only**
      — a blank-node-named graph is skipped because it is not `FROM NAMED`-referenceable, so
      advertising it would be a fiction. A default-only dataset emits no `sd:namedGraph`.

  Note `sd:UnionDefaultGraph` is deliberately **not** advertised — the engine's default graph
  holds only default-graph triples (named graphs are not folded in), so claiming it would be
  dishonest.

Both are **double opt-in** and OFF by default: compiled only with the
`federation-descriptors` cargo feature **and** served only when
`--federation-descriptors` / `SPARQ_FEDERATION_DESCRIPTORS=1` is also set. Without the
feature there is zero cost (no `sparq-introspect` dependency, no routes); with the feature
but not the flag, `/.well-known/void` is `404` and a no-query `GET /sparql` is the historical
`400 missing 'query'`. Both content-negotiate the RDF syntax (`Accept: text/turtle` default,
`application/n-triples`, `application/rdf+xml`); reads are gated by `--auth-token-read` like
any GET. The dataset/endpoint IRIs self-describe the `Host` the client used.

```sh
cargo run -p sparq-server --features federation-descriptors -- data.ttl --federation-descriptors
curl http://127.0.0.1:3030/.well-known/void                         # VoID + scs: char-set stats (Turtle by default)
curl -H 'Accept: application/n-triples' http://127.0.0.1:3030/sparql # Service Description (no query)
```

**5d. Triple Pattern Fragments / LDF source endpoint (OPT-IN, `sq-bzh1`).** A server can expose
itself as a low-cost [Linked Data Fragments](http://linkeddatafragments.org/) /
[Triple Pattern Fragments](https://www.hydra-cg.com/spec/latest/triple-pattern-fragments/)
**source** that a TPF client (Comunica / the LDF client) drives a join against — far cheaper per
request than a full SPARQL endpoint:

- `GET /tpf?subject=&predicate=&object=` — a **paged** RDF fragment of the triples matching one
  triple pattern. Each of `subject` / `predicate` / `object` is an **N-Triples term**
  (`<iri>`, `"lit"`, `"lit"@en`, `"lit"^^<dt>`); an absent / empty parameter is a variable
  (unbound). `page=N` (0-based) selects the page; the page size is bounded (default 100).
- The fragment carries **Hydra controls**: `hydra:totalItems` / `void:triples` (the matched-triple
  count, reusing the engine's cheap cardinality **estimate** — NOT a full scan),
  `hydra:itemsPerPage`, the full `PartialCollectionView` paging vocabulary —
  `hydra:next` / `hydra:previous` (present only when a next / previous page exists) plus
  `hydra:first` / `hydra:last` (emitted on EVERY page so a client can jump to either end of the
  view from anywhere; `first` is always page 0, `last` is the page holding the final match —
  derived from the same estimate as `totalItems`/`next`) — and a `hydra:search` / `hydra:template`
  / `hydra:mapping` control describing the `{subject,predicate,object}` URI template so a generic
  client can request any other pattern.

**5d-bis. brTPF — bind-restricted Triple Pattern Fragments (OPT-IN, `sq-dxhb`; feature `brtpf`,
implies `tpf`).** brTPF ([Hartig & Buil-Aranda, ODBASE 2016](http://olafhartig.de/files/HartigBuilAranda_ODBASE2016_Preprint.pdf))
extends the SAME `/tpf` endpoint so a client can attach a **set of solution mappings** and the
server returns only the page of pattern matches COMPATIBLE WITH AT LEAST ONE supplied binding —
pushing a bind-join's semi-join down to the source, so far less data crosses the wire than
re-fetching the whole pattern once per binding.

- The binding set rides the `values` query parameter (`GET`) or — preferred for a large set —
  the request **body** of a `POST /tpf`. The wire format is one mapping per line; within a line,
  whitespace-separated `position=term` pairs where `position` is `subject`/`predicate`/`object`
  (or short `s`/`p`/`o`) and `term` is the SAME N-Triples-term grammar as the pattern parameters
  (e.g. `s=<http://ex/alice>`). A blank line / empty payload is the no-restriction (plain-TPF)
  case; a malformed payload is a sanitized `400` (the offending input is NOT echoed).
- The fragment is the **deduplicated union** of each mapping's specialised-pattern matches.
  `hydra:totalItems` and the paging window reflect the bindings-RESTRICTED result, not the
  unrestricted pattern, and the `hydra:search` control advertises an extra `hydra:mapping` for the
  `values` variable so a client discovers the dataset accepts a restriction.
- A `tpf`-only build is **byte-identical** to before: the `values` parsing + the `POST` route are
  `#[cfg]`-stripped, so a stray `values` parameter is just an ignored unknown parameter (plain
  TPF). Still governed by the same `--tpf` runtime flag and read-auth (`POST /tpf` is a READ — it
  returns a fragment, it never writes).
- **DoS caps on the binding set (`sq-r74h`).** The brTPF fragment runs ONE index scan per
  attached mapping (`tpf::evaluate_brtpf`), so the per-request cost is super-linear in the mapping
  **count**, not the payload **bytes** — and `--max-body-bytes` bounds the count only transitively
  (a 1 MiB body of `s=<a>`-sized mappings is ~150k scans) and does NOT cover the `values`
  **query-string** carrier of a `GET /tpf` at all (it is a body limit). Two dedicated, ON-by-default
  caps close that: `--brtpf-max-bindings` (default `1024`) bounds the mapping count, and
  `--brtpf-max-values-bytes` (default `1 MiB`) bounds the raw `values` payload bytes — enforced
  BEFORE any parse/index work. A breach is a `413` (the same refusal class as `--max-body-bytes`,
  distinct from the malformed-payload `400`); the message names the cap, never the caller's input
  (no echo). `0` disables either cap. The pure parser is `tpf::parse_bindings_capped(payload,
  tpf::BindingLimits { max_mappings, max_payload_bytes })`, returning `tpf::BindingError::{Malformed
  → 400, TooLarge → 413}`.

```sh
cargo run -p sparq-server --features brtpf -- data.ttl --tpf
# restrict `?s ex:knows ?o` to a single subject via the `values` parameter
curl 'http://127.0.0.1:3030/tpf?predicate=%3Chttp%3A%2F%2Fex%2Fknows%3E&values=s%3D%3Chttp%3A%2F%2Fex%2Fcarol%3E'
# a larger binding set in a POST body (one `position=term` mapping per line)
curl -X POST -H 'Accept: application/n-triples' \
  --data $'s=<http://ex/alice>\ns=<http://ex/carol>' \
  'http://127.0.0.1:3030/tpf?predicate=%3Chttp%3A%2F%2Fex%2Fknows%3E'
```

**Double opt-in**, OFF by default and **READ-only** (no write path): compiled only with the `tpf`
cargo feature **and** served only when `--tpf` / `SPARQ_TPF=1` is also set (mirrors
`federation-descriptors`). Without the feature, zero cost (no route); with the feature but not
the flag, `/tpf` is `404`. Content-negotiates `Accept: text/turtle` (default) /
`application/n-triples` / `application/rdf+xml`; reads are gated by `--auth-token-read` like any
GET. The fragment/dataset/template IRIs self-describe the `Host` the client used.

```sh
cargo run -p sparq-server --features tpf -- data.ttl --tpf
# all triples with predicate ex:knows, page 0 (Turtle by default)
curl 'http://127.0.0.1:3030/tpf?predicate=%3Chttp%3A%2F%2Fex%2Fknows%3E'
# a fully-bound pattern, as N-Triples
curl -H 'Accept: application/n-triples' \
  'http://127.0.0.1:3030/tpf?subject=%3Chttp%3A%2F%2Fex%2Falice%3E&predicate=%3Chttp%3A%2F%2Fex%2Fknows%3E'
```

**5e. SHACL validation endpoint (OPT-IN, `sq-r868`, from-pss gh-162 follow-up; feature
`shacl`).** Validate the server's **currently-loaded data graph** against a SHACL **shapes**
graph the client POSTs — the server-side / large-graph path from gh-162 (the store is already in
memory, so there is no per-request data parse, and the 100k-node case where the JS
`rdf-validate-shacl` OOMs is handled natively by `sparq-shacl`).

- `POST /shacl/validate` — the request **body** is the SHACL shapes graph (RDF: `text/turtle` /
  `application/n-triples` / `application/n-quads` / `application/trig` / `application/rdf+xml`,
  classified by `Content-Type` like a GSP write body, and gzip-decoded under the same zip-bomb
  cap). The **data** graph is the server's pinned store snapshot.
- **Response, content-negotiated from `Accept`:** the default is the JSON projection PSS / the
  wasm `shacl` binding consume — `{ "conforms": bool, "results": [{ "focusNode", "path", "value",
  "sourceShape", "sourceConstraintComponent", "severity", "message" }] }`; `Accept: text/turtle`
  yields the W3C SHACL report-vocabulary graph (`sparq_shacl::ValidationReport::to_turtle`).
- Always `200` regardless of conformance — the verdict is in the body (`conforms`), not the HTTP
  status. A malformed shapes body is a `400`, an unsupported `Content-Type` a `415`, a non-`POST`
  method a `405`.
- Covers SHACL Core + SHACL-SPARQL (`sh:sparql`, §5.2) + custom SPARQL constraint components (§6)
  — whatever `sparq-shacl::validate` supports (it is the SAME engine the wasm binding and the
  `sparq-shacl` crate expose; see the `shacl-validation` skill). SHACL-AF `sh:rule` inference is
  not part of validation and is not run by this endpoint.

```sh
cargo run -p sparq-server --features shacl -- data.ttl --shacl
# validate the loaded store against POSTed shapes → JSON report (default)
curl -X POST -H 'Content-Type: text/turtle' --data-binary @shapes.ttl \
  http://127.0.0.1:3030/shacl/validate
# the W3C report vocabulary as Turtle
curl -X POST -H 'Content-Type: text/turtle' -H 'Accept: text/turtle' --data-binary @shapes.ttl \
  http://127.0.0.1:3030/shacl/validate
```

**Double opt-in**, OFF by default and **READ-only** (validation never mutates the store):
compiled only with the `shacl` cargo feature **and** served only when `--shacl` / `SPARQ_SHACL=1`
is also set (mirrors `tpf` / `federation-descriptors`). Without the feature, zero cost (no route,
no SHACL code — `sparq-core`/the wasm bundle are untouched); with the feature but not the flag,
`/shacl/validate` is `404`. Reads are gated by `--auth-token-read` like any GET.

**6. Hardening — flags / env / library.** Each flag overrides its `SPARQ_*` env var; the
env overrides the default.

| Flag | Env | Default | Effect |
| --- | --- | --- | --- |
| `--query-timeout SECS` | `SPARQ_QUERY_TIMEOUT` | `30` (`0`=off) | per-request timeout → `503` |
| `--update-where-timeout SECS` | `SPARQ_UPDATE_WHERE_TIMEOUT` | unset (`0`/unset = use `--query-timeout`) | **separate, typically-SHORTER writer-side WHERE deadline for SPARQL UPDATE** — bounds writer-queue **head-of-line blocking** from a slow update (the single sequenced writer is released within this window instead of holding it for the full read timeout); the update WHERE budget is `min(query_timeout, update_where_timeout)` → slow update `503` (`sq-nulp`) |
| `--max-body-bytes N` | `SPARQ_MAX_BODY_BYTES` | `1048576` | body cap → `413` |
| `--max-concurrent N` | `SPARQ_MAX_CONCURRENT` | `32` | in-flight cap, load-shed → `429` |
| `--header-read-timeout SECS` | `SPARQ_HEADER_READ_TIMEOUT` | `15` (`0`=off) | **slow-loris guard**: max time a connection may take to send its complete request-header block — enforced at hyper's HTTP/1 connection layer by `sparq_server::serve` (NOT `axum::serve`, which never installs a timer so its header deadline is inert), so it fires BEFORE a handler and frees the concurrency slot a dribbling client would otherwise hold forever; connection closed when exceeded (`sq-2gqr`) |
| `--max-results N` | `SPARQ_MAX_RESULTS` | unlimited (`0`=off) | result/solution cap (SELECT + CONSTRUCT/DESCRIBE WHERE-solutions + EXPLAIN ANALYZE; not ASK/GSP-read/UPDATE) → honest `413` (not truncation) |
| `--max-query-rows N` | `SPARQ_MAX_QUERY_ROWS` | unlimited (`0`=off) | **memory cap** (coarse): working-set ROW ceiling on **every** form → honest `413` (`sq-ebii`) |
| `--max-query-bytes N` | `SPARQ_MAX_QUERY_BYTES` | unlimited (`0`=off) | **byte-accounted memory cap**: prices working-set row WIDTH (`rows × vars × id-size`) + computed-literal bytes on **every** form → honest `413` (`sq-s5is`) |
| `--max-decompress-ratio N` | `SPARQ_MAX_DECOMPRESS_RATIO` | `20` (`0`=refuse gzip) | **zip-bomb guard**: cap on decompressed:compressed for a `Content-Encoding: gzip` body → `413` (`sq-ebii`) |
| `--max-subscriptions N` | `SPARQ_MAX_SUBSCRIPTIONS` | `256` | server-wide subs |
| `--max-subscriptions-per-conn N` | `SPARQ_MAX_SUBSCRIPTIONS_PER_CONN` | `16` | per-socket subs |
| `--verbose` | — | off | TraceLayer request logging (respects `RUST_LOG`); request URIs **redacted by default** — see "Request-log redaction" |
| `--log-full-requests` | `SPARQ_LOG_FULL_REQUESTS` | off (redaction ON) | (`sq-toze.34`) OPT OUT of request-log redaction: log the raw request URI (incl. the full `?query=` SPARQL text) verbatim. Inert without `--verbose` — see "Request-log redaction" |
| `--auth-token TOKEN` | `SPARQ_AUTH_TOKEN` | off (no auth) | require `Authorization: Bearer TOKEN` on every WRITE (SPARQL Update + GSP PUT/POST/DELETE) → `401` + `WWW-Authenticate: Bearer` otherwise; constant-time compared (QLever's `-a`) |
| `--auth-token-read` | `SPARQ_AUTH_TOKEN_READ` | off | ALSO gate reads with the same token (only meaningful with a token set) |
| `--allow-remote` | `SPARQ_ALLOW_REMOTE` | off | opt in to a non-loopback bind; without it a non-loopback `--addr` is **refused** unless the surface is fully authenticated (`--auth-token` AND `--auth-token-read`), with it it warns and proceeds |
| `--service-allow HOST\|*.SUFFIX` (repeatable) | `SPARQ_SERVICE_ALLOW` (comma/ws-sep) | empty = **deny ALL SERVICE** | (feature `service`) allowlist a SERVICE egress host (exact or `*.suffix` wildcard); CLI + file + env are all merged (combined additively) |
| `--service-allow-file PATH` | — | — | (feature `service`) load allowlist entries, one per line (`#` comments + blanks ignored) |
| `--time-travel-generations N` | `SPARQ_TIME_TRAVEL_GENERATIONS` | `16` | (feature) retained generations |
| `--time-travel-max-age SECS` | `SPARQ_TIME_TRAVEL_MAX_AGE` | off | (feature) age-out window |
| `--federation-descriptors` | `SPARQ_FEDERATION_DESCRIPTORS` | off | (feature `federation-descriptors`) serve a VoID at `/.well-known/void` + a SPARQL Service Description on `GET /sparql` with no query — see "Federation discovery" |
| `--tpf` | `SPARQ_TPF` | off | (feature `tpf`) serve a Triple Pattern Fragments / LDF source endpoint at `GET /tpf?subject=&predicate=&object=` (paged, full Hydra paging incl. `first`/`last`, read-only); same flag also serves brTPF bind-restricted fragments (`values` param / `POST` body) when built with the `brtpf` feature — see "Triple Pattern Fragments" |
| `--shacl` | `SPARQ_SHACL` | off | (feature `shacl`) serve the SHACL validate endpoint `POST /shacl/validate` — POST a shapes graph, the server validates its loaded data graph against it; JSON report (default) or W3C report Turtle (`Accept: text/turtle`); read-only — see "SHACL validation endpoint" |
| `--brtpf-max-bindings N` | `SPARQ_BRTPF_MAX_BINDINGS` | `1024` (`0`=off) | (feature `brtpf`) **DoS cap on the brTPF binding-set mapping COUNT** — one index scan per mapping, so cost is super-linear in the count, not the bytes → `413` (`sq-r74h`) |
| `--brtpf-max-values-bytes N` | `SPARQ_BRTPF_MAX_VALUES_BYTES` | `1048576` (`0`=off) | (feature `brtpf`) **DoS cap on the raw brTPF `values` payload BYTES** — bounds the GET query-string carrier that `--max-body-bytes` never sees → `413` (`sq-r74h`) |
| `--audit-log` | `SPARQ_AUDIT_LOG` | off | (feature `audit-log`) per-query **access audit log** — see "Access audit log" |
| `--access-audit <file\|stderr>` | `SPARQ_ACCESS_AUDIT` | off | (feature `access-audit`) richer **structured access-audit sink** (typed JSON-Lines: actor / action / resource / decision+basis / ts / fingerprint) — see "Structured access-audit sink" |

In a library: `AppState::with_config(graph, ServerConfig { max_concurrent: 64, ..Default::default() })`
then `router(state)`, or `harden(my_router, &config)`.

### Server hardening — the DoS/SSRF limits (`sq-ebii` + `sq-4w18` + `sq-s5is`)

The threat model (a public, unauthenticated endpoint behind a gateway) calls for these
distinct limits. **Be precise about what each bounds** — only the body-size and ratio caps
are byte-hard; the timeout and both memory caps are *cooperative* (approximate in time), and
the memory caps are coarse working-set ceilings (row count / estimated bytes), not an RSS
quota:

1. **Query timeout** (`--query-timeout`, `SPARQ_QUERY_TIMEOUT`, default `30s`, `0`=off).
   The engine's cooperative `QueryBudget.deadline` stops the worker at its next coarse check
   (operator entry / per outer loop iteration); a wall-clock hard cap of `timeout + 2s` grace
   guarantees the HTTP `503` even if the engine is mid-stretch. Applies to **all** forms now
   — SELECT / ASK / CONSTRUCT / DESCRIBE / GSP-read **and** SPARQL Update
   (`application/sparql-update` + GSP `PUT`/`POST`/`DELETE`): the update path runs under the
   same cooperative budget on the writer thread *and* the same wall-clock await cap on the
   HTTP side. *Bounds:* wall-clock per request, approximately (next-check granularity).
   *Caveat:* updates are sequenced on a single writer, so a long update blocks the queue
   behind it until it finishes — this cap bounds the **client's own wait**, while the writer
   runs the WHERE to its cooperative stop. To bound that **head-of-line blocking** of the
   queue, set a separate, shorter WHERE deadline — see (1b).
   - **1b. UPDATE writer-side WHERE deadline / head-of-line bound**
     (`--update-where-timeout`, `SPARQ_UPDATE_WHERE_TIMEOUT`, default **unset** = use
     `--query-timeout`; `sq-nulp`). A SEPARATE, typically-shorter cooperative deadline applied
     ONLY to the WHERE phase of a SPARQL UPDATE on the writer thread: the update's budget
     deadline becomes `min(query_timeout, update_where_timeout)`, so a slow update releases the
     single sequenced writer within this window instead of holding it for the full (usually
     longer) read timeout — bounding how long one slow update can head-of-line block every
     queued update behind it. Cooperative, like the query timeout (next-check granularity); a
     tunable backstop, not hard preemption. Unset ⇒ the update WHERE budget is exactly
     `--query-timeout` (unchanged). The offending update gets a `503`.
2. **Memory cap** (`--max-query-rows`, `SPARQ_MAX_QUERY_ROWS`, default **off**, `0`=off).
   A coarse OOM circuit-breaker: an upper bound on the **row count** of any *materialised
   intermediate or final* result the engine builds, on **every** form (including
   CONSTRUCT/DESCRIBE and an UPDATE's `DELETE/INSERT … WHERE`), via the engine's
   `QueryBudget.max_rows` working-set bound. A join blow-up aborts with an honest `413`
   instead of OOMing; the speculative cross-product allocation is also capped up-front.
   *Bounds:* **cardinality (rows), not bytes.* Peak heap ≈ `rows × per-row term cost`, so a
   query with few but very wide rows (many vars / huge literals) can still exceed the implied
   memory; dictionary growth, sort/group scratch and a CONSTRUCT template are outside it. It
   is also approximate in time (coarse checks). Treat it as a blunt anti-OOM breaker, **not**
   an RSS quota. Distinct from `--max-results` (the result/solution cap, folded into the
   budget on SELECT / CONSTRUCT/DESCRIBE / EXPLAIN ANALYZE — but not ASK / GSP-read / UPDATE);
   on a path where both apply, the effective cap is the tighter of the two. For the row cap's
   width/literal blind spots, see the byte-accounted companion (2b); writer-queue
   head-of-line blocking from a slow UPDATE is deferred (`sq-nulp`).
2b. **Byte-accounted memory cap** (`--max-query-bytes`, `SPARQ_MAX_QUERY_BYTES`, default
   **off**, `0`=off; `sq-s5is`). The byte-accounted twin of (2): instead of counting ROWS it
   costs the estimated working-set BYTES — `rows × width × size_of::<Id>()` for each
   materialised intermediate (so it prices the WIDTH the row cap is blind to) PLUS the bytes
   of query-COMPUTED terms (BIND / aggregate / CONSTRUCT scratch interned into the per-query
   local vocab — the non-row allocations the row cap misses). Enforced via
   `QueryBudget.max_bytes` on **every** form (incl. an UPDATE's WHERE), at the same coarse
   cooperative sites, honest `413` on overflow. *Bounds:* the QUERY working set, estimated as
   a portable **lower** bound on real heap (ignores allocator overhead, `SmallVec`
   inline-vs-spill, and the pre-existing dictionary/index memory) — so it is strictly tighter
   and more width/literal-aware than the row cap, but still a coarse circuit-breaker, **not**
   an exact RSS quota. Composes with (2) and `--max-results`: whichever ceiling trips first
   aborts.
3. **Decompression-ratio cap** (`--max-decompress-ratio`, `SPARQ_MAX_DECOMPRESS_RATIO`,
   default `20`×, `0`=refuse gzip). When a GSP write body arrives `Content-Encoding: gzip`
   the server inflates it with a hard ceiling of
   `min(ratio × compressed_len, max_body_bytes)`, checked **during** inflate (bounded
   `Read::take`), and refuses with `413` the moment the decompressed output would cross it —
   so a tiny but pathologically compressible body cannot inflate into an OOM. `0` refuses
   gzip bodies outright (fail-closed). *Bounds:* the bodies **the server itself** inflates
   (GSP `PUT`/`POST`). An unknown `Content-Encoding` is a `415`. *Caveat:* it does **not**
   cover a compressed payload the *engine* fetches behind a SPARQL `LOAD <url>` / `SERVICE`
   — those use their own ingest; `SERVICE` egress is bounded separately by limit 4. The
   compressed bytes pass the `--max-body-bytes` gate first, and the decompressed ceiling is
   `min(ratio × compressed_len, max_body_bytes)` — so the decompressed output is itself capped
   at `max_body_bytes`, never `max_body_bytes × ratio`.
4. **SERVICE-SSRF egress allowlist** (`--service-allow` / `--service-allow-file` /
   `SPARQ_SERVICE_ALLOW`, default **deny ALL**, feature `service`) — shipped in `sq-4w18`,
   see "SERVICE federation (egress allowlist)" below. A `SERVICE <iri>` turns
   attacker-controlled query text into an outbound request from the server host (textbook
   SSRF; worst case the `169.254.169.254` cloud-metadata IP), so it reaches **nothing**
   unless its host is allowlisted, enforced on the *resolved* IP before any socket opens
   (DNS-rebinding-safe), uniformly across queries / ASK / CONSTRUCT/DESCRIBE / subscriptions
   / federated `INSERT … WHERE`.

Library callers set these on `ServerConfig` (`query_timeout`, `max_query_rows`,
`max_query_bytes`, `max_decompress_ratio`, `service_allow`). Embedders driving the engine
directly thread a `sparq_engine::QueryBudget { deadline, max_rows, max_bytes }` into
`*_with_budget` query entry points and `update_in_place_with_budget`, and wrap calls in
`sparq_engine::with_service_egress_policy(strict, [host], || …)`.

### Access audit log — opt-in per-query audit trail (`sq-0bxp`, CDMC CD-2)

A **per-query access audit log** for compliance regimes that need a per-subject / per-query
trail (CDMC CD-2, ISO 27001 A.8.15, EU CRA logging) — distinct from the aggregate-only
`/metrics`. **Doubly opt-in:** compile with the `audit-log` cargo feature, then turn it on at
runtime with `--audit-log` (env `SPARQ_AUDIT_LOG=1`). Off (either), the module and every call
site are `#[cfg]`-stripped or short-circuited before any record is built — a request pays
essentially zero.

For each query / update / Graph-Store request the server emits **one** structured `tracing`
event under the dedicated `target: "sparq_server::audit"` (`tracing::info!`). Route it to your
sink with the standard `RUST_LOG` machinery, independently of the `--verbose` request log:
`RUST_LOG=sparq_server::audit=info`. (`--audit-log` installs a subscriber on its own if
`--verbose` did not.) Fields:

| field | meaning |
| --- | --- |
| `requester` | `anonymous`, or `token:<fnv1a-hash>` of the presented Bearer token — **never the raw token** |
| `op` | `query` / `update` / `graph_read` / `graph_write` (operation class, keyed on whether it mutates) |
| `fingerprint` | FNV-1a hash (hex) of the trimmed query/update text — **not the query text** |
| `decision` | `allowed` or `denied` |
| `reason` | denial reason (`auth: missing or invalid Bearer token`); empty when allowed |
| `status` | the HTTP status the client saw (`200` / `401` / `413` / …) |
| `rows` | result-row count when known (else absent — `status` is the authoritative outcome) |
| `duration_us` | handler wall-clock in microseconds |

**No-PII / no-info-leak posture (reuses the #241 lesson):** this is a **server-side** log under
the operator's control — it is NEVER written to the HTTP response. It deliberately does **not**
log the full query text (raw SPARQL can disclose loaded-data fragments or caller PII — the #241
contract) nor the Bearer secret; only a stable, non-reversible fingerprint of each, enough to
correlate repeated identical queries / a recurring caller across requests and restarts. Library
callers set `ServerConfig { audit_log: true, .. }` (field present only with the feature). See
`crates/sparq-server/src/audit.rs`.

### Request-log redaction — keep query text out of the `--verbose` log (`sq-toze.34`, ON by default)

`--verbose` installs `tower_http::trace`, whose default span records the request **URI** — and
for `GET /sparql?query=…` the *full SPARQL query text is in that URI*, where it can carry PII (a
patient IRI, an email in a `FILTER`, a literal in `INSERT DATA`). Logging it verbatim leaks
sensitive content into operator logs.

**Redaction is ON by default** (always compiled — no feature gate): with `--verbose` the request
log keeps the URI *path* verbatim but replaces the *query string* with `?<redacted len=N fp=…>`
— a length signal + a stable, non-reversible FNV-1a fingerprint (the same construction the audit
log uses). Logs stay correlation-useful (same query → same `fp`) without exposing content.
**`--log-full-requests`** (env `SPARQ_LOG_FULL_REQUESTS=1`) opts OUT and logs the URI verbatim,
as the bare TraceLayer did — the deliberate debug escape hatch. Library: `ServerConfig {
verbose: true, redact_logs: true /* default */, .. }`.

**Default rationale:** enabling `--verbose` for debugging should not silently write
potentially-sensitive query text to disk / a SIEM; content-logging is the *deliberate* choice.
**Honest boundary — log-CONTENT redaction, not anonymity:** the log still records method,
path/endpoint, status, a size signal and timing (it would not be a request log otherwise), so
an adversary still learns *that* a request of roughly-this-size hit *this* endpoint at *this*
time and (via `fp`) that the same query recurred. That metadata is not erased. It is also NOT
the ZK/MPC privacy story — purely operator-log hygiene, complementary to error-body sanitisation
(`sq-kfel`/#241) and the audit fingerprint. See `crates/sparq-server/src/redact.rs`.

### Structured access-audit sink — opt-in pluggable JSON-Lines trail (`sq-gos8`, epic sq-toze, ASVS V7 / ISO 27001 A.8.15 / CDMC CD-2)

A **richer, structured** sibling of the `audit-log` trail above, for compliance audit trails that
need a TYPED, self-describing access record per ENFORCED decision rather than a flat `tracing`
line. **Opt-in:** compile with the `access-audit` cargo feature, then configure a sink with
`--access-audit <file|stderr>` (env `SPARQ_ACCESS_AUDIT`; the literal `stderr` writes to stderr,
any other value is a file path). Off (no feature, or no sink configured), the module + every call
site are `#[cfg]`-stripped / short-circuited (`Option` check) — a request pays essentially zero.

It hooks the **real enforcement seam** (the same `auth_gate` that actually allows/denies the
request), so the recorded decision is the one the server enforced — never a claimed-but-
disconnected one. Each event is emitted through a pluggable **`AuditSink` trait** (the default
`WriterSink` writes one JSON object per line; heavy/external sinks — a SIEM client, an OTel
exporter — stay OUT of core, an embedder implements the trait and installs an
`Arc<dyn AuditSink>`). Record fields:

| field | meaning |
| --- | --- |
| `ts` | RFC-3339 UTC timestamp (`YYYY-MM-DDTHH:MM:SS.mmmZ`) |
| `actor` | `anonymous`, `token:<fnv1a>` (Bearer fingerprint, **never the raw token**), or `webid:<iri>` (an authenticated WebID/agent IRI — recorded verbatim) |
| `action` | `query` / `update` / `graph_read` / `graph_write` |
| `resource` + `resource_kind` | the dataset (`/sparql`) or the named-**graph IRI** the request addressed (`named_graph`) |
| `decision` | `allow` / `deny` (the ACTUALLY-enforced outcome) |
| `policy_basis` | the enforcement reason (`bearer-auth: allowed` / `bearer-auth: missing or invalid token`) |
| `fingerprint` | FNV-1a hash (hex) of the trimmed query/update — **not the query text** (`-` for a GSP body) |
| `status` | the HTTP status the client saw |
| `duration_us` | handler wall-clock, microseconds |

**Privacy boundary — stated honestly:** an audit trail exists to record WHO accessed WHAT, so —
**by design, and unlike the request log** — this sink **records identities and resource IRIs**
(the actor + the named-graph IRI are first-class fields; that is the operator's deliberate opt-in
choice). What it does **NOT** record is query **CONTENT**: the query/update text is logged only as
its non-reversible `fingerprint`, never raw, because a query body can carry PII (a patient IRI in a
`FILTER`, an email literal) — the #241 / sq-toze.34 redaction posture. It does **not** double-log
the content the redaction work just protected. One line: **identities + resources are logged;
content stays fingerprinted.** Library callers set `ServerConfig { access_audit:
Some(SinkTarget::File(path)), .. }` (field present only with the feature). See
`crates/sparq-server/src/access_audit.rs`.

## Gotchas / feature flags / prerequisites

- **Auth — optional Bearer write gate; loopback-by-default; non-loopback bind refused
  unless safe.** With no `--auth-token` the server is unauthenticated (read+write open to
  anyone who can reach the port) — the back-compat default. Set `--auth-token <TOKEN>` (env
  `SPARQ_AUTH_TOKEN`) to require `Authorization: Bearer <TOKEN>` on every WRITE (SPARQL Update
  + GSP `PUT`/`POST`/`DELETE`); `401` + `WWW-Authenticate: Bearer` otherwise (constant-time
  compared; QLever's `-a <token>`; missing-vs-wrong are indistinguishable). The classification
  keys on whether the request **mutates**, not the route — an Update smuggled through the
  query path is gated too. Add `--auth-token-read` (env `SPARQ_AUTH_TOKEN_READ=1`) to ALSO
  gate reads — INCLUDING the subscription transports (`/subscriptions` WS + `/subscriptions/sse`
  SSE, both a read surface), closing the prior read-auth bypass (bead `sq-cxk5`): the SSE GET
  takes the `Authorization: Bearer` header; the WS upgrade accepts that header OR (for browsers)
  a `Sec-WebSocket-Protocol: bearer.<token>` subprotocol. The binary binds
  `127.0.0.1:3030` by default and **refuses** a non-loopback
  `--addr` (incl. `0.0.0.0`/`::`) unless `--allow-remote` / `SPARQ_ALLOW_REMOTE=1` OR the
  whole surface is authenticated (`--auth-token` AND `--auth-token-read`); even then it warns.
  A write-token alone still leaves reads open, so it does NOT by itself make a remote bind
  safe. Deliver the token over TLS (terminate at a proxy). For per-user authz front it with a
  reverse proxy / gateway (or `sparq-solid`). The token is authentication, NOT a resource
  cap: it is orthogonal to the DoS caps. No rate limit; `--max-results` / `--max-query-rows`
  are unlimited by default — set the four hardening caps (timeout, memory, decompression-ratio,
  SERVICE allowlist; see "Server hardening — the four DoS/SSRF limits") **plus** a gateway
  rate limiter before exposing it (beads `sq-zcby`, `sq-o4qf`, `sq-ebii`, `sq-4w18`).
- **SERVICE federation (egress allowlist).** `SERVICE` is OFF in the default build (build
  with `--features service` to enable it). Even enabled, the server is **default-DENY-all
  SERVICE**: a `SERVICE <iri>` clause reaches **nothing** unless its host is allowlisted —
  via `--service-allow HOST` / `*.SUFFIX` (repeatable), `--service-allow-file PATH` (one
  entry per line), or `SPARQ_SERVICE_ALLOW` (comma/whitespace-separated). All three are
  combined additively (the CLI only ever widens the env baseline). Rationale: a `SERVICE`
  clause turns attacker-controlled query text into an outbound request from the server
  host (textbook SSRF; worst case the `169.254.169.254` cloud-metadata IP), so the
  network-exposed surface must opt in to every reachable host. Matching is
  case-insensitive against the SERVICE IRI authority; a `*.example.org` entry matches the
  apex `example.org` and any subdomain. Unlike the engine's standalone default (which lets
  public IPs through and only blocks private ones), the server is **strict**: even a public
  host must be on the allowlist. The allowlist applies uniformly to queries, ASK,
  CONSTRUCT/DESCRIBE, subscriptions and federated `INSERT … WHERE` updates, and is enforced
  before any socket is opened (DNS-rebinding-safe, on the *resolved* IP). The startup log
  prints the effective allowlist. Beads `sq-4w18` (this wiring), `sq-2v6f` (the engine SSRF
  filter). Embedders that drive the engine directly use
  `sparq_engine::with_service_egress_policy(strict, [host], || …)` /
  `with_service_egress_allow([host], || …)`.
- **Feature flags.** `server` (default-on) pulls axum/tokio/tower — the binary needs it
  (`required-features = ["server"]`). `time-travel` (default **off**) enables
  `?generation=N` pinning, the `Sparq-Generation` header, `AppState::at`, and the
  retention flags. `geo` (default **off**) installs sparq-geo's `geof:` GeoSPARQL
  functions on query/update/subscription paths; without it an unknown `geof:` IRI is a
  `500`. `federation-descriptors` (default **off**, `sq-d3d8`) pulls the light
  `sparq-introspect` crate and serves the OPT-IN VoID + Service-Description discovery
  endpoints (still gated at runtime by `--federation-descriptors`; see "Federation
  discovery"). Run feature tests: `cargo test -p sparq-server --features time-travel` /
  `--features geo` / `--features federation-descriptors`.
- **Named graphs are real (since conformance round 3).** The engine stores the FULL dataset
  — default graph + named graphs — so `GRAPH <g> { … }` / `GRAPH ?g { … }` evaluate, and a
  GSP graph resource (`?graph=<iri>` or the direct request URI) addresses a genuine named
  graph (no longer a default-graph alias). `FROM`/`FROM NAMED` re-scope the active dataset, and
  the protocol's dataset-override params are **applied** (sq-z33x), not just accepted:
  `default-graph-uri`/`named-graph-uri` synthesize the query's active dataset (replacing any
  in-query `FROM`/`FROM NAMED` per §2.1.4), and `using-graph-uri`/`using-named-graph-uri` re-scope
  an update's `WHERE` (per §2.2; combining with an in-update `USING`/`USING NAMED`/`WITH` is a
  `400`, as is a non-IRI graph value). Time-travel pinning is `/sparql` queries only (GSP
  read/write and subscriptions always operate on current).
- **Update operations.** Engine handles `INSERT DATA`, `DELETE DATA`, `CLEAR`/`DROP`/`CREATE`
  (DEFAULT / named / ALL), `LOAD`, and `DELETE/INSERT … WHERE` — over the default graph AND
  named graphs. A failing operation is refused with `400`, atomically (no partial effect
  published). `apply_update` **blocks** (group-commit + O(graph) fork) — never call it on the
  async runtime directly; the HTTP handler (and the GSP write verbs) already use
  `spawn_blocking`.
- **Multi-operation update bodies are accepted and applied ATOMICALLY (one request, one
  commit).** A SPARQL 1.1 Update request is a sequence of `;`-separated operations, and the
  endpoint takes the WHOLE body as ONE update (it is never split on `;`): e.g. a Solid PSS
  `putDocument` body — `DROP SILENT GRAPH <r> ; INSERT DATA { GRAPH <r> … ; GRAPH <parent>
  ldp:contains <r> }` — is one request → a single `204`, with the resource graph AND the parent
  containment triple either BOTH applied or (on any operation failing) NEITHER. The sequenced
  writer applies the body to a private fork and publishes only on full success, so a partial
  body is never visible (in-memory all-or-nothing). On `--persist` the whole body's resolved
  delta is committed as ONE fsync'd transaction-journal frame BEFORE the `204`, so a crash
  mid-body can never leave the parent `ldp:contains` desynced from the child graph it points at
  (the journal redoes the whole frame, or none of it, on `Graph::open`). (sq-ycle / gh-48; see
  `crates/sparq-server/tests/persist.rs::pss_combined_multiop_body_accepted_and_atomic` and
  `::invalid_second_op_leaves_no_partial_write`.)
- **GSP write created-vs-replaced status is advisory.** PUT/POST sample graph existence from
  the current generation to choose `201` vs `204`/`200`; the write itself is atomic on the
  sequenced writer regardless. An existing-but-empty named graph reads as absent (the engine
  has no separate "empty graph exists" bit outside an in-flight update), so it may report
  `201` on a write — this never affects correctness of the data, only the status code.
- **Durability — opt-in via `--persist DIR` (default in-memory).** With NO `--persist` the
  server is in-memory and updates are **lost on restart** (the back-compat default). Pass
  `--persist <DIR>` (env `SPARQ_PERSIST_DIR`) to make the on-disk index at `DIR` the durable,
  rebuildable source of truth (QLever's `--persist-updates`): every committed UPDATE — default
  graph AND named graphs — is write-ahead-logged + **fsync'd before the group-commit ack** (the
  `204`), so a process restart on the same `DIR` restores **all** prior updates with **no
  rebuild** (`Graph::open` replays the WAL). On startup an existing store at `DIR` is opened (its
  WAL replayed; any `DATA_FILE` seed ignored — the persisted store wins); an empty `DIR` is
  seeded from `DATA_FILE`. Library callers set `ServerConfig::persist_dir` and build with
  `AppState::try_with_config` (returns the durable-open error). Deferred hardening (beaded):
  byte-accounted durability metrics, graceful degradation on a *transient* disk error (today a
  durability failure refuses the write rather than losing it), and WAL-durable `CLEAR`/`DROP
  GRAPH <g>` of an existing named graph. (sq-7cxr / gh-44.)
- **Time-travel memory cost is real.** Each retained generation is a *full* `Graph` today
  (~780 MB/generation at 1M triples); size `--time-travel-generations` accordingly.
- **Error bodies.** Every error is structured JSON `{"error": "..."}` with
  `Content-Type: application/json` (the `405` keeps its `Allow` header). POST query
  requires `Content-Type: application/sparql-query` or `application/x-www-form-urlencoded`
  (else `415`); a GET without `query=` is `400`.
- **Transient vs permanent status contract (for retry classifiers — sq-r5bv / gh-50).** A retry
  classifier should treat **only `429` and `503` as transient** (a retry of the identical request
  may succeed): `429` is a concurrency shed (the request never ran), `503` is a query/UPDATE
  **timeout**, a durable-write refusal (write NOT applied), or a subscription-capacity refusal.
  Everything else is **permanent** for the identical request — `400`/`401`/`404`/`405`/`410`/
  `413`/`415` — and `500` is a defect (caught panic / unclassified internal error), not
  back-pressure. The trap a `5xx`-only classifier hits against sparq: a **`413` result/row cap is a
  PERMANENT honest refusal** (narrow the query / add `LIMIT`), not a transient signal and not a
  truncation. **Classify on the status code, not the body text** (bodies are sanitised generic
  classes — see the next bullet). There is **no `Retry-After`** header today. Full contract +
  rationale: the `sparq_server::status_contract` crate doc, asserted by `tests/status_contract.rs`.
- **Error bodies are sanitized — no information leak (sq-cz89 / sq-j9zs).** On the
  no-auth-by-default path an error body carries only a **stable, generic CLASS message**
  (e.g. `malformed query`, `malformed RDF body`, `malformed gzip body`,
  `query execution error`, `update failed: invalid SPARQL update`). It deliberately does
  **NOT** echo the caller's submitted query/UPDATE/RDF text, a fragment of the loaded RDF
  (parsers like `oxttl`/`spargebra` quote the offending token — that would confirm loaded
  triples), or any server-side filesystem path (e.g. a `--persist` mirror's path inside a
  transient durable-write `503`). The full detailed cause is preserved for the **operator**
  via the server-side `tracing` log (target `sparq_server`), surfaced only through the
  opt-in `--verbose` / `RUST_LOG` subscriber — never the HTTP response. Status semantics
  (the `400`/`413`/`415`/`503`/`500` classification) are unchanged; only the prose detail
  is withheld. Regression tests assert a sentinel token never appears in any error body.
- **mimalloc** is the binary's global allocator (matters under concurrent load).

## See also

- `serve` — the underlying `sparq-serve` generation-ring + sequenced group-commit writer
  (the concurrency primitives `router`/`AppState` wire up).
- `engine` — `sparq-engine` query/ask/construct/describe + `QueryBudget` the server drives.
- `cli` — the `sparq` command-line surface over the same engine.
- `geo` — the `geof:` GeoSPARQL extension functions enabled by `--features geo`.
- `core` — `sparq_core::Graph` (`Graph::load_str`) the dataset is loaded into.
