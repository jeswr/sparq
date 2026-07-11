// AUTHORED-BY Claude Opus 4.8
//! STRICT (deterministic) adversarial invariants — the gated companion to `examples/adversarial_bench.rs`.
//!
//! The example measures the adversarial arms UNDER LOAD and emits an advisory-labelled JSON report;
//! THIS file asserts the underlying SECURITY INVARIANTS deterministically so they run in `cargo test`
//! (the merge gate). Every assertion here is a reproducible pass/fail — no timing, no wall-clock.
//! Built over the FULL router (auth middleware → WAC → store) with the production verified-token
//! cache ENABLED, so the invariants are checked on the same code path the perf harness benchmarks.

mod common;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{to_bytes, Body, Bytes};
use axum::http::{Request, StatusCode};
use common::{ath, jwks_provider, mint_access_token, mint_dpop_proof, KeyKit, BASE_URL, WEBID};
use serde_json::json;
use solid_oidc_verifier::config::{StaticJwksProvider, VerifierConfig};
use solid_oidc_verifier::replay::InMemoryReplayStore;
use solid_oidc_verifier::verifier::Verifier;
use sparq_lws_core::app::{build_router, AppState};
use sparq_lws_core::auth::AuthContext;
use sparq_lws_core::auth_cache::{ProofPolicy, SharedReplay, VerifiedTokenCache};
use sparq_lws_core::ldp::handler::LdpState;
use sparq_lws_core::store::{CompositeStore, InMemoryBlobStore, InMemorySparqClient, Store};
use tower::ServiceExt;

const FOREIGN_WEBID: &str = "https://pod.example/mallory/profile/card#me";
const SMALL_TURTLE: &str =
    "<https://pod.example/alice/small#me> <http://xmlns.com/foaf/0.1/name> \"Alice\" .";

type CachedStore = CompositeStore<InMemorySparqClient, InMemoryBlobStore>;
type Router = axum::Router;

static JTI: AtomicU64 = AtomicU64::new(0);
fn unique(prefix: &str) -> String {
    format!("{prefix}-{}", JTI.fetch_add(1, Ordering::Relaxed))
}
fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Mint an RFC-9068 access token for an ARBITRARY webid (the foreign-reader arm).
fn mint_token_for(issuer: &KeyKit, cnf_jkt: &str, webid: &str) -> String {
    let header = json!({ "alg": "ES256", "typ": "at+jwt" });
    let iat = now();
    let claims = json!({
        "iss": common::ISSUER, "sub": webid, "jti": unique("at"),
        "client_id": common::CLIENT_ID, "aud": BASE_URL, "webid": webid,
        "cnf": { "jkt": cnf_jkt }, "iat": iat, "exp": iat + 300,
    });
    issuer.sign(&header, &claims)
}

/// Mint a DPoP proof REUSING a fixed jti (the replay-storm arm).
fn mint_proof_fixed_jti(
    client: &KeyKit,
    method: &str,
    url: &str,
    token: &str,
    jti: &str,
) -> String {
    let header = json!({ "alg": "ES256", "typ": "dpop+jwt", "jwk": client.public_jwk });
    let claims = json!({ "htm": method, "htu": url, "jti": jti, "iat": now(), "ath": ath(token) });
    client.sign(&header, &claims)
}

