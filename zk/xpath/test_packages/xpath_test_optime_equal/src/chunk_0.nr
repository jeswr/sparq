//! Test chunk 0 for op:time-equal
//! Contains 10 tests

use dep::xpath::{time_equal, time_from_microseconds_with_tz};

#[test]
fn op_time_equal2args_1() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(0, 0);
    let t2 = time_from_microseconds_with_tz(0, 0);
    assert(time_equal(t1, t2) == true);
}

#[test]
fn op_time_equal2args_2() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(29015000000, 0);
    let t2 = time_from_microseconds_with_tz(0, 0);
    assert(time_equal(t1, t2) == false);
}

#[test]
fn op_time_equal2args_3() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(86399000000, 0);
    let t2 = time_from_microseconds_with_tz(0, 0);
    assert(time_equal(t1, t2) == false);
}

#[test]
fn op_time_equal2args_4() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(0, 0);
    let t2 = time_from_microseconds_with_tz(29015000000, 0);
    assert(time_equal(t1, t2) == false);
}

#[test]
fn op_time_equal2args_5() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(0, 0);
    let t2 = time_from_microseconds_with_tz(86399000000, 0);
    assert(time_equal(t1, t2) == false);
}

#[test]
fn cbcl_time_equal_002() {
    //  test comparison of time
    let t1 = time_from_microseconds_with_tz(50400000000, -720);
    let t2 = time_from_microseconds_with_tz(7200000000, 0);
    assert(time_equal(t1, t2) == false);
}

#[test]
fn cbcl_time_equal_003() {
    //  test comparison of time
    let t1 = time_from_microseconds_with_tz(7200000000, 0);
    let t2 = time_from_microseconds_with_tz(50400000000, -720);
    assert(time_equal(t1, t2) == false);
}

#[test]
fn cbcl_time_equal_004() {
    //  test comparison of time
    let t1 = time_from_microseconds_with_tz(1000000, 60);
    let t2 = time_from_microseconds_with_tz(0, 0);
    assert(time_equal(t1, t2) == false);
}

#[test]
fn cbcl_time_equal_005() {
    //  test comparison of time
    let t1 = time_from_microseconds_with_tz(0, 0);
    let t2 = time_from_microseconds_with_tz(1000000, 60);
    assert(time_equal(t1, t2) == false);
}

#[test]
fn cbcl_time_equal_017() {
    //  test comparison of time
    let t1 = time_from_microseconds_with_tz(0, 720);
    let t2 = time_from_microseconds_with_tz(1000000, 0);
    assert(time_equal(t1, t2) == false);
}
