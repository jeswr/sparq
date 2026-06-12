//! Test chunk 0 for op:subtract-dateTimes
//! Contains 3 tests

use dep::xpath::{
    datetime_difference, datetime_from_epoch_microseconds_with_tz, duration_equal,
    duration_from_microseconds,
};

#[test]
fn op_subtract_datetimes_yielding_dtd_1() {
    //  ******************************************************* Test:
    let dt1 = datetime_from_epoch_microseconds_with_tz(972904320000000, -300);
    let dt2 = datetime_from_epoch_microseconds_with_tz(943779600000000, 0);
    assert(duration_equal(
        datetime_difference(dt1, dt2),
        duration_from_microseconds(29124720000000),
    ));
}

#[test]
fn op_subtract_datetimes_yielding_dtd_2() {
    //  ******************************************************* Test:
    let dt1 = datetime_from_epoch_microseconds_with_tz(976594087000000, 300);
    let dt2 = datetime_from_epoch_microseconds_with_tz(944971687000000, 300);
    assert(duration_equal(
        datetime_difference(dt1, dt2),
        duration_from_microseconds(31622400000000),
    ));
}

#[test]
fn op_subtract_datetimes_yielding_dtd_3() {
    //  ******************************************************* Test:
    let dt1 = datetime_from_epoch_microseconds_with_tz(949565347000000, -360);
    let dt2 = datetime_from_epoch_microseconds_with_tz(886493347000000, -360);
    assert(duration_equal(
        datetime_difference(dt1, dt2),
        duration_from_microseconds(63072000000000),
    ));
}
