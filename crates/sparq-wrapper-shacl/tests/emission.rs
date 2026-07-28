//! [FABLE-5] (sq-1rg2q.12) The emission half of the acceptance test: the
//! fixture emits byte-identical Rust twice, that Rust really compiles, and the
//! generated closed-shape loader really rejects an extra predicate.
//!
//! The compile-and-run step is the point. A generator can emit plausible-looking
//! source that does not parse, does not typecheck, or typechecks but never
//! enforces anything; only handing the bytes to `rustc` and running the result
//! rules all three out. The harness below is compiled with `-D warnings`, so
//! the generated module must also be warning-clean, not merely valid.

mod common;

use std::path::PathBuf;
use std::process::Command;

use common::acceptance_model;
use sparq_wrapper_shacl::{emit, generate, lower};

/// Emission is a pure function of the IR.
#[test]
fn emitting_the_same_model_twice_is_byte_identical() {
    let ir = lower(&acceptance_model()).expect("well-formed");
    assert_eq!(emit(&ir), emit(&ir));
}

/// The stronger claim: re-parsing the Turtle re-mints every blank-node label and
/// re-hashes every term, and the emitted bytes still match. This is what would
/// break if any traversal-order- or label-dependent value reached the output.
#[test]
fn re_parsing_the_shapes_graph_emits_identical_rust() {
    let first = generate(&acceptance_model()).expect("well-formed");
    let second = generate(&acceptance_model()).expect("well-formed");
    assert_eq!(first, second);
    assert!(!first.contains("_:"), "a blank-node label reached the output");
}

/// Everything the fixture's mappings should produce, in the emitted source.
#[test]
fn the_emitted_source_renders_every_mapping() {
    let src = generate(&acceptance_model()).expect("well-formed");
    for expected in [
        "pub struct PersonShape",
        "pub struct AddressShape",
        "pub struct PersonShapeHomepage",
        "pub name: String",                        // minCount 1 + maxCount 1
        "pub age: Option<i64>",                    // maxCount 1
        "pub visits: Option<String>",              // unbounded xsd:integer
        "pub active: Option<bool>",                //
        "pub score: Option<f64>",                  //
        "pub label: Option<String>",               // disjunctive sh:datatype
        "pub nickname: Vec<String>",               // minCount 1, no max
        "pub note: Vec<Value>",                    // no value constraint
        "pub employer: Option<Ref<OrganizationClass>>", // sh:class
        "pub address: Box<AddressShape>",          // sh:node, required
        "pub manager: Option<Box<PersonShape>>",   // recursive sh:node
        "pub homepage: Option<Box<PersonShapeHomepage>>",
        "pub const ALLOWED_PREDICATES",            // sh:closed
        "pub struct OrganizationClass;",
        "pub struct DocumentClass;",
    ] {
        assert!(src.contains(expected), "missing from emitted source: {expected}");
    }
}

/// A shape with no `sh:closed` gets no whitelist and no closed check — the
/// negative half of the closed-shape mapping.
#[test]
fn an_open_shape_emits_no_whitelist() {
    let src = generate(&common::shapes_model(
        "@prefix sh: <http://www.w3.org/ns/shacl#> .\n\
         @prefix ex: <http://example.org/> .\n\
         ex:S a sh:NodeShape ; sh:property [ sh:path ex:p ] .",
    ))
    .expect("well-formed");
    assert!(!src.contains("ALLOWED_PREDICATES"));
    assert!(!src.contains("check_closed(Self::SHAPE"));
}

/// Compiles the emitted module together with a harness that exercises it, runs
/// the binary, and asserts it succeeded.
#[test]
fn the_generated_code_compiles_and_enforces_its_closed_shape() {
    let src = generate(&acceptance_model()).expect("well-formed");

    let dir = scratch_dir("compile");
    std::fs::create_dir_all(&dir).expect("scratch dir");
    std::fs::write(dir.join("model.rs"), &src).expect("write generated module");
    std::fs::write(dir.join("main.rs"), HARNESS).expect("write harness");

    let binary = dir.join("harness");
    let compile = Command::new(rustc())
        .args(["--edition", "2021", "--crate-type", "bin", "-D", "warnings"])
        .arg("-o")
        .arg(&binary)
        .arg(dir.join("main.rs"))
        .output()
        .expect("rustc must be runnable: cargo just used it to build this test");
    assert!(
        compile.status.success(),
        "the generated module did not compile:\n{}\n--- generated ---\n{src}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(&binary).output().expect("run the harness");
    assert!(
        run.status.success(),
        "the generated loader behaved wrongly:\n{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = std::fs::remove_dir_all(&dir);
}

fn rustc() -> String {
    // Cargo sets RUSTC for build scripts and tests; fall back to PATH.
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

fn scratch_dir(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "sparq-wrapper-shacl-{tag}-{}",
        std::process::id()
    ))
}

/// The harness compiled against the generated module. Every assertion is about
/// BEHAVIOUR of the generated loader, so a generator that emits compilable but
/// inert code still fails here.
const HARNESS: &str = r#"
mod model;

use model::{LoadError, ObjectKind, PersonShape, Triple, Value};

const TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const ALICE: &str = "http://example.org/alice";
const ADDR: &str = "http://example.org/addr1";
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const XSD_LONG: &str = "http://www.w3.org/2001/XMLSchema#long";
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";

// One past i64::MAX: a perfectly valid xsd:integer lexical form.
const BEYOND_I64: &str = "9223372036854775808";

fn base() -> Vec<Triple<'static>> {
    vec![
        Triple::node(ALICE, TYPE, "http://example.org/Person"),
        Triple::literal(ALICE, "http://example.org/name", "Alice", XSD_STRING),
        Triple::literal(ALICE, "http://example.org/nickname", "Al", XSD_STRING),
        Triple::literal(ALICE, "http://example.org/age", "41", XSD_LONG),
        Triple::node(ALICE, "http://example.org/address", ADDR),
        Triple::literal(ADDR, "http://example.org/city", "Bristol", XSD_STRING),
    ]
}

