//! Test chunk 0 for op:date-less-than
//! Contains 10 tests

use dep::xpath::{date_from_epoch_days_with_tz, date_less_than};

#[test]
fn op_date_less_than2args_1() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(0, 0);
    let d2 = date_from_epoch_days_with_tz(0, 0);
    assert(date_less_than(d1, d2) == false);
}

#[test]
fn op_date_less_than2args_2() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(5068, 0);
    let d2 = date_from_epoch_days_with_tz(0, 0);
    assert(date_less_than(d1, d2) == false);
}

#[test]
fn op_date_less_than2args_3() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(22279, 0);
    let d2 = date_from_epoch_days_with_tz(0, 0);
    assert(date_less_than(d1, d2) == false);
}

#[test]
fn op_date_less_than2args_4() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(0, 0);
    let d2 = date_from_epoch_days_with_tz(5068, 0);
    assert(date_less_than(d1, d2) == true);
}

#[test]
fn op_date_less_than2args_5() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(0, 0);
    let d2 = date_from_epoch_days_with_tz(22279, 0);
    assert(date_less_than(d1, d2) == true);
}

#[test]
fn k_datelt_1() {
    //  *******************************************************  Test: K-DateLT-1
    let d1 = date_from_epoch_days_with_tz(12611, 0);
    let d2 = date_from_epoch_days_with_tz(12612, 0);
    assert(date_less_than(d1, d2) == true);
}

#[test]
fn cbcl_date_less_than_002() {
    //  test comparison of date
    let d1 = date_from_epoch_days_with_tz(13908, 0);
    let d2 = date_from_epoch_days_with_tz(13909, 540);
    assert(date_less_than(d1, d2) == true);
}

#[test]
fn cbcl_date_less_than_003() {
    //  test comparison of date
    let d1 = date_from_epoch_days_with_tz(13909, 540);
    let d2 = date_from_epoch_days_with_tz(13908, 0);
    assert(date_less_than(d1, d2) == false);
}

#[test]
fn cbcl_date_less_than_004() {
    //  test comparison of date
    let d1 = date_from_epoch_days_with_tz(13909, 0);
    let d2 = date_from_epoch_days_with_tz(13909, 540);
    assert(date_less_than(d1, d2) == false);
}

#[test]
fn cbcl_date_less_than_005() {
    //  test comparison of date
    let d1 = date_from_epoch_days_with_tz(13909, 540);
    let d2 = date_from_epoch_days_with_tz(13909, 0);
    assert(date_less_than(d1, d2) == true);
}
