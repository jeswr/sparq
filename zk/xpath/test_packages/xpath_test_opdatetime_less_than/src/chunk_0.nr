//! Test chunk 0 for op:dateTime-less-than
//! Contains 9 tests

use dep::xpath::{datetime_from_epoch_microseconds_with_tz, datetime_less_than};

#[test]
fn op_datetime_less_than2args_1() {
    //  ******************************************************* Test:
    let dt1 = datetime_from_epoch_microseconds_with_tz(0, 0);
    let dt2 = datetime_from_epoch_microseconds_with_tz(0, 0);
    assert(datetime_less_than(dt1, dt2) == false);
}

#[test]
fn op_datetime_less_than2args_2() {
    //  ******************************************************* Test:
    let dt1 = datetime_from_epoch_microseconds_with_tz(828841252000000, 0);
    let dt2 = datetime_from_epoch_microseconds_with_tz(0, 0);
    assert(datetime_less_than(dt1, dt2) == false);
}

#[test]
fn op_datetime_less_than2args_3() {
    //  ******************************************************* Test:
    let dt1 = datetime_from_epoch_microseconds_with_tz(1924991999000000, 0);
    let dt2 = datetime_from_epoch_microseconds_with_tz(0, 0);
    assert(datetime_less_than(dt1, dt2) == false);
}

#[test]
fn op_datetime_less_than2args_4() {
    //  ******************************************************* Test:
    let dt1 = datetime_from_epoch_microseconds_with_tz(0, 0);
    let dt2 = datetime_from_epoch_microseconds_with_tz(828841252000000, 0);
    assert(datetime_less_than(dt1, dt2) == true);
}

#[test]
fn op_datetime_less_than2args_5() {
    //  ******************************************************* Test:
    let dt1 = datetime_from_epoch_microseconds_with_tz(0, 0);
    let dt2 = datetime_from_epoch_microseconds_with_tz(1924991999000000, 0);
    assert(datetime_less_than(dt1, dt2) == true);
}

#[test]
fn cbcl_datetime_less_than_002() {
    //  test comparison of dateTime
    let dt1 = datetime_from_epoch_microseconds_with_tz(1201651260000000, 0);
    let dt2 = datetime_from_epoch_microseconds_with_tz(1201708800000000, 540);
    assert(datetime_less_than(dt1, dt2) == true);
}

#[test]
fn cbcl_datetime_less_than_003() {
    //  test comparison of dateTime
    let dt1 = datetime_from_epoch_microseconds_with_tz(1201705260000000, 540);
    let dt2 = datetime_from_epoch_microseconds_with_tz(1201651260000000, 0);
    assert(datetime_less_than(dt1, dt2) == false);
}

#[test]
fn cbcl_datetime_less_than_004() {
    //  test comparison of dateTime
    let dt1 = datetime_from_epoch_microseconds_with_tz(1201737660000000, 0);
    let dt2 = datetime_from_epoch_microseconds_with_tz(1201705260000000, 540);
    assert(datetime_less_than(dt1, dt2) == false);
}

#[test]
fn cbcl_datetime_less_than_005() {
    //  test comparison of dateTime
    let dt1 = datetime_from_epoch_microseconds_with_tz(1201705260000000, 540);
    let dt2 = datetime_from_epoch_microseconds_with_tz(1201737660000000, 0);
    assert(datetime_less_than(dt1, dt2) == true);
}
