// [OPUS-5] sq-km34 (design §6 step 5, the CORE of the bead): the IT-MAC on the
// equality/multiplication open, so the hidden-value equi-join is honest-majority
// TAMPER-EVIDENT-WITH-ABORT at the MINIMAL `n = 2t+1` — the one cell the capability
// matrix names as the real semi-honest hole. The semi-honest twin
// (`join::HiddenValueJoin`) opens the masked product `m = d·r` at degree `2t`,
// where `n = 2t+1` carries ZERO Reed–Solomon redundancy, so a forged product
// share silently FLIPS a match verdict (pinned by
// `adversarial_tests::tampered_share_in_secure_equality_open_is_undetectable_at_n_eq_2t_plus_1`).
// This module carries an IT-MAC through the equality product and MAC-checks the
// whole batch BEFORE any match bit is acted on, so that deviation ABORTS instead.
// Read the "What this does NOT close" section before making any tier claim.

//! [OPUS-5] sq-km34 — honest-majority **tamper-evident-with-abort** hidden-value
//! equality / equi-join (design `research/mpc-malicious-security-design.md`
//! §2.4–§2.5, §6 step 5; Hole 1 and Hole 3).
//!
//! **⚠ EXPERIMENTAL, and NOT an audited malicious-security guarantee.** The design
//! record §3 *aspires* to malicious-with-abort, and the construction below is built
//! for it — but what the in-process tests can establish is **tamper-evidence on the
//! deviations they exercise**, not the malicious-security theorem, and the external
//! accredited-cryptographer review that would discharge the difference is an
//! UNRESOLVED release gate (`sq-qhy4`). So the public entry points are named
//! `experimental_tamper_evident_*` (the same containment
//! [`crate::auth_disclose`] applies to its own IT-MAC twin: a doc caveat under a
//! `malicious_*` name is not containment), and [`equality_join_security`]
//! deliberately does NOT report AXIS-1 [`crate::backend::AdversaryModel::Malicious`]
//! — see that function. Treat this module as research scaffolding: do NOT deploy it
//! as the integrity tier against a genuinely malicious party. `[OPUS-5]`
//!
//! ## The hole this closes
//!
//! [`crate::join::HiddenValueJoin`]'s private `secure_equal` decides a match by
//! opening the masked difference `m = (a − b)·r` at degree `2t`. With the honest-majority
//! `t = ⌊(n−1)/2⌋` and odd `n` (the minimal, and cheapest, party count
//! `n = 2t+1`), degree `2t` is pinned by exactly `2t+1 = n` points — **zero RS
//! redundancy**. A deviating party that forges one product share therefore selects
//! a different, equally-consistent degree-`2t` polynomial: the open returns a
//! *wrong* `m` with **nothing to detect**, flipping a non-match into a match (or
//! back). Reed–Solomon cannot help at any `n` for the sibling deviation either —
//! a wrong `degree_reduce` re-sharing (Hole 2) produces a perfectly consistent
//! degree-`t` codeword of the WRONG product.
//!
//! Both are closed by authentication rather than redundancy: every value on this
//! path carries an information-theoretic MAC `α·x` under a session-global,
//! secret-shared `[α]` that no party knows, and the §2.5 batched random-challenge
//! check opens a leakage-free `σ` and **aborts iff `σ ≠ 0`** before any match bit
//! is returned. Soundness comes from the secrecy of `α` (`≈ 1 − 2^{−61}`), which
//! is exactly why it works at `n = 2t+1` where RS redundancy is zero.
//!
//! ## Operand ORDER is load-bearing (do not "simplify" it)
//!
//! [`MacSession::auth_mul`] carries the output MAC as `[α·z] = reduce([α·x]·[y])`
//! — from the FIRST operand's MAC and the SECOND operand's VALUE. That asymmetry
//! means a value deviation on the **first** operand is CAUGHT (the MAC still holds
//! the true `α·x`, so `σ ≠ 0`) while one on the **second** operand is **ADOPTED**
//! by the MAC (both halves move together, so `σ = 0`).
//!
//! This module therefore always multiplies **`[[d]] · [[r]]`** — the key
//! difference FIRST, the mask SECOND — and that choice is what makes the verdict
//! sound:
//!
//! - a deviation on `[d]` (i.e. on either key sharing, or inside the value reduce)
//!   is caught, so a matching pair cannot be pushed to `d ≠ 0` and a non-matching
//!   pair cannot be pushed to `d = 0`;
//! - a deviation on `[r]` is adopted, but it only replaces the mask with
//!   `r' = r + Δ` for a `Δ` chosen **without knowledge of `r`**. The verdict
//!   `m = d·r' == 0` is unchanged unless `r' = 0`, which needs `Δ = −r`, i.e.
//!   guessing the uniform secret mask (probability `1/p ≈ 2^{−61}` per pair).
//!
//! The reversed order `[[r]] · [[d]]` would be **unsound**: a deviation on `[d]`
//! would then be adopted, and shifting a *matching* pair's `d = 0` to `d = δ ≠ 0`
//! needs no secret at all — a free false non-match. This is pinned by
//! `mask_must_be_the_second_operand_or_a_matching_pair_can_be_flipped`.
//!
//! ## What this does NOT close (state loudly)
//!
//! - **NOT the `r ≠ 0` threat.** The MAC authenticates that the opened `m` equals
//!   `d·r` for the `r` that was actually shared — **not** that `r ≠ 0`. A
//!   correctly-authenticated `r = 0` opens `m = 0` for EVERY pair, i.e. a false
//!   match on every row, and passes the check. Here the mask is drawn nonzero by
//!   the session's own dealer ([`crate::shamir::ShamirDealer::draw_nonzero_fp`])
//!   and no party contributes to it, so no *compute party* can reach that state —
//!   but the guarantee rests on that trusted-dealer randomness assumption, not on
//!   the MAC. A dealer-less (PRSS / coin-toss) source needs a biasing-resistant
//!   joint generation plus an authenticated distributed nonzero test before this
//!   tier survives the move; see `crate::randomness` (module docs, "the `r = 0`
//!   threat") and `research/mpc-distributed-randomness-design.md`. The hole is
//!   exhibited, not merely described, by
//!   `a_zeroed_mask_is_adopted_by_the_mac_and_flips_every_verdict`.
//! - **NOT the L2 match-graph leak.** Exactly as in the semi-honest twin, one
//!   match bit per candidate pair is opened, so the driver learns the whole
//!   bipartite match graph. MACs are an INTEGRITY axis; obliviousness is a
//!   separate CONFIDENTIALITY axis (beads sq-jnkm / sq-xhaw —
//!   [`crate::join::HiddenValueJoin::fully_oblivious_batched_join`] is the
//!   never-opened-bit tier, and it is semi-honest).
//! - **NOT identifiable abort, NOT GOD, NOT dishonest-majority.** The REPORTED
//!   tier is AXIS-1 `SemiHonest` × AXIS-2 `Abort(Unanimous)` × AXIS-3
//!   `HonestMajority` ([`equality_join_security`]) — the IT-MAC hardening is
//!   reported on the OUTPUT-GUARANTEE axis, as
//!   [`crate::backend::SecurityDescriptor::shamir_degree_recon`] already reports
//!   its RS-checked hardening. A cheater can always force an abort,
//!   no cheater is attributed, and `n ≤ 2t` is refused.
//! - **NOT externally audited — and that is why AXIS-1 is not promoted.** Like every
//!   soundness/privacy statement in this crate, the argument above is INTERNAL;
//!   external accredited-cryptographer sign-off is PENDING (bead `sq-qhy4`), which
//!   is an unresolved RELEASE GATE, not a formality the in-process tamper tests can
//!   discharge. Research-grade.
//!
//! ## coZK-2025/1026 (opening on an inconsistent witness)
//!
//! The coZK malicious-pitfalls result (eprint 2025/1026) is why the degree-`2t`
//! hole is a **confidentiality** problem and not only a correctness one: opening a
//! value computed from an inconsistent witness can leak honest inputs. The
//! discipline here is check-then-act at the open boundary —
//! [`MacSession::mac_check_and_open`] returns the values it just authenticated and
//! returns `Err` with **no value at all** when `σ ≠ 0`, so no match bit is ever
//! returned to a caller (or turned into an output row) off an unverified witness.
//! Two honest scopes on that: (1) the batch is *reconstructed* in the same step
//! that forms `σ`, so this is "nothing is acted on before the check", not "nothing
//! is broadcast before the check" — the stronger property needs commit-then-open,
//! which this crate does not have; (2) each opened `m_j` is masked by its OWN
//! fresh `r_j`, so a tampered `m_j` is still uniform (or zero) and carries no more
//! about the honest keys than the match bit itself.

