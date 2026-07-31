// [OPUS-5] sq-km34 (the core promotion): IT-MAC on the degree-2t equality/mult open
// at the MINIMAL n = 2t+1. The masked product `m = d·r` is an AUTHENTICATED value and
// is MAC-checked BEFORE it is opened (design §2.5), and the mask `[[r]]` carries an
// authenticated NONZERO WITNESS `u = r·s` opened in the same batch — so the Hole-1
// forged-share flip, the Hole-2 wrong re-sharing, and the Hole-3 `r = 0` false-match
// are all fail-closed aborts where the semi-honest `join::HiddenValueJoin::secure_equal`
// had ZERO Reed-Solomon redundancy to detect anything.
//! Honest-majority **malicious-with-abort** secret-shared equality — the IT-MAC
//! upgrade of the semi-honest degree-`2t` equality open (design
//! `research/mpc-malicious-security-design.md` Hole 1, the headline hole).
//!
//! ## What this closes (design §1 Hole 1 + Hole 2 + Hole 3)
//!
//! The semi-honest equality test
//! ([`HiddenValueJoin::secure_equal`](crate::join::HiddenValueJoin)) computes
//! `d = a − b`, draws a nonzero mask `r`, and opens the product `m = d·r` at degree
//! `2t`; `m == 0 ⇔ a == b`, and for `a ≠ b` the opened `m` is uniform nonzero so only
//! the match bit is revealed. At the honest-majority default `t = ⌊(n−1)/2⌋` and odd
//! `n` this is `n = 2t+1`, where a degree-`2t` codeword is determined by **exactly**
//! the `n` shares — **zero** Reed–Solomon redundancy. A single forged product share
//! therefore flips the match bit *information-theoretically undetectably*, and per
//! coZK eprint 2025/1026 acting on a mid-pipeline opened value that was never
//! integrity-checked is a **confidentiality** interaction as well as a correctness
//! one: the open happens before anything has established that the value opened is the
//! value the protocol computed.
//!
//! Reed–Solomon cannot fix this — there is nothing to over-determine. The IT-MAC can,
//! because its soundness comes from the *secret* `α`, not from codeword redundancy
//! (design §2.5). This module wires the landed primitives — authenticated sharing
//! ([`crate::authenticated`], sq-km34.1), the MAC-carrying multiplication
//! ([`MacSession::auth_mul`], sq-km34.3), and the batched MAC-check at the open
//! boundary ([`MacSession::mac_check_and_open`], sq-km34.4) — into the equality
//! operator itself (design §6 step 5).
//!
//! ## The construction (per pair)
//!
//! ```text
//!   [[d]] = [[a]] − [[b]]                     FREE (linear; both components)
//!   [[r]], [[s]]  ← two fresh nonzero masks, authenticated under the session key
//!   [[m]] = auth_mul([[d]], [[r]])            the masked difference   (verdict)
//!   [[u]] = auth_mul([[r]], [[s]])            the mask NONZERO WITNESS
//!   ── one batched MAC-check over the WHOLE batch, then open ──
//!   abort unless u ≠ 0        ;   verdict := (m == 0)
//! ```
//!
//! Both products are degree-`t` after the authenticated mult-then-reduce, so the open
//! is a degree-`t` open of a value whose integrity has already been established — not
//! the bare degree-`2t` open the semi-honest path performs.
//!
//! ### Why the operand ORDER is load-bearing
//!
//! [`MacSession::auth_mul`] carries the MAC forward as `[α·z] = reduce([α·x]·[y])` —
//! from the **first** operand's MAC times the **second** operand's value. That is what
//! makes a deviation in either `degree_reduce` tamper-evident (Hole 2), but it also
//! means a value-only tamper on the **second** operand is *adopted*: `z` and `α·z` both
//! track the tampered `y`, so `σ = 0` and the batched check passes. This is a property
//! of the primitive, documented on [`MacSession::auth_mul`] and demonstrated to be
//! end-to-end exploitable on the disclose path
//! ([`crate::auth_disclose`], witness
//! `a_zeroed_zero_test_mask_defeats_the_range_proof_and_flips_the_verdict`).
//!
//! So the two gates here are ordered deliberately:
//!
//! - `[[m]] = auth_mul([[d]], [[r]])` puts the **data** (`d`, hence both join keys) in
//!   the MAC-carrying FIRST slot. Every deviation that could change the *verdict* by
//!   changing the data — a forged input share, a tampered input MAC, a wrong re-sharing
//!   in either reduce, a forged share on the final open — lands in a slot the check
//!   covers, and aborts.
//! - the adopted SECOND slot holds only the mask `r`, whose value is irrelevant to the
//!   verdict *provided it is nonzero* — and that proviso is exactly what the witness
//!   gate below establishes.
//!
//! ### Why the mask needs a nonzero WITNESS (Hole 3, and why a MAC alone is not enough)
//!
//! `m == 0 ⇔ d == 0` holds **iff `r ≠ 0`**. A party that forces `r = 0` makes `m = 0`
//! for every pair — a false match on every row, and the one deviation CLASS in this
//! protocol that produces a wrong answer rather than an abort (residual (2) below is the
//! same class reached by a different route). An IT-MAC on the open
//! does not close it: it authenticates that `m` is `d·r` for the `r` that was actually
//! used, not that `r ≠ 0`, so a **consistently** authenticated `[[r]] = ([0],[0])` (which
//! `auth_const_sharing(0)` produces, and which needs no knowledge of `α`) opens
//! `m = 0`, `α·m = 0` — perfectly MAC-consistent — and flips the verdict. This is
//! stated as an open residual in [`crate::randomness`] ("an IT-MAC on the open **alone**
//! does NOT suffice") and as Hole 3 in the design doc, where it is deferred to the
//! jointly-generated mask (sq-yyro).
//!
//! The witness closes it **inside this operator**, without waiting for dealer-less
//! randomness, by opening a second authenticated product `u = r·s` for a fresh nonzero
//! `s` and refusing to act on any verdict unless `u ≠ 0`. Over a field `u = r·s ≠ 0`
//! iff `r ≠ 0` and `s ≠ 0`, so:
//!
//! - `[[r]]` consistently zeroed → `u = 0` → **abort** (the value-level gate fires even
//!   though the MAC-check is satisfied);
//! - `[r]`'s value shares tampered without its MAC → `r` sits in the FIRST slot of the
//!   witness gate, so the defect propagates into `u`'s MAC → `σ ≠ 0` → **abort**;
//! - `[s]` tampered in the adopted second slot → it can only change `u` to another value
//!   the adversary cannot steer to `0` without knowing `s`; and if it *were* steered to
//!   `0` the gate aborts anyway (fail-closed in the adversary's disfavour).
//!
//! `u` is leakage-free with respect to the data: `r`, `s` are fresh, uniform, nonzero
//! and independent of the keys, so `u` is uniform nonzero and independent of `d`. Both
//! masks are drawn **per pair** — reusing `s` across a batch would make
//! `m_i/u_i = d_i/s`, whose pairwise ratios leak whether two pairs share the same key
//! difference. The per-pair freshness is therefore a confidentiality requirement, not a
//! stylistic one.
//!
//! ## Security tier (stated precisely, no over-claim)
//!
//! **Honest-majority, malicious-with-abort, unanimous abort** for the equality verdict,
//! at the **minimal `n = 2t+1`** (design §3). AXIS-1 `Malicious`, AXIS-2
//! `Abort(Unanimous)` — detect-and-abort, NOT identifiable abort, NOT GOD; AXIS-3
//! `HonestMajority` (fails closed if `n < 2t+1`). It does NOT enter the
//! dishonest-majority SPDZ regime (route (b) / sq-j5ok). Nothing here is externally
//! audited — MPC in this crate is research-grade and EXTERNAL accredited-cryptographer
//! sign-off is PENDING (`sq-qhy4`).
//!
//! **What this does NOT deliver — read before relying on the tier:**
//!
//! 1. **The registry still reports `SemiHonestOnly` for `OperatorClass::EqualityJoin`.**
//!    Wiring `operator_descriptor` (and a malicious backend mode) is a SEPARATE bead,
//!    sq-km34.7. This module delivers the *protocol*; a federation inspecting
//!    [`crate::backend::BackendInfo`] still sees the semi-honest baseline, and
//!    [`crate::join::HiddenValueJoin`] still runs the semi-honest `secure_equal`.
//! 2. **Mask SECRECY is still the trusted-dealer simulation.** The witness proves the
//!    mask used was nonzero; it does not make the mask *jointly generated*. One
//!    [`crate::shamir::ShamirDealer`] draws `r` and `s` and therefore knows them
//!    ([`crate::randomness::RandomnessModel::TrustedDealerSim`]). A party that BOTH knows
//!    `r` AND can present a different `[r]` to the verdict gate than to the witness gate
//!    could shift the adopted mask to `0` and flip a verdict. Neither holds in the model
//!    as implemented — the dealer is honest by construction, and the two gates consume
//!    the SAME [`AuthenticatedShare`] object, so operand consistency across gates is
//!    structural rather than conventional. Both become real obligations in a distributed
//!    deployment, and are closed by dealer-less randomness (sq-yyro) plus Chida-style
//!    multiplication-gate verification binding both operands.
//! 3. **No confidentiality upgrade.** MACs make the match bits *correct and
//!    tamper-evident*; they do not hide them. The per-pair match-graph leak (L2) of an
//!    all-pairs join is an orthogonal axis, closed by keeping the bit secret-shared
//!    ([`crate::compare::secure_equal_to_bit`] /
//!    [`crate::join::HiddenValueJoin::fully_oblivious_batched_join`]), not by this module.
//!
//! ## Cost (honest; the harness measures it — no numbers in prose)
//!
//! Per pair: **two** authenticated multiplications (the verdict product and the mask
//! witness), each a mult-then-reduce for the value plus an independent mult-then-reduce
//! for the MAC — against the semi-honest path's single `mul_shares_raw` + open. The
//! design (§5) budgeted one authenticated multiplication per equality; the witness is
//! the documented additional price of closing Hole 3 in-operator rather than deferring it
//! to sq-yyro. The **check** amortises: one `σ` open covers the WHOLE batch, so an
//! all-pairs join's `|L|·|R|` equality opens share a single check
//! (`one_sigma_open_amortises_the_whole_batch` measures this against the real code path).
//! Pass the whole batch to [`auth_equal_verdicts`] in one call — calling it per pair
//! forfeits exactly that amortisation. `[OPUS-5]`

