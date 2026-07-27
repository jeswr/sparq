//! SHACL shapes → IR. [FABLE-5] (sq-1rg2q.12)
//!
//! The acceptance case parses one Turtle node shape that exercises EVERY
//! mapping the crate claims — `Option<T>` / required `T` / `Vec<T>`, `sh:class`
//! references, checked `sh:datatype` scalars, nested `sh:node` types and a
//! `sh:closed` predicate whitelist — through `sparq_shacl::ShapesModel`, and
//! pins the resulting IR exactly. Pinning the IR (rather than the emitted text)
//! is what makes a mapping change visible as a mapping change.
//!
//! The rest of the file is the negative half: every ill-formed or contradictory
//! shape must be a TYPED error, never a guess.

use sparq_core::Graph;
use sparq_shacl::ShapesModel;
use sparq_wrapper_shacl::{
    lower, Cardinality, ClosedShape, LoweringError, RustField, RustModel, RustReference, RustType,
    ScalarType, ValueType,
};

const PREFIXES: &str = r#"
    @prefix sh:   <http://www.w3.org/ns/shacl#> .
    @prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
    @prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
    @prefix ex:   <http://example.org/> .
"#;

fn model(shapes: &str) -> Result<RustModel, LoweringError> {
    let graph = Graph::load_str(&format!("{}{}", PREFIXES, shapes), "turtle")
        .expect("shapes graph must parse");
    lower(&ShapesModel::parse(&graph))
}

fn err(shapes: &str) -> LoweringError {
    model(shapes).expect_err("shapes graph should not have lowered")
}

/// One node shape touching every documented mapping. `tests/emission.rs`
/// carries the same fixture, and compiles what this file pins.
const EVERY_MAPPING: &str = r#"
    ex:PersonShape a sh:NodeShape ;
        sh:targetClass ex:Person ;
        sh:closed true ;
        sh:ignoredProperties ( rdf:type ) ;
        # sh:minCount >= 1 with sh:maxCount 1 -> required T
        sh:property [ sh:path ex:name ;      sh:datatype xsd:string ;
                      sh:minCount 1 ; sh:maxCount 1 ] ;
        # sh:maxCount 1 with no sh:minCount -> Option<T>
        sh:property [ sh:path ex:age ;       sh:datatype xsd:integer ; sh:maxCount 1 ] ;
        # sh:maxCount 1 with an explicit sh:minCount 0 -> Option<T>
        sh:property [ sh:path ex:height ;    sh:datatype xsd:double ;
                      sh:minCount 0 ; sh:maxCount 1 ] ;
        # remaining cardinalities -> Vec<T>, bounds checked
        sh:property [ sh:path ex:nickname ;  sh:datatype xsd:string ; sh:minCount 1 ] ;
        sh:property [ sh:path ex:score ;     sh:datatype xsd:decimal ;
                      sh:minCount 1 ; sh:maxCount 3 ] ;
        sh:property [ sh:path ex:active ;    sh:datatype xsd:boolean ; sh:maxCount 1 ] ;
        sh:property [ sh:path ex:homepage ;  sh:datatype xsd:anyURI ; sh:maxCount 1 ] ;
        sh:property [ sh:path ex:createdAt ; sh:datatype xsd:dateTime ;
                      sh:minCount 1 ; sh:maxCount 1 ] ;
        # sh:class -> a typed reference
        sh:property [ sh:path ex:employer ;  sh:class ex:Organization ;
                      sh:minCount 1 ; sh:maxCount 1 ] ;
        sh:property [ sh:path ex:knows ;     sh:class ex:Person ] ;
        # sh:node -> a nested type (named, and anonymous)
        sh:property [ sh:path ex:address ;   sh:node ex:AddressShape ;
                      sh:minCount 1 ; sh:maxCount 1 ] ;
        sh:property [ sh:path ex:pronouns ;  sh:node [ a sh:NodeShape ;
                          sh:property [ sh:path ex:subject ; sh:datatype xsd:string ;
                                        sh:minCount 1 ; sh:maxCount 1 ] ] ;
                      sh:maxCount 1 ] .

    ex:AddressShape a sh:NodeShape ;
        sh:property [ sh:path ex:city ; sh:datatype xsd:string ;
                      sh:minCount 1 ; sh:maxCount 1 ] .
