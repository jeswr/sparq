// AUTHORED-BY Claude Fable 5
//! End-to-end HTTP tests for the opt-in demo playground seed (`SOLID_SERVER_SEED_DEMO` — the §3.2
//! public-demo posture of `research/lws-demo-architecture.md`, sq-5ougp).
//!
//! The seeded posture under test: a shared root-level `/playground/` container any AUTHENTICATED
//! agent can Read/Write/Append (via `acl:agentClass acl:AuthenticatedAgent` + `acl:default`
//! inheritance), publicly readable, with NO `acl:Control` granted to anyone; plus a public-read
//! `/README` Turtle document carrying the ephemeral-demo banner. Flag-off (unseeded) boots stay
//! byte-identical fail-closed: no `/playground/`, no ACLs, everything denied.
//!
//! Each request carries a fresh, well-formed DPoP-bound token + a per-request proof (a new jti) so
//! the verifier's single-use replay protection never rejects a follow-up request.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::{jwks_provider, mint_access_token, mint_dpop_proof, KeyKit, BASE_URL};
use solid_oidc_verifier::config::VerifierConfig;
use solid_oidc_verifier::replay::InMemoryReplayStore;
use solid_oidc_verifier::verifier::Verifier;
use sparq_lws_core::app::{build_router, AppState};
use sparq_lws_core::auth::AuthContext;
use sparq_lws_core::ldp::handler::LdpState;
use sparq_lws_core::store::{CompositeStore, InMemoryBlobStore};
use tower::ServiceExt;

const TURTLE: &str =
    "<https://pod.example/playground/note#it> <http://xmlns.com/foaf/0.1/name> \"Note\" .";

/// The same backend split as `ldp_http.rs`: the embedded in-process engine on the default build,
/// the in-memory double on `--no-default-features`.
#[cfg(feature = "embedded-sparq")]
type BackendSparqClient = sparq_lws_core::store::EmbeddedSparqClient;
#[cfg(not(feature = "embedded-sparq"))]
type BackendSparqClient = sparq_lws_core::store::InMemorySparqClient;

fn backend_sparq_client() -> BackendSparqClient {
    #[cfg(feature = "embedded-sparq")]
    {
        sparq_lws_core::store::EmbeddedSparqClient::in_memory()
            .expect("fresh in-memory embedded graph")
    }
    #[cfg(not(feature = "embedded-sparq"))]
    {
        sparq_lws_core::store::InMemorySparqClient::new()
    }
}

/// One assembled app over a fresh store, either demo-seeded (flag ON) or untouched (flag OFF).
struct Harness {
    app: axum::Router,
    issuer_key: KeyKit,
    client_key: KeyKit,
}

impl Harness {
    async fn new(demo_seeded: bool) -> Self {
        let store = CompositeStore::new(backend_sparq_client(), InMemoryBlobStore::new());
        if demo_seeded {
            sparq_lws_core::seed::seed_demo(&store, BASE_URL)
                .await
                .expect("demo seed");
        }
        let issuer_key = KeyKit::generate();
        let client_key = KeyKit::generate();
        let config = VerifierConfig::new(vec![common::ISSUER.to_string()], BASE_URL);
        let replay = InMemoryReplayStore::with_window(config.replay_ttl());
        let verifier = Verifier::new(config, jwks_provider(&issuer_key), replay).unwrap();
        let ctx = AuthContext::new(verifier, BASE_URL);
        let ldp = LdpState::new(store, BASE_URL);
        let app = build_router(AppState::new(ctx, ldp));
        Self {
            app,
            issuer_key,
            client_key,
        }
    }

    /// An AUTHENTICATED request (fresh DPoP-bound token + proof; the WebID is `common::WEBID` — an
    /// arbitrary authenticated agent, deliberately NOT named by any seeded ACL).
    async fn request(
        &self,
        method: &str,
        path: &str,
        content_type: Option<&str>,
        body: Body,
    ) -> axum::http::Response<Body> {
        let access = mint_access_token(&self.issuer_key, &self.client_key.thumbprint);
        let htu = format!("{BASE_URL}{path}");
        let proof = mint_dpop_proof(&self.client_key, method, &htu, &access);
        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header("authorization", format!("DPoP {access}"))
            .header("dpop", proof);
        if let Some(ct) = content_type {
            builder = builder.header("content-type", ct);
        }
        self.app
            .clone()
            .oneshot(builder.body(body).unwrap())
            .await
            .unwrap()
    }

