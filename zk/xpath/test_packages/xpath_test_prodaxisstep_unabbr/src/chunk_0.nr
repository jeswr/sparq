//! Test chunk 0 for prod:AxisStep.unabbr
//! Contains 2 tests

use dep::xpath::stub_prodaxisstep_unabbr;

#[test]
fn unabbreviatedsyntax_3() {
    //  Evaluate selecting all text node children of the context node
    // XPath: for $h in (/works[1]/employee[2]) return $h/child::text()
    // Expected: Text data from Employee[2]
    // Calls stub function - will fail until prod:AxisStep.unabbr is implemented
    let _ = stub_prodaxisstep_unabbr();
}

#[test]
fn unabbreviatedsyntax_14() {
    //  Evaluate self (self::hours)- Select the context node, if it is an 'employee', o
    // XPath: for $h in (/works[1]/employee[1]) return fn:count(($h/self::
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.unabbr is implemented
    let _ = stub_prodaxisstep_unabbr();
}
