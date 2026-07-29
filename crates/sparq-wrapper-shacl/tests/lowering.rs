//! Acceptance test for the SHACL -> object-model IR lowering (sq-1rg2q.12).
//!
//! One Turtle `sh:NodeShape` exercises EVERY mapping the crate promises — the
//! three cardinality classes, `sh:datatype` scalars (including the SHACL-1.2
//! disjunctive list form), `sh:class` typed references (single and list),
//! `sh:node` nested types (named and anonymous), `sh:nodeKind sh:IRI`, an
//! unconstrained term field, and `sh:closed` — and the result is compared against
//! the exact IR, not a summary of it.
//!
//! [FABLE-5] (sq-1rg2q.12)
#![cfg(feature = "oo-models")]

use sparq_core::Graph;
use sparq_shacl::ShapesModel;
use sparq_wrapper_shacl::{
    lower, Cardinality, ClosedSchema, FieldSchema, ModelSchema, ReferenceSchema, ScalarKind,
    SchemaError, StructSchema, ValueSchema,
};

const EX: &str = "http://example.org/";
const XSD: &str = "http://www.w3.org/2001/XMLSchema#";
const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

/// The shapes graph the acceptance assertions are pinned to.
const SHAPES: &str = r#"
@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix ex:   <http://example.org/> .

ex:PersonShape a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:closed true ;
    sh:ignoredProperties ( rdf:type ) ;
    # minCount >= 1 + maxCount 1 -> required T
    sh:property [ sh:path ex:name ;     sh:datatype xsd:string  ; sh:minCount 1 ; sh:maxCount 1 ] ;
    # arbitrary-precision xsd:integer has no faithful std scalar -> lexical String
    sh:property [ sh:path ex:age ;      sh:datatype xsd:integer ; sh:minCount 1 ; sh:maxCount 1 ] ;
    # a BOUNDED integer type does fit i64 -> i64
    sh:property [ sh:path ex:height ;   sh:datatype xsd:int     ; sh:maxCount 1 ] ;
    # maxCount 1 + minCount 0/absent -> Option<T>
    sh:property [ sh:path ex:nickname ; sh:datatype xsd:string  ; sh:minCount 0 ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:score ;    sh:datatype xsd:double  ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:active ;   sh:datatype xsd:boolean ; sh:maxCount 1 ] ;
    # everything else -> Vec<T>, surviving bounds kept
    sh:property [ sh:path ex:email ;    sh:datatype xsd:string  ; sh:minCount 1 ] ;
    # an explicit `sh:minCount 0` is the absent default and must normalise away
    sh:property [ sh:path ex:alias ;    sh:datatype xsd:string  ; sh:minCount 0 ] ;
    sh:property [ sh:path ex:tag ;      sh:datatype xsd:string  ; sh:minCount 1 ; sh:maxCount 3 ] ;
    # SHACL-1.2 disjunctive datatype list -> one checked scalar over both
    sh:property [ sh:path ex:label ;    sh:datatype ( xsd:string rdf:langString ) ] ;
    # sh:class -> typed reference
    sh:property [ sh:path ex:knows ;    sh:class ex:Person ] ;
    sh:property [ sh:path ex:contact ;  sh:class ( ex:Person ex:Organization ) ] ;
    # sh:node -> nested type (named, then anonymous)
    sh:property [ sh:path ex:address ;  sh:node ex:AddressShape ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:geo ;      sh:maxCount 1 ;
                  sh:node [ sh:property [ sh:path ex:lat ; sh:datatype xsd:double ;
                                          sh:minCount 1 ; sh:maxCount 1 ] ] ] ;
    # sh:nodeKind sh:IRI alone -> a bare IRI
    sh:property [ sh:path ex:homepage ; sh:nodeKind sh:IRI ; sh:maxCount 1 ] ;
    # no value-shaping constraint -> any term
    sh:property [ sh:path ex:note ] .

ex:AddressShape a sh:NodeShape ;
    sh:property [ sh:path ex:city ; sh:datatype xsd:string ; sh:minCount 1 ; sh:maxCount 1 ] .
"#;

fn shapes_model(turtle: &str) -> ShapesModel {
    let graph = Graph::load_str(turtle, "turtle").expect("shapes graph parses");
    ShapesModel::parse(&graph)
}

fn ex(local: &str) -> String {
    format!("{EX}{local}")
}

fn xsd(local: &str) -> String {
    format!("{XSD}{local}")
}

fn scalar(kind: ScalarKind, datatypes: &[String]) -> ValueSchema {
    ValueSchema::Scalar {
        kind,
        datatypes: datatypes.to_vec(),
    }
}

fn field(name: &str, predicate: &str, cardinality: Cardinality, value: ValueSchema) -> FieldSchema {
    FieldSchema {
        name: name.to_string(),
        predicate: predicate.to_string(),
        cardinality,
        value,
    }
}

