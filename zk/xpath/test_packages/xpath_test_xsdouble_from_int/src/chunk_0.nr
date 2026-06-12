//! Test chunk 0 for xs:double-from-int
//! Contains 18 tests

use dep::xpath::{cast_integer_to_double, XsdDouble};

#[test]
fn casthc4() {
    //  Simple test for evaluation of a simple expression casted as xs:double
    assert(cast_integer_to_double(10).to_bits() == 4621819117588971520);
}

#[test]
fn casthc39() {
    //  Simple test for casting a boolean as double
    assert(cast_integer_to_double(1).to_bits() == 4607182418800017408);
}

#[test]
fn castas259() {
    // Try casting xs:integer(-100) to xs:double
    assert(cast_integer_to_double(-100).to_bits() == 13860109328209412096);
}

#[test]
fn castas554() {
    // Try casting xs:boolean(true) to xs:double
    assert(cast_integer_to_double(1).to_bits() == 4607182418800017408);
}

#[test]
fn castas555() {
    // Try casting xs:boolean(false) to xs:double
    assert(cast_integer_to_double(0) == XsdDouble::zero());
}

#[test]
fn k2_seqexprcast_213() {
    //  Cast an xs:positiveInteger value to xs:double.
    assert(cast_integer_to_double(52).to_bits() == 4632515166703976448);
}

#[test]
fn k2_seqexprcast_333() {
    //  Cast xs:byte to xs:double.
    assert(cast_integer_to_double(3).to_bits() == 4613937818241073152);
}

#[test]
fn k2_seqexprcast_334() {
    //  Cast xs:int to xs:double.
    assert(cast_integer_to_double(3).to_bits() == 4613937818241073152);
}

#[test]
fn k2_seqexprcast_335() {
    //  Cast xs:long to xs:double.
    assert(cast_integer_to_double(3).to_bits() == 4613937818241073152);
}

#[test]
fn k2_seqexprcast_336() {
    //  Cast xs:negativeInteger to xs:double.
    assert(cast_integer_to_double(-3).to_bits() == 13837309855095848960);
}

#[test]
fn k2_seqexprcast_337() {
    //  Cast xs:nonNegativeInteger to xs:double.
    assert(cast_integer_to_double(3).to_bits() == 4613937818241073152);
}

#[test]
fn k2_seqexprcast_338() {
    //  Cast xs:nonPositiveInteger to xs:double.
    assert(cast_integer_to_double(0) == XsdDouble::zero());
}

#[test]
fn k2_seqexprcast_339() {
    //  Cast xs:short to xs:double.
    assert(cast_integer_to_double(3).to_bits() == 4613937818241073152);
}

#[test]
fn k2_seqexprcast_340() {
    //  Cast xs:unsignedByte to xs:double.
    assert(cast_integer_to_double(3).to_bits() == 4613937818241073152);
}

#[test]
fn k2_seqexprcast_341() {
    //  Cast xs:unsignedInt to xs:double.
    assert(cast_integer_to_double(3).to_bits() == 4613937818241073152);
}

#[test]
fn k2_seqexprcast_342() {
    //  Cast xs:unsignedInt to xs:double.
    assert(cast_integer_to_double(3).to_bits() == 4613937818241073152);
}

#[test]
fn k2_seqexprcast_343() {
    //  Cast xs:unsignedLong to xs:double.
    assert(cast_integer_to_double(3).to_bits() == 4613937818241073152);
}

#[test]
fn k2_seqexprcast_344() {
    //  Cast xs:unsignedShort to xs:double.
    assert(cast_integer_to_double(3).to_bits() == 4613937818241073152);
}
