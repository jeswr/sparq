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
//!   `TrustModel::DishonestMajority` + a non-`SemiHonestOnly`
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

// =============================================================================
// [OPUS-4.8] sq-mq8q — THREE-AXIS SECURITY DESCRIPTOR (Fable unavailable;
// re-review when Fable returns).
//
// Background: the original two enums ([`TrustModel`] + [`MaliciousSecurity`])
// FLATTENED what the literature treats as THREE genuinely-orthogonal axes
// (research/mpc-security-models-and-benchmarks.md §1.2). `TrustModel` was a
// *binary* take on the corruption-threshold axis (it lost the n/3-vs-n/2 split
// that separates perfect-GOD from statistical-GOD), and `MaliciousSecurity`
// BUNDLED the adversary axis (semi-honest vs active) WITH the output-guarantee
// axis (abort vs robust) AND soldered "HonestMajority" into the *variant names*
// — so dishonest-majority-malicious-abort (= the SPDZ regime) was UNNAMEABLE,
// and covert / identifiable-vs-unanimous abort had NO representation.
//
// This module replaces that flattening with three orthogonal axes
// ([`AdversaryModel`], [`OutputGuarantee`], [`CorruptionThreshold`]) plus a
// [`PublicVerifiability`] marker, composed into a [`SecurityDescriptor`].
// Cleve's impossibility (STOC'86 — fairness/GOD is impossible without an honest
// majority) is encoded as a TYPE-LEVEL INVARIANT: the stronger output
// guarantees are only constructible under an honest-majority threshold (see
// [`OutputGuarantee::fairness`] / [`OutputGuarantee::guaranteed_output`], which
// are the ONLY way to build those variants and which take the threshold as
// proof of honest majority). The old [`MaliciousSecurity`] enum and the
// [`BackendInfo::malicious_security`] field are KEPT as a back-compat
// PROJECTION (see [`SecurityDescriptor::malicious_security`]) so
// `ShamirBackend::info()` and every existing `BackendInfo` caller still compile
// unchanged. Guarantees are reported PER-OPERATOR ([`OperatorClass`] /
// [`MpcBackend::operator_security`]) because they genuinely differ — the
// degree-`t` aggregate is robust while the degree-`2t` equality open is
// semi-honest-only at `n = 2t+1`, and one backend-level bit would lie.
//
// Scope: this is a CONTROLLED enum refactor behind the unchanged `MpcBackend`
// trait, NOT a rewrite. The fail-closed selection registry that consumes these
// axes is a SEPARATE concern — tracked as bead sq-a6p1 (depends on this).
// =============================================================================

/// AXIS-1 — the **adversary model**: how a corrupt party is allowed to behave
/// (research §1.2). Orthogonal to the corruption *threshold* ([`CorruptionThreshold`],
/// AXIS-3) and to the *output guarantee* ([`OutputGuarantee`], AXIS-2). Ordered
/// weakest → strongest. `[OPUS-4.8]`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdversaryModel {
    /// **Passive / honest-but-curious.** A corrupt party follows the protocol
    /// faithfully but tries to infer secrets from its view. The v1 regime among
    /// cooperating holders (Shamir semi-honest).
    SemiHonest,
    /// **Covert (ε-deterrence).** A corrupt party MAY deviate arbitrarily but is
    /// *caught with probability ε* (Aumann–Lindell'07); a PVC variant adds a
    /// publicly-verifiable cheating certificate (see [`PublicVerifiability`]).
    /// The genuine middle tier between semi-honest and malicious — absent from
    /// the old enums. ε is carried as an exact rational `deterrence_num /
    /// deterrence_den` (e.g. `1/2`), avoiding a float in a security descriptor.
    ///
    /// **Invariant (now enforced, not just documented).** ε must be a valid
    /// probability in `(0, 1]`: `deterrence_den > 0` and `deterrence_num <=
    /// deterrence_den`. The fields are PRIVATE and the variant carries a private
    /// witness token, so it is unbuildable except through [`AdversaryModel::covert`],
    /// which range-checks and returns `None` on a nonsensical ε. Read the ε back
    /// via [`AdversaryModel::deterrence`]. This mirrors the Cleve type-invariant on
    /// [`OutputGuarantee`]. `[OPUS-4.8]` (Copilot review #87).
    Covert {
        /// Numerator of the deterrence probability ε. Private — set only by the
        /// range-checking [`AdversaryModel::covert`] constructor.
        deterrence_num: u32,
        /// Denominator of the deterrence probability ε. Private; the constructor
        /// guarantees `> 0` and `>= deterrence_num`.
        deterrence_den: u32,
        /// Zero-sized witness that [`AdversaryModel::covert`] validated the ε
        /// invariant — its private type bars any struct-literal escape hatch.
        _checked: private::CovertEpsilonProof,
    },
    /// **Active / malicious.** A corrupt party may deviate from the protocol
    /// arbitrarily. The output guarantee under this adversary is determined by
    /// AXIS-2 ([`OutputGuarantee`]) and constrained by AXIS-3 (Cleve).
    Malicious,
}

