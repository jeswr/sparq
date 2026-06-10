//! Integration test on the real olympics dataset (1.78M triples). SKIPS (passes,
//! with a note on stderr) when `bench/qlever-olympics/olympics.nt` is absent — the
//! file is a downloaded benchmark fixture, not checked in. Override the path with
//! `SPARQ_OLYMPICS_NT`. Build-time measurement lives in
//! `examples/olympics_introspect.rs` (run with `--release`); this test asserts the
//! design-doc gate: the text summary must mention the dataset's ACTUAL classes.

use sparq_core::Graph;
use sparq_introspect::Introspection;

fn data_path() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("SPARQ_OLYMPICS_NT") {
        return Some(p.into());
    }
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../bench/qlever-olympics/olympics.nt");
    p.exists().then_some(p)
}

#[test]
fn olympics_summary_names_the_real_schema() {
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

    let ix = Introspection::build(&g);

    // Structural sanity at real-data scale.
    assert_eq!(ix.triples, g.len() as u64);
    assert!(ix.subjects > 100_000);
    assert!(!ix.classes.is_empty() && !ix.predicates.is_empty());
    assert!(ix.characteristic_sets.distinct > 0);
    // Characteristic sets partition the subjects exactly.
    let covered: u64 = ix.characteristic_sets.sets.iter().map(|s| s.subjects).sum();
    assert_eq!(
        covered + ix.characteristic_sets.elided_subjects,
        ix.subjects
    );
    // The dominant class is foaf:Person (134,730 athletes — 98% of typed entities).
    assert_eq!(ix.classes[0].class, "http://xmlns.com/foaf/0.1/Person");
    assert!(ix.classes[0].instances > 100_000);

    // The GATE (research/genai-design.md §4): the prompt-ready text summary must
    // mention the dataset's actual classes.
    let summary = ix.to_text_summary(4000);
    assert!(
        summary.chars().count() <= 4000,
        "summary must respect its budget"
    );
    for class in ["Person", "SportsEvent", "SportsTeam", "Olympics"] {
        assert!(
            summary.contains(class),
            "summary must mention class {class}:\n{summary}"
        );
    }
    // …and carry counts (the selectivity signal) and the prefix glossary.
    assert!(summary.contains("instances"));
    assert!(summary.contains("foaf: http://xmlns.com/foaf/0.1/"));

    // JSON surface parses and agrees on the headline numbers.
    let v: serde_json::Value = serde_json::from_str(&ix.to_json()).expect("valid JSON");
    assert_eq!(v["triples"].as_u64(), Some(ix.triples));
    assert_eq!(v["classes"][0]["class"], "http://xmlns.com/foaf/0.1/Person");
}
