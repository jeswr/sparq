// [OPUS-4.8] sq-ujz8: ORQ-style oblivious sort-merge join over SECRET keys —
// replace the O(|L|·|R|) all-pairs `HiddenValueJoin` candidate enumeration with an
// O(n log² n) oblivious sort (n = |L|+|R|) + a linear segmented merge scan. Native-
// only; in-process multi-party simulation; communication cost MODELLED not measured.
// Authored under the Opus 4.8 fallback model — re-review when Fable returns.
//! Oblivious **sort-merge** join over private keys (bead sq-ujz8).
//!
//! ## Why this exists — the quadratic cost centre ORQ/Secrecy engineer away
//!
//! [`crate::join::HiddenValueJoin`] decides a hidden-key join by testing ONE
//! secret-shared equality **per candidate pair** — `O(|L|·|R|)` secure equalities
//! (`join.rs`, all three of `join` / `batched_join` / `fully_oblivious_batched_join`
//! enumerate every `(i, j)`). That all-pairs structure is exactly the quadratic
//! cost centre the relational-MPC line (Secrecy NSDI'23, ORQ SOSP'25 — see
//! `research/mpc-security-models-and-benchmarks.md` §2, `research/mpc-sparql-
//! capability-matrix.md` P6) replaces with a **sort-merge**: concatenate the two
//! sides into one relation of `n = |L|+|R|` rows, **obliviously sort by the join
//! key**, then a LINEAR scan over the sorted sequence decides the join — because
//! equal keys are adjacent after sorting, a row's matches are found among its
//! neighbours in its key-group, not by scanning the whole other side. Cost drops
//! from `O(|L|·|R|)` secure equalities to `O(n log² n)` compare-exchanges (the
//! Batcher network) + `O(n)` segmented-scan multiplications.
//!
//! This is distinct from the circuit-PSI approach (beads sq-h99 / sq-t21) per the
//! research synthesis — sort-merge is the operator that COMPOSES for multi-pattern
//! BGPs / sub-SELECTs, where circuit-PSI does not compose for free (matrix §2.5).
//!
//! ## What this module delivers (sound today) vs the honestly-scoped remainder
//!
//! Every primitive the ORQ operator needs has landed: the data-independent sort
//! NETWORK (`crate::oblivious::SortingNetwork`, sq-18lk), the degree-reduction round
//! (`ShamirDealer::degree_reduce`, sq-dvuc), the secure comparator
//! (`crate::compare::secure_greater_than`, sq-rrz4), and the oblivious shuffle
//! (`crate::oblivious::shuffle`, sq-18lk). The remaining integration step the
//! capability matrix names — *"the wired secret-key sort path"* — is built here:
//!
//! ### Delivered (sound, honest-majority / semi-honest)
//!
//! - [`oblivious_sort_by_secret_key`] — the **secret-key oblivious sort** substrate:
//!   the Batcher odd-even mergesort access pattern (data-independent, sq-18lk) driven
//!   by the landed P5 comparator ([`crate::compare::secure_greater_than_shared`], a
//!   secret verdict that is NEVER opened) and a **secret-bit-gated arithmetic
//!   conditional swap** `x' = x + b·(y−x)` (one secure multiplication per carried
//!   field element per switch). The only openings during the sort are the comparator's
//!   per-compare masked Rabbit decompositions (`c = x + r mod p`, near-uniform /
//!   statistically hiding at `≤ 2^{-61}` each — see the leakage statement); the verdict
//!   and the payload columns ride through the swaps as never-opened secret shares. This is the P6 sort the
//!   HIDDEN-key regime needs (the disclosed-key sort `sort_with_keys` was already
//!   sound — sq-18lk).
//! - [`sort_merge_semi_anti`] — the **semi-join and anti-join** (`MINUS` / `EXISTS` /
//!   `NOT EXISTS` family) via the sort-merge: sort the tagged union, a two-pass
//!   segmented scan computes each row's secret *"a row from the OTHER side shares my
//!   key"* bit, and the kept L rows are revealed through an oblivious shuffle +
//!   padded reveal (so which L rows matched is disclosed — that IS the result — but
//!   the keys, the R identities, and the sorted-key ORDER are not). Differential-
//!   tested against a plaintext semi-/anti-join oracle (the `minus_bindings`
//!   semantics for anti).
//!
//! ### Scoped remainder (NOT built here — honest, filed as follow-ups, not faked)
//!
//! - **Inner-join MATERIALISATION with many-to-many fan-out** and **left-outer with
//!   the matched R payload joined on** need the ORQ *expansion* step (an oblivious
//!   distribution of one side's payload across a key-group), which is a separate,
//!   larger protocol. Until it lands, inner-join materialisation is still served by
//!   the existing all-pairs [`crate::join::HiddenValueJoin::fully_oblivious_batched_join`]
//!   (correct, `O(|L|·|R|)`). This module deliberately does the fan-out-FREE family
//!   (semi / anti / existence-mark), which is the `O(n log n)` core.
//! - **Eager numeric aggregation** (per-key `COUNT`/`SUM` over the sorted groups via
//!   a segmented prefix-sum) reuses this module's sort + scan and is a natural
//!   follow-up on the same substrate.
//!
//! ## Privacy / leakage statement (empirical-honesty rule — state it loudly)
//!
//! - **The join KEY is never reconstructed — but the hiding is STATISTICAL, not
//!   information-theoretic.** The comparator and the adjacency-equality keep their
//!   VERDICT secret-shared (never opened) and the conditional swap opens nothing —
//!   HOWEVER each comparison/equality bit-decomposes its operands via the Rabbit path
//!   ([`crate::compare`]'s `secure_bit_decompose_rabbit`), whose ONE opening is a
//!   masked `c = (x + r) mod p` under a full-field mask `r` no party knows. That `c`
//!   is *near-uniform*, not perfectly uniform: it sits a statistical distance
//!   `≤ 2^{-61}` from uniform (the `p = 2^61 − 1` field-size floor — see the "Hiding"
//!   note on `secure_bit_decompose_rabbit`), so each opening leaks at most a `2^{-61}`
//!   advantage about its operand — NOT the perfect per-key independence a categorical
//!   "Shamir hiding" would assert. Each comparison / equality does this per operand
//!   TWICE: the masked decomposition open `c = x + r mod p`, and the in-protocol
//!   Rabbit range proof's masked zero-test open (`verify_value_in_range_rabbit`), each
//!   a `≤ 2^{-61}`-hiding open. **Composition:** one `sort_merge_semi_anti` therefore
//!   performs `Θ(|compare-exchanges| + n)` masked openings — a small constant per sort
//!   compare-exchange (`|compare-exchanges| = O(n log² n)`) and per adjacency equality
//!   (`n − 1` of them) — so by the standard hybrid / union-bound argument the total
//!   advantage to any `≤ t`-collusion composes to `≤ O(n log² n) · 2^{-61}`,
//!   cryptographically negligible for every realistic `n` but a *statistical* bound
//!   that must be composed, not a per-opening independence. (The Shamir SHARINGS of
//!   the keys are themselves perfectly `t`-private; the residual floor is the masked
//!   OPENINGS the comparator performs, not the sharings.)
//! - **The sorted-key ORDER is never revealed.** The final result is emitted through
//!   an oblivious [`crate::oblivious::shuffle`], so the position a revealed row came
//!   from — hence the key order — is destroyed before anything opens.
//! - **What IS revealed** (the result — plus the negligible masked-opening statistical
//!   floor noted above): for each kept L row, its
//!   *payload id* (which L row it is) is opened — that is the semi-/anti-join output
//!   the authorised recipient is entitled to. The number of kept rows (the result
//!   size) is learned by the recipient after filtering dummies, exactly as every
//!   other oblivious-output path in the crate (`oblivious_join`; the padded-bound
//!   scheme, NOT differential privacy — DP cardinality is the separate bead sq-shk5).
//!   `|L|`, `|R|` and `n` are public (standard MPC assumption; matrix §4.1).
//! - **Malicious security is unchanged** from the Shamir/`compare` layer: every
//!   multiplication routes through `degree_reduce`, which has no in-protocol check
//!   that a deviating party re-shared honestly — semi-honest, honest-majority only.
//!   External soundness sign-off is pending (sq-qhy4).
//!
//! ## Communication / round cost (MODELLED — counted, not measured)
//!
//! [`SortMergeCost`] reports what a real deployment would pay, derived from `n`
//! (never a hard-coded number, AGENTS.md): the sort's `O(n log² n)` compare-exchanges
//! (each a secure comparison + a conditional swap), the `O(n)` segmented-scan
//! multiplications, the oblivious shuffle's switch count/depth, and the padded
//! reveal opens. The in-process simulation's wall-clock is NOT an MPC latency.