impl AdversaryModel {
    /// Construct [`AdversaryModel::Covert`] with deterrence probability
    /// ε = `num / den`, range-checking the invariant: `den > 0` and `num <= den`,
    /// so ε is a valid probability in `(0, 1]`. Returns `None` for a nonsensical ε
    /// (zero denominator, or ε > 1) instead of silently building a descriptor that
    /// misrepresents the deterrence security parameter. This is the ONLY way to
    /// obtain a `Covert` value (its fields and witness token are private).
    /// `num == 0` (ε = 0, i.e. no deterrence at all) is rejected too: that is the
    /// semi-honest regime — use [`AdversaryModel::SemiHonest`]. `[OPUS-4.8]`
    pub fn covert(num: u32, den: u32) -> Option<Self> {
        if den == 0 || num == 0 || num > den {
            return None;
        }
        Some(AdversaryModel::Covert {
            deterrence_num: num,
            deterrence_den: den,
            _checked: private::CovertEpsilonProof(()),
        })
    }

    /// The deterrence probability ε = `(num, den)` if this is a covert adversary,
    /// else `None`. The returned pair always satisfies the constructor invariant
    /// (`den > 0`, `0 < num <= den`). `[OPUS-4.8]`
    pub fn deterrence(self) -> Option<(u32, u32)> {
        match self {
            AdversaryModel::Covert {
                deterrence_num,
                deterrence_den,
                ..
            } => Some((deterrence_num, deterrence_den)),
            _ => None,
        }
    }
}

/// The flavour of an `abort` output guarantee (research §1.1 rows 3/4): what the
/// honest parties learn on a detected cheat. Ordered weakest → strongest.
/// `[OPUS-4.8]`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbortKind {
    /// **Selective abort.** A cheat is detected and the protocol aborts, but the
    /// adversary may cause only *some* honest parties to abort.
    Selective,
    /// **Unanimous abort.** On a detected cheat, *all* honest parties agree to
    /// abort (no split-brain), but no cheater is attributed. The SPDZ-style
    /// detect-and-abort guarantee.
    Unanimous,
    /// **Identifiable abort (IA).** On abort, the honest parties additionally
    /// AGREE on (at least one) cheater — sound cheater attribution. NB: the
    /// current Shamir backend's `Tampered{cheaters}` set is *heuristic* on the
    /// abort path, NOT a sound IA, so the backend must not advertise this for
    /// the equality open (see [`OperatorClass`] reporting).
    Identifiable,
}

/// AXIS-2 — the **output guarantee** the protocol delivers against the chosen
/// adversary (research §1.2): does it abort on a cheat, or always deliver the
/// correct output? Orthogonal to AXIS-1 ([`AdversaryModel`]) and AXIS-3
/// ([`CorruptionThreshold`]).
///
/// ## Cleve's impossibility as a type-level invariant
/// Cleve (STOC'86): **fairness and guaranteed output are impossible without an
/// honest majority.** This type encodes that as a constructor invariant — the
/// [`OutputGuarantee::Fairness`] and [`OutputGuarantee::GuaranteedOutput`]
/// variants have a PRIVATE field, so they cannot be built directly; the ONLY way
/// to obtain them is via [`OutputGuarantee::fairness`] /
/// [`OutputGuarantee::guaranteed_output`], which take a [`CorruptionThreshold`]
/// and return `None` for [`CorruptionThreshold::DishonestMajority`]. A
/// dishonest-majority backend is therefore *unable to construct* an output
/// guarantee Cleve forbids — the impossibility is a compile-/construct-time
/// property, not a runtime check that can be forgotten. `[OPUS-4.8]`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputGuarantee {
    /// **Abort (no guaranteed output).** On a detected cheat the protocol aborts
    /// (per [`AbortKind`]) rather than returning a wrong answer; a single cheater
    /// can still force the abort, so a correct output is NOT guaranteed. The ONLY
    /// guarantee available under a dishonest majority (Cleve).
    Abort(AbortKind),
    /// **Fairness.** Either all parties learn the output or none do — but, unlike
    /// [`OutputGuarantee::GuaranteedOutput`], a cheater may still deny everyone
    /// the output. Sits strictly between abort and GOD. Honest-majority-only
    /// (Cleve); construct via [`OutputGuarantee::fairness`]. The private unit
    /// field makes the variant unbuildable except through that constructor.
    Fairness(private::HonestMajorityProof),
    /// **Guaranteed output delivery (GOD).** The honest parties ALWAYS obtain the
    /// correct output, even when up to the threshold of parties actively deviate.
    /// Honest-majority-only (Cleve); construct via
    /// [`OutputGuarantee::guaranteed_output`]. The private unit field makes the
    /// variant unbuildable except through that constructor.
    GuaranteedOutput(private::HonestMajorityProof),
}

/// Private witness module: the unit token that gates construction of the
/// Cleve-restricted [`OutputGuarantee`] variants. Because the type is private to
/// this module, code OUTSIDE `backend` cannot mint a `Fairness`/`GuaranteedOutput`
/// directly — it must go through the honest-majority-checking constructors. This
/// is what makes Cleve a *type-level* invariant rather than a runtime assert.
/// `[OPUS-4.8]`
mod private {
    /// Zero-sized proof-of-honest-majority. Only the honest-majority constructors
    /// in [`super::OutputGuarantee`] can mint one, so its presence in a
    /// `Fairness`/`GuaranteedOutput` value is evidence the Cleve precondition was
    /// checked. `[OPUS-4.8]`
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct HonestMajorityProof(pub(super) ());

