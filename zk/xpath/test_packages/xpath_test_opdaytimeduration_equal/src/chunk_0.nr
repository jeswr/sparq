//! Test chunk 0 for op:dayTimeDuration-equal
//! Contains 7 tests

use dep::xpath::{duration_equal, duration_from_microseconds};

#[test]
fn op_daytimeduration_equal2args_1() {
    //  Evaluates The 'op:dayTimeDuration-equal' operator with the arguments set as
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(0);
    assert(duration_equal(dur1, dur2) == true);
}

#[test]
fn op_daytimeduration_equal2args_2() {
    //  Evaluates The 'op:dayTimeDuration-equal' operator with the arguments set as
    let dur1 = duration_from_microseconds(1339199000000);
    let dur2 = duration_from_microseconds(0);
    assert(duration_equal(dur1, dur2) == false);
}

#[test]
fn op_daytimeduration_equal2args_3() {
    //  Evaluates The 'op:dayTimeDuration-equal' operator with the arguments set as
    let dur1 = duration_from_microseconds(2764799000000);
    let dur2 = duration_from_microseconds(0);
    assert(duration_equal(dur1, dur2) == false);
}

#[test]
fn op_daytimeduration_equal2args_4() {
    //  Evaluates The 'op:dayTimeDuration-equal' operator with the arguments set as
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(1339199000000);
    assert(duration_equal(dur1, dur2) == false);
}

#[test]
fn op_daytimeduration_equal2args_5() {
    //  Evaluates The 'op:dayTimeDuration-equal' operator with the arguments set as
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(2764799000000);
    assert(duration_equal(dur1, dur2) == false);
}

#[test]
fn k_daytimedurationeq_2() {
    // Testing 'eq' involving xs:dayTimeDuration with two zeroed values.
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(0);
    assert(duration_equal(dur1, dur2) == true);
}

#[test]
fn k_daytimedurationeq_6() {
    // The xs:dayTimeDuration values -PT0S and PT0S are equal.
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(0);
    assert(duration_equal(dur1, dur2) == true);
}
