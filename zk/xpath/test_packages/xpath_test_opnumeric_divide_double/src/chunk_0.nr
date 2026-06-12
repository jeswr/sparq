//! Test chunk 0 for op:numeric-divide-double
//! Contains 4 tests

use dep::xpath::{numeric_divide_double, XsdDouble};

#[test]
fn op_numeric_dividedbl2args_1() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    let a = XsdDouble::from_bits(18442240474082181119);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_divide_double(a, b).to_bits() == 4607182418800017408);
}

#[test]
fn op_numeric_dividedbl2args_2() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    let a = XsdDouble::from_bits(0);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_divide_double(a, b) == XsdDouble::zero());
}

#[test]
fn op_numeric_dividedbl2args_3() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    let a = XsdDouble::from_bits(9218868437227405311);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_divide_double(a, b).to_bits() == 13830554455654793216);
}

#[test]
fn op_numeric_dividedbl2args_4() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    let a = XsdDouble::from_bits(18442240474082181119);
    let b = XsdDouble::from_bits(9218868437227405311);
    assert(numeric_divide_double(a, b).to_bits() == 13830554455654793216);
}
