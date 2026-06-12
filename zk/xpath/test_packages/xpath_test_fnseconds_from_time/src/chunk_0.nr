//! Test chunk 0 for fn:seconds-from-time
//! Contains 4 tests

use dep::xpath::{seconds_from_time, time_from_microseconds_with_tz};

#[test]
fn fn_seconds_from_time1args_1() {
    //  Evaluates The 'seconds-from-time' function with the arguments set as follows:
    let t = time_from_microseconds_with_tz(0, 0);
    assert(seconds_from_time(t) == 0);
}

#[test]
fn fn_seconds_from_time1args_2() {
    //  Evaluates The 'seconds-from-time' function with the arguments set as follows:
    let t = time_from_microseconds_with_tz(29015000000, 0);
    assert(seconds_from_time(t) == 35);
}

#[test]
fn fn_seconds_from_time1args_3() {
    //  Evaluates The 'seconds-from-time' function with the arguments set as follows:
    let t = time_from_microseconds_with_tz(86399000000, 0);
    assert(seconds_from_time(t) == 59);
}

#[test]
fn fn_seconds_from_time_7() {
    //  Evaluates The 'seconds-from-time' function with the argument set to return 59.
    let t = time_from_microseconds_with_tz(84059000000, 0);
    assert(seconds_from_time(t) == 59);
}
