//! Test chunk 0 for fn:for-each-pair
//! Contains 5 tests

use dep::xpath::stub_fnfor_each_pair;

#[test]
fn fn_for_each_pair_008() {
    // Attempts to reference the 'for-each-pair' function with arity three.
    // XPath: fn:exists( fn:for-each-pair#3 )
    // Expected: true
    // Calls stub function - will fail until fn:for-each-pair is implemented
    let _ = stub_fnfor_each_pair();
}

#[test]
fn fn_for_each_pair_017() {
    // Evaluates the 'for-each-pair' function with $seq1, $seq2 set to a mix of item ty
    // XPath: fn:for-each-pair((//node(), 1, \"string\", 3.14, 2.7e0, fn:e
    // Expected: true true true true true true true true
    // Calls stub function - will fail until fn:for-each-pair is implemented
    let _ = stub_fnfor_each_pair();
}

#[test]
fn fn_for_each_pair_025() {
    // Evaluates the 'for-each-pair' function with a function which returns exactly one
    // XPath: fn:for-each-pair( 1 to 3, 1 to 3, function($arg1, $arg2) { (
    // Expected: 2 4 6
    // Calls stub function - will fail until fn:for-each-pair is implemented
    let _ = stub_fnfor_each_pair();
}

#[test]
fn fn_for_each_pair_026() {
    // Evaluates the 'for-each-pair' function with a function which returns zero or one
    // XPath: fn:for-each-pair( (true(), false(), true()), 1 to 3, functio
    // Expected: 1 3
    // Calls stub function - will fail until fn:for-each-pair is implemented
    let _ = stub_fnfor_each_pair();
}

#[test]
fn fn_for_each_pair_027() {
    // Evaluates the 'for-each-pair' function with a function which returns more than o
    // XPath: fn:for-each-pair( 1 to 3, 1 to 3, function($arg1, $arg2) { (
    // Expected: 1 1 2 2 3 3
    // Calls stub function - will fail until fn:for-each-pair is implemented
    let _ = stub_fnfor_each_pair();
}
