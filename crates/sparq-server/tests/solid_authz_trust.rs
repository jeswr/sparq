//! [SONNET-4.6] sq-pfae.17 — integration tests for the OPT-IN stateless trust-graph extension
//! to `POST /authz/decide` (the `solid-authz-trust` cargo feature).
//!
//! These tests run ONLY with the `solid-authz-trust` cargo feature (the whole file is gated).
//! They spin the REAL axum server and assert the load-bearing invariants:
//!   * DOUBLE-OPT-IN posture: the trust extension is only active when BOTH (a) the feature is
//!     compiled AND (b) `ServerConfig::solid_authz_trust == true`. Without the flag the endpoint
//!     falls through to the unchanged `solid-authz` path (tested in tests/solid_authz.rs).
//!   * FAIL-CLOSED on malformed trust block: a request with a `"trust"` block that is not a
//!     JSON object => 403 deny, never 500/allow.
//!   * FAIL-CLOSED on missing required fields in the trust block: missing `agentIri` /
//!     `nowUnixSecs` / `rules` / `credentials` => 403 deny.
//!   * FAIL-CLOSED on invalid IRI in trust rule: `"source"` that is not a valid IRI => 403 deny.
//!   * FAIL-CLOSED on missing credentials: an empty `"credentials"` array + a valid rule set =>
//!     the admission gate admits NOTHING => the underlying WAC decision falls through (deny for
//!     the anonymous session — fail-closed, never opens).
//!   * Feature ON + trust flag OFF: a `"trust"` block is silently IGNORED and the endpoint
//!     behaves byte-identically to the `solid-authz` path (the double-opt-in contract).
//!   * The `trustDenied` field is present and `true` in every fail-closed trust deny response.
//!
//! The test DOES NOT attempt to construct a valid cryptographic signature — that would require
//! the ZK key-gen primitives in the integration test, which is out of scope for a server test.
//! What IS tested is the full plumbing: the request parsing, the fail-closed gates, the double-
//! opt-in config flag, and the JSON response shape. The cryptographic path is covered by
//! `sparq-trust`'s own unit/integration tests (`tests/` under that crate).
//!
//! HONEST SCOPE: the trust extension is a clear-path admission mechanism (anchored-not-proven);
//! it is NOT a ZK/MPC or unlinkability claim (sq-qhy4 pending external audit).
//!
//! [SONNET-4.6] sq-pfae.17. POSITIONAL format args used throughout (CodeQL guard).
#![cfg(feature = "solid-authz-trust")]

use sparq_core::Graph;
use sparq_server::{router, AppState, ServerConfig};
use tokio::net::TcpListener;

/// A minimal boot graph — the `/authz/decide` endpoint NEVER reads it (it authorises over the
/// dataset in the request body), but the server needs a graph to boot.
const BOOT: &str = r#"
    @prefix ex: <http://example.org/> .
    ex:boot a ex:Graph .
"#;

/// A minimal WAC dataset: alice has Read on the root container, inherited by all children.
const WAC_NQUADS: &str = r#"
<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
<https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
"#;

