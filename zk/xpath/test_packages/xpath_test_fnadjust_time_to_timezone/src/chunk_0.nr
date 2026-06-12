//! Test chunk 0 for fn:adjust-time-to-timezone
//! Contains 8 tests

use dep::xpath::{
    adjust_time_to_timezone, adjust_time_to_timezone_none, duration_equal,
    duration_from_microseconds, hours_from_time, minutes_from_time, seconds_from_time,
    time_from_microseconds_with_tz, timezone_from_time,
};

#[test]
fn fn_adjust_time_to_timezone1args_1() {
    //  Evaluates The 'adjust-time-to-timezone' function with the arguments set as
    let t = time_from_microseconds_with_tz(0, 0);
    let result = adjust_time_to_timezone(t, -600);
    assert(hours_from_time(result) == 14);
    assert(minutes_from_time(result) == 0);
    assert(seconds_from_time(result) == 0);
    assert(
        duration_equal(
            timezone_from_time(result),
            duration_from_microseconds(-36000000000),
        ),
    );
}

#[test]
fn fn_adjust_time_to_timezone1args_2() {
    //  Evaluates The 'adjust-time-to-timezone' function with the arguments set as
    let t = time_from_microseconds_with_tz(29015000000, 0);
    let result = adjust_time_to_timezone(t, -600);
    assert(hours_from_time(result) == 22);
    assert(minutes_from_time(result) == 3);
    assert(seconds_from_time(result) == 35);
    assert(
        duration_equal(
            timezone_from_time(result),
            duration_from_microseconds(-36000000000),
        ),
    );
}

#[test]
fn fn_adjust_time_to_timezone1args_3() {
    //  Evaluates The 'adjust-time-to-timezone' function with the arguments set as
    let t = time_from_microseconds_with_tz(86399000000, 0);
    let result = adjust_time_to_timezone(t, -600);
    assert(hours_from_time(result) == 13);
    assert(minutes_from_time(result) == 59);
    assert(seconds_from_time(result) == 59);
    assert(
        duration_equal(
            timezone_from_time(result),
            duration_from_microseconds(-36000000000),
        ),
    );
}

#[test]
fn fn_adjust_time_to_timezone_1() {
    // Test Description: Evaluates The 'adjust-time-to-timezone' function as per
    let t = time_from_microseconds_with_tz(36000000000, -300);
    let result = adjust_time_to_timezone(t, -300);
    assert(hours_from_time(result) == 10);
    assert(minutes_from_time(result) == 0);
    assert(seconds_from_time(result) == 0);
    assert(
        duration_equal(
            timezone_from_time(result),
            duration_from_microseconds(-18000000000),
        ),
    );
}

#[test]
fn fn_adjust_time_to_timezone_2() {
    // Test Description: Evaluates The 'adjust-time-to-timezone' function as per
    let t = time_from_microseconds_with_tz(36000000000, -420);
    let result = adjust_time_to_timezone(t, -300);
    assert(hours_from_time(result) == 12);
    assert(minutes_from_time(result) == 0);
    assert(seconds_from_time(result) == 0);
    assert(
        duration_equal(
            timezone_from_time(result),
            duration_from_microseconds(-18000000000),
        ),
    );
}

#[test]
fn fn_adjust_time_to_timezone_5() {
    // Test Description: Evaluates The 'adjust-time-to-timezone' function as per
    let t = time_from_microseconds_with_tz(36000000000, -300);
    let result = adjust_time_to_timezone_none(t);
    assert(hours_from_time(result) == 10);
    assert(minutes_from_time(result) == 0);
    assert(seconds_from_time(result) == 0);
}

#[test]
fn fn_adjust_time_to_timezone_6() {
    // Test Description: Evaluates The 'adjust-time-to-timezone' function as per
    let t = time_from_microseconds_with_tz(36000000000, -420);
    let result = adjust_time_to_timezone_none(t);
    assert(hours_from_time(result) == 10);
    assert(minutes_from_time(result) == 0);
    assert(seconds_from_time(result) == 0);
}

#[test]
fn fn_adjust_time_to_timezone_7() {
    // Test Description: Evaluates The 'adjust-time-to-timezone' function as per
    let t = time_from_microseconds_with_tz(36000000000, -420);
    let result = adjust_time_to_timezone(t, 600);
    assert(hours_from_time(result) == 3);
    assert(minutes_from_time(result) == 0);
    assert(seconds_from_time(result) == 0);
    assert(
        duration_equal(
            timezone_from_time(result),
            duration_from_microseconds(36000000000),
        ),
    );
}
