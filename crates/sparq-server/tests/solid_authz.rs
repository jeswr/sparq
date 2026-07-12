//! [FABLE-5] sq-snopa.6 (issue #992 FR-4): e2e integration tests for the OPT-IN Solid WAC/ACP HTTP
//! authorization surface (`POST /authz/decide`, `POST /authz/wac-allow`, `POST /authz/query`).
//!
//! These tests run ONLY with the `solid-authz` cargo feature (the whole file is gated). They spin
//! the REAL axum server and assert, over an HTTP request, that the endpoints exercise the REAL
//! `sparq-solid` authoriser path (not a mock) and uphold the load-bearing invariants:
//!   * the OPT-IN posture — `/authz/*` is `404` unless the config flag is set;
//!   * a real allow — an authenticated session with a grant gets `allow:true` + the FR-5
//!     `governingAcl` / `aclLink` provenance;
//!   * FAIL-CLOSED (the soundness invariant) — an anonymous session over the SAME dataset is a
//!     `403` deny; an UNPARSEABLE dataset is a fail-closed `400` (NOT an empty-dataset allow); an
//!     unknown mode is a `403` deny (never a grant);
//!   * `/authz/wac-allow` builds the RFC-style permission advertisement, and a grant-less session
//!     advertises nothing (`user=""`);
//!   * `/authz/query` runs an ACCESS-CONTROLLED query — the SAME query returns the granted rows for
//!     the authorised session and ZERO rows for an anonymous one (fail-closed view);
//!   * read-auth gating (`--auth-token-read`) — `401` without the bearer token, `200`/`403` with it.
#![cfg(feature = "solid-authz")]

use sparq_core::Graph;
use sparq_server::{router, AppState, ServerConfig};
use tokio::net::TcpListener;

/// A minimal boot store — the `/authz/*` endpoints NEVER read it (they authorise over the dataset
/// supplied in the request body), but the server needs a graph to boot.
const BOOT: &str = r#"
    @prefix ex: <http://example.org/> .
    ex:alice a ex:Person .
"#;

/// The pod dataset the tests POST: alice has Read on the root `.acl`, inherited by `notes/n1`.
const WAC_NQUADS: &str = r#"
<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
<https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
"#;