use crate::authenticated::{auth_sub, AuthenticatedShare};
use crate::field::Fp;
use crate::partial::MpcError;
use crate::shamir::{MacSession, ShamirBackend};

/// Values this operator opens per pair: the masked difference `m` (which carries the
/// verdict) and the mask nonzero-witness `u`. Both go into the SAME batched MAC-check,
/// so the per-pair opens cost no extra `σ`.
const OPENS_PER_PAIR: usize = 2;

/// Where a pair's equality mask `[[r]]` comes from.
///
/// Production has exactly one variant. The test-only variants exist so the adversarial
/// suite drives the deviations through the **real** [`auth_equal_verdicts`] body rather
/// than a mirrored copy that could drift from it — the Hole-3 `r = 0` attack in
/// particular must be exercised against production code for
/// `consistently_zeroed_mask_is_caught_by_the_nonzero_witness` to mean anything.
#[derive(Clone, Copy)]
enum MaskSource {
    /// A fresh, uniform, NONZERO mask drawn from the session's masking CSPRNG and
    /// authenticated under the session key — the production source.
    FreshNonzero,
    /// **Test-only.** A *consistently authenticated* sharing of `0`: `[[r]] = ([0],[0])`,
    /// which satisfies the MAC relation `α·0 = 0` and so passes the batched check. This
    /// is the Hole-3 deviation the nonzero witness exists to catch — and the deviation
    /// [`MacSession::auth_mul`]'s second-operand adoption would otherwise let through
    /// silently, flipping every verdict to "equal".
    #[cfg(test)]
    ConsistentlyZeroed,
    /// **Test-only.** A fresh nonzero mask whose VALUE shares are shifted while its MAC
    /// is left alone. In the verdict gate this sits in the adopted second slot, but the
    /// witness gate consumes the same `[[r]]` in the MAC-carrying FIRST slot, so the
    /// defect propagates into `u`'s MAC and the batched check fires.
    #[cfg(test)]
    ValueShiftedOnly,
}

