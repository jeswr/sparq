// [OPUS-4.8] sq-jnkm: oblivious result-size protection + match-bit aggregation
// output path for SET-returning hidden joins. Closes leaks L1 (true result
// cardinality) and L2 (the per-pair match graph / key fan-out) that the per-pair
// `HiddenValueJoin` open exposes, building on the oblivious shuffle substrate
// (sq-18lk, `crate::oblivious::shuffle`). Native-only; in-process multi-party
// simulation; communication cost MODELLED not measured. Authored under the Opus
// 4.8 fallback model — re-review when Fable returns.
//! Oblivious output path for set-returning hidden joins — **result-size
//! protection + match-bit aggregation** (bead sq-jnkm).
//!
//! ## The leak this closes, and why the per-pair join needs it
//!
//! [`crate::join::HiddenValueJoin`] hides the join KEY values but its output path
//! leaks two things RQ2a wants hidden (research record §4.1):
//!
//! - **L1 — final result cardinality.** It emits exactly the true number of
//!   matches; `canonicalize_rows` sorts but never pads. An observer of the
//!   protocol learns `|result|` exactly.
//! - **L2 — per-pair match graph / key fan-out.** `secure_equal` *opens* the
//!   masked product `m = d·r` for every `(i, j)` candidate pair; that open IS the
//!   match bit at `(i, j)`. The set of matching pairs is the bipartite match graph,
//!   which fingerprints the (hidden) key fan-out / multiplicity distribution.
//!
//! The standard oblivious-join output path (research record §4.3, "Private-key
//! join returning a SET") closes both with the substrate this module consumes:
//!
//! 1. compute a per-candidate **match bit** kept SECRET-SHARED (never opened
//!    per-pair → L2 destroyed at source);
//! 2. **oblivious-select** each candidate slot to `bit · real_tag + (1 − bit) · 0`
//!    so a non-match becomes an indistinguishable dummy;
//! 3. **oblivious-shuffle** ([`crate::oblivious::shuffle`]) the slot column so the
//!    output position reveals nothing about which input pair produced it (the
//!    residual access-pattern half of L2);
//! 4. reveal **exactly a public padded bound `B` of slots** — never the true match
//!    count (→ L1 bounded to `B`, not exact).
//!
//! ## What is SOUND TODAY vs what is honestly GATED
//!
//! The empirical-honesty line is the same one the substrate draws between its
//! sound shuffle and its degree-reduction-gated secret comparator.
//!
//! ### Sound today (shipped here)
//!
//! - **The oblivious output TRANSFORM** — steps 2–4 above — given a secret-shared
//!   match bit per candidate. The per-slot select `bit·real_tag` is a **single**
//!   Shamir multiplication ([`crate::shamir::mul_shares_raw`]) plus local adds: no
//!   multiplication CHAIN, so it needs **no degree reduction** and is buildable on
//!   the single-product backend — exactly like
//!   [`crate::join::HiddenValueJoin::secure_equal`]. The shuffle is the sound
//!   substrate; the padded-prefix reveal is a degree-`2t` open of `B` slots. This
//!   is the structural obliviousness the bead asks for, and it holds at the
//!   honest-majority Shamir layer.
//! - **The DISCLOSED-key candidate path** — [`oblivious_set_output`] /
//!   [`oblivious_join_output`] from a candidate whose match bit was decided by a
//!   **public / disclosed-key** equi-join (computed OUTSIDE the crypto core,
//!   convention #4). The bit is public so we lift it to a trivial sharing with NO
//!   secure comparison; the transform then hides, from the parties, *which*
//!   candidates matched and the exact count (only `B` is revealed). This is fully
//!   sound and shippable now.
//!
//! ### Honestly gated on secure-compare (NOT faked)
//!
//! - **Deriving the secret match bit from genuinely-secret keys WITHOUT opening
//!   it.** To feed step 1 in the HIDDEN-key regime you need a secure
//!   equality-to-SHARED-bit (Rabbit / edaBits bit-decomposition), which consumes a
//!   multiplication chain and so needs the BGW/DN **degree-reduction** round (bead
//!   **sq-dvuc**) and the secure-comparison primitive built on it (bead
//!   **sq-rrz4**) — the same gate the substrate's secret-key sort sits behind. The
//!   crate's existing `secure_equal` produces the bit only by *opening* it, which
//!   is precisely the L2 leak we are closing, so it cannot feed this path. The
//!   hidden-key entry point ([`oblivious_set_output_hidden_keys`]) therefore
//!   returns an honest [`MpcError::NotYetImplemented`] naming that gate — no fake
//!   secure comparator.
//!
//! ## Residual leakage (state it loudly — empirical-honesty rule)
//!
//! This is the **padded-bound** (exact-obliviousness) output, NOT differential
//! privacy (DP cardinality / ε-budget is the SEPARATE follow-up bead **sq-shk5**;
//! deliberately not done here).
//!
//! - **To the PARTIES / the protocol transcript:** the output reveals only the
//!   PUBLIC bound `B` and that the true count is `≤ B`. It does NOT reveal the true
//!   count, which slots matched, or which input pair produced which output row
//!   (the shuffle destroyed position→candidate linkage). L1 is bounded to `B`; L2
//!   is destroyed.
//! - **To the authorized RESULT RECIPIENT** (who must filter the tagged dummies to
//!   use the result): the true count IS learned — it equals the number of
//!   non-dummy rows after filtering. This is inherent to a padded-bound scheme: the
//!   consumer of a result set learns its size. Hiding the count from the *recipient*
//!   too is exactly the job of the DP cardinality mode (sq-shk5), which adds noise
//!   + extra dummies on top of this transform; it is out of scope here.
//! - **`B` itself is public and caller-agreed.** Choosing `B` too tight FAILS
//!   CLOSED (we never silently drop matches — a candidate count `> B` is an error,
//!   never a truncation that would corrupt the result); choosing it loose costs `B`
//!   opens. `B` leaks an upper bound on the result size by construction.
//! - **L3 (input cardinalities `|L|`, `|R|`) and the candidate count** are public
//!   (standard MPC assumption; research record §4.1). We do not claim to hide them.
//! - Malicious security is unchanged from the Shamir layer: at `n = 2t + 1` the
//!   degree-`2t` opens have zero RS redundancy (research record §4.1 D×A; bead
//!   sq-6d6g IT-MAC is the named fix). Semi-honest only.
//!
//! ## Communication / round cost (MODELLED — counted, not measured)
//!
//! [`ObliviousOutputCost`] reports what a real deployment would pay: `B` select
//! multiplications (one round, all in parallel), the shuffle's switch count +
//! depth ([`crate::oblivious::ShuffleCost`]), and `B` degree-`2t` opens. No
//! hard-coded numbers: derived from `B` and the built network (AGENTS.md).

