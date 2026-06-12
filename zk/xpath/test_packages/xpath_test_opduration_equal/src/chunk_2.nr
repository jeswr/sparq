//! Test chunk 2 for op:duration-equal
//! Contains 24 tests

use dep::xpath::stub_opduration_equal;

#[test]
fn k_daytimedurationeq_6() {
    // The xs:dayTimeDuration values -PT0S and PT0S are equal.
    // XPath: xs:dayTimeDuration(\"-PT0S\") eq xs:dayTimeDuration(\"PT0S\"
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn k_daytimedurationeq_7() {
    //  The xs:dayTimeDuration values -P2DT5H and P2DT5H are not equal.
    // XPath: xs:dayTimeDuration(\"-P2DT5H\") ne xs:dayTimeDuration(\"P2DT
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn k_daytimedurationeq_8() {
    // Test eq operator for xs:dayTimeDuration values with large milli second component
    // XPath: xs:dayTimeDuration(\"P6DT16H34M59.613999S\") eq xs:dayTimeDu
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn k_yearmonthdurationeq_1() {
    //  Simple test of 'eq' for xs:yearMonthDuration, returning positive.
    // XPath: xs:yearMonthDuration(\"P1999Y10M\") eq xs:yearMonthDuration(
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn k_yearmonthdurationeq_2() {
    // Testing 'eq' involving xs:yearMonthDuration with two zeroed values.
    // XPath: xs:yearMonthDuration(\"P0M\") eq xs:yearMonthDuration(\"P0M\
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn k_yearmonthdurationeq_3() {
    //  Simple test of 'eq' for xs:yearMonthDuration.
    // XPath: not(xs:yearMonthDuration(\"P1999Y\") eq xs:yearMonthDuration
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn k_yearmonthdurationeq_4() {
    // Simple test of 'ne' for xs:yearMonthDuration.
    // XPath: xs:yearMonthDuration(\"P1999Y01M\") ne xs:yearMonthDuration(
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn k_yearmonthdurationeq_5() {
    //  Simple test of 'ne' for xs:yearMonthDuration.
    // XPath: not(xs:yearMonthDuration(\"P1999Y10M\") ne xs:yearMonthDurat
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn k_yearmonthdurationeq_6() {
    //  The xs:yearMonthDuration values -P0M and P0M are equal.
    // XPath: xs:yearMonthDuration(\"-P3Y8M\") ne xs:yearMonthDuration(\"P
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn k_yearmonthdurationeq_7() {
    // The xs:yearMonthDuration values -P3Y8M and +P3Y8M are not equal.
    // XPath: xs:yearMonthDuration(\"-P3Y8M\") ne xs:yearMonthDuration(\"P
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn cbcl_daytimeduration_equal_001() {
    //  test comparison of dayTimeDurations
    // XPath: declare function local:dayTimeDuration($days as xs:integer,
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn cbcl_daytimeduration_equal_002() {
    //  test comparison of dayTimeDurations
    // XPath: declare function local:dayTimeDuration($days as xs:integer,
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn cbcl_daytimeduration_equal_003() {
    //  test comparison of xs:dayTimeDuration
    // XPath: declare function local:dayTimeDuration($dayTimeDuration as x
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn cbcl_daytimeduration_equal_004() {
    //  test comparison of xs:dayTimeDuration
    // XPath: declare function local:dayTimeDuration($dayTimeDuration as x
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn cbcl_daytimeduration_equal_005() {
    //  test comparison of xs:dayTimeDuration
    // XPath: declare function local:dayTimeDuration($dayTimeDuration as x
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn cbcl_daytimeduration_equal_006() {
    //  test comparison of xs:dayTimeDuration
    // XPath: declare function local:dayTimeDuration($dayTimeDuration as x
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn cbcl_duration_equal_001() {
    //  test comparison of dayTimeDurations
    // XPath: declare function local:duration($days as xs:integer, $hours
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn cbcl_duration_equal_002() {
    //  test comparison of dayTimeDurations
    // XPath: declare function local:duration($days as xs:integer, $hours
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn cbcl_duration_equal_003() {
    //  test comparison of xs:duration
    // XPath: declare function local:duration($duration as xs:duration, $n
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn cbcl_duration_equal_004() {
    //  test comparison of xs:duration
    // XPath: declare function local:duration($duration as xs:duration, $n
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn cbcl_duration_equal_005() {
    //  test comparison of xs:duration
    // XPath: declare function local:duration($duration as xs:duration, $n
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn cbcl_duration_equal_006() {
    //  test comparison of xs:duration
    // XPath: declare function local:duration($duration as xs:duration, $n
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn cbcl_yearmonthduration_equal_001() {
    //  test equality of xs:yearMonthDuration
    // XPath: declare function local:yearMonthDuration($years as xs:intege
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn cbcl_yearmonthduration_equal_002() {
    //  test equality of xs:yearMonthDuration
    // XPath: declare function local:yearMonthDuration($years as xs:intege
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}
