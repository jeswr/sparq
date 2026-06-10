//! Integration tests for the double-buffered SPARQL Update path (T17 wiring).
//!
//! Covers the properties the design promises (see `src/http.rs::Writer` and the README's
//! "Update concurrency model"):
//!   * sequential consistency over HTTP — every committed update is visible to the next
//!     query, including updates whose effects reach the published graph via LAG REPLAY
//!     on the other buffer;
//!   * atomicity of failures — a refused update leaves the published graph untouched and
//!     the server recovers (the discarded buffer is rebuilt by the next update);
//!   * update latency — steady-state in-place updates are far cheaper than the first
//!     (rebuild) update on the same graph;
//!   * reader isolation — queries never block on an in-flight update beyond the publish
//!     pointer-swap, even while the writer is doing O(graph) work;
//!   * periodic compaction — overlay fold-back keeps results identical.
//!
//! The `#[ignore]`d benchmark at the bottom is the honest before/after measurement on a
//! 1M-triple graph:
//!   cargo test -p sparq-server --release --test updates -- --ignored --nocapture

use std::time::{Duration, Instant};

use sparq_core::dict::Dict;
use sparq_core::Graph;
use sparq_server::{router, AppState, ServerConfig};
use tokio::net::TcpListener;

/// A synthetic graph of `n` distinct triples (10 predicates, n/10 subjects), built
/// directly from interned ids — cheap enough for debug-mode tests.
fn synthetic_graph(n: usize) -> Graph {
    let nsubj = (n / 10).max(1);
    let mut dict = Dict::new();
    let preds: Vec<_> = (0..10).map(|p| dict.intern_iri(&format!("http://ex/p{p}"))).collect();
    let subjs: Vec<_> = (0..nsubj).map(|s| dict.intern_iri(&format!("http://ex/s{s}"))).collect();
    // Murmur-style mix so object indexes don't collapse under set dedup.
    let mix = |i: usize| -> usize {
        let mut h = i as u64;
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
        h ^= h >> 33;
        (h % nsubj as u64) as usize
    };
    let triples: Vec<_> = (0..n).map(|i| [subjs[i % nsubj], preds[i % 10], subjs[mix(i)]]).collect();
    Graph::from_parts(dict, triples)
}

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

/// Counts the solutions of `query` by counting binding objects in the JSON result.
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

// ---------------------------------------------------------------------------
// Sequential consistency + lag replay
// ---------------------------------------------------------------------------

/// Every update in a sequence must be visible to the immediately following query. With
/// the double buffer, update k is applied to one buffer directly and reaches the other
/// buffer via lag replay during update k+1 — interleaving queries after every update
/// exercises both lines (a replay bug would drop an earlier update from every second
/// published graph).
#[tokio::test]
async fn sequential_updates_are_all_visible() {
    let base = spawn_with(synthetic_graph(1_000), ServerConfig::default()).await;
    let cl = reqwest::Client::new();
    for i in 0..12 {
        let ins = format!("INSERT DATA {{ <http://ex/u{i}> <http://ex/seen> <http://ex/yes> }}");
        assert_eq!(post_update(&cl, &base, &ins).await, 204);
        let n = count_rows(&cl, &base, "SELECT ?s WHERE { ?s <http://ex/seen> ?o }").await;
        assert_eq!(n, i + 1, "update {i} (or an earlier one lost in lag replay) is not visible");
    }
    // Deletes flow through the same path.
    assert_eq!(
        post_update(&cl, &base, "DELETE DATA { <http://ex/u0> <http://ex/seen> <http://ex/yes> }").await,
        204
    );
    assert_eq!(count_rows(&cl, &base, "SELECT ?s WHERE { ?s <http://ex/seen> ?o }").await, 11);
}

// ---------------------------------------------------------------------------
// Failed updates: atomic + recoverable
// ---------------------------------------------------------------------------