use crate::compare::{secure_equal_to_bit_shared, secure_greater_than_shared, COMPARE_MAX_EXCLUSIVE};
use crate::field::Fp;
use crate::oblivious::{shuffle, SortingNetwork};
use crate::partial::{HolderId, MpcError, PartialResult};
use crate::shamir::{self, ShamirBackend, ShamirDealer, Share};
use oxrdf::{Term, Variable};

/// Which fan-out-free sort-merge join this operator computes over the L side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMergeJoinKind {
    /// **Semi-join**: keep each L row that has AT LEAST ONE R row with the same key
    /// (`FILTER EXISTS` / the presence half of an inner join). The R payload is not
    /// joined on — only L rows are emitted (fan-out-free).
    Semi,
    /// **Anti-join**: keep each L row that has NO R row with the same key (`MINUS` /
    /// `FILTER NOT EXISTS`; the `minus_bindings` semantics on a single shared key).
    Anti,
}

/// MODELLED communication / round cost of one [`sort_merge_semi_anti`]. Counted,
/// never measured (see the module cost note). No hard-coded numbers: derived from
/// the union size and the built networks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortMergeCost {
    /// Union size `n = |L| + |R|` (public).
    pub union_size: usize,
    /// Compare-exchange gates in the oblivious sort (Batcher odd-even mergesort over
    /// `n`) — each is one secure comparison + one conditional swap.
    pub sort_compare_exchanges: usize,
    /// Sort network depth in comparator layers (≈ communication rounds).
    pub sort_depth_rounds: usize,
    /// Secure multiplications spent in the segmented merge scan (adjacency
    /// equalities + the forward/backward OR passes + the per-row selector).
    pub scan_mults: usize,
    /// Degree-`t` opens performed at the padded reveal (one selector per revealed
    /// slot, plus the payload id of each kept row).
    pub opens: usize,
    /// The final oblivious shuffle's cost over the `n` reveal slots.
    pub shuffle: crate::oblivious::ShuffleCost,
}

/// The result of a sort-merge semi-/anti-join: the oblivious [`PartialResult`] (the
/// kept L rows only, dummies filtered, federation-attributed, canonical order) plus
/// the MODELLED [`SortMergeCost`].
pub type SortMergeOutput = (PartialResult, SortMergeCost);

/// A secret-shared union row as it travels through the oblivious sort: the join
/// KEY, a `1{is L-side}` flag, and the L-side PAYLOAD ID (which L row it is), each a
/// full degree-`t` sharing. All three ride through the conditional swaps together so
/// a row's `(key, side, id)` stay locked as it is permuted.
struct Row {
    key: Vec<Share>,
    is_l: Vec<Share>,
    /// The original L-row index for an L row; irrelevant (never revealed) for an R
    /// row, which always has `is_l = 0` so its selector is 0.
    pid: Vec<Share>,
}

