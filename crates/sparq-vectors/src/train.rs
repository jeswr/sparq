//! Thin in-tree shallow-KGE **trainer** (DistMult) — the embedding producer the
//! structure-aware-vectorisation eval harness measures.
//!
//! [OPUS-4.8] sq-0wo9e.8 (epic sq-0wo9e; design `research/structure-aware-vectorisation.md`
//! §P6 eval-harness + sq-0wo9e.8 trainer). This module is **opt-in** (the `kge` cargo feature,
//! off by default, which implies `structure`) and changes **nothing** in the default
//! `sparq-vectors` build or the core engine.
//!
//! # What this is — and what it is NOT
//!
//! `sparq-vectors` is a vector-**search** + **import** surface; embeddings are normally produced
//! **out-of-process** ([`crate::embed`]). This module is the deferred remainder noted by P0
//! ([`crate::structure`]): a **minimal, CPU-only, deterministically-seeded** trainer that
//! *consumes* the P0 [`crate::structure::NegativeSampler`] (type-constrained
//! negatives) over the P0 closure ([`materialise_closure`](crate::structure::materialise_closure))
//! and emits per-entity + per-relation embeddings. Its only purpose is to **produce embeddings to
//! measure** — it is research-grade, not a production training subsystem (no GPU, no minibatch
//! parallelism, no learning-rate schedule beyond a fixed step, no early stopping). It is **not** a
//! claim that sparq ships SOTA KGE.
//!
//! # Why DistMult (the simplest sound choice)
//!
//! DistMult (Yang et al. 2015, *Embedding Entities and Relations for Learning and Inference in
//! Knowledge Bases*) scores a triple `(h, r, t)` as the trilinear product
//! `score = Σ_i e_h[i] · w_r[i] · e_t[i]` — one entity matrix and one (diagonal) relation vector,
//! the fewest parameters of any standard bilinear KGE. It is symmetric in `h`/`t` (it cannot model
//! asymmetric relations) — an honest limitation we accept because the point here is a *measurement
//! substrate for the P0 priors*, not relation-pattern coverage. (ComplEx/RotatE would fix the
//! asymmetry at more code; deferred.)
//!
//! # Training objective
//!
//! Sigmoid + binary-cross-entropy over each positive triple and its sampled negatives
//! (the standard "1-vs-N" / negative-sampling NLL, e.g. Trouillon et al. 2016): for a positive we
//! push `σ(score) → 1`, for each negative `σ(score) → 0`. Gradients are the closed-form DistMult
//! partials; we step with plain SGD plus a small L2 penalty (weight decay) to keep embeddings from
//! diverging. All randomness (init + negative draws) is a deterministic SplitMix64 stream seeded
//! from [`TrainConfig::seed`], so a fixed config reproduces bit-identical embeddings.
//!
//! # Does it actually learn?
//!
//! [`TrainReport`] records per-epoch mean loss; [`train`] asserts nothing itself, but the crate
//! tests (`tests/kge.rs`) check the load-bearing invariants: **loss decreases** over epochs and the
//! embeddings are **non-degenerate** (not all-equal / not collapsed to a point). The eval harness
//! ([`crate::eval`]) then measures filtered link-prediction quality.

use crate::structure::{Corrupt, NegativeSampler, SamplingMode, TypeConstraints};
use rustc_hash::FxHashMap;
use sparq_core::dict::{Id, TermParts};
use sparq_core::Graph;

/// SplitMix64 step — the same deterministic PRNG the rest of the crate uses (kept local so
/// `train` carries no cross-module coupling beyond the feature gate).
#[inline]
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A uniform `f32` in `(-1, 1)` from the PRNG stream (for embedding init).
#[inline]
fn next_unit(state: &mut u64) -> f32 {
    // 24 random bits → [0, 1), then map to (-1, 1).
    let bits = (splitmix64(state) >> 40) as u32; // top 24 bits
    let u = (bits as f32) / ((1u32 << 24) as f32);
    2.0 * u - 1.0
}

/// Numerically-stable logistic sigmoid.
#[inline]
fn sigmoid(x: f32) -> f32 {
    if x >= 0.0 {
        1.0 / (1.0 + (-x).exp())
    } else {
        let e = x.exp();
        e / (1.0 + e)
    }
}

