//! [OPUS-4.8] (sq-7cxr, gh-44) Integration tests for DURABLE PERSISTENCE — the QLever
//! `--persist-updates` equivalent wired onto the engine's directory-backed `Graph` + WAL.
//!
//! The acceptance criterion (gh-44): **a process RESTART preserves ALL updates with NO
//! rebuild.** We prove it the way an operator would: start a server with `--persist <dir>`
//! (modelled by `ServerConfig::persist_dir`), apply several UPDATEs — default graph AND a
//! named graph — over HTTP, SHUT THE SERVER DOWN (drop its state so the writer thread joins
//! and every WAL handle is released), start a BRAND-NEW server on the SAME `<dir>`, and assert
//! every triple is present via query — with no explicit rebuild step. We also assert the
//! back-compat contract: with NO persist dir the server is in-memory, so a "restart" (a fresh
//! in-memory server) does NOT see the prior updates.
//!
//! Durability point: each `post_update` 204 is the group-commit ack, which the writer only
//! sends AFTER `ServerApplier::seal` has WAL-fsync'd the batch to the durable graph (see
//! `src/http.rs`). So by the time a 204 returns, the update is already on disk — the restart
//! merely re-opens it.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use sparq_core::Graph;
use sparq_server::{router, AppState, ServerConfig};
use tokio::net::TcpListener;

/// A unique scratch directory under the system temp dir, removed on drop so the test leaves
/// no persist-dir cruft behind (repo hygiene + the `df` watchdog). No `tempfile` dep in the
/// workspace, so this mirrors the existing `std::env::temp_dir()` + manual-cleanup pattern.
struct ScratchDir(PathBuf);

impl ScratchDir {
    fn new() -> Self {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "sparq-persist-test-{}-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
        ));
        ScratchDir(p)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A running server we can SHUT DOWN: holds the graceful-shutdown trigger + the serve task's
/// join handle, so a test can drop the whole thing and be sure the writer thread joined (which
/// happens when the last `AppState`/`Writer` Arc drops) before reopening the same persist dir.
struct Server {
    base: String,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    task: Option<tokio::task::JoinHandle<()>>,
}

impl Server {
    /// Boots a server on an ephemeral port with the given config, returning a handle whose
    /// `base` is the URL. The serve future resolves on the shutdown signal, so `stop()` truly
    /// tears the server (and its durable writer) down.
    async fn start(config: ServerConfig) -> Self {
        // Empty seed; an existing persist dir is opened (seed ignored), an empty one is created.
        let graph = Graph::load_str("", "turtle").unwrap();
        let state = AppState::try_with_config(graph, config).expect("durable open");
        let app = router(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = rx.await;
                })
                .await
                .unwrap();
        });
        Server { base: format!("http://{addr}"), shutdown: Some(tx), task: Some(task) }
    }

    /// Signals graceful shutdown and waits for the serve task to finish, so the `AppState`
    /// (and the writer thread it owns) is dropped — modelling a clean process restart.
    async fn stop(mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
    }
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

/// Number of solutions to `query` (counts JSON result bindings).
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

fn persist_config(dir: &Path) -> ServerConfig {
    ServerConfig { persist_dir: Some(dir.to_path_buf()), ..ServerConfig::default() }
}

// ---------------------------------------------------------------------------
// The acceptance test: restart preserves ALL updates (default + named) with NO rebuild.
// ---------------------------------------------------------------------------

