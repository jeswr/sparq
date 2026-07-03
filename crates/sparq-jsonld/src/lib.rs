#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]
//!
//! ---
//!
//! # Build-out status
//!
//! This is the document-level W3C JSON-LD 1.1 program (design record
//! `research/jsonld-1.1-design.md`). The surface that ships today is:
//!
//! - [`json`] — the minimal, dependency-free `Json` AST (moved verbatim out of
//!   `sparq-engine`'s JSON-LD writer; public API preserved via a re-export shim there),
//!   plus a small `Json::parse` used to read remote contexts.
//! - [`error`] — the full JSON-LD 1.1 error-code registry ([`JsonLdErrorCode`]) plus the
//!   fallible [`JsonLdError`] type every algorithm returns.
//! - [`options`] — [`JsonLdOptions`], the processing options honoured across the pipeline.
//! - [`loader`] — the [`DocumentLoader`] trait, **deny-by-default** via [`NoopLoader`]
//!   (no ambient network — a remote fetch raises `loading document failed`), plus the
//!   local-fixture [`FsLoader`].
//! - [`context`] — the **Context Processing** foundation (bead `sq-oy1f.24`): the
//!   `ActiveContext`, Create Term Definition, and IRI Expansion. Bead `sq-90mu3` adds the
//!   compaction-side companions: Inverse Context Creation (§4.3), IRI Compaction (§7.1),
//!   and Term Selection (§7.2), exposed as `ActiveContext::inverse_context` and
//!   `compact_iri`.
//! - [`expand`](mod@expand) — the document-level **Expansion Algorithm** + Value Expansion
//!   (bead `sq-oy1f.25`): [`expand`](expand::expand) turns a compact document into its
//!   canonical expanded form (scoped contexts, container maps, `@nest`, `@reverse`,
//!   `@included`, `@json`, keyword aliases), threading `frameExpansion` for the framing
//!   pipeline.
//!
//! The remaining algorithm modules ([`node_map`], [`flatten`], [`compact`], [`frame`],
//! [`from_rdf`], [`to_rdf`], [`api`]) are **stubs**: they carry the spec references and the
//! public shape only, filled by the dependency-ordered follow-on beads. Nothing panics: the
//! crate is `todo!()`-free.

// Real, shipped surfaces.
pub mod context;
pub mod error;
pub mod json;
pub mod loader;
pub mod options;

// Algorithm scaffolds — spec references + public shape only (filled by later beads).
pub mod api;
pub mod compact;
pub mod expand;
pub mod flatten;
pub mod frame;
pub mod from_rdf;
pub mod node_map;
pub mod to_rdf;

pub use context::inverse::{compact_iri, InverseContext};
pub use context::{ActiveContext, Direction, Override, TermDefinition};
pub use error::{JsonLdError, JsonLdErrorCode};
pub use expand::expand;
pub use json::{Json, JsonParseError};
pub use loader::{DocumentLoader, FsLoader, NoopLoader, RemoteDocument};
pub use options::{EmbedFlag, JsonLdOptions, ProcessingMode, RdfDirection};
