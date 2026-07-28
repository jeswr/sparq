//! The object-model IR: what a shapes graph means as Rust types.
//!
//! [FABLE-5] (sq-1rg2q.12) [`lower`](crate::lower()) builds this,
//! [`emit`](crate::emit()) renders it, and tests assert on it directly —
//! keeping the "what does this SHACL mean" decision separate from the "how does
//! that read as Rust" decision.
//!
//! # Determinism
//!
//! Every collection in this IR is in a **total, content-derived order**:
//! [`ObjectModel::types`] and [`ObjectModel::class_markers`] are sorted by name,
//! [`ModelType::fields`] by predicate IRI, and every IRI list
//! ([`ModelType::target_classes`], [`ModelType::closed`], [`ScalarType::datatypes`],
//! [`ClassMarker::classes`]) is sorted and deduplicated. Nothing is carried over
//! in shapes-graph traversal order, so two runs over the same shapes produce an
//! identical IR — and therefore byte-identical Rust — regardless of how the
//! triples happened to hash.

/// A whole generated module: the types plus the class markers they reference.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ObjectModel {
    /// Generated structs, sorted by [`ModelType::name`].
    pub types: Vec<ModelType>,
    /// Class markers referenced by [`ValueType::Reference`] and
    /// [`ValueType::Nested`], sorted by [`ClassMarker::name`].
    pub class_markers: Vec<ClassMarker>,
}

/// One generated struct: a node shape rendered as a Rust type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelType {
    /// The Rust type name (UpperCamelCase).
    pub name: String,
    /// The shape's node in the shapes graph, as a stable display string (`<iri>`
    /// for a named shape, `_:label` for an inline one). Diagnostic only.
    pub shape: String,
    /// `sh:targetClass` / implicit-class IRIs, sorted and deduplicated. Empty
    /// when the shape declares no class target.
    pub target_classes: Vec<String>,
    /// Fields, sorted by [`Field::predicate`].
    pub fields: Vec<Field>,
    /// `Some(whitelist)` when the shape is `sh:closed true`: the sorted,
    /// deduplicated set of predicate IRIs a conforming node may carry (this
    /// shape's own `sh:property` predicate paths plus `sh:ignoredProperties`),
    /// matching the set `sparq-shacl` itself evaluates. `None` when open.
    pub closed: Option<Vec<String>>,
}

/// One generated struct field: a property shape rendered as a Rust field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// The Rust field name (snake_case, escaped if it is a keyword).
    pub name: String,
    /// The full predicate IRI of the shape's `sh:path`.
    pub predicate: String,
    /// What one value of this property is.
    pub value: ValueType,
    /// How many values there may be.
    pub cardinality: Cardinality,
}

/// How `sh:minCount` / `sh:maxCount` become a Rust field type.
///
/// The effective minimum is the **greatest** `sh:minCount` on the shape and the
/// effective maximum the **least** `sh:maxCount` (repeated constraints are
/// conjunctive in SHACL); `min > max` is a
/// [`ContradictoryCardinality`](crate::LowerError::ContradictoryCardinality)
/// error rather than an uninhabitable field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    /// `sh:maxCount 1` with no `sh:minCount` (or `sh:minCount 0`) — `Option<T>`.
    Optional,
    /// `sh:minCount >= 1` with `sh:maxCount 1` — a required, bare `T`.
    Required,
    /// Every other cardinality — `Vec<T>`, with the retained bounds checked at
    /// load time. `min`/`max` are `None` when unconstrained.
    Many {
        /// The effective `sh:minCount`, omitted when it is zero.
        min: Option<u64>,
        /// The effective `sh:maxCount`.
        max: Option<u64>,
    },
}

/// What one value of a field is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    /// `sh:datatype` — a checked scalar.
    Scalar(ScalarType),
    /// `sh:class` without `sh:node` — a typed reference: the IRI of a node whose
    /// `rdf:type` is checked against the named [`ClassMarker`].
    Reference(String),
    /// `sh:node` — a nested generated type, optionally also `sh:class`-checked.
    Nested {
        /// The nested [`ModelType::name`].
        type_name: String,
        /// The [`ClassMarker::name`] of an accompanying `sh:class`, if any.
        class_marker: Option<String>,
    },
    /// No value-typing component — the raw term, literal or node.
    Term,
}

/// A `sh:datatype` set and the Rust scalar it lowers to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScalarType {
    /// The Rust scalar the lexical form is parsed into.
    pub rust: RustScalar,
    /// The permitted datatype IRIs, sorted and deduplicated, always non-empty.
    /// More than one entry is SHACL 1.2's disjunctive
    /// `sh:datatype ( xsd:string rdf:langString )` form.
    pub datatypes: Vec<String>,
}

/// The Rust scalars a `sh:datatype` can lower to.
///
/// The mapping is deliberately narrow: only datatypes whose value space a Rust
/// primitive represents **without loss** get a primitive. Everything else —
/// including `xsd:decimal`, whose value space is not binary floating point, and
/// the unbounded/unsigned integer types (`xsd:integer` among them) — keeps its
/// lexical form as a [`RustScalar::String`], with the datatype still checked at
/// load time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustScalar {
    /// The literal's lexical form, verbatim.
    String,
    /// `xsd:boolean`.
    Bool,
    /// The fixed-width signed integer datatypes whose whole value space fits:
    /// `xsd:long`, `xsd:int`, `xsd:short`, `xsd:byte`. Unbounded `xsd:integer`
    /// is deliberately NOT one of them.
    I64,
    /// `xsd:float`.
    F32,
    /// `xsd:double`.
    F64,
}

impl RustScalar {
    /// The Rust type this scalar is written as.
    #[must_use]
    pub fn rust_type(self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Bool => "bool",
            Self::I64 => "i64",
            Self::F32 => "f32",
            Self::F64 => "f64",
        }
    }
}

/// A named set of classes a typed reference is checked against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassMarker {
    /// The Rust marker-struct name.
    pub name: String,
    /// The class IRIs, sorted and deduplicated, always non-empty. More than one
    /// entry is SHACL 1.2's disjunctive `sh:class ( ex:A ex:B )` form: a value
    /// conforms if it is an instance of **any** of them.
    pub classes: Vec<String>,
}
