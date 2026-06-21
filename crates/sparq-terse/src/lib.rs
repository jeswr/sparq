#![doc = include_str!("../README.md")]
// [OPUS-4.8] sq-leg8n: render the feature gating on docs.rs.
#![cfg_attr(docsrs, feature(doc_cfg))]

// ---------------------------------------------------------------------------------------
// Phase 1 — the verifiable transpiler skeleton (always available, dependency-light).
// ---------------------------------------------------------------------------------------
pub mod transpile;

pub use transpile::{
    scan_v_calls, splice_v_calls, terse_to_sparql, validate_canonical, Expansion, Resolution,
    ResolutionMethod, TerseError, VCall,
};

// ---------------------------------------------------------------------------------------
// Phase 2 — V("phrase") concept resolution (the `link` feature: lexical-first), with the
// vector fallback behind the `vectors` feature. Both keep the default build a pure
// transpiler skeleton with no engine/model dependency.
// ---------------------------------------------------------------------------------------
#[cfg(feature = "link")]
#[cfg_attr(docsrs, doc(cfg(feature = "link")))]
pub mod resolve;

#[cfg(feature = "link")]
#[cfg_attr(docsrs, doc(cfg(feature = "link")))]
pub use resolve::{ResolveConfig, Resolver};

#[cfg(feature = "vectors")]
#[cfg_attr(docsrs, doc(cfg(feature = "vectors")))]
pub mod vector;

#[cfg(feature = "vectors")]
#[cfg_attr(docsrs, doc(cfg(feature = "vectors")))]
pub use vector::VectorBackend;
