//! Emission of the object-model IR (the `schema` module) as Rust source.
//!
//! The emitted file is **std-only**: it declares its own `Value`/`Source`/
//! `LoadError` vocabulary rather than naming a graph type, so it compiles with no
//! dependencies and any store (including `sparq_wrapper::Store`) can be adapted to
//! it by implementing one two-method trait. Load it with `mod`/`#[path] mod` or
//! `include!`; it carries no inner attributes, so both work.
//!
//! Emission is a pure function of the IR — the IR is fully sorted, and nothing
//! here consults a hash map — so `emit(&schema)` is byte-identical across runs.
//!
//! [FABLE-5] (sq-1rg2q.12)

use std::fmt::Write as _;

use crate::schema::{Cardinality, ModelSchema, ScalarKind, StructSchema, ValueSchema};

/// The fixed vocabulary every generated file opens with: the RDF term type, the
/// two-method `Source` the loaders read through, the typed load error, and the
/// scalar/reference helpers the per-shape loaders call.
const PRELUDE: &str = r#"use std::fmt;

/// An RDF term, as the generated loaders see it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Value {
    Iri(String),
    Blank(String),
    Literal {
        lexical: String,
        datatype: String,
        language: Option<String>,
    },
}

#[allow(dead_code)]
impl Value {
    /// The IRI or blank-node label, or `None` for a literal.
    pub fn node_id(&self) -> Option<&str> {
        match self {
            Value::Iri(iri) => Some(iri),
            Value::Blank(label) => Some(label),
            Value::Literal { .. } => None,
        }
    }

    fn describe(&self) -> String {
        match self {
            Value::Iri(iri) => format!("<{}>", iri),
            Value::Blank(label) => format!("_:{}", label),
            Value::Literal {
                lexical, datatype, ..
            } => format!("{:?}^^<{}>", lexical, datatype),
        }
    }
}

/// The graph a generated loader reads.
#[allow(dead_code)]
pub trait Source {
    /// The objects of `(subject, predicate)`, in any order.
    fn values(&self, subject: &Value, predicate: &str) -> Vec<Value>;
    /// Every predicate IRI asserted on `subject`. Read only by closed shapes.
    fn predicates(&self, subject: &Value) -> Vec<String>;
}

/// `rdf:type`.
pub const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
/// `rdfs:subClassOf`.
pub const RDFS_SUB_CLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";

/// Why a focus node could not be loaded into a generated struct.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LoadError {
    /// The value count is outside the shape's `sh:minCount`/`sh:maxCount`.
    Cardinality {
        shape: &'static str,
        predicate: &'static str,
        min: Option<u64>,
        max: Option<u64>,
        got: usize,
    },
    /// A literal's datatype is outside the shape's `sh:datatype` set.
    Datatype {
        shape: &'static str,
        predicate: &'static str,
        allowed: &'static [&'static str],
        got: String,
    },
    /// The datatype matched but the lexical form does not parse as the Rust scalar.
    Lexical {
        shape: &'static str,
        predicate: &'static str,
        rust: &'static str,
        got: String,
    },
    /// A literal was required (`sh:datatype`) and something else was found.
    NotLiteral {
        shape: &'static str,
        predicate: &'static str,
        got: String,
    },
    /// An IRI or blank node was required and a literal was found.
    NotNode {
        shape: &'static str,
        predicate: &'static str,
        got: String,
    },
    /// The referent is not a SHACL instance of any class in the `sh:class` set.
    Class {
        shape: &'static str,
        predicate: &'static str,
        allowed: &'static [&'static str],
        got: String,
    },
    /// The focus node carries a predicate the `sh:closed` shape does not allow.
    ClosedPredicate {
        shape: &'static str,
        predicate: String,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Cardinality {
                shape,
                predicate,
                min,
                max,
                got,
            } => write!(
                f,
                "<{}> <{}>: {} value(s) outside sh:minCount {:?} / sh:maxCount {:?}",
                shape, predicate, got, min, max
            ),
            LoadError::Datatype {
                shape,
                predicate,
                allowed,
                got,
            } => write!(
                f,
                "<{}> <{}>: datatype <{}> is not one of {:?}",
                shape, predicate, got, allowed
            ),
            LoadError::Lexical {
                shape,
                predicate,
                rust,
                got,
            } => write!(
                f,
                "<{}> <{}>: {:?} does not parse as {}",
                shape, predicate, got, rust
            ),
            LoadError::NotLiteral {
                shape,
                predicate,
                got,
            } => write!(f, "<{}> <{}>: expected a literal, found {}", shape, predicate, got),
            LoadError::NotNode {
                shape,
                predicate,
                got,
            } => write!(
                f,
                "<{}> <{}>: expected an IRI or blank node, found {}",
                shape, predicate, got
            ),
            LoadError::Class {
                shape,
                predicate,
                allowed,
                got,
            } => write!(
                f,
                "<{}> <{}>: {} is not a SHACL instance of any of {:?}",
                shape, predicate, got, allowed
            ),
            LoadError::ClosedPredicate { shape, predicate } => write!(
                f,
                "<{}>: sh:closed shape does not allow predicate <{}>",
                shape, predicate
            ),
        }
    }
}

