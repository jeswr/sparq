// [OPUS-4.8] sq-rrz4 — secure secret-shared greater-than / threshold, opening
// ONLY the boolean verdict bit (never the operands).
//! Secure comparison (`>`, threshold) over secret-shared `F_p` values that opens
//! **only the 1-bit verdict**, never the operands or their difference.
//!
//! Architecture refs: §4.3 step 4 (the secure computation over secret-shared
//! per-source values) and the driving **four-flatmates** use case (architecture
//! §2; Wright CEUR Vol-4085): N cooperating holders prove their cumulative salary
//! `> £100k` while disclosing ONLY the boolean verdict, never the exact total.
//! `research/mpc-security-models-and-benchmarks.md` §3 names this operator class:
//! "FILTER (<, ≤, >) → Rabbit (eprint 2021/119) / edaBits (Crypto'20)
//! bit-decomposition; honest- & dishonest-majority". The capability matrix
//! (`research/mpc-sparql-capability-matrix.md` §4.2) marked it **absent / OPEN,
//! blocked on degree reduction (sq-dvuc)** and named it the keystone unblocking the
//! whole MEDIUM operator zone; this module realises the honest-majority,
//! semi-honest version now that degree reduction has landed.
//!
//! ## The gap this closes (disclosure-minimisation)
//!
//! Before this, the only way to answer `sum > £100k` was
//! [`crate::shamir::ShamirBackend::reconstruct_disclosed`] — which OPENS the exact
//! integer sum, so a verifier recomputing the threshold over a DISCLOSED aggregate
//! learns the precise total (violates disclosure-minimisation, §2 convention #4
//! read as "the minimal answer"). The missing piece is a secret-shared comparison
//! that opens ONLY the match bit. That is what [`secure_threshold`] /
//! [`secure_greater_than`] provide: the verdict is a fresh degree-`t` sharing of a
//! 0/1 value, and the operand shares are NEVER reconstructed on the verdict path.
//!
//! ## Protocol chosen: **bit-decomposition MSB-first comparison** — and WHY
//!
//! The skill / design record list two families for secure `<`: a constant-round
//! less-than (DGK / Rabbit / edaBits-style, fewer rounds, more preprocessing
//! machinery) and **bit-decomposition** (a Boolean comparison circuit over the
//! value's bits). For THIS in-process honest-majority simulation we choose
//! **bit-decomposition** because:
//!
//! - It is the one that is simplest to get *provably correct* on top of the
//!   primitives the backend already has ([`crate::shamir::mul_shares_raw`] +
//!   [`crate::shamir::ShamirDealer::degree_reduce`]); the bead explicitly notes
//!   "bit-decomposition is usually simplest to get correct here".
//! - It needs **no preprocessing** (Beaver bit-triples / edaBits) that we cannot
//!   honestly simulate in-process — every step is a Shamir multiplication chained
//!   through degree reduction, exactly the prerequisite that just landed (sq-dvuc,
//!   PR #119). A constant-round less-than would need an honestly-simulated random
//!   bit / edaBit generator we do not have, so bit-decomposition is the choice
//!   that ships *correct* rather than *fast-but-faked*.
//! - The honest cost trade-off is stated, not hidden: bit-decomposition is
//!   `O(L)` multiplication ROUNDS (`L = `[`COMPARE_BITS`]`), which is the
//!   round-per-depth profile that is fine on LAN and poor on WAN. The
//!   constant-round (Rabbit/DGK/edaBits) family is the WAN answer and is left as
//!   future work (capability matrix §4.2; bead sq-38zk for the WAN backend).
//!
//! ### The circuit (MSB-first "first differing bit decides")
//!
//! For two `L`-bit non-negative integers `a = Σ a_k 2^k`, `b = Σ b_k 2^k`, the
//! relation `a > b` is decided by the **most significant bit at which they
//! differ**: `a > b` iff at that bit `a_k = 1, b_k = 0`. Scanning MSB→LSB and
//! tracking `eq` = "all higher bits equal so far":
//!
//! ```text
//! gt = 0 ; eq = 1
//! for k from L-1 downto 0:
//!     gt = gt + eq · (a_k · (1 - b_k))    // a_k=1,b_k=0 at the first diff ⇒ a>b
//!     eq = eq · (1 - (a_k - b_k)^2)       // bits equal ⇒ (a_k-b_k)^2 = 0
//! gt  is the verdict bit
//! ```
//!
//! `a_k`, `b_k` are bits (0/1), so `(a_k - b_k)^2 ∈ {0,1}` is exactly "bits
//! differ", and `a_k·(1 - b_k)` is exactly "a_k set, b_k clear". `gt` accumulates
//! at most one `1` (the first differing bit from the top); once a higher bit
//! differs, `eq` is `0` for every lower bit, so nothing further is added. The
//! result is therefore a single 0/1: `1` iff `a > b`, else `0` (covering both
//! `a == b` and `a < b`). Each `·` on secret values is one [`shamir::mul_shares_raw`]
//! followed by one [`ShamirDealer::degree_reduce`] (the BGW reshare-and-recombine),
//! so the comparison is a genuine multiplication CHAIN of depth `O(L)` — depth > 1,
//! the thing degree reduction unlocks.
//!
//! ### Why the operands stay hidden (bit-only disclosure)
//!
//! The dealer bit-decomposes each secret value into `L` secret-shared bits — this
//! is part of *dealing* the input (the dealer already holds the cleartext to share
//! it, exactly as [`ShamirDealer::share`] / [`crate::join::HiddenValueJoin`]'s
//! `secure_equal` do). The whole circuit runs on those secret-shared bits; the
//! ONLY value ever opened is the final `gt` sharing — a single 0/1. The value
//! bits, the operands, and their difference are never reconstructed (asserted
//! structurally by the disclosure-minimisation acceptance test). When the comparand
//! is a PUBLIC threshold ([`secure_threshold`]) its bits are public CONSTANTS, so
//! the per-bit products against it are LOCAL affine maps on the secret bit sharings
//! (no extra multiplication round) — the cheaper path, and the headline £100k path.
//!
//! ## Range precondition (no field wraparound) — fail-closed
//!
//! The comparison is over the INTEGERS, recovered from the field bits, so it is
//! exact only while the values fit in the `L` bits we decompose and never wrap the
//! modulus. We fix `L = `[`COMPARE_BITS`]` = 60` and REQUIRE both operands `<
//! 2^60` (a fail-closed [`MpcError::Protocol`] otherwise). `p = 2^61 − 1`, so a
//! 60-bit value is canonical and unambiguous; the flatmate cumulative salary (four
//! × ~10^5 ≈ 10^6) is many orders of magnitude clear of `2^60`. This is the SAME
//! "values fit the field, no wrap" assumption the secure sum already relies on
//! (see [`crate::field`] doc), made explicit and checked here because a comparison
//! — unlike a sum — is meaningless if a value silently wrapped.
//!
//! ## Security model — honest-majority, semi-honest (NOT malicious)
//!
//! This inherits EXACTLY the [`crate::shamir::ShamirBackend`] model and adds no
//! new assumption: **honest-majority, semi-honest (honest-but-curious),
//! `n >= 2t+1`.** Privacy holds while `<= t` parties collude (any `<= t` bit
//! sharings, and any `<= t` views of the `eq`/`gt` chain's fresh re-sharings, are
//! independent of the secret — the standard Shamir + BGW privacy argument). It is
//! **NOT** maliciously secure: every multiplication routes through
//! [`ShamirDealer::degree_reduce`], whose re-sharing step has no in-protocol check
//! that a deviating party re-shared honestly (the same boundary as the degree-`2t`
//! equality open at `n = 2t+1`, and as `degree_reduce` itself documents). The
//! verdict is opened at degree `t`, so the WI-1 RS checker ([`crate::robust`])
//! still detects/corrects tampering of the FINAL opening where redundancy exists
//! (`n > t+1`) — but a party that cheats *inside* a re-sharing round is not caught.
//! We therefore report this operator as
//! [`crate::backend::OperatorClass::Comparison`] semi-honest-only (see
//! [`crate::shamir::ShamirBackend::operator_descriptor`]); we do NOT claim
//! malicious security. Malicious hardening (IT-MACs / verifiable resharing) is the
//! same future seam the rest of the backend defers
//! (`research/mpc-security-models-and-benchmarks.md` §3, §8 steps 5–6).

