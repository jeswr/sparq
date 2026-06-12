//! Test chunk 0 for fn:parse-xml-fragment
//! Contains 9 tests

use dep::xpath::stub_fnparse_xml_fragment;

#[test]
fn parse_xml_fragment_005() {
    // parse-xml-fragment test - empty string is OK
    // XPath: parse-xml-fragment(\"\")
    // Expected:
    // Calls stub function - will fail until fn:parse-xml-fragment is implemented
    let _ = stub_fnparse_xml_fragment();
}

#[test]
fn parse_xml_fragment_006() {
    // parse-xml-fragment test - text only is OK
    // XPath: parse-xml-fragment(\"vanessa\")
    // Expected: vanessa
    // Calls stub function - will fail until fn:parse-xml-fragment is implemented
    let _ = stub_fnparse_xml_fragment();
}

#[test]
fn parse_xml_fragment_008() {
    // parse-xml-fragment test - whitespace is preserved
    // XPath: parse-xml-fragment(\"  \")
    // Expected:
    // Calls stub function - will fail until fn:parse-xml-fragment is implemented
    let _ = stub_fnparse_xml_fragment();
}

#[test]
fn parse_xml_fragment_011() {
    // parse-xml-fragment test - built-in entity references are OK
    // XPath: string(parse-xml-fragment(codepoints-to-string((38, 108, 116
    // Expected: "<"
    // Calls stub function - will fail until fn:parse-xml-fragment is implemented
    let _ = stub_fnparse_xml_fragment();
}

#[test]
fn parse_xml_fragment_012() {
    // parse-xml-fragment test - numeric character references are OK
    // XPath: string(parse-xml-fragment(codepoints-to-string((38, 35, 51,
    // Expected: "&"
    // Calls stub function - will fail until fn:parse-xml-fragment is implemented
    let _ = stub_fnparse_xml_fragment();
}

#[test]
fn parse_xml_fragment_014() {
    // parse-xml-fragment test - text declaration on its own is OK
    // XPath: parse-xml-fragment(\"<?xml version='1.0' encoding='utf-8'?>\
    // Expected:
    // Calls stub function - will fail until fn:parse-xml-fragment is implemented
    let _ = stub_fnparse_xml_fragment();
}

#[test]
fn parse_xml_fragment_015() {
    // parse-xml-fragment test - text declaration plus text on its own is OK
    // XPath: parse-xml-fragment(\"<?xml version='1.0' encoding='utf-8'?>a
    // Expected: abc
    // Calls stub function - will fail until fn:parse-xml-fragment is implemented
    let _ = stub_fnparse_xml_fragment();
}

#[test]
fn parse_xml_fragment_021() {
    // parse-xml-fragment test - line endings are normalized
    // XPath: string-to-codepoints(parse-xml-fragment(\"a\"||codepoints-to
    // Expected: 97 10 98
    // Calls stub function - will fail until fn:parse-xml-fragment is implemented
    let _ = stub_fnparse_xml_fragment();
}

#[test]
fn parse_xml_fragment_023() {
    // parse-xml-fragment test - result is a document node
    // XPath: parse-xml-fragment(\"<a/>\") instance of document-node()
    // Expected: true
    // Calls stub function - will fail until fn:parse-xml-fragment is implemented
    let _ = stub_fnparse_xml_fragment();
}
