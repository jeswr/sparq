//! Test chunk 0 for fn:hours-from-time
//! Contains 9 tests

use dep::xpath::{hours_from_time, time_from_microseconds_with_tz};

#[test]
fn fn_hours_from_time1args_1() {
    //  Evaluates The 'hours-from-time' function with the arguments set as follows:
    let t = time_from_microseconds_with_tz(0, 0);
    assert(hours_from_time(t) == 0);
}

#[test]
fn fn_hours_from_time1args_2() {
    //  Evaluates The 'hours-from-time' function with the arguments set as follows:
    let t = time_from_microseconds_with_tz(29015000000, 0);
    assert(hours_from_time(t) == 8);
}

#[test]
fn fn_hours_from_time1args_3() {
    //  Evaluates The 'hours-from-time' function with the arguments set as follows:
    let t = time_from_microseconds_with_tz(86399000000, 0);
    assert(hours_from_time(t) == 23);
}

#[test]
fn fn_hours_from_time_1() {
    //  Evaluates The 'hours-from-time' function As per example 1 of the F&O specs
    let t = time_from_microseconds_with_tz(40980000000, 0);
    assert(hours_from_time(t) == 11);
}

#[test]
fn fn_hours_from_time_2() {
    //  Evaluates The 'hours-from-time' function As per example 2 of the F&O specs
    let t = time_from_microseconds_with_tz(76980000000, 0);
    assert(hours_from_time(t) == 21);
}

#[test]
fn fn_hours_from_time_3() {
    //  Evaluates The 'hours-from-time' function As per example 3 of the F&O specs
    let t = time_from_microseconds_with_tz(4980000000, 300);
    assert(hours_from_time(t) == 1);
}

#[test]
fn fn_hours_from_time_4() {
    //  Evaluates The 'hours-from-time' function As per example 4 of the F&O specs
    let t = time_from_microseconds_with_tz(73380000000, 0);
    assert(hours_from_time(t) == 20);
}

#[test]
fn fn_hours_from_time_6() {
    //  Evaluates The 'hours-from-time' function that returns 0.
    let t = time_from_microseconds_with_tz(1200000000, 0);
    assert(hours_from_time(t) == 0);
}

#[test]
fn fn_hours_from_time_7() {
    //  Evaluates The 'hours-from-time' function that returns 23.
    let t = time_from_microseconds_with_tz(84000000000, 0);
    assert(hours_from_time(t) == 23);
}
