//! Test chunk 0 for fn:json-to-xml
//! Contains 1 tests

use dep::xpath::stub_fnjson_to_xml;

#[test]
fn json_to_xml_041() {
    // Base URI of the result
    // XPath: static-base-uri() eq base-uri(fn:json-to-xml('true'))
    // Expected: true
    // Calls stub function - will fail until fn:json-to-xml is implemented
    let _ = stub_fnjson_to_xml();
}
