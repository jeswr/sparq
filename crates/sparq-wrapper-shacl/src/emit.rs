//! [`RustModel`] → Rust source. [FABLE-5] (sq-1rg2q.12)
//!
//! [`emit`] is a pure function of the IR, so the same shapes graph always
//! produces byte-identical source — the property a checked-in generated module
//! needs if `git diff` is to stay empty across regenerations.
//!
//! ## What the emitted module depends on
//!
//! Nothing. The generated source is `std`-only: it declares its own
//! `ValueSource` trait, its own `LoadError`, its own exact numeric types, and
//! its own lexical checkers, so it compiles standalone with `rustc` and a
//! consumer can adapt any RDF source to it (a `sparq-wrapper` focus, a
//! `sparq-core` graph, a fixture) by implementing the `ValueSource` trait.
//!
//! Write the output to its own file and declare it with `mod` — it opens with a
//! module-level `#![allow(dead_code)]`, so `include!`ing it mid-file will not
//! compile.

use std::fmt::Write as _;

use crate::lower::SUBJECT_FIELD;
use crate::schema::{Cardinality, RustField, RustModel, RustType, ScalarType, ValueType};

/// Renders `model` as a self-contained Rust module.
#[must_use]
pub fn emit(model: &RustModel) -> String {
    let mut out = String::new();
    out.push_str(PRELUDE);

    for reference in &model.references {
        let _ = write!(
            out,
            "\n/// A typed reference to the IRI of an instance of `<{}>`.\n\
             ///\n\
             /// Constructing one through a generated `load` checks membership —\n\
             /// not just that the term is an IRI — via\n\
             /// [`ValueSource::is_instance_of`]. Constructing one directly\n\
             /// asserts membership without checking it.\n\
             #[derive(Debug, Clone, PartialEq, Eq, Hash)]\n\
             pub struct {}(pub String);\n\
             \n\
             impl {} {{\n    \
                 /// The `sh:class` this reference is constrained to.\n    \
                 pub const CLASS: &'static str = {};\n\
             }}\n",
            reference.class,
            reference.name,
            reference.name,
            quote(&reference.class),
        );
    }

    for ty in &model.types {
        emit_type(&mut out, ty);
    }
    out
}

fn emit_type(out: &mut String, ty: &RustType) {
    let _ = write!(out, "\n/// Generated from ");
    match &ty.shape {
        Some(iri) => {
            let _ = writeln!(out, "the node shape `<{}>`.", iri);
        }
        None => {
            let _ = writeln!(out, "an anonymous `sh:node` shape.");
        }
    }
    let _ = writeln!(out, "#[derive(Debug, Clone, PartialEq)]");
    let _ = writeln!(out, "pub struct {} {{", ty.name);
    let _ = writeln!(
        out,
        "    /// The subject this value was loaded from.\n    pub {}: String,",
        SUBJECT_FIELD
    );
    for field in &ty.fields {
        let _ = writeln!(out, "    /// `<{}>`", field.predicate);
        let _ = writeln!(out, "    pub {}: {},", field.name, field_type(field));
    }
    let _ = writeln!(out, "}}\n");

    let _ = writeln!(out, "impl {} {{", ty.name);
    if let Some(iri) = &ty.shape {
        let _ = writeln!(
            out,
            "    /// The shape this type was generated from.\n    \
             pub const SHAPE: &'static str = {};",
            quote(iri)
        );
    }
    let _ = writeln!(
        out,
        "    /// The shape's `sh:targetClass` IRIs.\n    \
         pub const TARGET_CLASSES: &'static [&'static str] = &[{}];",
        join_quoted(&ty.target_classes)
    );
    if let Some(closed) = &ty.closed {
        let _ = writeln!(
            out,
            "    /// The `sh:closed true` predicate whitelist: [`Self::load`] rejects\n    \
             /// any other predicate on the subject.\n    \
             pub const ALLOWED_PREDICATES: &'static [&'static str] = &[{}];",
            join_quoted(&closed.allowed)
        );
    }

    let _ = writeln!(
        out,
        "\n    /// Loads `subject` from `source`, checking every cardinality,\n    \
         /// datatype and `sh:class` membership the shape declares.\n    \
         ///\n    \
         /// # Errors\n    \
         ///\n    \
         /// Returns a [`LoadError`] when the subject does not satisfy the shape.\n    \
         pub fn load<S: ValueSource + ?Sized>(source: &S, subject: &str) -> Result<Self, LoadError> {{"
    );
    if ty.closed.is_some() {
        let _ = writeln!(
            out,
            "        for predicate in source.predicates(subject) {{\n            \
                 if !Self::ALLOWED_PREDICATES.contains(&predicate.as_str()) {{\n                \
                     return Err(LoadError::ClosedShape {{ type_name: {}, predicate }});\n            \
                 }}\n        \
             }}",
            quote(&ty.name)
        );
    }
    for field in &ty.fields {
        emit_field_load(out, &ty.name, field);
    }
    let _ = writeln!(
        out,
        "        Ok(Self {{ {}: subject.to_string(){} }})",
        SUBJECT_FIELD,
        ty.fields
            .iter()
            .map(|f| format!(", {}", f.name))
            .collect::<String>()
    );
    let _ = writeln!(out, "    }}\n}}");
}

