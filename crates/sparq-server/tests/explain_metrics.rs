//! Integration tests for T22: the `/sparql` EXPLAIN surface (query parameter and
//! `Accept: text/x-sparq-explain`) and the `/metrics` Prometheus endpoint.
//!
//! [OPUS-4.8] (sq-1b390) Gate the whole suite on the `server` feature. It spins the real axum
//! server and uses the `server`-gated `sparq_server::router` / `AppState` API, so under
//! `--no-default-features --all-targets` (the pure-serialiser-library build) this file must
//! compile OUT — otherwise `clippy --no-default-features --all-targets` breaks on the
//! unresolved axum / serde_json / router imports. 🤖 SPARQ agent.
#![cfg(feature = "server")]

use sparq_core::Graph;
use sparq_server::{router, AppState};
use tokio::net::TcpListener;

const DATA: &str = r#"
    @prefix ex: <http://ex/> .
    ex:alice ex:knows ex:bob ; ex:age 30 ; ex:name "Alice" .
    ex:bob   ex:knows ex:carol ; ex:age 25 ; ex:name "Bob" .
    ex:carol ex:age 35 ; ex:name "Carol" .
"#;

/// Boots the server on a random local port and returns its base URL.
async fn spawn() -> String {
    let graph = Graph::load_str(DATA, "turtle").unwrap();
    let app = router(AppState::new(graph));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

const JOIN_QUERY: &str =
    "PREFIX ex: <http://ex/> SELECT ?a ?b ?age WHERE { ?a ex:knows ?b . ?b ex:age ?age }";

// ---------------------------------------------------------------------------
// EXPLAIN
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_explain_param_returns_plan_text() {
    let base = spawn().await;
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", JOIN_QUERY), ("explain", "true")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.headers()["content-type"]
        .to_str()
        .unwrap()
        .starts_with("text/plain"));
    let body = resp.text().await.unwrap();
    assert!(body.contains("EXPLAIN (SELECT)"), "{body}");
    assert!(body.contains("planning-only"), "{body}");
    // The plan names the join order and strategy — not a result set.
    assert!(body.contains("merge join on ?b"), "{body}");
    assert!(!body.contains("\"bindings\""), "{body}");
}

#[tokio::test]
async fn accept_header_requests_explain() {
    let base = spawn().await;
    let resp = client()
        .post(format!("{base}/sparql"))
        .header("content-type", "application/sparql-query")
        .header("accept", "text/x-sparq-explain")
        .body(JOIN_QUERY)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("EXPLAIN (SELECT)"), "{body}");
    assert!(
        body.contains("BGP [binary join plan: greedy GOO ordering]"),
        "{body}"
    );
}

#[tokio::test]
async fn post_form_explain_analyze_executes_and_traces() {
    let base = spawn().await;
    let resp = client()
        .post(format!("{base}/sparql"))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(format!("query={}&explain=analyze", urlencoding(JOIN_QUERY)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("EXPLAIN ANALYZE (SELECT)"), "{body}");
    assert!(body.contains("Execution trace"), "{body}");
    assert!(body.contains("rows=2"), "{body}");
    assert!(body.contains("Total: 2 result row(s)"), "{body}");
}

#[tokio::test]
async fn explain_of_malformed_query_is_400_and_explain_false_executes() {
    let base = spawn().await;
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT WHERE {"), ("explain", "true")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    // explain=false answers with normal results.
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", JOIN_QUERY), ("explain", "false")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.text().await.unwrap().contains("\"bindings\""));
}

/// Minimal percent-encoding for the form body (space + reserved chars used here).
fn urlencoding(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// /metrics
// ---------------------------------------------------------------------------

#[tokio::test]
async fn metrics_exposes_requests_histogram_and_gauges() {
    let base = spawn().await;
    // One successful query, one malformed (400).
    let r = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", JOIN_QUERY)])
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let r = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT WHERE {")])
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 400);
    let r = client().get(format!("{base}/health")).send().await.unwrap();
    assert_eq!(r.status(), 200);

    let resp = client()
        .get(format!("{base}/metrics"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.headers()["content-type"]
        .to_str()
        .unwrap()
        .starts_with("text/plain"));
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("sparq_http_requests_total{endpoint=\"/sparql\",status=\"200\"} 1"),
        "{body}"
    );
    assert!(
        body.contains("sparq_http_requests_total{endpoint=\"/sparql\",status=\"400\"} 1"),
        "{body}"
    );
    assert!(
        body.contains("sparq_http_requests_total{endpoint=\"/health\",status=\"200\"} 1"),
        "{body}"
    );
    // Histogram: both /sparql requests observed; +Inf equals the count.
    assert!(
        body.contains("sparq_query_duration_seconds_bucket{le=\"+Inf\"} 2"),
        "{body}"
    );
    assert!(
        body.contains("sparq_query_duration_seconds_count 2"),
        "{body}"
    );
    assert!(body.contains("sparq_query_duration_seconds_sum "), "{body}");
    // Gauges read live state: 8 triples in DATA, no subscriptions open.
    assert!(body.contains("sparq_graph_triples 8"), "{body}");
    assert!(body.contains("sparq_active_subscriptions 0"), "{body}");
    assert!(body.contains("sparq_updates_total 0"), "{body}");
}

#[tokio::test]
async fn metrics_counts_updates_and_triple_gauge_follows() {
    let base = spawn().await;
    let resp = client()
        .post(format!("{base}/sparql"))
        .header("content-type", "application/sparql-update")
        .body("INSERT DATA { <http://ex/new> <http://ex/p> <http://ex/o> }")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    let body = client()
        .get(format!("{base}/metrics"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(body.contains("sparq_updates_total 1"), "{body}");
    assert!(body.contains("sparq_graph_triples 9"), "{body}");
    // The update went through /sparql, so the histogram observed it too.
    assert!(
        body.contains("sparq_query_duration_seconds_count 1"),
        "{body}"
    );
}
