// [OPUS-4.8] Honest-majority Shamir secret-sharing backend (M3, first concrete MpcBackend).
//! Shamir `t`-of-`n` secret sharing — the first concrete [`MpcBackend`] (M3).
//!
//! Architecture refs: §3.1 (secret-sharing families), §4.2 (trust model:
//! **honest-majority** is the stated first target), §4.3 step 4 (the secure
//! computation over secret-shared per-source values). Skill `mpc-protocols`:
//! "honest-majority … semi-honest among *cooperating* holders … is the viable
//! first target".
//!
//! ## Scheme chosen: Shamir `t`-of-`n` over `F_p` — and WHY (vs replicated 3PC)
//!
//! The driving use case is the **four flatmates** (`research/…architecture.md`
//! §2; Wright CEUR Vol-4085): N *cooperating* holders, each with a private
//! payslip value, jointly proving a cumulative aggregate without revealing
//! individual values. Two honest-majority candidates from the skill:
//!
//! - **Replicated 3PC secret sharing** is the *performance* sweet spot
//!   (1 element / mult-gate) **but is fixed at n = 3**: each of three parties
//!   holds two of three additive shares, assuming ≥2-of-3 non-colluding compute
//!   parties. It does **not** generalise to "four flatmates", or five, or any N.
//! - **Shamir `t`-of-`n`** uses a degree-`t` random polynomial whose constant
//!   term is the secret; honest-majority means `t < n/2`. It **generalises to
//!   any N** (the flatmate count is a deployment parameter, not a constant baked
//!   into the protocol), additions/linear combinations are **local / non-
//!   interactive** (free — no rounds), and reconstruction is exact Lagrange
//!   interpolation. The cumulative-salary aggregate is a *pure linear function*
//!   of the per-holder inputs, so under Shamir it costs **zero communication
//!   rounds** for the arithmetic itself — the dominant honest-majority win for
//!   THIS computation.
//!
//! **Decision (resolving Q2 for v1): Shamir `t`-of-`n`, honest-majority,
//! semi-honest.** It matches the "any number of cooperating flatmates" shape
//! that replicated-3PC cannot, and the aggregate we secure first is linear, so
//! Shamir's free-addition property is exactly the right cost profile. Replicated
//! 3PC would be the choice only if N were pinned at 3 and multiplication-heavy.
//!
//! ## Security model — stated explicitly (do not paper over)
//!
//! - **Honest-majority, semi-honest (honest-but-curious).** Privacy holds as
//!   long as **fewer than `t+1`** parties pool their shares. With threshold `t`,
//!   any set of `≤ t` shares is information-theoretically independent of the
//!   secret (Shamir 1979): `t` points leave one degree of freedom, so every
//!   candidate secret is equally consistent. The honest-majority instantiation
//!   sets `t = ⌊(n-1)/2⌋`, so a strict majority of honest parties cannot have
//!   their shares completed into the secret by the minority.
//! - **What each party learns:** ONLY its own share(s) of each input and of the
//!   result, plus the disclosed reconstructed output. It learns nothing about
//!   another holder's private input (confidentiality, guarantee (A)).
//! - **NOT in scope for v1:** active/malicious deviation (guarantee (D)). A
//!   semi-honest party follows the protocol; a malicious one could feed
//!   inconsistent shares. Malicious honest-majority (≈2× cost, e.g. with
//!   information-theoretic MACs / verifiable secret sharing) is a *future*
//!   hardening behind the SAME [`MpcBackend`] trait — see [`crate::backend`] doc
//!   and PLAN M3/M4. We do NOT claim malicious security here.
//!
//! ## Randomness (sq-1vt: CSPRNG, resolved) [OPUS-4.8]
//!
//! The masking polynomial coefficients and the equality-test mask need
//! randomness, and that randomness is **security-critical**: if it is
//! predictable, shares and masks become predictable and confidentiality
//! collapses (sq-1vt). The randomness therefore comes through the [`crate::rng`]
//! seam:
//!
//! - The **production / default** path ([`ShamirBackend::new`] →
//!   [`ShamirBackend::dealer`]) mints a fresh [`crate::rng::SecureRng`] — a
//!   ChaCha20 CSPRNG seeded from OS entropy — for each dealing session. Field
//!   elements are drawn uniformly via rejection sampling (no modulo bias).
//! - A **deterministic, seedable** path (`ShamirBackend::new_seeded`, gated
//!   behind `#[cfg(any(test, feature = "insecure-test-rng"))]` — so not an
//!   intra-doc link in default builds) drives the
//!   in-process multi-party SIMULATION and its differential/stress tests
//!   reproducibly. It is feature-gated out of normal builds: the real masking
//!   path cannot reach the deterministic RNG. No security is claimed for it.
//!
//! Crucially the live RNG state lives on a short-lived [`ShamirDealer`], not on
//! the `Clone`-able [`ShamirBackend`] config — so cloning a backend never
//! duplicates (and thus reuses) a CSPRNG keystream. Each `dealer()` call gets
//! independent randomness.

use crate::backend::{
    BackendInfo, MaliciousSecurity, MpcBackend, OperatorClass, SecurityDescriptor,
};
use crate::field::Fp;
use crate::holder::Holder;
use crate::partial::{HolderId, MpcError, PartialResult};
use crate::rng::{MpcRng, SecureRng};

/// A single Shamir share: the polynomial evaluated at a party's nonzero point.
/// `x` is the party index (1-based; the secret sits at `x = 0`), `y = f(x)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Share {
    /// The evaluation point (party index, always `>= 1`). `x = 0` is reserved
    /// for the secret and is never handed to a party.
    pub x: u64,
    /// The share value `f(x)` in `F_p`.
    pub y: Fp,
}

/// How a [`ShamirBackend`] seeds the masking RNG for each dealing session.
///
/// This is a *descriptor*, not live RNG state — it is cheaply `Clone`/`Copy` and
/// carries no keystream, so cloning a backend can never duplicate (and reuse) a
/// CSPRNG. The live randomness is minted per session by [`ShamirBackend::dealer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RngSource {
    /// **Production / default.** Each dealing session gets a fresh
    /// [`SecureRng`] (ChaCha20 CSPRNG) seeded from OS entropy. This is the only
    /// variant a default-feature build can construct (sq-1vt).
    Os,
    /// **Test/benchmark only.** Deterministic, seedable masking RNG for
    /// reproducible simulation. Gated out of normal builds.
    #[cfg(any(test, feature = "insecure-test-rng"))]
    InsecureSeed(u64),
}

/// Honest-majority Shamir `t`-of-`n` secret-sharing backend (the immutable
/// *configuration*).
///
/// `n` = number of compute parties (= holders in the flatmate model); the
/// privacy threshold is `t = ⌊(n-1)/2⌋` (honest majority). The backend holds NO
/// live RNG state — only `n`, `t`, and the `RngSource` descriptor (private) — so it is
/// freely `Clone`-able without ever cloning a CSPRNG keystream. The masking
/// randomness lives on the short-lived [`ShamirDealer`] minted by [`Self::dealer`].
///
/// The backend is the *coordinator-free* description of the scheme; the
/// in-process multi-party simulation that runs the parties is driven by
/// [`MpcBackend::run_secure`] + the helpers below, which is what the differential
/// tests exercise.
#[derive(Debug, Clone)]
pub struct ShamirBackend {
    n: usize,
    t: usize,
    rng_source: RngSource,
}

