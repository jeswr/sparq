// [OPUS-5] sq-km34 (the residual "one real hole" of the capability matrix §4.2):
// IT-MAC on the degree-2t equality/multiplication open at the MINIMAL n = 2t+1.
// Promotes the equality primitive from semi-honest-only (Reed-Solomon has ZERO
// redundancy at degree 2t when n = 2t+1) to honest-majority DETECT-AND-ABORT, and
// runs the check BEFORE any verdict is acted on (the coZK-2025/1026 discipline).
//! Honest-majority **detect-and-abort** secret-shared equality — the IT-MAC upgrade
//! of the semi-honest masked-open equality test at the MINIMAL party count.
//!
//! ## What this closes (capability matrix §4.2 "the one real hole"; design Hole 1)
//!
//! The semi-honest equality test ([`crate::join::HiddenValueJoin`]'s `secure_equal`,
//! and every hidden-key join / bounded-path hop built on it) computes
//! `m = (a − b)·r` for a fresh nonzero mask `r` and **opens `m` at degree `2t`**:
//! `m == 0 ⇔ a == b`, and `m` is uniform nonzero otherwise, so only the match bit
//! is revealed. The integrity problem is the *degree* of that open. The
//! honest-majority constructor fixes `t = ⌊(n−1)/2⌋`, so for odd `n` we have
//! `n = 2t+1` and a degree-`2t` codeword over `n` points has **zero** Reed–Solomon
//! redundancy: EVERY vector of `n` field elements is a valid degree-`2t` codeword.
//! A deviating party can therefore shift the opened product to any value it likes
//! and the RS checker cannot see it — the match bit flips **silently**. That is
//! not a hypothetical: it is the `honest_majority_equality_open_*` boundary the
//! semi-honest path deliberately pins rather than claims away, and
//! `semi_honest_degree_2t_open_at_minimal_n_accepts_a_forged_share` below
//! re-demonstrates it as the *baseline this module improves on*.
//!
//! ## The fix (design §2.4 route (a) + §2.5, at `n = 2t+1`)
//!
//! Never open a degree-`2t` product at all. Run the same algebraic test over
//! **authenticated** sharings `[[x]] = ([x], [α·x])` under a session-global,
//! secret-shared MAC key `[α]` no party knows:
//!
//! 1. authenticate both keys ([`MacSession::authenticated_share`]);
//! 2. `[[d]] = [[a]] − [[b]]` — FREE (the IT-MAC is linear, [`auth_sub`]);
//! 3. `[[m]] = [[d]] · [[r]]` for a fresh nonzero authenticated mask `[[r]]` —
//!    ONE [`MacSession::auth_mul`], i.e. a degree-`2t` product **followed by a BGW
//!    degree reduction**, with a SECOND independent reduce carrying the MAC
//!    (`[α·m] = reduce([α·d]·[r])`), which MAC-covers the re-sharing (Hole 2);
//! 4. **MAC-check, then open** ([`MacSession::mac_check_and_open`], §2.5): open a
//!    leakage-free `σ` and abort with [`MpcError::MacCheckFailed`] iff `σ ≠ 0`;
//!    only on `σ == 0` is the (now verified) `m` handed back and turned into a
//!    match bit.
//!
//! So the value the verdict reads is opened at degree **`t`**, not `2t`, and its
//! integrity rests on the secret `α` rather than on RS over-determination — which
//! is exactly why it works at `n = 2t+1` where RS has nothing to work with.
//!
//! ## Operand order is load-bearing (do NOT swap the `auth_mul` arguments)
//!
//! [`MacSession::auth_mul`] carries the MAC forward through its **first** operand
//! (`[α·z] = reduce([α·x]·[y])`), so a consistent value-only tamper on the SECOND
//! operand is *adopted*: the product and its MAC both track the tampered operand and
//! `σ` stays `0` (the documented [`crate::auth_disclose`] limitation). This module
//! therefore multiplies `auth_mul(&d, &r)` — **difference first, mask second** — so:
//!
//! - a tamper on `[[d]]` (the operand that carries the SECRET keys) is **caught**:
//!   `σ = α·d·r − (d+δ)·r·α = −α·δ·r ≠ 0`
//!   (pinned by `tamper_on_the_difference_operand_is_caught`, which goes RED if the
//!   arguments are ever swapped);
//! - a tamper on `[[r]]` sits in the adopted slot, but the mask is *only* required
//!   to be nonzero: the adopted product is `d·r'`, which is zero iff `d` is zero, so
//!   the **verdict is preserved** unless the adversary lands `r' = 0` — and `r` is
//!   secret, so it can only guess it (probability `1/p ≈ 2^{−61}`). Pinned by
//!   `tamper_on_the_mask_operand_preserves_the_verdict`.
//!
//! That asymmetry is why the equality test can be promoted here while
//! [`crate::auth_disclose`] cannot: the disclose path's range proof puts a
//! *load-bearing* mask in the adopted slot (zeroing it switches the proof off),
//! whereas here the adopted slot holds a mask whose only job is to be nonzero.
//!
//! ## Adversary model, stated precisely (no over-claim)
//!
//! Honest-majority, up to `t` deviating parties, **malicious-with-abort (unanimous
//! abort)** on the equality open: AXIS-1 `Malicious`, AXIS-2 `Abort(Unanimous)`,
//! AXIS-3 `HonestMajority` — reported as
//! [`OperatorClass::AuthenticatedEqualityJoin`](crate::backend::OperatorClass::AuthenticatedEqualityJoin),
//! as against the semi-honest-only
//! [`OperatorClass::EqualityJoin`](crate::backend::OperatorClass::EqualityJoin) the
//! unauthenticated path still honestly reports. Concretely, for any deviation by
//! `≤ t` parties on the authenticated path, the opened product is either UNCHANGED
//! or the check fires with probability `≥ 1 − 1/p`: writing the per-party
//! deviations as `δ_i` (value mult) and `δ'_i` (MAC mult), the check opens
//! `σ = Σ λ_i·((α·d)_i·δ'_i − α·d_i·δ_i)`, which the adversary can only zero — without
//! knowing `α` — by also zeroing the value shift `Σ λ_i·d_i·δ_i`, i.e. by not changing
//! the answer.
//!
//! **Residuals (honest, not closed here).**
//! - The trusted-dealer simulation (`RandomnessModel::TrustedDealerSim`, bead
//!   `sq-yyro`) is unchanged: an adversary that corrupts the DEALER can rewrite a
//!   whole share vector, which `≤ t` parties provably cannot (a degree-`t` sharing
//!   with `t+1` untouched points pins its polynomial). Even then, per the operand-order
//!   argument above, rewriting the MASK cannot flip a verdict without hitting `r' = 0`.
//! - Fiat–Shamir challenge scope + the `≈ 2^61` grinding bound documented on
//!   [`MacSession::mac_check_and_open`] apply unchanged.
//! - Selective-failure/abort leakage (whether an abort correlates with the inputs) is
//!   the open composition question tracked with the rest of `sq-km34`.
//! - **Research-grade: NOT externally audited.** Accredited-cryptographer sign-off is
//!   PENDING (`sq-qhy4`); no unqualified soundness or privacy claim is made here.

