// [OPUS-5] sq-km34 (the core promotion): IT-MAC on the degree-2t equality/mult open
// at the MINIMAL n = 2t+1. The masked product `m = d·r` is an AUTHENTICATED value and
// the batched IT-MAC check covers it BEFORE any verdict is derived from it (design
// §2.5 — open, then verify, then release; NOT verify-then-broadcast, see the transcript
// -order section below), and the mask `[[r]]` carries an authenticated NONZERO WITNESS
// `u = r·s` opened in the same batch — so the Hole-1 forged-share flip, the Hole-2 wrong
// re-sharing, and the Hole-3 `r = 0` false-match are all fail-closed aborts where the
// semi-honest `join::HiddenValueJoin::secure_equal` had ZERO Reed-Solomon redundancy to
// detect anything.
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
//!   ── open the whole batch, then ONE batched MAC-check over it ──
//!   abort unless σ == 0        ;   abort unless u ≠ 0   ;   verdict := (m == 0)
//! ```
//!
//! Both products are degree-`t` after the authenticated mult-then-reduce, so the open
//! is a degree-`t` open of a value carrying a MAC that the batched check then verifies —
//! not the bare, unauthenticated degree-`2t` open the semi-honest path performs. No
//! verdict is derived from a value the check has not covered.
//!
//! ### Transcript ORDER — the check does NOT precede the open (stated exactly)
//!
//! [`MacSession::mac_check_and_open`] **reconstructs every batched value first**, then
//! builds and opens `σ`, and only then returns: `Err` and *no values at all* when
//! `σ ≠ 0`. In a real distributed transport reconstruction IS the broadcast, so
//! withholding a Rust return value does not un-send a share. The accurate statement is
//! therefore **open → verify → release**, and what the IT-MAC buys is
//! *detect-and-abort on everything acted upon*: no verdict here is ever derived from an
//! open the check did not cover. It is NOT "nothing inconsistent is ever broadcast".
//!
//! Why opening before verifying is nonetheless acceptable *for this operator*, and what
//! it still leaves open:
//!
//! - The only values reconstructed per pair are `m = d·r` and `u = r·s`, each multiplied
//!   by a fresh uniform nonzero factor independent of the keys. Under a deviation the
//!   transcript carries `(d + δ_d)·(r + δ_r)` for offsets the adversary itself chose, so
//!   an honest key difference is never revealed directly — only through whether that
//!   product is zero.
//! - That residue is a **selective-failure guess**: shifting `d` by a chosen `δ` makes
//!   `m` open to `0` exactly when `d = −δ`, so a deviating party can test one guess of
//!   the key difference per execution and read the answer off the transcript *before* the
//!   unanimous abort fires. The MAC turns the deviation into an abort instead of a
//!   flipped verdict; it does not erase that one bit. We record this as a residual
//!   ((4) below) rather than claiming it absent.
//!
//! So this module closes Hole 1's **correctness** flip and enforces the design §2.5
//! "nothing is *acted on* before the check" discipline. It does **not** establish the
//! stronger verify-strictly-before-reveal ordering, and it is therefore NOT claimed to
//! close the coZK eprint 2025/1026 confidentiality interaction on its own — that would
//! need a transcript in which the opened values are validated before they are revealed
//! (commit-then-open, or a verified-reconstruction step), which `mac_check_and_open`
//! does not implement. Nothing here is externally audited (`sq-qhy4`).
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
//! 4. **The check runs after the open, so the coZK-2025/1026 interaction is MITIGATED,
//!    not closed.** Per the transcript-order section above, a deviating party still sees
//!    the reconstructed `m` before the abort, which is a one-guess selective-failure test
//!    on the key difference. Closing it needs verify-before-reveal (commitments or a
//!    verified reconstruction) in [`MacSession::mac_check_and_open`], not a change here.
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

