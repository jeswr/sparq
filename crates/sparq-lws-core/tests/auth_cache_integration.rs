// AUTHORED-BY Claude Opus 4.8
//! Round-3 verified-access-token cache — END-TO-END against the REAL `solid-oidc-verifier`.
//!
//! Unlike `auth_cache`'s in-module unit tests (which drive the cache's hit-path proof verification in
//! isolation), these tests wire a cache-enabled [`AuthContext`] over the ACTUAL verifier (with the
//! static-JWKS and in-memory-replay doubles) and assert the FULL miss->insert->hit flow with genuine
//! ES256 token and DPoP verification. They prove the cache returns the SAME verified identity on a
//! hit as the verifier produced on the miss, still rejects a swapped DPoP key (`cnf.jkt`) on a hit,
//! still rejects a replayed proof on a hit (a replay store shared with the verifier), and never turns
//! a would-be-401 into a 200.

mod common;

use std::sync::Arc;

use common::{jwks_provider, mint_access_token, mint_dpop_proof, KeyKit, BASE_URL, WEBID};
use solid_oidc_verifier::config::{StaticJwksProvider, VerifierConfig};
use solid_oidc_verifier::replay::InMemoryReplayStore;
use solid_oidc_verifier::verifier::Verifier;
use sparq_lws_core::auth::AuthContext;
use sparq_lws_core::auth_cache::{ProofPolicy, SharedReplay, VerifiedTokenCache};

type CachedCtx = AuthContext<StaticJwksProvider, SharedReplay<InMemoryReplayStore>>;

/// Build a cache-ENABLED `AuthContext` over the real verifier, sharing ONE replay store between the
/// verifier (miss path) and the cache (hit path) — the production wiring.
fn cached_ctx(issuer_key: &KeyKit) -> CachedCtx {
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
    let verifier = Verifier::new(config, jwks_provider(issuer_key), shared).expect("valid config");
    let cache = VerifiedTokenCache::new(64, policy);
    AuthContext::with_cache(verifier, BASE_URL, cache, cache_replay)
}

#[test]
fn miss_then_hit_returns_same_identity_with_fresh_proof() {
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let ctx = cached_ctx(&issuer_key);

    let access = mint_access_token(&issuer_key, &client_key.thumbprint);
    let htu = format!("{BASE_URL}/alice/data");

    // First request: a MISS -> the verifier runs -> success -> cached.
    let proof1 = mint_dpop_proof(&client_key, "GET", &htu, &access);
    let t1 = ctx
        .authenticate(
            Some(format!("DPoP {access}")),
            Some(proof1),
            "GET",
            "/alice/data",
        )
        .expect("first authed request must succeed (verifier)");
    assert_eq!(t1.web_id.as_deref(), Some(WEBID));

    // Second request: a HIT (token signature NOT re-verified) BUT a FRESH proof IS verified. Same id.
    let proof2 = mint_dpop_proof(&client_key, "GET", &htu, &access);
    let t2 = ctx
        .authenticate(
            Some(format!("DPoP {access}")),
            Some(proof2),
            "GET",
            "/alice/data",
        )
        .expect("second authed request must succeed (cache hit + fresh proof)");
    assert_eq!(t2.web_id, t1.web_id, "hit must return the same WebID");
    assert_eq!(t2.cnf_jkt, t1.cnf_jkt, "hit must return the same cnf.jkt");
    assert_eq!(t2.issuer, t1.issuer);
}

#[test]
fn swapped_dpop_key_rejected_on_hit() {
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let attacker_key = KeyKit::generate();
    let ctx = cached_ctx(&issuer_key);

    let access = mint_access_token(&issuer_key, &client_key.thumbprint);
    let htu = format!("{BASE_URL}/alice/data");

    // Prime the cache with a legitimate request.
    let proof1 = mint_dpop_proof(&client_key, "GET", &htu, &access);
    ctx.authenticate(
        Some(format!("DPoP {access}")),
        Some(proof1),
        "GET",
        "/alice/data",
    )
    .expect("priming request succeeds");

    // Attacker replays the (cached) token but signs the proof with THEIR key -> cnf.jkt mismatch on hit.
    let attacker_proof = mint_dpop_proof(&attacker_key, "GET", &htu, &access);
    let err = ctx
        .authenticate(
            Some(format!("DPoP {access}")),
            Some(attacker_proof),
            "GET",
            "/alice/data",
        )
        .expect_err("a swapped DPoP key must be rejected on a cache hit");
    match err {
        sparq_lws_core::error::ServerError::Unauthorized { status, .. } => assert_eq!(status, 401),
        other => panic!("expected 401 Unauthorized, got {other:?}"),
    }
}

#[test]
fn replayed_proof_rejected_on_hit() {
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let ctx = cached_ctx(&issuer_key);

    let access = mint_access_token(&issuer_key, &client_key.thumbprint);
    let htu = format!("{BASE_URL}/alice/data");

    // Prime the cache (miss -> insert).
    let proof1 = mint_dpop_proof(&client_key, "GET", &htu, &access);
    ctx.authenticate(
        Some(format!("DPoP {access}")),
        Some(proof1),
        "GET",
        "/alice/data",
    )
    .expect("priming request succeeds");

    // A fresh proof: succeeds (hit).
    let proof2 = mint_dpop_proof(&client_key, "GET", &htu, &access);
    ctx.authenticate(
        Some(format!("DPoP {access}")),
        Some(proof2.clone()),
        "GET",
        "/alice/data",
    )
    .expect("a fresh proof on a hit succeeds");

    // REPLAY the same proof2: rejected (the shared replay store caught it on the hit path).
    let err = ctx
        .authenticate(
            Some(format!("DPoP {access}")),
            Some(proof2),
            "GET",
            "/alice/data",
        )
        .expect_err("a replayed proof must be rejected on a cache hit");
    match err {
        sparq_lws_core::error::ServerError::Unauthorized {
            status, message, ..
        } => {
            assert_eq!(status, 401);
            assert!(message.contains("replay"), "message: {message}");
        }
        other => panic!("expected 401 Unauthorized, got {other:?}"),
    }
}

#[test]
fn first_use_of_a_proof_jti_is_not_replayed_on_the_miss_path() {
    // A jti minted on the MISS path is marked in the SHARED store; re-presenting the SAME proof (a
    // replay) on the next request — which is now a cache HIT — is caught. This proves the miss-path
    // mark and the hit-path mark target one store (no replay bypass across the cache boundary).
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let ctx = cached_ctx(&issuer_key);

    let access = mint_access_token(&issuer_key, &client_key.thumbprint);
    let htu = format!("{BASE_URL}/alice/data");

    let proof = mint_dpop_proof(&client_key, "GET", &htu, &access);
    // Miss path: verifier marks the jti.
    ctx.authenticate(
        Some(format!("DPoP {access}")),
        Some(proof.clone()),
        "GET",
        "/alice/data",
    )
    .expect("miss-path request succeeds + marks jti");

    // Hit path: the SAME proof is a replay -> rejected.
    let err = ctx
        .authenticate(
            Some(format!("DPoP {access}")),
            Some(proof),
            "GET",
            "/alice/data",
        )
        .expect_err("the same proof on the hit path is a replay");
    match err {
        sparq_lws_core::error::ServerError::Unauthorized { message, .. } => {
            assert!(message.contains("replay"), "message: {message}");
        }
        other => panic!("expected Unauthorized replay, got {other:?}"),
    }
}