use crate::field::Fp;
use crate::partial::{HolderId, MpcError, PartialResult};
use crate::shamir::{self, ShamirBackend, ShamirDealer, Share};

/// Bit-width the comparison decomposes operands into. `p = 2^61 − 1`, so a value
/// `< 2^60` is canonical and the integer comparison recovered from its bits is
/// exact (no modular wraparound). Operands `>= 2^60` are rejected fail-closed.
pub const COMPARE_BITS: usize = 60;

/// The exclusive upper bound an operand must respect: `2^60`.
pub const COMPARE_MAX_EXCLUSIVE: u64 = 1u64 << COMPARE_BITS;

/// Fail-closed range check: an operand must be a canonical field element strictly
/// below `2^COMPARE_BITS`, else the bit-decomposition comparison could wrap the
/// modulus and silently return a wrong verdict.
fn check_in_range(label: &str, v: Fp) -> Result<(), MpcError> {
    if v.value() >= COMPARE_MAX_EXCLUSIVE {
        return Err(MpcError::Protocol(format!(
            "secure comparison: {label} = {} is out of range (must be < 2^{COMPARE_BITS} = {} so \
             the bit-decomposition comparison cannot wrap the field modulus)",
            v.value(),
            COMPARE_MAX_EXCLUSIVE
        )));
    }
    Ok(())
}

/// Fail-closed party-count precondition shared by every entry point: a comparison
/// is a multiplication CHAIN, and each multiplication's degree reduction needs
/// `n >= 2t+1` (so the degree-`2t` product is over-determined and the `2t+1`
/// recombination points exist). The honest-majority constructor already fixes
/// `t = ⌊(n−1)/2⌋ ⇒ n >= 2t+1`, so this only fires on a mis-built backend; it
/// fails with a descriptive error rather than letting `degree_reduce` panic later.
fn check_party_count(dealer: &ShamirDealer) -> Result<(), MpcError> {
    let t = dealer.threshold();
    if dealer.parties() < 2 * t + 1 {
        return Err(MpcError::Protocol(format!(
            "secure comparison needs n >= 2t+1 (each multiplication's degree reduction does); \
             got n = {}, t = {}",
            dealer.parties(),
            t
        )));
    }
    Ok(())
}

/// A degree-`t` sharing of the PUBLIC constant `c` on the canonical points
/// `x = 1..=n`: the constant polynomial `f(x) = c` evaluated at each party. This is
/// the standard "public value as a trivial sharing" — it reconstructs to `c` and is
/// degree `t` (degree 0 ⊆ degree `t`), so it composes with the secret sharings in
/// `add_shares`/`mul_shares_raw` without a dealer or any randomness.
fn const_sharing(n: usize, c: Fp) -> Vec<Share> {
    (1..=n as u64).map(|x| Share { x, y: c }).collect()
}

