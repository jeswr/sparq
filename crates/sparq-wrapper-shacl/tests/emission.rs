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
//!    the cardinality, datatype, `sh:class` membership and XSD value-space
//!    checks.
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
        "rejects-wrong-datatype",
        "rejects-non-literal-scalar",
        "rejects-literal-reference",
        "rejects-unclassified-reference",
        "requires-subclass-entailment",
        "accepts-exact-datatype",
        "keeps-integers-beyond-i64",
        "keeps-decimal-precision",
        "rejects-missing-required",
        "rejects-too-many",
        "rejects-nested-violation",
        "checks-date-time-boundaries",
        "checks-any-uri-boundaries",
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

use generated::{
    AddressShape, Decimal, Integer, LoadError, OrganizationRef, PersonRef, PersonShape, Value,
    ValueSource,
};

struct Fixture(Vec<(String, String, Value)>);

impl ValueSource for Fixture {
    fn predicates(&self, subject: &str) -> Vec<String> {
        self.0.iter().filter(|t| t.0 == subject).map(|t| t.1.clone()).collect()
    }
    fn values(&self, subject: &str, predicate: &str) -> Vec<Value> {
        self.0
            .iter()
            .filter(|t| t.0 == subject && t.1 == predicate)
            .map(|t| t.2.clone())
            .collect()
    }
    /// The entailment the trait specifies: `rdf:type/rdfs:subClassOf*`, walked
    /// transitively over the fixture's own triples. Nothing else is entailed.
    fn is_instance_of(&self, node: &str, class: &str) -> bool {
        let named = |source: &Self, subject: &str, predicate: &str| -> Vec<String> {
            source
                .values(subject, predicate)
                .into_iter()
                .filter_map(|v| match v {
                    Value::NamedNode(iri) => Some(iri),
                    _ => None,
                })
                .collect()
        };
        let mut frontier = named(self, node, RDF_TYPE);
        let mut seen: Vec<String> = Vec::new();
        while let Some(current) = frontier.pop() {
            if current == class {
                return true;
            }
            if seen.contains(&current) {
                continue;
            }
            frontier.extend(named(self, &current, SUB_CLASS_OF));
            seen.push(current);
        }
        false
    }
}

const ALICE: &str = "http://example.org/alice";
const ADDR: &str = "http://example.org/addr1";
const AGE: &str = "http://example.org/age";
const CREATED_AT: &str = "http://example.org/createdAt";
const SCORE: &str = "http://example.org/score";
const EMPLOYER: &str = "http://example.org/employer";
const ACME: &str = "http://example.org/acme";
const BOB: &str = "http://example.org/bob";
const EMPLOYEE: &str = "http://example.org/Employee";
const ORGANIZATION: &str = "http://example.org/Organization";
const PERSON: &str = "http://example.org/Person";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const SUB_CLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
const XSD: &str = "http://www.w3.org/2001/XMLSchema#";

/// A literal of an XSD datatype, named by its local part.
fn lit(lexical: &str, local: &str) -> Value {
    Value::Literal {
        lexical: lexical.to_string(),
        datatype: format!("{}{}", XSD, local),
        language: None,
    }
}

fn iri(value: &str) -> Value {
    Value::NamedNode(value.to_string())
}

fn base() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        (ALICE, "http://example.org/name", lit("Alice", "string")),
        (ALICE, AGE, lit("42", "integer")),
        (ALICE, CREATED_AT, lit("2026-07-27T10:30:00Z", "dateTime")),
        (ALICE, "http://example.org/nickname", lit("Ali", "string")),
        (ALICE, "http://example.org/nickname", lit("Al", "string")),
        (ALICE, SCORE, lit("1.5", "decimal")),
        (ALICE, SCORE, lit("2", "decimal")),
        (ALICE, EMPLOYER, iri(ACME)),
        (ALICE, "http://example.org/knows", iri(BOB)),
        (ALICE, "http://example.org/address", iri(ADDR)),
        (ADDR, "http://example.org/city", lit("Dublin", "string")),
        // The class memberships sh:class requires: acme directly, bob only
        // through the subclass step.
        (ACME, RDF_TYPE, iri(ORGANIZATION)),
        (BOB, RDF_TYPE, iri(EMPLOYEE)),
        (EMPLOYEE, SUB_CLASS_OF, iri(PERSON)),
    ]
}

