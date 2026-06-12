//! Test chunk 0 for xs:integer-from-double
//! Contains 3 tests

use dep::xpath::{cast_double_to_integer, XsdDouble};

#[test]
fn castas214() {
    // Try casting xs:double(-0.0E0) to xs:integer
    let d = XsdDouble::from_bits(9223372036854775808);
    assert(cast_double_to_integer(d).is_some());
    assert(cast_double_to_integer(d).unwrap() == 0);
}

#[test]
fn castas215() {
    // Try casting xs:double(1e5) to xs:integer
    let d = XsdDouble::from_bits(4681608360884174848);
    assert(cast_double_to_integer(d).is_some());
    assert(cast_double_to_integer(d).unwrap() == 100000);
}

#[test]
fn castas216() {
    // Try casting xs:double(-1.75e-3) to xs:integer
    let d = XsdDouble::from_bits(13789085310285965689);
    assert(cast_double_to_integer(d).is_some());
    assert(cast_double_to_integer(d).unwrap() == 0);
}
