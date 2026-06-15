// [OPUS-4.8] Milestone-0 scaffold for MPC over federated SPARQL (RQ2).
//! # sparq-mpc — MPC over federated SPARQL (RQ2)
//!
//! Distributed half of the verifiable-data-sublanguage vision: a set of
//! mutually-distrusting **holders** jointly evaluate ONE SPARQL query over the
//! *union* of their privately-held, issuer-signed RDF named graphs and produce
//! one verifiable response carrying a zero-knowledge proof that the result is
//! (a) the correct PAG-semantics evaluation of the query AND (b) derived only
//! from issuer-attested sources — while disclosing the minimum inter-source
//! information needed to compute it.
//!
//! Blueprint: `research/mpc-zkp-research-and-architecture.md`. This crate is
//! the engine-side sibling of the single-holder ZK estate (`sparq-zk`,
//! `sparq-zk-compose`) and wraps it rather than replacing it (architecture
//! §4.4).
//!
//! ## What Milestone 0 actually delivers (and what it deliberately does not)
//!
//! This is a CONSERVATIVE scaffold. It builds ONLY the parts that are
//! **invariant** to the two open design forks (architecture §5.2):
//!
//! - **Q1 (research risk):** verifying a holder's BBS+/EdDSA signature over a
//!   *secret-shared* witness inside a collaborative proof is unsolved in the
//!   literature — "the join nobody has built". See [`proof`].
//! - **Q2 (trust model):** honest-majority vs dishonest-majority reshapes the
//!   whole MPC primitive choice. See [`backend`].
//!
//! Everything fork-dependent is an explicit, compiling stub that returns
//! [`MpcError::NotYetImplemented`] with the gating milestone/issue named — NO
//! fake crypto. The scaffold's value is the invariant structure + a REAL
//! working per-holder local sub-evaluation ([`holder`]) + the build plan
//! (`PLAN.md`).
//!
//! ## Module map (each cites its architecture section)
//!
//! - [`holder`] — **(§4.1 Parties; §4.3 step 1; §4.2 "minimise data
//!   sharing")** Per-holder local SPARQL sub-evaluation. The one piece that is
//!   real and tested at M0: each holder evaluates a query fragment over its
//!   OWN named graphs locally via `sparq-engine`, returning local partial
//!   results. Holders never ship raw graphs — this IS the invariant
//!   minimise-data-sharing core.
//! - [`backend`] — **(§3.1; §4.2 trust model; §5.2 Q2)** The [`MpcBackend`]
//!   trait abstracts the secret-sharing / MPC primitive so honest- vs
//!   dishonest-majority become swappable impls. Q2 is RESOLVED FOR v1:
//!   honest-majority. The configurability seam is documented here.
//! - [`field`] / [`shamir`] — **(§3.1; §4.2; §4.3 step 4) — M3, REAL crypto.**
//!   [`shamir::ShamirBackend`] is the first concrete [`MpcBackend`]: honest-
//!   majority Shamir `t`-of-`n` secret sharing over the prime field [`field`].
//!   It secret-shares a holder's private input, runs the secure cumulative-sum
//!   aggregate (zero-round local addition), reconstructs only the disclosed
//!   output, and supplies the secret-shared equality primitive the hidden-value
//!   join uses. Semi-honest. **Masking randomness is a CSPRNG** ([`rng`],
//!   sq-1vt): the dealer's coefficients and the equality mask come from
//!   OS-seeded ChaCha20; a deterministic PRNG is available only behind a
//!   test-only feature gate.
//! - [`join`] — **(§2 convention #6; §4.3 step 4; §5.2 Q3)** The [`GlobalJoin`]
//!   trait + [`DisclosedKeyJoin`] (M2, crypto-free disclosed-key equi-join over
//!   GLOBAL IRIs) AND [`HiddenValueJoin`] (M3): joining on a PRIVATE key via the
//!   Shamir-backed secret-shared equality test, disclosing only the result
//!   payload — the capability M2 could not provide. (The crypto-free path
//!   remains the default where the key is a public global IRI, convention #4.)
//! - [`oblivious`] — **(§3 the substrate gap; §4.1 L2; §8 step 1) — sq-18lk, the
//!   keystone hidden-regime primitive.** Oblivious **shuffle** (Waksman/Beneš
//!   permutation network over secret-shared columns — sound today; the in-process
//!   simulation routes via cleartext control bits held by the dealer, so each
//!   switch is a local swap costing **0 multiplications**, while the *deployed*
//!   protocol uses **secret** control bits and pays **1 multiplication per
//!   switch** for the arithmetic conditional swap — surfaced via
//!   [`oblivious::ShuffleCost`]) + oblivious **sort** (Batcher odd-even mergesort
//!   network
//!   whose compare-exchange access pattern is data-independent — the obliviousness
//!   substrate). DISTINCT / ORDER BY / GROUP BY-over-hidden / MIN-MAX /
//!   OPTIONAL-MINUS / the set-returning oblivious-join output path / ~linear joins
//!   all reduce to it. The disclosed-key sort is sound; the secret-key comparator
//!   is honestly gated on degree reduction (no fake secure comparison). See the
//!   module docs for the per-primitive security/leakage statement.
//! - [`proof`] — **(§4.3 step 5; §4.4; §5.1 hard dependency; §5.2 Q1)** The
//!   [`CollaborativeProof`] / [`Attestation`] boundary that will emit the ZKP
//!   that the result is correct AND issuer-attested. Interface + doc; impl
//!   gated on the ZK-foundation remediation (#3 issuer-sig / #4 replay / #5/#6
//!   FILTER-binding / #8/#9 attribution / #12 revocation) and on Q1.
//! - [`partial`] — shared value types crossing module boundaries
//!   ([`PartialResult`], [`HolderId`], [`MpcError`]). No crypto.
//!
//! ## Why native-only / not in the wasm build
//!
//! MPC is inter-process / inter-host and the eventual crypto (secret sharing,
//! collaborative proving) has no browser story at this milestone. Keeping
//! `sparq-mpc` out of `sparq-wasm`'s dependency graph guarantees the browser
//! bundle carries zero MPC surface (mirrors how `sparq-zk` is isolated).
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

