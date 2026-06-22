// [OPUS-4.8] sq-1q9h: SIGNED xsd:integer FILTER member with MD=4 MAGNITUDE
// digits. Choosing this member leaks ceil(log10(|value|)) of the hidden
// operand (the documented envelope leak, same as `filter_int_d{D}`). Relation:
// sparq_zk_compose_core::filter_signed::filter_signed_int_check.
use sparq_zk_compose_core::filter_signed::filter_signed_int_check;

fn main(
    // -- public --
    challenge: pub Field,
    operand_enc: pub Field,
    op: pub u32,
    bound_neg: pub bool,
    bound: pub u64,
    expected: pub bool,
    // -- private --
    neg: bool,
    mag_digits: [u8; 4],
) {
    let _ = challenge;
    filter_signed_int_check::<4>(operand_enc, op, bound_neg, bound, expected, neg, mag_digits);
}
