//! Pure form-value differencing and SPARQL 1.1 Update rendering.
//! [GPT-5.6] sq-wn788

use crate::{FormDescription, FormField, TermRef};
use serde::{Deserialize, Serialize};

/// One value added to or removed from a bare forward-predicate field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldValueDiff {
    /// The field path, always a bare forward predicate such as `<http://example.org/name>`.
    pub path: String,
    /// The RDF term added or removed at that path.
    pub value: TermRef,
}

/// The editable term-level difference between two descriptions of one focus node.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FormDiff {
    /// Values present only in the edited description.
    pub added: Vec<FieldValueDiff>,
    /// Values present only in the original description.
    pub removed: Vec<FieldValueDiff>,
}

impl FormDiff {
    /// Computes the writable difference between two descriptions.
    ///
    /// A mismatched focus node produces an empty diff. Only fields editable in
    /// both descriptions are compared; a newly introduced editable field may
    /// contribute additions. Inverse, read-only, and complex-path fields are ignored.
    pub fn between(before: &FormDescription, after: &FormDescription) -> Self {
        if before.focus != after.focus {
            return Self::default();
        }

        let mut diff = Self::default();
        for after_field in fields(after).filter(|field| eligible(field)) {
            let Some(predicate) = bare_predicate(&after_field.path) else {
                continue;
            };
            let before_field = fields(before)
                .find(|field| eligible(field) && bare_predicate(&field.path) == Some(predicate));

            for value in &after_field.values {
                let was_present = before_field
                    .is_some_and(|field| field.values.iter().any(|old| old.term == value.term));
                if !was_present {
                    diff.added.push(FieldValueDiff {
                        path: after_field.path.clone(),
                        value: value.term.clone(),
                    });
                }
            }

            if let Some(before_field) = before_field {
                for value in &before_field.values {
                    if !after_field.values.iter().any(|new| new.term == value.term) {
                        diff.removed.push(FieldValueDiff {
                            path: after_field.path.clone(),
                            value: value.term.clone(),
                        });
                    }
                }
            }
        }
        diff
    }
}

/// Builds one SPARQL 1.1 `DELETE`/`INSERT` update for an edited form.
///
/// Returns an empty string when there is no writable change (including when
/// the descriptions name different focus nodes). Callers may treat that as a no-op.
pub fn to_sparql_update(before: &FormDescription, after: &FormDescription) -> String {
    let diff = FormDiff::between(before, after);
    if diff.added.is_empty() && diff.removed.is_empty() {
        return String::new();
    }

    let subject = term_to_ntriples(&after.focus);
    let triples = |changes: &[FieldValueDiff]| {
        changes
            .iter()
            .map(|change| {
                format!(
                    "  {subject} {} {} .\n",
                    change.path,
                    term_to_ntriples(&change.value)
                )
            })
            .collect::<String>()
    };

    format!(
        "DELETE {{\n{}}}\nINSERT {{\n{}}}\nWHERE {{}}",
        triples(&diff.removed),
        triples(&diff.added)
    )
}

fn fields(form: &FormDescription) -> impl Iterator<Item = &FormField> {
    form.groups.iter().flat_map(|group| group.fields.iter())
}

fn eligible(field: &FormField) -> bool {
    field.editable && !field.inverse && bare_predicate(&field.path).is_some()
}

fn bare_predicate(path: &str) -> Option<&str> {
    let iri = path.strip_prefix('<')?.strip_suffix('>')?;
    (!iri.is_empty() && !iri.contains(['<', '>', ' ', '\t', '\r', '\n'])).then_some(iri)
}

fn term_to_ntriples(term: &TermRef) -> String {
    match term.kind.as_str() {
        "iri" => format!("<{}>", escape_iri(&term.value)),
        "bnode" => format!("_:{}", term.value),
        "literal" => {
            let literal = format!("\"{}\"", escape_literal(&term.value));
            if let Some(language) = &term.language {
                format!("{literal}@{language}")
            } else if let Some(datatype) = &term.datatype {
                format!("{literal}^^<{}>", escape_iri(datatype))
            } else {
                literal
            }
        }
        // RDF 1.2 triple terms are already stored as N-Triples text.
        _ => term.value.clone(),
    }
}

fn escape_iri(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            '\\' => "\\\\".to_string(),
            '>' => "\\u003E".to_string(),
            c if c <= '\u{20}' || matches!(c, '<' | '"' | '{' | '}' | '|' | '^' | '`') => {
                unicode_escape(c)
            }
            c => c.to_string(),
        })
        .collect()
}

fn escape_literal(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            '\\' => "\\\\".to_string(),
            '"' => "\\\"".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            '\u{8}' => "\\b".to_string(),
            '\u{c}' => "\\f".to_string(),
            c if c < '\u{20}' => unicode_escape(c),
            c => c.to_string(),
        })
        .collect()
}

fn unicode_escape(c: char) -> String {
    let n = c as u32;
    if n <= 0xffff {
        format!("\\u{n:04X}")
    } else {
        format!("\\U{n:08X}")
    }
}
