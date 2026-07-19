// [OPUS-4.8] sq-shk5: differential-privacy output-cardinality mode + per-relying-
// party epsilon-budget tracking (ShrinkWrap/SAQE/Doquet-style) for SET-returning
// hidden joins. Builds on the sq-jnkm oblivious output transform
// (`crate::oblivious_join`) + the sq-18lk shuffle substrate. Native-only; in-process
// multi-party simulation; the noise is drawn CENTRALLY in the simulation and the
// communication cost is MODELLED, not measured. Authored under the Opus 4.8 fallback
// model — re-review when Fable returns.
//! Differential-privacy output-cardinality mode + ε-budget accounting (bead
//! sq-shk5).
//!
//! ## The problem this trades against
//!
//! The exact oblivious output path ([`crate::oblivious_join::oblivious_set_output`])
//! reveals a **public, data-INDEPENDENT** padded bound `B`. To hide the true result
//! cardinality of a hidden join it must pad to the WORST case `B = |L|·|R|` (every
//! candidate pair) — an `O(|L|·|R|)` reveal that is catastrophic for a set-returning
//! query (research record §5; `research/mpc-sparql-capability-matrix.md` §5/§7.4).
//!
//! ShrinkWrap (Bater et al., VLDB'19), SAQE and Doquet observe that padding to a
//! **differentially-private noised cardinality** — `B = true_count + nonneg_noise`
//! — is order-of-magnitude cheaper: you reveal only `≈ true_count + O(1/ε)` slots
//! instead of the worst case. The price is that `B` is now **data-DEPENDENT**, so
//! its value leaks a noised view of the true result cardinality that this module
//! **calibrates to an (ε, δ)-DP target**. This module implements that trade + the
//! ε-budget that composes over repeated queries — but see the HONESTY section
//! below: the implementation does NOT claim an achieved, certified (ε, δ)-DP
//! guarantee.
//!
//! ## HONESTY — what the DP guarantee is, and (loudly) what it is NOT
//!
//! The privacy-claims gate is LIVE. This is a **research-grade, externally-unaudited
//! (`sq-qhy4`)**, honest-majority / **semi-honest-only** mechanism. Specifically:
//!
//! - **This is NOT information-theoretic.** The exact padded-bound mode
//!   ([`crate::oblivious_join`]) hides the cardinality information-theoretically (the
//!   bound is data-independent). DP mode DELIBERATELY WEAKENS that to a computational/
//!   statistical **(ε, δ)** guarantee in exchange for cost. The released bound `B`
//!   leaks that the query ran AND a noised size; over repeated queries the leak
//!   **composes** and MUST be tracked against a budget (this module's [`PrivacyBudget`]).
//! - **What DP is meant to protect here:** the *value of the released slot count
//!   `B`* is calibrated so that, in exact arithmetic, it is an (ε, δ)-DP function
//!   of the true match count `m` under the neighbouring-input model the CALLER
//!   fixes via `sensitivity` (see the Sensitivity section — for a join,
//!   sensitivity is generally NOT 1). Two neighbouring inputs — inputs whose true
//!   counts differ by at most `sensitivity` — then produce `B` distributions
//!   within `e^ε` of each other up to the accounted `δ` failure events (the
//!   neighbouring-support gap — which subsumes the clamp — and the tail
//!   truncation, δ/2 each). It does NOT make dummies
//!   indistinguishable from real rows to an authorized recipient who filters tagged
//!   dummies — that recipient still learns `m` exactly, inherent to a
//!   set-returning result and UNCHANGED from the base transform (see
//!   [`crate::oblivious_join`] residual-leakage note). DP bounds the *cardinality*
//!   leak to the parties / transcript, not the recipient's own filtering.
//! - **The noise is drawn CENTRALLY in this in-process simulation** (the dealer's
//!   masking CSPRNG plays the trusted-dealer role). A real deployment needs
//!   **distributed noise generation** — each party contributes a share of the noise
//!   so no single party learns `ν` and can de-noise `B` back to the true `m`. That
//!   distributed discrete-noise sampler is **NOT built here** and is honestly named
//!   as the residual gate (it composes with the same dealer-less-randomness seam,
//!   [`crate::randomness`], the rest of the crate is gated behind). The float
//!   inverse-CDF geometric sampler below is a **simulation-grade** sampler, NOT a
//!   hardened DP sampler — floating-point DP samplers have known precision
//!   side-channels (Mironov, CCS'12, "On Significance of the Least Significant
//!   Bits"); a deployment needs an exact discrete sampler (Canonne–Kamath–Steinke).
//!   **For that reason this module does NOT claim an achieved (ε, δ)-DP
//!   guarantee.** The mechanism is *calibrated to the (ε, δ) target* with the two
//!   discrete failure events explicitly accounted (clamp ≤ δ/2, `K_CAP` tail
//!   truncation ≤ δ/2, both enforced fail-closed at [`DpParams::new`]), but the
//!   floating-point sampling error itself is UNquantified until the exact discrete
//!   sampler lands. Treat the release as a noised bound with a DP *design target*,
//!   not a certified DP release. `[FABLE-5]`
//!
//! ## Sensitivity is the caller's obligation (a join is generally NOT `Δ = 1`)
//!
//! `sensitivity` (`Δ`) must upper-bound how much the true result count `m` can
//! change between two neighbouring inputs under the caller's neighbouring-input
//! definition (add/remove one input row, unless the caller defines otherwise).
//! For a plain count over ONE relation that is 1. **For a join it is NOT**: one
//! added input row produces as many output rows as it has join partners on the
//! other side — even for SET output — so the row-level sensitivity of an
//! unrestricted join is the maximum join degree (fan-out), which is data-dependent
//! and unbounded in general. A caller MUST either (a) enforce a contribution/
//! degree bound on the inputs and pass that bound as `Δ`, or (b) pass a public
//! conservative bound for the query shape (e.g. `|R|` for `L ⋈ R` under
//! one-added-row-in-`L` neighbouring). Passing `Δ = 1` for an unrestricted join
//! under-noises the release and voids the ε target (witness test:
//! `join_fanout_sensitivity_witness`). `[FABLE-5]`
//!
//! ## The mechanism (calibration, stated in exact arithmetic)
//!
//! For sensitivity `Δ` and privacy loss `ε`, let `α = exp(−ε/Δ)`. The **two-sided
//! geometric mechanism** (discrete Laplace; Ghosh–Roughgarden–Sundararajan) releases
//! `m + G` with `Pr[G = k] = (1−α)/(1+α) · α^{|k|}`, which is **ε-DP for an
//! integer count in exact arithmetic**. To make the released count **non-negative**
//! (and so never truncate a real match), we add a deterministic **shift** `Γ` and
//! clamp at the true count: `B = m + max(0, Γ + G)`. Because `B` never falls
//! below the true count, neighbouring counts `m` and `m + Δ` have MISMATCHED
//! supports: outputs in `[m, m + Δ − 1]` are possible under the lower count but
//! impossible under the higher — a perfectly distinguishing event whose whole
//! mass is charged to `δ`. `Γ` is therefore the smallest integer with
//! `Pr[G ≤ Δ − 1 − Γ] ≤ δ/2` (see [`DpParams::shift`]; for `Δ = 1` this is the
//! plain clamp event `Pr[G ≤ −Γ]`, and the clamp/reverse-direction event is a
//! subset of this budget for every `Δ`); the probability that either geometric
//! draw of a release is altered by the `K_CAP` safety clamp is separately
//! enforced ≤ `δ/2` at construction. Together the two accounted failure events
//! fit the stated `δ` (the ShrinkWrap-style analysis).
//! What remains UNaccounted is the floating-point sampler error itself — see the
//! HONESTY section. Because `max(0, Γ + G) ≥ 0`, we always have `B ≥ m`: **every
//! real match is revealed; the crate's never-truncate rule is preserved.**
//!
//! ## Cost
//!
//! The reveal is `B = m + O(1/ε)` slots (via [`crate::oblivious_join::oblivious_set_output`]'s
//! [`crate::oblivious_join::ObliviousOutputCost`]) instead of the worst-case
//! `|L|·|R|`. That is the ShrinkWrap saving. No hard-coded performance numbers: `B`
//! is derived from the true count + the drawn noise.