/// **The oblivious sort-merge semi-/anti-join** (bead sq-ujz8).
///
/// Join the L side's private keys against the R side's private keys WITHOUT
/// revealing any key: keep each L row whose key is present ([`SortMergeJoinKind::Semi`])
/// or absent ([`SortMergeJoinKind::Anti`]) in the R key multiset. The output is the
/// kept L rows' disclosed payloads (schema `out_vars`), attributed to the synthetic
/// `federation` holder, in canonical order.
///
/// `left` is `(private_key, disclosed_payload)` per L row; `right_keys` is the R
/// side's private keys (R payloads are not emitted by a semi/anti-join, so R needs
/// only its keys). See the module docs for the delivered-vs-scoped split, the
/// leakage statement, and the honest-majority / semi-honest security model.
///
/// ## Fail-closed contracts
///
/// - Every key (both sides) must be `< 2^60` (`COMPARE_MAX_EXCLUSIVE`) so the
///   in-MPC bit-decomposition comparator is injective — a larger key is a
///   [`MpcError::Protocol`] error, never a silently-wrapped comparison.
/// - `n >= 2t + 1` for the secure multiplications (fail-closed inside the primitives).
/// - Every L payload must have arity `out_vars.len()` (uniform schema for the
///   recipient) — a short row is a malformed input, not a silent truncation.
pub fn sort_merge_semi_anti(
    backend: &ShamirBackend,
    left: &[(Fp, Vec<Option<Term>>)],
    right_keys: &[Fp],
    out_vars: Vec<Variable>,
    kind: SortMergeJoinKind,
) -> Result<SortMergeOutput, MpcError> {
    let payload_arity = out_vars.len();
    // Validate payload arity + key range up front (fail closed before any protocol
    // work, so a malformed input never enters the crypto core).
    for (k, (key, pay)) in left.iter().enumerate() {
        if pay.len() != payload_arity {
            return Err(MpcError::Protocol(format!(
                "sort_merge_semi_anti: L row {k} has {} payload terms, expected arity {payload_arity}",
                pay.len()
            )));
        }
        check_key_range("L", k, *key)?;
    }
    for (k, key) in right_keys.iter().enumerate() {
        check_key_range("R", k, *key)?;
    }

    let ll = left.len();
    let rr = right_keys.len();
    let n = ll + rr;

    let mut dealer = backend.dealer();
    let t = dealer.threshold();
    let parties = dealer.parties();
    if parties < 2 * t + 1 {
        return Err(MpcError::Protocol(format!(
            "sort_merge_semi_anti needs n >= 2t+1 for the secure multiplications (n={parties}, t={t})"
        )));
    }

    // --- Build the tagged secret union in original order. `is_l`/`pid` start as
    // PUBLIC facts (position), shared as constants; the conditional swaps then mix
    // them into genuine secret combinations aligned to the (secret) sorted order. ---
    let mut rows: Vec<Row> = Vec::with_capacity(n);
    for (i, (key, _pay)) in left.iter().enumerate() {
        rows.push(Row {
            key: dealer.share(*key),
            is_l: dealer.share(Fp::one()),
            pid: dealer.share(Fp::new(i as u64)),
        });
    }
    for key in right_keys {
        rows.push(Row {
            key: dealer.share(*key),
            is_l: dealer.share(Fp::zero()),
            pid: dealer.share(Fp::zero()),
        });
    }

    // --- (1) Oblivious sort by the SECRET key (ascending). ---
    let net = SortingNetwork::new(n);
    for &(i, j) in net.compare_exchanges() {
        // Swap iff key_i > key_j (sort ascending) — a NEVER-opened secret verdict.
        let (lo, hi) = (i.min(j), i.max(j));
        let swap_bit = secure_greater_than_shared(&mut dealer, backend, &rows[lo].key, &rows[hi].key)?;
        conditional_swap(&mut dealer, &mut rows, lo, hi, &swap_bit)?;
    }

    // --- (2) Segmented merge scan: per sorted position, the secret bit "a row from
    //         the OTHER side (an R row) shares my key-group". Equal keys are now
    //         adjacent, so a group is a maximal run of equal keys; we broadcast
    //         "the run contains an R row" to every member with a forward + backward
    //         OR pass gated by the adjacent-equal bit. ---
    let mut scan_mults = 0usize;
    // Adjacent-key equalities eq[k] = [key_k == key_{k+1}] (secret), for k in 0..n-1.
    let mut eq: Vec<Vec<Share>> = Vec::with_capacity(n.saturating_sub(1));
    for k in 0..n.saturating_sub(1) {
        eq.push(secure_equal_to_bit_shared(
            &mut dealer,
            backend,
            &rows[k].key,
            &rows[k + 1].key,
        )?);
        scan_mults += 1; // equal_to_bit_from_bits multiplications, modelled per call
    }
    // is_r[k] = 1 - is_l[k] (local).
    let is_r: Vec<Vec<Share>> = rows
        .iter()
        .map(|r| shamir::add_constant(&shamir::scale(&r.is_l, Fp::one().neg()), Fp::one()))
        .collect();

    // Backward: b_r[k] = is_r[k] OR (eq[k] AND b_r[k+1]) — "R at or after k in run".
    let mut b_r: Vec<Vec<Share>> = vec![Vec::new(); n];
    if n > 0 {
        b_r[n - 1] = is_r[n - 1].clone();
        for k in (0..n - 1).rev() {
            let carried = secure_mul(&mut dealer, &eq[k], &b_r[k + 1])?;
            b_r[k] = secure_or(&mut dealer, &is_r[k], &carried)?;
            scan_mults += 2;
        }
    }
    // Forward: f_r[k] = is_r[k] OR (eq[k-1] AND f_r[k-1]) — "R at or before k in run".
    let mut f_r: Vec<Vec<Share>> = vec![Vec::new(); n];
    if n > 0 {
        f_r[0] = is_r[0].clone();
        for k in 1..n {
            let carried = secure_mul(&mut dealer, &eq[k - 1], &f_r[k - 1])?;
            f_r[k] = secure_or(&mut dealer, &is_r[k], &carried)?;
            scan_mults += 2;
        }
    }

    // --- (3) Per-position selector: keep this row iff it is an L row and the
    //         semi/anti condition on "any R in my run" holds. ---
    let mut selectors: Vec<Vec<Share>> = Vec::with_capacity(n);
    for k in 0..n {
        // any_r[k] = b_r[k] OR f_r[k].
        let any_r = secure_or(&mut dealer, &b_r[k], &f_r[k])?;
        scan_mults += 1;
        let cond = match kind {
            SortMergeJoinKind::Semi => any_r,
            // NOT any_r = 1 - any_r (local).
            SortMergeJoinKind::Anti => {
                shamir::add_constant(&shamir::scale(&any_r, Fp::one().neg()), Fp::one())
            }
        };
        // selector = is_l AND cond (one secure mult).
        selectors.push(secure_mul(&mut dealer, &rows[k].is_l, &cond)?);
        scan_mults += 1;
    }

    // --- (4) Oblivious reveal: pack (selector, pid) per position, oblivious-shuffle
    //         so the revealed order hides the sorted-key order, open the selector
    //         (0/1) per slot, and for a kept row open its pid → the L payload. ---
    let mut packed: Vec<Vec<Share>> = Vec::with_capacity(n);
    for (sel, row) in selectors.iter().zip(rows.iter()) {
        let mut entry = sel.clone();
        entry.extend_from_slice(&row.pid);
        packed.push(entry);
    }
    let (shuffled, shuffle_cost) = shuffle(&mut dealer, packed)?;

    let mut rows_out: Vec<Vec<Option<Term>>> = Vec::new();
    let mut opens = 0usize;
    for entry in &shuffled {
        // entry = [selector shares (parties) | pid shares (parties)].
        let (sel_part, pid_part) = entry.split_at(parties);
        let sel = backend.reconstruct(sel_part)?;
        opens += 1;
        if sel == Fp::one() {
            // A kept L row — reveal its payload id (this row IS in the result, so
            // disclosing which L row it is discloses only the result the recipient
            // gets). Map back to the L payload.
            let pid = backend.reconstruct(pid_part)?;
            opens += 1;
            let idx = pid.value() as usize;
            let payload = left
                .get(idx)
                .map(|(_, p)| p.clone())
                .ok_or_else(|| {
                    MpcError::Protocol(format!(
                        "sort_merge_semi_anti: revealed payload id {idx} out of range (|L|={ll}) \
                         — corrupted share slipped past the open"
                    ))
                })?;
            rows_out.push(payload);
        } else if sel == Fp::zero() {
            // A dropped L row or any R row — an indistinguishable dummy.
        } else {
            // A correct run opens a selector to exactly 0 or 1. Anything else means a
            // tampered/inconsistent share survived the open — refuse rather than
            // silently drop or emit a corrupted row.
            return Err(MpcError::Tampered {
                detail: format!(
                    "sort_merge_semi_anti: opened selector {} is neither 0 nor 1 — malformed \
                     scan or corrupted share; refusing rather than dropping/forging a row",
                    sel.value()
                ),
                cheaters: Vec::new(),
            });
        }
    }
    canonicalize_rows(&mut rows_out);

    let cost = SortMergeCost {
        union_size: n,
        sort_compare_exchanges: net.gate_count(),
        sort_depth_rounds: net.depth(),
        scan_mults,
        opens,
        shuffle: shuffle_cost,
    };
    Ok((
        PartialResult {
            holder: HolderId::new("federation"),
            vars: out_vars,
            rows: rows_out,
        },
        cost,
    ))
}

