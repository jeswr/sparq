//! Test chunk 0 for fn:namespace-uri-from-QName
//! Contains 15 tests

use dep::xpath::stub_fnnamespace_uri_from_qname;

#[test]
fn namespaceurifromqnamefunc001() {
    //  Test function fn:namespace-uri-from-QName. Using QName element as input
    // XPath: fn:namespace-uri-from-QName(fn:QName(\"http://www.example.co
    // Expected: http://www.example.com/urn
    // Calls stub function - will fail until fn:namespace-uri-from-QName is implemented
    let _ = stub_fnnamespace_uri_from_qname();
}

#[test]
fn namespaceurifromqnamefunc002() {
    //  Test function fn:namespace-uri-from-QName. Using QName attribute as input
    // XPath: fn:namespace-uri-from-QName(fn:QName(\"http://www.example.co
    // Expected: http://www.example.com/urn
    // Calls stub function - will fail until fn:namespace-uri-from-QName is implemented
    let _ = stub_fnnamespace_uri_from_qname();
}

#[test]
fn namespaceurifromqnamefunc003() {
    //  Test function fn:namespace-uri-from-QName. Using element derived from QName as
    // XPath: fn:namespace-uri-from-QName(fn:QName(\"http://www.example.co
    // Expected: http://www.example.com/urn
    // Calls stub function - will fail until fn:namespace-uri-from-QName is implemented
    let _ = stub_fnnamespace_uri_from_qname();
}

#[test]
fn namespaceurifromqnamefunc004() {
    //  Test function fn:namespace-uri-from-QName. Using attribute derived from QName a
    // XPath: fn:namespace-uri-from-QName(fn:QName(\"http://www.example.co
    // Expected: http://www.example.com/urn
    // Calls stub function - will fail until fn:namespace-uri-from-QName is implemented
    let _ = stub_fnnamespace_uri_from_qname();
}

#[test]
fn namespaceurifromqnamefunc005() {
    //  Test function fn:namespace-uri-from-QName. Check return type of function is xs:
    // XPath: fn:namespace-uri-from-QName(fn:QName(\"http://www.example.co
    // Expected: true
    // Calls stub function - will fail until fn:namespace-uri-from-QName is implemented
    let _ = stub_fnnamespace_uri_from_qname();
}

#[test]
fn namespaceurifromqnamefunc018() {
    //  Test function fn:namespace-uri-from-QName. Explicit call to data to extract sim
    // XPath: fn:namespace-uri-from-QName(fn:QName(\"http://www.example.co
    // Expected: http://www.example.com/urn
    // Calls stub function - will fail until fn:namespace-uri-from-QName is implemented
    let _ = stub_fnnamespace_uri_from_qname();
}

#[test]
fn namespaceurifromqnamefunc019() {
    //  Test function fn:namespace-uri-from-QName. Conditional expression as parameter
    // XPath: fn:namespace-uri-from-QName( if( true() ) then fn:QName(\"ht
    // Expected: http://www.example.com/urn
    // Calls stub function - will fail until fn:namespace-uri-from-QName is implemented
    let _ = stub_fnnamespace_uri_from_qname();
}

#[test]
fn namespaceurifromqnamefunc020() {
    //  Test function fn:namespace-uri-from-QName. Conditional expression as parameter
    // XPath: fn:namespace-uri-from-QName( if( false() ) then fn:QName(\"h
    // Expected: http://www.example.com/urn
    // Calls stub function - will fail until fn:namespace-uri-from-QName is implemented
    let _ = stub_fnnamespace_uri_from_qname();
}

#[test]
fn k_namespaceurifromqnamefunc_3() {
    //  A test whose essence is: `empty(namespace-uri-from-QName( () ))`.
    // XPath: empty(namespace-uri-from-QName( () ))
    // Expected: true
    // Calls stub function - will fail until fn:namespace-uri-from-QName is implemented
    let _ = stub_fnnamespace_uri_from_qname();
}

#[test]
fn k_namespaceurifromqnamefunc_4() {
    //  A test whose essence is: `namespace-uri-from-QName( QName('example.com/', 'pre:
    // XPath: namespace-uri-from-QName( QName(\"example.com/\", \"pre:lnam
    // Expected: true
    // Calls stub function - will fail until fn:namespace-uri-from-QName is implemented
    let _ = stub_fnnamespace_uri_from_qname();
}

#[test]
fn k_namespaceurifromqnamefunc_5() {
    //  A test whose essence is: `namespace-uri-from-QName( QName('example.com/', 'pre:
    // XPath: namespace-uri-from-QName( QName(\"example.com/\", \"pre:lnam
    // Expected: true
    // Calls stub function - will fail until fn:namespace-uri-from-QName is implemented
    let _ = stub_fnnamespace_uri_from_qname();
}

#[test]
fn k2_namespaceurifromqnamefunc_1() {
    //  Test that xmlns declarations in direct element constructors are honored in embe
    // XPath: declare namespace ns = \"http://example.com/ANamespace\";
    // Expected: http://example.com/BNamespace
    // Calls stub function - will fail until fn:namespace-uri-from-QName is implemented
    let _ = stub_fnnamespace_uri_from_qname();
}

#[test]
fn k2_namespaceurifromqnamefunc_2() {
    //  Check that the default element namespace is picked up with computed element con
    // XPath: <e xmlns=\"http://example.com/\"> {namespace-uri-from-QName(
    // Expected: http://example.com/
    // Calls stub function - will fail until fn:namespace-uri-from-QName is implemented
    let _ = stub_fnnamespace_uri_from_qname();
}

#[test]
fn k2_namespaceurifromqnamefunc_3() {
    //  Check that xmlns declarations overrides.
    // XPath: declare default element namespace \"http://example.com/\";
    // Expected: ""
    // Calls stub function - will fail until fn:namespace-uri-from-QName is implemented
    let _ = stub_fnnamespace_uri_from_qname();
}

#[test]
fn k2_namespaceurifromqnamefunc_4() {
    //  Check that xmlns declarations overrides(#2).
    // XPath: declare default element namespace \"http://example.com/\";
    // Expected: [|http://example.com/3]
    // Calls stub function - will fail until fn:namespace-uri-from-QName is implemented
    let _ = stub_fnnamespace_uri_from_qname();
}