impl ShamirBackend {
    /// Build a **production** honest-majority backend for `n` parties (`n >= 2`).
    /// The privacy threshold is the honest-majority maximum `t = ⌊(n-1)/2⌋`: any
    /// `<= t` colluding parties learn nothing; reconstruction needs `>= t+1`.
    ///
    /// Masking randomness comes from a fresh OS-seeded ChaCha20 CSPRNG per
    /// dealing session — the only RNG this constructor wires in (sq-1vt). There
    /// is no seed parameter: a real masking RNG must not be predictable. For a
    /// reproducible test simulation use `Self::new_seeded` (test-gated, so not an
    /// intra-doc link in default builds).
    pub fn new(n: usize) -> Result<Self, MpcError> {
        Self::with_source(n, RngSource::Os)
    }

    /// Build a backend whose masking RNG is a **deterministic, seedable**
    /// SplitMix64 — **for reproducible tests / benchmarks ONLY**. Gated behind
    /// `#[cfg(any(test, feature = "insecure-test-rng"))]` so the real protocol
    /// cannot construct it. The masks it produces are predictable; never use it
    /// for a real deployment (that is the sq-1vt weakness).
    #[cfg(any(test, feature = "insecure-test-rng"))]
    pub fn new_seeded(n: usize, seed: u64) -> Result<Self, MpcError> {
        Self::with_source(n, RngSource::InsecureSeed(seed))
    }

    fn with_source(n: usize, rng_source: RngSource) -> Result<Self, MpcError> {
        if n < 2 {
            return Err(MpcError::Protocol(
                "Shamir honest-majority backend needs n >= 2 parties".into(),
            ));
        }
        let t = (n - 1) / 2;
        Ok(ShamirBackend { n, t, rng_source })
    }

    /// Number of compute parties.
    pub fn parties(&self) -> usize {
        self.n
    }

    /// The privacy threshold `t`: any subset of `<= t` parties' shares is
    /// independent of the secret.
    pub fn threshold(&self) -> usize {
        self.t
    }

    /// The active-security guarantee this `(n, t)` configuration delivers at the
    /// degree-`t` reconstruction (WI-1, parent bead sq-uu0u), derived purely from
    /// the RS redundancy `n − (t + 1)`. See [`MaliciousSecurity`] and
    /// [`crate::robust::reconstruct_robust`] for the bound; surfaced via
    /// [`MpcBackend::info`]. `[OPUS-4.8]`
    pub fn malicious_security(&self) -> MaliciousSecurity {
        // No redundancy at exactly `t+1` shares: tampering is information-
        // theoretically undetectable, so claim nothing.
        if self.n <= self.t + 1 {
            return MaliciousSecurity::SemiHonestOnly;
        }
        // RS / Berlekamp–Welch correction budget at degree `t`.
        let max_cheaters = (self.n - self.t - 1) / 2;
        if max_cheaters == 0 {
            // One redundant share lets us cross-check and detect, but not correct.
            MaliciousSecurity::HonestMajorityAbort
        } else {
            MaliciousSecurity::HonestMajorityRobust { max_cheaters }
        }
    }

    /// [OPUS-4.8] sq-mq8q — the RS/Berlekamp–Welch correction budget
    /// `e = ⌊(n − degree − 1)/2⌋` at a given reconstruction `degree`, or `None`
    /// when there is NO redundancy (`n <= degree + 1`, so tampering is
    /// information-theoretically undetectable). Shared by the per-operator
    /// reporting below: the linear aggregate opens at `degree = t`, the equality
    /// join at `degree = 2t`.
    fn rs_correction_budget(&self, degree: usize) -> Option<usize> {
        if self.n <= degree + 1 {
            None
        } else {
            Some((self.n - degree - 1) / 2)
        }
    }

    /// [OPUS-4.8] sq-mq8q — the three-axis [`SecurityDescriptor`] for one
    /// [`OperatorClass`] at THIS `(n, t)`. Guarantees genuinely differ per
    /// operator (the degree-`t` aggregate carries RS redundancy for every valid
    /// honest-majority `(n, t)` and is robust; the degree-`2t` equality open has
    /// ZERO redundancy at `n = 2t+1` and is semi-honest-only there), so one
    /// backend-level bit would lie. Surfaced via [`MpcBackend::operator_security`].
    pub fn operator_descriptor(&self, operator: OperatorClass) -> SecurityDescriptor {
        match operator {
            // Linear aggregate: degree-`t` open. Has redundancy for every valid
            // honest-majority `t = ⌊(n−1)/2⌋`, so in practice it never hits the
            // no-redundancy branch — but we still match `None` explicitly rather
            // than collapsing it into `e = 0`. `unwrap_or(0)` would map the
            // no-redundancy case (tampering information-theoretically undetectable)
            // onto the SAME `e = 0` detect-and-abort descriptor as the
            // one-redundant-share case, over-claiming detection where there is
            // none. Mirror the `EqualityJoin` arm: `None` → `semi_honest_only`.
            // `[OPUS-4.8]` (Copilot review #87).
            OperatorClass::LinearAggregate => match self.rs_correction_budget(self.t) {
                None => SecurityDescriptor::semi_honest_only(self.n, self.t),
                Some(e) => SecurityDescriptor::shamir_degree_recon(self.n, self.t, e),
            },
            // Equality / hidden-value join: degree-`2t` open. No redundancy at
            // `n = 2t+1` (odd-`n` honest majority) → semi-honest-only; otherwise
            // the RS budget at degree `2t` decides detect-and-abort vs robust.
            OperatorClass::EqualityJoin => match self.rs_correction_budget(2 * self.t) {
                None => SecurityDescriptor::semi_honest_only(self.n, self.t),
                Some(e) => SecurityDescriptor::shamir_degree_recon(self.n, self.t, e),
            },
            // Comparison (`<`,`≤`,`>`) is not realized in-crypto in the crate
            // (disclosed operands are recomputed by the verifier outside the
            // crypto). Report the honest "no in-crypto guarantee" baseline so a
            // federation sees the gap rather than an over-claim. (Realizing it
            // in-crypto is tracked separately — see the operator matrix in
            // research/mpc-security-models-and-benchmarks.md §3.)
            OperatorClass::Comparison => SecurityDescriptor::semi_honest_only(self.n, self.t),
        }
    }

    /// Mint a fresh [`ShamirDealer`] with **independent** masking randomness for
    /// one dealing session. In production this seeds a brand-new OS-seeded
    /// ChaCha20 CSPRNG — so two dealers from the same (or a cloned) backend draw
    /// independent, unpredictable masks, never a reused keystream.
    pub fn dealer(&self) -> ShamirDealer {
        let rng: Box<dyn MpcRng> = match self.rng_source {
            RngSource::Os => Box::new(SecureRng::from_os()),
            #[cfg(any(test, feature = "insecure-test-rng"))]
            RngSource::InsecureSeed(seed) => Box::new(crate::rng::InsecureTestRng::new(seed)),
        };
        ShamirDealer {
            n: self.n,
            t: self.t,
            rng,
        }
    }

