//! Forms F5 — `rdf:langString` + RDF 1.2 base direction in the lang widgets.
//! [OPUS-5] sq-lsp7k.1.5

use oxrdf::{NamedNode, Term};
use sparq_core::Graph;
use sparq_engine::update;
use sparq_forms::{derive_form, to_sparql_update, FormDescription, FormOptions, TermRef, DASH};

const PREFIXES: &str = r#"
  @prefix sh: <http://www.w3.org/ns/shacl#> .
  @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
  @prefix ex: <http://example.org/> .
"#;

const SHAPES: &str = r#"
  ex:PersonShape a sh:NodeShape ; sh:targetClass ex:Person ;
    sh:property [ sh:path ex:label ; sh:name "Label" ] ;
    sh:property [ sh:path ex:bio ; sh:name "Bio" ; sh:datatype rdf:dirLangString ] .
"#;

fn g(ttl: &str) -> Graph {
    Graph::load_str(&format!("{PREFIXES}{ttl}"), "turtle").unwrap()
}

fn alice() -> Term {
    Term::from(NamedNode::new_unchecked("http://example.org/alice"))
}

fn dash(name: &str) -> String {
    format!("{DASH}{name}")
}

/// A directional language-tagged value carries its base direction and selects
/// the SAME lang widgets a plain `rdf:langString` does — the plain text
/// widgets must decline it just as firmly.
#[test]
fn directional_values_select_the_lang_widgets_and_carry_the_direction() {
    let data = g(r#"ex:alice a ex:Person ; ex:label "مرحبا"@ar--rtl ."#);
    let form = derive_form(&data, &g(SHAPES), &alice(), &FormOptions::default());
    let label = form
        .groups
        .iter()
        .flat_map(|group| &group.fields)
        .find(|f| f.path == "<http://example.org/label>")
        .unwrap();

    let term = &label.values[0].term;
    assert_eq!(term.value, "مرحبا");
    assert_eq!(term.language.as_deref(), Some("ar"));
    assert_eq!(term.direction.as_deref(), Some("rtl"));
    assert_eq!(
        term.datatype.as_deref(),
        Some("http://www.w3.org/1999/02/22-rdf-syntax-ns#dirLangString")
    );

    assert_eq!(
        label.widget.editor.as_deref(),
        Some(&*dash("TextFieldWithLangEditor")),
        "score 11 — same as a plain rdf:langString value"
    );
    assert_eq!(
        label.widget.viewer.as_deref(),
        Some(&*dash("LangStringViewer"))
    );
    assert!(
        !label
            .widget
            .editor_alternatives
            .contains(&dash("TextFieldEditor")),
        "the plain text field is unsuitable for a language-tagged value"
    );
}

/// `sh:datatype rdf:dirLangString` on an EMPTY field steers widget scoring the
/// same way `rdf:langString` does (score 5 — permitted, not `singleLine false`).
#[test]
fn dir_lang_string_datatype_constraint_steers_an_empty_field() {
    let data = g("ex:alice a ex:Person .");
    let form = derive_form(&data, &g(SHAPES), &alice(), &FormOptions::default());
    let bio = form
        .groups
        .iter()
        .flat_map(|group| &group.fields)
        .find(|f| f.path == "<http://example.org/bio>")
        .unwrap();
    assert!(bio.values.is_empty());
    let lang_editors = [
        dash("TextFieldWithLangEditor"),
        dash("TextAreaWithLangEditor"),
    ];
    let offered: Vec<&String> = bio
        .widget
        .editor
        .iter()
        .chain(bio.widget.editor_alternatives.iter())
        .collect();
    for editor in &lang_editors {
        assert!(
            offered.contains(&editor),
            "{editor} must be offered for an rdf:dirLangString field, got {offered:?}"
        );
    }
    assert!(
        !offered.contains(&&dash("TextFieldEditor")),
        "a language-tagged-only field never offers the plain text field: {offered:?}"
    );
}

/// The write path renders `"…"@lang--dir`: dropping the direction would
/// silently rewrite the term into a different one.
#[test]
fn the_update_path_preserves_the_base_direction() {
    let data = g(r#"ex:alice a ex:Person ; ex:label "مرحبا"@ar--rtl ."#);
    let shapes = g(SHAPES);
    let before = derive_form(&data, &shapes, &alice(), &FormOptions::default());
    let mut after = before.clone();
    let label = after
        .groups
        .iter_mut()
        .flat_map(|group| &mut group.fields)
        .find(|f| f.path == "<http://example.org/label>")
        .unwrap();
    label.values[0].term.value = "أهلا".into();

    let text = to_sparql_update(&before, &after);
    assert!(text.contains("@ar--rtl"), "direction preserved: {text}");

    let updated = update(&data, &text).unwrap();
    let derived = derive_form(&updated, &shapes, &alice(), &FormOptions::default());
    assert_eq!(derived, after, "the edit round-trips as a directional literal");
}

/// `TermRef` is public and `Deserialize`, so a renderer can hand back a term
/// whose language tag / base direction never came from the parser. Neither has
/// an escape form in N-Triples, so an unconstrained value would be spliced into
/// the update as executable syntax. The write path fails CLOSED instead: an
/// invalid tag or direction yields NO update at all.
#[test]
fn deserialized_term_metadata_cannot_inject_update_syntax() {
    let data = g(r#"ex:alice a ex:Person ; ex:label "مرحبا"@ar--rtl ."#);
    let shapes = g(SHAPES);
    let before = derive_form(&data, &shapes, &alice(), &FormOptions::default());

    // Positive control: the SAME edit with well-formed metadata does produce an
    // update, so an empty result below is the validation and not an inert form.
    assert!(
        to_sparql_update(&before, &edited(&before, r#"{
          "kind": "literal", "value": "أهلا", "language": "ar", "direction": "rtl"
        }"#))
        .contains("@ar--rtl"),
        "a well-formed directional literal still writes back"
    );

    let payloads = [
        // Punctuation smuggled through the base direction …
        r#"{
          "kind": "literal", "value": "pwned", "language": "en",
          "direction": "ltr . } ; DROP ALL ; INSERT { <http://example.org/x> <http://example.org/y> \"z\""
        }"#,
        // … and through the language tag.
        r#"{
          "kind": "literal", "value": "pwned",
          "language": "en . } ; DROP ALL ; INSERT { <http://example.org/x> <http://example.org/y> \"z\""
        }"#,
        // A direction that is neither `ltr` nor `rtl` is refused outright.
        r#"{"kind": "literal", "value": "pwned", "language": "en", "direction": "auto"}"#,
        // A base direction with no language tag names no RDF 1.2 term.
        r#"{"kind": "literal", "value": "pwned", "direction": "ltr"}"#,
    ];
    for payload in payloads {
        let text = to_sparql_update(&before, &edited(&before, payload));
        assert_eq!(
            text, "",
            "no update may be emitted for term metadata {payload}: got {text}"
        );
    }
}

/// `before` with the label field's single value replaced by a `TermRef`
/// deserialized from `json` — i.e. exactly what an untrusted renderer round-trip
/// can hand back.
fn edited(before: &FormDescription, json: &str) -> FormDescription {
    let mut after = before.clone();
    let label = after
        .groups
        .iter_mut()
        .flat_map(|group| &mut group.fields)
        .find(|f| f.path == "<http://example.org/label>")
        .unwrap();
    label.values[0].term = serde_json::from_str::<TermRef>(json).unwrap();
    after
}

/// An identical form is a byte-level no-op: a directional literal must not
/// round-trip into a DIFFERENT term (which a dropped `--dir` would cause).
#[test]
fn identity_diff_over_a_directional_literal_is_a_noop() {
    let data = g(r#"ex:alice a ex:Person ; ex:label "مرحبا"@ar--rtl ."#);
    let form = derive_form(&data, &g(SHAPES), &alice(), &FormOptions::default());
    assert_eq!(to_sparql_update(&form, &form), "");
}
