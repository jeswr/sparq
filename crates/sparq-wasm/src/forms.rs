//! [GPT-5.6] sq-q4apb: hosted-web `FormDescription` derivation bridge.
//!
//! This opt-in module is a thin, stateless boundary over `sparq_forms::derive_form`:
//! serialized data + shapes graphs and structured term/options JSON go in; the direct
//! serde serialization of the derived `FormDescription` comes out. It never consults
//! the receiver's graph and never reconstructs form keys, widgets, or groups.

use std::str::FromStr;

use oxrdf::{BlankNode, Literal, NamedNode, Term};
use serde_json::Value;
use sparq_core::Graph;
use sparq_forms::{FormOptions, Mode, TermRef};
use wasm_bindgen::prelude::*;

use crate::Store;

/// Converts the renderer-facing `TermRef` representation back into an RDF term.
fn term_from_ref(reference: TermRef, field: &str, node_only: bool) -> Result<Term, String> {
    let term = match reference.kind.as_str() {
        "iri" => NamedNode::new(&reference.value)
            .map(Term::from)
            .map_err(|error| {
                format!("Invalid {field} TermRef IRI {:?}: {error}", reference.value)
            })?,
        "bnode" => BlankNode::new(&reference.value)
            .map(Term::from)
            .map_err(|error| {
                format!(
                    "Invalid {field} TermRef blank-node label {:?}: {error}",
                    reference.value
                )
            })?,
        "literal" => {
            if node_only {
                return Err(format!(
                    "Invalid {field} TermRef kind \"literal\"; an IRI or blank node is required."
                ));
            }
            match (reference.language.as_deref(), reference.datatype.as_deref()) {
                (Some(_), Some(_)) => {
                    return Err(format!(
                        "Invalid {field} literal TermRef; language and datatype are mutually exclusive."
                    ));
                }
                (Some(language), None) => {
                    Literal::new_language_tagged_literal(&reference.value, language)
                        .map(Term::from)
                        .map_err(|error| {
                            format!("Invalid {field} TermRef language tag {language:?}: {error}")
                        })?
                }
                (None, Some(datatype)) => {
                    let datatype = NamedNode::new(datatype).map_err(|error| {
                        format!("Invalid {field} TermRef datatype {datatype:?}: {error}")
                    })?;
                    Term::from(Literal::new_typed_literal(reference.value, datatype))
                }
                (None, None) => Term::from(Literal::new_simple_literal(reference.value)),
            }
        }
        "triple" => {
            if node_only {
                return Err(format!(
                    "Invalid {field} TermRef kind \"triple\"; an IRI or blank node is required."
                ));
            }
            let term = Term::from_str(&reference.value).map_err(|error| {
                format!(
                    "Invalid {field} RDF 1.2 triple-term text {:?}: {error}",
                    reference.value
                )
            })?;
            if !matches!(term, Term::Triple(_)) {
                return Err(format!(
                    "Invalid {field} TermRef kind \"triple\"; value must be RDF 1.2 triple-term text."
                ));
            }
            term
        }
        other => {
            return Err(format!(
                "Invalid {field} TermRef kind {other:?}; expected \"iri\", \"bnode\", \"literal\", or \"triple\"."
            ));
        }
    };
    Ok(term)
}

/// Decodes the public structured JSON input. A raw IRI / `_:label` is also accepted so
/// the already-shipped hosted-workbench scaffold can call this binding without a second
/// frontend compatibility patch; new callers should send the documented TermRef JSON.
fn parse_term_input(input: &str, field: &str, node_only: bool) -> Result<Term, String> {
    let trimmed = input.trim();
    let reference = if trimmed.starts_with('{') || trimmed.starts_with('[') {
        serde_json::from_str::<TermRef>(trimmed)
            .map_err(|error| format!("Invalid {field} TermRef JSON: {error}"))?
    } else if let Some(label) = trimmed.strip_prefix("_:") {
        TermRef {
            kind: "bnode".to_string(),
            value: label.to_string(),
            datatype: None,
            language: None,
        }
    } else {
        TermRef {
            kind: "iri".to_string(),
            value: trimmed.to_string(),
            datatype: None,
            language: None,
        }
    };
    term_from_ref(reference, field, node_only)
}

