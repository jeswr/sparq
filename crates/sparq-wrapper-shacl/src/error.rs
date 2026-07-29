//! Typed errors for shapes graphs that cannot be lowered to a Rust object model.
//!
//! Lowering never panics and never silently drops a constraint it cannot model
//! faithfully: an ill-formed or self-contradictory shapes graph produces one of
//! these values instead.
//!
//! [FABLE-5] (sq-1rg2q.12)

use std::fmt;

/// Why a shapes graph could not be lowered to an object model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaError {
    /// The shapes graph itself is ill-formed — the record comes straight from
    /// `sparq_shacl::ShapesModel::ill_formed` (an unparsable `sh:path`, a
    /// non-integer `sh:minCount`, a malformed SHACL list, …). SHACL validation
    /// merely *skips* such a construct; code generation cannot, because the
    /// generated type would silently omit it.
    IllFormedShapes {
        shape: String,
        predicate: String,
        message: String,
    },
    /// A property shape's `sh:path` is not a single predicate, so it has no
    /// field to be generated onto.
    UnsupportedPath { shape: String, detail: String },
    /// `sh:minCount` exceeds `sh:maxCount` (after conjoining every property
    /// shape on the predicate) — no value set can satisfy the shape.
    ContradictoryCardinality {
        shape: String,
        predicate: String,
        min: u64,
        max: u64,
    },
    /// Two constraints on one predicate demand incompatible value types (e.g.
    /// `sh:datatype` beside `sh:class`, or two different `sh:node` shapes).
    /// `first`/`second` are ordered lexicographically so the error is
    /// independent of shapes-graph traversal order.
    ConflictingValueTypes {
        shape: String,
        predicate: String,
        first: String,
        second: String,
    },
    /// Conjoining several `sh:datatype` sets on one predicate left no datatype
    /// that satisfies all of them.
    EmptyDatatypeSet { shape: String, predicate: String },
    /// `sh:closed sh:ByTypes` computes its allowed-predicate set per value node
    /// from that node's `rdf:type`s, so there is no static whitelist to
    /// generate.
    ClosedByTypes { shape: String },
    /// An `sh:ignoredProperties` member is not an IRI.
    NonIriIgnoredProperty { shape: String, term: String },
    /// No Rust identifier could be derived for a shape or a property predicate.
    Unnameable { shape: String, detail: String },
    /// Two distinct shapes (or class sets) want the same Rust type name.
    NameCollision {
        rust: String,
        first: String,
        second: String,
    },
    /// Two distinct predicates on one node shape want the same Rust field name.
    FieldCollision {
        shape: String,
        field: String,
        first: String,
        second: String,
    },
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemaError::IllFormedShapes {
                shape,
                predicate,
                message,
            } => write!(f, "ill-formed shapes graph at <{shape}> {predicate}: {message}"),
            SchemaError::UnsupportedPath { shape, detail } => {
                write!(f, "shape <{shape}> has an unsupported sh:path: {detail}")
            }
            SchemaError::ContradictoryCardinality {
                shape,
                predicate,
                min,
                max,
            } => write!(
                f,
                "shape <{shape}> predicate <{predicate}>: sh:minCount {min} exceeds sh:maxCount {max}"
            ),
            SchemaError::ConflictingValueTypes {
                shape,
                predicate,
                first,
                second,
            } => write!(
                f,
                "shape <{shape}> predicate <{predicate}>: conflicting value types {first} and {second}"
            ),
            SchemaError::EmptyDatatypeSet { shape, predicate } => write!(
                f,
                "shape <{shape}> predicate <{predicate}>: sh:datatype sets have no datatype in common"
            ),
            SchemaError::ClosedByTypes { shape } => write!(
                f,
                "shape <{shape}> uses sh:closed sh:ByTypes, which has no static predicate whitelist"
            ),
            SchemaError::NonIriIgnoredProperty { shape, term } => write!(
                f,
                "shape <{shape}> has a non-IRI sh:ignoredProperties member: {term}"
            ),
            SchemaError::Unnameable { shape, detail } => {
                write!(f, "shape <{shape}>: {detail}")
            }
            SchemaError::NameCollision {
                rust,
                first,
                second,
            } => write!(
                f,
                "Rust type name `{rust}` is claimed by both {first} and {second}"
            ),
            SchemaError::FieldCollision {
                shape,
                field,
                first,
                second,
            } => write!(
                f,
                "shape <{shape}>: Rust field name `{field}` is claimed by both <{first}> and <{second}>"
            ),
        }
    }
}

impl std::error::Error for SchemaError {}