use crate::authenticated::{auth_sub, AuthenticatedShare};
use crate::field::Fp;
use crate::partial::MpcError;
use crate::shamir::{MacSession, ShamirBackend};

/// `n >= 2t+1` is required so the authenticated multiplication's degree reduction
/// exists and the MAC-check is honest-majority. Fail-closed, mirroring
/// [`crate::auth_compare`]'s guard with the equality-mode message.
fn check_party_count(n: usize, t: usize) -> Result<(), MpcError> {
    if n < 2 * t + 1 {
        return Err(MpcError::Protocol(format!(
            "IT-MAC secure equality needs n >= 2t+1 (the authenticated multiplication's degree \
             reduction does, and the MAC-check is honest-majority); got n = {n}, t = {t}"
        )));
    }
    Ok(())
}

/// The authenticated masked difference `[[m]] = ([[a]] − [[b]])·[[r]]` for a fresh
/// nonzero mask `r` drawn from the session's CSPRNG — the composable core of the
/// malicious-with-abort equality test.
///
/// `m == 0 ⇔ a == b`, and for unequal keys `m` is uniform nonzero, so opening it
/// reveals ONLY the match bit (never a key or their difference) — the same
/// disclosure envelope as the semi-honest test. The caller MUST NOT open the
/// returned sharing directly: hand it (ideally with the rest of the batch) to
/// [`MacSession::mac_check_and_open`], which verifies before it returns the value.
///
/// Exposed so a batching caller — an all-pairs hidden join, a bounded-path hop
/// chain — can build `|L|·|R|` products under ONE session and pay a SINGLE `σ` open
/// for the whole batch (the §5 amortisation; see [`malicious_secure_equal_batch`]).
///
/// `a`, `b` must be authenticated under `session`'s key. The mask's MAC is not
/// load-bearing (see the module docs' operand-order note): `auth_mul(&d, &r)` puts
/// the SECRET difference in the MAC-covered slot and the mask in the adopted one, so
/// tampering the mask cannot flip a verdict without landing `r' = 0`. `[OPUS-5]`
pub fn auth_masked_equality_product(
    session: &mut MacSession,
    a: &AuthenticatedShare,
    b: &AuthenticatedShare,
) -> Result<AuthenticatedShare, MpcError> {
    // d = a - b: FREE (the IT-MAC is linear — value and MAC both subtract).
    mask_authenticated_difference(session, &auth_sub(a, b)?)
}

