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
//!   `TrustModel::DishonestMajority` + a non-`None`
//!   [`MaliciousSecurity`] via [`BackendInfo`]. Crucially the
//!   `share_private_input` / `run_secure` /
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

/// The malicious-security guarantee a backend delivers against *actively*
/// deviating parties — guarantee (D) of §4.2, orthogonal to [`TrustModel`]'s
/// majority axis. A `bool` is too coarse: WI-1/WI-2 gave the honest-majority
/// Shamir backend two *distinct* active-security levels depending on how much
/// reconstruction redundancy a given `(n, t)` configuration carries, and a
/// federation inspecting [`BackendInfo`] needs to tell them apart. The variants
/// are ordered from weakest to strongest. `[OPUS-4.8]`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaliciousSecurity {
    /// **No active-security guarantee.** Security holds only against
    /// semi-honest / honest-but-curious parties; an actively-deviating party can
    /// silently corrupt the output with no detection. The honest baseline for a
    /// not-yet-built / stub backend, and the guarantee for any reconstruction at
    /// exactly `degree + 1` shares (no RS redundancy — e.g. the degree-`2t`
    /// equality open at `n = 2t + 1`). NB: with the honest-majority
    /// `t = ⌊(n−1)/2⌋` the degree-`t` cumulative-aggregate path always has
    /// redundancy, so it never reports `None`.
    None,
    /// **Detect-and-abort (no guaranteed output).** Any tampering by an
    /// actively-deviating party is *detected* and the protocol aborts with a
    /// typed error rather than returning a wrong answer — but a single cheater
    /// can still force an abort, so a correct output is NOT guaranteed. This is
    /// the guarantee when there is at least one redundant share to cross-check
    /// (`n > t + 1`) but not enough to *correct* a cheater (`max_cheaters == 0`).
    HonestMajorityAbort,
    /// **Robust: guaranteed-correct output up to a cheater budget.** The protocol
    /// returns the *true* output even when up to `max_cheaters` parties actively
    /// deviate (Reed–Solomon / Berlekamp–Welch correction), and detect-and-aborts
    /// beyond that budget. `max_cheaters >= 1` always (a `0` budget is
    /// [`MaliciousSecurity::HonestMajorityAbort`], not this variant).
    HonestMajorityRobust {
        /// The maximum number of actively-cheating parties tolerated while still
        /// producing the guaranteed-correct output. For the honest-majority
        /// Shamir backend at degree `t` this is `e = ⌊(n − t − 1)/2⌋` (the RS
        /// correction bound); always `>= 1` for this variant.
        max_cheaters: usize,
    },
}

impl MaliciousSecurity {
    /// `true` iff the backend provides *some* active-security guarantee (detects
    /// or corrects an actively-deviating party), i.e. anything other than
    /// [`MaliciousSecurity::None`]. The backwards-compatible projection of the
    /// former `malicious_secure: bool` field, for callers that only need the
    /// coarse "is this hardened at all?" bit.
    pub fn is_malicious_secure(self) -> bool {
        self != MaliciousSecurity::None
    }
}

/// Static description of a concrete backend's guarantees — what a federation
/// inspects to decide whether a backend is acceptable BEFORE running anything.
#[derive(Debug, Clone)]
pub struct BackendInfo {
    /// Human-readable backend name (e.g. an eventual `"replicated-3pc"`).
    pub name: &'static str,
    /// The trust regime (the Q2 axis).
    pub trust_model: TrustModel,
    /// The malicious-security guarantee against actively-deviating parties
    /// (§4.2's guarantee (D)), orthogonal to `trust_model`. Replaces the former
    /// coarse `malicious_secure: bool` so the *kind* of active security
    /// (detect-and-abort vs robust-up-to-`max_cheaters`) is legible to a
    /// federation. Use [`BackendInfo::malicious_secure`] for the coarse bool.
    /// `[OPUS-4.8]`
    pub malicious_security: MaliciousSecurity,
}

impl BackendInfo {
    /// Backwards-compatible coarse bool: `true` iff this backend provides *any*
    /// active-security guarantee (i.e. `malicious_security != None`). Preserves
    /// the question the old `malicious_secure: bool` field answered for callers
    /// that don't need the finer variant.
    pub fn malicious_secure(&self) -> bool {
        self.malicious_security.is_malicious_secure()
    }
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