use crate::field::P;
use crate::oblivious_join::{oblivious_set_output, Candidate, MatchBit, ObliviousOutput};
use crate::partial::{HolderId, MpcError};
use crate::shamir::{ShamirBackend, ShamirDealer};
use oxrdf::Term;
use std::collections::HashMap;

/// One materialized result row — a column vector of nullable `oxrdf` terms, the same
/// row shape [`crate::partial::PartialResult`] / the oblivious output path use.
type ResultRow = Vec<Option<Term>>;

/// Upper bound on the deterministic DP shift `Γ` (and so, roughly, on the padding a
/// query pays). A `(ε, δ)` pair that would need a larger shift is REJECTED
/// fail-closed rather than silently padding to an impractical size (a too-small `ε`
/// or too-small `δ` makes the shift blow up). `65536` bounds the worst legitimate
/// padding; anything past it means the requested privacy is too strict for a
/// practical output-cardinality release. `[OPUS-4.8]`
pub const MAX_DP_SHIFT: u64 = 1 << 16;

/// Safety clamp on a single geometric draw `K`, so a pathological CSPRNG draw
/// cannot request an unbounded allocation. Truncating the geometric's tail
/// (`Pr[K > k] = α^{k+1}`) is a REAL distortion of the mechanism, and across the
/// accepted `(ε, δ)` space it is NOT automatically negligible ([`MAX_DP_SHIFT`]
/// alone does not floor `ε/Δ` when `δ` is large). The per-release truncation
/// probability — two draws, union bound, `2·α^{K_CAP+1}` — is therefore enforced
/// `≤ δ/2` at [`DpParams::new`]: parameters whose tail mass does not fit that half
/// of the failure budget are REJECTED fail-closed, so the accounted `δ` explicitly
/// covers the clamp event. `[FABLE-5]`
const K_CAP: u64 = 1 << 20;

/// Differential-privacy parameters for one output-cardinality release: the privacy
/// loss `ε`, the failure probability `δ`, and the query **sensitivity** `Δ` — an
/// upper bound, supplied AND enforced by the caller, on how much the true result
/// count can change between neighbouring inputs. `1` is correct for a count over
/// one relation under add/remove-one-row neighbouring; **for a join it is generally
/// NOT `1`** — one added input row can match many rows on the other side, so `Δ`
/// must come from an enforced contribution/degree bound or a public worst-case
/// bound for the query shape (see the module's Sensitivity section).
///
/// Constructed via [`DpParams::new`], which validates the ranges, that the sampler
/// base `α` is strictly representable inside `(0, 1)`, that the implied
/// non-negative shift is practical ([`MAX_DP_SHIFT`]), and that the `K_CAP`
/// tail-truncation mass fits its `δ/2` budget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DpParams {
    epsilon: f64,
    delta: f64,
    sensitivity: u64,
}

impl DpParams {
    /// Build validated DP parameters. Fails closed ([`MpcError::Protocol`]) if:
    /// `ε` is not finite/positive; `δ` is not in `(0, 1)` (a non-negative,
    /// never-truncating release needs `δ > 0` — the clamp event; pure `ε`-DP cannot
    /// bound the count below without it); `sensitivity` is `0`; the sampler base
    /// `α = exp(−ε/Δ)` is not STRICTLY inside `(0, 1)` in `f64` (a too-extreme
    /// ratio rounds it to `1.0` — which would silently collapse the sampler into a
    /// deterministic, privacy-free release — or underflows it to `0.0`); the
    /// implied shift `Γ` exceeds [`MAX_DP_SHIFT`] (the requested privacy is too
    /// strict to pad to practically — a too-small `ε`/`δ`); or the `K_CAP`
    /// tail-truncation mass `2·α^{K_CAP+1}` exceeds its `δ/2` budget (the capped
    /// sampler cannot honestly account the requested `(ε, δ)`).
    pub fn new(epsilon: f64, delta: f64, sensitivity: u64) -> Result<Self, MpcError> {
        if !epsilon.is_finite() || epsilon <= 0.0 {
            return Err(MpcError::Protocol(format!(
                "DpParams: epsilon must be finite and > 0 (got {epsilon})"
            )));
        }
        if !delta.is_finite() || delta <= 0.0 || delta >= 1.0 {
            return Err(MpcError::Protocol(format!(
                "DpParams: delta must be in the open interval (0, 1) — a non-negative, \
                 never-truncating DP cardinality release needs delta > 0 (got {delta})"
            )));
        }
        if sensitivity == 0 {
            return Err(MpcError::Protocol(
                "DpParams: sensitivity must be >= 1 (one input row changes the count by \
                 at least this much)"
                    .to_string(),
            ));
        }
        let p = DpParams {
            epsilon,
            delta,
            sensitivity,
        };
        // The sampler base must be STRICTLY representable inside (0, 1) in f64. A
        // tiny eps/sensitivity ratio rounds exp(-eps/sensitivity) to exactly 1.0,
        // which makes both log-denominators zero and collapses the "noise" into a
        // DETERMINISTIC release (no privacy at all); a huge ratio underflows the
        // base to 0.0 and degenerates the formulas. Both are rejected fail-closed.
        // (alpha cannot be NaN: exp of a finite negative ratio is finite or 0.)
        let alpha = p.alpha();
        if alpha <= 0.0 || alpha >= 1.0 {
            return Err(MpcError::Protocol(format!(
                "DpParams: alpha = exp(-epsilon/sensitivity) must be strictly inside \
                 (0, 1) in f64, got {alpha} (epsilon={epsilon}, sensitivity=\
                 {sensitivity}); the ratio is too extreme for the sampler — refusing \
                 fail-closed rather than releasing degenerate noise"
            )));
        }
        let shift = p.shift();
        if shift > MAX_DP_SHIFT {
            return Err(MpcError::Protocol(format!(
                "DpParams: (epsilon={epsilon}, delta={delta}, sensitivity={sensitivity}) \
                 implies a DP shift {shift} > MAX_DP_SHIFT {MAX_DP_SHIFT}; the requested \
                 privacy is too strict to pad to a practical output cardinality"
            )));
        }
        // The K_CAP clamp truncates each geometric draw's tail with probability
        // alpha^(K_CAP+1); over the two draws of one release the union bound is
        // 2*alpha^(K_CAP+1). That distortion is charged against the delta/2
        // truncation half of the failure budget (the neighbouring-support-gap
        // event the shift is calibrated to takes the other delta/2). If it does not fit, refuse
        // fail-closed rather than release with an unaccounted tail. Computed as
        // exp(-(eps/sens)*(K_CAP+1)) — mathematically identical, better precision
        // than powi for alpha near 1.
        let truncation_mass =
            2.0 * (-(epsilon / sensitivity as f64) * (K_CAP as f64 + 1.0)).exp();
        if truncation_mass > delta / 2.0 {
            return Err(MpcError::Protocol(format!(
                "DpParams: the K_CAP={K_CAP} geometric-tail truncation mass \
                 {truncation_mass} exceeds its delta/2 budget {} (epsilon={epsilon}, \
                 delta={delta}, sensitivity={sensitivity}); the requested privacy is \
                 too strict for the capped sampler — refusing fail-closed",
                delta / 2.0
            )));
        }
        Ok(p)
    }