    /// Zero-sized proof that the covert ε invariant (`den > 0`, `num <= den`) was
    /// range-checked. Only [`super::AdversaryModel::covert`] can mint one, so its
    /// presence in a `Covert` value is evidence ε is a valid probability in
    /// `(0, 1]`. `[OPUS-4.8]`
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CovertEpsilonProof(pub(super) ());
}

impl OutputGuarantee {
    /// Construct [`OutputGuarantee::Fairness`] IFF the threshold gives an honest
    /// majority (`HonestMajority` or `SuperHonestMajority`). Returns `None` under
    /// [`CorruptionThreshold::DishonestMajority`] — Cleve forbids fairness there.
    /// This is the ONLY way to obtain the `Fairness` variant. `[OPUS-4.8]`
    pub fn fairness(threshold: CorruptionThreshold) -> Option<Self> {
        threshold
            .is_honest_majority()
            .then_some(OutputGuarantee::Fairness(private::HonestMajorityProof(())))
    }

    /// Construct [`OutputGuarantee::GuaranteedOutput`] (GOD) IFF the threshold
    /// gives an honest majority. Returns `None` under
    /// [`CorruptionThreshold::DishonestMajority`] — Cleve forbids GOD there. This
    /// is the ONLY way to obtain the `GuaranteedOutput` variant. `[OPUS-4.8]`
    pub fn guaranteed_output(threshold: CorruptionThreshold) -> Option<Self> {
        threshold
            .is_honest_majority()
            .then_some(OutputGuarantee::GuaranteedOutput(
                private::HonestMajorityProof(()),
            ))
    }

    /// `true` iff this guarantee is some kind of abort (no guaranteed output).
    pub fn is_abort(self) -> bool {
        matches!(self, OutputGuarantee::Abort(_))
    }

    /// `true` iff this guarantee is fairness or stronger (guaranteed output) —
    /// i.e. it is constructible only under an honest majority. `[OPUS-4.8]`
    pub fn requires_honest_majority(self) -> bool {
        matches!(
            self,
            OutputGuarantee::Fairness(_) | OutputGuarantee::GuaranteedOutput(_)
        )
    }
}

/// AXIS-3 — the **corruption threshold**: how many of `n` parties may be corrupt,
/// carried as the concrete `t` so the same descriptor is expressible in BOTH
/// majority regimes (research §1.2). This is the axis the old binary
/// [`TrustModel`] collapsed; the three variants restore the `n/3`-vs-`n/2` split
/// that separates *perfect* GOD (BGW, point-to-point) from *statistical* GOD
/// (requires broadcast). `[OPUS-4.8]`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorruptionThreshold {
    /// **Dishonest majority** (`t < n`): up to `n − 1` parties may be corrupt.
    /// The realistic cross-org model; only `Abort` output guarantees are
    /// reachable (Cleve). Carries the tolerated corruption count `t`.
    DishonestMajority {
        /// Number of corrupt parties tolerated (`t < n`).
        t: usize,
    },
    /// **Honest majority** (`n > 2t`): a strict majority of parties is honest.
    /// Admits statistical GOD (with a broadcast channel). Carries `t`.
    HonestMajority {
        /// Number of corrupt parties tolerated (`n > 2t`).
        t: usize,
    },
    /// **Super-honest majority** (`n > 3t`): more than two-thirds honest. Admits
    /// PERFECT, error-free GOD over point-to-point channels (BGW). Carries `t`.
    SuperHonestMajority {
        /// Number of corrupt parties tolerated (`n > 3t`).
        t: usize,
    },
}

impl CorruptionThreshold {
    /// The tolerated corruption count `t`, regardless of regime.
    pub fn threshold(self) -> usize {
        match self {
            CorruptionThreshold::DishonestMajority { t }
            | CorruptionThreshold::HonestMajority { t }
            | CorruptionThreshold::SuperHonestMajority { t } => t,
        }
    }

    /// `true` iff this threshold guarantees an honest majority (`HonestMajority`
    /// or `SuperHonestMajority`). The Cleve precondition for fairness/GOD.
    pub fn is_honest_majority(self) -> bool {
        !matches!(self, CorruptionThreshold::DishonestMajority { .. })
    }

    /// Derive the honest-majority threshold for a backend with `n` parties at the
    /// usual `t = ⌊(n−1)/2⌋` (the Shamir honest-majority constructor's choice),
    /// classifying it into the strongest applicable regime: `n > 3t` →
    /// super-honest, else `n > 2t` → honest (always true for `t = ⌊(n−1)/2⌋`,
    /// `n >= 2`). `[OPUS-4.8]`
    pub fn from_n_t(n: usize, t: usize) -> Self {
        if n > 3 * t {
            CorruptionThreshold::SuperHonestMajority { t }
        } else if n > 2 * t {
            CorruptionThreshold::HonestMajority { t }
        } else {
            CorruptionThreshold::DishonestMajority { t }
        }
    }
}

/// Marker for **public verifiability** (research §1.3): whether a cheat / the
/// computation is verifiable by an external party who did not run it — PVC's
/// publicly-verifiable cheating certificate, or a publicly-verifiable
/// collaborative-zk proof. Orthogonal to the three axes. `[OPUS-4.8]`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PublicVerifiability(pub bool);

