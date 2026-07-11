// AUTHORED-BY Claude Opus 4.8
//! PoP Tier-1b — RFC 8705 mTLS cert-bound-token confirmation dispatch, fail-closed tests.
//!
//! Drives `AuthContext::authenticate_with_cert` (the seam the mTLS acceptor's per-connection
//! `ConnPop` feeds) through the REAL `solid-oidc-verifier` (static-JWKS + in-memory-replay doubles).
//!
//! Two postures are exercised:
//! - `require_dpop(false)` (the `ctx()` helper) isolates the Tier-1b DISPATCH logic (`finalize_pop`):
//!   a Bearer cert-bound token is admitted by the verifier regardless, and the dispatch adds the
//!   fail-closed cert match on top.
//! - `require_dpop(true)` (the `ctx_require_dpop()` helper) is the PoP Tier-1 LIVE path in the real
//!   Solid posture: the verifier itself now ADMITS a cert-bound Bearer token when the RS-verified
//!   client-certificate thumbprint is threaded into `AuthRequest.client_cert_x5t_s256` (verifier rev
//!   `89c8962` — design §10 bead 1, now landed). These are the "authenticates END-TO-END through
//!   `verify()`, not merely at dispatch" tests: a matching cert authenticates; no cert / wrong cert /
//!   plain bearer are 401; and with the mTLS flag OFF the cert is never threaded so a cert-bound
//!   Bearer is rejected exactly as before (byte-identical CTH/conformance posture).
//!
//! Fail-closed invariants asserted (per the task brief):
//! - a cert-bound token presented with NO client cert ⇒ 401;
//! - a WRONG cert thumbprint ⇒ 401;
//! - a MATCHING cert ⇒ authenticated;
//! - a MALFORMED cert binding ⇒ 401 (never collapsed to "unbound");
//! - NO method-downgrade — a cert-bound token is not accepted as bearer/DPoP without the cert, and a
//!   multi-binding token (both `cnf.jkt` + `cnf.x5t#S256`) is refused even with a matching cert + proof;
//! - the DPoP path is UNCHANGED when the flag is on (cert ignored) AND when off (no dispatch at all).

mod common;

use common::{
    cert_x5t_s256, jwks_provider, mint_access_token, mint_cert_bound_access_token, mint_dpop_proof,
    mint_dual_bound_access_token, mint_malformed_cert_bound_access_token,
    mint_unbound_access_token, KeyKit, BASE_URL, WEBID,
};
use std::sync::Arc;

use solid_oidc_verifier::config::VerifierConfig;
use solid_oidc_verifier::replay::InMemoryReplayStore;
use solid_oidc_verifier::verifier::Verifier;
use sparq_lws_core::auth::AuthContext;
use sparq_lws_core::auth_cache::{ProofPolicy, SharedReplay, VerifiedTokenCache};
use sparq_lws_core::pop::cert_bound::CertThumbprint;

type TestVerifier = Verifier<solid_oidc_verifier::config::StaticJwksProvider, InMemoryReplayStore>;
type CachedCtx =
    AuthContext<solid_oidc_verifier::config::StaticJwksProvider, SharedReplay<InMemoryReplayStore>>;

/// A cache-ENABLED `AuthContext` (require_dpop=false, mTLS dispatch ON) — for the roborev-requested
/// cache-interaction tests (a rejected multi-binding token must not be served from the cache).
fn cached_ctx_mtls(issuer_key: &KeyKit) -> CachedCtx {
    let config =
        VerifierConfig::new(vec![common::ISSUER.to_string()], BASE_URL).require_dpop(false);
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
    AuthContext::with_cache(verifier, BASE_URL, cache, cache_replay).with_mtls_bound_tokens(true)
}

/// Two distinct, stable client-certificate DER stand-ins (any bytes — the binding only ever hashes the
/// DER; it never parses X.509).
const CERT_A: &[u8] = b"-----client-cert-A-----";
const CERT_B: &[u8] = b"-----client-cert-B-----";

/// An `AuthContext` whose verifier trusts `issuer_key`, with `require_dpop=false` (so a Bearer
/// cert-bound token is verified), and the mTLS confirmation dispatch turned ON or OFF per `mtls`.
fn ctx(
    issuer_key: &KeyKit,
    mtls: bool,
) -> AuthContext<solid_oidc_verifier::config::StaticJwksProvider, InMemoryReplayStore> {
    let config =
        VerifierConfig::new(vec![common::ISSUER.to_string()], BASE_URL).require_dpop(false);
    let replay = InMemoryReplayStore::with_window(config.replay_ttl());
    let verifier: TestVerifier =
        Verifier::new(config, jwks_provider(issuer_key), replay).expect("valid config");
    AuthContext::new(verifier, BASE_URL).with_mtls_bound_tokens(mtls)
}

