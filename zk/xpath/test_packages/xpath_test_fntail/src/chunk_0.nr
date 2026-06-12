//! Test chunk 0 for fn:tail
//! Contains 2 tests

use dep::xpath::stub_fntail;

#[test]
fn tail_003() {
    //  tail() of a singleton sequence
    // XPath: count(tail(\"a\"))
    // Expected: 0
    // Calls stub function - will fail until fn:tail is implemented
    let _ = stub_fntail();
}

#[test]
fn tail_004() {
    //  tail() of an empty sequence
    // XPath: let $a := /works/employee return count(tail($a/z))
    // Expected: 0
    // Calls stub function - will fail until fn:tail is implemented
    let _ = stub_fntail();
}