/// Hyper-parameters for the [`train`] loop. All fields are explicit (no hidden defaults) so a run
/// is fully reproducible from its config; [`TrainConfig::small`] gives a sane, work-box-sized
/// preset for tests and the indicative ablation.
#[derive(Clone, Copy, Debug)]
pub struct TrainConfig {
    /// Embedding dimensionality (entity vectors and the diagonal relation vector share it).
    pub dim: usize,
    /// Number of full passes over the positive triples.
    pub epochs: usize,
    /// SGD step size.
    pub lr: f32,
    /// L2 weight-decay coefficient applied to the parameters touched each step (keeps embeddings
    /// bounded; the DistMult objective is otherwise unbounded below for collinear vectors).
    pub l2: f32,
    /// Negatives sampled per positive, split evenly between head- and tail-corruption.
    pub negatives_per_positive: usize,
    /// The P0 ablation switch: [`SamplingMode::TypeConstrained`] (type-valid negatives) vs
    /// [`SamplingMode::Unconstrained`] (uniform-random negatives).
    pub sampling: SamplingMode,
    /// Master seed for init + negative draws + positive shuffle. Fixed ⇒ reproducible.
    pub seed: u64,
}

impl TrainConfig {
    /// A small, work-box-sized preset: 32-dim, a handful of epochs, modest LR. Sized to train a
    /// few-thousand-triple graph in seconds on a CPU — **not** a tuned production configuration.
    pub fn small(sampling: SamplingMode, seed: u64) -> TrainConfig {
        TrainConfig {
            dim: 32,
            epochs: 50,
            lr: 0.1,
            l2: 1e-4,
            negatives_per_positive: 8,
            sampling,
            seed,
        }
    }
}

/// The trained model: dense parameter matrices plus the id→row maps that translate a graph
/// dictionary id into a parameter row. Scoring ([`TrainedModel::score`]) and the eval harness read
/// only this struct, so a model is self-contained once trained.
pub struct TrainedModel {
    /// Embedding dimensionality.
    pub dim: usize,
    /// `entity_emb[row*dim .. row*dim+dim]` is the embedding of the entity whose row is `row`.
    pub entity_emb: Vec<f32>,
    /// `rel_emb[row*dim .. row*dim+dim]` is the diagonal relation vector of relation row `row`.
    pub rel_emb: Vec<f32>,
    /// Graph entity dict-id → entity row.
    entity_row: FxHashMap<Id, usize>,
    /// Graph relation (predicate) dict-id → relation row.
    rel_row: FxHashMap<Id, usize>,
}

impl TrainedModel {
    /// Number of entity rows.
    pub fn num_entities(&self) -> usize {
        self.entity_row.len()
    }

    /// Number of relation rows.
    pub fn num_relations(&self) -> usize {
        self.rel_row.len()
    }

    /// The parameter row of an entity dict-id, if the model knows it.
    pub fn entity_row(&self, id: Id) -> Option<usize> {
        self.entity_row.get(&id).copied()
    }

    /// The parameter row of a relation dict-id, if the model knows it.
    pub fn rel_row(&self, id: Id) -> Option<usize> {
        self.rel_row.get(&id).copied()
    }

    /// A view of an entity row's embedding slice.
    #[inline]
    pub fn entity_vec(&self, row: usize) -> &[f32] {
        &self.entity_emb[row * self.dim..row * self.dim + self.dim]
    }

    /// A view of a relation row's diagonal vector.
    #[inline]
    pub fn rel_vec(&self, row: usize) -> &[f32] {
        &self.rel_emb[row * self.dim..row * self.dim + self.dim]
    }

    /// DistMult score `Σ_i e_h[i] · w_r[i] · e_t[i]` for parameter rows `(h_row, r_row, t_row)`.
    #[inline]
    pub fn score_rows(&self, h_row: usize, r_row: usize, t_row: usize) -> f32 {
        let h = self.entity_vec(h_row);
        let r = self.rel_vec(r_row);
        let t = self.entity_vec(t_row);
        let mut s = 0.0f32;
        for i in 0..self.dim {
            s += h[i] * r[i] * t[i];
        }
        s
    }

