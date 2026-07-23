// [OPUS-4.8] sq-6fv7 (sq-ka8m residual; design Hole 4): route the THREE disclose
// path opens — the square-protocol `a²`, the masked open `c = sum + r`, and the
// boolean `verdict` — through the §2.5 batched IT-MAC check, the malicious-secure
// twin of `compare::disclose_threshold_verdict`.
//! Honest-majority **malicious-with-abort** secure threshold disclosure over an
//! EXISTING secret-shared sum — the IT-MAC upgrade of the semi-honest
//! [`crate::compare::disclose_threshold_verdict`] (Hole 4, the federation £100k path).
//!
//! ## What this closes (sq-ka8m residual)
//!
//! sq-ka8m's [`crate::auth_compare`] made the *cleartext fresh-operand* comparison
//! malicious-secure: it bit-shares the operands as authenticated values and
//! MAC-checks the verdict. But the FEDERATION path —
//! [`crate::compare::disclose_threshold_verdict`], which runs over an EXISTING
//! degree-`t` sharing `[sum]` (e.g. the cumulative aggregate from
//! [`crate::backend::MpcBackend::run_secure`]) — uses a *masked-open*
//! bit-decomposition (Damgård et al. TCC'06), not cleartext bit-sharing. That path
//! has **three distinct opens**, every one of them semi-honest (no integrity check)
//! on `origin/main`:
//!
//! 1. the **square-protocol `c = a²`** opened once per mask bit
//!    ([`crate::compare`] `square_protocol_random_bit`) — a forged open share /
//!    wrong re-sharing flips a mask bit;
//! 2. the **masked open `c = sum + r`** ([`crate::compare`] `secure_bit_decompose`)
//!    — the single load-bearing disclosure whose bits drive the whole comparison;
//! 3. the **boolean `verdict`** ([`crate::compare::open_verdict`]).
//!
//! Each is exactly the Hole-1/Hole-2 deviation the IT-MAC defends against: a
//! deviating party re-shares a consistent degree-`t` codeword of the *wrong* value,
//! undetectable by Reed–Solomon even at over-provisioned `n`. This module carries an
//! IT-MAC through all three and MAC-checks each opened value **before it is acted
//! on**, so a tamper aborts fail-closed at the minimal `n = 2t+1` (soundness from the
//! secret `α`, not RS redundancy — design §2.5).
//!
//! ## The construction
//!
//! - **Authenticate the existing sum.** `[[sum]] = ([sum], [α·sum])` via
//!   [`crate::shamir::MacSession::authenticate_existing`] — one BGW mult-then-reduce
//!   over `[α]` and `[sum]`, never reconstructing the sum.
//! - **Authenticated square-protocol mask bits.** Each mask bit `[[b]]` is produced
//!   from a fresh authenticated `[[a]]`: `[[a²]] = auth_mul([[a]],[[a]])`,
//!   **MAC-check `[[a²]]`**, open `c = a²`, then `[[b]] = (d⁻¹·[[a]] + 1)·2⁻¹` as
//!   FREE authenticated affine maps — so the bit stays MAC-carried into the
//!   subtraction circuit. `c = a²` is independent of the sign of `a`, so the open
//!   reveals nothing about the bit; the MAC-check catches a tampered open.
//! - **Authenticated masked open.** `[[r]] = Σ_k [[b_k]]·2^k` (free),
//!   `[[sum+r]] = auth_add([[sum]],[[r]])`, **MAC-check `[[sum+r]]`**, then open
//!   `c = sum + r`. The mask `r` is jointly generated (no party knows it), so `c`
//!   statistically hides `sum` exactly as the semi-honest path's open does.
//! - **Authenticated bitwise subtraction + comparison.** `[[a_k]] = c_k ⊖ [[r_k]]`
//!   over the PUBLIC bits of `c` and the AUTHENTICATED mask bits, with an
//!   authenticated ripple borrow (every AND an [`crate::shamir::MacSession::auth_mul`]);
//!   then the MSB-first comparator against the public threshold over `[[a_k]]`.
//! - **MAC-check the verdict, then open.** The verdict is the third and final open;
//!   the MAC-check over it (and the chain values it depends on) fires on any tamper
//!   in the subtraction or comparison gates.
//!
//! ## Security tier (stated precisely, no over-claim)
//!
//! **Honest-majority, malicious-with-abort, unanimous abort** (design §3), matching
//! [`crate::auth_compare`]: AXIS-1 `Malicious`, AXIS-2 `Abort(Unanimous)`, AXIS-3
//! `HonestMajority`. The exact sum is NEVER opened — only `c = a²` (sign-independent),
//! the statistically-masked `c = sum + r`, and the 1-bit verdict, and the latter two
//! only after their MAC-checks pass.
//!
//! ## The range-proof zero-tests are MAC-checked too (sq-m4zi / sq-e7ma)
//!
//! The in-protocol **range proof** (`sum ∈ [0, 2^`[`crate::compare::DECOMP_VALUE_BITS`]`)`,
//! sq-nx0s) — clause (1) recompose `sum == Σ b_k·2^k` and clause (2) high-part
//! `Σ_{k≥value_bits} b_k·2^k == 0`, each a secret zero-test of a `v·r` masked product
//! — was, on `sq-6fv7`, still reused VERBATIM from the SEMI-HONEST
//! `crate::compare::verify_sum_in_range`: a tampered open of either `v·r` product
//! (a wrong re-sharing producing a consistent codeword of the wrong value) could flip
//! "was it zero?" and let an out-of-range / field-wrapping sum masquerade as in-range,
//! defeating the fail-closed guard. `auth_verify_sum_in_range` closes that residual:
//! the range proof now runs over the AUTHENTICATED sum and bits, the recompose-diff and
//! high-part are FREE authenticated linear combinations, and each zero-test multiplies
//! by a fresh nonzero AUTHENTICATED mask via [`crate::shamir::MacSession::auth_mul`] and
//! **MAC-checks the product `[[v·r]]` before its open is read** — so a tampered zero-test
//! open aborts with [`MpcError::MacCheckFailed`] like the other three.
//!
//! ## Residual (honest scope statement)
//!
//! The masked open statistically hides `sum` (mask gap `κ = `
//! [`crate::compare::DECOMP_STAT_SECURITY_BITS`], the `p = 2^61−1` field is too small
//! for a perfect mask above the value width) — unchanged from the semi-honest path. The
//! dishonest-majority SPDZ regime (route (b) / sq-j5ok) is NOT entered. `[OPUS-4.8]`