use crate::authenticated::{auth_sub, AuthenticatedShare};
use crate::backend::SecurityDescriptor;
use crate::field::Fp;
use crate::join::{canonicalize_rows, HiddenKeyedRows};
use crate::partial::{HolderId, MpcError, PartialResult};
use crate::shamir::{MacSession, ShamirBackend};
use oxrdf::Term;

/// `n >= 2t+1` is required so the equality's authenticated multiplication has a
/// degree reduction to run and the MAC-check's honest-majority argument holds.
/// Mirrors [`crate::auth_compare`]'s fail-closed check, with the join's message.
fn check_party_count(n: usize, t: usize) -> Result<(), MpcError> {
    if n < 2 * t + 1 {
        return Err(MpcError::Protocol(format!(
            "authenticated hidden equality needs n >= 2t+1 (the authenticated multiplication's \
             degree reduction does, and the IT-MAC check is honest-majority); got n = {n}, t = {t}"
        )));
    }
    Ok(())
}

/// The authenticated masked difference `[[m]] = [[a − b]] · [[r]]` for ONE
/// candidate pair — the authenticated core of the equality test, and the only
/// place this module spends a multiplication.
///
/// `[[d]] = [[a]] − [[b]]` is a FREE local linear op (the MAC is linear in the
/// value, §2.3). The mask `[[r]]` is drawn **fresh per pair** from the session's
/// masking RNG: reusing one mask across pairs would make the opened
/// `m_i / m_j = d_i / d_j` a ratio of key differences — a leak strictly worse than
/// the match bit. The mask is drawn NONZERO ([`crate::shamir`]'s
/// `draw_nonzero_fp`), which is what makes `m == 0 ⇔ a == b`.
///
/// **Operand order:** `d` FIRST, `r` SECOND — see the module docs. Swapping them
/// makes a deviation on the key difference adopted-not-caught, and a matching pair
/// can then be flipped for free.
fn auth_masked_difference(
    session: &mut MacSession<'_>,
    a: &AuthenticatedShare,
    b: &AuthenticatedShare,
) -> Result<AuthenticatedShare, MpcError> {
    masked_zero_test(session, &auth_sub(a, b)?)
}

/// The masked zero-test `[[m]] = [[d]] · [[r]]` over an ALREADY-FORMED authenticated
/// difference — split out of [`auth_masked_difference`] so the operand-order
/// invariant is testable against THIS body: the ordering witness
/// (`mask_must_be_the_second_operand_or_a_matching_pair_can_be_flipped`) feeds a
/// deviating party's tampered `[[d]]` straight into the production call, so swapping
/// the operands here turns that test RED rather than leaving the invariant pinned
/// only against a hand-rolled copy.
fn masked_zero_test(
    session: &mut MacSession<'_>,
    d: &AuthenticatedShare,
) -> Result<AuthenticatedShare, MpcError> {
    let mask = session.draw_nonzero_fp();
    let r = session.authenticated_share(mask);
    // `d` FIRST, `r` SECOND — load-bearing; see the module docs.
    session.auth_mul(d, &r)
}

