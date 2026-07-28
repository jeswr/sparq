//! Shapes graph → [`ObjectModel`]: the "what does this SHACL mean as Rust" half.
//!
//! [FABLE-5] (sq-1rg2q.12) The input is [`sparq_shacl::ShapesModel`] — the
//! workspace's one SHACL parser. This module never looks at triples, only at the
//! parsed [`Shape`] / [`Component`] / [`Path`] / [`Target`] surface, so shape
//! syntax has exactly one implementation.
//!
//! # What is lowered
//!
//! | SHACL | Rust |
//! |---|---|
//! | a named node shape | a `pub struct` |
//! | `sh:property` with `sh:path <p>` | a field named after `<p>` |
//! | `sh:maxCount 1`, no `sh:minCount` | `Option<T>` |
//! | `sh:minCount >= 1` + `sh:maxCount 1` | required `T` |
//! | any other cardinality | `Vec<T>` + retained min/max checks |
//! | `sh:datatype` | a checked scalar ([`ScalarType`]) |
//! | `sh:class` | a typed reference ([`ClassMarker`]) |
//! | `sh:node` | a nested generated type |
//! | `sh:closed true` | a predicate whitelist |
//!
//! # What is NOT lowered
//!
//! Every other constraint component — `sh:pattern`, `sh:minInclusive`, `sh:in`,
//! `sh:or`, `sh:qualifiedValueShape`, `sh:sparql`, … — carries no *structural*
//! information, so it does not change the generated types and is **silently not
//! enforced by the generated loader**. The generated code is a typed reader, not
//! a SHACL validator: run [`sparq_shacl::validate`] for conformance. This is
//! stated again on the emitted module header and in the crate README so a caller
//! cannot mistake one for the other.

use std::collections::{BTreeMap, VecDeque};

use oxrdf::Term;
use sparq_shacl::{Component, Path, Shape, ShapesModel, Target};

use crate::error::LowerError;
use crate::schema::{
    Cardinality, ClassMarker, Field, ModelType, ObjectModel, RustScalar, ScalarType, ValueType,
};

/// Item names the emitted prelude always defines, plus the `std` prelude names
/// the emitted code relies on. A shape or class that wants one of these is a
/// [`LowerError::ReservedName`] rather than a silently shadowed type.
const RESERVED: &[&str] = &[
    "Box",
    "ClassMarker",
    "Err",
    "LoadError",
    "None",
    "ObjectKind",
    "Ok",
    "Option",
    "Ref",
    "Result",
    "Some",
    "String",
    "Triple",
    "Value",
    "Vec",
    "bool",
    "f32",
    "f64",
    "i64",
    "str",
];

const XSD: &str = "http://www.w3.org/2001/XMLSchema#";

/// Lowers a parsed shapes graph to a deterministic Rust object model.
///
/// # Errors
///
/// Returns the first [`LowerError`] in a deterministic order — an ill-formed
/// shapes graph is rejected before anything is lowered, then shapes are visited
/// in sorted order. See [`LowerError`] for the failure taxonomy.
pub fn lower(model: &ShapesModel) -> Result<ObjectModel, LowerError> {
    if let Some(err) = first_ill_formed(model) {
        return Err(err);
    }
    Ctx::new(model).run()
}

/// Relays the SHACL parser's own syntax-rule findings as a typed error, picking
/// the lexicographically first so the message does not depend on parse order.
fn first_ill_formed(model: &ShapesModel) -> Option<LowerError> {
    model
        .ill_formed()
        .iter()
        .map(|c| (term_display(&c.node), c.predicate.clone(), c.message.clone()))
        .min()
        .map(|(node, predicate, message)| LowerError::IllFormedShape {
            node,
            predicate,
            message,
        })
}