/// The Rust spelling of one field, after cardinality wrapping.
///
/// Nested values are boxed: two shapes that reference each other through
/// `sh:node` would otherwise lower to infinitely sized Rust types.
fn field_type(field: &RustField) -> String {
    let base = match &field.value {
        ValueType::Scalar(scalar) => scalar.rust_type().to_string(),
        ValueType::Reference { name, .. } | ValueType::Nested(name) => name.clone(),
    };
    let nested = matches!(field.value, ValueType::Nested(_));
    match field.cardinality {
        Cardinality::Required if nested => format!("Box<{}>", base),
        Cardinality::Required => base,
        Cardinality::Optional if nested => format!("Option<Box<{}>>", base),
        Cardinality::Optional => format!("Option<{}>", base),
        Cardinality::Many { .. } => format!("Vec<{}>", base),
    }
}

/// The expression converting the RDF term bound to `raw` into one value of
/// `field`'s type, short-circuiting with `?`/`return` on failure.
///
/// Every arm checks the term KIND (and, for `sh:datatype`, the exact datatype)
/// before it looks at any lexical form: `sh:datatype xsd:integer` constrains the
/// term, so an IRI spelled `42` and an `xsd:string` literal `"42"` must both be
/// rejected even though their lexical forms parse.
///
/// A `sh:class` arm then checks CLASS MEMBERSHIP, because that — not IRI-ness —
/// is what `sh:class` requires of a value node. The newtype records the
/// constraint; only the `is_instance_of` call enforces it.
fn convert(type_name: &str, field: &RustField) -> String {
    match &field.value {
        ValueType::Scalar(scalar) => {
            let datatype = quote(&scalar.datatype());
            format!(
                "match into_literal(raw, {}) {{ \
                 Ok(lexical) => match {}(&lexical) {{ \
                 Some(value) => value, \
                 None => return Err(LoadError::Datatype {{ type_name: {}, field: {}, \
                 datatype: {}, value: lexical }}) }}, \
                 Err(found) => return Err(LoadError::TermType {{ type_name: {}, field: {}, \
                 expected: {}, found }}) }}",
                datatype,
                checker(*scalar),
                quote(type_name),
                quote(&field.name),
                datatype,
                quote(type_name),
                quote(&field.name),
                datatype,
            )
        }
        ValueType::Reference { name, .. } => format!(
            "match into_iri(raw) {{ \
             Ok(iri) => if source.is_instance_of(&iri, {}::CLASS) {{ {}(iri) }} else {{ \
             return Err(LoadError::ClassMembership {{ type_name: {}, field: {}, \
             class: {}::CLASS, node: iri }}) }}, \
             Err(found) => return Err(LoadError::TermType {{ type_name: {}, field: {}, \
             expected: \"an IRI\", found }}) }}",
            name,
            name,
            quote(type_name),
            quote(&field.name),
            name,
            quote(type_name),
            quote(&field.name),
        ),
        ValueType::Nested(name) => format!(
            "match into_subject(raw) {{ Ok(node) => {}::load(source, &node)?, \
             Err(found) => return Err(LoadError::TermType {{ type_name: {}, field: {}, \
             expected: \"an IRI or a blank node\", found }}) }}",
            name,
            quote(type_name),
            quote(&field.name),
        ),
    }
}

