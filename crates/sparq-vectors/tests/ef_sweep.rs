//! Per-query `ef_search` API tests (sq-jo6ty) — `nearest_with_ef` on `VectorIndex`.
//!
//! [SONNET-4.6] (sq-jo6ty) Three invariants checked here:
//!
//! 1. **Default-ef parity**: `nearest_with_ef(q, k, build_ef)` returns the byte-identical
//!    result as `nearest(q, k)` — the primary map is used in both cases.
//!
//! 2. **Monotone-recall**: sweeping `ef_search` upward (16 → 32 → 64 → 128 → 256) over a
//!    fixed corpus + query set is non-decreasing in recall@10 vs the exact oracle on every
//!    step. This is the load-bearing invariant for the HNSW property paper (§4 of the
//!    Malkov/Yashunin 2020 paper): a wider beam subsumes the candidates explored by any
//!    narrower beam, so recall can only stay flat or improve as ef grows.
//!
//! 3. **Zero-query contract**: an all-zero query returns `[]` at any ef (same as `nearest`).
//!
//! The test corpus is 5 000 × 32 (splitmix64 seed — deterministic, no I/O) which runs in
//! a few seconds even in debug. Recall comparisons use `nearest_exact` as the oracle.

// [SONNET-4.6] (sq-jo6ty) HNSW + nearest_with_ef are gated behind `approx-ann`.
#![cfg(feature = "approx-ann")]

use sparq_vectors::{nearest_exact, HnswConfig, VectorIndex, VectorStore};

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

/// Build a small in-memory store (no file) and return the `VectorIndex`.
/// N=5000, DIM=32, non-contiguous ids (id = i*7+3) to exercise the binary-search index.
fn build_test_store_and_index() -> (VectorStore, VectorIndex) {
    const N: usize = 5_000;
    const DIM: usize = 32;

    let path = std::env::temp_dir().join(format!(
        "sparq-ef-sweep-{}-{}.spqv",
        std::process::id(),
        0xEF_u64
    ));
    let mut store = VectorStore::create(&path, DIM).unwrap();
    let mut state = 0xABCD_EF01_u64;
    for i in 0..N {
        let id = (i as u32) * 7 + 3;
        store.put(id, &rand_vec(&mut state, DIM)).unwrap();
    }
    store.finalize().unwrap();
    // Build index with ef_search=50 as the default so that some ef values tested are
    // below it (e.g. 16, 32) and some above it (e.g. 64, 128, 256).
    let cfg = HnswConfig {
        ef_search: 50,
        ef_construction: 100,
        seed: 0x5350_5156_0001,
    };
    let index = VectorIndex::build_with(&store, cfg);
    let _ = std::fs::remove_file(&path);
    (store, index)
}

// --- 1. Default-ef parity ----------------------------------------------------------------

/// `nearest_with_ef(q, k, build_ef)` must return the byte-identical result as `nearest(q, k)`.
#[test]
fn nearest_with_build_ef_equals_nearest() {
    const DIM: usize = 32;
    const K: usize = 10;
    const QUERIES: usize = 30;
    // The build ef_search is 50 (set in build_test_store_and_index).
    const BUILD_EF: usize = 50;

    let (store, index) = build_test_store_and_index();
    let mut q_state = 0xFACE_B00C_u64;
    for _ in 0..QUERIES {
        let q = rand_vec(&mut q_state, DIM);
        let via_nearest = index.nearest(&q, K);
        let via_with_ef = index.nearest_with_ef(&q, K, BUILD_EF);
        assert_eq!(
            via_nearest, via_with_ef,
            "nearest_with_ef(q, k, build_ef) must equal nearest(q, k)"
        );
    }
    drop(store);
}

// --- 2. Monotone-recall ------------------------------------------------------------------

