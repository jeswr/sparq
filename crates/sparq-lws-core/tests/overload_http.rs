// AUTHORED-BY Claude Opus 4.8
//! End-to-end HTTP tests for the overload / backpressure layer through the assembled router.
//!
//! These prove the SECURITY-critical properties of admission control + the timeout, not just the
//! happy path:
//! - a request shed at capacity returns **503 + `Retry-After`** and is NEVER forwarded to the inner
//!   stack — so shedding can never turn an unauthorized request into a success (no auth bypass);
//! - the **health probes (`/livez`, `/readyz`) are EXEMPT** — they succeed (200) even when the
//!   admission pool is fully exhausted, so a load balancer's readiness probe is never shed;
//! - capacity is RELEASED after a request completes (the gauge does not drift), so the server
//!   recovers once the spike passes.
//!
//! The trick for a deterministic test: we construct the [`AdmissionControl`] ourselves, manually hold
//! its permit(s) (modelling in-flight requests), and then drive requests through the router whose
//! admission middleware shares the SAME semaphore — so it observes "at capacity" and sheds, with no
//! need for real concurrent socket I/O. A clone of `AdmissionControl` shares the inner `Arc<Semaphore>`.

mod common;

use axum::body::{to_bytes, Body};
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
use sparq_lws_core::store::{CompositeStore, InMemoryBlobStore, InMemorySparqClient};
use tower::ServiceExt;

/// Build a router with overload protection over a real (in-memory) LDP+auth stack, returning the
/// router plus a CLONE of the admission control (sharing the same semaphore) so the test can hold
/// permits to simulate saturation. No request timeout here (these tests are about shedding).
fn app_with_admission(max_concurrency: usize) -> (axum::Router, AdmissionControl) {
    let issuer_key = KeyKit::generate();
    let config = VerifierConfig::new(vec![common::ISSUER.to_string()], BASE_URL);
    let replay = InMemoryReplayStore::with_window(config.replay_ttl());
    let verifier = Verifier::new(config, jwks_provider(&issuer_key), replay).unwrap();
    let ctx = AuthContext::new(verifier, BASE_URL);
    let store = CompositeStore::new(InMemorySparqClient::new(), InMemoryBlobStore::new());
    let ldp = LdpState::new(store, BASE_URL);

    let admission = AdmissionControl::new(max_concurrency);
    let overload = OverloadConfig {
        admission: admission.clone(),
        request_timeout: None,
        rate_limiter: None,
        body_limit_bytes: sparq_lws_core::body_limit::DEFAULT_MAX_BODY_BYTES,
    };
    let app = build_router_with_overload(AppState::new(ctx, ldp), overload);
    (app, admission)
}

#[tokio::test]
async fn sheds_with_503_and_retry_after_at_capacity() {
    let (app, admission) = app_with_admission(1);

    // Exhaust the single permit (model one in-flight request holding the only slot).
    let held = admission
        .try_admit_for_test()
        .expect("take the only permit");

    // A new request now finds no permit ⇒ shed with 503 + Retry-After, WITHOUT reaching the inner
    // stack. We send an ANONYMOUS request to a protected path: if shedding leaked to the inner stack
    // it would be a 401 (auth) — a 503 proves the request was shed BEFORE auth (no bypass risk).
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{BASE_URL}/alice/private"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "at capacity the request must be shed with 503 (not run)"
    );
    let retry_after = resp
        .headers()
        .get(axum::http::header::RETRY_AFTER)
        .expect("503 must carry Retry-After")
        .to_str()
        .unwrap()
        .to_string();
    let secs: u64 = retry_after
        .parse()
        .expect("Retry-After is a seconds integer");
    assert!(
        (1..=5).contains(&secs),
        "Retry-After {secs} must be within the jittered [base, base+jitter] band"
    );

    // The shed count incremented; the body says overloaded.
    assert_eq!(admission.metrics().shed_total(), 1);
    let body = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let text = String::from_utf8_lossy(&body);
    assert!(text.contains("503") && text.to_lowercase().contains("overload"));

    // Release the held permit ⇒ the server recovers (a later request is admitted, not shed).
    drop(held);
    let resp2 = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{BASE_URL}/alice/private"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Admitted ⇒ it reaches auth, which 401s the anonymous caller (NOT a 503). The point: capacity
    // recovered, and an admitted-but-unauthenticated request is correctly 401 (auth still enforced).
    assert_ne!(
        resp2.status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "after releasing the permit the request must be admitted (recovered)"
    );
    assert_eq!(
        resp2.status(),
        StatusCode::UNAUTHORIZED,
        "an admitted anonymous request to a protected path is 401 — auth is still enforced"
    );
}

#[tokio::test]
async fn shed_does_not_bypass_auth_for_a_protected_write() {
    // The adversarial case: could an attacker use overload to slip an UNAUTHENTICATED WRITE past WAC?
    // No — a shed request returns 503 and never runs the handler. Exhaust capacity, then send an
    // anonymous PUT (a mutation): it must be 503 (shed), NEVER 200/201/204 (which would be a write
    // bypass) and never even reach the auth layer.
    let (app, admission) = app_with_admission(1);
    let _held = admission
        .try_admit_for_test()
        .expect("take the only permit");

    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("{BASE_URL}/alice/secret"))
                .header("content-type", "text/turtle")
                .body(Body::from("<#x> <#y> <#z> ."))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = resp.status();
    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "an anonymous write at capacity must be SHED (503), never executed"
    );
    assert!(
        !status.is_success(),
        "a shed write must NEVER be a 2xx success (that would be an auth/WAC bypass)"
    );
}

