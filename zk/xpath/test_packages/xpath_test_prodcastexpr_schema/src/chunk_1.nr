//! Test chunk 1 for prod:CastExpr.schema
//! Contains 23 tests

use dep::xpath::stub_prodcastexpr_schema;

#[test]
fn user_defined_7() {
    //  use constructor function for user defined type and check if it        is castab
    // XPath: import schema namespace myType=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn user_defined_8() {
    //  Use constructor function to evaluate if two user defined types can be        ca
    // XPath: import schema namespace myType=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn user_defined_9() {
    //  Use constructor function to evaluate if two user defined types are castable to
    // XPath: import schema namespace myType=\"http://www.w3.org/XQueryTes
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn user_defined_10() {
    //  Use constructor function to use addition operation on two user defined types.
    // XPath: import schema namespace myType=\"http://www.w3.org/XQueryTes
    // Expected: 32
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_uniontype_12() {
    // Constructor function for a namespace-sensitive union type
    // XPath: import schema namespace s=\"http://www.w3.org/XQueryTest/uni
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_uniontype_16() {
    // Constructor function for union type with facets
    // XPath: import schema namespace s=\"http://www.w3.org/XQueryTest/uni
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_uniontype_19() {
    // Cast to a union type from one of its member type
    // XPath: import schema namespace s=\"http://www.w3.org/XQueryTest/uni
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_uniontype_20() {
    // Cast to a union type from one of its member type
    // XPath: import schema namespace s=\"http://www.w3.org/XQueryTest/uni
    // Expected: a
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_uniontype_21() {
    // Cast to a union type from a subtype of one of its member type
    // XPath: import schema namespace s=\"http://www.w3.org/XQueryTest/uni
    // Expected: zorobastic
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_uniontype_25() {
    // Cast QName to a union type including QName as a member:         prefix does not
    // XPath: import schema namespace s=\"http://www.w3.org/XQueryTest/uni
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_uniontype_28() {
    // Cast to a union of list types (namespace sensitive)
    // XPath: import schema namespace s=\"http://www.w3.org/XQueryTest/uni
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_uniontype_29() {
    // Constructor function to a union of list types (namespace sensitive)
    // XPath: import schema namespace s=\"http://www.w3.org/XQueryTest/uni
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_uniontype_30() {
    // Function literal for constructor to a union of list types (namespace sensitive)
    // XPath: import schema namespace s=\"http://www.w3.org/XQueryTest/uni
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_uniontype_31() {
    // Function lookup for constructor to a union of list types (namespace sensitive)
    // XPath: import schema namespace s=\"http://www.w3.org/XQueryTest/uni
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_uniontype_32() {
    // Type of constructor function to a union of list types (namespace sensitive)
    // XPath: import schema namespace s=\"http://www.w3.org/XQueryTest/uni
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_uniontype_33() {
    // Cast to a union type from a node whose typed value is one of its member types
    // XPath: import schema namespace s=\"http://www.w3.org/XQueryTest/uni
    // Expected: space
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_listtype_21() {
    // Cast to a user-defined list whose member type is namespace-sensitive union.
    // XPath: import schema namespace s=\"http://www.w3.org/XQueryTest/uni
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_listtype_27() {
    // Type of constructor function for casting to list
    // XPath: let $f as function(xs:anyAtomicType) as xs:NMTOKEN* := xs:NM
    // Expected: 3
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_listtype_28() {
    // Castable to list: true
    // XPath: \"a b c\" castable as xs:NMTOKENS
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_listtype_29() {
    // Castable to list: false
    // XPath: \"a b 12\" castable as xs:IDREFS
    // Expected: false
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_listtype_30() {
    // Castable to list: false (violates minLength)
    // XPath: \" \" castable as xs:NMTOKENS
    // Expected: false
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_listtype_31() {
    // Castable to list: false (violates pattern)
    // XPath: import schema namespace s=\"http://www.w3.org/XQueryTest/uni
    // Expected: false
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}

#[test]
fn castas_listtype_32() {
    // Castable to list: true (satisfies pattern after whitespace normalization)
    // XPath: import schema namespace s=\"http://www.w3.org/XQueryTest/uni
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr.schema is implemented
    let _ = stub_prodcastexpr_schema();
}
