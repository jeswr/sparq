# sparq-server

<p>
  <a href="https://crates.io/crates/sparq-server"><img src="https://img.shields.io/crates/v/sparq-server.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-server"><img src="https://docs.rs/sparq-server/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

A **W3C-conformant HTTP server** exposing the [sparq](../../README.md) query engine.

Implements the **SPARQL 1.1 Protocol** (`query` + `update` at `/sparql`) and the **Graph
Store HTTP Protocol** (read + write) over an in-memory `Graph`, with `Accept`-driven content
negotiation (SELECT/ASK JSON/XML/CSV/TSV; CONSTRUCT/DESCRIBE + Graph Store
N-Triples/Turtle/RDF-XML), EXPLAIN, Prometheus `/metrics`, WebSocket subscriptions, and an
opt-in time-travel feature. Reads and writes never share a lock (a generation-ring snapshot
chain + a single sequenced group-commit writer), so queries never wait on the writer.

## Security posture — no built-in auth (read before exposing)

**`sparq-server` has NO authentication on any endpoint** — including the mutating
`application/sparql-update` path and the `/subscriptions` WebSocket. Anyone who can reach the
port can read AND write the whole dataset. Authorization belongs to a layer in front (a
reverse proxy / gateway, or [`sparq-solid`](../sparq-solid)). Therefore:

- The server **binds loopback by default** (`127.0.0.1:3030`); a non-loopback bind is
  **refused** unless you opt in with `--allow-remote` (env `SPARQ_ALLOW_REMOTE=1`).
- SPARQL `SERVICE` federation is **OFF in the default build** (the `service` cargo feature).
  Built with `--features service` it is **default-DENY-all**: a `SERVICE <iri>` reaches
  nothing unless its host is on the egress allowlist (`--service-allow` / `--service-allow-file`
  / `SPARQ_SERVICE_ALLOW`). This is an SSRF guard — a `SERVICE` clause turns attacker-supplied
  query text into an outbound request from the server host (worst case cloud-metadata). The
  allowlist is enforced before any socket opens, on the resolved IP (DNS-rebinding-safe).
- DoS guards that ARE on by default: query timeout (`503`), body cap (`413`), concurrency
  load-shedding (`429`), panic→`500`. There is **no rate limit**, and `--max-results` is
  unlimited by default — set both in the gateway / flags if you expose the port.

## 🚀 Quickstart

```sh
# serve a Turtle file on 127.0.0.1:3030 (loopback — the safe default)
cargo run -p sparq-server -- --format turtle data.ttl

# query it
curl -G http://127.0.0.1:3030/sparql --data-urlencode 'query=SELECT * WHERE { ?s ?p ?o } LIMIT 5'
```

## ✨ Features

- **SPARQL 1.1 Protocol** — `query` (GET / POST direct / POST url-encoded / HEAD) and
  `update` (`application/sparql-update` → `204`, atomic).
- **Graph Store HTTP Protocol** — `GET`/`HEAD` read and `PUT`/`POST`/`DELETE` write, indirect
  (`?graph=`/`?default`) and direct (request-URI) graph identification.
- **Content negotiation** — q-value aware; SELECT/ASK in JSON/XML/CSV/TSV, CONSTRUCT/DESCRIBE
  and GSP read in N-Triples / prefix-Turtle / RDF/XML; streamed SELECT bodies.
- **Hardening flags** — `--query-timeout` / `--max-body-bytes` / `--max-concurrent` /
  `--max-results` / `--max-subscriptions*`, each with a `SPARQ_*` env override.
- **EXPLAIN / EXPLAIN ANALYZE**, Prometheus **`/metrics`**, and SEPA-style **WebSocket
  subscriptions** (live SELECT diffs).
- **Opt-in features** — `time-travel` (`?generation=N` snapshot pinning), `geo` (sparq-geo
  `geof:` functions), `service` (SERVICE federation, default-deny).

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
