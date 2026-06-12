//! Test chunk 0 for fn:day-from-date
//! Contains 6 tests

use dep::xpath::{date_from_epoch_days_with_tz, day_from_date};

#[test]
fn fn_day_from_date1args_1() {
    //  Evaluates The 'day-from-date' function with the arguments set as follows: $arg
    let d = date_from_epoch_days_with_tz(0, 0);
    assert(day_from_date(d) == 1);
}

#[test]
fn fn_day_from_date1args_2() {
    //  Evaluates The 'day-from-date' function with the arguments set as follows: $arg
    let d = date_from_epoch_days_with_tz(5068, 0);
    assert(day_from_date(d) == 17);
}

#[test]
fn fn_day_from_date1args_3() {
    //  Evaluates The 'day-from-date' function with the arguments set as follows: $arg
    let d = date_from_epoch_days_with_tz(22279, 0);
    assert(day_from_date(d) == 31);
}

#[test]
fn fn_day_from_date_1() {
    //  Evaluates The 'day-from-date' function As per example 1 of the F&O specs
    let d = date_from_epoch_days_with_tz(10742, -300);
    assert(day_from_date(d) == 31);
}

#[test]
fn fn_day_from_date_2() {
    //  Evaluates The 'day-from-date' function As per example 2 of the F&O specs
    let d = date_from_epoch_days_with_tz(10957, 300);
    assert(day_from_date(d) == 1);
}

#[test]
fn fn_day_from_date_6() {
    //  Evaluates The 'day-from-date' function that returns 31.
    let d = date_from_epoch_days_with_tz(10742, 0);
    assert(day_from_date(d) == 31);
}
