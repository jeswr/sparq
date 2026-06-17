// [OPUS-4.8] sq-1q9h: xsd:decimal FILTER member with ID=3 integer digits and
// FD=2 fraction digits (e.g. "123.45" -- decimal amounts). Binds the canonical
// '[-]?<int>.<frac>' token (like `filter_int`, plus the point and an optional
// '-') and compares at FD=2 fixed-point places: `bound_scaled = round(|bound| *
// 100)` is the host-prescaled FILTER constant. Relation:
// sparq_zk_compose_core::filter_signed::filter_decimal_check.
use sparq_zk_compose_core::filter_signed::filter_decimal_check;

fn main(
    // -- public --
    challenge: pub Field, // verifier nonce (replay binding S2.5)
    operand_enc: pub Field, // the hidden column's term encoding (scan-proof anchor)
    op: pub u32, // 0=lt 1=le 2=gt 3=ge 4=eq 5=ne
    bound_neg: pub bool, // sign of the FILTER's constant operand
    bound_scaled: pub u64, // round(|FILTER constant| * 10^FD), FD=2
    expected: pub bool, // the disclosed verdict for this operand
    // -- private --
    neg: bool, // sign flag of the hidden value
    int_digits: [u8; 3], // canonical integer-part digits
    frac_digits: [u8; 2], // FD fraction digits (exactly FD)
) {
    let _ = challenge;
    filter_decimal_check::<3, 2>(
        operand_enc,
        op,
        bound_neg,
        bound_scaled,
        expected,
        neg,
        int_digits,
        frac_digits,
    );
}
