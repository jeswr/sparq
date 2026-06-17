//! The axum HTTP surface: routing, extraction, status/error semantics.
//!
//! Two route groups:
//!   * `/sparql` — the SPARQL 1.1 Protocol `query` operation (GET + the two POST forms).
//!   * Graph Store HTTP Protocol — `GET`/`HEAD` (read) and, since sq-gxsj, `PUT`/`POST`/
//!     `DELETE` (write) on `/graphs/*path` (direct) and on `/sparql/graph` via
//!     `?graph=<uri>` / `?default` (indirect). [OPUS-4.8] The write verbs translate into a
//!     SPARQL Update and submit through the SAME sequenced [`sparq_serve::Writer`] the
//!     `application/sparql-update` path uses, so they inherit its atomicity, group commit
//!     and snapshot-consistency — and its **no-auth** posture (a GSP write is as powerful
//!     as an UPDATE; see the README "Security posture").
//!
//! Shared state is the sparq-serve GENERATION RING + SEQUENCED WRITER (Wave A,
//! research/concurrent-serving.md §6): queries pin the current generation once per request
//! ([`GenerationRing::current`], lock-free) and evaluate against its immutable snapshot
//! for the whole response — including streamed bodies; SPARQL Update submits through the
//! single sequenced [`sparq_serve::Writer`] (group-commit window, §6.5), which publishes
//! each batch as ONE new generation. Readers never block the writer; the writer never
//! waits for (or reclaims from) readers. This replaced the previous double-buffered
//! `RwLock<Arc<Graph>>` + spare-reclaim writer, whose measured pathologies (§4.3/§4.4:
//! pinned-snapshot writer stalls and reclaim polling) the ring removes by design.

use std::collections::HashMap;
use std::net::SocketAddr; // [OPUS-4.8] sq-o4qf: bind_posture classifies the listen address
use std::sync::Arc;
use std::time::Duration;

use axum::{
    body::Bytes,
    error_handling::HandleErrorLayer,
    extract::{DefaultBodyLimit, Query, RawQuery, State},
    http::{header, HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{any, get, post},
    BoxError, Router,
};
use tower::ServiceBuilder;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::trace::TraceLayer;

use sparq_core::Graph;
use sparq_engine::QueryBudget;
use sparq_serve::{
    ApplyUpdates, Generation, GenerationRing, GraphApplier, PodId, WriteError, Writer, WriterConfig,
};

use crate::exec::{prepare, PrepareError, QueryForm};
use crate::negotiate::{negotiate, negotiate_graph, Format, GraphFormat};
use crate::results;

// ---------------------------------------------------------------------------
// SPARQL extension functions (opt-in `geo` cargo feature)
// ---------------------------------------------------------------------------

/// Runs `f` — a synchronous engine call (or a whole blocking-pool work item) — with
/// the server's SPARQL extension functions installed.
///
/// With the opt-in `geo` cargo feature this is sparq-geo's `geof_registry()`
/// (the GeoSPARQL spatial functions), built once per process and scoped
/// thread-locally around the call (the engine re-installs it inside its rayon
/// workers — see docs/extension-functions.md). Without the feature it is the
/// identity function, so the engine call is *exactly* the registry-free one —
/// zero cost, no new dependencies, byte-identical behaviour.
#[cfg(feature = "geo")]
pub(crate) fn with_extensions<T>(f: impl FnOnce() -> T) -> T {
    static GEOF: std::sync::OnceLock<sparq_engine::FunctionRegistry> = std::sync::OnceLock::new();
    sparq_engine::with_functions(GEOF.get_or_init(sparq_geo::geof_registry), f)
}

#[cfg(not(feature = "geo"))]
#[inline(always)]
pub(crate) fn with_extensions<T>(f: impl FnOnce() -> T) -> T {
    f()
}

/// [OPUS-4.8] (sq-4w18) Runs an engine call inside the full per-request engine scope:
/// the SERVICE egress allowlist policy AND the SPARQL extension functions.
///
/// With the `service` cargo feature, this installs
/// [`sparq_engine::with_service_egress_policy`] in STRICT (allowlist-only) mode for the
/// config's [`ServerConfig::service_allow`]: SERVICE may reach ONLY allowlisted hosts —
/// an empty allowlist (the default) refuses ALL federation before any network call.
/// Every engine entry point that can evaluate a `SERVICE` clause (query, ASK,
/// CONSTRUCT/DESCRIBE, the subscription re-eval, and updates with a federated WHERE) is
/// wrapped in this, so the policy applies uniformly. The policy is a thread-local
/// installed for the closure's duration; the engine re-checks it inside the ureq
/// resolver on the same thread, so it covers the blocking-pool worker that runs the call.
///
/// Without the `service` feature this is exactly [`with_extensions`] — no federation
/// code exists, so there is nothing to gate (a SERVICE clause errors at execution).
#[cfg(feature = "service")]
pub(crate) fn with_engine_scope<T>(config: &ServerConfig, f: impl FnOnce() -> T) -> T {
    sparq_engine::with_service_egress_policy(true, config.service_allow.engine_entries(), || {
        with_extensions(f)
    })
}

#[cfg(not(feature = "service"))]
#[inline(always)]
pub(crate) fn with_engine_scope<T>(_config: &ServerConfig, f: impl FnOnce() -> T) -> T {
    with_extensions(f)
}

// ---------------------------------------------------------------------------
// Hardening configuration (T15)
// ---------------------------------------------------------------------------

/// Tunable guards that make the endpoint safe to expose publicly (T15).
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Per-request query timeout. The engine's cooperative [`QueryBudget`] stops the
    /// worker mid-query at its next coarse check; a hard await-cap of
    /// `timeout + TIMEOUT_GRACE` guarantees the HTTP 503 even if the engine is inside
    /// an uninstrumented stretch. `None` disables the timeout.
    pub query_timeout: Option<Duration>,
    /// [OPUS-4.8] (sq-nulp) **Writer-side WHERE deadline for SPARQL UPDATE — a head-of-line
    /// blocking bound.** Updates are *sequenced* on a single writer thread (sparq-serve's
    /// group-commit writer), so while one update runs its `DELETE/INSERT … WHERE` to its
    /// cooperative stop, every queued update behind it waits. The plain [`query_timeout`](Self::query_timeout)
    /// already bounds the WHERE evaluation, but it is the *read* timeout (default 30 s) — far
    /// too long to hold the writer, because it bounds the offending client's own wait, not the
    /// head-of-line blocking of the whole writer queue.
    ///
    /// When set, this is a SEPARATE, typically-shorter cooperative deadline applied ONLY to
    /// the WHERE phase of an UPDATE on the writer thread: the update's budget deadline becomes
    /// `min(query_timeout, update_where_timeout)`, so any single update releases the writer
    /// within this bound and the queue behind it cannot be head-of-line blocked longer than
    /// that — *no matter how long the read timeout is*. `None` (the default) keeps the
    /// historical behaviour exactly: the update WHERE budget is the plain [`query_timeout`](Self::query_timeout).
    ///
    /// This is a cooperative deadline (checked at the engine's coarse sites — operator entry /
    /// per outer-loop iteration), so it bounds head-of-line blocking *approximately*, like
    /// every other [`QueryBudget`] deadline: an uninstrumented stretch can overrun until the
    /// next check. It is a tunable backstop, not a hard preemption (true preemption of the
    /// writer is out of scope — see `sparq_serve`'s scheduler docs). Set via
    /// `--update-where-timeout` / `SPARQ_UPDATE_WHERE_TIMEOUT` (seconds; `0` disables).
    pub update_where_timeout: Option<Duration>,
    /// Maximum accepted request body, enforced before the handler reads it (413).
    pub max_body_bytes: usize,
    /// Maximum in-flight requests; excess requests are shed with 429.
    pub max_concurrent: usize,
    /// Maximum SELECT result rows. Exceeding it is an honest 413 refusal (the engine
    /// aborts evaluation via the row budget), never a silent truncation.
    pub max_results: Option<usize>,
    /// [OPUS-4.8] (sq-ebii) **Memory cap (coarse).** Upper bound on the row count of any
    /// *materialised intermediate or final* result the engine builds for ONE query, on
    /// EVERY form (SELECT / ASK / CONSTRUCT / DESCRIBE / GSP-read), enforced via the
    /// engine's cooperative [`QueryBudget::max_rows`] working-set bound — the query aborts
    /// (413) the moment any materialised result crosses it. This is the OOM guard for a
    /// pathological query whose join blows up the intermediate cardinality.
    ///
    /// **What it actually bounds — be precise:** it caps the NUMBER OF ROWS, not bytes.
    /// Peak heap is roughly `max_query_rows × (per-row term cost)`, so it is a *cardinality*
    /// ceiling, not a hard byte guarantee: a query that materialises few but very wide rows
    /// (many projected vars, or huge string literals) can still use more memory than the row
    /// count suggests, and the engine's non-row allocations (dictionary growth on UPDATE, a
    /// CONSTRUCT template, sort/group scratch) are outside this bound. It is also approximate
    /// in *time*: the budget is checked at coarse sites (operator entry / per outer loop
    /// iteration), so a single uninstrumented stretch can transiently exceed it before the
    /// next check. Treat it as a blunt anti-OOM circuit-breaker, NOT an RSS quota. `None`
    /// (the default) disables it. For the byte-accounted companion that DOES price row
    /// width and computed-literal size, see [`max_query_bytes`](Self::max_query_bytes).
    ///
    /// Distinct from [`max_results`]: this caps the working set on *all* forms, whereas
    /// `max_results` is folded into the budget only on the paths that pass
    /// `make_budget(_, true)` — SELECT (the final projection) AND CONSTRUCT/DESCRIBE (their
    /// WHERE-pattern solution count) AND EXPLAIN ANALYZE. It is NOT applied on the
    /// `make_budget(_, false)` paths — ASK and GSP-read — nor to UPDATE (`update_budget`,
    /// which has no projection). When both apply, the effective cap is the tighter of the two.
    pub max_query_rows: Option<usize>,
    /// [OPUS-4.8] (sq-s5is) **Byte-accounted memory cap.** The byte-accounted companion to
    /// [`max_query_rows`](Self::max_query_rows): an upper bound, in BYTES, on the engine's
    /// estimated working-set size for ONE query, on EVERY form (SELECT / ASK / CONSTRUCT /
    /// DESCRIBE / GSP-read / UPDATE-WHERE), enforced via [`QueryBudget::max_bytes`]. A query
    /// whose estimate crosses it aborts (413) at the next coarse cooperative check.
    ///
    /// **Why it exists — the row cap's blind spots:** `--max-query-rows` counts ROWS, so it
    /// under-prices (a) FEW but very WIDE rows (many projected variables → more bytes per row)
    /// and (b) huge query-COMPUTED literals (BIND / aggregate / CONSTRUCT scratch interned
    /// into the per-query local vocabulary — non-row allocations). This cap prices BOTH:
    /// `rows × width × size_of::<Id>()` for each materialised intermediate PLUS the bytes of
    /// the computed terms. A 10-column join and a 1-column join with the same row count now
    /// have different byte budgets, and a `BIND(CONCAT(…huge…))` over a handful of rows is
    /// caught even though the row count is tiny.
    ///
    /// **Honest scope — still a coarse circuit-breaker, NOT an exact RSS quota.** The
    /// estimate is a portable LOWER bound on real heap (it ignores allocator overhead,
    /// `SmallVec` inline-vs-spill, and the graph dictionary / index memory that pre-exists the
    /// query); it is checked at the SAME coarse sites as the row cap, so a single
    /// uninstrumented stretch can transiently overshoot before the next check; and it bounds
    /// the QUERY working set, not process RSS. It is strictly TIGHTER and more
    /// width/literal-aware than the row cap, not a hardware-enforced quota. `None` (the
    /// default) disables it; it composes with `--max-query-rows` and `--max-results`
    /// (whichever ceiling trips first aborts).
    pub max_query_bytes: Option<usize>,
    /// [OPUS-4.8] (sq-ebii) **Decompression-ratio cap (zip-bomb guard).** When a request
    /// body arrives `Content-Encoding: gzip` (the GSP write / RDF-load path), the server
    /// streams the inflate but refuses once the decompressed size would exceed
    /// `min(max_decompress_ratio × compressed_len, max_body_bytes)` — so a tiny highly-
    /// compressible body cannot inflate into an OOM. Rejected with 413 BEFORE the full
    /// decompressed image is held. `0` disables ratio-capped decompression entirely (a
    /// `Content-Encoding` body is then refused outright — fail-closed). Default 20×.
    pub max_decompress_ratio: usize,
    /// Maximum active subscriptions per WebSocket connection (T23); further `subscribe`
    /// requests on the socket are refused with a protocol error message.
    pub max_subscriptions_per_conn: usize,
    /// Maximum active subscriptions across the whole server (T23).
    pub max_subscriptions: usize,
    /// How many generations older than current stay queryable via `?generation=N`
    /// (opt-in `time-travel` feature). Composes with the ring's concurrency bound
    /// K = 4 as a floor (`max(K, this)` — see `sparq_serve::TimeTravelConfig`).
    /// **Memory cost is real:** each retained generation is a FULL `Graph` until
    /// the structural-fork follow-up lands.
    #[cfg(feature = "time-travel")]
    pub time_travel_generations: usize,
    /// Additionally age time-travel generations out after this duration (pruned at
    /// the next publish; the K newest are never age-evicted). `None` = count-bounded
    /// only.
    #[cfg(feature = "time-travel")]
    pub time_travel_max_age: Option<Duration>,
    /// Log every request/response via `tower_http::trace::TraceLayer`.
    pub verbose: bool,
    /// [OPUS-4.8] (sq-toze.34, epic sq-toze) **Redact request content from the `--verbose`
    /// request log.** When `true` (the **default**), the request log records the URI's *path*
    /// verbatim but replaces its *query string* — where `GET /sparql?query=…` carries the full
    /// SPARQL query text, which can contain PII (a patient IRI, an email in a `FILTER`) — with a
    /// `<redacted len=N fp=…>` placeholder: a length signal plus a stable NON-reversible
    /// fingerprint, so logs stay correlation-useful (same query => same `fp`) without exposing
    /// content. When `false` (`--log-full-requests` / `SPARQ_LOG_FULL_REQUESTS=1`), the URI is
    /// logged verbatim, exactly as the bare `TraceLayer` did — the deliberate debug escape hatch.
    ///
    /// **Default ON, rationale:** a privacy-respecting server should not leak request content into
    /// operator logs by accident; turning `--verbose` on for debugging should not silently start
    /// writing potentially-sensitive query text to disk / a SIEM. Operators who genuinely need the
    /// raw text opt in explicitly. This is **log-CONTENT redaction, not anonymity**: the log still
    /// records method, path/endpoint, status, a size signal and timing (it would not be a request
    /// log otherwise). It is also NOT the ZK/MPC privacy story. Inert unless `verbose` is also on
    /// (no request log => nothing to redact). See [`crate::redact`].
    pub redact_logs: bool,
    /// [OPUS-4.8] (sq-0bxp) **Per-query access audit log** runtime switch (CDMC CD-2 / ISO
    /// 27001 A.8.15 / EU CRA logging). When `true`, every query / update / Graph-Store request
    /// emits a structured `tracing` record under the dedicated `target: "sparq_server::audit"`
    /// — requester identity (a Bearer-token fingerprint or `anonymous`, NEVER the secret),
    /// operation class, a NON-reversible query fingerprint (NOT the full query text — the #241
    /// info-leak posture), the access decision (allowed / denied + reason), the HTTP status /
    /// result-row count and the duration. Operators route the target to their compliance sink
    /// via `RUST_LOG`. `false` (the default) emits nothing — the audit instrumentation is a
    /// single boolean check before any record is built, so an audit-disabled request pays
    /// essentially zero. Set by `--audit-log` / `SPARQ_AUDIT_LOG=1`. Present only with the
    /// `audit-log` cargo feature; without it the field, the flag and every call site are
    /// compiled out, so a request pays EXACTLY zero (byte-identical to before). See
    /// [`crate::audit`].
    #[cfg(feature = "audit-log")]
    pub audit_log: bool,
    /// [OPUS-4.8] (sq-gos8, epic sq-toze) **Richer STRUCTURED access-audit sink** target (ASVS
    /// V7 / ISO 27001 A.8.15 / CDMC CD-2). `Some(target)` installs the default JSON-Lines sink
    /// ([`crate::access_audit::WriterSink`]) writing to a file or stderr; every enforced access
    /// decision is then recorded as a TYPED record (actor / action / resource / decision +
    /// policy-basis / timestamp / non-reversible request fingerprint). `None` (the default)
    /// installs no sink — every call site is a single `Option` check, so an audit-disabled
    /// request pays essentially zero. Set by `--access-audit <file|stderr>` /
    /// `SPARQ_ACCESS_AUDIT`. PRIVACY BOUNDARY: identities + resource IRIs are recorded by design
    /// (the audit trail); query CONTENT only as a fingerprint, never the raw text. Present only
    /// with the `access-audit` cargo feature; without it the field + every call site are
    /// compiled out (byte-identical to before). See [`crate::access_audit`].
    #[cfg(feature = "access-audit")]
    pub access_audit: Option<crate::access_audit::SinkTarget>,
    /// [OPUS-4.8] sq-o4qf: explicit opt-in to bind a **non-loopback** address.
    ///
    /// By default the server has **no authentication** on any endpoint — including the
    /// mutating `application/sparql-update` path and the `/subscriptions` WebSocket.
    /// Binding a non-loopback address (e.g. `0.0.0.0`) therefore exposes the entire dataset
    /// for **read AND write** to anyone who can reach the port. To make that a deliberate
    /// act rather than a foot-gun, the binary REFUSES to bind a non-loopback address unless
    /// this is set (CLI `--allow-remote` / env `SPARQ_ALLOW_REMOTE=1`) OR the whole surface
    /// is authenticated by [`auth_token`](Self::auth_token) AND
    /// [`auth_token_read`](Self::auth_token_read) (sq-zcby) — a write-token alone still
    /// leaves reads open, so it does NOT by itself make a remote bind safe. Even an allowed
    /// remote bind logs a warning. Loopback binds are unaffected. See [`bind_posture`] for
    /// the decision, and `crates/sparq-server/README.md` → "Security posture".
    ///
    /// This is purely a *bind-time* posture gate in the binary; it does not add any
    /// per-request auth and does not affect the library `router`/`harden` surface.
    pub allow_remote: bool,
    /// [OPUS-4.8] sq-zcby (PSS gh-46): the required Bearer token that gates the **write
    /// surface**. When `Some(token)`, every request that MUTATES the dataset must present
    /// `Authorization: Bearer <token>` (scheme casing tolerated) or it is refused `401`
    /// with `WWW-Authenticate: Bearer`. The write surface is: a SPARQL UPDATE on `/sparql`
    /// — `Content-Type: application/sparql-update` OR an `update=` form field (BOTH are
    /// updates by SPARQL-Protocol definition, gated as writes UNCONDITIONALLY), and ALSO a
    /// `query=`/`application/sparql-query` body that *parses as an update* (an update smuggled
    /// through the query path — classification there keys on whether the request mutates, not
    /// the route) — and the Graph-Store-Protocol write methods (`PUT`/`POST`/`DELETE`/`PATCH`) on
    /// `/sparql/graph` and `/graphs/{*path}`. The token is compared in **constant time**
    /// ([`constant_time_eq`]). A missing vs a wrong token produce the *identical* 401, so an
    /// attacker cannot learn whether a token was presented. `None` (the default) means **no
    /// write auth** — today's behaviour, preserved exactly. Mirrors QLever's `-a <token>`.
    /// Enforced by the library `router` itself, so an embedder gets the gate for free.
    pub auth_token: Option<String>,
    /// [OPUS-4.8] sq-zcby: ALSO gate **reads** (SPARQL query, GSP `GET`/`HEAD`) with the same
    /// [`auth_token`](Self::auth_token). Off by default (QLever-style: writes gated, reads
    /// open). Has no effect unless `auth_token` is also set. When on, the whole surface is
    /// authenticated, which the bind posture ([`AuthPosture`]) treats as "auth present" for
    /// allowing a non-loopback bind without `--allow-remote`.
    ///
    /// [OPUS-4.8] sq-cxk5: the subscription READ surfaces — the `/subscriptions` WebSocket and
    /// the `/subscriptions/sse` Server-Sent-Events stream (both stream live SELECT diffs) — are
    /// gated by this flag too, closing the read-auth bypass that existed when they were always
    /// open. The SSE GET reads the `Authorization: Bearer` header like any other GET; the WS
    /// UPGRADE accepts the token from that header OR (for browsers, which cannot set headers on a
    /// WS handshake) a `Sec-WebSocket-Protocol: bearer.<token>` subprotocol — see
    /// [`crate::subscriptions::subscriptions_endpoint`] and [`ws_auth_gate`].
    pub auth_token_read: bool,
    /// [OPUS-4.8] (sq-4w18) SERVICE federation egress allowlist. SPARQL `SERVICE <iri>`
    /// makes attacker-controlled query text trigger an outbound HTTP request from the
    /// server host — an SSRF surface. The server's posture is **default-DENY-all
    /// SERVICE**: with the `service` cargo feature on but this allowlist EMPTY (the
    /// default), every SERVICE clause is refused before any network call. An operator
    /// opts hosts back in via `--service-allow` / `--service-allow-file` /
    /// `SPARQ_SERVICE_ALLOW` (see [`crate::ServiceAllowlist`]); only listed hosts (or
    /// `*.suffix` matches) become reachable — even a public host must be listed.
    ///
    /// Enforced by installing [`sparq_engine::with_service_egress_policy`] (strict =
    /// allowlist-only) around every engine call. Without the `service` cargo feature
    /// this field is still present (so the config shape is stable) but inert: no
    /// federation code is compiled and a SERVICE clause errors at execution as before.
    pub service_allow: crate::service_config::ServiceAllowlist,
    /// [OPUS-4.8] (sq-7cxr, gh-44) DURABLE PERSISTENCE directory — the QLever `--persist-updates`
    /// equivalent. When `Some(dir)`, the server treats the on-disk index at `dir` as the durable,
    /// rebuildable source of truth: at startup it opens the existing store there (replaying its
    /// write-ahead log so prior updates are present with **no rebuild**), or creates it from the
    /// seed graph; and every committed SPARQL Update is WAL-appended + fsync'd to `dir` (default
    /// graph and named graphs alike) BEFORE the group-commit ack, so a process restart preserves
    /// ALL updates. `None` (the default) is the historical purely in-memory server — updates are
    /// lost on restart. Set by the binary's `--persist <DIR>` flag / `SPARQ_PERSIST_DIR` env.
    pub persist_dir: Option<std::path::PathBuf>,
    /// [OPUS-4.8] (sq-d3d8, epic sq-3183) OPT-IN federation discovery descriptors. When
    /// `true`, the server serves a W3C VoID dataset description at `GET /.well-known/void`
    /// and a SPARQL 1.1 Service Description for a `GET /sparql` with no `query` parameter
    /// (advertising the endpoint, supported languages, result formats and the default
    /// dataset). `false` (the default) leaves both off: `/.well-known/void` is `404` and a
    /// `GET /sparql` with no `query` returns the historical `400 missing 'query'`.
    ///
    /// This field exists only with the `federation-descriptors` cargo feature (like
    /// [`time_travel_generations`](Self::time_travel_generations) under `time-travel`); a
    /// build without that feature compiles no descriptor code and pays zero cost. Set by the
    /// binary's `--federation-descriptors` flag / `SPARQ_FEDERATION_DESCRIPTORS=1` env.
    #[cfg(feature = "federation-descriptors")]
    pub federation_descriptors: bool,
    /// [OPUS-4.8] (sq-bzh1, epic sq-3183) OPT-IN Triple Pattern Fragments / Linked Data
    /// Fragments READ-ONLY source endpoint. When `true`, the server serves a paged RDF
    /// fragment of the triples matching one triple pattern at `GET /tpf?subject=&predicate=&
    /// object=` (with Hydra controls: `hydra:totalItems` from the cheap cardinality estimate,
    /// `hydra:next`/`hydra:previous` paging, the `hydra:search` template). `false` (the
    /// default) leaves it off: `/tpf` is `404`.
    ///
    /// This field exists only with the `tpf` cargo feature (like
    /// [`federation_descriptors`](Self::federation_descriptors) under
    /// `federation-descriptors`); a build without that feature compiles no TPF code and pays
    /// zero cost. Set by the binary's `--tpf` flag / `SPARQ_TPF=1` env.
    #[cfg(feature = "tpf")]
    pub tpf: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            query_timeout: Some(Duration::from_secs(30)),
            // [OPUS-4.8] sq-nulp: no separate writer-side WHERE deadline by default — an
            // update's WHERE budget is the plain query_timeout, exactly as before. An operator
            // who wants to bound writer-queue head-of-line blocking opts a shorter one in.
            update_where_timeout: None,
            max_body_bytes: 1024 * 1024, // 1 MiB
            max_concurrent: 32,
            max_results: None,
            // [OPUS-4.8] sq-ebii: memory cap OFF by default (no surprise refusals on an
            // unconfigured server); an operator exposing the endpoint opts a ceiling in.
            max_query_rows: None,
            // [OPUS-4.8] sq-s5is: byte-accounted cap likewise OFF by default; opt-in ceiling.
            max_query_bytes: None,
            // [OPUS-4.8] sq-ebii: 20× decompressed:compressed is a permissive-but-bounded
            // default (well above real RDF gzip ratios ~3–8×, far below a bomb's ~1000×+).
            max_decompress_ratio: 20,
            max_subscriptions_per_conn: 16,
            max_subscriptions: 256,
            #[cfg(feature = "time-travel")]
            time_travel_generations: 16,
            #[cfg(feature = "time-travel")]
            time_travel_max_age: None,
            verbose: false,
            // [OPUS-4.8] sq-toze.34: redact request content from the verbose log by DEFAULT — a
            // privacy-respecting default so enabling --verbose for debugging does not silently
            // start writing query text (possibly PII) into operator logs. Opt out with
            // --log-full-requests / SPARQ_LOG_FULL_REQUESTS=1. Inert unless `verbose` is also set.
            redact_logs: true,
            // [OPUS-4.8] sq-0bxp: audit log OFF by default even when the feature is compiled in
            // (the operator opts in deliberately via --audit-log / SPARQ_AUDIT_LOG=1).
            #[cfg(feature = "audit-log")]
            audit_log: false,
            // [OPUS-4.8] sq-gos8: no structured access-audit sink by default even when the
            // feature is compiled in (the operator opts in via --access-audit / SPARQ_ACCESS_AUDIT).
            #[cfg(feature = "access-audit")]
            access_audit: None,
            allow_remote: false, // [OPUS-4.8] sq-o4qf: safe default — refuse non-loopback bind unless opted in
            // [OPUS-4.8] sq-zcby: safe default — no token => no write auth (back-compat).
            auth_token: None,
            auth_token_read: false,
            // [OPUS-4.8] sq-4w18: safe default — empty allowlist = deny ALL SERVICE.
            service_allow: crate::service_config::ServiceAllowlist::default(),
            // [OPUS-4.8] sq-7cxr: safe default — no persistence dir = in-memory (back-compat).
            persist_dir: None,
            // [OPUS-4.8] sq-d3d8: safe default — federation discovery descriptors OFF even
            // when the feature is compiled in (the operator opts in deliberately).
            #[cfg(feature = "federation-descriptors")]
            federation_descriptors: false,
            // [OPUS-4.8] sq-bzh1: safe default — the TPF / LDF source endpoint is OFF even when
            // the feature is compiled in (the operator opts in deliberately via --tpf / SPARQ_TPF=1).
            #[cfg(feature = "tpf")]
            tpf: false,
        }
    }
}

