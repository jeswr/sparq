//! [OPUS-4.8] sq-yqi1 (#162): the `Store::validate(data, shapes)` SHACL binding.
//!
//! Exposes `sparq-shacl`'s existing public validation API
//! ([`sparq_shacl::validate`] — SHACL Core + SHACL-SPARQL §5.2) to JS/WASM
//! consumers as a drop-in for `rdf-validate-shacl` (PSS's ADR-0014
//! `ShaclValidator` seam). It does NOT reimplement validation — it loads the two
//! graphs exactly as [`Store::load`] does and calls straight through to the
//! engine, then hand-serialises the [`sparq_shacl::ValidationReport`] to a small
//! JSON string (the bundle has no serde, mirroring the SPARQL-JSON path).
//!
//! This module compiles ONLY under the opt-in `shacl` feature, so the default
//! browser bundle carries zero SHACL code.
//!
//! ## Report shape (the JSON returned by `validate`)
//!
//! ```json
//! {
//!   "conforms": false,
//!   "results": [
//!     {
//!       "focusNode": "http://example.org/bob",
//!       "path": "<http://example.org/age>",
//!       "value": "\"-1\"^^<http://www.w3.org/2001/XMLSchema#integer>",
//!       "sourceShape": "_:b0",
//!       "sourceConstraintComponent": "http://www.w3.org/ns/shacl#MinInclusiveConstraintComponent",
//!       "severity": "http://www.w3.org/ns/shacl#Violation",
//!       "message": "Value is not >= 0"
//!     }
//!   ]
//! }
//! ```
//!
//! `path`, `value` and `message` are `null` when the result carries none.
//! `focusNode`/`value`/`sourceShape` are N-Triples term strings (the same lexical
//! form `Term`'s `Display` produces); `path` is a SHACL Turtle path expression.
//! `message` is the plain text of the shape's first `sh:message` (or the
//! generated default), suitable for surfacing directly in a UI.

use oxrdf::Term;
use sparq_core::Graph;
use sparq_shacl::{ValidationReport, ValidationResult};
use wasm_bindgen::prelude::*;

use crate::Store;

/// Appends `s` to `out` as a JSON string literal (quotes + minimal escaping).
/// The bundle carries no serde, so — exactly like the SPARQL-JSON serialiser —
/// strings are escaped by hand. Escapes the two mandatory characters (`"` and
/// `\`) plus the C0 control characters JSON forbids unescaped.
fn push_json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Appends `"key":<json-string>` (or `"key":null` when `val` is `None`).
fn push_field(out: &mut String, key: &str, val: Option<&str>) {
    out.push('"');
    out.push_str(key);
    out.push_str("\":");
    match val {
        Some(v) => push_json_string(out, v),
        None => out.push_str("null"),
    }
}

/// The plain text of a result's first `sh:message`, or its generated default —
/// the human-readable string a UI surfaces (mirrors `to_text`'s message choice).
fn result_message(r: &ValidationResult) -> String {
    match r.messages.first() {
        Some(Term::Literal(l)) => l.value().to_string(),
        _ => r.default_message.clone(),
    }
}

/// Serialises a [`ValidationReport`] to the JS-facing JSON described in the
/// module docs. Kept here (not in `sparq-shacl`) so the report struct stays
/// serde-free and the wasm bundle keeps its hand-rolled-JSON convention.
fn report_to_json(report: &ValidationReport) -> String {
    let mut out = String::from("{\"conforms\":");
    out.push_str(if report.conforms { "true" } else { "false" });
    out.push_str(",\"results\":[");
    for (i, r) in report.results.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('{');
        push_field(&mut out, "focusNode", Some(&r.focus_node.to_string()));
        out.push(',');
        let path = r.path.as_ref().map(|p| p.to_turtle());
        push_field(&mut out, "path", path.as_deref());
        out.push(',');
        let value = r.value.as_ref().map(|v| v.to_string());
        push_field(&mut out, "value", value.as_deref());
        out.push(',');
        push_field(&mut out, "sourceShape", Some(&r.source_shape.to_string()));
        out.push(',');
        push_field(
            &mut out,
            "sourceConstraintComponent",
            Some(&r.source_component),
        );
        out.push(',');
        push_field(&mut out, "severity", Some(&r.severity));
        out.push(',');
        push_field(&mut out, "message", Some(&result_message(r)));
        out.push('}');
    }
    out.push_str("]}");
    out
}

