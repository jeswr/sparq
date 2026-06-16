#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

pub mod exec;
/// [OPUS-4.8] sq-rt6v: RDF *graph* serialisation (N-Triples / prefix-compacting Turtle /
/// RDF/XML) + RDF/XML body parsing for the CONSTRUCT/DESCRIBE + Graph Store read/write
/// surface. Pure (no async), like [`results`].
pub mod graph;
pub mod negotiate;
pub mod results;
/// [OPUS-4.8] (sq-4w18) SERVICE federation egress allowlist — the
/// [`service_config::ServiceAllowlist`] config type (parse `--service-allow` /
/// `--service-allow-file` / `SPARQ_SERVICE_ALLOW`, exact-host + `*.suffix` matcher).
/// Pure (no async), always compiled + unit-tested; it only *carries* the policy. The
/// policy is enforced by the engine's `service` feature, which the server enables via
/// its own opt-in `service` feature. See the module docs for the default-DENY-all
/// posture and rationale.
pub mod service_config;

#[cfg(feature = "server")]
pub mod http;

/// [OPUS-4.8] (sq-0bxp) OPT-IN per-query access audit log (CDMC CD-2 / ISO 27001 A.8.15 /
/// EU CRA logging). Compiled only behind the `audit-log` feature, and emitted only when
/// [`ServerConfig::audit_log`] (`--audit-log` / `SPARQ_AUDIT_LOG=1`) is also set. Emits a
/// structured `tracing` record per request under target `sparq_server::audit`; logs a
/// NON-reversible query fingerprint + token-identity fingerprint, never the full query text
/// or the Bearer secret (the #241 info-leak posture). See the module docs.
#[cfg(feature = "audit-log")]
pub mod audit;

/// [OPUS-4.8] (sq-d3d8, epic sq-3183) OPT-IN federation discovery descriptors — the W3C
/// VoID dataset description (`GET /.well-known/void`) and the SPARQL 1.1 Service Description
/// (a `GET /sparql` with no `query`). Compiled only behind the `federation-descriptors`
/// feature, and served only when [`ServerConfig::federation_descriptors`] is also set. See
/// the module docs for the OPT-IN posture (feature + flag, both OFF by default).
#[cfg(feature = "federation-descriptors")]
pub mod descriptors;

/// Prometheus metrics — hand-rolled text exposition at `GET /metrics` (T22).
#[cfg(feature = "server")]
pub mod metrics;

/// SEPA-style SPARQL subscriptions over WebSocket (T23) — see `SUBSCRIPTIONS.md`.
#[cfg(feature = "server")]
pub mod subscriptions;

#[cfg(feature = "server")]
pub use http::{
    bind_posture, harden, router, AppState, AuthPosture, BindPosture, PinnedGen, ServerConfig,
    GLOBAL_POD,
}; // [OPUS-4.8] sq-o4qf: bind_posture / BindPosture for the bind gate; sq-zcby: AuthPosture folds the --auth-token gate into it

/// [OPUS-4.8] (sq-4w18) The SERVICE egress allowlist config type, re-exported at the
/// crate root next to [`ServerConfig`].
pub use service_config::ServiceAllowlist;

/// [OPUS-4.8] (sq-uqh, Wave B) Re-exported for consumers (and tests) that introspect a
/// pinned generation's per-pod epoch vector — the cache-invalidation hook the server's
/// updates feed via [`http::AppState::apply_update`]. A `PinnedGen` exposes
/// `.epochs().epoch(&PodId)` (sparq-serve's `PodEpochs`); a write to one named graph bumps
/// only that graph's `PodId` epoch, so a read scoped to another graph is not invalidated.
#[cfg(feature = "server")]
pub use sparq_serve::{Epoch, PodEpochs, PodId};
