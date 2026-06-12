//! Test chunk 0 for op:numeric-multiply-double
//! Contains 5 tests

use dep::xpath::{numeric_multiply_double, XsdDouble};

#[test]
fn op_numeric_multiplydbl2args_1() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    let a = XsdDouble::from_bits(13830554455654793216);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_multiply_double(a, b).to_bits() == 9218868437227405311);
}

#[test]
fn op_numeric_multiplydbl2args_2() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    let a = XsdDouble::from_bits(0);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_multiply_double(a, b) == XsdDouble::zero());
}

#[test]
fn op_numeric_multiplydbl2args_3() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    let a = XsdDouble::from_bits(9218868437227405311);
    let b = XsdDouble::from_bits(13830554455654793216);
    assert(numeric_multiply_double(a, b).to_bits() == 18442240474082181119);
}

#[test]
fn op_numeric_multiplydbl2args_4() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    let a = XsdDouble::from_bits(18442240474082181119);
    let b = XsdDouble::from_bits(0);
    assert(numeric_multiply_double(a, b) == XsdDouble::zero());
}

#[test]
fn op_numeric_multiplydbl2args_5() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    let a = XsdDouble::from_bits(13830554455654793216);
    let b = XsdDouble::from_bits(9218868437227405311);
    assert(numeric_multiply_double(a, b).to_bits() == 18442240474082181119);
}