#[wasm_bindgen]
impl Store {
    /// [OPUS-4.8] sq-yqi1 (#162): validates an RDF **data graph** against a SHACL
    /// **shapes graph**, returning a SHACL validation report as a JSON string.
    ///
    /// Both arguments are RDF documents in the same syntaxes [`Store::load`]
    /// accepts (`"turtle"` | `"ntriples"` | `"nquads"` | `"trig"`); they are
    /// parsed identically (named graphs folded into the default graph). This is a
    /// stateless one-shot — it does not consult the receiver's stored triples —
    /// so it is the drop-in replacement for `rdf-validate-shacl`'s
    /// `validate(dataDataset, { shapes })`: validation runs through
    /// `sparq-shacl`'s SHACL Core + SHACL-SPARQL (`sh:sparql`, §5.2) engine.
    ///
    /// Returns a JSON object `{ conforms: boolean, results: [...] }`; each result
    /// has `focusNode`, `path`, `value`, `sourceShape`,
    /// `sourceConstraintComponent`, `severity` and `message` (see the module
    /// docs for the exact shape). `JSON.parse` it on the JS side. `sh:conforms`
    /// counts EVERY result regardless of severity (the W3C-suite notion); filter
    /// `results` by `severity` for a violations-only gate.
    ///
    /// Errors only if a graph fails to parse (a `JsError` carrying the parse
    /// error) — malformed shapes are skipped by the engine, never surfaced as an
    /// error. Small-document write-validation (~10–100 triples) sits far below
    /// the wasm linear-memory ceiling; very large data graphs should use the
    /// server-side HTTP `validate` path instead (#162 path (c)).
    ///
    /// The `data`/`shapes` arguments take ownership of two parameters; both
    /// graphs are dropped when the call returns.
    pub fn validate(&self, data: &str, shapes: &str, format: &str) -> Result<String, JsError> {
        let data_graph = Graph::load_str(data, format).map_err(|e| JsError::new(&e))?;
        let shapes_graph = Graph::load_str(shapes, format).map_err(|e| JsError::new(&e))?;
        let report = sparq_shacl::validate(&data_graph, &shapes_graph);
        Ok(report_to_json(&report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHAPES: &str = r#"
        @prefix sh: <http://www.w3.org/ns/shacl#> .
        @prefix ex: <http://example.org/> .
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
        ex:PersonShape a sh:NodeShape ;
          sh:targetClass ex:Person ;
          sh:property [
            sh:path ex:age ;
            sh:datatype xsd:integer ;
            sh:minInclusive 0 ;
            sh:message "age must be a non-negative integer" ;
          ] ;
          sh:property [
            sh:path ex:name ;
            sh:minCount 1 ;
          ] .
    "#;

    /// The serialiser used by the wasm `validate` export runs natively, so the
    /// report-shape contract is tested here without a wasm runtime (the
    /// `JsError`-returning wasm wrapper itself cannot run off-wasm — `JsError::new`
    /// is a wasm-bindgen import that panics natively — so, exactly as the other
    /// wasm tests do, we exercise the engine call + the JSON serialiser it feeds).
    fn validate_json(data: &str) -> String {
        let data = Graph::load_str(data, "turtle").unwrap();
        let shapes = Graph::load_str(SHAPES, "turtle").unwrap();
        let report = sparq_shacl::validate(&data, &shapes);
        report_to_json(&report)
    }

    /// Conforming data: the report says `conforms: true` with an empty results array.
    #[test]
    fn conforming_report_json() {
        let json = validate_json(
            r#"
            @prefix ex: <http://example.org/> .
            ex:alice a ex:Person ; ex:age 30 ; ex:name "Alice" .
        "#,
        );
        assert_eq!(json, r#"{"conforms":true,"results":[]}"#, "{json}");
    }

    /// Violating data: a constraint violation surfaces with focusNode / path /
    /// severity / message — the fields PSS's `ShaclValidator` seam consumes.
    #[test]
    fn violating_report_json_has_focus_path_message() {
        let json = validate_json(
            r#"
            @prefix ex: <http://example.org/> .
            ex:bob a ex:Person ; ex:age -1 .
        "#,
        );
        // Not conforming, and at least the minInclusive (negative age) + minCount
        // (missing name) violations are present.
        assert!(
            json.starts_with(r#"{"conforms":false,"results":[{"#),
            "{json}"
        );
        // The focus node is the offending instance, as an N-Triples IRI term.
        assert!(
            json.contains(r#""focusNode":"<http://example.org/bob>""#),
            "{json}"
        );
        // The age property shape's path + declared message surface.
        assert!(
            json.contains(r#""path":"<http://example.org/age>""#),
            "path present: {json}"
        );
        assert!(
            json.contains(r#""message":"age must be a non-negative integer""#),
            "declared message present: {json}"
        );
        // The offending value is reported as an N-Triples literal term.
        assert!(
            json.contains(r#""value":"\"-1\"^^<http://www.w3.org/2001/XMLSchema#integer>""#),
            "value present: {json}"
        );
        // Severity defaults to sh:Violation, and the constraint component is named.
        assert!(
            json.contains(r#""severity":"http://www.w3.org/ns/shacl#Violation""#),
            "{json}"
        );
        assert!(
            json.contains("MinInclusiveConstraintComponent"),
            "minInclusive component present: {json}"
        );
        assert!(
            json.contains("MinCountConstraintComponent"),
            "minCount component present: {json}"
        );
    }

    /// A result with no path / value (a node-shape minCount) emits explicit
    /// `null`s, and the generated default message is used when none is declared.
    #[test]
    fn null_path_and_value_and_default_message() {
        let shapes = Graph::load_str(
            r#"
            @prefix sh: <http://www.w3.org/ns/shacl#> .
            @prefix ex: <http://example.org/> .
            ex:S a sh:NodeShape ;
              sh:targetNode ex:x ;
              sh:property [ sh:path ex:p ; sh:minCount 1 ] .
        "#,
            "turtle",
        )
        .unwrap();
        // ex:x has no ex:p => minCount violation with no value node.
        let data = Graph::load_str(
            "@prefix ex: <http://example.org/> . ex:x a ex:Thing .",
            "turtle",
        )
        .unwrap();
        let report = sparq_shacl::validate(&data, &shapes);
        let json = report_to_json(&report);
        assert!(json.contains("\"conforms\":false"), "{json}");
        // minCount on a property shape: path IS present (the ex:p path), value is null.
        assert!(json.contains(r#""value":null"#), "value null: {json}");
        // A non-empty default message string (not null) is always emitted.
        assert!(!json.contains(r#""message":null"#), "{json}");
    }

    /// The hand-rolled JSON escaper handles quotes, backslashes and control chars
    /// in a message so the document stays valid JSON.
    #[test]
    fn json_string_escaping() {
        let mut s = String::new();
        push_json_string(&mut s, "a\"b\\c\nd\te");
        assert_eq!(s, r#""a\"b\\c\nd\te""#, "{s}");
        // A C0 control char is \u-escaped (raw control chars are invalid in JSON).
        let mut ctrl = String::new();
        push_json_string(&mut ctrl, "\u{0001}");
        assert_eq!(ctrl, "\"\\u0001\"", "{ctrl}");
    }
}
