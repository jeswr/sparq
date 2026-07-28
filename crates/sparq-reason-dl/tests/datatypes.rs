// [SONNET-4.6] sq-pbz04.4.19 (epic sq-pbz04.4) — acceptance tests for the datatype-aware
// L1 + the concrete-domain satisfiability oracle wired into the L3 ALCH(D) tableau
// (design record research/owl2-direct-semantics-scoping.md §5c; tableau module docs §5b).
//
// 🤖 SPARQ agent. Three families:
//   (a) the HEADLINE UNLOCK — two datatype ranges from DISJOINT value-space families make a
//       data property's extension provably empty, so a class demanding a value for it is
//       UNSATISFIABLE. This is the WebOnt-I5.3-015 mechanism that the pre-sq-pbz04.4.19
//       fragment could only fail closed on (sq-pbz04.4.9);
//   (b) the BOUNDARY — everything the oracle does not decide EXACTLY still refuses
//       extraction fail-closed with `DataConstruct`, in BOTH feature states;
//   (c) round-trip — a concrete-domain model renders back to RDF and re-extracts equal.
//
// The whole file is gated on `dl_datatypes`: with the feature off there is nothing to test
// here (the boundary cases are covered by the always-on `tests/extract.rs` arm).
#![cfg(feature = "dl_datatypes")]

use sparq_core::dict::Dict;
use sparq_core::Graph;
use sparq_reason_dl::cdomain::Datatype;
use sparq_reason_dl::model::{Axiom, ClassExpression as CE, DataPropertyExpression as DPE, DataRange};
use sparq_reason_dl::tableau::{class_satisfiability, consistency_from_extraction, Budget, Verdict};
use sparq_reason_dl::{extract, render_to_triples, ExtractError};

const PREFIXES: &str = "@prefix owl: <http://www.w3.org/2002/07/owl#> .\n\
                        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
                        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
                        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
                        @prefix : <http://ex/> .\n";

fn parse(body: &str) -> (Dict, Vec<[sparq_core::dict::Id; 3]>) {
    Graph::parse_to_triples(&format!("{}{}", PREFIXES, body), "turtle").expect("parse")
}

/// The dict id of `:A`, for building a class-satisfiability query over an extracted model.
fn class_a(dict: &Dict) -> CE {
    CE::Class(dict.lookup(&oxrdf::Term::NamedNode(
        oxrdf::NamedNode::new_unchecked("http://ex/A"),
    )))
}

// -------------------------------------------------------------------------------------------
// (a) The headline unlock — value-space disjointness decides
// -------------------------------------------------------------------------------------------

/// Two `rdfs:range` axioms from DISJOINT datatype families (`xsd:integer` / `xsd:string`)
/// leave the data property with an EMPTY extension, so `A ⊑ ∃p.rdfs:Literal` makes `A`
/// unsatisfiable. Before sq-pbz04.4.19 this input refused extraction outright
/// (sq-pbz04.4.9's fail-closed boundary); now it gets a definitive verdict.
#[test]
fn disjoint_datatype_ranges_make_the_demanding_class_unsatisfiable() {
    let (dict, triples) = parse(
        ":p a owl:DatatypeProperty ; rdfs:range xsd:integer ; rdfs:range xsd:string .\n\
         :A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :p ; \
            owl:someValuesFrom rdfs:Literal ] .",
    );
    let onto = extract(&dict, &triples).expect("the ALCH(D) fragment extracts this");
    assert_eq!(
        class_satisfiability(&class_a(&dict), &onto, Budget::default()),
        Verdict::Unsatisfiable,
        "xsd:integer and xsd:string have disjoint value spaces, so p has no values at all"
    );
    // The ontology ITSELF stays consistent — nothing forces an instance of A to exist.
    assert_eq!(
        consistency_from_extraction(&Ok(onto), Budget::default()),
        Verdict::Satisfiable
    );
}

/// The control for the test above: with only ONE range the value space is inhabited, so the
/// same class is satisfiable. Without this pair the unsat verdict could come from anywhere.
#[test]
fn a_single_datatype_range_leaves_the_class_satisfiable() {
    let (dict, triples) = parse(
        ":p a owl:DatatypeProperty ; rdfs:range xsd:integer .\n\
         :A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :p ; \
            owl:someValuesFrom rdfs:Literal ] .",
    );
    let onto = extract(&dict, &triples).expect("extracts");
    assert_eq!(
        class_satisfiability(&class_a(&dict), &onto, Budget::default()),
        Verdict::Satisfiable
    );
}

