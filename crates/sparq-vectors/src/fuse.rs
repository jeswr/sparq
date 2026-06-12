//! **Rank/score fusion** for hybrid retrieval: combine the text-vector ranking from
//! this crate with any other ranked signal — typically `sparq-sim`'s structural
//! similarity — without this crate depending on the other.
//!
//! Both helpers take plain ranked `(item, score)` lists (best first), so any producer
//! works: [`VectorIndex::nearest_term`](crate::VectorIndex::nearest_term) cosine,
//! `sparq_sim::Sim::most_similar` weighted Jaccard, a lexical/BM25 list, …
//!
//! Which to use (industry conventions distilled in
//! `research/genai-text-embedding-practices.md`):
//!
//! - [`fuse_rrf`] — **Reciprocal Rank Fusion**, `Σ 1/(k + rank)`, the standard
//!   `k = `[`RRF_K`]` = 60`. Uses only the *ranks*, so it needs no score
//!   normalization — the right default when the signals' scales differ (cosine in
//!   `[-1, 1]` vs Jaccard in `[0, 1]`).
//! - [`fuse_scores`] — **relative score fusion**: min-max normalizes each list to
//!   `[0, 1]`, then blends `alpha·a + (1 − alpha)·b`. Preserves score *magnitudes*
//!   (a runaway top hit stays a runaway), at the cost of an `alpha` to tune.
//!
//! ```
//! use sparq_vectors::{fuse_rrf, fuse_scores, RRF_K};
//!
//! let text: Vec<(&str, f64)> = vec![("a", 0.92), ("b", 0.85), ("c", 0.20)];
//! let structural: Vec<(&str, f64)> = vec![("b", 0.61), ("d", 0.55), ("a", 0.10)];
//!
//! let hybrid = fuse_rrf(&[&text, &structural], RRF_K, 3);
//! assert_eq!(hybrid[0].0, "b"); // ranked 2nd + 1st — consensus beats either alone
//!
//! let blended = fuse_scores(&text, &structural, 0.7, 3); // 70% text, 30% structural
//! assert_eq!(blended[0].0, "b"); // strong in both signals
//! assert_eq!(fuse_scores(&text, &structural, 1.0, 3)[0].0, "a"); // text only
//! ```

use rustc_hash::FxHashMap;
use std::hash::Hash;

/// The standard RRF rank constant (`k = 60`), empirically robust across datasets and
/// the default in Azure AI Search, Elasticsearch, MariaDB, … Lower `k` trusts each
/// list's top hit more; higher `k` rewards consensus across lists.
pub const RRF_K: f64 = 60.0;

/// **Reciprocal Rank Fusion** of any number of ranked lists (each best first):
/// `score(item) = Σ_lists 1 / (k + rank)`, ranks 1-based. Input scores are ignored —
/// only the order matters — so differently-scaled signals fuse without normalization.
/// Returns the top `top_k` fused items, best first; ties break by first appearance
/// (earlier list, then earlier rank), so the result is deterministic.
///
/// Use [`RRF_K`] for `k` unless you have a reason not to.
pub fn fuse_rrf<T: Clone + Eq + Hash>(
    lists: &[&[(T, f64)]],
    k: f64,
    top_k: usize,
) -> Vec<(T, f64)> {
    let weighted: Vec<(&[(T, f64)], f64)> = lists.iter().map(|&l| (l, 1.0)).collect();
    fuse_rrf_weighted(&weighted, k, top_k)
}

/// **Weighted Reciprocal Rank Fusion** (Elasticsearch-style): like
/// [`fuse_rrf`] but each list carries a non-negative weight —
/// `score(item) = Σ_lists weight_l / (k + rank_l)`, ranks 1-based. Use it to
/// down-weight a noisier signal without dropping it (e.g. text 1.0,
/// structural 0.5); weight 0 mutes a list entirely (its items still appear,
/// scored only by the other lists), and all-1.0 weights reduce to plain
/// [`fuse_rrf`]. Ties break by first appearance across `(list, rank)` order,
/// deterministically.
///
/// # Panics
/// If `k ≤ 0`, or any weight is negative or non-finite.
pub fn fuse_rrf_weighted<T: Clone + Eq + Hash>(
    lists: &[(&[(T, f64)], f64)],
    k: f64,
    top_k: usize,
) -> Vec<(T, f64)> {
    assert!(k > 0.0, "RRF k must be positive");
    assert!(
        lists.iter().all(|&(_, w)| w.is_finite() && w >= 0.0),
        "RRF list weights must be finite and non-negative"
    );
    // item -> (fused score, first-seen order for deterministic ties)
    let mut acc: FxHashMap<T, (f64, usize)> = FxHashMap::default();
    let mut order = 0usize;
    for (list, weight) in lists {
        for (rank0, (item, _)) in list.iter().enumerate() {
            let contribution = weight / (k + (rank0 + 1) as f64);
            let entry = acc.entry(item.clone()).or_insert_with(|| {
                order += 1;
                (0.0, order)
            });
            entry.0 += contribution;
        }
    }
    top_n(acc, top_k)
}

