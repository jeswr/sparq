//! [SONNET-4.6] (sq-3bz2w, follow-up to sq-u4lgr/#902) OPT-IN **slow-query log** — the
//! server-side policy layer over the engine's `explain_json::SlowQueryRing`.
//!
//! The engine provides the data structure (a bounded ring keeping the N worst-by-wall-time
//! queries, with a serde-free `to_json`); this module decides the POLICY: which requests are
//! recorded, from which measurement, and who may read the result.
//!
//! Compiled ONLY behind the `slow-query-log` feature (which pulls
//! `sparq-engine/explain-json` for the ring type and adds NO new dependency), and armed only
//! when [`ServerConfig::slow_query_threshold`](crate::http::ServerConfig) is also set
//! (`--slow-query-log <MS>` / `SPARQ_SLOW_QUERY_LOG`) — the same double-opt-in as `tpf` /
//! `shacl` / `templates`. With the feature off this module, the config fields, the route and
//! the recording seam are all `#[cfg]`-stripped, so a request pays exactly zero.
//!
//! # Route
//!
//! - `GET /admin/slow-queries` — the retained queries, slowest first, as JSON. **WRITE/admin
//!   auth-gated** (fail-closed), and a `404` when no threshold is configured.
//!
//! # What is measured, and what is NOT
//!
//! This is a deliberately cheap observer, so its honest boundaries matter more than usual:
//!
//! * `totalNanos` is the **already-measured request wall time** — the elapsed time of the
//!   server's own query dispatch, taken by the caller of `SlowQueryLog::record` and handed
//!   in. The query is never re-executed to obtain it (the engine's
//!   `SlowQueryRing::record` would do exactly that, which is why this module uses
//!   `SlowQueryRing::push` instead). For a **streamed** SELECT body the measurement ends when
//!   the response head is produced, so the body serialisation that follows is NOT included —
//!   a streamed query's recorded time is a LOWER BOUND on its full cost.
//! * The retained `plan` is the **planning-only** tree (`sparq_engine::explain_plan`): every node carries
//!   the planner's `estimated` cardinality, and `actual` / `nanos` / `qError` are all `null`.
//!   Filling those in would mean running the query a second time under the operator trace —
//!   precisely the re-execution this seam exists to avoid. The plan therefore answers "what
//!   did the planner decide, and what did it think it would cost", not "where did the time
//!   actually go".
//! * `totalRows` is **always 0 and carries no information**: the row count is not observed at
//!   the response seam (each query form renders its own body). The envelope states this
//!   explicitly as `"rowsObserved": false` so a reader cannot mistake the zero for a measured
//!   empty result.
//!
//! # Privacy boundary (deliberately different from `/queries`)
//!
//! The ring stores the **RAW SPARQL text** of every recorded query, and the route serves it.
//! That is a deliberate departure from the running-query registry (`GET /queries`, sq-qsm5z),
//! which exposes only a non-reversible fingerprint under the #241 info-leak posture: a
//! slow-query view whose queries are fingerprints cannot be acted on. The exposure is bounded
//! three ways — the feature is off by default, the threshold flag is off by default, and the
//! route is gated on the WRITE/admin token rather than the read token (the payload is other
//! tenants' query text, so read access is not sufficient authority to see it). An operator who
//! cannot accept raw query text at rest in server memory should leave this off.

use std::sync::Mutex;
use std::time::Duration;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
};

use sparq_core::Graph;
use sparq_engine::{explain_plan, SlowQuery, SlowQueryRing};

use crate::http::{auth_gate, json_error, text_response, AppState, Operation};

/// The server's slow-query policy: a threshold, a bounded ring, and the lock around it.
///
/// Cloned `AppState` values share one log (it lives behind an `Arc`). The mutex is held only
/// for the ring insert / render — never while the plan is built or engine code runs.
pub struct SlowQueryLog {
    /// Only requests whose measured wall time is `>=` this are recorded. A zero threshold
    /// records every query (and makes every query pay the plan replay) — that is a legitimate
    /// debugging choice, not a mistake, so it is not special-cased.
    threshold: Duration,
    /// The ring's bound, kept alongside it because [`SlowQueryRing`] does not expose it.
    capacity: usize,
    ring: Mutex<SlowQueryRing>,
}

