//! Acceptance test for Rust emission (sq-1rg2q.12).
//!
//! Emission is asserted end to end and for real: the same shapes graph is lowered
//! and emitted TWICE and the two sources must be byte-identical; the source is
//! then written out, **compiled** by `rustc` under `-D warnings` together with a
//! harness that drives the generated loaders, and the harness is **run** — so the
//! test proves the generated closed-shape loader actually rejects a predicate the
//! shape does not allow, rather than proving only that some text was produced.
//!
//! [FABLE-5] (sq-1rg2q.12)
#![cfg(feature = "oo-models")]

use std::path::{Path, PathBuf};
use std::process::Command;

use sparq_core::Graph;
use sparq_shacl::ShapesModel;
use sparq_wrapper_shacl::{emit, lower};

/// A closed node shape with one of each value kind, so the compiled harness
/// exercises scalars, a typed reference, a nested type and the whitelist.
const SHAPES: &str = r#"
@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix ex:   <http://example.org/> .

ex:PersonShape a sh:NodeShape ;
    sh:targetClass ex:Person ;
    sh:closed true ;
    sh:ignoredProperties ( rdf:type ) ;
    sh:property [ sh:path ex:name ;     sh:datatype xsd:string  ; sh:minCount 1 ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:age ;      sh:datatype xsd:integer ; sh:minCount 1 ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:nickname ; sh:datatype xsd:string  ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:active ;   sh:datatype xsd:boolean ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:score ;    sh:datatype xsd:double  ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:email ;    sh:datatype xsd:string  ; sh:minCount 1 ] ;
    sh:property [ sh:path ex:homepage ; sh:nodeKind sh:IRI ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:note ] ;
    sh:property [ sh:path ex:knows ;    sh:class ex:Person ] ;
    sh:property [ sh:path ex:address ;  sh:node ex:AddressShape ; sh:maxCount 1 ] .

ex:AddressShape a sh:NodeShape ;
    sh:property [ sh:path ex:city ; sh:datatype xsd:string ; sh:minCount 1 ; sh:maxCount 1 ] .
"#;

/// Drives the generated loaders. Kept out of `SHAPES`' crate so the generated
/// file is loaded exactly as a consumer would load it (`mod`, no edits).
const HARNESS: &str = r#"
#[allow(dead_code)]
mod model;

use model::{LoadError, Person, Source, Value};

const EX: &str = "http://example.org/";
const XSD: &str = "http://www.w3.org/2001/XMLSchema#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

struct Triples(Vec<(Value, String, Value)>);

impl Source for Triples {
    fn values(&self, subject: &Value, predicate: &str) -> Vec<Value> {
        self.0
            .iter()
            .filter(|(s, p, _)| s == subject && p == predicate)
            .map(|(_, _, o)| o.clone())
            .collect()
    }

    fn predicates(&self, subject: &Value) -> Vec<String> {
        self.0
            .iter()
            .filter(|(s, _, _)| s == subject)
            .map(|(_, p, _)| p.clone())
            .collect()
    }
}

fn iri(local: &str) -> Value {
    Value::Iri(format!("{}{}", EX, local))
}

fn lit(lexical: &str, datatype: &str) -> Value {
    Value::Literal {
        lexical: lexical.to_string(),
        datatype: format!("{}{}", XSD, datatype),
        language: None,
    }
}

fn p(local: &str) -> String {
    format!("{}{}", EX, local)
}

fn base() -> Vec<(Value, String, Value)> {
    vec![
        (iri("alice"), RDF_TYPE.to_string(), iri("Person")),
        (iri("alice"), p("name"), lit("Alice", "string")),
        (iri("alice"), p("age"), lit("42", "integer")),
        (iri("alice"), p("active"), lit("true", "boolean")),
        (iri("alice"), p("score"), lit("1.5", "double")),
        (iri("alice"), p("email"), lit("alice@example.org", "string")),
        (iri("alice"), p("homepage"), iri("alice/home")),
        (iri("alice"), p("note"), lit("free text", "string")),
        (iri("alice"), p("knows"), iri("bob")),
        (iri("alice"), p("address"), iri("addr1")),
        (iri("bob"), RDF_TYPE.to_string(), iri("Person")),
        (iri("addr1"), p("city"), lit("Bristol", "string")),
    ]
}

