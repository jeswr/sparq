//! Test chunk 0 for fn:hours-from-dateTime
//! Contains 9 tests

use dep::xpath::{datetime_from_epoch_microseconds_with_tz, hours_from_datetime};

#[test]
fn fn_hours_from_datetime1args_1() {
    //  Evaluates The 'hours-from-dateTime' function with the arguments set as
    let dt = datetime_from_epoch_microseconds_with_tz(0, 0);
    assert(hours_from_datetime(dt) == 0);
}

#[test]
fn fn_hours_from_datetime1args_2() {
    //  Evaluates The 'hours-from-dateTime' function with the arguments set as
    let dt = datetime_from_epoch_microseconds_with_tz(828841252000000, 0);
    assert(hours_from_datetime(dt) == 1);
}

#[test]
fn fn_hours_from_datetime1args_3() {
    //  Evaluates The 'hours-from-dateTime' function with the arguments set as
    let dt = datetime_from_epoch_microseconds_with_tz(1924991999000000, 0);
    assert(hours_from_datetime(dt) == 23);
}

#[test]
fn fn_hours_from_datetime_1() {
    //  Evaluates The 'hours-from-dateTime' function As per example 1 of the F&O specs
    let dt = datetime_from_epoch_microseconds_with_tz(928156800000000, -300);
    assert(hours_from_datetime(dt) == 8);
}

#[test]
fn fn_hours_from_datetime_2() {
    //  Evaluates The 'hours-from-dateTime' function As per example 2 of the F&O specs
    let dt = datetime_from_epoch_microseconds_with_tz(946693200000000, -300);
    assert(hours_from_datetime(dt) == 21);
}

#[test]
fn fn_hours_from_datetime_3() {
    //  Evaluates The 'hours-from-dateTime' function As per example 3 of the F&O specs
    let dt = datetime_from_epoch_microseconds_with_tz(946693200000000, 0);
    assert(hours_from_datetime(dt) == 2);
}

#[test]
fn fn_hours_from_datetime_4() {
    //  Evaluates The 'hours-from-dateTime' function As per example 4 of the F&O specs
    let dt = datetime_from_epoch_microseconds_with_tz(946641600000000, 0);
    assert(hours_from_datetime(dt) == 12);
}

#[test]
fn fn_hours_from_datetime_6() {
    //  Evaluates The 'hours-from-dateTime' function that returns 0.
    let dt = datetime_from_epoch_microseconds_with_tz(928128000000000, -300);
    assert(hours_from_datetime(dt) == 0);
}

#[test]
fn fn_hours_from_datetime_7() {
    //  Evaluates The 'hours-from-dateTime' function that returns 23.
    let dt = datetime_from_epoch_microseconds_with_tz(928210800000000, -300);
    assert(hours_from_datetime(dt) == 23);
}
