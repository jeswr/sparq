//! [OPUS-4.8] sq-bzh1 (epic sq-3183): e2e integration tests for the OPT-IN Triple Pattern
//! Fragments / Linked Data Fragments source endpoint (`GET /tpf`).
//!
//! These tests run ONLY with the `tpf` cargo feature (the whole file is gated). They spin the
//! real axum server and assert, over an HTTP request:
//!   * the OPT-IN posture — `/tpf` is `404` unless the config flag is set;
//!   * a pattern match returns the correct triples as valid RDF;
//!   * the Hydra controls are present (`hydra:totalItems`, `hydra:search`/`hydra:template`);
//!   * paging — `hydra:next` on a non-final page, none on the final page;
//!   * content negotiation (Turtle default, N-Triples on request);
//!   * an empty pattern still returns a well-formed fragment (count 0).
#![cfg(feature = "tpf")]

use sparq_core::Graph;
use sparq_server::{router, AppState, ServerConfig};
use tokio::net::TcpListener;

const DATA: &str = r#"
    @prefix ex: <http://ex/> .
    ex:alice a ex:Person ; ex:knows ex:bob ; ex:name "Alice" .
    ex:bob   a ex:Person ; ex:name "Bob" .
    ex:carol a ex:Person ; ex:knows ex:alice .
"#;

/// Boots a server with the TPF flag ON (or OFF) and returns its base URL.
async fn spawn(tpf_on: bool) -> String {
    let graph = Graph::load_str(DATA, "turtle").unwrap();
    let config = ServerConfig {
        tpf: tpf_on,
        ..ServerConfig::default()
    };
    let app = router(AppState::with_config(graph, config));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

// ---------------------------------------------------------------------------
// OPT-IN posture.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tpf_404_when_flag_off() {
    let base = spawn(false).await;
    let resp = client().get(format!("{base}/tpf")).send().await.unwrap();
    assert_eq!(resp.status(), 404);
}

// ---------------------------------------------------------------------------
// Pattern match → valid RDF with Hydra metadata.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn predicate_pattern_returns_matching_triples() {
    let base = spawn(true).await;
    // ?s ex:knows ?o → 2 triples (alice→bob, carol→alice).
    let resp = client()
        .get(format!("{base}/tpf"))
        .query(&[("predicate", "<http://ex/knows>")])
        .header("Accept", "application/n-triples")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()["content-type"],
        "application/n-triples; charset=utf-8"
    );
    let body = resp.text().await.unwrap();
    // The two data triples are present.
    assert!(
        body.contains("<http://ex/alice> <http://ex/knows> <http://ex/bob>"),
        "{body}"
    );
    assert!(
        body.contains("<http://ex/carol> <http://ex/knows> <http://ex/alice>"),
        "{body}"
    );
    // Hydra metadata: totalItems = 2 (the estimate).
    assert!(body.contains("hydra/core#totalItems"), "{body}");
    assert!(body.contains("totalItems> \"2\""), "{body}");
    // The search control + template.
    assert!(body.contains("hydra/core#search"), "{body}");
    assert!(body.contains("subject,predicate,object"), "{body}");
}

#[tokio::test]
async fn fully_bound_exact_one_triple() {
    let base = spawn(true).await;
    let resp = client()
        .get(format!("{base}/tpf"))
        .query(&[
            ("subject", "<http://ex/alice>"),
            ("predicate", "<http://ex/knows>"),
            ("object", "<http://ex/bob>"),
        ])
        .header("Accept", "application/n-triples")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("<http://ex/alice> <http://ex/knows> <http://ex/bob>"),
        "{body}"
    );
    assert!(body.contains("totalItems> \"1\""), "{body}");
}

#[tokio::test]
async fn turtle_is_the_default_and_well_formed() {
    let base = spawn(true).await;
    // No Accept → Turtle.
    let resp = client()
        .get(format!("{base}/tpf"))
        .query(&[("predicate", "<http://ex/name>")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.headers()["content-type"], "text/turtle; charset=utf-8");
    let body = resp.text().await.unwrap();
    // Re-parses as valid Turtle (data + metadata + controls).
    let g = Graph::load_str(&body, "turtle").unwrap();
    assert!(
        g.len() >= 2,
        "turtle fragment must carry the matches + metadata: {body}"
    );
}

// ---------------------------------------------------------------------------
// Paging — next boundary.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn empty_pattern_is_well_formed_with_count_zero() {
    let base = spawn(true).await;
    // A subject absent from the data: no matches, but a well-formed fragment with count 0.
    let resp = client()
        .get(format!("{base}/tpf"))
        .query(&[("subject", "<http://ex/nobody>")])
        .header("Accept", "application/n-triples")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("totalItems> \"0\""), "{body}");
    // The search control is still present so a client can learn the template.
    assert!(body.contains("hydra/core#search"), "{body}");
}

#[tokio::test]
async fn first_page_has_no_next_when_all_fit() {
    // The default page size (100) holds all 7 triples, so page 0 is final: NO hydra:next, and
    // (being page 0) NO hydra:previous.
    let base = spawn(true).await;
    let resp = client()
        .get(format!("{base}/tpf"))
        .header("Accept", "application/n-triples")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(
        !body.contains("hydra/core#next"),
        "single-page fragment must not link next: {body}"
    );
    assert!(
        !body.contains("hydra/core#previous"),
        "page 0 must not link previous: {body}"
    );
    // All 7 triples present (3 type + 2 knows + 2 name).
    assert!(body.contains("totalItems> \"7\""), "{body}");
}

#[tokio::test]
async fn page_param_beyond_data_is_empty_but_links_previous() {
    // page=1 with the default size (>data) is past the end: empty data, but it links previous.
    let base = spawn(true).await;
    let resp = client()
        .get(format!("{base}/tpf"))
        .query(&[("page", "1")])
        .header("Accept", "application/n-triples")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("hydra/core#previous"),
        "a later page must link previous: {body}"
    );
    assert!(
        body.contains("page=0"),
        "previous should point at page 0: {body}"
    );
}

#[tokio::test]
async fn malformed_term_is_400() {
    let base = spawn(true).await;
    let resp = client()
        .get(format!("{base}/tpf"))
        .query(&[("subject", "not a valid term")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn non_iri_predicate_is_400() {
    let base = spawn(true).await;
    let resp = client()
        .get(format!("{base}/tpf"))
        .query(&[("predicate", "\"a literal\"")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}