/// The base fixture with the triples matching `drop` removed.
fn without(drop: impl Fn(&(&'static str, &'static str, Value)) -> bool) -> Vec<(&'static str, &'static str, Value)> {
    base().into_iter().filter(|t| !drop(t)).collect()
}

/// The base fixture with Alice's `ex:age` — an `xsd:integer` field — replaced.
fn with_age(value: Value) -> Vec<(&'static str, &'static str, Value)> {
    base()
        .into_iter()
        .map(|t| if t.1 == AGE { (t.0, t.1, value.clone()) } else { t })
        .collect()
}

/// Whether `lexical` is accepted for the `xsd:dateTime` field, distinguishing a
/// lexical rejection from any other load failure.
fn date_time_ok(lexical: &str) -> bool {
    let triples = base()
        .into_iter()
        .map(|t| {
            if t.1 == CREATED_AT { (t.0, t.1, lit(lexical, "dateTime")) } else { t }
        })
        .collect();
    match PersonShape::load(&source(triples), ALICE) {
        Ok(person) => {
            assert_eq!(person.created_at, lexical);
            true
        }
        Err(LoadError::Datatype { field, .. }) if field == "created_at" => false,
        other => panic!("unexpected result for {:?}: {:?}", lexical, other),
    }
}

/// The same, for the optional `xsd:anyURI` field.
fn any_uri_ok(lexical: &str) -> bool {
    let mut triples = base();
    triples.push((ALICE, "http://example.org/homepage", lit(lexical, "anyURI")));
    match PersonShape::load(&source(triples), ALICE) {
        Ok(person) => {
            assert_eq!(person.homepage.as_deref(), Some(lexical));
            true
        }
        Err(LoadError::Datatype { field, .. }) if field == "homepage" => false,
        other => panic!("unexpected result for {:?}: {:?}", lexical, other),
    }
}

