// [FABLE-5] sq-pbz04.4.1 (epic sq-pbz04.4): integration tests for the FAIL-CLOSED reverse RDF
// mapping (design record research/owl2-direct-semantics-scoping.md §L1).
//
// 🤖 SPARQ agent. Two families:
//   * ACCEPT — every ALCH class-expression / axiom / assertion shape maps to the expected
//     structural model (hand-built and asserted with `PartialEq`), plus a model→RDF→model
//     round-trip on the flat subset.
//   * REJECT — ONE test per rejection-taxonomy arm (OutOfFragment, DataConstruct, MalformedList,
//     MalformedClassExpression, Unclassifiable), plus the load-bearing fail-closed invariant:
//     a single out-of-fragment triple rejects the WHOLE graph (never a partial model).

use oxrdf::{NamedNode, Term as OTerm};
use sparq_core::dict::{Dict, Id};
use sparq_core::Graph;
use sparq_reason_dl::model::{Axiom, ClassExpression as CE, ObjectPropertyExpression as OPE};
use sparq_reason_dl::{extract, ExtractError, Ontology};

const OWL: &str = "http://www.w3.org/2002/07/owl#";
const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";

const PRE: &str = r#"
    @prefix : <http://ex/> .
    @prefix owl: <http://www.w3.org/2002/07/owl#> .
    @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
    @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
    @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
"#;

fn parse(body: &str) -> (Dict, Vec<[Id; 3]>) {
    Graph::parse_to_triples(&format!("{PRE}{body}"), "turtle").expect("parse")
}

fn iri(dict: &Dict, full: &str) -> Id {
    dict.lookup(&OTerm::NamedNode(NamedNode::new_unchecked(full.to_string())))
}

/// `http://ex/{frag}` dict id.
fn ex(dict: &Dict, frag: &str) -> Id {
    iri(dict, &format!("http://ex/{frag}"))
}

fn class(dict: &Dict, frag: &str) -> CE {
    CE::Class(ex(dict, frag))
}

fn prop(dict: &Dict, frag: &str) -> OPE {
    OPE::ObjectProperty(ex(dict, frag))
}

// ============================== ACCEPT ==============================

#[test]
fn subclassof_named() {
    let (d, t) = parse(":A rdfs:subClassOf :B .");
    let o = extract(&d, &t).expect("accept");
    assert_eq!(
        o.axioms,
        vec![Axiom::SubClassOf {
            sub: class(&d, "A"),
            sup: class(&d, "B"),
        }]
    );
}

#[test]
fn equivalent_and_disjoint() {
    let (d, t) = parse(":A owl:equivalentClass :B .\n:C owl:disjointWith :D .");
    let o = extract(&d, &t).expect("accept");
    assert!(o
        .axioms
        .contains(&Axiom::EquivalentClasses(class(&d, "A"), class(&d, "B"))));
    assert!(o
        .axioms
        .contains(&Axiom::DisjointClasses(class(&d, "C"), class(&d, "D"))));
    assert_eq!(o.len(), 2);
}

#[test]
fn some_values_from() {
    let (d, t) = parse(":A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :r ; owl:someValuesFrom :C ] .");
    let o = extract(&d, &t).expect("accept");
    assert_eq!(
        o.axioms,
        vec![Axiom::SubClassOf {
            sub: class(&d, "A"),
            sup: CE::ObjectSomeValuesFrom(prop(&d, "r"), Box::new(class(&d, "C"))),
        }]
    );
}

#[test]
fn all_values_from() {
    let (d, t) = parse(":A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :r ; owl:allValuesFrom :C ] .");
    let o = extract(&d, &t).expect("accept");
    assert_eq!(
        o.axioms,
        vec![Axiom::SubClassOf {
            sub: class(&d, "A"),
            sup: CE::ObjectAllValuesFrom(prop(&d, "r"), Box::new(class(&d, "C"))),
        }]
    );
}

#[test]
fn intersection_of() {
    let (d, t) = parse("[ owl:intersectionOf ( :A :B ) ] rdfs:subClassOf :C .");
    let o = extract(&d, &t).expect("accept");
    assert_eq!(
        o.axioms,
        vec![Axiom::SubClassOf {
            sub: CE::ObjectIntersectionOf(vec![class(&d, "A"), class(&d, "B")]),
            sup: class(&d, "C"),
        }]
    );
}

#[test]
fn union_of() {
    let (d, t) = parse(":C rdfs:subClassOf [ owl:unionOf ( :A :B ) ] .");
    let o = extract(&d, &t).expect("accept");
    assert_eq!(
        o.axioms,
        vec![Axiom::SubClassOf {
            sub: class(&d, "C"),
            sup: CE::ObjectUnionOf(vec![class(&d, "A"), class(&d, "B")]),
        }]
    );
}

#[test]
fn complement_of() {
    let (d, t) = parse(":A rdfs:subClassOf [ owl:complementOf :B ] .");
    let o = extract(&d, &t).expect("accept");
    assert_eq!(
        o.axioms,
        vec![Axiom::SubClassOf {
            sub: class(&d, "A"),
            sup: CE::ObjectComplementOf(Box::new(class(&d, "B"))),
        }]
    );
}

