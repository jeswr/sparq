// [OPUS-4.8] sq-jxl0: single-source the crate overview from README.md so crates.io
// (package.readme) and the docs.rs front page render identical content. The README's
// rust fences are API-map sketches marked `ignore` (they reference an external graph and
// the sibling `sparq-vectors` crate, which is not a dependency of `sparq-sim`).
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

use oxrdf::{NamedNode, Term};
use rustc_hash::FxHashMap;
#[cfg(feature = "multi-hop")]
use rustc_hash::FxHashSet;
use sparq_core::dict::{self, Id, NO_ID};
use sparq_core::store::Pattern;
use sparq_core::Graph;

/// What a signature element records about each adjacent triple.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SignatureMode {
    /// `(direction, predicate, neighbor)` — entities are similar when they relate to the
    /// SAME neighbors via the same predicates (co-citation / shared-context similarity).
    /// The default, and the mode `most_similar`'s index-driven candidate generation is
    /// designed around.
    #[default]
    PredicateNeighbor,
    /// `(direction, predicate)` only — entities are similar when they are USED the same
    /// way, regardless of which neighbors they touch (predicate-profile / role
    /// similarity, a characteristic-set overlap).
    Predicates,
}

/// Configuration for a [`Sim`] instance.
#[derive(Clone, Debug)]
pub struct SimConfig {
    pub mode: SignatureMode,
    /// Weigh each element by its predicate's inverse frequency (`1 + ln(|G|/freq(p))`),
    /// from the store's existing per-predicate stats. `false` = plain Jaccard.
    pub idf: bool,
    /// Predicates to EXCLUDE from signatures entirely (e.g. `rdf:type` when evaluating
    /// against type ground truth, or noisy provenance predicates).
    pub exclude_predicates: Vec<NamedNode>,
    /// Candidate-generation cap: signature elements matched by more than this many
    /// triples are SKIPPED during `most_similar` candidate generation (they are the
    /// lowest-IDF, least-informative elements, and scanning a hub's full extent would
    /// dominate the cost). Exact similarity scoring is unaffected; see
    /// [`Sim::most_similar`] for what this approximates. Default: 10 000.
    pub max_pair_frequency: usize,
    /// Neighbor-sparse fallback (default `true`): when `most_similar`'s
    /// index-driven `(predicate, neighbor)` candidate generation yields fewer than
    /// `k` results — typical for entities whose signature elements all point at
    /// degree-1 neighbors (every event names exactly one sport) — fill the remaining
    /// slots with [`SignatureMode::Predicates`]-style candidates, ranked below every
    /// exactly-scored result and scored by the predicate-profile Jaccard. `false`
    /// restores the strict v1 behaviour (only neighbor-level evidence is returned).
    pub profile_fallback: bool,
    /// Maximum structural-neighborhood depth. The default `1` preserves the original
    /// adjacent-triple signature exactly. Values greater than `1` expand breadth-first,
    /// retaining first-hop elements at full weight and attenuating hop `h` by
    /// `0.5^(h - 1)`. A value of `0` is treated as `1`.
    #[cfg(feature = "multi-hop")]
    pub depth: usize,
}

impl Default for SimConfig {
    fn default() -> Self {
        SimConfig {
            mode: SignatureMode::PredicateNeighbor,
            idf: true,
            exclude_predicates: Vec::new(),
            max_pair_frequency: 10_000,
            profile_fallback: true,
            #[cfg(feature = "multi-hop")]
            depth: 1,
        }
    }
}

/// Fixed per-hop attenuation used by the `multi-hop` feature.
#[cfg(feature = "multi-hop")]
const MULTI_HOP_ATTENUATION: f64 = 0.5;

/// Edge direction within a signature element (outgoing: the entity is the subject;
/// incoming: the entity is the object). Direction is part of the element — "directs
/// films" and "is directed by" are different roles.
const OUT: u8 = 0;
const IN: u8 = 1;

/// An entity's structural signature: its sorted, deduplicated `(direction, predicate,
/// neighbor)` elements (neighbor = [`NO_ID`] in [`SignatureMode::Predicates`]) with one
/// weight per element. Build via [`Sim::signature`]; compare via [`Sim::similarity`] /
/// [`Sim::similar_by_signature`].
#[derive(Clone, Debug)]
pub struct Signature {
    /// Sorted ascending, unique.
    elems: Vec<(u8, Id, Id)>,
    /// Parallel to `elems`.
    weights: Vec<f64>,
    /// Sum of `weights`.
    total: f64,
}

impl Signature {
    pub fn len(&self) -> usize {
        self.elems.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }

    /// The sum of element weights (the weighted-Jaccard denominator contribution).
    pub fn total_weight(&self) -> f64 {
        self.total
    }
}

/// Structural similarity over a graph. Cheap to construct ([`Sim::new`] resolves the
/// excluded predicates and snapshots the triple count; predicate frequencies come from
/// the store's already-computed planner stats, O(1) per lookup).
pub struct Sim<'g> {
    graph: &'g Graph,
    mode: SignatureMode,
    idf: bool,
    excluded: Vec<Id>,
    max_pair_frequency: usize,
    profile_fallback: bool,
    #[cfg(feature = "multi-hop")]
    depth: usize,
    /// |G| as f64, the IDF numerator.
    n_triples: f64,
}

impl<'g> Sim<'g> {
    /// A `Sim` with the default configuration (predicate+neighbor signatures, pred-IDF
    /// weighting on).
    pub fn new(graph: &'g Graph) -> Self {
        Self::with_config(graph, SimConfig::default())
    }