/// The full **three-axis security descriptor** for a backend (or a single
/// operator within it): adversary × output-guarantee × corruption-threshold,
/// plus the [`PublicVerifiability`] marker. This is the configurable security
/// descriptor that replaces the entangled [`TrustModel`] + [`MaliciousSecurity`]
/// pair; it composes them losslessly and projects BACK to the old enums for
/// back-compat (see [`SecurityDescriptor::malicious_security`] /
/// [`SecurityDescriptor::trust_model`]). `[OPUS-4.8]`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecurityDescriptor {
    /// AXIS-1: how a corrupt party may behave.
    pub adversary: AdversaryModel,
    /// AXIS-2: what output guarantee holds against that adversary. Construction
    /// of the stronger variants is gated on `threshold` being honest-majority
    /// (Cleve), so any value here is internally consistent with `threshold`.
    pub output_guarantee: OutputGuarantee,
    /// AXIS-3: the corruption threshold (carrying `t`).
    pub threshold: CorruptionThreshold,
    /// Whether a cheat / the computation is publicly verifiable.
    pub public_verifiability: PublicVerifiability,
}

impl SecurityDescriptor {
    /// The v1 semi-honest, detect-and-abort-on-redundancy honest-majority
    /// descriptor for a `(n, t)` Shamir backend whose degree-`t` reconstruction
    /// carries RS redundancy. `robust_cheaters` is the RS correction budget
    /// `e = ⌊(n−t−1)/2⌋` (`0` ⇒ detect-and-abort only). Helper used by
    /// [`crate::shamir::ShamirBackend`] to build its per-operator descriptors
    /// without duplicating the Cleve plumbing. `[OPUS-4.8]`
    pub fn shamir_degree_recon(n: usize, t: usize, robust_cheaters: usize) -> Self {
        let threshold = CorruptionThreshold::from_n_t(n, t);
        // FAIL-CLOSED under a dishonest majority (`n <= 2t`). The RS-redundancy
        // detect-and-abort / Berlekamp–Welch correction this constructor encodes
        // is an *honest-majority-specific* claim: the bound `e = ⌊(n−t−1)/2⌋` and
        // the "any tampering is detected" guarantee both assume a strict honest
        // majority on the reconstruction. With `n <= 2t` there is not enough
        // honest redundancy to back that claim, so emitting an `Abort(Unanimous)`
        // here would over-claim — and would project (via `malicious_security`) to
        // `HonestMajorityAbort` while `trust_model()` reports `DishonestMajority`,
        // an internally contradictory `BackendInfo`. The legacy `MaliciousSecurity`
        // enum cannot represent dishonest-majority active security without
        // over-claiming, so this honest-majority constructor degrades to the
        // semi-honest-only baseline there. A genuine dishonest-majority active
        // backend (SPDZ/MASCOT: MAC-checked abort) is a *different* construction
        // and would build its descriptor directly, not via this helper. `[OPUS-4.8]`
        if !threshold.is_honest_majority() {
            return SecurityDescriptor::semi_honest_only(n, t);
        }
        // The honest-majority Shamir backend is semi-honest among cooperating
        // holders; the RS-checked reconstruction adds a detect/correct step. We
        // surface it as semi-honest adversary with the *output guarantee* the RS
        // redundancy delivers (the active-security hardening is the guarantee
        // axis, not a claim that the parties are malicious).
        let output_guarantee = if robust_cheaters >= 1 {
            // Robust correction up to a budget ⇒ guaranteed output (GOD) under
            // honest majority. Cleve-gated constructor; honest majority holds for
            // every valid Shamir `(n, t = ⌊(n−1)/2⌋)`, so this is `Some`.
            OutputGuarantee::guaranteed_output(threshold)
                .unwrap_or(OutputGuarantee::Abort(AbortKind::Unanimous))
        } else {
            // Redundancy lets us detect-and-abort (unanimous), but not correct.
            OutputGuarantee::Abort(AbortKind::Unanimous)
        };
        SecurityDescriptor {
            adversary: AdversaryModel::SemiHonest,
            output_guarantee,
            threshold,
            public_verifiability: PublicVerifiability(false),
        }
    }

    /// A no-redundancy descriptor: semi-honest-only, with no detection. Used for
    /// the degree-`2t` equality open at `n = 2t+1` (zero RS redundancy at degree
    /// `2t`) and for honest stub backends. `[OPUS-4.8]`
    pub fn semi_honest_only(n: usize, t: usize) -> Self {
        SecurityDescriptor {
            adversary: AdversaryModel::SemiHonest,
            output_guarantee: OutputGuarantee::Abort(AbortKind::Selective),
            threshold: CorruptionThreshold::from_n_t(n, t),
            public_verifiability: PublicVerifiability(false),
        }
    }

    /// **Back-compat PROJECTION → [`TrustModel`].** Collapses AXIS-3 to the old
    /// binary trust model so existing callers of `BackendInfo::trust_model`
    /// keep working: honest-/super-honest-majority project to
    /// [`TrustModel::HonestMajority`], dishonest-majority to
    /// [`TrustModel::DishonestMajority`]. `[OPUS-4.8]`
    pub fn trust_model(&self) -> TrustModel {
        if self.threshold.is_honest_majority() {
            TrustModel::HonestMajority
        } else {
            TrustModel::DishonestMajority
        }
    }

