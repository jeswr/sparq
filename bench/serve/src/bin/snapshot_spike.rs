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
//!
//! Run: `cargo run --release --bin snapshot_spike [data.nt]` in bench/serve.
//!   cargo run --release --bin snapshot_spike -- [data.nt] --json /tmp/snap.json  # strictly-additive
//!
//! [OPUS-4.8] (sq-5vm.1) `--json <path>` writes the SAME snapshot/update/RSS figures STDOUT
//! prints as a stable, DEPENDENCY-FREE JSON document, mirroring writer_spike's emit (sq-k5qq).
//! No serde dep is added to the harness (serde_json is a TEST-only dev-dep that parses the emit
//! back). STDOUT is byte-for-byte unchanged whether or not the flag is present; every number is
//! best-effort, MEASURED on the running host — ADVISORY + NON-CANONICAL (stated in the emitted
//! `note`) — nothing is committed.

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

// ---------------------------------------------------------------------------
// [OPUS-4.8] (sq-5vm.1) --json <path> machine-readable results emit
// ---------------------------------------------------------------------------

/// All the figures this spike prints, captured once so STDOUT and the optional JSON
/// emit share a single source of truth.
#[derive(Default)]
struct Results {
    triples: usize,
    load_secs: f64,
    rss_after_load_mb: f64,
    snapshot_ns_per_op: f64,
    snapshots_per_s_millions: f64,
    first_update_secs: f64,
    rss_after_first_update_mb: f64,
    steady_p50_us: u64,
    steady_max_us: u64,
    pinned_update1_secs: f64,
    pinned_update2_secs: f64,
    rss_with_pinned_mb: f64,
    pinned_snapshot_triples: usize,
    after_drop_p50_us: u64,
    after_drop_max_us: u64,
    final_rss_mb: f64,
}

/// Extracts `--json <path>` from argv, returning argv WITHOUT the flag pair. A bare
/// `--json` is a usage error (exit 2), mirroring `sparq-cli` / `writer_spike`'s flag.
fn take_json_flag(args: Vec<String>) -> (Vec<String>, Option<String>) {
    let mut out = Vec::with_capacity(args.len());
    let mut json_path = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--json" {
            match args.get(i + 1) {
                Some(p) => {
                    json_path = Some(p.clone());
                    i += 2;
                    continue;
                }
                None => {
                    eprintln!("`--json` requires a path argument: --json <path>");
                    std::process::exit(2);
                }
            }
        }
        out.push(args[i].clone());
        i += 1;
    }
    (out, json_path)
}

/// Serialise the run to stable, dependency-free JSON. Every metric is ADVISORY +
/// NON-CANONICAL (stated in `note`).
fn results_json(r: &Results) -> String {
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str("  \"harness\": \"serve-spikes snapshot_spike\",\n");
    s.push_str(
        "  \"note\": \"snapshot cost + generation-ring retention semantics; every ns/op, \
         update-latency, M-snapshots/s and RSS figure is best-effort, MEASURED on the running \
         host — ADVISORY, NON-CANONICAL (this dev box) — do not bake into committed files\",\n",
    );
    s.push_str(&format!("  \"triples\": {},\n", r.triples));
    s.push_str(&format!("  \"load_secs\": {:.3},\n", r.load_secs));
    s.push_str(&format!("  \"rss_after_load_mb\": {:.0},\n", r.rss_after_load_mb));
    s.push_str(&format!("  \"snapshot_ns_per_op\": {:.3},\n", r.snapshot_ns_per_op));
    s.push_str(&format!(
        "  \"snapshots_per_s_millions\": {:.3},\n",
        r.snapshots_per_s_millions
    ));
    s.push_str(&format!("  \"first_update_secs\": {:.3},\n", r.first_update_secs));
    s.push_str(&format!(
        "  \"rss_after_first_update_mb\": {:.0},\n",
        r.rss_after_first_update_mb
    ));
    s.push_str(&format!("  \"steady_p50_us\": {},\n", r.steady_p50_us));
    s.push_str(&format!("  \"steady_max_us\": {},\n", r.steady_max_us));
    s.push_str(&format!("  \"pinned_update1_secs\": {:.3},\n", r.pinned_update1_secs));
    s.push_str(&format!("  \"pinned_update2_secs\": {:.3},\n", r.pinned_update2_secs));
    s.push_str(&format!("  \"rss_with_pinned_mb\": {:.0},\n", r.rss_with_pinned_mb));
    s.push_str(&format!(
        "  \"pinned_snapshot_triples\": {},\n",
        r.pinned_snapshot_triples
    ));
    s.push_str(&format!("  \"after_drop_p50_us\": {},\n", r.after_drop_p50_us));
    s.push_str(&format!("  \"after_drop_max_us\": {},\n", r.after_drop_max_us));
    s.push_str(&format!("  \"final_rss_mb\": {:.0}\n", r.final_rss_mb));
    s.push_str("}\n");
    s
}

