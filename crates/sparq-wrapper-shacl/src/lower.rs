//! Deterministic lowering of a `sparq_shacl::ShapesModel` to the object-model IR
//! (the `schema` module).
//!
//! # What is generated
//!
//! A struct is generated for every **root** node shape — a named, non-deactivated
//! shape with no `sh:path` and at least one `sh:property` child — plus the
//! transitive closure of the node shapes those structs reach through `sh:node`.
//! Inline operand shapes (`sh:not` / `sh:or` / `sh:qualifiedValueShape` / …) are
//! therefore not turned into types.
//!
//! # The mapping
//!
//! | shapes graph | IR |
//! |---|---|
//! | `sh:maxCount 1` with `sh:minCount 0`/absent | [`Cardinality::Optional`] (`Option<T>`) |
//! | `sh:minCount >= 1` with `sh:maxCount 1` | [`Cardinality::Required`] (`T`) |
//! | every other cardinality | [`Cardinality::Many`] (`Vec<T>`, surviving bounds checked) |
//! | `sh:datatype` | [`ValueSchema::Scalar`] — a checked scalar |
//! | `sh:class` | [`ValueSchema::Reference`] — a typed reference newtype |
//! | `sh:node` | [`ValueSchema::Nested`] — a nested generated struct |
//! | `sh:nodeKind sh:IRI` (alone) | [`ValueSchema::Iri`] |
//! | `sh:closed true` | [`ClosedSchema`] — a predicate whitelist |
//!
//! A `sh:datatype` lowers to a numeric Rust scalar only where that scalar carries
//! the datatype's **whole** XSD value space: `i64` for the bounded integer types,
//! `f64` for `xsd:double`/`xsd:float`, `bool` for `xsd:boolean`. The
//! arbitrary-precision and out-of-range families — `xsd:integer` and its
//! unbounded derivations, `xsd:unsignedLong` (maximum 2^64-1), `xsd:decimal` —
//! have no faithful std-only scalar, so they keep their exact lexical form as a
//! `String`. The generated loader checks the lexical against the specific
//! datatype's value space for EVERY kind, so it neither accepts
//! `-1`^^`xsd:nonNegativeInteger` nor rejects a conforming
//! `18446744073709551615`^^`xsd:unsignedLong`.
//!
//! Several property shapes on one predicate are conjoined (`min` = the largest
//! `sh:minCount`, `max` = the smallest `sh:maxCount`, datatype sets intersected);
//! an irreconcilable conjunction is a [`SchemaError`], never a silent drop.
//! Constraints outside the table (`sh:pattern`, `sh:minLength`, `sh:in`, …) do
//! not shape the Rust type and are left to `sparq-shacl` validation.
//!
//! # Determinism
//!
//! `ShapesModel::shapes` is in shapes-graph traversal order, which the IR must not
//! inherit. Roots are visited in IRI order, fields are keyed by predicate IRI in a
//! `BTreeMap`, every set is sorted, and the emitted struct/reference lists are
//! sorted by Rust type name — so lowering the same graph twice yields equal IRs.
//!
//! [FABLE-5] (sq-1rg2q.12)

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use oxrdf::Term;
use sparq_shacl::{Component, Path, ShapesModel};

use crate::error::SchemaError;
use crate::schema::{
    Cardinality, ClosedSchema, FieldSchema, ModelSchema, ReferenceSchema, ScalarKind, StructSchema,
    ValueSchema,
};

const XSD: &str = "http://www.w3.org/2001/XMLSchema#";
const SH_IRI: &str = "http://www.w3.org/ns/shacl#IRI";

