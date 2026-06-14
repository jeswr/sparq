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
    routing::{any, get},
    BoxError, Router,
};
use tower::ServiceBuilder;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::trace::TraceLayer;

use sparq_core::Graph;
use sparq_engine::QueryBudget;
use sparq_serve::{ApplyUpdates, Generation, GenerationRing, GraphApplier, PodId, WriteError, Writer, WriterConfig};

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
    /// (the default) disables it. A true byte-accounted allocator cap is deferred (bead).
    ///
    /// Distinct from [`max_results`]: `max_results` caps only the FINAL SELECT projection
    /// (ASK/CONSTRUCT/DESCRIBE ignore it); this caps the working set on *all* forms. When
    /// both are set, a SELECT uses the tighter of the two.
    pub max_query_rows: Option<usize>,
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
    /// [OPUS-4.8] sq-o4qf: explicit opt-in to bind a **non-loopback** address.
    ///
    /// The server has **no authentication** on any endpoint — including the mutating
    /// `application/sparql-update` path and the `/subscriptions` WebSocket. Binding a
    /// non-loopback address (e.g. `0.0.0.0`) therefore exposes the entire dataset for
    /// **read AND write** to anyone who can reach the port. To make that a deliberate
    /// act rather than a foot-gun, the binary REFUSES to bind a non-loopback address
    /// unless this is set (CLI `--allow-remote` / env `SPARQ_ALLOW_REMOTE=1`); even
    /// then it logs a loud warning. Loopback binds are unaffected. See
    /// [`bind_posture`] for the decision, and `crates/sparq-server/README.md` →
    /// "Security posture (no built-in auth)".
    ///
    /// This is purely a *bind-time* posture gate in the binary; it does not add any
    /// per-request auth and does not affect the library `router`/`harden` surface.
    pub allow_remote: bool,
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
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            query_timeout: Some(Duration::from_secs(30)),
            max_body_bytes: 1024 * 1024, // 1 MiB
            max_concurrent: 32,
            max_results: None,
            // [OPUS-4.8] sq-ebii: memory cap OFF by default (no surprise refusals on an
            // unconfigured server); an operator exposing the endpoint opts a ceiling in.
            max_query_rows: None,
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
            allow_remote: false, // [OPUS-4.8] sq-o4qf: safe default — refuse non-loopback bind unless opted in
            // [OPUS-4.8] sq-4w18: safe default — empty allowlist = deny ALL SERVICE.
            service_allow: crate::service_config::ServiceAllowlist::default(),
        }
    }
}

impl ServerConfig {
    /// Defaults overridden by the `SPARQ_QUERY_TIMEOUT` (seconds; `0` disables),
    /// `SPARQ_MAX_BODY_BYTES`, `SPARQ_MAX_CONCURRENT`, `SPARQ_MAX_RESULTS`,
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
        // [OPUS-4.8] sq-o4qf: SPARQ_ALLOW_REMOTE truthy ("1"/"true"/"yes"/"on", case-insensitive)
        // opts in to a non-loopback bind. Anything else (incl. unset / "0" / "false") leaves the
        // safe default of refusing to expose the unauthenticated surface beyond loopback.
        if let Ok(v) = std::env::var("SPARQ_ALLOW_REMOTE") {
            cfg.allow_remote = env_truthy(&v);
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
        Ok(cfg)
    }
}

/// [OPUS-4.8] sq-o4qf: parse a boolean-ish env value. Truthy: `1`, `true`, `yes`, `on`
/// (case-insensitive, trimmed); everything else (including empty) is false.
fn env_truthy(v: &str) -> bool {
    matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-o4qf — bind-time security posture (no built-in auth)
// ---------------------------------------------------------------------------

/// [OPUS-4.8] sq-o4qf: the decision the binary makes about a requested bind address,
/// given the no-auth posture. Returned by [`bind_posture`] so `main` (and tests) can act
/// on it without the side effect of actually binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindPosture {
    /// Loopback (or otherwise not-remotely-reachable) bind — safe, proceed silently.
    Loopback,
    /// Non-loopback bind explicitly opted into (`--allow-remote` / `SPARQ_ALLOW_REMOTE`).
    /// Proceed, but the caller MUST surface `warning` (the unauthenticated surface is now
    /// reachable from the network).
    RemoteAllowed { warning: String },
    /// Non-loopback bind WITHOUT the opt-in. The caller MUST refuse to bind and print
    /// `message` (which explains the no-auth risk and how to opt in).
    RemoteRefused { message: String },
}