/// Decompose a secret value (held in cleartext here ONLY because this routine
/// plays ALL parties in one process, exactly like the dealer that shares it) into
/// `COMPARE_BITS` secret-shared bits, LSB-first (`out[k]` = sharing of bit `k`).
/// Each bit is a fresh degree-`t` sharing of a 0/1. The cleartext value is used
/// ONLY to deal the bit shares — it is never opened or returned, mirroring
/// [`ShamirDealer::share`].
fn share_bits(dealer: &mut ShamirDealer, v: Fp) -> Vec<Vec<Share>> {
    let raw = v.value();
    (0..COMPARE_BITS)
        .map(|k| {
            let bit = (raw >> k) & 1;
            dealer.share(Fp::new(bit))
        })
        .collect()
}

/// Secret AND of two secret-shared bits: `[a ∧ b] = [a]·[b]` (a multiplication on
/// 0/1 values is exactly logical AND). One [`shamir::mul_shares_raw`] (degree `2t`)
/// followed by one [`ShamirDealer::degree_reduce`] back to a fresh degree-`t`
/// sharing — the multiplication CHAIN step that needs the landed degree reduction
/// (sq-dvuc). `mul_shares_raw` preserves the canonical `x = 1..=n` point set (it is
/// component-wise), which is exactly `degree_reduce`'s fail-closed precondition.
fn secret_and(dealer: &mut ShamirDealer, a: &[Share], b: &[Share]) -> Result<Vec<Share>, MpcError> {
    let prod_2t = shamir::mul_shares_raw(a, b)?;
    dealer.degree_reduce(&prod_2t)
}

/// `[a ∧ ¬b]` for secret BITS `a`, `b` (= `a > b` at a single bit). `¬b = 1 − b`
/// is a local affine map on the sharing; the `∧` is one secure multiplication.
fn secret_and_not(
    dealer: &mut ShamirDealer,
    a: &[Share],
    b: &[Share],
) -> Result<Vec<Share>, MpcError> {
    let not_b = shamir::add_constant(&shamir::scale(b, Fp::one().neg()), Fp::one()); // 1 - b
    secret_and(dealer, a, &not_b)
}

/// `[a == b]` for secret BITS: `1 − (a − b)^2`. `(a−b)` is local; `(a−b)^2` is one
/// secure multiplication; `1 − …` is local. Returns a secret-shared 0/1 (`1` iff
/// the bits are equal).
fn secret_bit_eq(
    dealer: &mut ShamirDealer,
    a: &[Share],
    b: &[Share],
) -> Result<Vec<Share>, MpcError> {
    let diff = shamir::sub_shares(a, b)?; // a - b ∈ {-1,0,1} (mod p)
    let sq_2t = shamir::mul_shares_raw(&diff, &diff)?; // (a-b)^2 ∈ {0,1}
    let sq_t = dealer.degree_reduce(&sq_2t)?;
    // 1 - (a-b)^2 : bits equal ⇒ 1, differ ⇒ 0.
    Ok(shamir::add_constant(
        &shamir::scale(&sq_t, Fp::one().neg()),
        Fp::one(),
    ))
}

/// Core MSB-first comparison circuit over secret-shared bit vectors (LSB-first
/// `a_bits`, `b_bits`), returning a fresh degree-`t` sharing of the verdict bit
/// `a > b`. See the module docs for the recurrence. The operands' bit sharings are
/// the only inputs; nothing is opened here.
fn greater_than_bits(
    dealer: &mut ShamirDealer,
    a_bits: &[Vec<Share>],
    b_bits: &[Vec<Share>],
) -> Result<Vec<Share>, MpcError> {
    let n = dealer.parties();
    // gt accumulates the verdict; eq tracks "all higher bits equal so far".
    let mut gt = const_sharing(n, Fp::zero()); // [0]
    let mut eq = const_sharing(n, Fp::one()); // [1]

    for k in (0..COMPARE_BITS).rev() {
        let a_k = &a_bits[k];
        let b_k = &b_bits[k];
        // term = eq · (a_k ∧ ¬b_k) : contributes 1 only at the first MSB-down bit
        // where a_k=1, b_k=0 while everything above was equal.
        let a_gt_b_here = secret_and_not(dealer, a_k, b_k)?;
        let term = secret_and(dealer, &eq, &a_gt_b_here)?;
        gt = shamir::add_shares(&gt, &term)?;
        // eq = eq · (a_k == b_k) : once a higher bit differed, eq is 0 forever.
        let eq_here = secret_bit_eq(dealer, a_k, b_k)?;
        eq = secret_and(dealer, &eq, &eq_here)?;
    }
    Ok(gt)
}

