//! [OPUS-4.8] (sq-o5bi) Integration tests for ONLINE consistent-snapshot backup + restore —
//! the `backup` feature's `POST /admin/backup` + `POST /admin/restore` admin routes and the
//! restore-on-start path (`ServerConfig::restore`).
//!
//! These exercise the REAL HTTP path (not a mock): boot the server, apply updates so the
//! generation has moved past 0, take a backup, then restore it into a fresh (or the same)
//! server and assert the restored store is queryable and IDENTICAL. We also assert the
//! load-bearing fail-closed invariant: a corrupt / non-artifact body is rejected and the live
//! store is left untouched; an unauthenticated request is gated; a `--persist` server refuses.
//!
//! Run: `cargo test -p sparq-server --features backup --test backup`

#![cfg(feature = "backup")]

use std::sync::atomic::{AtomicU64, Ordering};

use sparq_core::Graph;
use sparq_server::{router, AppState, ServerConfig};
use tokio::net::TcpListener;

/// Boots the real server over `graph` on an ephemeral port; returns its base URL.
async fn spawn_with(graph: Graph, config: ServerConfig) -> String {
    let app = router(AppState::with_config(graph, config));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

async fn post_update(cl: &reqwest::Client, base: &str, update: &str) -> reqwest::StatusCode {
    cl.post(format!("{base}/sparql"))
        .header("content-type", "application/sparql-update")
        .body(update.to_string())
        .send()
        .await
        .unwrap()
        .status()
}

/// Counts the solutions of `query` over the named/default dataset (`FROM`-free, so the engine
/// queries the whole dataset).
async fn count_rows(cl: &reqwest::Client, base: &str, query: &str) -> usize {
    let body: serde_json::Value = cl
        .get(format!("{base}/sparql"))
        .header("accept", "application/sparql-results+json")
        .query(&[("query", query)])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    body["results"]["bindings"].as_array().unwrap().len()
}

/// Downloads a backup artifact from `POST /admin/backup`, asserting 200.
async fn backup(cl: &reqwest::Client, base: &str) -> Vec<u8> {
    let resp = cl
        .post(format!("{base}/admin/backup"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK, "backup must be 200");
    resp.bytes().await.unwrap().to_vec()
}

/// Counts all quads (default + named) across the dataset via a wildcard query that also walks
/// named graphs through a UNION — so a missing named graph would be caught.
async fn count_all(cl: &reqwest::Client, base: &str) -> usize {
    // `GRAPH ?g { ?s ?p ?o }` covers named graphs; the bare BGP covers the default graph.
    count_rows(
        cl,
        base,
        "SELECT * WHERE { { ?s ?p ?o } UNION { GRAPH ?g { ?s ?p ?o } } }",
    )
    .await
}

/// ROUND TRIP: backup of a moved-on store, restored into a FRESH server, is queryable and
/// equal — and the restored server keeps accepting updates afterward.
#[tokio::test]
async fn backup_then_restore_into_fresh_server_is_identical() {
    let cl = reqwest::Client::new();
    let src = spawn_with(
        Graph::load_str("<http://ex/s0> <http://ex/p> <http://ex/o0> .", "turtle").unwrap(),
        ServerConfig::default(),
    )
    .await;
    // Move the generation past 0 with a default-graph insert and a NAMED-graph insert.
    assert_eq!(
        post_update(
            &cl,
            &src,
            "INSERT DATA { <http://ex/s1> <http://ex/p> <http://ex/o1> }"
        )
        .await,
        reqwest::StatusCode::NO_CONTENT
    );
    assert_eq!(
        post_update(
            &cl,
            &src,
            "INSERT DATA { GRAPH <http://ex/g> { <http://ex/s2> <http://ex/p> <http://ex/o2> } }"
        )
        .await,
        reqwest::StatusCode::NO_CONTENT
    );
    let src_total = count_all(&cl, &src).await;
    assert_eq!(src_total, 3, "1 seed + 1 default insert + 1 named insert");

    let artifact = backup(&cl, &src).await;
    assert!(
        artifact.starts_with(b"SPARQ-BACKUP "),
        "artifact is the self-describing format"
    );

    // Restore into a brand-new empty server.
    let dst = spawn_with(
        Graph::load_str("", "turtle").unwrap(),
        ServerConfig::default(),
    )
    .await;
    assert_eq!(count_all(&cl, &dst).await, 0, "fresh server starts empty");
    let resp = cl
        .post(format!("{dst}/admin/restore"))
        .body(artifact)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::OK,
        "restore must be 200"
    );

    // The restored store is queryable and identical.
    assert_eq!(
        count_all(&cl, &dst).await,
        src_total,
        "restored quad count equals source"
    );
    assert_eq!(
        count_rows(
            &cl,
            &dst,
            "SELECT * WHERE { GRAPH <http://ex/g> { ?s ?p ?o } }"
        )
        .await,
        1,
        "the named graph survived the round trip"
    );
    // The restored server keeps serving WRITES (fresh ring+writer wired in).
    assert_eq!(
        post_update(
            &cl,
            &dst,
            "INSERT DATA { <http://ex/s3> <http://ex/p> <http://ex/o3> }"
        )
        .await,
        reqwest::StatusCode::NO_CONTENT
    );
    assert_eq!(
        count_all(&cl, &dst).await,
        src_total + 1,
        "post-restore update is visible"
    );
}

/// RESTORE-ON-START: a server constructed from a graph rehydrated by the restore-on-start
/// import path serves the restored data immediately. We drive the same `backup_import` the
/// `--restore` flag uses, then boot a server on the result — modelling exactly the binary's
/// startup wiring without spawning a subprocess.
#[tokio::test]
async fn restore_on_start_seeds_the_store() {
    let cl = reqwest::Client::new();
    // Produce an artifact from a populated server.
    let src = spawn_with(
        Graph::load_str("<http://ex/a> <http://ex/p> <http://ex/b> .", "turtle").unwrap(),
        ServerConfig::default(),
    )
    .await;
    post_update(
        &cl,
        &src,
        "INSERT DATA { <http://ex/c> <http://ex/p> <http://ex/d> }",
    )
    .await;
    let artifact = backup(&cl, &src).await;

    // The restore-on-start import path: import the artifact into a seed graph, boot on it.
    let (graph, meta) = sparq_serve::backup_import(&artifact[..]).expect("artifact imports");
    assert_eq!(
        meta.triples, 2,
        "metadata records the captured triple count"
    );
    let restored = spawn_with(graph, ServerConfig::default()).await;
    assert_eq!(
        count_all(&cl, &restored).await,
        2,
        "a server booted from the restored graph serves the data on start"
    );
}

/// FAIL-CLOSED: a non-artifact body is rejected (400) and the live store is untouched.
#[tokio::test]
async fn restore_rejects_non_artifact_and_leaves_store_intact() {
    let cl = reqwest::Client::new();
    let base = spawn_with(
        Graph::load_str("<http://ex/keep> <http://ex/p> <http://ex/o> .", "turtle").unwrap(),
        ServerConfig::default(),
    )
    .await;
    assert_eq!(count_all(&cl, &base).await, 1);

    // A bare N-Quads body looks like a body but lacks the artifact magic → fail closed.
    let resp = cl
        .post(format!("{base}/admin/restore"))
        .body("<http://ex/x> <http://ex/y> <http://ex/z> .\n")
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::BAD_REQUEST,
        "a non-artifact restore is rejected"
    );
    // The live store is unchanged — the rejected restore swapped nothing in.
    assert_eq!(
        count_all(&cl, &base).await,
        1,
        "the store is intact after a rejected restore"
    );
}

