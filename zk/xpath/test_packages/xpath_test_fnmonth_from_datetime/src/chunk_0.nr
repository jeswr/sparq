//! Test chunk 0 for fn:month-from-dateTime
//! Contains 6 tests

use dep::xpath::{datetime_from_epoch_microseconds_with_tz, month_from_datetime};

#[test]
fn fn_month_from_datetime1args_1() {
    //  Evaluates The 'month-from-dateTime' function with the arguments set as
    let dt = datetime_from_epoch_microseconds_with_tz(0, 0);
    assert(month_from_datetime(dt) == 1);
}

#[test]
fn fn_month_from_datetime1args_2() {
    //  Evaluates The 'month-from-dateTime' function with the arguments set as
    let dt = datetime_from_epoch_microseconds_with_tz(828841252000000, 0);
    assert(month_from_datetime(dt) == 4);
}

#[test]
fn fn_month_from_datetime1args_3() {
    //  Evaluates The 'month-from-dateTime' function with the arguments set as
    let dt = datetime_from_epoch_microseconds_with_tz(1924991999000000, 0);
    assert(month_from_datetime(dt) == 12);
}

#[test]
fn fn_month_from_datetime_1() {
    //  Evaluates The 'month-from-dateTime' function As per example 1 (of this
    let dt = datetime_from_epoch_microseconds_with_tz(928174800000000, -300);
    assert(month_from_datetime(dt) == 5);
}

#[test]
fn fn_month_from_datetime_2() {
    //  Evaluates The 'month-from-dateTime' function As per example 2 (for this
    let dt = datetime_from_epoch_microseconds_with_tz(946686000000000, -300);
    assert(month_from_datetime(dt) == 12);
}

#[test]
fn fn_month_from_datetime_3() {
    //  Evaluates The 'month-from-dateTime' function As per example 3 (for this
    let dt = datetime_from_epoch_microseconds_with_tz(946686000000000, 0);
    assert(month_from_datetime(dt) == 1);
}
