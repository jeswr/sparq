//! The axum HTTP surface: routing, extraction, status/error semantics.
//!
//! Two route groups:
//!   * `/sparql` — the SPARQL 1.1 Protocol `query` operation (GET + the two POST forms).
//!   * Graph Store HTTP Protocol READ — `GET`/`HEAD` on `/graphs/*path` (direct) and on
//!     `/sparql/graph` via `?graph=<uri>` / `?default` (indirect). Write verbs → 501.
//!
//! Shared state is a [`sparq_core::Graph`] behind an `RwLock<Arc<…>>`: queries take a cheap
//! `Arc` snapshot (the read lock is held only for the refcount bump); SPARQL Update goes
//! through the DOUBLE-BUFFERED writer (see [`Writer`]) — the T17 delta-overlay
//! `update_in_place` applied off-line to the spare buffer, published by an atomic `Arc` swap.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
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

use crate::exec::{prepare, PrepareError, QueryForm};
use crate::negotiate::{negotiate, negotiate_graph, Format};
use crate::results;

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
    /// Maximum active subscriptions per WebSocket connection (T23); further `subscribe`
    /// requests on the socket are refused with a protocol error message.
    pub max_subscriptions_per_conn: usize,
    /// Maximum active subscriptions across the whole server (T23).
    pub max_subscriptions: usize,
    /// Fold the update delta-overlay back into a fresh immutable index after this many
    /// in-place update batches have accumulated on a buffer (`0` disables). Keeps scans
    /// near overlay-free speed and bounds overlay/dictionary growth in a long-running
    /// server; each compaction is O(graph), amortised over `compact_every` updates.
    pub compact_every: usize,
    /// Log every request/response via `tower_http::trace::TraceLayer`.
    pub verbose: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            query_timeout: Some(Duration::from_secs(30)),
            max_body_bytes: 1024 * 1024, // 1 MiB
            max_concurrent: 32,
            max_results: None,
            max_subscriptions_per_conn: 16,
            max_subscriptions: 256,
            compact_every: 1024,
            verbose: false,
        }
    }
}

impl ServerConfig {
    /// Defaults overridden by the `SPARQ_QUERY_TIMEOUT` (seconds; `0` disables),
    /// `SPARQ_MAX_BODY_BYTES`, `SPARQ_MAX_CONCURRENT` and `SPARQ_MAX_RESULTS`
    /// environment variables. CLI flags override these in `main`.
    pub fn from_env() -> Self {
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
        if let Some(n) = env_parse::<usize>("SPARQ_MAX_SUBSCRIPTIONS_PER_CONN") {
            cfg.max_subscriptions_per_conn = n;
        }
        if let Some(n) = env_parse::<usize>("SPARQ_MAX_SUBSCRIPTIONS") {
            cfg.max_subscriptions = n;
        }
        if let Some(n) = env_parse::<usize>("SPARQ_COMPACT_EVERY") {
            cfg.compact_every = n;
        }
        cfg
    }
}

fn env_parse<T: std::str::FromStr>(name: &str) -> Option<T> {
    std::env::var(name).ok()?.trim().parse().ok()
}

/// Extra wall-clock allowance past the cooperative deadline before the server gives up
/// awaiting the worker and answers 503 anyway (the detached worker still stops at its
/// next budget check — it is not leaked indefinitely).
pub(crate) const TIMEOUT_GRACE: Duration = Duration::from_secs(2);

/// Shared server state: the dataset under query, published as an `Arc<Graph>` inside an
/// `RwLock` slot. Queries take a cheap `Arc` snapshot (the read lock is held only for the
/// refcount bump — readers never wait on an update beyond the instant of the publish
/// pointer-swap); SPARQL Update goes through the double-buffered [`Writer`]. Also carries
/// the hardening [`ServerConfig`] and the subscription plumbing (T23): a `watch` channel
/// that broadcasts the commit generation to every open `/subscriptions` socket, plus the
/// global active-subscription counters.
#[derive(Clone)]
pub struct AppState {
    graph: Arc<RwLock<Arc<Graph>>>,
    /// Single-writer update state (the double-buffer) — see [`Writer`]. A `std` mutex:
    /// `apply_update` always runs on the blocking pool, and it serialises updates.
    writer: Arc<Mutex<Writer>>,
    config: Arc<ServerConfig>,
    /// Commit generation, bumped after every successful graph swap. Subscription
    /// connections hold a `watch::Receiver` and re-evaluate when it changes; the watch
    /// channel inherently coalesces bursts (see `subscriptions` module docs).
    commits: Arc<tokio::sync::watch::Sender<u64>>,
    /// Global subscription bookkeeping shared by all `/subscriptions` connections.
    pub(crate) subs: Arc<crate::subscriptions::SubscriptionCounters>,
}

