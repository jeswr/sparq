// [OPUS-4.8] sq-bwwl / sq-8mwz: in-circuit hidden cross-credential JOIN.
//! Hidden cross-credential JOIN relation (`research/zk-hidden-join-design.md`,
//! bead `sq-bwwl`; epic `sq-1s2`).
//!
//! # The leak this closes
//! A multi-pattern BGP that shares a variable across two credentials --
//! `{ ?p :worksFor :ACME } { ?p :hasSalary ?s }` answered by two different
//! committed graphs -- is proved today by two separate `scan_k...` sub-proofs
//! whose PUBLIC `rows` disclose the shared variable's term encoding `Enc(?p)` in
//! BOTH proofs. The privacy-sensitive joined entity (`?p`, a person/account/DID)
//! is therefore revealed whenever two credentials are joined -- the largest
//! remaining single-prover privacy leak. There is no in-circuit join relation in
//! `compose_core` (only scan/filter/revoke/issuer). This member adds one.
//!
//! # Trust model -- SINGLE-PROVER (distinct from the MPC PSI join `sq-t21`)
//! ONE prover legitimately holds BOTH credentials and sees both plaintexts;
//! there is nothing to hide FROM the prover, only from the VERIFIER. The
//! verifier must learn THAT a join holds but NOT the joined term. This is the
//! single-prover analogue of `sq-t21` (two mutually-distrusting holders, one
//! each -> requires MPC, not single-prover ZK). They are complementary, not
//! competing: use `join_eq` when one party holds both credentials; use the MPC
//! path when the two sides are distinct, mutually-private parties.
//!
//! # The relation (design S2.2)
//! Given the two graph commitments (PUBLIC, == the two scan proofs'
//! `commitments[g]`), the two join SLOTS (PUBLIC, == the query-derived shared
//! variable's positions), and a HIDING commitment to the join value (PUBLIC):
//!
//!   1. Re-commit both witnessed graphs to their PUBLIC commitments
//!      (`commit_fold` over `h3` leaves -- identical to `scan.nr` step 1). This
//!      binds the witnessed contents to the SAME `C(G)` the scan proofs expose
//!      and the issuer signed (anti-row-swap, design S2.3 / A2).
//!   2. `row_a` / `row_b` are genuinely present rows of their committed graphs
//!      (the `scan.nr:139-149` present-in-graph discipline).
//!   3. Select the join slot at the PUBLIC, query-bound positions `slot_a` /
//!      `slot_b` (in {0,1,2}); the verifier equates these to the query-derived
//!      positions (design S4.4 slot binding) so the prover cannot pick a
//!      convenient column.
//!   4. THE JOIN -- `assert(a_val == b_val)`, a single Field equality over the
//!      two HIDDEN values (design S2.1 / S4.1 / A1). The joined term encoding is
//!      a PRIVATE witness; it NEVER appears in a public input.
//!   5. Bind the (single) join value under a per-presentation blinder into the
//!      PUBLIC `join_commitment` (the only public join artefact; design S2.4 /
//!      S4.3 -- defeats the dictionary attack R4 a bare encoding would invite).
//!
//! # Public vs private (the privacy boundary -- design S4.4)
//! - PUBLIC:  `challenge` (verifier nonce, replay-binding, unconstrained in the
//!            relation -- the verifier byte-binds it), `commit_a`, `commit_b`
//!            (the two graph commitments), `join_commitment` (HIDING commitment
//!            to the join value), `slot_a`, `slot_b` (the query-revealed join
//!            columns in {0,1,2}).
//! - PRIVATE: `enc_a`/`counts_a`, `enc_b`/`counts_b` (the two graph contents +
//!            sizes), `row_a`/`row_b` (the joined rows), `blinding` (the
//!            per-presentation blinder).
//!
//! The join VALUE is HIDDEN; the join SLOTS are PUBLIC (the query already
//! reveals which column the shared variable occupies -- not new leakage). This is
//! the precise, honest privacy boundary: `a_val`/`b_val` and the rows that carry
//! them are private witnesses, so the joined entity is absent from the public
//! interface by construction (the test `join_eq_value_absent_from_public_inputs`
//! pins this against the host commitment).
//!
//! # Cost note (noir-optimisation S5)
//! Dominated by the TWO `commit_fold` re-commitments (step 1) -- roughly the
//! per-graph hashing of two `scan` members at the same `N`. The equality (step
//! 4) is ~free; the hiding commitment (step 5) is one `h3`. No scalar-muls, no
//! foreign-field, no pairings -- far cheaper than `hidden_issuer_d{D}`. The
//! actual `bb gates` figure is regression-gated under bead `sq-mj2z` (NOT
//! asserted here -- repo policy forbids unmeasured numbers).
//!
//! # The CARDINALITY-HIDING tier (`join_eq_r_check`, bead `sq-uii0`)
//! [`join_eq_check`] proves ONE pair per sub-proof, so presenting `N` matched
//! pairs is `N` visible sub-proofs and discloses `N` (design S1.4 R2).
//! [`join_eq_r_check`] proves `R` lanes over ONE pair of graph re-commitments --
//! amortising the dominant cost AND decoupling the proof SHAPE from `N`, by
//! padding unused lanes with a REPEATED genuine pair under a fresh blinder. It
//! proves `R` NOT-NECESSARILY-DISTINCT genuine pairs, which -- as a statement
//! ABOUT CARDINALITY -- says no more than [`join_eq_check`]'s "at least one
//! match exists": `R` is a public SHAPE parameter, NOT a proven bound on `N`.
//! See its own docs for what `N` denotes and the full "what this does not
//! prove" list.

