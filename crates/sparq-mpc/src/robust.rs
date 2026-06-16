// [OPUS-4.8] sq-m34i (MPC WI-1): Reed-Solomon consistency-checked + robust
// (Berlekamp-Welch) reconstruction over Fp. Closes malicious-security gap (D)
// at the Shamir layer for the honest-majority abort/robust regime. Written while
// Fable unavailable — re-review on return.
//! Reed–Solomon error-detection / error-correction for Shamir reconstruction.
//!
//! ## The codeword view (why this works with NO new dependency)
//!
//! A Shamir sharing of a degree-`t` secret polynomial `Q` evaluated at `n`
//! distinct nonzero points `x_1..x_n` is **exactly** an `[n, t+1]` Reed–Solomon
//! codeword over the existing field `F_p` ([`crate::field`]). The message is the
//! `t+1` low coefficients of `Q` (the secret is `Q(0)`); the codeword is the `n`
//! evaluations `y_i = Q(x_i)`. Plain Lagrange interpolation
//! (`shamir::reconstruct_at_zero`, now `#[cfg(test)]` — no production caller after
//! WI-2, so not an intra-doc link) is RS *decoding under the assumption
//! that every symbol is correct* — it has no consistency check, so one tampered
//! `y_i` silently yields the wrong `Q(0)` even when redundancy is present
//! (pinned by `adversarial_tests::tamper`; bead sq-uu0u).
//!
//! With redundancy (`n > t+1`) the same codeword view gives us, for FREE over the
//! same `F_p` (no Feldman/Pedersen DLOG group, no SPDZ MAC preprocessing, no
//! bigint — see sq-uu0u "REJECTED for now"):
//!
//! - **Detect** any tampering, for any number of errors, by checking all `n`
//!   points lie on a single degree-`t` polynomial.
//! - **Correct** up to `e = ⌊(n − t − 1)/2⌋` tampered shares (the RS / BW bound
//!   `n ≥ t + 2e + 1`) and return the TRUE secret — *robust* reconstruction.
//!
//! ## Threat model (honest-majority, information-theoretic)
//!
//! This adds detect-and-abort (and, where the bound allows, robust correction)
//! against ACTIVE tampering of share *values* — guarantee (D) at the Shamir
//! layer. It needs no PKI, broadcast, or preprocessing. It does NOT defend:
//!
//! - **Exactly `t+1` shares (no redundancy).** Any `t+1` arbitrary points define
//!   *some* degree-`t` polynomial, so a single forged share just selects a
//!   different (wrong) secret with no inconsistency to detect. No scheme can
//!   detect this; we preserve the existing behaviour and say so explicitly
//!   ([`reconstruct_robust`] returns the interpolation, never a false
//!   `Tampered`). This is pinned by a test.
//! - **More than the correctable budget with sub-detectable structure.** When
//!   `#errors > e` the BW system can fail to yield a consistent codeword; we
//!   ABORT ([`MpcError::Tampered`]) rather than guess. (RS *miscorrection* — a
//!   second valid codeword within distance `e` — needs `> e` coordinated errors
//!   and is the standard RS limitation, not a regression over plain Lagrange,
//!   which silently miscorrects on a SINGLE error.)
//! - **The degree-`2t` equality/mult open at `n = 2t+1`** (`join::secure_equal`).
//!   WI-2 (bead sq-7q9i) routes that open through THIS primitive at degree `2t`
//!   ([`crate::shamir::reconstruct_degree`]): it detects/corrects when
//!   `n > 2t+1`, but at exactly `n = 2t+1` degree `2t` has NO RS redundancy, so
//!   the same no-redundancy non-detection above applies — a forged product share
//!   is undetectable there. A true fix at `n = 2t+1` needs an information-
//!   theoretic MAC (deferred WI-4, bead sq-6d6g), not RS redundancy.
//!
//! ## Algorithm — Berlekamp–Welch via Gaussian elimination over `F_p`
//!
//! For target error budget `e`, find an **error-locator** `E(x)` of degree `e`
//! (monic: leading coeff 1) and `N(x)` of degree `t + e` such that
//! `N(x_i) = y_i · E(x_i)` for every share `i`, where (by construction)
//! `N(x) = E(x)·Q(x)`. Writing the unknown coefficients of `E` (the `e` below the
//! monic top) and of `N` (`t+e+1` of them) gives a linear system in `t + 2e + 1`
//! unknowns over `n ≥ t + 2e + 1` equations:
//!
//! ```text
//!   N(x_i) − y_i·E(x_i) = 0
//!   ⟺  Σ_{k≤t+e} N_k x_i^k  −  y_i·(Σ_{k<e} E_k x_i^k + x_i^e) = 0
//!   ⟺  Σ_{k≤t+e} N_k x_i^k  −  Σ_{k<e} (y_i x_i^k) E_k  =  y_i x_i^e
//! ```
//!
//! We solve it by Gaussian elimination over `F_p` (uses [`Fp::inv`]/`mul`),
//! recover `Q = N / E` by exact polynomial division (the remainder MUST be zero
//! — if it is not, the candidate is rejected), and return `Q(0)`. Finally we
//! re-verify that *all* `n` original points satisfy the recovered `Q`; only then
//! is the result trusted. `O(n^3)` Gaussian elimination is fine at the intended
//! party counts (`n` = 3..9).