/// The IR the acceptance shapes graph must lower to, spelled out in full.
fn expected() -> ModelSchema {
    let string = vec![xsd("string")];
    ModelSchema {
        structs: vec![
            StructSchema {
                name: "Address".to_string(),
                shape: ex("AddressShape"),
                fields: vec![field(
                    "city",
                    &ex("city"),
                    Cardinality::Required,
                    scalar(ScalarKind::Lexical, &string),
                )],
                closed: None,
            },
            StructSchema {
                name: "Person".to_string(),
                shape: ex("PersonShape"),
                fields: vec![
                    field(
                        "active",
                        &ex("active"),
                        Cardinality::Optional,
                        scalar(ScalarKind::Bool, &[xsd("boolean")]),
                    ),
                    field(
                        "address",
                        &ex("address"),
                        Cardinality::Optional,
                        ValueSchema::Nested {
                            rust: "Address".to_string(),
                        },
                    ),
                    field(
                        "age",
                        &ex("age"),
                        Cardinality::Required,
                        // `xsd:integer` is arbitrary-precision: an `i64` would
                        // reject conforming values, so the lexical form is kept.
                        scalar(ScalarKind::Lexical, &[xsd("integer")]),
                    ),
                    field(
                        "alias",
                        &ex("alias"),
                        // `sh:minCount 0` normalises to "no lower bound", so the
                        // two spellings of the default produce ONE IR.
                        Cardinality::Many {
                            min: None,
                            max: None,
                        },
                        scalar(ScalarKind::Lexical, &string),
                    ),
                    field(
                        "contact",
                        &ex("contact"),
                        Cardinality::Many {
                            min: None,
                            max: None,
                        },
                        ValueSchema::Reference {
                            rust: "OrganizationPersonRef".to_string(),
                            classes: vec![ex("Organization"), ex("Person")],
                        },
                    ),
                    field(
                        "email",
                        &ex("email"),
                        Cardinality::Many {
                            min: Some(1),
                            max: None,
                        },
                        scalar(ScalarKind::Lexical, &string),
                    ),
                    field(
                        "geo",
                        &ex("geo"),
                        Cardinality::Optional,
                        ValueSchema::Nested {
                            rust: "PersonGeo".to_string(),
                        },
                    ),
                    field(
                        "height",
                        &ex("height"),
                        Cardinality::Optional,
                        // `xsd:int`'s whole value space fits in `i64`.
                        scalar(ScalarKind::I64, &[xsd("int")]),
                    ),
                    field(
                        "homepage",
                        &ex("homepage"),
                        Cardinality::Optional,
                        ValueSchema::Iri,
                    ),
                    field(
                        "knows",
                        &ex("knows"),
                        Cardinality::Many {
                            min: None,
                            max: None,
                        },
                        ValueSchema::Reference {
                            rust: "PersonRef".to_string(),
                            classes: vec![ex("Person")],
                        },
                    ),
                    field(
                        "label",
                        &ex("label"),
                        Cardinality::Many {
                            min: None,
                            max: None,
                        },
                        scalar(
                            ScalarKind::Lexical,
                            &[format!("{RDF}langString"), xsd("string")],
                        ),
                    ),
                    field(
                        "name",
                        &ex("name"),
                        Cardinality::Required,
                        scalar(ScalarKind::Lexical, &string),
                    ),
                    field(
                        "nickname",
                        &ex("nickname"),
                        Cardinality::Optional,
                        scalar(ScalarKind::Lexical, &string),
                    ),
                    field(
                        "note",
                        &ex("note"),
                        Cardinality::Many {
                            min: None,
                            max: None,
                        },
                        ValueSchema::Term,
                    ),
                    field(
                        "score",
                        &ex("score"),
                        Cardinality::Optional,
                        scalar(ScalarKind::F64, &[xsd("double")]),
                    ),
                    field(
                        "tag",
                        &ex("tag"),
                        Cardinality::Many {
                            min: Some(1),
                            max: Some(3),
                        },
                        scalar(ScalarKind::Lexical, &string),
                    ),
                ],
                closed: Some(ClosedSchema {
                    allowed: vec![
                        ex("active"),
                        ex("address"),
                        ex("age"),
                        ex("alias"),
                        ex("contact"),
                        ex("email"),
                        ex("geo"),
                        ex("height"),
                        ex("homepage"),
                        ex("knows"),
                        ex("label"),
                        ex("name"),
                        ex("nickname"),
                        ex("note"),
                        ex("score"),
                        ex("tag"),
                        format!("{RDF}type"),
                    ],
                }),
            },
            StructSchema {
                name: "PersonGeo".to_string(),
                // The anonymous `sh:node [ … ]`: recorded by its blank-node label,
                // which the parser assigns, so only the shape's SHAPE constant
                // varies — the type name is derived from owner + field.
                shape: String::new(),
                fields: vec![field(
                    "lat",
                    &ex("lat"),
                    Cardinality::Required,
                    scalar(ScalarKind::F64, &[xsd("double")]),
                )],
                closed: None,
            },
        ],
        references: vec![
            ReferenceSchema {
                name: "OrganizationPersonRef".to_string(),
                classes: vec![ex("Organization"), ex("Person")],
            },
            ReferenceSchema {
                name: "PersonRef".to_string(),
                classes: vec![ex("Person")],
            },
        ],
    }
}

