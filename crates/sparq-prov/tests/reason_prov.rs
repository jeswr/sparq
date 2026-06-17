//! End-to-end: sparq-reason `why()` proof tree → PROV-O lineage RDF.
//! [OPUS-4.8] sq-m3i0 — reasoner-materialization lineage.
#![cfg(feature = "reason")]

use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use oxrdf::vocab::xsd;
use oxrdf::Triple;
use sparq_core::dict::{Dict, Id};
use sparq_prov::{prov_from_proof, ProvProofConfig};
use sparq_reason::n3::Term as N3Term;
use sparq_reason::{MaterializedGraph, MaterializedN3Graph};

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDFS_SUBCLASS: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
const PROV: &str = "http://www.w3.org/ns/prov#";
const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";

/// Build an RDFS graph over `(rex a Dog), (Dog ⊑ Animal)` and return the graph,
/// the dict, and the id of the inferred fact `(rex a Animal)` (via cax-sco/rdfs9).
fn dog_setup() -> (MaterializedGraph, Dict, [Id; 3]) {
    let mut dict = Dict::default();
    let i = |d: &mut Dict, s: &str| d.intern_iri(s);
    let rex = i(&mut dict, "http://ex/rex");
    let dog = i(&mut dict, "http://ex/Dog");
    let animal = i(&mut dict, "http://ex/Animal");
    let ty = i(&mut dict, RDF_TYPE);
    let sc = i(&mut dict, RDFS_SUBCLASS);
    let base = vec![[rex, ty, dog], [dog, sc, animal]];
    let g = MaterializedGraph::new(&mut dict, &base);
    (g, dict, [rex, ty, animal])
}

/// All distinct predicate IRIs in a graph.
fn preds(g: &[Triple]) -> HashSet<String> {
    g.iter().map(|t| t.predicate.as_str().to_string()).collect()
}

/// Map subject IRI -> set of (predicate, object-IRI-or-literal-lexical) for assertions.
fn lines(g: &[Triple]) -> HashSet<String> {
    g.iter()
        .map(|t| format!("{} {} {}", t.subject, t.predicate, t.object))
        .collect()
}

#[test]
fn derived_fact_proof_emits_entity_activity_and_lineage() {
    let (g, dict, fact) = dog_setup();
    let proof = g.why(&dict, fact).expect("inferred fact has a proof");
    // The proof: root (rex a Animal) ← rule(rdfs9/cax-sco) ← 2 asserted leaves.
    assert!(
        proof.nodes().len() >= 3,
        "expected a multi-node proof, got {}",
        proof.nodes().len()
    );

    let prov = prov_from_proof(&proof, &ProvProofConfig::default());
    let p = preds(&prov);
    // Core PROV-O lineage predicates are present.
    assert!(
        p.contains(&format!("{PROV}wasGeneratedBy")),
        "missing wasGeneratedBy: {p:?}"
    );
    assert!(p.contains(&format!("{PROV}used")), "missing used");
    assert!(
        p.contains(&format!("{PROV}wasDerivedFrom")),
        "missing wasDerivedFrom"
    );

    // Exactly one prov:Activity per non-leaf node; here the single rule firing.
    let activities: Vec<&Triple> = prov
        .iter()
        .filter(|t| {
            t.predicate.as_str() == RDF_TYPE && t.object.to_string() == format!("<{PROV}Activity>")
        })
        .collect();
    assert_eq!(activities.len(), 1, "one rule firing ⇒ one Activity");

    // Every premise that the activity `used` is also `wasDerivedFrom` by the result.
    let used: HashSet<String> = prov
        .iter()
        .filter(|t| t.predicate.as_str() == format!("{PROV}used"))
        .map(|t| t.object.to_string())
        .collect();
    let derived_from: HashSet<String> = prov
        .iter()
        .filter(|t| t.predicate.as_str() == format!("{PROV}wasDerivedFrom"))
        .map(|t| t.object.to_string())
        .collect();
    assert_eq!(used, derived_from, "used inputs == wasDerivedFrom inputs");
    assert_eq!(
        used.len(),
        2,
        "rdfs9 has two premises (typing + subClassOf)"
    );

    // The activity carries the rule id as an rdfs:label (cax-sco / rdfs9).
    let label = prov
        .iter()
        .find(|t| t.predicate.as_str() == RDFS_LABEL && t.subject.to_string().contains(":rule:"))
        .map(|t| t.object.to_string())
        .expect("rule activity has a label");
    assert!(
        label.contains("rdfs9") || label.contains("cax-sco"),
        "unexpected rule label: {label}"
    );
}

