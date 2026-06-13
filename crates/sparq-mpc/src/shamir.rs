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
//! ## Randomness honesty note
//!
//! The masking polynomial coefficients need randomness. This module's
//! [`SeededRng`] is a deterministic SplitMix64 PRNG, used so the in-process
//! multi-party SIMULATION (and its differential tests) are reproducible. **In a
//! real deployment the dealer's coefficients MUST come from a cryptographically
//! secure RNG** — a deterministic PRNG would make shares predictable and break
//! confidentiality. This is flagged, not hidden: see [`SeededRng`]. No security
//! is claimed for the PRNG itself; it stands in for the network/RNG layer that a
//! production backend supplies.

use crate::backend::{BackendInfo, MpcBackend, TrustModel};
use crate::field::Fp;
use crate::holder::Holder;
use crate::partial::{HolderId, MpcError, PartialResult};

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

/// Deterministic SplitMix64 PRNG. **Simulation/testing only** — see the module
/// "Randomness honesty note". A production backend MUST substitute a CSPRNG for
/// the dealer's masking coefficients.
#[derive(Debug, Clone)]
pub struct SeededRng {
    state: u64,
}

impl SeededRng {
    /// New PRNG from a fixed seed (reproducible simulation).
    pub fn new(seed: u64) -> Self {
        SeededRng { state: seed }
    }

    /// Next field element (uniform-ish over `F_p` for simulation; NOT a security
    /// guarantee — see module note).
    fn next_fp(&mut self) -> Fp {
        // SplitMix64.
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        Fp::new(z)
    }
}

/// Honest-majority Shamir `t`-of-`n` secret-sharing backend.
///
/// `n` = number of compute parties (= holders in the flatmate model); the
/// privacy threshold is `t = ⌊(n-1)/2⌋` (honest majority). The backend is the
/// *coordinator-free* description of the scheme; the in-process multi-party
/// simulation that runs the parties is driven by [`MpcBackend::run_secure`] +
/// the helpers below, which is what the differential tests exercise.
#[derive(Debug, Clone)]
pub struct ShamirBackend {
    n: usize,
    t: usize,
    rng: SeededRng,
}

impl ShamirBackend {
    /// Build an honest-majority backend for `n` parties (`n >= 2`). The privacy
    /// threshold is set to the honest-majority maximum `t = ⌊(n-1)/2⌋`: any
    /// `<= t` colluding parties learn nothing; reconstruction needs `>= t+1`.
    ///
    /// `seed` fixes the simulation PRNG (see the module randomness note).
    pub fn new(n: usize, seed: u64) -> Result<Self, MpcError> {
        if n < 2 {
            return Err(MpcError::Protocol(
                "Shamir honest-majority backend needs n >= 2 parties".into(),
            ));
        }
        let t = (n - 1) / 2;
        Ok(ShamirBackend { n, t, rng: SeededRng::new(seed) })
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

    /// Secret-share one field element into `n` shares on a fresh degree-`t`
    /// polynomial `f` with `f(0) = secret`. Free coefficients are random (PRNG
    /// in simulation; CSPRNG in production — see module note).
    pub fn share(&mut self, secret: Fp) -> Vec<Share> {
        // f(z) = secret + c_1 z + ... + c_t z^t.
        let mut coeffs = Vec::with_capacity(self.t + 1);
        coeffs.push(secret);
        for _ in 0..self.t {
            coeffs.push(self.rng.next_fp());
        }
        (1..=self.n as u64)
            .map(|x| Share { x, y: eval_poly(&coeffs, Fp::new(x)) })
            .collect()
    }

    /// Reconstruct the secret `f(0)` from `>= t+1` shares via Lagrange
    /// interpolation. Fewer than `t+1` shares is a protocol error (the whole
    /// point of the threshold). Shares must have distinct `x`.
    pub fn reconstruct(&self, shares: &[Share]) -> Result<Fp, MpcError> {
        reconstruct_at_zero(shares, self.t)
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

/// Lagrange-interpolate the shares' polynomial and evaluate it at `x = 0` (the
/// secret). Requires `>= t+1` distinct points.
fn reconstruct_at_zero(shares: &[Share], t: usize) -> Result<Fp, MpcError> {
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
            Ok(Share { x: sa.x, y: sa.y.add(sb.y) })
        })
        .collect()
}

/// Add a public field constant to a sharing (local, non-interactive). Adding `c`
/// to every share replaces `f(x)` by `f(x) + c`, whose value at `x = 0` is
/// `secret + c`, still a valid degree-`t` sharing.
pub fn add_constant(a: &[Share], c: Fp) -> Vec<Share> {
    a.iter().map(|s| Share { x: s.x, y: s.y.add(c) }).collect()
}

/// Multiply a sharing by a public field constant (local, non-interactive): scale
/// every share. `c * f(x)` interpolates to `c * secret` at `x = 0` and stays
/// degree `t`.
pub fn scale(a: &[Share], c: Fp) -> Vec<Share> {
    a.iter().map(|s| Share { x: s.x, y: s.y.mul(c) }).collect()
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
            Ok(Share { x: sa.x, y: sa.y.sub(sb.y) })
        })
        .collect()
}