    pub fn with_config(graph: &'g Graph, cfg: SimConfig) -> Self {
        // Resolve excluded predicates to ids once; absent predicates can never occur.
        let mut excluded: Vec<Id> = cfg
            .exclude_predicates
            .iter()
            .filter_map(|p| graph.id_of(&Term::NamedNode(p.clone())))
            .collect();
        excluded.sort_unstable();
        Sim {
            graph,
            mode: cfg.mode,
            idf: cfg.idf,
            excluded,
            max_pair_frequency: cfg.max_pair_frequency,
            profile_fallback: cfg.profile_fallback,
            #[cfg(feature = "multi-hop")]
            depth: cfg.depth.max(1),
            n_triples: graph.len().max(1) as f64,
        }
    }

    /// The weight of a signature element with predicate `p`: `1 + ln(|G| / freq(p))`
    /// when IDF weighting is on (freq from the store's per-predicate planner stats — no
    /// scan), else 1. Monotonically decreasing in predicate frequency, ≥ 1.
    #[inline]
    fn weight(&self, p: Id) -> f64 {
        if !self.idf {
            return 1.0;
        }
        let freq = self.graph.store.pred_stat(p).map_or(1, |s| s.count.max(1));
        1.0 + (self.n_triples / freq as f64).ln().max(0.0)
    }

    #[inline]
    fn is_excluded(&self, p: Id) -> bool {
        self.excluded.binary_search(&p).is_ok()
    }

    /// The structural signature of `term`, or `None` if the term is not in the graph's
    /// dictionary. `O(log n + degree)`: one SPO range scan (outgoing) + one OSP/OPS
    /// range scan (incoming).
    pub fn signature(&self, term: &Term) -> Option<Signature> {
        let id = self.graph.id_of(term)?;
        Some(self.signature_of_id(id))
    }

    fn signature_of_id(&self, id: Id) -> Signature {
        self.signature_of_id_mode(id, self.mode == SignatureMode::PredicateNeighbor)
    }

    fn signature_of_id_mode(&self, id: Id, keep_neighbor: bool) -> Signature {
        #[cfg(feature = "multi-hop")]
        if self.depth > 1 {
            return self.expanded_signature_of_id_mode(id, keep_neighbor);
        }

        self.one_hop_signature_of_id_mode(id, keep_neighbor)
    }

    /// The original depth-1 implementation stays on its own path so the default feature
    /// state and `depth = 1` retain byte-identical floating-point operations and ordering.
    fn one_hop_signature_of_id_mode(&self, id: Id, keep_neighbor: bool) -> Signature {
        let mut elems: Vec<(u8, Id, Id)> = Vec::new();
        // Outgoing: (id, p, o) — one contiguous SPO range.
        let scan = self.graph.store.scan(&[Some(id), None, None]);
        for row in scan.rows.iter() {
            let [_, p, o] = scan.to_spo(row);
            if !self.is_excluded(p) {
                elems.push((OUT, p, if keep_neighbor { o } else { NO_ID }));
            }
        }
        // Incoming: (s, p, id) — one contiguous OSP/OPS range.
        let scan = self.graph.store.scan(&[None, None, Some(id)]);
        for row in scan.rows.iter() {
            let [s, p, _] = scan.to_spo(row);
            if !self.is_excluded(p) {
                elems.push((IN, p, if keep_neighbor { s } else { NO_ID }));
            }
        }
        elems.sort_unstable();
        elems.dedup();
        let weights: Vec<f64> = elems.iter().map(|&(_, p, _)| self.weight(p)).collect();
        let total = weights.iter().sum();
        Signature { elems, weights, total }
    }

    /// Breadth-first expansion for the opt-in `multi-hop` feature. Each layer scans in
    /// canonical id order; previously visited nodes are not reintroduced by backtracking.
    /// When several paths produce one element, the nearest (largest) weight wins.
    #[cfg(feature = "multi-hop")]
    fn expanded_signature_of_id_mode(&self, id: Id, keep_neighbor: bool) -> Signature {
        let mut element_weights: FxHashMap<(u8, Id, Id), f64> = FxHashMap::default();
        let mut visited: FxHashSet<Id> = FxHashSet::default();
        visited.insert(id);
        let mut frontier = vec![id];
        let mut attenuation = 1.0;

        for hop in 1..=self.depth {
            let mut next_frontier = Vec::new();
            for center in frontier {
                let outgoing = self.graph.store.scan(&[Some(center), None, None]);
                for row in outgoing.rows.iter() {
                    let [_, p, neighbor] = outgoing.to_spo(row);
                    if !self.is_excluded(p) && (hop == 1 || !visited.contains(&neighbor)) {
                        let elem = (OUT, p, if keep_neighbor { neighbor } else { NO_ID });
                        let weight = self.weight(p) * attenuation;
                        element_weights
                            .entry(elem)
                            .and_modify(|current| *current = current.max(weight))
                            .or_insert(weight);
                    }
                    if !self.is_excluded(p) && !visited.contains(&neighbor) {
                        next_frontier.push(neighbor);
                    }
                }

                let incoming = self.graph.store.scan(&[None, None, Some(center)]);
                for row in incoming.rows.iter() {
                    let [neighbor, p, _] = incoming.to_spo(row);
                    if !self.is_excluded(p) && (hop == 1 || !visited.contains(&neighbor)) {
                        let elem = (IN, p, if keep_neighbor { neighbor } else { NO_ID });
                        let weight = self.weight(p) * attenuation;
                        element_weights
                            .entry(elem)
                            .and_modify(|current| *current = current.max(weight))
                            .or_insert(weight);
                    }
                    if !self.is_excluded(p) && !visited.contains(&neighbor) {
                        next_frontier.push(neighbor);
                    }
                }
            }

            if next_frontier.is_empty() {
                break;
            }
            next_frontier.sort_unstable();
            next_frontier.dedup();
            visited.extend(next_frontier.iter().copied());
            frontier = next_frontier;
            attenuation *= MULTI_HOP_ATTENUATION;
        }

        let mut weighted: Vec<((u8, Id, Id), f64)> = element_weights.into_iter().collect();
        weighted.sort_unstable_by_key(|&(elem, _)| elem);
        let (elems, weights): (Vec<_>, Vec<_>) = weighted.into_iter().unzip();
        let total = weights.iter().sum();
        Signature {
            elems,
            weights,
            total,
        }
    }

