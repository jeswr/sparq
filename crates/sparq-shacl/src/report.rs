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
    /// [OPUS-4.8] (sq-f8gu) Nested `sh:detail` sub-results explaining WHY this
    /// result fired — currently the per-member (`sh:memberShape`) / per-duplicate
    /// (`sh:uniqueMembers`) results produced by validating each offending list
    /// member against the member shape. Empty for every other component. The
    /// SHACL spec keeps `sh:detail` non-normative (the W3C suite compares only
    /// top-level result fields), so these are informational and never affect
    /// `sh:conforms`.
    pub details: Vec<ValidationResult>,
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

    /// Severity-aware conformance: `true` iff no result carries
    /// `sh:Violation` severity — i.e. `sh:Warning` / `sh:Info` (and custom
    /// severities) are reported but not conformance-breaking. The spec's
    /// `sh:conforms` ([`conforms`](Self::conforms), what the W3C suite
    /// checks) counts EVERY result regardless of severity; this is the
    /// "violations only" toggle several implementations expose for CI-style
    /// gating.
    pub fn conforms_violations_only(&self) -> bool {
        self.results_with_severity(&format!("{SH}Violation"))
            .next()
            .is_none()
    }

    /// The results whose `sh:resultSeverity` is exactly `severity` (a full
    /// IRI, e.g. `http://www.w3.org/ns/shacl#Warning` or a custom severity).
    pub fn results_with_severity<'a>(
        &'a self,
        severity: &'a str,
    ) -> impl Iterator<Item = &'a ValidationResult> + 'a {
        self.results.iter().filter(move |r| r.severity == severity)
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
            let _ = write!(out, "  sh:result ");
            write_result_node(&mut out, r);
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
            // [OPUS-4.8] (sq-f8gu) Render nested sh:detail sub-results, indented.
            for d in &r.details {
                let dcomp = d
                    .source_component
                    .rsplit(['#', '/'])
                    .next()
                    .unwrap_or(&d.source_component);
                let _ = write!(out, "      detail: {dcomp}");
                if let Some(v) = &d.value {
                    let _ = write!(out, " | value {v}");
                }
                let _ = writeln!(out);
            }
        }
        out
    }
}

