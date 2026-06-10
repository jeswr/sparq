//! Build-time + output measurement for `sparq-introspect` on the real benchmark
//! datasets (the performance gate from `research/genai-design.md` §4):
//!
//! - olympics (1.78M triples): `bench/qlever-olympics/olympics.nt`, override with
//!   `SPARQ_OLYMPICS_NT`;
//! - qlever-synthetic (8M triples, optional): `bench/qlever-synthetic/synthetic.nt`,
//!   override with `SPARQ_SYNTHETIC_NT` — measured when present.
//!
//! Reported per dataset: load time, `Introspection::build` time, `to_json` /
//! `to_text_summary` time and sizes, plus the first part of the text summary.
//!
//! Run: `cargo run -p sparq-introspect --example olympics_introspect --release`

use sparq_core::Graph;
use sparq_introspect::Introspection;
use std::time::Instant;

fn path_of(env: &str, rel: &str) -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var(env) {
        return Some(p.into());
    }
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    p.exists().then_some(p)
}

fn measure(name: &str, path: &std::path::Path) {
    println!("==== {name} ({}) ====", path.display());
    let t0 = Instant::now();
    let text = std::fs::read_to_string(path).expect("read dataset");
    let g = Graph::load_str(&text, "ntriples").expect("parse dataset");
    drop(text);
    println!(
        "loaded {} triples in {:.2}s",
        g.len(),
        t0.elapsed().as_secs_f64()
    );

    let t1 = Instant::now();
    let ix = Introspection::build(&g);
    let build_s = t1.elapsed().as_secs_f64();
    println!(
        "Introspection::build: {build_s:.2}s — {} subjects, {} classes, {} predicates, {} characteristic sets, {} namespaces",
        ix.subjects,
        ix.classes.len(),
        ix.predicates.len(),
        ix.characteristic_sets.distinct,
        ix.vocabularies.distinct
    );

    let t2 = Instant::now();
    let json = ix.to_json();
    println!(
        "to_json: {:.0}ms, {} bytes",
        t2.elapsed().as_secs_f64() * 1000.0,
        json.len()
    );

    let t3 = Instant::now();
    let summary = ix.to_text_summary(2500);
    println!(
        "to_text_summary(2500): {:.1}ms, {} chars",
        t3.elapsed().as_secs_f64() * 1000.0,
        summary.chars().count()
    );
    println!("---- text summary (budget 2500) ----\n{summary}");
}

fn main() {
    match path_of(
        "SPARQ_OLYMPICS_NT",
        "../../bench/qlever-olympics/olympics.nt",
    ) {
        Some(p) => measure("olympics 1.78M", &p),
        None => eprintln!("olympics.nt not found (set SPARQ_OLYMPICS_NT) — skipping"),
    }
    match path_of(
        "SPARQ_SYNTHETIC_NT",
        "../../bench/qlever-synthetic/synthetic.nt",
    ) {
        Some(p) => measure("qlever-synthetic 8M", &p),
        None => eprintln!("synthetic.nt not found (set SPARQ_SYNTHETIC_NT) — skipping"),
    }
}
