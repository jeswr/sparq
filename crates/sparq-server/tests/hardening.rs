//! Integration tests for the T15 hardening guards: per-request timeout, body-size limit,
//! concurrency shedding, panic recovery and the SELECT row cap. Each test boots the real
//! hardened router on an ephemeral port and drives it over HTTP, asserting both the HTTP
//! status semantics and the structured JSON error bodies.

use std::time::Duration;

use sparq_core::Graph;
use sparq_server::{harden, router, AppState, ServerConfig};
use tokio::net::TcpListener;

const DATA: &str = r#"
    @prefix ex: <http://ex/> .
    ex:alice ex:knows ex:bob ; ex:age 30 ; ex:name "Alice" .
    ex:bob   ex:age 25 ; ex:name "Bob"@en .
    ex:carol ex:age 35 .
"#;

/// A dense graph whose 3-way disconnected pattern forces a cross-product far too large
/// to ever finish — the deliberately slow query for the timeout / shedding tests.
fn dense_graph_ttl() -> String {
    let n = 80; // 80×80 = 6 400 edges; (6 400)^3 cross-product rows ≈ 2.6e11 — never finishes
    let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
    for i in 0..n {
        for j in 0..n {
            ttl.push_str(&format!("ex:n{i} ex:e ex:n{j} .\n"));
        }
    }
    ttl
}

const SLOW_QUERY: &str =
    "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?c ex:e ?d . ?e ex:e ?f }";

async fn serve(app: axum::Router) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

async fn spawn_with(ttl: &str, config: ServerConfig) -> String {
    let graph = Graph::load_str(ttl, "turtle").unwrap();
    serve(router(AppState::with_config(graph, config))).await
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

// ---------------------------------------------------------------------------
// (1) Request timeout — slow query → 503, worker stopped cooperatively
// ---------------------------------------------------------------------------

#[tokio::test]
async fn timeout_fires_on_slow_query() {
    let config = ServerConfig { query_timeout: Some(Duration::from_secs(1)), ..ServerConfig::default() };
    let base = spawn_with(&dense_graph_ttl(), config).await;

    let started = std::time::Instant::now();
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", SLOW_QUERY)])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 503);
    // 1s budget deadline + 2s hard grace; anything beyond means the guard didn't fire.
    assert!(started.elapsed() < Duration::from_secs(5), "503 took {:?}", started.elapsed());
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("{\"error\":"), "structured JSON error, got: {body}");
    assert!(body.contains("timed out"), "got: {body}");
}

#[tokio::test]
async fn fast_query_unaffected_by_timeout() {
    let config = ServerConfig { query_timeout: Some(Duration::from_secs(1)), ..ServerConfig::default() };
    let base = spawn_with(DATA, config).await;
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT ?s WHERE { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.text().await.unwrap().contains("\"bindings\""));
}

// ---------------------------------------------------------------------------
// (2a) Body-size limit → 413
// ---------------------------------------------------------------------------

#[tokio::test]
async fn oversized_body_is_413() {
    let config = ServerConfig { max_body_bytes: 256, ..ServerConfig::default() };
    let base = spawn_with(DATA, config).await;
    // A syntactically valid (huge) query body over the limit — rejected before parsing.
    let big = format!("SELECT * WHERE {{ ?s ?p ?o }} # {}", "x".repeat(1024));
    let resp = client()
        .post(format!("{base}/sparql"))
        .header("content-type", "application/sparql-query")
        .body(big)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 413);
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("{\"error\":"), "structured JSON error, got: {body}");

    // Under the limit still works.
    let resp = client()
        .post(format!("{base}/sparql"))
        .header("content-type", "application/sparql-query")
        .body("SELECT * WHERE { ?s ?p ?o }")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

// ---------------------------------------------------------------------------
// (2b) Concurrency limit + load shed → 429
// ---------------------------------------------------------------------------

#[tokio::test]
async fn concurrency_limit_sheds_with_429() {
    let config = ServerConfig {
        max_concurrent: 1,
        query_timeout: Some(Duration::from_secs(3)),
        ..ServerConfig::default()
    };
    let base = spawn_with(&dense_graph_ttl(), config).await;

    // Occupy the single slot with the slow query…
    let base2 = base.clone();
    let slow = tokio::spawn(async move {
        client().get(format!("{base2}/sparql")).query(&[("query", SLOW_QUERY)]).send().await
    });
    tokio::time::sleep(Duration::from_millis(300)).await;

    // …then any further request is shed immediately.
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT * WHERE { ?s ?p ?o } LIMIT 1")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 429);
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("{\"error\":"), "structured JSON error, got: {body}");

    // The occupying request still terminates via the timeout guard (503).
    let slow_resp = slow.await.unwrap().unwrap();
    assert_eq!(slow_resp.status(), 503);
}

// ---------------------------------------------------------------------------
// (2c/3) --max-results row cap → honest 413 refusal, never truncation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn max_results_refuses_with_413() {
    let config = ServerConfig { max_results: Some(2), ..ServerConfig::default() };
    let base = spawn_with(DATA, config).await;

    // 3 matching rows > cap of 2 → refused, with the limit named in the error.
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT ?s WHERE { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 413);
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("{\"error\":"), "structured JSON error, got: {body}");
    assert!(body.contains("max-results"), "got: {body}");
    assert!(!body.contains("bindings"), "must refuse, not truncate: {body}");

    // Within the cap → normal result (and LIMIT keeps queries under the cap usable).
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT ?s WHERE { ?s <http://ex/age> ?a } LIMIT 2")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap().matches("\"type\":\"uri\"").count(), 2);

    // The cap applies to every SELECT serialisation, not just JSON.
    let resp = client()
        .get(format!("{base}/sparql"))
        .header("accept", "text/csv")
        .query(&[("query", "SELECT ?s WHERE { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 413);

    // ASK is existence-only and ignores the row cap.
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "ASK { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.text().await.unwrap().contains("true"));
}

// ---------------------------------------------------------------------------
// (4) Panic → 500 (CatchPanicLayer), connection stays alive
// ---------------------------------------------------------------------------

#[tokio::test]
async fn handler_panic_is_500_not_dead_connection() {
    // Wrap a deliberately panicking route in the exact production middleware stack.
    async fn panicking() -> &'static str {
        panic!("boom")
    }
    let app = harden(
        axum::Router::new().route("/panic", axum::routing::get(panicking)),
        &ServerConfig::default(),
    );
    let base = serve(app).await;

    let cl = client();
    let resp = cl.get(format!("{base}/panic")).send().await.unwrap();
    assert_eq!(resp.status(), 500);
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("{\"error\":"), "structured JSON error, got: {body}");

    // The server (and the client's pooled connection) survives the panic.
    let resp = cl.get(format!("{base}/panic")).send().await.unwrap();
    assert_eq!(resp.status(), 500);
}

// ---------------------------------------------------------------------------
// Structured JSON error bodies on the ordinary error paths too
// ---------------------------------------------------------------------------

#[tokio::test]
async fn errors_are_structured_json() {
    let base = spawn_with(DATA, ServerConfig::default()).await;

    // 400 malformed query.
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT WHERE {")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("{\"error\":"), "got: {body}");

    // 405 keeps its Allow header after JSON normalisation.
    let resp = client()
        .request(reqwest::Method::DELETE, format!("{base}/sparql"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 405);
    assert!(resp.headers().contains_key("allow"));
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("{\"error\":"), "got: {body}");
}
