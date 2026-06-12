//! Test chunk 0 for op:gYearMonth-equal
//! Contains 46 tests

use dep::xpath::stub_opgyearmonth_equal;

#[test]
fn op_gyearmonth_equal2args_1() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: xs:gYearMonth(\"1970-01Z\") eq xs:gYearMonth(\"1970-01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equal2args_2() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: xs:gYearMonth(\"1984-12Z\") eq xs:gYearMonth(\"1970-01Z\")
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equal2args_3() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: xs:gYearMonth(\"2030-12Z\") eq xs:gYearMonth(\"1970-01Z\")
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equal2args_4() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: xs:gYearMonth(\"1970-01Z\") eq xs:gYearMonth(\"1984-12Z\")
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equal2args_5() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: xs:gYearMonth(\"1970-01Z\") eq xs:gYearMonth(\"2030-12Z\")
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equal2args_6() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: xs:gYearMonth(\"1970-01Z\") ne xs:gYearMonth(\"1970-01Z\")
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equal2args_7() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: xs:gYearMonth(\"1984-12Z\") ne xs:gYearMonth(\"1970-01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equal2args_8() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: xs:gYearMonth(\"2030-12Z\") ne xs:gYearMonth(\"1970-01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equal2args_9() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: xs:gYearMonth(\"1970-01Z\") ne xs:gYearMonth(\"1984-12Z\")
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equal2args_10() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: xs:gYearMonth(\"1970-01Z\") ne xs:gYearMonth(\"2030-12Z\")
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equalnew_1() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: (xs:gYearMonth(\"1976-02-05:00\") eq xs:gYearMonth(\"1976-03
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equalnew_2() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: (xs:gYearMonth(\"1976-03-05:00\") eq xs:gYearMonth(\"1976-03
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equalnew_3() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: fn:not((xs:gYearMonth(\"1995-02Z\") eq xs:gYearMonth(\"1995-
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equalnew_4() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: fn:not(xs:gYearMonth(\"2005-02Z\") ne xs:gYearMonth(\"2006-0
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equalnew_5() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: fn:not(xs:gYearMonth(\"2000-01Z\") eq xs:gYearMonth(\"2001-0
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equalnew_6() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: fn:not(xs:gYearMonth(\"2005-01Z\") ne xs:gYearMonth(\"2005-0
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equalnew_7() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: (xs:gYearMonth(\"2000-02Z\") eq xs:gYearMonth(\"2000-03Z\"))
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equalnew_8() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: (xs:gYearMonth(\"2000-01Z\") ne xs:gYearMonth(\"2000-01Z\"))
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equalnew_9() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: (xs:gYearMonth(\"2000-01Z\") eq xs:gYearMonth(\"2000-03Z\"))
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equalnew_10() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: (xs:gYearMonth(\"1976-01Z\") ne xs:gYearMonth(\"1976-02Z\"))
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equalnew_11() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: (xs:gYearMonth(\"1980-05Z\") eq xs:gYearMonth(\"1980-05Z\"))
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equalnew_12() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: (xs:gYearMonth(\"2000-06Z\") ne xs:gYearMonth(\"2000-07Z\"))
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equalnew_13() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: (xs:gYearMonth(\"1980-09Z\") eq xs:gYearMonth(\"1980-09Z\"))
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn op_gyearmonth_equalnew_14() {
    //  ******************************************************* Test: op-gYearMonth-equ
    // XPath: (xs:gYearMonth(\"1980-03Z\") ne xs:gYearMonth(\"1980-03Z\"))
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn k_gyearmontheq_1() {
    //  *******************************************************  Test: K-gYearMonthEQ-1
    // XPath: xs:gYearMonth(\"2001-01 \") eq xs:gYearMonth(\"2001-01\")
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn k_gyearmontheq_2() {
    //  *******************************************************  Test: K-gYearMonthEQ-2
    // XPath: not(xs:gYearMonth(\"2001-03\") eq xs:gYearMonth(\"2000-03\")
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn k_gyearmontheq_3() {
    //  *******************************************************  Test: K-gYearMonthEQ-3
    // XPath: xs:gYearMonth(\"2001-12\") ne xs:gYearMonth(\"2001-11\")
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn k_gyearmontheq_4() {
    //  *******************************************************  Test: K-gYearMonthEQ-4
    // XPath: not(xs:gYearMonth(\"1995-11\") ne xs:gYearMonth(\"1995-11\")
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn k_gyearmontheq_5() {
    //  *******************************************************  Test: K-gYearMonthEQ-5
    // XPath: xs:gYearMonth(\"1999-01-00:00\") eq xs:gYearMonth(\"1999-01Z
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn k_gyearmontheq_6() {
    //  *******************************************************  Test: K-gYearMonthEQ-6
    // XPath: xs:gYearMonth(\"1999-01+00:00\") eq xs:gYearMonth(\"1999-01Z
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn k_gyearmontheq_7() {
    //  *******************************************************  Test: K-gYearMonthEQ-7
    // XPath: xs:gYearMonth(\"1999-01Z\") eq xs:gYearMonth(\"1999-01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn k_gyearmontheq_8() {
    //  *******************************************************  Test: K-gYearMonthEQ-8
    // XPath: xs:gYearMonth(\"1999-01-00:00\") eq xs:gYearMonth(\"1999-01+
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn cbcl_gyearmonth_equal_001() {
    //  test comparison of gYearMonth
    // XPath: declare function local:gYearMonth($year as xs:integer) { xs:
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn cbcl_gyearmonth_equal_002() {
    //  test comparison of gYearMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn cbcl_gyearmonth_equal_003() {
    //  test comparison of gYearMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn cbcl_gyearmonth_equal_004() {
    //  test comparison of gYearMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn cbcl_gyearmonth_equal_005() {
    //  test comparison of gYearMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn cbcl_gyearmonth_equal_006() {
    //  test comparison of gYearMonth
    // XPath: declare function local:gYearMonth($year as xs:integer) { xs:
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn cbcl_gyearmonth_equal_007() {
    //  test comparison of gYearMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn cbcl_gyearmonth_equal_008() {
    //  test comparison of gYearMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn cbcl_gyearmonth_equal_009() {
    //  test comparison of gYearMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn cbcl_gyearmonth_equal_010() {
    //  test comparison of gYearMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn cbcl_gyearmonth_equal_011() {
    //  test comparison of gYearMonth
    // XPath: declare function local:gYearMonth($gYearMonth as xs:gYearMon
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn cbcl_gyearmonth_equal_012() {
    //  test comparison of gYearMonth
    // XPath: declare function local:gYearMonth($gYearMonth as xs:gYearMon
    // Expected: true
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn cbcl_gyearmonth_equal_013() {
    //  test comparison of gYearMonth
    // XPath: declare function local:gYearMonth($gYearMonth as xs:gYearMon
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}

#[test]
fn cbcl_gyearmonth_equal_014() {
    //  test comparison of gYearMonth
    // XPath: declare function local:gYearMonth($gYearMonth as xs:gYearMon
    // Expected: false
    // Calls stub function - will fail until op:gYearMonth-equal is implemented
    let _ = stub_opgyearmonth_equal();
}