/// **Relative score fusion** of two ranked lists: min-max normalizes each list's
/// scores to `[0, 1]` independently (a constant list normalizes to all-1.0), then
/// scores the union as `alpha·norm_a + (1 − alpha)·norm_b`, items missing from a list
/// contributing 0 for it. Returns the top `top_k`, best first; ties break by first
/// appearance (list `a` order, then list `b`), deterministically.
///
/// `alpha` is the text-vs-other balance: `1.0` = only `a`, `0.0` = only `b`, `0.5` =
/// equal. Because both lists are normalized first, `alpha` compares like with like
/// even when the raw scales differ (cosine vs Jaccard). Prefer [`fuse_rrf`] when you
/// don't want to tune anything.
pub fn fuse_scores<T: Clone + Eq + Hash>(
    a: &[(T, f64)],
    b: &[(T, f64)],
    alpha: f64,
    top_k: usize,
) -> Vec<(T, f64)> {
    assert!((0.0..=1.0).contains(&alpha), "alpha must be in [0, 1]");
    let mut acc: FxHashMap<T, (f64, usize)> = FxHashMap::default();
    let mut order = 0usize;
    for (weight, list) in [(alpha, a), (1.0 - alpha, b)] {
        for (item, norm) in normalized(list) {
            let entry = acc.entry(item.clone()).or_insert_with(|| {
                order += 1;
                (0.0, order)
            });
            entry.0 += weight * norm;
        }
    }
    top_n(acc, top_k)
}

/// Min-max normalization of one list's scores to `[0, 1]`; all-1.0 when the scores
/// are constant (rank information is all there is).
fn normalized<T: Clone>(list: &[(T, f64)]) -> Vec<(T, f64)> {
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for (_, s) in list {
        lo = lo.min(*s);
        hi = hi.max(*s);
    }
    list.iter()
        .map(|(t, s)| {
            let n = if hi > lo { (s - lo) / (hi - lo) } else { 1.0 };
            (t.clone(), n)
        })
        .collect()
}

