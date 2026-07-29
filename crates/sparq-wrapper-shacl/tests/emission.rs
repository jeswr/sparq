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
    sh:property [ sh:path ex:weight ;   sh:datatype xsd:float   ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:ratio ;    sh:datatype xsd:decimal ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:quota ;    sh:datatype xsd:unsignedLong ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:visits ;   sh:datatype xsd:nonNegativeInteger ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:rank ;     sh:datatype xsd:byte    ; sh:maxCount 1 ] ;
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

/// An `xsd:integer` well outside `i64` — and outside `i128` too, so the
/// arbitrary-precision path is exercised past every fixed-width fallback.
const HUGE: &str = "170141183460469231731687303715884105728";
/// `xsd:unsignedLong`'s maximum, 2^64-1: conforming, and above `i64::MAX`.
const UNSIGNED_LONG_MAX: &str = "18446744073709551615";
/// More significant digits than an `f64` can hold.
const LONG_DECIMAL: &str = "3.14159265358979323846264338327950288419716939937510";

fn base() -> Vec<(Value, String, Value)> {
    vec![
        (iri("alice"), RDF_TYPE.to_string(), iri("Person")),
        (iri("alice"), p("name"), lit("Alice", "string")),
        (iri("alice"), p("age"), lit(HUGE, "integer")),
        (iri("alice"), p("active"), lit("true", "boolean")),
        (iri("alice"), p("score"), lit("1.5", "double")),
        (iri("alice"), p("weight"), lit("0.1", "float")),
        (iri("alice"), p("ratio"), lit(LONG_DECIMAL, "decimal")),
        (iri("alice"), p("quota"), lit(UNSIGNED_LONG_MAX, "unsignedLong")),
        (iri("alice"), p("visits"), lit("0", "nonNegativeInteger")),
        (iri("alice"), p("rank"), lit("-7", "byte")),
        (iri("alice"), p("email"), lit("alice@example.org", "string")),
        (iri("alice"), p("homepage"), iri("alice/home")),
        (iri("alice"), p("note"), lit("free text", "string")),
        (iri("alice"), p("knows"), iri("bob")),
        (iri("alice"), p("address"), iri("addr1")),
        (iri("bob"), RDF_TYPE.to_string(), iri("Person")),
        (iri("addr1"), p("city"), lit("Bristol", "string")),
    ]
}

/// `base()` with the single object of `pred` replaced.
fn with(pred: &str, object: Value) -> Vec<(Value, String, Value)> {
    let mut triples: Vec<_> = base()
        .into_iter()
        .filter(|(s, pr, _)| !(s == &iri("alice") && pr == &p(pred)))
        .collect();
    triples.push((iri("alice"), p(pred), object));
    triples
}

/// Loads with `pred` replaced, asserting the value CONFORMS.
fn loads(pred: &str, object: Value) -> Person {
    Person::load(&Triples(with(pred, object)), &iri("alice"))
        .unwrap_or_else(|e| panic!("conforming {} rejected: {:?}", pred, e))
}

/// Asserts the value is refused as outside its datatype's XSD value space.
fn outside_value_space(pred: &str, object: Value) {
    match Person::load(&Triples(with(pred, object)), &iri("alice")) {
        Err(LoadError::ValueSpace { predicate, .. }) => assert_eq!(predicate, p(pred)),
        other => panic!("{} accepted a non-conforming lexical: {:?}", pred, other),
    }
}

fn main() {
    // Happy path: every mapping round-trips into the generated struct.
    let person = Person::load(&Triples(base()), &iri("alice")).expect("a conforming focus loads");
    assert_eq!(person.name, "Alice");
    // Arbitrary-precision families keep their exact lexical form — an `i64`/`f64`
    // would have rejected or rounded every one of these conforming values.
    assert_eq!(person.age, HUGE);
    assert_eq!(person.quota.as_deref(), Some(UNSIGNED_LONG_MAX));
    assert_eq!(person.visits.as_deref(), Some("0"));
    assert_eq!(person.ratio.as_deref(), Some(LONG_DECIMAL));
    // A BOUNDED integer type still lowers to a real `i64`.
    assert_eq!(person.rank, Some(-7));
    assert_eq!(person.active, Some(true));
    assert_eq!(person.score, Some(1.5));
    // xsd:float is binary32, so its value is the f32 one widened — NOT the binary64
    // value the same lexical would have named as an xsd:double.
    assert_eq!(person.weight, Some(f64::from(0.1f32)));
    assert_ne!(person.weight, Some(0.1f64));
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

    // A derived integer type's VALUE space is enforced, not just the lexical
    // shape it shares with xsd:integer.
    outside_value_space("visits", lit("-1", "nonNegativeInteger"));
    outside_value_space("rank", lit("999", "byte"));
    outside_value_space("quota", lit(HUGE, "unsignedLong"));
    assert_eq!(loads("rank", lit("+007", "byte")).rank, Some(7));

    // xsd:decimal has no exponent and no special values, however happily an `f64`
    // parse would have swallowed them.
    outside_value_space("ratio", lit("1e5", "decimal"));
    outside_value_space("ratio", lit("NaN", "decimal"));
    assert_eq!(loads("ratio", lit("-.5", "decimal")).ratio.as_deref(), Some("-.5"));

    // xsd:double's specials are EXACTLY `INF`/`-INF`/`NaN` — the `inf`/`infinity`/
    // `nan` spellings Rust's own float parser accepts are not in the value space.
    assert_eq!(loads("score", lit("INF", "double")).score, Some(f64::INFINITY));
    assert_eq!(loads("score", lit("-INF", "double")).score, Some(f64::NEG_INFINITY));
    assert!(loads("score", lit("NaN", "double")).score.unwrap().is_nan());
    outside_value_space("score", lit("inf", "double"));
    outside_value_space("score", lit("infinity", "double"));
    outside_value_space("score", lit("nan", "double"));
}
"#;

