//! Test chunk 0 for prod:EQName
//! Contains 16 tests

use dep::xpath::stub_prodeqname;

#[test]
fn eqname_014() {
    //  EQName in XPath element name
    // XPath: string((//Q{http://www.example.com/AuctionWatch}Start)[1])
    // Expected: 3.00
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}

#[test]
fn eqname_015() {
    //  EQName in XPath variable name
    // XPath: for $Q{http://example.com/ns}x in 1 to 10 return $Q{http://e
    // Expected: 2 3 4 5 6 7 8 9 10 11
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}

#[test]
fn eqname_017() {
    //  Use EQName-style wildcard (XML namespace) to select attribute nodes
    // XPath: string-join(<a foo=\"3\" bar=\"5\" xml:space=\"preserve\"/>
    // Expected: "preserve"
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}

#[test]
fn eqname_020() {
    //  EQName - allowed spaces in EQName, leading spaces are collapsed, Bug 29708
    // XPath: Q{  http://www.w3.org/2005/xpath-functions/math}pi()
    // Expected: math:pi()
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}

#[test]
fn eqname_021() {
    //  EQName - allowed spaces in EQName, trailing spaces are collapsed, Bug 29708
    // XPath: Q{http://www.w3.org/2005/xpath-functions/math   }pi()
    // Expected: math:pi()
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}

#[test]
fn eqname_022() {
    //  EQName - allowed spaces in EQName, leading and trailing spaces are collapsed, B
    // XPath: Q{            http://www.w3.org/2005/xpath-functions/math
    // Expected: math:pi()
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}

#[test]
fn eqname_024() {
    // Tests that the whitespace is properly collapsed by comparing single-ws with mult
    // XPath: for $Q{ urn:foo bar }x in 1 to 5 return $Q{urn:foo   bar}x +
    // Expected: 2 4 6 8 10
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}

#[test]
fn eqname_025() {
    // Tests that the whitespace is properly collapsed for no-namespace variables
    // XPath: for $Q{}T in 1 to 5 return $Q{  }T + $T
    // Expected: 2 4 6 8 10
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}

#[test]
fn eqname_026() {
    //  EQName in XPath element name, variant of eqname-014, tests Bug 29708
    // XPath: string((//Q{  http://www.example.com/AuctionWatch  }Start)[1
    // Expected: 3.00
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}

#[test]
fn eqname_027() {
    // Finding a namespaces attribute using EQName syntax
    // XPath: string(          /Q{  http://www.example.com/AuctionWatch}Au
    // Expected: 13143816
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}

#[test]
fn eqname_029() {
    //  EQName - using entity references in valid URI, using function call syntax with
    // XPath: Q{http:&#x2F;&#x2F;www.w3.org&#x2F;2005&#x2F;xpath-functions
    // Expected: math:pi()
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}

#[test]
fn eqname_029p() {
    //  EQName - using entity references in valid URI, using function call syntax with
    // XPath: Q{http://www.w3.org/2005/xpath-functions/math}pi()
    // Expected: math:pi()
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}

#[test]
fn eqname_030() {
    //  EQName - inline function with BracedURILiteral parameters
    // XPath: let $f := function($Q{''}a, $Q{http:}a) { $Q{''}a + $Q{http:
    // Expected: 42
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}

#[test]
fn eqname_031() {
    //  EQName - inline function reference with BracedURILiteral parameters
    // XPath: let $Q{t:e:s:t}f := function($Q{''}a, $Q{http:}a) { $Q{''}a
    // Expected: 42
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}

#[test]
fn eqname_032() {
    //  EQName - it is legal, but discouraged, to use URI escapes, even if this is an e
    // XPath: let $Q{%7D}a := 12, $Q{}b := 13          return $Q{%7D}a + $
    // Expected: 25
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}

#[test]
fn eqname_033() {
    //  EQName - it is legal, but discouraged, to use URI escapes, testing they are dif
    // XPath: let $Q{foo%20bar}a := 12, $Q{foo bar}a := 13          return
    // Expected: 25
    // Calls stub function - will fail until prod:EQName is implemented
    let _ = stub_prodeqname();
}
