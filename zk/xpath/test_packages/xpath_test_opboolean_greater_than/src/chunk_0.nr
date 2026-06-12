//! Test chunk 0 for op:boolean-greater-than
//! Contains 19 tests

use dep::xpath::boolean_greater_than;

#[test]
fn op_boolean_greater_than2args_1() {
    //  ******************************************************* Test:
    assert(boolean_greater_than(false, false) == false);
}

#[test]
fn op_boolean_greater_than2args_2() {
    //  ******************************************************* Test:
    assert(boolean_greater_than(true, false) == true);
}

#[test]
fn op_boolean_greater_than2args_3() {
    //  ******************************************************* Test:
    assert(boolean_greater_than(false, false) == false);
}

#[test]
fn op_boolean_greater_than2args_4() {
    //  ******************************************************* Test:
    assert(boolean_greater_than(false, true) == false);
}

#[test]
fn op_boolean_greater_than2args_5() {
    //  ******************************************************* Test:
    assert(boolean_greater_than(false, false) == false);
}

#[test]
fn op_boolean_greater_than_1() {
    //  ******************************************************* Test:
    assert(boolean_greater_than(false, true) == false);
}

#[test]
fn op_boolean_greater_than_3() {
    //  ******************************************************* Test:
    assert(boolean_greater_than(false, false) == false);
}

#[test]
fn op_boolean_greater_than_5() {
    //  ******************************************************* Test:
    assert(boolean_greater_than(true, true) == false);
}

#[test]
fn op_boolean_greater_than_7() {
    //  ******************************************************* Test:
    assert(boolean_greater_than(true, false) == true);
}

#[test]
fn op_boolean_greater_than_9() {
    //  ******************************************************* Test:
    assert(boolean_greater_than(true, true) == false);
}

#[test]
fn op_boolean_greater_than_11() {
    //  ******************************************************* Test:
    assert(boolean_greater_than(true, false) == true);
}

#[test]
fn op_boolean_greater_than_more_args_001() {
    //  *******************************************************  Test:
    assert(boolean_greater_than(true, true) == false);
}

#[test]
fn op_boolean_greater_than_more_args_002() {
    //  *******************************************************  Test:
    assert(boolean_greater_than(true, true) == false);
}

#[test]
fn op_boolean_greater_than_more_args_003() {
    //  *******************************************************  Test:
    assert(boolean_greater_than(false, true) == false);
}

#[test]
fn op_boolean_greater_than_more_args_004() {
    //  *******************************************************  Test:
    assert(boolean_greater_than(true, true) == false);
}

#[test]
fn op_boolean_greater_than_more_args_005() {
    //  *******************************************************  Test:
    assert(boolean_greater_than(true, false) == true);
}

#[test]
fn op_boolean_greater_than_more_args_006() {
    //  *******************************************************  Test:
    assert(boolean_greater_than(true, false) == true);
}

#[test]
fn op_boolean_greater_than_more_args_007() {
    //  *******************************************************  Test:
    assert(boolean_greater_than(false, true) == false);
}

#[test]
fn k_booleangt_1() {
    //  *******************************************************  Test: K-BooleanGT-1
    assert(boolean_greater_than(true, false) == true);
}
