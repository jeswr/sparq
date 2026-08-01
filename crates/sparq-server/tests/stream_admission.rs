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
//!    answer is affordable — and affordability is ENFORCED by the degraded path's own byte
//!    budget (`DEGRADED_BUFFER_MAX_BYTES`), not inferred from a configured ceiling. No engine
//!    ceiling bounds serialised response bytes: `--max-query-rows` / `--max-results` bound ROWS,
//!    and `--max-query-bytes` prices the id-level working set plus query-COMPUTED terms, so a
//!    single row projecting a huge literal already stored in the graph is cheap under all three
//!    and enormous once serialised. That exact shape is asserted here to be refused rather than
//!    materialised.
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

/// A configured memory ceiling far above anything [`TRIPLES`] can produce, so the engine never
/// refuses these queries and the admission verdict is the only thing under test. It is NOT what
/// bounds the buffered fallback — see `over_cap_a_few_huge_stored_literals_are_refused_not_buffered`.
const ROOMY_ROW_CEILING: usize = 1_000_000;

/// The degraded path's own allocation budget, mirroring `http::DEGRADED_BUFFER_MAX_BYTES` (a
/// private constant — this integration test sees only the behaviour and the `Warning` text, so
/// it re-states the value and pins it against the header the server emits).
const DEGRADED_BUFFER_MAX_BYTES: usize = 8 * 1024 * 1024;

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

/// Bytes in each pre-existing stored literal of [`huge_literal_graph`], and how many rows carry
/// one. The product (12 MiB) is comfortably over [`DEGRADED_BUFFER_MAX_BYTES`] while the ROW
/// count and the id-level working set are trivial — the asymmetry the affordability test needs.
const HUGE_LITERAL_BYTES: usize = 3 * 1024 * 1024;
const HUGE_LITERAL_ROWS: usize = 4;

/// A graph whose whole size lives in a handful of literals that are ALREADY INTERNED before the
/// query runs. `SELECT * WHERE { ?s ?p ?o }` over it binds [`HUGE_LITERAL_ROWS`] rows of three
/// ids — nothing a row cap or a working-set byte cap can object to — and serialises to over
/// 12 MiB, because the serialiser must write out the dictionary-resident lexical forms those ids
/// point at. No `x` needs JSON escaping, so the serialised size is the literal size.
fn huge_literal_graph() -> Graph {
    let padding = "x".repeat(HUGE_LITERAL_BYTES);
    let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
    for i in 0..HUGE_LITERAL_ROWS {
        ttl.push_str(&format!("ex:s{} ex:p \"{}-{}\" .\n", i, padding, i));
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
    // …and names the byte budget the degraded body is held to, so an operator reading the
    // header knows what bounds the allocation the server just chose to make.
    assert!(
        warning.contains(&format!("capped at {} bytes", DEGRADED_BUFFER_MAX_BYTES)),
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
/// result size) for an allocation the size of the whole serialised answer — and NO configured
/// engine ceiling bounds that number of BYTES. This is the shape that proves it: FOUR rows, each
/// projecting a multi-megabyte literal that was already in the graph before the query ran. Every
/// engine ceiling is set and comfortably satisfied — the row caps see four rows, and
/// `--max-query-bytes` prices the id-level working set (a handful of `Id`s) plus the terms the
/// query COMPUTES, of which there are none — yet the serialised answer is
/// [`HUGE_LITERAL_ROWS`] × [`HUGE_LITERAL_BYTES`], well over the degraded path's budget.
///
/// So an attacker who pairs ring pressure with a low-cardinality, high-byte result must get a
/// BOUNDED REFUSAL, not a materialised body. Transient (`503`), because the same query streams
/// fine once pressure drops — the refusal is about capacity, not about the request.
#[tokio::test]
async fn over_cap_a_few_huge_stored_literals_are_refused_not_buffered() {
    let app = router(AppState::with_config(
        huge_literal_graph(),
        ServerConfig {
            stream_max_live_generations: Some(2),
            // Every ceiling configured, and every one of them SATISFIED by this result — which
            // is exactly the point: a row cap is not a response-byte bound, and the byte cap
            // prices the working set, not the stored literals the working set points AT.
            max_query_rows: Some(ROOMY_ROW_CEILING),
            max_results: Some(ROOMY_ROW_CEILING),
            max_query_bytes: Some(1024 * 1024),
            ..ServerConfig::default()
        },
    ));
    publish_generations(&app, 3).await;

    let response = app.clone().oneshot(select_request()).await.unwrap();
    let (parts, body) = response.into_parts();
    assert_eq!(
        parts.status,
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        "FORBIDDEN: a few huge stored literals must not be materialised whole just because \
         every ROW cap was satisfied"
    );
    let out = drain(body).await;
    assert!(
        out.bytes.len() < HUGE_LITERAL_BYTES,
        "a refusal must be a short error body, not the answer: {} bytes",
        out.bytes.len()
    );
    assert!(
        serde_json::from_slice::<serde_json::Value>(&out.bytes)
            .ok()
            .and_then(|v| v["results"]["bindings"].as_array().map(Vec::len))
            .is_none(),
        "a refusal must not carry a result set"
    );
}

/// CONTROL (anti-vacuity) for the test above: the SAME graph and the SAME ceilings BELOW the
/// pressure cap are admitted and stream the whole answer. Without this, "refused" could just
/// mean "this server cannot serve big literals at all" — the refusal has to be the degradation
/// budget, not the query.
#[tokio::test]
async fn below_the_cap_the_same_huge_literals_stream_in_full() {
    let app = router(AppState::with_config(
        huge_literal_graph(),
        ServerConfig {
            stream_max_live_generations: Some(64),
            max_query_rows: Some(ROOMY_ROW_CEILING),
            max_results: Some(ROOMY_ROW_CEILING),
            max_query_bytes: Some(1024 * 1024),
            ..ServerConfig::default()
        },
    ));
    publish_generations(&app, 3).await;

    let response = app.clone().oneshot(select_request()).await.unwrap();
    let (parts, body) = response.into_parts();
    assert_eq!(parts.status, axum::http::StatusCode::OK);
    assert!(parts.headers.get(axum::http::header::WARNING).is_none());
    let out = drain(body).await;
    assert!(!out.failed);
    assert!(
        out.bytes.len() > DEGRADED_BUFFER_MAX_BYTES,
        "the control must exceed the degraded budget, or it proves nothing: {} bytes",
        out.bytes.len()
    );
    assert!(out.bytes.ends_with(b"]}}"));
}

/// …and the engine ceilings still apply ON TOP of the degradation budget: over-cap pressure plus
/// a result that exceeds a configured ceiling is the usual honest pre-first-byte `413`, not a
/// materialised body. (A ceiling bounds the engine's working set, which is a real and separate
/// guard — it is only the claim that it bounds RESPONSE BYTES that would be wrong.)
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