struct Ctx<'a> {
    model: &'a ShapesModel,
    /// Shape index → Rust type name.
    names: BTreeMap<usize, String>,
    /// Shape index → its stable display string (see [`Ctx::assign_name`]).
    displays: BTreeMap<usize, String>,
    /// Rust item name → the shapes-graph node that claimed it.
    claimed: BTreeMap<String, String>,
    /// Class-marker name → its sorted class IRIs.
    markers: BTreeMap<String, Vec<String>>,
    /// Shape indices still to lower.
    queue: VecDeque<usize>,
    /// Type name → lowered type (a `BTreeMap` so the output is name-sorted).
    out: BTreeMap<String, ModelType>,
}

impl<'a> Ctx<'a> {
    fn new(model: &'a ShapesModel) -> Self {
        Self {
            model,
            names: BTreeMap::new(),
            displays: BTreeMap::new(),
            claimed: BTreeMap::new(),
            markers: BTreeMap::new(),
            queue: VecDeque::new(),
            out: BTreeMap::new(),
        }
    }

    fn run(mut self) -> Result<ObjectModel, LowerError> {
        // Seeds: every NAMED node shape (no `sh:path`), in IRI order. Inline
        // shapes reached through `sh:node` are queued as they are discovered.
        let mut seeds: Vec<(String, usize)> = self
            .model
            .shapes
            .iter()
            .enumerate()
            .filter(|(_, s)| s.path.is_none())
            .filter_map(|(i, s)| match &s.node {
                Term::NamedNode(n) => Some((n.as_str().to_string(), i)),
                _ => None,
            })
            .collect();
        seeds.sort();

        for (_, idx) in seeds {
            self.assign_name(idx, None)?;
        }
        while let Some(idx) = self.queue.pop_front() {
            self.lower_type(idx)?;
        }

        let class_markers = self
            .markers
            .into_iter()
            .map(|(name, classes)| ClassMarker { name, classes })
            .collect();
        Ok(ObjectModel {
            types: self.out.into_values().collect(),
            class_markers,
        })
    }

    /// Records `name` as taken by `source`, rejecting reserved names and clashes.
    fn claim(&mut self, name: &str, source: &str) -> Result<(), LowerError> {
        if RESERVED.contains(&name) {
            return Err(LowerError::ReservedName {
                name: name.to_string(),
                source: source.to_string(),
            });
        }
        if let Some(first) = self.claimed.get(name) {
            if first != source {
                return Err(LowerError::DuplicateTypeName {
                    name: name.to_string(),
                    first: first.clone(),
                    second: source.to_string(),
                });
            }
            return Ok(());
        }
        self.claimed.insert(name.to_string(), source.to_string());
        Ok(())
    }

    /// Names a shape and queues it for lowering.
    ///
    /// `fallback` is the `(name, display)` pair derived from the referring
    /// shape, used for an inline (blank-node) shape. A blank node's LABEL is
    /// re-minted on every parse of the same document, so it must never reach
    /// the IR (and hence the emitted `SHAPE` constant) — an inline shape is
    /// identified by where it was referenced FROM, which is content-derived and
    /// therefore stable.
    fn assign_name(
        &mut self,
        idx: usize,
        fallback: Option<(&str, &str)>,
    ) -> Result<String, LowerError> {
        if let Some(existing) = self.names.get(&idx) {
            return Ok(existing.clone());
        }
        let node = &self.model.shapes[idx].node;
        let (name, display) = match node {
            Term::NamedNode(n) => (upper_camel_from_iri(n.as_str())?, term_display(node)),
            _ => {
                let (name, display) =
                    fallback.ok_or_else(|| LowerError::UnnameableIri {
                        iri: term_display(node),
                    })?;
                (name.to_string(), display.to_string())
            }
        };
        self.claim(&name, &display)?;
        self.names.insert(idx, name.clone());
        self.displays.insert(idx, display);
        self.queue.push_back(idx);
        Ok(name)
    }

