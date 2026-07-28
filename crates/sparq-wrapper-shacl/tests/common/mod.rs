//! [FABLE-5] (sq-1rg2q.12) The shared acceptance fixture: ONE Turtle node shape
//! exercising every mapping the lowering invariant names, so `lowering.rs`
//! (exact IR) and `emission.rs` (deterministic, compilable, closed-shape-
//! enforcing Rust) are asserting about the same shapes graph and cannot drift.
//!
//! Coverage, mapping by mapping:
//!
//! | property | mapping under test |
//! |---|---|
//! | `ex:name` | `sh:minCount 1` + `sh:maxCount 1` → required `T` |
//! | `ex:age` | `sh:maxCount 1` → `Option<T>` |
//! | `ex:nickname` | `sh:minCount 1`, no max → `Vec<T>` with a retained check |
//! | `ex:note` | no cardinality, no value type → `Vec<Value>` |
//! | `ex:active` / `ex:score` | `sh:datatype` → non-string checked scalars |
//! | `ex:visits` | unbounded `xsd:integer` → the lexical form, NOT `i64` |
//! | `ex:label` | SHACL 1.2 disjunctive `sh:datatype ( … )` |
//! | `ex:employer` | `sh:class` → a typed reference |
//! | `ex:address` | `sh:node` → a nested named type |
//! | `ex:manager` | `sh:node` back to the OWN shape → a boxed recursive type |
//! | `ex:homepage` | inline `sh:node [ … ]` + `sh:class` → derived-name nesting |
//! | `sh:closed` | the predicate whitelist, including `sh:ignoredProperties` |

#![allow(dead_code)] // a shared fixture: not every helper is used by every test

/// The acceptance shapes graph.
pub const SHAPES_TURTLE: &str = r#"
@prefix sh:  <http://www.w3.org/ns/shacl#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:  <http://example.org/> .

ex:PersonShape a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:closed true ;
    sh:ignoredProperties ( rdf:type ) ;
    sh:property [ sh:path ex:name     ; sh:datatype xsd:string  ; sh:minCount 1 ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:age      ; sh:datatype xsd:long    ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:visits   ; sh:datatype xsd:integer ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:active   ; sh:datatype xsd:boolean ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:score    ; sh:datatype xsd:double  ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:label    ; sh:datatype ( xsd:string rdf:langString ) ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:nickname ; sh:datatype xsd:string  ; sh:minCount 1 ] ;
    sh:property [ sh:path ex:note ] ;
    sh:property [ sh:path ex:employer ; sh:class ex:Organization ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:address  ; sh:node ex:AddressShape ; sh:minCount 1 ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:manager  ; sh:node ex:PersonShape ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:homepage ; sh:class ex:Document ; sh:maxCount 1 ;
                  sh:node [ sh:property [ sh:path ex:url ; sh:datatype xsd:string ;
                                          sh:minCount 1 ; sh:maxCount 1 ] ] ] .

ex:AddressShape a sh:NodeShape ;
    sh:property [ sh:path ex:city ; sh:datatype xsd:string ; sh:minCount 1 ; sh:maxCount 1 ] .
"#;

/// Parses `turtle` into a [`sparq_shacl::ShapesModel`].
///
/// # Panics
///
/// Panics if the fixture is not valid Turtle — a broken fixture is a broken
/// test, not a condition under test.
pub fn shapes_model(turtle: &str) -> sparq_shacl::ShapesModel {
    let graph = sparq_shacl::load_turtle_with_base(turtle, "http://example.org/")
        .expect("fixture must be valid Turtle");
    sparq_shacl::ShapesModel::parse(&graph)
}

/// Parses [`SHAPES_TURTLE`].
///
/// # Panics
///
/// See [`shapes_model`].
pub fn acceptance_model() -> sparq_shacl::ShapesModel {
    shapes_model(SHAPES_TURTLE)
}

/// Expands `ex:`/`xsd:`/`rdf:` shorthand to the full IRI the IR carries.
#[must_use]
pub fn iri(curie: &str) -> String {
    for (prefix, ns) in [
        ("ex:", "http://example.org/"),
        ("xsd:", "http://www.w3.org/2001/XMLSchema#"),
        ("rdf:", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
    ] {
        if let Some(local) = curie.strip_prefix(prefix) {
            return format!("{ns}{local}");
        }
    }
    curie.to_string()
}
