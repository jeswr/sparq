//! sparq-server: a W3C-conformant HTTP server over the sparq query engine.
//!
//! Implements two W3C specs against an in-memory [`sparq_core::Graph`]:
//!
//! * **SPARQL 1.1 Protocol** (<https://www.w3.org/TR/sparql11-protocol/>) — the `query`
//!   operation over `GET`/`POST` at `/sparql`, with content negotiation to SPARQL Results
//!   JSON / XML / CSV / TSV (and boolean JSON/XML for `ASK`), and the `update` operation
//!   (`POST` with `application/sparql-update`), applied through sparq-serve's single
//!   sequenced group-commit writer over the lock-free generation ring (Wave A wiring —
//!   see `http::AppState` and the README's "Update concurrency model").
//! * **SPARQL 1.1 Graph Store HTTP Protocol** (<https://www.w3.org/TR/sparql11-http-rdf-update/>)
//!   — `GET`/`HEAD` on a graph resource (direct `/graphs/...` and indirect
//!   `?graph=<uri>` / `?default`). The Graph Store **write** verbs (PUT/POST/DELETE)
//!   are answered with `501`.
//!
//! The pure pieces — result serialisers ([`results`]), query classification ([`exec`]) and
//! content negotiation ([`negotiate`]) — are always compiled and unit-tested. The async
//! HTTP surface ([`http`]) is behind the default-on `server` feature so the wasm build (and
//! any consumer that only wants the serialisers) never pulls axum/tokio.

pub mod exec;
pub mod negotiate;
pub mod results;

#[cfg(feature = "server")]
pub mod http;

/// Prometheus metrics — hand-rolled text exposition at `GET /metrics` (T22).
#[cfg(feature = "server")]
pub mod metrics;

/// SEPA-style SPARQL subscriptions over WebSocket (T23) — see `SUBSCRIPTIONS.md`.
#[cfg(feature = "server")]
pub mod subscriptions;

#[cfg(feature = "server")]
pub use http::{harden, router, AppState, PinnedGen, ServerConfig, GLOBAL_POD};

/// [OPUS-4.8] (sq-uqh, Wave B) Re-exported for consumers (and tests) that introspect a
/// pinned generation's per-pod epoch vector — the cache-invalidation hook the server's
/// updates feed via [`http::AppState::apply_update`]. A `PinnedGen` exposes
/// `.epochs().epoch(&PodId)` (sparq-serve's `PodEpochs`); a write to one named graph bumps
/// only that graph's `PodId` epoch, so a read scoped to another graph is not invalidated.
#[cfg(feature = "server")]
pub use sparq_serve::{Epoch, PodEpochs, PodId};
