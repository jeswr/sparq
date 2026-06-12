//! Test chunk 0 for op:numeric-equal-double
//! Contains 9 tests

use dep::xpath::{numeric_equal_double, XsdDouble};

#[test]
fn op_numeric_equaldbl2args_1() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    let a = XsdDouble::from_bits(18442240474082181119);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_equal_double(a, b) == true);
}

#[test]
fn op_numeric_equaldbl2args_2() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    let a = XsdDouble::from_bits(0);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_equal_double(a, b) == false);
}

#[test]
fn op_numeric_equaldbl2args_3() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    let a = XsdDouble::from_bits(9218868437227405311);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_equal_double(a, b) == false);
}

#[test]
fn op_numeric_equaldbl2args_4() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    let a = XsdDouble::from_bits(18442240474082181119);
    let b = XsdDouble::from_bits(0);
    assert(numeric_equal_double(a, b) == false);
}

#[test]
fn op_numeric_equaldbl2args_5() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    let a = XsdDouble::from_bits(18442240474082181119);
    let b = XsdDouble::from_bits(9218868437227405311);
    assert(numeric_equal_double(a, b) == false);
}

#[test]
fn k_numericequal_12() {
    //  A test whose essence is: `xs:double(1.1) eq xs:double(1.1)`.
    let a = XsdDouble::from_bits(4607632778762754458);
    let b = XsdDouble::from_bits(4607632778762754458);
    assert(numeric_equal_double(a, b) == true);
}

#[test]
fn k_numericequal_13() {
    //  A test whose essence is: `xs:double(-1.1) eq xs:double(-1.1)`.
    let a = XsdDouble::from_bits(13831004815617530266);
    let b = XsdDouble::from_bits(13831004815617530266);
    assert(numeric_equal_double(a, b) == true);
}

#[test]
fn k_numericequal_19() {
    //  A test whose essence is: `xs:double('-INF') eq xs:double('-INF')`.
    let a = XsdDouble::from_bits(18442240474082181120);
    let b = XsdDouble::from_bits(18442240474082181120);
    assert(numeric_equal_double(a, b) == true);
}

#[test]
fn k_numericequal_20() {
    //  A test whose essence is: `xs:double('INF') eq xs:double('INF')`.
    let a = XsdDouble::from_bits(9218868437227405312);
    let b = XsdDouble::from_bits(9218868437227405312);
    assert(numeric_equal_double(a, b) == true);
}
