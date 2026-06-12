//! Test chunk 0 for op:subtract-times
//! Contains 8 tests

use dep::xpath::{
    duration_equal, duration_from_microseconds, subtract_times, time_from_microseconds_with_tz,
};

#[test]
fn op_subtract_times2args_1() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(0, 0);
    let t2 = time_from_microseconds_with_tz(0, 0);
    assert(duration_equal(subtract_times(t1, t2), duration_from_microseconds(0)));
}

#[test]
fn op_subtract_times2args_2() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(29015000000, 0);
    let t2 = time_from_microseconds_with_tz(0, 0);
    assert(
        duration_equal(
            subtract_times(t1, t2),
            duration_from_microseconds(29015000000),
        ),
    );
}

#[test]
fn op_subtract_times2args_3() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(86399000000, 0);
    let t2 = time_from_microseconds_with_tz(0, 0);
    assert(
        duration_equal(
            subtract_times(t1, t2),
            duration_from_microseconds(86399000000),
        ),
    );
}

#[test]
fn op_subtract_times2args_4() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(0, 0);
    let t2 = time_from_microseconds_with_tz(29015000000, 0);
    assert(
        duration_equal(
            subtract_times(t1, t2),
            duration_from_microseconds(-29015000000),
        ),
    );
}

#[test]
fn op_subtract_times2args_5() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(0, 0);
    let t2 = time_from_microseconds_with_tz(86399000000, 0);
    assert(
        duration_equal(
            subtract_times(t1, t2),
            duration_from_microseconds(-86399000000),
        ),
    );
}

#[test]
fn op_subtract_times_1() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(40320000000, 0);
    let t2 = time_from_microseconds_with_tz(14400000000, 0);
    assert(
        duration_equal(
            subtract_times(t1, t2),
            duration_from_microseconds(25920000000),
        ),
    );
}

#[test]
fn op_subtract_times_2() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(39600000000, -300);
    let t2 = time_from_microseconds_with_tz(77400000000, 330);
    assert(duration_equal(subtract_times(t1, t2), duration_from_microseconds(0)));
}

#[test]
fn op_subtract_times_3() {
    //  ******************************************************* Test:
    let t1 = time_from_microseconds_with_tz(61200000000, -360);
    let t2 = time_from_microseconds_with_tz(28800000000, 540);
    assert(
        duration_equal(
            subtract_times(t1, t2),
            duration_from_microseconds(86400000000),
        ),
    );
}