use crate::authenticated::{auth_add, auth_add_constant, auth_scale, auth_sub, AuthenticatedShare};
use crate::compare::{
    check_party_count, DECOMP_MASK_BITS, DECOMP_VALUE_BITS, DECOMP_VALUE_MAX_EXCLUSIVE,
};
use crate::backend::SecurityDescriptor;
use crate::field::Fp;
use crate::partial::{HolderId, MpcError, PartialResult};
use crate::shamir::{MacSession, ShamirBackend, Share};

/// The typed three-axis [`SecurityDescriptor`] of this module's protocol at the
/// backend's `(n, t)` — identical to [`crate::auth_compare::security_descriptor`]
/// by the module contract above ("matching `crate::auth_compare`": AXIS-1
/// `Malicious`, AXIS-2 `Abort(Unanimous)`, AXIS-3 honest-majority), so it
/// delegates rather than restating the tier. This is the implementation-owned
/// source the `secprop-annotations` graph's adversary-drift test pins the RDF
/// annotations to.
pub fn security_descriptor(backend: &ShamirBackend) -> SecurityDescriptor {
    crate::auth_compare::security_descriptor(backend)
}

/// **Malicious-secure threshold disclosure over an existing secret-shared sum** —
/// the IT-MAC twin of [`crate::compare::disclose_threshold_verdict`]. Returns a
/// [`PartialResult`] carrying ONLY the boolean `sum > public_threshold`, never the
/// exact sum, with all three decomposition opens MAC-checked before they are acted
/// on.
///
/// `sum_shares` is the degree-`t` sharing of the cumulative aggregate;
/// `public_threshold` is the public bar (e.g. £100k). `public_threshold` must be
/// `< 2^`[`DECOMP_VALUE_BITS`] (fail-closed up front); the SUM is range-checked
/// in-protocol after its bit-decomposition by the MAC-checked sq-nx0s range proof
/// (`auth_verify_sum_in_range`, whose zero-test `v·r` opens are MAC-checked too —
/// sq-m4zi/sq-e7ma). On any tampered open the function ABORTS with
/// [`MpcError::MacCheckFailed`] (or the fail-closed non-boolean-verdict guard) rather
/// than returning a wrong verdict.
///
/// Honest-majority, **malicious-with-abort** (design §3). `[OPUS-4.8]`
pub fn malicious_disclose_threshold_verdict(
    backend: &ShamirBackend,
    sum_shares: &[Share],
    public_threshold: u64,
) -> Result<PartialResult, MpcError> {
    let verdict = malicious_threshold_over_sum(backend, sum_shares, public_threshold)?;
    Ok(PartialResult {
        holder: HolderId::new("federation"),
        vars: vec![oxrdf::Variable::new_unchecked("over_threshold")],
        rows: vec![vec![Some(oxrdf::Term::Literal(
            oxrdf::Literal::new_typed_literal(verdict.to_string(), oxrdf::vocab::xsd::BOOLEAN),
        ))]],
    })
}

/// The boolean core of [`malicious_disclose_threshold_verdict`]: run the full
/// MAC-checked decompose + range-proof + compare chain over the existing `[sum]`
/// and return `sum > public_threshold`. Separated so tests can assert the boolean
/// directly (the `PartialResult` wrapper is the only difference from the public fn).
fn malicious_threshold_over_sum(
    backend: &ShamirBackend,
    sum_shares: &[Share],
    public_threshold: u64,
) -> Result<bool, MpcError> {
    check_party_count(backend.parties(), backend.threshold())?;
    // Fail closed on the public threshold BEFORE any protocol work (same bound as
    // the semi-honest path — the in-MPC masked decomposition's statistically-safe
    // no-wrap magnitude).
    if public_threshold >= DECOMP_VALUE_MAX_EXCLUSIVE {
        return Err(MpcError::Protocol(format!(
            "malicious_disclose_threshold_verdict: public_threshold = {public_threshold} is out of \
             range (must be < 2^{DECOMP_VALUE_BITS} = {DECOMP_VALUE_MAX_EXCLUSIVE}; the in-MPC \
             bit-decomposition masks the secret sum and opens only `sum + r`)"
        )));
    }

    let mut dealer = backend.dealer();
    let mut session = dealer.new_mac_session();
    let key = session.mac_key();

    // 1. Authenticate the EXISTING sum sharing (no reconstruction).
    let auth_sum = session.authenticate_existing(sum_shares)?;

    // 2. Authenticated masked open: recover the sum's bits via the MAC-checked
    //    `c = sum + r` open. `auth_r_bits[k]` is `[[r_k]]`; the recovered
    //    `auth_sum_bits[k]` is `[[a_k]]` (the k-th bit of the sum), MAC-carried.
    let (auth_sum_bits, _auth_r_bits) =
        auth_masked_bit_decompose(&mut session, backend, &auth_sum, &key)?;

    // 3. IN-PROTOCOL range proof of the secret-shared sum, MAC-CHECKED end to end
    //    (sq-m4zi/sq-e7ma): the two zero-test `v·r` opens are now routed through the
    //    §2.5 IT-MAC check over the AUTHENTICATED sum and bits, so a tampered zero-test
    //    open — which would otherwise let an out-of-range sum masquerade as in-range,
    //    bypassing the fail-closed guard — aborts with `MpcError::MacCheckFailed`
    //    rather than letting the verdict be derived from a corrupted decomposition.
    auth_verify_sum_in_range(&mut session, backend, &auth_sum, &auth_sum_bits)?;

    // 4. Authenticated MSB-first comparison of the recovered sum bits against the
    //    PUBLIC threshold; the verdict is MAC-carried.
    let verdict =
        auth_greater_than_public_bits(&mut session, &auth_sum_bits, public_threshold, &key)?;

    // 5. MAC-CHECK the verdict BEFORE opening it (the third and final open).
    session.mac_check(std::slice::from_ref(&verdict))?;

    // 6. Open ONLY the (MAC-verified) verdict bit.
    open_auth_verdict(backend, &verdict)
}