/// **Local share-products before degree reduction.** Multiplying two degree-`t`
/// sharings component-wise yields, at each party point, `f(x)·g(x)` — a sharing
/// of `a·b` but on a polynomial of degree `2t`. Reconstructing it therefore
/// needs `2t+1` points (honest-majority gives `2t+1 <= n`). In a full BGW/DN
/// protocol the parties would *re-share and recombine* to bring the degree back
/// to `t` in ONE communication round; for the in-process simulation we keep the
/// degree-`2t` product and reconstruct it directly when the value is opened (the
/// equality test opens its masked product, so no further multiplication chains
/// on the high-degree result — degree reduction is unnecessary for THIS
/// primitive). See [`reconstruct_degree`].
///
/// HONESTY: the round/degree cost is real and stated — one multiplication is one
/// interaction round and consumes the `n >= 2t+1` headroom. Chained
/// multiplications need explicit degree reduction (not implemented; the equality
/// primitive deliberately needs only a single product).
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
            Ok(Share { x: sa.x, y: sa.y.mul(sb.y) })
        })
        .collect()
}

/// Reconstruct the secret of a sharing of a known polynomial `degree`, requiring
/// `degree + 1` points. Used to open the degree-`2t` product of the equality
/// test (`degree = 2t`).
pub fn reconstruct_degree(shares: &[Share], degree: usize) -> Result<Fp, MpcError> {
    // reconstruct_at_zero needs `t+1` points where here the "t" is `degree`.
    reconstruct_at_zero(shares, degree)
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
        BackendInfo {
            name: "shamir-honest-majority",
            trust_model: TrustModel::HonestMajority,
            // Semi-honest only at v1 — see module "Security model".
            malicious_secure: false,
        }
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
        // A fresh sharing requires fresh randomness; clone the RNG state so the
        // backend stays `&self` (the trait is immutable-self by design — the
        // dealer's randomness is conceptually per-input).
        let mut dealer = self.clone();
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
    fn reconstruct_disclosed(&self, result_shares: &[Self::Share]) -> Result<PartialResult, MpcError> {
        if result_shares.len() != 1 {
            return Err(MpcError::Protocol(
                "reconstruct_disclosed: expected exactly one result sharing".into(),
            ));
        }
        let sum = self.reconstruct(&result_shares[0])?;
        Ok(PartialResult {
            holder: HolderId::new("federation"),
            vars: vec![oxrdf::Variable::new_unchecked("cumulative")],
            rows: vec![vec![Some(oxrdf::Term::Literal(oxrdf::Literal::new_typed_literal(
                sum.value().to_string(),
                oxrdf::vocab::xsd::INTEGER,
            )))]],
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
        let mut b = ShamirBackend::new(4, 0xC0FFEE).unwrap();
        for secret in [0u64, 1, 42, 100_000, crate::field::P - 1] {
            let shares = b.share(Fp::new(secret));
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
        let mut b = ShamirBackend::new(4, 1).unwrap();
        let shares = b.share(Fp::new(73));
        let err = b.reconstruct(&shares[..1]).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(_)));
    }

    #[test]
    fn threshold_hides_secret_information_theoretically() {
        // CONFIDENTIALITY witness: with t = 2 (n = 5), any 2 shares are
        // consistent with EVERY candidate secret. Concretely: there exists a
        // valid polynomial through the 2 shares for any chosen f(0). We show the
        // 2 shares do not determine the secret by exhibiting two different
        // secrets whose sharings agree on the same 2 points.
        let mut b = ShamirBackend::new(5, 7).unwrap();
        assert_eq!(b.threshold(), 2);
        let shares_a = b.share(Fp::new(1000));
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
        assert_eq!(reconstruct_at_zero(&combined, 2).unwrap(), target,
            "2 shares are consistent with a different secret → they hide it");
    }

    /// Given 2 shares on a degree-2 polynomial, produce a 3rd share (at a fresh
    /// x) such that the 3 interpolate to `target` at x=0. Demonstrates the
    /// hiding property; test-only.
    fn forge_third_share(two: &[Share], target: Fp) -> Share {
        // Pick x3 distinct from the two.
        let x3 = (1..=1000u64).find(|x| two.iter().all(|s| s.x != *x)).unwrap();
        // We need f(0)=target with f through (x1,y1),(x2,y2),(x3,y3). Solve y3 so
        // the Lagrange-at-0 of the three equals target.
        // target = y1 L1 + y2 L2 + y3 L3  where Li are the Lagrange weights at 0.
        let xs = [two[0].x, two[1].x, x3];
        let weight = |i: usize| -> Fp {
            let xi = Fp::new(xs[i]);
            let mut num = Fp::one();
            let mut den = Fp::one();
            for (j, &xj) in xs.iter().enumerate() {
                if i == j { continue; }
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

        let mut dealer = ShamirBackend::new(4, 0xABCD).unwrap();
        let shared: Vec<ShareVec> = salaries.iter().map(|&s| dealer.share(Fp::new(s))).collect();

        let backend = ShamirBackend::new(4, 0).unwrap(); // reconstruction is RNG-free
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
        let mut dealer = ShamirBackend::new(3, 5).unwrap();
        let one = vec![dealer.share(Fp::new(12345))];
        let backend = ShamirBackend::new(3, 0).unwrap();
        let summed = backend.run_secure(&one).unwrap();
        let got = backend.reconstruct(&summed[0]).unwrap();
        assert_eq!(got, Fp::new(12345));
    }

    #[test]
    fn n_too_small_is_a_protocol_error() {
        assert!(matches!(ShamirBackend::new(1, 0), Err(MpcError::Protocol(_))));
    }

    #[test]
    fn share_private_input_from_holder_roundtrips() {
        // End-to-end: a holder's PRIVATE salary is shared (not disclosed), summed
        // with another holder's, and reconstructed to the plaintext total.
        const PFX: &str = "@prefix ex: <http://ex/> . @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";
        let alice = Holder::from_rdf("alice", &format!("{PFX} ex:alice ex:salary \"30000\"^^xsd:integer ."), "turtle").unwrap();
        let bob = Holder::from_rdf("bob", &format!("{PFX} ex:bob ex:salary \"45000\"^^xsd:integer ."), "turtle").unwrap();

        let backend = ShamirBackend::new(2, 99).unwrap();
        let mut sa = backend.share_private_input(&alice).unwrap();
        let sb = backend.share_private_input(&bob).unwrap();
        sa.extend(sb);
        let summed = backend.run_secure(&sa).unwrap();
        let got = backend.reconstruct(&summed[0]).unwrap();
        assert_eq!(got, Fp::new(75_000));
    }
}
