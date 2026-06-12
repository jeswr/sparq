//! Test chunk 0 for op:numeric-divide
//! Contains 25 tests

use dep::xpath::numeric_divide_int;

#[test]
fn op_numeric_divideint2args_1() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(-2147483648, -2147483648) == 1);
}

#[test]
fn op_numeric_divideintg2args_1() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(-999999999999999999, -999999999999999999) == 1);
}

#[test]
fn op_numeric_divideintg2args_3() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(999999999999999999, -999999999999999999) == -1);
}

#[test]
fn op_numeric_divideintg2args_5() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(-999999999999999999, 999999999999999999) == -1);
}

#[test]
fn op_numeric_dividedec2args_1() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(-999999999999999999, -999999999999999999) == 1);
}

#[test]
fn op_numeric_dividedec2args_3() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(999999999999999999, -999999999999999999) == -1);
}

#[test]
fn op_numeric_dividedec2args_5() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(-999999999999999999, 999999999999999999) == -1);
}

#[test]
fn op_numeric_dividelng2args_1() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(-92233720368547758, -92233720368547758) == 1);
}

#[test]
fn op_numeric_dividelng2args_3() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(92233720368547758, -92233720368547758) == -1);
}

#[test]
fn op_numeric_dividelng2args_5() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(-92233720368547758, 92233720368547758) == -1);
}

#[test]
fn op_numeric_divideusht2args_1() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(0, 44633) == 0);
}

#[test]
fn op_numeric_divideusht2args_2() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(0, 65535) == 0);
}

#[test]
fn op_numeric_dividenint2args_1() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(-999999999999999999, -999999999999999999) == 1);
}

#[test]
fn op_numeric_dividenint2args_5() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(-999999999999999999, -1) == 999999999999999999);
}

#[test]
fn op_numeric_dividepint2args_1() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(1, 1) == 1);
}

#[test]
fn op_numeric_dividepint2args_2() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(52704602390610033, 1) == 52704602390610033);
}

#[test]
fn op_numeric_dividepint2args_3() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(999999999999999999, 1) == 999999999999999999);
}

#[test]
fn op_numeric_divideulng2args_1() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(0, 130747108607674654) == 0);
}

#[test]
fn op_numeric_divideulng2args_2() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(0, 184467440737095516) == 0);
}

#[test]
fn op_numeric_dividenpi2args_1() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(-999999999999999999, -999999999999999999) == 1);
}

#[test]
fn op_numeric_dividenpi2args_3() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(0, -999999999999999999) == 0);
}

#[test]
fn op_numeric_dividenni2args_1() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(0, 303884545991464527) == 0);
}

#[test]
fn op_numeric_dividenni2args_2() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(0, 999999999999999999) == 0);
}

#[test]
fn op_numeric_dividesht2args_1() {
    //  Evaluates The 'op:numeric-divide' operator with the arguments set as follows:
    assert(numeric_divide_int(-32768, -32768) == 1);
}

#[test]
fn op_numeric_dividemix2args_3() {
    //  Simple division test, second operator cast string to integer
    assert(numeric_divide_int(1, 1) == 1);
}
