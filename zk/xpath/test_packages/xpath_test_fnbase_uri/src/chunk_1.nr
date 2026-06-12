//! Test chunk 1 for fn:base-uri
//! Contains 5 tests

use dep::xpath::stub_fnbase_uri;

#[test]
fn k2_baseurifunc_33() {
    //  Ensure recursive resolution works.
    // XPath: <e xml:base=\"http://example.com/ABC/\"> <a xml:base=\"../\"
    // Expected: http://example.com/DEF/file.test
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn cbcl_base_uri_001() {
    // The base-uri property of the copied node, and of each of its descendants,
    // XPath: let $d := document { <root xml:base=\"http://www.w3.org/\"><
    // Expected: http://www.w3.org/TR/xquery
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn cbcl_base_uri_002() {
    //        The base-uri property of the copied node, and of each of its descendants,
    // XPath: let $d := document { <root xml:base=\"http://www.w3.org/\">
    // Expected: http://www.w3.org/TR/xquery
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn cbcl_base_uri_003() {
    //        The base-uri property of the copied node, and of each of its descendants,
    // XPath: let $d := document { <root xml:base=\"http://www.w3.org/\">
    // Expected: http://www.example.org
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn cbcl_base_uri_004() {
    //        The base-uri property of the copied node, and of each of its descendants,
    // XPath: let $d := document { <root xml:base=\"http://www.w3.org/\">
    // Expected: http://www.example.org
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}
