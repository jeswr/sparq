<!-- [OPUS-4.8] sq-inzv: README brought to template. -->
# sparq-server

<p>
  <a href="https://crates.io/crates/sparq-server"><img src="https://img.shields.io/crates/v/sparq-server.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-server"><img src="https://docs.rs/sparq-server/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

A **W3C-conformant HTTP server** exposing the [sparq](../../README.md) query engine: the
**SPARQL 1.1 Protocol** (`query` + `update` at `/sparql`) and the **Graph Store HTTP Protocol**
over a `Graph`. **In-memory by default** (updates lost on restart) or **durable** with
`--persist <DIR>`. Adds content negotiation, EXPLAIN, `/metrics`, WebSocket/SSE subscriptions, and
generation-pinned snapshot reads. Reads and writes never share a lock, so queries never wait on the writer.

## 🚀 Quickstart

```sh
# serve a Turtle file on 127.0.0.1:3030 (loopback — the safe default)
cargo run -p sparq-server -- --format turtle data.ttl

# query it
curl -G http://127.0.0.1:3030/sparql --data-urlencode 'query=SELECT * WHERE { ?s ?p ?o } LIMIT 5'
```

**Read "Security posture" before exposing the port** — by default there is **no auth**, loopback only.

## ✨ Features

- **SPARQL 1.1 Protocol** — `query` (GET / POST / HEAD / the query-only HTTP `QUERY` method, for
  Oxigraph interop) and `update` (`application/sparql-update` → `204`, atomic) + dataset overrides.
- **Named graphs + Graph Store Protocol** — a full RDF dataset (`GRAPH` patterns, cross-graph
  joins, `FROM`/`FROM NAMED`, graph-scoped updates through the same writer) plus GSP `GET`/`HEAD`
  read and `PUT`/`POST`/`DELETE`/`PATCH` write (indirect `?graph=`/`?default` or direct request-URI).
  A `PATCH` applies an **atomic, graph-scoped in-place modify**: an always-on
  `application/sparql-update` body (executed atomically through the same writer, with its WHERE
  dataset scoped to the addressed graph), and — behind the OPT-IN `n3-patch` feature + `--n3-patch`
  flag — a Solid-style `text/n3` **N3-Patch** (`solid:InsertDeletePatch`).