/// Boot the server with a given config. Returns the base URL.
async fn spawn_with_config(config: ServerConfig) -> String {
    let graph = Graph::load_str(BOOT, "turtle").unwrap();
    let app = router(AppState::with_config(graph, config));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

/// Boot with both `solid_authz` and `solid_authz_trust` ON.
async fn spawn_trust_on() -> String {
    spawn_with_config(ServerConfig {
        solid_authz: true,
        solid_authz_trust: true,
        ..ServerConfig::default()
    })
    .await
}

/// Boot with `solid_authz` ON but `solid_authz_trust` OFF (trust flag not set).
async fn spawn_trust_flag_off() -> String {
    spawn_with_config(ServerConfig {
        solid_authz: true,
        solid_authz_trust: false,
        ..ServerConfig::default()
    })
    .await
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

/// A well-formed `decide` body WITH a `"trust"` block. The trust block carries a valid JSON
/// structure but a deliberately INVALID issuer key hex — so the admission gate fails closed
/// (the key cannot parse) and returns a 403. This exercises the full trust-parse path.
fn decide_body_with_invalid_trust(resource: &str) -> serde_json::Value {
    serde_json::json!({
        "dataset": WAC_NQUADS,
        "session": { "agent": "https://alice.ex/card#me" },
        "resource": resource,
        "mode": "read",
        "view": "wac",
        "trust": {
            "agentIri": "https://alice.ex/card#me",
            "nowUnixSecs": 1_720_000_000_i64,
            "rules": [
                {
                    "source": "https://issuer.example/",
                    "issuerKeyHex": "not-a-valid-hex-key",
                    "scopeIri": "https://pod.ex/",
                    "freshWithinSecs": 86400_i64,
                    "shapePredicateIri": "https://schema.org/age"
                }
            ],
            "certifications": [],
            "credentials": []
        }
    })
}

/// A body with a `"trust"` block that is NOT a JSON object (malformed).
fn decide_body_trust_not_object(resource: &str) -> serde_json::Value {
    serde_json::json!({
        "dataset": WAC_NQUADS,
        "session": { "agent": "https://alice.ex/card#me" },
        "resource": resource,
        "mode": "read",
        "view": "wac",
        "trust": "this-is-not-an-object"
    })
}

/// A body with a `"trust"` block missing `agentIri`.
fn decide_body_trust_missing_agent(resource: &str) -> serde_json::Value {
    serde_json::json!({
        "dataset": WAC_NQUADS,
        "session": { "agent": "https://alice.ex/card#me" },
        "resource": resource,
        "mode": "read",
        "view": "wac",
        "trust": {
            "nowUnixSecs": 1_720_000_000_i64,
            "rules": [],
            "credentials": []
        }
    })
}

/// A body with a `"trust"` block where `agentIri` is not a valid IRI.
fn decide_body_trust_invalid_agent_iri(resource: &str) -> serde_json::Value {
    serde_json::json!({
        "dataset": WAC_NQUADS,
        "session": { "agent": "https://alice.ex/card#me" },
        "resource": resource,
        "mode": "read",
        "view": "wac",
        "trust": {
            "agentIri": "not a valid IRI (has spaces)",
            "nowUnixSecs": 1_720_000_000_i64,
            "rules": [],
            "credentials": []
        }
    })
}

/// A body with a well-formed trust block but EMPTY credentials — no facts can be admitted, so the
/// WAC decision falls through on its own (anonymous/no-trust session => deny). Tests that the
/// endpoint does NOT open on an empty admit.
fn decide_body_trust_empty_credentials(resource: &str) -> serde_json::Value {
    serde_json::json!({
        "dataset": WAC_NQUADS,
        "session": {},
        "resource": resource,
        "mode": "read",
        "view": "wac",
        "trust": {
            "agentIri": "https://alice.ex/card#me",
            "nowUnixSecs": 1_720_000_000_i64,
            "rules": [],
            "certifications": [],
            "credentials": []
        }
    })
}

/// A body without any `"trust"` block — falls through to the unchanged solid-authz path.
fn decide_body_no_trust(agent: Option<&str>, resource: &str) -> serde_json::Value {
    let mut session = serde_json::Map::new();
    if let Some(a) = agent {
        session.insert("agent".into(), a.into());
    }
    serde_json::json!({
        "dataset": WAC_NQUADS,
        "session": session,
        "resource": resource,
        "mode": "read",
        "view": "wac",
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// A `"trust"` block that is not a JSON object => 403 DENY, `trustDenied: true`.
/// FAIL-CLOSED: a malformed trust block never admits.
#[tokio::test]
async fn trust_block_not_object_is_403() {
    let base = spawn_trust_on().await;
    let resp = client()
        .post(format!("{}/authz/decide", base))
        .json(&decide_body_trust_not_object("https://pod.ex/notes/n1"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "malformed trust block => 403");
    let body: serde_json::Value = resp.json().await.unwrap();
    // [SONNET-4.6] POSITIONAL format args (CodeQL guard).
    assert_eq!(
        body.get("allow").and_then(|v| v.as_bool()),
        Some(false),
        "trust block not object => allow:false"
    );
    assert_eq!(
        body.get("trustDenied").and_then(|v| v.as_bool()),
        Some(true),
        "trustDenied must be true on a trust-path deny"
    );
}

/// A `"trust"` block missing `agentIri` => 403 DENY, `trustDenied: true`.
#[tokio::test]
async fn trust_block_missing_agent_iri_is_403() {
    let base = spawn_trust_on().await;
    let resp = client()
        .post(format!("{}/authz/decide", base))
        .json(&decide_body_trust_missing_agent("https://pod.ex/notes/n1"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "missing agentIri => 403");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body.get("trustDenied").and_then(|v| v.as_bool()),
        Some(true),
    );
}

/// A `"trust"` block with an invalid `agentIri` (not a valid IRI) => 403 DENY.
#[tokio::test]
async fn trust_block_invalid_agent_iri_is_403() {
    let base = spawn_trust_on().await;
    let resp = client()
        .post(format!("{}/authz/decide", base))
        .json(&decide_body_trust_invalid_agent_iri("https://pod.ex/notes/n1"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "invalid agentIri => 403");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body.get("trustDenied").and_then(|v| v.as_bool()),
        Some(true),
    );
}

/// A trust rule with an invalid `issuerKeyHex` => 403 DENY.
/// The invalid key fails the public-key parse, which is fail-closed.
#[tokio::test]
async fn trust_rule_invalid_key_hex_is_403() {
    let base = spawn_trust_on().await;
    let resp = client()
        .post(format!("{}/authz/decide", base))
        .json(&decide_body_with_invalid_trust("https://pod.ex/notes/n1"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "invalid issuerKeyHex => 403");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body.get("trustDenied").and_then(|v| v.as_bool()),
        Some(true),
    );
}

/// DOUBLE-OPT-IN: with `solid_authz_trust == false` the `"trust"` block is IGNORED and the
/// endpoint falls through to the unchanged `solid-authz` path. A request with a (malformed)
/// `"trust"` block is treated as if no trust block was present — the WAC decision runs normally.
/// Alice has Read on the root, so the session-with-agent request => 200 allow.
///
/// This directly tests the double-opt-in contract: the trust extension is only active when BOTH
/// (a) the feature is compiled AND (b) the config flag is set.
#[tokio::test]
async fn trust_flag_off_ignores_trust_block_falls_through_to_wac() {
    let base = spawn_trust_flag_off().await;
    // Even with a malformed trust block, the flag is OFF so it is ignored entirely.
    let resp = client()
        .post(format!("{}/authz/decide", base))
        .json(&decide_body_trust_not_object("https://pod.ex/notes/n1"))
        .send()
        .await
        .unwrap();
    // Flag OFF => trust block ignored => normal WAC decision => alice (session.agent) has Read => 200.
    assert_eq!(resp.status(), 200, "flag off => trust block ignored => WAC falls through");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body.get("allow").and_then(|v| v.as_bool()),
        Some(true),
        "WAC allows alice on Read"
    );
    // No trustDenied field on the normal WAC path.
    assert!(
        body.get("trustDenied").is_none(),
        "no trustDenied on the WAC fallthrough path"
    );
}

/// FAIL-CLOSED: an empty `credentials` array => no facts admitted => the WAC decision runs
/// on the ORIGINAL dataset (no trust-injected facts) with an anonymous session => 403 deny.
/// Tests that the trust path NEVER opens an admission when no credentials are presented.
#[tokio::test]
async fn trust_empty_credentials_falls_through_to_wac_deny() {
    let base = spawn_trust_on().await;
    let resp = client()
        .post(format!("{}/authz/decide", base))
        .json(&decide_body_trust_empty_credentials("https://pod.ex/notes/n1"))
        .send()
        .await
        .unwrap();
    // [SONNET-4.6] POSITIONAL format args (CodeQL guard).
    // Session has no agent (anonymous) + no admitted facts => WAC deny => 403.
    assert_eq!(resp.status(), 403, "empty credentials + anon session => 403 WAC deny");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body.get("allow").and_then(|v| v.as_bool()),
        Some(false),
        "no credentials admitted => allow:false"
    );
    // The response must NOT carry trustDenied: the trust block was valid (parsed OK), so this
    // is a normal WAC deny, not a trust-parse failure.
    assert!(
        body.get("trustDenied").is_none(),
        "empty-credentials WAC deny is not a trust-parse error"
    );
}

/// Without a `"trust"` block, the endpoint behaves byte-identically to the `solid-authz` path
/// even when `solid_authz_trust == true` (double-opt-in: no trust block => fall through).
/// An authenticated session gets `200 allow`.
#[tokio::test]
async fn no_trust_block_falls_through_to_wac_allow() {
    let base = spawn_trust_on().await;
    let resp = client()
        .post(format!("{}/authz/decide", base))
        .json(&decide_body_no_trust(
            Some("https://alice.ex/card#me"),
            "https://pod.ex/notes/n1",
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "no trust block => WAC fallthrough => alice allow");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body.get("allow").and_then(|v| v.as_bool()), Some(true));
    assert!(body.get("trustDenied").is_none());
}

/// Without a `"trust"` block, an anonymous session still gets a 403 deny (the WAC path is
/// unchanged — no trust block can widen the WAC decision). Tests that the feature being compiled
/// does NOT change any behaviour for requests without a trust block.
#[tokio::test]
async fn no_trust_block_anon_session_is_wac_deny() {
    let base = spawn_trust_on().await;
    let resp = client()
        .post(format!("{}/authz/decide", base))
        .json(&decide_body_no_trust(None, "https://pod.ex/notes/n1"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "no trust block + anon session => WAC deny 403");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body.get("allow").and_then(|v| v.as_bool()), Some(false));
    assert!(body.get("trustDenied").is_none());
}

/// A trust rule where `"source"` is not a valid IRI => 403 DENY. Fail-closed on IRI validation.
#[tokio::test]
async fn trust_rule_invalid_source_iri_is_403() {
    let base = spawn_trust_on().await;
    let body = serde_json::json!({
        "dataset": WAC_NQUADS,
        "session": { "agent": "https://alice.ex/card#me" },
        "resource": "https://pod.ex/notes/n1",
        "mode": "read",
        "view": "wac",
        "trust": {
            "agentIri": "https://alice.ex/card#me",
            "nowUnixSecs": 1_720_000_000_i64,
            "rules": [
                {
                    "source": "not a valid IRI",
                    "issuerKeyHex": "aabbcc",
                    "scopeIri": "https://pod.ex/",
                    "freshWithinSecs": 86400_i64,
                    "shapePredicateIri": "https://schema.org/age"
                }
            ],
            "certifications": [],
            "credentials": []
        }
    });
    let resp = client()
        .post(format!("{}/authz/decide", base))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "invalid source IRI => 403");
    let resp_body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        resp_body.get("trustDenied").and_then(|v| v.as_bool()),
        Some(true),
    );
}

/// A certification with an unsupported `"scopeKind"` (not `"anyService"`) => 403 DENY.
/// The fail-closed rule: only explicitly modelled scope kinds are accepted.
#[tokio::test]
async fn trust_certification_unsupported_scope_kind_is_403() {
    let base = spawn_trust_on().await;
    // A plausible-looking hex public key (all zeros — invalid as a curve point, which is
    // caught later, but the scope-kind gate fires first so we never reach key parse).
    let zero_key = "00".repeat(33);
    let body = serde_json::json!({
        "dataset": WAC_NQUADS,
        "session": { "agent": "https://alice.ex/card#me" },
        "resource": "https://pod.ex/notes/n1",
        "mode": "read",
        "view": "wac",
        "trust": {
            "agentIri": "https://alice.ex/card#me",
            "nowUnixSecs": 1_720_000_000_i64,
            "rules": [],
            "certifications": [
                {
                    "certifierIri": "https://gov.example/framework",
                    "certifierKeyHex": zero_key,
                    "certifiedIssuerIri": "https://issuer.example/dvs",
                    "certifiedKeyHex": zero_key,
                    "validFromUnixSecs": 0_i64,
                    "validUntilUnixSecs": 9_999_999_999_i64,
                    "signatureHex": "aabbcc",
                    "scopeKind": "unknownScopeKind"
                }
            ],
            "credentials": []
        }
    });
    let resp = client()
        .post(format!("{}/authz/decide", base))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "unknown scopeKind => 403");
    let resp_body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        resp_body.get("trustDenied").and_then(|v| v.as_bool()),
        Some(true),
    );
}

/// A credential with invalid N-Quads text => 403 DENY. Fail-closed on credential parse.
#[tokio::test]
async fn trust_credential_invalid_nquads_is_403() {
    let base = spawn_trust_on().await;
    let body = serde_json::json!({
        "dataset": WAC_NQUADS,
        "session": { "agent": "https://alice.ex/card#me" },
        "resource": "https://pod.ex/notes/n1",
        "mode": "read",
        "view": "wac",
        "trust": {
            "agentIri": "https://alice.ex/card#me",
            "nowUnixSecs": 1_720_000_000_i64,
            "rules": [],
            "certifications": [],
            "credentials": [
                {
                    "graphNquads": "THIS IS NOT VALID N-QUADS !!!",
                    "issuerSignatureHex": "aabbcc",
                    "saltHex": "00".repeat(32),
                    "issuedAtUnixSecs": 1_720_000_000_i64,
                    "revoked": false
                }
            ]
        }
    });
    let resp = client()
        .post(format!("{}/authz/decide", base))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "invalid credential N-Quads => 403");
    let resp_body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        resp_body.get("trustDenied").and_then(|v| v.as_bool()),
        Some(true),
    );
}
