#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

// [OPUS-4.8] sq-fmprw (epic sq-qonbz, umbrella sq-6tykl) — see research/shared-eval-substrate.md.
//
// This crate is the KEYSTONE leaf of the shared-evaluation-substrate move-chain. It is a
// SCAFFOLD: it defines / re-exports the shared id-tuple vocabulary (`rows`) and the XSD
// numeric value tower (`numeric`), behind DEFAULT-OFF features, with NO evaluation logic
// moved yet. The join kernels (merge / hash / bind / leapfrog-trie) and the engine's
// `compare_values` total order + `Num` arithmetic land in LATER beads of the epic — this
// PR moves no engine or reasoner code and changes no behaviour anywhere.
//
// ZERO-OVERHEAD INTENT (the contract every later phase must keep): the shareable kernels
// will be FREE FUNCTIONS monomorphic over the concrete `Id = u32` and the `SmallVec` row
// aliases below — NEVER `Box<dyn>` / `&dyn` / a vtable between a join's probe and its key
// comparison (research record §2.3, §4). The compiler then emits one specialised, inlinable
// body per call site, so the engine's hot loops keep identical codegen after the move and
// the reasoners gain a real join. This scaffold introduces no dynamic dispatch.

#[cfg(feature = "rows")]
pub mod rows;

#[cfg(feature = "numeric")]
pub mod numeric;
