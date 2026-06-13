// [OPUS-4.8] MpcBackend trait — abstracts the secret-sharing / MPC primitive.
//! The MPC primitive abstraction (interface only; no crypto at M0).
//!
//! Architecture refs: §3.1 (secret-sharing families & network model), §4.2
//! (trust & threat model), and the load-bearing OPEN QUESTION **§5.2 Q2**.
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
//! Every method that would touch secret-shared data returns
//! [`MpcError::NotYetImplemented`] naming the gate. No method here fakes a
//! sharing scheme.

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
/// ## TODO(Q2) — THE DECISION POINT
/// Choosing honest- vs dishonest-majority (and, on a WAN, secret-sharing vs
/// garbled-circuit) reshapes `Share`, the round structure, and the preprocessing
/// model. Resolve Q2 (architecture §5.2) BEFORE the first concrete impl (M3).
/// The use-case question — do *cooperating* flatmates vs an external landlord
/// actually need dishonest-majority among holders? — gates this.
///
/// ## Implementation status
/// No implementor exists at M0. The first will be an honest-majority backend
/// (M3, see `PLAN.md`), behind this trait so it stays swappable.
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
    /// Gated on Q2 (no scheme chosen) and on the ZK foundation; returns
    /// [`MpcError::NotYetImplemented`].
    fn share_private_input(&self, holder: &Holder) -> Result<Vec<Self::Share>, MpcError>;

    /// Run the secure computation over secret-shared inputs from all holders
    /// (e.g. the cumulative-salary comparison whose per-source addends stay
    /// private). Returns the shares of the result.
    ///
    /// Gated on Q2; returns [`MpcError::NotYetImplemented`].
    fn run_secure(&self, shares: &[Self::Share]) -> Result<Vec<Self::Share>, MpcError>;

    /// Reconstruct ONLY the disclosed output from result shares (the minimal
    /// answer — e.g. a boolean `cumulative > £100k`), under the
    /// no-proof-of-revealed-properties discipline (§2 convention #4).
    ///
    /// Gated on Q2; returns [`MpcError::NotYetImplemented`].
    fn reconstruct_disclosed(&self, result_shares: &[Self::Share]) -> Result<PartialResult, MpcError>;
}
