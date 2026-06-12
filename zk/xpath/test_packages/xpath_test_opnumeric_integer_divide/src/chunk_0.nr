//! Test chunk 0 for op:numeric-integer-divide
//! Contains 47 tests

use dep::xpath::numeric_divide_int;

#[test]
fn op_numeric_integer_divideint2args_1() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-2147483648, -2147483648) == 1);
}

#[test]
fn op_numeric_integer_divideint2args_2() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-1873914410, -2147483648) == 0);
}

#[test]
fn op_numeric_integer_divideint2args_3() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(2147483647, -2147483648) == 0);
}

#[test]
fn op_numeric_integer_divideint2args_4() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-2147483648, -1873914410) == 1);
}

#[test]
fn op_numeric_integer_divideint2args_5() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-2147483648, 2147483647) == -1);
}

#[test]
fn op_numeric_integer_divideintg2args_1() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-999999999999999999, -999999999999999999) == 1);
}

#[test]
fn op_numeric_integer_divideintg2args_2() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(830993497117024304, -999999999999999999) == 0);
}

#[test]
fn op_numeric_integer_divideintg2args_3() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(999999999999999999, -999999999999999999) == -1);
}

#[test]
fn op_numeric_integer_divideintg2args_4() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-999999999999999999, 830993497117024304) == -1);
}

#[test]
fn op_numeric_integer_divideintg2args_5() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-999999999999999999, 999999999999999999) == -1);
}

#[test]
fn op_numeric_integer_dividedec2args_1() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-999999999999999999, -999999999999999999) == 1);
}

#[test]
fn op_numeric_integer_dividedec2args_2() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(617375191608514839, -999999999999999999) == 0);
}

#[test]
fn op_numeric_integer_dividedec2args_3() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(999999999999999999, -999999999999999999) == -1);
}

#[test]
fn op_numeric_integer_dividedec2args_4() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-999999999999999999, 617375191608514839) == -1);
}

#[test]
fn op_numeric_integer_dividedec2args_5() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-999999999999999999, 999999999999999999) == -1);
}

#[test]
fn op_numeric_integer_dividelng2args_1() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-92233720368547758, -92233720368547758) == 1);
}

#[test]
fn op_numeric_integer_dividelng2args_2() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-47175562203048468, -92233720368547758) == 0);
}

#[test]
fn op_numeric_integer_dividelng2args_3() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(92233720368547758, -92233720368547758) == -1);
}

#[test]
fn op_numeric_integer_dividelng2args_4() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-92233720368547758, -47175562203048468) == 1);
}

#[test]
fn op_numeric_integer_dividelng2args_5() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-92233720368547758, 92233720368547758) == -1);
}

#[test]
fn op_numeric_integer_divideusht2args_1() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(0, 44633) == 0);
}

#[test]
fn op_numeric_integer_divideusht2args_2() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(0, 65535) == 0);
}

#[test]
fn op_numeric_integer_dividenint2args_1() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-999999999999999999, -999999999999999999) == 1);
}

#[test]
fn op_numeric_integer_dividenint2args_2() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-297014075999096793, -999999999999999999) == 0);
}

#[test]
fn op_numeric_integer_dividenint2args_3() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-1, -999999999999999999) == 0);
}

#[test]
fn op_numeric_integer_dividenint2args_4() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-999999999999999999, -297014075999096793) == 3);
}

#[test]
fn op_numeric_integer_dividenint2args_5() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-999999999999999999, -1) == 999999999999999999);
}

#[test]
fn op_numeric_integer_dividepint2args_1() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(1, 1) == 1);
}

#[test]
fn op_numeric_integer_dividepint2args_2() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(52704602390610033, 1) == 52704602390610033);
}

#[test]
fn op_numeric_integer_dividepint2args_3() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(999999999999999999, 1) == 999999999999999999);
}

#[test]
fn op_numeric_integer_dividepint2args_4() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(1, 52704602390610033) == 0);
}

#[test]
fn op_numeric_integer_dividepint2args_5() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(1, 999999999999999999) == 0);
}

#[test]
fn op_numeric_integer_divideulng2args_1() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(0, 130747108607674654) == 0);
}

#[test]
fn op_numeric_integer_divideulng2args_2() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(0, 184467440737095516) == 0);
}

#[test]
fn op_numeric_integer_dividenpi2args_1() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-999999999999999999, -999999999999999999) == 1);
}

#[test]
fn op_numeric_integer_dividenpi2args_2() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-475688437271870490, -999999999999999999) == 0);
}

#[test]
fn op_numeric_integer_dividenpi2args_3() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(0, -999999999999999999) == 0);
}

#[test]
fn op_numeric_integer_dividenpi2args_4() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-999999999999999999, -475688437271870490) == 2);
}

#[test]
fn op_numeric_integer_dividenni2args_1() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(0, 303884545991464527) == 0);
}

#[test]
fn op_numeric_integer_dividenni2args_2() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(0, 999999999999999999) == 0);
}

#[test]
fn op_numeric_integer_dividesht2args_1() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-32768, -32768) == 1);
}

#[test]
fn op_numeric_integer_dividesht2args_2() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-5324, -32768) == 0);
}

#[test]
fn op_numeric_integer_dividesht2args_3() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(32767, -32768) == 0);
}

#[test]
fn op_numeric_integer_dividesht2args_4() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-32768, -5324) == 6);
}

#[test]
fn op_numeric_integer_dividesht2args_5() {
    //  Evaluates The 'op:numeric-integer-divide' operator with the arguments set as
    assert(numeric_divide_int(-32768, 32767) == -1);
}

#[test]
fn op_numeric_integer_dividemix2args_3() {
    //  Simple integer division test, second operator cast string to integer
    assert(numeric_divide_int(1, 1) == 1);
}

#[test]
fn cbcl_numeric_idivide_005() {
    //  Test idiv by one on an xs:integer
    assert(numeric_divide_int(2, 1) == 2);
}
