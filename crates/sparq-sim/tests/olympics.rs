//! Integration test on the real olympics dataset (1.78M triples). SKIPS (passes,
//! with a note on stderr) when `bench/qlever-olympics/olympics.nt` is absent — the
//! file is a downloaded benchmark fixture, not checked in. Override the path with
//! `SPARQ_OLYMPICS_NT`. The full accuracy/latency evaluation lives in
//! `examples/olympics_eval.rs` (run with `--release`); this test asserts the
//! structural sanity of the API at real-data scale.

use oxrdf::{NamedNode, Term};
use sparq_core::Graph;
use sparq_sim::{Sim, SimConfig};

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

fn data_path() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("SPARQ_OLYMPICS_NT") {
        return Some(p.into());
    }
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../bench/qlever-olympics/olympics.nt");
    p.exists().then_some(p)
}

#[test]
fn olympics_most_similar_sanity() {
    let Some(path) = data_path() else {
        eprintln!("skipping: olympics.nt not present (downloaded benchmark fixture)");
        return;
    };
    let text = std::fs::read_to_string(&path).expect("read olympics.nt");
    let g = Graph::load_str(&text, "ntriples").expect("parse olympics.nt");
    drop(text);
    assert!(
        g.len() > 1_700_000,
        "expected the 1.78M-triple olympics dataset, got {}",
        g.len()
    );

    // Type triples excluded (the eval discipline: rdf:type is the ground truth).
    let rdf_type = NamedNode::new(RDF_TYPE).unwrap();
    let sim = Sim::with_config(
        &g,
        SimConfig {
            exclude_predicates: vec![rdf_type],
            ..SimConfig::default()
        },
    );

    let athlete = Term::NamedNode(
        NamedNode::new("http://wallscope.co.uk/resource/olympics/athlete/AAananthaSambuMayavo")
            .unwrap(),
    );
    let top = sim.most_similar(&athlete, 10);
    assert_eq!(
        top.len(),
        10,
        "a 134k-athlete graph must yield 10 structural neighbours"
    );
    // Scores are valid similarities, sorted best-first, self excluded.
    let mut prev = f64::INFINITY;
    for (t, s) in &top {
        assert!(*s > 0.0 && *s <= 1.0, "score out of range: {s}");
        assert!(*s <= prev, "scores must be non-increasing");
        prev = *s;
        assert_ne!(t, &athlete, "self must be excluded");
        assert!(!matches!(t, Term::Literal(_)), "literals are not entities");
        // most_similar's re-score must agree with the pairwise API.
        assert_eq!(*s, sim.similarity(&athlete, t));
    }
    // The structural neighbours of an athlete should overwhelmingly be other
    // athletes: same team / games / sport context. Loose floor — the precise
    // precision@10 gate is measured by the eval example.
    let athletes = top
        .iter()
        .filter(|(t, _)| t.to_string().contains("/olympics/athlete/"))
        .count();
    assert!(
        athletes >= 5,
        "expected mostly athlete neighbours, got {athletes}/10: {top:?}"
    );
}
