//! Integration tests for the SPARQL 1.1 Protocol + Graph Store HTTP Protocol (read side).
//!
//! Each test spins the actual axum server on an ephemeral port in-process and drives it
//! over real HTTP with `reqwest`, asserting the protocol's request forms, exact result
//! media types, payload shapes, ASK booleans and HTTP status semantics. This is structured
//! so the official W3C SPARQL Protocol test suite could be pointed at the running endpoint
//! (see the crate README for how to run conformance).

use sparq_core::Graph;
use sparq_server::{router, AppState};
use tokio::net::TcpListener;

const DATA: &str = r#"
    @prefix ex: <http://ex/> .
    ex:alice ex:knows ex:bob ; ex:age 30 ; ex:name "Alice" .
    ex:bob   ex:age 25 ; ex:name "Bob"@en .
    ex:carol ex:age 35 .
"#;

/// Boots the server on a random local port and returns its base URL.
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

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

// ---------------------------------------------------------------------------
// Query operation — request forms
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_query_json_default() {
    let base = spawn().await;
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT ?s WHERE { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()["content-type"],
        "application/sparql-results+json"
    );
    let body = resp.text().await.unwrap();
    assert!(body.contains("\"head\""));
    assert!(body.contains("\"bindings\""));
    // three subjects have ex:age
    assert_eq!(body.matches("\"type\":\"uri\"").count(), 3);
}

#[tokio::test]
async fn post_direct_sparql_query() {
    let base = spawn().await;
    let resp = client()
        .post(format!("{base}/sparql"))
        .header("content-type", "application/sparql-query")
        .header("accept", "application/sparql-results+json")
        .body("SELECT ?s WHERE { ?s <http://ex/age> ?a }")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("\"bindings\""));
}

#[tokio::test]
async fn post_urlencoded_query() {
    let base = spawn().await;
    let resp = client()
        .post(format!("{base}/sparql"))
        .form(&[("query", "SELECT ?s WHERE { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()["content-type"],
        "application/sparql-results+json"
    );
}

// ---------------------------------------------------------------------------
// Result format content negotiation
// ---------------------------------------------------------------------------

async fn select_with_accept(base: &str, accept: &str) -> reqwest::Response {
    client()
        .get(format!("{base}/sparql"))
        .header("accept", accept)
        .query(&[("query", "SELECT ?s ?a WHERE { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn negotiate_xml() {
    let base = spawn().await;
    let resp = select_with_accept(&base, "application/sparql-results+xml").await;
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()["content-type"],
        "application/sparql-results+xml"
    );
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("<?xml"));
    assert!(body.contains("xmlns=\"http://www.w3.org/2005/sparql-results#\""));
    assert!(body.contains("<variable name=\"s\"/>"));
}

#[tokio::test]
async fn negotiate_csv() {
    let base = spawn().await;
    let resp = select_with_accept(&base, "text/csv").await;
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.headers()["content-type"], "text/csv; charset=utf-8");
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("s,a\r\n"));
}

#[tokio::test]
async fn negotiate_tsv() {
    let base = spawn().await;
    let resp = select_with_accept(&base, "text/tab-separated-values").await;
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()["content-type"],
        "text/tab-separated-values; charset=utf-8"
    );
    let body = resp.text().await.unwrap();
    assert!(body.starts_with("?s\t?a\n"));
    assert!(body.contains("\"30\"^^<http://www.w3.org/2001/XMLSchema#integer>"));
}

// ---------------------------------------------------------------------------
// ASK
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ask_json_true_and_false() {
    let base = spawn().await;
    let t = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "ASK { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(t.status(), 200);
    assert_eq!(
        t.headers()["content-type"],
        "application/sparql-results+json"
    );
    assert_eq!(t.text().await.unwrap(), "{\"head\":{},\"boolean\":true}");

    let f = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "ASK { ?s <http://ex/nope> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(f.text().await.unwrap(), "{\"head\":{},\"boolean\":false}");
}