impl ServerConfig {
    /// Defaults overridden by the `SPARQ_QUERY_TIMEOUT` (seconds; `0` disables),
    /// `SPARQ_UPDATE_WHERE_TIMEOUT` ([OPUS-4.8] sq-nulp: the separate, typically-shorter
    /// writer-side WHERE deadline that bounds writer-queue head-of-line blocking from a slow
    /// UPDATE; seconds, `0`/unset disables — the update WHERE budget is then the plain
    /// `query_timeout`), `SPARQ_MAX_BODY_BYTES`, `SPARQ_MAX_CONCURRENT`, `SPARQ_MAX_RESULTS`,
    /// `SPARQ_MAX_QUERY_ROWS` ([OPUS-4.8] sq-ebii: the coarse memory cap; `0` disables) and
    /// `SPARQ_MAX_DECOMPRESS_RATIO` ([OPUS-4.8] sq-ebii: the zip-bomb guard; `0` refuses gzip
    /// bodies) environment variables — plus, with the `time-travel` feature,
    /// `SPARQ_TIME_TRAVEL_GENERATIONS` and `SPARQ_TIME_TRAVEL_MAX_AGE` (seconds;
    /// `0` disables the age bound), `SPARQ_ALLOW_REMOTE` (non-loopback bind opt-in),
    /// and `SPARQ_SERVICE_ALLOW` (comma/whitespace-separated SERVICE egress allowlist;
    /// [OPUS-4.8] sq-4w18). CLI flags override / widen these in `main` (the allowlist is
    /// additive: `--service-allow` / `--service-allow-file` UNION with the env baseline).
    ///
    /// [OPUS-4.8] sq-4w18: returns `Err` (rather than panicking) on a malformed
    /// `SPARQ_SERVICE_ALLOW` entry. `from_env` is public config API an embedder may
    /// call, and a panic in a config constructor is a hostile surprise; a `Result`
    /// (the crate's `Result<_, String>` error style, e.g. `from_sources` / nlq's
    /// `from_env`) lets the caller surface a clean user-facing message. The valid
    /// path is byte-for-byte unchanged.
    pub fn from_env() -> Result<Self, String> {
        let mut cfg = Self::default();
        if let Some(secs) = env_parse::<u64>("SPARQ_QUERY_TIMEOUT") {
            cfg.query_timeout = (secs > 0).then(|| Duration::from_secs(secs));
        }
        // [OPUS-4.8] sq-nulp: separate, typically-shorter writer-side WHERE deadline that
        // bounds writer-queue head-of-line blocking from a slow UPDATE; 0 / unset disables it
        // (the update WHERE budget is then the plain query_timeout, exactly as before).
        if let Some(secs) = env_parse::<u64>("SPARQ_UPDATE_WHERE_TIMEOUT") {
            cfg.update_where_timeout = (secs > 0).then(|| Duration::from_secs(secs));
        }
        if let Some(n) = env_parse::<usize>("SPARQ_MAX_BODY_BYTES") {
            cfg.max_body_bytes = n;
        }
        if let Some(n) = env_parse::<usize>("SPARQ_MAX_CONCURRENT") {
            cfg.max_concurrent = n.max(1);
        }
        if let Some(n) = env_parse::<usize>("SPARQ_MAX_RESULTS") {
            cfg.max_results = (n > 0).then_some(n);
        }
        // [OPUS-4.8] sq-ebii: memory cap (coarse working-set row ceiling on every form);
        // 0 / unset disables it.
        if let Some(n) = env_parse::<usize>("SPARQ_MAX_QUERY_ROWS") {
            cfg.max_query_rows = (n > 0).then_some(n);
        }
        // [OPUS-4.8] sq-s5is: byte-accounted memory cap (prices row width + computed
        // literals, on every form); 0 / unset disables it.
        if let Some(n) = env_parse::<usize>("SPARQ_MAX_QUERY_BYTES") {
            cfg.max_query_bytes = (n > 0).then_some(n);
        }
        // [OPUS-4.8] sq-ebii: decompression-ratio cap (zip-bomb guard); 0 disables
        // ratio-capped decompression (a Content-Encoding body is then refused outright).
        if let Some(n) = env_parse::<usize>("SPARQ_MAX_DECOMPRESS_RATIO") {
            cfg.max_decompress_ratio = n;
        }
        if let Some(n) = env_parse::<usize>("SPARQ_MAX_SUBSCRIPTIONS_PER_CONN") {
            cfg.max_subscriptions_per_conn = n;
        }
        if let Some(n) = env_parse::<usize>("SPARQ_MAX_SUBSCRIPTIONS") {
            cfg.max_subscriptions = n;
        }
        #[cfg(feature = "time-travel")]
        {
            if let Some(n) = env_parse::<usize>("SPARQ_TIME_TRAVEL_GENERATIONS") {
                cfg.time_travel_generations = n;
            }
            if let Some(secs) = env_parse::<u64>("SPARQ_TIME_TRAVEL_MAX_AGE") {
                cfg.time_travel_max_age = (secs > 0).then(|| Duration::from_secs(secs));
            }
        }
        // [OPUS-4.8] sq-0bxp: SPARQ_AUDIT_LOG truthy ("1"/"true"/"yes"/"on") turns the per-query
        // access audit log on at runtime (only when the `audit-log` feature is compiled in).
        #[cfg(feature = "audit-log")]
        if let Ok(v) = std::env::var("SPARQ_AUDIT_LOG") {
            cfg.audit_log = env_truthy(&v);
        }
        // [OPUS-4.8] sq-toze.34: SPARQ_LOG_FULL_REQUESTS truthy ("1"/"true"/"yes"/"on") OPTS OUT of
        // request-log redaction (logs full URIs/query text verbatim). Default (unset / falsey) keeps
        // redaction ON — the privacy-respecting posture. Only meaningful together with --verbose.
        if let Ok(v) = std::env::var("SPARQ_LOG_FULL_REQUESTS") {
            cfg.redact_logs = !env_truthy(&v);
        }
        // [OPUS-4.8] sq-gos8: SPARQ_ACCESS_AUDIT=<file|stderr> installs the structured
        // access-audit sink (only when the `access-audit` feature is compiled in). An empty /
        // unset value leaves it off. The literal `stderr` selects the stderr sink; anything else
        // is a file path the default JSON-Lines sink appends to.
        #[cfg(feature = "access-audit")]
        if let Ok(v) = std::env::var("SPARQ_ACCESS_AUDIT") {
            let v = v.trim();
            if !v.is_empty() {
                cfg.access_audit = Some(crate::access_audit::SinkTarget::parse(v));
            }
        }
        // [OPUS-4.8] sq-o4qf: SPARQ_ALLOW_REMOTE truthy ("1"/"true"/"yes"/"on", case-insensitive)
        // opts in to a non-loopback bind. Anything else (incl. unset / "0" / "false") leaves the
        // safe default of refusing to expose the unauthenticated surface beyond loopback.
        if let Ok(v) = std::env::var("SPARQ_ALLOW_REMOTE") {
            cfg.allow_remote = env_truthy(&v);
        }
        // [OPUS-4.8] sq-zcby: SPARQ_AUTH_TOKEN sets the write-gate token (an empty value is
        // treated as "unset" — an empty shared secret is a footgun, never a valid token);
        // SPARQ_AUTH_TOKEN_READ truthy ("1"/"true"/"yes"/"on") additionally gates reads.
        if let Ok(v) = std::env::var("SPARQ_AUTH_TOKEN") {
            cfg.auth_token = (!v.is_empty()).then_some(v);
        }
        if let Ok(v) = std::env::var("SPARQ_AUTH_TOKEN_READ") {
            cfg.auth_token_read = env_truthy(&v);
        }
        // [OPUS-4.8] sq-4w18: SERVICE egress allowlist baseline from SPARQ_SERVICE_ALLOW
        // (comma/whitespace-separated). The binary then ADDS any `--service-allow` /
        // `--service-allow-file` entries (the union — CLI only ever widens). A malformed
        // env entry is a hard startup error (propagated, not panicked) rather than a
        // silently-dropped host, so the operator's allowlist is never quietly narrower
        // than written.
        if let Ok(v) = std::env::var("SPARQ_SERVICE_ALLOW") {
            cfg.service_allow
                .add_many(&v)
                .map_err(|e| format!("SPARQ_SERVICE_ALLOW: {e}"))?;
        }
        // [OPUS-4.8] sq-7cxr: SPARQ_PERSIST_DIR enables durable persistence at the given
        // directory (the binary's --persist flag overrides it). An empty value is "unset".
        if let Ok(v) = std::env::var("SPARQ_PERSIST_DIR") {
            cfg.persist_dir = (!v.is_empty()).then(|| std::path::PathBuf::from(v));
        }
        // [OPUS-4.8] sq-d3d8: SPARQ_FEDERATION_DESCRIPTORS truthy ("1"/"true"/"yes"/"on")
        // serves the VoID + Service-Description discovery endpoints. Off by default. Only
        // present with the `federation-descriptors` feature.
        #[cfg(feature = "federation-descriptors")]
        if let Ok(v) = std::env::var("SPARQ_FEDERATION_DESCRIPTORS") {
            cfg.federation_descriptors = env_truthy(&v);
        }
        // [OPUS-4.8] sq-bzh1: SPARQ_TPF truthy ("1"/"true"/"yes"/"on") serves the Triple Pattern
        // Fragments / LDF source endpoint. Off by default. Only present with the `tpf` feature.
        #[cfg(feature = "tpf")]
        if let Ok(v) = std::env::var("SPARQ_TPF") {
            cfg.tpf = env_truthy(&v);
        }
        Ok(cfg)
    }
}

