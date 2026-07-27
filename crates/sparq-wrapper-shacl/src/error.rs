//! Typed lowering errors. [FABLE-5] (sq-1rg2q.12)
//!
//! Every way a shapes graph can fail to describe a Rust object model is a
//! distinct variant here — the generator never guesses, and never silently
//! drops a constraint it cannot represent. Ill-formed shapes are rejected up
//! front from [`sparq_shacl::ShapesModel::ill_formed`] rather than re-detected,
//! so this crate does not carry a second SHACL syntax checker.

use std::fmt;

/// Why a shapes graph could not be lowered to a Rust object model.
///
/// The variants split into three families:
///
/// * **ill-formed** — the shapes graph violates the SHACL syntax rules
///   ([`Self::IllFormedShapes`]); reported by `sparq-shacl` itself.
/// * **contradictory** — the shape is well-formed SHACL but describes no
///   inhabitable Rust type ([`Self::ContradictoryCardinality`],
///   [`Self::ConflictingCardinality`], [`Self::ConflictingValueType`],
///   [`Self::MissingValueType`], [`Self::DuplicateField`],
///   [`Self::DuplicateType`]).
/// * **unrepresentable** — valid SHACL this generator deliberately declines to
///   approximate ([`Self::UnsupportedPath`], [`Self::UnsupportedDatatype`],
///   [`Self::AmbiguousValueType`], [`Self::UnsupportedClosedMode`],
///   [`Self::InvalidIdentifier`]).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LoweringError {
    /// The shapes graph carries constructs that violate the SHACL syntax rules.
    /// Each entry is a rendered `sparq_shacl::IllFormedConstruct`.
    IllFormedShapes {
        /// One rendered `node predicate: message` line per ill-formed construct.
        constructs: Vec<String>,
    },
    /// A property shape's `sh:path` is not a single predicate IRI. Sequence,
    /// inverse, alternative and the `*`/`+`/`?` modifiers have no faithful
    /// struct-field spelling, so they are refused rather than approximated.
    UnsupportedPath {
        /// The property shape's node.
        shape: String,
        /// Why the path cannot become a field.
        detail: String,
    },
    /// A property shape declares none of `sh:datatype`, `sh:class` or `sh:node`,
    /// so the field has no Rust type.
    MissingValueType {
        /// The property shape's node.
        shape: String,
        /// The property shape's predicate.
        predicate: String,
    },
    /// A property shape declares more than one of `sh:datatype` / `sh:class` /
    /// `sh:node`; a single field cannot be all of them.
    ConflictingValueType {
        /// The property shape's node.
        shape: String,
        /// The property shape's predicate.
        predicate: String,
        /// The SHACL keyword seen first (e.g. `sh:datatype`).
        first: String,
        /// The conflicting SHACL keyword.
        second: String,
    },
    /// A disjunctive value-type spelling (`sh:datatype ( a b )`, `sh:class ( A B )`)
    /// admits several Rust types; the generator will not pick one.
    AmbiguousValueType {
        /// The property shape's node.
        shape: String,
        /// The property shape's predicate.
        predicate: String,
        /// The SHACL keyword carrying the list.
        keyword: String,
        /// The listed alternatives, in shapes-graph order.
        values: Vec<String>,
    },
    /// `sh:datatype` names an XSD datatype this generator has no checked scalar
    /// for. See [`crate::ScalarType`] for the supported set.
    UnsupportedDatatype {
        /// The property shape's node.
        shape: String,
        /// The property shape's predicate.
        predicate: String,
        /// The unsupported datatype IRI.
        datatype: String,
    },
    /// `sh:minCount` exceeds `sh:maxCount`: no value set satisfies the shape.
    ContradictoryCardinality {
        /// The property shape's node.
        shape: String,
        /// The property shape's predicate.
        predicate: String,
        /// The declared `sh:minCount`.
        min: u64,
        /// The declared `sh:maxCount`.
        max: u64,
    },
    /// The same count keyword is declared twice with different values, so the
    /// lowering is not determined by the shape.
    ConflictingCardinality {
        /// The property shape's node.
        shape: String,
        /// The property shape's predicate.
        predicate: String,
        /// `sh:minCount` or `sh:maxCount`.
        keyword: String,
        /// The first declared value.
        first: u64,
        /// The second, differing declared value.
        second: u64,
    },
    /// `sh:closed sh:ByTypes` computes its allowed-predicate set per value node
    /// from the data graph, so there is no static whitelist to emit.
    UnsupportedClosedMode {
        /// The node shape's node.
        shape: String,
    },
    /// An IRI's local name does not yield a Rust identifier.
    InvalidIdentifier {
        /// The IRI whose local name was rejected.
        iri: String,
        /// Why no identifier could be derived.
        reason: String,
    },
    /// Two property shapes of one node shape derive the same Rust field name.
    DuplicateField {
        /// The node shape's node.
        shape: String,
        /// The colliding field name.
        field: String,
        /// The predicate that claimed the name first.
        first_predicate: String,
        /// The predicate that collided with it.
        second_predicate: String,
    },
    /// Two shapes derive the same Rust type name.
    DuplicateType {
        /// The colliding type name.
        name: String,
        /// The shape that claimed the name first.
        first_shape: String,
        /// The shape that collided with it.
        second_shape: String,
    },
}