#[test]
fn asserted_leaves_are_entities_without_a_generating_activity() {
    let (g, dict, fact) = dog_setup();
    let proof = g.why(&dict, fact).unwrap();
    let prov = prov_from_proof(&proof, &ProvProofConfig::default());

    // Subjects that wasGeneratedBy something = derived entities.
    let generated: HashSet<String> = prov
        .iter()
        .filter(|t| t.predicate.as_str() == format!("{PROV}wasGeneratedBy"))
        .map(|t| t.subject.to_string())
        .collect();
    // Every fact node is typed prov:Entity.
    let entities: HashSet<String> = prov
        .iter()
        .filter(|t| {
            t.predicate.as_str() == RDF_TYPE && t.object.to_string() == format!("<{PROV}Entity>")
        })
        .map(|t| t.subject.to_string())
        .collect();
    // 3 distinct facts (2 leaves + 1 root) ⇒ 3 entities; exactly 1 is generated.
    assert_eq!(entities.len(), 3);
    assert_eq!(generated.len(), 1);
    assert!(generated.is_subset(&entities));
    // The two leaves have no generating activity.
    assert_eq!(entities.difference(&generated).count(), 2);
}

#[test]
fn lineage_is_valid_rdf_and_round_trips() {
    let (g, dict, fact) = dog_setup();
    let proof = g.why(&dict, fact).unwrap();
    let prov = prov_from_proof(&proof, &ProvProofConfig::default());
    let nt = sparq_engine::triples_to_ntriples(&prov);

    let reloaded = sparq_core::Graph::load_str(&nt, "turtle").expect("PROV-O must be valid RDF");
    assert_eq!(reloaded.len(), prov.len());

    use oxttl::NTriplesParser;
    let parsed: Vec<_> = NTriplesParser::new()
        .for_reader(nt.as_bytes())
        .collect::<Result<_, _>>()
        .expect("emitted lineage must be well-formed N-Triples");
    assert_eq!(parsed.len(), prov.len());
}

#[test]
fn deterministic_and_content_addressed() {
    let (g, dict, fact) = dog_setup();
    let proof = g.why(&dict, fact).unwrap();
    let a = prov_from_proof(&proof, &ProvProofConfig::default());
    let b = prov_from_proof(&proof, &ProvProofConfig::default());
    assert_eq!(
        lines(&a),
        lines(&b),
        "same proof + config ⇒ identical lineage"
    );
    // The minted entity/activity IRIs sit under the default namespace.
    assert!(a
        .iter()
        .any(|t| t.subject.to_string().contains("urn:sparq:prov:fact:")));
    assert!(a
        .iter()
        .any(|t| t.subject.to_string().contains("urn:sparq:prov:rule:")));
}

#[test]
fn shared_subfacts_stitch_across_proofs() {
    // Two derived facts that share the asserted typing premise (rex a Dog) must
    // name the SAME entity IRI for that shared fact, so lineage DAGs join up.
    let mut dict = Dict::default();
    let i = |d: &mut Dict, s: &str| d.intern_iri(s);
    let rex = i(&mut dict, "http://ex/rex");
    let dog = i(&mut dict, "http://ex/Dog");
    let animal = i(&mut dict, "http://ex/Animal");
    let mammal = i(&mut dict, "http://ex/Mammal");
    let ty = i(&mut dict, RDF_TYPE);
    let sc = i(&mut dict, RDFS_SUBCLASS);
    // Dog ⊑ Mammal ⊑ Animal, rex a Dog ⇒ rex a Mammal and rex a Animal.
    let base = vec![[rex, ty, dog], [dog, sc, mammal], [mammal, sc, animal]];
    let g = MaterializedGraph::new(&mut dict, &base);

    let cfg = ProvProofConfig::default();
    let p1 = prov_from_proof(&g.why(&dict, [rex, ty, mammal]).unwrap(), &cfg);
    let p2 = prov_from_proof(&g.why(&dict, [rex, ty, animal]).unwrap(), &cfg);

    // Both proofs share the asserted leaf (rex a Dog), and the SAME fact must mint
    // the SAME content-addressed entity IRI — so the lineage DAGs join up.
    let ents1: HashSet<String> = p1
        .iter()
        .filter(|t| {
            t.predicate.as_str() == RDF_TYPE && t.object.to_string() == format!("<{PROV}Entity>")
        })
        .map(|t| t.subject.to_string())
        .collect();
    let ents2: HashSet<String> = p2
        .iter()
        .filter(|t| {
            t.predicate.as_str() == RDF_TYPE && t.object.to_string() == format!("<{PROV}Entity>")
        })
        .map(|t| t.subject.to_string())
        .collect();
    // At least one shared entity (the (rex a Dog) leaf) appears in both.
    assert!(
        ents1.intersection(&ents2).next().is_some(),
        "overlapping proofs must share at least the (rex a Dog) leaf entity"
    );
}

