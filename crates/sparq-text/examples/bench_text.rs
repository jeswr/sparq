//! [OPUS-4.8] (sq-ustq) TextIndex benchmark runner — the G1 TSV-emitting runner for
//! `bench/fts/`, the full-text-search analogue of `sparq-cli bench` / `bench_shacl`.
//!
//! Two axes, one binary (design: research/capability-benchmark-program.md §3.4):
//!
//! 1. **Latency axis (engine surface):** N synthetic 8-word literals over a ~10k-term
//!    Zipf-skewed vocabulary (deterministic seed) loaded into a sparq Graph; a
//!    positions-enabled BM25 index; AND / OR / prefix / phrase / near query latency.
//! 2. **Deterministic IR-structure axis:** the TOTAL hit count over a FIXED query set
//!    (independent of the corpus-gen RNG draw order) and the index BYTES-PER-DOC are
//!    integer-exact at the pinned `(N, seed)` corpus — the FTS analogue of LUBM's count
//!    diff / `store_bytes_per_triple`. These are the hard gate columns `bench/fts/run.sh`
//!    asserts against `expected.tsv`.
//!
//! ```sh
//! # No args reproduces the PINNED per-commit corpus (N=100000, seed=0) that
//! # bench/fts/gen.sh + expected.tsv are derived from — so a casual `cargo run`
//! # reproduces the gated counts. The heavy/latency 1M tier is opt-in via the first arg.
//! cargo run -p sparq-text --release --example bench_text              # N=100000 seed=0 iters=3
//! cargo run -p sparq-text --release --example bench_text -- [N] [seed] [iters]
//! ```
//!
//! Output: one TSV line per workload (sorted by name), the same `name\tcount\tus`
//! 3-column contract the ci-bench hook consumes, PLUS the deterministic `bytes_per_doc`
//! row whose `count` is the integer index footprint per document:
//!
//! ```text
//! and_terms     <total_hits>   <min_us_per_query>
//! or_terms      <total_hits>   <min_us_per_query>
//! prefix4       <total_hits>   <min_us_per_query>
//! phrase        <total_hits>   <min_us_per_query>
//! near_slop2    <total_hits>   <min_us_per_query>
//! bytes_per_doc <bytes_per_doc> <build_us>
//! ```
//!
//! - `count` for the query workloads = the TOTAL hit count summed over the fixed query
//!   set (a deterministic integer at the pinned corpus — the gate).
//! - `count` for `bytes_per_doc` = `heap_bytes() / len()` truncated to an integer
//!   (deterministic, runner-noise-immune — like `store_bytes_per_triple`).
//! - `us` = the best-of-`iters` time (ADVISORY; this dev box is non-canonical).
//!   For the query rows it is microseconds-per-query; for `bytes_per_doc` it is the
//!   index BUILD time in microseconds (advisory `text_build`).
//!
//! `bench/fts/run.sh` parses this, asserts the `count` columns against
//! `bench/fts/expected.tsv` (exit 1 on drift), and forwards the same 3-column contract.
//! Correctness lives in `expected.tsv`, NOT in the binary — exactly like LUBM / SHACL.

use std::time::Instant;

use rand::{rngs::StdRng, Rng, SeedableRng};
use sparq_core::Graph;
use sparq_text::TextIndex;

/// The DEFAULT corpus RNG seed — pinned to match `bench/fts/gen.sh` (seed 0) so a no-arg
/// `cargo run --example bench_text` reproduces EXACTLY the committed `bench/fts/expected.tsv`
/// corpus. (Distinct from the query-set seed below so the two are independent — the query
/// set never shifts when corpus generation changes its RNG consumption, which is what makes
/// the hit counts a STABLE gate.)
const CORPUS_SEED: u64 = 0;

/// The DEFAULT corpus size — pinned to match `bench/fts/gen.sh` (N=100000, the per-commit
/// tier) so a no-arg run reproduces the gated `expected.tsv` counts. The heavy/latency
/// 1M-literal tier is opt-in via the first arg (`cargo run --example bench_text -- 1000000`).
const CORPUS_N: usize = 100_000;

const STEMS: [&str; 20] = [
    "data", "graph", "query", "index", "store", "term", "literal", "token", "search", "join",
    "plan", "merge", "scan", "parse", "shard", "cache", "delta", "score", "match", "prefix",
];

/// A ~10k-word vocabulary: 20 stems x ~500 numbered suffixes; a Zipf-ish skew comes
/// from drawing the suffix index quadratically (low numbers common, high numbers rare).
fn word(rng: &mut StdRng) -> String {
    let stem = STEMS[rng.random_range(0..STEMS.len())];
    let r: f64 = rng.random_range(0.0..1.0);
    let suffix = (r * r * 500.0) as u32; // quadratic skew: ~common head, long tail
    format!("{stem}{suffix}")
}

/// A FIXED query set, derived ONLY from a dedicated seed (NOT from the post-corpus RNG
/// state). This is the load-bearing determinism trick: the same `(a, b)` term pairs are
/// drawn no matter how the corpus generator evolves, so the asserted hit counts only
/// change when search SEMANTICS change (the regression we want to catch), never as an
/// artefact of how many RNG draws the corpus consumed. Returns `QUERIES` term pairs.
fn fixed_queries(n: usize) -> Vec<(String, String)> {
    let mut q = StdRng::seed_from_u64(0xF7501); // "FTS-01" — independent query seed
    (0..n).map(|_| (word(&mut q), word(&mut q))).collect()
}

