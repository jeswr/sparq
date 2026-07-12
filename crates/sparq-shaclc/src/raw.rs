//! The raw rdf-shuttle-generated parser modules — checked-in artifacts,
//! regenerated only by `scripts/regen-shuttle-parsers.sh` (provenance pins in
//! [`crate::provenance`]).
//!
//! Each module is one self-contained, dependency-free, `forbid(unsafe_code)`
//! streaming parser with its own term model (`Term` / `Triple` /
//! `SyntaxError` / `PushParser`). Use these directly for streaming/push
//! parsing on the generated term model; use the crate root's [`crate::parse`]
//! family for the `oxrdf`-typed convenience API.
//!
//! Both artifacts are PARSE-ONLY today: the print direction (serializer with
//! the residual-based "not compact-expressible" verdict) is derived in
//! rdf-shuttle's gen-js backend and lands here once gen-rs grows the same
//! derivation (tracked in bead sq-tonhr.6 / upstream).

/// Strict SHACL Compact Syntax 1.2 (`--profile rdf12`): the W3C CG surface
/// plus the RDF 1.2 layer. The four shaclc-js extensions are ABSENT from this
/// parser's tables, so it provably rejects them (the enforcement-leak fix).
pub mod shaclc12;

/// Extended SHACL Compact Syntax 1.2 (`--profile rdf12,ext`): everything in
/// [`shaclc12`] plus the four shaclc-js extensions (turtle-style node-shape
/// annotation lists, `% … %` property-shape escapes, trailing arbitrary
/// Turtle statements, the `a` keyword).
pub mod shaclc12ext;