impl std::error::Error for LoadError {}

#[allow(dead_code)]
fn checked_literal<'a>(
    shape: &'static str,
    predicate: &'static str,
    allowed: &'static [&'static str],
    value: &'a Value,
) -> Result<&'a str, LoadError> {
    match value {
        Value::Literal {
            lexical, datatype, ..
        } => {
            if allowed.contains(&datatype.as_str()) {
                Ok(lexical)
            } else {
                Err(LoadError::Datatype {
                    shape,
                    predicate,
                    allowed,
                    got: datatype.clone(),
                })
            }
        }
        other => Err(LoadError::NotLiteral {
            shape,
            predicate,
            got: other.describe(),
        }),
    }
}

#[allow(dead_code)]
fn load_string(
    shape: &'static str,
    predicate: &'static str,
    allowed: &'static [&'static str],
    value: &Value,
) -> Result<String, LoadError> {
    Ok(checked_literal(shape, predicate, allowed, value)?.to_string())
}

#[allow(dead_code)]
fn load_i64(
    shape: &'static str,
    predicate: &'static str,
    allowed: &'static [&'static str],
    value: &Value,
) -> Result<i64, LoadError> {
    let lexical = checked_literal(shape, predicate, allowed, value)?;
    lexical.parse::<i64>().map_err(|_| LoadError::Lexical {
        shape,
        predicate,
        rust: "i64",
        got: lexical.to_string(),
    })
}

#[allow(dead_code)]
fn load_f64(
    shape: &'static str,
    predicate: &'static str,
    allowed: &'static [&'static str],
    value: &Value,
) -> Result<f64, LoadError> {
    let lexical = checked_literal(shape, predicate, allowed, value)?;
    lexical.parse::<f64>().map_err(|_| LoadError::Lexical {
        shape,
        predicate,
        rust: "f64",
        got: lexical.to_string(),
    })
}

#[allow(dead_code)]
fn load_bool(
    shape: &'static str,
    predicate: &'static str,
    allowed: &'static [&'static str],
    value: &Value,
) -> Result<bool, LoadError> {
    let lexical = checked_literal(shape, predicate, allowed, value)?;
    match lexical {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        other => Err(LoadError::Lexical {
            shape,
            predicate,
            rust: "bool",
            got: other.to_string(),
        }),
    }
}

#[allow(dead_code)]
fn load_iri(
    shape: &'static str,
    predicate: &'static str,
    value: &Value,
) -> Result<String, LoadError> {
    match value {
        Value::Iri(iri) => Ok(iri.clone()),
        other => Err(LoadError::NotNode {
            shape,
            predicate,
            got: other.describe(),
        }),
    }
}

