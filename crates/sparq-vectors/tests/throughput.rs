//! Throughput measurement behind `#[ignore]` — the source of the README numbers.
//! Run with:
//!
//! ```text
//! cargo test -p sparq-vectors --release --test throughput -- --ignored --nocapture
//! ```
//!
//! Prints store build/finalize/open, HNSW build, and exact vs HNSW query throughput
//! on the same 50k×32 random workload as the recall gate (tests/recall.rs).

// [OPUS-4.8] (sq-ip3a) HNSW (VectorIndex) is the approximate backend — gated behind `approx-ann`.
#![cfg(feature = "approx-ann")]

use sparq_vectors::{nearest_exact, VectorIndex, VectorStore};
use std::time::Instant;

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

fn rand_vec(state: &mut u64, dim: usize) -> Vec<f32> {
    (0..dim)
        .map(|_| ((splitmix64(state) >> 40) as f32 / (1u64 << 23) as f32) * 2.0 - 1.0)
        .collect()
}

#[test]
#[ignore = "throughput measurement, not a correctness gate; run with --ignored --nocapture"]
fn throughput_50k() {
    const N: usize = 50_000;
    const DIM: usize = 32;
    const K: usize = 10;
    const QUERIES: usize = 200;

    let path = std::env::temp_dir()
        .join(format!("sparq-vectors-throughput-{}.spqv", std::process::id()));

    let t = Instant::now();
    let mut store = VectorStore::create(&path, DIM).unwrap();
    let mut state = 0xC0FFEE_u64;
    for i in 0..N {
        store.put((i as u32) * 7 + 3, &rand_vec(&mut state, DIM)).unwrap();
    }
    let put = t.elapsed();
    let t = Instant::now();
    store.finalize().unwrap();
    let finalize = t.elapsed();
    let t = Instant::now();
    let reopened = VectorStore::open(&path).unwrap();
    let open = t.elapsed();
    assert_eq!(reopened.len(), N);

    let t = Instant::now();
    let index = VectorIndex::build(&store);
    let build = t.elapsed();

    let mut q_state = 0xDECAF_u64;
    let queries: Vec<Vec<f32>> = (0..QUERIES).map(|_| rand_vec(&mut q_state, DIM)).collect();

    let t = Instant::now();
    let mut sink = 0usize;
    for q in &queries {
        sink += nearest_exact(&store, q, K).len();
    }
    let exact = t.elapsed();

    let t = Instant::now();
    for q in &queries {
        sink += index.nearest(q, K).len();
    }
    let hnsw = t.elapsed();
    assert_eq!(sink, 2 * QUERIES * K);

    let per = |d: std::time::Duration| d.as_secs_f64() / QUERIES as f64;
    eprintln!("--- sparq-vectors throughput, {N}x{DIM}, k={K} ---");
    eprintln!("put {N} vectors:        {put:?}");
    eprintln!("finalize (write+mmap):  {finalize:?}");
    eprintln!("open (mmap):            {open:?}");
    eprintln!("HNSW build:             {build:?}");
    eprintln!(
        "exact top-{K}:           {:.3} ms/query ({:.0} q/s)",
        per(exact) * 1e3,
        1.0 / per(exact)
    );
    eprintln!(
        "HNSW top-{K}:            {:.4} ms/query ({:.0} q/s)",
        per(hnsw) * 1e3,
        1.0 / per(hnsw)
    );

    let _ = std::fs::remove_file(&path);
}
