//! Unit + adversarial tests (noir-testing conventions: accept path plus one
//! `should_fail_with` per distinct rejection clause, message-matched so each
//! test pins WHICH clause caught the lie -- noir-circuit-patterns
//! "Adversarial-prover testing").
//!
//! Cross-vector chain: the constants below are the `nargo execute` vectors
//! from `crates/sparq-zk/tests/poseidon2_noir_cross.rs` (fixture
//! `tests/fixtures/noir_poseidon2/`). Asserting them here proves
//! `compose_core` hashes through the exact function the Rust commitment
//! pipeline ports -- the bit-compatibility requirement for commitment
//! recomputation in-circuit.

use crate::filter_int::{filter_int_check, OP_EQ, OP_GE, OP_GT, OP_LE, OP_LT, OP_NE};
use crate::filter_float::filter_f64_check;
use crate::hashes::{commit_fold, h2, h3, TYPE_CODE_LITERAL};
use crate::join::{join_eq_check, join_value_commitment, JOIN_DOMAIN};
use crate::revoke::{revoke_unset_check, revoke_unset_check_committed, SIG_DOMAIN_STATUS_IDX_COMMIT};
use crate::scan::{scan_check, PatternSpec};

// ---------------------------------------------------------------------------
// Cross-vectors (sparq-zk fixture outputs, nargo 1.0.0-beta.21).
// ---------------------------------------------------------------------------

#[test]
fn commit_fold_matches_sparq_zk_cross_vectors() {
    // Poseidon2::hash([1], 1) ... ([1,2,3,4], 4) -- the h2/h3 layers and small
    // folds.
    assert_eq(
        commit_fold([1], 1),
        0x168758332d5b3e2d13be8048c8011b454590e06c44bce7f702f09103eef5a373,
    );
    assert_eq(
        h2(1, 2),
        0x038682aa1cb5ae4e0a3f13da432a95c77c5c111f6f030faf9cad641ce1ed7383,
    );
    assert_eq(
        h3(1, 2, 3),
        0x23864adb160dddf590f1d3303683ebcb914f828e2635f6e85a32f0a1aecd3dd8,
    );
    assert_eq(
        commit_fold([1, 2, 3, 4], 4),
        0x130bf204a32cac1f0ace56c78b731aa3809f06df2731ebcf6b3464a15788b1b9,
    );
    assert_eq(
        commit_fold([5, 6, 7, 8, 9, 10, 11], 7),
        0x0ceb49bb97a406f119fcceb666e68b0acb48c286d9e011a4fa9427cd49989db5,
    );
    // The 40-leaf commitment-scale fold.
    let mut leaves: [Field; 40] = [0; 40];
    for i in 0..40 {
        leaves[i] = (i as Field) + 1;
    }
    assert_eq(
        commit_fold(leaves, 40),
        0x046964eee810ddedae9085f126317d43ad22b719fedfba0a78a1de86795dffee,
    );
}

#[test]
fn commit_fold_padding_slots_do_not_enter_the_commitment() {
    // The (k, n) bucket property: a graph of 4 triples committed inside an
    // N = 8 circuit bucket must produce the SAME commitment the Rust
    // pipeline computed over exactly 4 leaves -- and garbage in padding slots
    // must not matter.
    let exact = commit_fold([1, 2, 3, 4], 4);
    assert_eq(commit_fold([1, 2, 3, 4, 0, 0, 0, 0], 4), exact);
    assert_eq(commit_fold([1, 2, 3, 4, 99, 77, 55, 33], 4), exact);
    // ...and the count is domain-separating (length-bearing IV).
    assert(commit_fold([1, 2, 3, 4, 0, 0, 0, 0], 5) != exact);
}

// ---------------------------------------------------------------------------
// scan_check -- accept + adversarial. Tiny (K=2, N=4, R=2) monomorphisation.
// ---------------------------------------------------------------------------

// Test universe: term encodings are arbitrary nonzero field elements; what
// matters to the relation is equality structure, not provenance.
global S1: Field = 101;
global P: Field = 7;
global O1: Field = 201;
global S2: Field = 102;
global O2: Field = 202;
global P_OTHER: Field = 8;

fn test_graphs() -> ([[[Field; 3]; 4]; 2], [u32; 2], [Field; 2]) {
    // [OPUS-4.8] sq-7p9c: the fixture's per-graph witnesses are laid out in
    // CANONICAL ascending-commitment order, the same canonicalisation a real host
    // builder performs (scan.nr step 1b / plan S2.5): `scan_check` now requires the
    // PUBLIC `commitments` array STRICTLY INCREASING (`commitments[0] < ...`, the
    // distinct-graph guard added by #95 that closes the duplicate-inclusion / COUNT
    // forgery class). `commit_fold` is hash-like, so a graph's commitment value is
    // unrelated to its content "size" -- the single-triple graph { (S2,P,O2) }
    // happens to commit to the SMALLER field element than the two-triple graph
    // { (S1,P,O1), (S1,P_OTHER,O2) }, so the single-triple graph is graph 0 and the
    // two-triple graph is graph 1. Graphs are an unordered set keyed by commitment,
    // so reordering them (with the matching `counts` and disclosed `rows`) does NOT
    // change the proved statement -- only the now-canonical index assignment, which
    // every per-test attribution array below tracks. The trailing assert pins the
    // ordering invariant so this fixture fails LOUDLY (not the circuit's strict-order
    // assert silently) if an encoding edit ever flips the natural order again.
    //
    // Graph 0: { (S2, P, O2) }                    -- 1 active slot.
    // Graph 1: { (S1, P, O1), (S1, P_OTHER, O2) } -- 2 active slots.
    let enc: [[[Field; 3]; 4]; 2] = [
        [[S2, P, O2], [0, 0, 0], [0, 0, 0], [0, 0, 0]],
        [[S1, P, O1], [S1, P_OTHER, O2], [0, 0, 0], [0, 0, 0]],
    ];
    let counts: [u32; 2] = [1, 2];
    let mut commitments: [Field; 2] = [0, 0];
    for g in 0..2 {
        let mut leaves: [Field; 4] = [0; 4];
        for i in 0..4 {
            leaves[i] = h3(enc[g][i][0], enc[g][i][1], enc[g][i][2]);
        }
        commitments[g] = commit_fold(leaves, counts[g]);
    }
    // [OPUS-4.8] Canonical-order invariant: the public commitments this fixture
    // hands to `scan_check` MUST already be strictly increasing (the circuit's
    // step-1b contract). If a future encoding change flips the natural order, this
    // asserts HERE -- a clear fixture bug -- rather than tripping the circuit's
    // "per-graph commitments not strictly increasing" guard inside an unrelated test.
    assert(commitments[0].lt(commitments[1]), "test_graphs fixture not in canonical ascending-commitment order");
    (enc, counts, commitments)
}

