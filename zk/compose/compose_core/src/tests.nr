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
    // Graph 0: { (S1, P, O1), (S1, P_OTHER, O2) } -- 2 active slots.
    // Graph 1: { (S2, P, O2) }                    -- 1 active slot.
    let enc: [[[Field; 3]; 4]; 2] = [
        [[S1, P, O1], [S1, P_OTHER, O2], [0, 0, 0], [0, 0, 0]],
        [[S2, P, O2], [0, 0, 0], [0, 0, 0], [0, 0, 0]],
    ];
    let counts: [u32; 2] = [2, 1];
    let mut commitments: [Field; 2] = [0, 0];
    for g in 0..2 {
        let mut leaves: [Field; 4] = [0; 4];
        for i in 0..4 {
            leaves[i] = h3(enc[g][i][0], enc[g][i][1], enc[g][i][2]);
        }
        commitments[g] = commit_fold(leaves, counts[g]);
    }
    (enc, counts, commitments)
}

fn p_pattern() -> PatternSpec {
    // ?s <P> ?o -- predicate constant, subject/object variables.
    PatternSpec { is_const: [false, true, false], const_enc: [0, P, 0] }
}

#[test]
fn scan_accepts_complete_match_set() {
    let (enc, counts, commitments) = test_graphs();
    // Pattern ?s P ?o matches (S1,P,O1) in graph 0 and (S2,P,O2) in graph 1.
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [S2, P, O2]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 2, counts, enc);
}

#[test]
fn scan_accepts_constant_only_pattern_with_padding_rows() {
    let (enc, counts, commitments) = test_graphs();
    // Fully-constant pattern (an ASK-shaped membership): one matching row,
    // second row slot padded.
    let pattern = PatternSpec { is_const: [true, true, true], const_enc: [S2, P, O2] };
    let rows: [[Field; 3]; 2] = [[S2, P, O2], [0, 0, 0]];
    scan_check::<2, 4, 2>(commitments, pattern, rows, 1, counts, enc);
}

#[test(should_fail_with = "graph commitment mismatch")]
fn scan_rejects_tampered_commitment() {
    let (enc, counts, mut commitments) = test_graphs();
    commitments[0] += 1;
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [S2, P, O2]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 2, counts, enc);
}

#[test(should_fail_with = "graph commitment mismatch")]
fn scan_rejects_tampered_graph_content() {
    let (mut enc, counts, commitments) = test_graphs();
    // Suppress a committed triple by overwriting an active slot ("license:
    // suspended" hiding, plan S1.1) -- the fold no longer reproduces C.
    enc[0][0] = [S1, P_OTHER, O1];
    let rows: [[Field; 3]; 2] = [[S2, P, O2], [0, 0, 0]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 1, counts, enc);
}

#[test(should_fail_with = "matching triple missing from disclosed rows")]
fn scan_rejects_suppressed_row() {
    let (enc, counts, commitments) = test_graphs();
    // Graph 1's match is suppressed: completeness sweep must catch it.
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [0, 0, 0]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 1, counts, enc);
}

#[test(should_fail_with = "disclosed row not in any committed graph")]
fn scan_rejects_fabricated_row() {
    let (enc, counts, commitments) = test_graphs();
    // A row that matches the pattern but is in NO committed graph.
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [999, P, 998]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 2, counts, enc);
}

#[test(should_fail_with = "disclosed row does not match the pattern")]
fn scan_rejects_off_pattern_row() {
    let (enc, counts, commitments) = test_graphs();
    // A committed triple that does NOT match the pattern's constant slot.
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [S1, P_OTHER, O2]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 2, counts, enc);
}

#[test(should_fail_with = "matching triple missing from disclosed rows")]
fn scan_rejects_inactive_row_padding_as_disclosure() {
    let (enc, counts, commitments) = test_graphs();
    // The matching triples are in the row array but row_count hides one:
    // padding rows must NOT satisfy completeness.
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [S2, P, O2]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 1, counts, enc);
}

#[test(should_fail_with = "graph size exceeds circuit bucket")]
fn scan_rejects_oversized_count() {
    let (enc, _, commitments) = test_graphs();
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [S2, P, O2]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 2, [5, 1], enc);
}

#[test(should_fail_with = "row_count exceeds circuit bucket")]
fn scan_rejects_oversized_row_count() {
    let (enc, counts, commitments) = test_graphs();
    let rows: [[Field; 3]; 2] = [[S1, P, O1], [S2, P, O2]];
    scan_check::<2, 4, 2>(commitments, p_pattern(), rows, 3, counts, enc);
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