fn main() {
    // 1. A conforming subject loads, with every cardinality mapped as declared.
    let triples = base();
    let alice = PersonShape::load(ALICE, &triples).expect("alice conforms");
    assert_eq!(alice.name, "Alice");                       // required T
    assert_eq!(alice.age, Some(41));                       // Option<T>
    assert_eq!(alice.nickname, vec!["Al".to_string()]);    // Vec<T>
    assert_eq!(alice.address.city, "Bristol");             // nested type
    assert!(alice.manager.is_none());
    assert!(alice.visits.is_none());
    assert!(alice.note.is_empty());
    assert_eq!(PersonShape::TARGET_CLASSES, &["http://example.org/Person"]);

    // 2. THE CLOSED-SHAPE CLAIM: one extra predicate, otherwise identical, is
    //    rejected — and rejected by NAME, not by some unrelated failure.
    let mut extra = base();
    extra.push(Triple::literal(ALICE, "http://example.org/hobby", "chess", XSD_STRING));
    match PersonShape::load(ALICE, &extra) {
        Err(LoadError::ClosedShapeViolation { predicate, subject, .. }) => {
            assert_eq!(predicate, "http://example.org/hobby");
            assert_eq!(subject, ALICE);
        }
        other => panic!("closed shape accepted an extra predicate: {:?}", other),
    }
    // The whitelist is exactly the shape's own paths plus sh:ignoredProperties.
    assert!(PersonShape::ALLOWED_PREDICATES.contains(&TYPE));
    assert!(!PersonShape::ALLOWED_PREDICATES.contains(&"http://example.org/hobby"));

    // 3. Cardinality is enforced, not merely typed.
    let mut two_names = base();
    two_names.push(Triple::literal(ALICE, "http://example.org/name", "Alicia", XSD_STRING));
    assert!(matches!(
        PersonShape::load(ALICE, &two_names),
        Err(LoadError::TooManyValues { max: 1, found: 2, .. })
    ));

    let missing_nickname: Vec<Triple> = base()
        .into_iter()
        .filter(|t| t.predicate != "http://example.org/nickname")
        .collect();
    assert!(matches!(
        PersonShape::load(ALICE, &missing_nickname),
        Err(LoadError::TooFewValues { min: 1, found: 0, .. })
    ));

    // 4. sh:datatype is checked, and the lexical form is really parsed.
    let mut bad_datatype = base();
    for t in &mut bad_datatype {
        if t.predicate == "http://example.org/age" {
            t.object_kind = ObjectKind::Literal { datatype: XSD_STRING };
        }
    }
    assert!(matches!(
        PersonShape::load(ALICE, &bad_datatype),
        Err(LoadError::DatatypeMismatch { .. })
    ));

    let mut bad_lexical = base();
    for t in &mut bad_lexical {
        if t.predicate == "http://example.org/age" {
            t.object = "forty-one";
        }
    }
    assert!(matches!(
        PersonShape::load(ALICE, &bad_lexical),
        Err(LoadError::LexicalForm { .. })
    ));

    // 5. sh:class is checked against rdf:type, and a missing type is rejected.
    let mut employed = base();
    employed.push(Triple::node(ALICE, "http://example.org/employer", "http://example.org/acme"));
    assert!(matches!(
        PersonShape::load(ALICE, &employed),
        Err(LoadError::ClassMismatch { .. })
    ));
    employed.push(Triple::node("http://example.org/acme", TYPE, "http://example.org/Organization"));
    let alice = PersonShape::load(ALICE, &employed).expect("acme is now typed");
    assert_eq!(alice.employer.expect("employer").iri(), "http://example.org/acme");

    // 6. An unconstrained property keeps whatever the triple carried.
    let mut noted = base();
    noted.push(Triple::literal(ALICE, "http://example.org/note", "hi", XSD_STRING));
    let alice = PersonShape::load(ALICE, &noted).expect("notes are unconstrained");
    assert_eq!(
        alice.note,
        vec![Value::Literal { lexical: "hi".to_string(), datatype: XSD_STRING.to_string() }]
    );

    // 7. An UNBOUNDED xsd:integer keeps its lexical form, so a value outside
    //    i64 — which xsd:integer's value space contains — still loads. An i64
    //    lowering would fail the second case with LoadError::LexicalForm.
    for lexical in ["7", BEYOND_I64] {
        let mut visited = base();
        visited.push(Triple::literal(ALICE, "http://example.org/visits", lexical, XSD_INTEGER));
        let alice = PersonShape::load(ALICE, &visited)
            .unwrap_or_else(|e| panic!("xsd:integer {} must load: {:?}", lexical, e));
        assert_eq!(alice.visits.as_deref(), Some(lexical));
    }
    // The datatype is still checked, even though the value stays lexical.
    let mut wrong_datatype = base();
    wrong_datatype.push(Triple::literal(ALICE, "http://example.org/visits", "7", XSD_LONG));
    assert!(matches!(
        PersonShape::load(ALICE, &wrong_datatype),
        Err(LoadError::DatatypeMismatch { .. })
    ));

    // 8. A recursive sh:node really nests, and cyclic data terminates.
    let mut managed = base();
    managed.push(Triple::node(ALICE, "http://example.org/manager", ALICE));
    match PersonShape::load(ALICE, &managed) {
        Err(LoadError::DepthExceeded { .. }) => {}
        other => panic!("a self-managing person must not recurse forever: {:?}", other),
    }

    println!("generated-loader behaviour verified");
}
"#;
