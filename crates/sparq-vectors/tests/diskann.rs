//! [OPUS-4.8] Accuracy + persistence gate for the on-disk Vamana index ([`DiskAnnIndex`]):
//!
//! 1. **recall@10 vs exact brute force** ≥ 0.90 on the same 50k synthetic set the HNSW gate
//!    (`tests/recall.rs`) uses — a persisted graph must find the true neighbours;
//! 2. **restart survival** — build → drop the handle → `open` a fresh one (no rebuild) and get
//!    byte-identical neighbours, the acceptance criterion of sq-7zc;
//! 3. **parity with the in-RAM searchers** on tiny separable clusters, and the degenerate
//!    all-zero-query contract shared with `nearest_exact` / `VectorIndex`.

use sparq_vectors::{nearest_exact, DiskAnnIndex, VamanaConfig, VectorStore};
// [OPUS-4.8] (sq-ip3a) HNSW is the approximate backend — `approx-ann` only (the disk-vs-HNSW
// parity test below is gated to match).
#[cfg(feature = "approx-ann")]
use sparq_vectors::VectorIndex;

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

fn tmp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("sparq-vectors-{name}-{}", std::process::id()))
}

#[test]
fn diskann_recall_at_10_vs_brute_force_on_50k() {
    const N: usize = 50_000;
    const DIM: usize = 32;
    const K: usize = 10;
    const QUERIES: usize = 100;

    let store_path = tmp("diskann-recall.spqv");
    let graph_path = tmp("diskann-recall.spqg");
    let mut store = VectorStore::create(&store_path, DIM).unwrap();
    let mut state = 0xC0FFEE_u64;
    for i in 0..N {
        // Sparse, non-contiguous ids to exercise the id mapping (same as the HNSW gate).
        let id = (i as u32) * 7 + 3;
        store.put(id, &rand_vec(&mut state, DIM)).unwrap();
    }
    store.finalize().unwrap();

    let index = DiskAnnIndex::build(&store, &graph_path).unwrap();
    assert_eq!(index.len(), N);
    assert_eq!(index.dim(), DIM);

    let mut q_state = 0xDECAF_u64;
    let mut hits = 0usize;
    for _ in 0..QUERIES {
        let q = rand_vec(&mut q_state, DIM);
        let exact: Vec<u32> =
            nearest_exact(&store, &q, K).into_iter().map(|(id, _)| id).collect();
        let approx: Vec<u32> = index.nearest(&q, K).into_iter().map(|(id, _)| id).collect();
        assert_eq!(exact.len(), K);
        assert_eq!(approx.len(), K);
        hits += approx.iter().filter(|id| exact.contains(id)).count();
    }
    let recall = hits as f64 / (QUERIES * K) as f64;
    eprintln!("DiskANN recall@{K} over {QUERIES} queries on {N}x{DIM}: {recall:.4}");
    assert!(recall >= 0.90, "recall@{K} gate failed: {recall:.4} < 0.90");

    let _ = std::fs::remove_file(&store_path);
    let _ = std::fs::remove_file(&graph_path);
}

#[test]
fn reopen_without_rebuild_returns_identical_neighbours() {
    // The sq-7zc acceptance criterion: a versioned on-disk artifact opened WITHOUT a rebuild,
    // with recall parity to the built index. We build once, capture results, drop the handle
    // (simulating process exit), then `open` a fresh handle off the file alone and assert the
    // neighbours+scores match exactly — proving the graph round-tripped through disk intact.
    const N: usize = 5_000;
    const DIM: usize = 24;
    const K: usize = 10;

    let store_path = tmp("diskann-reopen.spqv");
    let graph_path = tmp("diskann-reopen.spqg");
    let mut store = VectorStore::create(&store_path, DIM).unwrap();
    let mut state = 0x1234_5678_u64;
    for i in 0..N {
        store.put((i as u32) * 3 + 1, &rand_vec(&mut state, DIM)).unwrap();
    }
    store.finalize().unwrap();

    let mut q_state = 0xFACE_u64;
    let queries: Vec<Vec<f32>> = (0..50).map(|_| rand_vec(&mut q_state, DIM)).collect();

    // Build, query, then drop the built handle entirely.
    let built_results: Vec<Vec<(u32, f32)>> = {
        let built = DiskAnnIndex::build_with(&store, &graph_path, VamanaConfig::default()).unwrap();
        queries.iter().map(|q| built.nearest(q, K)).collect()
    };

    // Reopen from the file alone — NO rebuild — and re-query.
    let reopened = DiskAnnIndex::open(&graph_path).unwrap();
    assert_eq!(reopened.len(), N);
    assert_eq!(reopened.dim(), DIM);
    for (q, expected) in queries.iter().zip(&built_results) {
        let got = reopened.nearest(q, K);
        assert_eq!(got.len(), expected.len());
        for (g, e) in got.iter().zip(expected) {
            assert_eq!(g.0, e.0, "reopened neighbour id differs from the built index");
            assert!(
                (g.1 - e.1).abs() < 1e-6,
                "reopened cosine {} differs from built {}",
                g.1,
                e.1
            );
        }
    }

    let _ = std::fs::remove_file(&store_path);
    let _ = std::fs::remove_file(&graph_path);
}