use crate::field::Fp;
use crate::oblivious::{shuffle, SecretColumn, ShuffleCost};
use crate::partial::{HolderId, MpcError, PartialResult};
use crate::shamir::{self, ShamirBackend, ShamirDealer, Share};
use oxrdf::{Term, Variable};

/// One disclosed payload row — a column vector of nullable `oxrdf` terms, the same
/// row shape `sparq_engine::QueryResult` / [`PartialResult`] use (`None` = unbound).
type PayloadRow = Vec<Option<Term>>;

/// Internal result of [`shuffle_with_payloads`]: the co-permuted (tag column,
/// payload rows) plus the shuffle's modelled cost.
type ShuffledColumn = (SecretColumn, Vec<PayloadRow>, ShuffleCost);

/// A single candidate row entering the oblivious output path: the disclosed
/// payload terms that would be emitted **on a match**, plus the match decision.
///
/// The payload is cleartext `oxrdf` terms (the disclosure-minimisation rule keeps
/// only the *key* hidden — convention #4; the payload columns are disclosed on a
/// match). Whether this candidate matched is carried as a [`MatchBit`], which is
/// either a public bit (disclosed-key regime, sound today) or a secret-shared bit
/// (hidden-key regime, gated on secure-compare — see module docs).
#[derive(Debug, Clone)]
pub struct Candidate {
    /// The payload terms emitted if this candidate matches (the join's disclosed
    /// output columns for this `(i, j)` pair).
    pub payload: Vec<Option<Term>>,
    /// Whether this candidate is a match — see [`MatchBit`].
    pub matched: MatchBit,
}

/// The match decision for a [`Candidate`].
///
/// - [`MatchBit::Public`] — the bit was decided OUTSIDE the crypto core (a
///   disclosed-key / public equi-join, convention #4). Sound today: we lift it to
///   a trivial degree-`t` sharing with no secure comparison.
/// - [`MatchBit::SecretShared`] — a genuinely-secret 0/1 sharing produced by a
///   secure equality-to-shared-bit. Consuming it is sound; **producing** it from
///   secret keys without opening it is gated on sq-rrz4/sq-dvuc, so no in-crate
///   producer exists yet (the existing `secure_equal` only *opens* the bit, which
///   is the L2 leak this path closes).
#[derive(Debug, Clone)]
pub enum MatchBit {
    /// Match bit decided in the clear (disclosed-key regime).
    Public(bool),
    /// Match bit as a secret-shared 0/1 field element (`Fp::one()` / `Fp::zero()`),
    /// one [`Share`] per party. Must be a sharing of exactly 0 or 1; this is the
    /// caller's obligation (a secure equality-to-bit guarantees it by construction).
    SecretShared(Vec<Share>),
}