    /// The privacy loss `ε`.
    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }

    /// The failure probability `δ`.
    pub fn delta(&self) -> f64 {
        self.delta
    }

    /// The query sensitivity.
    pub fn sensitivity(&self) -> u64 {
        self.sensitivity
    }

    /// The two-sided-geometric base `α = exp(−ε/Δ) ∈ (0, 1)`.
    fn alpha(&self) -> f64 {
        (-(self.epsilon / self.sensitivity as f64)).exp()
    }

    /// The deterministic non-negative shift `Γ`: the smallest integer `≥ Δ` with
    /// `Pr[G ≤ Δ−1−Γ] = α^{Γ−Δ+1} / (1 + α) ≤ δ/2` — the **support-gap** event,
    /// not merely the zero-noise clamp.
    ///
    /// Why the support gap is the right event (review finding): the release
    /// `B = m + max(0, Γ + G)` never falls below the true count, so for
    /// neighbouring counts `m` and `m + Δ` every output in `[m, m + Δ − 1]` is
    /// possible under the lower count but IMPOSSIBLE under the higher one — a
    /// perfectly distinguishing event whose whole mass must be charged to `δ`.
    /// The lower count lands there exactly when `max(0, Γ + G) ≤ Δ − 1`, i.e.
    /// `G ≤ Δ − 1 − Γ`. Calibrating only `Pr[G ≤ −Γ] ≤ δ/2` (the `Δ = 1`
    /// special case) under-counts that mass by `≈ e^{ε(Δ−1)/Δ}` for `Δ > 1`.
    /// The reverse DP direction's bad event — the clamp atom
    /// `Pr[G ≤ −Γ] = α^Γ/(1+α)`, where the higher count's clamped release
    /// `m + Δ` outweighs the lower count's point mass there — is a subset of
    /// this budget (`Γ ≥ Δ` so `α^Γ ≤ α^{Γ−Δ+1}`), so the same `Γ` covers both
    /// directions with `≤ δ/2` each; the other `δ/2` covers the `K_CAP` tail
    /// truncation, enforced at [`DpParams::new`]. `[FABLE-5]`
    ///
    /// Pure function of the parameters (no randomness) — exposed so a caller / test
    /// can size the expected padding a query will pay before running it.
    pub fn shift(&self) -> u64 {
        let alpha = self.alpha();
        // Solve alpha^(Gamma-Delta+1) / (1+alpha) <= delta/2
        //   =>  Gamma >= (Delta-1) + ln((delta/2)*(1+alpha)) / ln(alpha).
        // For validated params alpha is strictly inside (0, 1) (ln(alpha) finite
        // and < 0) and (delta/2)*(1+alpha) < 1 (delta < 1, alpha < 1), so the ratio
        // is finite and positive. A near-1 alpha (tiny epsilon) makes it large —
        // clamped by MAX_DP_SHIFT at construction (which thereby also bounds the
        // accepted sensitivity, so the saturating add below never actually
        // saturates for accepted params).
        let numer = (self.delta / 2.0 * (1.0 + alpha)).ln();
        let denom = alpha.ln();
        let raw = numer / denom;
        if !raw.is_finite() {
            // Unreachable for constructor-validated params (alpha is strictly
            // inside (0, 1)); kept as a defensive floor.
            return 1;
        }
        let base = (raw.ceil().max(1.0)) as u64;
        base.saturating_add(self.sensitivity - 1)
    }
}

/// The outcome of one DP output-cardinality release — the simulation's oracle view.
///
/// `revealed_bound` (`B`) is the ONLY field that is public in a real deployment;
/// `true_count` and `noise`/`shift` are the simulation's ground truth (a deployment
/// never materializes `true_count` centrally — the modelled protocol opens only the
/// noised `B`). They are returned so tests and callers can verify the mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DpCardinality {
    /// The true number of real result rows `m` (simulation oracle — never public).
    pub true_count: usize,
    /// The revealed, DP-noised slot count `B = true_count + noise` (PUBLIC) —
    /// calibrated to the (ε, δ) target, NOT a certified DP release (see the module
    /// HONESTY section: floating-point sampler, pending exact discrete sampler).
    pub revealed_bound: usize,
    /// The non-negative extra dummies `B − true_count = max(0, Γ + G)` (`≥ 0`, so a
    /// real match is never truncated).
    pub noise: u64,
    /// The deterministic shift `Γ` used (the mechanism's centre).
    pub shift: u64,
}

/// A per-relying-party **differential-privacy budget** with **basic sequential
/// composition**: each query charges its `(ε, δ)` and the totals accumulate; a
/// charge that would exceed the budget is REFUSED fail-closed BEFORE the query runs
/// (so an over-budget query reveals nothing).
///
/// HONESTY: this is **basic sequential composition** (Dwork–McSherry–Nissim–Smith) —
/// `ε` and `δ` simply add. It is NOT advanced composition, Rényi-DP, or zCDP
/// accounting, which would give a tighter budget for many queries. Using the looser
/// (larger) basic sum is the conservative, always-sound choice; a tighter accountant
/// is a future upgrade, not claimed here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrivacyBudget {
    epsilon_total: f64,
    delta_total: f64,
    epsilon_spent: f64,
    delta_spent: f64,
}

/// Absolute tolerance for the float budget comparison, so accumulated rounding does
/// not spuriously reject a charge that exactly meets the budget.
const BUDGET_EPS_TOL: f64 = 1e-12;