    /// An UNAUTHENTICATED request (no Authorization / DPoP).
    async fn unauth_request(
        &self,
        method: &str,
        path: &str,
        content_type: Option<&str>,
        body: Body,
    ) -> axum::http::Response<Body> {
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(ct) = content_type {
            builder = builder.header("content-type", ct);
        }
        self.app
            .clone()
            .oneshot(builder.body(body).unwrap())
            .await
            .unwrap()
    }
}

/// Any authenticated agent (a throwaway demo identity) can create a resource in the playground —
/// the `acl:AuthenticatedAgent` Write grant flows to the new child via `acl:default`.
#[tokio::test]
async fn demo_seed_authenticated_put_under_playground_creates() {
    let h = Harness::new(true).await;
    let put = h
        .request("PUT", "/playground/note", Some("text/turtle"), Body::from(TURTLE))
        .await;
    assert_eq!(put.status(), StatusCode::CREATED);

    // And any authenticated agent (incl. the same one) reads it back.
    let get = h.request("GET", "/playground/note", None, Body::empty()).await;
    assert_eq!(get.status(), StatusCode::OK);
}

/// Anonymous writes stay rejected (the §3.1 "anonymous writes stay rejected" claim): the public
/// grant is Read-only, so the only write friction the public demo relies on — registration — holds.
#[tokio::test]
async fn demo_seed_anonymous_put_under_playground_is_unauthorized() {
    let h = Harness::new(true).await;
    let put = h
        .unauth_request("PUT", "/playground/anon", Some("text/turtle"), Body::from(TURTLE))
        .await;
    assert_eq!(put.status(), StatusCode::UNAUTHORIZED);
}

/// The `/README` banner document is anonymously dereferenceable (public Read).
#[tokio::test]
async fn demo_seed_anonymous_get_readme_is_public() {
    let h = Harness::new(true).await;
    let get = h.unauth_request("GET", "/README", None, Body::empty()).await;
    assert_eq!(get.status(), StatusCode::OK);
    // The playground container itself is anonymously readable too (all data public-readable).
    let list = h
        .unauth_request("GET", "/playground/", None, Body::empty())
        .await;
    assert_eq!(list.status(), StatusCode::OK);
}

/// NOBODY holds `acl:Control`, so even an authenticated visitor cannot rewrite the playground ACL —
/// the sandbox can never be widened, locked, or hijacked over HTTP.
#[tokio::test]
async fn demo_seed_authenticated_put_of_playground_acl_is_denied() {
    let h = Harness::new(true).await;
    // A well-formed ACL body, so the denial is authorization, not parsing.
    let widened = r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
<#public> a acl:Authorization;
          acl:agentClass <http://xmlns.com/foaf/0.1/Agent>;
          acl:accessTo </playground/>;
          acl:mode acl:Read, acl:Write, acl:Control."#;
    let put = h
        .request(
            "PUT",
            "/playground/.acl",
            Some("text/turtle"),
            Body::from(widened),
        )
        .await;
    assert_eq!(put.status(), StatusCode::FORBIDDEN);
    // Anonymous is likewise shut out (401, fail-closed before any Control question).
    let anon = h
        .unauth_request(
            "PUT",
            "/playground/.acl",
            Some("text/turtle"),
            Body::from(widened),
        )
        .await;
    assert_eq!(anon.status(), StatusCode::UNAUTHORIZED);
}

/// Flag OFF ⇒ byte-identical fail-closed boot: no `/playground/`, no README, no ACLs — every
/// request is denied exactly as on an unseeded server (the feature-off-by-default invariant).
#[tokio::test]
async fn demo_seed_flag_off_boot_has_no_playground() {
    let h = Harness::new(false).await;
    // Authenticated GET: WAC fail-closed (no ACL anywhere) → 403, never a 200 listing.
    let get = h.request("GET", "/playground/", None, Body::empty()).await;
    assert_eq!(get.status(), StatusCode::FORBIDDEN);
    // Authenticated PUT into the (nonexistent) playground: denied, nothing is auto-granted.
    let put = h
        .request("PUT", "/playground/note", Some("text/turtle"), Body::from(TURTLE))
        .await;
    assert_eq!(put.status(), StatusCode::FORBIDDEN);
    // The README does not exist / is not readable either.
    let readme = h.unauth_request("GET", "/README", None, Body::empty()).await;
    assert_eq!(readme.status(), StatusCode::UNAUTHORIZED);
}
