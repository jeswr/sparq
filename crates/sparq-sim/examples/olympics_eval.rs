//! The sparq-sim accuracy + latency evaluation on the real olympics dataset
//! (`bench/qlever-olympics/olympics.nt`, 1.78M triples) — the benchmark gate from
//! `research/genai-design.md` §4.
//!
//! Ground truth: `rdf:type` (foaf:Person, dbo:SportsTeam, dbo:SportsEvent, dbo:Sport,
//! dbo:Olympics, dbo:City). **Type triples are EXCLUDED from the signatures** so the
//! similarity signal cannot trivially read off the label it is judged against (the
//! leakage rule, design doc §5.5).
//!
//! Reported:
//! - **AUC**: over stratified same-class vs cross-class entity pairs (the class
//!   distribution is 98% Person, so pairs are sampled per class, not uniformly);
//! - **precision@10**: fraction of `most_similar(seed, 10)` results sharing a class
//!   with the seed (untyped results count as misses — conservative);
//! - **latency**: `most_similar(k=10)` wall-clock per seed (mean / p50 / p95 / max).
//!
//! Run: `cargo run -p sparq-sim --example olympics_eval --release`
//! Data path: `$SPARQ_OLYMPICS_NT` or `bench/qlever-olympics/olympics.nt` at the repo root.

use oxrdf::{NamedNode, Term};
use sparq_core::dict::Id;
use sparq_core::Graph;
use sparq_sim::{Sim, SimConfig};
use std::collections::HashMap;
use std::time::Instant;

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
/// Per-class sample sizes (the dataset is 98% foaf:Person — stratify, don't sample uniformly).
const PER_CLASS: usize = 40;
/// Only classes with at least this many members participate (skips the 2-member tails).
const MIN_CLASS: usize = 40;

fn data_path() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("SPARQ_OLYMPICS_NT") {
        return Some(p.into());
    }
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../bench/qlever-olympics/olympics.nt");
    p.exists().then_some(p)
}

