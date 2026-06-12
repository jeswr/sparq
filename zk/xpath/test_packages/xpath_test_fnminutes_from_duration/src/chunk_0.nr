//! Test chunk 0 for fn:minutes-from-duration
//! Contains 6 tests

use dep::xpath::{duration_from_microseconds, minutes_from_duration};

#[test]
fn fn_minutes_from_duration1args_1() {
    //  Evaluates The 'minutes-from-duration' function with the arguments set as
    let dur = duration_from_microseconds(0);
    assert(minutes_from_duration(dur) == 0);
}

#[test]
fn fn_minutes_from_duration1args_2() {
    //  Evaluates The 'minutes-from-duration' function with the arguments set as
    let dur = duration_from_microseconds(1339199000000);
    assert(minutes_from_duration(dur) == 59);
}

#[test]
fn fn_minutes_from_duration1args_3() {
    //  Evaluates The 'minutes-from-duration' function with the arguments set as
    let dur = duration_from_microseconds(2764799000000);
    assert(minutes_from_duration(dur) == 59);
}

#[test]
fn fn_minutes_from_duration_1() {
    //  Evaluates The 'minutes-from-duration' function As per example 1 (for this
    let dur = duration_from_microseconds(295200000000);
    assert(minutes_from_duration(dur) == 0);
}

#[test]
fn fn_minutes_from_duration_6() {
    //  Evaluates The 'minutes-from-duration' function that returns 1.
    let dur = duration_from_microseconds(90060000000);
    assert(minutes_from_duration(dur) == 1);
}

#[test]
fn fn_minutes_from_duration_21() {
    //  Evaluates The 'minutes-from-duration' function to evaluate normalization of
    let dur = duration_from_microseconds(1854300000000);
    assert(minutes_from_duration(dur) == 5);
}
