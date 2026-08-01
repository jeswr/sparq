//! [SONNET-4.6] (sq-7d3dj.28) **Streaming admission + the per-stream pin cap** — Wave D child
//! D5 of `research/wave-d-pull-streaming-response-body.md` §7.
//!
//! A streamed SELECT-JSON body keeps its `spawn_blocking` worker — and therefore its
//! `Arc<Generation>` pin — alive for the whole client drain. Nothing else bounds that: the
//! generation ring's retention bound `K` bounds only the ring's OWN references (a reader's pin
//! keeps a generation resident after it ages out of retention), and the tower concurrency limit
//! releases its permit when the handler RETURNS the response, not when the body finishes
//! draining. So without this bead N stalled readers across N publishes hold N stale
//! generations, unbounded.
//!
//! Two mechanisms close that, and both are asserted here:
//!
//! 1. **Admission** — at/over `--stream-max-live-generations` live generations a new response
//!    does not stream at all: it GRACEFULLY DEGRADES to the buffered serialiser (identical
//!    bytes, `Content-Length`, never a pin held past serialize) with a `Warning` header. The
//!    design prefers this to a `503` shed: a correct answer beats a refusal when the correct
//!    answer is affordable — and the buffered answer is only AFFORDABLE when a memory ceiling
//!    (`--max-query-bytes` / `--max-query-rows` / `--max-results`) bounds it, since the buffered
//!    serialiser materialises the whole response. With no ceiling configured the stream is
//!    admitted under any pressure, and both halves of that are asserted here.
//! 2. **The per-stream pin cap** — a reader that stops draining past `--stream-pin-timeout` has
//!    its stream ABANDONED so the pin is released, and the body is truncated under the
//!    sq-7d3dj.26 contract: the document-closing `]}}` is never written and the reason is
//!    reported as `pin-deadline`. A clean short `200` remains forbidden.
//!
//! Each mechanism carries a CONTROL asserting the same request streams normally when the guard
//! is not engaged, so neither test could pass on a server that simply never streams. 🤖 SPARQ
//! agent.

#![cfg(feature = "server")]

use http_body::Body as _;
use sparq_core::Graph;
use sparq_server::{router, AppState, ServerConfig};
use std::time::Duration;
use tower::ServiceExt;

/// Enough triples that the serialised result runs to many 64 KiB chunks — a genuinely
/// MULTI-chunk result, which is the only shape that streams (a single-chunk result is returned
/// buffered with a `Content-Length` whatever the admission verdict says).
const TRIPLES: usize = 20_000;

const QUERY: &str = "SELECT * WHERE { ?s ?p ?o }";

/// A configured memory ceiling, far above anything [`TRIPLES`] can produce, so it never refuses
/// these queries. Its presence is what makes the buffered fallback a BOUNDED allocation, and
/// therefore what lets admission degrade a stream at all: without one the server would be
/// trading a bounded transport for an unbounded buffer precisely when it is under pressure.
const ROOMY_ROW_CEILING: usize = 1_000_000;

fn big_graph() -> Graph {
    let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
    for i in 0..TRIPLES {
        ttl.push_str(&format!(
            "ex:s{} ex:p \"value-{}-padding-padding-padding\" .\n",
            i, i
        ));
    }
    Graph::load_str(&ttl, "turtle").unwrap()
}

/// A drained response body.
struct Drained {
    bytes: Vec<u8>,
    trailers: Option<axum::http::HeaderMap>,
    /// The body failed a frame — over a real socket this aborts the chunked stream without its
    /// terminating zero-length chunk, so the client sees a transport error.
    failed: bool,
}

async fn drain(body: axum::body::Body) -> Drained {
    let mut body = Box::pin(body);
    let mut out = Drained { bytes: Vec::new(), trailers: None, failed: false };
    loop {
        match std::future::poll_fn(|cx| body.as_mut().poll_frame(cx)).await {
            None => return out,
            Some(Err(_)) => {
                out.failed = true;
                return out;
            }
            Some(Ok(frame)) => match frame.into_data() {
                Ok(data) => out.bytes.extend_from_slice(&data),
                Err(frame) => out.trailers = frame.into_trailers().ok(),
            },
        }
    }
}

fn select_request() -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .method(axum::http::Method::POST)
        .uri("/sparql")
        .header(axum::http::header::CONTENT_TYPE, "application/sparql-query")
        .header(
            axum::http::header::ACCEPT,
            "application/sparql-results+json",
        )
        .header(axum::http::header::TE, "trailers")
        .body(axum::body::Body::from(QUERY))
        .unwrap()
}

