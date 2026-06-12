//! Test chunk 0 for prod:UnaryLookup
//! Contains 1 tests

use dep::xpath::stub_produnarylookup;

#[test]
fn unarylookup_048() {
    // Decimal literal allowed in parentheses for maps
    // XPath: (map{1.1:1, 2.2:2, 3.3:3},  map{1.1:2, 2.2:3, 3.3:4})[?(2.2)
    // Expected: 4
    // Calls stub function - will fail until prod:UnaryLookup is implemented
    let _ = stub_produnarylookup();
}