/// **The secret-key oblivious sort substrate** (the P6 sort the matrix names as the
/// remaining integration step). Sort a column of secret-shared field values ascending
/// via the data-independent Batcher odd-even mergesort access pattern
/// ([`SortingNetwork`], sq-18lk), each compare-exchange decided by the landed secure
/// comparator ([`secure_greater_than_shared`], a NEVER-opened secret verdict) and
/// applied by a **secret-bit-gated arithmetic conditional swap**. Returns the sorted
/// sharings — the sorted VALUES are never opened; the only openings are the
/// comparator's per-compare masked Rabbit decompositions (`c = x + r mod p`,
/// near-uniform / statistically hiding at `≤ 2^{-61}` each, composing over the
/// `O(n log² n)` gates — see the module leakage statement), so the caller holds a
/// re-ordered secret column whose order is STATISTICALLY hidden (not
/// information-theoretically independent) from any `≤ t` collusion.
///
/// This is exposed for the composed operators (and their differential tests) that
/// need only the sort — DISTINCT-over-hidden, ORDER BY, MIN/MAX, the sort-merge
/// join's own union sort. Values must be `< 2^60` (comparator range, fail-closed).
/// `n >= 2t+1`. Honest-majority, semi-honest (sq-qhy4 pending).
pub fn oblivious_sort_by_secret_key(
    backend: &ShamirBackend,
    dealer: &mut ShamirDealer,
    mut col: Vec<Vec<Share>>,
) -> Result<Vec<Vec<Share>>, MpcError> {
    let n = col.len();
    let net = SortingNetwork::new(n);
    for &(i, j) in net.compare_exchanges() {
        let (lo, hi) = (i.min(j), i.max(j));
        let swap_bit = secure_greater_than_shared(dealer, backend, &col[lo], &col[hi])?;
        // x_lo' = x_lo + b·(x_hi − x_lo); x_hi' = x_hi − b·(x_hi − x_lo).
        let delta = shamir::sub_shares(&col[hi], &col[lo])?;
        let step = secure_mul(dealer, &swap_bit, &delta)?;
        col[lo] = shamir::add_shares(&col[lo], &step)?;
        col[hi] = shamir::sub_shares(&col[hi], &step)?;
    }
    Ok(col)
}

/// Range-check a key against the comparator's injective window (`< 2^60`), failing
/// closed so an out-of-range key never yields a silently-wrapped comparison.
fn check_key_range(side: &str, idx: usize, key: Fp) -> Result<(), MpcError> {
    if key.value() >= COMPARE_MAX_EXCLUSIVE {
        return Err(MpcError::Protocol(format!(
            "sort_merge_semi_anti: {side} key #{idx} = {} is out of range (must be < 2^60 = {} so \
             the in-MPC bit-decomposition comparator is injective — use a <=60-bit key encoding)",
            key.value(),
            COMPARE_MAX_EXCLUSIVE
        )));
    }
    Ok(())
}

