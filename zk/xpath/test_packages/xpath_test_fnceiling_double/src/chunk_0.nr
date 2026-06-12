//! Test chunk 0 for fn:ceiling-double
//! Contains 3 tests

use dep::xpath::{ceil_double, XsdDouble};

#[test]
fn fn_ceilingdbl1args_1() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    let val = XsdDouble::from_bits(18442240474082181119);
    assert(ceil_double(val).to_bits() == 18442240474082181119);
}

#[test]
fn fn_ceilingdbl1args_2() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    let val = XsdDouble::from_bits(0);
    assert(ceil_double(val) == XsdDouble::zero());
}

#[test]
fn fn_ceilingdbl1args_3() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    let val = XsdDouble::from_bits(9218868437227405311);
    assert(ceil_double(val).to_bits() == 9218868437227405311);
}
