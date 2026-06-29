#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

// [OPUS-4.8] sq-ev41x (epic sq-qonbz, umbrella sq-6tykl) — see research/shared-eval-substrate.md.
//
// This crate is the KEYSTONE leaf of the shared-evaluation-substrate move-chain. It defines
// the shared id-tuple vocabulary (`rows`) and the XSD numeric value tower (`numeric`),
// behind DEFAULT-OFF features.
//
// PHASE 2 (sq-ev41x) has now landed `numeric`: the engine's id-level `Num` / `Dec` value
// tower + `as_numeric` classification + the numeric arithmetic ops (`binop` / `neg` / `abs`
// / rounding) and the XSD lexical helpers were MOVED here verbatim from `sparq-engine::exec`
// and the engine now consumes `sparq_substrate::numeric::{Num, Dec, as_numeric, ...}`. The
// move is behaviour-neutral (the W3C SPARQL conformance floor + ORDER BY / numeric / relop
// tests are bit-identical). STILL PENDING in later beads of the epic: the join kernels
// (merge / hash / bind / leapfrog-trie) and the engine's `compare_values` total order
// (which is irreducibly coupled to the engine's `Value` enum + the temporal subsystem, so it
// moves with `Value` in a follow-up, not here).
//
// ZERO-OVERHEAD INTENT (the contract every phase keeps): the shareable kernels are FREE
// FUNCTIONS / methods monomorphic over the concrete `Id = u32` and the numeric tiers —
// NEVER `Box<dyn>` / `&dyn` / a vtable on a hot path (research record §2.3, §4). Every
// `numeric` item carries `#[inline]`, so cross-crate inlining (with the workspace LTO
// profile) keeps the engine's FILTER / BIND / ORDER BY hot loops identical to pre-move.

#[cfg(feature = "rows")]
pub mod rows;

#[cfg(feature = "numeric")]
pub mod numeric;
