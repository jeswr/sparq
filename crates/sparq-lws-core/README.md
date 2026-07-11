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
# Full test suite (default features)
cargo test -p sparq-lws-core
# With the in-process SPARQ engine backend (opt-in)
cargo test -p sparq-lws-core --features embedded-sparq
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
- **Storage seams** — `Store` / `SparqClient` / `BlobStore` traits: in-memory
  double, live SPARQ HTTP client, and `object_store` blob backends.
- **Transport hardening** — HTTP/2 rapid-reset + slowloris guards, request
  timeouts, body limits, per-connection max-requests, rate limiting, overload
  shedding.
- Cargo features (all **off** by default; the default build carries no
  sparq-engine or redis dependency):
  - `embedded-sparq` — the in-process SPARQ engine backend (in-workspace path
    deps on `sparq-core`/`sparq-engine`; formerly git deps in the source repo).
  - `redis-replay` — a shared Redis-backed DPoP `jti` replay store for
    horizontally-scaled deployments.

## 📚 Learn more

- Epic sq-gg0qq tracks the migration: bench/, conformance/, docs/, decisions/
  remain in the source repo until their own beads land; the EmbeddedSparqClient
  promotion to a first-class Store backend is sq-gg0qq.3.
- Design records: `docs/` + `decisions/` in
  [jeswr/solid-server-rs](https://github.com/jeswr/solid-server-rs) (e.g.
  `decisions/0001-embed-sparq-in-process.md`, the high-throughput PoP design).
- Related crates: [`sparq-solid`](../sparq-solid) (Solid protocol pieces inside
  sparq), [`sparq-server`](../sparq-server) (the SPARQL HTTP endpoint this
  server can delegate to).

## License

MIT OR Apache-2.0 (preserved from the source repository — see LICENSE-MIT and
LICENSE-APACHE in this directory).
