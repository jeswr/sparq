// [FABLE-5] sq-1rg2q: namespace for still-unlanded rdfjs/wrapper proposals.

//! Ports of still-unlanded rdfjs/wrapper proposals, each behind its own
//! default-off `proposed-*` cargo feature.
//!
//! Every submodule here only exists when its feature is enabled; with no
//! `proposed-*` feature this module is empty and contributes no code. The
//! typed focus kinds and bound node factories live in the feature-gated
//! `typed_focus` submodule (cargo feature `proposed-typed-focus`).

#[cfg(feature = "proposed-cardinality")]
pub mod cardinality;
#[cfg(feature = "proposed-codecs")]
pub mod codecs;
#[cfg(feature = "proposed-graph-scope")]
pub mod graph_scope;
#[cfg(feature = "proposed-observe")]
pub mod observe;
#[cfg(feature = "proposed-typed-focus")]
pub mod typed_focus;
