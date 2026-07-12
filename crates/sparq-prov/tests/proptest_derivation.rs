// [GPT-5.6] sq-hggpo: property tests for derivation PROV-O structure.

use std::collections::HashSet;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use oxrdf::{NamedNode, NamedOrBlankNode, Term, Triple};
use proptest::prelude::*;
use sparq_core::Graph;
use sparq_engine::QueryBudget;
use sparq_prov::{
    derive_construct, derive_construct_with_budget, derive_update, derive_update_with_budget,
    ProvConfig,
};

const PROV: &str = "http://www.w3.org/ns/prov#";

fn fixed_clock() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

fn config(inputs: &[u8]) -> ProvConfig {
    ProvConfig {
        activity: Some(NamedNode::new_unchecked("http://example.test/activity")),
        entity: Some(NamedNode::new_unchecked("http://example.test/output")),
        used: inputs
            .iter()
            .map(|id| NamedNode::new_unchecked(format!("http://example.test/input/{id}")))
            .collect(),
        agent: None,
        clock: fixed_clock,
    }
}

fn input_graph(ids: &[u8]) -> Graph {
    let triples = ids
        .iter()
        .map(|id| {
            format!(
                "<http://example.test/s/{id}> <http://example.test/p> <http://example.test/o/{id}> ."
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Graph::load_str(&triples, "ntriples").expect("generated input is valid N-Triples")
}

fn triple_key(triple: &Triple) -> String {
    format!("{} {} {}", triple.subject, triple.predicate, triple.object)
}

fn assert_round_trip(expected: &[Triple], serialized: &str) {
    let parsed = Graph::load_str(serialized, "ntriples")
        .expect("serialized provenance must parse through sparq-core");
    let actual: HashSet<String> = parsed
        .iter_ids()
        .map(|ids| {
            format!(
                "{} {} {}",
                parsed.dict.term(ids[0]),
                parsed.dict.term(ids[1]),
                parsed.dict.term(ids[2])
            )
        })
        .collect();
    let expected: HashSet<String> = expected.iter().map(triple_key).collect();
    assert_eq!(actual, expected, "serialization changed the PROV graph");
}

fn has_edge(graph: &[Triple], subject: &NamedNode, predicate: &str, object: &NamedNode) -> bool {
    graph.iter().any(|triple| {
        triple.subject == NamedOrBlankNode::NamedNode(subject.clone())
            && triple.predicate.as_str() == predicate
            && triple.object == Term::NamedNode(object.clone())
    })
}

fn distinct(mut values: Vec<u8>) -> Vec<u8> {
    values.sort_unstable();
    values.dedup();
    values
}

proptest! {
    #[test]
    fn construct_provenance_is_structurally_complete(
        source_ids in prop::collection::vec(0_u8..24, 0..12),
        input_ids in prop::collection::vec(0_u8..24, 0..12),
        budgeted in any::<bool>(),
    ) {
        let source_ids = distinct(source_ids);
        let input_ids = distinct(input_ids);
        let graph = input_graph(&source_ids);
        let query = "CONSTRUCT { ?s <http://example.test/outputPredicate> ?o } WHERE { ?s <http://example.test/p> ?o }";
        let derivation = if budgeted {
            derive_construct_with_budget(&graph, query, config(&input_ids), &QueryBudget::unlimited())
        } else {
            derive_construct(&graph, query, config(&input_ids))
        }.expect("generated CONSTRUCT must execute");

        prop_assert_eq!(derivation.triples().len(), source_ids.len());
        let provenance = derivation.prov_graph();
        let used_predicate = format!("{PROV}used");
        let generated_predicate = format!("{PROV}wasGeneratedBy");
        for id in &input_ids {
            let input = NamedNode::new_unchecked(format!("http://example.test/input/{id}"));
            prop_assert!(has_edge(&provenance, derivation.activity(), &used_predicate, &input));
        }
        prop_assert!(has_edge(
            &provenance,
            derivation.entity(),
            &generated_predicate,
            derivation.activity(),
        ));
        let (started, ended) = derivation.timing();
        prop_assert!(started <= ended);
        assert_round_trip(&provenance, &derivation.prov_ntriples());
    }

    #[test]
    fn update_provenance_is_structurally_complete(
        output_ids in prop::collection::vec(0_u8..24, 0..12),
        input_ids in prop::collection::vec(0_u8..24, 0..12),
        budgeted in any::<bool>(),
    ) {
        let output_ids = distinct(output_ids);
        let input_ids = distinct(input_ids);
        let mut graph = Graph::load_str("", "ntriples").expect("empty graph is valid");
        let body = output_ids.iter().map(|id| {
            format!("<http://example.test/s/{id}> <http://example.test/p> <http://example.test/o/{id}> .")
        }).collect::<Vec<_>>().join(" ");
        let update = format!("INSERT DATA {{ {body} }}");
        let derivation = if budgeted {
            derive_update_with_budget(&mut graph, &update, config(&input_ids), &QueryBudget::unlimited())
        } else {
            derive_update(&mut graph, &update, config(&input_ids))
        }.expect("generated INSERT DATA must execute");

        prop_assert_eq!(derivation.inserted().len(), output_ids.len());
        let provenance = derivation.prov_graph();
        let used_predicate = format!("{PROV}used");
        let generated_predicate = format!("{PROV}wasGeneratedBy");
        for id in &input_ids {
            let input = NamedNode::new_unchecked(format!("http://example.test/input/{id}"));
            prop_assert!(has_edge(&provenance, derivation.activity(), &used_predicate, &input));
        }
        if output_ids.is_empty() {
            prop_assert!(!has_edge(
                &provenance,
                derivation.entity(),
                &generated_predicate,
                derivation.activity(),
            ));
        } else {
            prop_assert!(has_edge(
                &provenance,
                derivation.entity(),
                &generated_predicate,
                derivation.activity(),
            ));
        }
        let (started, ended) = derivation.timing();
        prop_assert!(started <= ended);
        assert_round_trip(&provenance, &derivation.prov_ntriples());
    }
}