/// Lowers a parsed shapes graph to the object-model IR.
///
/// # Errors
///
/// Returns a [`SchemaError`] when the shapes graph is ill-formed (the records
/// `ShapesModel::ill_formed` collected at parse), self-contradictory (e.g.
/// `sh:minCount` above `sh:maxCount`), or uses a construct that has no faithful
/// Rust counterpart (a non-predicate `sh:path`, `sh:closed sh:ByTypes`).
pub fn lower(model: &ShapesModel) -> Result<ModelSchema, SchemaError> {
    // An ill-formed construct is SKIPPED by validation but cannot be skipped by code
    // generation: the generated type would silently lack the constraint. Report the
    // first record (the vec is in parse order, which is stable for a given graph).
    if let Some(bad) = model.ill_formed().first() {
        return Err(SchemaError::IllFormedShapes {
            shape: term_label(&bad.node),
            predicate: bad.predicate.clone(),
            message: bad.message.clone(),
        });
    }
    Lowerer::new(model).run()
}

/// A value-shaping constraint gathered from the property shapes on one predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ValueSource {
    /// `sh:datatype` — a DISJUNCTIVE set of datatype IRIs.
    Datatype(BTreeSet<String>),
    /// `sh:class` — a DISJUNCTIVE set of class IRIs.
    Class(BTreeSet<String>),
    /// `sh:node` — an index into `ShapesModel::shapes`.
    Node(usize),
}

impl ValueSource {
    fn describe(&self, model: &ShapesModel) -> String {
        match self {
            ValueSource::Datatype(d) => format!("sh:datatype {{{}}}", join(d)),
            ValueSource::Class(c) => format!("sh:class {{{}}}", join(c)),
            ValueSource::Node(i) => format!("sh:node <{}>", term_label(&model.shapes[*i].node)),
        }
    }
}

fn join(set: &BTreeSet<String>) -> String {
    set.iter().cloned().collect::<Vec<_>>().join(", ")
}

struct Lowerer<'m> {
    model: &'m ShapesModel,
    /// Rust type name -> what claimed it (for collision reporting).
    taken: BTreeMap<String, String>,
    /// Shape index -> the Rust struct name assigned to it.
    named: BTreeMap<usize, String>,
    /// Rust newtype name -> its class set.
    references: BTreeMap<String, BTreeSet<String>>,
    /// Shape indices still to be turned into structs, with their assigned names.
    queue: VecDeque<(usize, String)>,
}

impl<'m> Lowerer<'m> {
    fn new(model: &'m ShapesModel) -> Self {
        Self {
            model,
            taken: BTreeMap::new(),
            named: BTreeMap::new(),
            references: BTreeMap::new(),
            queue: VecDeque::new(),
        }
    }

    fn run(mut self) -> Result<ModelSchema, SchemaError> {
        // Roots in IRI order — the one place shapes-graph order could leak in.
        let mut roots: Vec<(String, usize)> = Vec::new();
        for (idx, shape) in self.model.shapes.iter().enumerate() {
            if shape.path.is_some() || shape.deactivated || shape.property_children.is_empty() {
                continue;
            }
            if let Term::NamedNode(n) = &shape.node {
                roots.push((n.as_str().to_string(), idx));
            }
        }
        roots.sort();
        for (iri, idx) in roots {
            let proposed = struct_name_for_iri(&iri)?;
            self.claim(idx, proposed, format!("<{iri}>"))?;
        }

        let mut structs: Vec<StructSchema> = Vec::new();
        while let Some((idx, name)) = self.queue.pop_front() {
            structs.push(self.build_struct(idx, &name)?);
        }
        structs.sort_by(|a, b| a.name.cmp(&b.name));

        let references = self
            .references
            .into_iter()
            .map(|(name, classes)| ReferenceSchema {
                name,
                classes: classes.into_iter().collect(),
            })
            .collect();

        Ok(ModelSchema {
            structs,
            references,
        })
    }

    /// Assigns `proposed` to shape `idx` and queues it, or returns the name it
    /// already has. `source` describes the claimant for collision reporting.
    fn claim(&mut self, idx: usize, proposed: String, source: String) -> Result<String, SchemaError> {
        if let Some(existing) = self.named.get(&idx) {
            return Ok(existing.clone());
        }
        if let Some(other) = self.taken.get(&proposed) {
            let (first, second) = ordered(other.clone(), source);
            return Err(SchemaError::NameCollision {
                rust: proposed,
                first,
                second,
            });
        }
        self.taken.insert(proposed.clone(), source);
        self.named.insert(idx, proposed.clone());
        self.queue.push_back((idx, proposed.clone()));
        Ok(proposed)
    }

