// [OPUS-4.8] sq-yyro: the DEALER-LESS correlated-randomness seam — PRSS /
// honest-majority coin-toss / dealer-less VSS. Design + seam only; NO dealer-less
// crypto ships here. See research/mpc-distributed-randomness-design.md.
//! Distributed / dealer-less correlated randomness — the seam that will replace
//! the single-trusted-dealer simulation (sq-yyro).
//!
//! ## Why this module exists
//!
//! Masks and correlated randomness are **security-critical** and, in a REAL
//! federation, must be generated so that **no single party knows them**. Today a
//! single [`crate::shamir::ShamirDealer`] owns the masking CSPRNG and draws every
//! mask / share centrally — an honest **single-dealer SIMULATION**, but not a
//! federation-ready source: the dealer *knows* every mask (it drew it). This
//! module names the contract a dealer-less source must satisfy and labels the
//! current dealer honestly as the simulation it is.
//!
//! The randomness *quality* is already honest (sq-1vt: OS-seeded ChaCha20,
//! uniform rejection sampling — see [`crate::rng`]). The gap this seam addresses
//! is **who** draws it: PRSS (replicated-PRF, non-interactive, small-`n`) or a
//! distributed coin-toss (interactive, any-`n`) generate correlated randomness
//! jointly, and **dealer-less VSS** lets each holder share its OWN input
//! verifiably. See `research/mpc-distributed-randomness-design.md` for the
//! PRSS-vs-coin-toss decision and the dealer-less-VSS construction.
//!
//! ## The `r = 0` threat (the load-bearing correctness point)
//!
//! [`secure_equal`](crate::compare::secure_equal_to_bit) hides a key difference
//! `d = a − b` behind `m = d · r` and opens `m`; the verdict `m == 0 ⇔ a == b`
//! holds **iff `r ≠ 0`**. An adversary who forces the mask to `r = 0` makes
//! `m = 0` for EVERY pair — a false match on every row. So the equality mask must
//! be uniform, **nonzero**, and jointly generated so no party controls it. Under
//! the current single honest dealer this is guaranteed by
//! [`draw_nonzero_fp`](crate::shamir::ShamirDealer::draw_nonzero_fp); under a
//! semi-honest PRSS/coin-toss source `r` is uniform + unknown but **NOT defended
//! against an ACTIVE adversary** — enforcing `r ≠ 0` against a malicious
//! contributor needs the IT-MAC'd open (`mpc-malicious-security-design.md`,
//! sq-km34.*) or a distributed nonzero zero-test-and-redraw. That defense is a
//! FOLLOW-ON bead; [`DistributedRandomness::shared_nonzero_mask`] is documented
//! semi-honest-only until it lands.
//!
//! ## Status — seam only, no fake crypto
//!
//! The ONLY implementor here is [`ShamirDealer`], which reports
//! [`RandomnessModel::TrustedDealerSim`] (`deployable() == false`).
//! The dealer-less implementors ([`RandomnessModel::Prss`] /
//! [`RandomnessModel::HonestMajorityCoinToss`]) are follow-on beads behind this
//! same trait — no dealer-less crypto is implemented in this module.

use crate::field::Fp;
use crate::partial::MpcError;
use crate::shamir::{ShamirDealer, Share};

/// Which correlated-randomness regime is producing masks — an **honest
/// self-description** so a caller (or a `BackendInfo`-style report) can state
/// whether the randomness source is federation-ready, never guess it from the
/// concrete type.
///
/// See `research/mpc-distributed-randomness-design.md` §2 for the PRSS-vs-coin-toss
/// decision behind the two dealer-less variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RandomnessModel {
    /// The CURRENT in-process **single-trusted-dealer simulation**: one
    /// [`ShamirDealer`] draws every mask from the CSPRNG and therefore *knows*
    /// it. Honest for testing / the in-process protocol simulation, but **NOT
    /// deployable** to a real federation (there is no trusted dealer). This is
    /// the only variant that ships today.
    TrustedDealerSim,
    /// **PRSS** — replicated-PRF pseudo-random secret sharing (eprint 2021/1223,
    /// IETF draft-thomson-ppm-prss). Non-interactive (0 online rounds) after a
    /// one-time replicated-seed setup; the primary choice for **small `n`**
    /// (the four-flatmates target). Dealer-less. Follow-on bead — not built.
    Prss,
    /// **Distributed coin-toss** — commit-open (or VSS-summed) joint randomness.
    /// Interactive (≥1 round per batch) but needs no combinatorial setup, so it
    /// is the general-`n` / setup-free fallback and the malicious-with-abort
    /// upgrade path. Dealer-less. Follow-on bead — not built.
    HonestMajorityCoinToss,
}

