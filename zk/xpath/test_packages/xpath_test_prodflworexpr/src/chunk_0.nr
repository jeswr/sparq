//! Test chunk 0 for prod:FLWORExpr
//! Contains 9 tests

use dep::xpath::stub_prodflworexpr;

#[test]
fn forexprtype025() {
    //  FLWOR expressions with type declaration (attribute type). Interleave type integ
    // XPath: import schema default element namespace \"http://www.example
    // Expected: 10 20
    // Calls stub function - will fail until prod:FLWORExpr is implemented
    let _ = stub_prodflworexpr();
}

#[test]
fn forexprtype026() {
    //  FLWOR expressions with type declaration (attribute type). Match user defined si
    // XPath: import schema default element namespace \"http://www.example
    // Expected: ok
    // Calls stub function - will fail until prod:FLWORExpr is implemented
    let _ = stub_prodflworexpr();
}

#[test]
fn forexprtype027() {
    //  FLWOR expressions with type declaration (attribute type). Wildcard match on use
    // XPath: import schema default element namespace \"http://www.example
    // Expected: ok
    // Calls stub function - will fail until prod:FLWORExpr is implemented
    let _ = stub_prodflworexpr();
}

#[test]
fn forexprtype048() {
    //  FLWOR expressions with type declaration. Non-trivial value type promotion: anyA
    // XPath: import schema default element namespace \"http://www.example
    // Expected: 10
    // Calls stub function - will fail until prod:FLWORExpr is implemented
    let _ = stub_prodflworexpr();
}

#[test]
fn forexprtype049() {
    //  FLWOR expressions with type declaration. Non-trivial value type promotion: Inte
    // XPath: import schema default element namespace \"http://www.example
    // Expected: 10 20
    // Calls stub function - will fail until prod:FLWORExpr is implemented
    let _ = stub_prodflworexpr();
}

#[test]
fn forexprtype050() {
    //  FLWOR expressions with type declaration. Non-trivial value type promotion: Unio
    // XPath: import schema default element namespace \"http://www.example
    // Expected: 20
    // Calls stub function - will fail until prod:FLWORExpr is implemented
    let _ = stub_prodflworexpr();
}

#[test]
fn forexprtype051() {
    //  FLWOR expressions with type declaration. Non-trivial value type promotion: List
    // XPath: import schema default element namespace \"http://www.example
    // Expected: entity1 entity2 entity3
    // Calls stub function - will fail until prod:FLWORExpr is implemented
    let _ = stub_prodflworexpr();
}

#[test]
fn forexprtype052() {
    //  FLWOR expressions with type declaration. Non-trivial value type promotion: User
    // XPath: import schema default element namespace \"http://www.example
    // Expected: ok
    // Calls stub function - will fail until prod:FLWORExpr is implemented
    let _ = stub_prodflworexpr();
}

#[test]
fn forexprtype053() {
    //  FLWOR expressions with type declaration. Non-trivial value type promotion: User
    // XPath: import schema default element namespace \"http://www.example
    // Expected: ok
    // Calls stub function - will fail until prod:FLWORExpr is implemented
    let _ = stub_prodflworexpr();
}
