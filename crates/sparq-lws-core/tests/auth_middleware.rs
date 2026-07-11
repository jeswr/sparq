// AUTHORED-BY Claude Opus 4.8
//! Auth-middleware tests — valid / invalid / missing token, driven through the real
//! `solid-oidc-verifier` (via the static-JWKS + in-memory-replay test doubles).
//!
//! Covers both the `AuthContext::authenticate` seam directly and the full axum middleware via an
//! end-to-end `oneshot` request through the router.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{jwks_provider, mint_access_token, mint_dpop_proof, KeyKit, BASE_URL, WEBID};
use solid_oidc_verifier::config::VerifierConfig;
use solid_oidc_verifier::replay::InMemoryReplayStore;
use solid_oidc_verifier::verifier::Verifier;
use sparq_lws_core::app::{build_router, AppState};
use sparq_lws_core::auth::AuthContext;
use sparq_lws_core::ldp::handler::LdpState;
use sparq_lws_core::store::{CompositeStore, InMemoryBlobStore, InMemorySparqClient};
use tower::ServiceExt;

type TestVerifier = Verifier<solid_oidc_verifier::config::StaticJwksProvider, InMemoryReplayStore>;

/// Build an `AuthContext` whose verifier trusts `issuer_key`, with the base URL == audience.
fn auth_context(
    issuer_key: &KeyKit,
) -> AuthContext<solid_oidc_verifier::config::StaticJwksProvider, InMemoryReplayStore> {
    let config = VerifierConfig::new(vec![common::ISSUER.to_string()], BASE_URL);
    let replay = InMemoryReplayStore::with_window(config.replay_ttl());
    let verifier: TestVerifier =
        Verifier::new(config, jwks_provider(issuer_key), replay).expect("valid config");
    AuthContext::new(verifier, BASE_URL)
}

#[test]
fn missing_authorization_is_public_not_an_error() {
    let issuer_key = KeyKit::generate();
    let ctx = auth_context(&issuer_key);

    let token = ctx
        .authenticate(None, None, "GET", "/alice/data")
        .expect("no Authorization ⇒ public credentials, not an error");
    assert!(
        token.is_public(),
        "absent auth must yield public credentials"
    );
    assert!(token.web_id.is_none());
}

#[test]
fn valid_dpop_bound_token_authenticates() {
    // The client key is what the token is cnf-bound to; the issuer key signs the token.
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let ctx = auth_context(&issuer_key);

    let access = mint_access_token(&issuer_key, &client_key.thumbprint);
    let htu = format!("{BASE_URL}/alice/data");
    let proof = mint_dpop_proof(&client_key, "GET", &htu, &access);

    let token = ctx
        .authenticate(
            Some(format!("DPoP {access}")),
            Some(proof),
            "GET",
            "/alice/data",
        )
        .expect("a well-formed DPoP-bound token must authenticate");
    assert_eq!(token.web_id.as_deref(), Some(WEBID));
    assert_eq!(token.issuer.as_deref(), Some(common::ISSUER));
}

#[test]
fn token_from_an_untrusted_issuer_is_rejected() {
    // The verifier trusts `issuer_key`, but the token is signed by a DIFFERENT key.
    let trusted_key = KeyKit::generate();
    let rogue_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let ctx = auth_context(&trusted_key);

    let access = mint_access_token(&rogue_key, &client_key.thumbprint);
    let htu = format!("{BASE_URL}/alice/data");
    let proof = mint_dpop_proof(&client_key, "GET", &htu, &access);

    let err = ctx
        .authenticate(
            Some(format!("DPoP {access}")),
            Some(proof),
            "GET",
            "/alice/data",
        )
        .expect_err("a token whose signature does not verify against the JWKS must be rejected");
    assert_eq!(err.status().as_u16(), 401);
}

#[test]
fn dpop_proof_for_a_different_url_is_rejected() {
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let ctx = auth_context(&issuer_key);

    let access = mint_access_token(&issuer_key, &client_key.thumbprint);
    // The proof's htu binds to /other, but the request is for /alice/data — htu mismatch.
    let wrong_htu = format!("{BASE_URL}/other");
    let proof = mint_dpop_proof(&client_key, "GET", &wrong_htu, &access);

    let err = ctx
        .authenticate(
            Some(format!("DPoP {access}")),
            Some(proof),
            "GET",
            "/alice/data",
        )
        .expect_err("a DPoP proof bound to a different htu must be rejected");
    assert_eq!(err.status().as_u16(), 401);
}

#[test]
fn bearer_without_dpop_is_rejected() {
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let ctx = auth_context(&issuer_key);

    let access = mint_access_token(&issuer_key, &client_key.thumbprint);
    // A bare Bearer (no DPoP proof) — proof-of-possession is required by default.
    let err = ctx
        .authenticate(Some(format!("Bearer {access}")), None, "GET", "/alice/data")
        .expect_err("bare Bearer must be rejected when DPoP is required");
    assert_eq!(err.status().as_u16(), 401);
}