    fn build_struct(&mut self, idx: usize, name: &str) -> Result<StructSchema, SchemaError> {
        // Rebind the model behind its own lifetime so the shape borrow does not
        // conflict with the `&mut self` calls below.
        let model: &'m ShapesModel = self.model;
        let shape = &model.shapes[idx];
        let shape_label = term_label(&shape.node);

        // Group the property shapes by predicate; a BTreeMap fixes field order to
        // predicate-IRI order regardless of `property_children` order.
        let mut by_pred: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for &child in &shape.property_children {
            let ch = &model.shapes[child];
            if ch.deactivated {
                continue;
            }
            let Some(path) = ch.path.as_ref() else {
                return Err(SchemaError::UnsupportedPath {
                    shape: shape_label.clone(),
                    detail: "sh:property child without a sh:path".to_string(),
                });
            };
            let Path::Predicate(pred) = path else {
                return Err(SchemaError::UnsupportedPath {
                    shape: shape_label.clone(),
                    detail: describe_path(path),
                });
            };
            by_pred.entry(pred.clone()).or_default().push(child);
        }
        for children in by_pred.values_mut() {
            children.sort_unstable();
        }

        let mut fields: Vec<FieldSchema> = Vec::new();
        let mut field_names: BTreeMap<String, String> = BTreeMap::new();
        for (pred, children) in &by_pred {
            let field_name = field_name_for_iri(pred, &shape_label)?;
            if let Some(other) = field_names.get(&field_name) {
                let (first, second) = ordered(other.clone(), pred.clone());
                return Err(SchemaError::FieldCollision {
                    shape: shape_label.clone(),
                    field: field_name,
                    first,
                    second,
                });
            }
            field_names.insert(field_name.clone(), pred.clone());
            let field = self.build_field(&shape_label, name, pred, children, field_name)?;
            fields.push(field);
        }

        let closed = self.closed_for(shape_label.as_str(), idx, &fields)?;

        Ok(StructSchema {
            name: name.to_string(),
            shape: shape_label,
            fields,
            closed,
        })
    }

