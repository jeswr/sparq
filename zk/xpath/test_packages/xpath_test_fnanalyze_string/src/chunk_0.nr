//! Test chunk 0 for fn:analyze-string
//! Contains 5 tests

use dep::xpath::stub_fnanalyze_string;

#[test]
fn analyzestring_002a() {
    //  analyze-string with empty sequence
    // XPath: count(analyze-string((), \"abc\"))
    // Expected: 1
    // Calls stub function - will fail until fn:analyze-string is implemented
    let _ = stub_fnanalyze_string();
}

#[test]
fn analyzestring_022() {
    //  test string value of result of analyze-string
    // XPath: let $result := analyze-string(\"banana\", \"(b)(anana)\") re
    // Expected: banana
    // Calls stub function - will fail until fn:analyze-string is implemented
    let _ = stub_fnanalyze_string();
}

#[test]
fn analyzestring_023() {
    //  test string value of result of analyze-string
    // XPath: let $result := analyze-string(\"banana\", \"(b)(anana)\") re
    // Expected: banana
    // Calls stub function - will fail until fn:analyze-string is implemented
    let _ = stub_fnanalyze_string();
}

#[test]
fn analyzestring_026() {
    //  '.' does NOT match CR in default mode
    // XPath: exactly-one(fn:analyze-string(concat('Mary', codepoints-to-s
    // Expected: concat('Mary', codepoints-to-string(13),
    // Calls stub function - will fail until fn:analyze-string is implemented
    let _ = stub_fnanalyze_string();
}

#[test]
fn analyzestring_027() {
    //  '.' does NOT match CR in default mode
    // XPath: exactly-one(fn:analyze-string(concat('Mary', codepoints-to-s
    // Expected: concat('y', codepoints-to-string(13), 'J
    // Calls stub function - will fail until fn:analyze-string is implemented
    let _ = stub_fnanalyze_string();
}