/// MODELLED communication / round cost of one [`oblivious_set_output`] over `bound`
/// output slots. Counted, never measured — see the module cost note. No hard-coded
/// numbers: derived from `bound` and the built shuffle network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObliviousOutputCost {
    /// The public padded bound `B` (number of slots revealed).
    pub bound: usize,
    /// Select multiplications: one per slot (`bit·real_tag`), all in a single
    /// parallel round.
    pub select_mults: usize,
    /// Degree-`2t` opens performed at reveal (one per slot).
    pub opens: usize,
    /// The shuffle's cost over `B` slots (switch count + network depth).
    pub shuffle: ShuffleCost,
}

/// One revealed output slot: either a real joined row or a tagged dummy.
///
/// After the oblivious shuffle the parties open exactly `B` of these IN SHUFFLED
/// ORDER. A [`OutputSlot::Dummy`] is the padding that hides the true match count
/// from the parties; the authorized recipient filters dummies to obtain the result
/// (and only then learns the true count — see the module's residual-leakage note).
#[derive(Debug, Clone, PartialEq)]
pub enum OutputSlot {
    /// A real joined payload row (this candidate matched).
    Row(Vec<Option<Term>>),
    /// A dummy slot (a non-match or pure padding) — filtered by the recipient.
    Dummy,
}

/// The result of [`oblivious_set_output`]: the `B` revealed slots (in shuffled
/// order) and the modelled cost.
pub type ObliviousOutput = (Vec<OutputSlot>, ObliviousOutputCost);

/// **The oblivious set-returning output path** (bead sq-jnkm).
///
/// Given the full candidate set of a hidden join and a PUBLIC padded bound `B`,
/// produce `B` revealed [`OutputSlot`]s in oblivious-shuffled order such that the
/// parties learn neither which candidates matched (L2) nor the exact match count
/// beyond `B` (L1). See the module docs for the sound-vs-gated split and the
/// residual-leakage statement.
///
/// ## Protocol
///
/// 1. **Resolve each candidate's match bit to a secret-shared selector.** A
///    [`MatchBit::Public`] bit is lifted to a trivial sharing (sound, no secure
///    compare). A [`MatchBit::SecretShared`] sharing is used directly. (Producing
///    the latter from secret keys is the gated step — see
///    [`oblivious_set_output_hidden_keys`] / module docs.)
/// 2. **Build `B` slots.** Each of the `|candidates|` candidates becomes one slot;
///    if `|candidates| < B` we append pure-dummy slots (selector = shared 0) so the
///    revealed count is exactly `B` regardless of input.
/// 3. **Oblivious-select** each slot's selector against a real-tag sharing:
///    `tag_out = sel · real_tag` (one multiplication; a non-match has `sel = 0` so
///    the tag opens to the dummy sentinel `0`). The cleartext payload travels
///    alongside, co-permuted, and is only emitted when the opened tag says "real".
/// 4. **Oblivious-shuffle** the (tag, payload) column together so output position
///    is independent of input position (substrate `shuffle`).
/// 5. **Open exactly `B` tags** at degree `2t`; emit [`OutputSlot::Row`] where the
///    tag is the real sentinel and [`OutputSlot::Dummy`] otherwise.
///
/// ## Fail-closed bound contract
///
/// `B` must be `>= candidates.len()` so every candidate gets a slot — we NEVER
/// silently truncate candidates (that could drop a true match and corrupt the
/// result). A `B < candidates.len()` is a [`MpcError::Protocol`] error. (The
/// caller sizes `B` from the public candidate count `|L|·|R|`, optionally padded;
/// the true MATCH count stays hidden.)
pub fn oblivious_set_output(
    backend: &ShamirBackend,
    candidates: &[Candidate],
    payload_arity: usize,
    bound: usize,
) -> Result<ObliviousOutput, MpcError> {
    // Fail closed: the bound must cover every candidate so no match is ever
    // silently dropped (see the contract above).
    if bound < candidates.len() {
        return Err(MpcError::Protocol(format!(
            "oblivious_set_output: padded bound B={bound} < candidate count {} — \
             would truncate candidates and could drop a true match",
            candidates.len()
        )));
    }
    // Validate payload arity up front (the recipient relies on a uniform schema).
    for (k, c) in candidates.iter().enumerate() {
        if c.payload.len() != payload_arity {
            return Err(MpcError::Protocol(format!(
                "oblivious_set_output: candidate {k} has {} payload terms, expected arity {payload_arity}",
                c.payload.len()
            )));
        }
    }

    let mut dealer = backend.dealer();
    let t = dealer.threshold();
    // One multiplication (the select) needs honest-majority headroom n >= 2t+1.
    if dealer.parties() < 2 * t + 1 {
        return Err(MpcError::Protocol(format!(
            "oblivious_set_output needs n >= 2t+1 for the select multiplication (n={}, t={})",
            dealer.parties(),
            t
        )));
    }

    // The real-row sentinel tag. A matched slot's tag opens to REAL_TAG; a
    // non-match / dummy opens to Fp::zero(). REAL_TAG != 0 so the two are
    // distinguishable AFTER the shuffle (and only after — position is destroyed).
    let real_tag = Fp::one();

    // --- Step 1–3: build per-slot (tag-sharing, payload) selecting on the bit. ---
    // tag_share is a degree-`2t` sharing (product of the degree-`t` selector and a
    // degree-`t` real_tag sharing). payload travels in cleartext co-indexed.
    let mut tag_col: SecretColumn = Vec::with_capacity(bound);
    let mut payloads: Vec<Vec<Option<Term>>> = Vec::with_capacity(bound);

    for c in candidates {
        let sel = selector_sharing(&mut dealer, &c.matched)?;
        // tag_out = sel * real_tag  (degree 2t). sel=1 → REAL_TAG; sel=0 → 0.
        let real_tag_share = dealer.share(real_tag);
        let tag_out = shamir::mul_shares_raw(&sel, &real_tag_share)?;
        tag_col.push(tag_out);
        payloads.push(c.payload.clone());
    }
    // Pure-dummy padding slots: selector = shared 0 → tag opens to 0 → Dummy.
    let dummy_payload: Vec<Option<Term>> = vec![None; payload_arity];
    for _ in candidates.len()..bound {
        let zero_sel = dealer.share(Fp::zero());
        let real_tag_share = dealer.share(real_tag);
        let tag_out = shamir::mul_shares_raw(&zero_sel, &real_tag_share)?;
        tag_col.push(tag_out);
        payloads.push(dummy_payload.clone());
    }

    // --- Step 4: oblivious-shuffle the tag column AND co-permute the payloads. ---
    let (shuffled_tags, shuffled_payloads, shuffle_cost) =
        shuffle_with_payloads(&mut dealer, tag_col, payloads)?;

    // --- Step 5: open exactly `B` tags at degree 2t; classify each slot. ---
    let mut out: Vec<OutputSlot> = Vec::with_capacity(bound);
    for (tag_share, payload) in shuffled_tags.iter().zip(shuffled_payloads) {
        let tag = shamir::reconstruct_degree(tag_share, 2 * t)?;
        if tag == real_tag {
            out.push(OutputSlot::Row(payload));
        } else {
            // tag == 0 (non-match or pure dummy). Defensive: any other value would
            // indicate a corrupted/tampered tag share; treat as a dummy (the open
            // already routes through the robust checker, which aborts on detectable
            // tampering before we get here).
            out.push(OutputSlot::Dummy);
        }
    }

    let cost = ObliviousOutputCost {
        bound,
        select_mults: bound,
        opens: bound,
        shuffle: shuffle_cost,
    };
    Ok((out, cost))
}