use crate::field::Fp;
use crate::partial::MpcError;
use crate::shamir::Share;

/// Robust / consistency-checked reconstruction of a degree-`degree` sharing's
/// value at `x = 0`, using the Reed–Solomon structure of the shares.
///
/// `degree` is the polynomial degree (`t` for a normal sharing). Let `n` be the
/// number of (distinct-`x`) shares supplied. Behaviour by redundancy:
///
/// - `n < degree + 1` → [`MpcError::Protocol`] (below threshold — cannot
///   reconstruct at all; same precondition as plain Lagrange).
/// - `n == degree + 1` (NO redundancy) → plain Lagrange interpolation. Tampering
///   is **undetectable** here and we do NOT claim otherwise (see module docs):
///   never returns [`MpcError::Tampered`].
/// - `n > degree + 1` (redundancy) → Berlekamp–Welch with the maximal
///   correctable budget `e = ⌊(n − degree − 1)/2⌋`:
///   - `≤ e` tampered shares → corrects them, returns the TRUE secret.
///   - `> e` tampered shares but still inconsistent → [`MpcError::Tampered`].
///   - all consistent → the secret (no behaviour change on honest input).
///
/// The returned secret is `Q(0)` of the recovered/agreed degree-`degree`
/// polynomial. On any inconsistency that cannot be corrected, returns
/// [`MpcError::Tampered`] with the identified cheater points where possible.
pub fn reconstruct_robust(shares: &[Share], degree: usize) -> Result<Fp, MpcError> {
    let n = shares.len();
    if n < degree + 1 {
        return Err(MpcError::Protocol(format!(
            "robust reconstruction needs >= {} shares, got {}",
            degree + 1,
            n
        )));
    }
    // Distinct evaluation points are required for RS decoding (a repeated x is a
    // structural protocol violation, not a value tamper).
    check_distinct_points(shares)?;

    // No redundancy: t+1 points always define SOME degree-t polynomial. There is
    // nothing to cross-check, so we preserve plain-Lagrange behaviour and make NO
    // detection claim (returning Tampered here would be a false positive).
    if n == degree + 1 {
        return lagrange_at_zero(shares);
    }

    // Redundant: maximal Berlekamp–Welch error budget for this (n, degree).
    // n >= degree + 2e + 1  ⟺  e <= (n - degree - 1)/2.
    let e_max = (n - degree - 1) / 2;

    // Try increasing error budgets 0..=e_max. e = 0 is the pure consistency
    // check (interpolate degree+1 points, verify the rest); larger e attempts
    // correction. The first budget that yields a polynomial consistent with all
    // points within that error budget wins.
    for e in 0..=e_max {
        if let Some(q) = berlekamp_welch(shares, degree, e) {
            // Re-verify against ALL points: the recovered Q must disagree with at
            // most e of them; the agreeing majority pins the true codeword.
            let bad = disagreeing_points(shares, &q);
            if bad.len() <= e {
                return Ok(eval_poly(&q, Fp::zero()));
            }
        }
    }

    // No degree-`degree` polynomial fits within the correctable budget: the
    // shares are mutually inconsistent beyond what redundancy can repair. Abort
    // (detect-and-abort), naming the cheaters we can pin against the best fit.
    Err(tampered_error(shares, degree, e_max))
}