/// Which deviation (if any) a run of [`auth_equal_verdicts_with`] injects.
///
/// Production has exactly one variant, [`Deviation::None`]. The test-only variants exist
/// so the adversarial suite drives each deviation through the **real**
/// [`auth_equal_verdicts`] body rather than a mirrored copy that could drift from it —
/// the Hole-3 `r = 0` attack in particular must be exercised against production code for
/// `consistently_zeroed_mask_is_caught_by_the_nonzero_witness` to mean anything, and the
/// two `…ReduceReshare` variants are the ONLY way to reach the Hole-2 deviation *inside*
/// this operator's multiplications (a post-hoc tamper on the finished product lands after
/// the MAC is committed and so cannot attribute the catch to the independent MAC reduce —
/// see `MacSession::auth_mul_with_value_reduce_tamper_for_test`).
#[derive(Clone, Copy)]
enum Deviation {
    /// No deviation: a fresh, uniform, NONZERO mask drawn from the session's masking
    /// CSPRNG and authenticated under the session key, and honest multiplications — the
    /// production path.
    None,
    /// **Test-only.** A *consistently authenticated* zero mask `[[r]] = ([0],[0])`, which
    /// satisfies the MAC relation `α·0 = 0` and so passes the batched check. This is the
    /// Hole-3 deviation the nonzero witness exists to catch — and the deviation
    /// [`MacSession::auth_mul`]'s second-operand adoption would otherwise let through
    /// silently, flipping every verdict to "equal".
    #[cfg(test)]
    ConsistentlyZeroedMask,
    /// **Test-only.** A fresh nonzero mask whose VALUE shares are shifted while its MAC
    /// is left alone. In the verdict gate this sits in the adopted second slot, but the
    /// witness gate consumes the same `[[r]]` in the MAC-carrying FIRST slot, so the
    /// defect propagates into `u`'s MAC and the batched check fires.
    #[cfg(test)]
    ValueShiftedMask,
    /// **Test-only.** An honest run with a CHOSEN mask instead of a drawn one, so a test
    /// can pin the deterministic algebra of the open (`m = d·r`) — including the
    /// protocol-valid `r = 1` case, where `m` legitimately equals `d` — without asserting
    /// that some valid random outcome never occurs.
    #[cfg(test)]
    ChosenMask(Fp),
    /// **Test-only, Hole 2 inside the VERDICT multiplication.** Injects `δ` into the BGW
    /// re-sharing of `[[m]] = [[d]]·[[r]]`'s VALUE reduce, so `[m]` is a *perfectly
    /// consistent* degree-`t` sharing of the WRONG product (nothing for Reed–Solomon to
    /// see). Under the production [`crate::shamir::MacCarry::SoundIndependentReduce`] the
    /// MAC reduce never saw `δ` and the batched check fires; under the rejected
    /// `UnsoundFromValueTimesAlpha` carry the MAC tracks the tampered value and a WRONG
    /// VERDICT is returned — which is what makes the abort non-vacuous.
    #[cfg(test)]
    VerdictReduceReshare(crate::shamir::MacCarry),
    /// **Test-only, Hole 2 inside the WITNESS multiplication, composed with the Hole-3
    /// zero mask.** The mask is the consistently-authenticated zero AND `δ` is injected
    /// into the re-sharing of `[[u]] = [[r]]·[[s]]`'s value reduce — so the witness opens
    /// to `λ·δ ≠ 0` instead of `0` and the value-level nonzero gate is *satisfied*. Only
    /// the MAC on that reduce stands between this composition and a false match on every
    /// pair; the unsound carry demonstrates exactly that wrong verdict.
    #[cfg(test)]
    ZeroMaskAndWitnessReduceReshare(crate::shamir::MacCarry),
    /// **Test-only, Hole 1 at THIS operator's open boundary.** After the gadget has built
    /// `[[m]]`, every share of its VALUE sharing is shifted by `1` — a consistent
    /// degree-`t` codeword of `m + 1`, so the robust open cannot flag it — while its MAC
    /// is left alone. This is the forged-product-share flip the semi-honest path suffers
    /// information-theoretically undetectably at `n = 2t+1`.
    #[cfg(test)]
    ForgedProductShareAtOpen,
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
/// verdict is derived, so a batch with one bad mask yields no verdicts at all. Note the
/// exact ordering (module docs, "Transcript ORDER"): the batch is **opened, then
/// verified**, so this is abort-before-*acting*, not check-before-broadcast.
///
/// `n >= 2t+1` is fail-closed. Honest-majority; research-grade, externally unaudited
/// (`sq-qhy4`). See the module docs for the exact residuals. `[OPUS-5]`
pub fn auth_equal_verdicts(
    session: &mut MacSession,
    pairs: &[(AuthenticatedShare, AuthenticatedShare)],
) -> Result<Vec<bool>, MpcError> {
    auth_equal_verdicts_with(session, pairs, Deviation::None)
}

/// The body of [`auth_equal_verdicts`], parameterised by the injected [`Deviation`] so
/// the adversarial tests exercise THIS code rather than a copy of it.
fn auth_equal_verdicts_with(
    session: &mut MacSession,
    pairs: &[(AuthenticatedShare, AuthenticatedShare)],
    deviation: Deviation,
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
        #[cfg_attr(not(test), allow(unused_mut))]
        let (mut m, u) = equality_gadget(session, a, b, deviation)?;
        // Hole 1 at the open boundary: forge the product's value codeword AFTER the
        // gadget committed its MAC — the deviation the semi-honest path cannot see.
        #[cfg(test)]
        if matches!(deviation, Deviation::ForgedProductShareAtOpen) {
            m = shift_value_shares(&m);
        }
        batch.push(m);
        batch.push(u);
    }

