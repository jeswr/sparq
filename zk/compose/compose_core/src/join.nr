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
    let mut leaves_a: [Field; N_A] = [0; N_A];
    for i in 0..N_A {
        leaves_a[i] = h3(enc_a[i][0], enc_a[i][1], enc_a[i][2]);
    }
    assert_eq(commit_fold(leaves_a, counts_a), commit_a, "graph-A commitment mismatch");

    let mut leaves_b: [Field; N_B] = [0; N_B];
    for i in 0..N_B {
        leaves_b[i] = h3(enc_b[i][0], enc_b[i][1], enc_b[i][2]);
    }
    assert_eq(commit_fold(leaves_b, counts_b), commit_b, "graph-B commitment mismatch");

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
