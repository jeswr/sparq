//! Test chunk 0 for fn:ceiling-float
//! Contains 3 tests

use dep::xpath::{ceil_float, XsdFloat};

#[test]
fn fn_ceilingflt1args_1() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    let val = XsdFloat::from_bits(4286578687);
    assert(ceil_float(val).to_bits() == 4286578687);
}

#[test]
fn fn_ceilingflt1args_2() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    let val = XsdFloat::from_bits(0);
    assert(ceil_float(val) == XsdFloat::zero());
}

#[test]
fn fn_ceilingflt1args_3() {
    //  Evaluates The 'ceiling' function with the arguments set as follows: $arg =
    let val = XsdFloat::from_bits(2139095039);
    assert(ceil_float(val).to_bits() == 2139095039);
}
