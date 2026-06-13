//! # sparq-zk — ZK derived-credential foundation for sparq
//!
//! Engine-side half of the ZK derived-credentials design
//! (`research/zkp-query-proofs-plan.md` v3): the **commitment pipeline** and
//! the **zk-trace seam**. Circuits and proof composition are later
//! deliverables; everything here is off-circuit Rust whose outputs (BN254
//! field elements, leaf orderings, witness input sets) are what those
//! circuits will consume.
//!
//! Modules:
//! - [`field`] — BN254 scalar-field helpers (Noir `Field` compatible).
//! - [`poseidon2`] — Poseidon2-BN254 permutation + sponge, bit-compatible
//!   with noir-lang/poseidon (cross-tested against `nargo`).
//! - [`canon`] — RDFC10 canonicalization of a named graph's content
//!   (W3C rdf-canon test-suite validated).
//! - [`encode`] — per-term / per-triple field encoding with graph-scoped
//!   bnode salting (plan §2.2, Q6).
//! - [`commit`] — per-named-graph flat Poseidon2 commitments (plan §2.2).
//! - [`registry`] — `<urn:sparq:zk>` registry-graph plumbing (plan Q13),
//!   mirroring `sparq-solid`'s `<urn:sparq:auth>` conventions.
//! - [`sig`] — issuer signatures over per-graph commitments (audit #3):
//!   Schnorr over Baby-JubJub with a Poseidon2 challenge, the verifier-side
//!   sound interim that binds `C(G)` to an issuer key in a disclosed key-set.
//! - [`trace`] — the zk-trace seam (plan §4.E): per-obligation input sets
//!   from an executed query, leaf-index resolution against per-graph
//!   commitment orderings, and the Q6 cross-graph bnode-join guard
//!   (prover side, plan §2.4 layer 2).
//! - [`verify`] — the verifier-side static re-check (plan §2.4 layer 3):
//!   independent re-derivation of fragment patterns and cross-graph join
//!   obligations from the query text.
//!
//! NOTHING in the sparq workspace depends on this crate; default builds and
//! the wasm artifact are byte-identical with or without it.

pub mod canon;
pub mod commit;
pub mod encode;
pub mod field;
pub mod poseidon2;
mod poseidon2_constants;
pub mod registry;
pub mod sig;
pub mod trace;
pub mod verify;

pub use field::Fr;
