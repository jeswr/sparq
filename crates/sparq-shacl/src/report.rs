//! Validation reports: the result structs, the report RDF graph (as Turtle, per
//! the SHACL results vocabulary) and a human-readable text rendering.

use crate::model::SH;
use crate::path::Path;
use oxrdf::Term;
use std::fmt::Write as _;

/// One validation result (one constraint violation/warning/info).
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// The node that was being validated.
    pub focus_node: Term,
    /// The property path of the source property shape (sh:resultPath), or the
    /// offending predicate for sh:closed.
    pub path: Option<Path>,
    /// The offending value node, when the component reports one.
    pub value: Option<Term>,
    /// The shape the failing constraint is declared on.
    pub source_shape: Term,
    /// The constraint-component IRI (e.g. `sh:MinCountConstraintComponent`).
    pub source_component: String,
    /// Severity IRI (sh:Violation / sh:Warning / sh:Info, or custom).
    pub severity: String,
    /// `sh:message`s of the source shape (literals, possibly language-tagged).
    pub messages: Vec<Term>,
    /// A generated message used when the shape declares none.
    pub default_message: String,
}

/// The outcome of validating a data graph against a shapes graph.
#[derive(Debug)]
pub struct ValidationReport {
    /// True iff there are no validation results.
    pub conforms: bool,
    pub results: Vec<ValidationResult>,
}

impl ValidationReport {
    pub(crate) fn new(results: Vec<ValidationResult>) -> Self {
        ValidationReport {
            conforms: results.is_empty(),
            results,
        }
    }

    /// The report as an RDF graph serialised to Turtle, using the SHACL
    /// validation-report vocabulary.
    pub fn to_turtle(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "@prefix sh: <{SH}> .\n");
        let _ = writeln!(out, "[] a sh:ValidationReport ;");
        let _ = write!(out, "  sh:conforms {}", self.conforms);
        for r in &self.results {
            let _ = writeln!(out, " ;");
            let _ = writeln!(out, "  sh:result [");
            let _ = writeln!(out, "    a sh:ValidationResult ;");
            let _ = writeln!(out, "    sh:focusNode {} ;", r.focus_node);
            if let Some(p) = &r.path {
                let _ = writeln!(out, "    sh:resultPath {} ;", p.to_turtle());
            }
            if let Some(v) = &r.value {
                let _ = writeln!(out, "    sh:value {v} ;");
            }
            // A blank-node source shape has no graph-independent identity; emit
            // its label so the report stays self-consistent and parseable.
            let _ = writeln!(out, "    sh:sourceShape {} ;", r.source_shape);
            for m in r.effective_messages() {
                let _ = writeln!(out, "    sh:resultMessage {m} ;");
            }
            let _ = writeln!(out, "    sh:resultSeverity <{}> ;", r.severity);
            let _ = write!(
                out,
                "    sh:sourceConstraintComponent <{}> ]",
                r.source_component
            );
        }
        let _ = writeln!(out, " .");
        out
    }

    /// A human-readable rendering of the report.
    pub fn to_text(&self) -> String {
        if self.conforms {
            return "Conforms: data graph satisfies all shapes.\n".into();
        }
        let mut out = format!("Does not conform: {} result(s)\n", self.results.len());
        for r in &self.results {
            let sev = r.severity.rsplit(['#', '/']).next().unwrap_or(&r.severity);
            let comp = r
                .source_component
                .rsplit(['#', '/'])
                .next()
                .unwrap_or(&r.source_component);
            let _ = write!(out, "- [{sev}] focus {}", r.focus_node);
            if let Some(p) = &r.path {
                let _ = write!(out, " | path {}", p.to_turtle());
            }
            if let Some(v) = &r.value {
                let _ = write!(out, " | value {v}");
            }
            let msg = match r.messages.first() {
                Some(Term::Literal(l)) => l.value().to_string(),
                _ => r.default_message.clone(),
            };
            let _ = writeln!(out, "\n    {comp}: {msg}");
        }
        out
    }
}

impl ValidationResult {
    /// The result messages: the source shape's sh:message literals, or the
    /// generated default when the shape declares none.
    pub fn effective_messages(&self) -> Vec<Term> {
        if self.messages.is_empty() {
            vec![Term::Literal(oxrdf::Literal::new_simple_literal(
                &self.default_message,
            ))]
        } else {
            self.messages.clone()
        }
    }
}
