//! Test chunk 0 for fn:for-each
//! Contains 1 tests

use dep::xpath::stub_fnfor_each;

#[test]
fn for_each_010() {
    // for-each function - promotion from anyURI to string
    // XPath: for-each(namespace-uri-from-QName(function-name(xs:int#1)),
    // Expected: 'HTTP://WWW.W3.ORG/2001/XMLSCHEMA'
    // Calls stub function - will fail until fn:for-each is implemented
    let _ = stub_fnfor_each();
}