    /// Interns a class marker for `classes`, reusing an identical existing one.
    fn marker_for(&mut self, classes: Vec<String>, source: &str) -> Result<String, LowerError> {
        let classes = sorted_dedup(classes);
        let mut name = String::new();
        for c in &classes {
            name.push_str(&upper_camel_from_iri(c)?);
        }
        name.push_str(if classes.len() == 1 { "Class" } else { "Classes" });
        match self.markers.get(&name) {
            Some(existing) if *existing == classes => Ok(name),
            Some(existing) => Err(LowerError::DuplicateTypeName {
                first: existing
                    .iter()
                    .map(|c| format!("<{c}>"))
                    .collect::<Vec<_>>()
                    .join(", "),
                second: source.to_string(),
                name,
            }),
            None => {
                self.claim(&name, source)?;
                self.markers.insert(name.clone(), classes);
                Ok(name)
            }
        }
    }

    fn lower_type(&mut self, idx: usize) -> Result<(), LowerError> {
        let type_name = self.names[&idx].clone();
        if self.out.contains_key(&type_name) {
            return Ok(());
        }
        let shape = &self.model.shapes[idx];
        let shape_disp = self.displays[&idx].clone();

        let target_classes = sorted_dedup(
            shape
                .targets
                .iter()
                .filter_map(|t| match t {
                    Target::Class(Term::NamedNode(n)) | Target::ImplicitClass(Term::NamedNode(n)) => {
                        Some(n.as_str().to_string())
                    }
                    _ => None,
                })
                .collect(),
        );

        let closed = self.closed_whitelist(idx, &shape_disp)?;

        // Lower the property children in predicate order so both the emitted
        // fields AND the order in which a lowering error surfaces are functions
        // of the shapes graph's content, never of its traversal order.
        //
        // A property shape is usually a blank node, so it too is described by
        // where it hangs rather than by its (unstable) label.
        let mut children: Vec<(String, String, usize)> = shape
            .property_children
            .iter()
            .map(|&c| {
                let child = &self.model.shapes[c];
                match &child.path {
                    Some(Path::Predicate(p)) => (
                        p.clone(),
                        format!("{shape_disp} sh:property <{p}>"),
                        c,
                    ),
                    _ => (String::new(), term_display(&child.node), c),
                }
            })
            .collect();
        children.sort();
        children.dedup_by_key(|(_, _, c)| *c);

        let mut fields = Vec::with_capacity(children.len());
        let mut by_name: BTreeMap<String, String> = BTreeMap::new();
        for (_, child_disp, child) in children {
            let field = self.lower_field(&type_name, &child_disp, child)?;
            if let Some(first) = by_name.get(&field.name) {
                return Err(LowerError::DuplicateFieldName {
                    type_name,
                    field: field.name,
                    first_predicate: first.clone(),
                    second_predicate: field.predicate,
                });
            }
            by_name.insert(field.name.clone(), field.predicate.clone());
            fields.push(field);
        }
        fields.sort_by(|a, b| a.predicate.cmp(&b.predicate));

        self.out.insert(
            type_name.clone(),
            ModelType {
                name: type_name,
                shape: shape_disp,
                target_classes,
                fields,
                closed,
            },
        );
        Ok(())
    }

    /// The static allowed-predicate set of an `sh:closed true` shape: this
    /// shape's own `sh:property` predicate paths plus `sh:ignoredProperties` —
    /// the same set `sparq-shacl`'s evaluator computes for the fixed case.
    fn closed_whitelist(
        &self,
        idx: usize,
        shape_disp: &str,
    ) -> Result<Option<Vec<String>>, LowerError> {
        let shape = &self.model.shapes[idx];
        let mut closed = None;
        for component in &shape.components {
            let Component::Closed { ignored, by_types } = component else {
                continue;
            };
            if *by_types {
                return Err(LowerError::UnsupportedComponent {
                    shape: shape_disp.to_string(),
                    component: "sh:closed sh:ByTypes",
                });
            }
            let mut allowed: Vec<String> = ignored
                .iter()
                .filter_map(|t| match t {
                    Term::NamedNode(n) => Some(n.as_str().to_string()),
                    _ => None,
                })
                .collect();
            for &child in &shape.property_children {
                if let Some(Path::Predicate(p)) = &self.model.shapes[child].path {
                    allowed.push(p.clone());
                }
            }
            closed = Some(sorted_dedup(allowed));
        }
        Ok(closed)
    }

