// Circuit-family member: hidden xsd:integer FILTER operand with D=3 decimal
// digits (the digit count is a family parameter because the blake3 blackbox
// needs a comptime token length; choosing this member leaks ceil(log10(v))
// of the hidden operand -- documented envelope leak). Relation:
// sparq_zk_compose_core::filter_int.
//
// [OPUS-4.8] sq-wto: fills the 3-digit gap in the compiled {1,2,4} family. The
// derivation now requires an EXACT digit-count match (a 3-digit operand used to
// derive d=4, whose `digits: [u8; 4]` witness no 3-digit value can satisfy --
// a silently-unprovable FILTER). With this member 1..=4 digit operands all have
// a provable circuit; out-of-family counts (5..=19, >19) return None cleanly.
use sparq_zk_compose_core::filter_int::filter_int_check;

fn main(
    // -- public --
    challenge: pub Field,   // verifier nonce (replay binding S2.5)
    operand_enc: pub Field, // the hidden column's term encoding (scan-proof anchor)
    op: pub u32,            // 0=lt 1=le 2=gt 3=ge 4=eq 5=ne
    bound: pub u64,         // the FILTER's constant operand
    expected: pub bool,     // the disclosed verdict for this operand
    // -- private --
    digits: [u8; 3],       // canonical decimal digits of the hidden value
) {
    let _ = challenge;
    filter_int_check::<3>(operand_enc, op, bound, expected, digits);
}