    /// **Back-compat PROJECTION → [`MaliciousSecurity`].** Maps the three-axis
    /// descriptor onto the old bundled enum so `ShamirBackend::info()` and every
    /// existing `BackendInfo` caller still type-checks against the same
    /// [`MaliciousSecurity`] values:
    ///
    /// - no active-security guarantee (semi-honest adversary AND a no-detection
    ///   selective abort) → [`MaliciousSecurity::SemiHonestOnly`];
    /// - guaranteed output / robust correction up to a budget →
    ///   [`MaliciousSecurity::HonestMajorityRobust`] (the budget is recovered
    ///   from `robust_cheaters`, threaded through this projection);
    /// - any other detect-and-abort → [`MaliciousSecurity::HonestMajorityAbort`].
    ///
    /// `robust_cheaters` is the RS correction budget the *operator* delivers
    /// (`>= 1` for the robust case); it cannot be recovered from the abstract
    /// `OutputGuarantee` alone (which only says "GOD", not "GOD up to e"), so it
    /// is supplied by the backend that built the descriptor. This keeps the
    /// projection faithful to the old enum's `max_cheaters` field. `[OPUS-4.8]`
    pub fn malicious_security(&self, robust_cheaters: usize) -> MaliciousSecurity {
        match self.output_guarantee {
            // Selective abort under a semi-honest adversary = the honest
            // "we claim nothing" baseline (no redundancy / stub).
            OutputGuarantee::Abort(AbortKind::Selective)
                if self.adversary == AdversaryModel::SemiHonest =>
            {
                MaliciousSecurity::SemiHonestOnly
            }
            OutputGuarantee::GuaranteedOutput(_) if robust_cheaters >= 1 => {
                MaliciousSecurity::HonestMajorityRobust {
                    max_cheaters: robust_cheaters,
                }
            }
            // Everything else that detects (unanimous/identifiable abort, or a
            // GOD claim with a zero correction budget — which is really
            // detect-and-abort) is the abort guarantee.
            _ => MaliciousSecurity::HonestMajorityAbort,
        }
    }
}

/// The class of SPARQL/MPC operator a [`SecurityDescriptor`] is reported FOR.
/// Guarantees differ PER-OPERATOR for the same backend `(n, t)` — the degree-`t`
/// linear aggregate carries RS redundancy (robust) while the degree-`2t`
/// equality/join open has NO redundancy at `n = 2t+1` (semi-honest-only) — so a
/// single backend-level bit would lie (research §1.2). A backend reports its
/// guarantee keyed by this class via [`MpcBackend::operator_security`].
/// `[OPUS-4.8]`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorClass {
    /// Linear / cumulative aggregate (SUM/COUNT): free local share-addition,
    /// reconstructed at degree `t`. Carries RS redundancy for every valid
    /// honest-majority `(n, t)`, so it is the robust path.
    LinearAggregate,
    /// Equality / hidden-value join: opens a degree-`2t` product. At `n = 2t+1`
    /// (the honest-majority odd-`n` case) degree `2t` has ZERO RS redundancy, so
    /// this open is semi-honest-only there even though the aggregate is robust.
    EqualityJoin,
    /// Comparison (`<`, `≤`, `>`): bit-decomposition; not yet realized in the
    /// crate (disclosed operands are recomputed by the verifier outside the
    /// crypto). Reported for completeness so a federation sees the gap.
    Comparison,
}

/// The trust regime an [`MpcBackend`] implementation provides. This is the
/// Q2 axis made explicit in the type system so a federation can refuse a
/// backend whose guarantees do not match its threat model (§4.2).
///
/// **Back-compat note (sq-mq8q).** This binary enum is now a *projection* of the
/// richer [`CorruptionThreshold`] axis (which carries `t` and distinguishes
/// honest- from super-honest-majority). It is retained so existing
/// `BackendInfo::trust_model` callers keep compiling; new code should prefer
/// `BackendInfo::security.threshold`. See [`SecurityDescriptor::trust_model`] for
/// the projection. `[OPUS-4.8]`
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
///
/// **Back-compat note (sq-mq8q).** This enum bundled AXIS-1 (semi-honest vs
/// active) with AXIS-2 (abort vs robust) and soldered "HonestMajority" into the
/// variant names. It is now a *projection* of the orthogonal three-axis
/// [`SecurityDescriptor`] (see [`SecurityDescriptor::malicious_security`]),
/// retained so `ShamirBackend::info()` and every existing caller keep compiling.
/// New code should prefer `BackendInfo::security` / [`MpcBackend::operator_security`].
/// `[OPUS-4.8]`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaliciousSecurity {
    /// **No active-security guarantee — semi-honest only.** Security holds only
    /// against semi-honest / honest-but-curious parties; an actively-deviating
    /// party can silently corrupt the output with no detection. The honest
    /// baseline for a not-yet-built / stub backend, and the guarantee for any
    /// reconstruction at exactly `degree + 1` shares (no RS redundancy — e.g. the
    /// degree-`2t` equality open at `n = 2t + 1`). NB: with the honest-majority
    /// `t = ⌊(n−1)/2⌋` the degree-`t` cumulative-aggregate path always has
    /// redundancy, so it never reports `SemiHonestOnly`. Named `SemiHonestOnly`
    /// (not `None`) so it cannot be confused with [`Option::None`] nor collide
    /// under a `use MaliciousSecurity::*` glob import. `[OPUS-4.8]`
    SemiHonestOnly,
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
    /// [`MaliciousSecurity::SemiHonestOnly`]. The backwards-compatible projection
    /// of the former `malicious_secure: bool` field, for callers that only need
    /// the coarse "is this hardened at all?" bit.
    pub fn is_malicious_secure(self) -> bool {
        self != MaliciousSecurity::SemiHonestOnly
    }
}