/// **EXPERIMENTAL tamper-evident secure equality over two SECRET keys**, opening
/// only the match bit — and only after the batched IT-MAC check passes.
///
/// The IT-MAC twin of [`crate::join::HiddenValueJoin`]'s private `secure_equal`. Returns
/// `a == b` without ever reconstructing `a`, `b`, or their difference: the ONLY
/// value opened is the masked product `m = (a − b)·r` (uniform nonzero for
/// unequal keys), and it is opened by [`MacSession::mac_check_and_open`], which
/// verifies the MAC first and hands back only what it verified. A forged product
/// share, a wrong `degree_reduce` re-sharing in either reduce (value or MAC), a
/// coordinated deviation in both, or a tampered open all yield
/// [`MpcError::MacCheckFailed`] — **at the minimal `n = 2t+1`**, where the
/// semi-honest twin silently returns the wrong bit.
///
/// The cleartext `a`/`b` are passed here ONLY because this routine plays ALL
/// parties in one process; they are secret-shared (authenticated) internally and
/// never used after sharing, exactly as the dealer that shares them does.
///
/// `n >= 2t+1` is fail-closed. Honest-majority, tamper-evident-with-abort against
/// the deviations exercised by this module's tests, relative to the trusted-dealer
/// mask assumption. This is NOT an audited malicious-security guarantee — read the
/// module docs' banner and "What this does NOT close" before relying on the tier
/// (external sign-off is pending, `sq-qhy4`). `[OPUS-5]`
pub fn experimental_tamper_evident_secure_equal(
    backend: &ShamirBackend,
    a: Fp,
    b: Fp,
) -> Result<bool, MpcError> {
    check_party_count(backend.parties(), backend.threshold())?;

    let mut dealer = backend.dealer();
    let mut session = dealer.new_mac_session();

    let a_auth = session.authenticated_share(a);
    let b_auth = session.authenticated_share(b);
    let m = auth_masked_difference(&mut session, &a_auth, &b_auth)?;

    Ok(checked_match_bits(&mut session, std::slice::from_ref(&m))?[0])
}

/// Turn a batch of authenticated masked products into match bits — **the one place
/// this module decides anything**, and therefore the one place the check-before-act
/// discipline has to hold.
///
/// [`MacSession::mac_check_and_open`] runs the §2.5 batched check and returns the
/// very values it authenticated, so the bits acted on cannot drift from the bits
/// verified; on `σ ≠ 0` it returns `Err` and NO value, so a tampered batch never
/// becomes a match decision (the coZK-2025/1026 check-then-act discipline). Both
/// public entry points route through here, and
/// `a_tampered_batch_never_becomes_a_match_decision` pins it: replacing this with a
/// bare open turns that test red.
fn checked_match_bits(
    session: &mut MacSession<'_>,
    products: &[AuthenticatedShare],
) -> Result<Vec<bool>, MpcError> {
    Ok(session
        .mac_check_and_open(products)?
        .into_iter()
        .map(|m| m == Fp::zero())
        .collect())
}

/// **EXPERIMENTAL tamper-evident hidden-value equi-join** over two holders'
/// PRIVATE join keys — the IT-MAC twin of [`crate::join::HiddenValueJoin::join`],
/// and the `EqualityJoin` promotion this bead exists to deliver. NOT an audited
/// malicious-security guarantee (module banner; `sq-qhy4` pending).
///
/// Output schema and rows are identical to the semi-honest twin (the key is never
/// projected; the disclosed payload columns of matching pairs are emitted, then
/// canonicalised into an order-independent multiset), so the two are differentially
/// comparable against a plaintext join. What changes is the INTEGRITY of each match
/// decision, not the answer on an honest run.
///
/// ## One `σ` open for the WHOLE cross-product (the §5 amortisation)
///
/// Every candidate pair's masked product is authenticated and accumulated, and the
/// entire `|L|·|R|` batch is verified by a **single** batched MAC-check before ANY
/// match bit is read. The marginal round cost of the IT-MAC upgrade is therefore
/// `O(1)` per join, not `O(1)` per pair — checking per pair would forfeit exactly
/// that. Pinned by `one_sigma_open_amortises_the_whole_cross_product`.
///
/// **Cost, honestly.** Each pair spends one [`MacSession::auth_mul`] — two
/// `mul_shares_raw` + two `degree_reduce` rounds against the semi-honest twin's
/// one product and one degree-`2t` open (design §5: ~2× the multiplication work,
/// plus ~2× the share volume for the MAC halves). Batching for one `σ` also means
/// the whole cross-product's authenticated sharings are held at once —
/// `O(|L|·|R|·n)` field elements — so the all-pairs shape is a memory cost here
/// as well as a time cost. The shape is quadratic in the row counts and this is a
/// batch-scale, interactive-latency-hostile operation: **do not put it behind a
/// synchronous query path, and do not extrapolate from small inputs.** No
/// row-count or wall-clock envelope is quoted here — there is no canonical
/// benchmark artifact for this path, and any figure written into a doc-comment
/// would go stale unmeasured.
///
/// ## Leakage is UNCHANGED (this is an integrity upgrade only)
///
/// One match bit per pair is still opened, so the driver still learns the full
/// bipartite match graph (**L2**) and the true result cardinality (**L1**),
/// exactly as the semi-honest twin does. Nothing here narrows the leakage tiers;
/// see [`crate::join::HiddenValueJoin::fully_oblivious_batched_join`] for the
/// never-opened-bit (but semi-honest) tier and the module docs for why the two
/// axes are orthogonal.
///
/// `n >= 2t+1` is fail-closed; an empty cross-product runs no crypto and spends no
/// `σ` open. `[OPUS-5]`
pub fn experimental_tamper_evident_hidden_join(
    backend: &ShamirBackend,
    left: &HiddenKeyedRows,
    right: &HiddenKeyedRows,
) -> Result<PartialResult, MpcError> {
    check_party_count(backend.parties(), backend.threshold())?;

    let mut out_vars = left.payload_vars.clone();
    out_vars.extend(right.payload_vars.iter().cloned());

    let mut dealer = backend.dealer();
    let mut session = dealer.new_mac_session();

    // Each key is authenticated ONCE and reused across the pairs it appears in —
    // re-sharing a key per pair would buy nothing (the per-pair freshness that is
    // load-bearing is the MASK, drawn inside `auth_masked_difference`).
    let left_keys: Vec<AuthenticatedShare> = left
        .rows
        .iter()
        .map(|(k, _)| session.authenticated_share(*k))
        .collect();
    let right_keys: Vec<AuthenticatedShare> = right
        .rows
        .iter()
        .map(|(k, _)| session.authenticated_share(*k))
        .collect();

    let mut products = Vec::with_capacity(left_keys.len() * right_keys.len());
    for lk in &left_keys {
        for rk in &right_keys {
            products.push(auth_masked_difference(&mut session, lk, rk)?);
        }
    }

    // ONE batched check for the whole cross-product, BEFORE any match bit is read.
    // On `σ ≠ 0` this returns `Err` and no value, so a tampered join never yields a
    // partial (or a partially-correct) result set.
    let matched = checked_match_bits(&mut session, &products)?;

    let mut out_rows: Vec<Vec<Option<Term>>> = Vec::new();
    let mut k = 0usize;
    for (_, lpay) in &left.rows {
        for (_, rpay) in &right.rows {
            if matched[k] {
                let mut merged = lpay.clone();
                merged.extend(rpay.iter().cloned());
                out_rows.push(merged);
            }
            k += 1;
        }
    }
    canonicalize_rows(&mut out_rows);
    Ok(PartialResult {
        holder: HolderId::new("federation"),
        vars: out_vars,
        rows: out_rows,
    })
}

