//! Integration tests for the T15 hardening guards: per-request timeout, body-size limit,
//! concurrency shedding, panic recovery and the SELECT row cap. Each test boots the real
//! hardened router on an ephemeral port and drives it over HTTP, asserting both the HTTP
//! status semantics and the structured JSON error bodies.

use std::time::Duration;

use sparq_core::Graph;
use sparq_server::{harden, router, AppState, ServerConfig};
use tokio::net::TcpListener;

const DATA: &str = r#"
    @prefix ex: <http://ex/> .
    ex:alice ex:knows ex:bob ; ex:age 30 ; ex:name "Alice" .
    ex:bob   ex:age 25 ; ex:name "Bob"@en .
    ex:carol ex:age 35 .
"#;

/// A dense graph whose 3-way disconnected pattern forces a cross-product far too large
/// to ever finish — the deliberately slow query for the timeout / shedding tests.
fn dense_graph_ttl() -> String {
    let n = 80; // 80×80 = 6 400 edges; (6 400)^3 cross-product rows ≈ 2.6e11 — never finishes
    let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
    for i in 0..n {
        for j in 0..n {
            ttl.push_str(&format!("ex:n{i} ex:e ex:n{j} .\n"));
        }
    }
    ttl
}

const SLOW_QUERY: &str =
    "PREFIX ex: <http://ex/> SELECT * WHERE { ?a ex:e ?b . ?c ex:e ?d . ?e ex:e ?f }";

async fn serve(app: axum::Router) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

async fn spawn_with(ttl: &str, config: ServerConfig) -> String {
    let graph = Graph::load_str(ttl, "turtle").unwrap();
    serve(router(AppState::with_config(graph, config))).await
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

// ---------------------------------------------------------------------------
// (1) Request timeout — slow query → 503, worker stopped cooperatively
// ---------------------------------------------------------------------------

#[tokio::test]
async fn timeout_fires_on_slow_query() {
    let config = ServerConfig { query_timeout: Some(Duration::from_secs(1)), ..ServerConfig::default() };
    let base = spawn_with(&dense_graph_ttl(), config).await;

    let started = std::time::Instant::now();
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", SLOW_QUERY)])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 503);
    // 1s budget deadline + 2s hard grace; anything beyond means the guard didn't fire.
    assert!(started.elapsed() < Duration::from_secs(5), "503 took {:?}", started.elapsed());
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("{\"error\":"), "structured JSON error, got: {body}");
    assert!(body.contains("timed out"), "got: {body}");
}

#[tokio::test]
async fn fast_query_unaffected_by_timeout() {
    let config = ServerConfig { query_timeout: Some(Duration::from_secs(1)), ..ServerConfig::default() };
    let base = spawn_with(DATA, config).await;
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT ?s WHERE { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.text().await.unwrap().contains("\"bindings\""));
}

// ---------------------------------------------------------------------------
// (2a) Body-size limit → 413
// ---------------------------------------------------------------------------

#[tokio::test]
async fn oversized_body_is_413() {
    let config = ServerConfig { max_body_bytes: 256, ..ServerConfig::default() };
    let base = spawn_with(DATA, config).await;
    // A syntactically valid (huge) query body over the limit — rejected before parsing.
    let big = format!("SELECT * WHERE {{ ?s ?p ?o }} # {}", "x".repeat(1024));
    let resp = client()
        .post(format!("{base}/sparql"))
        .header("content-type", "application/sparql-query")
        .body(big)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 413);
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("{\"error\":"), "structured JSON error, got: {body}");

    // Under the limit still works.
    let resp = client()
        .post(format!("{base}/sparql"))
        .header("content-type", "application/sparql-query")
        .body("SELECT * WHERE { ?s ?p ?o }")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

// ---------------------------------------------------------------------------
// (2b) Concurrency limit + load shed → 429
// ---------------------------------------------------------------------------

#[tokio::test]
async fn concurrency_limit_sheds_with_429() {
    let config = ServerConfig {
        max_concurrent: 1,
        query_timeout: Some(Duration::from_secs(3)),
        ..ServerConfig::default()
    };
    let base = spawn_with(&dense_graph_ttl(), config).await;

    // Occupy the single slot with the slow query…
    let base2 = base.clone();
    let slow = tokio::spawn(async move {
        client().get(format!("{base2}/sparql")).query(&[("query", SLOW_QUERY)]).send().await
    });
    tokio::time::sleep(Duration::from_millis(300)).await;

    // …then any further request is shed immediately.
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT * WHERE { ?s ?p ?o } LIMIT 1")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 429);
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("{\"error\":"), "structured JSON error, got: {body}");

    // The occupying request still terminates via the timeout guard (503).
    let slow_resp = slow.await.unwrap().unwrap();
    assert_eq!(slow_resp.status(), 503);
}

