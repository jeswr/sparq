//! Test chunk 1 for op:numeric-greater-than
//! Contains 8 tests

use dep::xpath::numeric_greater_than_int;

#[test]
fn op_numeric_greater_thansht2args_1() {
    //  ******************************************************* Test:
    assert(numeric_greater_than_int(-32768, -32768) == false);
}

#[test]
fn op_numeric_greater_thansht2args_2() {
    //  ******************************************************* Test:
    assert(numeric_greater_than_int(-5324, -32768) == true);
}

#[test]
fn op_numeric_greater_thansht2args_3() {
    //  ******************************************************* Test:
    assert(numeric_greater_than_int(32767, -32768) == true);
}

#[test]
fn op_numeric_greater_thansht2args_4() {
    //  ******************************************************* Test:
    assert(numeric_greater_than_int(-32768, -5324) == false);
}

#[test]
fn op_numeric_greater_thansht2args_5() {
    //  ******************************************************* Test:
    assert(numeric_greater_than_int(-32768, 32767) == false);
}

#[test]
fn k_numericgt_1() {
    //  *******************************************************  Test: K-NumericGT-1
    assert(numeric_greater_than_int(1, -1) == true);
}

#[test]
fn k_numericgt_3() {
    //  *******************************************************  Test: K-NumericGT-3
    assert(numeric_greater_than_int(1, -1) == true);
}

#[test]
fn k_numericgt_23() {
    //  *******************************************************  Test: K-NumericGT-23
    assert(numeric_greater_than_int(4, 0) == true);
}