    fn build_field(
        &mut self,
        shape_label: &str,
        owner: &str,
        pred: &str,
        children: &[usize],
        field_name: String,
    ) -> Result<FieldSchema, SchemaError> {
        let model: &'m ShapesModel = self.model;
        let mut min: Option<u64> = None;
        let mut max: Option<u64> = None;
        let mut iri_only = false;
        let mut source: Option<ValueSource> = None;

        for &child in children {
            for component in &model.shapes[child].components {
                match component {
                    Component::MinCount(n) => min = Some(min.map_or(*n, |m: u64| m.max(*n))),
                    Component::MaxCount(n) => max = Some(max.map_or(*n, |m: u64| m.min(*n))),
                    Component::Datatype(d) => {
                        let set: BTreeSet<String> = d.iter().cloned().collect();
                        source = Some(merge_source(
                            source,
                            ValueSource::Datatype(set),
                            model,
                            shape_label,
                            pred,
                        )?);
                    }
                    Component::Class(c) => {
                        let set: BTreeSet<String> = [term_label(c)].into_iter().collect();
                        source = Some(merge_source(
                            source,
                            ValueSource::Class(set),
                            model,
                            shape_label,
                            pred,
                        )?);
                    }
                    Component::ClassIn(cs) => {
                        let set: BTreeSet<String> = cs.iter().map(term_label).collect();
                        source = Some(merge_source(
                            source,
                            ValueSource::Class(set),
                            model,
                            shape_label,
                            pred,
                        )?);
                    }
                    Component::Node(target) => {
                        source = Some(merge_source(
                            source,
                            ValueSource::Node(*target),
                            model,
                            shape_label,
                            pred,
                        )?);
                    }
                    Component::NodeKind(kinds) if kinds.len() == 1 && kinds[0] == SH_IRI => {
                        iri_only = true;
                    }
                    // Every other component constrains VALUES, not the Rust type;
                    // `sparq-shacl` validation remains the place those are enforced.
                    _ => {}
                }
            }
        }

        if let (Some(lo), Some(hi)) = (min, max) {
            if lo > hi {
                return Err(SchemaError::ContradictoryCardinality {
                    shape: shape_label.to_string(),
                    predicate: pred.to_string(),
                    min: lo,
                    max: hi,
                });
            }
        }
        let cardinality = match (min, max) {
            (m, Some(1)) if m.unwrap_or(0) == 0 => Cardinality::Optional,
            (Some(_), Some(1)) => Cardinality::Required,
            // `sh:minCount 0` is the absent default; normalise so the two spellings
            // produce one IR.
            _ => Cardinality::Many {
                min: min.filter(|m| *m > 0),
                max,
            },
        };

        let value = match source {
            None if iri_only => ValueSchema::Iri,
            None => ValueSchema::Term,
            Some(ValueSource::Datatype(set)) => {
                if set.is_empty() {
                    return Err(SchemaError::EmptyDatatypeSet {
                        shape: shape_label.to_string(),
                        predicate: pred.to_string(),
                    });
                }
                ValueSchema::Scalar {
                    kind: scalar_kind(&set),
                    datatypes: set.into_iter().collect(),
                }
            }
            Some(ValueSource::Class(set)) => {
                let rust = self.reference_name(&set)?;
                ValueSchema::Reference {
                    rust,
                    classes: set.into_iter().collect(),
                }
            }
            Some(ValueSource::Node(target)) => {
                let proposed = match &model.shapes[target].node {
                    Term::NamedNode(n) => struct_name_for_iri(n.as_str())?,
                    // An anonymous `sh:node [ … ]` has no name of its own; derive a
                    // stable one from the owner struct and the field it hangs off.
                    other => {
                        let derived = format!("{owner}{}", to_upper_camel(&field_name));
                        sanitise_type(derived, &term_label(other))?
                    }
                };
                let claim_source = format!("<{}>", term_label(&model.shapes[target].node));
                let rust = self.claim(target, proposed, claim_source)?;
                ValueSchema::Nested { rust }
            }
        };

        Ok(FieldSchema {
            name: field_name,
            predicate: pred.to_string(),
            cardinality,
            value,
        })
    }

    /// The predicate whitelist of an `sh:closed` node shape, if it is closed.
    fn closed_for(
        &self,
        shape_label: &str,
        idx: usize,
        fields: &[FieldSchema],
    ) -> Result<Option<ClosedSchema>, SchemaError> {
        let mut allowed: Option<BTreeSet<String>> = None;
        for component in &self.model.shapes[idx].components {
            let Component::Closed { ignored, by_types } = component else {
                continue;
            };
            if *by_types {
                return Err(SchemaError::ClosedByTypes {
                    shape: shape_label.to_string(),
                });
            }
            let set = allowed.get_or_insert_with(BTreeSet::new);
            for f in fields {
                set.insert(f.predicate.clone());
            }
            for term in ignored {
                match term {
                    Term::NamedNode(n) => {
                        set.insert(n.as_str().to_string());
                    }
                    other => {
                        return Err(SchemaError::NonIriIgnoredProperty {
                            shape: shape_label.to_string(),
                            term: term_label(other),
                        })
                    }
                }
            }
        }
        Ok(allowed.map(|allowed| ClosedSchema {
            allowed: allowed.into_iter().collect(),
        }))
    }