    /// DistMult score for a triple of dict-ids, or `None` if any element is unknown to the model
    /// (e.g. an entity that only appeared as a literal object, which has no entity row).
    pub fn score(&self, h: Id, r: Id, t: Id) -> Option<f32> {
        let hr = *self.entity_row.get(&h)?;
        let rr = *self.rel_row.get(&r)?;
        let tr = *self.entity_row.get(&t)?;
        Some(self.score_rows(hr, rr, tr))
    }

    /// Mean L2 norm of the entity embeddings — a cheap **non-degeneracy** probe. A collapsed model
    /// (all rows equal / all-zero) has a tiny or zero spread; see [`TrainedModel::row_spread`].
    pub fn mean_entity_norm(&self) -> f32 {
        if self.entity_row.is_empty() {
            return 0.0;
        }
        let n = self.num_entities();
        let mut total = 0.0f32;
        for row in 0..n {
            let v = self.entity_vec(row);
            total += v.iter().map(|x| x * x).sum::<f32>().sqrt();
        }
        total / n as f32
    }

    /// Mean pairwise-ish spread of entity rows: the mean over dimensions of the per-dimension
    /// standard deviation across rows. Near-zero ⇒ all entity vectors collapsed to one point (a
    /// degenerate solution). Used by the non-degeneracy test.
    pub fn row_spread(&self) -> f32 {
        let n = self.num_entities();
        if n < 2 {
            return 0.0;
        }
        let mut total_std = 0.0f32;
        for d in 0..self.dim {
            let mut mean = 0.0f32;
            for row in 0..n {
                mean += self.entity_emb[row * self.dim + d];
            }
            mean /= n as f32;
            let mut var = 0.0f32;
            for row in 0..n {
                let x = self.entity_emb[row * self.dim + d] - mean;
                var += x * x;
            }
            total_std += (var / n as f32).sqrt();
        }
        total_std / self.dim as f32
    }
}

/// Per-run training diagnostics. **Non-canonical** — these are *learning* signals (the loss curve),
/// not a quality or performance metric, and must never be baked into committed docs.
#[derive(Clone, Debug)]
pub struct TrainReport {
    /// Mean BCE loss per epoch (length == `config.epochs`), in epoch order.
    pub epoch_loss: Vec<f32>,
    /// Positive triples trained on (object-property triples of the graph).
    pub positives: usize,
    /// Entity rows / relation rows in the model.
    pub entities: usize,
    pub relations: usize,
}

impl TrainReport {
    /// The first epoch's mean loss (the "before learning" baseline).
    pub fn first_loss(&self) -> Option<f32> {
        self.epoch_loss.first().copied()
    }
    /// The last epoch's mean loss.
    pub fn last_loss(&self) -> Option<f32> {
        self.epoch_loss.last().copied()
    }
    /// Did the loss decrease overall (last < first)? The load-bearing "it learned" check.
    pub fn loss_decreased(&self) -> bool {
        match (self.first_loss(), self.last_loss()) {
            (Some(a), Some(b)) => b < a,
            _ => false,
        }
    }
}

/// Is the term id an **entity** (named/blank node), as opposed to a literal? Only entities get a
/// row in entity space (mirrors [`crate::structure`]).
fn is_entity(graph: &Graph, id: Id) -> bool {
    matches!(
        graph.dict.term_parts(id),
        TermParts::Iri { .. } | TermParts::Blank(_)
    )
}

/// The output of [`collect_positives`]: the positive triples, the entity id→row map, and the
/// relation id→row map.
type CollectedPositives = (Vec<[Id; 3]>, FxHashMap<Id, usize>, FxHashMap<Id, usize>);

