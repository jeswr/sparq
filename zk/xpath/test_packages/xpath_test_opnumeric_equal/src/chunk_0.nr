//! Test chunk 0 for op:numeric-equal
//! Contains 50 tests

use dep::xpath::numeric_equal_int;

#[test]
fn op_numeric_equalint2args_1() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-2147483648, -2147483648) == true);
}

#[test]
fn op_numeric_equalint2args_2() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-1873914410, -2147483648) == false);
}

#[test]
fn op_numeric_equalint2args_3() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(2147483647, -2147483648) == false);
}

#[test]
fn op_numeric_equalint2args_4() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-2147483648, -1873914410) == false);
}

#[test]
fn op_numeric_equalint2args_5() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-2147483648, 2147483647) == false);
}

#[test]
fn op_numeric_equalintg2args_1() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-999999999999999999, -999999999999999999) == true);
}

#[test]
fn op_numeric_equalintg2args_2() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(830993497117024304, -999999999999999999) == false);
}

#[test]
fn op_numeric_equalintg2args_3() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(999999999999999999, -999999999999999999) == false);
}

#[test]
fn op_numeric_equalintg2args_4() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-999999999999999999, 830993497117024304) == false);
}

#[test]
fn op_numeric_equalintg2args_5() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-999999999999999999, 999999999999999999) == false);
}

#[test]
fn op_numeric_equaldec2args_1() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-999999999999999999, -999999999999999999) == true);
}

#[test]
fn op_numeric_equaldec2args_2() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(617375191608514839, -999999999999999999) == false);
}

#[test]
fn op_numeric_equaldec2args_3() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(999999999999999999, -999999999999999999) == false);
}

#[test]
fn op_numeric_equaldec2args_4() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-999999999999999999, 617375191608514839) == false);
}

#[test]
fn op_numeric_equaldec2args_5() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-999999999999999999, 999999999999999999) == false);
}

#[test]
fn op_numeric_equallng2args_1() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-92233720368547758, -92233720368547758) == true);
}

#[test]
fn op_numeric_equallng2args_2() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-47175562203048468, -92233720368547758) == false);
}

#[test]
fn op_numeric_equallng2args_3() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(92233720368547758, -92233720368547758) == false);
}

#[test]
fn op_numeric_equallng2args_4() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-92233720368547758, -47175562203048468) == false);
}

#[test]
fn op_numeric_equallng2args_5() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-92233720368547758, 92233720368547758) == false);
}

#[test]
fn op_numeric_equalusht2args_1() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(0, 0) == true);
}

#[test]
fn op_numeric_equalusht2args_2() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(44633, 0) == false);
}

#[test]
fn op_numeric_equalusht2args_3() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(65535, 0) == false);
}

#[test]
fn op_numeric_equalusht2args_4() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(0, 44633) == false);
}

#[test]
fn op_numeric_equalusht2args_5() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(0, 65535) == false);
}

#[test]
fn op_numeric_equalnint2args_1() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-999999999999999999, -999999999999999999) == true);
}

#[test]
fn op_numeric_equalnint2args_2() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-297014075999096793, -999999999999999999) == false);
}

#[test]
fn op_numeric_equalnint2args_3() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-1, -999999999999999999) == false);
}

#[test]
fn op_numeric_equalnint2args_4() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-999999999999999999, -297014075999096793) == false);
}

#[test]
fn op_numeric_equalnint2args_5() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-999999999999999999, -1) == false);
}

#[test]
fn op_numeric_equalpint2args_1() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(1, 1) == true);
}

#[test]
fn op_numeric_equalpint2args_2() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(52704602390610033, 1) == false);
}

#[test]
fn op_numeric_equalpint2args_3() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(999999999999999999, 1) == false);
}

#[test]
fn op_numeric_equalpint2args_4() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(1, 52704602390610033) == false);
}

#[test]
fn op_numeric_equalpint2args_5() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(1, 999999999999999999) == false);
}

#[test]
fn op_numeric_equalulng2args_1() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(0, 0) == true);
}

#[test]
fn op_numeric_equalulng2args_2() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(130747108607674654, 0) == false);
}

#[test]
fn op_numeric_equalulng2args_3() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(184467440737095516, 0) == false);
}

#[test]
fn op_numeric_equalulng2args_4() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(0, 130747108607674654) == false);
}

#[test]
fn op_numeric_equalulng2args_5() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(0, 184467440737095516) == false);
}

#[test]
fn op_numeric_equalnpi2args_1() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-999999999999999999, -999999999999999999) == true);
}

#[test]
fn op_numeric_equalnpi2args_2() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-475688437271870490, -999999999999999999) == false);
}

#[test]
fn op_numeric_equalnpi2args_3() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(0, -999999999999999999) == false);
}

#[test]
fn op_numeric_equalnpi2args_4() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-999999999999999999, -475688437271870490) == false);
}

#[test]
fn op_numeric_equalnpi2args_5() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(-999999999999999999, 0) == false);
}

#[test]
fn op_numeric_equalnni2args_1() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(0, 0) == true);
}

#[test]
fn op_numeric_equalnni2args_2() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(303884545991464527, 0) == false);
}

#[test]
fn op_numeric_equalnni2args_3() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(999999999999999999, 0) == false);
}

#[test]
fn op_numeric_equalnni2args_4() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(0, 303884545991464527) == false);
}

#[test]
fn op_numeric_equalnni2args_5() {
    //  Evaluates The 'op:numeric-equal' operator with the arguments set as follows:
    assert(numeric_equal_int(0, 999999999999999999) == false);
}