/// [OPUS-4.8] sq-o4qf: parse a boolean-ish env value. Truthy: `1`, `true`, `yes`, `on`
/// (case-insensitive, trimmed); everything else (including empty) is false.
fn env_truthy(v: &str) -> bool {
    matches!(
        v.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-o4qf / sq-zcby — bind-time security posture (auth-aware)
// ---------------------------------------------------------------------------

/// [OPUS-4.8] sq-o4qf: the decision the binary makes about a requested bind address,
/// given the configured auth posture. Returned by [`bind_posture`] so `main` (and tests)
/// can act on it without the side effect of actually binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindPosture {
    /// Loopback (or otherwise not-remotely-reachable) bind — safe, proceed silently.
    Loopback,
    /// Non-loopback bind that is allowed to proceed (opted in via `--allow-remote` /
    /// `SPARQ_ALLOW_REMOTE`, or the whole surface is authenticated — sq-zcby). Proceed, but the
    /// caller MUST surface `warning` (it describes exactly what is now reachable: a fully open
    /// surface, an open read endpoint behind a write gate, or a fully authenticated surface).
    RemoteAllowed { warning: String },
    /// Non-loopback bind that is refused. The caller MUST refuse to bind and print `message`
    /// (which explains the exposure and how to proceed: gate the surface or opt in).
    RemoteRefused { message: String },
}

/// [OPUS-4.8] sq-o4qf / sq-zcby: classify a requested bind address under the configured auth
/// posture.
///
/// By default `sparq-server` has no authentication on any endpoint (query,
/// `application/sparql-update`, the `/subscriptions` WebSocket). A loopback bind
/// (`127.0.0.0/8`, `::1`) is only reachable from the same host, so it is safe by default. A
/// **non-loopback** bind exposes the surface to the network, so the binary refuses it unless
/// the operator opts in via `--allow-remote` / `SPARQ_ALLOW_REMOTE=1` (and even then warns) OR
/// the whole surface is authenticated ([`AuthPosture::ReadAndWrite`]). A write-token alone
/// ([`AuthPosture::WriteOnly`]) leaves reads open, so it is treated like no auth for this
/// decision: still refused without `--allow-remote`.
///
/// "Loopback" here means the literal loopback ranges. Note `0.0.0.0` / `::` (the unspecified
/// "bind to all interfaces" addresses) are NOT loopback — they are the most common way the
/// surface gets exposed, so they are treated as remote. This is a deliberately blunt,
/// fail-closed check: it errs toward refusing exposure, not toward allowing it.
///
/// [OPUS-4.8] sq-zcby: `auth` folds the configured Bearer-token gate into the decision. A
/// non-loopback bind is allowed when `--allow-remote` is set OR the **whole surface** is
/// authenticated ([`AuthPosture::ReadAndWrite`]) — a write-token alone ([`AuthPosture::
/// WriteOnly`]) still requires `--allow-remote`, because it leaves an OPEN read endpoint on
/// the remote bind; we still warn in that case that reads remain open.
pub fn bind_posture(addr: &SocketAddr, allow_remote: bool, auth: AuthPosture) -> BindPosture {
    if addr.ip().is_loopback() {
        return BindPosture::Loopback;
    }
    // A fully-authenticated surface (token gates reads AND writes) is safe to expose without
    // --allow-remote: there is no open endpoint left. We still warn (a single shared secret
    // is not per-user authz, and the token must be carried over TLS).
    if auth == AuthPosture::ReadAndWrite {
        return BindPosture::RemoteAllowed {
            warning: format!(
                "WARNING: sparq-server is binding the non-loopback address {addr}. The whole \
                 surface is gated by the --auth-token Bearer token (reads AND writes), so it \
                 is not open to anonymous access. NOTE: the token is a single shared secret \
                 (not per-user authz) — deliver it over TLS (terminate at a proxy), and front \
                 it with a real authorization layer (a reverse proxy / gateway or sparq-solid) \
                 for per-user access control. [OPUS-4.8] sq-cxk5: the /subscriptions WebSocket \
                 AND /subscriptions/sse stream (read surfaces) are gated by --auth-token-read \
                 too — browser WS clients pass the token as a 'Sec-WebSocket-Protocol: \
                 bearer.<token>' subprotocol."
            ),
        };
    }
    if allow_remote {
        // The surface is reachable from the network; describe exactly what is open.
        let exposure = match auth {
            AuthPosture::None => {
                "The full dataset is exposed for READ AND WRITE (SPARQL Update + the \
                 /subscriptions WebSocket) to anyone who can reach this port — there is NO \
                 authentication."
            }
            // WriteOnly: writes are gated, but reads are still open on this remote bind.
            AuthPosture::WriteOnly => {
                "Writes are gated by --auth-token, but READS remain OPEN to anyone who can \
                 reach this port (add --auth-token-read to gate reads too). The /subscriptions \
                 WebSocket (a read surface) is also open."
            }
            AuthPosture::ReadAndWrite => unreachable!("handled above"),
        };
        BindPosture::RemoteAllowed {
            warning: format!(
                "WARNING: sparq-server is binding the non-loopback address {addr}. {exposure} \
                 Put it behind a reverse proxy / API gateway (or sparq-solid) that enforces \
                 auth before exposing it to an untrusted network. \
                 (--allow-remote / SPARQ_ALLOW_REMOTE is set, so this bind proceeds.)"
            ),
        }
    } else {
        // Refused. A write-token alone is NOT sufficient (reads stay open), so we name the
        // two ways to proceed: gate reads too (--auth-token-read) or opt in (--allow-remote).
        let reason = match auth {
            AuthPosture::None => {
                "sparq-server has NO authentication, so this would expose the full dataset for \
                 READ AND WRITE to the network"
            }
            AuthPosture::WriteOnly => {
                "--auth-token gates writes but READS would still be OPEN on a non-loopback bind \
                 (add --auth-token-read to gate reads too, which makes the whole surface \
                 authenticated and the bind safe)"
            }
            AuthPosture::ReadAndWrite => unreachable!("handled above"),
        };
        BindPosture::RemoteRefused {
            message: format!(
                "refusing to bind non-loopback address {addr}: {reason}. If a network bind is \
                 intended, either gate the whole surface (--auth-token AND --auth-token-read) \
                 or run behind a reverse proxy / gateway that enforces auth and re-run with \
                 --allow-remote (or SPARQ_ALLOW_REMOTE=1). To serve only this host, bind a \
                 loopback address such as 127.0.0.1."
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-zcby (PSS gh-46) — the Bearer-token auth gate (write surface +
// optional read gate), mirroring QLever's `-a <token>`.
// ---------------------------------------------------------------------------

/// [OPUS-4.8] sq-zcby: how much of the surface a configured token authenticates — folded into
/// the bind decision ([`bind_posture`]). `WriteOnly` (a token, reads open) still requires
/// `--allow-remote` for a non-loopback bind because reads stay open; `ReadAndWrite` (token +
/// `--auth-token-read`) makes the whole surface authenticated, so it does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthPosture {
    /// No token configured — no auth on any endpoint.
    None,
    /// A token gates writes; reads are open.
    WriteOnly,
    /// A token gates writes AND reads (`--auth-token-read`).
    ReadAndWrite,
}

impl AuthPosture {
    /// Derives the posture from a [`ServerConfig`]: no token → `None`; a token without the
    /// read gate → `WriteOnly`; a token with `--auth-token-read` → `ReadAndWrite`.
    pub fn from_config(config: &ServerConfig) -> Self {
        match (config.auth_token.is_some(), config.auth_token_read) {
            (false, _) => AuthPosture::None,
            (true, false) => AuthPosture::WriteOnly,
            (true, true) => AuthPosture::ReadAndWrite,
        }
    }
}

/// [OPUS-4.8] sq-zcby: whether a request is a WRITE (mutates the dataset) or a READ, for the
/// auth gate. Classification keys on "does this mutate", NOT the route — an UPDATE smuggled
/// through the query path is a write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Operation {
    Read,
    Write,
}

/// [OPUS-4.8] sq-zcby: constant-time byte-string equality. Returns `true` iff `a == b`,
/// taking time that depends only on `a.len()` (not on the contents or on how far the first
/// difference is), so a token check cannot be turned into a timing oracle that recovers the
/// secret byte-by-byte.
///
/// Hand-rolled rather than pulling in the `subtle` crate: it is a few lines, sparq-server
/// has no other crypto dependency (keeping it out of the supply-chain / SBOM surface), and a
/// length-difference + per-byte XOR-accumulate is the standard, well-understood construction.
/// `a` is the configured secret and `b` the presented token; the length comparison reveals
/// only whether the *presented* token has the secret's length (already inferable, and not the
/// secret's bytes). The accumulator is `#[inline(never)]` and read into a `black_box`-style
/// volatile-ish fold to keep the optimiser from short-circuiting (no data-dependent branch).
#[inline(never)]
pub(crate) fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    // [OPUS-4.8] sq-zcby (Copilot PR#71 fix): route the accumulator through
    // `core::hint::black_box` BEFORE the `== 0` test so the optimiser cannot prove anything
    // about `diff` and rewrite the loop+compare into an early-exit `memcmp`-style
    // short-circuit. This matches the doc claim ("black_box-style fold"): without it the
    // bare `diff == 0` was a plain data-dependent comparison the compiler is free to
    // short-circuit. Fold to bool with no data-dependent branch on the secret.
    core::hint::black_box(diff) == 0
}

/// [OPUS-4.8] sq-zcby: extracts the token from an `Authorization: Bearer <token>` header,
/// tolerant of scheme casing (`Bearer`/`bearer`/`BEARER`) and of leading/trailing space. Any
/// other scheme (or no header) yields `None`.
fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let raw = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = raw.split_once(' ')?;
    if scheme.eq_ignore_ascii_case("bearer") {
        Some(token.trim())
    } else {
        None
    }
}

/// [OPUS-4.8] sq-cxk5: the WebSocket subprotocol-token prefix. A browser cannot set an
/// `Authorization` header on a WebSocket handshake, so it carries the Bearer token as a
/// `Sec-WebSocket-Protocol: bearer.<token>` subprotocol instead. See [`subprotocol_bearer_token`]
/// and `crates/sparq-server/README.md` → "Authenticating a WebSocket subscription from a browser".
pub(crate) const WS_BEARER_SUBPROTOCOL_PREFIX: &str = "bearer.";

/// [OPUS-4.8] sq-cxk5: extracts a Bearer token offered as a `Sec-WebSocket-Protocol`
/// subprotocol of the form `bearer.<token>` (the browser-compatible channel — browsers CANNOT
/// set an `Authorization` header on a WS handshake, but `new WebSocket(url, [proto])` sets
/// `Sec-WebSocket-Protocol`). The header may list several comma-separated subprotocols; the FIRST
/// entry whose value starts with `bearer.` wins, and the substring AFTER the prefix is the token
/// (no trimming of the token body — a subprotocol token is exact). Returns the FULL matched
/// subprotocol string too (caller echoes it back per RFC 6455). `None` when no offered subprotocol
/// carries the prefix.
pub(crate) fn subprotocol_bearer_token(headers: &HeaderMap) -> Option<(&str, &str)> {
    // A client may send multiple `Sec-WebSocket-Protocol` header lines, each itself a
    // comma-separated list; scan every entry across all lines.
    headers
        .get_all(header::SEC_WEBSOCKET_PROTOCOL)
        .iter()
        .find_map(|v| {
            let raw = v.to_str().ok()?;
            raw.split(',').map(str::trim).find_map(|proto| {
                proto
                    .strip_prefix(WS_BEARER_SUBPROTOCOL_PREFIX)
                    .map(|tok| (proto, tok))
            })
        })
}

/// [OPUS-4.8] sq-zcby / sq-cxk5: the per-request auth decision core. Returns `None` to proceed,
/// or `Some(401)` to refuse. A request is gated when its [`Operation`] is covered by the
/// configured posture: a `Write` whenever a token is set; a `Read` only additionally when
/// `--auth-token-read` is on. An ungated operation (or no token configured at all) always
/// proceeds. A gated request must present the exact token (constant-time compared); `presented`
/// is the candidate token already extracted from whichever channel the transport allows (the
/// `Authorization: Bearer` header on plain HTTP, or ALSO the `bearer.<token>` WebSocket
/// subprotocol — sq-cxk5). The 401 is byte-identical for a missing vs a wrong token, so it never
/// leaks which.
pub(crate) fn auth_check(
    config: &ServerConfig,
    op: Operation,
    presented: Option<&str>,
) -> Option<Response> {
    let token = config.auth_token.as_deref()?; // no token configured => never gated
    let gated = match op {
        Operation::Write => true,
        Operation::Read => config.auth_token_read,
    };
    if !gated {
        return None;
    }
    let ok = presented.is_some_and(|p| constant_time_eq(token.as_bytes(), p.as_bytes()));
    if ok {
        None
    } else {
        Some(unauthorized())
    }
}

/// [OPUS-4.8] sq-zcby: the per-request auth gate for the plain-HTTP surface — validates the
/// `Authorization: Bearer <token>` header against the configured posture. See [`auth_check`].
/// The SSE subscription GET (`/subscriptions/sse`) uses this exactly like the other GET routes
/// (sq-cxk5): it is a plain GET, so the Bearer header is the only channel.
pub(crate) fn auth_gate(
    config: &ServerConfig,
    headers: &HeaderMap,
    op: Operation,
) -> Option<Response> {
    auth_check(config, op, bearer_token(headers))
}

/// [OPUS-4.8] sq-cxk5: the auth gate for the `/subscriptions` WebSocket UPGRADE. A browser cannot
/// set an `Authorization` header on a WS handshake, so this accepts the token from EITHER channel:
/// the `Authorization: Bearer <token>` header (non-browser clients) OR a `Sec-WebSocket-Protocol:
/// bearer.<token>` subprotocol (browsers). The token is validated against the read token
/// (constant-time, [`auth_check`] with [`Operation::Read`]) — a subprotocol token is VALIDATED,
/// never merely echoed. When `--auth-token-read` is not set (or no token is configured) the
/// upgrade is unchanged (open) — back-compatible. The `Authorization` header is preferred when
/// present; the subprotocol is the fallback so a browser is not penalised for offering an
/// unrelated subprotocol alongside `bearer.<token>`.
pub(crate) fn ws_auth_gate(
    config: &ServerConfig,
    headers: &HeaderMap,
    op: Operation,
) -> Option<Response> {
    let presented =
        bearer_token(headers).or_else(|| subprotocol_bearer_token(headers).map(|(_, tok)| tok));
    auth_check(config, op, presented)
}

/// [OPUS-4.8] sq-zcby: the 401 a gated request without a valid token gets — `WWW-Authenticate:
/// Bearer` plus the server's standard JSON error body. Identical for a missing vs a wrong
/// token (the body carries no hint either way), so it never leaks which.
///
/// [OPUS-4.8] sq-2bhm (ASVS-G1): this auth-refusal is a SENSITIVE response, so — unlike the
/// general surface, where a blanket `Cache-Control` is deliberately NOT forced (results are
/// uncached by default; see [`security_headers`]) — it carries `Cache-Control: no-store` so a
/// shared cache / proxy never retains the 401 (or any future body it grows). This is the
/// narrow, targeted use of `no-store` the security-header gap calls for, scoped to the auth
/// path rather than imposed globally.
pub(crate) fn unauthorized() -> Response {
    let mut resp = json_error(
        StatusCode::UNAUTHORIZED,
        "authentication required: present a valid Bearer token",
    );
    let headers = resp.headers_mut();
    headers.insert(
        header::WWW_AUTHENTICATE,
        header::HeaderValue::from_static("Bearer"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    resp
}

/// [OPUS-4.8] sq-zcby: does this SPARQL string MUTATE the dataset, for the auth classifier?
/// The classifier keys on this, not on the route or Content-Type, so an UPDATE smuggled
/// through a `query=`/generic body path is still gated as a write.
///
/// [OPUS-4.8] (Copilot PR#71 doc/impl-consistency fix) The implemented rule is
/// **default-to-write unless it provably parses as a read-only query**: a string that parses
/// as a read-only query (SELECT/ASK/CONSTRUCT/DESCRIBE) returns `false` (a read); EVERYTHING
/// ELSE returns `true` (gated as a write). That "everything else" deliberately collapses the
/// "parses as a SPARQL Update" and the "parses as NEITHER" cases into one branch — both are
/// fail-safe to gate as a write, so a positive `parse_update` check would only add cost
/// without changing the answer. This is the conservative/secure default: ambiguous or
/// malformed bodies are gated as writes (fail-closed), so a body can never slip past the
/// write gate by being unparsable, and gating a non-mutating-but-unparsable body as a write
/// never wrongly OPENS the write surface (the writer/query handler rejects it anyway).
///
/// Note this function is only the classifier for the AMBIGUOUS body path; the unambiguous
/// `update=` form field and `application/sparql-update` Content-Type are gated as writes
/// unconditionally at their call sites (they ARE updates by protocol definition), without
/// consulting this function.
pub(crate) fn payload_mutates(sparql: &str) -> bool {
    // Provably a read-only query => a read (`is_ok`). Otherwise (parses as an update, OR
    // parses as neither) => fail-safe to a write (`is_err`). The two non-read cases share one
    // branch on purpose: both must be gated as a write, so distinguishing them changes nothing.
    spargebra::SparqlParser::new().parse_query(sparql).is_err()
}

/// [OPUS-4.8] (sq-gos8) Maps the auth-classifier [`Operation`] to the structured-audit
/// [`Action`](crate::access_audit::Action) for the plain SPARQL surface (read => a query,
/// write => an update).
#[cfg(feature = "access-audit")]
fn sparql_action(op: Operation) -> crate::access_audit::Action {
    match op {
        Operation::Read => crate::access_audit::Action::Query,
        Operation::Write => crate::access_audit::Action::Update,
    }
}

/// [OPUS-4.8] (sq-gos8) Begins a structured access-audit record IFF a sink is installed —
/// snapshotting the actor (from the presented Bearer token), the action class, the resource and
/// the (non-reversible) request fingerprint at the enforcement seam. Returns `None` (and builds
/// nothing) when no sink is configured, so an audit-disabled request pays only the `Option`
/// check. The returned [`AuditPending`] is `finish`ed once the enforced decision is known and
/// handed to the sink via [`audit_access_finish`].
#[cfg(feature = "access-audit")]
fn audit_access_begin(
    state: &AppState,
    action: crate::access_audit::Action,
    resource: crate::access_audit::Resource,
    headers: &HeaderMap,
    sparql: Option<&str>,
) -> Option<crate::access_audit::AuditPending> {
    state.access_audit_sink().map(|_| {
        crate::access_audit::AuditPending::begin(
            action,
            crate::access_audit::Actor::from_token(bearer_token(headers)),
            resource,
            sparql,
        )
    })
}

/// [OPUS-4.8] (sq-gos8) Finishes a pending record and records it through the installed sink,
/// deriving the ACTUALLY-enforced decision from the finished [`Response`]: a `401` is the auth
/// gate's denial (`Deny` + the bearer-auth policy basis), anything else is the allowed-and-served
/// outcome. So the recorded decision is the one the server enforced, never a claimed-but-
/// disconnected one. A no-op when `pending` is `None` (no sink).
#[cfg(feature = "access-audit")]
fn audit_access_finish(
    state: &AppState,
    pending: Option<crate::access_audit::AuditPending>,
    resp: &Response,
) {
    use crate::access_audit::AccessDecision;
    let (Some(pending), Some(sink)) = (pending, state.access_audit_sink()) else {
        return;
    };
    let status = resp.status();
    let (decision, basis) = if status == StatusCode::UNAUTHORIZED {
        (
            AccessDecision::Deny,
            "bearer-auth: missing or invalid token",
        )
    } else {
        (AccessDecision::Allow, "bearer-auth: allowed")
    };
    let event = pending.finish(decision, basis, status.as_u16());
    sink.record(&event);
}

fn env_parse<T: std::str::FromStr>(name: &str) -> Option<T> {
    std::env::var(name).ok()?.trim().parse().ok()
}

/// Extra wall-clock allowance past the cooperative deadline before the server gives up
/// awaiting the worker and answers 503 anyway (the detached worker still stops at its
/// next budget check — it is not leaked indefinitely).
pub(crate) const TIMEOUT_GRACE: Duration = Duration::from_secs(2);

/// A request's pinned generation: holding this `Arc` keeps the generation's immutable
/// snapshot alive no matter how far the writer publishes past it. Pinned ONCE per
/// request and kept for the lifetime of response production (streamed bodies included —
/// see [`chunked_response`]), so every response is snapshot-consistent with its start.
pub type PinnedGen = Arc<Generation<Graph>>;

/// The catch-all pod: the visibility scope every query implicitly reads, and the
/// honest conflict unit for any write that cannot be scoped to a finite set of named
/// graphs (§6.3/§6.5).
///
/// [OPUS-4.8] (sq-uqh, Wave B) Bumping this pod's epoch means "invalidate everything":
/// it represents the DEFAULT graph (which the GSP surface maps every direct graph onto)
/// plus any cross-graph / dynamically-scoped write whose touched named graphs are not
/// statically knowable from the parsed update. A correct cache MUST therefore record
/// this pod's epoch on every entry, so a global bump invalidates all cached reads. Writes
/// that DO name a finite set of graphs additionally bump those graphs' per-named-graph
/// pods (see [`touched_pods`]), so a cache keyed on finer-than-global pods is invalidated
/// too — finer scoping is purely additive over this catch-all, never a replacement for it.
///
/// Public so a cache layer (and the update tests) can record/compare this catch-all pod's
/// epoch on every entry — the contract that makes a global bump invalidate everything.
pub const GLOBAL_POD: &str = "urn:sparq:pod:global";

/// [OPUS-4.8] (sq-uqh, Wave B) The visibility scope (`PodId` set) a SPARQL Update writes,
/// for per-named-graph cache invalidation (§6.3/§6.5).
///
/// Each pod is a named graph (its graph IRI); a write to graph A bumps only graph A's
/// epoch, so a cached read scoped to graph B is untouched. Correctness is paramount —
/// a missed bump is a stale read — so this OVER-invalidates whenever it cannot prove a
/// finer scope:
///
///   * INSERT/DELETE DATA, LOAD, CREATE, CLEAR/DROP GRAPH `<g>`: scoped to the concrete
///     named graph(s) they name. A *default-graph* quad / target / LOAD destination falls
///     back to [`GLOBAL_POD`] (the default graph is the catch-all pod).
///   * CLEAR/DROP DEFAULT/NAMED/ALL: cross-graph and unbounded → [`GLOBAL_POD`].
///   * DELETE/INSERT … WHERE: scoped to the concrete graph slots its delete/insert
///     TEMPLATES write — but the moment any template targets a *variable* graph name
///     (`GRAPH ?g { … }`), the written graphs are not statically knowable → [`GLOBAL_POD`].
///     The WHERE/USING READ scope is irrelevant: invalidation tracks what a write MODIFIES,
///     not what it reads.
///
/// A parse failure also returns [`GLOBAL_POD`] — the writer re-parses and rejects the
/// update, so nothing is actually published, but tagging conservatively keeps this
/// function total and never under-invalidates if the two parses ever disagree.
///
/// Concrete named pods are ALWAYS accumulated even when the global flag is also set, so
/// the returned set is correct whether a cache entry records the global pod, the specific
/// pods, or both. Over-invalidation (a redundant epoch bump) only costs a cache miss;
/// under-invalidation costs a stale read — so when in doubt, this bumps more.
fn touched_pods(sparql: &str) -> Vec<PodId> {
    use spargebra::algebra::GraphTarget;
    use spargebra::term::{GraphName, GraphNamePattern};
    use spargebra::GraphUpdateOperation;

    let mut acc = TouchedPods::default();
    let upd = match spargebra::SparqlParser::new().parse_update(sparql) {
        Ok(upd) => upd,
        // Unparsable here = the writer will also reject it (nothing published); tag global
        // so we are total and never under-invalidate on a parser disagreement.
        Err(_) => return vec![PodId::new(GLOBAL_POD)],
    };

    // A graph slot a write targets: a named graph is its own pod; the default graph is
    // the catch-all (global).
    let touch_graph = |g: &GraphName, acc: &mut TouchedPods| match g {
        GraphName::NamedNode(n) => acc.named(n.as_str()),
        GraphName::DefaultGraph => acc.global(),
    };

    for op in &upd.operations {
        match op {
            GraphUpdateOperation::InsertData { data } => {
                for q in data {
                    touch_graph(&q.graph_name, &mut acc);
                }
            }
            GraphUpdateOperation::DeleteData { data } => {
                for q in data {
                    touch_graph(&q.graph_name, &mut acc);
                }
            }
            GraphUpdateOperation::Load { destination, .. } => touch_graph(destination, &mut acc),
            // CLEAR / DROP: a single named graph is scopable; DEFAULT / NAMED / ALL are
            // cross-graph or unbounded → global.
            GraphUpdateOperation::Clear { graph: target, .. }
            | GraphUpdateOperation::Drop { graph: target, .. } => match target {
                GraphTarget::NamedNode(n) => acc.named(n.as_str()),
                GraphTarget::DefaultGraph | GraphTarget::NamedGraphs | GraphTarget::AllGraphs => {
                    acc.global()
                }
            },
            // CREATE makes one empty named graph — touches only it.
            GraphUpdateOperation::Create { graph, .. } => acc.named(graph.as_str()),
            // DELETE/INSERT … WHERE writes the template graph slots. Concrete slots are
            // scopable; a variable graph name (or a default-graph slot) is not, so it
            // widens to global. Reads (WHERE / USING) do not affect invalidation scope.
            GraphUpdateOperation::DeleteInsert { delete, insert, .. } => {
                for slot in delete
                    .iter()
                    .map(|q| &q.graph_name)
                    .chain(insert.iter().map(|q| &q.graph_name))
                {
                    match slot {
                        GraphNamePattern::NamedNode(n) => acc.named(n.as_str()),
                        // Default graph or a dynamically-bound graph name: unscopable → global.
                        GraphNamePattern::DefaultGraph | GraphNamePattern::Variable(_) => {
                            acc.global()
                        }
                    }
                }
            }
        }
    }

    acc.into_pods()
}

/// [OPUS-4.8] (sq-uqh) Accumulator for [`touched_pods`]: the set of concrete named-graph
/// pods a write touches, plus a flag for any unscopable (global) effect. The two are kept
/// independent — concrete pods are recorded even alongside a global effect — so the final
/// set never under-invalidates regardless of how a cache entry records its scope.
#[derive(Default)]
struct TouchedPods {
    named: std::collections::BTreeSet<String>,
    global: bool,
}

impl TouchedPods {
    /// Records a write to a concrete named graph.
    fn named(&mut self, iri: &str) {
        self.named.insert(iri.to_string());
    }

    /// Records an unscopable / cross-graph / default-graph write (the catch-all pod).
    fn global(&mut self) {
        self.global = true;
    }

    /// Materialises the pod set. Always non-empty so the publish records *some* epoch
    /// bump: an update that somehow named no graph at all (e.g. an empty operation list)
    /// still conservatively bumps global.
    fn into_pods(self) -> Vec<PodId> {
        let mut pods: Vec<PodId> = self.named.into_iter().map(PodId::new).collect();
        if self.global || pods.is_empty() {
            pods.push(PodId::new(GLOBAL_POD));
        }
        pods
    }
}

/// The sequenced writer's snapshot-production strategy: sparq-serve's [`GraphApplier`]
/// (per-batch O(graph) fork + O(batch) `update_in_place` — see its module docs for the
/// recorded cost decision), with every engine call wrapped in [`with_engine_scope`] so the
/// opt-in `geo` registry AND the SERVICE egress allowlist (sq-4w18) apply to updates
/// exactly as they do to queries — an `INSERT … WHERE { SERVICE <iri> { … } }` update
/// federates under the same default-deny allowlist as a read. The trait is sparq-serve's
/// documented seam for exactly this wrapper. [OPUS-4.8]
/// [OPUS-4.8] (sq-7cxr, gh-44) Resolves the durable directory-backed [`Graph`] for
/// `--persist <DIR>`: OPEN an existing on-disk store (replaying its WAL — prior updates
/// present with NO rebuild — and ignoring `seed`), or CREATE a fresh store by saving `seed`
/// there and re-opening it so it carries a WAL.
///
/// "Existing store" is keyed on the dictionary file the engine's [`Graph::open`] requires
/// (`dict-meta.bin`, or the legacy `dict.bin`). A directory that exists but is empty (or holds
/// only an unrelated file) is treated as fresh — we create the store from `seed`. A directory
/// that holds a store but fails to open (corruption) surfaces the open error rather than
/// silently clobbering it.
fn open_or_create_durable(dir: &std::path::Path, seed: Graph) -> Result<Graph, String> {
    let has_store = dir.join("dict-meta.bin").exists() || dir.join("dict.bin").exists();
    if has_store {
        // Existing durable store IS the source of truth: open + WAL-replay, no rebuild.
        return Graph::open(dir).map_err(|e| format!("opening persist dir {}: {e}", dir.display()));
    }
    // Fresh: persist the seed, then open it so the returned graph is directory-backed
    // (carries its own WAL) and ready for WAL-durable updates.
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("creating persist dir {}: {e}", dir.display()))?;
    seed.save(dir)
        .map_err(|e| format!("initialising persist dir {}: {e}", dir.display()))?;
    Graph::open(dir).map_err(|e| {
        format!(
            "opening freshly-initialised persist dir {}: {e}",
            dir.display()
        )
    })
}

/// [OPUS-4.8] (sq-vpx4) Internal marker prefixed onto the error string of a
/// [`WriteError::Unavailable`] (durable-write refusal) as it crosses from
/// [`AppState::apply_update`] to the HTTP response mapper, so
/// [`update_rejection_response`] can route it to HTTP 503 (retryable) rather than the
/// default 400 (client error). Never leaves the process — stripped before the message
/// reaches the client.
const DURABLE_UNAVAILABLE_PREFIX: &str = "\u{1}durable-unavailable\u{1}";

struct ServerApplier {
    inner: GraphApplier,
    /// The config the writer thread enforces around every engine call (carries the
    /// SERVICE egress allowlist; the writer has no per-request config, so it owns its own).
    config: Arc<ServerConfig>,
    /// [OPUS-4.8] (sq-7cxr, gh-44) The optional DURABLE mirror of the published lineage.
    /// `Some` exactly when `--persist <DIR>` is set: a directory-backed [`Graph`] (its own
    /// WAL + named-graph persistence) that the writer thread keeps in lockstep with the ring
    /// by re-applying each committed batch to it, WAL-durably, BEFORE the generation is
    /// published (and hence before the client ack). `None` is today's purely in-memory server.
    durable: Option<DurableStore>,
}

/// [OPUS-4.8] (sq-7cxr, gh-44) The writer-thread-owned durable store: the on-disk [`Graph`]
/// plus the buffer of RESOLVED update effects accumulated for the batch currently being committed.
///
/// The writer drives `fork → apply* → seal` per batch (and re-`fork`s + replays the
/// already-successful prefix on a mid-batch failure). We mirror that exactly: [`fork`] clears
/// `batch` (a fresh attempt starts), [`apply`] appends the effects of an update only AFTER it
/// applied cleanly to the in-memory working copy, and [`seal`] flushes `batch` to `graph`
/// WAL-durably. So at seal time `batch` is precisely the resolved delta of the updates that will
/// be published — applied once, in order, to the durable graph. Because all three run on the
/// single writer thread, no lock is needed and the durable graph never diverges from the
/// published lineage.
///
/// [OPUS-4.8] (Copilot PR#80) We buffer the *resolved* [`sparq_engine::UpdateEffect`] log
/// captured during the in-memory application — NOT the update text. Re-executing the text against
/// the durable graph would re-roll non-deterministic functions (`NOW()`/`RAND()`/`UUID()`/fresh
/// `BNODE()`) and could re-fetch different `LOAD <remote>` content, so the durable state could
/// diverge from the already-acked in-memory state. Replaying the captured delta makes the durable
/// phase deterministic by construction — the durable graph receives the identical resolved triples.
struct DurableStore {
    graph: Graph,
    batch: Vec<sparq_engine::UpdateEffect>,
    /// [OPUS-4.8] (sq-vpx4) TEST SEAM for injecting durable-write I/O failures into the
    /// seal path. In production this is `None` and the real [`sparq_engine::apply_effects`]
    /// runs. A test can install a hook that fails the durable commit (e.g. once with a
    /// simulated `ENOSPC`, then clears) WITHOUT touching the production code path — the
    /// hook decides per-call whether to fail. Boxed `FnMut` so it can hold per-call state
    /// (a "fail the next N seals" counter). Production behaviour is unchanged: `None` ⇒
    /// exactly the prior `apply_effects` call.
    #[cfg(any(test, feature = "test-seams"))]
    fail_seal: Option<Box<dyn FnMut() -> Option<String> + Send>>,
}

impl DurableStore {
    fn new(graph: Graph) -> Self {
        DurableStore {
            graph,
            batch: Vec::new(),
            #[cfg(any(test, feature = "test-seams"))]
            fail_seal: None,
        }
    }

    /// [OPUS-4.8] (sq-vpx4) Commit the buffered batch durably. Returns `Err` on a
    /// durable-write failure (real I/O error, or an injected one via the test seam) —
    /// the caller ([`ServerApplier::seal`]) propagates this WITHOUT publishing or
    /// acking, so a write that didn't durably commit is never observed.
    fn commit_batch(&mut self) -> Result<(), String> {
        let batch = std::mem::take(&mut self.batch);
        #[cfg(any(test, feature = "test-seams"))]
        if let Some(hook) = self.fail_seal.as_mut() {
            if let Some(e) = hook() {
                return Err(e);
            }
        }
        sparq_engine::apply_effects(&mut self.graph, &batch)
    }

    /// [OPUS-4.8] (sq-x32t) WAL COMPACTION / VACUUM for erasure-completeness. Rewrites the
    /// durable on-disk store to a fresh segment set containing ONLY the current LIVE triples,
    /// so superseded INSERTs, logically-DELETEd data and DROPped graphs that still linger in
    /// earlier WAL segments are PHYSICALLY removed from the on-disk history (the manual purge
    /// in `compliance/privacy/retention-erasure-runbook.md` §7a, automated). [`Graph::vacuum`]
    /// re-interns the live triples into a fresh dictionary (so orphaned term VALUES are purged
    /// too), writes the new base to a sibling dir, then does a rollback-safe two-rename swap
    /// (parent dir fsync'd between renames) and truncates the WAL; an interrupted swap is healed
    /// deterministically by `recover_compaction` on the next `Graph::open`. The live triple set
    /// is preserved EXACTLY (round-trip), so the published in-memory snapshot is unaffected —
    /// only the durable image is rewritten.
    ///
    /// Runs on the writer thread (via [`ServerApplier::maintain`]), so the durable graph is
    /// never touched concurrently with a commit. A pending in-memory `batch` is impossible
    /// here: maintenance is sequenced AFTER the preceding batch is sealed (see `Writer::run`),
    /// so `commit_batch` has already drained it. We assert that to fail loudly if the invariant
    /// is ever broken rather than silently dropping buffered effects.
    fn compact(&mut self) -> Result<(), String> {
        debug_assert!(
            self.batch.is_empty(),
            "compaction must run between batches (the durable batch buffer must be drained by seal)"
        );
        // [OPUS-4.8] (sq-x32t) Use the ERASURE-GRADE `vacuum`, not the serving-path `compact`:
        // vacuum re-interns into a fresh dictionary so a term VALUE orphaned by a DELETE / DROP
        // GRAPH (e.g. a personal-data literal) is physically purged from the on-disk dict blob,
        // not just from the triple indexes. `compact` keeps the dict for O(triples) folding,
        // which would leave the orphaned bytes on disk — not erasure-complete.
        self.graph.vacuum()
    }
}

impl ServerApplier {
    fn new(config: Arc<ServerConfig>) -> Self {
        Self {
            inner: GraphApplier::default(),
            config,
            durable: None,
        }
    }

    /// [OPUS-4.8] (sq-7cxr, gh-44) An applier whose committed batches are also mirrored to
    /// `graph` — a directory-backed [`Graph`] opened/created at `--persist <DIR>`. Every
    /// committed batch is re-applied to it WAL-durably in [`seal`] before publish.
    fn with_durable(config: Arc<ServerConfig>, graph: Graph) -> Self {
        Self {
            inner: GraphApplier::default(),
            config,
            durable: Some(DurableStore::new(graph)),
        }
    }
}

impl ApplyUpdates for ServerApplier {
    type Snapshot = Graph;
    type Working = Graph;
    type Update = String;

    fn fork(&mut self, base: &Graph) -> Result<Graph, String> {
        // [OPUS-4.8] (sq-7cxr) A fresh attempt (first fork of a batch OR a re-fork during the
        // writer's failure replay) restarts the durable batch buffer; only the updates that go
        // on to apply cleanly are re-accumulated, so `seal` mirrors exactly the published set.
        if let Some(d) = &mut self.durable {
            d.batch.clear();
        }
        with_engine_scope(&self.config, || self.inner.fork(base))
    }

    fn apply(&mut self, working: &mut Graph, update: &String) -> Result<(), String> {
        // [OPUS-4.8] sq-ebii: run the update under the SAME cooperative QueryBudget the read
        // paths use — the memory cap (`max_query_rows`) bounds a `DELETE/INSERT … WHERE`
        // whose WHERE blows up (cross-product alloc capped + abort at the row cap), and the
        // deadline (from `query_timeout`, measured from when the writer starts THIS update)
        // aborts a long WHERE evaluation cooperatively instead of running the writer thread
        // to an OOM. The deadline here is the writer-side cooperative stop; the HTTP side
        // separately hard-caps the client's await (see `await_update_worker`). With both
        // limits off (the default) this is the unlimited budget — identical to before.
        let budget = update_budget(&self.config);
        // [OPUS-4.8] (Copilot PR#80) When mirroring durably, CAPTURE the resolved effect log of
        // the in-memory application so the durable graph can replay the EXACT committed delta —
        // never re-executing the (possibly non-deterministic / side-effecting) update text. With
        // no durable mirror this is `update_in_place_with_budget` exactly as before (no capture).
        if let Some(d) = &mut self.durable {
            let effects = with_engine_scope(&self.config, || {
                sparq_engine::update_in_place_capturing(working, update, &budget)
            })?;
            // Record for the durable mirror ONLY after it applied cleanly in memory — a rejected
            // update is never persisted (it is not published either).
            d.batch.extend(effects);
        } else {
            with_engine_scope(&self.config, || {
                sparq_engine::update_in_place_with_budget(working, update, &budget)
            })?;
        }
        Ok(())
    }

    fn seal(&mut self, working: Graph) -> Result<Graph, String> {
        // [OPUS-4.8] (sq-7cxr, gh-44) Persist the committed batch to the durable graph BEFORE
        // returning — `seal` runs on the writer thread immediately before `publish`, so the
        // batch is WAL-durable (fsync'd by `Graph::apply_delta`) before the generation is
        // published and thus before any client ack.
        //
        // [OPUS-4.8] (Copilot PR#80) The batch is the RESOLVED effect log captured from the
        // in-memory application (see `apply`), replayed via `apply_effects` rather than by
        // re-executing the update text. This is what makes the durable graph BYTE-EQUIVALENT to
        // the published in-memory state even for non-deterministic / side-effecting updates
        // (`NOW()`/`RAND()`/`UUID()`/`BNODE()`, `LOAD <remote>`): both apply the identical
        // resolved triples, in the same order, starting from one shared seed.
        //
        // [OPUS-4.8] FAIL-CLOSED, GRACEFULLY (sq-vpx4, was sq-7cxr fatal-panic): a durable-write
        // error (disk full, I/O error) is propagated as `Err` rather than panicking the writer
        // thread. `seal` runs BEFORE `publish`, so returning `Err` here means the generation is
        // NEVER published and the in-flight batch's submitters are failed with
        // `WriteError::Unavailable` (HTTP 503, retryable) — NOT a false 2xx success. The
        // fail-closed correctness invariant is unchanged (a write that didn't durably commit is
        // never acked nor published); what changes is that a TRANSIENT error (e.g. a brief
        // `ENOSPC` that later clears) no longer kills the writer thread / the whole server. The
        // writer stays alive (degraded), so reads keep being served from the last published
        // snapshot and a subsequent write succeeds once durability recovers. A PERSISTENT error
        // simply yields repeated 503s. The in-memory `working` is dropped on `Err` (never
        // published), so the durable store and the published lineage cannot diverge.
        if let Some(d) = &mut self.durable {
            d.commit_batch().map_err(|e| {
                format!("durable persist failed (sq-vpx4); write refused (not acked, not published): {e}")
            })?;
        }
        self.inner.seal(working)
    }

    /// [OPUS-4.8] (sq-x32t) Out-of-band WAL COMPACTION / VACUUM of the durable store, for
    /// erasure-completeness. A no-op for an in-memory server (`durable == None`) — there is no
    /// on-disk history to purge. For a `--persist` server it physically rewrites the on-disk
    /// store to contain only the current live triples (see [`DurableStore::compact`]). It runs
    /// on the writer thread strictly between batches (the `Writer` sequences it through the same
    /// queue as updates), so the durable graph is never accessed concurrently and the
    /// compaction folds in every write that preceded the request. No generation is published —
    /// the live triple set is preserved exactly, so readers are unaffected throughout.
    fn maintain(&mut self) -> Result<(), String> {
        match &mut self.durable {
            Some(d) => d.compact(),
            // In-memory server: nothing on disk to compact (erasure is immediate — the dropped
            // generations free by `Arc` drop, no WAL history survives).
            None => Ok(()),
        }
    }
}

/// Shared server state: the dataset under query, served from a sparq-serve
/// [`GenerationRing`] of immutable snapshots. Queries pin the current generation
/// ([`AppState::current`], lock-free `ArcSwap` load) and evaluate against it for the
/// whole response; SPARQL Update submits through the single sequenced
/// [`sparq_serve::Writer`], which group-commits each batch as ONE new generation.
/// Also carries the hardening [`ServerConfig`] and the subscription plumbing (T23):
/// a `watch` channel that broadcasts the committed generation number to every open
/// `/subscriptions` socket, plus the global active-subscription counters.
///
/// **What this replaced (Wave A4):** the double-buffered `RwLock<Arc<Graph>>` writer —
/// spare-buffer reclaim via `Arc::try_unwrap` + 200 µs polling, lag replay, periodic
/// `compact_every` fold-back. The ring makes reclaim impossible *by design* (it retains
/// up to K old generations, so a published graph never drains back to the writer) and
/// unnecessary (old generations free by plain `Arc` drop) — removing the measured
/// §4.3/§4.4 pathologies: the 5.4 s/32 s pinned-snapshot writer stall and reclaim-poll
/// degradation under reader churn. `compact_every` went with it: the writer's per-batch
/// fork rebuilds a folded base, so overlays never accumulate across batches.
#[derive(Clone)]
pub struct AppState {
    /// The generation ring (Wave A1): lock-free `current()`, bounded retention.
    ring: Arc<GenerationRing<Graph>>,
    /// The single sequenced writer (Wave A2): the sole publisher of generations.
    /// `submit` blocks for the group-commit window + batch application, so update
    /// handlers call it on the blocking pool.
    writer: Arc<Writer<String>>,
    config: Arc<ServerConfig>,
    /// Committed generation number, advanced after every successful update. Subscription
    /// connections hold a `watch::Receiver` and re-evaluate when it changes; the watch
    /// channel inherently coalesces bursts (see `subscriptions` module docs).
    commits: Arc<tokio::sync::watch::Sender<u64>>,
    /// Global subscription bookkeeping shared by all `/subscriptions` connections.
    pub(crate) subs: Arc<crate::subscriptions::SubscriptionCounters>,
    /// Prometheus metrics (T22), exposed at `GET /metrics`.
    metrics: Arc<crate::metrics::Metrics>,
    /// [OPUS-4.8] (sq-gos8) The opt-in structured access-audit sink. `None` unless the operator
    /// configured one (`--access-audit` / `SPARQ_ACCESS_AUDIT`); shared across handlers.
    #[cfg(feature = "access-audit")]
    access_audit_sink: Option<Arc<dyn crate::access_audit::AuditSink>>,
}

impl AppState {
    pub fn new(graph: Graph) -> Self {
        Self::with_config(graph, ServerConfig::default())
    }

    /// Builds the state, panicking on a durable-persistence open error. Back-compat for the
    /// in-memory callers (which never set [`ServerConfig::persist_dir`], so it never fails);
    /// the binary and the persistence tests use [`try_with_config`](Self::try_with_config),
    /// which surfaces the open error cleanly.
    pub fn with_config(graph: Graph, config: ServerConfig) -> Self {
        Self::try_with_config(graph, config).expect("AppState::with_config: durable open failed")
    }

    /// [OPUS-4.8] (sq-7cxr, gh-44) Builds the state, returning the durable-persistence open
    /// error (rather than panicking) so the binary can print a clean message and exit non-zero.
    ///
    /// PERSISTENCE (`config.persist_dir == Some(dir)`): the on-disk index at `dir` is the
    /// durable source of truth. If `dir` already holds a store, it is OPENED — replaying its
    /// write-ahead log so prior updates are present with **no rebuild** — and the passed-in
    /// `graph` seed is IGNORED (re-loading a data file over an existing durable store would be
    /// destructive; the store, like QLever's persisted index, wins). If `dir` is empty/absent,
    /// the `graph` seed is SAVED there and then re-opened so it carries a WAL. The ring is then
    /// seeded from the durable graph's snapshot, and the writer's applier mirrors every
    /// committed batch back to the durable graph (WAL-durable before publish — see
    /// [`ServerApplier::seal`]). With `persist_dir == None` this is exactly the historical
    /// in-memory path (the ring is seeded from `graph`; no durable mirror; never errors).
    pub fn try_with_config(graph: Graph, config: ServerConfig) -> Result<Self, String> {
        Self::try_with_config_inner(
            graph,
            config,
            #[cfg(any(test, feature = "test-seams"))]
            None,
        )
    }

    /// [OPUS-4.8] (sq-vpx4) Like [`try_with_config`](Self::try_with_config) but installs a
    /// durable-write failure-injection hook on the `--persist` seal path. The hook is called
    /// once per seal; returning `Some(msg)` makes that seal fail (modelling a transient/persistent
    /// I/O error), `None` lets the real durable commit run. For the graceful-degradation
    /// integration test ONLY — gated behind the `test-seams` feature so it cannot exist in a
    /// production build. Requires `config.persist_dir` to be set (else the hook has nothing to
    /// gate).
    #[cfg(feature = "test-seams")]
    pub fn with_config_inject_durable_failure(
        graph: Graph,
        config: ServerConfig,
        fail_seal: Box<dyn FnMut() -> Option<String> + Send>,
    ) -> Result<Self, String> {
        Self::try_with_config_inner(graph, config, Some(fail_seal))
    }

    fn try_with_config_inner(
        graph: Graph,
        config: ServerConfig,
        #[cfg(any(test, feature = "test-seams"))] fail_seal: Option<
            Box<dyn FnMut() -> Option<String> + Send>,
        >,
    ) -> Result<Self, String> {
        // [OPUS-4.8] sq-7cxr: resolve the durable graph (open existing / create fresh) when a
        // persistence dir is configured. `seed` is what the ring starts from; `durable` is the
        // on-disk graph the writer mirrors committed batches to.
        let (seed, durable) = match &config.persist_dir {
            Some(dir) => {
                let g = open_or_create_durable(dir, graph)?;
                // Seed the ring from an independent snapshot so the ring's lineage and the
                // durable graph are distinct objects (the durable graph keeps its WAL; the
                // ring's published snapshots are pure in-memory forks queried concurrently).
                (g.snapshot().into_graph(), Some(g))
            }
            None => (graph, None),
        };

        // Default ring (concurrency retention only); the opt-in `time-travel`
        // feature extends retention so `?generation=N` has history to serve.
        #[cfg(not(feature = "time-travel"))]
        let ring_config = sparq_serve::RingConfig::default();
        #[cfg(feature = "time-travel")]
        let ring_config = sparq_serve::RingConfig {
            time_travel: Some(sparq_serve::TimeTravelConfig {
                max_generations: config.time_travel_generations,
                max_age: config.time_travel_max_age,
            }),
            ..sparq_serve::RingConfig::default()
        };
        let ring = Arc::new(GenerationRing::with_config(seed, ring_config));
        // [OPUS-4.8] sq-4w18: share the config Arc with the writer so its update path
        // enforces the same SERVICE egress allowlist (a federated `INSERT … WHERE` is
        // gated like a read).
        let config = Arc::new(config);
        // [OPUS-4.8] sq-7cxr: a durable-mirroring applier when persistence is on, else the
        // historical in-memory applier.
        let applier = match durable {
            Some(g) => {
                #[allow(unused_mut)]
                let mut a = ServerApplier::with_durable(config.clone(), g);
                // [OPUS-4.8] (sq-vpx4) Install the durable-write failure-injection hook, if any
                // (test-seams only). A no-op in production (the field/parameter do not exist).
                #[cfg(any(test, feature = "test-seams"))]
                if let (Some(d), Some(hook)) = (a.durable.as_mut(), fail_seal) {
                    d.fail_seal = Some(hook);
                }
                a
            }
            None => ServerApplier::new(config.clone()),
        };
        let writer = Arc::new(Writer::spawn(
            ring.clone(),
            applier,
            WriterConfig::default(),
        ));
        // [OPUS-4.8] sq-gos8: open the structured access-audit sink if one is configured. A
        // failure to open (e.g. an unwritable audit-file path) is surfaced as a clean startup
        // error rather than silently dropping the trail — the operator asked for an audit log.
        #[cfg(feature = "access-audit")]
        let access_audit_sink = match &config.access_audit {
            Some(target) => Some(
                crate::access_audit::make_sink(target)
                    .map_err(|e| format!("access-audit sink open failed: {e}"))?,
            ),
            None => None,
        };
        Ok(Self {
            ring,
            writer,
            config,
            commits: Arc::new(tokio::sync::watch::channel(0).0),
            subs: Arc::new(crate::subscriptions::SubscriptionCounters::default()),
            metrics: Arc::new(crate::metrics::Metrics::default()),
            #[cfg(feature = "access-audit")]
            access_audit_sink,
        })
    }

    /// The server's Prometheus metrics (T22).
    pub(crate) fn metrics(&self) -> &crate::metrics::Metrics {
        &self.metrics
    }

    /// Pins the current generation for a request: lock-free, ~10–20 ns, never blocked
    /// by an in-flight update. Hold the returned `Arc` for as long as the response is
    /// being produced; `gen.snapshot()` is the immutable [`Graph`] to evaluate against.
    pub fn current(&self) -> PinnedGen {
        self.ring.current()
    }

    /// Pins the retained generation numbered `number` (time travel): `None` when it
    /// was never published or has aged out of the retention window. See
    /// [`sparq_serve::GenerationRing::at`].
    #[cfg(feature = "time-travel")]
    pub fn at(&self, number: u64) -> Option<PinnedGen> {
        self.ring.at(number)
    }

    /// Applies a SPARQL Update through the sequenced writer: the update joins the
    /// writer's current group-commit window and this call **blocks until the generation
    /// containing it is published** (group-commit ack). Failed updates stay atomic to
    /// clients — the writer applies batches to a private working copy, skips the failing
    /// update, and re-forks, so the published chain never contains a partial effect.
    ///
    /// On success, advances the commit watch so active subscriptions re-evaluate (T23)
    /// and returns the number of the published generation containing the update — the
    /// read-your-writes token a client can later pin with `?generation=N` (time travel)
    /// or compare against `Sparq-Generation` response headers. The advance happens
    /// strictly *after* the publish (submit returns the published generation number),
    /// so a woken subscription always pins a generation at least as new as the commit
    /// it was woken for. Blocking (window + batch application): call it on the
    /// blocking pool.
    pub fn apply_update(&self, sparql: &str) -> Result<u64, String> {
        // [OPUS-4.8] (sq-uqh, Wave B) Extract the per-named-graph visibility scope the
        // update writes (§6.3/§6.5) so the publish bumps only those pods' epochs — a
        // write to graph A no longer churns a cache scoped to graph B. Cross-graph /
        // default-graph / dynamically-scoped writes still fall back to the global pod
        // (conservative, never under-invalidating). See [`touched_pods`].
        let touched = touched_pods(sparql);
        match self.writer.submit(sparql.to_string(), touched) {
            Ok(number) => {
                // Monotonic max: batch-mates share a generation number and may ack in
                // any order relative to a later batch's submitters.
                self.commits.send_if_modified(|g| {
                    if number > *g {
                        *g = number;
                        true
                    } else {
                        false
                    }
                });
                Ok(number)
            }
            Err(WriteError::Rejected(e)) => Err(e),
            Err(WriteError::Shutdown) => Err("update writer has shut down".to_string()),
            // [OPUS-4.8] (sq-vpx4) Durable-write failure (e.g. transient ENOSPC on the
            // `--persist` mirror): the write was REFUSED — not durably committed, not
            // published, not acked. It is retryable, so it maps to HTTP 503 (see
            // `update_rejection_response`'s `DURABLE_UNAVAILABLE_PREFIX` sniff), NOT a
            // 400 (the update itself was valid) and NOT a 500 (the server is healthy
            // and still serving reads). The writer thread is alive; the client may retry.
            Err(WriteError::Unavailable(e)) => Err(format!("{DURABLE_UNAVAILABLE_PREFIX}{e}")),
        }
    }

    /// [OPUS-4.8] (sq-x32t) Compacts/vacuums the durable on-disk store for ERASURE-
    /// COMPLETENESS: physically rewrites the `--persist` directory so only the current live
    /// triples remain, dropping superseded INSERTs / DELETEd data / DROPped graphs that still
    /// linger in earlier WAL segments. **Blocks until the compaction completes** (it runs on
    /// the writer thread, strictly between batches — see [`Writer::maintain`]). The live triple
    /// set is preserved exactly, so no generation is published and readers are unaffected.
    ///
    /// Returns `Ok(())` on success (including the no-op for an in-memory server with no persist
    /// dir — there is no on-disk history to purge, so erasure is already complete). `Err(msg)`
    /// is a compaction failure (e.g. an I/O error during the durable rewrite); the writer thread
    /// stays alive and reads keep being served from the last published snapshot. Blocking: call
    /// it on the blocking pool.
    pub fn compact(&self) -> Result<(), String> {
        match self.writer.maintain() {
            Ok(res) => res,
            Err(WriteError::Shutdown) => Err("update writer has shut down".to_string()),
            // `maintain` only ever returns `Ok(_)` or `Shutdown` from the writer.
            Err(e) => Err(e.to_string()),
        }
    }

    /// A receiver of the committed generation number for a subscription connection (T23).
    pub(crate) fn subscribe_commits(&self) -> tokio::sync::watch::Receiver<u64> {
        self.commits.subscribe()
    }

    pub(crate) fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// [OPUS-4.8] (sq-gos8) The installed structured access-audit sink, if any. `None` short-
    /// circuits every audit call site (no record is built), so a request with no sink configured
    /// pays only this `Option` check.
    #[cfg(feature = "access-audit")]
    pub(crate) fn access_audit_sink(&self) -> Option<&Arc<dyn crate::access_audit::AuditSink>> {
        self.access_audit_sink.as_ref()
    }
}

/// Builds the application router for the SPARQL Protocol + GSP-read endpoints, hardened
/// with the state's [`ServerConfig`] guards (see [`harden`]).
pub fn router(state: AppState) -> Router {
    let config = state.config.clone();
    let routes = Router::new()
        // SPARQL 1.1 Protocol — query operation. `any` so we can return a proper 405 with
        // an `Allow` header for unsupported methods (update verbs are T11b).
        .route("/sparql", any(sparql_endpoint))
        // Graph Store HTTP Protocol (indirect graph identification): ?graph=<uri> / ?default
        .route("/sparql/graph", any(graph_store_indirect))
        // Graph Store HTTP Protocol (direct graph identification).
        // [OPUS-4.8] axum 0.8 (matchit 0.8) wildcard-capture syntax: `/*path` -> `/{*path}`.
        .route("/graphs/{*path}", any(graph_store_direct))
        // SEPA-style SPARQL subscriptions over WebSocket (T23).
        .route(
            "/subscriptions",
            get(crate::subscriptions::subscriptions_endpoint),
        )
        // [OPUS-4.8] sq-bxog: the same subscription engine over Server-Sent Events
        // (text/event-stream) — one subscription per stream, query in the query string.
        .route(
            "/subscriptions/sse",
            get(crate::subscriptions::sse::sse_endpoint),
        )
        // Liveness.
        .route("/health", get(|| async { "ok" }))
        // [OPUS-4.8] (sq-x32t) ADMIN: WAL compaction/vacuum for erasure-completeness. POST-only
        // (it mutates the durable on-disk store), gated by the WRITE auth token (the existing
        // admin gate). Physically purges superseded/deleted data from the on-disk WAL history so
        // a logical DELETE / DROP GRAPH is followed by real erasure. See [`admin_compact`].
        .route("/admin/compact", post(admin_compact))
        // Prometheus metrics (T22).
        .route("/metrics", get(metrics_endpoint));
    // [OPUS-4.8] sq-d3d8 (epic sq-3183): OPT-IN federation discovery — the VoID dataset
    // description at /.well-known/void. Compiled only with the `federation-descriptors`
    // feature; even then the handler refuses (404) unless the config flag is set. The SD is
    // served on the existing /sparql GET path (no extra route — it is the protocol's
    // "GET with no query" response). See [`crate::descriptors`].
    #[cfg(feature = "federation-descriptors")]
    let routes = routes.route("/.well-known/void", get(well_known_void));
    // [OPUS-4.8] sq-bzh1 (epic sq-3183): OPT-IN Triple Pattern Fragments / LDF source endpoint.
    // Compiled only with the `tpf` feature; even then the handler refuses (404) unless the config
    // flag is set. READ-only — a GET (with HEAD) only. See [`crate::tpf`].
    // [OPUS-4.8] sq-dxhb: with the `brtpf` feature it ALSO accepts POST — a brTPF client posts a
    // (potentially large) binding set in the body, which would not fit a query string. POST is
    // still a READ (it returns a fragment; it never mutates the store) and is gated by the same
    // read auth + flag.
    #[cfg(all(feature = "tpf", not(feature = "brtpf")))]
    let routes = routes.route("/tpf", get(tpf_endpoint).head(tpf_endpoint));
    #[cfg(feature = "brtpf")]
    let routes = routes.route(
        "/tpf",
        get(tpf_endpoint).head(tpf_endpoint).post(tpf_endpoint),
    );
    let routes = routes.with_state(state.clone());
    // The metrics middleware wraps the WHOLE hardened stack so shed requests
    // (429), body-limit rejections (413) and panics (500) are counted with the
    // status the client actually saw.
    harden(routes, &config).layer(axum::middleware::from_fn_with_state(
        state,
        crate::metrics::track,
    ))
}

/// `GET /metrics` — Prometheus text exposition (T22). The gauges (graph triple
/// count, active subscriptions) are read at scrape time from live state.
async fn metrics_endpoint(State(state): State<AppState>) -> Response {
    let triples = state.current().snapshot().len();
    let subs = state.subs.active_count();
    let body = state.metrics().render(subs, triples);
    text_response(
        StatusCode::OK,
        "text/plain; version=0.0.4; charset=utf-8",
        body,
        false,
    )
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-d3d8 (epic sq-3183) — OPT-IN federation discovery descriptors
// ---------------------------------------------------------------------------

/// [OPUS-4.8] sq-d3d8: derives the base URL (`scheme://host`) this server is reached at,
/// from the request `Host` header. Used to name the VoID dataset, the `sd:Service`/endpoint
/// and the `dcterms:source` link in the descriptors, so they self-describe the URL a client
/// actually used to fetch them.
///
/// Scheme is `http` (the server terminates plain HTTP; a TLS-terminating reverse proxy
/// forwarding `X-Forwarded-Proto` is out of scope for this minimal discovery surface). If
/// there is no usable `Host` header (HTTP/1.0 without one), falls back to `http://localhost`
/// so the descriptor is always well-formed RDF rather than a 500.
#[cfg(feature = "federation-descriptors")]
fn request_base(headers: &HeaderMap) -> String {
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|h| !h.is_empty())
        .unwrap_or("localhost");
    let base = format!("http://{host}");
    // The Host header is attacker-controlled; a value that makes `http://{host}` an
    // invalid IRI (spaces, control chars, `<`/`>`, …) would otherwise propagate into the
    // descriptor IRIs and yield malformed RDF → a 500. Validate here and fall back to a
    // fixed safe base so the descriptor is always well-formed RDF, as the doc promises.
    if oxrdf::NamedNode::new(&base).is_ok() {
        base
    } else {
        "http://localhost".to_string()
    }
}

/// [OPUS-4.8] sq-d3d8: `GET /.well-known/void` — the W3C VoID dataset description (read-only).
///
/// OPT-IN: returns `404` unless [`ServerConfig::federation_descriptors`] is set (the route is
/// mounted only with the `federation-descriptors` feature, and the handler refuses unless the
/// operator also turned the flag on). Reads a pinned snapshot and delegates generation to
/// [`crate::descriptors::void_descriptor`] (`Introspection::to_void`). Content-negotiates
/// Turtle (default) / N-Triples / RDF-XML from `Accept`. As a read, it is gated by
/// `--auth-token-read` like any other GET.
#[cfg(feature = "federation-descriptors")]
async fn well_known_void(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if !state.config().federation_descriptors {
        return json_error(StatusCode::NOT_FOUND, "not found");
    }
    if let Some(resp) = auth_gate(state.config(), &headers, Operation::Read) {
        return resp;
    }
    let base = request_base(&headers);
    let dataset_iri = format!("{base}/.well-known/void#dataset");
    let accept = headers.get(header::ACCEPT).and_then(|v| v.to_str().ok());
    // Pin the current generation; generate against its immutable snapshot.
    let pin = state.current();
    match crate::descriptors::void_descriptor(pin.snapshot(), &dataset_iri, accept) {
        Ok(d) => text_response(StatusCode::OK, d.content_type, d.body, false),
        // [OPUS-4.8] (sq-kfel, ASVS-G3) `e` is an internal serializer error — withhold it.
        Err(e) => sanitized_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "void-descriptor",
            "failed to generate dataset description",
            &e,
        ),
    }
}

/// [OPUS-4.8] sq-d3d8: the SPARQL 1.1 Service Description served for a `GET /sparql` with no
/// `query` parameter (SPARQL Protocol §2.1.2 / Service Description §2). Returns `Some(resp)`
/// when the descriptor should be served (the feature is compiled in AND the config flag is
/// set), or `None` to fall through to the historical `400 missing 'query'`.
///
/// Advertises the endpoint, the supported query languages, the supported result formats and
/// the default dataset (linked to the VoID document). Content-negotiates from `Accept`.
#[cfg(feature = "federation-descriptors")]
fn service_description_response(state: &AppState, headers: &HeaderMap) -> Option<Response> {
    if !state.config().federation_descriptors {
        return None;
    }
    let base = request_base(headers);
    let endpoint_iri = format!("{base}/sparql");
    let dataset_iri = format!("{base}/.well-known/void#dataset");
    let accept = headers.get(header::ACCEPT).and_then(|v| v.to_str().ok());
    let caps = service_capabilities(state.config());
    // [OPUS-4.8] sq-optl: enumerate the served dataset's named graphs so the SD advertises each
    // as an sd:namedGraph (not just the default graph). Read off the same pinned snapshot the
    // VoID descriptor uses, so the two descriptors describe one consistent dataset state.
    let pin = state.current();
    let named_graphs = crate::descriptors::named_graph_descriptions(pin.snapshot());
    Some(
        match crate::descriptors::service_description(
            &endpoint_iri,
            &endpoint_iri,
            &dataset_iri,
            &caps,
            &named_graphs,
            accept,
        ) {
            Ok(d) => text_response(StatusCode::OK, d.content_type, d.body, false),
            // [OPUS-4.8] (sq-kfel, ASVS-G3) `e` is an internal serializer error — withhold it.
            Err(e) => sanitized_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "service-description",
                "failed to generate service description",
                &e,
            ),
        },
    )
}

/// [OPUS-4.8] sq-qfcb: derive the server's ACTUAL capability profile for the Service
/// Description from this build's cargo features + the running config — never a fiction.
///
///   * `update` — advertise `sd:SPARQL11Update` only when an anonymous client could run one:
///     the write path is always compiled in, but a configured write-token
///     ([`ServerConfig::auth_token`]) makes anonymous Update impossible, so we suppress the
///     advertisement in that case (the operator who holds the token still knows it works).
///   * `federated_query` — `true` exactly when built with the `service` feature (the engine's
///     `SERVICE` evaluation is compiled in); otherwise a `SERVICE` clause errors at execution.
///   * `extension_functions` — the IRIs the engine has ACTUALLY registered. With the `geo`
///     feature that is sparq-geo's `geof:` registry, read back through
///     [`sparq_engine::FunctionRegistry::iris`] so the list can never drift from what runs;
///     without it, no extension functions are registered, so the list is empty.
#[cfg(feature = "federation-descriptors")]
fn service_capabilities(config: &ServerConfig) -> crate::descriptors::Capabilities {
    // Anonymous Update is possible iff no write-token gates the write surface.
    let update = config.auth_token.is_none();
    let federated_query = cfg!(feature = "service");
    #[cfg(feature = "geo")]
    let extension_functions: Vec<String> = {
        static GEOF: std::sync::OnceLock<sparq_engine::FunctionRegistry> =
            std::sync::OnceLock::new();
        GEOF.get_or_init(sparq_geo::geof_registry)
            .iris()
            .map(str::to_string)
            .collect()
    };
    #[cfg(not(feature = "geo"))]
    let extension_functions: Vec<String> = Vec::new();
    crate::descriptors::Capabilities {
        update,
        federated_query,
        extension_functions,
    }
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-bzh1 (epic sq-3183) — OPT-IN Triple Pattern Fragments / LDF source endpoint
// ---------------------------------------------------------------------------

/// [OPUS-4.8] sq-bzh1: derives `scheme://host` from the request `Host` header for the TPF
/// fragment / dataset / template URLs (independent of the `federation-descriptors`-gated
/// [`request_base`] so the two opt-in features never depend on each other). Same `http` +
/// safe-`localhost` fallback posture: a hostile `Host` that would make an invalid IRI falls
/// back so the fragment is always well-formed RDF rather than a 500.
#[cfg(feature = "tpf")]
fn tpf_base(headers: &HeaderMap) -> String {
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|h| !h.is_empty())
        .unwrap_or("localhost");
    let base = format!("http://{host}");
    if oxrdf::NamedNode::new(&base).is_ok() {
        base
    } else {
        "http://localhost".to_string()
    }
}

/// [OPUS-4.8] sq-bzh1: percent-encodes a query-parameter VALUE for building the `hydra:next` /
/// `hydra:previous` page URLs. Encodes everything outside the RFC 3986 unreserved set plus the
/// few sub-delims safe in a query value, so an N-Triples term value (`<`, `>`, `"`, spaces, …)
/// round-trips through the URL back to the same term. Hand-rolled to avoid a new dependency.
#[cfg(feature = "tpf")]
fn pct_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// [OPUS-4.8] sq-bzh1: builds a `/tpf` page URL for the given pattern parameters and page,
/// re-encoding the bound positions so the link is self-contained.
#[cfg(feature = "tpf")]
fn tpf_page_url(
    base: &str,
    subject: Option<&str>,
    predicate: Option<&str>,
    object: Option<&str>,
    page: usize,
) -> String {
    let mut url = format!("{base}/tpf");
    let mut sep = '?';
    let add = |url: &mut String, sep: &mut char, k: &str, v: Option<&str>| {
        if let Some(v) = v.map(str::trim).filter(|v| !v.is_empty()) {
            url.push(*sep);
            url.push_str(k);
            url.push('=');
            url.push_str(&pct_encode(v));
            *sep = '&';
        }
    };
    add(&mut url, &mut sep, "subject", subject);
    add(&mut url, &mut sep, "predicate", predicate);
    add(&mut url, &mut sep, "object", object);
    // Page is always present so the URL is an unambiguous fragment identifier (page 0 included).
    url.push(sep);
    url.push_str(&format!("page={page}"));
    url
}

/// [OPUS-4.8] sq-bzh1: negotiates the TPF fragment serialisation. TPF clients send
/// `Accept: text/turtle` (or N-Triples); we reuse the graph negotiation but DEFAULT to Turtle
/// (the conventional, control-readable TPF serialisation), matching the descriptor surface.
#[cfg(feature = "tpf")]
fn negotiate_tpf(accept: Option<&str>) -> GraphFormat {
    match accept {
        Some(a) if !a.trim().is_empty() => {
            let f = negotiate_graph(Some(a));
            // negotiate_graph defaults to N-Triples on an unrecognised header; for TPF a bare
            // `*/*` (or anything unmatched) should land on Turtle.
            if a.contains("n-triples") || a.contains("turtle") || a.contains("rdf+xml") {
                f
            } else {
                GraphFormat::Turtle
            }
        }
        _ => GraphFormat::Turtle,
    }
}

/// [OPUS-4.8] sq-bzh1: `GET /tpf?subject=&predicate=&object=` — the Triple Pattern Fragments /
/// LDF source endpoint (read-only). Returns a paged RDF fragment of the triples matching the
/// pattern, with Hydra controls (`hydra:totalItems` from the cheap cardinality estimate,
/// `hydra:next`/`hydra:previous` paging, the `hydra:search` template).
///
/// OPT-IN: returns `404` unless [`ServerConfig::tpf`] is set (the route is mounted only with the
/// `tpf` feature, and the handler refuses unless the operator also turned the flag on). As a
/// read, it is gated by `--auth-token-read` like any other GET. Content-negotiates Turtle
/// (default) / N-Triples / RDF-XML from `Accept`.
#[cfg(feature = "tpf")]
async fn tpf_endpoint(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
    body: Bytes,
) -> Response {
    use crate::tpf::{evaluate, fragment_triples, TriplePattern, DEFAULT_PAGE_SIZE};

    if !state.config().tpf {
        return json_error(StatusCode::NOT_FOUND, "not found");
    }
    // GET/HEAD always; POST only with brTPF (a binding set too large for a query string).
    #[cfg(not(feature = "brtpf"))]
    let method_ok = method == Method::GET || method == Method::HEAD;
    #[cfg(feature = "brtpf")]
    let method_ok = matches!(method, Method::GET | Method::HEAD | Method::POST);
    if !method_ok {
        #[cfg(not(feature = "brtpf"))]
        return method_not_allowed(&[Method::GET, Method::HEAD]);
        #[cfg(feature = "brtpf")]
        return method_not_allowed(&[Method::GET, Method::HEAD, Method::POST]);
    }
    // A fragment is a READ even via POST (brTPF posts the binding set; it never writes).
    if let Some(resp) = auth_gate(state.config(), &headers, Operation::Read) {
        return resp;
    }
    let head_only = method == Method::HEAD;
    // The body is unused without brTPF (GET/HEAD carry none); silence the warning.
    #[cfg(not(feature = "brtpf"))]
    let _ = &body;

    let params = parse_form(raw_query.as_deref().unwrap_or(""));
    let subject = params.get("subject").map(String::as_str);
    let predicate = params.get("predicate").map(String::as_str);
    let object = params.get("object").map(String::as_str);

    // Parse the triple pattern; a malformed term is a 400.
    // [OPUS-4.8] (sq-kfel, ASVS-G3) `e` quotes the caller's offending term verbatim — the same
    // echo-of-input info-leak class the rest of the surface sanitizes (sq-cz89/sq-j9zs). Withhold
    // it from the client; the operator gets the full parse error in the server log.
    let pattern = match TriplePattern::parse(subject, predicate, object) {
        Ok(p) => p,
        Err(e) => {
            return sanitized_error(
                StatusCode::BAD_REQUEST,
                "tpf-term-parse",
                "malformed triple pattern term",
                &e.to_string(),
            )
        }
    };
    // Page number (0-based). A non-numeric / absent page is page 0.
    let page = params
        .get("page")
        .and_then(|p| p.trim().parse::<usize>().ok())
        .unwrap_or(0);
    let page_size = DEFAULT_PAGE_SIZE;

    let base = tpf_base(&headers);
    let dataset_url = format!("{base}/tpf");

    // [OPUS-4.8] sq-dxhb: the brTPF binding set. The mappings come from the `values` query
    // parameter (GET) or — preferred for a large set — the request BODY (POST). An empty / absent
    // payload is the plain-TPF case (no restriction). A malformed payload is a 400.
    #[cfg(feature = "brtpf")]
    let bindings = {
        // POST body if non-empty, else the `values` query parameter.
        let raw_values: std::borrow::Cow<'_, str> = if !body.is_empty() {
            String::from_utf8_lossy(&body)
        } else {
            std::borrow::Cow::Borrowed(params.get("values").map(String::as_str).unwrap_or(""))
        };
        match crate::tpf::parse_bindings(&raw_values) {
            Ok(b) => b,
            Err(e) => {
                return sanitized_error(
                    StatusCode::BAD_REQUEST,
                    "brtpf-bindings-parse",
                    "malformed brTPF binding set",
                    &e.to_string(),
                )
            }
        }
    };

    // The template advertises the brTPF `{values}` control only when the feature is compiled.
    #[cfg(feature = "brtpf")]
    let template = format!("{base}/tpf{{?subject,predicate,object,values}}");
    #[cfg(not(feature = "brtpf"))]
    let template = format!("{base}/tpf{{?subject,predicate,object}}");

    // Pin the current generation; evaluate against its immutable snapshot. With a non-empty
    // brTPF binding set the fragment is the bindings-RESTRICTED result; otherwise it is plain TPF.
    let pin = state.current();
    #[cfg(feature = "brtpf")]
    let frag = if bindings.is_empty() {
        evaluate(pin.snapshot(), &pattern, page, page_size)
    } else {
        crate::tpf::evaluate_brtpf(pin.snapshot(), &pattern, &bindings, page, page_size)
    };
    #[cfg(not(feature = "brtpf"))]
    let frag = evaluate(pin.snapshot(), &pattern, page, page_size);

    let fragment_url = tpf_page_url(&base, subject, predicate, object, page);
    let next_url = frag
        .has_next()
        .then(|| tpf_page_url(&base, subject, predicate, object, page + 1));
    let prev_url = frag
        .has_previous()
        .then(|| tpf_page_url(&base, subject, predicate, object, page - 1));
    // [OPUS-4.8] sq-dxhb: the fuller Hydra paging vocabulary — hydra:first (always page 0) and
    // hydra:last (the page holding the final match). Emitted on every page.
    let first_url = tpf_page_url(&base, subject, predicate, object, 0);
    let last_url = tpf_page_url(&base, subject, predicate, object, frag.last_page());

    let triples = fragment_triples(
        &frag,
        &fragment_url,
        &dataset_url,
        &template,
        next_url.as_deref(),
        prev_url.as_deref(),
        Some(&first_url),
        Some(&last_url),
    );

    let fmt = negotiate_tpf(headers.get(header::ACCEPT).and_then(|v| v.to_str().ok()));
    let body = match fmt {
        GraphFormat::NTriples => crate::graph::triples_to_ntriples(&triples),
        GraphFormat::Turtle => crate::graph::triples_to_turtle(&triples),
        GraphFormat::RdfXml => crate::graph::triples_to_rdfxml(&triples),
    };
    text_response(StatusCode::OK, fmt.content_type(), body, head_only)
}