/// Boxes a nested conversion for the single-valued cardinalities.
fn convert_boxed(type_name: &str, field: &RustField) -> String {
    let inner = convert(type_name, field);
    if matches!(field.value, ValueType::Nested(_)) {
        format!("Box::new({})", inner)
    } else {
        inner
    }
}

fn cardinality_error(type_name: &str, field: &RustField, min: u64, max: Option<u64>) -> String {
    format!(
        "return Err(LoadError::Cardinality {{ type_name: {}, field: {}, predicate: {}, \
         found: values.len(), min: {}, max: {} }});",
        quote(type_name),
        quote(&field.name),
        quote(&field.predicate),
        min,
        match max {
            Some(n) => format!("Some({})", n),
            None => "None".to_string(),
        },
    )
}

fn emit_field_load(out: &mut String, type_name: &str, field: &RustField) {
    let _ = writeln!(out, "        let {} = {{", field.name);
    match field.cardinality {
        Cardinality::Required => {
            let _ = writeln!(
                out,
                "            let mut values = source.values(subject, {});",
                quote(&field.predicate)
            );
            let _ = writeln!(
                out,
                "            if values.len() != 1 {{ {} }}",
                cardinality_error(type_name, field, 1, Some(1))
            );
            let _ = writeln!(out, "            let raw = values.remove(0);");
            let _ = writeln!(out, "            {}", convert_boxed(type_name, field));
        }
        Cardinality::Optional => {
            let _ = writeln!(
                out,
                "            let mut values = source.values(subject, {});",
                quote(&field.predicate)
            );
            let _ = writeln!(
                out,
                "            if values.len() > 1 {{ {} }}",
                cardinality_error(type_name, field, 0, Some(1))
            );
            let _ = writeln!(out, "            if values.is_empty() {{");
            let _ = writeln!(out, "                None");
            let _ = writeln!(out, "            }} else {{");
            let _ = writeln!(out, "                let raw = values.remove(0);");
            let _ = writeln!(
                out,
                "                Some({})",
                convert_boxed(type_name, field)
            );
            let _ = writeln!(out, "            }}");
        }
        Cardinality::Many { min, max } => {
            let _ = writeln!(
                out,
                "            let values = source.values(subject, {});",
                quote(&field.predicate)
            );
            // `values.len() < 0` is vacuous, and rustc's `unused_comparisons`
            // would reject the emitted module under `-D warnings`.
            if min > 0 {
                let _ = writeln!(
                    out,
                    "            if values.len() < {} {{ {} }}",
                    min,
                    cardinality_error(type_name, field, min, max)
                );
            }
            if let Some(max) = max {
                let _ = writeln!(
                    out,
                    "            if values.len() > {} {{ {} }}",
                    max,
                    cardinality_error(type_name, field, min, Some(max))
                );
            }
            let _ = writeln!(out, "            let mut items = Vec::new();");
            let _ = writeln!(out, "            for raw in values {{");
            let _ = writeln!(out, "                items.push({});", convert(type_name, field));
            let _ = writeln!(out, "            }}");
            let _ = writeln!(out, "            items");
        }
    }
    let _ = writeln!(out, "        }};");
}

fn checker(scalar: ScalarType) -> &'static str {
    match scalar {
        ScalarType::String => "check_string",
        ScalarType::Boolean => "check_boolean",
        ScalarType::Integer => "check_integer",
        ScalarType::Decimal => "check_decimal",
        ScalarType::Double => "check_double",
        ScalarType::DateTime => "check_date_time",
        ScalarType::AnyUri => "check_any_uri",
    }
}

