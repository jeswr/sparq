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
an opt-in time-travel feature. Reads and writes never share a lock (a generation-ring snapshot
chain + a single sequenced group-commit writer), so queries never wait on the writer.

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
  GSP `GET`/`HEAD`) with the same token. Off by default (QLever-style: writes gated, reads
  open). Has no effect unless a token is also configured.
- The 401 is **identical for a missing vs a wrong token**, so an attacker cannot learn whether
  a token was presented.
- The `/subscriptions` WebSocket (live SELECT diffs, a read surface) is **not** gated by this
  token — keep it behind a proxy if it must be restricted (bead `sq-cxk5` tracks gating it).

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
- **Atomicity.** Persistence rides on the existing generation-ring + sequenced-writer model —
  a rejected update is never persisted (it is never published either), and the durable store
  stays in lockstep with the published snapshot chain. The durable graph is written from the
  **resolved delta** captured during the in-memory commit (not by re-executing the update text),
  so a non-deterministic or side-effecting update (`NOW()`/`RAND()`/`UUID()`/`BNODE()`,
  `LOAD <remote>`) persists the EXACT value that was acked — never a re-rolled one — so a restart
  surfaces precisely what the client saw.
- **Deferred hardening (beaded, not yet wired):** byte-accounted durability metrics, graceful
  degradation on a *transient* disk-I/O error (today a durability failure is fatal — the write
  is refused rather than silently lost), online compaction tuning under sustained write load,
  and WAL-durable `CLEAR`/`DROP GRAPH <g>` of an *existing* named graph (today those operations
  are applied in memory and persisted only at the next compaction).

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
- **Opt-in features** — `time-travel` (`?generation=N` snapshot pinning), `geo` (sparq-geo
  `geof:` functions), `service` (SERVICE federation, default-deny), `zlib-ng` (native-only
  faster zlib-ng C backend for `Content-Encoding: gzip` request inflate; off by default,
  pure-Rust `miniz_oxide` otherwise; never in the wasm build).

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
