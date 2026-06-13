// [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
//! # sparq-zk-compose — ZK proof composition for sparq (stage 2)
//!
//! Drives the per-property Noir circuit family at `zk/compose/` into a full
//! query-result proof, and verifies one (plan v3 §S4.E modules ii/iii).
//!
//! Pipeline:
//! 1. [`build`] turns sparq-zk [`GraphCommitment`]s + a BGP pattern into a
//!    scan [`ProofInputs`], deriving the (k, n, r) [`CircuitId`] from the data.
//! 2. [`driver::CircuitProver`] proves it via nargo/bb subprocesses.
//! 3. A [`ProofManifest`] carries the public inputs + bb proof to the verifier.
//! 4. [`verifier`] re-derives the circuit id, re-runs the sparq-zk bnode
//!    re-check (`sparq_zk::verify::recheck`), checks binding-consistency
//!    edges, and verifies each sub-proof via bb.
//!
//! Like `sparq-zk`, NOTHING in the workspace depends on this crate — default
//! builds and the wasm artifact are byte-identical with or without it.
//!
//! Model: Opus 4.8 (Fable 5 unavailable — flag for re-review/upgrade).

pub mod build;
pub mod driver;
pub mod manifest;
pub mod toml;
pub mod verifier;

pub use manifest::{
    AttestedStatusRef, BindingEdge, BindingMode, CircuitId, EntailmentRegime, FieldHex, FilterOp,
    ProofInputs, ProofManifest, RevocationStatus, StatusListSnapshot, SubProof,
};
// [OPUS-4.8] audit #4: verifier-issued nonce + single-use store.
pub use verifier::{InMemorySeenNonces, SeenNonces, VerifierNonce};
// [OPUS-4.8] audit #12: revocation / freshness policy.
pub use verifier::RevocationPolicy;