/// An `AuthContext` in the REAL Solid posture (`require_dpop=true`) with the mTLS confirmation
/// dispatch turned ON or OFF per `mtls`. This is the PoP Tier-1 LIVE configuration: DPoP is mandatory
/// for every non-cert-bound token, and a cert-bound Bearer token is admitted ONLY when the verifier
/// receives the RS-verified client-certificate thumbprint (which `authenticate_with_cert` threads into
/// `AuthRequest.client_cert_x5t_s256` only when `mtls` is on).
fn ctx_require_dpop(
    issuer_key: &KeyKit,
    mtls: bool,
) -> AuthContext<solid_oidc_verifier::config::StaticJwksProvider, InMemoryReplayStore> {
    let config = VerifierConfig::new(vec![common::ISSUER.to_string()], BASE_URL).require_dpop(true);
    let replay = InMemoryReplayStore::with_window(config.replay_ttl());
    let verifier: TestVerifier =
        Verifier::new(config, jwks_provider(issuer_key), replay).expect("valid config");
    AuthContext::new(verifier, BASE_URL).with_mtls_bound_tokens(mtls)
}

/// The presented-cert thumbprint the acceptor would compute for `der`.
fn presented(der: &[u8]) -> CertThumbprint {
    CertThumbprint::from_cert_der(der)
}

#[test]
fn cert_bound_token_with_no_client_cert_is_denied() {
    // Fail-closed: a cert-bound token on a connection that presented NO client certificate ⇒ 401
    // (never a downgrade to bearer acceptance).
    let issuer_key = KeyKit::generate();
    let ctx = ctx(&issuer_key, true);
    let access = mint_cert_bound_access_token(&issuer_key, &cert_x5t_s256(CERT_A));

    let err = ctx
        .authenticate_with_cert(
            Some(format!("Bearer {access}")),
            None,
            "GET",
            "/alice/data",
            None, // no client certificate on the connection
        )
        .expect_err("cert-bound token + no client cert must be denied");
    assert_eq!(err.status().as_u16(), 401);
}

#[test]
fn cert_bound_token_with_wrong_cert_is_denied() {
    // The token is bound to CERT_A but the connection presented CERT_B ⇒ 401.
    let issuer_key = KeyKit::generate();
    let ctx = ctx(&issuer_key, true);
    let access = mint_cert_bound_access_token(&issuer_key, &cert_x5t_s256(CERT_A));

    let err = ctx
        .authenticate_with_cert(
            Some(format!("Bearer {access}")),
            None,
            "GET",
            "/alice/data",
            Some(&presented(CERT_B)),
        )
        .expect_err("cert-bound token + wrong client cert must be denied");
    assert_eq!(err.status().as_u16(), 401);
}

#[test]
fn cert_bound_token_with_matching_cert_authenticates() {
    // The happy path: the presented cert matches the token's cnf.x5t#S256 ⇒ authenticated.
    let issuer_key = KeyKit::generate();
    let ctx = ctx(&issuer_key, true);
    let access = mint_cert_bound_access_token(&issuer_key, &cert_x5t_s256(CERT_A));

    let token = ctx
        .authenticate_with_cert(
            Some(format!("Bearer {access}")),
            None,
            "GET",
            "/alice/data",
            Some(&presented(CERT_A)),
        )
        .expect("cert-bound token + matching client cert must authenticate");
    assert_eq!(token.web_id.as_deref(), Some(WEBID));
}

#[test]
fn malformed_cert_binding_is_denied_not_treated_as_unbound() {
    // A present-but-malformed cnf.x5t#S256 must fail CLOSED — even with a matching-looking cert
    // presented, a broken binding is never collapsed to "unbound"/accepted.
    let issuer_key = KeyKit::generate();
    let ctx = ctx(&issuer_key, true);
    let access = mint_malformed_cert_bound_access_token(&issuer_key);

    let err = ctx
        .authenticate_with_cert(
            Some(format!("Bearer {access}")),
            None,
            "GET",
            "/alice/data",
            Some(&presented(CERT_A)),
        )
        .expect_err("a malformed cert binding must be denied fail-closed");
    assert_eq!(err.status().as_u16(), 401);
}

