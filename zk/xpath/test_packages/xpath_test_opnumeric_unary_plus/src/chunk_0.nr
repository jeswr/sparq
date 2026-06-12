//! Test chunk 0 for op:numeric-unary-plus
//! Contains 33 tests

use dep::xpath::numeric_unary_plus_int;

#[test]
fn op_numeric_unary_plusint1args_1() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(-2147483648) == -2147483648);
}

#[test]
fn op_numeric_unary_plusint1args_2() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(-1873914410) == -1873914410);
}

#[test]
fn op_numeric_unary_plusint1args_3() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(2147483647) == 2147483647);
}

#[test]
fn op_numeric_unary_plusintg1args_1() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn op_numeric_unary_plusintg1args_2() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(830993497117024304) == 830993497117024304);
}

#[test]
fn op_numeric_unary_plusintg1args_3() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(999999999999999999) == 999999999999999999);
}

#[test]
fn op_numeric_unary_plusdec1args_1() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn op_numeric_unary_plusdec1args_2() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(617375191608514839) == 617375191608514839);
}

#[test]
fn op_numeric_unary_plusdec1args_3() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(999999999999999999) == 999999999999999999);
}

#[test]
fn op_numeric_unary_pluslng1args_1() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(-92233720368547758) == -92233720368547758);
}

#[test]
fn op_numeric_unary_pluslng1args_2() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(-47175562203048468) == -47175562203048468);
}

#[test]
fn op_numeric_unary_pluslng1args_3() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(92233720368547758) == 92233720368547758);
}

#[test]
fn op_numeric_unary_plususht1args_1() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(0) == 0);
}

#[test]
fn op_numeric_unary_plususht1args_2() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(44633) == 44633);
}

#[test]
fn op_numeric_unary_plususht1args_3() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(65535) == 65535);
}

#[test]
fn op_numeric_unary_plusnint1args_1() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn op_numeric_unary_plusnint1args_2() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(-297014075999096793) == -297014075999096793);
}

#[test]
fn op_numeric_unary_plusnint1args_3() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(-1) == -1);
}

#[test]
fn op_numeric_unary_pluspint1args_1() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(1) == 1);
}

#[test]
fn op_numeric_unary_pluspint1args_2() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(52704602390610033) == 52704602390610033);
}

#[test]
fn op_numeric_unary_pluspint1args_3() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(999999999999999999) == 999999999999999999);
}

#[test]
fn op_numeric_unary_plusulng1args_1() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(0) == 0);
}

#[test]
fn op_numeric_unary_plusulng1args_2() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(130747108607674654) == 130747108607674654);
}

#[test]
fn op_numeric_unary_plusulng1args_3() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(184467440737095516) == 184467440737095516);
}

#[test]
fn op_numeric_unary_plusnpi1args_1() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(-999999999999999999) == -999999999999999999);
}

#[test]
fn op_numeric_unary_plusnpi1args_2() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(-475688437271870490) == -475688437271870490);
}

#[test]
fn op_numeric_unary_plusnpi1args_3() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(0) == 0);
}

#[test]
fn op_numeric_unary_plusnni1args_1() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(0) == 0);
}

#[test]
fn op_numeric_unary_plusnni1args_2() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(303884545991464527) == 303884545991464527);
}

#[test]
fn op_numeric_unary_plusnni1args_3() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(999999999999999999) == 999999999999999999);
}

#[test]
fn op_numeric_unary_plussht1args_1() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(-32768) == -32768);
}

#[test]
fn op_numeric_unary_plussht1args_2() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(-5324) == -5324);
}

#[test]
fn op_numeric_unary_plussht1args_3() {
    //  Evaluates The 'op:numeric-unary-plus' operator with the arguments set as
    assert(numeric_unary_plus_int(32767) == 32767);
}