/// Collect the **object-property** positives `(h, r, t)` of `graph` (both ends entities) plus the
/// distinct entity and relation id sets, assigning each a dense parameter row (sorted id order, so
/// row assignment is deterministic and platform-independent).
fn collect_positives(graph: &Graph) -> CollectedPositives {
    let mut positives: Vec<[Id; 3]> = Vec::new();
    let mut entity_ids: Vec<Id> = Vec::new();
    let mut rel_ids: Vec<Id> = Vec::new();
    let mut seen_e = rustc_hash::FxHashSet::default();
    let mut seen_r = rustc_hash::FxHashSet::default();

    for [s, p, o] in graph.iter_ids() {
        if is_entity(graph, s) && is_entity(graph, o) {
            positives.push([s, p, o]);
            if seen_e.insert(s) {
                entity_ids.push(s);
            }
            if seen_e.insert(o) {
                entity_ids.push(o);
            }
            if seen_r.insert(p) {
                rel_ids.push(p);
            }
        }
    }
    entity_ids.sort_unstable();
    rel_ids.sort_unstable();
    let entity_row: FxHashMap<Id, usize> = entity_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();
    let rel_row: FxHashMap<Id, usize> =
        rel_ids.iter().enumerate().map(|(i, &id)| (id, i)).collect();
    (positives, entity_row, rel_row)
}