pub mod backend;
pub mod field;
pub mod holder;
pub mod join;
pub mod partial;
pub mod proof;
// [OPUS-4.8] sq-1vt: the CSPRNG masking seam (production SecureRng + test-only
// InsecureTestRng). The real protocol's secret-sharing randomness lives here.
pub mod rng;
// [OPUS-4.8] sq-m34i (MPC WI-1): Reed-Solomon consistency-checked + robust
// (Berlekamp-Welch) reconstruction over Fp — detect-and-abort / correct tampered
// shares when redundancy is present. Closes malicious-security gap (D) at the
// Shamir layer (parent bead sq-uu0u). See the module docs for the threat model.
pub mod robust;
pub mod shamir;
// [OPUS-4.8] sq-18lk: oblivious shuffle (Waksman/Benes net) + sort (Batcher
// odd-even mergesort) substrate over Shamir Fp — the keystone hidden-regime primitive
// (ORQ SOSP'25). DISTINCT / ORDER BY / GROUP BY-over-hidden / MIN-MAX /
// OPTIONAL-MINUS / the set-returning oblivious-join output path / ~linear joins
// all reduce to it. The shuffle is sound today; the sort NETWORK + its
// data-independent access pattern (the substrate) are sound, with the secret-key
// comparator honestly gated on degree reduction. See the module docs.
pub mod oblivious;
// [OPUS-4.8] sq-jnkm: oblivious result-size protection + match-bit aggregation
// output path for SET-returning hidden joins, built on the sq-18lk shuffle
// substrate. Closes leaks L1 (true result cardinality, padded to a public bound)
// and L2 (the per-pair match graph / key fan-out, destroyed by the oblivious
// shuffle) that HiddenValueJoin's per-pair open exposes. The output TRANSFORM +
// the disclosed-key path are sound today; deriving the secret match bit from
// secret keys WITHOUT opening it is honestly gated on secure-compare
// (sq-rrz4/sq-dvuc). See the module docs for the residual-leakage statement.
pub mod oblivious_join;

// [OPUS-4.8] sq-nuok: adversarial-share negative suite + 'no fake crypto' stub
// gate. Test-only; compiled only under `cfg(test)` so it can drive the seedable
// simulation RNG (`ShamirBackend::new_seeded`) and the deferred-stub trait
// surface. Asserts the honest-but-robust properties that ARE claimed and PINS
// that the deferred parts fail closed. See the module docs.
#[cfg(test)]
mod adversarial_tests;

pub use backend::{
    AbortKind, AdversaryModel, BackendInfo, CorruptionThreshold, MaliciousSecurity, MpcBackend,
    OperatorClass, OutputGuarantee, PublicVerifiability, SecurityDescriptor, TrustModel,
};
pub use field::Fp;
pub use holder::{Holder, HolderResult};
pub use join::{DisclosedKeyJoin, GlobalJoin, HiddenKeyedRows, HiddenValueJoin, JoinPlan};
pub use partial::{HolderId, MpcError, PartialResult};
pub use proof::{Attestation, CollaborativeProof, ProofStatement};
pub use oblivious::{
    shuffle, sort_by, sort_with_keys, AccessPattern, Comparator, SecretColumn, ShuffleCost,
    SortByResult, SortCost, SortWithKeysResult, SortingNetwork, Switch, WaksmanNetwork,
};
// [OPUS-4.8] sq-jnkm: the oblivious set-returning output path surface.
pub use oblivious_join::{
    oblivious_join_output, oblivious_set_output, oblivious_set_output_hidden_keys, Candidate,
    MatchBit, ObliviousOutput, ObliviousOutputCost, OutputSlot,
};
pub use rng::{MpcRng, SecureRng};
pub use robust::reconstruct_robust;
pub use shamir::{ShamirBackend, Share};
