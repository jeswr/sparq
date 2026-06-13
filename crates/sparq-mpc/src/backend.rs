// [OPUS-4.8] MpcBackend trait — abstracts the secret-sharing / MPC primitive.
//! The MPC primitive abstraction. The trait is primitive-agnostic; the first
//! concrete impl is [`crate::shamir::ShamirBackend`] (honest-majority, M3).
//!
//! Architecture refs: §3.1 (secret-sharing families & network model), §4.2
//! (trust & threat model), and decision point **§5.2 Q2** (resolved for v1 =
//! honest-majority; configurable long-term).
//!
//! ## The decision point this trait exists to defer (Q2)
//!
//! The whole MPC primitive choice hinges on the trust model, and the literature
//! splits cleanly (architecture §3.1, §4.2):
//!
//! - **Honest-majority** (e.g. replicated 3PC secret sharing): the performance
//!   sweet spot, and — critically — the ONLY regime in which malicious-secure
//!   query-evaluation correctness (Senate/ORQ) AND authenticated inputs (Dutta,
//!   the attestation pillar) are both demonstrated.
//! - **Dishonest-majority malicious** (SPDZ/MASCOT/Overdrive): the realistic
//!   cross-org model for a hostile-pod federation, but pays expensive
//!   input-independent preprocessing and has NO demonstrated query-eval
//!   correctness in the cited literature.
//!
//! Network model compounds it: secret-sharing wins on LAN, garbled/constant-
//! round wins on WAN (the federated setting). The architecture's honest verdict
//! (§5.3) is that the *viable* first target is honest-majority, LAN/datacenter,
//! small data — and that whether the four-flatmates use case (cooperating
//! holders vs an external landlord) even NEEDS dishonest-majority among holders
//! is itself unresolved.
//!
//! **Therefore this trait commits to NO primitive.** It defines the seam so an
//! honest-majority impl (M3, first) and a dishonest-majority impl (later, if
//! Q2 resolves that way) are swappable without touching the join or proof
//! layers. Convention #7 (modularity is the contribution) demands exactly this.
//!
//! ## Q2 RESOLVED for v1 + how the trust model stays CONFIGURABLE
//!
//! Jesse's decision: **honest-majority for v1, configurable long-term.** The
//! concrete v1 impl is [`crate::shamir::ShamirBackend`] (honest-majority Shamir
//! `t`-of-`n`, semi-honest). The trait keeps that choice swappable:
//!
//! - **Callers select a backend by [`TrustModel`], never by concrete type.** A
//!   federation inspects [`BackendInfo`] and refuses a backend whose guarantees
//!   don't match its threat model. The join/proof layers are written against the
//!   `MpcBackend` trait (e.g. [`crate::join::HiddenValueJoin`] takes *a backend*,
//!   not "Shamir"), so substituting a dishonest-majority impl touches NO caller.
//! - **The associated [`MpcBackend::Share`] type absorbs the scheme change.**
//!   Shamir's share is a per-party polynomial-point vector; a SPDZ-style
//!   dishonest-majority backend's share is an *authenticated* additive share
//!   (value + MAC tag) with a preprocessing (triples) phase. Both hide behind
//!   `type Share`, so the difference never leaks into the join/proof signatures.
//! - **What a dishonest-majority (SPDZ/MASCOT/Overdrive) backend would add, all
//!   BEHIND this trait:** (1) an input-independent preprocessing step producing
//!   Beaver triples + MACs (a private step, not a trait change); (2) `run_secure`
//!   consuming triples for multiplications and tracking MACs; (3)
//!   `reconstruct_disclosed` doing a MAC-check before opening (abort on cheat →
//!   guarantee (D), malicious security). It reports
//!   `TrustModel::DishonestMajority` + `malicious_secure: true` via
//!   [`BackendInfo`]. Crucially the `share_private_input` / `run_secure` /
//!   `reconstruct_disclosed` SIGNATURES are unchanged, so
//!   [`crate::join::HiddenValueJoin`] and the future collaborative-proof layer
//!   compose onto it unmodified.
//!
//! No trust model is hardcoded into the join/protocol layer: it is a property of
//! the chosen `MpcBackend` value, surfaced via [`BackendInfo`].

use crate::holder::Holder;
use crate::partial::{MpcError, PartialResult};

