//! Test chunk 0 for op:numeric-divide-float
//! Contains 4 tests

use dep::xpath::{numeric_divide_float, XsdFloat};

#[test]
fn op_numeric_divideflt2args_1() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    let a = XsdFloat::from_bits(4286578687);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_divide_float(a, b).to_bits() == 1065353216);
}

#[test]
fn op_numeric_divideflt2args_2() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    let a = XsdFloat::from_bits(0);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_divide_float(a, b) == XsdFloat::zero());
}

#[test]
fn op_numeric_divideflt2args_3() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    let a = XsdFloat::from_bits(2139095039);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_divide_float(a, b).to_bits() == 3212836864);
}

#[test]
fn op_numeric_divideflt2args_4() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    let a = XsdFloat::from_bits(4286578687);
    let b = XsdFloat::from_bits(2139095039);
    assert(numeric_divide_float(a, b).to_bits() == 3212836864);
}
