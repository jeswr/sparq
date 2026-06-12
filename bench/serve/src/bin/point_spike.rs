//! RESEARCH SPIKE — point-BGP lookup ceiling (research/concurrent-serving.md §a/§d).
//!
//! The NON-cached fast path: a fully-bound-subject single-pattern SELECT executed on the
//! engine against an immutable `Arc<Graph>` snapshot. This is what a cache MISS on a
//! point query costs end-to-end inside the process (parse + plan + index probe + JSON
//! serialisation), i.e. the per-core ceiling of the "executor fast tier" when the result
//! cache cannot answer.
//!
//! Measures, single-thread and N-thread (distinct subjects, Zipf-free — the worst case
//! for any cache, the best case for the index):
//!   * `query_json` end-to-end (what the server's SELECT path pays after snapshot);
//!   * parse-only (spargebra) for the same strings — isolating parse share;
//!   * `ask` on a bound triple (the cheapest possible engine round-trip).

use std::sync::Arc;
use std::time::Instant;

use sparq_core::Graph;
use sparq_engine::QueryBudget;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let threads: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(8);
    let mut nt = String::new();
    for i in 0..1_000_000 {
        nt.push_str(&format!(
            "<http://ex/s{}> <http://xmlns.com/foaf/0.1/name> \"person number {}\" .\n",
            i, i
        ));
    }
    let graph = Arc::new(Graph::load_str(&nt, "ntriples").expect("load"));
    println!("graph: {} triples", graph.len());

    let q_of = |i: usize| {
        format!("SELECT ?n WHERE {{ <http://ex/s{i}> <http://xmlns.com/foaf/0.1/name> ?n }}")
    };

    // parse-only baseline for these exact strings
    let iters = 50_000u64;
    let t = Instant::now();
    for i in 0..iters {
        let q = q_of(i as usize % 1_000_000);
        std::hint::black_box(spargebra::SparqlParser::new().parse_query(&q).unwrap());
    }
    println!(
        "parse only (incl. format!): {:.2} us/op",
        t.elapsed().as_nanos() as f64 / iters as f64 / 1000.0
    );

    // single-thread end-to-end point SELECT → JSON
    let b = QueryBudget::unlimited();
    let iters = 50_000u64;
    let t = Instant::now();
    let mut sink = 0usize;
    for i in 0..iters {
        let q = q_of((i as usize).wrapping_mul(2654435761) % 1_000_000);
        sink = sink.wrapping_add(sparq_engine::query_json_with_budget(&graph, &q, &b).unwrap().len());
    }
    let el = t.elapsed();
    println!(
        "point SELECT->JSON, 1 thread: {:.2} us/op = {:.0} K ops/s [{sink}]",
        el.as_nanos() as f64 / iters as f64 / 1000.0,
        iters as f64 / el.as_secs_f64() / 1e3
    );

    // ASK on a fully bound triple — cheapest engine round-trip
    let t = Instant::now();
    for i in 0..iters {
        let s = (i as usize).wrapping_mul(2654435761) % 1_000_000;
        let q = format!(
            "ASK {{ <http://ex/s{s}> <http://xmlns.com/foaf/0.1/name> \"person number {s}\" }}"
        );
        std::hint::black_box(sparq_engine::ask_with_budget(&graph, &q, &b).unwrap());
    }
    let el = t.elapsed();
    println!(
        "bound ASK, 1 thread: {:.2} us/op = {:.0} K ops/s",
        el.as_nanos() as f64 / iters as f64 / 1000.0,
        iters as f64 / el.as_secs_f64() / 1e3
    );

    // N-thread point SELECT → JSON on shared snapshot
    let per_thread = 30_000u64;
    let t = Instant::now();
    let handles: Vec<_> = (0..threads)
        .map(|ti| {
            let g = graph.clone();
            std::thread::spawn(move || {
                let b = QueryBudget::unlimited();
                let mut sink = 0usize;
                for i in 0..per_thread {
                    let s = ((i as usize) ^ (ti * 7919)).wrapping_mul(2654435761) % 1_000_000;
                    let q = format!(
                        "SELECT ?n WHERE {{ <http://ex/s{s}> <http://xmlns.com/foaf/0.1/name> ?n }}"
                    );
                    sink = sink
                        .wrapping_add(sparq_engine::query_json_with_budget(&g, &q, &b).unwrap().len());
                }
                sink
            })
        })
        .collect();
    let mut sink = 0usize;
    for h in handles {
        sink = sink.wrapping_add(h.join().unwrap());
    }
    let el = t.elapsed();
    let total = per_thread * threads as u64;
    println!(
        "point SELECT->JSON, {threads} threads: {:.2} M ops/s total ({:.2} us/op/thread) [{sink}]",
        total as f64 / el.as_secs_f64() / 1e6,
        el.as_nanos() as f64 / (total as f64 / threads as f64) / 1000.0
    );
}
