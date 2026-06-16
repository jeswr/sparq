//! Unit tests for the CONSTRUCT → PROV-O lineage path. [OPUS-4.8] sq-ntcg.

use super::*;
use oxrdf::vocab::xsd;
use std::collections::HashSet;
use std::time::Duration;

const DATA: &str = r#"
    @prefix ex: <http://ex/> .
    ex:alice ex:age 30 ; ex:name "Alice" .
    ex:bob   ex:age 25 ; ex:name "Bob" .
"#;

fn g() -> Graph {
    Graph::load_str(DATA, "turtle").unwrap()
}

/// A fixed clock so activity timing (and the minted IRIs) are deterministic.
fn fixed_clock() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

fn cfg() -> ProvConfig {
    ProvConfig {
        clock: fixed_clock,
        used: vec![NamedNode::new_unchecked("http://ex/source-graph")],
        ..ProvConfig::default()
    }
}

/// Render a triple as an N-Triples line for substring assertions.
fn line(t: &Triple) -> String {
    format!("{} {} {} .", t.subject, t.predicate, t.object)
}

#[test]
fn construct_derivation_emits_core_prov_shape() {
    let d = derive_construct(
        &g(),
        "PREFIX ex: <http://ex/> CONSTRUCT { ?s ex:years ?a } WHERE { ?s ex:age ?a }",
        cfg(),
    )
    .unwrap();

    // The derived data itself: one renamed triple per subject.
    assert_eq!(d.triples().len(), 2);

    let prov = d.prov_graph();
    let lines: HashSet<String> = prov.iter().map(line).collect();
    let a = d.activity().as_str();
    let e = d.entity().as_str();

    // Types.
    assert!(lines.contains(&format!(
        "<{a}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/prov#Activity> ."
    )));
    assert!(lines.contains(&format!(
        "<{e}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/prov#Entity> ."
    )));
    // Generation + derivation edges.
    assert!(lines.contains(&format!(
        "<{e}> <http://www.w3.org/ns/prov#wasGeneratedBy> <{a}> ."
    )));
    assert!(lines.contains(&format!(
        "<{a}> <http://www.w3.org/ns/prov#used> <http://ex/source-graph> ."
    )));
    assert!(lines.contains(&format!(
        "<{e}> <http://www.w3.org/ns/prov#wasDerivedFrom> <http://ex/source-graph> ."
    )));
    // Timing: the fixed clock = 2023-11-14T22:13:20Z, typed xsd:dateTime.
    let dt = format!(
        "<{a}> <http://www.w3.org/ns/prov#startedAtTime> \"2023-11-14T22:13:20Z\"^^<{}> .",
        xsd::DATE_TIME.as_str()
    );
    assert!(lines.contains(&dt), "missing startedAtTime; got {lines:#?}");
    assert!(lines.iter().any(|l| l.contains("#endedAtTime>")));
}

#[test]
fn prov_graph_is_valid_rdf_and_round_trips() {
    let d = derive_construct(
        &g(),
        "PREFIX ex: <http://ex/> CONSTRUCT { ?s ex:years ?a } WHERE { ?s ex:age ?a }",
        cfg(),
    )
    .unwrap();

    // N-Triples ⊂ Turtle: the emitted lineage must load back as a graph...
    let nt = d.prov_ntriples();
    let reloaded = Graph::load_str(&nt, "turtle").expect("PROV-O output must be valid RDF");
    assert_eq!(reloaded.len(), d.prov_graph().len());

    // ...and parse cleanly with an independent N-Triples parser too.
    use oxttl::NTriplesParser;
    let parsed: Vec<_> = NTriplesParser::new()
        .for_reader(nt.as_bytes())
        .collect::<Result<_, _>>()
        .expect("emitted lineage must be well-formed N-Triples");
    assert_eq!(parsed.len(), d.prov_graph().len());
}

#[test]
fn explicit_iris_and_agent_are_honoured() {
    let activity = NamedNode::new_unchecked("http://ex/act/1");
    let entity = NamedNode::new_unchecked("http://ex/result/1");
    let agent = NamedNode::new_unchecked("http://ex/service");
    let config = ProvConfig {
        activity: Some(activity.clone()),
        entity: Some(entity.clone()),
        agent: Some(agent.clone()),
        used: vec![NamedNode::new_unchecked("http://ex/g")],
        clock: fixed_clock,
    };
    let d = derive_construct(&g(), "DESCRIBE <http://ex/alice>", config).unwrap();
    assert_eq!(d.activity(), &activity);
    assert_eq!(d.entity(), &entity);

    let lines: HashSet<String> = d.prov_graph().iter().map(line).collect();
    assert!(lines.contains(
        "<http://ex/act/1> <http://www.w3.org/ns/prov#wasAssociatedWith> <http://ex/service> ."
    ));
    // DESCRIBE is a derivation too (CBD of alice = her 2 outbound triples).
    assert_eq!(d.triples().len(), 2);
}

#[test]
fn minted_iris_are_stable_for_the_same_derivation() {
    let q = "PREFIX ex: <http://ex/> CONSTRUCT { ?s ex:years ?a } WHERE { ?s ex:age ?a }";
    let a = derive_construct(&g(), q, cfg()).unwrap();
    let b = derive_construct(&g(), q, cfg()).unwrap();
    // Same query + same (fixed) clock ⇒ same minted activity/entity IRIs.
    assert_eq!(a.activity(), b.activity());
    assert_eq!(a.entity(), b.entity());
    // A different query mints different nodes.
    let c = derive_construct(
        &g(),
        "PREFIX ex: <http://ex/> CONSTRUCT { ?s ex:n ?n } WHERE { ?s ex:name ?n }",
        cfg(),
    )
    .unwrap();
    assert_ne!(a.activity(), c.activity());
}

#[test]
fn empty_inputs_omit_used_and_derived_edges() {
    let config = ProvConfig {
        clock: fixed_clock,
        ..ProvConfig::default()
    };
    let d = derive_construct(
        &g(),
        "PREFIX ex: <http://ex/> CONSTRUCT { ?s ex:years ?a } WHERE { ?s ex:age ?a }",
        config,
    )
    .unwrap();
    let prov = d.prov_graph();
    assert!(prov
        .iter()
        .all(|t| t.predicate.as_str() != "http://www.w3.org/ns/prov#used"));
    assert!(prov
        .iter()
        .all(|t| t.predicate.as_str() != "http://www.w3.org/ns/prov#wasDerivedFrom"));
    // The core activity/entity/generation triples are still present.
    assert!(prov
        .iter()
        .any(|t| t.predicate.as_str() == "http://www.w3.org/ns/prov#wasGeneratedBy"));
}

#[test]
fn rejects_non_graph_queries() {
    assert!(derive_construct(&g(), "SELECT * WHERE { ?s ?p ?o }", cfg()).is_err());
    assert!(derive_construct(&g(), "ASK { ?s ?p ?o }", cfg()).is_err());
}
