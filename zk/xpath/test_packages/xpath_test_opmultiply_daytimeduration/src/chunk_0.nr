//! Test chunk 0 for op:multiply-dayTimeDuration
//! Contains 3 tests

use dep::xpath::{duration_equal, duration_from_microseconds, duration_multiply};

#[test]
fn op_multiply_daytimeduration2args_2() {
    //  ******************************************************* Test:
    let dur = duration_from_microseconds(1339199000000);
    assert(duration_equal(duration_multiply(dur, 0), duration_from_microseconds(0)));
}

#[test]
fn op_multiply_daytimeduration2args_3() {
    //  ******************************************************* Test:
    let dur = duration_from_microseconds(2764799000000);
    assert(duration_equal(duration_multiply(dur, 0), duration_from_microseconds(0)));
}

#[test]
fn op_multiply_daytimeduration2args_4() {
    //  ******************************************************* Test:
    let dur = duration_from_microseconds(0);
    assert(duration_equal(duration_multiply(dur, 0), duration_from_microseconds(0)));
}
