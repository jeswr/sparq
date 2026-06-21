//! [OPUS-4.8] sq-0wo9e.8 / P6 (epic sq-0wo9e) — integration tests for the structure-aware
//! vectorisation MEASUREMENT FOUNDATION: the thin DistMult trainer + the filtered link-prediction
//! eval harness, exercising the REAL end-to-end path (parse → close → split → train → rank).
//!
//! These are the load-bearing invariants the design requires before any prior is adopted:
//! - the harness can detect NON-TRIVIAL ranking quality on a clearly-learnable relation (so a low
//!   number elsewhere is about the data/model, not a harness bug);
//! - the FILTERED protocol is correct (no train/test leakage; the filter is the union of all splits);
//! - the trainer actually learns; the ablation matrix runs every cell on the same split;
//! - the gUFO slice is a non-trivial baseline the harness can already ablate.
//!
//! No number is asserted as a quality claim — the only numeric assertion is a sanity FLOOR on a
//! deliberately-easy relation, to prove the instrument works.

#![cfg(feature = "kge")]

use sparq_reason::Profile;
use sparq_vectors::eval::{
    run_ablation, synthetic_gufo_ttl, synthetic_relational_ttl, EvalConfig, Splits,
};
use sparq_vectors::structure::close_for_vectorise;
use sparq_vectors::train::{train, TrainConfig};
use sparq_vectors::SamplingMode;
use std::collections::HashSet;

/// A clearly-learnable SYMMETRIC relation (DistMult's strength): K disjoint cliques connected by
/// `sim` within each clique. A correct harness MUST yield clearly non-trivial filtered MRR/Hits here.
fn clique_ttl(k: usize, per: usize) -> String {
    let mut ttl = String::from("@prefix ex: <http://ex/> .\n");
    for c in 0..k {
        let members: Vec<usize> = (0..per).map(|m| c * per + m).collect();
        for &a in &members {
            for &b in &members {
                if a != b {
                    ttl.push_str(&format!("ex:e{} ex:sim ex:e{} .\n", a, b));
                }
            }
        }
    }
    ttl
}

#[test]
fn harness_detects_quality_on_learnable_relation() {
    // The instrument-validity test: on an easy symmetric relation the harness must report high
    // filtered metrics. If this regresses, the harness (not the data) is broken.
    let ttl = clique_ttl(12, 8);
    let mut cfg = EvalConfig::small(7);
    cfg.train.epochs = 120;
    cfg.train.dim = 48;
    cfg.train.negatives_per_positive = 16;
    let cells = run_ablation(&ttl, "turtle", cfg).unwrap();
    // No schema in this graph → all four cells must be identical (the priors are genuine no-ops).
    for c in &cells[1..] {
        assert!(
            (c.metrics.mrr - cells[0].metrics.mrr).abs() < 1e-9,
            "no-op priors must not change metrics"
        );
    }
    let m = &cells[0].metrics;
    assert!(m.queries > 0);
    // A deliberately conservative floor (the model reaches ~0.9 here; we assert well below that so
    // the test is robust, while still proving the harness measures real ranking quality).
    assert!(
        m.mrr > 0.5,
        "harness should detect strong ranking on a learnable relation: MRR={}",
        m.mrr
    );
    assert!(
        m.hits10 > 0.7,
        "Hits@10 floor on a learnable relation: {}",
        m.hits10
    );
}

#[test]
fn no_leakage_train_disjoint_from_test() {
    let ttl = synthetic_relational_ttl(300, 5);
    let c = close_for_vectorise(&ttl, "turtle", Profile::Rdfs).unwrap();
    let s = Splits::split(&c.graph, 0.8, 0.1, 42);
    let train: HashSet<[u32; 3]> = s.train.iter().copied().collect();
    let valid: HashSet<[u32; 3]> = s.valid.iter().copied().collect();
    let test: HashSet<[u32; 3]> = s.test.iter().copied().collect();
    assert!(
        train.is_disjoint(&test),
        "train must be disjoint from test (no leakage)"
    );
    assert!(
        train.is_disjoint(&valid),
        "train must be disjoint from valid"
    );
    assert!(valid.is_disjoint(&test), "valid must be disjoint from test");
    assert!(!test.is_empty(), "need a non-empty test split");
    // The filter set is the union of all three.
    assert_eq!(s.filter_set_len(), train.len() + valid.len() + test.len());
}

#[test]
fn trainer_learns_on_relational_slice() {
    let ttl = synthetic_relational_ttl(300, 9);
    let c = close_for_vectorise(&ttl, "turtle", Profile::Rdfs).unwrap();
    let tc = sparq_vectors::TypeConstraints::mine(&c.graph);
    let cfg = TrainConfig::small(SamplingMode::TypeConstrained, 1);
    let (model, report) = train(&c.graph, &tc, cfg);
    assert!(report.loss_decreased(), "loss must decrease");
    assert!(
        model.row_spread() > 1e-3,
        "embeddings must be non-degenerate"
    );
}

#[test]
fn ablation_matrix_runs_on_both_slices() {
    for (name, ttl) in [
        ("relational", synthetic_relational_ttl(200, 2)),
        ("gufo", synthetic_gufo_ttl(150, 3)),
    ] {
        let cfg = EvalConfig::small(11);
        let cells = run_ablation(&ttl, "turtle", cfg).unwrap_or_else(|e| panic!("{}: {}", name, e));
        assert_eq!(cells.len(), 4, "{}: 2x2 matrix", name);
        for cell in &cells {
            assert!(
                cell.metrics.queries > 0,
                "{}: every cell must produce scorable queries",
                name
            );
            assert!(
                cell.report.loss_decreased(),
                "{}: every cell's model must learn",
                name
            );
            // long-tail buckets partition the queries.
            assert_eq!(
                cell.long_tail.head.queries + cell.long_tail.tail.queries,
                cell.metrics.queries,
                "{}: long-tail buckets must partition queries",
                name
            );
            // The gUFO prior axis is exposed but OFF at the baseline phase.
            assert!(!cell.gufo_prior);
        }
    }
}

#[test]
fn gufo_closure_axis_actually_changes_something() {
    // The gUFO slice is designed so the RDFS closure is NOT a no-op (people are typed only by their
    // anti-rigid phase/role, so the rigid kind Person must be DERIVED). Confirm closure adds triples.
    let ttl = synthetic_gufo_ttl(150, 4);
    let c = close_for_vectorise(&ttl, "turtle", Profile::Rdfs).unwrap();
    assert!(
        c.entailed_triples > 0,
        "gUFO closure should derive entailed Person memberships (closure axis must bite)"
    );
}
