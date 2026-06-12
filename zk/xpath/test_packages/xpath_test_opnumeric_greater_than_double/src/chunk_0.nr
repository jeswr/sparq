//! Test chunk 0 for op:numeric-greater-than-double
//! Contains 9 tests

use dep::xpath::{numeric_greater_than_double, XsdDouble};

#[test]
fn op_numeric_greater_thandbl2args_1() {
    //  ******************************************************* Test:
    let a = XsdDouble::from_bits(18442240474082181119);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_greater_than_double(a, b) == false);
}

#[test]
fn op_numeric_greater_thandbl2args_2() {
    //  ******************************************************* Test:
    let a = XsdDouble::from_bits(0);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_greater_than_double(a, b) == true);
}

#[test]
fn op_numeric_greater_thandbl2args_3() {
    //  ******************************************************* Test:
    let a = XsdDouble::from_bits(9218868437227405311);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_greater_than_double(a, b) == true);
}

#[test]
fn op_numeric_greater_thandbl2args_4() {
    //  ******************************************************* Test:
    let a = XsdDouble::from_bits(18442240474082181119);
    let b = XsdDouble::from_bits(0);
    assert(numeric_greater_than_double(a, b) == false);
}

#[test]
fn op_numeric_greater_thandbl2args_5() {
    //  ******************************************************* Test:
    let a = XsdDouble::from_bits(18442240474082181119);
    let b = XsdDouble::from_bits(9218868437227405311);
    assert(numeric_greater_than_double(a, b) == false);
}

#[test]
fn k_numericgt_5() {
    //  *******************************************************  Test: K-NumericGT-5
    let a = XsdDouble::from_bits(0);
    let b = XsdDouble::from_bits(18442240474082181120);
    assert(numeric_greater_than_double(a, b) == true);
}

#[test]
fn k_numericgt_6() {
    //  *******************************************************  Test: K-NumericGT-6
    let a = XsdDouble::from_bits(9218868437227405312);
    let b = XsdDouble::from_bits(0);
    assert(numeric_greater_than_double(a, b) == true);
}

#[test]
fn k_numericgt_7() {
    //  *******************************************************  Test: K-NumericGT-7
    let a = XsdDouble::from_bits(4607182418800017408);
    let b = XsdDouble::from_bits(13830554455654793216);
    assert(numeric_greater_than_double(a, b) == true);
}

#[test]
fn k2_numericgt_3() {
    // compare two doubles, where the second value is NaN
    let a = XsdDouble::from_bits(4613937818241073152);
    let b = XsdDouble::from_bits(9221120237041090560);
    assert(numeric_greater_than_double(a, b) == false);
}