fn p_pattern() -> PatternSpec {
    // ?s <P> ?o -- predicate constant, subject/object variables.
    PatternSpec { is_const: [false, true, false], const_enc: [0, P, 0] }
}

// Pattern ?s P ?o matches in BOTH graph 0 and graph 1 (test_graphs): the true
// attribution for the accept paths is therefore [true, true].
global ATTR_BOTH: [bool; 2] = [true, true];

#[test]
fn scan_accepts_complete_match_set() {
    let (enc, counts, commitments) = test_graphs();
    // Pattern ?s P ?o matches (S2,P,O2) in graph 0 and (S1,P,O1) in graph 1
    // (canonical ascending-commitment order, see test_graphs). Disclosed rows are
    // listed in that same graph order: graph-0 row first, then graph-1.
    let rows: [[Field; 3]; 2] = [[S2, P, O2], [S1, P, O1]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 2, ATTR_BOTH, counts, enc);
}

#[test]
fn scan_accepts_constant_only_pattern_with_padding_rows() {
    let (enc, counts, commitments) = test_graphs();
    // Fully-constant pattern (an ASK-shaped membership): one matching row,
    // second row slot padded. (S2,P,O2) lives only in graph 0 (canonical
    // ascending-commitment order, see test_graphs), so the bound attribution is
    // [true, false].
    let pattern = PatternSpec { is_const: [true, true, true], const_enc: [S2, P, O2] };
    let rows: [[Field; 3]; 2] = [[S2, P, O2], [0, 0, 0]];
    scan_check::<2, 4, 2>(commitments, pattern, rows, 1, [true, false], counts, enc);
}

// [OPUS-4.8] audit #8: the per-graph attribution is constrained to the true
// matched-graph set. Lying in EITHER direction is rejected.
#[test(should_fail_with = "attribution does not match the proved source graphs")]
fn scan_rejects_attribution_understated() {
    // The [[0],[0]] collapse-two-graphs lie at the circuit level: the pattern
    // matches in graph 1 too (it holds (S1,P,O1)), but the prover claims
    // attribution[1] = false to hide the cross-graph source. The true attribution
    // is [true, true]; [true, false] understates the genuinely-matching graph 1.
    let (enc, counts, commitments) = test_graphs();
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [S2, P, O2]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 2, [true, false], counts, enc);
}

#[test(should_fail_with = "attribution does not match the proved source graphs")]
fn scan_rejects_attribution_overstated() {
    // Claiming a graph contributes when it does not is also rejected (exact
    // matched set, not a superset). (S2,P,O2) lives only in graph 0, so the true
    // attribution is [true, false]; [true, true] overstates the non-matching
    // graph 1.
    let (enc, counts, commitments) = test_graphs();
    let pattern = PatternSpec { is_const: [true, true, true], const_enc: [S2, P, O2] };
    let rows: [[Field; 3]; 2] = [[S2, P, O2], [0, 0, 0]];
    scan_check::<2, 4, 2>(commitments, pattern, rows, 1, [true, true], counts, enc);
}

#[test(should_fail_with = "graph commitment mismatch")]
fn scan_rejects_tampered_commitment() {
    let (enc, counts, mut commitments) = test_graphs();
    commitments[0] += 1;
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [S2, P, O2]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 2, ATTR_BOTH, counts, enc);
}

#[test(should_fail_with = "graph commitment mismatch")]
fn scan_rejects_tampered_graph_content() {
    let (mut enc, counts, commitments) = test_graphs();
    // Suppress a committed triple by overwriting an active slot ("license:
    // suspended" hiding, plan S1.1) -- the fold no longer reproduces C. Here we
    // overwrite graph 1's first active slot (S1,P,O1); the step-1 recompute of
    // graph 1's commitment then mismatches BEFORE any row/attribution check.
    enc[1][0] = [S1, P_OTHER, O1];
    let rows: [[Field; 3]; 2] = [[S2, P, O2], [0, 0, 0]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 1, [true, false], counts, enc);
}

#[test(should_fail_with = "matching triple missing from disclosed rows")]
fn scan_rejects_suppressed_row() {
    let (enc, counts, commitments) = test_graphs();
    // Graph 0's match (S2,P,O2) is suppressed (only (S1,P,O1) disclosed):
    // completeness sweep must catch the undisclosed matching slot.
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [0, 0, 0]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 1, ATTR_BOTH, counts, enc);
}

#[test(should_fail_with = "disclosed row not in any committed graph")]
fn scan_rejects_fabricated_row() {
    let (enc, counts, commitments) = test_graphs();
    // A row that matches the pattern but is in NO committed graph.
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [999, P, 998]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 2, ATTR_BOTH, counts, enc);
}

#[test(should_fail_with = "disclosed row does not match the pattern")]
fn scan_rejects_off_pattern_row() {
    let (enc, counts, commitments) = test_graphs();
    // A committed triple that does NOT match the pattern's constant slot.
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [S1, P_OTHER, O2]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 2, ATTR_BOTH, counts, enc);
}

#[test(should_fail_with = "matching triple missing from disclosed rows")]
fn scan_rejects_inactive_row_padding_as_disclosure() {
    let (enc, counts, commitments) = test_graphs();
    // The matching triples are in the row array but row_count hides one:
    // padding rows must NOT satisfy completeness.
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [S2, P, O2]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 1, ATTR_BOTH, counts, enc);
}

#[test(should_fail_with = "graph size exceeds circuit bucket")]
fn scan_rejects_oversized_count() {
    let (enc, _, commitments) = test_graphs();
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [S2, P, O2]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 2, ATTR_BOTH, [5, 1], enc);
}

