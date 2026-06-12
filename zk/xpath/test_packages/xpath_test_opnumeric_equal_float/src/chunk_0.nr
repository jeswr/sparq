//! Test chunk 0 for op:numeric-equal-float
//! Contains 9 tests

use dep::xpath::{numeric_equal_float, XsdFloat};

#[test]
fn op_numeric_equalflt2args_1() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    let a = XsdFloat::from_bits(4286578687);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_equal_float(a, b) == true);
}

#[test]
fn op_numeric_equalflt2args_2() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    let a = XsdFloat::from_bits(0);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_equal_float(a, b) == false);
}

#[test]
fn op_numeric_equalflt2args_3() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    let a = XsdFloat::from_bits(2139095039);
    let b = XsdFloat::from_bits(4286578687);
    assert(numeric_equal_float(a, b) == false);
}

#[test]
fn op_numeric_equalflt2args_4() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    let a = XsdFloat::from_bits(4286578687);
    let b = XsdFloat::from_bits(0);
    assert(numeric_equal_float(a, b) == false);
}

#[test]
fn op_numeric_equalflt2args_5() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    let a = XsdFloat::from_bits(4286578687);
    let b = XsdFloat::from_bits(2139095039);
    assert(numeric_equal_float(a, b) == false);
}

#[test]
fn k_numericequal_27() {
    //  A test whose essence is: `xs:float(1.1) eq xs:float(1.1)`.
    let a = XsdFloat::from_bits(1066192077);
    let b = XsdFloat::from_bits(1066192077);
    assert(numeric_equal_float(a, b) == true);
}

#[test]
fn k_numericequal_28() {
    //  A test whose essence is: `xs:float(-1.1) eq xs:float(-1.1)`.
    let a = XsdFloat::from_bits(3213675725);
    let b = XsdFloat::from_bits(3213675725);
    assert(numeric_equal_float(a, b) == true);
}

#[test]
fn k_numericequal_33() {
    //  A test whose essence is: `xs:float('-INF') eq xs:float('-INF')`.
    let a = XsdFloat::from_bits(4286578688);
    let b = XsdFloat::from_bits(4286578688);
    assert(numeric_equal_float(a, b) == true);
}

#[test]
fn k_numericequal_34() {
    //  A test whose essence is: `xs:float('INF') eq xs:float('INF')`.
    let a = XsdFloat::from_bits(2139095040);
    let b = XsdFloat::from_bits(2139095040);
    assert(numeric_equal_float(a, b) == true);
}
