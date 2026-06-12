//! Test chunk 1 for prod:LetClause
//! Contains 2 tests

use dep::xpath::stub_prodletclause;

#[test]
fn xquery31keywords4() {
    // Purpose : New XQuery 3.1 keyword can be used as fn-prefixed function name
    // XPath: declare function local:exponent-separator () {126};
    // Expected: true
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn xquery31keywords5() {
    // Purpose : New XQuery 3.1 keyword can be used as unprefixed function name
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}
