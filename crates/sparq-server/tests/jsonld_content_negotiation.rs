//! [OPUS-4.8] sq-oy1f.1 — Server JSON-LD content negotiation integration tests.
//!
//! Spins the real axum server in-process and drives it over HTTP with `reqwest`, exercising
//! BOTH directions of the `application/ld+json` wiring and the negotiation/contract corners:
//!
//!   * EMIT — a SPARQL `CONSTRUCT` with `Accept: application/ld+json` returns valid JSON-LD
//!     whose `toRdf` round-trips back to the same triples (asserted by re-parsing the body
//!     through the engine's JSON-LD loader and comparing the canonical triple set).
//!   * ACCEPT — a Graph-Store-Protocol `PUT` of an `application/ld+json` body, then a `GET`,
//!     returns exactly the triples that were loaded.
//!   * the q-value-aware negotiation contract: a JSON-LD `Accept` beats a lower-q sibling, a
//!     `q=0` rejects it, an unknown `Accept` does not 406 (the server has a default), and an
//!     unsupported write `Content-Type` is a `415`.
//!
//! The positive tests compile with the `jsonld` feature — which, as of sq-oy1f.4, is in the
//! server's DEFAULT set (a maintainer-directed exception to opt-in-by-default), so the plain
//! `cargo test -p sparq-server` runs them. A separate block under `#[cfg(not(feature =
//! "jsonld"))]` asserts the feature-OFF contract (`--no-default-features --features server`): an
//! `application/ld+json` write body is a plain `415`, and an `Accept: application/ld+json` query
//! falls back to a supported graph format (NOT a 406) — so a JSON-LD-disabled build is unchanged.

use sparq_core::Graph;
use sparq_server::{router, AppState};
use tokio::net::TcpListener;

const DATA: &str = r#"
    @prefix ex: <http://ex/> .
    ex:alice ex:knows ex:bob ; ex:age 30 ; ex:name "Alice" .
    ex:bob   ex:age 25 ; ex:name "Bob"@en .
    ex:carol ex:age 35 .
"#;

/// Boots the server on a random local port over the loaded `DATA` graph; returns its base URL.
async fn spawn() -> String {
    let graph = Graph::load_str(DATA, "turtle").unwrap();
    let app = router(AppState::new(graph));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

/// Boots an EMPTY server (no initial data) — for the GSP write-then-read round-trip.
async fn spawn_empty() -> String {
    let graph = Graph::load_str("", "turtle").unwrap();
    let app = router(AppState::new(graph));
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

// ===========================================================================
// Feature ON: the real JSON-LD wiring.
// ===========================================================================

/// The canonical, order-independent triple set of an RDF document, for round-trip equality.
/// We parse via the engine (the same parser the server uses) and compare N-Triples lines as a
/// sorted set — JSON-LD specifies no node order, so a set comparison is the correct invariant.
#[cfg(feature = "jsonld")]
fn triple_set(text: &str, format: &str) -> std::collections::BTreeSet<String> {
    let g = Graph::load_str(text, format).expect("parse RDF document");
    let nt =
        sparq_engine::construct_or_describe(&g, "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }").unwrap();
    nt.iter()
        .map(|t| format!("{} {} {} .", t.subject, t.predicate, t.object))
        .collect()
}

/// EMIT: a `CONSTRUCT` with `Accept: application/ld+json` returns JSON-LD with the right
/// `Content-Type`, and that JSON-LD `toRdf` round-trips back to the same triples as the
/// equivalent N-Triples emission. This is the load-bearing result-equivalence invariant.
#[cfg(feature = "jsonld")]
#[tokio::test]
async fn construct_accept_jsonld_roundtrips() {
    let base = spawn().await;
    let query = "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }";

    // JSON-LD emission.
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", query)])
        .header("accept", "application/ld+json")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.headers()["content-type"], "application/ld+json; charset=utf-8");
    let jsonld = resp.text().await.unwrap();
    // Flattened form: a `@graph` envelope of node objects.
    assert!(jsonld.contains("@graph"), "flattened JSON-LD carries an @graph envelope: {jsonld}");

    // N-Triples emission of the same CONSTRUCT, as the reference triple set.
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", query)])
        .header("accept", "application/n-triples")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ntriples = resp.text().await.unwrap();

    // toRdf(JSON-LD) == the N-Triples triple set. This proves the emitted JSON-LD is valid AND
    // lossless — the real serialiser path, not a mock.
    assert_eq!(
        triple_set(&jsonld, "jsonld"),
        triple_set(&ntriples, "ntriples"),
        "JSON-LD CONSTRUCT output must round-trip to the same RDF graph as N-Triples"
    );
    // Sanity: the round-trip is non-trivial (the DATA is a multi-triple graph), so the
    // equality above is a real assertion, not a trivially-empty-set match.
    assert!(triple_set(&ntriples, "ntriples").len() >= 5);
}

/// ACCEPT: a GSP `PUT` of an `application/ld+json` body, then a `GET`, returns exactly the
/// loaded triples — the parse path wired through the engine's JSON-LD loader.
#[cfg(feature = "jsonld")]
#[tokio::test]
async fn gsp_put_jsonld_then_get_roundtrips() {
    let base = spawn_empty().await;
    // An expanded JSON-LD document: two triples on one subject.
    let jsonld_body = r#"{
        "@id": "http://ex/widget",
        "http://ex/label": [{"@value": "Widget"}],
        "http://ex/count": [{"@value": "7", "@type": "http://www.w3.org/2001/XMLSchema#integer"}]
    }"#;

    let put = client()
        .put(format!("{base}/graphs/things"))
        .header("content-type", "application/ld+json")
        .body(jsonld_body)
        .send()
        .await
        .unwrap();
    assert!(
        put.status().is_success(),
        "GSP PUT of application/ld+json should succeed, got {}",
        put.status()
    );

    // GET the graph back as N-Triples and compare the triple set to what we loaded.
    let get = client()
        .get(format!("{base}/graphs/things"))
        .header("accept", "application/n-triples")
        .send()
        .await
        .unwrap();
    assert_eq!(get.status(), 200);
    let back = get.text().await.unwrap();
    let got = triple_set(&back, "ntriples");
    let expected = triple_set(jsonld_body, "jsonld");
    assert_eq!(got, expected, "GSP PUT(JSON-LD) -> GET must return the same triples");
    assert_eq!(expected.len(), 2, "the body carries exactly two triples");
}

