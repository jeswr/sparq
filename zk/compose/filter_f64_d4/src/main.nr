// [OPUS-4.8] sq-q7e + sq-tat: MANIFEST-COMPOSABLE xsd:double FILTER member with
// D=4 decimal digits over the INTEGER-VALUED double fragment. Unlike the raw
// `filter_f64` building block (free `a_bits`), this binds the hidden operand to
// the committed literal via the canonical token (blake3, like filter_int) and
// DERIVES the IEEE bits from the bound value (`f64::from(value)`), so `a_bits`
// is a constrained function of the operand -- no prover-free bits. Relation:
// sparq_zk_compose_core::filter_float::filter_f64_composable_check.
use sparq_zk_compose_core::filter_float::filter_f64_composable_check;

fn main(
    // -- public --
    challenge: pub Field,   // verifier nonce (replay binding S2.5)
    operand_enc: pub Field, // the hidden column's term encoding (scan-proof anchor)
    op: pub u32,            // 0=lt 1=le 2=gt 3=ge 4=eq 5=ne
    b_bits: pub u64,        // the FILTER's constant operand (f64 bit pattern)
    expected: pub bool,     // the disclosed verdict for this operand
    // -- private --
    digits: [u8; 4],       // canonical decimal digits of the hidden integer-valued double
) {
    let _ = challenge;
    filter_f64_composable_check::<4>(operand_enc, op, b_bits, expected, digits);
}
