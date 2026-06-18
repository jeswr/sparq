// [OPUS-4.8] sq-mslu (sq-7lrq part-2 / sq-lxi7): general xsd:double decimal ->
// IEEE-754 FILTER BUILDING BLOCK for a FRACTIONAL / SCIENTIFIC lexical form,
// over the SOUND integer-VALUED fragment.
//
// This is the gate-counted, tested conversion CORE: it takes the lexically-
// parsed operand components (`sign`, `mantissa_int`, base-10 `dexp`) and a
// public comparison, and converts `sign * mantissa_int * 10^dexp` to its
// correctly-rounded IEEE-754 double via a SINGLE `f64::from(u128)` op (the
// integer-valued fragment, where one rounding is exact -- genuinely-fractional
// values, dexp < 0, are out of the sound fragment and rejected; see the
// `filter_float` module docs).
//
// Like the raw `filter_f64` building block, it takes a raw `b_bits` and is NOT
// yet manifest-composable: composing it would additionally require rebuilding
// the operand's canonical N-Triples token in-circuit, which for xsd:double is
// the shortest-round-trip canonical PRINTING gap (sparq-zk-compose README
// "sparq-zk API gaps"; expected to be subsumed by #514's value-lane which signs
// the IEEE-754 bits directly and removes in-circuit literal parsing).
// Relation: sparq_zk_compose_core::filter_float::filter_f64_decimal_check.
use sparq_zk_compose_core::filter_float::filter_f64_decimal_check;

fn main(
    // -- public --
    challenge: pub Field, // verifier nonce (replay binding S2.5)
    op: pub u32, // 0=lt 1=le 2=gt 3=ge 4=eq 5=ne
    b_bits: pub u64, // the FILTER's constant operand (f64 bit pattern)
    expected: pub bool, // the disclosed verdict
    // -- private --
    sign: bool, // mantissa sign of the hidden xsd:double operand
    mantissa_int: u128, // parsed integer mantissa (digits with the point removed)
    dexp: u32, // base-10 exponent so value = mantissa_int * 10^dexp
) {
    let _ = challenge;
    filter_f64_decimal_check(sign, mantissa_int, dexp, b_bits, op, expected);
}