/// Train a [`TrainedModel`] over `graph` (which should already be **closed** if the closure arm of
/// the ablation is on — call [`materialise_closure`](crate::structure::materialise_closure) first).
///
/// `constraints` are the P0 type constraints mined from the same graph; they only bite when
/// `config.sampling` is [`SamplingMode::TypeConstrained`] (otherwise the sampler ignores them).
///
/// Returns the trained model and a [`TrainReport`] (the per-epoch loss curve). The loop:
/// 1. init every entity/relation parameter from the seeded PRNG in `(-1, 1)`, scaled by `1/√dim`;
/// 2. for each epoch, shuffle the positives (seeded), and for each positive take one SGD step over
///    the positive plus `negatives_per_positive` sampled negatives (half head-, half tail-corrupt);
/// 3. accumulate the per-step BCE into the epoch's mean loss.
///
/// CPU-only, single-threaded, allocation-light: the only large buffers are the two parameter
/// matrices. Determinism: identical `(graph, constraints, config)` ⇒ identical model.
pub fn train(
    graph: &Graph,
    constraints: &TypeConstraints,
    config: TrainConfig,
) -> (TrainedModel, TrainReport) {
    let (positives, entity_row, rel_row) = collect_positives(graph);
    let dim = config.dim;
    let n_ent = entity_row.len();
    let n_rel = rel_row.len();

    // Parameter init: small uniform values scaled by 1/√dim (a standard KGE init that keeps the
    // initial trilinear score near zero so the sigmoid starts in its linear regime).
    let scale = 1.0 / (dim as f32).sqrt();
    let mut init_state = config.seed ^ 0xA5A5_5A5A_F0F0_0F0F;
    let mut entity_emb = vec![0.0f32; n_ent * dim];
    for x in entity_emb.iter_mut() {
        *x = next_unit(&mut init_state) * scale;
    }
    let mut rel_emb = vec![0.0f32; n_rel.max(1) * dim];
    for x in rel_emb.iter_mut() {
        *x = next_unit(&mut init_state) * scale;
    }

    let sampler = NegativeSampler::new(graph, constraints, config.sampling);

    let mut epoch_loss = Vec::with_capacity(config.epochs);
    let lr = config.lr;
    let l2 = config.l2;
    let half_neg = config.negatives_per_positive.div_ceil(2);

    for epoch in 0..config.epochs {
        // Deterministic per-epoch shuffle: Fisher–Yates over an index permutation seeded by
        // (seed, epoch), so the positive order varies between epochs but reproduces across runs.
        let mut order: Vec<usize> = (0..positives.len()).collect();
        let mut shuf_state = config.seed ^ (epoch as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        for i in (1..order.len()).rev() {
            let j = (splitmix64(&mut shuf_state) as usize) % (i + 1);
            order.swap(i, j);
        }

        let mut loss_sum = 0.0f64;
        let mut loss_terms = 0usize;

        for &pi in &order {
            let [h, r, t] = positives[pi];
            // rows are guaranteed present: every positive's ends/relation were rowed above.
            let hr = entity_row[&h];
            let rr = rel_row[&r];
            let tr = entity_row[&t];

            // Negative-sampling seed: stable per (seed, epoch, positive index) so the draws are
            // reproducible yet differ across epochs (the trainer sees fresh negatives each pass).
            let neg_seed = config
                .seed
                .wrapping_add((epoch as u64).wrapping_mul(0x100_0000))
                .wrapping_add(pi as u64);

            // The training batch: 1 positive (label 1) + negatives (label 0).
            // Positive step.
            let (l, _) = step(&mut entity_emb, &mut rel_emb, dim, hr, rr, tr, 1.0, lr, l2);
            loss_sum += l as f64;
            loss_terms += 1;

            // Tail-corrupted negatives.
            for neg in sampler.sample([h, r, t], Corrupt::Tail, half_neg, neg_seed) {
                let ntr = entity_row[&neg[2]];
                let (l, _) = step(&mut entity_emb, &mut rel_emb, dim, hr, rr, ntr, 0.0, lr, l2);
                loss_sum += l as f64;
                loss_terms += 1;
            }
            // Head-corrupted negatives.
            for neg in sampler.sample(
                [h, r, t],
                Corrupt::Head,
                config.negatives_per_positive - half_neg,
                neg_seed ^ 0xDEAD_BEEF,
            ) {
                let nhr = entity_row[&neg[0]];
                let (l, _) = step(&mut entity_emb, &mut rel_emb, dim, nhr, rr, tr, 0.0, lr, l2);
                loss_sum += l as f64;
                loss_terms += 1;
            }
        }

        let mean = if loss_terms == 0 {
            0.0
        } else {
            (loss_sum / loss_terms as f64) as f32
        };
        epoch_loss.push(mean);
    }

    let report = TrainReport {
        epoch_loss,
        positives: positives.len(),
        entities: n_ent,
        relations: n_rel,
    };
    let model = TrainedModel {
        dim,
        entity_emb,
        rel_emb,
        entity_row,
        rel_row,
    };
    (model, report)
}

/// One SGD step on a single labelled triple (rows `h,r,t`, `label ∈ {0,1}`).
///
/// DistMult score `s = Σ_i e_h[i]·w_r[i]·e_t[i]`; BCE loss `L = -[y·log σ(s) + (1-y)·log(1-σ(s))]`.
/// The gradient of `L` w.r.t. `s` is `σ(s) - y`; the parameter partials are the standard trilinear
/// products. We add an L2 penalty `½·l2·‖θ‖²` (gradient `l2·θ`) to the three touched rows.
/// Returns `(loss, score)`.
#[allow(clippy::too_many_arguments)]
fn step(
    entity_emb: &mut [f32],
    rel_emb: &mut [f32],
    dim: usize,
    h_row: usize,
    r_row: usize,
    t_row: usize,
    label: f32,
    lr: f32,
    l2: f32,
) -> (f32, f32) {
    let hb = h_row * dim;
    let rb = r_row * dim;
    let tb = t_row * dim;

    // Forward: score.
    let mut s = 0.0f32;
    for i in 0..dim {
        s += entity_emb[hb + i] * rel_emb[rb + i] * entity_emb[tb + i];
    }
    let p = sigmoid(s);
    // Numerically-stable BCE.
    let eps = 1e-7f32;
    let pc = p.clamp(eps, 1.0 - eps);
    let loss = -(label * pc.ln() + (1.0 - label) * (1.0 - pc).ln());

    // dL/ds = σ(s) - y.
    let g = p - label;

    // Parameter gradients (note h_row could equal t_row for a self-loop; we read into locals
    // first so the simultaneous update is consistent, then write back).
    // dL/de_h[i] = g · w_r[i] · e_t[i]; dL/dw_r[i] = g · e_h[i] · e_t[i];
    // dL/de_t[i] = g · e_h[i] · w_r[i].
    for i in 0..dim {
        let eh = entity_emb[hb + i];
        let et = entity_emb[tb + i];
        let wr = rel_emb[rb + i];

        let gh = g * wr * et + l2 * eh;
        let gt = g * eh * wr + l2 * et;
        let gr = g * eh * et + l2 * wr;

        entity_emb[hb + i] = eh - lr * gh;
        // If h_row == t_row this second write must use the *updated* h slot's logical value, but
        // because gh and gt are computed from the pre-update reads above, applying both as separate
        // subtractions on the same cell yields eh - lr*gh - lr*gt, which is the correct combined
        // gradient step for a shared parameter. So read-then-write per index is safe.
        entity_emb[tb + i] -= lr * gt;
        rel_emb[rb + i] = wr - lr * gr;
    }

    (loss, s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::close_for_vectorise;
    use sparq_reason::Profile;

    const TTL: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex:   <http://ex/> .

ex:Dog   rdfs:subClassOf ex:Animal .
ex:Cat   rdfs:subClassOf ex:Animal .
ex:Owner rdfs:subClassOf ex:Person .

ex:owns rdfs:domain ex:Person ; rdfs:range ex:Animal .

ex:alice a ex:Owner . ex:bob a ex:Owner . ex:carol a ex:Owner .
ex:rex a ex:Dog . ex:tom a ex:Cat . ex:fido a ex:Dog . ex:milo a ex:Cat .

ex:alice ex:owns ex:rex .
ex:bob   ex:owns ex:tom .
ex:carol ex:owns ex:fido .
ex:alice ex:owns ex:milo .
"#;

    #[test]
    fn trains_and_loss_decreases() {
        let c = close_for_vectorise(TTL, "turtle", Profile::Rdfs).unwrap();
        let tc = TypeConstraints::mine(&c.graph);
        let cfg = TrainConfig::small(SamplingMode::TypeConstrained, 7);
        let (model, report) = train(&c.graph, &tc, cfg);
        assert!(
            report.positives >= 4,
            "expected the 4 owns triples as positives"
        );
        assert!(model.num_entities() >= 7);
        assert!(model.num_relations() >= 1);
        assert!(
            report.loss_decreased(),
            "loss must decrease: first={:?} last={:?}",
            report.first_loss(),
            report.last_loss()
        );
    }

    #[test]
    fn embeddings_are_non_degenerate() {
        let c = close_for_vectorise(TTL, "turtle", Profile::Rdfs).unwrap();
        let tc = TypeConstraints::mine(&c.graph);
        let cfg = TrainConfig::small(SamplingMode::TypeConstrained, 11);
        let (model, _) = train(&c.graph, &tc, cfg);
        // Non-degeneracy: rows must not have collapsed to a single point.
        assert!(
            model.row_spread() > 1e-3,
            "entity rows collapsed: spread={}",
            model.row_spread()
        );
        assert!(model.mean_entity_norm() > 1e-3, "entity norms collapsed");
    }

    #[test]
    fn is_deterministic_for_fixed_config() {
        let c = close_for_vectorise(TTL, "turtle", Profile::Rdfs).unwrap();
        let tc = TypeConstraints::mine(&c.graph);
        let cfg = TrainConfig::small(SamplingMode::TypeConstrained, 3);
        let (m1, r1) = train(&c.graph, &tc, cfg);
        let (m2, r2) = train(&c.graph, &tc, cfg);
        assert_eq!(r1.epoch_loss, r2.epoch_loss, "loss curve must reproduce");
        assert_eq!(m1.entity_emb, m2.entity_emb, "entity params must reproduce");
        assert_eq!(m1.rel_emb, m2.rel_emb, "relation params must reproduce");
    }

    #[test]
    fn learns_to_rank_true_above_false() {
        // After training, a known-true positive should score higher than a type-valid corruption
        // for most positives (the basic "it learned the signal" check, weaker than the harness MRR).
        let c = close_for_vectorise(TTL, "turtle", Profile::Rdfs).unwrap();
        let tc = TypeConstraints::mine(&c.graph);
        let cfg = TrainConfig::small(SamplingMode::TypeConstrained, 5);
        let (model, _) = train(&c.graph, &tc, cfg);
        let owns = c.graph.id_of(&iri("http://ex/owns")).unwrap();
        let alice = c.graph.id_of(&iri("http://ex/alice")).unwrap();
        let rex = c.graph.id_of(&iri("http://ex/rex")).unwrap();
        let tom = c.graph.id_of(&iri("http://ex/tom")).unwrap();
        // (alice owns rex) is true; (alice owns tom) is false (bob owns tom).
        let pos = model.score(alice, owns, rex).unwrap();
        let neg = model.score(alice, owns, tom).unwrap();
        assert!(
            pos > neg,
            "true triple {} should outrank false {}",
            pos,
            neg
        );
    }

    fn iri(s: &str) -> oxrdf::Term {
        oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(s))
    }
}
