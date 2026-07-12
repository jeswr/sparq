//! [GPT-5.6] sq-lsp7k.24: mutation-witnessed differential coverage for HDT
//! triple-pattern filtering at the one-shot SPO decode seam.

#![cfg(feature = "load-filter")]

use oxrdf::{NamedNode, NamedOrBlankNode, Term};
use sparq_core::Graph;
use sparq_hdt::TriplePattern;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

type RenderedTriple = [String; 3];

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn triple_set(graph: &Graph) -> BTreeSet<RenderedTriple> {
    let scan = graph.store.scan(&[None, None, None]);
    scan.rows
        .iter()
        .map(|row| {
            let [subject, predicate, object] = scan.to_spo(row);
            [
                graph.dict.term(subject).to_string(),
                graph.dict.term(predicate).to_string(),
                graph.dict.term(object).to_string(),
            ]
        })
        .collect()
}

fn load_bytes() -> Vec<u8> {
    std::fs::read(fixture("snikmeta.hdt")).expect("reading HDT fixture")
}

fn filtered(pattern: &TriplePattern) -> Graph {
    sparq_hdt::load_reader_filtered(std::io::Cursor::new(load_bytes()), pattern)
        .expect("filtering HDT fixture")
}

#[test]
fn predicate_filter_matches_full_load_oracle() {
    let predicate = NamedNode::new("http://www.w3.org/2000/01/rdf-schema#label").unwrap();
    let pattern = (None, Some(predicate.clone()), None);

    let full = sparq_hdt::load(fixture("snikmeta.hdt")).expect("loading full HDT fixture");
    let expected: BTreeSet<_> = triple_set(&full)
        .into_iter()
        .filter(|triple| triple[1] == predicate.to_string())
        .collect();
    let actual = filtered(&pattern);

    assert_eq!(expected.len(), 74, "fixture predicate count changed");
    assert_eq!(actual.len(), 74);
    assert_eq!(triple_set(&actual), expected);
}

#[test]
fn all_wildcards_are_identical_to_load_reader() {
    let bytes = load_bytes();
    let full = sparq_hdt::load_reader(std::io::Cursor::new(bytes.clone())).unwrap();
    let filtered =
        sparq_hdt::load_reader_filtered(std::io::Cursor::new(bytes), &(None, None, None)).unwrap();

    assert_eq!(filtered.len(), full.len());
    assert_eq!(triple_set(&filtered), triple_set(&full));
    assert_eq!(filtered.dict.len(), full.dict.len());
}

#[test]
fn unmatched_pattern_returns_a_truly_empty_graph() {
    let missing = NamedNode::new("http://example.test/predicate/not-in-fixture").unwrap();
    let result = filtered(&(None, Some(missing), None));

    assert!(result.is_empty());
    assert_eq!(result.dict.len(), 0, "unmatched terms must not be interned");
    assert!(triple_set(&result).is_empty());
}

#[test]
fn fully_bound_pattern_matches_exactly_one_triple() {
    let subject = NamedOrBlankNode::from(
        NamedNode::new("http://www.snik.eu/ontology/meta/ApplicationComponent").unwrap(),
    );
    let predicate = NamedNode::new("http://www.w3.org/2000/01/rdf-schema#label").unwrap();
    let object = Term::from(
        oxrdf::Literal::new_language_tagged_literal("application component", "en").unwrap(),
    );
    let pattern = (
        Some(subject.clone()),
        Some(predicate.clone()),
        Some(object.clone()),
    );
    let result = filtered(&pattern);

    let expected = BTreeSet::from([[
        Term::from(subject).to_string(),
        predicate.to_string(),
        object.to_string(),
    ]]);
    assert_eq!(result.len(), 1);
    assert_eq!(triple_set(&result), expected);
    assert_eq!(
        result.dict.len(),
        3,
        "only the accepted triple's terms are interned"
    );
}
