//! Test chunk 0 for fn:year-from-date
//! Contains 5 tests

use dep::xpath::{date_from_epoch_days_with_tz, year_from_date};

#[test]
fn fn_year_from_date1args_1() {
    //  Evaluates The 'year-from-date' function with the arguments set as follows:
    let d = date_from_epoch_days_with_tz(0, 0);
    assert(year_from_date(d) == 1970);
}

#[test]
fn fn_year_from_date1args_2() {
    //  Evaluates The 'year-from-date' function with the arguments set as follows:
    let d = date_from_epoch_days_with_tz(5068, 0);
    assert(year_from_date(d) == 1983);
}

#[test]
fn fn_year_from_date1args_3() {
    //  Evaluates The 'year-from-date' function with the arguments set as follows:
    let d = date_from_epoch_days_with_tz(22279, 0);
    assert(year_from_date(d) == 2030);
}

#[test]
fn fn_year_from_date_1() {
    //  Evaluates The 'year-from-date' function As per example 1 of the F&O specs
    let d = date_from_epoch_days_with_tz(10742, 0);
    assert(year_from_date(d) == 1999);
}

#[test]
fn fn_year_from_date_2() {
    //  Evaluates The 'year-from-date' function As per example 2 of the F&O specs
    let d = date_from_epoch_days_with_tz(10957, 0);
    assert(year_from_date(d) == 2000);
}
