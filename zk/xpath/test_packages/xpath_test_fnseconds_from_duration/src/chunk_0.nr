//! Test chunk 0 for fn:seconds-from-duration
//! Contains 5 tests

use dep::xpath::{duration_from_microseconds, seconds_from_duration};

#[test]
fn fn_seconds_from_duration1args_1() {
    //  Evaluates The 'seconds-from-duration' function with the arguments set as
    let dur = duration_from_microseconds(0);
    assert(seconds_from_duration(dur) == 0);
}

#[test]
fn fn_seconds_from_duration1args_2() {
    //  Evaluates The 'seconds-from-duration' function with the arguments set as
    let dur = duration_from_microseconds(1339199000000);
    assert(seconds_from_duration(dur) == 59);
}

#[test]
fn fn_seconds_from_duration1args_3() {
    //  Evaluates The 'seconds-from-duration' function with the arguments set as
    let dur = duration_from_microseconds(2764799000000);
    assert(seconds_from_duration(dur) == 59);
}

#[test]
fn fn_seconds_from_duration_6() {
    //  Evaluates The 'seconds-from-duration' function that returns 1.
    let dur = duration_from_microseconds(90061000000);
    assert(seconds_from_duration(dur) == 1);
}

#[test]
fn fn_seconds_from_duration_21() {
    //  Evaluates The 'seconds-from-duration' function to evaluate normalization of
    let dur = duration_from_microseconds(1851090000000);
    assert(seconds_from_duration(dur) == 30);
}
