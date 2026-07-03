#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)] // [FABLE-5] sq-pbz04.4.1: this crate has zero `unsafe`.

//! 🤖 SPARQ agent. See the crate README (rendered above) for the capability overview and the
//! honest boundary. This is layer **L1** of the OWL 2 Direct-Semantics workstream (epic
//! sq-pbz04.4, design record `research/owl2-direct-semantics-scoping.md`): a purely
//! **structural** OWL model for the ALCH fragment plus a **fail-closed reverse RDF mapping**.
//! No reasoning happens here — L1 turns an RDF graph into a typed [`Ontology`] it understood
//! IN FULL, or refuses it with a typed [`ExtractError`].
//!
//! The load-bearing entry point is [`extract()`] (`(Dict, &[[Id; 3]])` → [`Ontology`] |
//! [`ExtractError`]); the typed model lives in [`model`]. The stub modules `profile`, `nnf`,
//! `tableau`, and `check` are RESERVED for the later layers (L2–L4) and carry no logic yet —
//! see their module docs for the owning bead.

pub mod extract;
pub mod model;

// Stub modules pre-declared by L1 (design record §7) so L2/L3 populate `profile.rs` /
// `nnf.rs` / `tableau.rs` and L4 populates `check.rs` WITHOUT editing this file. They contain
// module documentation only — no logic.
pub mod check;
pub mod nnf;
pub mod profile;
pub mod tableau;

#[doc(inline)]
pub use extract::{extract, ExtractError};
#[doc(inline)]
pub use model::{Axiom, ClassExpression, ObjectPropertyExpression, Ontology};