impl RandomnessModel {
    /// Whether the randomness is generated **without a trusted dealer** — i.e.
    /// no single party knows a mask. Only [`Self::TrustedDealerSim`] is not.
    pub fn is_dealer_less(self) -> bool {
        match self {
            RandomnessModel::TrustedDealerSim => false,
            RandomnessModel::Prss | RandomnessModel::HonestMajorityCoinToss => true,
        }
    }

    /// Whether this source is honest to deploy to a REAL federation. The
    /// single-dealer simulation is not (the dealer knows every mask); the
    /// dealer-less regimes are. Exactly the negation of "is a simulation" today,
    /// but kept distinct so a future *insecure* dealer-less variant could report
    /// `is_dealer_less() == true` while `deployable() == false`.
    pub fn deployable(self) -> bool {
        self.is_dealer_less()
    }
}

/// The correlated-randomness contract the MPC protocol consumes, independent of
/// **how** the randomness is produced (single dealer today; PRSS / coin-toss in a
/// real deployment). A dealer-less source is a new type implementing this trait,
/// selected by [`RandomnessModel`] — never by concrete type — so the join /
/// comparison / oblivious-shuffle callers compose onto it unmodified. This is the
/// randomness analogue of the [`crate::backend::MpcBackend`] primitive seam.
///
/// See `research/mpc-distributed-randomness-design.md` for the full design and
/// the follow-on implementation beads.
pub trait DistributedRandomness {
    /// Honest self-description of which regime is producing the randomness — so a
    /// caller can refuse a non-`deployable()` source in production (fail toward
    /// the operator) rather than silently run the simulation as if it were a
    /// federation.
    fn randomness_model(&self) -> RandomnessModel;

    /// A fresh degree-`t` sharing of a **uniform** mask that no `≤ t` parties
    /// know. Used wherever the protocol needs correlated randomness whose value
    /// is irrelevant as long as it is uniform and secret (e.g. blinding a value
    /// before an open, degree-reduction re-sharing randomness).
    fn shared_mask(&mut self) -> Result<Vec<Share>, MpcError>;

    /// A fresh degree-`t` sharing of a uniform **nonzero** mask `r` — the
    /// equality-test mask (see the module `r = 0` threat note). The nonzero
    /// guarantee is the whole correctness of `secure_equal`.
    ///
    /// **Semi-honest only.** A semi-honest dealer-less source yields a uniform,
    /// party-unknown `r`, but does NOT defend `r = 0` against an *active*
    /// adversary that biases its contribution; that needs the IT-MAC'd open or a
    /// distributed nonzero zero-test (a follow-on bead). The current
    /// single-dealer impl is honest because the one dealer draws `r` nonzero.
    fn shared_nonzero_mask(&mut self) -> Result<Vec<Share>, MpcError>;

    /// A degree-`t` sharing of a holder's **own** `secret`. In a real deployment
    /// this is **dealer-less VSS**: the holder is the dealer of its own input and
    /// the other parties verify the sharing is a consistent degree-`t`
    /// polynomial (aborting on a cheater — the input-side twin of
    /// [`crate::robust`]'s output-side robustness). In the semi-honest
    /// simulation it is a plain sharing (the holder is assumed to share
    /// correctly).
    fn vss_own_input(&mut self, secret: Fp) -> Result<Vec<Share>, MpcError>;
}

