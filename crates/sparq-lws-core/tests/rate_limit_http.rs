// AUTHORED-BY Claude Opus 4.8
//! End-to-end HTTP tests for the PRE-CRYPTO per-IP rate-limit layer through the assembled router.
//!
//! These prove the SECURITY-critical properties of the rate limiter, not just the happy path:
//! - a per-IP flood beyond the bucket returns **429 + `Retry-After`** and is NEVER forwarded to the
//!   inner stack — so a rate-limited request can never reach auth, never become a 2xx (no auth bypass);
//! - **per-IP isolation**: one IP's flood does NOT throttle a DIFFERENT IP in the same window;
//! - the **default config never trips** at a modest sequential rate (the conformance/normal-use
//!   guarantee — `passed=41/41` is the live proof, this pins it at the unit/router level);
//! - the **health probes (`/livez`, `/readyz`) are EXEMPT** — 200 even under a tripped limit;
//! - **`ConnectInfo` absent ⇒ FAIL OPEN** (the request proceeds to auth — a 401 here, NOT a 429 — so a
//!   limiter wiring gap never denies all traffic, and "fail-open" means "proceed to auth", never "bypass");
//! - **XFF is ignored by default** (a spoofed rotating XFF from one peer cannot dodge the per-IP limit).
//!
//! The trick for a deterministic router-level test: `tower::ServiceExt::oneshot` on a plain `Router`
//! does NOT inject `ConnectInfo` (that only happens via `into_make_service_with_connect_info`), so we
//! manually insert `ConnectInfo<SocketAddr>` into each request's extensions to model a given peer IP —
//! exactly the extension the live serve path populates. The limiter is built with a TIGHT capacity so a
//! couple of requests trip it, proving the short-circuit BEFORE auth.

mod common;

use std::net::SocketAddr;

use axum::body::{to_bytes, Body};
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use common::{jwks_provider, KeyKit, BASE_URL};
use solid_oidc_verifier::config::VerifierConfig;
use solid_oidc_verifier::replay::InMemoryReplayStore;
use solid_oidc_verifier::verifier::Verifier;
use sparq_lws_core::app::{
    build_router_with_overload, AppState, OverloadConfig, LIVEZ_PATH, READYZ_PATH,
};
use sparq_lws_core::auth::AuthContext;
use sparq_lws_core::ldp::handler::LdpState;
use sparq_lws_core::overload::AdmissionControl;
use sparq_lws_core::rate_limit::RateLimiter;
use sparq_lws_core::store::{CompositeStore, InMemoryBlobStore, InMemorySparqClient};
use tower::ServiceExt;

/// Build a router with the FULL production layering (rate limiter OUTERMOST, then admission, then auth)
/// over a real (in-memory) LDP+auth stack, with the rate limiter sized to `(rate, burst)` and the given
/// `trusted_proxy_hops` / `exempt_loopback`. A generous admission ceiling (won't shed) so only the rate
/// limiter is exercised. `exempt_internal` is set FALSE for these tests (they drive PUBLIC TEST-NET IPs
/// through the bucket; with the default-on internal exemption those would be exempted and the bucket
/// never exercised — the internal-exemption behaviour itself is unit-tested in `rate_limit.rs`).
fn app_with_rate_limit(
    rate: f64,
    burst: f64,
    trusted_proxy_hops: usize,
    exempt_loopback: bool,
) -> axum::Router {
    let issuer_key = KeyKit::generate();
    let config = VerifierConfig::new(vec![common::ISSUER.to_string()], BASE_URL);
    let replay = InMemoryReplayStore::with_window(config.replay_ttl());
    let verifier = Verifier::new(config, jwks_provider(&issuer_key), replay).unwrap();
    let ctx = AuthContext::new(verifier, BASE_URL);
    let store = CompositeStore::new(InMemorySparqClient::new(), InMemoryBlobStore::new());
    let ldp = LdpState::new(store, BASE_URL);

    let overload = OverloadConfig {
        admission: AdmissionControl::new(10_000),
        request_timeout: None,
        rate_limiter: Some(RateLimiter::new(
            rate,
            burst,
            trusted_proxy_hops,
            exempt_loopback,
            /* exempt_internal = */ false,
        )),
        body_limit_bytes: sparq_lws_core::body_limit::DEFAULT_MAX_BODY_BYTES,
    };
    build_router_with_overload(AppState::new(ctx, ldp), overload)
}