/// [OPUS-4.8] sq-o4qf: classify a requested bind address under the no-built-in-auth posture.
///
/// `sparq-server` has no authentication on any endpoint (query, `application/sparql-update`,
/// the `/subscriptions` WebSocket). A loopback bind (`127.0.0.0/8`, `::1`) is only reachable
/// from the same host, so it is safe by default. A **non-loopback** bind exposes the full
/// read+write surface to the network, so the binary refuses it unless the operator explicitly
/// opts in via `--allow-remote` / `SPARQ_ALLOW_REMOTE=1` (and even then warns).
///
/// "Loopback" here means the literal loopback ranges. Note `0.0.0.0` / `::` (the unspecified
/// "bind to all interfaces" addresses) are NOT loopback — they are the most common way the
/// surface gets exposed, so they are treated as remote. This is a deliberately blunt,
/// fail-closed check: it errs toward refusing exposure, not toward allowing it.
pub fn bind_posture(addr: &SocketAddr, allow_remote: bool) -> BindPosture {
    if addr.ip().is_loopback() {
        return BindPosture::Loopback;
    }
    if allow_remote {
        BindPosture::RemoteAllowed {
            warning: format!(
                "WARNING: sparq-server is binding the non-loopback address {addr} and has NO \
                 built-in authentication. The full dataset is exposed for READ AND WRITE \
                 (SPARQL Update + the /subscriptions WebSocket) to anyone who can reach this \
                 port. Put it behind a reverse proxy / API gateway (or sparq-solid) that \
                 enforces auth before exposing it to an untrusted network. \
                 (--allow-remote / SPARQ_ALLOW_REMOTE is set, so this bind proceeds.)"
            ),
        }
    } else {
        BindPosture::RemoteRefused {
            message: format!(
                "refusing to bind non-loopback address {addr}: sparq-server has NO built-in \
                 authentication, so this would expose the full dataset for READ AND WRITE \
                 (SPARQL Update + the /subscriptions WebSocket) to the network. If that is \
                 intended, run behind a reverse proxy / gateway that enforces auth and \
                 re-run with --allow-remote (or SPARQ_ALLOW_REMOTE=1). To serve only this \
                 host, bind a loopback address such as 127.0.0.1."
            ),
        }
    }
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
                GraphTarget::DefaultGraph | GraphTarget::NamedGraphs | GraphTarget::AllGraphs => acc.global(),
            },
            // CREATE makes one empty named graph — touches only it.
            GraphUpdateOperation::Create { graph, .. } => acc.named(graph.as_str()),
            // DELETE/INSERT … WHERE writes the template graph slots. Concrete slots are
            // scopable; a variable graph name (or a default-graph slot) is not, so it
            // widens to global. Reads (WHERE / USING) do not affect invalidation scope.
            GraphUpdateOperation::DeleteInsert { delete, insert, .. } => {
                for slot in delete.iter().map(|q| &q.graph_name).chain(insert.iter().map(|q| &q.graph_name)) {
                    match slot {
                        GraphNamePattern::NamedNode(n) => acc.named(n.as_str()),
                        // Default graph or a dynamically-bound graph name: unscopable → global.
                        GraphNamePattern::DefaultGraph | GraphNamePattern::Variable(_) => acc.global(),
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
struct ServerApplier {
    inner: GraphApplier,
    /// The config the writer thread enforces around every engine call (carries the
    /// SERVICE egress allowlist; the writer has no per-request config, so it owns its own).
    config: Arc<ServerConfig>,
}

impl ServerApplier {
    fn new(config: Arc<ServerConfig>) -> Self {
        Self { inner: GraphApplier::default(), config }
    }
}

impl ApplyUpdates for ServerApplier {
    type Snapshot = Graph;
    type Working = Graph;
    type Update = String;

    fn fork(&mut self, base: &Graph) -> Result<Graph, String> {
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
        with_engine_scope(&self.config, || sparq_engine::update_in_place_with_budget(working, update, &budget))
    }

    fn seal(&mut self, working: Graph) -> Graph {
        self.inner.seal(working)
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
}

impl AppState {
    pub fn new(graph: Graph) -> Self {
        Self::with_config(graph, ServerConfig::default())
    }

    pub fn with_config(graph: Graph, config: ServerConfig) -> Self {
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
        let ring = Arc::new(GenerationRing::with_config(graph, ring_config));
        // [OPUS-4.8] sq-4w18: share the config Arc with the writer so its update path
        // enforces the same SERVICE egress allowlist (a federated `INSERT … WHERE` is
        // gated like a read).
        let config = Arc::new(config);
        let writer = Arc::new(Writer::spawn(
            ring.clone(),
            ServerApplier::new(config.clone()),
            WriterConfig::default(),
        ));
        Self {
            ring,
            writer,
            config,
            commits: Arc::new(tokio::sync::watch::channel(0).0),
            subs: Arc::new(crate::subscriptions::SubscriptionCounters::default()),
            metrics: Arc::new(crate::metrics::Metrics::default()),
        }
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
        }
    }

    /// A receiver of the committed generation number for a subscription connection (T23).
    pub(crate) fn subscribe_commits(&self) -> tokio::sync::watch::Receiver<u64> {
        self.commits.subscribe()
    }

    pub(crate) fn config(&self) -> &ServerConfig {
        &self.config
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
        .route("/subscriptions", get(crate::subscriptions::subscriptions_endpoint))
        // Liveness.
        .route("/health", get(|| async { "ok" }))
        // Prometheus metrics (T22).
        .route("/metrics", get(metrics_endpoint))
        .with_state(state.clone());
    // The metrics middleware wraps the WHOLE hardened stack so shed requests
    // (429), body-limit rejections (413) and panics (500) are counted with the
    // status the client actually saw.
    harden(routes, &config).layer(axum::middleware::from_fn_with_state(state, crate::metrics::track))
}

/// `GET /metrics` — Prometheus text exposition (T22). The gauges (graph triple
/// count, active subscriptions) are read at scrape time from live state.
async fn metrics_endpoint(State(state): State<AppState>) -> Response {
    let triples = state.current().snapshot().len();
    let subs = state.subs.active_count();
    let body = state.metrics().render(subs, triples);
    text_response(StatusCode::OK, "text/plain; version=0.0.4; charset=utf-8", body, false)
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
            .layer(CatchPanicLayer::custom(|_: Box<dyn std::any::Any + Send>| {
                json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal server error (panic)")
            }))
            // Load-shed converts "concurrency limit reached" into an immediate error
            // (mapped to 429) instead of queueing unboundedly.
            .layer(HandleErrorLayer::new(|err: BoxError| async move {
                if err.is::<tower::load_shed::error::Overloaded>() {
                    json_error(StatusCode::TOO_MANY_REQUESTS, "server is at its concurrent-request limit, retry later")
                } else {
                    json_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("middleware error: {err}"))
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
        routes.layer(TraceLayer::new_for_http())
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
    if let Some(v) = body_params.and_then(|m| m.get("explain")).or_else(|| url_params.get("explain")) {
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
    let raw = match body_params.and_then(|m| m.get("generation")).or_else(|| url_params.get("generation")) {
        Some(raw) => raw,
        None => return Ok(state.current()),
    };
    let number: u64 = raw
        .parse()
        .map_err(|_| bad_request(&format!("invalid 'generation' parameter '{raw}': expected a generation number")))?;
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
            // GET without a `query` parameter is a malformed request (400).
            match url_params.get("query") {
                Some(q) => {
                    let pin = match resolve_pin(&state, &url_params, None) {
                        Ok(pin) => pin,
                        Err(resp) => return resp,
                    };
                    let explain = explain_mode(&url_params, None, &headers);
                    run_query(&state, q, &headers, method == Method::HEAD, explain, pin).await
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
        // POST directly — body IS the SPARQL query.
        match std::str::from_utf8(body) {
            Ok(q) => {
                let pin = match resolve_pin(state, url_params, None) {
                    Ok(pin) => pin,
                    Err(resp) => return resp,
                };
                let explain = explain_mode(url_params, None, headers);
                run_query(state, q, headers, false, explain, pin).await
            }
            Err(_) => bad_request("request body is not valid UTF-8"),
        }
    } else if ct.starts_with(FORM_CT) {
        // POST url-encoded — `query=` in the body.
        let s = match std::str::from_utf8(body) {
            Ok(s) => s,
            Err(_) => return bad_request("request body is not valid UTF-8"),
        };
        let params = parse_form(s);
        match params.get("query") {
            Some(q) => {
                let pin = match resolve_pin(state, url_params, Some(&params)) {
                    Ok(pin) => pin,
                    Err(resp) => return resp,
                };
                let explain = explain_mode(url_params, Some(&params), headers);
                run_query(state, q, headers, false, explain, pin).await
            }
            None => bad_request("missing 'query' parameter in url-encoded body"),
        }
    } else if ct.starts_with("application/sparql-update") {
        // SPARQL 1.1 Protocol — update operation (T11b). Body IS the update; success → 204.
        // `apply_update` blocks for the writer's group-commit ack (window + batch
        // application, which includes an O(graph) fork), so it runs off the async workers.
        // Time travel is a READ concept: an update can only apply to the current
        // generation, so a `generation` pin on an update is an honest refusal, not
        // a silent ignore.
        #[cfg(feature = "time-travel")]
        if url_params.contains_key("generation") {
            return bad_request(
                "the 'generation' parameter pins queries to a retained generation; \
                 updates always apply to the current generation",
            );
        }
        match std::str::from_utf8(body) {
            Ok(u) => {
                let st = state.clone();
                let u = u.to_string();
                let task = tokio::task::spawn_blocking(move || st.apply_update(&u));
                // [OPUS-4.8] sq-ebii: enforce the query timeout on the UPDATE path too — the
                // SAME wall-clock hard cap (`timeout + TIMEOUT_GRACE`) the read paths use via
                // `await_worker`. See `await_update_worker` for the honest caveat (the
                // engine's update WHERE evaluation is not yet cooperatively budgeted, so this
                // is a wall-clock cap on the await, not a mid-evaluation abort).
                match await_update_worker(task, &state.config).await {
                    UpdateOutcome::Ok(number) => {
                        state.metrics().inc_updates();
                        // The 204 carries the generation containing the update (the
                        // read-your-writes token) under the time-travel feature.
                        with_generation_header(StatusCode::NO_CONTENT.into_response(), number)
                    }
                    UpdateOutcome::Rejected(e) => update_rejection_response(&e, &state.config),
                    UpdateOutcome::TimedOut => timeout_response(&state.config),
                    UpdateOutcome::Panicked => json_error(StatusCode::INTERNAL_SERVER_ERROR, "update worker panicked"),
                }
            }
            Err(_) => bad_request("request body is not valid UTF-8"),
        }
    } else {
        // Unsupported media type for the POST query operation.
        json_error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "POST requires Content-Type 'application/sparql-query' or 'application/x-www-form-urlencoded'",
        )
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
        Err(PrepareError::Malformed(msg)) => return bad_request(&format!("malformed query: {msg}")),
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
                Ok(text) => text_response(StatusCode::OK, "text/plain; charset=utf-8", text, head_only),
                // ANALYZE of a non-SELECT/ASK form is a client error, not a server one.
                Err(e) if e.contains("EXPLAIN ANALYZE supports") => bad_request(&e),
                Err(e) => engine_error_response(&e, &cfg),
            }
        });
        return await_worker(task, &config).await;
    }
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok());
    let fmt = negotiate(accept);
    let config = state.config.clone();

    match prepared.form {
        QueryForm::Select | QueryForm::Ask => {
            let is_ask = prepared.form == QueryForm::Ask;
            let select = prepared.runnable;
            let budget = make_budget(&config, !is_ask);
            let cfg = config.clone();
            let task = tokio::task::spawn_blocking(move || {
                with_engine_scope(&cfg, || if is_ask {
                    match sparq_engine::ask_with_budget(gen.snapshot(), &select, &budget) {
                        Ok(value) => {
                            let (body, ct) = match fmt {
                                Format::Xml => (results::ask_to_xml(value), fmt.ask_content_type()),
                                _ => (results::ask_to_json(value), fmt.ask_content_type()),
                            };
                            text_response(StatusCode::OK, ct, body, head_only)
                        }
                        Err(e) => engine_error_response(&e, &cfg),
                    }
                } else {
                    render_select(&gen, &select, fmt, head_only, &budget, &cfg)
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
                match with_engine_scope(&cfg, || sparq_engine::construct_or_describe_with_budget(gen.snapshot(), &query, &budget)) {
                    Ok(triples) => {
                        let body = serialise_graph_triples(&triples, gfmt);
                        text_response(StatusCode::OK, gfmt.content_type(), body, head_only)
                    }
                    Err(e) => engine_error_response(&e, &cfg),
                }
            });
            await_worker(task, &config).await
        }
    }
}

/// The per-request engine budget: deadline from the configured timeout; the row cap from
/// the memory cap (`--max-query-rows`, applied on EVERY form) AND — when `apply_max_results`
/// is set (SELECT projections) — `--max-results`, whichever is tighter.
///
/// [OPUS-4.8] (sq-ebii) `--max-query-rows` is the coarse memory cap: it bounds the
/// working-set cardinality of any materialised intermediate/final result on all forms
/// (SELECT/ASK/CONSTRUCT/DESCRIBE/GSP-read), so a join blow-up aborts (413) instead of
/// OOMing. `--max-results` is the narrower SELECT-projection cap (ASK ignores it). Both map
/// onto the single engine `max_rows` budget, so the effective ceiling is their min.
pub(crate) fn make_budget(config: &ServerConfig, apply_max_results: bool) -> QueryBudget {
    let results_cap = if apply_max_results { config.max_results } else { None };
    QueryBudget {
        deadline: config.query_timeout.map(|t| std::time::Instant::now() + t),
        max_rows: tighter(config.max_query_rows, results_cap),
    }
}

/// [OPUS-4.8] (sq-ebii) The per-UPDATE engine budget: the coarse memory cap
/// (`--max-query-rows`) as the working-set row ceiling, and the query timeout as a
/// cooperative deadline measured from NOW (the moment the writer starts this update). The
/// SELECT-projection cap (`--max-results`) does NOT apply to updates — there is no result to
/// project — but the working-set memory cap and the deadline do.
fn update_budget(config: &ServerConfig) -> QueryBudget {
    QueryBudget {
        deadline: config.query_timeout.map(|t| std::time::Instant::now() + t),
        max_rows: config.max_query_rows,
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
        Err(e) if e.is_panic() => json_error(StatusCode::INTERNAL_SERVER_ERROR, "query worker panicked"),
        Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "query worker was cancelled"),
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
/// HTTP *await*. Unlike the read paths, the cooperative [`QueryBudget`] deadline does NOT
/// reach inside the update: the engine's update entry point (`update_in_place`) takes no
/// budget, so an in-progress update's WHERE evaluation is not interrupted at the deadline —
/// the writer thread keeps running it to completion and the result is simply discarded once
/// the HTTP side has already answered 503. Updates are also *sequenced* on a single writer,
/// so a long-running update blocks the queue behind it until it finishes (this cap bounds
/// the client's wait, not the writer's work). Threading a real cooperative budget through the
/// engine update path + the sequenced writer is the proper fix — deferred (bead).
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
        Format::Json => match sparq_engine::query_json_chunks_with_budget(graph, select, budget) {
            Ok(chunks) => return chunked_response(StatusCode::OK, ct, chunks, head_only, gen.clone()),
            Err(e) => return engine_error_response(&e, config),
        },
        _ => {
            let result = match sparq_engine::query_with_budget(graph, select, budget) {
                Ok(r) => r,
                Err(e) => return engine_error_response(&e, config),
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
fn engine_error_response(e: &str, config: &ServerConfig) -> Response {
    if e.contains("query budget exceeded (timeout)") {
        return timeout_response(config);
    }
    if e.contains("query budget exceeded (max-rows)") {
        // [OPUS-4.8] sq-ebii: the row budget is fed by BOTH --max-results (SELECT projection)
        // and --max-query-rows (the coarse memory cap, all forms); name the tighter one that
        // actually tripped so the operator knows which knob to raise.
        let max = tighter(config.max_query_rows, config.max_results).unwrap_or(0);
        let which = match (config.max_query_rows, config.max_results) {
            (Some(q), Some(r)) if q <= r => "--max-query-rows (memory cap)",
            (Some(_), None) => "--max-query-rows (memory cap)",
            _ => "--max-results",
        };
        return json_error(
            StatusCode::PAYLOAD_TOO_LARGE,
            &format!("result exceeds the server's working-set row limit ({max} rows, {which}); narrow the query (e.g. add LIMIT) or raise the limit"),
        );
    }
    execution_error(e)
}

/// [OPUS-4.8] (sq-ebii) Maps a writer-rejected UPDATE onto HTTP status: a cooperative budget
/// hit (the memory cap / deadline tripped inside a `DELETE/INSERT … WHERE`) is a 413 / 503,
/// exactly like a query; any other rejection (parse / semantic error) is the client's 400.
fn update_rejection_response(e: &str, config: &ServerConfig) -> Response {
    if e.contains("query budget exceeded") {
        return engine_error_response(e, config);
    }
    bad_request(&format!("update failed: {e}"))
}

fn timeout_response(config: &ServerConfig) -> Response {
    let secs = config.query_timeout.map(|t| t.as_secs()).unwrap_or(0);
    json_error(
        StatusCode::SERVICE_UNAVAILABLE,
        &format!("query timed out (server limit: {secs}s); simplify the query or raise --query-timeout"),
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
        (false, None) => return bad_request("indirect graph identification requires '?default' or '?graph=<uri>'"),
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
async fn graph_store(state: &AppState, method: &Method, graph: GraphRef, headers: &HeaderMap, body: Bytes) -> Response {
    match *method {
        Method::GET | Method::HEAD => {
            let head_only = *method == Method::HEAD;
            serialise_graph(state, graph, headers, head_only).await
        }
        Method::PUT => gsp_put(state, graph, headers, &body).await,
        Method::POST => gsp_post(state, graph, headers, &body).await,
        Method::DELETE => gsp_delete(state, graph).await,
        // PATCH is not part of the Graph Store HTTP Protocol.
        _ => method_not_allowed(&[Method::GET, Method::HEAD, Method::PUT, Method::POST, Method::DELETE]),
    }
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
fn body_to_ntriples(body: &Bytes, content_type: &str, base: Option<&str>) -> Result<String, Response> {
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
            let text = std::str::from_utf8(body).map_err(|_| bad_request("request body is not valid UTF-8"))?;
            let graph = Graph::load_str(text, format).map_err(|e| bad_request(&format!("malformed RDF body: {e}")))?;
            // Re-emit as canonical N-Triples via the engine scan — no private store API needed,
            // and the terms are exactly what `INSERT DATA` will re-intern.
            nt_dump(&graph, &QueryBudget::default()).map_err(|e| execution_error(&e))
        }
        // [OPUS-4.8] sq-rt6v: parse RDF/XML to triples and emit canonical N-Triples directly.
        // `oxrdf`'s Display is canonical N-Triples term syntax, exactly what `INSERT DATA`
        // re-interns, so no Graph round-trip is needed (and RDF/XML is not a loader token).
        BodyFormat::RdfXml => {
            let triples = crate::graph::parse_rdfxml(body, base).map_err(|e| bad_request(&format!("malformed RDF/XML body: {e}")))?;
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
                bad_request(&format!("graph store write failed: {e}"))
            }
        }
        UpdateOutcome::TimedOut => timeout_response(&state.config),
        UpdateOutcome::Panicked => json_error(StatusCode::INTERNAL_SERVER_ERROR, "update worker panicked"),
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
    let success = if existed { StatusCode::NO_CONTENT } else { StatusCode::CREATED };
    apply_gsp_update(state, update, success).await
}

/// `POST <graph>` — MERGE (additive) the request body into the graph (GSP §5). Maps to
/// `INSERT DATA { GRAPH <g> { … } }`. 201 when the merge created the graph (a selector-less
/// POST, or a POST to an absent named graph), 204 when it added to an existing graph.
async fn gsp_post(state: &AppState, graph: GraphRef, headers: &HeaderMap, body: &Bytes) -> Response {
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
            GraphRef::Default => return with_generation_header(StatusCode::NO_CONTENT.into_response(), state.current().number()),
            GraphRef::Named(_) if existed => {
                return with_generation_header(StatusCode::NO_CONTENT.into_response(), state.current().number())
            }
            GraphRef::Named(iri) => format!("CREATE SILENT GRAPH <{}>", escape_iri(iri)),
        }
    } else {
        let block = graph_data_block(&graph, &ntriples);
        format!("INSERT DATA {{ {block} }}")
    };
    let success = if existed { StatusCode::NO_CONTENT } else { StatusCode::CREATED };
    apply_gsp_update(state, update, success).await
}

/// `DELETE <graph>` — DROP the graph (GSP §6). 204 on success; 404 when the named graph is
/// absent (per the spec's "graph does not exist" semantics). The default graph always
/// exists, so `DELETE ?default` empties it and is always 204. Maps to `DROP GRAPH <g>` /
/// `CLEAR DEFAULT`.
async fn gsp_delete(state: &AppState, graph: GraphRef) -> Response {
    match &graph {
        GraphRef::Default => apply_gsp_update(state, "CLEAR DEFAULT".to_string(), StatusCode::NO_CONTENT).await,
        GraphRef::Named(iri) => {
            if !graph_exists(state, &graph) {
                return json_error(StatusCode::NOT_FOUND, &format!("graph <{iri}> does not exist"));
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
async fn serialise_graph(state: &AppState, graph: GraphRef, headers: &HeaderMap, head_only: bool) -> Response {
    let accept = headers.get(header::ACCEPT).and_then(|v| v.to_str().ok());
    let gfmt = negotiate_graph(accept);
    let ct = gfmt.content_type().to_string();
    let config = state.config.clone();
    let budget = make_budget(&config, false);
    // Pinned for the whole dump: the serialisation is consistent with request start.
    let gen = state.current();
    let cfg = config.clone();
    let task = tokio::task::spawn_blocking(move || match graph_dump_triples(gen.snapshot(), &graph, &budget) {
        Ok(triples) => text_response(StatusCode::OK, &ct, serialise_graph_triples(&triples, gfmt), head_only),
        Err(e) => engine_error_response(&e, &cfg),
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
fn graph_dump_triples(graph: &Graph, target: &GraphRef, budget: &QueryBudget) -> Result<Vec<oxrdf::Triple>, String> {
    let select = match target {
        GraphRef::Default => "SELECT ?s ?p ?o WHERE { ?s ?p ?o }".to_string(),
        GraphRef::Named(iri) => format!("SELECT ?s ?p ?o WHERE {{ GRAPH <{}> {{ ?s ?p ?o }} }}", escape_iri(iri)),
    };
    let r = sparq_engine::query_with_budget(graph, &select, budget)?;
    let mut triples = Vec::with_capacity(r.rows.len());
    for row in &r.rows {
        let (Some(s), Some(p), Some(o)) = (&row[0], &row[1], &row[2]) else { continue };
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
    let Term::NamedNode(predicate) = p else { return None };
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
        let (Some(s), Some(p), Some(o)) = (&row[0], &row[1], &row[2]) else { continue };
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
    /// The decompressed image crossed the ratio/absolute ceiling → 413 (zip-bomb guard),
    /// or the gzip stream was malformed → 400.
    TooLarge(String),
    Malformed(String),
}

impl DecodeError {
    fn into_response(self) -> Response {
        match self {
            DecodeError::Unsupported(m) => json_error(StatusCode::UNSUPPORTED_MEDIA_TYPE, &m),
            DecodeError::TooLarge(m) => json_error(StatusCode::PAYLOAD_TOO_LARGE, &m),
            DecodeError::Malformed(m) => bad_request(&m),
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
/// limit (`--max-body-bytes`, 413) still caps the COMPRESSED bytes first, so the absolute
/// decompressed ceiling here is at most `max_body_bytes × max_decompress_ratio`.
fn decode_request_body(body: &Bytes, headers: &HeaderMap, config: &ServerConfig) -> Result<Bytes, DecodeError> {
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
        .map_err(|e| DecodeError::Malformed(format!("malformed gzip body: {e}")))?;
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
fn text_response(status: StatusCode, content_type: &str, body: String, head_only: bool) -> Response {
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
fn chunked_response(status: StatusCode, content_type: &str, chunks: Vec<String>, head_only: bool, pin: PinnedGen) -> Response {
    let len: usize = chunks.iter().map(String::len).sum();
    let body = if head_only {
        axum::body::Body::empty()
    } else {
        axum::body::Body::from_stream(futures_util::stream::iter(
            chunks.into_iter().map(move |c| {
                let _pinned_for_stream_lifetime = &pin;
                Ok::<_, std::convert::Infallible>(Bytes::from(c.into_bytes()))
            }),
        ))
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
fn json_error(status: StatusCode, msg: &str) -> Response {
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
    let bytes = axum::body::to_bytes(body, 64 * 1024).await.unwrap_or_default();
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

fn execution_error(msg: &str) -> Response {
    json_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("query execution error: {msg}"))
}

fn method_not_allowed(allow: &[Method]) -> Response {
    let allow_value = allow.iter().map(|m| m.as_str()).collect::<Vec<_>>().join(", ");
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
        touched_pods(update).into_iter().map(|p| p.as_str().to_string()).collect()
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
        let ins = "INSERT DATA { GRAPH <http://ex/g/A> { <http://ex/s> <http://ex/p> <http://ex/o> } }";
        assert_eq!(pods(ins), ["http://ex/g/A".to_string()].into_iter().collect());
        assert!(!is_global(ins), "a single-graph write must not bump the global pod");
        assert!(!has(ins, "http://ex/g/B"), "a write to A must not touch B");

        let del = "DELETE DATA { GRAPH <http://ex/g/A> { <http://ex/s> <http://ex/p> <http://ex/o> } }";
        assert_eq!(pods(del), ["http://ex/g/A".to_string()].into_iter().collect());
    }

    /// Multiple GRAPH blocks in one operation each contribute their own pod; no global.
    #[test]
    fn multiple_named_graphs_each_get_a_pod() {
        let u = "INSERT DATA { GRAPH <http://ex/g/A> { <http://ex/s> <http://ex/p> <http://ex/o> } \
                              GRAPH <http://ex/g/B> { <http://ex/s> <http://ex/p> <http://ex/o> } }";
        assert_eq!(pods(u), ["http://ex/g/A".to_string(), "http://ex/g/B".to_string()].into_iter().collect());
        assert!(!is_global(u));
    }

    /// A default-graph data op is the catch-all pod (global) — the GSP surface maps every
    /// direct graph onto the default graph, so it cannot be scoped finer.
    #[test]
    fn default_graph_data_op_is_global() {
        let u = "INSERT DATA { <http://ex/s> <http://ex/p> <http://ex/o> }";
        assert!(is_global(u), "a default-graph write must bump the global pod");
        assert_eq!(pods(u), [GLOBAL_POD.to_string()].into_iter().collect());
    }

    /// CLEAR/DROP of a single named graph is scoped; DEFAULT/NAMED/ALL widen to global.
    #[test]
    fn clear_drop_scoping() {
        assert_eq!(pods("CLEAR GRAPH <http://ex/g/A>"), ["http://ex/g/A".to_string()].into_iter().collect());
        assert_eq!(pods("DROP GRAPH <http://ex/g/A>"), ["http://ex/g/A".to_string()].into_iter().collect());
        assert!(is_global("CLEAR DEFAULT"));
        assert!(is_global("CLEAR NAMED"));
        assert!(is_global("CLEAR ALL"));
        assert!(is_global("DROP ALL"));
    }

    /// CREATE touches only the new graph.
    #[test]
    fn create_is_scoped() {
        assert_eq!(pods("CREATE GRAPH <http://ex/g/new>"), ["http://ex/g/new".to_string()].into_iter().collect());
    }

    /// DELETE/INSERT … WHERE with CONCRETE template graphs is scoped to exactly the
    /// graphs the templates WRITE — the WHERE/USING read scope does not widen it.
    #[test]
    fn delete_insert_concrete_templates_are_scoped() {
        let u = "INSERT { GRAPH <http://ex/g/dst> { ?s ?p ?o } } \
                 WHERE  { GRAPH <http://ex/g/src> { ?s ?p ?o } }";
        // Only the write target (dst) is invalidation-relevant; the read source (src) is not.
        assert_eq!(pods(u), ["http://ex/g/dst".to_string()].into_iter().collect());
        assert!(!has(u, "http://ex/g/src"), "the WHERE read source must not be invalidated");
        assert!(!is_global(u));
    }

    /// A VARIABLE graph name in a template (`GRAPH ?g { … }`) makes the written graphs
    /// unknowable at parse time → must fall back to global (never under-invalidate).
    #[test]
    fn delete_insert_variable_graph_target_is_global() {
        let u = "INSERT { GRAPH ?g { ?s ?p ?o } } WHERE { GRAPH ?g { ?s ?p ?o } }";
        assert!(is_global(u), "a dynamically-scoped write target must bump the global pod");
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
        let u = "INSERT DATA { GRAPH <http://ex/g/A> { <http://ex/s> <http://ex/p> <http://ex/o> } \
                              <http://ex/d> <http://ex/p> <http://ex/o> }";
        assert!(has(u, "http://ex/g/A"));
        assert!(is_global(u), "the default-graph half must still bump global");
    }

    /// LOAD into a named graph is scoped; LOAD into the default graph is global. (The LOAD
    /// itself is refused at apply time without an allowlist — extraction is parse-only.)
    #[test]
    fn load_scoping() {
        assert_eq!(
            pods("LOAD <file:///x.ttl> INTO GRAPH <http://ex/g/A>"),
            ["http://ex/g/A".to_string()].into_iter().collect()
        );
        assert!(is_global("LOAD <file:///x.ttl>"), "LOAD into the default graph is global");
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
    use super::{bind_posture, env_truthy, BindPosture};
    use std::net::SocketAddr;

    fn addr(s: &str) -> SocketAddr {
        s.parse().unwrap()
    }

    #[test]
    fn loopback_proceeds_regardless_of_flag() {
        for a in ["127.0.0.1:3030", "127.0.0.5:80", "[::1]:3030"] {
            assert_eq!(bind_posture(&addr(a), false), BindPosture::Loopback, "{a} (no opt-in)");
            assert_eq!(bind_posture(&addr(a), true), BindPosture::Loopback, "{a} (opt-in)");
        }
    }

    #[test]
    fn non_loopback_without_optin_is_refused() {
        // 0.0.0.0 / :: (all-interfaces), an RFC1918 address, a link-local, and the cloud
        // metadata IP all fail closed without --allow-remote.
        for a in ["0.0.0.0:3030", "[::]:3030", "10.0.0.1:8080", "169.254.169.254:80", "192.168.1.5:3030"] {
            match bind_posture(&addr(a), false) {
                BindPosture::RemoteRefused { message } => {
                    assert!(message.contains("refusing to bind"), "{a}: {message}");
                    assert!(message.contains("--allow-remote"), "{a}: must name the opt-in flag");
                    assert!(message.contains("authentication"), "{a}: must explain the no-auth risk");
                }
                other => panic!("{a} must be refused without opt-in, got {other:?}"),
            }
        }
    }

    #[test]
    fn non_loopback_with_optin_warns_but_proceeds() {
        match bind_posture(&addr("0.0.0.0:3030"), true) {
            BindPosture::RemoteAllowed { warning } => {
                assert!(warning.contains("WARNING"), "{warning}");
                assert!(warning.contains("READ AND WRITE"), "{warning}");
                assert!(warning.contains("0.0.0.0:3030"), "must name the address: {warning}");
            }
            other => panic!("opt-in must allow with a warning, got {other:?}"),
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
// [OPUS-4.8] sq-4w18 — SERVICE egress allowlist config wiring tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod hardening_unit_tests {
    //! [OPUS-4.8] (sq-ebii) Pure-logic units for the memory cap (`tighter`) and the
    //! decompression-ratio cap (`decode_request_body`). The HTTP-level 413/503 wiring is
    //! covered by tests/hardening.rs; these pin the boundary math without a server boot.
    use super::{decode_request_body, tighter, ServerConfig};
    use axum::body::Bytes;
    use axum::http::{header, HeaderMap, HeaderValue};

    #[test]
    fn tighter_takes_the_smaller_present_cap() {
        assert_eq!(tighter(None, None), None);
        assert_eq!(tighter(Some(5), None), Some(5));
        assert_eq!(tighter(None, Some(7)), Some(7));
        assert_eq!(tighter(Some(5), Some(7)), Some(5));
        assert_eq!(tighter(Some(9), Some(7)), Some(7));
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
        assert_eq!(decode_request_body(&body, &headers_with_encoding(None), &cfg).unwrap(), body);
        assert_eq!(
            decode_request_body(&body, &headers_with_encoding(Some("identity")), &cfg).unwrap(),
            body
        );
    }

    #[test]
    fn gzip_within_ratio_decodes() {
        let cfg = ServerConfig { max_decompress_ratio: 100, ..ServerConfig::default() };
        let plain = b"some moderately repetitive payload payload payload";
        let body = gz(plain);
        let out = decode_request_body(&body, &headers_with_encoding(Some("gzip")), &cfg).unwrap();
        assert_eq!(&out[..], &plain[..]);
    }

    #[test]
    fn high_ratio_gzip_is_refused() {
        // A 1 MiB run of zeros gzips to a tiny body — ratio far above 2× → refused.
        let cfg = ServerConfig { max_decompress_ratio: 2, max_body_bytes: 1 << 30, ..ServerConfig::default() };
        let body = gz(&vec![0u8; 1 << 20]);
        let err = decode_request_body(&body, &headers_with_encoding(Some("gzip")), &cfg).unwrap_err();
        assert!(matches!(err, super::DecodeError::TooLarge(_)));
    }

    #[test]
    fn ratio_zero_refuses_gzip() {
        let cfg = ServerConfig { max_decompress_ratio: 0, ..ServerConfig::default() };
        let body = gz(b"x");
        let err = decode_request_body(&body, &headers_with_encoding(Some("gzip")), &cfg).unwrap_err();
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
        assert_eq!(e, vec![".internal".to_string(), "sparql.example.org".to_string()]);
    }
}
