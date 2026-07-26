//! Forms F5 — SHACL-AF `sh:values` COMPUTED fields, evaluated on demand and
//! rendered read-only next to the asserted data. [OPUS-5] sq-lsp7k.1.5
//!
//! Whole-file gated: the evaluation path only exists with the opt-in `computed`
//! feature (which turns on `sparq-shacl/shacl-af`). The default-build contract
//! — flagged `computed`, read-only, EMPTY values — is asserted in
//! `tests/rdf12_roles.rs`, which runs in every feature state.
#![cfg(feature = "computed")]

use oxrdf::{NamedNode, Term};
use sparq_core::Graph;
use sparq_forms::{derive_form, to_sparql_update, FormDescription, FormField, FormOptions, Mode};

const PREFIXES: &str = r#"
  @prefix sh: <http://www.w3.org/ns/shacl#> .
  @prefix dash: <http://datashapes.org/dash#> .
  @prefix ex: <http://example.org/> .
"#;

fn g(ttl: &str) -> Graph {
    Graph::load_str(&format!("{PREFIXES}{ttl}"), "turtle").unwrap()
}

fn alice() -> Term {
    Term::from(NamedNode::new_unchecked("http://example.org/alice"))
}

fn field<'a>(form: &'a FormDescription, path: &str) -> &'a FormField {
    form.groups
        .iter()
        .flat_map(|group| &group.fields)
        .find(|f| f.path == format!("<http://example.org/{path}>"))
        .unwrap_or_else(|| panic!("no field for {path}"))
}

const DATA: &str = r#"
  ex:alice a ex:Person ; ex:name "Alice" ; ex:knows ex:bob, ex:carol ; ex:friendNames "stale" .
  ex:bob ex:name "Bob" .
  ex:carol ex:name "Carol" .
"#;

const SHAPES: &str = r#"
  ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person ;
    sh:property [ sh:path ex:name ; sh:name "Name" ; sh:order 1 ] ;
    sh:property [ sh:path ex:friendNames ; sh:name "Friend names" ; sh:order 2 ;
                  sh:values [ sh:path ( ex:knows ex:name ) ] ] .
"#;

/// A `sh:values` path expression is evaluated against the focus node, and the
/// computed values are rendered read-only NEXT TO the asserted fields.
#[test]
fn sh_values_path_expression_computes_the_field_values() {
    let form = derive_form(&g(DATA), &g(SHAPES), &alice(), &FormOptions::default());

    let asserted = field(&form, "name");
    assert!(!asserted.computed && asserted.editable);
    assert_eq!(asserted.values[0].term.value, "Alice");

    let computed = field(&form, "friendNames");
    assert!(computed.computed);
    assert!(!computed.editable, "computed fields are read-only");
    let mut values: Vec<&str> = computed
        .values
        .iter()
        .map(|v| v.term.value.as_str())
        .collect();
    values.sort_unstable();
    assert_eq!(values, vec!["Bob", "Carol"]);
    assert!(
        !values.contains(&"stale"),
        "the asserted ex:friendNames triple is NOT the computed value set"
    );
}

/// A constant / union expression evaluates too, and a computed field never
/// contributes to an update even when a renderer edits it.
#[test]
fn computed_values_never_reach_the_update() {
    let shapes = g(r#"
      ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person ;
        sh:property [ sh:path ex:tag ; sh:name "Tag" ; sh:values ex:constant ] .
    "#);
    let data = g("ex:alice a ex:Person ; ex:tag ex:asserted .");
    let before = derive_form(&data, &shapes, &alice(), &FormOptions::default());
    let tag = field(&before, "tag");
    assert_eq!(tag.values.len(), 1);
    assert_eq!(tag.values[0].term.value, "http://example.org/constant");

    let mut after = before.clone();
    after
        .groups
        .iter_mut()
        .flat_map(|group| &mut group.fields)
        .find(|f| f.computed)
        .unwrap()
        .values
        .clear();
    assert_eq!(
        to_sparql_update(&before, &after),
        "",
        "a computed field is not writable, so an echoed edit is dropped"
    );
}

/// An expression form the node-expression algebra does not support yields NO
/// values (the validator's lenient skip) — never the asserted data at the path.
#[test]
fn an_unsupported_expression_yields_no_values() {
    let shapes = g(r#"
      ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person ;
        sh:property [ sh:path ex:tag ; sh:name "Tag" ;
                      sh:values [ ex:unregisteredFunction ( ex:a ) ] ] .
    "#);
    let data = g("ex:alice a ex:Person ; ex:tag ex:asserted .");
    let form = derive_form(&data, &shapes, &alice(), &FormOptions::default());
    let tag = field(&form, "tag");
    assert!(tag.computed && !tag.editable);
    assert!(tag.values.is_empty());
}

/// Computed values are not asserted statements, so nothing reifies them: a
/// computed field carries no RDF 1.2 annotations even when the data graph
/// annotates the triple sitting at the same path.
#[test]
fn computed_values_carry_no_annotations() {
    let shapes = g(r#"
      ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person ;
        sh:property [ sh:path ex:tag ; sh:name "Tag" ; sh:values ex:constant ] .
    "#);
    let data = g("ex:alice a ex:Person . ex:alice ex:tag ex:asserted ~ex:r1 {| ex:source ex:x |} .");
    let form = derive_form(&data, &shapes, &alice(), &FormOptions::default());
    assert!(field(&form, "tag")
        .values
        .iter()
        .all(|v| v.annotations.is_empty()));
}

/// `Mode::View` changes nothing for a computed field: it is already read-only.
#[test]
fn view_mode_computed_fields_still_evaluate() {
    let form = derive_form(
        &g(DATA),
        &g(SHAPES),
        &alice(),
        &FormOptions {
            mode: Mode::View,
            ..FormOptions::default()
        },
    );
    let computed = field(&form, "friendNames");
    assert!(computed.computed && !computed.editable);
    assert_eq!(computed.values.len(), 2);
}
