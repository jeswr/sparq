//! sparq-server: a W3C-conformant HTTP server over the sparq query engine.
//!
//! Implements the **read** side of two W3C specs against an in-memory
//! [`sparq_core::Graph`]:
//!
//! * **SPARQL 1.1 Protocol** (<https://www.w3.org/TR/sparql11-protocol/>) — the `query`
//!   operation over `GET`/`POST` at `/sparql`, with content negotiation to SPARQL Results
//!   JSON / XML / CSV / TSV (and boolean JSON/XML for `ASK`).
//! * **SPARQL 1.1 Graph Store HTTP Protocol** (<https://www.w3.org/TR/sparql11-http-rdf-update/>)
//!   — `GET`/`HEAD` on a graph resource (direct `/graphs/...` and indirect
//!   `?graph=<uri>` / `?default`). The **write** side (PUT/POST/DELETE) and SPARQL
//!   **Update** are out of scope here (thread **T11b**) and answered with `501`.
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

/// SEPA-style SPARQL subscriptions over WebSocket (T23) — see `SUBSCRIPTIONS.md`.
#[cfg(feature = "server")]
pub mod subscriptions;

#[cfg(feature = "server")]
pub use http::{harden, router, AppState, ServerConfig};
