//! Test chunk 0 for fn:year-from-dateTime
//! Contains 6 tests

use dep::xpath::{datetime_from_epoch_microseconds_with_tz, year_from_datetime};

#[test]
fn fn_year_from_datetime1args_1() {
    //  Evaluates The 'year-from-dateTime' function with the arguments set as follows:
    let dt = datetime_from_epoch_microseconds_with_tz(0, 0);
    assert(year_from_datetime(dt) == 1970);
}

#[test]
fn fn_year_from_datetime1args_2() {
    //  Evaluates The 'year-from-dateTime' function with the arguments set as follows:
    let dt = datetime_from_epoch_microseconds_with_tz(828841252000000, 0);
    assert(year_from_datetime(dt) == 1996);
}

#[test]
fn fn_year_from_datetime1args_3() {
    //  Evaluates The 'year-from-dateTime' function with the arguments set as follows:
    let dt = datetime_from_epoch_microseconds_with_tz(1924991999000000, 0);
    assert(year_from_datetime(dt) == 2030);
}

#[test]
fn fn_year_from_datetime_1() {
    //  Evaluates The 'year-from-dateTime' function As per example 1 (of this
    let dt = datetime_from_epoch_microseconds_with_tz(928174800000000, -300);
    assert(year_from_datetime(dt) == 1999);
}

#[test]
fn fn_year_from_datetime_2() {
    //  Evaluates The 'year-from-dateTime' function As per example 2 (for this
    let dt = datetime_from_epoch_microseconds_with_tz(928204200000000, -300);
    assert(year_from_datetime(dt) == 1999);
}

#[test]
fn fn_year_from_datetime_3() {
    //  Evaluates The 'year-from-dateTime' function As per example 3 (for this
    let dt = datetime_from_epoch_microseconds_with_tz(946668000000000, 0);
    assert(year_from_datetime(dt) == 1999);
}