impl PrivacyBudget {
    /// Build a fresh budget with `epsilon_total` / `delta_total` privacy to spend.
    /// Fails closed if either is not finite or is negative (a zero budget is allowed
    /// — it simply refuses every non-trivial query).
    pub fn new(epsilon_total: f64, delta_total: f64) -> Result<Self, MpcError> {
        if !epsilon_total.is_finite() || epsilon_total < 0.0 {
            return Err(MpcError::Protocol(format!(
                "PrivacyBudget: epsilon_total must be finite and >= 0 (got {epsilon_total})"
            )));
        }
        if !delta_total.is_finite() || delta_total < 0.0 {
            return Err(MpcError::Protocol(format!(
                "PrivacyBudget: delta_total must be finite and >= 0 (got {delta_total})"
            )));
        }
        Ok(PrivacyBudget {
            epsilon_total,
            delta_total,
            epsilon_spent: 0.0,
            delta_spent: 0.0,
        })
    }

    /// Remaining `ε` (never negative).
    pub fn remaining_epsilon(&self) -> f64 {
        (self.epsilon_total - self.epsilon_spent).max(0.0)
    }

    /// Remaining `δ` (never negative).
    pub fn remaining_delta(&self) -> f64 {
        (self.delta_total - self.delta_spent).max(0.0)
    }

    /// `ε` spent so far.
    pub fn spent_epsilon(&self) -> f64 {
        self.epsilon_spent
    }

    /// `δ` spent so far.
    pub fn spent_delta(&self) -> f64 {
        self.delta_spent
    }

    /// Would charging `params` exceed this budget? (Pure check, no state change.)
    pub fn would_exceed(&self, params: &DpParams) -> bool {
        self.epsilon_spent + params.epsilon > self.epsilon_total + BUDGET_EPS_TOL
            || self.delta_spent + params.delta > self.delta_total + BUDGET_EPS_TOL
    }

    /// Charge `params` against the budget via sequential composition. Fails closed
    /// ([`MpcError::Protocol`], greppable prefix `"privacy budget exhausted"`) WITHOUT
    /// mutating the budget if the charge would exceed either total. On success the
    /// `(ε, δ)` are added to the running spend.
    pub fn charge(&mut self, params: &DpParams) -> Result<(), MpcError> {
        if self.would_exceed(params) {
            return Err(MpcError::Protocol(format!(
                "privacy budget exhausted: charging (epsilon={}, delta={}) would exceed the \
                 budget (epsilon_total={}, spent={}; delta_total={}, spent={})",
                params.epsilon,
                params.delta,
                self.epsilon_total,
                self.epsilon_spent,
                self.delta_total,
                self.delta_spent,
            )));
        }
        self.epsilon_spent += params.epsilon;
        self.delta_spent += params.delta;
        Ok(())
    }
}

/// A ledger of [`PrivacyBudget`]s keyed by **relying party** (the consumer of query
/// results, identified by [`HolderId`]). Each relying party has its OWN budget that
/// composes over the queries IT issues — so one party exhausting its budget does not
/// affect another. This is the per-relying-party accounting the residual-leak
/// honesty rule requires (a cardinality leak composes per observer).
#[derive(Debug, Clone, Default)]
pub struct BudgetLedger {
    budgets: HashMap<HolderId, PrivacyBudget>,
}

impl BudgetLedger {
    /// A ledger with no registered parties.
    pub fn new() -> Self {
        BudgetLedger {
            budgets: HashMap::new(),
        }
    }

    /// Register (or replace) the budget for a relying party.
    pub fn register(&mut self, relying_party: HolderId, budget: PrivacyBudget) {
        self.budgets.insert(relying_party, budget);
    }

    /// The current budget for a relying party, if registered.
    pub fn budget(&self, relying_party: &HolderId) -> Option<&PrivacyBudget> {
        self.budgets.get(relying_party)
    }

    /// Charge `params` to a relying party's budget (sequential composition). Fails
    /// closed if the party is not registered (an unknown consumer has NO budget to
    /// spend — refuse rather than leak) or if the charge would exceed its budget.
    pub fn charge(&mut self, relying_party: &HolderId, params: &DpParams) -> Result<(), MpcError> {
        match self.budgets.get_mut(relying_party) {
            Some(b) => b.charge(params),
            None => Err(MpcError::Protocol(format!(
                "privacy budget exhausted: relying party {relying_party} has no registered \
                 DP budget — refusing a cardinality release fail-closed"
            ))),
        }
    }
}

/// Sample the non-negative DP noise `max(0, Γ + G)` where `G` is a two-sided
/// geometric (discrete Laplace) draw with base `α = exp(−ε/Δ)`. Returns the extra
/// dummy count. Draws from the dealer's masking CSPRNG (the trusted-dealer role in
/// this simulation — see the module's distributed-noise-generation gate).
fn sample_nonneg_noise(dealer: &mut ShamirDealer, params: &DpParams) -> u64 {
    let alpha = params.alpha();
    let shift = params.shift() as i128;
    // G = K1 - K2, each K ~ Geometric on {0,1,...} with Pr[K=k] = (1-alpha) alpha^k,
    // sampled by inverse CDF: K = floor(ln(U) / ln(alpha)) for U ~ Uniform(0, 1].
    let k1 = geometric_draw(dealer, alpha) as i128;
    let k2 = geometric_draw(dealer, alpha) as i128;
    let noisy = shift + k1 - k2;
    if noisy <= 0 {
        0
    } else {
        noisy as u64
    }
}

/// One geometric draw `K = floor(ln(U) / ln(α))`, `U ~ Uniform(0, 1]`, clamped to
/// `K_CAP` (the truncation event is charged against its `δ/2` budget at
/// [`DpParams::new`] — see the `K_CAP` docs).
fn geometric_draw(dealer: &mut ShamirDealer, alpha: f64) -> u64 {
    // U in (0, 1]: (v + 1) / (P + 1), v uniform in [0, P). Never 0 (avoids ln(0)).
    let v = dealer.draw_fp().value();
    let u = (v as f64 + 1.0) / (P as f64 + 1.0);
    let k = (u.ln() / alpha.ln()).floor();
    if !k.is_finite() || k <= 0.0 {
        return 0;
    }
    (k as u64).min(K_CAP)
}

