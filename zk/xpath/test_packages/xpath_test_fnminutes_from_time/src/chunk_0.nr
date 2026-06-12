//! Test chunk 0 for fn:minutes-from-time
//! Contains 4 tests

use dep::xpath::{minutes_from_time, time_from_microseconds_with_tz};

#[test]
fn fn_minutes_from_time1args_1() {
    //  Evaluates The 'minutes-from-time' function with the arguments set as follows:
    let t = time_from_microseconds_with_tz(0, 0);
    assert(minutes_from_time(t) == 0);
}

#[test]
fn fn_minutes_from_time1args_2() {
    //  Evaluates The 'minutes-from-time' function with the arguments set as follows:
    let t = time_from_microseconds_with_tz(29015000000, 0);
    assert(minutes_from_time(t) == 3);
}

#[test]
fn fn_minutes_from_time1args_3() {
    //  Evaluates The 'minutes-from-time' function with the arguments set as follows:
    let t = time_from_microseconds_with_tz(86399000000, 0);
    assert(minutes_from_time(t) == 59);
}

#[test]
fn fn_minutes_from_time_1() {
    //  Evaluates The 'minutes-from-time' function As per example 1 of the F&O specs
    let t = time_from_microseconds_with_tz(46800000000, 0);
    assert(minutes_from_time(t) == 0);
}