/// Authenticated masked-open bit-decomposition of `[[sum]]` into `L =
/// `[`DECOMP_MASK_BITS`] AUTHENTICATED bits (LSB-first), the malicious-secure twin of
/// [`crate::compare`] `secure_bit_decompose`. Returns `(sum_bits, r_bits)` — both as
/// authenticated sharings.
///
/// The masked open `c = sum + r` is the SECOND of the three named opens; it is
/// MAC-checked before `c` is read. The mask bits come from the AUTHENTICATED square
/// protocol (whose `a²` opens are MAC-checked individually). The bitwise subtraction
/// `c ⊖ [[r]]` runs over the public `c` bits and the authenticated `[[r_k]]`, each
/// AND an [`MacSession::auth_mul`], so the recovered sum bits stay MAC-carried.
fn auth_masked_bit_decompose(
    session: &mut MacSession,
    backend: &ShamirBackend,
    auth_sum: &AuthenticatedShare,
    key: &crate::authenticated::MacKey,
) -> Result<(Vec<AuthenticatedShare>, Vec<AuthenticatedShare>), MpcError> {
    // 1. Authenticated solved-bits: each [[r_k]] a square-protocol bit (MAC-checked
    //    a²), and [[r]] = Σ [[r_k]]·2^k as a FREE authenticated linear combination.
    let mut r_bits: Vec<AuthenticatedShare> = Vec::with_capacity(DECOMP_MASK_BITS);
    let mut auth_r = session.auth_const_sharing(Fp::zero());
    for k in 0..DECOMP_MASK_BITS {
        let bit = auth_square_protocol_random_bit(session, backend, key)?;
        auth_r = auth_add(&auth_r, &auth_scale(&bit, Fp::new(1u64 << k)))?;
        r_bits.push(bit);
    }

    // 2. Masked open c = sum + r, MAC-CHECKED before the public c is read.
    let auth_c = auth_add(auth_sum, &auth_r)?;
    session.mac_check(std::slice::from_ref(&auth_c))?;
    let c = backend.reconstruct(auth_c.value_shares())?.value();

    // 3. Authenticated bitwise subtraction [[a_k]] = c_k ⊖ [[r_k]] with an
    //    authenticated ripple borrow. a_k = c_k XOR r_k XOR borrow;
    //    borrow_out = (¬c_k ∧ r_k) ∨ (borrow ∧ ¬(c_k XOR r_k)).
    let mut out: Vec<AuthenticatedShare> = Vec::with_capacity(DECOMP_MASK_BITS);
    let mut borrow = session.auth_const_sharing(Fp::zero());
    #[allow(clippy::needless_range_loop)]
    for k in 0..DECOMP_MASK_BITS {
        let c_k = (c >> k) & 1;
        let r_k = &r_bits[k];
        // x = c_k XOR r_k (public XOR authenticated, FREE affine): c_k=1 → 1−[[r_k]].
        let x = if c_k == 1 {
            auth_add_constant(&auth_scale(r_k, Fp::one().neg()), Fp::one(), key)?
        } else {
            r_k.clone()
        };
        // a_k = x XOR borrow = x + borrow − 2·(x∧borrow) (one auth_mul).
        let x_and_borrow = session.auth_mul(&x, &borrow)?;
        let a_k = auth_sub(
            &auth_add(&x, &borrow)?,
            &auth_scale(&x_and_borrow, Fp::new(2)),
        )?;
        // borrow_out = (¬c_k ∧ r_k) ∨ (borrow ∧ ¬x).
        //   ¬c_k ∧ r_k : c_k public → c_k=1 ⇒ [[0]]; c_k=0 ⇒ [[r_k]].
        let nc_and_r = if c_k == 1 {
            session.auth_const_sharing(Fp::zero())
        } else {
            r_k.clone()
        };
        //   ¬x = 1 − x (free); borrow ∧ ¬x : one auth_mul; OR : one auth_mul.
        let not_x = auth_add_constant(&auth_scale(&x, Fp::one().neg()), Fp::one(), key)?;
        let borrow_and_notx = session.auth_mul(&borrow, &not_x)?;
        borrow = auth_or(session, &nc_and_r, &borrow_and_notx)?;
        out.push(a_k);
    }
    Ok((out, r_bits))
}

