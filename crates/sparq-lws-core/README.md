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

Release tags publish `ghcr.io/sparq-org/sparq-lws-core` as a multi-architecture
image for `linux/amd64` and `linux/arm64`. Use an immutable `X.Y.Z` tag in a
deployment; releases also publish `X.Y` and `latest` convenience tags.

```bash
VERSION=0.1.0 # replace with the release to deploy
IMAGE="ghcr.io/sparq-org/sparq-lws-core:${VERSION}"
docker pull "${IMAGE}"

# Example with TLS terminated by a trusted reverse proxy.
docker run --rm --name sparq-lws-core \
  -p 127.0.0.1:3000:3000 \
  -e SOLID_SERVER_BASE_URL=https://solid.example \
  -e SOLID_SERVER_TRUSTED_ISSUER=https://idp.example/realms/solid \
  -e SOLID_SERVER_AUDIENCE=https://solid.example \
  "${IMAGE}"
```

The image binds `0.0.0.0:3000` through `SOLID_SERVER_BIND` and runs as uid/gid
65532. `GET /livez` and `GET /readyz` are the unauthenticated liveness and
readiness probes. All other `SOLID_SERVER_*` / `PSS_*` settings pass through to
the binary; the image enables no auth bypass or development escape hatch.

### Required production configuration

- **Public base URL:** set `SOLID_SERVER_BASE_URL` to the externally visible
  `https://` origin. Set `SOLID_SERVER_AUDIENCE` when token audience differs;
  otherwise it defaults to the base URL.
- **TLS:** either terminate TLS at a trusted ingress/reverse proxy, or mount
  readable PEM files and set both `SOLID_SERVER_TLS_CERT` and
  `SOLID_SERVER_TLS_KEY`. Setting only one fails boot. Keep the container port
  private when TLS terminates upstream.
- **Authentication and authorization:** set `SOLID_SERVER_TRUSTED_ISSUER` to
  the production HTTPS Solid-OIDC issuer. DPoP, HTTPS-only WebIDs, anonymous
  mutation rejection, and strict bidirectional WebID-to-issuer validation are
  secure defaults. Do not set `SOLID_SERVER_ALLOW_LOOPBACK`, the seed variables,
  or `SOLID_SERVER_BIDIRECTIONAL=off` in production. Resource access remains
  controlled by WAC.
- **Storage:** explicitly review `PSS_SPARQ_BACKEND` before deployment. Its
  default, `memory`, is ephemeral. The default-feature image also includes the
  `embedded` backend, selected with `PSS_SPARQ_BACKEND=embedded`; without
  `SOLID_SERVER_SPARQ_DIR` it is also ephemeral. The current native binary has
  only an in-memory blob backend and therefore deliberately refuses a durable
  SPARQ index paired with ephemeral resource bytes. Until a durable BlobStore
  implementation is available, this image is not suitable for durable
  production data. The `http` SPARQ and Redis replay backends require opt-in
  compile-time features and are not present in this default-feature image.

For in-process TLS, mount certificate material read-only and ensure uid 65532
can read it:

```bash
docker run --rm -p 3000:3000 \
  --mount type=bind,src="${PWD}/tls",dst=/run/tls,readonly \
  -e SOLID_SERVER_BASE_URL=https://solid.example \
  -e SOLID_SERVER_TRUSTED_ISSUER=https://idp.example/realms/solid \
  -e SOLID_SERVER_TLS_CERT=/run/tls/tls.crt \
  -e SOLID_SERVER_TLS_KEY=/run/tls/tls.key \
  "ghcr.io/sparq-org/sparq-lws-core:${VERSION}"
```

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
  - `http3` (off, [GPT-5.6] sq-oprna.2) — when the existing TLS PEM variables
    are configured, also serve the same hardened LDP router over HTTP/3 on UDP
    at the resolved `SOLID_SERVER_BIND` address and port. TCP remains HTTP/2 +
    HTTP/1.1; WebSocket notifications remain on TCP.
  - `redis-replay` (off) — a shared Redis-backed DPoP `jti` replay store for
    horizontally-scaled deployments.

## 📚 Learn more

- Epic sq-gg0qq tracks the migration: bench/, conformance/, docs/, decisions/
  remain in the source repo until their own beads land; sq-gg0qq.3 (landed)
  promoted the EmbeddedSparqClient to the first-class default Store backend.
- Design records: `docs/` + `decisions/` in
  [jeswr/solid-server-rs](https://github.com/jeswr/solid-server-rs) (e.g.
  `decisions/0001-embed-sparq-in-process.md`, the high-throughput PoP design).
- Related crates: [`sparq-solid`](../sparq-solid) (Solid protocol pieces inside
  sparq), [`sparq-server`](../sparq-server) (the SPARQL HTTP endpoint this
  server can delegate to).

## License

MIT OR Apache-2.0 (preserved from the source repository — see LICENSE-MIT and
LICENSE-APACHE in this directory).