use crate::hashes::{commit_fold, h3};

// [OPUS-4.8] sq-bwwl: domain tag for the hiding join-value commitment, IDENTICAL
// to the Rust `sparq_zk::sig::SIG_DOMAIN_JOIN` tag ("ZKSIG_JN" =
// 0x5a4b_5349_475f_4a4e). Both compute `Poseidon2::hash([DOMAIN, value, blinding],
// 3)` so the in-circuit commitment byte-equals the host one (single source of
// truth). A DISTINCT tag from every other `SIG_DOMAIN_*`/status commitment so a
// join commitment can never be cross-substituted for another commitment.
pub global JOIN_DOMAIN: Field = 0x5a4b5349475f4a4e;

/// The hiding commitment to a join value (design S2.4): `h3(JOIN_DOMAIN, value,
/// blinding)`. Mirrors `sparq_zk::sig::join_value_commitment` exactly.
pub fn join_value_commitment(value: Field, blinding: Field) -> Field {
    h3(JOIN_DOMAIN, value, blinding)
}

/// Select slot `slot` (in {0,1,2}) of a row WITHOUT a secret-dependent index:
/// `slot` is a PUBLIC input, so the selection is constrained to a public
/// position (design S4.4 -- the slot is not secret, only the value is). The
/// branch is over a public value and `slot` is range-checked to {0,1,2}: an
/// out-of-range slot is rejected (a join column must be subject/predicate/object).
fn select_slot(row: [Field; 3], slot: u32) -> Field {
    assert(slot < 3, "join slot out of range (must be subject/predicate/object)");
    let mut out = row[0];
    if slot == 1 {
        out = row[1];
    }
    if slot == 2 {
        out = row[2];
    }
    out
}

/// Re-commit a witnessed graph `(enc, count)` -- step 1 of the relation,
/// identical to `scan.nr` step 1 (leaf = `h3(s, p, o)`, `commit_fold` over the
/// first `count` leaves). The caller asserts the result equals the PUBLIC
/// `commit`, which is THE load-bearing binding: the prover must reproduce the
/// exact `C(G)` the scan proof published (and the issuer signed), so it cannot
/// feed the join a graph other than the attested one (anti-row-swap, design
/// S2.3 / A2).
///
/// Shared verbatim by [`join_eq_check`] (one pair) and [`join_eq_r_check`] (the
/// R-lane cardinality-hiding tier) -- Noir monomorphises + inlines, so factoring
/// it out is ACIR-identical (pinned by the UNCHANGED `join_eq_na*` gate-count
/// baselines).
fn recommit_graph<let N: u32>(enc: [[Field; 3]; N], count: u32) -> Field {
    let mut leaves: [Field; N] = [0; N];
    for i in 0..N {
        leaves[i] = h3(enc[i][0], enc[i][1], enc[i][2]);
    }
    commit_fold(leaves, count)
}

