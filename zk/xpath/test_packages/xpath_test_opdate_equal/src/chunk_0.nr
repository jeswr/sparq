//! Test chunk 0 for op:date-equal
//! Contains 14 tests

use dep::xpath::{date_equal, date_from_epoch_days_with_tz};

#[test]
fn op_date_equal2args_1() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(0, 0);
    let d2 = date_from_epoch_days_with_tz(0, 0);
    assert(date_equal(d1, d2) == true);
}

#[test]
fn op_date_equal2args_2() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(5068, 0);
    let d2 = date_from_epoch_days_with_tz(0, 0);
    assert(date_equal(d1, d2) == false);
}

#[test]
fn op_date_equal2args_3() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(22279, 0);
    let d2 = date_from_epoch_days_with_tz(0, 0);
    assert(date_equal(d1, d2) == false);
}

#[test]
fn op_date_equal2args_4() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(0, 0);
    let d2 = date_from_epoch_days_with_tz(5068, 0);
    assert(date_equal(d1, d2) == false);
}

#[test]
fn op_date_equal2args_5() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(0, 0);
    let d2 = date_from_epoch_days_with_tz(22279, 0);
    assert(date_equal(d1, d2) == false);
}

#[test]
fn k_dateeq_1() {
    //  *******************************************************  Test: K-DateEQ-1
    let d1 = date_from_epoch_days_with_tz(12642, 0);
    let d2 = date_from_epoch_days_with_tz(12642, 0);
    assert(date_equal(d1, d2) == true);
}

#[test]
fn k_dateeq_5() {
    //  *******************************************************  Test: K-DateEQ-5
    let d1 = date_from_epoch_days_with_tz(10929, 0);
    let d2 = date_from_epoch_days_with_tz(10929, 0);
    assert(date_equal(d1, d2) == true);
}

#[test]
fn k_dateeq_6() {
    //  *******************************************************  Test: K-DateEQ-6
    let d1 = date_from_epoch_days_with_tz(10929, 0);
    let d2 = date_from_epoch_days_with_tz(10929, 0);
    assert(date_equal(d1, d2) == true);
}

#[test]
fn k_dateeq_7() {
    //  *******************************************************  Test: K-DateEQ-7
    let d1 = date_from_epoch_days_with_tz(10929, 0);
    let d2 = date_from_epoch_days_with_tz(10929, 0);
    assert(date_equal(d1, d2) == true);
}

#[test]
fn k_dateeq_8() {
    //  *******************************************************  Test: K-DateEQ-8
    let d1 = date_from_epoch_days_with_tz(10929, 0);
    let d2 = date_from_epoch_days_with_tz(10929, 0);
    assert(date_equal(d1, d2) == true);
}

#[test]
fn cbcl_date_equal_002() {
    //  test comparison of date
    let d1 = date_from_epoch_days_with_tz(13908, 0);
    let d2 = date_from_epoch_days_with_tz(13909, 540);
    assert(date_equal(d1, d2) == false);
}

#[test]
fn cbcl_date_equal_003() {
    //  test comparison of date
    let d1 = date_from_epoch_days_with_tz(13909, 540);
    let d2 = date_from_epoch_days_with_tz(13908, 0);
    assert(date_equal(d1, d2) == false);
}

#[test]
fn cbcl_date_equal_004() {
    //  test comparison of date
    let d1 = date_from_epoch_days_with_tz(13909, 0);
    let d2 = date_from_epoch_days_with_tz(13909, 540);
    assert(date_equal(d1, d2) == false);
}

#[test]
fn cbcl_date_equal_005() {
    //  test comparison of date
    let d1 = date_from_epoch_days_with_tz(13909, 540);
    let d2 = date_from_epoch_days_with_tz(13909, 0);
    assert(date_equal(d1, d2) == false);
}
