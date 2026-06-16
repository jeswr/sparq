//! [OPUS-4.8] sq-d3d8 (epic sq-3183): integration tests for the OPT-IN federation
//! discovery descriptors — the W3C VoID document at `GET /.well-known/void` and the SPARQL
//! 1.1 Service Description served for a `GET /sparql` with no `query` parameter.
//!
//! These tests run ONLY with the `federation-descriptors` cargo feature (the whole test
//! file is gated). They spin the real axum server and assert:
//!   * the OPT-IN posture — both endpoints are off unless the config flag is set;
//!   * valid RDF bodies carrying the expected triples (`void:triples`, `sd:endpoint`, …);
//!   * content negotiation (Turtle default, N-Triples on request).
#![cfg(feature = "federation-descriptors")]

use sparq_core::Graph;
use sparq_server::{router, AppState, ServerConfig};
use tokio::net::TcpListener;

const DATA: &str = r#"
    @prefix ex: <http://ex/> .
    ex:alice a ex:Person ; ex:knows ex:bob ; ex:name "Alice" .
    ex:bob   a ex:Person ; ex:name "Bob" .
    ex:carol a ex:Person .
"#;

/// Boots a server with the descriptors flag ON (or OFF) and returns its base URL.
async fn spawn(descriptors_on: bool) -> String {
    let graph = Graph::load_str(DATA, "turtle").unwrap();
    let config = ServerConfig {
        federation_descriptors: descriptors_on,
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
// OPT-IN posture: both endpoints are off unless the flag is set.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn void_404_when_flag_off() {
    let base = spawn(false).await;
    let resp = client()
        .get(format!("{base}/.well-known/void"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn sparql_get_no_query_is_400_when_flag_off() {
    // Historical behaviour preserved: a GET /sparql with no `query` is a 400.
    let base = spawn(false).await;
    let resp = client().get(format!("{base}/sparql")).send().await.unwrap();
    assert_eq!(resp.status(), 400);
}

// ---------------------------------------------------------------------------
// VoID document.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn void_served_as_turtle_by_default() {
    let base = spawn(true).await;
    let resp = client()
        .get(format!("{base}/.well-known/void"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.headers()["content-type"], "text/turtle; charset=utf-8");
    let body = resp.text().await.unwrap();
    // The dataset IRI self-describes the host the client used.
    assert!(
        body.contains("/.well-known/void#dataset"),
        "VoID body: {body}"
    );
    // VoID counts present. The source graph has 6 triples (3 rdf:type + 2 ex:name + 1 ex:knows).
    assert!(
        body.contains("triples"),
        "VoID must carry void:triples: {body}"
    );
    assert!(
        body.contains("triples> 6"),
        "void:triples should be 6: {body}"
    );
    // Class partition for ex:Person (3 instances).
    assert!(
        body.contains("http://ex/Person"),
        "VoID must list the class: {body}"
    );
}

#[tokio::test]
async fn void_negotiates_ntriples() {
    let base = spawn(true).await;
    let resp = client()
        .get(format!("{base}/.well-known/void"))
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
    assert!(
        body.contains("<http://rdfs.org/ns/void#Dataset>"),
        "N-Triples VoID: {body}"
    );
    assert!(
        body.contains("<http://rdfs.org/ns/void#triples>"),
        "N-Triples VoID: {body}"
    );
}

#[tokio::test]
async fn void_carries_characteristic_set_stats() {
    // [OPUS-4.8] sq-mr32 (federation A3/Z2): the served VoID now also carries the
    // characteristic-set source statistics (sparq `scs:` extension), end-to-end over the
    // real HTTP server — so a federation client polling this node gets per-entity-type
    // predicate co-occurrence + multiplicity, not just bare VoID counts.
    let base = spawn(true).await;
    let resp = client()
        .get(format!("{base}/.well-known/void"))
        .header("Accept", "application/n-triples")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    // Standard VoID still present (CS rides alongside, never replaces it).
    assert!(
        body.contains("<http://rdfs.org/ns/void#Dataset>"),
        "VoID still present: {body}"
    );
    // Dataset links to characteristic sets + carries the exact distinct-set count.
    assert!(
        body.contains("<http://sparq.dev/ns/cs#characteristicSet>"),
        "served VoID must link characteristic sets: {body}"
    );
    assert!(
        body.contains("<http://sparq.dev/ns/cs#distinctCharacteristicSets>"),
        "served VoID must carry the distinct-set count: {body}"
    );
    // A typed CS node with subjects + per-predicate stats (property/triples/avgMult).
    assert!(
        body.contains("<http://sparq.dev/ns/cs#CharacteristicSet>"),
        "{body}"
    );
    assert!(body.contains("<http://sparq.dev/ns/cs#subjects>"), "{body}");
    assert!(
        body.contains("<http://sparq.dev/ns/cs#avgMultiplicity>"),
        "{body}"
    );
    // The CS partition reuses void:property and names a real predicate from the graph.
    assert!(
        body.contains("<http://ex/knows>"),
        "CS stat must name the predicate: {body}"
    );
    // The whole body must still be valid RDF (re-parses as N-Triples).
    let n = oxttl::NTriplesParser::new()
        .for_slice(body.as_bytes())
        .filter(|t| t.is_ok())
        .count();
    assert!(
        n >= 10,
        "VoID+CS should re-parse to many triples, got {n}: {body}"
    );
}

// ---------------------------------------------------------------------------
// SPARQL Service Description (GET /sparql with no query).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn service_description_served_on_get_no_query() {
    let base = spawn(true).await;
    let resp = client()
        .get(format!("{base}/sparql"))
        .header("Accept", "text/turtle")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.headers()["content-type"], "text/turtle; charset=utf-8");
    let body = resp.text().await.unwrap();
    // sd:Service + sd:endpoint pointing at this /sparql.
    assert!(
        body.contains("Service"),
        "SD must declare sd:Service: {body}"
    );
    assert!(
        body.contains("/sparql"),
        "SD must carry the endpoint URL: {body}"
    );
    // Advertised query language + a result format.
    assert!(
        body.contains("SPARQL11Query"),
        "SD must advertise SPARQL11Query: {body}"
    );
    assert!(
        body.contains("formats"),
        "SD must advertise result formats: {body}"
    );
    // Link to the VoID document.
    assert!(
        body.contains("/.well-known/void#dataset"),
        "SD must link the VoID dataset: {body}"
    );
}

#[tokio::test]
async fn service_description_ntriples() {
    let base = spawn(true).await;
    let resp = client()
        .get(format!("{base}/sparql"))
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
    assert!(
        body.contains("<http://www.w3.org/ns/sparql-service-description#endpoint>"),
        "SD N-Triples must carry sd:endpoint: {body}"
    );
}

#[tokio::test]
async fn sparql_query_still_works_with_descriptors_on() {
    // The SD only intercepts the no-query GET; a real query is unaffected.
    let base = spawn(true).await;
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT ?s WHERE { ?s a <http://ex/Person> }")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()["content-type"],
        "application/sparql-results+json"
    );
    let body = resp.text().await.unwrap();
    assert_eq!(body.matches("\"type\":\"uri\"").count(), 3);
}