/// General secure multiply of two degree-`t` sharings: one [`shamir::mul_shares_raw`]
/// (→ degree `2t`) + one [`ShamirDealer::degree_reduce`] back to a fresh degree-`t`
/// sharing (the mult-chaining step, sq-dvuc). Used for logical AND (`a·b` on 0/1
/// operands) AND for the conditional swap's `b·(x_j − x_i)` (`b` a 0/1 bit, the
/// delta a general field value) — a product is a product.
fn secure_mul(
    dealer: &mut ShamirDealer,
    a: &[Share],
    b: &[Share],
) -> Result<Vec<Share>, MpcError> {
    let prod_2t = shamir::mul_shares_raw(a, b)?;
    dealer.degree_reduce(&prod_2t)
}

/// Secret OR of two 0/1 sharings: `a OR b = a + b − a·b` (one secure multiplication,
/// the rest local). Correct for 0/1 operands.
fn secure_or(
    dealer: &mut ShamirDealer,
    a: &[Share],
    b: &[Share],
) -> Result<Vec<Share>, MpcError> {
    let and = secure_mul(dealer, a, b)?;
    let sum = shamir::add_shares(a, b)?;
    shamir::sub_shares(&sum, &and)
}

/// Secret-bit-gated conditional swap of two whole [`Row`]s: for each carried field
/// `x_i, x_j`, `x_i' = x_i + b·(x_j − x_i)` and `x_j' = x_j − b·(x_j − x_i)`. With a
/// secret bit `b ∈ {0,1}` this is a swap when `b = 1` and identity when `b = 0`,
/// opening nothing and conserving the pair (`x_i' + x_j' = x_i + x_j`). One secure
/// multiplication per carried field (here 3: key, is_l, pid).
fn conditional_swap(
    dealer: &mut ShamirDealer,
    rows: &mut [Row],
    i: usize,
    j: usize,
    b: &[Share],
) -> Result<(), MpcError> {
    // Field-by-field so we borrow one column pair at a time.
    swap_field(dealer, b, |r| &r.key, |r, v| r.key = v, rows, i, j)?;
    swap_field(dealer, b, |r| &r.is_l, |r, v| r.is_l = v, rows, i, j)?;
    swap_field(dealer, b, |r| &r.pid, |r, v| r.pid = v, rows, i, j)?;
    Ok(())
}

/// Apply the conditional swap to ONE field of the two rows, selected by `get`/`set`.
fn swap_field(
    dealer: &mut ShamirDealer,
    b: &[Share],
    get: impl Fn(&Row) -> &Vec<Share>,
    set: impl Fn(&mut Row, Vec<Share>),
    rows: &mut [Row],
    i: usize,
    j: usize,
) -> Result<(), MpcError> {
    let delta = shamir::sub_shares(get(&rows[j]), get(&rows[i]))?; // x_j − x_i
    let step = secure_mul(dealer, b, &delta)?; // b·(x_j − x_i)
    let new_i = shamir::add_shares(get(&rows[i]), &step)?;
    let new_j = shamir::sub_shares(get(&rows[j]), &step)?;
    set(&mut rows[i], new_i);
    set(&mut rows[j], new_j);
    Ok(())
}

