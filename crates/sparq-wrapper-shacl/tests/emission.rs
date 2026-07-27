//! IR → Rust source, and the source's behaviour. [FABLE-5] (sq-1rg2q.12)
//!
//! Two obligations the IR test in `tests/lowering.rs` cannot discharge:
//!
//! 1. **Determinism** — generating twice from the same shapes graph must give
//!    byte-identical source, so a checked-in generated module only changes when
//!    the shapes do.
//! 2. **The generated code is real Rust that behaves** — the acceptance case
//!    shells out to `rustc` to COMPILE the emitted module (under `-D warnings`)
//!    together with a harness that runs it, and the harness proves the
//!    `sh:closed` loader rejects a predicate outside the whitelist, alongside
//!    the cardinality and datatype checks.
//!
//! Compiling from a test rather than trusting a string comparison is the point:
//! a codegen bug that produces plausible-looking but unparsable Rust would pass
//! any snapshot assertion.

use std::path::{Path, PathBuf};
use std::process::Command;

use sparq_core::Graph;
use sparq_shacl::ShapesModel;
use sparq_wrapper_shacl::generate;

/// The same coverage fixture `tests/lowering.rs` pins the IR of: required /
/// optional / repeated cardinalities, every checked scalar, a `sh:class`
/// reference, named and anonymous `sh:node` nesting, and `sh:closed`.
const SHAPES: &str = r#"
    @prefix sh:   <http://www.w3.org/ns/shacl#> .
    @prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
    @prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .
    @prefix ex:   <http://example.org/> .

    ex:PersonShape a sh:NodeShape ;
        sh:targetClass ex:Person ;
        sh:closed true ;
        sh:ignoredProperties ( rdf:type ) ;
        sh:property [ sh:path ex:name ;      sh:datatype xsd:string ;
                      sh:minCount 1 ; sh:maxCount 1 ] ;
        sh:property [ sh:path ex:age ;       sh:datatype xsd:integer ; sh:maxCount 1 ] ;
        sh:property [ sh:path ex:height ;    sh:datatype xsd:double ;
                      sh:minCount 0 ; sh:maxCount 1 ] ;
        sh:property [ sh:path ex:nickname ;  sh:datatype xsd:string ; sh:minCount 1 ] ;
        sh:property [ sh:path ex:score ;     sh:datatype xsd:decimal ;
                      sh:minCount 1 ; sh:maxCount 3 ] ;
        sh:property [ sh:path ex:active ;    sh:datatype xsd:boolean ; sh:maxCount 1 ] ;
        sh:property [ sh:path ex:homepage ;  sh:datatype xsd:anyURI ; sh:maxCount 1 ] ;
        sh:property [ sh:path ex:createdAt ; sh:datatype xsd:dateTime ;
                      sh:minCount 1 ; sh:maxCount 1 ] ;
        sh:property [ sh:path ex:employer ;  sh:class ex:Organization ;
                      sh:minCount 1 ; sh:maxCount 1 ] ;
        sh:property [ sh:path ex:knows ;     sh:class ex:Person ] ;
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

fn emit(shapes: &str) -> String {
    let graph = Graph::load_str(shapes, "turtle").expect("shapes graph must parse");
    generate(&ShapesModel::parse(&graph)).expect("the fixture is well-formed")
}

#[test]
fn emits_identical_rust_twice() {
    let first = emit(SHAPES);
    let second = emit(SHAPES);
    assert_eq!(first, second, "emission is not deterministic");

    // ...and from an independently parsed graph whose statements are in a
    // different order, which is where a hash-map iteration order would leak in.
    let reordered = SHAPES.replace(
        "ex:AddressShape a sh:NodeShape ;\n        sh:property [ sh:path ex:city ; sh:datatype xsd:string ;\n                      sh:minCount 1 ; sh:maxCount 1 ] .",
        "",
    );
    assert_ne!(reordered, SHAPES, "the fixture rewrite did nothing");
    let reordered = format!(
        "{}\n    ex:AddressShape a sh:NodeShape ;\n        sh:property [ sh:path ex:city ; sh:datatype xsd:string ;\n                      sh:minCount 1 ; sh:maxCount 1 ] .\n",
        reordered
    );
    assert_eq!(first, emit(&reordered), "emission depends on statement order");
}

