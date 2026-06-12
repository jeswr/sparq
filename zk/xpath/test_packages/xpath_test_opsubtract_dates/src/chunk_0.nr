//! Test chunk 0 for op:subtract-dates
//! Contains 3 tests

use dep::xpath::{
    date_from_epoch_days_with_tz, duration_equal, duration_from_microseconds, subtract_dates,
};

#[test]
fn op_subtract_dates_yielding_dtd_1() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(11260, 0);
    let d2 = date_from_epoch_days_with_tz(10923, 0);
    assert(
        duration_equal(
            subtract_dates(d1, d2),
            duration_from_microseconds(29116800000000),
        ),
    );
}

#[test]
fn op_subtract_dates_yielding_dtd_2() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(11260, 300);
    let d2 = date_from_epoch_days_with_tz(10923, 0);
    assert(
        duration_equal(
            subtract_dates(d1, d2),
            duration_from_microseconds(29098800000000),
        ),
    );
}

#[test]
fn op_subtract_dates_yielding_dtd_3() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(11245, -300);
    let d2 = date_from_epoch_days_with_tz(11240, 120);
    assert(
        duration_equal(
            subtract_dates(d1, d2),
            duration_from_microseconds(457200000000),
        ),
    );
}
