//! [FABLE-5] (sq-1rg2q.12) The lowering half of the acceptance test: the
//! fixture shapes graph lowers to an EXACT, fully-specified object model, and
//! every ill-formed or contradictory shape lowers to a typed error.

mod common;

use common::{acceptance_model, iri, shapes_model};
use sparq_wrapper_shacl::{
    lower, Cardinality, ClassMarker, Field, LowerError, ModelType, ObjectModel, RustScalar,
    ScalarType, ValueType,
};

fn scalar(rust: RustScalar, datatypes: &[&str]) -> ValueType {
    ValueType::Scalar(ScalarType {
        rust,
        datatypes: datatypes.iter().map(|d| iri(d)).collect(),
    })
}

fn field(name: &str, predicate: &str, value: ValueType, cardinality: Cardinality) -> Field {
    Field {
        name: name.to_string(),
        predicate: iri(predicate),
        value,
        cardinality,
    }
}

/// The whole acceptance assertion: the fixture's IR, spelled out. Written as a
/// single expected value rather than a pile of `assert!(contains)` probes so a
/// change anywhere in lowering — a field that silently disappears, a
/// cardinality that flips, a nested type that loses its class check — fails
/// here instead of passing an existence check.
#[test]
fn the_fixture_lowers_to_exactly_this_object_model() {
    let ir = lower(&acceptance_model()).expect("the fixture is well-formed");

    let expected = ObjectModel {
        types: vec![
            ModelType {
                name: "AddressShape".to_string(),
                shape: "<http://example.org/AddressShape>".to_string(),
                target_classes: vec![],
                closed: None,
                fields: vec![field(
                    "city",
                    "ex:city",
                    scalar(RustScalar::String, &["xsd:string"]),
                    Cardinality::Required,
                )],
            },
            ModelType {
                name: "PersonShape".to_string(),
                shape: "<http://example.org/PersonShape>".to_string(),
                target_classes: vec![iri("ex:Person")],
                // `sh:property` paths plus `sh:ignoredProperties`, sorted.
                closed: Some(
                    [
                        "ex:active",
                        "ex:address",
                        "ex:age",
                        "ex:employer",
                        "ex:homepage",
                        "ex:label",
                        "ex:manager",
                        "ex:name",
                        "ex:nickname",
                        "ex:note",
                        "ex:score",
                        "ex:visits",
                        "rdf:type",
                    ]
                    .iter()
                    .map(|c| iri(c))
                    .collect(),
                ),
                fields: vec![
                    field(
                        "active",
                        "ex:active",
                        scalar(RustScalar::Bool, &["xsd:boolean"]),
                        Cardinality::Optional,
                    ),
                    field(
                        "address",
                        "ex:address",
                        ValueType::Nested {
                            type_name: "AddressShape".to_string(),
                            class_marker: None,
                        },
                        Cardinality::Required,
                    ),
                    field(
                        "age",
                        "ex:age",
                        scalar(RustScalar::I64, &["xsd:long"]),
                        Cardinality::Optional,
                    ),
                    field(
                        "employer",
                        "ex:employer",
                        ValueType::Reference("OrganizationClass".to_string()),
                        Cardinality::Optional,
                    ),
                    // Inline `sh:node [ … ]`: the type name is derived from the
                    // parent type and the field, and the sibling `sh:class`
                    // rides along as a marker rather than replacing the nesting.
                    field(
                        "homepage",
                        "ex:homepage",
                        ValueType::Nested {
                            type_name: "PersonShapeHomepage".to_string(),
                            class_marker: Some("DocumentClass".to_string()),
                        },
                        Cardinality::Optional,
                    ),
                    // SHACL 1.2 disjunctive `sh:datatype`: no single Rust
                    // primitive, so the lexical form is kept and both datatypes
                    // stay checkable.
                    field(
                        "label",
                        "ex:label",
                        scalar(RustScalar::String, &["rdf:langString", "xsd:string"]),
                        Cardinality::Optional,
                    ),
                    field(
                        "manager",
                        "ex:manager",
                        ValueType::Nested {
                            type_name: "PersonShape".to_string(),
                            class_marker: None,
                        },
                        Cardinality::Optional,
                    ),
                    field(
                        "name",
                        "ex:name",
                        scalar(RustScalar::String, &["xsd:string"]),
                        Cardinality::Required,
                    ),
                    field(
                        "nickname",
                        "ex:nickname",
                        scalar(RustScalar::String, &["xsd:string"]),
                        Cardinality::Many {
                            min: Some(1),
                            max: None,
                        },
                    ),
                    field(
                        "note",
                        "ex:note",
                        ValueType::Term,
                        Cardinality::Many {
                            min: None,
                            max: None,
                        },
                    ),
                    field(
                        "score",
                        "ex:score",
                        scalar(RustScalar::F64, &["xsd:double"]),
                        Cardinality::Optional,
                    ),
                    // `xsd:integer`'s value space is unbounded, so no fixed-
                    // width primitive is lossless: the lexical form is kept.
                    field(
                        "visits",
                        "ex:visits",
                        scalar(RustScalar::String, &["xsd:integer"]),
                        Cardinality::Optional,
                    ),
                ],
            },
            ModelType {
                name: "PersonShapeHomepage".to_string(),
                // An inline shape is a blank node, and blank-node LABELS are
                // re-minted on every parse — so it is identified by where it
                // was referenced from, which is content-derived and stable.
                shape: "<http://example.org/PersonShape> sh:property \
                        <http://example.org/homepage> sh:node"
                    .to_string(),
                target_classes: vec![],
                closed: None,
                fields: vec![field(
                    "url",
                    "ex:url",
                    scalar(RustScalar::String, &["xsd:string"]),
                    Cardinality::Required,
                )],
            },
        ],
        class_markers: vec![
            ClassMarker {
                name: "DocumentClass".to_string(),
                classes: vec![iri("ex:Document")],
            },
            ClassMarker {
                name: "OrganizationClass".to_string(),
                classes: vec![iri("ex:Organization")],
            },
        ],
    };

    assert_eq!(ir, expected);
    // No volatile blank-node label may reach the IR, since it lands verbatim in
    // the emitted `SHAPE` constant.
    assert!(
        !format!("{ir:?}").contains("_:"),
        "a blank-node label leaked into the IR: {ir:?}"
    );
}

