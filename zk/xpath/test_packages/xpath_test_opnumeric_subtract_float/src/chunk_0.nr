//! Test chunk 0 for op:numeric-subtract-float
//! Contains 6 tests

use dep::xpath::{numeric_subtract_float, XsdFloat};

#[test]
fn op_numeric_subtractflt2args_1() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    let a = XsdFloat::from_bits(4286578687);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_subtract_float(a, b) == XsdFloat::zero());
}

#[test]
fn op_numeric_subtractflt2args_2() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    let a = XsdFloat::from_bits(0);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_subtract_float(a, b).to_bits() == 2139095039);
}

#[test]
fn op_numeric_subtractflt2args_3() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    let a = XsdFloat::from_bits(0);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_subtract_float(a, b).to_bits() == 2139095039);
}

#[test]
fn op_numeric_subtractflt2args_4() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    let a = XsdFloat::from_bits(4286578687);
    let b = XsdFloat::from_bits(0);
    assert(numeric_subtract_float(a, b).to_bits() == 4286578687);
}

#[test]
fn op_numeric_subtractflt2args_5() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    let a = XsdFloat::from_bits(0);
    let b = XsdFloat::from_bits(2139095039);
    assert(numeric_subtract_float(a, b).to_bits() == 4286578687);
}

#[test]
fn k2_numericsubtract_2() {
    //  Substract xs:double(0) from xs:integer(0).
    let a = XsdFloat::from_bits(0);
    let b = XsdFloat::from_bits(0);
    assert(numeric_subtract_float(a, b) == XsdFloat::zero());
}