/// **DP output-cardinality release over already-materialized result rows** (the
/// sound-today entry point, bead sq-shk5).
///
/// Given the `matched_rows` a set-returning join produced (each a disclosed payload
/// row of `payload_arity` nullable terms — e.g. the output of a disclosed-key join
/// or a prior sound oblivious path), pad them with `ν = max(0, Γ + G)` **non-negative
/// DP-noise dummy slots** to a released bound `B = |matched_rows| + ν`, then run the
/// oblivious output transform ([`crate::oblivious_join::oblivious_set_output`]) so the
/// `B` revealed slots are oblivious-shuffled. The released `B` is a DP-noised view
/// of the true result cardinality, **calibrated to the (ε, δ) target** (NOT a
/// certified DP claim — see the module HONESTY section; and the target is only
/// meaningful if `params.sensitivity()` really bounds the neighbouring-input count
/// change — for a join it is generally NOT 1, see the module Sensitivity section).
///
/// ## Budget ordering: pre-authorize → validate → release → commit (fail-closed)
///
/// The `budget` is PRE-AUTHORIZED first: if the fixed, data-independent `(ε, δ)`
/// cost does not fit, the query does **no** crypto and reveals nothing — an
/// over-budget query is refused before any cardinality leaks. The charge is
/// COMMITTED only after the release has succeeded, so a request that fails
/// validation (row arity), bound construction (overflow) or the oblivious
/// transform consumes NO budget — no release, no spend (an invalid request cannot
/// be used to drain a relying party's budget).
///
/// ## Why this is sound today (and what is gated)
///
/// The rows are already MATERIALIZED (the disclosed-regime match set is public, or
/// they come from a prior sound path), so counting them leaks nothing NEW; the DP
/// noise + shuffle then produce a noised, position-oblivious release. Deriving the
/// noised cardinality from **secret** per-pair match bits WITHOUT materializing the
/// matches needs **oblivious compaction** to the noised size — honestly gated in
/// [`dp_oblivious_set_output_hidden_keys`].
///
/// HONESTY: research-grade, externally unaudited (`sq-qhy4`), honest-majority /
/// semi-honest only, noise drawn centrally in-simulation (distributed noise
/// generation is the residual gate — see the module docs). NOT information-theoretic.
pub fn dp_release_result_rows(
    backend: &ShamirBackend,
    matched_rows: &[ResultRow],
    payload_arity: usize,
    params: &DpParams,
    budget: &mut PrivacyBudget,
) -> Result<(ObliviousOutput, DpCardinality), MpcError> {
    // Pre-authorize the (data-independent) charge: fail closed BEFORE any work if
    // the budget cannot afford it. `PrivacyBudget` is `Copy`, so probing a scratch
    // copy leaves the real budget untouched until the release succeeds.
    {
        let mut probe = *budget;
        probe.charge(params)?;
    }

    // Validate the row schema before anything is spent or drawn (the recipient
    // relies on a uniform arity).
    for (k, row) in matched_rows.iter().enumerate() {
        if row.len() != payload_arity {
            return Err(MpcError::Protocol(format!(
                "dp_release_result_rows: row {k} has {} terms, expected arity {payload_arity}",
                row.len()
            )));
        }
    }

    let true_count = matched_rows.len();

    // Draw the non-negative DP noise and size the released bound. `noise >= 0` so
    // `bound >= true_count` — every real match is revealed (never truncated).
    let mut dealer = backend.dealer();
    let noise = sample_nonneg_noise(&mut dealer, params);
    let bound = true_count
        .checked_add(noise as usize)
        .ok_or_else(|| MpcError::Protocol("dp_release_result_rows: bound overflow".to_string()))?;

    // Each materialized row is a matched (Public-true) candidate; the transform pads
    // the remaining `noise` slots with pure dummies, shuffles, and reveals `bound`.
    let candidates: Vec<Candidate> = matched_rows
        .iter()
        .map(|row| Candidate {
            payload: row.clone(),
            matched: MatchBit::Public(true),
        })
        .collect();

    let output = oblivious_set_output(backend, &candidates, payload_arity, bound)?;

    // Commit the pre-authorized charge only now that the release has succeeded —
    // every failure path above returned WITHOUT spending. We hold the exclusive
    // borrow and the identical charge passed the probe, so this cannot fail.
    budget.charge(params)?;

    let card = DpCardinality {
        true_count,
        revealed_bound: bound,
        noise,
        shift: params.shift(),
    };
    Ok((output, card))
}