"#;

fn scalar(
    name: &str,
    predicate: &str,
    ty: ScalarType,
    cardinality: Cardinality,
) -> RustField {
    RustField {
        name: name.to_string(),
        predicate: format!("http://example.org/{}", predicate),
        value: ValueType::Scalar(ty),
        cardinality,
    }
}

#[test]
fn lowers_every_mapping_to_the_exact_ir() {
    let model = model(EVERY_MAPPING).expect("the shapes graph is well-formed");

    assert_eq!(
        model,
        RustModel {
            types: vec![
                RustType {
                    name: "AddressShape".to_string(),
                    shape: Some("http://example.org/AddressShape".to_string()),
                    target_classes: vec![],
                    fields: vec![scalar(
                        "city",
                        "city",
                        ScalarType::String,
                        Cardinality::Required
                    )],
                    closed: None,
                },
                RustType {
                    name: "PersonShape".to_string(),
                    shape: Some("http://example.org/PersonShape".to_string()),
                    target_classes: vec!["http://example.org/Person".to_string()],
                    // Sorted by predicate IRI, so the IR does not depend on the
                    // order the shapes graph happened to be serialised in.
                    fields: vec![
                        scalar("active", "active", ScalarType::Boolean, Cardinality::Optional),
                        RustField {
                            name: "address".to_string(),
                            predicate: "http://example.org/address".to_string(),
                            value: ValueType::Nested("AddressShape".to_string()),
                            cardinality: Cardinality::Required,
                        },
                        scalar("age", "age", ScalarType::Integer, Cardinality::Optional),
                        scalar(
                            "created_at",
                            "createdAt",
                            ScalarType::DateTime,
                            Cardinality::Required
                        ),
                        RustField {
                            name: "employer".to_string(),
                            predicate: "http://example.org/employer".to_string(),
                            value: ValueType::Reference {
                                class: "http://example.org/Organization".to_string(),
                                name: "OrganizationRef".to_string(),
                            },
                            cardinality: Cardinality::Required,
                        },
                        scalar("height", "height", ScalarType::Double, Cardinality::Optional),
                        scalar(
                            "homepage",
                            "homepage",
                            ScalarType::AnyUri,
                            Cardinality::Optional
                        ),
                        RustField {
                            name: "knows".to_string(),
                            predicate: "http://example.org/knows".to_string(),
                            value: ValueType::Reference {
                                class: "http://example.org/Person".to_string(),
                                name: "PersonRef".to_string(),
                            },
                            cardinality: Cardinality::Many { min: 0, max: None },
                        },
                        scalar(
                            "name",
                            "name",
                            ScalarType::String,
                            Cardinality::Required
                        ),
                        scalar(
                            "nickname",
                            "nickname",
                            ScalarType::String,
                            Cardinality::Many {
                                min: 1,
                                max: None
                            }
                        ),
                        RustField {
                            name: "pronouns".to_string(),
                            predicate: "http://example.org/pronouns".to_string(),
                            value: ValueType::Nested("PersonShapePronouns".to_string()),
                            cardinality: Cardinality::Optional,
                        },
                        scalar(
                            "score",
                            "score",
                            ScalarType::Decimal,
                            Cardinality::Many {
                                min: 1,
                                max: Some(3)
                            }
                        ),
                    ],
                    closed: Some(ClosedShape {
                        allowed: vec![
                            "http://example.org/active".to_string(),
                            "http://example.org/address".to_string(),
                            "http://example.org/age".to_string(),
                            "http://example.org/createdAt".to_string(),
                            "http://example.org/employer".to_string(),
                            "http://example.org/height".to_string(),
                            "http://example.org/homepage".to_string(),
                            "http://example.org/knows".to_string(),
                            "http://example.org/name".to_string(),
                            "http://example.org/nickname".to_string(),
                            "http://example.org/pronouns".to_string(),
                            "http://example.org/score".to_string(),
                            // `sh:ignoredProperties`, sorted in with the rest.
                            "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
                        ],
                    }),
                },
                RustType {
                    // An anonymous `sh:node [ … ]` is named after the parent
                    // type and the field that reaches it — no blank-node id
                    // leaks into the model.
                    name: "PersonShapePronouns".to_string(),
                    shape: None,
                    target_classes: vec![],
                    fields: vec![scalar(
                        "subject",
                        "subject",
                        ScalarType::String,
                        Cardinality::Required
                    )],
                    closed: None,
                },
            ],
            references: vec![
                RustReference {
                    name: "OrganizationRef".to_string(),
                    class: "http://example.org/Organization".to_string(),
                },
                RustReference {
                    name: "PersonRef".to_string(),
                    class: "http://example.org/Person".to_string(),
                },
            ],
        }
    );
}

