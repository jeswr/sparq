//! Test chunk 0 for fn:filter
//! Contains 1 tests

use dep::xpath::stub_fnfilter;

#[test]
fn fn_filter_005() {
    // Attempts to reference the 'filter' function with arity two.
    // XPath: fn:exists( fn:filter#2 )
    // Expected: true
    // Calls stub function - will fail until fn:filter is implemented
    let _ = stub_fnfilter();
}
