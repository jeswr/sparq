//! RESEARCH SPIKE — snapshot cost + retention semantics (research/concurrent-serving.md §d.iii).
//!
//! Exercises the PRODUCTION snapshot mechanism (`sparq_server::AppState`). As of Wave A4
//! that mechanism is the [`sparq_serve::GenerationRing`]: readers pin the current
//! generation via [`AppState::current`] (lock-free `ArcSwap::load_full`) and read its
//! immutable snapshot; updates group-commit through the single sequenced
//! [`sparq_serve::Writer`], which publishes each batch as ONE new generation
//! (structural fork, in-place delta, atomic ring swap). The ring REPLACED the
//! double-buffered `RwLock<Arc<Graph>>` writer this spike was originally written for —
//! the part of the design this spike now demonstrates as FIXED is exactly step 3.
//!
//! Measures, on a real dataset (pass the .nt path; defaults to a synthetic graph):
//!   1. snapshot cost — the lock-free `current()` pin, the per-query price of consistency;
//!   2. steady-state in-place update latency with NO snapshot held;
//!   3. update latency while a reader HOLDS the previous generation. Under the OLD
//!      double-buffer this stalled the writer (it could not reclaim its pinned spare,
//!      waited out `query_timeout + 2s grace`, then fell back to an O(graph) rebuild) —
//!      the §4.3/§4.4 pathology. Under the ring a pinned generation is just one of K
//!      retained `Arc`s and is NEVER reclaimed by the writer, so the writer publishes a
//!      fresh generation regardless: this step now measures that the long-lived-stream
//!      stall is GONE (latency should match step 2), the central result for the design;
//!   4. RSS at each stage (1 generation, 2 generations pinned) — the memory price of one
//!      retained snapshot generation (the ring's bounded-retention cost).
//!
//! Honest scope: the 1s query-timeout is now immaterial to step 3 (there is no reclaim
//! wait to bound), kept only so the budget matches the original run; it no longer makes
//! the stall "proportionally worse" because there is no stall.

use std::time::{Duration, Instant};

use sparq_core::Graph;
use sparq_server::{AppState, ServerConfig};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn rss_mb() -> f64 {
    let out = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .expect("ps");
    String::from_utf8_lossy(&out.stdout).trim().parse::<f64>().unwrap_or(0.0) / 1024.0
}

fn main() {
    let path = std::env::args().nth(1);
    let t = Instant::now();
    let graph = match &path {
        Some(p) => {
            eprintln!("loading {p} ...");
            Graph::load_str(&std::fs::read_to_string(p).expect("read data"), "ntriples").expect("load")
        }
        None => {
            let mut nt = String::new();
            for i in 0..1_000_000 {
                nt.push_str(&format!(
                    "<http://ex/s{}> <http://ex/p{}> \"v{}\" .\n",
                    i % 200_000,
                    i % 50,
                    i
                ));
            }
            Graph::load_str(&nt, "ntriples").expect("load synthetic")
        }
    };
    let triples = graph.len();
    println!("loaded {triples} triples in {:.1}s; RSS after load: {:.0} MB", t.elapsed().as_secs_f64(), rss_mb());

    let config = ServerConfig {
        query_timeout: Some(Duration::from_secs(1)), // keeps step 3 short; default is 30s
        ..ServerConfig::default()
    };
    let state = AppState::with_config(graph, config);

    // 1. snapshot cost — pinning the current generation (lock-free `ArcSwap::load_full`).
    let iters = 10_000_000u64;
    let t = Instant::now();
    for _ in 0..iters {
        std::hint::black_box(state.current());
    }
    println!(
        "snapshot (lock-free current(): ArcSwap load + Arc clone): {:.1} ns/op — {:.1} M snapshots/s single thread",
        t.elapsed().as_nanos() as f64 / iters as f64,
        iters as f64 / t.elapsed().as_secs_f64() / 1e6
    );

    // 2. update latencies, no snapshot held.
    let upd = |i: usize| format!("INSERT DATA {{ <http://ex/new{i}> <http://ex/p0> \"fresh{i}\" }}");
    let t = Instant::now();
    state.apply_update(&upd(0)).expect("first update");
    println!("update #1 (first => one-time O(graph) dictionary/cache freeze to mint the shared fork base): {:.2}s", t.elapsed().as_secs_f64());
    println!("RSS after first update (base + first forked generation live): {:.0} MB", rss_mb());

    let mut steady = Vec::new();
    for i in 1..21 {
        let t = Instant::now();
        state.apply_update(&upd(i)).expect("steady update");
        steady.push(t.elapsed().as_micros() as u64);
    }
    steady.sort_unstable();
    println!(
        "updates #2-#21 (steady in-place): p50={}us max={}us",
        steady[steady.len() / 2],
        steady.last().unwrap()
    );

    // 3. update while a reader pins the PREVIOUS generation (a long-lived stream).
    // Under the ring the pinned generation is just one of K retained Arcs — the writer
    // never has to reclaim it, so neither of these updates stalls (contrast the old
    // double-buffer, where the writer waited out timeout+grace then rebuilt O(graph)).
    let pinned = state.current(); // pins the currently-published generation
    let t = Instant::now();
    state.apply_update(&upd(100)).expect("update while previous gen pinned");
    println!("update #1 with a generation pinned (writer publishes a fresh generation, no reclaim): {:.3}s", t.elapsed().as_secs_f64());
    let t = Instant::now();
    state.apply_update(&upd(101)).expect("update while previous gen still pinned");
    println!(
        "update #2 with a generation still pinned: {:.3}s  (ring: NO reclaim wait, NO O(graph) rebuild — should match step 2 steady-state)",
        t.elapsed().as_secs_f64()
    );
    println!("RSS with the pinned generation + the newer published generations live: {:.0} MB", rss_mb());
    println!("pinned snapshot still answers at its generation: {} triples (published now has more)", pinned.snapshot().len());
    drop(pinned);

    // 4. after the stream ends: the pinned generation has dropped, so the ring holds one
    // fewer retained Arc. There is no reclaim path to "recover", so latency is unchanged
    // from steps 2/3 — recorded here to confirm exactly that (no recovery transient).
    let mut after = Vec::new();
    for i in 200..210 {
        let t = Instant::now();
        state.apply_update(&upd(i)).expect("post-drop update");
        after.push(t.elapsed().as_micros() as u64);
    }
    after.sort_unstable();
    println!("updates after snapshot dropped: p50={}us max={}us (no recovery transient under the ring)", after[after.len() / 2], after.last().unwrap());
    println!("final RSS: {:.0} MB", rss_mb());
}
