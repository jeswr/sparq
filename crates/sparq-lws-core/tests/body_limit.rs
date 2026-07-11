// AUTHORED-BY Claude Opus 4.8
//! End-to-end HTTP tests for the explicit request-body size limit (bead 7hg) through the assembled
//! router. Proves the SECURITY-relevant property: an authenticated request whose body EXCEEDS the
//! configured limit is rejected with **413 Payload Too Large** — before the handler buffers it — while
//! a body AT/under the limit is accepted. The limit is an explicit, configurable memory-exhaustion
//! bound (`crate::body_limit`), not axum's invisible implicit default.
//!
//! The router is built via `build_router_with_overload` with a TIGHT `body_limit_bytes` so a small
//! over-limit body trips it deterministically. Requests are authenticated (a fresh DPoP-bound token +
//! per-request proof) because the body extractor runs INSIDE the auth-gated handler — the point of the
//! bound is to cap an AUTHENTICATED oversized upload (an anonymous one is 401'd before any body read).

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{jwks_provider, mint_access_token, mint_dpop_proof, KeyKit, BASE_URL};
use solid_oidc_verifier::config::VerifierConfig;
use solid_oidc_verifier::replay::InMemoryReplayStore;
use solid_oidc_verifier::verifier::Verifier;
use sparq_lws_core::app::{build_router_with_overload, AppState, OverloadConfig};
use sparq_lws_core::auth::AuthContext;
use sparq_lws_core::ldp::handler::LdpState;
use sparq_lws_core::overload::AdmissionControl;
use sparq_lws_core::store::{CompositeStore, InMemoryBlobStore, InMemorySparqClient, Store};
use tower::ServiceExt;

/// Seed a ROOT `.acl` granting the owner Read/Write/Control on everything, so an under-limit authed PUT
/// reaches the store and returns a clean 2xx (isolating the body-limit from a WAC 403). Test fixture.
async fn seed_root_owner_acl(
    store: &CompositeStore<InMemorySparqClient, InMemoryBlobStore>,
    owner_webid: &str,
) {
    let root = format!("{BASE_URL}/");
    let acl_iri = format!("{root}.acl");
    let acl_body = format!(
        r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
<#owner> a acl:Authorization;
         acl:agent <{owner_webid}>;
         acl:accessTo <{root}>;
         acl:default <{root}>;
         acl:mode acl:Read, acl:Write, acl:Control."#
    );
    store
        .write(&acl_iri, axum::body::Bytes::from(acl_body), "text/turtle")
        .await
        .expect("seed root acl");
}

/// Build the router with a tight `body_limit_bytes`, plus the auth keys to mint requests.
struct Harness {
    app: axum::Router,
    issuer_key: KeyKit,
    client_key: KeyKit,
}

impl Harness {
    async fn new(body_limit_bytes: usize) -> Self {
        let issuer_key = KeyKit::generate();
        let client_key = KeyKit::generate();
        let config = VerifierConfig::new(vec![common::ISSUER.to_string()], BASE_URL);
        let replay = InMemoryReplayStore::with_window(config.replay_ttl());
        let verifier = Verifier::new(config, jwks_provider(&issuer_key), replay).unwrap();
        let ctx = AuthContext::new(verifier, BASE_URL);
        let store = CompositeStore::new(InMemorySparqClient::new(), InMemoryBlobStore::new());
        seed_root_owner_acl(&store, common::WEBID).await;
        let ldp = LdpState::new(store, BASE_URL);
        let overload = OverloadConfig {
            admission: AdmissionControl::new(10_000),
            request_timeout: None,
            rate_limiter: None,
            body_limit_bytes,
        };
        let app = build_router_with_overload(AppState::new(ctx, ldp), overload);
        Self {
            app,
            issuer_key,
            client_key,
        }
    }

    async fn authed_put(&self, path: &str, body: &'static [u8]) -> StatusCode {
        let access = mint_access_token(&self.issuer_key, &self.client_key.thumbprint);
        let htu = format!("{BASE_URL}{path}");
        let proof = mint_dpop_proof(&self.client_key, "PUT", &htu, &access);
        let req = Request::builder()
            .method("PUT")
            .uri(path)
            .header("authorization", format!("DPoP {access}"))
            .header("dpop", proof)
            .header("content-type", "text/turtle")
            .body(Body::from(body))
            .unwrap();
        self.app.clone().oneshot(req).await.unwrap().status()
    }
}

#[tokio::test]
async fn body_over_the_limit_is_413_and_under_is_accepted() {
    // A tight 16-byte ceiling. A 13-byte body is under it ⇒ accepted (2xx, owner ACL seeded); a
    // larger body is over it ⇒ 413 BEFORE the handler buffers it.
    let h = Harness::new(16).await;

    let small: &[u8] = b"<a> <b> <c> ."; // 13 bytes, under 16
    assert!(small.len() <= 16);
    let ok = h.authed_put("/alice/small", small).await;
    assert!(
        ok.is_success(),
        "an under-limit authed PUT must be accepted (got {ok}), proving the limit blocks ONLY oversized bodies"
    );

    let big: &[u8] = b"<a> <b> <c> . <d> <e> <f> . <g> <h> <i> ."; // 41 bytes, over 16
    assert!(big.len() > 16);
    let too_large = h.authed_put("/alice/big", big).await;
    assert_eq!(
        too_large,
        StatusCode::PAYLOAD_TOO_LARGE,
        "a body over the configured limit must be 413 (explicit memory-exhaustion bound)"
    );
}

#[tokio::test]
async fn default_two_mib_limit_admits_a_normal_body() {
    // Under the DEFAULT (2 MiB) limit, a normal small body is never 413'd — the default is
    // behaviour-preserving (identical to axum's implicit default).
    let h = Harness::new(sparq_lws_core::body_limit::DEFAULT_MAX_BODY_BYTES).await;
    let status = h.authed_put("/alice/normal", b"<a> <b> <c> .").await;
    assert_ne!(
        status,
        StatusCode::PAYLOAD_TOO_LARGE,
        "a normal body under the 2 MiB default must not be 413"
    );
    assert!(status.is_success(), "and it is stored (got {status})");
}
