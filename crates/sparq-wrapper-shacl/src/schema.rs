//! The object-model IR: the deterministic intermediate representation that sits
//! between a `sparq_shacl::ShapesModel` and emitted Rust.
//!
//! The IR is the *contract* of this crate: the `lower` module produces it, the
//! `emit` module consumes it, and both are pure functions of their input.
//! Every collection in it is sorted (structs and reference newtypes by Rust type
//! name, fields by predicate IRI, datatype/class/predicate sets lexicographically),
//! so two lowerings of the same shapes graph compare equal even though
//! `ShapesModel::shapes` itself is in graph-traversal order.
//!
//! [FABLE-5] (sq-1rg2q.12)

/// A complete generated object model.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModelSchema {
    /// Generated structs, sorted by [`StructSchema::name`].
    pub structs: Vec<StructSchema>,
    /// Generated typed-reference newtypes, sorted by [`ReferenceSchema::name`].
    pub references: Vec<ReferenceSchema>,
}

/// One generated struct: a SHACL node shape and its property shapes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructSchema {
    /// The Rust type name.
    pub name: String,
    /// The shape node this struct was lowered from (an IRI, or `_:label` for an
    /// anonymous `sh:node` shape).
    pub shape: String,
    /// Fields, sorted by [`FieldSchema::predicate`].
    pub fields: Vec<FieldSchema>,
    /// `Some` when the node shape is `sh:closed true` — the predicate whitelist
    /// the generated loader enforces.
    pub closed: Option<ClosedSchema>,
}

/// The predicate whitelist of an `sh:closed` node shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedSchema {
    /// Every predicate the focus node may carry: the shape's own property-shape
    /// paths unioned with its `sh:ignoredProperties`. Sorted and deduplicated.
    pub allowed: Vec<String>,
}

/// One generated field: a SHACL property shape (or the conjunction of several
/// property shapes that share one predicate path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSchema {
    /// The Rust field name.
    pub name: String,
    /// The `sh:path` predicate IRI.
    pub predicate: String,
    pub cardinality: Cardinality,
    pub value: ValueSchema,
}

/// How `sh:minCount` / `sh:maxCount` shape the field's Rust type.
///
/// This is the mapping the crate is specified around:
/// `maxCount = 1` with `minCount = 0` (or absent) is [`Optional`](Self::Optional),
/// `minCount >= 1` with `maxCount = 1` is [`Required`](Self::Required), and every
/// remaining cardinality is [`Many`](Self::Many) — a `Vec<T>` whose surviving
/// bounds the generated loader still checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    /// `Option<T>`.
    Optional,
    /// A bare `T`.
    Required,
    /// `Vec<T>`, with the retained bounds enforced at load time.
    Many {
        min: Option<u64>,
        max: Option<u64>,
    },
}

/// What a field's values are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueSchema {
    /// `sh:datatype` — a checked scalar. The loader rejects any value whose
    /// datatype IRI is outside `datatypes`, then parses the lexical form.
    Scalar {
        kind: ScalarKind,
        /// The allowed datatype IRIs, sorted and deduplicated. A single
        /// `sh:datatype` object is a singleton; the SHACL-1.2 disjunctive list
        /// form gives the multi-element set.
        datatypes: Vec<String>,
    },
    /// `sh:class` — a typed reference. `rust` names the generated newtype.
    Reference {
        rust: String,
        /// The allowed class IRIs, sorted and deduplicated.
        classes: Vec<String>,
    },
    /// `sh:node` — a nested generated struct. `rust` names it.
    Nested { rust: String },
    /// `sh:nodeKind sh:IRI` without a class — a bare IRI.
    Iri,
    /// No value-shaping constraint — any RDF term.
    Term,
}

/// The Rust scalar a checked `sh:datatype` field lowers to.
///
/// A datatype set whose members do not all map to the same scalar falls back to
/// [`Lexical`](Self::Lexical): the datatype check still runs, but the value is
/// carried as its lexical form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarKind {
    /// `String` (the lexical form of a well-known string datatype).
    Lexical,
    /// `i64`.
    I64,
    /// `f64`.
    F64,
    /// `bool`.
    Bool,
}

/// A generated typed-reference newtype: the target of one or more `sh:class`
/// constraints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceSchema {
    /// The Rust type name.
    pub name: String,
    /// The class IRIs a referent must be a SHACL instance of, sorted and
    /// deduplicated. Membership is `rdf:type`/`rdfs:subClassOf*`, matching
    /// SHACL's "SHACL instance of".
    pub classes: Vec<String>,
}

impl ScalarKind {
    /// The Rust type this scalar is emitted as.
    pub fn rust_type(self) -> &'static str {
        match self {
            ScalarKind::Lexical => "String",
            ScalarKind::I64 => "i64",
            ScalarKind::F64 => "f64",
            ScalarKind::Bool => "bool",
        }
    }
}

impl ValueSchema {
    /// The Rust type of ONE value of this field (before the cardinality wrapper).
    ///
    /// [`Nested`](Self::Nested) is boxed unconditionally: a node shape may
    /// reference itself directly or through a cycle, and a uniform `Box` keeps
    /// every generated struct finite-sized without the emitted layout depending on
    /// which cycles happen to exist.
    pub fn rust_type(&self) -> String {
        match self {
            ValueSchema::Scalar { kind, .. } => kind.rust_type().to_string(),
            ValueSchema::Reference { rust, .. } => rust.clone(),
            ValueSchema::Nested { rust } => format!("Box<{rust}>"),
            ValueSchema::Iri => "String".to_string(),
            ValueSchema::Term => "Value".to_string(),
        }
    }

    /// The Rust type of one value inside a `Vec` — as [`rust_type`](Self::rust_type),
    /// except a nested struct needs no `Box` (the `Vec` is already the indirection).
    pub fn rust_type_in_vec(&self) -> String {
        match self {
            ValueSchema::Nested { rust } => rust.clone(),
            other => other.rust_type(),
        }
    }
}

impl FieldSchema {
    /// The declared Rust type of the field, cardinality wrapper included.
    pub fn rust_type(&self) -> String {
        match self.cardinality {
            Cardinality::Optional => format!("Option<{}>", self.value.rust_type()),
            Cardinality::Required => self.value.rust_type(),
            Cardinality::Many { .. } => format!("Vec<{}>", self.value.rust_type_in_vec()),
        }
    }
}
