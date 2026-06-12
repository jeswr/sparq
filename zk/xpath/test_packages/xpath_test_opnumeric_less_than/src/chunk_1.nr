//! Test chunk 1 for op:numeric-less-than
//! Contains 8 tests

use dep::xpath::numeric_less_than_int;

#[test]
fn op_numeric_less_thansht2args_1() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-32768, -32768) == false);
}

#[test]
fn op_numeric_less_thansht2args_2() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-5324, -32768) == false);
}

#[test]
fn op_numeric_less_thansht2args_3() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(32767, -32768) == false);
}

#[test]
fn op_numeric_less_thansht2args_4() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-32768, -5324) == true);
}

#[test]
fn op_numeric_less_thansht2args_5() {
    //  ******************************************************* Test:
    assert(numeric_less_than_int(-32768, 32767) == true);
}

#[test]
fn k_numericlt_1() {
    //  *******************************************************  Test: K-NumericLT-1
    assert(numeric_less_than_int(-1, 1) == true);
}

#[test]
fn k_numericlt_3() {
    //  *******************************************************  Test: K-NumericLT-3
    assert(numeric_less_than_int(-1, 1) == true);
}

#[test]
fn k_numericlt_22() {
    //  *******************************************************  Test: K-NumericLT-22
    assert(numeric_less_than_int(0, 4) == true);
}