/// **Malicious-with-abort secret-shared equality over already-authenticated keys** —
/// the batched core, and the entry point a hidden-value join should drive.
///
/// Decides `a_i == b_i` for every pair, opening (per pair) only the masked difference
/// and the mask nonzero-witness, and only after ONE batched IT-MAC check over the whole
/// batch has passed. Returns the verdicts positionally aligned with `pairs`.
///
/// **Aborts, never a wrong answer.** Returns [`MpcError::MacCheckFailed`] if the batched
/// check fails (a forged share, a tampered input MAC, a wrong `degree_reduce`
/// re-sharing, a tampered open) **or** if any pair's mask nonzero-witness opens to `0`
/// (the Hole-3 `r = 0` false-match deviation). Every witness is validated before ANY
/// verdict is derived, so a batch with one bad mask yields no verdicts at all — the
/// abort-before-acting discipline the coZK-2025/1026 interaction requires.
///
/// `n >= 2t+1` is fail-closed. Honest-majority; research-grade, externally unaudited
/// (`sq-qhy4`). See the module docs for the exact residuals. `[OPUS-5]`
pub fn auth_equal_verdicts(
    session: &mut MacSession,
    pairs: &[(AuthenticatedShare, AuthenticatedShare)],
) -> Result<Vec<bool>, MpcError> {
    auth_equal_verdicts_with(session, pairs, MaskSource::FreshNonzero)
}

