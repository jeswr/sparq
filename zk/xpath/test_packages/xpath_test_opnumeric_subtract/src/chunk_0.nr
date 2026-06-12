//! Test chunk 0 for op:numeric-subtract
//! Contains 50 tests

use dep::xpath::numeric_subtract_int;

#[test]
fn op_numeric_subtractint2args_1() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-2147483648, -2147483648) == 0);
}

#[test]
fn op_numeric_subtractint2args_2() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-1873914410, -2147483648) == 273569238);
}

#[test]
fn op_numeric_subtractint2args_3() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-1, -2147483648) == 2147483647);
}

#[test]
fn op_numeric_subtractint2args_4() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-2147483648, -1873914410) == -273569238);
}

#[test]
fn op_numeric_subtractint2args_5() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-1, 2147483647) == -2147483648);
}

#[test]
fn op_numeric_subtractintg2args_1() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-999999999999999999, -999999999999999999) == 0);
}

#[test]
fn op_numeric_subtractintg2args_2() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(0, -999999999999999999) == 999999999999999999);
}

#[test]
fn op_numeric_subtractintg2args_3() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(0, -999999999999999999) == 999999999999999999);
}

#[test]
fn op_numeric_subtractintg2args_4() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-169006502882975695, 830993497117024304) == -999999999999999999);
}

#[test]
fn op_numeric_subtractintg2args_5() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(0, 999999999999999999) == -999999999999999999);
}

#[test]
fn op_numeric_subtractdec2args_1() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-999999999999999999, -999999999999999999) == 0);
}

#[test]
fn op_numeric_subtractdec2args_2() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(0, -999999999999999999) == 999999999999999999);
}

#[test]
fn op_numeric_subtractdec2args_3() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(0, -999999999999999999) == 999999999999999999);
}

#[test]
fn op_numeric_subtractdec2args_4() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-382624808391485160, 617375191608514839) == -999999999999999999);
}

#[test]
fn op_numeric_subtractdec2args_5() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(0, 999999999999999999) == -999999999999999999);
}

#[test]
fn op_numeric_subtractlng2args_1() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-92233720368547758, -92233720368547758) == 0);
}

#[test]
fn op_numeric_subtractlng2args_2() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-47175562203048468, -92233720368547758) == 45058158165499290);
}

#[test]
fn op_numeric_subtractlng2args_3() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(0, -92233720368547758) == 92233720368547758);
}

#[test]
fn op_numeric_subtractlng2args_4() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-92233720368547758, -47175562203048468) == -45058158165499290);
}

#[test]
fn op_numeric_subtractlng2args_5() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(0, 92233720368547758) == -92233720368547758);
}

#[test]
fn op_numeric_subtractusht2args_1() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(0, 0) == 0);
}

#[test]
fn op_numeric_subtractusht2args_2() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(44633, 0) == 44633);
}

#[test]
fn op_numeric_subtractusht2args_3() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(65535, 0) == 65535);
}

#[test]
fn op_numeric_subtractusht2args_4() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(44633, 44633) == 0);
}

#[test]
fn op_numeric_subtractusht2args_5() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(65535, 65535) == 0);
}

#[test]
fn op_numeric_subtractnint2args_1() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-999999999999999999, -297014075999096793) == -702985924000903206);
}

#[test]
fn op_numeric_subtractnint2args_2() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-999999999999999999, -1) == -999999999999999998);
}

#[test]
fn op_numeric_subtractpint2args_1() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(52704602390610033, 1) == 52704602390610032);
}

#[test]
fn op_numeric_subtractpint2args_2() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(999999999999999999, 1) == 999999999999999998);
}

#[test]
fn op_numeric_subtractpint2args_3() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(52704602390610034, 52704602390610033) == 1);
}

#[test]
fn op_numeric_subtractulng2args_1() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(0, 0) == 0);
}

#[test]
fn op_numeric_subtractulng2args_2() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(130747108607674654, 0) == 130747108607674654);
}

#[test]
fn op_numeric_subtractulng2args_3() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(184467440737095516, 0) == 184467440737095516);
}

#[test]
fn op_numeric_subtractulng2args_4() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(130747108607674654, 130747108607674654) == 0);
}

#[test]
fn op_numeric_subtractulng2args_5() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(184467440737095516, 184467440737095516) == 0);
}

#[test]
fn op_numeric_subtractnpi2args_1() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-999999999999999999, -999999999999999999) == 0);
}

#[test]
fn op_numeric_subtractnpi2args_2() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-999999999999999999, -999999999999999999) == 0);
}

#[test]
fn op_numeric_subtractnpi2args_3() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-999999999999999999, -999999999999999999) == 0);
}

#[test]
fn op_numeric_subtractnpi2args_4() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-999999999999999999, -475688437271870490) == -524311562728129509);
}

#[test]
fn op_numeric_subtractnpi2args_5() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-999999999999999999, 0) == -999999999999999999);
}

#[test]
fn op_numeric_subtractnni2args_1() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(0, 0) == 0);
}

#[test]
fn op_numeric_subtractnni2args_2() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(303884545991464527, 0) == 303884545991464527);
}

#[test]
fn op_numeric_subtractnni2args_3() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(999999999999999999, 0) == 999999999999999999);
}

#[test]
fn op_numeric_subtractnni2args_4() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(303884545991464527, 303884545991464527) == 0);
}

#[test]
fn op_numeric_subtractnni2args_5() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(999999999999999999, 999999999999999999) == 0);
}

#[test]
fn op_numeric_subtractsht2args_1() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-32768, -32768) == 0);
}

#[test]
fn op_numeric_subtractsht2args_2() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-5324, -32768) == 27444);
}

#[test]
fn op_numeric_subtractsht2args_3() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-1, -32768) == 32767);
}

#[test]
fn op_numeric_subtractsht2args_4() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-32768, -5324) == -27444);
}

#[test]
fn op_numeric_subtractsht2args_5() {
    //  Evaluates The 'op:numeric-subtract' operator with the arguments set as
    assert(numeric_subtract_int(-1, 32767) == -32768);
}