/// Identify, against the best-effort interpolation of the first `degree+1`
/// points, which point indices are off-curve — a best-effort cheater list for
/// the abort message. (When correction is impossible this is heuristic: the
/// honest set is unknown, so we report the minority relative to one reference
/// subset. Detection itself is sound; the *attribution* is best-effort.)
fn tampered_error(shares: &[Share], degree: usize, e_max: usize) -> MpcError {
    let n = shares.len();
    // Reference polynomial from the first degree+1 points (an arbitrary subset).
    let suspects = match lagrange_at_zero_poly(&shares[..degree + 1]) {
        Ok(q) => disagreeing_points(shares, &q)
            .into_iter()
            .map(|i| shares[i].x)
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };
    MpcError::Tampered {
        detail: format!(
            "RS consistency check failed: {n} shares of a degree-{degree} secret are not all on \
             one polynomial and the inconsistency exceeds the correctable budget e={e_max} \
             (need n >= degree + 2e + 1 to correct e errors)"
        ),
        cheaters: suspects,
    }
}

/// Indices of shares whose `y` does NOT equal `Q(x)` for the candidate poly `q`.
fn disagreeing_points(shares: &[Share], q: &[Fp]) -> Vec<usize> {
    shares
        .iter()
        .enumerate()
        .filter(|(_, s)| eval_poly(q, Fp::new(s.x)) != s.y)
        .map(|(i, _)| i)
        .collect()
}

/// Reject repeated evaluation points (RS decoding assumes distinct `x_i`).
fn check_distinct_points(shares: &[Share]) -> Result<(), MpcError> {
    for i in 0..shares.len() {
        for j in (i + 1)..shares.len() {
            if shares[i].x == shares[j].x {
                return Err(MpcError::Protocol(format!(
                    "robust reconstruction: repeated evaluation point x={}",
                    shares[i].x
                )));
            }
        }
    }
    Ok(())
}

/// Berlekamp–Welch decode for `n` shares of a degree-`degree` polynomial,
/// targeting exactly `e` correctable errors. Returns the recovered polynomial
/// `Q` (coeffs low→high, length `degree+1`) if the key-equation system is
/// solvable and `E | N` exactly; otherwise `None`.
///
/// Unknowns: `E_0..E_{e-1}` (the `e` non-leading error-locator coeffs; `E_e = 1`
/// is fixed monic) and `N_0..N_{degree+e}` (`degree+e+1` numerator coeffs). The
/// equation per share `i` is `N(x_i) − y_i·E(x_i) = 0`, rearranged so the known
/// `y_i·x_i^e` (from the fixed monic top of `E`) is the RHS.
fn berlekamp_welch(shares: &[Share], degree: usize, e: usize) -> Option<Vec<Fp>> {
    let n = shares.len();
    let num_n = degree + e + 1; // coeffs of N: N_0..N_{degree+e}
    let num_e = e; // unknown coeffs of E: E_0..E_{e-1}
    let cols = num_n + num_e;

    // We need at least as many equations as unknowns. With n >= degree+2e+1,
    // n >= num_n + num_e holds; use exactly the available n rows.
    if n < cols {
        return None;
    }

    // Build the augmented matrix [A | b] with n rows, `cols` unknown columns.
    // Column layout: [N_0..N_{degree+e}, E_0..E_{e-1}].
    let mut aug: Vec<Vec<Fp>> = Vec::with_capacity(n);
    for s in shares {
        let x = Fp::new(s.x);
        let y = s.y;
        let mut row = vec![Fp::zero(); cols + 1];
        // N coefficients: + x^k.
        let mut xpow = Fp::one();
        for col in row.iter_mut().take(num_n) {
            *col = xpow;
            xpow = xpow.mul(x);
        }
        // E coefficients (unknown, k = 0..e-1): − y·x^k.
        let mut xpow = Fp::one();
        for k in 0..num_e {
            row[num_n + k] = y.mul(xpow).neg();
            xpow = xpow.mul(x);
        }
        // RHS: the fixed monic top E_e = 1 contributes y·x^e to N(x_i)=y·E(x_i),
        // i.e. moves to the RHS as + y·x^e.
        // [OPUS-4.8] sq-7ltf: `e` is the PUBLIC error-correction parameter and `x`
        // a PUBLIC evaluation point, so the variable-time exponentiation is sound
        // here (no secret in the exponent or base).
        row[cols] = y.mul(x.pow_vartime(e as u64));
        aug.push(row);
    }

    let sol = solve_linear_system(aug, cols)?;
    let n_coeffs = &sol[..num_n]; // N_0..N_{degree+e}
                                  // Rebuild E with the fixed monic leading coefficient.
    let mut e_coeffs: Vec<Fp> = sol[num_n..].to_vec(); // E_0..E_{e-1}
    e_coeffs.push(Fp::one()); // E_e = 1

    // Q = N / E exactly (remainder must vanish, else this (E,N) is spurious).
    let (q, rem) = poly_divmod(n_coeffs, &e_coeffs)?;
    if !rem.iter().all(|c| *c == Fp::zero()) {
        return None;
    }
    // Q must have degree <= `degree`. Trim and length-check.
    let q = trim_trailing_zeros(q);
    if q.len() > degree + 1 {
        return None;
    }
    let mut q = q;
    q.resize(degree + 1, Fp::zero());
    Some(q)
}