/// [OPUS-4.8] sq-m4zi/sq-e7ma — **MAC-checked in-protocol range proof of the
/// secret-shared sum**, the malicious-with-abort twin of the semi-honest
/// `crate::compare::verify_sum_in_range` (sq-nx0s; that masked-open reference is
/// now test-only after the sq-bgsn Rabbit lift on the semi-honest production path).
/// Given the AUTHENTICATED sum
/// `[[sum]]` and the `L = `[`DECOMP_MASK_BITS`] AUTHENTICATED bits `[[b_0..b_{L-1}]]`
/// that [`auth_masked_bit_decompose`] recovered, it PROVES — without reconstructing the
/// sum — that `sum ∈ [0, 2^`[`DECOMP_VALUE_BITS`]`)` via the SAME two secret zero-tests
/// the semi-honest proof runs, but now each zero-test's `v·r` open is MAC-checked:
///
/// 1. **recompose clause** (no field wrap / fits `L` bits): `sum − Σ_{k<L} b_k·2^k == 0`;
/// 2. **magnitude clause** (below the supported width): the high part
///    `Σ_{k≥`[`DECOMP_VALUE_BITS`]`} b_k·2^k == 0`.
///
/// Both `[[recompose_diff]]` and `[[high_part]]` are FREE authenticated linear
/// combinations of `[[sum]]` and the `[[b_k]]` (no `auth_mul`); each is then fed to the
/// MAC-checked zero-test [`auth_secret_is_zero`]. On violation it returns a fail-closed
/// [`MpcError::Protocol`] (an out-of-range / wrapping sum), and on a TAMPERED zero-test
/// open [`MpcError::MacCheckFailed`] — so a deviating party can no longer flip
/// "was it zero?" to smuggle an out-of-range sum past the guard. Honest-majority,
/// malicious-with-abort. `[OPUS-4.8]`
fn auth_verify_sum_in_range(
    session: &mut MacSession,
    backend: &ShamirBackend,
    auth_sum: &AuthenticatedShare,
    auth_sum_bits: &[AuthenticatedShare],
) -> Result<(), MpcError> {
    // Recompose [[Σ b_k·2^k]] over ALL L bits and SEPARATELY the high part (bits ≥
    // DECOMP_VALUE_BITS). Both are FREE authenticated linear combinations of the bit
    // sharings — no auth_mul — so the recomposition stays MAC-carried.
    let mut recomposed = session.auth_const_sharing(Fp::zero());
    let mut high_part = session.auth_const_sharing(Fp::zero());
    for (k, bit) in auth_sum_bits.iter().enumerate() {
        let weighted = auth_scale(bit, Fp::new(1u64 << k));
        recomposed = auth_add(&recomposed, &weighted)?;
        if k >= DECOMP_VALUE_BITS {
            high_part = auth_add(&high_part, &weighted)?;
        }
    }

    // Clause (1): no field wrap / fits L bits ⇔ [[sum − Σ b_k 2^k]] == 0.
    let recompose_diff = auth_sub(auth_sum, &recomposed)?;
    if !auth_secret_is_zero(session, backend, &recompose_diff)? {
        return Err(MpcError::Protocol(format!(
            "malicious_disclose_threshold_verdict: in-protocol range proof FAILED — the \
             secret-shared sum does not equal the bit-composition of its recovered \
             {DECOMP_MASK_BITS} bits (the masked open `sum + r` wrapped the field modulus \
             p = 2^61-1, or the sum has content above bit {DECOMP_MASK_BITS}). The verdict would \
             be derived from a corrupted decomposition, so it is REJECTED fail-closed rather than \
             returned wrong. The sum must be < 2^{DECOMP_VALUE_BITS} = {DECOMP_VALUE_MAX_EXCLUSIVE}."
        )));
    }

    // Clause (2): below the supported magnitude ⇔ all bits ≥ DECOMP_VALUE_BITS are
    // zero ⇔ [[high_part]] == 0.
    if !auth_secret_is_zero(session, backend, &high_part)? {
        return Err(MpcError::Protocol(format!(
            "malicious_disclose_threshold_verdict: in-protocol range proof FAILED — the \
             secret-shared sum is >= 2^{DECOMP_VALUE_BITS} = {DECOMP_VALUE_MAX_EXCLUSIVE} (a bit \
             at or above position {DECOMP_VALUE_BITS} is set). That exceeds the statistically-safe \
             magnitude the in-MPC masked bit-decomposition supports, so the verdict is REJECTED \
             fail-closed rather than returned wrong."
        )));
    }
    Ok(())
}

/// [OPUS-4.8] sq-m4zi/sq-e7ma — **MAC-checked secret zero-test**: returns `true` iff the
/// AUTHENTICATED `[[v]]` reconstructs to `0`, WITHOUT reconstructing `v`, the
/// malicious-with-abort twin of [`crate::compare`] `secret_is_zero` (sq-nx0s). Draw a
/// fresh NONZERO authenticated mask `[[r]]`, form `[[m]] = [[v]]·[[r]]` via
/// [`MacSession::auth_mul`] (which carries the sound MAC `[α·m]` — design §2.4), then
/// **MAC-check `[[m]]` BEFORE its value open is read** ([`MacSession::mac_check`], §2.5),
/// and test `m == 0`.
///
/// Soundness of the disclosure (unchanged from the semi-honest twin): if `v == 0` then
/// `m == 0` regardless of `r`; if `v != 0` then `v·r` is a UNIFORM NONZERO field element
/// (nonzero times uniform nonzero), so the open of `m` reveals ONLY the single bit
/// "was `v` zero?" — nothing about `v`'s magnitude. The nonzero mask is load-bearing: a
/// zero mask would open `m = 0` even for `v != 0` (a false "in-range"). The MAC-check is
/// the new integrity layer: a tampered open of `m` (a wrong re-sharing producing a
/// consistent degree-`t` codeword of a DIFFERENT value, undetectable by Reed–Solomon at
/// `n = 2t+1`) makes `σ ≠ 0` and aborts with [`MpcError::MacCheckFailed`], so a deviating
/// party cannot flip the zero-test result. Honest-majority, malicious-with-abort.
/// `[OPUS-4.8]`
fn auth_secret_is_zero(
    session: &mut MacSession,
    backend: &ShamirBackend,
    v: &AuthenticatedShare,
) -> Result<bool, MpcError> {
    // Fresh NONZERO authenticated mask [[r]]. Nonzero is load-bearing: m = v·0 = 0
    // would falsely report v == 0. `authenticated_share` mints [[r]] = ([r], [α·r]).
    let mask_value = session.draw_nonzero_fp();
    let auth_mask = session.authenticated_share(mask_value);
    // [[m]] = [[v]]·[[r]] — a MAC-carrying multiplication (degree-t value product with
    // the SOUND independent-reduce MAC, design §2.4): a tampered value reduce makes
    // σ ≠ 0 at the check below.
    let auth_m = session.auth_mul(v, &auth_mask)?;
    // MAC-CHECK [[m]] BEFORE its value is opened — the §2.5 batched check is the sole
    // detector at the minimal n = 2t+1 (soundness ≈ 1 − 2^-61 from the secret α).
    session.mac_check(std::slice::from_ref(&auth_m))?;
    // Open the (MAC-verified) product and test m == 0 ⇔ v == 0. v itself is never
    // reconstructed; m is a uniform nonzero when v != 0 (leaks only "was it zero?").
    let m = backend.reconstruct(auth_m.value_shares())?;
    Ok(m == Fp::zero())
}