/// MSB-first comparison of secret bits `a_bits` against a PUBLIC value `pub_val`
/// (its bits are constants). Same recurrence as [`greater_than_bits`] but every
/// product against a public bit is a local affine map, so the only secure
/// multiplications are the `eq`/`gt` chain steps.
fn greater_than_public_bits(
    dealer: &mut ShamirDealer,
    a_bits: &[Vec<Share>],
    pub_val: u64,
) -> Result<Vec<Share>, MpcError> {
    let n = dealer.parties();
    let mut gt = const_sharing(n, Fp::zero());
    let mut eq = const_sharing(n, Fp::one());

    for k in (0..COMPARE_BITS).rev() {
        let a_k = &a_bits[k];
        let b_k = (pub_val >> k) & 1;
        // a_k == b_k with b_k PUBLIC: if b_k=1 it is a_k; if b_k=0 it is 1 - a_k.
        let eq_here: Vec<Share> = if b_k == 1 {
            // [OPUS-4.8] When b_k=1, a_k ∧ ¬b_k = a_k ∧ ¬1 = 0 is a PUBLIC CONSTANT,
            // so term = eq · 0 = 0 and `gt` is unchanged — adding it is a no-op.
            // SKIP the `secret_and(dealer, &eq, a_gt_b_here)` here: it would be a
            // wasted mul + degree-reduce round on a known-zero operand. Only the
            // `eq` update remains a secure multiplication. (a_k == 1) == a_k.
            a_k.clone()
        } else {
            // b_k=0: a_k ∧ ¬0 = a_k, so term = eq · a_k IS a real secure mult.
            // (a_k == 0) == 1 - a_k.
            let term = secret_and(dealer, &eq, a_k)?;
            gt = shamir::add_shares(&gt, &term)?;
            shamir::add_constant(&shamir::scale(a_k, Fp::one().neg()), Fp::one())
        };
        // eq = eq · (a_k == b_k) — one secure multiplication.
        eq = secret_and(dealer, &eq, &eq_here)?;
    }
    Ok(gt)
}

/// **Secure greater-than over two SECRET operands**, opening only the verdict bit.
///
/// Returns a fresh degree-`t` sharing of the boolean `a > b` (1 if `a > b`, else
/// 0). The operands `a`, `b` are passed as cleartext here ONLY because this
/// routine plays ALL parties in one process — it secret-shares them internally
/// (their BITS) and never reconstructs them, exactly as the dealer that shares
/// them does; cf. [`crate::join::HiddenValueJoin`]'s `secure_equal`. To learn the
/// verdict, reconstruct the returned sharing with [`open_verdict`] (it opens to
/// 0/1) — that is the only thing ever opened.
///
/// Both operands must be `< 2^`[`COMPARE_BITS`] (fail-closed). `n >= 2t+1`
/// (fail-closed). Honest-majority, semi-honest — NOT malicious (module docs).
pub fn secure_greater_than(
    dealer: &mut ShamirDealer,
    a: Fp,
    b: Fp,
) -> Result<Vec<Share>, MpcError> {
    check_party_count(dealer)?;
    check_in_range("a", a)?;
    check_in_range("b", b)?;
    let a_bits = share_bits(dealer, a);
    let b_bits = share_bits(dealer, b);
    greater_than_bits(dealer, &a_bits, &b_bits)
}

/// **Secure greater-than against a PUBLIC threshold**, opening only the verdict bit
/// — the four-flatmates £100k path. Returns a fresh degree-`t` sharing of
/// `secret > threshold`.
///
/// The threshold is public, so its bits are public CONSTANTS: the per-bit products
/// against it (`a_k ∧ ¬b_k`, `a_k == b_k`) collapse to LOCAL affine maps on the
/// secret bit sharings — no multiplication round is spent on the threshold side,
/// only on chaining `eq`/`gt` (still a depth-`O(L)` chain). This is the cheaper
/// path and the headline disclosure-minimisation win: the verifier learns ONLY
/// `secret > threshold`, never the exact secret.
///
/// `secret` and `threshold` must be `< 2^`[`COMPARE_BITS`] (fail-closed).
/// `n >= 2t+1` (fail-closed). Honest-majority, semi-honest.
pub fn secure_threshold(
    dealer: &mut ShamirDealer,
    secret: Fp,
    threshold: Fp,
) -> Result<Vec<Share>, MpcError> {
    check_party_count(dealer)?;
    check_in_range("secret", secret)?;
    check_in_range("threshold", threshold)?;
    let a_bits = share_bits(dealer, secret);
    greater_than_public_bits(dealer, &a_bits, threshold.value())
}

/// Open ONLY the verdict bit of a comparison result. The result sharing is a
/// degree-`t` sharing of a 0/1; this reconstructs it (consistency-checked / robust
/// where redundancy exists, like every other open) and returns the boolean. This
/// is the ONLY value the comparison path ever opens — the operands are not
/// reconstructed.
pub fn open_verdict(backend: &ShamirBackend, verdict: &[Share]) -> Result<bool, MpcError> {
    let bit = backend.reconstruct(verdict)?;
    if bit == Fp::zero() {
        Ok(false)
    } else if bit == Fp::one() {
        Ok(true)
    } else {
        // A correct run always reconstructs to exactly 0 or 1. Anything else means
        // a tampered/inconsistent share survived reconstruction — refuse rather
        // than coerce a non-boolean to a verdict.
        Err(MpcError::Protocol(format!(
            "secure comparison verdict reconstructed to a non-boolean field element {} \
             (expected 0 or 1) — refusing to coerce",
            bit.value()
        )))
    }
}