/// The single-writer side of the **double-buffered update path** (T17 wiring).
///
/// `Graph` is deliberately not `Clone` (a deep copy of the dictionary arena + six
/// permutation indexes is O(n)), and readers must keep their lock-free `Arc` snapshots —
/// so the writer never mutates a graph a reader could be holding. Instead two physical
/// graphs alternate between the roles *published* and *spare*; an update:
///
/// 1. **reclaims** the spare — the previously published `Arc`, unwrapped once the last
///    reader snapshot of it drops (bounded: snapshots cannot outlive the query budget),
/// 2. **replays** the updates the spare missed while it was published ([`Writer::lag`]),
/// 3. applies the new update via [`sparq_engine::update_in_place`] — the T17
///    delta-overlay path, O(batch) instead of the O(graph) rebuild,
/// 4. **publishes** it with an `Arc` pointer-swap under the write lock (the only instant
///    readers contend), demoting the old published graph to be the next spare with
///    `lag = [this update]`.
///
/// Failed updates stay **atomic** to clients: every mutation happens on the off-line
/// buffer, so the published graph is untouched; the (possibly partially-updated) buffer
/// is discarded and the next update re-materialises it via the rebuild path.
///
/// Cost model: steady-state update latency is microseconds (plus, every
/// [`ServerConfig::compact_every`] batches, one O(graph) overlay fold-back). The price
/// is ~2x graph residency, paid lazily from the first update. The first update (and the
/// first after a discard) is an O(graph) rebuild — it doubles as the materialisation of
/// the second buffer.
struct Writer {
    /// The off-line buffer: the previously published graph, waiting for its last reader
    /// snapshots to drop so it can be reclaimed and mutated. `None` before the first
    /// update and after a failed in-place update (discarded; the next update rebuilds).
    spare: Option<Arc<Graph>>,
    /// Updates committed to the published line that `spare` has not seen yet, in commit
    /// order (replayed before the next update is applied). Non-empty only while `spare`
    /// is `Some`.
    lag: Vec<String>,
    /// In-place batches accumulated on the published / spare graph since each was last
    /// compacted — drives the periodic overlay fold-back. Swapped at publish so each
    /// counter tracks its buffer.
    published_ops: usize,
    spare_ops: usize,
}

/// Poll interval while waiting for the spare buffer's last reader snapshot to drop.
const RECLAIM_POLL: Duration = Duration::from_micros(200);
/// Bound on that wait when no query timeout is configured (snapshot lifetimes are then
/// unbounded); past it the update falls back to the rebuild path.
const RECLAIM_FALLBACK_WAIT: Duration = Duration::from_secs(5);

impl AppState {
    pub fn new(graph: Graph) -> Self {
        Self::with_config(graph, ServerConfig::default())
    }

    pub fn with_config(graph: Graph, config: ServerConfig) -> Self {
        Self {
            graph: Arc::new(RwLock::new(Arc::new(graph))),
            writer: Arc::new(Mutex::new(Writer {
                spare: None,
                lag: Vec::new(),
                published_ops: 0,
                spare_ops: 0,
            })),
            config: Arc::new(config),
            commits: Arc::new(tokio::sync::watch::channel(0).0),
            subs: Arc::new(crate::subscriptions::SubscriptionCounters::default()),
        }
    }

    /// A cheap, consistent snapshot of the current graph for the duration of a request.
    pub fn snapshot(&self) -> Arc<Graph> {
        self.graph.read().expect("graph lock poisoned").clone()
    }

    /// Applies a SPARQL Update through the double-buffered writer (see [`Writer`]):
    /// steady-state, the T17 `update_in_place` delta-overlay path applied to the spare
    /// buffer and published by an atomic `Arc` swap — readers keep their snapshots and
    /// only contend for the instant of the pointer swap. On success, bumps the commit
    /// generation so active subscriptions re-evaluate (T23). The bump happens strictly
    /// *after* the swap, so a woken subscription always snapshots a graph at least as
    /// new as the commit it was woken for. Synchronous and possibly slow (first update /
    /// fallbacks rebuild): call it on the blocking pool.
    pub fn apply_update(&self, sparql: &str) -> Result<(), String> {
        let mut w = self.writer.lock().expect("writer lock poisoned");
        match self.reclaim_spare(&mut w) {
            Some(g) => self.apply_in_place(&mut w, g, sparql),
            None => self.apply_by_rebuild(&mut w, sparql),
        }?;
        self.commits.send_modify(|gen| *gen += 1);
        Ok(())
    }