/// gh-44 ACCEPTANCE. Apply default-graph and named-graph updates to a `--persist` server,
/// shut it down, start a NEW server on the SAME dir, and assert every triple is present —
/// no explicit rebuild. This is the showstopper the PSS migration was blocked on.
#[tokio::test]
async fn restart_preserves_all_updates_no_rebuild() {
    let scratch = ScratchDir::new();
    let cl = reqwest::Client::new();

    // --- Process 1: write into the default graph AND a named graph, then DELETE one. ---
    let s1 = Server::start(persist_config(scratch.path())).await;
    for i in 0..5 {
        let ins = format!("INSERT DATA {{ <http://ex/d{i}> <http://ex/p> <http://ex/v> }}");
        assert_eq!(post_update(&cl, &s1.base, &ins).await, 204, "default insert {i}");
    }
    // A GRAPH-scoped INSERT that first creates a brand-new named graph (the durability gap
    // this work closed: the new named graph must be born directory-backed / WAL'd).
    for i in 0..3 {
        let ins = format!(
            "INSERT DATA {{ GRAPH <http://ex/g1> {{ <http://ex/n{i}> <http://ex/q> <http://ex/w> }} }}"
        );
        assert_eq!(post_update(&cl, &s1.base, &ins).await, 204, "named insert {i}");
    }
    // Delete one default-graph triple — the retraction must persist too (not resurrect on reopen).
    assert_eq!(
        post_update(&cl, &s1.base, "DELETE DATA { <http://ex/d0> <http://ex/p> <http://ex/v> }").await,
        204
    );

    // Sanity in-process before restart: 4 default + 3 named.
    assert_eq!(count_rows(&cl, &s1.base, "SELECT * WHERE { ?s <http://ex/p> ?o }").await, 4);
    assert_eq!(
        count_rows(&cl, &s1.base, "SELECT * WHERE { GRAPH <http://ex/g1> { ?s ?p ?o } }").await,
        3
    );

    // --- RESTART: tear the first server fully down, boot a fresh one on the SAME dir. ---
    s1.stop().await;
    let s2 = Server::start(persist_config(scratch.path())).await;

    // No rebuild step ran — `Graph::open` replayed the WAL. Everything must be present.
    assert_eq!(
        count_rows(&cl, &s2.base, "SELECT * WHERE { ?s <http://ex/p> ?o }").await,
        4,
        "default-graph inserts (minus the one delete) must survive the restart"
    );
    // The deleted triple must STAY deleted across the restart (the retraction was WAL'd).
    assert_eq!(
        count_rows(&cl, &s2.base, "SELECT * WHERE { <http://ex/d0> <http://ex/p> ?o }").await,
        0,
        "the deletion must persist (not resurrect on reopen)"
    );
    // Named-graph triples must survive too — the showstopper-within-the-showstopper.
    assert_eq!(
        count_rows(&cl, &s2.base, "SELECT * WHERE { GRAPH <http://ex/g1> { ?s ?p ?o } }").await,
        3,
        "named-graph inserts must survive the restart"
    );

    // And the reopened server is still WRITABLE and durable: one more update, restart again.
    assert_eq!(
        post_update(&cl, &s2.base, "INSERT DATA { <http://ex/d9> <http://ex/p> <http://ex/v> }").await,
        204
    );
    s2.stop().await;
    let s3 = Server::start(persist_config(scratch.path())).await;
    assert_eq!(
        count_rows(&cl, &s3.base, "SELECT * WHERE { ?s <http://ex/p> ?o }").await,
        5,
        "an update applied after the first restart must itself survive a second restart"
    );
    s3.stop().await;
}

// ---------------------------------------------------------------------------
// Back-compat: no persist dir => in-memory => a fresh server does NOT see prior updates.
// ---------------------------------------------------------------------------

/// Without `--persist`, the server is purely in-memory: a brand-new server (a "restart")
/// starts empty. This pins the historical behaviour so the new flag is the ONLY thing that
/// changes durability — `persist_dir == None` is byte-for-byte the old in-memory server.
#[tokio::test]
async fn no_persist_dir_is_in_memory_and_lost_on_restart() {
    let cl = reqwest::Client::new();

    let s1 = Server::start(ServerConfig::default()).await;
    assert_eq!(
        post_update(&cl, &s1.base, "INSERT DATA { <http://ex/x> <http://ex/p> <http://ex/y> }").await,
        204
    );
    assert_eq!(count_rows(&cl, &s1.base, "SELECT * WHERE { ?s ?p ?o }").await, 1);
    s1.stop().await;

    // A new in-memory server shares nothing with the old one — it starts empty.
    let s2 = Server::start(ServerConfig::default()).await;
    assert_eq!(
        count_rows(&cl, &s2.base, "SELECT * WHERE { ?s ?p ?o }").await,
        0,
        "an in-memory server (no --persist) must NOT see a previous server's updates"
    );
    s2.stop().await;
}

// ---------------------------------------------------------------------------
// A pre-existing on-disk store is the source of truth: the DATA_FILE seed is ignored.
// ---------------------------------------------------------------------------

/// When the persist dir already holds a store, opening it must take that store as-is and
/// ignore the seed graph (QLever's persisted-index-wins semantics). We prove it by seeding a
/// dir via one server's updates, then booting a second server on the same dir with a NON-empty
/// seed graph — the seed must NOT appear; only the persisted data does.
#[tokio::test]
async fn existing_store_wins_over_seed() {
    let scratch = ScratchDir::new();
    let cl = reqwest::Client::new();

    // Establish a persisted store with exactly one triple.
    let s1 = Server::start(persist_config(scratch.path())).await;
    assert_eq!(
        post_update(&cl, &s1.base, "INSERT DATA { <http://ex/keep> <http://ex/p> <http://ex/v> }").await,
        204
    );
    s1.stop().await;

    // Reopen with a NON-empty seed graph; the existing store must win (seed ignored).
    let seed = Graph::load_str("<http://ex/seed> <http://ex/p> <http://ex/v> .", "ntriples").unwrap();
    let state = AppState::try_with_config(seed, persist_config(scratch.path())).expect("reopen");
    let app = router(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        axum::serve(listener, app).with_graceful_shutdown(async { let _ = rx.await; }).await.unwrap();
    });
    let base = format!("http://{addr}");

    assert_eq!(
        count_rows(&cl, &base, "SELECT * WHERE { <http://ex/keep> ?p ?o }").await,
        1,
        "the persisted triple must be present"
    );
    assert_eq!(
        count_rows(&cl, &base, "SELECT * WHERE { <http://ex/seed> ?p ?o }").await,
        0,
        "the seed graph must be IGNORED when the persist dir already holds a store"
    );

    let _ = tx.send(());
    let _ = task.await;
}