/// The three-axis [`SecurityDescriptor`] this module's equality/join path delivers
/// at `backend`'s `(n, t)` — **the sq-km34 promotion, reported rather than
/// asserted in prose**.
///
/// It is deliberately a function on THIS module and not a change to
/// [`ShamirBackend::operator_descriptor`]`(OperatorClass::EqualityJoin)`: that
/// method describes the **semi-honest** [`crate::join::HiddenValueJoin`] path,
/// which is still `SemiHonestOnly` at `n = 2t+1` and must keep saying so. The
/// promotion belongs to the code that carries the MACs, so a federation reads the
/// tier off the path it actually runs. The contrast is pinned by
/// `equality_join_promotion_is_reported_only_for_the_authenticated_path`.
///
/// **The promotion is on AXIS-2 ONLY, and that is deliberate.** The descriptor
/// reports `SemiHonest` + `Abort(Unanimous)`: detect-and-abort where the
/// unauthenticated path detects nothing, WITHOUT asserting AXIS-1
/// [`crate::backend::AdversaryModel::Malicious`]. Promoting the adversary axis
/// would publish an unaudited malicious-security claim through the API, and the
/// external accredited-cryptographer review that would license it is an unresolved
/// release gate (`sq-qhy4`); the in-process tamper tests below evidence
/// tamper-EVIDENCE on the deviations they exercise, not the theorem. This mirrors
/// [`SecurityDescriptor::shamir_degree_recon`], which reports its RS-checked
/// hardening the same way ("the active-security hardening is the guarantee axis").
/// AXIS-1 moves when `sq-qhy4` closes, not before — pinned by
/// `the_adversary_axis_is_not_promoted_before_external_sign_off`.
///
/// Projects (via [`SecurityDescriptor::malicious_security`]) to
/// [`crate::backend::MaliciousSecurity::HonestMajorityAbort`] under an honest
/// majority — the `SemiHonestOnly → Abort` promotion the design record asks for —
/// and fails closed to [`SecurityDescriptor::semi_honest_only`] when `n ≤ 2t`,
/// where this module's entry points refuse to run at all. `[OPUS-5]`
pub fn equality_join_security(backend: &ShamirBackend) -> SecurityDescriptor {
    SecurityDescriptor::authenticated_abort(backend.parties(), backend.threshold())
}

#[cfg(test)]
mod tests {
    //! sq-km34 acceptance (design §6 step 5). The load-bearing ones:
    //!   1. differential parity with the plaintext join at the minimal `n = 2t+1`;
    //!   2. the PROMOTION — the in-reduce deviation that the semi-honest path at
    //!      `n = 2t+1` cannot even in principle detect now ABORTS;
    //!   3. the amortisation — one `σ` open for the whole cross-product;
    //!   4. the two HONEST BOUNDARIES — the adopted-mask (`r = 0`) hole, and the
    //!      operand order that keeps the adoption harmless.
    use super::*;
    use crate::backend::{AdversaryModel, MaliciousSecurity, OperatorClass, OutputGuarantee};
    use crate::join::HiddenValueJoin;
    use crate::shamir::{MacCarry, ShamirBackend, Share};
    use oxrdf::{Literal, Variable};

    fn fp(v: u64) -> Fp {
        Fp::new(v)
    }

    /// `n = 3, t = 1` — the MINIMAL honest-majority party count, `n = 2t+1`, where
    /// the degree-`2t` open has zero RS redundancy. Every claim in this bead is
    /// about exactly this configuration.
    fn minimal_backend(seed: u64) -> ShamirBackend {
        let b = ShamirBackend::new_seeded(3, seed).unwrap();
        assert_eq!(b.parties(), 2 * b.threshold() + 1, "the minimal n = 2t+1");
        b
    }

    fn rows(holder: &str, keys: &[(u64, &str)]) -> HiddenKeyedRows {
        HiddenKeyedRows {
            holder: HolderId::new(holder),
            payload_vars: vec![Variable::new("v").unwrap()],
            rows: keys
                .iter()
                .map(|(k, v)| {
                    (
                        fp(*k),
                        vec![Some(Term::Literal(Literal::new_simple_literal(*v)))],
                    )
                })
                .collect(),
        }
    }

    #[test]
    fn tamper_evident_secure_equal_is_correct_at_the_minimal_party_count() {
        let backend = minimal_backend(0xA11CE);
        assert!(experimental_tamper_evident_secure_equal(&backend, fp(12345), fp(12345)).unwrap());
        assert!(!experimental_tamper_evident_secure_equal(&backend, fp(12345), fp(12346)).unwrap());
        assert!(
            experimental_tamper_evident_secure_equal(&backend, Fp::zero(), Fp::zero()).unwrap()
        );
        assert!(!experimental_tamper_evident_secure_equal(&backend, Fp::zero(), fp(1)).unwrap());
    }

    #[test]
    fn tamper_evident_secure_equal_agrees_with_the_semi_honest_twin_over_many_pairs() {
        let backend = minimal_backend(0xB0B);
        let join = HiddenValueJoin::new(backend.clone());
        for a in 0..7u64 {
            for b in 0..7u64 {
                let authenticated =
                    experimental_tamper_evident_secure_equal(&backend, fp(a * 31), fp(b * 31))
                        .unwrap();
                // The semi-honest twin's scalar entry is private; drive it through the
                // one-row-each join, whose output is non-empty iff the keys matched.
                let semi = join
                    .join(&rows("l", &[(a * 31, "x")]), &rows("r", &[(b * 31, "y")]))
                    .unwrap();
                assert_eq!(
                    authenticated,
                    a == b,
                    "the authenticated equality must equal plaintext equality for ({a}, {b})"
                );
                assert_eq!(
                    authenticated,
                    !semi.rows.is_empty(),
                    "authenticated and semi-honest equality must agree for ({a}, {b})"
                );
            }
        }
    }

