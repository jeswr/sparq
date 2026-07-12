# HTTP/3 (QUIC) across all sparq HTTP servers — opt-in design (2026-07) [OPUS-4.8]

Status: **design-for-review, maintainer-requested** (epic `sq-oprna`, P1). This
record designs an **opt-in, feature-gated** HTTP/3 path for the two sparq HTTP
servers — `sparq-server` (the query API) and `sparq-lws-core` (the Solid/LDP
server) — decomposes it into disjoint child beads, and is honest about the
maturity of the Rust HTTP/3 stack. The author is a SPARQ research agent; nothing
here is implemented — it is a plan for the maintainer to review before any code
lands.

## 0. Honesty boundary (read first)

- **No QUIC/HTTP/3 exists anywhere in the workspace today.** Verified:
  `grep -rn 'quinn\|h3\|quic' crates/ --include=Cargo.toml` returns zero QUIC
  hits (only `quick-xml`); every `h3` in source is a cryptographic 3-input hash
  (`h3(...)` in `crates/sparq-zk/`) or a hypothesis label. This is greenfield.
- **The brief's transport premise is CORRECT with one refinement.**
  `sparq-server` is HTTP/1.1-**only** and terminates **plain HTTP** (no TLS at
  all) — confirmed. But `sparq-lws-core` is **not** HTTP/1.1-only: on its
  **TLS/mTLS path** it already negotiates **HTTP/2** (ALPN `["h2","http/1.1"]`,
  the hyper-util `auto::Builder` with the h2 knobs applied). Its **plain-TCP**
  path uses `axum::serve` (auto-builder default). So "no HTTP/2 even" holds for
  `sparq-server` but is **false for `sparq-lws-core`'s TLS path**. This changes
  the "cheap HTTP/2 prerequisite" recommendation per-server (§4).
- **The Rust HTTP/3 stack is pre-1.0.** The mainstream path is the `h3` crate
  (hyperium/h3, published `0.0.x`) over `h3-quinn` (`0.0.x`) over `quinn`
  (`0.11.x`, a mature 1.0-adjacent QUIC transport) over `rustls 0.23`. `h3`
  itself is explicitly a work-in-progress API and **not** a stable dependency;
  this is a real adoption risk and the single strongest argument for keeping the
  whole thing behind a default-off feature. External helper crates exist
  (`h3-axum`, `libhttp3`, `h3x`) but are early/thin — the design does **not**
  depend on them; it drives `h3` directly and dispatches into the shared
  `axum::Router` via `tower::Service`. Version pins here are the versions
  observed as current in July 2026 and are **starting points to resolve at
  implementation time**, not asserted-exact.
- **This is a work box (EC2).** No timings are canonical; none are quoted.

## 1. Problem framing

HTTP/3 runs HTTP semantics over **QUIC** (a UDP-based, TLS-1.3-integrated,
multiplexed transport) instead of TCP. Its wins over HTTP/1.1/2 are: no
head-of-line blocking across streams, faster (0/1-RTT) connection setup,
connection migration, and mandatory encryption. For sparq the realistic value is
(a) modern-client parity (browsers prefer h3 when advertised) and (b) better
behaviour for many concurrent small SPARQL requests over lossy links. It is a
**capability/parity** feature, not a throughput claim — we make **no** perf
claim here.

The core constraint: **axum/hyper do not serve HTTP/3.** hyper is TCP-only; QUIC
lives in a separate transport (`quinn`) and HTTP/3 framing in a separate crate
(`h3`). So HTTP/3 cannot be a config flag on the existing server — it is a
**second listener** (a UDP `quinn::Endpoint`) running **alongside** the existing
TCP/axum server, both dispatching into the **same** application `Router`.

### 1.1 What the code actually looks like (verified)

`sparq-server` (query API):

