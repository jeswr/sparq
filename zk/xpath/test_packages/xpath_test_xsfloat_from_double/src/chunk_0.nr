//! Test chunk 0 for xs:float-from-double
//! Contains 3 tests

use dep::xpath::{cast_double_to_float, XsdDouble, XsdFloat};

#[test]
fn castas203() {
    // Try casting xs:double(-0.0E0) to xs:float
    let d = XsdDouble::from_bits(9223372036854775808);
    assert(cast_double_to_float(d) == XsdFloat::zero());
}

#[test]
fn castas205() {
    // Try casting xs:double(1e5) to xs:float
    let d = XsdDouble::from_bits(4681608360884174848);
    assert(cast_double_to_float(d).to_bits() == 1203982336);
}

#[test]
fn castas206() {
    // Try casting xs:double(-1.75e-3) to xs:float
    let d = XsdDouble::from_bits(13789085310285965689);
    assert(cast_double_to_float(d).to_bits() == 3135594562);
}