    /// Reconstruct the secret `f(0)` from `>= t+1` shares. Fewer than `t+1`
    /// shares is a protocol error (the whole point of the threshold). Shares must
    /// have distinct `x`. RNG-free.
    ///
    /// **Consistency-checked / robust (sq-m34i, WI-1).** When redundancy is
    /// present (`n > t+1`) this routes through [`crate::robust::reconstruct_robust`]:
    /// it verifies all points lie on one degree-`t` polynomial, CORRECTS up to
    /// `e = ⌊(n−t−1)/2⌋` tampered shares (returning the true secret), and aborts
    /// with [`MpcError::Tampered`] on any inconsistency it cannot repair —
    /// closing the malicious-security gap (D) at the Shamir layer (parent bead
    /// sq-uu0u). On clean input it returns exactly the same value as plain
    /// Lagrange (no behaviour change). At exactly `t+1` shares (no redundancy)
    /// tampering is information-theoretically undetectable, so it falls back to
    /// plain Lagrange and makes NO detection claim.
    pub fn reconstruct(&self, shares: &[Share]) -> Result<Fp, MpcError> {
        crate::robust::reconstruct_robust(shares, self.t)
    }
}

/// A short-lived dealer holding **live masking randomness** for one sharing
/// session. It owns the CSPRNG (production) / deterministic PRNG (test) and is
/// the only thing that draws masking field elements. Created by
/// [`ShamirBackend::dealer`]; not `Clone` (its RNG state must not be duplicated).
pub struct ShamirDealer {
    n: usize,
    t: usize,
    rng: Box<dyn MpcRng>,
}

impl std::fmt::Debug for ShamirDealer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShamirDealer")
            .field("n", &self.n)
            .field("t", &self.t)
            .field("rng", &"<live masking RNG>")
            .finish()
    }
}

impl ShamirDealer {
    /// Number of compute parties (mirrors the backend's).
    pub fn parties(&self) -> usize {
        self.n
    }

    /// The privacy threshold `t` (mirrors the backend's).
    pub fn threshold(&self) -> usize {
        self.t
    }

    /// Draw one uniform field element from the masking RNG (advances state).
    /// Used by the equality-test mask. Rejection-sampled for exact uniformity
    /// (see [`crate::rng`]).
    pub fn draw_fp(&mut self) -> Fp {
        self.rng.next_fp()
    }

    /// Draw a uniform **nonzero** field element. The equality mask must be
    /// nonzero (a zero mask would open `m = 0` even for unequal keys).
    pub fn draw_nonzero_fp(&mut self) -> Fp {
        self.rng.next_nonzero_fp()
    }

    /// Secret-share one field element into `n` shares on a fresh degree-`t`
    /// polynomial `f` with `f(0) = secret`. Free coefficients are uniform random
    /// from the masking RNG (a CSPRNG in production — sq-1vt).
    pub fn share(&mut self, secret: Fp) -> Vec<Share> {
        // f(z) = secret + c_1 z + ... + c_t z^t.
        let mut coeffs = Vec::with_capacity(self.t + 1);
        coeffs.push(secret);
        for _ in 0..self.t {
            coeffs.push(self.rng.next_fp());
        }
        (1..=self.n as u64)
            .map(|x| Share {
                x,
                y: eval_poly(&coeffs, Fp::new(x)),
            })
            .collect()
    }

    /// **BGW/GRR degree-reduction round (sq-dvuc).** Reduce a degree-`2t` product
    /// sharing back to a fresh degree-`t` sharing of the SAME secret, so that
    /// secret-shared multiplications can **chain** (`a·b·c`, secure comparison /
    /// threshold, conjunctive hidden-pattern joins) instead of being limited to a
    /// single non-reducing product ([`mul_shares_raw`]). `[OPUS-4.8]`
    ///
    /// This is the standard **BGW** reduction with a **public recombination
    /// vector** (Gennaro–Rabin–Rabin'98 simplification of BGW'88), run over the
    /// in-process party simulation:
    ///
    /// 1. The component-wise product `h_i = f(x_i)·g(x_i)` of two degree-`t`
    ///    sharings lies on a degree-`2t` polynomial `H` with `H(0) = a·b` (the
    ///    secret product). Reconstructing `H(0)` from its first `2t+1` evaluation
    ///    points is a FIXED public linear map — the Lagrange-at-0 weights
    ///    `λ_1..λ_{2t+1}`: `H(0) = Σ_i λ_i · h_i`.
    /// 2. Each simulated party `i` (`i = 1..2t+1`) **re-shares** its own degree-`2t`
    ///    share value `h_i` under a FRESH, INDEPENDENT degree-`t` polynomial
    ///    (`[h_i]_t`), using this dealer's masking RNG — a fresh OS-seeded ChaCha20
    ///    CSPRNG in production (sq-1vt). This is the one (simulated) communication
    ///    round.
    /// 3. Every party then locally applies the SAME public recombination vector to
    ///    the sub-shares it received: `[a·b]_t = Σ_i λ_i · [h_i]_t`. Because the
    ///    `λ_i` are public scalars and the `[h_i]_t` are degree-`t` sharings, the
    ///    result is a degree-`t` sharing whose secret is `Σ_i λ_i·H(x_i) = H(0) =
    ///    a·b` — a fresh degree-`t` sharing of the original product.
    ///
    /// **Precondition (fail-closed):** `degree_reduce` requires `n >= 2t+1` (so a
    /// degree-`2t` polynomial is determined by the `n` party points and the `2t+1`
    /// recombination points exist). The honest-majority constructor already fixes
    /// `t = ⌊(n−1)/2⌋`, so every backend `Self` builds satisfies this; the check is
    /// here to fail with a descriptive [`MpcError::Protocol`] rather than panic if a
    /// short / mis-built share vector is ever passed. The input must be the full
    /// `n`-party degree-`2t` sharing on the canonical points `x = 1..n`.
    ///
    /// **Security model — UNCHANGED honest-majority / semi-honest (do not
    /// over-claim).** The reduction is the BGW reduction step under the SAME trust
    /// assumptions as the rest of this backend (module docs §"Security model"): it
    /// is correct and confidentiality-preserving when every party follows the
    /// protocol and at most `t` collude. It is **NOT** maliciously secure: a
    /// deviating party can feed an inconsistent re-sharing, and — exactly as for the
    /// degree-`2t` equality open at `n = 2t+1` — there is no in-protocol check here
    /// that detects it. Each fresh re-sharing draws its own random masking
    /// coefficients, so any `≤ t` parties' view of the sub-shares is independent of
    /// the reduced secret (the standard BGW privacy argument). Malicious hardening
    /// (IT-MACs / verifiable resharing) is future work behind the SAME backend, not
    /// claimed here. See `research/mpc-security-models-and-benchmarks.md` §3 (the
    /// "general Shamir multiplication needs degree reduction" gap) and §6 step 6.
    ///
    /// Returns a fresh degree-`t` sharing on the canonical points `x = 1..n`.
    pub fn degree_reduce(&mut self, shares_2t: &[Share]) -> Result<Vec<Share>, MpcError> {
        // Fail-closed precondition: we need n >= 2t+1 distinct party points so the
        // degree-2t polynomial is over-determined and 2t+1 recombination points
        // exist. (Equivalently: the supplied sharing must cover the full party set.)
        if shares_2t.len() < 2 * self.t + 1 {
            return Err(MpcError::Protocol(format!(
                "degree_reduce: need a degree-2t product sharing on n >= 2t+1 = {} parties, \
                 got {} shares (honest-majority needs n >= 2t+1 to reduce a degree-2t product)",
                2 * self.t + 1,
                shares_2t.len()
            )));
        }
        if shares_2t.len() != self.n {
            return Err(MpcError::Protocol(format!(
                "degree_reduce: expected the full {}-party sharing, got {} shares",
                self.n,
                shares_2t.len()
            )));
        }

        // Public recombination vector: the Lagrange-at-0 weights for the first
        // 2t+1 evaluation points. H(0) = Σ_{i=1}^{2t+1} λ_i · H(x_i), a FIXED
        // public linear map (depends only on the party points, never on secrets).
        let recomb_points: Vec<u64> = shares_2t[..2 * self.t + 1].iter().map(|s| s.x).collect();
        let lambdas = lagrange_zero_weights(&recomb_points);

        // Step 2: each of the 2t+1 recombination parties re-shares its degree-2t
        // share h_i under a FRESH, independent degree-t polynomial. Each re-sharing
        // is an n-vector of sub-shares on the canonical points x = 1..n.
        let mut resharings: Vec<Vec<Share>> = Vec::with_capacity(2 * self.t + 1);
        for s in &shares_2t[..2 * self.t + 1] {
            resharings.push(self.share(s.y));
        }

        // Step 3: every party j locally forms Σ_i λ_i · (sub-share i held by j).
        // The λ_i are public scalars, the sub-sharings are degree-t, so the result
        // is a degree-t sharing of Σ_i λ_i·h_i = H(0) = a·b.
        let mut reduced: Vec<Share> = shares_2t
            .iter()
            .map(|s| Share {
                x: s.x,
                y: Fp::zero(),
            })
            .collect();
        for (i, sub) in resharings.iter().enumerate() {
            let lambda = lambdas[i];
            for (out, sub_share) in reduced.iter_mut().zip(sub.iter()) {
                debug_assert_eq!(out.x, sub_share.x, "resharing must use canonical points");
                out.y = out.y.add(lambda.mul(sub_share.y));
            }
        }
        Ok(reduced)
    }