/// Renders `value` as a Rust string literal.
fn quote(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn join_quoted(values: &[String]) -> String {
    values
        .iter()
        .map(String::as_str)
        .map(quote)
        .collect::<Vec<_>>()
        .join(", ")
}

/// The fixed head of every emitted module: the source trait, the load error and
/// the XSD lexical checkers. Emitted verbatim so the output stays a pure
/// function of the IR.
const PRELUDE: &str = r##"// @generated by sparq-wrapper-shacl from a SHACL shapes graph — DO NOT EDIT.
//
// Regenerate from the shapes graph with `sparq-wrapper-shacl`. This module is
// std-only: adapt any RDF source by implementing `ValueSource`.
#![allow(dead_code)]

use std::fmt;

/// One RDF term, bound as the object of a triple.
///
/// The term KIND and the literal's datatype are carried through rather than
/// flattened to a lexical string, because that is what `sh:datatype` and
/// `sh:class` constrain. A loader handed only `"42"` could not tell a
/// conforming `"42"^^xsd:integer` from an `xsd:string` literal or from an IRI
/// that happens to spell the same characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// An IRI.
    NamedNode(String),
    /// A blank node, carrying the label the source identifies it by.
    BlankNode(String),
    /// A literal.
    Literal {
        /// The lexical form.
        lexical: String,
        /// The datatype IRI — `xsd:string` for a plain literal, and
        /// `rdf:langString` when `language` is set.
        datatype: String,
        /// The language tag of an `rdf:langString`.
        language: Option<String>,
    },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NamedNode(iri) => write!(f, "<{}>", iri),
            Self::BlankNode(label) => write!(f, "_:{}", label),
            Self::Literal { lexical, language: Some(tag), .. } => {
                write!(f, "{:?}@{}", lexical, tag)
            }
            Self::Literal { lexical, datatype, .. } => write!(f, "{:?}^^<{}>", lexical, datatype),
        }
    }
}

/// The subject-oriented view of an RDF source the generated loaders read.
pub trait ValueSource {
    /// Every predicate asserted on `subject`. Used by `sh:closed` shapes.
    fn predicates(&self, subject: &str) -> Vec<String>;
    /// Every object of `(subject, predicate)`, as RDF terms.
    fn values(&self, subject: &str, predicate: &str) -> Vec<Value>;
    /// Whether `node` is a SHACL instance of `class` — the `sh:class` test.
    ///
    /// The required semantics are SHACL's: answer `true` exactly when the data
    /// graph contains `node rdf:type/rdfs:subClassOf* class`, i.e. an
    /// `rdf:type` reached directly OR through any number of `rdfs:subClassOf`
    /// steps. Nothing else is entailed — this is not full RDFS.
    ///
    /// The SOURCE owns the entailment because only it knows which graph is
    /// being read and how much of the subclass closure it has materialised; the
    /// loader never re-derives it. That makes this method load-bearing: a
    /// `sh:class` field is exactly as sound as the implementation here, and one
    /// that returns `true` unconditionally turns every typed reference back
    /// into an unchecked IRI.
    fn is_instance_of(&self, node: &str, class: &str) -> bool;
}

/// Why a subject did not satisfy the shape its generated type came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// The number of values is outside the shape's `sh:minCount`/`sh:maxCount`.
    Cardinality {
        /// The generated type being loaded.
        type_name: &'static str,
        /// The field being loaded.
        field: &'static str,
        /// The field's predicate IRI.
        predicate: &'static str,
        /// How many values the source held.
        found: usize,
        /// The shape's `sh:minCount`, defaulting to 0.
        min: u64,
        /// The shape's `sh:maxCount`, `None` when unbounded.
        max: Option<u64>,
    },
    /// A value's RDF term kind — or its literal datatype — is not what the
    /// shape declares. Checked before the lexical form: `sh:datatype` and
    /// `sh:class` constrain the term, not the characters it spells.
    TermType {
        /// The generated type being loaded.
        type_name: &'static str,
        /// The field being loaded.
        field: &'static str,
        /// What the shape requires: the `sh:datatype` IRI, or the term kind a
        /// `sh:class` / `sh:node` value must have.
        expected: &'static str,
        /// The term found instead.
        found: Value,
    },
    /// A correctly typed literal's lexical form is not valid for the field's
    /// `sh:datatype`.
    Datatype {
        /// The generated type being loaded.
        type_name: &'static str,
        /// The field being loaded.
        field: &'static str,
        /// The expected XSD datatype IRI.
        datatype: &'static str,
        /// The lexical form that failed the check.
        value: String,
    },
    /// A `sh:class` value is an IRI, but not a SHACL instance of the class the
    /// shape declares — as `ValueSource::is_instance_of` decides.
    ClassMembership {
        /// The generated type being loaded.
        type_name: &'static str,
        /// The field being loaded.
        field: &'static str,
        /// The `sh:class` IRI the value must be an instance of.
        class: &'static str,
        /// The IRI that is not one.
        node: String,
    },
    /// The subject carries a predicate a `sh:closed true` shape does not allow.
    ClosedShape {
        /// The generated type being loaded.
        type_name: &'static str,
        /// The predicate outside the shape's whitelist.
        predicate: String,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cardinality { type_name, field, predicate, found, min, max } => write!(
                f,
                "{}.{} (<{}>): {} value(s) outside [{}, {}]",
                type_name,
                field,
                predicate,
                found,
                min,
                match max { Some(n) => n.to_string(), None => "*".to_string() },
            ),
            Self::TermType { type_name, field, expected, found } => write!(
                f,
                "{}.{}: {} does not satisfy {}",
                type_name, field, found, expected,
            ),
            Self::Datatype { type_name, field, datatype, value } => write!(
                f,
                "{}.{}: {:?} is not a valid <{}>",
                type_name, field, value, datatype,
            ),
            Self::ClassMembership { type_name, field, class, node } => write!(
                f,
                "{}.{}: <{}> is not an instance of <{}>",
                type_name, field, node, class,
            ),
            Self::ClosedShape { type_name, predicate } => write!(
                f,
                "{}: <{}> is not allowed by the closed shape",
                type_name, predicate,
            ),
        }
    }
}