    /// Interns the newtype for a class set, reusing it when the same set recurs.
    fn reference_name(&mut self, classes: &BTreeSet<String>) -> Result<String, SchemaError> {
        let mut stem = String::new();
        for class in classes {
            stem.push_str(&to_upper_camel(local_name(class)));
        }
        let proposed = sanitise_type(format!("{stem}Ref"), &join(classes))?;
        let source = format!("sh:class {{{}}}", join(classes));
        match self.references.get(&proposed) {
            Some(existing) if existing == classes => return Ok(proposed),
            Some(existing) => {
                let (first, second) =
                    ordered(format!("sh:class {{{}}}", join(existing)), source);
                return Err(SchemaError::NameCollision {
                    rust: proposed,
                    first,
                    second,
                });
            }
            None => {}
        }
        if let Some(other) = self.taken.get(&proposed) {
            let (first, second) = ordered(other.clone(), source);
            return Err(SchemaError::NameCollision {
                rust: proposed,
                first,
                second,
            });
        }
        self.taken.insert(proposed.clone(), source);
        self.references.insert(proposed.clone(), classes.clone());
        Ok(proposed)
    }
}

/// Conjoins a newly-seen value constraint with what the predicate already had.
fn merge_source(
    have: Option<ValueSource>,
    new: ValueSource,
    model: &ShapesModel,
    shape: &str,
    pred: &str,
) -> Result<ValueSource, SchemaError> {
    let Some(have) = have else { return Ok(new) };
    match (&have, &new) {
        // Two `sh:datatype` constraints conjoin: a value must satisfy both, so the
        // allowed set is the INTERSECTION.
        (ValueSource::Datatype(a), ValueSource::Datatype(b)) => {
            let both: BTreeSet<String> = a.intersection(b).cloned().collect();
            if both.is_empty() {
                return Err(SchemaError::EmptyDatatypeSet {
                    shape: shape.to_string(),
                    predicate: pred.to_string(),
                });
            }
            Ok(ValueSource::Datatype(both))
        }
        _ if have == new => Ok(have),
        _ => {
            let (first, second) = ordered(have.describe(model), new.describe(model));
            Err(SchemaError::ConflictingValueTypes {
                shape: shape.to_string(),
                predicate: pred.to_string(),
                first,
                second,
            })
        }
    }
}

/// Orders two descriptions lexicographically so an error never depends on which
/// constraint the shapes-graph traversal reached first.
fn ordered(a: String, b: String) -> (String, String) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

fn term_label(term: &Term) -> String {
    match term {
        Term::NamedNode(n) => n.as_str().to_string(),
        Term::BlankNode(b) => format!("_:{}", b.as_str()),
        other => other.to_string(),
    }
}

fn describe_path(path: &Path) -> String {
    match path {
        Path::Predicate(p) => format!("<{p}>"),
        Path::Inverse(inner) => format!("^{}", describe_path(inner)),
        Path::Sequence(parts) => format!(
            "({})",
            parts.iter().map(describe_path).collect::<Vec<_>>().join(" / ")
        ),
        Path::Alternative(parts) => format!(
            "({})",
            parts.iter().map(describe_path).collect::<Vec<_>>().join(" | ")
        ),
        Path::ZeroOrMore(inner) => format!("{}*", describe_path(inner)),
        Path::OneOrMore(inner) => format!("{}+", describe_path(inner)),
        Path::ZeroOrOne(inner) => format!("{}?", describe_path(inner)),
    }
}

/// The scalar every datatype in the set maps to, or [`ScalarKind::Lexical`] when
/// they disagree (the datatype check still runs; the value is carried as text).
fn scalar_kind(datatypes: &BTreeSet<String>) -> ScalarKind {
    let mut kinds = datatypes.iter().map(|d| kind_of(d));
    let Some(first) = kinds.next() else {
        return ScalarKind::Lexical;
    };
    if kinds.all(|k| k == first) {
        first
    } else {
        ScalarKind::Lexical
    }
}