/// The IR is a function of the shapes graph's CONTENT. Re-parsing the same
/// Turtle re-hashes every term, so an IR that leaked traversal order would
/// differ here.
#[test]
fn lowering_is_independent_of_parse_order() {
    let first = lower(&acceptance_model()).expect("well-formed");
    let second = lower(&acceptance_model()).expect("well-formed");
    assert_eq!(first, second);
}

fn lower_err(turtle: &str) -> LowerError {
    lower(&shapes_model(turtle)).expect_err("this fixture must not lower")
}

const PREFIXES: &str = r#"
@prefix sh:  <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/> .
"#;

fn shape_with(body: &str) -> String {
    format!("{PREFIXES}\nex:S a sh:NodeShape ; {body} .")
}

#[test]
fn a_minimum_above_the_maximum_is_a_typed_error() {
    let err = lower_err(&shape_with(
        "sh:property [ sh:path ex:p ; sh:minCount 2 ; sh:maxCount 1 ]",
    ));
    assert!(
        matches!(
            err,
            LowerError::ContradictoryCardinality { min: 2, max: 1, .. }
        ),
        "{err}"
    );
}

#[test]
fn repeated_cardinality_constraints_are_conjunctive_before_they_contradict() {
    // min = max(1, 3) = 3, max = min(2, 5) = 2, so 3 > 2 contradicts even
    // though no single pair of statements does.
    let err = lower_err(&shape_with(
        "sh:property [ sh:path ex:p ; sh:minCount 1 ; sh:minCount 3 ; sh:maxCount 5 ; sh:maxCount 2 ]",
    ));
    assert!(
        matches!(
            err,
            LowerError::ContradictoryCardinality { min: 3, max: 2, .. }
        ),
        "{err}"
    );
}

#[test]
fn a_literal_valued_property_cannot_also_be_node_valued() {
    let err = lower_err(&shape_with(
        "sh:property [ sh:path ex:p ; sh:datatype xsd:string ; sh:class ex:C ]",
    ));
    assert!(
        matches!(
            err,
            LowerError::ConflictingValueType {
                first: "sh:datatype",
                second: "sh:class",
                ..
            }
        ),
        "{err}"
    );

    let err = lower_err(&shape_with(
        "sh:property [ sh:path ex:p ; sh:datatype xsd:string ; sh:node ex:Other ]",
    ));
    assert!(
        matches!(
            err,
            LowerError::ConflictingValueType {
                first: "sh:datatype",
                second: "sh:node",
                ..
            }
        ),
        "{err}"
    );
}