/// `row` occurs in some active slot of the graph `(enc, count)` -- the
/// `scan.nr:139-149` present-in-graph discipline. A row not in the graph has no
/// witness (every active slot is checked; padding slots `i >= count` are
/// excluded by `slot_active`).
fn row_present_in<let N: u32>(enc: [[Field; 3]; N], count: u32, row: [Field; 3]) -> bool {
    let mut present = false;
    for i in 0..N {
        let slot_active = i < count;
        let eq = (enc[i][0] == row[0]) & (enc[i][1] == row[1]) & (enc[i][2] == row[2]);
        present |= slot_active & eq;
    }
    present
}

/// The generic hidden-join relation; each `join_eq_na{N_A}_nb{N_B}` bin package
/// is one monomorphisation. See module docs for the statement and the
/// public/private split.
///
/// `commit_a`/`commit_b` (PUBLIC) MUST byte-equal the two scan proofs'
/// `commitments[g]` (the verifier's `bind_joins` gate, design S3.3 -- NOT
/// enforced here; this is the in-circuit relation). `slot_a`/`slot_b` (PUBLIC)
/// MUST equal the query-derived shared-variable positions (the verifier's
/// `bind_query_correctness`, design S4.4). `join_commitment` (PUBLIC) is the
/// only public artefact carrying the join value, and it is HIDING.
pub fn join_eq_check<let N_A: u32, let N_B: u32>(
    commit_a: Field,
    commit_b: Field,
    join_commitment: Field,
    slot_a: u32,
    slot_b: u32,
    enc_a: [[Field; 3]; N_A],
    counts_a: u32,
    enc_b: [[Field; 3]; N_B],
    counts_b: u32,
    row_a: [Field; 3],
    row_b: [Field; 3],
    blinding: Field,
) {
    assert(counts_a <= N_A, "graph-A size exceeds circuit bucket");
    assert(counts_b <= N_B, "graph-B size exceeds circuit bucket");

    // 1. Re-commit both graphs to their PUBLIC commitments (binds the witnessed
    //    contents to the SAME C(G) the scan proofs expose -- anti-row-swap, A2).
    //    Identical to scan.nr step 1: leaf = h3(s,p,o), fold over first `count`.
    assert_eq(recommit_graph(enc_a, counts_a), commit_a, "graph-A commitment mismatch");
    assert_eq(recommit_graph(enc_b, counts_b), commit_b, "graph-B commitment mismatch");

    // 2-5. The per-pair relation (presence, slot selection, equality, hiding
    //      commitment) -- shared verbatim with the R-lane tier below.
    join_pair_check(
        join_commitment,
        slot_a,
        slot_b,
        enc_a,
        counts_a,
        enc_b,
        counts_b,
        row_a,
        row_b,
        blinding,
    );
}

