//! Test chunk 0 for op:numeric-subtract-double
//! Contains 6 tests

use dep::xpath::{numeric_subtract_double, XsdDouble};

#[test]
fn op_numeric_subtractdbl2args_1() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    let a = XsdDouble::from_bits(18442240474082181119);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_subtract_double(a, b) == XsdDouble::zero());
}

#[test]
fn op_numeric_subtractdbl2args_2() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    let a = XsdDouble::from_bits(0);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_subtract_double(a, b).to_bits() == 9218868437227405311);
}

#[test]
fn op_numeric_subtractdbl2args_3() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    let a = XsdDouble::from_bits(0);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_subtract_double(a, b).to_bits() == 9218868437227405311);
}

#[test]
fn op_numeric_subtractdbl2args_4() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    let a = XsdDouble::from_bits(18442240474082181119);
    let b = XsdDouble::from_bits(0);
    assert(numeric_subtract_double(a, b).to_bits() == 18442240474082181119);
}

#[test]
fn op_numeric_subtractdbl2args_5() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    let a = XsdDouble::from_bits(0);
    let b = XsdDouble::from_bits(9218868437227405311);
    assert(numeric_subtract_double(a, b).to_bits() == 18442240474082181119);
}

#[test]
fn k2_numericsubtract_1() {
    //  Substract xs:double(0) from xs:integer(0).
    let a = XsdDouble::from_bits(0);
    let b = XsdDouble::from_bits(0);
    assert(numeric_subtract_double(a, b) == XsdDouble::zero());
}
