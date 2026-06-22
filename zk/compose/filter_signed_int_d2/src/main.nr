// [OPUS-4.8] sq-1q9h: SIGNED xsd:integer FILTER member with MD=2 MAGNITUDE
// digits (negative coordinates / amounts). Extends `filter_int` (non-negative
// only) by binding an optional leading '-' into the canonical token and doing
// sign-aware comparison. `neg` is the operand's sign flag; `bound_neg`/`bound`
// is the FILTER constant's sign-split magnitude. Relation:
// sparq_zk_compose_core::filter_signed::filter_signed_int_check.
use sparq_zk_compose_core::filter_signed::filter_signed_int_check;

fn main(
    // -- public --
    challenge: pub Field, // verifier nonce (replay binding S2.5)
    operand_enc: pub Field, // the hidden column's term encoding (scan-proof anchor)
    op: pub u32, // 0=lt 1=le 2=gt 3=ge 4=eq 5=ne
    bound_neg: pub bool, // sign of the FILTER's constant operand
    bound: pub u64, // |FILTER constant operand|
    expected: pub bool, // the disclosed verdict for this operand
    // -- private --
    neg: bool, // sign flag of the hidden value
    mag_digits: [u8; 2], // canonical magnitude digits of the hidden value
) {
    let _ = challenge;
    filter_signed_int_check::<2>(operand_enc, op, bound_neg, bound, expected, neg, mag_digits);
}