/// Applies the T15 hardening middleware stack to a router (outermost first):
/// optional request logging, panic→500, concurrency limit with load-shedding→429,
/// JSON error bodies, and the request body-size limit→413. Public so integration
/// tests can wrap probe routes (e.g. a deliberately panicking handler) in the
/// exact production stack.
pub fn harden(routes: Router, config: &ServerConfig) -> Router {
    let routes = routes.layer(
        ServiceBuilder::new()
            // Panic in any inner layer/handler => 500 with a JSON body, not a dead socket.
            .layer(CatchPanicLayer::custom(
                |_: Box<dyn std::any::Any + Send>| {
                    json_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal server error (panic)",
                    )
                },
            ))
            // [OPUS-4.8] sq-cmvh (ASVS V14.4): stamp the security hardening headers onto EVERY
            // response. Placed second so it runs LAST on the response path (after the
            // panic-catcher, the load-shed 429 mapper and `json_error_bodies`), guaranteeing the
            // headers land on success, streamed, error and panic responses alike. See
            // `security_headers` / `SECURITY_HEADERS`.
            .layer(axum::middleware::map_response(security_headers))
            // Load-shed converts "concurrency limit reached" into an immediate error
            // (mapped to 429) instead of queueing unboundedly.
            .layer(HandleErrorLayer::new(|err: BoxError| async move {
                if err.is::<tower::load_shed::error::Overloaded>() {
                    json_error(
                        StatusCode::TOO_MANY_REQUESTS,
                        "server is at its concurrent-request limit, retry later",
                    )
                } else {
                    // [OPUS-4.8] (sq-kfel, ASVS-G3) Defensive fallback (in this configured stack
                    // only `load_shed` produces a `BoxError`, handled above, so this is
                    // effectively unreachable). Do NOT Display the internal `err` into the body —
                    // that would leak the internal error type / chain. Log it server-side, return
                    // a generic 500.
                    tracing::warn!(target: "sparq_server", detail = %err, "unexpected middleware error (detail withheld from client)");
                    json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
                }
            }))
            .load_shed()
            .concurrency_limit(config.max_concurrent)
            // Normalise plain-text error bodies (e.g. extractor rejections like the
            // body-size 413) into the structured JSON shape used everywhere else.
            .layer(axum::middleware::map_response(json_error_bodies))
            .layer(DefaultBodyLimit::max(config.max_body_bytes)),
    );
    if config.verbose {
        // [OPUS-4.8] sq-toze.34: with redaction ON (the default), swap tower-http's
        // `DefaultMakeSpan` (which records the RAW request URI — and so the full
        // `?query=…` SPARQL text, a PII exposure) for a span that records a redacted
        // target (path verbatim, query string => `<redacted len=N fp=…>`). With
        // --log-full-requests the bare TraceLayer is used, logging the URI verbatim as before.
        if config.redact_logs {
            routes
                .layer(TraceLayer::new_for_http().make_span_with(crate::redact::RedactingMakeSpan))
        } else {
            routes.layer(TraceLayer::new_for_http())
        }
    } else {
        routes
    }
}

// ---------------------------------------------------------------------------
// SPARQL 1.1 Protocol — query operation
// ---------------------------------------------------------------------------

const SPARQL_QUERY_CT: &str = "application/sparql-query";
const FORM_CT: &str = "application/x-www-form-urlencoded";
/// `Accept` media type that turns a query request into an EXPLAIN response (T22).
const EXPLAIN_CT: &str = "text/x-sparq-explain";

