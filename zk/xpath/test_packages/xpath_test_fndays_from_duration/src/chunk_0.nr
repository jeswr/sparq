//! Test chunk 0 for fn:days-from-duration
//! Contains 6 tests

use dep::xpath::{days_from_duration, duration_from_microseconds};

#[test]
fn fn_days_from_duration1args_1() {
    //  Evaluates The 'days-from-duration' function with the arguments set as follows:
    let dur = duration_from_microseconds(0);
    assert(days_from_duration(dur) == 0);
}

#[test]
fn fn_days_from_duration1args_2() {
    //  Evaluates The 'days-from-duration' function with the arguments set as follows:
    let dur = duration_from_microseconds(1339199000000);
    assert(days_from_duration(dur) == 15);
}

#[test]
fn fn_days_from_duration1args_3() {
    //  Evaluates The 'days-from-duration' function with the arguments set as follows:
    let dur = duration_from_microseconds(2764799000000);
    assert(days_from_duration(dur) == 31);
}

#[test]
fn fn_days_from_duration_1() {
    //  Evaluates The 'days-from-duration' function As per example 1 (for this
    let dur = duration_from_microseconds(295200000000);
    assert(days_from_duration(dur) == 3);
}

#[test]
fn fn_days_from_duration_2() {
    //  Evaluates The 'days-from-duration' function As per example 2 (for this
    let dur = duration_from_microseconds(457200000000);
    assert(days_from_duration(dur) == 5);
}

#[test]
fn fn_days_from_duration_6() {
    //  Evaluates The 'days-from-duration' function that returns 1.
    let dur = duration_from_microseconds(90000000000);
    assert(days_from_duration(dur) == 1);
}