/// Solve a (possibly over-determined) linear system `A·z = b` over `F_p` for
/// exactly `cols` unknowns, given an augmented matrix with `>= cols` rows
/// (last entry of each row is the RHS). Returns the unique solution, or `None`
/// if the system is inconsistent or rank-deficient (no unique solution). Plain
/// Gaussian elimination with partial (nonzero) pivoting; `O(rows · cols^2)`.
fn solve_linear_system(mut aug: Vec<Vec<Fp>>, cols: usize) -> Option<Vec<Fp>> {
    let rows = aug.len();
    let mut pivot_row = 0usize;
    let mut where_pivot = vec![usize::MAX; cols]; // pivot row for each column

    for col in 0..cols {
        if pivot_row >= rows {
            break;
        }
        // Find a row at/below pivot_row with a nonzero entry in this column.
        let sel = (pivot_row..rows).find(|&r| aug[r][col] != Fp::zero());
        let Some(sel) = sel else { continue };
        aug.swap(pivot_row, sel);

        // Normalize the pivot row so the pivot is 1.
        let inv = aug[pivot_row][col].inv();
        for entry in aug[pivot_row].iter_mut().skip(col) {
            *entry = entry.mul(inv);
        }
        // Eliminate this column from every other row. Snapshot the (now-borrowed-
        // immutably) pivot row so we can mutate the other rows in place.
        let pivot_snapshot = aug[pivot_row].clone();
        for (r, row) in aug.iter_mut().enumerate() {
            if r != pivot_row && row[col] != Fp::zero() {
                let factor = row[col];
                for (entry, &p) in row.iter_mut().zip(pivot_snapshot.iter()).skip(col) {
                    *entry = entry.sub(factor.mul(p));
                }
            }
        }
        where_pivot[col] = pivot_row;
        pivot_row += 1;
    }

    // Every unknown must be pivoted (unique solution), else underdetermined.
    if where_pivot.contains(&usize::MAX) {
        return None;
    }
    // Consistency: any all-zero coefficient row must have zero RHS.
    for row in aug.iter() {
        if row[..cols].iter().all(|c| *c == Fp::zero()) && row[cols] != Fp::zero() {
            return None;
        }
    }
    let mut sol = vec![Fp::zero(); cols];
    for (col, &pr) in where_pivot.iter().enumerate() {
        sol[col] = aug[pr][cols];
    }
    Some(sol)
}

/// Polynomial division `num / den` over `F_p` (coeffs low→high). Returns
/// `(quotient, remainder)`. `None` if `den` is the zero polynomial.
fn poly_divmod(num: &[Fp], den: &[Fp]) -> Option<(Vec<Fp>, Vec<Fp>)> {
    let den = trim_trailing_zeros(den.to_vec());
    if den.is_empty() {
        return None; // division by zero polynomial
    }
    let mut rem = trim_trailing_zeros(num.to_vec());
    let den_deg = den.len() - 1;
    let den_lead_inv = den[den_deg].inv();

    if rem.len() < den.len() {
        return Some((vec![Fp::zero()], rem));
    }
    let mut quot = vec![Fp::zero(); rem.len() - den_deg];
    while rem.len() >= den.len() && !rem.is_empty() {
        let rem_deg = rem.len() - 1;
        let shift = rem_deg - den_deg;
        let coeff = rem[rem_deg].mul(den_lead_inv);
        quot[shift] = coeff;
        // rem -= coeff · x^shift · den.
        for (k, &d) in den.iter().enumerate() {
            let idx = shift + k;
            rem[idx] = rem[idx].sub(coeff.mul(d));
        }
        rem = trim_trailing_zeros(rem);
    }
    Some((trim_trailing_zeros(quot), rem))
}

