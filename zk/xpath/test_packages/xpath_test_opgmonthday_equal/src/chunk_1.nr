//! Test chunk 1 for op:gMonthDay-equal
//! Contains 1 tests

use dep::xpath::stub_opgmonthday_equal;

#[test]
fn cbcl_gmonthday_equal_020() {
    //  test comparison of gMonthDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT1M'))
    // Expected: false
    // Calls stub function - will fail until op:gMonthDay-equal is implemented
    let _ = stub_opgmonthday_equal();
}