    /// Weighted Jaccard similarity of two terms' structural signatures, in `[0, 1]`:
    /// `Σ w(e) over A∩B / Σ w(e) over A∪B`. 0 if either term is absent from the graph
    /// or both signatures are empty; 1 iff the (non-empty) signatures are identical.
    /// Symmetric. Cost: two signature builds + one sorted merge.
    pub fn similarity(&self, a: &Term, b: &Term) -> f64 {
        let (Some(a), Some(b)) = (self.graph.id_of(a), self.graph.id_of(b)) else {
            return 0.0;
        };
        weighted_jaccard(&self.signature_of_id(a), &self.signature_of_id(b))
    }

    /// The `k` entities most similar to `a` (excluding `a` itself), best first.
    ///
    /// **Candidate generation is index-driven, not a full scan.** For each element of
    /// `a`'s signature, the entities sharing it form one contiguous index range:
    /// - outgoing `(p, n)`: subjects of `(?, p, n)` — a POS range scan;
    /// - incoming `(p, n)`: objects of `(n, p, ?)` — an SPO range scan;
    /// - [`SignatureMode::Predicates`]: the distinct subjects (PSO) / objects (POS) of
    ///   the predicate's whole block.
    ///
    /// Candidates accumulate the weight of each element they share (an upper bound on
    /// their intersection weight); the top `max(4k, 64)` accumulated candidates are then
    /// re-scored EXACTLY with [`similarity`](Self::similarity) semantics.
    ///
    /// **Complexity** `O( Σ_e min(freq(e), F) )` for generation over the signature's
    /// elements `e` (each a binary search + contiguous range read), plus
    /// `O( M · (log n + d̄) )` for re-scoring `M = max(4k, 64)` candidates of average
    /// degree `d̄`.
    ///
    /// **Approximation:** elements matched by more than `max_pair_frequency` (`F`)
    /// triples are skipped during *generation* (hub elements — the lowest-IDF, least
    /// informative, and the only unbounded cost). An entity is therefore guaranteed to
    /// be found iff it shares at least one sub-cap element with `a`; re-scoring is
    /// always exact over full signatures. Raise the cap (or set it to `usize::MAX`) for
    /// exact-but-slower candidate generation.
    ///
    /// **Neighbor-sparse fallback** (on by default, [`SimConfig::profile_fallback`]):
    /// when generation yields fewer than `k` candidates, the remaining slots are
    /// filled with entities sharing a `(direction, predicate)` ROLE with `a`,
    /// ranked below every exactly-scored result and scored by the predicate-profile
    /// Jaccard (what [`similarity`](Self::similarity) would return in
    /// [`SignatureMode::Predicates`]). Fallback scores are comparable among
    /// themselves, not with the exactly-scored block.
    pub fn most_similar(&self, a: &Term, k: usize) -> Vec<(Term, f64)> {
        let Some(id) = self.graph.id_of(a) else {
            return Vec::new();
        };
        let sig = self.signature_of_id(id);
        self.similar_by_sig(&sig, k, Some(id))
    }

    /// Like [`most_similar`](Self::most_similar) but for an arbitrary (possibly
    /// synthetic) signature — e.g. a probe built from a handful of (predicate, neighbor)
    /// constraints, or a signature carried over from another graph.
    pub fn similar_by_signature(&self, sig: &Signature, k: usize) -> Vec<(Term, f64)> {
        self.similar_by_sig(sig, k, None)
    }

    fn similar_by_sig(&self, sig: &Signature, k: usize, exclude: Option<Id>) -> Vec<(Term, f64)> {
        if k == 0 || sig.is_empty() {
            return Vec::new();
        }
        // 1. Accumulate, per candidate entity, the total weight of shared elements.
        let mut acc: FxHashMap<Id, f64> = FxHashMap::default();
        for (i, &(dir, p, n)) in sig.elems.iter().enumerate() {
            let w = sig.weights[i];
            // The candidates sharing this element are one contiguous range; which
            // pattern (and which row column holds the candidate) depends on the
            // direction and mode.
            let (pattern, sorted_by): (Pattern, usize) = match (self.mode, dir) {
                // Who else has (p, n) outgoing? subjects of (?, p, n) — POS range.
                (SignatureMode::PredicateNeighbor, OUT) => ([None, Some(p), Some(n)], 0),
                // Who else has (p, n) incoming? objects of (n, p, ?) — SPO range.
                (SignatureMode::PredicateNeighbor, _) => ([Some(n), Some(p), None], 2),
                // Predicate-profile mode: every subject / object of p's block.
                (SignatureMode::Predicates, OUT) => ([None, Some(p), None], 0),
                (SignatureMode::Predicates, _) => ([None, Some(p), None], 2),
            };
            if self.graph.store.estimate(&pattern) > self.max_pair_frequency {
                continue; // hub element: skipped (documented approximation)
            }
            // `scan_sorted` orders rows by the candidate column, so duplicates (one
            // subject with many objects under p, etc.) are consecutive — dedup as we go
            // so each candidate receives this element's weight exactly once.
            let scan = self.graph.store.scan_sorted(&pattern, sorted_by);
            let mut last: Option<Id> = None;
            for row in scan.rows.iter() {
                let cand = scan.to_spo(row)[sorted_by];
                if last == Some(cand) {
                    continue;
                }
                last = Some(cand);
                if Some(cand) == exclude || !self.is_entity(cand) {
                    continue;
                }
                *acc.entry(cand).or_insert(0.0) += w;
            }
        }
        // 2. Keep the strongest M candidates by accumulated (upper-bound) weight …
        let m = (4 * k).max(64);
        let mut cands: Vec<(Id, f64)> = acc.into_iter().collect();
        cands.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        cands.truncate(m);
        // 3. … and re-score them exactly against the full signature.
        let mut scored: Vec<(Id, f64)> = cands
            .into_iter()
            .map(|(c, _)| (c, weighted_jaccard(sig, &self.signature_of_id(c))))
            .collect();
        scored.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        scored.truncate(k);
        // 4. Neighbor-sparse fallback: when index-driven generation starves
        // (fewer than k candidates), fill the remaining slots with
        // predicate-PROFILE matches (see `profile_fallback`).
        if self.profile_fallback && self.mode == SignatureMode::PredicateNeighbor && scored.len() < k
        {
            self.fill_profile_fallback(sig, k, exclude, &mut scored);
        }
        scored.into_iter().map(|(c, s)| (self.graph.dict.term(c), s)).collect()
    }