/// Static description of a concrete backend's guarantees — what a federation
/// inspects to decide whether a backend is acceptable BEFORE running anything.
///
/// **sq-mq8q.** The source of truth is now the three-axis [`security`] descriptor
/// (adversary × output-guarantee × corruption-threshold + public verifiability).
/// The `trust_model` and `malicious_security` fields are KEPT as a back-compat
/// projection of it (every existing caller of those two fields keeps compiling),
/// but a `BackendInfo` is best built via [`BackendInfo::new`] so the projection
/// stays internally consistent, and new code should read `security` /
/// [`MpcBackend::operator_security`]. `[OPUS-4.8]`
///
/// [`security`]: BackendInfo::security
#[derive(Debug, Clone)]
pub struct BackendInfo {
    /// Human-readable backend name (e.g. an eventual `"replicated-3pc"`).
    pub name: &'static str,
    /// The backend-level three-axis security descriptor (sq-mq8q). It describes
    /// the backend's PRIMARY reconstruction path (for the Shamir backend, the
    /// degree-`t` linear-aggregate open). It is NOT a single bit that can
    /// summarise every operator: a backend whose guarantees differ per operator
    /// (the Shamir degree-`2t` equality open is semi-honest-only at `n = 2t+1`
    /// even though the degree-`t` aggregate is robust) reports the per-operator
    /// detail via [`MpcBackend::operator_security`] — query that, do not assume
    /// this field covers every operator. `[OPUS-4.8]`
    pub security: SecurityDescriptor,
    /// The trust regime (the Q2 axis). **Back-compat projection** of
    /// `security.threshold` — see [`SecurityDescriptor::trust_model`].
    pub trust_model: TrustModel,
    /// The malicious-security guarantee against actively-deviating parties
    /// (§4.2's guarantee (D)), orthogonal to `trust_model`. **Back-compat
    /// projection** of `security` — see [`SecurityDescriptor::malicious_security`].
    /// Replaces the former coarse `malicious_secure: bool` so the *kind* of active
    /// security (detect-and-abort vs robust-up-to-`max_cheaters`) is legible to a
    /// federation. Use [`BackendInfo::malicious_secure`] for the coarse bool.
    /// `[OPUS-4.8]`
    pub malicious_security: MaliciousSecurity,
}

impl BackendInfo {
    /// Build a `BackendInfo` from the three-axis [`SecurityDescriptor`], deriving
    /// the back-compat `trust_model` / `malicious_security` projection fields so
    /// they can never drift from `security`. `robust_cheaters` is the RS
    /// correction budget the backend's reconstruction delivers (`0` ⇒
    /// detect-and-abort only); it is threaded into the
    /// [`MaliciousSecurity::HonestMajorityRobust`] projection so the old enum's
    /// `max_cheaters` stays faithful. `[OPUS-4.8]`
    pub fn new(name: &'static str, security: SecurityDescriptor, robust_cheaters: usize) -> Self {
        BackendInfo {
            name,
            trust_model: security.trust_model(),
            malicious_security: security.malicious_security(robust_cheaters),
            security,
        }
    }

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

    /// Static guarantees of this backend (the three-axis [`SecurityDescriptor`]
    /// plus its back-compat `trust_model` / `malicious_security` projection). The
    /// `security` field describes the backend's PRIMARY reconstruction path; use
    /// [`MpcBackend::operator_security`] for the per-operator detail where they
    /// differ.
    fn info(&self) -> BackendInfo;

    /// Per-operator security guarantee (sq-mq8q). Guarantees differ by
    /// [`OperatorClass`] for the same backend `(n, t)` — e.g. the Shamir
    /// degree-`t` linear aggregate is robust while the degree-`2t` equality open
    /// has no RS redundancy at `n = 2t+1` — so a single backend-level bit would
    /// lie. The default returns the backend-level `info().security`; backends
    /// whose operators genuinely differ (the Shamir backend) override this to
    /// report each class precisely. `[OPUS-4.8]`
    fn operator_security(&self, _operator: OperatorClass) -> SecurityDescriptor {
        self.info().security
    }

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

// =============================================================================
// [OPUS-4.8] sq-mq8q — three-axis security descriptor tests: the axes construct
// correctly; the Cleve impossibility is a type-level invariant (a
// DishonestMajority + GuaranteedOutput is UNREPRESENTABLE — the constructor
// refuses it); the back-compat projection maps old↔new faithfully. Per-operator
// reporting for the real Shamir backend is exercised in `adversarial_tests.rs`
// where the `(n, t)` fixtures live. Fable unavailable — flag for re-review.
// =============================================================================
#[cfg(test)]
mod axis_tests {
    use super::*;

