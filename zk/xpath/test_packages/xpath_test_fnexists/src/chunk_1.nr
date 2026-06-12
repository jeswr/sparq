//! Test chunk 1 for fn:exists
//! Contains 6 tests

use dep::xpath::stub_fnexists;

#[test]
fn k_seqexistsfunc_14() {
    //  fn:empty combined with fn:remove and fn:not.
    // XPath: not(empty(remove((current-time(), 1), 1)))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn cbcl_exists_001() {
    //  test fn:exists on a mix of values and expressions
    // XPath: declare function local:generate($arg as xs:integer?) { if ($
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn cbcl_exists_002() {
    //  Tests aggregation optimizations for exists
    // XPath: exists(for $x in (1 to 10)[. mod 2 = 0] return \"blah\")
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn cbcl_exists_003() {
    //  Tests exists on text node
    // XPath: exists(text {(1 to 10)[. mod 2 = 0]})
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn cbcl_exists_004() {
    //  Test on optimization as aggregate function
    // XPath: exists(for $x in (1 to 10)[. mod 2 = 0] return true())
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}

#[test]
fn cbcl_exists_005() {
    //  Tests exists on a map expression
    // XPath: exists(for $x in (1 to 10)[. mod 2 = 0] return floor($x))
    // Expected: true
    // Calls stub function - will fail until fn:exists is implemented
    let _ = stub_fnexists();
}