    /// Takes the spare buffer, waiting (bounded) for outstanding reader snapshots of it
    /// to drop. Snapshots cannot legally outlive the query budget, so the wait is capped
    /// at `query_timeout + TIMEOUT_GRACE` (or [`RECLAIM_FALLBACK_WAIT`] without a
    /// timeout); in practice the spare has usually drained long before the next update.
    /// Returns `None` when there is no spare (first update / after a failure) or a
    /// reader still holds it at the deadline — the caller then rebuilds instead.
    fn reclaim_spare(&self, w: &mut Writer) -> Option<Graph> {
        let mut arc = w.spare.take()?;
        let wait = self.config.query_timeout.map_or(RECLAIM_FALLBACK_WAIT, |t| t + TIMEOUT_GRACE);
        let deadline = std::time::Instant::now() + wait;
        loop {
            match Arc::try_unwrap(arc) {
                Ok(g) => return Some(g),
                Err(still_shared) => {
                    if std::time::Instant::now() >= deadline {
                        // A reader is stuck past its budget: drop our reference (the
                        // memory frees when the reader finishes) and rebuild instead.
                        w.lag.clear();
                        return None;
                    }
                    arc = still_shared;
                    std::thread::sleep(RECLAIM_POLL);
                }
            }
        }
    }

    /// The fast path: replay the spare's lag, apply the new update through the T17
    /// delta-overlay (O(batch)), periodically fold the overlay back, and publish.
    fn apply_in_place(&self, w: &mut Writer, mut g: Graph, sparql: &str) -> Result<(), String> {
        for missed in std::mem::take(&mut w.lag) {
            if sparq_engine::update_in_place(&mut g, &missed).is_err() {
                // Unreachable (the op already succeeded on the published line and the
                // two paths are observationally identical) — but if it ever happens,
                // recover by discarding the buffer and rebuilding from published.
                w.spare_ops = 0;
                return self.apply_by_rebuild(w, sparql);
            }
            w.spare_ops += 1;
        }
        if let Err(e) = sparq_engine::update_in_place(&mut g, sparql) {
            // A failed request may have applied a *prefix* of its operations to `g`.
            // The published graph is untouched (failures stay atomic to clients); the
            // possibly-inconsistent buffer is dropped here and rebuilt lazily.
            w.spare_ops = 0;
            return Err(e);
        }
        w.spare_ops += 1;
        if self.config.compact_every > 0 && w.spare_ops >= self.config.compact_every {
            // Fold the accumulated overlay into a fresh immutable base — O(graph),
            // amortised over `compact_every` updates. Infallible for the in-memory
            // graphs the server hosts (the Err arm is the directory-backed WAL swap).
            g.compact()?;
            w.spare_ops = 0;
        }
        self.publish(w, g, sparql);
        Ok(())
    }

    /// The slow path — the engine's O(graph) rebuild `update` — used for the first
    /// update and whenever the spare buffer is unavailable (stuck reader, prior failed
    /// update). Doubles as the materialisation of the second buffer: the old published
    /// graph is demoted to spare unchanged.
    fn apply_by_rebuild(&self, w: &mut Writer, sparql: &str) -> Result<(), String> {
        let next = sparq_engine::update(&self.snapshot(), sparql)?;
        w.spare_ops = 0; // freshly rebuilt: no overlay
        self.publish(w, next, sparql);
        Ok(())
    }

    /// Atomically swaps `g` into the published slot — the *only* instant readers contend
    /// with the writer — and demotes the old published graph to be the next spare, whose
    /// one missed update is `sparql`. The op counters swap roles with the buffers.
    fn publish(&self, w: &mut Writer, g: Graph, sparql: &str) {
        let old = {
            let mut slot = self.graph.write().expect("graph lock poisoned");
            std::mem::replace(&mut *slot, Arc::new(g))
        };
        w.spare = Some(old);
        w.lag = vec![sparql.to_string()];
        std::mem::swap(&mut w.published_ops, &mut w.spare_ops);
    }