/// Raises the ring's `live_generations()` reading: each accepted UPDATE publishes a new
/// generation, and the ring retains the last `RingConfig::retain` (4) of them plus the current
/// one — so after a few updates the reading is comfortably above a small configured cap,
/// with no reader pin needed.
async fn publish_generations(app: &axum::Router, count: usize) {
    for i in 0..count {
        let request = axum::http::Request::builder()
            .method(axum::http::Method::POST)
            .uri("/sparql")
            .header(axum::http::header::CONTENT_TYPE, "application/sparql-update")
            .body(axum::body::Body::from(format!(
                "INSERT DATA {{ <http://ex/churn{}> <http://ex/p> \"v\" }}",
                i
            )))
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert!(
            response.status().is_success(),
            "update {} failed: {}",
            i,
            response.status()
        );
    }
}

/// THE ADMISSION TEST. Under forced ring pressure a multi-chunk SELECT-JSON that would
/// otherwise stream is served BUFFERED instead — so it holds no generation pin past serialize —
/// and says so in a `Warning` header. The answer itself is unchanged: complete, parseable, every
/// row present.
#[tokio::test]
async fn under_ring_pressure_a_stream_degrades_to_the_buffered_path() {
    const CAP: usize = 2;
    let app = router(AppState::with_config(
        big_graph(),
        ServerConfig {
            stream_max_live_generations: Some(CAP),
            max_query_rows: Some(ROOMY_ROW_CEILING),
            ..ServerConfig::default()
        },
    ));
    publish_generations(&app, 3).await;

    let response = app.clone().oneshot(select_request()).await.unwrap();
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, axum::http::StatusCode::OK);

    // Degraded: the buffered path knows the whole length up front, and never announces
    // trailers because there is no post-body verdict to report.
    assert!(
        parts.headers.get(axum::http::header::CONTENT_LENGTH).is_some(),
        "a declined stream must fall back to the buffered (Content-Length) path"
    );
    assert!(parts.headers.get(axum::http::header::TRAILER).is_none());

    // …and says why, naming the reading and the cap it hit.
    let warning = parts
        .headers
        .get(axum::http::header::WARNING)
        .expect("a degraded response must carry a Warning")
        .to_str()
        .unwrap()
        .to_owned();
    assert!(warning.starts_with("199 sparq "), "warning: {}", warning);
    assert!(
        warning.contains("cap of 2") && warning.contains("live generations"),
        "warning: {}",
        warning
    );
    // …and names the ceiling that made buffering the whole answer affordable, so an operator
    // can see WHICH of their limits the fallback is leaning on.
    assert!(
        warning.contains("bounded by --max-query-rows"),
        "warning: {}",
        warning
    );

    // The client still gets the COMPLETE result — degradation is a transport choice, never a
    // shed and never a short answer.
    let out = drain(body).await;
    assert!(!out.failed);
    assert!(out.bytes.ends_with(b"]}}"));
    let parsed: serde_json::Value = serde_json::from_slice(&out.bytes).expect("valid JSON");
    assert_eq!(
        parsed["results"]["bindings"].as_array().map(Vec::len),
        Some(TRIPLES + 3),
        "the buffered fallback must carry every row"
    );
}

/// The live-generation reading the server put in its `Warning` — the only externally visible
/// window onto the ring pressure admission is policing.
fn reported_live_generations(warning: &str) -> usize {
    let tail = warning.split("declined: ").nth(1).expect(warning);
    tail.split_whitespace()
        .next()
        .expect(warning)
        .parse()
        .expect(warning)
}

/// Degrading to "buffered" is only the safer choice if the buffered response does not itself
/// hold a generation until the client finishes reading — otherwise the cap bounds nothing and
/// slow readers accumulate pins exactly as they would have while streaming.
///
/// This asserts the property differentially, so it does not depend on the ring's retention
/// constants: take some degraded responses and DO NOT drain them, churn the ring past them, and
/// the pressure reading must be the same as if they had never been issued.
#[tokio::test]
async fn a_degraded_response_holds_no_pin_while_the_client_drains() {
    async fn reading_with_held_bodies(held: usize) -> usize {
        let app = router(AppState::with_config(
            // Small: this is about the PIN, not the body size, and every buffered SELECT-JSON
            // response goes through the same chunked path whatever its length.
            Graph::load_str("<http://ex/s> <http://ex/p> \"o\" .", "turtle").unwrap(),
            ServerConfig {
                stream_max_live_generations: Some(2),
                max_query_rows: Some(ROOMY_ROW_CEILING),
                ..ServerConfig::default()
            },
        ));
        publish_generations(&app, 3).await;

        // Undrained response bodies — a client that took the response and stopped reading.
        let mut bodies = Vec::new();
        for _ in 0..held {
            let response = app.clone().oneshot(select_request()).await.unwrap();
            assert!(
                response.headers().get(axum::http::header::WARNING).is_some(),
                "this test only means anything while the responses are being degraded"
            );
            bodies.push(response.into_body());
        }

        // Churn past whatever generation those held responses were evaluated against, so a
        // leaked pin would keep an aged-out generation alive and show up in the reading.
        publish_generations(&app, 6).await;

        let response = app.clone().oneshot(select_request()).await.unwrap();
        let warning = response
            .headers()
            .get(axum::http::header::WARNING)
            .expect("still under pressure")
            .to_str()
            .unwrap()
            .to_owned();
        drop(bodies);
        reported_live_generations(&warning)
    }

    let baseline = reading_with_held_bodies(0).await;
    let with_held = reading_with_held_bodies(3).await;
    assert_eq!(
        with_held, baseline,
        "each undrained degraded response leaked a generation pin ({} vs {})",
        with_held, baseline
    );
}