/// How a `/sparql` query request should be answered (T22 EXPLAIN).
#[derive(Clone, Copy, PartialEq, Eq)]
enum ExplainMode {
    /// Execute normally (the default).
    Off,
    /// Return the engine's planning-only plan text instead of executing.
    Plan,
    /// Execute under the normal budget and return the plan + per-operator trace.
    Analyze,
}

/// EXPLAIN is requested via an `explain` parameter (URL query string, or the
/// url-encoded POST body — the body wins) — `explain`, `explain=true`,
/// `explain=plan` for the dry run, `explain=analyze` to execute and trace — or
/// via `Accept: text/x-sparq-explain` (plan only).
fn explain_mode(
    url_params: &HashMap<String, String>,
    body_params: Option<&HashMap<String, String>>,
    headers: &HeaderMap,
) -> ExplainMode {
    if let Some(v) = body_params
        .and_then(|m| m.get("explain"))
        .or_else(|| url_params.get("explain"))
    {
        return match v.to_ascii_lowercase().as_str() {
            "false" | "0" | "off" | "no" => ExplainMode::Off,
            "analyze" | "analyse" => ExplainMode::Analyze,
            _ => ExplainMode::Plan,
        };
    }
    let accepts_explain = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|a| a.contains(EXPLAIN_CT));
    if accepts_explain {
        ExplainMode::Plan
    } else {
        ExplainMode::Off
    }
}

// ---------------------------------------------------------------------------
// Time-travel pinning (opt-in `time-travel` cargo feature)
// ---------------------------------------------------------------------------

/// Resolves the generation a query request is pinned to (opt-in `time-travel`
/// feature). `generation=N` — URL query string, or the url-encoded POST body,
/// the body winning (same precedence as `explain`) — pins the retained
/// generation N for the whole request: the response is the store *as of* that
/// generation. Chosen over an `As-Of` timestamp header because it fits the
/// endpoint's existing parameter contract (`query=`, `explain=`) and the number
/// is the exact token the server itself hands out in `Sparq-Generation` (the
/// read-your-writes / shard_seq concept — no clock resolution or skew
/// ambiguity); callers that track timestamps resolve them via the library's
/// `GenerationRing::as_of`.
///
/// Errors per the endpoint's status semantics: unparsable number → 400; not yet
/// published → 400 (the client cannot have obtained that token from this
/// server); published but no longer retained → **410 Gone** (it aged out of the
/// retention window — gone permanently, retrying cannot help).
#[cfg(feature = "time-travel")]
#[allow(clippy::result_large_err)] // Err is axum's Response; boxing would desync the cfg variants
fn resolve_pin(
    state: &AppState,
    url_params: &HashMap<String, String>,
    body_params: Option<&HashMap<String, String>>,
) -> Result<PinnedGen, Response> {
    let raw = match body_params
        .and_then(|m| m.get("generation"))
        .or_else(|| url_params.get("generation"))
    {
        Some(raw) => raw,
        None => return Ok(state.current()),
    };
    let number: u64 = raw.parse().map_err(|_| {
        bad_request(&format!(
            "invalid 'generation' parameter '{raw}': expected a generation number"
        ))
    })?;
    let current = state.current();
    if number > current.number() {
        return Err(bad_request(&format!(
            "generation {number} has not been published yet (current generation: {})",
            current.number()
        )));
    }
    if number == current.number() {
        return Ok(current);
    }
    state.ring.at(number).ok_or_else(|| {
        json_error(
            StatusCode::GONE,
            &format!(
                "generation {number} has aged out of the retention window (oldest retained: {}, current: {}); \
                 raise --time-travel-generations / --time-travel-max-age to keep more history",
                state.ring.oldest_retained(),
                current.number()
            ),
        )
    })
}

/// Without the `time-travel` feature the parameter handling is compiled out:
/// every query runs against the current generation and a `generation` parameter
/// is just an ignored unknown parameter (like any other).
#[cfg(not(feature = "time-travel"))]
#[inline(always)]
// clippy: Err is axum's `Response` (the idiomatic handler error); boxing it would
// only desync this signature from the `time-travel` variant and every call-site match.
#[allow(clippy::result_large_err)]
fn resolve_pin(
    state: &AppState,
    _url_params: &HashMap<String, String>,
    _body_params: Option<&HashMap<String, String>>,
) -> Result<PinnedGen, Response> {
    Ok(state.current())
}

/// Stamps the `Sparq-Generation` response header (opt-in `time-travel` feature):
/// the generation number the response was produced against — the current
/// generation for unpinned queries (capture it as the time-travel /
/// read-your-writes token; the same generation-number concept as the
/// horizontal-scaling ADR's `shard_seq`), the pin for pinned queries, and the
/// generation containing the update for a 204 update ack.
#[cfg(feature = "time-travel")]
fn with_generation_header(mut resp: Response, number: u64) -> Response {
    resp.headers_mut().insert(
        header::HeaderName::from_static("sparq-generation"),
        header::HeaderValue::from(number),
    );
    resp
}

/// Without the `time-travel` feature there is no generation header (identity).
#[cfg(not(feature = "time-travel"))]
#[inline(always)]
fn with_generation_header(resp: Response, _number: u64) -> Response {
    resp
}

async fn sparql_endpoint(
    State(state): State<AppState>,
    method: Method,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let url_params = parse_form(raw_query.as_deref().unwrap_or(""));
    match method {
        Method::GET | Method::HEAD => {
            // Query string carries `query=` (+ optional dataset params). Per protocol, a
            // GET without a `query` parameter is a malformed request (400). A GET is always a
            // READ (the protocol has no GET update), so it is gated only under --auth-token-read.
            // [OPUS-4.8] sq-zcby.
            // [OPUS-4.8] sq-0bxp: begin an access-audit record for the GET query (always a
            // read; fingerprint the `query=` param if present).
            #[cfg(feature = "audit-log")]
            let audit = crate::audit::enabled(state.config()).then(|| {
                crate::audit::AuditRecord::begin(
                    crate::audit::AuditOp::Query,
                    bearer_token(&headers),
                    url_params.get("query").map(String::as_str),
                )
            });
            // [OPUS-4.8] sq-gos8: structured access-audit for the GET query (a read on the
            // dataset; the `query=` param is fingerprinted, never recorded raw).
            #[cfg(feature = "access-audit")]
            let aa = audit_access_begin(
                &state,
                crate::access_audit::Action::Query,
                crate::access_audit::Resource::Dataset("/sparql".to_string()),
                &headers,
                url_params.get("query").map(String::as_str),
            );
            if let Some(resp) = auth_gate(state.config(), &headers, Operation::Read) {
                #[cfg(feature = "audit-log")]
                if let Some(a) = audit {
                    a.emit(&resp);
                }
                #[cfg(feature = "access-audit")]
                audit_access_finish(&state, aa, &resp);
                return resp;
            }
            match url_params.get("query") {
                Some(q) => {
                    let pin = match resolve_pin(&state, &url_params, None) {
                        Ok(pin) => pin,
                        Err(resp) => {
                            #[cfg(feature = "audit-log")]
                            if let Some(a) = audit {
                                a.emit(&resp);
                            }
                            #[cfg(feature = "access-audit")]
                            audit_access_finish(&state, aa, &resp);
                            return resp;
                        }
                    };
                    let explain = explain_mode(&url_params, None, &headers);
                    let resp =
                        run_query(&state, q, &headers, method == Method::HEAD, explain, pin).await;
                    #[cfg(feature = "audit-log")]
                    if let Some(a) = audit {
                        a.emit(&resp);
                    }
                    #[cfg(feature = "access-audit")]
                    audit_access_finish(&state, aa, &resp);
                    resp
                }
                // [OPUS-4.8] sq-d3d8: per SPARQL Protocol §2.1.2, a GET with no `query` may
                // serve the endpoint's Service Description (OPT-IN — only when the
                // `federation-descriptors` feature is compiled in AND the config flag is set).
                // Otherwise the historical 400.
                #[cfg(feature = "federation-descriptors")]
                None if method != Method::HEAD => {
                    match service_description_response(&state, &headers) {
                        Some(resp) => resp,
                        None => bad_request("missing 'query' parameter"),
                    }
                }
                None => bad_request("missing 'query' parameter"),
            }
        }
        Method::POST => handle_post(&state, &headers, &body, &url_params).await,
        _ => method_not_allowed(&[Method::GET, Method::HEAD, Method::POST]),
    }
}

async fn handle_post(
    state: &AppState,
    headers: &HeaderMap,
    body: &Bytes,
    url_params: &HashMap<String, String>,
) -> Response {
    let ct = content_type(headers);
    if ct.starts_with(SPARQL_QUERY_CT) {
        // POST directly — body IS the SPARQL query. [OPUS-4.8] sq-zcby: the auth gate keys on
        // whether the body MUTATES, not on the route — an UPDATE smuggled through the query
        // Content-Type is gated as a write before the query handler ever sees it.
        let s = match std::str::from_utf8(body) {
            Ok(s) => s,
            Err(_) => return bad_request("request body is not valid UTF-8"),
        };
        let op = if payload_mutates(s) {
            Operation::Write
        } else {
            Operation::Read
        };
        // [OPUS-4.8] sq-0bxp: audit the direct-POST query/update (op keys on whether it mutates).
        #[cfg(feature = "audit-log")]
        let audit = crate::audit::enabled(state.config()).then(|| {
            crate::audit::AuditRecord::begin(
                crate::audit::AuditOp::from_sparql(op),
                bearer_token(headers),
                Some(s),
            )
        });
        // [OPUS-4.8] sq-gos8: structured access-audit — the plain SPARQL surface names no
        // per-graph resource at the request boundary, so the resource is the dataset (`/sparql`);
        // the query body is fingerprinted, NEVER recorded raw (the privacy boundary).
        #[cfg(feature = "access-audit")]
        let aa = audit_access_begin(
            state,
            sparql_action(op),
            crate::access_audit::Resource::Dataset("/sparql".to_string()),
            headers,
            Some(s),
        );
        if let Some(resp) = auth_gate(state.config(), headers, op) {
            #[cfg(feature = "audit-log")]
            if let Some(a) = audit {
                a.emit(&resp);
            }
            #[cfg(feature = "access-audit")]
            audit_access_finish(state, aa, &resp);
            return resp;
        }
        // NB: a write smuggled through the query Content-Type still goes to `run_query` (the
        // historical behaviour — it parses as a non-query and returns the existing error); the
        // audit op already records it as an `update` for the access trail.
        let pin = match resolve_pin(state, url_params, None) {
            Ok(pin) => pin,
            Err(resp) => {
                #[cfg(feature = "audit-log")]
                if let Some(a) = audit {
                    a.emit(&resp);
                }
                #[cfg(feature = "access-audit")]
                audit_access_finish(state, aa, &resp);
                return resp;
            }
        };
        let explain = explain_mode(url_params, None, headers);
        let resp = run_query(state, s, headers, false, explain, pin).await;
        #[cfg(feature = "audit-log")]
        if let Some(a) = audit {
            a.emit(&resp);
        }
        #[cfg(feature = "access-audit")]
        audit_access_finish(state, aa, &resp);
        resp
    } else if ct.starts_with(FORM_CT) {
        // POST url-encoded — `query=` (read) or `update=` (write) in the body. [OPUS-4.8]
        // sq-zcby: classify on the payload for the ambiguous `query=` path; an `update=` form
        // is ALWAYS a write (see below).
        let s = match std::str::from_utf8(body) {
            Ok(s) => s,
            Err(_) => return bad_request("request body is not valid UTF-8"),
        };
        let params = parse_form(s);
        // [OPUS-4.8] sq-zcby (Copilot PR#71 SECURITY fix): the `update=` form field IS an
        // update operation BY DEFINITION (SPARQL 1.1 Protocol §2.2), so it is a WRITE
        // UNCONDITIONALLY — regardless of whether its value happens to parse as a read-only
        // query (e.g. `update=SELECT…`). Classifying it on its payload (the old
        // `payload_mutates(u)`) let such a request slip past the write gate: the auth-bypass
        // this fixes. Only the ambiguous `query=`/generic path falls through to content
        // inspection — a `query=` whose value parses as an UPDATE is still gated as a write
        // (an UPDATE smuggled through the query parameter).
        let op = if params.contains_key("update") {
            Operation::Write
        } else {
            match params.get("query") {
                Some(q) if payload_mutates(q) => Operation::Write,
                Some(_) => Operation::Read,
                // Neither parameter present: not a write, let the query handler return its 400.
                None => Operation::Read,
            }
        };
        // [OPUS-4.8] sq-0bxp: audit the url-encoded form request (fingerprint the update= or
        // query= payload it carries).
        #[cfg(feature = "audit-log")]
        let audit = crate::audit::enabled(state.config()).then(|| {
            crate::audit::AuditRecord::begin(
                crate::audit::AuditOp::from_sparql(op),
                bearer_token(headers),
                params
                    .get("update")
                    .or_else(|| params.get("query"))
                    .map(String::as_str),
            )
        });
        // [OPUS-4.8] sq-gos8: structured access-audit for the url-encoded form request.
        #[cfg(feature = "access-audit")]
        let aa = audit_access_begin(
            state,
            sparql_action(op),
            crate::access_audit::Resource::Dataset("/sparql".to_string()),
            headers,
            params
                .get("update")
                .or_else(|| params.get("query"))
                .map(String::as_str),
        );
        if let Some(resp) = auth_gate(state.config(), headers, op) {
            #[cfg(feature = "audit-log")]
            if let Some(a) = audit {
                a.emit(&resp);
            }
            #[cfg(feature = "access-audit")]
            audit_access_finish(state, aa, &resp);
            return resp;
        }
        // The SPARQL 1.1 Protocol url-encoded UPDATE operation (`update=` form) submits through
        // the same sequenced writer as `application/sparql-update`.
        if let Some(u) = params.get("update") {
            #[cfg(feature = "time-travel")]
            if url_params.contains_key("generation") {
                let resp = bad_request(
                    "the 'generation' parameter pins queries to a retained generation; \
                     updates always apply to the current generation",
                );
                #[cfg(feature = "audit-log")]
                if let Some(a) = audit {
                    a.emit(&resp);
                }
                #[cfg(feature = "access-audit")]
                audit_access_finish(state, aa, &resp);
                return resp;
            }
            let resp = run_update(state, u.clone()).await;
            #[cfg(feature = "audit-log")]
            if let Some(a) = audit {
                a.emit(&resp);
            }
            #[cfg(feature = "access-audit")]
            audit_access_finish(state, aa, &resp);
            return resp;
        }
        let resp = match params.get("query") {
            Some(q) => {
                let pin = match resolve_pin(state, url_params, Some(&params)) {
                    Ok(pin) => pin,
                    Err(resp) => {
                        #[cfg(feature = "audit-log")]
                        if let Some(a) = audit {
                            a.emit(&resp);
                        }
                        #[cfg(feature = "access-audit")]
                        audit_access_finish(state, aa, &resp);
                        return resp;
                    }
                };
                let explain = explain_mode(url_params, Some(&params), headers);
                run_query(state, q, headers, false, explain, pin).await
            }
            None => bad_request("missing 'query' or 'update' parameter in url-encoded body"),
        };
        #[cfg(feature = "audit-log")]
        if let Some(a) = audit {
            a.emit(&resp);
        }
        #[cfg(feature = "access-audit")]
        audit_access_finish(state, aa, &resp);
        resp
    } else if ct.starts_with("application/sparql-update") {
        // SPARQL 1.1 Protocol — update operation (T11b). Body IS the update; success → 204.
        // [OPUS-4.8] sq-zcby: an UPDATE is always a write — gate it before doing any work.
        // [OPUS-4.8] sq-0bxp: audit the update (fingerprint the body if it is valid UTF-8).
        #[cfg(feature = "audit-log")]
        let audit = crate::audit::enabled(state.config()).then(|| {
            crate::audit::AuditRecord::begin(
                crate::audit::AuditOp::Update,
                bearer_token(headers),
                std::str::from_utf8(body).ok(),
            )
        });
        // [OPUS-4.8] sq-gos8: structured access-audit for the SPARQL UPDATE body.
        #[cfg(feature = "access-audit")]
        let aa = audit_access_begin(
            state,
            crate::access_audit::Action::Update,
            crate::access_audit::Resource::Dataset("/sparql".to_string()),
            headers,
            std::str::from_utf8(body).ok(),
        );
        if let Some(resp) = auth_gate(state.config(), headers, Operation::Write) {
            #[cfg(feature = "audit-log")]
            if let Some(a) = audit {
                a.emit(&resp);
            }
            #[cfg(feature = "access-audit")]
            audit_access_finish(state, aa, &resp);
            return resp;
        }
        // `apply_update` blocks for the writer's group-commit ack (window + batch
        // application, which includes an O(graph) fork), so it runs off the async workers.
        // Time travel is a READ concept: an update can only apply to the current
        // generation, so a `generation` pin on an update is an honest refusal, not
        // a silent ignore.
        #[cfg(feature = "time-travel")]
        if url_params.contains_key("generation") {
            let resp = bad_request(
                "the 'generation' parameter pins queries to a retained generation; \
                 updates always apply to the current generation",
            );
            #[cfg(feature = "audit-log")]
            if let Some(a) = audit {
                a.emit(&resp);
            }
            #[cfg(feature = "access-audit")]
            audit_access_finish(state, aa, &resp);
            return resp;
        }
        let resp = match std::str::from_utf8(body) {
            Ok(u) => run_update(state, u.to_string()).await,
            Err(_) => bad_request("request body is not valid UTF-8"),
        };
        #[cfg(feature = "audit-log")]
        if let Some(a) = audit {
            a.emit(&resp);
        }
        #[cfg(feature = "access-audit")]
        audit_access_finish(state, aa, &resp);
        resp
    } else {
        // Unsupported media type for the POST query operation.
        json_error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "POST requires Content-Type 'application/sparql-query' or 'application/x-www-form-urlencoded'",
        )
    }
}

/// Applies a SPARQL Update string through the sequenced writer off the async workers (it
/// blocks for the group-commit ack), mapping the outcome onto 204/400/503/500. Shared by the
/// `application/sparql-update` body path and the url-encoded `update=` form path (T11b /
/// [OPUS-4.8] sq-zcby). The caller is responsible for the auth gate (so the gate runs before
/// any work) and any `generation`-pin refusal.
///
/// [OPUS-4.8] sq-ebii: both update entry points inherit the query-timeout cap here — the SAME
/// wall-clock hard cap (`timeout + TIMEOUT_GRACE`) the read paths use, applied via
/// [`await_update_worker`]. The cooperative `QueryBudget` ALSO reaches the update itself
/// (`ServerApplier::apply` runs it via `update_in_place_with_budget`), so a `DELETE/INSERT …
/// WHERE` aborts at the deadline / row cap mid-evaluation; this wall-clock await cap is the
/// BACKSTOP for an uninstrumented stretch, not the only stop. See [`await_update_worker`] for
/// the remaining caveats (coarse cooperative checks; single sequenced writer; the writer
/// finishes its next budget check after the HTTP side has already answered 503).
/// [OPUS-4.8] (sq-x32t) `POST /admin/compact` — WAL COMPACTION / VACUUM for ERASURE-
/// COMPLETENESS (epic sq-toze.33). A logical SPARQL `DELETE` / `DROP GRAPH` retracts data from
/// the live view but leaves the superseded bytes in earlier `--persist` WAL segments until a
/// compaction folds the live state into a fresh base. This operator-invokable endpoint runs that
/// compaction on demand, so erased data is PHYSICALLY gone from the on-disk store (the manual
/// quiesce→export→reseed purge in `compliance/privacy/retention-erasure-runbook.md` §7a,
/// automated as one atomic, crash-safe operation).
///
/// - **Gated** behind the WRITE auth token (the existing admin gate) — it mutates the durable
///   store. POST-only (a GET could not mutate; the route is registered POST-only so other verbs
///   get a 405).
/// - **In-memory server** (no `--persist`): 409 Conflict — there is no on-disk history to purge
///   (erasure is already immediate: dropped generations free by `Arc` drop, no WAL survives), so
///   a no-op success would be misleading. The body explains the precondition.
/// - **Persistent server**: runs [`AppState::compact`] on the blocking pool (it blocks for the
///   compaction, which runs on the writer thread between batches). 200 on success; 503 if the
///   durable rewrite hit a transient I/O error (retryable — the writer stays alive); 500 if the
///   worker panicked.
///
/// Physical-erasure caveat (documented honestly): compaction guarantees the LIVE store no longer
/// references the erased triples and the new on-disk segments do not contain them; it cannot, by
/// itself, scrub bytes already copied off-box (filesystem snapshots, block-level COW history,
/// external backups). Those are out of scope for the engine and must be handled by the storage /
/// backup tier per the retention-erasure runbook.
async fn admin_compact(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(resp) = auth_gate(state.config(), &headers, Operation::Write) {
        return resp;
    }
    if state.config().persist_dir.is_none() {
        return json_error(
            StatusCode::CONFLICT,
            "compaction requires durable persistence; this server is in-memory (no --persist dir). \
             In-memory erasure is already immediate (no on-disk WAL history to purge).",
        );
    }
    let st = state.clone();
    let task = tokio::task::spawn_blocking(move || st.compact());
    match task.await {
        Ok(Ok(())) => text_response(
            StatusCode::OK,
            "text/plain; charset=utf-8",
            "compaction complete\n".to_string(),
            false,
        ),
        // A compaction failure (e.g. durable-write I/O error) is retryable: the writer thread
        // stays alive and reads keep flowing from the last published snapshot.
        // [OPUS-4.8] (sq-kfel, ASVS-G3) `e` is the durable-rewrite error string, which carries
        // the server's `--persist` filesystem path (an I/O error embeds the absolute path).
        // Although this route is write/admin-gated, withhold the path from the response body —
        // the operator gets the full detail in the server log.
        Ok(Err(e)) => sanitized_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "compaction",
            "compaction failed (retryable)",
            &e,
        ),
        Err(_) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "compaction worker panicked",
        ),
    }
}

async fn run_update(state: &AppState, update: String) -> Response {
    let st = state.clone();
    let task = tokio::task::spawn_blocking(move || st.apply_update(&update));
    match await_update_worker(task, &state.config).await {
        UpdateOutcome::Ok(number) => {
            state.metrics().inc_updates();
            // The 204 carries the generation containing the update (the read-your-writes
            // token) under the time-travel feature.
            with_generation_header(StatusCode::NO_CONTENT.into_response(), number)
        }
        UpdateOutcome::Rejected(e) => update_rejection_response(&e, &state.config),
        UpdateOutcome::TimedOut => timeout_response(&state.config),
        UpdateOutcome::Panicked => {
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "update worker panicked")
        }
    }
}

/// Executes a query string against the request's pinned generation and renders the
/// negotiated representation, stamped with the `Sparq-Generation` header under the
/// `time-travel` feature.
///
/// The engine call is synchronous + CPU-bound, so it runs on the blocking pool under a
/// cooperative [`QueryBudget`] (deadline + SELECT row cap). The await is additionally
/// hard-capped at `timeout + TIMEOUT_GRACE` so a worker stuck in an uninstrumented
/// stretch still gets its 503 on time (the worker itself stops at its next budget check).
async fn run_query(
    state: &AppState,
    sparql: &str,
    headers: &HeaderMap,
    head_only: bool,
    explain: ExplainMode,
    gen: PinnedGen,
) -> Response {
    let number = gen.number();
    let resp = run_query_pinned(state, sparql, headers, head_only, explain, gen).await;
    with_generation_header(resp, number)
}

async fn run_query_pinned(
    state: &AppState,
    sparql: &str,
    headers: &HeaderMap,
    head_only: bool,
    explain: ExplainMode,
    gen: PinnedGen,
) -> Response {
    let prepared = match prepare(sparql) {
        Ok(p) => p,
        // [OPUS-4.8] (sq-cz89/sq-j9zs) The parser echoes the offending query token verbatim;
        // withhold it from the body (it is caller input, but an info-leak contract regardless)
        // — the operator gets the full parse error in the server log.
        Err(PrepareError::Malformed(msg)) => {
            return sanitized_error(
                StatusCode::BAD_REQUEST,
                "query-parse",
                "malformed query",
                &msg,
            )
        }
    };
    // The generation was pinned ONCE per request (the caller's `resolve_pin`: the
    // current generation, or — under the `time-travel` feature — the requested
    // retained one); every evaluation below (and the streamed JSON body) reads this
    // snapshot, so the response is consistent with its pin even while the writer
    // publishes new generations concurrently.
    // T22 EXPLAIN: answer with the plan text instead of (plan-only) / alongside
    // executing (analyze). Plan-only is a planning dry run — no budget needed —
    // but both run on the blocking pool and under the worker timeout cap anyway;
    // analyze executes, so it gets the standard per-request budget.
    if explain != ExplainMode::Off {
        let config = state.config.clone();
        let cfg = config.clone();
        let q = sparql.to_string();
        let analyze = explain == ExplainMode::Analyze;
        let budget = make_budget(&config, true);
        let task = tokio::task::spawn_blocking(move || {
            let graph = gen.snapshot();
            // [OPUS-4.8] sq-4w18: EXPLAIN ANALYZE executes (can hit SERVICE), so it runs
            // under the egress allowlist policy like a normal query; plan-only is a dry
            // run but is wrapped identically for uniformity (it never dials).
            let r = with_engine_scope(&cfg, || {
                if analyze {
                    sparq_engine::explain_analyze_with_budget(graph, &q, &budget)
                } else {
                    sparq_engine::explain(graph, &q)
                }
            });
            match r {
                Ok(text) => {
                    text_response(StatusCode::OK, "text/plain; charset=utf-8", text, head_only)
                }
                // ANALYZE of a non-SELECT/ASK form is a client error, not a server one.
                Err(e) if e.contains("EXPLAIN ANALYZE supports") => bad_request(&e),
                // EXPLAIN ANALYZE used `make_budget(_, true)` → max_results applied.
                Err(e) => engine_error_response(&e, &cfg, true),
            }
        });
        return await_worker(task, &config).await;
    }
    let accept = headers.get(header::ACCEPT).and_then(|v| v.to_str().ok());
    let fmt = negotiate(accept);
    let config = state.config.clone();

    match prepared.form {
        QueryForm::Select | QueryForm::Ask => {
            let is_ask = prepared.form == QueryForm::Ask;
            let select = prepared.runnable;
            let budget = make_budget(&config, !is_ask);
            let cfg = config.clone();
            let task = tokio::task::spawn_blocking(move || {
                with_engine_scope(&cfg, || {
                    if is_ask {
                        match sparq_engine::ask_with_budget(gen.snapshot(), &select, &budget) {
                            Ok(value) => {
                                let (body, ct) = match fmt {
                                    Format::Xml => {
                                        (results::ask_to_xml(value), fmt.ask_content_type())
                                    }
                                    _ => (results::ask_to_json(value), fmt.ask_content_type()),
                                };
                                text_response(StatusCode::OK, ct, body, head_only)
                            }
                            // ASK used `make_budget(_, false)` → max_results did NOT apply.
                            Err(e) => engine_error_response(&e, &cfg, false),
                        }
                    } else {
                        render_select(&gen, &select, fmt, head_only, &budget, &cfg)
                    }
                })
            });
            await_worker(task, &config).await
        }
        // CONSTRUCT / DESCRIBE (T16): an RDF graph result, negotiated between
        // `application/n-triples` (default) and `text/turtle` (the N-Triples body is a
        // syntactic subset of Turtle). DESCRIBE returns concise bounded descriptions.
        // The engine's row budget applies to the WHERE-pattern solutions, the deadline
        // to the whole evaluation — same guard semantics as SELECT.
        QueryForm::Construct | QueryForm::Describe => {
            let query = prepared.runnable;
            let gfmt = negotiate_graph(accept);
            let budget = make_budget(&config, true);
            let cfg = config.clone();
            let task = tokio::task::spawn_blocking(move || {
                // [OPUS-4.8] sq-rt6v: produce the triple list once, then serialise it in the
                // negotiated graph syntax — N-Triples (default), prefix-compacting Turtle, or
                // RDF/XML — rather than always emitting N-Triples.
                match with_engine_scope(&cfg, || {
                    sparq_engine::construct_or_describe_with_budget(gen.snapshot(), &query, &budget)
                }) {
                    Ok(triples) => {
                        let body = serialise_graph_triples(&triples, gfmt);
                        text_response(StatusCode::OK, gfmt.content_type(), body, head_only)
                    }
                    // CONSTRUCT/DESCRIBE used `make_budget(_, true)` → max_results applied.
                    Err(e) => engine_error_response(&e, &cfg, true),
                }
            });
            await_worker(task, &config).await
        }
    }
}

