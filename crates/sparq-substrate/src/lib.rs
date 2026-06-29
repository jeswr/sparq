#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

// [OPUS-4.8] sq-ev41x (epic sq-qonbz, umbrella sq-6tykl) — see research/shared-eval-substrate.md.
//
// This crate is the KEYSTONE leaf of the shared-evaluation-substrate move-chain. It defines
// the shared id-tuple vocabulary (`rows`) and the XSD numeric value tower (`numeric`),
// behind DEFAULT-OFF features.
//
// PHASE 2 (sq-ev41x) landed `numeric`: the engine's id-level `Num` / `Dec` value tower +
// `as_numeric` classification + the numeric arithmetic ops (`binop` / `neg` / `abs` /
// rounding) and the XSD lexical helpers were MOVED here verbatim from `sparq-engine::exec`
// and the engine now consumes `sparq_substrate::numeric::{Num, Dec, as_numeric, ...}`.
//
// PHASE 3 (sq-hknqs) has now landed `join`: the FOUR id-tuple join kernels — sorted
// merge-join, radix-partitioned hash-join, index-nested-loop bind-join and leapfrog
// trie-join (WCOJ) — were MOVED here from `sparq-engine::exec`, generalised over a generic
// `JoinKeys` descriptor (the row→key projection + combine layout) and a generic `Budget`
// cooperative-cancel hook. The engine instantiates them with its own `Bindings`-derived key
// columns and its thread-local query budget; a future reasoner can instantiate the SAME
// probe loops with its own key projection — neither pays a vtable. The planner, `Bindings`,
// `LocalVocab` interning and `ScanCmp` filter pushdown stay engine-private.
//
// Both moves are behaviour-neutral (the W3C SPARQL conformance floor + the join/scan/BGP
// micro-benches are bit-identical / within noise). STILL PENDING in later beads of the epic:
// the engine's `compare_values` total order (irreducibly coupled to the engine's `Value`
// enum + the temporal subsystem, so it moves with `Value` in a follow-up — the deferred
// sq-vezew — NOT here; the join kernels deliberately do NOT hoist `Value`, the engine
// supplies its `Value`-based compare itself).
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

// [OPUS-4.8] sq-hknqs (epic sq-qonbz) — Phase 3 of the substrate move-chain. The FOUR id-tuple
// join kernels (merge / hash / bind / leapfrog-trie) now live here, MOVED from
// `sparq-engine::exec`, generic over a `JoinKeys` descriptor + a generic `Budget` hook so the
// engine and a future reasoner share one probe loop with NO `Box<dyn>`/`&dyn`/vtable between a
// join's probe and its key comparison (research/shared-eval-substrate.md §2.3, §4).
#[cfg(feature = "join")]
pub mod join;
