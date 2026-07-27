//! Faithful Rust object models generated from SHACL shapes. [FABLE-5] (sq-1rg2q.12)
//!
//! This is the SHACL integration seam for `sparq-wrapper`: it turns a shapes
//! graph into `struct`s a Rust program can hold, instead of leaving callers to
//! traverse predicates by hand.
//!
//! It parses no Turtle and re-implements no SHACL. Shapes come in as a
//! [`sparq_shacl::ShapesModel`] — the same parser the validator uses, so a shape
//! cannot mean one thing to validation and another to code generation.
//!
//! # Pipeline
//!
//! 1. [`lower`] maps a [`ShapesModel`] to the [`RustModel`] IR.
//! 2. [`emit`] renders that IR as a self-contained, `std`-only Rust module.
//!
//! [`generate`] runs both. The IR is a separate, comparable value so the mapping
//! can be asserted exactly without string-matching generated source.
//!
//! # The mapping
//!
//! | SHACL | Rust |
//! |-------|------|
//! | `sh:maxCount 1` with `sh:minCount 0` or absent | `Option<T>` |
//! | `sh:maxCount 1` with `sh:minCount >= 1` | `T` |
//! | any other cardinality | `Vec<T>`, bounds checked on load |
//! | `sh:datatype` | a checked scalar ([`ScalarType`]) |
//! | `sh:class` | a typed reference newtype ([`RustReference`]) |
//! | `sh:node` | a nested generated struct (boxed) |
//! | `sh:closed true` | a static predicate whitelist the loader enforces |
//!
//! Anything ill-formed or contradictory is a typed [`LoweringError`], never a
//! guess. Constraints that restrict *which* graphs load rather than *what* the
//! Rust type is — `sh:pattern`, `sh:minInclusive`, `sh:or`, … — are left to the
//! validator and do not appear in the generated model.
//!
//! A consumer adapts an RDF source to the generated module by implementing its
//! `ValueSource` trait over RDF *terms*, not lexical strings. That is what lets
//! the loader enforce `sh:datatype` as SHACL defines it — a value must be a
//! literal of exactly the declared datatype *before* its lexical form is
//! checked, so neither an IRI nor an `xsd:string` literal spelling `42`
//! satisfies `sh:datatype xsd:integer`.
//!
//! # Example
//!
//! ```
//! use sparq_core::Graph;
//! use sparq_shacl::ShapesModel;
//! use sparq_wrapper_shacl::{lower, Cardinality};
//!
//! let shapes = Graph::load_str(
//!     r#"
//!     @prefix sh:  <http://www.w3.org/ns/shacl#> .
//!     @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
//!     @prefix ex:  <http://example.org/> .
//!     ex:PersonShape a sh:NodeShape ;
//!         sh:targetClass ex:Person ;
//!         sh:property [ sh:path ex:name ; sh:datatype xsd:string ;
//!                       sh:minCount 1 ; sh:maxCount 1 ] .
//!     "#,
//!     "turtle",
//! )?;
//! let model = lower(&ShapesModel::parse(&shapes))?;
//!
//! let person = &model.types[0];
//! assert_eq!(person.name, "PersonShape");
//! assert_eq!(person.fields[0].name, "name");
//! assert_eq!(person.fields[0].cardinality, Cardinality::Required);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

mod emit;
mod error;
mod lower;
mod schema;

pub use emit::emit;
pub use error::LoweringError;
pub use lower::lower;
pub use schema::{
    Cardinality, ClosedShape, RustField, RustModel, RustReference, RustType, ScalarType, ValueType,
};

use sparq_shacl::ShapesModel;

/// Lowers `shapes` and renders the result as a self-contained Rust module.
///
/// The output is a pure function of `shapes`: generating twice from the same
/// shapes graph produces byte-identical source, so a checked-in generated
/// module only changes when the shapes do.
///
/// Write the output to its own file and declare it with `mod` — it opens with a
/// module-level `#![allow(dead_code)]`, so `include!`ing it mid-file will not
/// compile.
///
/// # Errors
///
/// Returns a [`LoweringError`] when the shapes graph is ill-formed, describes a
/// contradiction, or uses SHACL this generator declines to approximate.
pub fn generate(shapes: &ShapesModel) -> Result<String, LoweringError> {
    Ok(emit(&lower(shapes)?))
}
