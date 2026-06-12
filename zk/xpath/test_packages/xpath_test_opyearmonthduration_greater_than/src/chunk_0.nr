//! Test chunk 0 for op:yearMonthDuration-greater-than
//! Contains 6 tests

use dep::xpath::{duration_from_microseconds, XsdYearMonthDuration, ym_duration_greater_than};

#[test]
fn op_yearmonthduration_greater_than2args_1() {
    //  ******************************************************* Test:
    let d1 = XsdYearMonthDuration::new(0);
    let d2 = XsdYearMonthDuration::new(0);
    assert(ym_duration_greater_than(d1, d2) == false);
}

#[test]
fn op_yearmonthduration_greater_than2args_2() {
    //  ******************************************************* Test:
    let d1 = XsdYearMonthDuration::new(12006);
    let d2 = XsdYearMonthDuration::new(0);
    assert(ym_duration_greater_than(d1, d2) == true);
}

#[test]
fn op_yearmonthduration_greater_than2args_3() {
    //  ******************************************************* Test:
    let d1 = XsdYearMonthDuration::new(24372);
    let d2 = XsdYearMonthDuration::new(0);
    assert(ym_duration_greater_than(d1, d2) == true);
}

#[test]
fn op_yearmonthduration_greater_than2args_4() {
    //  ******************************************************* Test:
    let d1 = XsdYearMonthDuration::new(0);
    let d2 = XsdYearMonthDuration::new(12006);
    assert(ym_duration_greater_than(d1, d2) == false);
}

#[test]
fn op_yearmonthduration_greater_than2args_5() {
    //  ******************************************************* Test:
    let d1 = XsdYearMonthDuration::new(0);
    let d2 = XsdYearMonthDuration::new(24372);
    assert(ym_duration_greater_than(d1, d2) == false);
}

#[test]
fn k_yearmonthdurationgt_1() {
    //  *******************************************************  Test:
    let d1 = XsdYearMonthDuration::new(23998);
    let d2 = XsdYearMonthDuration::new(23997);
    assert(ym_duration_greater_than(d1, d2) == true);
}