    /// Reconstruct (RNG-free). Like [`ShamirBackend::reconstruct`], this uses the
    /// consistency-checked / robust path (sq-m34i) when redundancy is present.
    pub fn reconstruct(&self, shares: &[Share]) -> Result<Fp, MpcError> {
        crate::robust::reconstruct_robust(shares, self.t)
    }
}

/// Evaluate a polynomial (coeffs low-to-high) at `x` by Horner's method.
fn eval_poly(coeffs: &[Fp], x: Fp) -> Fp {
    let mut acc = Fp::zero();
    for &c in coeffs.iter().rev() {
        acc = acc.mul(x).add(c);
    }
    acc
}

/// The public **Lagrange-at-0 recombination weights** `λ_i` for evaluation points
/// `xs`: the unique scalars with `P(0) = Σ_i λ_i · P(x_i)` for any polynomial `P`
/// of degree `< xs.len()` sampled at `xs`. `λ_i = Π_{j≠i} (0 − x_j)/(x_i − x_j)`.
///
/// This is the SAME basis-weight computation as Lagrange interpolation at zero,
/// factored out so [`ShamirDealer::degree_reduce`] can reuse it as a FIXED public
/// linear map (sq-dvuc). The points must be distinct and nonzero; the caller
/// guarantees this (party indices are `1..=n`). RNG-free / public. `[OPUS-4.8]`
fn lagrange_zero_weights(xs: &[u64]) -> Vec<Fp> {
    xs.iter()
        .enumerate()
        .map(|(i, &xi_raw)| {
            let xi = Fp::new(xi_raw);
            let mut num = Fp::one();
            let mut den = Fp::one();
            for (j, &xj_raw) in xs.iter().enumerate() {
                if i == j {
                    continue;
                }
                let xj = Fp::new(xj_raw);
                num = num.mul(xj.neg()); // (0 − x_j)
                den = den.mul(xi.sub(xj)); // (x_i − x_j)
            }
            num.mul(den.inv())
        })
        .collect()
}

/// Lagrange-interpolate the shares' polynomial and evaluate it at `x = 0` (the
/// secret). Requires `>= t+1` distinct points.
///
/// This is the unchecked RS-decode-under-no-errors primitive — it does NOT
/// detect tampering. Every PRODUCTION reconstruction path now routes through the
/// consistency-checked / robust entry point [`crate::robust::reconstruct_robust`]
/// (wired into [`ShamirBackend::reconstruct`] AND, since WI-2 / sq-7q9i, into the
/// degree-`2t` [`reconstruct_degree`] open). This unchecked Lagrange-at-0 helper
/// therefore has NO production caller left; it is kept `#[cfg(test)]` purely as
/// the differential REFERENCE the adversarial suite compares the robust path
/// against (robust-vs-robust would be vacuous — see `robust.rs` tests).
#[cfg(test)]
pub(crate) fn reconstruct_at_zero(shares: &[Share], t: usize) -> Result<Fp, MpcError> {
    if shares.len() < t + 1 {
        return Err(MpcError::Protocol(format!(
            "Shamir reconstruction needs >= {} shares, got {}",
            t + 1,
            shares.len()
        )));
    }
    // Lagrange at 0: sum_i y_i * prod_{j != i} (0 - x_j)/(x_i - x_j).
    let mut secret = Fp::zero();
    for (i, si) in shares.iter().enumerate() {
        let xi = Fp::new(si.x);
        let mut num = Fp::one();
        let mut den = Fp::one();
        for (j, sj) in shares.iter().enumerate() {
            if i == j {
                continue;
            }
            let xj = Fp::new(sj.x);
            num = num.mul(xj.neg()); // (0 - x_j)
            den = den.mul(xi.sub(xj)); // (x_i - x_j)
        }
        secret = secret.add(si.y.mul(num.mul(den.inv())));
    }
    Ok(secret)
}

/// Add two share-vectors component-wise (parties at the same `x` add their
/// shares). This is the **local, non-interactive** linear operation that makes
/// the cumulative-sum aggregate free under Shamir: `Share(a) + Share(b)` is a
/// valid sharing of `a + b` on the SAME points, with no communication. Both
/// inputs must be sharings on the identical party-point set.
pub fn add_shares(a: &[Share], b: &[Share]) -> Result<Vec<Share>, MpcError> {
    if a.len() != b.len() {
        return Err(MpcError::Protocol(
            "add_shares: share vectors differ in length (different party sets)".into(),
        ));
    }
    a.iter()
        .zip(b.iter())
        .map(|(sa, sb)| {
            if sa.x != sb.x {
                return Err(MpcError::Protocol(
                    "add_shares: shares are on different evaluation points".into(),
                ));
            }
            Ok(Share {
                x: sa.x,
                y: sa.y.add(sb.y),
            })
        })
        .collect()
}

/// Add a public field constant to a sharing (local, non-interactive). Adding `c`
/// to every share replaces `f(x)` by `f(x) + c`, whose value at `x = 0` is
/// `secret + c`, still a valid degree-`t` sharing.
pub fn add_constant(a: &[Share], c: Fp) -> Vec<Share> {
    a.iter()
        .map(|s| Share {
            x: s.x,
            y: s.y.add(c),
        })
        .collect()
}