impl fmt::Display for LoweringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IllFormedShapes { constructs } => write!(
                f,
                "shapes graph is ill-formed ({} construct(s)): {}",
                constructs.len(),
                constructs.join("; ")
            ),
            Self::UnsupportedPath { shape, detail } => {
                write!(f, "property shape {}: unsupported sh:path ({})", shape, detail)
            }
            Self::MissingValueType { shape, predicate } => write!(
                f,
                "property shape {} on <{}>: none of sh:datatype / sh:class / sh:node declared",
                shape, predicate
            ),
            Self::ConflictingValueType {
                shape,
                predicate,
                first,
                second,
            } => write!(
                f,
                "property shape {} on <{}>: conflicting value types {} and {}",
                shape, predicate, first, second
            ),
            Self::AmbiguousValueType {
                shape,
                predicate,
                keyword,
                values,
            } => write!(
                f,
                "property shape {} on <{}>: {} lists {} alternatives ({}) — no single Rust type",
                shape,
                predicate,
                keyword,
                values.len(),
                values.join(", ")
            ),
            Self::UnsupportedDatatype {
                shape,
                predicate,
                datatype,
            } => write!(
                f,
                "property shape {} on <{}>: no checked Rust scalar for datatype <{}>",
                shape, predicate, datatype
            ),
            Self::ContradictoryCardinality {
                shape,
                predicate,
                min,
                max,
            } => write!(
                f,
                "property shape {} on <{}>: sh:minCount {} exceeds sh:maxCount {}",
                shape, predicate, min, max
            ),
            Self::ConflictingCardinality {
                shape,
                predicate,
                keyword,
                first,
                second,
            } => write!(
                f,
                "property shape {} on <{}>: {} declared twice as {} and {}",
                shape, predicate, keyword, first, second
            ),
            Self::UnsupportedClosedMode { shape } => write!(
                f,
                "node shape {}: sh:closed sh:ByTypes has no static predicate whitelist",
                shape
            ),
            Self::InvalidIdentifier { iri, reason } => {
                write!(f, "cannot derive a Rust identifier from <{}>: {}", iri, reason)
            }
            Self::DuplicateField {
                shape,
                field,
                first_predicate,
                second_predicate,
            } => write!(
                f,
                "node shape {}: <{}> and <{}> both map to field `{}`",
                shape, first_predicate, second_predicate, field
            ),
            Self::DuplicateType {
                name,
                first_shape,
                second_shape,
            } => write!(
                f,
                "shapes {} and {} both map to Rust type `{}`",
                first_shape, second_shape, name
            ),
        }
    }
}

impl std::error::Error for LoweringError {}
