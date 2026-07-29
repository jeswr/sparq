//! SHACL integration seam for `sparq-wrapper`: SHACL shapes to Rust object models.
//!
//! With the OPT-IN `oo-models` feature this crate turns a shapes graph into Rust
//! structs — the "generate typed objects from my SHACL" half of the wrapper
//! surface. It is a *two-stage* pipeline with an inspectable IR in the middle:
//!
//! ```text
//!   shapes graph --[sparq-shacl]--> ShapesModel --[lower]--> ModelSchema --[emit]--> Rust
//! ```
//!
//! The SHACL parser is **reused, never re-implemented**: shapes come in as
//! `sparq_shacl::{ShapesModel, Shape, Component, Path, Target}` (see
//! `skills/rdf-wrapper/SKILL.md`).
//!
//! Both stages are deterministic — `lower` sorts away `ShapesModel`'s
//! graph-traversal order, and `emit` is a pure function of the sorted IR — so
//! the same shapes graph always produces byte-identical source. Anything that
//! cannot be modelled faithfully (an ill-formed shapes graph, a contradictory
//! cardinality, a non-predicate `sh:path`) is a typed `SchemaError`, never a
//! silently dropped constraint.
//!
//! See the `lower` module for the full shapes-to-IR mapping table and the
//! `emit` module for the shape of the generated code.
//!
//! With the feature off the crate stays the dependency-free seam stub it was.
//!
//! [FABLE-5] (sq-1rg2q.12)
#![forbid(unsafe_code)]

#[cfg(feature = "oo-models")]
pub mod emit;
#[cfg(feature = "oo-models")]
pub mod error;
#[cfg(feature = "oo-models")]
pub mod lower;
#[cfg(feature = "oo-models")]
pub mod schema;

#[cfg(feature = "oo-models")]
pub use emit::emit;
#[cfg(feature = "oo-models")]
pub use error::SchemaError;
#[cfg(feature = "oo-models")]
pub use lower::lower;
#[cfg(feature = "oo-models")]
pub use schema::{
    Cardinality, ClosedSchema, FieldSchema, ModelSchema, ReferenceSchema, ScalarKind, StructSchema,
    ValueSchema,
};
