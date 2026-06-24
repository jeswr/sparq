//! [OPUS-4.8] sq-0wo9e.5 (epic sq-0wo9e) — integration tests for the P4 **flexible grounding**
//! selector + verbaliser over a REAL graph + a REAL `.spqv` store (the modalities that need a
//! built store / schema header cannot be exercised by the in-crate unit tests alone).
//!
//! What is proven here:
//! - the `OutputType → Modality` dispatcher routes a request to the right modality and `ground`
//!   produces the matching object (subgraph / typed sub-vector / NL string / typed value);
//! - the **typed-sub-vector** modality projects ONLY the requested blocks out of a real stored
//!   vector, keyed by the `.spqv` `SchemaHeader` — minimal by construction;
//! - **profile-relative completeness**: a fact entailed only via the closure
//!   (`close_for_vectorise`) appears in the subgraph grounding over the closed graph, and is ABSENT
//!   over the un-closed graph (the honest "complete relative to whatever you closed under" contract);
//! - the subgraph grounding contains only facts present in the graph (never an approximate signal).
//!
//! Gated on `structure` (the grounding module).
#![cfg(feature = "structure")]

use sparq_core::Graph;
use sparq_reason::Profile;
use sparq_vectors::encode::{Block, Encoder, Metric, SchemaHeader};
use sparq_vectors::structure::close_for_vectorise;
use sparq_vectors::{ground, Grounding, GroundingConfig, Modality, OutputType, VectorStore};

fn tmp(name: &str) -> std::path::PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "sparq_vec_grounding_{}_{name}.spqv",
        std::process::id()
    ));
    p
}

fn iri(s: &str) -> oxrdf::Term {
    oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(s))
}

const TTL: &str = r#"
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex:   <http://ex/> .

ex:SportsCar rdfs:subClassOf ex:Car .
ex:Car       rdfs:subClassOf ex:Vehicle .

ex:bolt a ex:SportsCar ;
        rdfs:label "Bolt" ;
        ex:seats 2 ;
        ex:electric true .
"#;

#[test]
fn typed_sub_vector_projects_only_requested_blocks() {
    let g = Graph::load_str(TTL, "turtle").unwrap();
    let bolt = g.id_of(&iri("http://ex/bolt")).unwrap();

    // A 6-dim structured row: a 4-dim numeric block + a 2-dim text block.
    let header = SchemaHeader::new(vec![
        Block::new(Encoder::Numeric, Metric::Euclidean, 0, 4),
        Block::new(Encoder::Other, Metric::Euclidean, 4, 2),
    ])
    .unwrap();

    let mut store = VectorStore::create(tmp("subvec"), 6).unwrap();
    store.put(bolt, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
    store.finalize().unwrap();

    // Keep only the numeric block → the first 4 dims.
    let cfg = GroundingConfig {
        keep_blocks: vec![Encoder::Numeric],
        ..Default::default()
    };
    let Some(Grounding::TypedSubVector { values, blocks }) = ground(
        &g,
        &iri("http://ex/bolt"),
        Modality::TypedSubVector,
        &cfg,
        None,
        Some((&store, &header)),
    ) else {
        panic!("expected a typed-sub-vector grounding");
    };
    assert_eq!(values, vec![1.0, 2.0, 3.0, 4.0], "numeric block only");
    assert_eq!(blocks, vec![Encoder::Numeric]);

    // Keep all (empty keep list) → the full row.
    let all = GroundingConfig::default();
    let Some(Grounding::TypedSubVector { values, blocks }) = ground(
        &g,
        &iri("http://ex/bolt"),
        Modality::TypedSubVector,
        &all,
        None,
        Some((&store, &header)),
    ) else {
        panic!()
    };
    assert_eq!(values, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], "all blocks");
    assert_eq!(blocks, vec![Encoder::Numeric, Encoder::Other]);
}

#[test]
fn typed_sub_vector_none_when_node_has_no_vector() {
    let g = Graph::load_str(TTL, "turtle").unwrap();
    let header =
        SchemaHeader::new(vec![Block::new(Encoder::Numeric, Metric::Euclidean, 0, 4)]).unwrap();
    let mut store = VectorStore::create(tmp("novec"), 4).unwrap();
    // Put a DIFFERENT node's vector; bolt has none.
    let other = g.id_of(&iri("http://ex/Car")).unwrap();
    store.put(other, &[1.0, 2.0, 3.0, 4.0]).unwrap();
    store.finalize().unwrap();
    assert!(
        ground(
            &g,
            &iri("http://ex/bolt"),
            Modality::TypedSubVector,
            &GroundingConfig::default(),
            None,
            Some((&store, &header)),
        )
        .is_none(),
        "a node with no stored vector grounds to None for TypedSubVector"
    );
}

#[test]
fn completeness_is_profile_relative_closure_vs_no_closure() {
    // The entailed fact `ex:bolt a ex:Vehicle` exists only after the RDFS closure.
    let asserted = Graph::load_str(TTL, "turtle").unwrap();
    let closed = close_for_vectorise(TTL, "turtle", Profile::Rdfs)
        .unwrap()
        .graph;

    let cfg = GroundingConfig {
        minimal_type_pattern: false,
        ..Default::default()
    };

    let has_vehicle = |g: &Graph| -> bool {
        match ground(
            g,
            &iri("http://ex/bolt"),
            Modality::Subgraph,
            &cfg,
            None,
            None,
        ) {
            Some(Grounding::Subgraph(facts)) => facts.iter().any(|f| {
                f.predicate == "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
                    && f.object == "http://ex/Vehicle"
            }),
            _ => false,
        }
    };

    // Over the CLOSED graph the entailed type is present (complete relative to RDFS).
    assert!(
        has_vehicle(&closed),
        "closure-before-grounding must surface the entailed Vehicle type"
    );
    // Over the UN-closed graph it is silently absent — the honest profile-relative boundary.
    assert!(
        !has_vehicle(&asserted),
        "an un-closed graph is incomplete outside the asserted facts (no answer-completeness claim)"
    );
}

#[test]
fn dispatch_via_output_type_grounds_each_modality() {
    let g = close_for_vectorise(TTL, "turtle", Profile::Rdfs)
        .unwrap()
        .graph;
    let node = iri("http://ex/bolt");

    // Facts → Subgraph.
    let m = Modality::for_output(OutputType::Facts);
    assert!(matches!(
        ground(&g, &node, m, &GroundingConfig::default(), None, None),
        Some(Grounding::Subgraph(_))
    ));
    // Text → NlString.
    let m = Modality::for_output(OutputType::Text);
    assert!(matches!(
        ground(&g, &node, m, &GroundingConfig::default(), None, None),
        Some(Grounding::NlString(_))
    ));
    // Value → TypedValue (bolt has seats/electric).
    let m = Modality::for_output(OutputType::Value);
    assert!(matches!(
        ground(&g, &node, m, &GroundingConfig::default(), None, None),
        Some(Grounding::TypedValue(_))
    ));
    // Ambiguous → the EXACT subgraph default.
    let m = Modality::for_output(OutputType::Ambiguous);
    assert!(matches!(
        ground(&g, &node, m, &GroundingConfig::default(), None, None),
        Some(Grounding::Subgraph(_))
    ));
}
