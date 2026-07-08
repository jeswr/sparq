//! [FABLE-5] sq-hmd7l.2 — standalone generator that REPLICATES the deterministic
//! corpus and fixed-query-set generation of `crates/sparq-text/examples/bench_text.rs`,
//! so the FTS same-box harness (`scripts/bench/fts-same-box.sh`) can materialise the
//! SAME corpus for a competitor engine (Fuseki + jena-text) and translate the SAME 200
//! query pairs, without modifying sparq-text (read-only for this bead).
//!
//! The `word()` / `fixed_queries()` bodies and every constant below are copied
//! verbatim-in-behaviour from `bench_text.rs` (same STEMS table, same quadratic Zipf
//! skew, same corpus line format, same independent query seed 0xF7501, same 200-pair
//! query count). `rand` is pinned `=0.10.1` (the root Cargo.lock version) because the
//! output is a function of StdRng's exact stream. Drift between this replica and
//! bench_text is DETECTED, not assumed away: the harness cross-checks per-workload hit
//! counts engine-vs-engine (`count_crosscheck`) and re-derives the pinned pairs file
//! (`pairs_pinned_check`); a mismatch is a recorded result, never a fixup.
//!
//! ```text
//! fts-corpus-gen corpus <N> <seed>   # N-Triples corpus on stdout (bench_text's exact lines)
//! fts-corpus-gen queries             # the fixed 200 query pairs, one `a<TAB>b` per line
//! ```

use rand::{rngs::StdRng, RngExt, SeedableRng};
use std::io::Write;

/// Verbatim from bench_text.rs: the 20-stem vocabulary base.
const STEMS: [&str; 20] = [
    "data", "graph", "query", "index", "store", "term", "literal", "token", "search", "join",
    "plan", "merge", "scan", "parse", "shard", "cache", "delta", "score", "match", "prefix",
];

/// Verbatim from bench_text.rs: number of fixed query pairs.
const QUERIES: usize = 200;

/// Verbatim from bench_text.rs: one Zipf-skewed vocabulary word (stem + quadratically
/// skewed numbered suffix). The DRAW ORDER (usize range, then f64 range) is load-bearing:
/// it must consume the StdRng stream exactly as bench_text does.
fn word(rng: &mut StdRng) -> String {
    let stem = STEMS[rng.random_range(0..STEMS.len())];
    let r: f64 = rng.random_range(0.0..1.0);
    let suffix = (r * r * 500.0) as u32; // quadratic skew: ~common head, long tail
    format!("{}{}", stem, suffix)
}

/// Verbatim from bench_text.rs: the fixed query set, drawn ONLY from the dedicated
/// query seed (0xF7501), independent of corpus generation.
fn fixed_queries(n: usize) -> Vec<(String, String)> {
    let mut q = StdRng::seed_from_u64(0xF7501); // "FTS-01" — independent query seed
    (0..n).map(|_| (word(&mut q), word(&mut q))).collect()
}

fn usage() -> ! {
    eprintln!("usage: fts-corpus-gen corpus <N> <seed>   # N-Triples corpus on stdout");
    eprintln!("       fts-corpus-gen queries             # 200 fixed query pairs (a<TAB>b)");
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("corpus") => {
            let (Some(n), Some(seed)) = (
                args.get(2).and_then(|a| a.parse::<usize>().ok()),
                args.get(3).and_then(|a| a.parse::<u64>().ok()),
            ) else {
                usage()
            };
            // Buffered + locked stdout: N=1e6 emits ~120 MB.
            let stdout = std::io::stdout();
            let mut out = std::io::BufWriter::new(stdout.lock());
            let mut rng = StdRng::seed_from_u64(seed);
            for i in 0..n {
                // Verbatim bench_text corpus line: 8 space-joined words per literal.
                let sentence: Vec<String> = (0..8).map(|_| word(&mut rng)).collect();
                writeln!(
                    out,
                    "<http://example.org/e{}> <http://example.org/comment> \"{}\" .",
                    i,
                    sentence.join(" ")
                )
                .expect("stdout write failed");
            }
            out.flush().expect("stdout flush failed");
        }
        Some("queries") => {
            let stdout = std::io::stdout();
            let mut out = std::io::BufWriter::new(stdout.lock());
            for (a, b) in fixed_queries(QUERIES) {
                writeln!(out, "{}\t{}", a, b).expect("stdout write failed");
            }
            out.flush().expect("stdout flush failed");
        }
        _ => usage(),
    }
}
