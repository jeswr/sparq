//! Test chunk 0 for op:numeric-add-double
//! Contains 4 tests

use dep::xpath::{numeric_add_double, XsdDouble};

#[test]
fn op_numeric_adddbl2args_1() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    let a = XsdDouble::from_bits(0);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_add_double(a, b).to_bits() == 18442240474082181119);
}

#[test]
fn op_numeric_adddbl2args_2() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    let a = XsdDouble::from_bits(9218868437227405311);
    let b = XsdDouble::from_bits(18442240474082181119);
    assert(numeric_add_double(a, b) == XsdDouble::zero());
}

#[test]
fn op_numeric_adddbl2args_3() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    let a = XsdDouble::from_bits(18442240474082181119);
    let b = XsdDouble::from_bits(0);
    assert(numeric_add_double(a, b).to_bits() == 18442240474082181119);
}

#[test]
fn op_numeric_adddbl2args_4() {
    //  Evaluates The 'op:numeric-add' operator with the arguments set as follows:
    let a = XsdDouble::from_bits(18442240474082181119);
    let b = XsdDouble::from_bits(9218868437227405311);
    assert(numeric_add_double(a, b) == XsdDouble::zero());
}