/// A SELF-REFERENTIAL node shape: `ex:next`/`ex:alt` both nest `ex:LinkShape`
/// itself, which the IR boxes. Cyclic data under it is what the guard is for.
/// `ex:child` is unbounded, so the `Vec<Nested>` descent — the other emitted
/// nesting path — is compiled and guarded here too, not only the boxed one.
const CYCLIC_SHAPES: &str = r#"
@prefix sh:   <http://www.w3.org/ns/shacl#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
@prefix ex:   <http://example.org/> .

ex:LinkShape a sh:NodeShape ;
    sh:property [ sh:path ex:label ; sh:datatype xsd:string ; sh:minCount 1 ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:next ;  sh:node ex:LinkShape ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:alt ;   sh:node ex:LinkShape ; sh:maxCount 1 ] ;
    sh:property [ sh:path ex:child ; sh:node ex:LinkShape ] .
"#;

/// Drives the recursive model: a cyclic graph must be a typed error, an acyclic
/// one must still load, and a node reached twice on DISJOINT branches must not be
/// mistaken for a cycle.
const CYCLIC_HARNESS: &str = r#"
#[allow(dead_code)]
mod model;

use model::{Link, LoadError, Source, Value};

const EX: &str = "http://example.org/";
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

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

fn p(local: &str) -> String {
    format!("{}{}", EX, local)
}

fn label(node: &str) -> (Value, String, Value) {
    (
        iri(node),
        p("label"),
        Value::Literal {
            lexical: node.to_string(),
            datatype: XSD_STRING.to_string(),
            language: None,
        },
    )
}

/// Asserts the graph is refused as cyclic, at `at`.
fn cyclic(triples: Vec<(Value, String, Value)>, at: &str) {
    match Link::load(&Triples(triples), &iri("a")) {
        Err(LoadError::Cycle { shape, node }) => {
            assert_eq!(shape, Link::SHAPE);
            assert_eq!(node, format!("<{}{}>", EX, at));
        }
        other => panic!("cyclic sh:node data was not refused: {:?}", other),
    }
}

fn main() {
    // THE obligation: a self-loop under a self-referential shape is a typed error,
    // not recursion until the stack is exhausted.
    cyclic(vec![label("a"), (iri("a"), p("next"), iri("a"))], "a");

    // And a longer cycle, a -> b -> a, reported where it closes.
    cyclic(
        vec![
            label("a"),
            (iri("a"), p("next"), iri("b")),
            label("b"),
            (iri("b"), p("next"), iri("a")),
        ],
        "a",
    );

    // The `Vec<Nested>` descent is guarded on the same terms as the boxed one.
    cyclic(
        vec![
            label("a"),
            (iri("a"), p("child"), iri("b")),
            label("b"),
            (iri("b"), p("child"), iri("a")),
        ],
        "a",
    );

    // An ACYCLIC chain of the very same shape still loads in full.
    let chain = Triples(vec![
        label("a"),
        (iri("a"), p("next"), iri("b")),
        label("b"),
        (iri("b"), p("next"), iri("c")),
        label("c"),
    ]);
    let link = Link::load(&chain, &iri("a")).expect("an acyclic chain loads");
    assert_eq!(link.label, "a");
    let second = link.next.as_ref().expect("a -> b");
    assert_eq!(second.label, "b");
    assert_eq!(second.next.as_ref().map(|l| l.label.as_str()), Some("c"));

    // One node reached twice on DISJOINT branches is a tree, not a cycle: the
    // guard tracks loads IN PROGRESS, not every node ever seen.
    let shared = Triples(vec![
        label("a"),
        (iri("a"), p("next"), iri("b")),
        (iri("a"), p("alt"), iri("b")),
        label("b"),
    ]);
    let link = Link::load(&shared, &iri("a")).expect("a shared child is not a cycle");
    assert_eq!(link.next.as_ref().map(|l| l.label.as_str()), Some("b"));
    assert_eq!(link.alt.as_ref().map(|l| l.label.as_str()), Some("b"));
    assert!(link.child.is_empty());
}
"#;

fn generated_source() -> String {
    source_for(SHAPES)
}

fn source_for(shapes: &str) -> String {
    let graph = Graph::load_str(shapes, "turtle").expect("shapes graph parses");
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

/// The closed-shape/scalar/reference model: compiled and run for real.
#[test]
fn generated_model_compiles_and_enforces_its_shape() {
    compile_and_run("sq-1rg2q-12-emission", &generated_source(), HARNESS);
}

/// A self-referential `sh:node` shape over cyclic data must be a DETERMINISTIC
/// typed error. Before the active-`(shape, node)` guard this recursed until the
/// harness process died of stack exhaustion — so this test fails loudly, at the
/// run step, if the guard is removed.
#[test]
fn generated_loader_refuses_cyclic_nesting() {
    compile_and_run(
        "sq-1rg2q-12-cyclic",
        &source_for(CYCLIC_SHAPES),
        CYCLIC_HARNESS,
    );
}

/// Writes `model` plus `harness` into a scratch dir named `slug`, compiles them
/// with `rustc -D warnings`, and runs the result.
fn compile_and_run(slug: &str, model: &str, harness: &str) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(slug);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");

    let model_rs = dir.join("model.rs");
    let main_rs = dir.join("main.rs");
    std::fs::write(&model_rs, model).expect("write model.rs");
    std::fs::write(&main_rs, harness).expect("write main.rs");

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