/// The ∀_D-rule against an incompatible ∃_D filler: `p`'s range is `xsd:byte` while `A`
/// demands a `xsd:string` value. The concrete successor's label becomes `{string, byte}`,
/// which the oracle reports empty.
#[test]
fn range_and_restriction_filler_from_different_families_clash() {
    let (dict, triples) = parse(
        ":p a owl:DatatypeProperty ; rdfs:range xsd:byte .\n\
         :A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :p ; \
            owl:someValuesFrom xsd:string ] .",
    );
    let onto = extract(&dict, &triples).expect("extracts");
    assert_eq!(
        class_satisfiability(&class_a(&dict), &onto, Budget::default()),
        Verdict::Unsatisfiable
    );
}

/// Overlapping integer ranges do NOT clash — the sub-lattice is an interval lattice, not a
/// tree. `xsd:long ⊓ xsd:nonNegativeInteger` is `[0, 2⁶³−1]`, which is inhabited; adding
/// `xsd:negativeInteger` empties it.
#[test]
fn integer_range_intersection_is_decided_by_intervals() {
    let overlapping = ":p a owl:DatatypeProperty ; rdfs:range xsd:long ; \
                       rdfs:range xsd:nonNegativeInteger .\n\
                       :A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :p ; \
                          owl:someValuesFrom rdfs:Literal ] .";
    let (dict, triples) = parse(overlapping);
    let onto = extract(&dict, &triples).expect("extracts");
    assert_eq!(
        class_satisfiability(&class_a(&dict), &onto, Budget::default()),
        Verdict::Satisfiable
    );

    let (dict, triples) = parse(&format!(
        "{}\n:p rdfs:range xsd:negativeInteger .",
        overlapping
    ));
    let onto = extract(&dict, &triples).expect("extracts");
    assert_eq!(
        class_satisfiability(&class_a(&dict), &onto, Budget::default()),
        Verdict::Unsatisfiable
    );
}

/// `rdfs:domain` on a data property internalises as `∃T.⊤_D ⊑ C` — the only place the
/// fragment ever produces a NEGATED data range (`∀T.¬rdfs:Literal`, i.e. `∀T.⊥_D`). Here
/// `A` demands a `p`-value but is disjoint from `p`'s domain, so `A` is unsatisfiable.
#[test]
fn data_property_domain_propagates_to_the_subject() {
    let (dict, triples) = parse(
        ":p a owl:DatatypeProperty ; rdfs:domain :C .\n\
         :A owl:disjointWith :C .\n\
         :A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :p ; \
            owl:someValuesFrom rdfs:Literal ] .",
    );
    let onto = extract(&dict, &triples).expect("extracts");
    assert_eq!(
        class_satisfiability(&class_a(&dict), &onto, Budget::default()),
        Verdict::Unsatisfiable
    );
}

/// The data-property hierarchy: a range on the SUPER-property constrains values reached
/// through the SUB-property, exactly as `∀`-propagation does for object properties.
#[test]
fn data_range_propagates_through_the_sub_property_hierarchy() {
    let (dict, triples) = parse(
        ":q a owl:DatatypeProperty ; rdfs:range xsd:string .\n\
         :p a owl:DatatypeProperty ; rdfs:subPropertyOf :q .\n\
         :A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :p ; \
            owl:someValuesFrom xsd:integer ] .",
    );
    let onto = extract(&dict, &triples).expect("extracts");
    assert_eq!(
        class_satisfiability(&class_a(&dict), &onto, Budget::default()),
        Verdict::Unsatisfiable,
        "the super-property's xsd:string range must reach the sub-property's value"
    );
}

/// A `∀T.dr` restriction alone imposes nothing (no value need exist), so the class stays
/// satisfiable — the ∀_D-rule must not manufacture a concrete successor.
#[test]
fn a_universal_data_restriction_alone_is_satisfiable() {
    let (dict, triples) = parse(
        ":p a owl:DatatypeProperty ; rdfs:range xsd:string .\n\
         :A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :p ; \
            owl:allValuesFrom xsd:integer ] .",
    );
    let onto = extract(&dict, &triples).expect("extracts");
    assert_eq!(
        class_satisfiability(&class_a(&dict), &onto, Budget::default()),
        Verdict::Satisfiable
    );
}

