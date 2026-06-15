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