#[test]
fn nested_class_expression() {
    // A ⊑ ∃r.(B ⊓ ¬C) — exercises recursion through restriction → intersection → complement.
    let (d, t) = parse(
        ":A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :r ; owl:someValuesFrom \
         [ owl:intersectionOf ( :B [ owl:complementOf :C ] ) ] ] .",
    );
    let o = extract(&d, &t).expect("accept");
    assert_eq!(
        o.axioms,
        vec![Axiom::SubClassOf {
            sub: class(&d, "A"),
            sup: CE::ObjectSomeValuesFrom(
                prop(&d, "r"),
                Box::new(CE::ObjectIntersectionOf(vec![
                    class(&d, "B"),
                    CE::ObjectComplementOf(Box::new(class(&d, "C"))),
                ])),
            ),
        }]
    );
}

#[test]
fn sub_property_domain_range() {
    let (d, t) = parse(":r rdfs:subPropertyOf :s .\n:r rdfs:domain :A .\n:r rdfs:range :B .");
    let o = extract(&d, &t).expect("accept");
    assert!(o.axioms.contains(&Axiom::SubObjectPropertyOf {
        sub: prop(&d, "r"),
        sup: prop(&d, "s"),
    }));
    assert!(o.axioms.contains(&Axiom::ObjectPropertyDomain {
        property: prop(&d, "r"),
        domain: class(&d, "A"),
    }));
    assert!(o.axioms.contains(&Axiom::ObjectPropertyRange {
        property: prop(&d, "r"),
        range: class(&d, "B"),
    }));
    assert_eq!(o.len(), 3);
}

#[test]
fn class_assertion() {
    let (d, t) = parse(":x a :A .");
    let o = extract(&d, &t).expect("accept");
    assert_eq!(
        o.axioms,
        vec![Axiom::ClassAssertion {
            class: class(&d, "A"),
            individual: ex(&d, "x"),
        }]
    );
}

#[test]
fn object_property_assertion_declared() {
    let (d, t) = parse(":p a owl:ObjectProperty .\n:x :p :y .");
    let o = extract(&d, &t).expect("accept");
    assert_eq!(
        o.axioms,
        vec![Axiom::ObjectPropertyAssertion {
            property: prop(&d, "p"),
            source: ex(&d, "x"),
            target: ex(&d, "y"),
        }]
    );
}

#[test]
fn thing_and_nothing() {
    let (d, t) = parse(":A rdfs:subClassOf owl:Thing .\n:B rdfs:subClassOf owl:Nothing .\n:x a owl:Thing .");
    let o = extract(&d, &t).expect("accept");
    assert!(o.axioms.contains(&Axiom::SubClassOf {
        sub: class(&d, "A"),
        sup: CE::Thing,
    }));
    assert!(o.axioms.contains(&Axiom::SubClassOf {
        sub: class(&d, "B"),
        sup: CE::Nothing,
    }));
    assert!(o.axioms.contains(&Axiom::ClassAssertion {
        class: CE::Thing,
        individual: ex(&d, "x"),
    }));
}

#[test]
fn annotations_and_declarations_ignored() {
    let (d, t) = parse(
        ":A a owl:Class .\n:A rdfs:label \"a class\" .\n:A rdfs:comment \"note\" .\n\
         :O a owl:Ontology .\n:O owl:versionInfo \"1.0\" .\n:r a owl:ObjectProperty .",
    );
    let o = extract(&d, &t).expect("accept");
    assert!(o.is_empty(), "no logical axioms, got {:?}", o.axioms);
}

// ============================== ROUND-TRIP ==============================

/// Renders the FLAT axiom subset (named-class `SubClassOf`, named `ClassAssertion`, and role
/// assertions) back to RDF triples, interning an `owl:ObjectProperty` declaration for each
/// asserting property so the reverse mapping recognises it. Complex class expressions are NOT
/// round-tripped (their RDF encoding mints fresh blank nodes, so blank-node identity is not
/// preserved) — hence "where applicable".
fn render_flat(dict: &mut Dict, onto: &Ontology) -> Vec<[Id; 3]> {
    let ty = iri(dict, &format!("{RDF}type"));
    let sco = iri(dict, &format!("{RDFS}subClassOf"));
    let owl_obj = iri(dict, &format!("{OWL}ObjectProperty"));
    let mut out = Vec::new();
    let mut declared = std::collections::HashSet::new();
    for ax in &onto.axioms {
        match ax {
            Axiom::SubClassOf {
                sub: CE::Class(a),
                sup: CE::Class(b),
            } => out.push([*a, sco, *b]),
            Axiom::ClassAssertion {
                class: CE::Class(c),
                individual,
            } => out.push([*individual, ty, *c]),
            Axiom::ObjectPropertyAssertion {
                property: OPE::ObjectProperty(p),
                source,
                target,
            } => {
                if declared.insert(*p) {
                    out.push([*p, ty, owl_obj]);
                }
                out.push([*source, *p, *target]);
            }
            other => panic!("render_flat only supports the flat subset, got {:?}", other),
        }
    }
    out
}

