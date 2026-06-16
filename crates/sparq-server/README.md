# sparq-server

<p>
  <a href="https://crates.io/crates/sparq-server"><img src="https://img.shields.io/crates/v/sparq-server.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-server"><img src="https://docs.rs/sparq-server/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

A **W3C-conformant HTTP server** exposing the [sparq](../../README.md) query engine.

Implements the **SPARQL 1.1 Protocol** (`query` + `update` at `/sparql`) and the **Graph
Store HTTP Protocol** (read + write) over a `Graph` — **in-memory by default** (updates lost
on restart) or, with **`--persist <DIR>`**, **durable** (write-ahead-logged + fsync'd to an
on-disk index that survives a restart with no rebuild — see "Durable persistence"). Adds
`Accept`-driven content negotiation (SELECT/ASK JSON/XML/CSV/TSV; CONSTRUCT/DESCRIBE + Graph
Store N-Triples/Turtle/RDF-XML), EXPLAIN, Prometheus `/metrics`, WebSocket subscriptions, and
an opt-in time-travel feature. Reads and writes never share a lock, so queries never wait on
the writer (the concurrency model is in the design doc linked below).

## Security posture (read before exposing)

### Authentication — optional Bearer-token write gate (mirrors QLever's `-a`)

By default `sparq-server` has **no authentication** — anyone who can reach the port can read
AND write the whole dataset. You can turn on a **required Bearer-token gate on the write
surface** (PSS gh-46), with an **optional read gate**:

