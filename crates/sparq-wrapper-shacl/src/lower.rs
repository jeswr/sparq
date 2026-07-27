//! SHACL shapes → [`RustModel`]. [FABLE-5] (sq-1rg2q.12)
//!
//! Reuses `sparq-shacl`'s [`ShapesModel`] verbatim — this crate parses no
//! Turtle and re-implements no SHACL syntax rules. Ill-formed constructs are
//! taken from [`ShapesModel::ill_formed`] and turned into a typed error rather
//! than re-detected here.
//!
//! ## Determinism
//!
//! The lowering must not depend on the order statements happened to appear in
//! the shapes graph, so every iteration order is fixed by content:
//!
//! * root node shapes are visited in ascending IRI order;
//! * a shape's fields are sorted by predicate IRI;
//! * anonymous (`sh:node [ … ]`) types are named after the parent type and the
//!   field that reaches them, discovered in that same fixed order;
//! * the output vectors are sorted by generated name.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use oxrdf::Term;
use sparq_shacl::{Component, Path, ShapesModel, Target};

use crate::error::LoweringError;
use crate::schema::{
    Cardinality, ClosedShape, RustField, RustModel, RustReference, RustType, ScalarType, ValueType,
};

/// Lowers every named node shape in `shapes` — plus every shape reachable from
/// one through `sh:node` — to a Rust object model.
///
/// Shapes that describe no object model are skipped rather than approximated:
/// property shapes (they become fields of their parent), and anonymous shapes
/// used only as logical operands (`sh:not`, `sh:or`, …).
///
/// # Errors
///
/// Returns a [`LoweringError`] when the shapes graph is ill-formed, describes a
/// contradiction, or uses SHACL this generator declines to approximate. See the
/// variant docs for the full taxonomy.
pub fn lower(shapes: &ShapesModel) -> Result<RustModel, LoweringError> {
    let ill_formed = shapes.ill_formed();
    if !ill_formed.is_empty() {
        return Err(LoweringError::IllFormedShapes {
            constructs: ill_formed
                .iter()
                .map(|c| {
                    format!(
                        "{} <{}>: {}",
                        term_label(&c.node),
                        c.predicate,
                        c.message
                    )
                })
                .collect(),
        });
    }

    let mut ctx = Ctx {
        shapes,
        assigned: vec![None; shapes.shapes.len()],
        taken: BTreeMap::new(),
        queue: VecDeque::new(),
        references: BTreeMap::new(),
    };

    // Roots: named node shapes (no `sh:path`), in ascending IRI order.
    let mut roots: Vec<(String, usize)> = shapes
        .shapes
        .iter()
        .enumerate()
        .filter(|(_, s)| s.path.is_none())
        .filter_map(|(i, s)| match &s.node {
            Term::NamedNode(n) => Some((n.as_str().to_string(), i)),
            _ => None,
        })
        .collect();
    roots.sort();
    for (iri, index) in &roots {
        let name = pascal_ident(iri)?;
        ctx.assign(*index, name, &term_label(&shapes.shapes[*index].node))?;
    }

    let mut types = Vec::new();
    while let Some(index) = ctx.queue.pop_front() {
        let ty = ctx.lower_type(index)?;
        types.push(ty);
    }
    types.sort_by(|a, b| a.name.cmp(&b.name));

    // Reference newtypes last, so a `sh:class` target can never steal a name a
    // real shape claimed (and the collision is reported the same way round
    // regardless of traversal order).
    let discovered: Vec<(String, String)> = ctx
        .references
        .iter()
        .map(|(class, name)| (class.clone(), name.clone()))
        .collect();
    let mut references = Vec::new();
    for (class, name) in discovered {
        ctx.claim(name.clone(), format!("<{}>", class))?;
        references.push(RustReference { name, class });
    }
    references.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(RustModel { types, references })
}

