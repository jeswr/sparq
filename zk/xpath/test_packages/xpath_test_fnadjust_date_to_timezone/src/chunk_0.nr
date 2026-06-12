//! Test chunk 0 for fn:adjust-date-to-timezone
//! Contains 6 tests

use dep::xpath::{
    adjust_date_to_timezone, adjust_date_to_timezone_none, date_from_epoch_days_with_tz,
    day_from_date, duration_equal, duration_from_microseconds, month_from_date, timezone_from_date,
    year_from_date,
};

#[test]
fn fn_adjust_date_to_timezone1args_2() {
    //  Evaluates The 'adjust-date-to-timezone' function with the arguments set as
    let d = date_from_epoch_days_with_tz(5068, 0);
    let result = adjust_date_to_timezone(d, -600);
    assert(year_from_date(result) == 1983);
    assert(month_from_date(result) == 11);
    assert(day_from_date(result) == 16);
    assert(
        duration_equal(
            timezone_from_date(result),
            duration_from_microseconds(-36000000000),
        ),
    );
}

#[test]
fn fn_adjust_date_to_timezone1args_3() {
    //  Evaluates The 'adjust-date-to-timezone' function with the arguments set as
    let d = date_from_epoch_days_with_tz(22279, 0);
    let result = adjust_date_to_timezone(d, -600);
    assert(year_from_date(result) == 2030);
    assert(month_from_date(result) == 12);
    assert(day_from_date(result) == 30);
    assert(
        duration_equal(
            timezone_from_date(result),
            duration_from_microseconds(-36000000000),
        ),
    );
}

#[test]
fn fn_adjust_date_to_timezone_1() {
    // Test Description: Evaluates The 'adjust-date-to-timezone' function as per
    let d = date_from_epoch_days_with_tz(11753, -300);
    let result = adjust_date_to_timezone(d, -300);
    assert(year_from_date(result) == 2002);
    assert(month_from_date(result) == 3);
    assert(day_from_date(result) == 7);
    assert(
        duration_equal(
            timezone_from_date(result),
            duration_from_microseconds(-18000000000),
        ),
    );
}

#[test]
fn fn_adjust_date_to_timezone_2() {
    // Test Description: Evaluates The 'adjust-date-to-timezone' function as per
    let d = date_from_epoch_days_with_tz(11753, -420);
    let result = adjust_date_to_timezone(d, -300);
    assert(year_from_date(result) == 2002);
    assert(month_from_date(result) == 3);
    assert(day_from_date(result) == 7);
    assert(
        duration_equal(
            timezone_from_date(result),
            duration_from_microseconds(-18000000000),
        ),
    );
}

#[test]
fn fn_adjust_date_to_timezone_5() {
    // Test Description: Evaluates The 'adjust-date-to-timezone' function as per
    let d = date_from_epoch_days_with_tz(11753, 0);
    let result = adjust_date_to_timezone_none(d);
    assert(year_from_date(result) == 2002);
    assert(month_from_date(result) == 3);
    assert(day_from_date(result) == 7);
}

#[test]
fn fn_adjust_date_to_timezone_6() {
    // Test Description: Evaluates The 'adjust-date-to-timezone' function as per
    let d = date_from_epoch_days_with_tz(11753, -420);
    let result = adjust_date_to_timezone_none(d);
    assert(year_from_date(result) == 2002);
    assert(month_from_date(result) == 3);
    assert(day_from_date(result) == 7);
}