impl std::error::Error for LoadError {}

/// The lexical form of `value`, when it is a literal of exactly `datatype` —
/// the `sh:datatype` term check no lexical checker can make. Hands the term back
/// unchanged when it is not, so the caller can report what it actually found.
fn into_literal(value: Value, datatype: &str) -> Result<String, Value> {
    match value {
        Value::Literal { lexical, datatype: found, language } => {
            if found == datatype && language.is_none() {
                Ok(lexical)
            } else {
                Err(Value::Literal { lexical, datatype: found, language })
            }
        }
        other => Err(other),
    }
}

/// The IRI of a named node: what a `sh:class` reference newtype stands for.
fn into_iri(value: Value) -> Result<String, Value> {
    match value {
        Value::NamedNode(iri) => Ok(iri),
        other => Err(other),
    }
}

/// The identifier a nested (`sh:node`) shape is loaded from — an IRI, or a
/// blank node's label. A literal is not a node and cannot carry properties.
fn into_subject(value: Value) -> Result<String, Value> {
    match value {
        Value::NamedNode(iri) => Ok(iri),
        Value::BlankNode(label) => Ok(label),
        other => Err(other),
    }
}

fn digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit())
}

/// Splits the optional leading sign off a numeric lexical form.
fn split_sign(lexical: &str) -> (bool, &str) {
    match lexical.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, lexical.strip_prefix('+').unwrap_or(lexical)),
    }
}

/// `value` without leading zeros, keeping one digit.
fn trim_leading_zeros(value: &str) -> &str {
    let trimmed = value.trim_start_matches('0');
    if trimmed.is_empty() {
        "0"
    } else {
        trimmed
    }
}

/// Re-attaches a `-` unless `magnitude` is zero: XSD canonical form has exactly
/// one spelling of zero, so `-0` and `0` must not compare unequal.
fn with_sign(negative: bool, magnitude: String) -> String {
    if negative && magnitude.bytes().any(|b| b.is_ascii_digit() && b != b'0') {
        format!("-{}", magnitude)
    } else {
        magnitude
    }
}

/// An `xsd:integer`: an ARBITRARY-PRECISION integer, held in the canonical
/// lexical form of its value.
///
/// Deliberately not `i64`. The XSD integer value space is unbounded, so a
/// machine integer would reject values the shape admits — a loader answering
/// "not a valid xsd:integer" for a 30-digit literal would be wrong about the
/// datatype, not strict about it. Narrowing is available, but only as an
/// explicit, fallible step ([`Integer::to_i64`]).
///
/// Canonicalising on construction (no `+`, no leading zeros, one spelling of
/// zero) is what makes the derived `PartialEq` VALUE equality: `007`, `+7` and
/// `7` are one integer and compare equal.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Integer(String);

impl Integer {
    /// The value of an `xsd:integer` lexical form, or `None` when `lexical` is
    /// not one.
    pub fn parse(lexical: &str) -> Option<Self> {
        let (negative, magnitude) = split_sign(lexical);
        if !digits(magnitude) {
            return None;
        }
        Some(Self(with_sign(
            negative,
            trim_leading_zeros(magnitude).to_string(),
        )))
    }