/// Multiply a sharing by a public field constant (local, non-interactive): scale
/// every share. `c * f(x)` interpolates to `c * secret` at `x = 0` and stays
/// degree `t`.
pub fn scale(a: &[Share], c: Fp) -> Vec<Share> {
    a.iter()
        .map(|s| Share {
            x: s.x,
            y: s.y.mul(c),
        })
        .collect()
}

/// Subtract two sharings component-wise (local). `Share(a) - Share(b)` is a
/// valid degree-`t` sharing of `a - b`.
pub fn sub_shares(a: &[Share], b: &[Share]) -> Result<Vec<Share>, MpcError> {
    if a.len() != b.len() {
        return Err(MpcError::Protocol("sub_shares: length mismatch".into()));
    }
    a.iter()
        .zip(b.iter())
        .map(|(sa, sb)| {
            if sa.x != sb.x {
                return Err(MpcError::Protocol("sub_shares: point mismatch".into()));
            }
            Ok(Share {
                x: sa.x,
                y: sa.y.sub(sb.y),
            })
        })
        .collect()
}

/// **Local share-products before degree reduction.** Multiplying two degree-`t`
/// sharings component-wise yields, at each party point, `f(x)·g(x)` — a sharing
/// of `a·b` but on a polynomial of degree `2t`. Reconstructing it therefore
/// needs `2t+1` points (honest-majority gives `2t+1 <= n`). For a single product
/// that is opened immediately (the equality test opens its masked product) this
/// degree-`2t` sharing is reconstructed directly via [`reconstruct_degree`] — no
/// further work is needed.
///
/// To CHAIN multiplications (`a·b·c`, secure comparison/threshold, conjunctive
/// hidden-pattern joins) the degree-`2t` product must be brought back to degree
/// `t` first: feed the result of `mul_shares_raw` into
/// [`ShamirDealer::degree_reduce`] (the BGW reshare-and-recombine round, sq-dvuc)
/// before the next multiplication.
///
/// HONESTY: the round/degree cost is real and stated — one multiplication is one
/// interaction round and consumes the `n >= 2t+1` headroom; the degree-reduction
/// round (sq-dvuc) is a SECOND (simulated) round that restores degree `t` so the
/// next product fits, under the SAME honest-majority / semi-honest model.
pub fn mul_shares_raw(a: &[Share], b: &[Share]) -> Result<Vec<Share>, MpcError> {
    if a.len() != b.len() {
        return Err(MpcError::Protocol("mul_shares_raw: length mismatch".into()));
    }
    a.iter()
        .zip(b.iter())
        .map(|(sa, sb)| {
            if sa.x != sb.x {
                return Err(MpcError::Protocol("mul_shares_raw: point mismatch".into()));
            }
            Ok(Share {
                x: sa.x,
                y: sa.y.mul(sb.y),
            })
        })
        .collect()
}

/// Reconstruct the secret of a sharing of a known polynomial `degree`, requiring
/// `degree + 1` points. Used to open the degree-`2t` product of the equality
/// test (`degree = 2t`).
///
/// **Consistency-checked / robust at degree `degree` (sq-7q9i, WI-2).** This
/// routes the open through [`crate::robust::reconstruct_robust`] at the GIVEN
/// `degree` — the SAME Reed–Solomon checker WI-1 wired into the degree-`t`
/// reconstruction, just instantiated at the product's degree. The codeword view
/// is identical: a sharing of a degree-`degree` polynomial at `n` distinct points
/// is an `[n, degree+1]` RS codeword, so the checker:
///
/// - `n == degree + 1` (**NO redundancy**) → plain Lagrange; tampering is
///   information-theoretically undetectable and is NOT claimed otherwise.
/// - `n > degree + 1` (**redundancy**) → Berlekamp–Welch: detect any tampering
///   and abort with [`MpcError::Tampered`], correcting up to
///   `e = ⌊(n − degree − 1)/2⌋` tampered shares first.
///
/// HONESTY (the degree-`2t` boundary; bead sq-7q9i / parent sq-uu0u): for the
/// equality/mult open `degree = 2t`, redundancy exists ONLY when `n > 2t + 1`.
/// The honest-majority constructor fixes `t = ⌊(n−1)/2⌋`, so `n = 2t + 1` for odd
/// `n` (e.g. n=3,5,7,9): there is **zero** RS redundancy at degree `2t` and
/// tampering one product share is undetectable — pinned by a boundary test. A
/// true fix at `n = 2t + 1` needs an information-theoretic MAC (the deferred WI-4
/// seam, bead sq-6d6g), not RS redundancy. Even `n` (n=4,6,8) yields exactly one
/// redundant share at degree `2t` (`e_max = 0`), so tampering is DETECT-only
/// there; correction (`e_max ≥ 1`) at degree `2t` needs `n ≥ 2t + 3`.
pub fn reconstruct_degree(shares: &[Share], degree: usize) -> Result<Fp, MpcError> {
    // Same RS-consistency-checked entry point as ShamirBackend::reconstruct,
    // instantiated at the product's `degree` (here `2t`) rather than `t`. At
    // `n == degree+1` it falls back to plain Lagrange (no detection claim); with
    // redundancy it detects/corrects. See the doc above for the degree-2t bound.
    crate::robust::reconstruct_robust(shares, degree)
}

/// The Shamir `Share` representation surfaced through the trait. A holder's
/// private contribution is shared into one [`Share`] *per party*; we carry the
/// whole per-party vector as the trait's opaque `Share` so the rest of the crate
/// need not know the scheme's internals (see [`MpcBackend::Share`]).
pub type ShareVec = Vec<Share>;

impl MpcBackend for ShamirBackend {
    /// One trait-`Share` is the full per-party share-vector of a single secret.
    type Share = ShareVec;

    fn info(&self) -> BackendInfo {
        // [OPUS-4.8] sq-mq8q — the backend-level descriptor describes the PRIMARY
        // (degree-`t` linear-aggregate) reconstruction path. `BackendInfo::new`
        // derives the back-compat `trust_model` / `malicious_security` projection
        // from it, threading the degree-`t` RS correction budget `e` so the old
        // enum's `max_cheaters` stays faithful (this PRESERVES the exact prior
        // `info().malicious_security` value — `self.malicious_security()`). The
        // weaker degree-`2t` equality-open guarantee is NOT smuggled into the
        // backend-level bit; it is reported per-operator via `operator_security`.
        // Shamir shares are an [n, t+1] RS codeword, so `reconstruct` (degree `t`):
        //   - n == t+1 (NO redundancy)  → no detection possible → SemiHonestOnly.
        //   - n  > t+1, e == 0          → detect-and-abort       → Abort.
        //   - n  > t+1, e >= 1          → robust up to e cheaters → Robust{e}.
        // NB: with the honest-majority t = ⌊(n−1)/2⌋, every valid n >= 2 has
        // n > t+1, so the no-redundancy branch is UNREACHABLE for the aggregate
        // (it is real only for the degree-`2t` equality open / a stub backend).
        let e_t = self.rs_correction_budget(self.t).unwrap_or(0);
        BackendInfo::new(
            "shamir-honest-majority",
            self.operator_descriptor(OperatorClass::LinearAggregate),
            e_t,
        )
    }