#[test]
fn multi_binding_token_is_refused_even_with_matching_cert_and_proof() {
    // NO downgrade / no partial-satisfaction: a token declaring BOTH cnf.jkt and cnf.x5t#S256 is
    // refused (combined verification unimplemented) even when presented as DPoP with a valid proof AND
    // the matching client cert — satisfying only one binding would bypass the other.
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let ctx = ctx(&issuer_key, true);
    let access =
        mint_dual_bound_access_token(&issuer_key, &client_key.thumbprint, &cert_x5t_s256(CERT_A));
    let htu = format!("{BASE_URL}/alice/data");
    let proof = mint_dpop_proof(&client_key, "GET", &htu, &access);

    let err = ctx
        .authenticate_with_cert(
            Some(format!("DPoP {access}")),
            Some(proof),
            "GET",
            "/alice/data",
            Some(&presented(CERT_A)),
        )
        .expect_err("a multi-binding token must be refused (no partial satisfaction)");
    assert_eq!(err.status().as_u16(), 401);
}

#[test]
fn dpop_token_is_unchanged_when_mtls_on_cert_ignored() {
    // A normal DPoP-bound token authenticates identically with the mTLS flag ON — a certificate is
    // neither required nor consulted for a DPoP token (no cross-method interference).
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let ctx = ctx(&issuer_key, true);
    let access = mint_access_token(&issuer_key, &client_key.thumbprint);
    let htu = format!("{BASE_URL}/alice/data");
    let proof = mint_dpop_proof(&client_key, "GET", &htu, &access);

    // With NO client cert presented — a DPoP token must still authenticate (the cert is irrelevant).
    let token = ctx
        .authenticate_with_cert(
            Some(format!("DPoP {access}")),
            Some(proof),
            "GET",
            "/alice/data",
            None,
        )
        .expect("a DPoP token must authenticate regardless of the mTLS flag / presented cert");
    assert_eq!(token.web_id.as_deref(), Some(WEBID));
}

#[test]
fn cert_bound_dispatch_is_inactive_when_flag_off() {
    // Flag OFF ⇒ the presented certificate is NEVER threaded into the verifier and NO confirmation
    // dispatch runs. Even with the MATCHING client certificate on the wire, a cert-bound Bearer token is
    // rejected — because `authenticate_with_cert` supplies `client_cert_x5t_s256 = None` when the flag is
    // off, so the verifier (which always enforces an RFC 8705 cert binding fail-closed) sees no cert to
    // satisfy it. This proves the mTLS path (both the verifier-threading AND the dispatch) genuinely
    // gates on the flag and does not silently activate when off.
    let issuer_key = KeyKit::generate();
    let ctx_off = ctx(&issuer_key, false);
    let access = mint_cert_bound_access_token(&issuer_key, &cert_x5t_s256(CERT_A));

    // Matching cert on the wire, but flag off ⇒ cert not threaded ⇒ verifier rejects fail-closed.
    let err = ctx_off
        .authenticate_with_cert(
            Some(format!("Bearer {access}")),
            None,
            "GET",
            "/alice/data",
            Some(&presented(CERT_A)),
        )
        .expect_err("flag off ⇒ cert never threaded ⇒ cert-bound Bearer rejected fail-closed");
    assert_eq!(err.status().as_u16(), 401);

    // And the plain 4-arg `authenticate` (no cert) behaves identically for a DPoP token, flag on or off.
    let client_key = KeyKit::generate();
    let dpop_access = mint_access_token(&issuer_key, &client_key.thumbprint);
    let htu = format!("{BASE_URL}/alice/data");
    let proof = mint_dpop_proof(&client_key, "GET", &htu, &dpop_access);
    let token = ctx_off
        .authenticate(
            Some(format!("DPoP {dpop_access}")),
            Some(proof),
            "GET",
            "/alice/data",
        )
        .expect("DPoP token authenticates via the 4-arg path with the flag off");
    assert_eq!(token.web_id.as_deref(), Some(WEBID));
}

#[test]
fn cache_enabled_multi_binding_token_is_rejected_both_times_and_not_served_from_cache() {
    // roborev Medium regression guard: with the token cache ENABLED + mTLS ON, a multi-binding token
    // (cnf.jkt + cnf.x5t#S256, presented as DPoP with a valid proof) is rejected on the first attempt
    // AND on a second attempt with a FRESH proof — never accepted, never served from the cache (the
    // insert is skipped for a cert-binding-carrying token, so it cannot occupy an LRU slot / be replayed
    // as a hit). Consistent 401s prove finalize_pop runs on both the miss and any hit path.
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let ctx = cached_ctx_mtls(&issuer_key);
    let access =
        mint_dual_bound_access_token(&issuer_key, &client_key.thumbprint, &cert_x5t_s256(CERT_A));
    let htu = format!("{BASE_URL}/alice/data");

    for _ in 0..2 {
        // A fresh proof each attempt (distinct jti) so a rejection is not merely a replay rejection.
        let proof = mint_dpop_proof(&client_key, "GET", &htu, &access);
        let err = ctx
            .authenticate_with_cert(
                Some(format!("DPoP {access}")),
                Some(proof),
                "GET",
                "/alice/data",
                Some(&presented(CERT_A)),
            )
            .expect_err("a multi-binding token must be refused on every attempt, cache or not");
        assert_eq!(err.status().as_u16(), 401);
    }
}