#[test]
fn a_non_predicate_path_has_no_faithful_field_rendering() {
    let err = lower_err(&shape_with(
        "sh:property [ sh:path ( ex:a ex:b ) ; sh:datatype xsd:string ]",
    ));
    assert!(
        matches!(
            err,
            LowerError::UnsupportedPath {
                form: "sequence",
                ..
            }
        ),
        "{err}"
    );

    let err = lower_err(&shape_with(
        "sh:property [ sh:path [ sh:inversePath ex:a ] ; sh:datatype xsd:string ]",
    ));
    assert!(
        matches!(err, LowerError::UnsupportedPath { form: "inverse", .. }),
        "{err}"
    );
}

/// `sh:closed sh:ByTypes` recomputes its allowed set per value node from that
/// node's `rdf:type`s, so there is no static whitelist to emit.
#[test]
fn close_by_types_has_no_static_whitelist() {
    let err = lower_err(&shape_with(
        "sh:closed sh:ByTypes ; sh:property [ sh:path ex:p ]",
    ));
    assert!(
        matches!(
            err,
            LowerError::UnsupportedComponent {
                component: "sh:closed sh:ByTypes",
                ..
            }
        ),
        "{err}"
    );
}

/// The SHACL parser's own syntax-rule findings are relayed rather than
/// re-derived: `"1"^^xsd:string` is integer-lexical but not an `xsd:integer`.
#[test]
fn an_ill_formed_shapes_graph_is_rejected_before_anything_is_lowered() {
    let err = lower_err(&shape_with(
        r#"sh:property [ sh:path ex:p ; sh:minCount "1"^^xsd:string ]"#,
    ));
    let LowerError::IllFormedShape { predicate, .. } = &err else {
        panic!("{err}");
    };
    assert_eq!(predicate, "http://www.w3.org/ns/shacl#minCount");
}

#[test]
fn two_predicates_may_not_collapse_into_one_field() {
    let err = lower_err(&shape_with(
        "sh:property [ sh:path ex:firstName ] ; sh:property [ sh:path ex:first-name ]",
    ));
    let LowerError::DuplicateFieldName { field, .. } = &err else {
        panic!("{err}");
    };
    assert_eq!(field, "first_name");
}

#[test]
fn two_shapes_may_not_claim_one_rust_name() {
    let turtle = format!(
        "{PREFIXES}\n@prefix other: <http://other.example/> .\n\
         ex:Thing a sh:NodeShape .\nother:Thing a sh:NodeShape ."
    );
    let err = lower_err(&turtle);
    let LowerError::DuplicateTypeName { name, .. } = &err else {
        panic!("{err}");
    };
    assert_eq!(name, "Thing");
}

#[test]
fn a_shape_may_not_shadow_the_emitted_prelude() {
    let err = lower_err(&format!("{PREFIXES}\nex:Triple a sh:NodeShape ."));
    assert!(
        matches!(err, LowerError::ReservedName { ref name, .. } if name == "Triple"),
        "{err}"
    );
}

/// `Self` is a reserved TYPE keyword, so `pub struct Self` does not parse. It
/// is the only keyword the upper-camel derivation can emit — every other Rust
/// keyword is lowercase and the first letter is always capitalised — so it must
/// be rejected at lowering rather than reaching `rustc` as a broken module.
#[test]
fn a_shape_named_self_may_not_become_the_type_keyword() {
    let err = lower_err(&format!("{PREFIXES}\nex:self a sh:NodeShape ."));
    assert!(
        matches!(err, LowerError::ReservedName { ref name, .. } if name == "Self"),
        "{err}"
    );
}

#[test]
fn an_iri_with_no_local_name_cannot_seed_an_identifier() {
    let err = lower_err(&format!("{PREFIXES}\n<http://example.org/ns#> a sh:NodeShape ."));
    assert!(matches!(err, LowerError::UnnameableIri { .. }), "{err}");
}

/// Keyword field names are escaped, not rejected, and non-structural components
/// neither add fields nor block lowering.
#[test]
fn keywords_are_escaped_and_non_structural_components_are_ignored() {
    let ir = lower(&shapes_model(&shape_with(
        r#"sh:property [ sh:path ex:type ; sh:datatype xsd:string ; sh:maxCount 1 ;
                         sh:pattern "^a" ; sh:minLength 1 ]"#,
    )))
    .expect("well-formed");
    assert_eq!(ir.types.len(), 1);
    assert_eq!(ir.types[0].fields.len(), 1);
    assert_eq!(ir.types[0].fields[0].name, "type_");
    assert_eq!(ir.types[0].fields[0].cardinality, Cardinality::Optional);
}
