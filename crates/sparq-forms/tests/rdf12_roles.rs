//! Forms F5 — RDF 1.2 annotation editing, DASH property roles, `dash:abstract`
//! instantiation exclusion, and the base-build contract for computed
//! (`sh:values`) fields. [OPUS-5] sq-lsp7k.1.5
//!
//! The `sh:values` EVALUATION path lives in `tests/computed_fields.rs` behind
//! the opt-in `computed` feature; the flag-and-empty contract asserted here is
//! what a DEFAULT build must guarantee.

use oxrdf::{NamedNode, Term};
use sparq_core::Graph;
use sparq_engine::update;
use sparq_forms::{
    applicable_shapes, derive_form, to_sparql_update, FormDescription, FormField, FormOptions, Mode,
};
use sparq_shacl::ShapesModel;

const PREFIXES: &str = r#"
  @prefix sh: <http://www.w3.org/ns/shacl#> .
  @prefix dash: <http://datashapes.org/dash#> .
  @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
  @prefix ex: <http://example.org/> .
"#;

fn g(ttl: &str) -> Graph {
    Graph::load_str(&format!("{PREFIXES}{ttl}"), "turtle").unwrap()
}

fn iri(local: &str) -> Term {
    Term::from(NamedNode::new_unchecked(format!(
        "http://example.org/{local}"
    )))
}

fn field<'a>(form: &'a FormDescription, path: &str) -> &'a FormField {
    form.groups
        .iter()
        .flat_map(|group| &group.fields)
        .find(|f| f.path == format!("<http://example.org/{path}>"))
        .unwrap_or_else(|| panic!("no field for {path}"))
}

// ---- (1) RDF 1.2 triple-term annotation sub-fields --------------------------

const ANNOTATED_SHAPES: &str = r#"
  ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person ;
    sh:property [ sh:path ex:age ; sh:name "Age" ; sh:datatype xsd:integer ] .
"#;

