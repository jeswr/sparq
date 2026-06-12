//! Test chunk 0 for prod:NodeTest
//! Contains 4 tests

use dep::xpath::stub_prodnodetest;

#[test]
fn k2_nodetest_2() {
    //  Check the child count of an empty direct element constructor.
    // XPath: count(<a/>/node())
    // Expected: 0
    // Calls stub function - will fail until prod:NodeTest is implemented
    let _ = stub_prodnodetest();
}

#[test]
fn k2_nodetest_32() {
    //  We evaluate to only atomic values, although the static type does not reflect th
    // XPath: <e> <a/> <b/> </e>/(if(position() = 10) then (<e/>, .) else
    // Expected: 4
    // Calls stub function - will fail until prod:NodeTest is implemented
    let _ = stub_prodnodetest();
}

#[test]
fn k2_nodetest_42() {
    //  Use fn:last() with nested predicates(#2).
    // XPath: (4, 5)[position() = 2][last() = 1]
    // Expected: 5
    // Calls stub function - will fail until prod:NodeTest is implemented
    let _ = stub_prodnodetest();
}

#[test]
fn k2_nodetest_43() {
    //  Use fn:last() with nested predicates(#3).
    // XPath: (4, 5)[position() = 2][last() = 1][last() = 1][last()]
    // Expected: 5
    // Calls stub function - will fail until prod:NodeTest is implemented
    let _ = stub_prodnodetest();
}
