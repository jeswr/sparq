// AUTHORED-BY Claude Fable 5
//! PoP Tier-2 (DPoP-SK) end-to-end tests through the assembled router — establishment,
//! per-request attestation, and the spec's negative cases, all fail-closed.
//!
//! Drives the REAL stack: `solid-oidc-verifier` (static-JWKS + in-memory-replay doubles) → the
//! auth middleware's DPoP-SK dispatch → the LDP handlers over the in-memory store with an
//! enforced WAC seed — the same harness shape as `ldp_http.rs`, plus `AuthContext::with_dpop_sk`.
//!
//! Spec negative-case coverage (the task brief's REQUIRED set):
//! - replayed counters (inside the window + below it)          → `replayed_nonce_*`
//! - HMAC over mutated components (method/target/token/sig)    → `mutated_components_are_denied`
//! - wrong exporter label                                       → `wrong_exporter_label_fails_confirm_and_attestation`
//! - session-key reuse across connections (tls-exporter pin)    → `tls_exporter_session_is_connection_pinned`
//! - downgrade: stripped negotiation still demands DPoP          → `stripping_the_attestation_leaves_a_polless_request_401`,
//!   flavour substitution refused                                → `establishment_refuses_unavailable_or_unknown_flavours`,
//!   flag-off ignores attestations entirely                      → `flag_off_build_still_demands_dpop`,
//!   dual-mechanism exclusivity                                  → `invalid_attestation_wins_over_a_valid_dpop_proof`,
//!   alg mismatch                                                → `alg_mismatch_is_denied`
//! - Appendix-A byte-for-byte reproduction lives in the unit suites (`pop::sk::derive` /
//!   `pop::sk::sig` / `pop::sk::verify` — published vectors asserted against the real code paths).

mod common;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine as _;
use common::{jwks_provider, mint_access_token, mint_dpop_proof, KeyKit, BASE_URL};
use solid_oidc_verifier::config::VerifierConfig;
use solid_oidc_verifier::replay::InMemoryReplayStore;
use solid_oidc_verifier::verifier::Verifier;
use sparq_lws_core::app::{build_router, AppState};
use sparq_lws_core::auth::AuthContext;
use sparq_lws_core::ldp::handler::LdpState;
use sparq_lws_core::pop::sk::derive::{
    confirm_value, derive_session_key, hmac_sign, Ekm, SessionKey,
};
use sparq_lws_core::pop::sk::{ConnSk, SkConfig, SkState, SESSION_ENDPOINT_PATH};
use sparq_lws_core::store::{CompositeStore, InMemoryBlobStore, InMemorySparqClient};
use std::sync::Arc;
use tower::ServiceExt;