fn main() {
    let Some(path) = data_path() else {
        eprintln!("olympics.nt not found (set SPARQ_OLYMPICS_NT or place it under bench/qlever-olympics/) — skipping");
        return;
    };
    let t0 = Instant::now();
    let text = std::fs::read_to_string(&path).expect("read olympics.nt");
    let g = Graph::load_str(&text, "ntriples").expect("parse olympics.nt");
    drop(text);
    println!("loaded {} triples in {:.1}s", g.len(), t0.elapsed().as_secs_f64());

    let rdf_type = NamedNode::new(RDF_TYPE).unwrap();
    let sim = Sim::with_config(
        &g,
        SimConfig { exclude_predicates: vec![rdf_type.clone()], ..SimConfig::default() },
    );

    // ---- Ground truth: entity id -> class ids, via one POS scan of the rdf:type block.
    let type_id = g.id_of(&Term::NamedNode(rdf_type)).expect("rdf:type present");
    let scan = g.store.scan(&[None, Some(type_id), None]);
    let mut classes_of: HashMap<Id, Vec<Id>> = HashMap::new();
    let mut members: HashMap<Id, Vec<Id>> = HashMap::new();
    for row in scan.rows.iter() {
        let [s, _, o] = scan.to_spo(row);
        classes_of.entry(s).or_default().push(o);
        members.entry(o).or_default().push(s);
    }
    let mut classes: Vec<(Id, Vec<Id>)> =
        members.into_iter().filter(|(_, m)| m.len() >= MIN_CLASS).collect();
    classes.sort_by_key(|(c, _)| *c);
    println!("classes with >= {MIN_CLASS} members: {}", classes.len());

    // ---- Stratified sample: PER_CLASS entities per class, deterministic stride.
    let mut sample: Vec<(Id, Id)> = Vec::new(); // (entity, class)
    for (c, m) in &classes {
        let mut m = m.clone();
        m.sort_unstable();
        let stride = (m.len() / PER_CLASS).max(1);
        for e in m.iter().step_by(stride).take(PER_CLASS) {
            sample.push((*e, *c));
        }
        let name = g.dict.term(*c);
        println!("  class {name}: {} members, sampled {}", m.len(), m.len().div_ceil(stride).min(PER_CLASS));
    }

    // ---- AUC over all sampled pairs (signatures cached), in BOTH signature modes:
    // PredicateNeighbor measures shared concrete context (the ranking signal used by
    // most_similar); Predicates measures shared role/profile (the class-separation
    // signal — e.g. two Sports share no concrete neighbour, every event names exactly
    // one sport, but their predicate profiles are near-identical).
    let same_class = |a: Id, b: Id| -> bool {
        let (ca, cb) = (&classes_of[&a], &classes_of[&b]);
        ca.iter().any(|c| cb.contains(c))
    };
    let rdf_type_nn = NamedNode::new(RDF_TYPE).unwrap();
    for (label, mode) in [
        ("predicate+neighbor", sparq_sim::SignatureMode::PredicateNeighbor),
        ("predicate-profile ", sparq_sim::SignatureMode::Predicates),
    ] {
        let s = Sim::with_config(
            &g,
            SimConfig {
                mode,
                exclude_predicates: vec![rdf_type_nn.clone()],
                ..SimConfig::default()
            },
        );
        let sigs: Vec<(Id, sparq_sim::Signature)> = sample
            .iter()
            .map(|&(e, _)| (e, s.signature(&g.dict.term(e)).expect("sampled entity in dict")))
            .collect();
        let mut pos: Vec<f64> = Vec::new();
        let mut neg: Vec<f64> = Vec::new();
        let t_auc = Instant::now();
        for i in 0..sigs.len() {
            for j in (i + 1)..sigs.len() {
                let v = sparq_sim::weighted_jaccard(&sigs[i].1, &sigs[j].1);
                if same_class(sigs[i].0, sigs[j].0) {
                    pos.push(v);
                } else {
                    neg.push(v);
                }
            }
        }
        println!(
            "AUC [{label}] (same-class above cross-class): {:.4}   [{} pos / {} neg pairs, {:.1}s]",
            auc(&pos, &neg),
            pos.len(),
            neg.len(),
            t_auc.elapsed().as_secs_f64()
        );
    }

    // ---- precision@10 + most_similar latency, per class and overall.
    let mut latencies: Vec<f64> = Vec::new();
    let mut overall = (0usize, 0usize); // (hits, scored)
    for (c, _) in &classes {
        let seeds: Vec<Id> = sample.iter().filter(|(_, sc)| sc == c).map(|(e, _)| *e).collect();
        let mut hits = 0;
        let mut scored = 0;
        for &e in &seeds {
            let term = g.dict.term(e);
            let t = Instant::now();
            let top = sim.most_similar(&term, 10);
            latencies.push(t.elapsed().as_secs_f64() * 1000.0);
            for (cand, _) in &top {
                scored += 1;
                if let Some(cid) = g.id_of(cand) {
                    if classes_of.contains_key(&cid) && same_class(e, cid) {
                        hits += 1;
                    }
                }
            }
        }
        overall.0 += hits;
        overall.1 += scored;
        let name = g.dict.term(*c);
        println!(
            "  precision@10 {name}: {:.3}  ({hits}/{scored}, {} seeds)",
            hits as f64 / scored.max(1) as f64,
            seeds.len()
        );
    }
    println!(
        "precision@10 overall: {:.3}  ({}/{})",
        overall.0 as f64 / overall.1.max(1) as f64,
        overall.0,
        overall.1
    );

    latencies.sort_by(f64::total_cmp);
    let pct = |p: f64| latencies[((latencies.len() - 1) as f64 * p) as usize];
    println!(
        "most_similar(k=10) latency over {} calls: mean {:.2} ms, p50 {:.2} ms, p95 {:.2} ms, max {:.2} ms",
        latencies.len(),
        latencies.iter().sum::<f64>() / latencies.len() as f64,
        pct(0.50),
        pct(0.95),
        latencies.last().unwrap()
    );
}

/// Exact AUC by pairwise comparison (ties count ½).
fn auc(pos: &[f64], neg: &[f64]) -> f64 {
    // Rank-sum (Mann-Whitney) over the merged score list — O((P+N) log (P+N)).
    let mut all: Vec<(f64, bool)> = pos.iter().map(|&s| (s, true)).chain(neg.iter().map(|&s| (s, false))).collect();
    all.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut rank_sum = 0.0;
    let mut i = 0;
    let mut rank = 1.0;
    while i < all.len() {
        let mut j = i;
        while j < all.len() && all[j].0 == all[i].0 {
            j += 1;
        }
        let avg_rank = (rank + (rank + (j - i) as f64 - 1.0)) / 2.0;
        for item in &all[i..j] {
            if item.1 {
                rank_sum += avg_rank;
            }
        }
        rank += (j - i) as f64;
        i = j;
    }
    let (p, n) = (pos.len() as f64, neg.len() as f64);
    (rank_sum - p * (p + 1.0) / 2.0) / (p * n)
}