/// `[[a ∨ b]]` for authenticated 0/1 sharings: `a + b − a·b`. One [`MacSession::auth_mul`]
/// (the `a·b` term); the rest is FREE authenticated linear ops.
fn auth_or(
    session: &mut MacSession,
    a: &AuthenticatedShare,
    b: &AuthenticatedShare,
) -> Result<AuthenticatedShare, MpcError> {
    let ab = session.auth_mul(a, b)?;
    auth_sub(&auth_add(a, b)?, &ab)
}

/// A single AUTHENTICATED uniform random bit `[[b]] ∈ {0,1}` from the square protocol,
/// the malicious-secure twin of [`crate::compare`] `square_protocol_random_bit`. The
/// `c = a²` open (the FIRST of the three named opens) is **MAC-checked before it is
/// read**: a tampered open of `a²` makes `σ ≠ 0` and aborts, rather than silently
/// flipping the mask bit.
///
/// `[[a]]` is a fresh nonzero authenticated value; `[[a²]] = auth_mul([[a]],[[a]])`;
/// MAC-check `[[a²]]`; open `c = a²` (sign-independent, leaks nothing about the bit);
/// then `[[b]] = (d⁻¹·[[a]] + 1)·2⁻¹` as FREE authenticated affine maps so the bit
/// stays MAC-carried. `[OPUS-4.8]`
fn auth_square_protocol_random_bit(
    session: &mut MacSession,
    backend: &ShamirBackend,
    key: &crate::authenticated::MacKey,
) -> Result<AuthenticatedShare, MpcError> {
    for _attempt in 0..64 {
        // 1. Fresh nonzero authenticated [[a]] — no party knows a; never opened.
        let a_value = session.draw_nonzero_fp();
        let auth_a = session.authenticated_share(a_value);
        // 2. [[a²]] via one auth_mul, then MAC-CHECK before opening c = a².
        let auth_a_sq = session.auth_mul(&auth_a, &auth_a)?;
        session.mac_check(std::slice::from_ref(&auth_a_sq))?;
        let c = backend.reconstruct(auth_a_sq.value_shares())?;
        // 3. c == 0 (a was 0; Pr 1/p) ⇒ retry with fresh [[a]].
        if c == Fp::zero() {
            continue;
        }
        // 4. Public square root d of c, re-checked fail-closed.
        let d = c.sqrt_residue();
        if d.mul(d) != c {
            return Err(MpcError::Protocol(format!(
                "malicious square-protocol random bit: c = {} is not a quadratic residue \
                 (sqrt d = {}, d² = {} ≠ c) — the MAC-checked open of a² is inconsistent; \
                 refusing to emit a bit",
                c.value(),
                d.value(),
                d.mul(d).value()
            )));
        }
        // 5. [[s]] = d⁻¹·[[a]] ∈ {+1,−1}; [[b]] = (s+1)·2⁻¹ (free authenticated affine).
        let d_inv = d.inv();
        let s = auth_scale(&auth_a, d_inv);
        let two_inv = Fp::new(2).inv();
        let b = auth_scale(&auth_add_constant(&s, Fp::one(), key)?, two_inv);
        return Ok(b);
    }
    Err(MpcError::Protocol(
        "malicious square-protocol random bit: drew a == 0 on every attempt (astronomically \
         improbable) — refusing to emit a bit"
            .into(),
    ))
}

/// MSB-first comparison of AUTHENTICATED secret bits `a_bits` (LSB-first) against a
/// PUBLIC value `pub_val`, returning the authenticated verdict `[[a > pub_val]]`. The
/// public bits are constants, so per-bit products against them collapse to FREE
/// authenticated affine maps; only the `eq`/`gt` chain spends authenticated
/// multiplications. Mirrors [`crate::auth_compare`] `auth_greater_than_public_bits`.
fn auth_greater_than_public_bits(
    session: &mut MacSession,
    a_bits: &[AuthenticatedShare],
    pub_val: u64,
    key: &crate::authenticated::MacKey,
) -> Result<AuthenticatedShare, MpcError> {
    let mut gt = session.auth_const_sharing(Fp::zero());
    let mut eq = session.auth_const_sharing(Fp::one());
    let l = a_bits.len();
    for k in (0..l).rev() {
        let a_k = &a_bits[k];
        let b_k = (pub_val >> k) & 1;
        let eq_here = if b_k == 1 {
            // a_k ∧ ¬1 = 0 ⇒ gt unchanged; (a_k == 1) == a_k.
            a_k.clone()
        } else {
            // b_k=0: a_k ∧ ¬0 = a_k ⇒ term = eq·a_k (one auth_mul).
            let term = session.auth_mul(&eq, a_k)?;
            gt = auth_add(&gt, &term)?;
            // (a_k == 0) == 1 − a_k.
            auth_add_constant(&auth_scale(a_k, Fp::one().neg()), Fp::one(), key)?
        };
        eq = session.auth_mul(&eq, &eq_here)?;
    }
    Ok(gt)
}

