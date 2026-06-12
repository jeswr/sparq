//! Test chunk 0 for op:gMonth-equal
//! Contains 45 tests

use dep::xpath::stub_opgmonth_equal;

#[test]
fn op_gmonth_equal2args_1() {
    //  ******************************************************* Test: op-gMonth-equal2a
    // XPath: xs:gMonth(\"--01Z\") eq xs:gMonth(\"--01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal2args_2() {
    //  ******************************************************* Test: op-gMonth-equal2a
    // XPath: xs:gMonth(\"--07Z\") eq xs:gMonth(\"--01Z\")
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal2args_3() {
    //  ******************************************************* Test: op-gMonth-equal2a
    // XPath: xs:gMonth(\"--12Z\") eq xs:gMonth(\"--01Z\")
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal2args_4() {
    //  ******************************************************* Test: op-gMonth-equal2a
    // XPath: xs:gMonth(\"--01Z\") eq xs:gMonth(\"--07Z\")
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal2args_5() {
    //  ******************************************************* Test: op-gMonth-equal2a
    // XPath: xs:gMonth(\"--01Z\") eq xs:gMonth(\"--12Z\")
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal2args_6() {
    //  ******************************************************* Test: op-gMonth-equal2a
    // XPath: xs:gMonth(\"--01Z\") ne xs:gMonth(\"--01Z\")
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal2args_7() {
    //  ******************************************************* Test: op-gMonth-equal2a
    // XPath: xs:gMonth(\"--07Z\") ne xs:gMonth(\"--01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal2args_8() {
    //  ******************************************************* Test: op-gMonth-equal2a
    // XPath: xs:gMonth(\"--12Z\") ne xs:gMonth(\"--01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal2args_9() {
    //  ******************************************************* Test: op-gMonth-equal2a
    // XPath: xs:gMonth(\"--01Z\") ne xs:gMonth(\"--07Z\")
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal2args_10() {
    //  ******************************************************* Test: op-gMonth-equal2a
    // XPath: xs:gMonth(\"--01Z\") ne xs:gMonth(\"--12Z\")
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal_2() {
    //  ******************************************************* Test: op-gMonth-equal-2
    // XPath: (xs:gMonth(\"--12-05:00\") eq xs:gMonth(\"--12Z\"))
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal_3() {
    //  ******************************************************* Test: op-gMonth-equal-3
    // XPath: fn:not((xs:gMonth(\"--12Z\") eq xs:gMonth(\"--12Z\")))
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal_4() {
    //  ******************************************************* Test: op-gMonth-equal-4
    // XPath: fn:not(xs:gMonth(\"--05Z\") ne xs:gMonth(\"--06Z\"))
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal_5() {
    //  ******************************************************* Test: op-gMonth-equal-5
    // XPath: fn:not(xs:gMonth(\"--11Z\") eq xs:gMonth(\"--10Z\"))
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal_6() {
    //  ******************************************************* Test: op-gMonth-equal-6
    // XPath: fn:not(xs:gMonth(\"--05Z\") ne xs:gMonth(\"--05Z\"))
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal_7() {
    //  ******************************************************* Test: op-gMonth-equal-7
    // XPath: (xs:gMonth(\"--04Z\") eq xs:gMonth(\"--02Z\")) and (xs:gMont
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal_8() {
    //  ******************************************************* Test: op-gMonth-equal-8
    // XPath: (xs:gMonth(\"--12Z\") ne xs:gMonth(\"--03Z\")) and (xs:gMont
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal_9() {
    //  ******************************************************* Test: op-gMonth-equal-9
    // XPath: (xs:gMonth(\"--02Z\") eq xs:gMonth(\"--02Z\")) or (xs:gMonth
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal_10() {
    //  ******************************************************* Test: op-gMonth-equal-1
    // XPath: (xs:gMonth(\"--06Z\") ne xs:gMonth(\"--06Z\")) or (xs:gMonth
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal_11() {
    //  ******************************************************* Test: op-gMonth-equal-1
    // XPath: (xs:gMonth(\"--03Z\") eq xs:gMonth(\"--01Z\")) or (fn:true()
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal_12() {
    //  ******************************************************* Test: op-gMonth-equal-1
    // XPath: (xs:gMonth(\"--08Z\") ne xs:gMonth(\"--07Z\")) or (fn:true()
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal_13() {
    //  ******************************************************* Test: op-gMonth-equal-1
    // XPath: (xs:gMonth(\"--05Z\") eq xs:gMonth(\"--05Z\")) or (fn:false(
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn op_gmonth_equal_14() {
    //  ******************************************************* Test: op-gMonth-equal-1
    // XPath: (xs:gMonth(\"--09Z\") ne xs:gMonth(\"--09Z\")) or (fn:false(
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn k_gmontheq_1() {
    //  *******************************************************  Test: K-gMonthEQ-1
    // XPath: xs:gMonth(\"--11 \") eq xs:gMonth(\"--11\")
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn k_gmontheq_2() {
    //  *******************************************************  Test: K-gMonthEQ-2
    // XPath: not(xs:gMonth(\"--11\") eq xs:gMonth(\"--01\"))
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn k_gmontheq_3() {
    //  *******************************************************  Test: K-gMonthEQ-3
    // XPath: xs:gMonth(\"--12\") ne xs:gMonth(\"--10\")
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn k_gmontheq_4() {
    //  *******************************************************  Test: K-gMonthEQ-4
    // XPath: not(xs:gMonth(\"--03\") ne xs:gMonth(\"--03\"))
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn k_gmontheq_5() {
    //  *******************************************************  Test: K-gMonthEQ-5
    // XPath: xs:gMonth(\"--01-00:00\") eq xs:gMonth(\"--01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn k_gmontheq_6() {
    //  *******************************************************  Test: K-gMonthEQ-6
    // XPath: xs:gMonth(\"--01+00:00\") eq xs:gMonth(\"--01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn k_gmontheq_7() {
    //  *******************************************************  Test: K-gMonthEQ-7
    // XPath: xs:gMonth(\"--01Z\") eq xs:gMonth(\"--01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn k_gmontheq_8() {
    //  *******************************************************  Test: K-gMonthEQ-8
    // XPath: xs:gMonth(\"--01-00:00\") eq xs:gMonth(\"--01+00:00\")
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn cbcl_gmonth_equal_001() {
    //  test comparison of gMonth
    // XPath: declare function local:gMonth($month as xs:integer) { if ($m
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn cbcl_gmonth_equal_002() {
    //  test comparison of gMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn cbcl_gmonth_equal_003() {
    //  test comparison of gMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn cbcl_gmonth_equal_004() {
    //  test comparison of gMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn cbcl_gmonth_equal_005() {
    //  test comparison of gMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn cbcl_gmonth_equal_006() {
    //  test comparison of gMonth
    // XPath: declare function local:gMonth($month as xs:integer) { if ($m
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn cbcl_gmonth_equal_007() {
    //  test comparison of gMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn cbcl_gmonth_equal_008() {
    //  test comparison of gMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn cbcl_gmonth_equal_009() {
    //  test comparison of gMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn cbcl_gmonth_equal_010() {
    //  test comparison of gMonth
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn cbcl_gmonth_equal_011() {
    //  test comparison of gMonth
    // XPath: declare function local:gMonth($gMonth as xs:gMonth, $null as
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn cbcl_gmonth_equal_012() {
    //  test comparison of gMonth
    // XPath: declare function local:gMonth($gMonth as xs:gMonth, $null as
    // Expected: true
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn cbcl_gmonth_equal_013() {
    //  test comparison of gMonth
    // XPath: declare function local:gMonth($gMonth as xs:gMonth, $null as
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}

#[test]
fn cbcl_gmonth_equal_014() {
    //  test comparison of gMonth
    // XPath: declare function local:gMonth($gMonth as xs:gMonth, $null as
    // Expected: false
    // Calls stub function - will fail until op:gMonth-equal is implemented
    let _ = stub_opgmonth_equal();
}
