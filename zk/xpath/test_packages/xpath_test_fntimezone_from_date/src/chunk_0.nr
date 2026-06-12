//! Test chunk 0 for fn:timezone-from-date
//! Contains 4 tests

use dep::xpath::{
    date_from_epoch_days_with_tz, duration_equal, duration_from_microseconds, timezone_from_date,
};

#[test]
fn fn_timezone_from_date_1() {
    //  Evaluates The 'timezone-from-date' function As per example 1 of the F&O specs
    let d = date_from_epoch_days_with_tz(10742, -300);
    assert(
        duration_equal(
            timezone_from_date(d),
            duration_from_microseconds(-18000000000),
        ),
    );
}

#[test]
fn fn_timezone_from_date_2() {
    //  Evaluates The 'timezone-from-date' function As per example 2 of the F&O specs.
    let d = date_from_epoch_days_with_tz(11120, 0);
    assert(duration_equal(timezone_from_date(d), duration_from_microseconds(0)));
}

#[test]
fn fn_timezone_from_date_6() {
    //  Evaluates The 'timezone-from-date' function that returns a dayTimeDuration of
    let d = date_from_epoch_days_with_tz(10742, 0);
    assert(duration_equal(timezone_from_date(d), duration_from_microseconds(0)));
}

#[test]
fn fn_timezone_from_date_7() {
    //  Evaluates The 'timezone-from-date' function that uses a timezone of -0.
    let d = date_from_epoch_days_with_tz(10956, 0);
    assert(duration_equal(timezone_from_date(d), duration_from_microseconds(0)));
}