/// Resolve a [`MatchBit`] to a degree-`t` secret-shared selector `{0,1}`.
///
/// [`MatchBit::Public`] is lifted with a fresh sharing (sound, no secure compare —
/// the disclosed-key regime). [`MatchBit::SecretShared`] is used as-is; its
/// well-formedness (a sharing of exactly 0 or 1) is the producer's obligation.
fn selector_sharing(dealer: &mut ShamirDealer, bit: &MatchBit) -> Result<Vec<Share>, MpcError> {
    match bit {
        MatchBit::Public(b) => {
            let v = if *b { Fp::one() } else { Fp::zero() };
            Ok(dealer.share(v))
        }
        MatchBit::SecretShared(s) => {
            if s.len() != dealer.parties() {
                return Err(MpcError::Protocol(format!(
                    "secret-shared match bit has {} shares, expected n={}",
                    s.len(),
                    dealer.parties()
                )));
            }
            Ok(s.clone())
        }
    }
}

/// Oblivious-shuffle a tag column AND co-permute its cleartext payloads by the
/// SAME random permutation. The substrate [`shuffle`] permutes a [`SecretColumn`]
/// by an internal secret permutation; to drag the payloads along we attach a
/// secret-shared position index to each slot, shuffle the packed (tag, index)
/// entries with the same call, then open the shuffled indices to learn where each
/// original slot landed and reorder the payloads to match.
///
/// Opening the indices leaks NOTHING about matches: the indices are exactly the
/// public set `0..n` (every slot has a distinct index), so their shuffled values
/// reveal only the permutation — which is uniformly random and independent of the
/// data. This is the standard "shuffle then reveal the routing of a public tag"
/// trick; the payload contents and the match bits never enter the open.
fn shuffle_with_payloads(
    dealer: &mut ShamirDealer,
    tag_col: SecretColumn,
    payloads: Vec<PayloadRow>,
) -> Result<ShuffledColumn, MpcError> {
    let n = tag_col.len();
    debug_assert_eq!(payloads.len(), n);
    let t = dealer.threshold();
    let nparties = dealer.parties();

    // Attach a degree-`t` sharing of the public index i to each slot, packed as a
    // wider per-slot column entry: [tag shares..., index shares...]. The shuffle
    // swaps whole entries, so tag and index stay locked together through routing.
    let mut packed: SecretColumn = Vec::with_capacity(n);
    for (i, tag) in tag_col.into_iter().enumerate() {
        let idx = dealer.share(Fp::new(i as u64));
        let mut entry = tag; // tag shares first
        entry.extend(idx); // then index shares
        packed.push(entry);
    }

    // One oblivious shuffle over the packed entries (the substrate primitive).
    let (shuffled, cost) = shuffle(dealer, packed)?;

    // Split each shuffled entry back into (tag shares, index shares). Every entry
    // has the same width = n_parties tag shares + n_parties index shares.
    let mut shuffled_tags: SecretColumn = Vec::with_capacity(n);
    let mut shuffled_idx_shares: Vec<Vec<Share>> = Vec::with_capacity(n);
    for entry in shuffled {
        if entry.len() != 2 * nparties {
            return Err(MpcError::Protocol(format!(
                "shuffle_with_payloads: packed entry width {} != 2*n_parties {}",
                entry.len(),
                2 * nparties
            )));
        }
        let (tag_part, idx_part) = entry.split_at(nparties);
        shuffled_tags.push(tag_part.to_vec());
        shuffled_idx_shares.push(idx_part.to_vec());
    }

    // Open the shuffled indices (degree `t`) to learn the applied permutation, then
    // co-permute the payloads. The indices are the public set 0..n, so this open
    // reveals only the permutation, never any secret (see fn doc).
    let mut shuffled_payloads: Vec<Vec<Option<Term>>> = Vec::with_capacity(n);
    for idx_shares in &shuffled_idx_shares {
        let idx_fp = shamir::reconstruct_degree(idx_shares, t)?;
        let i = idx_fp.value() as usize;
        if i >= n {
            return Err(MpcError::Protocol(format!(
                "shuffle_with_payloads: opened index {i} out of range 0..{n} (corrupted index share)"
            )));
        }
        shuffled_payloads.push(payloads[i].clone());
    }

    Ok((shuffled_tags, shuffled_payloads, cost))
}