#[tokio::test]
async fn ask_xml() {
    let base = spawn().await;
    let resp = client()
        .get(format!("{base}/sparql"))
        .header("accept", "application/sparql-results+xml")
        .query(&[("query", "ASK { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.headers()["content-type"],
        "application/sparql-results+xml"
    );
    assert!(resp.text().await.unwrap().contains("<boolean>true</boolean>"));
}

// ---------------------------------------------------------------------------
// HTTP semantics: 400 / 405 / 501 / HEAD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn malformed_query_is_400() {
    let base = spawn().await;
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[("query", "SELECT WHERE {")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn missing_query_param_is_400() {
    let base = spawn().await;
    let resp = client().get(format!("{base}/sparql")).send().await.unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn unsupported_method_is_405_with_allow() {
    let base = spawn().await;
    let resp = client()
        .request(reqwest::Method::DELETE, format!("{base}/sparql"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 405);
    let allow = resp.headers()["allow"].to_str().unwrap();
    assert!(allow.contains("GET"));
    assert!(allow.contains("POST"));
}

#[tokio::test]
async fn sparql_update_insert_then_query() {
    let base = spawn().await;
    let cl = client();
    // INSERT DATA via the SPARQL 1.1 Protocol update operation -> 204 No Content.
    let resp = cl
        .post(format!("{base}/sparql"))
        .header("content-type", "application/sparql-update")
        .body("INSERT DATA { <http://ex/newS> <http://ex/newP> <http://ex/newO> }")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // The inserted triple is now visible to a query against the swapped-in graph.
    let body = cl
        .get(format!("{base}/sparql"))
        .header("accept", "application/sparql-results+json")
        .query(&[("query", "SELECT ?o WHERE { <http://ex/newS> <http://ex/newP> ?o }")])
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(body.contains("http://ex/newO"), "inserted object should be queryable: {body}");

    // A malformed update is a 400.
    let bad = cl
        .post(format!("{base}/sparql"))
        .header("content-type", "application/sparql-update")
        .body("INSERT DATA { not valid sparql")
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 400);
}

#[tokio::test]
async fn head_query_has_no_body_but_content_type() {
    let base = spawn().await;
    let resp = client()
        .head(format!("{base}/sparql"))
        .query(&[("query", "SELECT ?s WHERE { ?s <http://ex/age> ?a }")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()["content-type"],
        "application/sparql-results+json"
    );
    assert_eq!(resp.text().await.unwrap(), "");
}

// ---------------------------------------------------------------------------
// Graph Store HTTP Protocol — READ side
// ---------------------------------------------------------------------------

#[tokio::test]
async fn gsp_get_default_graph_indirect() {
    let base = spawn().await;
    let resp = client()
        .get(format!("{base}/sparql/graph?default"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp.headers()["content-type"].to_str().unwrap().to_string();
    assert!(ct.starts_with("application/n-triples"));
    let body = resp.text().await.unwrap();
    // dump must contain the triples in N-Triples syntax
    assert!(body.contains("<http://ex/alice> <http://ex/age>"));
    assert!(body.lines().all(|l| l.is_empty() || l.ends_with(" .")));
}

#[tokio::test]
async fn gsp_get_direct_graph_turtle_accept() {
    let base = spawn().await;
    let resp = client()
        .get(format!("{base}/graphs/mygraph"))
        .header("accept", "text/turtle")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp.headers()["content-type"]
        .to_str()
        .unwrap()
        .starts_with("text/turtle"));
}

#[tokio::test]
async fn gsp_write_is_501() {
    let base = spawn().await;
    let resp = client()
        .put(format!("{base}/sparql/graph?default"))
        .body("<http://ex/x> <http://ex/p> <http://ex/y> .")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 501);
}

#[tokio::test]
async fn gsp_indirect_requires_graph_selector() {
    let base = spawn().await;
    let resp = client()
        .get(format!("{base}/sparql/graph"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

// ---------------------------------------------------------------------------
// dataset params accepted (no effect with a single default graph)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn default_graph_uri_param_is_accepted() {
    let base = spawn().await;
    let resp = client()
        .get(format!("{base}/sparql"))
        .query(&[
            ("query", "SELECT ?s WHERE { ?s <http://ex/age> ?a }"),
            ("default-graph-uri", "http://ex/g"),
        ])
        .send()
        .await
        .unwrap();
    // accepted + threaded through; with one default graph it has no effect but must not error
    assert_eq!(resp.status(), 200);
}