/// Mask an ALREADY-FORMED authenticated difference: draw a fresh nonzero `r` and
/// return `[[d·r]]`. Split out of [`auth_masked_equality_product`] so the
/// operand-order guard (`tamper_on_the_difference_operand_is_caught`) can hand in a
/// TAMPERED `[[d]]` and still exercise the PRODUCTION multiplication — a guard that
/// re-spelled `auth_mul` itself would not go red when this call is mutated.
fn mask_authenticated_difference(
    session: &mut MacSession,
    d: &AuthenticatedShare,
) -> Result<AuthenticatedShare, MpcError> {
    // Fresh nonzero mask from the session's masking CSPRNG, authenticated under the
    // same α. Zero is rejected inside `draw_nonzero_fp`: masking by zero would make
    // m = 0 even for unequal keys (a false match).
    let mask_value = session.draw_nonzero_fp();
    let r = session.authenticated_share(mask_value);
    // m = d·r — ONE MAC-carrying multiplication. ORDER IS LOAD-BEARING: the
    // difference (which carries the secret keys) goes in the MAC-covered FIRST slot
    // so a tamper on it is caught; the mask goes in the adopted SECOND slot where a
    // tamper is verdict-preserving unless it hits r' = 0. Swapping these would move
    // the forgeable slot onto the secret difference and let a deviating party turn a
    // match into a non-match. See the module docs and
    // `tamper_on_the_difference_operand_is_caught`.
    session.auth_mul(d, &r)
}

/// Turn a MAC-VERIFIED opened masked product into the match bit. Total on `Fp`
/// (unlike the comparison's verdict, every field element is a legal product: a
/// nonzero `m` simply means "not equal"), so there is no coercion guard to write —
/// the fail-closed behaviour lives entirely in the MAC-check that produced `m`.
fn match_bit(m: Fp) -> bool {
    m == Fp::zero()
}