- **`--auth-token <TOKEN>`** (env `SPARQ_AUTH_TOKEN`) — gates **writes**. Every request that
  MUTATES the dataset must present `Authorization: Bearer <TOKEN>` or it is refused `401`
  (with `WWW-Authenticate: Bearer`). The write surface is: a SPARQL **UPDATE** on `/sparql`
  (`Content-Type: application/sparql-update`, OR a `query`/`update` body that *parses as an
  update* — classification keys on "does this mutate", not the route), and the **Graph-Store
  Protocol write methods** (`PUT`/`POST`/`DELETE`) on `/sparql/graph` and `/graphs/{*path}`.
  The token is compared in **constant time**. Scheme casing is tolerated (`Bearer`/`bearer`).
  When unset, there is **no write auth** (today's behaviour preserved exactly).
- **`--auth-token-read`** (env `SPARQ_AUTH_TOKEN_READ=1`) — ALSO gate **reads** (SPARQL query,
  GSP `GET`/`HEAD`, AND the subscription read surfaces — see below) with the same token. Off by
  default (QLever-style: writes gated, reads open). Has no effect unless a token is also configured.
- The 401 is **identical for a missing vs a wrong token**, so an attacker cannot learn whether
  a token was presented.
- The subscription transports — the **`/subscriptions` WebSocket** and the
  **`/subscriptions/sse`** Server-Sent-Events stream (both stream live SELECT diffs, a *read*
  surface) — are gated by `--auth-token-read` exactly like the other reads (bead `sq-cxk5`,
  closing the prior read-auth bypass). The **SSE** GET takes the `Authorization: Bearer <TOKEN>`
  header like any GET. The **WebSocket** upgrade accepts the token from either channel — see
  the next paragraph. With no read gate configured, both are open (back-compatible).

#### Authenticating a WebSocket subscription from a browser

A browser's `WebSocket` API **cannot set arbitrary headers** on the handshake, so it cannot send
`Authorization: Bearer`. The `/subscriptions` upgrade therefore accepts the read token from
**either** channel and validates it against `--auth-token` (constant-time) when `--auth-token-read`
is on:

- **`Authorization: Bearer <TOKEN>`** header — for non-browser clients (a CLI WS client, a proxy).
- **`Sec-WebSocket-Protocol: bearer.<TOKEN>`** subprotocol — the browser channel:
  `new WebSocket("ws://host/subscriptions", ["bearer." + token])`. The server picks the
  *first* offered subprotocol whose value starts with `bearer.`, takes the substring after the
  `bearer.` prefix as the token, and (per RFC 6455) echoes that exact subprotocol back as the
  selected one so the handshake completes. The token is **validated, not merely echoed** — a
  wrong/absent token is refused with the same `401` the HTTP surface uses, *before* the upgrade.

This is a complement to, not a replacement for, a real authorization layer (a reverse proxy /
gateway, or [`sparq-solid`](../sparq-solid)) — the gate is a single shared secret with no
per-user identity, scopes, or TLS of its own. **Deliver the token over TLS** (terminate it at a
proxy); a bare `Bearer` token on plaintext HTTP is sniffable.

### Bind posture — loopback by default; the auth × bind matrix

The server **binds loopback by default** (`127.0.0.1:3030`, reachable only from this host). A
**non-loopback** bind (e.g. `0.0.0.0`) exposes the surface to the network, so the binary
refuses it unless the posture makes it safe. The full matrix:

| `--auth-token`? | `--auth-token-read`? | `--allow-remote`? | non-loopback bind |
| --- | --- | --- | --- |
| no  | —   | no  | **refused** — read+write fully open |
| no  | —   | yes | allowed, **warns** read+write exposed |
| yes | no  | no  | **refused** — writes gated but **reads still open** |
| yes | no  | yes | allowed, **warns** reads remain open |
| yes | yes | no  | **allowed** — whole surface authenticated (still warns) |
| yes | yes | yes | **allowed** (still warns) |

The rule: a configured write-token counts as "auth present" for the bind decision **only when
reads are also gated** (`--auth-token-read`) — because a write-token alone still leaves an open
read endpoint on a remote bind. When only writes are gated, `--allow-remote` is still required
and the server warns that reads remain open. Loopback binds are always allowed.

### Security response headers (always on)

Every response — success, streamed, error and auth-gated (`401`) alike — carries a standard
hardening header set (ASVS V14.4 / ASVS-G1; beads `sq-cmvh`, `sq-2bhm`), stamped by a
`map_response` layer in `harden()`:

| Header | Value | Why |
| --- | --- | --- |
| `X-Content-Type-Options` | `nosniff` | no MIME-sniffing into a type we did not send |
| `Content-Security-Policy` | `default-src 'none'; frame-ancestors 'none'` | a data API serves no subresources/scripts; tightest CSP makes any sniffed body inert and forbids framing |
| `X-Frame-Options` | `DENY` | legacy clickjacking guard for non-CSP agents; the API is never framed |
| `Referrer-Policy` | `no-referrer` | never leak a request URL (a `query=` may carry sensitive terms) as a `Referer` |

Each header is only added when absent, so a handler may override a specific one.
**Deliberately omitted:** `Strict-Transport-Security` (this origin serves plain HTTP — HSTS
belongs on the fronting TLS proxy, setting it here is meaningless or wrongly pins a host);
`X-XSS-Protection` (deprecated, a no-op/harmful in modern browsers, superseded by CSP); and CORS
/ `Cross-Origin-*` / `Permissions-Policy` (browser-document policies with no meaning for a
no-CORS data API — adding CORS would *widen* the surface). No *blanket* `Cache-Control: no-store`
is forced: query results are uncached by default (no `ETag` / `Cache-Control: public` is ever
set), so there is nothing to tighten, and a blanket value would wrongly override `/health` and
`/metrics`. **The one targeted exception (`sq-2bhm`):** the sensitive auth-refusal — the `401`
from a gated request without a valid Bearer token — carries `Cache-Control: no-store` so a
shared cache / proxy never retains it.

### Error responses do not leak internals (ASVS V7 / ASVS-G3)

Every error response is the structured `{"error":"<message>"}` envelope, and the `<message>` is
always a **stable, generic class** — never the caller's submitted input, a fragment of the
loaded RDF, a server-side filesystem path (e.g. a `--persist` directory in an I/O error), a
secret/token, or a `Debug` dump of an internal type. The actionable category is preserved (a
client can still tell a malformed-query `400` from an auth `401` from a `404` from a `500`); only
the *internal* detail is withheld. That detail is emitted server-side instead, via `tracing`
under `target: "sparq_server"` (surfaced by the operator's opt-in `--verbose` / `RUST_LOG`
subscriber) — **detail in the log, class in the body**.

This is enforced by routing every error path that could carry sensitive content through one
`sanitized_error` helper (beads `sq-cz89` / `sq-j9zs`; [OPUS-4.8] `sq-kfel` closed the residual
raw-echo paths: the Graph-Store-Protocol write minted-update rejection, the Triple-Pattern-
Fragments term parse, the federation-descriptor serialize errors, the `--persist` compaction
failure, and the defensive middleware-error fallback). Regression-guarded by `tests/hardening.rs`
(`no_echo_*` plus `FORBIDDEN_INTERNALS`, which asserts no absolute-path prefix /
internal-type `Debug` / secret survives into any error body for the main failure classes) and
`tests/tpf.rs`. The auth `401` is additionally byte-identical for a missing vs a wrong token, so
it is not a differential-error oracle.

### SERVICE federation + DoS guards

- SPARQL `SERVICE` federation is **OFF in the default build** (the `service` cargo feature).
  Built with `--features service` it is **default-DENY-all**: a `SERVICE <iri>` reaches
  nothing unless its host is on the egress allowlist (`--service-allow` / `--service-allow-file`
  / `SPARQ_SERVICE_ALLOW`). This is an SSRF guard — a `SERVICE` clause turns attacker-supplied
  query text into an outbound request from the server host (worst case cloud-metadata). The
  allowlist is enforced before any socket opens, on the resolved IP (DNS-rebinding-safe).
- DoS guards that ARE on by default: query timeout (`503` — now on the UPDATE path too),
  body cap (`413`), concurrency load-shedding (`429`), a 20× gzip-body decompression-ratio
  cap (`413` zip-bomb guard, `sq-ebii`), panic→`500`. OFF by default (opt in if you expose
  the port): the coarse **memory cap** `--max-query-rows` (working-set row ceiling on every
  form → `413`, `sq-ebii`) and `--max-results`. There is **no rate limit** — add one in the
  gateway. See the four-limit "Server hardening" section in the SKILL for the precise
  (honest) semantics of each cap.

### Request-log redaction (`--verbose` — ON by default)

`--verbose` installs a per-request log (`tower_http::trace`). Its default span records the
request **URI** — and for the SPARQL Protocol's `GET /sparql?query=…` form the *full SPARQL
query text lives in that URI*, where it can carry PII (a patient IRI, an email in a `FILTER`, a
literal in an `INSERT DATA`). Logging that verbatim writes sensitive content into operator logs.

So **redaction is ON by default**: with `--verbose` the request log keeps the URI *path*
verbatim (it is a route, not user content) but replaces the *query string* with a
`?<redacted len=N fp=…>` placeholder — a length signal plus a stable, **non-reversible** FNV-1a
fingerprint. Logs stay useful for correlation (the same query yields the same `fp`; a size
signal from `len`) **without exposing the content**. Operators who genuinely need the raw text
in a debug session opt out explicitly with **`--log-full-requests`** (env
`SPARQ_LOG_FULL_REQUESTS=1`), which logs the URI verbatim as the bare `TraceLayer` did.

**Rationale for the default:** turning on `--verbose` for debugging should not silently start
writing potentially-sensitive query text to disk / a SIEM; a privacy-respecting server makes
content-logging the *deliberate* choice. **Honest boundary:** this is **log-CONTENT redaction,
not anonymity** — the log still necessarily records the method, the path / endpoint, the
response status, a size signal and the timing. An adversary with the redacted log still learns
*that* a request of roughly-this-size hit *this* endpoint at *this* time, and (via `fp`) that
the same query recurred. That metadata is not erased here. It is also **not** the ZK/MPC privacy
story (which concerns what a *remote party* learns from a *computation*) — purely operator-log
hygiene. Complementary to the error-body sanitisation (`sq-kfel`/#241: detail to the server log,
a generic class to the client) and the opt-in access audit log (which already logs only a query
*fingerprint*, never the text).

### Access-audit trails (opt-in — ASVS V7 / ISO 27001 A.8.15 / CDMC CD-2)

Two opt-in, off-by-default per-request access trails for compliance regimes that need more than
the aggregate `GET /metrics` counters:

- **`audit-log`** (cargo feature + `--audit-log` / `SPARQ_AUDIT_LOG=1`) emits one flat structured
  `tracing` event per request under target `sparq_server::audit` (requester fingerprint, op class,
  query fingerprint, decision, status, duration) — route it with `RUST_LOG`.
- **`access-audit`** (cargo feature + `--access-audit <file|stderr>` / `SPARQ_ACCESS_AUDIT`)
  emits a **richer, typed JSON-Lines record** through a pluggable `AuditSink` trait, hooked at the
  REAL enforcement seam so the recorded decision is the one actually enforced: **actor** (a WebID /
  agent IRI when known, else a Bearer-token fingerprint, never the raw token), **action**
  (query / update / graph read / graph write), **resource** (the named-graph IRI / dataset
  touched), **decision + policy-basis**, an RFC-3339 timestamp, and a non-reversible request
  fingerprint. The default sink writes a file or stderr; heavy/external sinks (SIEM, OTel) stay out
  of core (implement the trait). Off (no feature or no sink), every call site is `#[cfg]`-stripped /
  an `Option` check — zero cost.

  **Privacy boundary (honest):** an audit trail's purpose is to record WHO accessed WHAT, so — by
  design, unlike the request log — it **records identities and resource IRIs**. It does **NOT**
  record query **content**: the query/update text is logged only as its fingerprint, never raw (a
  query body can carry PII — the same #241 redaction posture as the error-leak guard above). One
  line: identities + resources are logged; content stays fingerprinted. See the SKILL's
  "Structured access-audit sink" section + `src/access_audit.rs`.

## 🚀 Quickstart

```sh
# serve a Turtle file on 127.0.0.1:3030 (loopback — the safe default)
cargo run -p sparq-server -- --format turtle data.ttl

# query it
curl -G http://127.0.0.1:3030/sparql --data-urlencode 'query=SELECT * WHERE { ?s ?p ?o } LIMIT 5'
```

## Durable persistence (`--persist DIR` — QLever's `--persist-updates`)

By default the server is **in-memory**: updates apply to a `Graph` held only in RAM, so they
are **lost on restart**. Pass **`--persist <DIR>`** (env `SPARQ_PERSIST_DIR`) to treat the
on-disk index at `DIR` as the **durable, rebuildable source of truth** — the equivalent of
running QLever on an on-disk index with `--persist-updates`:

```sh
# first run: seed the durable store at ./store from data.ttl, then serve
cargo run -p sparq-server -- --persist ./store --format turtle data.ttl

# apply an update (the 204 returns only after it is fsync'd to ./store)
curl -X POST http://127.0.0.1:3030/sparql -H 'content-type: application/sparql-update' \
  --data 'INSERT DATA { <http://ex/s> <http://ex/p> <http://ex/o> }'

# restart on the SAME dir — every prior update is present, with NO rebuild
cargo run -p sparq-server -- --persist ./store
curl -G http://127.0.0.1:3030/sparql --data-urlencode 'query=SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }'
```

Semantics:

- **Durability point.** Every committed SPARQL Update (the default graph **and** named graphs)
  is appended to a per-graph write-ahead log and **fsync'd before the group-commit ack** (the
  `204`). So once a write returns success it is on disk; a crash or restart re-opens the store
  and **replays the WAL** — no rebuild, no re-load.
- **Startup.** If `DIR` already holds a store it is **opened** (its WAL replayed) and any
  `DATA_FILE` seed is **ignored** — the persisted store wins, exactly like QLever's persisted
  index. If `DIR` is empty/absent, the `DATA_FILE` seed (or an empty graph) is written there
  and opened.
- **Back-compat.** Without `--persist` the behaviour is unchanged: in-memory, lost on restart.
- **Atomicity.** A rejected update is never persisted (it is never published either), and the
  durable store stays in lockstep with the published in-memory state. The durable graph is written from the
  **resolved delta** captured during the in-memory commit (not by re-executing the update text),
  so a non-deterministic or side-effecting update (`NOW()`/`RAND()`/`UUID()`/`BNODE()`,
  `LOAD <remote>`) persists the EXACT value that was acked — never a re-rolled one — so a restart
  surfaces precisely what the client saw.
- **Graceful degradation on a durable-write error (sq-vpx4).** A durable-write failure (e.g. a
  transient `ENOSPC` / I/O error on the `--persist` mirror) no longer kills the server: the
  in-flight write is **refused with `503` (retryable)** — never acked `2xx`, never published —
  and the writer thread stays alive. Reads keep being served from the last published snapshot
  (degraded read-only), and a subsequent write succeeds once durability recovers; a persistent
  error simply yields repeated `503`s. The fail-closed invariant is unchanged: a write that did
  not durably commit is never observed by any reader.
- **WAL compaction / vacuum for erasure-completeness (`sq-x32t`).** A logical `DELETE` /
  `DROP GRAPH` retracts data from the live view, but the superseded bytes linger in earlier WAL
  segments (and the dictionary) until a compaction folds the live state into a fresh base. Two
  operator-invocable ways to physically purge them:
  - **Online:** `POST /admin/compact` — **POST-only**, gated by the **write** auth token
    (`--auth-token`), like an UPDATE. Runs on the writer thread strictly **between batches** (no
    race with a concurrent write), publishes **no** generation (the live triple set is preserved
    exactly, so reads keep flowing). `200` ok; `409` if the server is in-memory (no `--persist` —
    nothing to purge); `503` on a transient durable-write error (retryable; the writer stays alive).
  - **Offline:** `sparq-cli compact <persist-dir>` (stop the server, run it, restart).

  Both **rewrite** the on-disk store to only the current live triples with a **re-interned
  (purged) dictionary**, then **atomically swap** the directory (rollback-safe two-rename, parent
  dir fsync'd between renames, WAL truncated; an interrupted swap is healed deterministically on
  the next open). So a deleted triple's value — including an orphaned **literal value** (e.g.
  personal data) — is **physically gone** from the engine's on-disk segments + dictionary, not
  merely hidden. **Honest scope:** it cannot reach bytes already copied **off-box** (filesystem
  snapshots, block-level COW history, external backups) — those remain the operator's
  responsibility. See `compliance/privacy/retention-erasure-runbook.md` §7a.
