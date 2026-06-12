//! Test chunk 0 for fn:json-doc
//! Contains 5 tests

use dep::xpath::stub_fnjson_doc;

#[test]
fn json_doc_010() {
    // More complex JSON document containing escape character
    // XPath: fn:json-doc('http://www.w3.org/qt3/json/escapeText-json')?ke
    // Expected: Data with " within it
    // Calls stub function - will fail until fn:json-doc is implemented
    let _ = stub_fnjson_doc();
}

#[test]
fn json_doc_011() {
    // JSON document containing escape character, using escape options parameter
    // XPath: fn:json-doc('http://www.w3.org/qt3/json/escapeText-json', ma
    // Expected: Data with " within it
    // Calls stub function - will fail until fn:json-doc is implemented
    let _ = stub_fnjson_doc();
}

#[test]
fn json_doc_012() {
    // JSON document containing escape character, using escape options parameter
    // XPath: fn:json-doc('http://www.w3.org/qt3/json/escapeText-json', ma
    // Expected: Data with " within it
    // Calls stub function - will fail until fn:json-doc is implemented
    let _ = stub_fnjson_doc();
}

#[test]
fn json_doc_039() {
    // Unpaired surrogate, fallback
    // XPath: fn:json-doc('http://www.w3.org/qt3/json/stringUnpaired-json'
    // Expected: oh dear DEAD
    // Calls stub function - will fail until fn:json-doc is implemented
    let _ = stub_fnjson_doc();
}

#[test]
fn json_doc_043() {
    // Lower case hex digits are allowed
    // XPath: fn:json-doc('http://www.w3.org/qt3/json/stringHex-json')
    // Expected: \
    // Calls stub function - will fail until fn:json-doc is implemented
    let _ = stub_fnjson_doc();
}
