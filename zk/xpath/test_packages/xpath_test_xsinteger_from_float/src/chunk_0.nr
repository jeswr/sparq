//! Test chunk 0 for xs:integer-from-float
//! Contains 2 tests

use dep::xpath::{cast_float_to_integer, XsdFloat};

#[test]
fn castas178() {
    // Try casting xs:float(-0.0E0) to xs:integer
    let f = XsdFloat::from_bits(2147483648);
    assert(cast_float_to_integer(f).is_some());
    assert(cast_float_to_integer(f).unwrap() == 0);
}

#[test]
fn castas179() {
    // Try casting xs:float(1e5) to xs:integer
    let f = XsdFloat::from_bits(1203982336);
    assert(cast_float_to_integer(f).is_some());
    assert(cast_float_to_integer(f).unwrap() == 100000);
}
