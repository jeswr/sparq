//! Test chunk 0 for fn:hours-from-duration
//! Contains 7 tests

use dep::xpath::{duration_from_microseconds, hours_from_duration};

#[test]
fn fn_hours_from_duration1args_1() {
    //  Evaluates The 'hours-from-duration' function with the arguments set as
    let dur = duration_from_microseconds(0);
    assert(hours_from_duration(dur) == 0);
}

#[test]
fn fn_hours_from_duration1args_2() {
    //  Evaluates The 'hours-from-duration' function with the arguments set as
    let dur = duration_from_microseconds(1339199000000);
    assert(hours_from_duration(dur) == 11);
}

#[test]
fn fn_hours_from_duration1args_3() {
    //  Evaluates The 'hours-from-duration' function with the arguments set as
    let dur = duration_from_microseconds(2764799000000);
    assert(hours_from_duration(dur) == 23);
}

#[test]
fn fn_hours_from_duration_1() {
    //  Evaluates The 'hours-from-duration' function As per example 1 (for this
    let dur = duration_from_microseconds(295200000000);
    assert(hours_from_duration(dur) == 10);
}

#[test]
fn fn_hours_from_duration_2() {
    //  Evaluates The 'hours-from-duration' function As per example 2 (for this
    let dur = duration_from_microseconds(304332000000);
    assert(hours_from_duration(dur) == 12);
}

#[test]
fn fn_hours_from_duration_3() {
    //  Evaluates The 'hours-from-duration' function as per example 3 of this function
    let dur = duration_from_microseconds(442800000000);
    assert(hours_from_duration(dur) == 3);
}

#[test]
fn fn_hours_from_duration_6() {
    //  Evaluates The 'hours-from-duration' function that returns 1.
    let dur = duration_from_microseconds(90000000000);
    assert(hours_from_duration(dur) == 1);
}
