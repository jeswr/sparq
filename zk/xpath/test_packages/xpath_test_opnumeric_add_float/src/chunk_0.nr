//! Test chunk 0 for op:numeric-add-float
//! Contains 4 tests

use dep::xpath::{numeric_add_float, XsdFloat};

#[test]
fn op_numeric_addflt2args_1() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    let a = XsdFloat::from_bits(0);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_add_float(a, b).to_bits() == 4286578687);
}

#[test]
fn op_numeric_addflt2args_2() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    let a = XsdFloat::from_bits(2139095039);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_add_float(a, b) == XsdFloat::zero());
}

#[test]
fn op_numeric_addflt2args_3() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    let a = XsdFloat::from_bits(4286578687);
    let b = XsdFloat::from_bits(0);
    assert(numeric_add_float(a, b).to_bits() == 4286578687);
}

#[test]
fn op_numeric_addflt2args_4() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    let a = XsdFloat::from_bits(4286578687);
    let b = XsdFloat::from_bits(2139095039);
    assert(numeric_add_float(a, b) == XsdFloat::zero());
}