#[test(should_fail_with = "row_count exceeds circuit bucket")]
fn scan_rejects_oversized_row_count() {
    let (enc, counts, commitments) = test_graphs();
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [S2, P, O2]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 3, ATTR_BOTH, counts, enc);
}

// ---------------------------------------------------------------------------
// filter_int -- accept + adversarial.
// ---------------------------------------------------------------------------

/// Test-side replica of the operand binding (digits -> Enc). The
/// NON-circular anchor for this construction is the Rust e2e
/// (crates/sparq-zk-compose), which computes `operand_enc` via
/// `sparq_zk::encode` and proves against these circuits.
fn enc_int_literal<let D: u32>(digits: [u8; D]) -> Field {
    let suffix: [u8; 44] = "^^<http://www.w3.org/2001/XMLSchema#integer>".as_bytes();
    let mut token: [u8; D + 46] = [0; D + 46];
    token[0] = 34;
    for i in 0..D {
        token[i + 1] = digits[i];
    }
    token[D + 1] = 34;
    for i in 0..44 {
        token[D + 2 + i] = suffix[i];
    }
    let digest: [u8; 32] = std::hash::blake3(token);
    let mut hs: Field = 0;
    for i in 1..32 {
        hs = hs * 256 + (digest[i] as Field);
    }
    h2(TYPE_CODE_LITERAL, hs)
}

#[test]
fn filter_int_all_ops_verdicts() {
    let digits: [u8; 2] = [50, 53]; // "25"
    let enc = enc_int_literal(digits);
    // 25 vs 18: lt false, le false, gt true, ge true, eq false, ne true.
    filter_int_check::<2>(enc, OP_LT, 18, false, digits);
    filter_int_check::<2>(enc, OP_LE, 18, false, digits);
    filter_int_check::<2>(enc, OP_GT, 18, true, digits);
    filter_int_check::<2>(enc, OP_GE, 18, true, digits);
    filter_int_check::<2>(enc, OP_EQ, 18, false, digits);
    filter_int_check::<2>(enc, OP_NE, 18, true, digits);
    // Boundary: 25 vs 25.
    filter_int_check::<2>(enc, OP_GE, 25, true, digits);
    filter_int_check::<2>(enc, OP_GT, 25, false, digits);
    filter_int_check::<2>(enc, OP_EQ, 25, true, digits);
}

#[test]
fn filter_int_single_digit_zero_is_canonical() {
    let digits: [u8; 1] = [48]; // "0"
    let enc = enc_int_literal(digits);
    filter_int_check::<1>(enc, OP_LT, 1, true, digits);
}

#[test(should_fail_with = "filter verdict mismatch")]
fn filter_int_rejects_lying_verdict() {
    let digits: [u8; 2] = [49, 55]; // "17"
    let enc = enc_int_literal(digits);
    // 17 > 18 claimed true -- the per-row pruning lie a prover would need to
    // drop a result row (or keep a failing one).
    filter_int_check::<2>(enc, OP_GT, 18, true, digits);
}

#[test(should_fail_with = "operand encoding mismatch")]
fn filter_int_rejects_swapped_operand() {
    // Digits for 99, anchor encoding for 17: the prover cannot satisfy the
    // predicate with a different committed value.
    let enc17 = enc_int_literal([49, 55]);
    filter_int_check::<2>(enc17, OP_GT, 18, true, [57, 57]);
}

#[test(should_fail_with = "not a decimal digit")]
fn filter_int_rejects_non_digit_bytes() {
    let digits: [u8; 2] = [49, 65]; // "1A"
    let enc = enc_int_literal(digits);
    filter_int_check::<2>(enc, OP_GT, 0, true, digits);
}

#[test(should_fail_with = "non-canonical leading zero")]
fn filter_int_rejects_leading_zero() {
    let digits: [u8; 2] = [48, 53]; // "05"
    let enc = enc_int_literal(digits);
    filter_int_check::<2>(enc, OP_LT, 18, true, digits);
}

#[test(should_fail_with = "unknown comparison operator")]
fn filter_int_rejects_unknown_op() {
    let digits: [u8; 1] = [53];
    let enc = enc_int_literal(digits);
    filter_int_check::<1>(enc, 6, 1, true, digits);
}

// ---------------------------------------------------------------------------
// filter_f64 -- IEEE semantics via sparq_ieee754.
// ---------------------------------------------------------------------------

global F64_ONE_POINT_FIVE: u64 = 0x3FF8000000000000;
global F64_TWO_POINT_FIVE: u64 = 0x4004000000000000;
global F64_NAN: u64 = 0x7FF8000000000000;
global F64_POS_ZERO: u64 = 0x0000000000000000;
global F64_NEG_ZERO: u64 = 0x8000000000000000;

#[test]
fn filter_f64_ordered_comparisons() {
    filter_f64_check(F64_ONE_POINT_FIVE, F64_TWO_POINT_FIVE, OP_LT, true);
    filter_f64_check(F64_ONE_POINT_FIVE, F64_TWO_POINT_FIVE, OP_GE, false);
    filter_f64_check(F64_TWO_POINT_FIVE, F64_ONE_POINT_FIVE, OP_GT, true);
    filter_f64_check(F64_ONE_POINT_FIVE, F64_ONE_POINT_FIVE, OP_LE, true);
    filter_f64_check(F64_ONE_POINT_FIVE, F64_ONE_POINT_FIVE, OP_EQ, true);
}

#[test]
fn filter_f64_ieee_edge_cases() {
    // NaN is unordered against everything, including itself; ne(NaN) = true.
    filter_f64_check(F64_NAN, F64_ONE_POINT_FIVE, OP_LT, false);
    filter_f64_check(F64_NAN, F64_ONE_POINT_FIVE, OP_GE, false);
    filter_f64_check(F64_NAN, F64_NAN, OP_EQ, false);
    filter_f64_check(F64_NAN, F64_NAN, OP_NE, true);
    // -0 == +0 despite distinct bit patterns.
    filter_f64_check(F64_NEG_ZERO, F64_POS_ZERO, OP_EQ, true);
    filter_f64_check(F64_NEG_ZERO, F64_POS_ZERO, OP_LT, false);
}