// ---------------------------------------------------------------------------
// (2c/3) --max-results row cap → honest 413 refusal, never truncation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn max_results_refuses_with_413() {
    let config = ServerConfig { max_results: Some(2), ..ServerConfig::default() };
    let base = spawn_with(DATA, config).await;

    // 3 matching rows > cap of 2 → refused, with the limit named in the error.
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT ?s WHERE { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 413);
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("{\"error\":"), "structured JSON error, got: {body}");
    assert!(body.contains("max-results"), "got: {body}");
    assert!(!body.contains("bindings"), "must refuse, not truncate: {body}");

    // Within the cap → normal result (and LIMIT keeps queries under the cap usable).
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT ?s WHERE { ?s <http://ex/age> ?a } LIMIT 2")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap().matches("\"type\":\"uri\"").count(), 2);

    // The cap applies to every SELECT serialisation, not just JSON.
    let resp = client()
        .get(format!("{base}/sparql"))
        .header("accept", "text/csv")
        .query(&[("query", "SELECT ?s WHERE { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 413);

    // ASK is existence-only and ignores the row cap.
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "ASK { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.text().await.unwrap().contains("true"));
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] (sq-ebii) Memory cap (--max-query-rows): coarse working-set row ceiling
// applied on EVERY query form (not just the SELECT projection), → honest 413.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn max_query_rows_caps_every_form() {
    // 3 ex:age triples. A cap of 2 must trip on the SELECT working set AND — unlike
    // --max-results, which only caps the final SELECT projection — on a CONSTRUCT, whose
    // WHERE pattern materialises the same 3 rows.
    let config = ServerConfig { max_query_rows: Some(2), ..ServerConfig::default() };
    let base = spawn_with(DATA, config).await;

    // SELECT over 3 rows > cap → 413, naming the memory cap.
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT ?s WHERE { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 413);
    let body = resp.text().await.unwrap();
    assert!(body.contains("max-query-rows"), "should name the memory cap: {body}");

    // CONSTRUCT materialises the same 3 WHERE rows → also 413 (max-results would NOT cap this).
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "CONSTRUCT { ?s <http://ex/a> ?a } WHERE { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 413, "CONSTRUCT working set must be bounded by --max-query-rows");

    // Within the cap → normal 200.
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT ?s WHERE { ?s <http://ex/age> ?a } LIMIT 2")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] (sq-ebii) UPDATE timeout: the per-request query timeout now applies to the
// `application/sparql-update` path too — a never-finishing DELETE/INSERT WHERE → 503,
// bounded by `timeout + TIMEOUT_GRACE` (the same hard wall-clock cap the read paths use).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn update_path_honours_timeout() {
    let config = ServerConfig { query_timeout: Some(Duration::from_secs(1)), ..ServerConfig::default() };
    let base = spawn_with(&dense_graph_ttl(), config).await;

    // A DELETE/INSERT … WHERE whose WHERE is the same never-finishing 3-way cross-product.
    let update = "PREFIX ex: <http://ex/> \
         DELETE { ?a ex:e ?b } INSERT { ?a ex:gone ?b } \
         WHERE { ?a ex:e ?b . ?c ex:e ?d . ?e ex:e ?f }";
    let started = std::time::Instant::now();
    let resp = client()
        .post(format!("{base}/sparql"))
        .header("content-type", "application/sparql-update")
        .body(update)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 503, "slow UPDATE must time out, not hang");
    assert!(started.elapsed() < Duration::from_secs(8), "503 took {:?}", started.elapsed());
    let body = resp.text().await.unwrap();
    assert!(body.contains("timed out"), "got: {body}");

    // A fast update on the same server still succeeds (204).
    let resp = client()
        .post(format!("{base}/sparql"))
        .header("content-type", "application/sparql-update")
        .body("INSERT DATA { <http://ex/x> <http://ex/p> <http://ex/y> }")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] (sq-ebii) Decompression-ratio cap (--max-decompress-ratio): a high-ratio
// gzip body (a "zip bomb") on the GSP write path is refused with 413 BEFORE the full
// decompressed image is held; a benign small gzip body within the ratio decodes fine.
// ---------------------------------------------------------------------------

fn gzip(data: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    e.write_all(data).unwrap();
    e.finish().unwrap()
}

#[tokio::test]
async fn decompress_ratio_cap_rejects_zip_bomb() {
    // Small ratio cap (3×) + a generous body limit so the COMPRESSED bytes pass the
    // body-size gate and the ratio cap is what actually trips.
    let config = ServerConfig { max_decompress_ratio: 3, max_body_bytes: 1 << 20, ..ServerConfig::default() };
    let base = spawn_with(DATA, config).await;

    // A highly compressible Turtle body: ~200 KiB of valid triples compresses to a few KiB,
    // i.e. a ratio well above 3× — the decompressed image would exceed 3 × compressed.
    let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
    for i in 0..4000 {
        ttl.push_str(&format!("ex:s{i} ex:p ex:o .\n"));
    }
    let compressed = gzip(ttl.as_bytes());
    assert!(
        ttl.len() > compressed.len() * 3,
        "test fixture must exceed the 3x ratio (plain {}, gz {})",
        ttl.len(),
        compressed.len()
    );

    let resp = client()
        .put(format!("{base}/graphs/bomb"))
        .header("content-type", "text/turtle")
        .header("content-encoding", "gzip")
        .body(compressed)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 413, "high-ratio gzip body must be refused as a possible zip bomb");
    let body = resp.text().await.unwrap();
    assert!(body.contains("decompression-ratio") || body.contains("zip bomb"), "got: {body}");

    // A benign small gzip body whose ratio is under the cap decodes and writes fine.
    let small = "@prefix ex: <http://ex/> . ex:a ex:p ex:b .";
    let small_gz = gzip(small.as_bytes());
    // (a 43-byte body gzips LARGER than itself, so 3× is plenty of headroom)
    let resp = client()
        .put(format!("{base}/graphs/ok"))
        .header("content-type", "text/turtle")
        .header("content-encoding", "gzip")
        .body(small_gz)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "benign gzip body should write, got {}", resp.status());
}