impl SlowQueryLog {
    /// A log retaining the `capacity` slowest queries at or above `threshold`.
    ///
    /// `capacity` is clamped to at least 1 by [`SlowQueryRing::new`].
    pub fn new(capacity: usize, threshold: Duration) -> Self {
        SlowQueryLog {
            threshold,
            capacity: capacity.max(1),
            ring: Mutex::new(SlowQueryRing::new(capacity)),
        }
    }

    /// The configured threshold.
    pub fn threshold(&self) -> Duration {
        self.threshold
    }

    /// Records one finished query request, returning whether it was captured.
    ///
    /// `elapsed` is the time the caller ALREADY measured — the query is never re-run. A
    /// request faster than the threshold returns `false` immediately, before any work; only a
    /// request over it pays the planning replay ([`explain_plan`], a dry run that executes
    /// nothing). A query the planner cannot re-plan (a parse failure, which a query that just
    /// ran should not produce, or a form `explain_plan` rejects) is skipped rather than
    /// recorded without a plan.
    ///
    /// Being captured is not the same as being retained: the ring keeps only the `capacity`
    /// slowest, so a captured query slower than nothing in a full ring is dropped by
    /// [`SlowQueryRing::push`]. `true` means "offered to the ring", which is what the
    /// threshold decision controls.
    pub fn record(&self, graph: &Graph, sparql: &str, elapsed: Duration) -> bool {
        if elapsed < self.threshold {
            return false;
        }
        let plan = match explain_plan(graph, sparql) {
            Ok(p) => p,
            Err(_) => return false,
        };
        let q = SlowQuery {
            sparql: sparql.to_string(),
            plan,
            total_nanos: u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX),
            // Not observed at the response seam — see the module docs. The envelope's
            // `rowsObserved: false` is what tells a reader this zero means "unknown".
            total_rows: 0,
        };
        self.lock().push(q);
        true
    }

    /// The number of retained queries.
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    /// Whether nothing has been retained yet.
    pub fn is_empty(&self) -> bool {
        self.lock().is_empty()
    }

    /// The admin response body: the ring's own JSON array of retained queries (slowest first)
    /// inside an envelope that states the policy and the measurement's boundaries, so a
    /// consumer never has to infer them from the numbers.
    ///
    /// `planKind` is always `"estimated"` and `rowsObserved` always `false` — both are honest
    /// constants describing this recording seam, not runtime state (see the module docs).
    pub fn to_json(&self) -> String {
        let ring = self.lock();
        format!(
            "{{\"thresholdMs\":{},\"capacity\":{},\"retained\":{},\"planKind\":\"estimated\",\"rowsObserved\":false,\"queries\":{}}}",
            self.threshold.as_millis(),
            self.capacity,
            ring.len(),
            ring.to_json()
        )
    }

    /// The ring, recovering from a poisoned lock: a panic while holding it can only have
    /// happened mid-insert, and a partially-updated observability ring is not worth taking the
    /// server down for.
    fn lock(&self) -> std::sync::MutexGuard<'_, SlowQueryRing> {
        self.ring.lock().unwrap_or_else(|p| p.into_inner())
    }
}