/// CONTROL (anti-vacuity) for the admission test: the SAME pressure with admission DISABLED
/// streams, so the assertions above are testing the cap and not just "this query never
/// streams".
#[tokio::test]
async fn with_admission_disabled_the_same_pressure_still_streams() {
    let app = router(AppState::with_config(
        big_graph(),
        ServerConfig {
            stream_max_live_generations: None,
            // Everything else matches the degradation test, so the cap is the only difference.
            max_query_rows: Some(ROOMY_ROW_CEILING),
            ..ServerConfig::default()
        },
    ));
    publish_generations(&app, 3).await;

    let response = app.clone().oneshot(select_request()).await.unwrap();
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, axum::http::StatusCode::OK);
    assert!(
        parts.headers.get(axum::http::header::CONTENT_LENGTH).is_none(),
        "an admitted stream declares no Content-Length"
    );
    assert!(parts.headers.get(axum::http::header::WARNING).is_none());
    let out = drain(body).await;
    assert!(out.bytes.ends_with(b"]}}"));
    assert_eq!(
        out.trailers
            .as_ref()
            .and_then(|t| t.get("x-sparq-complete"))
            .map(|v| v.to_str().unwrap()),
        Some("true")
    );
}

/// CONTROL: pressure BELOW the cap admits the stream. Together with the test above this pins
/// the comparison itself — a server that degraded unconditionally would fail here.
#[tokio::test]
async fn below_the_cap_a_stream_is_admitted() {
    let app = router(AppState::with_config(
        big_graph(),
        ServerConfig {
            // The ring holds the current generation plus up to 4 retained, so 16 is far above
            // anything three updates can produce.
            stream_max_live_generations: Some(16),
            // Set, so this control isolates the CAP comparison — a stream admitted here is
            // admitted because the pressure is below 16, not because degrading was unaffordable.
            max_query_rows: Some(ROOMY_ROW_CEILING),
            ..ServerConfig::default()
        },
    ));
    publish_generations(&app, 3).await;

    let response = app.clone().oneshot(select_request()).await.unwrap();
    let (parts, _body) = response.into_parts();
    assert_eq!(parts.status, axum::http::StatusCode::OK);
    assert!(
        parts.headers.get(axum::http::header::CONTENT_LENGTH).is_none(),
        "pressure below the cap must not degrade the response"
    );
    assert!(parts.headers.get(axum::http::header::WARNING).is_none());
}

/// THE AFFORDABILITY TEST. Degrading swaps a bounded transport (four channel slots, whatever the
/// result size) for an allocation the size of the whole serialised answer. On a server with NO
/// memory ceiling configured — which is `ServerConfig::default()` — nothing bounds that answer, so
/// the same over-cap pressure must NOT degrade: a client could otherwise pair a large SELECT with
/// ring pressure to force the server to materialise an unbounded response exactly when it is
/// already under load. Streaming is the memory-safer of the two, and the pin it holds is bounded
/// by `--stream-pin-timeout` instead.
#[tokio::test]
async fn over_the_cap_with_no_memory_ceiling_the_stream_is_still_admitted() {
    let app = router(AppState::with_config(
        big_graph(),
        ServerConfig {
            // The SAME cap the degradation test uses, under the SAME pressure — the only
            // difference is that no ceiling bounds the buffered answer.
            stream_max_live_generations: Some(2),
            max_query_rows: None,
            max_query_bytes: None,
            max_results: None,
            ..ServerConfig::default()
        },
    ));
    publish_generations(&app, 3).await;

    let response = app.clone().oneshot(select_request()).await.unwrap();
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, axum::http::StatusCode::OK);
    assert!(
        parts.headers.get(axum::http::header::CONTENT_LENGTH).is_none(),
        "FORBIDDEN: an unbounded result must not be buffered whole just because the ring is busy"
    );
    assert!(
        parts.headers.get(axum::http::header::WARNING).is_none(),
        "nothing was degraded, so nothing should claim it was"
    );
    let out = drain(body).await;
    assert!(out.bytes.ends_with(b"]}}"));
}

