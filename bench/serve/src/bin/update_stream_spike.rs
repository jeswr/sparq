//! RESEARCH SPIKE — sustained small-update stream (research/concurrent-serving.md §d.iv):
//! the Solid write workload, one SPARQL UPDATE per resource write, run for tens of
//! thousands of updates against the production `AppState` writer.
//!
//! This is the direct test of the QLever failure mode prod-solid-server designed around
//! (qlever#2481: memory climbing under sustained live updates → OOM): does sparq's
//! double-buffered delta-overlay writer keep BOTH update latency and RSS flat over a
//! long update stream, including the periodic O(graph) compactions?
//!
//! Each update models prod-solid-server's `putDocument` shape scaled to the default
//! graph (the server's published surface today): DELETE the resource's previous ~9
//! metadata triples + INSERT fresh ones (~350 bytes of SPARQL). Resources are revisited
//! round-robin over a pool, so deletes really hit (an overlay that only grows by
//! inserts would flatter the result).
//!
//! Optional concurrent readers (`--readers N`) take snapshots and run a point query in
//! a loop — checking that the reclaim path doesn't degrade under read load.
//!
//! Output: update p50/p99/max per 1k-window, RSS per window, total throughput.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use sparq_core::Graph;
use sparq_engine::QueryBudget;
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

fn put_document_update(res: usize, version: usize) -> String {
    // prod-solid-server putDocument analog: replace the resource's metadata triples.
    // DELETE WHERE clears the previous version's triples; INSERT DATA writes ~9 fresh ones.
    format!(
        "DELETE WHERE {{ <http://pod.ex/r{res}> ?p ?o }} ; \
         INSERT DATA {{ \
           <http://pod.ex/r{res}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/ldp#Resource> . \
           <http://pod.ex/r{res}> <https://pss.dev/ns#contentType> \"text/turtle\" . \
           <http://pod.ex/r{res}> <http://www.w3.org/ns/posix/stat#size> {version} . \
           <http://pod.ex/r{res}> <http://www.w3.org/ns/posix/stat#mtime> {version} . \
           <http://pod.ex/r{res}> <http://purl.org/dc/terms/modified> \"2026-06-12T00:00:{:02}\" . \
           <http://pod.ex/r{res}> <https://pss.dev/ns#etag> \"etag-{res}-{version}\" . \
           <http://pod.ex/r{res}> <https://pss.dev/ns#s3Key> \"k/{res}/{version}\" . \
           <http://pod.ex/parent> <http://www.w3.org/ns/ldp#contains> <http://pod.ex/r{res}> . \
         }}",
        version % 60
    )
}

fn main() {
    let mut updates = 20_000usize;
    let mut resources = 2_000usize;
    let mut readers = 0usize;
    let mut compact_every = 1024usize;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut next = || args.next().expect("flag value").parse::<usize>().expect("number");
        match a.as_str() {
            "--updates" => updates = next(),
            "--resources" => resources = next(),
            "--readers" => readers = next(),
            "--compact-every" => compact_every = next(),
            other => panic!("unknown arg {other}"),
        }
    }

    // Base graph: 1M triples of unrelated data + initial resource metadata.
    let mut nt = String::new();
    for i in 0..1_000_000 {
        nt.push_str(&format!("<http://ex/s{}> <http://ex/p{}> \"v{}\" .\n", i % 200_000, i % 50, i));
    }
    let graph = Graph::load_str(&nt, "ntriples").expect("load");
    println!(
        "base graph: {} triples, RSS {:.0} MB; {updates} updates over {resources} resources, compact_every={compact_every}, readers={readers}",
        graph.len(),
        rss_mb()
    );

    let config = ServerConfig {
        query_timeout: Some(Duration::from_secs(5)),
        compact_every,
        ..ServerConfig::default()
    };
    let state = AppState::with_config(graph, config);
    let stop = Arc::new(AtomicBool::new(false));

    let reader_handles: Vec<_> = (0..readers)
        .map(|i| {
            let st = state.clone();
            let stop = stop.clone();
            std::thread::spawn(move || {
                let b = QueryBudget::unlimited();
                let mut n = 0u64;
                while !stop.load(Ordering::Relaxed) {
                    let g = st.snapshot();
                    let s = (n as usize).wrapping_mul(2654435761).wrapping_add(i) % 200_000;
                    let q = format!("SELECT ?o WHERE {{ <http://ex/s{s}> ?p ?o }} LIMIT 5");
                    std::hint::black_box(sparq_engine::query_json_with_budget(&g, &q, &b).ok());
                    n += 1;
                }
                n
            })
        })
        .collect();

    let window = 1_000usize;
    let mut lat = Vec::with_capacity(window);
    let run_start = Instant::now();
    let mut window_start = Instant::now();
    for i in 0..updates {
        let res = i % resources;
        let upd = put_document_update(res, i / resources + 1);
        let t = Instant::now();
        state.apply_update(&upd).expect("update");
        lat.push(t.elapsed().as_micros() as u64);
        if lat.len() == window {
            lat.sort_unstable();
            println!(
                "updates {:>6}-{:>6}: thr={:>6.0}/s p50={:>5}us p99={:>6}us max={:>8}us  RSS={:>5.0} MB",
                i + 1 - window,
                i + 1,
                window as f64 / window_start.elapsed().as_secs_f64(),
                lat[window / 2],
                lat[window * 99 / 100],
                lat[window - 1],
                rss_mb()
            );
            lat.clear();
            window_start = Instant::now();
        }
    }
    stop.store(true, Ordering::Relaxed);
    let reads: u64 = reader_handles.into_iter().map(|h| h.join().unwrap()).sum();
    println!(
        "TOTAL: {updates} updates in {:.1}s = {:.0} updates/s sustained; reader queries completed: {reads}; final RSS {:.0} MB",
        run_start.elapsed().as_secs_f64(),
        updates as f64 / run_start.elapsed().as_secs_f64(),
        rss_mb()
    );
    // Final published graph sanity: each resource should have exactly 8 triples + parent containment.
    let g = state.snapshot();
    println!("final graph: {} triples", g.len());
}
