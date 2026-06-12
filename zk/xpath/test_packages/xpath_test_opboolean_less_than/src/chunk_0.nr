//! Test chunk 0 for op:boolean-less-than
//! Contains 19 tests

use dep::xpath::boolean_less_than;

#[test]
fn op_boolean_less_than2args_1() {
    //  ******************************************************* Test:
    assert(boolean_less_than(false, false) == false);
}

#[test]
fn op_boolean_less_than2args_2() {
    //  ******************************************************* Test:
    assert(boolean_less_than(true, false) == false);
}

#[test]
fn op_boolean_less_than2args_3() {
    //  ******************************************************* Test:
    assert(boolean_less_than(false, false) == false);
}

#[test]
fn op_boolean_less_than2args_4() {
    //  ******************************************************* Test:
    assert(boolean_less_than(false, true) == true);
}

#[test]
fn op_boolean_less_than2args_5() {
    //  ******************************************************* Test:
    assert(boolean_less_than(false, false) == false);
}

#[test]
fn op_boolean_less_than_1() {
    //  ******************************************************* Test:
    assert(boolean_less_than(false, true) == true);
}

#[test]
fn op_boolean_less_than_3() {
    //  ******************************************************* Test:
    assert(boolean_less_than(false, false) == false);
}

#[test]
fn op_boolean_less_than_5() {
    //  ******************************************************* Test:
    assert(boolean_less_than(true, true) == false);
}

#[test]
fn op_boolean_less_than_7() {
    //  ******************************************************* Test:
    assert(boolean_less_than(true, false) == false);
}

#[test]
fn op_boolean_less_than_9() {
    //  ******************************************************* Test:
    assert(boolean_less_than(true, true) == false);
}

#[test]
fn op_boolean_less_than_11() {
    //  ******************************************************* Test:
    assert(boolean_less_than(true, false) == false);
}

#[test]
fn op_boolean_less_than_more_args_001() {
    //  *******************************************************  Test:
    assert(boolean_less_than(true, true) == false);
}

#[test]
fn op_boolean_less_than_more_args_002() {
    //  *******************************************************  Test:
    assert(boolean_less_than(true, true) == false);
}

#[test]
fn op_boolean_less_than_more_args_003() {
    //  *******************************************************  Test:
    assert(boolean_less_than(false, true) == true);
}

#[test]
fn op_boolean_less_than_more_args_004() {
    //  *******************************************************  Test:
    assert(boolean_less_than(true, true) == false);
}

#[test]
fn op_boolean_less_than_more_args_005() {
    //  *******************************************************  Test:
    assert(boolean_less_than(true, false) == false);
}

#[test]
fn op_boolean_less_than_more_args_006() {
    //  *******************************************************  Test:
    assert(boolean_less_than(true, false) == false);
}

#[test]
fn op_boolean_less_than_more_args_007() {
    //  *******************************************************  Test:
    assert(boolean_less_than(false, true) == true);
}

#[test]
fn k_booleanlt_1() {
    //  *******************************************************  Test: K-BooleanLT-1
    assert(boolean_less_than(false, true) == true);
}