    fn lower_field(
        &mut self,
        parent: &str,
        disp: &str,
        idx: usize,
    ) -> Result<Field, LowerError> {
        let shape = &self.model.shapes[idx];
        let disp = disp.to_string();

        let predicate = match &shape.path {
            Some(Path::Predicate(p)) => p.clone(),
            Some(other) => {
                return Err(LowerError::UnsupportedPath {
                    shape: disp,
                    form: path_form(other),
                })
            }
            None => return Err(LowerError::MissingPath { shape: disp }),
        };
        let name = snake_from_iri(&predicate)?;
        let cardinality = cardinality_of(shape, &disp)?;

        // Repeated constraints are conjunctive in SHACL, so a second occurrence
        // of the SAME value-typing component is a contradiction the generator
        // refuses rather than resolves.
        let mut datatypes: Option<Vec<String>> = None;
        let mut classes: Option<Vec<String>> = None;
        let mut nested: Option<usize> = None;
        for component in &shape.components {
            let (slot_taken, kind): (bool, &'static str) = match component {
                Component::Datatype(dts) => {
                    let taken = datatypes.is_some();
                    if !taken {
                        datatypes = Some(dts.clone());
                    }
                    (taken, "sh:datatype")
                }
                Component::Class(t) => {
                    let taken = classes.is_some();
                    if !taken {
                        classes = Some(iris_of(std::slice::from_ref(t)));
                    }
                    (taken, "sh:class")
                }
                Component::ClassIn(ts) => {
                    let taken = classes.is_some();
                    if !taken {
                        classes = Some(iris_of(ts));
                    }
                    (taken, "sh:class")
                }
                Component::Node(i) => {
                    let taken = nested.is_some();
                    if !taken {
                        nested = Some(*i);
                    }
                    (taken, "sh:node")
                }
                _ => (false, ""),
            };
            if slot_taken {
                return Err(LowerError::ConflictingValueType {
                    shape: disp,
                    first: kind,
                    second: kind,
                });
            }
        }
        // A literal-valued property cannot also be a node-valued one.
        if datatypes.is_some() {
            if let Some(second) = classes
                .is_some()
                .then_some("sh:class")
                .or_else(|| nested.is_some().then_some("sh:node"))
            {
                return Err(LowerError::ConflictingValueType {
                    shape: disp,
                    first: "sh:datatype",
                    second,
                });
            }
        }

        let value = if let Some(dts) = datatypes.filter(|d| !d.is_empty()) {
            ValueType::Scalar(scalar_type(dts))
        } else if let Some(i) = nested {
            let fallback_name = format!("{parent}{}", upper_camel_from_ident(&name));
            let fallback_display = format!("{disp} sh:node");
            let type_name = self.assign_name(i, Some((&fallback_name, &fallback_display)))?;
            let class_marker = match classes.filter(|c| !c.is_empty()) {
                Some(c) => Some(self.marker_for(c, &disp)?),
                None => None,
            };
            ValueType::Nested {
                type_name,
                class_marker,
            }
        } else if let Some(c) = classes.filter(|c| !c.is_empty()) {
            ValueType::Reference(self.marker_for(c, &disp)?)
        } else {
            ValueType::Term
        };

        Ok(Field {
            name,
            predicate,
            value,
            cardinality,
        })
    }
}

