//! Test chunk 0 for op:numeric-multiply-float
//! Contains 5 tests

use dep::xpath::{numeric_multiply_float, XsdFloat};

#[test]
fn op_numeric_multiplyflt2args_1() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    let a = XsdFloat::from_bits(3212836864);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_multiply_float(a, b).to_bits() == 2139095039);
}

#[test]
fn op_numeric_multiplyflt2args_2() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    let a = XsdFloat::from_bits(0);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_multiply_float(a, b) == XsdFloat::zero());
}

#[test]
fn op_numeric_multiplyflt2args_3() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    let a = XsdFloat::from_bits(2139095039);
    let b = XsdFloat::from_bits(3212836864);
    assert(numeric_multiply_float(a, b).to_bits() == 4286578687);
}

#[test]
fn op_numeric_multiplyflt2args_4() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    let a = XsdFloat::from_bits(4286578687);
    let b = XsdFloat::from_bits(0);
    assert(numeric_multiply_float(a, b) == XsdFloat::zero());
}

#[test]
fn op_numeric_multiplyflt2args_5() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    let a = XsdFloat::from_bits(3212836864);
    let b = XsdFloat::from_bits(2139095039);
    assert(numeric_multiply_float(a, b).to_bits() == 4286578687);
}