/// FAIL-CLOSED: a CORRUPT artifact (a flipped body byte) is rejected (400), store intact.
#[tokio::test]
async fn restore_rejects_corrupt_artifact() {
    let cl = reqwest::Client::new();
    let base = spawn_with(
        Graph::load_str("<http://ex/keep> <http://ex/p> <http://ex/o> .", "turtle").unwrap(),
        ServerConfig::default(),
    )
    .await;
    let mut artifact = backup(&cl, &base).await;
    // Flip the very last byte of the body — the digest check must catch it.
    let last = artifact.len() - 1;
    artifact[last] ^= 0xff;
    let resp = cl
        .post(format!("{base}/admin/restore"))
        .body(artifact)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
    assert_eq!(
        count_all(&cl, &base).await,
        1,
        "store intact after a corrupt restore"
    );
}

/// AUTH: with a write token set, both admin routes require the Bearer token.
#[tokio::test]
async fn admin_routes_are_write_gated() {
    let cl = reqwest::Client::new();
    let config = ServerConfig {
        auth_token: Some("s3cret".into()),
        ..ServerConfig::default()
    };
    let base = spawn_with(
        Graph::load_str("<http://ex/s> <http://ex/p> <http://ex/o> .", "turtle").unwrap(),
        config,
    )
    .await;

    // Unauthenticated backup → 401.
    let resp = cl
        .post(format!("{base}/admin/backup"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
    // Unauthenticated restore → 401.
    let resp = cl
        .post(format!("{base}/admin/restore"))
        .body("anything")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);

    // Authenticated backup → 200.
    let resp = cl
        .post(format!("{base}/admin/backup"))
        .header("authorization", "Bearer s3cret")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
}

/// PERSIST REFUSAL: a `--persist` durable server refuses restore (409) in v1.
#[tokio::test]
async fn restore_refuses_on_persist_server() {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "sparq-backup-persist-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed),
    ));
    let cl = reqwest::Client::new();
    let config = ServerConfig {
        persist_dir: Some(dir.clone()),
        ..ServerConfig::default()
    };
    let base = spawn_with(Graph::load_str("", "turtle").unwrap(), config).await;
    let resp = cl
        .post(format!("{base}/admin/restore"))
        .body("anything")
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::CONFLICT,
        "restore is refused on a --persist server in v1"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