#[cfg(feature = "approx-ann")]
#[test]
fn diskann_parity_with_hnsw_on_separable_clusters() {
    // Three well-separated clusters; the on-disk index, the in-RAM HNSW, and exact brute force
    // must all return the query's own cluster, with cosine scores in agreement.
    const DIM: usize = 8;
    let store_path = tmp("diskann-tiny.spqv");
    let graph_path = tmp("diskann-tiny.spqg");
    let mut store = VectorStore::create(&store_path, DIM).unwrap();
    let mut state = 7_u64;
    let mut centers = Vec::new();
    for c in 0..3 {
        let center = rand_vec(&mut state, DIM);
        for i in 0..20 {
            let v: Vec<f32> = center
                .iter()
                .map(|x| {
                    x + ((splitmix64(&mut state) >> 40) as f32 / (1u64 << 23) as f32 - 0.5) * 0.05
                })
                .collect();
            store.put((c * 100 + i) as u32 + 1, &v).unwrap();
        }
        centers.push(center);
    }
    store.finalize().unwrap();

    // Small graphs: degree must stay ≤ build_beam, and both ≤ the cluster sizes are fine.
    let cfg = VamanaConfig { degree: 16, build_beam: 32, search_beam: 32, ..Default::default() };
    let disk = DiskAnnIndex::build_with(&store, &graph_path, cfg).unwrap();
    let hnsw = VectorIndex::build(&store);

    for (c, center) in centers.iter().enumerate() {
        let exact = nearest_exact(&store, center, 5);
        let approx_disk = disk.nearest(center, 5);
        let approx_hnsw = hnsw.nearest(center, 5);
        for &(id, sim) in exact.iter().chain(approx_disk.iter()).chain(approx_hnsw.iter()) {
            assert_eq!((id - 1) / 100, c as u32, "neighbor {id} from wrong cluster");
            assert!(sim > 0.99, "cluster member should be near-identical, got {sim}");
        }
        // The on-disk index returns the same id set as the in-RAM HNSW on separable data.
        let disk_ids: std::collections::HashSet<u32> =
            approx_disk.iter().map(|&(id, _)| id).collect();
        let hnsw_ids: std::collections::HashSet<u32> =
            approx_hnsw.iter().map(|&(id, _)| id).collect();
        assert_eq!(disk_ids, hnsw_ids, "on-disk and HNSW disagree on separable cluster {c}");
    }

    // Degenerate: an all-zero query has no direction → empty, matching the other searchers.
    let zero = vec![0.0f32; DIM];
    assert!(disk.nearest(&zero, 5).is_empty());

    let _ = std::fs::remove_file(&store_path);
    let _ = std::fs::remove_file(&graph_path);
}

#[test]
fn empty_and_singleton_graphs_roundtrip() {
    let store_path = tmp("diskann-empty.spqv");
    let graph_path = tmp("diskann-empty.spqg");

    // Empty store → empty graph; opens, searches return nothing.
    let mut store = VectorStore::create(&store_path, 8).unwrap();
    store.finalize().unwrap();
    let idx = DiskAnnIndex::build(&store, &graph_path).unwrap();
    assert!(idx.is_empty());
    assert!(idx.nearest(&[1.0; 8], 5).is_empty());
    let reopened = DiskAnnIndex::open(&graph_path).unwrap();
    assert!(reopened.is_empty());

    // Singleton store → the one vector is its own nearest neighbour.
    let store_path1 = tmp("diskann-one.spqv");
    let graph_path1 = tmp("diskann-one.spqg");
    let mut one = VectorStore::create(&store_path1, 4).unwrap();
    one.put(99, &[1.0, 2.0, 3.0, 4.0]).unwrap();
    one.finalize().unwrap();
    let idx1 = DiskAnnIndex::build(&one, &graph_path1).unwrap();
    let nn = idx1.nearest(&[1.0, 2.0, 3.0, 4.0], 5);
    assert_eq!(nn.len(), 1);
    assert_eq!(nn[0].0, 99);
    assert!((nn[0].1 - 1.0).abs() < 1e-5, "self-similarity should be ~1, got {}", nn[0].1);

    for p in [&store_path, &graph_path, &store_path1, &graph_path1] {
        let _ = std::fs::remove_file(p);
    }
}

#[test]
fn open_rejects_corrupt_and_wrong_magic() {
    let path = tmp("diskann-bad.spqg");
    // Wrong magic.
    std::fs::write(&path, b"NOPExxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").unwrap();
    assert!(DiskAnnIndex::open(&path).is_err());
    // Truncated header.
    std::fs::write(&path, b"SPQG").unwrap();
    assert!(DiskAnnIndex::open(&path).is_err());
    let _ = std::fs::remove_file(&path);
}
