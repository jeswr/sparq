# sparq-server

A **W3C-conformant HTTP server** exposing the [sparq](../../README.md) query engine.
Implements the **read** side of two W3C specifications over an in-memory
[`sparq_core::Graph`]:

* **[SPARQL 1.1 Protocol](https://www.w3.org/TR/sparql11-protocol/)** — the `query`
  operation, and the `update` operation (`POST /sparql` with
  `Content-Type: application/sparql-update`; see the supported-operations note under
  Limitations and the [update concurrency model](#update-concurrency-model)).
* **[SPARQL 1.1 Graph Store HTTP Protocol](https://www.w3.org/TR/sparql11-http-rdf-update/)**
  — `GET`/`HEAD` (read) AND `PUT`/`POST`/`DELETE` (write) on a graph resource, via both
  indirect (`?graph=<iri>` / `?default`) and direct (request-URI-as-graph) identification.
  The write verbs translate into a server-minted SPARQL Update submitted through the same
  sequenced group-commit writer the `application/sparql-update` operation uses (so they
  share its atomicity and the **no-auth posture below** — a GSP write is as powerful as an
  UPDATE). Implemented in bead `sq-gxsj`. <!-- [OPUS-4.8] -->

<!-- [OPUS-4.8] sq-o4qf / sq-2v6f -->
## Security posture (no built-in auth) — read before exposing it

**`sparq-server` has NO authentication on any endpoint.** This is by design: the engine is
not an auth boundary — authorization belongs to a layer in front of it (a reverse proxy /
API gateway, or [`sparq-solid`](../sparq-solid/README.md)). Concretely:

* Every endpoint is **unauthenticated**, including the **mutating** `application/sparql-update`
  path on `/sparql` and the `/subscriptions` WebSocket. Anyone who can reach the port can
  **read AND write the entire dataset**.
* Therefore the server **binds loopback by default** (`127.0.0.1:3030`), reachable only from
  the same host. A **non-loopback** bind (e.g. `--addr 0.0.0.0:8080`) is **refused** unless
  you explicitly opt in with **`--allow-remote`** (env `SPARQ_ALLOW_REMOTE=1`). Even with the
  opt-in, the server logs a loud warning at startup. (`0.0.0.0` / `::` — bind-all-interfaces —
  count as remote: they are the usual way the surface gets exposed.)

  > Do **not** expose `0.0.0.0` to an untrusted network without a reverse proxy / gateway (or
  > sparq-solid) that enforces authentication and, if you need it, authorization, rate
  > limiting, and TLS in front of this server.

* **DoS controls that ARE built in** (safe defaults, see [Hardening flags](#hardening-flags-t15)):
  per-request **query timeout** (30 s → 503), **request body cap** (1 MiB → 413),
  **concurrency limit** with load-shedding (32 in-flight → 429), structured JSON errors, and
  panic→500 recovery. An **opt-in SELECT row cap** (`--max-results`) gives an honest 413
  refusal rather than silent truncation.
* **DoS controls that are NOT built in:** there is **no rate limit**, `--max-results` is
  **unlimited by default**, and there is no query-complexity bound. If you expose the server,
  set `--max-results`/`--max-concurrent` appropriately and put a rate limiter in the gateway.
  (Tracked: bead `sq-ebii` — the broader timeout/memory/rate-limit/SSRF policy.)
* **`SERVICE` federation SSRF — gated behind a default-deny egress allowlist** <!-- [OPUS-4.8] sq-4w18 -->.
  SPARQL `SERVICE` (federated query) is **OFF in the default build** — gated behind the
  server's non-default `service` cargo feature (which turns on the engine's `service` feature;
  off, `ureq` is absent from the dependency tree and a `SERVICE` clause errors at execution).
  Built with `--features service`, the server is **default-DENY-all SERVICE**: a `SERVICE <iri>`
  clause reaches **nothing** unless its host is on the egress allowlist. A `SERVICE` clause turns
  attacker-controlled query text into an outbound request from the server host (textbook SSRF;
  worst case `169.254.169.254` cloud-metadata), so the operator must opt in to every reachable
  host. Configure the allowlist with any of (UNIONed, additive):
  * `--service-allow HOST` / `--service-allow *.SUFFIX` — repeatable; exact host or suffix wildcard;
  * `--service-allow-file PATH` — one entry per line (`#` comments + blanks ignored);
  * `SPARQ_SERVICE_ALLOW` — comma/whitespace-separated.

  Matching is case-insensitive against the SERVICE IRI authority; `*.example.org` matches the apex
  `example.org` and any subdomain. The server is **strict** — unlike the engine's standalone
  default (which permits public IPs and only blocks private/internal ones via
  `with_service_egress_allow`), here even a public host must be on the allowlist. Enforcement
  happens before any socket is opened, on the *resolved* IP (DNS-rebinding-safe), and applies
  uniformly to queries, ASK, CONSTRUCT/DESCRIBE, subscriptions and federated `INSERT … WHERE`
  updates. The startup log prints the effective allowlist. (Beads `sq-4w18` this wiring, `sq-2v6f`
  the engine SSRF filter; engine seam `crates/sparq-engine/src/service.rs`
  `with_service_egress_policy`.)

## Running

```sh
# build (native; the server stack is behind the default-on `server` feature)
cargo build -p sparq-server

# serve a Turtle file on the default address 127.0.0.1:3030 (loopback — safe default)
cargo run -p sparq-server -- --format turtle data.ttl

# custom address / format (turtle | ntriples | nquads | trig). A NON-loopback bind is
# REFUSED unless --allow-remote (or SPARQ_ALLOW_REMOTE=1) is set — see "Security posture".
# Only do this behind a reverse proxy / gateway that enforces auth:
cargo run -p sparq-server -- --addr 0.0.0.0:8080 --allow-remote --format ntriples data.nt

# no data file => empty default graph (still answers queries)
cargo run -p sparq-server

# GeoSPARQL: the opt-in `geo` feature installs sparq-geo's geof: functions
# (distance / sf* relations / envelope / boundary / convexHull) on the query,
# update and subscription paths — see the "GeoSPARQL" section below
cargo run -p sparq-server --features geo -- --format turtle places.ttl
```

## GeoSPARQL (`geof:`) — opt-in `geo` feature

Built with `--features geo` (default **off**), every SPARQL endpoint evaluates
the [sparq-geo](../sparq-geo/README.md) `geof:` extension functions inside
`FILTER`/`BIND` expressions, via the engine's extension-function registry
([docs/extension-functions.md](../../docs/extension-functions.md)):

```sh
curl -G http://127.0.0.1:3030/sparql --data-urlencode query='
  PREFIX geof: <http://www.opengis.net/def/function/geosparql/>
  PREFIX uom:  <http://www.opengis.net/def/uom/OGC/1.0/>
  SELECT ?city WHERE {
    <http://ex/london> <http://ex/loc> ?here . ?city <http://ex/loc> ?there .
    FILTER(geof:distance(?here, ?there, uom:kilometre) < 400)
  }'
```

With the feature **off** (the default) nothing changes: sparq-geo and the
georust stack are not compiled in, every engine call is the registry-free one,
and an unknown `geof:` IRI is the usual hard "unsupported SPARQL function"
error (a 500). Feature-gated tests: `cargo test -p sparq-server --features geo`
(see `tests/geo.rs`).

## Time-travel queries — opt-in `time-travel` feature

Built with `--features time-travel` (default **off** — opt-in is the contract), a
query can be pinned to a **retained generation**: the sparq-serve generation ring
already keeps recent generations alive for concurrency, and a retained generation *is*
a queryable snapshot.

```sh
# every /sparql response carries the generation it was produced against
curl -siG http://127.0.0.1:3030/sparql --data-urlencode query='ASK { ?s ?p ?o }' \
  | grep -i sparq-generation
# sparq-generation: 17

# an update's 204 carries the generation CONTAINING the update (read-your-writes token)
curl -si http://127.0.0.1:3030/sparql -H 'content-type: application/sparql-update' \
  --data 'INSERT DATA { <http://ex/a> <http://ex/p> <http://ex/b> }' | grep -i sparq-generation
# sparq-generation: 18

# pin a query to a captured generation: the response is the store AS OF that generation
curl -G http://127.0.0.1:3030/sparql \
  --data-urlencode query='SELECT ?s WHERE { ?s ?p ?o }' --data-urlencode generation=17
```

* **Mechanism.** `generation=N` — URL query string or the url-encoded POST body (body
  wins, same precedence as `explain`). A generation *number* rather than an `As-Of`
  timestamp because it fits the endpoint's existing parameter contract and is the exact
  token the server hands out in the `Sparq-Generation` response header (the same
  generation-number concept as the horizontal-scaling read-your-writes `shard_seq`
  token) — no clock-resolution or skew ambiguity. Callers that track timestamps resolve
  them through the library (`sparq_serve::GenerationRing::as_of` over
  `Generation::published_at`).
* **Status semantics** (additions to the table below): pinned success → `200` +
  `Sparq-Generation`; generation aged out of the retention window → **`410 Gone`**
  (the error body names the oldest retained generation); generation never published,
  unparsable, or a pin on an *update* (updates always apply to the current generation)
  → `400`.
* **Retention.** `--time-travel-generations N` [16, env
  `SPARQ_TIME_TRAVEL_GENERATIONS`] keeps N generations older than current queryable;
  `--time-travel-max-age SECS` [off, env `SPARQ_TIME_TRAVEL_MAX_AGE`] additionally ages
  them out (pruned at the next publish, never mid-request — a pinned response always
  completes). The ring's concurrency bound K = 4 is a **floor**: time travel extends
  retention, never shrinks it below K (so `--time-travel-generations 2` still leaves
  the K window queryable).
* **Memory cost, stated honestly.** Each retained generation is a **full `Graph`**
  today (the writer's per-batch fork shares no structure — the recorded A2 trade), so
  the budget is `N × full graph`: ~780 MB per generation at 1 M triples. Size
  `--time-travel-generations` accordingly. When the structural-fork follow-up lands
  (persistent/COW indexes), retained generations become delta chains and this cost
  collapses; the API is number/timestamp-based so that swap needs no contract change
  (an OSTRICH-style delta-chain archive is the recorded follow-up).
* **Scope.** `/sparql` queries only: the Graph Store read endpoints and
  `/subscriptions` always serve the current generation; SPARQL-syntax-level `AS OF`
  and cross-generation diffs are out of scope.

With the feature **off** (the default) the parameter handling is compiled out:
`?generation=` is just an ignored unknown parameter, no `Sparq-Generation` header
exists, and the ring keeps only its concurrency window. Feature-gated tests run both
ways — `cargo test -p sparq-server` and `cargo test -p sparq-server --features
time-travel` (see `tests/time_travel.rs`).

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
| `--allow-remote` | `SPARQ_ALLOW_REMOTE` | off | Opt in to a **non-loopback** bind despite the no-auth posture. Without it (and without the env var truthy: `1`/`true`/`yes`/`on`), a non-loopback `--addr` (e.g. `0.0.0.0`) is **refused** at startup; with it, the bind proceeds but logs a loud warning. See [Security posture](#security-posture-no-built-in-auth--read-before-exposing-it). <!-- [OPUS-4.8] sq-o4qf --> |

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
refusal and slot cleanup after a dropped socket. `tests/updates.rs` covers the
generation-ring update path (Wave A): sequential visibility (group-commit ack ⇒ the
update's generation is published), atomicity + recovery after a refused update,
concurrent reads proceeding unstalled while updates commit (over HTTP and in-process),
generation pinning across commits — plus the `#[ignore]`d 1M-triple update-cost
benchmark. `tests/time_travel.rs` compiles both ways (run it with and without
`--features time-travel`): with the feature, `?generation=N` pinning end to end
(old data after subsequent updates, `Sparq-Generation` tokens, `410` on aged-out,
`400` on misuse, the K-floor retention composition); without it, that the parameter
handling is compiled out (no header, `?generation=` ignored).

## Endpoints

### SPARQL 1.1 Protocol — `query`

`/sparql`

| Request form | How |
| --- | --- |
| `GET` | `?query=<encoded>` (+ optional `default-graph-uri` / `named-graph-uri`) |
| `POST` direct | `Content-Type: application/sparql-query`, body = the query |
| `POST` url-encoded | `Content-Type: application/x-www-form-urlencoded`, `query=…` in body |
| `HEAD` | same as `GET` but no body (Content-Type + Content-Length preserved) |
| `POST` update | `Content-Type: application/sparql-update`, body = the update → `204` (failure → `400`, atomic: no partial effect) |

### Update concurrency model

Readers and the writer never share a mutable graph — and, since Wave A, never share a
lock either. The server state is a **sparq-serve generation ring** (an arc-swapped chain
of immutable `Graph` snapshots with bounded retention) plus the **single sequenced
writer** with a group-commit window (`crates/sparq-serve`, research/concurrent-serving.md
§6):

* **Queries** pin the current generation once per request (`GenerationRing::current`, a
  lock-free arc-swap load, ~10–20 ns) and evaluate against its immutable snapshot for
  the whole response — streamed bodies keep the generation pinned until the last chunk
  is written, so every response is snapshot-consistent with query *start*. Readers never
  wait on the writer, and the writer never waits on (or reclaims from) readers: old
  generations are freed by ordinary `Arc` drop when their last holder lets go.
* **Updates** submit to the sequenced writer (`POST` with `application/sparql-update`):
  every update arriving within a group-commit window (3 ms / 256 updates) is applied —
  in submission order — to one writer-private working copy and published as **one** new
  generation; the HTTP 204 returns when that generation is published (group-commit ack).
  Each batch pays one O(graph) fork (a fresh folded base) plus O(batch) in-place deltas;
  concurrent submitters amortise the fork across the window.
* **Atomicity.** A failing update is rejected (`400`) and *skipped*: the writer discards
  the working copy, re-forks, and replays the batch's other updates, so the published
  chain never contains a partial effect and batch-mates never see each other's failures.
* **Subscriptions (T23).** The commit watch advances to the published generation number
  strictly *after* the writer's ack, so a woken subscription always pins a generation at
  least as new as the commit it was woken for.
* **Pods.** Every generation carries a per-pod epoch vector (Wave B's cache-invalidation
  hook). The server currently tags every update with one **global pod**
  (`urn:sparq:pod:global`) — honest over-coarse tagging until real visibility-scope
  extraction lands with the Wave B cache work.

**What this replaced, and why.** The previous double-buffered writer (two graphs
alternating *published*/*spare*, `Arc::try_unwrap` + 200 µs reclaim polling, lag replay,
`--compact-every` overlay fold-back) had two measured pathologies
(research/concurrent-serving.md §4.3/§4.4): a reader pinning a snapshot stalled the
writer for the full reclaim wait (5.4 s, worst-case 32 s), and reclaim polling degraded
under reader churn. The ring removes both **by design** — there is no reclaim and no
poll — in exchange for bounded extra residency (the ring retains up to K = 4 old
generations) and a fork-priced batch commit (the recorded A2 trade; a cheap structural
fork is the follow-up deliverable). `--compact-every` went with it: each batch's fork
rebuilds a freshly folded base, so overlays never accumulate across batches. Reproduce
the update-cost numbers with:

```sh
cargo test -p sparq-server --release --test updates -- --ignored --nocapture
```

### EXPLAIN — query-plan introspection (T22)

Any `/sparql` **query** request (all three request forms) can ask for the engine's plan
instead of results:

| How | Effect |
| --- | --- |
| `explain` / `explain=true` / `explain=plan` parameter (URL query string, or the url-encoded body) | **planning-only dry run** — the algebra tree, the greedy (GOO) join order with per-pattern cardinality estimates, the join strategy per step (merge / hash / bind / worst-case-optimal), and pushed-down filters. Nothing is executed, so this is cheap regardless of query cost. |
| `explain=analyze` | plan **plus execution**: the query runs under the normal request budget and the response appends a per-operator trace (output rows + wall time per operator) and totals. SELECT/ASK only. |
| `Accept: text/x-sparq-explain` | same as `explain=plan` |

The response is `text/plain`. One caveat (also stated in the output header): the executor
picks the bind-join cutover from *actual* intermediate row counts at run time; the
plan-only dry run predicts that choice from the same cardinality estimates it reports.

```sh
curl 'localhost:3030/sparql?explain=true' --data-urlencode \
  'query=SELECT * WHERE { ?a <http://ex/knows> ?b . ?b <http://ex/age> ?age }' -G
# EXPLAIN (SELECT) — planning-only dry run; nothing is executed.
# Plan:
#   Project ?a, ?b, ?age
#     BGP [binary join plan: greedy GOO ordering] (2 patterns)
#       1. scan ?a <http://ex/knows> ?b (est 2 rows, sorted by ?b) [seed: smallest estimate]
#       2. merge join on ?b with scan ?b <http://ex/age> ?age (est 3 rows, sorted by ?b) → est 2 rows
```

### Prometheus metrics — `GET /metrics` (T22)

Hand-rolled Prometheus text exposition (no metrics dependency). The middleware wraps the
whole hardening stack, so shed requests (429), body-limit rejections (413) and panics
(500) are all counted with the status the client saw.

| Metric | Type | What |
| --- | --- | --- |
| `sparq_http_requests_total{endpoint,status}` | counter | requests by endpoint (`/sparql`, `/sparql/graph`, `/graphs`, `/subscriptions`, `/health`, `/metrics`, `other`) and response status |
| `sparq_query_duration_seconds` | histogram | wall time of `/sparql` requests (query **and** update operations); fixed buckets 1 ms … 10 s |
| `sparq_active_subscriptions` | gauge | currently active WebSocket subscriptions (read at scrape time) |
| `sparq_graph_triples` | gauge | triples in the published graph (read at scrape time) |
| `sparq_updates_total` | counter | successfully applied SPARQL updates |

### Graph Store HTTP Protocol — read + write <!-- [OPUS-4.8] sq-gxsj -->

| Resource | How |
| --- | --- |
| Indirect | `/sparql/graph?default` or `/sparql/graph?graph=<iri>` |
| Direct | `/graphs/<path>` (the request URI IS the graph IRI: `http://<host>/graphs/<path>`) |

**Read.** `GET`/`HEAD` serialise the addressed graph in the `Accept`-negotiated RDF syntax
(q-value aware; default N-Triples): **N-Triples** (`application/n-triples`),
**prefix-compacting Turtle** (`text/turtle`) or **RDF/XML** (`application/rdf+xml`).
<!-- [OPUS-4.8] sq-rt6v: Turtle is now a real prefix-compacting document and RDF/XML is offered. -->

**Write** (`PUT`/`POST`/`DELETE`). The request body is RDF, parsed by `Content-Type`
(`text/turtle` | `application/n-triples` | `application/n-quads` | `application/trig` |
`application/rdf+xml`; absent → Turtle); a malformed body is a `400`, an unsupported type a
`415`. The body
carries the triples for the one addressed graph (the URL names the graph, not the body —
quad-syntax graph names are folded in). Each verb is translated into a server-minted SPARQL
Update and submitted through the **same sequenced group-commit writer** the
`application/sparql-update` operation uses, so a GSP write inherits its atomicity, snapshot
consistency, the `Sparq-Generation` header (time-travel feature), and **no-auth posture**.

| Verb | Effect | Update | Status |
| --- | --- | --- | --- |
| `PUT <g>` | REPLACE the graph contents | `DROP SILENT GRAPH <g>` / `CLEAR DEFAULT` then `INSERT DATA { … }` | `201` created / `204` replaced |
| `POST <g>` | MERGE (additive) into the graph | `INSERT DATA { … }` | `201` created / `204` merged |
| `POST /sparql/graph` (no selector) | create a fresh server-named graph (§5.5) | `INSERT DATA { GRAPH <minted> { … } }` | `201` |
| `DELETE <g>` | DROP the graph (`?default` → `CLEAR DEFAULT`) | `DROP GRAPH <g>` / `CLEAR DEFAULT` | `204`; `404` if a named graph is absent |

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
| CONSTRUCT / DESCRIBE | `application/n-triples` (default) / `text/turtle` / `application/rdf+xml` | matching media `; charset=utf-8` | conformant graph result (T16, sq-rt6v): N-Triples, prefix-compacting Turtle (via `oxttl`), or RDF/XML (via `oxrdfxml`) |

Negotiation is q-value aware; unsupported / absent `Accept` defaults to JSON (SELECT/ASK)
or N-Triples (CONSTRUCT/DESCRIBE + GSP read). The graph syntaxes share one writer set
(`crate::graph`) — N-Triples (canonical line form), **prefix-compacting Turtle** (a small set
of common namespaces — rdf/rdfs/xsd/owl/foaf/dc/dcterms/skos/schema — compact to
`prefix:local`) and **RDF/XML** — used by both CONSTRUCT/DESCRIBE and the GSP read side.
<!-- [OPUS-4.8] sq-rt6v -->.

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
| SPARQL Update success | `204` |
| SPARQL Update failure (malformed / unsupported operation) | `400` (atomic — no partial effect) |
| Graph Store write success (`PUT`/`POST`) | `201` (created) / `204` (replaced/merged) |
| Graph Store `DELETE` success | `204` |
| Graph Store `DELETE` of an absent named graph | `404` |
| Graph Store write — malformed RDF body | `400` |
| Graph Store write — unsupported body `Content-Type` | `415` |

All error bodies are structured JSON: `{"error": "..."}`.

## Limitations / follow-ups

* **Named graphs.** The engine stores the FULL dataset (default + named graphs) since
  conformance round 3: `GRAPH <g> { … }` / `GRAPH ?g { … }` evaluate, `FROM` / `FROM NAMED`
  scope the active dataset, and the Graph Store named-graph selectors (`?graph=<iri>` and the
  direct request-URI form) address genuine named graphs. The protocol's `default-graph-uri`
  / `named-graph-uri` query params are accepted and threaded through. <!-- [OPUS-4.8] sq-gxsj -->
* **CONSTRUCT / DESCRIBE serialisations.** Implemented (T16) via the engine's RDF-graph
  result API (`sparq_engine::construct` / `describe`), negotiated between
  `application/n-triples`, prefix-compacting `text/turtle` and `application/rdf+xml`
  (`crate::graph`, via `oxttl` / `oxrdfxml`). <!-- [OPUS-4.8] sq-rt6v: RDF/XML + prefix Turtle landed -->
  The last result-format conformance gap is closed.
* **SPARQL Update operations.** The engine supports `INSERT DATA`, `DELETE DATA`,
  `CLEAR` / `DROP` / `CREATE` (DEFAULT / named / ALL), `LOAD`, and `DELETE/INSERT … WHERE`
  over the default graph AND named graphs; a failing operation is refused with `400`
  (atomically — see the update concurrency model). The Graph Store **write** verbs
  (`PUT`/`POST`/`DELETE`) are implemented on top of this same path (bead `sq-gxsj`). GSP
  read negotiates N-Triples / prefix-compacting Turtle / RDF/XML, and GSP write accepts an
  `application/rdf+xml` body in addition to the Turtle/N-Triples/N-Quads/TriG forms.
  <!-- [OPUS-4.8] sq-gxsj, sq-rt6v -->
* **Update durability.** The served graph is in-memory; updates are not persisted across
  a restart (the engine's WAL-backed directory graphs are a CLI/embedding feature the
  server does not use yet).
* **Time-travel retention cost.** With the opt-in `time-travel` feature, every retained
  generation is a full `Graph` (see the section above). The recorded follow-up — gated
  on the structural-fork (persistent/COW index) work — is delta-chain retention
  (OSTRICH-style snapshot + delta archive), which the number/timestamp-based API
  already accommodates without a contract change.
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
3. The **query** (including CONSTRUCT/DESCRIBE), **result-format**, **HTTP-semantics**,
   **named-graph dataset** and **Graph Store HTTP Protocol** (read + write) sections are
   expected to pass; all result formats — SPARQL-results JSON/XML/CSV/TSV (SELECT/ASK) and
   N-Triples / prefix-compacting Turtle / RDF/XML (CONSTRUCT/DESCRIBE + GSP) — are conformant.
   <!-- [OPUS-4.8] sq-rt6v: RDF/XML gap closed -->

The in-process `tests/protocol.rs` suite mirrors the same assertions and runs in CI via
`cargo test -p sparq-server`.
