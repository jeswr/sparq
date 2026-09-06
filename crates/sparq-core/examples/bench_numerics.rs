//! [OPUS-5] (sq-7d3dj.8) A/B harness for the NATIVE numerics-cache layout — roadmap item 12
//! of `research/optimization-audit-2026-07.md`, verdict recorded in
//! `research/numerics-sparsify-measured.md`.
//!
//! `Graph::build` keeps a DENSE `Vec<f64>` numerics cache — a flat 8 B per dictionary term,
//! NaN for the non-numeric majority — so `NumData::lookup` is an O(1) array index on the
//! numeric FILTER / ORDER BY / MIN-MAX fast path. Sparsifying it (as the temporal cache
//! already does at build) would cut the footprint on string-heavy data to ~17 B per REAL
//! numeric literal, but turns that array index into a hashmap probe. This binary measures
//! BOTH sides of that trade so the decision is made on numbers, not intuition:
//!
//! ```sh
//! # arm A (control, the shipped default): dense cache
//! cargo run -p sparq-core --release --example bench_numerics -- [rows] [iters]
//! # arm B (treatment): the same source, sparsified at build
//! cargo run -p sparq-core --release --example bench_numerics --features sparse-numerics -- [rows] [iters]
//! ```
//!
//! Two corpora at the SAME row count, differing only in how many object literals are
//! numeric — that density is the variable the `into_sparse_if_worthwhile` heuristic keys on:
//!
//! - `string_heavy` — 2% numeric objects. Well under the heuristic's 25% cut, so arm B
//!   sparsifies: this is where the footprint win, if any, shows up.
//! - `numeric_heavy` — 80% numeric objects. Over the cut, so arm B DECLINES and stays
//!   dense: it is the control that proves the heuristic protects the fast path, and the
//!   arms should be indistinguishable on it.
//!
//! Output is the house 3-column TSV (`name\tcount\tus`), one row per corpus x workload:
//!
//! ```text
//! <corpus>_heap_bytes   <Graph::heap_bytes()>   <build_us>
//! <corpus>_probe_random <numeric_hits>          <best_us_per_full_probe_pass>
//! <corpus>_probe_seq    <numeric_hits>          <best_us_per_full_scan_pass>
//! ```
//!
//! - `probe_random` walks every dictionary id in a deterministic COPRIME-STRIDE permutation
//!   — the cache-hostile, random-access shape a FILTER / ORDER BY key extraction actually
//!   has (row order has nothing to do with id order). This is the workload the dense layout
//!   exists to serve, so it is where a sparse cache must hold within noise to be adoptable.
//! - `probe_seq` walks ids 1..=n in order — the MIN/MAX aggregate shape, prefetch-friendly.
//! - The `count` column is the number of cached numeric literals found. It is a pure
//!   function of the corpus, so it MUST be byte-identical across the two arms: a differing
//!   count means the arms are not computing the same thing and the timing comparison is
//!   void. (`sparse_numerics_ab_arms_agree_on_every_id` in the crate pins the same property
//!   per id.)
//! - `heap_bytes` cites `Graph::heap_bytes()` — the store's own self-accounting — NEVER
//!   process RSS. `us` is best-of-`iters`; on any developer box it is ADVISORY and
//!   NON-CANONICAL. The adoption verdict requires a quiet EC2 run.

use std::time::Instant;

use sparq_core::Graph;

/// Rows per corpus. 200k rows -> ~400k dictionary terms, so the dense cache is ~3.2 MB —
/// comfortably past L2 on any box, which is the regime the layout question is about.
const DEFAULT_ROWS: usize = 200_000;
/// Timed repetitions per workload; the reported `us` is the BEST (least-interrupted) pass.
const DEFAULT_ITERS: usize = 5;

