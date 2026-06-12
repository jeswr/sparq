//! Test chunk 0 for op:date-greater-than
//! Contains 6 tests

use dep::xpath::{date_from_epoch_days_with_tz, date_greater_than};

#[test]
fn op_date_greater_than2args_1() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(0, 0);
    let d2 = date_from_epoch_days_with_tz(0, 0);
    assert(date_greater_than(d1, d2) == false);
}

#[test]
fn op_date_greater_than2args_2() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(5068, 0);
    let d2 = date_from_epoch_days_with_tz(0, 0);
    assert(date_greater_than(d1, d2) == true);
}

#[test]
fn op_date_greater_than2args_3() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(22279, 0);
    let d2 = date_from_epoch_days_with_tz(0, 0);
    assert(date_greater_than(d1, d2) == true);
}

#[test]
fn op_date_greater_than2args_4() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(0, 0);
    let d2 = date_from_epoch_days_with_tz(5068, 0);
    assert(date_greater_than(d1, d2) == false);
}

#[test]
fn op_date_greater_than2args_5() {
    //  ******************************************************* Test:
    let d1 = date_from_epoch_days_with_tz(0, 0);
    let d2 = date_from_epoch_days_with_tz(22279, 0);
    assert(date_greater_than(d1, d2) == false);
}

#[test]
fn k_dategt_1() {
    //  *******************************************************  Test: K-DateGT-1
    let d1 = date_from_epoch_days_with_tz(12612, 0);
    let d2 = date_from_epoch_days_with_tz(12611, 0);
    assert(date_greater_than(d1, d2) == true);
}
