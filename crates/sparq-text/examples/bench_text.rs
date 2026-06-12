//! TextIndex benchmark: build time, index size and query latency on a
//! synthetic literal-heavy dataset.
//!
//! Run with:
//! ```sh
//! cargo run --release -p sparq-text --example bench_text
//! ```
//!
//! Generates N distinct literals (seeded; 8-word Zipf-flavoured sentences
//! over a ~10k-word vocabulary, so token frequencies span common -> rare),
//! loads them into a sparq Graph as N-Triples, builds the TextIndex, then
//! measures AND / OR / prefix query latency. Numbers land in the crate
//! README (with the contended-machine caveat).

use std::time::Instant;

use rand::{rngs::StdRng, Rng, SeedableRng};
use sparq_core::Graph;
use sparq_text::TextIndex;

/// A ~10k-word vocabulary: 100 stems x 100 numbered suffixes; a Zipf-ish
/// skew comes from drawing the suffix index quadratically (low numbers
/// common, high numbers rare).
fn word(rng: &mut StdRng) -> String {
    const STEMS: [&str; 20] = [
        "data", "graph", "query", "index", "store", "term", "literal", "token", "search", "join",
        "plan", "merge", "scan", "parse", "shard", "cache", "delta", "score", "match", "prefix",
    ];
    let stem = STEMS[rng.random_range(0..STEMS.len())];
    let r: f64 = rng.random_range(0.0..1.0);
    let suffix = (r * r * 500.0) as u32; // quadratic skew: ~common head, long tail
    format!("{stem}{suffix}")
}

fn main() {
    let n: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(1_000_000);
    let mut rng = StdRng::seed_from_u64(20260612);

    // ---- data generation + graph load ------------------------------------------
    let mut nt = String::with_capacity(n * 120);
    for i in 0..n {
        let sentence: Vec<String> = (0..8).map(|_| word(&mut rng)).collect();
        nt.push_str(&format!(
            "<http://example.org/e{i}> <http://example.org/comment> \"{}\" .\n",
            sentence.join(" ")
        ));
    }
    let t = Instant::now();
    let graph = Graph::load_str(&nt, "ntriples").unwrap();
    let load = t.elapsed();
    println!("graph load     : {n} literal triples in {load:.2?}");

    // ---- index build -------------------------------------------------------------
    let t = Instant::now();
    let index = TextIndex::build(&graph);
    let build = t.elapsed();
    println!(
        "index build    : {} docs, {} tokens in {build:.2?} ({:.2} Mdocs/s)",
        index.len(),
        index.token_count(),
        index.len() as f64 / build.as_secs_f64() / 1e6
    );
    println!(
        "index size     : ~{:.1} MiB heap ({:.1} B/doc)",
        index.heap_bytes() as f64 / (1024.0 * 1024.0),
        index.heap_bytes() as f64 / index.len() as f64
    );

    // ---- queries ------------------------------------------------------------------
    let queries: Vec<(String, String)> = (0..1000)
        .map(|_| (word(&mut rng), word(&mut rng)))
        .collect();
    for (label, all) in [("AND 2 terms", true), ("OR  2 terms", false)] {
        let t = Instant::now();
        let mut total = 0usize;
        for (a, b) in &queries {
            let q = format!("{a} {b}");
            total += if all { index.search(&q).len() } else { index.search_any(&q).len() };
        }
        let el = t.elapsed();
        println!(
            "{label}    : {:>8.1} µs/query ({:.1} avg hits)",
            el.as_micros() as f64 / queries.len() as f64,
            total as f64 / queries.len() as f64
        );
    }
    let t = Instant::now();
    let mut total = 0usize;
    for (a, _) in &queries {
        total += index.search(&format!("{}*", &a[..a.len().min(4)])).len();
    }
    let el = t.elapsed();
    println!(
        "prefix (4 ch)  : {:>8.1} µs/query ({:.1} avg hits)",
        el.as_micros() as f64 / queries.len() as f64,
        total as f64 / queries.len() as f64
    );
}