// -------------------------------------------------------------------------------------------
// (b) The boundary — everything not decided EXACTLY still fails closed
// -------------------------------------------------------------------------------------------

/// Datatypes the oracle deliberately declines (module docs §2) keep refusing extraction, so
/// the tableau never sees a value space it cannot decide.
#[test]
fn unadmitted_datatypes_still_refuse_extraction() {
    for datatype in [
        "xsd:double",
        "xsd:float",
        "xsd:token",
        "xsd:language",
        "xsd:hexBinary",
        "xsd:date",
        "xsd:dateTimeStamp",
        "rdf:PlainLiteral",
        "rdf:XMLLiteral",
    ] {
        let (dict, triples) = parse(&format!(
            ":p a owl:DatatypeProperty ; rdfs:range {} .",
            datatype
        ));
        assert!(
            matches!(
                extract(&dict, &triples),
                Err(ExtractError::DataConstruct(_))
            ),
            "{} must stay a fail-closed DataConstruct refusal",
            datatype
        );
    }
}

/// Facets, data-range complements, enumerations, data-property ASSERTIONS and literals in a
/// data-range position all remain refused — the sq-pbz04.4.9 boundary is narrowed, not
/// removed. Which fail-closed ARM carries the refusal depends on where the construct is
/// caught: a recognised out-of-fragment PREDICATE (`owl:oneOf`) is refused by the top-level
/// triple router before any data-range decoding, everything else by the data-range decoder.
#[test]
fn facets_complements_and_data_assertions_still_refuse_extraction() {
    let data_construct: &[(&str, &str)] = &[
        (
            "faceted range",
            ":p a owl:DatatypeProperty ; rdfs:range [ owl:onDatatype xsd:integer ; \
             owl:withRestrictions ( [ xsd:minInclusive 3 ] ) ] .",
        ),
        (
            "datatype complement",
            ":p a owl:DatatypeProperty ; rdfs:range [ owl:datatypeComplementOf xsd:integer ] .",
        ),
        (
            "data assertion",
            ":p a owl:DatatypeProperty ; rdfs:range xsd:string .\n:a :p \"x\" .",
        ),
        (
            "literal data range",
            ":p a owl:DatatypeProperty ; rdfs:range \"nonsense\" .",
        ),
        (
            "class IRI as a data range",
            ":p a owl:DatatypeProperty ; rdfs:range :SomeClass .",
        ),
        (
            "bare blank node as a data range",
            ":p a owl:DatatypeProperty ; rdfs:range [] .",
        ),
        (
            "data restriction with a class filler",
            ":p a owl:DatatypeProperty .\n\
             :A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :p ; \
                owl:someValuesFrom :C ] .",
        ),
        (
            "data restriction with an unadmitted datatype filler",
            ":p a owl:DatatypeProperty .\n\
             :A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :p ; \
                owl:allValuesFrom xsd:double ] .",
        ),
    ];
    for (label, body) in data_construct {
        let (dict, triples) = parse(body);
        let result = extract(&dict, &triples);
        assert!(
            matches!(result, Err(ExtractError::DataConstruct(_))),
            "{} must refuse fail-closed as DataConstruct, got {:?}",
            label,
            result
        );
    }
    // `owl:oneOf` is a recognised out-of-fragment LOGICAL predicate, so it is refused one
    // layer earlier — still fail-closed, just a different arm of the taxonomy.
    let (dict, triples) = parse(":p a owl:DatatypeProperty ; rdfs:range [ owl:oneOf ( 1 2 ) ] .");
    let result = extract(&dict, &triples);
    assert!(
        matches!(result, Err(ExtractError::OutOfFragment(_))),
        "an enumerated data range must refuse fail-closed, got {:?}",
        result
    );
}

