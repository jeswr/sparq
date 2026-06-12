//! Test chunk 1 for fn:empty
//! Contains 2 tests

use dep::xpath::stub_fnempty;

#[test]
fn cbcl_empty_func_005() {
    //  Tests on optimization as aggregate function
    // XPath: empty(for $x in (1 to 10)[. mod 2 = 0] return true())
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}

#[test]
fn cbcl_empty_func_006() {
    //  Tests empty on a map expression
    // XPath: empty(for $x in (1 to 10)[. mod 2 = 0] return floor($x))
    // Expected: false
    // Calls stub function - will fail until fn:empty is implemented
    let _ = stub_fnempty();
}
