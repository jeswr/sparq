//! Test chunk 1 for fn:reverse
//! Contains 7 tests

use dep::xpath::stub_fnreverse;

#[test]
fn k_seqreversefunc_17() {
    //  Apply a predicate to the result of fn:reverse().
    // XPath: reverse((1, 2, 3))[last() - 2]
    // Expected: 3
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k_seqreversefunc_18() {
    //  Apply a predicate to the result of fn:reverse().
    // XPath: reverse((1, 2, 3))[last() - 1]
    // Expected: 2
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k_seqreversefunc_19() {
    //  Apply a predicate to the result of fn:reverse().
    // XPath: deep-equal((3, 2, 1), reverse((1, 2, 3))[true()])
    // Expected: true
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k_seqreversefunc_20() {
    //  Apply a predicate to the result of fn:reverse().
    // XPath: reverse((1, 2, current-time(), 3))[last() - 1]
    // Expected: 2
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k_seqreversefunc_21() {
    //  Apply a predicate to the result of fn:reverse().
    // XPath: reverse((1, 2, current-time(), 3))[last() - 0]
    // Expected: 1
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k2_seqreversefunc_2() {
    //  Apply a confusing amount of ordered/unordered{} expressions.
    // XPath: declare variable $myVar := unordered{ordered{unordered{fn:re
    // Expected: true
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}

#[test]
fn k2_seqreversefunc_3() {
    //  Apply fn:unordered() to fn:reverse().
    // XPath: declare variable $myVar := unordered(fn:reverse((<a/>, <b/>)
    // Expected: true
    // Calls stub function - will fail until fn:reverse is implemented
    let _ = stub_fnreverse();
}
