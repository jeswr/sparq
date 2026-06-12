//! Test chunk 0 for op:numeric-multiply
//! Contains 28 tests

use dep::xpath::numeric_multiply_int;

#[test]
fn op_numeric_multiplydec2args_1() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(-1, -999999999999999999) == 999999999999999999);
}

#[test]
fn op_numeric_multiplydec2args_2() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(617375191608514839, 0) == 0);
}

#[test]
fn op_numeric_multiplydec2args_3() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(999999999999999999, -1) == -999999999999999999);
}

#[test]
fn op_numeric_multiplydec2args_4() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(0, 617375191608514839) == 0);
}

#[test]
fn op_numeric_multiplydec2args_5() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(-1, 999999999999999999) == -999999999999999999);
}

#[test]
fn op_numeric_multiplyusht2args_1() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(0, 0) == 0);
}

#[test]
fn op_numeric_multiplyusht2args_2() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(44633, 0) == 0);
}

#[test]
fn op_numeric_multiplyusht2args_3() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(65535, 0) == 0);
}

#[test]
fn op_numeric_multiplyusht2args_4() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(0, 44633) == 0);
}

#[test]
fn op_numeric_multiplyusht2args_5() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(0, 65535) == 0);
}

#[test]
fn op_numeric_multiplypint2args_1() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(1, 1) == 1);
}

#[test]
fn op_numeric_multiplypint2args_2() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(52704602390610033, 1) == 52704602390610033);
}

#[test]
fn op_numeric_multiplypint2args_3() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(999999999999999999, 1) == 999999999999999999);
}

#[test]
fn op_numeric_multiplypint2args_4() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(1, 52704602390610033) == 52704602390610033);
}

#[test]
fn op_numeric_multiplypint2args_5() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(1, 999999999999999999) == 999999999999999999);
}

#[test]
fn op_numeric_multiplyulng2args_1() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(0, 0) == 0);
}

#[test]
fn op_numeric_multiplyulng2args_2() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(130747108607674654, 0) == 0);
}

#[test]
fn op_numeric_multiplyulng2args_3() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(184467440737095516, 0) == 0);
}

#[test]
fn op_numeric_multiplyulng2args_4() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(0, 130747108607674654) == 0);
}

#[test]
fn op_numeric_multiplyulng2args_5() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(0, 184467440737095516) == 0);
}

#[test]
fn op_numeric_multiplynpi2args_1() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(0, -999999999999999999) == 0);
}

#[test]
fn op_numeric_multiplynpi2args_2() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(-999999999999999999, 0) == 0);
}

#[test]
fn op_numeric_multiplynni2args_1() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(0, 0) == 0);
}

#[test]
fn op_numeric_multiplynni2args_2() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(303884545991464527, 0) == 0);
}

#[test]
fn op_numeric_multiplynni2args_3() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(999999999999999999, 0) == 0);
}

#[test]
fn op_numeric_multiplynni2args_4() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(0, 303884545991464527) == 0);
}

#[test]
fn op_numeric_multiplynni2args_5() {
    //  Evaluates The 'op:numeric-multiply' operator with the arguments set as
    assert(numeric_multiply_int(0, 999999999999999999) == 0);
}

#[test]
fn op_numeric_multiplymix2args_3() {
    //  Simple multiplication test, second operator cast string to integer
    assert(numeric_multiply_int(1, 1) == 1);
}
