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
//! - [`ingest`] — the trusted-ingest Stage-2 path (sq-610 + sq-cn8): a salt
//!   mint that draws globally-unique per-graph RDFC10 salts from the OS CSPRNG
//!   and per-named-graph commitment wired through to the zk-trace consumer.
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
//!
//! ## FIPS posture (CR-G4) [OPUS-4.8]
//!
//! The primitives here — BN254, Poseidon2, and Schnorr over Baby-JubJub
//! ([`sig`]) — are **not** FIPS-approved and are **not** inside any FIPS 140-3 /
//! CMVP-validated module. sparq makes **no FIPS claim and no CMVP claim**; these
//! are ZK-friendly primitives chosen for in-circuit efficiency, not regulatory
//! approval. This crate is `publish = false` and not in the default dependency
//! graph, so a FIPS-constrained operator keeps it out of FIPS scope by not
//! opting in. See `compliance/cryptoreview/fips-posture.md`.
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

pub mod canon;
pub mod commit;
// [OPUS-4.8] sq-xojl: DUAL-LEAF value-lane host encoding (VALUE_HOOK + lexical
// leaf). OPT-IN behind the `dual-leaf` cargo feature (OFF by default): the
// default string-canonical pipeline is byte-unchanged. Carries the documented
// INV-VL downgrade (#769 accepted, CR-G8 / sq-qhy4); NOT externally audited.
#[cfg(feature = "dual-leaf")]
pub mod dual_leaf;
pub mod encode;
pub mod field;
pub mod ingest;
// [FABLE-5] sq-we9vs: dual-leaf `xsd:dateTime`/`xsd:date` host encoders — the
// §13 signed scaled-epoch, Z-only, fail-closed lanes (same feature gate, same
// INV-VL downgrade + CR-G8 / sq-qhy4 audit obligation as `dual_leaf`).
#[cfg(feature = "dual-leaf")]
pub mod dual_leaf_datetime;
pub mod poseidon2;
mod poseidon2_constants;
pub mod registry;
// [OPUS-4.8] sq-bevd3 (Phase 3, epic sq-0dksu): the PER-METHOD security-properties
// annotation graph + the over-claim guards (no `Proven` on a positive property while
// sq-qhy4 is open; completeness; source-layer non-transfer). OPT-IN behind the
// `secprop-annotations` cargo feature (OFF by default): the default build is
// byte-unchanged and the module is compiled out. RECORDS claims + their epistemic
// basis; asserts NO soundness/privacy property (estate NOT externally audited,
// sq-qhy4).
#[cfg(feature = "secprop-annotations")]
pub mod secprop;
pub mod sig;
pub mod trace;
pub mod verify;

pub use field::Fr;
