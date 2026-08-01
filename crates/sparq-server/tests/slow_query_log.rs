//! [SONNET-4.6] (sq-3bz2w, follow-up to sq-u4lgr/#902) Integration tests for the
//! `slow-query-log` feature: the recording seam on the real query path and the gated
//! `GET /admin/slow-queries` route.
//!
//! These drive the REAL HTTP path (boot the router, talk to it with `reqwest`) rather than
//! calling the recorder directly, so they pin the WIRING — that a `/sparql` request actually
//! reaches the ring — not just the policy struct (which `src/slow_query.rs`'s unit tests
//! cover). They assert:
//!   * disarmed (no `--slow-query-log`) => `404`, even though the feature is compiled in;
//!   * the route is WRITE/admin-gated even when READS are ungated (the deliberate posture:
//!     the body is raw query text);
//!   * an armed server records a real `/sparql` query, with a real measured `totalNanos`;
//!   * a threshold ABOVE the query's cost records nothing (the threshold is load-bearing);
//!   * the envelope states the measurement's boundaries (`planKind` / `rowsObserved`).
//!
//! Run: `cargo test -p sparq-server --features slow-query-log --test slow_query_log`
//!
//! 🤖 SPARQ agent

#![cfg(feature = "slow-query-log")]

use std::time::Duration;

use sparq_core::Graph;
use sparq_server::{router, AppState, ServerConfig};
use tokio::net::TcpListener;

const TOKEN: &str = "s3cr3t-admin-token";
const WRONG: &str = "definitely-not-the-token";

/// Two triples so the query below has something to match and the planner something to size.
const TURTLE: &str = "<http://e/a> <http://e/p> <http://e/b> . <http://e/b> <http://e/p> <http://e/c> .";

/// The query every recording test issues. Distinctive enough to find verbatim in the log.
const QUERY: &str = "SELECT * WHERE { ?s <http://e/p> ?o }";