    // ONE batched IT-MAC check over the WHOLE batch, applied to exactly the values it
    // returns — `mac_check_and_open` opens first and verifies second (module docs,
    // "Transcript ORDER"), but hands back nothing at all on a failing check, so the
    // values acted on below cannot drift from the values verified, and nothing is
    // opened twice.
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
    deviation: Deviation,
) -> Result<(AuthenticatedShare, AuthenticatedShare), MpcError> {
    // d = a − b: FREE (local, both components), degree t.
    let d = auth_sub(a, b)?;
    let r = draw_mask(session, deviation);
    // s is ALWAYS an honest fresh nonzero draw: the deviations under test live on `r`,
    // and a witness whose own second operand were also corrupted would not isolate them.
    let s_val = session.draw_nonzero_fp();
    let s = session.authenticated_share(s_val);
    // m = d·r — `d` FIRST so every data-side deviation propagates into m's MAC.
    // u = r·s — `r` FIRST so a value-only tamper on the mask propagates into u's MAC,
    // and `u == 0` iff the mask (or s) is zero.
    match deviation {
        #[cfg(test)]
        Deviation::VerdictReduceReshare(carry) => {
            let m = session.auth_mul_with_value_reduce_tamper_for_test(
                &d,
                &r,
                RESHARE_PARTY,
                Fp::new(RESHARE_DELTA),
                carry,
            )?;
            let u = session.auth_mul(&r, &s)?;
            Ok((m, u))
        }
        #[cfg(test)]
        Deviation::ZeroMaskAndWitnessReduceReshare(carry) => {
            let m = session.auth_mul(&d, &r)?;
            let u = session.auth_mul_with_value_reduce_tamper_for_test(
                &r,
                &s,
                RESHARE_PARTY,
                Fp::new(RESHARE_DELTA),
                carry,
            )?;
            Ok((m, u))
        }
        _ => {
            let m = session.auth_mul(&d, &r)?;
            let u = session.auth_mul(&r, &s)?;
            Ok((m, u))
        }
    }
}

/// **Test-only.** The re-sharing party whose BGW contribution the `…ReduceReshare`
/// deviations corrupt, and the `δ` it adds. The reduced secret shifts by `λ·δ` for the
/// nonzero Lagrange-at-0 weight `λ` of that recombination point, so any nonzero `δ`
/// yields a wrong — but perfectly degree-`t` consistent — product.
#[cfg(test)]
const RESHARE_PARTY: usize = 0;
#[cfg(test)]
const RESHARE_DELTA: u64 = 123;

/// **Test-only.** Shift every share of `av`'s VALUE sharing by `1`, leaving its MAC
/// untouched: a *consistent* degree-`t` codeword of `value + 1`, which the robust open
/// accepts and only the IT-MAC can reject.
#[cfg(test)]
fn shift_value_shares(av: &AuthenticatedShare) -> AuthenticatedShare {
    let value: Vec<crate::shamir::Share> = av
        .value_shares()
        .iter()
        .map(|s| crate::shamir::Share {
            x: s.x,
            y: s.y.add(Fp::one()),
        })
        .collect();
    AuthenticatedShare::new(value, av.mac_shares().to_vec())
        .expect("shifting share values keeps the party points and length identical")
}

