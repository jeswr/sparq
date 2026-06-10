//! The opt-in `geo` cargo feature: GeoSPARQL `geof:` functions over real HTTP.
//!
//! Runs only with `cargo test -p sparq-server --features geo`. Each test boots the
//! actual axum server in-process and drives `/sparql` with queries/updates whose
//! FILTER/BIND expressions call `geof:` — proving the server installs sparq-geo's
//! registry on the query path (spawn_blocking workers) and the update path (the
//! double-buffered writer, both the rebuild and the in-place application).
#![cfg(feature = "geo")]

use sparq_core::Graph;
use sparq_server::{router, AppState};
use tokio::net::TcpListener;

const DATA: &str = r#"
    @prefix geo: <http://www.opengis.net/ont/geosparql#> .
    @prefix ex:  <http://ex/> .
    ex:london ex:loc "POINT(-0.1278 51.5074)"^^geo:wktLiteral .
    ex:paris  ex:loc "POINT(2.3522 48.8566)"^^geo:wktLiteral .
    ex:lyon   ex:loc "POINT(4.8357 45.7640)"^^geo:wktLiteral .
"#;

const PREFIXES: &str = "PREFIX geof: <http://www.opengis.net/def/function/geosparql/> \
                        PREFIX uom:  <http://www.opengis.net/def/uom/OGC/1.0/> \
                        PREFIX ex:   <http://ex/> ";

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

#[tokio::test]
async fn select_with_geof_distance_filter() {
    let base = spawn().await;
    // Cities within 400 km of London: Paris (≈343.6 km) only.
    let resp = reqwest::Client::new()
        .get(format!("{base}/sparql"))
        .query(&[(
            "query",
            format!(
                "{PREFIXES} SELECT ?city WHERE {{ \
                   ex:london ex:loc ?here . ?city ex:loc ?there . \
                   FILTER(?city != ex:london && geof:distance(?here, ?there, uom:kilometre) < 400) \
                 }}"
            ),
        )])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("http://ex/paris"), "got: {body}");
    assert!(!body.contains("http://ex/lyon"), "got: {body}");
}

#[tokio::test]
async fn ask_with_geof_relation() {
    let base = spawn().await;
    let resp = reqwest::Client::new()
        .get(format!("{base}/sparql"))
        .query(&[(
            "query",
            format!(
                "{PREFIXES} ASK {{ ex:paris ex:loc ?pt . \
                   FILTER(geof:sfWithin(?pt, \"POLYGON((-1 42.5, 7 42.5, 7 51, -1 51, -1 42.5))\"^^<http://www.opengis.net/ont/geosparql#wktLiteral>)) }}"
            ),
        )])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("\"boolean\":true") || body.contains("\"boolean\": true"), "got: {body}");
}

#[tokio::test]
async fn update_where_with_geof_filter() {
    let base = spawn().await;
    let client = reqwest::Client::new();
    // The update path (writer rebuild + in-place): classify cities near London.
    let update = format!(
        "{PREFIXES} INSERT {{ ?city ex:nearLondon true }} WHERE {{ \
           ex:london ex:loc ?here . ?city ex:loc ?there . \
           FILTER(?city != ex:london && geof:distance(?here, ?there, uom:kilometre) < 400) \
         }}"
    );
    // Twice: the first update exercises apply_by_rebuild, the second apply_in_place.
    for _ in 0..2 {
        let resp = client
            .post(format!("{base}/sparql"))
            .header("content-type", "application/sparql-update")
            .body(update.clone())
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 204);
    }
    let resp = client
        .get(format!("{base}/sparql"))
        .query(&[("query", format!("{PREFIXES} SELECT ?c WHERE {{ ?c ex:nearLondon true }}"))])
        .send()
        .await
        .unwrap();
    let body = resp.text().await.unwrap();
    assert!(body.contains("http://ex/paris"), "got: {body}");
    assert!(!body.contains("http://ex/lyon"), "got: {body}");
}

#[tokio::test]
async fn unregistered_geof_iri_is_still_an_engine_error() {
    let base = spawn().await;
    // geof:buffer is not in the registry: the engine's hard unknown-function
    // error surfaces as the server's 500 engine-error response, not a 200.
    let resp = reqwest::Client::new()
        .get(format!("{base}/sparql"))
        .query(&[(
            "query",
            format!("{PREFIXES} SELECT ?s WHERE {{ ?s ex:loc ?g . FILTER(geof:buffer(?g, 1.0) > 0) }}"),
        )])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 500);
    let body = resp.text().await.unwrap();
    assert!(body.contains("unsupported SPARQL function"), "got: {body}");
}
