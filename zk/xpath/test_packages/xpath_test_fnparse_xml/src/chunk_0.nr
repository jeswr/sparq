//! Test chunk 0 for fn:parse-xml
//! Contains 1 tests

use dep::xpath::stub_fnparse_xml;

#[test]
fn parse_xml_007() {
    // parse-xml test - base URI of result
    // XPath: base-uri(parse-xml(\"<a>foo</a>\")) eq static-base-uri()
    // Expected: true
    // Calls stub function - will fail until fn:parse-xml is implemented
    let _ = stub_fnparse_xml();
}