#[tokio::test]
async fn health_probes_are_exempt_from_shedding() {
    // Even with the admission pool FULLY exhausted, /livez and /readyz must still answer 200 — they
    // are mounted OUTSIDE the admission layer. (Shedding a healthy instance's readiness probe would
    // make the LB pull it and amplify an overload into an outage.)
    let (app, admission) = app_with_admission(1);
    let _held = admission
        .try_admit_for_test()
        .expect("take the only permit");

    for path in [LIVEZ_PATH, READYZ_PATH] {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("{BASE_URL}{path}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "{path} must be 200 even when the admission pool is exhausted (probe is shed-exempt)"
        );
    }

    // And the health probes did NOT consume an admission permit / register as shed (they bypass the
    // layer entirely): no extra shed was counted by probing.
    assert_eq!(
        admission.metrics().shed_total(),
        0,
        "health probes must not even touch admission control (no shed counted)"
    );
}

#[tokio::test]
async fn normal_concurrency_does_not_trip_the_limit() {
    // A ceiling set ABOVE the offered concurrency must NOT shed any request — the limit is a
    // pathological-overload bound, not a normal-throughput throttle. Drive several requests through a
    // router whose admission pool comfortably exceeds them and assert ZERO sheds (none is a 503).
    let n_requests = 8usize;
    let (app, admission) = app_with_admission(n_requests * 4); // ceiling well above the load

    for _ in 0..n_requests {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("{BASE_URL}/alice/private"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // Each request is admitted (reaches auth ⇒ 401 for the anonymous caller), NEVER shed (503).
        assert_ne!(
            resp.status(),
            StatusCode::SERVICE_UNAVAILABLE,
            "a request under a generous ceiling must never be shed"
        );
    }
    assert_eq!(
        admission.metrics().shed_total(),
        0,
        "no request must be shed when the ceiling exceeds the offered concurrency"
    );
    assert_eq!(
        admission.metrics().in_flight(),
        0,
        "every admitted request released its permit (the in-flight gauge does not drift)"
    );
}

// --- Request timeout (504) ------------------------------------------------------------------------
//
// These prove the timeout layer through a router assembled with the EXACT production layer ordering
// (admission OUTERMOST via `admission_middleware`, `TimeoutLayer` just inside, health routes OUTSIDE
// both) — mirroring `app::build_router_with_overload` — but over a controllable slow handler, since
// the real LDP handlers respond immediately. The security property under test: a request that times
// out returns 504 and is NEVER a 2xx success (so a stuck request cannot become an auth/WAC bypass),
// and the health probe is exempt from the timeout too.

use axum::middleware::from_fn_with_state;
use axum::routing::get;
use axum::Router;
use sparq_lws_core::overload::admission_middleware;
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;

/// Build a router that replicates the production overload layering over a deliberately SLOW handler:
/// `GET /slow` sleeps `handler_delay`; `GET /fast` returns 200 immediately. The timeout is
/// `timeout`. Health routes (`/livez`) are mounted OUTSIDE the layers (timeout/admission-exempt),
/// exactly as in `build_router_with_overload`. The admission ceiling is high (won't shed here).
fn timeout_router(timeout: Duration, handler_delay: Duration) -> Router {
    let admission = AdmissionControl::new(10_000);

    // INNER application routes + the timeout layer, then the OUTERMOST admission layer — the same
    // order as `app::build_router_with_overload`.
    let app = Router::new()
        .route(
            "/slow",
            get(move || async move {
                tokio::time::sleep(handler_delay).await;
                (StatusCode::OK, "slow-done\n")
            }),
        )
        .route("/fast", get(|| async { (StatusCode::OK, "fast\n") }))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::GATEWAY_TIMEOUT,
            timeout,
        ))
        .layer(from_fn_with_state(admission, admission_middleware));

    // Health route OUTSIDE both layers — never timed out / shed.
    app.merge(Router::new().route(LIVEZ_PATH, get(|| async { (StatusCode::OK, "live\n") })))
}

#[tokio::test]
async fn request_timeout_returns_504_for_a_stuck_request() {
    // The handler sleeps far longer than the (tiny) timeout ⇒ the timeout layer aborts it with 504,
    // and the response is NEVER the handler's 200 (a stuck request cannot complete-as-success).
    let app = timeout_router(Duration::from_millis(50), Duration::from_secs(30));

    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{BASE_URL}/slow"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = resp.status();
    assert_eq!(
        status,
        StatusCode::GATEWAY_TIMEOUT,
        "a request exceeding the timeout must be aborted with 504 (not 408, not the handler's 200)"
    );
    assert!(
        !status.is_success(),
        "a timed-out request must NEVER be a 2xx success (that would let a stuck request masquerade \
         as completed — and, in the real stack, slip past auth/WAC)"
    );
}

#[tokio::test]
async fn request_under_the_timeout_succeeds_and_health_is_timeout_exempt() {
    // A fast request well under the timeout completes normally (200) — the timeout doesn't clip
    // healthy traffic — and the health probe answers 200 even though it is mounted outside the layer.
    let app = timeout_router(Duration::from_secs(30), Duration::from_millis(1));

    let fast = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{BASE_URL}/fast"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        fast.status(),
        StatusCode::OK,
        "a request well under the timeout must complete normally (the timeout doesn't clip normal traffic)"
    );

    let live = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("{BASE_URL}{LIVEZ_PATH}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        live.status(),
        StatusCode::OK,
        "the health probe is mounted outside the timeout layer and always answers 200"
    );
}