/// Number of queries in the fixed set (kept modest so the per-commit corpus stays
/// sub-second; the hit-count SUM over this set is the deterministic gate).
const QUERIES: usize = 200;

fn main() {
    let n: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(CORPUS_N);
    let seed: u64 = std::env::args().nth(2).and_then(|a| a.parse().ok()).unwrap_or(CORPUS_SEED);
    let iters: usize = std::env::args().nth(3).and_then(|a| a.parse().ok()).unwrap_or(3);
    let iters = iters.max(1);

    // ---- data generation + graph load ------------------------------------------------
    let mut rng = StdRng::seed_from_u64(seed);
    let mut nt = String::with_capacity(n * 120);
    for i in 0..n {
        let sentence: Vec<String> = (0..8).map(|_| word(&mut rng)).collect();
        nt.push_str(&format!(
            "<http://example.org/e{i}> <http://example.org/comment> \"{}\" .\n",
            sentence.join(" ")
        ));
    }
    let graph = Graph::load_str(&nt, "ntriples").unwrap();

    // ---- index build (positions enabled so phrase/near are exercised) -----------------
    // Best-of-iters build for the advisory `text_build` timing; the resulting index is
    // identical every iteration (build is a pure function of the graph), so heap_bytes()
    // and every hit count are deterministic.
    let mut build_us = f64::INFINITY;
    let mut index = TextIndex::default();
    for _ in 0..iters {
        let t = Instant::now();
        index = TextIndex::build_with_positions(&graph);
        build_us = build_us.min(t.elapsed().as_secs_f64() * 1e6);
    }
    let docs = index.len();
    assert!(docs > 0, "empty index — corpus generation produced no literals");
    // Integer bytes-per-doc (truncating div) — runner-noise-immune, like store_bytes_per_triple.
    let bytes_per_doc = index.heap_bytes() / docs;

    // ---- the fixed query set ----------------------------------------------------------
    let queries = fixed_queries(QUERIES);

    // Each workload: (name, total-hit-count over the query set, best-of-iters µs/query).
    // The count is summed deterministically; the timing is best-of-iters / |queries|.
    let mut rows: Vec<(String, usize, f64)> = Vec::new();

    // AND of two terms (`text:matches`): docs containing BOTH terms.
    {
        let (mut count, mut us) = (0usize, f64::INFINITY);
        for _ in 0..iters {
            let t = Instant::now();
            let mut c = 0usize;
            for (a, b) in &queries {
                c += index.search(&format!("{a} {b}")).len();
            }
            us = us.min(t.elapsed().as_secs_f64() * 1e6 / queries.len() as f64);
            count = c;
        }
        rows.push(("and_terms".into(), count, us));
    }

    // OR of two terms (`text:matchesAny`): docs containing AT LEAST ONE term.
    {
        let (mut count, mut us) = (0usize, f64::INFINITY);
        for _ in 0..iters {
            let t = Instant::now();
            let mut c = 0usize;
            for (a, b) in &queries {
                c += index.search_any(&format!("{a} {b}")).len();
            }
            us = us.min(t.elapsed().as_secs_f64() * 1e6 / queries.len() as f64);
            count = c;
        }
        rows.push(("or_terms".into(), count, us));
    }

    // Prefix (4 chars) of the first term — autocomplete shape (`text:matches "foo*"`).
    {
        let (mut count, mut us) = (0usize, f64::INFINITY);
        for _ in 0..iters {
            let t = Instant::now();
            let mut c = 0usize;
            for (a, _) in &queries {
                let p = &a[..a.len().min(4)];
                c += index.search(&format!("{p}*")).len();
            }
            us = us.min(t.elapsed().as_secs_f64() * 1e6 / queries.len() as f64);
            count = c;
        }
        rows.push(("prefix4".into(), count, us));
    }

    // Phrase (two adjacent, in-order terms — `text:phrase "foo bar"`).
    {
        let (mut count, mut us) = (0usize, f64::INFINITY);
        for _ in 0..iters {
            let t = Instant::now();
            let mut c = 0usize;
            for (a, b) in &queries {
                c += index.phrase(&format!("{a} {b}")).len();
            }
            us = us.min(t.elapsed().as_secs_f64() * 1e6 / queries.len() as f64);
            count = c;
        }
        rows.push(("phrase".into(), count, us));
    }

    // Proximity / slop (two in-order terms within gap 2 — `text:near "foo bar"` slop 2).
    {
        let (mut count, mut us) = (0usize, f64::INFINITY);
        for _ in 0..iters {
            let t = Instant::now();
            let mut c = 0usize;
            for (a, b) in &queries {
                c += index.phrase_near(&format!("{a} {b}"), 2).len();
            }
            us = us.min(t.elapsed().as_secs_f64() * 1e6 / queries.len() as f64);
            count = c;
        }
        rows.push(("near_slop2".into(), count, us));
    }

    rows.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, count, us) in &rows {
        println!("{name}\t{count}\t{us:.1}");
    }
    // The deterministic index-footprint gate (count = integer B/doc; us = advisory build time).
    println!("bytes_per_doc\t{bytes_per_doc}\t{build_us:.1}");
}