    /// ACCEPTANCE (design §6 step 5, "differential parity with plaintext join
    /// preserved"): on an HONEST run the authenticated join returns exactly the
    /// semi-honest join's answer, which is exactly the plaintext join's answer.
    #[test]
    fn tamper_evident_hidden_join_matches_the_plaintext_and_semi_honest_joins() {
        let backend = minimal_backend(0xC0FFEE);
        let left = rows("l", &[(10, "a"), (20, "b"), (30, "c"), (20, "b2")]);
        let right = rows("r", &[(20, "p"), (30, "q"), (40, "r"), (20, "s")]);

        let authenticated =
            experimental_tamper_evident_hidden_join(&backend, &left, &right).unwrap();
        let semi = HiddenValueJoin::new(backend.clone())
            .join(&left, &right)
            .unwrap();

        // Plaintext reference: the same all-pairs match on cleartext keys.
        let mut plaintext: Vec<Vec<Option<Term>>> = Vec::new();
        for (lk, lpay) in &left.rows {
            for (rk, rpay) in &right.rows {
                if lk == rk {
                    let mut merged = lpay.clone();
                    merged.extend(rpay.iter().cloned());
                    plaintext.push(merged);
                }
            }
        }
        canonicalize_rows(&mut plaintext);

        assert_eq!(authenticated.vars, semi.vars, "identical output schema");
        assert_eq!(
            authenticated.rows, plaintext,
            "the authenticated join must equal the plaintext join on an honest run"
        );
        assert_eq!(
            authenticated.rows, semi.rows,
            "the authenticated join must equal the semi-honest join on an honest run"
        );
        assert_eq!(authenticated.rows.len(), 5, "2·2 on key 20 + 1 on key 30");
    }

    #[test]
    fn empty_input_joins_to_empty_and_spends_no_crypto() {
        let backend = minimal_backend(0xE117);
        let empty = rows("l", &[]);
        let right = rows("r", &[(1, "p")]);
        let out = experimental_tamper_evident_hidden_join(&backend, &empty, &right).unwrap();
        assert!(out.rows.is_empty());
    }

    /// **THE sq-km34 PROMOTION (acceptance 2).** The genuine Hole-2 deviation — a
    /// party re-shares `h_i + δ` INSIDE the equality multiplication's value
    /// degree-reduce — produces a *perfectly consistent* degree-`t` sharing of a
    /// WRONG product. Reed–Solomon cannot see it at ANY `n` (there is no off-curve
    /// point), and at `n = 2t+1` the semi-honest degree-`2t` open has no redundancy
    /// either. The IT-MAC catches it: the MAC is carried by an INDEPENDENT reduce
    /// over `[α·d]·[r]` that never saw `δ`, so `σ = −λδ·α ≠ 0` and the check ABORTS
    /// with [`MpcError::MacCheckFailed`].
    ///
    /// We reproduce `experimental_tamper_evident_secure_equal`'s exact body, substituting only the
    /// tampering `auth_mul` — the same idiom `adversarial_tests` uses to model a
    /// deviating party against a primitive whose production path offers no hook.
    #[test]
    fn in_reduce_deviation_in_the_equality_multiplication_aborts_at_n_eq_2t_plus_1() {
        let backend = minimal_backend(0xDEAD);
        let t = backend.threshold();
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();

        // Two EQUAL keys: the honest verdict is "match". A deviating party wants to
        // shift the product off zero and turn it into a NON-match.
        let a = session.authenticated_share(fp(42));
        let b = session.authenticated_share(fp(42));
        let d = auth_sub(&a, &b).unwrap();
        let mask = session.draw_nonzero_fp();
        let r = session.authenticated_share(mask);

        let tampered = session
            .auth_mul_with_value_reduce_tamper_for_test(
                &d,
                &r,
                0,
                fp(7),
                MacCarry::SoundIndependentReduce,
            )
            .unwrap();

        // The tampered product is a CONSISTENT degree-t sharing of a wrong value —
        // the open itself sees nothing wrong (this is what RS cannot catch)...
        let opened_raw = backend.reconstruct(tampered.value_shares()).unwrap();
        assert_ne!(
            opened_raw,
            Fp::zero(),
            "the deviation did shift the product off zero, i.e. it WOULD have flipped \
             this matching pair into a non-match on the unauthenticated path"
        );

        // ...but the MAC-check fires, and hands back NO value.
        let err = session
            .mac_check_and_open(std::slice::from_ref(&tampered))
            .unwrap_err();
        assert!(
            matches!(err, MpcError::MacCheckFailed { .. }),
            "sq-km34: an in-reduce deviation in the equality multiplication must ABORT at \
             n = 2t+1 (t = {t}); got {err:?}"
        );
    }

    /// The coordinated variant: a party deviates inside BOTH degree-reduces of the
    /// equality multiplication, choosing the two deviations jointly. `σ = δ_m −
    /// α·δ_v` is zero only by guessing the secret `α`, so this aborts too.
    #[test]
    fn coordinated_deviation_in_both_reduces_of_the_equality_aborts() {
        let backend = minimal_backend(0xBEEF);
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();

        let a = session.authenticated_share(fp(1));
        let b = session.authenticated_share(fp(2));
        let d = auth_sub(&a, &b).unwrap();
        let mask = session.draw_nonzero_fp();
        let r = session.authenticated_share(mask);

        let tampered = session
            .auth_mul_with_both_reduces_tampered_for_test(&d, &r, 0, fp(5), fp(9))
            .unwrap();
        let err = session
            .mac_check_and_open(std::slice::from_ref(&tampered))
            .unwrap_err();
        assert!(
            matches!(err, MpcError::MacCheckFailed { .. }),
            "a coordinated both-reduce deviation must abort; got {err:?}"
        );
    }