#[test(should_fail_with = "float filter verdict mismatch")]
fn filter_f64_rejects_lying_verdict() {
    filter_f64_check(F64_TWO_POINT_FIVE, F64_ONE_POINT_FIVE, OP_LT, true);
}

// ---------------------------------------------------------------------------
// revoke -- hidden-index status-list inclusion + bit-unset (sq-3e5 / sq-h2v).
// ---------------------------------------------------------------------------
//
// A depth-2 tree over 4 leaves built BY HAND with the same `h2` the Rust root
// builder uses. Leaves are the status bits (0 = active, 1 = revoked):
//   leaf0=0  leaf1=1  leaf2=0  leaf3=0
//   n01 = h2(leaf0, leaf1)   n23 = h2(leaf2, leaf3)
//   root = h2(n01, n23)
// The proof for an active index reveals neither the index nor the bits; the
// witness here just exercises the relation in-circuit.

// Authentication path for index 0 (an UNSET leaf): bottom-up siblings are
// [leaf1, n23]; directions are LSB-first bits of 0 = [0,0] (left, left).
#[test]
fn revoke_unset_index0_active_accepts() {
    let leaf0: Field = 0;
    let leaf1: Field = 1;
    let n23 = h2(0, 0); // h2(leaf2=0, leaf3=0)
    let root = h2(h2(leaf0, leaf1), n23);
    revoke_unset_check::<2>(root, 0, leaf0, [leaf1, n23]);
}

// Index 2 (also UNSET): directions = bits of 2 = [0,1]; siblings = [leaf3, n01].
#[test]
fn revoke_unset_index2_active_accepts() {
    let n01 = h2(0, 1); // h2(leaf0=0, leaf1=1)
    let leaf2: Field = 0;
    let leaf3: Field = 0;
    let root = h2(n01, h2(leaf2, leaf3));
    revoke_unset_check::<2>(root, 2, leaf2, [leaf3, n01]);
}

// A REVOKED leaf (bit==1, index 1) must be REJECTED: the bit-unset assertion
// fails. The holder of a revoked credential cannot produce a proof.
#[test(should_fail_with = "credential is revoked: status bit is set")]
fn revoke_set_bit_rejected() {
    let leaf0: Field = 0;
    let leaf1: Field = 1;
    let n23 = h2(0, 0);
    let root = h2(h2(leaf0, leaf1), n23);
    // index 1, bit 1 (revoked): path siblings bottom-up = [leaf0, n23],
    // directions = bits of 1 = [1,0].
    revoke_unset_check::<2>(root, 1, leaf1, [leaf0, n23]);
}

// A leaf whose authentication path does NOT recompute the committed root is
// rejected (a forged inclusion / wrong sibling).
#[test(should_fail_with = "merkle inclusion does not recompute the committed root")]
fn revoke_wrong_path_rejected() {
    let leaf0: Field = 0;
    let leaf1: Field = 1;
    let n23 = h2(0, 0);
    let root = h2(h2(leaf0, leaf1), n23);
    // Correct directions for index 0, but a WRONG sibling at level 0 (99 not
    // leaf1) so the fold misses the root.
    revoke_unset_check::<2>(root, 0, leaf0, [99, n23]);
}

// The index IS the path direction (directions are decomposed FROM `index`
// in-circuit, not a separate witness). So claiming index 2 (directions [0,1])
// while supplying index-0's authentication path ([leaf1, n23]) folds along the
// WRONG side at level 1 and cannot recompute the root -- rejected. A prover
// therefore cannot reuse another (unset) leaf's path under a different claimed
// index: changing the index changes the fold directions, breaking inclusion.
#[test(should_fail_with = "merkle inclusion does not recompute the committed root")]
fn revoke_index_path_mismatch_rejected() {
    let leaf0: Field = 0;
    let leaf1: Field = 1;
    let n23 = h2(0, 0);
    let root = h2(h2(leaf0, leaf1), n23);
    revoke_unset_check::<2>(root, 2, leaf0, [leaf1, n23]);
}

// `index` is range-bound to `2^D` by the in-circuit bit-decomposition: an index
// >= 2^D has a nonzero bit beyond position D, which `to_le_bits::<D>` constrains
// to zero, so the witness is unsatisfiable. (D=2 here => valid indices 0..=3;
// index 4 = 0b100 overflows.) This pins the in-range guarantee the tree needs.
#[test(should_fail)]
fn revoke_out_of_range_index_rejected() {
    let leaf0: Field = 0;
    let leaf1: Field = 1;
    let n23 = h2(0, 0);
    let root = h2(h2(leaf0, leaf1), n23);
    revoke_unset_check::<2>(root, 4, leaf0, [leaf1, n23]);
}

// [OPUS-4.8] sq-ayv: the committed variant binds a hiding index commitment to the
// SAME private index proven unset. ACCEPT: index 0 active, with the CORRECT
// commitment h3(DOMAIN, index, blinding) supplied as the public input.
#[test]
fn revoke_committed_active_with_correct_commitment_accepts() {
    let leaf0: Field = 0;
    let leaf1: Field = 1;
    let n23 = h2(0, 0);
    let root = h2(h2(leaf0, leaf1), n23);
    let index: Field = 0;
    let blinding: Field = 0xb1; // the holder's hiding randomness
    let index_commitment = h3(SIG_DOMAIN_STATUS_IDX_COMMIT, index, blinding);
    revoke_unset_check_committed::<2>(root, index_commitment, index, leaf0, blinding, [leaf1, n23]);
}

