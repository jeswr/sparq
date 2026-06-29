#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

// [OPUS-4.8] sq-fmprw / sq-ev41x (epic sq-qonbz, umbrella sq-6tykl) — see
// research/shared-eval-substrate.md.
//
// This crate is the KEYSTONE leaf of the shared-evaluation-substrate move-chain. It defines /
// re-exports the shared id-tuple vocabulary (`rows`) and now HOSTS the XSD numeric value tower
// (`numeric`) MOVED from `sparq-engine::exec` in Phase 2 (sq-ev41x): the `Num` / `Dec` types,
// the arithmetic / rounding ops, the EXACT XSD lexical parsers, the literal classifier
// `as_numeric`, and `num_compare` — all behind DEFAULT-OFF features. The move is
// BEHAVIOUR-NEUTRAL: the engine `use`s these and computes bit-identical answers (validated by
// the W3C SPARQL conformance floor + the ORDER BY / numeric / relop tests). The join kernels
// (merge / hash / bind / leapfrog-trie) and the full `compare_values` total order over the
// engine's `Value` land in LATER beads of the epic.
//
// ZERO-OVERHEAD CONTRACT: every shareable item is a FREE FUNCTION / method monomorphic over the
// concrete `Id = u32`, the `SmallVec` row aliases, and the concrete numeric types — NEVER
// `Box<dyn>` / `&dyn` / a vtable on a hot path (research record §2.3, §4). With the workspace
// `lto = "fat"` profile the compiler emits one specialised, inlinable body per call site, so the
// engine's FILTER / BIND / ORDER BY hot loops keep identical codegen after the move and the
// reasoners gain a real join. This crate introduces no dynamic dispatch.

#[cfg(feature = "rows")]
pub mod rows;

#[cfg(feature = "numeric")]
pub mod numeric;
