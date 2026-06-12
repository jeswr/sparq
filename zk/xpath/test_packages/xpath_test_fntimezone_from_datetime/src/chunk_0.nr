//! Test chunk 0 for fn:timezone-from-dateTime
//! Contains 4 tests

use dep::xpath::{
    datetime_from_epoch_microseconds_with_tz, duration_equal, duration_from_microseconds,
    timezone_from_datetime,
};

#[test]
fn fn_timezone_from_datetimealt_1() {
    //  Evaluates The 'timezone-from-dateTime' function As per example 1 (for this
    let dt = datetime_from_epoch_microseconds_with_tz(928174800000000, -300);
    assert(
        duration_equal(
            timezone_from_datetime(dt),
            duration_from_microseconds(-18000000000),
        ),
    );
}

#[test]
fn fn_timezone_from_datetimealt_2() {
    //  Evaluates The 'timezone-from-dateTime' function As per example 2 (for this
    let dt = datetime_from_epoch_microseconds_with_tz(960816000000000, 0);
    assert(duration_equal(timezone_from_datetime(dt), duration_from_microseconds(0)));
}

#[test]
fn fn_timezone_from_datetimealt_6() {
    //  Evaluates The 'timezone-from-dateTime' function that returns dayTimeDuration
    let dt = datetime_from_epoch_microseconds_with_tz(928110000000000, 0);
    assert(duration_equal(timezone_from_datetime(dt), duration_from_microseconds(0)));
}

#[test]
fn fn_timezone_from_datetimealt_7() {
    //  Evaluates The 'timezone-from-dateTime' function that evaluates a timezone of
    let dt = datetime_from_epoch_microseconds_with_tz(928192800000000, 0);
    assert(duration_equal(timezone_from_datetime(dt), duration_from_microseconds(0)));
}