/// Sort rows into a canonical order (by their `{:?}` term tuple) so the disclosed
/// result multiset is independent of the shuffle / row ordering — the recipient
/// recomputes any ORDER BY over the disclosed multiset (convention #4).
fn canonicalize_rows(rows: &mut [Vec<Option<Term>>]) {
    rows.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shamir::ShamirBackend;

    fn var(n: &str) -> Variable {
        Variable::new_unchecked(n)
    }

    fn lit(s: &str) -> Option<Term> {
        Some(Term::Literal(oxrdf::Literal::new_simple_literal(s)))
    }

    /// The real (non-dummy) rows of a result, as an order-independent multiset of
    /// debug strings.
    fn multiset(rows: &[Vec<Option<Term>>]) -> Vec<String> {
        let mut m: Vec<String> = rows.iter().map(|r| format!("{r:?}")).collect();
        m.sort();
        m
    }

    /// Plaintext oracle: the semi-/anti-join of L against R's key set (the
    /// `minus_bindings` semantics for anti on a single shared key).
    fn plaintext_expected(
        left: &[(Fp, Vec<Option<Term>>)],
        right_keys: &[Fp],
        kind: SortMergeJoinKind,
    ) -> Vec<String> {
        let rset: std::collections::HashSet<u64> = right_keys.iter().map(|k| k.value()).collect();
        let mut m: Vec<String> = left
            .iter()
            .filter(|(k, _)| match kind {
                SortMergeJoinKind::Semi => rset.contains(&k.value()),
                SortMergeJoinKind::Anti => !rset.contains(&k.value()),
            })
            .map(|(_, p)| format!("{p:?}"))
            .collect();
        m.sort();
        m
    }

    // ---- oblivious sort substrate: reconstruct-after-sort == plaintext sort -------

    #[test]
    fn oblivious_secret_key_sort_matches_plaintext_sort() {
        for seed in 0..8u64 {
            let backend = ShamirBackend::new_seeded(3, 100 + seed).unwrap();
            let mut dealer = backend.dealer();
            let vals: Vec<u64> = vec![7, 2, 9, 2, 5, 0, 5, 3];
            let col: Vec<Vec<Share>> = vals.iter().map(|&v| dealer.share(Fp::new(v))).collect();
            let sorted = oblivious_sort_by_secret_key(&backend, &mut dealer, col).unwrap();
            let got: Vec<u64> = sorted
                .iter()
                .map(|s| backend.reconstruct(s).unwrap().value())
                .collect();
            let mut expected = vals.clone();
            expected.sort_unstable();
            assert_eq!(got, expected, "seed {seed}: secret-key sort wrong");
        }
    }

    /// The sort ACCESS PATTERN is data-independent (a function of `n` only) — the
    /// obliviousness substrate. Two different data columns of the same length drive
    /// the IDENTICAL compare-exchange sequence.
    #[test]
    fn sort_access_pattern_is_data_independent() {
        let a = SortingNetwork::new(8);
        let b = SortingNetwork::new(8);
        assert_eq!(a.compare_exchanges(), b.compare_exchanges());
    }

    // ---- differential: semi-join == plaintext semi-join --------------------------

    #[test]
    fn semi_join_matches_plaintext() {
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        // L keys: 1,2,3,4 ; R keys: 3,1,1 → present {1,3}. Semi keeps rows 1 and 3.
        let left = vec![
            (Fp::new(1), vec![lit("a")]),
            (Fp::new(2), vec![lit("b")]),
            (Fp::new(3), vec![lit("c")]),
            (Fp::new(4), vec![lit("d")]),
        ];
        let right = [Fp::new(3), Fp::new(1), Fp::new(1)];
        let (res, cost) =
            sort_merge_semi_anti(&backend, &left, &right, vec![var("x")], SortMergeJoinKind::Semi)
                .unwrap();
        assert_eq!(res.holder, HolderId::new("federation"));
        assert_eq!(
            multiset(&res.rows),
            plaintext_expected(&left, &right, SortMergeJoinKind::Semi)
        );
        assert_eq!(multiset(&res.rows), vec![format!("{:?}", vec![lit("a")]), format!("{:?}", vec![lit("c")])]);
        assert_eq!(cost.union_size, 7);
    }

    // ---- differential: anti-join == plaintext anti-join (MINUS semantics) ---------

    #[test]
    fn anti_join_matches_plaintext_minus() {
        let backend = ShamirBackend::new_seeded(3, 2).unwrap();
        let left = vec![
            (Fp::new(1), vec![lit("a")]),
            (Fp::new(2), vec![lit("b")]),
            (Fp::new(3), vec![lit("c")]),
            (Fp::new(4), vec![lit("d")]),
        ];
        let right = [Fp::new(3), Fp::new(1), Fp::new(1)];
        // Anti keeps the L rows whose key is NOT in R: keys 2 and 4 → rows b, d.
        let (res, _) =
            sort_merge_semi_anti(&backend, &left, &right, vec![var("x")], SortMergeJoinKind::Anti)
                .unwrap();
        assert_eq!(
            multiset(&res.rows),
            plaintext_expected(&left, &right, SortMergeJoinKind::Anti)
        );
        assert_eq!(multiset(&res.rows), vec![format!("{:?}", vec![lit("b")]), format!("{:?}", vec![lit("d")])]);
    }

    /// Randomised differential across many key layouts, party counts and seeds —
    /// the sort-merge result must equal the plaintext oracle for BOTH kinds,
    /// including duplicate keys (fan-out groups) and disjoint / identical sides.
    #[test]
    fn randomised_differential_semi_and_anti() {
        // A few hand-varied layouts (no RNG in tests — vary by index, per harness).
        let layouts: Vec<(Vec<u64>, Vec<u64>)> = vec![
            (vec![1, 2, 3], vec![]),                 // empty R: semi=∅, anti=all
            (vec![], vec![1, 2]),                    // empty L: both ∅
            (vec![5, 5, 5], vec![5]),                // all L share one key present in R
            (vec![5, 5, 5], vec![9]),                // group absent from R
            (vec![1, 2, 2, 3, 4], vec![2, 4, 4, 6]), // mixed duplicates both sides
            (vec![10, 20, 30], vec![30, 20, 10]),    // identical sets, permuted
        ];
        for (li, (lk, rk)) in layouts.iter().enumerate() {
            for parties in [3usize, 5] {
                let backend = ShamirBackend::new_seeded(parties, 500 + li as u64).unwrap();
                let left: Vec<(Fp, Vec<Option<Term>>)> = lk
                    .iter()
                    .enumerate()
                    .map(|(i, &k)| (Fp::new(k), vec![lit(&format!("L{i}"))]))
                    .collect();
                let right: Vec<Fp> = rk.iter().map(|&k| Fp::new(k)).collect();
                for kind in [SortMergeJoinKind::Semi, SortMergeJoinKind::Anti] {
                    let (res, _) = sort_merge_semi_anti(
                        &backend,
                        &left,
                        &right,
                        vec![var("x")],
                        kind,
                    )
                    .unwrap();
                    assert_eq!(
                        multiset(&res.rows),
                        plaintext_expected(&left, &right, kind),
                        "layout {li} parties {parties} kind {kind:?}: sort-merge != plaintext"
                    );
                }
            }
        }
    }

    /// The revealed result MULTISET is invariant to the oblivious shuffle / seed —
    /// correctness holds no matter which permutation the shuffle drew (the shuffle's
    /// order-hiding obliviousness itself is the substrate property tested in
    /// `oblivious.rs`; here the operator canonicalises its output, so we assert the
    /// seed-invariant multiset, not an order).
    #[test]
    fn result_multiset_is_shuffle_invariant() {
        let left = vec![
            (Fp::new(1), vec![lit("p0")]),
            (Fp::new(2), vec![lit("p1")]),
            (Fp::new(3), vec![lit("p2")]),
            (Fp::new(4), vec![lit("p3")]),
        ];
        let right = [Fp::new(1), Fp::new(2), Fp::new(3), Fp::new(4)]; // all present
        let expected = plaintext_expected(&left, &right, SortMergeJoinKind::Semi);
        for seed in 0..8u64 {
            let backend = ShamirBackend::new_seeded(3, 700 + seed).unwrap();
            let (res, _) = sort_merge_semi_anti(
                &backend,
                &left,
                &right,
                vec![var("x")],
                SortMergeJoinKind::Semi,
            )
            .unwrap();
            assert_eq!(multiset(&res.rows), expected, "seed {seed}: multiset changed");
        }
    }

    // ---- real randomized differential + mutation (non-vacuity) check -------------

    /// Deterministic SplitMix64 — a per-index-seeded PRNG so the differential is
    /// *generated* (many lengths / seeds / key multisets) yet fully reproducible, with
    /// no `Math::random`-style nondeterminism (harness convention: vary by seed index).
    fn splitmix64(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Generate one L/R key-multiset layout for `seed`, deliberately spanning the
    /// coverage the hand-picked layouts miss: arbitrary lengths (incl. empty sides),
    /// heavy duplicate/collision domains (small key space → fan-out groups + shared
    /// keys), AND keys near the supported upper bound `2^60 − 1` (max Rabbit width).
    fn generate_layout(seed: u64) -> (Vec<u64>, Vec<u64>) {
        let mut s = seed.wrapping_mul(0x2545_F491_4F6C_DD1D).wrapping_add(1);
        let ll = (splitmix64(&mut s) % 7) as usize; // 0..=6 (empty L reachable)
        let rr = (splitmix64(&mut s) % 7) as usize; // 0..=6 (empty R reachable)
        let key = |s: &mut u64| -> u64 {
            let r = splitmix64(s);
            match r % 5 {
                // Tiny domain → many duplicates / shared keys across sides.
                0 | 1 => r % 4,
                2 => r % 12,
                // Near the supported upper bound (exercises full 60-bit Rabbit width).
                3 => (COMPARE_MAX_EXCLUSIVE - 1) - (r % 3),
                _ => r % 40,
            }
        };
        let lk: Vec<u64> = (0..ll).map(|_| key(&mut s)).collect();
        let rk: Vec<u64> = (0..rr).map(|_| key(&mut s)).collect();
        (lk, rk)
    }

    /// A REAL generated randomized differential: for many generated layouts (varying
    /// lengths, seeds, duplicate/collision patterns, empty sides, and upper-bound
    /// values), the actual `sort_merge_semi_anti` output MUST equal the plaintext
    /// oracle for BOTH kinds and several party counts. This is the coverage the six
    /// fixed layouts do not provide.
    #[test]
    fn real_randomised_differential_generated() {
        let mut ran = 0usize;
        for seed in 0..24u64 {
            let (lk, rk) = generate_layout(seed);
            let left: Vec<(Fp, Vec<Option<Term>>)> = lk
                .iter()
                .enumerate()
                .map(|(i, &k)| (Fp::new(k), vec![lit(&format!("L{i}"))]))
                .collect();
            let right: Vec<Fp> = rk.iter().map(|&k| Fp::new(k)).collect();
            // Party count varies with the seed (both honest-majority sizes exercised).
            let parties = if seed % 4 == 0 { 5 } else { 3 };
            let backend = ShamirBackend::new_seeded(parties, 900 + seed).unwrap();
            for kind in [SortMergeJoinKind::Semi, SortMergeJoinKind::Anti] {
                let (res, _) =
                    sort_merge_semi_anti(&backend, &left, &right, vec![var("x")], kind).unwrap();
                assert_eq!(
                    multiset(&res.rows),
                    plaintext_expected(&left, &right, kind),
                    "seed {seed} parties {parties} kind {kind:?} \
                     (L={lk:?} R={rk:?}): sort-merge != plaintext oracle"
                );
                ran += 1;
            }
        }
        assert!(ran >= 40, "expected a broad generated sweep, ran only {ran}");
    }

    /// The named corruption points of the novel sort/scan, injected into a cleartext
    /// MODEL of the exact pipeline so a mutation test can prove the oracle differential
    /// is NON-VACUOUS (it goes red on a wrong answer) without threading `cfg(test)`
    /// fault hooks through the secret-shared crypto core.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Mutation {
        /// No corruption — must reproduce the production output and the oracle.
        None,
        /// Emit R rows instead of L rows (the `is_l AND cond` selector gated wrong).
        Selector,
        /// The adjacency-equality bit is stuck at 0 (equal keys never seen as a run).
        AdjacencyEquality,
        /// Only the forward OR pass runs (the backward scan direction is dropped).
        ScanDirection,
        /// The oblivious sort's compare-exchanges never fire (swap decision corrupted),
        /// so equal keys across the L/R boundary are no longer adjacent.
        SwapDecision,
    }

    /// Cleartext model of the EXACT sort-merge pipeline (tagged union → oblivious sort
    /// by key → adjacency equalities → forward/backward OR scan → `is_l AND cond`
    /// selector), returning the kept-L-payload multiset. `Mutation::None` mirrors the
    /// secret-shared implementation step-for-step; the other variants corrupt one named
    /// step. Validated equal to the production output on every generated input by
    /// [`model_matches_production`], so it is a faithful stand-in for the mutation test.
    fn sort_merge_model(
        left: &[(Fp, Vec<Option<Term>>)],
        right_keys: &[Fp],
        kind: SortMergeJoinKind,
        mutation: Mutation,
    ) -> Vec<String> {
        // Tagged union: (key, is_l, pid).
        let mut rows: Vec<(u64, bool, usize)> = Vec::new();
        for (i, (k, _)) in left.iter().enumerate() {
            rows.push((k.value(), true, i));
        }
        for k in right_keys {
            rows.push((k.value(), false, 0));
        }
        // (1) oblivious sort by key ascending — unless the swap decisions are corrupted.
        if mutation != Mutation::SwapDecision {
            rows.sort_by_key(|&(k, _, _)| k);
        }
        let n = rows.len();
        // (2) adjacency equalities eq[k] = key_k == key_{k+1}.
        let eq: Vec<bool> = (0..n.saturating_sub(1))
            .map(|k| match mutation {
                Mutation::AdjacencyEquality => false,
                _ => rows[k].0 == rows[k + 1].0,
            })
            .collect();
        let is_r: Vec<bool> = rows.iter().map(|&(_, is_l, _)| !is_l).collect();
        // (3) backward + forward OR passes (broadcast "an R shares my run").
        let mut b_r = vec![false; n];
        let mut f_r = vec![false; n];
        if n > 0 {
            b_r[n - 1] = is_r[n - 1];
            for k in (0..n - 1).rev() {
                b_r[k] = is_r[k] || (eq[k] && b_r[k + 1]);
            }
            f_r[0] = is_r[0];
            for k in 1..n {
                f_r[k] = is_r[k] || (eq[k - 1] && f_r[k - 1]);
            }
        }
        // (4) selector: keep iff is_l AND the semi/anti condition on "any R in my run".
        let mut out: Vec<String> = Vec::new();
        for k in 0..n {
            let (_, is_l, pid) = rows[k];
            let any_r = match mutation {
                // Drop the backward pass: an L before its R match is no longer seen.
                Mutation::ScanDirection => f_r[k],
                _ => b_r[k] || f_r[k],
            };
            let cond = match kind {
                SortMergeJoinKind::Semi => any_r,
                SortMergeJoinKind::Anti => !any_r,
            };
            // The `is_l AND cond` gate — corrupted to gate on is_r under Selector.
            let gate = match mutation {
                Mutation::Selector => !is_l,
                _ => is_l,
            };
            if gate && cond {
                out.push(format!("{:?}", left[pid].1));
            }
        }
        out.sort();
        out
    }

    /// The cleartext model is faithful: with NO mutation it equals BOTH the plaintext
    /// oracle AND the real secret-shared `sort_merge_semi_anti` output on every
    /// generated input — so mutating the model is a valid proxy for corrupting the
    /// production sort/scan.
    #[test]
    fn model_matches_production() {
        for seed in 0..16u64 {
            let (lk, rk) = generate_layout(seed);
            let left: Vec<(Fp, Vec<Option<Term>>)> = lk
                .iter()
                .enumerate()
                .map(|(i, &k)| (Fp::new(k), vec![lit(&format!("L{i}"))]))
                .collect();
            let right: Vec<Fp> = rk.iter().map(|&k| Fp::new(k)).collect();
            let backend = ShamirBackend::new_seeded(3, 800 + seed).unwrap();
            for kind in [SortMergeJoinKind::Semi, SortMergeJoinKind::Anti] {
                let oracle = plaintext_expected(&left, &right, kind);
                assert_eq!(
                    sort_merge_model(&left, &right, kind, Mutation::None),
                    oracle,
                    "seed {seed} kind {kind:?}: model(None) != oracle"
                );
                let (res, _) =
                    sort_merge_semi_anti(&backend, &left, &right, vec![var("x")], kind).unwrap();
                assert_eq!(
                    multiset(&res.rows),
                    oracle,
                    "seed {seed} kind {kind:?}: production != oracle"
                );
            }
        }
    }

    /// Mutation / non-vacuity check: each named corruption of the sort/scan (a wrong
    /// selector, a stuck adjacency equality, a dropped scan direction, a corrupted
    /// swap decision) MUST make the pipeline disagree with the plaintext oracle on at
    /// least one generated input — i.e. the differential provably turns RED on a
    /// deliberately wrong answer. Because the unmutated model is validated equal to the
    /// production output ([`model_matches_production`]), a real corruption of any of
    /// these steps would likewise make `sort_merge_semi_anti` fail the oracle.
    #[test]
    fn mutations_turn_the_oracle_differential_red() {
        let mutations = [
            Mutation::Selector,
            Mutation::AdjacencyEquality,
            Mutation::ScanDirection,
            Mutation::SwapDecision,
        ];
        for &mutation in &mutations {
            let mut caught = false;
            'search: for seed in 0..64u64 {
                let (lk, rk) = generate_layout(seed);
                let left: Vec<(Fp, Vec<Option<Term>>)> = lk
                    .iter()
                    .enumerate()
                    .map(|(i, &k)| (Fp::new(k), vec![lit(&format!("L{i}"))]))
                    .collect();
                let right: Vec<Fp> = rk.iter().map(|&k| Fp::new(k)).collect();
                for kind in [SortMergeJoinKind::Semi, SortMergeJoinKind::Anti] {
                    let oracle = plaintext_expected(&left, &right, kind);
                    // Sanity: the SAME harness with no mutation still matches (so the
                    // divergence below is due to the mutation, not the input).
                    assert_eq!(
                        sort_merge_model(&left, &right, kind, Mutation::None),
                        oracle,
                        "unmutated model diverged — bad control for {mutation:?}"
                    );
                    if sort_merge_model(&left, &right, kind, mutation) != oracle {
                        caught = true;
                        break 'search;
                    }
                }
            }
            assert!(
                caught,
                "{mutation:?}: oracle differential did NOT go red on any generated \
                 input — the test would be vacuous against this corruption"
            );
        }
    }

    // ---- fail-closed contracts ---------------------------------------------------

    #[test]
    fn out_of_range_key_fails_closed() {
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        let big = Fp::new(COMPARE_MAX_EXCLUSIVE); // == 2^60, out of range
        let left = vec![(big, vec![lit("a")])];
        let right = [Fp::new(1)];
        let err = sort_merge_semi_anti(&backend, &left, &right, vec![var("x")], SortMergeJoinKind::Semi)
            .unwrap_err();
        assert!(matches!(err, MpcError::Protocol(m) if m.contains("out of range")));
    }

    #[test]
    fn payload_arity_mismatch_fails_closed() {
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        let left = vec![(Fp::new(1), vec![lit("a"), lit("b")])]; // arity 2
        let right = [Fp::new(1)];
        let err = sort_merge_semi_anti(&backend, &left, &right, vec![var("x")], SortMergeJoinKind::Semi)
            .unwrap_err();
        assert!(matches!(err, MpcError::Protocol(m) if m.contains("arity")));
    }

    #[test]
    fn empty_sides_are_handled() {
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        // Empty L → empty result for both kinds.
        let (res, cost) =
            sort_merge_semi_anti(&backend, &[], &[Fp::new(1)], vec![var("x")], SortMergeJoinKind::Anti)
                .unwrap();
        assert!(res.rows.is_empty());
        assert_eq!(cost.union_size, 1);
    }
}