/// Surface the £100k flatmate verdict as a disclosed [`PartialResult`] carrying
/// ONLY the boolean `sum > threshold` — never the exact sum. This is the
/// disclosure-minimising counterpart to
/// [`crate::shamir::ShamirBackend::reconstruct_disclosed`] (which opens the
/// integer): here the secure comparison runs over the secret-shared sum, and only
/// the 1-bit verdict is reconstructed.
///
/// `sum_shares` is the degree-`t` sharing of the cumulative aggregate (e.g. from
/// [`crate::shamir::ShamirBackend::run_secure`]); `public_threshold` is the public
/// bar (e.g. £100k). Only the 1-bit verdict ever leaves the computation.
///
/// [OPUS-4.8] NB on the in-process simulation — REAL protocol vs SIMULATION (be
/// precise, do not overclaim "the sum is never reconstructed"): in the REAL
/// multi-party protocol the sum stays secret-shared end-to-end — the parties
/// bit-decompose the EXISTING sum sharing IN-MPC (a known secret bit-decomposition
/// sub-protocol, e.g. via edaBits) so no single party ever sees the total, and only
/// the verdict bit is opened. In THIS in-process SIMULATION, however, the routine
/// plays ALL parties in one process, so it obtains the sum by reconstructing
/// `sum_shares` LOCALLY and re-deals its bits — legitimate ONLY because one process
/// holds every share (mirroring how the dealer holds cleartext to *share* inputs;
/// cf. `secure_equal`/[`share_bits`]). That local open is a simulation artefact, not
/// a disclosure: it never crosses the API boundary. The DISCLOSURE guarantee — only
/// the verdict bit ever LEAVES the computation — holds in both: the returned partial
/// carries the boolean alone, never the integer total. Removing even the
/// simulation-local open (in-MPC bit-decomposition of the existing sum sharing) is
/// the residual deployment step, tracked as a follow-up bead (sq-g7t5).
pub fn disclose_threshold_verdict(
    backend: &ShamirBackend,
    sum_shares: &[Share],
    public_threshold: u64,
) -> Result<PartialResult, MpcError> {
    // [OPUS-4.8] Fail closed BEFORE `Fp::new` reduces mod p. A `public_threshold`
    // >= p would wrap to an in-range element and silently compare against the wrong
    // bar; even a threshold in `[2^60, p)` is meaningless to the bit-decomposition
    // comparison (it cannot represent it). Reject anything outside the SAME no-wrap
    // safe range (`< 2^COMPARE_BITS`) the operands use, with a descriptive error,
    // rather than `Fp::new` silently wrapping. `check_in_range` below re-validates
    // the reduced element, but it would only see the post-wrap value — so the bound
    // must be enforced on the raw `u64` here.
    if public_threshold >= COMPARE_MAX_EXCLUSIVE {
        return Err(MpcError::Protocol(format!(
            "disclose_threshold_verdict: public_threshold = {public_threshold} is out of range \
             (must be < 2^{COMPARE_BITS} = {COMPARE_MAX_EXCLUSIVE} so it neither wraps the field \
             modulus p = 2^61-1 under Fp::new nor exceeds the bit-decomposition comparison's \
             representable range)"
        )));
    }
    let sum = backend.reconstruct(sum_shares)?;
    let mut dealer = backend.dealer();
    let verdict_shares = secure_threshold(&mut dealer, sum, Fp::new(public_threshold))?;
    let verdict = open_verdict(backend, &verdict_shares)?;
    Ok(PartialResult {
        holder: HolderId::new("federation"),
        vars: vec![oxrdf::Variable::new_unchecked("over_threshold")],
        rows: vec![vec![Some(oxrdf::Term::Literal(
            oxrdf::Literal::new_typed_literal(verdict.to_string(), oxrdf::vocab::xsd::BOOLEAN),
        ))]],
    })
}

#[cfg(test)]
mod tests {
    //! sq-rrz4 acceptance suite. The load-bearing ones are:
    //! - `differential_*`: across many (a,b)/(sum,threshold) pairs incl. edges,
    //!   the reconstructed verdict equals the plaintext `a > b` / `sum > threshold`.
    //! - `disclosure_minimisation_*`: only the 1-bit verdict ever LEAVES the
    //!   computation. [OPUS-4.8] Precise claim: in the REAL multi-party protocol the
    //!   operands and the sum stay secret-shared and are bit-decomposed in-MPC, so
    //!   nothing but the verdict bit is ever opened. In THIS in-process SIMULATION
    //!   `disclose_threshold_verdict` does a local `reconstruct(sum_shares)` — but
    //!   only because one process holds ALL shares, exactly as the dealer holds
    //!   cleartext to *share* inputs; that local open re-deals the sum's bits and
    //!   never crosses the API boundary. The DISCLOSURE guarantee (only the boolean
    //!   verdict is returned, never the operand/sum integer) holds in both, and the
    //!   tests assert it structurally on the returned partial.
    //! - `verdict_is_valid_degree_t_bit_sharing`: the result reconstructs
    //!   consistently to a 0/1 from ANY t+1 shares.
    //! - `*_fails_closed`: n<2t+1 and out-of-range operands are descriptive errors.
    use super::*;
    use crate::shamir::ShamirBackend;

    /// Reconstruct from the first `k` shares (to prove any t+1 suffice).
    fn recon_subset(backend: &ShamirBackend, shares: &[Share], k: usize) -> Fp {
        backend.reconstruct(&shares[..k]).unwrap()
    }