#[allow(dead_code)]
fn load_node(
    shape: &'static str,
    predicate: &'static str,
    value: &Value,
) -> Result<Value, LoadError> {
    match value {
        Value::Iri(_) | Value::Blank(_) => Ok(value.clone()),
        other => Err(LoadError::NotNode {
            shape,
            predicate,
            got: other.describe(),
        }),
    }
}

/// SHACL "SHACL instance of": `rdf:type` followed transitively by
/// `rdfs:subClassOf`, cycle-guarded.
#[allow(dead_code)]
fn is_instance_of<S: Source + ?Sized>(
    source: &S,
    node: &Value,
    classes: &[&'static str],
) -> bool {
    let mut frontier = source.values(node, RDF_TYPE);
    let mut seen: Vec<String> = Vec::new();
    while let Some(current) = frontier.pop() {
        let id = match current.node_id() {
            Some(id) => id.to_string(),
            None => continue,
        };
        if classes.contains(&id.as_str()) {
            return true;
        }
        if seen.iter().any(|s| *s == id) {
            continue;
        }
        seen.push(id);
        frontier.extend(source.values(&current, RDFS_SUB_CLASS_OF));
    }
    false
}
"#;

/// Emits the whole object model as one Rust source file.
///
/// The output is deterministic: emitting the same [`ModelSchema`] twice produces
/// byte-identical source.
pub fn emit(schema: &ModelSchema) -> String {
    let mut out = String::new();
    out.push_str(
        "// Generated by sparq-wrapper-shacl from SHACL shapes. DO NOT EDIT.\n\
         //\n\
         // Load with `mod`/`#[path = \"…\"] mod` or `include!`. The file is std-only:\n\
         // implement `Source` over your store to drive the generated loaders.\n\n",
    );
    out.push_str(PRELUDE);

    for reference in &schema.references {
        out.push('\n');
        emit_reference(&mut out, &reference.name, &reference.classes);
    }
    for def in &schema.structs {
        out.push('\n');
        emit_struct(&mut out, def);
    }
    out
}

fn emit_reference(out: &mut String, name: &str, classes: &[String]) {
    let _ = writeln!(
        out,
        "/// Typed reference: the referent must be a SHACL instance of one of {}.",
        classes
            .iter()
            .map(|c| format!("`{c}`"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    out.push_str("#[derive(Debug, Clone, PartialEq, Eq)]\n");
    let _ = writeln!(out, "pub struct {name} {{");
    out.push_str("    /// The referent node (an IRI or a blank node).\n");
    out.push_str("    pub node: Value,\n}\n\n");
    out.push_str("#[allow(dead_code)]\n");
    let _ = writeln!(out, "impl {name} {{");
    out.push_str("    /// The classes a referent must be a SHACL instance of.\n");
    let _ = writeln!(
        out,
        "    pub const CLASSES: &'static [&'static str] = &[{}];",
        classes
            .iter()
            .map(|c| rust_str(c))
            .collect::<Vec<_>>()
            .join(", ")
    );
    out.push_str(
        "\n    /// The referent's IRI, or `None` when it is a blank node.\n\
         \x20   pub fn iri(&self) -> Option<&str> {\n\
         \x20       match &self.node {\n\
         \x20           Value::Iri(iri) => Some(iri),\n\
         \x20           _ => None,\n\
         \x20       }\n\
         \x20   }\n\n\
         \x20   /// Checks and wraps one value of a `sh:class`-constrained property.\n\
         \x20   pub fn load<S: Source + ?Sized>(\n\
         \x20       source: &S,\n\
         \x20       shape: &'static str,\n\
         \x20       predicate: &'static str,\n\
         \x20       value: &Value,\n\
         \x20   ) -> Result<Self, LoadError> {\n\
         \x20       let node = load_node(shape, predicate, value)?;\n\
         \x20       if !is_instance_of(source, &node, Self::CLASSES) {\n\
         \x20           return Err(LoadError::Class {\n\
         \x20               shape,\n\
         \x20               predicate,\n\
         \x20               allowed: Self::CLASSES,\n\
         \x20               got: node.describe(),\n\
         \x20           });\n\
         \x20       }\n\
         \x20       Ok(Self { node })\n\
         \x20   }\n}\n",
    );
}

fn emit_struct(out: &mut String, def: &StructSchema) {
    let _ = writeln!(
        out,
        "/// Generated from the SHACL node shape `{}`.",
        def.shape
    );
    out.push_str("#[derive(Debug, Clone, PartialEq)]\n");
    let _ = writeln!(out, "pub struct {} {{", def.name);
    for field in &def.fields {
        let _ = writeln!(out, "    /// `{}`", field.predicate);
        let _ = writeln!(out, "    pub {}: {},", field.name, field.rust_type());
    }
    out.push_str("}\n\n#[allow(dead_code)]\n");
    let _ = writeln!(out, "impl {} {{", def.name);
    out.push_str("    /// The node shape this struct was generated from.\n");
    let _ = writeln!(
        out,
        "    pub const SHAPE: &'static str = {};",
        rust_str(&def.shape)
    );
    if let Some(closed) = &def.closed {
        out.push_str("    /// The predicates `sh:closed` permits on the focus node.\n");
        let _ = writeln!(
            out,
            "    pub const ALLOWED_PREDICATES: &'static [&'static str] = &[{}];",
            closed
                .allowed
                .iter()
                .map(|p| rust_str(p))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    for field in &def.fields {
        let _ = writeln!(
            out,
            "    const {}: &'static str = {};",
            predicate_const(&field.name),
            rust_str(&field.predicate)
        );
        if let ValueSchema::Scalar { datatypes, .. } = &field.value {
            let _ = writeln!(
                out,
                "    const {}: &'static [&'static str] = &[{}];",
                datatype_const(&field.name),
                datatypes
                    .iter()
                    .map(|d| rust_str(d))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    out.push_str("\n    /// Loads `focus` from `source`, enforcing the shape's constraints.\n");
    out.push_str(
        "    pub fn load<S: Source + ?Sized>(source: &S, focus: &Value) -> Result<Self, LoadError> {\n",
    );
    if def.fields.is_empty() && def.closed.is_none() {
        out.push_str("        let _ = (source, focus);\n");
    }
    if def.closed.is_some() {
        out.push_str(
            "        let mut present = source.predicates(focus);\n\
             \x20       present.sort();\n\
             \x20       present.dedup();\n\
             \x20       for predicate in present {\n\
             \x20           if !Self::ALLOWED_PREDICATES.contains(&predicate.as_str()) {\n\
             \x20               return Err(LoadError::ClosedPredicate {\n\
             \x20                   shape: Self::SHAPE,\n\
             \x20                   predicate,\n\
             \x20               });\n\
             \x20           }\n\
             \x20       }\n",
        );
    }
    for field in &def.fields {
        emit_field_load(out, field);
    }
    out.push_str("        Ok(Self {\n");
    for field in &def.fields {
        let _ = writeln!(out, "            {},", field.name);
    }
    out.push_str("        })\n    }\n}\n");
}

fn emit_field_load(out: &mut String, field: &crate::schema::FieldSchema) {
    let p = predicate_const(&field.name);
    let convert = converter(field);
    let _ = writeln!(out, "        let {} = {{", field.name);
    let _ = writeln!(
        out,
        "            let values = source.values(focus, Self::{p});"
    );
    match field.cardinality {
        Cardinality::Optional => {
            let _ = writeln!(
                out,
                "            if values.len() > 1 {{\n\
                 \x20               return Err(LoadError::Cardinality {{\n\
                 \x20                   shape: Self::SHAPE,\n\
                 \x20                   predicate: Self::{p},\n\
                 \x20                   min: None,\n\
                 \x20                   max: Some(1),\n\
                 \x20                   got: values.len(),\n\
                 \x20               }});\n\
                 \x20           }}\n\
                 \x20           match values.first() {{\n\
                 \x20               Some(value) => Some({convert}),\n\
                 \x20               None => None,\n\
                 \x20           }}"
            );
        }
        Cardinality::Required => {
            let _ = writeln!(
                out,
                "            if values.len() != 1 {{\n\
                 \x20               return Err(LoadError::Cardinality {{\n\
                 \x20                   shape: Self::SHAPE,\n\
                 \x20                   predicate: Self::{p},\n\
                 \x20                   min: Some(1),\n\
                 \x20                   max: Some(1),\n\
                 \x20                   got: values.len(),\n\
                 \x20               }});\n\
                 \x20           }}\n\
                 \x20           let value = &values[0];\n\
                 \x20           {convert}"
            );
        }
        Cardinality::Many { min, max } => {
            let min_lit = option_lit(min);
            let max_lit = option_lit(max);
            for (cmp, bound) in [("<", min), (">", max)] {
                let Some(bound) = bound else { continue };
                // `len() < 1` is `is_empty()`; emit the idiomatic spelling so the
                // generated file is clean under the consumer's own clippy too.
                let test = if cmp == "<" && bound == 1 {
                    "values.is_empty()".to_string()
                } else {
                    format!("values.len() {cmp} {bound}")
                };
                let _ = writeln!(
                    out,
                    "            if {test} {{\n\
                     \x20               return Err(LoadError::Cardinality {{\n\
                     \x20                   shape: Self::SHAPE,\n\
                     \x20                   predicate: Self::{p},\n\
                     \x20                   min: {min_lit},\n\
                     \x20                   max: {max_lit},\n\
                     \x20                   got: values.len(),\n\
                     \x20               }});\n\
                     \x20           }}"
                );
            }
            let _ = writeln!(
                out,
                "            let mut collected = Vec::with_capacity(values.len());\n\
                 \x20           for value in &values {{\n\
                 \x20               collected.push({convert});\n\
                 \x20           }}\n\
                 \x20           collected"
            );
        }
    }
    out.push_str("        };\n");
}

/// The expression that turns the in-scope `value: &Value` into one field value.
fn converter(field: &crate::schema::FieldSchema) -> String {
    let p = predicate_const(&field.name);
    let d = datatype_const(&field.name);
    match &field.value {
        ValueSchema::Scalar { kind, .. } => {
            let f = match kind {
                ScalarKind::Lexical => "load_string",
                ScalarKind::I64 => "load_i64",
                ScalarKind::F64 => "load_f64",
                ScalarKind::Bool => "load_bool",
            };
            format!("{f}(Self::SHAPE, Self::{p}, Self::{d}, value)?")
        }
        ValueSchema::Reference { rust, .. } => {
            format!("{rust}::load(source, Self::SHAPE, Self::{p}, value)?")
        }
        ValueSchema::Nested { rust } => match field.cardinality {
            Cardinality::Many { .. } => format!("{rust}::load(source, value)?"),
            _ => format!("Box::new({rust}::load(source, value)?)"),
        },
        ValueSchema::Iri => format!("load_iri(Self::SHAPE, Self::{p}, value)?"),
        ValueSchema::Term => "value.clone()".to_string(),
    }
}

fn option_lit(v: Option<u64>) -> String {
    match v {
        Some(n) => format!("Some({n})"),
        None => "None".to_string(),
    }
}

fn predicate_const(field: &str) -> String {
    format!("P_{}", field.to_uppercase())
}

fn datatype_const(field: &str) -> String {
    format!("D_{}", field.to_uppercase())
}

/// Renders `s` as a Rust string literal.
fn rust_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}