struct Ctx<'a> {
    shapes: &'a ShapesModel,
    /// Generated type name per shape index, once assigned.
    assigned: Vec<Option<String>>,
    /// Every claimed Rust type name → the shape or class that claimed it.
    taken: BTreeMap<String, String>,
    /// Shape indices awaiting lowering, in assignment order.
    queue: VecDeque<usize>,
    /// `sh:class` IRI → generated reference newtype name.
    references: BTreeMap<String, String>,
}

impl Ctx<'_> {
    /// Claims `name` for `owner`, failing if a different owner already holds it.
    fn claim(&mut self, name: String, owner: String) -> Result<(), LoweringError> {
        match self.taken.get(&name) {
            Some(first) if *first != owner => Err(LoweringError::DuplicateType {
                name,
                first_shape: first.clone(),
                second_shape: owner,
            }),
            Some(_) => Ok(()),
            None => {
                self.taken.insert(name, owner);
                Ok(())
            }
        }
    }

    /// Assigns `name` to shape `index` and enqueues it, or returns the name it
    /// already has.
    fn assign(
        &mut self,
        index: usize,
        name: String,
        owner: &str,
    ) -> Result<String, LoweringError> {
        if let Some(existing) = &self.assigned[index] {
            return Ok(existing.clone());
        }
        self.claim(name.clone(), owner.to_string())?;
        self.assigned[index] = Some(name.clone());
        self.queue.push_back(index);
        Ok(name)
    }

    fn lower_type(&mut self, index: usize) -> Result<RustType, LoweringError> {
        let shape = &self.shapes.shapes[index];
        let label = term_label(&shape.node);
        let name = self.assigned[index]
            .clone()
            .expect("every queued shape was assigned a name");

        let mut target_classes: BTreeSet<String> = BTreeSet::new();
        for target in &shape.targets {
            if let Target::Class(Term::NamedNode(n)) | Target::ImplicitClass(Term::NamedNode(n)) =
                target
            {
                target_classes.insert(n.as_str().to_string());
            }
        }

        let mut fields: Vec<RustField> = Vec::new();
        for child in shape.property_children.clone() {
            fields.push(self.lower_field(child, &name)?);
        }
        // Sorted BEFORE the collision check, so which of two colliding
        // predicates is reported as "first" does not depend on the order the
        // shapes graph was serialised in.
        fields.sort_by(|a, b| a.predicate.cmp(&b.predicate));
        let mut by_name: BTreeMap<String, String> = BTreeMap::new();
        for field in &fields {
            if let Some(first) = by_name.get(&field.name) {
                return Err(LoweringError::DuplicateField {
                    shape: label,
                    field: field.name.clone(),
                    first_predicate: first.clone(),
                    second_predicate: field.predicate.clone(),
                });
            }
            by_name.insert(field.name.clone(), field.predicate.clone());
        }

        let shape = &self.shapes.shapes[index];
        let mut closed = None;
        for component in &shape.components {
            let Component::Closed { ignored, by_types } = component else {
                continue;
            };
            if *by_types {
                return Err(LoweringError::UnsupportedClosedMode { shape: label });
            }
            let mut allowed: BTreeSet<String> =
                fields.iter().map(|f| f.predicate.clone()).collect();
            for term in ignored {
                if let Term::NamedNode(n) = term {
                    allowed.insert(n.as_str().to_string());
                }
            }
            closed = Some(ClosedShape {
                allowed: allowed.into_iter().collect(),
            });
        }

        Ok(RustType {
            name,
            shape: match &shape.node {
                Term::NamedNode(n) => Some(n.as_str().to_string()),
                _ => None,
            },
            target_classes: target_classes.into_iter().collect(),
            fields,
            closed,
        })
    }

    fn lower_field(
        &mut self,
        index: usize,
        parent: &str,
    ) -> Result<RustField, LoweringError> {
        let shape = &self.shapes.shapes[index];
        let label = term_label(&shape.node);
        let predicate = match &shape.path {
            Some(Path::Predicate(p)) => p.clone(),
            Some(other) => {
                return Err(LoweringError::UnsupportedPath {
                    shape: label,
                    detail: format!("{} is not a single predicate IRI", path_kind(other)),
                })
            }
            None => {
                return Err(LoweringError::UnsupportedPath {
                    shape: label,
                    detail: "no sh:path".to_string(),
                })
            }
        };
        let name = snake_ident(&predicate)?;

        let mut min: Option<u64> = None;
        let mut max: Option<u64> = None;
        let mut value: Option<(&'static str, ValueType)> = None;
        // Cloned so the `sh:node` arm can mutate the context while walking.
        for component in shape.components.clone() {
            match component {
                Component::MinCount(n) => {
                    set_count(&mut min, n, "sh:minCount", &label, &predicate)?;
                }
                Component::MaxCount(n) => {
                    set_count(&mut max, n, "sh:maxCount", &label, &predicate)?;
                }
                Component::Datatype(datatypes) => {
                    if datatypes.len() != 1 {
                        return Err(LoweringError::AmbiguousValueType {
                            shape: label,
                            predicate,
                            keyword: "sh:datatype".to_string(),
                            values: datatypes,
                        });
                    }
                    let datatype = &datatypes[0];
                    let scalar = ScalarType::from_datatype(datatype).ok_or_else(|| {
                        LoweringError::UnsupportedDatatype {
                            shape: label.clone(),
                            predicate: predicate.clone(),
                            datatype: datatype.clone(),
                        }
                    })?;
                    set_value(
                        &mut value,
                        ("sh:datatype", ValueType::Scalar(scalar)),
                        &label,
                        &predicate,
                    )?;
                }
                Component::Class(class) => {
                    let Term::NamedNode(class) = &class else {
                        return Err(LoweringError::InvalidIdentifier {
                            iri: term_label(&class),
                            reason: format!("sh:class of {} must be an IRI", label),
                        });
                    };
                    let class = class.as_str().to_string();
                    let reference = reference_ident(&class)?;
                    self.references.insert(class.clone(), reference.clone());
                    set_value(
                        &mut value,
                        (
                            "sh:class",
                            ValueType::Reference {
                                class,
                                name: reference,
                            },
                        ),
                        &label,
                        &predicate,
                    )?;
                }
                Component::ClassIn(classes) => {
                    return Err(LoweringError::AmbiguousValueType {
                        shape: label,
                        predicate,
                        keyword: "sh:class".to_string(),
                        values: classes.iter().map(term_label).collect(),
                    })
                }
                Component::Node(child) => {
                    let proposed = match &self.shapes.shapes[child].node {
                        Term::NamedNode(n) => pascal_ident(n.as_str())?,
                        _ => format!("{}{}", parent, pascal_from_ident(&name)),
                    };
                    let owner = term_label(&self.shapes.shapes[child].node);
                    let nested = self.assign(child, proposed, &owner)?;
                    set_value(
                        &mut value,
                        ("sh:node", ValueType::Nested(nested)),
                        &label,
                        &predicate,
                    )?;
                }
                // Every other constraint component stays the validator's job:
                // it restricts which graphs load, not what the Rust type is.
                _ => {}
            }
        }

        let Some((_, value)) = value else {
            return Err(LoweringError::MissingValueType {
                shape: label,
                predicate,
            });
        };

        match (min, max) {
            (Some(mn), Some(mx)) if mn > mx => {
                return Err(LoweringError::ContradictoryCardinality {
                    shape: label,
                    predicate,
                    min: mn,
                    max: mx,
                })
            }
            _ => {}
        }
        // `sh:maxCount 1` is the only cardinality with a single-valued Rust
        // spelling; `sh:minCount` then decides whether it is optional.
        let cardinality = if max == Some(1) {
            if min.unwrap_or(0) >= 1 {
                Cardinality::Required
            } else {
                Cardinality::Optional
            }
        } else {
            Cardinality::Many {
                min: min.unwrap_or(0),
                max,
            }
        };

        Ok(RustField {
            name,
            predicate,
            value,
            cardinality,
        })
    }
}

