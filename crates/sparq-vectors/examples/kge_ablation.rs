//! [OPUS-4.8] sq-0wo9e.8 / P6 (epic sq-0wo9e, design `research/structure-aware-vectorisation.md`
//! §P6 eval harness) — the runnable **P0 ablation** for structure-aware vectorisation.
//!
//! Trains the thin in-tree shallow KGE ([`sparq_vectors::train`]) and measures it with the
//! standard FILTERED link-prediction harness ([`sparq_vectors::eval`]) across the 2×2 P0 prior
//! matrix `{closure on/off} × {type-constrained negatives, uniform-random negatives}`, with a
//! long-tail (head vs rare entity) breakdown, on two synthetic slices:
//!
//! - a WN18RR-style **structured relational** slice ([`synthetic_relational_ttl`]), and
//! - the synthetic **gUFO** slice ([`synthetic_gufo_ttl`]) (rigid kinds vs anti-rigid roles/phases).
//!
//! It runs the matrix under **both** scoring models — the symmetric **DistMult** and the asymmetric
//! **ComplEx** — because both slices are ~100 % directional, where DistMult is structurally
//! near-random (the adversarial-review finding). The ablation deltas must be read off the ASYMMETRIC
//! ComplEx run, and each cell is reported as a **mean ± std over several seeds** so a single-seed
//! delta is not mistaken for signal.
//!
//! ```sh
//! # Default: the synthetic slices, sized for this NON-CANONICAL work-box (runs in minutes).
//! cargo run -p sparq-vectors --release --features kge --example kge_ablation
//! cargo run -p sparq-vectors --release --features kge --example kge_ablation -- [entities] [people] [seed]
//!
//! # DATASET-GATED full run: point at a real N-Triples/Turtle KG (e.g. a WN18RR dump). Absent the
//! # env var the real-dataset arm is SKIPPED, so a casual run never needs a multi-GB download.
//! SPARQ_KGE_DATASET=/path/to/wn18rr.nt cargo run -p sparq-vectors --release --features kge --example kge_ablation
//! ```
//!
//! All numbers printed here are **INDICATIVE only** — the work-box is non-canonical, the model is a
//! tiny research-grade shallow KGE, and no figure is baked into any committed doc. The harness is the
//! instrument; the numbers must be re-measured on a canonical machine, on a REAL dataset, under the
//! asymmetric model, with multi-seed reporting, before any prior is adopted.

#[cfg(feature = "kge")]
fn main() {
    use sparq_vectors::eval::{
        run_ablation_multiseed, synthetic_gufo_ttl, synthetic_relational_ttl, EvalConfig,
        MultiSeedCell,
    };
    use sparq_vectors::ModelKind;

    let args: Vec<String> = std::env::args().collect();
    let entities: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(800);
    let people: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(600);
    let seed0: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(20260620);

    // Five seeds: the mean ± std over them is what should be read, NOT a single run.
    let seeds: Vec<u64> = (0..5).map(|i| seed0.wrapping_add(i)).collect();

    println!("# sq-0wo9e P0 ablation — INDICATIVE numbers on a NON-CANONICAL work-box");
    println!("# filtered link-prediction: MRR / Hits@1 / Hits@3 / Hits@10 (higher is better)");
    println!("# matrix: closure {{off,on}} x negatives {{uniform,type-constrained}}");
    println!("# reported as MEAN +/- STD over {} seeds; read the ASYMMETRIC ComplEx run for any", seeds.len());
    println!("# inter-cell delta (DistMult is near-random on these 100%-directional slices).");
    println!();

    let print_cell = |c: &MultiSeedCell| {
        let label = format!(
            "closure={} neg={}",
            if c.closure { "ON " } else { "OFF" },
            if c.type_constrained { "type" } else { "unif" },
        );
        let m = &c.metrics;
        println!(
            "{:<24} q~{:<5.0} MRR={:.4}+/-{:.4} H@1={:.4} H@3={:.4} H@10={:.4}+/-{:.4}",
            label,
            m.queries.mean,
            m.mrr.mean,
            m.mrr.std,
            m.hits1.mean,
            m.hits3.mean,
            m.hits10.mean,
            m.hits10.std,
        );
        println!(
            "    head MRR={:.4}+/-{:.4} H@10={:.4}    tail MRR={:.4}+/-{:.4} H@10={:.4}",
            c.head.mrr.mean,
            c.head.mrr.std,
            c.head.hits10.mean,
            c.tail.mrr.mean,
            c.tail.mrr.std,
            c.tail.hits10.mean,
        );
    };

    let run_slice = |name: &str, ttl: &str| {
        // The work-box can afford a longer, slightly larger run than the test preset, for less
        // noisy indicative numbers (still NON-CANONICAL).
        for model in [ModelKind::DistMult, ModelKind::ComplEx] {
            let tag = match model {
                ModelKind::DistMult => "DistMult (SYMMETRIC — near-random on directional data)",
                ModelKind::ComplEx => "ComplEx (ASYMMETRIC — read deltas here)",
            };
            println!("== {} :: {} ==", name, tag);
            let mut cfg = EvalConfig::small(seed0);
            cfg.train.model = model;
            cfg.train.epochs = 150;
            cfg.train.dim = 64;
            cfg.train.negatives_per_positive = 16;
            match run_ablation_multiseed(ttl, "turtle", cfg, &seeds) {
                Ok(cells) => {
                    for c in &cells {
                        print_cell(c);
                    }
                }
                Err(e) => println!("  (ablation failed: {})", e),
            }
            println!();
        }
    };

    run_slice(
        "synthetic relational (WN18RR-style)",
        &synthetic_relational_ttl(entities, seed0),
    );
    run_slice(
        "synthetic gUFO slice",
        &synthetic_gufo_ttl(people, seed0 ^ 0x9),
    );

    // DATASET-GATED full run.
    if let Ok(path) = std::env::var("SPARQ_KGE_DATASET") {
        println!("== real dataset: {} ==", path);
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let format = if path.ends_with(".ttl") {
                    "turtle"
                } else {
                    "ntriples"
                };
                let _ = format;
                run_slice(&format!("dataset {}", path), &text);
            }
            Err(e) => println!("  (could not read {}: {})", path, e),
        }
    } else {
        println!("# (real-dataset arm SKIPPED — set SPARQ_KGE_DATASET=/path/to/kg.nt to enable)");
    }
}

#[cfg(not(feature = "kge"))]
fn main() {
    eprintln!("kge_ablation requires --features kge");
}
