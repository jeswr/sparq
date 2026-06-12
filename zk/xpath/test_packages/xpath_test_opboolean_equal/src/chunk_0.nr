//! Test chunk 0 for op:boolean-equal
//! Contains 41 tests

use dep::xpath::boolean_equal;

#[test]
fn op_boolean_equal2args_1() {
    //  ******************************************************* Test:
    assert(boolean_equal(false, false) == true);
}

#[test]
fn op_boolean_equal2args_2() {
    //  ******************************************************* Test:
    assert(boolean_equal(true, false) == false);
}

#[test]
fn op_boolean_equal2args_3() {
    //  ******************************************************* Test:
    assert(boolean_equal(false, false) == true);
}

#[test]
fn op_boolean_equal2args_4() {
    //  ******************************************************* Test:
    assert(boolean_equal(false, true) == false);
}

#[test]
fn op_boolean_equal2args_5() {
    //  ******************************************************* Test:
    assert(boolean_equal(false, false) == true);
}

#[test]
fn op_boolean_equal_1() {
    //  ******************************************************* Test:
    assert(boolean_equal(false, true) == false);
}

#[test]
fn op_boolean_equal_2() {
    //  ******************************************************* Test:
    assert(boolean_equal(false, false) == true);
}

#[test]
fn op_boolean_equal_3() {
    //  ******************************************************* Test:
    assert(boolean_equal(true, false) == false);
}

#[test]
fn op_boolean_equal_4() {
    //  ******************************************************* Test:
    assert(boolean_equal(true, false) == false);
}

#[test]
fn op_boolean_equal_5() {
    //  ******************************************************* Test:
    assert(boolean_equal(false, true) == false);
}

#[test]
fn op_boolean_equal_6() {
    //  ******************************************************* Test:
    assert(boolean_equal(true, false) == false);
}

#[test]
fn op_boolean_equal_7() {
    //  ******************************************************* Test:
    assert(boolean_equal(true, true) == true);
}

#[test]
fn op_boolean_equal_8() {
    //  ******************************************************* Test:
    assert(boolean_equal(true, true) == true);
}

#[test]
fn op_boolean_equal_9() {
    //  ******************************************************* Test:
    assert(boolean_equal(false, true) == false);
}

#[test]
fn op_boolean_equal_10() {
    //  ******************************************************* Test:
    assert(boolean_equal(true, false) == false);
}

#[test]
fn op_boolean_equal_11() {
    //  ******************************************************* Test:
    assert(boolean_equal(true, true) == true);
}

#[test]
fn op_boolean_equal_15() {
    //  ******************************************************* Test:
    assert(boolean_equal(true, false) == false);
}

#[test]
fn op_boolean_equal_16() {
    //  ******************************************************* Test:
    assert(boolean_equal(true, false) == false);
}

#[test]
fn op_boolean_equal_19() {
    // Only one comparison allowed per expression, unless parenthesized
    assert(boolean_equal(true, true) == true);
}

#[test]
fn op_boolean_equal_more_args_001() {
    //  *******************************************************  Test:
    assert(boolean_equal(false, true) == false);
}

#[test]
fn op_boolean_equal_more_args_002() {
    //  *******************************************************  Test:
    assert(boolean_equal(false, true) == false);
}

#[test]
fn op_boolean_equal_more_args_003() {
    //  *******************************************************  Test:
    assert(boolean_equal(false, false) == true);
}

#[test]
fn op_boolean_equal_more_args_004() {
    //  *******************************************************  Test:
    assert(boolean_equal(false, false) == true);
}

#[test]
fn op_boolean_equal_more_args_005() {
    //  *******************************************************  Test:
    assert(boolean_equal(false, true) == false);
}

#[test]
fn op_boolean_equal_more_args_006() {
    //  *******************************************************  Test:
    assert(boolean_equal(false, true) == false);
}

#[test]
fn op_boolean_equal_more_args_007() {
    //  *******************************************************  Test:
    assert(boolean_equal(false, false) == true);
}

#[test]
fn op_boolean_equal_more_args_008() {
    //  *******************************************************  Test:
    assert(boolean_equal(true, true) == true);
}

#[test]
fn op_boolean_equal_more_args_009() {
    //  *******************************************************  Test:
    assert(boolean_equal(true, false) == false);
}

#[test]
fn op_boolean_equal_more_args_010() {
    //  *******************************************************  Test:
    assert(boolean_equal(true, true) == true);
}

#[test]
fn op_boolean_equal_more_args_011() {
    //  *******************************************************  Test:
    assert(boolean_equal(true, false) == false);
}

#[test]
fn op_boolean_equal_more_args_012() {
    //  *******************************************************  Test:
    assert(boolean_equal(true, true) == true);
}

#[test]
fn op_boolean_equal_more_args_013() {
    //  *******************************************************  Test:
    assert(boolean_equal(false, true) == false);
}

#[test]
fn op_boolean_equal_more_args_014() {
    //  *******************************************************  Test:
    assert(boolean_equal(true, true) == true);
}

#[test]
fn op_boolean_equal_more_args_015() {
    //  *******************************************************  Test:
    assert(boolean_equal(true, false) == false);
}

#[test]
fn op_boolean_equal_more_args_016() {
    //  *******************************************************  Test:
    assert(boolean_equal(true, true) == true);
}

#[test]
fn op_boolean_equal_more_args_017() {
    //  *******************************************************  Test:
    assert(boolean_equal(true, true) == true);
}

#[test]
fn op_boolean_equal_more_args_018() {
    //  *******************************************************  Test:
    assert(boolean_equal(true, true) == true);
}

#[test]
fn op_boolean_equal_more_args_019() {
    //  *******************************************************  Test:
    assert(boolean_equal(true, false) == false);
}

#[test]
fn k_booleanequal_1() {
    //  *******************************************************  Test:
    assert(boolean_equal(false, false) == true);
}

#[test]
fn k_booleanequal_2() {
    //  *******************************************************  Test:
    assert(boolean_equal(true, true) == true);
}

#[test]
fn k_booleanequal_5() {
    //  *******************************************************  Test:
    assert(boolean_equal(false, false) == true);
}