/// **Malicious-with-abort secret-shared equality** `a == b` at the MINIMAL
/// `n = 2t+1` — the IT-MAC twin of the semi-honest masked-open equality test.
///
/// Returns `a == b`, or [`MpcError::MacCheckFailed`] if the batched IT-MAC check
/// caught a tamper on the authenticated path (a forged share at the open, a wrong
/// `degree_reduce` re-sharing, a corrupted difference). It NEVER returns a wrong
/// boolean because a tampered product was opened: the check runs BEFORE the verdict
/// is available to be acted on (the coZK-2025/1026 discipline).
///
/// `a`, `b` are passed in the clear ONLY because this routine plays ALL parties in
/// one process (exactly as the dealer that shares them does); they are secret-shared
/// internally and never reconstructed. The ONLY value ever opened is the masked
/// product (uniform nonzero unless the keys match) and the leakage-free `σ`
/// (identically `0` on an honest run).
///
/// No magnitude bound — equality is exact over the whole field `F_p` (unlike
/// [`crate::auth_compare`], whose bit-decomposition caps operands at `2^60`).
/// `n >= 2t+1` is fail-closed. Prefer [`malicious_secure_equal_batch`] whenever more
/// than one pair is tested: it spends ONE `σ` open for the whole batch. `[OPUS-5]`
pub fn malicious_secure_equal(backend: &ShamirBackend, a: Fp, b: Fp) -> Result<bool, MpcError> {
    Ok(malicious_secure_equal_batch(backend, &[(a, b)])?[0])
}

/// **Batched malicious-with-abort equality** — the join-shaped entry point: test
/// `|pairs|` key pairs and pay a **single** `σ` open for all of them.
///
/// This is the §5 amortisation the design banks on: an all-pairs hidden join runs
/// `|L|·|R|` equality opens, and authenticating the whole sweep costs ONE extra
/// open — the marginal round cost of the malicious upgrade is `O(1)` per *batch*,
/// not per *pair* (pinned by `batched_equality_spends_one_sigma_open`). All pairs
/// run under ONE session key `α`, which is what makes the single random-linear-
/// combination check cover the whole vector.
///
/// Returns one match bit per input pair, positionally aligned with `pairs`, or
/// [`MpcError::MacCheckFailed`] — in which case NO verdict is returned at all
/// (fail-closed: one tampered pair aborts the batch rather than yielding `|pairs|−1`
/// trustworthy bits and one silent lie). An empty input is an empty result (no
/// session, no open). `[OPUS-5]`
pub fn malicious_secure_equal_batch(
    backend: &ShamirBackend,
    pairs: &[(Fp, Fp)],
) -> Result<Vec<bool>, MpcError> {
    check_party_count(backend.parties(), backend.threshold())?;
    if pairs.is_empty() {
        return Ok(Vec::new());
    }
    let mut dealer = backend.dealer();
    let mut session = dealer.new_mac_session();

    let mut products = Vec::with_capacity(pairs.len());
    for &(a, b) in pairs {
        let aa = session.authenticated_share(a);
        let bb = session.authenticated_share(b);
        products.push(auth_masked_equality_product(&mut session, &aa, &bb)?);
    }

    // MAC-CHECK BEFORE OPEN (design §2.5). `mac_check_and_open` hands back exactly
    // the products it authenticated, so the values we turn into match bits are the
    // ones the check covered — and on σ != 0 it returns an error and NO value, so no
    // inconsistent result can be acted on (the coZK-2025/1026 confidentiality
    // discipline). ONE σ open for the whole batch.
    let opened = session.mac_check_and_open(&products)?;
    Ok(opened.into_iter().map(match_bit).collect())
}

#[cfg(test)]
mod tests {
    //! sq-km34 acceptance suite. The load-bearing ones:
    //! - `differential_*`: the MAC-checked verdict equals plaintext `a == b` across
    //!   party counts and edge values (and never spuriously aborts).
    //! - `semi_honest_degree_2t_open_at_minimal_n_accepts_a_forged_share` +
    //!   `forged_open_share_at_minimal_n_is_caught`: the BEFORE/AFTER pair that makes
    //!   the promotion a demonstrated property, not a claim — the same class of
    //!   deviation is invisible to the degree-`2t` open at `n = 2t+1` and ABORTS on
    //!   the authenticated path.
    //! - `tamper_on_the_difference_operand_is_caught`: the operand-order mutation
    //!   guard (goes RED if `auth_mul(&d, &r)` is ever swapped).
    //! - `batched_equality_spends_one_sigma_open`: the §5 amortisation, MEASURED.
    use super::*;
    use crate::backend::{
        AbortKind, AdversaryModel, CorruptionThreshold, MaliciousSecurity, MpcBackend,
        OperatorClass, OutputGuarantee,
    };
    use crate::field::P;
    use crate::shamir::{self, ShamirBackend, Share};

