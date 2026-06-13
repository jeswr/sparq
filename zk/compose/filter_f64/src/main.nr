// xsd:double FILTER comparison BUILDING BLOCK (gate-counted, tested; not yet
// manifest-composable -- the f64-bits-to-canonical-lexical operand binding is
// a documented v1 deferral, see sparq_zk_compose_core::filter_float docs and
// crates/sparq-zk-compose/README.md).
use sparq_zk_compose_core::filter_float::filter_f64_check;

fn main(
    // -- public --
    challenge: pub Field, // verifier nonce (replay binding S2.5)
    op: pub u32,          // 0=lt 1=le 2=gt 3=ge 4=eq 5=ne
    b_bits: pub u64,      // the FILTER's constant operand (f64 bit pattern)
    expected: pub bool,   // the disclosed verdict
    // -- private --
    a_bits: u64,          // the hidden operand (f64 bit pattern)
) {
    let _ = challenge;
    filter_f64_check(a_bits, b_bits, op, expected);
}