/// Assemble the FULL cache-enabled router over in-memory doubles, with a seeded owner-root ACL +
/// `/alice/small`. Returns the router + the issuer/client keys.
async fn harness() -> (Router, KeyKit, KeyKit) {
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();

    let config = VerifierConfig::new(vec![common::ISSUER.to_string()], BASE_URL);
    let policy = ProofPolicy {
        clock_tolerance_secs: config.clock_tolerance_secs,
        allow_missing_ath: config.allow_missing_ath,
        replay_fail_closed: config.replay_fail_closed,
    };
    let shared = SharedReplay::new(Arc::new(InMemoryReplayStore::with_window(
        config.replay_ttl(),
    )));
    let cache_replay = Arc::new(shared.clone());
    let verifier: Verifier<StaticJwksProvider, SharedReplay<InMemoryReplayStore>> =
        Verifier::new(config, jwks_provider(&issuer_key), shared).unwrap();
    let cache = VerifiedTokenCache::new(64, policy);
    let ctx = AuthContext::with_cache(verifier, BASE_URL, cache, cache_replay);

    let store: CachedStore =
        CompositeStore::new(InMemorySparqClient::new(), InMemoryBlobStore::new());
    seed_owner_root_acl(&store, WEBID).await;
    store
        .write(
            "https://pod.example/alice/small",
            Bytes::from_static(SMALL_TURTLE.as_bytes()),
            "text/turtle",
        )
        .await
        .unwrap();
    let ldp = LdpState::new(store, BASE_URL);
    let app = build_router(AppState::new(ctx, ldp));
    (app, issuer_key, client_key)
}

async fn seed_owner_root_acl(store: &CachedStore, owner: &str) {
    let acl = format!(
        r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
<#owner> a acl:Authorization; acl:agent <{owner}>;
  acl:accessTo <{BASE_URL}/>; acl:default <{BASE_URL}/>;
  acl:mode acl:Read, acl:Write, acl:Control."#
    );
    store
        .write(&format!("{BASE_URL}/.acl"), Bytes::from(acl), "text/turtle")
        .await
        .unwrap();
}

fn authed(method: &str, path: &str, token: &str, proof: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("DPoP {token}"))
        .header("dpop", proof)
        .body(Body::empty())
        .unwrap()
}

async fn status_of(app: &Router, req: Request<Body>) -> StatusCode {
    let resp = app.clone().oneshot(req).await.unwrap();
    let s = resp.status();
    // Drain the body so the request completes exactly as under load.
    let _ = to_bytes(resp.into_body(), usize::MAX).await;
    s
}

// -------------------------------------------------------------------------------------------------

/// A foreign authenticated reader must be denied identically for an EXISTING forbidden resource and a
/// NON-EXISTENT one — existence is not disclosed — and NEVER served a 200.
#[tokio::test]
async fn existence_is_not_disclosed_to_a_foreign_reader() {
    let (app, issuer, client) = harness().await;
    let token = mint_token_for(&issuer, &client.thumbprint, FOREIGN_WEBID);

    let p1 = mint_dpop_proof(&client, "GET", &format!("{BASE_URL}/alice/small"), &token);
    let existing = status_of(&app, authed("GET", "/alice/small", &token, &p1)).await;

    let p2 = mint_dpop_proof(&client, "GET", &format!("{BASE_URL}/alice/ghost"), &token);
    let nonexistent = status_of(&app, authed("GET", "/alice/ghost", &token, &p2)).await;

    assert_ne!(
        existing,
        StatusCode::OK,
        "foreign reader must never be served the resource"
    );
    assert_ne!(nonexistent, StatusCode::OK);
    assert_eq!(
        existing, nonexistent,
        "existing-forbidden ({existing}) and nonexistent ({nonexistent}) must return the SAME status"
    );
}

/// A replayed DPoP proof (same jti) must be rejected on the second use.
#[tokio::test]
async fn a_replayed_proof_is_rejected() {
    let (app, issuer, client) = harness().await;
    let token = mint_access_token(&issuer, &client.thumbprint);
    let jti = unique("fixed");
    let url = format!("{BASE_URL}/alice/small");

    let proof = mint_proof_fixed_jti(&client, "GET", &url, &token, &jti);
    let first = status_of(&app, authed("GET", "/alice/small", &token, &proof)).await;
    assert_eq!(first, StatusCode::OK, "first use of a fresh jti succeeds");

    // Same jti again — the replay store must reject it.
    let proof2 = mint_proof_fixed_jti(&client, "GET", &url, &token, &jti);
    let second = status_of(&app, authed("GET", "/alice/small", &token, &proof2)).await;
    assert_eq!(
        second,
        StatusCode::UNAUTHORIZED,
        "a replayed jti must be 401"
    );
}