#[test]
fn replayed_dpop_proof_jti_is_rejected() {
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let ctx = auth_context(&issuer_key);

    let access = mint_access_token(&issuer_key, &client_key.thumbprint);
    let htu = format!("{BASE_URL}/alice/data");
    let proof = mint_dpop_proof(&client_key, "GET", &htu, &access);

    // First use succeeds.
    ctx.authenticate(
        Some(format!("DPoP {access}")),
        Some(proof.clone()),
        "GET",
        "/alice/data",
    )
    .expect("first use of a fresh proof authenticates");

    // Re-using the SAME proof (same jti) must fail — single-use replay protection.
    let err = ctx
        .authenticate(
            Some(format!("DPoP {access}")),
            Some(proof),
            "GET",
            "/alice/data",
        )
        .expect_err("a replayed DPoP jti must be rejected");
    assert_eq!(err.status().as_u16(), 401);
}

#[tokio::test]
async fn http_get_without_auth_injects_public_creds_and_the_ldp_layer_gates() {
    // End-to-end through the router: no Authorization ⇒ the middleware injects PUBLIC credentials and
    // the handler runs (it does NOT 400/short-circuit). The WAC read gate then decides per the
    // effective ACL:
    //   - the WebID profile card has a public-read ACL (seeded by `test_app`) ⇒ anonymous read is
    //     allowed and reaches the (empty) store → 404 (nothing stored) — proving public creds were
    //     injected and the read ran;
    //   - any OTHER resource has no ACL anywhere ⇒ anonymous read is denied 401 + `WWW-Authenticate`.
    let issuer_key = KeyKit::generate();
    let app = test_app(&issuer_key).await;

    // The public-read profile document: anonymous read is allowed, so it reaches the (empty) store → 404.
    let public = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/alice/profile/card")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(public.status(), StatusCode::NOT_FOUND);

    // A resource with no ACL anywhere: anonymous read ⇒ 401 + challenge (WAC fail-closed).
    let gated = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/alice/data")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(gated.status(), StatusCode::UNAUTHORIZED);
    assert!(gated
        .headers()
        .contains_key(axum::http::header::WWW_AUTHENTICATE));
}

#[tokio::test]
async fn http_get_with_a_bad_token_is_401_with_www_authenticate() {
    let issuer_key = KeyKit::generate();
    let app = test_app(&issuer_key).await;

    // A garbage token — the verifier rejects it, the middleware returns its 401 + challenge.
    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/alice/data")
                .header("authorization", "DPoP not-a-real-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    assert!(
        resp.headers()
            .contains_key(axum::http::header::WWW_AUTHENTICATE),
        "a 401 from the verifier must carry a WWW-Authenticate challenge"
    );
}

#[tokio::test]
async fn http_get_with_a_non_utf8_authorization_header_is_400_not_public() {
    // A present-but-unparseable Authorization header must NOT be silently downgraded to public
    // access (a fail-open). It is a 400, distinct from an absent header (which is public → 404 here).
    let issuer_key = KeyKit::generate();
    let app = test_app(&issuer_key).await;

    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/alice/data")
                // 0xFF is not valid UTF-8 → HeaderValue present but not str-convertible.
                .header(
                    axum::http::header::AUTHORIZATION,
                    axum::http::HeaderValue::from_bytes(b"DPoP \xFF").unwrap(),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

/// Assemble the full app over the in-memory store + a verifier trusting `issuer_key`.
async fn test_app(issuer_key: &KeyKit) -> axum::Router {
    use sparq_lws_core::store::Store;
    let ctx = auth_context(issuer_key);
    let store = CompositeStore::new(InMemorySparqClient::new(), InMemoryBlobStore::new());
    // WAC is enforced, so a public read needs a public-read ACL. Seed `/alice/profile/card.acl`
    // granting `foaf:Agent acl:Read` (the WebID-profile public-read class the real conformance seed
    // writes), so an anonymous GET of the card is ALLOWED and reaches the (empty) store → 404; every
    // other path has no ACL anywhere → anonymous read is denied with 401 (fail-closed).
    let card_acl = format!("{BASE_URL}/alice/profile/card.acl");
    let card_acl_body = format!(
        r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
@prefix foaf: <http://xmlns.com/foaf/0.1/>.
<#public> a acl:Authorization;
          acl:agentClass foaf:Agent;
          acl:accessTo <{BASE_URL}/alice/profile/card>;
          acl:mode acl:Read."#
    );
    store
        .write(
            &card_acl,
            axum::body::Bytes::from(card_acl_body),
            "text/turtle",
        )
        .await
        .expect("seed profile-card public-read acl");
    let ldp = LdpState::new(store, BASE_URL);
    // `AppState::new` wires the verifier-derived anonymous-401 challenge into the LDP layer.
    build_router(AppState::new(ctx, ldp))
}
