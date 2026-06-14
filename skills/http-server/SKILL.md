---
name: http-server
description: Run or point an agent at a sparq SPARQL 1.1 Protocol HTTP endpoint (sparq-server) — /sparql query+update over GET/POST, content negotiation (SELECT/ASK JSON/XML/CSV/TSV; CONSTRUCT/DESCRIBE + Graph Store N-Triples/prefix-Turtle/RDF-XML), Graph Store read AND write (PUT/POST/DELETE on graph resources, RDF/XML bodies accepted), EXPLAIN, Prometheus /metrics, WebSocket subscriptions, and opt-in time-travel ?generation pinning. Use when starting the server, querying/updating a running endpoint, choosing Accept/Content-Type, or embedding the axum router.
---

# sparq-http-server

`sparq-server` is a W3C-conformant HTTP server (axum/tokio) that exposes the sparq
query engine over an in-memory `sparq_core::Graph`. It implements the **SPARQL 1.1
Protocol** (`query` + `update` at `/sparql`) and the **Graph Store HTTP Protocol**
(read + write), with `Accept`-driven content negotiation, hardening guards, Prometheus
`/metrics`, WebSocket subscriptions, and opt-in time-travel + GeoSPARQL.

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

> **Security: no built-in auth.** Every endpoint is unauthenticated — including the
> mutating `application/sparql-update` path and the `/subscriptions` WebSocket. The
> server binds **loopback by default** and **refuses a non-loopback bind** (e.g.
> `0.0.0.0`) unless you set `--allow-remote` (env `SPARQ_ALLOW_REMOTE=1`), warning loudly
> even then. Do not expose it to an untrusted network without a reverse proxy / API
> gateway (or `sparq-solid`) enforcing auth in front. SPARQL `SERVICE` federation is OFF
> in the default build (the `service` cargo feature is off); a `SERVICE` clause then
> errors at execution. Build with `--features service` to enable it — and even then the
> server is **default-DENY-all SERVICE**: a `SERVICE <iri>` reaches **nothing** unless its
> host is on the egress allowlist (`--service-allow` / `--service-allow-file` /
> `SPARQ_SERVICE_ALLOW`; bead `sq-4w18`). This is an SSRF guard: a `SERVICE` clause turns
> attacker-controlled query text into an outbound request from the server host (worst case
> the `169.254.169.254` cloud-metadata IP). The allowlist is enforced before any socket is
> opened, on the *resolved* IP (DNS-rebinding-safe). See "SERVICE federation (egress
> allowlist)" below.

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
  (`/sparql`, `/sparql/graph`, `/graphs/*path`, `/subscriptions`, `/health`, `/metrics`).
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
- `struct ServerConfig { query_timeout: Option<Duration>, max_body_bytes: usize,
  max_concurrent: usize, max_results: Option<usize>, max_subscriptions: usize,
  max_subscriptions_per_conn: usize, verbose: bool, allow_remote: bool, /* + time_travel_*
  under feature */ }` with `ServerConfig::default()` and `ServerConfig::from_env()`.
- `fn bind_posture(addr: &SocketAddr, allow_remote: bool) -> BindPosture` — the no-auth
  bind gate the binary applies: `Loopback` (proceed), `RemoteAllowed { warning }` (proceed
  + log), or `RemoteRefused { message }` (refuse). `allow_remote` is a *binary* posture
  gate only — it does not add per-request auth and the library `router`/`harden` surface
  ignores it.
- `fn harden(routes: axum::Router, config: &ServerConfig) -> axum::Router` — wrap any
  router in the production middleware (panic→500, concurrency-limit→429, body-limit→413,
  JSON error bodies, optional trace).
- Re-exports for cache layers/tests: `PinnedGen`, `GLOBAL_POD: &str`
  (`"urn:sparq:pod:global"`), and `sparq_serve::{Epoch, PodEpochs, PodId}`.