    #[test]
    fn differential_greater_than_across_many_pairs() {
        // THE sq-rrz4 differential (both-secret): the secure verdict must equal the
        // plaintext a > b across a spread of values incl. edges, several honest-
        // majority party counts. Deterministic seeds so the simulation reproduces.
        let cases: &[(u64, u64)] = &[
            (0, 0),                         // equal, zero
            (1, 0),                         // a > b minimal
            (0, 1),                         // a < b minimal
            (5, 5),                         // equal
            (100_000, 100_000),             // equal at the use-case scale
            (100_001, 100_000),             // a just over
            (99_999, 100_000),              // a just under
            (1 << 59, (1 << 59) - 1),       // high-bit difference
            ((1 << 60) - 1, 0),             // max vs zero
            ((1 << 60) - 1, (1 << 60) - 2), // adjacent near the top
            (42, 1_000_000),
            (1_000_000, 42),
        ];
        for n in [3usize, 5, 7] {
            for (idx, &(a, b)) in cases.iter().enumerate() {
                let backend = ShamirBackend::new_seeded(
                    n,
                    (idx as u64).wrapping_mul(97).wrapping_add(n as u64),
                )
                .unwrap();
                let mut dealer = backend.dealer();
                let v = secure_greater_than(&mut dealer, Fp::new(a), Fp::new(b)).unwrap();
                let got = open_verdict(&backend, &v).unwrap();
                assert_eq!(
                    got,
                    a > b,
                    "n={n} a={a} b={b}: secure > disagreed with plaintext"
                );
            }
        }
    }

    #[test]
    fn differential_threshold_across_many_pairs() {
        // The £100k path differential: verdict == (sum > threshold), public bar,
        // incl. edges (sum==threshold, sum=0, max value, threshold=0).
        let clean: &[(u64, u64)] = &[
            (0, 100_000),
            (100_000, 100_000),       // sum == threshold ⇒ NOT strictly greater
            (100_001, 100_000),       // just over the £100k bar
            (99_999, 100_000),        // just under
            (250_000, 100_000),       // the four-flatmates cumulative clears it
            ((1 << 60) - 1, 100_000), // max value
            (100_000, 0),             // threshold zero
            (0, 0),                   // both zero (0 > 0 is false)
        ];
        for n in [3usize, 5] {
            for (idx, &(sum, thr)) in clean.iter().enumerate() {
                let backend = ShamirBackend::new_seeded(n, 1234 + idx as u64).unwrap();
                let mut dealer = backend.dealer();
                let v = secure_threshold(&mut dealer, Fp::new(sum), Fp::new(thr)).unwrap();
                let got = open_verdict(&backend, &v).unwrap();
                assert_eq!(
                    got,
                    sum > thr,
                    "n={n} sum={sum} thr={thr}: threshold verdict wrong"
                );
            }
        }
    }