/// The CURRENT single-trusted-dealer **SIMULATION** implements the seam.
///
/// [`ShamirDealer`] owns the masking CSPRNG and therefore *knows* every mask it
/// produces — so it reports [`RandomnessModel::TrustedDealerSim`] and is **NOT
/// deployable** to a real federation. Implementing the trait here (rather than
/// leaving it an unused stub) pins the exact call-shape a future dealer-less PRSS
/// / coin-toss source must satisfy and lets callers migrate to
/// `&mut dyn DistributedRandomness` incrementally, with the dealer-less source
/// swapped in behind the same trait. No behaviour changes: each method routes to
/// the dealer's existing inherent randomness methods.
impl DistributedRandomness for ShamirDealer {
    fn randomness_model(&self) -> RandomnessModel {
        RandomnessModel::TrustedDealerSim
    }

    fn shared_mask(&mut self) -> Result<Vec<Share>, MpcError> {
        // Simulation artefact: the dealer draws AND knows `r`. A dealer-less
        // source would generate `[r]` jointly (PRSS/coin-toss) so no party knows it.
        let r = self.draw_fp();
        Ok(self.share(r))
    }

    fn shared_nonzero_mask(&mut self) -> Result<Vec<Share>, MpcError> {
        // Honest here only because the one dealer draws `r != 0` (rejection loop).
        // A semi-honest dealer-less source needs the active `r = 0` defense — see
        // the module note.
        let r = self.draw_nonzero_fp();
        Ok(self.share(r))
    }

    fn vss_own_input(&mut self, secret: Fp) -> Result<Vec<Share>, MpcError> {
        // Simulation: the dealer shares on the holder's behalf. Dealer-less VSS
        // would have the holder share its own input with a distributed
        // consistency check.
        Ok(self.share(secret))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Fp;
    use crate::shamir::ShamirBackend;

    fn seeded_dealer() -> ShamirDealer {
        // Deterministic simulation RNG (test-gated) so the seam is exercised
        // reproducibly; the production dealer draws OS-seeded ChaCha20.
        ShamirBackend::new_seeded(5, 0xC0FFEE_u64)
            .expect("n >= 2")
            .dealer()
    }

    #[test]
    fn dealer_reports_the_honest_simulation_model() {
        let dealer = seeded_dealer();
        let model = dealer.randomness_model();
        assert_eq!(model, RandomnessModel::TrustedDealerSim);
        // The whole point of the seam: the current source is NOT dealer-less and
        // NOT deployable to a real federation — labelled honestly, not silently.
        assert!(!model.is_dealer_less());
        assert!(!model.deployable());
    }

    #[test]
    fn dealer_less_models_are_deployable() {
        for m in [
            RandomnessModel::Prss,
            RandomnessModel::HonestMajorityCoinToss,
        ] {
            assert!(m.is_dealer_less(), "{m:?} must be dealer-less");
            assert!(m.deployable(), "{m:?} must be deployable");
        }
    }

    #[test]
    fn shared_mask_is_a_full_degree_t_sharing() {
        let mut dealer = seeded_dealer();
        let shares = dealer.shared_mask().expect("mask");
        // n shares on the canonical points x = 1..n; reconstructible by the
        // backend (a well-formed degree-t sharing of the drawn mask).
        assert_eq!(shares.len(), dealer.parties());
    }

    #[test]
    fn shared_nonzero_mask_never_shares_zero() {
        // The r = 0 threat: the equality mask must be nonzero. Reconstruct the
        // sharing and assert the secret is never zero across many draws.
        let backend = ShamirBackend::new_seeded(5, 7).expect("n >= 2");
        for _ in 0..1_000 {
            let mut dealer = backend.dealer();
            let shares = dealer.shared_nonzero_mask().expect("nonzero mask");
            let r = backend.reconstruct(&shares).expect("reconstruct");
            assert_ne!(r, Fp::zero(), "equality mask must never be r = 0");
        }
    }

    #[test]
    fn vss_own_input_reconstructs_to_the_holders_value() {
        let backend = ShamirBackend::new_seeded(5, 11).expect("n >= 2");
        let mut dealer = backend.dealer();
        let secret = Fp::new(4085);
        let shares = dealer.vss_own_input(secret).expect("vss");
        assert_eq!(backend.reconstruct(&shares).expect("reconstruct"), secret);
    }
}