#[test]
fn the_emitted_module_is_self_contained() {
    let source = emit(SHAPES);
    let imports: Vec<&str> = source
        .lines()
        .filter(|l| l.starts_with("use ") || l.starts_with("extern crate "))
        .collect();
    assert_eq!(
        imports,
        ["use std::fmt;"],
        "the generated module must depend on nothing but std"
    );
    assert!(source.starts_with("// @generated"), "missing the generated marker");
}

/// The acceptance case: compile the emitted module with `rustc` and run a
/// harness against it.
#[test]
fn generated_code_compiles_and_the_closed_loader_rejects_an_extra_predicate() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("sq-1rg2q-12");
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clearing the previous run");
    }
    std::fs::create_dir_all(&dir).expect("creating the scratch dir");
    std::fs::write(dir.join("generated.rs"), emit(SHAPES)).expect("writing the generated module");
    std::fs::write(dir.join("main.rs"), HARNESS).expect("writing the harness");

    let binary = dir.join("harness");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let compile = Command::new(&rustc)
        .args(["--edition", "2021", "-D", "warnings", "-o"])
        .arg(&binary)
        .arg(dir.join("main.rs"))
        .current_dir(&dir)
        .output()
        .expect("failed to spawn rustc");
    assert!(
        compile.status.success(),
        "the generated module does not compile under `-D warnings`:\n{}\n--- generated ---\n{}",
        String::from_utf8_lossy(&compile.stderr),
        numbered(&dir.join("generated.rs")),
    );

    let run = Command::new(&binary)
        .output()
        .expect("failed to run the compiled harness");
    assert!(
        run.status.success(),
        "the generated loaders misbehaved:\n--- stdout ---\n{}\n--- stderr ---\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    );
    // Every named check must actually have run — a harness that exited 0 after
    // asserting nothing would otherwise look like a pass.
    let stdout = String::from_utf8_lossy(&run.stdout);
    for check in [
        "loads-conforming",
        "rejects-extra-predicate",
        "allows-ignored-predicate",
        "rejects-bad-datatype",
        "rejects-missing-required",
        "rejects-too-many",
        "rejects-nested-violation",
    ] {
        assert!(stdout.contains(check), "harness never ran {}:\n{}", check, stdout);
    }
}

/// Renders a file with line numbers, so a compile failure points at the exact
/// emitted line rather than at a wall of generated text.
fn numbered(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .enumerate()
        .map(|(i, line)| format!("{:>4} | {}\n", i + 1, line))
        .collect()
}

/// The `main.rs` compiled alongside the generated module. It implements
/// `ValueSource` over a triple list and exercises each loader guarantee.
const HARNESS: &str = r##"mod generated;

use generated::{AddressShape, LoadError, OrganizationRef, PersonRef, PersonShape, ValueSource};

struct Fixture(Vec<(String, String, String)>);

impl ValueSource for Fixture {
    fn predicates(&self, subject: &str) -> Vec<String> {
        self.0.iter().filter(|t| t.0 == subject).map(|t| t.1.clone()).collect()
    }
    fn values(&self, subject: &str, predicate: &str) -> Vec<String> {
        self.0
            .iter()
            .filter(|t| t.0 == subject && t.1 == predicate)
            .map(|t| t.2.clone())
            .collect()
    }
}

const ALICE: &str = "http://example.org/alice";
const ADDR: &str = "http://example.org/addr1";

fn base() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        (ALICE, "http://example.org/name", "Alice"),
        (ALICE, "http://example.org/age", "42"),
        (ALICE, "http://example.org/createdAt", "2026-07-27T10:30:00Z"),
        (ALICE, "http://example.org/nickname", "Ali"),
        (ALICE, "http://example.org/nickname", "Al"),
        (ALICE, "http://example.org/score", "1.5"),
        (ALICE, "http://example.org/score", "2"),
        (ALICE, "http://example.org/employer", "http://example.org/acme"),
        (ALICE, "http://example.org/knows", "http://example.org/bob"),
        (ALICE, "http://example.org/address", ADDR),
        (ADDR, "http://example.org/city", "Dublin"),
    ]
}