/// A GET request to a protected path, tagged with a peer `ConnectInfo<SocketAddr>` (the extension the
/// live serve path injects). Optionally carries an `X-Forwarded-For` header.
fn req_from(peer: &str, xff: Option<&str>) -> Request<Body> {
    let addr: SocketAddr = peer.parse().unwrap();
    let mut builder = Request::builder()
        .method("GET")
        .uri(format!("{BASE_URL}/alice/private"));
    if let Some(h) = xff {
        builder = builder.header("x-forwarded-for", h);
    }
    let mut req = builder.body(Body::empty()).unwrap();
    req.extensions_mut().insert(ConnectInfo(addr));
    req
}

#[tokio::test]
async fn flood_from_one_ip_429s_and_a_different_ip_is_unaffected() {
    // capacity 2, negligible refill. IP A makes 2 (the burst) + a 3rd that is 429'd; IP B in the same
    // window still has its FULL burst — proving PER-IP isolation (a shared-bucket mutation would 429 B).
    let app = app_with_rate_limit(0.0001, 2.0, 0, false);

    // A: two pass (anonymous ⇒ reach auth ⇒ 401), the third is 429 BEFORE auth.
    for i in 0..2 {
        let resp = app
            .clone()
            .oneshot(req_from("203.0.113.10:5000", None))
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "A request {i} is within the burst ⇒ reaches auth ⇒ 401 (anonymous), not 429"
        );
    }
    let limited = app
        .clone()
        .oneshot(req_from("203.0.113.10:5001", None))
        .await
        .unwrap();
    assert_eq!(
        limited.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "A's 3rd request exceeds the burst ⇒ 429 (rate-limited BEFORE auth)"
    );
    // The 429 carries a jittered Retry-After + no-store, and a body that says rate limit (not overload).
    let retry: u64 = limited
        .headers()
        .get(axum::http::header::RETRY_AFTER)
        .expect("429 must carry Retry-After")
        .to_str()
        .unwrap()
        .parse()
        .expect("Retry-After is a seconds integer");
    assert!(
        (1..=5).contains(&retry),
        "Retry-After {retry} must be within the jittered band"
    );
    assert_eq!(
        limited
            .headers()
            .get(axum::http::header::CACHE_CONTROL)
            .unwrap(),
        "no-store"
    );
    let body = to_bytes(limited.into_body(), 64 * 1024).await.unwrap();
    let text = String::from_utf8_lossy(&body).to_lowercase();
    assert!(text.contains("429") && text.contains("rate limit"));

    // B (a DIFFERENT IP) is unaffected — its requests still reach auth (401), not 429.
    for i in 0..2 {
        let resp = app
            .clone()
            .oneshot(req_from("198.51.100.99:6000", None))
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "B request {i} is unaffected by A's flood (independent per-IP bucket) ⇒ reaches auth"
        );
    }
}

