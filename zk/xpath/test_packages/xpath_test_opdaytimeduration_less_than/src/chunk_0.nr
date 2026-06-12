//! Test chunk 0 for op:dayTimeDuration-less-than
//! Contains 5 tests

use dep::xpath::{duration_from_microseconds, duration_less_than};

#[test]
fn op_daytimeduration_less_than2args_1() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(0);
    assert(duration_less_than(dur1, dur2) == false);
}

#[test]
fn op_daytimeduration_less_than2args_2() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(1339199000000);
    let dur2 = duration_from_microseconds(0);
    assert(duration_less_than(dur1, dur2) == false);
}

#[test]
fn op_daytimeduration_less_than2args_3() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(2764799000000);
    let dur2 = duration_from_microseconds(0);
    assert(duration_less_than(dur1, dur2) == false);
}

#[test]
fn op_daytimeduration_less_than2args_4() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(1339199000000);
    assert(duration_less_than(dur1, dur2) == true);
}

#[test]
fn op_daytimeduration_less_than2args_5() {
    //  ******************************************************* Test:
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(2764799000000);
    assert(duration_less_than(dur1, dur2) == true);
}