    /// A receiver of the commit generation for a subscription connection (T23).
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
        .route("/graphs/*path", any(graph_store_direct))
        // SEPA-style SPARQL subscriptions over WebSocket (T23).
        .route("/subscriptions", get(crate::subscriptions::subscriptions_endpoint))
        // Liveness.
        .route("/health", get(|| async { "ok" }))
        .with_state(state);
    harden(routes, &config)
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

async fn sparql_endpoint(
    State(state): State<AppState>,
    method: Method,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    match method {
        Method::GET | Method::HEAD => {
            // Query string carries `query=` (+ optional dataset params). Per protocol, a
            // GET without a `query` parameter is a malformed request (400).
            let params = parse_form(raw_query.as_deref().unwrap_or(""));
            match params.get("query") {
                Some(q) => run_query(&state, q, &headers, method == Method::HEAD).await,
                None => bad_request("missing 'query' parameter"),
            }
        }
        Method::POST => handle_post(&state, &headers, &body).await,
        _ => method_not_allowed(&[Method::GET, Method::HEAD, Method::POST]),
    }
}

async fn handle_post(state: &AppState, headers: &HeaderMap, body: &Bytes) -> Response {
    let ct = content_type(headers);
    if ct.starts_with(SPARQL_QUERY_CT) {
        // POST directly — body IS the SPARQL query.
        match std::str::from_utf8(body) {
            Ok(q) => run_query(state, q, headers, false).await,
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
            Some(q) => run_query(state, q, headers, false).await,
            None => bad_request("missing 'query' parameter in url-encoded body"),
        }
    } else if ct.starts_with("application/sparql-update") {
        // SPARQL 1.1 Protocol — update operation (T11b). Body IS the update; success → 204.
        // `apply_update` may block (spare-buffer reclaim) or run long (first-update /
        // fallback rebuild, periodic compaction), so it runs off the async workers too.
        match std::str::from_utf8(body) {
            Ok(u) => {
                let st = state.clone();
                let u = u.to_string();
                let joined = tokio::task::spawn_blocking(move || st.apply_update(&u)).await;
                match joined {
                    Ok(Ok(())) => StatusCode::NO_CONTENT.into_response(),
                    Ok(Err(e)) => bad_request(&format!("update failed: {e}")),
                    Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "update worker panicked"),
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

/// Executes a query string and renders the negotiated representation.
///
/// The engine call is synchronous + CPU-bound, so it runs on the blocking pool under a
/// cooperative [`QueryBudget`] (deadline + SELECT row cap). The await is additionally
/// hard-capped at `timeout + TIMEOUT_GRACE` so a worker stuck in an uninstrumented
/// stretch still gets its 503 on time (the worker itself stops at its next budget check).
async fn run_query(state: &AppState, sparql: &str, headers: &HeaderMap, head_only: bool) -> Response {
    let prepared = match prepare(sparql) {
        Ok(p) => p,
        Err(PrepareError::Malformed(msg)) => return bad_request(&format!("malformed query: {msg}")),
    };
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
            let graph = state.snapshot();
            let cfg = config.clone();
            let task = tokio::task::spawn_blocking(move || {
                if is_ask {
                    match sparq_engine::ask_with_budget(&graph, &select, &budget) {
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
                    render_select(&graph, &select, fmt, head_only, &budget, &cfg)
                }
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
            let graph = state.snapshot();
            let cfg = config.clone();
            let task = tokio::task::spawn_blocking(move || {
                match sparq_engine::construct_ntriples_with_budget(&graph, &query, &budget) {
                    Ok(body) => text_response(StatusCode::OK, gfmt.content_type(), body, head_only),
                    Err(e) => engine_error_response(&e, &cfg),
                }
            });
            await_worker(task, &config).await
        }
    }
}

/// The per-request engine budget: deadline from the configured timeout; the SELECT
/// row cap from `--max-results` (ASK only needs existence, so no row cap there).
pub(crate) fn make_budget(config: &ServerConfig, apply_max_results: bool) -> QueryBudget {
    QueryBudget {
        deadline: config.query_timeout.map(|t| std::time::Instant::now() + t),
        max_rows: if apply_max_results { config.max_results } else { None },
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

/// Runs a SELECT on the engine and serialises it in the negotiated format. JSON uses the
/// engine's direct id→JSON fast path and is **streamed**: the engine returns the body as
/// an ordered chunk sequence (concatenation byte-identical to the single-string form) and
/// the response body hands those chunks to hyper one by one — the peak never holds a
/// second whole-result copy (T16). `Content-Length` is known up front (the chunks are
/// fully evaluated before the response starts), so the response framing is identical to
/// the buffered path. The other formats go through the materialised `QueryResult`.
fn render_select(
    graph: &Graph,
    select: &str,
    fmt: Format,
    head_only: bool,
    budget: &QueryBudget,
    config: &ServerConfig,
) -> Response {
    let ct = fmt.select_content_type();
    let body = match fmt {
        Format::Json => match sparq_engine::query_json_chunks_with_budget(graph, select, budget) {
            Ok(chunks) => return chunked_response(StatusCode::OK, ct, chunks, head_only),
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
        let max = config.max_results.unwrap_or(0);
        return json_error(
            StatusCode::PAYLOAD_TOO_LARGE,
            &format!("result exceeds the server's max-results limit ({max} rows); narrow the query (e.g. add LIMIT) or raise --max-results"),
        );
    }
    execution_error(e)
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

/// Indirect graph identification: `?default` or `?graph=<iri>`.
async fn graph_store_indirect(
    State(state): State<AppState>,
    method: Method,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let is_default = params.contains_key("default");
    let named = params.get("graph").cloned();
    if !is_default && named.is_none() {
        return bad_request("indirect graph identification requires '?default' or '?graph=<uri>'");
    }
    graph_store_read(&state, &method, named.as_deref(), &headers).await
}

/// Direct graph identification: the path IS (a path under) the graph's resource.
async fn graph_store_direct(
    State(state): State<AppState>,
    method: Method,
    axum::extract::Path(path): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Response {
    // The named graph here is the request's effective URI path; with no named-graph store,
    // every direct graph maps onto the default graph (documented limitation).
    let _ = path;
    graph_store_read(&state, &method, None, &headers).await
}

/// Shared GSP handler. READ (GET/HEAD) serialises the (default) graph; write verbs → 501.
async fn graph_store_read(state: &AppState, method: &Method, named: Option<&str>, headers: &HeaderMap) -> Response {
    match *method {
        Method::GET | Method::HEAD => {
            // The engine has no named-graph store: only the default graph is materialised.
            // A request for a *named* graph other than default cannot be served as that
            // graph; we serve the default-graph contents and document the limitation.
            let _ = named;
            let head_only = *method == Method::HEAD;
            serialise_default_graph(state, headers, head_only).await
        }
        Method::PUT | Method::POST | Method::DELETE | Method::PATCH => not_implemented(
            "Graph Store write operations (PUT/POST/DELETE) are not implemented (thread T11b)",
        ),
        _ => method_not_allowed(&[Method::GET, Method::HEAD]),
    }
}

/// Serialises the default graph as N-Triples (the one RDF serialisation we can emit from
/// the engine's term-level scan without a full Turtle writer). Negotiation is minimal:
/// `text/turtle` and `application/n-triples` both yield N-Triples (valid Turtle), default
/// `application/n-triples`. CPU-bound (a full-graph dump), so it runs on the blocking pool
/// under the request timeout (no row cap: a dump is inherently graph-sized).
async fn serialise_default_graph(state: &AppState, headers: &HeaderMap, head_only: bool) -> Response {
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    // N-Triples is a syntactic subset of Turtle, so it satisfies a text/turtle Accept too.
    let ct = if accept.contains("text/turtle") {
        "text/turtle; charset=utf-8"
    } else {
        "application/n-triples; charset=utf-8"
    };
    let ct = ct.to_string();
    let config = state.config.clone();
    let budget = make_budget(&config, false);
    let graph = state.snapshot();
    let cfg = config.clone();
    let task = tokio::task::spawn_blocking(move || match nt_dump(&graph, &budget) {
        Ok(body) => text_response(StatusCode::OK, &ct, body, head_only),
        Err(e) => engine_error_response(&e, &cfg),
    });
    await_worker(task, &config).await
}

/// Dumps the whole default graph as N-Triples by reusing the engine's SELECT path
/// (`?s ?p ?o`) and the materialised terms — no private store API needed.
fn nt_dump(graph: &Graph, budget: &QueryBudget) -> Result<String, String> {
    let r = sparq_engine::query_with_budget(graph, "SELECT ?s ?p ?o WHERE { ?s ?p ?o }", budget)?;
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
fn chunked_response(status: StatusCode, content_type: &str, chunks: Vec<String>, head_only: bool) -> Response {
    let len: usize = chunks.iter().map(String::len).sum();
    let body = if head_only {
        axum::body::Body::empty()
    } else {
        axum::body::Body::from_stream(futures_util::stream::iter(
            chunks.into_iter().map(|c| Ok::<_, std::convert::Infallible>(Bytes::from(c.into_bytes()))),
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

fn not_implemented(msg: &str) -> Response {
    json_error(StatusCode::NOT_IMPLEMENTED, msg)
}

fn method_not_allowed(allow: &[Method]) -> Response {
    let allow_value = allow.iter().map(|m| m.as_str()).collect::<Vec<_>>().join(", ");
    Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .header(header::ALLOW, allow_value)
        .body(axum::body::Body::from("method not allowed"))
        .unwrap()
}