/// Boots a server with the solid-authz flag set as requested, returns its base URL.
async fn spawn_with(authz_on: bool, config: ServerConfig) -> String {
    let graph = Graph::load_str(BOOT, "turtle").unwrap();
    let config = ServerConfig {
        solid_authz: authz_on,
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

async fn spawn(authz_on: bool) -> String {
    spawn_with(authz_on, ServerConfig::default()).await
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

/// The `{dataset, session:{agent}, resource, mode}` decide body for `agent` (or anonymous when
/// `agent` is `None`).
fn decide_body(agent: Option<&str>, resource: &str, mode: &str) -> serde_json::Value {
    let mut session = serde_json::Map::new();
    if let Some(a) = agent {
        session.insert("agent".into(), a.into());
    }
    serde_json::json!({
        "dataset": WAC_NQUADS,
        "session": session,
        "resource": resource,
        "mode": mode,
        "view": "wac",
    })
}

// ---------------------------------------------------------------------------
// OPT-IN posture.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn authz_404_when_flag_off() {
    let base = spawn(false).await;
    for path in ["/authz/decide", "/authz/wac-allow", "/authz/query"] {
        let resp = client()
            .post(format!("{base}{path}"))
            .json(&serde_json::json!({ "dataset": WAC_NQUADS }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 404, "{path} must be 404 with the flag off");
    }
}

// ---------------------------------------------------------------------------
// /authz/decide — a real allow with FR-5 provenance, and fail-closed denies.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn decide_allows_authenticated_grant_with_provenance() {
    let base = spawn(true).await;
    let resp = client()
        .post(format!("{base}/authz/decide"))
        .json(&decide_body(
            Some("https://alice.ex/card#me"),
            "https://pod.ex/notes/n1",
            "read",
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["allow"], serde_json::Value::Bool(true));
    assert_eq!(body["status"], "resolved");
    // FR-5 provenance is in the body (feeds the sq-snopa.7 Link header).
    assert_eq!(body["governingAcl"], "https://pod.ex/.acl");
    assert_eq!(body["aclLink"], r#"<https://pod.ex/.acl>; rel="acl""#);
    assert!(body["grantedModes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m == "read"));
}

/// FAIL-CLOSED: the SAME dataset + resource, but anonymous, is an authoritative `403` deny.
#[tokio::test]
async fn decide_denies_anonymous_fail_closed() {
    let base = spawn(true).await;
    let resp = client()
        .post(format!("{base}/authz/decide"))
        .json(&decide_body(None, "https://pod.ex/notes/n1", "read"))
        .send()
        .await
        .unwrap();
    // A definitive permission deny -> 403 (not a 503, not an allow).
    assert_eq!(resp.status(), 403);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["allow"], serde_json::Value::Bool(false));
    assert_eq!(body["status"], "resolved");
}

/// FAIL-CLOSED: an UNPARSEABLE dataset is a `400`, NOT an empty-dataset allow. This is the
/// load-bearing soundness invariant — an error path in the HTTP layer must DENY.
#[tokio::test]
async fn decide_denies_on_unparseable_dataset() {
    let base = spawn(true).await;
    let resp = client()
        .post(format!("{base}/authz/decide"))
        .json(&serde_json::json!({
            "dataset": "this is @@@ not n-quads",
            "session": { "agent": "https://alice.ex/card#me" },
            "resource": "https://pod.ex/notes/n1",
            "mode": "read",
            "view": "wac",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400, "an unparseable dataset must fail closed, not allow");
}

/// FAIL-CLOSED: an unknown mode is a `403` deny — the HTTP layer never grants an unrecognised mode.
#[tokio::test]
async fn decide_denies_unknown_mode_fail_closed() {
    let base = spawn(true).await;
    let resp = client()
        .post(format!("{base}/authz/decide"))
        .json(&decide_body(
            Some("https://alice.ex/card#me"),
            "https://pod.ex/notes/n1",
            "delete", // not a WAC mode
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["allow"], serde_json::Value::Bool(false));
}

/// A mode the agent lacks (Write) is a `403` deny, not a 503 — an authoritative outcome.
#[tokio::test]
async fn decide_denies_ungranted_mode_403_not_503() {
    let base = spawn(true).await;
    let resp = client()
        .post(format!("{base}/authz/decide"))
        .json(&decide_body(
            Some("https://alice.ex/card#me"),
            "https://pod.ex/notes/n1",
            "write",
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

// ---------------------------------------------------------------------------
// /authz/wac-allow — the RFC-style permission advertisement.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn wac_allow_advertises_granted_modes_and_public() {
    let base = spawn(true).await;
    let resp = client()
        .post(format!("{base}/authz/wac-allow"))
        .json(&serde_json::json!({
            "dataset": WAC_NQUADS,
            "session": { "agent": "https://alice.ex/card#me" },
            "resource": "https://pod.ex/notes/n1",
            "view": "wac",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    // alice holds read; there is no public grant here, so public is empty.
    assert_eq!(body["wacAllow"], r#"user="read",public="""#);
}

/// FAIL-CLOSED: a grant-less (anonymous) session advertises NOTHING (`user=""`).
#[tokio::test]
async fn wac_allow_grantless_advertises_nothing() {
    let base = spawn(true).await;
    let resp = client()
        .post(format!("{base}/authz/wac-allow"))
        .json(&serde_json::json!({
            "dataset": WAC_NQUADS,
            "session": {},
            "resource": "https://pod.ex/notes/n1",
            "view": "wac",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["wacAllow"], r#"user="",public="""#);
}

// ---------------------------------------------------------------------------
// /authz/query — access-controlled query (two sessions, different results).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn query_is_access_controlled_per_session() {
    let base = spawn(true).await;
    let query = "SELECT ?title WHERE { GRAPH ?g { ?s <https://ex.dev/ns#title> ?title } }";

    // The authorised session sees the row.
    let resp = client()
        .post(format!("{base}/authz/query"))
        .json(&serde_json::json!({
            "dataset": WAC_NQUADS,
            "session": { "agent": "https://alice.ex/card#me" },
            "query": query,
            "view": "wac",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let rows = body["results"]["bindings"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "the authorised session sees the granted row");

    // FAIL-CLOSED: the SAME query as anonymous returns ZERO rows (empty view).
    let resp = client()
        .post(format!("{base}/authz/query"))
        .json(&serde_json::json!({
            "dataset": WAC_NQUADS,
            "session": {},
            "query": query,
            "view": "wac",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let rows = body["results"]["bindings"].as_array().unwrap();
    assert_eq!(rows.len(), 0, "an anonymous session sees ZERO rows (fail-closed view)");
}

// ---------------------------------------------------------------------------
// sq-snopa.7 — Link: rel="acl" response header (FR-5).
// The decide and wac-allow endpoints must emit the RFC 8288 Link header value from
// WacDecision::acl_link_header() when a governing ACL was discovered, and MUST NOT emit it
// when none was found (fail-closed — nothing to advertise).
// ---------------------------------------------------------------------------

/// decide emits `Link: <acl-iri>; rel="acl"` when a governing ACL is known.
/// MUTATION SPOT-CHECK: change the expected value below to `<https://pod.ex/.OTHER>; rel="acl"`
/// and this test goes red — confirming it exercises the real header, not a vacuous assertion.
#[tokio::test]
async fn decide_emits_link_header_when_governing_acl_known() {
    // [SONNET-4.6] sq-snopa.7: assert the RFC-8288 Link header is present and correct.
    let base = spawn(true).await;
    let resp = client()
        .post(format!("{base}/authz/decide"))
        .json(&decide_body(
            Some("https://alice.ex/card#me"),
            "https://pod.ex/notes/n1",
            "read",
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let link = resp
        .headers()
        .get("link")
        .expect("decide must emit a Link header when a governing ACL is known")
        .to_str()
        .unwrap();
    assert_eq!(
        link,
        r#"<https://pod.ex/.acl>; rel="acl""#,
        "Link header must be the RFC-8288 link-value for the governing ACL"
    );
}

/// decide does NOT emit a Link header when no governing ACL exists (resource outside the pod).
/// Fail-closed: if there is no ACL document to advertise, nothing is emitted.
#[tokio::test]
async fn decide_omits_link_header_when_no_governing_acl() {
    // [SONNET-4.6] sq-snopa.7: resource with no governing ACL → no Link header.
    let base = spawn(true).await;
    let resp = client()
        .post(format!("{base}/authz/decide"))
        .json(&decide_body(
            Some("https://alice.ex/card#me"),
            "https://other.ex/resource", // not in the WAC dataset — no governing ACL
            "read",
        ))
        .send()
        .await
        .unwrap();
    // The status is a deny (no grant) — but we are testing the HEADER, not the body.
    assert!(
        resp.headers().get("link").is_none(),
        "decide must NOT emit a Link header when no governing ACL was discovered"
    );
}

/// wac-allow emits `Link: <acl-iri>; rel="acl"` when a governing ACL is known.
#[tokio::test]
async fn wac_allow_emits_link_header_when_governing_acl_known() {
    // [SONNET-4.6] sq-snopa.7: wac-allow must also carry the Link header.
    let base = spawn(true).await;
    let resp = client()
        .post(format!("{base}/authz/wac-allow"))
        .json(&serde_json::json!({
            "dataset": WAC_NQUADS,
            "session": { "agent": "https://alice.ex/card#me" },
            "resource": "https://pod.ex/notes/n1",
            "view": "wac",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let link = resp
        .headers()
        .get("link")
        .expect("wac-allow must emit a Link header when a governing ACL is known")
        .to_str()
        .unwrap();
    assert_eq!(
        link,
        r#"<https://pod.ex/.acl>; rel="acl""#,
        "Link header must be the RFC-8288 link-value for the governing ACL"
    );
}

/// wac-allow does NOT emit a Link header when no governing ACL exists.
#[tokio::test]
async fn wac_allow_omits_link_header_when_no_governing_acl() {
    // [SONNET-4.6] sq-snopa.7: resource outside the dataset → no governing ACL → no Link header.
    let base = spawn(true).await;
    let resp = client()
        .post(format!("{base}/authz/wac-allow"))
        .json(&serde_json::json!({
            "dataset": WAC_NQUADS,
            "session": { "agent": "https://alice.ex/card#me" },
            "resource": "https://other.ex/resource", // not in the WAC dataset
            "view": "wac",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert!(
        resp.headers().get("link").is_none(),
        "wac-allow must NOT emit a Link header when no governing ACL was discovered"
    );
}

// ---------------------------------------------------------------------------
// read-auth gating.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn decide_is_read_auth_gated() {
    let config = ServerConfig {
        auth_token: Some("sekret".into()),
        auth_token_read: true,
        ..ServerConfig::default()
    };
    let base = spawn_with(true, config).await;

    // Without the bearer token -> 401.
    let resp = client()
        .post(format!("{base}/authz/decide"))
        .json(&decide_body(
            Some("https://alice.ex/card#me"),
            "https://pod.ex/notes/n1",
            "read",
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // With it -> 200 (the decision then runs).
    let resp = client()
        .post(format!("{base}/authz/decide"))
        .header("Authorization", "Bearer sekret")
        .json(&decide_body(
            Some("https://alice.ex/card#me"),
            "https://pod.ex/notes/n1",
            "read",
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}