/// The per-request engine budget: deadline from the configured timeout; the row cap from
/// the memory cap (`--max-query-rows`, applied on EVERY form) AND — when `apply_max_results`
/// is set — `--max-results`, whichever is tighter.
///
/// [OPUS-4.8] (sq-ebii) `--max-query-rows` is the coarse memory cap: it bounds the
/// working-set cardinality of any materialised intermediate/final result on all forms
/// (SELECT/ASK/CONSTRUCT/DESCRIBE/GSP-read), so a join blow-up aborts (413) instead of
/// OOMing. `--max-results` is the narrower result/solution cap: callers pass
/// `apply_max_results = true` on SELECT (the final projection), CONSTRUCT/DESCRIBE (their
/// WHERE-pattern solution count) and EXPLAIN ANALYZE; `false` on ASK and GSP-read (which
/// have no projection to cap). Both map onto the single engine `max_rows` budget, so the
/// effective ceiling on a path where both apply is their min.
pub(crate) fn make_budget(config: &ServerConfig, apply_max_results: bool) -> QueryBudget {
    let results_cap = if apply_max_results {
        config.max_results
    } else {
        None
    };
    QueryBudget {
        deadline: config.query_timeout.map(|t| std::time::Instant::now() + t),
        max_rows: tighter(config.max_query_rows, results_cap),
        // [OPUS-4.8] (sq-s5is) byte-accounted cap applies on every form (it has no
        // `--max-results` analogue — it bounds the working set, not the projection).
        max_bytes: config.max_query_bytes,
    }
}

/// [OPUS-4.8] (sq-ebii) The per-UPDATE engine budget: the coarse memory cap
/// (`--max-query-rows`) as the working-set row ceiling, and a cooperative deadline measured
/// from NOW (the moment the writer starts this update). The SELECT-projection cap
/// (`--max-results`) does NOT apply to updates — there is no result to project — but the
/// working-set memory cap and the deadline do.
///
/// [OPUS-4.8] (sq-nulp) The WHERE deadline is [`update_where_deadline`]: the TIGHTER of the
/// read [`query_timeout`](ServerConfig::query_timeout) and the optional, typically-shorter
/// [`update_where_timeout`](ServerConfig::update_where_timeout). Because updates are sequenced
/// on a single writer thread, this deadline is what BOUNDS writer-queue head-of-line blocking:
/// a slow update releases the writer within this window, so the queue behind it cannot be held
/// longer than that (cooperatively — see [`QueryBudget`]'s coarse-check caveat). With
/// `update_where_timeout` unset (the default) it is exactly `query_timeout`, unchanged.
fn update_budget(config: &ServerConfig) -> QueryBudget {
    QueryBudget {
        deadline: update_where_deadline(config).map(|t| std::time::Instant::now() + t),
        max_rows: config.max_query_rows,
        // [OPUS-4.8] (sq-s5is) the byte cap reaches the UPDATE's WHERE evaluation too.
        max_bytes: config.max_query_bytes,
    }
}

/// [OPUS-4.8] (sq-nulp) The effective WHERE-phase deadline for a SPARQL UPDATE on the writer
/// thread: the tighter of the read [`query_timeout`](ServerConfig::query_timeout) and the
/// opt-in [`update_where_timeout`](ServerConfig::update_where_timeout). `None` (no deadline)
/// only when BOTH are unset; otherwise the smaller of whichever are present. This is the knob
/// that bounds writer-queue head-of-line blocking independently of the (usually longer) read
/// timeout — see [`update_budget`].
fn update_where_deadline(config: &ServerConfig) -> Option<Duration> {
    match (config.query_timeout, config.update_where_timeout) {
        (Some(q), Some(u)) => Some(q.min(u)),
        (q, u) => q.or(u),
    }
}

/// [OPUS-4.8] (sq-ebii) The tighter (smaller) of two optional row caps: `None` is "no cap",
/// so the combined cap is `None` only when both are `None`, else the min of those present.
fn tighter(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}

/// Awaits a blocking engine worker under the hard timeout cap; maps a worker panic to a
/// 500 (CatchPanicLayer cannot see panics on the blocking pool — the JoinError carries them).
async fn await_worker(task: tokio::task::JoinHandle<Response>, config: &ServerConfig) -> Response {
    let joined = match config.query_timeout {
        Some(t) => match tokio::time::timeout(t + TIMEOUT_GRACE, task).await {
            Ok(j) => j,
            Err(_elapsed) => return timeout_response(config),
        },
        None => task.await,
    };
    match joined {
        Ok(resp) => resp,
        Err(e) if e.is_panic() => {
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "query worker panicked")
        }
        Err(_) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "query worker was cancelled",
        ),
    }
}

/// [OPUS-4.8] (sq-ebii) The result of awaiting a SPARQL Update worker under the timeout cap.
pub(crate) enum UpdateOutcome {
    /// Published; carries the generation number containing the update.
    Ok(u64),
    /// The writer rejected the update (parse / semantic error) — a client 400.
    Rejected(String),
    /// The wall-clock update cap elapsed before the writer acked — a 503.
    TimedOut,
    /// The blocking worker panicked / was cancelled — a 500.
    Panicked,
}

/// [OPUS-4.8] (sq-ebii) Awaits a SPARQL Update worker under the SAME hard wall-clock cap the
/// read paths use (`query_timeout + TIMEOUT_GRACE`), so a pathological UPDATE (e.g. a
/// `DELETE/INSERT … WHERE` whose WHERE pattern blows up) cannot pin a connection indefinitely.
///
/// **Honest scope — what this does and does NOT bound:** this is a wall-clock cap on the
/// HTTP *await*, and it is a BACKSTOP, not the only stop. The cooperative [`QueryBudget`]
/// (deadline + `max_rows`) DOES reach inside the update: `ServerApplier::apply` runs it via
/// [`sparq_engine::update_in_place_with_budget`] with [`update_budget`], which installs the
/// budget thread-locally, so a `DELETE/INSERT … WHERE` whose WHERE pattern blows up aborts
/// mid-evaluation at the row cap / deadline exactly as a budgeted `SELECT` does. The
/// remaining caveats this wall-clock cap covers: (1) the cooperative budget is checked only
/// at *coarse* sites (operator entry / per outer-loop iteration), so a single uninstrumented
/// stretch can overrun the deadline before the next check — this await then answers 503 on
/// time while the writer thread keeps running to its next budget check and discards the
/// result; (2) the non-WHERE operations (INSERT/DELETE DATA, CLEAR/DROP/CREATE/LOAD) do not
/// consult the budget (they are bounded by operand size, already capped by `--max-body-bytes`);
/// (3) updates are *sequenced* on a single writer, so a long-running update blocks the queue
/// behind it until it finishes — this *await* cap bounds the client's own wait, not the writer's
/// work. [OPUS-4.8] (sq-nulp) The writer's work — and hence that head-of-line blocking — is
/// bounded SEPARATELY by the WHERE-phase deadline ([`update_budget`] / [`update_where_deadline`]):
/// `--update-where-timeout` sets a typically-shorter cooperative deadline so a slow update
/// releases the writer within that window instead of holding it for the full (longer) read
/// timeout, cooperatively per caveat (1).
async fn await_update_worker(
    task: tokio::task::JoinHandle<Result<u64, String>>,
    config: &ServerConfig,
) -> UpdateOutcome {
    let joined = match config.query_timeout {
        Some(t) => match tokio::time::timeout(t + TIMEOUT_GRACE, task).await {
            Ok(j) => j,
            Err(_elapsed) => return UpdateOutcome::TimedOut,
        },
        None => task.await,
    };
    match joined {
        Ok(Ok(number)) => UpdateOutcome::Ok(number),
        Ok(Err(e)) => UpdateOutcome::Rejected(e),
        Err(_) => UpdateOutcome::Panicked,
    }
}

/// Runs a SELECT on the engine and serialises it in the negotiated format. JSON uses the
/// engine's direct id→JSON fast path and is **streamed**: the engine returns the body as
/// an ordered chunk sequence (concatenation byte-identical to the single-string form) and
/// the response body hands those chunks to hyper one by one — the peak never holds a
/// second whole-result copy (T16). `Content-Length` is known up front (the chunks are
/// fully evaluated before the response starts), so the response framing is identical to
/// the buffered path. The other formats go through the materialised `QueryResult`.
///
/// Takes the request's pinned generation (not a bare `&Graph`) so the streamed JSON body
/// can keep the generation pinned until the last chunk is written — the stream stays
/// snapshot-consistent with query START even if the writer publishes past it mid-response.
fn render_select(
    gen: &PinnedGen,
    select: &str,
    fmt: Format,
    head_only: bool,
    budget: &QueryBudget,
    config: &ServerConfig,
) -> Response {
    let graph = gen.snapshot();
    let ct = fmt.select_content_type();
    let body = match fmt {
        // SELECT projections fold in --max-results (`make_budget(_, true)`) → name it.
        Format::Json => match sparq_engine::query_json_chunks_with_budget(graph, select, budget) {
            Ok(chunks) => {
                return chunked_response(StatusCode::OK, ct, chunks, head_only, gen.clone())
            }
            Err(e) => return engine_error_response(&e, config, true),
        },
        _ => {
            let result = match sparq_engine::query_with_budget(graph, select, budget) {
                Ok(r) => r,
                Err(e) => return engine_error_response(&e, config, true),
            };
            match fmt {
                Format::Xml => results::select_to_xml(&result),
                Format::Csv => results::select_to_csv(&result),
                Format::Tsv => results::select_to_tsv(&result),
                Format::Json => unreachable!(),
            }
        }
    };
    text_response(StatusCode::OK, ct, body, head_only)
}

/// Maps an engine error string onto the HTTP guard semantics: budget timeout → 503,
/// budget row cap → 413 (honest refusal), anything else → 500.
///
/// `apply_max_results` MUST match the flag the path passed to [`make_budget`]: it tells this
/// function whether `--max-results` actually contributed to the `max_rows` budget on THIS
/// request, so the 413 message names the right knob.
///
/// [OPUS-4.8] (sq-ebii) Only SELECT projections (and EXPLAIN ANALYZE / CONSTRUCT / DESCRIBE,
/// which also pass `apply_max_results = true`) fold `--max-results` into the budget. ASK,
/// GSP-read and UPDATE build their budget from `--max-query-rows` ALONE
/// (`make_budget(_, false)` / `update_budget`), so on those paths `--max-results` did NOT
/// participate even when it is set and smaller. Picking the named knob from the global config
/// (the old behaviour) therefore misreported the cap on those paths — e.g. an ASK with
/// `--max-query-rows 100 --max-results 10` aborts at 100 rows but would have been reported as
/// "10 rows, --max-results". Gating the `max_results` consideration on `apply_max_results`
/// makes the message path-accurate (the 413 status itself was always correct — only the
/// human-readable knob name / row number could be wrong).
fn engine_error_response(e: &str, config: &ServerConfig, apply_max_results: bool) -> Response {
    if e.contains("query budget exceeded (timeout)") {
        return timeout_response(config);
    }
    if e.contains("query budget exceeded (max-rows)") {
        // Only the caps that actually fed THIS path's budget are eligible to be named.
        let results_cap = apply_max_results.then_some(config.max_results).flatten();
        let max = tighter(config.max_query_rows, results_cap).unwrap_or(0);
        let which = match (config.max_query_rows, results_cap) {
            (Some(q), Some(r)) if q <= r => "--max-query-rows (memory cap)",
            (Some(_), None) => "--max-query-rows (memory cap)",
            _ => "--max-results",
        };
        return json_error(
            StatusCode::PAYLOAD_TOO_LARGE,
            &format!("result exceeds the server's working-set row limit ({max} rows, {which}); narrow the query (e.g. add LIMIT) or raise the limit"),
        );
    }
    // [OPUS-4.8] (sq-s5is) the byte-accounted cap tripped — an honest 413, same class as the
    // row cap, naming the byte knob (this path applies on EVERY form, so no `--max-results`).
    if e.contains("query budget exceeded (max-bytes)") {
        let max = config.max_query_bytes.unwrap_or(0);
        return json_error(
            StatusCode::PAYLOAD_TOO_LARGE,
            &format!("result exceeds the server's working-set byte limit ({max} bytes, --max-query-bytes (memory cap)); narrow the query (e.g. project fewer variables, add LIMIT) or raise the limit"),
        );
    }
    execution_error(e)
}

/// [OPUS-4.8] (sq-ebii) Maps a writer-rejected UPDATE onto HTTP status: a cooperative budget
/// hit (the memory cap / deadline tripped inside a `DELETE/INSERT … WHERE`) is a 413 / 503,
/// exactly like a query; any other rejection (parse / semantic error) is the client's 400.
fn update_rejection_response(e: &str, config: &ServerConfig) -> Response {
    // [OPUS-4.8] (sq-vpx4) A durable-write refusal is a retryable 503, NOT a 400. The write
    // was valid but could not be made durable (transient ENOSPC/I/O on the `--persist` mirror);
    // nothing was published or acked, the server is still serving reads, and the client should
    // retry. Sniffed before the budget/parse mapping so it always wins.
    if let Some(detail) = e.strip_prefix(DURABLE_UNAVAILABLE_PREFIX) {
        // [OPUS-4.8] (sq-cz89/sq-j9zs) `detail` is the underlying I/O error, which carries the
        // server's `--persist` filesystem path (e.g. an ENOSPC on the mirror) — withhold it
        // from the client; the operator sees the path in the server log.
        return sanitized_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "durable-unavailable",
            "update not durably committed (transient durable-write error); the write was refused and NOT applied — retry",
            detail,
        );
    }
    if e.contains("query budget exceeded") {
        // Updates budget from `--max-query-rows` alone (`update_budget`, no `--max-results`),
        // so the 413 message must not consider `--max-results` → `apply_max_results = false`.
        return engine_error_response(e, config, false);
    }
    // [OPUS-4.8] (sq-cz89/sq-j9zs) A parse/semantic rejection echoes the offending UPDATE
    // token (and, for a `DELETE/INSERT … WHERE`, can quote loaded data); withhold it — the
    // full reason is in the server log.
    sanitized_error(
        StatusCode::BAD_REQUEST,
        "update-parse",
        "update failed: invalid SPARQL update",
        e,
    )
}

fn timeout_response(config: &ServerConfig) -> Response {
    let secs = config.query_timeout.map(|t| t.as_secs()).unwrap_or(0);
    json_error(
        StatusCode::SERVICE_UNAVAILABLE,
        &format!(
            "query timed out (server limit: {secs}s); simplify the query or raise --query-timeout"
        ),
    )
}

// ---------------------------------------------------------------------------
// Graph Store HTTP Protocol — READ side
// ---------------------------------------------------------------------------

/// [OPUS-4.8] (sq-gxsj) Which graph a GSP request addresses: the default graph or a
/// concrete named graph. Indirect identification yields `?default` → [`GraphRef::Default`]
/// or `?graph=<iri>` → [`GraphRef::Named`]; direct identification turns the request URI
/// into the graph's IRI ([`GraphRef::Named`]). The engine has a real named-graph store
/// (`crates/sparq-engine/src/update.rs` — "over the FULL DATASET"), so this is a faithful
/// addressing scheme, not a default-graph alias.
#[derive(Clone)]
enum GraphRef {
    Default,
    Named(String),
}

/// Indirect graph identification: `?default` or `?graph=<iri>`.
async fn graph_store_indirect(
    State(state): State<AppState>,
    method: Method,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let is_default = params.contains_key("default");
    let named = params.get("graph").cloned();
    if is_default && named.is_some() {
        return bad_request("indirect graph identification accepts exactly one of '?default' or '?graph=<uri>', not both");
    }
    let graph = match (is_default, named) {
        (true, _) => GraphRef::Default,
        (false, Some(g)) => GraphRef::Named(g),
        // POST to the GSP endpoint with no graph selector creates a fresh graph (spec
        // §5.5); other write verbs and reads still require an explicit selector.
        (false, None) if method == Method::POST => GraphRef::Named(mint_graph_iri()),
        (false, None) => {
            return bad_request(
                "indirect graph identification requires '?default' or '?graph=<uri>'",
            )
        }
    };
    graph_store(&state, &method, graph, &headers, body).await
}

/// Direct graph identification: the request URI IS the graph's resource (RFC 3986). The
/// engine stores named graphs, so the direct form addresses a real per-request graph IRI
/// reconstructed from the `Host` header and the matched path — round-trippable with the
/// indirect `?graph=<that-iri>` form.
async fn graph_store_direct(
    State(state): State<AppState>,
    method: Method,
    axum::extract::Path(path): axum::extract::Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let iri = direct_graph_iri(&headers, &path);
    graph_store(&state, &method, GraphRef::Named(iri), &headers, body).await
}

/// Reconstructs the graph IRI a direct-identification request addresses: `http://<host>/
/// graphs/<path>`, using the `Host` header (a sane default when absent). This is the
/// request URI per the GSP "direct graph identification" rule, and is exactly the IRI a
/// later indirect `?graph=<iri>` request must use to address the same graph.
fn direct_graph_iri(headers: &HeaderMap, path: &str) -> String {
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");
    format!("http://{host}/graphs/{path}")
}

/// Mints a fresh, server-allocated graph IRI for a selector-less POST (GSP §5.5: "create a
/// new graph"). UUID-free to avoid a dependency: a process-unique counter plus the nanos
/// clock is collision-free within a process and good enough for an opaque server-chosen IRI.
fn mint_graph_iri() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("urn:sparq:gsp:graph:{nanos:x}-{n:x}")
}

/// [OPUS-4.8] (sq-gxsj) Shared GSP dispatcher across direct + indirect identification.
/// READ (`GET`/`HEAD`) serialises the addressed graph; WRITE (`PUT`/`POST`/`DELETE`)
/// translates into a SPARQL Update submitted through the sequenced writer.
async fn graph_store(
    state: &AppState,
    method: &Method,
    graph: GraphRef,
    headers: &HeaderMap,
    body: Bytes,
) -> Response {
    // [OPUS-4.8] sq-zcby: the GSP write methods (PUT/POST/DELETE, and PATCH which we 405) are
    // as powerful as an UPDATE, so they are gated as writes; GET/HEAD are reads (gated only
    // under --auth-token-read). Any other method is gated as a write too (fail-closed). The
    // gate runs before any work — even the 405 for an unsupported method is behind it for a
    // write verb, so an attacker cannot probe the surface without the token.
    let op = match *method {
        Method::GET | Method::HEAD => Operation::Read,
        _ => Operation::Write,
    };
    // [OPUS-4.8] sq-0bxp: begin an access-audit record for the GSP request (the Graph-Store
    // surface has no query text to fingerprint — only the operation class + graph access).
    #[cfg(feature = "audit-log")]
    let audit = crate::audit::enabled(state.config()).then(|| {
        crate::audit::AuditRecord::begin(
            crate::audit::AuditOp::from_graph(op),
            bearer_token(headers),
            None,
        )
    });
    // [OPUS-4.8] sq-gos8: begin the RICHER structured access-audit record. The GSP surface
    // carries the resource at its highest fidelity — the named-graph IRI (or the default graph)
    // the request addresses. There is no query body to fingerprint (the body is RDF, not a
    // query); the action class distinguishes read vs write.
    #[cfg(feature = "access-audit")]
    let aa = audit_access_begin(
        state,
        match op {
            Operation::Read => crate::access_audit::Action::GraphRead,
            Operation::Write => crate::access_audit::Action::GraphWrite,
        },
        match &graph {
            GraphRef::Default => crate::access_audit::Resource::Dataset("default".to_string()),
            GraphRef::Named(iri) => crate::access_audit::Resource::NamedGraph(iri.clone()),
        },
        headers,
        None,
    );
    if let Some(resp) = auth_gate(state.config(), headers, op) {
        #[cfg(feature = "audit-log")]
        if let Some(a) = audit {
            a.emit(&resp);
        }
        #[cfg(feature = "access-audit")]
        audit_access_finish(state, aa, &resp);
        return resp;
    }
    let resp = match *method {
        Method::GET | Method::HEAD => {
            let head_only = *method == Method::HEAD;
            serialise_graph(state, graph, headers, head_only).await
        }
        Method::PUT => gsp_put(state, graph, headers, &body).await,
        Method::POST => gsp_post(state, graph, headers, &body).await,
        Method::DELETE => gsp_delete(state, graph).await,
        // PATCH is not part of the Graph Store HTTP Protocol.
        _ => method_not_allowed(&[
            Method::GET,
            Method::HEAD,
            Method::PUT,
            Method::POST,
            Method::DELETE,
        ]),
    };
    #[cfg(feature = "audit-log")]
    if let Some(a) = audit {
        a.emit(&resp);
    }
    #[cfg(feature = "access-audit")]
    audit_access_finish(state, aa, &resp);
    resp
}

// ---------------------------------------------------------------------------
// Graph Store HTTP Protocol — WRITE side (sq-gxsj) [OPUS-4.8]
// ---------------------------------------------------------------------------