/// The body of [`auth_equal_verdicts`], parameterised by where the mask comes from so
/// the adversarial tests exercise THIS code rather than a copy of it.
fn auth_equal_verdicts_with(
    session: &mut MacSession,
    pairs: &[(AuthenticatedShare, AuthenticatedShare)],
    mask: MaskSource,
) -> Result<Vec<bool>, MpcError> {
    if pairs.is_empty() {
        return Ok(Vec::new());
    }
    // Honest-majority headroom: each authenticated multiplication degree-reduces a
    // degree-`2t` product, which needs `2t+1 <= n`. Fail closed BEFORE any sharing.
    let key = session.mac_key();
    let (n, t) = (key.parties(), key.threshold());
    if n < 2 * t + 1 {
        return Err(MpcError::Protocol(format!(
            "malicious-secure equality needs n >= 2t+1 (each authenticated multiplication's \
             degree reduction does); got n = {n}, t = {t}"
        )));
    }

    // Build every pair's gadget first, so the batched check below covers the WHOLE
    // batch with a single σ open (design §5 amortisation).
    let mut batch: Vec<AuthenticatedShare> = Vec::with_capacity(pairs.len() * OPENS_PER_PAIR);
    for (a, b) in pairs {
        let (m, u) = equality_gadget(session, a, b, mask)?;
        batch.push(m);
        batch.push(u);
    }

    // ONE batched IT-MAC check, then open — `mac_check_and_open` hands back exactly the
    // values it authenticated, so the values acted on below cannot drift from the values
    // verified, and nothing is opened twice.
    let opened = session.mac_check_and_open(&batch)?;

    // Validate EVERY mask witness before deriving ANY verdict. A zeroed mask is
    // MAC-consistent (α·0 = 0), so this value-level gate — not the MAC-check — is what
    // catches Hole 3.
    for (i, chunk) in opened.chunks_exact(OPENS_PER_PAIR).enumerate() {
        if chunk[1] == Fp::zero() {
            return Err(MpcError::MacCheckFailed {
                detail: format!(
                    "equality mask nonzero-witness for pair {i} of {} opened u = r·s = 0 — the \
                     mask used in the masked difference m = d·r was ZERO, which forces m = 0 (a \
                     false match) for every key pair regardless of the keys. A zeroed mask is \
                     MAC-CONSISTENT (α·0 = 0), so the batched IT-MAC check cannot see it; this \
                     value-level witness is the gate that does. Refusing the whole batch \
                     (design Hole 3; n = {n}, t = {t})",
                    pairs.len()
                ),
            });
        }
    }

    // Only now, with every witness established, read the verdicts.
    Ok(opened
        .chunks_exact(OPENS_PER_PAIR)
        .map(|chunk| chunk[0] == Fp::zero())
        .collect())
}

/// One pair's authenticated equality gadget: the masked difference `[[m]] = [[d]]·[[r]]`
/// (whose open decides the verdict) and the mask nonzero-witness `[[u]] = [[r]]·[[s]]`.
///
/// Both are returned UNOPENED and UNCHECKED — the caller batches them into a single
/// [`MacSession::mac_check_and_open`]. See the module docs for why `d` takes the
/// MAC-carrying first operand slot in the verdict gate and `r` takes it in the witness
/// gate.
fn equality_gadget(
    session: &mut MacSession,
    a: &AuthenticatedShare,
    b: &AuthenticatedShare,
    mask: MaskSource,
) -> Result<(AuthenticatedShare, AuthenticatedShare), MpcError> {
    // d = a − b: FREE (local, both components), degree t.
    let d = auth_sub(a, b)?;
    let r = draw_mask(session, mask);
    // s is ALWAYS an honest fresh nonzero draw: the deviations under test live on `r`,
    // and a witness whose own second operand were also corrupted would not isolate them.
    let s_val = session.draw_nonzero_fp();
    let s = session.authenticated_share(s_val);
    // m = d·r — `d` FIRST so every data-side deviation propagates into m's MAC.
    let m = session.auth_mul(&d, &r)?;
    // u = r·s — `r` FIRST so a value-only tamper on the mask propagates into u's MAC,
    // and `u == 0` iff the mask (or s) is zero.
    let u = session.auth_mul(&r, &s)?;
    Ok((m, u))
}

