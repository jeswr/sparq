//! Test chunk 0 for op:numeric-less-than-float
//! Contains 8 tests

use dep::xpath::{numeric_less_than_float, XsdFloat};

#[test]
fn op_numeric_less_thanflt2args_1() {
    //  ******************************************************* Test:
    let a = XsdFloat::from_bits(4286578687);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_less_than_float(a, b) == false);
}

#[test]
fn op_numeric_less_thanflt2args_2() {
    //  ******************************************************* Test:
    let a = XsdFloat::from_bits(0);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_less_than_float(a, b) == false);
}

#[test]
fn op_numeric_less_thanflt2args_3() {
    //  ******************************************************* Test:
    let a = XsdFloat::from_bits(2139095039);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_less_than_float(a, b) == false);
}

#[test]
fn op_numeric_less_thanflt2args_4() {
    //  ******************************************************* Test:
    let a = XsdFloat::from_bits(4286578687);
    let b = XsdFloat::from_bits(0);
    assert(numeric_less_than_float(a, b) == true);
}

#[test]
fn op_numeric_less_thanflt2args_5() {
    //  ******************************************************* Test:
    let a = XsdFloat::from_bits(4286578687);
    let b = XsdFloat::from_bits(2139095039);
    assert(numeric_less_than_float(a, b) == true);
}

#[test]
fn k_numericlt_9() {
    //  *******************************************************  Test: K-NumericLT-9
    let a = XsdFloat::from_bits(0);
    let b = XsdFloat::from_bits(2139095040);
    assert(numeric_less_than_float(a, b) == true);
}

#[test]
fn k_numericlt_10() {
    //  *******************************************************  Test: K-NumericLT-10
    let a = XsdFloat::from_bits(4286578688);
    let b = XsdFloat::from_bits(0);
    assert(numeric_less_than_float(a, b) == true);
}

#[test]
fn k_numericlt_11() {
    //  *******************************************************  Test: K-NumericLT-11
    let a = XsdFloat::from_bits(3212836864);
    let b = XsdFloat::from_bits(1065353216);
    assert(numeric_less_than_float(a, b) == true);
}
