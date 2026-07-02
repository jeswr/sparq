//! Context processing — the active context, term definitions, and the inverse context.
//!
//! **Scaffold (`sq-oy1f.23`).** This module currently declares only the public *shape* the
//! Context Processing Algorithm and IRI expansion/compaction will operate over; the
//! algorithms land in bead `sq-oy1f.24`:
//!
//! - [`process`] — the Context Processing Algorithm (JSON-LD 1.1 API §4) and Create Term
//!   Definition (§4.2).
//! - [`iri`] — IRI Expansion (§5.2), IRI Compaction, and Term Selection (§7.1–7.2).
//!
//! Spec: <https://www.w3.org/TR/json-ld11-api/#context-processing-algorithms>.

pub mod iri;
pub mod process;

/// The **active context**: the processed result of applying a chain of `@context`
/// definitions (JSON-LD 1.1 API §3.1).
///
/// Scaffold placeholder — the term-definition map, default language/direction/base/vocab,
/// the previous-context link, and the cached inverse context are populated by the Context
/// Processing Algorithm in bead `sq-oy1f.24`.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct ActiveContext {}

/// One **term definition** from an active context (JSON-LD 1.1 API §3.2), including the
/// `@protected` flag that guards against redefinition.
///
/// Scaffold placeholder — the IRI mapping, type/language/direction/index/container/nest
/// mappings, the `@prefix`/`@reverse`/`@protected` flags, and the scoped local `@context`
/// are populated in bead `sq-oy1f.24`.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct TermDefinition {}

/// The **inverse context** derived from an active context, used to select the best term for
/// a given IRI/value during compaction (JSON-LD 1.1 API §4.3).
///
/// Scaffold placeholder — built lazily from the active context in bead `sq-oy1f.24`.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct InverseContext {}