fn source(triples: Vec<(&'static str, &'static str, &'static str)>) -> Fixture {
    Fixture(
        triples
            .into_iter()
            .map(|(s, p, o)| (s.to_string(), p.to_string(), o.to_string()))
            .collect(),
    )
}

fn check(name: &str) {
    println!("{}", name);
}

fn main() {
    assert_eq!(AddressShape::SHAPE, "http://example.org/AddressShape");
    assert_eq!(PersonShape::TARGET_CLASSES, &["http://example.org/Person"]);

    // 1. A conforming subject loads, with each cardinality wrapped as the
    //    mapping promises.
    let person = PersonShape::load(&source(base()), ALICE).expect("the fixture conforms");
    assert_eq!(person.iri, ALICE);
    assert_eq!(person.name, "Alice");
    assert_eq!(person.created_at, "2026-07-27T10:30:00Z");
    assert_eq!(person.age, Some(42));
    assert_eq!(person.active, None);
    assert_eq!(person.height, None);
    assert_eq!(person.homepage, None);
    assert_eq!(person.pronouns, None);
    assert_eq!(person.nickname, vec!["Ali".to_string(), "Al".to_string()]);
    assert_eq!(person.score, vec![1.5, 2.0]);
    assert_eq!(person.employer, OrganizationRef("http://example.org/acme".to_string()));
    assert_eq!(person.knows, vec![PersonRef("http://example.org/bob".to_string())]);
    assert_eq!(person.address.city, "Dublin");
    assert_eq!(person.address.iri, ADDR);
    check("loads-conforming");

    // 2. THE CLOSED-SHAPE OBLIGATION: a predicate outside the whitelist is
    //    rejected, naming the offending predicate.
    let mut extra = base();
    extra.push((ALICE, "http://example.org/secret", "leaked"));
    match PersonShape::load(&source(extra), ALICE) {
        Err(LoadError::ClosedShape { type_name, predicate }) => {
            assert_eq!(type_name, "PersonShape");
            assert_eq!(predicate, "http://example.org/secret");
        }
        other => panic!("closed shape accepted an extra predicate: {:?}", other),
    }
    check("rejects-extra-predicate");

    // 3. ...but sh:ignoredProperties is honoured.
    let mut ignored = base();
    ignored.push((
        ALICE,
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
        "http://example.org/Person",
    ));
    PersonShape::load(&source(ignored), ALICE).expect("rdf:type is ignored, not forbidden");
    check("allows-ignored-predicate");

    // 4. A scalar whose lexical form is not valid for its sh:datatype.
    let bad = base()
        .into_iter()
        .map(|t| if t.1 == "http://example.org/age" { (t.0, t.1, "forty-two") } else { t })
        .collect();
    match PersonShape::load(&source(bad), ALICE) {
        Err(LoadError::Datatype { field, datatype, value, .. }) => {
            assert_eq!(field, "age");
            assert_eq!(datatype, "http://www.w3.org/2001/XMLSchema#integer");
            assert_eq!(value, "forty-two");
        }
        other => panic!("an unchecked scalar was accepted: {:?}", other),
    }
    check("rejects-bad-datatype");

    // 5. A missing sh:minCount 1 / sh:maxCount 1 value.
    let missing = base()
        .into_iter()
        .filter(|t| t.1 != "http://example.org/name")
        .collect();
    match PersonShape::load(&source(missing), ALICE) {
        Err(LoadError::Cardinality { field, found, min, max, .. }) => {
            assert_eq!(field, "name");
            assert_eq!((found, min, max), (0, 1, Some(1)));
        }
        other => panic!("a required field was allowed to be absent: {:?}", other),
    }
    check("rejects-missing-required");

    // 6. More values than sh:maxCount allows on a Vec field.
    let mut many = base();
    many.push((ALICE, "http://example.org/score", "3"));
    many.push((ALICE, "http://example.org/score", "4"));
    match PersonShape::load(&source(many), ALICE) {
        Err(LoadError::Cardinality { field, found, min, max, .. }) => {
            assert_eq!(field, "score");
            assert_eq!((found, min, max), (4, 1, Some(3)));
        }
        other => panic!("a Vec field ignored its sh:maxCount: {:?}", other),
    }
    check("rejects-too-many");

    // 7. A nested sh:node violation surfaces with the NESTED type's name.
    let nested = base()
        .into_iter()
        .filter(|t| t.1 != "http://example.org/city")
        .collect();
    match PersonShape::load(&source(nested), ALICE) {
        Err(LoadError::Cardinality { type_name, field, .. }) => {
            assert_eq!(type_name, "AddressShape");
            assert_eq!(field, "city");
        }
        other => panic!("a nested violation did not propagate: {:?}", other),
    }
    check("rejects-nested-violation");
}
"##;