- Serializer/negotiation helpers (always compiled, no `server` feature): module
  `sparq_server::negotiate` — `fn negotiate(accept: Option<&str>) -> Format`,
  `fn negotiate_graph(accept: Option<&str>) -> GraphFormat`; module
  `sparq_server::exec` — `fn prepare(&str) -> Result<Prepared, PrepareError>`,
  `enum QueryForm { Select, Ask, Construct, Describe }`; module `sparq_server::results`.

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
```
GSP **writes** translate into a server-minted SPARQL Update (`DROP`/`CLEAR` + `INSERT
DATA`) and submit through the SAME sequenced group-commit writer the
`application/sparql-update` operation uses — so they share its atomicity, snapshot
consistency, blocking-on-commit semantics, the `Sparq-Generation` header (time-travel
feature), AND its **no-auth** posture (a GSP write is as powerful as an UPDATE — see the
security gotcha). A malformed body → `400`; an unsupported body Content-Type → `415`.

**6. Hardening — flags / env / library.** Each flag overrides its `SPARQ_*` env var; the
env overrides the default.

| Flag | Env | Default | Effect |
| --- | --- | --- | --- |
| `--query-timeout SECS` | `SPARQ_QUERY_TIMEOUT` | `30` (`0`=off) | per-request timeout → `503` |
| `--max-body-bytes N` | `SPARQ_MAX_BODY_BYTES` | `1048576` | body cap → `413` |
| `--max-concurrent N` | `SPARQ_MAX_CONCURRENT` | `32` | in-flight cap, load-shed → `429` |
| `--max-results N` | `SPARQ_MAX_RESULTS` | unlimited (`0`=off) | SELECT row cap → honest `413` (not truncation) |
| `--max-subscriptions N` | `SPARQ_MAX_SUBSCRIPTIONS` | `256` | server-wide subs |
| `--max-subscriptions-per-conn N` | `SPARQ_MAX_SUBSCRIPTIONS_PER_CONN` | `16` | per-socket subs |
| `--verbose` | — | off | TraceLayer request logging (respects `RUST_LOG`) |
| `--allow-remote` | `SPARQ_ALLOW_REMOTE` | off | opt in to a non-loopback bind despite no auth; without it a non-loopback `--addr` is **refused** at startup, with it it warns and proceeds |
| `--service-allow HOST\|*.SUFFIX` (repeatable) | `SPARQ_SERVICE_ALLOW` (comma/ws-sep) | empty = **deny ALL SERVICE** | (feature `service`) allowlist a SERVICE egress host (exact or `*.suffix` wildcard); CLI + file + env are all merged (combined additively) |
| `--service-allow-file PATH` | — | — | (feature `service`) load allowlist entries, one per line (`#` comments + blanks ignored) |
| `--time-travel-generations N` | `SPARQ_TIME_TRAVEL_GENERATIONS` | `16` | (feature) retained generations |
| `--time-travel-max-age SECS` | `SPARQ_TIME_TRAVEL_MAX_AGE` | off | (feature) age-out window |

In a library: `AppState::with_config(graph, ServerConfig { max_concurrent: 64, ..Default::default() })`
then `router(state)`, or `harden(my_router, &config)`.

## Gotchas / feature flags / prerequisites

- **No built-in auth — loopback-by-default, non-loopback bind is refused.** Every endpoint
  (query, `application/sparql-update`, `/subscriptions` WS) is unauthenticated → read+write
  open to anyone who can reach the port. The binary binds `127.0.0.1:3030` by default and
  **refuses** a non-loopback `--addr` (incl. `0.0.0.0`/`::`) unless `--allow-remote` /
  `SPARQ_ALLOW_REMOTE=1`; even then it warns. Front it with a reverse proxy / gateway (or
  `sparq-solid`) for auth. No rate limit and `--max-results` is unlimited by default — set
  caps + a gateway rate limiter if exposing it (beads `sq-o4qf`, `sq-ebii`).
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
  `500`. Run feature tests: `cargo test -p sparq-server --features time-travel` /
  `--features geo`.
- **Named graphs are real (since conformance round 3).** The engine stores the FULL dataset
  — default graph + named graphs — so `GRAPH <g> { … }` / `GRAPH ?g { … }` evaluate, and a
  GSP graph resource (`?graph=<iri>` or the direct request URI) addresses a genuine named
  graph (no longer a default-graph alias). `FROM`/`FROM NAMED` and the protocol's
  `default-graph-uri`/`named-graph-uri` params are accepted/threaded. Time-travel pinning is
  `/sparql` queries only (GSP read/write and subscriptions always operate on current).
- **Update operations.** Engine handles `INSERT DATA`, `DELETE DATA`, `CLEAR`/`DROP`/`CREATE`
  (DEFAULT / named / ALL), `LOAD`, and `DELETE/INSERT … WHERE` — over the default graph AND
  named graphs. A failing operation is refused with `400`, atomically (no partial effect
  published). `apply_update` **blocks** (group-commit + O(graph) fork) — never call it on the
  async runtime directly; the HTTP handler (and the GSP write verbs) already use
  `spawn_blocking`.
- **GSP write created-vs-replaced status is advisory.** PUT/POST sample graph existence from
  the current generation to choose `201` vs `204`/`200`; the write itself is atomic on the
  sequenced writer regardless. An existing-but-empty named graph reads as absent (the engine
  has no separate "empty graph exists" bit outside an in-flight update), so it may report
  `201` on a write — this never affects correctness of the data, only the status code.
- **In-memory only / no durability.** Updates are not persisted across restart.
- **Time-travel memory cost is real.** Each retained generation is a *full* `Graph` today
  (~780 MB/generation at 1M triples); size `--time-travel-generations` accordingly.
- **Error bodies.** Every error is structured JSON `{"error": "..."}` with
  `Content-Type: application/json` (the `405` keeps its `Allow` header). POST query
  requires `Content-Type: application/sparql-query` or `application/x-www-form-urlencoded`
  (else `415`); a GET without `query=` is `400`.
- **mimalloc** is the binary's global allocator (matters under concurrent load).

## See also

- `serve` — the underlying `sparq-serve` generation-ring + sequenced group-commit writer
  (the concurrency primitives `router`/`AppState` wire up).
- `engine` — `sparq-engine` query/ask/construct/describe + `QueryBudget` the server drives.
- `cli` — the `sparq` command-line surface over the same engine.
- `geo` — the `geof:` GeoSPARQL extension functions enabled by `--features geo`.
- `core` — `sparq_core::Graph` (`Graph::load_str`) the dataset is loaded into.
