//! Test chunk 0 for fn:day-from-dateTime
//! Contains 7 tests

use dep::xpath::{datetime_from_epoch_microseconds_with_tz, day_from_datetime};

#[test]
fn fn_day_from_datetime1args_1() {
    //  Evaluates The 'day-from-dateTime' function with the arguments set as follows:
    let dt = datetime_from_epoch_microseconds_with_tz(0, 0);
    assert(day_from_datetime(dt) == 1);
}

#[test]
fn fn_day_from_datetime1args_2() {
    //  Evaluates The 'day-from-dateTime' function with the arguments set as follows:
    let dt = datetime_from_epoch_microseconds_with_tz(828841252000000, 0);
    assert(day_from_datetime(dt) == 7);
}

#[test]
fn fn_day_from_datetime1args_3() {
    //  Evaluates The 'day-from-dateTime' function with the arguments set as follows:
    let dt = datetime_from_epoch_microseconds_with_tz(1924991999000000, 0);
    assert(day_from_datetime(dt) == 31);
}

#[test]
fn fn_day_from_datetime_1() {
    //  Evaluates The 'day-from-dateTime' function As per example 1 (of this fucntion)
    let dt = datetime_from_epoch_microseconds_with_tz(928174800000000, -300);
    assert(day_from_datetime(dt) == 31);
}

#[test]
fn fn_day_from_datetime_2() {
    //  Evaluates The 'day-from-dateTime' function As per example 2 (for this
    let dt = datetime_from_epoch_microseconds_with_tz(946688400000000, -300);
    assert(day_from_datetime(dt) == 31);
}

#[test]
fn fn_day_from_datetime_3() {
    //  Evaluates The 'day-from-dateTime' function As per example 3 (for this
    let dt = datetime_from_epoch_microseconds_with_tz(946686000000000, 0);
    assert(day_from_datetime(dt) == 1);
}

#[test]
fn fn_day_from_datetime_6() {
    //  Evaluates The 'day-from-dateTime' function that returns 1.
    let dt = datetime_from_epoch_microseconds_with_tz(915168000000000, -300);
    assert(day_from_datetime(dt) == 1);
}