fn main() {
    // Happy path: every mapping round-trips into the generated struct.
    let person = Person::load(&Triples(base()), &iri("alice")).expect("a conforming focus loads");
    assert_eq!(person.name, "Alice");
    assert_eq!(person.age, 42);
    assert_eq!(person.active, Some(true));
    assert_eq!(person.score, Some(1.5));
    assert_eq!(person.nickname, None);
    assert_eq!(person.email, vec!["alice@example.org".to_string()]);
    assert_eq!(person.homepage.as_deref(), Some("http://example.org/alice/home"));
    assert_eq!(person.note.len(), 1);
    assert_eq!(person.knows.len(), 1);
    assert_eq!(person.knows[0].iri(), Some("http://example.org/bob"));
    assert_eq!(person.address.as_ref().map(|a| a.city.as_str()), Some("Bristol"));

    // THE closed-shape obligation: one extra predicate and the loader refuses.
    let mut extra = base();
    extra.push((iri("alice"), p("undeclared"), lit("boom", "string")));
    match Person::load(&Triples(extra), &iri("alice")) {
        Err(LoadError::ClosedPredicate { shape, predicate }) => {
            assert_eq!(shape, Person::SHAPE);
            assert_eq!(predicate, p("undeclared"));
        }
        other => panic!("closed shape accepted an extra predicate: {:?}", other),
    }

    // sh:minCount 1 on a required field.
    let missing: Vec<_> = base()
        .into_iter()
        .filter(|(_, pred, _)| pred != &p("name"))
        .collect();
    match Person::load(&Triples(missing), &iri("alice")) {
        Err(LoadError::Cardinality { predicate, got, .. }) => {
            assert_eq!(predicate, p("name"));
            assert_eq!(got, 0);
        }
        other => panic!("missing required value accepted: {:?}", other),
    }

    // sh:maxCount 1 on an optional field.
    let mut twice = base();
    twice.push((iri("alice"), p("nickname"), lit("Al", "string")));
    twice.push((iri("alice"), p("nickname"), lit("Ali", "string")));
    match Person::load(&Triples(twice), &iri("alice")) {
        Err(LoadError::Cardinality { predicate, got, .. }) => {
            assert_eq!(predicate, p("nickname"));
            assert_eq!(got, 2);
        }
        other => panic!("sh:maxCount 1 violated without error: {:?}", other),
    }

    // sh:datatype is checked, not assumed.
    let wrong: Vec<_> = base()
        .into_iter()
        .map(|(s, pred, o)| {
            if pred == p("age") {
                (s, pred, lit("42", "string"))
            } else {
                (s, pred, o)
            }
        })
        .collect();
    match Person::load(&Triples(wrong), &iri("alice")) {
        Err(LoadError::Datatype { predicate, got, .. }) => {
            assert_eq!(predicate, p("age"));
            assert_eq!(got, format!("{}string", XSD));
        }
        other => panic!("wrong datatype accepted: {:?}", other),
    }

    // sh:class is checked through rdf:type.
    let untyped: Vec<_> = base()
        .into_iter()
        .filter(|(s, pred, _)| !(s == &iri("bob") && pred == RDF_TYPE))
        .collect();
    match Person::load(&Triples(untyped), &iri("alice")) {
        Err(LoadError::Class { predicate, .. }) => assert_eq!(predicate, p("knows")),
        other => panic!("sh:class not enforced: {:?}", other),
    }
}
"#;

fn generated_source() -> String {
    let graph = Graph::load_str(SHAPES, "turtle").expect("shapes graph parses");
    let model = ShapesModel::parse(&graph);
    let schema = lower(&model).expect("the shapes graph lowers");
    emit(&schema)
}

#[test]
fn emission_is_byte_identical_across_runs() {
    let first = generated_source();
    let second = generated_source();
    assert_eq!(first, second, "emission must be deterministic");

    // And twice off ONE lowering, so neither stage is the source of the stability.
    let graph = Graph::load_str(SHAPES, "turtle").expect("shapes graph parses");
    let schema = lower(&ShapesModel::parse(&graph)).expect("lowers");
    assert_eq!(emit(&schema), emit(&schema));
}

/// Writes the generated model plus the harness, compiles both with `rustc -D
/// warnings`, and runs the result.
#[test]
fn generated_model_compiles_and_enforces_its_shape() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("sq-1rg2q-12-emission");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");

    let model_rs = dir.join("model.rs");
    let main_rs = dir.join("main.rs");
    std::fs::write(&model_rs, generated_source()).expect("write model.rs");
    std::fs::write(&main_rs, HARNESS).expect("write main.rs");

    let exe = dir.join("harness");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let compile = Command::new(&rustc)
        .args(["--edition", "2021", "-D", "warnings", "-o"])
        .arg(&exe)
        .arg(&main_rs)
        .output()
        .expect("failed to spawn rustc");
    assert!(
        compile.status.success(),
        "generated model did not compile cleanly:\n{}\n--- {} ---\n{}",
        String::from_utf8_lossy(&compile.stderr),
        model_rs.display(),
        numbered(&model_rs),
    );

    let run = Command::new(&exe).output().expect("failed to run the harness");
    assert!(
        run.status.success(),
        "the generated loaders did not behave as the shape requires:\n{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    );
}

/// Line-numbered generated source, so a compile failure names a line the reader
/// can find.
fn numbered(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .enumerate()
        .map(|(i, line)| format!("{:>4} | {line}\n", i + 1))
        .collect()
}