#[tokio::test]
async fn rate_limited_request_never_reaches_auth_no_bypass() {
    // The adversarial case: a request that WOULD authenticate must STILL be 429'd (and never run) when
    // the source is over its limit — the limit short-circuits BEFORE auth/crypto. We drive an anonymous
    // request (which would otherwise 401); the KEY assertion is the rate-limited one is 429, NOT any
    // 2xx and NOT a 401-from-auth — i.e. the inner stack (auth + the verifier crypto) never ran.
    //
    // Mutation kill (no-auth-bypass / always-allow): an "always allow" limiter mutation would let the
    // 3rd request through to auth ⇒ 401, FAILING this assertion (we require 429). A 429 here is the
    // only outcome that proves the short-circuit happened before auth.
    let app = app_with_rate_limit(0.0001, 1.0, 0, false); // capacity 1 ⇒ the 2nd request is limited

    let first = app
        .clone()
        .oneshot(req_from("192.0.2.77:4000", None))
        .await
        .unwrap();
    assert_eq!(
        first.status(),
        StatusCode::UNAUTHORIZED,
        "the first request is within the burst ⇒ reaches auth ⇒ 401 (proves auth DOES run normally)"
    );

    let second = app
        .clone()
        .oneshot(req_from("192.0.2.77:4001", None))
        .await
        .unwrap();
    let status = second.status();
    assert_eq!(
        status,
        StatusCode::TOO_MANY_REQUESTS,
        "the 2nd request is over the limit ⇒ 429 BEFORE auth (never reaches the verifier)"
    );
    assert!(
        !status.is_success(),
        "a rate-limited request must NEVER be a 2xx (that would be an auth bypass)"
    );
    assert_ne!(
        status,
        StatusCode::UNAUTHORIZED,
        "a 429 (not a 401) proves the request was rejected BEFORE the auth layer ran"
    );
}

#[tokio::test]
async fn modest_sequential_rate_under_default_is_never_limited() {
    // Under the DEFAULT config, a modest sequential burst from one IP must never be 429'd — this is the
    // conformance/normal-use guarantee at the router level (the CTH's `passed=41/41` is the live proof).
    // We use a public (non-loopback) IP so the burst alone — not the loopback exemption — proves it.
    let app = app_with_rate_limit(
        sparq_lws_core::rate_limit::DEFAULT_RATE_PER_IP,
        sparq_lws_core::rate_limit::DEFAULT_BURST,
        0,
        false,
    );
    for i in 0..64u32 {
        let resp = app
            .clone()
            .oneshot(req_from("192.0.2.123:7000", None))
            .await
            .unwrap();
        assert_ne!(
            resp.status(),
            StatusCode::TOO_MANY_REQUESTS,
            "request {i} under the generous default must NOT be rate-limited"
        );
    }
}

#[tokio::test]
async fn health_probes_are_exempt_even_under_a_tripped_limit() {
    // capacity 1 from the loopback peer the probes will appear to come from; but the probes must answer
    // 200 regardless — they are mounted OUTSIDE the rate-limit layer AND skipped by path in the
    // middleware. We exhaust a non-health path first to trip the limit for that peer, then probe.
    let app = app_with_rate_limit(0.0001, 1.0, 0, false);

    // Trip the limit for the peer 127.0.0.1 on a protected path (exemption is OFF so loopback IS limited).
    let _ = app
        .clone()
        .oneshot(req_from("127.0.0.1:9000", None))
        .await
        .unwrap();
    let tripped = app
        .clone()
        .oneshot(req_from("127.0.0.1:9001", None))
        .await
        .unwrap();
    assert_eq!(
        tripped.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "sanity: the protected path IS rate-limited for this peer (exemption off)"
    );

    // The health probes from the SAME (now-limited) peer must still be 200.
    for path in [LIVEZ_PATH, READYZ_PATH] {
        let mut req = Request::builder()
            .method("GET")
            .uri(format!("{BASE_URL}{path}"))
            .body(Body::empty())
            .unwrap();
        req.extensions_mut()
            .insert(ConnectInfo("127.0.0.1:9002".parse::<SocketAddr>().unwrap()));
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "{path} must be 200 even under a tripped limit (health is rate-limit-EXEMPT)"
        );
    }
}

