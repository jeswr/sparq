//! [`RustModel`] → Rust source. [FABLE-5] (sq-1rg2q.12)
//!
//! [`emit`] is a pure function of the IR, so the same shapes graph always
//! produces byte-identical source — the property a checked-in generated module
//! needs if `git diff` is to stay empty across regenerations.
//!
//! ## What the emitted module depends on
//!
//! Nothing. The generated source is `std`-only: it declares its own
//! `ValueSource` trait, its own `LoadError`, and its own lexical checkers, so it
//! compiles standalone with `rustc` and a consumer can adapt any RDF source to
//! it (a `sparq-wrapper` focus, a `sparq-core` graph, a fixture) by
//! implementing two methods.
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
        "\n    /// Loads `subject` from `source`, checking every cardinality and\n    \
         /// datatype the shape declares.\n    \
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

/// The expression converting the raw lexical value bound to `raw` into one
/// value of `field`'s type, short-circuiting with `?`/`return` on failure.
fn convert(type_name: &str, field: &RustField) -> String {
    match &field.value {
        ValueType::Scalar(scalar) => format!(
            "match {}(&raw) {{ Some(value) => value, None => return Err(LoadError::Datatype {{ \
             type_name: {}, field: {}, datatype: {}, value: raw }}) }}",
            checker(*scalar),
            quote(type_name),
            quote(&field.name),
            quote(&scalar.datatype()),
        ),
        ValueType::Reference { name, .. } => format!("{}(raw)", name),
        ValueType::Nested(name) => format!("{}::load(source, &raw)?", name),
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

/// The subject-oriented view of an RDF source the generated loaders read.
pub trait ValueSource {
    /// Every predicate asserted on `subject`. Used by `sh:closed` shapes.
    fn predicates(&self, subject: &str) -> Vec<String>;
    /// The lexical form of every object of `(subject, predicate)` — the literal's
    /// lexical value, or the IRI of a named node.
    fn values(&self, subject: &str, predicate: &str) -> Vec<String>;
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
    /// A value's lexical form is not valid for the field's `sh:datatype`.
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
            Self::Datatype { type_name, field, datatype, value } => write!(
                f,
                "{}.{}: {:?} is not a valid <{}>",
                type_name, field, value, datatype,
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

fn digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit())
}

fn two_digits(value: &str, low: u32, high: u32) -> bool {
    value.len() == 2 && digits(value) && matches!(value.parse::<u32>(), Ok(n) if n >= low && n <= high)
}

fn check_string(raw: &str) -> Option<String> {
    Some(raw.to_string())
}

fn check_any_uri(raw: &str) -> Option<String> {
    if raw.is_empty() {
        None
    } else {
        Some(raw.to_string())
    }
}

fn check_boolean(raw: &str) -> Option<bool> {
    match raw {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

fn check_integer(raw: &str) -> Option<i64> {
    if !digits(raw.strip_prefix(['+', '-']).unwrap_or(raw)) {
        return None;
    }
    raw.parse::<i64>().ok()
}

/// `xsd:decimal`: an optional sign then digits with an optional fraction — no
/// exponent, and none of the `INF`/`NaN` spellings `f64::from_str` accepts.
fn check_decimal(raw: &str) -> Option<f64> {
    let body = raw.strip_prefix(['+', '-']).unwrap_or(raw);
    let (integral, fraction) = match body.split_once('.') {
        Some((integral, fraction)) => (integral, Some(fraction)),
        None => (body, None),
    };
    if integral.is_empty() && fraction.map_or(true, str::is_empty) {
        return None;
    }
    if !integral.is_empty() && !digits(integral) {
        return None;
    }
    if let Some(fraction) = fraction {
        if !fraction.is_empty() && !digits(fraction) {
            return None;
        }
    }
    raw.parse::<f64>().ok()
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

/// `xsd:dateTime`: `-?YYYY-MM-DDThh:mm:ss(.s+)?(Z|(+|-)hh:mm)?`, with the field
/// ranges checked. Leap-second `60` is accepted; day-of-month is bounded at 31
/// rather than resolved per month.
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
    let clock = match time.strip_suffix('Z') {
        Some(clock) => clock,
        None if time.len() >= 6 => {
            let (clock, zone) = time.split_at(time.len() - 6);
            let bytes = zone.as_bytes();
            if (bytes[0] == b'+' || bytes[0] == b'-') && bytes[3] == b':' {
                if !two_digits(&zone[1..3], 0, 14) || !two_digits(&zone[4..6], 0, 59) {
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
    if !two_digits(second, 0, 60) {
        return None;
    }
    if let Some(fraction) = fraction {
        if !digits(fraction) {
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
        assert_eq!(field_type(&field(scalar.clone(), Cardinality::Required)), "i64");
        assert_eq!(
            field_type(&field(scalar.clone(), Cardinality::Optional)),
            "Option<i64>"
        );
        assert_eq!(
            field_type(&field(scalar, Cardinality::Many { min: 0, max: None })),
            "Vec<i64>"
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
