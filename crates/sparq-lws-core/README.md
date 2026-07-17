<!-- [FABLE-5] sq-gg0qq.2: imported from jeswr/solid-server-rs@1e555b10 (see epic sq-gg0qq). -->
# sparq-lws-core

> **EXPERIMENTAL** — a parallel-track Rust implementation of a Solid/LDP (Linked Web
> Storage) server. It does **not** replace the TypeScript prod-solid-server. Imported
> whole from [jeswr/solid-server-rs](https://github.com/jeswr/solid-server-rs)
> (rev `1e555b10`); the 3-way lws-core / lws / solid-server split is a later bead.

A Solid/LDP server core: SPARQ-authoritative for RDF and access control (WAC),
`object_store`/S3 backup-only for bytes. Solid-OIDC + DPoP auth (via the pinned
[solid-oidc-verifier](https://github.com/jeswr/solid-oidc-verifier)), tiered
proof-of-possession (mTLS cert-bound tokens, DPoP-SK), notifications
(WebSocketChannel2023), and a DoS-hardened hyper/axum transport.

## 🚀 Quickstart

```bash
# Build + run the server binary (in-memory store, plain TCP)
cargo run -p sparq-lws-core
# Full test suite (default features — includes the in-process engine backend)
cargo test -p sparq-lws-core
# Core Solid tier: engine-free and `/sparql` compiled out
cargo test -p sparq-lws-core --no-default-features
# [GPT-5.6] Opt in to one periodic orphan-blob sweep task (interval in seconds)
SOLID_SERVER_RECONCILE_INTERVAL_SECS=3600 cargo run -p sparq-lws-core
```

The binary is configured entirely by `SOLID_SERVER_*` / `PSS_*` environment
variables (bind address, TLS PEM paths, backend selection, seeding) — see the
module docs on `src/main.rs`. `SOLID_SERVER_RECONCILE_INTERVAL_SECS` is unset by
default; a positive integer enables one periodic sweep using the unchanged
one-hour orphan grace period. Invalid or zero values fail boot.

## 📦 Native container image

<!-- [GPT-5.6] sq-lmz40: native image contract; distinct from the wasm/npm development host. -->

Release tags publish `ghcr.io/sparq-org/sparq-lws-core` as a multi-arch image
(`linux/amd64` + `linux/arm64`); pin an immutable `X.Y.Z` tag (also `X.Y` / `latest`).
It binds `0.0.0.0:3000` via `SOLID_SERVER_BIND`, runs as uid/gid **65532** (non-root),
serves unauthenticated `GET /livez` + `GET /readyz` probes, and enables **no** auth
bypass or dev escape hatch — all other `SOLID_SERVER_*` / `PSS_*` settings pass through.

```bash
VERSION=0.1.0   # the release to deploy
docker run --rm --name sparq-lws-core -p 127.0.0.1:3000:3000 \
  -e SOLID_SERVER_BASE_URL=https://solid.example \
  -e SOLID_SERVER_TRUSTED_ISSUER=https://idp.example/realms/solid \
  "ghcr.io/sparq-org/sparq-lws-core:${VERSION}"
```

**Required production configuration:**

- **Base URL:** `SOLID_SERVER_BASE_URL` = the public `https://` origin (set
  `SOLID_SERVER_AUDIENCE` only if the token audience differs).
- **TLS:** terminate at a trusted proxy (keep the container port private), or mount PEMs
  and set both `SOLID_SERVER_TLS_CERT` + `SOLID_SERVER_TLS_KEY` (setting one fails boot;
  uid 65532 must read them).
- **Auth:** `SOLID_SERVER_TRUSTED_ISSUER` = the production Solid-OIDC issuer. DPoP,
  HTTPS-only WebIDs, anonymous-mutation rejection, and strict WebID↔issuer checks are on
  by default — never set `SOLID_SERVER_ALLOW_LOOPBACK`, the seed vars, or
  `SOLID_SERVER_BIDIRECTIONAL=off`; access stays WAC-controlled.
- **Storage:** review `PSS_SPARQ_BACKEND` — its default `memory` (and `embedded` without
  `SOLID_SERVER_SPARQ_DIR`) is **ephemeral**, and the native binary has only an in-memory
  blob backend, so this image is **not for durable production data** yet. The `http` and
  `redis-replay` backends need opt-in compile-time features.

## ✨ Features

- **LDP surface** — containers + RDF/non-RDF resources, Turtle / JSON-LD content
  negotiation (oxrdf/oxttl/oxjsonld), conditional requests, `Content-Range` reads.
- **Access control** — WAC (`acl:`) evaluated against the SPARQ-authoritative
  store, with an ACL decision cache; public-read fast path.
- **WAC-scoped query endpoint** — [GPT-5.6] default-on, query-only
  `GET`/`POST /sparql` for SELECT, ASK, and CONSTRUCT. Each request rebuilds a
  named-graph-per-readable-resource dataset; the default graph is empty and
  unreadable or uncertain resources never reach the query engine.
- **Auth** — Solid-OIDC access tokens + mandatory DPoP, verified-token cache,
  tiered PoP: RFC 8705 mTLS cert-bound tokens and HKDF/HMAC DPoP-SK attestation.
- **Storage seams** — `Store` / `SparqClient` / `BlobStore` traits: the
  embedded in-process engine (default), in-memory double, opt-in live SPARQ
  HTTP client, and `object_store` blob backends.
- **Notification observability** — [GPT-5.6] process-wide backlog-overflow totals
  are available through `notifications::ws::NotificationMetrics::snapshot()`.
- **Transport hardening** — HTTP/2 rapid-reset and HTTP/1 slowloris guards,
  including explicit header-count, aggregate-byte, and slow-header timeout
  bounds; request timeouts, body limits, per-connection max-requests, rate
  limiting, and overload shedding.
- Cargo features:
  - `embedded-sparq` (**default-on**, sq-gg0qq.3) — the first-class in-process
    SPARQ engine backend (in-workspace path deps on `sparq-core`/`sparq-engine`);
    `--no-default-features` builds the engine-free profile.
  - `sparql-endpoint` (**default-on**, [GPT-5.6] sq-r1ei8) — the WAC-scoped,
    query-only `/sparql` route. Disable all defaults for the pure Solid core
    tier; the internal Store methods used by LDP/WAC are independent of it.
  - `http-sparq` (off) — the remote SPARQL-over-HTTP backend
    (`PSS_SPARQ_BACKEND=http`) for a shared-service deployment.
  - `http3` (off, [GPT-5.6] sq-oprna.2) — with the TLS PEM variables configured,
    also serve the same hardened LDP router over HTTP/3 on UDP at the resolved
    `SOLID_SERVER_BIND` address+port; TCP stays HTTP/2 + HTTP/1.1 (and WS).
  - `redis-replay` (off) — a shared Redis-backed DPoP `jti` replay store for
    horizontally-scaled deployments.
  - `odrl-authz` (off, [SONNET-4.6] sq-elg47) — the native ODRL policy gate seam
    on the read/query path (`authz::odrl`, attached via `LdpState::set_odrl_gate`;
    deny-overrides / permit-extends over the WAC decision, fail-closed).

## 📚 Learn more

- Epic sq-gg0qq tracks the migration: bench/, conformance/, docs/, decisions/
  stay in the source repo until their own beads land (sq-gg0qq.3 landed).
- Design records: `docs/` + `decisions/` in
  [jeswr/solid-server-rs](https://github.com/jeswr/solid-server-rs) (e.g.
  `decisions/0001-embed-sparq-in-process.md`, the high-throughput PoP design).
- Related crates: [`sparq-solid`](../sparq-solid) (Solid protocol pieces),
  [`sparq-server`](../sparq-server) (the SPARQL endpoint it can delegate to).

## License

MIT OR Apache-2.0 (preserved from the source repository — see LICENSE-MIT and
LICENSE-APACHE in this directory).
