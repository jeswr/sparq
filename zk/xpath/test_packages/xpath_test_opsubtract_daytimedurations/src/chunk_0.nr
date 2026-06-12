//! Test chunk 0 for op:subtract-dayTimeDurations
//! Contains 6 tests

use dep::xpath::{duration_equal, duration_from_microseconds, duration_subtract};

#[test]
fn op_subtract_daytimedurations2args_1() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(0);
    assert(duration_equal(duration_subtract(dur1, dur2), duration_from_microseconds(0)));
}

#[test]
fn op_subtract_daytimedurations2args_2() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(1339199000000);
    let dur2 = duration_from_microseconds(0);
    assert(duration_equal(
        duration_subtract(dur1, dur2),
        duration_from_microseconds(1339199000000),
    ));
}

#[test]
fn op_subtract_daytimedurations2args_3() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(2764799000000);
    let dur2 = duration_from_microseconds(0);
    assert(duration_equal(
        duration_subtract(dur1, dur2),
        duration_from_microseconds(2764799000000),
    ));
}

#[test]
fn op_subtract_daytimedurations2args_4() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(1339199000000);
    assert(duration_equal(
        duration_subtract(dur1, dur2),
        duration_from_microseconds(-1339199000000),
    ));
}

#[test]
fn op_subtract_daytimedurations2args_5() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(2764799000000);
    assert(duration_equal(
        duration_subtract(dur1, dur2),
        duration_from_microseconds(-2764799000000),
    ));
}

#[test]
fn op_subtract_daytimedurations_1() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(216000000000);
    let dur2 = duration_from_microseconds(124200000000);
    assert(
        duration_equal(
            duration_subtract(dur1, dur2),
            duration_from_microseconds(91800000000),
        ),
    );
}