/// Drop trailing zero (high-degree) coefficients; an all-zero poly becomes `[]`.
fn trim_trailing_zeros(mut v: Vec<Fp>) -> Vec<Fp> {
    while v.last() == Some(&Fp::zero()) {
        v.pop();
    }
    v
}

/// Evaluate a polynomial (coeffs low→high) at `x` by Horner's method.
fn eval_poly(coeffs: &[Fp], x: Fp) -> Fp {
    let mut acc = Fp::zero();
    for &c in coeffs.iter().rev() {
        acc = acc.mul(x).add(c);
    }
    acc
}

/// Plain Lagrange interpolation at `x = 0` (the secret), no consistency check.
/// Used on the no-redundancy path where detection is impossible.
fn lagrange_at_zero(shares: &[Share]) -> Result<Fp, MpcError> {
    Ok(eval_poly(&lagrange_at_zero_poly(shares)?, Fp::zero()))
}

/// Full Lagrange interpolation: recover the polynomial (coeffs low→high) through
/// the given points. Used both to evaluate at 0 and to verify other points.
fn lagrange_at_zero_poly(shares: &[Share]) -> Result<Vec<Fp>, MpcError> {
    if shares.is_empty() {
        return Err(MpcError::Protocol("interpolation needs >= 1 point".into()));
    }
    // Build P(x) = Σ_i y_i · Π_{j≠i} (x − x_j)/(x_i − x_j) by accumulating the
    // weighted basis polynomials. O(n^2) over small n.
    let n = shares.len();
    let mut result = vec![Fp::zero(); n];
    for (i, si) in shares.iter().enumerate() {
        let xi = Fp::new(si.x);
        // Numerator polynomial Π_{j≠i} (x − x_j) and the scalar denominator.
        let mut basis = vec![Fp::one()]; // start with the constant poly 1
        let mut den = Fp::one();
        for (j, sj) in shares.iter().enumerate() {
            if i == j {
                continue;
            }
            let xj = Fp::new(sj.x);
            basis = poly_mul_linear(&basis, xj.neg()); // multiply by (x − x_j)
            den = den.mul(xi.sub(xj));
        }
        let scale = si.y.mul(den.inv());
        for (k, b) in basis.iter().enumerate() {
            result[k] = result[k].add(b.mul(scale));
        }
    }
    Ok(result)
}

