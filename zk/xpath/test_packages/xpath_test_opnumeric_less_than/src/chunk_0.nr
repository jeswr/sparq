//! Test chunk 0 for op:numeric-less-than
//! Contains 50 tests

use dep::xpath::numeric_less_than_int;

#[test]
fn op_numeric_less_thanint2args_1() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-2147483648, -2147483648) == false);
}

#[test]
fn op_numeric_less_thanint2args_2() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-1873914410, -2147483648) == false);
}

#[test]
fn op_numeric_less_thanint2args_3() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(2147483647, -2147483648) == false);
}

#[test]
fn op_numeric_less_thanint2args_4() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-2147483648, -1873914410) == true);
}

#[test]
fn op_numeric_less_thanint2args_5() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-2147483648, 2147483647) == true);
}

#[test]
fn op_numeric_less_thanintg2args_1() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-999999999999999999, -999999999999999999) == false);
}

#[test]
fn op_numeric_less_thanintg2args_2() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(830993497117024304, -999999999999999999) == false);
}

#[test]
fn op_numeric_less_thanintg2args_3() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(999999999999999999, -999999999999999999) == false);
}

#[test]
fn op_numeric_less_thanintg2args_4() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-999999999999999999, 830993497117024304) == true);
}

#[test]
fn op_numeric_less_thanintg2args_5() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-999999999999999999, 999999999999999999) == true);
}

#[test]
fn op_numeric_less_thandec2args_1() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-999999999999999999, -999999999999999999) == false);
}

#[test]
fn op_numeric_less_thandec2args_2() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(617375191608514839, -999999999999999999) == false);
}

#[test]
fn op_numeric_less_thandec2args_3() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(999999999999999999, -999999999999999999) == false);
}

#[test]
fn op_numeric_less_thandec2args_4() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-999999999999999999, 617375191608514839) == true);
}

#[test]
fn op_numeric_less_thandec2args_5() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-999999999999999999, 999999999999999999) == true);
}

#[test]
fn op_numeric_less_thanlng2args_1() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-92233720368547758, -92233720368547758) == false);
}

#[test]
fn op_numeric_less_thanlng2args_2() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-47175562203048468, -92233720368547758) == false);
}

#[test]
fn op_numeric_less_thanlng2args_3() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(92233720368547758, -92233720368547758) == false);
}

#[test]
fn op_numeric_less_thanlng2args_4() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-92233720368547758, -47175562203048468) == true);
}

#[test]
fn op_numeric_less_thanlng2args_5() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-92233720368547758, 92233720368547758) == true);
}

#[test]
fn op_numeric_less_thanusht2args_1() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(0, 0) == false);
}

#[test]
fn op_numeric_less_thanusht2args_2() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(44633, 0) == false);
}

#[test]
fn op_numeric_less_thanusht2args_3() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(65535, 0) == false);
}

#[test]
fn op_numeric_less_thanusht2args_4() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(0, 44633) == true);
}

#[test]
fn op_numeric_less_thanusht2args_5() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(0, 65535) == true);
}

#[test]
fn op_numeric_less_thannint2args_1() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-999999999999999999, -999999999999999999) == false);
}

#[test]
fn op_numeric_less_thannint2args_2() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-297014075999096793, -999999999999999999) == false);
}

#[test]
fn op_numeric_less_thannint2args_3() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-1, -999999999999999999) == false);
}

#[test]
fn op_numeric_less_thannint2args_4() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-999999999999999999, -297014075999096793) == true);
}

#[test]
fn op_numeric_less_thannint2args_5() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-999999999999999999, -1) == true);
}

#[test]
fn op_numeric_less_thanpint2args_1() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(1, 1) == false);
}

#[test]
fn op_numeric_less_thanpint2args_2() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(52704602390610033, 1) == false);
}

#[test]
fn op_numeric_less_thanpint2args_3() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(999999999999999999, 1) == false);
}

#[test]
fn op_numeric_less_thanpint2args_4() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(1, 52704602390610033) == true);
}

#[test]
fn op_numeric_less_thanpint2args_5() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(1, 999999999999999999) == true);
}

#[test]
fn op_numeric_less_thanulng2args_1() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(0, 0) == false);
}

#[test]
fn op_numeric_less_thanulng2args_2() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(130747108607674654, 0) == false);
}

#[test]
fn op_numeric_less_thanulng2args_3() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(184467440737095516, 0) == false);
}

#[test]
fn op_numeric_less_thanulng2args_4() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(0, 130747108607674654) == true);
}

#[test]
fn op_numeric_less_thanulng2args_5() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(0, 184467440737095516) == true);
}

#[test]
fn op_numeric_less_thannpi2args_1() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-999999999999999999, -999999999999999999) == false);
}

#[test]
fn op_numeric_less_thannpi2args_2() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-475688437271870490, -999999999999999999) == false);
}

#[test]
fn op_numeric_less_thannpi2args_3() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(0, -999999999999999999) == false);
}

#[test]
fn op_numeric_less_thannpi2args_4() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-999999999999999999, -475688437271870490) == true);
}

#[test]
fn op_numeric_less_thannpi2args_5() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-999999999999999999, 0) == true);
}

#[test]
fn op_numeric_less_thannni2args_1() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(0, 0) == false);
}

#[test]
fn op_numeric_less_thannni2args_2() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(303884545991464527, 0) == false);
}

#[test]
fn op_numeric_less_thannni2args_3() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(999999999999999999, 0) == false);
}

#[test]
fn op_numeric_less_thannni2args_4() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(0, 303884545991464527) == true);
}

#[test]
fn op_numeric_less_thannni2args_5() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(0, 999999999999999999) == true);
}
