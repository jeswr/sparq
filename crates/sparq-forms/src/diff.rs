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
    /// The subject the change applies to when it is NOT the focus node: the
    /// **IRI reifier** of an RDF 1.2 annotation sub-field. Absent (the common
    /// case) means the focus node. [OPUS-5] sq-lsp7k.1.5
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub subject: Option<TermRef>,
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
    /// contribute additions. Inverse, read-only, computed, and complex-path
    /// fields are ignored.
    ///
    /// [OPUS-5] sq-lsp7k.1.5 RDF 1.2 **annotation** sub-fields participate too,
    /// matched per (IRI reifier, predicate): editing a reifier's
    /// provenance/time/confidence metadata produces changes carrying that
    /// reifier as [`FieldValueDiff::subject`]. Blank-node reifiers derive
    /// read-only, so they never reach here.
    pub fn between(before: &FormDescription, after: &FormDescription) -> Self {
        if before.focus != after.focus {
            return Self::default();
        }

        let mut diff = Self::default();
        for (subject, after_field) in fields(after).filter(|(_, field)| eligible(field)) {
            let Some(predicate) = bare_predicate(&after_field.path) else {
                continue;
            };
            let before_field = fields(before)
                .find(|(s, field)| {
                    *s == subject && eligible(field) && bare_predicate(&field.path) == Some(predicate)
                })
                .map(|(_, field)| field);

            for value in &after_field.values {
                let was_present = before_field
                    .is_some_and(|field| field.values.iter().any(|old| old.term == value.term));
                if !was_present {
                    diff.added.push(FieldValueDiff {
                        path: after_field.path.clone(),
                        value: value.term.clone(),
                        subject: subject.cloned(),
                    });
                }
            }

            if let Some(before_field) = before_field {
                for value in &before_field.values {
                    if !after_field.values.iter().any(|new| new.term == value.term) {
                        diff.removed.push(FieldValueDiff {
                            path: after_field.path.clone(),
                            value: value.term.clone(),
                            subject: subject.cloned(),
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
///
/// [SONNET-4.6] It also returns an empty string — rather than partial or
/// malformed syntax — when any term in the diff carries metadata that cannot be
/// serialized safely: a language tag outside the N-Triples `LANGTAG` grammar,
/// or a base direction that is neither `ltr` nor `rtl`. A [`TermRef`] is public and
/// `Deserialize`, so a renderer can hand back a term that never came from the
/// parser; the update is built ALL-or-NOTHING so such a term can never
/// contribute executable syntax.
pub fn to_sparql_update(before: &FormDescription, after: &FormDescription) -> String {
    let diff = FormDiff::between(before, after);
    if diff.added.is_empty() && diff.removed.is_empty() {
        return String::new();
    }

    let Some(focus) = term_to_ntriples(&after.focus) else {
        return String::new();
    };
    let triples = |changes: &[FieldValueDiff]| -> Option<String> {
        changes
            .iter()
            .map(|change| {
                // An annotation change is written against its IRI reifier, not
                // the focus node. [OPUS-5] sq-lsp7k.1.5
                let subject = match &change.subject {
                    Some(reifier) => term_to_ntriples(reifier)?,
                    None => focus.clone(),
                };
                Some(format!(
                    "  {subject} {} {} .\n",
                    change.path,
                    term_to_ntriples(&change.value)?
                ))
            })
            .collect()
    };

    let (Some(removed), Some(added)) = (triples(&diff.removed), triples(&diff.added)) else {
        return String::new();
    };
    format!("DELETE {{\n{}}}\nINSERT {{\n{}}}\nWHERE {{}}", removed, added)
}

/// Every writable field paired with the subject it belongs to: `None` for the
/// focus node's own fields, `Some(reifier)` for an RDF 1.2 annotation
/// sub-field. Only **IRI** reifiers are yielded — a blank-node reifier cannot
/// be named in a SPARQL Update `DELETE` template, so its annotations are never
/// written back. [OPUS-5] sq-lsp7k.1.5
fn fields(form: &FormDescription) -> impl Iterator<Item = (Option<&TermRef>, &FormField)> {
    form.groups.iter().flat_map(|group| {
        group.fields.iter().flat_map(|field| {
            std::iter::once((None, field)).chain(
                field
                    .values
                    .iter()
                    .flat_map(|value| &value.annotations)
                    .filter(|annotation| annotation.reifier.kind == "iri")
                    .flat_map(|annotation| {
                        annotation
                            .fields
                            .iter()
                            .map(move |sub| (Some(&annotation.reifier), sub))
                    }),
            )
        })
    })
}

fn eligible(field: &FormField) -> bool {
    field.editable && !field.inverse && bare_predicate(&field.path).is_some()
}

fn bare_predicate(path: &str) -> Option<&str> {
    let iri = path.strip_prefix('<')?.strip_suffix('>')?;
    (!iri.is_empty() && !iri.contains(['<', '>', ' ', '\t', '\r', '\n'])).then_some(iri)
}

/// Renders a term as N-Triples, or `None` when its metadata cannot be
/// serialized safely.
///
/// [SONNET-4.6] A language tag and a base direction have NO escape form in
/// N-Triples — they are written literally after `@` — so, unlike a lexical form
/// or an IRI, they cannot be neutralised by escaping. [`TermRef`] is public and
/// `Deserialize`, so those two components can carry anything a caller supplies;
/// validating them here (and refusing to render otherwise) is what stops
/// punctuation in them reaching [`to_sparql_update`] as executable syntax.
pub(crate) fn term_to_ntriples(term: &TermRef) -> Option<String> {
    Some(match term.kind.as_str() {
        "iri" => format!("<{}>", escape_iri(&term.value)),
        "bnode" => format!("_:{}", term.value),
        "literal" => {
            let literal = format!("\"{}\"", escape_literal(&term.value));
            match (&term.language, &term.direction) {
                // RDF 1.2: a directional language-tagged string is written
                // `"…"@lang--dir`. Dropping the direction would silently
                // rewrite the term. [OPUS-5] sq-lsp7k.1.5
                (Some(language), Some(direction)) => {
                    if !valid_language(language) || !valid_direction(direction) {
                        return None;
                    }
                    format!("{literal}@{language}--{direction}")
                }
                (Some(language), None) => {
                    if !valid_language(language) {
                        return None;
                    }
                    format!("{literal}@{language}")
                }
                // A base direction with no language tag is not a term RDF 1.2
                // can express; rendering the plain literal would silently drop
                // it, so decline instead. [SONNET-4.6]
                (None, Some(_)) => return None,
                (None, None) => match &term.datatype {
                    Some(datatype) => format!("{literal}^^<{}>", escape_iri(datatype)),
                    None => literal,
                },
            }
        }
        // RDF 1.2 triple terms: re-render from the structured components when
        // present (so renamed blank-node labels and escaping stay consistent),
        // else fall back to the carried N-Triples text. [OPUS-5] sq-lsp7k.1.5
        _ => match &term.triple {
            Some(t) => format!(
                "<<( {} {} {} )>>",
                term_to_ntriples(&t.subject)?,
                term_to_ntriples(&t.predicate)?,
                term_to_ntriples(&t.object)?
            ),
            None => term.value.clone(),
        },
    })
}

/// RDF 1.2 admits exactly two base directions; anything else is not a direction
/// this crate will write. [SONNET-4.6]
fn valid_direction(direction: &str) -> bool {
    matches!(direction, "ltr" | "rtl")
}

/// The N-Triples `LANGTAG` production: `[a-zA-Z]+ ('-' [a-zA-Z0-9]+)*`.
/// [SONNET-4.6]
fn valid_language(language: &str) -> bool {
    let mut parts = language.split('-');
    parts
        .next()
        .is_some_and(|primary| !primary.is_empty() && primary.bytes().all(|b| b.is_ascii_alphabetic()))
        && parts.all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_alphanumeric()))
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