// ---------------------------------------------------------------------------------------------------
// PoP Tier-1 LIVE — the REAL Solid posture (`require_dpop=true`). These assert a cert-bound Bearer
// token now AUTHENTICATES END-TO-END through the verifier's `verify()` (not merely at the dispatch
// layer): the RS-verified certificate thumbprint is threaded into `AuthRequest.client_cert_x5t_s256`
// and the verifier admits + enforces it. All the fail-closed cases must still hold under require_dpop.
// ---------------------------------------------------------------------------------------------------

#[test]
fn live_cert_bound_bearer_authenticates_through_verify_under_require_dpop() {
    // THE Tier-1 LIVE happy path: under the real Solid posture (require_dpop=true, mTLS ON), a Bearer
    // cert-bound token whose cnf.x5t#S256 matches the presented client cert authenticates THROUGH the
    // verifier's verify() — before this rev, verify() rejected any Bearer under require_dpop. No DPoP
    // proof is presented; the certificate is the proof-of-possession (RFC 8705 §3).
    let issuer_key = KeyKit::generate();
    let ctx = ctx_require_dpop(&issuer_key, true);
    let access = mint_cert_bound_access_token(&issuer_key, &cert_x5t_s256(CERT_A));

    let token = ctx
        .authenticate_with_cert(
            Some(format!("Bearer {access}")),
            None, // no DPoP proof — the client certificate is the PoP
            "GET",
            "/alice/data",
            Some(&presented(CERT_A)),
        )
        .expect("a cert-bound Bearer + matching cert must authenticate through verify() under require_dpop");
    assert_eq!(token.web_id.as_deref(), Some(WEBID));
}

#[test]
fn live_cert_bound_bearer_no_cert_is_401_under_require_dpop() {
    // Fail-closed at the verifier: a cert-bound Bearer under require_dpop with NO client certificate
    // presented ⇒ 401 (the verifier's validate_cert_binding rejects: no presented cert to satisfy the
    // binding, and there is no DPoP downgrade).
    let issuer_key = KeyKit::generate();
    let ctx = ctx_require_dpop(&issuer_key, true);
    let access = mint_cert_bound_access_token(&issuer_key, &cert_x5t_s256(CERT_A));

    let err = ctx
        .authenticate_with_cert(
            Some(format!("Bearer {access}")),
            None,
            "GET",
            "/alice/data",
            None, // no client certificate
        )
        .expect_err("cert-bound Bearer + no cert must be 401 under require_dpop");
    assert_eq!(err.status().as_u16(), 401);
}

#[test]
fn live_cert_bound_bearer_wrong_cert_is_401_under_require_dpop() {
    // Fail-closed at the verifier: the token is bound to CERT_A, the connection presented CERT_B ⇒ the
    // constant-time thumbprint compare fails ⇒ 401 (no bare-bearer fallback).
    let issuer_key = KeyKit::generate();
    let ctx = ctx_require_dpop(&issuer_key, true);
    let access = mint_cert_bound_access_token(&issuer_key, &cert_x5t_s256(CERT_A));

    let err = ctx
        .authenticate_with_cert(
            Some(format!("Bearer {access}")),
            None,
            "GET",
            "/alice/data",
            Some(&presented(CERT_B)),
        )
        .expect_err("cert-bound Bearer + wrong cert must be 401 under require_dpop");
    assert_eq!(err.status().as_u16(), 401);
}

#[test]
fn live_plain_bearer_still_rejected_under_require_dpop() {
    // The DPoP-mandatory posture is UNCHANGED for non-cert-bound tokens: a plain UNBOUND Bearer token
    // (no cnf at all) is still rejected under require_dpop, even with a client certificate on the wire —
    // a certificate never admits a token that is not itself cert-bound.
    let issuer_key = KeyKit::generate();
    let ctx = ctx_require_dpop(&issuer_key, true);
    let access = mint_unbound_access_token(&issuer_key);

    let err = ctx
        .authenticate_with_cert(
            Some(format!("Bearer {access}")),
            None,
            "GET",
            "/alice/data",
            Some(&presented(CERT_A)), // a cert on the wire must not admit a non-cert-bound token
        )
        .expect_err("a plain unbound Bearer must remain 401 under require_dpop");
    assert_eq!(err.status().as_u16(), 401);
}

