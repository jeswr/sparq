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

/// [OPUS-4.8] (sq-o7o0, ASVS V14.5.3) **First-party CORS origin allowlist** — the
/// [`cors_config::CorsAllowlist`] config type (parse `--cors-allow-origin` /
/// `--cors-allow-origin-file` / `SPARQ_CORS_ALLOW_ORIGIN`, exact-origin matcher). Pure
/// (no async), always compiled + unit-tested. EMPTY by default = NO CORS headers (the
/// historical, safe posture for a data API); an operator opts specific first-party
/// browser origins back in. Enforced by the [`http::harden`] middleware. See the module
/// docs for the deliberately-conservative policy (no `*`, no credentials).
pub mod cors_config;

/// [OPUS-4.8] (sq-toze.34, epic sq-toze) **Request-log redaction** — keeps SPARQL query / update
/// text out of the `--verbose` request log (a PII/privacy exposure: `GET /sparql?query=…` puts
/// the full query in the logged URI). On by default ([`ServerConfig::redact_logs`]); opt out with
/// `--log-full-requests` / `SPARQ_LOG_FULL_REQUESTS=1`. Log-CONTENT redaction, not anonymity — see
/// the module docs for the metadata boundary that remains. Always compiled + unit-tested.
pub mod redact;

#[cfg(feature = "server")]
pub mod http;

/// [OPUS-4.8] (sq-r5bv, gh-50) **Stable HTTP error/status contract** — the consumer-facing
/// transient-vs-permanent classification a client should encode (which status codes/messages
/// sparq emits for timeout / row-cap / malformed query / auth failure / etc., and which are
/// retryable). Documentation only (no code); asserted by `tests/status_contract.rs`. Gated on
/// `server` because it describes the [`http`] surface. See the module docs.
#[cfg(feature = "server")]
pub mod status_contract;

/// [OPUS-4.8] (sq-0bxp) OPT-IN per-query access audit log (CDMC CD-2 / ISO 27001 A.8.15 /
/// EU CRA logging). Compiled only behind the `audit-log` feature, and emitted only when
/// [`ServerConfig::audit_log`] (`--audit-log` / `SPARQ_AUDIT_LOG=1`) is also set. Emits a
/// structured `tracing` record per request under target `sparq_server::audit`; logs a
/// NON-reversible query fingerprint + token-identity fingerprint, never the full query text
/// or the Bearer secret (the #241 info-leak posture). See the module docs.
#[cfg(feature = "audit-log")]
pub mod audit;

/// [OPUS-4.8] (sq-gos8, epic sq-toze) OPT-IN RICHER STRUCTURED access-audit sink (ASVS V7 /
/// ISO 27001 A.8.15 / CDMC CD-2 logging). Compiled only behind the `access-audit` feature, and
/// emitted only when a sink is configured (`--access-audit <file|stderr>` /
/// `SPARQ_ACCESS_AUDIT`). Emits a TYPED access RECORD per enforced decision (actor / action /
/// resource / decision + policy-basis / timestamp / request fingerprint) as JSON-Lines through
/// the pluggable [`access_audit::AuditSink`] trait. PRIVACY BOUNDARY: identities + resource IRIs
/// are recorded by design (the audit trail); the query CONTENT only as a non-reversible
/// fingerprint, never the raw text. See the module docs.
#[cfg(feature = "access-audit")]
pub mod access_audit;

/// [OPUS-4.8] (sq-d3d8, epic sq-3183) OPT-IN federation discovery descriptors — the W3C
/// VoID dataset description (`GET /.well-known/void`) and the SPARQL 1.1 Service Description
/// (a `GET /sparql` with no `query`). Compiled only behind the `federation-descriptors`
/// feature, and served only when [`ServerConfig::federation_descriptors`] is also set. See
/// the module docs for the OPT-IN posture (feature + flag, both OFF by default).
#[cfg(feature = "federation-descriptors")]
pub mod descriptors;

/// [OPUS-4.8] (sq-bzh1, epic sq-3183) OPT-IN Triple Pattern Fragments (TPF) / Linked Data
/// Fragments (Hartig LDF) READ-ONLY source endpoint (`GET /tpf?subject=&predicate=&object=`) —
/// a paged RDF fragment of the triples matching one triple pattern, with Hydra controls
/// (`hydra:totalItems` from the cheap cardinality ESTIMATE, `hydra:next`/`hydra:previous`
/// paging, the `hydra:search` template). Pure (no async); the async handler is in
/// [`http`] (`tpf_endpoint`). Compiled only behind the `tpf` feature, and served only when
/// [`ServerConfig::tpf`] is also set (`--tpf` / `SPARQ_TPF=1`) — the same double-opt-in as
/// [`descriptors`]. See the module docs.
#[cfg(feature = "tpf")]
pub mod tpf;

/// Prometheus metrics — hand-rolled text exposition at `GET /metrics` (T22).
#[cfg(feature = "server")]
pub mod metrics;

/// SEPA-style SPARQL subscriptions over WebSocket (T23) — see `SUBSCRIPTIONS.md`.
#[cfg(feature = "server")]
pub mod subscriptions;

#[cfg(feature = "server")]
pub use http::{
    bind_posture, harden, router, serve, AppState, AuthPosture, BindPosture, PinnedGen,
    ServerConfig, GLOBAL_POD,
}; // [OPUS-4.8] sq-o4qf: bind_posture / BindPosture for the bind gate; sq-zcby: AuthPosture folds the --auth-token gate into it; sq-2gqr: serve = the accept loop with the slow-loris header-read deadline

/// [OPUS-4.8] (sq-4w18) The SERVICE egress allowlist config type, re-exported at the
/// crate root next to [`ServerConfig`].
pub use service_config::ServiceAllowlist;

/// [OPUS-4.8] (sq-o7o0, ASVS V14.5.3) The first-party CORS origin allowlist config type,
/// re-exported at the crate root next to [`ServerConfig`].
pub use cors_config::CorsAllowlist;

/// [OPUS-4.8] (sq-uqh, Wave B) Re-exported for consumers (and tests) that introspect a
/// pinned generation's per-pod epoch vector — the cache-invalidation hook the server's
/// updates feed via [`http::AppState::apply_update`]. A `PinnedGen` exposes
/// `.epochs().epoch(&PodId)` (sparq-serve's `PodEpochs`); a write to one named graph bumps
/// only that graph's `PodId` epoch, so a read scoped to another graph is not invalidated.
#[cfg(feature = "server")]
pub use sparq_serve::{Epoch, PodEpochs, PodId};