#[test]
fn lowers_every_mapping_to_the_exact_ir() {
    let model = shapes_model(SHAPES);
    let mut actual = lower(&model).expect("the acceptance shapes graph lowers");

    // The anonymous nested shape is identified by a parser-assigned blank-node
    // label, the ONE part of the IR that is not a function of the shapes graph's
    // text. Assert its form, then normalise it so the rest compares exactly.
    let geo = actual
        .structs
        .iter_mut()
        .find(|s| s.name == "PersonGeo")
        .expect("the anonymous sh:node became a nested struct");
    assert!(
        geo.shape.starts_with("_:"),
        "anonymous nested shape should be labelled by its blank node, got {:?}",
        geo.shape
    );
    geo.shape = String::new();

    assert_eq!(actual, expected());
}

#[test]
fn lowering_is_deterministic() {
    // Two independent parses of the same text: `ShapesModel::shapes` is in
    // graph-traversal order, so this is the assertion that the IR does not
    // inherit it.
    let first = lower(&shapes_model(SHAPES)).expect("lowers");
    let second = lower(&shapes_model(SHAPES)).expect("lowers");
    let strip = |m: &ModelSchema| {
        let mut m = m.clone();
        for s in &mut m.structs {
            if s.shape.starts_with("_:") {
                s.shape = String::new();
            }
        }
        m
    };
    assert_eq!(strip(&first), strip(&second));
}

#[test]
fn min_count_above_max_count_is_a_typed_error() {
    let turtle = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/> .
ex:S a sh:NodeShape ;
    sh:property [ sh:path ex:p ; sh:datatype xsd:string ; sh:minCount 2 ; sh:maxCount 1 ] .
"#;
    assert_eq!(
        lower(&shapes_model(turtle)),
        Err(SchemaError::ContradictoryCardinality {
            shape: ex("S"),
            predicate: ex("p"),
            min: 2,
            max: 1,
        })
    );
}

#[test]
fn datatype_beside_class_is_a_typed_error() {
    let turtle = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/> .
ex:S a sh:NodeShape ;
    sh:property [ sh:path ex:p ; sh:datatype xsd:string ; sh:class ex:C ] .
"#;
    assert_eq!(
        lower(&shapes_model(turtle)),
        Err(SchemaError::ConflictingValueTypes {
            shape: ex("S"),
            predicate: ex("p"),
            // Ordered lexicographically, so the error does not depend on which
            // constraint the shapes-graph traversal reached first.
            first: format!("sh:class {{{}}}", ex("C")),
            second: format!("sh:datatype {{{}}}", xsd("string")),
        })
    );
}

#[test]
fn a_non_predicate_path_is_a_typed_error() {
    let turtle = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/> .
ex:S a sh:NodeShape ;
    sh:property [ sh:path ( ex:a ex:b ) ; sh:datatype xsd:string ] .
"#;
    assert_eq!(
        lower(&shapes_model(turtle)),
        Err(SchemaError::UnsupportedPath {
            shape: ex("S"),
            detail: format!("(<{}> / <{}>)", ex("a"), ex("b")),
        })
    );
}

#[test]
fn closed_by_types_has_no_static_whitelist() {
    let turtle = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/> .
ex:S a sh:NodeShape ;
    sh:closed sh:ByTypes ;
    sh:property [ sh:path ex:p ; sh:datatype xsd:string ] .
"#;
    assert_eq!(
        lower(&shapes_model(turtle)),
        Err(SchemaError::ClosedByTypes { shape: ex("S") })
    );
}

#[test]
fn an_ill_formed_shapes_graph_is_a_typed_error() {
    // A non-integer `sh:minCount` — validation merely SKIPS the construct, but
    // generating a type from a shape with a dropped constraint would be a lie.
    let turtle = r#"
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex: <http://example.org/> .
ex:S a sh:NodeShape ;
    sh:property [ sh:path ex:p ; sh:datatype xsd:string ; sh:minCount "many" ] .
"#;
    let err = lower(&shapes_model(turtle)).expect_err("an ill-formed shapes graph is rejected");
    assert!(
        matches!(err, SchemaError::IllFormedShapes { .. }),
        "expected IllFormedShapes, got {err:?}"
    );
}