- **Deferred hardening (beaded, not yet wired):** byte-accounted durability metrics, online
  compaction tuning under sustained write load, and WAL-durable `CLEAR`/`DROP GRAPH <g>` of an
  *existing* named graph (today those operations are applied in memory and persisted only at the
  next compaction).

## 🐳 Running the container image (ghcr.io)

A container image is published to `ghcr.io/jeswr/sparq-server` on every release tag (built
from the repo `Dockerfile`; a CI smoke test runs the built image and curls `/health` + a
query before it ships). The runtime stage is distroless (no shell, no package manager).

```sh
# boots out of the box, empty default graph, no auth (see the warning it prints):
docker run --rm -p 3030:3030 ghcr.io/jeswr/sparq-server

# serve a dataset (mount it read-only):
docker run --rm -p 3030:3030 -v "$PWD/data:/data:ro" \
  ghcr.io/jeswr/sparq-server --format turtle /data/dataset.ttl

curl http://127.0.0.1:3030/health   # -> ok
curl -G http://127.0.0.1:3030/sparql --data-urlencode 'query=ASK {}'
```

**Why it binds `0.0.0.0`.** Inside a container the only useful bind is the non-loopback
`0.0.0.0` (loopback is unreachable through Docker's port mapping). The fail-closed bind
posture above refuses a non-loopback bind unless opted in, so the image sets
`SPARQ_ALLOW_REMOTE=1` to boot — running the container is itself the operator's explicit
choice to publish a network surface. **This default posture has no auth**: anyone who can
reach the published port can READ AND WRITE the dataset (the server logs a loud no-auth
WARNING on startup — heed it).

**Securing it (recommended for anything beyond local use).** Every `SPARQ_*` var is read
from the *environment*, so turn on the Bearer gate with `-e` — no flag wiring needed:

```sh
# fully gated — writes AND reads require the token (drop the second -e for QLever-style
# writes-gated/reads-open). Deliver $TOK over TLS — terminate at a reverse proxy:
docker run --rm -p 3030:3030 \
  -e SPARQ_AUTH_TOKEN="$TOK" -e SPARQ_AUTH_TOKEN_READ=1 ghcr.io/jeswr/sparq-server
```

For per-user authz, front it with a gateway or `sparq-solid`. The other `SPARQ_*` hardening
vars (timeout, body cap, concurrency, …) work the same way through `-e`. See the auth × bind
matrix under "Security posture".

## ✨ Features

- **SPARQL 1.1 Protocol** — `query` (GET / POST direct / POST url-encoded / HEAD) and
  `update` (`application/sparql-update` → `204`, atomic).
- **Durable persistence** — `--persist <DIR>` (env `SPARQ_PERSIST_DIR`) makes the on-disk index
  the source of truth: updates are WAL-fsync'd before ack and survive a restart with **no
  rebuild** (QLever's `--persist-updates`). Off by default (in-memory). See "Durable persistence".
- **Graph Store HTTP Protocol** — `GET`/`HEAD` read and `PUT`/`POST`/`DELETE` write, indirect
  (`?graph=`/`?default`) and direct (request-URI) graph identification.
- **Content negotiation** — q-value aware; SELECT/ASK in JSON/XML/CSV/TSV, CONSTRUCT/DESCRIBE
  and GSP read in N-Triples / prefix-Turtle / RDF/XML; streamed SELECT bodies.
- **Authentication** — optional `--auth-token <TOKEN>` Bearer gate on the write surface
  (constant-time compared; mirrors QLever's `-a`), with an optional `--auth-token-read` gate
  for reads. Off by default (back-compat). See "Security posture".
- **Hardening flags** — `--query-timeout` / `--max-body-bytes` / `--max-concurrent` /
  `--max-results` / `--max-query-rows` (coarse memory cap) / `--max-decompress-ratio`
  (zip-bomb guard) / `--service-allow*` (SERVICE SSRF egress) / `--max-subscriptions*`, each
  with a `SPARQ_*` env override. The four DoS/SSRF limits are documented together (with their
  honest semantics) in the SKILL's "Server hardening" section.
- **EXPLAIN / EXPLAIN ANALYZE**, Prometheus **`/metrics`**, and SEPA-style **WebSocket
  subscriptions** (live SELECT diffs).
- **Federation discovery** — opt-in (`federation-descriptors` feature + `--federation-descriptors`
  flag, both off by default): a [W3C VoID](https://www.w3.org/TR/void/) dataset description at
  `GET /.well-known/void` and a [SPARQL 1.1 Service Description](https://www.w3.org/TR/sparql11-service-description/)
  for a `GET /sparql` with no `query`. Content-negotiated RDF. See the SKILL's "Federation
  discovery" section.
- **Triple Pattern Fragments / LDF source** — opt-in (`tpf` feature + `--tpf` flag, both off by
  default; read-only): a paged [Triple Pattern Fragments](https://www.hydra-cg.com/spec/latest/triple-pattern-fragments/)
  / [Linked Data Fragments](http://linkeddatafragments.org/) endpoint at
  `GET /tpf?subject=&predicate=&object=` that lets a TPF client drive a join cheaply against this
  server. Each page carries Hydra controls (`hydra:totalItems` from the engine's cheap cardinality
  estimate, `hydra:next`/`hydra:previous` paging, the `hydra:search` template). Content-negotiated
  RDF (Turtle default). See the SKILL's "Triple Pattern Fragments" section.
- **Access-audit trails** — opt-in, off by default: `audit-log` (flat `tracing` event per request)
  and `access-audit` (richer typed JSON-Lines records via a pluggable sink — actor / action /
  resource / decision+basis / timestamp / fingerprint, hooked at the real enforcement seam).
  Identities + resources logged by design; query content stays fingerprinted. See "Access-audit
  trails".
- **Opt-in features** — `time-travel` (`?generation=N` snapshot pinning), `geo` (sparq-geo
  `geof:` functions), `service` (SERVICE federation, default-deny), `federation-descriptors`
  (VoID + Service Description discovery endpoints — see "Federation discovery"), `tpf` (Triple
  Pattern Fragments / LDF source endpoint — see "Triple Pattern Fragments / LDF source"),
  `audit-log` / `access-audit` (access-audit trails — see "Access-audit trails"), `zlib-ng`
  (native-only faster zlib-ng C backend for `Content-Encoding: gzip` request inflate; off by
  default, pure-Rust `miniz_oxide` otherwise; never in the wasm build).

## 📚 Learn more

- **How-to** — [`skills/http-server/SKILL.md`](../../skills/http-server/SKILL.md) (all
  endpoints, request/response forms, status codes, hardening flags, metrics, embedding the
  axum router) and [`SUBSCRIPTIONS.md`](SUBSCRIPTIONS.md) (the subscription protocol).
- **API reference** — [docs.rs/sparq-server](https://docs.rs/sparq-server).
- **Design** — the lock-free generation-ring + sequenced-writer concurrency model is in
  [`research/concurrent-serving.md`](../../research/concurrent-serving.md).
- **Performance** — not baked into docs; see the
  [benchmarks dashboard](https://jeswr.github.io/sparq/dev/bench) and the `#[ignore]`d
  update-cost benchmark in `tests/updates.rs`.
- **Contribute** — [`AGENTS.md`](../../AGENTS.md) and [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## License

[MIT](../../LICENSE).