    /// Consistently shift the VALUE sharing of an authenticated share by `delta`
    /// (add it to EVERY share) WITHOUT touching the MAC — a perfectly consistent
    /// degree-`t` codeword of a DIFFERENT value, which Reed–Solomon cannot detect at
    /// any `n`. Only the IT-MAC catches it.
    fn tamper_value(av: &AuthenticatedShare, delta: Fp) -> AuthenticatedShare {
        let value: Vec<Share> = av
            .value_shares()
            .iter()
            .map(|s| Share {
                x: s.x,
                y: s.y.add(delta),
            })
            .collect();
        AuthenticatedShare::new(value, av.mac_shares().to_vec()).unwrap()
    }

    /// THE differential: the MAC-checked equality verdict equals plaintext `a == b`
    /// across honest-majority party counts (including the MINIMAL odd `n = 2t+1`,
    /// which is the whole point of this module) and edge values.
    #[test]
    fn differential_malicious_secure_equal() {
        let cases: &[(u64, u64)] = &[
            (0, 0),
            (0, 1),
            (1, 0),
            (7, 7),
            (100_000, 100_000),
            (100_000, 100_001),
            (P - 1, P - 1),
            (P - 1, 0),
            (1 << 60, 1 << 60),
            (1 << 60, (1 << 60) + 1),
        ];
        for n in [3usize, 5, 7] {
            for (idx, &(a, b)) in cases.iter().enumerate() {
                let backend =
                    ShamirBackend::new_seeded(n, (idx as u64).wrapping_mul(97) + n as u64).unwrap();
                let got = malicious_secure_equal(&backend, Fp::new(a), Fp::new(b)).unwrap();
                assert_eq!(
                    got,
                    a == b,
                    "n={n}: MAC-checked equality verdict for ({a} == {b}) must match plaintext"
                );
            }
        }
    }

    /// The batch entry point agrees with the scalar one pair-for-pair, and never
    /// spuriously aborts on an honest run.
    #[test]
    fn differential_batched_matches_plaintext() {
        let pairs: Vec<(Fp, Fp)> = (0..24u64)
            .map(|i| (Fp::new(i % 5), Fp::new(i % 3)))
            .collect();
        for n in [3usize, 5] {
            let backend = ShamirBackend::new_seeded(n, 0xBA7C).unwrap();
            let got = malicious_secure_equal_batch(&backend, &pairs).unwrap();
            let want: Vec<bool> = pairs.iter().map(|&(a, b)| a == b).collect();
            assert_eq!(got, want, "n={n}: batched verdicts must match plaintext");
        }
    }