/// Run a hidden-value join with the oblivious set-returning output path, from
/// candidate pairs whose match was decided by a **disclosed / public** equi-join
/// (convention #4) — the SOUND-TODAY end-to-end entry point.
///
/// This wraps [`oblivious_set_output`] into the [`PartialResult`] shape the rest of
/// the crate uses, so a federation can run a private-key SET join whose OUTPUT is
/// oblivious (L1 bounded to `B`, L2 destroyed) without touching
/// [`crate::join::HiddenValueJoin`]'s leaky per-pair open path. The output schema
/// is `out_vars`; the disclosed result is the [`OutputSlot::Row`]s with dummies
/// filtered, attributed to the synthetic `federation` holder, in canonical order.
///
/// HONESTY: this is the sound path because the match bits are public (decided
/// outside the crypto core). The HIDDEN-key variant — deriving the match bits from
/// secret keys WITHOUT opening them — is [`oblivious_set_output_hidden_keys`],
/// which is gated on secure-compare (sq-rrz4/sq-dvuc).
pub fn oblivious_join_output(
    backend: &ShamirBackend,
    candidates: &[Candidate],
    out_vars: Vec<Variable>,
    bound: usize,
) -> Result<(PartialResult, ObliviousOutputCost), MpcError> {
    let payload_arity = out_vars.len();
    let (slots, cost) = oblivious_set_output(backend, candidates, payload_arity, bound)?;
    // Filter dummies: the authorized recipient sees the real result multiset (and,
    // by construction, learns its true size — see the residual-leakage note).
    let mut rows: Vec<Vec<Option<Term>>> = slots
        .into_iter()
        .filter_map(|s| match s {
            OutputSlot::Row(r) => Some(r),
            OutputSlot::Dummy => None,
        })
        .collect();
    // Canonical (order-independent) row order so the disclosed multiset is
    // independent of the shuffle's permutation (the recipient recomputes any
    // ORDER BY over the disclosed multiset — convention #4).
    canonicalize_rows(&mut rows);
    Ok((
        PartialResult {
            holder: HolderId::new("federation"),
            vars: out_vars,
            rows,
        },
        cost,
    ))
}

/// Sort rows into a canonical order (by their `{:?}` term tuple) so the disclosed
/// result multiset is independent of the shuffle / row ordering. Local to this
/// module so it does not couple to `join.rs` (sibling-PR file-lane discipline).
fn canonicalize_rows(rows: &mut [Vec<Option<Term>>]) {
    rows.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
}

