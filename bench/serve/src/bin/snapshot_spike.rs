//! RESEARCH SPIKE — snapshot cost + retention semantics (research/concurrent-serving.md §d.iii).
//!
//! Exercises the PRODUCTION snapshot mechanism (`sparq_server::AppState`): readers take
//! `Arc<Graph>` clones of the published generation; updates go through the
//! double-buffered writer (in-place delta on the reclaimed spare, atomic publish swap).
//!
//! Measures, on a real dataset (pass the .nt path; defaults to a synthetic graph):
//!   1. snapshot (`Arc` clone) cost — the per-query price of consistency;
//!   2. steady-state in-place update latency with NO snapshot held;
//!   3. update latency while a reader HOLDS the previous generation — the writer
//!      cannot reclaim its spare, waits out `query_timeout + 2s grace`, then falls
//!      back to the O(graph) rebuild: this is the cost a long-lived STREAM imposes
//!      on the write path today, the central number for the streaming design;
//!   4. RSS at each stage (1 generation, 2 generations pinned) — the memory price
//!      of one retained snapshot generation.
//!
//! Honest scope: this spike uses a 1s query-timeout so step 3 completes quickly; the
//! production default is 30s, i.e. the stall is proportionally worse there.

use std::sync::Arc;
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
        compact_every: 1024,
        ..ServerConfig::default()
    };
    let state = AppState::with_config(graph, config);

    // 1. snapshot cost — the Arc clone under the read lock.
    let iters = 10_000_000u64;
    let t = Instant::now();
    for _ in 0..iters {
        std::hint::black_box(state.snapshot());
    }
    println!(
        "snapshot (RwLock read + Arc clone): {:.1} ns/op — {:.1} M snapshots/s single thread",
        t.elapsed().as_nanos() as f64 / iters as f64,
        iters as f64 / t.elapsed().as_secs_f64() / 1e6
    );

    // 2. update latencies, no snapshot held.
    let upd = |i: usize| format!("INSERT DATA {{ <http://ex/new{i}> <http://ex/p0> \"fresh{i}\" }}");
    let t = Instant::now();
    state.apply_update(&upd(0)).expect("first update");
    println!("update #1 (first => O(graph) rebuild + buffer materialisation): {:.2}s", t.elapsed().as_secs_f64());
    println!("RSS after first update (two buffers live): {:.0} MB", rss_mb());

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
    let pinned: Arc<Graph> = state.snapshot(); // pins the currently-published generation
    let t = Instant::now();
    state.apply_update(&upd(100)).expect("update; pinned gen becomes spare");
    println!("update with snapshot pinned, gen N->spare demotion (still in-place): {:.3}s", t.elapsed().as_secs_f64());
    let t = Instant::now();
    state.apply_update(&upd(101)).expect("update that needs the pinned spare");
    println!(
        "update needing the PINNED spare: {:.3}s  (= reclaim wait [timeout {}s + 2s grace] + O(graph) rebuild)",
        t.elapsed().as_secs_f64(),
        1
    );
    println!("RSS with one pinned generation + rebuilt published + spare: {:.0} MB", rss_mb());
    println!("pinned snapshot still answers at its generation: {} triples (published now has more)", pinned.len());
    drop(pinned);

    // 4. recovery after the stream ends.
    let mut after = Vec::new();
    for i in 200..210 {
        let t = Instant::now();
        state.apply_update(&upd(i)).expect("post-drop update");
        after.push(t.elapsed().as_micros() as u64);
    }
    after.sort_unstable();
    println!("updates after snapshot dropped: p50={}us max={}us (first may rebuild once)", after[after.len() / 2], after.last().unwrap());
    println!("final RSS: {:.0} MB", rss_mb());
}