/// Sorts the accumulated `(score, first-seen)` map best-first (score desc, then
/// first-seen asc) and keeps `top_k`.
fn top_n<T>(acc: FxHashMap<T, (f64, usize)>, top_k: usize) -> Vec<(T, f64)> {
    let mut out: Vec<(T, f64, usize)> =
        acc.into_iter().map(|(t, (s, o))| (t, s, o)).collect();
    out.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.2.cmp(&y.2)));
    out.truncate(top_k);
    out.into_iter().map(|(t, s, _)| (t, s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rrf_hand_computed() {
        let a: Vec<(&str, f64)> = vec![("x", 0.9), ("y", 0.8)];
        let b: Vec<(&str, f64)> = vec![("y", 0.3), ("z", 0.2)];
        let fused = fuse_rrf(&[&a, &b], 60.0, 10);
        // y: 1/62 + 1/61; x: 1/61; z: 1/62.
        assert_eq!(fused[0].0, "y");
        assert!((fused[0].1 - (1.0 / 62.0 + 1.0 / 61.0)).abs() < 1e-12);
        assert_eq!(fused[1].0, "x");
        assert!((fused[1].1 - 1.0 / 61.0).abs() < 1e-12);
        assert_eq!(fused[2].0, "z");
        // Raw scores must not matter, only ranks.
        let a2: Vec<(&str, f64)> = vec![("x", 1000.0), ("y", -5.0)];
        let fused2 = fuse_rrf(&[&a2, &b], 60.0, 10);
        assert_eq!(
            fused.iter().map(|(t, _)| *t).collect::<Vec<_>>(),
            fused2.iter().map(|(t, _)| *t).collect::<Vec<_>>()
        );
    }

    #[test]
    fn rrf_ties_break_by_first_appearance_and_top_k_truncates() {
        let a: Vec<(&str, f64)> = vec![("x", 1.0)];
        let b: Vec<(&str, f64)> = vec![("y", 1.0)];
        // x and y both score 1/61; x appeared first.
        let fused = fuse_rrf(&[&a, &b], 60.0, 10);
        assert_eq!(fused[0].0, "x");
        assert_eq!(fused[1].0, "y");
        assert_eq!(fuse_rrf(&[&a, &b], 60.0, 1).len(), 1);
    }

    #[test]
    fn scores_min_max_normalize_then_blend() {
        // Different scales: cosine-ish [0.2, 0.9] vs jaccard-ish [0.0, 0.1].
        let text: Vec<(&str, f64)> = vec![("x", 0.9), ("y", 0.2)];
        let strct: Vec<(&str, f64)> = vec![("y", 0.1), ("x", 0.0)];
        // norm: text x=1 y=0; struct y=1 x=0.
        let fused = fuse_scores(&text, &strct, 0.7, 10);
        assert_eq!(fused[0].0, "x");
        assert!((fused[0].1 - 0.7).abs() < 1e-12);
        assert_eq!(fused[1].0, "y");
        assert!((fused[1].1 - 0.3).abs() < 1e-12);
        // alpha = 0.5 ties; first appearance (text order) wins deterministically.
        let even = fuse_scores(&text, &strct, 0.5, 10);
        assert_eq!(even[0].0, "x");
    }

    #[test]
    fn scores_missing_items_and_constant_lists() {
        let a: Vec<(&str, f64)> = vec![("x", 0.5), ("y", 0.5)]; // constant → both 1.0
        let b: Vec<(&str, f64)> = vec![("z", 0.4)]; // singleton → 1.0
        let fused = fuse_scores(&a, &b, 0.6, 10);
        assert_eq!(fused.len(), 3);
        assert_eq!(fused[0].0, "x"); // 0.6, ties with y, x first
        assert!((fused[0].1 - 0.6).abs() < 1e-12);
        assert_eq!(fused[2].0, "z"); // 0.4 from list b only
        assert!((fused[2].1 - 0.4).abs() < 1e-12);
    }

    #[test]
    #[should_panic(expected = "alpha must be in [0, 1]")]
    fn scores_rejects_bad_alpha() {
        let a: Vec<(&str, f64)> = vec![];
        let b: Vec<(&str, f64)> = vec![];
        fuse_scores(&a, &b, 1.5, 1);
    }

    #[test]
    fn weighted_rrf_hand_computed_and_reduces_to_plain() {
        let a: Vec<(&str, f64)> = vec![("x", 0.9), ("y", 0.8)];
        let b: Vec<(&str, f64)> = vec![("y", 0.3), ("z", 0.2)];
        // Unit weights ≡ plain RRF (scores and order).
        let plain = fuse_rrf(&[&a, &b], 60.0, 10);
        let unit = fuse_rrf_weighted(&[(&a, 1.0), (&b, 1.0)], 60.0, 10);
        assert_eq!(plain, unit);
        // Down-weight list b to 0.5: y = 1/61·0 + … hand-computed.
        let fused = fuse_rrf_weighted(&[(&a, 1.0), (&b, 0.5)], 60.0, 10);
        // x: 1/61; y: 1/62 + 0.5/61; z: 0.5/62.
        assert_eq!(fused[0].0, "y");
        assert!((fused[0].1 - (1.0 / 62.0 + 0.5 / 61.0)).abs() < 1e-12);
        assert_eq!(fused[1].0, "x");
        assert!((fused[1].1 - 1.0 / 61.0).abs() < 1e-12);
        assert_eq!(fused[2].0, "z");
        assert!((fused[2].1 - 0.5 / 62.0).abs() < 1e-12);
        // Weight strong enough to flip the consensus winner:
        // x = 3/61 ≈ 0.04918 must beat y = 3/62 + 0.01/61 ≈ 0.04855.
        let text_heavy = fuse_rrf_weighted(&[(&a, 3.0), (&b, 0.01)], 60.0, 10);
        assert_eq!(text_heavy[0].0, "x");
    }

    #[test]
    fn weighted_rrf_zero_weight_mutes_a_list() {
        let a: Vec<(&str, f64)> = vec![("x", 1.0)];
        let b: Vec<(&str, f64)> = vec![("y", 1.0), ("x", 0.5)];
        let fused = fuse_rrf_weighted(&[(&a, 0.0), (&b, 1.0)], 60.0, 10);
        // x still appears (rank 2 in b) but a contributes nothing.
        assert_eq!(fused[0].0, "y");
        assert!((fused[1].1 - 1.0 / 62.0).abs() < 1e-12);
    }

    #[test]
    #[should_panic(expected = "weights must be finite and non-negative")]
    fn weighted_rrf_rejects_negative_weight() {
        let a: Vec<(&str, f64)> = vec![("x", 1.0)];
        fuse_rrf_weighted(&[(&a, -0.1)], 60.0, 1);
    }
}
