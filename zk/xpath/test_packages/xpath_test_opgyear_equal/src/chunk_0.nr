//! Test chunk 0 for op:gYear-equal
//! Contains 46 tests

use dep::xpath::stub_opgyear_equal;

#[test]
fn op_gyear_equal2args_1() {
    //  ******************************************************* Test: op-gYear-equal2ar
    // XPath: xs:gYear(\"1970Z\") eq xs:gYear(\"1970Z\")
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal2args_2() {
    //  ******************************************************* Test: op-gYear-equal2ar
    // XPath: xs:gYear(\"2012Z\") eq xs:gYear(\"1970Z\")
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal2args_3() {
    //  ******************************************************* Test: op-gYear-equal2ar
    // XPath: xs:gYear(\"2030Z\") eq xs:gYear(\"1970Z\")
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal2args_4() {
    //  ******************************************************* Test: op-gYear-equal2ar
    // XPath: xs:gYear(\"1970Z\") eq xs:gYear(\"2012Z\")
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal2args_5() {
    //  ******************************************************* Test: op-gYear-equal2ar
    // XPath: xs:gYear(\"1970Z\") eq xs:gYear(\"2030Z\")
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal2args_6() {
    //  ******************************************************* Test: op-gYear-equal2ar
    // XPath: xs:gYear(\"1970Z\") ne xs:gYear(\"1970Z\")
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal2args_7() {
    //  ******************************************************* Test: op-gYear-equal2ar
    // XPath: xs:gYear(\"2012Z\") ne xs:gYear(\"1970Z\")
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal2args_8() {
    //  ******************************************************* Test: op-gYear-equal2ar
    // XPath: xs:gYear(\"2030Z\") ne xs:gYear(\"1970Z\")
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal2args_9() {
    //  ******************************************************* Test: op-gYear-equal2ar
    // XPath: xs:gYear(\"1970Z\") ne xs:gYear(\"2012Z\")
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal2args_10() {
    //  ******************************************************* Test: op-gYear-equal2ar
    // XPath: xs:gYear(\"1970Z\") ne xs:gYear(\"2030Z\")
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal_1() {
    //  ******************************************************* Test: op-gYear-equal-1
    // XPath: (xs:gYear(\"2005-12:00\") eq xs:gYear(\"2005+12:00\"))
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal_2() {
    //  ******************************************************* Test: op-gYear-equal-2
    // XPath: (xs:gYear(\"1976-05:00\") eq xs:gYear(\"1976-05:00\"))
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equalnew_3() {
    //  ******************************************************* Test: op-gYear-equal-3
    // XPath: fn:not((xs:gYear(\"1995Z\") eq xs:gYear(\"1995Z\")))
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equalnew_4() {
    //  ******************************************************* Test: op-gYear-equal-4
    // XPath: fn:not(xs:gYear(\"2005Z\") ne xs:gYear(\"2006Z\"))
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal_5() {
    //  ******************************************************* Test: op-gYear-equal-5
    // XPath: fn:not(xs:gYear(\"2000Z\") eq xs:gYear(\"2001Z\"))
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equalnew_6() {
    //  ******************************************************* Test: op-gYear-equal-6
    // XPath: fn:not(xs:gYear(\"2005Z\") ne xs:gYear(\"2005Z\"))
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equalnew_7() {
    //  ******************************************************* Test: op-gYear-equal-7
    // XPath: (xs:gYear(\"2000Z\") eq xs:gYear(\"2000Z\")) and (xs:gYear(\
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal_8() {
    //  ******************************************************* Test: op-gYear-equal-8
    // XPath: (xs:gYear(\"2000Z\") ne xs:gYear(\"2000Z\")) and (xs:gYear(\
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equalnew_9() {
    //  ******************************************************* Test: op-gYear-equal-9
    // XPath: (xs:gYear(\"2000Z\") eq xs:gYear(\"2000Z\")) or (xs:gYear(\"
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal_10() {
    //  ******************************************************* Test: op-gYear-equal-10
    // XPath: (xs:gYear(\"1976Z\") ne xs:gYear(\"1976Z\")) or (xs:gYear(\"
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal_11() {
    //  ******************************************************* Test: op-gYear-equal-11
    // XPath: (xs:gYear(\"1980Z\") eq xs:gYear(\"1980Z\")) or (fn:true())
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equalnew_12() {
    //  ******************************************************* Test: op-gYear-equal-12
    // XPath: (xs:gYear(\"2000Z\") ne xs:gYear(\"2000Z\")) or (fn:true())
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal_13() {
    //  ******************************************************* Test: op-gYear-equal-13
    // XPath: (xs:gYear(\"1980Z\") eq xs:gYear(\"1980Z\")) or (fn:false())
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn op_gyear_equal_14() {
    //  ******************************************************* Test: op-gYear-equal-14
    // XPath: (xs:gYear(\"1980Z\") ne xs:gYear(\"1980Z\")) or (fn:false())
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn k_gyeareq_1() {
    //  *******************************************************  Test: K-gYearEQ-1
    // XPath: xs:gYear(\" 1956 \") eq xs:gYear(\"1956\")
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn k_gyeareq_2() {
    //  *******************************************************  Test: K-gYearEQ-2
    // XPath: not(xs:gYear(\"1956\") eq xs:gYear(\"1958\"))
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn k_gyeareq_3() {
    //  *******************************************************  Test: K-gYearEQ-3
    // XPath: xs:gYear(\"1956\") ne xs:gYear(\"1958\")
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn k_gyeareq_4() {
    //  *******************************************************  Test: K-gYearEQ-4
    // XPath: not(xs:gYear(\"1956\") ne xs:gYear(\"1956\"))
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn k_gyeareq_5() {
    //  *******************************************************  Test: K-gYearEQ-5
    // XPath: xs:gYear(\"1956-00:00\") eq xs:gYear(\"1956Z\")
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn k_gyeareq_6() {
    //  *******************************************************  Test: K-gYearEQ-6
    // XPath: xs:gYear(\"1956+00:00\") eq xs:gYear(\"1956Z\")
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn k_gyeareq_7() {
    //  *******************************************************  Test: K-gYearEQ-7
    // XPath: xs:gYear(\"1956Z\") eq xs:gYear(\"1956Z\")
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn k_gyeareq_8() {
    //  *******************************************************  Test: K-gYearEQ-8
    // XPath: xs:gYear(\"1956-00:00\") eq xs:gYear(\"1956+00:00\")
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn cbcl_gyear_equal_001() {
    //  test comparison of gYear
    // XPath: declare function local:gYear($year as xs:integer) { xs:gYear
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn cbcl_gyear_equal_002() {
    //  test comparison of gYear
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn cbcl_gyear_equal_003() {
    //  test comparison of gYear
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn cbcl_gyear_equal_004() {
    //  test comparison of gYear
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn cbcl_gyear_equal_005() {
    //  test comparison of gYear
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn cbcl_gyear_equal_006() {
    //  test comparison of gYear
    // XPath: declare function local:gYear($year as xs:integer) { xs:gYear
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn cbcl_gyear_equal_007() {
    //  test comparison of gYear
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn cbcl_gyear_equal_008() {
    //  test comparison of gYear
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn cbcl_gyear_equal_009() {
    //  test comparison of gYear
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn cbcl_gyear_equal_010() {
    //  test comparison of gYear
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn cbcl_gyear_equal_011() {
    //  test comparison of gYear
    // XPath: declare function local:gYear($gYear as xs:gYear, $null as xs
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn cbcl_gyear_equal_012() {
    //  test comparison of gYear
    // XPath: declare function local:gYear($gYear as xs:gYear, $null as xs
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn cbcl_gyear_equal_013() {
    //  test comparison of gYear
    // XPath: declare function local:gYear($gYear as xs:gYear, $null as xs
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}

#[test]
fn cbcl_gyear_equal_014() {
    //  test comparison of gYear
    // XPath: declare function local:gYear($gYear as xs:gYear, $null as xs
    // Expected: false
    // Calls stub function - will fail until op:gYear-equal is implemented
    let _ = stub_opgyear_equal();
}