- Served by the crate's **own** `sparq_server::serve` (`src/http.rs:4157`), not
  `axum::serve` — it drives `hyper::server::conn::http1::Builder` directly
  (`http.rs:4227`) to get the slow-loris `header_read_timeout` timer hook. axum
  is compiled `http1`-only (`Cargo.toml`: `axum` features `["http1",...]`,
  `hyper` `["http1","server"]`) — **no `http2` feature anywhere**.
- **No TLS.** No `rustls`/`axum-server`/`tokio-rustls` dep; the crate documents
  that it "terminates PLAIN HTTP" behind a TLS-terminating proxy
  (`http.rs:3173`, `7244`).
- The app is an `axum::Router`; builder `pub fn router(state: AppState) -> Router`
  (`http.rs:2988`), hardened by `harden(routes, config) -> Router` (`http.rs:4037`).
- Uses HTTP/1 `Upgrade` for `/subscriptions` (WebSocket, `.with_upgrades()`).

`sparq-lws-core` (Solid server) — a lib **and** a binary (`src/lib.rs` +
`src/main.rs`):

- **Plain path** (`main.rs:938`): `axum::serve(listener,
  app.into_make_service_with_connect_info::<SocketAddr>())`.
- **TLS path** (`main.rs:839`): `axum_server::from_tcp_rustls(std_listener,
  config)` then `transport_config.apply_to_builder(server.http_builder())`
  (the hyper-util `auto::Builder`) — **h1 + h2**, ALPN `["h2","http/1.1"]`
  (`tls.rs:230`).
- **mTLS path** (`main.rs:911`): the **same** TLS listener wrapped with a
  `ConnPopAcceptor` to read the peer client cert — not a separate bind.
- TLS = `rustls 0.23` + `aws-lc-rs`; the crypto provider is installed **once,
  process-wide** at `main.rs:253`
  (`rustls::crypto::aws_lc_rs::default_provider().install_default()`);
  `axum-server` uses `tls-rustls-no-provider` to reuse it.
