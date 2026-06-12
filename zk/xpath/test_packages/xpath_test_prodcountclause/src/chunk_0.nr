//! Test chunk 0 for prod:CountClause
//! Contains 1 tests

use dep::xpath::stub_prodcountclause;

#[test]
fn count_010() {
    // Count is reset on each execution of a FLWOR expression
    // XPath: for $x in 1 to 4 return       for $y in 1 to 4       count $
    // Expected: 1 2 3 4 1 2 3 4 1 2 3 4 1 2 3 4
    // Calls stub function - will fail until prod:CountClause is implemented
    let _ = stub_prodcountclause();
}