/// The `{| … |}` annotation syntax mints a reifier of the asserted statement.
/// The asserted value stays the field value; the reifier's provenance hangs off
/// it as annotation sub-fields — and `rdf:reifies` itself never shows up as
/// metadata.
#[test]
fn annotations_render_as_sub_fields_of_the_asserted_value() {
    let data = g(r#"
      ex:alice a ex:Person .
      ex:alice ex:age 41 ~ex:r1 {| ex:source ex:census ; ex:confidence 0.9 |} .
    "#);
    let form = derive_form(&data, &g(ANNOTATED_SHAPES), &iri("alice"), &FormOptions::default());

    let age = field(&form, "age");
    assert_eq!(age.values.len(), 1);
    assert_eq!(age.values[0].term.value, "41");

    let annotations = &age.values[0].annotations;
    assert_eq!(annotations.len(), 1, "one reifier");
    let a = &annotations[0];
    assert_eq!(a.reifier.value, "http://example.org/r1");
    // The reified statement is carried structurally, not as opaque text.
    assert_eq!(a.statement.subject.value, "http://example.org/alice");
    assert_eq!(a.statement.predicate.value, "http://example.org/age");
    assert_eq!(a.statement.object.value, "41");

    let predicates: Vec<&str> = a.fields.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(
        predicates,
        vec![
            "<http://example.org/confidence>",
            "<http://example.org/source>"
        ],
        "rdf:reifies is the link, not annotation metadata"
    );
    assert!(
        a.fields.iter().all(|f| f.editable),
        "an IRI reifier's annotations are editable in edit mode"
    );
}

/// A blank-node reifier (what bare `{| … |}` mints) still RENDERS, but derives
/// read-only: SPARQL Update forbids blank nodes in a `DELETE` template, so
/// there is no safe write-back path. Its label is normalised like every other
/// blank node in the description.
#[test]
fn blank_node_reifier_annotations_are_read_only() {
    let data = g(r#"
      ex:alice a ex:Person .
      ex:alice ex:age 41 {| ex:source ex:census |} .
    "#);
    let form = derive_form(&data, &g(ANNOTATED_SHAPES), &iri("alice"), &FormOptions::default());
    let a = &field(&form, "age").values[0].annotations[0];
    assert_eq!(a.reifier.kind, "bnode");
    assert!(
        a.reifier.value.starts_with('b') && a.reifier.value.len() <= 3,
        "parser blank-node ids never leak: got {}",
        a.reifier.value
    );
    assert!(
        a.fields.iter().all(|f| !f.editable),
        "a blank-node reifier cannot be written back, so it renders read-only"
    );
    // Two derivations of the same graph agree (the determinism contract).
    let again = derive_form(&data, &g(ANNOTATED_SHAPES), &iri("alice"), &FormOptions::default());
    assert_eq!(form, again);
}

/// Editing annotation metadata produces an UPDATE written against the reifier,
/// not the focus node — and applying it round-trips.
#[test]
fn annotation_edits_write_back_against_the_reifier() {
    let data = g(r#"
      ex:alice a ex:Person .
      ex:alice ex:age 41 ~ex:r1 {| ex:source ex:census |} .
    "#);
    let shapes = g(ANNOTATED_SHAPES);
    let before = derive_form(&data, &shapes, &iri("alice"), &FormOptions::default());
    let mut after = before.clone();
    let source = after
        .groups
        .iter_mut()
        .flat_map(|group| &mut group.fields)
        .flat_map(|f| &mut f.values)
        .flat_map(|v| &mut v.annotations)
        .flat_map(|a| &mut a.fields)
        .find(|f| f.path == "<http://example.org/source>")
        .unwrap();
    source.values[0].term.value = "http://example.org/survey".into();

    let text = to_sparql_update(&before, &after);
    assert!(
        text.contains("<http://example.org/r1> <http://example.org/source> <http://example.org/census>"),
        "the DELETE names the reifier as subject: {text}"
    );
    assert!(
        text.contains("<http://example.org/r1> <http://example.org/source> <http://example.org/survey>"),
        "the INSERT names the reifier as subject: {text}"
    );
    assert!(
        !text.contains("<http://example.org/alice> <http://example.org/source>"),
        "an annotation is never written against the focus node: {text}"
    );

    let updated = update(&data, &text).unwrap();
    let derived = derive_form(&updated, &shapes, &iri("alice"), &FormOptions::default());
    assert_eq!(derived, after, "the edit round-trips through the store");
}

/// A read-only annotation (blank-node reifier) contributes nothing to the diff
/// even when a renderer echoes back an edited value.
#[test]
fn blank_node_reifier_edits_never_reach_the_update() {
    let data = g(r#"
      ex:alice a ex:Person .
      ex:alice ex:age 41 {| ex:source ex:census |} .
    "#);
    let shapes = g(ANNOTATED_SHAPES);
    let before = derive_form(&data, &shapes, &iri("alice"), &FormOptions::default());
    let mut after = before.clone();
    after.groups[0].fields[0].values[0].annotations[0].fields[0].values[0]
        .term
        .value = "http://example.org/survey".into();
    assert_eq!(to_sparql_update(&before, &after), "");
}

/// `Mode::View` derives annotations read-only; a complex (sequence) path names
/// no single statement, so it carries none at all.
#[test]
fn view_mode_and_complex_paths_carry_no_editable_annotations() {
    let data = g(r#"
      ex:alice a ex:Person .
      ex:alice ex:age 41 ~ex:r1 {| ex:source ex:census |} .
    "#);
    let shapes = g(ANNOTATED_SHAPES);
    let view = derive_form(
        &data,
        &shapes,
        &iri("alice"),
        &FormOptions {
            mode: Mode::View,
            ..FormOptions::default()
        },
    );
    let a = &field(&view, "age").values[0].annotations[0];
    assert!(a.fields.iter().all(|f| !f.editable));

    // A sequence path names no single statement, so its DECLARED field carries
    // no annotations. (`ex:age` is then off-shape, so the implicit "Other
    // properties" group still surfaces the annotated triple — read-only.)
    let seq_shapes = g(r#"
      ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person ;
        sh:property [ sh:path ( ex:knows ex:name ) ; sh:name "Friend name" ] .
    "#);
    let form = derive_form(&data, &seq_shapes, &iri("alice"), &FormOptions::default());
    let declared: Vec<_> = form
        .groups
        .iter()
        .flat_map(|group| &group.fields)
        .filter(|f| f.property_shape.is_some())
        .collect();
    assert_eq!(declared.len(), 1);
    assert!(declared[0].values.iter().all(|v| v.annotations.is_empty()));
}

/// An off-shape ("Other properties") value shows its annotations too — the
/// implicit group is the only place an un-shaped statement appears.
#[test]
fn off_shape_values_carry_annotations() {
    let data = g(r#"
      ex:alice a ex:Person .
      ex:alice ex:nickname "Ally" ~ex:r2 {| ex:source ex:selfReported |} .
    "#);
    let form = derive_form(
        &data,
        &g("ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person ."),
        &iri("alice"),
        &FormOptions::default(),
    );
    let nickname = field(&form, "nickname");
    assert!(!nickname.editable, "off-shape fields stay read-only");
    assert_eq!(nickname.values[0].annotations[0].reifier.value, "http://example.org/r2");
    assert!(
        nickname.values[0].annotations[0]
            .fields
            .iter()
            .all(|f| !f.editable),
        "an off-shape annotation is read-only along with its statement"
    );
}

/// A form value that IS a triple term carries the quoted statement
/// structurally (a renderer draws `s p o`, not N-Triples text).
#[test]
fn triple_term_values_expose_their_components() {
    let data = g(r#"
      ex:r1 a ex:Claim .
      ex:r1 <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> <<( ex:alice ex:age 41 )>> .
    "#);
    let shapes = g(r#"
      ex:ClaimShape a sh:NodeShape ; sh:targetClass ex:Claim ;
        sh:property [
          sh:path <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ;
          sh:name "Reifies" ] .
    "#);
    let form = derive_form(&data, &shapes, &iri("r1"), &FormOptions::default());
    let term = &form.groups[0].fields[0].values[0].term;
    assert_eq!(term.kind, "triple");
    let t = term.triple.as_ref().expect("structured components");
    assert_eq!(t.subject.value, "http://example.org/alice");
    assert_eq!(t.predicate.value, "http://example.org/age");
    assert_eq!(t.object.value, "41");
}

// ---- (3) dash:propertyRole --------------------------------------------------

/// `dash:propertyRole dash:LabelRole` is carried per field AND supplies the
/// focus node's display label, taken in RENDER order (`sh:order`).
#[test]
fn label_role_drives_the_display_label() {
    let shapes = g(r#"
      ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person ;
        sh:property [ sh:path ex:preferredName ; sh:name "Preferred" ; sh:order 1 ;
                      dash:propertyRole dash:LabelRole ] ;
        sh:property [ sh:path ex:legalName ; sh:name "Legal" ; sh:order 2 ;
                      dash:propertyRole dash:LabelRole ] ;
        sh:property [ sh:path ex:age ; sh:name "Age" ; sh:order 3 ] .
    "#);
    let data = g(r#"ex:alice a ex:Person ; ex:preferredName "Ally" ; ex:legalName "Alice" ; ex:age 41 ."#);
    let form = derive_form(&data, &shapes, &iri("alice"), &FormOptions::default());

    assert_eq!(form.label.as_deref(), Some("Ally"), "sh:order 1 wins");
    assert_eq!(
        field(&form, "preferredName").property_role.as_deref(),
        Some("http://datashapes.org/dash#LabelRole")
    );
    assert!(field(&form, "age").property_role.is_none());
}

/// No label-role field with a value ⇒ no derived label (the renderer falls back
/// to its own convention rather than being handed a guess).
#[test]
fn label_is_absent_without_a_valued_label_role_field() {
    let shapes = g(r#"
      ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person ;
        sh:property [ sh:path ex:preferredName ; dash:propertyRole dash:LabelRole ] ;
        sh:property [ sh:path ex:homepage ; dash:propertyRole dash:KeyInfoRole ] .
    "#);
    let data = g("ex:alice a ex:Person ; ex:homepage <http://example.org/a> .");
    let form = derive_form(&data, &shapes, &iri("alice"), &FormOptions::default());
    assert!(form.label.is_none());
    assert_eq!(
        field(&form, "homepage").property_role.as_deref(),
        Some("http://datashapes.org/dash#KeyInfoRole"),
        "non-label roles are still carried for the renderer"
    );
    // An IRI-valued label-role field does not supply a display string either.
    let iri_label = g(r#"
      ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person ;
        sh:property [ sh:path ex:homepage ; dash:propertyRole dash:LabelRole ] .
    "#);
    assert!(derive_form(&data, &iri_label, &iri("alice"), &FormOptions::default())
        .label
        .is_none());
}

// ---- (5) dash:abstract ------------------------------------------------------

/// A `dash:abstract` class must not be offered for instantiation: its shape is
/// flagged and sorts after every concrete choice, so it never becomes the
/// default selected shape while a concrete one applies.
#[test]
fn abstract_shapes_are_flagged_and_never_default_selected() {
    let shapes = g(r#"
      ex:Agent dash:abstract true .
      ex:AgentShape a sh:NodeShape ; sh:targetClass ex:Agent ;
        sh:property [ sh:path ex:name ; sh:name "Name" ] .
      ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person ;
        sh:property [ sh:path ex:name ; sh:name "Name" ] .
    "#);
    let data = g("ex:alice a ex:Agent, ex:Person ; ex:name \"Alice\" .");
    let model = ShapesModel::parse(&shapes);

    let choices = applicable_shapes(&data, &shapes, &model, &iri("alice"));
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].shape.value, "http://example.org/PersonShape");
    assert!(!choices[0].is_abstract);
    assert_eq!(choices[1].shape.value, "http://example.org/AgentShape");
    assert!(
        choices[1].is_abstract,
        "dash:abstract on the TARGET CLASS marks the shape abstract"
    );

    let form = derive_form(&data, &shapes, &iri("alice"), &FormOptions::default());
    assert_eq!(
        form.shape.as_ref().map(|s| s.value.as_str()),
        Some("http://example.org/PersonShape")
    );
    // …but an abstract shape stays selectable for viewing existing data.
    let explicit = derive_form(
        &data,
        &shapes,
        &iri("alice"),
        &FormOptions {
            shape: Some(iri("AgentShape")),
            ..FormOptions::default()
        },
    );
    assert_eq!(
        explicit.shape.as_ref().map(|s| s.value.as_str()),
        Some("http://example.org/AgentShape")
    );
}

/// `dash:abstract` declared on the node shape itself (a `sh:ShapeClass`-style
/// shape) marks it just the same.
#[test]
fn abstract_on_the_shape_node_is_honoured() {
    let shapes =
        g("ex:AgentShape a sh:NodeShape ; sh:targetClass ex:Agent ; dash:abstract true .");
    let data = g("ex:alice a ex:Agent .");
    let model = ShapesModel::parse(&shapes);
    let choices = applicable_shapes(&data, &shapes, &model, &iri("alice"));
    assert!(choices[0].is_abstract);
}

// ---- (2) computed fields: the DEFAULT-build contract ------------------------

/// Without the opt-in `computed` feature a `sh:values` field is still FLAGGED
/// computed and read-only, with NO values — it must never fall back to showing
/// asserted data the shape does not describe, and never contributes an edit.
#[test]
fn computed_fields_are_flagged_read_only_and_never_diff() {
    let shapes = g(r#"
      ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person ;
        sh:property [ sh:path ex:age ; sh:name "Age" ] ;
        sh:property [ sh:path ex:friendName ; sh:name "Friend name" ;
                      sh:values [ sh:path ( ex:knows ex:name ) ] ] .
    "#);
    let data = g(r#"
      ex:alice a ex:Person ; ex:age 41 ; ex:knows ex:bob ; ex:friendName "stale" .
      ex:bob ex:name "Bob" .
    "#);
    let form = derive_form(&data, &shapes, &iri("alice"), &FormOptions::default());
    let computed = field(&form, "friendName");
    assert!(computed.computed);
    assert!(
        !computed.editable,
        "computed values are derived, so there is nothing to write back"
    );
    assert!(!field(&form, "age").computed);

    // The two feature states have DISTINCT contracts, so each is asserted
    // exactly. [SONNET-4.6]
    let values: Vec<&str> = computed
        .values
        .iter()
        .map(|v| v.term.value.as_str())
        .collect();
    #[cfg(not(feature = "computed"))]
    assert!(
        values.is_empty(),
        "a default build carries no sh:values evaluator, so the field derives EMPTY: {values:?}"
    );
    #[cfg(feature = "computed")]
    assert_eq!(
        values,
        vec!["Bob"],
        "with `computed` on, the field holds the evaluated ( ex:knows ex:name ) value set"
    );
    assert!(
        !values.contains(&"stale"),
        "a computed field never shows the asserted triple at its path: {values:?}"
    );

    // Even if a renderer echoes edited values back, nothing is written.
    let mut after = form.clone();
    after
        .groups
        .iter_mut()
        .flat_map(|group| &mut group.fields)
        .find(|f| f.computed)
        .unwrap()
        .values
        .clear();
    assert_eq!(to_sparql_update(&form, &after), "");
}
