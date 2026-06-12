//! Test chunk 0 for op:divide-dayTimeDuration-by-dayTimeDuration
//! Contains 5 tests

use dep::xpath::{duration_divide_by_duration, duration_from_microseconds};

#[test]
fn op_divide_daytimeduration_by_daytimeduration2args_1() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(1000000);
    assert(duration_divide_by_duration(dur1, dur2) == 0);
}

#[test]
fn op_divide_daytimeduration_by_daytimeduration2args_2() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(1339199000000);
    let dur2 = duration_from_microseconds(1000000);
    assert(duration_divide_by_duration(dur1, dur2) == 1339199);
}

#[test]
fn op_divide_daytimeduration_by_daytimeduration2args_3() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(2764799000000);
    let dur2 = duration_from_microseconds(1000000);
    assert(duration_divide_by_duration(dur1, dur2) == 2764799);
}

#[test]
fn op_divide_daytimeduration_by_daytimeduration2args_4() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(1339199000000);
    assert(duration_divide_by_duration(dur1, dur2) == 0);
}

#[test]
fn op_divide_daytimeduration_by_daytimeduration2args_5() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(2764799000000);
    assert(duration_divide_by_duration(dur1, dur2) == 0);
}