    /// A tampered masked product at the OPEN — Hole 1, and the sharpest form of the
    /// sq-km34 promotion. The deviation shifts the opened value to EXACTLY zero, so
    /// an unequal pair would report a **false match**; and because every share moves
    /// by the same `δ` the result is a *perfectly consistent* degree-`t` codeword,
    /// so the robust reconstruction has nothing to flag — the IT-MAC is the SOLE
    /// detector, which is the whole point at `n = 2t+1`.
    #[test]
    fn tampered_product_open_aborts_instead_of_flipping_the_verdict_to_a_false_match() {
        let backend = minimal_backend(0xF00D);
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();

        // Unequal keys: the honest verdict is "no match" (m != 0).
        let a = session.authenticated_share(fp(100));
        let b = session.authenticated_share(fp(200));
        let honest = auth_masked_difference(&mut session, &a, &b).unwrap();

        // `true_m` is knowable only because one process plays all parties; the
        // deviating party shifts EVERY share by −m so the open lands on 0.
        let true_m = backend.reconstruct(honest.value_shares()).unwrap();
        assert_ne!(true_m, Fp::zero(), "sanity: unequal keys give m != 0");
        let forged_value: Vec<Share> = honest
            .value_shares()
            .iter()
            .map(|s| Share {
                x: s.x,
                y: s.y.sub(true_m),
            })
            .collect();
        assert_eq!(
            backend.reconstruct(&forged_value).unwrap(),
            Fp::zero(),
            "the tamper is a consistent degree-t codeword opening to 0 — a FALSE MATCH \
             that the robust open cannot flag"
        );
        let forged =
            AuthenticatedShare::new(forged_value, honest.mac_shares().to_vec()).unwrap();

        let err = session
            .mac_check_and_open(std::slice::from_ref(&forged))
            .unwrap_err();
        assert!(
            matches!(err, MpcError::MacCheckFailed { .. }),
            "sq-km34: a tampered product open must ABORT at n = 2t+1, never silently \
             flip the verdict to a false match; got {err:?}"
        );
    }

    /// **THE CHECK-BEFORE-ACT PIN (coZK-2025/1026 discipline).** Both public entry
    /// points turn products into match bits through `checked_match_bits`, so this
    /// pins the one place the decision is made: a batch containing ONE tampered
    /// product yields `Err` and NO bits at all — not a partially-correct verdict
    /// list, and not the honest bits for the untampered members. Replacing the
    /// `mac_check_and_open` in `checked_match_bits` with a bare open turns this red.
    #[test]
    fn a_tampered_batch_never_becomes_a_match_decision() {
        let backend = minimal_backend(0xACE);
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();

        // Three honest pairs (match, non-match, match) — these alone decide cleanly.
        let keys = [(fp(5), fp(5)), (fp(5), fp(6)), (fp(9), fp(9))];
        let honest: Vec<AuthenticatedShare> = keys
            .iter()
            .map(|(a, b)| {
                let a = session.authenticated_share(*a);
                let b = session.authenticated_share(*b);
                auth_masked_difference(&mut session, &a, &b).unwrap()
            })
            .collect();
        assert_eq!(
            checked_match_bits(&mut session, &honest).unwrap(),
            vec![true, false, true],
            "sanity: the honest batch decides correctly"
        );

        // Now tamper ONE member of the batch. The whole batch must fail.
        let mut tampered = Vec::new();
        for (i, p) in honest.iter().enumerate() {
            tampered.push(if i == 1 {
                shift_value(p, fp(1))
            } else {
                AuthenticatedShare::new(p.value_shares().to_vec(), p.mac_shares().to_vec()).unwrap()
            });
        }
        let err = checked_match_bits(&mut session, &tampered).unwrap_err();
        assert!(
            matches!(err, MpcError::MacCheckFailed { .. }),
            "one tampered product must abort the WHOLE batch with no bits returned; got {err:?}"
        );
    }

    /// **ACCEPTANCE (amortisation, design §5).** The IT-MAC upgrade costs ONE
    /// `σ` open for the WHOLE `|L|·|R|` cross-product, not one per pair — measured
    /// against the real code path, not asserted in prose.
    #[test]
    fn one_sigma_open_amortises_the_whole_cross_product() {
        let backend = minimal_backend(0x5161A);
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();

        let lefts: Vec<_> = (0..4u64)
            .map(|k| session.authenticated_share(fp(k)))
            .collect();
        let rights: Vec<_> = (0..5u64)
            .map(|k| session.authenticated_share(fp(k)))
            .collect();
        let mut products = Vec::new();
        for l in &lefts {
            for r in &rights {
                products.push(auth_masked_difference(&mut session, l, r).unwrap());
            }
        }
        assert_eq!(products.len(), 20, "4 × 5 candidate pairs");
        assert_eq!(session.sigma_opens_for_test(), 0, "no check run yet");

        let opened = session.mac_check_and_open(&products).unwrap();
        assert_eq!(opened.len(), 20, "every pair's value is returned by the check");
        assert_eq!(
            session.sigma_opens_for_test(),
            1,
            "ONE sigma open authenticates all 20 equality opens (the §5 amortisation); \
             per-pair checking would have spent 20"
        );
        // ...and the verdicts are the plaintext ones (4 diagonal matches: k = 0..3).
        let matches = opened.iter().filter(|m| **m == Fp::zero()).count();
        assert_eq!(matches, 4);
    }