    /// The neighbor-sparse fallback of [`most_similar`](Self::most_similar):
    /// when `(direction, predicate, neighbor)` candidate generation yields
    /// fewer than `k` results (every signature element points at degree-1
    /// neighbors — e.g. each event names exactly one sport, so the reverse
    /// scan from each neighbor finds only the query entity itself), the
    /// remaining slots are filled by [`SignatureMode::Predicates`]-style
    /// generation: entities sharing a `(direction, predicate)` ROLE with the
    /// query, scored by the **predicate-profile weighted Jaccard** (the score
    /// `similarity` would return in `Predicates` mode).
    ///
    /// Fallback entries always rank BELOW every exactly-scored primary result
    /// (neighbor-level evidence outranks role-only evidence), and their scores
    /// are profile Jaccards — comparable among themselves, not with the
    /// primary block. Predicate blocks are scanned most-selective-first and
    /// generation stops once `max(4k, 64)` fallback candidates accumulate, so
    /// a hub predicate is only scanned when the starvation is real and nothing
    /// cheaper fills the budget.
    fn fill_profile_fallback(
        &self,
        sig: &Signature,
        k: usize,
        exclude: Option<Id>,
        scored: &mut Vec<(Id, f64)>,
    ) {
        let have: rustc_hash::FxHashSet<Id> = scored.iter().map(|&(c, _)| c).collect();
        // Distinct (direction, predicate) roles of the query's signature,
        // most selective predicate block first.
        let mut roles: Vec<(u8, Id, usize)> = Vec::new();
        for &(dir, p, _) in sig.elems.iter() {
            if roles.iter().all(|&(d, q, _)| (d, q) != (dir, p)) {
                let est = self.graph.store.estimate(&[None, Some(p), None]);
                roles.push((dir, p, est));
            }
        }
        roles.sort_unstable_by_key(|&(_, _, est)| est);
        let budget = (4 * k).max(64);
        let mut acc: FxHashMap<Id, f64> = FxHashMap::default();
        for &(dir, p, _) in &roles {
            // Who else plays this role? Every subject (OUT) / object (IN) of
            // the predicate's block.
            let sorted_by = if dir == OUT { 0 } else { 2 };
            let scan = self.graph.store.scan_sorted(&[None, Some(p), None], sorted_by);
            let mut last: Option<Id> = None;
            for row in scan.rows.iter() {
                let cand = scan.to_spo(row)[sorted_by];
                if last == Some(cand) {
                    continue;
                }
                last = Some(cand);
                if Some(cand) == exclude || have.contains(&cand) || !self.is_entity(cand) {
                    continue;
                }
                if !acc.contains_key(&cand) && acc.len() >= budget {
                    continue; // budget full: existing candidates may still accrue
                }
                *acc.entry(cand).or_insert(0.0) += self.weight(p);
            }
            if acc.len() >= budget {
                break;
            }
        }
        // Re-score by predicate-profile Jaccard and fill the remaining slots.
        let probe = self.profile_of(sig);
        let mut fb: Vec<(Id, f64)> = acc
            .into_keys()
            .map(|c| (c, weighted_jaccard(&probe, &self.signature_of_id_mode(c, false))))
            .filter(|&(_, s)| s > 0.0)
            .collect();
        fb.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        fb.truncate(k - scored.len());
        scored.extend(fb);
    }

    /// Collapses a signature to its predicate profile: distinct
    /// `(direction, predicate)` elements (neighbor = [`NO_ID`]), weighted like
    /// a [`SignatureMode::Predicates`] signature.
    fn profile_of(&self, sig: &Signature) -> Signature {
        #[cfg(feature = "multi-hop")]
        if self.depth > 1 {
            let mut role_weights: FxHashMap<(u8, Id, Id), f64> = FxHashMap::default();
            for (&(direction, predicate, _), &weight) in sig.elems.iter().zip(&sig.weights) {
                role_weights
                    .entry((direction, predicate, NO_ID))
                    .and_modify(|current| *current = current.max(weight))
                    .or_insert(weight);
            }
            let mut weighted: Vec<((u8, Id, Id), f64)> = role_weights.into_iter().collect();
            weighted.sort_unstable_by_key(|&(elem, _)| elem);
            let (elems, weights): (Vec<_>, Vec<_>) = weighted.into_iter().unzip();
            let total = weights.iter().sum();
            return Signature {
                elems,
                weights,
                total,
            };
        }

        let mut elems: Vec<(u8, Id, Id)> =
            sig.elems.iter().map(|&(d, p, _)| (d, p, NO_ID)).collect();
        elems.sort_unstable();
        elems.dedup();
        let weights: Vec<f64> = elems.iter().map(|&(_, p, _)| self.weight(p)).collect();
        let total = weights.iter().sum();
        Signature { elems, weights, total }
    }