#[test]
fn clock_and_agent_are_recorded_on_activities() {
    let (g, dict, fact) = dog_setup();
    let proof = g.why(&dict, fact).unwrap();

    fn fixed() -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_700_000_000)
    }
    let cfg = ProvProofConfig {
        agent: Some(oxrdf::NamedNode::new_unchecked("http://ex/rdfs-reasoner")),
        ..ProvProofConfig::with_clock(fixed)
    };
    let prov = prov_from_proof(&proof, &cfg);

    // generatedAtTime on the activity, typed xsd:dateTime = 2023-11-14T22:13:20Z.
    let ts = prov
        .iter()
        .find(|t| t.predicate.as_str() == format!("{PROV}generatedAtTime"))
        .map(|t| t.object.to_string())
        .expect("activity is time-stamped");
    assert!(ts.contains("2023-11-14T22:13:20Z"), "got {ts}");
    assert!(ts.contains(xsd::DATE_TIME.as_str()));

    // wasAssociatedWith the configured agent.
    assert!(prov.iter().any(
        |t| t.predicate.as_str() == format!("{PROV}wasAssociatedWith")
            && t.object.to_string() == "<http://ex/rdfs-reasoner>"
    ));
}

#[test]
fn n3_rule_proof_emits_lineage_with_n3_rule_label() {
    // The same bridge works on N3 proofs (the bead is RDFS/OWL-RL/N3): a forward
    // rule firing becomes a prov:Activity labelled `n3-rule-<i>`.
    let ex = |l: &str| N3Term::Iri(format!("http://ex/{l}"));
    let rules = r#"
        @prefix : <http://ex/> .
        { ?x :parent ?y } => { ?x :ancestor ?y } .
    "#;
    let base = vec![[ex("a"), ex("parent"), ex("b")]];
    let g = MaterializedN3Graph::new(rules, &base).expect("rules parse");
    let fact = [ex("a"), ex("ancestor"), ex("b")];
    assert!(g.contains(&fact));
    let proof = g.why(&fact).expect("derived N3 fact explains");

    let prov = prov_from_proof(&proof, &ProvProofConfig::default());
    // The rule firing is a labelled Activity.
    let label = prov
        .iter()
        .find(|t| t.predicate.as_str() == RDFS_LABEL && t.subject.to_string().contains(":rule:"))
        .map(|t| t.object.to_string())
        .expect("rule activity has a label");
    assert!(
        label.contains("n3-rule-0"),
        "unexpected N3 rule label: {label}"
    );
    // wasGeneratedBy + a single used/wasDerivedFrom premise (the asserted parent edge).
    assert!(preds(&prov).contains(&format!("{PROV}wasGeneratedBy")));
    let used = prov
        .iter()
        .filter(|t| t.predicate.as_str() == format!("{PROV}used"))
        .count();
    assert_eq!(used, 1, "one premise: a :parent b");
    // Valid RDF.
    let nt = sparq_engine::triples_to_ntriples(&prov);
    assert!(sparq_core::Graph::load_str(&nt, "turtle").is_ok());
}

#[test]
fn asserted_fact_proof_is_a_lone_entity() {
    // why() on a base triple returns a single asserted leaf ⇒ one Entity, no Activity.
    let (g, dict, _) = dog_setup();
    let mut dict2 = dict;
    let rex = dict2.intern_iri("http://ex/rex");
    let dog = dict2.intern_iri("http://ex/Dog");
    let ty = dict2.intern_iri(RDF_TYPE);
    let proof = g
        .why(&dict2, [rex, ty, dog])
        .expect("asserted fact explains");
    assert_eq!(proof.nodes().len(), 1);
    let prov = prov_from_proof(&proof, &ProvProofConfig::default());
    // One Entity typing triple, nothing else (no activity, no edges).
    let mut by_pred: HashMap<String, usize> = HashMap::new();
    for t in &prov {
        *by_pred.entry(t.predicate.as_str().to_string()).or_default() += 1;
    }
    assert_eq!(by_pred.get(RDF_TYPE), Some(&1));
    assert!(!by_pred.contains_key(&format!("{PROV}wasGeneratedBy")));
}
