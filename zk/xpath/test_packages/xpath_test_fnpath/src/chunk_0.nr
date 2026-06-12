//! Test chunk 0 for fn:path
//! Contains 1 tests

use dep::xpath::stub_fnpath;

#[test]
fn path014() {
    // path() applied to a parentless element
    // XPath: import module namespace copy=\"http://www.w3.org/QT3/copy\";
    // Expected: Q{http://www.w3.org/2005/xpath-functions
    // Calls stub function - will fail until fn:path is implemented
    let _ = stub_fnpath();
}