    #[test]
    fn differential_randomized_stress() {
        // A broad randomized differential across the field (deterministic LCG so the
        // run reproduces): the secure verdict must match plaintext for MANY random
        // (a, b) pairs over several party counts, mixing small (high collision /
        // equal) and large (full ~53-bit, well inside 2^60) ranges so every bit
        // position is exercised.
        let mut state: u64 = 0xDEAD_BEEF_CAFE;
        let mut next = || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            state >> 11 // 53 random bits, well inside 2^60
        };
        for n in [3usize, 5, 7] {
            for trial in 0..120u64 {
                // Mix small-range (collisions/equality) and large-range operands.
                let (a, b) = if trial % 2 == 0 {
                    (next() % 64, next() % 64)
                } else {
                    (next() & ((1 << 60) - 1), next() & ((1 << 60) - 1))
                };
                let backend =
                    ShamirBackend::new_seeded(n, trial.wrapping_mul(7).wrapping_add(n as u64))
                        .unwrap();
                let mut dealer = backend.dealer();
                let v = secure_greater_than(&mut dealer, Fp::new(a), Fp::new(b)).unwrap();
                assert_eq!(
                    open_verdict(&backend, &v).unwrap(),
                    a > b,
                    "n={n} a={a} b={b}: randomized differential mismatch"
                );
            }
        }
    }

    #[test]
    fn four_flatmates_hundred_k_verdict() {
        // The architecture §4.3 use case end-to-end: four private salaries are
        // secret-shared, summed securely (run_secure), and the £100k verdict is
        // disclosed as ONLY the boolean — the exact total is never in the output.
        let salaries = [30_000u64, 28_000, 26_000, 24_000]; // sums to 108_000 > 100k
        let backend = ShamirBackend::new(5).unwrap();
        let shared: Vec<Vec<Share>> = salaries
            .iter()
            .map(|&s| backend.dealer().share(Fp::new(s)))
            .collect();
        let summed = {
            use crate::backend::MpcBackend;
            backend.run_secure(&shared).unwrap()
        };
        let out = disclose_threshold_verdict(&backend, &summed[0], 100_000).unwrap();
        // The disclosed partial carries the boolean verdict, NOT the sum integer.
        assert_eq!(out.vars.len(), 1);
        assert_eq!(out.vars[0].as_str(), "over_threshold");
        let term = out.rows[0][0].as_ref().unwrap();
        match term {
            oxrdf::Term::Literal(l) => {
                assert_eq!(l.value(), "true", "108k > 100k ⇒ verdict true");
                // And it is NOT the integer total leaking out.
                assert_ne!(l.value(), "108000");
            }
            other => panic!("expected boolean literal, got {other:?}"),
        }
    }

    #[test]
    fn four_flatmates_below_threshold_is_false() {
        let salaries = [10_000u64, 9_000, 8_000, 7_000]; // 34_000 < 100k
        let backend = ShamirBackend::new(3).unwrap();
        let shared: Vec<Vec<Share>> = salaries
            .iter()
            .map(|&s| backend.dealer().share(Fp::new(s)))
            .collect();
        let summed = {
            use crate::backend::MpcBackend;
            backend.run_secure(&shared).unwrap()
        };
        let out = disclose_threshold_verdict(&backend, &summed[0], 100_000).unwrap();
        match out.rows[0][0].as_ref().unwrap() {
            oxrdf::Term::Literal(l) => assert_eq!(l.value(), "false"),
            other => panic!("expected boolean literal, got {other:?}"),
        }
    }

    #[test]
    fn verdict_is_valid_degree_t_bit_sharing() {
        // Acceptance #3: the result is a valid degree-t sharing of a 0/1 — it
        // reconstructs CONSISTENTLY to the same boolean from ANY t+1 shares, and
        // the value is exactly 0 or 1.
        let backend = ShamirBackend::new_seeded(5, 0xABC).unwrap(); // t = 2
        let mut dealer = backend.dealer();
        let v = secure_greater_than(&mut dealer, Fp::new(100_001), Fp::new(100_000)).unwrap();
        let t = backend.threshold();
        // Full reconstruction is the verdict bit (1).
        let full = backend.reconstruct(&v).unwrap();
        assert_eq!(full, Fp::one());
        // ANY t+1 shares reconstruct to the SAME value (consistency of a degree-t
        // sharing). Exercise several distinct (t+1)-subsets.
        assert_eq!(recon_subset(&backend, &v, t + 1), Fp::one());
        // A different window of t+1 contiguous shares also agrees.
        let window = &v[1..1 + (t + 1)];
        assert_eq!(backend.reconstruct(window).unwrap(), Fp::one());
        // And a 0-verdict is likewise a valid 0-sharing.
        let mut dealer2 = backend.dealer();
        let v0 = secure_greater_than(&mut dealer2, Fp::new(5), Fp::new(9)).unwrap();
        assert_eq!(backend.reconstruct(&v0).unwrap(), Fp::zero());
        assert_eq!(backend.reconstruct(&v0[..t + 1]).unwrap(), Fp::zero());
    }

    #[test]
    fn disclosure_minimisation_only_the_bit_is_opened() {
        // Acceptance #2: the operands are NEVER reconstructed on the verdict path —
        // ONLY the 1-bit verdict is. We verify this STRUCTURALLY: reconstructing
        // the returned sharing yields a single 0/1 (the bit), two pairs with the
        // SAME order but wildly different operands open to the SAME verdict, and the
        // returned sharing is exactly ONE field element per party (the width of a
        // single bit, not of any operand bit-vector). The operand bit-sharings are
        // local to the protocol and are never returned, so a caller cannot open
        // them.
        let backend = ShamirBackend::new_seeded(5, 7).unwrap();
        let mut d1 = backend.dealer();
        let mut d2 = backend.dealer();
        // Same relation (a > b), very different magnitudes.
        let v_small = secure_greater_than(&mut d1, Fp::new(2), Fp::new(1)).unwrap();
        let v_huge = secure_greater_than(&mut d2, Fp::new((1 << 60) - 1), Fp::new(0)).unwrap();
        // The ONLY thing openable is the verdict, and both open to the SAME bit (1)
        // — the opened value reveals the order relation, not the operand sizes.
        assert_eq!(backend.reconstruct(&v_small).unwrap(), Fp::one());
        assert_eq!(backend.reconstruct(&v_huge).unwrap(), Fp::one());
        // The returned sharing is exactly one field element per party — the width
        // of a single secret bit, not of any operand bit-vector (which would be
        // COMPARE_BITS sharings). i.e. only the verdict crosses the API boundary.
        assert_eq!(v_small.len(), backend.parties());
        assert_eq!(v_huge.len(), backend.parties());

        // The £100k path likewise discloses ONLY the boolean: the partial carries a
        // boolean literal, and no integer sum appears anywhere in it.
        let shared: Vec<Vec<Share>> = [60_000u64, 60_000]
            .iter()
            .map(|&s| backend.dealer().share(Fp::new(s)))
            .collect();
        let summed = {
            use crate::backend::MpcBackend;
            backend.run_secure(&shared).unwrap()
        };
        let out = disclose_threshold_verdict(&backend, &summed[0], 100_000).unwrap();
        // Render the entire disclosed partial; the exact total (120000) must NOT
        // appear anywhere — only the boolean verdict does.
        let rendered = format!("{:?}", out.rows);
        assert!(rendered.contains("true"), "verdict should be disclosed");
        assert!(
            !rendered.contains("120000"),
            "the exact sum must NOT be disclosed"
        );
    }

    #[test]
    fn range_precondition_fails_closed() {
        // Acceptance #4 (range half): an operand >= 2^COMPARE_BITS could let the
        // bit-decomposition comparison wrap the modulus, so it is a descriptive
        // fail-closed Protocol error — not a wrong verdict, not a panic.
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        let mut dealer = backend.dealer();
        // Out-of-range operand a (both-secret path).
        let err = secure_greater_than(&mut dealer, Fp::new(COMPARE_MAX_EXCLUSIVE), Fp::new(0))
            .unwrap_err();
        match err {
            MpcError::Protocol(m) => assert!(
                m.contains("out of range") && m.contains("2^60"),
                "expected range fail-closed, got: {m}"
            ),
            other => panic!("expected Protocol range error, got {other:?}"),
        }
        // Out-of-range operand b.
        assert!(matches!(
            secure_greater_than(&mut dealer, Fp::new(0), Fp::new(COMPARE_MAX_EXCLUSIVE)),
            Err(MpcError::Protocol(_))
        ));
        // Out-of-range secret / threshold on the £100k path.
        assert!(matches!(
            secure_threshold(&mut dealer, Fp::new(COMPARE_MAX_EXCLUSIVE), Fp::new(0)),
            Err(MpcError::Protocol(_))
        ));
        assert!(matches!(
            secure_threshold(&mut dealer, Fp::new(0), Fp::new(COMPARE_MAX_EXCLUSIVE)),
            Err(MpcError::Protocol(_))
        ));
        // The maximum IN-range value (2^60 - 1) is accepted (boundary is exclusive).
        let mut d2 = backend.dealer();
        assert!(
            secure_greater_than(&mut d2, Fp::new(COMPARE_MAX_EXCLUSIVE - 1), Fp::new(0)).is_ok()
        );
    }

    #[test]
    fn disclose_threshold_out_of_range_public_threshold_fails_closed() {
        // [OPUS-4.8] A `public_threshold` outside the safe no-wrap range must be
        // REJECTED fail-closed, NOT silently reduced mod p by `Fp::new`. We exercise
        // two distinct hazards:
        //   (a) threshold >= p (= 2^61 - 1): `Fp::new` would wrap it to an in-range
        //       element, silently comparing the sum against the WRONG bar.
        //   (b) threshold in [2^60, p): in canonical range for Fp but NOT
        //       representable by the COMPARE_BITS bit-decomposition comparison.
        // Both must surface a descriptive Protocol error from disclose_threshold_verdict.
        let backend = ShamirBackend::new_seeded(5, 0x7e57).unwrap();
        // A small, legitimate secret-shared sum to compare against.
        let summed = {
            use crate::backend::MpcBackend;
            let shared: Vec<Vec<Share>> = [50_000u64, 50_000]
                .iter()
                .map(|&s| backend.dealer().share(Fp::new(s)))
                .collect();
            backend.run_secure(&shared).unwrap()
        };

        // (a) threshold >= p would wrap under Fp::new — must be refused, not wrapped.
        let p = (1u64 << 61) - 1;
        for over_p in [p, p + 7, u64::MAX] {
            let err = disclose_threshold_verdict(&backend, &summed[0], over_p).unwrap_err();
            match err {
                MpcError::Protocol(m) => assert!(
                    m.contains("out of range") && m.contains("2^60"),
                    "expected fail-closed range error for threshold {over_p}, got: {m}"
                ),
                other => panic!("expected Protocol range error, got {other:?}"),
            }
        }

        // Concretely prove it did NOT silently wrap: p + 1 ≡ 1 (mod p). If the bound
        // were missing, the threshold would become 1 and the (100_000 > 1) verdict
        // would be `true`. The fail-closed error is returned instead.
        assert!(matches!(
            disclose_threshold_verdict(&backend, &summed[0], p + 1),
            Err(MpcError::Protocol(_))
        ));

        // (b) threshold in [2^60, p): canonical Fp element but out of the
        // bit-decomposition's representable range — also refused fail-closed.
        for in_field_but_oob in [
            COMPARE_MAX_EXCLUSIVE,
            COMPARE_MAX_EXCLUSIVE + 1,
            (1u64 << 61) - 2,
        ] {
            assert!(
                matches!(
                    disclose_threshold_verdict(&backend, &summed[0], in_field_but_oob),
                    Err(MpcError::Protocol(_))
                ),
                "threshold {in_field_but_oob} (>= 2^60) must fail closed"
            );
        }

        // The maximum IN-range threshold (2^60 - 1) is still accepted (boundary is
        // exclusive) — the guard rejects only out-of-range values, not valid ones.
        assert!(
            disclose_threshold_verdict(&backend, &summed[0], COMPARE_MAX_EXCLUSIVE - 1).is_ok()
        );
    }

    #[test]
    fn party_count_precondition_fails_closed() {
        // Acceptance #4 (party-count half): a comparison is a multiplication chain,
        // so each step's degree reduction needs n >= 2t+1. `check_party_count`
        // fails closed with a descriptive Protocol error when that does not hold,
        // rather than letting `degree_reduce` panic/mis-reduce later.
        //
        // The honest-majority constructor always picks t = ⌊(n−1)/2⌋ ⇒ n >= 2t+1,
        // so it cannot itself build a deficient (n, t). We reach the guard through
        // the test-only `with_unchecked_threshold` seam (cfg(test)), which builds a
        // backend with an explicit oversized t purely to exercise this fail-closed
        // path — it is NOT a production constructor.
        let bad = ShamirBackend::with_unchecked_threshold(3, 2); // n=3, t=2 ⇒ 2t+1=5 > 3
        let mut dealer = bad.dealer();
        let err = secure_greater_than(&mut dealer, Fp::new(1), Fp::new(0)).unwrap_err();
        match err {
            MpcError::Protocol(m) => assert!(
                m.contains("n >= 2t+1"),
                "expected n>=2t+1 fail-closed, got: {m}"
            ),
            other => panic!("expected Protocol party-count error, got {other:?}"),
        }
        // The threshold path guards identically.
        assert!(matches!(
            secure_threshold(&mut dealer, Fp::new(1), Fp::new(0)),
            Err(MpcError::Protocol(_))
        ));
    }

    #[test]
    fn non_boolean_verdict_is_refused() {
        // open_verdict must REFUSE a sharing that does not reconstruct to 0/1
        // (e.g. a tampered/garbage sharing), rather than coerce it to a verdict.
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        let mut dealer = backend.dealer();
        let garbage = dealer.share(Fp::new(7)); // not a 0/1
        let err = open_verdict(&backend, &garbage).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(_)));
    }
}