#[test]
fn round_trip_flat_subset() {
    let (mut d, t) =
        parse(":A rdfs:subClassOf :B .\n:x a :A .\n:p a owl:ObjectProperty .\n:x :p :y .");
    let onto1 = extract(&d, &t).expect("accept");
    let t2 = render_flat(&mut d, &onto1);
    let onto2 = extract(&d, &t2).expect("re-extract");
    assert_eq!(onto1.axioms, onto2.axioms, "model → RDF → model must be identity");
    // Sanity: the flat subset really was present.
    assert_eq!(onto1.len(), 3);
}

// ============================== REJECT (one per taxonomy arm) ==============================

#[test]
fn reject_out_of_fragment_cardinality() {
    let (d, t) = parse(
        ":A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :r ; owl:maxCardinality 1 ] .",
    );
    assert!(matches!(
        extract(&d, &t),
        Err(ExtractError::OutOfFragment(_))
    ));
}

#[test]
fn reject_out_of_fragment_sameas() {
    let (d, t) = parse(":x owl:sameAs :y .");
    assert!(matches!(
        extract(&d, &t),
        Err(ExtractError::OutOfFragment(_))
    ));
}

#[test]
fn reject_out_of_fragment_inverse() {
    let (d, t) = parse(":r owl:inverseOf :s .");
    assert!(matches!(
        extract(&d, &t),
        Err(ExtractError::OutOfFragment(_))
    ));
}

#[test]
fn reject_out_of_fragment_nominal_oneof() {
    let (d, t) = parse(":A owl:oneOf ( :a :b ) .");
    assert!(matches!(
        extract(&d, &t),
        Err(ExtractError::OutOfFragment(_))
    ));
}

#[test]
fn reject_out_of_fragment_property_characteristic() {
    let (d, t) = parse(":p a owl:TransitiveProperty .");
    assert!(matches!(
        extract(&d, &t),
        Err(ExtractError::OutOfFragment(_))
    ));
}

#[test]
fn reject_data_construct_literal_in_class() {
    let (d, t) = parse(":A rdfs:subClassOf \"a literal\" .");
    assert!(matches!(
        extract(&d, &t),
        Err(ExtractError::DataConstruct(_))
    ));
}

#[test]
fn reject_data_construct_data_property() {
    let (d, t) = parse(":p a owl:DatatypeProperty .\n:x :p \"30\"^^xsd:integer .");
    assert!(matches!(
        extract(&d, &t),
        Err(ExtractError::DataConstruct(_))
    ));
}

#[test]
fn reject_malformed_list_unterminated() {
    // A list cell with rdf:first but no rdf:rest.
    let (d, t) = parse(":C rdfs:subClassOf _:x .\n_:x owl:intersectionOf _:l .\n_:l rdf:first :A .");
    assert!(matches!(
        extract(&d, &t),
        Err(ExtractError::MalformedList(_))
    ));
}

#[test]
fn reject_malformed_list_empty() {
    let (d, t) = parse(":C rdfs:subClassOf [ owl:intersectionOf ( ) ] .");
    assert!(matches!(
        extract(&d, &t),
        Err(ExtractError::MalformedList(_))
    ));
}

#[test]
fn reject_malformed_restriction_missing_on_property() {
    let (d, t) = parse(":A rdfs:subClassOf [ a owl:Restriction ; owl:someValuesFrom :C ] .");
    assert!(matches!(
        extract(&d, &t),
        Err(ExtractError::MalformedClassExpression(_))
    ));
}

#[test]
fn reject_malformed_restriction_missing_filler() {
    let (d, t) = parse(":A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :r ] .");
    assert!(matches!(
        extract(&d, &t),
        Err(ExtractError::MalformedClassExpression(_))
    ));
}

#[test]
fn reject_unclassifiable_undeclared_predicate() {
    // :x :undeclared :y — :undeclared is neither a known annotation property nor a declared
    // object property, and :y is an IRI: a role assertion and an annotation are indistinguishable.
    let (d, t) = parse(":x :undeclared :y .");
    assert!(matches!(
        extract(&d, &t),
        Err(ExtractError::Unclassifiable(_))
    ));
}

// ============================== FAIL-CLOSED INVARIANT ==============================

#[test]
fn fail_closed_rejects_whole_graph_on_one_bad_triple() {
    // The load-bearing soundness invariant: a graph that is 99% in-fragment but contains ONE
    // out-of-fragment triple is REFUSED whole — never returned as a partial model that silently
    // dropped the offending axiom (which could flip a downstream consistency verdict).
    let (d, t) = parse(":A rdfs:subClassOf :B .\n:B rdfs:subClassOf :C .\n:x owl:sameAs :y .");
    assert!(
        matches!(extract(&d, &t), Err(ExtractError::OutOfFragment(_))),
        "one out-of-fragment triple must reject the whole graph"
    );
}