#[test]
fn lowering_is_deterministic_across_statement_order() {
    let forward = model(EVERY_MAPPING).expect("the shapes graph is well-formed");
    assert_eq!(forward, model(EVERY_MAPPING).unwrap(), "not idempotent");

    // Same triples, serialised in the opposite order. `ShapesModel` is built
    // from hash maps, so any iteration order the lowering inherited would show
    // up here.
    let reordered: String = {
        let mut statements: Vec<&str> = EVERY_MAPPING.split(" ;\n").collect();
        statements.reverse();
        statements.join(" ;\n")
    };
    // The reversal above is only meaningful if it actually changed the text.
    assert_ne!(reordered, EVERY_MAPPING);

    let shuffled = r#"
        ex:AddressShape a sh:NodeShape ;
            sh:property [ sh:path ex:city ; sh:datatype xsd:string ;
                          sh:minCount 1 ; sh:maxCount 1 ] .

        ex:ZShape a sh:NodeShape ;
            sh:property [ sh:path ex:b ; sh:node ex:AddressShape ; sh:maxCount 1 ] ;
            sh:property [ sh:path ex:a ; sh:class ex:Thing ] .

        ex:AShape a sh:NodeShape ;
            sh:property [ sh:path ex:c ; sh:datatype xsd:string ] .
    "#;
    let a = model(shuffled).unwrap();

    let same_triples_other_order = r#"
        ex:AShape a sh:NodeShape ;
            sh:property [ sh:path ex:c ; sh:datatype xsd:string ] .

        ex:ZShape a sh:NodeShape ;
            sh:property [ sh:path ex:a ; sh:class ex:Thing ] ;
            sh:property [ sh:path ex:b ; sh:node ex:AddressShape ; sh:maxCount 1 ] .

        ex:AddressShape a sh:NodeShape ;
            sh:property [ sh:path ex:city ; sh:datatype xsd:string ;
                          sh:minCount 1 ; sh:maxCount 1 ] .
    "#;
    assert_eq!(a, model(same_triples_other_order).unwrap());
    assert_eq!(
        a.types.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
        ["AShape", "AddressShape", "ZShape"],
        "types must be emitted in a content-determined order"
    );
}

#[test]
fn an_ill_formed_shapes_graph_is_a_typed_error() {
    // A non-integer sh:minCount violates the SHACL syntax rules; sparq-shacl
    // records it, and the generator refuses rather than defaulting the count.
    let e = err(
        r#"ex:S a sh:NodeShape ;
               sh:property [ sh:path ex:p ; sh:datatype xsd:string ; sh:minCount "many" ] ."#,
    );
    let LoweringError::IllFormedShapes { constructs } = &e else {
        panic!("expected IllFormedShapes, got {:?}", e);
    };
    assert!(
        constructs.iter().any(|c| c.contains("minCount")),
        "expected the offending construct to be named: {:?}",
        constructs
    );
}

#[test]
fn a_contradictory_cardinality_is_a_typed_error() {
    let e = err(
        r#"ex:S a sh:NodeShape ;
               sh:property [ sh:path ex:p ; sh:datatype xsd:string ;
                             sh:minCount 2 ; sh:maxCount 1 ] ."#,
    );
    assert_eq!(
        e,
        LoweringError::ContradictoryCardinality {
            shape: e_shape(&e),
            predicate: "http://example.org/p".to_string(),
            min: 2,
            max: 1,
        }
    );
    assert!(e.to_string().contains("exceeds"), "{}", e);
}