/// …and the corollary: when a ceiling IS configured, it really does bound the degraded response.
/// Over-cap pressure plus a result that EXCEEDS the ceiling is an honest pre-first-byte `413`,
/// not a materialised body — the buffered fallback is bounded by refusal, which is the property
/// that makes degrading affordable in the first place.
#[tokio::test]
async fn a_degraded_response_that_exceeds_the_ceiling_is_refused_not_materialised() {
    let app = router(AppState::with_config(
        big_graph(),
        ServerConfig {
            stream_max_live_generations: Some(2),
            // Far below TRIPLES: this result cannot fit under the ceiling.
            max_query_rows: Some(100),
            ..ServerConfig::default()
        },
    ));
    publish_generations(&app, 3).await;

    let response = app.clone().oneshot(select_request()).await.unwrap();
    let (parts, body) = response.into_parts();
    assert_eq!(
        parts.status,
        axum::http::StatusCode::PAYLOAD_TOO_LARGE,
        "a degraded response over the ceiling must be refused before the first byte"
    );
    let out = drain(body).await;
    assert!(
        serde_json::from_slice::<serde_json::Value>(&out.bytes)
            .ok()
            .and_then(|v| v["results"]["bindings"].as_array().map(Vec::len))
            .is_none(),
        "a refusal must not carry a result set"
    );
}

/// THE PIN-CAP TEST. A client that takes the response and then STOPS READING is abandoned once
/// the pin deadline elapses — the worker stops, drops its generation pin, and the body is
/// truncated per D3: the document is never closed and the reason is `pin-deadline`. The
/// forbidden outcome (a clean, parseable, short `200`) is asserted against directly.
#[tokio::test]
async fn a_reader_that_stalls_past_the_pin_cap_is_truncated() {
    let app = router(AppState::with_config(
        big_graph(),
        ServerConfig {
            stream_pin_timeout: Some(Duration::from_millis(100)),
            ..ServerConfig::default()
        },
    ));

    let response = app.oneshot(select_request()).await.unwrap();
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, axum::http::StatusCode::OK);
    assert!(
        parts.headers.get(axum::http::header::CONTENT_LENGTH).is_none(),
        "this must be the streaming shape for the stall to mean anything"
    );

    // The stalled reader: hold the body without polling it well past the deadline. The worker
    // fills the bounded channel, then gives up rather than pinning its generation indefinitely.
    tokio::time::sleep(Duration::from_millis(600)).await;

    let out = drain(body).await;
    assert!(
        !out.bytes.is_empty(),
        "the chunks buffered before the stall are still delivered"
    );
    assert!(
        !out.bytes.ends_with(b"]}}"),
        "FORBIDDEN: an abandoned stream must not close the JSON document"
    );
    serde_json::from_slice::<serde_json::Value>(&out.bytes)
        .expect_err("FORBIDDEN: an abandoned stream must not parse as a complete result");
    let trailers = out.trailers.expect("the truncation reason must be reported");
    assert_eq!(
        trailers.get("x-sparq-truncated").map(|v| v.to_str().unwrap()),
        Some("pin-deadline")
    );
    assert!(
        trailers.get("x-sparq-complete").is_none(),
        "an abandoned stream must never claim completeness"
    );
}

/// CONTROL (anti-vacuity) for the pin cap: the SAME stall with the cap DISABLED completes
/// normally — the worker waits for the reader however long it takes. Without this the test
/// above would pass on a server that truncated every slow stream regardless of the knob.
#[tokio::test]
async fn with_the_pin_cap_disabled_the_same_stall_completes() {
    let app = router(AppState::with_config(
        big_graph(),
        ServerConfig { stream_pin_timeout: None, ..ServerConfig::default() },
    ));

    let response = app.oneshot(select_request()).await.unwrap();
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, axum::http::StatusCode::OK);
    assert!(parts.headers.get(axum::http::header::CONTENT_LENGTH).is_none());

    tokio::time::sleep(Duration::from_millis(600)).await;

    let out = drain(body).await;
    assert!(!out.failed);
    assert!(
        out.bytes.ends_with(b"]}}"),
        "an unbounded stream must still complete for a client that resumes reading"
    );
    let parsed: serde_json::Value = serde_json::from_slice(&out.bytes).expect("valid JSON");
    assert_eq!(
        parsed["results"]["bindings"].as_array().map(Vec::len),
        Some(TRIPLES)
    );
}
