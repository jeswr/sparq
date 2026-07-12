// AUTHORED-BY GPT-5.6
//! Focused acceptance tests for provider WebID content negotiation.

mod common;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use common::{jwks_provider, KeyKit, BASE_URL};
use solid_oidc_verifier::config::VerifierConfig;
use solid_oidc_verifier::replay::InMemoryReplayStore;
use solid_oidc_verifier::verifier::Verifier;
use sparq_lws_core::app::{build_router, AppState};
use sparq_lws_core::auth::AuthContext;
use sparq_lws_core::identity::IdentityConfig;
use sparq_lws_core::ldp::handler::LdpState;
use sparq_lws_core::seed::seed_conformance_with_identity;
use sparq_lws_core::store::{CompositeStore, InMemoryBlobStore, InMemorySparqClient};
use tower::ServiceExt;

const ID_HOST: &str = "id.pod.example";

async fn app() -> axum::Router {
    let issuer_key = KeyKit::generate();
    let config = VerifierConfig::new(vec![common::ISSUER.to_string()], BASE_URL);
    let replay = InMemoryReplayStore::with_window(config.replay_ttl());
    let verifier = Verifier::new(config, jwks_provider(&issuer_key), replay).unwrap();
    let auth = AuthContext::new(verifier, BASE_URL);
    let store = CompositeStore::new(InMemorySparqClient::new(), InMemoryBlobStore::new());
    let identity = IdentityConfig::new(BASE_URL, None).unwrap();

    seed_conformance_with_identity(&store, BASE_URL, common::ISSUER, Some(&identity))
        .await
        .unwrap();

    let ldp = LdpState::new(store, BASE_URL);
    build_router(AppState::new(auth, ldp).with_identity(identity))
}

async fn request(
    method: &str,
    host: &str,
    path: &str,
    accept: Option<&str>,
) -> axum::http::Response<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("host", host);
    if let Some(value) = accept {
        builder = builder.header("accept", value);
    }
    app()
        .await
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn identity_document_negotiates_turtle_and_json_ld() {
    for (accept, content_type) in [
        ("text/turtle", "text/turtle"),
        ("application/ld+json", "application/ld+json"),
    ] {
        let response = request("GET", ID_HOST, "/alice", Some(accept)).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            content_type
        );
        assert_eq!(response.headers().get("vary").unwrap(), "accept");
        assert_eq!(
            response.headers().get("cache-control").unwrap(),
            "public, max-age=300"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("https://id.pod.example/alice#me"));
        assert!(body.contains("oidcIssuer"));
        assert!(body.contains(common::ISSUER));
    }
}

#[tokio::test]
async fn reserved_identity_namespace_still_refuses_writes() {
    for method in ["POST", "PUT"] {
        let response = request(method, "pod.example", "/.identity/alice", None).await;
        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "{method} must not reach the LDP write surface"
        );
    }
}
