//! The Rust object-model IR that sits between SHACL shapes and Rust source.
//! [FABLE-5] (sq-1rg2q.12)
//!
//! [`lower`](crate::lower) builds a [`RustModel`] from a
//! [`sparq_shacl::ShapesModel`]; [`emit`](crate::emit) renders one to Rust
//! source. Splitting the two means the mapping can be asserted exactly in a
//! test without string-matching generated code, and the emitter is a pure
//! function of the IR (so the same shapes always produce byte-identical Rust).
//!
//! Everything here derives `PartialEq`/`Eq` so a test can pin the IR literally.

/// A complete Rust object model lowered from one shapes graph.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RustModel {
    /// The generated struct types, sorted by [`RustType::name`].
    pub types: Vec<RustType>,
    /// The typed reference newtypes generated for `sh:class` targets, sorted by
    /// [`RustReference::name`].
    pub references: Vec<RustReference>,
}

/// A typed reference newtype standing for "the IRI of an instance of `class`".
///
/// `sh:class ex:Organization` lowers to a field of type `OrganizationRef`
/// rather than a bare `String`, so a person's employer cannot be assigned a
/// project's IRI without a cast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustReference {
    /// The generated Rust type name (PascalCase local name + `Ref`).
    pub name: String,
    /// The `sh:class` IRI this reference is constrained to.
    pub class: String,
}

/// One generated struct: a node shape and its property shapes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustType {
    /// The generated Rust type name.
    pub name: String,
    /// The originating shape's IRI, or `None` for an anonymous (blank-node)
    /// shape reached only through `sh:node`.
    ///
    /// Anonymous shapes deliberately carry no label: a parser's blank-node
    /// identifiers are not stable across statement reorderings, so recording
    /// one would make the lowering — and the emitted source — depend on how the
    /// shapes graph happened to be serialised.
    pub shape: Option<String>,
    /// The shape's `sh:targetClass` / implicit-class IRIs, sorted and deduped.
    pub target_classes: Vec<String>,
    /// The struct's fields, sorted by [`RustField::predicate`].
    pub fields: Vec<RustField>,
    /// `Some` when the shape is `sh:closed true` — the predicate whitelist the
    /// generated loader enforces.
    pub closed: Option<ClosedShape>,
}

/// The static predicate whitelist of a `sh:closed true` node shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedShape {
    /// Every predicate the shape permits: its property-shape paths plus
    /// `sh:ignoredProperties`. Sorted and deduped.
    pub allowed: Vec<String>,
}

/// One generated struct field: a property shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustField {
    /// The generated Rust field name (snake_case local name).
    pub name: String,
    /// The property shape's `sh:path` predicate IRI.
    pub predicate: String,
    /// What a single value of this field is.
    pub value: ValueType,
    /// How many values the shape admits, and therefore how the field is wrapped.
    pub cardinality: Cardinality,
}

/// The Rust type of a *single* value of a field, before cardinality wrapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    /// `sh:datatype` — a checked scalar.
    Scalar(ScalarType),
    /// `sh:class` — a typed reference to another resource's IRI.
    Reference {
        /// The `sh:class` IRI.
        class: String,
        /// The generated [`RustReference::name`].
        name: String,
    },
    /// `sh:node` — a nested generated struct, named by [`RustType::name`].
    ///
    /// The emitter boxes nested values so mutually recursive shapes still
    /// produce a finitely sized (and therefore compilable) Rust type.
    Nested(String),
}

/// The checked XSD scalars this generator emits.
///
/// Deliberately small: a datatype is listed only when the generated loader can
/// actually *check* its lexical form. Anything else is
/// [`LoweringError::UnsupportedDatatype`](crate::LoweringError::UnsupportedDatatype)
/// rather than a silent fallback to `String`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScalarType {
    /// `xsd:string` → `String`. Every lexical form is valid.
    String,
    /// `xsd:boolean` → `bool`. Accepts `true`/`false`/`1`/`0`.
    Boolean,
    /// `xsd:integer` → `i64`.
    Integer,
    /// `xsd:decimal` → `f64`. No exponent, no `INF`/`NaN`.
    Decimal,
    /// `xsd:double` → `f64`. Exponent plus the `INF`/`-INF`/`NaN` spellings.
    Double,
    /// `xsd:dateTime` → `String`, with its lexical form checked.
    DateTime,
    /// `xsd:anyURI` → `String`. Every non-empty lexical form is accepted.
    AnyUri,
}

const XSD: &str = "http://www.w3.org/2001/XMLSchema#";

impl ScalarType {
    /// The XSD datatype IRI this scalar lowers from.
    #[must_use]
    pub fn datatype(self) -> String {
        format!("{}{}", XSD, self.datatype_local())
    }

    /// The local name of [`Self::datatype`].
    #[must_use]
    pub fn datatype_local(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::Decimal => "decimal",
            Self::Double => "double",
            Self::DateTime => "dateTime",
            Self::AnyUri => "anyURI",
        }
    }

    /// The Rust type the emitter spells for this scalar.
    #[must_use]
    pub fn rust_type(self) -> &'static str {
        match self {
            Self::String | Self::DateTime | Self::AnyUri => "String",
            Self::Boolean => "bool",
            Self::Integer => "i64",
            Self::Decimal | Self::Double => "f64",
        }
    }

    /// Resolves an XSD datatype IRI to its checked scalar, or `None` when this
    /// generator has no faithful mapping for it.
    #[must_use]
    pub fn from_datatype(iri: &str) -> Option<Self> {
        let local = iri.strip_prefix(XSD)?;
        [
            Self::String,
            Self::Boolean,
            Self::Integer,
            Self::Decimal,
            Self::Double,
            Self::DateTime,
            Self::AnyUri,
        ]
        .into_iter()
        .find(|s| s.datatype_local() == local)
    }
}

/// How many values a property shape admits, and therefore how the emitter wraps
/// the field's [`ValueType`].
///
/// The mapping is total and deterministic:
///
/// | `sh:minCount` | `sh:maxCount` | Rust                       |
/// |---------------|---------------|----------------------------|
/// | absent or `0` | `1`           | [`Optional`](Self::Optional) — `Option<T>` |
/// | `>= 1`        | `1`           | [`Required`](Self::Required) — `T`         |
/// | anything else | anything else | [`Many`](Self::Many) — `Vec<T>` + bounds check |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    /// At most one value: `Option<T>`.
    Optional,
    /// Exactly one value: `T`.
    Required,
    /// `Vec<T>`, with the declared bounds checked by the generated loader.
    Many {
        /// `sh:minCount`, defaulting to `0`.
        min: u64,
        /// `sh:maxCount`, `None` when unbounded.
        max: Option<u64>,
    },
}