    /// Whether an id names an entity (IRI or blank node) — literals (incl. inline
    /// integers) are valid signature *neighbors* but are never returned as similar
    /// *entities*.
    #[inline]
    fn is_entity(&self, id: Id) -> bool {
        if dict::is_inline(id) {
            return false;
        }
        matches!(
            self.graph.dict.term_parts(id),
            dict::TermParts::Iri { .. } | dict::TermParts::Blank(_)
        )
    }
}

/// Weighted Jaccard of two prebuilt signatures — `Σ min(wA, wB) / Σ max(wA, wB)` by a
/// single merge of the two sorted element lists; 0 when the union is empty. The
/// signature-level counterpart of [`Sim::similarity`], for callers that cache
/// signatures (e.g. all-pairs evaluation).
pub fn weighted_jaccard(a: &Signature, b: &Signature) -> f64 {
    let inter = weighted_intersection(a, b);
    let union = a.total + b.total - inter;
    if union <= 0.0 {
        0.0
    } else {
        inter / union
    }
}

/// Weighted Sorensen-Dice coefficient of two prebuilt signatures:
/// `2 * Σ min(wA, wB) / (Σ wA + Σ wB)`. Returns 0 when both totals are 0.
pub fn dice_coefficient(a: &Signature, b: &Signature) -> f64 {
    let denom = a.total + b.total;
    if denom <= 0.0 {
        0.0
    } else {
        2.0 * weighted_intersection(a, b) / denom
    }
}

/// Weighted overlap (Szymkiewicz-Simpson) coefficient of two prebuilt signatures:
/// `Σ min(wA, wB) / min(Σ wA, Σ wB)`. Returns 0 when either total is 0.
pub fn overlap_coefficient(a: &Signature, b: &Signature) -> f64 {
    let min_total = a.total.min(b.total);
    if min_total <= 0.0 {
        0.0
    } else {
        weighted_intersection(a, b) / min_total
    }
}