/// The body media type a GSP write carries. `sparq-core` token formats parse through
/// `Graph::load_str`; RDF/XML is parsed by `oxrdfxml` ([OPUS-4.8] sq-rt6v) since the engine
/// loader has no RDF/XML token. The body carries the triples for ONE graph (the graph is
/// named by the URL, not the body), so a quad syntax (N-Quads/TriG) is parsed as triples too
/// — its graph names are folded, exactly as `Graph::load_str` does.
enum BodyFormat {
    /// A `sparq-core` `Graph::load_str` token format ("turtle"/"ntriples"/"nquads"/"trig").
    Core(&'static str),
    /// `application/rdf+xml`, parsed via `oxrdfxml`. [OPUS-4.8] sq-rt6v.
    RdfXml,
}

/// Classifies a GSP write-body `Content-Type` into the [`BodyFormat`] used to parse it.
fn rdf_format_for(content_type: &str) -> Option<BodyFormat> {
    // `content_type` is already lowercased by `content_type()`; ignore any `; charset=…`.
    let mt = content_type.split(';').next().unwrap_or("").trim();
    match mt {
        "text/turtle" | "application/x-turtle" => Some(BodyFormat::Core("turtle")),
        "application/n-triples" | "text/plain" => Some(BodyFormat::Core("ntriples")),
        "application/n-quads" => Some(BodyFormat::Core("nquads")),
        "application/trig" => Some(BodyFormat::Core("trig")),
        // [OPUS-4.8] sq-rt6v: RDF/XML request body.
        "application/rdf+xml" => Some(BodyFormat::RdfXml),
        // No explicit Content-Type: default to Turtle (a superset of N-Triples), matching
        // the read side's default emission and common GSP client behaviour.
        "" => Some(BodyFormat::Core("turtle")),
        _ => None,
    }
}

/// Parses a GSP request body into canonical N-Triples (the term syntax accepted verbatim
/// inside a SPARQL `INSERT DATA` block), validating the RDF in the process. The `sparq-core`
/// token formats reuse `Graph::load_str` (the same parsers the loader uses); RDF/XML is
/// parsed by `oxrdfxml` and serialised straight to canonical N-Triples ([OPUS-4.8] sq-rt6v).
/// A parse failure is the caller's 400. `base` resolves any relative IRIs in the body — the
/// addressed graph's IRI, so a relative reference resolves predictably.
// clippy: Err is axum's `Response` (the idiomatic handler error, as in `resolve_pin`);
// boxing it would only desync this from the rest of the handler error convention.
#[allow(clippy::result_large_err)]
fn body_to_ntriples(
    body: &Bytes,
    content_type: &str,
    base: Option<&str>,
) -> Result<String, Response> {
    let format = match rdf_format_for(content_type) {
        Some(f) => f,
        None => {
            return Err(json_error(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "GSP write body must be RDF: Content-Type 'text/turtle', 'application/n-triples', 'application/n-quads', 'application/trig' or 'application/rdf+xml'",
            ))
        }
    };
    match format {
        BodyFormat::Core(format) => {
            let text = std::str::from_utf8(body)
                .map_err(|_| bad_request("request body is not valid UTF-8"))?;
            // [OPUS-4.8] (sq-cz89/sq-j9zs) `oxttl` echoes the offending body token verbatim
            // (e.g. an invalid subject quotes the loaded term) — withhold it; the operator
            // gets the full parse error in the server log.
            let graph = Graph::load_str(text, format).map_err(|e| {
                sanitized_error(
                    StatusCode::BAD_REQUEST,
                    "rdf-body-parse",
                    "malformed RDF body",
                    &e,
                )
            })?;
            // Re-emit as canonical N-Triples via the engine scan — no private store API needed,
            // and the terms are exactly what `INSERT DATA` will re-intern.
            nt_dump(&graph, &QueryBudget::default()).map_err(|e| execution_error(&e))
        }
        // [OPUS-4.8] sq-rt6v: parse RDF/XML to triples and emit canonical N-Triples directly.
        // `oxrdf`'s Display is canonical N-Triples term syntax, exactly what `INSERT DATA`
        // re-interns, so no Graph round-trip is needed (and RDF/XML is not a loader token).
        BodyFormat::RdfXml => {
            // [OPUS-4.8] (sq-cz89/sq-j9zs) `oxrdfxml` echoes the offending body fragment — withhold it.
            let triples = crate::graph::parse_rdfxml(body, base).map_err(|e| {
                sanitized_error(
                    StatusCode::BAD_REQUEST,
                    "rdfxml-body-parse",
                    "malformed RDF/XML body",
                    &e,
                )
            })?;
            Ok(crate::graph::triples_to_ntriples(&triples))
        }
    }
}

/// [OPUS-4.8] sq-rt6v: the base IRI a GSP write body resolves relative references against —
/// the addressed named graph's IRI (a sensible, round-trippable base), or `None` for the
/// default graph (no natural base; an RDF/XML body with relative IRIs against the default
/// graph is then a parse error, which is the honest outcome).
fn base_iri(graph: &GraphRef) -> Option<&str> {
    match graph {
        GraphRef::Default => None,
        GraphRef::Named(iri) => Some(iri.as_str()),
    }
}

/// Wraps an N-Triples body in the `GRAPH <iri> { … }` block for a named graph, or returns
/// it bare for the default graph — the shape an `INSERT DATA` / `DELETE DATA` operand takes.
fn graph_data_block(graph: &GraphRef, ntriples: &str) -> String {
    match graph {
        GraphRef::Default => ntriples.to_string(),
        GraphRef::Named(iri) => format!("GRAPH <{}> {{\n{ntriples}}}\n", escape_iri(iri)),
    }
}

/// Minimal IRI escaping for safe interpolation into a SPARQL `<…>` term: the characters
/// SPARQL forbids inside an IRIREF (controls, spaces, and the delimiters `<>"{}|^`\``).
/// A graph IRI containing them would otherwise break the generated update; percent-encode
/// them so the update parses and addresses a stable IRI.
fn escape_iri(iri: &str) -> String {
    let mut out = String::with_capacity(iri.len());
    for c in iri.chars() {
        match c {
            '<' | '>' | '"' | '{' | '}' | '|' | '^' | '`' | '\\' | ' ' => {
                for b in c.to_string().bytes() {
                    out.push_str(&format!("%{b:02X}"));
                }
            }
            c if (c as u32) < 0x20 => out.push_str(&format!("%{:02X}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Runs a server-minted SPARQL Update through the sequenced writer off the async workers
/// (it blocks for the group-commit ack), mapping the outcome onto `success`/400/500 — the
/// same status discipline as the `application/sparql-update` path.
async fn apply_gsp_update(state: &AppState, update: String, success: StatusCode) -> Response {
    let st = state.clone();
    let task = tokio::task::spawn_blocking(move || st.apply_update(&update));
    // [OPUS-4.8] sq-ebii: GSP writes inherit the same UPDATE timeout cap as the
    // `application/sparql-update` path (they share the sequenced writer).
    match await_update_worker(task, &state.config).await {
        UpdateOutcome::Ok(number) => {
            state.metrics().inc_updates();
            with_generation_header(success.into_response(), number)
        }
        UpdateOutcome::Rejected(e) => {
            // [OPUS-4.8] sq-ebii: a budget hit (timeout / memory cap) inside a GSP write's
            // WHERE maps to 503/413; a genuine parse/semantic failure stays a 400.
            if e.contains("query budget exceeded") {
                update_rejection_response(&e, &state.config)
            } else {
                // [OPUS-4.8] (sq-kfel, ASVS-G3) The writer-rejection string `e` is the engine's
                // error for the SERVER-MINTED `DROP`/`INSERT DATA` update built from the request
                // body — it can quote term text drawn from the (caller-supplied) body, exactly
                // the info-leak class the rest of the surface sanitizes (sq-cz89/sq-j9zs). Route
                // it through `sanitized_error`: a generic class message to the client, the full
                // engine detail to the server log. The other UPDATE entry points already go
                // through `update_rejection_response` → `sanitized_error`; this GSP-write minted-
                // update branch was the one path that echoed `e` verbatim. Stays a 400.
                sanitized_error(
                    StatusCode::BAD_REQUEST,
                    "gsp-write",
                    "graph store write failed: invalid RDF body",
                    &e,
                )
            }
        }
        UpdateOutcome::TimedOut => timeout_response(&state.config),
        UpdateOutcome::Panicked => {
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "update worker panicked")
        }
    }
}

/// `PUT <graph>` — REPLACE the graph's contents with the request body (GSP §4.2). Maps to
/// `DROP SILENT GRAPH <g>; INSERT DATA { GRAPH <g> { … } }` (or `CLEAR DEFAULT` + `INSERT
/// DATA` for the default graph) so the replace is one atomic, group-committed generation.
/// 201 when the graph did not exist (created), 204 when it replaced an existing one.
async fn gsp_put(state: &AppState, graph: GraphRef, headers: &HeaderMap, body: &Bytes) -> Response {
    // [OPUS-4.8] sq-ebii: inflate a `Content-Encoding: gzip` body under the
    // decompression-ratio cap (zip-bomb guard) before parsing it as RDF.
    let body = match decode_request_body(body, headers, state.config()) {
        Ok(b) => b,
        Err(e) => return e.into_response(),
    };
    let ntriples = match body_to_ntriples(&body, &content_type(headers), base_iri(&graph)) {
        Ok(nt) => nt,
        Err(resp) => return resp,
    };
    // Created (201) vs replaced (204) is decided by pre-existence, sampled from the current
    // generation. A racing writer cannot break this: the status is advisory and the write
    // itself is atomic on the sequenced writer regardless of the sampled flag.
    let existed = graph_exists(state, &graph);
    let clear = match &graph {
        GraphRef::Default => "CLEAR DEFAULT".to_string(),
        GraphRef::Named(iri) => format!("DROP SILENT GRAPH <{}>", escape_iri(iri)),
    };
    // An empty body means "the graph is now empty" — the clear alone achieves that; emitting
    // an empty `INSERT DATA { }` would be a needless (and possibly unparsable) no-op clause.
    let update = if ntriples.is_empty() {
        clear
    } else {
        let block = graph_data_block(&graph, &ntriples);
        format!("{clear} ;\nINSERT DATA {{ {block} }}")
    };
    let success = if existed {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::CREATED
    };
    apply_gsp_update(state, update, success).await
}

/// `POST <graph>` — MERGE (additive) the request body into the graph (GSP §5). Maps to
/// `INSERT DATA { GRAPH <g> { … } }`. 201 when the merge created the graph (a selector-less
/// POST, or a POST to an absent named graph), 204 when it added to an existing graph.
async fn gsp_post(
    state: &AppState,
    graph: GraphRef,
    headers: &HeaderMap,
    body: &Bytes,
) -> Response {
    // [OPUS-4.8] sq-ebii: inflate a gzip body under the decompression-ratio cap first.
    let body = match decode_request_body(body, headers, state.config()) {
        Ok(b) => b,
        Err(e) => return e.into_response(),
    };
    let ntriples = match body_to_ntriples(&body, &content_type(headers), base_iri(&graph)) {
        Ok(nt) => nt,
        Err(resp) => return resp,
    };
    let existed = graph_exists(state, &graph);
    // An empty merge body adds nothing. For an existing graph that is a 204 no-op; for an
    // absent named graph the spec wants the graph created — `CREATE SILENT GRAPH <g>` makes
    // the empty graph so a subsequent read addresses it (201).
    let update = if ntriples.is_empty() {
        match &graph {
            GraphRef::Default => {
                return with_generation_header(
                    StatusCode::NO_CONTENT.into_response(),
                    state.current().number(),
                )
            }
            GraphRef::Named(_) if existed => {
                return with_generation_header(
                    StatusCode::NO_CONTENT.into_response(),
                    state.current().number(),
                )
            }
            GraphRef::Named(iri) => format!("CREATE SILENT GRAPH <{}>", escape_iri(iri)),
        }
    } else {
        let block = graph_data_block(&graph, &ntriples);
        format!("INSERT DATA {{ {block} }}")
    };
    let success = if existed {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::CREATED
    };
    apply_gsp_update(state, update, success).await
}

/// `DELETE <graph>` — DROP the graph (GSP §6). 204 on success; 404 when the named graph is
/// absent (per the spec's "graph does not exist" semantics). The default graph always
/// exists, so `DELETE ?default` empties it and is always 204. Maps to `DROP GRAPH <g>` /
/// `CLEAR DEFAULT`.
async fn gsp_delete(state: &AppState, graph: GraphRef) -> Response {
    match &graph {
        GraphRef::Default => {
            apply_gsp_update(state, "CLEAR DEFAULT".to_string(), StatusCode::NO_CONTENT).await
        }
        GraphRef::Named(iri) => {
            if !graph_exists(state, &graph) {
                return json_error(
                    StatusCode::NOT_FOUND,
                    &format!("graph <{iri}> does not exist"),
                );
            }
            let update = format!("DROP GRAPH <{}>", escape_iri(iri));
            apply_gsp_update(state, update, StatusCode::NO_CONTENT).await
        }
    }
}

/// Whether the addressed graph currently exists / is non-empty in the current generation.
/// For a named graph this is `ASK { GRAPH <g> { ?s ?p ?o } }`; for the default graph,
/// `ASK { ?s ?p ?o }`. Note the engine has no separate "empty named graph exists" bit
/// outside of an in-flight update, so an existing-but-empty named graph reads as absent —
/// which only affects the advisory created-vs-replaced status code, never write atomicity.
fn graph_exists(state: &AppState, graph: &GraphRef) -> bool {
    let ask = match graph {
        GraphRef::Default => "ASK { ?s ?p ?o }".to_string(),
        GraphRef::Named(iri) => format!("ASK {{ GRAPH <{}> {{ ?s ?p ?o }} }}", escape_iri(iri)),
    };
    let gen = state.current();
    matches!(sparq_engine::ask(gen.snapshot(), &ask), Ok(true))
}

// ---------------------------------------------------------------------------
// Graph Store HTTP Protocol — READ side
// ---------------------------------------------------------------------------

/// Serialises the addressed graph in the negotiated RDF syntax: `application/n-triples`
/// (default), prefix-compacting `text/turtle`, or `application/rdf+xml` ([OPUS-4.8] sq-rt6v
/// — `text/turtle` is now a real compact Turtle document, not N-Triples-as-Turtle, and
/// RDF/XML is newly offered). CPU-bound (a full-graph dump), so it runs on the blocking pool
/// under the request timeout (no row cap: a dump is inherently graph-sized). A named graph
/// that does not exist serialises as the empty graph (200, empty body) per GSP read.
async fn serialise_graph(
    state: &AppState,
    graph: GraphRef,
    headers: &HeaderMap,
    head_only: bool,
) -> Response {
    let accept = headers.get(header::ACCEPT).and_then(|v| v.to_str().ok());
    let gfmt = negotiate_graph(accept);
    let ct = gfmt.content_type().to_string();
    let config = state.config.clone();
    let budget = make_budget(&config, false);
    // Pinned for the whole dump: the serialisation is consistent with request start.
    let gen = state.current();
    let cfg = config.clone();
    let task = tokio::task::spawn_blocking(move || {
        match graph_dump_triples(gen.snapshot(), &graph, &budget) {
            Ok(triples) => text_response(
                StatusCode::OK,
                &ct,
                serialise_graph_triples(&triples, gfmt),
                head_only,
            ),
            // GSP-read used `make_budget(_, false)` → max_results did NOT apply.
            Err(e) => engine_error_response(&e, &cfg, false),
        }
    });
    await_worker(task, &config).await
}

/// [OPUS-4.8] sq-rt6v: serialises an RDF graph (triple list) in the negotiated [`GraphFormat`]
/// — the single dispatch shared by the CONSTRUCT/DESCRIBE and GSP-read paths. N-Triples is the
/// canonical line form; Turtle compacts the [`crate::graph::COMMON_PREFIXES`]; RDF/XML is the
/// `application/rdf+xml` document. The writers are guaranteed to emit well-formed output.
fn serialise_graph_triples(triples: &[oxrdf::Triple], gfmt: GraphFormat) -> String {
    match gfmt {
        GraphFormat::NTriples => crate::graph::triples_to_ntriples(triples),
        GraphFormat::Turtle => crate::graph::triples_to_turtle(triples),
        GraphFormat::RdfXml => crate::graph::triples_to_rdfxml(triples),
    }
}

/// Dumps the addressed graph as a triple list by reusing the engine's SELECT path and the
/// materialised terms — `?s ?p ?o` for the default graph, `GRAPH <g> { ?s ?p ?o }` for a
/// named graph — so no private store API is needed. [OPUS-4.8] sq-rt6v: returns `Vec<Triple>`
/// (was N-Triples text) so the caller can serialise in any RDF syntax.
fn graph_dump_triples(
    graph: &Graph,
    target: &GraphRef,
    budget: &QueryBudget,
) -> Result<Vec<oxrdf::Triple>, String> {
    let select = match target {
        GraphRef::Default => "SELECT ?s ?p ?o WHERE { ?s ?p ?o }".to_string(),
        GraphRef::Named(iri) => format!(
            "SELECT ?s ?p ?o WHERE {{ GRAPH <{}> {{ ?s ?p ?o }} }}",
            escape_iri(iri)
        ),
    };
    let r = sparq_engine::query_with_budget(graph, &select, budget)?;
    let mut triples = Vec::with_capacity(r.rows.len());
    for row in &r.rows {
        let (Some(s), Some(p), Some(o)) = (&row[0], &row[1], &row[2]) else {
            continue;
        };
        // A triple from a real RDF graph always has a NamedNode/BlankNode subject and a
        // NamedNode predicate; a row that somehow violates that (it cannot, for `?s ?p ?o`
        // over a graph) is skipped rather than panicked on.
        if let Some(t) = row_to_triple(s, p, o) {
            triples.push(t);
        }
    }
    Ok(triples)
}

/// [OPUS-4.8] sq-rt6v: builds an [`oxrdf::Triple`] from an `?s ?p ?o` solution row, or `None`
/// if the slots are not a legal triple shape (subject must be IRI/bnode, predicate an IRI).
fn row_to_triple(s: &oxrdf::Term, p: &oxrdf::Term, o: &oxrdf::Term) -> Option<oxrdf::Triple> {
    use oxrdf::{NamedOrBlankNode, Term};
    let subject = match s {
        Term::NamedNode(n) => NamedOrBlankNode::NamedNode(n.clone()),
        Term::BlankNode(b) => NamedOrBlankNode::BlankNode(b.clone()),
        _ => return None,
    };
    let Term::NamedNode(predicate) = p else {
        return None;
    };
    Some(oxrdf::Triple::new(subject, predicate.clone(), o.clone()))
}

/// Dumps the whole default graph as N-Triples by reusing the engine's SELECT path
/// (`?s ?p ?o`) and the materialised terms — no private store API needed.
fn nt_dump(graph: &Graph, budget: &QueryBudget) -> Result<String, String> {
    nt_dump_select(graph, "SELECT ?s ?p ?o WHERE { ?s ?p ?o }", budget)
}

/// Shared `?s ?p ?o` → N-Triples projection for the GSP read / body-canonicalisation paths.
fn nt_dump_select(graph: &Graph, select: &str, budget: &QueryBudget) -> Result<String, String> {
    let r = sparq_engine::query_with_budget(graph, select, budget)?;
    let mut out = String::with_capacity(r.rows.len() * 64);
    for row in &r.rows {
        let (Some(s), Some(p), Some(o)) = (&row[0], &row[1], &row[2]) else {
            continue;
        };
        nt_term(&mut out, s);
        out.push(' ');
        nt_term(&mut out, p);
        out.push(' ');
        nt_term(&mut out, o);
        out.push_str(" .\n");
    }
    Ok(out)
}

fn nt_term(out: &mut String, t: &oxrdf::Term) {
    // oxrdf's Display for Term already produces canonical N-Triples term syntax.
    use std::fmt::Write;
    let _ = write!(out, "{t}");
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parses an `application/x-www-form-urlencoded` string into a map (last value wins).
fn parse_form(s: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in s.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        map.insert(form_decode(k), form_decode(v));
    }
    map
}

/// Decodes a single `application/x-www-form-urlencoded` component (`+` → space, `%XX`).
fn form_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hi = hex_val(bytes[i + 1]);
                let lo = hex_val(bytes[i + 2]);
                match (hi, lo) {
                    (Some(h), Some(l)) => {
                        out.push((h << 4) | l);
                        i += 3;
                    }
                    _ => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn content_type(headers: &HeaderMap) -> String {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase()
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-ebii — decompression-ratio cap (zip-bomb guard)
// ---------------------------------------------------------------------------

/// [OPUS-4.8] (sq-ebii) The error of a bounded body decode, mapped to its HTTP status.
#[derive(Debug)]
enum DecodeError {
    /// `Content-Encoding` names a codec we do not decode (only `gzip`/`x-gzip` and
    /// `identity` are supported) → 415.
    Unsupported(String),
    /// The decompressed image crossed the ratio/absolute ceiling → 413 (zip-bomb guard).
    /// The message is server-constructed (sizes/knob names only — no caller content).
    TooLarge(String),
    /// The gzip stream was malformed → 400. The payload is the underlying decoder error,
    /// which can quote bytes of the caller's compressed body — it is the WITHHELD detail
    /// ([OPUS-4.8] sq-cz89/sq-j9zs), routed to the server log, never the response body.
    Malformed(String),
}

impl DecodeError {
    fn into_response(self) -> Response {
        match self {
            DecodeError::Unsupported(m) => json_error(StatusCode::UNSUPPORTED_MEDIA_TYPE, &m),
            DecodeError::TooLarge(m) => json_error(StatusCode::PAYLOAD_TOO_LARGE, &m),
            // [OPUS-4.8] (sq-cz89/sq-j9zs) Generic class to the client; decoder detail to the log.
            DecodeError::Malformed(detail) => sanitized_error(
                StatusCode::BAD_REQUEST,
                "gzip-decode",
                "malformed gzip body",
                &detail,
            ),
        }
    }
}

/// [OPUS-4.8] (sq-ebii) Decodes a request body honouring `Content-Encoding`, under the
/// decompression-ratio cap (zip-bomb guard). Returns the body verbatim for `identity` /
/// no encoding; for `gzip` it inflates with a HARD ceiling of
/// `min(max_decompress_ratio × compressed_len, max_body_bytes)` and refuses (413) the
/// instant the inflated output would cross it — so a tiny but pathologically compressible
/// body cannot inflate into an OOM. The ceiling is checked DURING inflate (bounded
/// `Read::take`), so the full decompressed image is never materialised past the cap.
///
/// `max_decompress_ratio == 0` disables ratio-capped decompression entirely: a
/// `Content-Encoding: gzip` body is then refused outright (fail-closed) rather than
/// inflated.
///
/// **Honest scope:** this guards the bodies the server itself inflates (the GSP write /
/// RDF-load path). It does NOT cover a compressed payload the *engine* fetches behind a
/// SPARQL `LOAD <url>` or `SERVICE` — those go through their own ingest path; the SERVICE
/// surface is separately bounded by the egress allowlist (sq-4w18). The request body-size
/// limit (`--max-body-bytes`, 413) caps the COMPRESSED bytes first, and the decompressed
/// ceiling is `min(max_decompress_ratio × compressed_len, max_body_bytes)` (see
/// [`decode_gzip_bounded`]) — so the decompressed output is itself capped at `max_body_bytes`,
/// never `max_body_bytes × max_decompress_ratio`.
fn decode_request_body(
    body: &Bytes,
    headers: &HeaderMap,
    config: &ServerConfig,
) -> Result<Bytes, DecodeError> {
    let encoding = headers
        .get(header::CONTENT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    match encoding.as_str() {
        "" | "identity" => Ok(body.clone()),
        "gzip" | "x-gzip" => decode_gzip_bounded(body, config),
        other => Err(DecodeError::Unsupported(format!(
            "unsupported Content-Encoding '{other}'; the server decodes only 'gzip' (and 'identity')"
        ))),
    }
}

/// [OPUS-4.8] (sq-ebii) gzip inflate bounded by the decompression-ratio cap. The ceiling is
/// `min(ratio × compressed_len, max_body_bytes)`, clamped to at least 1 byte so an empty /
/// tiny compressed body whose ratio product rounds below its real (small) output still
/// decodes. Reading is wrapped in `Read::take(ceiling + 1)`: if the decoder produces more
/// than `ceiling` bytes the read is cut short and we refuse (413) WITHOUT having held the
/// whole bomb in memory.
fn decode_gzip_bounded(body: &Bytes, config: &ServerConfig) -> Result<Bytes, DecodeError> {
    use std::io::Read;
    if config.max_decompress_ratio == 0 {
        return Err(DecodeError::TooLarge(
            "compressed (Content-Encoding: gzip) request bodies are disabled on this server \
             (--max-decompress-ratio 0); send an uncompressed body"
                .to_string(),
        ));
    }
    let ratio = config.max_decompress_ratio;
    // The decompressed ceiling: the ratio bound, but never above the absolute body limit's
    // worth of plaintext, and at least 1 so a tiny body is decodable. `saturating_mul`
    // avoids overflow on a huge compressed body (already bounded by --max-body-bytes anyway).
    let ceiling = body
        .len()
        .saturating_mul(ratio)
        .min(config.max_body_bytes.max(1))
        .max(1);
    let mut decoder = flate2::read::MultiGzDecoder::new(&body[..]).take(ceiling as u64 + 1);
    let mut out = Vec::with_capacity(ceiling.min(64 * 1024));
    decoder
        .read_to_end(&mut out)
        // [OPUS-4.8] (sq-cz89/sq-j9zs) Carry only the raw decoder detail; `into_response`
        // logs it server-side and returns a generic "malformed gzip body" to the client.
        .map_err(|e| DecodeError::Malformed(e.to_string()))?;
    if out.len() > ceiling {
        return Err(DecodeError::TooLarge(format!(
            "decompressed body exceeds the server's decompression-ratio cap (compressed {} bytes \
             × {ratio}× ratio, capped at {ceiling} bytes; raise --max-decompress-ratio / \
             --max-body-bytes or send a smaller body) — refused as a possible zip bomb",
            body.len()
        )));
    }
    Ok(Bytes::from(out))
}

/// Builds a `text`-ish response with the given content type; for HEAD, omits the body but
/// keeps the `Content-Type` (and an accurate `Content-Length` via the header) so HEAD
/// mirrors GET.
fn text_response(
    status: StatusCode,
    content_type: &str,
    body: String,
    head_only: bool,
) -> Response {
    let len = body.len();
    // For HEAD we advertise the same Content-Length the GET would have, with an empty body.
    let out_body = if head_only { String::new() } else { body };
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, len)
        .body(out_body.into())
        .unwrap()
}

/// Builds a response whose body is streamed from a pre-evaluated chunk sequence
/// (T16): hyper writes the chunks one by one instead of the server concatenating
/// them into a single giant `String` first. The total length is known, so the
/// response carries the same `Content-Type`/`Content-Length` headers — and, by the
/// engine's chunking contract, byte-identical body content — as the buffered path.
/// HEAD mirrors GET: same headers, empty body.
///
/// `pin` is the generation the chunks were evaluated against; the body stream owns it,
/// so the generation stays pinned until the response finishes (or the client goes away)
/// — the ring can never let the snapshot's memory go while the body is in flight. Today
/// the chunks are fully materialised strings, so this is belt-and-braces; it becomes
/// load-bearing the moment chunks evaluate lazily (Wave D push/pull streaming).
fn chunked_response(
    status: StatusCode,
    content_type: &str,
    chunks: Vec<String>,
    head_only: bool,
    pin: PinnedGen,
) -> Response {
    let len: usize = chunks.iter().map(String::len).sum();
    let body = if head_only {
        axum::body::Body::empty()
    } else {
        axum::body::Body::from_stream(futures_util::stream::iter(chunks.into_iter().map(
            move |c| {
                let _pinned_for_stream_lifetime = &pin;
                Ok::<_, std::convert::Infallible>(Bytes::from(c.into_bytes()))
            },
        )))
    };
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, len)
        .body(body)
        .unwrap()
}

/// Structured error body: every error the server emits is `{"error": "..."}` JSON, so
/// programmatic clients never have to scrape prose out of plain text.
///
/// [OPUS-4.8] sq-bxog: `pub(crate)` so the SSE subscription handler returns the SAME error
/// envelope as the rest of the HTTP surface for a pre-stream registration refusal.
pub(crate) fn json_error(status: StatusCode, msg: &str) -> Response {
    let mut body = String::with_capacity(msg.len() + 16);
    body.push_str("{\"error\":\"");
    for c in msg.chars() {
        match c {
            '"' => body.push_str("\\\""),
            '\\' => body.push_str("\\\\"),
            '\n' => body.push_str("\\n"),
            '\r' => body.push_str("\\r"),
            '\t' => body.push_str("\\t"),
            c if (c as u32) < 0x20 => body.push_str(&format!("\\u{:04x}", c as u32)),
            c => body.push(c),
        }
    }
    body.push_str("\"}");
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .body(body.into())
        .unwrap()
}

/// `map_response` middleware: rewrites plain-text error bodies produced *inside* the
/// stack (e.g. axum's body-limit 413 rejection) into the structured JSON error shape.
/// Non-error responses and already-JSON errors pass through untouched; original headers
/// (e.g. `Allow` on a 405) are preserved.
async fn json_error_bodies(resp: Response) -> Response {
    let status = resp.status();
    if !(status.is_client_error() || status.is_server_error()) {
        return resp;
    }
    let is_json = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("application/json"));
    if is_json {
        return resp;
    }
    let (mut parts, body) = resp.into_parts();
    // Error bodies are short; cap the read defensively.
    let bytes = axum::body::to_bytes(body, 64 * 1024)
        .await
        .unwrap_or_default();
    let msg = String::from_utf8_lossy(&bytes);
    let json = json_error(status, msg.trim());
    parts.headers.remove(header::CONTENT_TYPE);
    parts.headers.remove(header::CONTENT_LENGTH);
    let (jparts, jbody) = json.into_parts();
    for (k, v) in jparts.headers.iter() {
        parts.headers.insert(k, v.clone());
    }
    Response::from_parts(parts, jbody)
}

fn bad_request(msg: &str) -> Response {
    json_error(StatusCode::BAD_REQUEST, msg)
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-cmvh (ASVS-G1, cert remediation #237) — security response headers
//
// ASVS V14.4 wants standard hardening response headers on every HTTP response. sparq-server
// is a SPARQL *API* (it emits SPARQL-results JSON/XML/CSV/TSV and RDF — never HTML it asks a
// browser to render), so the header set is the subset that is meaningful for a non-HTML API,
// chosen deliberately rather than copy-pasting a browser-app template:
//
//   * `X-Content-Type-Options: nosniff` — stops a browser/proxy from MIME-sniffing a response
//     into a type we did not send. Always appropriate; cheap defence-in-depth even though we
//     always send an explicit `Content-Type`.
//   * `Content-Security-Policy: default-src 'none'; frame-ancestors 'none'` — the responses are
//     data, not scripts/styles/images, so the tightest possible CSP (`default-src 'none'`)
//     fits exactly: a `default-src 'none'` document loads no subresources and runs no inline
//     script, so an injected/`text/html`-sniffed body is inert. `frame-ancestors 'none'` is the
//     modern, header-spoofing-proof way to say "must not be framed" (it supersedes
//     X-Frame-Options for CSP-aware agents and, unlike X-Frame-Options, also covers nested
//     frames). We send BOTH (next bullet) for coverage of older agents.
//   * `X-Frame-Options: DENY` — the legacy clickjacking guard for agents that do not honour
//     CSP `frame-ancestors`. The API is never meant to be framed, so DENY is correct.
//   * `Referrer-Policy: no-referrer` — a programmatic API client has no referrer concept, but
//     if a response IRI is ever followed from a browser context this prevents the request URL
//     (which may carry a `query=` containing sensitive terms) leaking in a `Referer` header.
//
// DELIBERATELY OMITTED (with reason), so the audit trail is explicit:
//   * `Strict-Transport-Security` (HSTS): sparq-server terminates PLAIN HTTP (TLS is the job of
//     a fronting reverse proxy — see README "Security posture"). Emitting HSTS from the origin
//     would be meaningless at best and, if it reached a browser over the proxy's TLS, could
//     wrongly pin a host; the TLS-terminating proxy is the correct place to set HSTS. N/A here.
//   * `X-XSS-Protection`: deprecated and a no-op (or harmful) in modern browsers; superseded by
//     CSP. Not emitted.
//   * `Cross-Origin-*` / `Permissions-Policy` / CORS headers: browser-app document policies with
//     no meaning for a data API that serves no documents and (by design) no CORS. Not emitted —
//     adding CORS headers would *widen* the surface, the opposite of hardening.
//
// `Cache-Control` is NOT forced here: existing responses already manage their own cache
// semantics where they need to, and a blanket `no-store` would override `/health`, `/metrics`
// and the federation descriptors. Query results are not cached by default (no `Cache-Control:
// public`/`ETag` is ever set), so there is nothing to tighten — see the omission note in
// `skills/http-server/SKILL.md`. Applied to EVERY response (success, streamed, and error —
// it runs on the response path of the same map_response stack `json_error_bodies` uses), so an
// error envelope is hardened identically to a 200.
//
// Headers are only INSERTED when absent, so a handler that sets a more specific value (e.g. a
// future per-route CSP) is never clobbered.

/// The static security response headers (name, value) added to every response. Kept as a
/// single source of truth so the integration test and the middleware agree on the exact set.
pub(crate) const SECURITY_HEADERS: &[(header::HeaderName, &str)] = &[
    (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
    (
        header::CONTENT_SECURITY_POLICY,
        "default-src 'none'; frame-ancestors 'none'",
    ),
    (header::X_FRAME_OPTIONS, "DENY"),
    (header::REFERRER_POLICY, "no-referrer"),
];

/// `map_response` middleware ([OPUS-4.8] sq-cmvh): stamps the [`SECURITY_HEADERS`] hardening
/// set onto every response — success, streamed and error alike. Each header is only set when
/// absent, so a handler may override a specific one without this clobbering it. Header names
/// and values are all static and pre-validated, so the `from_static` conversions never fail.
async fn security_headers(mut resp: Response) -> Response {
    let headers = resp.headers_mut();
    for (name, value) in SECURITY_HEADERS {
        if !headers.contains_key(name) {
            headers.insert(name, header::HeaderValue::from_static(value));
        }
    }
    resp
}

// ---------------------------------------------------------------------------
// Information-leak guard (sq-cz89 / sq-j9zs / sq-zg0u) [OPUS-4.8]
//
// On the B3 no-auth-by-default path an unauthenticated caller could provoke an error
// whose body echoed its own (or the loaded data's, or the server's filesystem path's)
// content verbatim — parse errors from `oxttl` / `spargebra` quote the offending token,
// `io::Error` carries paths, etc. That is an information leak: it confirms loaded triples
// (e.g. `patient_alice_smith is not a valid subject`) and discloses server-side paths.
//
// Fix: HTTP error bodies carry only a STABLE, GENERIC class message — never the caller's
// input, loaded-data fragments, or filesystem paths. The full detail is preserved for the
// operator on the SERVER SIDE via `tracing` (surfaced by the existing opt-in `--verbose` /
// RUST_LOG subscriber set up in `main.rs`), exactly the posture the TraceLayer request log
// already uses. Detail in the log, class in the body.
// ---------------------------------------------------------------------------

/// Emits the full (potentially sensitive) `detail` to the server-side `tracing` log under
/// the `sparq_server` target — visible only to the operator who opted into `--verbose` /
/// `RUST_LOG` — and returns a SANITIZED [`Response`] carrying just `safe_msg`, which MUST NOT
/// contain any caller-submitted input, loaded-data fragment, or filesystem path.
///
/// `class` names the error category for the log line so an operator can correlate a generic
/// client-facing message back to its detailed cause.
fn sanitized_error(status: StatusCode, class: &str, safe_msg: &str, detail: &str) -> Response {
    // Detail (the echoed token / path / input) goes ONLY to the server log, gated behind the
    // operator's opt-in subscriber — never into the response body.
    tracing::warn!(target: "sparq_server", class = class, detail = %detail, "request error (detail withheld from client)");
    json_error(status, safe_msg)
}

fn execution_error(msg: &str) -> Response {
    // Engine error strings can embed term text drawn from the loaded graph — withhold them
    // from the client; the operator sees the full message in the server log.
    sanitized_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "query-execution",
        "query execution error",
        msg,
    )
}

fn method_not_allowed(allow: &[Method]) -> Response {
    let allow_value = allow
        .iter()
        .map(|m| m.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .header(header::ALLOW, allow_value)
        .body(axum::body::Body::from("method not allowed"))
        .unwrap()
}

// ---------------------------------------------------------------------------
// Visibility-scope extraction (sq-uqh, Wave B) — unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod touched_pods_tests {
    //! [OPUS-4.8] (sq-uqh) Per-named-graph `PodId` extraction from a parsed SPARQL Update.
    //! The whole point of Wave B's finer-than-global tagging: a write that names graph A
    //! must NOT bump graph B's epoch, while any write that cannot be scoped finer than the
    //! whole dataset MUST still bump the global pod (conservative, never under-invalidating).
    use super::{touched_pods, GLOBAL_POD};

    /// The set of pod-id strings a given update touches (order-independent).
    fn pods(update: &str) -> std::collections::BTreeSet<String> {
        touched_pods(update)
            .into_iter()
            .map(|p| p.as_str().to_string())
            .collect()
    }

    fn has(update: &str, iri: &str) -> bool {
        pods(update).contains(iri)
    }

    fn is_global(update: &str) -> bool {
        has(update, GLOBAL_POD)
    }

    /// A GRAPH-scoped INSERT/DELETE DATA touches exactly that named graph — never global,
    /// and never any OTHER graph (the A-not-B property at the extraction layer).
    #[test]
    fn graph_scoped_data_ops_are_scoped() {
        let ins =
            "INSERT DATA { GRAPH <http://ex/g/A> { <http://ex/s> <http://ex/p> <http://ex/o> } }";
        assert_eq!(
            pods(ins),
            ["http://ex/g/A".to_string()].into_iter().collect()
        );
        assert!(
            !is_global(ins),
            "a single-graph write must not bump the global pod"
        );
        assert!(!has(ins, "http://ex/g/B"), "a write to A must not touch B");

        let del =
            "DELETE DATA { GRAPH <http://ex/g/A> { <http://ex/s> <http://ex/p> <http://ex/o> } }";
        assert_eq!(
            pods(del),
            ["http://ex/g/A".to_string()].into_iter().collect()
        );
    }

    /// Multiple GRAPH blocks in one operation each contribute their own pod; no global.
    #[test]
    fn multiple_named_graphs_each_get_a_pod() {
        let u = "INSERT DATA { GRAPH <http://ex/g/A> { <http://ex/s> <http://ex/p> <http://ex/o> } \
                              GRAPH <http://ex/g/B> { <http://ex/s> <http://ex/p> <http://ex/o> } }";
        assert_eq!(
            pods(u),
            ["http://ex/g/A".to_string(), "http://ex/g/B".to_string()]
                .into_iter()
                .collect()
        );
        assert!(!is_global(u));
    }

    /// A default-graph data op is the catch-all pod (global) — the GSP surface maps every
    /// direct graph onto the default graph, so it cannot be scoped finer.
    #[test]
    fn default_graph_data_op_is_global() {
        let u = "INSERT DATA { <http://ex/s> <http://ex/p> <http://ex/o> }";
        assert!(
            is_global(u),
            "a default-graph write must bump the global pod"
        );
        assert_eq!(pods(u), [GLOBAL_POD.to_string()].into_iter().collect());
    }

    /// CLEAR/DROP of a single named graph is scoped; DEFAULT/NAMED/ALL widen to global.
    #[test]
    fn clear_drop_scoping() {
        assert_eq!(
            pods("CLEAR GRAPH <http://ex/g/A>"),
            ["http://ex/g/A".to_string()].into_iter().collect()
        );
        assert_eq!(
            pods("DROP GRAPH <http://ex/g/A>"),
            ["http://ex/g/A".to_string()].into_iter().collect()
        );
        assert!(is_global("CLEAR DEFAULT"));
        assert!(is_global("CLEAR NAMED"));
        assert!(is_global("CLEAR ALL"));
        assert!(is_global("DROP ALL"));
    }

    /// CREATE touches only the new graph.
    #[test]
    fn create_is_scoped() {
        assert_eq!(
            pods("CREATE GRAPH <http://ex/g/new>"),
            ["http://ex/g/new".to_string()].into_iter().collect()
        );
    }

    /// DELETE/INSERT … WHERE with CONCRETE template graphs is scoped to exactly the
    /// graphs the templates WRITE — the WHERE/USING read scope does not widen it.
    #[test]
    fn delete_insert_concrete_templates_are_scoped() {
        let u = "INSERT { GRAPH <http://ex/g/dst> { ?s ?p ?o } } \
                 WHERE  { GRAPH <http://ex/g/src> { ?s ?p ?o } }";
        // Only the write target (dst) is invalidation-relevant; the read source (src) is not.
        assert_eq!(
            pods(u),
            ["http://ex/g/dst".to_string()].into_iter().collect()
        );
        assert!(
            !has(u, "http://ex/g/src"),
            "the WHERE read source must not be invalidated"
        );
        assert!(!is_global(u));
    }

    /// A VARIABLE graph name in a template (`GRAPH ?g { … }`) makes the written graphs
    /// unknowable at parse time → must fall back to global (never under-invalidate).
    #[test]
    fn delete_insert_variable_graph_target_is_global() {
        let u = "INSERT { GRAPH ?g { ?s ?p ?o } } WHERE { GRAPH ?g { ?s ?p ?o } }";
        assert!(
            is_global(u),
            "a dynamically-scoped write target must bump the global pod"
        );
    }

    /// A default-graph DELETE/INSERT template is the catch-all pod.
    #[test]
    fn delete_insert_default_template_is_global() {
        let u = "DELETE { ?s ?p ?o } WHERE { ?s ?p ?o }";
        assert!(is_global(u));
    }

    /// A write that mixes a scopable named-graph effect with an unscopable one bumps BOTH
    /// the named pod AND global — finer scoping is additive over the catch-all.
    #[test]
    fn mixed_named_and_global_bumps_both() {
        let u =
            "INSERT DATA { GRAPH <http://ex/g/A> { <http://ex/s> <http://ex/p> <http://ex/o> } \
                              <http://ex/d> <http://ex/p> <http://ex/o> }";
        assert!(has(u, "http://ex/g/A"));
        assert!(
            is_global(u),
            "the default-graph half must still bump global"
        );
    }

    /// LOAD into a named graph is scoped; LOAD into the default graph is global. (The LOAD
    /// itself is refused at apply time without an allowlist — extraction is parse-only.)
    #[test]
    fn load_scoping() {
        assert_eq!(
            pods("LOAD <file:///x.ttl> INTO GRAPH <http://ex/g/A>"),
            ["http://ex/g/A".to_string()].into_iter().collect()
        );
        assert!(
            is_global("LOAD <file:///x.ttl>"),
            "LOAD into the default graph is global"
        );
    }

    /// An unparsable update is tagged global so extraction is total and never
    /// under-invalidates (the writer re-parses and rejects it anyway).
    #[test]
    fn unparsable_update_is_global() {
        assert!(is_global("THIS IS NOT SPARQL"));
    }
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-o4qf — bind-posture (no-auth non-loopback gate) tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod bind_posture_tests {
    //! The server has no built-in auth, so a non-loopback bind must be a deliberate opt-in.
    //! These tests pin: (a) loopback always proceeds; (b) a non-loopback bind WITHOUT the
    //! opt-in is refused (fail-closed, incl. the `0.0.0.0`/`::` all-interfaces addresses,
    //! which are the usual way the surface gets exposed); (c) WITH the opt-in it proceeds
    //! but warns. Pure functions over an explicit `allow_remote` flag — no process-env
    //! mutation, so they are parallel-safe.
    use super::{bind_posture, env_truthy, AuthPosture, BindPosture};
    use std::net::SocketAddr;

    fn addr(s: &str) -> SocketAddr {
        s.parse().unwrap()
    }

    #[test]
    fn loopback_proceeds_regardless_of_flag() {
        for a in ["127.0.0.1:3030", "127.0.0.5:80", "[::1]:3030"] {
            assert_eq!(
                bind_posture(&addr(a), false, AuthPosture::None),
                BindPosture::Loopback,
                "{a} (no opt-in)"
            );
            assert_eq!(
                bind_posture(&addr(a), true, AuthPosture::None),
                BindPosture::Loopback,
                "{a} (opt-in)"
            );
        }
    }

    #[test]
    fn non_loopback_without_optin_is_refused() {
        // 0.0.0.0 / :: (all-interfaces), an RFC1918 address, a link-local, and the cloud
        // metadata IP all fail closed without --allow-remote (and with no auth).
        for a in [
            "0.0.0.0:3030",
            "[::]:3030",
            "10.0.0.1:8080",
            "169.254.169.254:80",
            "192.168.1.5:3030",
        ] {
            match bind_posture(&addr(a), false, AuthPosture::None) {
                BindPosture::RemoteRefused { message } => {
                    assert!(message.contains("refusing to bind"), "{a}: {message}");
                    assert!(
                        message.contains("--allow-remote"),
                        "{a}: must name the opt-in flag"
                    );
                    assert!(
                        message.contains("authentication"),
                        "{a}: must explain the no-auth risk"
                    );
                }
                other => panic!("{a} must be refused without opt-in, got {other:?}"),
            }
        }
    }

    #[test]
    fn non_loopback_with_optin_warns_but_proceeds() {
        match bind_posture(&addr("0.0.0.0:3030"), true, AuthPosture::None) {
            BindPosture::RemoteAllowed { warning } => {
                assert!(warning.contains("WARNING"), "{warning}");
                assert!(warning.contains("READ AND WRITE"), "{warning}");
                assert!(
                    warning.contains("0.0.0.0:3030"),
                    "must name the address: {warning}"
                );
            }
            other => panic!("opt-in must allow with a warning, got {other:?}"),
        }
    }

    // [OPUS-4.8] sq-zcby — auth folded into the bind decision.

    /// A WRITE-only token (reads still open) is NOT sufficient to bind a non-loopback address:
    /// --allow-remote is still required, because reads remain open on the remote bind. The
    /// refusal must point at --auth-token-read as the way to make the whole surface safe.
    #[test]
    fn write_only_token_still_refused_without_optin() {
        match bind_posture(&addr("0.0.0.0:3030"), false, AuthPosture::WriteOnly) {
            BindPosture::RemoteRefused { message } => {
                assert!(message.contains("refusing to bind"), "{message}");
                assert!(
                    message.contains("--auth-token-read"),
                    "must name the read-gate flag: {message}"
                );
            }
            other => {
                panic!("a write-only token must still be refused without opt-in, got {other:?}")
            }
        }
    }

    /// A WRITE-only token WITH --allow-remote proceeds, but warns that READS remain open.
    #[test]
    fn write_only_token_with_optin_warns_reads_open() {
        match bind_posture(&addr("0.0.0.0:3030"), true, AuthPosture::WriteOnly) {
            BindPosture::RemoteAllowed { warning } => {
                assert!(warning.contains("WARNING"), "{warning}");
                assert!(
                    warning.to_ascii_uppercase().contains("READS"),
                    "must warn reads are open: {warning}"
                );
            }
            other => panic!("write-only + opt-in must allow with a warning, got {other:?}"),
        }
    }

    /// A FULLY authenticated surface (token gates reads AND writes) is allowed to bind a
    /// non-loopback address WITHOUT --allow-remote — there is no open endpoint left (sq-cxk5:
    /// the /subscriptions WS + SSE read surfaces are gated by --auth-token-read too). It still
    /// warns (single shared secret, deliver over TLS).
    #[test]
    fn full_auth_allows_remote_bind_without_optin() {
        match bind_posture(&addr("0.0.0.0:3030"), false, AuthPosture::ReadAndWrite) {
            BindPosture::RemoteAllowed { warning } => {
                assert!(warning.contains("WARNING"), "{warning}");
                assert!(
                    warning.contains("--auth-token"),
                    "must name the gate: {warning}"
                );
            }
            other => panic!("full auth must allow a remote bind without opt-in, got {other:?}"),
        }
    }

    #[test]
    fn env_truthy_recognises_common_forms() {
        for t in ["1", "true", "TRUE", " yes ", "On", "Yes"] {
            assert!(env_truthy(t), "{t:?} should be truthy");
        }
        for f in ["", "0", "false", "no", "off", "2", "maybe"] {
            assert!(!env_truthy(f), "{f:?} should be falsy");
        }
    }
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-zcby — auth-gate unit tests (constant-time eq, Bearer parsing,
// mutation classification, the gate decision, posture derivation)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod auth_tests {
    //! Pure-function coverage for the Bearer-token write gate. The end-to-end HTTP behaviour
    //! (401 shape, mutation-applied, read-gate, GSP write gate, update-via-query/form path) is
    //! the integration suite in `tests/auth.rs`; these pin the building blocks.
    use super::*;
    use axum::http::HeaderValue;

    fn headers_with_auth(value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(header::AUTHORIZATION, HeaderValue::from_str(value).unwrap());
        h
    }

    #[test]
    fn constant_time_eq_matches_plain_eq() {
        let cases: &[(&[u8], &[u8])] = &[
            (b"", b""),
            (b"a", b"a"),
            (b"a", b"b"),
            (b"token", b"token"),
            (b"token", b"toker"),
            (b"token", b"tokens"), // length mismatch
            (b"", b"x"),
        ];
        for (a, b) in cases {
            assert_eq!(constant_time_eq(a, b), a == b, "{a:?} vs {b:?}");
        }
    }

    #[test]
    fn bearer_token_tolerates_scheme_casing() {
        for v in ["Bearer t0k", "bearer t0k", "BEARER t0k", "BeArEr t0k"] {
            assert_eq!(bearer_token(&headers_with_auth(v)), Some("t0k"), "{v:?}");
        }
        // Trailing/leading space around the token is trimmed.
        assert_eq!(
            bearer_token(&headers_with_auth("Bearer  t0k  ")),
            Some("t0k")
        );
    }

    #[test]
    fn bearer_token_rejects_other_schemes_and_absence() {
        assert_eq!(bearer_token(&headers_with_auth("Basic abc")), None);
        assert_eq!(bearer_token(&headers_with_auth("t0k")), None); // no scheme
        assert_eq!(bearer_token(&HeaderMap::new()), None); // no header
    }

    #[test]
    fn payload_mutates_classifies_reads_and_writes() {
        // Reads (well-formed queries) do not mutate.
        for q in [
            "SELECT ?s WHERE { ?s ?p ?o }",
            "ASK { ?s ?p ?o }",
            "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }",
            "DESCRIBE <http://ex/s>",
        ] {
            assert!(!payload_mutates(q), "{q:?} must be a read");
        }
        // Writes (updates) mutate.
        for u in [
            "INSERT DATA { <http://ex/s> <http://ex/p> <http://ex/o> }",
            "DELETE DATA { <http://ex/s> <http://ex/p> <http://ex/o> }",
            "DELETE { ?s ?p ?o } WHERE { ?s ?p ?o }",
            "DROP ALL",
            "CLEAR DEFAULT",
            "LOAD <http://ex/d> INTO GRAPH <http://ex/g>",
        ] {
            assert!(payload_mutates(u), "{u:?} must be a write");
        }
        // Garbage that parses as neither is fail-closed to a write.
        assert!(
            payload_mutates("this is not sparql"),
            "unparsable must fail closed to a write"
        );
    }

    #[test]
    fn auth_posture_from_config() {
        let mut cfg = ServerConfig::default();
        assert_eq!(AuthPosture::from_config(&cfg), AuthPosture::None);
        cfg.auth_token = Some("t".into());
        assert_eq!(AuthPosture::from_config(&cfg), AuthPosture::WriteOnly);
        cfg.auth_token_read = true;
        assert_eq!(AuthPosture::from_config(&cfg), AuthPosture::ReadAndWrite);
        // read-gate without a token is still "None" (no token => nothing to gate with).
        cfg.auth_token = None;
        assert_eq!(AuthPosture::from_config(&cfg), AuthPosture::None);
    }

    #[test]
    fn auth_gate_no_token_never_gates() {
        let cfg = ServerConfig::default(); // no auth_token
        let h = HeaderMap::new();
        assert!(auth_gate(&cfg, &h, Operation::Write).is_none());
        assert!(auth_gate(&cfg, &h, Operation::Read).is_none());
    }

    #[test]
    fn auth_gate_write_only_gates_writes_not_reads() {
        let cfg = ServerConfig {
            auth_token: Some("secret".into()),
            ..ServerConfig::default()
        };
        // Writes need the token.
        assert!(
            auth_gate(&cfg, &HeaderMap::new(), Operation::Write).is_some(),
            "missing token => 401"
        );
        assert!(
            auth_gate(&cfg, &headers_with_auth("Bearer wrong"), Operation::Write).is_some(),
            "wrong token => 401"
        );
        assert!(
            auth_gate(&cfg, &headers_with_auth("Bearer secret"), Operation::Write).is_none(),
            "correct token => proceed"
        );
        // Reads stay open (no read gate).
        assert!(
            auth_gate(&cfg, &HeaderMap::new(), Operation::Read).is_none(),
            "reads open in write-only mode"
        );
    }

    #[test]
    fn auth_gate_read_gate_also_gates_reads() {
        let cfg = ServerConfig {
            auth_token: Some("secret".into()),
            auth_token_read: true,
            ..ServerConfig::default()
        };
        assert!(
            auth_gate(&cfg, &HeaderMap::new(), Operation::Read).is_some(),
            "read gated => 401"
        );
        assert!(
            auth_gate(&cfg, &headers_with_auth("bearer secret"), Operation::Read).is_none(),
            "correct token (lowercase scheme) => proceed"
        );
    }

    #[test]
    fn unauthorized_carries_www_authenticate_bearer() {
        let resp = unauthorized();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            resp.headers()
                .get(header::WWW_AUTHENTICATE)
                .and_then(|v| v.to_str().ok()),
            Some("Bearer")
        );
    }

    // [OPUS-4.8] sq-cxk5: the WebSocket subprotocol-token channel + the WS auth gate.

    fn headers_with_subprotocol(value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(
            header::SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_str(value).unwrap(),
        );
        h
    }

    #[test]
    fn subprotocol_bearer_token_extracts_the_bearer_subprotocol() {
        // The matched subprotocol AND the token after the `bearer.` prefix are returned.
        assert_eq!(
            subprotocol_bearer_token(&headers_with_subprotocol("bearer.t0k")),
            Some(("bearer.t0k", "t0k"))
        );
        // First bearer.* entry wins, even alongside other offered subprotocols.
        assert_eq!(
            subprotocol_bearer_token(&headers_with_subprotocol("graphql-ws, bearer.t0k, other")),
            Some(("bearer.t0k", "t0k"))
        );
        // The token body is exact (no trimming) — a subprotocol value carries no spaces anyway.
        assert_eq!(
            subprotocol_bearer_token(&headers_with_subprotocol("bearer.")),
            Some(("bearer.", ""))
        );
    }

    #[test]
    fn subprotocol_bearer_token_absent_without_the_prefix() {
        assert_eq!(
            subprotocol_bearer_token(&headers_with_subprotocol("graphql-ws, chat")),
            None
        );
        assert_eq!(subprotocol_bearer_token(&HeaderMap::new()), None);
        // A scheme other than the `bearer.` subprotocol prefix is not a match.
        assert_eq!(
            subprotocol_bearer_token(&headers_with_subprotocol("token.t0k")),
            None
        );
    }

    #[test]
    fn ws_auth_gate_no_token_never_gates() {
        let cfg = ServerConfig::default();
        assert!(ws_auth_gate(&cfg, &HeaderMap::new(), Operation::Read).is_none());
        // Even an offered bearer subprotocol proceeds when nothing is configured.
        assert!(ws_auth_gate(
            &cfg,
            &headers_with_subprotocol("bearer.anything"),
            Operation::Read
        )
        .is_none());
    }

    #[test]
    fn ws_auth_gate_read_gate_accepts_header_or_subprotocol() {
        let cfg = ServerConfig {
            auth_token: Some("secret".into()),
            auth_token_read: true,
            ..ServerConfig::default()
        };
        // Neither channel => 401.
        assert!(
            ws_auth_gate(&cfg, &HeaderMap::new(), Operation::Read).is_some(),
            "no credentials => 401"
        );
        // The Authorization: Bearer header is accepted (non-browser clients).
        assert!(
            ws_auth_gate(&cfg, &headers_with_auth("Bearer secret"), Operation::Read).is_none(),
            "header token => proceed"
        );
        // The Sec-WebSocket-Protocol bearer.<token> subprotocol is accepted (browsers).
        assert!(
            ws_auth_gate(
                &cfg,
                &headers_with_subprotocol("bearer.secret"),
                Operation::Read
            )
            .is_none(),
            "subprotocol token => proceed"
        );
        // A WRONG subprotocol token is VALIDATED, not echoed — 401.
        assert!(
            ws_auth_gate(
                &cfg,
                &headers_with_subprotocol("bearer.wrong"),
                Operation::Read
            )
            .is_some(),
            "wrong subprotocol token => 401"
        );
    }

    #[test]
    fn ws_auth_gate_write_only_leaves_ws_read_open() {
        // A write-only token (no --auth-token-read) does not gate the WS read upgrade.
        let cfg = ServerConfig {
            auth_token: Some("secret".into()),
            ..ServerConfig::default()
        };
        assert!(
            ws_auth_gate(&cfg, &HeaderMap::new(), Operation::Read).is_none(),
            "WS read open in write-only mode"
        );
    }
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-4w18 — SERVICE egress allowlist config wiring tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod hardening_unit_tests {
    //! [OPUS-4.8] (sq-ebii) Pure-logic units for the memory cap (`tighter`) and the
    //! decompression-ratio cap (`decode_request_body`). The HTTP-level 413/503 wiring is
    //! covered by tests/hardening.rs; these pin the boundary math without a server boot.
    use super::{decode_request_body, tighter, update_where_deadline, ServerConfig};
    use axum::body::Bytes;
    use axum::http::{header, HeaderMap, HeaderValue};
    use std::time::Duration;

    #[test]
    fn tighter_takes_the_smaller_present_cap() {
        assert_eq!(tighter(None, None), None);
        assert_eq!(tighter(Some(5), None), Some(5));
        assert_eq!(tighter(None, Some(7)), Some(7));
        assert_eq!(tighter(Some(5), Some(7)), Some(5));
        assert_eq!(tighter(Some(9), Some(7)), Some(7));
    }

    // [OPUS-4.8] (sq-nulp) The writer-side WHERE deadline for an UPDATE is the TIGHTER of the
    // read query_timeout and the opt-in update_where_timeout — that smaller value is what
    // bounds writer-queue head-of-line blocking. `None` only when BOTH are unset.
    #[test]
    fn update_where_deadline_is_the_tighter_of_query_and_update_timeouts() {
        let with = |q: Option<u64>, u: Option<u64>| {
            update_where_deadline(&ServerConfig {
                query_timeout: q.map(Duration::from_secs),
                update_where_timeout: u.map(Duration::from_secs),
                ..ServerConfig::default()
            })
        };
        // Both unset => no deadline at all.
        assert_eq!(with(None, None), None);
        // Only one present => that one (whichever it is).
        assert_eq!(with(Some(30), None), Some(Duration::from_secs(30)));
        assert_eq!(with(None, Some(2)), Some(Duration::from_secs(2)));
        // Both present => the SMALLER. The whole point: a short update_where_timeout wins over
        // a long read query_timeout, so the writer is released sooner (head-of-line bound).
        assert_eq!(with(Some(30), Some(2)), Some(Duration::from_secs(2)));
        // ...and an update_where_timeout LARGER than query_timeout never loosens the bound.
        assert_eq!(with(Some(5), Some(30)), Some(Duration::from_secs(5)));
    }

    // [OPUS-4.8] (sq-nulp) The DEFAULT config sets no update_where_timeout, so the update WHERE
    // budget is exactly the plain query_timeout — byte-for-byte the historical behaviour.
    #[test]
    fn update_where_deadline_defaults_to_query_timeout_unchanged() {
        let cfg = ServerConfig::default();
        assert_eq!(cfg.update_where_timeout, None, "no separate update deadline by default");
        assert_eq!(update_where_deadline(&cfg), cfg.query_timeout);
    }

    fn gz(data: &[u8]) -> Bytes {
        use std::io::Write;
        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
        e.write_all(data).unwrap();
        Bytes::from(e.finish().unwrap())
    }

    fn headers_with_encoding(enc: Option<&str>) -> HeaderMap {
        let mut h = HeaderMap::new();
        if let Some(e) = enc {
            h.insert(header::CONTENT_ENCODING, HeaderValue::from_str(e).unwrap());
        }
        h
    }

    #[test]
    fn identity_body_passes_through() {
        let cfg = ServerConfig::default();
        let body = Bytes::from_static(b"hello");
        // No Content-Encoding and explicit identity both pass verbatim.
        assert_eq!(
            decode_request_body(&body, &headers_with_encoding(None), &cfg).unwrap(),
            body
        );
        assert_eq!(
            decode_request_body(&body, &headers_with_encoding(Some("identity")), &cfg).unwrap(),
            body
        );
    }

    #[test]
    fn gzip_within_ratio_decodes() {
        let cfg = ServerConfig {
            max_decompress_ratio: 100,
            ..ServerConfig::default()
        };
        let plain = b"some moderately repetitive payload payload payload";
        let body = gz(plain);
        let out = decode_request_body(&body, &headers_with_encoding(Some("gzip")), &cfg).unwrap();
        assert_eq!(&out[..], &plain[..]);
    }

    #[test]
    fn high_ratio_gzip_is_refused() {
        // A 1 MiB run of zeros gzips to a tiny body — ratio far above 2× → refused.
        let cfg = ServerConfig {
            max_decompress_ratio: 2,
            max_body_bytes: 1 << 30,
            ..ServerConfig::default()
        };
        let body = gz(&vec![0u8; 1 << 20]);
        let err =
            decode_request_body(&body, &headers_with_encoding(Some("gzip")), &cfg).unwrap_err();
        assert!(matches!(err, super::DecodeError::TooLarge(_)));
    }

    #[test]
    fn ratio_zero_refuses_gzip() {
        let cfg = ServerConfig {
            max_decompress_ratio: 0,
            ..ServerConfig::default()
        };
        let body = gz(b"x");
        let err =
            decode_request_body(&body, &headers_with_encoding(Some("gzip")), &cfg).unwrap_err();
        assert!(matches!(err, super::DecodeError::TooLarge(_)));
    }

    #[test]
    fn unknown_encoding_is_unsupported() {
        let cfg = ServerConfig::default();
        let body = Bytes::from_static(b"x");
        let err = decode_request_body(&body, &headers_with_encoding(Some("br")), &cfg).unwrap_err();
        assert!(matches!(err, super::DecodeError::Unsupported(_)));
    }
}

#[cfg(test)]
mod service_allow_config_tests {
    //! The default posture and the ServerConfig <-> engine-entries plumbing. These are
    //! pure (no process-env mutation) so they parallelise; the env-precedence path
    //! (SPARQ_SERVICE_ALLOW baseline + CLI/file union) is covered by
    //! `service_config::tests::from_sources_unions_cli_file_env`.
    use super::ServerConfig;
    use crate::service_config::ServiceAllowlist;

    #[test]
    fn default_config_denies_all_service() {
        // The safe default: no host allowlisted => deny ALL SERVICE.
        let cfg = ServerConfig::default();
        assert!(cfg.service_allow.is_empty(), "default must be deny-all");
        assert!(cfg.service_allow.engine_entries().is_empty());
    }

    #[test]
    fn populated_allowlist_flows_to_engine_entries() {
        let mut cfg = ServerConfig::default();
        let mut allow = ServiceAllowlist::default();
        allow.add("sparql.example.org").unwrap();
        allow.add("*.internal").unwrap();
        cfg.service_allow = allow;
        let mut e = cfg.service_allow.engine_entries();
        e.sort();
        // Exact host verbatim; the wildcard in the engine's leading-dot form.
        assert_eq!(
            e,
            vec![".internal".to_string(), "sparql.example.org".to_string()]
        );
    }
}

/// [OPUS-4.8] (sq-vpx4) The HTTP status CONTRACT for a durable-write refusal: a
/// `WriteError::Unavailable` (carried across `apply_update` with the internal
/// `DURABLE_UNAVAILABLE_PREFIX`) maps to HTTP 503 (retryable) — NOT the default 400 for a
/// rejected update, and NOT a 500. The internal marker must never leak to the client.
#[cfg(test)]
mod durable_degrade_tests {
    use super::{update_rejection_response, ServerConfig, DURABLE_UNAVAILABLE_PREFIX};
    use axum::http::StatusCode;

    #[tokio::test]
    async fn durable_unavailable_maps_to_503_and_hides_marker() {
        let cfg = ServerConfig::default();
        let tagged = format!("{DURABLE_UNAVAILABLE_PREFIX}injected ENOSPC");
        let resp = update_rejection_response(&tagged, &cfg);
        assert_eq!(
            resp.status(),
            StatusCode::SERVICE_UNAVAILABLE,
            "a durable-write refusal must be a retryable 503, not a 400/500",
        );

        // The internal marker must NEVER reach the client: assert it is absent from the
        // BODY (not just that the status is right). A future leak of the prefix into the
        // client-facing JSON — raw OR JSON-escaped (`json_error` escapes the U+0001 control
        // chars to ``) — is caught here. The human-readable detail after the marker
        // is expected to survive; only the marker bytes must be stripped.
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(
            !body.contains(DURABLE_UNAVAILABLE_PREFIX),
            "raw internal marker leaked into the client-facing body: {body:?}",
        );
        // The JSON-escaped form of the marker's control bytes (U+0001 -> ).
        assert!(
            !body.contains("\\u0001"),
            "JSON-escaped internal marker leaked into the client-facing body: {body:?}",
        );
        // [OPUS-4.8] (sq-cz89/sq-j9zs) The post-marker detail is the underlying I/O error,
        // which can carry the server's `--persist` filesystem PATH (an info-leak). It is now
        // WITHHELD from the client body (routed to the server log instead). The client gets
        // only the stable, retry-actionable class message.
        assert!(
            !body.contains("injected ENOSPC"),
            "the underlying detail (may carry a server path) must NOT reach the client: {body:?}",
        );
        assert!(
            body.contains("retry") && body.contains("NOT applied"),
            "the client must still get the stable retry-actionable class message: {body:?}",
        );
    }

    #[test]
    fn ordinary_update_rejection_still_400() {
        // A plain application error (parse/semantic) keeps its 400 — the 503 sniff must not
        // hijack the normal rejection path.
        let cfg = ServerConfig::default();
        let resp = update_rejection_response("some parse error", &cfg);
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
