//! Test chunk 0 for op:numeric-unary-minus
//! Contains 35 tests

use dep::xpath::numeric_unary_minus_int;

#[test]
fn op_numeric_unary_minusint1args_1() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(-2147483647) == 2147483647);
}

#[test]
fn op_numeric_unary_minusint1args_2() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(-1873914410) == 1873914410);
}

#[test]
fn op_numeric_unary_minusint1args_3() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(2147483647) == -2147483647);
}

#[test]
fn op_numeric_unary_minusintg1args_1() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(-999999999999999999) == 999999999999999999);
}

#[test]
fn op_numeric_unary_minusintg1args_2() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(830993497117024304) == -830993497117024304);
}

#[test]
fn op_numeric_unary_minusintg1args_3() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(999999999999999999) == -999999999999999999);
}

#[test]
fn op_numeric_unary_minusdec1args_1() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(-999999999999999999) == 999999999999999999);
}

#[test]
fn op_numeric_unary_minusdec1args_2() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(617375191608514839) == -617375191608514839);
}

#[test]
fn op_numeric_unary_minusdec1args_3() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(999999999999999999) == -999999999999999999);
}

#[test]
fn op_numeric_unary_minuslng1args_1() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(-92233720368547758) == 92233720368547758);
}

#[test]
fn op_numeric_unary_minuslng1args_2() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(-47175562203048468) == 47175562203048468);
}

#[test]
fn op_numeric_unary_minuslng1args_3() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(92233720368547758) == -92233720368547758);
}

#[test]
fn op_numeric_unary_minususht1args_1() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(0) == 0);
}

#[test]
fn op_numeric_unary_minususht1args_2() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(44633) == -44633);
}

#[test]
fn op_numeric_unary_minususht1args_3() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(65535) == -65535);
}

#[test]
fn op_numeric_unary_minusnint1args_1() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(-999999999999999999) == 999999999999999999);
}

#[test]
fn op_numeric_unary_minusnint1args_2() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(-297014075999096793) == 297014075999096793);
}

#[test]
fn op_numeric_unary_minusnint1args_3() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(-1) == 1);
}

#[test]
fn op_numeric_unary_minuspint1args_1() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(1) == -1);
}

#[test]
fn op_numeric_unary_minuspint1args_2() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(52704602390610033) == -52704602390610033);
}

#[test]
fn op_numeric_unary_minuspint1args_3() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(999999999999999999) == -999999999999999999);
}

#[test]
fn op_numeric_unary_minusulng1args_1() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(0) == 0);
}

#[test]
fn op_numeric_unary_minusulng1args_2() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(130747108607674654) == -130747108607674654);
}

#[test]
fn op_numeric_unary_minusulng1args_3() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(184467440737095516) == -184467440737095516);
}

#[test]
fn op_numeric_unary_minusnpi1args_1() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(-999999999999999999) == 999999999999999999);
}

#[test]
fn op_numeric_unary_minusnpi1args_2() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(-475688437271870490) == 475688437271870490);
}

#[test]
fn op_numeric_unary_minusnpi1args_3() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(0) == 0);
}

#[test]
fn op_numeric_unary_minusnni1args_1() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(0) == 0);
}

#[test]
fn op_numeric_unary_minusnni1args_2() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(303884545991464527) == -303884545991464527);
}

#[test]
fn op_numeric_unary_minusnni1args_3() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(999999999999999999) == -999999999999999999);
}

#[test]
fn op_numeric_unary_minussht1args_1() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(-32768) == 32768);
}

#[test]
fn op_numeric_unary_minussht1args_2() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(-5324) == 5324);
}

#[test]
fn op_numeric_unary_minussht1args_3() {
    //  Evaluates The 'op:numeric-unary-minus' operator with the arguments set as
    assert(numeric_unary_minus_int(32767) == -32767);
}

#[test]
fn k2_numericunaryminus_3() {
    //  Operand is constructor function for xs:decimal.
    assert(numeric_unary_minus_int(0) == 0);
}

#[test]
fn k2_numericunaryminus_4() {
    //  Operand is constructor function for xs:integer.
    assert(numeric_unary_minus_int(0) == 0);
}