/// Multiply a polynomial (coeffs low→high) by the linear factor `(x + c)`.
fn poly_mul_linear(poly: &[Fp], c: Fp) -> Vec<Fp> {
    let mut out = vec![Fp::zero(); poly.len() + 1];
    for (k, &p) in poly.iter().enumerate() {
        out[k] = out[k].add(p.mul(c)); // p · c  → x^k term
        out[k + 1] = out[k + 1].add(p); // p · x  → x^{k+1} term
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shamir::ShamirBackend;

    fn fp(v: u64) -> Fp {
        Fp::new(v)
    }

    fn corrupt(shares: &mut [Share], idx: usize, delta: u64) {
        shares[idx].y = shares[idx].y.add(fp(delta));
    }

    #[test]
    fn untampered_full_set_reconstructs() {
        // [OPUS-4.8] Genuine differential vs the UNCHECKED Lagrange primitive on
        // clean inputs: no regression. We must NOT compare against
        // `ShamirBackend::reconstruct`, which now routes through
        // `reconstruct_robust` itself (robust-vs-robust is vacuous). Compare the
        // robust path against `reconstruct_at_zero` (the plain Lagrange-at-0
        // building block, `pub(crate)` for exactly this suite) so the assertion
        // genuinely pins robust == honest-Lagrange with no errors present.
        let b = ShamirBackend::new_seeded(7, 0xC0FFEE).unwrap();
        let t = b.threshold();
        for secret in [0u64, 1, 42, 100_000, crate::field::P - 1] {
            let shares = b.dealer().share(fp(secret));
            let robust = reconstruct_robust(&shares, t).unwrap();
            let honest = crate::shamir::reconstruct_at_zero(&shares, t).unwrap();
            assert_eq!(
                robust, honest,
                "robust must match unchecked Lagrange on clean input"
            );
            assert_eq!(robust, fp(secret));
        }
    }

    #[test]
    fn detects_single_tamper_with_full_redundancy_when_e_zero_not_enough() {
        // n=4, t=1 → e_max = floor((4-1-1)/2) = 1: a single error is CORRECTABLE.
        // n=3, t=1 → e_max = 0: a single error is only DETECTABLE (abort).
        let b = ShamirBackend::new_seeded(3, 1).unwrap(); // n=3, t=1
        let t = b.threshold();
        let mut shares = b.dealer().share(fp(2024));
        // Honest robust reconstruct is fine.
        assert_eq!(reconstruct_robust(&shares, t).unwrap(), fp(2024));
        // Tamper one share: redundancy present (n=3 > t+1=2) but e_max=0 ⇒ abort.
        corrupt(&mut shares, 1, 5);
        let err = reconstruct_robust(&shares, t).unwrap_err();
        assert!(matches!(err, MpcError::Tampered { .. }), "got {err:?}");
    }

    #[test]
    fn corrects_single_tamper_when_budget_allows() {
        // n=4, t=1 → e_max = floor((4-1-1)/2) = 1: correct ONE error, return truth.
        let b = ShamirBackend::new_seeded(4, 7).unwrap();
        let t = b.threshold();
        assert_eq!(t, 1);
        let secret = fp(98_765);
        let mut shares = b.dealer().share(secret);
        corrupt(&mut shares, 2, 42); // one tampered share, within budget
        let got = reconstruct_robust(&shares, t).unwrap();
        assert_eq!(
            got, secret,
            "BW must correct one error and return the truth"
        );
    }

    #[test]
    fn corrects_two_tampers_at_n9() {
        // [OPUS-4.8] The backend fixes t = floor((n-1)/2), so for n=9, t=4 (NOT
        // t=2). With t=4 the BW correction budget is e_max = floor((n-t-1)/2) =
        // floor((9-4-1)/2) = 2: this test corrects exactly e_max = 2 errors and
        // then asserts that a 3rd error (over budget) aborts.
        let b = ShamirBackend::new_seeded(9, 0x9999).unwrap();
        let t = b.threshold();
        assert_eq!(t, 4); // floor((9-1)/2) = 4
                          // t=4 ⇒ e_max = floor((9-4-1)/2) = 2: correct up to TWO errors.
        let secret = fp(555_555);
        let mut shares = b.dealer().share(secret);
        corrupt(&mut shares, 0, 11);
        corrupt(&mut shares, 5, 22);
        let got = reconstruct_robust(&shares, t).unwrap();
        assert_eq!(got, secret, "BW must correct two errors at n=9,t=4");
        // A THIRD error exceeds e_max=2 ⇒ abort.
        let mut three = b.dealer().share(secret);
        corrupt(&mut three, 0, 11);
        corrupt(&mut three, 4, 22);
        corrupt(&mut three, 7, 33);
        let err = reconstruct_robust(&three, t).unwrap_err();
        assert!(matches!(err, MpcError::Tampered { .. }), "got {err:?}");
    }

    #[test]
    fn no_redundancy_does_not_claim_detection() {
        // Exactly t+1 shares: tampering selects a different valid secret, NOT a
        // detectable inconsistency. Must NOT return Tampered (no false positive).
        let b = ShamirBackend::new_seeded(5, 0xABCD).unwrap();
        let t = b.threshold(); // 2
        let secret = fp(123_456);
        let shares = b.dealer().share(secret);
        let mut min_set = shares[..t + 1].to_vec();
        corrupt(&mut min_set, 0, 9_999);
        let got = reconstruct_robust(&min_set, t).expect("t+1 points always interpolate");
        assert_ne!(got, secret, "t+1: tamper changes the secret (no detection)");
    }
}