/// A refused update (unsupported operation) must leave the published graph untouched —
/// including when the refusal happens after a *prefix* of the request already applied to
/// the spare buffer — and the server must keep accepting updates afterwards.
#[tokio::test]
async fn failed_update_is_atomic_and_server_recovers() {
    let base = spawn_with(synthetic_graph(1_000), ServerConfig::default()).await;
    let cl = reqwest::Client::new();

    // Two good updates first, so the double buffer is in steady state.
    assert_eq!(post_update(&cl, &base, "INSERT DATA { <http://ex/a> <http://ex/q> <http://ex/b> }").await, 204);
    assert_eq!(post_update(&cl, &base, "INSERT DATA { <http://ex/c> <http://ex/q> <http://ex/d> }").await, 204);

    // A request whose FIRST operation succeeds and SECOND fails at execution (LOAD from a
    // nonexistent file — named-graph ops themselves are supported since conformance round 3):
    // must be a 400 with no partial effect visible.
    let partial = "INSERT DATA { <http://ex/leak> <http://ex/q> <http://ex/leak> } ; \
                   LOAD <file:///nonexistent/sparq-test-no-such-file.nt>";
    assert_eq!(post_update(&cl, &base, partial).await, 400);
    assert_eq!(
        count_rows(&cl, &base, "SELECT ?s WHERE { <http://ex/leak> <http://ex/q> ?s }").await,
        0,
        "a refused update leaked a partial application into the published graph"
    );
    assert_eq!(count_rows(&cl, &base, "SELECT ?s WHERE { ?s <http://ex/q> ?o }").await, 2);

    // The discarded buffer is rebuilt transparently: further updates still work.
    assert_eq!(post_update(&cl, &base, "INSERT DATA { <http://ex/e> <http://ex/q> <http://ex/f> }").await, 204);
    assert_eq!(post_update(&cl, &base, "INSERT DATA { <http://ex/g> <http://ex/q> <http://ex/h> }").await, 204);
    assert_eq!(count_rows(&cl, &base, "SELECT ?s WHERE { ?s <http://ex/q> ?o }").await, 4);
}

// ---------------------------------------------------------------------------
// Update latency smoke test
// ---------------------------------------------------------------------------

/// Steady-state updates take the in-place delta-overlay path, so they must be far
/// cheaper than the first update on the same state, which pays the O(graph) rebuild
/// (it materialises the second buffer — exactly the cost EVERY update used to pay).
/// The bound is deliberately loose (5x; the release-mode gap is ~5 orders of magnitude
/// on 1M triples — see the ignored benchmark) so machine noise can't flake it.
#[tokio::test]
async fn steady_state_update_latency_is_far_below_rebuild() {
    let n = 100_000;
    let base = spawn_with(synthetic_graph(n), ServerConfig::default()).await;
    let cl = reqwest::Client::new();

    let t = Instant::now();
    assert_eq!(post_update(&cl, &base, "INSERT DATA { <http://ex/n0> <http://ex/q> <http://ex/m0> }").await, 204);
    let first = t.elapsed(); // bootstrap: the old rebuild-per-update cost

    let mut steady = Vec::new();
    for i in 1..=20 {
        let ins = format!("INSERT DATA {{ <http://ex/n{i}> <http://ex/q> <http://ex/m{i}> }}");
        let t = Instant::now();
        assert_eq!(post_update(&cl, &base, &ins).await, 204);
        steady.push(t.elapsed());
    }
    steady.sort();
    let median = steady[steady.len() / 2];
    assert_eq!(count_rows(&cl, &base, "SELECT ?s WHERE { ?s <http://ex/q> ?o }").await, 21);
    assert!(
        median * 5 < first,
        "steady-state update (median {median:?}) should be much cheaper than the rebuild bootstrap ({first:?})"
    );
}

// ---------------------------------------------------------------------------
// Readers never block on the writer
// ---------------------------------------------------------------------------

/// While an update is doing O(graph) work (here: the bootstrap rebuild), readers must
/// keep taking snapshots and answering queries — they may only contend for the instant
/// of the publish pointer-swap. Each observed count must also be one of the two
/// committed states (snapshot consistency: old or new, never partial).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn readers_are_not_blocked_during_a_slow_update() {
    let n = 200_000;
    let state = AppState::with_config(synthetic_graph(n), ServerConfig::default());
    let probe = "SELECT ?o WHERE { <http://ex/w> <http://ex/w> ?o }";
    let before = sparq_engine::query(&state.snapshot(), probe).unwrap().rows.len();
    assert_eq!(before, 0);

    let writer_state = state.clone();
    let writer = tokio::task::spawn_blocking(move || {
        let t = Instant::now();
        writer_state
            .apply_update("INSERT DATA { <http://ex/w> <http://ex/w> <http://ex/w> }")
            .unwrap();
        t.elapsed()
    });

    // Sample reader latency until the update commits (and a little after).
    let mut max_read = Duration::ZERO;
    let mut samples = 0u32;
    let mut committed = false;
    let deadline = Instant::now() + Duration::from_secs(60);
    while !committed && Instant::now() < deadline {
        let t = Instant::now();
        let snap = state.snapshot();
        let rows = sparq_engine::query(&snap, probe).unwrap().rows.len();
        let dt = t.elapsed();
        max_read = max_read.max(dt);
        samples += 1;
        assert!(rows == 0 || rows == 1, "reader saw a state that was never committed: {rows} rows");
        committed = rows == 1;
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    let update_took = writer.await.unwrap();
    assert!(committed, "update did not become visible within 60s");
    assert!(samples > 3, "sampler barely ran ({samples} samples) — cannot judge blocking");
    // The write lock is held only for a pointer swap; 250ms is an enormous margin over
    // the microseconds it actually takes, while the rebuild itself takes much longer.
    assert!(
        max_read < Duration::from_millis(250),
        "a reader stalled for {max_read:?} during an update that took {update_took:?}"
    );
}

