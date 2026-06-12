//! Test chunk 0 for op:dateTime-equal
//! Contains 10 tests

use dep::xpath::{datetime_equal, datetime_from_epoch_microseconds_with_tz};

#[test]
fn op_datetime_equal2args_1() {
    //  ******************************************************* Test:
    let dt1 = datetime_from_epoch_microseconds_with_tz(0, 0);
    let dt2 = datetime_from_epoch_microseconds_with_tz(0, 0);
    assert(datetime_equal(dt1, dt2) == true);
}

#[test]
fn op_datetime_equal2args_2() {
    //  ******************************************************* Test:
    let dt1 = datetime_from_epoch_microseconds_with_tz(828841252000000, 0);
    let dt2 = datetime_from_epoch_microseconds_with_tz(0, 0);
    assert(datetime_equal(dt1, dt2) == false);
}

#[test]
fn op_datetime_equal2args_3() {
    //  ******************************************************* Test:
    let dt1 = datetime_from_epoch_microseconds_with_tz(1924991999000000, 0);
    let dt2 = datetime_from_epoch_microseconds_with_tz(0, 0);
    assert(datetime_equal(dt1, dt2) == false);
}

#[test]
fn op_datetime_equal2args_4() {
    //  ******************************************************* Test:
    let dt1 = datetime_from_epoch_microseconds_with_tz(0, 0);
    let dt2 = datetime_from_epoch_microseconds_with_tz(828841252000000, 0);
    assert(datetime_equal(dt1, dt2) == false);
}

#[test]
fn op_datetime_equal2args_5() {
    //  ******************************************************* Test:
    let dt1 = datetime_from_epoch_microseconds_with_tz(0, 0);
    let dt2 = datetime_from_epoch_microseconds_with_tz(1924991999000000, 0);
    assert(datetime_equal(dt1, dt2) == false);
}

#[test]
fn cbcl_datetime_equal_002() {
    //  test comparison of dateTime
    let dt1 = datetime_from_epoch_microseconds_with_tz(1201651260000000, 0);
    let dt2 = datetime_from_epoch_microseconds_with_tz(1201708800000000, 540);
    assert(datetime_equal(dt1, dt2) == false);
}

#[test]
fn cbcl_datetime_equal_003() {
    //  test comparison of dateTime
    let dt1 = datetime_from_epoch_microseconds_with_tz(1201705260000000, 540);
    let dt2 = datetime_from_epoch_microseconds_with_tz(1201651260000000, 0);
    assert(datetime_equal(dt1, dt2) == false);
}

#[test]
fn cbcl_datetime_equal_004() {
    //  test comparison of dateTime
    let dt1 = datetime_from_epoch_microseconds_with_tz(1201737660000000, 0);
    let dt2 = datetime_from_epoch_microseconds_with_tz(1201705260000000, 540);
    assert(datetime_equal(dt1, dt2) == false);
}

#[test]
fn cbcl_datetime_equal_005() {
    //  test comparison of dateTime
    let dt1 = datetime_from_epoch_microseconds_with_tz(1201705260000000, 540);
    let dt2 = datetime_from_epoch_microseconds_with_tz(1201737660000000, 0);
    assert(datetime_equal(dt1, dt2) == false);
}

#[test]
fn cbcl_datetime_equal_017() {
    //  test comparison of dateTime
    let dt1 = datetime_from_epoch_microseconds_with_tz(93182400000000, 720);
    let dt2 = datetime_from_epoch_microseconds_with_tz(93225601000000, 0);
    assert(datetime_equal(dt1, dt2) == false);
}