fn parse_options(options_json: &str) -> Result<FormOptions, String> {
    let value: Value = serde_json::from_str(options_json)
        .map_err(|error| format!("Invalid form options JSON: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "Invalid form options JSON: expected an object.".to_string())?;
    let mode = match object.get("mode").and_then(Value::as_str) {
        Some("edit") => Mode::Edit,
        Some("view") => Mode::View,
        Some(other) => {
            return Err(format!(
                "Unsupported form mode {other:?}; expected \"edit\" or \"view\"."
            ));
        }
        None => {
            return Err(
                "Invalid form options JSON: required string field \"mode\" is missing.".to_string(),
            );
        }
    };

    let shape = match object.get("shape") {
        None | Some(Value::Null) => None,
        Some(Value::String(input)) => Some(parse_term_input(input, "shape", true)?),
        Some(Value::Object(_)) => {
            let reference: TermRef = serde_json::from_value(object["shape"].clone())
                .map_err(|error| format!("Invalid shape TermRef JSON: {error}"))?;
            Some(term_from_ref(reference, "shape", true)?)
        }
        Some(_) => {
            return Err(
                "Invalid shape TermRef JSON: expected an object, JSON string, or null.".to_string(),
            );
        }
    };

    Ok(FormOptions {
        mode,
        shape,
        ..FormOptions::default()
    })
}

/// Shared fallible body used by native tests without constructing a native `JsError`.
pub(crate) fn derive_form_json(
    data: &str,
    shapes: &str,
    focus: &str,
    format: &str,
    options_json: &str,
) -> Result<String, String> {
    let data = Graph::load_dataset(data, format)
        .map_err(|error| format!("Could not parse the form data graph as {format}: {error}"))?;
    let shapes = Graph::load_dataset(shapes, format)
        .map_err(|error| format!("Could not parse the form shapes graph as {format}: {error}"))?;
    let focus = parse_term_input(focus, "focus", false)?;
    let options = parse_options(options_json)?;
    let description = sparq_forms::derive_form(&data, &shapes, &focus, &options);
    serde_json::to_string(&description)
        .map_err(|error| format!("Could not serialize the derived FormDescription: {error}"))
}

#[wasm_bindgen]
impl Store {
    /// Derives a complete `sparq_forms::FormDescription` from serialized data and SHACL
    /// shapes graphs, returning its direct snake_case serde JSON.
    ///
    /// `focus` is a serialized `TermRef`; `format` is the RDF syntax for both graphs; and
    /// `options_json` is `{ "mode": "edit" | "view", "shape"?: TermRef }`. The explicit
    /// shape must be an IRI or blank node. This is stateless: the receiver's graph is ignored.
    /// Parse/term/options failures throw a `JsError` with contextual input-boundary text.
    #[wasm_bindgen(js_name = deriveForm)]
    pub fn derive_form(
        &self,
        data: &str,
        shapes: &str,
        focus: &str,
        format: &str,
        options_json: &str,
    ) -> Result<String, JsError> {
        derive_form_json(data, shapes, focus, format, options_json)
            .map_err(|error| JsError::new(&error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_inputs_fail_with_boundary_context() {
        let good_data = "<http://example.org/alice> <http://example.org/name> \"Alice\" .";
        let good_shapes = "<http://example.org/Shape> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/shacl#NodeShape> .";

        let graph_error = derive_form_json(
            "<http://example.org/alice> <http://example.org/name>",
            good_shapes,
            r#"{"kind":"iri","value":"http://example.org/alice"}"#,
            "ntriples",
            r#"{"mode":"edit"}"#,
        )
        .expect_err("truncated data must fail");
        assert!(graph_error.contains("form data graph"), "{graph_error}");

        let focus_error = derive_form_json(
            good_data,
            good_shapes,
            r#"{"kind":"iri"}"#,
            "ntriples",
            r#"{"mode":"edit"}"#,
        )
        .expect_err("TermRef without value must fail");
        assert!(focus_error.contains("focus TermRef JSON"), "{focus_error}");

        let mode_error = derive_form_json(
            good_data,
            good_shapes,
            r#"{"kind":"iri","value":"http://example.org/alice"}"#,
            "ntriples",
            r#"{"mode":"compose"}"#,
        )
        .expect_err("unknown form mode must fail");
        assert!(mode_error.contains("Unsupported form mode"), "{mode_error}");
    }
}