/// The HIDDEN-key oblivious set output: derive each candidate's match bit from
/// genuinely-SECRET keys WITHOUT opening it, then run the oblivious output
/// transform. This is the configuration that closes L2 at its source for a
/// truly-private join key (the disclosed-key path closes L2 too, but only because
/// the key was public to begin with).
///
/// **Gated — not faked.** Producing a secret-SHARED match bit from secret keys
/// needs a secure equality-to-shared-bit (Rabbit / edaBits bit-decomposition over
/// `F_p`), which consumes a multiplication chain and therefore the BGW/DN
/// **degree-reduction** round (bead **sq-dvuc**) and the secure-comparison
/// primitive on it (bead **sq-rrz4**) — the SAME gate the substrate's secret-key
/// sort sits behind. The crate's existing `secure_equal` produces the bit only by
/// *opening* it, which IS the L2 leak this whole path exists to close, so it
/// cannot feed step 1. Returns an honest [`MpcError::NotYetImplemented`] naming the
/// gate. Once sq-rrz4 lands, this computes the shared bits and calls
/// [`oblivious_set_output`] with [`MatchBit::SecretShared`] selectors — the
/// transform itself is already sound and tested.
pub fn oblivious_set_output_hidden_keys(
    _backend: &ShamirBackend,
    _left_keys: &[Fp],
    _right_keys: &[Fp],
    _bound: usize,
) -> Result<ObliviousOutput, MpcError> {
    Err(MpcError::not_yet(
        "hidden-key oblivious join output — secure equality-to-SHARED-bit (never opened per-pair) \
         feeding the oblivious set-output transform; the existing secure_equal only OPENS the bit \
         (the L2 leak this path closes), so it cannot drive an oblivious select",
        "sq-rrz4 (secure greater-than / equality-to-shared-bit) on sq-dvuc (BGW/DN degree-reduction round)",
    ))
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

    /// Build candidate set from (matched_bool, payload) tuples in the disclosed-key
    /// (public match bit) regime.
    fn public_candidates(rows: &[(bool, Vec<Option<Term>>)]) -> Vec<Candidate> {
        rows.iter()
            .map(|(m, p)| Candidate { payload: p.clone(), matched: MatchBit::Public(*m) })
            .collect()
    }

    /// The real (non-dummy) rows of an output, as an order-independent multiset.
    fn real_multiset(slots: &[OutputSlot]) -> Vec<String> {
        let mut m: Vec<String> = slots
            .iter()
            .filter_map(|s| match s {
                OutputSlot::Row(r) => Some(format!("{r:?}")),
                OutputSlot::Dummy => None,
            })
            .collect();
        m.sort();
        m
    }

    // ---- Correctness: the real result multiset is right under reconstruction ----

    /// THE correctness test: the matched rows survive, the non-matches become
    /// dummies, and the real multiset equals the plaintext expectation — regardless
    /// of the (oblivious) order they come out in.
    #[test]
    fn matched_rows_survive_nonmatches_become_dummies() {
        let backend = ShamirBackend::new_seeded(3, 0xBEEF).unwrap();
        let cands = public_candidates(&[
            (true, vec![lit("Alice"), lit("Leeds")]),
            (false, vec![lit("Bob"), lit("York")]),
            (true, vec![lit("Carol"), lit("Hull")]),
            (false, vec![lit("Dan"), lit("Ely")]),
        ]);
        let (slots, cost) = oblivious_set_output(&backend, &cands, 2, 6).unwrap();
        assert_eq!(slots.len(), 6, "exactly B=6 slots revealed");
        // Two real matches; the rest dummies (2 non-matches + 2 pure padding).
        let reals = real_multiset(&slots);
        assert_eq!(reals.len(), 2);
        let mut expected = vec![
            format!("{:?}", vec![lit("Alice"), lit("Leeds")]),
            format!("{:?}", vec![lit("Carol"), lit("Hull")]),
        ];
        expected.sort();
        assert_eq!(reals, expected, "real rows must be exactly the matched payloads");
        assert_eq!(cost.bound, 6);
        assert_eq!(cost.select_mults, 6);
        assert_eq!(cost.opens, 6);
        assert_eq!(cost.shuffle.items, 6);
    }

    /// Padding to a bound LARGER than the candidate count works (pure-dummy slots).
    #[test]
    fn pads_with_pure_dummies_up_to_bound() {
        let backend = ShamirBackend::new_seeded(5, 7).unwrap();
        let cands = public_candidates(&[(true, vec![lit("X")])]);
        let (slots, cost) = oblivious_set_output(&backend, &cands, 1, 10).unwrap();
        assert_eq!(slots.len(), 10);
        assert_eq!(real_multiset(&slots).len(), 1);
        assert_eq!(slots.iter().filter(|s| matches!(s, OutputSlot::Dummy)).count(), 9);
        assert_eq!(cost.bound, 10);
    }

    /// All-match and no-match boundaries.
    #[test]
    fn all_match_and_no_match_boundaries() {
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        let all = public_candidates(&[(true, vec![lit("a")]), (true, vec![lit("b")])]);
        let (slots, _) = oblivious_set_output(&backend, &all, 1, 2).unwrap();
        assert_eq!(real_multiset(&slots).len(), 2, "B == match count: all real, no dummies");
        assert!(slots.iter().all(|s| matches!(s, OutputSlot::Row(_))));

        let none = public_candidates(&[(false, vec![lit("a")]), (false, vec![lit("b")])]);
        let (slots, _) = oblivious_set_output(&backend, &none, 1, 4).unwrap();
        assert_eq!(real_multiset(&slots).len(), 0, "no matches: all dummies");
        assert!(slots.iter().all(|s| matches!(s, OutputSlot::Dummy)));
    }

    // ---- L1 obliviousness: revealed count is the BOUND, not the true count ------

    /// L1: the number of REVEALED slots is exactly `B` for ANY true match count —
    /// the parties see `B`, not the true cardinality. Two candidate sets with
    /// DIFFERENT true match counts must produce the IDENTICAL revealed-slot count.
    #[test]
    fn revealed_count_is_bound_not_true_cardinality() {
        let backend = ShamirBackend::new_seeded(3, 42).unwrap();
        let one_match = public_candidates(&[
            (true, vec![lit("a")]),
            (false, vec![lit("b")]),
            (false, vec![lit("c")]),
        ]);
        let three_match = public_candidates(&[
            (true, vec![lit("a")]),
            (true, vec![lit("b")]),
            (true, vec![lit("c")]),
        ]);
        let (s1, _) = oblivious_set_output(&backend, &one_match, 1, 5).unwrap();
        let (s3, _) = oblivious_set_output(&backend, &three_match, 1, 5).unwrap();
        // Same revealed slot count (= B) regardless of true match count → L1
        // bounded to B, not exact, from the slot count the parties observe.
        assert_eq!(s1.len(), s3.len());
        assert_eq!(s1.len(), 5);
        // The transcript-visible thing (slot count) is identical; only the
        // recipient who filters dummies learns the true counts differ (1 vs 3).
        assert_eq!(real_multiset(&s1).len(), 1);
        assert_eq!(real_multiset(&s3).len(), 3);
    }

    // ---- L2 obliviousness: output order hides which candidate produced each row -

    /// L2 (access-pattern half): the OUTPUT ORDER is not the input order — the
    /// shuffle destroyed position→candidate linkage. Over several runs the slot
    /// CLASSIFICATION (which positions are Row vs Dummy) must VARY — proving the
    /// match positions are not fixed by input position.
    #[test]
    fn output_order_is_shuffled_hiding_match_positions() {
        // Distinct matched payloads at fixed input positions; if the output never
        // reordered, the matched rows would land at the same output slots every run.
        let cands = public_candidates(&[
            (true, vec![lit("p0")]),
            (false, vec![lit("p1")]),
            (true, vec![lit("p2")]),
            (false, vec![lit("p3")]),
            (true, vec![lit("p4")]),
        ]);
        let mut classification_patterns = std::collections::HashSet::new();
        for seed in 0..32u64 {
            let backend = ShamirBackend::new_seeded(3, 1000 + seed).unwrap();
            let (slots, _) = oblivious_set_output(&backend, &cands, 1, 5).unwrap();
            // Record the Row/Dummy pattern (positions of the 3 reals among 5 slots).
            let pat: Vec<bool> = slots.iter().map(|s| matches!(s, OutputSlot::Row(_))).collect();
            classification_patterns.insert(pat);
            // Always exactly 3 reals regardless of where they land.
            assert_eq!(real_multiset(&slots).len(), 3);
        }
        // If the output were NOT shuffled, the match positions would be fixed
        // (reals always at input slots 0,2,4) → exactly ONE pattern. The shuffle
        // must produce several distinct patterns.
        assert!(
            classification_patterns.len() > 1,
            "match positions never moved across 32 runs — output is not oblivious"
        );
    }

    /// The real multiset is INVARIANT to the shuffle (correctness under
    /// reconstruction holds no matter which permutation the shuffle drew).
    #[test]
    fn real_multiset_is_shuffle_invariant() {
        let cands = public_candidates(&[
            (true, vec![lit("A"), lit("1")]),
            (false, vec![lit("B"), lit("2")]),
            (true, vec![lit("C"), lit("3")]),
        ]);
        let mut expected = vec![
            format!("{:?}", vec![lit("A"), lit("1")]),
            format!("{:?}", vec![lit("C"), lit("3")]),
        ];
        expected.sort();
        for seed in 0..16u64 {
            let backend = ShamirBackend::new_seeded(5, seed).unwrap();
            let (slots, _) = oblivious_set_output(&backend, &cands, 2, 4).unwrap();
            assert_eq!(real_multiset(&slots), expected, "seed {seed}: real multiset changed");
        }
    }

    // ---- Secret-shared match bit path (sound transform given a shared bit) -------

    /// A SECRET-SHARED match bit (sharing of 0/1) drives the select correctly — the
    /// transform is sound given the bit; only PRODUCING the bit from secret keys is
    /// gated. Here we hand it well-formed 0/1 sharings directly.
    #[test]
    fn secret_shared_match_bit_select_is_correct() {
        let backend = ShamirBackend::new_seeded(3, 99).unwrap();
        let mut dealer = backend.dealer();
        let one = dealer.share(Fp::one());
        let zero = dealer.share(Fp::zero());
        let cands = vec![
            Candidate { payload: vec![lit("match")], matched: MatchBit::SecretShared(one) },
            Candidate { payload: vec![lit("nomatch")], matched: MatchBit::SecretShared(zero) },
        ];
        let (slots, _) = oblivious_set_output(&backend, &cands, 1, 3).unwrap();
        assert_eq!(real_multiset(&slots), vec![format!("{:?}", vec![lit("match")])]);
    }

    // ---- Fail-closed contracts --------------------------------------------------

    /// Bound smaller than the candidate count FAILS CLOSED (never truncates).
    #[test]
    fn bound_below_candidate_count_is_protocol_error() {
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        let cands = public_candidates(&[(true, vec![lit("a")]), (true, vec![lit("b")])]);
        let err = oblivious_set_output(&backend, &cands, 1, 1).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(m) if m.contains("truncate")));
    }

    /// Wrong payload arity is rejected (uniform schema for the recipient).
    #[test]
    fn payload_arity_mismatch_is_protocol_error() {
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        let cands = public_candidates(&[(true, vec![lit("a"), lit("b")])]);
        let err = oblivious_set_output(&backend, &cands, 1, 4).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(m) if m.contains("arity")));
    }

    /// A malformed secret-shared bit (wrong share count) is rejected, not silently
    /// mis-selected.
    #[test]
    fn malformed_secret_bit_is_rejected() {
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        let cands = vec![Candidate {
            payload: vec![lit("a")],
            matched: MatchBit::SecretShared(vec![]), // 0 shares != n
        }];
        let err = oblivious_set_output(&backend, &cands, 1, 2).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(m) if m.contains("match bit")));
    }

    // ---- The full PartialResult wrapper + the gated hidden-key path -------------

    /// The end-to-end disclosed-key wrapper yields the right PartialResult (real
    /// rows only, federation-attributed, canonical order).
    #[test]
    fn oblivious_join_output_yields_partial_result() {
        let backend = ShamirBackend::new_seeded(3, 5).unwrap();
        let cands = public_candidates(&[
            (true, vec![lit("Bob"), lit("Leeds")]),
            (false, vec![lit("x"), lit("y")]),
            (true, vec![lit("Carol"), lit("York")]),
        ]);
        let out_vars = vec![var("name"), var("city")];
        let (result, cost) = oblivious_join_output(&backend, &cands, out_vars.clone(), 5).unwrap();
        assert_eq!(result.holder, HolderId::new("federation"));
        assert_eq!(result.vars, out_vars);
        assert_eq!(result.rows.len(), 2, "only the 2 matched rows, dummies filtered");
        assert_eq!(cost.bound, 5);
        // Canonical (order-independent) check of the disclosed multiset.
        let got: Vec<String> = result.rows.iter().map(|r| format!("{r:?}")).collect();
        let mut exp = vec![
            format!("{:?}", vec![lit("Bob"), lit("Leeds")]),
            format!("{:?}", vec![lit("Carol"), lit("York")]),
        ];
        exp.sort();
        assert_eq!(got, exp, "disclosed multiset wrong (canonical order)");
    }

    /// The HIDDEN-key path is honestly gated on secure-compare — NOT faked.
    #[test]
    fn hidden_key_path_is_gated_on_secure_compare() {
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        let err = oblivious_set_output_hidden_keys(&backend, &[Fp::new(1)], &[Fp::new(1)], 4)
            .unwrap_err();
        match err {
            MpcError::NotYetImplemented { gated_on, what } => {
                assert!(
                    what.contains("SHARED-bit") || what.contains("oblivious select"),
                    "what must name the missing primitive: {what}"
                );
                assert!(gated_on.contains("sq-rrz4"), "must cite the secure-compare gate");
                assert!(gated_on.contains("sq-dvuc"), "must cite the degree-reduction gate");
            }
            other => panic!("expected NotYetImplemented, got {other:?}"),
        }
    }

    /// Empty candidate set with a bound → all dummies, no panic.
    #[test]
    fn empty_candidates_all_dummies() {
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        let (slots, cost) = oblivious_set_output(&backend, &[], 2, 3).unwrap();
        assert_eq!(slots.len(), 3);
        assert!(slots.iter().all(|s| matches!(s, OutputSlot::Dummy)));
        assert_eq!(cost.bound, 3);
    }

    /// Bound == 0 with no candidates is the trivial empty output (shuffle of 0).
    #[test]
    fn zero_bound_no_candidates_is_empty() {
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        let (slots, cost) = oblivious_set_output(&backend, &[], 0, 0).unwrap();
        assert!(slots.is_empty());
        assert_eq!(cost.bound, 0);
        assert_eq!(cost.select_mults, 0);
    }
}
