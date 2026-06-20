//! [OPUS-4.8] sq-0wo9e.8 / P6 (epic sq-0wo9e, design `research/structure-aware-vectorisation.md`
//! §P6 eval harness) — the runnable **P0 ablation** for structure-aware vectorisation.
//!
//! Trains the thin in-tree DistMult KGE ([`sparq_vectors::train`]) and measures it with the
//! standard FILTERED link-prediction harness ([`sparq_vectors::eval`]) across the 2×2 P0 prior
//! matrix `{closure on/off} × {type-constrained negatives, uniform-random negatives}`, with a
//! long-tail (head vs rare entity) breakdown, on two synthetic slices:
//!
//! - a WN18RR-style **structured relational** slice ([`synthetic_relational_ttl`]), and
//! - the synthetic **gUFO** slice ([`synthetic_gufo_ttl`]) (rigid kinds vs anti-rigid roles/phases).
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
//! tiny research-grade DistMult, and no figure is baked into any committed doc. The harness is the
//! instrument; the numbers must be re-measured on a canonical machine before any prior is adopted.

#[cfg(feature = "kge")]
fn main() {
    use sparq_vectors::eval::{
        run_ablation, synthetic_gufo_ttl, synthetic_relational_ttl, AblationCell, EvalConfig,
        Metrics,
    };

    let args: Vec<String> = std::env::args().collect();
    let entities: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(800);
    let people: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(600);
    let seed: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(20260620);

    println!("# sq-0wo9e P0 ablation — INDICATIVE numbers on a NON-CANONICAL work-box");
    println!("# filtered link-prediction: MRR / Hits@1 / Hits@3 / Hits@10 (higher is better)");
    println!("# matrix: closure {{off,on}} x negatives {{uniform,type-constrained}}");
    println!();

    let print_cell = |label: &str, c: &AblationCell| {
        let m: &Metrics = &c.metrics;
        println!(
            "{:<40} q={:<5} MRR={:.4} H@1={:.4} H@3={:.4} H@10={:.4}",
            label, m.queries, m.mrr, m.hits1, m.hits3, m.hits10
        );
        println!(
            "    long-tail  head(q={:<4}) MRR={:.4} H@10={:.4}   tail(q={:<4}) MRR={:.4} H@10={:.4}   [thr<={}]",
            c.long_tail.head.queries,
            c.long_tail.head.mrr,
            c.long_tail.head.hits10,
            c.long_tail.tail.queries,
            c.long_tail.tail.mrr,
            c.long_tail.tail.hits10,
            c.long_tail.threshold,
        );
        println!(
            "    learn      first_loss={:.4} last_loss={:.4} (decreased={})",
            c.report.first_loss().unwrap_or(0.0),
            c.report.last_loss().unwrap_or(0.0),
            c.report.loss_decreased(),
        );
    };

    let label = |c: &AblationCell| {
        format!(
            "closure={} neg={}",
            if c.closure { "ON " } else { "OFF" },
            if c.type_constrained { "type" } else { "unif" },
        )
    };

    let run_slice = |name: &str, ttl: &str| {
        println!("== {} ==", name);
        let mut cfg = EvalConfig::small(seed);
        // The work-box can afford a longer, slightly larger run than the test preset, for less
        // noisy indicative numbers (still NON-CANONICAL).
        cfg.train.epochs = 150;
        cfg.train.dim = 64;
        cfg.train.negatives_per_positive = 16;
        match run_ablation(ttl, "turtle", cfg) {
            Ok(cells) => {
                for c in &cells {
                    print_cell(&label(c), c);
                }
            }
            Err(e) => println!("  (ablation failed: {})", e),
        }
        println!();
    };

    run_slice(
        "synthetic relational (WN18RR-style)",
        &synthetic_relational_ttl(entities, seed),
    );
    run_slice(
        "synthetic gUFO slice",
        &synthetic_gufo_ttl(people, seed ^ 0x9),
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
                run_slice(&format!("dataset {}", path), &text);
                let _ = format;
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