/// Open ONLY the verdict bit of an authenticated comparison result — the third named
/// open. The value sharing `[verdict]` is reconstructed (the MAC was consumed by the
/// check, never opened); a non-boolean reconstruction is refused rather than coerced.
/// This is the ONLY value this path opens besides the sign-independent `a²` and the
/// statistically-masked `sum + r`. `[OPUS-4.8]`
fn open_auth_verdict(
    backend: &ShamirBackend,
    verdict: &AuthenticatedShare,
) -> Result<bool, MpcError> {
    let bit = backend.reconstruct(verdict.value_shares())?;
    if bit == Fp::zero() {
        Ok(false)
    } else if bit == Fp::one() {
        Ok(true)
    } else {
        Err(MpcError::Protocol(format!(
            "malicious_disclose_threshold_verdict: verdict reconstructed to a non-boolean field \
             element {} (expected 0 or 1) — refusing to coerce",
            bit.value()
        )))
    }
}

#[cfg(test)]
mod tests {
    //! sq-6fv7 + sq-m4zi/sq-e7ma acceptance suite. The load-bearing ones:
    //! - `differential_*`: across many (sum, threshold) pairs incl. edges, the
    //!   MAC-checked verdict equals the plaintext `sum > threshold`.
    //! - `matches_semi_honest_disclose`: the malicious path agrees with the
    //!   semi-honest [`crate::compare::disclose_threshold_verdict`] verdict.
    //! - `tamper_in_*_is_caught`: a tampered open on each of the three named opens
    //!   (`a²`, `c=sum+r`, `verdict`) AND on the range-proof zero-test product
    //!   (`tamper_in_the_zero_test_open_is_caught`, sq-m4zi) aborts fail-closed.
    //! - `auth_secret_is_zero_matches_plaintext` / `auth_range_proof_*`: the
    //!   MAC-checked zero-test and range proof accept the in-range / zero cases and
    //!   reject the out-of-range / nonzero ones, agreeing with the plaintext.
    //! - `*_fails_closed`: out-of-range threshold / sum and n<2t+1 are descriptive
    //!   errors, never a silent wrong verdict.
    use super::*;
    use crate::compare::disclose_threshold_verdict;
    use crate::shamir::ShamirBackend;

    /// The typed descriptor is IDENTICAL to [`crate::auth_compare`]'s (the module
    /// contract's "matching" claim, made mechanical), and reports the malicious
    /// adversary the annotation graph's closed-world encoding rests on.
    #[test]
    fn security_descriptor_matches_auth_compare() {
        let backend = ShamirBackend::new(3).unwrap();
        let desc = security_descriptor(&backend);
        assert_eq!(desc, crate::auth_compare::security_descriptor(&backend));
        assert_eq!(desc.adversary, crate::backend::AdversaryModel::Malicious);
    }

    /// Deal a degree-`t` sharing of a cleartext sum, exactly as the federation
    /// aggregate would hand `disclose_threshold_verdict` its `[sum]`.
    fn share_sum(backend: &ShamirBackend, sum: u64) -> Vec<Share> {
        backend.dealer().share(Fp::new(sum))
    }

    /// `authenticate_existing` lifts a plain `[v]` into `[[v]] = ([v],[α·v])`: the
    /// value sharing is reused verbatim and the MAC sharing reconstructs to `α·v`
    /// (the relation the §2.5 check verifies). The cleartext `α` is read ONLY via the
    /// test-only accessor — production has no such path.
    #[test]
    fn authenticate_existing_carries_the_alpha_v_mac() {
        for &v in &[0u64, 1, 42, 100_000, DECOMP_VALUE_MAX_EXCLUSIVE - 1] {
            let backend = ShamirBackend::new_seeded(5, v.wrapping_add(3)).unwrap();
            let plain = share_sum(&backend, v);
            let mut dealer = backend.dealer();
            let mut session = dealer.new_mac_session();
            let auth = session.authenticate_existing(&plain).unwrap();
            // Value sharing is the caller's, unchanged.
            assert_eq!(
                backend.reconstruct(auth.value_shares()).unwrap(),
                Fp::new(v)
            );
            // MAC reconstructs to α·v (read α only via the test accessor).
            let alpha = session.alpha_for_test();
            assert_eq!(
                backend.reconstruct(auth.mac_shares()).unwrap(),
                alpha.mul(Fp::new(v)),
                "authenticate_existing MAC must equal α·v"
            );
            // And the honest authenticated value passes its MAC-check.
            session.mac_check(std::slice::from_ref(&auth)).unwrap();
        }
    }

    /// THE differential: the MAC-checked verdict equals the plaintext `sum > thr`
    /// across a spread of in-range sums and thresholds incl. edges, several
    /// honest-majority party counts.
    #[test]
    fn differential_malicious_disclose() {
        let cases: &[(u64, u64)] = &[
            (0, 0),
            (1, 0),
            (0, 1),
            (100_000, 100_000),
            (100_001, 100_000),
            (99_999, 100_000),
            (DECOMP_VALUE_MAX_EXCLUSIVE - 1, 0),
            (
                DECOMP_VALUE_MAX_EXCLUSIVE - 1,
                DECOMP_VALUE_MAX_EXCLUSIVE - 2,
            ),
            (42, 1000),
            (1000, 42),
        ];
        for n in [3usize, 5, 7] {
            for (idx, &(sum, thr)) in cases.iter().enumerate() {
                let backend = ShamirBackend::new_seeded(
                    n,
                    (idx as u64).wrapping_mul(131).wrapping_add(n as u64),
                )
                .unwrap();
                let shares = share_sum(&backend, sum);
                let got = malicious_threshold_over_sum(&backend, &shares, thr).unwrap();
                assert_eq!(
                    got,
                    sum > thr,
                    "n={n}: malicious disclose verdict for ({sum} > {thr}) must match plaintext"
                );
            }
        }
    }

