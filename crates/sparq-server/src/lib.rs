//! sparq-server: a W3C-conformant HTTP server over the sparq query engine.
//!
//! Implements two W3C specs against an in-memory [`sparq_core::Graph`]:
//!
//! * **SPARQL 1.1 Protocol** (<https://www.w3.org/TR/sparql11-protocol/>) — the `query`
//!   operation over `GET`/`POST` at `/sparql`, with content negotiation to SPARQL Results
//!   JSON / XML / CSV / TSV (and boolean JSON/XML for `ASK`), and the `update` operation
//!   (`POST` with `application/sparql-update`), applied through the double-buffered
//!   delta-overlay writer (T17 wiring — see `http::Writer` and the README's "Update
//!   concurrency model").
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
pub use http::{harden, router, AppState, ServerConfig};