/// `sh:minCount` / `sh:maxCount` → a Rust field shape. Repeated constraints are
/// conjunctive: the effective minimum is the greatest `sh:minCount` and the
/// effective maximum the least `sh:maxCount`.
fn cardinality_of(shape: &Shape, disp: &str) -> Result<Cardinality, LowerError> {
    let mut min: Option<u64> = None;
    let mut max: Option<u64> = None;
    for component in &shape.components {
        match component {
            Component::MinCount(n) => min = Some(min.map_or(*n, |m: u64| m.max(*n))),
            Component::MaxCount(n) => max = Some(max.map_or(*n, |m: u64| m.min(*n))),
            _ => {}
        }
    }
    let effective_min = min.unwrap_or(0);
    if let Some(mx) = max {
        if effective_min > mx {
            return Err(LowerError::ContradictoryCardinality {
                shape: disp.to_string(),
                min: effective_min,
                max: mx,
            });
        }
    }
    Ok(match max {
        Some(1) if effective_min == 0 => Cardinality::Optional,
        Some(1) => Cardinality::Required,
        _ => Cardinality::Many {
            min: (effective_min > 0).then_some(effective_min),
            max,
        },
    })
}

fn scalar_type(datatypes: Vec<String>) -> ScalarType {
    let datatypes = sorted_dedup(datatypes);
    // A disjunctive `sh:datatype ( … )` set has no single Rust primitive, so the
    // lexical form is kept and every listed datatype is accepted.
    let rust = match datatypes.as_slice() {
        [one] => rust_scalar(one),
        _ => RustScalar::String,
    };
    ScalarType { rust, datatypes }
}

/// Only datatypes a Rust primitive represents WITHOUT loss get a primitive.
/// `xsd:decimal` (whose value space is not binary floating point) and the
/// unbounded / unsigned integer types deliberately keep their lexical form.
fn rust_scalar(datatype: &str) -> RustScalar {
    let Some(local) = datatype.strip_prefix(XSD) else {
        return RustScalar::String;
    };
    match local {
        "boolean" => RustScalar::Bool,
        "integer" | "int" | "long" | "short" | "byte" => RustScalar::I64,
        "float" => RustScalar::F32,
        "double" => RustScalar::F64,
        _ => RustScalar::String,
    }
}

fn iris_of(terms: &[Term]) -> Vec<String> {
    terms
        .iter()
        .filter_map(|t| match t {
            Term::NamedNode(n) => Some(n.as_str().to_string()),
            _ => None,
        })
        .collect()
}

fn sorted_dedup(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v.dedup();
    v
}

fn path_form(path: &Path) -> &'static str {
    match path {
        Path::Predicate(_) => "predicate",
        Path::Inverse(_) => "inverse",
        Path::Sequence(_) => "sequence",
        Path::Alternative(_) => "alternative",
        Path::ZeroOrMore(_) => "zero-or-more",
        Path::OneOrMore(_) => "one-or-more",
        Path::ZeroOrOne(_) => "zero-or-one",
    }
}

/// A stable display string for a shapes-graph node, used in diagnostics and as
/// the identity a Rust name is claimed by.
fn term_display(term: &Term) -> String {
    match term {
        Term::NamedNode(n) => format!("<{}>", n.as_str()),
        Term::BlankNode(b) => format!("_:{}", b.as_str()),
        other => other.to_string(),
    }
}

/// The local name of an IRI: everything after the last `#`, `/` or `:`.
fn local_name(iri: &str) -> &str {
    match iri.rfind(['#', '/', ':']) {
        Some(i) => &iri[i + 1..],
        None => iri,
    }
}

fn upper_camel_from_iri(iri: &str) -> Result<String, LowerError> {
    let name = upper_camel(local_name(iri));
    if name.is_empty() {
        return Err(LowerError::UnnameableIri {
            iri: iri.to_string(),
        });
    }
    Ok(name)
}