/// The blank-node label of the offending property shape, which the parser
/// chooses — read back out of the error so the assertion above can pin every
/// other field exactly.
fn e_shape(e: &LoweringError) -> String {
    match e {
        LoweringError::ContradictoryCardinality { shape, .. }
        | LoweringError::ConflictingCardinality { shape, .. }
        | LoweringError::ConflictingValueType { shape, .. }
        | LoweringError::MissingValueType { shape, .. }
        | LoweringError::UnsupportedDatatype { shape, .. }
        | LoweringError::AmbiguousValueType { shape, .. }
        | LoweringError::UnsupportedPath { shape, .. } => shape.clone(),
        other => panic!("no shape label on {:?}", other),
    }
}

#[test]
fn conflicting_and_missing_value_types_are_typed_errors() {
    let both = err(
        r#"ex:S a sh:NodeShape ;
               sh:property [ sh:path ex:p ; sh:datatype xsd:string ; sh:class ex:Thing ] ."#,
    );
    assert_eq!(
        both,
        LoweringError::ConflictingValueType {
            shape: e_shape(&both),
            predicate: "http://example.org/p".to_string(),
            // Reported alphabetically, not in encounter order.
            first: "sh:class".to_string(),
            second: "sh:datatype".to_string(),
        }
    );

    let neither = err(
        r#"ex:S a sh:NodeShape ;
               sh:property [ sh:path ex:p ; sh:minCount 1 ] ."#,
    );
    assert_eq!(
        neither,
        LoweringError::MissingValueType {
            shape: e_shape(&neither),
            predicate: "http://example.org/p".to_string(),
        }
    );
}

#[test]
fn an_unrepresentable_datatype_is_a_typed_error_not_a_string_fallback() {
    let e = err(
        r#"ex:S a sh:NodeShape ;
               sh:property [ sh:path ex:p ; sh:datatype xsd:gYearMonth ] ."#,
    );
    assert_eq!(
        e,
        LoweringError::UnsupportedDatatype {
            shape: e_shape(&e),
            predicate: "http://example.org/p".to_string(),
            datatype: "http://www.w3.org/2001/XMLSchema#gYearMonth".to_string(),
        }
    );
}

#[test]
fn a_disjunctive_value_type_is_a_typed_error() {
    let e = err(
        r#"ex:S a sh:NodeShape ;
               sh:property [ sh:path ex:p ; sh:datatype ( xsd:string xsd:integer ) ] ."#,
    );
    let LoweringError::AmbiguousValueType { keyword, values, .. } = &e else {
        panic!("expected AmbiguousValueType, got {:?}", e);
    };
    assert_eq!(keyword, "sh:datatype");
    assert_eq!(values.len(), 2);
}

#[test]
fn a_non_predicate_path_is_a_typed_error() {
    let e = err(
        r#"ex:S a sh:NodeShape ;
               sh:property [ sh:path [ sh:inversePath ex:p ] ; sh:datatype xsd:string ] ."#,
    );
    assert_eq!(
        e,
        LoweringError::UnsupportedPath {
            shape: e_shape(&e),
            detail: "an inverse path is not a single predicate IRI".to_string(),
        }
    );
}

#[test]
fn close_by_types_has_no_static_whitelist_and_is_a_typed_error() {
    let e = err(
        r#"ex:S a sh:NodeShape ;
               sh:closed sh:ByTypes ;
               sh:property [ sh:path ex:p ; sh:datatype xsd:string ] ."#,
    );
    assert_eq!(
        e,
        LoweringError::UnsupportedClosedMode {
            shape: "<http://example.org/S>".to_string(),
        }
    );
}

