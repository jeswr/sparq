//! RESEARCH SPIKE — streaming seam assessment (research/concurrent-serving.md §d.iv).
//!
//! `sparq_engine::query_json_chunks_with_budget` is the server's existing "streamed"
//! SELECT path. This spike measures whether it actually streams in TIME (incremental
//! delivery) or only in SPACE (memory): time-to-first-chunk vs total time, against
//! `query_json` (single string) and `count` (pure compute, no serialisation).
//!
//! Hypothesis to test: the chunks Vec is fully evaluated before the function returns,
//! so time-to-first-chunk == full evaluation time — i.e. today's seam removes the
//! second result copy but does NOT give a pull-streaming executor; true streaming
//! needs an iterator/callback seam in the engine (an engine-hooks ask).

use std::time::Instant;

use sparq_core::Graph;
use sparq_engine::QueryBudget;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let path = std::env::args().nth(1);
    let graph = match &path {
        Some(p) => {
            eprintln!("loading {p} ...");
            Graph::load_str(&std::fs::read_to_string(p).expect("read"), "ntriples").expect("load")
        }
        None => {
            let mut nt = String::new();
            for i in 0..1_000_000 {
                nt.push_str(&format!("<http://ex/s{i}> <http://ex/p> \"value number {i}\" .\n"));
            }
            Graph::load_str(&nt, "ntriples").expect("load synthetic")
        }
    };
    println!("loaded {} triples", graph.len());

    for q in [
        "SELECT ?s ?p ?o WHERE { ?s ?p ?o }",                 // full scan, graph-sized result
        "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 100000",    // large but bounded
        "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10",        // the cheap case
    ] {
        let b = QueryBudget::unlimited();

        // Pure compute (no term materialisation, no serialisation).
        let t = Instant::now();
        let n = sparq_engine::count(&graph, q).expect("count");
        let t_count = t.elapsed();

        // Chunked path: time-to-return == time-to-first-chunk (the Vec is complete
        // before we can see chunk 0), then "drain" the chunks like hyper would.
        let t = Instant::now();
        let chunks = sparq_engine::query_json_chunks_with_budget(&graph, q, &b).expect("chunks");
        let t_first = t.elapsed();
        let total_bytes: usize = chunks.iter().map(String::len).sum();
        let n_chunks = chunks.len();
        let t = Instant::now();
        let mut sink = 0usize;
        for c in &chunks {
            sink = sink.wrapping_add(std::hint::black_box(c.len()));
        }
        let t_drain = t.elapsed();

        // Single-string path for comparison.
        let t = Instant::now();
        let s = sparq_engine::query_json(&graph, q).expect("json");
        let t_single = t.elapsed();

        println!(
            "{q}\n  rows={n} chunks={n_chunks} bytes={total_bytes}\n  count(compute only)      {:>10.1} ms\n  chunks: time-to-first    {:>10.1} ms   drain {:.3} ms  [{sink}]\n  query_json (one string)  {:>10.1} ms\n  => first-chunk latency is {:.0}% of full evaluation (1.0 == no time streaming)",
            t_count.as_secs_f64() * 1e3,
            t_first.as_secs_f64() * 1e3,
            t_drain.as_secs_f64() * 1e3,
            t_single.as_secs_f64() * 1e3,
            t_first.as_secs_f64() / t_single.as_secs_f64() * 100.0,
        );
        let _ = s;
    }
}
