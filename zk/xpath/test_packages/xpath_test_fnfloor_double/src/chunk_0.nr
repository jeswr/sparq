//! Test chunk 0 for fn:floor-double
//! Contains 3 tests

use dep::xpath::{floor_double, XsdDouble};

#[test]
fn fn_floordbl1args_1() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    let val = XsdDouble::from_bits(18442240474082181119);
    assert(floor_double(val).to_bits() == 18442240474082181119);
}

#[test]
fn fn_floordbl1args_2() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    let val = XsdDouble::from_bits(0);
    assert(floor_double(val) == XsdDouble::zero());
}

#[test]
fn fn_floordbl1args_3() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    let val = XsdDouble::from_bits(9218868437227405311);
    assert(floor_double(val).to_bits() == 9218868437227405311);
}
