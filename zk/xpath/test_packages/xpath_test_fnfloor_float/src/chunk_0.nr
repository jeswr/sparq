//! Test chunk 0 for fn:floor-float
//! Contains 3 tests

use dep::xpath::{floor_float, XsdFloat};

#[test]
fn fn_floorflt1args_1() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    let val = XsdFloat::from_bits(4286578687);
    assert(floor_float(val).to_bits() == 4286578687);
}

#[test]
fn fn_floorflt1args_2() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    let val = XsdFloat::from_bits(0);
    assert(floor_float(val) == XsdFloat::zero());
}

#[test]
fn fn_floorflt1args_3() {
    //  Evaluates The 'floor' function with the arguments set as follows: $arg =
    let val = XsdFloat::from_bits(2139095039);
    assert(floor_float(val).to_bits() == 2139095039);
}