    /// The canonical lexical form — the exact value, at any size.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The value as an `i64`, or `None` when it does not fit. Narrowing is the
    /// caller's decision to make, and to handle.
    pub fn to_i64(&self) -> Option<i64> {
        self.0.parse().ok()
    }
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An `xsd:decimal`: an ARBITRARY-PRECISION exact decimal, held in the
/// canonical lexical form of its value.
///
/// Deliberately not `f64`. The XSD decimal value space is exact and unbounded
/// in both precision and magnitude, so binary floating point would round
/// distinct values together and turn a large — but perfectly valid — decimal
/// into `INF`, which is not an `xsd:decimal` value at all. Conversion is
/// available, but only as an explicit, documented-lossy step
/// ([`Decimal::to_f64`]).
///
/// Canonicalising on construction (required decimal point, one digit either
/// side, no leading or trailing zeros beyond that, one spelling of zero) is
/// what makes the derived `PartialEq` VALUE equality: `1.50` and `1.5` are one
/// decimal and compare equal.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Decimal(String);

impl Decimal {
    /// The value of an `xsd:decimal` lexical form, or `None` when `lexical` is
    /// not one — no exponent, and none of the `INF`/`NaN` spellings, which
    /// belong to `xsd:double`.
    pub fn parse(lexical: &str) -> Option<Self> {
        let (negative, body) = split_sign(lexical);
        let (integral, fraction) = match body.split_once('.') {
            Some((integral, fraction)) => (integral, fraction),
            None => (body, ""),
        };
        if integral.is_empty() && fraction.is_empty() {
            return None;
        }
        if !integral.is_empty() && !digits(integral) {
            return None;
        }
        if !fraction.is_empty() && !digits(fraction) {
            return None;
        }
        let fraction = fraction.trim_end_matches('0');
        Some(Self(with_sign(
            negative,
            format!(
                "{}.{}",
                trim_leading_zeros(integral),
                if fraction.is_empty() { "0" } else { fraction },
            ),
        )))
    }

