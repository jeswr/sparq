//! Nearest-neighbour search over a [`VectorStore`]: an exact brute-force baseline and
//! an HNSW index (`instant-distance`, pure Rust).
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
//!
//! [OPUS-4.8] (sq-ip3a) **The HNSW index (`VectorIndex`) is gated behind the opt-in
//! `approx-ann` feature** — it is the only thing here that pulls the third-party
//! `instant-distance` crate, so with `approx-ann` OFF the default build carries the exact
//! brute-force searchers ([`nearest_exact`], [`nearest_term_exact`]) and NO heavy ANN
//! dependency. Approximate search is APPROXIMATE: its recall is `< 1.0` (measured against
//! [`nearest_exact`], the ground truth) — only the exact path is answer-exact.
//!
//! [SONNET-4.6] (sq-jo6ty) **Per-query `ef_search` (`nearest_with_ef`)**: `instant-distance`
//! encodes `ef_search` into `HnswMap` at build time and does not expose a per-search
//! override. `VectorIndex` therefore caches the L2-normalised `NPoint` vectors and builds
//! a secondary map on first use of each new `ef_search` value (same graph topology;
//! `ef_construction` and `seed` are unchanged). The secondary map is stored in a
//! `Mutex<HashMap<usize, HnswMap<…>>>` so sweeps amortise the rebuild cost: 100 queries
//! at `ef=16`, then 100 at `ef=32`, pay one extra build per ef level, not one per query.
//! `nearest_with_ef(q, k, ef)` where `ef == build_ef_search` is free (uses the primary map).

