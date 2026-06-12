//! Test chunk 0 for xs:float-from-int
//! Contains 17 tests

use dep::xpath::{cast_integer_to_float, XsdFloat};

#[test]
fn casthc2() {
    //  Simple test for evaluation of a simple expression casted as xs:float
    assert(cast_integer_to_float(10).to_bits() == 1092616192);
}

#[test]
fn casthc38() {
    //  Simple test for casting a boolean as float
    assert(cast_integer_to_float(1).to_bits() == 1065353216);
}

#[test]
fn castas258() {
    // Try casting xs:integer(-100) to xs:float
    assert(cast_integer_to_float(-100).to_bits() == 3267887104);
}

#[test]
fn castas552() {
    // Try casting xs:boolean(true) to xs:float
    assert(cast_integer_to_float(1).to_bits() == 1065353216);
}

#[test]
fn castas553() {
    // Try casting xs:boolean(false) to xs:float
    assert(cast_integer_to_float(0) == XsdFloat::zero());
}

#[test]
fn k2_seqexprcast_345() {
    //  Cast xs:byte to xs:float.
    assert(cast_integer_to_float(3).to_bits() == 1077936128);
}

#[test]
fn k2_seqexprcast_346() {
    //  Cast xs:int to xs:float.
    assert(cast_integer_to_float(3).to_bits() == 1077936128);
}

#[test]
fn k2_seqexprcast_347() {
    //  Cast xs:long to xs:float.
    assert(cast_integer_to_float(3).to_bits() == 1077936128);
}

#[test]
fn k2_seqexprcast_348() {
    //  Cast xs:negativeInteger to xs:float.
    assert(cast_integer_to_float(-3).to_bits() == 3225419776);
}

#[test]
fn k2_seqexprcast_349() {
    //  Cast xs:nonNegativeInteger to xs:float.
    assert(cast_integer_to_float(3).to_bits() == 1077936128);
}

#[test]
fn k2_seqexprcast_350() {
    //  Cast xs:nonPositiveInteger to xs:float.
    assert(cast_integer_to_float(0) == XsdFloat::zero());
}

#[test]
fn k2_seqexprcast_351() {
    //  Cast xs:short to xs:float.
    assert(cast_integer_to_float(3).to_bits() == 1077936128);
}

#[test]
fn k2_seqexprcast_352() {
    //  Cast xs:unsignedByte to xs:float.
    assert(cast_integer_to_float(3).to_bits() == 1077936128);
}

#[test]
fn k2_seqexprcast_353() {
    //  Cast xs:unsignedInt to xs:float.
    assert(cast_integer_to_float(3).to_bits() == 1077936128);
}

#[test]
fn k2_seqexprcast_354() {
    //  Cast xs:unsignedInt to xs:float.
    assert(cast_integer_to_float(3).to_bits() == 1077936128);
}

#[test]
fn k2_seqexprcast_355() {
    //  Cast xs:unsignedLong to xs:float.
    assert(cast_integer_to_float(3).to_bits() == 1077936128);
}

#[test]
fn k2_seqexprcast_356() {
    //  Cast xs:unsignedShort to xs:float.
    assert(cast_integer_to_float(3).to_bits() == 1077936128);
}