fn kind_of(datatype: &str) -> ScalarKind {
    let Some(local) = datatype.strip_prefix(XSD) else {
        return ScalarKind::Lexical;
    };
    match local {
        "boolean" => ScalarKind::Bool,
        // ONLY the integer types whose whole value space fits in `i64`. The
        // arbitrary-precision ones (`integer` and the four unbounded derived
        // types) and `unsignedLong` — whose maximum, 2^64-1, is above `i64::MAX`
        // — would have conforming values rejected by an `i64` parse, so they keep
        // their exact lexical form instead. The generated loader still checks
        // every one of them against its own XSD value space.
        "long" | "int" | "short" | "byte" | "unsignedInt" | "unsignedShort" | "unsignedByte" => {
            ScalarKind::I64
        }
        // IEEE binary64/binary32 — the only numeric families an `f64` carries
        // faithfully. `decimal` is arbitrary-precision (and excludes the special
        // values), so it is carried lexically for the same reason as `integer`.
        "double" | "float" => ScalarKind::F64,
        _ => ScalarKind::Lexical,
    }
}

fn local_name(iri: &str) -> &str {
    match iri.rfind(['#', '/', ':']) {
        Some(i) => &iri[i + 1..],
        None => iri,
    }
}

fn to_upper_camel(s: &str) -> String {
    let mut out = String::new();
    let mut upper = true;
    for ch in s.chars() {
        if ch.is_alphanumeric() {
            if upper {
                out.extend(ch.to_uppercase());
                upper = false;
            } else {
                out.push(ch);
            }
        } else {
            upper = true;
        }
    }
    out
}

fn to_snake(s: &str) -> String {
    let mut out = String::new();
    let mut prev_kept = false;
    for ch in s.chars() {
        if ch.is_uppercase() {
            if prev_kept && !out.ends_with('_') {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
            prev_kept = true;
        } else if ch.is_alphanumeric() {
            out.push(ch);
            prev_kept = true;
        } else {
            if prev_kept && !out.ends_with('_') {
                out.push('_');
            }
            prev_kept = false;
        }
    }
    out.trim_matches('_').to_string()
}

/// `ex:PersonShape` -> `Person`: the trailing `Shape` is noise on the Rust type,
/// but only when something is left after removing it.
fn struct_name_for_iri(iri: &str) -> Result<String, SchemaError> {
    let camel = to_upper_camel(local_name(iri));
    let trimmed = match camel.strip_suffix("Shape") {
        Some(rest) if !rest.is_empty() => rest.to_string(),
        _ => camel,
    };
    sanitise_type(trimmed, iri)
}

fn field_name_for_iri(iri: &str, shape: &str) -> Result<String, SchemaError> {
    let snake = to_snake(local_name(iri));
    if snake.is_empty() {
        return Err(SchemaError::Unnameable {
            shape: shape.to_string(),
            detail: format!("no Rust field name can be derived from predicate <{iri}>"),
        });
    }
    let snake = if snake.starts_with(|c: char| c.is_ascii_digit()) {
        format!("n{snake}")
    } else {
        snake
    };
    Ok(if is_keyword(&snake) {
        format!("{snake}_")
    } else {
        snake
    })
}

fn sanitise_type(name: String, source: &str) -> Result<String, SchemaError> {
    if name.is_empty() {
        return Err(SchemaError::Unnameable {
            shape: source.to_string(),
            detail: "no Rust type name can be derived from this shape".to_string(),
        });
    }
    let name = if name.starts_with(|c: char| c.is_ascii_digit()) {
        format!("N{name}")
    } else {
        name
    };
    Ok(if is_keyword(&name) {
        format!("{name}_")
    } else {
        name
    })
}

/// Rust keywords (2015/2018/2021 strict + reserved) that cannot be raw identifiers
/// in the positions this crate emits.
fn is_keyword(s: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "abstract", "as", "async", "await", "become", "box", "break", "const", "continue",
        "crate", "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if",
        "impl", "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv",
        "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true",
        "try", "type", "typeof", "unsafe", "unsized", "use", "virtual", "where", "while",
        "yield",
    ];
    KEYWORDS.contains(&s)
}