    /// **HONEST BOUNDARY (the `r = 0` threat, `crate::randomness` module docs).**
    /// The MAC authenticates that `m` equals `d·r` for the `r` that was shared — it
    /// does NOT authenticate `r ≠ 0`. A mask forced to zero is therefore ADOPTED by
    /// the MAC carry (`[α·z] = reduce([α·d]·[r])` sees the same zero), `σ = 0`, and
    /// EVERY pair opens `m = 0` — a false match on every row with no abort.
    ///
    /// This is exhibited, not merely documented, so the tier claim stays bounded:
    /// the production entry points draw the mask nonzero from the session's own
    /// dealer and no compute party contributes to it, so this state is unreachable
    /// through them — the guarantee rests on that trusted-dealer assumption, and a
    /// dealer-less randomness source needs an authenticated nonzero test before it
    /// inherits this module's tier (follow-on; `research/mpc-distributed-randomness-design.md`).
    #[test]
    fn a_zeroed_mask_is_adopted_by_the_mac_and_flips_every_verdict() {
        let backend = minimal_backend(0x2E80);
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();

        // Unequal keys — the honest verdict is "no match".
        let a = session.authenticated_share(fp(7));
        let b = session.authenticated_share(fp(9));
        let d = auth_sub(&a, &b).unwrap();

        // The mask is a sharing of ZERO instead of a nonzero draw.
        let zero_mask = session.authenticated_share(Fp::zero());
        let m = session.auth_mul(&d, &zero_mask).unwrap();

        let opened = session
            .mac_check_and_open(std::slice::from_ref(&m))
            .expect("WITNESS: the MAC-check PASSES on a zeroed mask — it authenticates m = d·r, not r != 0");
        assert_eq!(
            opened[0],
            Fp::zero(),
            "WITNESS: a zeroed mask opens m = 0 for UNEQUAL keys, i.e. a false match, \
             and the IT-MAC does not catch it (crate::randomness `r = 0` threat)"
        );
    }