// [GPT-5.6] sq-da2bz: share the canonical sorted merge across signature metrics.
fn weighted_intersection(a: &Signature, b: &Signature) -> f64 {
    let mut inter = 0.0;
    let (mut i, mut j) = (0, 0);
    while i < a.elems.len() && j < b.elems.len() {
        match a.elems[i].cmp(&b.elems[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                inter += a.weights[i].min(b.weights[j]);
                i += 1;
                j += 1;
            }
        }
    }
    inter
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iri(s: &str) -> Term {
        Term::NamedNode(NamedNode::new(format!("http://ex.org/{s}")).unwrap())
    }

    fn graph(ttl: &str) -> Graph {
        let prefix = "@prefix : <http://ex.org/> . @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n";
        Graph::load_str(&format!("{prefix}{ttl}"), "turtle").unwrap()
    }

    fn unweighted(g: &Graph) -> Sim<'_> {
        Sim::with_config(g, SimConfig { idf: false, ..SimConfig::default() })
    }

    fn explicit_signature(entries: &[((u8, Id, Id), f64)]) -> Signature {
        let mut entries = entries.to_vec();
        entries.sort_unstable_by_key(|&(elem, _)| elem);
        let total = entries.iter().map(|(_, weight)| weight).sum();
        let (elems, weights) = entries.into_iter().unzip();
        Signature {
            elems,
            weights,
            total,
        }
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-12,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn signature_coefficients_identical_are_one() {
        let sig = explicit_signature(&[((OUT, 10, 100), 1.5), ((IN, 20, 200), 2.5)]);

        assert_close(dice_coefficient(&sig, &sig), 1.0);
        assert_close(overlap_coefficient(&sig, &sig), 1.0);
    }

    #[test]
    fn signature_coefficients_disjoint_are_zero() {
        let a = explicit_signature(&[((OUT, 10, 100), 2.0)]);
        let b = explicit_signature(&[((IN, 20, 200), 3.0)]);

        assert_eq!(dice_coefficient(&a, &b), 0.0);
        assert_eq!(overlap_coefficient(&a, &b), 0.0);
    }

    #[test]
    fn signature_coefficients_partial_overlap_are_weighted_and_symmetric() {
        // A totals 3; B totals 4; their shared element contributes min(2, 1) = 1.
        let a = explicit_signature(&[((OUT, 10, 100), 2.0), ((OUT, 20, 200), 1.0)]);
        let b = explicit_signature(&[((OUT, 10, 100), 1.0), ((IN, 30, 300), 3.0)]);
        let dice = dice_coefficient(&a, &b);
        let overlap = overlap_coefficient(&a, &b);
        let jaccard = weighted_jaccard(&a, &b);

        assert_close(dice, 2.0 / 7.0);
        assert_close(overlap, 1.0 / 3.0);
        assert_close(jaccard, 1.0 / 6.0);
        assert!(dice > jaccard, "Dice need not be at most Jaccard");
        assert!(overlap >= jaccard);
        assert!(jaccard >= 0.0);
        assert_close(dice_coefficient(&b, &a), dice);
        assert_close(overlap_coefficient(&b, &a), overlap);
    }

    #[test]
    fn signature_coefficients_with_empty_signature_are_zero_and_finite() {
        let empty = explicit_signature(&[]);
        let nonempty = explicit_signature(&[((OUT, 10, 100), 2.0)]);

        for value in [
            dice_coefficient(&empty, &nonempty),
            dice_coefficient(&nonempty, &empty),
            overlap_coefficient(&empty, &nonempty),
            overlap_coefficient(&nonempty, &empty),
        ] {
            assert_eq!(value, 0.0);
            assert!(!value.is_nan());
        }
    }

    #[cfg(feature = "multi-hop")]
    fn multi_hop_unweighted(g: &Graph, depth: usize) -> Sim<'_> {
        Sim::with_config(
            g,
            SimConfig {
                idf: false,
                depth,
                profile_fallback: false,
                ..SimConfig::default()
            },
        )
    }

    #[cfg(feature = "multi-hop")]
    #[test]
    fn default_depth_pins_original_signature_and_similarity() {
        let g = graph(":a :p :b . :twin :p :b . :b :q :c .");
        let sim = Sim::with_config(
            &g,
            SimConfig {
                idf: false,
                profile_fallback: false,
                ..SimConfig::default()
            },
        );
        let p = g.id_of(&iri("p")).unwrap();
        let b = g.id_of(&iri("b")).unwrap();
        let signature = sim.signature(&iri("a")).unwrap();

        assert_eq!(signature.elems, vec![(OUT, p, b)]);
        assert_eq!(signature.weights, vec![1.0]);
        assert_eq!(signature.total_weight(), 1.0);
        assert_eq!(sim.similarity(&iri("a"), &iri("twin")), 1.0);
    }

    #[cfg(feature = "multi-hop")]
    #[test]
    fn depth_two_chain_has_exact_attenuated_signature() {
        let g = graph(":a :p :b . :b :q :c .");
        let sim = multi_hop_unweighted(&g, 2);
        let mut expected = vec![
            (
                (
                    OUT,
                    g.id_of(&iri("p")).unwrap(),
                    g.id_of(&iri("b")).unwrap(),
                ),
                1.0,
            ),
            (
                (
                    OUT,
                    g.id_of(&iri("q")).unwrap(),
                    g.id_of(&iri("c")).unwrap(),
                ),
                0.5,
            ),
        ];
        expected.sort_unstable_by_key(|&(elem, _)| elem);
        let signature = sim.signature(&iri("a")).unwrap();
        let actual: Vec<_> = signature
            .elems
            .iter()
            .copied()
            .zip(signature.weights.iter().copied())
            .collect();

        assert_eq!(actual, expected);
        assert_eq!(signature.total_weight(), 1.5);
    }

    #[test]
    fn identical_signatures_similarity_one() {
        let g = graph(":alice :knows :bob ; :worksAt :acme . :carol :knows :bob ; :worksAt :acme .");
        let sim = unweighted(&g);
        assert_eq!(sim.similarity(&iri("alice"), &iri("carol")), 1.0);
        assert_eq!(sim.similarity(&iri("alice"), &iri("alice")), 1.0);
    }

    #[test]
    fn plain_jaccard_hand_computed() {
        // sig(alice) = {(out,knows,bob), (out,knows,eve), (out,worksAt,acme)}  (3 elems)
        // sig(dave)  = {(out,knows,eve), (out,plays,chess)}                    (2 elems)
        // intersection = {(out,knows,eve)} = 1; union = 4 → 0.25.
        let g = graph(":alice :knows :bob, :eve ; :worksAt :acme . :dave :knows :eve ; :plays :chess .");
        let sim = unweighted(&g);
        assert_eq!(sim.similarity(&iri("alice"), &iri("dave")), 0.25);
        // symmetric
        assert_eq!(sim.similarity(&iri("dave"), &iri("alice")), 0.25);
    }

    #[test]
    fn disjoint_signatures_zero_and_absent_term_zero() {
        let g = graph(":a :p :x . :b :q :y .");
        let sim = unweighted(&g);
        assert_eq!(sim.similarity(&iri("a"), &iri("b")), 0.0);
        assert_eq!(sim.similarity(&iri("a"), &iri("missing")), 0.0);
        assert!(sim.signature(&iri("missing")).is_none());
        assert!(sim.most_similar(&iri("missing"), 5).is_empty());
    }

    #[test]
    fn incoming_edges_count() {
        // a and b share ONLY their incoming context: x→a and x→b via :p.
        let g = graph(":x :p :a . :x :p :b .");
        let sim = unweighted(&g);
        // sig(a) = sig(b) = {(in, p, x)} → 1.0
        assert_eq!(sim.similarity(&iri("a"), &iri("b")), 1.0);
        // and direction matters: x's OUTGOING (p, a) is not a's INCOMING (p, x)
        assert_eq!(sim.similarity(&iri("x"), &iri("a")), 0.0);
    }

    #[test]
    fn predicates_only_mode_ignores_neighbors() {
        let g = graph(":a :knows :x ; :worksAt :acme . :b :knows :y ; :worksAt :corp .");
        let pred_only = Sim::with_config(
            &g,
            SimConfig { mode: SignatureMode::Predicates, idf: false, ..SimConfig::default() },
        );
        // Same predicate profile, different neighbors.
        assert_eq!(pred_only.similarity(&iri("a"), &iri("b")), 1.0);
        let full = unweighted(&g);
        assert_eq!(full.similarity(&iri("a"), &iri("b")), 0.0);
    }

    #[test]
    fn idf_weights_rare_predicates_higher() {
        // a∩b share the RARE predicate (2 triples); a∩c share the COMMON one (8 triples).
        // Unweighted the two pairs tie; with IDF the rare-shared pair must win.
        let mut ttl = String::from(":a :rare :r ; :common :c1 . :b :rare :r . :c :common :c1 .");
        for i in 0..6 {
            ttl.push_str(&format!(" :f{i} :common :c{i} ."));
        }
        let g = graph(&ttl);
        let plain = unweighted(&g);
        let ab = plain.similarity(&iri("a"), &iri("b"));
        let ac = plain.similarity(&iri("a"), &iri("c"));
        assert_eq!(ab, ac, "unweighted: both pairs share exactly one of a's two elements");
        let idf = Sim::new(&g);
        assert!(
            idf.similarity(&iri("a"), &iri("b")) > idf.similarity(&iri("a"), &iri("c")),
            "IDF must favour the rare shared predicate"
        );
    }

    #[test]
    fn excluded_predicates_drop_from_signature() {
        let g = graph(":a rdf:type :T ; :p :x . :b rdf:type :T ; :q :y .");
        let rdf_type = NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap();
        let with_type = unweighted(&g);
        assert!(with_type.similarity(&iri("a"), &iri("b")) > 0.0);
        let no_type = Sim::with_config(
            &g,
            SimConfig { idf: false, exclude_predicates: vec![rdf_type], ..SimConfig::default() },
        );
        assert_eq!(no_type.similarity(&iri("a"), &iri("b")), 0.0);
        assert_eq!(no_type.signature(&iri("a")).unwrap().len(), 1); // just (out, :p, :x)
    }

    #[test]
    fn most_similar_ranks_and_excludes_self() {
        let g = graph(
            ":a :knows :bob, :eve ; :worksAt :acme .
             :twin :knows :bob, :eve ; :worksAt :acme .
             :half :knows :bob ; :worksAt :other .
             :far  :plays :chess .",
        );
        let sim = unweighted(&g);
        let top = sim.most_similar(&iri("a"), 10);
        let names: Vec<String> = top.iter().map(|(t, _)| t.to_string()).collect();
        assert_eq!(names[0], "<http://ex.org/twin>");
        assert_eq!(top[0].1, 1.0);
        assert_eq!(names[1], "<http://ex.org/half>"); // shares 1 elem; union = 3+2-1 → 1/4
        assert_eq!(top[1].1, 0.25);
        assert!(!names.iter().any(|n| n == "<http://ex.org/a>"), "self must be excluded");
        assert!(!names.iter().any(|n| n == "<http://ex.org/far>"), "disjoint entity must not appear");
        // most_similar scores must agree with the pairwise API.
        assert_eq!(top[1].1, sim.similarity(&iri("a"), &iri("half")));
        // … and literals must never be returned as entities.
        let g2 = graph(":s1 :p \"lit\" . :s2 :p \"lit\" .");
        let sim2 = unweighted(&g2);
        assert!(sim2.most_similar(&iri("s1"), 5).iter().all(|(t, _)| !matches!(t, Term::Literal(_))));
    }

    #[test]
    fn similar_by_signature_matches_most_similar_modulo_self() {
        let g = graph(
            ":a :knows :bob ; :worksAt :acme .
             :twin :knows :bob ; :worksAt :acme .
             :half :knows :bob .",
        );
        let sim = unweighted(&g);
        let sig = sim.signature(&iri("a")).unwrap();
        let by_sig = sim.similar_by_signature(&sig, 10);
        // Without an exclusion, `a` itself is the best match for its own signature.
        assert_eq!(by_sig[0].0.to_string(), "<http://ex.org/a>");
        assert_eq!(by_sig[0].1, 1.0);
        assert_eq!(by_sig[1].0.to_string(), "<http://ex.org/twin>");
    }

    #[test]
    fn hub_elements_are_capped_but_rescore_is_exact() {
        // (p, hub) is shared by 6 entities; with the cap below 6 the hub element cannot
        // GENERATE candidates, but :a and :b also share the rare (q, r) pair, so :b is
        // still found — and its score is the EXACT similarity including the hub element.
        let mut ttl = String::from(":a :q :r . :b :q :r .");
        for e in ["a", "b", "c", "d", "e", "f"] {
            ttl.push_str(&format!(" :{e} :p :hub ."));
        }
        let g = graph(&ttl);
        let strict = Sim::with_config(
            &g,
            SimConfig {
                idf: false,
                max_pair_frequency: 5,
                profile_fallback: false,
                ..SimConfig::default()
            },
        );
        let top = strict.most_similar(&iri("a"), 10);
        assert_eq!(top[0].0.to_string(), "<http://ex.org/b>");
        assert_eq!(top[0].1, 1.0, "re-score must include the capped hub element");
        // c..f share only the capped hub element → not generated (the approximation;
        // fallback disabled pins the strict v1 behaviour).
        assert_eq!(top.len(), 1);
        // With the default fallback, the starved slots fill with role-similar
        // entities — BELOW the exactly-scored :b, scored by profile Jaccard
        // (c..f share 1 of a's 2 roles → 0.5).
        let sim = Sim::with_config(
            &g,
            SimConfig { idf: false, max_pair_frequency: 5, ..SimConfig::default() },
        );
        let top = sim.most_similar(&iri("a"), 10);
        assert_eq!(top[0].0.to_string(), "<http://ex.org/b>");
        assert_eq!(top[0].1, 1.0);
        assert_eq!(top.len(), 5, "fallback fills with c..f");
        assert!(top[1..].iter().all(|(_, s)| *s == 0.5));
    }

    /// The neighbor-sparse fallback (olympics Sport 0/0): entities whose every
    /// signature element points at a degree-1 neighbor generate NO candidates
    /// through (predicate, neighbor) ranges — every event names exactly one
    /// sport, so the reverse scan finds only the query itself. The fallback
    /// fills with role-similar entities scored by profile Jaccard.
    #[test]
    fn neighbor_sparse_entities_fall_back_to_profile_candidates() {
        // Star topology: each sport is named by its own events, via one predicate.
        let g = graph(
            ":e1 :inSport :s1 . :e2 :inSport :s1 .
             :e3 :inSport :s2 . :e4 :inSport :s3 .",
        );
        let strict = Sim::with_config(
            &g,
            SimConfig { idf: false, profile_fallback: false, ..SimConfig::default() },
        );
        assert!(
            strict.most_similar(&iri("s1"), 5).is_empty(),
            "v1 starvation: no shared (predicate, neighbor) element"
        );
        let sim = unweighted(&g);
        let top = sim.most_similar(&iri("s1"), 5);
        let names: Vec<String> = top.iter().map(|(t, _)| t.to_string()).collect();
        assert_eq!(names, ["<http://ex.org/s2>", "<http://ex.org/s3>"]);
        // Both share s1's full role profile {(In, inSport)} → profile Jaccard 1.
        assert!(top.iter().all(|(_, s)| *s == 1.0));
        // The events do NOT leak in: e1..e4 have OUTGOING :inSport, not incoming.
        assert!(!names.iter().any(|n| n.starts_with("<http://ex.org/e")));
    }

    /// Exactly-scored candidates always rank ABOVE fallback entries, even when
    /// the fallback profile score is numerically higher.
    #[test]
    fn fallback_entries_rank_below_exact_matches() {
        // :a shares a concrete neighbor with :twin (exact score < 1) and only a
        // role with :role (profile score 1.0).
        let g = graph(
            ":a :knows :bob ; :likes :x1 .
             :twin :knows :bob ; :likes :x2 .
             :role :knows :y1 ; :likes :y2 .",
        );
        let sim = unweighted(&g);
        let top = sim.most_similar(&iri("a"), 5);
        let names: Vec<String> = top.iter().map(|(t, _)| t.to_string()).collect();
        assert_eq!(names[0], "<http://ex.org/twin>", "exact match first: {names:?}");
        assert_eq!(top[0].1, sim.similarity(&iri("a"), &iri("twin")));
        assert!(names.contains(&"<http://ex.org/role>".to_string()), "{names:?}");
        // No fallback when generation already yields k candidates.
        let top1 = sim.most_similar(&iri("a"), 1);
        assert_eq!(top1.len(), 1);
        assert_eq!(top1[0].0.to_string(), "<http://ex.org/twin>");
    }

    /// The accuracy gate on a GENERATED TAXONOMY (research/genai-design.md §4): entities
    /// of the same class share class-correlated structure plus per-entity noise; with
    /// rdf:type EXCLUDED from signatures, same-class pairs must rank above cross-class
    /// pairs with AUC > 0.9.
    #[test]
    fn generated_taxonomy_auc_gate() {
        const CLASSES: usize = 6;
        const PER_CLASS: usize = 20;
        // Deterministic xorshift, no rand dep.
        let mut st = 0xC0FFEE123u64;
        let mut rng = move |m: usize| {
            st ^= st << 13;
            st ^= st >> 7;
            st ^= st << 17;
            (st % m as u64) as usize
        };
        let mut ttl = String::new();
        for c in 0..CLASSES {
            for e in 0..PER_CLASS {
                let s = format!(":c{c}e{e}");
                ttl.push_str(&format!("{s} rdf:type :Class{c} .\n"));
                // Class-correlated structure: each class has 3 hub neighbors reached via
                // class-specific predicates (each entity links to 2 of the 3).
                for h in 0..3 {
                    if (e + h) % 3 != 0 {
                        ttl.push_str(&format!("{s} :p{c}_{h} :hub{c}_{h} .\n"));
                    }
                }
                // Noise: 2 random edges via shared predicates to random targets.
                for _ in 0..2 {
                    ttl.push_str(&format!("{s} :noise{} :n{} .\n", rng(4), rng(40)));
                }
            }
        }
        let g = graph(&ttl);
        let rdf_type = NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type").unwrap();
        let sim = Sim::with_config(
            &g,
            SimConfig { exclude_predicates: vec![rdf_type], ..SimConfig::default() },
        );
        // All pairwise similarities over the population, labelled by same-class.
        let entities: Vec<(usize, Term)> = (0..CLASSES)
            .flat_map(|c| (0..PER_CLASS).map(move |e| (c, iri(&format!("c{c}e{e}")))))
            .collect();
        let sigs: Vec<(usize, Signature)> =
            entities.iter().map(|(c, t)| (*c, sim.signature(t).unwrap())).collect();
        let mut pos: Vec<f64> = Vec::new();
        let mut neg: Vec<f64> = Vec::new();
        for i in 0..sigs.len() {
            for j in (i + 1)..sigs.len() {
                let s = weighted_jaccard(&sigs[i].1, &sigs[j].1);
                if sigs[i].0 == sigs[j].0 {
                    pos.push(s);
                } else {
                    neg.push(s);
                }
            }
        }
        let auc = auc(&pos, &neg);
        assert!(auc > 0.9, "taxonomy AUC gate failed: {auc:.4}");
    }

    /// AUC by pairwise comparison (ties count ½) — exact, fine at test scale.
    fn auc(pos: &[f64], neg: &[f64]) -> f64 {
        let mut wins = 0.0;
        for &p in pos {
            for &n in neg {
                if p > n {
                    wins += 1.0;
                } else if p == n {
                    wins += 0.5;
                }
            }
        }
        wins / (pos.len() as f64 * neg.len() as f64)
    }
}