fn set_count(
    slot: &mut Option<u64>,
    found: u64,
    keyword: &str,
    shape: &str,
    predicate: &str,
) -> Result<(), LoweringError> {
    match slot {
        // Reported low-then-high rather than in encounter order: a shapes graph
        // that declares the same keyword twice has no statement order to speak of.
        Some(first) if *first != found => Err(LoweringError::ConflictingCardinality {
            shape: shape.to_string(),
            predicate: predicate.to_string(),
            keyword: keyword.to_string(),
            first: (*first).min(found),
            second: (*first).max(found),
        }),
        _ => {
            *slot = Some(found);
            Ok(())
        }
    }
}

fn set_value(
    slot: &mut Option<(&'static str, ValueType)>,
    found: (&'static str, ValueType),
    shape: &str,
    predicate: &str,
) -> Result<(), LoweringError> {
    match slot {
        // Keywords reported in a fixed (alphabetical) order — `sh:datatype`
        // before `sh:node` — so the error does not depend on which constraint
        // the parser happened to visit first.
        Some(first) if *first != found => Err(LoweringError::ConflictingValueType {
            shape: shape.to_string(),
            predicate: predicate.to_string(),
            first: first.0.min(found.0).to_string(),
            second: first.0.max(found.0).to_string(),
        }),
        _ => {
            *slot = Some(found);
            Ok(())
        }
    }
}

fn path_kind(path: &Path) -> &'static str {
    match path {
        Path::Predicate(_) => "a predicate path",
        Path::Inverse(_) => "an inverse path",
        Path::Sequence(_) => "a sequence path",
        Path::Alternative(_) => "an alternative path",
        Path::ZeroOrMore(_) => "a zero-or-more path",
        Path::OneOrMore(_) => "a one-or-more path",
        Path::ZeroOrOne(_) => "a zero-or-one path",
    }
}

fn term_label(term: &Term) -> String {
    match term {
        Term::NamedNode(n) => format!("<{}>", n.as_str()),
        Term::BlankNode(b) => format!("_:{}", b.as_str()),
        #[allow(unreachable_patterns)]
        other => other.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Identifier derivation
// ---------------------------------------------------------------------------

/// The IRI's local name: everything after the last `#`, `/` or `:`.
fn local_name(iri: &str) -> &str {
    match iri.rfind(['#', '/', ':']) {
        Some(cut) => &iri[cut + 1..],
        None => iri,
    }
}

/// Splits a local name into alphanumeric words, breaking on separators and on
/// `camelCase`, `HTTPStatus` (acronym) and letter/digit transitions.
fn words(local: &str) -> Vec<String> {
    let chars: Vec<char> = local.chars().collect();
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if !c.is_ascii_alphanumeric() {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
            continue;
        }
        let previous = if current.is_empty() {
            None
        } else {
            chars[..i].iter().rev().find(|p| p.is_ascii_alphanumeric())
        };
        let boundary = match previous {
            // `givenName` / `x3` / `3d` — a change of character class.
            Some(&p) if p.is_ascii_lowercase() && c.is_ascii_uppercase() => true,
            Some(&p) if p.is_ascii_digit() != c.is_ascii_digit() => true,
            // `HTTPStatus` — the last capital of an acronym starts the next word.
            Some(&p)
                if p.is_ascii_uppercase()
                    && c.is_ascii_uppercase()
                    && chars.get(i + 1).is_some_and(char::is_ascii_lowercase) =>
            {
                true
            }
            _ => false,
        };
        if boundary {
            out.push(std::mem::take(&mut current));
        }
        current.push(c);
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

fn reject_empty(iri: &str, local: &str, produced: &str) -> Result<(), LoweringError> {
    if produced.is_empty() {
        return Err(LoweringError::InvalidIdentifier {
            iri: iri.to_string(),
            reason: format!(
                "local name {:?} contains no ASCII alphanumeric characters",
                local
            ),
        });
    }
    Ok(())
}

/// `http://example.org/PersonShape` → `PersonShape`; `ex:home-address` → `HomeAddress`.
fn pascal_ident(iri: &str) -> Result<String, LoweringError> {
    let local = local_name(iri);
    let mut out = String::new();
    for word in words(local) {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.push_str(chars.as_str());
        }
    }
    reject_empty(iri, local, &out)?;
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    Ok(out)
}

/// Re-pascalises an already-derived snake_case field name, for naming the
/// anonymous type an `sh:node [ … ]` on that field lowers to.
fn pascal_from_ident(ident: &str) -> String {
    ident
        .split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect()
}

/// `http://example.org/givenName` → `given_name`, escaping Rust keywords.
fn snake_ident(iri: &str) -> Result<String, LoweringError> {
    let local = local_name(iri);
    let mut out = words(local)
        .iter()
        .map(|w| w.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("_");
    reject_empty(iri, local, &out)?;
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    // `iri` is the generated struct's own subject field, so a predicate that
    // derives it is escaped exactly like a keyword rather than colliding.
    if is_rust_keyword(&out) || out == SUBJECT_FIELD {
        out.push('_');
    }
    Ok(out)
}

/// The field every generated struct carries for the subject it was loaded from.
pub(crate) const SUBJECT_FIELD: &str = "iri";

/// `http://example.org/Organization` → `OrganizationRef`.
fn reference_ident(iri: &str) -> Result<String, LoweringError> {
    Ok(format!("{}Ref", pascal_ident(iri)?))
}

/// Every Rust 2021 keyword, including the reserved ones — a field named after
/// `ex:become` must still compile in a future edition.
fn is_rust_keyword(word: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true",
        "type", "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final",
        "macro", "override", "priv", "try", "typeof", "unsized", "virtual", "yield", "gen",
    ];
    KEYWORDS.contains(&word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_name_splits_on_the_last_separator() {
        assert_eq!(local_name("http://example.org/ns#givenName"), "givenName");
        assert_eq!(local_name("http://example.org/givenName"), "givenName");
        assert_eq!(local_name("urn:example:givenName"), "givenName");
        assert_eq!(local_name("bare"), "bare");
    }

    #[test]
    fn identifiers_are_derived_from_local_names() {
        assert_eq!(pascal_ident("http://e.org/PersonShape").unwrap(), "PersonShape");
        assert_eq!(pascal_ident("http://e.org/home-address").unwrap(), "HomeAddress");
        assert_eq!(snake_ident("http://e.org/ns#givenName").unwrap(), "given_name");
        assert_eq!(snake_ident("http://e.org/ns#HTTPStatus").unwrap(), "http_status");
        assert_eq!(reference_ident("http://e.org/Organization").unwrap(), "OrganizationRef");
    }

    #[test]
    fn digit_led_and_keyword_identifiers_are_escaped() {
        assert_eq!(snake_ident("http://e.org/ns#type").unwrap(), "type_");
        assert_eq!(snake_ident("http://e.org/ns#3d").unwrap(), "_3_d");
        assert_eq!(pascal_ident("http://e.org/ns#3d").unwrap(), "_3D");
    }

    #[test]
    fn a_local_name_without_alphanumerics_is_a_typed_error() {
        let err = snake_ident("http://e.org/ns#--").unwrap_err();
        assert!(
            matches!(err, LoweringError::InvalidIdentifier { .. }),
            "unexpected error: {:?}",
            err
        );
        assert!(err.to_string().contains("no ASCII alphanumeric"));
    }

    #[test]
    fn pascal_from_ident_rebuilds_a_type_name_from_a_field_name() {
        assert_eq!(pascal_from_ident("home_address"), "HomeAddress");
        assert_eq!(pascal_from_ident("type_"), "Type");
    }

    #[test]
    fn path_kinds_are_named_for_the_error_message() {
        assert_eq!(path_kind(&Path::Predicate("p".into())), "a predicate path");
        assert_eq!(
            path_kind(&Path::Inverse(Box::new(Path::Predicate("p".into())))),
            "an inverse path"
        );
    }
}