    /// [OPUS-4.8] sq-mq8q — per-operator security: the degree-`t` aggregate is
    /// robust while the degree-`2t` equality open is semi-honest-only at
    /// `n = 2t+1`, so this reports each [`OperatorClass`] precisely rather than
    /// letting one backend-level bit lie. Delegates to [`Self::operator_descriptor`].
    fn operator_security(&self, operator: OperatorClass) -> SecurityDescriptor {
        self.operator_descriptor(operator)
    }

    /// Secret-share a holder's *private* contribution. For the cumulative-
    /// aggregate sub-case the holder's contribution is one private integer (its
    /// salary). We extract it from the holder's local single-row, single-column
    /// partial and share it across the `n` parties. The cleartext value NEVER
    /// leaves as cleartext — only its `n` shares do, and any `<= t` of them are
    /// independent of it.
    ///
    /// (The holder is queried for the private value via the SAME local-eval path
    /// as the disclosed flow — `evaluate_local` — but the value is shared rather
    /// than disclosed. The fragment must project exactly one integer-valued
    /// column in one row; otherwise it is a protocol error, not a guess.)
    fn share_private_input(&self, holder: &Holder) -> Result<Vec<Self::Share>, MpcError> {
        // The private contribution is named by a convention fragment: a single
        // integer the holder agrees to *secret-share* (not disclose). We use the
        // dedicated private-salary fragment.
        let private = holder.evaluate_local(PRIVATE_SALARY_FRAGMENT)?;
        let v = extract_single_integer(&private)?;
        // A fresh sharing requires fresh, INDEPENDENT randomness. Mint a fresh
        // dealer (production: a fresh OS-seeded CSPRNG — sq-1vt) rather than
        // cloning RNG state, so per-input sharings never reuse a keystream. The
        // trait stays `&self`; the dealer's randomness is per-input by design.
        let mut dealer = self.dealer();
        Ok(vec![dealer.share(Fp::new(v))])
    }

    /// Run the secure computation over the shared inputs. For the v1 aggregate
    /// this is the **cumulative sum** of the holders' private values: a pure
    /// linear function, so it is the local component-wise addition of the
    /// sharings ([`add_shares`]) — zero communication rounds, the honest-
    /// majority Shamir sweet spot. Returns the sharing of the sum.
    fn run_secure(&self, shares: &[Self::Share]) -> Result<Vec<Self::Share>, MpcError> {
        if shares.is_empty() {
            return Err(MpcError::Protocol("run_secure: no shared inputs".into()));
        }
        let mut acc = shares[0].clone();
        for next in &shares[1..] {
            acc = add_shares(&acc, next)?;
        }
        Ok(vec![acc])
    }

    /// Reconstruct ONLY the disclosed output (the aggregate sum). Per convention
    /// #4 the disclosed value is the minimal answer; here we reconstruct the
    /// summed sharing to a single integer and surface it as a one-row partial.
    /// Any disclosed-property post-processing (e.g. the boolean `sum > £100k`)
    /// is recomputed by the verifier OUTSIDE the crypto core (M5), not here.
    fn reconstruct_disclosed(
        &self,
        result_shares: &[Self::Share],
    ) -> Result<PartialResult, MpcError> {
        if result_shares.len() != 1 {
            return Err(MpcError::Protocol(
                "reconstruct_disclosed: expected exactly one result sharing".into(),
            ));
        }
        let sum = self.reconstruct(&result_shares[0])?;
        Ok(PartialResult {
            holder: HolderId::new("federation"),
            vars: vec![oxrdf::Variable::new_unchecked("cumulative")],
            rows: vec![vec![Some(oxrdf::Term::Literal(
                oxrdf::Literal::new_typed_literal(
                    sum.value().to_string(),
                    oxrdf::vocab::xsd::INTEGER,
                ),
            ))]],
        })
    }
}

/// The fragment a holder evaluates to surface its single private salary as a
/// value to be SECRET-SHARED (not disclosed). Kept distinct from any disclosed
/// fragment to make the disclose-vs-hide boundary explicit at the call site.
const PRIVATE_SALARY_FRAGMENT: &str =
    "PREFIX ex: <http://ex/> SELECT ?salary WHERE { ?p ex:salary ?salary }";

