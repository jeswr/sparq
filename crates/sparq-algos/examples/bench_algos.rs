//! Micro-benchmark for the sparq-algos surface: build a synthetic scale-free-ish RDF
//! graph, project it onto a [`NodeGraph`], and time PageRank, degree centrality, and the
//! two community passes. Prints `metric_us` lines (min-of-iters microseconds) so a harness
//! can scrape them. No external dataset — the graph is generated in-process.
//!
//! Run: `cargo run -p sparq-algos --release --example bench_algos -- [nodes] [avg_degree]`.

use std::time::Instant;

use sparq_algos::{
    degree_centrality, label_propagation, pagerank, weakly_connected_components, Direction,
    LabelPropConfig, NodeGraph, PageRankConfig,
};
use sparq_core::Graph;

fn main() {
    let mut args = std::env::args().skip(1);
    let nodes: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(50_000);
    let avg_degree: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(8);

    // Generate N-Triples for a preferential-attachment-style graph: node i links to a
    // handful of lower-indexed nodes (deterministic LCG so the run is reproducible).
    let mut nt = String::with_capacity(nodes * avg_degree * 48);
    let mut rng: u64 = 0x9e3779b97f4a7c15;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    for i in 1..nodes {
        for _ in 0..avg_degree {
            let target = (next() as usize) % i;
            nt.push_str(&format!(
                "<http://e/{i}> <http://p/k> <http://e/{target}> .\n"
            ));
        }
    }

    let t = Instant::now();
    let graph = Graph::load_str(&nt, "nt").expect("parse");
    let load_us = t.elapsed().as_micros();
    println!("metric_us load={load_us}");

    let t = Instant::now();
    let g = NodeGraph::build(&graph);
    println!("metric_us build={}", t.elapsed().as_micros());
    println!("nodes={} edges={}", g.len(), g.edge_count());

    let best = |label: &str, f: &dyn Fn() -> u128| {
        let mut lo = u128::MAX;
        for _ in 0..3 {
            lo = lo.min(f());
        }
        println!("metric_us {label}={lo}");
    };

    best("pagerank", &|| {
        let t = Instant::now();
        let r = pagerank(&g, PageRankConfig::default());
        std::hint::black_box(&r);
        t.elapsed().as_micros()
    });
    best("degree_total", &|| {
        let t = Instant::now();
        let d = degree_centrality(&g, Direction::Total);
        std::hint::black_box(&d);
        t.elapsed().as_micros()
    });
    best("wcc", &|| {
        let t = Instant::now();
        let c = weakly_connected_components(&g);
        std::hint::black_box(&c);
        t.elapsed().as_micros()
    });
    best("label_prop", &|| {
        let t = Instant::now();
        let c = label_propagation(&g, LabelPropConfig::default());
        std::hint::black_box(&c);
        t.elapsed().as_micros()
    });
}
