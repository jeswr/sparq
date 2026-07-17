//! Edge-case floors for form defaults and shape applicability.
//! [GPT-5.6] sq-bif.34

use oxrdf::{NamedNode, Term};
use sparq_core::Graph;
use sparq_forms::{applicable_shapes, derive_form, FormOptions, GroupKind, Mode};
use sparq_shacl::ShapesModel;

const PREFIXES: &str = r#"
  @prefix sh: <http://www.w3.org/ns/shacl#> .
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

#[test]
fn form_options_default_contract() {
    assert_eq!(
        FormOptions::default(),
        FormOptions {
            mode: Mode::Edit,
            role: None,
            shape: None,
            max_depth: 3,
        }
    );
}

#[test]
fn target_class_requires_a_matching_focus_type() {
    let shapes = g("ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person .");
    let model = ShapesModel::parse(&shapes);
    let data = g(r#"
          ex:alice ex:name "Alice" .
          ex:bob a ex:Person .
        "#);

    let choices = applicable_shapes(&data, &shapes, &model, &iri("alice"));
    assert!(choices.is_empty());

    let matching_choices = applicable_shapes(&data, &shapes, &model, &iri("bob"));
    assert_eq!(matching_choices.len(), 1);

    let form = derive_form(&data, &shapes, &iri("alice"), &FormOptions::default());
    assert_eq!(form.focus.kind, "iri");
    assert_eq!(form.focus.value, "http://example.org/alice");
    assert_eq!(form.mode, Mode::Edit);
    assert!(form.role.is_none());
    assert!(form.shapes.is_empty());
    assert!(form.shape.is_none());

    assert_eq!(form.groups.len(), 1);
    let other = &form.groups[0];
    assert_eq!(other.kind, GroupKind::Other);
    assert!(other.group.is_none());
    assert_eq!(other.label.as_deref(), Some("Other properties"));
    assert!(other.order.is_none());
    assert_eq!(other.fields.len(), 1);
    assert_eq!(other.fields[0].path, "<http://example.org/name>");
}
