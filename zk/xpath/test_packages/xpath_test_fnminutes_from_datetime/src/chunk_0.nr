//! Test chunk 0 for fn:minutes-from-dateTime
//! Contains 7 tests

use dep::xpath::{datetime_from_epoch_microseconds_with_tz, minutes_from_datetime};

#[test]
fn fn_minutes_from_datetime1args_1() {
    //  Evaluates The 'minutes-from-dateTime' function with the arguments set as
    let dt = datetime_from_epoch_microseconds_with_tz(0, 0);
    assert(minutes_from_datetime(dt) == 0);
}

#[test]
fn fn_minutes_from_datetime1args_2() {
    //  Evaluates The 'minutes-from-dateTime' function with the arguments set as
    let dt = datetime_from_epoch_microseconds_with_tz(828841252000000, 0);
    assert(minutes_from_datetime(dt) == 40);
}

#[test]
fn fn_minutes_from_datetime1args_3() {
    //  Evaluates The 'minutes-from-dateTime' function with the arguments set as
    let dt = datetime_from_epoch_microseconds_with_tz(1924991999000000, 0);
    assert(minutes_from_datetime(dt) == 59);
}

#[test]
fn fn_minutes_from_datetime_1() {
    //  Evaluates The 'minutes-from-dateTime' function As per example 1 of the F&O
    let dt = datetime_from_epoch_microseconds_with_tz(928174800000000, -300);
    assert(minutes_from_datetime(dt) == 20);
}

#[test]
fn fn_minutes_from_datetime_2() {
    //  Evaluates The 'minutes-from-dateTime' function As per example 2 of the F&O
    let dt = datetime_from_epoch_microseconds_with_tz(928137600000000, 330);
    assert(minutes_from_datetime(dt) == 30);
}

#[test]
fn fn_minutes_from_datetime_6() {
    //  Evaluates The 'minutes-from-dateTime' function that returns 0.
    let dt = datetime_from_epoch_microseconds_with_tz(928198800000000, -300);
    assert(minutes_from_datetime(dt) == 0);
}

#[test]
fn fn_minutes_from_datetime_7() {
    //  Evaluates The 'minutes-from-dateTime' function that returns 59.
    let dt = datetime_from_epoch_microseconds_with_tz(928213140000000, -300);
    assert(minutes_from_datetime(dt) == 59);
}