#[test]
fn live_dpop_bound_token_presented_as_bearer_with_cert_is_401_under_require_dpop() {
    // No token-replay downgrade: a DPoP-BOUND token (cnf.jkt, no cnf.x5t#S256) mis-presented as
    // `Bearer` is rejected under require_dpop EVEN WITH a matching-looking client certificate on the
    // wire. The certificate must NEVER admit a token that is not itself cert-bound — a captured
    // DPoP-bound token replayed as Bearer would otherwise skip its proof-of-possession. (`token_is_cert_bound`
    // is false for a cnf.jkt-only token, so the verifier's Bearer/require_dpop gate rejects it before the
    // cert path is ever consulted.)
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let ctx = ctx_require_dpop(&issuer_key, true);
    let access = mint_access_token(&issuer_key, &client_key.thumbprint);

    let err = ctx
        .authenticate_with_cert(
            Some(format!("Bearer {access}")), // DPoP-bound token, but presented as Bearer
            None,
            "GET",
            "/alice/data",
            Some(&presented(CERT_A)), // a cert on the wire must not admit a DPoP-bound Bearer
        )
        .expect_err(
            "a cnf.jkt token presented as Bearer must be 401 under require_dpop, cert or not",
        );
    assert_eq!(err.status().as_u16(), 401);
}

#[test]
fn live_flag_off_is_byte_identical_cert_bound_bearer_rejected_under_require_dpop() {
    // FLAG-OFF byte-identical proof (item 4): with the mTLS flag OFF, `authenticate_with_cert` NEVER
    // threads the presented certificate into the verifier — so under require_dpop a cert-bound Bearer
    // token is rejected EXACTLY as it was pre-Tier-1, EVEN WHEN the matching client certificate is on
    // the connection. The verifier's CTH/conformance posture (DPoP mandatory, Bearer refused) is
    // unchanged when the flag is off.
    let issuer_key = KeyKit::generate();
    let ctx_off = ctx_require_dpop(&issuer_key, false);
    let access = mint_cert_bound_access_token(&issuer_key, &cert_x5t_s256(CERT_A));

    let err = ctx_off
        .authenticate_with_cert(
            Some(format!("Bearer {access}")),
            None,
            "GET",
            "/alice/data",
            Some(&presented(CERT_A)), // matching cert present, but the flag-off path ignores it entirely
        )
        .expect_err("flag OFF ⇒ cert never threaded ⇒ cert-bound Bearer rejected under require_dpop (byte-identical)");
    assert_eq!(err.status().as_u16(), 401);
}

#[test]
fn live_dpop_token_authenticates_unchanged_under_require_dpop() {
    // A normal DPoP-bound token authenticates through the LIVE posture untouched — the cert-bound admission
    // is strictly additive and does not disturb the DPoP path (no proof/cert cross-interference).
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let ctx = ctx_require_dpop(&issuer_key, true);
    let access = mint_access_token(&issuer_key, &client_key.thumbprint);
    let htu = format!("{BASE_URL}/alice/data");
    let proof = mint_dpop_proof(&client_key, "GET", &htu, &access);

    let token = ctx
        .authenticate_with_cert(
            Some(format!("DPoP {access}")),
            Some(proof),
            "GET",
            "/alice/data",
            None,
        )
        .expect("a DPoP token authenticates unchanged under require_dpop with mTLS on");
    assert_eq!(token.web_id.as_deref(), Some(WEBID));
}

#[test]
fn cache_enabled_pure_dpop_token_still_caches_and_authenticates() {
    // Sanity: the cache-skip guard is NARROW — a PURELY DPoP-bound token (no cert binding) still caches
    // and authenticates normally under mTLS-on (miss then hit with a fresh proof), so the fix did not
    // disable the cache for legitimate DPoP traffic.
    let issuer_key = KeyKit::generate();
    let client_key = KeyKit::generate();
    let ctx = cached_ctx_mtls(&issuer_key);
    let access = mint_access_token(&issuer_key, &client_key.thumbprint);
    let htu = format!("{BASE_URL}/alice/data");

    for _ in 0..2 {
        let proof = mint_dpop_proof(&client_key, "GET", &htu, &access);
        let token = ctx
            .authenticate_with_cert(
                Some(format!("DPoP {access}")),
                Some(proof),
                "GET",
                "/alice/data",
                None,
            )
            .expect("a pure DPoP token authenticates (miss then cache hit) under mTLS-on");
        assert_eq!(token.web_id.as_deref(), Some(WEBID));
    }
}
