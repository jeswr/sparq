#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]
//!
//! ---
//!
//! # Scaffold status (bead `sq-oy1f.23`)
//!
//! This is **Phase A** of the document-level W3C JSON-LD 1.1 program
//! (design record `research/jsonld-1.1-design.md`). The concrete surface that ships
//! today is:
//!
//! - [`json`] — the minimal, dependency-free `Json` AST (moved verbatim out of
//!   `sparq-engine`'s JSON-LD writer; public API preserved via a re-export shim there).
//! - [`error`] — the full JSON-LD 1.1 error-code registry ([`JsonLdErrorCode`]) plus the
//!   fallible [`JsonLdError`] type every algorithm will return.
//! - [`options`] — [`JsonLdOptions`], the processing options honoured across the pipeline.
//! - [`loader`] — the [`DocumentLoader`] trait, **deny-by-default** via [`NoopLoader`]
//!   (no ambient network — a remote fetch raises `loading document failed`), plus the
//!   local-fixture [`FsLoader`].
//!
//! The algorithm modules ([`context`], [`expand`], [`node_map`], [`flatten`],
//! [`compact`], [`frame`], [`from_rdf`], [`to_rdf`], [`api`]) are **stubs**: they carry
//! the spec references and the public shape only. Their implementations land in the
//! dependency-ordered follow-on beads (`sq-oy1f.24`+). No algorithm here yet, and nothing
//! panics: the crate is `todo!()`-free.

// Real, shipped surfaces (Phase A).
pub mod error;
pub mod json;
pub mod loader;
pub mod options;

// Algorithm scaffolds — spec references + public shape only (filled by sq-oy1f.24+).
pub mod api;
pub mod compact;
pub mod context;
pub mod expand;
pub mod flatten;
pub mod frame;
pub mod from_rdf;
pub mod node_map;
pub mod to_rdf;

pub use error::{JsonLdError, JsonLdErrorCode};
pub use json::Json;
pub use loader::{DocumentLoader, FsLoader, NoopLoader, RemoteDocument};
pub use options::{EmbedFlag, JsonLdOptions, ProcessingMode, RdfDirection};