fn snake_from_iri(iri: &str) -> Result<String, LowerError> {
    let name = snake(local_name(iri));
    if name.is_empty() {
        return Err(LowerError::UnnameableIri {
            iri: iri.to_string(),
        });
    }
    Ok(escape_keyword(&name))
}

fn upper_camel_from_ident(snake_name: &str) -> String {
    upper_camel(snake_name.trim_end_matches('_'))
}

/// `person-shape` / `person_shape` / `PersonShape` → `PersonShape`. Existing
/// internal capitalisation is preserved so `ISBNShape` stays `ISBNShape`.
fn upper_camel(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut capitalise = true;
    for c in s.chars() {
        if !c.is_ascii_alphanumeric() {
            capitalise = true;
            continue;
        }
        if capitalise {
            out.extend(c.to_uppercase());
            capitalise = false;
        } else {
            out.push(c);
        }
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, 'N');
    }
    out
}

/// `firstName` / `first-name` → `first_name`; `ISBNCode` → `isbn_code`.
fn snake(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len() + 4);
    for (i, &c) in chars.iter().enumerate() {
        if !c.is_ascii_alphanumeric() {
            if !out.ends_with('_') && !out.is_empty() {
                out.push('_');
            }
            continue;
        }
        let prev = i.checked_sub(1).map(|p| chars[p]);
        let next = chars.get(i + 1).copied();
        let boundary = c.is_ascii_uppercase()
            && match (prev, next) {
                (Some(p), _) if p.is_ascii_lowercase() || p.is_ascii_digit() => true,
                // The tail of an acronym run: `ISBNCode` breaks before `Code`.
                (Some(p), Some(n)) if p.is_ascii_uppercase() && n.is_ascii_lowercase() => true,
                _ => false,
            };
        if boundary && !out.is_empty() && !out.ends_with('_') {
            out.push('_');
        }
        out.extend(c.to_lowercase());
    }
    let trimmed = out.trim_matches('_').to_string();
    if trimmed.starts_with(|c: char| c.is_ascii_digit()) {
        format!("n{trimmed}")
    } else {
        trimmed
    }
}

/// Rust keywords (2015/2018/2021 strict + reserved) a field name must not be.
const KEYWORDS: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
    "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "self", "static", "struct", "super", "trait", "true", "try", "type", "typeof",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

fn escape_keyword(name: &str) -> String {
    if KEYWORDS.contains(&name) {
        format!("{name}_")
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_derivation_is_total_and_stable() {
        assert_eq!(upper_camel("person-shape"), "PersonShape");
        assert_eq!(upper_camel("PersonShape"), "PersonShape");
        assert_eq!(upper_camel("2nd"), "N2nd");
        assert_eq!(snake("firstName"), "first_name");
        assert_eq!(snake("birth-date"), "birth_date");
        assert_eq!(snake("ISBNCode"), "isbn_code");
        assert_eq!(snake("ISBN"), "isbn");
        assert_eq!(escape_keyword("type"), "type_");
        assert_eq!(upper_camel_from_ident("first_name"), "FirstName");
    }

    #[test]
    fn local_name_splits_on_the_last_separator() {
        assert_eq!(local_name("http://example.org/ns#name"), "name");
        assert_eq!(local_name("http://example.org/ns/name"), "name");
        assert_eq!(local_name("urn:example:name"), "name");
        assert!(upper_camel_from_iri("http://example.org/ns#").is_err());
    }

    #[test]
    fn unknown_and_lossy_datatypes_keep_their_lexical_form() {
        assert_eq!(rust_scalar(&format!("{XSD}decimal")), RustScalar::String);
        assert_eq!(
            rust_scalar(&format!("{XSD}unsignedLong")),
            RustScalar::String
        );
        assert_eq!(rust_scalar(&format!("{XSD}integer")), RustScalar::I64);
        assert_eq!(rust_scalar(&format!("{XSD}double")), RustScalar::F64);
        assert_eq!(rust_scalar("http://example.org/dt"), RustScalar::String);
    }
}
