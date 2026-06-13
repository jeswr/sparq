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
//!   dishonest-majority become swappable impls. Interface only; the trust-model
//!   decision point is documented, no primitive is chosen.
//! - [`join`] — **(§2 convention #6; §4.3 step 4; §5.2 Q3)** The [`GlobalJoin`]
//!   trait + [`DisclosedKeyJoin`] (M2): joining holders' partials on GLOBAL IRIs
//!   (the distinguishing feature vs node-local-id graph-MPC). The disclosed-key
//!   equi-join is REAL and crypto-free (the key is a public global IRI, joined
//!   in the clear OUTSIDE the cryptographic core, convention #4). The hidden-
//!   value (private-key) join stays a gated `NotYetImplemented` (M3 + Q2/Q3).
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

pub mod backend;
pub mod field;
pub mod holder;
pub mod join;
pub mod partial;
pub mod proof;
pub mod shamir;

pub use backend::{BackendInfo, MpcBackend, TrustModel};
pub use field::Fp;
pub use holder::{Holder, HolderResult};
pub use join::{DisclosedKeyJoin, GlobalJoin, HiddenKeyedRows, HiddenValueJoin, JoinPlan};
pub use partial::{HolderId, MpcError, PartialResult};
pub use proof::{Attestation, CollaborativeProof, ProofStatement};
pub use shamir::{Share, ShamirBackend};