/// Steps 2-5 of the relation for ONE joined pair, over graphs whose contents
/// have ALREADY been re-committed to their public commitments by the caller
/// (step 1). Factored out so [`join_eq_check`] (one pair) and
/// [`join_eq_r_check`] (R lanes) prove the SAME per-pair statement by
/// construction -- there is no second copy of the equality to drift.
///
/// The assert messages are the v1 literals verbatim: the forge-and-verify suite
/// (`crates/sparq-zk-compose/tests/join_forge.rs`, `tests.nr` J1-J5) pins them.
fn join_pair_check<let N_A: u32, let N_B: u32>(
    join_commitment: Field,
    slot_a: u32,
    slot_b: u32,
    enc_a: [[Field; 3]; N_A],
    counts_a: u32,
    enc_b: [[Field; 3]; N_B],
    counts_b: u32,
    row_a: [Field; 3],
    row_b: [Field; 3],
    blinding: Field,
) {
    // 2. row_a / row_b are genuinely present rows of their committed graphs.
    //    Because step 1 re-commits the SAME contents, presence here is presence
    //    in the attested credential -- a row not in G_a has no witness.
    assert(row_present_in(enc_a, counts_a, row_a), "join row A not present in committed graph A");
    assert(row_present_in(enc_b, counts_b, row_b), "join row B not present in committed graph B");

    // 3. Select the join slot at the PUBLIC, query-bound positions. The slot is
    //    not secret (the query reveals it); making it public lets the verifier
    //    pin the equality to the right column (design S4.4) with a plain
    //    public-input equality.
    let a_val = select_slot(row_a, slot_a);
    let b_val = select_slot(row_b, slot_b);

    // 4. THE JOIN -- equality over the two HIDDEN values. By soundness of the
    //    proof system a valid proof exists only if a_val == b_val: a prover with
    //    unequal join slots simply has no satisfying witness (anti-forge, A1).
    //    a_val/b_val are derived from PRIVATE rows and are NOT public inputs.
    assert(a_val == b_val, "join slots are not equal");

    // 5. Bind the (single) join value under a per-presentation blinder into the
    //    PUBLIC hiding commitment -- the only public join artefact. The verifier
    //    never opens it (it only byte-equals it across a multi-way join chain);
    //    the blinder makes two presentations of the same value unlinkable, and
    //    the bound value is the SAME a_val the equality used (anti-equivocation).
    assert_eq(
        join_commitment,
        join_value_commitment(a_val, blinding),
        "join commitment does not bind the proven join value",
    );
}