/// [OPUS-4.8] (sq-f8gu) Writes one `sh:ValidationResult` as an anonymous
/// blank-node block, recursively nesting `sh:detail` sub-results. Emitted as a
/// continuation of an already-written predicate (`sh:result ` / `sh:detail `),
/// and closed with `]` (no trailing `.` — the caller owns statement framing).
fn write_result_node(out: &mut String, r: &ValidationResult) {
    let _ = writeln!(out, "[");
    let _ = writeln!(out, "    a sh:ValidationResult ;");
    let _ = writeln!(out, "    sh:focusNode {} ;", r.focus_node);
    if let Some(p) = &r.path {
        let _ = writeln!(out, "    sh:resultPath {} ;", p.to_turtle());
    }
    if let Some(v) = &r.value {
        let _ = writeln!(out, "    sh:value {v} ;");
    }
    // A blank-node source shape has no graph-independent identity; emit its
    // label so the report stays self-consistent and parseable.
    let _ = writeln!(out, "    sh:sourceShape {} ;", r.source_shape);
    for m in r.effective_messages() {
        let _ = writeln!(out, "    sh:resultMessage {m} ;");
    }
    let _ = writeln!(out, "    sh:resultSeverity <{}> ;", r.severity);
    for d in &r.details {
        let _ = write!(out, "    sh:detail ");
        write_result_node(out, d);
        let _ = writeln!(out, " ;");
    }
    let _ = write!(
        out,
        "    sh:sourceConstraintComponent <{}> ]",
        r.source_component
    );
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

// [OPUS-4.8] Report-serialisation coverage (sq-qap0). report.rs (~64% covered)
// has branches the validate()-level tests don't reliably hit in isolation: the
// path/value-present vs -absent arms of to_turtle / to_text, the
// conforming-vs-not text rendering, the default-vs-declared message path, and
// the severity helpers. Constructing ValidationResults directly (all fields are
// pub) exercises each arm with a hand-built report.
#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::{Literal, NamedNode, Term};

    const EX: &str = "http://example.org/";

    fn iri(s: &str) -> Term {
        Term::NamedNode(NamedNode::new_unchecked(s.to_string()))
    }

    fn lit(s: &str) -> Term {
        Term::Literal(Literal::new_simple_literal(s))
    }

    /// A violation result with controllable path / value / messages.
    fn result(
        focus: Term,
        path: Option<Path>,
        value: Option<Term>,
        component: &str,
        severity: &str,
        messages: Vec<Term>,
    ) -> ValidationResult {
        ValidationResult {
            focus_node: focus,
            path,
            value,
            source_shape: iri(&format!("{EX}Shape")),
            source_component: format!("{SH}{component}"),
            severity: format!("{SH}{severity}"),
            messages,
            default_message: "generated default".into(),
            details: Vec::new(),
        }
    }

    /// Parses the (Turtle) report back to triples — both validates the syntax
    /// and lets a test assert on emitted predicates/objects.
    fn parse_report(ttl: &str) -> Vec<oxrdf::Triple> {
        oxttl::TurtleParser::new()
            .for_slice(ttl.as_bytes())
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_else(|e| panic!("report Turtle does not parse: {e}\n{ttl}"))
    }

    #[test]
    fn conforming_report_turtle_and_text() {
        let r = ValidationReport::new(vec![]);
        assert!(r.conforms);
        let ttl = r.to_turtle();
        assert!(ttl.contains("sh:conforms true"));
        // No sh:result blocks when conforming.
        assert!(!ttl.contains("sh:result"));
        parse_report(&ttl);
        assert_eq!(r.to_text(), "Conforms: data graph satisfies all shapes.\n");
    }

    #[test]
    fn turtle_emits_path_value_and_message_branches() {
        // A property-shape result WITH a path, value and a declared message.
        let r = ValidationReport::new(vec![result(
            iri(&format!("{EX}alice")),
            Some(Path::Predicate(format!("{EX}age"))),
            Some(lit("thirty")),
            "DatatypeConstraintComponent",
            "Violation",
            vec![lit("bad age")],
        )]);
        assert!(!r.conforms);
        let ttl = r.to_turtle();
        assert!(ttl.contains("sh:conforms false"));
        assert!(ttl.contains("sh:resultPath"), "path branch: {ttl}");
        assert!(ttl.contains("sh:value"), "value branch: {ttl}");
        assert!(
            ttl.contains(r#"sh:resultMessage "bad age""#),
            "declared message: {ttl}"
        );
        assert!(ttl.contains("DatatypeConstraintComponent"));
        let triples = parse_report(&ttl);
        assert!(triples
            .iter()
            .any(|t| t.predicate.as_str().ends_with("resultPath")));
        assert!(triples
            .iter()
            .any(|t| t.predicate.as_str().ends_with("value")));
    }

    #[test]
    fn turtle_omits_absent_path_and_value() {
        // A node-shape / count result: NO path, NO value, NO declared message —
        // the default message is emitted instead.
        let r = ValidationReport::new(vec![result(
            iri(&format!("{EX}bob")),
            None,
            None,
            "MinCountConstraintComponent",
            "Violation",
            vec![],
        )]);
        let ttl = r.to_turtle();
        assert!(!ttl.contains("sh:resultPath"), "no path expected: {ttl}");
        assert!(!ttl.contains("sh:value"), "no value expected: {ttl}");
        // effective_messages falls back to the default message.
        assert!(
            ttl.contains(r#"sh:resultMessage "generated default""#),
            "{ttl}"
        );
        parse_report(&ttl);
    }

    #[test]
    fn turtle_handles_multiple_results() {
        let r = ValidationReport::new(vec![
            result(
                iri(&format!("{EX}a")),
                None,
                None,
                "MinCountConstraintComponent",
                "Violation",
                vec![],
            ),
            result(
                iri(&format!("{EX}b")),
                Some(Path::Predicate(format!("{EX}p"))),
                Some(iri(&format!("{EX}v"))),
                "ClassConstraintComponent",
                "Warning",
                vec![],
            ),
        ]);
        let ttl = r.to_turtle();
        let triples = parse_report(&ttl);
        // Two sh:result links from the report node.
        let n_results = triples
            .iter()
            .filter(|t| t.predicate.as_str().ends_with("#result"))
            .count();
        assert_eq!(n_results, 2, "{ttl}");
    }

    #[test]
    fn text_rendering_shortens_iris_and_uses_messages() {
        let r = ValidationReport::new(vec![
            result(
                iri(&format!("{EX}alice")),
                Some(Path::Predicate(format!("{EX}age"))),
                Some(lit("thirty")),
                "DatatypeConstraintComponent",
                "Violation",
                vec![lit("custom msg")],
            ),
            result(
                iri(&format!("{EX}bob")),
                None,
                None,
                "MinCountConstraintComponent",
                "Warning",
                vec![],
            ),
        ]);
        let text = r.to_text();
        assert!(text.starts_with("Does not conform: 2 result(s)"), "{text}");
        // Severity + component are rendered as their local names.
        assert!(text.contains("[Violation]"), "{text}");
        assert!(text.contains("[Warning]"), "{text}");
        assert!(text.contains("DatatypeConstraintComponent:"), "{text}");
        assert!(text.contains("MinCountConstraintComponent:"), "{text}");
        // path + value rendered when present.
        assert!(text.contains("| path "), "{text}");
        assert!(text.contains("| value "), "{text}");
        // Declared message wins for the first; default for the second.
        assert!(text.contains("custom msg"), "{text}");
        assert!(text.contains("generated default"), "{text}");
    }

    #[test]
    fn effective_messages_default_and_declared() {
        let no_msg = result(
            iri(&format!("{EX}x")),
            None,
            None,
            "MinCountConstraintComponent",
            "Violation",
            vec![],
        );
        let m = no_msg.effective_messages();
        assert_eq!(m.len(), 1);
        assert!(matches!(&m[0], Term::Literal(l) if l.value() == "generated default"));

        let with_msg = result(
            iri(&format!("{EX}x")),
            None,
            None,
            "MinCountConstraintComponent",
            "Violation",
            vec![lit("m1"), lit("m2")],
        );
        let m = with_msg.effective_messages();
        assert_eq!(m.len(), 2);
        assert_eq!(m, vec![lit("m1"), lit("m2")]);
    }

    // [OPUS-4.8] (sq-f8gu) A result carrying sh:detail sub-results nests each as a
    // sh:ValidationResult blank node under sh:detail; both Turtle and text render
    // them, and the Turtle round-trips.
    #[test]
    fn turtle_and_text_nest_detail_sub_results() {
        let mut top = result(
            iri(&format!("{EX}list")),
            None,
            Some(iri(&format!("{EX}list"))),
            "MemberShapeConstraintComponent",
            "Violation",
            vec![],
        );
        top.details = vec![
            result(
                iri(&format!("{EX}list")),
                None,
                Some(lit("bad1")),
                "NodeKindConstraintComponent",
                "Violation",
                vec![],
            ),
            result(
                iri(&format!("{EX}list")),
                None,
                Some(lit("bad2")),
                "NodeKindConstraintComponent",
                "Violation",
                vec![],
            ),
        ];
        let r = ValidationReport::new(vec![top]);
        let ttl = r.to_turtle();
        assert!(ttl.contains("sh:detail"), "{ttl}");
        let triples = parse_report(&ttl);
        // Two sh:detail links, and three sh:ValidationResult nodes (top + 2 details).
        assert_eq!(
            triples
                .iter()
                .filter(|t| t.predicate.as_str().ends_with("#detail"))
                .count(),
            2,
            "{ttl}"
        );
        assert_eq!(
            triples
                .iter()
                .filter(|t| t.predicate.as_str().ends_with("#type")
                    && t.object.to_string().ends_with("ValidationResult>"))
                .count(),
            3,
            "{ttl}"
        );
        // The text rendering surfaces the nested details.
        let text = r.to_text();
        assert_eq!(
            text.matches("detail: NodeKindConstraintComponent").count(),
            2,
            "{text}"
        );
    }

    #[test]
    fn severity_helpers() {
        let r = ValidationReport::new(vec![
            result(
                iri(&format!("{EX}a")),
                None,
                None,
                "MinCountConstraintComponent",
                "Warning",
                vec![],
            ),
            result(
                iri(&format!("{EX}b")),
                None,
                None,
                "MaxCountConstraintComponent",
                "Info",
                vec![],
            ),
        ]);
        // sh:conforms counts every result; the violations-only toggle passes
        // because neither result is a sh:Violation.
        assert!(!r.conforms);
        assert!(r.conforms_violations_only());
        assert_eq!(r.results_with_severity(&format!("{SH}Warning")).count(), 1);
        assert_eq!(r.results_with_severity(&format!("{SH}Info")).count(), 1);
        assert_eq!(
            r.results_with_severity(&format!("{SH}Violation")).count(),
            0
        );

        // Add a Violation: the toggle now fails too.
        let r = ValidationReport::new(vec![result(
            iri(&format!("{EX}c")),
            None,
            None,
            "NodeConstraintComponent",
            "Violation",
            vec![],
        )]);
        assert!(!r.conforms_violations_only());
    }
}