    /// The canonical lexical form — the exact value, at any precision.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The value as an `f64`. LOSSY: binary floating point cannot represent
    /// every decimal exactly, and a magnitude beyond `f64`'s range becomes
    /// infinite. [`Decimal::as_str`] is the exact value.
    pub fn to_f64(&self) -> f64 {
        self.0.parse().unwrap_or(f64::NAN)
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn two_digits(value: &str, low: u32, high: u32) -> bool {
    value.len() == 2 && digits(value) && matches!(value.parse::<u32>(), Ok(n) if n >= low && n <= high)
}

fn check_string(raw: &str) -> Option<String> {
    Some(raw.to_string())
}

/// `xsd:anyURI`: a string that can be escaped into an RFC 3986 URI reference.
///
/// The EMPTY string is such a reference (the same-document relative reference),
/// so it is accepted. What is rejected is the set RFC 3986 excludes outright and
/// escaping cannot rescue in place: the space, ASCII control characters, and the
/// delimiters `<`, `>`, `"`, `{`, `}`, `|`, `\`, `^` and `` ` ``.
fn check_any_uri(raw: &str) -> Option<String> {
    let excluded = |c: char| {
        c.is_ascii_control()
            || matches!(c, ' ' | '<' | '>' | '"' | '{' | '}' | '|' | '\\' | '^' | '`')
    };
    if raw.chars().any(excluded) {
        return None;
    }
    Some(raw.to_string())
}

fn check_boolean(raw: &str) -> Option<bool> {
    match raw {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

fn check_integer(raw: &str) -> Option<Integer> {
    Integer::parse(raw)
}

/// `xsd:decimal`: an optional sign then digits with an optional fraction — no
/// exponent, and none of the `INF`/`NaN` spellings `f64::from_str` accepts.
fn check_decimal(raw: &str) -> Option<Decimal> {
    Decimal::parse(raw)
}

/// `xsd:double`: `check_decimal`'s mantissa plus an optional exponent, and the
/// three special values.
fn check_double(raw: &str) -> Option<f64> {
    match raw {
        "INF" => return Some(f64::INFINITY),
        "-INF" => return Some(f64::NEG_INFINITY),
        "NaN" => return Some(f64::NAN),
        _ => {}
    }
    let (mantissa, exponent) = match raw.find(['e', 'E']) {
        Some(at) => (&raw[..at], Some(&raw[at + 1..])),
        None => (raw, None),
    };
    check_decimal(mantissa)?;
    if let Some(exponent) = exponent {
        if !digits(exponent.strip_prefix(['+', '-']).unwrap_or(exponent)) {
            return None;
        }
    }
    raw.parse::<f64>().ok()
}

/// The length of `month` (1-12) in a year that is, or is not, a `leap` year.
fn days_in_month(month: u32, leap: bool) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if leap => 29,
        _ => 28,
    }
}

/// `xsd:dateTime`: `-?YYYY(Y*)-MM-DDThh:mm:ss(.s+)?(Z|(+|-)hh:mm)?`, checked
/// against the value space and not just the field widths — the day resolved
/// against its month under the proleptic Gregorian leap rule, `24:00:00`
/// admitted only with zero minutes/seconds/fraction, seconds bounded at 59
/// (XSD has no leap second), and the timezone bounded at `±14:00` rather than
/// `±14:59`.
fn check_date_time(raw: &str) -> Option<String> {
    if !raw.is_ascii() {
        return None;
    }
    let (date, time) = raw.strip_prefix('-').unwrap_or(raw).split_once('T')?;
    let mut parts = date.split('-');
    let (year, month, day) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() || year.len() < 4 || !digits(year) {
        return None;
    }
    if year.len() > 4 && year.starts_with('0') {
        return None;
    }
    if !two_digits(month, 1, 12) || !two_digits(day, 1, 31) {
        return None;
    }
    // 10000 is a multiple of 400, so the last four digits fix the year's
    // residues mod 4, 100 and 400 — and therefore whether it is a leap year —
    // however many digits the year has. Negation preserves divisibility, so a
    // negative (BCE) year needs no separate rule.
    let tail = year[year.len() - 4..].parse::<u32>().ok()?;
    let leap = tail % 4 == 0 && (tail % 100 != 0 || tail % 400 == 0);
    if day.parse::<u32>().ok()? > days_in_month(month.parse::<u32>().ok()?, leap) {
        return None;
    }
    let clock = match time.strip_suffix('Z') {
        Some(clock) => clock,
        None if time.len() >= 6 => {
            let (clock, zone) = time.split_at(time.len() - 6);
            let bytes = zone.as_bytes();
            if (bytes[0] == b'+' || bytes[0] == b'-') && bytes[3] == b':' {
                if !two_digits(&zone[1..3], 0, 14) || !two_digits(&zone[4..6], 0, 59) {
                    return None;
                }
                // The offset is bounded at ±14:00, so 14 admits no minutes.
                if &zone[1..3] == "14" && &zone[4..6] != "00" {
                    return None;
                }
                clock
            } else {
                time
            }
        }
        None => time,
    };
    let mut fields = clock.split(':');
    let (hour, minute, second) = (fields.next()?, fields.next()?, fields.next()?);
    if fields.next().is_some() || !two_digits(hour, 0, 24) || !two_digits(minute, 0, 59) {
        return None;
    }
    let (second, fraction) = match second.split_once('.') {
        Some((second, fraction)) => (second, Some(fraction)),
        None => (second, None),
    };
    if !two_digits(second, 0, 59) {
        return None;
    }
    if let Some(fraction) = fraction {
        if !digits(fraction) {
            return None;
        }
    }
    // `24:00:00` is the ONLY lexical form hour 24 has — it names the midnight
    // starting the next day, so a nonzero minute, second or fraction alongside
    // it denotes no point in time at all.
    if hour == "24" {
        let zero_fraction = match fraction {
            Some(fraction) => fraction.bytes().all(|b| b == b'0'),
            None => true,
        };
        if minute != "00" || second != "00" || !zero_fraction {
            return None;
        }
    }
    Some(raw.to_string())
}
"##;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Cardinality, RustField, ValueType};

    fn field(value: ValueType, cardinality: Cardinality) -> RustField {
        RustField {
            name: "f".to_string(),
            predicate: "http://e.org/f".to_string(),
            value,
            cardinality,
        }
    }

    #[test]
    fn cardinality_wraps_the_value_type() {
        let scalar = ValueType::Scalar(ScalarType::Integer);
        assert_eq!(
            field_type(&field(scalar.clone(), Cardinality::Required)),
            "Integer"
        );
        assert_eq!(
            field_type(&field(scalar.clone(), Cardinality::Optional)),
            "Option<Integer>"
        );
        assert_eq!(
            field_type(&field(scalar, Cardinality::Many { min: 0, max: None })),
            "Vec<Integer>"
        );
    }

    /// The unbounded XSD value spaces do not lower to machine numbers — the
    /// mapping this crate calls faithful would not be, if they did.
    #[test]
    fn unbounded_xsd_numbers_lower_to_exact_types_not_i64_or_f64() {
        assert_eq!(ScalarType::Integer.rust_type(), "Integer");
        assert_eq!(ScalarType::Decimal.rust_type(), "Decimal");
        // xsd:double IS binary64, so f64 stays right for it.
        assert_eq!(ScalarType::Double.rust_type(), "f64");
        for exact in ["pub struct Integer(String)", "pub struct Decimal(String)"] {
            assert!(PRELUDE.contains(exact), "prelude is missing {}", exact);
        }
    }

    /// `sh:class` is a membership constraint, so the emitted loader must consult
    /// the source — a newtype wrapping any IRI would enforce nothing.
    #[test]
    fn a_reference_field_checks_class_membership() {
        let reference = ValueType::Reference {
            class: "http://e.org/Org".to_string(),
            name: "OrgRef".to_string(),
        };
        let source = convert("T", &field(reference, Cardinality::Required));
        assert!(
            source.contains("source.is_instance_of(&iri, OrgRef::CLASS)"),
            "the reference arm does not check membership:\n{}",
            source
        );
        assert!(
            source.contains("LoadError::ClassMembership"),
            "the reference arm has no membership failure:\n{}",
            source
        );
    }

    #[test]
    fn nested_values_are_boxed_so_recursive_shapes_stay_finitely_sized() {
        let nested = ValueType::Nested("Address".to_string());
        assert_eq!(
            field_type(&field(nested.clone(), Cardinality::Required)),
            "Box<Address>"
        );
        assert_eq!(
            field_type(&field(nested.clone(), Cardinality::Optional)),
            "Option<Box<Address>>"
        );
        // A `Vec` is already indirect, so it needs no extra box.
        assert_eq!(
            field_type(&field(nested, Cardinality::Many { min: 0, max: None })),
            "Vec<Address>"
        );
    }

    #[test]
    fn references_keep_their_newtype_name() {
        let reference = ValueType::Reference {
            class: "http://e.org/Org".to_string(),
            name: "OrgRef".to_string(),
        };
        assert_eq!(field_type(&field(reference, Cardinality::Required)), "OrgRef");
    }

    #[test]
    fn every_scalar_has_a_checker_in_the_prelude() {
        for scalar in [
            ScalarType::String,
            ScalarType::Boolean,
            ScalarType::Integer,
            ScalarType::Decimal,
            ScalarType::Double,
            ScalarType::DateTime,
            ScalarType::AnyUri,
        ] {
            let name = checker(scalar);
            assert!(
                PRELUDE.contains(&format!("fn {}(raw: &str)", name)),
                "prelude is missing {}",
                name
            );
        }
    }

    #[test]
    fn string_literals_are_escaped() {
        assert_eq!(quote("a\"b\\c"), "\"a\\\"b\\\\c\"");
        assert_eq!(join_quoted(&["a".to_string(), "b".to_string()]), "\"a\", \"b\"");
        assert_eq!(join_quoted(&[]), "");
    }

    #[test]
    fn an_empty_model_emits_just_the_prelude() {
        assert_eq!(emit(&RustModel::default()), PRELUDE);
    }

    #[test]
    fn a_vacuous_minimum_emits_no_comparison() {
        let mut out = String::new();
        emit_field_load(
            &mut out,
            "T",
            &field(
                ValueType::Scalar(ScalarType::String),
                Cardinality::Many { min: 0, max: None },
            ),
        );
        assert!(!out.contains("values.len() < 0"), "emitted a vacuous check:\n{}", out);
        assert!(!out.contains("values.len() >"), "emitted an unbounded maximum:\n{}", out);
    }
}
