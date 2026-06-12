//! Test chunk 0 for op:time-less-than
//! Contains 8 tests

use dep::xpath::{time_from_microseconds_with_tz, time_less_than};

#[test]
fn op_time_less_than2args_1() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(0, 0);
    let t2 = time_from_microseconds_with_tz(0, 0);
    assert(time_less_than(t1, t2) == false);
}

#[test]
fn op_time_less_than2args_2() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(29015000000, 0);
    let t2 = time_from_microseconds_with_tz(0, 0);
    assert(time_less_than(t1, t2) == false);
}

#[test]
fn op_time_less_than2args_3() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(86399000000, 0);
    let t2 = time_from_microseconds_with_tz(0, 0);
    assert(time_less_than(t1, t2) == false);
}

#[test]
fn op_time_less_than2args_4() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(0, 0);
    let t2 = time_from_microseconds_with_tz(29015000000, 0);
    assert(time_less_than(t1, t2) == true);
}

#[test]
fn op_time_less_than2args_5() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(0, 0);
    let t2 = time_from_microseconds_with_tz(86399000000, 0);
    assert(time_less_than(t1, t2) == true);
}

#[test]
fn op_time_less_than_2() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(39600000000, -300);
    let t2 = time_from_microseconds_with_tz(61200000000, 0);
    assert(time_less_than(t1, t2) == true);
}

#[test]
fn cbcl_time_less_than_002() {
    //  test comparison of time
    let t1 = time_from_microseconds_with_tz(50400000000, -720);
    let t2 = time_from_microseconds_with_tz(7200000000, 0);
    assert(time_less_than(t1, t2) == false);
}

#[test]
fn cbcl_time_less_than_003() {
    //  test comparison of time
    let t1 = time_from_microseconds_with_tz(7200000000, 0);
    let t2 = time_from_microseconds_with_tz(50400000000, -720);
    assert(time_less_than(t1, t2) == true);
}
