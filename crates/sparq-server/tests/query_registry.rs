//! [SONNET-4.6] (sq-qsm5z) Integration tests for the `query-registry` feature's
//! `GET /queries` and `DELETE /queries/{id}` HTTP routes.
//!
//! These exercise the REAL HTTP path (not a mock): boot the server, drive it over
//! HTTP with `reqwest`, and assert:
//!   * both routes are fail-closed on auth — no token => 401, wrong token => 401;
//!   * no token-enumeration: the 401 for a missing token is identical to a wrong token;
//!   * `GET /queries` with the correct READ token => 200 + well-formed `{"queries":[...]}`;
//!   * `DELETE /queries/{id}` with the correct WRITE token => 404 for unknown id.
//!
//! Harness: same pattern as `tests/auth.rs` (spawn_with / reqwest / ephemeral port).
//!
//! Run: `cargo test -p sparq-server --features query-registry --test query_registry`
//!
//! 🤖 SPARQ agent

#![cfg(feature = "query-registry")]

use sparq_core::Graph;
use sparq_server::{router, AppState, ServerConfig};
use tokio::net::TcpListener;

/// The single shared Bearer token — used for both reads (when `auth_token_read=true`)
/// and writes, matching the `ServerConfig::auth_token` / `auth_check` model exactly.
const TOKEN: &str = "s3cr3t-registry-token";
const WRONG: &str = "definitely-not-the-token";

/// Boots a real axum server with a write+read gate (both routes are auth-gated).
/// Returns the base URL.
async fn spawn_gated() -> String {
    let graph = Graph::load_str("", "turtle").unwrap();
    // auth_token: the single token for all operations.
    // auth_token_read: true so that GET /queries (Operation::Read) is also gated.
    let config = ServerConfig {
        auth_token: Some(TOKEN.to_string()),
        auth_token_read: true,
        ..ServerConfig::default()
    };
    let app = router(AppState::with_config(graph, config));
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

// ---------------------------------------------------------------------------
// GET /queries — auth gate
// ---------------------------------------------------------------------------

/// `GET /queries` with NO Authorization header => 401 (fail-closed).
#[tokio::test]
async fn list_queries_no_token_is_401() {
    let base = spawn_gated().await;
    let resp = client()
        .get(format!("{base}/queries"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "GET /queries with no token must be 401"
    );
    // WWW-Authenticate: Bearer is the standard fail-closed signal.
    assert_eq!(
        resp.headers()
            .get("www-authenticate")
            .and_then(|v| v.to_str().ok()),
        Some("Bearer"),
        "GET /queries 401 must carry WWW-Authenticate: Bearer"
    );
}

/// `GET /queries` with the WRONG token => 401 (indistinguishable from missing-token 401,
/// so no enumeration is possible).
#[tokio::test]
async fn list_queries_wrong_token_is_401() {
    let base = spawn_gated().await;
    let resp = client()
        .get(format!("{base}/queries"))
        .header("authorization", format!("Bearer {WRONG}"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "GET /queries with wrong token must be 401 (no enumeration)"
    );
    assert_eq!(
        resp.headers()
            .get("www-authenticate")
            .and_then(|v| v.to_str().ok()),
        Some("Bearer"),
        "wrong-token 401 must carry the same WWW-Authenticate: Bearer"
    );
}

/// `GET /queries` with the correct token => 200 + `{"queries":[...]}` body.
#[tokio::test]
async fn list_queries_correct_token_is_200_with_body() {
    let base = spawn_gated().await;
    let resp = client()
        .get(format!("{base}/queries"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::OK,
        "GET /queries with correct token must be 200"
    );
    // Body must be well-formed JSON with a top-level "queries" array.
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body.get("queries").and_then(|v| v.as_array()).is_some(),
        "response body must be {{\"queries\":[...]}}; got: {body}"
    );
    // On an idle server with no executing queries the list must be empty.
    let queries = body["queries"].as_array().unwrap();
    assert!(
        queries.is_empty(),
        "idle server must report an empty query list"
    );
}

// ---------------------------------------------------------------------------
// DELETE /queries/{id} — auth gate
// ---------------------------------------------------------------------------

/// `DELETE /queries/{id}` with NO token => 401.
#[tokio::test]
async fn cancel_query_no_token_is_401() {
    let base = spawn_gated().await;
    let resp = client()
        .delete(format!("{base}/queries/0000000000000000"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "DELETE /queries/{{id}} with no token must be 401"
    );
    assert_eq!(
        resp.headers()
            .get("www-authenticate")
            .and_then(|v| v.to_str().ok()),
        Some("Bearer"),
        "DELETE 401 must carry WWW-Authenticate: Bearer"
    );
}

/// `DELETE /queries/{id}` with the WRONG token => 401.
#[tokio::test]
async fn cancel_query_wrong_token_is_401() {
    let base = spawn_gated().await;
    let resp = client()
        .delete(format!("{base}/queries/0000000000000000"))
        .header("authorization", format!("Bearer {WRONG}"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "DELETE /queries/{{id}} with wrong token must be 401 (no enumeration)"
    );
}

/// `DELETE /queries/{id}` with the correct WRITE token but an unknown id => 404.
/// This proves the auth gate PASSES and the handler logic runs (non-vacuous: the gate
/// did not short-circuit to 401 for a valid token, and the 404 comes from the registry).
#[tokio::test]
async fn cancel_query_correct_token_unknown_id_is_404() {
    let base = spawn_gated().await;
    let resp = client()
        .delete(format!("{base}/queries/0000000000000000"))
        .header("authorization", format!("Bearer {TOKEN}"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::NOT_FOUND,
        "DELETE /queries/{{id}} with correct token + unknown id must be 404"
    );
}