/// The trust regime an [`MpcBackend`] implementation provides. This is the
/// Q2 axis made explicit in the type system so a federation can refuse a
/// backend whose guarantees do not match its threat model (§4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustModel {
    /// Security holds only if a strict majority of compute parties are honest.
    /// The viable first target (M3). Sub-variants (semi-honest vs malicious)
    /// are an impl detail surfaced via [`BackendInfo`].
    HonestMajority,
    /// Security holds against up to n-1 corrupt parties. The realistic
    /// cross-org model; deferred pending the Q2 decision and far heavier.
    DishonestMajority,
}

/// Static description of a concrete backend's guarantees — what a federation
/// inspects to decide whether a backend is acceptable BEFORE running anything.
#[derive(Debug, Clone)]
pub struct BackendInfo {
    /// Human-readable backend name (e.g. an eventual `"replicated-3pc"`).
    pub name: &'static str,
    /// The trust regime (the Q2 axis).
    pub trust_model: TrustModel,
    /// Whether the backend is secure against actively-malicious parties (as
    /// opposed to semi-honest / honest-but-curious only). Distinct from
    /// `trust_model`: §4.2's guarantee (D) malicious-security is orthogonal to
    /// the majority assumption.
    pub malicious_secure: bool,
}

/// Abstracts the secret-sharing / MPC primitive over which the federated SPARQL
/// operator pipeline runs (architecture §4.3 step 4).
///
/// The interface is deliberately minimal and primitive-agnostic at M0: it
/// captures the obligations every secret-sharing MPC must meet — share private
/// inputs, run the secure computation, reconstruct only the disclosed output —
/// WITHOUT naming a scheme. Associated type [`Self::Share`] stands in for the
/// scheme's share representation so honest- and dishonest-majority impls can
/// carry entirely different share types behind the same trait.
///
/// ## Q2 DECISION (resolved for v1)
/// Honest- vs dishonest-majority (and, on a WAN, secret-sharing vs garbled-
/// circuit) reshapes `Share`, the round structure, and the preprocessing model.
/// **v1 resolves Q2 to honest-majority** (Jesse's decision: honest-majority now,
/// configurable long-term) — the four *cooperating* flatmates prove an aggregate
/// to an external landlord; among themselves they are honest-but-curious, the
/// regime Shamir serves. Dishonest-majority remains future research behind this
/// same trait (see the module-level "how the trust model stays CONFIGURABLE").
///
/// ## Implementation status
/// First concrete implementor: [`crate::shamir::ShamirBackend`] (M3, honest-
/// majority Shamir `t`-of-`n`, semi-honest). It implements all three methods
/// below for real (over an in-process multi-party simulation). A dishonest-
/// majority impl slots in behind the same trait unchanged.
pub trait MpcBackend {
    /// The scheme-specific representation of a secret share. Opaque to the rest
    /// of the crate; only this trait's impl manipulates it.
    type Share;

    /// Static guarantees of this backend (trust model, malicious-security).
    fn info(&self) -> BackendInfo;

    /// Secret-share a holder's *private* contribution to the computation —
    /// the per-source intermediate values that must NEVER leave the source in
    /// the clear (architecture §4.3 step 4: "intermediate per-source values
    /// never leave a source"). Contrast [`Holder::evaluate_local`], which
    /// discloses; this path hides.
    ///
    /// Implemented by [`crate::shamir::ShamirBackend`] (M3): extracts the
    /// holder's single private integer and Shamir-shares it across the parties.
    fn share_private_input(&self, holder: &Holder) -> Result<Vec<Self::Share>, MpcError>;

    /// Run the secure computation over secret-shared inputs from all holders
    /// (e.g. the cumulative-salary aggregate whose per-source addends stay
    /// private). Returns the shares of the result.
    ///
    /// Implemented by [`crate::shamir::ShamirBackend`] (M3) for the cumulative
    /// sum — a pure linear function, so it is the zero-round local addition of
    /// the sharings (the honest-majority Shamir sweet spot).
    fn run_secure(&self, shares: &[Self::Share]) -> Result<Vec<Self::Share>, MpcError>;

    /// Reconstruct ONLY the disclosed output from result shares (the minimal
    /// answer — e.g. the cumulative integer, from which the verifier recomputes
    /// `cumulative > £100k` OUTSIDE the crypto), under the
    /// no-proof-of-revealed-properties discipline (§2 convention #4).
    ///
    /// Implemented by [`crate::shamir::ShamirBackend`] (M3) via Lagrange
    /// interpolation of the result sharing at `x = 0`.
    fn reconstruct_disclosed(&self, result_shares: &[Self::Share]) -> Result<PartialResult, MpcError>;
}