/// Fresh jtis against one token must all keep being accepted (the replay store does not false-reject).
#[tokio::test]
async fn fresh_jtis_keep_being_accepted() {
    let (app, issuer, client) = harness().await;
    let token = mint_access_token(&issuer, &client.thumbprint);
    let url = format!("{BASE_URL}/alice/small");
    for _ in 0..50 {
        let proof = mint_dpop_proof(&client, "GET", &url, &token);
        let s = status_of(&app, authed("GET", "/alice/small", &token, &proof)).await;
        assert_eq!(s, StatusCode::OK);
    }
}

/// Busting the verified-token cache (a distinct valid token per request) must NOT weaken auth: every
/// distinct VALID token still authorizes, and a FORGED token (wrong issuer key) is still rejected.
#[tokio::test]
async fn cache_bust_does_not_weaken_auth() {
    let (app, issuer, client) = harness().await;
    let url = format!("{BASE_URL}/alice/small");

    // Distinct valid tokens — all cache misses — all still authorize.
    for _ in 0..30 {
        let token = mint_access_token(&issuer, &client.thumbprint);
        let proof = mint_dpop_proof(&client, "GET", &url, &token);
        let s = status_of(&app, authed("GET", "/alice/small", &token, &proof)).await;
        assert_eq!(s, StatusCode::OK);
    }

    // A token signed by an UNTRUSTED issuer key must be rejected even amid the miss flood.
    let attacker_issuer = KeyKit::generate();
    let forged = mint_access_token(&attacker_issuer, &client.thumbprint);
    let proof = mint_dpop_proof(&client, "GET", &url, &forged);
    let s = status_of(&app, authed("GET", "/alice/small", &forged, &proof)).await;
    assert_eq!(
        s,
        StatusCode::UNAUTHORIZED,
        "a forged-issuer token must never authorize"
    );
}

/// Malformed credentials must be rejected, never served a 200.
#[tokio::test]
async fn bogus_credentials_never_authorize() {
    let (app, issuer, client) = harness().await;
    let token = mint_access_token(&issuer, &client.thumbprint);

    // Valid token, garbage proof.
    let s1 = status_of(
        &app,
        authed("GET", "/alice/small", &token, "not.a.valid-proof"),
    )
    .await;
    assert_ne!(s1, StatusCode::OK);

    // Garbage token, garbage proof.
    let s2 = status_of(
        &app,
        authed("GET", "/alice/small", "not-a-jwt", "not.a.valid-proof"),
    )
    .await;
    assert_ne!(s2, StatusCode::OK);
}

/// After a flood of hostile traffic the WAC decisions on the live server are unchanged: the owner is
/// still authorized, the foreign reader still denied (the attack corrupted no state).
#[tokio::test]
async fn wac_holds_after_an_attack_flood() {
    let (app, issuer, client) = harness().await;
    let owner = mint_access_token(&issuer, &client.thumbprint);
    let foreign = mint_token_for(&issuer, &client.thumbprint, FOREIGN_WEBID);
    let url = format!("{BASE_URL}/alice/small");

    // Flood: replays, forged tokens, bogus proofs.
    let fixed = unique("flood");
    for _ in 0..40 {
        let replay = mint_proof_fixed_jti(&client, "GET", &url, &owner, &fixed);
        let _ = status_of(&app, authed("GET", "/alice/small", &owner, &replay)).await;
        let forged = mint_access_token(&KeyKit::generate(), &client.thumbprint);
        let fp = mint_dpop_proof(&client, "GET", &url, &forged);
        let _ = status_of(&app, authed("GET", "/alice/small", &forged, &fp)).await;
    }

    // Post-attack: owner authorized, foreign denied.
    let op = mint_dpop_proof(&client, "GET", &url, &owner);
    assert_eq!(
        status_of(&app, authed("GET", "/alice/small", &owner, &op)).await,
        StatusCode::OK,
        "owner must still be authorized after the flood"
    );
    let fp = mint_dpop_proof(&client, "GET", &url, &foreign);
    assert_ne!(
        status_of(&app, authed("GET", "/alice/small", &foreign, &fp)).await,
        StatusCode::OK,
        "foreign reader must still be denied after the flood"
    );
}