    // --- AXIS-1: AdversaryModel constructs across all three tiers. ----------
    #[test]
    fn adversary_model_three_tiers_construct() {
        assert_eq!(AdversaryModel::SemiHonest, AdversaryModel::SemiHonest);
        // Covert carries an exact rational deterrence ε = 1/2, via the checked
        // constructor (the only way to build the variant).
        let covert = AdversaryModel::covert(1, 2).expect("ε = 1/2 is a valid probability");
        assert_ne!(covert, AdversaryModel::SemiHonest);
        assert_ne!(covert, AdversaryModel::Malicious);
        assert_ne!(AdversaryModel::Malicious, AdversaryModel::SemiHonest);
        assert_eq!(covert.deterrence(), Some((1, 2)));
        assert_eq!(AdversaryModel::SemiHonest.deterrence(), None);
        assert_eq!(AdversaryModel::Malicious.deterrence(), None);
    }

    // The covert ε invariant (`den > 0`, `0 < num <= den`) is ENFORCED by the
    // constructor, not merely documented — a nonsensical ε returns `None` rather
    // than building a descriptor that misrepresents the deterrence parameter.
    // `[OPUS-4.8]` (Copilot review #87).
    #[test]
    fn covert_constructor_enforces_epsilon_invariant() {
        // Valid: ε in (0, 1].
        assert!(AdversaryModel::covert(1, 2).is_some());
        assert!(AdversaryModel::covert(1, 1).is_some()); // ε = 1 (always caught)
        assert!(AdversaryModel::covert(3, 4).is_some());
        // Invalid: zero denominator (division by zero / undefined ε).
        assert_eq!(AdversaryModel::covert(1, 0), None);
        // Invalid: ε > 1 (num > den) — not a probability.
        assert_eq!(AdversaryModel::covert(3, 2), None);
        // Invalid: ε = 0 (num == 0) — that is the semi-honest regime, not covert.
        assert_eq!(AdversaryModel::covert(0, 2), None);
        assert_eq!(AdversaryModel::covert(0, 0), None);
    }

    // --- AXIS-3: CorruptionThreshold carries t and classifies the regime. ---
    #[test]
    fn corruption_threshold_carries_t_and_classifies_regime() {
        // n > 3t → super-honest (perfect GOD, BGW point-to-point).
        assert_eq!(
            CorruptionThreshold::from_n_t(4, 1),
            CorruptionThreshold::SuperHonestMajority { t: 1 }
        );
        // 2t < n <= 3t → honest (statistical GOD, needs broadcast).
        assert_eq!(
            CorruptionThreshold::from_n_t(5, 2),
            CorruptionThreshold::HonestMajority { t: 2 }
        );
        // n <= 2t → dishonest majority.
        assert_eq!(
            CorruptionThreshold::from_n_t(3, 2),
            CorruptionThreshold::DishonestMajority { t: 2 }
        );
        // t is carried (and recoverable) in every regime.
        assert_eq!(
            CorruptionThreshold::DishonestMajority { t: 7 }.threshold(),
            7
        );
        assert_eq!(CorruptionThreshold::HonestMajority { t: 3 }.threshold(), 3);
        assert_eq!(
            CorruptionThreshold::SuperHonestMajority { t: 2 }.threshold(),
            2
        );
        // The honest-majority classifier.
        assert!(CorruptionThreshold::HonestMajority { t: 1 }.is_honest_majority());
        assert!(CorruptionThreshold::SuperHonestMajority { t: 1 }.is_honest_majority());
        assert!(!CorruptionThreshold::DishonestMajority { t: 1 }.is_honest_majority());
    }

    // --- AXIS-2 + CLEVE: the impossibility is a type-level invariant. -------
    // Fairness/GuaranteedOutput are ONLY constructible under an honest majority;
    // a DishonestMajority + GuaranteedOutput is unrepresentable (the constructor
    // returns None — and the variant's private field makes it the only path).
    #[test]
    fn cleve_invariant_dishonest_majority_cannot_construct_fairness_or_god() {
        let dishonest = CorruptionThreshold::DishonestMajority { t: 2 };
        assert_eq!(
            OutputGuarantee::fairness(dishonest),
            None,
            "Cleve: fairness is impossible without an honest majority"
        );
        assert_eq!(
            OutputGuarantee::guaranteed_output(dishonest),
            None,
            "Cleve: guaranteed output (GOD) is impossible without an honest majority"
        );
        // Abort, by contrast, IS available under a dishonest majority.
        let abort = OutputGuarantee::Abort(AbortKind::Unanimous);
        assert!(abort.is_abort());
        assert!(!abort.requires_honest_majority());
    }

    #[test]
    fn cleve_invariant_honest_majority_can_construct_fairness_and_god() {
        for threshold in [
            CorruptionThreshold::HonestMajority { t: 1 },
            CorruptionThreshold::SuperHonestMajority { t: 1 },
        ] {
            let fair =
                OutputGuarantee::fairness(threshold).expect("honest majority admits fairness");
            let god =
                OutputGuarantee::guaranteed_output(threshold).expect("honest majority admits GOD");
            assert!(fair.requires_honest_majority());
            assert!(god.requires_honest_majority());
            assert!(!fair.is_abort());
            assert!(!god.is_abort());
            assert_ne!(fair, god);
        }
    }

    // The private witness field means the ONLY way to obtain a Fairness/GOD value
    // is the Cleve-checking constructor — there is no struct-literal escape
    // hatch from outside `backend`. (This test lives inside the module so it can
    // even *name* the private token; external code cannot, which is the point.)
    #[test]
    fn cleve_god_value_implies_an_honest_majority_was_proven() {
        let god = OutputGuarantee::guaranteed_output(CorruptionThreshold::HonestMajority { t: 1 })
            .unwrap();
        // The value exists, so a HonestMajorityProof token was minted — which only
        // the honest-majority constructor can do. Round-trip the discriminant.
        assert!(matches!(god, OutputGuarantee::GuaranteedOutput(_)));
    }

