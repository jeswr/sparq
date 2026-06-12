//! Test chunk 7 for prod:OrExpr
//! Contains 7 tests

use dep::xpath::stub_prodorexpr;

#[test]
fn k_logicexpr_43() {
    //  and-expression combined with fn:boolean().
    // XPath: boolean((1, 2, 3, current-time())[1] treat as xs:integer) an
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn k_logicexpr_44() {
    //  and-expression combined with fn:boolean().
    // XPath: true() and boolean((1, 2, 3, current-time())[1] treat as xs:
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn cbcl_logical_and_002() {
    //  test logical and
    // XPath: declare function local:times-table($n as xs:integer) as xs:i
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn cbcl_logical_and_003() {
    //  test logical and
    // XPath: declare function local:factorial($n as xs:integer) as xs:int
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn cbcl_logical_or_001() {
    //  test logical or
    // XPath: declare function local:times-table($n as xs:integer) as xs:i
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn cbcl_logical_or_002() {
    //  test logical or
    // XPath: declare function local:factorial($n as xs:integer) as xs:int
    // Expected: true
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}

#[test]
fn cbcl_logical_or_003() {
    //  test logical or
    // XPath: declare function local:factorial($n as xs:integer) as xs:int
    // Expected: false
    // Calls stub function - will fail until prod:OrExpr is implemented
    let _ = stub_prodorexpr();
}