/// **DP output-cardinality over SECRET per-pair match bits** — the hidden-key
/// all-pairs variant that would derive the noised cardinality WITHOUT materializing
/// which pairs matched.
///
/// Honestly **GATED**: releasing `B = m + noise` slots when `m` is the secret sum of
/// never-opened match bits requires **oblivious compaction** to the noised size
/// (route the `m` matched rows into `B` output slots without revealing which
/// candidates matched). The secure equality-to-shared-bit this would consume HAS
/// landed (`sq-rrz4` / `sq-dvuc`, used by
/// [`crate::oblivious_join::oblivious_set_output_hidden_keys`]), but the oblivious
/// compaction primitive is not built here — it composes with the ORQ-style
/// sort-merge substrate (`sq-ujz8`). Returns [`MpcError::NotYetImplemented`] with the
/// gate named rather than faking it; the sound-today path is
/// [`dp_release_result_rows`] over the (disclosed / prior-sound) materialized rows.
pub fn dp_oblivious_set_output_hidden_keys(
    _backend: &ShamirBackend,
    _left_keys: &[crate::field::Fp],
    _right_keys: &[crate::field::Fp],
    _params: &DpParams,
    _budget: &mut PrivacyBudget,
) -> Result<(ObliviousOutput, DpCardinality), MpcError> {
    Err(MpcError::not_yet(
        "DP output-cardinality release over SECRET per-pair match bits (noised \
         cardinality without materializing the match set)",
        "oblivious compaction to the noised size (composes with the ORQ sort-merge \
         substrate sq-ujz8); secure equality-to-shared-bit already landed (sq-rrz4/sq-dvuc). \
         Use dp_release_result_rows over materialized/disclosed rows for the sound-today path",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oblivious_join::OutputSlot;
    use oxrdf::Literal;

    fn lit(s: &str) -> Option<Term> {
        Some(Term::Literal(Literal::new_simple_literal(s)))
    }

    fn rows(n: usize) -> Vec<ResultRow> {
        (0..n).map(|i| vec![lit(&format!("row{i}"))]).collect()
    }

    fn reals(slots: &[OutputSlot]) -> usize {
        slots
            .iter()
            .filter(|s| matches!(s, OutputSlot::Row(_)))
            .count()
    }

    fn real_multiset(slots: &[OutputSlot]) -> Vec<String> {
        let mut m: Vec<String> = slots
            .iter()
            .filter_map(|s| match s {
                OutputSlot::Row(r) => Some(format!("{r:?}")),
                OutputSlot::Dummy => None,
            })
            .collect();
        m.sort();
        m
    }

    // ---- DpParams / shift ------------------------------------------------------

    #[test]
    fn params_reject_invalid_ranges() {
        assert!(DpParams::new(0.0, 1e-6, 1).is_err(), "epsilon must be > 0");
        assert!(DpParams::new(-1.0, 1e-6, 1).is_err());
        assert!(DpParams::new(1.0, 0.0, 1).is_err(), "delta must be > 0");
        assert!(DpParams::new(1.0, 1.0, 1).is_err(), "delta must be < 1");
        assert!(DpParams::new(1.0, 1e-6, 0).is_err(), "sensitivity >= 1");
        assert!(DpParams::new(f64::INFINITY, 1e-6, 1).is_err());
    }

    /// A too-strict (ε, δ) — tiny ε — needs an impractical shift and is refused
    /// fail-closed rather than padding to an enormous size.
    #[test]
    fn params_reject_too_strict_shift() {
        let err = DpParams::new(1e-5, 1e-9, 1).unwrap_err();
        assert!(
            matches!(err, MpcError::Protocol(m) if m.contains("too strict") || m.contains("MAX_DP_SHIFT")),
            "expected a shift-too-large protocol error"
        );
    }

    /// Boundary (review finding): an ε so small that `α = exp(−ε/Δ)` rounds to
    /// exactly `1.0` in f64 used to be ACCEPTED whenever δ kept the shift small —
    /// and then collapsed the sampler into a DETERMINISTIC release (`B = m + 1`
    /// always: zero log-denominators — no privacy at all). It must be rejected
    /// fail-closed at construction, for every δ.
    #[test]
    fn tiny_epsilon_alpha_collapse_is_rejected() {
        for delta in [0.9, 0.5, 1e-6] {
            let err = DpParams::new(1e-18, delta, 1).unwrap_err();
            assert!(
                matches!(err, MpcError::Protocol(ref m) if m.contains("alpha")),
                "delta={delta}: expected the strict-alpha rejection, got {err:?}"
            );
        }
        // The collapse is driven by the RATIO ε/Δ, so a huge sensitivity with a
        // moderate ε is the same degenerate case.
        assert!(
            DpParams::new(1e-3, 0.5, u64::MAX).is_err(),
            "eps/sensitivity ratio that rounds alpha to 1.0 must be rejected"
        );
    }

    /// Boundary: a huge ε/Δ underflows `α` to `0.0`, degenerating the formulas —
    /// rejected fail-closed rather than silently producing meaningless noise.
    #[test]
    fn huge_epsilon_alpha_underflow_is_rejected() {
        let err = DpParams::new(1e4, 1e-6, 1).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(m) if m.contains("alpha")));
    }

    /// Boundary (review finding): parameters whose `K_CAP` tail-truncation mass
    /// does not fit its δ/2 budget are rejected even when the shift is small. A
    /// large δ used to mask this: `MAX_DP_SHIFT` alone does not floor ε/Δ there,
    /// and the truncated tail was a non-negligible, unaccounted distortion.
    #[test]
    fn unaccounted_truncation_tail_is_rejected() {
        // alpha is strictly < 1 and the implied shift (~2e4) fits MAX_DP_SHIFT,
        // but the tail mass 2·α^(K_CAP+1) ≈ 1.2 far exceeds δ/2.
        let err = DpParams::new(5e-7, 0.99, 1).unwrap_err();
        assert!(
            matches!(err, MpcError::Protocol(ref m) if m.contains("truncation")),
            "expected the tail-truncation rejection, got {err:?}"
        );
    }

    /// Boundary: at the strictest ACCEPTED parameters (shift near [`MAX_DP_SHIFT`],
    /// tail accounted) the sampler still produces genuinely random noise — the
    /// release is not deterministic near the acceptance boundary.
    #[test]
    fn smallest_accepted_epsilon_still_noises() {
        let params = DpParams::new(2.2e-4, 1e-6, 1).unwrap();
        assert!(
            params.shift() <= MAX_DP_SHIFT && params.shift() > MAX_DP_SHIFT / 2,
            "test premise: shift {} should sit near the cap {MAX_DP_SHIFT}",
            params.shift()
        );
        let mut seen = std::collections::HashSet::new();
        for seed in 0..16u64 {
            let backend = ShamirBackend::new_seeded(3, 40_000 + seed).unwrap();
            let mut dealer = backend.dealer();
            seen.insert(sample_nonneg_noise(&mut dealer, &params));
        }
        assert!(
            seen.len() > 1,
            "noise never varied across 16 seeds at the acceptance boundary — the \
             sampler has collapsed"
        );
    }

    // ---- Sensitivity: a join's row-level sensitivity is generally NOT 1 --------

    /// Witness (review finding): for a join, adding ONE input row changes the
    /// output count by the number of rows it matches on the other side — here 4,
    /// not 1 — even for SET output. A caller passing `sensitivity = 1` for this
    /// shape under-noises the release ~4×; the supplied Δ must bound the join
    /// degree (or the caller must enforce a contribution bound). Pins the module
    /// documentation's Sensitivity section.
    #[test]
    fn join_fanout_sensitivity_witness() {
        // L ⋈ R on a key column; R holds 4 rows with key "k". The neighbouring
        // inputs differ by the single L row carrying key "k".
        let right_keys = ["k", "k", "k", "k", "other"];
        let join_count = |left: &[&str]| -> usize {
            left.iter()
                .map(|lk| right_keys.iter().filter(|rk| *rk == lk).count())
                .sum()
        };
        let without = join_count(&["a"]);
        let with = join_count(&["a", "k"]);
        assert_eq!(
            with - without,
            4,
            "one added input row changed the join's output count by 4, not 1"
        );
        // The honest parameterization for this shape carries Δ ≥ 4 — and pays for
        // it with a correspondingly larger shift (more noise/padding) than the
        // under-noised Δ = 1 it must not use.
        let under = DpParams::new(1.0, 1e-6, 1).unwrap();
        let honest = DpParams::new(1.0, 1e-6, 4).unwrap();
        assert!(
            honest.shift() > under.shift(),
            "Δ=4 must pad more than the (wrong-for-a-join) Δ=1: {} vs {}",
            honest.shift(),
            under.shift()
        );
    }

    /// Deterministic probability bound (review round 2): for neighbouring counts
    /// `m` and `m + Δ`, every output in `[m, m + Δ − 1]` is possible under the
    /// lower count but IMPOSSIBLE under the higher one (`B ≥ true count` always)
    /// — a perfectly distinguishing support-gap event whose WHOLE mass is
    /// charged to δ. The lower count lands there exactly when
    /// `max(0, Γ + G) ≤ Δ − 1`, i.e. `G ≤ Δ − 1 − Γ`. Sum the exact discrete
    /// two-sided-geometric point masses of that event and require it to fit the
    /// δ/2 clamp share — for Δ > 1 this is STRICTLY more than the zero-noise
    /// clamp event `G ≤ −Γ` the shift used to calibrate. Also checks the reverse
    /// DP direction: its bad event (the clamp atom `Pr[G ≤ −Γ]`, where the
    /// higher count's clamped release outweighs the lower count's point mass) is
    /// a subset of the same budget. Covers the fan-out-witness parameterization
    /// (Δ = 4) among others.
    #[test]
    fn support_gap_mass_fits_delta_for_sensitivity_above_one() {
        for (eps, delta, sens) in [
            (1.0, 1e-6, 4u64), // the join_fanout_sensitivity_witness parameters
            (0.5, 1e-4, 2),
            (2.0, 1e-9, 16),
            (1.0, 0.05, 1), // Δ = 1: the support gap degenerates to the clamp event
        ] {
            let p = DpParams::new(eps, delta, sens).unwrap();
            let gamma = p.shift() as i64;
            let alpha = (-(eps / sens as f64)).exp();
            // The support-gap event is G <= hi with hi = Δ-1-Γ; it must sit at
            // strictly negative noise (Γ >= Δ) or the gap would include the
            // no-noise point and could never fit a small δ.
            let hi = sens as i64 - 1 - gamma;
            assert!(hi < 0, "Γ = {gamma} must exceed Δ-1 = {}", sens - 1);
            // Exact point masses Pr[G = k] = (1-α)/(1+α)·α^{|k|}, summed over
            // k = hi, hi-1, ... The terms decay geometrically; 4096 terms leave
            // a remainder below f64 significance for every α accepted here.
            let coeff = (1.0 - alpha) / (1.0 + alpha);
            let mut summed = 0.0f64;
            for k in 0..4096i64 {
                summed += coeff * alpha.powi((hi - k).unsigned_abs() as i32);
            }
            // The closed-form tail Pr[G <= hi] = α^{-hi}/(1+α) must agree with
            // the summation (they are the same mass) and fit the δ/2 budget.
            let closed = alpha.powi((-hi) as i32) / (1.0 + alpha);
            assert!(
                (summed - closed).abs() <= 1e-12 * closed,
                "(ε={eps}, δ={delta}, Δ={sens}): summed gap mass {summed} disagrees \
                 with the closed-form tail {closed}"
            );
            assert!(
                closed <= delta / 2.0,
                "(ε={eps}, δ={delta}, Δ={sens}): support-gap mass {closed} exceeds \
                 its δ/2 budget {}",
                delta / 2.0
            );
            // Reverse DP direction: the clamp atom Pr[G <= -Γ] is a subset of
            // the same budget (Γ >= Δ ⇒ α^Γ <= α^{Γ-Δ+1}).
            let clamp_atom = alpha.powi(gamma as i32) / (1.0 + alpha);
            assert!(
                clamp_atom <= closed && clamp_atom <= delta / 2.0,
                "(ε={eps}, δ={delta}, Δ={sens}): reverse-direction clamp atom \
                 {clamp_atom} must fit inside the support-gap budget {closed}"
            );
        }
    }

    /// Regression witness (review round 2): at the fan-out-witness parameters
    /// (Δ = 4) the OLD shift — calibrated only against the zero-noise clamp
    /// `Pr[G ≤ −Γ₀] ≤ δ/2` — leaves a support-gap mass `α^{Γ₀−Δ+1}/(1+α)`
    /// ≈ `e^{ε(Δ−1)/Δ}` times the calibrated tail, which demonstrably EXCEEDS
    /// the δ/2 budget; the corrected shift is exactly Γ₀ + (Δ − 1).
    #[test]
    fn old_zero_noise_calibration_undershoots_for_fanout_sensitivity() {
        let (eps, delta, sens) = (1.0f64, 1e-6f64, 4u64);
        let p = DpParams::new(eps, delta, sens).unwrap();
        let alpha = (-(eps / sens as f64)).exp();
        // The pre-fix shift: smallest Γ₀ ≥ 1 with Pr[G ≤ −Γ₀] = α^Γ₀/(1+α) ≤ δ/2.
        let gamma_old = ((delta / 2.0 * (1.0 + alpha)).ln() / alpha.ln())
            .ceil()
            .max(1.0) as u64;
        assert_eq!(
            p.shift(),
            gamma_old + (sens - 1),
            "the corrected shift must widen the old one by exactly the Δ-1 support gap"
        );
        let old_gap_mass = alpha.powi((gamma_old - (sens - 1)) as i32) / (1.0 + alpha);
        assert!(
            old_gap_mass > delta / 2.0,
            "test premise: the old calibration must violate the support-gap budget \
             for Δ > 1 ({old_gap_mass} vs {})",
            delta / 2.0
        );
    }

    /// Larger ε (weaker privacy) ⇒ smaller shift (less padding); the shift is a pure
    /// function of the parameters.
    #[test]
    fn larger_epsilon_gives_smaller_shift() {
        let strong = DpParams::new(0.5, 1e-6, 1).unwrap();
        let weak = DpParams::new(2.0, 1e-6, 1).unwrap();
        assert!(
            weak.shift() < strong.shift(),
            "weaker privacy (larger eps) must pad less: weak={}, strong={}",
            weak.shift(),
            strong.shift()
        );
        assert!(strong.shift() >= 1);
    }

    // ---- The release: never truncates, reveals all reals -----------------------

    /// The released bound is ALWAYS ≥ the true count (non-negative noise), so every
    /// real row survives — across many seeds — and the revealed slot count equals the
    /// bound. This is the crate's never-truncate rule under DP.
    #[test]
    fn release_never_truncates_and_reveals_all_reals() {
        let params = DpParams::new(1.0, 1e-6, 1).unwrap();
        let data = rows(5);
        let expected = {
            let mut e: Vec<String> = data.iter().map(|r| format!("{r:?}")).collect();
            e.sort();
            e
        };
        for seed in 0..24u64 {
            let backend = ShamirBackend::new_seeded(3, 500 + seed).unwrap();
            let mut budget = PrivacyBudget::new(100.0, 1.0).unwrap();
            let (out, card) =
                dp_release_result_rows(&backend, &data, 1, &params, &mut budget).unwrap();
            let (slots, _cost) = out;
            assert!(
                card.revealed_bound >= card.true_count,
                "seed {seed}: bound {} < true {} — truncation!",
                card.revealed_bound,
                card.true_count
            );
            assert_eq!(card.true_count, 5);
            assert_eq!(card.noise as usize, card.revealed_bound - card.true_count);
            assert_eq!(slots.len(), card.revealed_bound, "reveal exactly B slots");
            assert_eq!(reals(&slots), 5, "all 5 real rows survive");
            assert_eq!(
                real_multiset(&slots),
                expected,
                "seed {seed}: real multiset changed under DP padding"
            );
        }
    }

    /// The released bound is DATA-DEPENDENT and NOISY: across seeds it varies (it is
    /// not a fixed public constant like the exact mode's bound). This is the leak DP
    /// bounds — and the reason a budget must track it.
    #[test]
    fn released_bound_is_noisy_and_data_dependent() {
        let params = DpParams::new(1.0, 1e-6, 1).unwrap();
        let data = rows(3);
        let mut seen = std::collections::HashSet::new();
        for seed in 0..48u64 {
            let backend = ShamirBackend::new_seeded(3, 9000 + seed).unwrap();
            let mut budget = PrivacyBudget::new(1000.0, 100.0).unwrap();
            let (_, card) =
                dp_release_result_rows(&backend, &data, 1, &params, &mut budget).unwrap();
            seen.insert(card.revealed_bound);
        }
        assert!(
            seen.len() > 1,
            "the released bound never varied across 48 seeds — noise is not being applied"
        );
    }

    /// Empty result set still releases a DP-noised (≥ 0) number of pure-dummy slots —
    /// the *existence* + a noised near-zero size is what leaks, not that it was empty.
    #[test]
    fn empty_result_still_releases_noise() {
        let params = DpParams::new(1.0, 1e-6, 1).unwrap();
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        let mut budget = PrivacyBudget::new(10.0, 1.0).unwrap();
        let (out, card) = dp_release_result_rows(&backend, &[], 1, &params, &mut budget).unwrap();
        let (slots, _) = out;
        assert_eq!(card.true_count, 0);
        assert_eq!(reals(&slots), 0, "no real rows");
        assert_eq!(slots.len(), card.revealed_bound);
        assert!(
            budget.spent_epsilon() > 0.99,
            "a SUCCESSFUL release must commit the charge (spent {})",
            budget.spent_epsilon()
        );
    }

    /// A row whose arity disagrees with `payload_arity` is refused (uniform schema)
    /// — and (review finding) the invalid request consumes NO budget: the charge is
    /// committed only when a release actually happens.
    #[test]
    fn wrong_arity_row_is_rejected_and_burns_no_budget() {
        let params = DpParams::new(1.0, 1e-6, 1).unwrap();
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        let mut budget = PrivacyBudget::new(10.0, 1.0).unwrap();
        let data = vec![vec![lit("a"), lit("b")]]; // arity 2
        let err = dp_release_result_rows(&backend, &data, 1, &params, &mut budget).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(m) if m.contains("arity")));
        assert_eq!(
            budget.spent_epsilon(),
            0.0,
            "a request that released nothing must not spend epsilon"
        );
        assert_eq!(
            budget.spent_delta(),
            0.0,
            "a request that released nothing must not spend delta"
        );
    }

    // ---- Budget: sequential composition + fail-closed --------------------------

    #[test]
    fn budget_composes_sequentially() {
        let mut b = PrivacyBudget::new(1.0, 1e-3).unwrap();
        let p = DpParams::new(0.4, 1e-4, 1).unwrap();
        b.charge(&p).unwrap(); // spent 0.4
        b.charge(&p).unwrap(); // spent 0.8
        assert!(b.spent_epsilon() > 0.79 && b.spent_epsilon() < 0.81);
        // Third charge (would be 1.2 > 1.0) is refused fail-closed, and does NOT
        // mutate the spend.
        let err = b.charge(&p).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(m) if m.contains("privacy budget exhausted")));
        assert!(b.spent_epsilon() < 0.81, "refused charge must not spend");
        assert!(b.remaining_epsilon() > 0.19 && b.remaining_epsilon() < 0.21);
    }

    /// The delta axis is enforced independently of epsilon.
    #[test]
    fn budget_enforces_delta_axis() {
        let mut b = PrivacyBudget::new(100.0, 1e-6).unwrap();
        // epsilon has plenty of room, but delta 1e-6 only affords ~1 charge of 1e-6.
        let p = DpParams::new(0.1, 1e-6, 1).unwrap();
        b.charge(&p).unwrap();
        let err = b.charge(&p).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(m) if m.contains("privacy budget exhausted")));
    }

    /// An over-budget release is refused BEFORE any crypto/reveal and does not spend.
    #[test]
    fn release_fails_closed_when_over_budget() {
        let params = DpParams::new(0.5, 1e-6, 1).unwrap();
        let backend = ShamirBackend::new_seeded(3, 3).unwrap();
        let mut budget = PrivacyBudget::new(0.1, 1e-9).unwrap(); // too small for eps=0.5
        let err =
            dp_release_result_rows(&backend, &rows(2), 1, &params, &mut budget).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(m) if m.contains("privacy budget exhausted")));
        assert!(
            budget.spent_epsilon() < 1e-12,
            "refused release must not spend"
        );
    }

    // ---- Ledger: per-relying-party independence --------------------------------

    #[test]
    fn ledger_budgets_are_per_relying_party() {
        let mut ledger = BudgetLedger::new();
        let alice = HolderId::new("alice");
        let bob = HolderId::new("bob");
        ledger.register(alice.clone(), PrivacyBudget::new(1.0, 1e-3).unwrap());
        ledger.register(bob.clone(), PrivacyBudget::new(1.0, 1e-3).unwrap());
        let p = DpParams::new(0.8, 1e-4, 1).unwrap();

        ledger.charge(&alice, &p).unwrap(); // alice: 0.8 spent
        // Alice's second charge (1.6 > 1.0) is refused; Bob is untouched.
        assert!(ledger.charge(&alice, &p).is_err());
        ledger.charge(&bob, &p).unwrap(); // bob still has full budget
        assert!(ledger.budget(&bob).unwrap().spent_epsilon() > 0.79);
        assert!(ledger.budget(&alice).unwrap().spent_epsilon() > 0.79);
    }

    /// An unregistered relying party has no budget — a release request is refused
    /// fail-closed (an unknown consumer cannot spend privacy).
    #[test]
    fn unregistered_party_is_refused() {
        let mut ledger = BudgetLedger::new();
        let p = DpParams::new(0.5, 1e-6, 1).unwrap();
        let err = ledger.charge(&HolderId::new("mallory"), &p).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(m) if m.contains("no registered")));
    }

    // ---- The hidden-key variant is honestly gated ------------------------------

    #[test]
    fn hidden_key_dp_is_honestly_gated() {
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        let params = DpParams::new(1.0, 1e-6, 1).unwrap();
        let mut budget = PrivacyBudget::new(10.0, 1.0).unwrap();
        let err = dp_oblivious_set_output_hidden_keys(
            &backend,
            &[crate::field::Fp::new(1)],
            &[crate::field::Fp::new(1)],
            &params,
            &mut budget,
        )
        .unwrap_err();
        match err {
            MpcError::NotYetImplemented { what, gated_on } => {
                assert!(what.contains("SECRET per-pair match bits"), "what: {what}");
                assert!(
                    gated_on.contains("oblivious compaction"),
                    "gated_on must name the missing primitive: {gated_on}"
                );
            }
            other => panic!("expected NotYetImplemented gate, got {other:?}"),
        }
    }

    // ---- Statistical sanity: the clamp (delta) event is rare -------------------

    /// The shift is calibrated so `Pr[Γ + G < 0]` (the clamp event, where the
    /// released bound would fall back to exactly the true count) is ≤ δ/2. With δ = 0.05
    /// and a modest sample we should see the clamp fire only a small fraction of the
    /// time — a coarse empirical check that the shift is sized right, not a proof.
    #[test]
    fn clamp_event_is_bounded_by_delta() {
        let params = DpParams::new(1.0, 0.05, 1).unwrap();
        let mut clamped = 0usize;
        let trials = 400usize;
        for seed in 0..trials as u64 {
            let backend = ShamirBackend::new_seeded(3, 20_000 + seed).unwrap();
            let mut dealer = backend.dealer();
            // The clamp fires exactly when the raw shifted draw is <= 0, i.e. noise == 0.
            let noise = sample_nonneg_noise(&mut dealer, &params);
            if noise == 0 {
                clamped += 1;
            }
        }
        // Generous slack over delta=0.05 for a 400-sample estimate; the point is it is
        // SMALL (well under, say, 20%), confirming the shift is sized, not that the
        // exact rate matches delta.
        assert!(
            (clamped as f64) / (trials as f64) < 0.20,
            "clamp fired {clamped}/{trials} — shift is undersized"
        );
    }
}
