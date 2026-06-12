//! Test chunk 0 for op:gDay-equal
//! Contains 50 tests

use dep::xpath::stub_opgday_equal;

#[test]
fn op_gday_equal2args_1() {
    //  ******************************************************* Test: op-gDay-equal2arg
    // XPath: xs:gDay(\"---01Z\") eq xs:gDay(\"---01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal2args_2() {
    //  ******************************************************* Test: op-gDay-equal2arg
    // XPath: xs:gDay(\"---14Z\") eq xs:gDay(\"---01Z\")
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal2args_3() {
    //  ******************************************************* Test: op-gDay-equal2arg
    // XPath: xs:gDay(\"---31Z\") eq xs:gDay(\"---01Z\")
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal2args_4() {
    //  ******************************************************* Test: op-gDay-equal2arg
    // XPath: xs:gDay(\"---01Z\") eq xs:gDay(\"---14Z\")
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal2args_5() {
    //  ******************************************************* Test: op-gDay-equal2arg
    // XPath: xs:gDay(\"---01Z\") eq xs:gDay(\"---31Z\")
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal2args_6() {
    //  ******************************************************* Test: op-gDay-equal2arg
    // XPath: xs:gDay(\"---01Z\") ne xs:gDay(\"---01Z\")
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal2args_7() {
    //  ******************************************************* Test: op-gDay-equal2arg
    // XPath: xs:gDay(\"---14Z\") ne xs:gDay(\"---01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal2args_8() {
    //  ******************************************************* Test: op-gDay-equal2arg
    // XPath: xs:gDay(\"---31Z\") ne xs:gDay(\"---01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal2args_9() {
    //  ******************************************************* Test: op-gDay-equal2arg
    // XPath: xs:gDay(\"---01Z\") ne xs:gDay(\"---14Z\")
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal2args_10() {
    //  ******************************************************* Test: op-gDay-equal2arg
    // XPath: xs:gDay(\"---01Z\") ne xs:gDay(\"---31Z\")
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal_2() {
    //  ******************************************************* Test: op-gDay-equal-2
    // XPath: (xs:gDay(\"---12-05:00\") eq xs:gDay(\"---12Z\"))
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal_3() {
    //  ******************************************************* Test: op-gDay-equal-3
    // XPath: fn:not((xs:gDay(\"---12Z\") eq xs:gDay(\"---12Z\")))
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal_4() {
    //  ******************************************************* Test: op-gDay-equal-4
    // XPath: fn:not(xs:gDay(\"---05Z\") ne xs:gDay(\"---06Z\"))
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal_5() {
    //  ******************************************************* Test: op-gDay-equal-5
    // XPath: fn:not(xs:gDay(\"---11Z\") eq xs:gDay(\"---10Z\"))
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal_6() {
    //  ******************************************************* Test: op-gDay-equal-6
    // XPath: fn:not(xs:gDay(\"---05Z\") ne xs:gDay(\"---05Z\"))
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal_7() {
    //  ******************************************************* Test: op-gDay-equal-7
    // XPath: (xs:gDay(\"---04Z\") eq xs:gDay(\"---02Z\")) and (xs:gDay(\"
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal_8() {
    //  ******************************************************* Test: op-gDay-equal-8
    // XPath: (xs:gDay(\"---12Z\") ne xs:gDay(\"---03Z\")) and (xs:gDay(\"
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal_9() {
    //  ******************************************************* Test: op-gDay-equal-9
    // XPath: (xs:gDay(\"---02Z\") eq xs:gDay(\"---02Z\")) or (xs:gDay(\"-
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal_10() {
    //  ******************************************************* Test: op-gDay-equal-10
    // XPath: (xs:gDay(\"---06Z\") ne xs:gDay(\"---06Z\")) or (xs:gDay(\"-
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal_11() {
    //  ******************************************************* Test: op-gDay-equal-11
    // XPath: (xs:gDay(\"---03Z\") eq xs:gDay(\"---01Z\")) or (fn:true())
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal_12() {
    //  ******************************************************* Test: op-gDay-equal-12
    // XPath: (xs:gDay(\"---08Z\") ne xs:gDay(\"---07Z\")) or (fn:true())
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal_13() {
    //  ******************************************************* Test: op-gDay-equal-13
    // XPath: (xs:gDay(\"---05Z\") eq xs:gDay(\"---05Z\")) or (fn:false())
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn op_gday_equal_14() {
    //  ******************************************************* Test: op-gDay-equal-14
    // XPath: (xs:gDay(\"---09Z\") ne xs:gDay(\"---09Z\")) or (fn:false())
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn k_gdayeq_1() {
    //  *******************************************************  Test: K-gDayEQ-1
    // XPath: xs:gDay(\" ---31 \") eq xs:gDay(\"---31\")
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn k_gdayeq_2() {
    //  *******************************************************  Test: K-gDayEQ-2
    // XPath: not(xs:gDay(\"---31\") eq xs:gDay(\"---01\"))
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn k_gdayeq_3() {
    //  *******************************************************  Test: K-gDayEQ-3
    // XPath: xs:gDay(\"---01\") ne xs:gDay(\"---10\")
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn k_gdayeq_4() {
    //  *******************************************************  Test: K-gDayEQ-4
    // XPath: not(xs:gDay(\"---01\") ne xs:gDay(\"---01\"))
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn k_gdayeq_5() {
    //  *******************************************************  Test: K-gDayEQ-5
    // XPath: xs:gDay(\"---01-00:00\") eq xs:gDay(\"---01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn k_gdayeq_6() {
    //  *******************************************************  Test: K-gDayEQ-6
    // XPath: xs:gDay(\"---01+00:00\") eq xs:gDay(\"---01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn k_gdayeq_7() {
    //  *******************************************************  Test: K-gDayEQ-7
    // XPath: xs:gDay(\"---01Z\") eq xs:gDay(\"---01Z\")
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn k_gdayeq_8() {
    //  *******************************************************  Test: K-gDayEQ-8
    // XPath: xs:gDay(\"---01-00:00\") eq xs:gDay(\"---01+00:00\")
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_001() {
    //  test comparison of gDay
    // XPath: declare function local:gDay($day as xs:integer) { if ($day l
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_002() {
    //  test comparison of gDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_003() {
    //  test comparison of gDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_004() {
    //  test comparison of gDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_005() {
    //  test comparison of gDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_006() {
    //  test comparison of gDay
    // XPath: declare function local:gDay($day as xs:integer) { if ($day l
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_007() {
    //  test comparison of gDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_008() {
    //  test comparison of gDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_009() {
    //  test comparison of gDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_010() {
    //  test comparison of gDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('-PT9M'))
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_011() {
    //  test comparison of gDay
    // XPath: declare function local:gDay($gDay as xs:gDay, $null as xs:bo
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_012() {
    //  test comparison of gDay
    // XPath: declare function local:gDay($gDay as xs:gDay, $null as xs:bo
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_013() {
    //  test comparison of gDay
    // XPath: declare function local:gDay($gDay as xs:gDay, $null as xs:bo
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_014() {
    //  test comparison of gDay
    // XPath: declare function local:gDay($gDay as xs:gDay, $null as xs:bo
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_015() {
    //  test comparison of gDay
    // XPath: xs:gDay(\"---30-12:00\") eq xs:gDay(\"---31+12:00\")
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_016() {
    //  test comparison of gDay
    // XPath: xs:gDay(\"---31+12:00\") eq xs:gDay(\"---30-12:00\")
    // Expected: true
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_017() {
    //  test comparison of gDay
    // XPath: xs:gDay(\"---30-12:00\") ne xs:gDay(\"---31+12:00\")
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_018() {
    //  test comparison of gDay
    // XPath: xs:gDay(\"---31+12:00\") ne xs:gDay(\"---30-12:00\")
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}

#[test]
fn cbcl_gday_equal_019() {
    //  test comparison of gDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT1H'))
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}