    /// BASELINE (the hole this module closes): at the MINIMAL `n = 2t+1` a forged
    /// share on a degree-`2t` product open is **information-theoretically
    /// undetectable** — the checked open ACCEPTS it and returns a DIFFERENT value,
    /// which is exactly a silently flipped match bit. This is the semi-honest
    /// equality path's pinned boundary, restated here as the "before" half of the
    /// promotion so the "after" half below is measured against something real.
    #[test]
    fn semi_honest_degree_2t_open_at_minimal_n_accepts_a_forged_share() {
        let backend = ShamirBackend::new_seeded(3, 0x0BAD).unwrap(); // n=3, t=1 → n=2t+1
        let t = backend.threshold();
        assert_eq!(backend.parties(), 2 * t + 1, "the minimal honest-majority n");
        let mut dealer = backend.dealer();

        // The semi-honest construction: m = (a-b)·r opened at degree 2t, for EQUAL
        // keys (so the honest verdict is `true`).
        let sa = dealer.share(Fp::new(42));
        let sb = dealer.share(Fp::new(42));
        let mask = dealer.draw_nonzero_fp();
        let r = dealer.share(mask);
        let d = shamir::sub_shares(&sa, &sb).unwrap();
        let mut m_shares = shamir::mul_shares_raw(&d, &r).unwrap();
        assert_eq!(
            shamir::reconstruct_degree(&m_shares, 2 * t).unwrap(),
            Fp::zero(),
            "honest run: equal keys open to m = 0 (a match)"
        );

        // ONE party forges its share of the degree-2t product.
        m_shares[0].y = m_shares[0].y.add(Fp::new(1));
        let forged = shamir::reconstruct_degree(&m_shares, 2 * t)
            .expect("degree-2t open at n = 2t+1 has ZERO RS redundancy — it cannot detect this");
        assert_ne!(
            forged,
            Fp::zero(),
            "the forged open yields a NON-zero m: the match bit flipped SILENTLY — this is the \
             hole the IT-MAC path closes"
        );
    }

    /// PROMOTION (the "after" half): the SAME class of deviation — a forged/shifted
    /// opened product at the MINIMAL `n = 2t+1` — makes the authenticated path ABORT
    /// with [`MpcError::MacCheckFailed`] instead of returning a flipped bit.
    /// Soundness here comes from the secret `α`, not from RS redundancy (there is
    /// none), which is precisely why it survives at `n = 2t+1`.
    #[test]
    fn forged_open_share_at_minimal_n_is_caught() {
        let backend = ShamirBackend::new_seeded(3, 0x600D).unwrap(); // n=3, t=1 → n=2t+1
        assert_eq!(backend.parties(), 2 * backend.threshold() + 1);
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();

        let a = session.authenticated_share(Fp::new(42));
        let b = session.authenticated_share(Fp::new(42));
        let m = auth_masked_equality_product(&mut session, &a, &b).unwrap();
        // Honest run: MAC-check passes and the equal keys open to a match.
        let honest = session.mac_check_and_open(std::slice::from_ref(&m)).unwrap();
        assert!(match_bit(honest[0]), "equal keys must open to a match");

        // Now shift the opened product (the deviation the degree-2t open above could
        // not see) — the MAC-check must abort rather than yield a flipped verdict.
        let tampered = tamper_value(&m, Fp::new(1));
        let outcome = session.mac_check_and_open(std::slice::from_ref(&tampered));
        assert!(
            matches!(outcome, Err(MpcError::MacCheckFailed { .. })),
            "a tampered equality product at n = 2t+1 must ABORT (the sq-km34 promotion), got \
             {outcome:?}"
        );
    }

    /// OPERAND-ORDER MUTATION GUARD, over the PRODUCTION multiplication. A consistent
    /// value tamper on the DIFFERENCE operand — the one carrying the secret keys —
    /// must be caught. It is caught only because `mask_authenticated_difference`
    /// multiplies `auth_mul(d, &r)`, putting `d` in the MAC-COVERED first slot; swap
    /// the arguments and `d` lands in the slot `auth_mul` ADOPTS, the check passes,
    /// and a deviating party can turn a match into a non-match. This test drives the
    /// production helper (not a re-spelled `auth_mul`) precisely so it goes RED on
    /// that mutation.
    #[test]
    fn tamper_on_the_difference_operand_is_caught() {
        let backend = ShamirBackend::new_seeded(3, 0xD1FF).unwrap();
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();

        // Equal keys ⇒ honest d = 0 ⇒ honest verdict "match". Corrupt d to a
        // consistent sharing of δ ≠ 0 without fixing its MAC: the attack that would
        // turn the match into a non-match.
        let a = session.authenticated_share(Fp::new(1234));
        let b = session.authenticated_share(Fp::new(1234));
        let d = tamper_value(&auth_sub(&a, &b).unwrap(), Fp::new(7));

        let m = mask_authenticated_difference(&mut session, &d).unwrap();

        let outcome = session.mac_check_and_open(std::slice::from_ref(&m));
        assert!(
            matches!(outcome, Err(MpcError::MacCheckFailed { .. })),
            "a tampered difference operand must abort (σ = −α·δ·r ≠ 0); if this passes, the \
             auth_mul operands were swapped and the secret operand sits in the adopted slot — got \
             {outcome:?}"
        );
    }