- **Content negotiation** — q-value aware; SELECT/ASK in JSON/XML/CSV/TSV, CONSTRUCT/DESCRIBE and
  GSP read in N-Triples / prefix-Turtle / RDF-XML / **JSON-LD** (`application/ld+json` — the
  `jsonld` feature, **default-on**: both emit and accept — see "Default-on JSON-LD"); TTFB-streamed
  SELECT-JSON bodies; a present-but-unsatisfiable `Accept` is **406** (Oxigraph parity), absent/`*/*`
  keeps the default. Plus **EXPLAIN / EXPLAIN ANALYZE**, Prometheus **`/metrics`**, **WebSocket + SSE**, and a **`Sparq-Generation`** header + **`?generation=N`** snapshot pin (default build, bounded to the ring's concurrency-retention window; aged-out → `410`).
- **Durable persistence** — `--persist <DIR>` makes the on-disk index the source of truth (off by
  default, in-memory). See "Durable persistence".
- **Authentication** — optional `--auth-token <TOKEN>` Bearer write gate (constant-time; mirrors
  QLever's `-a`), plus an optional `--auth-token-read` gate for reads (also gates `/metrics` and the
  subscription streams). Off by default. See "Security posture".
- **Hardening flags** — query/UPDATE-WHERE timeouts, body cap, concurrency load-shed, slow-loris
  read timeouts, row/byte memory caps, gzip zip-bomb ratio cap, and the SERVICE egress allowlist,
  each with a `SPARQ_*` env override (honest per-cap semantics in the SKILL's "Server hardening").
- **Opt-in features** (a build without a feature carries zero code for it) — `time-travel`
  (EXTENDS `?generation=N` retention past the default concurrency window), `geo` (`geof:` functions), `service` (SERVICE federation,
  **default-deny** SSRF guard), `federation-descriptors` (VoID + Service Description discovery),
  `tpf`/`brtpf` (Triple Pattern Fragments / bind-restricted LDF source), `shacl` (`POST /shacl/validate`),
  `terse` (`POST /terse/transpile` — the verifiable LLM-ergonomic `K:<name>`→canonical-SPARQL transpiler), `n3-patch` (Solid `text/n3` N3-Patch on GSP `PATCH`),
  `backup` (no-stop-the-world `/admin/backup` snapshot + PITR delta `/admin/backup/delta` +
  `/admin/restore`; on `--persist`, `?persist=true`/`--restore-persist` writes the restore through to
  disk crash-safely so it survives a restart), `change-stream` (durable CDC — commits recorded to a segmented fsync'd log + the Neptune-`GetRecords`-shaped `GET /streams` poll, `--change-stream DIR`),
  `audit-log`/`access-audit`, `zlib-ng`.
- **Default-on JSON-LD** ([OPUS-4.8] sq-oy1f.4, epic sq-oy1f) — the `jsonld` feature is in the
  server's **default** set: `application/ld+json` joins q-value-aware RDF conneg out of the box, **both
  directions** (flattened JSON-LD on CONSTRUCT/DESCRIBE/GSP-read; `oxjsonld` GSP write body). Off via
  `--no-default-features --features server` (→ 406 read, 415 write). Full conneg ratcheting is roadmap.
- **Default-on algebra rewrite** ([FABLE-5] sq-7d3dj.30.13) — `algebra-rewrite` is in the default set too: the engine's result-equivalent pre-execution rewrite (#1735 — `FILTER(?v = <iri>)` constant folding + `!bound` anti-join), so the shipped binary runs the same plans the CLI/canonical benchmarks measure. Drop via `--no-default-features --features server,jsonld`.

## Security posture (essentials — full detail in the SKILL)

By default: **no auth, loopback-only.** Hardening is opt-in but honest where it matters.

- **Auth × bind matrix.** A non-loopback bind is refused unless `--allow-remote` is set, and a write-token counts as "auth present" for that decision **only when reads are also gated** (`--auth-token-read`) — a write-token alone still leaves reads open on a remote bind. The 401 is byte-identical for a missing vs a wrong token. The Bearer gate is one shared secret with no per-user identity — for real authz front it with a gateway or [`sparq-solid`](../sparq-solid), over TLS.
- **Error responses do not leak internals** — every error is a generic `{"error":"…"}` class (never the caller's input, an RDF fragment, a path, or a token); detail to the log, class to the body (regression-guarded by `tests/hardening.rs`). An unmatched route is a categorised `404 {"error":"not found"}` (the message never echoes the requested path).
- **Stable retry contract** — **transient** (`429`/`503`): retry; **permanent** (`4xx`): fix the request — a `413` row/byte cap is a permanent honest refusal, **not** a silent truncation; **defect** (`500`): surface, don't hot-retry. Versioned table in the `status_contract` crate doc.
- **SERVICE federation** is OFF in the default build; with `--features service` it is **default-DENY-all** (egress allowlist enforced before any socket, on the resolved IP — DNS-rebinding-safe). DoS guards on by default (query timeout, body cap, concurrency shed, 20× gzip-ratio cap, panic→`500`); **no rate limit** — add one in the gateway.
- **Request-log redaction (ON by default with `--verbose`)** — a `GET /sparql?query=…` URL can
  carry PII, so the query string becomes a length + non-reversible fingerprint
  (`--log-full-requests` opts out). **Honest boundary:** this is log-CONTENT redaction, **not**
  anonymity (method/path/status/size/timing are still recorded), and **not** the ZK/MPC privacy
  story (what a *remote party* learns from a *computation*) — purely operator-log hygiene. <!-- privacy-claims-allow: negated/historical mention — explicitly states this is NOT the ZK/MPC privacy story -->
- Hardening headers (`nosniff`, CSP, `DENY`, `no-referrer`) are always on; **CORS is OFF by
  default** (opt-in exact first-party allowlist, never `*`); opt-in **access-audit** trails record
  identities + resources by design but keep query content fingerprinted.

## Concurrency contract — N readers, 1 sequenced writer

> **N concurrent lock-free readers against 1 sequenced writer.** Reads never block; all writes are
> *serialised* through one group-committing writer. There is **no distributed lock, replication, or
> consensus**, and the engine is **not sharded per logical dataset** — horizontal scale is an
> *external* deployment concern.

Readers pin the current immutable generation; writers commit batches as new generations
([`sparq-serve`](../sparq-serve/README.md)). **This single writer IS the write ceiling, by design**
— a feature for the interactive single-resource-write workload, not a gap. An in-engine
distributed/sharded writer is an **explicit Phase-2 non-goal** ([`research/`](../../research/adr-horizontal-scaling.md), gh-52 / PSS ADR-0012; no engine code).

## Durable persistence (`--persist DIR`)

`--persist <DIR>` makes the on-disk index the durable source of truth (QLever's `--persist-updates`):
every update is WAL-fsync'd **before the `204` ack** (restart replays the WAL, no rebuild); a rejected update is never persisted, a durable-write failure refuses with a **retryable `503`** (fail-closed), and WAL compaction (`POST /admin/compact` / `sparq-cli compact`) purges deleted bytes for erasure-completeness but **cannot** reach off-box copies (snapshots/backups) — see the SKILL.

**Restore into a live durable store** (`backup` feature): a `POST /admin/restore?persist=true` (or `--restore FILE --restore-persist` on start) REPLACES the durable store's contents with a backup artifact, written through to `DIR` so it survives a restart. The swap runs on the single writer thread, crash-safely (a two-rename directory swap healed deterministically on the next open), and is **fail-closed**: a corrupt artifact is rejected with the live store untouched. Without `?persist=true`, a `--persist` server refuses the restore (`409`) — an in-memory-only restore would be silently lost on restart.

## 📚 Learn more

- **How-to** — [`skills/http-server/SKILL.md`](../../skills/http-server/SKILL.md) (every endpoint,
  status code, the full auth × bind matrix, each hardening cap's honest semantics, CORS, audit
  sinks, TPF/brTPF, SHACL, federation discovery, the container image) and
  [`SUBSCRIPTIONS.md`](SUBSCRIPTIONS.md) (the subscription protocol).
- **Wire contract** — [`docs/http-wire-contract.md`](../../docs/http-wire-contract.md): the versioned v1
  HTTP surface + wire-semver policy, pinned by `tests/wire_contract.rs` (ratification pending, gh-1416). **API reference** — [docs.rs/sparq-server](https://docs.rs/sparq-server).
- **Design** — [`research/concurrent-serving.md`](../../research/concurrent-serving.md)
  (generation-ring + sequenced-writer) and
  [`research/adr-horizontal-scaling.md`](../../research/adr-horizontal-scaling.md) (the non-goal).
- **Performance** — not baked into docs; see the
  [benchmarks dashboard](https://sparq.jeswr.org/dev/bench).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md), [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## License

[MIT](../../LICENSE).
