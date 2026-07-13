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
# Engine-free profile (in-memory double only)
cargo test -p sparq-lws-core --no-default-features
```

The binary is configured entirely by `SOLID_SERVER_*` / `PSS_*` environment
variables (bind address, TLS PEM paths, backend selection, seeding) — see the
module docs on `src/main.rs`.

## ✨ Features

- **LDP surface** — containers + RDF/non-RDF resources, Turtle / JSON-LD content
  negotiation (oxrdf/oxttl/oxjsonld), conditional requests, `Content-Range` reads.
- **Access control** — WAC (`acl:`) evaluated against the SPARQ-authoritative
  store, with an ACL decision cache; public-read fast path.
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
