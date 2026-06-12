//! Test chunk 0 for fn:adjust-dateTime-to-timezone
//! Contains 9 tests

use dep::xpath::{
    adjust_datetime_to_timezone, adjust_datetime_to_timezone_none,
    datetime_from_epoch_microseconds_with_tz, day_from_datetime, duration_equal,
    duration_from_microseconds, hours_from_datetime, minutes_from_datetime, month_from_datetime,
    seconds_from_datetime, timezone_from_datetime, year_from_datetime,
};

#[test]
fn fn_adjust_datetime_to_timezone1args_2() {
    //  Evaluates The 'adjust-dateTime-to-timezone' function with the arguments set as
    let dt = datetime_from_epoch_microseconds_with_tz(828841252000000, 0);
    let result = adjust_datetime_to_timezone(dt, -600);
    assert(year_from_datetime(result) == 1996);
    assert(month_from_datetime(result) == 4);
    assert(day_from_datetime(result) == 6);
    assert(hours_from_datetime(result) == 15);
    assert(minutes_from_datetime(result) == 40);
    assert(seconds_from_datetime(result) == 52);
    assert(duration_equal(
        timezone_from_datetime(result),
        duration_from_microseconds(-36000000000),
    ));
}

#[test]
fn fn_adjust_datetime_to_timezone1args_3() {
    //  Evaluates The 'adjust-dateTime-to-timezone' function with the arguments set as
    let dt = datetime_from_epoch_microseconds_with_tz(1924991999000000, 0);
    let result = adjust_datetime_to_timezone(dt, -600);
    assert(year_from_datetime(result) == 2030);
    assert(month_from_datetime(result) == 12);
    assert(day_from_datetime(result) == 31);
    assert(hours_from_datetime(result) == 13);
    assert(minutes_from_datetime(result) == 59);
    assert(seconds_from_datetime(result) == 59);
    assert(duration_equal(
        timezone_from_datetime(result),
        duration_from_microseconds(-36000000000),
    ));
}

#[test]
fn fn_adjust_datetime_to_timezone_1() {
    // Test Description: Evaluates The 'adjust-dateTime-to-timezone' function as per
    let dt = datetime_from_epoch_microseconds_with_tz(1015513200000000, -300);
    let result = adjust_datetime_to_timezone(dt, -300);
    assert(year_from_datetime(result) == 2002);
    assert(month_from_datetime(result) == 3);
    assert(day_from_datetime(result) == 7);
    assert(hours_from_datetime(result) == 10);
    assert(minutes_from_datetime(result) == 0);
    assert(seconds_from_datetime(result) == 0);
    assert(duration_equal(
        timezone_from_datetime(result),
        duration_from_microseconds(-18000000000),
    ));
}

#[test]
fn fn_adjust_datetime_to_timezone_2() {
    // Test Description: Evaluates The 'adjust-dateTime-to-timezone' function as per
    let dt = datetime_from_epoch_microseconds_with_tz(1015520400000000, -420);
    let result = adjust_datetime_to_timezone(dt, -300);
    assert(year_from_datetime(result) == 2002);
    assert(month_from_datetime(result) == 3);
    assert(day_from_datetime(result) == 7);
    assert(hours_from_datetime(result) == 12);
    assert(minutes_from_datetime(result) == 0);
    assert(seconds_from_datetime(result) == 0);
    assert(duration_equal(
        timezone_from_datetime(result),
        duration_from_microseconds(-18000000000),
    ));
}

#[test]
fn fn_adjust_datetime_to_timezone_5() {
    // Test Description: Evaluates The 'adjust-dateTime-to-timezone' function as per
    let dt = datetime_from_epoch_microseconds_with_tz(1015520400000000, -420);
    let result = adjust_datetime_to_timezone(dt, 600);
    assert(year_from_datetime(result) == 2002);
    assert(month_from_datetime(result) == 3);
    assert(day_from_datetime(result) == 8);
    assert(hours_from_datetime(result) == 3);
    assert(minutes_from_datetime(result) == 0);
    assert(seconds_from_datetime(result) == 0);
    assert(
        duration_equal(
            timezone_from_datetime(result),
            duration_from_microseconds(36000000000),
        ),
    );
}

#[test]
fn fn_adjust_datetime_to_timezone_6() {
    // Test Description: Evaluates The 'adjust-dateTime-to-timezone' function as per
    let dt = datetime_from_epoch_microseconds_with_tz(1015455600000000, 60);
    let result = adjust_datetime_to_timezone(dt, -480);
    assert(year_from_datetime(result) == 2002);
    assert(month_from_datetime(result) == 3);
    assert(day_from_datetime(result) == 6);
    assert(hours_from_datetime(result) == 15);
    assert(minutes_from_datetime(result) == 0);
    assert(seconds_from_datetime(result) == 0);
    assert(duration_equal(
        timezone_from_datetime(result),
        duration_from_microseconds(-28800000000),
    ));
}

#[test]
fn fn_adjust_datetime_to_timezone_7() {
    // Test Description: Evaluates The 'adjust-dateTime-to-timezone' function as per
    let dt = datetime_from_epoch_microseconds_with_tz(1015495200000000, 0);
    let result = adjust_datetime_to_timezone_none(dt);
    assert(year_from_datetime(result) == 2002);
    assert(month_from_datetime(result) == 3);
    assert(day_from_datetime(result) == 7);
    assert(hours_from_datetime(result) == 10);
    assert(minutes_from_datetime(result) == 0);
    assert(seconds_from_datetime(result) == 0);
}

#[test]
fn fn_adjust_datetime_to_timezone_8() {
    // Test Description: Evaluates The 'adjust-dateTime-to-timezone' function as per
    let dt = datetime_from_epoch_microseconds_with_tz(1015520400000000, -420);
    let result = adjust_datetime_to_timezone_none(dt);
    assert(year_from_datetime(result) == 2002);
    assert(month_from_datetime(result) == 3);
    assert(day_from_datetime(result) == 7);
    assert(hours_from_datetime(result) == 10);
    assert(minutes_from_datetime(result) == 0);
    assert(seconds_from_datetime(result) == 0);
}

#[test]
fn k2_adjdatetimetotimezonefunc_1() {
    //  Ensure dateTime values are properly normalized.
    let dt = datetime_from_epoch_microseconds_with_tz(946684800000000, 0);
    let result = adjust_datetime_to_timezone_none(dt);
    assert(year_from_datetime(result) == 2000);
    assert(month_from_datetime(result) == 1);
    assert(day_from_datetime(result) == 1);
    assert(hours_from_datetime(result) == 0);
    assert(minutes_from_datetime(result) == 0);
    assert(seconds_from_datetime(result) == 0);
}