fn main() {
    // Strictly-additive: pull `--json <path>` out before reading the optional data path,
    // so the positional `.nt` argument is unaffected when the flag is absent.
    let (args, json_path) = take_json_flag(std::env::args().collect());
    let path = args.into_iter().nth(1);
    let mut res = Results::default();

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
    res.triples = triples;
    res.load_secs = t.elapsed().as_secs_f64();
    res.rss_after_load_mb = rss_mb();
    println!("loaded {triples} triples in {:.1}s; RSS after load: {:.0} MB", res.load_secs, res.rss_after_load_mb);

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
    res.snapshot_ns_per_op = t.elapsed().as_nanos() as f64 / iters as f64;
    res.snapshots_per_s_millions = iters as f64 / t.elapsed().as_secs_f64() / 1e6;
    println!(
        "snapshot (lock-free current(): ArcSwap load + Arc clone): {:.1} ns/op — {:.1} M snapshots/s single thread",
        res.snapshot_ns_per_op,
        res.snapshots_per_s_millions
    );

    // 2. update latencies, no snapshot held.
    let upd = |i: usize| format!("INSERT DATA {{ <http://ex/new{i}> <http://ex/p0> \"fresh{i}\" }}");
    let t = Instant::now();
    state.apply_update(&upd(0)).expect("first update");
    res.first_update_secs = t.elapsed().as_secs_f64();
    println!("update #1 (first => one-time O(graph) dictionary/cache freeze to mint the shared fork base): {:.2}s", res.first_update_secs);
    res.rss_after_first_update_mb = rss_mb();
    println!("RSS after first update (base + first forked generation live): {:.0} MB", res.rss_after_first_update_mb);

    let mut steady = Vec::new();
    for i in 1..21 {
        let t = Instant::now();
        state.apply_update(&upd(i)).expect("steady update");
        steady.push(t.elapsed().as_micros() as u64);
    }
    steady.sort_unstable();
    res.steady_p50_us = steady[steady.len() / 2];
    res.steady_max_us = *steady.last().unwrap();
    println!(
        "updates #2-#21 (steady in-place): p50={}us max={}us",
        res.steady_p50_us,
        res.steady_max_us
    );

    // 3. update while a reader pins the PREVIOUS generation (a long-lived stream).
    // Under the ring the pinned generation is just one of K retained Arcs — the writer
    // never has to reclaim it, so neither of these updates stalls (contrast the old
    // double-buffer, where the writer waited out timeout+grace then rebuilt O(graph)).
    let pinned = state.current(); // pins the currently-published generation
    let t = Instant::now();
    state.apply_update(&upd(100)).expect("update while previous gen pinned");
    res.pinned_update1_secs = t.elapsed().as_secs_f64();
    println!("update #1 with a generation pinned (writer publishes a fresh generation, no reclaim): {:.3}s", res.pinned_update1_secs);
    let t = Instant::now();
    state.apply_update(&upd(101)).expect("update while previous gen still pinned");
    res.pinned_update2_secs = t.elapsed().as_secs_f64();
    println!(
        "update #2 with a generation still pinned: {:.3}s  (ring: NO reclaim wait, NO O(graph) rebuild — should match step 2 steady-state)",
        res.pinned_update2_secs
    );
    res.rss_with_pinned_mb = rss_mb();
    println!("RSS with the pinned generation + the newer published generations live: {:.0} MB", res.rss_with_pinned_mb);
    res.pinned_snapshot_triples = pinned.snapshot().len();
    println!("pinned snapshot still answers at its generation: {} triples (published now has more)", res.pinned_snapshot_triples);
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
    res.after_drop_p50_us = after[after.len() / 2];
    res.after_drop_max_us = *after.last().unwrap();
    println!("updates after snapshot dropped: p50={}us max={}us (no recovery transient under the ring)", res.after_drop_p50_us, res.after_drop_max_us);
    res.final_rss_mb = rss_mb();
    println!("final RSS: {:.0} MB", res.final_rss_mb);

    // [OPUS-4.8] (sq-5vm.1) Strictly-additive JSON emit: only when `--json <path>` was given.
    // STDOUT above (the free-text metric lines) is the unchanged human/research output.
    if let Some(path) = json_path {
        let doc = results_json(&res);
        if let Err(e) = std::fs::write(&path, doc) {
            eprintln!("error writing --json results to {path}: {e}");
            std::process::exit(1);
        }
        eprintln!("wrote snapshot_spike results to {path}");
    }
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] (sq-5vm.1) --json emit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_flag_extraction() {
        let argv: Vec<String> = ["snapshot_spike", "data.nt", "--json", "/tmp/o.json"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let (positional, path) = take_json_flag(argv);
        assert_eq!(positional, vec!["snapshot_spike", "data.nt"]);
        assert_eq!(path.as_deref(), Some("/tmp/o.json"));
        // Absent flag: argv passes through untouched, no path.
        let plain: Vec<String> = ["snapshot_spike", "data.nt"].iter().map(|s| s.to_string()).collect();
        let (p2, none) = take_json_flag(plain.clone());
        assert_eq!(p2, plain);
        assert!(none.is_none());
    }

    #[test]
    fn results_json_round_trips() {
        let r = Results {
            triples: 1_000_000,
            load_secs: 1.5,
            rss_after_load_mb: 250.0,
            snapshot_ns_per_op: 18.4,
            snapshots_per_s_millions: 54.3,
            first_update_secs: 0.42,
            rss_after_first_update_mb: 480.0,
            steady_p50_us: 30,
            steady_max_us: 120,
            pinned_update1_secs: 0.031,
            pinned_update2_secs: 0.029,
            rss_with_pinned_mb: 510.0,
            pinned_snapshot_triples: 1_000_001,
            after_drop_p50_us: 28,
            after_drop_max_us: 95,
            final_rss_mb: 500.0,
        };
        let doc = results_json(&r);
        // The dependency-free emit must round-trip through a REAL serde_json parse.
        let v: serde_json::Value = serde_json::from_str(&doc).expect("emit must be valid JSON");
        assert_eq!(v["harness"], "serve-spikes snapshot_spike");
        assert!(v["note"].as_str().unwrap().contains("NON-CANONICAL"));
        assert_eq!(v["triples"], 1_000_000);
        assert!(v["snapshot_ns_per_op"].is_number());
        assert!(v["snapshots_per_s_millions"].is_number());
        assert_eq!(v["steady_p50_us"], 30);
        assert_eq!(v["pinned_snapshot_triples"], 1_000_001);
        assert!(v["pinned_update2_secs"].is_number());
        assert_eq!(v["after_drop_max_us"], 95);
        assert!(v["final_rss_mb"].is_number());

        // A defaulted (all-zero) Results is still valid JSON.
        let empty = results_json(&Results::default());
        let v2: serde_json::Value = serde_json::from_str(&empty).expect("default is valid JSON");
        assert_eq!(v2["triples"], 0);
    }
}
