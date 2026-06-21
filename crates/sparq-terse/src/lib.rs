#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! [OPUS-4.8] sq-leg8n — see the crate README (rendered above) for the overview. This
//! module documentation covers the public API contract.
//!
//! The crate is a **pre-parse transpiler**: [`terse_to_sparql`] takes a terse query and
//! returns an [`Expansion`] whose [`Expansion::canonical_sparql`] is *always* standard,
//! conformant SPARQL — the exact text the engine runs and the agent inspects. It never
//! touches the vendored `spargebra` grammar (design §3, research
//! `llm-ergonomic-sparql-surface.md`).
//!
//! - **Phase 1 (default build):** the verifiable transpiler skeleton — an identity
//!   pass-through for canonical SPARQL plus the silent-rewrite canary (re-parse the output
//!   under `spargebra`; a non-parsing emission is [`TerseError::CanaryFailed`], never
//!   handed back). A `V("phrase")` construct in the default build fails loudly with
//!   [`TerseError::FeatureRequired`].
//! - **Phase 2 (`vectors` feature):** `V("phrase")` lexical-first concept resolution behind
//!   the §6 soundness envelope — `terse_to_sparql_with` (the `vectors`-gated entry point)
//!   expands each `V("phrase")` to the
//!   canonical `<iri>` it resolves to, echoing IRI + score + runner-up + confidence +
//!   method in [`Expansion::resolutions`], confidence-gated and staleness-guarded.

mod error;
mod resolve;
mod transpile;

pub use error::TerseError;
pub use resolve::{Method, Resolution};
pub use transpile::{terse_to_sparql, Expansion};

#[cfg(feature = "vectors")]
pub use resolve::{ResolveCtx, ResolveGate};
#[cfg(feature = "vectors")]
pub use transpile::terse_to_sparql_with;