fn source(triples: Vec<(&'static str, &'static str, Value)>) -> Fixture {
    Fixture(
        triples
            .into_iter()
            .map(|(s, p, o)| (s.to_string(), p.to_string(), o))
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
    assert_eq!(person.age, Integer::parse("42"));
    assert_eq!(person.active, None);
    assert_eq!(person.height, None);
    assert_eq!(person.homepage, None);
    assert_eq!(person.pronouns, None);
    assert_eq!(person.nickname, vec!["Ali".to_string(), "Al".to_string()]);
    // "2" is the same decimal as "2.0", and canonicalisation says so.
    assert_eq!(
        person.score.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
        ["1.5", "2.0"]
    );
    assert_eq!(person.employer, OrganizationRef(ACME.to_string()));
    // Bob is a Person only through ex:Employee rdfs:subClassOf ex:Person.
    assert_eq!(person.knows, vec![PersonRef(BOB.to_string())]);
    assert_eq!(person.address.city, "Dublin");
    assert_eq!(person.address.iri, ADDR);
    check("loads-conforming");

    // 2. THE CLOSED-SHAPE OBLIGATION: a predicate outside the whitelist is
    //    rejected, naming the offending predicate.
    let mut extra = base();
    extra.push((ALICE, "http://example.org/secret", lit("leaked", "string")));
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
        iri("http://example.org/Person"),
    ));
    PersonShape::load(&source(ignored), ALICE).expect("rdf:type is ignored, not forbidden");
    check("allows-ignored-predicate");

    // 4. A correctly typed literal whose LEXICAL FORM is not valid for its
    //    sh:datatype.
    match PersonShape::load(&source(with_age(lit("forty-two", "integer"))), ALICE) {
        Err(LoadError::Datatype { field, datatype, value, .. }) => {
            assert_eq!(field, "age");
            assert_eq!(datatype, "http://www.w3.org/2001/XMLSchema#integer");
            assert_eq!(value, "forty-two");
        }
        other => panic!("an unchecked scalar was accepted: {:?}", other),
    }
    check("rejects-bad-datatype");

    // 4b. THE TERM-LEVEL OBLIGATION: sh:datatype constrains the RDF term, not
    //     the characters. "42"^^xsd:string parses as an integer and must STILL
    //     be rejected on an xsd:integer field — the case a loader that saw only
    //     lexical forms could not tell apart from the conforming value.
    match PersonShape::load(&source(with_age(lit("42", "string"))), ALICE) {
        Err(LoadError::TermType { field, expected, found, .. }) => {
            assert_eq!(field, "age");
            assert_eq!(expected, "http://www.w3.org/2001/XMLSchema#integer");
            assert_eq!(found, lit("42", "string"));
        }
        other => panic!("a wrongly typed literal was accepted: {:?}", other),
    }
    check("rejects-wrong-datatype");

    // 4c. ...and neither is an IRI that happens to spell a valid integer.
    match PersonShape::load(&source(with_age(iri("42"))), ALICE) {
        Err(LoadError::TermType { field, expected, found, .. }) => {
            assert_eq!(field, "age");
            assert_eq!(expected, "http://www.w3.org/2001/XMLSchema#integer");
            assert_eq!(found, iri("42"));
        }
        other => panic!("a named node was accepted as a literal: {:?}", other),
    }
    check("rejects-non-literal-scalar");

    // 4d. The mirror image: a sh:class reference is an IRI, so a literal that
    //     spells one is not a value of it.
    let literal_ref = base()
        .into_iter()
        .map(|t| if t.1 == EMPLOYER { (t.0, t.1, lit(ACME, "string")) } else { t })
        .collect();
    match PersonShape::load(&source(literal_ref), ALICE) {
        Err(LoadError::TermType { field, expected, .. }) => {
            assert_eq!(field, "employer");
            assert_eq!(expected, "an IRI");
        }
        other => panic!("a literal was accepted as a typed reference: {:?}", other),
    }
    check("rejects-literal-reference");

    // 4f. THE sh:class OBLIGATION: being an IRI is not being an INSTANCE. An
    //     unrelated resource — a perfectly good IRI, just not an
    //     ex:Organization — must not load as an OrganizationRef, or the newtype
    //     would be a label rather than a constraint.
    let unclassified = base()
        .into_iter()
        .map(|t| {
            if t.1 == EMPLOYER { (t.0, t.1, iri("http://example.org/dublin")) } else { t }
        })
        .collect();
    match PersonShape::load(&source(unclassified), ALICE) {
        Err(LoadError::ClassMembership { type_name, field, class, node }) => {
            assert_eq!(type_name, "PersonShape");
            assert_eq!(field, "employer");
            assert_eq!(class, ORGANIZATION);
            assert_eq!(node, "http://example.org/dublin");
        }
        other => panic!("an unrelated IRI loaded as a typed reference: {:?}", other),
    }
    check("rejects-unclassified-reference");

    // 4g. ...and the membership test is the SPECIFIED entailment, not just a
    //     direct rdf:type lookup: bob conforms in `base()` only because
    //     ex:Employee rdfs:subClassOf ex:Person. Drop that one triple and the
    //     very same IRI must stop conforming — which is what makes 4f a real
    //     check rather than a checker that rejects everything unfamiliar.
    match PersonShape::load(&source(without(|t| t.0 == EMPLOYEE)), ALICE) {
        Err(LoadError::ClassMembership { field, class, node, .. }) => {
            assert_eq!(field, "knows");
            assert_eq!(class, PERSON);
            assert_eq!(node, BOB);
        }
        other => panic!("membership ignored the subclass step: {:?}", other),
    }
    check("requires-subclass-entailment");

    // 4e. The conforming value of the very same field still loads, so the three
    //     rejections above are not a checker that rejects everything.
    let ok = PersonShape::load(&source(with_age(lit("42", "integer"))), ALICE)
        .expect("a correctly typed xsd:integer must still load");
    assert_eq!(ok.age, Integer::parse("42"));
    check("accepts-exact-datatype");

    // 4h. THE VALUE-SPACE OBLIGATION: xsd:integer is UNBOUNDED, so a value
    //     beyond i64 is conforming and must load exactly, not be reported as an
    //     invalid integer. Narrowing stays available, and stays explicit.
    let huge = "99999999999999999999999999999999";
    let loaded = PersonShape::load(&source(with_age(lit(huge, "integer"))), ALICE)
        .expect("an xsd:integer beyond i64 is still an xsd:integer");
    let loaded = loaded.age.expect("the field is present");
    assert_eq!(loaded.as_str(), huge);
    assert_eq!(loaded.to_i64(), None, "narrowing must be visibly fallible");
    // ...and canonicalisation is by VALUE: 007, +7 and 7 are one integer.
    assert_eq!(Integer::parse("007"), Integer::parse("7"));
    assert_eq!(Integer::parse("+7"), Integer::parse("7"));
    assert_eq!(Integer::parse("-0"), Integer::parse("0"));
    assert_eq!(Integer::parse("12.0"), None);
    check("keeps-integers-beyond-i64");

    // 4i. The same for xsd:decimal, which is arbitrary PRECISION as well as
    //     unbounded. Two values that f64 cannot tell apart must stay distinct,
    //     and a magnitude past f64's range must not silently become INF — which
    //     is not an xsd:decimal value at all.
    let near = ["0.1000000000000000000000001", "0.1000000000000000000000002"];
    let big = format!("1{}", "0".repeat(400));
    let mut scores = without(|t| t.1 == SCORE);
    for lexical in [near[0], near[1]] {
        scores.push((ALICE, SCORE, lit(lexical, "decimal")));
    }
    let decimals = PersonShape::load(&source(scores.clone()), ALICE)
        .expect("both decimals conform")
        .score;
    assert_eq!(decimals.len(), 2);
    assert_eq!(decimals[0].as_str(), near[0]);
    assert_ne!(decimals[0], decimals[1], "f64 rounding collapsed two values");
    assert_eq!(decimals[0].to_f64(), decimals[1].to_f64(), "the f64s DO collide");

    let mut scores = without(|t| t.1 == SCORE);
    scores.push((ALICE, SCORE, lit(&big, "decimal")));
    let loaded = PersonShape::load(&source(scores), ALICE)
        .expect("a large xsd:decimal conforms")
        .score;
    assert_eq!(loaded[0].as_str(), format!("{}.0", big));
    assert!(loaded[0].to_f64().is_infinite(), "the f64 view DOES overflow");
    // Canonicalisation is by value here too.
    assert_eq!(Decimal::parse("1.50"), Decimal::parse("1.5"));
    assert_eq!(Decimal::parse("2"), Decimal::parse("2.0"));
    assert_eq!(Decimal::parse("-0.0"), Decimal::parse("0.0"));
    assert_eq!(Decimal::parse("1e3"), None, "an exponent is xsd:double, not decimal");
    assert_eq!(Decimal::parse("INF"), None);
    check("keeps-decimal-precision");

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
    many.push((ALICE, "http://example.org/score", lit("3", "decimal")));
    many.push((ALICE, "http://example.org/score", lit("4", "decimal")));
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

    // 8. The XSD value space, not just the field widths: every case below is one
    //    a checker that only bounded each field would get wrong.
    for valid in [
        "2024-02-29T00:00:00Z",           // a leap day in a leap year
        "2000-02-29T00:00:00Z",           // ...and in a 400-year leap year
        "2026-12-31T24:00:00Z",           // 24:00:00 is midnight starting the next day
        "2026-07-27T10:30:00+14:00",      // the largest timezone offset
        "2026-07-27T10:30:00.500-05:00",
        "-0044-03-15T12:00:00Z",          // a BCE year
    ] {
        assert!(date_time_ok(valid), "rejected a valid xsd:dateTime: {}", valid);
    }
    for invalid in [
        "2026-02-29T00:00:00Z",      // 2026 is not a leap year
        "1900-02-29T00:00:00Z",      // nor is a non-400 century
        "2026-04-31T00:00:00Z",      // April has 30 days
        "2026-07-27T24:30:00Z",      // hour 24 admits no minutes
        "2026-07-27T10:30:60Z",      // XSD dateTime has no leap second
        "2026-07-27T10:30:00+14:30", // beyond the ±14:00 offset bound
    ] {
        assert!(!date_time_ok(invalid), "accepted an invalid xsd:dateTime: {}", invalid);
    }
    check("checks-date-time-boundaries");

    // The empty URI reference IS in the xsd:anyURI value space.
    for valid in ["", "http://example.org/", "urn:example:1", "#fragment"] {
        assert!(any_uri_ok(valid), "rejected a valid xsd:anyURI: {:?}", valid);
    }
    for invalid in ["has a space", "angle<bracket>", "back\\slash"] {
        assert!(!any_uri_ok(invalid), "accepted an invalid xsd:anyURI: {:?}", invalid);
    }
    check("checks-any-uri-boundaries");
}
"##;