    /// The malicious path agrees with the semi-honest `disclose_threshold_verdict` on
    /// the disclosed boolean (same verdict, just MAC-checked opens).
    #[test]
    fn matches_semi_honest_disclose() {
        for n in [3usize, 5] {
            for &(sum, thr) in &[
                (150_000u64, 100_000u64),
                (50_000, 100_000),
                (100_000, 100_000),
            ] {
                let backend = ShamirBackend::new_seeded(n, sum.wrapping_add(thr)).unwrap();
                let shares = share_sum(&backend, sum);
                let mal = malicious_threshold_over_sum(&backend, &shares, thr).unwrap();
                let semi = disclose_threshold_verdict(&backend, &shares, thr).unwrap();
                let semi_bool = semi.rows[0][0]
                    .as_ref()
                    .map(|t| t.to_string().contains("true"))
                    .unwrap_or(false);
                assert_eq!(
                    mal, semi_bool,
                    "n={n}: malicious must agree with semi-honest"
                );
                assert_eq!(mal, sum > thr);
            }
        }
    }

    /// The public PartialResult surface carries ONLY the boolean.
    #[test]
    fn public_partial_result_carries_only_the_bool() {
        let backend = ShamirBackend::new_seeded(5, 7).unwrap();
        let shares = share_sum(&backend, 200_000);
        let pr = malicious_disclose_threshold_verdict(&backend, &shares, 100_000).unwrap();
        assert_eq!(pr.rows.len(), 1);
        assert_eq!(pr.rows[0].len(), 1);
        assert_eq!(pr.vars.len(), 1);
        assert!(pr.rows[0][0].as_ref().unwrap().to_string().contains("true"));
    }

    /// ACCEPTANCE (open 3): a tamper INTRODUCED at the verdict (a wrong re-sharing of
    /// the final comparison product would land here) is caught by the MAC-check.
    #[test]
    fn tamper_in_the_verdict_is_caught() {
        let backend = ShamirBackend::new_seeded(5, 0x5151).unwrap();
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();
        let key = session.mac_key();
        let auth_sum = session
            .authenticate_existing(&backend.dealer().share(Fp::new(9)))
            .unwrap();
        let (bits, _r) =
            auth_masked_bit_decompose(&mut session, &backend, &auth_sum, &key).unwrap();
        let verdict = auth_greater_than_public_bits(&mut session, &bits, 4, &key).unwrap();
        let tampered = tamper_value(&verdict);
        assert!(
            matches!(
                session.mac_check(std::slice::from_ref(&tampered)),
                Err(MpcError::MacCheckFailed { .. })
            ),
            "a tampered verdict must make the MAC-check abort"
        );
    }

    /// ACCEPTANCE (open 1): a tamper on a square-protocol `[[a²]]` open is caught by
    /// the per-bit MAC-check — the bit is never emitted from a corrupted `a²` open.
    #[test]
    fn tamper_in_the_square_protocol_open_is_caught() {
        let backend = ShamirBackend::new_seeded(5, 0xA2A2).unwrap();
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();
        let a = session.authenticated_share(Fp::new(7));
        let a_sq = session.auth_mul(&a, &a).unwrap();
        // Honest a² passes its MAC-check.
        session.mac_check(std::slice::from_ref(&a_sq)).unwrap();
        // A tampered a² open is caught (the same check the square protocol runs).
        let tampered = tamper_value(&a_sq);
        assert!(matches!(
            session.mac_check(std::slice::from_ref(&tampered)),
            Err(MpcError::MacCheckFailed { .. })
        ));
    }

    /// ACCEPTANCE (open 2): a tamper on the masked-open `[[sum+r]]` (a wrong re-share
    /// producing a consistent codeword of the wrong c) is caught by the MAC-check —
    /// so the bit-decomposition never runs on a corrupted c.
    #[test]
    fn tamper_in_the_masked_open_is_caught() {
        let backend = ShamirBackend::new_seeded(5, 0x5057).unwrap();
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();
        let key = session.mac_key();
        let auth_sum = session
            .authenticate_existing(&backend.dealer().share(Fp::new(123)))
            .unwrap();
        // Build [[sum+r]] exactly as the decompose does, then tamper it before the open.
        let mut auth_r = session.auth_const_sharing(Fp::zero());
        for k in 0..DECOMP_MASK_BITS {
            let bit = auth_square_protocol_random_bit(&mut session, &backend, &key).unwrap();
            auth_r = auth_add(&auth_r, &auth_scale(&bit, Fp::new(1u64 << k))).unwrap();
        }
        let auth_c = auth_add(&auth_sum, &auth_r).unwrap();
        // Honest c passes.
        session.mac_check(std::slice::from_ref(&auth_c)).unwrap();
        // Tampered c is caught.
        let tampered = tamper_value(&auth_c);
        assert!(matches!(
            session.mac_check(std::slice::from_ref(&tampered)),
            Err(MpcError::MacCheckFailed { .. })
        ));
    }

    /// Out-of-range threshold/sum and n<2t+1 are descriptive errors, not silent
    /// wrong verdicts.
    #[test]
    fn fails_closed_on_bad_inputs() {
        let backend = ShamirBackend::new_seeded(5, 1).unwrap();
        let shares = share_sum(&backend, 100);
        // Threshold at/over 2^DECOMP_VALUE_BITS.
        assert!(matches!(
            malicious_threshold_over_sum(&backend, &shares, DECOMP_VALUE_MAX_EXCLUSIVE),
            Err(MpcError::Protocol(_))
        ));
        // An over-magnitude sum aborts via the in-protocol range proof.
        let big = share_sum(&backend, DECOMP_VALUE_MAX_EXCLUSIVE + 5);
        assert!(matches!(
            malicious_threshold_over_sum(&backend, &big, 100_000),
            Err(MpcError::Protocol(_))
        ));
    }