const TURTLE: &str =
    "<https://pod.example/alice/data#me> <http://xmlns.com/foaf/0.1/name> \"Alice\" .";

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Seed a ROOT `.acl` granting `owner_webid` full control on the root + descendants (mirrors the
/// `ldp_http.rs` seed so WAC is enforced, not bypassed, under the attested requests).
async fn seed_root_owner_acl(
    store: &CompositeStore<InMemorySparqClient, InMemoryBlobStore>,
    owner_webid: &str,
) {
    use sparq_lws_core::store::Store;
    let root = format!("{BASE_URL}/");
    let acl_body = format!(
        r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
<#owner> a acl:Authorization;
         acl:agent <{owner_webid}>;
         acl:accessTo <{root}>;
         acl:default <{root}>;
         acl:mode acl:Read, acl:Write, acl:Control."#
    );
    store
        .write(
            &format!("{root}.acl"),
            axum::body::Bytes::from(acl_body),
            "text/turtle",
        )
        .await
        .expect("seed root acl");
}

struct Harness {
    app: axum::Router,
    issuer_key: KeyKit,
    client_key: KeyKit,
}

impl Harness {
    /// Build the app; `sk` = the DPoP-SK state to enable (None ⇒ the flag-off build).
    async fn new(sk: Option<Arc<SkState>>) -> Self {
        let issuer_key = KeyKit::generate();
        let client_key = KeyKit::generate();
        let config = VerifierConfig::new(vec![common::ISSUER.to_string()], BASE_URL);
        let replay = InMemoryReplayStore::with_window(config.replay_ttl());
        let verifier = Verifier::new(config, jwks_provider(&issuer_key), replay).unwrap();
        let ctx = AuthContext::new(verifier, BASE_URL).with_dpop_sk(sk);
        let store = CompositeStore::new(InMemorySparqClient::new(), InMemoryBlobStore::new());
        seed_root_owner_acl(&store, common::WEBID).await;
        let ldp = LdpState::new(store, BASE_URL);
        let app = build_router(AppState::new(ctx, ldp));
        Self {
            app,
            issuer_key,
            client_key,
        }
    }

    async fn sk_enabled() -> Self {
        Self::new(Some(Arc::new(SkState::new(SkConfig::default())))).await
    }

    async fn sk_with_exporter() -> Self {
        Self::new(Some(Arc::new(SkState::new(SkConfig {
            exporter_available: true,
            ..SkConfig::default()
        }))))
        .await
    }

    /// A fresh DPoP-bound access token + a proof for `method path` (ordinary DPoP credentials).
    fn dpop_headers(&self, method: &str, path: &str) -> (String, String, String) {
        let access = mint_access_token(&self.issuer_key, &self.client_key.thumbprint);
        let htu = format!("{BASE_URL}{path}");
        let proof = mint_dpop_proof(&self.client_key, method, &htu, &access);
        (access.clone(), format!("DPoP {access}"), proof)
    }

    async fn send(&self, req: Request<Body>) -> axum::http::Response<Body> {
        self.app.clone().oneshot(req).await.unwrap()
    }

    /// POST /.pop/session with full DPoP + the given body; returns (status, parsed JSON, token).
    async fn establish(&self, body: &str) -> (StatusCode, serde_json::Value, String) {
        let (access, authz, proof) = self.dpop_headers("POST", SESSION_ENDPOINT_PATH);
        let req = Request::builder()
            .method("POST")
            .uri(SESSION_ENDPOINT_PATH)
            .header("authorization", &authz)
            .header("dpop", &proof)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let resp = self.send(req).await;
        let status = resp.status();
        let cache_control = resp
            .headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let bytes = to_bytes(resp.into_body(), 1 << 16).await.unwrap();
        let json: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        if status == StatusCode::CREATED {
            // Normative: the establishment response MUST carry Cache-Control: no-store.
            assert_eq!(cache_control.as_deref(), Some("no-store"));
        }
        (status, json, access)
    }

    /// Establish a `cb=none` session; returns (session_id, K, bound access token).
    async fn establish_none(&self) -> (String, SessionKey, String) {
        let (status, json, access) = self.establish(r#"{"cb":"none"}"#).await;
        assert_eq!(
            status,
            StatusCode::CREATED,
            "establishment must succeed: {json}"
        );
        assert_eq!(json["cb"], "none");
        assert_eq!(json["alg"], "hmac-sha256");
        let expires_in = json["expires_in"].as_i64().unwrap();
        assert!(
            expires_in > 0 && expires_in <= 300,
            "TTL bound: {expires_in}"
        );
        assert!(
            json.get("confirm").is_none(),
            "confirm is tls-exporter-only"
        );
        let key_bytes = URL_SAFE_NO_PAD
            .decode(json["key"].as_str().unwrap())
            .unwrap();
        let key = SessionKey::new(key_bytes.try_into().unwrap());
        (
            json["session_id"].as_str().unwrap().to_string(),
            key,
            access,
        )
    }
}

/// Build the attestation headers exactly as a conforming client would: the REQUIRED covered
/// triple, the received-serialization params line in the base, HMAC-SHA256 under K.
fn attestation(
    session_id: &str,
    key: &SessionKey,
    nonce: u64,
    created: i64,
    method: &str,
    path: &str,
    authorization: &str,
) -> (String, String) {
    let params = format!(
        "(\"@method\" \"@target-uri\" \"authorization\");created={created};keyid=\"{session_id}\";alg=\"hmac-sha256\";nonce=\"{nonce}\";tag=\"dpop-sk\""
    );
    let base = format!(
        "\"@method\": {method}\n\"@target-uri\": {BASE_URL}{path}\n\"authorization\": {authorization}\n\"@signature-params\": {params}"
    );
    let tag = hmac_sign(key, base.as_bytes());
    (
        format!("sig={params}"),
        format!("sig=:{}:", STANDARD.encode(tag)),
    )
}

/// An attested request (NO DPoP proof header — the attestation replaces it).
fn attested_request(
    method: &str,
    path: &str,
    access_token: &str,
    session_id: &str,
    key: &SessionKey,
    nonce: u64,
) -> Request<Body> {
    let authz = format!("DPoP {access_token}");
    let (sig_input, sig) = attestation(session_id, key, nonce, now(), method, path, &authz);
    Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", authz)
        .header("signature-input", sig_input)
        .header("signature", sig)
        .body(Body::empty())
        .unwrap()
}

fn assert_dpop_challenge(resp: &axum::http::Response<Body>) {
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let challenge = resp
        .headers()
        .get("www-authenticate")
        .and_then(|v| v.to_str().ok())
        .expect("401 must carry the standard challenge");
    assert!(
        challenge.contains("DPoP"),
        "the generic DPoP challenge, not an SK-specific error: {challenge}"
    );
}

// =================================================================================================
// Establishment + the happy path
// =================================================================================================

#[tokio::test]
async fn establishes_and_serves_attested_requests_without_a_dpop_proof() {
    let h = Harness::sk_enabled().await;
    // Seed a resource with an ordinary DPoP PUT.
    let (_, authz, proof) = h.dpop_headers("PUT", "/alice/data");
    let put = Request::builder()
        .method("PUT")
        .uri("/alice/data")
        .header("authorization", authz)
        .header("dpop", proof)
        .header("content-type", "text/turtle")
        .body(Body::from(TURTLE))
        .unwrap();
    assert_eq!(h.send(put).await.status(), StatusCode::CREATED);

    // One DPoP proof at establishment…
    let (session_id, key, access) = h.establish_none().await;
    // …then N attested requests: one HMAC each, no proof header, WAC still enforced.
    for nonce in 1..=3u64 {
        let resp = h
            .send(attested_request(
                "GET",
                "/alice/data",
                &access,
                &session_id,
                &key,
                nonce,
            ))
            .await;
        assert_eq!(resp.status(), StatusCode::OK, "attested GET #{nonce}");
    }
    // Out-of-order completion within the window (HTTP/2 reality): 10 then 5.
    let resp = h
        .send(attested_request(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &key,
            10,
        ))
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = h
        .send(attested_request(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &key,
            5,
        ))
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn metadata_document_advertises_the_profile() {
    let h = Harness::sk_enabled().await;
    let resp = h
        .send(
            Request::builder()
                .uri("/.well-known/oauth-protected-resource")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "application/json"
    );
    let bytes = to_bytes(resp.into_body(), 1 << 16).await.unwrap();
    let doc: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(doc["resource"], BASE_URL);
    assert_eq!(doc["dpop_bound_access_tokens_required"], true);
    let ps = &doc["pop_session"];
    assert_eq!(ps["endpoint"], format!("{BASE_URL}{SESSION_ENDPOINT_PATH}"));
    assert_eq!(ps["algs"][0], "hmac-sha256");
    // No in-process TLS in this harness ⇒ tls-exporter MUST NOT be advertised.
    assert_eq!(ps["channel_bindings"], serde_json::json!(["none"]));
    assert_eq!(ps["profile"], "https://w3id.org/jeswr/dpop-sk/v1");
}

// =================================================================================================
// Anti-replay negatives (spec: replayed sequence numbers, inside + outside the window)
// =================================================================================================

#[tokio::test]
async fn replayed_nonce_inside_the_window_is_denied() {
    let h = Harness::sk_enabled().await;
    let (session_id, key, access) = h.establish_none().await;
    let ok = h
        .send(attested_request(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &key,
            1,
        ))
        .await;
    // (resource absent ⇒ 404, but AUTH passed; that's fine for the replay assertion below)
    assert_ne!(ok.status(), StatusCode::UNAUTHORIZED);
    // The EXACT same attested request again (same nonce): denied with the generic challenge.
    let replay = h
        .send(attested_request(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &key,
            1,
        ))
        .await;
    assert_dpop_challenge(&replay);
}

#[tokio::test]
async fn nonce_below_the_window_is_denied_even_if_never_used() {
    let h = Harness::sk_enabled().await;
    let (session_id, key, access) = h.establish_none().await;
    // Advance the high-water mark far past the window width (1024).
    let resp = h
        .send(attested_request(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &key,
            5000,
        ))
        .await;
    assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
    // 5000 − 1024 = 3976 ⇒ counter 100 is below the window: denied although never used.
    let resp = h
        .send(attested_request(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &key,
            100,
        ))
        .await;
    assert_dpop_challenge(&resp);
    // The oldest in-window value is still fine (3977 > 5000 − 1024).
    let resp = h
        .send(attested_request(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &key,
            3977,
        ))
        .await;
    assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
}

// =================================================================================================
// HMAC-over-mutated-components negatives
// =================================================================================================

#[tokio::test]
async fn mutated_components_are_denied() {
    let h = Harness::sk_enabled().await;
    // Seed a resource so a would-be-successful mutation is observable.
    let (_, authz, proof) = h.dpop_headers("PUT", "/alice/data");
    let put = Request::builder()
        .method("PUT")
        .uri("/alice/data")
        .header("authorization", authz)
        .header("dpop", proof)
        .header("content-type", "text/turtle")
        .body(Body::from(TURTLE))
        .unwrap();
    assert_eq!(h.send(put).await.status(), StatusCode::CREATED);

    let (session_id, key, access) = h.establish_none().await;
    let authz = format!("DPoP {access}");

    // (a) Signed for GET /alice/data, sent to a DIFFERENT TARGET: the server recomputes
    // @target-uri from the actual request ⇒ HMAC mismatch ⇒ 401.
    let (sig_input, sig) = attestation(&session_id, &key, 1, now(), "GET", "/alice/data", &authz);
    let transplanted = Request::builder()
        .method("GET")
        .uri("/alice/other")
        .header("authorization", &authz)
        .header("signature-input", &sig_input)
        .header("signature", &sig)
        .body(Body::empty())
        .unwrap();
    assert_dpop_challenge(&h.send(transplanted).await);

    // (b) Signed for GET, sent as DELETE (method mutation): 401 AND the resource survives.
    let (sig_input, sig) = attestation(&session_id, &key, 2, now(), "GET", "/alice/data", &authz);
    let method_mutated = Request::builder()
        .method("DELETE")
        .uri("/alice/data")
        .header("authorization", &authz)
        .header("signature-input", &sig_input)
        .header("signature", &sig)
        .body(Body::empty())
        .unwrap();
    assert_dpop_challenge(&h.send(method_mutated).await);

    // (c) Tampered signature bytes: 401.
    let (sig_input, _) = attestation(&session_id, &key, 3, now(), "GET", "/alice/data", &authz);
    let mut wrong_tag = [0u8; 32];
    wrong_tag[0] = 0xff;
    let tampered = Request::builder()
        .method("GET")
        .uri("/alice/data")
        .header("authorization", &authz)
        .header("signature-input", sig_input)
        .header("signature", format!("sig=:{}:", STANDARD.encode(wrong_tag)))
        .body(Body::empty())
        .unwrap();
    assert_dpop_challenge(&h.send(tampered).await);

    // (d) A DIFFERENT TOKEN presented under a valid-for-another-token attestation: the token
    // binding (session token-hash) denies before/independently of the HMAC.
    let other_token = mint_access_token(&h.issuer_key, &h.client_key.thumbprint);
    let resp = h
        .send(attested_request(
            "GET",
            "/alice/data",
            &other_token,
            &session_id,
            &key,
            4,
        ))
        .await;
    assert_dpop_challenge(&resp);

    // None of the failures consumed counter 5's slot or broke the session: a genuine attested
    // request still succeeds (forged traffic cannot burn state — verify-then-mark).
    let resp = h
        .send(attested_request(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &key,
            5,
        ))
        .await;
    assert_eq!(resp.status(), StatusCode::OK);
    // And the DELETE in (b) did NOT happen.
    let resp = h
        .send(attested_request(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &key,
            6,
        ))
        .await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "resource must have survived the mutated DELETE"
    );
}

// =================================================================================================
// Downgrade / no-bearer-fallback negatives
// =================================================================================================

#[tokio::test]
async fn stripping_the_attestation_leaves_a_popless_request_401() {
    let h = Harness::sk_enabled().await;
    let (session_id, key, access) = h.establish_none().await;
    let _ = (session_id, key);
    // An attacker strips Signature/Signature-Input: what remains is a DPoP-scheme token with NO
    // proof-of-possession at all ⇒ the baseline verifier demands DPoP ⇒ 401 challenge. Never
    // bearer acceptance.
    let stripped = Request::builder()
        .method("GET")
        .uri("/alice/data")
        .header("authorization", format!("DPoP {access}"))
        .body(Body::empty())
        .unwrap();
    assert_dpop_challenge(&h.send(stripped).await);
}

#[tokio::test]
async fn flag_off_build_still_demands_dpop() {
    // The tier disabled (no with_dpop_sk): a fully VALID attestation is meaningless — Signature
    // headers are ignored, the DPoP baseline gates the request, and the token is never bearer.
    let h = Harness::new(None).await;
    // Establish is impossible (no route ⇒ the LDP wildcard handles /.pop/session), so craft an
    // attestation under an arbitrary key: the server must 401 on the missing DPoP proof.
    let key = SessionKey::new([9u8; 32]);
    let access = mint_access_token(&h.issuer_key, &h.client_key.thumbprint);
    let resp = h
        .send(attested_request(
            "GET",
            "/alice/data",
            &access,
            "some-session",
            &key,
            1,
        ))
        .await;
    assert_dpop_challenge(&resp);
    // And the metadata route is NOT mounted on a flag-off build (byte-identical surface):
    // the LDP wildcard answers for that path instead (404 for a missing resource, not a doc).
    let resp = h
        .send(
            Request::builder()
                .uri("/.well-known/oauth-protected-resource")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_ne!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn invalid_attestation_wins_over_a_valid_dpop_proof() {
    // Dual-mechanism exclusivity (downgrade rule 3): a request carrying a dpop-sk-tagged
    // signature is processed under the profile EXCLUSIVELY — an INVALID attestation is 401 even
    // though a fully valid DPoP proof rides on the same request.
    let h = Harness::sk_enabled().await;
    let (session_id, _key, access) = h.establish_none().await;
    let wrong_key = SessionKey::new([1u8; 32]);
    let authz = format!("DPoP {access}");
    let (sig_input, sig) = attestation(
        &session_id,
        &wrong_key,
        1,
        now(),
        "GET",
        "/alice/data",
        &authz,
    );
    let proof = mint_dpop_proof(
        &h.client_key,
        "GET",
        &format!("{BASE_URL}/alice/data"),
        &access,
    );
    let req = Request::builder()
        .method("GET")
        .uri("/alice/data")
        .header("authorization", authz)
        .header("dpop", proof) // valid proof — MUST NOT rescue the bad attestation
        .header("signature-input", sig_input)
        .header("signature", sig)
        .body(Body::empty())
        .unwrap();
    assert_dpop_challenge(&h.send(req).await);
}

#[tokio::test]
async fn alg_mismatch_is_denied() {
    let h = Harness::sk_enabled().await;
    let (session_id, key, access) = h.establish_none().await;
    let authz = format!("DPoP {access}");
    // Hand-craft an attestation declaring a different alg; the HMAC is even computed correctly
    // over that base — the alg policy alone must reject it (RFC 9421 §7.3.4 mixup guidance).
    let created = now();
    let params = format!(
        "(\"@method\" \"@target-uri\" \"authorization\");created={created};keyid=\"{session_id}\";alg=\"ed25519\";nonce=\"1\";tag=\"dpop-sk\""
    );
    let base = format!(
        "\"@method\": GET\n\"@target-uri\": {BASE_URL}/alice/data\n\"authorization\": {authz}\n\"@signature-params\": {params}"
    );
    let tag = hmac_sign(&key, base.as_bytes());
    let req = Request::builder()
        .method("GET")
        .uri("/alice/data")
        .header("authorization", authz)
        .header("signature-input", format!("sig={params}"))
        .header("signature", format!("sig=:{}:", STANDARD.encode(tag)))
        .body(Body::empty())
        .unwrap();
    assert_dpop_challenge(&h.send(req).await);
}

// =================================================================================================
// Establishment negatives
// =================================================================================================

#[tokio::test]
async fn establishment_requires_full_dpop() {
    let h = Harness::sk_enabled().await;
    // Anonymous.
    let req = Request::builder()
        .method("POST")
        .uri(SESSION_ENDPOINT_PATH)
        .header("content-type", "application/json")
        .body(Body::from(r#"{"cb":"none"}"#))
        .unwrap();
    assert_dpop_challenge(&h.send(req).await);
    // DPoP-scheme token WITHOUT a proof.
    let access = mint_access_token(&h.issuer_key, &h.client_key.thumbprint);
    let req = Request::builder()
        .method("POST")
        .uri(SESSION_ENDPOINT_PATH)
        .header("authorization", format!("DPoP {access}"))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"cb":"none"}"#))
        .unwrap();
    assert_dpop_challenge(&h.send(req).await);
}

#[tokio::test]
async fn establishment_refuses_unavailable_or_unknown_flavours() {
    let h = Harness::sk_enabled().await; // exporter NOT available in this harness
                                         // cb=tls-exporter cannot be satisfied ⇒ 400, and NO cb=none session is substituted.
    let (status, json, _) = h.establish(r#"{"cb":"tls-exporter"}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        json.get("session_id").is_none(),
        "no substituted session: {json}"
    );
    // Unknown flavour ⇒ 400. Missing cb ⇒ 400. Malformed JSON ⇒ 400.
    for body in [r#"{"cb":"noise"}"#, r#"{}"#, "{"] {
        let (status, _, _) = h.establish(body).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "body {body:?}");
    }
}

#[tokio::test]
async fn attested_establishment_is_refused() {
    // A session must not mint a session: POST /.pop/session authenticated by an ATTESTATION
    // (rather than a fresh DPoP proof) is refused — re-establishment costs one real proof.
    let h = Harness::sk_enabled().await;
    let (session_id, key, access) = h.establish_none().await;
    let authz = format!("DPoP {access}");
    let (sig_input, sig) = attestation(
        &session_id,
        &key,
        1,
        now(),
        "POST",
        SESSION_ENDPOINT_PATH,
        &authz,
    );
    let req = Request::builder()
        .method("POST")
        .uri(SESSION_ENDPOINT_PATH)
        .header("authorization", authz)
        .header("signature-input", sig_input)
        .header("signature", sig)
        .header("content-type", "application/json")
        .body(Body::from(r#"{"cb":"none"}"#))
        .unwrap();
    assert_dpop_challenge(&h.send(req).await);
}

// =================================================================================================
// Termination
// =================================================================================================

#[tokio::test]
async fn attested_delete_terminates_the_session() {
    let h = Harness::sk_enabled().await;
    let (session_id, key, access) = h.establish_none().await;
    // DELETE attested UNDER THE SESSION ITSELF (keyid names it).
    let req = attested_request(
        "DELETE",
        SESSION_ENDPOINT_PATH,
        &access,
        &session_id,
        &key,
        1,
    );
    let resp = h.send(req).await;
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    // The session is dead: a subsequent attested request is denied.
    let resp = h
        .send(attested_request(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &key,
            2,
        ))
        .await;
    assert_dpop_challenge(&resp);
}

#[tokio::test]
async fn unattested_delete_cannot_terminate() {
    // A plain-DPoP DELETE names no session (no keyid) ⇒ 401; the session survives.
    let h = Harness::sk_enabled().await;
    let (session_id, key, access) = h.establish_none().await;
    let (_, authz, proof) = h.dpop_headers("DELETE", SESSION_ENDPOINT_PATH);
    let req = Request::builder()
        .method("DELETE")
        .uri(SESSION_ENDPOINT_PATH)
        .header("authorization", authz)
        .header("dpop", proof)
        .body(Body::empty())
        .unwrap();
    assert_dpop_challenge(&h.send(req).await);
    // Session still live.
    let resp = h
        .send(attested_request(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &key,
            1,
        ))
        .await;
    assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
}

// =================================================================================================
// cb=tls-exporter (via the injected per-connection context — the acceptor seam)
// =================================================================================================

/// An establishment request carrying an injected `ConnSk` (simulating the TLS acceptor).
async fn establish_exporter(h: &Harness, conn: ConnSk) -> (StatusCode, serde_json::Value, String) {
    let (access, authz, proof) = h.dpop_headers("POST", SESSION_ENDPOINT_PATH);
    let mut req = Request::builder()
        .method("POST")
        .uri(SESSION_ENDPOINT_PATH)
        .header("authorization", &authz)
        .header("dpop", &proof)
        .header("content-type", "application/json")
        .body(Body::from(r#"{"cb":"tls-exporter"}"#))
        .unwrap();
    req.extensions_mut().insert(conn);
    let resp = h.send(req).await;
    let status = resp.status();
    let bytes = to_bytes(resp.into_body(), 1 << 16).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json, access)
}

/// An attested request carrying an injected `ConnSk` (conn_id only — no EKM off the
/// establishment path, mirroring the acceptor's behaviour).
fn attested_on_conn(
    method: &str,
    path: &str,
    access: &str,
    session_id: &str,
    key: &SessionKey,
    nonce: u64,
    conn_id: u64,
) -> Request<Body> {
    let mut req = attested_request(method, path, access, session_id, key, nonce);
    req.extensions_mut().insert(ConnSk::new(conn_id, None));
    req
}

#[tokio::test]
async fn tls_exporter_flavour_derives_and_confirms_the_same_key_both_sides() {
    let h = Harness::sk_with_exporter().await;
    let ekm_bytes = [0x5au8; 32];
    let (status, json, access) =
        establish_exporter(&h, ConnSk::new(7, Some(Ekm::new(ekm_bytes)))).await;
    assert_eq!(status, StatusCode::CREATED, "{json}");
    assert_eq!(json["cb"], "tls-exporter");
    assert!(
        json.get("key").is_none(),
        "no key bytes on the wire in this flavour"
    );
    let session_id = json["session_id"].as_str().unwrap().to_string();

    // CLIENT side: derive K from the same EKM + response session_id + its own ath/jkt…
    let ath = common::ath(&access);
    let jkt = &h.client_key.thumbprint;
    let k = derive_session_key(&Ekm::new(ekm_bytes), &session_id, &ath, jkt);
    // …and verify the key-confirmation BEFORE any attested request (spec A22).
    assert_eq!(
        json["confirm"].as_str().unwrap(),
        confirm_value(&k, &session_id),
        "both sides must have derived the same K"
    );

    // The derived key attests successfully on the pinned connection.
    let resp = h
        .send(attested_on_conn(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &k,
            1,
            7,
        ))
        .await;
    assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn tls_exporter_session_is_connection_pinned() {
    // Session-key reuse across connections is impossible: the session is pinned to its TLS
    // connection (spec verification step 3 / adversarial entry A4/A17).
    let h = Harness::sk_with_exporter().await;
    let ekm = [0x21u8; 32];
    let (status, json, access) = establish_exporter(&h, ConnSk::new(1, Some(Ekm::new(ekm)))).await;
    assert_eq!(status, StatusCode::CREATED);
    let session_id = json["session_id"].as_str().unwrap().to_string();
    let ath = common::ath(&access);
    let k = derive_session_key(&Ekm::new(ekm), &session_id, &ath, &h.client_key.thumbprint);

    // A DIFFERENT connection id: denied.
    let resp = h
        .send(attested_on_conn(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &k,
            1,
            2,
        ))
        .await;
    assert_dpop_challenge(&resp);
    // NO connection context at all (e.g. the plain listener): denied.
    let resp = h
        .send(attested_request(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &k,
            2,
        ))
        .await;
    assert_dpop_challenge(&resp);
    // The pinned connection still works (nothing was burned by the failures).
    let resp = h
        .send(attested_on_conn(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &k,
            3,
            1,
        ))
        .await;
    assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn wrong_exporter_label_fails_confirm_and_attestation() {
    // A client deriving from the WRONG exporter label (e.g. the RFC 9266 channel-binding value —
    // exactly the reuse the spec's A14 FINDING forbids) gets different EKM ⇒ different K: the
    // `confirm` member exposes the mismatch up front, and its attestations are denied.
    use sha2::{Digest as _, Sha256};
    let label_keyed_exporter = |label: &[u8]| -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(b"this-connection's-exporter-master-secret");
        hash.update(label);
        hash.finalize().into()
    };
    let h = Harness::sk_with_exporter().await;
    let server_ekm = label_keyed_exporter(b"EXPERIMENTAL-dpop-sk-v1");
    let (status, json, access) =
        establish_exporter(&h, ConnSk::new(3, Some(Ekm::new(server_ekm)))).await;
    assert_eq!(status, StatusCode::CREATED);
    let session_id = json["session_id"].as_str().unwrap().to_string();

    // Client mis-derives under the RFC 9266 label.
    let wrong_ekm = label_keyed_exporter(b"EXPORTER-Channel-Binding");
    let ath = common::ath(&access);
    let wrong_k = derive_session_key(
        &Ekm::new(wrong_ekm),
        &session_id,
        &ath,
        &h.client_key.thumbprint,
    );
    // The confirm member detects the mismatch BEFORE any attested request…
    assert_ne!(
        json["confirm"].as_str().unwrap(),
        confirm_value(&wrong_k, &session_id)
    );
    // …and a request attested under the wrong-label key is denied.
    let resp = h
        .send(attested_on_conn(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &wrong_k,
            1,
            3,
        ))
        .await;
    assert_dpop_challenge(&resp);
}

#[tokio::test]
async fn tls_exporter_without_ekm_is_refused_fail_closed() {
    // In-process TLS but no EKM for this connection (TLS 1.2 / export failure): establishment is
    // a 400 — the binding is never faked from weaker material, and never downgraded to cb=none.
    let h = Harness::sk_with_exporter().await;
    let (status, json, _) = establish_exporter(&h, ConnSk::new(9, None)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(json.get("session_id").is_none());
}

#[tokio::test]
async fn trailing_slash_base_url_still_verifies_attestations() {
    // Regression (roborev Medium on the Tier-2 commit): an operator-configured base_url WITH a
    // trailing slash must reconstruct the same @target-uri the client signed
    // (https://host/path, not https://host//path) — mirroring parse_target's htu normalization.
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let config = VerifierConfig::new(vec![common::ISSUER.to_string()], BASE_URL);
    let replay = InMemoryReplayStore::with_window(config.replay_ttl());
    let verifier = Verifier::new(config, jwks_provider(&issuer_key), replay).unwrap();
    // The trailing-slash form of the SAME origin.
    let ctx = AuthContext::new(verifier, format!("{BASE_URL}/"))
        .with_dpop_sk(Some(Arc::new(SkState::new(SkConfig::default()))));
    let store = CompositeStore::new(InMemorySparqClient::new(), InMemoryBlobStore::new());
    seed_root_owner_acl(&store, common::WEBID).await;
    let ldp = LdpState::new(store, BASE_URL);
    let h = Harness {
        app: build_router(AppState::new(ctx, ldp)),
        issuer_key,
        client_key,
    };
    let (session_id, key, access) = h.establish_none().await;
    // The client signs the slash-normalized target (what a conforming client sends).
    let resp = h
        .send(attested_request(
            "GET",
            "/alice/data",
            &access,
            &session_id,
            &key,
            1,
        ))
        .await;
    assert_ne!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "trailing-slash base_url must not break attestation verification"
    );
}
