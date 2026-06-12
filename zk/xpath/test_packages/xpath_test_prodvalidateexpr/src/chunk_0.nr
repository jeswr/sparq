//! Test chunk 0 for prod:ValidateExpr
//! Contains 23 tests

use dep::xpath::stub_prodvalidateexpr;

#[test]
fn validateexpr_10() {
    //  Evaluation of a validate expression that a newly strict validated node does not
    // XPath: declare default element namespace \"http://www.w3.org/XQuery
    // Expected: 0
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_11() {
    //  Evaluation of a validate expression that a newly lax validated node does not ha
    // XPath: declare default element namespace \"http://www.w3.org/XQuery
    // Expected: 0
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_12() {
    //  Evaluation of a validate expression that a newly created strict validated node
    // XPath: declare default element namespace \"http://www.w3.org/XQuery
    // Expected: 0
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_13() {
    //  Evaluation of a validate expression that a newly created lax validated node doe
    // XPath: declare default element namespace \"http://www.w3.org/XQuery
    // Expected: 0
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_24() {
    //  Test for the correct typed value when a list of union types is used.
    // XPath: declare namespace lu = 'http://www.w3.org/XQueryTest/ListUni
    // Expected: true true
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_30() {
    // Name : validateexpr-30  Test that validating a document node           (with pro
    // XPath: import schema default element namespace \"http://www.w3.org/
    // Expected: 3
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_31() {
    // Name : validateexpr-31  Test that validating a union of element nodes          d
    // XPath: import schema default element namespace \"http://www.w3.org/
    // Expected: 0
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_32() {
    // Name : validateexpr-32  Test that validating a union of document nodes
    // XPath: import schema default element namespace \"http://www.w3.org/
    // Expected: 0
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_33() {
    // Name : validateexpr-33  Test that validating a union of element and           do
    // XPath: import schema default element namespace \"http://www.w3.org/
    // Expected: 0
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_36() {
    //  Validation using xs:NOTATION
    // XPath: import schema namespace udt = \"http://www.w3.org/XQueryTest
    // Expected: "udt:value1"
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_38() {
    //  Validation using xs:NOTATION
    // XPath: import schema namespace udt = \"http://www.w3.org/XQueryTest
    // Expected: "udt:value1"
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_41() {
    //  Validation using xs:NOTATION
    // XPath: import schema namespace udt = \"http://www.w3.org/XQueryTest
    // Expected: "value1"
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_43() {
    // Type annotation for a complex type with simple content
    // XPath: import schema namespace tc = \"http://www.w3.org/XQueryTest/
    // Expected: true
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validate_sc_1() {
    // XPath: import schema namespace ss=\"http://www.w3.org/query-test/co
    // Expected: true
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_sc_2() {
    // XPath: import schema namespace ss=\"http://www.w3.org/query-test/co
    // Expected: true
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_sc_4() {
    // XPath: import schema namespace ss=\"http://www.w3.org/query-test/co
    // Expected: true
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_sc_5() {
    // XPath: import schema namespace ss=\"http://www.w3.org/query-test/co
    // Expected: true
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_sc_6() {
    // XPath: import schema namespace ss=\"http://www.w3.org/query-test/co
    // Expected: true
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_sc_7() {
    // XPath: import schema namespace ss=\"http://www.w3.org/query-test/co
    // Expected: true
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_sc_8() {
    // XPath: import schema namespace ss=\"http://www.w3.org/query-test/co
    // Expected: true
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn validateexpr_sc_9() {
    // XPath: import schema namespace ss=\"http://www.w3.org/query-test/co
    // Expected: true
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn cbcl_validateexpr_14() {
    // Description : name test against an element with schema validity not known.
    // XPath: declare default element namespace \"http://www.w3.org/XQuery
    // Expected: 0
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}

#[test]
fn cbcl_validateexpr_15() {
    //  Test against schema base type.
    // XPath: import schema namespace cd=\"http://www.cbcl.co.uk/XQueryTes
    // Expected: 1
    // Calls stub function - will fail until prod:ValidateExpr is implemented
    let _ = stub_prodvalidateexpr();
}