/// Pull a single non-negative integer out of a one-row / one-column partial.
/// Anything else (no rows, many rows, non-integer, multiple columns) is a
/// protocol error — we never guess a private input.
fn extract_single_integer(p: &PartialResult) -> Result<u64, MpcError> {
    if p.rows.len() != 1 {
        return Err(MpcError::Protocol(format!(
            "private input must be exactly one row, got {}",
            p.rows.len()
        )));
    }
    if p.vars.len() != 1 || p.rows[0].len() != 1 {
        return Err(MpcError::Protocol(
            "private input must be exactly one column".into(),
        ));
    }
    match &p.rows[0][0] {
        Some(oxrdf::Term::Literal(l)) => l
            .value()
            .parse::<u64>()
            .map_err(|e| MpcError::Protocol(format!("private input not a u64 integer: {e}"))),
        other => Err(MpcError::Protocol(format!(
            "private input must be an integer literal, got {other:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    //! M3 Shamir tests. The load-bearing ones are:
    //! - `reconstruct_*`: the secret-sharing LAYER is correct (round-trips, and
    //!   the threshold actually hides the secret below `t+1` shares).
    //! - `secure_sum_*`: `run_secure` computes the cumulative aggregate over
    //!   shares so that reconstructing it equals the PLAINTEXT sum — the M3
    //!   differential for the secure computation itself.
    use super::*;

    #[test]
    fn share_then_reconstruct_roundtrips() {
        // Use the seedable test RNG so the simulation is reproducible; the
        // PRODUCTION path uses the OS-seeded CSPRNG (see `production_csprng_*`).
        let b = ShamirBackend::new_seeded(4, 0xC0FFEE).unwrap();
        let mut dealer = b.dealer();
        for secret in [0u64, 1, 42, 100_000, crate::field::P - 1] {
            let shares = dealer.share(Fp::new(secret));
            assert_eq!(shares.len(), 4);
            // Full set reconstructs.
            assert_eq!(b.reconstruct(&shares).unwrap(), Fp::new(secret));
            // Any t+1 subset reconstructs (threshold t = (4-1)/2 = 1 → 2 shares).
            assert_eq!(b.threshold(), 1);
            assert_eq!(b.reconstruct(&shares[..2]).unwrap(), Fp::new(secret));
        }
    }

    #[test]
    fn fewer_than_threshold_plus_one_shares_cannot_reconstruct() {
        // t = 1, so a SINGLE share is below threshold: reconstruction must error,
        // not silently return a wrong/honest-looking value.
        let b = ShamirBackend::new_seeded(4, 1).unwrap();
        let shares = b.dealer().share(Fp::new(73));
        let err = b.reconstruct(&shares[..1]).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(_)));
    }

    #[test]
    fn production_csprng_share_reconstruct_roundtrips() {
        // The PRODUCTION path (OS-seeded ChaCha20 CSPRNG, sq-1vt) must still
        // produce correct sharings: round-trip the secret through fresh dealers.
        let b = ShamirBackend::new(4).unwrap();
        for secret in [0u64, 1, 42, 100_000, crate::field::P - 1] {
            let shares = b.dealer().share(Fp::new(secret));
            assert_eq!(b.reconstruct(&shares).unwrap(), Fp::new(secret));
        }
    }

    #[test]
    fn production_csprng_two_dealers_use_independent_randomness() {
        // sq-1vt unpredictability witness: two dealers minted from the SAME
        // (production) backend must NOT produce identical shares for the same
        // secret — each `dealer()` mints a fresh OS-seeded CSPRNG, so the masking
        // polynomials differ. (Collision is cryptographically negligible.)
        let b = ShamirBackend::new(5).unwrap(); // t = 2 → random coeffs present
        let s1 = b.dealer().share(Fp::new(42));
        let s2 = b.dealer().share(Fp::new(42));
        // Both reconstruct to 42 ...
        assert_eq!(b.reconstruct(&s1).unwrap(), Fp::new(42));
        assert_eq!(b.reconstruct(&s2).unwrap(), Fp::new(42));
        // ... but the share vectors differ (different random masking polynomials).
        assert_ne!(
            s1, s2,
            "two production dealers reused the same masking randomness"
        );
    }

    #[test]
    fn threshold_hides_secret_information_theoretically() {
        // CONFIDENTIALITY witness: with t = 2 (n = 5), any 2 shares are
        // consistent with EVERY candidate secret. Concretely: there exists a
        // valid polynomial through the 2 shares for any chosen f(0). We show the
        // 2 shares do not determine the secret by exhibiting two different
        // secrets whose sharings agree on the same 2 points.
        let b = ShamirBackend::new_seeded(5, 7).unwrap();
        assert_eq!(b.threshold(), 2);
        let shares_a = b.dealer().share(Fp::new(1000));
        // For ANY two of A's shares, a degree-2 poly through them + a third
        // chosen point reconstructs a DIFFERENT secret — i.e. 2 points underdet.
        let two = &shares_a[..2];
        // Forge a third share that, with the two, interpolates to secret 2000.
        // (Existence of such a forge is exactly the information-theoretic hiding
        // argument: 2 points leave the constant term free.)
        let target = Fp::new(2000);
        let forged_third = forge_third_share(two, target);
        let mut combined = two.to_vec();
        combined.push(forged_third);
        assert_eq!(
            reconstruct_at_zero(&combined, 2).unwrap(),
            target,
            "2 shares are consistent with a different secret → they hide it"
        );
    }

    /// Given 2 shares on a degree-2 polynomial, produce a 3rd share (at a fresh
    /// x) such that the 3 interpolate to `target` at x=0. Demonstrates the
    /// hiding property; test-only.
    fn forge_third_share(two: &[Share], target: Fp) -> Share {
        // Pick x3 distinct from the two.
        let x3 = (1..=1000u64)
            .find(|x| two.iter().all(|s| s.x != *x))
            .unwrap();
        // We need f(0)=target with f through (x1,y1),(x2,y2),(x3,y3). Solve y3 so
        // the Lagrange-at-0 of the three equals target.
        // target = y1 L1 + y2 L2 + y3 L3  where Li are the Lagrange weights at 0.
        let xs = [two[0].x, two[1].x, x3];
        let weight = |i: usize| -> Fp {
            let xi = Fp::new(xs[i]);
            let mut num = Fp::one();
            let mut den = Fp::one();
            for (j, &xj) in xs.iter().enumerate() {
                if i == j {
                    continue;
                }
                let xj = Fp::new(xj);
                num = num.mul(xj.neg());
                den = den.mul(xi.sub(xj));
            }
            num.mul(den.inv())
        };
        let l1 = weight(0);
        let l2 = weight(1);
        let l3 = weight(2);
        // y3 = (target - y1 L1 - y2 L2) / L3.
        let rhs = target.sub(two[0].y.mul(l1)).sub(two[1].y.mul(l2));
        let y3 = rhs.mul(l3.inv());
        Share { x: x3, y: y3 }
    }

    #[test]
    fn secure_sum_equals_plaintext_sum() {
        // THE M3 differential for run_secure: secret-share four private salaries,
        // run the secure cumulative sum, reconstruct — must equal the plaintext
        // sum. This is the flatmate cumulative-salary aggregate.
        let salaries = [30_000u64, 45_000, 28_000, 51_000];
        let plaintext_sum: u64 = salaries.iter().sum();

        let mut dealer = ShamirBackend::new_seeded(4, 0xABCD).unwrap().dealer();
        let shared: Vec<ShareVec> = salaries.iter().map(|&s| dealer.share(Fp::new(s))).collect();

        let backend = ShamirBackend::new(4).unwrap(); // reconstruction is RNG-free
        let summed = backend.run_secure(&shared).unwrap();
        let out = backend.reconstruct_disclosed(&summed).unwrap();

        // Disclosed partial carries exactly the cumulative integer.
        assert_eq!(out.rows.len(), 1);
        let got = match &out.rows[0][0] {
            Some(oxrdf::Term::Literal(l)) => l.value().parse::<u64>().unwrap(),
            other => panic!("expected integer literal, got {other:?}"),
        };
        assert_eq!(got, plaintext_sum, "secure sum must equal plaintext sum");
    }

    #[test]
    fn secure_sum_zero_and_single() {
        // Edge: a single holder's "sum" is its own value; reconstructs to it.
        let mut dealer = ShamirBackend::new_seeded(3, 5).unwrap().dealer();
        let one = vec![dealer.share(Fp::new(12345))];
        let backend = ShamirBackend::new(3).unwrap();
        let summed = backend.run_secure(&one).unwrap();
        let got = backend.reconstruct(&summed[0]).unwrap();
        assert_eq!(got, Fp::new(12345));
    }

    #[test]
    fn n_too_small_is_a_protocol_error() {
        assert!(matches!(ShamirBackend::new(1), Err(MpcError::Protocol(_))));
    }

    // ---- BGW degree-reduction round (sq-dvuc) [OPUS-4.8] ------------------

    #[test]
    fn degree_reduce_round_trips_single_product() {
        // Acceptance #1: degree_reduce(shares_2t) round-trips — reconstruct after
        // reduction == reconstruct before == plaintext product. Use n=5 (t=2) so
        // the degree-2t open at degree 2t=4 has the full 5 points, AND the reduced
        // degree-t sharing genuinely has t=2 < 5 (real reduction, not a no-op).
        let b = ShamirBackend::new_seeded(5, 0xD7C).unwrap();
        let t = b.threshold();
        assert_eq!(t, 2);
        let mut dealer = b.dealer();
        for (av, bv) in [
            (0u64, 7u64),
            (1, 1),
            (6, 9),
            (123_456, 654_321),
            (crate::field::P - 1, crate::field::P - 1),
            (crate::field::P - 1, 2),
        ] {
            let sa = dealer.share(Fp::new(av));
            let sb = dealer.share(Fp::new(bv));
            let prod_2t = mul_shares_raw(&sa, &sb).unwrap();
            let expected = Fp::new(av).mul(Fp::new(bv));

            // Reconstruct BEFORE reduction at degree 2t.
            let before = reconstruct_degree(&prod_2t, 2 * t).unwrap();
            assert_eq!(before, expected, "degree-2t product must equal a·b");

            // Reduce, then reconstruct AFTER at degree t.
            let reduced = dealer.degree_reduce(&prod_2t).unwrap();
            assert_eq!(reduced.len(), 5);
            let after = b.reconstruct(&reduced).unwrap();
            assert_eq!(after, expected, "reduced degree-t sharing must equal a·b");
            assert_eq!(before, after, "reduction must preserve the secret");
        }
    }

    #[test]
    fn two_multiplication_chain_a_b_c() {
        // Acceptance #2: a TWO-multiplication chain a·b·c — share a,b,c;
        // mul→degree-reduce→mul→degree-reduce; reconstruct; assert == plaintext
        // a·b·c. Several field values incl. edge values (0, 1, large). This is THE
        // load-bearing test: it proves multiplications now CHAIN.
        let b = ShamirBackend::new_seeded(5, 0xC4A1_5EED).unwrap();
        assert_eq!(b.threshold(), 2); // n=5 → t=2; chain stays well within 2t<n
        let mut dealer = b.dealer();
        let cases = [
            (0u64, 5u64, 9u64),          // zero short-circuits
            (1, 1, 1),                   // identities
            (1, 0, 12345),               // zero in the middle
            (2, 3, 4),                   // small
            (7, 11, 13),                 // small primes
            (100_000, 7, 9),             // mixed magnitude
            (crate::field::P - 1, 2, 3), // large × small × small (wraps mod P)
            (
                crate::field::P - 1,
                crate::field::P - 1,
                crate::field::P - 1,
            ), // all large
        ];
        for (av, bv, cv) in cases {
            let (fa, fb, fc) = (Fp::new(av), Fp::new(bv), Fp::new(cv));
            let expected = fa.mul(fb).mul(fc);

            let sa = dealer.share(fa);
            let sb = dealer.share(fb);
            let sc = dealer.share(fc);

            // mul → degree-reduce → mul → degree-reduce.
            let ab_2t = mul_shares_raw(&sa, &sb).unwrap();
            let ab_t = dealer.degree_reduce(&ab_2t).unwrap();
            let abc_2t = mul_shares_raw(&ab_t, &sc).unwrap();
            let abc_t = dealer.degree_reduce(&abc_2t).unwrap();

            let got = b.reconstruct(&abc_t).unwrap();
            assert_eq!(got, expected, "a·b·c chain failed for ({av}, {bv}, {cv})");
        }
    }

    #[test]
    fn degree_reduce_precondition_fails_closed() {
        // Acceptance #3: with n < 2t+1, degree_reduce returns the descriptive
        // error (no panic). We cannot build an honest-majority backend with
        // n < 2t+1 (the constructor fixes t = ⌊(n−1)/2⌋), so we simulate the
        // failure by handing degree_reduce a TRUNCATED share vector (fewer than
        // the 2t+1 = n points it needs) — the fail-closed precondition must catch
        // it with MpcError::Protocol, never a panic / wrong answer.
        let b = ShamirBackend::new_seeded(5, 1).unwrap();
        let t = b.threshold(); // 2 → needs 2t+1 = 5 shares
        let mut dealer = b.dealer();
        let prod = {
            let sa = dealer.share(Fp::new(3));
            let sb = dealer.share(Fp::new(4));
            mul_shares_raw(&sa, &sb).unwrap()
        };
        // Too few points to determine the degree-2t = 4 polynomial.
        let too_few = &prod[..2 * t]; // 4 < 2t+1 = 5
        let err = dealer.degree_reduce(too_few).unwrap_err();
        assert!(
            matches!(err, MpcError::Protocol(_)),
            "n < 2t+1 must be a descriptive protocol error, got {err:?}"
        );
        // A vector that has 2t+1 points but is NOT the full n-party set is also a
        // protocol error (degree_reduce expects the canonical full sharing).
        let mut wrong_n = prod.clone();
        wrong_n.push(Share {
            x: 99,
            y: Fp::new(7),
        });
        let err2 = dealer.degree_reduce(&wrong_n).unwrap_err();
        assert!(matches!(err2, MpcError::Protocol(_)), "got {err2:?}");
    }

    #[test]
    fn reduced_sharing_is_genuinely_degree_t() {
        // Acceptance #4: the reduced sharing is genuinely degree-t — any t+1 of
        // the new shares reconstruct the same secret, and a DIFFERENT t+1 subset
        // agrees. (If the reduction left the sharing at degree 2t, a t+1 subset
        // would interpolate to a WRONG value.) Use n=7 (t=3) so there are several
        // distinct t+1 = 4 subsets to compare and ample headroom over 2t=6.
        let b = ShamirBackend::new_seeded(7, 0x7E57).unwrap();
        let t = b.threshold();
        assert_eq!(t, 3);
        let mut dealer = b.dealer();
        let (av, bv) = (4321u64, 8765u64);
        let expected = Fp::new(av).mul(Fp::new(bv));
        let prod_2t =
            mul_shares_raw(&dealer.share(Fp::new(av)), &dealer.share(Fp::new(bv))).unwrap();
        let reduced = dealer.degree_reduce(&prod_2t).unwrap();
        assert_eq!(reduced.len(), 7);

        // First t+1 shares reconstruct the secret (use reconstruct_at_zero so we
        // interpolate EXACTLY t+1 points at degree t — no robustness fallback).
        let lo = reconstruct_at_zero(&reduced[..t + 1], t).unwrap();
        assert_eq!(lo, expected, "first t+1 reduced shares must give a·b");
        // A DIFFERENT, disjoint-ish t+1 subset agrees → consistent degree-t poly.
        let hi = reconstruct_at_zero(&reduced[reduced.len() - (t + 1)..], t).unwrap();
        assert_eq!(hi, expected, "last t+1 reduced shares must also give a·b");
        assert_eq!(lo, hi, "two t+1 subsets must agree (degree exactly t)");

        // Negative control: the ORIGINAL degree-2t product is NOT degree-t — any
        // t+1 of its shares interpolated AT DEGREE t give the WRONG value (it
        // takes 2t+1 of them). This pins that the reduction actually did work.
        let wrong = reconstruct_at_zero(&prod_2t[..t + 1], t).unwrap();
        assert_ne!(
            wrong, expected,
            "t+1 points of the degree-2t product must NOT give a·b (sanity: reduction was real)"
        );
    }

    #[test]
    fn share_private_input_from_holder_roundtrips() {
        // End-to-end: a holder's PRIVATE salary is shared (not disclosed), summed
        // with another holder's, and reconstructed to the plaintext total.
        const PFX: &str =
            "@prefix ex: <http://ex/> . @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";
        let alice = Holder::from_rdf(
            "alice",
            &format!("{PFX} ex:alice ex:salary \"30000\"^^xsd:integer ."),
            "turtle",
        )
        .unwrap();
        let bob = Holder::from_rdf(
            "bob",
            &format!("{PFX} ex:bob ex:salary \"45000\"^^xsd:integer ."),
            "turtle",
        )
        .unwrap();

        // Production backend (OS-seeded CSPRNG) — the round-trip is seed-agnostic.
        let backend = ShamirBackend::new(2).unwrap();
        let mut sa = backend.share_private_input(&alice).unwrap();
        let sb = backend.share_private_input(&bob).unwrap();
        sa.extend(sb);
        let summed = backend.run_secure(&sa).unwrap();
        let got = backend.reconstruct(&summed[0]).unwrap();
        assert_eq!(got, Fp::new(75_000));
    }
}