    /// HONEST SCOPE of the adopted slot: a consistent value tamper on the MASK
    /// operand is NOT caught (`auth_mul` adopts a second-operand tamper — the
    /// documented `auth_disclose` limitation), but it is **verdict-preserving**: the
    /// adopted product is `d·r'`, zero iff `d` is zero. Flipping a verdict this way
    /// needs `r' = 0`, i.e. guessing the secret mask. This test states that boundary
    /// exactly rather than claiming "any tamper aborts".
    #[test]
    fn tamper_on_the_mask_operand_preserves_the_verdict() {
        for (a, b, want) in [(5u64, 5u64, true), (5, 6, false)] {
            let backend = ShamirBackend::new_seeded(3, 0x1123 + a).unwrap();
            let mut dealer = backend.dealer();
            let mut session = dealer.new_mac_session();
            let aa = session.authenticated_share(Fp::new(a));
            let bb = session.authenticated_share(Fp::new(b));
            let d = auth_sub(&aa, &bb).unwrap();
            let mask = session.draw_nonzero_fp();
            // Shift the mask by a value the adversary picks blind (it does not know
            // `mask`, so it cannot aim at `r' = 0`).
            let r = tamper_value(&session.authenticated_share(mask), Fp::new(9));
            let m = session.auth_mul(&d, &r).unwrap();
            let opened = session
                .mac_check_and_open(std::slice::from_ref(&m))
                .expect("a second-operand tamper is ADOPTED — this is the honest scope boundary");
            assert_eq!(
                match_bit(opened[0]),
                want,
                "adopting a mask tamper must not change the ({a} == {b}) verdict"
            );
        }
    }

    /// A tamper on the MAC sharing (rather than the value) is caught too — the check
    /// binds both sides.
    #[test]
    fn tamper_on_the_product_mac_is_caught() {
        let backend = ShamirBackend::new_seeded(5, 0xACAC).unwrap();
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();
        let a = session.authenticated_share(Fp::new(11));
        let b = session.authenticated_share(Fp::new(11));
        let m = auth_masked_equality_product(&mut session, &a, &b).unwrap();
        let mac: Vec<Share> = m
            .mac_shares()
            .iter()
            .map(|s| Share {
                x: s.x,
                y: s.y.add(Fp::new(3)),
            })
            .collect();
        let tampered = AuthenticatedShare::new(m.value_shares().to_vec(), mac).unwrap();
        assert!(matches!(
            session.mac_check_and_open(std::slice::from_ref(&tampered)),
            Err(MpcError::MacCheckFailed { .. })
        ));
    }

    /// AMORTISATION (§5), MEASURED not asserted: a `k`-pair batch spends exactly ONE
    /// `σ` open, so the malicious upgrade's marginal round cost is `O(1)` per batch —
    /// the property that makes authenticating an all-pairs `|L|·|R|` join affordable.
    #[test]
    fn batched_equality_spends_one_sigma_open() {
        let backend = ShamirBackend::new_seeded(3, 0x5161).unwrap();
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();

        let mut products = Vec::new();
        for i in 0..12u64 {
            let a = session.authenticated_share(Fp::new(i));
            let b = session.authenticated_share(Fp::new(i % 4));
            products.push(auth_masked_equality_product(&mut session, &a, &b).unwrap());
        }
        let opened = session.mac_check_and_open(&products).unwrap();
        assert_eq!(opened.len(), 12);
        assert_eq!(
            session.sigma_opens_for_test(),
            1,
            "one batched MAC-check must authenticate the WHOLE sweep with a single σ open"
        );
        for (i, m) in opened.iter().enumerate() {
            assert_eq!(match_bit(*m), (i as u64) == (i as u64) % 4);
        }
    }