#[tokio::test]
async fn connect_info_absent_fails_open_to_auth() {
    // FAIL-OPEN: a request with NO ConnectInfo (the should-never-happen wiring gap) must PROCEED to
    // auth (a 401 here for the anonymous caller), NOT be denied. "Fail-open" = proceed to auth, which
    // STILL gates the request — never "bypass auth". We set a tiny capacity to show the limiter would
    // otherwise be in play, but with no peer IP it cannot key a bucket, so it lets the request through.
    //
    // Mutation kill (fail-CLOSED mutation): a limiter that DENIED on absent ConnectInfo would 429/503
    // here, failing this assertion (we require the auth 401).
    let app = app_with_rate_limit(0.0001, 1.0, 0, false);

    // No ConnectInfo inserted — and send several so a fail-closed-on-absent limiter would surely 429.
    for _ in 0..3 {
        let req = Request::builder()
            .method("GET")
            .uri(format!("{BASE_URL}/alice/private"))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::UNAUTHORIZED,
            "with no ConnectInfo the limiter FAILS OPEN ⇒ the request reaches auth ⇒ 401 (not 429)"
        );
    }
}

#[tokio::test]
async fn spoofed_rotating_xff_does_not_dodge_the_per_ip_limit() {
    // Mutation kill (XFF-trust-by-default bypass): with trusted_proxy_hops=0 (the default), a single
    // peer rotating a FAKE X-Forwarded-For on every request must NOT get a fresh bucket each time — the
    // direct peer IP is used. So after the burst, the SAME peer is 429'd regardless of a new spoofed XFF.
    let app = app_with_rate_limit(0.0001, 1.0, 0, false); // capacity 1

    // First request from the peer (with a spoofed XFF) — within burst ⇒ reaches auth.
    let r1 = app
        .clone()
        .oneshot(req_from("203.0.113.50:8000", Some("1.2.3.4")))
        .await
        .unwrap();
    assert_eq!(r1.status(), StatusCode::UNAUTHORIZED, "first within burst");

    // Second request from the SAME peer but a DIFFERENT spoofed XFF — must still be 429 (the spoof does
    // not grant a new bucket because XFF is untrusted ⇒ the direct peer IP keys the bucket).
    let r2 = app
        .clone()
        .oneshot(req_from("203.0.113.50:8001", Some("5.6.7.8")))
        .await
        .unwrap();
    assert_eq!(
        r2.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "rotating a spoofed XFF must NOT dodge the per-IP limit when XFF is untrusted"
    );
}

#[tokio::test]
async fn trusted_xff_keys_the_bucket_by_client_ip() {
    // With trusted_proxy_hops=1 and ONE proxy receiving DIRECTLY from the client, the header is
    // `XFF: <client>` (the proxy is our TCP peer and writes nothing of itself), so the client is the
    // SINGLE rightmost entry (the corrected off-by-one: index N-1 = 0). Two distinct CLIENT IPs behind
    // the same proxy peer each get their OWN bucket — keyed by the XFF client IP, not the shared proxy
    // peer. capacity 1: each client's first request reaches auth (401), the SECOND is 429 (own bucket
    // exhausted), while the OTHER client is unaffected. This proves trusted XFF is honoured AND keyed by
    // the real client (the bug had collapsed all clients onto the shared proxy bucket).
    let app = app_with_rate_limit(0.0001, 1.0, 1, false);
    let proxy = "10.0.0.1:1000"; // the direct peer is our trusted reverse proxy

    // Client 198.51.100.5 — XFF "<client>" (one proxy ⇒ a single entry = the client).
    let c1a = app
        .clone()
        .oneshot(req_from(proxy, Some("198.51.100.5")))
        .await
        .unwrap();
    assert_eq!(
        c1a.status(),
        StatusCode::UNAUTHORIZED,
        "client1 first ⇒ auth"
    );

    // A DIFFERENT client behind the same proxy is unaffected (own bucket) ⇒ auth.
    let c2a = app
        .clone()
        .oneshot(req_from(proxy, Some("198.51.100.6")))
        .await
        .unwrap();
    assert_eq!(
        c2a.status(),
        StatusCode::UNAUTHORIZED,
        "client2 unaffected by client1 (per-client bucket via trusted XFF)"
    );

    // client1's SECOND request exhausts ITS bucket ⇒ 429.
    let c1b = app
        .clone()
        .oneshot(req_from(proxy, Some("198.51.100.5")))
        .await
        .unwrap();
    assert_eq!(
        c1b.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "client1's second request is 429 (its own per-client bucket is exhausted)"
    );
}