- The app is an `axum::Router`; builders
  `pub fn build_router<J,R,S>(state) -> Router` (`app.rs:149`) and
  `build_router_with_overload<J,R,S>(state, overload) -> Router` (`app.rs:172`,
  the binary's path), generic over three seams `<J,R,S>`.
- The pre-crypto rate limiter **requires `ConnectInfo<SocketAddr>`** in request
  extensions and **fails open to auth** if it is missing — so any alternative
  listener must inject the peer address as `ConnectInfo<SocketAddr>`.

Workspace MSRV is `rust-version = "1.88"` (well above quinn/h3 floors).

## 2. The building blocks (external, honest maturity)

The verified-canonical h3-over-quinn server shape (from hyperium/h3 and quinn
examples):

1. Build a `rustls::ServerConfig` (0.23, aws-lc-rs provider) with
   `alpn_protocols = vec![b"h3".to_vec()]`.
2. `let qsc = quinn::crypto::rustls::QuicServerConfig::try_from(rustls_config)?;`
   then `quinn::ServerConfig::with_crypto(Arc::new(qsc))`.
3. `let endpoint = quinn::Endpoint::server(server_config, udp_addr)?;` — a UDP
   socket, typically the **same port** as the TCP :443 (UDP/443 vs TCP/443).
4. Accept loop: `while let Some(conn) = endpoint.accept().await { … }`; per
   connection, `h3::server::Connection::new(h3_quinn::Connection::new(conn.await?))`.
5. Per request: `conn.accept().await` yields `(http::Request<()>, stream)`; read
   the body from the stream, **build an `http::Request<Body>`**, call the shared
   `tower::Service` (the `Router`) with `ServiceExt::oneshot`, then write the
   `http::Response` status/headers/body back onto the h3 stream.

**Load-bearing design point:** the `h3` request handler is **not** a
`tower::Service` and axum's `into_make_service*` is TCP-connection-shaped, so we
do **not** reuse the make-service; we clone the `Router` and call
`(&router).oneshot(req)` per h3 request. The `Router` is `Service<Request<Body>>`
and `Clone`, so this is sound. The peer `SocketAddr` from the quinn connection is
injected into `req.extensions_mut()` as `ConnectInfo<SocketAddr>` — mandatory for
`sparq-lws-core`, harmless for `sparq-server`.

Maturity caveats to carry into every PR body and any doc: `h3`/`h3-quinn` are
`0.0.x` (API churn expected); WebSocket-over-HTTP/3 (RFC 9220 Extended CONNECT)
is a **separate** effort and does **not** come for free — so `sparq-server`'s
`/subscriptions` WS upgrade and `sparq-lws-core`'s `ws` endpoint remain
**HTTP/1.1/2-only** and clients must fall back (documented, not a regression).

## 3. Design

### 3.1 Shared plumbing — a small internal helper module

The two routers differ (monomorphic vs generic `<J,R,S>`), but **both final
apps are `axum::Router`**. So the shared helper takes an **already-built
`Router`** (sidestepping the generics) plus a QUIC config and runs the accept
loop. Proposed shape (illustrative, not final):

```rust
// serve one h3 connection's requests by dispatching into a cloned axum Router
pub async fn serve_h3(
    endpoint: quinn::Endpoint,
    router: axum::Router,          // already built + hardened
    shutdown: impl Future<Output = ()>,
) -> std::io::Result<()>;

pub fn quic_server_config(
    rustls_server_config: rustls::ServerConfig, // ALPN must include b"h3"
) -> Result<quinn::ServerConfig, Http3Error>;
```

**Where the helper lives** — decision (proceed-and-document): a **new tiny
opt-in crate `sparq-http3`** (crate-type `rlib`, all behind its own build, no
default in the workspace) rather than duplicating the loop in both servers or
bloating either. Rationale: the accept/dispatch loop and the
`rustls→QuicServerConfig` bridge are genuinely common; a shared crate is the
"opt-in feature crate" pattern the repo already prefers (core stays lean; the
QUIC deps `quinn`/`h3`/`h3-quinn` live in ONE place). Each server depends on it
**only** under its own `http3` feature. The crate is **not** published initially
(`publish = false`) — it's an internal seam until the h3 stack stabilises.

### 3.2 Per-server wiring, all behind a default-off `http3` feature

`sparq-server`:

- New `http3` feature ⇒ pulls `sparq-http3` + rustls (this crate has **no TLS
  today**, so `http3` also introduces the rustls/aws-lc-rs dependency and a
  cert/key source — QUIC is **mandatorily encrypted**, there is no plaintext
  HTTP/3). New flags `--http3`, `--http3-addr`, `--tls-cert`, `--tls-key` (only
  compiled under the feature). When `--http3` is set: build `router(state)` once,
  keep serving the existing plain-HTTP TCP path unchanged, **and** spawn the
  `serve_h3` UDP listener with a clone of the same `Router`.
- This is the larger delta (introduces TLS into a plain-HTTP crate). Keep the
  plain-HTTP default byte-identical when the feature is off.

`sparq-lws-core`:

- New `http3` feature ⇒ pulls `sparq-http3`. It **already** has rustls +
  aws-lc-rs + a cert/key path (the TLS serve path) and installs the crypto
  provider process-wide, so the QUIC config **reuses** that provider and cert.
  Add `h3` to the existing ALPN list only for the QUIC endpoint (the TCP ALPN
  stays `["h2","http/1.1"]`).
- When TLS is configured **and** `http3` is on: build `build_router_with_overload`
  once, serve the existing TLS TCP path unchanged, **and** spawn `serve_h3` with
  a clone, injecting the quinn peer `SocketAddr` as `ConnectInfo<SocketAddr>`
  (mandatory — the rate limiter fails open without it).
- Smaller delta than `sparq-server` (TLS already present).

### 3.3 Alt-Svc advertising (cheap, both servers)

On the HTTP/1.1+2 responses, add `Alt-Svc: h3=":<port>"; ma=86400` so compliant
clients discover and upgrade to HTTP/3 on the next request. Implement as a tiny
tower/axum response-header layer applied in each server's router **only when
`http3` is enabled and configured** (advertising h3 when it isn't listening is a
client-visible bug). This is header-only, no crypto — the cheap `gpt`-tier slice.

### 3.4 HTTP/2 — the "cheaper prerequisite win" question, per-server

- `sparq-lws-core` **already** has HTTP/2 on its TLS path — nothing to do there.
- `sparq-server` has **no** HTTP/2 and no TLS. Adding HTTP/2 is a *separate,
  cheaper, independently-valuable* win: it means switching its bespoke
  `http1::Builder` serve to the hyper-util `auto::Builder` (h1+h2) and adding the
  `http2` feature to axum/hyper — but **h2 cleartext (h2c) is rarely used by
  browsers**, so real HTTP/2 value on `sparq-server` also needs TLS + ALPN, which
  is exactly what `http3` introduces. **Recommendation:** treat HTTP/2 on
  `sparq-server` as an **optional, separate `http2` feature bead** (not a hard
  prerequisite for h3 — the h3 listener is independent of the TCP builder). It is
  a genuine parity gap worth its own bead but should not block the h3 epic.

## 4. Recommendation

1. **Ship HTTP/3 as a default-off `http3` feature per server, dispatching into
   the existing shared `axum::Router` via a new `sparq-http3` helper crate.**
   Never touch the default (plain-HTTP for `sparq-server`, h1/h2 for
   `sparq-lws-core`) build.
2. **Do `sparq-lws-core` first** — it already has the TLS/rustls/aws-lc-rs stack,
   ALPN ownership, and provider install, so the h3 delta is smallest and lowest
   risk. `sparq-server` second (it must also acquire TLS).
3. **Keep the `sparq-http3` helper crate the single home** for `quinn`/`h3`/
   `h3-quinn` so the pre-1.0 churn is contained to one crate.
4. **HTTP/2 on `sparq-server` is a separate, non-blocking bead**, not a
   prerequisite.
5. **Carry the maturity caveat** (`h3` is `0.0.x`) in every PR body and any doc;
   no perf claims; WS-over-h3 explicitly out of scope (clients fall back).

## 5. Phased plan (each phase → a future child bead of `sq-oprna`)

Ordered; feasibility/shared plumbing first. Each bead states {crate, model_tier,
INVARIANT, ACCEPTANCE TEST, depends_on}. Security-adjacent QUIC/rustls wiring →
`fable`; header/tests → `gpt`. Bead IDs are assigned at mint time; the sequence
is the build order.

1. **`sparq-http3` helper crate + `rustls→QuicServerConfig` bridge + generic
   `serve_h3(endpoint, router, shutdown)` loop.** (crate: new `sparq-http3`;
   tier: `fable`.) INVARIANT: no default-workspace build pulls quinn/h3 (crate
   is only referenced under downstream `http3` features); the loop injects
   `ConnectInfo<SocketAddr>` and dispatches via `Router::oneshot`. ACCEPTANCE:
   a unit/integration test in the crate boots an endpoint on an ephemeral UDP
   port, an `h3-quinn` client issues a GET into a trivial 2-route `Router`, and
   the response body/status match; `cargo build -p sparq-http3` green; workspace
   default build unaffected. depends_on: none.
2. **`sparq-lws-core` `http3` feature + h3 listener wired to
   `build_router_with_overload`, reusing the process-wide aws-lc-rs provider +
   existing cert; ALPN `h3` added to the QUIC endpoint only.** (crate:
   `sparq-lws-core`; tier: `fable`.) INVARIANT: feature OFF ⇒ serve paths
   byte-identical (plain + TLS + mTLS unchanged); feature ON + TLS configured ⇒
   TCP path unchanged AND a UDP/h3 listener serves the same Router with the peer
   addr injected as `ConnectInfo`. ACCEPTANCE: feature-off `cargo build`/`clippy`
   identical to main; feature-on integration test — start with a self-signed
   cert, an h3 client GETs an LDP resource and gets the **same** status+body as
   the h1/h2 client for the same request; rate-limiter `ConnectInfo` present
   (not fail-open). depends_on: 1.
3. **`sparq-server` `http3` feature: introduce rustls/aws-lc-rs + cert/key flags
   + h3 listener wired to `router(state)`, keeping the plain-HTTP TCP path.**
   (crate: `sparq-server`; tier: `fable`.) INVARIANT: feature OFF ⇒ no rustls in
   the tree, plain-HTTP serve byte-identical; feature ON ⇒ plain-HTTP TCP path
   unchanged AND an encrypted UDP/h3 listener serves the same `router(state)`.
   ACCEPTANCE: feature-off build/clippy identical to main; feature-on test — an
   h3 client issues a SPARQL query and gets the **same** result serialization as
   the HTTP/1.1 client; `--http3` requires cert/key or errors clearly.
   depends_on: 1.
4. **Alt-Svc advertising layer (both servers), gated on `http3` being enabled +
   configured.** (crate: `sparq-http3` for the shared layer + `sparq-server` and
   `sparq-lws-core` wiring; tier: `gpt`.) INVARIANT: `Alt-Svc: h3=":<port>"`
   header appears on h1/h2 responses **only** when an h3 listener is actually
   bound; never advertised otherwise. ACCEPTANCE: with `http3` on, an h1 GET
   response carries a well-formed `Alt-Svc` naming the live h3 port; with `http3`
   off, the header is absent. depends_on: 2, 3.
5. **Cross-protocol conformance/integration test: h3 request == h1 response for a
   representative endpoint on each server, plus a WS-fallback assertion.** (crate:
   `sparq-server` + `sparq-lws-core` tests; tier: `gpt`.) INVARIANT: for a matrix
   of representative requests, the h3 response is byte-equivalent (status, media
   type, body) to the h1 response; a WS/`/subscriptions` request over h3 is
   cleanly refused/falls back (documented) and still works over h1/h2. ACCEPTANCE:
   the test passes in the `http3`-feature CI leg; feature-off CI unaffected.
   depends_on: 2, 3, 4.
6. **(Non-blocking, separable) `sparq-server` optional `http2` feature: switch the
   TCP serve to the hyper-util `auto::Builder` (h1+h2), gated + default-off,
   preserving the slow-loris `header_read_timeout` hook.** (crate: `sparq-server`;
   tier: `fable`.) INVARIANT: feature OFF ⇒ HTTP/1.1-only serve byte-identical
   incl. the header-read-timeout timer; feature ON ⇒ h1+h2 with the same timeout
   behaviour. ACCEPTANCE: feature-off build identical; feature-on test negotiates
   h2 (over TLS from bead 3) and h1 still works; slow-loris timeout test still
   passes. depends_on: 3 (for TLS/ALPN). **Not required by 1–5** — independently
   valuable parity work.

## 6. Open questions for the maintainer

- **New crate vs per-server module:** this design mints `sparq-http3`. If you'd
  rather **not** add a crate, the fallback is a private `http3` module duplicated
  (or a shared `sparq-serve`-hosted module) — say which you prefer; I've proceeded
  with the crate per the opt-in-feature-crate house pattern.
- **Cert source for `sparq-server` h3:** `sparq-server` has no cert story today
  (plain HTTP behind a proxy). h3 forces in-process TLS. Are file-path
  `--tls-cert/--tls-key` flags acceptable, or should h3 on `sparq-server` be
  deferred until there's a broader TLS story for that crate? (`sparq-lws-core`
  already has this, which is why it's sequenced first.)
- **h3 maturity gate:** are you comfortable shipping against `h3` `0.0.x`
  (default-off), or should this wait for an `h3` `0.x`-stable / a `quinn`-native
  HTTP/3 API? The design contains the churn to one crate either way.
- **WS-over-HTTP/3 (RFC 9220):** confirmed out of scope for this epic — OK?
