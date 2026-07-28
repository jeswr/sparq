//! Typed lowering errors.
//!
//! [FABLE-5] (sq-1rg2q.12) Every way a shapes graph can fail to become a Rust
//! object model is one variant here — the generator never silently drops a
//! constraint it could not express, and never guesses. Two broad families:
//!
//! * **Ill-formed / contradictory** — the shapes graph violates a SHACL syntax
//!   rule ([`LowerError::IllFormedShape`], relayed from
//!   [`sparq_shacl::ShapesModel::ill_formed`]) or asks for something no value can
//!   satisfy ([`LowerError::ContradictoryCardinality`],
//!   [`LowerError::ConflictingValueType`]).
//! * **Out of scope for an object model** — well-formed SHACL whose meaning has
//!   no faithful struct-field rendering ([`LowerError::UnsupportedPath`],
//!   [`LowerError::UnsupportedComponent`]), or a naming clash the generator
//!   refuses to paper over ([`LowerError::DuplicateTypeName`],
//!   [`LowerError::ReservedName`], [`LowerError::UnnameableIri`]).
//!
//! Nothing here is a warning: `lower` returns `Err` rather than emitting Rust
//! that would misrepresent the shapes it came from.

use std::fmt;

/// Why a shapes graph could not be lowered to a Rust object model.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LowerError {
    /// The shapes graph carries a construct that violates a SHACL syntax rule.
    /// Relayed verbatim from [`sparq_shacl::ShapesModel::ill_formed`] — the
    /// generator refuses to build a model from a graph the SHACL parser already
    /// flagged. `node` is the offending shapes-graph node, `predicate` the full
    /// IRI of the SHACL predicate whose value is ill-formed.
    IllFormedShape {
        /// The shapes-graph node carrying the ill-formed construct.
        node: String,
        /// Full IRI of the SHACL predicate whose value is ill-formed.
        predicate: String,
        /// The SHACL parser's explanation of the violated syntax rule.
        message: String,
    },
    /// A property shape has no `sh:path`, so it names no predicate and cannot
    /// become a field.
    MissingPath {
        /// The property shape's node.
        shape: String,
    },
    /// A property shape's `sh:path` is well-formed SHACL but not a single
    /// predicate (a sequence, inverse, alternative or modified path). An object
    /// field is one predicate, so there is no faithful rendering; validate such
    /// shapes with `sparq-shacl` instead of generating for them.
    UnsupportedPath {
        /// The property shape's node.
        shape: String,
        /// Which path form was found.
        form: &'static str,
    },
    /// `sh:minCount` exceeds `sh:maxCount`, so no value count can conform.
    ContradictoryCardinality {
        /// The property shape's node.
        shape: String,
        /// The effective minimum (the greatest `sh:minCount`).
        min: u64,
        /// The effective maximum (the least `sh:maxCount`).
        max: u64,
    },
    /// Two value-typing components on one property shape disagree about what a
    /// value even is — e.g. `sh:datatype` (a literal) together with `sh:class`
    /// or `sh:node` (a node) — or the same kind appears twice with different
    /// arguments.
    ConflictingValueType {
        /// The property shape's node.
        shape: String,
        /// The first value-typing component found.
        first: &'static str,
        /// The conflicting component.
        second: &'static str,
    },
    /// A well-formed component whose meaning the object model cannot carry.
    /// Today this is only `sh:closed sh:ByTypes`, whose allowed-predicate set is
    /// recomputed per value node from its `rdf:type`s and so is not a static
    /// whitelist.
    UnsupportedComponent {
        /// The shape carrying the component.
        shape: String,
        /// The component that has no static rendering.
        component: &'static str,
    },
    /// An IRI has no local name that can seed a Rust identifier (it ends at its
    /// scheme separator, or its local name is entirely non-alphanumeric).
    UnnameableIri {
        /// The IRI that could not be turned into an identifier.
        iri: String,
    },
    /// Two distinct shapes (or a shape and a class marker) want the same Rust
    /// item name. The generator will not silently rename one of them.
    DuplicateTypeName {
        /// The contested Rust item name.
        name: String,
        /// The shapes-graph node that claimed the name first.
        first: String,
        /// The shapes-graph node that collided with it.
        second: String,
    },
    /// A generated name collides with an item the emitted prelude always
    /// defines (`Triple`, `LoadError`, …), or with `Self`, the one Rust keyword
    /// a generated item name can come out as. Rename the shape or the class.
    ReservedName {
        /// The contested Rust item name.
        name: String,
        /// The shapes-graph node that wanted it.
        source: String,
    },
    /// Two property shapes on the same node shape lower to the same field name.
    DuplicateFieldName {
        /// The Rust type whose fields collided.
        type_name: String,
        /// The contested field name.
        field: String,
        /// The predicate that claimed the field name first.
        first_predicate: String,
        /// The predicate that collided with it.
        second_predicate: String,
    },
}

impl fmt::Display for LowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IllFormedShape {
                node,
                predicate,
                message,
            } => write!(f, "ill-formed shapes graph: {node} <{predicate}>: {message}"),
            Self::MissingPath { shape } => {
                write!(f, "property shape {shape} has no sh:path")
            }
            Self::UnsupportedPath { shape, form } => write!(
                f,
                "property shape {shape} uses a {form} sh:path; an object field maps one predicate"
            ),
            Self::ContradictoryCardinality { shape, min, max } => write!(
                f,
                "property shape {shape} requires at least {min} and at most {max} values"
            ),
            Self::ConflictingValueType {
                shape,
                first,
                second,
            } => write!(
                f,
                "property shape {shape} combines {first} with {second}, which cannot describe one value type"
            ),
            Self::UnsupportedComponent { shape, component } => write!(
                f,
                "shape {shape} uses {component}, which has no static object-model rendering"
            ),
            Self::UnnameableIri { iri } => {
                write!(f, "<{iri}> has no local name usable as a Rust identifier")
            }
            Self::DuplicateTypeName {
                name,
                first,
                second,
            } => write!(
                f,
                "Rust name `{name}` is claimed by both {first} and {second}"
            ),
            Self::ReservedName { name, source } => write!(
                f,
                "{source} wants the Rust name `{name}`, which the generated module cannot use"
            ),
            Self::DuplicateFieldName {
                type_name,
                field,
                first_predicate,
                second_predicate,
            } => write!(
                f,
                "type `{type_name}` maps both <{first_predicate}> and <{second_predicate}> to field `{field}`"
            ),
        }
    }
}

impl std::error::Error for LowerError {}