// REJECT: the public index commitment does NOT bind the proven-unset index (it
// was computed for a DIFFERENT index 2). A holder cannot sign over a commitment
// to its revoked index and prove bit-unset for some other index -- the in-circuit
// recompute from the proof's own `index` (0) differs from the supplied commitment.
#[test(should_fail_with = "index commitment does not bind the proven-unset index")]
fn revoke_committed_wrong_index_commitment_rejected() {
    let leaf0: Field = 0;
    let leaf1: Field = 1;
    let n23 = h2(0, 0);
    let root = h2(h2(leaf0, leaf1), n23);
    let blinding: Field = 0xb1;
    // Commitment for index 2, but the proof proves index 0 active.
    let wrong_commitment = h3(SIG_DOMAIN_STATUS_IDX_COMMIT, 2, blinding);
    revoke_unset_check_committed::<2>(root, wrong_commitment, 0, leaf0, blinding, [leaf1, n23]);
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-z9l: in-circuit Schnorr-over-Baby-JubJub + hidden-key set
// membership. Vectors generated by `sparq_zk_compose::issuer` from a REAL
// signature (sig.rs Schnorr-over-Baby-JubJub) over commitment_message(0xc0ffee):
// a key set K = {seed100, seed101, seed102, seed103} committed as a depth-2
// Poseidon2 Merkle tree; the signer is seed102 at index 2; the membership path
// is [leaf(seed103), h2(leaf100, leaf101)]. The out-of-set vectors are a REAL
// signature by seed200 (a key NOT in K).
// ---------------------------------------------------------------------------

use crate::issuer::{hidden_issuer_attestation, key_set_membership, schnorr_verify, Point};

// In-set signer (seed102, index 2 in K).
global IV_M: Field = 0x18bca25b3980838a520d97b3d360b9d88fa497367d04d73db872ad86f05b9bab;
global IV_PK_X: Field = 0x2f55331c1d80c2398a6ca962b853ac337350e2ff4c17f1842024337ed07190c2;
global IV_PK_Y: Field = 0x10427ec74fc160b890be66dbc372aa61108183ddd27765186f476ff50044ddbe;
global IV_R_X: Field = 0x240647481be38e09b1e68097f680a049cd9000fd25da4ab17ecee8e1b0901ef9;
global IV_R_Y: Field = 0x0d0dc60835d519ccb1ea434908c96f409ce46a9df1ee570775439c7be7166b1a;
global IV_S: Field = 0x0371b558bbb53d69bb2cd844e813d01aff66e444a9aa8b3bbce206213276c7d1;
global IV_E: Field = 0x028528f1d34f843c12e5a7394de73ea7dd1e15d2c841114d52ed3d23a86c9760;
global IV_E_K: Field = 0x0000000000000000000000000000000000000000000000000000000000000003;
global IV_ROOT: Field = 0x104b3a86e296bb7a523bf0b8319dc05f2714a1ad5e4e805a65d7f78b40690b87;
global IV_SIB0: Field = 0x1e04339eeca97bcd93a8a4d853e159d48e7639c64e98dea5bd85fe52ae1bc75c;
global IV_SIB1: Field = 0x0476a552751e04b2d3eaf2726e9192f83da0ec60b186b1b0a85def79400d12e9;
global IV_IDX: Field = 2;

// Out-of-set signer (seed200, NOT in K).
global OV_PK_X: Field = 0x200dfe61440979d4e8b0827802a9ea8940fe2189c385b838f8305684ce5a56e8;
global OV_PK_Y: Field = 0x032de7d8b3f3f23488bd0819b6949e13456dc9a9d315cd7bfa7b87b7666ea421;
global OV_R_X: Field = 0x07fafb2d1553cc777b04eb9ed32b51d360a2ea02c2663a05b2e95c5d2b866600;
global OV_R_Y: Field = 0x24b9c4a554267ee5b4d5631d816eb1b022a101daccb32228d307cc75dbe1d1aa;
global OV_S: Field = 0x060040d301f61065d8bfcce98a1847a28ee8a57978be00f63b3c65af39e85673;
global OV_E: Field = 0x00216946ce445d7b02ac66fcd1c5f8a0f93d3950929ba2abb2e0bd4e8cacba00;
global OV_E_K: Field = 0x0000000000000000000000000000000000000000000000000000000000000003;

fn in_set_pk() -> Point { Point { x: IV_PK_X, y: IV_PK_Y } }
fn in_set_r() -> Point { Point { x: IV_R_X, y: IV_R_Y } }

// (1) ACCEPT: a valid signature by an IN-SET issuer verifies end-to-end, and the
// issuer key is NOT among the public inputs (m, key_set_root only). The full
// relation: Schnorr verify + key-set membership.
#[test]
fn hidden_issuer_in_set_signature_verifies() {
    hidden_issuer_attestation::<2>(
        IV_M, IV_ROOT,
        in_set_pk(), in_set_r(), IV_S, IV_E, IV_E_K,
        IV_IDX, [IV_SIB0, IV_SIB1],
    );
}

// (1b) The Schnorr gadget alone verifies the real signature (isolates the
// curve-arithmetic + challenge-binding from the membership fold).
#[test]
fn hidden_issuer_schnorr_equation_holds() {
    schnorr_verify(in_set_pk(), in_set_r(), IV_S, IV_E, IV_E_K, IV_M);
}

// (1c) The membership gadget alone: the in-set key folds to the public root.
#[test]
fn hidden_issuer_membership_holds() {
    key_set_membership::<2>(in_set_pk(), IV_ROOT, IV_IDX, [IV_SIB0, IV_SIB1]);
}

// (2) REJECT out-of-set key: a REAL signature by an out-of-set issuer (seed200)
// is a valid signature (the schnorr gadget would accept it) but the key is NOT a
// member of K, so the membership fold cannot recompute the public root.
#[test(should_fail_with = "issuer key is not a member of the committed key set")]
fn hidden_issuer_out_of_set_key_rejected() {
    // The out-of-set key's own valid signature, but folded against K's root with
    // the in-set member's path (the only path the prover could try to present).
    hidden_issuer_attestation::<2>(
        IV_M, IV_ROOT,
        Point { x: OV_PK_X, y: OV_PK_Y },
        Point { x: OV_R_X, y: OV_R_Y },
        OV_S, OV_E, OV_E_K,
        IV_IDX, [IV_SIB0, IV_SIB1],
    );
}

// (2b) The out-of-set signature is itself VALID (Schnorr equation holds) -- so
// the rejection in (2) is purely the membership gate, not a bad signature. This
// is the analogue of sig.rs's "a real sig by an untrusted key" forge.
#[test]
fn hidden_issuer_out_of_set_signature_is_itself_valid() {
    schnorr_verify(
        Point { x: OV_PK_X, y: OV_PK_Y },
        Point { x: OV_R_X, y: OV_R_Y },
        OV_S, OV_E, OV_E_K, IV_M,
    );
}

// (3) REJECT forged/invalid signature: tampering the signature scalar s breaks
// the verification equation s*G == R + e*pk.
#[test(should_fail_with = "schnorr verification equation failed")]
fn hidden_issuer_tampered_s_rejected() {
    schnorr_verify(in_set_pk(), in_set_r(), IV_S + 1, IV_E, IV_E_K, IV_M);
}

// (3b) REJECT a forged challenge: substituting a different e (not e_base mod L)
// breaks the reduction binding e_base == e + e_k*L.
#[test(should_fail_with = "challenge reduction not bound to its preimage")]
fn hidden_issuer_forged_challenge_rejected() {
    schnorr_verify(in_set_pk(), in_set_r(), IV_S, IV_E + 1, IV_E_K, IV_M);
}

// (3c) REJECT a wrong message: a signature over IV_M must not verify against a
// different message (the challenge recomputed over m' diverges, so the bound e
// no longer matches the equation). This is the truncated-commitment / message-
// swap shape from sig.rs::wrong_message_rejected.
#[test(should_fail_with = "challenge reduction not bound to its preimage")]
fn hidden_issuer_wrong_message_rejected() {
    schnorr_verify(in_set_pk(), in_set_r(), IV_S, IV_E, IV_E_K, IV_M + 1);
}

// (4) REJECT identity issuer key (universal-forgery guard, sig.rs codex #3).
#[test(should_fail_with = "identity issuer key rejected")]
fn hidden_issuer_identity_key_rejected() {
    // The neutral point (0, 1). s*G == R is satisfiable for any (R = s*G), so the
    // gadget must reject it before the equation can be gamed.
    schnorr_verify(Point { x: 0, y: 1 }, in_set_r(), IV_S, IV_E, IV_E_K, IV_M);
}

// (5) REJECT off-curve key: an off-curve pk (a forger's twist/small-subgroup
// point) is rejected by the on-curve guard before the equation.
#[test(should_fail_with = "point is not on Baby-JubJub")]
fn hidden_issuer_off_curve_key_rejected() {
    // (1, 1) is not on Baby-JubJub (1 + 1 != 1 + d).
    schnorr_verify(Point { x: 1, y: 1 }, in_set_r(), IV_S, IV_E, IV_E_K, IV_M);
}

// [OPUS-4.8] sq-ayv: pin the index-commitment cross-vector so the Rust
// `status_index_commitment` and this in-circuit `h3(DOMAIN, index, blinding)`
// stay bit-identical (the cross-binding rests on them computing the same value).
#[test]
fn revoke_idx_commit_cross_vector() {
    assert_eq(
        h3(SIG_DOMAIN_STATUS_IDX_COMMIT, 0, 0xb1),
        0x204fe7eb53d68b4f2c7b981619ed4fbd5f3b99879e0f387761bf489a21456118,
    );
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-xqfg (HolderPoP T5): in-circuit holder Proof-of-Possession (B2).
// Vectors generated by `sparq_zk_compose::holder` (via
// `sparq_zk::sig::in_circuit_holder_witness`) from a REAL holder key:
//   - HSK_* / HPK_* / HDIG: the holder secret + public key + holder_key_digest for
//     the seed-102 key (the SAME key as the IV_PK_* hidden-issuer vectors above, so
//     HPK_X/HPK_Y == IV_PK_X/IV_PK_Y -- cross-confirming the host witness).
//   - HPK2_* / HDIG2: a DIFFERENT holder key (seed 200, the OV_PK_* key), used to
//     show the digest is key-specific (the binding A cannot reuse with its own key).
// The digest is `holder_key_digest(hpk) = Poseidon2([ZKSIG_HK, hpk.x, hpk.y])` --
// bit-identical to the host helper and the issuer-attested value the T6 verifier
// gate cross-checks.
// ---------------------------------------------------------------------------

use crate::holder::{holder_key_digest as holder_digest, holder_pok};

// seed-102 holder key (== IV_PK_X / IV_PK_Y).
global HSK: Field = 0x04c49ec34f100efeb528ac3d436a6e1a2cb6b0c85fab6a485462c74c12a82d15;
global HPK_X: Field = 0x2f55331c1d80c2398a6ca962b853ac337350e2ff4c17f1842024337ed07190c2;
global HPK_Y: Field = 0x10427ec74fc160b890be66dbc372aa61108183ddd27765186f476ff50044ddbe;
global HDIG: Field = 0x19e7f4238df100483b1786ae2bd0e8e1c06fc0660ee1016430e95bf824d6e12b;

// seed-200 holder key (== OV_PK_X / OV_PK_Y), a DIFFERENT key.
global HSK2: Field = 0x0222b007e0222f517ed0c2039ede22d3b0566e6e513acb300be1e091d236dc0a;
global HPK2_X: Field = 0x200dfe61440979d4e8b0827802a9ea8940fe2189c385b838f8305684ce5a56e8;
global HPK2_Y: Field = 0x032de7d8b3f3f23488bd0819b6949e13456dc9a9d315cd7bfa7b87b7666ea421;
global HDIG2: Field = 0x1fcb2bb6901f29951dcdc1d75f7d3bee6eca922a34fcd58c88bdc194dba5a559;

fn holder_hpk() -> Point { Point { x: HPK_X, y: HPK_Y } }

// (H0) CROSS-VECTOR: the in-circuit holder_key_digest is bit-identical to the host
// `sparq_zk::sig::holder_key_digest` (the SECURITY-critical T6 cross-check anchor).
#[test]
fn holder_key_digest_cross_vector() {
    assert_eq(holder_digest(holder_hpk()), HDIG);
    assert_eq(holder_digest(Point { x: HPK2_X, y: HPK2_Y }), HDIG2);
}

// (H1) ACCEPT: knowledge of hsk with hpk = hsk*G AND digest match verifies. The
// holder secret and key are PRIVATE; only `holder_pk_digest` is public.
#[test]
fn holder_pok_accepts_valid_possession() {
    holder_pok(HDIG, HSK, holder_hpk());
}

// (H2) REJECT wrong secret: a key not derived from the proven hpk breaks the
// `hpk == hsk*G` relation (HSK2*G is HPK2, not HPK). A holder who does NOT hold
// the bound secret cannot satisfy it.
#[test(should_fail_with = "holder key mismatch")]
fn holder_pok_rejects_wrong_secret() {
    holder_pok(HDIG, HSK2, holder_hpk());
}

// (H3) REJECT key substitution: presenting a DIFFERENT (self-consistent) key pair
// against the issuer-attested digest HDIG fails the digest binding. This is the
// "malicious holder A substitutes its OWN key" forge: hpk2 = hsk2*G holds, but
// holder_key_digest(hpk2) != HDIG. (Design S4.1: A cannot launder B's credential
// under A's own key.)
#[test(should_fail_with = "holder digest mismatch")]
fn holder_pok_rejects_key_substitution() {
    holder_pok(HDIG, HSK2, Point { x: HPK2_X, y: HPK2_Y });
}

// (H4) REJECT digest mismatch: a tampered public digest (not the one bound to the
// real key) is rejected -- the in-circuit digest of the real key cannot equal it.
#[test(should_fail_with = "holder digest mismatch")]
fn holder_pok_rejects_tampered_digest() {
    holder_pok(HDIG + 1, HSK, holder_hpk());
}

// (H5) REJECT identity holder key (universal-degenerate guard, mirrors
// schnorr_verify's identity-key rejection). The neutral (0,1) is hsk*G only for
// hsk == 0 mod L and carries no possession secret.
#[test(should_fail_with = "identity holder key rejected")]
fn holder_pok_rejects_identity_key() {
    // digest of the neutral point, so only the identity guard (not the digest
    // binding) catches the lie.
    let id = Point { x: 0, y: 1 };
    holder_pok(holder_digest(id), 0, id);
}

// (H6) REJECT off-curve holder key (twist / small-subgroup forgery guard).
#[test(should_fail_with = "point is not on Baby-JubJub")]
fn holder_pok_rejects_off_curve_key() {
    // (1, 1) is not on Baby-JubJub (1 + 1 != 1 + d).
    let off = Point { x: 1, y: 1 };
    holder_pok(holder_digest(off), HSK, off);
}

// ---------------------------------------------------------------------------
// [OPUS-4.8] sq-bwwl / sq-8mwz: join_eq -- hidden cross-credential JOIN.
// accept + each adversarial reject + the public-input-hiding pin. Tiny (N_A=4,
// N_B=4) monomorphisations; the equality structure is what matters.
// ---------------------------------------------------------------------------
//
// Universe: two credentials sharing the join key JK (a global IRI encoding) at
// graph-A object slot 2 and graph-B object slot 2 (the `?p` join column).
//   Graph A: { (S1, P, JK) }                  -- ?p in slot 2 (object)
//   Graph B: { (JK, P_OTHER, O2) }            -- ?p in slot 0 (subject)
// So a true join is row_a[2] == row_b[0] == JK, slot_a = 2, slot_b = 0.
global JK: Field = 0xabc123; // the (hidden) shared join-key encoding
global JB: Field = 0xb11d; // the per-presentation blinder

// Build a single-triple graph's commitment over an N=4 bucket (the same
// commit_fold the join relation re-derives from witnessed contents).
fn commit_one(triple: [Field; 3]) -> Field {
    let leaves: [Field; 4] = [h3(triple[0], triple[1], triple[2]), 0, 0, 0];
    commit_fold(leaves, 1)
}

// (J0) Cross-vector: the in-circuit join-value commitment matches the host
// `sparq_zk::sig::join_value_commitment` (single source of truth). The host
// half is pinned in `crates/sparq-zk/src/sig.rs::join_value_commitment_is_*`;
// JOIN_DOMAIN here == SIG_DOMAIN_JOIN there ("ZKSIG_JN" = 0x5a4b5349475f4a4e).
#[test]
fn join_value_commitment_matches_host() {
    assert_eq(JOIN_DOMAIN, 0x5a4b5349475f4a4e);
    // h3(JOIN_DOMAIN, value, blinding) -- the exact preimage the host computes.
    assert_eq(join_value_commitment(JK, JB), h3(JOIN_DOMAIN, JK, JB));
}

// (J1) ACCEPT: two credentials genuinely share JK at the bound slots; the
// equality proves and the hiding commitment binds the join value.
#[test]
fn join_eq_genuine_join_accepts() {
    let row_a = [101, 7, JK]; // (S1, P, JK)
    let row_b = [JK, 8, 202]; // (JK, P_OTHER, O2)
    let enc_a = [row_a, [0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let enc_b = [row_b, [0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let commit_a = commit_one(row_a);
    let commit_b = commit_one(row_b);
    let jc = join_value_commitment(JK, JB);
    join_eq_check::<4, 4>(commit_a, commit_b, jc, 2, 0, enc_a, 1, enc_b, 1, row_a, row_b, JB);
}

// (J2 / A1) REJECT forged equality: the two slots hold DIFFERENT terms. A prover
// claiming a join of unequal terms has no satisfying witness -- the in-circuit
// equality fails. (Graph B's subject is a DIFFERENT key JK2 != JK.)
#[test(should_fail_with = "join slots are not equal")]
fn join_eq_unequal_slots_rejected() {
    let jk2: Field = 0xdead; // a DIFFERENT join key
    let row_a = [101, 7, JK];
    let row_b = [jk2, 8, 202];
    let enc_a = [row_a, [0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let enc_b = [row_b, [0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let commit_a = commit_one(row_a);
    let commit_b = commit_one(row_b);
    // Prover claims equality (and commits to JK), but row_b[0] = jk2 != JK.
    let jc = join_value_commitment(JK, JB);
    join_eq_check::<4, 4>(commit_a, commit_b, jc, 2, 0, enc_a, 1, enc_b, 1, row_a, row_b, JB);
}

// (J3 / A2) REJECT row-swap / forged graph: the prover supplies graph-A contents
// that do NOT recommit to the PUBLIC commit_a (it tries to pass a fabricated
// graph the scan proof never attested). The commitment recompute (step 1)
// catches it before any equality is even reached.
#[test(should_fail_with = "graph-A commitment mismatch")]
fn join_eq_forged_graph_a_rejected() {
    let row_a = [101, 7, JK];
    let row_b = [JK, 8, 202];
    let enc_a = [row_a, [0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let enc_b = [row_b, [0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let commit_b = commit_one(row_b);
    let jc = join_value_commitment(JK, JB);
    // commit_a is for a DIFFERENT graph (a row the prover does not actually
    // hold) -- it cannot match the re-commit of enc_a.
    let bogus_commit_a = commit_one([999, 999, 999]);
    join_eq_check::<4, 4>(bogus_commit_a, commit_b, jc, 2, 0, enc_a, 1, enc_b, 1, row_a, row_b, JB);
}

// (J4 / A2) REJECT join over a row NOT present in the (correctly committed)
// graph: enc_a/commit_a are consistent, but row_a is a triple the graph does not
// contain (it shares JK in slot 2, so the equality WOULD pass, but the
// present-in-graph check fails -- the join must attach to a real credential row).
#[test(should_fail_with = "join row A not present in committed graph A")]
fn join_eq_row_not_in_graph_rejected() {
    let real_a = [101, 7, JK]; // the row actually in graph A
    let row_b = [JK, 8, 202];
    let enc_a = [real_a, [0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let enc_b = [row_b, [0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let commit_a = commit_one(real_a);
    let commit_b = commit_one(row_b);
    let jc = join_value_commitment(JK, JB);
    // A fabricated row_a that shares JK in slot 2 but is NOT a row of graph A.
    let phantom_a = [555, 555, JK];
    join_eq_check::<4, 4>(commit_a, commit_b, jc, 2, 0, enc_a, 1, enc_b, 1, phantom_a, row_b, JB);
}

// (J5) REJECT a join commitment that does NOT bind the proven join value (the
// prover proves equality on JK but commits to a DIFFERENT value -- equivocation
// guard, design S4.3).
#[test(should_fail_with = "join commitment does not bind the proven join value")]
fn join_eq_wrong_commitment_rejected() {
    let row_a = [101, 7, JK];
    let row_b = [JK, 8, 202];
    let enc_a = [row_a, [0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let enc_b = [row_b, [0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let commit_a = commit_one(row_a);
    let commit_b = commit_one(row_b);
    // Commitment to a DIFFERENT value than the join value JK that was proven.
    let wrong_jc = join_value_commitment(0xfeed, JB);
    join_eq_check::<4, 4>(commit_a, commit_b, wrong_jc, 2, 0, enc_a, 1, enc_b, 1, row_a, row_b, JB);
}

// (J6) REJECT an out-of-range join slot (must be subject/predicate/object).
#[test(should_fail_with = "join slot out of range (must be subject/predicate/object)")]
fn join_eq_out_of_range_slot_rejected() {
    let row_a = [101, 7, JK];
    let row_b = [JK, 8, 202];
    let enc_a = [row_a, [0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let enc_b = [row_b, [0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let commit_a = commit_one(row_a);
    let commit_b = commit_one(row_b);
    let jc = join_value_commitment(JK, JB);
    join_eq_check::<4, 4>(commit_a, commit_b, jc, 3, 0, enc_a, 1, enc_b, 1, row_a, row_b, JB);
}

// (J7) PUBLIC-INPUT HIDING PIN (the headline privacy property). The relation's
// PUBLIC interface is exactly [challenge, commit_a, commit_b, join_commitment,
// slot_a, slot_b] (see `join_eq_na16_nb16/src/main.nr`). This test pins that the
// joined term encoding JK is ABSENT from every one of those public artefacts:
//   - commit_a / commit_b are commitments over the WHOLE graph (JK is one of
//     many absorbed leaves -- not recoverable without the full preimage);
//   - join_commitment is HIDING (Poseidon2 over JK *and a secret blinder*), so
//     it differs under a different blinder and is not a bare Enc(JK);
//   - slot_a / slot_b are positions, not values.
// A verifier reading the public inputs never sees JK. (The accept path J1 proves
// the same witnesses verify; here we assert the join VALUE does not equal any
// public field, and that the hiding commitment is blinder-dependent.)
#[test]
fn join_eq_value_absent_from_public_inputs() {
    let row_a = [101, 7, JK];
    let row_b = [JK, 8, 202];
    let commit_a = commit_one(row_a);
    let commit_b = commit_one(row_b);
    let jc = join_value_commitment(JK, JB);
    let slot_a: Field = 2;
    let slot_b: Field = 0;
    // The hidden join value is not literally any public input.
    assert(JK != commit_a);
    assert(JK != commit_b);
    assert(JK != jc);
    assert(JK != slot_a);
    assert(JK != slot_b);
    // The public join artefact is HIDING: the SAME join value under a DIFFERENT
    // blinder yields a different commitment, so the verifier cannot dictionary
    // -attack JK by recomputing a bare encoding (design S1.4 R4).
    let jc_other_blinder = join_value_commitment(JK, JB + 1);
    assert(jc != jc_other_blinder);
    // And the commitment is NOT the bare encoding of the value (which a verifier
    // *could* recompute from a guessed JK) -- the blinder is load-bearing.
    assert(jc != JK);
}

// (J8) ACCEPT a join where the shared key sits at DIFFERENT slots in each
// credential and one graph has MORE than one triple (the join row is selected
// from among several present rows). Exercises select_slot at slot 1 and the
// multi-row present-in-graph sweep.
#[test]
fn join_eq_mixed_slots_multi_row_accepts() {
    // Graph A holds two triples; the join key JK is in slot 1 (predicate-position
    // join, e.g. a reified statement) of the SECOND triple.
    let a0 = [11, 22, 33];
    let a1 = [44, JK, 66];
    let enc_a = [a0, a1, [0, 0, 0], [0, 0, 0]];
    // Graph B holds JK in slot 2 (object).
    let row_b = [77, 88, JK];
    let enc_b = [row_b, [0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let commit_a = commit_fold([h3(a0[0], a0[1], a0[2]), h3(a1[0], a1[1], a1[2]), 0, 0], 2);
    let commit_b = commit_one(row_b);
    let jc = join_value_commitment(JK, JB);
    // slot_a = 1 (predicate of a1), slot_b = 2 (object of row_b); join on a1.
    join_eq_check::<4, 4>(commit_a, commit_b, jc, 1, 2, enc_a, 2, enc_b, 1, a1, row_b, JB);
}
