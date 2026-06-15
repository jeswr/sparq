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
    ///
    /// [OPUS-4.8] (Copilot PR#80) We `expect()` the join result rather than discarding it: if
    /// the serve task PANICKED (e.g. the durable writer thread fail-closed and the panic
    /// propagated, or `axum::serve(...).await.unwrap()` hit a serve error), `task.await` returns
    /// `Err(JoinError)`, and swallowing it would let the test pass while masking a real failure.
    async fn stop(mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(task) = self.task.take() {
            task.await.expect("server serve task panicked / failed");
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

/// [OPUS-4.8] (sq-ycle) The boolean answer to an `ASK` query (the SPARQL JSON `boolean` field).
async fn ask(cl: &reqwest::Client, base: &str, query: &str) -> bool {
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
    body["boolean"].as_bool().unwrap()
}

/// The `value` field of the single binding of variable `var` for a one-row `query` (asserts
/// exactly one row). Used to read back a non-deterministically-generated literal.
async fn single_value(cl: &reqwest::Client, base: &str, query: &str, var: &str) -> String {
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
    let bindings = body["results"]["bindings"].as_array().unwrap();
    assert_eq!(bindings.len(), 1, "expected exactly one binding for ?{var}");
    bindings[0][var]["value"].as_str().unwrap().to_string()
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
// [OPUS-4.8] (Copilot PR#80) Non-deterministic update: the durable graph must persist the
// EXACT value the in-memory side committed and acked — NOT a re-rolled one.
// ---------------------------------------------------------------------------

/// REGRESSION for the durability-divergence finding (Copilot PR#80). A `DELETE/INSERT … WHERE`
/// that binds a non-deterministic value (`STRUUID()`) commits one resolved literal in memory and
/// 204-acks it. The durable mirror must persist *that* literal, not a second, independently-rolled
/// one — otherwise a restart would surface a value the client never saw, breaking "204 ⇒ durable".
///
/// With the old "re-execute the update string against the durable graph" mirror this FAILED: the
/// second execution re-rolled `STRUUID()`, so after a restart the persisted value differed from the
/// value the live server returned. We assert byte-equality of the value before and after the restart.
#[tokio::test]
async fn nondeterministic_update_persists_committed_value_not_a_reroll() {
    let scratch = ScratchDir::new();
    let cl = reqwest::Client::new();

    let s1 = Server::start(persist_config(scratch.path())).await;
    // Seed a subject, then tag it with a freshly-generated STRUUID via DELETE/INSERT … WHERE.
    assert_eq!(
        post_update(&cl, &s1.base, "INSERT DATA { <http://ex/s> <http://ex/p> <http://ex/o> }").await,
        204
    );
    assert_eq!(
        post_update(
            &cl,
            &s1.base,
            "INSERT { <http://ex/s> <http://ex/tag> ?u } WHERE { <http://ex/s> <http://ex/p> ?o . BIND(STRUUID() AS ?u) }",
        )
        .await,
        204
    );

    // The value the LIVE (in-memory, already-acked) server holds.
    let live = single_value(&cl, &s1.base, "SELECT ?u WHERE { <http://ex/s> <http://ex/tag> ?u }", "u").await;
    assert!(!live.is_empty(), "STRUUID() must have produced a value");

    // RESTART and read the persisted value: it MUST be the identical literal (not a re-roll).
    s1.stop().await;
    let s2 = Server::start(persist_config(scratch.path())).await;
    let persisted =
        single_value(&cl, &s2.base, "SELECT ?u WHERE { <http://ex/s> <http://ex/tag> ?u }", "u").await;
    assert_eq!(
        persisted, live,
        "the persisted STRUUID() value must equal the value the live server acked — \
         the durable mirror must NOT re-execute (and thus re-roll) the update"
    );
    s2.stop().await;
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] (sq-ycle, gh-48) PSS combined multi-op update body: accepted as ONE request
// AND committed ATOMICALLY (all-or-nothing) on the durable `--persist` path.
// ---------------------------------------------------------------------------

/// gh-48 (a)+(b). PSS's `putDocument`/`deleteResource` send ONE request body of `;`-separated
/// operations — `DROP SILENT GRAPH <r> ; INSERT DATA { GRAPH <r> … ; GRAPH <parent> ldp:contains
/// <r> }` — that rewrites the resource graph AND the parent's `ldp:contains` containment in a
/// single shot. PSS's reconciler does NOT repair index-internal containment desync, so this body
/// MUST be (a) accepted as one update request and (b) applied atomically: a single 204, with the
/// child graph AND the parent containment triple BOTH present afterwards (never one without the
/// other). This test sends PSS's exact shape, asserts the single 204 + the fully-applied post-state,
/// then RESTARTS the `--persist` server to prove the WHOLE body is durable as one atomic commit
/// (the [`sparq_engine`] txn-journal commit point, sq-ycle) — a crash mid-body can never leave the
/// parent containment pointing at a child graph that did not survive (or vice versa).
#[tokio::test]
async fn pss_combined_multiop_body_accepted_and_atomic() {
    const LDP_CONTAINS: &str = "http://www.w3.org/ns/ldp#contains";
    const R: &str = "http://ex/pod/resource1";
    const PARENT: &str = "http://ex/pod/";

    let scratch = ScratchDir::new();
    let cl = reqwest::Client::new();
    let s1 = Server::start(persist_config(scratch.path())).await;

    // The EXACT PSS combined body: DROP the resource graph, then in ONE INSERT DATA write the new
    // resource content into GRAPH <r> AND the parent containment triple into GRAPH <parent>.
    let body = format!(
        "DROP SILENT GRAPH <{R}> ; \
         INSERT DATA {{ \
           GRAPH <{R}> {{ <{R}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/ldp#Resource> . \
                          <{R}> <http://ex/title> \"hello\" }} \
           GRAPH <{PARENT}> {{ <{PARENT}> <{LDP_CONTAINS}> <{R}> }} \
         }}"
    );

    // (a) ACCEPTED: the `;`-separated multi-op body is one update request → a SINGLE 204.
    assert_eq!(post_update(&cl, &s1.base, &body).await, 204, "the combined multi-op body must be accepted as ONE update (single 204)");

    // (b) FULLY APPLIED: both halves of the body are present — the child graph content AND the
    // parent containment triple. (No partial write: containment without the resource, or vice versa.)
    let child = format!("SELECT * WHERE {{ GRAPH <{R}> {{ ?s ?p ?o }} }}");
    let containment = format!("ASK WHERE {{ GRAPH <{PARENT}> {{ <{PARENT}> <{LDP_CONTAINS}> <{R}> }} }}");
    assert_eq!(count_rows(&cl, &s1.base, &child).await, 2, "the child resource graph must be fully written");
    assert!(ask(&cl, &s1.base, &containment).await, "the parent ldp:contains triple must be present");

    // The whole body is ONE atomic durable commit: a restart re-opens both halves together.
    s1.stop().await;
    let s2 = Server::start(persist_config(scratch.path())).await;
    assert_eq!(count_rows(&cl, &s2.base, &child).await, 2, "the child graph must survive the restart (durable)");
    assert!(ask(&cl, &s2.base, &containment).await, "the parent containment must survive the restart, in lockstep with the child graph");

    // A second putDocument-shaped body REPLACES the resource graph atomically and is durable too.
    let body2 = format!(
        "DROP SILENT GRAPH <{R}> ; \
         INSERT DATA {{ \
           GRAPH <{R}> {{ <{R}> <http://ex/title> \"updated\" }} \
           GRAPH <{PARENT}> {{ <{PARENT}> <{LDP_CONTAINS}> <{R}> }} \
         }}"
    );
    assert_eq!(post_update(&cl, &s2.base, &body2).await, 204);
    assert_eq!(count_rows(&cl, &s2.base, &child).await, 1, "the DROP+re-INSERT replaced the resource graph atomically (old content gone)");
    s2.stop().await;
    let s3 = Server::start(persist_config(scratch.path())).await;
    assert_eq!(count_rows(&cl, &s3.base, &child).await, 1, "the replacement survives a second restart");
    assert!(ask(&cl, &s3.base, &containment).await);
    s3.stop().await;
}

/// gh-48 all-or-nothing. A multi-op request whose LATER operation is invalid must leave NO
/// partial write — the request fails (non-2xx) and the valid prefix is NOT committed (not in
/// memory, and — on `--persist` — not on disk after a restart). This is the engine's
/// request-level atomicity: the serve writer applies the body to a private fork and seals
/// (and durably commits) ONLY on full success, so a rejected body never publishes or persists
/// a prefix. (A non-SILENT `LOAD` of an unfetchable source is the reliable mid-body failure.)
#[tokio::test]
async fn invalid_second_op_leaves_no_partial_write() {
    let scratch = ScratchDir::new();
    let cl = reqwest::Client::new();
    let s1 = Server::start(persist_config(scratch.path())).await;

    // op 1 (valid INSERT DATA) ; op 2 (a non-SILENT LOAD that the engine refuses → request error).
    let failing = "INSERT DATA { <http://ex/a> <http://ex/p> <http://ex/b> } ; LOAD <http://ex/nope.ttl>";
    let status = post_update(&cl, &s1.base, failing).await;
    assert!(!status.is_success(), "a multi-op body with an invalid op must be rejected (non-2xx), got {status}");

    // The valid prefix must NOT have committed in memory (all-or-nothing).
    assert_eq!(
        count_rows(&cl, &s1.base, "SELECT * WHERE { <http://ex/a> ?p ?o }").await,
        0,
        "the valid INSERT DATA prefix must NOT be visible after a rejected multi-op request"
    );

    // …and nothing of it reached the durable store either: a restart shows an empty store.
    s1.stop().await;
    let s2 = Server::start(persist_config(scratch.path())).await;
    assert_eq!(
        count_rows(&cl, &s2.base, "SELECT * WHERE { ?s ?p ?o }").await,
        0,
        "a rejected multi-op request must persist NOTHING (no partial durable write)"
    );
    s2.stop().await;
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
    // [OPUS-4.8] (Copilot PR#80) Fail the test on a panicked/failed serve task rather than
    // discarding the join result (which would silently mask a server-side failure).
    task.await.expect("server serve task panicked / failed");
}
