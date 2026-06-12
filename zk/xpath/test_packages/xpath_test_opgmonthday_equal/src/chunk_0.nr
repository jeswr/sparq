//! Test chunk 0 for op:gMonthDay-equal
//! Contains 50 tests

use dep::xpath::stub_opgmonthday_equal;

#[test]
fn op_gmonthday_equal2args_1() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: xs:gMonthDay(\"--01-01Z\") eq xs:gMonthDay(\"--01-01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal2args_2() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: xs:gMonthDay(\"--04-10Z\") eq xs:gMonthDay(\"--01-01Z\")
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal2args_3() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: xs:gMonthDay(\"--12-31Z\") eq xs:gMonthDay(\"--01-01Z\")
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal2args_4() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: xs:gMonthDay(\"--01-01Z\") eq xs:gMonthDay(\"--04-10Z\")
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal2args_5() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: xs:gMonthDay(\"--01-01Z\") eq xs:gMonthDay(\"--12-31Z\")
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal2args_6() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: xs:gMonthDay(\"--01-01Z\") ne xs:gMonthDay(\"--01-01Z\")
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal2args_7() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: xs:gMonthDay(\"--04-10Z\") ne xs:gMonthDay(\"--01-01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal2args_8() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: xs:gMonthDay(\"--12-31Z\") ne xs:gMonthDay(\"--01-01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal2args_9() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: xs:gMonthDay(\"--01-01Z\") ne xs:gMonthDay(\"--04-10Z\")
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal2args_10() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: xs:gMonthDay(\"--01-01Z\") ne xs:gMonthDay(\"--12-31Z\")
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal_2() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: (xs:gMonthDay(\"--12-25Z\") eq xs:gMonthDay(\"--12-26Z\"))
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal_3() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: fn:not((xs:gMonthDay(\"--12-10Z\") eq xs:gMonthDay(\"--12-10
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal_4() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: fn:not(xs:gMonthDay(\"--05-01Z\") ne xs:gMonthDay(\"--06-12Z
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal_5() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: fn:not(xs:gMonthDay(\"--11-10Z\") eq xs:gMonthDay(\"--10-02Z
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal_6() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: fn:not(xs:gMonthDay(\"--05-10Z\") ne xs:gMonthDay(\"--05-10Z
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal_7() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: (xs:gMonthDay(\"--04-02Z\") eq xs:gMonthDay(\"--02-03Z\")) a
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal_8() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: (xs:gMonthDay(\"--12-10Z\") ne xs:gMonthDay(\"--03-11Z\")) a
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal_9() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: (xs:gMonthDay(\"--02-01Z\") eq xs:gMonthDay(\"--02-10Z\")) o
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal_10() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: (xs:gMonthDay(\"--06-02Z\") ne xs:gMonthDay(\"--06-09Z\")) o
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal_11() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: (xs:gMonthDay(\"--03-04Z\") eq xs:gMonthDay(\"--01-07Z\")) o
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal_12() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: (xs:gMonthDay(\"--08-04Z\") ne xs:gMonthDay(\"--07-12Z\")) o
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal_13() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: (xs:gMonthDay(\"--05-05Z\") eq xs:gMonthDay(\"--05-05Z\")) o
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn op_gmonthday_equal_14() {
    //  ******************************************************* Test: op-gMonthDay-equa
    // XPath: (xs:gMonthDay(\"--09-09Z\") ne xs:gMonthDay(\"--09-10Z\")) o
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn k_gmonthdayeq_1() {
    //  *******************************************************  Test: K-gMonthDayEQ-1
    // XPath: xs:gMonthDay(\"--04-01 \") eq xs:gMonthDay(\"--04-01\")
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn k_gmonthdayeq_2() {
    //  *******************************************************  Test: K-gMonthDayEQ-2
    // XPath: not(xs:gMonthDay(\"--03-03\") eq xs:gMonthDay(\"--04-03\"))
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn k_gmonthdayeq_3() {
    //  *******************************************************  Test: K-gMonthDayEQ-3
    // XPath: xs:gMonthDay(\"--04-12\") ne xs:gMonthDay(\"--04-11\")
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn k_gmonthdayeq_4() {
    //  *******************************************************  Test: K-gMonthDayEQ-4
    // XPath: not(xs:gMonthDay(\"--04-11\") ne xs:gMonthDay(\"--04-11\"))
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn k_gmonthdayeq_5() {
    //  *******************************************************  Test: K-gMonthDayEQ-5
    // XPath: xs:gMonthDay(\"--03-01-00:00\") eq xs:gMonthDay(\"--03-01Z\"
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn k_gmonthdayeq_6() {
    //  *******************************************************  Test: K-gMonthDayEQ-6
    // XPath: xs:gMonthDay(\"--03-01+00:00\") eq xs:gMonthDay(\"--03-01Z\"
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn k_gmonthdayeq_7() {
    //  *******************************************************  Test: K-gMonthDayEQ-7
    // XPath: xs:gMonthDay(\"--03-01Z\") eq xs:gMonthDay(\"--03-01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn k_gmonthdayeq_8() {
    //  *******************************************************  Test: K-gMonthDayEQ-8
    // XPath: xs:gMonthDay(\"--03-01-00:00\") eq xs:gMonthDay(\"--03-01+00
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_001() {
    //  test comparison of gMonthDay
    // XPath: declare function local:gMonthDay($day as xs:integer) { if ($
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_002() {
    //  test comparison of gMonthDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_003() {
    //  test comparison of gMonthDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_004() {
    //  test comparison of gMonthDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_005() {
    //  test comparison of gMonthDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_006() {
    //  test comparison of gMonthDay
    // XPath: declare function local:gMonthDay($day as xs:integer) { if ($
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_007() {
    //  test comparison of gMonthDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_008() {
    //  test comparison of gMonthDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_009() {
    //  test comparison of gMonthDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_010() {
    //  test comparison of gMonthDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_011() {
    //  test comparison of gMonthDay
    // XPath: declare function local:gMonthDay($gMonthDay as xs:gMonthDay,
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_012() {
    //  test comparison of gMonthDay
    // XPath: declare function local:gMonthDay($gMonthDay as xs:gMonthDay,
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_013() {
    //  test comparison of gMonthDay
    // XPath: declare function local:gMonthDay($gMonthDay as xs:gMonthDay,
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_014() {
    //  test comparison of gMonthDay
    // XPath: declare function local:gMonthDay($gMonthDay as xs:gMonthDay,
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_015() {
    //  test comparison of gDay
    // XPath: xs:gMonthDay(\"--01-30-12:00\") eq xs:gMonthDay(\"--01-31+12
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_016() {
    //  test comparison of gDay
    // XPath: xs:gMonthDay(\"--01-31+12:00\") eq xs:gMonthDay(\"--01-30-12
    // Expected: true
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_017() {
    //  test comparison of gDay
    // XPath: xs:gMonthDay(\"--01-30-12:00\") ne xs:gMonthDay(\"--01-31+12
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_018() {
    //  test comparison of gDay
    // XPath: xs:gMonthDay(\"--01-31+12:00\") ne xs:gMonthDay(\"--01-30-12
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}

#[test]
fn cbcl_gmonthday_equal_019() {
    //  test comparison of gMonthDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT1M'))
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}
