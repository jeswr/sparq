#![doc = include_str!("../README.md")]

// [FABLE-5] sq-1rg2q.12: SHACL shapes -> Rust object models, in two explicit
// steps (lower, then emit) so "what does this SHACL mean" is testable without
// reading generated source, and "how does that read as Rust" is testable without
// re-deriving the meaning.

pub mod emit;
pub mod error;
pub mod lower;
pub mod schema;

pub use emit::emit;
pub use error::LowerError;
pub use lower::lower;
pub use schema::{
    Cardinality, ClassMarker, Field, ModelType, ObjectModel, RustScalar, ScalarType, ValueType,
};

use sparq_shacl::ShapesModel;

/// Lowers a parsed shapes graph and renders it as a standalone Rust module.
///
/// The convenience composition of [`lower()`] and [`emit()`]. Both steps are
/// deterministic, so the same shapes graph always yields the same bytes.
///
/// # Errors
///
/// Propagates the [`LowerError`] from [`lower()`]; emission itself cannot fail.
///
/// ```
/// let shapes = sparq_shacl::load_turtle_with_base(
///     r#"
///         @prefix sh: <http://www.w3.org/ns/shacl#> .
///         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
///         @prefix ex: <http://example.org/> .
///         ex:PersonShape a sh:NodeShape ;
///             sh:targetClass ex:Person ;
///             sh:property [ sh:path ex:name ; sh:datatype xsd:string ;
///                           sh:minCount 1 ; sh:maxCount 1 ] .
///     "#,
///     "http://example.org/",
/// )?;
/// let model = sparq_shacl::ShapesModel::parse(&shapes);
/// let source = sparq_wrapper_shacl::generate(&model)?;
/// assert!(source.contains("pub struct PersonShape"));
/// assert!(source.contains("pub name: String"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn generate(shapes: &ShapesModel) -> Result<String, LowerError> {
    Ok(emit(&lower(shapes)?))
}