/// Mint a pair's mask under the requested [`MaskSource`]. Production is the single
/// `FreshNonzero` arm; the others are the adversarial suite's injection points.
fn draw_mask(session: &mut MacSession, mask: MaskSource) -> AuthenticatedShare {
    match mask {
        MaskSource::FreshNonzero => {
            let r_val = session.draw_nonzero_fp();
            session.authenticated_share(r_val)
        }
        // A *consistently authenticated* zero: value sharing [0] and MAC sharing
        // [α·0] = [0]. No knowledge of α is needed to produce it, and it satisfies the
        // MAC relation exactly — which is the whole point of the attack.
        #[cfg(test)]
        MaskSource::ConsistentlyZeroed => session.auth_const_sharing(Fp::zero()),
        #[cfg(test)]
        MaskSource::ValueShiftedOnly => {
            let r_val = session.draw_nonzero_fp();
            let honest = session.authenticated_share(r_val);
            let shifted: Vec<crate::shamir::Share> = honest
                .value_shares()
                .iter()
                .map(|s| crate::shamir::Share {
                    x: s.x,
                    y: s.y.add(Fp::one()),
                })
                .collect();
            AuthenticatedShare::new(shifted, honest.mac_shares().to_vec())
                .expect("shifting share values keeps the party points and length identical")
        }
    }
}

/// **Malicious-with-abort equality over two cleartext keys** — the scalar convenience
/// entry point, the IT-MAC twin of the semi-honest
/// [`HiddenValueJoin::secure_equal`](crate::join::HiddenValueJoin).
///
/// The keys are passed in cleartext ONLY because this routine plays all parties in one
/// process: they are authenticated-secret-shared internally and never reconstructed,
/// exactly as the dealer that shares them does. The only values opened are the masked
/// difference, the mask nonzero-witness, and the leakage-free `σ` (identically `0` on
/// an honest run) — never a key nor their difference.
///
/// Prefer [`malicious_secure_equal_batch`] / [`auth_equal_verdicts`] for more than one
/// pair: one batched MAC-check covers a whole batch, so per-pair calls pay one `σ` open
/// each for no added security. `[OPUS-5]`
pub fn malicious_secure_equal(backend: &ShamirBackend, a: Fp, b: Fp) -> Result<bool, MpcError> {
    Ok(malicious_secure_equal_batch(backend, &[(a, b)])?[0])
}

