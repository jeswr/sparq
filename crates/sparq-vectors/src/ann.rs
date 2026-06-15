//! Nearest-neighbour search over a [`VectorStore`]: an exact brute-force baseline and
//! an HNSW index ([`instant-distance`], pure Rust).
//!
//! Similarity is **cosine** throughout. The HNSW index stores L2-normalized copies of
//! the vectors and searches with Euclidean distance, which is rank-equivalent to cosine
//! on unit vectors (`cos = 1 − d²/2`); reported scores are converted back to cosine, so
//! exact and approximate results are directly comparable.
//!
//! **Persistence**: the HNSW index is rebuilt from the mmap'd store on open rather than
//! persisted. `instant-distance`'s serde persistence would add serde + bincode and a
//! second versioned artifact; at the scales this v1 targets the rebuild is a one-off
//! cost per process (tens of seconds for 50k×32 on an M1, rayon-parallel — see the README
//! throughput table). Out-of-core persistent ANN (DiskANN-style) is the recorded
//! follow-up for 10M+ stores.

use crate::store::VectorStore;
use instant_distance::{Builder, HnswMap, Point, Search};
use oxrdf::Term;
use sparq_core::dict::Id;
use sparq_core::Graph;

/// Cosine similarity of two equal-length vectors, in `[-1, 1]` (0 if either is zero).
/// Eight independent accumulator lanes so the compiler auto-vectorizes the loop.
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "cosine over mismatched dims");
    const LANES: usize = 8;
    let mut dot = [0f32; LANES];
    let mut na = [0f32; LANES];
    let mut nb = [0f32; LANES];
    let chunks = a.len() / LANES;
    for c in 0..chunks {
        for l in 0..LANES {
            let (x, y) = (a[c * LANES + l], b[c * LANES + l]);
            dot[l] += x * y;
            na[l] += x * x;
            nb[l] += y * y;
        }
    }
    for i in chunks * LANES..a.len() {
        dot[0] += a[i] * b[i];
        na[0] += a[i] * a[i];
        nb[0] += b[i] * b[i];
    }
    let (dot, na, nb) =
        (dot.iter().sum::<f32>(), na.iter().sum::<f32>(), nb.iter().sum::<f32>());
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na.sqrt() * nb.sqrt())
    }
}

/// **Exact** top-`k` by cosine similarity: a full scan of the store. The ground-truth
/// baseline the HNSW recall gate measures against, and a fine default below ~10⁵
/// vectors. Ties break on ascending id, so results are deterministic. An all-zero
/// `query` has no direction (cosine is undefined), so it returns no results — stored
/// vectors are never zero ([`VectorStore::put`] rejects them), and [`VectorIndex`]
/// treats a zero query the same way, so exact and HNSW agree on the degenerate case.
pub fn nearest_exact(store: &VectorStore, query: &[f32], k: usize) -> Vec<(Id, f32)> {
    if query.iter().all(|&v| v == 0.0) {
        return Vec::new();
    }
    let mut scored: Vec<(Id, f32)> =
        store.iter().map(|(id, v)| (id, cosine(query, v))).collect();
    scored.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.truncate(k);
    scored
}

/// Like [`nearest_exact`] but query-by-term: resolves `term` through the graph's
/// dictionary, looks its vector up in the store, excludes the term itself from the
/// results and maps neighbor ids back to [`Term`]s. Empty if the term is absent from
/// the dictionary or has no vector.
///
/// [OPUS-4.8] (sq-32i5) This does NOT verify `store` matches `graph` — a store keyed by a
/// stale graph generation silently mis-resolves `term`. Use
/// [`nearest_term_exact_checked`] (or call [`VectorStore::check_graph`] once) to make a
/// mismatch a hard error.
pub fn nearest_term_exact(
    store: &VectorStore,
    graph: &Graph,
    term: &Term,
    k: usize,
) -> Vec<(Term, f32)> {
    let Some(id) = graph.id_of(term) else { return Vec::new() };
    let Some(query) = store.get(id) else { return Vec::new() };
    nearest_exact(store, query, k + 1)
        .into_iter()
        .filter(|&(n, _)| n != id)
        .take(k)
        .map(|(n, s)| (graph.dict.term(n), s))
        .collect()
}

/// [OPUS-4.8] (sq-32i5) [`nearest_term_exact`] with the staleness check: returns `Err` if
/// `store` was built against a different graph generation than `graph` (which would otherwise
/// return silently-wrong neighbours), else `Ok` with the neighbours.
pub fn nearest_term_exact_checked(
    store: &VectorStore,
    graph: &Graph,
    term: &Term,
    k: usize,
) -> Result<Vec<(Term, f32)>, String> {
    store.check_graph(graph)?;
    Ok(nearest_term_exact(store, graph, term, k))
}

