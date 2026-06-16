//! The design-doc accuracy gate: HNSW recall@10 vs exact brute force ≥ 0.95 on 50k
//! random vectors (research/genai-design.md §4, sparq-vectors row). Runs as an
//! ordinary `cargo test` — the workspace Cargo.toml raises the dev opt-level for this
//! crate and `instant-distance` so the build stays seconds, not minutes.

// [OPUS-4.8] (sq-ip3a) HNSW (VectorIndex) is the approximate backend — gated behind `approx-ann`.
#![cfg(feature = "approx-ann")]

use sparq_vectors::{nearest_exact, VectorIndex, VectorStore};

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// Uniform in [-1, 1).
fn rand_vec(state: &mut u64, dim: usize) -> Vec<f32> {
    (0..dim)
        .map(|_| ((splitmix64(state) >> 40) as f32 / (1u64 << 23) as f32) * 2.0 - 1.0)
        .collect()
}

#[test]
fn hnsw_recall_at_10_vs_brute_force_on_50k() {
    const N: usize = 50_000;
    const DIM: usize = 32;
    const K: usize = 10;
    const QUERIES: usize = 100;

    let path = std::env::temp_dir()
        .join(format!("sparq-vectors-recall-{}.spqv", std::process::id()));
    let mut store = VectorStore::create(&path, DIM).unwrap();
    let mut state = 0xC0FFEE_u64;
    for i in 0..N {
        // Sparse, non-contiguous ids to exercise the id→slot index.
        let id = (i as u32) * 7 + 3;
        store.put(id, &rand_vec(&mut state, DIM)).unwrap();
    }
    store.finalize().unwrap();

    let index = VectorIndex::build(&store);

    let mut q_state = 0xDECAF_u64;
    let mut hits = 0usize;
    for _ in 0..QUERIES {
        let q = rand_vec(&mut q_state, DIM);
        let exact: Vec<u32> = nearest_exact(&store, &q, K).into_iter().map(|(id, _)| id).collect();
        let approx: Vec<u32> = index.nearest(&q, K).into_iter().map(|(id, _)| id).collect();
        assert_eq!(exact.len(), K);
        assert_eq!(approx.len(), K);
        hits += approx.iter().filter(|id| exact.contains(id)).count();
    }
    let recall = hits as f64 / (QUERIES * K) as f64;
    eprintln!("HNSW recall@{K} over {QUERIES} queries on {N}x{DIM}: {recall:.4}");
    assert!(recall >= 0.95, "recall@{K} gate failed: {recall:.4} < 0.95");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn exact_and_hnsw_agree_on_tiny_separable_data() {
    // Three well-separated clusters; both searchers must return the query's own
    // cluster, with cosine scores in agreement.
    const DIM: usize = 8;
    let path = std::env::temp_dir()
        .join(format!("sparq-vectors-tiny-{}.spqv", std::process::id()));
    let mut store = VectorStore::create(&path, DIM).unwrap();
    let mut state = 7_u64;
    let mut centers = Vec::new();
    for c in 0..3 {
        let center = rand_vec(&mut state, DIM);
        for i in 0..20 {
            let v: Vec<f32> = center
                .iter()
                .map(|x| x + ((splitmix64(&mut state) >> 40) as f32 / (1u64 << 23) as f32 - 0.5) * 0.05)
                .collect();
            store.put((c * 100 + i) as u32 + 1, &v).unwrap();
        }
        centers.push(center);
    }
    store.finalize().unwrap();
    let index = VectorIndex::build(&store);

    for (c, center) in centers.iter().enumerate() {
        let exact = nearest_exact(&store, center, 5);
        let approx = index.nearest(center, 5);
        for &(id, sim) in exact.iter().chain(approx.iter()) {
            assert_eq!((id - 1) / 100, c as u32, "neighbor {id} from wrong cluster");
            assert!(sim > 0.99, "cluster member should be near-identical, got {sim}");
        }
    }

    // An all-zero query has no direction (cosine undefined): both searchers return
    // empty — and stored vectors can never be zero (VectorStore::put rejects them),
    // so exact and HNSW agree on every degenerate case.
    let zero = vec![0.0f32; DIM];
    assert!(nearest_exact(&store, &zero, 5).is_empty());
    assert!(index.nearest(&zero, 5).is_empty());

    let _ = std::fs::remove_file(&path);
}