/// The CARDINALITY-HIDING tier of the hidden cross-credential JOIN (bead
/// `sq-uii0`, the optional v2 of `sq-bwwl`; design record
/// `research/zk-hidden-join-design.md` S2.5 / S5 / S8, step 7).
///
/// # What it is
/// `join_eq_r_check` proves `R` joined pairs in ONE member instead of one
/// sub-proof per pair. The two graph re-commitments (step 1, the dominant cost)
/// are paid ONCE for all `R` lanes; each lane pays only its two present-in-graph
/// sweeps, one equality, and one hiding commitment. Each lane `r` carries its
/// OWN row pair, its OWN blinder, and its OWN public `join_commitments[r]` --
/// a bag-semantics join produces pairs with DIFFERENT join values (`?p = alice`
/// and `?p = bob`), so one commitment per lane is required, not one shared one.
///
/// # What `N` denotes, and the residual-leakage improvement (design S1.4 R2)
/// Throughout, `N` is the TRUE number of joined pairs of the two committed
/// graphs at the query-bound slots -- a property of the GRAPHS, never of the
/// proof, and never something this member measures. v1 leaks it: presenting `N`
/// pairs is `N` visible `join_eq` sub-proofs, so the SHAPE of the presentation
/// IS `N`. This tier fixes that shape at the compile-time `R` whatever `N` is:
/// a prover disclosing fewer than `R` pairs fills the remaining lanes by
/// REPEATING a genuine pair with a FRESH blinder per lane. Every lane is
/// therefore a TRUE join (nothing false is ever proved -- the padding is real
/// content, not a dummy), and because `join_value_commitment` is hiding and the
/// blinders are independent, a repeated lane is indistinguishable from a
/// distinct one.
///
/// The improvement is exactly and only this: `N` can no longer be READ OFF the
/// proof shape. The verifier does not thereby LEARN anything about `N` beyond
/// `N >= 1` -- in particular it does not learn `N <= R` (see below). What stays
/// visible is the prover's CHOICE of bucket `R`; under the natural policy "take
/// the smallest compiled `R` >= the number of pairs to disclose", that coarsens
/// the disclosure to the bucket lattice rather than removing it. That coarsening
/// is a property of prover POLICY and of the compiled bucket set -- it is NOT an
/// in-circuit constraint and a verifier cannot rely on it.
///
/// # What this member does NOT prove (no overclaim -- design S1.4 / S8)
/// - It does NOT prove the pairs are DISTINCT, and therefore does NOT prove the
///   join cardinality IS `R`. The proven statement is exactly: "there exist `R`
///   (not necessarily distinct) pairs, each a genuine joined pair of the two
///   attested graphs at the query-bound slots". Never describe a `join_eq_r{R}`
///   proof as attesting `R` results.
/// - Because every lane may be the SAME pair, that statement is satisfiable as
///   soon as ONE genuine pair exists. AS A STATEMENT ABOUT CARDINALITY it is
///   therefore EQUIVALENT to [`join_eq_check`]'s "at least one match exists": it
///   proves neither `N <= R` nor `N >= R` nor any other bound on `N`. `R` fixes
///   only the member's public shape -- the number of lane commitments, and hence
///   the package name and the VK.
/// - It does NOT prove COMPLETENESS (that no further pair exists) -- completeness
///   is the scan member's obligation, unchanged.
/// - `R` itself is PUBLIC: it is the compiled member's identity (the bucket names
///   the package and hence the VK). A prover with `N > R` genuine pairs just
///   drops pairs or takes a larger bucket -- nothing in the circuit prevents
///   either, which is the same reason `R` is not an upper bound on `N`. Hiding
///   cardinality here is SHAPE padding, not erasure and not a proven bound --
///   exactly the cost/leakage trade-off design S2.5 flags.
/// - R1 is unchanged and intrinsic: `R >= 1` lanes force `N >= 1`, so the
///   verifier still learns THAT a join exists.
///
/// # Soundness posture
/// Same as v1: every lane re-derives from the SAME two PUBLIC graph commitments
/// (anti-A2) and proves its equality in-circuit (anti-A1). The composition
/// verifier remains NOT-yet-sound pending external audit (`sq-qhy4`); this member
/// asserts no soundness or ZK-privacy property as achieved.
///
/// # Public vs private
/// - PUBLIC:  `commit_a`, `commit_b`, `join_commitments[R]`, `slot_a`, `slot_b`
///            (plus the member's `challenge`, bound by the caller's `main`).
/// - PRIVATE: `enc_a`/`counts_a`, `enc_b`/`counts_b`, `rows_a`/`rows_b`,
///            `blindings`. No join VALUE appears in any public input.
// [OPUS-5] sq-uii0: cardinality-hiding join_eq_rN tier (optional v2 of sq-bwwl).
pub fn join_eq_r_check<let N_A: u32, let N_B: u32, let R: u32>(
    commit_a: Field,
    commit_b: Field,
    join_commitments: [Field; R],
    slot_a: u32,
    slot_b: u32,
    enc_a: [[Field; 3]; N_A],
    counts_a: u32,
    enc_b: [[Field; 3]; N_B],
    counts_b: u32,
    rows_a: [[Field; 3]; R],
    rows_b: [[Field; 3]; R],
    blindings: [Field; R],
) {
    assert(counts_a <= N_A, "graph-A size exceeds circuit bucket");
    assert(counts_b <= N_B, "graph-B size exceeds circuit bucket");

    // 1. The AMORTISED half: re-commit each graph ONCE, for all R lanes. This is
    //    the whole point of the tier -- the fold is O(N) per graph regardless of
    //    R, so R pairs cost one fold pair instead of R of them.
    assert_eq(recommit_graph(enc_a, counts_a), commit_a, "graph-A commitment mismatch");
    assert_eq(recommit_graph(enc_b, counts_b), commit_b, "graph-B commitment mismatch");

    // 2-5, per lane. Deliberately NO distinctness constraint across lanes and NO
    // per-lane "active" flag: repeating a genuine pair under a fresh blinder is
    // exactly the padding that keeps the shape independent of `N`, and an
    // `active` flag would either be public (re-leaking `N`) or let a prover
    // disable lanes at will. The price of that choice is stated in the docs:
    // without distinctness the member proves no bound on `N`.
    for r in 0..R {
        join_pair_check(
            join_commitments[r],
            slot_a,
            slot_b,
            enc_a,
            counts_a,
            enc_b,
            counts_b,
            rows_a[r],
            rows_b[r],
            blindings[r],
        );
    }
}