use crate::store::VectorStore;
#[cfg(feature = "approx-ann")]
use instant_distance::{Builder, HnswMap, Point, Search};
use oxrdf::Term;
use sparq_core::dict::Id;
use sparq_core::Graph;
#[cfg(feature = "approx-ann")]
use std::collections::HashMap;
#[cfg(feature = "approx-ann")]
use std::sync::{Arc, Mutex};

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
    let (dot, na, nb) = (
        dot.iter().sum::<f32>(),
        na.iter().sum::<f32>(),
        nb.iter().sum::<f32>(),
    );
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
/// vectors are never zero ([`VectorStore::put`] rejects them), and `VectorIndex`
/// treats a zero query the same way, so exact and HNSW agree on the degenerate case.
pub fn nearest_exact(store: &VectorStore, query: &[f32], k: usize) -> Vec<(Id, f32)> {
    if query.iter().all(|&v| v == 0.0) {
        return Vec::new();
    }
    let mut scored: Vec<(Id, f32)> = store.iter().map(|(id, v)| (id, cosine(query, v))).collect();
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
    let Some(id) = graph.id_of(term) else {
        return Vec::new();
    };
    let Some(query) = store.get(id) else {
        return Vec::new();
    };
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
/// [OPUS-4.8] (sq-ip3a) `approx-ann` only.
///
/// [OPUS-4.8] (sq-ose80) **`ef_construction` is the dominant BUILD-time knob.** The
/// `instant-distance` graph build is already `rayon`-parallel (per-layer
/// `into_par_iter`); its cost is dominated by the greedy distance search each insert runs,
/// and that search's beam width IS `ef_construction`. Roughly halving `ef_construction`
/// roughly halves build time. Lowering it also lowers the graph quality (fewer
/// back-links), so recall drops slightly — but on the measured corpora the drop stays well
/// inside the recall floor. Use [`fast_build`](Self::fast_build) when build latency
/// matters more than the last fraction of recall, [`high_recall`](Self::high_recall) for
/// the opposite. Both are **opt-in** presets; the [`Default`] is unchanged, so existing
/// callers keep exactly the same graph and recall.
#[cfg(feature = "approx-ann")]
#[derive(Clone, Copy, Debug)]
pub struct HnswConfig {
    /// Beam width during search (the recall knob; must be ≥ the `k` you will query).
    pub ef_search: usize,
    /// Beam width during construction — the dominant build-time knob (see the type docs).
    pub ef_construction: usize,
    /// Level-assignment RNG seed — fixed by default so builds are reproducible.
    pub seed: u64,
}

#[cfg(feature = "approx-ann")]
impl Default for HnswConfig {
    fn default() -> Self {
        HnswConfig {
            ef_search: 100,
            ef_construction: 100,
            seed: 0x5350_5156_0001,
        }
    }
}

#[cfg(feature = "approx-ann")]
impl HnswConfig {
    /// [OPUS-4.8] (sq-ose80) A **faster-build** preset: same `ef_search` and `seed` as the
    /// [`Default`], but a lower `ef_construction` (40 vs 100) so the graph build runs
    /// markedly faster at scale.
    ///
    /// **Why it exists.** The `instant-distance` build is already `rayon`-parallel, so the
    /// build-time gap vs a C++ HNSW is not a missing-parallelism bug — it is the per-insert
    /// greedy distance search, whose beam width is `ef_construction`. A narrower construction
    /// beam does less work per insert. On a 200k×128d cosine SIFT slice on the aarch64 work
    /// box (**NON-CANONICAL** timings — the ranking, not the absolute seconds, is what
    /// transfers) `ef_construction = 40` built in roughly a third of the `ef_construction =
    /// 100` time and still measured recall@10 = 0.9944 (vs 0.9990 at the default) — comfortably
    /// above the 0.95 floor the [`VectorIndex`] recall gate asserts. Prefer this when you rebuild
    /// the index often (e.g. per store generation) and a fraction of a percent of recall is an
    /// acceptable trade.
    ///
    /// This is a **pure config** preset: it adds no dependency, changes no default, and keeps
    /// the build deterministic for a fixed seed (the same seed yields the same graph). The
    /// `nearest` / `nearest_with_ef` query path and its monotone-recall contract are unchanged.
    pub fn fast_build() -> Self {
        HnswConfig {
            ef_construction: 40,
            ..HnswConfig::default()
        }
    }

    /// [OPUS-4.8] (sq-ose80) A **higher-recall** preset: same `ef_search` and `seed` as the
    /// [`Default`], but a wider `ef_construction` (200 vs 100) for a denser graph. This is the
    /// opposite trade to [`fast_build`](Self::fast_build): a wider construction beam links more
    /// back-neighbours, so recall rises, at a proportionally longer build. Prefer it for a
    /// build-once, query-forever index where the extra build time amortises. (Also a pure config
    /// preset — no dependency, no default change, deterministic for a fixed seed.)
    pub fn high_recall() -> Self {
        HnswConfig {
            ef_construction: 200,
            ..HnswConfig::default()
        }
    }
}

/// A normalized point in the HNSW graph. Euclidean distance over unit vectors is
/// rank-equivalent to cosine; see the module docs.
///
/// [FABLE-5] (sq-jk7w7) The vector data is behind an `Arc<[f32]>` so that `Clone` — which
/// `instant_distance::Point` requires, and which [`VectorIndex::build_with`] relies on to
/// retain the point set for lazy per-`ef_search` secondary maps — is a refcount bump, not a
/// deep copy. Before this, `build_with` deep-cloned the whole normalised point set into the
/// primary map (~512 MB extra peak at 1M×128d, doubling build peak memory; `instant-distance`
/// clones each point once more internally while shuffling, so the transient peak was 3×). The
/// distance kernel still sees a plain `&[f32]` (one indirection, same as `Vec`), so query and
/// build speed and determinism are unchanged.
#[cfg(feature = "approx-ann")]
#[derive(Clone)]
struct NPoint(Arc<[f32]>);

#[cfg(feature = "approx-ann")]
impl Point for NPoint {
    fn distance(&self, other: &Self) -> f32 {
        // [OPUS-4.8] (sq-lfo84) The HNSW build + search call this millions of times — it is the
        // dominant cost of both. Dispatch to the explicit NEON/AVX2 kernel (scalar fallback where
        // the vector ISA is absent), which measurably cuts build time and lifts QPS vs the previous
        // scalar-only loop. `instant-distance` wants the true Euclidean distance, so we `sqrt` the
        // squared kernel (sqrt is monotone, so `sqrt` itself introduces no reorder vs the squared
        // value). The SIMD kernel uses FMA (one rounding) vs the scalar `d*d`+`+=` (two roundings),
        // so a distance differs by <=1 ULP: rankings are stable up to exact near-ties, which the
        // HNSW recall FLOOR gate (tests/recall.rs, recall@10 >= 0.95) absorbs — NOT a bit-identity
        // claim. The reported cosine score is derived from `item.distance` exactly as before.
        crate::simd::l2_sq_dist(&self.0, &other.0).sqrt()
    }
}

/// L2-normalizes `v`; `None` for an all-zero vector (no direction). Stored vectors are
/// never zero ([`VectorStore::put`] rejects them), so `None` only arises for queries.
/// [FABLE-5] (sq-jk7w7) Returns the shared-ownership `Arc<[f32]>` form [`NPoint`] wraps.
#[cfg(feature = "approx-ann")]
fn normalized(v: &[f32]) -> Option<Arc<[f32]>> {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    (norm > 0.0).then(|| v.iter().map(|x| x / norm).collect())
}

/// An in-RAM HNSW index over a [`VectorStore`], for approximate top-`k` at scales
/// where [`nearest_exact`]'s full scan is too slow. Build once per store generation
/// (rebuilt on open — see the module docs on persistence).
///
/// [OPUS-4.8] (sq-ip3a) **APPROXIMATE** — recall `< 1.0`, gated behind the opt-in
/// `approx-ann` feature (the only thing that pulls `instant-distance`). For answer-exact
/// search use [`nearest_exact`]; this trades recall for speed at scale.
///
/// [SONNET-4.6] (sq-jo6ty) Exposes `nearest_with_ef` for per-query `ef_search` sweeps
/// (e.g. to build a recall–QPS Pareto curve at multiple ef levels without rebuilding the
/// whole store). The primary map (built at `HnswConfig::ef_search`) is used for
/// `nearest` and for `nearest_with_ef` when `ef == build_ef_search`. A secondary per-ef
/// map is built lazily and cached in `ef_cache` on first use of each new ef value.
#[cfg(feature = "approx-ann")]
pub struct VectorIndex {
    /// Primary HNSW map — built with the `HnswConfig::ef_search` value.
    map: HnswMap<NPoint, Id>,
    /// The `ef_search` the primary map was built with (needed for the cache short-circuit).
    ef_search_default: usize,
    /// The `ef_construction` / `seed` used to build the primary map (reused for secondary maps
    /// so that all ef levels share the same graph topology and seed).
    ef_construction: usize,
    seed: u64,
    /// L2-normalised points and their dict ids — retained so that secondary maps for different
    /// `ef_search` values can be built from the same normalised vectors (no store re-scan).
    /// [FABLE-5] (sq-jk7w7) Each [`NPoint`] is `Arc`-backed, so this retention (and every
    /// secondary-map build) SHARES the primary map's vector allocations — the per-index cost is
    /// one pointer-sized `Vec` per map, not a second copy of the float data.
    points: Vec<NPoint>,
    values: Vec<Id>,
    /// Lazily-built secondary maps keyed by `ef_search`. Populated on the first
    /// `nearest_with_ef` call at a new ef level; thereafter the cached map is reused.
    ef_cache: Mutex<HashMap<usize, HnswMap<NPoint, Id>>>,
    dim: usize,
}

#[cfg(feature = "approx-ann")]
impl VectorIndex {
    /// Builds the index over every vector in the store with default parameters
    /// (rayon-parallel inside `instant-distance`).
    pub fn build(store: &VectorStore) -> VectorIndex {
        VectorIndex::build_with(store, HnswConfig::default())
    }

    /// Builds the index with the given [`HnswConfig`].
    ///
    /// [FABLE-5] (sq-jk7w7) Peak memory is ONE copy of the L2-normalised point set plus
    /// pointer-sized bookkeeping: the internal `NPoint` is `Arc`-backed, so the `points.clone()` handed to
    /// the primary map shares the float allocations with the retained `self.points` (used for
    /// lazy per-`ef_search` secondary maps) instead of deep-copying them. Previously that clone
    /// duplicated the whole point set (~512 MB extra at 1M×128d), doubling build peak memory.
    pub fn build_with(store: &VectorStore, cfg: HnswConfig) -> VectorIndex {
        let mut points = Vec::with_capacity(store.len());
        let mut values = Vec::with_capacity(store.len());
        for (id, v) in store.iter() {
            let n = normalized(v).expect("stores never hold zero vectors (put rejects them)");
            points.push(NPoint(n));
            values.push(id);
        }
        // [FABLE-5] (sq-jk7w7) Cheap: clones Vecs of `Arc` handles + ids, NOT the vector data.
        let map = Builder::default()
            .ef_search(cfg.ef_search)
            .ef_construction(cfg.ef_construction)
            .seed(cfg.seed)
            .build(points.clone(), values.clone());
        VectorIndex {
            map,
            ef_search_default: cfg.ef_search,
            ef_construction: cfg.ef_construction,
            seed: cfg.seed,
            points,
            values,
            ef_cache: Mutex::new(HashMap::new()),
            dim: store.dim(),
        }
    }

    /// Approximate top-`k` ids by cosine similarity to `query`, best first.
    /// Uses the build-time `ef_search`. An all-zero `query` returns no
    /// results (same contract as [`nearest_exact`]).
    pub fn nearest(&self, query: &[f32], k: usize) -> Vec<(Id, f32)> {
        assert_eq!(
            query.len(),
            self.dim,
            "query dim {} != store dim {}",
            query.len(),
            self.dim
        );
        let Some(q) = normalized(query) else {
            return Vec::new();
        };
        let q = NPoint(q);
        let mut search = Search::default();
        self.map
            .search(&q, &mut search)
            .take(k)
            .map(|item| (*item.value, 1.0 - item.distance * item.distance / 2.0))
            .collect()
    }

    /// Approximate top-`k` ids with a **per-query** `ef_search` beam width.
    ///
    /// [SONNET-4.6] (sq-jo6ty) This is the recall–QPS sweep entry point: the caller can
    /// vary `ef_search` across queries to trace the recall–throughput Pareto frontier
    /// without rebuilding the index. The `ef_search` value controls the candidate beam at
    /// query time — a larger beam visits more neighbours and improves recall at the cost of
    /// throughput; a smaller beam is faster but may miss true top-`k` candidates.
    ///
    /// **Monotone-recall property**: for any fixed query and `k`, recall@k vs the exact
    /// oracle is non-decreasing as `ef_search` increases (sweeping upward can never lower
    /// recall). This follows from the HNSW algorithm: a wider beam subsumes the candidate
    /// set explored by any narrower beam over the same graph.
    ///
    /// When `ef_search == build_ef_search` (the value passed to [`HnswConfig`] at build
    /// time), the primary map is used directly — zero extra overhead. For any other value,
    /// a secondary map is built once and cached; subsequent calls at the same ef level
    /// reuse the cached map. All secondary maps share the same `ef_construction`, `seed`,
    /// and normalised point set as the primary map, so the graph topology is identical.
    ///
    /// **APPROXIMATE** — recall `< 1.0` at any finite `ef_search`; use [`nearest_exact`]
    /// for answer-exact results.
    pub fn nearest_with_ef(&self, query: &[f32], k: usize, ef_search: usize) -> Vec<(Id, f32)> {
        assert_eq!(
            query.len(),
            self.dim,
            "query dim {} != store dim {}",
            query.len(),
            self.dim
        );
        let Some(q) = normalized(query) else {
            return Vec::new();
        };
        let q = NPoint(q);
        let mut search = Search::default();
        // [SONNET-4.6] (sq-jo6ty) Short-circuit: the build-time ef → primary map (zero overhead).
        if ef_search == self.ef_search_default {
            return self
                .map
                .search(&q, &mut search)
                .take(k)
                .map(|item| (*item.value, 1.0 - item.distance * item.distance / 2.0))
                .collect();
        }
        // [SONNET-4.6] (sq-jo6ty) Non-default ef: check the cache, build and insert if absent,
        // then search with the lock held. The search is read-only on the map and only mutates
        // `search` (a local variable), so there is no correctness issue with holding the lock
        // here — throughput of the secondary path is a benchmarking sweep concern, not a
        // production hot path.
        let mut cache = self.ef_cache.lock().unwrap_or_else(|e| e.into_inner());
        let ef_construction = self.ef_construction;
        let seed = self.seed;
        let points = &self.points;
        let values = &self.values;
        cache.entry(ef_search).or_insert_with(|| {
            Builder::default()
                .ef_search(ef_search)
                .ef_construction(ef_construction)
                .seed(seed)
                .build(points.clone(), values.clone())
        });
        cache[&ef_search]
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
        let Some(id) = graph.id_of(term) else {
            return Vec::new();
        };
        let Some(query) = store.get(id) else {
            return Vec::new();
        };
        self.nearest(query, k + 1)
            .into_iter()
            .filter(|&(n, _)| n != id)
            .take(k)
            .map(|(n, s)| (graph.dict.term(n), s))
            .collect()
    }

    /// [OPUS-4.8] (sq-1wc1) **Predicate-constrained (filtered) top-`k`** over the HNSW index:
    /// returns only neighbours whose id the `mask` permits (the candidate id-set a SPARQL BGP
    /// selects). `instant-distance`'s adjacency is **not exposed**, so — unlike
    /// [`DiskAnnIndex::nearest_filtered`](crate::DiskAnnIndex::nearest_filtered), which can do
    /// predicate-agnostic graph traversal — the in-RAM HNSW cannot be walked with predicate-aware
    /// acceptance. This therefore uses the **exact pre-filter** strategy (scan only the masked ids):
    /// exact, and the right choice for a selective mask; for a broad mask over a large store prefer
    /// the on-disk index's filtered traversal. Empty mask / all-zero query → no results.
    #[cfg(feature = "filtered-ann")]
    pub fn nearest_filtered(
        &self,
        query: &[f32],
        mask: &crate::IdMask,
        store: &VectorStore,
        k: usize,
    ) -> Vec<(Id, f32)> {
        assert_eq!(
            query.len(),
            self.dim,
            "query dim {} != store dim {}",
            query.len(),
            self.dim
        );
        crate::filter::nearest_exact_filtered(store, query, mask, k)
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

#[cfg(test)]
mod tests {
    // [OPUS-4.8] Default-feature unit gate for the `cosine` primitive that the exact searcher, the
    // filtered pre-filter and the quantizers all rank with. The integration tests use it transitively
    // for ranking but never pin its documented numeric contract directly — these do, so a regression
    // in the zero-vector guard (which would emit a NaN instead of 0.0 and silently break every
    // ranking) or in the lane-tail handling is caught.
    use super::cosine;

    #[test]
    fn cosine_known_values_unit_orthogonal_and_opposite() {
        // Identical direction → 1, orthogonal → 0, opposite → −1 (length cancels: cosine is
        // magnitude-invariant, so 3·a and a give the same score).
        assert!((cosine(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < 1e-6);
        assert!(
            (cosine(&[1.0, 0.0], &[3.0, 0.0]) - 1.0).abs() < 1e-6,
            "cosine ignores magnitude"
        );
        assert!(
            cosine(&[1.0, 0.0], &[0.0, 1.0]).abs() < 1e-6,
            "orthogonal vectors are 0"
        );
        assert!(
            (cosine(&[1.0, 0.0], &[-1.0, 0.0]) + 1.0).abs() < 1e-6,
            "opposite vectors are -1"
        );
        // 45° between (1,0) and (1,1): cos = 1/√2.
        assert!((cosine(&[1.0, 0.0], &[1.0, 1.0]) - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
    }

    #[test]
    fn cosine_zero_vector_is_zero_not_nan() {
        // A zero vector has no direction — the documented contract is a defined `0.0`, NEVER a NaN
        // (which `dot / (0 * …)` would otherwise produce and which would poison `total_cmp` sorting).
        let z = [0.0f32, 0.0, 0.0];
        let v = [1.0f32, 2.0, 3.0];
        assert_eq!(cosine(&z, &v), 0.0, "zero on the left is 0, not NaN");
        assert_eq!(cosine(&v, &z), 0.0, "zero on the right is 0, not NaN");
        assert_eq!(cosine(&z, &z), 0.0, "both zero is 0, not NaN");
        assert!(!cosine(&z, &v).is_nan());
    }

    #[test]
    fn cosine_handles_a_non_lane_multiple_length() {
        // dim 11 is not a multiple of the 8 SIMD lanes, so the scalar tail loop must run. A vector
        // against itself is still exactly 1.0 — proving the tail accumulators are summed in.
        let v: Vec<f32> = (1..=11).map(|i| i as f32).collect();
        assert!(
            (cosine(&v, &v) - 1.0).abs() < 1e-5,
            "self-cosine over a tail length must be 1.0"
        );
        // And a length below one full lane (dim 3) goes entirely through the tail loop.
        let w = [2.0f32, -1.0, 4.0];
        assert!((cosine(&w, &w) - 1.0).abs() < 1e-6);
    }

    #[test]
    #[should_panic(expected = "cosine over mismatched dims")]
    fn cosine_panics_on_mismatched_dims() {
        // A dim mismatch is a programming error, not a silently-truncated comparison.
        cosine(&[1.0, 0.0, 0.0], &[1.0, 0.0]);
    }

    // [FABLE-5] (sq-jk7w7) Peak-memory contract of `build_with`: the point set handed to the
    // HNSW map must SHARE allocations with the retained `points` (for lazy per-ef secondary
    // maps), never deep-copy them — the deep copy doubled build peak memory (~512 MB extra at
    // 1M×128d). `Arc::strong_count` observes the sharing directly, so reintroducing a deep
    // copy (fresh buffers for the map, or for a secondary map) flips these asserts red
    // (mutation-checked: a deep-copying build fails with strong_count 1 vs 2).
    #[cfg(feature = "approx-ann")]
    mod point_set_sharing {
        use super::super::{HnswConfig, VectorIndex, VectorStore};
        use std::sync::Arc;

        #[test]
        fn build_shares_point_allocations_with_the_map_instead_of_deep_copying() {
            let path = std::env::temp_dir().join(format!(
                "sparq-vectors-arcshare-{}.spqv",
                std::process::id()
            ));
            let mut store = VectorStore::create(&path, 8).unwrap();
            for i in 0..64u32 {
                // Deterministic non-zero vectors; sparse ids exercise the id→slot index.
                let v: Vec<f32> = (0..8).map(|d| ((i + d + 1) as f32).sin() + 2.0).collect();
                store.put(i * 3 + 1, &v).unwrap();
            }
            store.finalize().unwrap();

            let index = VectorIndex::build_with(&store, HnswConfig::default());
            assert_eq!(index.points.len(), 64);
            for p in &index.points {
                assert_eq!(
                    Arc::strong_count(&p.0),
                    2,
                    "primary map must share each point's allocation with the retained set \
                     (strong_count 2 = retained + map); a deep copy leaves it at 1"
                );
            }

            // The lazily-built secondary map (non-default ef) must share too, not deep-copy.
            let q: Vec<f32> = (0..8).map(|d| ((d + 1) as f32).cos() + 2.0).collect();
            let default_ef = HnswConfig::default().ef_search;
            let via_default = index.nearest_with_ef(&q, 5, default_ef);
            let via_secondary = index.nearest_with_ef(&q, 5, default_ef + 7);
            for p in &index.points {
                assert_eq!(
                    Arc::strong_count(&p.0),
                    3,
                    "a secondary per-ef map must share the point allocations too"
                );
            }
            // Behavioural sanity: both ef levels answer over the same shared point set.
            assert_eq!(via_default.len(), 5);
            assert_eq!(via_secondary.len(), 5);

            let _ = std::fs::remove_file(&path);
        }
    }
}