// ---------------------------------------------------------------------------
// Periodic compaction
// ---------------------------------------------------------------------------

/// With `compact_every = 1` every published graph has had its overlay folded back into
/// the immutable base; with `0` the overlay persists. Results must be identical either
/// way — compaction is purely an internal representation change.
#[test]
fn compaction_folds_the_overlay_without_changing_results() {
    for (compact_every, expect_overlay) in [(1usize, false), (0usize, true)] {
        let config = ServerConfig { compact_every, ..ServerConfig::default() };
        let state = AppState::with_config(synthetic_graph(1_000), config);
        for i in 0..6 {
            let upd = format!("INSERT DATA {{ <http://ex/k{i}> <http://ex/kp> <http://ex/kv> }}");
            state.apply_update(&upd).unwrap_or_else(|e| panic!("{e}"));
        }
        let snap = state.snapshot();
        // The first update is a rebuild (no overlay); from the second on, the in-place
        // path leaves an overlay unless compaction folded it.
        assert_eq!(
            snap.store.has_overlay(),
            expect_overlay,
            "unexpected overlay state for compact_every = {compact_every}"
        );
        let rows = sparq_engine::query(&snap, "SELECT ?s WHERE { ?s <http://ex/kp> ?o }").unwrap();
        assert_eq!(rows.rows.len(), 6, "compact_every = {compact_every} changed query results");
    }
}

// ---------------------------------------------------------------------------
// Honest before/after benchmark (1M triples) — run explicitly in release mode
// ---------------------------------------------------------------------------

/// End-to-end HTTP update latency on a 1M-triple graph.
///   BEFORE = the rebuild path every update used to take (measured both as the engine
///            call and end-to-end as the first update, which still pays it once);
///   AFTER  = steady-state in-place updates over HTTP.
/// Run: cargo test -p sparq-server --release --test updates -- --ignored --nocapture
#[tokio::test]
#[ignore = "benchmark — run explicitly in --release"]
async fn bench_update_latency_1m() {
    let n: usize = std::env::var("SPARQ_BENCH_TRIPLES").ok().and_then(|v| v.parse().ok()).unwrap_or(1_000_000);
    let t = Instant::now();
    let graph = synthetic_graph(n);
    eprintln!("graph: {} triples, built in {:.2?}", graph.len(), t.elapsed());

    // BEFORE (engine-level): the rebuild `update` the server used to call per request.
    let ins = "INSERT DATA { <http://ex/b0> <http://ex/q> <http://ex/b0> }";
    let t = Instant::now();
    let rebuilt = sparq_engine::update(&graph, ins).unwrap();
    let before_engine = t.elapsed();
    eprintln!("BEFORE  rebuild update (engine call):        {before_engine:.2?}");
    drop(rebuilt);

    let base = spawn_with(graph, ServerConfig::default()).await;
    let cl = reqwest::Client::new();

    // BEFORE (end-to-end): the first update still pays exactly the old per-request cost
    // (it doubles as the materialisation of the second buffer).
    let t = Instant::now();
    assert_eq!(post_update(&cl, &base, ins).await, 204);
    let before_http = t.elapsed();
    eprintln!("BEFORE  rebuild update (HTTP, = 1st update): {before_http:.2?}");

    // AFTER: steady-state in-place updates, end-to-end over HTTP.
    let reps = 200;
    let mut lat = Vec::with_capacity(reps);
    for i in 0..reps {
        let upd = format!("INSERT DATA {{ <http://ex/a{i}> <http://ex/q> <http://ex/a{i}> }}");
        let t = Instant::now();
        assert_eq!(post_update(&cl, &base, &upd).await, 204);
        lat.push(t.elapsed());
    }
    lat.sort();
    let median = lat[reps / 2];
    let p99 = lat[reps * 99 / 100];
    let mean: Duration = lat.iter().sum::<Duration>() / reps as u32;
    eprintln!("AFTER   in-place update (HTTP, {reps} reps):  median {median:.2?}  mean {mean:.2?}  p99 {p99:.2?}");
    eprintln!(
        "end-to-end speedup: {:.0}x (median) over the first/rebuild update",
        before_http.as_secs_f64() / median.as_secs_f64()
    );
}