/// `GET /admin/slow-queries` — the N slowest recent queries with their plans.
///
/// - `404 Not Found` when no threshold is configured (the feature is compiled in but the
///   operator has not armed it) — the same "compiled but not enabled" shape as `/tpf`.
/// - `200 OK` with the [`SlowQueryLog::to_json`] envelope otherwise.
///
/// **Auth:** WRITE/admin-gated (fail-closed) — it serves raw query text, so the read token is
/// deliberately not sufficient (see the module's privacy boundary).
pub(crate) async fn slow_queries_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if let Some(resp) = auth_gate(state.config(), &headers, Operation::Write) {
        return resp;
    }
    match state.slow_queries() {
        Some(log) => text_response(
            StatusCode::OK,
            "application/json",
            log.to_json(),
            /* head_only */ false,
        ),
        None => json_error(
            StatusCode::NOT_FOUND,
            "slow-query log not enabled (start the server with --slow-query-log <MS>)",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sparq_core::Graph;

    /// A graph with a couple of triples so the planner has something to estimate.
    ///
    /// `sparq_engine::update` is PURE — it borrows the graph and RETURNS the updated one — so
    /// the seeded graph is its return value, not the input. (Discarding it left this fixture
    /// silently empty.)
    fn graph() -> Graph {
        let g = sparq_engine::update(
            &Graph::new(),
            "INSERT DATA { <http://e/a> <http://e/p> <http://e/b> . \
                           <http://e/b> <http://e/p> <http://e/c> }",
        )
        .expect("seed update");
        assert_eq!(g.len(), 2, "the fixture must actually be seeded");
        g
    }

    const Q: &str = "SELECT * WHERE { ?s <http://e/p> ?o }";

    #[test]
    fn below_threshold_is_not_recorded() {
        let g = graph();
        let log = SlowQueryLog::new(4, Duration::from_millis(100));
        assert!(!log.record(&g, Q, Duration::from_millis(99)));
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn at_or_above_threshold_is_recorded() {
        let g = graph();
        let log = SlowQueryLog::new(4, Duration::from_millis(100));
        // The threshold is inclusive: exactly-at is over the line.
        assert!(log.record(&g, Q, Duration::from_millis(100)));
        assert!(log.record(&g, Q, Duration::from_millis(250)));
        assert_eq!(log.len(), 2);
        assert!(!log.is_empty());
    }

    #[test]
    fn zero_threshold_records_every_query() {
        let g = graph();
        let log = SlowQueryLog::new(4, Duration::ZERO);
        assert!(log.record(&g, Q, Duration::ZERO));
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn unplannable_query_is_skipped_not_recorded_planless() {
        let g = graph();
        let log = SlowQueryLog::new(4, Duration::ZERO);
        assert!(!log.record(&g, "SELECT * WHERE { this is not sparql", Duration::ZERO));
        assert!(log.is_empty());
    }

    #[test]
    fn ring_is_bounded_and_keeps_the_slowest() {
        let g = graph();
        let log = SlowQueryLog::new(2, Duration::ZERO);
        for ms in [10u64, 500, 20, 900] {
            assert!(log.record(&g, Q, Duration::from_millis(ms)));
        }
        assert_eq!(log.len(), 2, "capacity bound holds");
        let json = log.to_json();
        let nanos_900 = Duration::from_millis(900).as_nanos();
        let nanos_500 = Duration::from_millis(500).as_nanos();
        let nanos_10 = Duration::from_millis(10).as_nanos();
        assert!(
            json.contains(&format!("\"totalNanos\":{}", nanos_900)),
            "slowest retained: {}",
            json
        );
        assert!(
            json.contains(&format!("\"totalNanos\":{}", nanos_500)),
            "second-slowest retained: {}",
            json
        );
        assert!(
            !json.contains(&format!("\"totalNanos\":{}", nanos_10)),
            "fastest evicted: {}",
            json
        );
    }

    #[test]
    fn envelope_states_the_policy_and_the_measurement_boundaries() {
        let g = graph();
        let log = SlowQueryLog::new(3, Duration::from_millis(250));
        assert!(log.record(&g, Q, Duration::from_millis(400)));
        let json = log.to_json();
        assert!(json.contains("\"thresholdMs\":250"), "{}", json);
        assert!(json.contains("\"capacity\":3"), "{}", json);
        assert!(json.contains("\"retained\":1"), "{}", json);
        // The two honesty constants: the plan is estimates-only and rows are unknown.
        assert!(json.contains("\"planKind\":\"estimated\""), "{}", json);
        assert!(json.contains("\"rowsObserved\":false"), "{}", json);
        // The retained entry carries the raw query text and an estimates-only plan.
        assert!(json.contains("SELECT * WHERE"), "{}", json);
        assert!(json.contains("\"actual\":null"), "{}", json);
        assert!(json.contains("\"qError\":null"), "{}", json);
    }

    #[test]
    fn recorded_nanos_are_the_measurement_handed_in() {
        let g = graph();
        let log = SlowQueryLog::new(2, Duration::ZERO);
        // The query text below would run in microseconds; the ring must report the 7s the
        // caller measured, proving the time is the caller's and not a re-execution's.
        assert!(log.record(&g, Q, Duration::from_secs(7)));
        let json = log.to_json();
        assert!(
            json.contains(&format!("\"totalNanos\":{}", 7_000_000_000u64)),
            "{}",
            json
        );
    }

    #[test]
    fn capacity_is_clamped_to_at_least_one() {
        let g = graph();
        let log = SlowQueryLog::new(0, Duration::ZERO);
        assert!(log.record(&g, Q, Duration::from_millis(1)));
        assert_eq!(log.len(), 1);
    }
}
