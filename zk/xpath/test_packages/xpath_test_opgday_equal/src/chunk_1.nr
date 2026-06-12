//! Test chunk 1 for op:gDay-equal
//! Contains 1 tests

use dep::xpath::stub_opgday_equal;

#[test]
fn cbcl_gday_equal_020() {
    //  test comparison of gDay
    // XPath: if (implicit-timezone() eq xs:dayTimeDuration('PT1H'))
    // Expected: false
    // Calls stub function - will fail until op:gDay-equal is implemented
    let _ = stub_opgday_equal();
}