/// **Malicious-with-abort equality over a BATCH of cleartext key pairs**, decided under
/// a single session and a single batched IT-MAC check — the amortised shape an
/// all-pairs hidden join wants (design §5).
///
/// Returns one verdict per pair, positionally aligned. Aborts (returning no verdict at
/// all) if the batched check fails or any pair's mask nonzero-witness opens to `0`.
/// `n >= 2t+1` is fail-closed. `[OPUS-5]`
pub fn malicious_secure_equal_batch(
    backend: &ShamirBackend,
    pairs: &[(Fp, Fp)],
) -> Result<Vec<bool>, MpcError> {
    if pairs.is_empty() {
        return Ok(Vec::new());
    }
    let mut dealer = backend.dealer();
    let mut session = dealer.new_mac_session();
    let authenticated: Vec<(AuthenticatedShare, AuthenticatedShare)> = pairs
        .iter()
        .map(|&(a, b)| (session.authenticated_share(a), session.authenticated_share(b)))
        .collect();
    auth_equal_verdicts(&mut session, &authenticated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shamir::Share;

    /// Every backend here is at the MINIMAL honest-majority party count `n = 2t+1`
    /// (odd `n` ⇒ `t = ⌊(n−1)/2⌋` ⇒ `n = 2t+1`), i.e. exactly the configuration where
    /// the degree-`2t` open has ZERO Reed–Solomon redundancy and the semi-honest path
    /// is information-theoretically undetectable. That is the point of the bead: the
    /// guarantees below come from the secret `α`, not from over-determination.
    const MINIMAL_N: [usize; 3] = [3, 5, 7];

    fn backend(n: usize, seed: u64) -> ShamirBackend {
        let b = ShamirBackend::new_seeded(n, seed).unwrap();
        assert_eq!(
            b.parties(),
            2 * b.threshold() + 1,
            "these tests must run at the MINIMAL n = 2t+1 (zero RS redundancy at degree 2t)"
        );
        b
    }

    /// Shift a sharing's value by `δ = 1` without touching its MAC — the canonical
    /// "forged share / inconsistent input" deviation.
    fn tamper_value(av: &AuthenticatedShare) -> AuthenticatedShare {
        let value: Vec<Share> = av
            .value_shares()
            .iter()
            .map(|s| Share {
                x: s.x,
                y: s.y.add(Fp::one()),
            })
            .collect();
        AuthenticatedShare::new(value, av.mac_shares().to_vec()).unwrap()
    }

    /// The dual deviation: shift the MAC sharing instead.
    fn tamper_mac(av: &AuthenticatedShare) -> AuthenticatedShare {
        let mac: Vec<Share> = av
            .mac_shares()
            .iter()
            .map(|s| Share {
                x: s.x,
                y: s.y.add(Fp::one()),
            })
            .collect();
        AuthenticatedShare::new(av.value_shares().to_vec(), mac).unwrap()
    }

    /// ACCEPTANCE (correctness): the MAC-checked verdict equals the plaintext `a == b`
    /// on every pair, at every MINIMAL `n = 2t+1` — the differential that proves the
    /// authenticated path computes the same function as the semi-honest one.
    #[test]
    fn verdicts_match_plaintext_equality_at_minimal_n() {
        let cases: [(u64, u64); 7] = [
            (0, 0),
            (0, 1),
            (1, 0),
            (12_345, 12_345),
            (12_345, 12_346),
            (u64::from(u32::MAX), u64::from(u32::MAX)),
            (1 << 40, (1 << 40) + 1),
        ];
        for n in MINIMAL_N {
            let b = backend(n, 0xE0_0000 + n as u64);
            let pairs: Vec<(Fp, Fp)> =
                cases.iter().map(|&(x, y)| (Fp::new(x), Fp::new(y))).collect();
            let got = malicious_secure_equal_batch(&b, &pairs).expect("honest path must not abort");
            let want: Vec<bool> = cases.iter().map(|&(x, y)| x == y).collect();
            assert_eq!(got, want, "n = {n}: MAC-checked verdicts must equal a == b");
            // ...and the scalar entry agrees with the batched one.
            for (&(x, y), &w) in cases.iter().zip(want.iter()) {
                assert_eq!(
                    malicious_secure_equal(&b, Fp::new(x), Fp::new(y)).unwrap(),
                    w,
                    "n = {n}: scalar entry must agree with the batch for ({x}, {y})"
                );
            }
        }
    }

    /// ACCEPTANCE (sq-km34, THE headline): **a forged input share at the minimal
    /// `n = 2t+1` aborts instead of flipping the match bit.** This is precisely the
    /// deviation Reed–Solomon cannot see there — the semi-honest
    /// `HiddenValueJoin::secure_equal` documents it as information-theoretically
    /// undetectable and `adversarial_tests::tampered_share_in_secure_equality_open_is_
    /// undetectable_at_n_eq_2t_plus_1` pins that it silently flips the verdict. Here the
    /// same class of deviation is caught by the secret `α`.
    ///
    /// The tamper is applied to an INPUT key's value sharing (an inconsistent input, the
    /// design's Hole-3 second sub-deviation), which flows into `d` — the MAC-carrying
    /// FIRST operand of the verdict gate — so the defect propagates into `m`'s MAC and
    /// `σ ≠ 0`.
    #[test]
    fn forged_input_key_share_aborts_at_minimal_n() {
        for n in MINIMAL_N {
            let b = backend(n, 0xF0_0000 + n as u64);
            let mut dealer = b.dealer();
            let mut session = dealer.new_mac_session();
            // Two EQUAL keys: honestly this is a match. The tamper shifts one of them,
            // so an undetected deviation would turn the match into a non-match.
            let a = session.authenticated_share(Fp::new(42));
            let b_share = session.authenticated_share(Fp::new(42));
            let got = auth_equal_verdicts(&mut session, &[(tamper_value(&a), b_share)]);
            assert!(
                matches!(got, Err(MpcError::MacCheckFailed { .. })),
                "n = {n}: a forged input share must abort at the minimal n = 2t+1, got {got:?}"
            );
        }
    }

    /// ACCEPTANCE: the dual deviation — tampering an input's MAC sharing rather than its
    /// value — is caught too. The batched check binds value and MAC, so breaking either
    /// side makes `σ ≠ 0`.
    #[test]
    fn tampered_input_mac_aborts_at_minimal_n() {
        let b = backend(3, 0xF1_0001);
        let mut dealer = b.dealer();
        let mut session = dealer.new_mac_session();
        let x = session.authenticated_share(Fp::new(7));
        let y = session.authenticated_share(Fp::new(9));
        let got = auth_equal_verdicts(&mut session, &[(tamper_mac(&x), y)]);
        assert!(
            matches!(got, Err(MpcError::MacCheckFailed { .. })),
            "a tampered input MAC must abort, got {got:?}"
        );
    }

    /// ACCEPTANCE (Hole 3, **the deviation the MAC alone cannot see**): a
    /// *consistently authenticated* zero mask `[[r]] = ([0],[0])` — which satisfies
    /// `α·0 = 0` and therefore passes the batched IT-MAC check — is caught by the
    /// value-level nonzero WITNESS and aborts.
    ///
    /// The test is deliberately two-sided so it cannot pass vacuously:
    ///
    /// 1. **The attack is real.** With the zeroed mask, the verdict gate's masked
    ///    difference `m` opens to `0` for a pair of UNEQUAL keys — i.e. without the
    ///    witness this path would return `true` (a false match) — and the batched
    ///    MAC-check over `m` alone PASSES, so nothing in the IT-MAC layer objects.
    /// 2. **The witness catches it.** The production entry point aborts on the same
    ///    inputs, and the honest control on the same keys returns the true verdict.
    ///
    /// This is the equality-path counterpart of `auth_disclose`'s
    /// `a_zeroed_zero_test_mask_defeats_the_range_proof_and_flips_the_verdict`, which
    /// records the same `auth_mul` second-operand adoption going UNCAUGHT on the
    /// disclose path. If part 1 ever starts failing, `MacSession` gained
    /// multiplication-gate verification binding both operands and this module's residual
    /// (2) should be revisited.
    #[test]
    fn consistently_zeroed_mask_is_caught_by_the_nonzero_witness() {
        for n in MINIMAL_N {
            let b = backend(n, 0xF2_0000 + n as u64);
            let (ka, kb) = (Fp::new(1_000), Fp::new(2_000)); // UNEQUAL keys.

            // --- 1. the attack is real: m opens to 0 and the MAC-check is happy ---
            {
                let mut dealer = b.dealer();
                let mut session = dealer.new_mac_session();
                let x = session.authenticated_share(ka);
                let y = session.authenticated_share(kb);
                let (m, _u) =
                    equality_gadget(&mut session, &x, &y, MaskSource::ConsistentlyZeroed).unwrap();
                let opened = session
                    .mac_check_and_open(std::slice::from_ref(&m))
                    .expect("a zeroed mask is MAC-CONSISTENT, so the check alone cannot see it");
                assert_eq!(
                    opened[0],
                    Fp::zero(),
                    "n = {n}: with r = 0 the masked difference opens to 0 for UNEQUAL keys — \
                     without the nonzero witness this is a silent FALSE MATCH"
                );
            }

            // --- 2. the witness catches it, and the honest control is unaffected ---
            let mut dealer = b.dealer();
            let mut session = dealer.new_mac_session();
            let x = session.authenticated_share(ka);
            let y = session.authenticated_share(kb);
            let got =
                auth_equal_verdicts_with(&mut session, &[(x, y)], MaskSource::ConsistentlyZeroed);
            assert!(
                matches!(got, Err(MpcError::MacCheckFailed { .. })),
                "n = {n}: a consistently-authenticated ZERO mask must abort on the nonzero \
                 witness, got {got:?}"
            );
            assert!(
                !malicious_secure_equal(&b, ka, kb).unwrap(),
                "n = {n}: the honest control on the same keys must report `not equal`"
            );
        }
    }

    /// ACCEPTANCE: a VALUE-only tamper on the mask — the slot
    /// [`MacSession::auth_mul`] *adopts* in the verdict gate — is caught, because the
    /// witness gate consumes the same `[[r]]` in the MAC-carrying FIRST slot. This is
    /// what binds "the mask that was witnessed" to "the mask that masked the
    /// difference" under the batched check.
    #[test]
    fn value_only_tamper_on_the_mask_is_caught_by_the_witness_gate() {
        for n in MINIMAL_N {
            let b = backend(n, 0xF3_0000 + n as u64);
            let mut dealer = b.dealer();
            let mut session = dealer.new_mac_session();
            let x = session.authenticated_share(Fp::new(5));
            let y = session.authenticated_share(Fp::new(5));
            let got =
                auth_equal_verdicts_with(&mut session, &[(x, y)], MaskSource::ValueShiftedOnly);
            assert!(
                matches!(got, Err(MpcError::MacCheckFailed { .. })),
                "n = {n}: a value-only mask tamper must abort via the witness gate, got {got:?}"
            );
        }
    }

    /// ACCEPTANCE (design §5 amortisation, MEASURED not asserted in prose): a batch of
    /// `k` pairs — `2k` opened values — costs exactly ONE `σ` open, so the malicious
    /// upgrade's marginal round cost is `O(1)` per opened BATCH, not per opened value.
    /// This is what makes the upgrade affordable for an all-pairs `|L|·|R|` join.
    #[test]
    fn one_sigma_open_amortises_the_whole_batch() {
        let b = backend(5, 0xF4_0004);
        let mut dealer = b.dealer();
        let mut session = dealer.new_mac_session();
        let pairs: Vec<(AuthenticatedShare, AuthenticatedShare)> = (0..8u64)
            .map(|i| {
                (
                    session.authenticated_share(Fp::new(i)),
                    session.authenticated_share(Fp::new(i % 3)),
                )
            })
            .collect();
        assert_eq!(session.sigma_opens_for_test(), 0, "no check has run yet");
        let got = auth_equal_verdicts(&mut session, &pairs).unwrap();
        assert_eq!(got, (0..8u64).map(|i| i == i % 3).collect::<Vec<_>>());
        assert_eq!(
            session.sigma_opens_for_test(),
            1,
            "8 pairs = 16 opened values must share ONE batched σ open (design §5)"
        );
    }

    /// ACCEPTANCE (confidentiality): the opened transcript carries the match bit and
    /// nothing else. For a FIXED unequal pair, the masked difference `m` is
    /// re-randomised on every run — it is (near-)never the key difference `d` itself and
    /// takes many distinct values across independent sessions — and the mask witness `u`
    /// is always nonzero and independent. This pins that the difference `a − b` (and
    /// hence either key) is not what gets opened.
    #[test]
    fn the_masked_difference_is_re_randomised_and_never_reveals_the_difference() {
        let (ka, kb) = (Fp::new(9_000), Fp::new(1_000));
        let d = ka.sub(kb);
        let mut seen = std::collections::BTreeSet::new();
        const RUNS: u64 = 32;
        for seed in 0..RUNS {
            let b = backend(3, 0xF5_0000 + seed);
            let mut dealer = b.dealer();
            let mut session = dealer.new_mac_session();
            let x = session.authenticated_share(ka);
            let y = session.authenticated_share(kb);
            let (m, u) = equality_gadget(&mut session, &x, &y, MaskSource::FreshNonzero).unwrap();
            let opened = session.mac_check_and_open(&[m, u]).unwrap();
            assert_ne!(
                opened[0],
                Fp::zero(),
                "unequal keys must open a NONZERO masked difference"
            );
            assert_ne!(
                opened[0], d,
                "the masked difference must not open to the key difference itself"
            );
            assert_ne!(
                opened[1],
                Fp::zero(),
                "an honest mask witness u = r·s is never zero"
            );
            seen.insert(opened[0].value());
        }
        assert!(
            seen.len() as u64 >= RUNS - 1,
            "the masked difference must be re-randomised per run (got {} distinct values over \
             {RUNS} runs) — a constant or low-entropy `m` would leak the difference",
            seen.len()
        );
    }

    /// ACCEPTANCE (fail-closed): the operator refuses a party count below `2t+1` rather
    /// than running a degree reduction it cannot complete. Exercised through the
    /// unchecked-threshold constructor, since the public constructor never builds such a
    /// configuration.
    #[test]
    fn party_count_below_2t_plus_1_is_refused() {
        let b = ShamirBackend::with_unchecked_threshold(3, 2);
        let mut dealer = b.dealer();
        let mut session = dealer.new_mac_session();
        let x = session.authenticated_share(Fp::new(1));
        let y = session.authenticated_share(Fp::new(1));
        let got = auth_equal_verdicts(&mut session, &[(x, y)]);
        assert!(
            matches!(got, Err(MpcError::Protocol(_))),
            "n < 2t+1 must be refused fail-closed, got {got:?}"
        );
    }

    /// An empty batch is a no-op that opens nothing (and so spends no `σ`), not an
    /// error — the natural identity for a join whose candidate list is empty.
    #[test]
    fn empty_batch_is_a_no_op() {
        let b = backend(3, 0xF6_0006);
        assert!(malicious_secure_equal_batch(&b, &[]).unwrap().is_empty());
        let mut dealer = b.dealer();
        let mut session = dealer.new_mac_session();
        assert!(auth_equal_verdicts(&mut session, &[]).unwrap().is_empty());
        assert_eq!(session.sigma_opens_for_test(), 0);
    }
}
