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
//! - **Phase 3 (default build): the `K:<name>` keyword layer (lever 1).** A small, fixed,
//!   versioned legend ([`legend`], [`LEGEND_VERSION`]) of the PKG hot predicates/classes,
//!   expanded pre-parse: `K:derivedFrom` becomes `<http://www.w3.org/ns/prov#wasDerivedFrom>`
//!   so an agent need not emit a `PREFIX` line. Every expansion is echoed in
//!   [`Expansion::keywords`]; an unknown keyword is [`TerseError::UnknownKeyword`] and a
//!   collision with a real `PREFIX K:` is [`TerseError::KeywordPrefixCollision`] — never a
//!   silent guess. Publish the legend behind the prompt-cache breakpoint with [`legend_card`].
//! - **Phase 4 (default build): the non-rewriting did-you-mean diagnostic (design §3.2).**
//!   On a *parse failure only*, [`TerseError::CanaryFailed`] carries [`KeywordHint`]s naming
//!   the tokens that look like mistyped SPARQL keywords (`FLTR` → `FILTER`). The suggestion
//!   is **never applied** — lenient parsing could silently yield a different, valid, wrong
//!   query — so the agent keeps its loud, recoverable feedback loop. [`keyword_hints`]
//!   exposes the same diagnostic for a parse error obtained elsewhere.
//! - **Phase 2 (`vectors` feature):** `V("phrase")` lexical-first concept resolution behind
//!   the §6 soundness envelope — `terse_to_sparql_with` (the `vectors`-gated entry point)
//!   expands each `V("phrase")` to the
//!   canonical `<iri>` it resolves to, echoing IRI + score + runner-up + confidence +
//!   method in [`Expansion::resolutions`], confidence-gated and staleness-guarded.

mod diagnose;
mod error;
mod keyword;
mod resolve;
mod transpile;

pub use diagnose::{keyword_hints, KeywordHint};
pub use error::TerseError;
pub use keyword::{legend, legend_card, legend_len, KeywordExpansion, LEGEND_VERSION};
pub use resolve::{Method, Resolution};
pub use transpile::{terse_to_sparql, Expansion};

#[cfg(feature = "vectors")]
pub use resolve::{ResolveCtx, ResolveGate};
#[cfg(feature = "vectors")]
pub use transpile::terse_to_sparql_with;
