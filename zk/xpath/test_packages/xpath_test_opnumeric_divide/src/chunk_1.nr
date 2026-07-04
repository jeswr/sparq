//! Hand-written chunk 1 for op:numeric-divide -- NON-EXACT quotients.
//!
//! NOT auto-generated. The qt3 corpus (chunk_0) only records exact-quotient
//! cases, so it can never distinguish op:numeric-divide from a truncating
//! integer division -- the exact bug class of sq-3x7dl.4. These cases pin the
//! spec semantics: two xs:integer operands yield the xs:decimal quotient
//! (7 div 2 = 3.5), computed here in IEEE 754 double precision (documented
//! approximation -- see numeric_divide_int_as_double). Expected bit patterns
//! from an independent Python hardware oracle:
//! struct.unpack('<Q', struct.pack('<d', a/b)). [FABLE-5]

use dep::xpath::{XsdDouble, xpath_op};

#[test]
fn op_numeric_divide_non_exact_7_2() {
    // 7 div 2 = 3.5 (0x400C000000000000) -- a truncating alias returns 3
    assert(xpath_op::numeric_divide(7, 2) == XsdDouble::from_bits(0x400C000000000000));
}

#[test]
fn op_numeric_divide_non_exact_neg_7_2() {
    // -7 div 2 = -3.5 (0xC00C000000000000)
    assert(xpath_op::numeric_divide(-7, 2) == XsdDouble::from_bits(0xC00C000000000000));
}

#[test]
fn op_numeric_divide_non_exact_1_3() {
    // 1 div 3 = 0.3333333333333333 (0x3FD5555555555555), correctly rounded
    assert(xpath_op::numeric_divide(1, 3) == XsdDouble::from_bits(0x3FD5555555555555));
}

#[test]
fn op_numeric_divide_non_exact_10_4() {
    // 10 div 4 = 2.5 (0x4004000000000000)
    assert(xpath_op::numeric_divide(10, 4) == XsdDouble::from_bits(0x4004000000000000));
}

#[test]
fn op_numeric_divide_distinct_from_integer_divide() {
    // De-aliasing check on the op: wiring itself: op:numeric-divide and
    // op:numeric-integer-divide must be DISTINCT operations on 7/2.
    assert(xpath_op::numeric_integer_divide(7, 2) == 3);
    assert(xpath_op::numeric_divide(7, 2) == XsdDouble::from_bits(0x400C000000000000)); // 3.5
    // 3.5 is not 3.0 (0x4008000000000000)
    assert(!(xpath_op::numeric_divide(7, 2) == XsdDouble::from_bits(0x4008000000000000)));
}