async fn spawn(config: ServerConfig) -> String {
    let graph = Graph::load_str(TURTLE, "turtle").unwrap();
    let app = router(AppState::with_config(graph, config));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

/// An ungated server with the log armed at `threshold`.
async fn spawn_armed(threshold: Duration) -> String {
    spawn(ServerConfig {
        slow_query_threshold: Some(threshold),
        slow_query_capacity: 8,
        ..ServerConfig::default()
    })
    .await
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

/// Runs `QUERY` over the real `/sparql` GET path and asserts it succeeded.
async fn run_query(base: &str) {
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", QUERY)])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK, "the query itself must succeed");
}

/// The slow-query log body (asserted 200 first).
async fn slow_queries(base: &str) -> String {
    let resp = client()
        .get(format!("{base}/admin/slow-queries"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK, "armed log must answer 200");
    resp.text().await.unwrap()
}

// ---------------------------------------------------------------------------
// Double opt-in: compiled in, but disarmed
// ---------------------------------------------------------------------------

/// With the feature compiled in but NO threshold configured, the route is a clean 404 — the
/// feature alone is not the opt-in.
#[tokio::test]
async fn disarmed_log_is_404() {
    let base = spawn(ServerConfig::default()).await;
    let resp = client()
        .get(format!("{base}/admin/slow-queries"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::NOT_FOUND,
        "no --slow-query-log => 404"
    );
}

// ---------------------------------------------------------------------------
// Auth: WRITE/admin-gated, not read-gated
// ---------------------------------------------------------------------------

/// The route demands the WRITE token even in the default posture where READS are ungated —
/// the whole point of gating it on `Operation::Write`: it serves other callers' raw query text.
#[tokio::test]
async fn route_is_write_gated_even_when_reads_are_open() {
    let base = spawn(ServerConfig {
        slow_query_threshold: Some(Duration::ZERO),
        auth_token: Some(TOKEN.to_string()),
        // Reads are NOT gated in this posture — `/sparql` is open to anyone...
        auth_token_read: false,
        ..ServerConfig::default()
    })
    .await;
    // ...proven here: an unauthenticated read succeeds.
    run_query(&base).await;
    // ...but the slow-query view still refuses without the admin token.
    for token in [None, Some(WRONG)] {
        let mut req = client().get(format!("{base}/admin/slow-queries"));
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        let resp = req.send().await.unwrap();
        assert_eq!(
            resp.status(),
            reqwest::StatusCode::UNAUTHORIZED,
            "missing/wrong admin token must be 401 (token={:?})",
            token
        );
    }
    // The correct WRITE token gets in.
    let resp = client()
        .get(format!("{base}/admin/slow-queries"))
        .bearer_auth(TOKEN)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    assert!(
        resp.text().await.unwrap().contains(QUERY),
        "the query run above must have been recorded"
    );
}

// ---------------------------------------------------------------------------
// The recording seam
// ---------------------------------------------------------------------------

/// An armed server records a query issued over the REAL `/sparql` path, with the wall time the
/// server itself measured. This is the wiring test: nothing here touches `SlowQueryLog`.
#[tokio::test]
async fn armed_log_records_a_real_query_with_a_real_measurement() {
    let base = spawn_armed(Duration::ZERO).await;
    assert!(
        slow_queries(&base).await.contains("\"queries\":[]"),
        "nothing recorded before any query runs"
    );
    run_query(&base).await;
    let body = slow_queries(&base).await;
    assert!(body.contains(QUERY), "raw query text is retained: {}", body);
    assert!(body.contains("\"retained\":1"), "exactly one entry: {}", body);
    // A real measurement, not a placeholder: the recorded time is non-zero.
    let nanos = recorded_nanos(&body);
    assert!(nanos > 0, "totalNanos must be the measured wall time, got {}", nanos);
}

/// A threshold the query cannot reach records NOTHING. This is the guard on the threshold
/// comparison itself: with the comparison removed or inverted, this test goes red.
#[tokio::test]
async fn threshold_above_the_query_cost_records_nothing() {
    // A minute: no `SELECT * WHERE { ?s <p> ?o }` over two triples reaches it.
    let base = spawn_armed(Duration::from_secs(60)).await;
    run_query(&base).await;
    run_query(&base).await;
    let body = slow_queries(&base).await;
    assert!(
        body.contains("\"retained\":0") && body.contains("\"queries\":[]"),
        "below-threshold queries must not be recorded: {}",
        body
    );
    assert!(
        !body.contains(QUERY),
        "no query text may leak into a log that recorded nothing: {}",
        body
    );
}

/// The envelope states the policy and the two honesty constants, so a consumer never has to
/// infer from the numbers that the plan is estimates-only and the row count is unknown.
#[tokio::test]
async fn envelope_declares_policy_and_measurement_boundaries() {
    let base = spawn_armed(Duration::from_millis(0)).await;
    run_query(&base).await;
    let body = slow_queries(&base).await;
    assert!(body.contains("\"thresholdMs\":0"), "{}", body);
    assert!(body.contains("\"capacity\":8"), "{}", body);
    assert!(body.contains("\"planKind\":\"estimated\""), "{}", body);
    assert!(body.contains("\"rowsObserved\":false"), "{}", body);
    // The recorded plan is the planning-only tree: estimates present, actuals absent.
    assert!(body.contains("\"actual\":null"), "no actuals (never re-executed): {}", body);
    assert!(body.contains("\"qError\":null"), "no q-error without actuals: {}", body);
    // `totalRows` is the documented always-0 placeholder that `rowsObserved:false` explains.
    assert!(body.contains("\"totalRows\":0"), "{}", body);
}

/// Extracts the first `"totalNanos":<n>` from a log body.
fn recorded_nanos(body: &str) -> u64 {
    let start = body
        .find("\"totalNanos\":")
        .map(|i| i + "\"totalNanos\":".len())
        .unwrap_or_else(|| panic!("no totalNanos in {}", body));
    let rest = &body[start..];
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().expect("totalNanos is a number")
}
