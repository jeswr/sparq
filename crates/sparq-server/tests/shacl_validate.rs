//! [OPUS-4.8] sq-r868 (from-pss gh-162 follow-up (c)): e2e integration tests for the
//! OPT-IN HTTP SHACL validate endpoint (`POST /shacl/validate`).
//!
//! These tests run ONLY with the `shacl` cargo feature (the whole file is gated). They spin
//! the real axum server and assert, over an HTTP request:
//!   * the OPT-IN posture — `/shacl/validate` is `404` unless the config flag is set;
//!   * the server's loaded DATA graph is validated against the POSTed SHACL SHAPES graph;
//!   * a conforming store yields `{ "conforms": true, "results": [] }`;
//!   * a violating store yields `conforms:false` with the PSS-consumed JSON report shape
//!     (`focusNode` / `path` / `value` / `sourceShape` / `sourceConstraintComponent` /
//!     `severity` / `message`);
//!   * content negotiation — JSON by default, the W3C report Turtle on `Accept: text/turtle`;
//!   * a malformed shapes body is a 400, an unsupported media type a 415;
//!   * the wrong HTTP method is a 405;
//!   * read-auth gating (`--auth-token-read`) — 401 without the bearer token, 200 with it.
#![cfg(feature = "shacl")]

use sparq_core::Graph;
use sparq_server::{router, AppState, ServerConfig};
use tokio::net::TcpListener;

/// A store with one `ex:Person` whose `ex:age` is a string (violates `xsd:integer`).
const VIOLATING_DATA: &str = r#"
    @prefix ex: <http://example.org/> .
    ex:alice a ex:Person ; ex:age "thirty" .
"#;

/// A store with one `ex:Person` whose `ex:age` is a valid integer.
const CONFORMING_DATA: &str = r#"
    @prefix ex: <http://example.org/> .
    ex:bob a ex:Person ; ex:age 42 .
"#;

const SHAPES: &str = r#"
    @prefix sh:  <http://www.w3.org/ns/shacl#> .
    @prefix ex:  <http://example.org/> .
    @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    ex:PersonShape a sh:NodeShape ;
      sh:targetClass ex:Person ;
      sh:property [ sh:path ex:age ; sh:datatype xsd:integer ; sh:minCount 1 ] .
"#;

/// Boots a server holding `data` with the SHACL flag set, returns its base URL.
async fn spawn_with(data: &str, shacl_on: bool, config: ServerConfig) -> String {
    let graph = Graph::load_str(data, "turtle").unwrap();
    let config = ServerConfig {
        shacl: shacl_on,
        ..config
    };
    let app = router(AppState::with_config(graph, config));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

async fn spawn(data: &str, shacl_on: bool) -> String {
    spawn_with(data, shacl_on, ServerConfig::default()).await
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

// ---------------------------------------------------------------------------
// OPT-IN posture.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn shacl_404_when_flag_off() {
    let base = spawn(VIOLATING_DATA, false).await;
    let resp = client()
        .post(format!("{base}/shacl/validate"))
        .header("Content-Type", "text/turtle")
        .body(SHAPES)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

// ---------------------------------------------------------------------------
// Validation verdicts (JSON report — the PSS-consumed shape).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn violating_store_reports_violation_json() {
    let base = spawn(VIOLATING_DATA, true).await;
    let resp = client()
        .post(format!("{base}/shacl/validate"))
        .header("Content-Type", "text/turtle")
        .body(SHAPES)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()["content-type"],
        "application/json; charset=utf-8"
    );
    let body = resp.text().await.unwrap();
    assert!(body.contains("\"conforms\":false"), "{body}");
    // The PSS-consumed JSON projection fields.
    assert!(body.contains("\"focusNode\":"), "{body}");
    assert!(body.contains("\"sourceConstraintComponent\":"), "{body}");
    assert!(
        body.contains("DatatypeConstraintComponent"),
        "expected the datatype violation: {body}"
    );
    assert!(body.contains("http://example.org/alice"), "{body}");
}

#[tokio::test]
async fn conforming_store_reports_conformance_json() {
    let base = spawn(CONFORMING_DATA, true).await;
    let resp = client()
        .post(format!("{base}/shacl/validate"))
        .header("Content-Type", "text/turtle")
        .body(SHAPES)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("\"conforms\":true"), "{body}");
    assert!(body.contains("\"results\":[]"), "{body}");
}

// ---------------------------------------------------------------------------
// Content negotiation — W3C report Turtle.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn turtle_report_on_accept_turtle() {
    let base = spawn(VIOLATING_DATA, true).await;
    let resp = client()
        .post(format!("{base}/shacl/validate"))
        .header("Content-Type", "text/turtle")
        .header("Accept", "text/turtle")
        .body(SHAPES)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.headers()["content-type"], "text/turtle; charset=utf-8");
    let body = resp.text().await.unwrap();
    // The W3C SHACL report vocabulary.
    assert!(body.contains("ValidationReport"), "{body}");
    assert!(body.contains("conforms"), "{body}");
    assert!(body.contains("ValidationResult"), "{body}");
}

// ---------------------------------------------------------------------------
// Error handling.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn malformed_shapes_body_is_400() {
    let base = spawn(VIOLATING_DATA, true).await;
    let resp = client()
        .post(format!("{base}/shacl/validate"))
        .header("Content-Type", "text/turtle")
        .body("this is not valid turtle @@@")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn unsupported_media_type_is_415() {
    let base = spawn(VIOLATING_DATA, true).await;
    let resp = client()
        .post(format!("{base}/shacl/validate"))
        .header("Content-Type", "application/pdf")
        .body(SHAPES)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 415);
}

#[tokio::test]
async fn get_is_405() {
    let base = spawn(VIOLATING_DATA, true).await;
    let resp = client()
        .get(format!("{base}/shacl/validate"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 405);
}

// ---------------------------------------------------------------------------
// Read-auth gating — validation is a READ over the store, gated like any GET.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn read_auth_gates_the_endpoint() {
    let config = ServerConfig {
        auth_token: Some("s3cret".to_string()),
        auth_token_read: true,
        ..ServerConfig::default()
    };
    let base = spawn_with(VIOLATING_DATA, true, config).await;

    // No token → 401.
    let resp = client()
        .post(format!("{base}/shacl/validate"))
        .header("Content-Type", "text/turtle")
        .body(SHAPES)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // Correct token → 200.
    let resp = client()
        .post(format!("{base}/shacl/validate"))
        .header("Content-Type", "text/turtle")
        .header("Authorization", "Bearer s3cret")
        .body(SHAPES)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}