fn main() {
    let mut args = std::env::args().skip(1);
    let rows: usize = args.next().and_then(|a| a.parse().ok()).unwrap_or(DEFAULT_ROWS);
    let iters: usize = args.next().and_then(|a| a.parse().ok()).unwrap_or(DEFAULT_ITERS);

    // Which arm produced these rows. On stderr so the TSV on stdout stays machine-clean.
    let arm = if cfg!(feature = "sparse-numerics") { "sparse (feature ON)" } else { "dense (default)" };
    eprintln!("bench_numerics: arm={} rows={} iters={}", arm, rows, iters);
    eprintln!("bench_numerics: `us` columns are ADVISORY on a shared box — canonical numbers need EC2.");

    for (corpus, numeric_frac) in [("string_heavy", 0.02_f64), ("numeric_heavy", 0.80_f64)] {
        let nt = corpus_ntriples(rows, numeric_frac);

        let t = Instant::now();
        let g = Graph::load_str(&nt, "ntriples").expect("corpus must parse");
        let build_us = t.elapsed().as_micros();
        println!("{}_heap_bytes\t{}\t{}", corpus, g.heap_bytes(), build_us);

        let n = g.dict.len();
        let (hits, us) = best_of(iters, || probe(&g, permuted_ids(n)));
        println!("{}_probe_random\t{}\t{}", corpus, hits, us);
        let (hits, us) = best_of(iters, || probe(&g, 1..=n as u32));
        println!("{}_probe_seq\t{}\t{}", corpus, hits, us);
    }
}

/// A deterministic N-Triples corpus: one triple per row, every subject and object distinct,
/// the first `numeric_frac` of the objects `xsd:double` literals and the rest plain strings.
/// Distinct doubles are what land in the numerics cache (small integers are inline-encoded
/// into their id and never reach it), so this is the shape that actually moves the layout.
fn corpus_ntriples(rows: usize, numeric_frac: f64) -> String {
    let cut = (rows as f64 * numeric_frac) as usize;
    // Rough pre-size: ~70 bytes per line, avoiding a dozen regrows on a 200k-row corpus.
    let mut nt = String::with_capacity(rows * 72);
    for i in 0..rows {
        nt.push_str(&format!("<http://ex/s{}> <http://ex/v> ", i));
        if i < cut {
            nt.push_str(&format!("\"{}.5\"^^<http://www.w3.org/2001/XMLSchema#double>", i));
        } else {
            nt.push_str(&format!("\"string value {}\"", i));
        }
        nt.push_str(" .\n");
    }
    nt
}

/// Every id in `1..=n`, in a deterministic permutation with a COPRIME stride — a full,
/// repeatable cover of the dictionary that defeats sequential prefetch, so the random-access
/// probe cost is measured rather than the hardware's guess at it.
fn permuted_ids(n: usize) -> impl Iterator<Item = u32> {
    // A large odd stride, nudged up until it is coprime with `n` (which makes the walk a
    // permutation rather than a short cycle).
    let mut stride = (n / 3).max(1) | 1;
    while gcd(stride, n) != 1 {
        stride += 2;
    }
    let mut cur = 0usize;
    (0..n).map(move |_| {
        cur = (cur + stride) % n;
        cur as u32 + 1
    })
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// One full pass of the numeric fast path (`Graph::numeric_value` -> `NumData::lookup`),
/// returning the number of cached numeric literals found. Summing the values keeps the
/// probe from being optimised out and mirrors an aggregate's real data dependency.
fn probe(g: &Graph, ids: impl Iterator<Item = u32>) -> u64 {
    let mut hits = 0u64;
    let mut sum = 0.0f64;
    for id in ids {
        if let Some(v) = g.numeric_value(id) {
            hits += 1;
            sum += v;
        }
    }
    std::hint::black_box(sum);
    hits
}

/// Runs `f` `iters` times, returning its (invariant) result and the BEST wall time in
/// microseconds — least-of-N is the noise-robust statistic for a deterministic pass.
fn best_of(iters: usize, mut f: impl FnMut() -> u64) -> (u64, u128) {
    let mut best = u128::MAX;
    let mut out = 0u64;
    for _ in 0..iters.max(1) {
        let t = Instant::now();
        out = f();
        best = best.min(t.elapsed().as_micros());
    }
    (out, best)
}