    // ---- sq-m4zi/sq-e7ma: the MAC-checked range-proof zero-tests -----------------

    /// The MAC-checked zero-test [`auth_secret_is_zero`] agrees with the plaintext
    /// `v == 0` over a spread of values, several honest-majority party counts — and
    /// never reconstructs `v` (only the masked `m = v·r` is opened inside).
    #[test]
    fn auth_secret_is_zero_matches_plaintext() {
        for n in [3usize, 5, 7] {
            for &v in &[0u64, 1, 2, 42, 100_000, DECOMP_VALUE_MAX_EXCLUSIVE - 1] {
                let backend = ShamirBackend::new_seeded(n, v.wrapping_add(n as u64)).unwrap();
                let mut dealer = backend.dealer();
                let mut session = dealer.new_mac_session();
                let auth_v = session.authenticate_existing(&backend.dealer().share(Fp::new(v)));
                let auth_v = auth_v.unwrap();
                let is_zero = auth_secret_is_zero(&mut session, &backend, &auth_v).unwrap();
                assert_eq!(
                    is_zero,
                    v == 0,
                    "n={n}: MAC-checked zero-test of v={v} must match plaintext v==0"
                );
            }
        }
    }

    /// ACCEPTANCE (range-proof open, sq-m4zi): a tamper on the zero-test's `[[v·r]]`
    /// product open (the canonical Hole-2 consistent-codeword-of-the-wrong-value
    /// deviation that RS cannot detect at n=2t+1) is caught by the MAC-check — so a
    /// deviating party cannot flip "was it zero?" to smuggle an out-of-range sum past
    /// the fail-closed guard.
    #[test]
    fn tamper_in_the_zero_test_open_is_caught() {
        let backend = ShamirBackend::new_seeded(5, 0x2E20).unwrap();
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();
        // A nonzero v whose zero-test would HONESTLY report false; build [[v·r]]
        // exactly as the zero-test does, then tamper its value before the open.
        let auth_v = session
            .authenticate_existing(&backend.dealer().share(Fp::new(7)))
            .unwrap();
        let auth_mask = session.authenticated_share(Fp::new(3));
        let auth_m = session.auth_mul(&auth_v, &auth_mask).unwrap();
        // Honest m passes its MAC-check (the check the zero-test runs before the open).
        session.mac_check(std::slice::from_ref(&auth_m)).unwrap();
        // A tampered m open is caught — the zero-test can no longer be flipped.
        let tampered = tamper_value(&auth_m);
        assert!(
            matches!(
                session.mac_check(std::slice::from_ref(&tampered)),
                Err(MpcError::MacCheckFailed { .. })
            ),
            "a tampered zero-test product open must make the MAC-check abort"
        );
    }

    /// The MAC-checked range proof [`auth_verify_sum_in_range`] ACCEPTS in-range sums
    /// (incl. edges) and REJECTS out-of-range / field-wrapping sums fail-closed —
    /// agreeing with the semi-honest [`crate::compare::verify_sum_in_range`] verdict on
    /// the same recovered bits.
    #[test]
    fn auth_range_proof_accepts_in_range_and_rejects_out_of_range() {
        // In-range sums (incl. the top of the supported width) must be accepted by the
        // full malicious path — the range proof passes, so a verdict is returned.
        for &sum in &[0u64, 1, 100_000, DECOMP_VALUE_MAX_EXCLUSIVE - 1] {
            let backend = ShamirBackend::new_seeded(5, sum.wrapping_add(11)).unwrap();
            let shares = share_sum(&backend, sum);
            assert!(
                malicious_threshold_over_sum(&backend, &shares, 100_000).is_ok(),
                "in-range sum {sum} must pass the MAC-checked range proof"
            );
        }
        // Out-of-range (>= 2^DECOMP_VALUE_BITS) sums must be REJECTED fail-closed by the
        // MAC-checked range proof's magnitude clause (a descriptive Protocol error,
        // never a silent wrong verdict).
        for &sum in &[
            DECOMP_VALUE_MAX_EXCLUSIVE,
            DECOMP_VALUE_MAX_EXCLUSIVE + 1,
            DECOMP_VALUE_MAX_EXCLUSIVE + 12345,
        ] {
            let backend =
                ShamirBackend::new_seeded(5, sum.wrapping_mul(7).wrapping_add(3)).unwrap();
            let shares = share_sum(&backend, sum);
            assert!(
                matches!(
                    malicious_threshold_over_sum(&backend, &shares, 100_000),
                    Err(MpcError::Protocol(_))
                ),
                "out-of-range sum {sum} must be rejected fail-closed by the range proof"
            );
        }
    }

    // ---- tamper helper (test-only) ---------------------------------------------

    /// Corrupt the VALUE sharing into a CONSISTENT degree-`t` sharing of a DIFFERENT
    /// value (the canonical Hole-2 deviation: a valid codeword of the wrong value,
    /// which RS cannot detect — only the IT-MAC catches it), WITHOUT touching the MAC.
    fn tamper_value(av: &AuthenticatedShare) -> AuthenticatedShare {
        let delta = Fp::new(1);
        let value: Vec<Share> = av
            .value_shares()
            .iter()
            .map(|s| Share {
                x: s.x,
                y: s.y.add(delta),
            })
            .collect();
        crate::authenticated::AuthenticatedShare::new(value, av.mac_shares().to_vec()).unwrap()
    }
}