    // --- Back-compat PROJECTION: new descriptor → old enums, faithfully. ----
    #[test]
    fn projection_trust_model_collapses_axis3_to_binary() {
        let mk = |threshold| SecurityDescriptor {
            adversary: AdversaryModel::SemiHonest,
            output_guarantee: OutputGuarantee::Abort(AbortKind::Unanimous),
            threshold,
            public_verifiability: PublicVerifiability(false),
        };
        assert_eq!(
            mk(CorruptionThreshold::SuperHonestMajority { t: 1 }).trust_model(),
            TrustModel::HonestMajority
        );
        assert_eq!(
            mk(CorruptionThreshold::HonestMajority { t: 1 }).trust_model(),
            TrustModel::HonestMajority
        );
        assert_eq!(
            mk(CorruptionThreshold::DishonestMajority { t: 1 }).trust_model(),
            TrustModel::DishonestMajority
        );
    }

    #[test]
    fn projection_malicious_security_maps_each_case() {
        // Semi-honest + no-detection selective abort → SemiHonestOnly.
        let semi = SecurityDescriptor::semi_honest_only(3, 1);
        assert_eq!(
            semi.malicious_security(0),
            MaliciousSecurity::SemiHonestOnly
        );
        // Detect-and-abort (redundancy, e == 0) → HonestMajorityAbort.
        let abort = SecurityDescriptor::shamir_degree_recon(3, 1, 0);
        assert_eq!(
            abort.malicious_security(0),
            MaliciousSecurity::HonestMajorityAbort
        );
        // Robust (GOD, e >= 1) → HonestMajorityRobust{ max_cheaters: e }, with the
        // budget threaded through the projection (it cannot be recovered from the
        // abstract OutputGuarantee alone).
        let robust = SecurityDescriptor::shamir_degree_recon(4, 1, 1);
        assert_eq!(
            robust.malicious_security(1),
            MaliciousSecurity::HonestMajorityRobust { max_cheaters: 1 }
        );
        let robust2 = SecurityDescriptor::shamir_degree_recon(9, 4, 2);
        assert_eq!(
            robust2.malicious_security(2),
            MaliciousSecurity::HonestMajorityRobust { max_cheaters: 2 }
        );
    }

    // BackendInfo::new keeps the projection fields internally consistent with the
    // three-axis descriptor (they can never drift).
    #[test]
    fn backend_info_new_derives_consistent_projection() {
        let info = BackendInfo::new("x", SecurityDescriptor::shamir_degree_recon(4, 1, 1), 1);
        assert_eq!(info.trust_model, TrustModel::HonestMajority);
        assert_eq!(
            info.malicious_security,
            MaliciousSecurity::HonestMajorityRobust { max_cheaters: 1 }
        );
        assert!(info.malicious_secure());
        // And the new descriptor is preserved verbatim.
        assert_eq!(info.security.adversary, AdversaryModel::SemiHonest);
        assert!(info.security.output_guarantee.requires_honest_majority());
    }

    // FAIL-CLOSED: `shamir_degree_recon` with a dishonest-majority `(n, t)`
    // (`n <= 2t`) must NOT emit an honest-majority active-security claim. The
    // RS-redundancy detect-and-abort it would otherwise encode is honest-majority
    // specific; under a dishonest majority it degrades to the semi-honest-only
    // baseline, so the projection stays internally consistent (no
    // `trust_model = DishonestMajority` + `malicious_security = HonestMajority…`).
    // `[OPUS-4.8]` sq-mq8q (Copilot review #87).
    #[test]
    fn shamir_degree_recon_fails_closed_under_dishonest_majority() {
        // n = 2, t = 1 → DishonestMajority (n <= 2t). Even with robust_cheaters = 0
        // (the detect-and-abort case) this must not claim honest-majority security.
        let desc = SecurityDescriptor::shamir_degree_recon(2, 1, 0);
        assert_eq!(desc.threshold, CorruptionThreshold::DishonestMajority { t: 1 });
        assert_eq!(desc.trust_model(), TrustModel::DishonestMajority);
        assert_eq!(desc.adversary, AdversaryModel::SemiHonest);
        // Degrades to the no-detection selective-abort baseline...
        assert_eq!(
            desc.output_guarantee,
            OutputGuarantee::Abort(AbortKind::Selective)
        );
        // ...so the back-compat projection is SemiHonestOnly, NOT HonestMajorityAbort.
        // The two projections are now mutually consistent.
        assert_eq!(desc.malicious_security(0), MaliciousSecurity::SemiHonestOnly);
        assert!(!desc.malicious_security(0).is_malicious_secure());

        // Even a non-zero correction budget cannot resurrect an honest-majority
        // claim under a dishonest majority (Cleve already blocks GOD; the abort
        // path is blocked here).
        let desc_budget = SecurityDescriptor::shamir_degree_recon(3, 2, 1);
        assert_eq!(
            desc_budget.threshold,
            CorruptionThreshold::DishonestMajority { t: 2 }
        );
        assert_eq!(
            desc_budget.malicious_security(1),
            MaliciousSecurity::SemiHonestOnly
        );
    }
}