#[test]
fn colliding_derived_names_are_typed_errors() {
    let field = err(
        r#"ex:S a sh:NodeShape ;
               sh:property [ sh:path ex:givenName  ; sh:datatype xsd:string ] ;
               sh:property [ sh:path ex:given_name ; sh:datatype xsd:string ] ."#,
    );
    assert_eq!(
        field,
        LoweringError::DuplicateField {
            shape: "<http://example.org/S>".to_string(),
            field: "given_name".to_string(),
            first_predicate: "http://example.org/givenName".to_string(),
            second_predicate: "http://example.org/given_name".to_string(),
        }
    );

    let ty = err(
        r#"ex:Thing  a sh:NodeShape ; sh:property [ sh:path ex:p ; sh:datatype xsd:string ] .
           ex:thing  a sh:NodeShape ; sh:property [ sh:path ex:q ; sh:datatype xsd:string ] ."#,
    );
    assert_eq!(
        ty,
        LoweringError::DuplicateType {
            name: "Thing".to_string(),
            first_shape: "<http://example.org/Thing>".to_string(),
            second_shape: "<http://example.org/thing>".to_string(),
        }
    );
}

/// Two properties sharing ONE anonymous `sh:node` shape: the nested type is
/// named after whichever property reaches it first, so if the children were
/// lowered in the order the parser exposed them, reordering the two statements
/// would rename the generated type. Sorting the fields afterwards cannot undo a
/// name that is already assigned, so the ordering has to happen before lowering.
#[test]
fn a_shared_anonymous_node_shape_is_named_independently_of_statement_order() {
    const SHARED: &str = r#"
        _:shared a sh:NodeShape ;
            sh:property [ sh:path ex:city ; sh:datatype xsd:string ;
                          sh:minCount 1 ; sh:maxCount 1 ] .
    "#;
    let alpha_first = model(&format!(
        r#"ex:S a sh:NodeShape ;
               sh:property [ sh:path ex:alpha ; sh:node _:shared ; sh:maxCount 1 ] ;
               sh:property [ sh:path ex:zulu  ; sh:node _:shared ; sh:maxCount 1 ] .
           {}"#,
        SHARED
    ))
    .expect("well-formed");
    let zulu_first = model(&format!(
        r#"ex:S a sh:NodeShape ;
               sh:property [ sh:path ex:zulu  ; sh:node _:shared ; sh:maxCount 1 ] ;
               sh:property [ sh:path ex:alpha ; sh:node _:shared ; sh:maxCount 1 ] .
           {}"#,
        SHARED
    ))
    .expect("well-formed");

    assert_eq!(
        alpha_first, zulu_first,
        "the shared anonymous type was named after the encounter order"
    );
    // Named after the alphabetically first field that reaches it, whichever
    // statement came first in the graph.
    assert_eq!(
        alpha_first.types.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
        ["S", "SAlpha"]
    );
    assert_eq!(
        alpha_first.types[0]
            .fields
            .iter()
            .map(|f| f.value.clone())
            .collect::<Vec<_>>(),
        [
            ValueType::Nested("SAlpha".to_string()),
            ValueType::Nested("SAlpha".to_string()),
        ],
        "both properties must reach the same nested type"
    );
}

#[test]
fn constraints_that_do_not_shape_the_rust_type_are_left_to_the_validator() {
    // `sh:pattern` / `sh:minInclusive` restrict WHICH graphs load, not WHAT the
    // Rust type is, so they neither appear in the IR nor block lowering.
    let model = model(
        r#"ex:S a sh:NodeShape ;
               sh:property [ sh:path ex:p ; sh:datatype xsd:integer ;
                             sh:minInclusive 0 ; sh:pattern "^[0-9]+$" ;
                             sh:minCount 1 ; sh:maxCount 1 ] ."#,
    )
    .expect("unmodelled constraints must not block lowering");
    assert_eq!(model.types[0].fields.len(), 1);
    assert_eq!(
        model.types[0].fields[0].value,
        ValueType::Scalar(ScalarType::Integer)
    );
}

#[test]
fn property_shapes_and_logical_operands_do_not_become_types() {
    let model = model(
        r#"ex:S a sh:NodeShape ;
               sh:not [ sh:path ex:secret ; sh:minCount 1 ] ;
               sh:property [ sh:path ex:p ; sh:datatype xsd:string ] .
           ex:LooseProperty a sh:PropertyShape ;
               sh:path ex:q ; sh:datatype xsd:string ."#,
    )
    .expect("well-formed");
    assert_eq!(
        model.types.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
        ["S"],
        "only node shapes become structs"
    );
}
