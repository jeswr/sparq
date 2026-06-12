//! Test chunk 0 for fn:month-from-date
//! Contains 7 tests

use dep::xpath::{date_from_epoch_days_with_tz, month_from_date};

#[test]
fn fn_month_from_date1args_1() {
    //  Evaluates The 'month-from-date' function with the arguments set as follows:
    let d = date_from_epoch_days_with_tz(0, 0);
    assert(month_from_date(d) == 1);
}

#[test]
fn fn_month_from_date1args_2() {
    //  Evaluates The 'month-from-date' function with the arguments set as follows:
    let d = date_from_epoch_days_with_tz(5068, 0);
    assert(month_from_date(d) == 11);
}

#[test]
fn fn_month_from_date1args_3() {
    //  Evaluates The 'month-from-date' function with the arguments set as follows:
    let d = date_from_epoch_days_with_tz(22279, 0);
    assert(month_from_date(d) == 12);
}

#[test]
fn fn_month_from_date_1() {
    //  Evaluates The 'month-from-date' function As per example 1 of the F&O specs
    let d = date_from_epoch_days_with_tz(10742, -300);
    assert(month_from_date(d) == 5);
}

#[test]
fn fn_month_from_date_2() {
    //  Evaluates The 'month-from-date' function As per example 2 of the F&O specs
    let d = date_from_epoch_days_with_tz(10957, 300);
    assert(month_from_date(d) == 1);
}

#[test]
fn fn_month_from_date_6() {
    //  Evaluates The 'month-from-date' function that returns 1.
    let d = date_from_epoch_days_with_tz(30, 0);
    assert(month_from_date(d) == 1);
}

#[test]
fn fn_month_from_date_7() {
    //  Evaluates The 'month-from-date' function that returns 12.
    let d = date_from_epoch_days_with_tz(10956, 0);
    assert(month_from_date(d) == 12);
}