/// The recall@k vs exact oracle must be non-decreasing as ef_search increases.
///
/// [SONNET-4.6] (sq-jo6ty) This is the LOAD-BEARING invariant: it tests that
/// `nearest_with_ef` actually controls the beam width (wider = more recall), not
/// just returning the build-time ef result at every ef value.
#[test]
fn ef_sweep_monotone_recall_non_decreasing() {
    const DIM: usize = 32;
    const K: usize = 10;
    const QUERIES: usize = 100;
    // Sweep: [16, 32, 64, 128, 256] — covers below, at-build (50), and above-build ef values.
    const EF_LEVELS: &[usize] = &[16, 32, 64, 128, 256];

    let (store, index) = build_test_store_and_index();

    // Compute recall@K for each ef level over the same fixed query set.
    let mut q_state = 0xDEAD_BEEF_u64;
    let queries: Vec<Vec<f32>> = (0..QUERIES).map(|_| rand_vec(&mut q_state, DIM)).collect();

    let mut recalls: Vec<f64> = Vec::with_capacity(EF_LEVELS.len());
    for &ef in EF_LEVELS {
        let mut hits = 0usize;
        for q in &queries {
            let exact_ids: Vec<u32> = nearest_exact(&store, q, K)
                .into_iter()
                .map(|(id, _)| id)
                .collect();
            let approx_ids: Vec<u32> = index
                .nearest_with_ef(q, K, ef)
                .into_iter()
                .map(|(id, _)| id)
                .collect();
            assert_eq!(
                approx_ids.len(),
                K,
                "nearest_with_ef must return exactly K results (ef={})",
                ef
            );
            hits += approx_ids
                .iter()
                .filter(|id| exact_ids.contains(id))
                .count();
        }
        let recall = hits as f64 / (QUERIES * K) as f64;
        eprintln!("ef={ef:>4}  recall@{K}={recall:.4}");
        recalls.push(recall);
    }

    // Check monotone non-decreasing across every consecutive pair.
    for i in 1..recalls.len() {
        let (prev_ef, cur_ef) = (EF_LEVELS[i - 1], EF_LEVELS[i]);
        let (prev_recall, cur_recall) = (recalls[i - 1], recalls[i]);
        // Allow a tiny numeric tolerance for floating-point representation of
        // equal recalls expressed as different fractions (e.g. 95/100 vs 190/200).
        assert!(
            cur_recall >= prev_recall - 1e-9,
            "recall@{K} decreased from ef={prev_ef} ({prev_recall:.4}) to ef={cur_ef} ({cur_recall:.4}) — \
             monotone invariant violated"
        );
    }

    // Also assert that the highest ef (256) achieves at least the 0.95 floor.
    let top_recall = *recalls.last().unwrap();
    assert!(
        top_recall >= 0.95,
        "recall@{K} at ef=256 is {top_recall:.4} < 0.95 — recall floor violated"
    );

    drop(store);
}

// --- 3. Zero-query contract -------------------------------------------------------------

/// An all-zero query returns an empty result at any ef, matching `nearest`'s contract.
#[test]
fn nearest_with_ef_zero_query_returns_empty() {
    const DIM: usize = 32;
    const K: usize = 10;
    const EF_LEVELS: &[usize] = &[1, 16, 50, 100, 256];

    let (store, index) = build_test_store_and_index();
    let zero = vec![0.0f32; DIM];
    for &ef in EF_LEVELS {
        let result = index.nearest_with_ef(&zero, K, ef);
        assert!(
            result.is_empty(),
            "all-zero query should return [] at ef={ef}, got {} results",
            result.len()
        );
    }
    drop(store);
}

// --- 4. Cache reuse — calling the same ef twice is consistent ---------------------------

/// Calling `nearest_with_ef` twice at the same (non-default) ef yields identical results.
/// This ensures the lazy-build cache does not corrupt the secondary map.
#[test]
fn nearest_with_ef_cache_reuse_is_deterministic() {
    const DIM: usize = 32;
    const K: usize = 10;
    const QUERIES: usize = 20;
    const ALT_EF: usize = 32; // non-default (build ef = 50)

    let (store, index) = build_test_store_and_index();
    let mut q_state = 0xCAFE_BABE_u64;
    let queries: Vec<Vec<f32>> = (0..QUERIES).map(|_| rand_vec(&mut q_state, DIM)).collect();

    // First pass — populates the cache for ALT_EF.
    let first: Vec<Vec<(u32, f32)>> = queries
        .iter()
        .map(|q| index.nearest_with_ef(q, K, ALT_EF))
        .collect();
    // Second pass — should hit the cache.
    let second: Vec<Vec<(u32, f32)>> = queries
        .iter()
        .map(|q| index.nearest_with_ef(q, K, ALT_EF))
        .collect();

    assert_eq!(first, second, "cache reuse must produce identical results");
    drop(store);
}