/// ACCEPT then re-EMIT as JSON-LD: a PUT of JSON-LD followed by a GET with
/// `Accept: application/ld+json` round-trips through both wired directions at once.
#[cfg(feature = "jsonld")]
#[tokio::test]
async fn gsp_put_jsonld_then_get_jsonld_roundtrips() {
    let base = spawn_empty().await;
    let jsonld_body = r#"[
        {"@id": "http://ex/a", "http://ex/p": [{"@id": "http://ex/b"}]},
        {"@id": "http://ex/b", "http://ex/q": [{"@value": "leaf"}]}
    ]"#;
    let put = client()
        .put(format!("{base}/graphs/g"))
        .header("content-type", "application/ld+json")
        .body(jsonld_body)
        .send()
        .await
        .unwrap();
    assert!(put.status().is_success());

    let get = client()
        .get(format!("{base}/graphs/g"))
        .header("accept", "application/ld+json")
        .send()
        .await
        .unwrap();
    assert_eq!(get.status(), 200);
    assert_eq!(get.headers()["content-type"], "application/ld+json; charset=utf-8");
    let back = get.text().await.unwrap();
    assert_eq!(
        triple_set(&back, "jsonld"),
        triple_set(jsonld_body, "jsonld"),
        "PUT(JSON-LD) -> GET(JSON-LD) must round-trip the graph"
    );
}

/// Negotiation contract: a higher-q JSON-LD wins over a lower-q Turtle; `q=0` rejects JSON-LD.
#[cfg(feature = "jsonld")]
#[tokio::test]
async fn construct_jsonld_qvalue_contract() {
    let base = spawn().await;
    let query = "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }";

    // JSON-LD q=0.9 beats Turtle q=0.5.
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", query)])
        .header("accept", "text/turtle;q=0.5, application/ld+json;q=0.9")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.headers()["content-type"], "application/ld+json; charset=utf-8");

    // q=0 rejects JSON-LD, so Turtle is served.
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", query)])
        .header("accept", "application/ld+json;q=0, text/turtle")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.headers()["content-type"], "text/turtle; charset=utf-8");
}

/// A GSP write body of an UNSUPPORTED type is still a `415` even with the feature on, and its
/// message advertises JSON-LD among the accepted dialects.
#[cfg(feature = "jsonld")]
#[tokio::test]
async fn gsp_put_unsupported_type_is_415_with_jsonld_advertised() {
    let base = spawn_empty().await;
    let resp = client()
        .put(format!("{base}/graphs/x"))
        .header("content-type", "application/pdf")
        .body("not rdf")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 415);
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("application/ld+json"),
        "feature-ON 415 message advertises JSON-LD: {body}"
    );
}

// ===========================================================================
// Feature OFF: the default build is byte-identical — JSON-LD is unsupported.
// ===========================================================================

/// Without the `jsonld` feature, an `application/ld+json` GSP write body is a plain `415`, and
/// the message does NOT advertise JSON-LD (the build cannot parse it).
#[cfg(not(feature = "jsonld"))]
#[tokio::test]
async fn gsp_put_jsonld_is_415_without_feature() {
    let base = spawn_empty().await;
    let resp = client()
        .put(format!("{base}/graphs/x"))
        .header("content-type", "application/ld+json")
        .body(r#"{"@id":"http://ex/a"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 415);
    let body = resp.text().await.unwrap();
    assert!(
        !body.contains("ld+json"),
        "feature-OFF 415 message must not advertise JSON-LD: {body}"
    );
}

/// Without the feature, an `Accept: application/ld+json` CONSTRUCT does NOT 406 — content
/// negotiation falls back to a supported graph format (N-Triples, the default). The endpoint
/// always has a representation, so it answers 200 with that fallback, never a 406.
#[cfg(not(feature = "jsonld"))]
#[tokio::test]
async fn construct_accept_jsonld_falls_back_without_feature() {
    let base = spawn().await;
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }")])
        .header("accept", "application/ld+json")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp.headers()["content-type"].to_str().unwrap().to_string();
    assert!(
        !ct.contains("ld+json"),
        "feature-OFF build must not emit JSON-LD; fell back to {ct}"
    );
}