/// HNSW construction/search parameters (passed through to `instant-distance`).
#[derive(Clone, Copy, Debug)]
pub struct HnswConfig {
    /// Beam width during search (the recall knob; must be ≥ the `k` you will query).
    pub ef_search: usize,
    /// Beam width during construction.
    pub ef_construction: usize,
    /// Level-assignment RNG seed — fixed by default so builds are reproducible.
    pub seed: u64,
}

impl Default for HnswConfig {
    fn default() -> Self {
        HnswConfig { ef_search: 100, ef_construction: 100, seed: 0x5350_5156_0001 }
    }
}

/// A normalized point in the HNSW graph. Euclidean distance over unit vectors is
/// rank-equivalent to cosine; see the module docs.
#[derive(Clone)]
struct NPoint(Vec<f32>);

impl Point for NPoint {
    fn distance(&self, other: &Self) -> f32 {
        const LANES: usize = 8;
        let (a, b) = (&self.0, &other.0);
        let mut acc = [0f32; LANES];
        let chunks = a.len() / LANES;
        for c in 0..chunks {
            for l in 0..LANES {
                let d = a[c * LANES + l] - b[c * LANES + l];
                acc[l] += d * d;
            }
        }
        for i in chunks * LANES..a.len() {
            let d = a[i] - b[i];
            acc[0] += d * d;
        }
        acc.iter().sum::<f32>().sqrt()
    }
}

/// L2-normalizes `v`; `None` for an all-zero vector (no direction). Stored vectors are
/// never zero ([`VectorStore::put`] rejects them), so `None` only arises for queries.
fn normalized(v: &[f32]) -> Option<Vec<f32>> {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    (norm > 0.0).then(|| v.iter().map(|x| x / norm).collect())
}

/// An in-RAM HNSW index over a [`VectorStore`], for approximate top-`k` at scales
/// where [`nearest_exact`]'s full scan is too slow. Build once per store generation
/// (rebuilt on open — see the module docs on persistence).
pub struct VectorIndex {
    map: HnswMap<NPoint, Id>,
    dim: usize,
}

impl VectorIndex {
    /// Builds the index over every vector in the store with default parameters
    /// (rayon-parallel inside `instant-distance`).
    pub fn build(store: &VectorStore) -> VectorIndex {
        VectorIndex::build_with(store, HnswConfig::default())
    }

    pub fn build_with(store: &VectorStore, cfg: HnswConfig) -> VectorIndex {
        let mut points = Vec::with_capacity(store.len());
        let mut values = Vec::with_capacity(store.len());
        for (id, v) in store.iter() {
            let n = normalized(v).expect("stores never hold zero vectors (put rejects them)");
            points.push(NPoint(n));
            values.push(id);
        }
        let map = Builder::default()
            .ef_search(cfg.ef_search)
            .ef_construction(cfg.ef_construction)
            .seed(cfg.seed)
            .build(points, values);
        VectorIndex { map, dim: store.dim() }
    }

    /// Approximate top-`k` ids by cosine similarity to `query`, best first.
    /// `k` is clamped by the index's `ef_search`. An all-zero `query` returns no
    /// results (same contract as [`nearest_exact`]).
    pub fn nearest(&self, query: &[f32], k: usize) -> Vec<(Id, f32)> {
        assert_eq!(query.len(), self.dim, "query dim {} != store dim {}", query.len(), self.dim);
        let Some(q) = normalized(query) else { return Vec::new() };
        let q = NPoint(q);
        let mut search = Search::default();
        self.map
            .search(&q, &mut search)
            .take(k)
            .map(|item| (*item.value, 1.0 - item.distance * item.distance / 2.0))
            .collect()
    }

    /// Approximate top-`k` neighbors of `term`: resolves it through the graph's
    /// dictionary, looks its vector up in `store`, excludes the term itself and maps
    /// neighbor ids back to [`Term`]s. Empty if the term is absent or unembedded.
    pub fn nearest_term(
        &self,
        term: &Term,
        graph: &Graph,
        store: &VectorStore,
        k: usize,
    ) -> Vec<(Term, f32)> {
        let Some(id) = graph.id_of(term) else { return Vec::new() };
        let Some(query) = store.get(id) else { return Vec::new() };
        self.nearest(query, k + 1)
            .into_iter()
            .filter(|&(n, _)| n != id)
            .take(k)
            .map(|(n, s)| (graph.dict.term(n), s))
            .collect()
    }

    /// [OPUS-4.8] (sq-32i5) [`nearest_term`](Self::nearest_term) with the staleness check: returns
    /// `Err` if `store` was built against a different graph generation than `graph` (which would
    /// otherwise return silently-wrong neighbours), else `Ok` with the neighbours. The HNSW index
    /// is rebuilt from `store` on open, so checking the store covers the index too.
    pub fn nearest_term_checked(
        &self,
        term: &Term,
        graph: &Graph,
        store: &VectorStore,
        k: usize,
    ) -> Result<Vec<(Term, f32)>, String> {
        store.check_graph(graph)?;
        Ok(self.nearest_term(term, graph, store, k))
    }
}