/// Mint a pair's mask under the requested [`Deviation`]. Every non-mask deviation (and
/// production) takes the single fresh-nonzero arm.
fn draw_mask(session: &mut MacSession, deviation: Deviation) -> AuthenticatedShare {
    match deviation {
        // A *consistently authenticated* zero: value sharing [0] and MAC sharing
        // [α·0] = [0]. No knowledge of α is needed to produce it, and it satisfies the
        // MAC relation exactly — which is the whole point of the attack.
        #[cfg(test)]
        Deviation::ConsistentlyZeroedMask | Deviation::ZeroMaskAndWitnessReduceReshare(_) => {
            session.auth_const_sharing(Fp::zero())
        }
        #[cfg(test)]
        Deviation::ValueShiftedMask => {
            let r_val = session.draw_nonzero_fp();
            let honest = session.authenticated_share(r_val);
            shift_value_shares(&honest)
        }
        // An honest sharing of a mask the TEST chose, so the algebra of the open is
        // deterministic. `r` still enters the protocol exactly as a drawn mask would.
        #[cfg(test)]
        Deviation::ChosenMask(r_val) => session.authenticated_share(r_val),
        _ => {
            let r_val = session.draw_nonzero_fp();
            session.authenticated_share(r_val)
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
    /// "forged share / inconsistent input" deviation. The same helper the
    /// [`Deviation::ForgedProductShareAtOpen`] injection uses, so the input-side and
    /// product-side tests deviate identically.
    use super::shift_value_shares as tamper_value;

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

    /// ACCEPTANCE: **an INCONSISTENT INPUT — a forged share on an input KEY, before any
    /// multiplication — aborts at the minimal `n = 2t+1`.** This is the design's Hole-3
    /// second sub-deviation, not Hole 1: the tamper lands on an input key's value
    /// sharing, which flows into `d` — the MAC-carrying FIRST operand of the verdict
    /// gate — so the defect propagates into `m`'s MAC and `σ ≠ 0`.
    ///
    /// The Hole-1 headline deviation (a forged share on the PRODUCT, at this operator's
    /// own open boundary) is a different injection point and is pinned separately by
    /// `forged_product_share_at_the_equality_open_aborts_at_minimal_n`; the in-reduce
    /// Hole-2 deviation by the two `wrong_degree_reduce_…` tests below.
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

    /// True iff `got` is the abort raised by the **batched IT-MAC check** (`σ ≠ 0`)
    /// rather than by the value-level mask witness — both are
    /// [`MpcError::MacCheckFailed`], so the tests below discriminate on the detail to
    /// prove WHICH gate fired.
    fn aborted_on_sigma<T: std::fmt::Debug>(got: &Result<T, MpcError>) -> bool {
        matches!(got, Err(MpcError::MacCheckFailed { detail }) if detail.contains("batched IT-MAC check"))
    }

    /// ACCEPTANCE (sq-km34, **THE headline Hole-1 deviation, at THIS operator's open
    /// boundary**): a forged share on the masked-difference PRODUCT — injected after the
    /// gadget has committed `m`'s MAC, exactly where the semi-honest path opens `m`
    /// undefended — aborts instead of flipping the match bit.
    ///
    /// Two-sided so it cannot pass vacuously:
    ///
    /// 1. **The attack is real and RS-invisible.** The forged value sharing is a
    ///    *consistent* degree-`t` codeword (every share shifted by the same `1`), so the
    ///    robust open accepts it and returns `m + 1`. For a pair of EQUAL keys the honest
    ///    `m` is `0`, so the semi-honest verdict rule `m == 0` would read the forged open
    ///    as **"not equal"** — a silent WRONG answer, which is precisely what
    ///    `adversarial_tests::tampered_share_in_secure_equality_open_is_undetectable_at_
    ///    n_eq_2t_plus_1` pins on the unauthenticated path.
    /// 2. **The IT-MAC catches it**, via `σ ≠ 0` (not via the witness gate — asserted),
    ///    and the honest control on the same keys still returns the true verdict.
    #[test]
    fn forged_product_share_at_the_equality_open_aborts_at_minimal_n() {
        for n in MINIMAL_N {
            let b = backend(n, 0xF7_0000 + n as u64);
            let (ka, kb) = (Fp::new(42), Fp::new(42)); // EQUAL keys → honest m = 0.

            // --- 1. the attack is real: a consistent codeword of the WRONG product ---
            {
                let mut dealer = b.dealer();
                let mut session = dealer.new_mac_session();
                let x = session.authenticated_share(ka);
                let y = session.authenticated_share(kb);
                let (m, _u) = equality_gadget(&mut session, &x, &y, Deviation::None).unwrap();
                assert_eq!(
                    b.reconstruct(m.value_shares()).unwrap(),
                    Fp::zero(),
                    "n = {n}: equal keys must give an honest masked difference of 0"
                );
                let forged = shift_value_shares(&m);
                let opened = b
                    .reconstruct(forged.value_shares())
                    .expect("the forged sharing is a CONSISTENT degree-t codeword — RS sees nothing");
                assert_ne!(
                    opened,
                    Fp::zero(),
                    "n = {n}: the forged product opens NONZERO, so the unauthenticated verdict \
                     rule `m == 0` would report `not equal` for EQUAL keys — a silent wrong answer"
                );
            }

            // --- 2. the MAC catches it, and the honest control is unaffected ---
            let mut dealer = b.dealer();
            let mut session = dealer.new_mac_session();
            let x = session.authenticated_share(ka);
            let y = session.authenticated_share(kb);
            let got = auth_equal_verdicts_with(
                &mut session,
                &[(x, y)],
                Deviation::ForgedProductShareAtOpen,
            );
            assert!(
                aborted_on_sigma(&got),
                "n = {n}: a forged product share must abort on the batched IT-MAC check at the \
                 minimal n = 2t+1, got {got:?}"
            );
            assert!(
                malicious_secure_equal(&b, ka, kb).unwrap(),
                "n = {n}: the honest control on the same EQUAL keys must report `equal`"
            );
        }
    }

    /// ACCEPTANCE (Hole 2, **inside the VERDICT multiplication**): a deviating party
    /// re-shares `h_i + δ` INSIDE the BGW re-sharing of `[[m]] = [[d]]·[[r]]`'s value
    /// reduce — producing a *perfectly consistent* degree-`t` sharing of the wrong
    /// product, with nothing for Reed–Solomon to detect — and the operator aborts.
    ///
    /// The **mutation check** is the second half: run the identical deviation with the
    /// design's REJECTED MAC carry (`[α·z] = reduce([z]·[α])`, which recomputes the MAC
    /// from the just-tampered value) and the batched check is fooled, so the operator
    /// returns a **WRONG VERDICT** — `not equal` for two EQUAL keys. The abort in the
    /// first half therefore comes from the independent MAC reduce and nothing else; if
    /// `auth_mul` were ever "simplified" to the unsound carry, this test goes red.
    #[test]
    fn wrong_degree_reduce_in_the_verdict_multiplication_aborts_at_minimal_n() {
        use crate::shamir::MacCarry;
        for n in MINIMAL_N {
            let b = backend(n, 0xF8_0000 + n as u64);
            let (ka, kb) = (Fp::new(77), Fp::new(77)); // EQUAL keys → honest verdict true.

            // Production carry: the MAC reduce never saw δ → σ ≠ 0 → ABORT.
            let mut dealer = b.dealer();
            let mut session = dealer.new_mac_session();
            let x = session.authenticated_share(ka);
            let y = session.authenticated_share(kb);
            let got = auth_equal_verdicts_with(
                &mut session,
                &[(x, y)],
                Deviation::VerdictReduceReshare(MacCarry::SoundIndependentReduce),
            );
            assert!(
                aborted_on_sigma(&got),
                "n = {n}: a wrong re-sharing inside the verdict multiplication must abort on the \
                 batched IT-MAC check, got {got:?}"
            );

            // MUTATION: the rejected carry adopts the tampered value → silent WRONG verdict.
            let mut dealer = b.dealer();
            let mut session = dealer.new_mac_session();
            let x = session.authenticated_share(ka);
            let y = session.authenticated_share(kb);
            let unsound = auth_equal_verdicts_with(
                &mut session,
                &[(x, y)],
                Deviation::VerdictReduceReshare(MacCarry::UnsoundFromValueTimesAlpha),
            );
            assert_eq!(
                unsound.expect("the unsound carry makes the tampered product MAC-consistent"),
                vec![false],
                "n = {n}: under the REJECTED MAC carry the same deviation returns `not equal` for \
                 EQUAL keys — the wrong answer the sound carry turns into an abort"
            );

            // Honest control: the same keys, no deviation, true verdict.
            assert!(
                malicious_secure_equal(&b, ka, kb).unwrap(),
                "n = {n}: the honest control on the same EQUAL keys must report `equal`"
            );
        }
    }

    /// ACCEPTANCE (Hole 2, **inside the WITNESS multiplication — composed with Hole 3**):
    /// the mask is the consistently-authenticated zero AND a deviating party re-shares
    /// `h_i + δ` inside `[[u]] = [[r]]·[[s]]`'s value reduce, so the witness opens to
    /// `λ·δ ≠ 0` and **satisfies** the value-level nonzero gate. That composition is the
    /// only route by which a zeroed mask could survive to a verdict, and it is the MAC on
    /// the witness reduce — nothing else — that stops it.
    ///
    /// The **mutation check**: with the rejected `[α·z] = reduce([z]·[α])` carry the
    /// batched check is fooled and the operator returns `equal` for two **UNEQUAL** keys
    /// — the full Hole-3 false match. Under the production carry it aborts on `σ`.
    #[test]
    fn wrong_degree_reduce_in_the_witness_multiplication_aborts_at_minimal_n() {
        use crate::shamir::MacCarry;
        for n in MINIMAL_N {
            let b = backend(n, 0xF9_0000 + n as u64);
            let (ka, kb) = (Fp::new(1_000), Fp::new(2_000)); // UNEQUAL keys.

            let mut dealer = b.dealer();
            let mut session = dealer.new_mac_session();
            let x = session.authenticated_share(ka);
            let y = session.authenticated_share(kb);
            let got = auth_equal_verdicts_with(
                &mut session,
                &[(x, y)],
                Deviation::ZeroMaskAndWitnessReduceReshare(MacCarry::SoundIndependentReduce),
            );
            assert!(
                aborted_on_sigma(&got),
                "n = {n}: a wrong re-sharing inside the witness multiplication must abort on the \
                 batched IT-MAC check (the witness value gate is SATISFIED here, u = λδ != 0), \
                 got {got:?}"
            );

            // MUTATION: the rejected carry makes the tampered witness MAC-consistent, so
            // the zeroed mask survives both gates and every verdict becomes a false match.
            let mut dealer = b.dealer();
            let mut session = dealer.new_mac_session();
            let x = session.authenticated_share(ka);
            let y = session.authenticated_share(kb);
            let unsound = auth_equal_verdicts_with(
                &mut session,
                &[(x, y)],
                Deviation::ZeroMaskAndWitnessReduceReshare(MacCarry::UnsoundFromValueTimesAlpha),
            );
            assert_eq!(
                unsound.expect("the unsound carry makes the tampered witness MAC-consistent"),
                vec![true],
                "n = {n}: under the REJECTED MAC carry the zeroed mask survives and reports \
                 `equal` for UNEQUAL keys — the Hole-3 false match"
            );

            assert!(
                !malicious_secure_equal(&b, ka, kb).unwrap(),
                "n = {n}: the honest control on the same UNEQUAL keys must report `not equal`"
            );
        }
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
                    equality_gadget(&mut session, &x, &y, Deviation::ConsistentlyZeroedMask).unwrap();
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
                auth_equal_verdicts_with(&mut session, &[(x, y)], Deviation::ConsistentlyZeroedMask);
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
                auth_equal_verdicts_with(&mut session, &[(x, y)], Deviation::ValueShiftedMask);
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

    /// ACCEPTANCE (confidentiality, the DETERMINISTIC half): what the transcript carries
    /// is exactly `m = d·r` for the mask actually used, and that value **singles out no
    /// difference at all** — for every candidate nonzero difference `d'` there is a
    /// nonzero `r'` with `d'·r' = m`, so the open is consistent with every possible key
    /// difference and reveals only whether `d` was zero.
    ///
    /// Driven through [`Deviation::ChosenMask`] rather than a drawn mask, so the algebra
    /// is checked as an identity instead of asserting that some protocol-valid random
    /// outcome cannot occur. In particular the `r = 1` case is included and `m` there
    /// **legitimately equals `d`** — a valid execution the previous form of this test
    /// wrongly treated as impossible. That coincidence is harmless precisely because the
    /// simulator argument above holds for every opened `m`, `d` included.
    #[test]
    fn the_opened_masked_difference_is_d_times_r_and_fits_every_nonzero_difference() {
        let (ka, kb) = (Fp::new(9_000), Fp::new(1_000));
        let d = ka.sub(kb);
        // r = 1 is the case where `m == d`; the others are ordinary masks.
        for (i, r_val) in [Fp::one(), Fp::new(2), Fp::new(7), Fp::new(1 << 40)]
            .into_iter()
            .enumerate()
        {
            let b = backend(3, 0xFA_0000 + i as u64);
            let mut dealer = b.dealer();
            let mut session = dealer.new_mac_session();
            let x = session.authenticated_share(ka);
            let y = session.authenticated_share(kb);
            let (m, u) =
                equality_gadget(&mut session, &x, &y, Deviation::ChosenMask(r_val)).unwrap();
            let opened = session.mac_check_and_open(&[m, u]).unwrap();

            assert_eq!(
                opened[0],
                d.mul(r_val),
                "the opened masked difference must be exactly d·r"
            );
            assert_ne!(
                opened[0],
                Fp::zero(),
                "unequal keys and a nonzero mask must open a NONZERO masked difference"
            );
            assert_ne!(
                opened[1],
                Fp::zero(),
                "an honest mask witness u = r·s is never zero (both factors are nonzero)"
            );
            if r_val == Fp::one() {
                assert_eq!(
                    opened[0], d,
                    "at r = 1 the open EQUALS the key difference — a protocol-valid execution, \
                     not a leak: see the simulator check below"
                );
            }
            // Simulator: any other nonzero difference explains the SAME open.
            for alt in [Fp::one(), Fp::new(3), Fp::new(123_456_789)] {
                let r_prime = opened[0].mul(alt.inv());
                assert_ne!(r_prime, Fp::zero(), "the explaining mask is itself nonzero");
                assert_eq!(
                    alt.mul(r_prime),
                    opened[0],
                    "difference {} explains the same opened m with mask {} — so m identifies no \
                     difference",
                    alt.value(),
                    r_prime.value()
                );
            }
        }
    }

    /// ACCEPTANCE (confidentiality, the DISTRIBUTIONAL half): with the PRODUCTION drawn
    /// mask, a FIXED unequal pair opens a different masked difference in every
    /// independent session — a constant or low-entropy `m` would leak `d`.
    ///
    /// **Explicit negligible-failure design.** The assertion is "no two of `RUNS`
    /// independent opens collide". For an ideal mask `m` is uniform over the `p − 1`
    /// nonzero residues of the 61-bit Mersenne field, so the birthday bound puts a
    /// spurious failure at `≤ RUNS·(RUNS−1)/(2(p−1)) < 2^-52`. The seeds are fixed, so
    /// this run is also reproducible; a red here means either a deliberate RNG change
    /// (re-seed and re-check) or a genuine loss of re-randomisation — not bad luck at any
    /// rate worth engineering around. The *algebraic* non-disclosure property is pinned
    /// deterministically by the test above, so nothing load-bearing rests on this bound.
    #[test]
    fn the_masked_difference_is_re_randomised_across_independent_sessions() {
        let (ka, kb) = (Fp::new(9_000), Fp::new(1_000));
        let mut seen = std::collections::BTreeSet::new();
        const RUNS: u64 = 32;
        for seed in 0..RUNS {
            let b = backend(3, 0xF5_0000 + seed);
            let mut dealer = b.dealer();
            let mut session = dealer.new_mac_session();
            let x = session.authenticated_share(ka);
            let y = session.authenticated_share(kb);
            let (m, u) = equality_gadget(&mut session, &x, &y, Deviation::None).unwrap();
            let opened = session.mac_check_and_open(&[m, u]).unwrap();
            assert_ne!(
                opened[0],
                Fp::zero(),
                "unequal keys must open a NONZERO masked difference"
            );
            assert_ne!(
                opened[1],
                Fp::zero(),
                "an honest mask witness u = r·s is never zero"
            );
            seen.insert(opened[0].value());
        }
        assert_eq!(
            seen.len() as u64,
            RUNS,
            "the masked difference must be re-randomised per session (got {} distinct values over \
             {RUNS} runs; a collision is a < 2^-52 event under an ideal mask)",
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
