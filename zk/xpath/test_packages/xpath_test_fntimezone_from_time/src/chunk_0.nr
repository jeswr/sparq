//! Test chunk 0 for fn:timezone-from-time
//! Contains 3 tests

use dep::xpath::{
    duration_equal, duration_from_microseconds, time_from_microseconds_with_tz, timezone_from_time,
};

#[test]
fn fn_timezone_from_time_1() {
    //  Evaluates The 'timezone-from-time' function As per example 1 of the F&O specs
    let t = time_from_microseconds_with_tz(48000000000, -300);
    assert(
        duration_equal(
            timezone_from_time(t),
            duration_from_microseconds(-18000000000),
        ),
    );
}

#[test]
fn fn_timezone_from_time_6() {
    //  Evaluates The 'timezone-from-time' function that returns a dayTimeDuration of
    let t = time_from_microseconds_with_tz(3540000000, 0);
    assert(duration_equal(timezone_from_time(t), duration_from_microseconds(0)));
}

#[test]
fn fn_timezone_from_time_7() {
    //  Evaluates The 'timezone-from-time' function with a timezone component of
    let t = time_from_microseconds_with_tz(84059000000, 0);
    assert(duration_equal(timezone_from_time(t), duration_from_microseconds(0)));
}