#[tokio::test]
async fn decompress_disabled_refuses_gzip_body() {
    // ratio 0 => Content-Encoding: gzip bodies are refused outright (fail-closed).
    let config = ServerConfig { max_decompress_ratio: 0, ..ServerConfig::default() };
    let base = spawn_with(DATA, config).await;
    let gz = gzip(b"@prefix ex: <http://ex/> . ex:a ex:p ex:b .");
    let resp = client()
        .put(format!("{base}/graphs/x"))
        .header("content-type", "text/turtle")
        .header("content-encoding", "gzip")
        .body(gz)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 413, "gzip body must be refused when decompression is disabled");
}

// ---------------------------------------------------------------------------
// (4) Panic → 500 (CatchPanicLayer), connection stays alive
// ---------------------------------------------------------------------------

#[tokio::test]
async fn handler_panic_is_500_not_dead_connection() {
    // Wrap a deliberately panicking route in the exact production middleware stack.
    async fn panicking() -> &'static str {
        panic!("boom")
    }
    let app = harden(
        axum::Router::new().route("/panic", axum::routing::get(panicking)),
        &ServerConfig::default(),
    );
    let base = serve(app).await;

    let cl = client();
    let resp = cl.get(format!("{base}/panic")).send().await.unwrap();
    assert_eq!(resp.status(), 500);
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("{\"error\":"), "structured JSON error, got: {body}");

    // The server (and the client's pooled connection) survives the panic.
    let resp = cl.get(format!("{base}/panic")).send().await.unwrap();
    assert_eq!(resp.status(), 500);
}

// ---------------------------------------------------------------------------
// Structured JSON error bodies on the ordinary error paths too
// ---------------------------------------------------------------------------

#[tokio::test]
async fn errors_are_structured_json() {
    let base = spawn_with(DATA, ServerConfig::default()).await;

    // 400 malformed query.
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT WHERE {")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("{\"error\":"), "got: {body}");

    // 405 keeps its Allow header after JSON normalisation.
    let resp = client()
        .request(reqwest::Method::DELETE, format!("{base}/sparql"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 405);
    assert!(resp.headers().contains_key("allow"));
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("{\"error\":"), "got: {body}");
}