/// A `rdfs:subPropertyOf` that MIXES a data property with a non-data property has no OWL 2
/// reading; it is refused rather than guessed in either direction.
#[test]
fn mixed_property_inclusion_is_unclassifiable() {
    let (dict, triples) = parse(":p a owl:DatatypeProperty ; rdfs:subPropertyOf :q .");
    assert!(matches!(
        extract(&dict, &triples),
        Err(ExtractError::Unclassifiable(_))
    ));
}

/// A bare datatype IRI in a genuine CLASS position (not a data range) keeps refusing — the
/// position check of sq-pbz04.4.9 is unchanged where no data property is involved.
#[test]
fn datatype_in_a_class_position_still_refuses() {
    let (dict, triples) = parse(":A rdfs:subClassOf xsd:string .");
    assert!(matches!(
        extract(&dict, &triples),
        Err(ExtractError::DataConstruct(_))
    ));
    // …including an rdfs:range whose subject is NOT a declared data property.
    let (dict, triples) = parse(":p rdfs:range xsd:integer .");
    assert!(matches!(
        extract(&dict, &triples),
        Err(ExtractError::DataConstruct(_))
    ));
}

/// Declaration ORDER must not change the verdict (RDF graphs are sets): the
/// `owl:DatatypeProperty` typing is honoured whether it precedes or follows its usage.
#[test]
fn extraction_is_declaration_order_independent() {
    let before = ":p a owl:DatatypeProperty .\n:p rdfs:range xsd:integer .";
    let after = ":p rdfs:range xsd:integer .\n:p a owl:DatatypeProperty .";
    let (d1, t1) = parse(before);
    let (d2, t2) = parse(after);
    let o1 = extract(&d1, &t1).expect("declaration first");
    let o2 = extract(&d2, &t2).expect("declaration last");
    assert_eq!(o1.len(), 1);
    assert_eq!(o1.len(), o2.len());
    assert!(matches!(
        o1.axioms()[0],
        Axiom::DataPropertyRange {
            range: DataRange::Datatype(Datatype::XsdInteger),
            ..
        }
    ));
}

// -------------------------------------------------------------------------------------------
// (c) Round-trip
// -------------------------------------------------------------------------------------------

/// `RDF → extract → render → extract` reproduces the concrete-domain model, so the forward
/// mapping and the fail-closed reverse mapping agree on the new constructs.
#[test]
fn concrete_domain_model_round_trips_through_rdf() {
    let (dict, triples) = parse(
        ":p a owl:DatatypeProperty ; rdfs:range xsd:integer ; rdfs:domain :C .\n\
         :q a owl:DatatypeProperty .\n\
         :p rdfs:subPropertyOf :q .\n\
         :A rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :p ; \
            owl:someValuesFrom xsd:byte ] .\n\
         :B rdfs:subClassOf [ a owl:Restriction ; owl:onProperty :q ; \
            owl:allValuesFrom xsd:decimal ] .",
    );
    let mut dict = dict;
    let onto = extract(&dict, &triples).expect("extracts");
    let rendered = render_to_triples(&onto, &mut dict);
    let round_tripped = extract(&dict, &rendered).expect("re-extracts");
    let mut a: Vec<String> = onto.axioms().iter().map(|x| format!("{:?}", x)).collect();
    let mut b: Vec<String> = round_tripped
        .axioms()
        .iter()
        .map(|x| format!("{:?}", x))
        .collect();
    a.sort();
    b.sort();
    assert_eq!(a, b, "the concrete-domain round-trip must be closed");
}

/// The structural model is usable directly (the tableau's real input type), without going
/// through RDF — the same unsat verdict from a hand-built ALCH(D) ontology.
#[test]
fn hand_built_structural_model_reaches_the_same_verdict() {
    let p = DPE::DataProperty(7);
    let onto = sparq_reason_dl::Ontology {
        axioms: vec![
            Axiom::DataPropertyRange {
                property: p.clone(),
                range: DataRange::Datatype(Datatype::XsdInteger),
            },
            Axiom::DataPropertyRange {
                property: p.clone(),
                range: DataRange::Datatype(Datatype::XsdString),
            },
        ],
    };
    let demanding = CE::DataSomeValuesFrom(p, DataRange::Datatype(Datatype::RdfsLiteral));
    assert_eq!(
        class_satisfiability(&demanding, &onto, Budget::default()),
        Verdict::Unsatisfiable
    );
}