    /// Shift an authenticated sharing's VALUE secret by `delta`, leaving its MAC
    /// alone — a deviating party submitting a wrong INPUT sharing to a
    /// multiplication. Shifting every share by the same `delta` keeps a perfectly
    /// consistent degree-`t` codeword (nothing off-curve for RS to see); only the
    /// value/MAC RELATION is broken, which is precisely what the §2.5 check tests.
    /// This is the deviation `auth_mul` treats asymmetrically between its operands —
    /// distinct from the in-reduce deviation of
    /// [`MacSession::auth_mul_with_value_reduce_tamper_for_test`], which is caught in
    /// EITHER slot because the MAC reduce never sees it.
    fn shift_value(av: &AuthenticatedShare, delta: Fp) -> AuthenticatedShare {
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

    /// **HONEST BOUNDARY / the operand-order invariant.** `auth_mul` carries
    /// `[α·z] = reduce([α·x]·[y])`, so a wrong INPUT VALUE sharing is CAUGHT in the
    /// first slot (the MAC still holds the true `α·x`) and **ADOPTED** in the second
    /// (the MAC is recomputed through the tampered value). With the production order
    /// `[[d]] · [[r]]` the adoption lands on the mask, where flipping a verdict would
    /// still need `r' = 0` — i.e. guessing the uniform secret mask. With the order
    /// REVERSED it lands on the key difference, and shifting a MATCHING pair's
    /// `d = 0` off zero needs no secret at all: a free false non-match with `σ = 0`.
    ///
    /// Both halves are exhibited so the ordering in `auth_masked_difference` cannot
    /// be "simplified" away silently.
    #[test]
    fn mask_must_be_the_second_operand_or_a_matching_pair_can_be_flipped() {
        let backend = minimal_backend(0x0DDE);
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();

        // Two EQUAL keys → the honest verdict is "match" (d = 0).
        let a = session.authenticated_share(fp(55));
        let b = session.authenticated_share(fp(55));
        let d = auth_sub(&a, &b).unwrap();
        let mask = session.draw_nonzero_fp();
        let r = session.authenticated_share(mask);

        // A deviating party submits `d + 3` instead of `d`, trying to turn this
        // matching pair into a non-match.
        let tampered_d = shift_value(&d, fp(3));

        // THE PRODUCTION BODY (`masked_zero_test`, difference FIRST): the deviation
        // lands in the checked slot and ABORTS. Swapping the operands in that
        // function turns this assertion red.
        let prod_order = masked_zero_test(&mut session, &tampered_d).unwrap();
        let err = session
            .mac_check_and_open(std::slice::from_ref(&prod_order))
            .unwrap_err();
        assert!(
            matches!(err, MpcError::MacCheckFailed { .. }),
            "difference-first: a wrong key-difference sharing must ABORT; got {err:?}"
        );

        // REVERSED order (mask FIRST): the SAME deviation now lands in the ADOPTED
        // second-operand slot, so the MAC tracks it, `σ = 0`, and the matching pair
        // silently becomes a NON-match — a wrong verdict with no abort.
        let reversed = session.auth_mul(&r, &tampered_d).unwrap();
        let opened = session
            .mac_check_and_open(std::slice::from_ref(&reversed))
            .expect("WITNESS: reversed order — the MAC-check PASSES on the tampered product");
        assert_ne!(
            opened[0],
            Fp::zero(),
            "WITNESS: reversed order turns a MATCHING pair into a non-match with no abort; \
             the production order (difference first, mask second) is load-bearing"
        );
    }

    /// The production-order corollary, stated on its own so it is not buried in the
    /// ordering witness: a deviating party that submits a WRONG SHARING OF EITHER KEY
    /// — the "inconsistent input" deviation, invisible to RS because the codeword is
    /// consistent — aborts at the minimal `n = 2t+1`, in both directions (turning a
    /// match into a non-match and a non-match into a match).
    #[test]
    fn a_wrong_key_sharing_aborts_in_both_directions() {
        let backend = minimal_backend(0x1CE);
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();
        let mask = session.draw_nonzero_fp();
        let r = session.authenticated_share(mask);

        // (a) match → non-match: shift one key of an EQUAL pair.
        let a = session.authenticated_share(fp(77));
        let b = session.authenticated_share(fp(77));
        let d = auth_sub(&shift_value(&a, fp(1)), &b).unwrap();
        let m = session.auth_mul(&d, &r).unwrap();
        assert!(
            matches!(
                session.mac_check_and_open(std::slice::from_ref(&m)),
                Err(MpcError::MacCheckFailed { .. })
            ),
            "a wrong LEFT key sharing must abort, not report a false non-match"
        );

        // (b) non-match → match: shift one key of an UNEQUAL pair onto the other.
        let a = session.authenticated_share(fp(10));
        let b = session.authenticated_share(fp(12));
        let d = auth_sub(&shift_value(&a, fp(2)), &b).unwrap();
        let m = session.auth_mul(&d, &r).unwrap();
        assert!(
            matches!(
                session.mac_check_and_open(std::slice::from_ref(&m)),
                Err(MpcError::MacCheckFailed { .. })
            ),
            "a wrong LEFT key sharing must abort, not report a false MATCH"
        );
    }

    /// **ACCEPTANCE (design §6 step 5, the reporting half).** The promotion is
    /// reported for the AUTHENTICATED path only: `SemiHonestOnly → Abort` at the
    /// minimal `n = 2t+1`, while `ShamirBackend::operator_descriptor` keeps telling
    /// the truth about the semi-honest `HiddenValueJoin` path it describes. The
    /// promotion is on AXIS-2 only — see
    /// `the_adversary_axis_is_not_promoted_before_external_sign_off`.
    #[test]
    fn equality_join_promotion_is_reported_only_for_the_authenticated_path() {
        let backend = minimal_backend(0x5EC);

        // The semi-honest path at n = 2t+1: no RS redundancy at degree 2t, and it
        // must keep saying so.
        let semi = backend.operator_descriptor(OperatorClass::EqualityJoin);
        assert_eq!(semi.adversary, AdversaryModel::SemiHonest);
        assert_eq!(
            semi.malicious_security(0),
            MaliciousSecurity::SemiHonestOnly,
            "the semi-honest hidden join is still SemiHonestOnly at n = 2t+1"
        );

        // The authenticated path: the sq-km34 promotion, on AXIS-2.
        let auth = equality_join_security(&backend);
        assert!(
            matches!(auth.output_guarantee, OutputGuarantee::Abort(_)),
            "AXIS-2 is detect-and-abort, never GOD"
        );
        assert!(
            auth.threshold.is_honest_majority(),
            "AXIS-3 unchanged: honest majority"
        );
        assert_eq!(
            auth.malicious_security(0),
            MaliciousSecurity::HonestMajorityAbort,
            "sq-km34: EqualityJoin promotes SemiHonestOnly -> Abort at the minimal n = 2t+1"
        );
        assert!(auth.malicious_security(0).is_malicious_secure());
    }

    /// **The audit gate, pinned as a test.** `sq-qhy4` — external
    /// accredited-cryptographer sign-off — is an UNRESOLVED release gate, so no
    /// public surface of this module may report AXIS-1 `Malicious`: the in-process
    /// tamper tests above establish tamper-evidence on the deviations they
    /// exercise, not the malicious-security theorem. Flipping
    /// `SecurityDescriptor::authenticated_abort` back to `AdversaryModel::Malicious`
    /// turns this RED. Delete it only together with `sq-qhy4`.
    #[test]
    fn the_adversary_axis_is_not_promoted_before_external_sign_off() {
        for n in [3usize, 5, 7] {
            let backend = ShamirBackend::new(n).unwrap();
            let auth = equality_join_security(&backend);
            assert_eq!(
                auth.adversary,
                AdversaryModel::SemiHonest,
                "sq-qhy4 is pending: the authenticated equality/join path must NOT \
                 report AXIS-1 Malicious at n = {}",
                n
            );
            // ...while the AXIS-2 hardening it DOES evidence is still reported.
            assert!(matches!(auth.output_guarantee, OutputGuarantee::Abort(_)));
        }
    }

    /// Fail-closed: a dishonest majority (`n <= 2t`) is REFUSED by the entry points
    /// and never described as tamper-evident-with-abort by the descriptor.
    #[test]
    fn dishonest_majority_is_refused_and_never_over_claimed() {
        let backend = ShamirBackend::with_unchecked_threshold(3, 2); // n = 3 <= 2t = 4
        let err = experimental_tamper_evident_secure_equal(&backend, fp(1), fp(1)).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(_)), "got {err:?}");
        let err = experimental_tamper_evident_hidden_join(
            &backend,
            &rows("l", &[(1, "a")]),
            &rows("r", &[(1, "b")]),
        )
        .unwrap_err();
        assert!(matches!(err, MpcError::Protocol(_)), "got {err:?}");
        assert_eq!(
            equality_join_security(&backend).malicious_security(0),
            MaliciousSecurity::SemiHonestOnly,
            "the descriptor must degrade to the honest baseline under a dishonest majority"
        );
    }

    /// **The mask is FRESH PER PAIR** — the confidentiality property the batching
    /// could silently break. All `|L|·|R|` masked products are opened together by one
    /// check, so if they shared a mask `r` the opened values would satisfy
    /// `m_i / m_j = d_i / d_j`, leaking ratios of key differences — strictly more than
    /// the match bits. Two pairs with the SAME key difference must therefore open to
    /// DIFFERENT values.
    #[test]
    fn every_pair_gets_its_own_mask() {
        let backend = minimal_backend(0xFA5);
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();

        // Two pairs with the IDENTICAL difference d = -1.
        let products: Vec<AuthenticatedShare> = [(fp(10), fp(11)), (fp(400), fp(401))]
            .iter()
            .map(|(a, b)| {
                let a = session.authenticated_share(*a);
                let b = session.authenticated_share(*b);
                auth_masked_difference(&mut session, &a, &b).unwrap()
            })
            .collect();
        let opened = session.mac_check_and_open(&products).unwrap();
        assert_ne!(opened[0], Fp::zero(), "non-match");
        assert_ne!(opened[1], Fp::zero(), "non-match");
        assert_ne!(
            opened[0], opened[1],
            "equal key differences must open to DIFFERENT masked products — a shared \
             mask would make the batch leak ratios of key differences"
        );
    }

    /// The keys are never reconstructed on the authenticated path: the only value the
    /// join opens is the masked product (and the identically-zero `σ`). For unequal
    /// keys that product is neither key, nor their difference.
    #[test]
    fn only_the_masked_product_is_ever_opened() {
        let backend = minimal_backend(0x09E4);
        let mut dealer = backend.dealer();
        let mut session = dealer.new_mac_session();
        let (a, b) = (fp(1_000_003), fp(7));
        let a_auth = session.authenticated_share(a);
        let b_auth = session.authenticated_share(b);
        let m = auth_masked_difference(&mut session, &a_auth, &b_auth).unwrap();
        let opened = session.mac_check_and_open(std::slice::from_ref(&m)).unwrap();
        assert_ne!(opened[0], Fp::zero(), "unequal keys: a non-match");
        assert_ne!(opened[0], a, "the opened value is not the left key");
        assert_ne!(opened[0], b, "the opened value is not the right key");
        assert_ne!(opened[0], a.sub(b), "nor their difference");
    }
}