    /// One tampered pair aborts the WHOLE batch — no partially-trustworthy verdict
    /// vector is ever returned.
    #[test]
    fn one_bad_pair_aborts_the_whole_batch() {
        let backend = ShamirBackend::new_seeded(3, 0x0B0B).unwrap();
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();
        let mut products = Vec::new();
        for i in 0..4u64 {
            let a = session.authenticated_share(Fp::new(i));
            let b = session.authenticated_share(Fp::new(i));
            products.push(auth_masked_equality_product(&mut session, &a, &b).unwrap());
        }
        products[2] = tamper_value(&products[2], Fp::new(5));
        assert!(matches!(
            session.mac_check_and_open(&products),
            Err(MpcError::MacCheckFailed { .. })
        ));
    }

    /// Fail-closed: a deficient `(n, t)` (`n < 2t+1`, which the public constructors
    /// cannot build) is refused with a descriptive error, never a wrong verdict.
    /// The empty batch is a no-op (nothing checked, nothing opened).
    #[test]
    fn fails_closed_on_deficient_party_count_and_empty_batch() {
        let deficient = ShamirBackend::with_unchecked_threshold(3, 2); // n=3 < 2t+1=5
        assert!(matches!(
            malicious_secure_equal(&deficient, Fp::new(1), Fp::new(1)),
            Err(MpcError::Protocol(_))
        ));
        let ok = ShamirBackend::new_seeded(3, 1).unwrap();
        assert!(malicious_secure_equal_batch(&ok, &[]).unwrap().is_empty());
    }

    /// REGISTRY (the sq-km34 promotion, per-operator): at the minimal `n = 2t+1` the
    /// authenticated equality operator reports **Malicious + Abort(Unanimous)**
    /// (projection: `HonestMajorityAbort`) while the unauthenticated one still
    /// honestly reports `SemiHonestOnly`. Both are true at the same `(n, t)` — which
    /// is exactly why the guarantee is reported PER OPERATOR.
    #[test]
    fn authenticated_equality_operator_reports_abort_at_minimal_n() {
        let b = ShamirBackend::new(3).unwrap(); // n=3, t=1 → n = 2t+1
        let plain = b.operator_security(OperatorClass::EqualityJoin);
        let auth = b.operator_security(OperatorClass::AuthenticatedEqualityJoin);

        assert_eq!(
            plain.malicious_security(0),
            MaliciousSecurity::SemiHonestOnly,
            "the unauthenticated degree-2t open is still semi-honest-only at n = 2t+1"
        );
        assert_eq!(auth.adversary, AdversaryModel::Malicious);
        assert_eq!(
            auth.output_guarantee,
            OutputGuarantee::Abort(AbortKind::Unanimous),
            "IT-MAC equality is detect-and-abort (unanimous), NOT identifiable abort, NOT GOD"
        );
        assert_eq!(auth.threshold, CorruptionThreshold::HonestMajority { t: 1 });
        assert_eq!(
            auth.malicious_security(0),
            MaliciousSecurity::HonestMajorityAbort,
            "the sq-km34 promotion: SemiHonestOnly → HonestMajorityAbort at the MINIMAL n = 2t+1"
        );
        assert_ne!(
            plain.output_guarantee, auth.output_guarantee,
            "the two equality operators must not collapse to one bit"
        );
    }

    /// The promotion holds for every honest-majority party count the constructor
    /// builds — it does not silently depend on over-provisioning (the whole point).
    #[test]
    fn promotion_holds_at_every_honest_majority_party_count() {
        for n in [2usize, 3, 4, 5, 6, 7, 9] {
            let b = ShamirBackend::new(n).unwrap();
            let auth = b.operator_security(OperatorClass::AuthenticatedEqualityJoin);
            assert_eq!(
                auth.malicious_security(0),
                MaliciousSecurity::HonestMajorityAbort,
                "n={n}: the IT-MAC equality open is detect-and-abort at every honest-majority n"
            );
        }
    }
}
